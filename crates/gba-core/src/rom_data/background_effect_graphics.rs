//! The complete European background effect tile and tilemap bank.
use super::{
    script39_registry_poses::{word, PriorSources},
    Resource,
};
use std::ops::Range;

const ROM_BASE: u32 = 0x0800_0000;
const DESCRIPTOR_LITERAL: u32 = 0x12fa38;
const TILE_LITERAL: u32 = 0x12fad4;
const DESCRIPTOR_START: u32 = 0x3ad318;
const DESCRIPTOR_END: u32 = 0x3ad444;
const TILE_START: u32 = 0xd6b4c8;
const MAP_START: u32 = 0xd8f308;
const MAP_END: u32 = 0xd96838;
const DESCRIPTOR_BYTES: u32 = 12;
const MAP_BYTES: u32 = 30 * 20 * 2;
pub(super) const PACKED_BYTES: u64 =
    (8 + DESCRIPTOR_END - DESCRIPTOR_START + MAP_END - TILE_START) as u64;

/// Validate the physical projection against the original directory. The
/// caller selects this layout only after authenticating the exact EUR ROM.
pub(super) fn source_ranges(
    rom: &[u8],
    existing: &[Resource],
) -> Result<[(Range<u32>, &'static str); 5], String> {
    if word(rom, DESCRIPTOR_LITERAL)? != ROM_BASE + DESCRIPTOR_START
        || word(rom, TILE_LITERAL)? != ROM_BASE + TILE_START
    {
        return Err("Unexpected background effect graphics roots".into());
    }

    let mut tile = TILE_START;
    let mut map = MAP_START;
    for row in (DESCRIPTOR_START..DESCRIPTOR_END).step_by(DESCRIPTOR_BYTES as usize) {
        let relative_tile = word(rom, row)?;
        let tile_bytes = word(rom, row + 4)?;
        let map_pointer = word(rom, row + 8)?;
        if relative_tile != tile - TILE_START
            || tile_bytes == 0
            || tile_bytes % 64 != 0
            || tile_bytes > MAP_START - tile
            || map_pointer != ROM_BASE + map
        {
            return Err(format!(
                "Invalid background effect graphics row at {row:#x}"
            ));
        }
        tile += tile_bytes;
        map += MAP_BYTES;
    }
    if tile != MAP_START || map != MAP_END || map as usize > rom.len() {
        return Err("Incomplete background effect graphics bank".into());
    }

    let ranges = [
        (
            DESCRIPTOR_LITERAL..DESCRIPTOR_LITERAL + 4,
            "descriptor-table-literal",
        ),
        (TILE_LITERAL..TILE_LITERAL + 4, "tile-bank-literal"),
        (DESCRIPTOR_START..DESCRIPTOR_END, "graphics-descriptors"),
        (TILE_START..MAP_START, "tile-blocks"),
        (MAP_START..MAP_END, "tilemaps"),
    ];
    let prior = PriorSources::new(existing)?;
    for (range, _) in &ranges {
        if prior.uncovered(range.clone()) != [range.clone()] {
            return Err(format!(
                "Background effect graphics overlaps an existing owner at {:#x}",
                range.start
            ));
        }
    }
    Ok(ranges)
}
