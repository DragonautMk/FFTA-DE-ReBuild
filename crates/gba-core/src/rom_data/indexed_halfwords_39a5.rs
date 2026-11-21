//! European halfwords selected by the original reader at 080e21e4.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x39a5c8;
pub const SOURCE_BYTES: usize = 6;
pub const SOURCE_SHA256: &str = "82e5abba625d96f08c42442ee3764adbc4bce616f9cde9118504575de0cbb1ac";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedHalfwords39a5 {
    pub source_offset: u32,
    pub values: [u16; 3],
}

impl IndexedHalfwords39a5 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET {
            return Err("Invalid EUR indexed halfword source".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.values.iter().flat_map(|v| v.to_le_bytes()).collect())
    }

    /// Physical bytes only: no additional limit is imposed on guest indices.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        let n = offset.checked_sub(self.source_offset)? as usize;
        let value = n / 2;
        Some((self.values.get(value)?.to_le_bytes()[n % 2], value))
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Indexed halfwords differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: IndexedHalfwords39a5,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Indexed halfwords require the exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[0xe2176..0xe21f4])
            != "6b5fad3d1d57d873a79c28ae8411babc1c75abeed7dc69e5d448a88234827e23"
        {
            return Err("Indexed halfword reader fingerprint mismatch".into());
        }
        let opcode = u16::from_le_bytes(rom[0xe21de..0xe21e0].try_into().unwrap());
        let root = u32::from_le_bytes(rom[0xe21f4..0xe21f8].try_into().unwrap());
        if opcode & 0xf800 != 0x4800
            || ((0xe21de + 4) & !3) + u32::from(opcode & 255) * 4 != 0xe21f4
            || root != 0x0800_0000 + SOURCE_OFFSET
        {
            return Err("Indexed halfword reader root mismatch".into());
        }
        let start = SOURCE_OFFSET as usize;
        let raw = &rom[start..start + SOURCE_BYTES];
        if ffta_rom::digest(raw) != SOURCE_SHA256
            || PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_BYTES)?
        {
            return Err("Indexed halfword source hash or ownership mismatch".into());
        }
        let decoded = IndexedHalfwords39a5 {
            source_offset: SOURCE_OFFSET,
            values: std::array::from_fn(|i| u16::from_le_bytes([raw[i * 2], raw[i * 2 + 1]])),
        };
        if decoded.source_bytes()? != raw {
            return Err("Indexed halfword source reconstruction mismatch".into());
        }
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: SOURCE_BYTES as u32,
                sha256: SOURCE_SHA256.into(),
                role: "indexed-halfwords-39a5c8".into(),
            }],
        })
    }
}
