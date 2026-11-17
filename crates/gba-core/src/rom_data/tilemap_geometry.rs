//! Source-backed EUR tilemap geometry projections; guest indexing stays original.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x371464;
pub const SOURCE_BYTES: usize = 48;
pub const SOURCE_SHA256: &str = "955f43accb6baf3c8d6743c1838ecd5724f154ff1da17648f93f409b6f65b6e7";
const ROLES: [&str; 6] = [
    "constructor-tilemap-bytes", "constructor-width", "constructor-height",
    "state-tilemap-bytes", "state-width", "state-height",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub source_offset: u32,
    pub tilemap_bytes: [u16; 4],
    pub width_pixels: [u16; 4],
    pub height_pixels: [u16; 4],
}
impl Bank {
    fn group(&self, index: usize) -> Option<&[u16; 4]> {
        match index {
            0 => Some(&self.tilemap_bytes),
            1 => Some(&self.width_pixels),
            2 => Some(&self.height_pixels),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TilemapGeometry {
    pub constructor: Bank,
    pub state: Bank,
}
impl TilemapGeometry {
    pub fn validate(&self) -> Result<(), String> {
        if self.constructor.source_offset != SOURCE || self.state.source_offset != SOURCE + 24 {
            return Err("Invalid EUR tilemap geometry source offsets".into());
        }
        Ok(())
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = usize::try_from(offset.checked_sub(SOURCE)?).ok()?;
        let bank = match relative / 24 {
            0 => &self.constructor,
            1 => &self.state,
            _ => return None,
        };
        let local = relative % 24;
        let value = bank.group(local / 8)?[local % 8 / 2];
        Some(value.to_le_bytes()[local % 2])
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        (0..SOURCE_BYTES).map(|at| self.byte_at(SOURCE + at as u32).ok_or_else(|| "Invalid tilemap geometry byte".into())).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Tilemap geometry differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: TilemapGeometry,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Tilemap geometry requires the exact EUR ROM".into());
        }
        for (start, end, digest) in [
            (0x1c7a0, 0x1c7d6, "ca55125f8429cbdc99bcc97f65b8239b42f3888d302bfa1ec8b97dbd8a9ab9c9"),
            (0x1f208, 0x1f23c, "0b2a972496f366750f9885ef7fa412d3b93071625cb21f67e78b47cba2b65f5c"),
            (0x1f570, 0x1f598, "a5a23f48d4634b35bd37ea9cdc68761b1ae43ef3d2b27592f61520978a3b388e"),
            (0x1f66e, 0x1f690, "49e9207ed96c76fb7ddc9d16810bdafa496446b3a4f909822419adb9acae17f1"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != digest {
                return Err("Tilemap geometry reader fingerprint mismatch".into());
            }
        }
        for (literal, offset) in [(0x1c828, 0), (0x1c82c, 8), (0x1c830, 16), (0x1f248, 24), (0x1f240, 32), (0x1f244, 40)] {
            let actual = u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap());
            if actual != 0x0800_0000 + SOURCE + offset {
                return Err("Tilemap geometry source literal mismatch".into());
            }
        }
        let start = SOURCE as usize;
        let raw = &rom[start..start + SOURCE_BYTES];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Tilemap geometry source fingerprint mismatch".into());
        }
        let bank = |offset: usize| {
            let group = |group: usize| std::array::from_fn(|i| {
                let at = offset + group * 8 + i * 2;
                u16::from_le_bytes(raw[at..at + 2].try_into().unwrap())
            });
            Bank { source_offset: SOURCE + offset as u32, tilemap_bytes: group(0), width_pixels: group(1), height_pixels: group(2) }
        };
        let decoded = TilemapGeometry { constructor: bank(0), state: bank(24) };
        decoded.validate()?;
        let sources = ROLES.iter().enumerate().map(|(i, role)| SourceRange {
            offset: SOURCE + (i * 8) as u32, length: 8,
            sha256: ffta_rom::digest(&raw[i * 8..i * 8 + 8]), role: (*role).into(),
        }).collect();
        Ok(Self { decoded, sources })
    }
}
