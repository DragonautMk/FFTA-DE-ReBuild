//! Immutable bus views of the tilemap streams and their indexed controls.

use super::super::{
    script39_registry_poses::PriorSources,
    tilemap_stream_bank::{self, IndexedTilemapControls},
    Resource, SourceRange,
};
use std::sync::atomic::{AtomicU64, Ordering};

struct BoundSource {
    offset: usize,
    bytes: Box<[u8]>,
    kind: SourceKind,
}

#[derive(Clone, Copy)]
enum SourceKind {
    Stream,
    IndexedControl,
}

pub(super) struct TilemapStreamBankResources {
    sources: Vec<BoundSource>,
    bytes_read: [AtomicU64; 2],
}

impl TilemapStreamBankResources {
    pub(super) fn new(
        rom: &[u8],
        existing: &[Resource],
        controls: Option<&IndexedTilemapControls>,
        stream_file: Option<(&[u8], &[SourceRange])>,
    ) -> Result<Self, String> {
        let prior = PriorSources::new(existing)?;
        let mut sources = Vec::new();
        let mut legacy_controls = None;
        if let Some((mut encoded, ranges)) = stream_file {
            for range in ranges {
                let (bytes, rest) = encoded
                    .split_at_checked(range.length as usize)
                    .ok_or("Truncated tilemap stream bank")?;
                sources.push(BoundSource {
                    offset: range.offset as usize,
                    bytes: bytes.into(),
                    kind: SourceKind::Stream,
                });
                encoded = rest;
            }
            if !encoded.is_empty() {
                return Err("Trailing bytes in tilemap stream bank".into());
            }
        } else {
            let streams = tilemap_stream_bank::from_verified_rom(rom)?;
            for (range, _) in tilemap_stream_bank::uncovered_ranges(&streams, &prior)? {
                let offset = range.start as usize;
                sources.push(BoundSource {
                    offset,
                    bytes: rom[offset..range.end as usize].into(),
                    kind: SourceKind::Stream,
                });
            }
            if controls.is_none() {
                legacy_controls = Some(IndexedTilemapControls::from_verified_rom(rom, &streams)?);
            }
        }
        let controls = controls
            .or(legacy_controls.as_ref())
            .ok_or("Missing indexed controls for the tilemap stream bank")?;
        for field in controls.uncovered_fields(&prior)? {
            let offset = field.offset as usize;
            if !Self::source_range(offset) || !Self::source_range(offset + field.bytes.len() - 1) {
                return Err(format!(
                    "Indexed tilemap control outside the source pages at {offset:#x}"
                ));
            }
            sources.push(BoundSource {
                offset,
                bytes: field.bytes,
                kind: SourceKind::IndexedControl,
            });
        }
        sources.sort_unstable_by_key(|source| source.offset);
        if sources
            .windows(2)
            .any(|pair| pair[0].offset + pair[0].bytes.len() > pair[1].offset)
        {
            return Err("Overlapping tilemap stream bank bindings".into());
        }
        Ok(Self {
            sources,
            bytes_read: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }

    pub(super) fn source_range(index: usize) -> bool {
        let last = index.saturating_add(3);
        (last >= tilemap_stream_bank::START as usize && index < tilemap_stream_bank::END as usize)
            || ((index >> 12)..=(last >> 12)).any(|page| matches!(page, 0x39b | 0x3a1 | 0x3a4))
    }

    fn byte(&self, offset: usize) -> Option<(u8, SourceKind)> {
        let at = self
            .sources
            .partition_point(|source| source.offset <= offset);
        let source = self.sources.get(at.checked_sub(1)?)?;
        Some((*source.bytes.get(offset - source.offset)?, source.kind))
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_range(index) {
            return prior;
        }
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        let mut served = [0; 2];
        for (_, kind) in cells.iter().flatten() {
            served[*kind as usize] += 1;
        }
        if served == [0; 2] {
            return prior;
        }
        for (counter, count) in self.bytes_read.iter().zip(served) {
            if count != 0 {
                counter.fetch_add(count, Ordering::Relaxed);
            }
        }
        Some(std::array::from_fn(|i| {
            cells[i].map_or_else(
                || prior.as_ref().map_or_else(|| rom[index + i], |v| v[i]),
                |(value, _)| value,
            )
        }))
    }

    pub(super) fn source_bytes_read(&self) -> u64 {
        self.bytes_read[SourceKind::Stream as usize].load(Ordering::Relaxed)
    }

    pub(super) fn indexed_control_bytes_read(&self) -> u64 {
        self.bytes_read[SourceKind::IndexedControl as usize].load(Ordering::Relaxed)
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.sources.capacity() * std::mem::size_of::<BoundSource>()
            + self
                .sources
                .iter()
                .map(|source| source.bytes.len())
                .sum::<usize>()
    }
}
