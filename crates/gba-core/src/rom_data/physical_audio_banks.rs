//! Physical M4A tone records and direct wave sources beyond the sequence catalog.
use super::{
    script39_registry_poses::{half, word, PriorSources},
    Resource,
};
use std::{collections::BTreeMap, ops::Range};

const ROM_BASE: u32 = 0x0800_0000;
// The four bank arrays are contiguous, including slots outside the sequence-derived projection.
const TONE_RECORDS: Range<u32> = 0x14d9fc..0x14f160;
const CGB_WAVES: Range<u32> = 0x14f47c..0x14f4fc;
pub(super) const PACKED_BYTES: u64 = 57_131;

/// Project the complete physical arrays from the authenticated EUR ROM,
/// leaving spans already supplied by the typed sequence-derived catalog in place.
pub(super) fn uncovered_ranges(
    rom: &[u8],
    resources: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let prior = PriorSources::new(resources)?;
    let mut pcm = BTreeMap::new();
    let mut cgb = 0u8;
    for at in TONE_RECORDS.step_by(12) {
        match rom[at as usize] {
            kind @ (0 | 8) => {
                let source = word(rom, at + 4)?
                    .checked_sub(ROM_BASE)
                    .ok_or("Invalid physical PCM pointer")?;
                let interpolated = pcm.entry(source).or_insert(false);
                *interpolated |= kind == 0;
            }
            3 => {
                let source = word(rom, at + 4)?
                    .checked_sub(ROM_BASE)
                    .ok_or("Invalid physical CGB pointer")?;
                if !CGB_WAVES.contains(&source) || source % 16 != CGB_WAVES.start % 16 {
                    return Err(format!("Unexpected physical CGB wave at {source:#x}"));
                }
                cgb |= 1 << ((source - CGB_WAVES.start) / 16);
            }
            1 | 2 | 0x40 | 0x80 => {}
            _ => return Err(format!("Unexpected physical tone kind at {at:#x}")),
        }
    }
    if pcm.len() != 285 || cgb != u8::MAX {
        return Err("Incomplete physical audio banks".into());
    }

    let mut ranges = Vec::new();
    ranges.extend(
        prior
            .uncovered(TONE_RECORDS)
            .into_iter()
            .map(|r| (r, "physical-tone-records")),
    );
    ranges.extend(
        prior
            .uncovered(CGB_WAVES)
            .into_iter()
            .map(|r| (r, "direct-cgb-waves")),
    );
    let mut previous_end = 0x150944;
    for (source, interpolates) in pcm {
        let flags = half(rom, source + 2)?;
        let frequency = word(rom, source + 4)?;
        let loop_start = word(rom, source + 8)?;
        let length = word(rom, source + 12)?;
        let end = source
            .checked_add(16 + u32::from(interpolates))
            .and_then(|start| start.checked_add(length))
            .filter(|&end| end as usize <= rom.len())
            .ok_or("Invalid physical PCM extent")?;
        if source % 4 != 0
            || source < previous_end
            || half(rom, source)? != 0
            || !matches!(flags, 0 | 0x4000)
            || frequency == 0
            || loop_start >= length
        {
            return Err(format!("Invalid physical PCM header at {source:#x}"));
        }
        ranges.extend(
            prior
                .uncovered(source..end)
                .into_iter()
                .map(|r| (r, "direct-pcm-waves")),
        );
        previous_end = end;
    }
    if ranges
        .iter()
        .map(|(r, _)| u64::from(r.end - r.start))
        .sum::<u64>()
        != PACKED_BYTES
    {
        return Err("Unexpected physical audio bank ownership".into());
    }
    Ok(ranges)
}
