//! Opaque EUR bytes addressed conditionally by two original language readers.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3ae472;
pub const SOURCE_SHA256: &str = "96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7";

/// The pair has no established meaning and is not part of the five-language tables.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalLanguagePair {
    pub source_offset: u32,
    pub opaque_bytes: [u8; 2],
}

impl ConditionalLanguagePair {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET {
            return Err("Invalid conditional language pair offset".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE_OFFSET,
            self.opaque_bytes.to_vec(),
            "conditional-language-index-opaque",
        )])
    }
}

pub struct Catalog {
    pub decoded: ConditionalLanguagePair,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional language pair requires the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0x14205a..0x1420c0])
            != "fca258e1b0151885137954deb000e55f9eaa49b27e174a36b892e20cbcbdaa87"
        {
            return Err("Conditional language reader instruction mismatch".into());
        }
        for (at, expected) in [(0x142118, 0x083ae42cu32), (0x142128, 0x083ae436u32)] {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != expected {
                return Err("Conditional language reader root mismatch".into());
            }
        }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, 2)? {
            return Err("Conditional language pair already has an owner".into());
        }
        let at = SOURCE_OFFSET as usize;
        let decoded = ConditionalLanguagePair {
            source_offset: SOURCE_OFFSET,
            opaque_bytes: rom[at..at + 2].try_into().unwrap(),
        };
        if ffta_rom::digest(&decoded.opaque_bytes) != SOURCE_SHA256 {
            return Err("Conditional language pair source hash mismatch".into());
        }
        let sources = decoded
            .source_spans()?
            .into_iter()
            .map(|(offset, bytes, role)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: role.into(),
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}
