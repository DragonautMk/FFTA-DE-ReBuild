//! Reconstruct the original tilemap source view from typed runs.
use super::super::{observed_tilemap as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapStats { pub bus_reads: u64, pub source_bytes: u64 }

pub(super) struct ObservedTilemapResources {
    data: model::ObservedTilemap,
    reads: AtomicU64,
    bytes: AtomicU64,
}

impl ObservedTilemapResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0xaf2 }

    pub(super) fn new(data: model::ObservedTilemap, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let bytes = data.source_bytes();
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len() || sources[0].role != "tilemap-runs"
            || sources[0].sha256 != ffta_rom::digest(&bytes) {
            return Err("Observed tilemap source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0) })
    }

    fn byte(&self, index: usize) -> Option<u8> {
        let n = index.checked_sub(self.data.source_offset as usize)?;
        match n {
            0 => Some(self.data.width), 1 => Some(self.data.height),
            2..=3 => Some(self.data.opaque_header.to_le_bytes()[n - 2]),
            4 => Some(self.data.mode), 5 => Some(self.data.flags),
            6..model::SOURCE_LENGTH => {
                let run = &self.data.runs[(n - 6) / 4];
                let at = (n - 6) % 4;
                Some(if at < 2 { run.control.to_le_bytes()[at] } else { run.value.to_le_bytes()[at - 2] })
            }
            _ => None,
        }
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> ObservedTilemapStats { ObservedTilemapStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed) } }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
