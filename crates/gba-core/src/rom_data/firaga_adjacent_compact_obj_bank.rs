//! Selected EUR Firaga task descriptors and adjacent compact OBJ sequence.
use super::{
    compact_obj_animations, obj_lists, script39_registry_poses::PriorSources, Resource,
    SourceRange, FIRAGA_COMPACT_OBJ_BANK_ID, OBJ_STATIC_LISTS_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const COUNT_LITERAL: u32 = 0x0eb9b8;
pub const COUNT_BYTE: u32 = 0x39cc7a;
pub const FIRST_POINTER: u32 = 0x39cc3c;
pub const FIRST_FIELDS: u32 = 0x39cc40;
pub const SECOND_DESCRIPTOR: u32 = 0x39cc54;
pub const BANK_ROOT: u32 = 0x6b36ac;
pub const SEQUENCE_DIRECTORY: u32 = 0x6b4224;
pub const FIRST_SEQUENCE: u32 = 0x6b422c;
pub const LIST_OFFSETS: [u16; 11] = [
    0x44, 0xb2, 0x120, 0x194, 0x208, 0x27c, 0x2f6, 0x37c, 0x402, 0x488, 0x48a,
];
const HASHES: [&str; 7] = [
    "d2ccfe8f8e4d12a15bdacc5a5099fb3a877469c5914e54269988c8339cabc7e2",
    "dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986",
    "6f4ebc3f6ba086fe46a7f8d8ce045e2f55ac52bcc39327a43a0ac4683e35dc45",
    "662dd1b49218499eff0cbfc498226cf7b16ff97c0b9f055aea9a58bf7bd3236b",
    "e25f17cba6284ed4bd2cae294ee8aceb79b25f19db63847c2b465e8a9e6a7865",
    "e545d395bb3fd971f91bf9a2b6722831df704efae6c1aa9da0989ed0970b77bb",
    "c10ddff8831cac5687b8fe86fecd1bfccbefa9381aed200e98aa66bdda1b1b14",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaAdjacentCompactObjBank {
    pub count_literal: u32,
    pub count: u8,
    pub first_descriptor_fields: [u16; 10],
    pub second_bank_pointer: u32,
    pub second_descriptor_fields: [u16; 10],
    pub flags: u16,
    pub split_offset: u16,
    pub list_offsets: [u16; 11],
    pub sequence_relative_offset: u16,
    pub frames: Vec<compact_obj_animations::Frame>,
}
impl FiragaAdjacentCompactObjBank {
    pub fn validate(&self) -> Result<(), String> {
        if (
            self.count_literal,
            self.count,
            self.second_bank_pointer,
            self.flags,
            self.split_offset,
            self.sequence_relative_offset,
        ) != (0x0839cc78, 2, 0x086b36ac, 12, 0x0b78, 8)
            || self.first_descriptor_fields[0] != 0
            || self.second_descriptor_fields[0] != 0
            || self.list_offsets != LIST_OFFSETS
            || self.frames.len() != 17
            || self
                .frames
                .iter()
                .any(|frame| usize::from(frame.list) >= self.list_offsets.len())
        {
            return Err("Invalid adjacent Firaga compact OBJ bank shape".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let fields =
            |values: &[u16]| -> Vec<u8> { values.iter().flat_map(|x| x.to_le_bytes()).collect() };
        let mut second = self.second_bank_pointer.to_le_bytes().to_vec();
        second.extend(fields(&self.second_descriptor_fields));
        let mut header = self.flags.to_le_bytes().to_vec();
        header.extend(self.split_offset.to_le_bytes());
        header.extend(fields(&self.list_offsets));
        let mut sequence = (self.frames.len() as u16).to_le_bytes().to_vec();
        for frame in &self.frames {
            sequence.extend([frame.list, frame.duration]);
        }
        Ok(vec![
            (
                COUNT_LITERAL,
                self.count_literal.to_le_bytes().to_vec(),
                "firaga-adjacent-obj-count-literal",
            ),
            (COUNT_BYTE, vec![self.count], "firaga-adjacent-obj-count"),
            (
                FIRST_FIELDS,
                fields(&self.first_descriptor_fields),
                "firaga-adjacent-obj-first-descriptor-fields",
            ),
            (
                SECOND_DESCRIPTOR,
                second,
                "firaga-adjacent-obj-second-descriptor",
            ),
            (BANK_ROOT, header, "firaga-adjacent-obj-header-directory"),
            (
                SEQUENCE_DIRECTORY,
                self.sequence_relative_offset.to_le_bytes().to_vec(),
                "firaga-adjacent-obj-sequence-offset",
            ),
            (FIRST_SEQUENCE, sequence, "firaga-adjacent-obj-sequence"),
        ])
    }
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        fn word(offset: u32, at: u32, value: u32) -> Option<u8> {
            value
                .to_le_bytes()
                .get(offset.checked_sub(at)? as usize)
                .copied()
        }
        fn halfs(offset: u32, at: u32, values: &[u16]) -> Option<u8> {
            let index = offset.checked_sub(at)? as usize;
            values
                .get(index / 2)
                .map(|value| value.to_le_bytes()[index % 2])
        }
        if let Some(byte) = word(offset, COUNT_LITERAL, self.count_literal) {
            return Some((byte, 0));
        }
        if offset == COUNT_BYTE {
            return Some((self.count, 1));
        }
        if let Some(byte) = halfs(offset, FIRST_FIELDS, &self.first_descriptor_fields) {
            return Some((byte, 2));
        }
        if let Some(byte) = word(offset, SECOND_DESCRIPTOR, self.second_bank_pointer) {
            return Some((byte, 3));
        }
        if let Some(byte) = halfs(
            offset,
            SECOND_DESCRIPTOR + 4,
            &self.second_descriptor_fields,
        ) {
            return Some((byte, 3));
        }
        if let Some(byte) = halfs(offset, BANK_ROOT, &[self.flags, self.split_offset]) {
            return Some((byte, 4));
        }
        if let Some(byte) = halfs(offset, BANK_ROOT + 4, &self.list_offsets) {
            return Some((byte, 4));
        }
        if let Some(byte) = halfs(offset, SEQUENCE_DIRECTORY, &[self.sequence_relative_offset]) {
            return Some((byte, 5));
        }
        if let Some(index) = offset.checked_sub(FIRST_SEQUENCE) {
            let index = index as usize;
            if index < 2 {
                return Some(((self.frames.len() as u16).to_le_bytes()[index], 6));
            }
            let index = index - 2;
            if let Some(frame) = self.frames.get(index / 2) {
                return Some((
                    if index % 2 == 0 {
                        frame.list
                    } else {
                        frame.duration
                    },
                    6,
                ));
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Adjacent Firaga compact OBJ bank differs from EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FiragaAdjacentCompactObjBank,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err(
                "Adjacent Firaga compact OBJ bank requires exact EUR ROM; USA pending".into(),
            );
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0xeb8e2, 0x4b35),
            (0xeb8e4, 0x782a),
            (0xeb8e6, 0x18d2),
            (0xeb8e8, 0x7812),
            (0xeb8f2, 0x4b32),
            (0xeb8fa, 0x6812),
            (0xeb906, 0xf7ee),
            (0xeb908, 0xf94f),
            (0xd9bac, 0x9c12),
            (0xd9bb0, 0x9e14),
            (0xd9bd2, 0x9604),
            (0xd896a, 0x0048),
            (0xd896c, 0x4450),
            (0xd896e, 0x00c0),
            (0xd8970, 0x69a9),
            (0xd8974, 0x8af0),
            (0xd89c6, 0x6831),
            (0xd89cc, 0x788a),
            (0xd89ce, 0x78c8),
            (0xd89d4, 0x1808),
            (0xd8a10, 0x88b2),
            (0xd8afc, 0x8af1),
            (0xd8b60, 0x6831),
        ];
        if checks.iter().any(|&(at, value)| half(at) != value)
            || word(0xd9c20) != 0x080d88fd
            || word(FIRST_POINTER as usize) != 0x086b3578
            || half(FIRST_SEQUENCE as usize) != 17
        {
            return Err("Adjacent Firaga OBJ reader fingerprint mismatch".into());
        }
        let decoded = FiragaAdjacentCompactObjBank {
            count_literal: word(COUNT_LITERAL as usize),
            count: rom[COUNT_BYTE as usize],
            first_descriptor_fields: std::array::from_fn(|i| half(FIRST_FIELDS as usize + 2 * i)),
            second_bank_pointer: word(SECOND_DESCRIPTOR as usize),
            second_descriptor_fields: std::array::from_fn(|i| {
                half(SECOND_DESCRIPTOR as usize + 4 + 2 * i)
            }),
            flags: half(BANK_ROOT as usize),
            split_offset: half(BANK_ROOT as usize + 2),
            list_offsets: std::array::from_fn(|i| half(BANK_ROOT as usize + 4 + 2 * i)),
            sequence_relative_offset: half(SEQUENCE_DIRECTORY as usize),
            frames: (0..17)
                .map(|i| compact_obj_animations::Frame {
                    list: rom[FIRST_SEQUENCE as usize + 2 + 2 * i],
                    duration: rom[FIRST_SEQUENCE as usize + 3 + 2 * i],
                })
                .collect(),
        };
        let prior = PriorSources::new(previous)?;
        if !previous.is_empty() {
            let old = previous
                .iter()
                .find(|resource| resource.id == FIRAGA_COMPACT_OBJ_BANK_ID)
                .ok_or("Missing prior Firaga compact bank owner")?;
            if !old
                .sources
                .iter()
                .any(|source| source.offset == FIRST_POINTER && source.length == 4)
            {
                return Err("Missing prior Firaga first descriptor pointer".into());
            }
            let objects = previous
                .iter()
                .find(|resource| resource.id == OBJ_STATIC_LISTS_ID)
                .ok_or("Missing Firaga OBJ list owner")?;
            for (i, offset) in LIST_OFFSETS.into_iter().enumerate() {
                let at = BANK_ROOT + 4 + u32::from(offset);
                let count = usize::from(half(at as usize));
                if !objects
                    .sources
                    .iter()
                    .any(|source| source.offset == at && source.length as usize == 2 + count * 6)
                    || (i == 9 && obj_lists::FIRAGA_ADJACENT_EMPTY_LIST.0 != at)
                {
                    return Err("Missing adjacent Firaga OBJ list source".into());
                }
            }
        }
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Adjacent Firaga OBJ source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Adjacent Firaga OBJ source hash mismatch".into());
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
