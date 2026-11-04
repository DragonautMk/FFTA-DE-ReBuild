<a id="ispettore-grafico--dbg-12"></a>

# Graphics inspector — DBG-12

In a [paused isolated copy](debug_control.md), **Debug > Capture graphics report**
produces a local HTML report. From the CLI:

```powershell
.\recomp.exe debug-control graphics 'target\case'
.\recomp.exe debug-control graphics 'target\case' --compare-last
```

The response identifies `status.graphics_report`, relative to the copy. Open
`Data/config/graphics/<run>/index.html`. The English report works offline and
retains JSON, PNG and the same raw buffers as previous PPU dumps: VRAM, OAM,
palette and I/O backing, plus central/wide RGB555 framebuffers. No additional
snapshot, guest patch or register change. Complete state is checked before and
after; the copy must be stopped without pending steps, search or replay. The audit
feature is unnecessary. The normal runtime does not expose this channel.

In an isolated EUR Wine/Xvfb session, a baseline capture followed by
`--compare-last` through the local-control CLI, parameter form and command
console retained identical guest state and raw-buffer hashes. Each comparison
identified the immediately preceding report and found no raw-byte differences.
The frontend composition can differ with the visible interface.

<a id="output-salvati-e-proiezioni"></a>

## Saved output and projections

The first view compares central framebuffer, active wide framebuffer and frontend
composition. The CPU sampler reuses the snapshot-preview path: view/crop, LUT,
temporal response and native UI at window dimensions. The report includes the same
view without UI, changed-pixel count/rectangle and UI/clip rectangles. Pause menu,
FPS and final display GPU effects (grid/filter/scaling) are excluded. This is an
explicitly declared diagnostic composition, not a monitor capture or presentation
latency proof.

Other views derive from the **current frozen registers and buffers**:

- BG text: native 240×160 projection with the renderer's existing readonly helpers;
- BG affine: raw tile map without inventing earlier scanline references;
- bitmap: current page in modes 3–5;
- OBJ: readonly renderer projection including current host objects;
- windows: BG0–3/OBJ/effects masks from current registers and OBJ-window projection;
- palette, BG/OBJ 4/8bpp tiles and OAM textures without transformations.

A saved framebuffer can include a partial frame or palettes changed between
scanlines. A frozen projection does not reconstruct that history. The report keeps
this distinction even when images appear equal.

<a id="selezionare-un-pixel"></a>

## Select a pixel

Clicking a tile, palette entry or BG projection shows VRAM/palette backing offsets,
map entry, flip, bank and index. OAM textures explicitly show slot, attributes,
hardware coordinates, size, affine matrix, priority and recorded host owner. The
texture is raw: no flip or matrix is applied; the corresponding pixel is a source
texel, not a screen position.

In the final framebuffer or composed OBJ, the winner and ROM source remain
**unknown**: matching color alone does not prove provenance. A VRAM tile is not
linked to a ROM resource without transfer evidence. Hardware OAM coordinates are
distinct from any extended host-object coordinates. Host owner is renderer metadata,
not ROM ownership.

Proven addresses cover only the declared frozen decoding. Actually read typed paths
remain in [DBG-06 audit](debug_rom_audit.md); backend execution differences remain
in [DBG-09 comparisons](debug_comparisons.md).

<a id="confrontare-e-conservare"></a>

## Compare and retain

`--compare-last` selects the latest complete report from the same copy, authenticates
ROM/schema and hashes of its buffers/images, then publishes a new report. It shows
side-by-side images and exact VRAM/OAM/palette/I/O/central/wide byte differences,
with the first 16 offsets. Offsets belong to named buffers, not ROM. With different
geometry, crop or settings, images are not aligned; byte differences remain declared
without automatic transformations.

A first comparison without a previous report is refused. A modified previous payload
is refused without overwriting retained reports. At most **eight directories per copy**,
including incomplete captures: no automatic pruning or replacement. Errors retain
artifacts for diagnosis; `graphics.json` is published last. A previous
`status.graphics_report` remains a previous capture when a new command fails: read
`ok/error` and current request identity.

Frontend captures accept windows up to **1600×1000**; larger sizes are refused
without silently rescaling UI. Only active wide area and necessary projections are
retained, with JSON limited to 2 MiB. ROMs, images, palettes and dumps are private:
Git-ignored and outside packages. Moving the whole copy preserves relative report links.

<a id="caso-verificato-pannello-ds-20-e-ritaglio-wide"></a>

## Verified case: DS-20 panel and wide crop

In the DS-20 copy at guest frame **22220**, the escape-from-battle panel (original
Italian game text: «Vuoi fuggire dalla battaglia?») distinguishes the UI canvas from
its visible area. With an **852×480** window, the 480×160 source canvas scaled 3×
has rectangle `[x=-294, y=0, w=1440, h=480]` and clip `[0,0,852,480]`. The negative
origin includes transparent wide-canvas padding; alone it does not prove panel text
is clipped.

Comparison of frontend composition with the view without UI measures **89,856
changed pixels** in rectangle `[114,48,738,192)`, 624 wide and 144 high: actual panel
content fits inside the clip. The side-by-side view, rectangles and count are retained
in the same report; PNG bytes are also checked independently of metadata. To reproduce
the observation, open the selected DS-20 seed in an isolated copy with `--paused`,
advance one frame with DBG-10, then capture two reports, the second with
`--compare-last`, retaining case view/settings. Seed and runtime identity remain in
the private manifest: another DS-20 save does not imply the same starting point.

Repeating capture without intermediate steps preserves complete state and produces
zero differences in the six raw buffers. In the BG0 projection, pixel `(65,10)` is
decoded from VRAM backing `0x06002091`; in the browser, color/alpha matches the
generated PNG pixel and declared palette/map. This proves frozen-projection decoding
without attributing that pixel to the framebuffer winner or a ROM source.

<a id="verifiche-e-limiti"></a>

## Checks and limits

Windows cross-build/Wine/Xvfb validation on 2026-10-28:

- Locked/offline release builds with and without audit, both without warnings;
  **58 distinct tests**: 3 report, 2 PPU, 6 control, 1 preview, 46 menu. Before GUI,
  saved RGB555 encoding (0xffff white) was distinguished from projection transparency;
  only the 3 affected report tests were repeated.
- **21 CLI cases** and GUI: menu capture, identical comparison and after a step,
  no predecessor, modified previous payload, eight-directory cap, refusal during
  execution/after closure. Capture and refusals preserve state, saves, progress and
  retained reports. The no-audit copy produces the same raw hashes and state as the
  audit-enabled copy.
- Five reports checked: buffer hashes/shapes, central/wide images compared pixel by
  pixel with RGB555, OAM attributes and exact differences; UI composition and bounding
  box recounted from PNG. Browser desktop 1400px and mobile 620px checked side-by-side
  images, pixel/palette/tile/OAM/window selection and no horizontal overflow; screenshots
  inspected.
- All 202 selected original inputs and normal Windows/Linux runtimes retained hashes
  and mtimes. Both copies closed correctly, without new snapshots. Temporary Chrome
  profiles were removed; reports and diagnostics retained under `target/debug-tools-graphics/`,
  ignored by Git.

Native Windows, native Linux/Wayland, physical controllers, native-runtime capture,
other scenes and the complete graphics-mode matrix remain unverified. CPU diagnostic
composition does not certify final GPU effects or the monitor. Current projections
do not reconstruct effects between scanlines or ROM transfer provenance. The guest
renderer and original preview sampler remain preserved. No new ROM/DS/native
acceptance follows from this check. [Change record](changes/2026-10-28-add-readonly-graphics-inspector.json).
