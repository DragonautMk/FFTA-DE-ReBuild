//! Typed row fields; original live progression and price code still executes.
use super::*;
use crate::rom_data::progression_rewards as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProgressionRewardsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub delta_bytes: u64,
    pub discount_bytes: u64,
    pub opaque_bytes: u64,
    pub record_mask: [u64; 2],
}
pub(super) struct ProgressionRewardsResources {
    data: model::ProgressionRewards,
    counters: [AtomicU64; 5],
    record_mask: [AtomicU64; 2],
}
impl ProgressionRewardsResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e7
    }
    pub(super) fn new(
        data: model::ProgressionRewards,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1 {
            return Err("Progression rewards require one source binding".into());
        }
        let source = &sources[0];
        let bytes = data.source_bytes();
        if source.offset != model::SOURCE_OFFSET
            || source.length as usize != bytes.len()
            || source.role != "progression-rewards/eur-records"
            || source.sha256 != ffta_rom::digest(&bytes)
        {
            return Err("Progression reward source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            record_mask: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut counts = [0u64; 5];
        let mut masks = [0u64; 2];
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::SOURCE_OFFSET as usize;
                let row = offset / model::RECORD_BYTES;
                let field = offset % model::RECORD_BYTES;
                counts[1] += 1;
                counts[if field < 8 {
                    2
                } else if field == 8 {
                    3
                } else {
                    4
                }] += 1;
                masks[row / 64] |= 1u64 << (row % 64);
                self.data.records[row].byte(field).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, mask) in self.record_mask.iter().zip(masks) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ProgressionRewardsStats {
        let [bus_reads, source_bytes, delta_bytes, discount_bytes, opaque_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        ProgressionRewardsStats {
            bus_reads,
            source_bytes,
            delta_bytes,
            discount_bytes,
            opaque_bytes,
            record_mask: self
                .record_mask
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
