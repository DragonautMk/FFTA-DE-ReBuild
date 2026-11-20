//! Authenticated EUR constructor template; live records remain guest-owned.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x14d1f0;
pub const SOURCE_LENGTH: usize = 64;
pub const SOURCE_SHA256: &str = "9588ca0950e95399d9d9756f23099af738bba0600a5e731c7ebf171fad3ac15f";

/// Preserve the unclassified fields as little-endian words.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructorTemplate64 {
    pub source_offset: u32,
    pub words: [u32; 16],
}

impl ConstructorTemplate64 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET {
            return Err("Invalid constructor template source offset".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> Vec<u8> {
        self.words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }
}

pub struct Catalog {
    pub decoded: ConstructorTemplate64,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Constructor template requires the exact EUR ROM".into());
        }
        // Includes the original literal load, fixed copy length, and BL.
        if ffta_rom::digest(&rom[0x10a80..0x10ab8])
            != "504c7119c1f23a34f8ecd3d02f2ae836b08451beff5d1459ce74181add25e434"
            || u32::from_le_bytes(rom[0x10ad0..0x10ad4].try_into().unwrap())
                != 0x0814d1f0
        {
            return Err("Constructor template original reader mismatch".into());
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 {
            return Err("Constructor template source hash mismatch".into());
        }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_LENGTH)? {
            return Err("Constructor template source already has an owner".into());
        }
        let decoded = ConstructorTemplate64 {
            source_offset: SOURCE_OFFSET,
            words: std::array::from_fn(|i| {
                u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())
            }),
        };
        decoded.validate()?;
        if decoded.source_bytes() != bytes {
            return Err("Constructor template reconstruction mismatch".into());
        }
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: SOURCE_LENGTH as u32,
                sha256: ffta_rom::digest(bytes),
                role: "constructor-template".into(),
            }],
        })
    }
}
