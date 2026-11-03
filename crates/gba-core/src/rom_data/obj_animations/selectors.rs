//! Independently rooted language-selected OBJ/sequence pointers, EUR only.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectorTables {
    pub selector_address: u32,
    pub object_table_offset: u32,
    pub sequence_table_offset: u32,
    /// Source ROM offsets. Projection adds the original 08000000 ROM base.
    pub object_roots: Vec<u32>,
    pub sequence_roots: Vec<u32>,
}
impl SelectorTables {
    pub fn select(&self, language_id: u8) -> Option<(u32, u32)> {
        Some((
            *self.object_roots.get(language_id as usize)?,
            *self.sequence_roots.get(language_id as usize)?,
        ))
    }
    pub(super) fn validate(&self, banks: &[Bank]) -> Result<(), String> {
        if self.selector_address != 0x02001f6e
            || self.object_table_offset & 3 != 0
            || self.sequence_table_offset & 3 != 0
            || self.object_roots.is_empty()
            || self.object_roots.len() > 256
            || self.object_roots.len() != self.sequence_roots.len()
        {
            return Err("Invalid OBJ language selector tables".into());
        }
        for (&objects, &sequences) in self.object_roots.iter().zip(&self.sequence_roots) {
            if !banks
                .iter()
                .any(|bank| bank.object_root == objects && bank.sequence_root == sequences)
            {
                return Err("OBJ language selector has no matching bank pair".into());
            }
        }
        Ok(())
    }
    pub(super) fn spans(&self) -> [(u32, usize, &'static str); 2] {
        [
            (
                self.object_table_offset,
                self.object_roots.len() * 4,
                "obj-animation-language-object-table",
            ),
            (
                self.sequence_table_offset,
                self.sequence_roots.len() * 4,
                "obj-animation-language-sequence-table",
            ),
        ]
    }
    pub(crate) fn source_byte(&self, index: usize) -> Option<u8> {
        for (root, values) in [
            (self.object_table_offset, &self.object_roots),
            (self.sequence_table_offset, &self.sequence_roots),
        ] {
            if let Some(at) = index
                .checked_sub(root as usize)
                .filter(|&at| at < values.len() * 4)
            {
                return Some((values[at / 4] + 0x08000000).to_le_bytes()[at % 4]);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SelectorReference {
    pub call: u32,
    pub constructor: u32,
    pub object_reader: u32,
    pub object_literal: u32,
    pub object_language_reader: u32,
    pub sequence_reader: u32,
    pub sequence_literal: u32,
    pub sequence_language_reader: u32,
}
pub const SELECTOR_REFERENCES: &[SelectorReference] = &[
    SelectorReference {
        call: 0x38110,
        constructor: 0x4e494,
        object_reader: 0x380ee,
        object_literal: 0x3811c,
        object_language_reader: 0x380f0,
        sequence_reader: 0x380fa,
        sequence_literal: 0x38120,
        sequence_language_reader: 0x380fc,
    },
    SelectorReference {
        call: 0x38188,
        constructor: 0x4e558,
        object_reader: 0x38166,
        object_literal: 0x381a0,
        object_language_reader: 0x38168,
        sequence_reader: 0x38172,
        sequence_literal: 0x381a4,
        sequence_language_reader: 0x38174,
    },
    SelectorReference {
        call: 0x381e0,
        constructor: 0x4e494,
        object_reader: 0x381be,
        object_literal: 0x38204,
        object_language_reader: 0x381c0,
        sequence_reader: 0x381ca,
        sequence_literal: 0x38208,
        sequence_language_reader: 0x381cc,
    },
];

pub(super) fn decode_europe(rom: &[u8]) -> Result<Vec<SelectorTables>, String> {
    for reference in SELECTOR_REFERENCES {
        for (reader, literal, language, table) in [
            (
                reference.object_reader,
                reference.object_literal,
                reference.object_language_reader,
                0x3955c4,
            ),
            (
                reference.sequence_reader,
                reference.sequence_literal,
                reference.sequence_language_reader,
                0x3955b0,
            ),
        ] {
            let op = halfword(rom, reader)?;
            if op & 0xf800 != 0x4800
                || (op >> 8) & 7 != 2
                || ((reader + 4) & !3) + u32::from(op & 255) * 4 != literal
                || word(rom, literal)? != 0x08000000 + table
            {
                return Err("OBJ language table literal mismatch".into());
            }
            // ldrb r1,[r4]; lsl r1,#2; add r1,r1,r2; ldr r1,[r1].
            // Decode fields, rather than treating a pointer-shaped scan as proof.
            let op = halfword(rom, language)?;
            let shift = halfword(rom, language + 2)?;
            let add = halfword(rom, language + 4)?;
            let load = halfword(rom, language + 6)?;
            if op & 0xf800 != 0x7800
                || (op >> 6) & 31 != 0
                || (op >> 3) & 7 != 4
                || op & 7 != 1
                || shift & 0xf800 != 0
                || (shift >> 6) & 31 != 2
                || (shift >> 3) & 7 != 1
                || shift & 7 != 1
                || add & 0xfe00 != 0x1800
                || (add >> 6) & 7 != 2
                || (add >> 3) & 7 != 1
                || add & 7 != 1
                || load & 0xf800 != 0x6800
                || (load >> 6) & 31 != 0
                || (load >> 3) & 7 != 1
                || load & 7 != 1
            {
                return Err("OBJ language selector/load path mismatch".into());
            }
        }
        let hi = halfword(rom, reference.call)?;
        let lo = halfword(rom, reference.call + 2)?;
        let displacement =
            (((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
        if hi & 0xf800 != 0xf000
            || lo & 0xf800 != 0xf800
            || reference
                .call
                .wrapping_add(4)
                .wrapping_add_signed(displacement)
                != reference.constructor
        {
            return Err("OBJ language original constructor mismatch".into());
        }
    }
    // Five language IDs are independently established by EUR's original
    // language selector. Out-of-domain IDs retain the original ROM fallback.
    let decode = |root: u32| {
        (0..5u32)
            .map(|i| {
                word(rom, root + i * 4)?
                    .checked_sub(0x08000000)
                    .filter(|&v| v < MAX_SOURCE)
                    .ok_or("Invalid language-selected OBJ bank pointer".into())
            })
            .collect::<Result<Vec<_>, String>>()
    };
    Ok(vec![SelectorTables {
        selector_address: 0x02001f6e,
        object_table_offset: 0x3955c4,
        sequence_table_offset: 0x3955b0,
        object_roots: decode(0x3955c4)?,
        sequence_roots: decode(0x3955b0)?,
    }])
}
