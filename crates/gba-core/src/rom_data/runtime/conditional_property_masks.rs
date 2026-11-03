//! Typed projection of the conditional mask words; guest arithmetic is unchanged.
use super::*;
use crate::rom_data::conditional_property_masks as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PropertyMasksStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
}

pub(super) struct PropertyMasksResources {
    data: model::Masks,
    counters: [AtomicU64; 2],
}

impl PropertyMasksResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x6e0 }

    pub(super) fn new(data: model::Masks, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != model::RECORD_COUNT * 4
            || sources[0].role != "conditional-property-masks"
            || sources[0].sha256 != model::SOURCE_SHA256
            || ffta_rom::digest(&bytes) != model::SOURCE_SHA256 {
            return Err("Conditional property masks source binding mismatch".into());
        }
        Ok(Self { data, counters: std::array::from_fn(|_| AtomicU64::new(0)) })
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let start = model::SOURCE_OFFSET as usize;
        let end = start + model::RECORD_COUNT * 4;
        if index >= end || index.saturating_add(N) <= start { return prior; }
        let mut source_bytes = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(byte) = self.data.byte_at(at as u32) {
                source_bytes += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[at], |b| b[i])
            }
        });
        self.counters[0].fetch_add(1, Ordering::Relaxed);
        self.counters[1].fetch_add(source_bytes, Ordering::Relaxed);
        Some(bytes)
    }

    pub(super) fn stats(&self) -> PropertyMasksStats {
        PropertyMasksStats { bus_reads: self.counters[0].load(Ordering::Relaxed), source_bytes: self.counters[1].load(Ordering::Relaxed) }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.words.capacity() * std::mem::size_of::<u32>()
    }
}
