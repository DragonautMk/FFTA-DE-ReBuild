//! Project immutable fields while leaving all guest index arithmetic intact.
use super::*;
use crate::rom_data::indexed_words_3959 as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndexedWords3959Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub fields_mask: u64,
    pub record_mask: [u64; 2],
}
pub(super) struct IndexedWords3959Resources {
    data: model::IndexedWords3959,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: AtomicU64,
    records: [AtomicU64; 2],
}
impl IndexedWords3959Resources {
    pub(super) fn source_page(index: usize) -> bool { matches!(index >> 12, 0x395) }
    pub(super) fn new(data: model::IndexedWords3959, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len() || sources[0].role != "indexed-words-3959a4"
            || sources[0].sha256 != ffta_rom::digest(&bytes) {
            return Err("Indexed word source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0),
            fields: AtomicU64::new(0), records: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let mut count = 0;
        let mut fields = 0u64;
        let mut records = [0u64; 2];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, record, field)) => {
                count += 1; fields |= 1u64 << field; records[record / 64] |= 1u64 << (record % 64); byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count, Ordering::Relaxed);
        self.fields.fetch_or(fields, Ordering::Relaxed);
        for (counter, mask) in self.records.iter().zip(records) { counter.fetch_or(mask, Ordering::Relaxed); }
        Some(result)
    }
    pub(super) fn stats(&self) -> IndexedWords3959Stats {
        IndexedWords3959Stats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed),
            fields_mask: self.fields.load(Ordering::Relaxed), record_mask: self.records.each_ref().map(|v| v.load(Ordering::Relaxed)) }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
