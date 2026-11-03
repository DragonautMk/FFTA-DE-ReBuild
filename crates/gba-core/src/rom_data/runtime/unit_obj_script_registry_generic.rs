//! Immutable typed conditional registry-generic script sources; animation scheduling, command pools and VRAM stay guest work.
use super::super::{
    unit_obj_initial_pose as initial, unit_obj_script_registry_generic as model, SourceRange,
};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitObjScriptRegistryGenericStats {
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
    source_start: u32,
    kind: Kind,
}
pub(super) struct UnitObjScriptRegistryGenericResources {
    data: model::UnitObjScriptRegistryGeneric,
    segments: Vec<Segment>,
    counters: [AtomicU64; 8],
    sequences: [AtomicU64; 13],
}
impl UnitObjScriptRegistryGenericResources {
    pub(super) fn source_page(index: usize) -> bool {
        // Only byte lookup below establishes ownership. Holes in these page
        // ranges compose with previous owners and mutable raw ROM unchanged.
        matches!(
            index >> 12,
            0x371
                | 0x372
                | 0x373
                | 0x374
                | 0x375
                | 0x376
                | 0x377
                | 0x378
                | 0x379
                | 0x37a
                | 0x37b
                | 0x380
                | 0x381
                | 0x382
                | 0x386
                | 0x387
                | 0x388
                | 0x389
                | 0x38a
                | 0x38b
                | 0x38c
                | 0x38d
                | 0x38e
                | 0x38f
                | 0x390
                | 0x393
                | 0x394
                | 0x89a
                | 0x89b
                | 0x89f
                | 0x8a4
                | 0x8a8
                | 0x8a9
                | 0x8ad
                | 0x8b2
                | 0x8b6
                | 0x8b7
                | 0x8bb
                | 0x8c0
                | 0x8c4
                | 0x8c5
                | 0x8c9
                | 0x8ce
                | 0x8d2
                | 0x8d3
                | 0x8d7
                | 0x8db
                | 0x8df
                | 0x8e0
                | 0x8e4
                | 0x8e8
                | 0x8e9
                | 0x8ed
                | 0x8f1
                | 0x8f2
                | 0x8f6
                | 0x8f7
                | 0x8fb
                | 0x900
                | 0x905
                | 0x909
                | 0x90a
                | 0x90e
                | 0x913
                | 0x917
                | 0x918
                | 0x91c
                | 0x91d
                | 0x921
                | 0x925
                | 0x926
                | 0x92a
                | 0x92b
                | 0x92f
                | 0x933
                | 0x934
                | 0x938
                | 0x93c
                | 0x940
                | 0x941
                | 0x944
                | 0x945
                | 0x949
                | 0x94d
                | 0x951
                | 0x952
                | 0x956
                | 0x9c7
                | 0x9d1
                | 0x9d2
                | 0x9d7
                | 0x9de
                | 0x9e6
                | 0x9ed
                | 0x9ee
                | 0x9fc
                | 0xa04
                | 0xa52
                | 0xa54
                | 0xa55
                | 0xa57
                | 0xa59
                | 0xa5a
                | 0xa5c
                | 0xa5e
                | 0xa60
                | 0xa61
                | 0xa62
                | 0xa63
                | 0xa65
                | 0xa67
                | 0xa69
                | 0xa6a
                | 0xa6b
                | 0xa6c
                | 0xa6e
                | 0xa70
                | 0xa72
                | 0xa73
                | 0xa74
                | 0xa75
                | 0xa76
                | 0xa78
                | 0xa79
                | 0xa7a
                | 0xa7c
                | 0xa7d
                | 0xa7f
                | 0xa81
                | 0xa82
                | 0xa84
                | 0xa85
                | 0xa86
                | 0xa87
                | 0xa89
                | 0xa8a
                | 0xa8c
                | 0xa8d
                | 0xa8e
                | 0xa90
                | 0xa91
                | 0xa93
                | 0xa94
                | 0xaad
                | 0xaaf
                | 0xab0
                | 0xab2
                | 0xab3
                | 0xab4
                | 0xab5
                | 0xab7
                | 0xab8
                | 0xab9
                | 0xaba
                | 0xabb
                | 0xae4
                | 0xae5
                | 0xae6
        )
    }
    pub(super) fn new(
        data: model::UnitObjScriptRegistryGeneric,
        sources: &[SourceRange],
        initial_pose: &initial::UnitObjInitialPose,
        equipment_pose: &super::super::unit_obj_equipment::UnitObjEquipment,
        normal_pose: &super::super::unit_obj_normal_battle::UnitObjNormalBattle,
        alternate_pose: &super::super::unit_obj_alternate_battle::UnitObjAlternateBattle,
        low_alternate_pose: &super::super::unit_obj_script_special_alternate::UnitObjScriptSpecialAlternate,
        known_special_pose: &super::super::unit_obj_script_registry_special::UnitObjScriptRegistrySpecial,
    ) -> Result<Self, String> {
        let spans = data.owned_spans(
            initial_pose,
            equipment_pose,
            normal_pose,
            alternate_pose,
            low_alternate_pose,
            known_special_pose,
        )?;
        if spans.len() != sources.len() {
            return Err("Registry-generic script OBJ source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Registry-generic script OBJ typed source binding mismatch".into());
            }
        }
        let mut segments = Vec::new();
        for (i, root) in data.roots.iter().enumerate() {
            segments.push(Segment {
                start: initial::DIRECTORY + u32::from(root.sprite_id) * 4,
                length: 4,
                source_start: 0,
                kind: Kind::Directory(i),
            });
        }
        for (i, b) in data.bindings.iter().enumerate() {
            segments.push(Segment {
                start: b.index_source,
                length: 4,
                source_start: 0,
                kind: Kind::Index(i),
            });
        }
        for (i, s) in data.sequences.iter().enumerate() {
            segments.push(Segment {
                start: s.source_offset,
                length: 4 + s.frames.len() * 20,
                source_start: 0,
                kind: Kind::Sequence(i),
            });
        }
        for (i, s) in data.objects.iter().enumerate() {
            segments.push(Segment {
                start: s.source_offset,
                length: s.source_len(),
                source_start: 0,
                kind: Kind::Objects(i),
            });
        }
        for (i, s) in data.pixels.iter().enumerate() {
            segments.push(Segment {
                start: s.source_offset,
                length: s.tiles.len() * 32,
                source_start: 0,
                kind: Kind::Pixels(i),
            });
        }
        let mut owned = Vec::new();
        for segment in segments {
            let end = segment.start + segment.length as u32;
            let first = sources.partition_point(|s| s.offset + s.length <= segment.start);
            for source in sources.iter().skip(first).take_while(|s| s.offset < end) {
                let start = segment.start.max(source.offset);
                let stop = end.min(source.offset + source.length);
                owned.push(Segment {
                    start,
                    length: (stop - start) as usize,
                    source_start: segment.start,
                    kind: segment.kind,
                });
            }
        }
        let mut segments = owned;
        segments.sort_by_key(|s| s.start);
        // Different logical selectors can name the same physical index cell.
        // source_spans already verified their exact same-role bytes.
        segments.dedup_by_key(|s| (s.start, s.length));
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
        if (at - s.start) as usize >= s.length {
            return None;
        }
        let n = (at - s.source_start) as usize;
        let (value, category) = match s.kind {
            Kind::Directory(i) => (
                (0x08000000 + self.data.roots[i].source_offset).to_le_bytes()[n],
                2,
            ),
            Kind::Index(i) => (self.data.bindings[i].target.bytes()[n], 3),
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
    pub(super) fn stats(&self) -> UnitObjScriptRegistryGenericStats {
        let [bus_reads, source_bytes, directory_bytes, index_bytes, sequence_header_bytes, frame_bytes, object_bytes, pixel_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        UnitObjScriptRegistryGenericStats {
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
            + self.data.producers.capacity() * std::mem::size_of::<model::Producer>()
            + self
                .data
                .producers
                .iter()
                .map(|p| p.script_sources.capacity() * std::mem::size_of::<u32>())
                .sum::<usize>()
            + self.data.roots.capacity() * std::mem::size_of::<model::Root>()
            + self.data.bindings.capacity() * std::mem::size_of::<model::Binding>()
            + self.data.sequences.capacity() * std::mem::size_of::<model::Sequence>()
            + self.data.objects.capacity() * std::mem::size_of::<super::super::obj_lists::List>()
            + self.data.pixels.capacity() * std::mem::size_of::<initial::PixelBlock>()
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
                .map(|s| s.tiles.capacity() * std::mem::size_of::<initial::Tile>())
                .sum::<usize>()
    }
}
