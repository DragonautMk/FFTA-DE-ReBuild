//! Counted item-link fields serve original reads; unit publication remains live.
use super::*;
use crate::rom_data::item_links as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemLinkStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub count_bytes: u64,
    pub selector_bytes: u64,
    pub value_bytes: u64,
    pub opaque_bytes: u64,
}
pub(super) struct ItemLinkResources {
    links: model::Links,
    file_sha256: String,
    counters: [AtomicU64; 6],
}
impl ItemLinkResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x6e3..=0x6e4)
    }
    pub(super) fn new(
        links: model::Links,
        sources: &[SourceRange],
        items: &items::Items,
        file_sha256: String,
    ) -> Result<Self, String> {
        links.validate_items(items)?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::END_OFFSET - model::SOURCE_OFFSET
            || sources[0].role != "item-links/eur-fixed"
            || sources[0].sha256 != ffta_rom::digest(&links.source_bytes())
        {
            return Err("Item-link source/file binding mismatch".into());
        }
        Ok(Self {
            links,
            file_sha256,
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
        let mut counts = [0u64; 6];
        let bytes = std::array::from_fn(|i| {
            let at = (index + i).checked_sub(model::SOURCE_OFFSET as usize);
            if let Some(at) =
                at.filter(|&at| at < (model::END_OFFSET - model::SOURCE_OFFSET) as usize)
            {
                let offset = at % model::RECORD_BYTES;
                counts[1] += 1;
                counts[match offset {
                    0 => 2,
                    1 => 5,
                    n if n & 1 == 0 => 3,
                    _ => 4,
                }] += 1;
                self.links.records[at / model::RECORD_BYTES]
                    .byte(offset)
                    .unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |b| b[i])
            }
        });
        if counts[1] == 0 {
            return prior;
        }
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ItemLinkStats {
        let [bus_reads, source_bytes, count_bytes, selector_bytes, value_bytes, opaque_bytes] =
            self.counters.each_ref().map(|c| c.load(Ordering::Relaxed));
        ItemLinkStats {
            bus_reads,
            source_bytes,
            count_bytes,
            selector_bytes,
            value_bytes,
            opaque_bytes,
        }
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.links.records.capacity() * std::mem::size_of::<model::Record>()
            + self.file_sha256.capacity()
    }
}
