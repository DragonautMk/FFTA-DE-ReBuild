//! European glyph data used by the original map-height display.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 2_308;

// E327 still supplies glyph 5, the base word and the second destination
// literal. Serve the other nine glyphs and the first copy destination.
pub(super) const SOURCES: [(Range<u32>, &str); 3] = [
    (0x0ac9a4..0x0ac9a8, "queue_literals"),
    (0x40e344..0x40e844, "glyph_tiles"),
    (0x40e944..0x40ed44, "glyph_tiles"),
];
