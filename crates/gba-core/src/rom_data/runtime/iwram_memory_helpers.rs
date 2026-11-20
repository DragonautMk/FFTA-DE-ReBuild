//! Serve typed helper-pointer bytes; original IWRAM execution remains unchanged.
use super::*;
use crate::rom_data::iwram_memory_helpers as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IwramMemoryHelpersStats {
    pub bus_reads: u64,
    pub helper_bytes: [u64; model::HELPER_COUNT],
}
pub(super) struct IwramMemoryHelpersResources {
    data: model::IwramMemoryHelpers,
    reads: AtomicU64,
    bytes: [AtomicU64; model::HELPER_COUNT],
}
impl IwramMemoryHelpersResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x370 }
    pub(super) fn new(data: model::IwramMemoryHelpers, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len() || sources[0].role != "iwram-memory-helper-vector"
            || sources[0].sha256 != ffta_rom::digest(&bytes) {
            return Err("IWRAM memory-helper source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let mut counts = [0u64; model::HELPER_COUNT];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, helper)) => { counts[helper] += 1; byte }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.bytes.iter().zip(counts) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }
    pub(super) fn stats(&self) -> IwramMemoryHelpersStats {
        IwramMemoryHelpersStats { bus_reads: self.reads.load(Ordering::Relaxed), helper_bytes: self.bytes.each_ref().map(|v| v.load(Ordering::Relaxed)) }
    }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
