//! Four source-rooted OBJ banks for the original u16-counter task.
//! Shared bank fields encode bytes only; the guest owns timing and indexing.
use super::{obj_animations, obj_lists, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const BANK_ROOTS: &[(u32, u32)] = &[
    (0x41634c, 0x411478),
    (0x41647c, 0x4114ac),
    (0x4165a0, 0x4114fc),
    (0x4166c4, 0x41154c),
];
const SHAPES: &[(usize, &[usize])] = &[(11, &[9, 0]), (15, &[16, 0]), (15, &[16, 0]), (9, &[8])];
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WordTaskObjAnimations {
    pub banks: Vec<obj_animations::Bank>,
}
impl WordTaskObjAnimations {
    pub fn validate(&self) -> Result<(), String> {
        if self.banks.len() != BANK_ROOTS.len() {
            return Err("Invalid EUR word-task OBJ bank count".into());
        }
        for ((bank, &(objects, sequences)), &(lists, frames)) in
            self.banks.iter().zip(BANK_ROOTS).zip(SHAPES)
        {
            bank.validate()?;
            if (bank.object_root, bank.sequence_root) != (objects, sequences)
                || bank.lists.len() != lists
                || bank.sequences.len() != frames.len()
                || bank
                    .sequences
                    .iter()
                    .zip(frames)
                    .any(|(s, &count)| s.frames.len() != count || s.zero_count_hold.is_some())
            {
                return Err("Invalid declared word-task OBJ bank shape".into());
            }
        }
        let spans = self.source_spans_unchecked()?;
        for pair in spans.windows(2) {
            if u64::from(pair[0].0) + pair[0].1.len() as u64 > u64::from(pair[1].0) {
                return Err("Overlapping word-task OBJ source spans".into());
            }
        }
        Ok(())
    }
    pub fn bank(&self, object_root: u32) -> Option<&obj_animations::Bank> {
        self.banks.iter().find(|b| b.object_root == object_root)
    }
    /// Only declared conditional entries; never a clamp of the original u16 input.
    pub fn sequence(&self, object_root: u32, selector: u16) -> Option<&obj_animations::Sequence> {
        self.bank(object_root)?.sequences.get(usize::from(selector))
    }
    pub fn frame(
        &self,
        object_root: u32,
        selector: u16,
        phase: u16,
    ) -> Option<&obj_animations::Frame> {
        self.sequence(object_root, selector)?
            .frames
            .get(usize::from(phase))
    }
    pub fn list(&self, object_root: u32, index: u16) -> Option<&obj_lists::List> {
        self.bank(object_root)?.lists.get(usize::from(index))
    }
    fn source_spans_unchecked(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = Vec::new();
        for bank in &self.banks {
            for (at, offsets, role) in [
                (
                    bank.object_root,
                    &bank.object_offsets,
                    "word-task-obj-list-index",
                ),
                (
                    bank.sequence_root,
                    &bank.sequence_offsets,
                    "word-task-obj-sequence-index",
                ),
            ] {
                spans.push((
                    at,
                    offsets.iter().flat_map(|o| o.to_le_bytes()).collect(),
                    role.into(),
                ));
            }
            for list in &bank.lists {
                spans.push((
                    list.source_offset,
                    list.source_bytes()?,
                    "word-task-obj-list".into(),
                ));
            }
            for sequence in &bank.sequences {
                spans.push((
                    sequence.source_offset,
                    sequence.source_bytes()?,
                    "word-task-obj-sequence".into(),
                ));
            }
        }
        spans.sort_by_key(|s| s.0);
        Ok(spans)
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        self.source_spans_unchecked()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.source_spans()
            .ok()?
            .into_iter()
            .find_map(|(at, raw, _)| raw.get(offset.checked_sub(at)? as usize).copied())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Word-task OBJ banks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: WordTaskObjAnimations,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Word-task OBJ banks require exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0x2c5e0,
                0x2c668,
                "67733b8af31a06c532df977d73eeb3c8ef9c061863611aaee36194cf46655ff1",
            ),
            (
                0x54b8e,
                0x54bd0,
                "4e4b91f8cf469a3ce89accb12d0be7b781d894530dbcca163e0af604cbb4412b",
            ),
            (
                0x5e6c4,
                0x5e728,
                "4d31a3112460d7e3f54bec97e9bd4da62af76148d7cc3de5a6e1acf8b2144674",
            ),
            (
                0x12f344,
                0x12f364,
                "492a6c4a06a259666c2c0a6970a05710ffa6cc5f004887ac1869faf7de4d5a88",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Original word-task OBJ reader/producer fingerprint mismatch".into());
            }
        }
        for (
            call,
            object_reader,
            object_literal,
            sequence_reader,
            sequence_literal,
            objects,
            sequences,
        ) in [
            (
                0x54b98, 0x54b90, 0x54bc8, 0x54b92, 0x54bcc, 0x41634c, 0x411478,
            ),
            (
                0x5e6e4, 0x5e6dc, 0x5e6f4, 0x5e6de, 0x5e6f8, 0x4165a0, 0x4114fc,
            ),
            (
                0x5e70c, 0x5e704, 0x5e720, 0x5e706, 0x5e724, 0x41647c, 0x4114ac,
            ),
            (
                0x12f350, 0x12f348, 0x12f35c, 0x12f34a, 0x12f360, 0x4166c4, 0x41154c,
            ),
        ] {
            let half = |at| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            for (reader, literal, root) in [
                (object_reader, object_literal, objects),
                (sequence_reader, sequence_literal, sequences),
            ] {
                let opcode = half(reader);
                if opcode & 0xf800 != 0x4800
                    || ((reader + 4) & !3) + usize::from(opcode & 255) * 4 != literal
                    || u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap())
                        != root + 0x08000000
                {
                    return Err("Word-task OBJ producer root literal mismatch".into());
                }
            }
            let (hi, lo) = (half(call), half(call + 2));
            let displacement =
                (((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
            if hi & 0xf800 != 0xf000
                || lo & 0xf800 != 0xf800
                || (call as u32 + 4).wrapping_add_signed(displacement) != 0x2c5e0
            {
                return Err("Word-task OBJ producer BL mismatch".into());
            }
        }
        let decoded = WordTaskObjAnimations {
            banks: BANK_ROOTS
                .iter()
                .map(|&(o, s)| obj_animations::Bank::decode(rom, o, s))
                .collect::<Result<_, _>>()?,
        };
        let sources = decoded
            .source_spans()?
            .into_iter()
            .map(|(offset, raw, role)| SourceRange {
                offset,
                length: raw.len() as u32,
                sha256: ffta_rom::digest(&raw),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests;
