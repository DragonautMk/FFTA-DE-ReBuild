//! Typed item-derived fragment; the original selectors and guest control flow remain live.
use super::*;
use crate::rom_data::{conditional_item_parameters as model, items};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConditionalItemParametersStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub byte00: u64,
    pub byte01: u64,
    pub value02_bytes: u64,
    pub record_mask: u64,
}

pub(super) struct ConditionalItemParametersResources {
    data: model::ConditionalItemParameters,
    counters: [AtomicU64; 5],
    records: AtomicU64,
}
impl ConditionalItemParametersResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e8
    }
    pub(super) fn new(
        data: model::ConditionalItemParameters,
        sources: &[SourceRange],
        items: &items::Items,
    ) -> Result<Self, String> {
        data.validate_items(items)?;
        let bytes = data.source_bytes()?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != model::SOURCE_BYTES
            || sources[0].role != "conditional-item-parameters"
            || sources[0].sha256 != model::SOURCE_SHA256
            || ffta_rom::digest(&bytes) != model::SOURCE_SHA256
        {
            return Err("Conditional item parameters source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            records: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let start = model::SOURCE_OFFSET as usize;
        let end = start + model::SOURCE_BYTES;
        if index >= end || index.saturating_add(N) <= start {
            return prior;
        }
        let mut counts = [0u64; 5];
        counts[0] = 1;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(byte) = self.data.byte_at(at as u32) {
                let relative = at - start;
                counts[1] += 1;
                counts[2 + (relative % 4).min(2)] += 1;
                self.records
                    .fetch_or(1u64 << (relative / 4), Ordering::Relaxed);
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[at], |b| b[i])
            }
        });
        for (counter, amount) in self.counters.iter().zip(counts) {
            counter.fetch_add(amount, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ConditionalItemParametersStats {
        let [bus_reads, source_bytes, byte00, byte01, value02_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        ConditionalItemParametersStats {
            bus_reads,
            source_bytes,
            byte00,
            byte01,
            value02_bytes,
            record_mask: self.records.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
