//! Typed EUR display-control constants read by the original guest routines.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x14d2dc;
pub const SOURCE_SHA256: &str = "5c5502e34c5e00cb3c556b9bd4f9e8582c6dec845b71c15669f362f30d020760";

/// The pointer values name live IWRAM structures; the structures are not static resources.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayControlTables {
    pub source_offset: u32,
    pub first_halfwords: [u16; 4],
    pub second_halfwords: [u16; 4],
    pub iwram_pointers: [u32; 4],
}

impl DisplayControlTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET
            || self
                .iwram_pointers
                .iter()
                .any(|&p| !(0x0300_0000..0x0300_8000).contains(&p) || p & 3 != 0)
        {
            return Err("Invalid display-control table shape".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![
            (
                SOURCE_OFFSET,
                self.first_halfwords
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
                "first-halfwords",
            ),
            (
                SOURCE_OFFSET + 8,
                self.second_halfwords
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
                "second-halfwords",
            ),
            (
                SOURCE_OFFSET + 16,
                self.iwram_pointers
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
                "iwram-pointers",
            ),
        ])
    }
}

pub struct Catalog {
    pub decoded: DisplayControlTables,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Display-control tables require the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (
                0x1f440,
                0x1f59e,
                "4fa4f8095bbd66c6a1b28ba69c69c84b04acbb097745ccd3a06580873b6864ea",
            ),
            (
                0x147dc0,
                0x147e22,
                "aa822d873bad381a62e3c3e07b06003a41beea59bc3edc791f7e316eb08b7ebb",
            ),
            (
                0x1f5b4,
                0x1f5c0,
                "0189c2355c4f4fa8f86858e87e3336ebdf04a863d0947b53858dffef986e09d2",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Display-control original reader mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize;
        if ffta_rom::digest(&rom[start..start + 32]) != SOURCE_SHA256 {
            return Err("Display-control table source hash mismatch".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let decoded = DisplayControlTables {
            source_offset: SOURCE_OFFSET,
            first_halfwords: std::array::from_fn(|i| half(start + 2 * i)),
            second_halfwords: std::array::from_fn(|i| half(start + 8 + 2 * i)),
            iwram_pointers: std::array::from_fn(|i| word(start + 16 + 4 * i)),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? {
                return Err("Display-control table source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Display-control table reconstruction mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
