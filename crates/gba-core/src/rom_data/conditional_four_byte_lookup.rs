//! Source-authenticated conditional EUR lookup projection; guest arithmetic stays original.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e6dbc;
pub const RECORD_COUNT: usize = 74;
pub const SOURCE_SHA256: &str =
    "76c83105709e84c7852de2d88b19c7b50b7862b443b57a478de4854ebb3a0ea1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub first: u16,
    pub second: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lookup {
    pub source_offset: u32,
    pub records: Vec<Record>,
}

impl Lookup {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid conditional EUR four-byte lookup projection".into());
        }
        Ok(())
    }

    /// The bounded projection does not imply a check in the original guest.
    pub fn select(&self, index: u32) -> Option<&Record> {
        self.records.get(index as usize)
    }

    /// Original 32-bit address arithmetic, including wrap, without a guest cap.
    pub fn encoded_view(index: u32) -> u32 {
        (0x08000000u32 + SOURCE_OFFSET).wrapping_add(index.wrapping_mul(4))
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(|record| {
            record.first.to_le_bytes().into_iter().chain(record.second.to_le_bytes())
        }).collect())
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let n = offset.checked_sub(self.source_offset)? as usize;
        let row = self.records.get(n / 4)?;
        Some(if n % 4 < 2 { row.first.to_le_bytes()[n % 2] } else { row.second.to_le_bytes()[n % 2] })
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Conditional lookup differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Lookup,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional lookup requires exact EUR ROM; USA pending".into());
        }
        let source = &rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + RECORD_COUNT * 4];
        if ffta_rom::digest(source) != SOURCE_SHA256 {
            return Err("Conditional lookup source fingerprint mismatch".into());
        }
        let decoded = Lookup {
            source_offset: SOURCE_OFFSET,
            records: source.chunks_exact(4).map(|v| Record {
                first: u16::from_le_bytes([v[0], v[1]]),
                second: u16::from_le_bytes([v[2], v[3]]),
            }).collect(),
        };
        decoded.validate()?;
        Ok(Self {
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: source.len() as u32,
                sha256: SOURCE_SHA256.into(),
                role: "conditional-four-byte-lookup".into(),
            }],
            decoded,
        })
    }
}
