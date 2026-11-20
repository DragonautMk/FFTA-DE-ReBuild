//! Serve typed DMA register address bytes; the guest still performs DMA accesses.
use super::*;
use crate::rom_data::dma_register_directories as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DmaRegisterDirectoriesStats {
    pub bus_reads: u64,
    pub directory_bytes: [u64; 2],
}

pub(super) struct DmaRegisterDirectoriesResources {
    data: model::DmaRegisterDirectories,
    reads: AtomicU64,
    bytes: [AtomicU64; 2],
}
impl DmaRegisterDirectoriesResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x370 }
    pub(super) fn new(data: model::DmaRegisterDirectories, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if sources.len() != spans.len() || sources.iter().zip(spans).any(|(source, (offset, bytes, role))|
            source.offset != offset || source.length as usize != bytes.len() || source.role != role || source.sha256 != ffta_rom::digest(&bytes)) {
            return Err("DMA register directory source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let mut counts = [0u64; 2];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, table)) => { counts[table] += 1; byte }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.bytes.iter().zip(counts) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }
    pub(super) fn stats(&self) -> DmaRegisterDirectoriesStats {
        DmaRegisterDirectoriesStats { bus_reads: self.reads.load(Ordering::Relaxed), directory_bytes: self.bytes.each_ref().map(|v| v.load(Ordering::Relaxed)) }
    }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
