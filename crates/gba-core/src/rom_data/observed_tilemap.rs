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
/// Seven adjacent pointer cells following the previously represented root.
pub const ADDITIONAL_RECTANGLE_POINTER_OFFSET: u32 = 0x3a4d40;
pub const ADDITIONAL_RECTANGLE_POINTER_COUNT: usize = 7;
pub const ADDITIONAL_RECTANGLE_POINTER_SHA256: &str = "0a783b3b2312cb2d8c092ad00325695b6c09ad591c78d3ba0525438f2fdc88e1";
/// Five distinct 1×12 sources selected by those seven cells.
pub const ADDITIONAL_RECTANGLE_SPECS: [(u32, &str); 5] = [
    (0xae7908, "530916278f19eccdf3ac9342f6506b06135967f318df813dbc744a83fb175486"),
    (0xae7924, "217cfe61b855fa00a86a57a7afabb9d7f2dc3243540f85c484b0316983ecffa6"),
    (0xae7940, "7ec5591cfa8636d88bb11cf93d7a34aea3d43d1348d3d3e6b9375dd4912a28ea"),
    (0xae795c, "429b843e3c7d15756094166727b3bb20b3c47951c7e9562a012bb738ace5d728"),
    (0xae7978, "ec71701ab4c0670d8b1e4f4810b5c50a73d2743a979754ed5381f698d0968eac"),
];
pub const ADDITIONAL_RECTANGLE_TARGETS: [usize; 7] = [0, 1, 2, 3, 2, 4, 3];
pub const ALTERNATE_SELECTOR_OFFSET: u32 = 0x3a4d5c;
pub const ALTERNATE_SELECTOR_LENGTH: usize = 8;
/// Eight authenticated source words used by an original indexed load.
pub const INDEXED_POINTER_OFFSET: u32 = 0x3a4d84;
pub const INDEXED_POINTER_COUNT: usize = 8;
pub const INDEXED_POINTER_SHA256: &str = "3e0b64950ead9c9f137549ab4262adc0016f33262fbce61f64f29879ace430c4";
/// Source offset, encoded length, source hash and decoded-cell hash.
pub const ADDITIONAL_STREAM_SPECS: [(u32, usize, &str, &str); 7] = [
    (0xaf2780, 102, "13f7cb94f750a8e08c9645e947de4f1df752a9c7c354c610f73d1c3f613895db", "3c0b933f32c794fae992ea28cd612943955627ccb235e2ba6ebc48eee066644c"),
    (0xaf27e8, 114, "39c6f8bdffbc49b6af1f419be6a28d28964fc92f086708887969bd16fcd147ab", "41af37f8550322ccffbbb40ba49f535435eeb18f973843d173e564e2a992bde2"),
    (0xaf285c, 102, "84fab55aca1f48a317b2094e6fe6be32ba9a69ca2d49772ecb9a15aa1fa8bb28", "57f050534992d971c22f941c1b994b233ca7eda0b7eea12e55983b2af1537558"),
    (0xaf28c4, 102, "3ff51e14f5c548202f6bcffefb999e0661e36fd55dc31a6e8dc3d9ac0907ad33", "6b68b97ffb9185c2bbfc0fa8954700086f1770519fdf1df3d6fbe1df58049cc6"),
    (0xaf292c, 90, "bf8e9ff8eb1ec9fc79d58a9d73c9758f983990acd1688704f98f93f085a472c0", "fdaf323403a5fb142cc34ab7ced77432b3c2ea80d4c3ba06a7d0c95c280839bc"),
    (0xaf2988, 106, "418a7dad4660fd79750c163dbc62d622be28618df939feba0b089e24baf729e2", "4bf3d2c49117f3dfa400e0a3c78fee1ed3200217ac3e6023c75f51e295d1b890"),
    (0xaf29f4, 106, "02374ecc58a35d25f6c4d9a24d2987890f6dd27535dc8533054cfbe0b8699803", "75a50350703d513babd22a91fc0464307827e5dfad8b957a16607ce81e2d4e2c"),
];
/// Source, height, width, hash and original PC-relative root literal.
pub const ALTERNATE_RECT_SPECS: [(u32, u8, u8, &str, u32); 2] = [
    (0xae6e14, 12, 3, "9088f2114cfa9e4d96ae04d9b3cf2c832c3a18347d1b45da598871b3333b6439", 0x10951c),
    (0xae79c0, 7, 3, "542c356d4f65297b26f3896a2559c96bdc476f5eb9c920f86527c99ac12df85c", 0x109538),
];
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

/// A compressed run can contain one repeated/ascending value or literal cells.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamToken {
    pub control: u16,
    pub values: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalStream {
    pub source_offset: u32,
    pub width: u8,
    pub height: u8,
    pub opaque_header: u16,
    pub mode: u8,
    pub flags: u8,
    pub tokens: Vec<StreamToken>,
}

impl AdditionalStream {
    fn from_source(source_offset: u32, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 6 { return Err("Truncated additional tilemap header".into()); }
        let mut tokens = Vec::new();
        let mut at = 6usize;
        let mut decoded = 0usize;
        while decoded < 256 {
            let control_bytes = bytes.get(at..at + 2).ok_or("Truncated additional tilemap control")?;
            let control = u16::from_le_bytes([control_bytes[0], control_bytes[1]]);
            let count = usize::from(control & 0x3fff);
            if count == 0 || decoded + count > 256 { return Err("Invalid additional tilemap token length".into()); }
            at += 2;
            let values = if control & 0xc000 == 0 { count } else { 1 };
            let encoded = bytes.get(at..at + values * 2).ok_or("Truncated additional tilemap values")?;
            tokens.push(StreamToken { control, values: encoded.chunks_exact(2).map(|v| u16::from_le_bytes([v[0], v[1]])).collect() });
            at += values * 2;
            decoded += count;
        }
        if at != bytes.len() { return Err("Additional tilemap source extent mismatch".into()); }
        let stream = Self { source_offset, width: bytes[0], height: bytes[1],
            opaque_header: u16::from_le_bytes([bytes[2], bytes[3]]), mode: bytes[4], flags: bytes[5], tokens };
        stream.decoded_cells()?;
        Ok(stream)
    }

    pub fn decoded_cells(&self) -> Result<Vec<u16>, String> {
        if (self.width, self.height, self.opaque_header, self.mode, self.flags) != (16, 16, 0, 4, 2) {
            return Err("Invalid additional tilemap header".into());
        }
        let mut cells = Vec::with_capacity(256);
        for token in &self.tokens {
            let count = usize::from(token.control & 0x3fff);
            if count == 0 || cells.len() + count > 256 {
                return Err("Invalid additional tilemap token length".into());
            }
            let encoded_count = if token.control & 0xc000 == 0 { count } else { 1 };
            if token.values.len() != encoded_count {
                return Err("Invalid additional tilemap token values".into());
            }
            if token.control & 0x8000 != 0 {
                cells.extend((0..count).map(|i| token.values[0].wrapping_add(i as u16)));
            } else if token.control & 0x4000 != 0 {
                cells.extend(std::iter::repeat_n(token.values[0], count));
            } else {
                cells.extend_from_slice(&token.values);
            }
        }
        if cells.len() != 256 { return Err("Incomplete additional tilemap".into()); }
        Ok(cells)
    }

    pub fn source_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.width, self.height];
        bytes.extend(self.opaque_header.to_le_bytes());
        bytes.extend([self.mode, self.flags]);
        for token in &self.tokens {
            bytes.extend(token.control.to_le_bytes());
            for value in &token.values { bytes.extend(value.to_le_bytes()); }
        }
        bytes
    }

    pub fn source_byte(&self, index: usize) -> Option<u8> {
        let mut at = index.checked_sub(self.source_offset as usize)?;
        let header = [self.width, self.height, self.opaque_header as u8, (self.opaque_header >> 8) as u8, self.mode, self.flags];
        if let Some(byte) = header.get(at) { return Some(*byte); }
        at -= header.len();
        for token in &self.tokens {
            if at < 2 { return Some(token.control.to_le_bytes()[at]); }
            at -= 2;
            if at < token.values.len() * 2 { return Some(token.values[at / 2].to_le_bytes()[at % 2]); }
            at -= token.values.len() * 2;
        }
        None
    }
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
    /// Two original alternatives selected by guest code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alternate_rectangles: Option<[Rectangle; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alternate_selector: Option<[u8; ALTERNATE_SELECTOR_LENGTH]>,
    /// Guest addresses remain guest-owned; entries outside this projection are raw.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexed_pointers: Option<[u32; INDEXED_POINTER_COUNT]>,
    /// Seven additional independently rooted mode-4 tilemaps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_streams: Option<[AdditionalStream; 7]>,
    /// Source cells retain repeated guest pointers, rather than duplicating targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_rectangle_pointers: Option<[u32; ADDITIONAL_RECTANGLE_POINTER_COUNT]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_rectangles: Option<[Rectangle; 5]>,
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
        if self.alternate_rectangles.is_some() != self.alternate_selector.is_some() {
            return Err("Incomplete alternate tilemap projection".into());
        }
        if self.indexed_pointers.as_ref().is_some_and(|pointers| {
            pointers[0] != 0x0800_0000 + self.source_offset
                || pointers.iter().any(|pointer| !(0x0800_0000..0x0900_0000).contains(pointer))
        }) {
            return Err("Invalid indexed tilemap pointer projection".into());
        }
        if let Some(streams) = &self.additional_streams {
            let pointers = self.indexed_pointers.as_ref().ok_or("Additional tilemaps require indexed pointers")?;
            for (i, (stream, &(offset, length, _, _))) in streams.iter().zip(&ADDITIONAL_STREAM_SPECS).enumerate() {
                if stream.source_offset != offset || pointers[i + 1] != 0x0800_0000 + offset {
                    return Err("Invalid additional tilemap source binding".into());
                }
                stream.decoded_cells()?;
                if stream.source_bytes().len() != length {
                    return Err("Invalid additional tilemap source length".into());
                }
            }
        }
        if let Some(rectangles) = &self.alternate_rectangles {
            for (rectangle, &(offset, height, width, _, _)) in rectangles.iter().zip(&ALTERNATE_RECT_SPECS) {
                if (rectangle.source_offset, rectangle.height, rectangle.width) != (offset, height, width)
                    || rectangle.cells.len() != usize::from(height) * usize::from(width) {
                    return Err("Invalid alternate tilemap rectangle geometry".into());
                }
            }
        }
        match (&self.additional_rectangle_pointers, &self.additional_rectangles) {
            (Some(pointers), Some(rectangles)) => {
                for (rectangle, &(offset, _)) in rectangles.iter().zip(&ADDITIONAL_RECTANGLE_SPECS) {
                    if (rectangle.source_offset, rectangle.height, rectangle.width) != (offset, 1, 12)
                        || rectangle.cells.len() != 12 {
                        return Err("Invalid additional rectangle geometry".into());
                    }
                }
                for (pointer, &target) in pointers.iter().zip(&ADDITIONAL_RECTANGLE_TARGETS) {
                    if *pointer != 0x0800_0000 + rectangles[target].source_offset {
                        return Err("Invalid additional rectangle pointer binding".into());
                    }
                }
            }
            (None, None) => {}
            _ => return Err("Incomplete additional rectangle projection".into()),
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
        if let Some(rectangles) = &self.alternate_rectangles {
            for (rectangle, role) in rectangles.iter().zip(["tilemap-alternate-0", "tilemap-alternate-1"]) {
                spans.push((rectangle.source_offset, rectangle.source_bytes(), role));
            }
        }
        if let Some(selector) = &self.alternate_selector {
            spans.push((ALTERNATE_SELECTOR_OFFSET, selector.to_vec(), "tilemap-alternate-selector"));
        }
        if let Some(pointers) = &self.indexed_pointers {
            spans.push((INDEXED_POINTER_OFFSET, pointers.iter().flat_map(|pointer| pointer.to_le_bytes()).collect(), "tilemap-indexed-pointers"));
        }
        if let Some(streams) = &self.additional_streams {
            for stream in streams { spans.push((stream.source_offset, stream.source_bytes(), "tilemap-additional-stream")); }
        }
        if let Some(pointers) = &self.additional_rectangle_pointers {
            spans.push((ADDITIONAL_RECTANGLE_POINTER_OFFSET, pointers.iter().flat_map(|pointer| pointer.to_le_bytes()).collect(), "tilemap-additional-rectangle-pointers"));
        }
        if let Some(rectangles) = &self.additional_rectangles {
            for rectangle in rectangles { spans.push((rectangle.source_offset, rectangle.source_bytes(), "tilemap-additional-rectangle")); }
        }
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
            (0x10950a, 0x109540, "aecc2689b8cf19427a67608665f7ff14c929c3b707611e342ec483461650122f"),
            (0x1091be, 0x1091f8, "33c5bfaa8bb7c60757b55277a78055871d7b3fbeb46a4d70a22c5595a4d1040e"),
            (0x3a4d64, 0x3a4d84, "209a51a14c6f038dea7c65ad499d603e259ecc7fcaa5b0afd6115d66e0470c5b"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Observed tilemap original reader mismatch".into());
            }
        }
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        if word(0x3a4b70) != 0x0800_0000 + SOURCE_OFFSET || word(0x3a4d84) != 0x0800_0000 + SOURCE_OFFSET || word(0x3a4b78) != 0x08af_2780
            || word(0x109228) != 0x0800_0000 + INDEXED_POINTER_OFFSET {
            return Err("Observed tilemap source root mismatch".into());
        }
        let pointer_start = INDEXED_POINTER_OFFSET as usize;
        let pointer_bytes = &rom[pointer_start..pointer_start + INDEXED_POINTER_COUNT * 4];
        if ffta_rom::digest(pointer_bytes) != INDEXED_POINTER_SHA256 {
            return Err("Indexed tilemap pointer source mismatch".into());
        }
        let indexed_pointers = std::array::from_fn(|i| word(pointer_start + i * 4));
        let mut streams = Vec::new();
        for &(offset, length, source_hash, decoded_hash) in &ADDITIONAL_STREAM_SPECS {
            let start = offset as usize;
            let source = &rom[start..start + length];
            if ffta_rom::digest(source) != source_hash { return Err("Additional tilemap source mismatch".into()); }
            let stream = AdditionalStream::from_source(offset, source)?;
            let decoded: Vec<_> = stream.decoded_cells()?.iter().flat_map(|cell| cell.to_le_bytes()).collect();
            if stream.source_bytes() != source || ffta_rom::digest(&decoded) != decoded_hash {
                return Err("Additional tilemap reconstruction mismatch".into());
            }
            streams.push(stream);
        }
        let additional_streams: [AdditionalStream; 7] = streams.try_into().map_err(|_| "Incomplete additional tilemaps")?;
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
        for &(offset, height, width, hash, root) in &ALTERNATE_RECT_SPECS {
            let start = offset as usize; let length = 2 + usize::from(height) * usize::from(width) * 2;
            if rom[start] != height || rom[start + 1] != width || ffta_rom::digest(&rom[start..start + length]) != hash
                || word(root as usize) != 0x0800_0000 + offset {
                return Err("Alternate tilemap rectangle source mismatch".into());
            }
        }
        if word(0x10953c) != 0x0800_0000 + ALTERNATE_SELECTOR_OFFSET {
            return Err("Alternate tilemap selector root mismatch".into());
        }
        let alternate_rectangles = std::array::from_fn(|i| {
            let (offset, height, width, _, _) = ALTERNATE_RECT_SPECS[i];
            let start = offset as usize + 2; let length = usize::from(height) * usize::from(width) * 2;
            let cells = rom[start..start + length].chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]])).collect();
            Rectangle { source_offset: offset, height, width, cells }
        });
        let selector = &rom[ALTERNATE_SELECTOR_OFFSET as usize..ALTERNATE_SELECTOR_OFFSET as usize + ALTERNATE_SELECTOR_LENGTH];
        if ffta_rom::digest(selector) != "2a196f6b7b03ae2c541aa8fa618342fa2f30dd118f90bd12f0b11e140213c7d2" {
            return Err("Alternate tilemap selector source mismatch".into());
        }
        let alternate_selector = selector.try_into().unwrap();
        let pointer_start = ADDITIONAL_RECTANGLE_POINTER_OFFSET as usize;
        let pointer_source = &rom[pointer_start..pointer_start + ADDITIONAL_RECTANGLE_POINTER_COUNT * 4];
        if ffta_rom::digest(pointer_source) != ADDITIONAL_RECTANGLE_POINTER_SHA256
            || word(0x109518) != 0x0800_0000 + RECTANGLE_TABLE_OFFSET {
            return Err("Additional rectangle pointer source mismatch".into());
        }
        let additional_rectangle_pointers = std::array::from_fn(|i| word(pointer_start + i * 4));
        let mut extra_rectangles = Vec::new();
        for &(offset, hash) in &ADDITIONAL_RECTANGLE_SPECS {
            let source = &rom[offset as usize..offset as usize + 26];
            if source[..2] != [1, 12] || ffta_rom::digest(source) != hash {
                return Err("Additional rectangle source mismatch".into());
            }
            let cells = source[2..].chunks_exact(2).map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]])).collect();
            extra_rectangles.push(Rectangle { source_offset: offset, height: 1, width: 12, cells });
        }
        let additional_rectangles: [Rectangle; 5] = extra_rectangles.try_into().map_err(|_| "Incomplete additional rectangles")?;
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
            alternate_rectangles: Some(alternate_rectangles), alternate_selector: Some(alternate_selector),
            indexed_pointers: Some(indexed_pointers),
            additional_streams: Some(additional_streams),
            additional_rectangle_pointers: Some(additional_rectangle_pointers),
            additional_rectangles: Some(additional_rectangles),
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
