//! One authenticated EUR descriptor for the observed tilemap.
use super::{observed_tilemap, script39_registry_poses::PriorSources, Resource, SourceRange, OBSERVED_TILEMAP_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3a4b70;
pub const SOURCE_LENGTH: usize = 8;
pub const SOURCE_SHA256: &str = "1a6f836689bf342c63de1ecab160ca4e134120ae3670782998696e02242f7a05";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTilemapDescriptor {
    pub source_offset: u32,
    pub tilemap_pointer: u32,
    pub parameter16: u16,
    pub parameter8: u8,
    pub opaque_byte: u8,
}

impl ObservedTilemapDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.tilemap_pointer != 0x0800_0000 + observed_tilemap::SOURCE_OFFSET {
            return Err("Invalid observed tilemap descriptor source or pointer".into());
        }
        Ok(())
    }

    pub fn validate_dependencies(&self, tilemap: &observed_tilemap::ObservedTilemap) -> Result<(), String> {
        self.validate()?;
        tilemap.validate()?;
        if self.tilemap_pointer != 0x0800_0000 + tilemap.source_offset {
            return Err("Observed tilemap descriptor dependency mismatch".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> [u8; SOURCE_LENGTH] {
        let mut bytes = [0; SOURCE_LENGTH];
        bytes[..4].copy_from_slice(&self.tilemap_pointer.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.parameter16.to_le_bytes());
        bytes[6] = self.parameter8;
        bytes[7] = self.opaque_byte;
        bytes
    }
}

pub struct Catalog {
    pub decoded: ObservedTilemapDescriptor,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed tilemap descriptor requires the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0xdb30a..0xdb326]) != "0ac3d1f008eb9ff01ae13ba635e54c40af5f41ddcc4150750a63c5dd100a7174"
            || u32::from_le_bytes(rom[0x3a4bf0..0x3a4bf4].try_into().unwrap()) != 0x083a4b70 {
            return Err("Observed tilemap descriptor original reader mismatch".into());
        }
        let map = previous.iter().find(|resource| resource.id == OBSERVED_TILEMAP_ID).ok_or("Missing observed tilemap dependency")?;
        if !map.sources.iter().any(|source| source.offset == observed_tilemap::SOURCE_OFFSET && source.length as usize == observed_tilemap::SOURCE_LENGTH) {
            return Err("Observed tilemap descriptor source dependency mismatch".into());
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Observed tilemap descriptor source hash mismatch".into()); }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_LENGTH)? {
            return Err("Observed tilemap descriptor source already has an owner".into());
        }
        let decoded = ObservedTilemapDescriptor {
            source_offset: SOURCE_OFFSET,
            tilemap_pointer: u32::from_le_bytes(bytes[..4].try_into().unwrap()),
            parameter16: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            parameter8: bytes[6], opaque_byte: bytes[7],
        };
        decoded.validate()?;
        if decoded.source_bytes() != bytes { return Err("Observed tilemap descriptor reconstruction mismatch".into()); }
        Ok(Self { decoded, sources: vec![SourceRange { offset: SOURCE_OFFSET, length: SOURCE_LENGTH as u32, sha256: ffta_rom::digest(bytes), role: "tilemap-descriptor".into() }] })
    }
}
