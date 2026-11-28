//! European Flash descriptors, timer rows, and division bit lengths.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 536;

// Verified source spans from the E660 Flash and division readers.
pub(super) const SOURCES: [(Range<u32>, &str); 8] = [
    (0x145788..0x14578c, "flash-selector-literal"),
    (0x145d70..0x145d74, "division-table-literal"),
    (0x145dc8..0x145dcc, "division-table-literal"),
    (0x145f40..0x145f44, "division-table-literal"),
    (0x370830..0x370840, "flash-chip-directory"),
    (0x370868..0x3708d8, "flash-timers-and-descriptors"),
    (0x370900..0x370988, "flash-timers-and-descriptors"),
    (0x370990..0x370a90, "division-bit-lengths"),
];
