<a id="ffta-estado-de-investigacion-del-fallo-real"></a>

# FFTA: investigation status for the reported issue

2026-10-18: USA Pub separated into the right corner; USA shop uses a centered
canvas; the USA empty-bank notice was identified. Shared USA service windows
are recognized by owner; the actual Prison scene remains NOT MEASURED.
[ffta_usa_town_menus.md](ffta_usa_town_menus.md).

2026-10-17: mouse profile for Press Start and the USA main menu implemented
and tested by replay. [ffta_usa_frontend_mouse.md](ffta_usa_frontend_mouse.md).

2026-10-17: user confirms magic effects already corrected; do not touch them.
Lightning damage banner measured in WIN0 [16,8,224,24], extracted in full
without the X=160 clipping. [ffta_spell_help_banner.md](ffta_spell_help_banner.md).

2026-10-17: Lester's action menu is split at X=160; its actual frame begins
at X=152. Extraction follows the BG2 header/footer, pending feedback.
[ffta_action_frame_width.md](ffta_action_frame_width.md).
Next pending issue: Thunder/Pyro magic effects repeated on both sides.

2026-10-17: Clara's ear tip in Giza captured live and corrected.
[ffta_portrait_ears.md](ffta_portrait_ears.md): two pieces at Y=48/56 excluded
by the old Y>=64 filter now share the portrait layer.

2026-10-17: Comprar/Vender centered; compact Vender categories and Monster Bank
measured by replay with coordinate clicks. Details and limits:
[ffta_shop_bank_mouse.md](ffta_shop_bank_mouse.md).

Priority: reproduce ffta-batalla-3.input before claiming a solution.
The earlier 44-object test does not reproduce the issue reported by the user.
No further game logic change is applied without approval.
No commits or installer changes in this investigation.

MEASURED: AGENTS.md read and git status --short run; the repository already
contains staged, unstaged and untracked changes.
MEASURED: initial release build passed in 0.42 s; diagnostics build passed in
26.86 s; recomp tests: 6 passed.

MEASURED: ffta-batalla-3.input ends with an event at frame 12440.
It contains release and press events on the same frame, for example 11730.
InputScript sorted tuples (frame, mask), changing their recorded order.
The reader was corrected to sort stably by frame only.
This affects host replay, not the game logic or bytes.

MEASURED: runc executes the native path with the same Machine, MemMap, run_dma
and PPU as play, without a window, desktop audio, vsync or frame wait.
--sav was added to preload a copy into memory; runc does not write the .sav.
Input is replayed from frame 0, not from the start of the analyzed segment.

MEASURED: created E:/AI/ARC/ffta-respaldo.sav without modifying the original.
The copy, the original and ffta-batalla-3.input.sav have SHA256:
71189F7FB6AED638640078FBA3A35FDA6C39C8962E74DCC75935AAC948DA9063.
The replay uses another private copy: target/ffta-replay3/start.sav.
ASSUMPTION: HLE startup, based on the BIOS paths searched locally;
the user was asked whether the recording used a real BIOS.

Exact command for the first complete replay:

```powershell
$env:FFTA_WIDESCREEN='32:9'
$env:FFTA_PATCH_BG='1'
$env:FFTA_PATCH_SPR='1'
$env:FFTA_BG_EDGE='transparent'
$env:FFTA_WIDE_DEBUG=''
$env:FFTA_OAM_LOG='1'
$env:FFTA_OAM_LOG_FROM='10500'
$env:FFTA_OAM_CAPTURE_PREFIX='target/ffta-replay3/run1'
$env:FFTA_BENCH_LIBRARY='target/ffta-movement/native-normal.dll'
.\target\release\recomp.exe runc 'Final Fantasy Tactics Advance (USA).gba' --frames 12540 --input ffta-batalla-3.input --sav target/ffta-replay3/start.sav --out target/ffta-replay3/run1-final.ppm *> target/ffta-replay3/run1.log
```

The named DLL is an already compiled local copy of the user's ROM;
it must not be distributed. From frame 10500 the log records, per frame,
the BG camera, calls/arguments to the three known producers, received
descriptors, host list and OAM entries. Every 30 frames it records hashes of
both framebuffers, OAM and OBJ VRAM; every 120 it can save an image.
MEASURED: the replay reached battle; images of the final segment were generated.
Two complete runs from frame 0 produced 69 identical samples of the central
framebuffer, wide framebuffer, OAM and OBJ VRAM, frames 10500..12540.
Literal script output: determinism_samples 69 mismatches [].
Final FNV hash for both: abc92001e317eae0.
Final central SHA256 hash for both:
3ffb91edfe2e1bc047624f0b43c0c30b39770af3ba031d364a260f6b48f66bc2.
NOT MEASURED: user confirmation that the recorded startup was HLE;
the recording stores no BIOS metadata and no installed BIOS was found.

MEASURED: nine graphics instances are identified by the priority pointer
passed to the producer minus 0x1e, not by palette or r5.
In the final segment, calls for some instances disappear before
030065A8; there are no mismatches in the host list or transferred OAM.
For example, 0201af18 stops arriving after frame 12042, near the original
left edge. Measurement of its jumps and tiles remains unfinished.

MEASURED by reading: 030062EC, Thumb, ROM source 08A38EA0..08A38FC8,
calculates list visibility with limits 0..239 in X and 0..159 in Y.
08394028 contains its Thumb pointer 030062ED. Caller 0808F804 invokes it
at 0808F832 through thunk 08142254 and marks it visible only if it returns 1.
08021DA8 returns NULL if flags & 0x40 == 0; caller 0808F1F4 exits
at 0808F216 on NULL, before emitting objects.
MEASURED: the 296 RAM bytes match the ROM source, hash:
1e988998a21067596046e2b303da1ad9b6924c18146f396ce0bf5f6e038433c7.
MEASURED: at producer frame 12041, X=-7, the function returns 1 and
030065A8 is invoked for 0201af18. At frame 12042, X=-10, its six components
are rejected on the left; it returns 0 and the producer is no longer invoked.
The trace reads actual interpreter instructions on the native path:

The literal game disassembly was removed from the public source distribution.
Only independently described behavior, hook addresses and verification hashes are retained.

At 0300638A, the r0 values for the six right edges are
-2, -10, -2, -10, -2, -10. All are below 0 but within the
32:9 margin, whose left limit is -164.
Caller 0808F804 does not mark the visibility result when it returns 0.
MEASURED: it is lost before the producer, not through host publication failure.

MEASURED: the 320 OBJ VRAM bytes referenced by that instance in its last
received list remain identical in image frames 12041..12044:
8dac051f7fa7a16c7abd58bbaef877779ae968e1b88dbb2defc903151d6d84eb.
170 nonzero bytes, zero references outside OBJ VRAM. Therefore, freeing these
tiles does not explain this particular disappearance.
NOT MEASURED: all assets of units never visible or all maps.
Filtering at 03006170 adjusts priority for intersections; it is not a direct
comparison with 240, and no change is proposed based on similarity.

MEASURED: gamedb.sqlite returned no functions for this ROM in 0808E000..08090000;
these addresses came from actual calls and instruction reads.
Complete disassemblies and logs remain in target/ffta-replay3, ignored
by git. No ROM bytes or generated DLLs are included in the distribution.

MEASURED by reading: another Thumb producer exists at 030068A4, originating
at 08A39458, with comparisons against 239/159; it is not in the current hook.
MEASURED: observation of 030068A4 recorded no entries in frames
10500..12539. The DLL table has other=false for 030068A5: its entry
would have passed through the observed interpreter. It does not participate
in this segment; NOT MEASURED: its use in other scenes or animations.
MEASURED, frames 11334..12539: 080014C8 from LR 08001D85, 1377 calls;
080016D4 from LR 08001DF3, 1206; both with limit 48 and inside the hook.
030065A8 from LR 0808F313, 5645 calls, and LR 0808F6CD, 1204.
Other documented callers of 14C8/16D4 were not observed in that segment;
the two functions are hooked by PC regardless of caller.

MEASURED: exact captures proof-12042.png and proof-12044.png show the
character disappearing at the left margin while the background remains.
The producer period is labeled with bus.frames before execution;
the capture is labeled when the frame completes. Publication and the PPU
introduce the visible offset between rejection at 12042 and image 12044.
NOT MEASURED: an exclusive correspondence between each component and a logical
game unit; the identifiers are pointers to graphics instances.

MEASURED: final normal tests: 80 + 7 + 3 passed, 0 failed;
3 tests ignored by default. The new input-reader test passed.
Instruction and tile diagnostics retained the same final hash in additional
replays. The ROM, original .sav and game condition were not modified.
Applying a visibility patch is pending.

Previous state, superseded by the eight-unit measurement described below:
Minimal proposal to hook predicate 030062EC by PC,
active under FFTA_PATCH_SPR and only in widescreen. Preserve original
execution and its cycles; on return, use the result calculated for the
rectangle X=-extra..239+extra, keeping Y=0..159. Store input arguments
on the host for that calculation and check the ROM SHA256 and the
complete predicate contents in ROM/RAM before enabling it.
Extra=0, incorrect hash or different bytes: original result unchanged.
This changes the result seen by the game and requires approval before
implementation. The existing producer and i32 list would continue to avoid
the later rejection and the 9-bit X limit for expanded objects.
Proposed tests: both complete A/B replays, presence of the instance
12041..12046, tiles, original OAM, equivalence for 600 frames with extra=0 and
32:9/1536; also check producer 030068A4 if actual calls occur.

NOT MEASURED: no usable complete checkpoint was found to restore
CPU, RAM and peripherals. The recompiler's IWRAM snapshots are code images
and do not replace accumulated state. No checkpoint was implemented.

Update: per-unit tutorial metric
MEASURED: X hook implemented under FFTA_PATCH_VIS and FFTA_PATCH_SPR,
checking SHA256 and the ROM/RAM function. Original result with extra=0.
MEASURED: final 1201 frames: current fails in 995; proposed fails in 493.
UNRESOLVED: eight units are not published in every frame.
MEASURED: six final bodies remain in RAM but receive Y rejection before
the producer. There is also an additional Leslaie body, reported separately.
ASSUMPTION: the reference eight correspond to IDs 0..7, excluding that body.
MEASURED: extra=0 preserves all 1201 metrics and 41 hash samples.
Details, limits and complete lists in docs/ffta_character_metric.md.
The additional proposal to remove Y rejection has not been applied.

Next investigation: tiles during zoom-out
MEASURED BY THE USER: characters validated by playing in 32:9; they no longer
disappear. The character patch is retained without changes.
Earlier failures to publish eight do not invalidate that visual validation;
the definition of the eighth character and the difference with Y visibility
remain documented in ffta_character_metric.md.
MEASURED: ffta-batalla-4.input and its sidecar are not at the repository root.
Search with rg --files under E:/AI/ARC, filtered for batalla-4, found no results.
NOT MEASURED: replay 4, determinism, range list in RAM, failure frames,
BG0/BG1 writer condition and before/after captures. The recording is missing.
MEASURED by reading: InputScript v1 contains frame and KEYINPUT mask;
the recorder writes only key changes. It does not record zoom_level or extra_y.
Zoom is processed separately in play.rs through Minus/Equal. Headless replay
configure_headless_widescreen only sets extra X; it does not reproduce zoom-out.
MEASURED by reading: MAX_ZOOM=2, MAX_EXTRA_Y=320; play limits steps by
MAX_ZOOM and maximum width/height. This does not mean extra_y=320 can be reached
from 32:9 with the current configuration. The actual capture maximum is missing.
Logging added under FFTA_OAM_LOG: FFTA_VIEW when dimensions change,
with frame, zoom level, extra X/Y and view/render dimensions.
It does not change emulated state or zoom, or touch sprites or the installer.
No new BG patch is applied before measuring the cause.

Zoom and tile update, manual test pending
MEASURED: runc runner supports FFTA_ZOOM_LEVEL=n or a frame:level schedule.
32:9 levels 0..6: extra X 164,199,238,282,331,387,448;
extra Y 0,10,21,34,47,63,80. If absent, the variable preserves previous behavior.
MEASURED: complete batalla-3 replays match in the final 1201 hashes.
movement-freeze.input was constructed retaining all events and adding
12540 3ff. No additional user recording is needed.
MEASURED: matrix computed at 02005CB0, 64x64; BG0 copies a 32x32 window.
In the final sample, 108 RAM tiles versus 102 VRAM tiles; 1024/1024 VRAM words
match the source RAM crop at (33,35).
MEASURED: Thumb writer 0801AF08, caller 0801B80C. Tile producer
0801D130: 20 calls style=1, caller 0801D803, logical X/Y coordinates
and tile projection, four quadrants per tile. The flags table
02008190 contains partial neighbor marks; counting bytes does not count tiles.
MEASURED: PPU wrote only 160 rows. At zoom 6 the additional 160 rows
remained black; captures before-11347.png and frozen-13101.ppm.
IMPLEMENTED FOR TESTING: FFTA_PATCH_MOVEMENT=1 requires PATCH_BG and the exact
ROM, SHA256 of 0801AF08..0801B5D0 and equality of the 1024 values
of each VRAM plane with its RAM crop. On mismatch it uses the previous path.
The host reads full RAM planes and completes the extra rows in battle.
It does not change original game writes, their cycles, the character hook or
the terrain edge policy. It is disabled by default at this stage.
MEASURED: extra=0, 600 frames, MOVEMENT=0/1: framebuffer and OAM identical.
NOT YET MEASURED: visual success of the new patch, per-tile metric before
and after, and regression of the 1201 frames with the new patch active.
Manual test prepared at target/ffta-zoom/manual/FFTA-movement-test.gba;
its private save comes from ffta-respaldo.sav. No original save was touched.
Update: the user confirms that the tiles no longer disappear.
MOVEMENT is enabled by default, with the same ROM and RAM checks.
MEASURED: the stage retains 64x64 RAM planes at 0200B1A0 and 020091A0;
VRAM contains 64x32 crops. The STAGE patch reads those full planes only
after checking the entire crop (2048 words per layer). It does not modify the game.
MEASURED: STAGE=0/1 comparison: 1861 character frames unchanged,
including the final 1201 of the recording; 63 central framebuffer samples
and OAM unchanged. Final frame: 8 characters, no duplicate groups.
MEASURED: extra=0, per-frame comparison for 600 frames: framebuffer and OAM
identical. Captures stage-off-13101.png and stage-on-13101.png show more roof
and side architecture. Confirmation during actual play remains NOT MEASURED.
MEASURED: additional zoom rows are now drawn rather than left empty.
Input contains Keyboard and Gamepad, remapping and an Xbox layout by default.
NOT MEASURED: physical controller; not tested on the user's instruction.
Linux installer: code, script and workflow prepared; execution NOT MEASURED.
The assets folder has build references and is not removed.

Sky correction during zoom:
MEASURED: replay frame 13100, palette0 changes by line: 7D21,7DA4,7E67,
7F2A,7FCC on lines 0,40,80,120,159. The gradient is a palette raster,
not an affine or bitmap layer as previously asserted without measurement.
Additional host rows read the VBlank palette (initial blue).
They now retain the edge colors before VBlank and extend those colors
beyond the 160 lines. No stage rows are copied.
Update: extending a fixed color still left a flat band.
The host now extrapolates the gradient observed between
lines 0 and 159 per RGB555 channel; original 240x160 pixels do not change.
MEASURED: at frame 13101, rows 0/79/80/239/240/319 change RGB
from 0,0,255 to 8,74,255 to 99,247,255 to 140,255,255 without a jump at
the original screen edges. Capture sky-extended-13101.png.
MEASURED: 600 frames extra=0 framebuffer and OAM equal; 1201 tutorial
character frames unchanged relative to the validated patch.
Removed nonempty OBJ VRAM/palette/OAM heuristics: according to the user they
did not resolve silhouettes and added per-line memory scans.
A check of each OBJ piece's own palette was attempted in
widescreen: it avoids drawing a piece when its 16-color bank is
completely empty. Whether it removes the actual silhouettes is unconfirmed.
NOT MEASURED: cause of transient silhouettes; they do not appear in samples
10020,10200,10400,10600 of the existing replay. They are not claimed fixed.

Stage zoom during menus and cutscenes (2026-10-12):
MEASURED IN CODE: the full-plane path accepted only BG2/BG3 and
required both backgrounds active with identical scroll. The earlier detector also
accepted terrain in BG0/BG1, but those layers did not read the full RAM plane.
CORRECTION: identify each plane by BGCNT 4C03/4E02 and its already verified
RAM state, without depending on activation or scroll of its partner.
ROM, FFTA_PATCH_STAGE and FFTA_PATCH_BG checks are retained.
Game data and the character patch are unchanged.
MEASURED: gba-core tests: 85 passed, 3 ignored. Synthetic test of a hidden
partner, BG0/BG2 change, vertical extension and equal 600 frames with extra=0.
NOT MEASURED: actual failure replay, framebuffer/OAM hashes for 600 real frames
and 8-character metric. No .input recordings exist in the repo
or at E:/AI/ARC. Gameplay validation is required; it is not claimed resolved.

Stats panel and zoom (2026-10-12):
MEASURED IN CODE: extra rows excluded UI layers but drew OBJ.
Change: include all active text layers in extra rows as well;
UI does not repeat vertically outside its tile map. extra=0 path intact.
MEASURED: 86 tests passed, 3 ignored; test for panel at y=180 and no repetition.
NOT MEASURED: visual reproduction of the enemy panel in the user's screenshot.

Panel while hovering over units (2026-10-12):
MEASURED: derived replay zoom-no-range.input recovered from the previous repo.
That repo's ffta-batalla-3.input recording contains only its header.
Two runs from frame 0 through 13000: hash f158877f466426fb equal.
Only target/ffta-stats/repro.sav, a backup copy, is used.
MEASURED: Ritz's panel is not a BG: producer 080014C8, list 083916FA,
four semitransparent pieces, tiles 1F0/200/208, OBJ palette 11.
All pieces and tiles exist. BLDCNT=3F43, BLDALPHA=090E: the background
blends with terrain and looks partially absent; pieces are not rejected.
Presentation change: FFTA_PATCH_STATS=1 by default makes only
that background opaque, identified in the host list of the exact ROM and verified functions.
It does not change OAM attributes or game logic; extra=0 retains the original path.
MEASURED: 87 normal tests and 3 with local ROM pass. Frame 13000 changes
2558 pixels, all within x600..687/y184..231; exterior identical and OAM equal.
MEASURED: framebuffer and OAM equal for 600 frames with extra=0, STATS=0/1.
Captures hover-13000.png and after-13000.png in target/ffta-stats.
NOT MEASURED: red enemy panel; the constructed replay did not show it.
The metric for this derived route has frames with fewer than 8: the constant
8 test is not considered passed and the character patch is not touched.
MEASURED: STATS=0/1 comparison over 1201 frames (11800..13000): metric
identical throughout. 297 frames have fewer than 8 in both versions of this
derived replay with the recovered native library; this is not a new regression.
The complete list is in target/ffta-stats/character-failures.json.
Windows Installer/payload and Linux TAR built and updated, without commits.

Correction to the previous stats attempt (2026-10-12):
The user confirms that removing semitransparency did NOT fix the clipping.
The original semi mode is always RESTORED; the opaque attempt is discarded.
MEASURED in VRAM: the four background pieces contain a curve with
transparent pixels: coverage of 67 pixels on row 104 and 88 on row 127.
This differs from the action-menu BG layers.
Host change: fill only transparent texels of those identified pieces,
using the live color index of rectangular tile 208.
Palette, lettering, OAM attributes, priority and original alpha are preserved.
FFTA_PATCH_STATS=0 keeps the original shape; 1 fills the background in widescreen.
This is not recovery of missing sprites or tiles: it completes panel background
geometry, limited to the verified template without touching the ROM.
MEASURED: 88 tests passed, 3 ignored. Test of gaps, live color, semitransparency
and extra=0. New actual 600-frame comparison: framebuffer/OAM equal.
NOT MEASURED: red panel; user visual validation remains pending.
MEASURED: complete replay through 13000 with the new change. Background without gaps
and alpha preserved: cap-after-13000.png. 217 pixels are filled, all
inside the panel; outside the panel and OAM attributes are identical.
MEASURED: character metric identical for all 1201 frames of STATS=0/1.
The 297 frames with fewer than 8 in the derived route remain unchanged.
The PowerShell command returned 1 despite reaching frame 13000 and producing
the dumps; the log records NativeCommandError for stderr messages.
Subsequent verifiers exit with code 0; that return is not concealed.
Windows and Linux built and updated, without touching original saves.

Correct target for the stats panel (user clarification):
The semicircle was NOT wrong. The problem is that the panel does not reach the edge.
Filling its transparent texels is REVERTED. Original shape and alpha.
New STATS=1: only rectangular pieces of tile 208 are extended
to the wide framebuffer edge. H-flip selects left or right.
Its live outer column is repeated, with its texture, palette and semitransparency.
Layers, text positions, curve, OAM and character logic remain intact.
MEASURED: 88 tests passed, 3 ignored. Test of both orientations, unfilled
curve, alpha and no extension with STATS=0 or extra=0.
MEASURED: actual 600-frame comparison extra=0: framebuffer/OAM equal.
MEASURED: reconstruction of the private dump in edge-preview.png shows
the transparent band through the right edge, with the curve retained.
NOT MEASURED: new complete 1201-frame replay, red panel and user gameplay
validation. The preview reconstructs the dump; it is not a new capture.
Windows and Linux built; Installer/payload and TAR updated, without commits.

Native pause menu adapted from the user's HTML (2026-10-13):
Parchment style, cream gradients, brass edges, ornamental corners,
leather/gold selection and original icon drawings. No HTML engine,
network, new dependencies or bundled web fonts or icons.
The native pixel font is retained: Cinzel/Inter and Web Audio effects
are not transferred literally. Actual av.cfg/input.cfg values are preserved.
Input opens Keyboard/Gamepad tabs and preserves the selected device.
Lists scroll by keyboard/controller/wheel; the Back button remains visible.
Video dropdowns accept/cancel; segmented FF is clickable.
Exit asks for confirmation with No selected by default.
MEASURED: 42 tests passed: input-config 12, menu 19, recomp 11.
Tests cover dropdowns, remapping targets with scroll, tabs, FF and safe exit.
MEASURED: native previews generated/reviewed for main, gamepad, FF,
exit and selector. Windows/Linux release builds and payload/TAR updated.
NOT MEASURED: gameplay interaction with window/physical controller and GPU
effects on the style; no game was started and original saves were untouched.
HTML reference in target/pause-menu/reference.html; previews in that folder.

<a id="2026-10-13---menu-moderno-e-iconos-de-mando-completos"></a>

## 2026-10-13 - Modern menu and complete controller icons

Menu rendered on a canvas independent of the GBA, without altering the game
framebuffer. Cinzel/Inter with antialiasing and OFL notices in Licenses/fonts.
Vector icons for A/B/X/Y, LB/RB, LT/RT, View/Menu/Guide, L3/R3,
the four D-pad directions and directions of both sticks.
Mice use canvas coordinates; GPU and CPU fallback present the same menu.
Game bindings and patches preserved. Preview: target/pause-menu/gamepad-modern.png.
Physical controller test: NOT MEASURED.

<a id="2026-10-16---tinycc-en-memoria-para-windows-y-linux"></a>

## 2026-10-16 - In-memory TinyCC for Windows and Linux
MEASURED: same TinyCC as Android, source 43c7708b85681a2fd4451c8a541af4494a8919b2.
MEASURED: complete EUR build 3.611 s; USA 4.706 s; previous USA GCC 116.634 s.
MEASURED: EUR cache 0.079 s; framebuffer/OAM equal to GCC over 600 frames for both ROMs.
MEASURED: USA extra=0 identical with patches off/on over 600 frames.
MEASURED: 141 existing tests and 2 additional TinyCC tests passed.
MEASURED: 12540-frame replay equal to GCC; old metric counts 4 in both, does not validate 8.
MEASURED: Windows and Linux built; GCC is not packaged. TinyCC includes its exact source and license.
NOT MEASURED: Linux execution; this host has no WSL installed.
Details and limits in docs/desktop_tinycc.md. No changes to sprites, backgrounds or gamepad.

<a id="2026-10-16---dialogos-anclados-al-borde-y-tamano-independiente-del-zoom"></a>

## 2026-10-16 - Edge-anchored dialogue with size independent of zoom
MEASURED: EUR snapshot separates BG2 text, BG3 background and five 8-bpp portrait OAM pieces.
MEASURED: 32:9 x1/x2 previews show text, background and portrait together at the edge, without extension.
MEASURED: 99 core tests passed; 600 frames extra=0 with layout off/on give identical framebuffer/OAM.
Windows built in recomp-dialogue-layout.exe and copied to the usual recomp.exe and payload.
NOT MEASURED: gameplay validation; no USA visual validation or other dialogue formats are claimed.
Future Settings point: MemMap.ffta_dialogue.style (scale, offset_x, offset_y), with no new menu.
A/B: FFTA_DIALOGUE_LAYOUT=0/1, default 1. Details in docs/dialogue_layout.md.


2026-10-16: Text Bubbles submenu under Video: persistent size and backing length.
Exact desktop zoom/crop now drives uniform dialogue scale. Host-list objects cannot
be treated as portrait pieces. Parked sentinel filter also covers horizontal-only wide.
MEASURED: 157 tests passed; EUR extra=0 framebuffer/OAM identical for 600 frames.
NOT MEASURED: visual elimination of startup flashes and appearance on the user display.


2026-10-16 portrait follow-up: corrected host-list portrait exclusion introduced
in prior dialogue update. Host portraits now share text/backing placement and scale.
MEASURED: both edges x 3 sizes x 7 zooms, host and OAM rendering identical with
expected portrait bounds; 158 tests passed, 4 ignored. No game logic or saves changed.


2026-10-16 independent dialogue UI layer (desktop, first migrated profile).
Native BG2/BG3 and portrait capture; world pass excludes them. GPU draws UI once
in physical window coordinates, independent of world zoom. Software and pause
paths included. FFTA_UI_LAYER=0 restores the preceding layout for comparison.
MEASURED: 189 suite tests pass; separate RTX 5050 offscreen GPU/CPU comparison passes.
MEASURED: EUR and USA extra=0 framebuffer/OAM equal for 600 frames each.
MEASURED: real saved snapshot UI data and 72,855 opaque output pixels unchanged
at x1, x1.122462 and x2. Captures target/ui-layer-after-*.png.
NOT MEASURED: live user transitions and 8-character replay metric in this change.
Other menu profiles remain on existing paths; Android and installer code unchanged.


2026-10-16 dialogue continuation marker: measured USA replay frame 2141,
OBJ attrs 803B/0088/2122, tile 0x122, 8x16, palette 2, priority 0. Its graphics
match the existing EUR snapshot. Marker now shares dialogue layer, retaining
bounce and disabled state; tested with raw OAM and host replacements.
Text Bubbles moved to root directly after Fast Forward; eight rows fit.
MEASURED: reconstructed before/after captures at x1/x2 confirm attachment.


2026-10-16 inward Size pivot: verified lower dialogue clipping in open window.
Native backing/portrait bounds choose upper/lower screen-edge anchor; Size grows
uniformly toward the middle, leaving space for the continuation marker without
following its bounce. Measured same live scene reconstructed at 110/200 percent.
100-percent image unchanged; EUR/USA extra=0 framebuffer/OAM equal for 600 frames.
192 tests pass, 5 ignored. Windows build: target/release/recomp-bubble-inward.exe.
User's existing game process was not closed; its in-use executable may remain old.

2026-10-16 independent action and hover-stat panels:
MEASURED: actual EUR open-actions snapshot BG2=5C08/BG3=4A09; native left
information/portrait and right action list extracted into two separate layers.
Stats recognized from complete four-piece palette-11 OAM backing; glyph palette
12 retained. Native alpha blends against raw top BG, not world OBJ or an already
blended BG. Reused buffers include serial and parallel zoom rows.
Panels anchor to their screen corners and Size grows uniformly inward; world
zoom does not affect native UI size. Length remains dialogue-only.
MEASURED: actual EUR action and USA stats snapshots reconstruct original hardware
framebuffer with changed_pixels=0, including Size=200 percent output captures:
target/panels-actions-after-200.png and target/panels-stats-after-200.png.
MEASURED: 196 tests pass, 5 ignored; separate RTX 5050 GPU/CPU test passes.
MEASURED: USA and EUR extra=0 framebuffer AND OAM equal for 600 frames each.
NOT MEASURED: new live user transitions and 1201-frame eight-character metric.
No sprite producer, installer logic, Android source, ROM or original save changed.
Windows executable: target/release/recomp-panels-corners.exe.
FFTA_UI_LAYER=0 retains old world UI for A/B comparison.

2026-10-16 panel Length: actions, portrait/info and hover stats now use the
persisted Text Bubbles Length for horizontal scale about their existing corner.
Size remains uniform scale; Length leaves vertical dimensions unchanged.
MEASURED: 47 screen/recomp tests pass, 1 ignored; actual saved USA stats and EUR
actions reconstruct native UI with zero changed pixels at Length 150 percent.
MEASURED: USA/EUR extra=0 framebuffer and OAM identical over 600 frames each.
Windows built: target/release/recomp-panels-length.exe; payload updated.
Captures: target/panels-actions-length150.png, target/panels-stats-length150.png.
NOT MEASURED: live user check. No sprite/game logic or installer source changed.

2026-10-16 Length correction: removed horizontal texture scaling for compact
panels. Capture native backing independently from glyphs/portraits/cursor.
Stats backing uses palette-11 pieces, excluding palette-12 glyph pieces.
Length translates foreground by an integer native offset toward screen center;
only the backing is extended using an interior column, keeping the outer cap.
Size still scales both axes equally; GPU/software/pause use the same packed UI.
MEASURED: complete test suite passes, including rigid glyph translation, fixed
outer cap and exact Length=100 percent pixels. Actual saved EUR action/USA
stats native reconstruction: changed_pixels=0. Length=150 percent captures:
target/panels-actions-length-content150.png and target/panels-stats-length-content150.png.
MEASURED: USA/EUR extra=0 framebuffer AND OAM identical for 600 frames each.
NOT MEASURED: live user transitions and new eight-character replay metric.
Windows: target/release/recomp-panels-length-content.exe; payload updated.
No guest logic, sprite producers, installer source or original saves changed.

2026-10-16 panel border completion:
MEASURED: read-only live EUR snapshot PID 9292 at Length=200 percent shows action
header/footer and info strip cut short. BG2 action frame tiles 005..00A and
0A8..0B9 are decoration, not ordinary action labels. Include these in backing.
Background-only capture now ignores foreground world OBJ: portrait silhouettes
must not punch holes into the blue plane used for Length completion.
Use a fixed flat-side stretch seam, not one following each row's curve.
Separate native stats capture excludes legacy EUR host gap padding; original
cap geometry is retained in both orientations. Guest sprite producers unchanged.
MEASURED: 201 tests passed, 5 ignored; tests cover action border retention,
background under portrait, mirrored rounded caps, and legacy host padding.
MEASURED: saved native UI reconstruction zero changed pixels, including Length
100/200 and zoom x1/x2, Size100/150 percent. Before/after captures:
target/panels-current-window.png (actual original window),
target/panels-borders-actions-before.png / panels-borders-actions-after.png,
target/panels-borders-stats-before.png / panels-borders-stats-after.png,
target/panels-borders-actions-zoom2-size150.png (snapshot reconstructions).
MEASURED: USA/EUR extra=0 framebuffer and OAM identical over 600 frames each.
NOT MEASURED: actual red enemy hover scene after fix; cap orientation regression
uses generated mirrored fixtures. No new 1201-frame character metric run.
Windows build target/release/recomp-panels-borders.exe; payload updated.
No installer source, original save, ROM or Android source modified. No commits.

2026-10-16 action-menu jitter:
Cause: full UI pixel bounds included animated OBJ cursor. Its varying left edge
changed Length displacement for the entire panel. Use static backing bounds for
Length distance and action panel anchoring. Cursor animation stays in foreground.
MEASURED: saved real EUR menu snapshot with only cursor X changed to 171/173/175/177:
all UI rects identical; packed menu excluding cursor animation has identical SHA256
084a981ed32e54d7c077a13699e921167230f687534350701c698d120ddea7a7.
Captures target/menu-jitter-171.png through menu-jitter-177.png.
MEASURED: 202 tests pass, 5 ignored; new regression includes cursor blink and Length
100/125/150/200 percent. USA/EUR extra=0 framebuffer AND OAM equal for 600 frames.
NOT MEASURED: live user validation after fix; live read-only sample had no matching
active menu cursor, so saved snapshot variants are a controlled reproduction.
Windows: target/release/recomp-menu-stable.exe; payload updated.
No installer source, guest producer, ROM or original save changes; no commits.

2026-10-16 target attack forecast in native screen-space UI:
MEASURED: captured live EUR window PID6968 and six graphics/RAM regions read-only.
BG attacker signature 5C08/4A09 coexists with target OBJ backing 1F0/200/208.
Bug: begin_dialogue_frame detected Stats then unconditionally replaced it with
Actions. Target OAM remained in zoomed world. Preserve detected Stats beside BG
attacker info. Forecast glyphs share bank11: backing-only capture must identify
backing tiles, not the entire bank (glyph 228/230/238 must stay foreground).
MEASURED: real EUR snapshot UI reconstruction changed_pixels=0 at zoom x1/x2;
UI layout rect identical, native target UI byte hash identical:
80F72591E91B9BC939864EFCEC4B82A1FEBFF6647AB755651B835EC5E3148841.
Captures target/forecast-live-window.png (before, actual window),
target/forecast-after-x1.png / forecast-after-x2.png (reconstructions),
target/forecast-after-size125-length200.png (Size/Length verification).
MEASURED: 203 tests pass, 5 ignored, including both regional profiles, two zooms,
combined attacker BG and target OBJ, bank11 glyph exclusion, original OAM intact.
MEASURED: USA/EUR extra=0 framebuffer and OAM equal for 600 frames each.
NOT MEASURED: actual USA target screenshot and live user verification after fix;
no new 1201-frame eight-character metric. No guest producer changes.
Windows target/release/recomp-attack-forecast.exe; payload updated.
No installer source, Android source, ROM or original save writes; no commits.

2026-10-16 experimental mouse, EUR language selector:
Read-only investigation plus opt-in semantic target layer (FFTA_MOUSE=1).
Five flag/text OAM groups and quill signature gate interaction; exact EUR SHA256
and short function bytes must match. Pointer0201C3D4 -> structure0200F3D4;
selection+2=0200F3D6, Thumb write0813FFCA. No guest memory/function modifications.
Hover navigates through ordinary Down pulses; click confirms once with A.
Targets follow existing viewport/zoom inverse transform. Per-frame virtual keys
recorded for replay. Manual buttons, pause/focus/scene loss cancel automation.
MEASURED: all five target requests reached; one confirmation edge;206 tests pass,
5 ignored.700-frame interpreter and two native replay PPM files byte-identical,
hash d17fc597c1fb9325. USA/EUR extra=0 framebuffer/OAM equal for600 frames.
NOT MEASURED: physical mouse in user window, touch, other game scenes. Disabled by
default; no original save writes, ROM distribution, installer source or commits.
See docs/ffta_mouse_language.md. Windows target/release/recomp-mouse-language.exe.

2026-10-16 mouse direct selection:
User requested skipping intermediate languages. Measured direct selector-byte
commit updates the original quill/text palettes in two frames with no intervening
languages. Controller now commits just the validated selection byte between
emulated frames, then original game redraws; A confirmation remains one edge.
Exact EUR hash, short original bytes and complete current scene/pointer bounds
required. Recording adds FRAME mouse:INDEX; frames/runc/verify use guarded replay
and fail on scene/ROM mismatch. Historical key-only inputs remain compatible.
MEASURED: all25 source/target pairs tested; real controller reaches4,2,0,3,1 with
no directional keys, no intermediate quill or palette state; one confirmation.
MEASURED:208 suite tests pass,5 ignored. Interpreter/native full PPM byte equality
at all five destinations304/308/312/316/320;370-frame final also identical to
repeated native replay, hash d17fc597c1fb9325. Private target/mouse-direct-* files.
MEASURED: USA/EUR extra=0 framebuffer AND OAM equal over600 frames without requests.
NOT MEASURED: user's physical mouse after direct-selection change. Experimental
FFTA_MOUSE=1 still required; only EUR language selector supported.
Windows target/release/recomp-mouse-direct.exe; payload updated. No original save,
ROM, installer source, Android or sprite producer changes; no commits.

2026-10-16 reusable invisible mouse grid:
Implemented interaction_grid.rs:8x8 logical cells across viewport, exact target
bounds within shared cells, EventId(context,option), layers/modal blockers,
cell tracking and hover entry/exit/click. Unchanged targets reuse the index.
EUR language adapter now routes through the grid; direct state commit unchanged.
MEASURED:214 tests pass,5 ignored. Exact reference match at153600 test positions;
all38400 native language pixel centers match the previous target rectangles.
Actual EUR run:600 cells,1 build, five direct selections, one confirmation.
Five rendered images and input events byte-identical to direct-rectangle build.
Native/interpreter replay equal,370-frame hash d17fc597c1fb9325.
USA/EUR extra=0 framebuffer AND OAM identical over600 frames each.
NOT MEASURED: user's physical mouse with this build, new menu event adapters, any
speed gain over the five-option prototype. Main menu and battles not started.
Docs ffta_interaction_grid.md. Windows target/release/recomp-mouse-grid.exe.
No original save, ROM, installer source or Android modifications; no commits.

2026-10-16 EUR main menu mouse:
Reused invisible grid for Press Start, New Game, Saved Game, Color Mode,
Language, name keyboard/categories/Delete/OK and name Yes/No. Direct verified
selection bytes and original confirm/back keys; queued clicks cannot cross
menu contexts. Scene/ROM checks fail closed. FFTA_MOUSE=1 remains experimental.
MEASURED: actual full-boot click scripts traverse the front-end, select TV,
Resume/Load and start New Game; letters/categories/name confirmation exercised.
Native1490-frame click replay equals interpreter pixels. USA/EUR extra=0
framebuffer AND OAM equal previous600-frame references.
NOT MEASURED: populated save-file slots/resume (all available saves blank), physical
mouse acceptance, USA mouse front-end and battle mouse controls.
Windows target/release/recomp-mouse-main.exe; docs/ffta_mouse_frontend.md.
No original save/ROM, installer source, Android or sprite-producer changes.
No commits.

2026-10-17 experimental battle hover:
Added ffta_battle_mouse.rs: reads logical stage dimensions, bounds, height/flags,
range styles and camera projection. Mouse movement queues a logical destination;
original cursor handler commits it. Left click confirms via original A after
commit; right click B. Exact USA/EUR ROM profiles and short code checks required.
FFTA_MOUSE=1 opt-in; action menus and unsupported cursor modes stay excluded.
MEASURED:196 existing tutorial cells,20 range-style1; original projection matches
196/196 and range query256/256. Real USA full replay: three direct targets stay
exact in interpreter and Windows native32:9 zoom6; no intermediate arrow walk.
Single-frame wake pulse fixes an observed extra-step failure in the first build.
MEASURED:221 tests pass,5 ignored; USA/EUR600-frame extra=0 framebuffer/OAM match.
Character metric11340..12540:1201 frames, exactly8 present, no duplicates.
EUR isolated original-function test passes on copied RAM; original scripted
mode4 was explicitly overridden only in that test, not in runtime behavior.
NOT MEASURED: full EUR manual hover, all stages/slopes, battle action menu mouse.
Windows target/release/recomp-mouse-battle.exe. docs/ffta_mouse_battle.md.
No original save/ROM, installer source, Android or sprite-producer edits.
No commits.

2026-10-17 battle input follow-up, awaiting user gameplay test:
Live vertical lists use their instance count/cursor rather than option names.
Original navigation commit USA080287A0 / EUR080299DC selects the requested row.
MEASURED: USA replay jumps0->3->0, opens Action and Snowball; first one-frame wake
was missed by the menu polling cadence and replaced with wake-until-commit plus
suppression of repeated steps. Exact ROM and short function-byte checks gate it.
Follow-up adds plain left=A/right=B outside semantic targets, including dialogue
and forecasts, and fixes the field gate when unused BG layers are disabled.
Wait-facing arrows adapt mouse direction and queue original direction then A.
Name typing supports ASCII letters/digits, Backspace and Enter; original glyph
selection and keys insert the name. EUR3/USA2-page profiles are checked.
MEASURED:10 battle-input tests and9 frontend/name tests pass; earlier filtered
FFTA suite37 pass,4 ignored. NOT MEASURED: user gameplay acceptance of follow-up.
Windows target/release/recomp.exe. No installer/save/ROM edits or commits.

2026-10-17 desktop mouse follow-up (experimental, off by default):
Input now has Keyboard / Gamepad / Mouse and Keyboard. The last uses its own
persisted ten-button profile (WASD, Q/E), preserving the Keyboard bindings.
The runtime and launcher select the active profile; assets/cursor.png is original.
MEASURED: Do it/Cancel is a separate two-button handler, USA08029350/EUR0802A58C.
The adapter reads its live button records and drives original input, including
deferred A after the requested row. A controlled call selects Cancel successfully.
MEASURED: world-map cursor logical stores USA080375CE/080376A2 and EUR08038B3A/
08038C0E, reached through the original input routine. Native replay matches the
interpreter framebuffer; requested native screen points remain exact after release.
Town window lists and nested shop/pub text lists use live row counts/geometry.
Two cursor styles are handled. Polling lists release each direction after an
observed step because the pub list consumes edges instead of repeating held input.
MEASURED: extra=0 framebuffer and OAM identical in600 frames against the previous
executable; tutorial regression8 present/8 drawn in all1201 final frames, no duplicates.
NOT MEASURED: every mission/inventory paging state and manual clicks at every scale.
Pending: reported end-turn facing issue and repeated thunder/effects in32:9.
Android port is deferred; mouse remains off by default there. No installer,
original ROM/save or sprite/PPU renderer edits. No commits. docs/ffta_mouse_world.md.
