//! EUR Firaga compact OBJ sequence and selected indirect roots.
use super::{
    compact_obj_animations, obj_lists, script39_registry_poses::PriorSources, Resource,
    SourceRange, OBJ_STATIC_LISTS_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const CALLER_LITERAL: u32 = 0x0eb9bc;
pub const DESCRIPTOR_ROOT: u32 = 0x39cc3c;
pub const SELECTED_POINTER: u32 = 0x39cc74;
pub const BANK_ROOT: u32 = 0x6b3578;
pub const SEQUENCE_ROOT: u32 = 0x6b3694;
pub const LIST_OFFSETS: [u16; 10] = [20, 28, 42, 68, 106, 162, 218, 274, 276, 278];
const HASHES: [&str; 5] = [
    "698e9979fac148fb6945f90e9eef9c3278de1973a266b2982775665bd32ebf08",
    "3a522b82f9b14ac7d12465c7e29d04c36e2fd7303e84f623953a237d1764870e",
    "e2e9512418db724859725e34a8ff4ddb7bc73815eb6aa18017b96fc39784f04c",
    "ecbee3672230cd493cb143d48b9c07a60ee53581e82a45cf79815230ea3a205f",
    "195e7dd9bc3c03bbaf8421c3a11c24b946724949ec8d808514cbbb74936c78da",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaCompactObjBank {
    pub caller_literal: u32,
    pub descriptor_root: u32,
    pub selected_pointer: u32,
    pub flags: u16,
    pub split_offset: u16,
    pub list_offsets: [u16; 10],
    pub sequence_relative_offset: u16,
    pub frames: Vec<compact_obj_animations::Frame>,
}
impl FiragaCompactObjBank {
    pub fn validate(&self) -> Result<(), String> {
        if (
            self.caller_literal,
            self.descriptor_root,
            self.selected_pointer,
            self.flags,
            self.split_offset,
            self.sequence_relative_offset,
        ) != (0x0839cc6c, 0x086b3578, 0x0839cc3c, 12, 284, 2)
            || self.list_offsets != LIST_OFFSETS
            || self.frames.len() != 10
            || self
                .frames
                .iter()
                .any(|frame| usize::from(frame.list) >= self.list_offsets.len())
        {
            return Err("Invalid Firaga compact OBJ bank shape".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut header = Vec::new();
        header.extend(self.flags.to_le_bytes());
        header.extend(self.split_offset.to_le_bytes());
        for offset in self.list_offsets {
            header.extend(offset.to_le_bytes());
        }
        let mut sequence = Vec::new();
        sequence.extend(self.sequence_relative_offset.to_le_bytes());
        sequence.extend((self.frames.len() as u16).to_le_bytes());
        for frame in &self.frames {
            sequence.extend([frame.list, frame.duration]);
        }
        Ok(vec![
            (
                CALLER_LITERAL,
                self.caller_literal.to_le_bytes().to_vec(),
                "firaga-compact-obj-caller-literal",
            ),
            (
                DESCRIPTOR_ROOT,
                self.descriptor_root.to_le_bytes().to_vec(),
                "firaga-compact-obj-descriptor-root",
            ),
            (
                SELECTED_POINTER,
                self.selected_pointer.to_le_bytes().to_vec(),
                "firaga-compact-obj-selected-pointer",
            ),
            (BANK_ROOT, header, "firaga-compact-obj-header-directory"),
            (SEQUENCE_ROOT, sequence, "firaga-compact-obj-sequence"),
        ])
    }
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        for (at, value, owner) in [
            (CALLER_LITERAL, self.caller_literal, 0),
            (DESCRIPTOR_ROOT, self.descriptor_root, 1),
            (SELECTED_POINTER, self.selected_pointer, 2),
        ] {
            if let Some(index) = offset.checked_sub(at) {
                if let Some(byte) = value.to_le_bytes().get(index as usize) {
                    return Some((*byte, owner));
                }
            }
        }
        if let Some(index) = offset.checked_sub(BANK_ROOT) {
            let index = index as usize;
            if index < 4 {
                let header = [
                    self.flags as u8,
                    (self.flags >> 8) as u8,
                    self.split_offset as u8,
                    (self.split_offset >> 8) as u8,
                ];
                return Some((header[index], 3));
            }
            let index = index - 4;
            if let Some(value) = self.list_offsets.get(index / 2) {
                return Some((value.to_le_bytes()[index % 2], 3));
            }
        }
        if let Some(index) = offset.checked_sub(SEQUENCE_ROOT) {
            let index = index as usize;
            if index < 2 {
                return Some((self.sequence_relative_offset.to_le_bytes()[index], 4));
            }
            if index < 4 {
                return Some(((self.frames.len() as u16).to_le_bytes()[index - 2], 4));
            }
            let index = index - 4;
            if let Some(frame) = self.frames.get(index / 2) {
                return Some((
                    if index % 2 == 0 {
                        frame.list
                    } else {
                        frame.duration
                    },
                    4,
                ));
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga compact OBJ bank differs from EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FiragaCompactObjBank,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga compact OBJ bank requires exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0xeb8f2, 0x4b32),
            (0xeb8f4, 0x782a),
            (0xeb8f6, 0x0092),
            (0xeb8f8, 0x18d2),
            (0xeb8fa, 0x6812),
            (0xeb906, 0xf7ee),
            (0xeb908, 0xf94f),
            (0xd99c0, 0x1d01),
            (0xd99c4, 0x7882),
            (0xd99c6, 0x78c1),
            (0xd99cc, 0x1844),
            (0xd981a, 0x1d01),
            (0xd981e, 0x7882),
            (0xd9820, 0x78c1),
            (0xd9826, 0x1846),
        ];
        if checks.iter().any(|&(at, value)| half(at) != value)
            || half(SEQUENCE_ROOT as usize + 2) != 10
        {
            return Err("Firaga compact OBJ reader fingerprint mismatch".into());
        }
        let decoded = FiragaCompactObjBank {
            caller_literal: word(CALLER_LITERAL as usize),
            descriptor_root: word(DESCRIPTOR_ROOT as usize),
            selected_pointer: word(SELECTED_POINTER as usize),
            flags: half(BANK_ROOT as usize),
            split_offset: half(BANK_ROOT as usize + 2),
            list_offsets: std::array::from_fn(|i| half(BANK_ROOT as usize + 4 + 2 * i)),
            sequence_relative_offset: half(SEQUENCE_ROOT as usize),
            frames: (0..10)
                .map(|i| compact_obj_animations::Frame {
                    list: rom[SEQUENCE_ROOT as usize + 4 + 2 * i],
                    duration: rom[SEQUENCE_ROOT as usize + 5 + 2 * i],
                })
                .collect(),
        };
        let prior = PriorSources::new(previous)?;
        if !previous.is_empty() {
            let objects = previous
                .iter()
                .find(|resource| resource.id == OBJ_STATIC_LISTS_ID)
                .ok_or("Missing previous Firaga OBJ list owner")?;
            for (i, offset) in LIST_OFFSETS.into_iter().enumerate() {
                let at = BANK_ROOT + 4 + u32::from(offset);
                let count = usize::from(half(at as usize));
                let length = 2 + count * 6;
                if !objects
                    .sources
                    .iter()
                    .any(|source| source.offset == at && source.length as usize == length)
                    || (i >= 7
                        && !obj_lists::FIRAGA_COMPACT_EMPTY_LISTS
                            .iter()
                            .any(|&(root, _, _)| root == at))
                {
                    return Err("Missing referenced compact Firaga OBJ list source".into());
                }
            }
        }
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga compact OBJ source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga compact OBJ source hash mismatch".into());
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
