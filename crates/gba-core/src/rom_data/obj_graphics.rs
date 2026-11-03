//! EUR custom graphics banks linked to OBJ uploads. Live scratch work stays RAM.
use super::{custom_lz, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
mod roots;
pub use roots::{
    DirectReference, TableReference, BANK_ROOTS, DIRECT_REFERENCES, TABLE_REFERENCES, TABLE_ROOTS,
};
pub const MAX_OUTPUT: usize = 32 * 1024;
pub const MAX_ENCODED: usize = 128 * 1024;
const ROM_LIMIT: u32 = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graphic {
    pub source_offset: u32,
    /// Exact little-endian decoded words. Internal subregions may be copied,
    /// interpreted or transformed by original code; no RGBA/padding is invented.
    pub words: Vec<u32>,
    pub encoded_source: Vec<u8>,
}
impl Graphic {
    pub fn decode(input: &[u8], source_offset: u32) -> Result<Self, String> {
        let block = custom_lz::decode(input, MAX_OUTPUT, MAX_ENCODED)?;
        if block.bytes.len() % 32 != 0 {
            return Err("OBJ custom bank is not a complete word/tile-unit extent".into());
        }
        let bank = Self {
            source_offset,
            words: block
                .bytes
                .chunks_exact(4)
                .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                .collect(),
            encoded_source: input[..block.consumed].to_vec(),
        };
        if source_offset
            .checked_add(block.consumed as u32)
            .is_none_or(|end| end > ROM_LIMIT)
        {
            return Err("OBJ custom bank source overflow".into());
        }
        Ok(bank)
    }
    pub fn decoded_bytes(&self) -> Vec<u8> {
        self.words.iter().flat_map(|v| v.to_le_bytes()).collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.words.len() > MAX_OUTPUT / 4 || self.encoded_source.len() > MAX_ENCODED {
            return Err("OBJ custom graphics words/source exceed malformed-input guard".into());
        }
        if &Self::decode(&self.encoded_source, self.source_offset)? != self {
            return Err("OBJ custom bank words/source disagree".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointerTable {
    pub source_offset: u32,
    /// Ordered ROM offsets; the guest projection restores 08000000 pointers.
    pub bank_offsets: Vec<u32>,
}
impl PointerTable {
    pub fn select(&self, index: u8) -> Option<u32> {
        self.bank_offsets.get(index as usize).copied()
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        if self.source_offset & 3 != 0
            || self.bank_offsets.is_empty()
            || self.bank_offsets.len() > 256
            || self.bank_offsets.iter().any(|&v| v >= ROM_LIMIT)
            || self
                .source_offset
                .checked_add((self.bank_offsets.len() * 4) as u32)
                .is_none_or(|end| end > ROM_LIMIT)
        {
            return Err("Invalid OBJ graphics selector table".into());
        }
        Ok(self
            .bank_offsets
            .iter()
            .flat_map(|&v| (v + 0x08000000).to_le_bytes())
            .collect())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graphics {
    pub banks: Vec<Graphic>,
    pub pointer_tables: Vec<PointerTable>,
}
impl Graphics {
    pub fn validate(&self) -> Result<(), String> {
        if self.banks.is_empty()
            || self.banks.len() > 256
            || self.pointer_tables.len() > 64
            || self
                .banks
                .windows(2)
                .any(|v| v[0].source_offset >= v[1].source_offset)
            || self
                .pointer_tables
                .windows(2)
                .any(|v| v[0].source_offset >= v[1].source_offset)
        {
            return Err("Invalid OBJ graphics catalog ordering/count".into());
        }
        let mut spans = Vec::new();
        for bank in &self.banks {
            bank.validate()?;
            spans.push((bank.source_offset, bank.encoded_source.len()));
        }
        for table in &self.pointer_tables {
            let bytes = table.source_bytes()?;
            if table.bank_offsets.iter().any(|v| {
                self.banks
                    .binary_search_by_key(v, |b| b.source_offset)
                    .is_err()
            }) {
                return Err("OBJ graphics selector has no matching bank".into());
            }
            spans.push((table.source_offset, bytes.len()));
        }
        spans.sort_unstable();
        if spans
            .windows(2)
            .any(|v| v[0].0 as usize + v[0].1 > v[1].0 as usize)
        {
            return Err("Overlapping OBJ graphics source ranges".into());
        }
        Ok(())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("OBJ custom banks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn word(rom: &[u8], at: u32) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        rom.get(at as usize..at as usize + 4)
            .ok_or("Truncated OBJ graphics source word")?
            .try_into()
            .unwrap(),
    ))
}
fn half(rom: &[u8], at: u32) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        rom.get(at as usize..at as usize + 2)
            .ok_or("Truncated OBJ graphics instruction")?
            .try_into()
            .unwrap(),
    ))
}
fn literal(rom: &[u8], reader: u32, address: u32, value: u32) -> Result<(), String> {
    let h = half(rom, reader)?;
    if h & 0xf800 != 0x4800
        || ((reader + 4) & !3) + u32::from(h & 255) * 4 != address
        || word(rom, address)? != value
    {
        return Err("OBJ graphics root literal mismatch".into());
    }
    Ok(())
}
fn call(rom: &[u8], at: u32, target: u32) -> Result<(), String> {
    let h = half(rom, at)?;
    let l = half(rom, at + 2)?;
    let d = (((u32::from(h & 2047) << 12) | (u32::from(l & 2047) << 1)) as i32) << 9 >> 9;
    if h & 0xf800 != 0xf000
        || l & 0xf800 != 0xf800
        || at.wrapping_add(4).wrapping_add_signed(d) != target
    {
        return Err("OBJ graphics original consumer mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Graphics,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("OBJ custom graphics require exact EUR ROM; USA pending".into());
        }
        for r in DIRECT_REFERENCES {
            literal(rom, r.reader, r.literal, 0x08000000 + r.source_offset)?;
            call(rom, r.call, 0x54a0)?;
            call(rom, r.transfer_call, r.transfer_target)?;
        }
        for r in TABLE_REFERENCES {
            literal(rom, r.reader, r.literal, 0x08000000 + r.table_offset)?;
            call(rom, r.call, 0x54a0)?;
        }
        let mut banks = Vec::new();
        for &at in BANK_ROOTS {
            banks.push(Graphic::decode(
                rom.get(at as usize..).ok_or("Truncated OBJ bank")?,
                at,
            )?);
        }
        let mut pointer_tables = Vec::new();
        for &at in TABLE_ROOTS {
            pointer_tables.push(PointerTable {
                source_offset: at,
                bank_offsets: (0..5u32)
                    .map(|i| {
                        word(rom, at + i * 4)?
                            .checked_sub(0x08000000)
                            .ok_or("Invalid OBJ graphics ROM pointer".into())
                    })
                    .collect::<Result<_, String>>()?,
            });
        }
        let decoded = Graphics {
            banks,
            pointer_tables,
        };
        decoded.validate()?;
        let mut sources = Vec::new();
        for bank in &decoded.banks {
            sources.push(SourceRange {
                offset: bank.source_offset,
                length: bank.encoded_source.len() as u32,
                sha256: ffta_rom::digest(&bank.encoded_source),
                role: format!("obj-custom-bank/{:08x}", bank.source_offset),
            });
        }
        for table in &decoded.pointer_tables {
            let bytes = table.source_bytes()?;
            if bytes
                != rom[table.source_offset as usize..table.source_offset as usize + bytes.len()]
            {
                return Err("OBJ graphics table projection mismatch".into());
            }
            sources.push(SourceRange {
                offset: table.source_offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: format!("obj-graphics-selector/{:08x}", table.source_offset),
            });
        }
        Ok(Self { decoded, sources })
    }
}
