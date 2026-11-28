//! European save CRC32 words and the lookup routine's two literals.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_032;

pub(super) const SOURCES: [(Range<u32>, &str); 2] = [
    (0x13ef1c..0x13ef24, "reader_literals"),
    (0x397180..0x397580, "checksum_table"),
];
