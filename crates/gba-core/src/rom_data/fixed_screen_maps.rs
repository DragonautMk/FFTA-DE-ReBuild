//! European tile cells read by the original 32-cell-stride screen copier.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 5_918;

// E622 bounds one 17-by-13 map and eight adjacent maps through all ten
// direct callers of 081441f8. Both localized references to b8d354 share it.
pub(super) const SOURCES: [(Range<u32>, &str); 12] = [
    (0x143d00..0x143d04, "reader_literals"),
    (0x143d98..0x143d9c, "reader_literals"),
    (0x143e54..0x143e58, "reader_literals"),
    (0x143f78..0x143f7c, "reader_literals"),
    (0x144018..0x14401c, "reader_literals"),
    (0x1440dc..0x1440e0, "reader_literals"),
    (0x144184..0x144188, "reader_literals"),
    (0x144364..0x144368, "reader_literals"),
    (0x1443c8..0x1443cc, "reader_literals"),
    (0x14445c..0x144460, "reader_literals"),
    (0x6df2dc..0x6df496, "tilemap_cells"),
    (0xb8c778..0xb8dcb4, "tilemap_cells"),
];
