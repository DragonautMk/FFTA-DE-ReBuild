# Shop and Monster Bank mouse follow-up, 2026-10-17

MEASURED: the private EUR Sell snapshots have category IDs [1,2,3,5],
drawn contiguously at native X=16,32,48,64 on BG3. Hit testing must use
the visible ordinal, not the item-category ID. Buy has all six slots.
The inventory callbacks differ: Buy USA08069659/EUR0806BB0D,
Sell USA0806B6D1/EUR0806DB7D. Both are accepted by the context gate.

MEASURED: a complete replay from frame 0 with mouse coordinates, using only
target/mouse-world-private.sav, selected Sell slots 3,2,1,0 at frames
5150,5350,5550,5750. These refer to IDs 5,3,2,1 respectively.
The replay and assertion script are private diagnostic files in target/.

MEASURED: Monster Bank is a parent-window component, not the Pub text-list.
Its root handlers are USA08058D40/EUR0805AEE4, with updaters
USA0805C594/EUR0805E7B4. Short ROM signatures, live callback pointers,
cursor ownership, count, geometry and index are checked before activation.
The measured EUR root has two rows, Y=92, window X=0..48; the cursor
anchor X=24 is not the start of the text. Cursor blinking does not end
ownership. Its original input updates the selection; the host releases
the direction once the requested row is observed.

MEASURED: pointer (12,116) selected Back at frame 5500; (12,98) selected
List at 5700; clicking List entered its original handler at 6100.
The EUR empty-list message uses handler 0805B624 and retains the centered
UI canvas. Detection uses state/geometry, never translated labels.

Buy/Sell and the bank use a centered, fixed native UI canvas. Their frames,
text and cursor share the same physical-to-native mouse transform.
Complete town/shop panels bypass the dialogue-only flat-edge clamp.
No installer, original save, ROM content or sprite behavior is changed.

MEASURED: native UI reconstruction is exact (changed_pixels=0) for the bank,
Buy and Sell snapshots. Captures: target/bank-menu-after.png,
target/shop-buy-centered-after.png, target/shop-sell-centered-after.png.
Core suite: 151 passed, 4 ignored (serial). After the final backing fix,
all 13 dialogue-layout tests passed. Centered-canvas screen test passed.
Windows executable: target/release-test-menu/release/recomp.exe; the normal
target/release/recomp.exe was left running and was not replaced.

NOT MEASURED: a populated monster list, every bank submenu, and manual Spanish
interaction. USA root signatures and synthetic geometry are tested; the
full bank pointer replay was EUR. The empty-list continuation above is
enabled only for the measured EUR handler.
