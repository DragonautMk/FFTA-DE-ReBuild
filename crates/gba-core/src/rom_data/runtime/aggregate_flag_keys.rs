//! Typed keys and sentinel; original global/context counting instructions remain.
use super::*;
use crate::rom_data::aggregate_flag_keys as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AggregateFlagKeysStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub key_bytes: u64,
    pub terminator_bytes: u64,
    pub entry_mask: [u64; 5],
}
pub(super) struct AggregateFlagKeysResources {
    data: model::AggregateFlagKeys,
    counters: [AtomicU64; 4],
    entry_mask: [AtomicU64; 5],
}
impl AggregateFlagKeysResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e9
    }
    pub(super) fn new(
        data: model::AggregateFlagKeys,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1 {
            return Err("Aggregate flag keys require one source binding".into());
        }
        let source = &sources[0];
        let bytes = data.source_bytes();
        if source.offset != model::SOURCE_OFFSET
            || source.length as usize != bytes.len()
            || source.role != "aggregate-flag-keys/eur-ordered-keys"
            || source.sha256 != ffta_rom::digest(&bytes)
        {
            return Err("Aggregate flag key source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            entry_mask: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= model::END_OFFSET as usize
            || index.saturating_add(N) <= model::SOURCE_OFFSET as usize
        {
            return prior;
        }
        let mut counts = [0u64; 4];
        let mut mask = [0u64; 5];
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::SOURCE_OFFSET as usize;
                counts[1] += 1;
                if offset < model::ENTRY_COUNT * 2 {
                    counts[2] += 1;
                    let entry = offset / 2;
                    mask[entry / 64] |= 1u64 << (entry % 64);
                } else {
                    counts[3] += 1;
                }
                self.data.byte(offset).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, bits) in self.entry_mask.iter().zip(mask) {
            counter.fetch_or(bits, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> AggregateFlagKeysStats {
        let [bus_reads, source_bytes, key_bytes, terminator_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        AggregateFlagKeysStats {
            bus_reads,
            source_bytes,
            key_bytes,
            terminator_bytes,
            entry_mask: self
                .entry_mask
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.entries.capacity() * std::mem::size_of::<model::Entry>()
    }
}
