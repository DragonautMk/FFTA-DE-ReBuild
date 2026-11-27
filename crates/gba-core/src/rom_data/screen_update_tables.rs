//! European screen-update curves, pointer directories and reader literals.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 6_434;

// E615 verified both physical copies and the literal pools against the exact
// EUR ROM. The first trailing zero belongs to step 150 of the last byte curve.
// Grid rows keep their compact OBJ catalog owner.
pub(super) const SOURCES: [(Range<u32>, &str); 10] = [
    (0x11a17c..0x11a180, "reader_literals"),
    (0x11a1cc..0x11a1d0, "reader_literals"),
    (0x11a204..0x11a208, "reader_literals"),
    (0x11a39c..0x11a3a4, "reader_literals"),
    (0x11b874..0x11b878, "reader_literals"),
    (0x11b8c4..0x11b8c8, "reader_literals"),
    (0x11b8fc..0x11b900, "reader_literals"),
    (0x11bab0..0x11bab4, "reader_literals"),
    (0x3a8e3c..0x3a9abb, "curves_and_directories"),
    (0x3aa1c4..0x3aae43, "curves_and_directories"),
];
