//! European source rows for the original b543a8 UI tilemap copies.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_902;

// Four directories and three direct roots select all 31 maps. Their
// aliases remain at their cartridge addresses. The final alignment stays
// outside the projection.
pub(super) const SOURCES: [(Range<u32>, &str); 8] = [
    (0x03a88c..0x03a894, "reader_literals"),
    (0x03a8f8..0x03a8fc, "reader_literals"),
    (0x03aaf8..0x03aafc, "reader_literals"),
    (0x03ab00..0x03ab04, "reader_literals"),
    (0x03aeec..0x03aef0, "reader_literals"),
    (0x03b084..0x03b08c, "reader_literals"),
    (0x395a44..0x395ae4, "indexed_directories"),
    (0xb543a8..0xb54a56, "tilemap_cells"),
];
