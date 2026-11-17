//! Serve typed geometry bytes while original guest readers retain control.
use super::super::{tilemap_geometry as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilemapGeometryStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub groups_used: u64,
}
pub(super) struct TilemapGeometryResources {
    data: model::TilemapGeometry,
    reads: AtomicU64,
    bytes: AtomicU64,
    used: AtomicU64,
}
impl TilemapGeometryResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x371 }
    pub(super) fn new(data: model::TilemapGeometry, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        let roles = ["constructor-tilemap-bytes", "constructor-width", "constructor-height", "state-tilemap-bytes", "state-width", "state-height"];
        if sources.len() != roles.len() || sources.iter().enumerate().any(|(i, s)|
            s.offset != model::SOURCE + (i * 8) as u32 || s.length != 8 || s.role != roles[i]
                || s.sha256 != ffta_rom::digest(&bytes[i * 8..i * 8 + 8])) {
            return Err("Tilemap geometry typed source binding mismatch".into());
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
    pub(super) fn stats(&self) -> TilemapGeometryStats {
        TilemapGeometryStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed), groups_used: u64::from(self.used.load(Ordering::Relaxed).count_ones()) }
    }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
