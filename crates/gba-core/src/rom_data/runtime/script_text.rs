//! Typed localized text ownership. Guest selection, decompression, clocks and live RAM remain original.
use super::super::{script_text as model, text_catalogs::ProgramUnit, SourceRange};
use super::text_program::TextProgramMap;
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptTextStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub top_index_bytes: u64,
    pub variant_index_bytes: u64,
    pub bank_index_bytes: u64,
    pub header_bytes: u64,
    pub encoded_control_bytes: u64,
    pub glyph_bytes: u64,
    pub command_bytes: u64,
    pub operand_bytes: u64,
    pub terminator_bytes: u64,
    pub decoded_tail_bytes: u64,
    pub alignment_bytes: u64,
    pub decoded_program_bytes: u64,
    pub languages_used: u64,
    pub banks_used: u64,
    pub programs_used: u64,
    pub variant_records_used: u64,
    pub alignment_spans_used: u64,
}
#[derive(Clone, Copy)]
enum Kind {
    Top(usize),
    Variants(usize),
    Index(usize, usize),
    Program(usize),
    Alignment(usize, usize),
}
struct Binding {
    offset: usize,
    length: usize,
    kind: Kind,
}
struct ProgramMap {
    language: usize,
    bank: usize,
    item: usize,
    map: TextProgramMap,
}
pub(super) struct ScriptTextResources {
    data: model::ScriptText,
    bindings: Vec<Binding>,
    programs: Vec<ProgramMap>,
    bus_reads: AtomicU64,
    counts: [AtomicU64; 11],
    languages: AtomicU64,
    banks: [AtomicU64; 7],
    program_mask: [AtomicU64; 641],
    variants: [AtomicU64; 3],
    alignment: [AtomicU64; 537],
}
impl ScriptTextResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0xbb0..=0xd68)
    }
    pub(super) fn new(data: model::ScriptText, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Script text source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Script text typed source binding mismatch".into());
            }
        }
        let mut bindings = Vec::new();
        let mut programs = Vec::new();
        for (li, language) in data.languages.iter().enumerate() {
            bindings.push(Binding {
                offset: language.source_offset as usize,
                length: 252,
                kind: Kind::Top(li),
            });
            bindings.push(Binding {
                offset: language.variants.source_offset as usize,
                length: 104,
                kind: Kind::Variants(li),
            });
            for (bi, bank) in language.banks.iter().enumerate() {
                bindings.push(Binding {
                    offset: bank.source_offset as usize,
                    length: (bank.programs.len() + 1) * 2,
                    kind: Kind::Index(li, bi),
                });
                for (pi, program) in bank.programs.iter().enumerate() {
                    let map = TextProgramMap::new(program)?;
                    let id = programs.len();
                    bindings.push(Binding {
                        offset: program.source_offset as usize,
                        length: map.source_len(program),
                        kind: Kind::Program(id),
                    });
                    programs.push(ProgramMap {
                        language: li,
                        bank: bi,
                        item: pi,
                        map,
                    });
                }
            }
        }
        for (i, gap) in data.alignment.iter().enumerate() {
            let li = model::ROOTS.partition_point(|&a| a <= gap.source_offset) - 1;
            bindings.push(Binding {
                offset: gap.source_offset as usize,
                length: gap.bytes.len(),
                kind: Kind::Alignment(li, i),
            });
        }
        bindings.sort_by_key(|b| b.offset);
        let mut cursor = model::ROOTS[0] as usize;
        for binding in &bindings {
            if binding.offset != cursor || binding.length == 0 {
                return Err("Script text runtime source gap/overlap".into());
            }
            cursor = cursor
                .checked_add(binding.length)
                .ok_or("Script text runtime extent overflow")?;
        }
        if cursor != model::END as usize || programs.len() != 41010 {
            return Err("Script text incomplete runtime source ownership".into());
        }
        Ok(Self {
            data,
            bindings,
            programs,
            bus_reads: AtomicU64::new(0),
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            languages: AtomicU64::new(0),
            banks: std::array::from_fn(|_| AtomicU64::new(0)),
            program_mask: std::array::from_fn(|_| AtomicU64::new(0)),
            variants: std::array::from_fn(|_| AtomicU64::new(0)),
            alignment: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn mark(masks: &[AtomicU64], id: usize) {
        masks[id / 64].fetch_or(1u64 << (id % 64), Ordering::Relaxed);
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let binding = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - binding.offset;
        if at >= binding.length {
            return None;
        }
        let (byte, category, li) = match binding.kind {
            Kind::Top(li) => {
                let language = &self.data.languages[li];
                let target = language.target_offset(&language.top[at / 4])?;
                (
                    target.wrapping_sub(language.source_offset).to_le_bytes()[at % 4],
                    0,
                    li,
                )
            }
            Kind::Variants(li) => {
                let language = &self.data.languages[li];
                let bank = &language.banks[usize::from(language.variants.bank_indices[at / 4])];
                Self::mark(&self.variants, li * 26 + at / 4);
                (
                    bank.source_offset
                        .wrapping_sub(language.source_offset)
                        .to_le_bytes()[at % 4],
                    1,
                    li,
                )
            }
            Kind::Index(li, bi) => {
                let bank = &self.data.languages[li].banks[bi];
                let value = bank.programs.get(at / 2).map_or(bank.sentinel, |p| {
                    p.source_offset.wrapping_sub(bank.source_offset) as u16
                });
                Self::mark(&self.banks, li * 77 + bi);
                (value.to_le_bytes()[at % 2], 2, li)
            }
            Kind::Program(id) => {
                let map = &self.programs[id];
                let p = &self.data.languages[map.language].banks[map.bank].programs[map.item];
                let (byte, kind) = map.map.source_byte(p, at)?;
                Self::mark(&self.program_mask, id);
                Self::mark(&self.banks, map.language * 77 + map.bank);
                (byte, kind as usize + 3, map.language)
            }
            Kind::Alignment(li, id) => {
                Self::mark(&self.alignment, id);
                (self.data.alignment[id].bytes[at], 10, li)
            }
        };
        self.languages.fetch_or(1u64 << li, Ordering::Relaxed);
        Some((byte, category))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= model::END as usize || index.saturating_add(N) <= model::ROOTS[0] as usize {
            return prior;
        }
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0; 11];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, kind)) = fields[i] {
                counts[kind] += 1;
                byte
            } else {
                prior
                    .as_ref()
                    .map_or_else(|| rom[index + i], |bytes| bytes[i])
            }
        });
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        for (counter, n) in self.counts.iter().zip(counts) {
            counter.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ScriptTextStats {
        let [top_index_bytes, variant_index_bytes, bank_index_bytes, header_bytes, encoded_control_bytes, glyph_bytes, command_bytes, operand_bytes, terminator_bytes, decoded_tail_bytes, alignment_bytes] =
            self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        let used = |masks: &[AtomicU64]| {
            masks
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum()
        };
        let decoded_program_bytes =
            glyph_bytes + command_bytes + operand_bytes + terminator_bytes + decoded_tail_bytes;
        ScriptTextStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: top_index_bytes
                + variant_index_bytes
                + bank_index_bytes
                + header_bytes
                + encoded_control_bytes
                + decoded_program_bytes
                + alignment_bytes,
            top_index_bytes,
            variant_index_bytes,
            bank_index_bytes,
            header_bytes,
            encoded_control_bytes,
            glyph_bytes,
            command_bytes,
            operand_bytes,
            terminator_bytes,
            decoded_tail_bytes,
            alignment_bytes,
            decoded_program_bytes,
            languages_used: u64::from(self.languages.load(Ordering::Relaxed).count_ones()),
            banks_used: used(&self.banks),
            programs_used: used(&self.program_mask),
            variant_records_used: used(&self.variants),
            alignment_spans_used: used(&self.alignment),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.programs.capacity() * std::mem::size_of::<ProgramMap>()
            + self
                .programs
                .iter()
                .map(|p| p.map.resident_bytes())
                .sum::<usize>()
            + self.data.alignment.capacity() * std::mem::size_of::<model::Alignment>()
            + self
                .data
                .alignment
                .iter()
                .map(|g| g.bytes.capacity())
                .sum::<usize>()
            + self.data.languages.capacity() * std::mem::size_of::<model::Language>()
            + self
                .data
                .languages
                .iter()
                .map(|l| {
                    l.top.capacity() * std::mem::size_of::<model::TopTarget>()
                        + l.variants.bank_indices.capacity()
                        + l.banks.capacity() * std::mem::size_of::<model::Bank>()
                        + l.banks
                            .iter()
                            .map(|b| {
                                b.programs.capacity()
                                    * std::mem::size_of::<super::super::text_catalogs::Program>()
                                    + b.programs
                                        .iter()
                                        .map(|p| {
                                            p.units.capacity() * std::mem::size_of::<ProgramUnit>()
                                                + p.units
                                                    .iter()
                                                    .map(|u| match u {
                                                        ProgramUnit::Command {
                                                            operands, ..
                                                        } => operands.capacity(),
                                                        _ => 0,
                                                    })
                                                    .sum::<usize>()
                                                + p.decoded_padding.capacity()
                                                + p.encoded.as_ref().map_or(0, Vec::capacity)
                                        })
                                        .sum::<usize>()
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
