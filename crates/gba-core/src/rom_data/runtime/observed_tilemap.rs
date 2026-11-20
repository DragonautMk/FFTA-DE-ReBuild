//! Reconstruct original tilemap runs and selected halfword rectangles.
use super::super::{observed_tilemap as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapStats { pub bus_reads: u64, pub source_bytes: u64, pub rectangle_bytes: [u64; 2] }

pub(super) struct ObservedTilemapResources {
    data: model::ObservedTilemap,
    reads: AtomicU64,
    bytes: AtomicU64,
    rectangle_bytes: [AtomicU64; 2],
}

impl ObservedTilemapResources {
    pub(super) fn source_page(index: usize) -> bool { matches!(index >> 12, 0xae7 | 0xaf2) }

    pub(super) fn new(data: model::ObservedTilemap, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans();
        if sources.len() != spans.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len()
                || source.role != *role || source.sha256 != ffta_rom::digest(bytes)
        }) { return Err("Observed tilemap source binding mismatch".into()); }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0), rectangle_bytes: std::array::from_fn(|_| AtomicU64::new(0)) })
    }

    fn byte(&self, index: usize) -> Option<(u8, Option<usize>)> {
        if let Some(n) = index.checked_sub(self.data.source_offset as usize) {
            let value = match n {
                0 => Some(self.data.width), 1 => Some(self.data.height),
                2..=3 => Some(self.data.opaque_header.to_le_bytes()[n - 2]),
                4 => Some(self.data.mode), 5 => Some(self.data.flags),
                6..model::SOURCE_LENGTH => {
                    let run = &self.data.runs[(n - 6) / 4]; let at = (n - 6) % 4;
                    Some(if at < 2 { run.control.to_le_bytes()[at] } else { run.value.to_le_bytes()[at - 2] })
                }
                _ => None,
            };
            if let Some(value) = value { return Some((value, None)); }
        }
        self.data.rectangles.iter().enumerate().find_map(|(index_in_data, rectangle)| {
            let at = index.checked_sub(rectangle.source_offset as usize)?;
            let value = match at {
                0 => Some(rectangle.height), 1 => Some(rectangle.width),
                _ => rectangle.cells.get((at - 2) / 2).map(|cell| cell.to_le_bytes()[(at - 2) % 2]),
            }?;
            Some((value, Some(index_in_data)))
        })
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].map(|(value, _)| value).unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        let mut rectangles = [0u64; 2];
        for (_, index) in cells.iter().flatten() { if let Some(index) = index { rectangles[*index] += 1; } }
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        for (counter, count) in self.rectangle_bytes.iter().zip(rectangles) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }

    pub(super) fn stats(&self) -> ObservedTilemapStats {
        ObservedTilemapStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed),
            rectangle_bytes: self.rectangle_bytes.each_ref().map(|counter| counter.load(Ordering::Relaxed)) }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.rectangles.iter().map(|rectangle| rectangle.cells.capacity() * std::mem::size_of::<u16>()).sum::<usize>()
    }
}
