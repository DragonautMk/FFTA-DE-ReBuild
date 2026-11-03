//! Original guest sequencer bus reads use typed instruction/table fields; RAM execution stays live.
use super::*;
use crate::rom_data::{audio_directory, audio_sequences as model, audio_tones};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioSequencesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub opcode_bytes: u64,
    pub operand_bytes: u64,
    pub clock_bytes: u64,
    pub command_bytes: u64,
    pub extended_bytes: u64,
    pub instructions_used: u64,
    pub clock_mask: u64,
    pub command_mask: u64,
    pub extended_mask: u64,
    pub host_reads: u64,
    pub host_source_bytes: u64,
}
struct Span {
    start: usize,
    end: usize,
    kind: u8,
    record: usize,
}
pub(super) struct AudioSequencesResources {
    data: model::AudioSequences,
    spans: Vec<Span>,
    counters: [AtomicU64; 9],
    instructions: Vec<AtomicU64>,
    instructions_used: AtomicU64,
    clock_mask: AtomicU64,
    command_mask: AtomicU64,
    extended_mask: AtomicU64,
}
impl AudioSequencesResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x337..=0x370)
    }
    pub(super) fn new(
        data: model::AudioSequences,
        sources: &[SourceRange],
        directory: &audio_directory::AudioDirectory,
        tones: &audio_tones::AudioTones,
    ) -> Result<Self, String> {
        data.validate_owners(directory, tones)?;
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Audio sequence source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Audio sequence typed source binding mismatch".into());
            }
        }
        let mut spans: Vec<_> = data
            .instructions
            .iter()
            .enumerate()
            .map(|(record, i)| Span {
                start: i.source_offset as usize,
                end: i.source_offset as usize + i.source_len(),
                kind: 0,
                record,
            })
            .collect();
        let d = &data.driver;
        spans.push(Span {
            start: d.clock_source_offset as usize,
            end: d.clock_source_offset as usize + 49,
            kind: 1,
            record: 0,
        });
        spans.push(Span {
            start: d.command_source_offset as usize,
            end: d.command_source_offset as usize + 144,
            kind: 2,
            record: 0,
        });
        spans.push(Span {
            start: d.extended_source_offset as usize,
            end: d.extended_source_offset as usize + 8,
            kind: 3,
            record: 0,
        });
        spans.sort_by_key(|s| s.start);
        Ok(Self {
            instructions: (0..data.instructions.len().div_ceil(64))
                .map(|_| AtomicU64::new(0))
                .collect(),
            data,
            spans,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            instructions_used: AtomicU64::new(0),
            clock_mask: AtomicU64::new(0),
            command_mask: AtomicU64::new(0),
            extended_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize, usize)> {
        let i = self
            .spans
            .partition_point(|s| s.start <= at)
            .checked_sub(1)?;
        let s = &self.spans[i];
        if at >= s.end {
            return None;
        }
        let offset = at - s.start;
        Some(match s.kind {
            0 => {
                let i = &self.data.instructions[s.record];
                (
                    self.data
                        .byte(i.id, offset)
                        .expect("validated typed sequence byte"),
                    if i.explicit && offset == 0 { 2 } else { 3 },
                    s.record,
                )
            }
            1 => (self.data.driver.clocks[offset], 4, offset),
            2 => (
                self.data.driver.commands[offset / 4].to_le_bytes()[offset % 4],
                5,
                offset / 4,
            ),
            _ => (
                self.data.driver.extended8_9[offset / 4].to_le_bytes()[offset % 4],
                6,
                offset / 4,
            ),
        })
    }
    fn read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
        host: bool,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 9];
        let bytes = std::array::from_fn(|i| {
            if let Some((value, kind, record)) = fields[i] {
                if host {
                    counts[8] += 1;
                } else {
                    counts[1] += 1;
                    counts[kind] += 1;
                    match kind {
                        2 | 3 => {
                            let bit = 1 << (record % 64);
                            if self.instructions[record / 64].fetch_or(bit, Ordering::Relaxed) & bit
                                == 0
                            {
                                self.instructions_used.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        4 => {
                            self.clock_mask.fetch_or(1 << record, Ordering::Relaxed);
                        }
                        5 => {
                            self.command_mask.fetch_or(1 << record, Ordering::Relaxed);
                        }
                        _ => {
                            self.extended_mask.fetch_or(1 << record, Ordering::Relaxed);
                        }
                    }
                }
                value
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        counts[if host { 7 } else { 0 }] = 1;
        for (c, n) in self.counters.iter().zip(counts) {
            c.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        self.read(index, rom, prior, false)
    }
    pub(super) fn host_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        self.read(index, rom, prior, true)
    }
    pub(super) fn stats(&self) -> AudioSequencesStats {
        let [bus_reads, source_bytes, opcode_bytes, operand_bytes, clock_bytes, command_bytes, extended_bytes, host_reads, host_source_bytes] =
            self.counters.each_ref().map(|n| n.load(Ordering::Relaxed));
        AudioSequencesStats {
            bus_reads,
            source_bytes,
            opcode_bytes,
            operand_bytes,
            clock_bytes,
            command_bytes,
            extended_bytes,
            host_reads,
            host_source_bytes,
            instructions_used: self.instructions_used.load(Ordering::Relaxed),
            clock_mask: self.clock_mask.load(Ordering::Relaxed),
            command_mask: self.command_mask.load(Ordering::Relaxed),
            extended_mask: self.extended_mask.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.spans.capacity() * std::mem::size_of::<Span>()
            + self.instructions.capacity() * std::mem::size_of::<AtomicU64>()
            + self.data.instructions.capacity() * std::mem::size_of::<model::Instruction>()
            + self.data.tracks.capacity() * std::mem::size_of::<model::Track>()
            + self.data.driver.clocks.capacity()
            + self.data.driver.commands.capacity() * 4
            + self.data.driver.extended8_9.capacity() * 4
    }
}
