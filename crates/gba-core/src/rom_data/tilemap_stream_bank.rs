//! Source extents and indexed controls for the contiguous European tilemap bank.

use super::{
    observed_tilemap_descriptor::DirectTilemapDescriptor,
    observed_tilemap_row::AdditionalRow,
    script39_registry_poses::PriorSources,
    tilemap_transfer_directory::{
        bytes, projected_row_offsets, row_at, target, Slot, ROM_BASE, ROW_BYTES,
    },
    Resource, SourceRange,
};
use crate::ffta_rom::{self, Version};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, ops::Range};

pub const START: u32 = 0xaf1818;
pub const END: u32 = 0xaf3ea0;
// The E450 type-5 rows, including the two rows that share one source.
const BYTE_ROWS: [u32; 8] = [
    0x39c110, 0x39c840, 0x39d3e0, 0x39e010, 0x3a14e4, 0x3a1f48, 0x3a69fc, 0x3ac400,
];

#[derive(Debug)]
pub struct Stream {
    pub extent: Range<u32>,
    pub width: u8,
    pub height: u8,
    pub mode: u8,
    pub byte_values: bool,
    /// The header includes the origin pair used by the geometry readers.
    /// Neither inter-stream alignment nor intra-token padding is selected.
    pub sources: Vec<Range<u32>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedTilemapControls {
    pub slots: [IndexedSlot; 12],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedSlot {
    pointer_offset: u32,
    pointer: u32,
    row: AdditionalRow,
    descriptor: DirectTilemapDescriptor,
}

pub(super) struct IndexedField {
    pub(super) offset: u32,
    pub(super) bytes: Box<[u8]>,
    role: &'static str,
}

impl IndexedField {
    pub(super) fn source_range(&self) -> SourceRange {
        SourceRange {
            offset: self.offset,
            length: self.bytes.len() as u32,
            sha256: ffta_rom::digest(&self.bytes),
            role: self.role.into(),
        }
    }
}

struct IndexedLists {
    geometry: u32,
    rows: u32,
    counts: u32,
    expected_count: u8,
}

// E451 resolves the list pointers and counts for the two-slot groups.
const INDEXED_LISTS: [IndexedLists; 2] = [
    IndexedLists {
        geometry: 0x39bcbc,
        rows: 0x39bc90,
        counts: 0x39bc98,
        expected_count: 4,
    },
    IndexedLists {
        geometry: 0x3a1c58,
        rows: 0x3a1c4c,
        counts: 0x3a1c54,
        expected_count: 3,
    },
];

impl IndexedTilemapControls {
    pub(super) fn from_verified_rom(rom: &[u8], streams: &[Stream]) -> Result<Self, String> {
        let slots = indexed_slots(rom, streams)?
            .try_into()
            .map_err(|_| "Unexpected indexed tilemap slot count")?;
        Ok(Self { slots })
    }

    pub(super) fn uncovered_fields(
        &self,
        prior: &PriorSources,
    ) -> Result<Vec<IndexedField>, String> {
        let mut fields = Vec::new();
        for slot in &self.slots {
            let pointer = slot.pointer.to_le_bytes();
            let row = slot.row.source_bytes();
            let descriptor = slot.descriptor.source_bytes();
            for (offset, bytes, role) in [
                (slot.pointer_offset, pointer.as_slice(), "geometry-pointer"),
                (slot.row.source_offset, row.as_slice(), "transfer-row"),
                (
                    slot.descriptor.source_offset,
                    descriptor.as_slice(),
                    "transfer-descriptor",
                ),
            ] {
                if !prior.contains(offset, bytes.len())? {
                    fields.push(IndexedField {
                        offset,
                        bytes: bytes.into(),
                        role,
                    });
                }
            }
        }
        fields.sort_unstable_by_key(|field| field.offset);
        Ok(fields)
    }
}

fn indexed_slots(rom: &[u8], streams: &[Stream]) -> Result<Vec<IndexedSlot>, String> {
    let mut slots = Vec::with_capacity(12);
    for table in INDEXED_LISTS {
        for index in 0..2 {
            let row_pointer = u32::from_le_bytes(bytes(rom, table.rows + 4 * index)?);
            let [row_count] = bytes(rom, table.counts + index)?;
            if row_count != table.expected_count {
                return Err("Unexpected indexed tilemap transfer list size".into());
            }
            slots.push(link_slot(
                rom,
                streams,
                table.geometry + 4 * index,
                Slot {
                    row_pointer,
                    row_count,
                },
            )?);
        }
    }
    // E202's eight fixed pairs contain the geometry row in their second slot.
    for index in 0..8 {
        slots.push(link_slot(
            rom,
            streams,
            0x3a4d84 + 4 * index,
            Slot {
                row_pointer: ROM_BASE + 0x3a4be0 + 2 * ROW_BYTES * index,
                row_count: 2,
            },
        )?);
    }
    Ok(slots)
}

fn link_slot(
    rom: &[u8],
    streams: &[Stream],
    pointer_offset: u32,
    list: Slot,
) -> Result<IndexedSlot, String> {
    let pointer = u32::from_le_bytes(bytes(rom, pointer_offset)?);
    let root = target(pointer, 6)?;
    let index = streams
        .binary_search_by_key(&root, |stream| stream.extent.start)
        .map_err(|_| {
            format!("Indexed tilemap points outside the stream bank at {pointer_offset:#x}")
        })?;
    let row_offset = list
        .row_offsets()?
        .last()
        .ok_or("Missing indexed tilemap transfer row")?;
    let row = row_at(rom, row_offset)?;
    let source_offset = target(row.descriptor_pointer, 8)?;
    let [a, b, c, d, lo, hi, parameter8, opaque_byte] = bytes(rom, source_offset)?;
    let tilemap_pointer = u32::from_le_bytes([a, b, c, d]);
    if streams[index].byte_values || row.field0 != 4 || tilemap_pointer != pointer {
        return Err(format!(
            "Inconsistent indexed tilemap transfer at {pointer_offset:#x}"
        ));
    }
    Ok(IndexedSlot {
        pointer_offset,
        pointer,
        row,
        descriptor: DirectTilemapDescriptor {
            source_offset,
            tilemap_pointer,
            parameter16: u16::from_le_bytes([lo, hi]),
            parameter8,
            opaque_byte,
        },
    })
}

pub fn from_rom(rom: &[u8]) -> Result<Vec<Stream>, String> {
    if ffta_rom::version(rom) != Some(Version::Europe) {
        return Err("Unexpected ROM for the European tilemap bank".into());
    }
    from_verified_rom(rom)
}

pub(super) fn from_verified_rom(rom: &[u8]) -> Result<Vec<Stream>, String> {
    let rom = rom.get(..END as usize).ok_or("Truncated tilemap bank")?;
    let mut byte_roots: BTreeSet<_> = BYTE_ROWS
        .into_iter()
        .map(|row| {
            if rom[row as usize] != 5 {
                return Err("Unexpected byte tilemap row kind");
            }
            let descriptor = pointer(rom, row + 4)?;
            let root = pointer(rom, descriptor)?;
            if !(START..END).contains(&root) {
                return Err("Byte tilemap source outside the bank");
            }
            Ok(root)
        })
        .collect::<Result<_, _>>()?;
    if byte_roots.len() != 7 {
        return Err("Unexpected number of byte tilemap sources".into());
    }
    let mut at = START;
    let mut streams = Vec::new();
    while at < END {
        let stream = read_stream(rom, at, byte_roots.remove(&at))
            .map_err(|error| format!("Tilemap source at {at:#x}: {error}"))?;
        at = (stream.extent.end + 3) & !3;
        streams.push(stream);
    }
    if at != END || streams.len() != 78 || !byte_roots.is_empty() {
        return Err("Tilemap inputs do not delimit the verified bank".into());
    }
    Ok(streams)
}

pub(super) fn uncovered_ranges(
    streams: &[Stream],
    prior: &PriorSources,
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let mut ranges = Vec::new();
    for stream in streams {
        let role = if stream.byte_values {
            "byte-stream"
        } else {
            "halfword-stream"
        };
        for range in &stream.sources {
            if !prior.contains(range.start, range.len())? {
                ranges.push((range.clone(), role));
            }
        }
    }
    Ok(ranges)
}

/// Select the type-4 and type-5 controls that link to the encoded stream bank.
pub(super) fn uncovered_direct_transfer_ranges(
    rom: &[u8],
    streams: &[Stream],
    existing: &[Resource],
    origin_controls: Option<&IndexedTilemapControls>,
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let prior = PriorSources::new(existing)?;
    let mut offsets = projected_row_offsets(rom)?;
    // The E445 declared row and the alternate list verified in E458.
    offsets.extend([0x39a704, 0x39a71c]);
    let mut seen_streams = BTreeSet::new();
    let mut descriptors = BTreeSet::new();
    let mut ranges = Vec::new();
    for at in offsets {
        let [kind] = bytes(rom, at)?;
        let (length, row_role, descriptor_role) = match kind {
            4 => (8, "type4-transfer-row", "type4-transfer-descriptor"),
            5 => (6, "type5-transfer-row", "type5-transfer-descriptor"),
            _ => continue,
        };
        let row = row_at(rom, at)?;
        let descriptor = target(row.descriptor_pointer, length)?;
        let root = target(u32::from_le_bytes(bytes(rom, descriptor)?), 6)?;
        let index = streams
            .binary_search_by_key(&root, |stream| stream.extent.start)
            .map_err(|_| format!("Direct tilemap stream is outside the bank at {at:#x}"))?;
        if streams[index].byte_values != (kind == 5) {
            return Err(format!("Direct tilemap stream kind mismatch at {at:#x}"));
        }
        seen_streams.insert(index);
        descriptors.insert((descriptor, length, descriptor_role));
        ranges.push((at..at + ROW_BYTES, row_role));
    }
    if seen_streams.len() != streams.len() {
        return Err("Direct tilemap controls leave streams unreferenced".into());
    }
    if let Some(controls) = origin_controls {
        let indexed_words: BTreeSet<_> = controls
            .slots
            .iter()
            .map(|slot| slot.pointer_offset)
            .collect();
        let descriptor_words: BTreeSet<_> = descriptors.iter().map(|&(at, _, _)| at).collect();
        let mut literal_count = 0;
        // E458 classifies all aligned references in the authenticated ROM.
        // Excluding descriptor and indexed words leaves the origin literals.
        for (index, word) in rom.chunks_exact(4).enumerate() {
            let pointer = u32::from_le_bytes(word.try_into().unwrap());
            if !(ROM_BASE + START..ROM_BASE + END).contains(&pointer) {
                continue;
            }
            let at = index as u32 * 4;
            if streams
                .binary_search_by_key(&(pointer - ROM_BASE), |stream| stream.extent.start)
                .is_err()
            {
                return Err(format!(
                    "Tilemap reference at {at:#x} misses the stream start"
                ));
            }
            if descriptor_words.contains(&at) {
                continue;
            }
            if indexed_words.contains(&at) {
                if !prior.contains(at, 4)? {
                    return Err(format!("Missing indexed tilemap pointer owner at {at:#x}"));
                }
                continue;
            }
            literal_count += 1;
            ranges.push((at..at + 4, "origin-literal"));
        }
        if literal_count != 153 {
            return Err("Unexpected size of the tilemap origin literal bank".into());
        }
    }
    ranges.extend(
        descriptors
            .into_iter()
            .map(|(at, length, role)| (at..at + length, role)),
    );
    ranges.sort_unstable_by_key(|(range, _)| range.start);
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return Err("Overlapping direct tilemap transfer controls".into());
    }
    Ok(ranges
        .into_iter()
        .flat_map(|(range, role)| {
            prior
                .uncovered(range)
                .into_iter()
                .map(move |span| (span, role))
        })
        .collect())
}

fn pointer(rom: &[u8], at: u32) -> Result<u32, &'static str> {
    let bytes = rom
        .get(at as usize..at as usize + 4)
        .ok_or("Tilemap descriptor pointer lies outside the source")?;
    let address = u32::from_le_bytes(bytes.try_into().unwrap());
    address
        .checked_sub(ROM_BASE)
        .filter(|offset| offset % 4 == 0)
        .ok_or("Invalid tilemap source pointer")
}

fn read_stream(rom: &[u8], start: u32, byte_values: bool) -> Result<Stream, &'static str> {
    let header = rom
        .get(start as usize..start as usize + 6)
        .ok_or("Truncated tilemap header")?;
    let (width, height, mode) = (header[0], header[1], header[4]);
    let allowed_mode = if byte_values {
        matches!(mode, 1 | 4)
    } else {
        matches!(mode, 1 | 2 | 4..=7)
    };
    if !(1..=32).contains(&width)
        || !(1..=32).contains(&height)
        || !allowed_mode
        || header[5] & 2 == 0
    {
        return Err("Invalid tilemap header");
    }

    let value_size = if byte_values { 1 } else { 2 };
    let mut remaining = u32::from(width) * u32::from(height);
    let mut at = start + 6;
    let mut source_start = start;
    let mut sources = Vec::new();
    while remaining != 0 {
        let control = rom
            .get(at as usize..at as usize + 2)
            .ok_or("Truncated tilemap token")?;
        let control = u16::from_le_bytes([control[0], control[1]]);
        let count = u32::from(control & 0x3fff);
        if count == 0 || count > remaining {
            return Err("Tilemap token exceeds its cell bound");
        }
        let values = if control & 0xc000 == 0 { count } else { 1 };
        let bytes = values * value_size;
        at += 2 + bytes;
        if bytes & 1 != 0 {
            sources.push(source_start..at);
            at += 1;
            source_start = at;
        }
        if at > rom.len() as u32 {
            return Err("Truncated tilemap values");
        }
        remaining -= count;
    }
    if source_start < at {
        sources.push(source_start..at);
    }
    Ok(Stream {
        extent: start..at,
        width,
        height,
        mode,
        byte_values,
        sources,
    })
}
