//! Reconstruct one descriptor while the guest retains live task state.
use super::super::{observed_tilemap_descriptor as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapDescriptorStats { pub bus_reads: u64, pub source_bytes: u64 }
pub(super) struct ObservedTilemapDescriptorResources { data: model::ObservedTilemapDescriptor, reads: AtomicU64, bytes: AtomicU64 }

impl ObservedTilemapDescriptorResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x3a4 }
    pub(super) fn new(data: model::ObservedTilemapDescriptor, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let bytes = data.source_bytes();
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET || sources[0].length as usize != bytes.len()
            || sources[0].role != "tilemap-descriptor" || sources[0].sha256 != ffta_rom::digest(&bytes) {
            return Err("Observed tilemap descriptor source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let bytes = self.data.source_bytes();
        let cells: [_; N] = std::array::from_fn(|i| (index + i).checked_sub(self.data.source_offset as usize).and_then(|offset| bytes.get(offset).copied()));
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> ObservedTilemapDescriptorStats { ObservedTilemapDescriptorStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed) } }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
