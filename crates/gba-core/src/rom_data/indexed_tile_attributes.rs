//! Packed attribute tables for European indexed tile archives.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_187;

// The E399 owner supplies the byte at 3c9460. The indexed archive
// resource supplies the five localized tables and their directory.
pub(super) const SOURCES: [(Range<u32>, &str); 5] = [
    (0x0cf3dc..0x0cf3e0, "reader_literals"),
    (0x0cf494..0x0cf49c, "reader_literals"),
    (0x3bd70c..0x3bdb24, "tile_attributes"),
    (0x3c943c..0x3c9460, "tile_attributes"),
    (0x3c9461..0x3c94bc, "tile_attributes"),
];
