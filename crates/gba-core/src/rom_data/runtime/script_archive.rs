//! File-derived archive storage, without a host script interpreter or ROM shadow.
use super::super::{script_archive as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptArchiveStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub index_bytes: u64,
    pub opcode_bytes: u64,
    pub parameter_bytes: u64,
    pub key_bytes: u64,
    pub padding_bytes: u64,
    pub banks_used: u64,
    pub programs_used: u64,
    pub key_records_used: u64,
    pub padding_spans_used: u64,
}
pub(super) struct ScriptArchiveResources {
    data: model::ScriptArchive,
    counters: [AtomicU64; 7],
    banks: AtomicU64,
    programs: [AtomicU64; 4],
    keys: [AtomicU64; 4],
    padding: AtomicU64,
}
impl ScriptArchiveResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0xb94..=0xbb0 | 0xd68)
    }
    pub(super) fn new(data: model::ScriptArchive, sources: &[SourceRange]) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Script archive binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Script archive typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            banks: AtomicU64::new(0),
            programs: std::array::from_fn(|_| AtomicU64::new(0)),
            keys: std::array::from_fn(|_| AtomicU64::new(0)),
            padding: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        if let Some(n) = at.checked_sub(self.data.key_offset) {
            if let Some(key) = self.data.keys.get(n as usize / 4) {
                self.keys[n as usize / 4 / 64].fetch_or(1 << ((n / 4) % 64), Ordering::Relaxed);
                return Some((key.byte(n as usize % 4)?, 5));
            }
        }
        if let Some(n) = at.checked_sub(self.data.archive_offset) {
            if n < self.data.banks.len() as u32 * 4 {
                self.banks.fetch_or(1 << (n / 4), Ordering::Relaxed);
                return Some((
                    self.data.banks[n as usize / 4]
                        .source_offset
                        .wrapping_sub(self.data.archive_offset)
                        .to_le_bytes()[n as usize % 4],
                    2,
                ));
            }
        }
        if let Some(n) = self
            .data
            .banks
            .partition_point(|b| b.source_offset <= at)
            .checked_sub(1)
        {
            let bank = &self.data.banks[n];
            let relative = (at - bank.source_offset) as usize;
            if let Some(slot) = bank.slots.get(relative / 4) {
                self.banks.fetch_or(1 << n, Ordering::Relaxed);
                return Some((
                    self.data
                        .target_offset(&slot.target)?
                        .wrapping_sub(bank.source_offset)
                        .to_le_bytes()[relative % 4],
                    2,
                ));
            }
        }
        if let Some(n) = self
            .data
            .programs
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)
        {
            let program = &self.data.programs[n];
            if let Some(i) = program
                .instructions
                .partition_point(|i| i.source_offset <= at)
                .checked_sub(1)
            {
                let instruction = &program.instructions[i];
                let relative = (at - instruction.source_offset) as usize;
                let field = if relative == 0 {
                    Some((instruction.command, 3))
                } else {
                    instruction.parameters.get(relative - 1).map(|v| (*v, 4))
                };
                if field.is_some() {
                    self.programs[n / 64].fetch_or(1 << (n % 64), Ordering::Relaxed);
                    return field;
                }
            }
        }
        if let Some(n) = self
            .data
            .padding
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)
        {
            let fill = &self.data.padding[n];
            if at - fill.source_offset < u32::from(fill.length) {
                self.padding.fetch_or(1 << n, Ordering::Relaxed);
                return Some((fill.value, 6));
            }
        }
        None
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
        let mut counts = [1, 0, 0, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, category)) = fields[i] {
                counts[1] += 1;
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
    pub(super) fn stats(&self) -> ScriptArchiveStats {
        let [bus_reads, source_bytes, index_bytes, opcode_bytes, parameter_bytes, key_bytes, padding_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        let used = |masks: &[AtomicU64]| {
            masks
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum()
        };
        ScriptArchiveStats {
            bus_reads,
            source_bytes,
            index_bytes,
            opcode_bytes,
            parameter_bytes,
            key_bytes,
            padding_bytes,
            banks_used: u64::from(self.banks.load(Ordering::Relaxed).count_ones()),
            programs_used: used(&self.programs),
            key_records_used: used(&self.keys),
            padding_spans_used: u64::from(self.padding.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.keys.capacity() * std::mem::size_of::<model::EventKey>()
            + self.data.banks.capacity() * std::mem::size_of::<model::Bank>()
            + self
                .data
                .banks
                .iter()
                .map(|b| b.slots.capacity() * std::mem::size_of::<model::Slot>())
                .sum::<usize>()
            + self.data.programs.capacity() * std::mem::size_of::<model::Program>()
            + self
                .data
                .programs
                .iter()
                .map(|p| {
                    p.instructions.capacity() * std::mem::size_of::<model::Instruction>()
                        + p.instructions
                            .iter()
                            .map(|i| i.parameters.capacity())
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.data.padding.capacity() * std::mem::size_of::<model::AlignmentFill>()
    }
}
