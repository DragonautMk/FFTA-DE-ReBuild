//! Sparse EUR compact banks attested by direct literal-root constructor calls.
//! Unwitnessed selectors, gaps and live scheduling retain guest semantics.
use super::{compact_obj_animations::{Sequence, Frame, SourceSpan}, obj_lists, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

mod contracts;
pub use contracts::{BANK_SPECS, SOURCE_SPANS};

pub struct BankSpec {
    pub root: u32,
    pub sequence_relative_offset: u16,
    pub selectors: &'static [u16],
    pub frame_counts: &'static [usize],
    pub list_indices: &'static [u16],
    pub spans: &'static [(u32, u32)],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedList {
    pub index: u16,
    pub relative_offset: u16,
    pub list: obj_lists::List,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedSequence {
    pub relative_offset: u16,
    pub sequence: Sequence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub root: u32,
    pub flags: u16,
    pub sequence_relative_offset: u16,
    pub lists: Vec<IndexedList>,
    pub sequences: Vec<IndexedSequence>,
}

fn merge_pieces(mut pieces: Vec<(u32, Vec<u8>)>) -> Result<Vec<(u32, Vec<u8>)>, String> {
    pieces.sort_by_key(|piece| piece.0);
    let mut spans: Vec<(u32, Vec<u8>)> = Vec::new();
    for (offset, bytes) in pieces {
        let end = offset.checked_add(bytes.len() as u32).ok_or("Compact catalog source overflow")?;
        if offset & 1 != 0 || end > 0x1000000 { return Err("Invalid compact catalog source range".into()); }
        if let Some((start, previous)) = spans.last_mut() {
            let previous_end = *start + previous.len() as u32;
            if offset < previous_end { return Err("Overlapping compact catalog fields".into()); }
            if offset == previous_end { previous.extend(bytes); continue; }
        }
        spans.push((offset, bytes));
    }
    Ok(spans)
}

impl Bank {
    pub fn sequence_root(&self) -> u32 { self.root + u32::from(self.sequence_relative_offset) }
    pub fn list_root(&self) -> u32 { self.root + 4 }
    pub fn sequence(&self, selector: u16) -> Option<&Sequence> {
        self.sequences.iter().find(|entry| entry.sequence.selector == selector).map(|entry| &entry.sequence)
    }
    pub fn list(&self, index: u16) -> Option<&obj_lists::List> {
        self.lists.iter().find(|entry| entry.index == index).map(|entry| &entry.list)
    }

    fn validate_fields(&self, spec: &BankSpec) -> Result<(), String> {
        if self.root != spec.root || self.flags != 12 || self.sequence_relative_offset != spec.sequence_relative_offset
            || self.lists.len() != spec.list_indices.len() || self.sequences.len() != spec.selectors.len()
        { return Err("Invalid compact catalog bank projection".into()); }
        for (entry, &index) in self.lists.iter().zip(spec.list_indices) {
            entry.list.validate()?;
            if entry.index != index || self.list_root() + u32::from(entry.relative_offset) != entry.list.source_offset {
                return Err("Invalid compact catalog list binding".into());
            }
        }
        let mut referenced = std::collections::BTreeSet::new();
        for ((entry, &selector), &count) in self.sequences.iter().zip(spec.selectors).zip(spec.frame_counts) {
            let sequence = &entry.sequence;
            if sequence.selector != selector || sequence.frames.len() != count
                || self.sequence_root() + u32::from(entry.relative_offset) != sequence.source_offset
            { return Err("Invalid compact catalog sequence binding".into()); }
            for frame in &sequence.frames {
                if !spec.list_indices.contains(&u16::from(frame.list)) { return Err("Unbound compact catalog frame list".into()); }
                referenced.insert(u16::from(frame.list));
            }
        }
        if referenced.into_iter().collect::<Vec<_>>() != spec.list_indices {
            return Err("Invalid compact catalog referenced-list projection".into());
        }
        Ok(())
    }

    fn source_pieces(&self) -> Result<Vec<(u32, Vec<u8>)>, String> {
        let mut header = self.flags.to_le_bytes().to_vec();
        header.extend(self.sequence_relative_offset.to_le_bytes());
        let mut pieces = vec![(self.root, header)];
        for entry in &self.lists {
            pieces.push((self.list_root() + u32::from(entry.index) * 2, entry.relative_offset.to_le_bytes().to_vec()));
            pieces.push((entry.list.source_offset, entry.list.source_bytes()?));
        }
        for entry in &self.sequences {
            let sequence = &entry.sequence;
            pieces.push((self.sequence_root() + u32::from(sequence.selector) * 2, entry.relative_offset.to_le_bytes().to_vec()));
            let mut bytes = (sequence.frames.len() as u16).to_le_bytes().to_vec();
            for frame in &sequence.frames { bytes.extend([frame.list, frame.duration]); }
            pieces.push((sequence.source_offset, bytes));
        }
        Ok(pieces)
    }

    /// Read only represented fields. A miss does not impose a guest bound.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(self.root)?;
        if relative < 2 { return Some(self.flags.to_le_bytes()[relative as usize]); }
        if relative < 4 { return Some(self.sequence_relative_offset.to_le_bytes()[relative as usize - 2]); }
        for entry in &self.lists {
            let at = self.list_root() + u32::from(entry.index) * 2;
            if (at..at + 2).contains(&offset) { return Some(entry.relative_offset.to_le_bytes()[(offset - at) as usize]); }
            if let Some(byte) = offset.checked_sub(entry.list.source_offset).and_then(|at| entry.list.source_byte(at as usize)) { return Some(byte); }
        }
        for entry in &self.sequences {
            let sequence = &entry.sequence;
            let at = self.sequence_root() + u32::from(sequence.selector) * 2;
            if (at..at + 2).contains(&offset) { return Some(entry.relative_offset.to_le_bytes()[(offset - at) as usize]); }
            if let Some(relative) = offset.checked_sub(sequence.source_offset) {
                if relative < 2 { return Some((sequence.frames.len() as u16).to_le_bytes()[relative as usize]); }
                if let Some(frame) = sequence.frames.get((relative as usize - 2) / 2) {
                    return Some(if relative & 1 == 0 { frame.list } else { frame.duration });
                }
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactObjCatalog { pub banks: Vec<Bank> }

impl CompactObjCatalog {
    pub fn validate(&self) -> Result<(), String> {
        if self.banks.len() != BANK_SPECS.len() { return Err("Invalid compact catalog bank set".into()); }
        for (bank, spec) in self.banks.iter().zip(BANK_SPECS) {
            bank.validate_fields(spec)?;
            let spans = merge_pieces(bank.source_pieces()?)?;
            if spans.len() != spec.spans.len() || spans.iter().zip(spec.spans).any(|((at, bytes), &(start, end))| *at != start || bytes.len() != (end - start) as usize) {
                return Err("Invalid compact catalog source geometry".into());
            }
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let pieces = self.banks.iter().map(Bank::source_pieces).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect();
        let spans = merge_pieces(pieces)?;
        if spans.len() != SOURCE_SPANS.len() || spans.iter().zip(SOURCE_SPANS).any(|((at, bytes), &(start, end))| *at != start || bytes.len() != (end - start) as usize) {
            return Err("Invalid compact catalog source union".into());
        }
        Ok(spans.into_iter().map(|(at, bytes)| (at, bytes, "conditional-compact-obj-catalog".into())).collect())
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> { self.banks.iter().find_map(|bank| bank.byte_at(offset)) }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded { return Err("Compact catalog differs from authenticated EUR source".into()); }
        Ok(())
    }
}

pub struct Catalog { pub decoded: CompactObjCatalog, pub sources: Vec<SourceRange> }
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Compact catalog requires exact EUR ROM; USA pending".into());
        }
        let half = |at: u32| u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
        let mut banks = Vec::new();
        for spec in BANK_SPECS {
            let list_root = spec.root + 4;
            let sequence_root = spec.root + u32::from(half(spec.root + 2));
            let mut lists = Vec::new();
            for &index in spec.list_indices {
                let relative_offset = half(list_root + u32::from(index) * 2);
                let at = list_root + u32::from(relative_offset);
                let length = 2 + usize::from(half(at)) * 6;
                let bytes = rom.get(at as usize..at as usize + length).ok_or("Truncated compact catalog list")?;
                lists.push(IndexedList { index, relative_offset, list: obj_lists::List::decode(at, bytes)? });
            }
            let mut sequences = Vec::new();
            for &selector in spec.selectors {
                let relative_offset = half(sequence_root + u32::from(selector) * 2);
                let at = sequence_root + u32::from(relative_offset);
                let count = usize::from(half(at));
                let bytes = rom.get(at as usize + 2..at as usize + 2 + count * 2).ok_or("Truncated compact catalog sequence")?;
                sequences.push(IndexedSequence { relative_offset, sequence: Sequence {
                    selector, source_offset: at,
                    frames: bytes.chunks_exact(2).map(|bytes| Frame { list: bytes[0], duration: bytes[1] }).collect(),
                } });
            }
            banks.push(Bank { root: spec.root, flags: half(spec.root), sequence_relative_offset: half(spec.root + 2), lists, sequences });
        }
        let decoded = CompactObjCatalog { banks };
        let sources = decoded.source_spans()?.into_iter().map(|(offset, bytes, role)| SourceRange {
            offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role,
        }).collect();
        Ok(Self { decoded, sources })
    }
}
