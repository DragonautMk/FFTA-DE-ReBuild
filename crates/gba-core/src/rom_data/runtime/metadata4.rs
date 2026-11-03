//! Exact pair bytes; original counting, walking and live qualification keep running.
use super::*;
use crate::rom_data::metadata4 as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata4Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub value_bytes: u64,
}
pub(super) struct Metadata4Resources {
    data: model::Metadata4,
    counters: [AtomicU64; 4],
}
impl Metadata4Resources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e6
    }
    pub(super) fn new(
        data: model::Metadata4,
        sources: &[SourceRange],
        owner: &crate::rom_data::metadata52::Metadata52,
    ) -> Result<Self, String> {
        data.validate_metadata52(owner)?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::END_OFFSET - model::SOURCE_OFFSET
            || sources[0].role != "metadata4/eur-fixed"
            || sources[0].sha256 != ffta_rom::digest(&data.source_bytes())
        {
            return Err("Metadata4 source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let bytes = std::array::from_fn(|i| {
            let at = (index + i).checked_sub(model::SOURCE_OFFSET as usize);
            if let Some(at) =
                at.filter(|&at| at < (model::END_OFFSET - model::SOURCE_OFFSET) as usize)
            {
                let offset = at % model::RECORD_BYTES;
                counts[1] += 1;
                counts[2 + offset % 2] += 1;
                self.data.records[at / model::RECORD_BYTES]
                    .byte(offset)
                    .unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |b| b[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> Metadata4Stats {
        let [bus_reads, source_bytes, selector_bytes, value_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        Metadata4Stats {
            bus_reads,
            source_bytes,
            selector_bytes,
            value_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
