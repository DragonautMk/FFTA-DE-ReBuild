//! Authenticated EUR rows, including one selecting the observed tilemap descriptor.
use super::{observed_tilemap_descriptor, script39_registry_poses::PriorSources, Resource, SourceRange, OBSERVED_TILEMAP_DESCRIPTOR_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3a4bec;
pub const SOURCE_LENGTH: usize = 12;
pub const SOURCE_SHA256: &str = "493de1159defc6d682a5f8316cfc031421b96f41571004142d7c4ce0a64c93be";
/// Four observed adjacent rows; their pointed descriptors have a separate owner.
pub const ADDITIONAL_SPECS: [(u32, u32, &str); 4] = [
    (0x3a4bbc, 0x083a4b18, "65193c653f3482da17830fa6fd78543bfc027487e1e8296e3620a456527dac6e"),
    (0x3a4bc8, 0x083a4b20, "5148af3b96dd567b348fe4610449d09aaddf22339745c34fff4f82ff5efd4e93"),
    (0x3a4bd4, 0x083a4b28, "da0d3b59b3b7b60aeca542678a03095c95012e053ff4441feefecd27c71e807b"),
    (0x3a4be0, 0x083a4b30, "0eb16da90a8b576d2e27b253c34c8bc2dae13593c75127ba2d183278670e7322"),
];
/// Three observed transfer rows linked to the additional LZ descriptors.
pub const TRANSFER_SPECS: [(u32, u32, &str); 3] = [
    (0x39978c, 0x08399774, "71be4dc4a18dc5f3f733ebb5c1cd6a3477fca2a44248f4cb44657aab5aa7c7fa"),
    (0x399798, 0x0839977c, "d1c81f1ecd310b05c65f35ec59458e745c667089419b4f0673609919fbfef66c"),
    (0x3997a4, 0x08399784, "d08c14b7c02d89bfe70cb58edab0318a1133d32273a62cd629ddb7e312fd33c1"),
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
    pub transfer_rows: [AdditionalRow; 3],
}

impl ObservedTilemapRow {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.descriptor_pointer != 0x0800_0000 + observed_tilemap_descriptor::SOURCE_OFFSET {
            return Err("Invalid observed tilemap row source or pointer".into());
        }
        for (row, &(offset, pointer, _)) in self.additional_rows.iter()
            .chain(&self.transfer_rows).zip(ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS)) {
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
        for (row, entry) in self.additional_rows.iter().chain(&self.transfer_rows)
            .zip(descriptor.additional_descriptors.iter().chain(&descriptor.transfer_descriptors)) {
            if row.descriptor_pointer != 0x0800_0000 + entry.source_offset {
                return Err("Additional tilemap row descriptor dependency mismatch".into());
            }
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
        for (row, role) in self.transfer_rows.iter().zip(["transfer-row-0", "transfer-row-1", "transfer-row-2"]) {
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
        for &(_, pointer, _) in ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS) {
            if !descriptor.sources.iter().any(|source| source.offset == pointer - 0x0800_0000
                && source.length as usize == observed_tilemap_descriptor::SOURCE_LENGTH) {
                return Err("Additional tilemap row source dependency mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Observed tilemap row source hash mismatch".into()); }
        for &(offset, _, hash) in ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS) {
            let at = offset as usize;
            if ffta_rom::digest(&rom[at..at + SOURCE_LENGTH]) != hash {
                return Err("Additional observed tilemap row source hash mismatch".into());
            }
        }
        let read_row = |offset: u32| {
            let at = offset as usize;
            let row = &rom[at..at + SOURCE_LENGTH];
            AdditionalRow {
                source_offset: at as u32, field0: row[0], field1: row[1],
                field2: u16::from_le_bytes(row[2..4].try_into().unwrap()),
                descriptor_pointer: u32::from_le_bytes(row[4..8].try_into().unwrap()),
                field8: u32::from_le_bytes(row[8..12].try_into().unwrap()),
            }
        };
        let additional_rows = std::array::from_fn(|i| read_row(ADDITIONAL_SPECS[i].0));
        let transfer_rows = std::array::from_fn(|i| read_row(TRANSFER_SPECS[i].0));
        let decoded = ObservedTilemapRow {
            source_offset: SOURCE_OFFSET,
            field0: bytes[0], field1: bytes[1],
            field2: u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
            descriptor_pointer: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            field8: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            additional_rows, transfer_rows,
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
