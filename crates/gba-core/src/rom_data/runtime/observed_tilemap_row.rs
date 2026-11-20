//! Reconstruct authenticated rows without restricting guest indexing.
use super::super::{observed_tilemap_row as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapRowStats { pub bus_reads: u64, pub source_bytes: u64, pub transfer_source_bytes: u64 }
pub(super) struct ObservedTilemapRowResources { data: model::ObservedTilemapRow, reads: AtomicU64, bytes: AtomicU64, transfer_bytes: AtomicU64 }

impl ObservedTilemapRowResources {
    pub(super) fn source_page(index: usize) -> bool { matches!(index >> 12, 0x399 | 0x3a4) }
    pub(super) fn new(data: model::ObservedTilemapRow, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans();
        if sources.len() != spans.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len()
                || source.role != *role || source.sha256 != ffta_rom::digest(bytes)
        }) {
            return Err("Observed tilemap row source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0), transfer_bytes: AtomicU64::new(0) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let primary = self.data.source_bytes();
        let cells: [_; N] = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(value) = at.checked_sub(self.data.source_offset as usize).and_then(|offset| primary.get(offset).copied()) {
                return Some((value, false));
            }
            self.data.additional_rows.iter().map(|row| (row, false))
                .chain(self.data.transfer_rows.iter().map(|row| (row, true))).find_map(|(row, transfer)| {
                at.checked_sub(row.source_offset as usize).and_then(|offset| row.source_bytes().get(offset).copied()).map(|value| (value, transfer))
            })
        });
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].map(|(value, _)| value).unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        self.transfer_bytes.fetch_add(cells.iter().filter(|cell| matches!(cell, Some((_, true)))).count() as u64, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> ObservedTilemapRowStats { ObservedTilemapRowStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed), transfer_source_bytes: self.transfer_bytes.load(Ordering::Relaxed) } }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
