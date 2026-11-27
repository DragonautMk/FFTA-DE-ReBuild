//! Complete physical EUR compact OBJ banks and their bounded descriptors.
use super::{
    script39_registry_poses::{half, word, PriorSources},
    Resource,
};
use std::{collections::BTreeMap, ops::Range};

const ROM_BASE: u32 = 0x0800_0000;
const CODE_END: usize = 0x145600;
const CONSTRUCTOR: i32 = 0x0d9ba8;
const BANKS: Range<u32> = 0x6b3428..0x6cd966;
const WIDE_BANK: u32 = 0x6bb838;
const DESCRIPTOR_BYTES: u32 = 24;
pub(super) const PACKED_BYTES: u64 = 93_964;

fn zero_bytes(rom: &[u8], range: Range<u32>) -> bool {
    rom.get(range.start as usize..range.end as usize)
        .is_some_and(|bytes| bytes.iter().all(|&byte| byte == 0))
}

/// Check every offset against consecutive counted records. Empty lists
/// retain their two-byte count, and alignment belongs outside the resource.
fn records(
    rom: &[u8],
    root: u32,
    limit: u32,
    width: u32,
    stride: u32,
    ranges: &mut Vec<Range<u32>>,
) -> Result<(u32, Vec<(u32, u32)>), String> {
    let relative = |at| {
        if width == 2 {
            half(rom, at).map(u32::from)
        } else {
            word(rom, at)
        }
    };
    let bytes = relative(root)?;
    if bytes == 0 || bytes > 4096 || bytes % width != 0 || root + bytes >= limit {
        return Err(format!("Invalid compact OBJ directory at {root:#x}"));
    }
    let mut end = root + bytes;
    ranges.push(root..end);
    let mut entries = Vec::new();
    for index in (root..end).step_by(width as usize) {
        let at = (end + width - 1) & !(width - 1);
        if relative(index)? != at - root || !zero_bytes(rom, end..at) {
            return Err(format!("Invalid compact OBJ entry at {index:#x}"));
        }
        let count = u32::from(half(rom, at)?);
        end = at + 2 + stride * count;
        if end > limit {
            return Err(format!("Compact OBJ record exceeds its bank at {at:#x}"));
        }
        ranges.push(at..end);
        entries.push((at, count));
    }
    Ok((end, entries))
}

fn physical_banks(
    rom: &[u8],
    ranges: &mut Vec<Range<u32>>,
) -> Result<BTreeMap<u32, usize>, String> {
    let mut banks = BTreeMap::new();
    let mut root = BANKS.start;
    loop {
        let flags = if root == WIDE_BANK { 4 } else { 12 };
        let sequence_index = root + u32::from(half(rom, root + 2)?);
        if half(rom, root)? != flags || !(root + 5..BANKS.end).contains(&sequence_index) {
            return Err(format!("Invalid physical compact OBJ bank at {root:#x}"));
        }
        ranges.push(root..root + 4);
        let (list_end, lists) = records(rom, root + 4, sequence_index, 2, 6, ranges)?;
        if sequence_index - list_end > 2 || !zero_bytes(rom, list_end..sequence_index) {
            return Err(format!(
                "Invalid compact OBJ list alignment at {list_end:#x}"
            ));
        }
        let width = if flags & 8 == 0 { 4 } else { 2 };
        let (end, sequences) = records(rom, sequence_index, BANKS.end, width, width, ranges)?;
        for &(at, count) in &sequences {
            for frame in (at + 2..at + 2 + width * count).step_by(width as usize) {
                let list = if width == 2 {
                    u16::from(rom[frame as usize])
                } else {
                    half(rom, frame)?
                };
                if usize::from(list) >= lists.len() {
                    return Err(format!("Bad compact OBJ frame list at {frame:#x}"));
                }
            }
        }
        banks.insert(root, sequences.len());
        if end == BANKS.end {
            break;
        }
        root = (end + 3) & !3;
        if root >= BANKS.end || !zero_bytes(rom, end..root) {
            return Err(format!("Invalid compact OBJ bank alignment at {end:#x}"));
        }
    }
    if banks.len() != 199 {
        return Err("Incomplete physical compact OBJ region".into());
    }
    Ok(banks)
}

fn stack_source(code: &[u16], call: usize, slot: u16, load: u16) -> Option<usize> {
    let store = (call.saturating_sub(29)..call)
        .rev()
        .find(|&at| code[at] & 0xf8ff == (0x9000 | slot))?;
    let at = store.checked_sub(1)?;
    (code[at] & 0xf800 == load && (code[at] >> 8) & 7 == (code[store] >> 8) & 7).then_some(at)
}

fn direct_descriptors(
    rom: &[u8],
    descriptors: &mut Vec<Range<u32>>,
    literals: &mut Vec<Range<u32>>,
) -> Result<(), String> {
    let code: Vec<_> = rom[..CODE_END]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let mut scanned = 0;
    let mut selected = 0;
    for (index, pair) in code.windows(2).enumerate() {
        if pair[0] & 0xf800 != 0xf000 || pair[1] & 0xf800 != 0xf800 {
            continue;
        }
        let displacement = i32::from(pair[0] & 0x07ff) << 12 | i32::from(pair[1] & 0x07ff) << 1;
        let signed = (displacement << 9) >> 9;
        if (index as i32) * 2 + 4 + signed != CONSTRUCTOR {
            continue;
        }
        scanned += 1;
        let (Some(count), Some(root)) = (
            stack_source(&code, index, 0, 0x2000),
            stack_source(&code, index, 2, 0x4800),
        ) else {
            continue;
        };
        let block = &code[count.min(root)..index];
        if block.iter().any(|&op| {
            matches!(op & 0xf800, 0xe000 | 0xf000 | 0xf800)
                || op & 0xf000 == 0xd000
                || matches!(op & 0xff00, 0x4700 | 0xb000 | 0xb500 | 0xbd00)
        }) {
            return Err(format!(
                "Unexpected compact OBJ caller branch at {:#x}",
                index * 2
            ));
        }
        let literal = ((root as u32 * 2 + 4) & !3) + 4 * u32::from(code[root] & 0xff);
        let start = word(rom, literal)?
            .checked_sub(ROM_BASE)
            .ok_or("Invalid compact OBJ descriptor pointer")?;
        let count = u32::from(code[count] & 0xff);
        if !(0x39a700..0x3acc00).contains(&start) || !(1..=8).contains(&count) {
            return Err(format!(
                "Invalid compact OBJ call arguments at {:#x}",
                index * 2
            ));
        }
        descriptors.push(start..start + DESCRIPTOR_BYTES * count);
        literals.push(literal..literal + 4);
        selected += 1;
    }
    if (scanned, selected) != (258, 222) {
        return Err("Incomplete compact OBJ direct caller census".into());
    }
    Ok(())
}

fn merge(mut ranges: Vec<Range<u32>>) -> Vec<Range<u32>> {
    ranges.sort_unstable_by_key(|range| (range.start, range.end));
    let mut merged: Vec<Range<u32>> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut() {
            if range.start <= last.end {
                last.end = last.end.max(range.end);
                continue;
            }
        }
        merged.push(range);
    }
    merged
}

/// Preserve earlier owners while publishing all the remaining E613 ranges.
pub(super) fn uncovered_ranges(
    rom: &[u8],
    resources: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let mut bank_ranges = Vec::new();
    let banks = physical_banks(rom, &mut bank_ranges)?;
    let mut descriptors = Vec::new();
    let mut literals = vec![
        0x0eb9b8..0x0eb9c0,
        0x0edae4..0x0edae8,
        0x0edc5c..0x0edc60,
        0x123f8c..0x123f90,
    ];
    direct_descriptors(rom, &mut descriptors, &mut literals)?;
    if rom.get(0x39cc78..0x39cc7b) != Some(&[2, 2, 2]) {
        return Err("Unexpected Firaga descriptor count table".into());
    }
    for (table, count) in [(0x39cc6c, 2), (0x39d7cc, 2), (0x39d7d8, 1)] {
        for slot in 0..3 {
            let root = word(rom, table + slot * 4)?
                .checked_sub(ROM_BASE)
                .ok_or("Invalid compact OBJ table pointer")?;
            descriptors.push(root..root + DESCRIPTOR_BYTES * count);
        }
    }
    let descriptors = merge(descriptors);
    let mut rows = 0;
    for range in &descriptors {
        if (range.end - range.start) % DESCRIPTOR_BYTES != 0 {
            return Err("Unaligned compact OBJ descriptor range".into());
        }
        for at in range.clone().step_by(DESCRIPTOR_BYTES as usize) {
            let bank = word(rom, at)?.wrapping_sub(ROM_BASE);
            let selector = usize::from(half(rom, at + 4)?);
            if !banks.get(&bank).is_some_and(|&count| selector < count) {
                return Err(format!("Invalid descriptor selection at {at:#x}"));
            }
            rows += 1;
        }
    }
    if rows != 548 {
        return Err("Incomplete compact OBJ descriptors".into());
    }
    let prior = PriorSources::new(resources)?;
    let mut ranges = Vec::new();
    for (group, role) in [
        (merge(bank_ranges), "banks"),
        (descriptors, "descriptors"),
        (
            vec![0x39cc6c..0x39cc7b, 0x39d7cc..0x39d7e4],
            "selector_tables",
        ),
        (merge(literals), "reader_literals"),
    ] {
        for range in group {
            ranges.extend(
                prior
                    .uncovered(range)
                    .into_iter()
                    .map(|missing| (missing, role)),
            );
        }
    }
    ranges.sort_unstable_by_key(|(range, _)| range.start);
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
        || ranges
            .iter()
            .map(|(range, _)| u64::from(range.end - range.start))
            .sum::<u64>()
            != PACKED_BYTES
    {
        return Err("Unexpected physical compact OBJ source ownership".into());
    }
    Ok(ranges)
}
