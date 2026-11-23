//! Reconstruct authenticated rows without restricting guest indexing.
use super::super::{observed_tilemap_row as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapRowStats { pub bus_reads: u64, pub source_bytes: u64, pub transfer_source_bytes: u64, pub list_source_bytes: u64, pub direct_tilemap_row_bytes: u64, pub interleaved_lz_row_bytes: u64, pub soil_row_mask: u8, pub soil_other_row_mask: u8, pub firaga_transfer_row_bytes: u64, pub star_cross_row_mask: u8 }
pub(super) struct ObservedTilemapRowResources { data: model::ObservedTilemapRow, reads: AtomicU64, bytes: AtomicU64, transfer_bytes: AtomicU64, list_bytes: AtomicU64, direct_bytes: AtomicU64, interleaved_bytes: AtomicU64, soil_rows: AtomicU64, firaga_bytes: AtomicU64, star_cross_rows: AtomicU64 }

impl ObservedTilemapRowResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| matches!(page, 0x399 | 0x39a | 0x39d | 0x39e | 0x3a4))
    }
    pub(super) fn new(data: model::ObservedTilemapRow, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans();
        if sources.len() != spans.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len()
                || source.role != *role || source.sha256 != ffta_rom::digest(bytes)
        }) {
            return Err("Observed tilemap row source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0), transfer_bytes: AtomicU64::new(0), list_bytes: AtomicU64::new(0), direct_bytes: AtomicU64::new(0), interleaved_bytes: AtomicU64::new(0), soil_rows: AtomicU64::new(0), firaga_bytes: AtomicU64::new(0), star_cross_rows: AtomicU64::new(0) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let primary = self.data.source_bytes();
        let cells: [_; N] = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(value) = at.checked_sub(self.data.source_offset as usize).and_then(|offset| primary.get(offset).copied()) {
                return Some((value, 0));
            }
            self.data.additional_rows.iter().map(|row| (row, 0))
                .chain(self.data.transfer_rows.iter().map(|row| (row, 1)))
                .chain(std::iter::once((&self.data.list_row, 2)))
                .chain(self.data.direct_tilemap_rows.iter().flatten().map(|row| (row, 3)))
                .chain(self.data.interleaved_lz_rows.iter().flatten().map(|row| (row, 4)))
                .chain(self.data.soil_transfer_rows.iter().flatten().enumerate().map(|(i, row)| (row, 5 + i)))
                .chain(self.data.soil_other_rows.iter().flatten().enumerate().map(|(i, row)| (row, 9 + i)))
                .chain(self.data.firaga_transfer_row.iter().map(|row| (row, 11)))
                .chain(self.data.star_cross_rows.iter().flatten().enumerate().map(|(i, row)| (row, 12 + i))).find_map(|(row, kind)| {
                at.checked_sub(row.source_offset as usize).and_then(|offset| row.source_bytes().get(offset).copied()).map(|value| (value, kind))
            })
        });
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].map(|(value, _)| value).unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        self.transfer_bytes.fetch_add(cells.iter().filter(|cell| matches!(cell, Some((_, 1)))).count() as u64, Ordering::Relaxed);
        self.list_bytes.fetch_add(cells.iter().filter(|cell| matches!(cell, Some((_, 2)))).count() as u64, Ordering::Relaxed);
        self.direct_bytes.fetch_add(cells.iter().filter(|cell| matches!(cell, Some((_, 3)))).count() as u64, Ordering::Relaxed);
        self.interleaved_bytes.fetch_add(cells.iter().filter(|cell| matches!(cell, Some((_, 4)))).count() as u64, Ordering::Relaxed);
        self.firaga_bytes.fetch_add(cells.iter().filter(|cell| matches!(cell, Some((_, 11)))).count() as u64, Ordering::Relaxed);
        let soil_mask = cells.iter().flatten().fold(0, |mask, (_, kind)| mask | if (5..11).contains(kind) { 1 << (*kind - 5) } else { 0 });
        self.soil_rows.fetch_or(soil_mask, Ordering::Relaxed);
        let star_mask = cells.iter().flatten().fold(0, |mask, (_, kind)| mask | if (12..16).contains(kind) { 1 << (*kind - 12) } else { 0 });
        self.star_cross_rows.fetch_or(star_mask, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> ObservedTilemapRowStats {
        let soil_rows = self.soil_rows.load(Ordering::Relaxed) as u8;
        ObservedTilemapRowStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed), transfer_source_bytes: self.transfer_bytes.load(Ordering::Relaxed), list_source_bytes: self.list_bytes.load(Ordering::Relaxed), direct_tilemap_row_bytes: self.direct_bytes.load(Ordering::Relaxed), interleaved_lz_row_bytes: self.interleaved_bytes.load(Ordering::Relaxed), soil_row_mask: soil_rows & 0xf, soil_other_row_mask: soil_rows >> 4, firaga_transfer_row_bytes: self.firaga_bytes.load(Ordering::Relaxed), star_cross_row_mask: self.star_cross_rows.load(Ordering::Relaxed) as u8 }
    }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
