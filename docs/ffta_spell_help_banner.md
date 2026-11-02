# Spell description strip

2026-10-17. MEASURED: live EUR Lightning damage prediction saved as
target/lightning-before.png and private target/lightning-live-*.bin.
BG2=5C08, BG3=4A09. WIN0H=10E0, WIN0V=0818, WININ=001F.
The native help strip occupies X=16..223,Y=8..23. Its complete text exists
in BG2; the fixed X=160 action split discarded its right part while an OBJ
target-stats panel occupied the second UI layer.

Inside the already verified battle action scene, the short WIN0 rectangle
and its BG3 border (0032/0033 in bank B, flipped at right/bottom) identify
the strip. All of its BG2/BG3 pixels stay with the left UI layer. The target
stats panel, action list and underlying guest framebuffer/OAM are unchanged.
No text labels or translated strings are used; no new game logic is patched.

Synthetic regression checks text at X=208 survives the split, stays out of the
right layer, and hardware framebuffer/OAM remain identical. Wrong-height
windows fail the strip classifier.

Spell-effect repetition: user reports it already resolved by an earlier fix.
No effect-rendering or character-sprite changes made in this task.

MEASURED: full gba-core suite 156 passed, 4 ignored. Windows release compiled.
Snapshots target/lightning-after.png and target/lightning-after-zoom2.png show
the complete Lightning damage! text and rounded right cap. Native composition
diagnostic: old binary 1316 differing pixels, patched binary 326, at zoom 1/2.
This is not an exact-reconstruction pass; residual forecast/UI differences
remain unclassified. The synthetic hardware framebuffer/OAM identity check
passes. NOT MEASURED: user confirmation of the running patched binary.
