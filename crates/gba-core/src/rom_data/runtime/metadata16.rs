//! Typed byte ownership; original live-list readers and RNG keep executing.
use super::*;
use crate::rom_data::metadata16 as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata16Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub alias_bytes: u64,
    pub display_bytes: u64,
    pub flag_bytes: u64,
    pub value_bytes: u64,
    pub opaque_bytes: u64,
}
pub(super) struct Metadata16Resources {
    data: model::Metadata16,
    counters: [AtomicU64; 7],
}
impl Metadata16Resources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x6e4..=0x6e5)
    }
    pub(super) fn new(
        data: model::Metadata16,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::END_OFFSET - model::SOURCE_OFFSET
            || sources[0].role != "metadata16/eur-fixed"
            || sources[0].sha256 != ffta_rom::digest(&data.source_bytes())
            || file_sha256.len() != 64
        {
            return Err("Metadata16 source/file binding mismatch".into());
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
                    0..=1 => 2,
                    2 => 3,
                    7 => 4,
                    9..=14 => 5,
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
    pub(super) fn stats(&self) -> Metadata16Stats {
        let [bus_reads, source_bytes, alias_bytes, display_bytes, flag_bytes, value_bytes, opaque_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        Metadata16Stats {
            bus_reads,
            source_bytes,
            alias_bytes,
            display_bytes,
            flag_bytes,
            value_bytes,
            opaque_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
