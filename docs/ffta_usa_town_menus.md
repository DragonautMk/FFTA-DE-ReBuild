# USA town menus, 2026-10-18

MEASURED: private replays from frame zero, USA exact hash, only copied saves.
Root menu at frame 3300: Pub 4 rows, X=8/Y=92/right=104; Shop 3 rows at
the same origin; Monster Bank 2 rows, X=0/Y=92/right=96. Pub Rumors has
8 rows at X=56/Y=20. Buy inventory and category handlers match the existing
verified USA profile (08069659 and 0806B6D1). All root mouse selectors were
accepted and their selected indices changed in replays.

Changes:
- USA Pub root gets a separate right/bottom UI panel. The greeting/portrait
  keeps its own canvas; Rumors/Missions list subwindows are not detached.
- USA Shop Buy/Sell/Leave uses the same centered canvas as its inventory.
- USA bank empty-list continuation is callback 08059455 with its own measured
  12-byte prefix, distinct from EUR 0805B625. The same canvas now survives
  entry into that message.
- USA active parent-window service lists are recognized from the verified
  region cursor callback, owner/back-reference, original Thumb handler/updater
  pointers, window geometry, row count/index and style. Read count/coordinates
  from the owner; do not identify options by translated names. Invalid or
  ambiguous ownership falls back to the existing path. Existing verified
  cursor hooks/input own selection and confirmation; no new game-code hook.
- Mouse inverse mapping of town panels uses raw native coordinates, matching
  their unextended canvas even when Text Bubbles Length is not 100 percent.

Evidence: target/usa-root-{pub,shop,bank}-after.png,
target/usa-bank-list-after.png. Each measured snapshot reconstructs the native
frame with changed_pixels=0. Synthetic regressions cover detached root/cursor,
unchanged framebuffer/OAM/RAM, parent owner validation, and both bank regions.

NOT MEASURED: actual Prison entry/selection. The world-map route with private
saves did not reach it, including a newer private save copy. The shared USA
service detector is implemented, but no claim that the real Prison window
matches it. A populated Monster Bank list and all mission/shop dialogs also
remain outside the measured paths. No original save, installer or ROM edited.

Validation: full core suite 159 passed, 4 ignored. Targeted service-owner test
rerun after restricting the new fallback to USA. Final Windows release build
succeeded. The 600-frame interpreter boot regression at extra=0 compares every
framebuffer/OAM byte with separated UI off/on, completing all 600 frames:
USA framebuffer c31042464ec20009, OAM 15ebcf419e7d9576;
EUR framebuffer a04a7922125b2159, OAM 971d444336ad5341 (rolling FNV-1a hashes).
These are boot checks, not 600 frames of every town-menu interaction.
USA Sell replay also verified compact category IDs [1,2,3,5] and actual
selected slots 0,3,2,0 at frames 4200,4500,4700,5000 after mouse category
clicks. Original item-list counts updated with each category. Assertions:
target/check_usa_sell.py; private input target/usa-shop-sell-verify.input.
