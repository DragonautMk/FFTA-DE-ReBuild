<a id="pendientes-de-mouse-para-la-proxima-sesion"></a>

# Mouse work pending for the next session

User report from 2026-10-17. Manual checks on the Windows preview.
Archived for continuation the following day; do not treat these features as finished.

<a id="medido-por-el-usuario"></a>

## MEASURED BY THE USER

- Latest check: Pub is still not fixed (small anchor shifts and direct jump).
  PENDING by user instruction; stop work on Pub. Earlier replay checks do not
  establish its result in the window. Next authorized work: mouse support for Shop.

<a id="primera-entrega-tienda-pub-sigue-pendiente"></a>

## First Shop delivery (Pub still pending)

<a id="segunda-entrega-categorias-paginas-y-seguimiento"></a>

### Second delivery: categories, pages and tracking

New project-owned adapter crates/gba-core/src/ffta_shop_mouse.rs. Exact ROM
profile and short signatures: selector USA0806EFA8/EUR08071490; original store +30;
category widget manager[0200F428]+4470. Original Shop owner
USA08069659/EUR0806BB0D; reject stale widgets from another scene.
Reads index, count and available category table, and geometry +78/+79.
MEASURED: buy ids[0,1,2,3,4,5], sell ids[1,2,3,5] in this game.
Hover changes category through the original store and redraw, without passing
through intermediate categories. A quick click on items waits for the requested
category to be published. A context change cancels the request.
The list reads page start list+1A and relative cursor: after scrolling, first1
and row3; the mouse jumps to row0 and row3 while keeping first1, tested in replay.
Categories, lists and quantity share a fixed UI canvas and its inverse transform;
previously lists could be drawn in UI while mouse hits were detected in the world.
Wheel: scrolls the list or adjusts quantity; clicking tens/units selects the digit.
Quantity: original Up changes01->02, measured; A/left confirms, B/right cancels.
Do not change quantity with the wheel at final confirmation. Gaps for categories
unavailable when selling and empty rows consume the click without buying an item.
New tests: both regions, availability, context change, deferred request, wrong
bytes, scrolled page and UI coordinates in multiple windows.
EUR replays:shop-mouse-buy and shop-mouse-sell cover all available categories;
private script target/shop_verify_replays.py. Final UI capture checks the right side.
Windows compiled. Extra=0:600/600 framebuffer/OAM hashes match the baseline.
NOT MEASURED: manual acceptance of this delivery, all shops/languages and physical
mouse checks in USA. Pub remains pending; installer unchanged.

MENU Buy/Sell/Leave: profile of3 rows, origin[8,92], game width by language.
It is extracted with the portrait/UI to a fixed right canvas and the mouse
inverts the same layout. This delivery does not change Pub profile or hook.
Shop-specific direct selection: original setter USA08017FB0/EUR080190F4;
its caller uses r4/r6 (not Pub's convention). Short setter and caller signatures
are checked before changing registers; writes are the original game's.
MEASURED replay shop-after:0->2 frame4902,2->0 frame5002, without intermediate rows.
Buy list: draw callback USA0806EB48/EUR08071030; signatures checked.
Reads count and RAM geometry;0->2 in5402, opens quantity and then purchase A/B.
Quantity/confirmation check: A at5800 opens That will be800gils, OK/CANCEL.
The runner's left/right click already sends A/B outside lists; do not confuse
that confirmation with a two-row selector. Quantity retains GBA controls.
EXPLICIT LIMIT: only lists without scrolling/paging (total<=visible rows,
start0). Mouse categories, paged lists and complete selling NOT MEASURED;
these are later work and this delivery is not presented as complete Shop support.
Manual test of the native window pending; private captures/logs in target/.

- The map still does not work with the mouse. The runner coordinate check does
  not prove it works in the real game.
- The map stretches too far to the left.
- Cyril's map menu responds to the mouse. Other towns NOT MEASURED.
  However, it moves through options one by one until reaching the target.
- Shop does not work with the mouse. Buy / Sell / Leave also fails and the
  blue part of the menu is displaced.
- In Pub, the entire menu appears too far left. It must be placed on the right.
- Pub submenus respond to the mouse, but the mission-acceptance confirmation
  (Accept / Yes / No) does not.
- New menus generally pass through intermediate rows instead of jumping
  directly to the option under the mouse.

<a id="comportamiento-requerido"></a>

## Required behavior

If selected option is 1 and the mouse moves directly to 3, select 3 immediately,
without selecting or passing through 2. The adapter must use the original
logical state and its complete update; do not simulate a sequence of keys.
This also applies to Cyril's menu, not only Shop and Pub.

Review panel positions and the mouse-coordinate transform together. Do not
assume a native-coordinate hit or an interpreter capture proves behavior
in the native window.

<a id="estado-al-parar"></a>

## State at stopping

- The preview command was delivered with target/release/recomp.exe, EUR ROM,
  FFTA_WIDESCREEN=32:9 and FFTA_MOUSE=1.
- The delivered preview does not necessarily contain all latest source changes:
  the final arrow-anchor adjustment was not confirmed as recompiled. Check
  source, artifact and logs when resuming.
- Pub replay still showed selected=2 when requesting target=3. Do not mark
  direct jumping or universal list support resolved.
- Previous pending items: character facing at end of turn, repeated effects
  such as thunder in 32:9 and a later Android port, disabled by default.
- No commits. Do not touch installer or original ROM/save. Build Windows after
  every delivered fix. Agent tests use save copies.

Also consult docs/ffta_mouse_world.md and docs/ffta_estado.md, treating this
manual report as the latest reference for what actually works.

<a id="primera-correccion-para-revision-del-usuario"></a>

## First correction for user review

Battle menu: mouse routing prioritized action_layout (main HUD, layer0) even
when the action list was in extra/Actions (layer1). The active list now has
priority. Do it/Cancel retains the main route when no extra action list is active.
The change is in crates/recomp/src/play.rs and serves both ROM profiles.
The test reproduces a simultaneous HUD and list option after pack_panel, with
different windows, aspects, Size and Length. Real-game acceptance remains open.
This correction is delivered separately; the other problems remain pending.
MEASURED:17 runner tests and14 battle-input tests pass. Framebuffer and OAM
identical in600/600 frames with extra=0 and mouse disabled against the previous
executable. Windows release compiled; logs target/battle-mouse-routing-build.log,
target/battle-mouse-routing-tests.log and target/battle-mouse-zero.log.
NOT MEASURED: manual acceptance of this first correction. Stop for that check.

<a id="reporte-adicional-del-usuario-anterior-a-esta-correccion"></a>

## Additional user report, before this correction

<a id="correccion-pub-2026-10-17"></a>

## Pub correction 2026-10-17

<a id="salto-directo-del-menu-y-estados-transitorios-del-cursor"></a>

### Direct MENU jump and transient cursor states

MEASURED replay pub-jump-ready.input, from0, hover to option2 at frame4900:
previously wrote0->1 at4902 and1->2 at4910 (02025F19, PC080190F8).
Now writes0->2 at4902, without intermediate index. The original setter stores
the index and row offset; its caller publishes the same index from the high
byte of r6. The hook changes r1/r6 and leaves writes to the game.
Short signatures checked: setter USA08017FB0/EUR080190F4;
caller return USA0805FA32/EUR08061C96, signatures checked in both ROMs.
Only root MENU and active mouse requests; physical input is unchanged.
Root anchoring ignores cursor status bit08: states07/0F retain the same layer,
with width varying by language. LIST retains its previous path.
Tests cover both profiles, widths96..224, states07/0F and changed signature.
Replay after:161 frames4800..4960 with right anchoring and no layer changes.
NOT MEASURED: exact reproduction of the user's reported microflicker;
feedback on the new executable and manual checks of all languages remain missing.

<a id="revision-final-raiz-izquierda-submenus-restaurados"></a>

### Final review: left root, submenus restored

Later update by user instruction: root MENU must be on the right. Only the fixed
canvas side changes (PPU and layout/mouse); the root profile and restored
submenu handling stay the same.
Language correction: actual capture pub-current-desktop.png shows MENU in
Spanish still on the left. The fixed English width96 is removed from the profile;
validated width is read from the original list. Tests cover widths96..224
as well as96, without including submenu style03. Manual validation of the
updated executable in that Spanish scene remains missing.

The user requests reverting LIST/Rumors/Missions displacement and anchoring
the complete root MENU on the left. The visual profile accepts only the
measured MENU: style0F, four rows, origin[8,92], right bound96;
LIST style03 keeps its previous visual path.
MENU, portrait and remaining root UI share a fixed native canvas of240x160,
without independent clipping or anchors calculated from animated pixels.
Mouse uses that same canvas. Sprite engine and installer unchanged.
MEASURED: replay from frame0, input pub-left-hold.input (A at5000 omitted
to avoid entering Rumors). Frames4800..5400:601/601 main=true, side=false,
extra=None, without layer/side changes. Test with extra_x306 and extra_y40.
Logs: target/pub-left-stable.log. Tests143 core+18 runner passed,4 ignored.
NOT MEASURED: manual acceptance of all Pub options and transitions.

Later correction: the user rejected the previous delivery; the LIST snapshot
did not cover MENU with Rumors/Missions/Quit Mission/Leave. Actual capture reviewed:
target/pub-live-desktop.png. Corresponding snapshot mouse-pub-simple-4800:
list of 4 options, origin [8,92], right bound96, cursor flags0F, not03.
The profile admits both measured styles. MENU is extracted to its own right layer;
Greetings/portrait/remaining UI retain an independent right layer, so dialogue
width does not leave the menu separated from its edge. Mouse inversion uses the
menu layer. The test now includes the 4-option MENU.
Final measured snapshot: town=true right=true, main UI [3,1,240,155],
independent MENU [0,78,88,157]. Complete tests:143 core+18 runner passed,
4 ignored. Extra=0:600 framebuffer/OAM pairs without differences. The previous
window still uses its renamed executable; open the new one to test.

MEASURED in EUR snapshot mouse-pub-direct-5400: BG0/1 4C03/4E02,
BG2/3 0B08/0A08; original 8-row list at 02025F08, flags 03.
The panel was mistaken for dialogue and anchored left; its OBJ arrow did not
enter the separate layer. The active-list profile now checks original callback
and geometry; panel and arrow go together to the right.
The mouse inverts the same layout and retains selection even when the arrow
blinks. Existing confirmations retain priority over the list.
Detection uses exact USA/EUR profiles without option names.
MEASURED: separate snapshot town=true right=true bounds=[40,6,192,149].
Row checks during blinking and right-panel coordinates pass.
NOT MEASURED: manual acceptance of the whole Pub and all submenus; Pub USA.
Installer and original save unchanged. Stop for user testing.

- MEASURED BY THE USER: Do it/Cancel already works. Preserve it.
- Battle menu stopped responding to the mouse.
- Notice and other warnings appear completely wrong.
- The upper part of long ears on Montblanc, Chiko and similar characters is
  displaced in battle. Do not yet assume it is a particular layer.
- With Lester and other characters, part of the action menu appears on the
  right, separated from the rest of the panel.
- Spell names extend across the screen; they should stay anchored to their
  corresponding corner.
- Mouse and Keyboard needs a switch exclusive with Keyboard and adjacent
  explanation including (Experimental).
- Prison: blue menu background displaced and mouse not working.
- Fast-forward with Normal/x1 audio: the user wants background music only,
  without effects or other sounds. Separating music and effects remains
  pending; do not confuse this with current behavior, which may still play both.
- Find shared components so a Shop, menu or ear-piece fix benefits all users,
  without name-specific shop/character solutions. Verify both USA/EUR profiles;
  do not treat one region as tested solely from the other.
