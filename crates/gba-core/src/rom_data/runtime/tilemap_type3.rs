//! BIOS stream resolution and counters for the type-3 transfer resource.
use super::{super::SourceRange, packed_tilemap_file::PackedTilemapFile};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilemapType3Stats {
    pub bus_bytes: u64,
    pub lz77_calls: u64,
    pub lz77_input_bytes: u64,
}

pub(super) struct TilemapType3Resources {
    pub(super) file: PackedTilemapFile,
    lz77_calls: AtomicU64,
    lz77_input_bytes: AtomicU64,
}

impl TilemapType3Resources {
    pub(super) fn new(file: &[u8], ranges: &[SourceRange]) -> Result<Self, String> {
        Ok(Self {
            file: PackedTilemapFile::new(file, ranges)?,
            lz77_calls: AtomicU64::new(0),
            lz77_input_bytes: AtomicU64::new(0),
        })
    }

    pub(super) fn lz77_index(&self, offset: usize) -> Option<usize> {
        let index = self.file.lz77_index(offset)?;
        self.lz77_calls.fetch_add(1, Ordering::Relaxed);
        self.lz77_input_bytes
            .fetch_add(self.file.input_bytes(index).len() as u64, Ordering::Relaxed);
        Some(index)
    }

    pub(super) fn stats(&self) -> TilemapType3Stats {
        TilemapType3Stats {
            bus_bytes: self.file.bus_bytes(),
            lz77_calls: self.lz77_calls.load(Ordering::Relaxed),
            lz77_input_bytes: self.lz77_input_bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        self.file.resident_bytes() + std::mem::size_of::<[AtomicU64; 2]>()
    }
}
