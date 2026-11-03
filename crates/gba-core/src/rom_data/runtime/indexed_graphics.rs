//! Immutable source bytes rebuilt from typed packets; original guest work remains active.
use super::super::{SourceRange, indexed_graphics as model};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndexedGraphicsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub archive_header_bytes: u64,
    pub index_bytes: u64,
    pub token_bytes: u64,
    pub literal_bytes: u64,
    pub dictionary_header_bytes: u64,
    pub dictionary_bytes: u64,
    pub archives_used: u64,
    pub programs_used: u64,
    pub groups_used: u64,
}
#[derive(Clone, Copy)]
struct Byte {
    value: u8,
    field: u8,
    bank: u8,
    group: u8,
    program: u16,
}
struct Span {
    offset: usize,
    bytes: Vec<Byte>,
}
pub(super) struct IndexedGraphicsResources {
    spans: Vec<Span>,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 7],
    banks: AtomicU64,
    groups: AtomicU64,
    programs: [AtomicU64; 11],
}
impl IndexedGraphicsResources {
    pub(super) fn source_page(index: usize) -> bool {
        SOURCE_BOUNDS.iter().any(|&(start, length)| {
            index >> 12 >= start >> 12 && index >> 12 <= (start + length - 1) >> 12
        })
    }
    pub(super) fn new(
        data: model::IndexedGraphics,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Indexed graphics binding count mismatch".into());
        }
        let mut compiled = Vec::new();
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if *offset != source.offset
                || bytes.len() != source.length as usize
                || *role != source.role
                || ffta_rom::digest(bytes) != source.sha256
            {
                return Err("Indexed graphics typed source binding mismatch".into());
            }
            let mut cells: Vec<_> = bytes
                .iter()
                .map(|&value| Byte {
                    value,
                    field: 255,
                    bank: 255,
                    group: 255,
                    program: u16::MAX,
                })
                .collect();
            if let Some(group) = data.groups.iter().position(|g| g.source_offset == *offset) {
                for cell in &mut cells {
                    cell.field = 0;
                    cell.group = group as u8;
                }
            } else {
                let bank = data
                    .archives
                    .iter()
                    .position(|a| a.source_offset == *offset)
                    .ok_or("Missing indexed graphics archive")?;
                let archive = &data.archives[bank];
                for cell in &mut cells {
                    cell.bank = bank as u8;
                }
                let first_program: usize =
                    data.archives[..bank].iter().map(|a| a.programs.len()).sum();
                for cell in &mut cells[..8] {
                    cell.field = 1;
                }
                for (i, program) in archive.programs.iter().enumerate() {
                    let id = (first_program + i) as u16;
                    for cell in &mut cells[8 + i * 4..12 + i * 4] {
                        cell.field = 2;
                        cell.program = id;
                    }
                    let mut at = program.relative_offset as usize;
                    for packet in &program.packets {
                        let token_length = match packet {
                            model::Packet::Literal { .. } => 1,
                            model::Packet::Dictionary { .. } => 2,
                        };
                        for cell in &mut cells[at..at + token_length] {
                            cell.field = 3;
                            cell.program = id;
                        }
                        at += token_length;
                        if let model::Packet::Literal { bytes } = packet {
                            for cell in &mut cells[at..at + bytes.len()] {
                                cell.field = 4;
                                cell.program = id;
                            }
                            at += bytes.len();
                        }
                    }
                }
                let dict = archive.dictionary_relative_offset as usize;
                for cell in &mut cells[dict..dict + 4] {
                    cell.field = 5;
                }
                for cell in &mut cells[dict + 4..] {
                    cell.field = 6;
                }
            }
            if cells.iter().any(|b| b.field == 255) {
                return Err("Incomplete indexed graphics source projection".into());
            }
            compiled.push(Span {
                offset: *offset as usize,
                bytes: cells,
            });
        }
        Ok(Self {
            spans: compiled,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
            banks: AtomicU64::new(0),
            groups: AtomicU64::new(0),
            programs: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<Byte> {
        self.spans
            .iter()
            .find_map(|s| s.bytes.get(index.checked_sub(s.offset)?).copied())
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index) {
            return prior;
        }
        let values: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if values.iter().all(Option::is_none) {
            return prior;
        }
        let (mut fields, mut banks, mut groups, mut programs) = ([0u64; 7], 0u64, 0u64, [0u64; 11]);
        let result = std::array::from_fn(|i| match values[i] {
            Some(cell) => {
                fields[cell.field as usize] += 1;
                if cell.bank != 255 {
                    banks |= 1 << cell.bank;
                }
                if cell.group != 255 {
                    groups |= 1 << cell.group;
                }
                if cell.program != u16::MAX {
                    programs[usize::from(cell.program) / 64] |= 1 << (cell.program % 64);
                }
                cell.value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(fields.iter().sum(), Ordering::Relaxed);
        for (counter, count) in self.fields.iter().zip(fields) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.banks.fetch_or(banks, Ordering::Relaxed);
        self.groups.fetch_or(groups, Ordering::Relaxed);
        for (counter, mask) in self.programs.iter().zip(programs) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(result)
    }
    pub(super) fn stats(&self) -> IndexedGraphicsStats {
        let v: Vec<_> = self
            .fields
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect();
        IndexedGraphicsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            selector_bytes: v[0],
            archive_header_bytes: v[1],
            index_bytes: v[2],
            token_bytes: v[3],
            literal_bytes: v[4],
            dictionary_header_bytes: v[5],
            dictionary_bytes: v[6],
            archives_used: u64::from(self.banks.load(Ordering::Relaxed).count_ones()),
            groups_used: u64::from(self.groups.load(Ordering::Relaxed).count_ones()),
            programs_used: self
                .programs
                .iter()
                .map(|m| u64::from(m.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        self.spans
            .iter()
            .map(|s| s.bytes.len() * std::mem::size_of::<Byte>())
            .sum()
    }
}

// Source pages only; the exact byte membership is determined by the immutable spans.
const SOURCE_BOUNDS: &[(usize, usize)] = &[
    (0x430248, 2580),
    (0x4334c4, 3923),
    (0x434418, 5067),
    (0x4357e4, 3028),
    (0x441608, 2746),
    (0x444610, 4185),
    (0x44566c, 4929),
    (0x4469b0, 2964),
    (0x452c7c, 2899),
    (0x455f18, 4416),
    (0x457058, 5586),
    (0x45862c, 2683),
    (0x464818, 2579),
    (0x46760c, 4089),
    (0x468608, 4754),
    (0x46989c, 3128),
    (0x4757e0, 2610),
    (0x478a5c, 4333),
    (0x479b4c, 5452),
    (0x47b098, 2876),
    (0x71e57c, 20),
    (0x71e61c, 20),
    (0x71e630, 20),
    (0x71e658, 20),
];
