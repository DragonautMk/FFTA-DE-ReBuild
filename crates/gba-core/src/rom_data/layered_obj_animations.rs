//! Conditional EUR layered OBJ banks. Guest task state and timing stay live.
use super::{obj_animations, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const DESCRIPTOR_ROOT: u32 = 0x3ae1a4;
pub const DESCRIPTOR_BYTES: usize = 28;
pub const BANK_SHAPES: &[(u32, u32, usize, &[usize])] = &[
    (0xb8fdf0, 0xb8ef84, 80, &[29, 15, 29]),
    (0xb90568, 0xb8f0c0, 85, &[33, 17, 33]),
    (0xb93cb8, 0xb8f818, 2, &[1]),
    (0xb90bb0, 0xb8f224, 56, &[57, 1, 1]),
    (0xb91de0, 0xb8f328, 56, &[57, 1, 1]),
    (0xb9369c, 0xb8f6e0, 67, &[30, 12, 30]),
    (0xb930d4, 0xb8f54c, 9, &[9, 9]),
    (0xb8f854, 0xb8ee88, 46, &[17, 9, 17]),
    (0xb93df4, 0xb8f824, 2, &[1]),
    (0xb92ad4, 0xb8f42c, 65, &[27, 9, 27]),
    (0xb93158, 0xb8f5a4, 84, &[29, 15, 29]),
    (0xb93e1c, 0xb8f830, 2, &[1]),
    (0xb93f3c, 0xb8f848, 2, &[1]),
    (0xb930c0, 0xb8f540, 2, &[1]),
    (0xb93ec8, 0xb8f83c, 2, &[1]),
    (0xb8fdac, 0xb8ef4c, 6, &[5, 5]),
];
const PRODUCERS: &[(usize, u8)] = &[
    (0x140fd0, 0),
    (0x140fea, 1),
    (0x141006, 3),
    (0x141022, 4),
    (0x14103e, 2),
    (0x1411d0, 5),
    (0x1411ec, 6),
    (0x141452, 7),
    (0x14146e, 8),
    (0x1415c6, 9),
    (0x1415e2, 10),
    (0x1415fc, 11),
    (0x141618, 12),
    (0x1417e6, 13),
    (0x141802, 15),
    (0x14181e, 14),
];
pub type SourceSpan = (u32, Vec<u8>, String);

/// The unclassified bytes retain the exact remaining descriptor storage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub object_root: u32,
    pub sequence_root: u32,
    pub opaque: [u8; 20],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayeredObjAnimations {
    pub descriptors: Vec<Descriptor>,
    pub banks: Vec<obj_animations::Bank>,
}

impl LayeredObjAnimations {
    pub fn validate(&self) -> Result<(), String> {
        if self.descriptors.len() != BANK_SHAPES.len() || self.banks.len() != BANK_SHAPES.len() {
            return Err("Invalid layered OBJ descriptor/bank count".into());
        }
        for ((descriptor, bank), &(objects, sequences, lists, frames)) in
            self.descriptors.iter().zip(&self.banks).zip(BANK_SHAPES)
        {
            bank.validate()?;
            if (descriptor.object_root, descriptor.sequence_root) != (objects, sequences)
                || (bank.object_root, bank.sequence_root) != (objects, sequences)
                || bank.lists.len() != lists
                || bank.sequences.len() != frames.len()
                || bank
                    .sequences
                    .iter()
                    .zip(frames)
                    .any(|(seq, &count)| seq.frames.len() != count || seq.zero_count_hold.is_some())
            {
                return Err("Invalid declared layered OBJ bank shape".into());
            }
        }
        let spans = self.source_spans_unchecked()?;
        if spans
            .windows(2)
            .any(|v| u64::from(v[0].0) + v[0].1.len() as u64 > u64::from(v[1].0))
        {
            return Err("Overlapping layered OBJ source spans".into());
        }
        Ok(())
    }

    pub fn bank(&self, descriptor_index: u8) -> Option<&obj_animations::Bank> {
        self.banks.get(usize::from(descriptor_index))
    }

    fn source_spans_unchecked(&self) -> Result<Vec<SourceSpan>, String> {
        let mut descriptors = Vec::with_capacity(self.descriptors.len() * DESCRIPTOR_BYTES);
        for d in &self.descriptors {
            for root in [d.object_root, d.sequence_root] {
                descriptors.extend(
                    root.checked_add(0x08000000)
                        .ok_or("Layered OBJ descriptor pointer overflow")?
                        .to_le_bytes(),
                );
            }
            descriptors.extend(d.opaque);
        }
        let mut spans = vec![(
            DESCRIPTOR_ROOT,
            descriptors,
            "layered-obj-descriptors".into(),
        )];
        for bank in &self.banks {
            for (at, offsets, role) in [
                (
                    bank.object_root,
                    &bank.object_offsets,
                    "layered-obj-list-index",
                ),
                (
                    bank.sequence_root,
                    &bank.sequence_offsets,
                    "layered-obj-sequence-index",
                ),
            ] {
                spans.push((
                    at,
                    offsets.iter().flat_map(|v| v.to_le_bytes()).collect(),
                    role.into(),
                ));
            }
            for list in &bank.lists {
                spans.push((
                    list.source_offset,
                    list.source_bytes()?,
                    "layered-obj-list".into(),
                ));
            }
            for sequence in &bank.sequences {
                spans.push((
                    sequence.source_offset,
                    sequence.source_bytes()?,
                    "layered-obj-sequence".into(),
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
            return Err("Layered OBJ banks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: LayeredObjAnimations,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Layered OBJ banks require exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0x141a24,
                0x141a88,
                "4739d54958cce203c5de97fd7bff36496b73b18a9d243119957a8f0894430b45",
            ),
            (
                0x142590,
                0x142664,
                "f4834f70593b895d7f69d6a8ddadc6e4a808a2b5ec52d9e29c02c5a777f793c6",
            ),
            (
                0x142664,
                0x142780,
                "ec00d9f806de57ab0b198f9ead2f3e6c62099ab4e8c0ab5aea1e3dcf230b27c9",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Original layered OBJ producer/reader fingerprint mismatch".into());
            }
        }
        for &(call, index) in PRODUCERS {
            let half = |at| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            let (hi, lo) = (half(call), half(call + 2));
            let displacement =
                (((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
            if half(call - 2) != 0x2000 | u16::from(index)
                || hi & 0xf800 != 0xf000
                || lo & 0xf800 != 0xf800
                || (call as u32 + 4).wrapping_add_signed(displacement) != 0x141a24
            {
                return Err("Original layered OBJ producer call mismatch".into());
            }
        }
        let mut descriptors = Vec::with_capacity(BANK_SHAPES.len());
        let mut banks = Vec::with_capacity(BANK_SHAPES.len());
        for (index, &(object_root, sequence_root, _, _)) in BANK_SHAPES.iter().enumerate() {
            let at = DESCRIPTOR_ROOT as usize + index * DESCRIPTOR_BYTES;
            let word =
                |offset| u32::from_le_bytes(rom[at + offset..at + offset + 4].try_into().unwrap());
            if word(0) != object_root + 0x08000000 || word(4) != sequence_root + 0x08000000 {
                return Err("Original layered OBJ descriptor roots mismatch".into());
            }
            descriptors.push(Descriptor {
                object_root,
                sequence_root,
                opaque: rom[at + 8..at + 28].try_into().unwrap(),
            });
            banks.push(obj_animations::Bank::decode(
                rom,
                object_root,
                sequence_root,
            )?);
        }
        let decoded = LayeredObjAnimations { descriptors, banks };
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
