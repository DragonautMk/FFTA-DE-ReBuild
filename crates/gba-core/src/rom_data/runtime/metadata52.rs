//! Typed byte ownership; original packed-field/redirect readers keep executing.
use super::*;
use crate::rom_data::metadata52 as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata52Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub header_bytes: u64,
    pub routing_bytes: u64,
    pub packed_bytes: u64,
    pub tail_bytes: u64,
    pub opaque_bytes: u64,
}
pub(super) struct Metadata52Resources {
    data: model::Metadata52,
    counters: [AtomicU64; 7],
}
impl Metadata52Resources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x6e5..=0x6e6)
    }
    pub(super) fn new(
        data: model::Metadata52,
        sources: &[SourceRange],
        links: &crate::rom_data::item_links::Links,
    ) -> Result<Self, String> {
        data.validate_links(links)?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::END_OFFSET - model::SOURCE_OFFSET
            || sources[0].role != "metadata52/eur-fixed"
            || sources[0].sha256 != ffta_rom::digest(&data.source_bytes())
        {
            return Err("Metadata52 source/file binding mismatch".into());
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
        let mut counts = [0u64; 7];
        let bytes = std::array::from_fn(|i| {
            let at = (index + i).checked_sub(model::SOURCE_OFFSET as usize);
            if let Some(at) =
                at.filter(|&at| at < (model::END_OFFSET - model::SOURCE_OFFSET) as usize)
            {
                let offset = at % model::RECORD_BYTES;
                counts[1] += 1;
                counts[match offset {
                    0..=1 | 4 | 6..=10 => 2,
                    5 => 3,
                    11..=38 | 40..=49 => 4,
                    50..=51 => 5,
                    _ => 6,
                }] += 1;
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
    pub(super) fn stats(&self) -> Metadata52Stats {
        let [bus_reads, source_bytes, header_bytes, routing_bytes, packed_bytes, tail_bytes, opaque_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        Metadata52Stats {
            bus_reads,
            source_bytes,
            header_bytes,
            routing_bytes,
            packed_bytes,
            tail_bytes,
            opaque_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
