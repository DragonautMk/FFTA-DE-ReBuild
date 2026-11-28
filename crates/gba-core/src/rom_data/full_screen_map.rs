//! European tile cells and literals read by the original fixed screen copy.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_212;

// E643 verifies 20 source rows of 30 cells and a 64-byte destination pitch.
pub(super) const SOURCES: [(Range<u32>, &str); 2] = [
    (0x04fdd4..0x04fde0, "reader_literals"),
    (0x3b53fc..0x3b58ac, "tilemap_cells"),
];
