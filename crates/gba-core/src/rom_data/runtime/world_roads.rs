//! Immutable sampled-road source bytes; original movement/facing/pathfinding stay guest code.
use super::super::{world_roads as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldRoadsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub count_bytes: u64,
    pub pointer_bytes: u64,
    pub sample_bytes: u64,
    pub opaque_bytes: u64,
    pub paths_used: u64,
}
pub(super) struct WorldRoadsResources {
    data: model::WorldRoads,
    reads: AtomicU64,
    bytes: AtomicU64,
    counts: AtomicU64,
    pointers: AtomicU64,
    samples: AtomicU64,
    opaque: AtomicU64,
    used: AtomicU64,
}
impl WorldRoadsResources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x395..=0x397).contains(&(index >> 12))
    }
    pub(super) fn new(data: model::WorldRoads, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("World-road source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("World-road typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            counts: AtomicU64::new(0),
            pointers: AtomicU64::new(0),
            samples: AtomicU64::new(0),
            opaque: AtomicU64::new(0),
            used: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index) {
            return prior;
        }
        let fields: [_; N] = std::array::from_fn(|i| {
            u32::try_from(index + i)
                .ok()
                .and_then(|at| self.data.byte_at(at))
        });
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0; 4];
        let mut used = 0;
        let bytes = std::array::from_fn(|i| match fields[i] {
            Some(value) => {
                let at = (index + i) as u32;
                let (path, class) = if at >= model::HEADERS {
                    let relative = (at - model::HEADERS) as usize;
                    (
                        relative / model::HEADER_BYTES,
                        match relative % model::HEADER_BYTES {
                            0 => 0,
                            4..=7 => 1,
                            _ => 3,
                        },
                    )
                } else {
                    let path = self
                        .data
                        .paths
                        .iter()
                        .position(|p| {
                            let start = p.points_offset().unwrap();
                            (start..start + p.points.len() as u32 * 2).contains(&at)
                        })
                        .unwrap();
                    (path, 2)
                };
                used |= 1 << path;
                counts[class] += 1;
                value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(counts.iter().sum(), Ordering::Relaxed);
        self.counts.fetch_add(counts[0], Ordering::Relaxed);
        self.pointers.fetch_add(counts[1], Ordering::Relaxed);
        self.samples.fetch_add(counts[2], Ordering::Relaxed);
        self.opaque.fetch_add(counts[3], Ordering::Relaxed);
        self.used.fetch_or(used, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> WorldRoadsStats {
        WorldRoadsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            count_bytes: self.counts.load(Ordering::Relaxed),
            pointer_bytes: self.pointers.load(Ordering::Relaxed),
            sample_bytes: self.samples.load(Ordering::Relaxed),
            opaque_bytes: self.opaque.load(Ordering::Relaxed),
            paths_used: u64::from(self.used.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.paths.capacity() * std::mem::size_of::<model::Path>()
            + self
                .data
                .paths
                .iter()
                .map(|p| p.points.capacity() * std::mem::size_of::<model::Point>())
                .sum::<usize>()
    }
}
