# U31-05: Widescreen original-game menus

Status: **partial source comparison; ADAPT only a verified original-game
scene**. Upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` is compared with
fork `decomp` `83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02.
The tag distinction is documented in [U31-02](U31-02-scrollbars.md). No
build, screenshot, game or new test was run.

Upstream `gba-core/src/ppu.rs::repeating_menu_background` identifies particular
EUR menu backdrop patterns: Clan's complete illustrated backdrop, Save/Resume
Battle and Party gallery tilemaps. For the latter two it verifies the complete
repeating tilemap; other register patterns are rejected. `begin_dialogue_frame`
publishes `menu_pattern`, and `render_menu_background_row` draws the backdrop
for host rows without repeating foreground windows/objects. Desktop
`recomp/src/play.rs::display_frame` invokes it for extra rows when zoom rows
are unavailable; upstream Android save rendering also calls the background
routine. A synthetic upstream test checks motif repetition at selected widths
and rows, rejects an altered map and checks VRAM/OAM preservation. These are
source assertions, not current real-scene results.

The fork has the Clan Atlas host pause menu with its own native adaptive layout,
minimum 480×360 and its own input model. Its original-game renderer is a
separate surface. The fork does not contain this upstream
`repeating_menu_background`/`render_menu_background_row` path; it already has
widescreen/zoom and distinct PPU edge handling, so this absence does not prove
all equivalent scenes have black borders. Existing Atlas source tests cover
multiple layouts including 480×360, 640×480, 854×480 and 1280×720, but their
presence does not establish original-game backdrop or ultrawide acceptance.

Select one exact original-game EUR scene and capture the central viewport and
margins before proposing a minimal pattern-specific adapter. Establish the
backdrop signature and ownership, clipping, zoom and native/UI input mapping;
never repeat labels, sprites or unrelated foreground as background. Preserve
the Atlas and its hitboxes, separate desktop from Android, and do not generalize
the EUR pattern to USA without region-specific evidence. Synthetic positive and
negative motif tests, exact central pixels/state/clock, frames during scene
transitions, multiple aspect ratios and inspected GUI captures are required.
No cross-region, native Windows, 32:9 or gameplay acceptance is claimed here.
