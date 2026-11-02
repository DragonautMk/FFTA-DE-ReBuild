# Variable action menu width

2026-10-17. MEASURED: EUR Lester, Giza, live snapshot
target/action-long-name-live-*.bin. BG2=5C08/BG3=4A09. The menu containing
Fight, White Magic and Item has its BG2 header C005/C006 at X=152,Y=80,
footer C009/C00A at Y=144. Cursor OAM slot 0 is 0075/0097/0125,
X=151,Y=117. The previous X=160 split incorrectly assigned eight pixels
of the menu and its cursor to the left portrait HUD/world.

The verified action scene now obtains its rectangle from the existing native
BG2 header and matching footer. The rectangle is routed entirely to the right
UI layer; the rest stays on the left layer. The cursor accepts the game's
one-pixel left overhang. No names, translated option strings, guest memory,
ROM data or UI text are changed. Ambiguous/unrecognized headers retain the
previous split rather than guessing a rectangle.

Synthetic regression checks four native menu widths, header and cursor
ownership, and unchanged native framebuffer/OAM. No installer/save changes.
MEASURED: full gba-core suite 155 passed, 4 ignored. Windows release build
succeeded. Real Lester snapshot at zoom 1 and 2: native reconstruction has
0 changed pixels. Captures target/action-long-name-after.png and
target/action-long-name-after-zoom2.png show no displaced left frame fragment.

NOT MEASURED: user confirmation after relaunch. Repeated spell effects remain
pending as the next separate task; this patch does not address them.
