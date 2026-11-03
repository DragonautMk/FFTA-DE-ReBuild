//! Typed immutable world-node fields; original live placement/pathfinding remain guest work.
use super::super::{world_nodes as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldNodesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub coordinate_bytes: u64,
    pub path_id_bytes: u64,
    pub neighbor_bytes: u64,
    pub opaque_bytes: u64,
    pub nodes_used: u64,
}

pub(super) struct WorldNodesResources {
    data: model::WorldNodes,
    reads: AtomicU64,
    bytes: AtomicU64,
    coordinates: AtomicU64,
    paths: AtomicU64,
    neighbors: AtomicU64,
    opaque: AtomicU64,
    used: AtomicU64,
}
impl WorldNodesResources {
    pub(super) fn new(data: model::WorldNodes, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("World-node source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("World-node typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            coordinates: AtomicU64::new(0),
            paths: AtomicU64::new(0),
            neighbors: AtomicU64::new(0),
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
        if index >> 12 != 0x395 {
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
                let relative = index + i - model::SOURCE as usize;
                used |= 1 << (relative / model::RECORD_BYTES);
                let field = match relative % model::RECORD_BYTES {
                    0 | 1 => 0,
                    2 | 6 | 10 | 14 => 1,
                    3 | 7 | 11 | 15 => 2,
                    _ => 3,
                };
                counts[field] += 1;
                value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(counts.iter().sum(), Ordering::Relaxed);
        self.coordinates.fetch_add(counts[0], Ordering::Relaxed);
        self.paths.fetch_add(counts[1], Ordering::Relaxed);
        self.neighbors.fetch_add(counts[2], Ordering::Relaxed);
        self.opaque.fetch_add(counts[3], Ordering::Relaxed);
        self.used.fetch_or(used, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> WorldNodesStats {
        WorldNodesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            coordinate_bytes: self.coordinates.load(Ordering::Relaxed),
            path_id_bytes: self.paths.load(Ordering::Relaxed),
            neighbor_bytes: self.neighbors.load(Ordering::Relaxed),
            opaque_bytes: self.opaque.load(Ordering::Relaxed),
            nodes_used: u64::from(self.used.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.nodes.capacity() * std::mem::size_of::<model::Node>()
    }
}
