//! Five complete European rectangles and the task's selector tables.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 2_458;

// The five rectangles have identical contents and distinct original addresses.
// Keep each address visible through the reader's directory and leave alignment
// outside the source projection.
pub(super) const SOURCES: [(Range<u32>, &str); 9] = [
    (0x124144..0x124148, "reader_literals"),
    (0x12414c..0x124150, "reader_literals"),
    (0x3ac9e4..0x3ac9f8, "selector_presets"),
    (0x3ac9f8..0x3aca0c, "rectangle_directory"),
    (0xae9110..0xae92f2, "rectangle_cells"),
    (0xae92f4..0xae94d6, "rectangle_cells"),
    (0xae94d8..0xae96ba, "rectangle_cells"),
    (0xae96bc..0xae989e, "rectangle_cells"),
    (0xae98a0..0xae9a82, "rectangle_cells"),
];
