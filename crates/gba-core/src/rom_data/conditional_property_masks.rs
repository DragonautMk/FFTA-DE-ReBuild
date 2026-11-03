//! Authenticated EUR mask projection; the original guest selector stays unbounded.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e0b44;
pub const RECORD_COUNT: usize = 35;
pub const SOURCE_SHA256: &str =
    "89fa534fce962ae4e48c9812619d9f41467ce37442cfb598d4beebf81fc6e065";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Masks {
    pub source_offset: u32,
    pub words: Vec<u32>,
}

impl Masks {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.words.len() != RECORD_COUNT {
            return Err("Invalid conditional EUR property-selected mask projection".into());
        }
        Ok(())
    }

    /// Projection bounds are not a new check in the original guest.
    pub fn select(&self, index: u32) -> Option<u32> {
        self.words.get(index as usize).copied()
    }

    /// Original 32-bit address arithmetic, including wrap, with no guest cap.
    pub fn encoded_view(index: u32) -> u32 {
        (0x08000000u32 + SOURCE_OFFSET).wrapping_add(index.wrapping_mul(4))
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.words.iter().flat_map(|word| word.to_le_bytes()).collect())
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let n = offset.checked_sub(self.source_offset)? as usize;
        Some(self.words.get(n / 4)?.to_le_bytes()[n % 4])
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Conditional property masks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Masks,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional property masks require exact EUR ROM; USA pending".into());
        }
        let source = &rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + RECORD_COUNT * 4];
        if ffta_rom::digest(source) != SOURCE_SHA256 {
            return Err("Conditional property mask source fingerprint mismatch".into());
        }
        let decoded = Masks {
            source_offset: SOURCE_OFFSET,
            words: source.chunks_exact(4).map(|v| u32::from_le_bytes(v.try_into().unwrap())).collect(),
        };
        decoded.validate()?;
        Ok(Self {
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: source.len() as u32,
                sha256: SOURCE_SHA256.into(),
                role: "conditional-property-masks".into(),
            }],
            decoded,
        })
    }
}
