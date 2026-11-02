# Notice battle confirmation

2026-10-17. MEASURED: live EUR Begin battle confirmation, private snapshot
target/battle-start-live-*.bin and screenshot target/battle-start-before.png.
BG2/BG3 are 5C08/4A09. Choice cursor OAM slot 0: 002C/001B/0125,
native X=27,Y=44. The two identical BG2 bullet images start at X=27 and 65.
The prior Notice fix moved the backgrounds but omitted this OBJ cursor and
deliberately suppressed all menu mouse targeting while Notice was present.

The bounded 0124..0127 cursor now joins the complete Notice panel. Mouse
coordinates use that panel's inverse transform, independent of world zoom,
Size and Length. The underlying action menu remains blocked.

Choice detection samples the 8x8 BG2 image under the original cursor and
requires exactly two identical nonblank images in the measured Notice window.
This obtains live option positions without translated labels or fixed text
widths. Original left/right input changes selection, observed OAM confirms it,
and A is deferred until selection matches. No selection RAM or ROM is patched.
An informational Notice without this cursor has no choice hit targets.

Synthetic tests vary choice spacing and verify one confirm pulse after the
original selection changes. The rendering regression verifies the cursor joins
the modal, exact native reconstruction, and unchanged hardware framebuffer/OAM.
When a portrait overlaps the old native rectangle, the independently relocated
modal deliberately retains the background behind it rather than baking that
overlap into a permanent hole. The real snapshot has 73 such pixels; the native
reconstruction diagnostic therefore reports 73 differences for this intended
host composition change. Hardware framebuffer and OAM remain unchanged.

MEASURED: Windows release build succeeded. Full gba-core suite: 154 passed,
4 ignored before the final portrait-occlusion adjustment; targeted Notice tests
were rerun after it. Captures: target/battle-start-after.png and
target/battle-start-after-zoom2.png.

NOT MEASURED: manual mouse interaction after relaunch, other language variants,
and warnings whose options use a different glyph layout. Those fail closed.
