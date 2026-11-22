//! Source-authenticated EUR compact cell frames and sequence used by Firaga.
use super::{
    observed_tilemap::StreamToken, script39_registry_poses::PriorSources, Resource, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const CALLER_LITERAL: u32 = 0x0eb9ac;
pub const TABLE_CELL: u32 = 0x39cca0;
pub const SOURCE_OFFSET: u32 = 0xae9c30;
pub const FIRST_END: u32 = 0xae9ea2;
pub const SEQUENCE_OFFSET: u32 = 0xae9ea4;
pub const SEQUENCE_END: u32 = 0xae9eba;
pub const DIRECTORY_OFFSETS: [u16; 10] = [20, 28, 44, 96, 148, 192, 280, 362, 434, 522];
const FRAME_SHAPES: [(u8, u8, usize); 10] = [
    (3, 2, 8),
    (3, 4, 16),
    (6, 6, 52),
    (7, 8, 52),
    (7, 8, 44),
    (10, 9, 88),
    (9, 9, 82),
    (7, 8, 72),
    (7, 8, 88),
    (6, 8, 100),
];
const HASHES: [&str; 4] = [
    "e951f81c9c58f38793c7a3e32159c7b7524496ad2af600f3f02434d35851165e",
    "f4325e4197bd08b3d5bf170ecd9c206b73494ed75e621901a3744a9d3b4b921b",
    "a1bce0c67692c799cbb8ef3a01e7edc5a01b17ea132ca63ad5800af233954e49",
    "b1d3420dcc8fb59742bd5c56de364d29716a3b2d84b06bca06e7ab2ab9fe87be",
];

/// The original decoder preserves both unexplained header bytes and run controls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub source_offset: u32,
    pub width: u8,
    pub height: u8,
    pub opaque_header: [u8; 2],
    pub tokens: Vec<StreamToken>,
}
impl Frame {
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        let cells = usize::from(self.width) * usize::from(self.height);
        if cells == 0 {
            return Err("Empty Firaga compact frame".into());
        }
        let mut result = vec![
            self.width,
            self.height,
            self.opaque_header[0],
            self.opaque_header[1],
        ];
        let mut decoded = 0usize;
        for token in &self.tokens {
            let count = usize::from(token.control & 0x3fff);
            let values = if token.control & 0xc000 == 0 {
                count
            } else {
                1
            };
            decoded = decoded
                .checked_add(count)
                .ok_or("Firaga compact run overflow")?;
            if count == 0 || decoded > cells || token.values.len() != values {
                return Err("Invalid Firaga compact run".into());
            }
            result.extend(token.control.to_le_bytes());
            for value in &token.values {
                result.extend(value.to_le_bytes());
            }
        }
        if decoded != cells {
            return Err("Incomplete Firaga compact frame".into());
        }
        Ok(result)
    }

    fn source_byte(&self, offset: u32) -> Option<u8> {
        let mut index = offset.checked_sub(self.source_offset)? as usize;
        let header = [
            self.width,
            self.height,
            self.opaque_header[0],
            self.opaque_header[1],
        ];
        if let Some(byte) = header.get(index) {
            return Some(*byte);
        }
        index -= header.len();
        for token in &self.tokens {
            if index < 2 {
                return Some(token.control.to_le_bytes()[index]);
            }
            index -= 2;
            if index < token.values.len() * 2 {
                return Some(token.values[index / 2].to_le_bytes()[index % 2]);
            }
            index -= token.values.len() * 2;
        }
        None
    }
}

/// The first byte is packed by the original reader; the second remains opaque.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceEntry {
    pub frame_index: u8,
    pub high_flags: u8,
    pub second_byte: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaCompactCells {
    pub caller_literal: u32,
    pub table_pointer: u32,
    pub flags: u16,
    pub split_offset: u16,
    pub directory_offsets: [u16; 10],
    pub frames: Vec<Frame>,
    pub sequence: Vec<SequenceEntry>,
}
impl FiragaCompactCells {
    pub fn validate(&self) -> Result<(), String> {
        if (
            self.caller_literal,
            self.table_pointer,
            self.flags,
            self.split_offset,
        ) != (0x0839cc98, 0x08ae9c30, 12, 628)
            || self.directory_offsets != DIRECTORY_OFFSETS
            || self.frames.len() != 10
            || self.sequence.len() != 10
        {
            return Err("Invalid Firaga compact source header or cardinality".into());
        }
        for (i, (frame, &(width, height, bytes))) in
            self.frames.iter().zip(&FRAME_SHAPES).enumerate()
        {
            if frame.source_offset != SOURCE_OFFSET + 4 + u32::from(self.directory_offsets[i])
                || (frame.width, frame.height) != (width, height)
                || frame.source_bytes()?.len() != bytes
            {
                return Err("Invalid Firaga compact frame geometry or extent".into());
            }
        }
        if self
            .sequence
            .iter()
            .any(|entry| entry.frame_index > 0x3f || entry.high_flags > 3)
        {
            return Err("Invalid Firaga compact sequence packed fields".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut first = Vec::new();
        first.extend(self.flags.to_le_bytes());
        first.extend(self.split_offset.to_le_bytes());
        for offset in self.directory_offsets {
            first.extend(offset.to_le_bytes());
        }
        for frame in &self.frames {
            first.extend(frame.source_bytes()?);
        }
        if first.len() != (FIRST_END - SOURCE_OFFSET) as usize {
            return Err("Firaga compact first extent mismatch".into());
        }
        let mut sequence = (self.sequence.len() as u16).to_le_bytes().to_vec();
        for entry in &self.sequence {
            sequence.extend([
                entry.frame_index | (entry.high_flags << 6),
                entry.second_byte,
            ]);
        }
        if sequence.len() != (SEQUENCE_END - SEQUENCE_OFFSET) as usize {
            return Err("Firaga compact sequence extent mismatch".into());
        }
        Ok(vec![
            (
                CALLER_LITERAL,
                self.caller_literal.to_le_bytes().to_vec(),
                "firaga-compact-caller-literal",
            ),
            (
                TABLE_CELL,
                self.table_pointer.to_le_bytes().to_vec(),
                "firaga-compact-table-cell",
            ),
            (SOURCE_OFFSET, first, "firaga-compact-cell-frames"),
            (SEQUENCE_OFFSET, sequence, "firaga-compact-sequence"),
        ])
    }
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        for (at, value, owner) in [
            (CALLER_LITERAL, self.caller_literal, 0),
            (TABLE_CELL, self.table_pointer, 1),
        ] {
            if let Some(index) = offset.checked_sub(at) {
                if let Some(byte) = value.to_le_bytes().get(index as usize) {
                    return Some((*byte, owner));
                }
            }
        }
        if let Some(index) = offset.checked_sub(SOURCE_OFFSET) {
            let index = index as usize;
            if index < 4 {
                let header = [
                    self.flags as u8,
                    (self.flags >> 8) as u8,
                    self.split_offset as u8,
                    (self.split_offset >> 8) as u8,
                ];
                return Some((header[index], 2));
            }
            if index < 24 {
                let index = index - 4;
                return Some((
                    self.directory_offsets[index / 2].to_le_bytes()[index % 2],
                    2,
                ));
            }
            for frame in &self.frames {
                if let Some(byte) = frame.source_byte(offset) {
                    return Some((byte, 2));
                }
            }
        }
        if let Some(index) = offset.checked_sub(SEQUENCE_OFFSET) {
            let index = index as usize;
            if index < 2 {
                return Some(((self.sequence.len() as u16).to_le_bytes()[index], 3));
            }
            let index = index - 2;
            if let Some(entry) = self.sequence.get(index / 2) {
                return Some((
                    if index % 2 == 0 {
                        entry.frame_index | (entry.high_flags << 6)
                    } else {
                        entry.second_byte
                    },
                    3,
                ));
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga compact cells differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: FiragaCompactCells,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga compact cells require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0xeb8b0, 0x493e),
            (0xeb8b2, 0x782a),
            (0xeb8b4, 0x0090),
            (0xeb8b6, 0x1840),
            (0xeb8b8, 0x6801),
            (0xeb8c6, 0xf7ec),
            (0xeb8c8, 0xfed5),
            (0xd868e, 0x78a1),
            (0xd8690, 0x78e0),
            (0xd8696, 0x1822),
            (0xd86c0, 0xf732),
            (0xd86c2, 0xfede),
            (0xb4f2, 0x4648),
            (0xb4f4, 0x6168),
            (0xb4f6, 0x4650),
            (0xb4f8, 0x61a8),
            (0xb088, 0x6990),
            (0xb08a, 0x7801),
            (0xb08c, 0x7840),
            (0xa85a, 0x8851),
            (0xa9a4, 0x7821),
            (0xa9a6, 0x7860),
            (0xa9b2, 0x4005),
            (0xa9c0, 0x4008),
            (0xa9e6, 0x4002),
            (0xae64, 0x69a9),
        ];
        if checks.iter().any(|&(at, expected)| half(at) != expected)
            || word(CALLER_LITERAL as usize) != 0x0839cc98
            || word(TABLE_CELL as usize) != 0x08ae9c30
        {
            return Err("Firaga compact original reader fingerprint mismatch".into());
        }
        let mut frames = Vec::new();
        for (i, &(width, height, _)) in FRAME_SHAPES.iter().enumerate() {
            let start = SOURCE_OFFSET as usize + 4 + usize::from(DIRECTORY_OFFSETS[i]);
            let mut at = start + 4;
            let mut decoded = 0usize;
            let cells = usize::from(width) * usize::from(height);
            let mut tokens = Vec::new();
            while decoded < cells {
                let control = half(at);
                at += 2;
                let count = usize::from(control & 0x3fff);
                if count == 0 || decoded + count > cells {
                    return Err("Invalid original Firaga compact run".into());
                }
                let values = if control & 0xc000 == 0 { count } else { 1 };
                if at + 2 * values > SEQUENCE_OFFSET as usize {
                    return Err("Truncated Firaga compact run".into());
                }
                let values = (0..values).map(|j| half(at + 2 * j)).collect();
                at += 2 * if control & 0xc000 == 0 { count } else { 1 };
                decoded += count;
                tokens.push(StreamToken { control, values });
            }
            let expected_end = if i + 1 < FRAME_SHAPES.len() {
                SOURCE_OFFSET as usize + 4 + usize::from(DIRECTORY_OFFSETS[i + 1])
            } else {
                FIRST_END as usize
            };
            if at != expected_end {
                return Err("Firaga compact frame boundary mismatch".into());
            }
            frames.push(Frame {
                source_offset: start as u32,
                width: rom[start],
                height: rom[start + 1],
                opaque_header: [rom[start + 2], rom[start + 3]],
                tokens,
            });
        }
        if half(SEQUENCE_OFFSET as usize) != 10 {
            return Err("Firaga compact sequence count mismatch".into());
        }
        let sequence = (0..10)
            .map(|i| {
                let at = SEQUENCE_OFFSET as usize + 2 + 2 * i;
                SequenceEntry {
                    frame_index: rom[at] & 0x3f,
                    high_flags: rom[at] >> 6,
                    second_byte: rom[at + 1],
                }
            })
            .collect();
        let decoded = FiragaCompactCells {
            caller_literal: word(CALLER_LITERAL as usize),
            table_pointer: word(TABLE_CELL as usize),
            flags: half(SOURCE_OFFSET as usize),
            split_offset: half(SOURCE_OFFSET as usize + 2),
            directory_offsets: std::array::from_fn(|i| half(SOURCE_OFFSET as usize + 4 + 2 * i)),
            frames,
            sequence,
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga compact source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga compact source hash mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
