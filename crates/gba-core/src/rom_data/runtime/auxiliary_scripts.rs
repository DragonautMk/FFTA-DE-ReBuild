//! Typed auxiliary script fields. Original predicates, cursors and callback dispatch stay live.
use super::super::{auxiliary_scripts as model, script_commands::Command, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AuxiliaryCommandStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub advance_bytes: u64,
    pub callback_bytes: u64,
    pub commands_used: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AuxiliaryArchiveStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub index_bytes: u64,
    pub opcode_bytes: u64,
    pub parameter_bytes: u64,
    pub alignment_bytes: u64,
    pub banks_used: u64,
    pub programs_used: u64,
    pub alignment_spans_used: u64,
}
pub(super) struct AuxiliaryScriptResources {
    data: model::AuxiliaryArchive,
    commands: model::AuxiliaryCommands,
    counters: [AtomicU64; 10],
    command_mask: AtomicU64,
    bank_mask: AtomicU64,
    programs: [AtomicU64; 3],
    alignment_mask: AtomicU64,
}
impl AuxiliaryScriptResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x370 | 0xd69..=0xd6a)
    }
    pub(super) fn new(
        data: model::AuxiliaryArchive,
        commands: model::AuxiliaryCommands,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate_dependencies(&commands)?;
        let spans: Vec<_> = data
            .source_spans()?
            .into_iter()
            .chain(commands.source_spans()?)
            .collect();
        if spans.len() != sources.len() {
            return Err("Auxiliary source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Auxiliary typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            commands,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            command_mask: AtomicU64::new(0),
            bank_mask: AtomicU64::new(0),
            programs: std::array::from_fn(|_| AtomicU64::new(0)),
            alignment_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        if let Some(byte) = self.commands.byte_at(at) {
            let relative = (at - self.commands.source_offset) as usize;
            self.command_mask
                .fetch_or(1 << (relative / 6), Ordering::Relaxed);
            return Some((byte, if relative % 6 < 2 { 2 } else { 3 }));
        }
        let byte = self.data.byte_at(at)?;
        let relative = (at - self.data.source_offset) as usize;
        let (bank, category) = if relative < self.data.banks.len() * 2 {
            (relative / 2, 6)
        } else {
            let id = self
                .data
                .banks
                .partition_point(|b| b.source_offset <= at)
                .checked_sub(1)?;
            let bank = &self.data.banks[id];
            let category = if (at - bank.source_offset) < bank.entries.len() as u32 * 2 {
                6
            } else {
                let program = bank.programs.get(
                    bank.programs
                        .partition_point(|p| p.source_offset <= at)
                        .checked_sub(1)?,
                )?;
                let ins = program.instructions.get(
                    program
                        .instructions
                        .partition_point(|i| i.source_offset <= at)
                        .checked_sub(1)?,
                )?;
                let delta = (at - ins.source_offset) as usize;
                if delta <= ins.parameters.len() {
                    let program = usize::from(program.id);
                    self.programs[program / 64].fetch_or(1 << (program % 64), Ordering::Relaxed);
                    if delta == 0 {
                        7
                    } else {
                        8
                    }
                } else {
                    self.alignment_mask.fetch_or(1 << id, Ordering::Relaxed);
                    9
                }
            };
            (id, category)
        };
        self.bank_mask.fetch_or(1 << bank, Ordering::Relaxed);
        Some((byte, category))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0; 10];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, category)) = fields[i] {
                let base = if category < 4 { 0 } else { 4 };
                counts[base] = 1;
                counts[base + 1] += 1;
                counts[category] += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn command_stats(&self) -> AuxiliaryCommandStats {
        let [bus_reads, source_bytes, advance_bytes, callback_bytes, ..] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        AuxiliaryCommandStats {
            bus_reads,
            source_bytes,
            advance_bytes,
            callback_bytes,
            commands_used: u64::from(self.command_mask.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn archive_stats(&self) -> AuxiliaryArchiveStats {
        let [.., bus_reads, source_bytes, index_bytes, opcode_bytes, parameter_bytes, alignment_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        AuxiliaryArchiveStats {
            bus_reads,
            source_bytes,
            index_bytes,
            opcode_bytes,
            parameter_bytes,
            alignment_bytes,
            banks_used: u64::from(self.bank_mask.load(Ordering::Relaxed).count_ones()),
            programs_used: self
                .programs
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
            alignment_spans_used: u64::from(
                self.alignment_mask.load(Ordering::Relaxed).count_ones(),
            ),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.commands.commands.capacity() * std::mem::size_of::<Command>()
            + self.data.banks.capacity() * std::mem::size_of::<model::Bank>()
            + self
                .data
                .banks
                .iter()
                .map(|b| {
                    b.entries.capacity() * std::mem::size_of::<model::Entry>()
                        + b.alignment_after.capacity()
                        + b.programs.capacity() * std::mem::size_of::<model::Program>()
                        + b.programs
                            .iter()
                            .map(|p| {
                                p.instructions.capacity()
                                    * std::mem::size_of::<model::Instruction>()
                                    + p.instructions
                                        .iter()
                                        .map(|i| i.parameters.capacity())
                                        .sum::<usize>()
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
