//! Serves only the extra typed callback words; the guest still dispatches them.
use super::*;
use crate::rom_data::audio_extended_dispatch::AudioExtendedDispatch;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioExtendedDispatchStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_mask: u64,
    pub host_reads: u64,
    pub host_source_bytes: u64,
}

pub(super) struct AudioExtendedDispatchResources {
    data: AudioExtendedDispatch,
    counters: [AtomicU64; 4],
    selectors: AtomicU64,
}

impl AudioExtendedDispatchResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x370
    }

    pub(super) fn new(
        data: AudioExtendedDispatch,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("M4A extended dispatch source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("M4A extended dispatch source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            selectors: AtomicU64::new(0),
        })
    }

    fn read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
        host: bool,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| {
            u32::try_from(index + i)
                .ok()
                .and_then(|at| self.data.byte_at(at))
        });
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut served = 0u64;
        let out = std::array::from_fn(|i| match fields[i] {
            Some(value) => {
                served += 1;
                if !host {
                    let selector = (index + i - self.data.source_offset as usize) / 4;
                    self.selectors.fetch_or(1u64 << selector, Ordering::Relaxed);
                }
                value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |v| v[i]),
        });
        let slot = if host { 2 } else { 0 };
        self.counters[slot].fetch_add(1, Ordering::Relaxed);
        self.counters[slot + 1].fetch_add(served, Ordering::Relaxed);
        Some(out)
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

    pub(super) fn stats(&self) -> AudioExtendedDispatchStats {
        let [bus_reads, source_bytes, host_reads, host_source_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        AudioExtendedDispatchStats {
            bus_reads,
            source_bytes,
            selector_mask: self.selectors.load(Ordering::Relaxed),
            host_reads,
            host_source_bytes,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.entries.capacity()
                * std::mem::size_of::<crate::rom_data::audio_extended_dispatch::Entry>()
    }
}
