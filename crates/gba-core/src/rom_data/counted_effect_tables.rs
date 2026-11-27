//! European effect tables with a halfword count and two-byte samples.
use super::script39_registry_poses::{half, word};
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 3_016;
const DIRECTORY: u32 = 0x3a7db4;
const FIRST_TABLE: u32 = 0xb4b0f4;
const TABLE_END: u32 = 0xb4bc84;
const COUNTS: [u16; 13] = [41, 126, 128, 128, 128, 128, 128, 64, 64, 240, 96, 80, 104];
const LITERALS: [Range<u32>; 13] = [
    0x0e9ee0..0x0e9ee4,
    0x0ea004..0x0ea008,
    0x0ef930..0x0ef934,
    0x0efa10..0x0efa14,
    0x0efc18..0x0efc1c,
    0x0efe08..0x0efe0c,
    0x0f0024..0x0f0028,
    0x0f6664..0x0f6668,
    0x0fd1cc..0x0fd1d0,
    0x113258..0x11325c,
    0x113398..0x11339c,
    0x114600..0x114604,
    0x11db34..0x11db3c,
];

pub(super) fn source_ranges(rom: &[u8]) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let mut ranges: Vec<_> = LITERALS
        .into_iter()
        .map(|range| (range, "reader_literals"))
        .collect();
    ranges.push((DIRECTORY..DIRECTORY + 24, "table_directory"));
    let mut start = FIRST_TABLE;
    for (index, count) in COUNTS.into_iter().enumerate() {
        if half(rom, start)? != count {
            return Err(format!("Unexpected effect table count at {start:#x}"));
        }
        if (1..=6).contains(&index)
            && word(rom, DIRECTORY + (index as u32 - 1) * 4)? != 0x0800_0000 + start
        {
            return Err(format!(
                "Unexpected effect table directory entry at {start:#x}"
            ));
        }
        let end = start + 2 + 2 * u32::from(count);
        let next = (end + 3) & !3;
        if next > TABLE_END
            || !rom
                .get(end as usize..next as usize)
                .is_some_and(|pad| pad.iter().all(|&b| b == 0))
        {
            return Err(format!("Invalid effect table alignment at {end:#x}"));
        }
        let (previous, _) = ranges.last_mut().unwrap();
        if previous.end == start {
            previous.end = end;
        } else {
            ranges.push((start..end, "counted_effect_tables"));
        }
        start = next;
    }
    if start != TABLE_END {
        return Err("Incomplete effect table bank".into());
    }
    Ok(ranges)
}
