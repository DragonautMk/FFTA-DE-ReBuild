//! European map cells and literals for the original queued transfers.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_284;

// The five physical maps occupy one contiguous range. Both readers of
// 3ba4c0 keep using their original live row count.
pub(super) const SOURCES: [(Range<u32>, &str); 9] = [
    (0x093c2c..0x093c30, "queue_literals"),
    (0x093f50..0x093f54, "queue_literals"),
    (0x094298..0x09429c, "queue_literals"),
    (0x0945d8..0x0945dc, "queue_literals"),
    (0x09525c..0x095260, "queue_literals"),
    (0x12d4ec..0x12d4f0, "queue_literals"),
    (0x12d674..0x12d678, "queue_literals"),
    (0x12d7cc..0x12d7d4, "queue_literals"),
    (0x3ba358..0x3ba838, "tilemap_cells"),
];
