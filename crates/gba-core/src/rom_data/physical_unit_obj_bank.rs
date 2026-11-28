//! Source ranges of the complete European unit OBJ sprite directory.
use super::{
    obj_lists::List,
    script39_registry_poses::{half, word, PriorSources},
    unit_obj_initial_pose::{transfer_bytes, transfer_tiles, DIRECTORY, OBJECT_BASE, PIXEL_BASE},
    Resource,
};
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    ops::Range,
};

const ROM_BASE: u32 = 0x0800_0000;
const INDEX: Range<u32> = 0x371494..DIRECTORY;
const DIRECTORY_END: u32 = 0x394c14;
const OBJECT_END: u32 = 0xa5129c;
const SEQUENCES: Range<u32> = 0xa512a0..0xae6dac;
pub(super) const PACKED_BYTES: u64 = 154_032;

fn sequence_roots(rom: &[u8]) -> Result<BTreeSet<u32>, String> {
    for (at, expected) in [
        (0x02219c, DIRECTORY),
        (0x0221b8, PIXEL_BASE),
        (0x0221c0, OBJECT_BASE),
        (0x0221d0, DIRECTORY_END),
    ] {
        if word(rom, at)? != ROM_BASE + expected {
            return Err(format!("Unit OBJ reader literal mismatch at {at:#x}"));
        }
    }

    let mut roots = BTreeSet::new();
    for at in (DIRECTORY..DIRECTORY_END).step_by(4) {
        let root = word(rom, at)?.wrapping_sub(ROM_BASE);
        if !INDEX.contains(&root) {
            return Err(format!("Unit OBJ index root outside the bank at {at:#x}"));
        }
        roots.insert(root);
    }
    if roots.len() != 245 || roots.first() != Some(&INDEX.start) {
        return Err("Incomplete unit OBJ index roots".into());
    }

    let mut sequences = BTreeSet::new();
    let mut nulls = 0;
    let bounds: Vec<_> = roots.into_iter().chain([INDEX.end]).collect();
    for pair in bounds.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        // Each group has three part slots for each physical facing.
        if (end - start) % 24 != 0 {
            return Err(format!("Incomplete unit OBJ pose row at {start:#x}"));
        }
        for at in (start..end).step_by(4) {
            let value = word(rom, at)?;
            if value == 0 {
                nulls += 1;
                continue;
            }
            let target = value.wrapping_sub(ROM_BASE);
            if !SEQUENCES.contains(&target) || target % 16 != 0 {
                return Err(format!("Invalid unit OBJ sequence pointer at {at:#x}"));
            }
            sequences.insert(target);
        }
    }
    if sequences.len() != 4_934 || nulls != 29_777 {
        return Err("Incomplete unit OBJ sequence index".into());
    }
    Ok(sequences)
}

fn object_sizes(rom: &[u8], at: u32) -> Result<(u32, u32), String> {
    if !(OBJECT_BASE..OBJECT_END).contains(&at) || at % 2 != 0 {
        return Err(format!("Unit OBJ list address out of range at {at:#x}"));
    }
    let end = at + 2 + 6 * u32::from(half(rom, at)?);
    if end > OBJECT_END {
        return Err(format!("Unit OBJ list exceeds its bank at {at:#x}"));
    }
    let list = List::decode(at, &rom[at as usize..end as usize])?;
    Ok((end, transfer_bytes(transfer_tiles(&list))))
}

fn covers_allocation(blocks: impl IntoIterator<Item = (u32, u32)>, range: Range<u32>) -> bool {
    let mut cursor = range.start;
    for (start, end) in blocks {
        if start != cursor || end <= start || end > range.end {
            return false;
        }
        cursor = end;
    }
    cursor == range.end
}

/// Follow the original directory and frame links before subtracting earlier owners.
pub(super) fn uncovered_ranges(
    rom: &[u8],
    resources: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let sequence_roots = sequence_roots(rom)?;
    let mut records: Vec<Range<u32>> = Vec::new();
    let mut objects = BTreeMap::new();
    let mut pixels = BTreeMap::new();
    let mut next_sequence = SEQUENCES.start;
    for at in sequence_roots {
        let count = u32::from(half(rom, at)?);
        let end = at + 4 + 20 * count;
        if at != next_sequence || count == 0 || end > SEQUENCES.end {
            return Err(format!("Invalid counted unit OBJ sequence at {at:#x}"));
        }
        next_sequence = end.next_multiple_of(16);
        if rom[end as usize..next_sequence as usize]
            .iter()
            .any(|&byte| byte != 0)
        {
            return Err(format!("Invalid unit OBJ sequence alignment at {end:#x}"));
        }
        if let Some(previous) = records.last_mut().filter(|span| span.end == at) {
            previous.end = end;
        } else {
            records.push(at..end);
        }

        for frame in (at + 4..end).step_by(20) {
            // Operation 1 treats the first two words as pixel and OBJ offsets.
            if rom[frame as usize + 9] != 1 {
                continue;
            }
            let object = OBJECT_BASE
                .checked_add(word(rom, frame + 4)?)
                .ok_or("Unit OBJ list offset overflow")?;
            let (_, bytes) = match objects.entry(object) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => *entry.insert(object_sizes(rom, object)?),
            };
            let relative = word(rom, frame)?;
            let pixel = PIXEL_BASE
                .checked_add(relative)
                .ok_or("Unit OBJ pixel offset overflow")?;
            let pixel_end = pixel
                .checked_add(bytes)
                .ok_or("Unit OBJ pixel extent overflow")?;
            if relative % 32 != 0
                || pixel_end > OBJECT_BASE
                || bytes == 0
                || *pixels.entry(pixel).or_insert(pixel_end) != pixel_end
            {
                return Err(format!("Invalid unit OBJ pixel block at {pixel:#x}"));
            }
        }
    }
    if records.last().map(|span| span.end) != Some(SEQUENCES.end)
        || objects.len() != 164
        || !covers_allocation(
            objects.into_iter().map(|(start, (end, _))| (start, end)),
            OBJECT_BASE..OBJECT_END,
        )
        || pixels.len() != 5_262
        || !covers_allocation(pixels, PIXEL_BASE..OBJECT_BASE)
    {
        return Err("Incomplete unit OBJ records, lists or pixel blocks".into());
    }

    let prior = PriorSources::new(resources)?;
    let ranges: Vec<_> = [
        (INDEX, "pose_index"),
        (DIRECTORY..DIRECTORY_END, "sprite_directory"),
        (PIXEL_BASE..OBJECT_BASE, "pixel_tiles"),
        (OBJECT_BASE..OBJECT_END, "object_lists"),
    ]
    .into_iter()
    .chain(records.into_iter().map(|range| (range, "pose_sequences")))
    .flat_map(|(range, role)| {
        prior
            .uncovered(range)
            .into_iter()
            .map(move |missing| (missing, role))
    })
    .collect();
    if ranges
        .iter()
        .map(|(span, _)| u64::from(span.end - span.start))
        .sum::<u64>()
        != PACKED_BYTES
    {
        return Err("Unexpected residual unit OBJ source length".into());
    }
    Ok(ranges)
}
