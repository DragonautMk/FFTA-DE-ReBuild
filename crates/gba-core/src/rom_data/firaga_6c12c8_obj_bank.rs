//! Selected EUR Firaga compact OBJ task at bank 6c12c8.
use super::{
    compact_obj_animations, obj_lists, script39_registry_poses::PriorSources, Resource,
    SourceRange, OBJ_STATIC_LISTS_ID, THUMB_PC_LITERALS_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const COUNT_BYTE: u32 = 0x39aa82;
pub const DESCRIPTOR_POINTER: u32 = 0x39aa2c;
pub const DESCRIPTOR: u32 = 0x39a8bc;
pub const BANK_ROOT: u32 = 0x6c12c8;
pub const SEQUENCE_ROOT: u32 = 0x6c13e0;
pub const LIST_OFFSETS: [u16; 10] = [20, 40, 60, 98, 136, 174, 212, 232, 252, 272];
const HASHES: [&str; 5] = [
    "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a",
    "836fe0a825765eba76cf07cfc4d14337df22e4ba6b105424c9028fd36b29288f",
    "8387ba0ebc00c1547254721f7b9dec0214dcecf0ab93941aa6a138bae9a780b3",
    "c943549a631c32f15cc0c395b75a03ea7ee78b8cec9d9d0ea7adcfb0f269de2e",
    "29821b54c282bc7426dff04bf3e43bdcc74d32dea4e7d7f809c26c0fe6e1046f",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Firaga6c12c8ObjBank {
    pub count: u8,
    pub descriptor_pointer: u32,
    pub bank_pointer: u32,
    pub descriptor_fields: [u16; 10],
    pub flags: u16,
    pub split_offset: u16,
    pub list_offsets: [u16; 10],
    pub sequence_relative_offset: u16,
    pub frames: Vec<compact_obj_animations::Frame>,
}
impl Firaga6c12c8ObjBank {
    pub fn validate(&self) -> Result<(), String> {
        if (
            self.count,
            self.descriptor_pointer,
            self.bank_pointer,
            self.flags,
            self.split_offset,
            self.sequence_relative_offset,
        ) != (1, 0x0839a8bc, 0x086c12c8, 12, 0x118, 2)
            || self.descriptor_fields[0] != 0
            || self.list_offsets != LIST_OFFSETS
            || self.frames.len() != 13
            || self
                .frames
                .iter()
                .any(|frame| usize::from(frame.list) >= self.list_offsets.len())
        {
            return Err("Invalid Firaga 6c12c8 compact OBJ bank shape".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut descriptor = self.bank_pointer.to_le_bytes().to_vec();
        for value in self.descriptor_fields {
            descriptor.extend(value.to_le_bytes());
        }
        let mut header = self.flags.to_le_bytes().to_vec();
        header.extend(self.split_offset.to_le_bytes());
        for offset in self.list_offsets {
            header.extend(offset.to_le_bytes());
        }
        let mut sequence = self.sequence_relative_offset.to_le_bytes().to_vec();
        sequence.extend((self.frames.len() as u16).to_le_bytes());
        for frame in &self.frames {
            sequence.extend([frame.list, frame.duration]);
        }
        Ok(vec![
            (COUNT_BYTE, vec![self.count], "firaga-6c12c8-obj-count"),
            (
                DESCRIPTOR_POINTER,
                self.descriptor_pointer.to_le_bytes().to_vec(),
                "firaga-6c12c8-obj-descriptor-pointer",
            ),
            (DESCRIPTOR, descriptor, "firaga-6c12c8-obj-descriptor"),
            (BANK_ROOT, header, "firaga-6c12c8-obj-header-directory"),
            (SEQUENCE_ROOT, sequence, "firaga-6c12c8-obj-sequence"),
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
        if offset == COUNT_BYTE {
            return Some((self.count, 0));
        }
        if let Some(byte) = word(offset, DESCRIPTOR_POINTER, self.descriptor_pointer) {
            return Some((byte, 1));
        }
        if let Some(byte) = word(offset, DESCRIPTOR, self.bank_pointer) {
            return Some((byte, 2));
        }
        if let Some(byte) = halfs(offset, DESCRIPTOR + 4, &self.descriptor_fields) {
            return Some((byte, 2));
        }
        if let Some(byte) = halfs(offset, BANK_ROOT, &[self.flags, self.split_offset]) {
            return Some((byte, 3));
        }
        if let Some(byte) = halfs(offset, BANK_ROOT + 4, &self.list_offsets) {
            return Some((byte, 3));
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
            return Err("Firaga 6c12c8 OBJ bank differs from EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: Firaga6c12c8ObjBank,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga 6c12c8 OBJ bank requires exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0xe4ae4, 0x4b25),
            (0xe4ae6, 0x78aa),
            (0xe4ae8, 0x18d2),
            (0xe4aea, 0x7812),
            (0xe4af4, 0x4b22),
            (0xe4af6, 0x78aa),
            (0xe4af8, 0x0092),
            (0xe4afa, 0x18d2),
            (0xe4afc, 0x6812),
            (0xe4b06, 0xf7f5),
            (0xe4b08, 0xf84f),
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
        ];
        if checks.iter().any(|&(at, value)| half(at) != value)
            || word(0xe4b7c) != 0x0839aa80
            || word(0xe4b80) != 0x0839aa24
            || half(SEQUENCE_ROOT as usize + 2) != 13
        {
            return Err("Firaga 6c12c8 OBJ reader fingerprint mismatch".into());
        }
        let decoded = Firaga6c12c8ObjBank {
            count: rom[COUNT_BYTE as usize],
            descriptor_pointer: word(DESCRIPTOR_POINTER as usize),
            bank_pointer: word(DESCRIPTOR as usize),
            descriptor_fields: std::array::from_fn(|i| half(DESCRIPTOR as usize + 4 + 2 * i)),
            flags: half(BANK_ROOT as usize),
            split_offset: half(BANK_ROOT as usize + 2),
            list_offsets: std::array::from_fn(|i| half(BANK_ROOT as usize + 4 + 2 * i)),
            sequence_relative_offset: half(SEQUENCE_ROOT as usize),
            frames: (0..13)
                .map(|i| compact_obj_animations::Frame {
                    list: rom[SEQUENCE_ROOT as usize + 4 + 2 * i],
                    duration: rom[SEQUENCE_ROOT as usize + 5 + 2 * i],
                })
                .collect(),
        };
        let prior = PriorSources::new(previous)?;
        if !previous.is_empty() {
            let literals = previous
                .iter()
                .find(|resource| resource.id == THUMB_PC_LITERALS_ID)
                .ok_or("Missing Firaga indexed caller literal owner")?;
            if [0xe4b7c, 0xe4b80].into_iter().any(|at| {
                !literals
                    .sources
                    .iter()
                    .any(|source| source.offset == at && source.length == 4)
            }) {
                return Err("Missing Firaga indexed caller literals".into());
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
                    || (i == 9 && obj_lists::FIRAGA_6C12C8_EMPTY_LIST.0 != at)
                {
                    return Err("Missing Firaga 6c12c8 OBJ list source".into());
                }
            }
        }
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga 6c12c8 OBJ source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga 6c12c8 OBJ source hash mismatch".into());
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
