//! Independently rooted raw-language tables; no guest bank fallback is invented.
use super::{BankPointer, Graphic};
use crate::{ffta_rom, rom_data::SourceRange};
use serde::{Deserialize, Serialize};

pub const ROOTS: [u32; 8] = [
    0x71e590, 0x71e5a4, 0x71e5b8, 0x71e5cc, 0x71e644, 0x71e680, 0x71e6bc, 0x71e6d0,
];
pub const OUTPUT_BYTES: [usize; 8] = [1472, 2688, 768, 4928, 1920, 7488, 1344, 2368];
pub const SOURCE_SHA256: &str = "df69408f23205472246a37ab071670913bdb2ceebe151a91b18fadf9e297d5b4";
pub type SourceSpan = (u32, Vec<u8>, String);
fn invalid() -> String {
    "Invalid declared table-selected UI graphics".into()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub source_offset: u32,
    pub pointers: [BankPointer; 5],
}
impl Table {
    pub fn select(&self, language: u8) -> Option<&BankPointer> {
        self.pointers.get(usize::from(language))
    }
    /// The original live unsigned-byte domain is independent of physical ownership.
    pub fn encoded_view(&self, language: u8) -> u32 {
        (0x08000000 + self.source_offset).wrapping_add(u32::from(language) * 4)
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        for (i, p) in self.pointers.iter().enumerate() {
            if p.source_offset
                != self
                    .source_offset
                    .checked_add(i as u32 * 4)
                    .ok_or_else(invalid)?
            {
                return Err(invalid());
            }
            bytes.extend(p.source_bytes()?);
        }
        Ok(bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableGraphics {
    pub tables: Vec<Table>,
    pub banks: Vec<Graphic>,
}
impl TableGraphics {
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.tables.len() != ROOTS.len() || self.banks.len() != 40 {
            return Err(invalid());
        }
        let mut linked = std::collections::BTreeMap::new();
        let mut spans = Vec::new();
        for (i, table) in self.tables.iter().enumerate() {
            if table.source_offset != ROOTS[i] {
                return Err(invalid());
            }
            spans.push((
                table.source_offset,
                table.source_bytes()?,
                format!("ui-custom-table/{:08x}", table.source_offset),
            ));
            for p in &table.pointers {
                if linked.insert(p.bank_offset, OUTPUT_BYTES[i]).is_some() {
                    return Err(invalid());
                }
            }
        }
        for bank in &self.banks {
            bank.validate()?;
            if linked.remove(&bank.source_offset) != Some(bank.words.len() * 4) {
                return Err(invalid());
            }
            spans.push((
                bank.source_offset,
                bank.encoded_source.clone(),
                format!("ui-custom-bank/{:08x}", bank.source_offset),
            ));
        }
        if !linked.is_empty()
            || self
                .banks
                .windows(2)
                .any(|a| a[0].source_offset >= a[1].source_offset)
        {
            return Err(invalid());
        }
        spans.sort_by_key(|v| v.0);
        if spans
            .windows(2)
            .any(|a| a[0].0 as usize + a[0].1.len() > a[1].0 as usize)
        {
            return Err(invalid());
        }
        Ok(spans)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.source_spans().map(|_| ())
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.source_spans()
            .ok()?
            .into_iter()
            .find_map(|(base, bytes, _)| bytes.get(offset.checked_sub(base)? as usize).copied())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Table graphics differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: TableGraphics,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Table graphics require exact EUR ROM; USA pending".into());
        }
        for &(start, len, sha) in WINDOWS {
            if ffta_rom::digest(&rom[start..start + len]) != sha {
                return Err("Table graphics reader fingerprint mismatch".into());
            }
        }
        let mut tables = Vec::new();
        let mut banks = std::collections::BTreeMap::new();
        for source_offset in ROOTS {
            let pointers = std::array::from_fn(|i| {
                let at = source_offset as usize + i * 4;
                BankPointer {
                    source_offset: at as u32,
                    bank_offset: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                        .wrapping_sub(0x08000000),
                }
            });
            for p in &pointers {
                if p.bank_offset as usize >= rom.len() {
                    return Err(invalid());
                }
                banks.insert(
                    p.bank_offset,
                    Graphic::decode(&rom[p.bank_offset as usize..], p.bank_offset)?,
                );
            }
            tables.push(Table {
                source_offset,
                pointers,
            });
        }
        let decoded = TableGraphics {
            tables,
            banks: banks.into_values().collect(),
        };
        let spans = decoded.source_spans()?;
        let bytes: Vec<_> = spans
            .iter()
            .flat_map(|(_, b, _)| b.iter().copied())
            .collect();
        if ffta_rom::digest(&bytes) != SOURCE_SHA256 {
            return Err("Table graphics source fingerprint mismatch".into());
        }
        let sources = spans
            .into_iter()
            .map(|(offset, bytes, role)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}
const WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x2c8c0,
        200,
        "de1469f13056e1dbfab83f8ab788536121f9f28d0334ad7de5f80fce69323aba",
    ),
    (
        0x1399d0,
        160,
        "7cf17820f99d04497d3c5f9078669ffaa1956b09dc2bb29a369696bf35c69665",
    ),
    (
        0x30688,
        104,
        "49b725f5f7e4d4a169350c71b545c26d0ca49ea482b4b09d3eb658f8d8370c00",
    ),
    (
        0x95140,
        80,
        "520d4e013ab375dc1d6edcaf199e7525e22c2c35f9b2cee7c4a13ff2fee6b423",
    ),
    (
        0x95190,
        120,
        "234c9289e11e8b0e2d7f41ac14192c59b8e21005abd1ca671e58623982a99257",
    ),
    (
        0x139a70,
        184,
        "65028a410f457cfa826c980384271ed010e238e5d4897ddb7f7a228edf054782",
    ),
    (
        0x23b00,
        16,
        "4056b398ca2b8d9ae67065989b1551c96a265589c14e37368cdf8b9093e37875",
    ),
    (
        0x55650,
        176,
        "19c8b6f1636e6762bf6cef6dd84521cd0dbbeea496df52895e0e5901b932fb37",
    ),
    (
        0x12be10,
        96,
        "158744779efa7b4c9e34ca2bc87d75ba51e1629879f3600495d9816183ac0c0c",
    ),
    (
        0x12c460,
        224,
        "3e3bcf4352eababfaa8559734e573d1994e0677eedfef6e0b5c63dd01874f780",
    ),
    (
        0x30600,
        136,
        "4f0e4936c00a79ccd39f02b46af1f869aa3257ecd0e1e3d14ca78b3d162aea47",
    ),
    (
        0x12bd90,
        128,
        "787eb52474d51d1d8ca39e1f3322db5872a34def750ba21ee7876f5290d92d29",
    ),
    (
        0x95208,
        92,
        "5057cc5cb769f64696158491207abd2ef62492796cffc3992a99853098e0904e",
    ),
    (
        0x23b10,
        200,
        "9ddcfca1a9392ca47e26553ed2a83586518c294c6c98ebe37bb340bd49704f1e",
    ),
];

#[cfg(test)]
mod tests;
