//! European curve records, position records and the independent terminal copy.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_450;

// The compact OBJ owner supplies seven bytes of each of the first 14
// position records. Index 60 of the last curve reads the terminal copy.
pub(super) const SOURCES: [(Range<u32>, &str); 26] = [
    (0x114200..0x114204, "reader_literals"),
    (0x114208..0x11420c, "reader_literals"),
    (0x1142c0..0x1142c4, "reader_literals"),
    (0x1152c0..0x1152c4, "reader_literals"),
    (0x1152cc..0x1152d0, "reader_literals"),
    (0x1154ac..0x1154b0, "reader_literals"),
    (0x1193d8..0x1193dc, "reader_literals"),
    (0x3a7fb0..0x3a808c, "curve_records"),
    (0x3a80cb..0x3a80cc, "position_records"),
    (0x3a80d3..0x3a80d4, "position_records"),
    (0x3a80db..0x3a80dc, "position_records"),
    (0x3a80e3..0x3a80e4, "position_records"),
    (0x3a80eb..0x3a80ec, "position_records"),
    (0x3a80f3..0x3a80f4, "position_records"),
    (0x3a80fb..0x3a80fc, "position_records"),
    (0x3a8103..0x3a8104, "position_records"),
    (0x3a810b..0x3a810c, "position_records"),
    (0x3a8113..0x3a8114, "position_records"),
    (0x3a811b..0x3a811c, "position_records"),
    (0x3a8123..0x3a8124, "position_records"),
    (0x3a812b..0x3a812c, "position_records"),
    (0x3a8133..0x3a8134, "position_records"),
    (0x3a823c..0x3a8304, "curve_records"),
    (0x3a8304..0x3a8384, "position_records"),
    (0x3a89b0..0x3a8cf8, "curve_samples"),
    (0x3a8cf8..0x3a8d0c, "terminal_copy"),
];
