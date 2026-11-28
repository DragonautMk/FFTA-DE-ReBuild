//! European screen-effect schedules, motion records and scanline inputs.
use std::ops::Range;

pub(super) const PACKED_BYTES: u64 = 1_089;

// The pending-release comparison at 3ac734 uses the existing blend owner.
pub(super) const SOURCES: [(Range<u32>, &str); 26] = [
    (0x122904..0x122910, "reader_literals"),
    (0x122914..0x12291c, "reader_literals"),
    (0x122a18..0x122a24, "reader_literals"),
    (0x122a28..0x122a2c, "reader_literals"),
    (0x122d04..0x122d08, "reader_literals"),
    (0x1231fc..0x123200, "reader_literals"),
    (0x123264..0x123268, "reader_literals"),
    (0x1233f8..0x1233fc, "reader_literals"),
    (0x123858..0x12385c, "reader_literals"),
    (0x1239ac..0x1239b0, "reader_literals"),
    (0x1239f4..0x1239f8, "reader_literals"),
    (0x123c48..0x123c50, "reader_literals"),
    (0x123c58..0x123c64, "reader_literals"),
    (0x123dec..0x123df8, "reader_literals"),
    (0x3ac4dc..0x3ac508, "sprite_spawn_times"),
    (0x3ac508..0x3ac560, "first_sprite_selectors"),
    (0x3ac560..0x3ac580, "first_sprite_motion"),
    (0x3ac580..0x3ac640, "second_sprite_selectors"),
    (0x3ac640..0x3ac680, "second_sprite_motion"),
    (0x3ac680..0x3ac6a8, "offset_curve"),
    (0x3ac6a8..0x3ac734, "timed_motion_records"),
    (0x3ac7b8..0x3ac860, "scale_curve"),
    (0x3ac860..0x3ac900, "scanline_profile"),
    (0x3ac900..0x3ac928, "background_motion"),
    (0x3ac928..0x3ac937, "background_selectors"),
    (0x3ac938..0x3ac942, "background_delay"),
];
