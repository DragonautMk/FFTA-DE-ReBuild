//! Static typed fields project original ROM bytes; property/bonus logic stays guest code.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub halfword_bytes: u64,
    pub signed_bytes: u64,
    pub other_bytes: u64,
}
pub(super) struct ItemResources {
    items: items::Items,
    file_sha256: String,
    counters: [AtomicU64; 5],
}
impl ItemResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x6e0..=0x6e3)
    }
    pub(super) fn new(
        items: items::Items,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        items.validate()?;
        if sources.len() != 1
            || sources[0].offset != items::SOURCE_OFFSET
            || sources[0].length != (items::END_OFFSET - items::SOURCE_OFFSET)
            || sources[0].role != "item-records/eur-fixed"
            || sources[0].sha256 != ffta_rom::digest(&items.source_bytes())
        {
            return Err("Item source/file binding mismatch".into());
        }
        Ok(Self {
            items,
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
        if index >= items::END_OFFSET as usize
            || index.saturating_add(N) <= items::SOURCE_OFFSET as usize
        {
            return prior;
        }
        let mut counts = [0u64; 5];
        let bytes = std::array::from_fn(|i| {
            let at = (index + i).checked_sub(items::SOURCE_OFFSET as usize);
            if let Some(at) =
                at.filter(|&at| at < (items::END_OFFSET - items::SOURCE_OFFSET) as usize)
            {
                let offset = at % 32;
                counts[1] += 1;
                counts[if offset < 8 {
                    2
                } else if (16..24).contains(&offset) {
                    3
                } else {
                    4
                }] += 1;
                self.items.records[at / 32].byte(offset).unwrap()
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
    pub(super) fn stats(&self) -> ItemStats {
        let [bus_reads, source_bytes, halfword_bytes, signed_bytes, other_bytes] =
            self.counters.each_ref().map(|c| c.load(Ordering::Relaxed));
        ItemStats {
            bus_reads,
            source_bytes,
            halfword_bytes,
            signed_bytes,
            other_bytes,
        }
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.items.records.capacity() * std::mem::size_of::<items::Record>()
            + self.file_sha256.capacity()
    }
}
