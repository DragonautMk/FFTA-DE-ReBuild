//! One immutable typed owner for the witnessed sparse compact catalog.
use super::*;
use crate::rom_data::compact_obj_catalog as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompactObjCatalogStats { pub bus_reads: u64, pub source_bytes: u64 }

pub(super) struct CompactObjCatalogResources {
    data: model::CompactObjCatalog,
    counters: [AtomicU64; 2],
}
impl CompactObjCatalogResources {
    pub(super) fn source_page(index: usize) -> bool {
        model::SOURCE_SPANS.iter().any(|&(start, end)| (start as usize >> 12..=(end as usize - 1) >> 12).contains(&(index >> 12)))
    }
    pub(super) fn new(data: model::CompactObjCatalog, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() { return Err("Compact catalog source binding count mismatch".into()); }
        for ((at, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *at || source.length as usize != bytes.len() || source.role != *role || source.sha256 != ffta_rom::digest(bytes) {
                return Err("Compact catalog source binding mismatch".into());
            }
        }
        Ok(Self { data, counters: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        if !model::SOURCE_SPANS.iter().any(|&(start, end)| index < end as usize && index.saturating_add(N) > start as usize) { return prior; }
        let mut source_bytes = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(byte) = self.data.byte_at(at as u32) { source_bytes += 1; byte }
            else { prior.as_ref().map_or_else(|| rom[at], |value| value[i]) }
        });
        self.counters[0].fetch_add(1, Ordering::Relaxed);
        self.counters[1].fetch_add(source_bytes, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> CompactObjCatalogStats {
        CompactObjCatalogStats { bus_reads: self.counters[0].load(Ordering::Relaxed), source_bytes: self.counters[1].load(Ordering::Relaxed) }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.banks.capacity() * std::mem::size_of::<model::Bank>()
            + self.data.banks.iter().map(|bank| bank.lists.capacity() * std::mem::size_of::<model::IndexedList>()
                + bank.lists.iter().map(|entry| entry.list.objects.capacity() * std::mem::size_of::<crate::rom_data::obj_lists::Object>()).sum::<usize>()
                + bank.sequences.capacity() * std::mem::size_of::<model::IndexedSequence>()
                + bank.sequences.iter().map(|entry| entry.sequence.frames.capacity() * std::mem::size_of::<crate::rom_data::compact_obj_animations::Frame>()).sum::<usize>()
            ).sum::<usize>()
    }
}
