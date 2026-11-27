//! Bounded European type-3 transfers from authenticated row projections.
use super::{
    script39_registry_poses::PriorSources,
    tilemap_transfer_directory::{
        bytes, descriptor_at, lz77_input_range, projected_row_offsets, recovered_list_sources,
        row_at, target, ROW_BYTES,
    },
    Resource,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

pub(super) const LZ77_INPUT_ROLE: &str = "lz77-input";

pub(super) fn uncovered_ranges(
    rom: &[u8],
    existing: &[Resource],
    include_recovered_lists: bool,
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let prior = PriorSources::new(existing)?;
    let mut rows = Vec::new();
    let mut descriptors = BTreeSet::new();
    let mut wrappers = BTreeSet::new();
    let mut lz77_roots = BTreeSet::new();
    let mut raw_lengths = BTreeMap::new();
    let mut offsets = projected_row_offsets(rom)?;
    if include_recovered_lists {
        for (_, slot) in recovered_list_sources(rom)? {
            offsets.extend(slot.row_offsets()?);
        }
    }
    for at in offsets {
        if bytes::<1>(rom, at)?[0] != 3 {
            continue;
        }
        let row = row_at(rom, at)?;
        let descriptor_at_offset = target(row.descriptor_pointer, 8)?;
        let wrapper = descriptor_at(rom, descriptor_at_offset)?.wrapper;
        let kind = wrapper.kind;
        let flags = wrapper.flags;
        if !(1..=4).contains(&kind) || flags > 3 {
            return Err(format!(
                "Invalid type-3 transfer wrapper at {:#x}",
                wrapper.source_offset
            ));
        }
        rows.push(at..at + ROW_BYTES);
        descriptors.insert(descriptor_at_offset);
        wrappers.insert(wrapper.source_offset);
        let start = wrapper.source_offset + 4;
        if flags & 2 != 0 {
            lz77_roots.insert(start);
        } else {
            let shift = flags & 1;
            let size = u32::from(row.field2);
            let quarter = size >> (2 + shift);
            let half = size >> (1 + shift);
            let length = [quarter, half, half + quarter, size >> shift][usize::from(kind - 1)];
            if length == 0 || length % 2 != 0 || (start + length) as usize > rom.len() {
                return Err(format!("Invalid type-3 raw input extent at {start:#x}"));
            }
            let maximum = raw_lengths.entry(start).or_insert(length);
            *maximum = (*maximum).max(length);
        }
    }

    let mut ranges: Vec<_> = rows.into_iter().map(|row| (row, "transfer-row")).collect();
    ranges.extend(
        descriptors
            .into_iter()
            .map(|at| (at..at + 8, "transfer-descriptor")),
    );
    ranges.extend(
        wrappers
            .into_iter()
            .map(|at| (at..at + 4, "stream-wrapper")),
    );
    ranges.extend(
        raw_lengths
            .into_iter()
            .map(|(at, length)| (at..at + length, "raw-input")),
    );
    for at in lz77_roots {
        ranges.push((lz77_input_range(rom, at)?, LZ77_INPUT_ROLE));
    }
    ranges.sort_unstable_by_key(|(range, _)| range.start);
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return Err("Overlapping type-3 transfer fields or inputs".into());
    }

    let mut uncovered = Vec::new();
    for (range, role) in ranges {
        if role == LZ77_INPUT_ROLE {
            if !prior.contains(range.start, range.len())? {
                uncovered.push((range, role));
            }
        } else {
            // The descriptor at 0x3abd74 already has an owner for its last word.
            uncovered.extend(prior.uncovered(range).into_iter().map(|span| (span, role)));
        }
    }
    Ok(uncovered)
}
