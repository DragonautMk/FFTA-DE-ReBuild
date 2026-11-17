//! Typed range table bytes; guest scanning and writes remain original.
use super::super::{range_lookup as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RangeLookupStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub records_used: u64,
}
pub(super) struct RangeLookupResources {
    data: model::RangeLookup,
    reads: AtomicU64,
    bytes: AtomicU64,
    used: AtomicU64,
}
impl RangeLookupResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x371 }
    pub(super) fn new(data: model::RangeLookup, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE
            || sources[0].length as usize != bytes.len()
            || sources[0].role != "bounded-range-lookup"
            || sources[0].sha256 != ffta_rom::digest(&bytes) {
            return Err("Range lookup typed source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0), used: AtomicU64::new(0) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        if !Self::source_page(index) { return prior; }
        let values: [_; N] = std::array::from_fn(|i| u32::try_from(index + i).ok().and_then(|at| self.data.byte_at(at)));
        if values.iter().all(Option::is_none) { return prior; }
        let mut count = 0;
        let mut used = 0;
        let result = std::array::from_fn(|i| match values[i] {
            Some(value) => { count += 1; used |= 1 << ((index + i - model::SOURCE as usize) / 8); value }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count, Ordering::Relaxed);
        self.used.fetch_or(used, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> RangeLookupStats {
        RangeLookupStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed), records_used: u64::from(self.used.load(Ordering::Relaxed).count_ones()) }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
