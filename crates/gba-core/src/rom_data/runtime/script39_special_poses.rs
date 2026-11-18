//! Typed opcode-39 pose sources; original guest selection and copying remain intact.
use super::super::{script39_special_poses as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Script39SpecialPoseStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub index_bytes: u64,
    pub sequence_bytes: u64,
    pub object_bytes: u64,
    pub pixel_bytes: u64,
}
#[derive(Clone, Copy)]
enum Kind {
    Index(usize),
    Sequence(usize),
    Object(usize),
    Pixel(usize),
}
struct Segment {
    start: u32,
    length: usize,
    kind: Kind,
}
pub(super) struct Script39SpecialPoseResources {
    data: model::Script39SpecialPoses,
    segments: Vec<Segment>,
    counts: [AtomicU64; 6],
}
impl Script39SpecialPoseResources {
    pub(super) fn source_page(index: usize) -> bool {
        // Neighbor pages cover mixed-width reads straddling a source page.
        matches!(index >> 12, 0x379..=0x394 | 0x89a..=0xae8)
    }
    pub(super) fn new(
        data: model::Script39SpecialPoses,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len()
            || spans
                .iter()
                .zip(sources)
                .any(|((offset, bytes, role), source)| {
                    source.offset != *offset
                        || source.length as usize != bytes.len()
                        || source.role != *role
                        || source.sha256 != ffta_rom::digest(bytes)
                })
        {
            return Err("Opcode-39 special pose typed source binding mismatch".into());
        }
        let mut segments = Vec::with_capacity(
            data.cells.len() + data.sequences.len() + data.objects.len() + data.pixels.len(),
        );
        for (i, cell) in data.cells.iter().enumerate() {
            segments.push(Segment {
                start: cell.source_offset,
                length: 4,
                kind: Kind::Index(i),
            });
        }
        for (i, seq) in data.sequences.iter().enumerate() {
            segments.push(Segment {
                start: seq.source_offset,
                length: 4 + 20 * seq.frames.len(),
                kind: Kind::Sequence(i),
            });
        }
        for (i, object) in data.objects.iter().enumerate() {
            segments.push(Segment {
                start: object.source_offset,
                length: object.source_len(),
                kind: Kind::Object(i),
            });
        }
        for (i, pixel) in data.pixels.iter().enumerate() {
            segments.push(Segment {
                start: pixel.source_offset,
                length: 32 * pixel.tiles.len(),
                kind: Kind::Pixel(i),
            });
        }
        segments.sort_by_key(|s| s.start);
        if segments
            .windows(2)
            .any(|w| w[0].start + w[0].length as u32 > w[1].start)
        {
            return Err("Overlapping opcode-39 special pose runtime sources".into());
        }
        Ok(Self {
            data,
            segments,
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(index).ok()?;
        let slot = self
            .segments
            .partition_point(|s| s.start <= at)
            .checked_sub(1)?;
        let segment = &self.segments[slot];
        let n = (at - segment.start) as usize;
        if n >= segment.length {
            return None;
        }
        Some(match segment.kind {
            Kind::Index(i) => (self.data.cells[i].target.bytes()[n], 2),
            Kind::Sequence(i) => {
                let seq = &self.data.sequences[i];
                if n < 2 {
                    ((seq.frames.len() as u16).to_le_bytes()[n], 3)
                } else if n < 4 {
                    (seq.opaque_header.to_le_bytes()[n - 2], 3)
                } else {
                    (seq.frames[(n - 4) / 20].bytes()[(n - 4) % 20], 3)
                }
            }
            Kind::Object(i) => (self.data.objects[i].source_byte(n)?, 4),
            Kind::Pixel(i) => {
                let tile = &self.data.pixels[i].tiles[n / 32];
                let p = n % 32;
                let y = p / 4;
                let x = p % 4 * 2;
                (tile.indices[y][x] | (tile.indices[y][x + 1] << 4), 5)
            }
        })
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
        let values = std::array::from_fn(|i| match fields[i] {
            Some((value, kind)) => {
                counts[1] += 1;
                counts[kind] += 1;
                value
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        for (counter, count) in self.counts.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(values)
    }
    pub(super) fn stats(&self) -> Script39SpecialPoseStats {
        let [bus_reads, source_bytes, index_bytes, sequence_bytes, object_bytes, pixel_bytes] =
            self.counts.each_ref().map(|v| v.load(Ordering::Relaxed));
        Script39SpecialPoseStats {
            bus_reads,
            source_bytes,
            index_bytes,
            sequence_bytes,
            object_bytes,
            pixel_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.segments.capacity() * std::mem::size_of::<Segment>()
            + self.data.cells.capacity()
                * std::mem::size_of::<super::super::script39_registry_poses::IndexCell>()
            + self.data.producer_pairs.capacity()
                * std::mem::size_of::<super::super::script39_registry_poses::ProducerPair>()
            + self.data.sequences.capacity()
                * std::mem::size_of::<super::super::unit_obj_equipment::Sequence>()
            + self.data.objects.capacity() * std::mem::size_of::<super::super::obj_lists::List>()
            + self
                .data
                .objects
                .iter()
                .map(|o| {
                    o.objects.capacity() * std::mem::size_of::<super::super::obj_lists::Object>()
                })
                .sum::<usize>()
            + self.data.pixels.capacity()
                * std::mem::size_of::<super::super::unit_obj_initial_pose::PixelBlock>()
            + self
                .data
                .producer_pairs
                .iter()
                .map(|p| p.script_sources.capacity() * std::mem::size_of::<u32>())
                .sum::<usize>()
            + self
                .data
                .sequences
                .iter()
                .map(|s| {
                    s.frames.capacity()
                        * std::mem::size_of::<super::super::unit_obj_equipment::Frame>()
                })
                .sum::<usize>()
            + self
                .data
                .pixels
                .iter()
                .map(|p| {
                    p.tiles.capacity()
                        * std::mem::size_of::<super::super::unit_obj_initial_pose::Tile>()
                })
                .sum::<usize>()
    }
}
