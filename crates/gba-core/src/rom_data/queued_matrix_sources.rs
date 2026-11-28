//! European queued maps and their original source literal spans.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 924;

// Source layout from the E653 queue and initializer readers.
pub(super) const SOURCES: [(Range<u32>, &str); 10] = [
    (0x02df28..0x02df2c, "queue_literals"),
    (0x02f33c..0x02f344, "queue_literals"),
    (0x02f7c8..0x02f7d0, "queue_literals"),
    (0x02faa4..0x02faac, "queue_literals"),
    (0x02fc3c..0x02fc44, "queue_literals"),
    (0x0300f8..0x030100, "queue_literals"),
    (0x0303b4..0x0303bc, "queue_literals"),
    (0x3ba048..0x3ba0d8, "queue_cells"),
    (0x437194..0x437300, "queue_cells"),
    (0x448278..0x4483e4, "queue_cells"),
];
