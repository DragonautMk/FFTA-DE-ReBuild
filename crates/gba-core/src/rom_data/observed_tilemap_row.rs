//! Authenticated EUR rows, including one selecting the observed tilemap descriptor.
use super::{observed_tilemap_descriptor, script39_registry_poses::PriorSources, Resource, SourceRange, OBSERVED_TILEMAP_DESCRIPTOR_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3a4bec;
pub const SOURCE_LENGTH: usize = 12;
pub const SOURCE_SHA256: &str = "493de1159defc6d682a5f8316cfc031421b96f41571004142d7c4ce0a64c93be";
/// Four observed adjacent rows; their pointed descriptors remain outside this owner.
pub const ADDITIONAL_SPECS: [(u32, u32, &str); 4] = [
    (0x3a4bbc, 0x083a4b18, "65193c653f3482da17830fa6fd78543bfc027487e1e8296e3620a456527dac6e"),
    (0x3a4bc8, 0x083a4b20, "5148af3b96dd567b348fe4610449d09aaddf22339745c34fff4f82ff5efd4e93"),
    (0x3a4bd4, 0x083a4b28, "da0d3b59b3b7b60aeca542678a03095c95012e053ff4441feefecd27c71e807b"),
    (0x3a4be0, 0x083a4b30, "0eb16da90a8b576d2e27b253c34c8bc2dae13593c75127ba2d183278670e7322"),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalRow {
    pub source_offset: u32,
    pub field0: u8,
    pub field1: u8,
    pub field2: u16,
    pub descriptor_pointer: u32,
    pub field8: u32,
}

impl AdditionalRow {
    pub fn source_bytes(&self) -> [u8; SOURCE_LENGTH] {
        row_bytes(self.field0, self.field1, self.field2, self.descriptor_pointer, self.field8)
    }
}

fn row_bytes(field0: u8, field1: u8, field2: u16, pointer: u32, field8: u32) -> [u8; SOURCE_LENGTH] {
    let mut bytes = [0; SOURCE_LENGTH];
    bytes[0] = field0;
    bytes[1] = field1;
    bytes[2..4].copy_from_slice(&field2.to_le_bytes());
    bytes[4..8].copy_from_slice(&pointer.to_le_bytes());
    bytes[8..12].copy_from_slice(&field8.to_le_bytes());
    bytes
}

/// Preserve the other observed fields without assigning unproved semantics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTilemapRow {
    pub source_offset: u32,
    pub field0: u8,
    pub field1: u8,
    pub field2: u16,
    pub descriptor_pointer: u32,
    pub field8: u32,
    pub additional_rows: [AdditionalRow; 4],
}

impl ObservedTilemapRow {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.descriptor_pointer != 0x0800_0000 + observed_tilemap_descriptor::SOURCE_OFFSET {
            return Err("Invalid observed tilemap row source or pointer".into());
        }
        for (row, &(offset, pointer, _)) in self.additional_rows.iter().zip(&ADDITIONAL_SPECS) {
            if row.source_offset != offset || row.descriptor_pointer != pointer {
                return Err("Invalid additional observed tilemap row source or pointer".into());
            }
        }
        Ok(())
    }

    pub fn validate_dependencies(&self, descriptor: &observed_tilemap_descriptor::ObservedTilemapDescriptor) -> Result<(), String> {
        self.validate()?;
        descriptor.validate()?;
        if self.descriptor_pointer != 0x0800_0000 + descriptor.source_offset {
            return Err("Observed tilemap row dependency mismatch".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> [u8; SOURCE_LENGTH] {
        row_bytes(self.field0, self.field1, self.field2, self.descriptor_pointer, self.field8)
    }

    pub fn source_spans(&self) -> Vec<(u32, [u8; SOURCE_LENGTH], &'static str)> {
        let mut spans = vec![(self.source_offset, self.source_bytes(), "tilemap-row")];
        for (row, role) in self.additional_rows.iter().zip(["tilemap-row-0", "tilemap-row-1", "tilemap-row-2", "tilemap-row-3"]) {
            spans.push((row.source_offset, row.source_bytes(), role));
        }
        spans
    }
}

pub struct Catalog {
    pub decoded: ObservedTilemapRow,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed tilemap row requires the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0xdb30a..0xdb326]) != "0ac3d1f008eb9ff01ae13ba635e54c40af5f41ddcc4150750a63c5dd100a7174"
            || ffta_rom::digest(&rom[0xdb49c..0xdb4b6]) != "e6b4dac3741f0c0927816d86e21bcd54ea1748a18b5e09d4456c0e1494c61192" {
            return Err("Observed tilemap row original reader mismatch".into());
        }
        let descriptor = previous.iter().find(|resource| resource.id == OBSERVED_TILEMAP_DESCRIPTOR_ID).ok_or("Missing observed tilemap descriptor dependency")?;
        if !descriptor.sources.iter().any(|source| source.offset == observed_tilemap_descriptor::SOURCE_OFFSET && source.length as usize == observed_tilemap_descriptor::SOURCE_LENGTH) {
            return Err("Observed tilemap row source dependency mismatch".into());
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Observed tilemap row source hash mismatch".into()); }
        for &(offset, _, hash) in &ADDITIONAL_SPECS {
            let at = offset as usize;
            if ffta_rom::digest(&rom[at..at + SOURCE_LENGTH]) != hash {
                return Err("Additional observed tilemap row source hash mismatch".into());
            }
        }
        let additional_rows = std::array::from_fn(|i| {
            let at = ADDITIONAL_SPECS[i].0 as usize;
            let row = &rom[at..at + SOURCE_LENGTH];
            AdditionalRow {
                source_offset: at as u32, field0: row[0], field1: row[1],
                field2: u16::from_le_bytes(row[2..4].try_into().unwrap()),
                descriptor_pointer: u32::from_le_bytes(row[4..8].try_into().unwrap()),
                field8: u32::from_le_bytes(row[8..12].try_into().unwrap()),
            }
        });
        let decoded = ObservedTilemapRow {
            source_offset: SOURCE_OFFSET,
            field0: bytes[0], field1: bytes[1],
            field2: u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
            descriptor_pointer: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            field8: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            additional_rows,
        };
        decoded.validate()?;
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans() {
            if prior.contains(offset, bytes.len())? { return Err("Observed tilemap row source already has an owner".into()); }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes { return Err("Observed tilemap row reconstruction mismatch".into()); }
            sources.push(SourceRange { offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role: role.into() });
        }
        Ok(Self { decoded, sources })
    }
}
