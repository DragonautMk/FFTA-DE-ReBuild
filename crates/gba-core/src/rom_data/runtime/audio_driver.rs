//! Typed physical driver tables; out-of-table live indices delegate to adjacent owners/raw ROM.
use super::*;
use crate::rom_data::{audio_driver as model, audio_sequences};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioDriverStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub table_bytes: [u64; 7],
    pub entries_used: [u64; 7],
    pub host_reads: u64,
    pub host_source_bytes: u64,
}
pub(super) struct AudioDriverResources {
    data: model::AudioDriver,
    counters: [AtomicU64; 11],
    entries: [[AtomicU64; 3]; 7],
}
impl AudioDriverResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x370
    }
    pub(super) fn new(
        data: model::AudioDriver,
        sources: &[SourceRange],
        owner: &audio_sequences::AudioSequences,
    ) -> Result<Self, String> {
        data.validate_sequence_owner(owner)?;
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Audio driver source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Audio driver typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            entries: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize, usize)> {
        for (table, &(_, start, count, width)) in model::TABLES.iter().enumerate() {
            if let Some(offset) = at
                .checked_sub(start as usize)
                .filter(|n| *n < count * width)
            {
                return Some((
                    self.data
                        .byte(table, offset)
                        .expect("validated typed driver byte"),
                    table,
                    offset / width,
                ));
            }
        }
        None
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
        let mut counts = [0u64; 11];
        let bytes = std::array::from_fn(|i| {
            if let Some((value, table, entry)) = fields[i] {
                if host {
                    counts[10] += 1;
                } else {
                    counts[1] += 1;
                    counts[2 + table] += 1;
                    self.entries[table][entry / 64].fetch_or(1 << (entry % 64), Ordering::Relaxed);
                }
                value
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        counts[if host { 9 } else { 0 }] = 1;
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
    pub(super) fn stats(&self) -> AudioDriverStats {
        let c = self.counters.each_ref().map(|n| n.load(Ordering::Relaxed));
        AudioDriverStats {
            bus_reads: c[0],
            source_bytes: c[1],
            table_bytes: std::array::from_fn(|i| c[2 + i]),
            entries_used: self.entries.each_ref().map(|words| {
                words
                    .iter()
                    .map(|w| u64::from(w.load(Ordering::Relaxed).count_ones()))
                    .sum()
            }),
            host_reads: c[9],
            host_source_bytes: c[10],
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.pcm_keys.entries.capacity() * std::mem::size_of::<model::KeyCode>()
            + self.data.pcm_bases.entries.capacity() * 4
            + self.data.sample_counts.entries.capacity() * 2
            + self.data.cgb_keys.entries.capacity() * std::mem::size_of::<model::KeyCode>()
            + self.data.cgb_bases.entries.capacity() * 2
            + self.data.noise_codes.entries.capacity()
            + self.data.wave_volume_codes.entries.capacity()
    }
}
