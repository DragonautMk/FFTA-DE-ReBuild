//! Sparse EUR compact banks attested by direct literal-root constructor calls.
//! Unwitnessed selectors, gaps and live scheduling retain guest semantics.
use super::{compact_obj_animations::{Sequence, Frame, SourceSpan}, obj_lists, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

mod contracts;
pub use contracts::{BANK_SPECS, SOURCE_SPANS};
pub mod selector_tables;

pub const RED_SPRING_RECORD_OFFSET: u32 = 0x3a4ca0;
pub const RED_SPRING_RECORD_LENGTH: usize = 24;
pub const RED_SPRING_RECORD_HASHES: [&str; 3] = [
    "136d1fe86b9516ea174e441624542ee6dba018ff18aa4615ffc91e88c8491ccb",
    "4680d3c3b164751c6c56f30a63878bc37ad0547257e964e7a078acd4b2d8c31b",
    "63d108fa4b4b0e2cf749c24b1ea1ad5a0f13a3a570c6873c8bec91612bed9c4b",
];

/// Preserve the observed fields without asserting semantics beyond the bank and selector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedSpringRecord {
    pub source_offset: u32,
    pub bank_pointer: u32,
    pub halfwords: [u16; 10],
}

impl RedSpringRecord {
    pub fn source_bytes(&self) -> [u8; RED_SPRING_RECORD_LENGTH] {
        let mut bytes = [0; RED_SPRING_RECORD_LENGTH];
        bytes[..4].copy_from_slice(&self.bank_pointer.to_le_bytes());
        for (index, value) in self.halfwords.iter().enumerate() {
            bytes[4 + index * 2..6 + index * 2].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

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

/// One observed compact sequence and its list offsets. List payloads remain
/// owned by the static OBJ resource; this projection owns metadata only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilEvidenceMetadata {
    pub flags: u16,
    pub sequence_relative_offset: u16,
    pub list_offsets: Vec<u16>,
    pub selector0_relative_offset: u16,
    pub frames: Vec<Frame>,
}

pub const SOIL_EVIDENCE_ROOT: u32 = 0x6c173c;
pub const SOIL_EVIDENCE_SEQUENCE_ROOT: u32 = 0x6c19b4;
pub const SOIL_EVIDENCE_METADATA_RANGES: [(u32, u32); 2] = [
    (SOIL_EVIDENCE_ROOT, 0x6c1768), (SOIL_EVIDENCE_SEQUENCE_ROOT, 0x6c19e0),
];

impl SoilEvidenceMetadata {
    pub fn validate(&self) -> Result<(), String> {
        if self.flags != 12 || self.sequence_relative_offset != 0x278
            || self.selector0_relative_offset != 2 || self.list_offsets.len() != 20 || self.frames.len() != 20
            || self.list_offsets.iter().zip(obj_lists::SOIL_EVIDENCE_LISTS).any(|(relative, (offset, _, _))|
                SOIL_EVIDENCE_ROOT + 4 + u32::from(*relative) != offset)
            || self.frames.iter().enumerate().any(|(index, frame)| usize::from(frame.list) != index || frame.duration == 0)
        { return Err("Invalid Soil Evidence compact metadata or list links".into()); }
        Ok(())
    }

    pub fn source_pieces(&self) -> Result<[(u32, Vec<u8>); 2], String> {
        self.validate()?;
        let mut header = self.flags.to_le_bytes().to_vec();
        header.extend(self.sequence_relative_offset.to_le_bytes());
        for value in &self.list_offsets { header.extend(value.to_le_bytes()); }
        let mut sequence = self.selector0_relative_offset.to_le_bytes().to_vec();
        sequence.extend((self.frames.len() as u16).to_le_bytes());
        for frame in &self.frames { sequence.extend([frame.list, frame.duration]); }
        Ok([(SOIL_EVIDENCE_ROOT, header), (SOIL_EVIDENCE_SEQUENCE_ROOT, sequence)])
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if let Some(relative) = offset.checked_sub(SOIL_EVIDENCE_ROOT) {
            let at = relative as usize;
            if at < 2 { return Some(self.flags.to_le_bytes()[at]); }
            if at < 4 { return Some(self.sequence_relative_offset.to_le_bytes()[at - 2]); }
            if at < 44 { return self.list_offsets.get((at - 4) / 2).map(|value| value.to_le_bytes()[(at - 4) % 2]); }
        }
        let at = offset.checked_sub(SOIL_EVIDENCE_SEQUENCE_ROOT)? as usize;
        if at < 2 { return Some(self.selector0_relative_offset.to_le_bytes()[at]); }
        if at < 4 { return Some((self.frames.len() as u16).to_le_bytes()[at - 2]); }
        self.frames.get((at - 4) / 2).map(|frame| if at & 1 == 0 { frame.list } else { frame.duration })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactObjCatalog {
    pub banks: Vec<Bank>,
    // Historical v1 evidence can still be decoded for bank identity comparison.
    // Current validation requires all fourteen source-authenticated tables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector_tables: Option<selector_tables::SelectorTables>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub red_spring_records: Option<[RedSpringRecord; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soil_evidence_metadata: Option<SoilEvidenceMetadata>,
}

impl CompactObjCatalog {
    pub fn validate(&self) -> Result<(), String> {
        if self.banks.len() != BANK_SPECS.len() { return Err("Invalid compact catalog bank set".into()); }
        let tables = self.selector_tables.as_ref().ok_or("Missing compact selector tables")?;
        tables.validate()?;
        for (root, values) in [
            (0x6c58b8, tables.motion.rows.iter().map(|row| u16::from(row.selector)).collect::<std::collections::BTreeSet<_>>()),
            (0x6c6e6c, tables.positions.rows.iter().map(|row| u16::from(row.selector)).collect()),
            (0x6cbaf8, tables.horizontal.rows.iter().map(|row| u16::from(row.selector)).collect()),
            (0x6c992c, tables.timed_positions.as_ref().unwrap().rows.iter().map(|row| u16::from(row.selector)).collect()),
            (0x6c95d4, tables.motion_positions.as_ref().unwrap().rows.iter().map(|row| u16::from(row.selector)).collect()),
            (0x6c0614, tables.timer_positions.as_ref().unwrap().rows.iter().map(|row| u16::from(row.selector)).collect()),
            (0x6bac28, tables.schedule.as_ref().unwrap().rows.iter().map(|row| u16::from(row.selector)).collect()),
            (0x6cb278, tables.script_lookup.as_ref().unwrap().rows.iter().map(|&value| u16::from(value)).collect()),
            (0x6ccae4, tables.scene.as_ref().unwrap().rows.iter().map(|row| u16::from(row.first_selector)).collect()),
            (0x6ccdec, tables.scene.as_ref().unwrap().rows.iter().map(|row| u16::from(row.second_selector) + 1).collect()),
        ] {
            let spec = BANK_SPECS.iter().find(|spec| spec.root == root).ok_or("Unbound compact selector table")?;
            if values.into_iter().collect::<Vec<_>>().as_slice() != spec.selectors {
                return Err("Compact selector table differs from bank projection".into());
            }
        }
        let effect_selectors: std::collections::BTreeSet<_> = tables.effects.as_ref().unwrap().rows.iter().filter_map(|row| row.selector.map(u16::from)).collect();
        let effect_spec = BANK_SPECS.iter().find(|spec| spec.root == 0x6bb574).ok_or("Unbound effect selector table")?;
        if effect_selectors.into_iter().collect::<Vec<_>>().as_slice() != effect_spec.selectors { return Err("Effect selectors differ from bank projection".into()); }
        let grid_selectors: std::collections::BTreeSet<_> = tables.grid_first.as_ref().unwrap().rows.iter().chain(&tables.grid_second.as_ref().unwrap().rows).map(|row| row.selector()).collect();
        let grid_spec = BANK_SPECS.iter().find(|spec| spec.root == 0x6cac40).ok_or("Unbound grid selector tables")?;
        if grid_selectors.into_iter().collect::<Vec<_>>().as_slice() != grid_spec.selectors { return Err("Grid selectors differ from bank projection".into()); }
        let mut selected: std::collections::BTreeSet<_> = tables.flight.as_ref().unwrap().rows.iter().map(|row| u16::from(row.selector)).collect();
        selected.extend(tables.burst.as_ref().unwrap().rows.iter().map(|row| u16::from(row.selector)));
        let spec = BANK_SPECS.iter().find(|spec| spec.root == 0x6babc4).ok_or("Unbound compact selector tables")?;
        if selected.into_iter().collect::<Vec<_>>().as_slice() != spec.selectors { return Err("Compact selector tables differ from bank projection".into()); }
        for (bank, spec) in self.banks.iter().zip(BANK_SPECS) {
            bank.validate_fields(spec)?;
            let spans = merge_pieces(bank.source_pieces()?)?;
            if spans.len() != spec.spans.len() || spans.iter().zip(spec.spans).any(|((at, bytes), &(start, end))| *at != start || bytes.len() != (end - start) as usize) {
                return Err("Invalid compact catalog source geometry".into());
            }
        }
        let bank = self.banks.iter().find(|bank| bank.root == 0x6c1e78).ok_or("Missing Red Spring compact bank")?;
        for (index, record) in self.red_spring_records.as_ref().ok_or("Missing Red Spring compact records")?.iter().enumerate() {
            if record.source_offset != RED_SPRING_RECORD_OFFSET + (index * RED_SPRING_RECORD_LENGTH) as u32
                || record.bank_pointer != 0x0800_0000 + bank.root || bank.sequence(record.halfwords[0]).is_none() {
                return Err("Invalid Red Spring compact record binding".into());
            }
        }
        self.soil_evidence_metadata.as_ref().ok_or("Missing Soil Evidence compact metadata")?.validate()?;
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut pieces: Vec<_> = self.banks.iter().map(Bank::source_pieces).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect();
        pieces.extend(self.selector_tables.as_ref().unwrap().source_pieces());
        pieces.extend(self.red_spring_records.as_ref().unwrap().iter().map(|record| (record.source_offset, record.source_bytes().to_vec())));
        pieces.extend(self.soil_evidence_metadata.as_ref().unwrap().source_pieces()?);
        let spans = merge_pieces(pieces)?;
        if spans.len() != SOURCE_SPANS.len() || spans.iter().zip(SOURCE_SPANS).any(|((at, bytes), &(start, end))| *at != start || bytes.len() != (end - start) as usize) {
            return Err("Invalid compact catalog source union".into());
        }
        Ok(spans.into_iter().map(|(at, bytes)| (at, bytes, "conditional-compact-obj-catalog".into())).collect())
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.banks.iter().find_map(|bank| bank.byte_at(offset))
            .or_else(|| self.selector_tables.as_ref().and_then(|tables| tables.byte_at(offset)))
            .or_else(|| self.red_spring_records.as_ref()?.iter().find_map(|record| {
                offset.checked_sub(record.source_offset).and_then(|at| record.source_bytes().get(at as usize).copied())
            }))
            .or_else(|| self.soil_evidence_metadata.as_ref()?.byte_at(offset))
    }
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
        for (start, end, expected) in [
            (0xd8968, 0xd897c, "e1db99bf0c51491f7314175ce66b14d5ecdb3c820edfb4a4831c040a3aebdae4"),
            (0xd8b60, 0xd8b80, "efadd2079800d4a984a4bdd5bd8f8a927547c5f2be71012f4f5e9011865e4b93"),
            (0xd8baa, 0xd8be2, "60e9949e9ca96ea25f54bbc3b237ae939f10f5f7616448a83d6907f74470e4ad"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected { return Err("Red Spring compact record reader mismatch".into()); }
        }
        for (index, expected) in RED_SPRING_RECORD_HASHES.iter().enumerate() {
            let start = RED_SPRING_RECORD_OFFSET as usize + index * RED_SPRING_RECORD_LENGTH;
            if ffta_rom::digest(&rom[start..start + RED_SPRING_RECORD_LENGTH]) != *expected {
                return Err("Red Spring compact record source mismatch".into());
            }
        }
        let red_spring_records = std::array::from_fn(|index| {
            let offset = RED_SPRING_RECORD_OFFSET + (index * RED_SPRING_RECORD_LENGTH) as u32;
            let start = offset as usize;
            RedSpringRecord { source_offset: offset,
                bank_pointer: u32::from_le_bytes(rom[start..start + 4].try_into().unwrap()),
                halfwords: std::array::from_fn(|index| half(offset + 4 + index as u32 * 2)), }
        });
        for (start, end, expected) in [
            (0x6c173c, 0x6c1768, "5c110e015c1a947f9a9cfd203c290decfa8e0dcc3980c26d072fea8513d73b9d"),
            (0x6c19b4, 0x6c19e0, "033d9442a96b97430edc149c1513c68e4d01e06ea4a2ae637aa4bc9645a15c76"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected { return Err("Soil Evidence compact metadata source mismatch".into()); }
        }
        let soil_evidence_metadata = SoilEvidenceMetadata {
            flags: half(SOIL_EVIDENCE_ROOT),
            sequence_relative_offset: half(SOIL_EVIDENCE_ROOT + 2),
            list_offsets: (0..20).map(|index| half(SOIL_EVIDENCE_ROOT + 4 + index * 2)).collect(),
            selector0_relative_offset: half(SOIL_EVIDENCE_SEQUENCE_ROOT),
            frames: (0..20).map(|index| {
                let at = SOIL_EVIDENCE_SEQUENCE_ROOT as usize + 4 + index * 2;
                Frame { list: rom[at], duration: rom[at + 1] }
            }).collect(),
        };
        if half(SOIL_EVIDENCE_SEQUENCE_ROOT + 2) != 20 { return Err("Soil Evidence compact frame count mismatch".into()); }
        let decoded = CompactObjCatalog { banks, selector_tables: Some(selector_tables::SelectorTables::from_rom(rom)), red_spring_records: Some(red_spring_records), soil_evidence_metadata: Some(soil_evidence_metadata) };
        let sources = decoded.source_spans()?.into_iter().map(|(offset, bytes, role)| SourceRange {
            offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role,
        }).collect();
        Ok(Self { decoded, sources })
    }
}
