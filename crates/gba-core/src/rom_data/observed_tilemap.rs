//! One authenticated EUR tilemap, preserving its original compressed runs.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0xaf2718;
pub const SOURCE_LENGTH: usize = 102;
pub const SOURCE_SHA256: &str = "f84b800ef7584f5f274fbbb8eb32bf4bb45a323c56ff026d7218b14c17f0709e";
pub const DECODED_SHA256: &str = "307e945d04b0bffeac6f011e54fdbd455db30da84b08d7d9482d25701311f186";

/// The high control bits select ascending or repeated 16-bit entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub control: u16,
    pub value: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTilemap {
    pub source_offset: u32,
    pub width: u8,
    pub height: u8,
    pub opaque_header: u16,
    pub mode: u8,
    pub flags: u8,
    pub runs: [Run; 24],
}

impl ObservedTilemap {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || (self.width, self.height, self.mode, self.flags) != (16, 16, 4, 2) {
            return Err("Invalid observed tilemap header".into());
        }
        let mut count = 0usize;
        for run in &self.runs {
            let n = usize::from(run.control & 0x3fff);
            if n == 0 || run.control & 0xc000 == 0 {
                return Err("Invalid observed tilemap run".into());
            }
            count = count.checked_add(n).ok_or("Observed tilemap length overflow")?;
            if count > 256 { return Err("Observed tilemap run exceeds its authenticated extent".into()); }
        }
        if count != 256 { return Err("Incomplete observed tilemap".into()); }
        Ok(())
    }

    pub fn source_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SOURCE_LENGTH);
        bytes.extend([self.width, self.height]);
        bytes.extend(self.opaque_header.to_le_bytes());
        bytes.extend([self.mode, self.flags]);
        for run in &self.runs {
            bytes.extend(run.control.to_le_bytes());
            bytes.extend(run.value.to_le_bytes());
        }
        bytes
    }

    pub fn decoded_cells(&self) -> Result<Vec<u16>, String> {
        self.validate()?;
        let mut cells = Vec::with_capacity(256);
        for run in &self.runs {
            for i in 0..(run.control & 0x3fff) {
                cells.push(if run.control & 0x8000 != 0 { run.value.wrapping_add(i) } else { run.value });
            }
        }
        Ok(cells)
    }
}

pub struct Catalog {
    pub decoded: ObservedTilemap,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed tilemap requires the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (0xda544, 0xda590, "1de61b0dea68ac7332b0ed9755fc4b7fd1df2d08c69cef8dbb47167dfe925b20"),
            (0xda64c, 0xda6a0, "7314784022a2b2a49bf98d6e8f5887b5b1102f30900ef327c354e70b7ada73d6"),
            (0xda14c, 0xda1fe, "24325845d23d8f6262749a416a14a752e68b1e85e21b035266c58229c73e4230"),
            (0xda2a8, 0xda458, "e54a9da79417bbc95a8c57fc84450ba87467c6a5f7053e7f6c6fee18e4fd87ef"),
            (0xdb30a, 0xdb326, "0ac3d1f008eb9ff01ae13ba635e54c40af5f41ddcc4150750a63c5dd100a7174"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Observed tilemap original reader mismatch".into());
            }
        }
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        if word(0x3a4b70) != 0x0800_0000 + SOURCE_OFFSET || word(0x3a4d84) != 0x0800_0000 + SOURCE_OFFSET || word(0x3a4b78) != 0x08af_2780 {
            return Err("Observed tilemap source root mismatch".into());
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Observed tilemap source hash mismatch".into()); }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_LENGTH)? {
            return Err("Observed tilemap source already has an owner".into());
        }
        let half = |at: usize| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
        let decoded = ObservedTilemap {
            source_offset: SOURCE_OFFSET,
            width: bytes[0], height: bytes[1], opaque_header: half(2), mode: bytes[4], flags: bytes[5],
            runs: std::array::from_fn(|i| Run { control: half(6 + i * 4), value: half(8 + i * 4) }),
        };
        let cells = decoded.decoded_cells()?;
        if decoded.source_bytes() != bytes || ffta_rom::digest(&cells.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()) != DECODED_SHA256 {
            return Err("Observed tilemap reconstruction mismatch".into());
        }
        Ok(Self {
            decoded,
            sources: vec![SourceRange { offset: SOURCE_OFFSET, length: SOURCE_LENGTH as u32, sha256: ffta_rom::digest(bytes), role: "tilemap-runs".into() }],
        })
    }
}
