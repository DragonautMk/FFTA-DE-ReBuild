//! Bounded EUR signed-byte projection used by a White Flowers scene reader.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x397b0a;
pub const SOURCE_LENGTH: usize = 48;
pub const ROOT_LITERAL: u32 = 0x083c14;
pub const SOURCE_ROLE: &str = "white-flowers-signed-byte-projection";
const SOURCE_HASH: &str = "a586d9892013a2a3c8dce2bd1629b4f6c611fed7b53e6dbb4a6ba542f13db020";
const READER_HASH: &str = "208a76e2e183a664cccff5f1f2cfb33f1ead1a8f8ec3ed3e9370915c1c818d2e";
const HELPER_HASH: &str = "83f774210a8e58ef8f1448f2fccae7675b8bce1c6b13cd1cac8e1333c081b42e";

/// Physical bytes are signed only after the original reader's sign extension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhiteFlowersSignedBytes {
    pub source_offset: u32,
    pub values: Vec<i8>,
}
impl WhiteFlowersSignedBytes {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.values.len() != SOURCE_LENGTH {
            return Err("Invalid White Flowers signed-byte projection source or length".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.values.iter().map(|&value| value as u8).collect())
    }
    /// Unselected addresses, including out-of-range initial cursors, keep their prior owner.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        let index = offset.checked_sub(self.source_offset)? as usize;
        self.values.get(index).map(|&value| (value as u8, index))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("White Flowers signed-byte projection differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: WhiteFlowersSignedBytes,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("White Flowers signed-byte projection requires exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[0x83bf2..0x83c38]) != READER_HASH
            || ffta_rom::digest(&rom[0x1464ec..0x1465a2]) != HELPER_HASH
            || u32::from_le_bytes(rom[ROOT_LITERAL as usize..ROOT_LITERAL as usize + 4].try_into().unwrap())
                != 0x0800_0000 + SOURCE_OFFSET
        {
            return Err("White Flowers signed-byte reader, helper or root fingerprint mismatch".into());
        }
        if !previous.is_empty() && !previous.iter().any(|resource| {
            resource.id == THUMB_PC_LITERALS_ID && resource.sources.iter().any(|source| {
                source.offset == ROOT_LITERAL && source.length == 4
            })
        }) {
            return Err("Missing separately owned White Flowers signed-byte root".into());
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_HASH || PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_LENGTH)? {
            return Err("White Flowers signed-byte source fingerprint or owner mismatch".into());
        }
        let decoded = WhiteFlowersSignedBytes {
            source_offset: SOURCE_OFFSET,
            values: bytes.iter().map(|&value| value as i8).collect(),
        };
        if decoded.source_bytes()? != bytes {
            return Err("White Flowers signed-byte reconstruction mismatch".into());
        }
        let sources = vec![SourceRange { offset: SOURCE_OFFSET, length: SOURCE_LENGTH as u32,
            sha256: SOURCE_HASH.into(), role: SOURCE_ROLE.into() }];
        Ok(Self { decoded, sources })
    }
}
