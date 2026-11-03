# USA title and main-menu mouse

2026-10-17. MEASURED: the USA LanguageMouse path previously allowed only name
keyboard input; the title/main-menu classifier contained EUR-only offsets.

Private USA replay from frame zero, without original saves:
title task at EWRAM 0200F3BC (5=Press Start, 7=menu), callback 08136085,
manager pointer at 0200F3B0 points to 0200F3DC. Selector offsets from manager
are 8962 (main), 8963 (Color Mode), 8993 (submenu mode). Main has three
entries, whose native text origins are Y=72,88,112; USA has no Language entry.
Press Start glyphs start at X=65,Y=105. Color options use the existing native
submenu positions. Saved Game uses manager+2158+105, same layout as EUR.

ROM checks: USA title prefix at 08136084 (12 bytes) and selection store at
08136A78 (4 bytes). Load selector store at 08089072. Exact region hash remains
required by LanguageMouse::new. No game code is copied or patched.

USA now routes to its own title/main profile and retains its existing name
keyboard support. EUR profile and five-language startup stay intact. Hover
updates only the verified selection byte; clicks use original Start/A/B input.

MEASURED: replays accepted Press Start at frame 900, main selection 0->2 at 1050,
Color Mode click at 1100, New Game click at 1100 in a separate replay, and
Saved Game click at 1100 using only E:/AI/ARC/ffta-respaldo.sav as read-only input.
Private artifacts target/usa-title-*.input, target/usa-mouse-{color,new,load}-*.
Color Mode option TV click at 1250 returned to the root menu at 1400.
Load and Resume Battle clicks were accepted and changed the selected entry,
but neither left that screen with this backup. Successful save loading is
NOT validated by these replays; do not infer it from accepted mouse events.
MEASURED: 10 mouse module tests passed, including all USA root/color selection
pairs, Press Start and EUR regressions. Windows release build succeeded.
NOT MEASURED: manual confirmation of the rebuilt windowed executable.
