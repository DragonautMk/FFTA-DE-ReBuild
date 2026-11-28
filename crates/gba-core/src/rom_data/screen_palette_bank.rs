//! European screen palette colors and their three original source literals.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_484;

// The first span of colors contains two adjacent 240-word palettes.
// The other span holds all 256 words read by the staged palette loader.
pub(super) const SOURCES: [(Range<u32>, &str); 5] = [
    (0x143ce0..0x143ce4, "reader_literals"),
    (0x144374..0x144378, "reader_literals"),
    (0x144b24..0x144b28, "reader_literals"),
    (0xb8be58..0xb8c218, "palette_colors"),
    (0xb8c378..0xb8c578, "palette_colors"),
];
