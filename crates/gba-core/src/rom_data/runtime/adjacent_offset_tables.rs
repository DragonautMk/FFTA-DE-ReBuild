//! Serve the extracted physical byte tables; guest interpretation is unchanged.
use super::*;
use crate::rom_data::adjacent_offset_tables as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdjacentOffsetTablesStats {
    pub bus_reads: u64,
    pub table_bytes: [u64; model::TABLE_COUNT],
}
pub(super) struct AdjacentOffsetTablesResources {
    data: model::AdjacentOffsetTables,
    reads: AtomicU64,
    bytes: [AtomicU64; model::TABLE_COUNT],
}
impl AdjacentOffsetTablesResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x3ad }
    pub(super) fn new(data: model::AdjacentOffsetTables, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len() || sources[0].role != "adjacent-offset-tables"
            || sources[0].sha256 != ffta_rom::digest(&bytes) {
            return Err("Adjacent offset source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let mut counts = [0u64; model::TABLE_COUNT];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, table)) => { counts[table] += 1; byte }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.bytes.iter().zip(counts) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }
    pub(super) fn stats(&self) -> AdjacentOffsetTablesStats {
        AdjacentOffsetTablesStats { bus_reads: self.reads.load(Ordering::Relaxed), table_bytes: self.bytes.each_ref().map(|v| v.load(Ordering::Relaxed)) }
    }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
