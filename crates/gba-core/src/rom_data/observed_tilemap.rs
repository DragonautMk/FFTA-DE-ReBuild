//! Authenticated EUR tilemaps, preserving runs and selected halfword rectangles.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0xaf2718;
pub const SOURCE_LENGTH: usize = 102;
pub const SOURCE_SHA256: &str = "f84b800ef7584f5f274fbbb8eb32bf4bb45a323c56ff026d7218b14c17f0709e";
pub const DECODED_SHA256: &str = "307e945d04b0bffeac6f011e54fdbd455db30da84b08d7d9482d25701311f186";
pub const RECTANGLE_CHAIN_OFFSET: u32 = 0x3a4b08;
pub const RECTANGLE_CHAIN_LENGTH: usize = 16;
pub const RECTANGLE_TABLE_OFFSET: u32 = 0x3a4d3c;
pub const RECTANGLE_TABLE_LENGTH: usize = 4;
/// Exact observed rectangles: source, height, width, source hash and pointer cell.
pub const RECT_SPECS: [(u32, u8, u8, &str, u32); 2] = [
    (0xae78ac, 2, 15, "2ab4fbbaef0dbd47b7c507b0beee844d09fbdec3d6c15043efb28d70d1f6ff32", 0x3a4b08),
    (0xae78ec, 1, 12, "8e385066f1da1a960099736dfc7b0c051ad08c454ae20320f383745db123a14d", 0x3a4d3c),
];

/// The high control bits select ascending or repeated 16-bit entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub control: u16,
    pub value: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rectangle {
    pub source_offset: u32,
    pub height: u8,
    pub width: u8,
    pub cells: Vec<u16>,
}

impl Rectangle {
    pub fn source_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.cells.len() * 2);
        bytes.extend([self.height, self.width]);
        for cell in &self.cells { bytes.extend(cell.to_le_bytes()); }
        bytes
    }
}

/// One observed descriptor, its one-entry list and a separate table pointer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RectangleRoots {
    pub first_rectangle_pointer: u32,
    pub descriptor_parameters: [u8; 4],
    pub entry_count: u8,
    pub list_padding: [u8; 3],
    pub descriptor_pointer: u32,
    pub second_rectangle_pointer: u32,
}

impl RectangleRoots {
    pub fn chain_bytes(&self) -> [u8; RECTANGLE_CHAIN_LENGTH] {
        let mut bytes = [0; RECTANGLE_CHAIN_LENGTH];
        bytes[..4].copy_from_slice(&self.first_rectangle_pointer.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.descriptor_parameters);
        bytes[8] = self.entry_count;
        bytes[9..12].copy_from_slice(&self.list_padding);
        bytes[12..16].copy_from_slice(&self.descriptor_pointer.to_le_bytes());
        bytes
    }

    pub fn table_bytes(&self) -> [u8; RECTANGLE_TABLE_LENGTH] {
        self.second_rectangle_pointer.to_le_bytes()
    }
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
    pub rectangles: [Rectangle; 2],
    pub rectangle_roots: RectangleRoots,
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
        for (rectangle, &(offset, height, width, _, _)) in self.rectangles.iter().zip(&RECT_SPECS) {
            if (rectangle.source_offset, rectangle.height, rectangle.width) != (offset, height, width)
                || rectangle.cells.len() != usize::from(height) * usize::from(width) {
                return Err("Invalid observed tilemap rectangle geometry".into());
            }
        }
        if self.rectangle_roots.first_rectangle_pointer != 0x0800_0000 + self.rectangles[0].source_offset
            || self.rectangle_roots.entry_count != 1
            || self.rectangle_roots.descriptor_pointer != 0x0800_0000 + RECTANGLE_CHAIN_OFFSET
            || self.rectangle_roots.second_rectangle_pointer != 0x0800_0000 + self.rectangles[1].source_offset {
            return Err("Invalid observed tilemap rectangle roots".into());
        }
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

    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>, &'static str)> {
        let mut spans = vec![(self.source_offset, self.source_bytes(), "tilemap-runs")];
        for (rectangle, role) in self.rectangles.iter().zip(["tilemap-rectangle-0", "tilemap-rectangle-1"]) {
            spans.push((rectangle.source_offset, rectangle.source_bytes(), role));
        }
        spans.push((RECTANGLE_CHAIN_OFFSET, self.rectangle_roots.chain_bytes().to_vec(), "tilemap-rectangle-chain"));
        spans.push((RECTANGLE_TABLE_OFFSET, self.rectangle_roots.table_bytes().to_vec(), "tilemap-rectangle-pointer"));
        spans
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
            (0xd80e0, 0xd8164, "f802daafd73dab107c4269b328430eec39b7c506f02aaa95abd0f8b0ec6fc883"),
            (0xd816c, 0xd81d4, "f9933c845686c44a098c164f52ba6576b027761e8d42cc17417e7130866b5ae1"),
            (0xdb374, 0xdb3be, "0f369afa9549965623a1a8c9a6c58039453416269eedfefc7f325f8a13431992"),
            (0x1094f2, 0x1094fc, "f44f4d84917dc2fb3a0cb21acdf06109ab433aea74f3551f44265d7d5275e712"),
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
        let half = |at: usize| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
        for &(offset, height, width, hash, root) in &RECT_SPECS {
            let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
            if rom[start] != height || rom[start + 1] != width || ffta_rom::digest(&rom[start..start + length]) != hash
                || word(root as usize) != 0x0800_0000 + offset {
                return Err("Observed tilemap rectangle source mismatch".into());
            }
        }
        let rectangles = std::array::from_fn(|i| {
            let (offset, height, width, _, _) = RECT_SPECS[i];
            let start = offset as usize + 2; let length = usize::from(height) * usize::from(width) * 2;
            let cells = rom[start..start + length].chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]])).collect();
            Rectangle { source_offset: offset, height, width, cells }
        });
        let chain = &rom[RECTANGLE_CHAIN_OFFSET as usize..RECTANGLE_CHAIN_OFFSET as usize + RECTANGLE_CHAIN_LENGTH];
        let table = &rom[RECTANGLE_TABLE_OFFSET as usize..RECTANGLE_TABLE_OFFSET as usize + RECTANGLE_TABLE_LENGTH];
        if ffta_rom::digest(chain) != "19299b60bbbc6931f613bfe31561ad5a81c3f9e23bd3eb565dd773d2b587a33a"
            || ffta_rom::digest(table) != "9446ee87a3e39ce1cc71ae499358d20e659a553dd1839fb726e0994294e96eba" {
            return Err("Observed tilemap rectangle root source mismatch".into());
        }
        let rectangle_roots = RectangleRoots {
            first_rectangle_pointer: u32::from_le_bytes(chain[..4].try_into().unwrap()),
            descriptor_parameters: chain[4..8].try_into().unwrap(),
            entry_count: chain[8], list_padding: chain[9..12].try_into().unwrap(),
            descriptor_pointer: u32::from_le_bytes(chain[12..16].try_into().unwrap()),
            second_rectangle_pointer: u32::from_le_bytes(table.try_into().unwrap()),
        };
        let decoded = ObservedTilemap {
            source_offset: SOURCE_OFFSET,
            width: bytes[0], height: bytes[1], opaque_header: half(2), mode: bytes[4], flags: bytes[5],
            runs: std::array::from_fn(|i| Run { control: half(6 + i * 4), value: half(8 + i * 4) }),
            rectangles, rectangle_roots,
        };
        let cells = decoded.decoded_cells()?;
        if decoded.source_bytes() != bytes || ffta_rom::digest(&cells.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()) != DECODED_SHA256 {
            return Err("Observed tilemap reconstruction mismatch".into());
        }
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans() {
            if prior.contains(offset, bytes.len())? { return Err("Observed tilemap source already has an owner".into()); }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes { return Err("Observed tilemap reconstruction mismatch".into()); }
            sources.push(SourceRange { offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role: role.into() });
        }
        Ok(Self { decoded, sources })
    }
}
