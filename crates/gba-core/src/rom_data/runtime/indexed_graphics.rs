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
    pub context_bytes: u64,
    pub archives_used: u64,
    pub programs_used: u64,
    pub groups_used: u64,
    pub color_source_bytes: u64,
    /// Physical color archive and program masks, in source order.
    pub color_archives_mask: u64,
    pub color_programs_mask: [u64; 8],
    pub white_flowers_caller_literal_bytes: u64,
    pub white_flowers_archive_bytes: [u64; 5],
    pub white_flowers_direct_literal_bytes: u64,
    pub white_flowers_direct_archive_bytes: u64,
    pub additional_literal_bytes: u64,
    pub additional_prefix_bytes: u64,
    pub additional_archive_bytes: [u64; 15],
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
    fields: [AtomicU64; 12],
    banks: AtomicU64,
    groups: AtomicU64,
    programs: [AtomicU64; 47],
    color_bytes: AtomicU64,
    color_programs: [AtomicU64; 8],
    white_flowers_bytes: [AtomicU64; 5],
    white_flowers_direct_bytes: AtomicU64,
    additional_archive_bytes: [AtomicU64; 15],
}
impl IndexedGraphicsResources {
    pub(super) fn source_page(index: usize) -> bool {
        SOURCE_BOUNDS.iter().any(|&(start, length)| {
            index >> 12 >= start >> 12 && index >> 12 <= (start + length - 1) >> 12
        }) || model::additional::source_page(index)
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
            if *offset == model::CONTEXT_SOURCE {
                for cell in &mut cells {
                    cell.field = 7;
                }
            } else if *offset == model::WHITE_FLOWERS_LITERALS {
                for cell in &mut cells { cell.field = 8; }
            } else if *offset == model::WHITE_FLOWERS_DIRECT_LITERAL {
                for cell in &mut cells { cell.field = 9; }
            } else if *offset == model::WHITE_FLOWERS_GROUP {
                for cell in &mut cells { cell.field = 0; cell.group = model::GROUPS.len() as u8; }
            } else if let Some(group) = data.groups.iter().position(|g| g.source_offset == *offset) {
                for cell in &mut cells {
                    cell.field = 0;
                    cell.group = group as u8;
                }
            } else if data.additional.as_ref().is_some_and(|_| model::additional::CALLER_LITERALS.iter().any(|(at, _)| at == offset)) {
                for cell in &mut cells { cell.field = 10; }
            } else if let Some(group) = data.additional.as_ref().and_then(|a| a.groups.iter().position(|g| g.source_offset == *offset)) {
                for cell in &mut cells { cell.field = 0; cell.group = (model::GROUPS.len() + 1 + group) as u8; }
            } else if data.additional.as_ref().is_some_and(|a| a.embedded_prefixes.iter().any(|p| p.source_offset == *offset)) {
                for cell in &mut cells { cell.field = 11; }
            } else {
                let (bank, archive, first_program) = if let Some(bank) = data.archives.iter().position(|a| a.source_offset == *offset) {
                    (bank, &data.archives[bank], data.archives[..bank].iter().map(|a| a.programs.len()).sum::<usize>())
                } else if *offset == model::WHITE_FLOWERS_DIRECT_ROOT {
                    let direct = data.white_flowers_direct.as_ref().ok_or("Missing White Flowers direct indexed graphics")?;
                    (model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len() + model::WHITE_FLOWERS_ROOTS.len(), &direct.archive,
                        model::BASE_PROGRAM_COUNT + model::COLOR_ARCHIVE_ROOTS.len() * 165 + model::WHITE_FLOWERS_ROOTS.len() * 33)
                } else if let Some(slot) = model::WHITE_FLOWERS_ROOTS.iter().position(|&at| at == *offset) {
                    let white = data.white_flowers.as_ref().ok_or("Missing White Flowers indexed graphics")?;
                    (model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len() + slot, &white.archives[slot],
                        model::BASE_PROGRAM_COUNT + model::COLOR_ARCHIVE_ROOTS.len() * 165 + slot * 33)
                } else if let Some((additional, slot)) = data.additional.as_ref().and_then(|a|
                    a.archives.iter().position(|archive| archive.source_offset == *offset).map(|i| (a, i))) {
                    let first = model::BASE_PROGRAM_COUNT + model::COLOR_ARCHIVE_ROOTS.len() * 165
                        + model::WHITE_FLOWERS_ROOTS.len() * 33 + 62
                        + additional.archives[..slot].iter().map(|a| a.programs.len()).sum::<usize>();
                    (model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len() + model::WHITE_FLOWERS_ROOTS.len() + 1 + slot,
                        &additional.archives[slot], first)
                } else {
                    let archives = data.color_archives.as_ref().ok_or("Missing color indexed graphics")?;
                    let color = model::COLOR_ARCHIVE_ROOTS.iter().enumerate().find_map(|(i, &root)| {
                        let end = root.checked_add(model::COLOR_ARCHIVE_LENGTHS[i] as u32)?;
                        (*offset >= root && offset.checked_add(bytes.len() as u32)? <= end).then_some(i)
                    }).ok_or("Missing indexed graphics archive")?;
                    let first = data.archives.iter().map(|a| a.programs.len()).sum::<usize>()
                        + archives[..color].iter().map(|a| a.programs.len()).sum::<usize>();
                    (data.archives.len() + color, &archives[color], first)
                };
                let complete = archive.source_bytes()?;
                let mut archive_cells: Vec<_> = complete.iter().map(|&value| Byte {
                    value, field: 255, bank: bank as u8, group: 255, program: u16::MAX,
                }).collect();
                for cell in &mut archive_cells[..8] {
                    cell.field = 1;
                }
                for (i, program) in archive.programs.iter().enumerate() {
                    let id = (first_program + i) as u16;
                    for cell in &mut archive_cells[8 + i * 4..12 + i * 4] {
                        cell.field = 2;
                        cell.program = id;
                    }
                    let mut at = program.relative_offset as usize;
                    for packet in &program.packets {
                        let token_length = match packet {
                            model::Packet::Literal { .. } => 1,
                            model::Packet::Dictionary { .. } => 2,
                        };
                        for cell in &mut archive_cells[at..at + token_length] {
                            cell.field = 3;
                            cell.program = id;
                        }
                        at += token_length;
                        if let model::Packet::Literal { bytes } = packet {
                            for cell in &mut archive_cells[at..at + bytes.len()] {
                                cell.field = 4;
                                cell.program = id;
                            }
                            at += bytes.len();
                        }
                    }
                }
                let dict = archive.dictionary_relative_offset as usize;
                for cell in &mut archive_cells[dict..dict + 4] {
                    cell.field = 5;
                }
                for cell in &mut archive_cells[dict + 4..] {
                    cell.field = 6;
                }
                let start = (*offset - archive.source_offset) as usize;
                cells = archive_cells.get(start..start + bytes.len())
                    .ok_or("Indexed graphics split outside archive")?.to_vec();
                if cells.iter().map(|cell| cell.value).ne(bytes.iter().copied()) {
                    return Err("Indexed graphics split source mismatch".into());
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
            color_bytes: AtomicU64::new(0),
            color_programs: std::array::from_fn(|_| AtomicU64::new(0)),
            white_flowers_bytes: std::array::from_fn(|_| AtomicU64::new(0)),
            white_flowers_direct_bytes: AtomicU64::new(0),
            additional_archive_bytes: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let (mut fields, mut banks, mut groups, mut programs) = ([0u64; 12], 0u64, 0u64, [0u64; 47]);
        let (mut color_bytes, mut color_programs) = (0u64, [0u64; 8]);
        let mut white_flowers_bytes = [0u64; 5];
        let mut white_flowers_direct_bytes = 0u64;
        let mut additional_archive_bytes = [0u64; 15];
        let result = std::array::from_fn(|i| match values[i] {
            Some(cell) => {
                fields[cell.field as usize] += 1;
                if cell.bank != 255 {
                    banks |= 1 << cell.bank;
                    if (model::ARCHIVES.len()..model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len()).contains(&usize::from(cell.bank)) {
                        color_bytes += 1;
                        if cell.program != u16::MAX {
                            let color_program = usize::from(cell.program) - model::BASE_PROGRAM_COUNT;
                            color_programs[color_program / 64] |= 1 << (color_program % 64);
                        }
                    } else if (model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len()..model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len() + model::WHITE_FLOWERS_ROOTS.len()).contains(&usize::from(cell.bank)) {
                        white_flowers_bytes[usize::from(cell.bank) - model::ARCHIVES.len() - model::COLOR_ARCHIVE_ROOTS.len()] += 1;
                    } else if usize::from(cell.bank) == model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len() + model::WHITE_FLOWERS_ROOTS.len() {
                        white_flowers_direct_bytes += 1;
                    } else {
                        let first = model::ARCHIVES.len() + model::COLOR_ARCHIVE_ROOTS.len() + model::WHITE_FLOWERS_ROOTS.len() + 1;
                        if let Some(slot) = usize::from(cell.bank).checked_sub(first).filter(|&slot| slot < additional_archive_bytes.len()) {
                            additional_archive_bytes[slot] += 1;
                        }
                    }
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
        self.color_bytes.fetch_add(color_bytes, Ordering::Relaxed);
        for (counter, mask) in self.color_programs.iter().zip(color_programs) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        for (counter, count) in self.white_flowers_bytes.iter().zip(white_flowers_bytes) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.white_flowers_direct_bytes.fetch_add(white_flowers_direct_bytes, Ordering::Relaxed);
        for (counter, count) in self.additional_archive_bytes.iter().zip(additional_archive_bytes) {
            counter.fetch_add(count, Ordering::Relaxed);
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
            context_bytes: v[7],
            archives_used: u64::from(self.banks.load(Ordering::Relaxed).count_ones()),
            groups_used: u64::from(self.groups.load(Ordering::Relaxed).count_ones()),
            color_source_bytes: self.color_bytes.load(Ordering::Relaxed),
            color_archives_mask: (self.banks.load(Ordering::Relaxed) >> model::ARCHIVES.len()) & 7,
            color_programs_mask: self.color_programs.each_ref().map(|v| v.load(Ordering::Relaxed)),
            white_flowers_caller_literal_bytes: v[8],
            white_flowers_archive_bytes: self.white_flowers_bytes.each_ref().map(|v| v.load(Ordering::Relaxed)),
            white_flowers_direct_literal_bytes: v[9],
            white_flowers_direct_archive_bytes: self.white_flowers_direct_bytes.load(Ordering::Relaxed),
            additional_literal_bytes: v[10],
            additional_prefix_bytes: v[11],
            additional_archive_bytes: self.additional_archive_bytes.each_ref().map(|v| v.load(Ordering::Relaxed)),
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
    (model::CONTEXT_SOURCE as usize, model::CONTEXT_RECORDS * 4),
    (model::WHITE_FLOWERS_LITERALS as usize, 16),
    (model::WHITE_FLOWERS_DIRECT_LITERAL as usize, 4),
    (model::WHITE_FLOWERS_DIRECT_ROOT as usize, model::WHITE_FLOWERS_DIRECT_LENGTH),
    (model::WHITE_FLOWERS_GROUP as usize, 20),
    (model::WHITE_FLOWERS_ROOTS[0] as usize, model::WHITE_FLOWERS_LENGTHS[0]),
    (model::WHITE_FLOWERS_ROOTS[1] as usize, model::WHITE_FLOWERS_LENGTHS[1]),
    (model::WHITE_FLOWERS_ROOTS[2] as usize, model::WHITE_FLOWERS_LENGTHS[2]),
    (model::WHITE_FLOWERS_ROOTS[3] as usize, model::WHITE_FLOWERS_LENGTHS[3]),
    (model::WHITE_FLOWERS_ROOTS[4] as usize, model::WHITE_FLOWERS_LENGTHS[4]),
    (model::COLOR_ARCHIVE_ROOTS[0] as usize, model::COLOR_ARCHIVE_LENGTHS[0]),
    (model::COLOR_ARCHIVE_ROOTS[1] as usize, model::COLOR_ARCHIVE_LENGTHS[1]),
    (model::COLOR_ARCHIVE_ROOTS[2] as usize, model::COLOR_ARCHIVE_LENGTHS[2]),
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
