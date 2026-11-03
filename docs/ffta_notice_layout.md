# Notice modal, live Giza capture, 2026-10-17

MEASURED: user's EUR runtime, DISPCNT=3F40, BG2=5C08, BG3=4A09,
WIN0H=10E0, WIN0V=1040, WININ=001F, WINOUT=003F.
The native Notice occupies [16,16,224,64]; BG2 contains the message,
BG3 contains its complete background, title and frame. The action layout
split at X=160 placed the left portion on the action HUD and the right
portion on its separate right-hand panel. The flat-edge dialogue clamp
also copied parts of this rectangle outside its original bounds.

The existing exact-ROM/scene-gated action path now identifies wide opaque
WIN0 rectangles with four matching BG3 frame corners B00D/B00E/B00F/B010.
Text and language are not inspected. Narrow Yes/No/action confirmations do
not qualify. Notice is captured as one complete panel; its rectangle is
excluded from the underlying HUD and the flat-edge clamp. The desktop
places it horizontally centered, preserves its top margin when Size changes,
and never applies the bubble Length extension. World zoom does not scale it.
While this modal is present, mouse aiming cannot select the underlying menu;
the existing global A/B click path remains responsible for dismissing it.

Private evidence: target/notice-before.png and target/notice-live-*.bin.
Regression uses synthetic own-color tiles and verifies complete separation,
native framebuffer/OAM identity and exact recombination. The previous Clara
ear regression remains in the suite. No installer or original save is changed.

NOT MEASURED: manual confirmation after relaunch and all other warning varieties.
