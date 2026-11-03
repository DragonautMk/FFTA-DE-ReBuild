//! Raw localized transfer views; live selectors, queues and copy work stay guest code.
use super::{SourceRange, ui_graphics::table_selected::Table};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub mod dynamic_fragments;

pub const ROOTS: [u32; 7] = [
    0x71e5f4, 0x71e608, 0x71e66c, 0x71e694, 0x71e6a8, 0x71e6e4, 0x71e6f8,
];
pub const VIEW_BYTES: [usize; 7] = [224, 160, 768, 1600, 2048, 2432, 384];
pub const VIEW_OFFSETS: [u32; 7] = [0, 0, 0, 0, 0, 0, 0x200];
pub const SOURCE_SHA256: &str = "e11ef4d5f16feea8ef90e7fce58056e19d689004b7ca973270285e3cd31b8f80";
pub type SourceSpan = (u32, Vec<u8>, String);
fn invalid() -> String {
    "Invalid declared localized raw transfer views".into()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankView {
    pub bank_offset: u32,
    pub relative_offset: u32,
    pub words: Vec<u32>,
}
impl BankView {
    pub fn source_offset(&self) -> Option<u32> {
        self.bank_offset.checked_add(self.relative_offset)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.words.iter().flat_map(|v| v.to_le_bytes()).collect()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(self.source_offset()?)? as usize;
        self.words
            .get(relative / 4)
            .map(|v| v.to_le_bytes()[relative % 4])
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTransfers {
    pub tables: Vec<Table>,
    pub banks: Vec<BankView>,
}
impl RawTransfers {
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.tables.len() != ROOTS.len() || self.banks.len() != 35 {
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
                format!("raw-table/{:08x}", table.source_offset),
            ));
            for pointer in &table.pointers {
                if linked
                    .insert(pointer.bank_offset, (VIEW_OFFSETS[i], VIEW_BYTES[i]))
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
        for bank in &self.banks {
            let at = bank.source_offset().ok_or_else(invalid)?;
            if bank.bank_offset & 3 != 0
                || at & 3 != 0
                || linked.remove(&bank.bank_offset)
                    != Some((
                        bank.relative_offset,
                        bank.words.len().checked_mul(4).ok_or_else(invalid)?,
                    ))
                || usize::try_from(at)
                    .ok()
                    .and_then(|v| v.checked_add(bank.words.len() * 4))
                    .is_none_or(|end| end > 0x02000000)
            {
                return Err(invalid());
            }
            spans.push((
                at,
                bank.source_bytes(),
                format!("raw-bank-view/{:08x}", bank.bank_offset),
            ));
        }
        if !linked.is_empty()
            || self
                .banks
                .windows(2)
                .any(|v| v[0].bank_offset >= v[1].bank_offset)
        {
            return Err(invalid());
        }
        spans.sort_by_key(|v| v.0);
        if spans
            .windows(2)
            .any(|v| u64::from(v[0].0) + v[0].1.len() as u64 > u64::from(v[1].0))
        {
            return Err(invalid());
        }
        Ok(spans)
    }
    /// Editable shape and original EUR provenance are checked independently.
    pub fn validate(&self) -> Result<(), String> {
        self.source_spans().map(|_| ())
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        for table in &self.tables {
            if let Some(i) = offset.checked_sub(table.source_offset).filter(|i| *i < 20) {
                let p = &table.pointers[i as usize / 4];
                return p.source_bytes().ok().map(|v| v[i as usize % 4]);
            }
        }
        self.banks.iter().find_map(|b| b.byte_at(offset))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Raw transfers differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: RawTransfers,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Raw transfers require exact EUR ROM; USA pending".into());
        }
        for &(at, len, sha) in WINDOWS {
            if ffta_rom::digest(&rom[at..at + len]) != sha {
                return Err("Raw transfer reader fingerprint mismatch".into());
            }
        }
        let mut tables = Vec::new();
        let mut banks = std::collections::BTreeMap::new();
        for (i, source_offset) in ROOTS.into_iter().enumerate() {
            let pointers = std::array::from_fn(|j| {
                let at = source_offset as usize + j * 4;
                super::ui_graphics::BankPointer {
                    source_offset: at as u32,
                    bank_offset: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                        .wrapping_sub(0x08000000),
                }
            });
            for p in &pointers {
                let at = p
                    .bank_offset
                    .checked_add(VIEW_OFFSETS[i])
                    .ok_or_else(invalid)? as usize;
                let bytes = rom
                    .get(at..at.checked_add(VIEW_BYTES[i]).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                if banks
                    .insert(
                        p.bank_offset,
                        BankView {
                            bank_offset: p.bank_offset,
                            relative_offset: VIEW_OFFSETS[i],
                            words: bytes
                                .chunks_exact(4)
                                .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                                .collect(),
                        },
                    )
                    .is_some()
                {
                    return Err(invalid());
                }
            }
            tables.push(Table {
                source_offset,
                pointers,
            });
        }
        let decoded = RawTransfers {
            tables,
            banks: banks.into_values().collect(),
        };
        let spans = decoded.source_spans()?;
        if ffta_rom::digest(
            &spans
                .iter()
                .flat_map(|v| v.1.iter().copied())
                .collect::<Vec<_>>(),
        ) != SOURCE_SHA256
        {
            return Err("Raw transfer source fingerprint mismatch".into());
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
        0x23a90,
        96,
        "ba325d1b164ae7d4bf2e88e73d3d922acc21794643a7cabc212fd794c0660f66",
    ),
    (
        0x23bcc,
        72,
        "f753bab2968a3edc13e548badc4735ddcf4ad0f5ae828e3852547d305ada3778",
    ),
    (
        0x3e600,
        128,
        "80b163132fcfa7f22566456340bda23eb963905152d726f34d3435918424c007",
    ),
    (
        0x4fcf0,
        136,
        "fc880dd82bbe9305ac4eee1ae9d92ef1d6bb3ffe41c1dfc7c680bf31e2fad11e",
    ),
    (
        0x12edb0,
        112,
        "daeb4e791f79cf24ec5ce22fd0194785dd1db3751981b1285b5e5fc9285cd588",
    ),
    (
        0x12ee84,
        48,
        "9463df5d4859ea634db7bfd95a00e3c5ea7b40294b1541ee8341bcfab3363a03",
    ),
    (
        0x2c7f0,
        144,
        "692a6f60c9e183e1229bcb7737c7781b81e6fa5c92287ca3598ee7cff27b7aee",
    ),
    (
        0x2cb70,
        176,
        "7faf16c3a7fb9679de261636c421ffc41d75e8845de12bdb3e45fed55d96e7f5",
    ),
    (
        0x2cc00,
        112,
        "09aee7db1ef517dc68acf245d3115708da929934277b72a513186bb77fb1303c",
    ),
    (
        0xc8e40,
        144,
        "857b164482573761ac5c0a4fbf982d43aeb011f921a74d9330ec56dbec42bf2d",
    ),
    (
        0xc8f10,
        120,
        "c187d74921d473d67e2b78dd629466fbe8efe4ba20f018b8d72a12da687a6d6a",
    ),
    (
        0xc8fc0,
        96,
        "d0a7d7b48ec24ccdcd09bea9429c164a8cc057bd1e4c746b3f7780c76c381922",
    ),
    (
        0xc9090,
        96,
        "91b35743b6f6a2a8da66e1541e6dcadff3df5cc0b7b1c32a2a557df5f989f73c",
    ),
    (
        0xc9100,
        96,
        "6da8b10962943a54bde4d8f040b5818bebf8102ee47f2f70a4254acb85939056",
    ),
    (
        0x4fc70,
        128,
        "f915c5a41348af70826e74639f2a427b6daec326cfce0be54672e3c649e1b757",
    ),
    (
        0x2c880,
        64,
        "2bcb617f7a2fd8751bb814583e38987fb9e0d48b447043a503e290f904b0d8d3",
    ),
    (
        0xc9060,
        48,
        "421a30020c6659fbe839f18904b20f3ca4f5eff7bbb5d1ed4eee552da524743e",
    ),
    (
        0xc90f0,
        16,
        "a9a6adcf7f2bdd714a53525178e925d79deb209937f1e1271a04dccd20b58dec",
    ),
    (
        0x145220,
        24,
        "5fc994fd728f1067e7423aafb639091183037454b4e6dd1a7e3391bfc2f42983",
    ),
    (
        0x4fbf0,
        104,
        "7e06abb3ad1e7c34f8706c76f0ca0484be37f10957e78af144eb09c276908f21",
    ),
    (
        0xc8d10,
        304,
        "d91e23e7cd6d7624403304cbe2c867f893dc2222b67e72eedd73b19747019966",
    ),
    (
        0xc91a0,
        176,
        "2089c1d4392d0e660862af4f4cf181aa9931749eb734a9d108681699e9ee1486",
    ),
    (
        0x3e5e8,
        24,
        "d51c204e445717fc045e080ef12bd06b4c52daa8e2ee8f4e95b3d9d2b8de58de",
    ),
    (
        0x23c14,
        150,
        "1b399d39f4b80fadaf4ff6244cca0e1f95a4ab7865bf06e278c3b907fd495d51",
    ),
    (
        0x12ee20,
        70,
        "e5ad9f8ac0c63cda068cf68b69aab1997ab21b58335a801850e90ebcb08f3ac4",
    ),
    (
        0xb14,
        440,
        "7a69a026b47a561f7e36eebadaafd48a478d6aac57f3f8bb629089e853d07311",
    ),
    (
        0x7c0,
        380,
        "4b656a463accbf9314d1247ff00cbb1ca8a263aca52c8ab19099aae9e5fdb481",
    ),
    (
        0x145c14,
        180,
        "06d86a220a9616c4491d01bd44efa48491008eee5f0b00034e1013190cf8d4b1",
    ),
    (
        0xd968a8,
        90,
        "4fcfc248aaad9bc841403a73d13c5ba8661fadf84b8bf3e2e2414882d78e1993",
    ),
    (
        0x2790,
        190,
        "de0bf86100484d1c7f6348cb4d8e459a428576428bcd28c4bc10bed0927b32b5",
    ),
    (
        0x2c8c0,
        200,
        "de1469f13056e1dbfab83f8ab788536121f9f28d0334ad7de5f80fce69323aba",
    ),
    (
        0xa64,
        190,
        "5ad2800f5af90166d42896d75003021cff5a40e343cdf5fc1d343387b1a75281",
    ),
];

#[cfg(test)]
mod tests;
