//! Independently rooted EUR UI animation banks. Live task indices/ticks stay RAM.
use super::{obj_lists::List, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

mod roots;
pub use roots::{BANK_ROOTS, REFERENCES};
mod selectors;
pub use selectors::{SelectorReference, SelectorTables, SELECTOR_REFERENCES};
const MAX_ENTRIES: usize = 4096;
const MAX_SOURCE: u32 = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct RootReference {
    pub call: u32,
    pub constructor: u32,
    pub object_reader: u32,
    pub object_literal: u32,
    pub sequence_reader: u32,
    pub sequence_literal: u32,
    pub object_root: u32,
    pub sequence_root: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub object_list: u16,
    /// Original u16 comparison against the task's wrapping u8 tick counter.
    pub duration: u16,
}

/// Original nonrepeat zero-count tasks still read their first list selector.
/// The duration aliases an existing typed header/index word, not another frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZeroCountHold {
    pub object_list: u16,
    pub duration_source_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub source_offset: u32,
    /// The original header count. Empty records have no invented frame.
    pub frames: Vec<Frame>,
    /// Present only when original nonrepeat caller/read semantics are proved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zero_count_hold: Option<ZeroCountHold>,
}
impl Sequence {
    pub fn source_len(&self) -> usize {
        2 + self.frames.len() * 4 + usize::from(self.zero_count_hold.is_some()) * 2
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        if self.frames.len() > 255
            || self.zero_count_hold.as_ref().is_some_and(|hold| {
                !self.frames.is_empty()
                    || self.source_offset.checked_add(4) != Some(hold.duration_source_offset)
            })
            || self.source_offset & 1 != 0
            || self
                .source_offset
                .checked_add(self.source_len() as u32)
                .is_none_or(|end| end > MAX_SOURCE)
        {
            return Err("Invalid OBJ animation sequence count/extent".into());
        }
        let mut bytes = (self.frames.len() as u16).to_le_bytes().to_vec();
        for frame in &self.frames {
            bytes.extend(frame.object_list.to_le_bytes());
            bytes.extend(frame.duration.to_le_bytes());
        }
        if let Some(hold) = &self.zero_count_hold {
            bytes.extend(hold.object_list.to_le_bytes());
        }
        Ok(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub object_root: u32,
    pub object_offsets: Vec<u32>,
    pub lists: Vec<List>,
    pub sequence_root: u32,
    pub sequence_offsets: Vec<u32>,
    pub sequences: Vec<Sequence>,
}

fn index_end(root: u32, offsets: &[u32]) -> Result<u32, String> {
    if root & 3 != 0
        || offsets.is_empty()
        || offsets.len() > MAX_ENTRIES
        || offsets[0] != offsets.len() as u32 * 4
        || offsets.windows(2).any(|v| v[0] >= v[1])
        || offsets.iter().any(|&v| v & 1 != 0)
    {
        return Err("Invalid OBJ animation relative index".into());
    }
    root.checked_add(offsets[0])
        .filter(|&v| v <= MAX_SOURCE)
        .ok_or("OBJ animation index overflow".into())
}

fn source<'a>(rom: &'a [u8], at: u32, len: usize) -> Result<&'a [u8], String> {
    let end = at
        .checked_add(u32::try_from(len).map_err(|_| "OBJ source length overflow")?)
        .filter(|&end| end <= MAX_SOURCE)
        .ok_or("OBJ source offset overflow")?;
    rom.get(at as usize..end as usize)
        .ok_or("Truncated OBJ animation source".into())
}
fn halfword(rom: &[u8], at: u32) -> Result<u16, String> {
    Ok(u16::from_le_bytes(source(rom, at, 2)?.try_into().unwrap()))
}
fn word(rom: &[u8], at: u32) -> Result<u32, String> {
    Ok(u32::from_le_bytes(source(rom, at, 4)?.try_into().unwrap()))
}
fn decode_index(rom: &[u8], root: u32) -> Result<Vec<u32>, String> {
    let first = word(rom, root)? as usize;
    if first == 0 || first % 4 != 0 || first / 4 > MAX_ENTRIES {
        return Err("Invalid OBJ animation index boundary".into());
    }
    let offsets: Vec<_> = source(rom, root, first)?
        .chunks_exact(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect();
    index_end(root, &offsets)?;
    Ok(offsets)
}

impl Bank {
    pub fn decode(rom: &[u8], object_root: u32, sequence_root: u32) -> Result<Self, String> {
        let object_offsets = decode_index(rom, object_root)?;
        let sequence_offsets = decode_index(rom, sequence_root)?;
        let mut lists = Vec::new();
        for &offset in &object_offsets {
            let at = object_root
                .checked_add(offset)
                .ok_or("OBJ list pointer overflow")?;
            let count = halfword(rom, at)? as usize;
            lists.push(List::decode(at, source(rom, at, 2 + count * 6)?)?);
        }
        let mut sequences = Vec::new();
        for &offset in &sequence_offsets {
            let at = sequence_root
                .checked_add(offset)
                .ok_or("OBJ sequence pointer overflow")?;
            let count = halfword(rom, at)? as usize;
            if count > 255 {
                return Err("OBJ sequence exceeds original task frame-count width".into());
            }
            let body = source(rom, at, 2 + count * 4)?;
            sequences.push(Sequence {
                source_offset: at,
                zero_count_hold: None,
                frames: body[2..]
                    .chunks_exact(4)
                    .map(|v| Frame {
                        object_list: u16::from_le_bytes([v[0], v[1]]),
                        duration: u16::from_le_bytes([v[2], v[3]]),
                    })
                    .collect(),
            });
        }
        let bank = Self {
            object_root,
            object_offsets,
            lists,
            sequence_root,
            sequence_offsets,
            sequences,
        };
        bank.validate()?;
        Ok(bank)
    }

    /// Explicit caller-proved variant; zero counts in other banks stay opaque.
    pub fn decode_zero_count_holds(
        rom: &[u8],
        object_root: u32,
        sequence_root: u32,
    ) -> Result<Self, String> {
        let mut bank = Self::decode(rom, object_root, sequence_root)?;
        for sequence in &mut bank.sequences {
            if !sequence.frames.is_empty() {
                return Err("Zero-count hold bank has a nonzero header".into());
            }
            sequence.zero_count_hold = Some(ZeroCountHold {
                object_list: halfword(
                    rom,
                    sequence
                        .source_offset
                        .checked_add(2)
                        .ok_or("Zero-count selector overflow")?,
                )?,
                duration_source_offset: sequence
                    .source_offset
                    .checked_add(4)
                    .ok_or("Zero-count duration alias overflow")?,
            });
        }
        bank.validate()?;
        Ok(bank)
    }

    pub fn validate(&self) -> Result<(), String> {
        index_end(self.object_root, &self.object_offsets)?;
        index_end(self.sequence_root, &self.sequence_offsets)?;
        if self.lists.len() != self.object_offsets.len()
            || self.sequences.len() != self.sequence_offsets.len()
        {
            return Err("OBJ animation index/record cardinality mismatch".into());
        }
        for (&offset, list) in self.object_offsets.iter().zip(&self.lists) {
            list.validate()?;
            if self.object_root.checked_add(offset) != Some(list.source_offset)
                || list
                    .source_offset
                    .checked_add(list.source_len() as u32)
                    .is_none_or(|end| end > MAX_SOURCE)
            {
                return Err("OBJ animation list source mismatch".into());
            }
        }
        for (&offset, sequence) in self.sequence_offsets.iter().zip(&self.sequences) {
            sequence.source_bytes()?;
            if self.sequence_root.checked_add(offset) != Some(sequence.source_offset)
                || sequence
                    .frames
                    .iter()
                    .any(|f| f.object_list as usize >= self.lists.len())
                || sequence
                    .zero_count_hold
                    .as_ref()
                    .is_some_and(|h| h.object_list as usize >= self.lists.len())
            {
                return Err("OBJ animation sequence pointer/list link mismatch".into());
            }
        }
        let mut spans = self.spans();
        check_spans(&mut spans)
    }

    pub fn spans(&self) -> Vec<(u32, usize, &'static str)> {
        let mut spans = vec![
            (
                self.object_root,
                self.object_offsets.len() * 4,
                "obj-animation-list-index",
            ),
            (
                self.sequence_root,
                self.sequence_offsets.len() * 4,
                "obj-animation-sequence-index",
            ),
        ];
        spans.extend(
            self.lists
                .iter()
                .map(|l| (l.source_offset, l.source_len(), "obj-animation-list")),
        );
        spans.extend(
            self.sequences
                .iter()
                .map(|s| (s.source_offset, s.source_len(), "obj-animation-sequence")),
        );
        spans
    }
}

fn check_spans(spans: &mut [(u32, usize, &str)]) -> Result<(), String> {
    spans.sort_by_key(|s| s.0);
    let mut end = 0;
    for &(at, len, _) in spans.iter() {
        if at < end {
            return Err("Overlapping OBJ animation sources".into());
        }
        end = at
            .checked_add(u32::try_from(len).map_err(|_| "OBJ animation source length overflow")?)
            .filter(|&end| end <= MAX_SOURCE)
            .ok_or("OBJ animation source overflow")?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Banks {
    pub banks: Vec<Bank>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selector_tables: Vec<SelectorTables>,
}
impl Banks {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .banks
            .windows(2)
            .any(|v| v[0].object_root >= v[1].object_root)
        {
            return Err("Unordered/duplicate OBJ animation banks".into());
        }
        for bank in &self.banks {
            bank.validate()?;
            for sequence in &bank.sequences {
                if let Some(hold) = &sequence.zero_count_hold {
                    if self.zero_count_duration(hold).is_none() {
                        return Err(
                            "Zero-count duration alias has no typed header/index owner".into()
                        );
                    }
                }
            }
        }
        let mut spans: Vec<_> = self.banks.iter().flat_map(Bank::spans).collect();
        for tables in &self.selector_tables {
            tables.validate(&self.banks)?;
            spans.extend(tables.spans());
        }
        check_spans(&mut spans)
    }
    /// Resolve only the exact two established alias-owner forms. This does not
    /// grant a zero-count task permission to advance or use repeat mode one.
    pub fn zero_count_duration(&self, hold: &ZeroCountHold) -> Option<u16> {
        for bank in &self.banks {
            if hold.duration_source_offset == bank.sequence_root {
                return Some(*bank.sequence_offsets.first()? as u16);
            }
            if let Ok(index) = bank
                .sequences
                .binary_search_by_key(&hold.duration_source_offset, |s| s.source_offset)
            {
                return Some(bank.sequences[index].frames.len() as u16);
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("OBJ animation banks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Banks,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("OBJ animation banks require exact EUR ROM; USA unverified".into());
        }
        for reference in REFERENCES {
            for (reader, literal, root) in [
                (
                    reference.object_reader,
                    reference.object_literal,
                    reference.object_root,
                ),
                (
                    reference.sequence_reader,
                    reference.sequence_literal,
                    reference.sequence_root,
                ),
            ] {
                let opcode = halfword(rom, reader)?;
                if opcode & 0xf800 != 0x4800
                    || ((reader + 4) & !3) + u32::from(opcode & 255) * 4 != literal
                    || word(rom, literal)? != root + 0x08000000
                {
                    return Err("OBJ animation source literal mismatch".into());
                }
            }
            let hi = halfword(rom, reference.call)?;
            let lo = halfword(rom, reference.call + 2)?;
            let displacement = ((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32;
            let displacement = (displacement << 9) >> 9;
            if hi & 0xf800 != 0xf000
                || lo & 0xf800 != 0xf800
                || reference
                    .call
                    .wrapping_add(4)
                    .wrapping_add_signed(displacement)
                    != reference.constructor
            {
                return Err("OBJ animation original constructor call mismatch".into());
            }
        }
        let banks = BANK_ROOTS
            .iter()
            .map(|&(objects, sequences)| {
                if (objects, sequences) == (0xb4d8e8, 0xb4f4e8) {
                    Bank::decode_zero_count_holds(rom, objects, sequences)
                } else {
                    Bank::decode(rom, objects, sequences)
                }
            })
            .collect::<Result<_, _>>()?;
        let decoded = Banks {
            banks,
            selector_tables: selectors::decode_europe(rom)?,
        };
        decoded.validate()?;
        let mut sources = Vec::new();
        for bank in &decoded.banks {
            for (offset, length, role) in bank.spans() {
                sources.push(SourceRange {
                    offset,
                    length: length as u32,
                    sha256: ffta_rom::digest(source(rom, offset, length)?),
                    role: role.into(),
                });
            }
        }
        for tables in &decoded.selector_tables {
            for (offset, length, role) in tables.spans() {
                sources.push(SourceRange {
                    offset,
                    length: length as u32,
                    sha256: ffta_rom::digest(source(rom, offset, length)?),
                    role: role.into(),
                });
            }
        }
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let mut rom = vec![0; 0x220];
        for (at, word) in [(0x100, 8u32), (0x104, 16), (0x200, 8), (0x204, 18)] {
            rom[at..at + 4].copy_from_slice(&word.to_le_bytes());
        }
        for at in [0x108, 0x110] {
            rom[at..at + 2].copy_from_slice(&1u16.to_le_bytes());
        }
        rom[0x208..0x20a].copy_from_slice(&2u16.to_le_bytes());
        for (at, word) in [(0x20a, 0u16), (0x20c, 0), (0x20e, 1), (0x210, 300)] {
            rom[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        rom
    }
    #[test]
    fn indexed_animation_banks_reject_bad_links_extents_and_overlaps() {
        let rom = fixture();
        let bank = Bank::decode(&rom, 0x100, 0x200).unwrap();
        assert_eq!(bank.sequences[0].frames[0].duration, 0);
        assert_eq!(bank.sequences[0].frames[1].duration, 300);
        assert!(bank.sequences[1].frames.is_empty());
        assert_eq!(bank.sequences[0].source_bytes().unwrap(), rom[0x208..0x212]);
        let mut edited = bank.clone();
        edited.sequences[0].frames[0].object_list = 2;
        assert!(edited.validate().is_err());
        edited = bank.clone();
        edited.object_offsets[1] = 8;
        assert!(edited.validate().is_err());
        edited = bank.clone();
        edited.sequence_offsets[1] = 12;
        edited.sequences[1].source_offset = 0x20c;
        assert!(edited.validate().is_err());
        edited = bank.clone();
        edited.sequences[1].frames = vec![
            Frame {
                object_list: 0,
                duration: 1
            };
            256
        ];
        assert!(edited.validate().is_err());
        assert!(Bank::decode(&rom[..0x212], 0x100, 0x200).is_err());
        assert!(Bank::decode(&rom, u32::MAX - 3, 0x200).is_err());
        assert!(Bank::decode(&rom, 0x101, 0x200).is_err());
        assert!(Banks {
            banks: vec![bank.clone(), bank],
            selector_tables: Vec::new()
        }
        .validate()
        .is_err());
        assert!(Catalog::from_rom(&rom).is_err());
    }
}
