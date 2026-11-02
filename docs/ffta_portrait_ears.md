# Action portrait ear pieces, 2026-10-17

MEASURED in the user's EUR Spanish Giza battle, Clara's portrait:
OAM slot 5: attrs 6038/9000/06AA, X=0 Y=56, 32x16, priority 1, 8-bpp.
OAM slot 6: attrs 2030/1010/06BA, X=16 Y=48, 8x8, priority 1, 8-bpp.
Other portrait pieces begin at Y=72,88,96,128,136.

The action-portrait filter required Y>=64. The two upper pieces consequently
remained in the world pass while the other pieces entered the independent UI.
The filter now accepts Y=-64..159, retaining the existing scene, X, 8-bpp,
non-affine, display-mode, priority and portrait-atlas checks. No names or
character IDs are used, and 4-bpp battle units remain outside this filter.

MEASURED: regression test uses the two measured OAM records with synthetic
own-color tile data. Both pieces enter the action UI and leave the world;
hardware framebuffer and OAM remain identical. All 14 dialogue-layout tests pass.
Private evidence: target/giza-ear-live-*.bin and the after captures below.
MEASURED: offline captures giza-ear-after-16x9.png and giza-ear-after-zoom2.png
show the formerly detached pieces connected to Clara's portrait at both zooms.
The full native-UI reconstruction diagnostic reports 61 differing pixels on
this live snapshot in BOTH the previous executable and the patched one.
That pre-existing reconstruction discrepancy is not resolved by this patch;
do not report this particular snapshot as an exact reconstruction.
NOT MEASURED: manual confirmation after relaunch; other long-eared portraits.
No installer changes, no ROM patch or bundled game pixels, no commits.
