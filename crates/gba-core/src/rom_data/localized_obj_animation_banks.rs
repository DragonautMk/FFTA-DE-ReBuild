//! The European five-language OBJ directory and its three directly selected peers.
use super::{
    custom_lz,
    obj_animations::Bank,
    script39_registry_poses::{half, word, PriorSources},
    Resource,
};
use std::ops::Range;

const ROM_BASE: u32 = 0x0800_0000;
const TABLE: u32 = 0x398e94;
const LANGUAGES: u32 = 5;
const SLOTS: u32 = 28;
const ROW_BYTES: u32 = 12;
const TABLE_END: u32 = TABLE + LANGUAGES * SLOTS * ROW_BYTES;
const TABLE_LITERALS: [u32; 2] = [0x0ac52c, 0x0ac638];
// Literal locations in the default, language 2 and language 4 branches.
const DIRECT_LITERALS: [([u32; 3], usize); 3] = [
    ([0x0954ac, 0x0954b0, 0x0954b4], 1984),
    ([0x0954c8, 0x0954cc, 0x0954d0], 1856),
    ([0x095548, 0x09554c, 0x095550], 1920),
];
pub(super) const PACKED_BYTES: u64 = 113_870;

fn source_pointer(rom: &[u8], value: u32) -> Result<u32, String> {
    value
        .checked_sub(ROM_BASE)
        .filter(|&offset| offset % 4 == 0 && (offset as usize) < rom.len())
        .ok_or_else(|| format!("Invalid localized OBJ source pointer {value:#x}"))
}

fn bank_ranges(
    rom: &[u8],
    sequence: u32,
    objects: u32,
) -> Result<impl Iterator<Item = (Range<u32>, &'static str)>, String> {
    let bank = Bank::decode(
        rom,
        source_pointer(rom, objects)?,
        source_pointer(rom, sequence)?,
    )?;
    Ok(bank
        .spans()
        .into_iter()
        .map(|(start, length, role)| (start..start + length as u32, role)))
}

/// Project authenticated directory fields and declared records. Guest selectors
/// and alignment bytes keep their original behavior outside this projection.
pub(super) fn source_ranges(
    rom: &[u8],
    existing: &[Resource],
) -> Result<Vec<(Range<u32>, &'static str)>, String> {
    let mut ranges = vec![(TABLE..TABLE_END, "animation-descriptors")];
    for literal in TABLE_LITERALS {
        if word(rom, literal)? != ROM_BASE + TABLE {
            return Err("Unexpected localized OBJ descriptor directory".into());
        }
        ranges.push((literal..literal + 4, "selector-literals"));
    }
    for language in 0..LANGUAGES {
        for slot in 0..SLOTS {
            let row = TABLE + (language * SLOTS + slot) * ROW_BYTES;
            let sequence = word(rom, row)?;
            let objects = word(rom, row + 4)?;
            let tiles = half(rom, row + 10)?;
            if slot < 2 {
                if (sequence, objects, word(rom, row + 8)?) != (0, 0, 0) {
                    return Err("Unexpected reserved localized OBJ descriptor".into());
                }
                continue;
            }
            if tiles == 0 || tiles % 32 != 0 {
                return Err(format!("Invalid localized OBJ transfer size at {row:#x}"));
            }
            ranges.extend(bank_ranges(rom, sequence, objects)?);
        }
    }
    for ([graphics, sequence, objects], size) in DIRECT_LITERALS {
        for literal in [graphics, sequence, objects] {
            ranges.push((literal..literal + 4, "selector-literals"));
        }
        let start = source_pointer(rom, word(rom, graphics)?)?;
        let source = &rom[start as usize..];
        let block = custom_lz::decode(source, size, super::obj_graphics::MAX_ENCODED)?;
        if block.bytes.len() != size {
            return Err(format!("Unexpected localized OBJ tile count at {start:#x}"));
        }
        ranges.push((start..start + block.consumed as u32, "tile-streams"));
        ranges.extend(bank_ranges(rom, word(rom, sequence)?, word(rom, objects)?)?);
    }

    ranges.sort_unstable_by_key(|(range, _)| range.start);
    if ranges
        .windows(2)
        .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return Err("Overlapping localized OBJ source records".into());
    }
    let prior = PriorSources::new(existing)?;
    let mut uncovered: Vec<(Range<u32>, &str)> = Vec::new();
    for (range, role) in ranges {
        for span in prior.uncovered(range) {
            if let Some((previous, previous_role)) = uncovered.last_mut() {
                if *previous_role == role && previous.end == span.start {
                    previous.end = span.end;
                    continue;
                }
            }
            uncovered.push((span, role));
        }
    }
    let length: u64 = uncovered
        .iter()
        .map(|(span, _)| u64::from(span.end - span.start))
        .sum();
    if length != PACKED_BYTES {
        return Err("Localized OBJ animation bank coverage differs from the source plan".into());
    }
    Ok(uncovered)
}
