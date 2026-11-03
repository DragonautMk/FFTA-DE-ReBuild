# U31-10: Android display color pipeline

Status: **partial source comparison; DEFER pending a measured display need**.
Upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` is compared with fork
`decomp` `83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02. The tag
distinction is documented in [U31-02](U31-02-scrollbars.md). No rendering,
device, performance or color measurements were made.

Upstream Android adds `gba-android/src/video/{color,lut,profile}.rs` and the
Kotlin `VideoPass`. `NativeBridge.gbaVideoPalette` builds a 32,768-entry LUT
from screen profile, darken and brightness settings with an sRGB target; the
pass caches/uploads it and can combine presentation scaling, temporal response
and grid effects after drawing world and UI. `VideoOptions` defaults to raw
screen, brightness 1, no response/grid, scaler 1 and vsync enabled; under
those settings the pass's `enabled` predicate bypasses the postprocess. The
source contains LUT tests for raw conversion, channel order, neutral white,
monotonicity and profile properties. These are not executed evidence here.

The fork's separate Android engine currently builds a cached 32,768-entry raw
BGR555-to-ARGB conversion table (`rgba_colors`) and tests the integer 5-to-8
conversion exhaustively in source. Desktop already has its own `screen` color
profiles, LUT, temporal response and GPU presentation pipeline, which is not
proof of Android parity. Raw lookup formulae differ at some 5-bit channel
values (fork integer `value*255/31` versus upstream LUT bit replication); the
upstream default bypasses `VideoPass`, so this observation alone does not prove
a visible or default-output regression. Compare actual final output and shader
sampling, including composed UI, before asserting equivalence or improvement.

Potential integration must keep color processing in the display path, preserve
raw guest/frame evidence and consider framebuffer recreation, history reset,
GPU capability, brightness bounds, defaults, settings persistence and device
color management. Screen models are presentation choices, not evidence of
measured accuracy to original hardware. First define an isolated reversible
Android option and verify exact bypass output, LUT reference values, channel
order, alpha/composition and changes across resize/pause/context loss. Then
compare real-device images and frame pacing with specified settings; desktop,
Android and different GPUs need separate evidence. No palette/ROMDATA migration
or whole-renderer import is justified by this comparison.
