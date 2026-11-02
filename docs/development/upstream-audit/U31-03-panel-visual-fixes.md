# U31-03: Original-game panel visual fixes

Status: **partial source comparison; ADAPT only after a reproducible defect**.
Compared upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` with fork
`decomp` `83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02. The
upstream tag and comparison target differ; see [U31-02](U31-02-scrollbars.md).
The initial audit used a bounded synthetic source comparison. A subsequent
isolated EUR action-scene capture is described below; it is not an application
GUI or device acceptance.

## Bounded candidate: animated panel geometry

Upstream `crates/gba-core/src/ffta_dialogue.rs` adds
`pack_panel_with_bounds`: an optional complete owner rectangle controls panel
extension width, backing pivot and available translation space, even when the
currently visible backing is only a partial reveal. `PackedLayer` keys its
cached output by exact pixels, backing, style and the optional geometry.
`crates/recomp/src/play.rs` supplies a fixed action HUD rectangle and derives
other geometry from captured UI scene nodes; `ffta_ui.rs` and `ppu.rs` supply
scene roles and native bounds. The synthetic upstream test
`animated_reveal_cannot_change_length_translation_or_cache_geometry` exercises
changing reveal widths with a constant explicit owner rectangle. This is a
source-level candidate, not evidence that the fork exhibits a visible bug.

The fork's `pack_panel` derives its rectangle from currently visible backing,
falling back to visible foreground; its existing test
`animated_cursor_cannot_change_panel_bounds_or_length_displacement` addresses a
moving cursor with a **stable** backing. The fork also has a deliberately fixed
Pub layout and a `pub_menu_mouse_keeps_fixed_right_origin_through_animation`
test. These are related protections, not proof about a shrinking backing in all
other scenes. `crates/recomp/src/ui_pixels.rs` already caches exact input and
packing settings, assigns output revisions and is used by the renderer. Its
existing accepted evidence is in
`docs/changes/2026-10-21-evaluate-ui-packing-input-cache.json`: the record
includes an action-menu 150% GUI check within its stated limits. Do not import
a second cache or rerun that accepted check merely to reconstruct context.

The optional upstream geometry also affects hit testing and scene ownership;
copying the packer alone would not establish correct rendering/input alignment.
The fork lacks the same `ffta_ui` scene abstraction and already has independent
panel and window classification, diagnostics, ROMDATA and renderer changes.
A fixed owner rectangle is safe only when the chosen fork scene supplies a
verified, stable rectangle for its actual producer; never apply upstream
coordinates generically across ROMs or menu families.

## Proposed next unit

Choose one action or dialogue panel and first obtain a bounded before/after
fixture demonstrating that its visible backing contracts while the stable owner
remains valid and that length greater than 100% causes an observable placement
defect. Compare source-frame pixels and the composed GUI, including corresponding
mouse coordinates. If that defect is absent, record the negative result and
select another distinct fix rather than importing this one.

If reproduced, propose an isolated geometry input integrated with the fork's
existing revision cache, preserving its exact invalidation rules and unchanged
output when geometry is absent. Synthetic tests should cover both anchors,
multiple reveal widths, geometry-only changes, alpha/foreground, missing or
invalid owners, empty/disappearing panels and relevant lengths. Acceptance
also needs a targeted real scene, screenshots at multiple frames and zooms,
input/paint alignment, equal guest state/frame/clock and native/interpreter
checks where affected. EUR and USA require separate evidence; native Windows is
not established by Linux or Wine. Do not conflate optional host motion with
this geometry issue; see [U31-04](U31-04-ui-motion.md).

Remaining work: other text bubble/window visual changes, actual scene
reproduction and platform results. Other changed upstream PPU code was not
exhaustively audited here.


## Synthetic comparison: changing backing width

On 2026-11-02, a standalone `rustc 1.94.0 --test` harness extracted the exact
`panel_bounds`, `pixel_bounds` and `pack_panel` functions from the fork and
the corresponding functions plus `pack_panel_with_bounds` from pinned
upstream. The harness supplied an equivalent minimal `UiPixel`/canvas shim; it
did not compile either full crate or its callers. Fork source SHA-256 was
`a9ab7848e08a51547b479d98367e8b4161b5d438c30ea42e46637d67c90c8b6f` and
upstream source SHA-256 was
`d8ce3227c93dec218462d4dc657eca40021eedc754fe23f5c28c1048c6e10cc2`.

The fixture uses a 480×160 transparent canvas, a 48-pixel-high solid backing
that reveals widths 8, 32, 64 and 88, one contrasting glyph and length 150%.
The assumed owner is `[0,104,88,152]` for a left panel or
`[152,104,240,152]` for a right panel. It passes that owner explicitly only to
the upstream function; the current fork function has no owner parameter. The
reported positions are native X, and each output contained exactly one glyph:

- Left anchor: fork `8, 20, 36, 48`; upstream explicit owner `48, 48, 48, 48`.
- Right anchor: fork `231, 219, 203, 191`; upstream explicit owner
  `191, 191, 191, 191`.

The one synthetic test passed, confirming that these modeled inputs lead to
different translations. The temporary generated source and binary are
`/tmp/ffta-u31-panel-comparison.rs` and `/tmp/ffta-u31-panel-tests` while
available; source SHA-256
`157bc48f114b74ba2b95f5215f96ec088d10c0620790822ad44b62f85166593b`. The
compiler reported two unused-item warnings in the isolated shim. The stable
owner rectangle is an explicit fixture assumption: no evidence here shows
that a real fork scene retains that owner during the reveal, that its foreground
is visible then, or that an upstream replacement is appropriate. A
scene-specific screenshot/input comparison remains necessary before proposing
a runtime change.


## Follow-up: fork action-panel owner and prior scene evidence

`ppu.rs::begin_dialogue_frame` clears the extra panel kind and bounds at each
frame. For a recognized battle action layout without an overriding Stats or
Notice panel, `action_menu_bounds` scans BG2 for a complete header and footer;
the result becomes `extra.bounds`, with `[160,0,240,160]` as fallback when the
full pattern is missing or ambiguous. The recognized rectangle identifies
layer membership in `UiPanelRow`; it is not passed to the fork's `pack_panel`.
`play.rs::refresh_ui_layers` packs the Actions layer using the visible
backing, and `ui_layout` and `game_mouse_aim` derive corresponding bounds
from current pixels/backing. Thus the fork has a real recognized owner when
the full frame is present, but this source trace does **not** show that the
same owner is available or stable throughout an entry/exit reveal. Blindly
using the fallback as an authoritative rectangle could misplace content.

The existing [variable action menu record](../../ffta_action_frame_width.md)
reports an EUR Lester/Giza scene with a recognized header/footer and an
observed long-name width; its synthetic checks cover four static widths and
framebuffer/OAM preservation. The older cursor-jitter evidence in
`docs/ffta_estado.md` section “2026-10-16 action-menu jitter” changes cursor X
while keeping backing geometry stable. The accepted October 21 input-cache
record documents a 100% to 150% style change on an action-menu snapshot. Its
retained `target/desktop-perf/ui-input-cache-style/style.py` captures before
and after with a one-second wait, rather than a series of reveal frames. These
are distinct checks and do not validate the synthetic shrinking-backing
assumption or mouse alignment throughout entry. The older named action and
jitter PNGs were unavailable locally when checked; their documented accepted
results remain historical evidence, not a reason to recreate them.

**Next evidence gate:** identify an already owned, explicitly isolated action
scene with a documented input transition and compare a bounded sequence of
composed frames and owner/backing/foreground bounds at length 100% and 150%.
Record exact scene/ROM identity and input, and check window/ownership fallback
transitions and pointer targets. If no such fixture is available, leave the
claim unverified; do not infer a defect from the synthetic rectangle or
recreate private data merely to restore retired artifacts. Only then decide
whether a fixed, cached or otherwise derived owner is warranted, taking the
fork's existing UI cache revisions, ROM guards and guest state into account.
This inspection did not run a game or mutate source, saves or retained data.

## Follow-up: observed EUR action-panel contraction and reveal

On 2026-11-02 a release-mode Linux host probe used an existing isolated EUR
action snapshot and ROM, verified the snapshot preview and payload, and ran a
bounded sequence of B, release, A and release for 72 guest frames in the fork
interpreter. The snapshot preview shows the original battle action menu. With
the side extension set to 22 pixels and separate UI capture enabled, the
recorded action backing contracted and reappeared. This is a new observation,
not a repeat of the previously accepted static style GUI check.

For example, during closing, the backing's native X bounds changed from
`[176, 240]` to `[192, 240]`, `[208, 240]`, `[224, 240]`, then empty. During
opening they returned through `[224, 240]`, `[208, 240]`, `[192, 240]` and
`[176, 240]`. Foreground pixels were present during some partial widths. The
`extra.bounds` owner was not a constant complete rectangle: it followed the
changing frame with a one-frame offset in observed samples, and temporarily
fell back to `[160, 0, 240, 160]`. Treating that fallback or the current owner
as a proven stable original rectangle would therefore be unsafe.

For each captured frame the same fork `pack_panel` inputs were processed at
100% and 150% length. At one closing sample with backing `[192, 240]`, the
packed visible X bounds were `[192, 240]` and `[168, 240]`; at the next
sample they were `[208, 240]` and `[192, 240]`. A second bounded replay was
used only to save selected layer rasters, and all 72 per-frame metadata records
matched the first corrected run. The sampled 100% and 150% layer images were
inspected on a fixed black diagnostic background. These are packed layers,
not the composed application window: they establish changing input geometry
and packing, not a visible application defect, equivalent cursor hit testing
or an approved fix. The foreground itself also changes as the game reveals
the panel, so its changing bounds cannot alone identify a misplaced glyph.

The first setup used no side extension; the PPU intentionally disabled
separate capture, leaving stale layer data. That run is **invalid for this
question** and was retained; the corrected capture above used extension 22.
Both protected ROM and snapshot hashes were unchanged. The local unit
`target/u31-action-reveal-zNIA7z/` retains the probe, metadata, selected layer
rasters, contact sheet and input guard. No native dispatch, full GUI, mouse,
Android, USA or native Windows result is claimed. Before proposing a code
change, compare a composed window and its actual hit coordinates through the
same transition and establish a stable owner independently of the fallback.

## Follow-up: software composition and coordinate round trip

On 2026-11-02 a new isolated Linux interpreter probe replayed the same 72-frame
EUR transition to obtain additional composition and coordinate evidence. It
used the fork's current `Layout`, `Frame::sample`, pixel-axis and
`menu_panel_native` source in a small standalone harness, together with the
real captured world and packed UI layers. The diagnostic image uses a neutral
RGB lookup, a fixed 852×480 window, a 284×160 view and no temporal response or
GPU. It is a software reconstruction, not an application screenshot. Selected
composite frames at lengths 100% and 150% were inspected; they are retained
with the harness and trace in `target/u31-action-composite-tUxpcq/`.

For each frame and length, the probe asked the current `MenuScene::read` for
the first row, chose a point inside its recognized hit region, transformed
that point to the reconstructed screen and back through `Layout::source` and
`menu_panel_native`, and checked `MenuScene::hit`. At each length, all 33
visible samples with a recognized menu returned to row 0. Those samples were
frame 0 and frames 40–71: they do **not** cover the partially revealed frames.
The point was derived from the menu's own hit geometry and tested against a
visible panel pixel; this is a coordinate round trip, not an independent
measurement of a text glyph, a real mouse click or proof for all rows.

In the remaining 39 frames, including the partial closing and opening samples,
`MenuScene::read` returned no menu, so the probe could not verify the menu hit
there. Source inspection shows `game_mouse_aim` may then follow its world
coordinate path, but no mouse event was injected and no unintended game action
is claimed. The first chosen sample point was transparent and produced zero
visible tests; that setup is retained as invalid. Moving the point inside the
visible backing produced the 33 observations above. The six protected ROM,
snapshot and source files retained their hashes; the isolated example build
emitted three unused-code warnings.

This narrows the remaining acceptance work: use the actual application window
and mouse events to verify ownership, visible target alignment and absence of
underlying actions during the partial frames. The reconstruction does not prove
a GPU, native Windows, Android or physical-input result, nor does it by itself
establish that the upstream geometry change fixes a reproduced defect.

## Follow-up: modeled click during partial reveal

On 2026-11-02 an isolated Linux interpreter comparison loaded the same EUR
snapshot into two machines and ran the same 72-frame B/release/A/release
sequence. At opening frame 37 the machines had identical serialized states;
the action panel was partially visible, `MenuScene::read` returned none and
`shop_layout` was false. A software-composed visible backing pixel at screen
(708, 310), with the same 852×480 window, 284×160 view and 150% panel length,
was mapped to world coordinate (214, 103). The comparison modeled the current
application's fallback branch and key pipeline for this one point: the
language scene was absent, battle mouse `aim` returned false, and the
unhandled-click fallback accepted an A press. The other machine received no
click.

On the next four frames the modeled-click machine received A (`0x3fe`), while
the control received neutral (`0x3ff`). At frame 40, and in the sampled
frames through 71, the control had a recognized battle menu and the clicked
machine did not; their serialized states differed. This establishes a
reproducible side effect in the modeled path for this sample, not what action
the user intended or a complete account of the resulting game scene. The
unit, source and trace are retained locally at
`target/u31-action-click-npL7Fp/`; ROM and snapshot hashes were unchanged.
Two initial compile attempts failed on a harness-only digest formatting error;
the corrected run passed with three unused-code warnings.

No actual window event, physical mouse, GPU or platform-specific application
was exercised. The frame-step debug interface does not deliver game clicks
while stepping. A real application-window test remains necessary before
claiming a GUI reproduction, selecting a fix or establishing behavior for
other pixels, timings, scales and platforms. The result does not authorize an
upstream merge or runtime implementation.

The managed cleanup helper rejected this U31 directory because it only accepts
`target/rom-data-*` units; the local evidence was retained.

## Follow-up: recognized-menu miss and upstream background guard

The pinned upstream desktop event handler records `menu_context` before aiming:
it includes a recognized language or battle menu, selected service layouts,
roster and map-award contexts. It only falls back to A when the aim was not
handled, no menu context was detected and the pointer maps inside the game
view. The fork's inspected left-click handler instead queues A whenever
`game_mouse_aim` returns false, including a recognized menu whose UI layout
rejects the pointer. In the fork, `battle_menu_ui_layer` selects the separate
Actions panel; when its layout rejects a point the aim handler cancels the
pending battle mouse and returns false. This guard is a separate input decision
from the upstream panel-packing geometry.

A new isolated Linux interpreter comparison used the existing EUR snapshot and
B/release/A/release sequence. At frame 40 both machines had identical serialized
state and the fork recognized an Actions menu. With a 852×480 view and 150%
panel length, `Layout::source` rejected the in-window point (0, 0) outside the
panel; the harness modeled the fork's cancel/false and unhandled-click branch.
The fork core accepted the queued A. In the same modeled conditions
`handled=false`, `menu=true`, `in_game=true`, the pinned upstream
`allow_background_confirm` expression yields false. At frames 41–44 the
clicked machine received A and the control received neutral; at sampled frame
44 the menu was no longer recognized in the clicked machine but remained
recognized in the control. This is an additional recognized-menu miss, not a
repeat of the earlier partial-reveal sample. The local unit
`target/u31-menu-miss-KG2OUl/` retains source and output; ROM and snapshot
hashes were unchanged. Its isolated build passed with seven unused-code warnings.

This compares the upstream guard condition with a modeled fork event and real
fork core; it does not run the upstream application, deliver a window event or
prove the upstream recognizer would identify the same underlying scene. In
particular, `action_layout` alone is not among that handler's enumerated
`menu_context` predicates, so the earlier partial-reveal case with
`MenuScene::read=None` is not shown to be fixed by this guard. Other upstream
recognizers, aim handlers and coordinate transforms differ. A future bounded
proposal should preserve intentional background A in dialogue/world contexts,
consider pointer-outside-view and missing-position behavior, and test recognized
menus, partial transitions, right-click, keyboard/controller, recording/replay
and EUR/USA separately. A GUI comparison remains necessary; no integration is
approved by this evidence. The environment checked for this unit had no Xvfb
or xdotool; the available display is shared and was not used.

The managed cleanup helper rejected this U31 unit because its accepted scope is
`target/rom-data-*`; the evidence remains in place.

A separate [recognized-menu miss proposal](U31-03-recognized-menu-miss-proposal.md)
sets out the bounded candidate change and its GUI, replay and regression gates.
It remains unapproved and does not close the partial-reveal or geometry work.
