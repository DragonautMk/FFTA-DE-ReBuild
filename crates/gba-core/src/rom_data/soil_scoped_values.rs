//! Authenticated EUR Soil values observed at six indexed source locations.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SPANS: [(u32, usize); 6] = [
    (0x39a498, 4),
    (0x39a828, 4),
    (0x39a878, 1),
    (0x39aa84, 1),
    (0x39abf4, 4),
    (0x39ac14, 2),
];
const HASHES: [&str; 6] = [
    "276b3ca9faa03714364c3fa5aa0e8f0797e4e64319085afd89404ddeef599141",
    "0f96f525eccde306c9a85b2ac7adc907d0f5772a3481f62780db88742aec435a",
    "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a",
    "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a",
    "8ababe54cd4854b81dc3f07477eefcbc4afdfe9a158cea9a07d24a4523a26338",
    "8cd4ae7da223953050b55b0ee9e4d3830f3b35138a0e1b6cc5f6e6b1e52d1314",
];
const ROLES: [&str; 6] = [
    "soil-scoped-word-a498",
    "soil-scoped-word-a828",
    "soil-scoped-byte-a878",
    "soil-scoped-byte-aa84",
    "soil-scoped-thumb-pointer-abf4",
    "soil-scoped-halfword-ac14",
];

/// Keep unknown meanings opaque while preserving the widths and Thumb pointer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilScopedValues {
    pub word_a498: u32,
    pub word_a828: u32,
    pub byte_a878: u8,
    pub byte_aa84: u8,
    pub thumb_pointer_abf4: u32,
    pub halfword_ac14: u16,
}
impl SoilScopedValues {
    pub fn validate(&self) -> Result<(), String> {
        if self.thumb_pointer_abf4 & 1 == 0 {
            return Err("Soil scoped Thumb pointer must retain its mode bit".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let bytes = [
            self.word_a498.to_le_bytes().to_vec(),
            self.word_a828.to_le_bytes().to_vec(),
            vec![self.byte_a878],
            vec![self.byte_aa84],
            self.thumb_pointer_abf4.to_le_bytes().to_vec(),
            self.halfword_ac14.to_le_bytes().to_vec(),
        ];
        Ok(SPANS
            .into_iter()
            .zip(bytes)
            .zip(ROLES)
            .map(|(((offset, _), bytes), role)| (offset, bytes, role))
            .collect())
    }
    /// Only witnessed physical entries are owned; guest indices remain unrestricted.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        let entries = [
            (SPANS[0].0, self.word_a498.to_le_bytes(), 4),
            (SPANS[1].0, self.word_a828.to_le_bytes(), 4),
            (SPANS[2].0, [self.byte_a878, 0, 0, 0], 1),
            (SPANS[3].0, [self.byte_aa84, 0, 0, 0], 1),
            (SPANS[4].0, self.thumb_pointer_abf4.to_le_bytes(), 4),
            (
                SPANS[5].0,
                [
                    self.halfword_ac14 as u8,
                    (self.halfword_ac14 >> 8) as u8,
                    0,
                    0,
                ],
                2,
            ),
        ];
        entries
            .into_iter()
            .enumerate()
            .find_map(|(index, (start, bytes, len))| {
                let at = offset.checked_sub(start)? as usize;
                (at < len).then(|| (bytes[at], index))
            })
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Soil scoped values differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: SoilScopedValues,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Soil scoped values require the exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0xdad60,
                0xdad68,
                "f63580bfd5836104226d5449d62ddb80da9c3ccda9d2ef2fb29256939fb1c283",
            ),
            (
                0xe61c4,
                0xe61dc,
                "cc8ba3d0c8a23c1398fd3659be6f9d3e9093f6545b700ab732834d25324dac48",
            ),
            (
                0xe6214,
                0xe6226,
                "4910e55f2dbdbd640d80862aa1632f69340abd7d3ffc59752e244408b41f838a",
            ),
            (
                0xe4ae4,
                0xe4aec,
                "c2fda133be28645f4e790799e0afb623eee2cd9144719667012c151ea231fb9f",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Soil scoped value reader fingerprint mismatch".into());
            }
        }
        let roots = [
            (0xdad60u32, 0xdadccu32, 0x39a498u32, 0usize, 4usize),
            (0xe61c4, 0xe61e8, 0x39a818, 4, 4),
            (0xe61d2, 0xe61ec, 0x39a874, 4, 1),
            (0xe4ae4, 0xe4b7c, 0x39aa80, 4, 1),
            (0xe6214, 0xe6238, 0x39abf4, 0, 4),
            (0xe621e, 0xe623c, 0x39ac14, 0, 2),
        ];
        let literals = previous.iter().find(|r| r.id == THUMB_PC_LITERALS_ID);
        if !previous.is_empty() && literals.is_none() {
            return Err("Missing Soil scoped value literal dependency".into());
        }
        for (i, (pc, literal, base, index, width)) in roots.into_iter().enumerate() {
            let op = u16::from_le_bytes(rom[pc as usize..pc as usize + 2].try_into().unwrap());
            let root = u32::from_le_bytes(
                rom[literal as usize..literal as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(op & 255) * 4 != literal
                || root != 0x0800_0000 + base
                || (base + (index * width) as u32, width) != SPANS[i]
            {
                return Err("Soil scoped value root mismatch".into());
            }
            if literals.is_some_and(|owner| {
                !owner
                    .sources
                    .iter()
                    .any(|s| s.offset == literal && s.length == 4)
            }) {
                return Err("Missing separately owned Soil scoped value literal".into());
            }
        }
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let decoded = SoilScopedValues {
            word_a498: word(0x39a498),
            word_a828: word(0x39a828),
            byte_a878: rom[0x39a878],
            byte_aa84: rom[0x39aa84],
            thumb_pointer_abf4: word(0x39abf4),
            halfword_ac14: u16::from_le_bytes(rom[0x39ac14..0x39ac16].try_into().unwrap()),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Soil scoped value source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Soil scoped value source hash mismatch".into());
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
