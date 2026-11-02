# Independent desktop dialogue layer

Current desktop path (2026-10-16): FFTA_UI_LAYER=1, enabled by default.
FFTA_UI_LAYER=0 selects the previous world-framebuffer layout for A/B comparison.

The core extracts BG2 text, BG3 backing and verified 8-bpp portrait pieces,
including host-list replacements, into a fixed native 480x160 canvas. Its padding
supports Length without stretching the text. Each pixel stores native color and
integer blend coefficients. Capture follows scanline registers, priorities,
windows, palette, brightness and the PPU's rounded six-bit green mixing.
Guest VRAM, OAM, coordinates and the hardware framebuffer remain unchanged.

The world-only host pass excludes these UI planes and portrait pieces. The
presenter draws the native UI after world filtering, directly in physical window
coordinates. Its layout depends on the base aspect, window, Size and attachment
edge; world zoom is not an input. There is no intermediate enlarged UI image.
GPU and software paths use the same native integer mix against the relocated
world, then the current color LUT. Pause composes this layer before its overlay.
UI bypasses world temporal response. Game UI uses the configured pixel grid
after native mixing and the color LUT, with the same band-limited apertures as
the world. Its grid follows the native UI layout and Size, independently of
world zoom. Grid strength, scaling-mode footprint, the small-scale fade and
EPX's grid disable rule are shared with the world pass. The FPS overlay remains
flat; the software fallback and pause-menu canvas retain their no-grid behavior.

## Native HUD grid correction (2026-10-23)

The separate native pass originally applied only integer blending and the LUT,
so extracted character portraits, battle HUDs and panels lost the Grid effect
that remained visible on the world. Both GPU passes now assemble the shared
`crates/screen/src/grid_filter.wgsl` model. Game layers explicitly receive the
current grid settings; application overlays opt out. Grid neighbours resolve
their backdrop at each relocated native texel centre. The existing CPU-defined
coordinate maps, exact transparent-pixel predicate and scissors still determine
the centre pixel and coverage. Turning Grid off preserves the previous output.

Validation on Windows binaries under Wine: the screen suite passes 44 tests
(8 optional GPU/window checks ignored); two explicit GPU checks pass for the
new grid behavior and existing grid-off CPU/GPU parity. The grid check covers
integer/fractional UI scales, settings changes with cached textures, zero/half/
full strength, world-zoom independence, transparent coverage, small-scale fade
and EPX disablement. Native Windows and Android are not validated here.
Two normal Windows/Wine GUI sessions also pass slot load, resume and window
close with Grid off/on; inspected battle captures show the lower-left portrait
and both HUD panels filtered, including at maximum world zoom, while FPS stays
flat. ROM, source snapshot/save and copied slot hashes are unchanged. An initial
bounded replay harness instead exited at 601/600 frames; that run is retained
as failed harness evidence and is not claimed as replay parity or a clean exit.
The successful normal-play checks do not depend on that harness. No performance
analysis or packaging was performed.

First migrated profile: measured battle dialogue BG2=0B08 / BG3=0A08 with verified
terrain and exact regional ROM gates. Other menus retain their existing rendering;
this is the foundation for migrating those profiles individually, not a claim
that every game screen has already been separated. Android is unchanged.

Validation:
- 189 suite tests passed, 5 ignored (including the GPU test run separately).
- GPU readback on NVIDIA GeForce RTX 5050 matches CPU output at every pixel,
  including alpha, with two world zooms, without requiring a window.
- Native dialogue reconstruction matches the hardware framebuffer with and
  without transparency; extraction leaves no portrait in the world layer.
- EUR and USA: 600 frames each, extra=0, framebuffer/OAM hashes unchanged.
- Private existing EUR snapshot: x1, x1.122462 and x2 UI data are identical:
  SHA256 AFABABEDA1C8119594BD233B99EE449E2C2465652F85E4C4AFEBF0CBED933F00.
  All 72,855 opaque output UI pixels are also identical across the three zooms.
- Captures: target/ui-layer-before.png, ui-layer-after-x1.png,
  ui-layer-after-step1.png and ui-layer-after-x2.png. These are reconstructed
  diagnostics; they do not restore the snapshot's complete host terrain lists.
- Eight-character gameplay replay metric was not rerun for this change.

Reproduce private snapshot preview (no ROM or game snapshot is distributed):

    cargo run --release -p recomp --example dialogue_layout -- target/live-action --rom ROM_PATH --zoom 2 --out target/ui-layer-after-x2.png

## Previous implementation notes (superseded by the path above)

# Dialogue anchoring and zoom, 2026-10-16

Measured from the existing private EUR battle dialogue snapshot:
BG2=0B08 (text, HOFS=6), BG3=0A08 (backing), terrain BG0/BG1=4C03/4E02.
The right-attached backing occupies x=92..239, y=14..65. Five priority-zero
8-bpp non-affine OAM pieces, tiles 240..29A, form the portrait. The battle
bodies in this snapshot are separate 4-bpp pieces. No ROM bytes are stored here.

For this verified scene signature, the wide view now relocates text,
backing and portrait instead of extending an edge column. The backing's
actual opaque pixels at x=0 and x=239 determine its attachment. Equal or
empty edge counts retain the previous layout. The gate also requires the
verified movement patch, enabled text layers and a non-wrap BG policy.
The gate accepts either verified ROM profile if the measured layout matches;
visual confirmation so far uses the EUR snapshot, not an unobserved USA frame.

The host UI is scaled by render-height / 160, then the ordinary world view
is presented as before. This compensates zoom-out: the dialogue retains its
screen-space size while the world shrinks. Sampling uses original native
tile coordinates, including scrolling, flips, palette and normal priority /
blend composition. The portrait uses the same transform and is not copied
from the already composited world image. Original OAM/VRAM stay untouched.

Only the wide framebuffer receives this layout. The hardware 240x160
framebuffer still receives the original composition; at extra=0 the new
path is inactive. World BG rows are reused for the second wide composition.
Portrait-identification is confined to the verified dialogue scene, 8-bpp
priority-zero tiles in the portrait allocation at the native left/right edge;
the unit producers, sprite list, camera and game logic are unchanged.

## Future Settings adjustment point

`MemMap::ffta_dialogue.style`, defined in `gba-core/src/ffta_dialogue.rs`, is
the single per-machine adjustment point: `scale` (default 1), `offset_x`
and `offset_y` (native UI pixels, default 0). Frontends can update these
between frames when Settings is implemented later. Nothing is exposed in
the current settings menu and no new settings persistence was added.

A/B: set `FFTA_DIALOGUE_LAYOUT=0` for the prior long backing, or `1` for
anchoring (default). Set it before launching a new process.

## Validation

- Core: 99 tests passed, including synthetic left/right anchoring at x1/x2,
  portrait sampling, hardware framebuffer/OAM preservation and unrelated /
  unverified scenes staying on the old path.
- 600 native frames, extra=0, layout off vs on: identical framebuffer and OAM
  hashes for every frame. No save was loaded or written for that check.
- Actual private graphics snapshot reconstructed before/after in 32:9 at
  x1 (568x160) and x2 (1136x320). Text and portrait remain together, the
  backing reaches its own edge without a stretched band, and its normalized
  screen size is stable. These are offline renders, not a live game capture.
- The specific changed behaviour still needs user feedback while playing.

Previews: `target/dialogue-anchor-before-x1.png`, `after-x1.png`,
`before-x2.png`, `after-x2.png` (all with prefix `dialogue-anchor-`).
Windows build: `target/release/recomp-dialogue-layout.exe`, also copied
to the usual `target/release/recomp.exe` and `Installer/payload/recomp.exe`.


## Zoom and user controls (2026-10-16)
Video > Text Bubbles exposes Size (50-200 percent) and Length (100-200 percent).
Both persist in the existing AV configuration as bubble.size and bubble.length.
Length expands only the flat backing, preserving text, portrait and curved cap proportions.
The desktop supplies the exact zoom/crop rather than using the rounded framebuffer height.
Raw portrait placement excludes host-list objects and objects outside the native screen.
The parked OAM sentinel filter now applies to horizontal widescreen as well as vertical zoom.
Transient startup objects: NOT visually verified; these are bounded candidate corrections, not a confirmed resolution.
Validation: 157 tests passed, 4 ignored; 13 zoom levels covered by placement test;
600 EUR frames have identical hardware framebuffer/OAM hashes against layout-disabled reference.


## Host portrait regression correction
The preceding exclusion of host-list objects was incorrect: published host entries
replace the native OAM portrait pieces during real play. They must use the same
placement as raw OAM pieces. Removed that exclusion, retaining the verified scene,
8-bpp/tile/priority filters and bounded native portrait coordinates (including
partially offscreen pieces). Length still expands only the backing; Size scales
text, portrait and backing together at their shared screen anchor.
New regression test publishes an OAM-replacing HostSprite and compares every
rendered pixel against raw OAM at both edges, three sizes and seven zoom levels.
It also asserts the portrait bounds against the shared dialogue transform.
MEASURED: 158 suite tests passed, 4 ignored. Live user appearance remains to confirm.


## Continuation marker and root settings entry
Measured from a complete USA replay using the backup save: frame 2141 has the
continuation marker in OAM slot 0, attributes 803B/0088/2122 (8x16, 4-bpp,
palette 2, priority 0, tile 0x122). Subsequent samples preserve Y=59/58/57
and 147/146/145 animation. The tile pair's 64 VRAM bytes also match the existing
EUR snapshot; no graphics bytes are included in source. The decoded image
confirms this is the downward continuation pointer, not a unit or battle cursor.
This exact OBJ signature now joins the dialogue layer and is excluded from the
world pass. Both raw OAM and published host replacements follow the same path;
blink/disabled state and bouncing Y remain owned by the game.
Captures: target/dialogue-pointer-native-layout.png (before),
target/dialogue-pointer-after-x1.png and target/dialogue-pointer-after-x2.png.
Text Bubbles is now a root menu sibling directly after Fast Forward, with an
original vector bubble icon. Main-menu empty header removed so all eight entries
fit while retaining the normal button/font sizes. Preview navigation and the
XInput capture test now target Input's new row.


## Size grows inward from the corresponding screen edge
Confirmed with a direct PrintWindow capture of the running desktop game. Lower
portrait/dialogue was being scaled around native screen Y=0, clipping the bottom
at Size=110 percent. The scene was copied with read-only process access for
reconstruction; the running game and its save were not modified or closed.
Native backing rows and portrait rectangles now determine upper/lower attachment.
Their nearest screen edge (Y=0 or Y=160) is the Size pivot. Horizontal attachment
continues to use the existing left/right edge. Both axes use one uniform factor.
The continuation marker is excluded when choosing attachment, preventing its
bounce from moving the entire layer, while the full native screen envelope
reserves room for its tip. GPU, software and paused backdrop share the layout.
MEASURED: same captured scene at Size=110 and 200 percent stays inside vertically;
100-percent PNG SHA256 remains AAC332A69764523F514F797CA545D5678F6C55140FFD8D326549282621299E6B.
192 tests passed, 5 ignored. EUR/USA extra=0 framebuffer/OAM unchanged for 600 frames.
Captures: target/bubble-size-window-before.png (actual window),
target/bubble-inward-before-110.png, bubble-inward-final-110.png,
bubble-inward-final-200.png (same scene reconstructed after correction).
