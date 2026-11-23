//! Authenticated EUR rows, including one selecting the observed tilemap descriptor.
use super::{observed_tilemap, observed_tilemap_descriptor, script39_registry_poses::PriorSources, Resource, SourceRange, OBSERVED_TILEMAP_DESCRIPTOR_ID, OBSERVED_TILEMAP_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3a4bec;
pub const SOURCE_LENGTH: usize = 12;
pub const SOURCE_SHA256: &str = "493de1159defc6d682a5f8316cfc031421b96f41571004142d7c4ce0a64c93be";
pub const LIST_ROW_OFFSET: u32 = 0x3a4bb0;
pub const LIST_ROW_SHA256: &str = "229d2b40cbf45f1609b5b9b08432167f98f24fabf938e1362945b3e5630db80a";
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
/// Seven separately selected rows pointing to the additional direct descriptors.
pub const DIRECT_ROW_SPECS: [(u32, u32, &str); 7] = [
    (0x3a4c04, 0x083a4b78, "c395e0bd8beb596ab91f1ec2f7371147325052532666137ea1a2274894b45de0"),
    (0x3a4c1c, 0x083a4b80, "8e09b74fb9aeed703ca05683ef7735aded7ad75db5e8d7de47a1d55b4bad0ce7"),
    (0x3a4c34, 0x083a4b88, "845a6166891dd2699be8fc387480dc1e26dbf39b2ddcc3003c34d4d205cbf75d"),
    (0x3a4c4c, 0x083a4b90, "8e448c59fdaea1d896e671d6bd093eeba03da49373185bc0737f4b673649bfb7"),
    (0x3a4c64, 0x083a4b98, "26934bf588438e1a541dd3702c61f707f191138b58a5117371fa9ba29a52dc02"),
    (0x3a4c7c, 0x083a4ba0, "ea841cd574041bbaf59741872b29fdb29fb8ff4a6a9b983c3064ab41a70e2003"),
    (0x3a4c94, 0x083a4ba8, "1eb7cd67692fc89c2a4e28fe4624e43c28a032443a6ad444f923f6ce64a83cba"),
];
/// Seven LZ rows interleaved with the direct tilemap rows.
pub const INTERLEAVED_LZ_ROW_SPECS: [(u32, u32, &str); 7] = [
    (0x3a4bf8, 0x083a4b38, "dce14adf828f18a27320d1c591eeab8f0f4a8ae96e7c96494a59e1f0fc87e4b1"),
    (0x3a4c10, 0x083a4b40, "ba678f316795a742f51c4dd87f25c88a04dac5eaf5371f650a683233624bbb44"),
    (0x3a4c28, 0x083a4b48, "027453dfe42579b8d1c507d80eb76fa7ccdb3d837a0568ad0ecd2734b94466c3"),
    (0x3a4c40, 0x083a4b50, "c010c040d778ec9224d71c3f344d9931ad6574ce9beddcfcbeaa7339d1f7cb18"),
    (0x3a4c58, 0x083a4b58, "2fc3489b3b2af5bfa158030e309aac1791109efac5c147c6ae8d5f8ac9a694ee"),
    (0x3a4c70, 0x083a4b60, "3407be32c28f73781d05d4e1334b8d554ce473dc0fb4e1a25917bde892401df7"),
    (0x3a4c88, 0x083a4b68, "34ff1f112fa5581ad8ede030542c120a20e326171b51ac1d8624c5240919da2f"),
];

/// Observed Soil Evidence transfer rows; guest index ranges remain unrestricted.
pub const SOIL_TRANSFER_ROW_SPECS: [(u32, u32, &str); 4] = [
    (0x39a6f8, 0x0839a648, "2a443d3f04f45fdc926fe14b26051138dc100ec234f87ec7e09617de1c424713"),
    (0x39a770, 0x0839a688, "fa25f25a66719394644ed1f9740cb2815e931ab04d0608614e1a9ea06f47d0bb"),
    (0x39e0ac, 0x0839e090, "1676cc178feec75d4c999899e2515a92ad04212c92fb79dad96f1bfd0194b44b"),
    (0x39e0b8, 0x0839e098, "2a03f79339c88d6fa5484e22d1a3b22050a3dd6eddd4b6c3e295089c84d67a3e"),
];
/// One list row and one mode-7 row; their targets have separate owners.
pub const SOIL_OTHER_ROW_SPECS: [(u32, u32, u8, &str); 2] = [
    (0x39e0a0, 0x0839e088, 2, "b3fd44665dc46e637300c936c4e03b7599b46764f2350a3375c23569e4f02cac"),
    (0x39a704, 0x0839a6e8, 4, "05b1e8c34e9f9c0bc1afa180b012f1a33310abff0645fdead826b4077175836a"),
];
pub const FIRAGA_TRANSFER_ROW_OFFSET: u32 = 0x39a758;
pub const FIRAGA_TRANSFER_ROW_SHA256: &str = "853052904759011e4afd9d823a5b7ad35fa1d0b0f4c87c72c755f458c9835c6e";

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
    pub list_row: AdditionalRow,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_tilemap_rows: Option<[AdditionalRow; 7]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interleaved_lz_rows: Option<[AdditionalRow; 7]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soil_transfer_rows: Option<[AdditionalRow; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soil_other_rows: Option<[AdditionalRow; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firaga_transfer_row: Option<AdditionalRow>,
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
        if self.list_row.source_offset != LIST_ROW_OFFSET || self.list_row.field0 != 2
            || self.list_row.descriptor_pointer != 0x0800_0000 + observed_tilemap::RECTANGLE_CHAIN_OFFSET + 8 {
            return Err("Invalid observed rectangle list row".into());
        }
        if let Some(rows) = &self.direct_tilemap_rows {
            for (row, &(offset, pointer, _)) in rows.iter().zip(&DIRECT_ROW_SPECS) {
                if row.source_offset != offset || row.descriptor_pointer != pointer {
                    return Err("Invalid direct tilemap row".into());
                }
            }
        }
        if let Some(rows) = &self.interleaved_lz_rows {
            for (row, &(offset, pointer, _)) in rows.iter().zip(&INTERLEAVED_LZ_ROW_SPECS) {
                if row.source_offset != offset || row.descriptor_pointer != pointer {
                    return Err("Invalid interleaved LZ row".into());
                }
            }
        }
        if let Some(rows) = &self.soil_transfer_rows {
            for (row, &(offset, pointer, _)) in rows.iter().zip(&SOIL_TRANSFER_ROW_SPECS) {
                if row.source_offset != offset || row.descriptor_pointer != pointer || row.field0 != 3 {
                    return Err("Invalid Soil transfer row".into());
                }
            }
        }
        if let Some(rows) = &self.soil_other_rows {
            for (row, &(offset, pointer, kind, _)) in rows.iter().zip(&SOIL_OTHER_ROW_SPECS) {
                if row.source_offset != offset || row.descriptor_pointer != pointer || row.field0 != kind {
                    return Err("Invalid Soil list or mode-7 row".into());
                }
            }
        }
        if let Some(row) = &self.firaga_transfer_row {
            if row.source_offset != FIRAGA_TRANSFER_ROW_OFFSET || row.field0 != 3
                || row.descriptor_pointer != 0x0800_0000 + observed_tilemap_descriptor::FIRAGA_TRANSFER_DESCRIPTOR_OFFSET {
                return Err("Invalid Firaga transfer row".into());
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
        if let Some(rows) = &self.direct_tilemap_rows {
            let entries = descriptor.additional_tilemap_descriptors.as_ref().ok_or("Missing direct tilemap descriptor dependencies")?;
            if rows.iter().zip(entries).any(|(row, entry)| row.descriptor_pointer != 0x0800_0000 + entry.source_offset) {
                return Err("Direct tilemap row descriptor dependency mismatch".into());
            }
        }
        if let Some(rows) = &self.interleaved_lz_rows {
            let entries = descriptor.interleaved_lz_descriptors.as_ref().ok_or("Missing interleaved LZ descriptor dependencies")?;
            if rows.iter().zip(entries).any(|(row, entry)| row.descriptor_pointer != 0x0800_0000 + entry.source_offset) {
                return Err("Interleaved LZ row descriptor dependency mismatch".into());
            }
        }
        if let Some(rows) = &self.soil_transfer_rows {
            let entries = descriptor.soil_transfer_descriptors.as_ref().ok_or("Missing Soil transfer descriptor dependencies")?;
            if rows.iter().zip(entries).any(|(row, entry)| row.descriptor_pointer != 0x0800_0000 + entry.source_offset) {
                return Err("Soil transfer row descriptor dependency mismatch".into());
            }
        }
        if let Some(rows) = &self.soil_other_rows {
            let entry = descriptor.soil_mode7_descriptor.as_ref().ok_or("Missing Soil mode-7 descriptor dependency")?;
            if rows[1].descriptor_pointer != 0x0800_0000 + entry.source_offset {
                return Err("Soil mode-7 row descriptor dependency mismatch".into());
            }
        }
        if let Some(row) = &self.firaga_transfer_row {
            let entry = descriptor.firaga_transfer_descriptor.as_ref().ok_or("Missing Firaga transfer descriptor dependency")?;
            if row.descriptor_pointer != 0x0800_0000 + entry.source_offset {
                return Err("Firaga transfer row descriptor dependency mismatch".into());
            }
        }
        Ok(())
    }

    pub fn validate_tilemap_dependency(&self, tilemap: &observed_tilemap::ObservedTilemap) -> Result<(), String> {
        self.validate()?;
        tilemap.validate()?;
        if self.list_row.descriptor_pointer != 0x0800_0000 + observed_tilemap::RECTANGLE_CHAIN_OFFSET + 8 {
            return Err("Observed rectangle list row dependency mismatch".into());
        }
        if let Some(rows) = &self.soil_other_rows {
            if tilemap.soil_transfer_sources.is_none()
                || rows[0].descriptor_pointer != 0x0800_0000 + observed_tilemap::SOIL_RECTANGLE_LIST_OFFSET {
                return Err("Missing Soil rectangle list dependency".into());
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
        spans.push((self.list_row.source_offset, self.list_row.source_bytes(), "rectangle-list-row"));
        if let Some(rows) = &self.direct_tilemap_rows {
            for row in rows { spans.push((row.source_offset, row.source_bytes(), "direct-tilemap-row")); }
        }
        if let Some(rows) = &self.interleaved_lz_rows {
            for row in rows { spans.push((row.source_offset, row.source_bytes(), "interleaved-lz-row")); }
        }
        if let Some(rows) = &self.soil_transfer_rows {
            for row in rows { spans.push((row.source_offset, row.source_bytes(), "soil-transfer-row")); }
        }
        if let Some(rows) = &self.soil_other_rows {
            for row in rows { spans.push((row.source_offset, row.source_bytes(), "soil-other-row")); }
        }
        if let Some(row) = &self.firaga_transfer_row {
            spans.push((row.source_offset, row.source_bytes(), "firaga-transfer-row"));
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
            || ffta_rom::digest(&rom[0xdb49c..0xdb4b6]) != "e6b4dac3741f0c0927816d86e21bcd54ea1748a18b5e09d4456c0e1494c61192"
            || ffta_rom::digest(&rom[0xdb374..0xdb3be]) != "0f369afa9549965623a1a8c9a6c58039453416269eedfefc7f325f8a13431992" {
            return Err("Observed tilemap row original reader mismatch".into());
        }
        let tilemap = previous.iter().find(|resource| resource.id == OBSERVED_TILEMAP_ID).ok_or("Missing rectangle list dependency")?;
        if !tilemap.sources.iter().any(|source| source.offset == observed_tilemap::RECTANGLE_CHAIN_OFFSET
            && source.length as usize == observed_tilemap::RECTANGLE_CHAIN_LENGTH) {
            return Err("Observed rectangle list source dependency mismatch".into());
        }
        if !tilemap.sources.iter().any(|source| source.offset == observed_tilemap::SOIL_RECTANGLE_LIST_OFFSET && source.length == 8) {
            return Err("Missing Soil rectangle list source dependency".into());
        }
        let descriptor = previous.iter().find(|resource| resource.id == OBSERVED_TILEMAP_DESCRIPTOR_ID).ok_or("Missing observed tilemap descriptor dependency")?;
        if !descriptor.sources.iter().any(|source| source.offset == observed_tilemap_descriptor::SOURCE_OFFSET && source.length as usize == observed_tilemap_descriptor::SOURCE_LENGTH) {
            return Err("Observed tilemap row source dependency mismatch".into());
        }
        for &(_, pointer, _) in ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS).chain(&DIRECT_ROW_SPECS).chain(&INTERLEAVED_LZ_ROW_SPECS).chain(&SOIL_TRANSFER_ROW_SPECS) {
            if !descriptor.sources.iter().any(|source| source.offset == pointer - 0x0800_0000
                && source.length as usize == observed_tilemap_descriptor::SOURCE_LENGTH) {
                return Err("Additional tilemap row source dependency mismatch".into());
            }
        }
        if !descriptor.sources.iter().any(|source| source.offset == observed_tilemap_descriptor::SOIL_MODE7_DESCRIPTOR_OFFSET
            && source.length as usize == observed_tilemap_descriptor::SOURCE_LENGTH) {
            return Err("Missing Soil mode-7 descriptor source dependency".into());
        }
        if !descriptor.sources.iter().any(|source| source.offset == observed_tilemap_descriptor::FIRAGA_TRANSFER_DESCRIPTOR_OFFSET
            && source.length as usize == observed_tilemap_descriptor::SOURCE_LENGTH) {
            return Err("Missing Firaga transfer descriptor source dependency".into());
        }
        let firaga_at = FIRAGA_TRANSFER_ROW_OFFSET as usize;
        if ffta_rom::digest(&rom[firaga_at..firaga_at + SOURCE_LENGTH]) != FIRAGA_TRANSFER_ROW_SHA256 {
            return Err("Firaga transfer row source mismatch".into());
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Observed tilemap row source hash mismatch".into()); }
        for &(offset, _, hash) in ADDITIONAL_SPECS.iter().chain(&TRANSFER_SPECS).chain(&DIRECT_ROW_SPECS).chain(&INTERLEAVED_LZ_ROW_SPECS).chain(&SOIL_TRANSFER_ROW_SPECS) {
            let at = offset as usize;
            if ffta_rom::digest(&rom[at..at + SOURCE_LENGTH]) != hash {
                return Err("Additional observed tilemap row source hash mismatch".into());
            }
        }
        for &(offset, _, _, hash) in &SOIL_OTHER_ROW_SPECS {
            let at = offset as usize;
            if ffta_rom::digest(&rom[at..at + SOURCE_LENGTH]) != hash {
                return Err("Soil list or mode-7 row source mismatch".into());
            }
        }
        let list_at = LIST_ROW_OFFSET as usize;
        if ffta_rom::digest(&rom[list_at..list_at + SOURCE_LENGTH]) != LIST_ROW_SHA256 {
            return Err("Observed rectangle list row source hash mismatch".into());
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
            additional_rows, transfer_rows, list_row: read_row(LIST_ROW_OFFSET),
            direct_tilemap_rows: Some(std::array::from_fn(|i| read_row(DIRECT_ROW_SPECS[i].0))),
            interleaved_lz_rows: Some(std::array::from_fn(|i| read_row(INTERLEAVED_LZ_ROW_SPECS[i].0))),
            soil_transfer_rows: Some(std::array::from_fn(|i| read_row(SOIL_TRANSFER_ROW_SPECS[i].0))),
            soil_other_rows: Some(std::array::from_fn(|i| read_row(SOIL_OTHER_ROW_SPECS[i].0))),
            firaga_transfer_row: Some(read_row(FIRAGA_TRANSFER_ROW_OFFSET)),
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
