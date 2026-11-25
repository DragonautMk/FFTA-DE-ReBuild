# Extracted ROM resource viewer checklist

Start with a palette page that opens offline on Windows. A reader should be
able to select a color and find its exact value in the extracted file.

To follow one color today, open `manifest.toml` in an existing extraction under
`target/rom-data/<ROM filename stem>/`. If `resources` lists
`graphics.bg-base-palettes`, open the file named by its `path`
(`bg-base-palettes.json`). In the v2 JSON format,
`entries[0].banks[0]` holds the 16 color words for the first bank of background
descriptor 0. [Extracted files](../rom_data.md#extracted-files) ·
[Palette layout](../../crates/gba-core/src/rom_data/bg_palettes.rs)

## Build the first view

- [x] Generate an offline HTML page from a chosen folder containing
  `manifest.toml`. Read the extraction in place and write the page outside the
  dataset. Opening the page should require only a browser.
- [x] List the resources declared by that manifest. Show the regional profile
  and extractor/decoder revisions once. Let the reader search by English name,
  resource ID or filename. Start with the existing [file descriptions](../../scripts/extract_rom_data.py).
- [x] Open **Background base palettes** for `bg-base-palettes-v2-json`. Offer a
  descriptor and bank selector, then draw 16 swatches in slot order. Label each
  swatch with its slot, original 16-bit hex word and display color (`#RRGGBB`).
  Show the relative filename and field path. For example, the fourth swatch
  for descriptor 0, bank 0 comes from `entries[0].banks[0][3]`.
- [x] Keep entries with an unknown format in the index with their ID, path and
  format. If a declared file is missing or its SHA-256 differs from the
  manifest, show the reason on its row and keep the other resources browsable.

## Verify the first view

- [x] Check a synthetic bank with known colors. `0x001F`, `0x03E0` and
  `0x7C00` must display red, green and blue respectively. `0x801F` must display
  the same red as `0x001F` and retain its distinct stored value in the label.
  [Existing RGB conversion](../../crates/recomp/src/debug_graphics.rs)
- [x] In a synthetic dataset, try an unknown format, a missing file and a hash
  mismatch. In each case the affected row should explain the problem and a
  valid palette should still open.

- [ ] Open the generated page offline on Windows using an existing local
  dataset. Search for “palette”, choose a bank, and match one labeled swatch
  to the same slot in the JSON. Confirm that the input file hashes are
  unchanged. Keep the dataset and generated previews local.

Generate a page with Python 3.11 or later:

```powershell
py scripts/view_rom_data.py "target/rom-data/extracted-europe" "target/rom-data-views/extracted-europe"
```

With Docker Desktop running Linux containers on Windows:

```powershell
.\scripts\windows\view_rom_data_docker.ps1 "target/rom-data/extracted-europe"
```

On Linux:

```sh
bash scripts/linux/view_rom_data_docker.sh "target/rom-data/extracted-europe"
```

Both commands print the generated index path. By default the index is
`rom_data_viewer_generated.html` in the project root; its companion directory
is `rom_data_viewer_generated.html.parts/`. Both are ignored by Git. An optional
second argument chooses an output directory containing `index.html` and
`index.html.parts/`. Keep the index and companion directory together when
moving the wiki. They contain private extracted data. Python also accepts
`--single-page` for the legacy all-in-one output. Docker downloads the Python
image on first use, and generation runs without network access with the dataset
mounted read-only. Windows Docker execution remains unverified.
Linux Docker unit `rom-data-viewer-docker-eur-gpeufuwj` generated the expanded
EUR wiki byte-for-byte identically to the local Python output while protecting
the dataset and existing root page. Unit `rom-data-viewer-docker-current-k3a7sjpd`
repeated that comparison after adding the font atlas and name-entry keyboard;
`rom-data-viewer-docker-obj-drd29d_i` repeated it after adding object tile sheets.
Unit `rom-data-viewer-tilemaps-docker-25_e_ybg` compared current Docker and
Python outputs after the tilemap additions. Unit
`rom-data-viewer-battle-docker-eqvfwhko` repeated output parity after adding the
battle catalogs; `rom-data-viewer-audio-fidelity-docker-j4h2znpv` checked the
updated PCM preview; `rom-data-viewer-sequence-tempo-docker-cxqeosrd` checked
the tempo control; `rom-data-viewer-track-controls-docker-kk259nh0` checked
track controls; `rom-data-viewer-script-sprites-docker-5d6shlsb` compared the
output after adding the script sprite catalogs; `rom-data-viewer-item-links-docker-v6wfinxm`
checked the item-linked pair view; `rom-data-viewer-raster-docker-ksrnblac`
checked the raster color programs; `rom-data-viewer-encounters-docker-5cf1c9ua`
checked the encounter views; `rom-data-viewer-cgb-square-docker-ipvtpori`
checked square-wave audition; `rom-data-viewer-mission-events-docker-j560g7_l`
checked the mission and event views; `rom-data-viewer-terrain-docker-zsipb4vs`
checked initial terrain grids; `rom-data-viewer-script-paths-docker-izr26ayk`
checked script coordinate paths; `rom-data-viewer-alternate-palettes-docker-0kqk3x86`
checked alternate palettes; `rom-data-viewer-ui-choices-docker-overlap-rqql4vzm`
checked UI choices; `rom-data-viewer-fixed-pcm-docker-guard-uby5jpm5` checked
fixed-rate PCM; `rom-data-viewer-palette-programs-docker-bjxmey4e` checked
palette programs; `rom-data-viewer-obj-animations-docker-4b6h0gza` checked
indexed object animations; `rom-data-viewer-obj-families-docker-tyqp9wuf`
checked additional OBJ catalogs; `rom-data-viewer-obj-layers-docker-2suga0n4`
checked layered previews. Scoped generation uses the accepted file inspector
from `scripts/viewer_stable_inspector.html` and `.js`; the suspended large-JSON
prototype remains in the legacy single-page template and is not in the scoped
wiki.

Open the resulting HTML index locally. Its topic links open separate offline
pages for world/interface data, object animations, actors/equipment, battle
actors, script actors, audio, text/fonts, event scripts and other resources. The overview embeds
only index fields, and each topic page embeds its own projected data and the
small shared dependencies required by its previews. Full manifest metadata is
retained on its topic page. Folder selection and other page-session controls
are local to the current page. The contents sidebar links to the views available for that dataset;
each section can be opened or closed independently, and the sidebar can open or
close all sections. Resource rows link directly to their dedicated views when
available. On narrow screens the sidebar and resource rows stack vertically.
Linux Chrome units `rom-data-viewer-wiki-legs66p4` and
`rom-data-viewer-wiki-eur-s3qt2yel` checked navigation and section controls,
including the compact layout, EUR text selection, palette and audio controls.
Synthetic unit `rom-data-viewer-scopes-synthetic-5ely1ibk` checked scoped
navigation, data isolation, the text search, output ownership and legacy API
compatibility. Protected EUR unit `rom-data-viewer-scopes-eur-browser-mb9vtp3r`
checked all eight pages, all 150 manifest rows and 39 projected fields, and
measured the index at 327,744 bytes versus 77,415,026 bytes previously. Docker
unit `rom-data-viewer-scopes-docker-393oeeim` matched every generated file;
generator regression unit `rom-data-viewer-scopes-generator-regression-47_vtmyr`
passed 26 tests. An initial browser instrumentation failure in
`rom-data-viewer-scopes-eur-f4x0f7hf` was isolated with
`rom-data-viewer-scope-hook-diagnostic-fv97rttp`; its candidate was reused for
the successful guarded browser and Docker checks. Windows remains unverified.

Sections with dropdowns offer **Link to selected values**. Opening or copying
that link preserves the dropdown positions for the same generated dataset;
checkboxes, entered text, canvas choices and playback state are not included.
The query contains control IDs and numeric positions rather than extracted
labels. Invalid or unavailable positions are ignored when the link opens.
Synthetic Chrome unit `rom-data-viewer-bookmarks-synthetic-pm8g31r3` passed ten
selection, navigation and existing-control tests. Protected local-page unit
`rom-data-viewer-bookmarks-local-f58jm7ej` checked nine sections across seven
topic pages without changing embedded data. Docker unit
`rom-data-viewer-bookmarks-docker-16eits0l` matched Python on synthetic input;
the checks used isolated temporary roots while `target` was unmounted.

On each topic page, **Manifest details** opens the resource's file hash,
dependencies, additional manifest fields and source ranges. The source table
renders at most 25 ranges at once; it supports previous/next, page selection
and filtering by role, offset, length, hash or extra fields. The overview keeps
its smaller index and links to the appropriate topic page for these details.
Synthetic Chrome unit `rom-data-viewer-manifest-synthetic-reik4nzj` passed eight
metadata, text, job-name and scope tests. Protected local-page unit
`rom-data-viewer-manifest-local-6z5wltxb` checked all eight topic pages and the
large text catalog without changing any embedded dataset bytes. Docker unit
`rom-data-viewer-manifest-docker-6o05hy2l` matched Python output for a synthetic
dataset. These units ran in isolated temporary validation roots while the
shared target volume was unavailable; the verified local pages were published
with protected input and output hashes.

Playback controls show the current position for PCM samples, CGB waves and
approximate song tracks; the sequence timeline also displays a playhead. For a
looping PCM sample, the position follows the stored loop range. Clicking Play
preserves focus on the active audio selector. The executed instruction trace is
collapsed initially and can be expanded to inspect and page through instructions.
Protected local-page unit `rom-data-viewer-trace-fold-ready-tupw3uy9` checked
visibility, pagination, song changes and the existing mute control in Linux
Chrome. In text views, the language pin
and command-visibility checkbox are shared across the views; an unavailable
pinned language is reported. Sprite animation starts at the selected frame,
stops at the end unless looping is enabled, and keeps its rate, frame choice
where available, loop setting and playback state when the sprite changes.
These settings last for the current page session. Synthetic browser unit
`rom-data-viewer-controls-repo-final-a75dspzj` checked these interactions,
including real pointer focus; protected EUR unit
`rom-data-viewer-controls-eur-yl_quy7d` checked the controls with Web Audio and
the extracted dataset in Linux Chrome. Unit
`rom-data-viewer-controls-cleanup-s43t_yno` reran the permanent test and checked
that a browser setup failure releases its process and temporary link. Physical audio and Windows remain
unverified. The root page uses the previously validated file inspector; the
suspended large-JSON prototype remains in the template.

Linux Chrome headless checked color labels, search and bank selection with
synthetic data. It also matched a selected swatch from the generated EUR page
to the authenticated dataset in unit `rom-data-viewer-eur-palette-i7frj4x5`.
The Windows walkthrough remains open.
The page template is `view_rom_data.html` at the project root.

Choose the extracted dataset folder once in the resource index to add an
**Inspect selected file** link for each matching relative path. The selection
stays local to the browser; an individual file can still be selected separately.
The folder link matches names only and does not recheck file contents against the
generated manifest. Disconnecting the folder removes its links and releases the
currently inspected file if it came from that folder. Synthetic units
`rom-data-viewer-directory-ready-id2lnqf6` and
`rom-data-viewer-directory-early-2ynmp1y9` checked nested paths, ambiguous names,
replacement, disconnect and selection during initialization; hex-search
regression unit `rom-data-viewer-directory-regression-n61r4p2c` passed.
Protected EUR unit `rom-data-viewer-directory-eur-ready-1p18b2r4` matched all
150 indexed resources and opened a font JSON tree and a large file's raw pages
in Linux Chrome. Initial unit `rom-data-viewer-directory-eur-is0fbozr` timed out
because its folder selection preceded script initialization; diagnostic
`rom-data-viewer-directory-diagnostic-8o8n0vt6` confirmed that race.

Scoped generation also creates a separate **Browse extracted file** page for
an authenticated manifest resource up to 8 MiB. The index and topic pages
contain links rather than the file contents; opening one page loads that file
into the existing structured JSON and raw text/hex inspector. Missing files,
hash mismatches and larger files do not get a page; they can still be selected
through the file inspector when available. Each page contains a copy of the
original extracted bytes, so generated pages must remain private. Generation
retains older owned resource pages and refuses to overwrite an edited or unowned
page. Remove obsolete generated copies deliberately when they are no longer
needed.

Synthetic unit `rom-data-viewer-resource-pages-synthetic-45imfvy_` passed 37
tests covering offline navigation, structured JSON, invalid JSON, binary data,
size and hash limits, payload isolation and ownership. Protected EUR unit
`rom-data-viewer-resource-pages-real-_ueykdkg` checked 308 records in four
metadata files against four authenticated source ranges and complete extracted
file hashes, exercised all four pages in Linux Chrome, and matched all 17 files
produced by Docker and Python. The four pages and their index links were
published to the ignored local wiki. The first real unit `_u03rmwz` reached a
reporting TypeError from a shadowed counter; the corrected unit passed. The
complete source dataset was unavailable, so other local files need a fresh
viewer generation from the dataset. The large-JSON worker remains paused;
Windows was deferred. Primary Terra/xhigh settings were not independently
confirmed.

The **Inspect extracted file** control opens a local file selected by the
reader. Previous and Next display bounded portions; text mode overlaps a few
bytes to preserve UTF-8 characters across boundaries, and hexadecimal mode
shows offsets, byte values and printable ASCII. Files other than JSON, TOML
and TXT open in hexadecimal mode by default. Selection happens in the browser
and the page operates offline. **Find next** searches case-sensitive UTF-8 text
or exact hexadecimal bytes in bounded blocks, highlights the match or matching
hex rows and can be cancelled. After the last match, the next search restarts
from the beginning. Synthetic real-time Chrome unit
`rom-data-viewer-hex-realtime-q9ny0kwp` checked split UTF-8, repeated and
overlapping matches, cancellation, binary paging and hex search across a block
boundary. Protected EUR unit `rom-data-viewer-hex-eur-vhuxoa4j` checked hex
display and byte search in an extracted file beyond 8 MiB without modifying it.

JSON files up to 8 MiB can also be explored as objects and arrays, with at most
100 entries per page and a path back to the parent. Larger or invalid JSON and
TOML remain available through raw paging. Linux Chrome unit
`rom-data-viewer-json-tree-dppszikd` checked nested navigation, array paging,
invalid and oversized files, and recovery after selecting another file.

Arrays can additionally be displayed as tables of up to 25 entries and 32
fields per page. Open a record to inspect every field, including fields omitted
by the column bound; nested objects and arrays remain navigable. Both views can
jump directly to a zero-based entry position. Missing fields and JSON `null`
are shown distinctly. Integers outside JavaScript's safe range are flagged
because their parsed value may have lost precision, and numeric overflow is
labeled instead of being displayed as JSON `null`. Other numeric precision
loss is also possible; the raw file remains available. Synthetic Linux Chrome unit
`rom-data-viewer-json-table-synthetic-6psxozlb` checked both viewer modes,
nested values, mixed arrays, bounds, unsafe integers and the unchanged 8 MiB
limit. Protected local-page unit `rom-data-viewer-json-table-local-sxcx4xax`
checked all nine pages and a 376-record embedded catalog, without changing
embedded data. Docker unit `rom-data-viewer-json-table-docker-slgo431n`
matched Python on synthetic input. An earlier `2euj73r5` run failed because
the test iterated a single selected header; the corrected test passed.
Overflow and precision labels were additionally checked in synthetic Chrome
unit `rom-data-viewer-json-table-synthetic-zqkmnmv6`, all nine local pages in
`rom-data-viewer-json-table-local-fkxzxk_r`, and Docker/Python output parity in
`rom-data-viewer-json-table-docker-_wvjna3r`.

Array records can be filtered by direct field names or immediate scalar values,
case-insensitively. Nested objects are not searched. Results keep their source
indices, work in both list and table views, and survive opening a record and
returning to its array. The scan yields to the browser, can be cancelled and
stops after 10,000 matches with a notice; at most 20 array filters are retained
per selected file. Numbers are searched after browser parsing and may already
have lost precision. Synthetic Chrome unit
`rom-data-viewer-json-filter-synthetic-9c6m4e88` checked both viewer modes,
source indices, nested scope, result bounds, cancellation and file replacement.
Protected local-page unit `rom-data-viewer-json-filter-local-96wneefn` checked
all nine pages and filtering a 376-record embedded catalog without changing
the embedded data. Docker unit `rom-data-viewer-json-filter-docker-ft6e9rx4`
matched Python on synthetic input.

The template also contains a suspended, unverified prototype for loading a
large JSON structure on request in a browser worker. Synthetic unit
`rom-data-viewer-largejson-kdrp6i3r` stopped on a test-harness TypeError; the
worker's page-transfer bounds and EUR behavior still need validation. The
generated root page remains at the previously verified version. Resume the
prototype only when the paused viewer task is requested again.

## Add other views after the palette milestone

- [x] **Tilemap and rectangle explorer.** Decode the documented mode-4 runs,
  additional streams and row-major rectangles in `observed-tilemap.json`.
  Browse their raw cell values, inspect interpreted tile/flip/palette fields,
  and choose a graphics bank, first tile ID and palette for a 4-bit preview.
  The pairing is manual; choose either one color bank or the bank identified
  by each cell. Missing tiles are magenta and missing palette banks are cyan.
  Synthetic unit
  `rom-data-viewer-tilemaps-uyjmm_og` checked decoding, invalid inputs, colors,
  flips and missing tiles. Protected EUR unit
  `rom-data-viewer-tilemaps-eur-corrected-gmofczjl` independently compared all
  2,224 cells in eight maps and eleven rectangles, and checked a source pixel
  and navigation in Linux Chrome. No complete scene or original palette pairing
  is inferred. Initial unit `rom-data-viewer-tilemaps-eur-tuysj7hm` stopped
  because its comparison did not allow the expected resource-status change.
- [x] **Per-cell tilemap palettes.** Optionally use each interpreted cell's
  palette ID within the manually selected palette source. Unit
  `rom-data-viewer-tilemap-palettes-hh4zsvun` checked per-cell colors, flips,
  missing banks and single-bank mode using synthetic data. Protected local-page
  unit `rom-data-viewer-tilemap-palettes-eur-_o8ajeki` checked a nonzero palette
  ID against two source colors, switching modes and missing-bank feedback in
  Linux Chrome. Actual in-game source pairing remains unverified.
- [x] **Sprite dimensions.** Show the 12 entries in `obj-dimensions.toml` by
  sprite shape and size index, with width and height in pixels. The EUR dataset
  and Linux Chrome check passed in validation unit `rom-data-viewer-dimensions-nkajqoin`.
- [x] **Color-Mode palette fragments.** Browse all stored unit, companion,
  supplemental, shared and special-unit source fragments by book, physical
  variant, fragment and page. Swatches retain the full BGR555 word, bit 15,
  RGB approximation and exact source/JSON position. Partial fragments and
  gaps stay separate; the documented EUR companion mode 1 aliases variant 0.
  Four slice-selector bytes are shown in a collapsible table. Invalid fragment
  data does not suppress valid unit sprite palettes, or vice versa. Synthetic
  unit `cw368u1m` passed 37 viewer/generator checks. Protected EUR unit
  `i9kyxyne` reconstructed the complete catalog and its 30 source ranges,
  checked every one of 2,873 colors and four selectors in Chrome, matched the
  existing 48 unit banks, and compared 14 Docker/Python files. The verified
  ignored index and actors page were published. Live copies and fades, actual
  scene colors and Windows remain unverified.
  [Source contract](../rom_data_color_palettes.md#original-source-contract)
- [x] **Background descriptors.** Browse the stored root-relative sources,
  five transfer slots, optional variant, conditional selectors, alternate
  palette key and unclassified final byte. The displayed selection follows
  the documented environment-low-bits and `0xFF` fallback contract. Links open
  available tile, base/alternate palette, raster, terrain and transfer views at
  the same descriptor; tile sheets link back. Manifest-derived absolute offsets
  are labeled as such. Scene names and live scene behavior are not inferred.
  Synthetic unit `rom-data-viewer-bg-descriptors-synthetic-2gdpftqa` passed 49
  viewer tests, including missing resources and invalid descriptor data.
  Protected EUR unit `rom-data-viewer-bg-descriptors-real-7zygjuro` matched
  the full reconstructed JSON hash and its authenticated source range, all 162
  graphics-source and alternate-palette links, 1,782 displayed fields and 648
  selector combinations. It counted 899 available links and followed each of
  six view types in Linux Chrome, then matched 14 Python/Docker output files. The verified
  ignored local index and world page were published with prior data preserved.
  Windows is deferred; the large-JSON worker remains paused.
  [Descriptor contract](../rom_data_graphics.md#authenticated-descriptor-metadata)
- [x] **Initial background planes.** Apply the ordered indexed-pair packet
  writes and descriptor inheritance to two 64×64 planes. Choose an explicit
  initial fill, either plane or a manual layer order, linked or manually chosen
  graphics, grayscale/base/alternate colors and per-cell or fixed palette
  banks. Color zero can be transparent; missing tile/palette markers can be
  hidden while their counts remain visible. Selecting a cell shows both raw
  values, tile, flips, palette and whether it came from a stored write or the
  chosen fill. The preview is rendered when opened; descriptor and tile views
  link to it, and it links back to the tile sheet. It is a chosen interpretation,
  not a complete live scene.
  Synthetic unit `rom-data-viewer-bg-planes-synthetic-oot2ek2u` passed 49 tests.
  Protected EUR unit `rom-data-viewer-bg-planes-real-367dco_5` authenticated
  all 352 packet source ranges and the full extracted JSON, independently
  checked 1,327,104 cells across 162 descriptors, sampled five rendered pixels,
  and matched 12 Docker/Python output files. Visual unit `b_z0jgel` inspected
  three local previews. Follow-up `rom-data-viewer-bg-planes-mark-real-xg4uwxb5`
  checked hiding and restoring a real missing-tile marker without changing a
  known pixel, and matched 16 Docker/Python files on synthetic input. The
  validated ignored index and world page were published. Earlier `9rljf8_r`
  passed its tests but failed the input guard when Python rewrote a bytecode
  cache; the corrected run disabled bytecode writes. Primary Terra/xhigh
  settings were not independently confirmed. Windows is deferred; original live fill,
  layering, animation behavior, scene composition and the paused large-JSON worker
  remain outside this preview.
  Follow-up: optionally apply each linked transfer slot in order at the
  0x06000000 tile base, choose frames, enable slots, play at a chosen
  tick rate and loop. Controls persist across descriptor changes. This
  applies complete writes immediately; flags, partial DMA and original
  scene timing are not simulated. Synthetic unit `tiaqrdg1` passed 51
  tests; focused unit `sahvk9vy` passed three animation tests. Protected
  EUR unit `_7jf5jyj` reused the authenticated transfer projection,
  compared 283 independently reconstructed compositions and two real
  frame pixels, and matched 17 Docker/Python files. The validated ignored
  world page was published locally.
  [Plane contract](../rom_data_maps.md#live-terrain-and-display-state)
- [x] **Background tile sheets.** View decoded `bg-main-graphics.json` words
  as 4-bit 8×8 tiles, choosing a descriptor, stored palette bank or grayscale,
  and page. The view labels destination tile indices and does not invent a
  missing destination prefix or assemble a scene. Synthetic unit
  `rom-data-viewer-bgtiles-r4lnzrf7` checked nibble order, RGB colors, aliases,
  paging and invalid data. Protected EUR unit `rom-data-viewer-bgtiles-eur-7pw4d56v`
  compared all 162 descriptor links and 50 graphic word arrays and checked an
  independently calculated palette pixel and the last selection in Chrome.
- [x] **Background alternate palettes.** Browse both decoded catalogs by
  descriptor, environment low bits and bank. Each swatch shows its full stored
  word and source path; the linked tile sheet can use the chosen palette.
  Descriptor key `0xFF` is shown as unavailable and never dereferenced. Synthetic
  `rom-data-viewer-alternate-palettes-synthetic-pl5eja0m` checked swatches,
  colors, navigation, missing keys and separate catalogs. Protected EUR unit
  `rom-data-viewer-alternate-palettes-eur-y5byo3lu` compared 314 palettes,
  40,192 color words and all descriptor links with the extracted source, and
  checked a real swatch, tile pixel and unavailable key in Linux Chrome. The
  local page retained its prior data and inspector. Catalog selection does not
  reproduce live palette effects or establish a complete scene.
  [Alternate palette format](../rom_data_graphics.md#typed-alternate-palette-codec)
- [x] **Map raster color programs.** View each stored BGR555 color in source
  order as a horizontal band, inspect its raw word, RGB channels and bit 15,
  and navigate between the two descriptor-linked programs and the corresponding
  background tile descriptor. Equal color contents keep separate program IDs;
  palettes also remain browsable without background graphics. The bands do not
  reproduce live camera state, raster timing or a complete scene. Synthetic unit
  `rom-data-viewer-raster-synthetic-mr110a13` checked raw colors, pixels,
  descriptor navigation and standalone data. Protected EUR unit
  `rom-data-viewer-raster-eur-5v3q9gwr` compared all 32 programs, 5,120 colors
  and 324 links from 162 descriptors to the extracted source and checked two
  reference pixels and both navigation directions in Linux Chrome. The local
  page kept its prior data and inspector. [Raster source](../rom_data_maps.md#map-raster-palette-source-contract)
- [x] **Palette programs and stored color cycles.** Browse physical programs,
  headers, commands and tagged parameters. Display stored color words as
  swatches and preview color-cycle sequences from a selected color, at a chosen
  rate and optionally in a loop. Producer tables retain null pointers, delay
  overrides and reserved values, with links to their programs. Synthetic
  `rom-data-viewer-palette-programs-synthetic-8henqh4g` checked color words,
  playback, settings, focus, producer links and empty programs. Protected EUR
  `rom-data-viewer-palette-programs-eur-kh44a5o1` compared 230 programs, 1,135
  commands and 25 producer tables with their source, including 53 color cycles
  and 240 color-word commands; a real cycle and producer link passed in Chrome.
  The local page retained its prior data and inspector. The preview shows stored
  order at a user-chosen rate; original direction, task state, timing and live
  palette effects are not reconstructed.
  [Palette program format](../rom_data_graphics.md#palette-program-source-model-and-original-reader-validation)
- [x] **Blend and brightness programs.** Browse program control masks, initial
  coefficients, raw command durations and targets, and bidirectional producer
  references. Step through illustrative blend colors or play stored targets at
  a chosen rate, optionally looping. Coefficients are shown both as stored and
  as five-bit sample values; sample colors use coefficients capped at 16. The
  sample uses user-chosen colors and does not reproduce scene layers, live
  messages, interpolation or original command timing. Unit
  `rom-data-viewer-blend-synthetic-a8pa0_t7` passed 32 projection, browser and
  existing-view checks. Protected EUR unit `rom-data-viewer-blend-recovered-lv80oy7_`
  matched all 160 programs, 345 commands, six producer tables and 158 source
  ranges against the authenticated ROM and manifest, with byte-identical
  reconstructed JSON. The same unit checked the local browser and all 11
  generated Docker/Python files. The index and world page were published
  locally with their inputs guarded; the other pages and paused inspector were
  preserved. Windows remains unverified.
  [Blend program format](../rom_data_graphics.md#typed-blend-program-source-model)
- [x] **Animated background transfers.** Browse active descriptor/slot links,
  frame durations, raw control bytes, shared tile sources and VRAM destinations.
  Draw each stored frame as paged 4-bit tiles using a chosen base or alternate
  palette, or grayscale. Play from the selected frame using stored durations at
  a chosen tick rate, with optional looping and preserved controls. Background
  tile and transfer views link to each other where a descriptor has both. A zero
  duration is displayed as the original 65,536-tick countdown. The preview does
  not assemble a scene or simulate live flags, reverse playback or partial DMA.
  Synthetic unit `rom-data-viewer-bgtrans-synthetic-71aiupi2` passed 34 projection,
  browser and existing-view checks. Protected EUR unit
  `rom-data-viewer-bgtrans-recovered-7t3o9ck4` matched the authenticated JSON,
  97 links, 42 controls, 34 tile banks and 76 source ranges; it checked the first
  and last frame pixels for each link and compared all 11 Docker/Python files.
  The validated index and world page were published locally with guarded inputs;
  the paused inspector and unrelated pages remain unchanged. Windows is deferred.
  [Transfer format](../rom_data_graphics.md#european-background-transfer-programs)
- [x] **Interface tile sheets.** Browse `ui-graphics.json` banks as a 4-bit
  interpretation, using grayscale or a manually chosen `ui-palettes.json`
  bank. Selectors and language IDs open their linked bank when it belongs to
  this catalog; unavailable links and incomplete tile words are labeled.
  Palette pairings and assembled screens are not inferred. Synthetic unit
  `rom-data-viewer-uitiles-wzgtb2u7` checked pixels, palette selection, paging,
  selectors and missing links; `rom-data-viewer-uitiles-bg-regression-hegdmdgx`
  checked the shared background renderer. Protected EUR unit
  `rom-data-viewer-uitiles-eur-bsaejp9_` compared all 56 graphics banks, 22
  palette banks and 11 selectors, and checked reference pixels in Chrome.
- [x] **Object tile sheets.** Browse direct and relative-archive banks from
  `obj-graphics.json` as 4-bit tiles, using grayscale or a manually chosen unit
  or interface palette. Language selectors link to direct banks; incomplete
  words and unavailable links are labeled. Some decoded words are not pixels,
  and palette pairings and assembled sprites are not inferred. Synthetic unit
  `rom-data-viewer-obj-n60gjoi1` checked pixels, colors, paging and links.
  Protected EUR unit `rom-data-viewer-obj-eur-qbmy3m8d` compared all 39 direct
  banks, 103 archive banks and six selectors, then checked source pixels and
  the final archive page in Linux Chrome.
- [x] **Observed LZ77 outputs.** Validate each stored compressed stream against
  its decoded bytes, then browse the outputs as manually interpreted 4-bit or
  8-bit tile sheets. The same outputs are selectable in object-animation and
  tilemap previews. Context labels identify documented observations, not proven
  asset identities; incomplete tiles and missing palette colors are reported.
  Synthetic unit `rom-data-viewer-observed-lz-synthetic-p05_jv4a` passed 65
  decoder, browser and regression checks. Protected EUR unit
  `rom-data-viewer-observed-lz-recovered-1o7oqra1` independently decoded 25
  streams (58,936 bytes), matched their full catalog hash and 28 source ranges,
  checked 50 tile-depth previews and both linked viewers in Chrome, and matched
  all 12 Docker/Python files. Three ignored local pages were published with
  guarded inputs. Graphics interpretation and palettes are manual; live game
  context and native Windows remain unverified.
  [Observed sources](../rom_data_graphics.md#e304-firaga-battle-lz77-runtime-sources)
- [x] **Firaga transfer sources and palette prefixes.** Inspect all four stored
  transfer records, their wrapper and format fields, and the two palette
  entries. Follow the three documented links to decoded LZ outputs and the two
  links to partial three-color rows, with reverse navigation. The two row
  headers and six BGR555 words remain visible even without the transfer
  catalog; no additional declared colors or final palette placement are
  invented. Missing or invalid resources simply leave their links unavailable.
  Synthetic unit `rom-data-viewer-firaga-transfer-synthetic-nasbqw44` passed
  75 projection, browser and regression checks. Protected EUR unit
  `rom-data-viewer-firaga-transfer-recovered-8iu3yb8y` matched three complete
  catalog hashes and 42 source ranges, checked four records, three LZ links,
  two palette links and six colors in Chrome, and matched all 12 Docker/Python
  files. The ignored index and world page were published with guarded inputs.
  Live palette placement, scene composition and native Windows remain
  unverified. [Palette prefixes](../rom_data_graphics.md#e323-firaga-palette-row-runtime)
- [x] **LZ byte alignment.** Choose a byte offset into a decoded LZ output in
  the tile-sheet, object-animation and tilemap previews. The offset remains set
  when changing sources, and invalid values are reported. The projection retains
  up to three trailing bytes so unaligned reads do not lose source data.
  Synthetic unit `rom-data-viewer-lz-alignment-synthetic-455fmpyn` passed 75
  decoder, browser and regression checks, including unaligned tail bytes and
  incomplete 8-bit tiles. Protected EUR unit
  `rom-data-viewer-lz-alignment-recovered-ih3l6kex` matched the full catalog
  hash and 28 source ranges, checked 73,595 shifted words and 150 4-bit/8-bit
  previews across 25 streams in Chrome, and matched all 12 Docker/Python files.
  Two ignored local pages were published with guarded inputs; graphics meaning,
  palette pairing and native Windows remain unverified.
- [x] **Items.** Display `items.json`, labeling unclassified fields by offset
  and preserving signed values. The EUR records and a synthetic negative value
  passed in Linux Chrome unit `rom-data-viewer-items-ycfow_31`.
- [x] **Item-linked pair records.** Browse every record of `item-links.json`,
  including its count, opaque byte, all nine selector/value pairs and the
  original source offset. A checkbox hides stored slots beyond the count.
  Item byte `0x1D` links to its pair record; that record links back to all
  referencing items. Unreferenced records remain browsable, and pair data can
  also be viewed without the item catalog. The fields are presented as stored,
  without inferring live conditions or semantic names. Synthetic unit
  `rom-data-viewer-item-links-synthetic-zv3p1x1l` checked both navigation
  directions, inactive slots, missing and unreferenced records in Chrome.
  Protected EUR unit `rom-data-viewer-item-links-eur-corrected-41ll2bfw`
  compared all 225 records, 2,025 slots and 376 item links against their source,
  including five unreferenced records, and checked both links in Chrome.
  Initial unit `rom-data-viewer-item-links-eur-9piev0p6` reached the browser
  check but the test incorrectly treated a single element as a list; the
  corrected harness passed. The existing local payload and inspector were kept.
- [x] **Text.** Select a language and entry in `short-texts.json`. Draw the
  extracted glyphs at their recorded widths and label control codes and
  unresolved glyph IDs inline. The preview uses grayscale for glyph color indices.
  EUR data and Linux Chrome checks passed in `rom-data-viewer-text-q0apycln`.
  [Text format](../rom_data_text.md)
- [x] **Font glyph atlas.** Browse every slot in `font-bank.json` in pages or
  find an exact glyph ID. Display the full 16×16 slot, including pixels outside
  the recorded variable width, using grayscale indices rather than an inferred
  game palette or Unicode mapping. Synthetic unit
  `rom-data-viewer-font-jauowovw` checked pixels, paging, ID lookup, navigation
  and section controls. Protected EUR unit `rom-data-viewer-font-eur-_yk0kauf`
  compared all 3,175 glyph slots and widths and checked a reference pixel
  outside the width metric and the final slot in Linux Chrome.
- [x] **Name-entry keyboard.** Explore all extracted pages, row widths and
  codes; select cells to compose a local preview with remove and clear controls.
  Space and unavailable glyphs are labeled, and unknown codes remain visible.
  This preview does not enforce game name rules or infer Unicode. Synthetic unit
  `rom-data-viewer-keyboard-fqsl7l4j` checked page validation, composition,
  fallback and limits. Protected EUR unit
  `rom-data-viewer-keyboard-eur-c8bwq3nz` compared all 180 cells and row widths,
  then checked a source glyph pixel, cross-page composition, navigation and the
  existing font atlas in Linux Chrome.
- [x] **UI choice lists.** Browse each physical list, plot its stored cursor
  coordinates and select an entry to inspect its ID, opaque byte, raw navigation
  bytes and source offset. Lists with equal contents retain their separate IDs;
  live filtering, labels and navigation behavior are not inferred. Synthetic
  `rom-data-viewer-ui-choices-synthetic-overlap-5jqfovcx` checked the chart,
  selection, overlapping coordinates and raw byte values. Protected EUR unit
  `rom-data-viewer-ui-choices-eur-overlap-ger83d3k` compared all 13 lists and
  42 entries with their source and checked a filtered template and a real point
  in Linux Chrome. The local page retained its previous data and inspector.
  [Choice list format](../rom_data_ui.md#lossless-model-and-original-wine-readers)
- [x] **Script text archives.** Browse the physical banks and programs in
  `script-text.json` by language and draw their referenced glyphs from
  `font-bank.json`. Original command and operand bytes are labeled inline;
  dynamic substitutions and formatting are not executed. A missing font glyph
  is labeled explicitly. Synthetic unit `rom-data-viewer-script-text-gcw9ab5e`
  checked selectors, a glyph pixel, command bytes and missing glyphs. EUR unit
  `rom-data-viewer-script-text-eur-0liwj5t4` checked five languages, all 385
  bank counts, 1,155 sampled programs, all 147 referenced font glyphs and the
  final program selection in Linux Chrome. The dataset remained protected.
- [x] **Event scripts and text-bank references.** The separate event-scripts
  page browses keys, bank/slot targets and physical programs, including shared
  programs, bank headers and fill. Stored instructions are paged and filterable
  by opcode; their raw parameters and effect-selector IDs remain visible. The
  command directory shows each stored advance and Thumb callback. A key's text
  reference links to the appropriate physical bank in each available language;
  variant-bank selection is manual and the user chooses the text program there.
  The browser does not execute commands or infer branch flow and argument names.
  Synthetic unit `rom-data-viewer-scripts-synthetic-1uou13yo` passed 37 browser,
  scope, projection and regression checks. Protected EUR unit
  `rom-data-viewer-scripts-recovered-hg6j1egc` authenticated 234 keys, 62 banks,
  207 programs, 35,048 instructions, 115 command records and five language
  index maps against eight source ranges; both extracted JSON hashes matched.
  It checked all key links, the language mappings and actual text navigation.
  Unit `rom-data-viewer-scripts-bookmark-lyza0soc` checked a real shared-header
  bookmark, text navigation and byte-identical Docker/Python output for 12
  files after the bookmark correction. Ten ignored local pages were published
  with guarded inputs; existing embedded data outside the documented index and
  navigation changes was retained. Windows remains unverified.
  [Script format](../rom_data_scripts.md#relative-script-archive-and-event-key-source-contract)
- [x] **Pointer text catalogs.** Browse `text-catalogs.json` by reader,
  language and entry, preserving aliases and showing glyphs and control bytes.
  Unavailable glyphs are labeled; controls and substitutions are not executed.
  Synthetic unit `rom-data-viewer-pointer-text-dl67rv3e` checked alias and
  selector behavior, glyphs, control bytes and rejection of invalid indices.
  Protected EUR unit `rom-data-viewer-pointer-eur-dphvwy1l` compared all nine
  readers, 45 language tables, 16,100 strings and 118 referenced glyphs with
  the source and checked the last entry in Linux Chrome.
- [x] **Relative text archives.** Browse the separate archive programs in
  `text-catalogs.json` by reader, language and program. As with script text,
  glyphs and command bytes are shown without executing dynamic commands.
  Synthetic unit `rom-data-viewer-relative-text-hmje8ydk` checked selectors,
  commands and the recovery of pointer text when an archive index is invalid.
  Protected EUR unit `rom-data-viewer-relative-eur-zveyyx3w` compared all
  17,035 programs in 145 archives, their 123 referenced glyphs and earlier
  text projections, and checked the final program in Linux Chrome.
- [x] **Readable text presentation.** Hiding commands and controls leaves one
  visible space between adjacent text runs, including consecutive commands.
  Each text view offers a shared normal-font setting using Verdana/Arial for a
  visually matched European Latin subset; unrecognized glyphs retain their game
  bitmap, and other profiles are not mapped. This is a partial visual mapping,
  not an executed script or complete Unicode translation. Synthetic Linux
  Chrome unit `rom-data-viewer-equipment-text-synthetic-agj8p4xw` checked all
  four text views, spacing, settings, font and bitmap fallback. Protected EUR
  unit `rom-data-viewer-equipment-text-eur-final-787ku_bs` checked an actual
  script with 19 hidden command gaps and the shared settings in Chrome.
- [x] **Readable entry selectors.** The four text selectors show a bounded
  recognized-text preview alongside each index. Command runs become spaces,
  unknown glyphs are marked, and entries without recognized glyphs or outside
  the European profile keep their numeric labels. The option value and shared
  settings remain unchanged. Synthetic Chrome unit
  `rom-data-viewer-text-labels-synthetic-xdsx_e9s` checked labels, truncation,
  fallback, navigation and existing controls. Protected local-page unit
  `rom-data-viewer-text-labels-local-bogtcyng` checked all four selectors and a
  narrow viewport without changing embedded data; Docker unit
  `rom-data-viewer-text-labels-docker-eacja4wc` matched Python on synthetic
  input. These checks used isolated temporary roots while `target` was unmounted.
- [x] **Search recognized text.** Search the four text collections together or
  filter by archive and language. The case-insensitive search uses the partial
  European glyph mapping, represents unknown glyphs as �, inserts one space
  for command runs and does not execute substitutions. It runs in bounded,
  cancellable steps, stops after 1,000 matches, and pages through links to the
  original entries; opening a result updates the pinned language when enabled.
  Synthetic Linux Chrome unit `rom-data-viewer-text-search-synthetic-wtgeh9_2`
  checked links, shared preferences, gaps, unknown glyphs, pagination, limits,
  cancellation and an unmapped profile. Protected EUR unit
  `rom-data-viewer-text-search-eur-w25qy1t1` searched and followed a result in
  each archive; Docker unit `rom-data-viewer-text-search-docker-3uu3oerp`
  matched the validated Python output. The ignored local page retains its
  previous data and the verified file inspector. Windows remains unverified.
- [x] **Selected job names.** The template and generator provide a view
  for the 20 stored slot-job references, joining direct metadata52 property 0
  to the documented European `080165E8` name reader. Names are rendered from
  extracted glyphs, with links to their localized pointer strings; redirects and
  missing data do not receive guessed names. This does not establish live slot
  selection or enumerate all jobs. Synthetic Chrome unit
  `rom-data-viewer-job-names-synthetic-2_l9y4cr` passed six tests; current
  regression unit `rom-data-viewer-job-names-current-7m1igk75` also checked
  scoped pages and text preferences. With the original dataset volume
  unavailable, protected unit `rom-data-viewer-job-names-recovered-b049typ1`
  authenticated the EUR ROM and retained manifest, reconstructed two extracted
  JSON files byte for byte, and checked all 20 joins, five 753-entry pointer
  tables, 3,765 source strings and 49 selected glyphs against the ROM and
  manifest hashes. It checked all language links and cross-page navigation in
  Chrome, plus Docker/Python parity for a reduced dataset reconstructed from
  those verified inputs. The local wiki was updated with guarded inputs and
  unchanged unrelated data. This is not a reconstruction of the complete
  extracted text file or proof of other job names. Earlier `b8hp7a8l` failed
  when the verifier reused a variable while writing its result; the corrected
  unit passed. An earlier synthetic click test hit a gap between two lines of
  an inline link; the passing test clicks an actual line fragment.
- [x] **Other documented metadata names.** The selected-job view also shows
  localized text for metadata rows 104–106, separately from the twenty stored
  slots. These rows were identified by the project's earlier local text lookup;
  that evidence does not establish live usage. Nonzero routing and unavailable
  strings remain unresolved. Synthetic Chrome unit
  `rom-data-viewer-job-names-current-pd57j0o5` checked all three rows,
  language changes, missing data and redirected records. Protected unit
  `rom-data-viewer-job-references-recovered-mx9555mc` authenticated the source,
  checked the documented English labels and all 15 localized links, verified
  cross-page navigation and matched Docker/Python output for a reduced dataset.
  It preserved the previously validated twenty-slot data and all unrelated
  embedded fields. The complete original text JSON was still unavailable.
  [Earlier lookup evidence](../rom_data_gameplay.md#ordinary-alternate-record-use-and-cli)
- [x] **Sprite tile sheets.** Display the 8×8 indexed tiles from
  `unit-obj-initial-pose.json` at integer zoom using grayscale palette indices.
  EUR catalog and Linux Chrome checks passed in `rom-data-viewer-sprite-ii7_y869`.
- [x] **Sprite frame links.** Select a sprite ID and frame to open its source
  tile block and inspect the object-list address and duration. Linux Chrome
  checked the links against EUR data in `rom-data-viewer-sprite-binding-wvf5hxk_`.
- [x] **Sprite pose.** Assemble supported non-affine 4bpp poses using a selectable
  extracted unit-base palette and 1D tile order. Live palette overrides are
  excluded from this static preview. An initial EUR frame, palette and reference
  pixel passed in Linux Chrome unit `rom-data-viewer-pose-gkpm1ziw`.
  [Sprite format](../rom_data_unit_obj.md)
- [x] **Sprite frame animation.** Play and stop the extracted initial-pose frame
  sequence at a selectable preview rate, using each stored duration and a
  shared frame origin. Zero durations use one preview tick; this does not claim
  game timing or support for additional object modes. Synthetic unit
  `rom-data-viewer-spriteanim-58wsec92` checked frame alignment, pixels, timing,
  stop and invalid rates. Protected EUR unit
  `rom-data-viewer-spriteanim-eur-4ekn367f` compared all 121 bindings and 477
  frame durations and checked a reference pixel and animation in Chrome.
- [x] **Static object lists.** Select a static source list by index and offset
  and inspect its objects in the shared layout renderer. Choose graphics and a
  palette manually; static lists are explicitly labeled and cannot be played
  as animations. Empty lists remain selectable, and existing animation controls
  resume when switching back to an animation catalog. Synthetic unit
  `rom-data-viewer-static-obj-synthetic-trxww2fv` passed 44 projection, browser
  and regression checks. Protected EUR unit
  `rom-data-viewer-static-obj-recovered-fxvkosah` matched 289 source ranges and
  the full reconstructed catalog JSON hash, then checked all 274 lists, 1,229
  objects and six empty lists in Chrome. All 12 Docker/Python output files
  matched; two ignored local pages were published with guarded inputs. Other
  fields in the catalog remain available through the file inspector; graphics
  and palette choices are illustrative, special object effects and native
  Windows remain unverified. [Static format](../rom_data_obj.md#format-and-original-consumers)
- [x] **Indexed object animation banks.** Browse physical banks, sequences,
  frame durations, object attributes and language selector links. Animate from
  the selected frame at a chosen rate, optionally looping, while retaining
  settings across selections. Manually selected OBJ graphics and palettes can
  render supported 4-bit and 8-bit objects with 1D tile order; unavailable
  tiles, special modes and missing colors are labeled. Reserved shapes remain
  inspectable as raw attributes. Empty sequences and documented zero-count
  holds do not invent frames. Synthetic unit
  `rom-data-viewer-obj-animations-synthetic-2vqot9k2` checked pixels, flips,
  palette IDs, missing data, settings, timing, focus and standalone layouts.
  Protected EUR unit `rom-data-viewer-obj-animations-eur-o6npg9q5` compared
  all 18 banks, 355 lists, 1,161 objects, 109 sequences and 382 frames with
  their source, then checked a real manually paired pixel, playback, selector
  and zero-count hold in Chrome. The local page retained its previous data and
  inspector. Graphics/palette pairing is manual; live task timing, affine,
  mosaic and special object effects are not reconstructed.
  [Indexed animation format](../rom_data_obj.md#indexed-ui-animation-banks)
- [x] **Word-task, layered and title OBJ banks.** The indexed object view also
  browses the four word-task banks, sixteen layered banks and two independently
  rooted title banks. Resource links select the corresponding catalog; playback,
  rate, loop, graphics and palette settings remain active across catalogs.
  Layered descriptor bytes remain visible. Synthetic
  `rom-data-viewer-obj-families-synthetic-vr3kmnd_`
  checked catalog navigation, settings, title banks and standalone resources.
  Protected EUR unit `rom-data-viewer-obj-families-eur-8sz6o368` compared all
  22 additional banks, 637 lists, 2,539 objects, 44 sequences and 631 frames
  with their sources, then checked a real manually paired pixel from each main
  family and both title banks in Chrome. The local page retained its previous
  data and inspector. Graphics/palette pairing and timing remain approximate;
  the other layered resource fields are not reproduced.
  [Word-task format](../rom_data_obj.md#european-obj-banks-with-u16-task-counters)
- [x] **Compact OBJ animation families.** Browse and play the eight witnessed
  compact catalogs in the same object-animation view. Sparse list IDs and
  sequence selectors remain visible instead of being renumbered, while each
  frame links to its decoded list. Playback, loop, rate and manual graphics and
  palette choices persist across catalogs. Resource links open the matching
  family on the object-animation page. Synthetic unit
  `rom-data-viewer-compact-synthetic-sbzmnfjp` passed 35 projection, browser and
  existing-view checks, including sparse IDs and manually paired pixels.
  Protected EUR unit `rom-data-viewer-compact-recovered-tca_p6t8` matched all
  eight JSON hashes and 22 source ranges, then checked 171 lists, 33 sequences
  and every one of their 338 frames in Chrome. The same unit compared all 12
  Docker/Python output files. Three ignored local pages were published with
  guarded inputs. Only the extracted caller-witnessed projections are shown;
  graphics and palette pairings, original timing and unsupported object modes
  remain unverified. Windows is deferred.
  [Compact OBJ format](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80)
- [x] **Witnessed compact OBJ catalog banks.** The larger compact catalog is
  available in the object-animation view, with bank flags, source selectors,
  sparse object-list IDs and frame playback. Synthetic
  unit `rom-data-viewer-compact-catalog-synthetic-mrrrizzs` passed 37 projection,
  browser and existing-view checks. Protected EUR unit
  `rom-data-viewer-compact-catalog-recovered-s0qzxiwk` matched all 322 source
  ranges and the full reconstructed JSON hash, then checked 63 banks, 651
  lists, 212 sequences and 1,032 frames in Chrome. It also matched all 12
  Docker/Python output files. Three ignored local pages were published with
  guarded inputs. Graphics/palette pairing and timing remain illustrative;
  unsupported object effects and Windows remain unverified.
  [Catalog format](../rom_data_compact_catalog.md#sparse-format)
- [x] **Compact catalog source tables and records.** Browse, filter and page
  through the fourteen selector/position tables, two record groups and three
  soil metadata groups in a collapsible object page section. Stored values and
  offsets are preserved; documented bank/selector relationships link to the
  corresponding animation, with reverse links back to source rows. Unsupported
  or absent references remain unlinked; the view does not simulate the game
  conditions selecting them. Synthetic unit
  `rom-data-viewer-compact-tables-synthetic-nrfs51kb` passed 40 projection,
  browser and regression checks. Protected EUR unit
  `rom-data-viewer-compact-tables-recovered-2l7_woa1` matched all 322 source
  ranges and the full reconstructed JSON hash, checked all 300 rows and 278
  links in 19 groups in Chrome, exercised four two-way navigation paths and
  matched all 12 Docker/Python output files. Two ignored local pages were
  published using guarded preimages. Native Windows remains deferred.
  [Catalog format](../rom_data_compact_catalog.md#sparse-format)
- [x] **Soil Evidence sequence.** Join the documented selector-zero compact
  metadata to the twenty separately owned static OBJ lists and preview the
  stored frame order and durations. The compact source-table section links to
  this animation, and the animation links back to the metadata groups. The
  view is omitted if either input or an expected source binding is absent.
  Graphics, palettes and preview timing remain manual/illustrative. Synthetic
  unit `rom-data-viewer-soil-anim-synthetic-7s5snhj6` passed 48 projection,
  browser and regression checks. Protected EUR unit
  `rom-data-viewer-soil-anim-recovered-9pd0bjc5` authenticated both complete
  catalogs and 611 source ranges, checked 20 frames, 20 lists and 91 objects
  in Chrome, exercised playback and looping, and matched all 12 Docker/Python
  files. One ignored local page was published with guarded inputs; native
  Windows and live game conditions remain unverified.
  [Metadata format](../rom_data_compact_catalog.md#e275-soil-evidence-compact-metadata-runtime)
- [x] **Firaga object sequences.** Browse and play the four documented indexed
  and compact sequences, preserving original list IDs, frame durations,
  source offsets and descriptor fields. Their resource links open the matching
  object catalog. Only referenced lists must be present; missing or invalid
  bindings leave that particular preview unavailable. Graphics and palettes
  are manually selected, and playback timing is illustrative. Synthetic unit
  `rom-data-viewer-firaga-obj-synthetic-y5_aesx4` passed 52 projection, browser
  and regression checks. Protected EUR unit
  `rom-data-viewer-firaga-obj-recovered-z7zebfoo` authenticated five complete
  catalog hashes and 310 source ranges, checked 59 frames and 419 frame-object
  instances in Chrome, exercised playback and looping, and matched all 12
  Docker/Python files. Three ignored local pages were published with guarded
  inputs; native Windows and live game context remain unverified.
  [Indexed](../rom_data_obj.md#e312-firaga-indexed-obj-runtime) ·
  [Compact](../rom_data_obj.md#e314-firaga-compact-obj-runtime) ·
  [Adjacent](../rom_data_obj.md#e316-firaga-adjacent-compact-obj-runtime) ·
  [6c12c8](../rom_data_obj.md#e318-firaga-6c12c8-compact-obj-runtime)
- [x] **Firaga compact cell frames.** Decode the ten witnessed compact grids
  into the tilemap explorer, inspect individual cells, source run controls,
  opaque frame headers and sequence fields, and preview the stored entry order
  at a manually chosen frame rate. Unavailable frame references stop playback;
  their original fields remain visible. Graphics and palettes are selected
  manually, and neither game timing nor the unknown second byte is inferred.
  The compact and observed-tilemap resource links remain independent when
  either catalog is invalid. Synthetic unit
  `rom-data-viewer-firaga-cells-synthetic-lnz9dru5` passed 59 decoder, browser
  and regression checks. Protected EUR unit
  `rom-data-viewer-firaga-cells-guard-recovered-ca3ejejo` matched the complete
  catalog hash and nine source ranges, checked ten frames, 497 cells and ten
  sequence entries in Chrome, exercised playback and matched all 12
  Docker/Python files. The ignored index and world page were published with
  guarded inputs; native Windows and live game context remain unverified.
  [Compact cell format](../rom_data_graphics.md#e310-firaga-compact-cell-runtime)
- [x] **Firaga and Soil Evidence source grids.** Inspect the two documented
  copied matrices and two Soil Evidence rectangles in the tilemap explorer,
  including every stored cell and the associated pointer fields. Their resource
  links open the correct grid independently; manually selected LZ outputs can
  be interpreted as tile graphics. Synthetic unit
  `rom-data-viewer-source-grids-synthetic-q5cw3qvv` passed 70 projection,
  browser and regression checks, including missing and invalid catalogs.
  Protected EUR unit `rom-data-viewer-source-grids-recovered-vthg8e9o`
  matched both full catalog hashes and nine source ranges, checked all four
  grids and 280 cells in Chrome, and matched all 12 Docker/Python files. The
  ignored index and world page were published with guarded inputs. Cell bit
  interpretation, graphics and palettes remain manual; the live scene and
  native Windows remain unverified.
  [Firaga matrices](../rom_data_graphics.md#e308-firaga-copied-matrix-runtime) ·
  [Soil rectangles](../rom_data_graphics.md#e301-soil-evidence-remaining-rectangle-runtime)
- [x] **Overlaid layered previews.** Optionally draw the sequences of a layered
  descriptor together at a shared origin. Each layer advances independently
  using its stored duration plus one preview tick; a table shows its current
  frame and object list. Loop and overlay choices persist when changing banks
  and catalogs. Synthetic `rom-data-viewer-obj-layers-synthetic-6zxvot5n`
  checked simultaneous pixels, independent frames, looping, empty selected
  layers and retained settings. Protected EUR unit
  `rom-data-viewer-obj-layers-eur-knhvoom1` checked all ten multi-layer banks,
  six playback checkpoints and two manually paired pixels from separate
  layers in Chrome. The local page retained its previous data and inspector.
  The shared origin, selected graphics/palettes and preview clock do not
  reproduce live offsets, task state, scene composition or the original clock.
  [Layered source](../rom_data_obj.md#conditional-layered-obj-animation-source-at-3ae1a4)
- [x] **Equipment sprite sequences.** Browse sprite IDs, animation/part bindings,
  null entries, frame durations, raw command fields and graphics in a separate
  collapsible section. Preview supported 4bpp poses with grayscale or a manually
  selected unit palette and animate from the selected frame, with optional
  looping. Graphics remain visible through non-graphics commands and clear on
  operation 6; live palettes, actor composition and original timing are not
  reproduced. Synthetic unit `rom-data-viewer-equipment-text-synthetic-agj8p4xw`
  checked links, command frames, nulls, pixels, timing and settings. Protected
  EUR unit `rom-data-viewer-equipment-text-eur-final-787ku_bs` compared all 748
  bindings, 258 sequences, 3,439 frames and 280 pixel blocks with the source and
  checked a reference pixel and controls in Chrome. Earlier units `2x2_nqve` and
  `f2rcnchf` failed in the validation harness: the pixel candidate was too
  restricted, then a query returned one element instead of a list. The corrected
  unit passed; the prior local payload and paused JSON prototype were preserved.
- [x] **Normal and alternate battle sprites.** The sprite sequence section also
  browses the two battle catalogs and links each indexed resource to the matching
  catalog. It retains animation, part, null and shared index-root targets, shows
  command details and plays supported 4bpp frames with manual palette and
  preview timing. A shared index root is labeled as an index, never as a sequence.
  Task commands are reported but are not executed; live palettes and actor-part
  composition remain outside the preview. Synthetic Chrome/Python unit
  `rom-data-viewer-battle-synthetic-sz5wql3k` checked catalog navigation,
  commands, pixels, null/shared targets and settings. Protected EUR unit
  `rom-data-viewer-battle-eur-dj9tn7wi` compared all 16,830 bindings, 3,287
  sequences, 20,665 frames and 4,419 pixel blocks to their sources, then checked
  a pixel from each catalog and a shared root in Chrome. The local page retained
  the previously validated inspector and unrelated payloads.
- [x] **Script sprite catalogs.** Browse the five extracted special-normal,
  special-alternate, override, registry-special and registry-generic catalogs
  in the same sprite sequence section. Each resource links to its own catalog.
  A separate collapsible, paginated table shows the matching producer conditions,
  metadata or assignment source and script offsets. These witnesses are
  conditional source relationships, not evidence that a particular actor uses
  a sequence during a particular scene. Nulls and shared index roots remain
  distinct, and animation settings persist across catalog and sprite changes.
  `rom-data-viewer-script-sprites-synthetic-1w6hgbmp` checked five script-only
  catalogs, pagination, source fields, alias/null handling, pixels and controls.
  Protected EUR unit `rom-data-viewer-script-sprites-eur-ykbzrpvg` compared
  all 4,308 producers, 5,778 bindings, 1,204 sequences, 7,619 frames and 2,899
  pixel blocks with their sources and checked a real pixel, pagination and a
  shared root in Linux Chrome. The local page retained the prior inspector and
  unrelated payloads. Live composition, palette, task commands and original
  timing remain outside the preview. [Script sprite formats](../rom_data_unit_obj.md)
- [x] **Maps.** Plot the tile coordinates from `world-nodes.json`. Let the
  reader select a node to see its ID, coordinates and recorded neighbors.
  EUR data and Linux Chrome selection passed in unit `rom-data-viewer-map-povy0tl_`.
  [World map data](../rom_data_maps.md)
- [x] **Initial terrain grids.** Reconstruct each normal 16×16 auxiliary seed
  by applying its ordered parent and child halfword writes to a zero-filled
  grid. Browse heights, flags and raw values, choose cells and color mode, and
  follow descriptor links to background tiles. A matching encounter's recorded
  coordinate list can be overlaid; out-of-grid points are counted and header
  pointer/count mismatches are reported. The default map association is
  conditional: scripts, live placement and later terrain writes can differ.
  Synthetic unit `rom-data-viewer-terrain-synthetic-r488fgwc` checked ordered
  overwrites, inheritance, cycles, colors, partial resources and overlays.
  Protected EUR unit `rom-data-viewer-terrain-eur-2dokwg9_` compared all 162
  seed hashes (41,472 cells) with the previously retained independent evidence,
  including nine inherited descriptors, and checked an inherited grid and
  encounter overlay in Linux Chrome. The previous local payload and inspector
  were kept. [Terrain seed source](../rom_data_maps.md#live-terrain-and-display-state)
- [x] **Script coordinate paths.** Browse ordered XY points, source alignment
  bytes and direct script references. Select a point and preview the remaining
  path at a chosen rate, optionally looping; playback, rate and backdrop settings
  remain selected when changing paths. A terrain grid can be chosen manually as
  a backdrop, with links in both directions and a count of visible points.
  Synthetic unit `rom-data-viewer-script-paths-synthetic-ctd_lkz_` checked
  playback, focus, settings, references, out-of-grid points and standalone
  paths. Protected EUR unit `rom-data-viewer-script-paths-eur-ihwu8qrl`
  compared all 210 paths, 856 points and 68 direct references with the extracted
  source, then checked real points, a marker pixel, playback and terrain links
  in Linux Chrome. The local page kept its previous data and inspector. Terrain
  selection does not imply a map relationship; actor selection, live height,
  script execution and original movement timing are not reproduced.
  [Path source](../rom_data_scripts.md)
- [x] **Encounter records and placement lists.** Browse extracted encounter
  headers, unit templates and coordinate lists in individually collapsible
  panels. Header fields, opaque bytes, unit attributes, equipment and physical
  source offsets stay visible; equipment IDs link to items and the item view
  links back to template references in pages. Coordinate lists show both signed
  and unsigned byte values and plot a selected interpretation. Header pointer
  and count mismatches are labeled; the views also tolerate missing catalogs.
  Recorded lists do not assign positions to units or reproduce live placement,
  RNG, unit initialization or map appearance. Synthetic unit
  `rom-data-viewer-encounters-synthetic-xr3_x1jv` checked raw values, signed
  plots, partial resources, inconsistent pointers and bidirectional links.
  Protected EUR unit `rom-data-viewer-encounters-eur-dtibuac9` compared all
  442 headers, 2,850 unit templates, 691 lists, 2,342 points and 14,250 equipment
  references with the sources, and checked a real plot and item navigation in
  Linux Chrome. The local page retained its previous data and inspector.
  [Encounter coordinates](../rom_data_maps.md#european-encounter-coordinate-lists-and-original-readers)
- [x] **Mission, event and observed parameter records.** Browse all stored
  fields and source offsets in separate collapsible panels. Follow a mission's
  stored event row, an event's stored parameter index and matching primary
  encounter ID, with paginated reverse references. The parameter selector
  retains sparse extracted indices; missing records remain missing. Matching
  primary IDs are labeled as conditional source references, not proof of live
  selection or mission availability. Synthetic unit
  `rom-data-viewer-mission-events-synthetic-6i5tbslj` checked navigation,
  pagination, missing resources, sparse IDs and signed fields. Protected EUR
  `rom-data-viewer-mission-events-eur-corrected-qbbojah0` compared 512 missions,
  256 events and 59 observed parameter records, checked their stored references,
  and exercised a real chain and large reference pages in Linux Chrome. The
  initial EUR unit `hjbckpaj` incorrectly iterated one DOM element in its test;
  the corrected harness passed. The prior local payload and inspector were kept.
  [Event source](../rom_data_gameplay.md#european-event-record-source-contract)
- [x] **Sampled world roads.** Overlay uniquely linked roads from
  `world-roads.json` on the node plot using each sample as a pixel offset from
  the lower-index node. Select a road or its node link to inspect and animate
  stored samples in either direction, at a chosen preview rate with optional
  looping; no endpoint is added and this is not game timing. Node links choose
  the matching direction, and the road controls can be folded away. Synthetic unit
  `rom-data-viewer-roads-io3__c78` checked short endpoints, reverse node links,
  sample selection and animation. Protected EUR unit
  `rom-data-viewer-roads-eur-h9_e9pfp` compared all 45 road sample arrays and
  node pairs and checked a short endpoint and marker in Chrome. Follow-up
  synthetic unit `_9uq_hpr` passed 36 tests; protected EUR unit `4fht7ab9`
  checked 4,376 sample positions in both directions across all 45 roads,
  authenticated 47 node/road source ranges and matched 15 Docker/Python
  files. The validated ignored world page was published; Windows remains
  deferred.
- [x] **Audio.** Draw one waveform from the signed PCM values in
  `audio-tones.json`. Show its length and loop start alongside the source entry.
  EUR sample values and Linux Chrome canvas passed in unit `rom-data-viewer-audio-6k9qlyf2`.
  [Audio format](../rom_data_audio.md)
- [x] **Stored audio values.** Inspect PCM header fields, signed sample values
  and their raw bytes in pages of 64. Jump to an index or click the waveform;
  selecting a sample does not stop playback. A separate collapsible table
  exposes the 32 stored amplitudes of each CGB wave. Synthetic Chrome unit
  `rom-data-viewer-audio-values-synthetic-s_dkk82o` checked paging, navigation,
  empty data and playback. Its earlier `izxyk4hb` run selected an adjacent
  sample because the synthetic click was rounded; diagnostic `klnybce8` and
  a click within the target bin resolved the test. Protected local-page unit
  `rom-data-viewer-audio-values-local-bqrh1s5u` checked all 277 PCM headers,
  44,744 sample values across 831 representative pages and both CGB waves;
  embedded data stayed unchanged. Docker unit
  `rom-data-viewer-audio-values-docker-j378b8gu` matched Python on synthetic
  input. The source dataset volume was unavailable for the local-page check.
- [x] **PCM audition.** Play one extracted PCM wave at a selectable sample rate.
  The initial 22,050 Hz rate is an audition setting; original pitch, loops and
  envelopes depend on the game sound driver. Synthetic interaction and EUR
  sample selection passed in Linux Chrome units `rom-data-viewer-playback-k6ro9a40`
  and `rom-data-viewer-playback-eur-y0nh6nbz`. Audible device output remains
  unverified in headless Chrome.
- [x] **PCM loop and WAV export.** Optionally repeat a wave from its recorded
  loop point and download one unlooped 8-bit mono WAV at the chosen audition
  rate. The option does not assert that the game enables looping for that wave;
  the WAV does not include loop metadata. Synthetic unit
  `rom-data-viewer-wav-1zs24m0c` checked signed sample conversion, WAV headers,
  padding, download, loop boundaries and invalid inputs. Protected EUR unit
  `rom-data-viewer-wav-eur-zm3n1sb2` matched all 277 waves to the dataset and
  checked every converted sample and one loop point in Linux Chrome.
- [x] **CGB wave audition.** Draw and play an extracted 32-step waveform at a
  selectable frequency for one second. This is a synthetic note, not a replay
  of the game's mixer or envelope. EUR data, selection and Web Audio controls
  passed in Linux Chrome unit `rom-data-viewer-cgb-eur-qfpcr8ak`; physical audio
  output was not checked.
- [x] **Sequence track audition.** Select a song and audition one decoded
  track or all its tracks together with manual or estimated tempo. The
  bounded instruction trace and note table identify the track; the mix level
  is limited based on its maximum concurrent notes. Waits, jumps, patterns,
  returns and note ties are followed. The preview can span 1–180 seconds and
  stops at a per-track work limit; a combined preview allows at most 128,000
  instructions and 8,000 notes across tracks. Notes are paginated and audio
  is scheduled progressively; delayed or skipped notes are reported if browser
  scheduling falls behind. Exact game timing, modulation, CGB envelopes and
  effects are not reproduced. Synthetic unit
  `rom-data-viewer-sequence-03jtgkmw` checked branch/return behavior, ties,
  loop limits, pagination, empty songs and Web Audio controls. Protected EUR
  unit `rom-data-viewer-sequence-eur-mli6tkjk` compared all 105,253 decoded
  instructions and successors, 1,610 tracks and 640 directory entries, then
  checked selection and audio controls in Linux Chrome. Audible device output
  and original-driver fidelity remain unverified. Synthetic units
  `rom-data-viewer-multitrack-q62ctukb` and
  `rom-data-viewer-multitrack-mix-e4bpfoob` checked individual selection,
  simultaneous scheduling, gain bounds and stop behavior. Protected EUR unit
  `rom-data-viewer-multitrack-eur-919lx0o8` checked a twelve-track song and
  switching between the combined and individual views in Linux Chrome.
- [x] **Longer sequence previews.** Synthetic real-time unit
  `rom-data-viewer-longaudio-a5q2rn7d` checked selectable duration, note pages,
  progressive scheduling, cancellation, late-note reporting and bounds;
  `rom-data-viewer-longaudio-pcm-regression-tkh5huj_` and
  `rom-data-viewer-longaudio-mix-regression-ehd71h0c` checked existing PCM and
  mix behavior. Protected EUR unit `rom-data-viewer-longaudio-eur-jds4jb_h`
  checked a thirty-second twelve-track preview, 180-second limits and stopping
  the scheduler in Linux Chrome.
- [x] **PCM instruments in sequence audition.** Resolve program changes through
  the extracted tone banks and key maps, play supported PCM samples with their
  stored loop flags, and label unmapped or unsupported voices as sine fallbacks.
  The user can still choose sine waves for every note. The audition does not
  reproduce exact driver timing, modulation or effects. Synthetic unit
  `rom-data-viewer-pcmvoice-51yozacf` checked direct and split mappings, sample
  data, rate, loops and fallbacks; `rom-data-viewer-pcmvoice-legacy-emswkwfc`
  checked datasets without instrument metadata. Protected EUR unit
  `rom-data-viewer-pcmvoice-eur-07frb244` compared the four banks, 429 tones,
  348 key links and 455 headers, then checked mapped notes and PCM Web Audio
  start/stop in Chrome. Physical audio and original-driver fidelity remain
  unverified.
- [x] **Fixed-rate PCM instruments.** Play supported type `0x08` tones from
  their extracted PCM samples, with the stored loop and an approximate envelope.
  Their preview rate is independent of note key and pitch controls. The assumed
  mixer rate is adjustable; its 15,768 Hz default was observed in one European
  checkpoint and is not a claim about every game state. A bad rate blocks only
  a selected fixed-PCM audition, not unrelated tracks or sine mode. Synthetic
  `rom-data-viewer-fixed-pcm-synthetic-guard-npb27tgy` checked sample data,
  loop, envelope, rate changes, fallback, CGB mute and the scoped rate guard.
  Protected EUR unit `rom-data-viewer-fixed-pcm-eur-guard-treqkh4d` compared
  the one fixed tone and all 13,468 sample values, then checked a real sequence
  note and live Web Audio in Linux Chrome. The local page retained its prior
  data and inspector. Actual mixer-rate changes, original mixer timing, effects
  and physical audio remain unverified. [Driver observations](../rom_data_audio.md)
- [x] **Instrument-bank browser and audition.** Browse stored banks, programs
  and key-map links independently of a song, and audition resolved PCM,
  fixed-rate PCM, CGB square and CGB wave tones. Missing or unsupported voices
  remain unavailable rather than being assigned a guessed waveform. The
  chosen key, fixed rate and CGB mute persist while changing programs; the mute
  is shared with the sequence player and can be changed during playback.
  Linked source waveforms open in the existing waveform view. The position bar
  follows the chosen duration or the end of an unlooped PCM sample, whichever
  comes first. Pitch, mixer, envelope and timing remain approximations.
  Synthetic Chrome units `rom-data-viewer-instruments-synthetic-xc9e3zo9` and
  `rom-data-viewer-instruments-oneshot-94ohotzu` checked mapping, playback,
  mute, focus, unsupported tones and one-shot progress. Protected local-page
  unit `rom-data-viewer-instruments-oneshot-local-q7k0c8uu` checked four stored
  instrument kinds, key maps, waveform links and real Web Audio in Chrome;
  Docker unit `rom-data-viewer-instruments-docker-w3mo8nxs` matched Python on
  synthetic input. These units used isolated temporary roots while `target`
  was unmounted; embedded data stayed unchanged and physical audio remains
  unverified.
- [x] **Export resolved instrument waveforms.** Save the currently resolved
  PCM, fixed-rate PCM, CGB square or CGB wave as a mono 8-bit, 32 kHz WAV at
  the selected key and preview duration. Stored loops repeat; an unlooped PCM
  sample ends at its recorded length. The export is normalized and omits the
  driver's effects and CGB mute setting. Synthetic Chrome unit
  `rom-data-viewer-instrument-wav-synthetic-uno61ye_` checked resampling,
  loops, short samples, controls and existing audio navigation. Independent
  Python WAV decoding passed in `rom-data-viewer-instrument-wav-decoder-10eyw6q9`.
  Protected local-page unit `rom-data-viewer-instrument-wav-local-3g_6k5r0`
  checked actual examples of all four types and unchanged embedded data;
  Docker unit `rom-data-viewer-instrument-wav-docker-h7atiora` matched Python
  on synthetic input. Pitch and interpolation are approximations, and the
  original dataset volume was unavailable for this local-page check.
- [x] **Song and instrument-bank links.** Each bank lists the song IDs whose
  stored headers reference it, in pages of 25. Follow a song to its sequence or
  open the current song's bank without starting playback; key and CGB mute
  settings remain selected. A bank reference does not establish which programs
  or tones a song uses. Synthetic Chrome unit
  `rom-data-viewer-song-banks-synthetic-vb32mf9z` checked pagination, duplicate
  IDs, missing references, settings and existing audio/bookmark controls. An
  earlier run `zvakd4im` failed because its test reused a Chrome profile;
  the corrected test passed. Protected local-page unit
  `rom-data-viewer-song-banks-local-ny618l30` checked every listed reference
  across four banks (45, 228, 85 and 96 songs), both directions and unchanged
  embedded data. Docker unit `rom-data-viewer-song-banks-docker-kry3o7x6`
  matched Python on synthetic input. The local-page check used its already
  embedded data because the original dataset volume was unavailable.
- [x] **Searchable song directory.** Browse the stored song IDs and headers,
  extracted track counts and bank addresses in pages of 25, filter by song or
  header ID and select an entry without starting playback. Duplicate IDs remain
  separate, and missing headers are identified as unavailable. The existing
  selector, CGB mute and bank navigation remain in use. Synthetic Chrome unit
  `rom-data-viewer-song-catalog-synthetic-gcgft5ld` checked paging, filtering,
  duplicate IDs, missing data and existing audio controls. Protected local-page
  unit `rom-data-viewer-song-catalog-local-nleo7_qy` compared all 640 entries
  over 26 pages with the data already embedded in the local wiki, preserving
  that data; Docker unit `rom-data-viewer-song-catalog-docker-holhf0mc` matched
  Python on synthetic input. Extracted track counts do not claim a complete
  original song; the source dataset volume was unavailable during this check.
- [x] **PCM note lengths, pitch and envelope preview.** Add the stored note
  extension to the clock length using the original byte wrapping, and use the
  extracted PCM pitch tables for the unbent key when available. Older datasets
  without those tables keep the previous pitch approximation. A checkbox enables
  an approximate PCM attack/decay/sustain/release envelope from the extracted
  tone word, with a bounded release and mix scaling; CGB and sine fallbacks keep
  their existing envelope. This does not reproduce exact tempo, pitch modulation,
  effects, mixer behavior or live CGB envelopes. Synthetic unit
  `rom-data-viewer-audio-fidelity-synthetic-kleo1rm5` checked extension wrapping,
  600 integer pitch calculations, envelope scheduling and prior controls.
  Protected EUR unit `rom-data-viewer-audio-fidelity-eur-cv4woo4e` checked all
  180 PCM keys, 12 bases and 429 tone words, observed 1,006 stored extensions,
  and verified an extended PCM note's pitch and envelope with live Web Audio in
  Linux Chrome. Physical audio output remains unverified.
- [x] **Estimated song tempo.** A checkbox follows decoded tempo commands from
  all song tracks, including when auditioning just one track; disabling it uses
  the manual tick rate throughout. Before the first tempo command or when none
  is found, the same manual rate is the fallback. Tempo changes affect note
  starts and ends, PCM envelope timing and the seconds-based note timeline;
  a zero tempo stops sequence advancement and leaves active note gates open
  until the preview ends. The tempo pass is
  bounded and reports incomplete track scans. The rate follows the continuous
  average implied by a [generic M4A command and accumulator reference](https://github.com/pret/pokeemerald/blob/master/src/m4a_1.s#L901-L910);
  per-frame quantization, live tempo controls and exact FFTA-driver behavior are
  not verified. Synthetic Chrome units `rom-data-viewer-sequence-tempo-synthetic-0bksevth`
  and `rom-data-viewer-sequence-tempo-timeline-lfnlvmos` checked cross-track
  changes, same-tick ordering, zero tempo, fallback and playhead alignment.
  Protected EUR unit `rom-data-viewer-sequence-tempo-eur-5k5nafnt` counted 1,150
  source commands and checked a real cross-track note in automatic and manual
  modes with live Web Audio in Chrome. Physical audio remains unverified.
- [x] **Track volume, stereo and pitch controls.** A checkbox applies decoded
  volume, pan, key shift, pitch bend, bend range and tuning commands, including
  changes while notes are active. A separate collapsible, paginated table lists
  the decoded controls. Browser gain, stereo panning and equal-tempered pitch
  ratios are approximations of the [generic M4A track controls](https://github.com/pret/pokeemerald/blob/master/src/m4a_1.s#L913-L991);
  before the first volume command the preview uses full gain. Pitch automation
  has a 100,000-event playback bound and reports when it is reached; unavailable
  browser panning or live pitch changes are reported. The CGB mute remains
  independent. Synthetic Chrome unit `rom-data-viewer-track-controls-synthetic-2pni2qje`
  checked live changes, per-track routing, manual bypass, tempo and CGB mute.
  Protected EUR unit `rom-data-viewer-track-controls-eur-co_43x87` counted 8,419
  source controls and checked real volume and pan changes during notes plus a
  PCM note with pitch bend using live Chrome Web Audio. Exact mixer, modulation and
  physical audio remain unverified.
- [x] **Individual track mute.** A collapsible mixer can mute or unmute each
  extracted track while the sequence is playing, independently of the CGB mute
  and the decoded volume/pan controls. Mute settings follow track numbers when
  changing songs within the page session; those numbers are local to each song.
  The note and instruction views still show muted tracks, and tempo scanning is
  unchanged. Synthetic Chrome unit `rom-data-viewer-track-mute-synthetic-1e1n1szc`
  passed 21 tests including live PCM/CGB routing, switching songs, mute-all,
  envelope, pan, pitch, track selection and the earlier viewer controls.
  Protected local-page unit `rom-data-viewer-track-mute-local-9ci_ok9o` checked
  two live tracks and the independent CGB mute with Web Audio; embedded data was
  unchanged. Docker unit `rom-data-viewer-track-mute-docker-m5o99h11` matched
  Python on synthetic input. Physical audio and the original mixer remain
  unverified; the source dataset volume was unavailable during the local check.
- [x] **CGB wave instruments in sequence audition.** Resolve supported CGB
  wave tones through the same program and key maps, and play their extracted
  32-step waveforms in a loop. Base pitch uses the driver tables when
  available; unsupported tones use sine fallbacks.
  Synthetic unit `rom-data-viewer-cgbvoice-e6iutnqa` checked mapping, waveform
  cache separation, pitch, loops and fallbacks. Protected EUR unit
  `rom-data-viewer-cgbvoice-eur-9_1pifpq` compared both source waves and checked
  a mapped sequence note, Web Audio start and stop in Linux Chrome. Physical
  audio and original-driver timing, tuning, envelopes and effects remain
  unverified.
- [x] **CGB square instruments in sequence audition.** Interpret the low two
  bits of the type 1/2 tone's word at offset 4 as the duty selection used by
  [generic M4A voice definitions](https://github.com/pret/pokefirered/blob/master/asm/macros/music_voice.inc).
  Loop the corresponding eight-step pattern from the project's
  [APU implementation](../../crates/gba-core/src/apu.rs), preserving key-mapped
  tones, pitch automation, the CGB mute and sine mode. This is an approximate
  browser audition: pitch automation, resampling, envelopes, channel
  allocation and game-specific driver behavior are not verified as exact.
  Synthetic `rom-data-viewer-cgb-square-synthetic-corrected-ii339smu` checked
  all four duty patterns, cache sharing, live pitch, mute and fallback. Protected
  EUR `rom-data-viewer-cgb-square-eur-corrected-nefqr2ow` compared four square
  tone records and checked a long real sequence note and live mute/unmute in
  Chrome Web Audio. The initial synthetic unit `nctgtt4c` lacked the required
  PCM fixture; diagnostic `pnk_eqbw` also intercepted browser startup. The
  initial EUR unit `7a46jqcr` read the gain synchronously; corrected checks
  waited for audio rendering and passed. Physical audio remains unverified.
- [x] **CGB driver-table pitch.** For square and 32-sample wave tones, use the
  extracted CGB key and signed base tables to calculate the initial register
  for the selected note. Convert its 11-bit hardware period using the project's
  APU clock model; the standalone instrument view labels the register and
  frequency, and sequence notes show the base register. Instrument WAV exports
  use the same pitch. Older datasets without both tables retain the previous
  equal-tempered approximation. Track pitch automation remains an approximation
  relative to that base; driver channel state, effects and live playback are not
  reproduced exactly. Synthetic unit `rom-data-viewer-cgb-pitch-synthetic-i2lvr97q`
  passed 43 audio, controls and generator tests. Protected EUR unit
  `rom-data-viewer-cgb-pitch-real-p3iqfzif` authenticated seven source ranges
  and the complete driver JSON, checked 1,024 register calculations against an
  independent implementation, and exercised three CGB types, live mute and a
  real sequence in Linux Chrome. Fourteen Docker/Python output files matched.
  The ignored local index and audio page were published with unrelated data
  preserved. Physical output and Windows remain unverified; the large-JSON
  worker remains paused. [Driver tables](../rom_data_audio.md#original-pitch-rate-noise-and-volume-lookup-tables)
- [x] **Mute CGB voices in sequences.** The sequence player can mute and
  unmute identified CGB tones during playback, including key-mapped tones and
  sine fallbacks. Unidentified voices remain audible. The checkbox stays set
  when changing track or song and is restored after reload when browser storage
  is available. Synthetic browser unit
  `rom-data-viewer-cgb-mute-corrected-xm0rn7ys` checked the CGB and non-CGB
  routes, live toggles, fallbacks, selection changes and reload. Protected EUR
  unit `rom-data-viewer-cgb-mute-eur-audible-898xw9ja` checked a mapped CGB
  note and the live Web Audio gain after rendering began, plus track and song
  changes in Linux Chrome. Physical audio and the original game's CGB output
  remain unverified.
- [ ] **Complete song playback.** Reproduce original instruments, driver timing,
  mixing and effects before describing a synthesized audition as playback of
  the original song.
- [x] **Search extracted file contents.** Search local files for text or hex
  bytes with bounded reads, highlighting and cancellation. The real-time
  checks above supersede the removed initial prototype whose browser test did
  not complete in `rom-data-viewer-search-t2dkxkfs`.
- [ ] **Further previews.** Identify other extracted graphical, text and audio
  resources that can be rendered or played from their documented formats.
