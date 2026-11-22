//! Selected EUR Firaga indexed sequence and object directory; guest state stays live.
use super::{
    obj_animations, obj_lists, script39_registry_poses::PriorSources, Resource, SourceRange,
    OBJ_STATIC_LISTS_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const DESCRIPTOR_OFFSET: u32 = 0x398f90;
pub const SEQUENCE_ROOT: u32 = 0x412050;
pub const SEQUENCE_OFFSET: u32 = 0x412054;
pub const OBJECT_ROOT: u32 = 0x418e9c;
pub const OBJECT_OFFSETS: [u32; 20] = [
    0x50, 0x54, 0x5c, 0x64, 0x74, 0x84, 0x94, 0xa8, 0xbc, 0xd8, 0xf4, 0x110, 0x12c, 0x148, 0x164,
    0x184, 0x1a4, 0x1c4, 0x1e4, 0x204,
];
const HASHES: [&str; 4] = [
    "ddffa86bafd358d1bede870549b8d0d481e29ebf40a2c3040c5c22db8b021c65",
    "fb5e512425fc9449316ec95969ebe71e2d576dbab833d61e2a5b9330fd70ee02",
    "0964d5e9802d5bfb785c8260e0aaa179051bce31b150a67a1c4c9ce7cb9724d4",
    "66017e9ee79b10f5ae1634d2695b96edb97f757dd1f3f032eef70fe09b2a23e8",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaIndexedObjBank {
    pub sequence_pointer: u32,
    pub object_pointer: u32,
    /// Original descriptor halfwords; their complete semantics remain unknown.
    pub descriptor_fields: [u16; 2],
    pub sequence_relative_offset: u32,
    pub sequence: obj_animations::Sequence,
    pub object_offsets: [u32; 20],
}
impl FiragaIndexedObjBank {
    pub fn validate(&self) -> Result<(), String> {
        if (
            self.sequence_pointer,
            self.object_pointer,
            self.descriptor_fields,
            self.sequence_relative_offset,
        ) != (0x08412050, 0x08418e9c, [127, 128], 4)
            || self.sequence.source_offset != SEQUENCE_OFFSET
            || self.sequence.zero_count_hold.is_some()
            || self.sequence.frames.len() != 19
            || self.object_offsets != OBJECT_OFFSETS
            || self
                .sequence
                .frames
                .iter()
                .any(|frame| usize::from(frame.object_list) >= self.object_offsets.len())
        {
            return Err("Invalid Firaga indexed OBJ bank shape".into());
        }
        if self.sequence.source_bytes()?.len() != 78 {
            return Err("Invalid Firaga indexed sequence extent".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut descriptor = Vec::new();
        descriptor.extend(self.sequence_pointer.to_le_bytes());
        descriptor.extend(self.object_pointer.to_le_bytes());
        for field in self.descriptor_fields {
            descriptor.extend(field.to_le_bytes());
        }
        let mut directory = Vec::new();
        for offset in self.object_offsets {
            directory.extend(offset.to_le_bytes());
        }
        Ok(vec![
            (
                DESCRIPTOR_OFFSET,
                descriptor,
                "firaga-indexed-obj-descriptor",
            ),
            (
                SEQUENCE_ROOT,
                self.sequence_relative_offset.to_le_bytes().to_vec(),
                "firaga-indexed-obj-sequence-index",
            ),
            (
                SEQUENCE_OFFSET,
                self.sequence.source_bytes()?,
                "firaga-indexed-obj-sequence",
            ),
            (OBJECT_ROOT, directory, "firaga-indexed-obj-list-index"),
        ])
    }
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        if let Some(index) = offset.checked_sub(DESCRIPTOR_OFFSET) {
            let index = index as usize;
            if index < 4 {
                return Some((self.sequence_pointer.to_le_bytes()[index], 0));
            }
            if index < 8 {
                return Some((self.object_pointer.to_le_bytes()[index - 4], 0));
            }
            if index < 12 {
                return Some((
                    self.descriptor_fields[(index - 8) / 2].to_le_bytes()[(index - 8) % 2],
                    0,
                ));
            }
        }
        if let Some(index) = offset.checked_sub(SEQUENCE_ROOT) {
            if index < 4 {
                return Some((
                    self.sequence_relative_offset.to_le_bytes()[index as usize],
                    1,
                ));
            }
        }
        if let Some(index) = offset.checked_sub(SEQUENCE_OFFSET) {
            let index = index as usize;
            if index < 2 {
                return Some(((self.sequence.frames.len() as u16).to_le_bytes()[index], 2));
            }
            let index = index - 2;
            if let Some(frame) = self.sequence.frames.get(index / 4) {
                let word = if index % 4 < 2 {
                    frame.object_list
                } else {
                    frame.duration
                };
                return Some((word.to_le_bytes()[index % 2], 2));
            }
        }
        if let Some(index) = offset.checked_sub(OBJECT_ROOT) {
            let index = index as usize;
            if let Some(value) = self.object_offsets.get(index / 4) {
                return Some((value.to_le_bytes()[index % 4], 3));
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga indexed OBJ bank differs from EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: FiragaIndexedObjBank,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga indexed OBJ bank requires exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0x4e50a, 0x0b92),
            (0x4e50c, 0x18aa),
            (0x4e50e, 0x6814),
            (0x4e518, 0x1928),
            (0x4e51a, 0x8800),
            (0x4e522, 0x3402),
            (0x13f818, 0x0080),
            (0x13f81c, 0x8800),
            (0x13f81e, 0x0080),
            (0x13f824, 0x6800),
            (0x13f888, 0x8849),
        ];
        if checks.iter().any(|&(at, value)| half(at) != value)
            || half(SEQUENCE_OFFSET as usize) != 19
        {
            return Err("Firaga indexed OBJ original reader fingerprint mismatch".into());
        }
        let frames = (0..19)
            .map(|i| obj_animations::Frame {
                object_list: half(SEQUENCE_OFFSET as usize + 2 + 4 * i),
                duration: half(SEQUENCE_OFFSET as usize + 4 + 4 * i),
            })
            .collect();
        let decoded = FiragaIndexedObjBank {
            sequence_pointer: word(DESCRIPTOR_OFFSET as usize),
            object_pointer: word(DESCRIPTOR_OFFSET as usize + 4),
            descriptor_fields: [
                half(DESCRIPTOR_OFFSET as usize + 8),
                half(DESCRIPTOR_OFFSET as usize + 10),
            ],
            sequence_relative_offset: word(SEQUENCE_ROOT as usize),
            sequence: obj_animations::Sequence {
                source_offset: SEQUENCE_OFFSET,
                frames,
                zero_count_hold: None,
            },
            object_offsets: std::array::from_fn(|i| word(OBJECT_ROOT as usize + 4 * i)),
        };
        let prior = PriorSources::new(previous)?;
        if !previous.is_empty() {
            let objects = previous
                .iter()
                .find(|resource| resource.id == OBJ_STATIC_LISTS_ID)
                .ok_or("Missing previous Firaga OBJ list owner")?;
            for index in std::iter::once(0).chain(2..20) {
                let offset = OBJECT_ROOT + OBJECT_OFFSETS[index];
                let count = usize::from(half(offset as usize));
                let len = 2 + count * 6;
                if !objects
                    .sources
                    .iter()
                    .any(|source| source.offset == offset && source.length as usize == len)
                    || (index == 0 && offset != obj_lists::FIRAGA_EMPTY_LIST.0)
                {
                    return Err("Missing referenced Firaga OBJ list source".into());
                }
            }
        }
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga indexed OBJ source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga indexed OBJ source hash mismatch".into());
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
