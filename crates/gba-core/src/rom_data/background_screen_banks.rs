//! European screen tile blocks and maps selected by the original BG task.
use super::{
    script39_registry_poses::{half, word, PriorSources},
    Resource,
};
use std::ops::Range;

const ROM_BASE: u32 = 0x0800_0000;
const BANKS: u32 = 5;
const TILE_POINTERS: u32 = 0x3ac770;
const TILE_COUNTS: u32 = 0x3ac784;
const MAP_POINTERS: u32 = 0x3ac790;
const MAP_DIMENSIONS: u32 = 0x3ac7a4;
const TILES_START: u32 = 0x6cd968;
const MAPS_START: u32 = 0x6dd9e8;
const EXTRA_ROW_START: u32 = 0x6de982;
const EXTRA_ROW_END: u32 = 0x6de9cc;
const MAPS_END: u32 = 0x6df2dc;
pub(super) const PACKED_BYTES: u64 =
    (32 + BANKS * 14 + MAPS_END - TILES_START - (EXTRA_ROW_END - EXTRA_ROW_START)) as u64;

/// Validate the projection of the original reader for the authenticated EUR ROM.
pub(super) fn source_ranges(
    rom: &[u8],
    existing: &[Resource],
) -> Result<[(Range<u32>, &'static str); 10], String> {
    for (at, value) in [
        (0x123618, ROM_BASE + TILE_POINTERS),
        (0x12361c, ROM_BASE + TILE_COUNTS),
        (0x1236bc, 0x0100_0800),
        (0x1236c0, ROM_BASE + MAP_DIMENSIONS),
        (0x1236c4, ROM_BASE + MAP_POINTERS),
        (0x1236c8, 0x03ff),
        (0x123724, ROM_BASE + MAP_DIMENSIONS),
        (0x123728, 0x0600_6000),
    ] {
        if word(rom, at)? != value {
            return Err(format!("Unexpected background screen literal at {at:#x}"));
        }
    }

    let mut tile = TILES_START;
    let mut map = MAPS_START;
    for index in 0..BANKS {
        let tile_bytes = u32::from(half(rom, TILE_COUNTS + 2 * index)?);
        let dimensions = word(rom, MAP_DIMENSIONS + 4 * index)?;
        let width = dimensions & 0xff;
        let height = (dimensions >> 8) & 0xff;
        if word(rom, TILE_POINTERS + 4 * index)? != ROM_BASE + tile
            || word(rom, MAP_POINTERS + 4 * index)? != ROM_BASE + map
            || tile_bytes == 0
            || tile_bytes % 32 != 0
            || tile + tile_bytes > MAPS_START
            || !(1..=64).contains(&width)
            || !(1..=64).contains(&height)
            || dimensions >> 16 != 0
            || 32 * (height - 1) + (width - 1) % 32 + 1024 * ((width - 1) / 32) >= 2048
        {
            return Err(format!("Invalid background screen entry {index}"));
        }

        tile += tile_bytes;
        map += 2 * width * height;
        if index == 2 {
            // The third map stores an extra row that its declared height excludes.
            let extra = rom.get(EXTRA_ROW_START as usize..EXTRA_ROW_END as usize);
            if map != EXTRA_ROW_START
                || map + 2 * width != EXTRA_ROW_END
                || !extra.is_some_and(|row| row.chunks_exact(2).all(|cell| cell == [0xff, 0x03]))
            {
                return Err("Unexpected row after the third background screen map".into());
            }
            map = EXTRA_ROW_END;
        }
    }
    if tile != MAPS_START || map != MAPS_END || map as usize > rom.len() {
        return Err("Incomplete background screen banks".into());
    }

    let ranges = [
        (0x123618..0x123620, "tile-directory-literals"),
        (0x1236bc..0x1236cc, "map-construction-literals"),
        (0x123724..0x12372c, "map-upload-literals"),
        (TILE_POINTERS..TILE_POINTERS + 4 * BANKS, "tile-pointers"),
        (TILE_COUNTS..TILE_COUNTS + 2 * BANKS, "tile-byte-counts"),
        (MAP_POINTERS..MAP_POINTERS + 4 * BANKS, "map-pointers"),
        (MAP_DIMENSIONS..MAP_DIMENSIONS + 4 * BANKS, "map-dimensions"),
        (TILES_START..MAPS_START, "tile-blocks"),
        (MAPS_START..EXTRA_ROW_START, "tilemaps-before-extra-row"),
        (EXTRA_ROW_END..MAPS_END, "tilemaps-after-extra-row"),
    ];
    let prior = PriorSources::new(existing)?;
    for (range, _) in &ranges {
        if prior.uncovered(range.clone()) != [range.clone()] {
            return Err(format!(
                "Background screen banks overlap an existing owner at {:#x}",
                range.start
            ));
        }
    }
    Ok(ranges)
}
