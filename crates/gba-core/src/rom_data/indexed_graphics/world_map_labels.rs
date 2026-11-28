//! European world-map label selectors and their counted OBJ variants.

use super::{SourceSpan, additional};
use serde::{Deserialize, Serialize};

const BASE: u32 = 0x0800_0000;
const VARIANTS: u32 = 0x395ae4;
const PROGRAM_OFFSETS: u32 = 0x395b7a;
const OBJ_DIRECTORY: u32 = 0x39573c;
const OBJ_LISTS: [(u32, usize); 7] = [
    (0x3956c2, 2),
    (0x3956d0, 2),
    (0x3956de, 3),
    (0x3956f2, 2),
    (0x395700, 3),
    (0x395714, 3),
    (0x395728, 3),
];
const LITERALS: [(u32, u32); 6] = [
    (0x037c0c, BASE + VARIANTS),
    (0x037c20, BASE + OBJ_DIRECTORY),
    (0x037c28, BASE + PROGRAM_OFFSETS),
    (0x037e88, BASE + VARIANTS),
    (0x037e94, BASE + OBJ_DIRECTORY),
    (0x037e9c, BASE + PROGRAM_OFFSETS),
];
const LANGUAGE_COUNT: usize = 5;
const NODE_COUNT: usize = 30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldMapLabels {
    pub variants: [[u8; NODE_COUNT]; LANGUAGE_COUNT],
    pub program_offsets: [[u16; NODE_COUNT]; LANGUAGE_COUNT],
    pub obj_slots: [u32; 8],
    pub obj_lists: [Vec<[u16; 3]>; OBJ_LISTS.len()],
    pub reader_literals: [u32; LITERALS.len()],
}

impl WorldMapLabels {
    pub(super) fn from_rom(rom: &[u8]) -> Self {
        let halfword = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let obj_lists = OBJ_LISTS.map(|(offset, _)| {
            let at = offset as usize;
            (0..usize::from(halfword(at)))
                .map(|index| {
                    let entry = at + 2 + 6 * index;
                    [halfword(entry), halfword(entry + 2), halfword(entry + 4)]
                })
                .collect()
        });
        Self {
            variants: std::array::from_fn(|language| {
                let at = VARIANTS as usize + language * NODE_COUNT;
                rom[at..at + NODE_COUNT].try_into().unwrap()
            }),
            program_offsets: std::array::from_fn(|language| {
                std::array::from_fn(|node| {
                    halfword(PROGRAM_OFFSETS as usize + 2 * (language * NODE_COUNT + node))
                })
            }),
            obj_slots: std::array::from_fn(|slot| word(OBJ_DIRECTORY as usize + 4 * slot)),
            obj_lists,
            reader_literals: LITERALS.map(|(offset, _)| word(offset as usize)),
        }
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.reader_literals != LITERALS.map(|(_, word)| word) {
            return Err("Invalid world-map label reader literals".into());
        }
        for (language, (&variants, &offsets)) in
            self.variants.iter().zip(&self.program_offsets).enumerate()
        {
            let archive_address = additional::POINTERS[1][language];
            let programs = additional::ARCHIVES
                .iter()
                .find(|archive| archive.offset == archive_address)
                .ok_or("Missing world-map label archive")?
                .programs;
            let mut slices = [(0, 0); NODE_COUNT];
            for ((variant, offset), slice) in variants.into_iter().zip(offsets).zip(&mut slices) {
                if offset & 1 != 0 || usize::from(variant) >= self.obj_slots.len() {
                    return Err("Invalid world-map label program or variant index".into());
                }
                let start = usize::from(offset) / 2;
                *slice = (start, start + usize::from(variant) + 5);
            }
            slices.sort_unstable();
            if slices[0].0 != 0
                || slices[NODE_COUNT - 1].1 != programs
                || !slices.windows(2).all(|pair| pair[0].1 == pair[1].0)
            {
                return Err("World-map label slices must cover the indexed archive".into());
            }
        }

        let mut obj_bytes = Vec::new();
        for (index, (entries, (address, count))) in self.obj_lists.iter().zip(OBJ_LISTS).enumerate()
        {
            if self.obj_slots[index] != BASE + address || entries.len() != count {
                return Err("Invalid world-map label OBJ list".into());
            }
            obj_bytes.extend((count as u16).to_le_bytes());
            for entry in entries {
                for field in entry {
                    obj_bytes.extend(field.to_le_bytes());
                }
            }
        }
        if self.obj_slots[7] != self.obj_slots[6] {
            return Err("Invalid world-map label OBJ alias".into());
        }

        let mut spans: Vec<_> = LITERALS
            .into_iter()
            .zip(self.reader_literals)
            .map(|((offset, _), value)| {
                (
                    offset,
                    value.to_le_bytes().to_vec(),
                    "label-reader-literal".into(),
                )
            })
            .collect();
        spans.push((OBJ_LISTS[0].0, obj_bytes, "label-obj-lists".into()));
        spans.push((
            OBJ_DIRECTORY,
            self.obj_slots
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect(),
            "label-obj-directory".into(),
        ));
        spans.push((
            VARIANTS,
            self.variants.iter().flatten().copied().collect(),
            "label-variant-rows".into(),
        ));
        spans.push((
            PROGRAM_OFFSETS,
            self.program_offsets
                .iter()
                .flatten()
                .flat_map(|value| value.to_le_bytes())
                .collect(),
            "label-program-rows".into(),
        ));
        Ok(spans)
    }
}

pub(crate) fn is_source_start(offset: u32) -> bool {
    matches!(offset, VARIANTS | PROGRAM_OFFSETS | OBJ_DIRECTORY)
        || offset == OBJ_LISTS[0].0
        || LITERALS.iter().any(|&(at, _)| at == offset)
}
