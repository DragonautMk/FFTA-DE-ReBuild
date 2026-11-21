//! European script lookup window between the command and selector directories.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3ad16e;
pub const SOURCE_BYTES: usize = 6;
pub const ADDITIONAL_ROOT: u32 = 0x126468;
const SOURCE_SHA256: &str = "9a64cb724d9e1396c3cb4392a93028bb09fbae75e3e7383bd44d77ce97eba189";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptByteLookup3ad {
    pub source_offset: u32,
    pub values: [u8; SOURCE_BYTES],
}

impl ScriptByteLookup3ad {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET {
            return Err("Invalid EUR script byte lookup source".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![
            (
                self.source_offset,
                self.values.to_vec(),
                "script-byte-lookup-window",
            ),
            (
                ADDITIONAL_ROOT,
                (0x0800_0000u32 + self.source_offset).to_le_bytes().to_vec(),
                "script-byte-lookup-second-root",
            ),
        ])
    }

    /// Physical bytes only: guest subtraction and outside indices stay original.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, bool)> {
        if let Some(n) = offset.checked_sub(self.source_offset) {
            if let Some(&value) = self.values.get(n as usize) {
                return Some((value, n as usize, false));
            }
        }
        if let Some(n) = offset.checked_sub(ADDITIONAL_ROOT) {
            if n < 4 {
                return Some((
                    (0x0800_0000u32 + self.source_offset).to_le_bytes()[n as usize],
                    0,
                    true,
                ));
            }
        }
        None
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Script byte lookup differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ScriptByteLookup3ad,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Script byte lookup requires the exact EUR ROM; USA pending".into());
        }
        for (start, end, expected) in [
            (
                0x1263b2,
                0x1263bc,
                "66ad6cd766efd1fb551b8a057e3b6b77f8962e22b9982de8ce5d989e52368b69",
            ),
            (
                0x126454,
                0x12645e,
                "73198daa3914520992b33e97e3fadfe43172204da73cd0fda5098165d1d6bc91",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Script byte lookup reader fingerprint mismatch".into());
            }
        }
        for (reader, literal) in [(0x1263b2u32, 0x1263c4u32), (0x126454, ADDITIONAL_ROOT)] {
            let op = u16::from_le_bytes(
                rom[reader as usize..reader as usize + 2]
                    .try_into()
                    .unwrap(),
            );
            let root = u32::from_le_bytes(
                rom[literal as usize..literal as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            if op & 0xf800 != 0x4800
                || ((reader + 4) & !3) + u32::from(op & 255) * 4 != literal
                || root != 0x0800_0000 + SOURCE_OFFSET
            {
                return Err("Script byte lookup reader root mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize;
        let raw = &rom[start..start + SOURCE_BYTES];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Script byte lookup source fingerprint mismatch".into());
        }
        let decoded = ScriptByteLookup3ad {
            source_offset: SOURCE_OFFSET,
            values: raw.try_into().unwrap(),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Script byte lookup source overlap or reconstruction mismatch".into());
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
