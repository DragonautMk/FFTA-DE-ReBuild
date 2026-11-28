//! European halfword pools and weighted rows bounded by the original readers.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 2_912;

// The first pool row (6e8994..6e89bc) and the literal at 02ed2c keep
// their prior owners. These are the residual E620 ranges in file order.
pub(super) const SOURCES: [(Range<u32>, &str); 11] = [
    (0x03b0a8..0x03b0ac, "reader_literals"),
    (0x0d3a5c..0x0d3a60, "reader_literals"),
    (0x0d3aa4..0x0d3aa8, "reader_literals"),
    (0x0d3af0..0x0d3af4, "reader_literals"),
    (0x0d3b34..0x0d3b38, "reader_literals"),
    (0x0d5db0..0x0d5db4, "reader_literals"),
    (0x0d5fbc..0x0d5fc0, "reader_literals"),
    (0x0d6ac4..0x0d6ac8, "reader_literals"),
    (0x6e89bc..0x6e8aac, "halfword_pools"),
    (0x6e900c..0x6e92dc, "halfword_pools"),
    (0x6e92dc..0x6e9a5c, "weighted_rows"),
];
