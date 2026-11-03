# Experimental battle mouse: logical cells and hover

Enable FFTA_MOUSE=1. The field cursor and live vertical battle lists have mouse
adapters. Move the mouse to aim a logical cell, left click confirms with original
A, right click sends original B. Physical buttons, focus loss, pause and leaving
the map cancel pending requests. Hover never confirms a move.

2026-10-17 follow-up: unsupported click contexts now send one original A/B pulse,
so dialogue and attack forecasts can be advanced without a recognized hit target.
Plain pulses survive scene transitions; physical controls cancel them. Field hit
testing requires only its active terrain pair, not all four BG layers. Enabled
story text planes retain A/B ownership instead of moving a suspended map cursor.
The measured Wait prompt's four OBJ pieces (tiles135/139, mirrored pairs) identify
facing selection. Hover sends the corresponding original direction; click queues
direction then A, without changing unit memory. No facing commit is injected.
MEASURED: pulse sequencing, free-field BG masks, direction-before-confirm, no hover
repeat and physical cancellation pass unit tests. NOT MEASURED: manual acceptance of
this follow-up in USA/EUR; the user will test it. No installer or save changes.

## Measured original structures

Both supported ROMs use these RAM structures in the tutorial:

- 02007F10 map descriptor. +4 points to two-byte cell records; +8/+9 are
  storage width/height. +0A/+0C are playable origin X/Y; +0D/+0F playable
  width/height. The observed storage is16x16; playable extent14x14.
- Each two-byte cell stores elevation followed by property flags. The original
  cursor rejects absent cells and property bit08. Bounds and contents are read
  from RAM for the current stage, never synthesized from a fixed screen grid.
- 02008168 range renderer descriptor. Active flag+0, mode+1 must equal3.
  Packed quarter styles are at02008190 and02008290, indexed y*16+x.
  The original query returns the first nonzero2-bit quarter of the first
  nonempty plane, high bits first. These are style values0..3, not a universal
  movement boolean. The host displays/queries them; original A retains final
  legality, pathfinding and movement costs.
- 0200F3AC cursor descriptor in both regions. +0 points to its OBJ instance,
  +4 is cursor mode, +0C/+10 are current X/Y, +14/+18 previous coordinates,
  +0A marks a cursor movement. Modes1/2 are adapted. Scripted camera mode4,
  disabled modes, action panels and missing cursor instances are excluded.

Original Thumb routines, measured from the user ROM:

| Function | USA | EUR |
|---|---|---|
| Existence/bounds query | 0801CC7C | 0801DDEC |
| Packed range query | 0801D58C | 0801E6FC |
| Isometric projection | 0801C918 | 0801DA88 |
| Cursor key handling | 08024AD8 | 08025CEC |
| Commit comparison hook | 08025132 | 08026346 |

The projection uses the descriptor02007F40. In native game pixels:

    screen_x = 16*(x-y) + i16[02007F60] - i16[02007F64]
    screen_y = 8*(x+y+1-elevation) + i16[02007F62] - i16[02007F66]

The mouse first uses the existing window/view/letterbox inverse transform, then
this logical-cell hit test. Cell surfaces are32x16 diamonds. Zoom changes the
window transform, never logical X/Y. When projected surfaces overlap, the
frontmost X+Y order wins, then height. Detailed sloped edges and every map have
not been exhaustively validated. A stationary mouse does not chase camera
motion; only mouse movement retargets, avoiding feedback from camera following.

## Commit and safety

State initialization requires the exact ROM SHA256 profile and short original
entry/commit checks. The host queues X/Y, rather than writing cursor or unit
coordinates at arbitrary times. One emulated frame of virtual Right wakes the
original cursor handler. At its verified comparison point the host substitutes
only Thumb registers r10/r9; original instructions compare positions, mark the
movement, save previous coordinates, commit X/Y and refresh the cursor normally.
The original routine runs interpreted only while a request is pending; native
execution resumes afterwards. No TinyCC cache or generated ROM payload change
is required. This hook works in Machine::step and native desktop dispatch.

An initial held wake-up key caused an extra step. It was rejected by the real
replay test and replaced with a single-frame pulse. A new hover cancels any
previous confirmation pulse. A click waits until commit and two frame boundaries
before original A. Invalid cells do not generate A.

Recording remains gba-input v1: mouse:256..511 encodes256+y*16+x. Old mouse
IDs and keyboard recordings remain compatible. Replay queues the same cursor
hook; recorded wake-up keys are not interpreted as manual cancellation.

## Evidence

MEASURED: isolated calls to original USA queries over256 storage cells find196
existing cells and20 range-style1 cells. Host range query matches all256 values;
host projection matches the original projection for all196 selectable cells.
Private evidence target/mouse-battle-grid-usa.log.

MEASURED: complete USA replay from frame0 using the private save copy, requests at
12560,12600,12640 select(6,1),(9,3),(1,12). Interpreter commits on12561,12601,
12641, and snapshots12580,12620,12680 retain those exact targets. Native32:9
zoom6 also reaches and retains all three destinations, including elevated(9,3).
Evidence target/mouse-battle-hover-one-frame.* and mouse-battle-hover-final.log.

MEASURED:221 tests pass,5 ignored. USA/EUR extra=0 framebuffer AND OAM identical
for600 frames against previous reference. USA1201-frame tutorial metric retains
exactly8 body IDs, valid roster, no duplicate groups. Evidence
 target/mouse-battle-characters.csv and mouse-battle-tests.log.

MEASURED: EUR original routines recognize196 cells in a real saved RAM snapshot.
An isolated EUR cursor-function test on copied RAM reaches(9,3) exactly. Its
original mode4 was explicitly changed to1 only inside the diagnostic copy;
this is NOT proof of interactive gameplay. Evidence
 target/mouse-battle-eur-controlled.log.

NOT MEASURED: full EUR gameplay hover acceptance, every stage/slope/occlusion,
physical mouse latency and real movement-confirmation UI in all battle modes.
Battle action menus, inventory, scrolling the camera with mouse edges and other
submenus remain separate work. No claim of universal battle-menu support.

Windows: target/release/recomp-mouse-battle.exe. No installer source, original
save, ROM, sprite producer or Android changes; no commits.
