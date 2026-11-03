//! Immutable typed unit sources; animation scheduling, command pools and VRAM stay guest work.
use super::super::{unit_obj_initial_pose as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitObjInitialPoseStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub directory_bytes: u64,
    pub index_bytes: u64,
    pub sequence_header_bytes: u64,
    pub frame_bytes: u64,
    pub object_bytes: u64,
    pub pixel_bytes: u64,
    pub sequences_used: u64,
}
#[derive(Clone, Copy)]
enum Kind {
    Directory(usize),
    Index(usize),
    Sequence(usize),
    Objects(usize),
    Pixels(usize),
}
struct Segment {
    start: u32,
    length: usize,
    kind: Kind,
}
pub(super) struct UnitObjInitialPoseResources {
    data: model::UnitObjInitialPose,
    segments: Vec<Segment>,
    counters: [AtomicU64; 8],
    sequences: [AtomicU64; 2],
}
impl UnitObjInitialPoseResources {
    pub(super) fn source_page(index: usize) -> bool {
        // Only byte lookup below establishes ownership. Holes in these page
        // ranges compose with previous owners and mutable raw ROM unchanged.
        matches!(index >> 12, 0x371..=0x384 | 0x394 | 0x898..=0xac2)
    }
    pub(super) fn new(
        data: model::UnitObjInitialPose,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Unit OBJ source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Unit OBJ typed source binding mismatch".into());
            }
        }
        let mut segments = Vec::new();
        for (i, b) in data.bindings.iter().enumerate() {
            segments.push(Segment {
                start: model::DIRECTORY + u32::from(b.sprite_id) * 4,
                length: 4,
                kind: Kind::Directory(i),
            });
            segments.push(Segment {
                start: b.index_source,
                length: 4,
                kind: Kind::Index(i),
            });
        }
        for (i, s) in data.sequences.iter().enumerate() {
            segments.push(Segment {
                start: s.source_offset,
                length: 4 + s.frames.len() * 20,
                kind: Kind::Sequence(i),
            });
        }
        for (i, s) in data.objects.iter().enumerate() {
            segments.push(Segment {
                start: s.source_offset,
                length: s.source_len(),
                kind: Kind::Objects(i),
            });
        }
        for (i, s) in data.pixels.iter().enumerate() {
            segments.push(Segment {
                start: s.source_offset,
                length: s.tiles.len() * 32,
                kind: Kind::Pixels(i),
            });
        }
        segments.sort_by_key(|s| s.start);
        Ok(Self {
            data,
            segments,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            sequences: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(index).ok()?;
        let slot = self
            .segments
            .partition_point(|s| s.start <= at)
            .checked_sub(1)?;
        let s = &self.segments[slot];
        let n = (at - s.start) as usize;
        if n >= s.length {
            return None;
        }
        let (value, category) = match s.kind {
            Kind::Directory(i) => (
                (0x08000000 + self.data.bindings[i].index_source).to_le_bytes()[n],
                2,
            ),
            Kind::Index(i) => (
                (0x08000000 + self.data.bindings[i].sequence_source).to_le_bytes()[n],
                3,
            ),
            Kind::Sequence(i) => {
                self.sequences[i / 64].fetch_or(1 << (i % 64), Ordering::Relaxed);
                let seq = &self.data.sequences[i];
                if n < 2 {
                    ((seq.frames.len() as u16).to_le_bytes()[n], 4)
                } else if n < 4 {
                    (seq.opaque_header.to_le_bytes()[n - 2], 4)
                } else {
                    (seq.frames[(n - 4) / 20].bytes()[(n - 4) % 20], 5)
                }
            }
            Kind::Objects(i) => (self.data.objects[i].source_byte(n)?, 6),
            Kind::Pixels(i) => {
                let tile = &self.data.pixels[i].tiles[n / 32];
                let p = n % 32;
                let y = p / 4;
                let x = p % 4 * 2;
                (tile.indices[y][x] | (tile.indices[y][x + 1] << 4), 7)
            }
        };
        Some((value, category))
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
        let mut counts = [1, 0, 0, 0, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((value, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                value
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |b| b[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> UnitObjInitialPoseStats {
        let [bus_reads, source_bytes, directory_bytes, index_bytes, sequence_header_bytes, frame_bytes, object_bytes, pixel_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        UnitObjInitialPoseStats {
            bus_reads,
            source_bytes,
            directory_bytes,
            index_bytes,
            sequence_header_bytes,
            frame_bytes,
            object_bytes,
            pixel_bytes,
            sequences_used: self
                .sequences
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.segments.capacity() * std::mem::size_of::<Segment>()
            + self.data.bindings.capacity() * std::mem::size_of::<model::Binding>()
            + self.data.sequences.capacity() * std::mem::size_of::<model::Sequence>()
            + self.data.objects.capacity() * std::mem::size_of::<super::super::obj_lists::List>()
            + self.data.pixels.capacity() * std::mem::size_of::<model::PixelBlock>()
            + self
                .data
                .sequences
                .iter()
                .map(|s| s.frames.capacity() * std::mem::size_of::<model::Frame>())
                .sum::<usize>()
            + self
                .data
                .objects
                .iter()
                .map(|s| {
                    s.objects.capacity() * std::mem::size_of::<super::super::obj_lists::Object>()
                })
                .sum::<usize>()
            + self
                .data
                .pixels
                .iter()
                .map(|s| s.tiles.capacity() * std::mem::size_of::<model::Tile>())
                .sum::<usize>()
    }
}
