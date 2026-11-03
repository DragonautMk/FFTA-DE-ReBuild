//! Typed packed-pair source bytes; original row/slot/mode instructions remain.
use super::*;
use crate::rom_data::metadata12 as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata12Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub primary_bytes: u64,
    pub secondary_bytes: u64,
    pub record_mask: [u64; 2],
}
impl Metadata12Stats {
    pub fn records_used(&self) -> u32 {
        self.record_mask.iter().map(|v| v.count_ones()).sum()
    }
}
pub(super) struct Metadata12Resources {
    data: model::Metadata12,
    counters: [AtomicU64; 4],
    record_mask: [AtomicU64; 2],
}
impl Metadata12Resources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e7
    }
    pub(super) fn new(data: model::Metadata12, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 2 {
            return Err("Metadata12 requires both bank source bindings".into());
        }
        for (i, (bank, source)) in data.banks.iter().zip(sources).enumerate() {
            let bytes: Vec<_> = bank
                .records
                .iter()
                .flat_map(model::Record::source_bytes)
                .collect();
            if source.offset != bank.source_offset
                || source.length as usize != bytes.len()
                || source.sha256 != ffta_rom::digest(&bytes)
                || source.role
                    != if i == 0 {
                        "metadata12/primary-records"
                    } else {
                        "metadata12/secondary-records"
                    }
            {
                return Err("Metadata12 bank source binding mismatch".into());
            }
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
        let mut counts = [0u64; 4];
        let mut mask = [0u64; 2];
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let b = usize::from(at >= model::SECONDARY_OFFSET as usize);
                let bank = &self.data.banks[b];
                let offset = at - bank.source_offset as usize;
                let record = offset / model::RECORD_BYTES;
                let global_record = if b == 0 { record } else { 92 + record };
                mask[global_record / 64] |= 1 << (global_record % 64);
                counts[1] += 1;
                counts[2 + b] += 1;
                bank.records[record]
                    .byte(offset % model::RECORD_BYTES)
                    .unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
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
    pub(super) fn stats(&self) -> Metadata12Stats {
        let [bus_reads, source_bytes, primary_bytes, secondary_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        Metadata12Stats {
            bus_reads,
            source_bytes,
            primary_bytes,
            secondary_bytes,
            record_mask: self
                .record_mask
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .data
                .banks
                .iter()
                .map(|b| b.records.capacity() * std::mem::size_of::<model::Record>())
                .sum::<usize>()
    }
}
