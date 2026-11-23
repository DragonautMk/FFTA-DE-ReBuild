//! Serve selected halfwords and one byte while preserving all prior fallback.
use super::*;
use crate::rom_data::white_flowers_isolated_values as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WhiteFlowersIsolatedValuesStats {
    pub bus_reads: u64,
    pub row_source_bytes: u64,
    pub byte_source_bytes: u64,
    pub row_values_mask: u64,
}
pub(super) struct WhiteFlowersIsolatedValuesResources {
    data: model::WhiteFlowersIsolatedValues,
    reads: AtomicU64,
    row_bytes: AtomicU64,
    byte_bytes: AtomicU64,
    row_values: AtomicU64,
}
impl WhiteFlowersIsolatedValuesResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| matches!(page, 0x3c9 | 0x6e8))
    }
    pub(super) fn new(data: model::WhiteFlowersIsolatedValues, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len() || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
        }) { return Err("White Flowers isolated value source binding mismatch".into()); }
        Ok(Self { data, reads: AtomicU64::new(0), row_bytes: AtomicU64::new(0),
            byte_bytes: AtomicU64::new(0), row_values: AtomicU64::new(0) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let (mut row_count, mut byte_count, mut mask) = (0u64, 0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => {
                if entry == model::ROW_LENGTH { byte_count += 1; }
                else { row_count += 1; mask |= 1u64 << entry; }
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.row_bytes.fetch_add(row_count, Ordering::Relaxed);
        self.byte_bytes.fetch_add(byte_count, Ordering::Relaxed);
        self.row_values.fetch_or(mask, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> WhiteFlowersIsolatedValuesStats {
        WhiteFlowersIsolatedValuesStats { bus_reads: self.reads.load(Ordering::Relaxed),
            row_source_bytes: self.row_bytes.load(Ordering::Relaxed), byte_source_bytes: self.byte_bytes.load(Ordering::Relaxed),
            row_values_mask: self.row_values.load(Ordering::Relaxed) }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.row.capacity() * std::mem::size_of::<u16>()
    }
}
