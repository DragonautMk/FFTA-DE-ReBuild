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
source assertions, not current real-scene results. The save-screen exception
and its different desktop/Android callers are described below; the complete
motif check is not a universal precondition for every background-row call.

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

## Follow-up: save-screen rendering is a separate path

The upstream `repeating_menu_background` detector checks the EUR profile,
display registers and, for Save/Resume Battle and Party, every entry in the
repeating tilemap before setting `menu_pattern`. However,
`render_menu_background_row` also accepts either EUR save callback
`0x0813dd75` or `0x0813d049` when `menu_pattern` is false. In that case it
renders BG0 and substitutes the palette backdrop for transparent pixels; the
function does not itself repeat the complete-map validation. An upstream
synthetic test named `original_pattern_can_render_before_cached_menu_flag_is_ready`
explicitly asserts the callback fallback using a blank synthetic machine. The
test's source was inspected, not executed for this follow-up. The exception
appears intended to let the save UI paint before the cached pattern flag is
ready; whether a particular transition can expose stale or foreground pixels
is not established by this source trace.

The distinction matters at the call sites. Desktop `display_frame` asks for
extra rows only when `menu_pattern` is true; separately, when its save screen
is active, `autosave::paint` calls the row renderer and copies the central
240 pixels of 150 rows before painting its card UI. Upstream Android has a
separate `gba-core` copy with the same callback exception; its active save
screen calls `save_states::paint`, which copies rendered background rows across
the whole output. Android's `render_frontend_background_row` also accepts the
save callback. These are separate frontend uses, not proof that each paints an
incorrect image in an actual scene.

Consequently, a future fork adaptation must not infer that every upstream
save-screen background has passed the full motif check or copy the callback
exception without its ownership, transition and rendering context. Test
callbacks with an invalid or changing map, stale ownership and enter/exit
frames, alongside valid backgrounds, native center pixels and mouse alignment.
The inspected upstream files came from the previously extracted pinned source;
desktop `ppu.rs` SHA-256 was
`f8d27cf2bdb476e5e18930d9d706c9c6783278972bce5215f6c5d346534f5ac3`.
The current fork HEAD is `c680b29517ac3dc99890a9d856122289c97f623f`;
the relevant runtime trees have no tracked committed changes since the fork
revision used in the initial audit. This follow-up is static: no game, build,
test, real transition, native Windows or Android device result is claimed.
