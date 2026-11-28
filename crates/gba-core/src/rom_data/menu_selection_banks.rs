//! Original European menu selection tables and reader literals.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 728;

// The E656 ranges preserve the separate indexed-word owner.
pub(super) const SOURCES: [(Range<u32>, &str); 27] = [
    (0x03bd94..0x03bd98, "menu-selection-literal"),
    (0x03bddc..0x03bde0, "menu-selection-literal"),
    (0x03be30..0x03be34, "menu-selection-literal"),
    (0x03be70..0x03be74, "menu-selection-literal"),
    (0x03beb8..0x03bebc, "menu-selection-literal"),
    (0x03bef8..0x03befc, "menu-selection-literal"),
    (0x03bf40..0x03bf44, "menu-selection-literal"),
    (0x03bfec..0x03bff0, "menu-selection-literal"),
    (0x03c2f4..0x03c2f8, "menu-selection-literal"),
    (0x03c32c..0x03c330, "menu-selection-literal"),
    (0x03c354..0x03c358, "menu-selection-literal"),
    (0x03c37c..0x03c380, "menu-selection-literal"),
    (0x03c3b4..0x03c3b8, "menu-selection-literal"),
    (0x03c3dc..0x03c3e0, "menu-selection-literal"),
    (0x03c414..0x03c418, "menu-selection-literal"),
    (0x03e040..0x03e044, "menu-selection-literal"),
    (0x03e068..0x03e06c, "menu-selection-literal"),
    (0x03e090..0x03e094, "menu-selection-literal"),
    (0x03e0b8..0x03e0bc, "menu-selection-literal"),
    (0x03e0f8..0x03e0fc, "menu-selection-literal"),
    (0x03e2f8..0x03e2fc, "menu-selection-literal"),
    (0x03ea10..0x03ea28, "menu-selection-literal"),
    (0x03eb58..0x03eb5c, "menu-selection-literal"),
    (0x0497a0..0x0497a4, "menu-selection-literal"),
    (0x0498dc..0x0498e0, "menu-selection-literal"),
    (0x39576c..0x3959a4, "menu-selection-tables"),
    (0x395a1c..0x395a44, "menu-row-selectors"),
];
