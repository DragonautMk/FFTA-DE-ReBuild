//! Typed coordinate-source ownership; guest copying, map height and movement stay original.
use super::super::{script_coordinates as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptCoordinateStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub index_bytes: u64,
    pub count_bytes: u64,
    pub coordinate_bytes: u64,
    pub alignment_bytes: u64,
    pub paths_used: u64,
    pub alignment_spans_used: u64,
}
pub(super) struct ScriptCoordinateResources {
    data: model::ScriptCoordinates,
    counters: [AtomicU64; 6],
    paths: [AtomicU64; 4],
    alignment: [AtomicU64; 4],
}
impl ScriptCoordinateResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0xd68..=0xd69)
    }
    pub(super) fn new(
        data: model::ScriptCoordinates,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Coordinate source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Coordinate typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            paths: std::array::from_fn(|_| AtomicU64::new(0)),
            alignment: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        let relative = at.checked_sub(self.data.source_offset)? as usize;
        let (id, byte, category) = if relative < model::PATH_COUNT * 4 {
            let id = relative / 4;
            (
                id,
                self.data.paths[id]
                    .source_offset
                    .wrapping_sub(self.data.source_offset)
                    .to_le_bytes()[relative % 4],
                2,
            )
        } else {
            let id = self
                .data
                .paths
                .partition_point(|p| p.source_offset <= at)
                .checked_sub(1)?;
            let path = &self.data.paths[id];
            let relative = at.checked_sub(path.source_offset)? as usize;
            let byte = path.byte(relative)?;
            let category = if relative < 2 {
                3
            } else if relative < 2 + path.points.len() * 2 {
                4
            } else {
                5
            };
            (id, byte, category)
        };
        self.paths[id / 64].fetch_or(1 << (id % 64), Ordering::Relaxed);
        if category == 5 {
            self.alignment[id / 64].fetch_or(1 << (id % 64), Ordering::Relaxed);
        }
        Some((byte, category))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [1, 0, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ScriptCoordinateStats {
        let [bus_reads, source_bytes, index_bytes, count_bytes, coordinate_bytes, alignment_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        let used = |masks: &[AtomicU64]| {
            masks
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum()
        };
        ScriptCoordinateStats {
            bus_reads,
            source_bytes,
            index_bytes,
            count_bytes,
            coordinate_bytes,
            alignment_bytes,
            paths_used: used(&self.paths),
            alignment_spans_used: used(&self.alignment),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.paths.capacity() * std::mem::size_of::<model::CoordinatePath>()
            + self
                .data
                .paths
                .iter()
                .map(|p| {
                    p.points.capacity() * std::mem::size_of::<model::Point>()
                        + p.alignment_after.capacity()
                })
                .sum::<usize>()
            + self.data.script_links.capacity() * std::mem::size_of::<model::ScriptLink>()
    }
}
