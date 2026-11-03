//! Typed special-unit bytes; original getters, live palettes and initialization remain.
use super::*;
use crate::rom_data::metadata14 as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata14Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub byte_fields: u64,
    pub halfword_bytes: u64,
    pub packed_bytes: u64,
    pub tail_bytes: u64,
    pub record_mask: [u64; 2],
}
impl Metadata14Stats {
    pub fn records_used(&self) -> u32 {
        self.record_mask.iter().map(|v| v.count_ones()).sum()
    }
}
pub(super) struct Metadata14Resources {
    data: model::Metadata14,
    counters: [AtomicU64; 6],
    record_mask: [AtomicU64; 2],
}
impl Metadata14Resources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x6e6 | 0x6e7)
    }
    pub(super) fn new(
        data: model::Metadata14,
        sources: &[SourceRange],
        owner: &crate::rom_data::metadata52::Metadata52,
    ) -> Result<Self, String> {
        data.validate_metadata52(owner)?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::END_OFFSET - model::SOURCE_OFFSET
            || sources[0].role != "metadata14/eur-records"
            || sources[0].sha256 != ffta_rom::digest(&data.source_bytes())
        {
            return Err("Metadata14 source binding mismatch".into());
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
        let mut counts = [0u64; 6];
        let mut mask = [0u64; 2];
        let bytes = std::array::from_fn(|i| {
            let offset = (index + i).checked_sub(model::SOURCE_OFFSET as usize);
            if let Some(offset) =
                offset.filter(|&at| at < (model::END_OFFSET - model::SOURCE_OFFSET) as usize)
            {
                let record = offset / model::RECORD_BYTES;
                let field = offset % model::RECORD_BYTES;
                mask[record / 64] |= 1 << (record % 64);
                counts[1] += 1;
                counts[match field {
                    0..=3 => 2,
                    4..=7 => 3,
                    8..=10 => 4,
                    _ => 5,
                }] += 1;
                self.data.records[record].byte(field).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |b| b[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, bits) in self.record_mask.iter().zip(mask) {
            counter.fetch_or(bits, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> Metadata14Stats {
        let [bus_reads, source_bytes, byte_fields, halfword_bytes, packed_bytes, tail_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        Metadata14Stats {
            bus_reads,
            source_bytes,
            byte_fields,
            halfword_bytes,
            packed_bytes,
            tail_bytes,
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
