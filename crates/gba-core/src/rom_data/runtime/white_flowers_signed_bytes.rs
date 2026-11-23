//! Serve a bounded signed-byte projection and leave other reads to prior owners.
use super::*;
use crate::rom_data::white_flowers_signed_bytes as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WhiteFlowersSignedBytesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub values_mask: u64,
}
pub(super) struct WhiteFlowersSignedBytesResources {
    data: model::WhiteFlowersSignedBytes,
    reads: AtomicU64,
    source_bytes: AtomicU64,
    values: AtomicU64,
}
impl WhiteFlowersSignedBytesResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| page == 0x397)
    }
    pub(super) fn new(data: model::WhiteFlowersSignedBytes, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len() || sources[0].role != model::SOURCE_ROLE
            || sources[0].sha256 != ffta_rom::digest(&bytes)
        {
            return Err("White Flowers signed-byte source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), source_bytes: AtomicU64::new(0), values: AtomicU64::new(0) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let (mut count, mut mask) = (0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => { count += 1; mask |= 1u64 << entry; byte }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(count, Ordering::Relaxed);
        self.values.fetch_or(mask, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> WhiteFlowersSignedBytesStats {
        WhiteFlowersSignedBytesStats { bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed), values_mask: self.values.load(Ordering::Relaxed) }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.values.capacity() * std::mem::size_of::<i8>()
    }
}
