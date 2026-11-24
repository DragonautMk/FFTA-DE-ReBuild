//! Sources of the bounded European transfer-list argument projections.
mod base_readers;

use super::{aligned_target, bytes, projected_row_offsets, target, Slot, ROW_BYTES};
use crate::rom_data::{script39_registry_poses::PriorSources, Resource};
use std::{collections::BTreeSet, ops::Range};

// E451 pointer-table literal load, count-table literal load, slot count.
// The two nine-slot tables belong to the conditional E453 projection.
const COUNTED_TABLES: &[(u32, u32, u8)] = &[
    (0x0e61c4, 0x0e61d2, 23),
    (0x0e7328, 0x0e7332, 9),
    (0x0e7340, 0x0e734a, 9),
    (0x0e777a, 0x0e776a, 3),
    (0x0e7d8c, 0x0e7d76, 2),
    (0x0e857a, 0x0e856a, 2),
    (0x0ebaee, 0x0ebade, 3),
    (0x0ebf26, 0x0ebf16, 3),
    (0x0ec366, 0x0ec356, 3),
    (0x0ed4ba, 0x0ed4a2, 2),
    (0x0f43c6, 0x0f43b6, 8),
    (0x0fbb1a, 0x0fbb08, 2),
    (0x0fe330, 0x0fe320, 2),
    (0x0feec0, 0x0feeb0, 2),
    (0x10008a, 0x100078, 2),
    (0x1020f6, 0x1020e6, 5),
    (0x102494, 0x102484, 2),
    (0x102a18, 0x102a22, 9),
    (0x1098c8, 0x1098b0, 3),
    (0x10c282, 0x10c270, 6),
    (0x10d2b8, 0x10d2a6, 2),
];

// These callers encode the row count in the original setup instructions.
const FIXED_COUNT_TABLES: &[(u32, u8, u8)] = &[(0x0f7836, 2, 3), (0x0fc9a6, 3, 2)];
const TABLE_ALIAS_READERS: &[(u32, u32)] = &[
    (0x0e7da8, 0x0e7d8c),
    (0x0ed4da, 0x0ed4ba),
    (0x1098e4, 0x1098c8),
];
const FIXED_BASE_READERS: [u32; 3] = [0x0faa74, 0x1087ec, 0x109562];

fn literal(rom: &[u8], reader: u32) -> Result<(u32, u32), String> {
    let opcode = u16::from_le_bytes(bytes(rom, reader)?);
    if opcode & 0xf800 != 0x4800 {
        return Err(format!(
            "Invalid tilemap directory literal reader at {reader:#x}"
        ));
    }
    let at = ((reader + 4) & !3) + 4 * u32::from(opcode & 0xff);
    Ok((at, u32::from_le_bytes(bytes(rom, at)?)))
}

fn list_pointer(rom: &[u8], reader: u32, rows: &BTreeSet<u32>) -> Result<u32, String> {
    let (at, pointer) = literal(rom, reader)?;
    if !rows.contains(&target(pointer, ROW_BYTES)?) {
        return Err(format!("Unverified tilemap list base at {at:#x}"));
    }
    Ok(at)
}

fn table_slots(
    rom: &[u8],
    pointer_table: u32,
    row_counts: impl IntoIterator<Item = u8>,
    rows: &BTreeSet<u32>,
    ranges: &mut Vec<(Range<u32>, &'static str)>,
) -> Result<(), String> {
    for (index, row_count) in row_counts.into_iter().enumerate() {
        let at = pointer_table + 4 * index as u32;
        let row_pointer = u32::from_le_bytes(bytes(rom, at)?);
        let slot = Slot {
            row_pointer,
            row_count,
        };
        if !slot.row_offsets()?.all(|row| rows.contains(&row)) {
            return Err(format!(
                "Tilemap directory slot at {at:#x} leaves the verified rows"
            ));
        }
        ranges.push((at..at + 4, "directory-pointer"));
    }
    Ok(())
}

pub(in crate::rom_data) fn uncovered_ranges(
    rom: &[u8],
    existing: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let rows = projected_row_offsets(rom)?;
    let mut ranges = Vec::new();
    for reader in base_readers::BASE_READERS
        .iter()
        .copied()
        .chain(FIXED_BASE_READERS)
    {
        let at = list_pointer(rom, reader, &rows)?;
        ranges.push((at..at + 4, "directory-base-literal"));
    }
    for &(pointer_reader, count_reader, slots) in COUNTED_TABLES {
        let (pointer_literal, pointer) = literal(rom, pointer_reader)?;
        let (count_literal, counts) = literal(rom, count_reader)?;
        let pointer_table = target(pointer, 4 * u32::from(slots))?;
        // The count table is a byte array that can start at an odd address.
        let count_table = aligned_target(counts, u32::from(slots), 1)?;
        let counts = rom
            .get(count_table as usize..count_table as usize + usize::from(slots))
            .ok_or("Truncated tilemap directory counts")?;
        ranges.push((
            pointer_literal..pointer_literal + 4,
            "directory-table-literal",
        ));
        ranges.push((count_literal..count_literal + 4, "directory-table-literal"));
        for at in count_table..count_table + u32::from(slots) {
            ranges.push((at..at + 1, "directory-count"));
        }
        table_slots(
            rom,
            pointer_table,
            counts.iter().copied(),
            &rows,
            &mut ranges,
        )?;
    }
    for &(reader, slots, row_count) in FIXED_COUNT_TABLES {
        let (at, pointer) = literal(rom, reader)?;
        ranges.push((at..at + 4, "directory-table-literal"));
        table_slots(
            rom,
            target(pointer, 4 * u32::from(slots))?,
            std::iter::repeat_n(row_count, usize::from(slots)),
            &rows,
            &mut ranges,
        )?;
    }
    for &(reader, canonical_reader) in TABLE_ALIAS_READERS {
        let (at, pointer) = literal(rom, reader)?;
        if pointer != literal(rom, canonical_reader)?.1 {
            return Err(format!(
                "Unexpected tilemap directory table alias at {at:#x}"
            ));
        }
        ranges.push((at..at + 4, "directory-table-literal"));
    }
    ranges.sort_unstable_by_key(|(span, _)| (span.start, span.end));
    ranges.dedup();
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return Err("Overlapping tilemap directory sources".into());
    }
    let prior = PriorSources::new(existing)?;
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
