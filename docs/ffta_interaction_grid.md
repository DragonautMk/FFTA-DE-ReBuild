# Invisible interaction grid

Implemented in crates/gba-core/src/interaction_grid.rs. No drawing, ROM data,
menu-specific RAM addresses or GBA input logic belongs to this layer.

-8x8 logical-pixel cells cover the published viewport, including zoom margins.
-Each region registers an EventId(context,option), exact rectangle and layer.
-Cells index candidate regions; exact half-open rectangle checks preserve gaps
 and separate options even when both occupy the same cell.
-Highest layer wins; a region with no event is a modal blocker.
-PointerUpdate reports current cell, hovered event, entry, exit and click.
 Crossing cells within one event does not repeat entry. Multiple regions may
 share one event ID. Click remains separate from selection.
-Target/view changes rebuild the index; unchanged publications reuse it.
 Scene loss clears targets and pointer focus. Regions and pointer positions must
 use the same coordinate space; the menu adapter owns their transforms.

## First integration

LanguageMouse publishes the five verified EUR language targets with context1.
The existing inverse window/world transform supplies pointer coordinates, so
window size, aspect and zoom keep their existing behavior. The shared grid
resolves the target; the existing guarded direct selector commit and A pulse
perform selection and confirmation. FFTA_MOUSE=1 remains required.

For each next menu, identify its active state and selection/confirmation events,
register the current regions, and route their EventIds to those validated events.
Reuse the grid, pointer transitions, priority/blocking rules and recording path.
This does not discover arbitrary game events automatically. The EUR front-end adapter now includes the main menu and its measured submenus.
Battle adapters are still pending.

## Measured

214 suite tests pass,5 ignored. Tests compare against an exact layered reference
at153600 positions, handle shared cells/gaps, hover deduplication, modal blocking,
negative zoom margins, viewport changes and scene clearing. All38400 native
selector pixel centers resolve to the original five target rectangles.

Actual EUR ROM: five direct language changes and one confirmation succeed;
600 cells, exactly one index build throughout the selector test. All five images
and recorded events match the preceding direct-selector prototype byte for byte.
Native replay equals interpreter; final370-frame hash d17fc597c1fb9325.
USA/EUR extra=0 framebuffer and OAM identical for600 frames each.

NOT MEASURED: physical mouse with the grid build, next game menus and performance
benefit over five rectangles. This change supplies reusable routing, not a claim
of faster rendering or automatic event discovery.

Windows: target/release/recomp-mouse-grid.exe.
