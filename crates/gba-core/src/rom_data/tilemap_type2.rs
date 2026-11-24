//! Counted European type-2 lists and the complete rectangles they reference.
use super::{
    firaga_transfer_program::FiragaTransferProgram,
    script39_registry_poses::PriorSources,
    tilemap_transfer_directory::{
        aligned_target, bytes, projected_row_offsets, row_at, target, Slot, ROW_BYTES,
    },
    Resource,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

const LIST_HEADER_BYTES: u32 = 8;
const LIST_ENTRY_BYTES: u32 = 8;
const RECTANGLE_HEADER_BYTES: u32 = 2;

pub(super) fn uncovered_ranges(
    rom: &[u8],
    firaga: &FiragaTransferProgram,
    existing: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let prior = PriorSources::new(existing)?;
    let mut offsets = projected_row_offsets(rom)?;
    offsets.extend(
        Slot {
            row_pointer: firaga.program_pointer,
            row_count: firaga.program_count,
        }
        .row_offsets()?,
    );

    let mut ranges = Vec::new();
    let mut lists = BTreeSet::new();
    for at in offsets {
        if bytes::<1>(rom, at)?[0] == 2 {
            let row = row_at(rom, at)?;
            lists.insert(target(row.descriptor_pointer, LIST_HEADER_BYTES)?);
            ranges.push((at..at + ROW_BYTES, "transfer-row"));
        }
    }

    let mut entries = BTreeSet::new();
    for at in lists {
        let [count, _, _, _, lo, mid_lo, mid_hi, hi] = bytes(rom, at)?;
        if !(1..=7).contains(&count) {
            return Err(format!("Invalid type-2 tilemap list count at {at:#x}"));
        }
        let length = LIST_ENTRY_BYTES * u32::from(count);
        let start = target(u32::from_le_bytes([lo, mid_lo, mid_hi, hi]), length)?;
        entries.extend((start..start + length).step_by(LIST_ENTRY_BYTES as usize));
        ranges.push((at..at + LIST_HEADER_BYTES, "rectangle-list-header"));
    }

    let mut rectangles = BTreeMap::new();
    for at in entries {
        let [lo, mid_lo, mid_hi, hi, _, _, first, count] = bytes(rom, at)?;
        let root = aligned_target(
            u32::from_le_bytes([lo, mid_lo, mid_hi, hi]),
            RECTANGLE_HEADER_BYTES,
            2,
        )?;
        let [height, width] = bytes(rom, root)?;
        if !(1..=16).contains(&width)
            || !(1..=16).contains(&height)
            || count == 0
            || u16::from(first) + u16::from(count) > u16::from(height)
        {
            return Err(format!(
                "Invalid type-2 tilemap rectangle window at {at:#x}"
            ));
        }
        let end = root + RECTANGLE_HEADER_BYTES + 2 * u32::from(height) * u32::from(width);
        if end as usize > rom.len() {
            return Err(format!("Truncated type-2 tilemap rectangle at {root:#x}"));
        }
        rectangles.insert(root, end);
        ranges.push((at..at + LIST_ENTRY_BYTES, "rectangle-list-entry"));
    }
    for (root, end) in rectangles {
        let cells = root + RECTANGLE_HEADER_BYTES;
        ranges.push((root..cells, "rectangle-header"));
        ranges.push((cells..end, "rectangle-cells"));
    }
    ranges.sort_unstable_by_key(|(range, _)| range.start);
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return Err("Overlapping type-2 tilemap lists or rectangles".into());
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
