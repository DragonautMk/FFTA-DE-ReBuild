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

## Section change badges

When adding or modifying a viewer section, update
`scripts/viewer_section_updates.json` for every affected scope and section ID.
Record `added` or `updated` and the actual UTC change time in ISO 8601 form;
keep unchanged entries intact rather than stamping the next generation time.
The badge shows the date and time in the browser's local time zone, while its
`datetime` and title retain UTC. Topic sections, their groups and navbar links
show the latest relevant change; the single-page view uses the newest matching
section timestamp. The index supports its own scope. Only visible sections are
badged, and generated data remains private.

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

The current local European wiki uses the verified e376-d375 dataset and includes
its separately scoped screen-effect motion explorer. The dataset and generated
pages remain private; see the e376 entry below for validation and limits.

The earlier European wiki refresh used extractor 334 / decoder 333. Protected
unit `n4iol0rn` built the source-authenticated core audio exporter, retained the
155 unchanged resource files and reconstructed the changed audio directory;
every file matches the accepted e334 manifest from `zw1q89b2`. Protected unit
`mssfn_fg` checked the generated index, world, script-actor, audio and new
file pages in Linux Chrome; preserved ten topic payloads and 144 prior resource
pages; and matched 15 Python/Docker files with the new audio directory. The
ignored local wiki and `out/rom-data-europe-e334-d333` were published; e333 is
retained as the named previous dataset. The full Linux application build
`1_bunyg0` could not start its ALSA-dependent component because the development
package is absent; the lightweight core export does not claim that build passed.
The timed-sound view below exposes the added sources and candidate auditions,
including an approximate timed mix; full scheduler playback remains open.
Windows is deferred.

The earlier European wiki refresh used extractor 335 / decoder 334. Protected
unit `mkd3en8j` retained the 155 unchanged resource files and used the
source-authenticated core exporter for the changed copied-lookup catalog;
every file matches the accepted e335 manifest from `sdjtlh9f`. Protected unit
`u_3wn8yk` checked the index, world, script-actor, audio, other-resources and
new file pages in Chrome; preserved ten topic payloads and 144 prior resource
pages; and matched 15 Python/Docker files. The ignored wiki and
`out/rom-data-europe-e335-d334` were published; e334 is retained for the named
comparison. This core export does not claim a full Linux application build.
Windows remains deferred.

The earlier European wiki refresh used extractor 336 / decoder 335. Protected
unit `yxq3a7bk` retained the 155 unchanged resource files and used the
authenticated core exporter for the changed copied-lookup catalog. Every file
matches the accepted e336 manifest from `hoduqk8r`, including 16 initializer
variant records. Protected unit `2yzppv5w` checked six wiki pages in Chrome,
preserved ten topic payloads including the direct metadata names and 144 prior
resource pages, and matched 15 Python/Docker files. The ignored wiki and
`out/rom-data-europe-e336-d335` were published; e335 remains available for the
named comparison. The core export is not a full Linux application build.
Windows remains deferred.

The earlier European wiki refresh used extractor 337 / decoder 336. Protected
unit `y_rfw70r` retained the 155 unchanged resource files and used the
authenticated core exporter for the changed copied-lookup catalog. Every file
matches the accepted e337 manifest from `xtp6kbf3`, including ten initializer
pairs. Protected unit `8xh8zwqj` checked six wiki pages in Chrome, preserved
ten topic payloads including the transfer LZ previews and 144 prior resource
pages, and matched 15 Python/Docker files. The ignored wiki and
`out/rom-data-europe-e337-d336` were published; e336 remains available for the
named comparison. The core export is not a full Linux application build.
Windows remains deferred.

The earlier European wiki refresh used extractor 338 / decoder 337. Protected
unit `n0musgix` retained 155 unchanged resource files and reconstructed the
updated raw-transfer catalog from the authenticated signed source table and
literal. Every file and the manifest match the candidate accepted in
`e555-signed-map-deltas-wine-hszbu_3b`. Protected unit `_yqtclhf` compared all
48 displayed signed values with the ROM, checked four wiki pages and the shared
transfer asset in Chrome, preserved 11 prior topic payloads apart from the
documented changed resource and its view, and matched 16 Docker/Python files.
Fourteen ignored wiki files and `out/rom-data-europe-e338-d337` were published;
e337 is retained for comparison. The construction uses the accepted core output
identity and is not a full Linux application build. Windows remains deferred.

The previous local European wiki used extractor 339 / decoder 338. Protected
unit `buwzwh4e` retained 154 unchanged resources and reconstructed the two
changed code-reference catalogs from nine authenticated source spans. Every
file and the manifest match the candidate accepted in
`e558-world-node-dispatch-wine-ua5x_s_7`. Protected unit `rom-data-viewer-refresh-e339-view-fpd_62za`
checked the 48 signed offsets and their preview in Chrome, preserved 11 topic
payloads apart from the two changed resource rows and 143 prior resource
pages, and matched 16 Docker/Python fixture files. Fifteen ignored wiki
files and `out/rom-data-europe-e339-d338` were published; e338 remains available
for comparison. This construction is not a full Linux application build.
Windows remains deferred.

The earlier local European wiki used extractor 340 / decoder 339. Existing
unit `yid7rliw` verified the retained European dataset and manifest. Unit
`574prtnv` guarded and checked the exact candidate before publication, including
11 unchanged topic payloads, three focused text scopes and one changed resource
page. Unit `4aktz7x0` exercised browser navigation, shared reading preferences
and combined search, and matched all 20 Python/Docker fixture files. Seventeen
ignored wiki files were published with a recoverable backup under
`out/ffta-viewer-before-e340-ivgdfkkd`; `out/rom-data-europe-e340-d339` points
to the verified dataset. This is not a native Windows or full Linux runtime
validation.

The previous local European wiki used extractor 343 / decoder 342. The retained
generation `out/rom-data-europe-e343-d342` was already created, verified and
reused by E573. Protected unit `viewer-e343-refresh-vhmqjk0v` authenticated all
156 resource files, regenerated all 16 wiki pages, and confirmed that only the
copied-halfword-lookup resource changed from e340. The updated file's structured
inspector exposes the 54 stored direct-copy argument entries. Existing topic
payloads and 153 older resource pages were preserved; Chrome navigation and 22
synthetic Python/Docker outputs passed. The ignored pages and new resource page
were published with preimages under `out/ffta-e343-before-y_6ydq0c`.

The previous local European wiki used extractor 344 / decoder 343. E576 created,
verified and reused the retained `out/rom-data-europe-e344-d343` dataset.
Protected unit `viewer-e344-refresh-qx239myo` authenticated all 156 resource
files and the 84 newly added producer literals, checked the updated structured
inspector in Chrome, preserved existing topic data across 16 pages and 154 earlier resource
pages, and matched 22 synthetic Python/Docker outputs. The ignored pages and
new resource page were published with preimages under
`out/ffta-e344-before-zqhroz9t`. The previous e343 dataset remains available
for this comparison; this refresh does not establish the remaining live pointer
domains or generated-copy paths.

The previous local European wiki used extractor 345 / decoder 344. E579 created,
verified and reused the retained `out/rom-data-europe-e345-d344` dataset. Protected
unit `viewer-e345-refresh-8hmrnt9m` authenticated all 156 resource files and the
27 added shared-slot literal words against the source ROM, checked the structured
inspector in Chrome, preserved all existing topic data across 16 pages and 155
earlier resource pages, and matched 22 synthetic Python/Docker outputs. The
ignored pages and new resource page were published with preimages under
`out/ffta-e345-before-ozc1yen9`. The previous e344 dataset remains available
for comparison. Live pointer domains and generated-copy paths remain unverified.


The previous local European wiki used extractor 346 / decoder 345. Protected
unit `viewer-e346-refresh-2yew90aq` authenticated all 156 resource files,
checked the two added switch tables and their 18 entries and twelve helper-field
literals against the source ROM, and opened the updated structured inspectors
in Chrome. It preserved all 18 topic payloads and 156 existing resource pages,
including the item sprite links, and matched 24 synthetic Python/Docker files.
The ignored wiki and its two new resource pages were published with preimages
under `out/ffta-e346-before-c6jp3v_5`; the previous e345 dataset remains
available for comparison. Live pointer behavior and native Windows are not
established by this refresh.


The previous local European wiki used extractor 352 / decoder 351, from the
verified retained e352 generation. The four updated engine resources remain
available through their file inspectors. Protected unit `obqvu6ow` authenticated
3,912 source ROM ranges, preserved all 22 topic payloads and 156 resource
entries, exercised Chrome and matched 173 full Python/Docker output files. It
retained the lazy name index and all 159 previous resource pages. The 23 ignored
wiki pages, four new resource pages and ownership marker were published with
preimages `out/ffta-e352-before-q3wjyxj6`. Live engine semantics and native
Windows remain unverified.

The previous local European wiki used the verified retained extractor 358 / decoder
357 (e358) generation. It contains 159 resources and preserves the 158 e357
resources, scoped data and lazy assets. Protected `viewer-e358-real-6yl44929`
compared all ten new source ranges and all earlier resource files with the
original source and e357, exercised all five maps at both tile depths in
Chrome, and matched 575 Python/Docker outputs. The 23 wiki pages, one new
resource page and ownership marker were published with preimages
`out/ffta-e358-before-1nm3r21b`; 170 earlier resource pages remain available.
The separate e358 runtime test `g1iu0n_6` initially failed because its fixture
used the wrong return address. The corrected Wine test `01bkwwt1` and retained
stage `ol67qzpt` passed using the same dataset and manifest. Native Windows and
ordinary scene selection remain deferred.

The previous local European wiki used the verified retained extractor 357 / decoder 356
(e357) generation. It contains 158 resources and preserves the prior 157 resource
records, scoped data and lazy assets. The newly extracted localized OBJ bank is
available for frame-by-frame preview and looping in Object animations. Protected
`viewer-e357-projection-3q5fi0k7` compared all 2,308 new source ranges and every
reconstructed index, sequence and object list with the original source. Protected
`viewer-e357-real-gkaul4s0` exercised six real banks in Chrome and matched all
574 Python/Docker generated files. The 23 wiki pages, one new resource page and
ownership marker were published with preimages `out/ffta-e357-before-3mxjmdl7`;
169 earlier resource pages remain available. Native Windows remains deferred.

The previous local European wiki used the verified retained extractor 356 / decoder 355
(e356) generation. It includes 157 resources: e355 updates the Thumb literal
resource and e356 adds the background-effect graphics bank. Protected unit
`viewer-bgfx-real-ei45gd6n` authenticated 3,377 original ROM ranges, preserved
21 other scoped payloads, 167 earlier resource pages and all existing lazy
assets, and matched 573 full Python/Docker output files. It reconstructed all
25 tile blocks and 15,000 map cells and compared 460,800 RGBA bytes across
three browser previews against an independent byte-based renderer. The 23
ignored wiki pages, two new resource pages and ownership marker were published
with preimages `out/ffta-bgfx-before-o76ky4_u`. A newer extractor revision may
be under development; live source selection, exact game composition and native
Windows remain unverified.

The previous local European wiki used extractor 354 / decoder 353, from the
verified retained e354 generation. Two transfer-graphics resources and two
engine resources changed while all 156 resource entries remain available. The
new 246-byte LZ77 input decodes to 448 bytes and is selectable in the transfer
graphics preview and the on-demand tilemap graphics catalog. Protected unit
`viewer-refresh-e354-real-a7kr8m_o` authenticated 6,453 ROM source ranges,
including all 14 newly added ranges (386 bytes); independently decoded the new
stream and checked its pixels in Chrome; preserved 21 other topic payloads,
the lazy name/text assets and 163 previous resource pages; and matched all 177
full-generation Python/Docker files. The 23 ignored wiki pages, four new
resource pages, transfer asset and ownership marker were published with
preimages `out/ffta-e354-before-jb_rrx0h`. Live source selection, exact game
composition and native Windows remain unverified.

The previous local European wiki used extractor 347 / decoder 346. Protected
unit `viewer-e347-refresh-yod_z36s` authenticated all 156 resource files and
46 context-record literals against the source ROM. It preserved all 21 topic
payloads and 158 previous resource pages, exercised the new structured inspector
and focused pages in Chrome, and matched 27 synthetic Python/Docker files. The
ignored wiki and its updated resource page were published with preimages under
`out/ffta-e347-before-pc9mcctj`; the previous e346 dataset remains available for
comparison. Initial unit `t3y3afqr` used the wrong JSON enum shape in its
check; the corrected unit compared all 46 literal values. Live pointer behavior
and native Windows are not established by this refresh.

Open the resulting HTML index locally. Its topic links open separate offline
pages for gameplay records, world maps and terrain, backgrounds and palettes,
interface controls and tiles, tilemaps and transfer graphics, object animations,
actors/equipment, battle actors, script actors, songs and timed sounds,
waveforms and instruments, CGB noise, focused short texts and fonts, combined
text search, event scripts, indexed graphics and tile attributes, transfer graphics and other resources. The overview embeds
only index fields, and each topic page embeds its own projected data and the
small shared dependencies required by its previews. PCM samples and battle or
script sprite tiles load from local assets when needed. The tilemap and
compact-cell views, and the object-animation view on the object page, can load
transfer graphics from the same local companion asset on request. Within a
single-page output, one button shares it across all three views. A saved link to
one of these graphics loads the asset automatically.
If the asset is missing or does not match, restore it and retry. Full
manifest metadata is retained on its topic page. Folder selection and other page-session controls
are local to the current page. The contents sidebar links to the views available for that dataset;
each section can be opened or closed independently, and the sidebar can open or
close all sections. Resource rows link directly to their dedicated views when
available. On narrow screens the sidebar and resource rows stack vertically.

**Navbar and compact overview.** The persistent navbar groups topic pages and
their available views into alphabetically ordered menus. Opening Audio shows
links to each available player and catalog immediately; links open the containing
section.
The overview groups its
resources by topic page and then by resource family; both levels start closed,
while search opens matching groups. On topic pages, the contents sidebar and
the available views are grouped into ordered, collapsible sections. Saved links
open their containing group as well as the requested view. The overview uses a
separate template and does not load topic-view code.


**On-demand manifest sources.** Split topic pages keep resource descriptions,
file hashes and dependencies available immediately. Source inventories larger
than 64 KiB are stored once in content-addressed local assets and loaded when
**Manifest details** opens. Pagination, filtering and safe text rendering remain
available; a missing or invalid asset reports an error and can be retried. Pages
share the same asset for identical inventories. Smaller lists and single-page
output remain inline; generated assets stay in the ignored companion directory.

Synthetic `viewer-lazy-manifest-synthetic-nqzowmmn` passed 15 tests including
shared assets, deferred loading, paging, filtering, unchanged output guards and
retry after missing or incomplete files. Protected e359
`viewer-lazy-manifest-real-irnnffap` compared all 19 deferred inventories with
the manifest across 22 scoped payloads, exercised real text and audio source
lists in Chrome, preserved 172 resource pages and matched 602 Python/Docker
files. The pointer-text page fell from 7,789,195 to 2,702,771 bytes and the
relative-text page from 11,950,373 to 6,863,949 bytes. The 23 local pages and
19 assets were published with preimages `out/ffta-lazy-manifest-before-wedid2xj`.
Native Windows remains unverified.

**Section change badges.** A maintained UTC registry identifies added and
updated sections by scope and ID. Their disclosure headings, sidebar and navbar
show a date/time badge, with parent dropdowns showing their latest child change.
Unchanged sections retain their recorded time across regenerations; the browser
shows local time and keeps the exact UTC timestamp on the HTML `time` element.
Synthetic `viewer-e360-compact-synthetic-md0max7y` checked topic and index
badges, stable regeneration, single-page output and a narrow mobile viewport.
Protected e360 `viewer-e360-compact-real-nxxwgbvm` checked the new object page
and previously added conditional table badge in Chrome, and matched all 605
Python/Docker outputs. Native Windows remains unverified.

**Search recognized names.** The navbar opens a local search for source-linked
item, action, job metadata and region references, with language and category
filters, pagination and links to the selected record. The compact generated
name index loads only when the panel opens; it stays in the ignored companion
directory with the extracted data. Unavailable or unauthenticated references
are excluded, unknown glyphs are marked, and region placement and live metadata
roles are not inferred. Synthetic `ue3v2v_r` passed 26 tests. Protected e347
`1tov3gy3` authenticated four source files and three ROM ranges, indexed 866
records, checked four cross-page routes and lazy loading in Chrome, preserved
22 scope payloads and 156 resource entries, and matched 28 Python/Docker files
including the generated index. Chrome unit `8m_fx21h` compared all 4,330
indexed language strings to the existing viewer labels with zero mismatches.
The private on-demand asset is 191,135 bytes. Published 23 ignored pages and the asset with preimages
`out/ffta-name-search-before-35xegfz8`. Windows remains deferred.

**Observed map nodes in name search.** The lazy name index also includes
six authenticated observed world nodes, with their localized region names and
a direct link selecting the node and language on the map. The search labels
these as observed nodes: the recorded placement is conditional on game state.
Missing or unverified names and absent nodes do not produce search entries.
Updated search and map sections show dated badges. Synthetic
`viewer-node-search-synthetic-ftj7i1uw` passed 58 projection, Chrome, focus,
name and regression tests; two existing projection tests now read the current
pointer-text scope. Protected `viewer-node-search-real-emfdxes0` checked all 30
localized names and six recorded placements against authenticated sources,
exercised Chrome search/map navigation, preserved 26 scoped payloads apart from
name-index and badge metadata and 175 resource pages, and matched 610 Python/
Docker files. Published 28 local files with preimages
`out/ffta-node-search-before-vame64sa`. Native Windows remains deferred.

**Focused world and interface pages.** World maps, terrain, paths and observed
world names have their own page. The gameplay page keeps the related item,
metadata, encounter and mission views together. Terrain carries the extracted
encounter headers and coordinates needed for overlays without the full unit
catalog; links between terrain and encounters retain the chosen descriptor and
encounter. Backgrounds, interface controls and tilemap/transfer previews are
separate pages, with only the shared graphics required by each preview. Existing
world and interface bookmarks redirect to moved views and retain their selected
values. Resource pages link to their new scope.

Regression unit `viewer-nav-focused-regression-etx2hr3v` passed 32 checks.
The additional 61-check unit `viewer-nav-focused-extra-qmjj6bfz` passed 59;
two assertions still looked for the former interface menu and passed after the
expected scope was corrected in `viewer-nav-focused-extra-fixes-2e4pbtxq`.
Protected e346 unit `viewer-nav-focused-real-t43s7_z8` preserved the topic data
and all 156 resource entries, checked eight pages in Chrome and matched 27
Python/Docker fixture files. The 21 ignored pages and 29 resource links were
published with preimages under `out/ffta-nav-focused-before-jadbhcjz`. The old
world page was 3,602,351 bytes; gameplay and world-map pages are 3,272,231 and
1,319,998 bytes. The old interface page was 9,253,051 bytes; backgrounds,
interface controls and tilemaps are 5,908,932, 1,498,144 and 6,603,470 bytes.
The first failed click check `viewer-nav-focused-synthetic-dq3eeg3g` clicked
between two wrapped lines; `viewer-nav-click-hit-bl56evfb` confirmed that the
link itself received clicks on either line. Windows remains deferred.

**Focused short texts and fonts.** A separate page loads only the short-text,
font and name-keyboard projections. The resource index, navbar and the three
corresponding resource pages open it directly. Its local search links to the
existing combined search page; language pinning, hidden commands and font
preferences remain shared. The all-text page and its existing bookmarks remain
available. Synthetic regression `6_2sozr2` passed 29 checks; focused unit
`qmu47x3h` passed four tests including index navigation and short-text search.
Protected e347 unit `oh6bolmr` authenticated 677 source ranges, preserved the
previous scope payloads and all 159 existing resource pages, exercised the new
page and old bookmarks in Chrome, and matched 27 Python/Docker files. The
focused page was 1,767,222 bytes; the combined page was 21,790,688 bytes at that checkpoint. The
first unit `fudl9y7t` stopped on a test-only assumption that 156 resource
entries meant 156 retained resource pages; the corrected unit reused its
candidate and preserved all 159. Twenty-six ignored files were published with
verified compressed preimages in `out/ffta-focused-text-before-txn15ise`.
Four completed, byte-identical viewer candidates were retired to regain space;
receipts and their verified replacements remain at
`out/ffta-viewer-duplicates-cleanup-21qvta_0`. Windows remains deferred.

**On-demand combined text search.** The search page contains a compact
archive index and loads only the selected text archives when a search starts.
An all-archive search loads all four; a cancelled search cannot later replace
its status or results. Missing companion files report an error and can be
retried. Results open the exact entry on its focused text page, retaining shared
language and display preferences. Older bookmarks to individual text, font or
registry views redirect to their focused pages with their selected values. The
search index links to those views and keeps full manifest ranges available on
focused resource pages. Local generated data remains in the ignored companion
directory.

Synthetic unit `viewer-lazy-text-synthetic-ezweg958` passed 18 browser and
generator tests, including lazy loading, cancellation, retries, all four result
routes, bookmarks and preferences. Protected e352 unit `viewer-lazy-text-real-cc544fov`
authenticated four extracted source files, compared five real searches with the
previous page, preserved 21 other scope payloads and 156 resources, and matched
35 Python/Docker files. The search page fell from 21,840,761 to 773,537 bytes;
its four separate assets are loaded only as needed. The first real unit
`5o9u6nxj` revealed that full manifest ranges still made the page 5,977,429
bytes; the corrected version uses the existing compact index rows. Published 23
ignored pages and four new assets with preimages
`out/ffta-lazy-text-before-jd_m6y94`. Font mapping, substitutions, additional
previews and native Windows remain open.

**Physical audio banks and e359.** The previous local wiki used the verified retained
extractor 359 / decoder 358 generation, with 160 resources. Its optional
physical-audio source adds 70 previously unlisted tone records, eight direct PCM
waves and six CGB waves. The four instrument banks expose all 499 physical
program slots, retaining prior tone IDs, wave IDs, program positions and known
key mappings. Added wave and program entries identify their source and use the
existing playback and lazy PCM loading controls; instrument and song previews
respect the shared CGB mute. New split or drum
program records do not invent missing key mappings. All earlier resource pages
and linked scope data remain available.

Protected `viewer-physical-audio-source-ce5fzo52` compared the tone type and words of all 499 slots,
285 PCM waves and eight CGB waves with the authenticated source. Synthetic
`viewer-physical-audio-synthetic-39g5oams` passed 14 generator and Chrome tests,
including the navbar, separated world/interface pages, playback, mute, lazy
loading and legacy links. Protected e359 `viewer-physical-audio-real-8b9zdtru`
checked all 47 new source ranges, exercised the eight new PCM and six new CGB
waves in Chrome, preserved 171 prior resource pages and matched 583 Python and
Docker files. The 23 private pages and eight new files were published with
preimages `out/ffta-e359-audio-before-qpoq280w`. Earlier checks failed on an
incorrect test module name, an unused historical asset and an overly strict
resource-marker comparison; corrected checks passed. Original instrument note
mappings, music timing and native Windows remain unverified.

**Audio topic pages.** Waveforms and instrument banks have their own topic page,
while the CGB noise generator loads without waveforms or song sequences. Songs
and timed schedules retain sequence and waveform metadata; their PCM samples
load when auditioned. Instrument-bank song links open the matching song on its topic
page, and old audio-page noise links redirect to the new page. The shared CGB
mute is retained across the pages. Synthetic unit `viewer-audio-scope-synthetic-0qmc05ef`
passed 52 checks for playback, navigation, bookmarks and controls; an earlier
unit `viewer-audio-scope-synthetic-9x54lx5k` exposed a shadowed page counter,
which was corrected. Protected e345 unit `viewer-audio-scope-real-zor6lsg1`
preserved existing topic data, exercised real noise and bank-to-song navigation
in Chrome, and matched 24 Python/Docker files. The former 17,315,942-byte audio
page remains the song player; the samples page is 9,510,147 bytes and the noise
page is 690,337 bytes. Eighteen ignored wiki pages were published with backup
`out/ffta-audio-scope-before-aiww1wpv`; Windows remains deferred.

**On-demand PCM samples.** Split audio pages reference individually
content-addressed local PCM wave assets instead of embedding all sample arrays.
Opening the PCM or instrument view loads its selected wave; song, timed-sound
and WAV previews load the distinct PCM waves needed for the chosen tracks.
Concurrent consumers share each load; cancellation prevents delayed playback,
missing or invalid files report an error and can be retried, and inline
single-page output retains its existing behavior. Synthetic unit
`viewer-lazy-pcm-synthetic-qpn83sv1` passed 18 tests including offline loading,
track selection, cancellation, retry, timed playback, real WAV output and audio
regressions. The earlier synthetic run caught and fixed a timing regression in
instrument WAV export. Focused unit `viewer-lazy-pcm-extra-7evua0_i` also checked
instrument play and WAV, selection focus and retry after a missing song wave; an
initial focus check had left its outer category closed. Protected e354 unit
`viewer-lazy-pcm-real-k1e_d1w0` compared all 277 wave assets to authenticated
source data, checked a real selected track in Chrome, preserved the other
scoped payloads and existing lazy assets, and matched 305 Python/Docker outputs.
The song page fell from 17,407,826 to 9,081,309 bytes and the sample page from
9,601,540 to 1,275,023 bytes. Published 23 ignored pages and 277 assets with
verified preimages `out/ffta-lazy-pcm-before-an9r53tb`. Selected song previews
still load every PCM wave used by their selected tracks; native Windows remains
unverified.

**On-demand song instructions.** Split audio pages retain track metadata and
load the reachable instruction graph for a selected song from a local,
content-addressed asset. Shared loads are reused, late selections cannot replace
current results, and a missing or invalid asset can be retried. Timed auditions
load their required songs and can be cancelled during loading; CGB and track
mutes, focus, WAV export and inline single-page output remain available. The
modified song and timed-sound sections carry dated badges. Synthetic
`viewer-lazy-songs-synthetic-kuvls2ga` passed 32 audio, retry, cancellation and
regression tests. Protected e364 `viewer-lazy-songs-real-8ue83huz` independently
compared all 454 header graphs and 105,253 instructions, matched notes, traces
and controls for 17 real songs in Chrome, exercised playback and focus,
preserved 29 scoped payloads apart from the instruction representation and
badges, retained 177 resource pages, and matched 1,069
Python/Docker files. The initial real test `ulr3you8` used a new retry selector
on the old baseline page; the corrected comparison passed. The song page fell
from 8,829,683 to 1,456,000 bytes; 30 pages and 454 local assets were published
with preimages `out/ffta-lazy-songs-before-ip7zr2q3`. Native Windows and exact
original audio fidelity remain unverified.

Synthetic unit `viewer-nav-synthetic-l9qux7c7` passed 61 browser and generator
checks, including search, selected-folder inspection, topic navigation and saved
selections. Protected local unit `viewer-nav-real-1fs0hl6a` verified all 156
resource entries, preserved the data in all 15 pages, checked the overview and
world page in Linux Chrome including a 375-pixel viewport, and matched all 21
files generated from a synthetic dataset by Python and Docker. That earlier
index was 94,458 bytes, down from 670,218 bytes; the 15 ignored local pages were
published with a recoverable backup under `out/ffta-viewer-nav-before-1cqrfezb`.
Native Windows remains unverified.

The world/gameplay and interface/graphics scopes now load separately. Links
between terrain and background tiles carry the selected descriptor across pages;
old world-page links to moved interface views retain their selections. Resource
pages point to the appropriate scope. Regression unit `viewer-navbar-regression-m7klgbt9`
passed 145 tests; `viewer-navbar-moved-5_u2_mjl` passed 74 checks of the moved
graphics views. Protected local unit `viewer-navbar-real-fya7nxy2` compared
the menu entries against every visible section on 15 topic pages, checked the
audio links and mobile menu in Chrome, preserved the scoped data and all 156
resources, and matched 22 synthetic Python/Docker files. The combined world page
measured 11,955,079 bytes; its replacements measure 3,481,448 and 9,168,863
bytes. Sixteen ignored wiki pages and 33 updated resource links were published
with recoverable preimages under `out/ffta-navbar-before-6fdhi1kt`.
Native Windows and Windows Docker remain unverified.

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

**Focused text scopes.** Separate pointer, relative and script-text pages
load only their associated text projections; the combined text/font page
supports searches across all archives. Name and event-script links target the
relevant focused page. Pinned language, hidden commands and normal-font choices
are saved in browser storage when available. Unit `w2vnicj9` passed all 53
synthetic checks; protected EUR `4aktz7x0` and `574prtnv` verified browser,
Docker and candidate data before publication. At that checkpoint the EUR combined page measured
21,646,829 bytes; the pointer, relative and script pages measured 7,534,167,
11,692,365 and 7,597,975 bytes respectively. Earlier unit `l5t1ecc_` had a
wrapped-link click failure corrected in `byl9lek3`; unit `7916yrmi` timed out
while the filesystem stalled, and `brjnqia7` passed its application checks but
failed runner cleanup because Chrome left a scratch symlink. Their diagnostics
remain available. Browser storage can be disabled; native Windows is deferred.

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

The split wiki's dated **Interpret file bytes** disclosure shows up to 64
manually interpreted 8-, 16- or 32-bit values per page. Choose byte order,
inspect stored bytes and signed/unsigned numbers, jump to a decimal or hex file
offset, and optionally display 16-bit values as BGR555 swatches. Incomplete
values are retained as bytes. Reads use bounded file slices, including files
above 8 MiB; changing files or closing the section discards stale reads. These
interpretations do not establish a file's actual format. Update both the
`inspect-values` badge in `viewer_section_updates.json` and the matching time
in `viewer_stable_inspector.html` when changing that disclosure.

Synthetic `viewer-binary-inspector-synthetic-bwe97m3r` passed both focused
browser cases, including limits, signed extremes, endian, colors, large files,
partial words and in-flight replacement. Thirteen existing regression cases
passed in `viewer-binary-inspector-synthetic-aw7tdfb1`; its second new test
stopped because its browser profile directory was reused. The first attempt
`mifu0ge1` exposed two test-harness waits. The later `a416bed_` confirmed that
legacy `--single-page` still uses its separate inspector; the split wiki and
resource-page test supersede that incorrect single-page expectation. Protected
`viewer-binary-inspector-real-ngnlxz5x` checked 1,212 source bytes, 321 browser
rows in six interpretations, 43 scoped payloads, 165 current and 23 historical
resource payloads, and 1,116 matching Python/Docker files. Published 210 private
files with preimages `out/ffta-binary-inspector-before-ic_2pplj`. The legacy
single-page inspector and native Windows were not changed or verified here.

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
case-insensitively. This array filter does not search nested objects. Results keep their source
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

A separate collapsible JSON structure search recursively matches field names
and scalar values, starting from the whole file or the current entry. It opens
results at their paths, displays 25 at a time, can be cancelled and limits
results to 1,000 and traversal to 256 levels with notices. Previous results
remain visible until replaced; selecting a different file clears them. The
existing 8 MiB structured parsing limit and browser numeric precision caveats
still apply. Synthetic `viewer-deep-json-synthetic-d79ymiyx` passed 12 checks,
including nesting, cancellation, file replacement, bounds and resource pages.
An initial test `pyd8t82i` started Chrome directly on a resource page although
the harness expects an index; the corrected test navigated from the index.
Protected e356 `viewer-deep-json-real-jnmjkl8b` independently checked 128 and
412 field paths in two extracted files in Chrome and matched 573 full
Python/Docker outputs. The 23 wiki pages and 146 current resource pages were
published with their ownership marker and preimages
`out/ffta-deep-json-before-n6plagou`; 23 older resource links remain intact.
Native Windows remains unverified.

The template also contains a suspended, unverified prototype for loading a
large JSON structure on request in a browser worker. Synthetic unit
`rom-data-viewer-largejson-kdrp6i3r` stopped on a test-harness TypeError; the
worker's page-transfer bounds and EUR behavior still need validation. The
generated root page remains at the previously verified version. Resume the
prototype only when the paused viewer task is requested again.

## Add other views after the palette milestone

- [x] **Background screen tile blocks and maps.** Browse five separately
  stored screens and their 3,157 cells with manually selected 4-bit or 8-bit
  tile interpretation, tile base and palette. Inspect source and destination
  positions, tile IDs, flips and palette fields, and optionally highlight
  unavailable tiles. The original map builder replaces exactly `0x03FF` with
  `0x0000`; the preview applies that replacement by default and retains
  `0x23FF` and its palette bits. The 74-byte extra row outside the declared
  third screen is not included. Source offsets, the character base, palette
  pairing and screen names remain structural rather than inferred scenes.
  Synthetic `viewer-e358-synthetic-8tjy9w2q` passed 19 checks; initial
  `5fl81rb9` used an overlapping range while expecting a different guard and
  the corrected fixture passed. Protected `viewer-e358-source-visual-8prc_syk`
  independently compared 1,616,384 RGBA bytes across both depths with the
  authenticated E609 source; protected `viewer-e358-real-6yl44929` checked
  the e358 candidate, all ten source ranges and the earlier resources, and
  matched 575 Python/Docker files. Runtime attachment, actual color mode,
  palette and scene composition and native Windows remain unverified.
- [x] **Background effect tile blocks and maps.** Browse all 25 e356 source
  descriptors, their 4bpp tile sheets and 30×20 tilemaps. Preview with grayscale
  or a manually selected palette, optionally using each stored cell palette ID;
  inspect tile IDs, flips and source offsets, highlight missing shared tiles,
  and play the stored entry order at a chosen rate with optional looping.
  The initial tile ID 82 assumes the documented DMA destination relative to a
  0x06008000 character base; the user can change it. Neither that base, the
  selected palette nor the preview order proves the live scene composition.
  Synthetic unit `viewer-bgfx-synthetic-m_6a25g7` passed 24 checks including
  source guards, pixels and flips, palette selection, missing tiles, focus,
  looping, bookmarks, the single-page output and existing scoped views. An
  initial synthetic run failed to import its new test module; the path was
  corrected. Protected e356 unit `viewer-bgfx-real-ei45gd6n` compared all
  147,008 tile bytes and 15,000 cells with the authenticated ROM and checked
  three complete RGBA previews in Chrome. Native Windows remains unverified.
- [x] **Optional shared tiles in background effects.** A collapsible control
  combines the effect's own block with one manually selected background,
  interface, object, archive or decompressed graphics source and its first
  tile ID. Local effect tiles take precedence; inspect the selected source,
  missing cells, flips and palette IDs. The choices persist when switching or
  playing effects. These manual combinations do not prove the live scene.
  Synthetic `viewer-bgfx-shared-synthetic-6b45l8t1` passed 25 tests; protected
  `viewer-bgfx-shared-real-12m0726h` compared 614,400 RGBA bytes across four
  real Chrome previews and matched 573 full Python/Docker output files. All
  23 scoped data payloads, 169 resource pages and lazy assets were retained;
  the 23 ignored wiki pages were published with preimages
  `out/ffta-bgfx-shared-before-gwdwp1dc`. An initial real check
  `viewer-bgfx-shared-real-_i6sque6` compared an intentionally omitted metadata
  field and failed; the corrected check verified the projected source fields.
  The original source/palette choice and native Windows remain unverified.
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
- [x] **Decoded tilemap stream bank.** Verify the extracted binary source
  ranges and decode 61 halfword streams into the existing tilemap explorer.
  Nine type-5 row/descriptor links identify seven byte streams; reconstruct
  their split source fragments while skipping unowned alignment bytes and
  display their values in a separate grayscale grid with inspectable cells.
  Other catalogs remain available if either the bank or its links are missing
  or malformed. Modes, flags, origins and source references remain visible;
  the views show the first decoded traversal, not the original game composition
  or palette. Synthetic Chrome unit `rom-data-viewer-tilemap-bank-synthetic-h11axgo7`
  passed 56 projection, browser and regression checks. Protected EUR e337/d336
  unit `rom-data-viewer-tilemap-bank-real-33ceaq_x` authenticated both source
  files, independently compared all 8,377 halfword and 1,733 byte cells and
  nine references with the ROM, checked each map in Chrome, preserved prior
  scope data and 145 resource pages, and matched 16 Docker/Python fixture files.
  Eleven ignored wiki pages were published; Windows remains deferred.
  [Stream format](../rom_data_runtime.md#e457-european-tilemap-stream-bank)
- [x] **Signed map offsets.** Browse the 48 signed bytes in the European
  map-reader table with their indices, source offsets, signed values and raw
  byte representations. The view is separately collapsible and linked from the
  raw-transfer resource; a malformed table does not hide valid rectangles, and
  a malformed rectangle does not hide a valid table. The stored bytes are not
  named locations and do not establish an in-game traversal. Synthetic Chrome
  unit `rom-data-viewer-signed-map-synthetic-txfibfdr` passed 41 projection,
  invalid-input, navigation and regression checks. Protected EUR e338/d337
  unit `rom-data-viewer-refresh-e338-view-_yqtclhf` compared every displayed
  value to the authenticated source and preserved 144 prior resource pages.
  [Source and scope](../rom_data_graphics.md#e555-european-signed-map-delta-runtime)
- [x] **Signed map offset preview.** Select a stored offset or its table
  row, view its vertical displacement from zero and play from that entry with
  a chosen rate and optional loop. Positive values move down. The documented
  renderer adds the offset to a region's screen Y; the preview's sequential
  order and rate do not establish the live selection or game timing. The stored
  values table can be collapsed independently. Synthetic Chrome unit
  `s91ptt4q` passed 45 checks. After adding a zero-only case and nested-section
  check, focused unit `8ub0enyk` passed six checks. Protected EUR e339/d338
  unit `rom-data-viewer-refresh-e339-view-fpd_62za` compared all 48 displayed values with the source
  and checked playback, looping and page navigation. Native Windows remains
  deferred. [Renderer](../rom_data_graphics.md#e556-european-world-node-drawing-dispatch)
- [x] **Type-2 source rectangles and copy windows.** Browse complete
  packed source rectangles and their available row windows, list headers and
  transfer-row references in the tilemap explorer. Two palette-row rectangles
  join previously typed prefixes to the packed data; three previously browsed
  rectangles gain their source windows. Incomplete cells, missing dependencies
  and unavailable entry targets are reported without hiding valid rectangles.
  Source values may be palette colors; tile graphics, interpretation and palette
  pairing remain manual. Synthetic Chrome unit
  `rom-data-viewer-type2-rectangles-synthetic-a4urna0v` passed 48 projection,
  partial-source, browser and regression checks. Protected EUR e337/d336 unit
  `rom-data-viewer-type2-real-3gn5lnjv` authenticated 1,482 packed ranges and
  both prior resources, compared 203 rectangles, 2,338 cells, 539 windows and
  their source links with the ROM, and checked all rectangles/windows in Chrome.
  Eleven scope payloads and 145 resource pages were preserved; 18 Docker/Python
  files matched. Twelve ignored wiki pages were published. Other source windows,
  live scene composition and Windows remain unverified.
  [Type-2 source](../rom_data_runtime.md#e483-european-type-2-transfer-resource)
- [x] **Localized queued rectangles.** The tilemap explorer browses the 30
  documented European localized transfer sources as 24×4 halfword grids. The
  selector identifies language and entry IDs; source metadata shows the 48-byte
  source and 64-byte destination row strides. Graphics and palettes are manual
  interpretations and do not establish the actual scene or destination contents.
  Missing or invalid localized data leave other views available. Synthetic unit
  `3zmfg5xd` passed 45/46 checks; its expected hexadecimal value omitted a
  leading zero. The corrected browser check passed `jnaacl22`. Protected EUR
  unit `7uuh8665` authenticated nine source spans, matched all 2,880 halfwords,
  checked all 30 rectangles and preview pixels in Chrome, preserved ten scope
  payloads and 145 resource pages, and matched 15 Docker/Python files. Eleven
  ignored local wiki pages were published; Windows remains deferred.
  [Source contract](../rom_data_ui_raw_transfers.md#e432-thirty-localized-queued-transfer-resources)
- [x] **UI source rectangles and template forms.** The tilemap explorer
  includes all three stored grids from `graphics.ui-raw-transfers`, linked
  from the resource index. For the two templates, choose stored cells, the
  staged 16-bit addition of `0x00C0`, or the direct-draw addition of `0x0300`.
  The staged 12-column template receives two zero columns per row. The cell
  inspector distinguishes original cells from transformed and added values.
  Graphics and palettes remain manually selected; these views do not recreate
  the full interface or establish the live scene. Synthetic Chrome unit
  `iy01h8l1` passed 50 projection, browser and regression checks. Protected
  EUR unit `ptfkl8c_` matched nine source ranges, all 490 stored cells, 728
  cells in seven views and a pixel, and compared 15 Docker/Python files. The
  validated ignored index and world page were published; Windows is deferred.
  [Source contract](../rom_data_ui_raw_transfers.md#e538-european-tilemap-rectangles)
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
- [x] **Indexed graphics programs.** A separate scoped page browses the
  decoded programs in the localized, color-mode, White Flowers, additional and
  reader-bank families. Choose an archive, program, 4-bit or 8-bit tile depth,
  grayscale or a stored palette, and tile page. Select a tile for its decoded
  byte offset; archive source fields and packet counts remain available. Play
  successive programs from the selection at a chosen rate with optional looping.
  Program order, chosen graphics interpretation and palettes do not establish
  a game animation or original timing. Synthetic unit `vrjlte3s` passed 54
  viewer and generator tests; follow-up `a7de4j7_` checked both catalog formats.
  Protected EUR unit `thlfcw8m` independently decoded all 3,811 programs in
  75 archives (677,408 bytes), compared their browser hashes and 24 sample
  pixels, preserved other scoped data and 145 file pages, and matched 16
  Docker/Python files. The dedicated page was 2,355,553 bytes. The validated
  ignored index and ten topic pages were published. Earlier `zhbkd6xw` exposed
  an unrecognized v2 resource format; the corrected run passed. Primary
  Terra/xhigh settings were not independently confirmed. Windows is deferred.
  [Archive format](../rom_data_indexed_graphics.md#e507-european-indexed-reader-banks)
- [x] **Indexed tile attributes.** A separate Graphics page browses two
  direct and five language-slot tables: 804 programs and 4,096 source nibbles.
  It shows source positions and the documented getter result, with two-way links
  to matching archive programs. Source owners are checked independently; missing
  owners leave available tables visible and mark the remainder unavailable.
  Synthetic `viewer-e372-attrs-synthetic-ilrzqp0j` passed 19 projection,
  Chrome, navigation and regression tests. Protected EUR
  `viewer-e372-attrs-real-ycgw4xrl` compared all 1,187 new source bytes and all
  4,096 displayed nibbles, retained 38 prior scoped payloads and 184 resource
  pages, and matched 1,109 Python/Docker files. The ignored local e372-d371 wiki
  was published with verified preimages in `out/ffta-e372-attrs-before-pii_wo82`.
  Nibble meanings, their assignment to rendered tiles and native Windows remain
  unverified. [Source evidence](../rom_data_indexed_graphics.md#e639-european-indexed-tile-attribute-tables)
- [x] **Direct indexed archives.** Two independently rooted archives with
  four and 41 programs are available on the indexed-graphics page, with their
  own resource links, source file and caller-literal positions. Their previews
  reuse the tile depth, palette and optional ordered playback controls. Each
  source is validated independently, so missing or invalid data do not hide
  other families. Synthetic unit `32khwscw` passed 42 tests. Protected EUR
  unit `uyz3u8oo` matched the four source hashes and independently decoded all
  45 programs (2,880 bytes), checked eight browser pixels and both relocated
  file-page links, preserved the other 143 file pages and matched 17
  Docker/Python files. Fourteen ignored wiki files were published. The initial
  synthetic unit `7bum0pad` had an incorrect expected ID; `q4ru284l` expected
  two topic links to remain unchanged. The corrected checks passed; primary
  Terra/xhigh settings were not independently confirmed. Interpretation,
  palettes and preview timing remain manual; Windows is deferred.
  [Direct sources](../rom_data_indexed_graphics.md#direct-european-archive-at-3ae484)
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
- [x] **Compressed tilemap transfer inputs.** Decode the individual LZ77
  segments in `tilemap-transfer-lz77.bin` only after checking manifest ranges,
  source hashes, overlap, encoded extent and bounded output. Their outputs are
  available alongside the earlier observed streams in the tile-sheet, tilemap
  and object-animation previews; the source file and offset remain visible.
  Interpretations, graphics depth and palettes are chosen manually and do not
  establish a finished game image or live pairing. Synthetic unit
  `rom-data-viewer-transfer-lz-synthetic-edpestec` passed 54 of 55 checks;
  the remaining fixture used a 32-byte tile for an 8-bit preview and the
  corrected four-test unit `rom-data-viewer-transfer-lz-retry-fk63b8__` passed.
  Protected EUR e336/d335 unit `rom-data-viewer-transfer-lz-real-xaic34k0`
  authenticated all 13 compressed ranges, independently decoded 6,112 bytes,
  checked all 13 tile previews and their availability in the other views,
  preserved prior scope data and 145 resource pages, and matched all 19
  Docker/Python fixture files. Eleven ignored wiki pages were published.
  Windows remains deferred. [Source contract](../rom_data_runtime.md#e480-european-task-345-lz77-resource)
- [x] **Transfer selector directory.** Browse the stored selector lists, row
  fields, shared rows, descriptors and wrappers in a collapsible interface view.
  Direct wrapper links open a unique matching decoded LZ77 source in the tile
  sheet, including earlier observed streams; missing or ambiguous targets remain
  unavailable. Tile depth, colors and scene composition are manual choices.
  Synthetic Chrome unit `viewer-transfer-directory-synthetic-g4b2wsdh` passed
  49 tests, including malformed or missing data, shared and empty selectors,
  direct previews and bookmarks. Protected e343 unit
  `viewer-transfer-directory-real-qzgr5pwp` authenticated 85 source spans and
  reconstructed the 23 selectors, 18 rows and 16 descriptors from the source;
  Chrome checked all 23 links, three empty selectors and shared rows. It preserved
  the other scope payloads and matched 20 Python/Docker files. Sixteen ignored
  wiki pages were updated with guarded preimages. Earlier real unit `1f4pbo5v`
  failed because its browser test used a single-element selector as a list;
  the corrected test passed. Windows remains deferred.
  [Source contract](../rom_data_runtime.md#e468-european-task-345-managed-transfer-metadata)
- [x] **Indexed tilemap control links.** Inspect stored indexed pointers, rows
  and descriptor fields, then open uniquely matched tilemaps. Each map links
  back to its matching control slots; mismatched pointers and missing or
  ambiguous map sources have no preview link. Graphics and palettes remain
  manual choices. Synthetic Chrome unit `viewer-indexed-maps-synthetic-wsvczx61`
  passed 54 checks covering forward and reverse links, partial data, ambiguous
  sources and scoped navigation. Protected e343 unit `viewer-indexed-maps-real-38kkyllk`
  authenticated 12 source spans, reconstructed all 12 control records from the
  source and checked every forward and reverse link in Chrome. It preserved the
  other scope payloads and matched 20 Python/Docker files; sixteen ignored pages
  were published with guarded preimages. Earlier real unit `euf9mysl` failed
  because its browser test treated a single selected element as a list; the
  corrected check passed. Windows remains deferred.
  [Source contract](../rom_data_runtime.md#e478-european-indexed-tilemap-control-resource)
- [x] **Type-3 transfer payload previews.** Verify each packed source extent
  and show complete LZ77 outputs and raw transfer inputs on a separate offline
  page. Decode each LZ77 stream independently; an invalid stream is reported
  and omitted without hiding other valid payloads. The tile-sheet view supports
  4-bit and 8-bit interpretation, manual palettes, pages and byte offsets.
  Payloads can contain non-graphics data; these controls do not establish an
  original palette, composition or live scene. Synthetic Chrome unit
  `rom-data-viewer-type3-payloads-synthetic-hgs0dbzz` passed 45 projection,
  isolation, browser and regression checks. Protected EUR e337/d336 unit
  `rom-data-viewer-type3-real-geoui_0a` authenticated 2,214 source ranges,
  independently decoded all 323 LZ77 inputs and compared 14 raw inputs,
  totaling 502,064 output bytes. Chrome checked all 337 previews at both tile
  depths; ten existing scope payloads and 144 resource pages were preserved,
  and 16 Docker/Python fixture files matched. Twelve local wiki pages and one
  changed resource page were published. Initial synthetic unit `bet42kmn`
  exposed an incompatible offset message; the accepted unit restored it.
  Windows remains deferred. [Source](../rom_data_runtime.md#e482-european-type-3-transfer-resource)
- [x] **On-demand transfer graphics for maps and animations.** The two world
  previews share a locally generated, content-addressed transfer graphics file,
  loaded only on request or when restoring a bookmarked transfer selection.
  Existing selections are preserved, absent or mismatched files can be retried,
  and single-page output uses its embedded data. Keep the companion asset with
  the other generated pages. Synthetic Chrome unit
  `rom-data-viewer-transfer-graphics-lazy-synthetic-mls3lln9` passed 44 offline,
  selection, retry, bookmark and regression checks. Protected EUR e337/d336 unit
  `rom-data-viewer-transfer-lazy-real-98zd4dr0` checked 323 compressed and 14 raw inputs in both views,
  sampled rendered pixels, restored a bookmark, preserved 11 prior scope
  payloads apart from the added world asset reference and 145 resource pages,
  and matched 20 Docker/Python fixture files. Twelve ignored wiki pages
  and one companion asset were published. Graphics interpretation, palette and
  scene pairing remain manual; native Windows remains deferred.
  [Type-3 source](../rom_data_runtime.md#e482-european-type-3-transfer-resource)
- [x] **On-demand transfer graphics for object animations.** Load compressed
  and raw transfer inputs into the object-animation viewer only when requested.
  The existing local asset is shared with world previews in single-page output;
  split topic pages refer to it without embedding it. Existing graphics choices
  remain selected, bookmarks restore transfer choices, and missing or mismatched
  assets can be restored and retried. Synthetic unit `qu1hh1xv` passed 46 of 47
  checks; one fixture expected the first-byte color at a later pixel. After
  correcting that expectation, Chrome unit `0ldr0r2e` passed all eight focused
  loading, pixel, error, bookmark and regression checks. Protected EUR e338/d337
  unit `rom-data-viewer-obj-transfer-real-8cy6xzm5` checked 323 compressed and 14 raw inputs, sampled a
  manually paired pixel from each kind, restored a bookmark, preserved 11 scope
  payloads apart from the new object-page asset reference and 145
  resource pages, and matched 22 Docker/Python files. Initial real
  unit `torujq_d` observed the disabled button before the async asset loaded;
  the accepted check waited for the populated selector and payload. Twelve ignored
  wiki pages were published using the existing asset. Graphics/palette and scene
  pairing remain manual and native Windows remains deferred.
  [Type-3 source](../rom_data_runtime.md#e482-european-type-3-transfer-resource)
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
- [x] **Localized item names.** The authenticated European item record's
  property-0 index links to the original five-language name reader. Show names
  in the item selector, item-effect selector and encounter equipment links;
  provide a language selector, a filterable, paginated directory and links to
  the source strings. Unknown glyphs retain their bitmaps. Other profiles,
  changed source identities, missing or ambiguous readers keep item IDs without
  guessed names. The world page embeds only the referenced names and glyphs,
  not the full text catalog.
  Synthetic unit `item-names-synthetic-675ze1_1` passed 54 projection, browser
  and regression tests. Protected EUR unit `item-names-recheck-5xhxehqn`
  verified 376 item records, 1,880 localized references and 81 glyphs against
  the ROM, checked Chrome navigation and matched 19 Python/Docker files using
  the actual item and text inputs. Initial unit `item-names-real-w97e1kgo`
  stopped on a test-only DOM iteration error; the corrected unit reused its
  candidate and Docker output. Fifteen ignored wiki pages were published with
  backup `out/ffta-item-names-before-0uxlj1k1`. Native Windows remains deferred.
  [Original lookup](../rom_data_gameplay.md#original-field-access-contract)
- [x] **Item equipment sprites.** Verified European item records link their
  normal stored equipment-sprite selectors to an available graphics frame and
  the existing animation player. The equipment view lists matching items by
  localized name, with filtering, pagination and links back to item records.
  Item zero, unsupported styles, missing frames and unverified sources do not
  gain inferred links. Action overrides, live palettes and actor composition
  remain separate from this base-sprite preview. Synthetic unit
  `viewer-item-sprites-synthetic-_3ilf95_` passed 52 checks; focused final unit
  `viewer-item-sprites-final-synthetic-4vczqawj` passed three checks after a
  label correction. Protected e345 unit `viewer-item-sprites-real-acxkc4cz`
  reconstructed 12,032 item bytes, authenticated 593 sprite source ranges,
  checked 252 item links and rendered all 61 referenced sprites in Chrome, and
  matched 24 Python/Docker files. Eighteen ignored pages were published with
  backup `out/ffta-item-sprites-before-j7qpzr0d`; Windows remains deferred.
  [Equipment source](../rom_data_unit_obj.md#equipment-poses-source-model)
- [x] **Evaluation coefficients and thresholds.** Browse the seven score
  divisor/weight pairs, two bonus pairs and five class thresholds, including
  source offsets and uninterpreted bytes. On authenticated European data, a
  collapsible calculator evaluates supplied 32-bit score inputs and predicate
  results with the documented signed/unsigned division and wrap rules; another
  explores class thresholds, including class 0 and signed low-16 draw behavior.
  Invalid values and zero divisors are identified. Unverified European sources
  show stored fields without offering the authenticated-reader calculations. This
  does not infer live input meanings, class selection or the complete game
  evaluation. Synthetic `hoypkpto` passed 20 checks and compared 800 arithmetic
  cases. Protected e347 `h0dlklc6` authenticated 38 source bytes in two ROM
  ranges, compared 1,000 scores and 65 thresholds in Chrome, preserved 21 other
  scope payloads and 156 resource entries, and matched 27 Python/Docker files.
  Published 23 ignored pages with preimages `out/ffta-evaluation-before-u722enjh`.
  Windows remains deferred. [Source proof](../rom_data_gameplay.md#evaluation-parameters-source-proof)
- [x] **Conditional descriptors and lookup tables.** Browse 684 records in
  banks 1–23 and the 15 documented bank-0 rows, with bank selection, filtering,
  pagination, source offsets and stored fields. The packed table shows all 33
  selected indices and reconstructs the original bytes from their bit fields;
  the independent four-byte lookup exposes 74 rows and the property table
  shows 35 masks and their set bits. Authenticated European type-5 descriptors
  link to the 42 in-slice packed references; unverified sources show raw fields
  without inferred links. Bank counts and ranges are conditional source bounds,
  not global selector limits or proven names. Synthetic
  `viewer-conditional-lookups-synthetic-lq4j_4mg` passed 20 projection, browser,
  navigation and regression checks. Protected e359
  `viewer-conditional-lookups-real-hfdu205d` reconstructed all 699 descriptor
  rows, 33 packed rows, 74 lookup rows and 35 masks against the original source,
  checked their pointer words and 42 references, exercised a link in Chrome,
  preserved 22 scoped payloads and 172 resource pages, and matched 602
  Python/Docker outputs. Published 23 local pages with preimages
  `out/ffta-conditional-lookups-before-fwzk6jw3`. Live uses, field meanings
  and native Windows remain unverified.
  [Descriptor source](../rom_data_gameplay.md#european-conditional-eight-byte-descriptor-banks),
  [packed lookup](../rom_data_gameplay.md#european-conditional-packed-four-byte-projection),
  [four-byte lookup](../rom_data_gameplay.md#european-conditional-four-byte-lookup-projection),
  [mask source](../rom_data_gameplay.md#conditional-property-selected-mask-table)
- [x] **Conditional source records.** Browse all 127 physical slots with
  gate, context and stored keys, filter and paginate them, and open a diagnostic
  using supplied flag or counter values. Authenticated European inputs show the
  documented predicate polarity, second-key bypass and shared-key behavior;
  unverified sources remain raw. The diagnostic assumes values stay fixed and
  does not read or change live game state or infer names. Synthetic `9ba0k1pv`
  passed 20 tests including 1,500 independently modeled cases. Protected e352
  `apxnqiol` reconstructed all 1,016 source bytes, checked 76 nonzero gates,
  105 live references and 2,032 diagnostic cases, exercised Chrome, preserved
  21 other scopes and 156 resources, and matched 27 Python/Docker files.
  Published 23 ignored pages with preimages `out/ffta-cond8-before-nevc6pr_`.
  Windows remains deferred. [Source contract](../rom_data_gameplay.md#european-conditional-eight-byte-record-source-contract)
- [x] **Packed-pair sources.** Browse the two indexed banks (92 and 16
  records of 48 pairs) and the separately rooted global source (44 pairs) in
  collapsible matrices. Bank and record selectors preserve the display mode;
  each row shows the source byte and its four numbered pairs. Authenticated
  European inputs offer the documented low/high reader-bit choices; other
  valid inputs show stored pairs only. Slots 44–47 remain visible without an
  inferred use or owner. Synthetic `1g0n26br` passed 13 projection, browser and
  navigation tests. Protected e352 `k5sw6170` compared 1,307 source bytes and
  15,684 matrix cells against ROM-derived values, preserved 21 other scope
  payloads and 156 resources, and matched 28 Python/Docker files. Published 23
  ignored pages with verified preimages `out/ffta-pairs-before-kvnjoowb`.
  Meanings, live state and native Windows remain unverified.
  [Indexed source](../rom_data_gameplay.md#indexed-twelve-byte-packed-pair-source-contract),
  [global source](../rom_data_gameplay.md#global-packed-pair-flags-source-proof)
- [x] **Item-derived parameter records.** Browse the 31 stored four-byte
  records, their source offsets, two bytes and little-endian halfword. Verified
  European item and parameter sources link each item's stored byte `0x18` to
  its record; the reverse list is filterable and paginated, with recognized
  localized item names when available. Unverified sources remain browsable
  without inferred item links. Field meanings, live selectors and formatter
  paths outside this conditional projection remain unproved. Synthetic unit
  `viewer-item-parameters-synthetic-yoolwrlr` passed 17 checks, including
  guard and browser navigation cases. Protected e347 unit
  `viewer-item-parameters-real-4bagrcjs` reconstructed 12,032 item bytes and
  124 parameter bytes against the source ROM, checked all 376 item links and
  31 records in Chrome, preserved other topic payloads, and matched 27
  Python/Docker files. Twenty-one ignored wiki pages were published with
  preimages under `out/ffta-item-params-before-tuxai27j`; Windows is deferred.
  [Source contract](../rom_data_gameplay.md#conditional-item-derived-four-byte-parameters)
- [x] **Item effect parameters.** Browse the fourteen documented European
  item-linked records, their nine coefficients, cap and opaque bits in a
  collapsible view, with links in both directions to item records. An optional
  calculator accepts a property, current value and quantity as bytes and shows
  the bounded increase from the documented helper. It does not change game
  state or reproduce the complete item-use UI. The view remains available when
  item records are absent and does not infer this mapping for other profiles.
  Synthetic unit `3d69s9dq` passed 39 tests; profile guard `8lkn7_xq` passed.
  Protected EUR unit `j0h3etxz` reconstructed and authenticated all 98 source
  bytes, checked 1,848 calculation cases and both links in Linux Chrome,
  preserved ten scope payloads and 145 resource pages, and matched 16
  Docker/Python files. Eleven ignored wiki pages were published. Windows is
  deferred. [Source contract](../rom_data_gameplay.md#item-effect-parameter-static-contract)
- [x] **Progression and pricing records.** Four independent, collapsible EUR
  views present stored progression deltas, receipt requirements, selection
  conditions and trade parameters. Records can be filtered by ID or ROM offset
  and paged; trade discount count zero is shown as the high byte of markup,
  with all later discounts and unclassified bytes available in nested panels.
  The views preserve fields without inferring skill names, item identities,
  live state, ordinary reachability or game outcomes. Synthetic Linux Chrome
  unit `rom-data-viewer-progression-synthetic-lmc8ahj2` passed 44 projection,
  browser and regression checks, including missing and invalid catalogs.
  Protected EUR e336/d335 unit `rom-data-viewer-progression-real-lf7sv8ry`
  compared 233 records and 3,272 reconstructed bytes with authenticated source,
  checked every displayed row in Chrome, preserved the prior scope data and
  145 resource pages, and matched all 18 Docker/Python fixture outputs. Eleven
  ignored local wiki pages were published. Windows remains deferred.
  [Source contracts](../rom_data_gameplay.md#next-ten-byte-progression-and-price-records)
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
- [x] **Item-pair and metadata links.** For authenticated European pair and
  metadata catalogs, active nonzero selectors link to matching metadata rows;
  the reverse list opens each matching pair and slot. Selector zero, `0xFF`,
  slots beyond the count, missing rows and unsupported source identities do
  not receive links. Direct names appear only with the authenticated text
  catalog; otherwise the numeric references remain usable. Item owners use
  the available localized item labels. The links show stored relationships,
  without evaluating live unit conditions. Synthetic unit
  `item-metadata-synthetic-0uvnwvf9` passed 49 tests. Protected EUR unit
  `item-metadata-real-2lo4iet7` reconstructed 225 pair records and 116 metadata
  rows from the ROM, checked 464 active links in both directions and eight
  `0xFF` slots in Chrome, and matched 20 Python/Docker files using the actual
  extracted inputs. Fifteen ignored wiki pages were published with backup
  `out/ffta-item-metadata-before-gfcuvkm4`; native Windows remains deferred.
  [Source contract](../rom_data_gameplay.md#item-linked-pair-source-contract)
- [x] **Sixteen-byte linked records.** A separate, dated World page makes all
  128 authenticated records browsable by ID, with their alias, 14 source bytes,
  bounded display-helper index and known reader uses. An expandable comparison
  table opens a selected row. Record names and live two-record outcomes are not
  inferred. Synthetic `viewer-metadata16-synthetic-5_w35yrk` passed 18 source,
  Chrome, navigation and regression checks. Protected EUR
  `viewer-metadata16-real-ez08ndf1` matched all 2,048 source bytes and 1,792
  displayed fields, preserved 39 prior scoped payloads and 184 resource pages,
  updated the original file page, and matched 1,110 Python/Docker files. The
  ignored local wiki was published with preimages in
  `out/ffta-metadata16-before-w0jea1dw`. Native Windows is deferred.
  [Reader contract](../rom_data_gameplay.md#sixteen-byte-linked-metadata-source-contract)
- [x] **Byte-pair metadata lookup.** Browse the 44 European key/value rows,
  their source offsets and links to verified localized metadata names. The
  metadata view links back to matching rows. A filter supports row, key, value
  and recognized name; a small calculator reproduces the documented forward
  lookup and inverse search, including the first matching key, zero on a miss
  and exclusion of row 43 from the inverse scan. Unverified files remain
  browsable without source-specific lookup claims or inferred value meanings.
  Synthetic unit `viewer-metadata2-synthetic-ug6y7tuw` passed 53 checks.
  Protected e345 unit `viewer-metadata2-real-ylvqkc4o` reconstructed all 88
  bytes from the source ROM, checked 44 forward rows, all 256 inverse inputs
  and 44 bidirectional metadata links in Chrome, and matched 25 Python/Docker
  files. Eighteen ignored wiki pages were published with backup
  `out/ffta-metadata2-before-1pc1ol2x`; Windows remains deferred.
  [Source contract](../rom_data_gameplay.md#two-byte-forward-and-inverse-source-contract)
- [x] **Two-pair metadata records.** Browse all twenty physical records and
  both selector/value slots, including zeros and stored values after a zero.
  The view distinguishes nonzero counts from the original leading walk. For
  authenticated European pair and metadata catalogs, selectors link to direct
  metadata names when the text source is also authenticated; metadata rows link
  back to both their raw offset-0x30 record and incoming selector slots. Owner
  lists support filtering and pagination. These are stored source references,
  not an evaluation of live job requirements or redirected rows.
  Synthetic unit `viewer-metadata4-synthetic-udnualg5` passed 54 checks.
  Protected EUR unit `viewer-metadata4-real-gz6mz0ix` reconstructed 6,112 ROM
  bytes, checked all 40 slots, 26 nonzero selector links and 116 metadata owner
  links in Chrome, and matched 20 Python/Docker fixture files. Sixteen ignored
  wiki pages were published with backup `out/ffta-metadata4-before-l1_1rvdt`.
  [Source contract](../rom_data_gameplay.md#four-byte-two-pair-source-contract)
- [x] **Special-unit source records.** Browse the 93 stored records, including
  their default job metadata references, normal and alternate battle sprite
  IDs and packed palette selectors. Verified job references link both ways to
  localized metadata names; these are job references, not character names, and
  live descriptors can override them. The two documented palette branches
  show the selected destination bank and, when available, one of 27 verified
  source palettes. Sprite links open a graphics frame and apply that selected
  palette as a preview; live palette state and actor composition are not inferred.
  Synthetic Chrome unit `viewer-special-units-synthetic-fk7q0v4h` passed 50
  checks, including identity guards, missing sources, reverse links, palette
  pixels and cross-page navigation. Protected EUR e343 unit
  `viewer-special-units-real-e0vzt_ay` reconstructed all 1,302 record bytes,
  checked the 27 palettes against the source, verified 51 active palette
  branches, 186 sprite links and 93 metadata reverse links, and matched 20
  Python/Docker files. Sixteen ignored wiki pages were published with guarded
  preimages. Windows remains deferred.
  [Record source](../rom_data_gameplay.md#typed-fourteen-byte-records-source-proof) ·
  [Palette source](../rom_data_color_palettes.md#original-source-contract)
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
- [x] **UI context templates.** Browse all eleven stored templates, plot their
  coordinates against a 240×160 reference and inspect the original entry
  counts, flag byte and every opaque byte. A dropdown and table connect the 15
  documented category values to their templates only when the EUR ROM identity
  and reconstructed source bytes match the verified source. The view remains
  available without that identity, but does not invent category labels, window
  sizes or live interface behavior. Synthetic unit
  `viewer-ui-context-synthetic-tre5gvyf` passed 45 checks. Protected EUR unit
  `viewer-ui-context-real-qfn6g2c9` reconstructed all 220 source bytes, compared
  the category mapping with the original-selector oracle, checked navigation,
  selection and bookmarks in Chrome, and matched 20 Python/Docker fixture files.
  Sixteen ignored wiki pages and one updated resource link were published with
  recoverable preimages under `out/ffta-ui-context-before-8goe0eri`.
  [Source contract](../rom_data_ui.md#category-selected-ui-context-templates)
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
- [x] **Effect/event and task links.** Browse stored event rows, effect
  selectors, both even same-state dispatch tables and all three physical task
  descriptor catalogs. Event-script effect links open their corresponding
  selectors; documented selector/event references lead to script descriptors
  and back. Display both distinct task-ID formulas for the script catalog,
  nullable odd Thumb callbacks, source offsets and raw fields without inferring
  execution. Missing or invalid catalogs leave independent sections available.
  Synthetic Chrome unit `rom-data-viewer-effect-tasks-synthetic-x8jlkpdd`
  passed 44 projection, isolation, browser and regression checks. Protected EUR
  e337/d336 unit `rom-data-viewer-effect-tasks-real-kf53rnm5` independently
  reconstructed 4,812 source bytes; checked 412 descriptors, 75 events, 51
  selectors, 28 dispatch entries, 493 script-selector links and 70 task links
  in Chrome; preserved ten scope payloads and 143 resource pages; and matched
  17 Docker/Python files. Eleven local wiki pages and two changed resource
  pages were published with historical ownership retained. Windows is deferred.
  [Effect source](../rom_data_events.md) · [Task source](../rom_data_tasks.md)
- [x] **Auxiliary script archive.** Browse the separate auxiliary banks, slots
  and all physical programs. Slot links preserve interior instruction entries;
  reverse links select those entries and the table pages and filters stored
  opcodes. An independent directory shows each callback and byte advance.
  Missing or invalid archive/directory files leave the other view usable. No
  predicate, execution order or argument meaning is inferred. Synthetic Chrome
  unit `rom-data-viewer-auxiliary-scripts-synthetic-6bo4tsoz` passed 40 browser,
  isolation and regression checks. Protected EUR e337/d336 unit
  `rom-data-viewer-auxiliary-scripts-real-h6or111a` independently reassembled
  all 4,200 source bytes and checked eight banks, 442 slots, 133 programs, 913
  instructions and 17 directory records in Chrome. Ten scope payloads and 143
  unchanged resource pages were preserved; 16 Docker/Python files matched.
  Eleven local wiki pages and two changed resource pages were published, while
  historical resource-page ownership was retained. Initial unit `bwxi9vzj`
  stopped because its inventory comparison did not allow four older retained
  pages; the successful unit merged their ownership. Windows remains deferred.
  [Source contract](../rom_data_scripts.md#european-auxiliary-script-archive-source-contract)
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
- [x] **Text reader registry.** Browse and filter stored callback slots by
  index or address. Unique Thumb-address matches link to extracted relative
  archives, which link back to all matching slots; null, invalid, missing and
  ambiguous entries stay visible without guessed links. The other-resources
  page carries only the small registry, while the focused and combined text
  pages also provide the reverse links. Address equality does not prove a
  callback executed. Synthetic unit `text-reader-registry-synthetic-pja7ibuu`
  passed 47 projection, browser and regression checks. Protected EUR e340 unit
  `text-reader-registry-real-690a5zd6` compared all 27 callback words and three
  source spans to the authenticated ROM, found 24 matching slots for 23 unique
  readers, checked real navigation in Chrome and matched 20 Docker/Python fixture
  files. Fifteen ignored local wiki pages were published with a recoverable
  backup under `out/ffta-text-registry-before-b_z8hud7`; Windows is deferred.
  [Source evidence](../rom_data_text.md#conditional-european-text-reader-registry)
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
- [x] **Direct metadata name references.** A separate collapsible view lists
  all 116 EUR metadata rows, with language selection, ID/name filtering,
  pagination and links to the original pointer text. The direct property-0
  values use the documented `080165E8` reader; nonzero routing is shown without
  evaluating it. References do not prove live role or job availability.
  Missing or ambiguous text catalogs leave names unavailable, and unknown
  glyphs retain their source image. Unit `rom-data-viewer-metadata-names-retry-318wv1u6`
  passed four synthetic projection/browser tests. Earlier unit
  `rom-data-viewer-metadata-names-synthetic-e7zs0drd` passed 40 of 41 checks;
  its remaining test reused a temporary Chrome profile and passed after the
  test harness was corrected. Protected EUR e335/d334 unit
  `rom-data-viewer-metadata-names-real-t_lxfhg8` compared all 112 direct rows
  and 560 localized strings with the authenticated metadata and text files,
  kept four redirected rows unresolved, and checked browser navigation, ten
  existing scope payloads and 145 resource pages. The synthetic Docker and
  Python outputs matched for all 17 files. Eleven ignored wiki pages were
  published from the verified output; Windows remains deferred.
- [x] **Job sprite references.** Direct metadata rows link the stored normal
  and alternate sprite IDs to the battle actor preview when both extracted
  source catalogs match their verified EUR identities. Selected job names open
  their corresponding metadata row. Users may choose grayscale or either stored
  palette nibble and a color-mode variant; the choice previews a source bank and
  does not infer the live palette, actor composition or redirected rows. The
  references and localized names stay on the world page; graphics and palettes
  load on the battle page. Synthetic unit `viewer-job-sprites-synthetic-zdg6f3ml`
  passed 52 checks, including identity guards, missing sources, palette pixels
  and navigation. Protected e344/d343 unit `viewer-job-sprites-real-1qfq_2b2`
  reconstructed 6,032 metadata bytes, checked 110 normal and 100 alternate
  sprite links and four redirects in Chrome, and matched 26 Python/Docker files.
  Sixteen ignored wiki pages were published with backup
  `out/ffta-job-sprites-before-3d2a1t89`; Windows remains deferred.
  [Sprite source](../rom_data_unit_obj.md) ·
  [Palette source](../rom_data_color_palettes.md#original-source-contract)
- [x] **Action-linked records and names.** Browse all physical 28-byte records,
  including their stored fields and flag bits. An original European lookup
  connects each first halfword to the localized text reader; names can be
  filtered, paged and opened at their source string. Missing text, ambiguous
  readers or other profiles leave the record fields browsable without guessed
  names. Only referenced names and glyphs are added to the world scope; the
  complete text catalog stays on its own page. Source unit
  `action-name-source-disasm-ndhlj_2c` records the original paths. Synthetic
  unit `action-names-synthetic-qdr71hgk` passed 51 tests. Protected EUR unit
  `action-names-real-yqidzlof` verified 347 records, 1,735 localized references
  and 88 glyphs against the ROM, exercised Chrome navigation, and matched 19
  Python/Docker files using the actual extracted inputs. Fifteen ignored wiki
  pages were published with backup `out/ffta-action-names-before-bdziom0j`.
  The source links do not establish every record's live role or action effects;
  native Windows remains deferred.
  [Original lookup](../rom_data_gameplay.md#european-indexed-28-byte-metadata-source-contract)
- [x] **Action-linked four-byte records.** Browse all 211 physical records
  and their four bytes. With authenticated European action and record sources,
  each action links its first three stored inline selectors to these records;
  each record lists matching action references with localized names, language
  synchronization, filtering and pagination. Zero remains a valid record and
  the fourth inline byte is not inferred as a link. Optional collapsible views
  expose the 107 encoded callback pointers and 93 callback records without
  executing them or assigning live effects. Synthetic `2q5a0oih` passed 24
  checks and `avhywubm` passed three final checks including missing callbacks.
  Protected e352 `z15qj5zj` reconstructed all source bytes across six ROM
  ranges, checked 1,041 action references to 194 records and Chrome navigation,
  preserved 21 other scope payloads and 156 resources, and matched 29 Python/Docker
  files. Published 23 ignored pages with preimages
  `out/ffta-action-slots-before-xz84ixn3`. Live action effects and Windows
  remain unverified. [Source contract](../rom_data_gameplay.md#european-linked-four-byte-record-source-contract)
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
- [x] **Localized and directly selected OBJ animations.** Browse all 130
  language/mode descriptor banks and three direct banks: 188 sequences, 4,435
  stored frames, 2,935 lists and 12,395 objects. Authenticated e357 fragments
  are joined with the existing Firaga, static-list and UI graphics owners;
  the preview reads only the extracted dataset. Select a bank, sequence and
  frame, step or loop at a chosen rate, and manually choose graphics and a
  palette. The three directly selected custom graphics outputs are available
  as explicit graphics choices. Source labels use language and mode IDs; they
  do not invent character names or live scene associations. Synthetic
  `viewer-e357-synthetic-faahz7xi` passed 17 checks including resource links,
  graphics, looping, empty sequences and retained settings. Protected
  `viewer-e357-projection-3q5fi0k7` checked all reconstructed source bytes;
  `viewer-e357-visual-iybqjbhq` independently decoded all three graphics
  streams and compared 349,664 RGBA bytes across three Chrome previews.
  Affine/special object modes, original live graphics/palette selection,
  task timing and native Windows remain unverified.
- [x] **Linked graphics for localized OBJ animations.** Use the authenticated
  `71e658` language selector and each descriptor's tile offset and byte count
  to select the matching decoded graphics for all 130 localized banks; the
  three directly selected banks use their documented graphics streams. The
  **Use linked tiles** button applies the selection once, while **Follow
  selected bank** reapplies it when changing banks. A manual graphics or tile
  base change disables following. If following is disabled, a changed bank
  identifies any retained previous source; missing graphics are marked instead
  of silently reusing the previous tiles. Alignment starts at the lowest tile ID
  referenced by that bank, a preview convention rather than proof of live VRAM
  placement. Palette selection remains manual. Protected
  `viewer-obj-tiles-confirm-gwjxj_ix` independently decoded all 459 programs
  from five language archives, checked the 130 descriptor ranges and compared
  39,232 RGBA bytes across five Chrome previews; all 575 Python/Docker output
  files matched and 171 earlier resource pages were preserved. Synthetic
  `viewer-obj-tiles-synthetic-s7b5v7zo` passed 20 tests, including following,
  manual overrides, missing sources, prior-source indication and existing
  controls. Direct graphics were separately compared in Chrome against
  independent decoding by `viewer-obj-tiles-direct-yei21d6y` (349,664 RGBA
  bytes). The initial real check `estdv1wg` passed pixel checks but failed before
  Docker due to a shadowed script variable; the corrected run passed. The 23
  wiki pages were published with preimages `out/ffta-obj-tiles-before-bkcsr_sl`
  and then `out/ffta-obj-tiles-confirm-before-tax2dcqq`. Original palette,
  live allocation, scene use and native Windows remain unverified.
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
- [x] **Constructor OBJ phases.** The object-animation page links the
  separately stored threshold/pointer table to its four authenticated static
  lists. Browse and play each phase from the selected frame, loop it and choose
  graphics and palettes manually. The preview starts each 8-bit counter at
  zero; thresholds below 255 produce threshold plus one preview ticks, while
  a larger threshold stops because the byte counter wraps. Tick rate is chosen
  by the user and does not establish original scene timing. Synthetic Chrome
  unit `rom-data-viewer-constructor-obj-synthetic-l19e38kl` passed 47
  projection, missing-source, playback, navigation and regression checks.
  Initial unit `q2_tbqhn` exposed a JavaScript inherited-property collision;
  the catalog key was changed and the failing navigation tests passed on retry.
  Protected EUR e338/d337 unit `rom-data-viewer-constructor-obj-real-fe1v12d_`
  authenticated five source spans, independently checked four phases and eight
  objects, verified playback in Chrome, preserved 11 scope payloads apart from
  the documented resource move and new view and 144 resource pages, and matched
  18 Docker/Python fixture files. Fourteen ignored wiki files were published.
  Graphics/palette pairing, live task state and original timing remain unproved;
  native Windows is deferred. [Source](../rom_data_obj.md#conditional-constructor-obj-animation-source-and-model)
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
- [x] **Complete physical compact object banks.** A separate page reconstructs
  all 199 e360 banks, 3,135 object lists, 13,633 objects, 560 sequences and
  5,045 frames from 592 new source ranges and 13 authenticated earlier owners.
  Both the usual two-byte and the single wide four-byte sequence formats use
  the existing frame, playback and looping controls. Graphics and palettes are
  selected manually; labels use source offsets without invented names. Earlier
  scoped pages and resources remain available. Protected
  `viewer-e360-compact-prior-u5s3iqmf` authenticated 211 prior ranges (27,454
  bytes). `viewer-e360-compact-projection-5hzb8l9e` independently compared all
  reconstructed lists and frames with source bytes. Synthetic
  `viewer-e360-compact-synthetic-md0max7y` passed 24 projection, playback, badge
  and navigation tests. Protected `viewer-e360-compact-real-nxxwgbvm` checked
  all 592 e360 ranges and every decoded list/sequence against source, exercised
  both formats and badges in Chrome, preserved 22 existing scopes and 172
  resource pages, and matched 605 Python/Docker files. The 24-page e360 wiki,
  one new resource page and assets were published with preimages
  `out/ffta-e360-compact-before-cv22fk1j`. The new page is 5,120,949 bytes;
  it does not place these banks in the existing animation page. Graphics/palette
  pairing, live scene selection, original timing and native Windows remain
  unverified. [Physical source](../rom_data_compact_catalog.md#e613-european-physical-compact-obj-region)
- [x] **Counted effect tables and e362 refresh.** That refresh used the
  verified e362-d361 dataset and exposed all 13 counted tables and 1,455 byte
  pairs on a separate, dated Interface page. Browse or play samples, inspect
  their source halfwords and the documented reversed or biased reader values,
  and view a diagnostic interval for each pair. Preview rate and looping remain
  selected when changing tables. The view does not infer scene names or claim
  original timing, clipping or a complete effect. Synthetic
  `viewer-e362-effects-synthetic-bojvtdjx` passed 45 projection, guard, Chrome,
  playback and regression tests. Protected `viewer-e362-effects-real-at6n3xm3`
  checked all 1,455 source samples and six directory slots, exercised Chrome
  navigation/playback, preserved 25 prior scoped payloads apart from revision
  and navigation metadata and 174 resource pages, and matched 610 Python/Docker
  files. Published 29 local files with preimages
  `out/ffta-e362-effects-before-7anahly0`. Native Windows remains deferred.
  [Source and readers](../rom_data_graphics.md#e619-european-counted-effect-table-runtime)
- [x] **Focused background planes.** The Interface navbar now opens the
  initial background-plane preview in its own dated page. The main Interface
  page no longer embeds the plane programs; the focused page carries the
  graphics, palette and transfer inputs needed for the existing composition
  and animation controls. Descriptor, tile-sheet and terrain resource links
  cross pages, while earlier Interface and World bookmarks redirect to the
  selected descriptor. Invalid paired data keeps the independent terrain view.
  For e371-d370, Interface fell from 6,165,115 to 4,906,588 bytes and the
  separately loaded plane page is 5,638,807 bytes. Synthetic
  `viewer-interface-planes-synthetic-sixl07rm` passed 34 navigation, animation,
  partial-source, single-page and regression tests. Protected
  `viewer-interface-planes-real-vudqpzdu` checked 352 source segments, preserved
  all 162 descriptors, compared three complete rendered previews including
  transfers, retained 37 scoped payloads and 184 resource pages, and matched
  1,107 Python/Docker files. Published 40 private files with preimages
  `out/ffta-interface-planes-before-wc_14fve`. Native Windows remains deferred.
  [Background packet sources](../rom_data_maps.md)
- [x] **Screen palette sources and e371 refresh.** The local wiki uses
  verified e371-d370. A dated, separately loaded Interface page displays three
  source-gated screen color tables in 16-color banks. Select a swatch to see
  its BGR555 word, RGB interpretation, high bit, original address and documented
  destination. The first two tables contain 240 colors each and the staged
  table contains 256; their active screen and later changes are not inferred.
  The reader literals and each source span must match before the preview is
  enabled. Synthetic `viewer-e371-palettes-synthetic-v6_3022v` passed 22
  source-guard, browser, bookmark and regression tests. Protected
  `viewer-e371-palettes-real-h70idj7m` compared all 1,484 source bytes and
  736 displayed colors, preserved 36 scoped payloads and 183 resource pages,
  and matched 1,106 Python/Docker files. Published 40 private files with
  preimages `out/ffta-e371-palettes-before-n3r23uii`. Native Windows remains
  deferred. [Source contract](../rom_data_ui_palettes.md#e638-european-b8-screen-palette-runtime)
- [x] **Physical unit sprite sequences and e370 refresh.** The verified
  e370-d369 refresh added a dated **Physical unit sprite sequences** page
  decodes the 4,751 authenticated, nonoverlapping source spans and joins their
  index cells, directory roots, sequence frames, OBJ lists and 4-bit tiles to
  previously extracted dependencies. It exposes 11,596 newly served physical
  cells and 860 linked sequences (817 new), across 228 sprite IDs with these
  cells. Original directory aliases remain separate IDs. Inspect null cells
  and frame operations, select a manual palette, and play from a chosen frame
  with optional looping; sprite changes retain preview settings. Tile assets
  load on demand. Conflicting, missing or unrecognized source dependencies
  disable or limit the preview rather than invent an association. Original
  timing, live actor composition, scheduler and palette selection are not
  simulated. Synthetic `viewer-e370-sprites-synthetic-gbsiqf6s` passed 21
  decoding, alias, playback, lazy-load and regression tests. Protected
  `viewer-e370-sprites-real-e3r27qx4` compared all 154,032 new bytes and
  4,751 spans with their sources, verified 11,596 cells, all 860 displayed
  sequences and every resolved object/tile against the ROM, and resolved all
  2,554 graphics frames. It preserved 35 scoped payloads and 182 resource pages,
  and matched 1,104 Python/Docker files. Published 60 private files with
  preimages `out/ffta-e370-sprites-before-cvsbhhh3`. Native Windows remains
  deferred. [Physical bank contract](../rom_data_unit_obj.md#e636-complete-european-unit-obj-bank-at-runtime)
- [x] **Focused encounter page.** The World navbar has a separate
  **Encounters and unit templates** page. Headers, unit templates and
  coordinate lists load only there; the World page keeps compact references
  needed for item, job and event reverse links. Follow both directions between
  encounter templates and items, metadata, events and terrain. A language
  selector keeps the available source-backed names readable on the encounter
  page; old World bookmarks redirect to the selected encounter and unit.
  Modified sections and the new page have dated badges. For the e369-d368
  dataset, World fell from 3,724,950 to 2,531,805 bytes; the encounter page is
  2,624,454 bytes and is loaded separately. Synthetic
  `viewer-encounter-scope-synthetic-ndzczgjo` passed 39 navigation, partial
  input, legacy bookmark and regression tests. Protected
  `viewer-encounter-scope-real-hvtei8cb` checked all 442 headers, 2,850 unit
  templates, 691 coordinate lists and 695 source segments; preserved 34 scoped
  payloads and all 182 resource pages, with only three page links updated;
  and matched 1,081 Python/Docker files. Published 40 private files with
  preimages `out/ffta-encounter-scope-before-xlxk_mal`. Native Windows remains
  deferred.
  [Encounter source](../rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership)
- [x] **Same-index mission text candidates.** The local wiki
  offers searchable, five-language candidates for 512 mission records. It
  authenticates both European source catalogs and checks their 512 record IDs,
  low-ten-bit values, matching text reader and five complete language tables.
  The association is explicitly tentative: equal indices and recognizable
  examples do not prove a title or live selection for every record. Search
  loads its compact name asset on demand, opens the exact mission and language,
  and links in both directions to the corresponding pointer-text entry. The
  mission panel shares pinned language, command visibility and font settings
  with other text views; modified sections and the added panel have dated
  badges. The index does not embed the game strings; local generated assets
  contain the extracted data. Synthetic `viewer-mission-text-synthetic-9k90t4ex`
  passed 28 projection, navigation, text-settings and regression tests.
  Protected `viewer-mission-text-real-2ctl456a` checked 512 records, 2,560
  localized text entries and 33,657 source segments, preserved 34 scoped
  payloads and 182 resource pages, and matched 1,080 Python/Docker files.
  Published 37 private files with preimages
  `out/ffta-mission-text-before-0sf6ru28`. Native Windows remains deferred.
  [Source contract](../rom_data_gameplay.md#european-mission-record-source-contract)
- [x] **Queued UI maps and e369 refresh.** That refresh used verified
  e369-d368 and added a separately loaded, dated Interface page for five physical
  maps, 624 cells and nine original source paths. Paths link to the relevant map,
  preserving shared and alternate targets. For the live-height map, select one
  through sixteen physically available rows for inspection; this is not a
  bound on the game's live row count. Graphics and palettes remain optional and
  share the on-demand map asset. Synthetic `viewer-queued-ui-synthetic-95fv_p7v`
  passed 40 projection, browser and regression tests, including retained row
  selection and single-page navigation. Protected `viewer-queued-ui-real-hse1rw8b`
  checked 1,284 source bytes, all cells, paths and sixteen row previews, preserved
  33 other scoped payloads and 181 resource pages, and matched 1,080 Python/Docker
  files. Published 37 private files with preimages
  `out/ffta-queued-ui-before-gcm5mb6p`. Scene, palette and live row bounds remain
  unverified; native Windows remains deferred.
  [Source and readers](../rom_data_ui_graphics.md#e633-european-queued-ui-map-runtime)
- [x] **Map-height digits and e368 refresh.** That refresh used verified
  e368-d367 and added a separately loaded, dated Interface page for ten
  map-height digit sources. It joins the nine new sources with the earlier
  hash-checked digit 5 transfer; if the latter is absent or inconsistent, the
  other nine remain usable and digit 5 is marked unavailable. Enter a height
  from 0 to 99, inspect individual source halfwords, and switch between solid
  and grayscale index previews. The renderer interprets each source as a 2×4
  arrangement of 4-bit tiles; the original palette and placement are not
  established. Synthetic `viewer-height-synthetic-9n37njc7` passed 35 source,
  browser and regression tests, including missing dependency and single-page
  behavior. Protected `viewer-height-real-a5ugubd7` checked all 2,560 joined
  glyph bytes and 100 preview values, preserved 32 earlier scoped payloads and
  180 resource pages, and matched 1,078 Python/Docker files. Published 36
  private files with preimages `out/ffta-height-before-htawh0lh`. Native Windows
  remains deferred. [Source and readers](../rom_data_runtime.md#e631-european-map-height-digit-runtime)
- [x] **On-demand map graphics and palettes.** The fixed-width maps, direct
  rectangles and indexed UI pages keep their cells immediately available, while
  a shared local asset supplies optional graphics and palettes when the reader
  selects **Load graphics and palettes**. Missing or invalid assets can be
  retried without losing the selected map; the legacy single-page viewer keeps
  its existing embedded inputs. All three modified sections have dated badges.
  Synthetic `viewer-lazy-maps-synthetic-budbp5gi` passed 29 browser, source and
  regression tests, including a missing-file retry and existing manual pixels.
  Protected `viewer-lazy-maps-real-okbh5lh6` preserved 32 scoped payloads and
  180 resource pages, checked all three pages offline with 117 graphics and
  184 palette choices, and matched 1,076 Python/Docker files. Initial page sizes
  fell from 4,226,622–4,235,929 bytes to 942,799–952,106 bytes; the shared
  2,964,154-byte asset loads only on request. Published 35 private files with
  preimages `out/ffta-lazy-maps-before-xfub0ev3`. Native Windows remains deferred.
- [x] **Indexed UI tilemaps and e367 refresh.** That refresh used verified
  e367-d366 and added a separately loaded, dated Interface page for the three
  language-selected maps and 28 distinct indexed maps. Four ten-slot
  directories link to the physical maps and disclose shared targets; each map
  lists its directory references. Inspect all 855 cells or apply a manually
  selected 16-bit caller addend with wrapping. Scene, graphics, palette and the
  actual caller value are not inferred. Synthetic
  `viewer-indexed-ui-synthetic-s8x4ue7i` passed 26 projection, browser and
  regression tests, including single-page navigation. Protected
  `viewer-indexed-ui-real-8utepglg` checked 1,902 source bytes, all 855 stored
  and adjusted cells and 40 slot links in Chrome, preserved 31 other scoped
  payloads and 179 resource pages, and matched 1,075 Python/Docker files.
  Published 35 private files with preimages `out/ffta-indexed-ui-before-1x7d4m5r`.
  Native Windows remains deferred. [Source and readers](../rom_data_ui_graphics.md#e629-european-indexed-ui-tilemap-runtime)
- [x] **Direct rectangle previews.** A separately loaded, dated Interface page
  exposes all five E627 15×16 cell rectangles at their distinct source
  addresses and the twenty selector preset bytes with the reader's +80 result.
  Preset rows open the corresponding rectangle, whose cells can be inspected
  and manually paired with existing graphics and palettes. The original task
  was verified copying row zero; the shared helper was checked for the other
  rows. Actual scene, graphics, palette and placement are not established.
  Synthetic `viewer-direct-rectangles-synthetic-6bsfrc_5` passed 21 projection,
  browser and regression tests, including single-page navigation. Protected
  `viewer-direct-rectangles-real-7miy5cj1` checked 2,458 source bytes, all
  1,200 cells and 20 presets in Chrome, preserved 30 other scope payloads and
  178 resource pages, and matched 1,073 Python/Docker files. Earlier unit
  `1lqrh6_x` reached the Docker step but failed on a shadowed test variable;
  the test script was corrected. Published 34 private files with preimages
  `out/ffta-direct-rectangles-before-b3pw5aos`. Native Windows remains deferred.
  [Source and readers](../rom_data_graphics.md#e627-european-direct-rectangle-runtime)
- [x] **Event-to-script links and e366 refresh.** That refresh used verified
  e366-d365 and linked authenticated European event rows to their stored script
  keys. Each key lists matching event rows and counts of stored mission
  references, with pagination and direct bookmarks in both directions. The
  key 0x4A live-selector exception and the distinction between stored references
  and execution are explicit. Missing or unauthenticated sources hide the join;
  missing mission authentication leaves event links available without counts.
  Synthetic `viewer-event-script-synthetic-m_86xw5y` passed 29 source-guard,
  browser and regression tests. An earlier unit `a5yfv6kd` exposed an obsolete
  text-page expectation and a same-URL reload test; both were corrected.
  Protected `viewer-event-script-real-wy_mph2l` checked 256 event rows, 512
  mission references and 234 archive keys against the ROM, exercised both
  navigation directions, preserved 30 scoped payloads apart from the join and
  revision/navigation metadata and 178 resource pages, and matched 1,072
  Python/Docker files. Published 33 private files with preimages
  `out/ffta-event-script-before-kdltpz0p`. At that stage the e366 rectangle
  resource was available in the local file inspector.
  Native Windows remains deferred. [Event source](../rom_data_gameplay.md#european-event-record-source-contract) ·
  [Script source](../rom_data_scripts.md#relative-script-archive-and-event-key-source-contract)
- [x] **Localized layouts and e365 refresh.** That refresh used the verified
  e365-d364 dataset and added a separately loaded, dated Interface page for
  four builders, five language IDs, 267 layout rows and three object lists.
  Inspect raw fields and the documented wrapping task positions on a plot,
  view relative untransformed OBJ outlines, and play the 17 stored transition
  pairs at a chosen preview rate with optional looping. Scene identities,
  actual composition and original transition timing are not inferred. Synthetic
  `viewer-e365-layout-synthetic-n97a9ppz` passed 29 projection, browser and
  regression tests. Protected `viewer-e365-layout-real-7rpajuy8` checked 3,718
  source bytes, all 20 layouts and 267 rows, eight objects and 17 transitions,
  exercised Chrome and playback, preserved 29 earlier scoped payloads apart
  from revision/navigation metadata and 177 resource pages, and matched 1,071
  Python/Docker files. It used the verified E625 retry `_2d3h_eq` and passing
  CLI subreceipt after the original staging unit's fixture-limit failure.
  Published 33 private files with preimages `out/ffta-e365-layout-before-1y3ojqz6`.
  Native Windows remains deferred. [Source and readers](../rom_data_ui_graphics.md#e625-european-localized-layout-runtime)
- [x] **Full-screen map and e374 refresh.** A separately loaded, dated
  Interface page displays all 600 authenticated cells of the 30×20 source map.
  Its loader-selected 600-tile graphics bank is joined through the independently
  authenticated literal and loaded only on request; palette choices remain
  manual. Cell source addresses and 32-cell destination stride are explained.
  Reader audit `viewer-e374-reader-audit-mt9ui1p4` confirmed the original
  graphics and map copy sequence. Synthetic `viewer-e374-map-synthetic-_h4apm9c`
  passed 30 source, browser, retry and regression tests. Protected EUR
  `viewer-e374-map-real-5yojkzg3` checked 1,212 source bytes, all 600 cells and
  38,400 grayscale pixels, preserved 41 scoped payloads and 186 resource pages,
  and matched 1,114 Python/Docker files. Published 45 private files with
  preimages `out/ffta-e374-map-before-4bmqu0vq`. Initial test `y065fnf5`
  counted an accessible row header as a map cell; the corrected test passed.
  Screen identity, live composition, ordinary execution and native Windows remain
  unverified. The later initializer-color association is documented below.
  [Source and readers](../rom_data_ui_graphics.md#e644-european-full-screen-map-at-runtime)
- [x] **Full-screen initializer colors.** The dated full-screen page can load
  the authenticated loader graphics with one of the preceding initializer's
  three palette sources. Its four color-mode values include the default alias,
  preserve the original first-color clear and use each cell's stored bank ID;
  manual choices remain available. The shared map-graphics asset still loads on
  request. Static audits `viewer-screen-init-audit-lz4xdapt` and
  `viewer-screen-caller-audit-n8dyno0p` trace the selection and dispatcher.
  Synthetic `viewer-screen-colors-synthetic-rvx4326w` passed 31 source, browser,
  retry and regression tests. Protected EUR `viewer-screen-colors-real-85frh7mv`
  checked three source ranges and all 38,400 pixels in each of four modes;
  preserved 43 scoped payloads and 188 resource pages; and matched 1,116
  Python/Docker files. Published 45 ignored files with verified preimages
  `out/ffta-screen-colors-before-n1x30_iv`. Zero-index transparency and other
  layers are not composited; subsequent palette changes, screen identity,
  ordinary execution and native Windows remain unverified.
  [Source and readers](../rom_data_ui_graphics.md#e644-european-full-screen-map-at-runtime)
- [x] **Fixed-width screen maps and e364 refresh.** That refresh used the
  verified e364-d363 dataset and added a separately loaded, dated Interface
  page for nine bounded source rectangles. Browse their 2,939 cells, inspect
  values and source offsets, and manually choose tile graphics and palettes.
  Two localized callers share one stored rectangle. Graphics/palette pairing,
  scene identity and original composition are not inferred. Synthetic
  `viewer-e364-maps-synthetic-777lcf44` passed 30 projection, browser and
  regression tests. Protected `viewer-e364-maps-real-poa0ee8y` checked 5,918
  source bytes, all cells and their displayed index colors, plus 27 manual
  graphics/palette pixels across nine maps. It preserved 28 prior scoped
  payloads and 176 resource pages and matched 615 Python/Docker files. Two
  earlier attempts (`fnid_m7v`, `b0ghn_b7`) stopped at an incorrect test DOM
  selector; the corrected test passed. Published 32 private files with
  preimages `out/ffta-e364-maps-before-ihnf3t19`. Native Windows remains
  deferred. [Source and readers](../rom_data_ui_graphics.md#e623-european-fixed-width-screen-map-runtime)
- [x] **Grouped selection banks and e363 refresh.** That refresh used the
  verified e363-d362 dataset and added a separately loaded, dated page for four
  banks of 25 groups (500 halfwords) and 160 weighted rows. Browse groups,
  search values, inspect source offsets and field widths, and select the 21
  observed European row ranges. Their weights are not presented as probabilities;
  names and live selections remain unverified. The separately owned first group
  is displayed only when its source checks pass; the remaining data stays usable
  if that dependency is absent. Synthetic `viewer-e363-groups-synthetic-sh_5k9_u`
  passed 17 source, browser and navigation tests. Protected real run
  `viewer-e363-groups-real-y6gej8y4` checked 2,952 source bytes, all 500 pool
  entries and 160 rows, 21 observed groups, Chrome navigation, 27 prior scoped
  payloads and 613 Python/Docker files, then failed a publication bookkeeping
  assertion comparing an absolute path with a relative path. The retained output
  and protected inputs were reconciled in `viewer-e363-groups-finish-k2r2am76`
  PASS; 175 previous resource pages were preserved. Published 31 private files
  with preimages `out/ffta-e363-groups-before-26rtfkov`. Native Windows remains
  deferred. [Source and readers](../rom_data_gameplay.md#e621-european-grouped-selection-runtime)
- [x] **Selection thresholds and weight scans.** Browse the four source
  thresholds and 30 ordinal weights on a separate, dated World and other data
  page. Supply a header count and signed helper result to inspect the threshold
  comparison; independently choose eligible ordinals for each live scan and a
  32-bit rank to inspect the wrapping subtraction and selected ordinal. Zero
  weights, zero-total exit, negative ranks and independent eligibility are
  preserved. The view does not infer a random distribution or ordinal meaning.
  Synthetic `viewer-selection-synthetic-f_rli0ap` passed 61 projection, guard,
  Chrome and regression tests. Protected `viewer-selection-real-qnrazhjx`
  checked 34 source bytes, six count cases and 96 browser scan combinations
  across 2,880 result cells; it preserved 26 earlier scopes apart from resource
  relocation/navigation metadata, retained 174 unchanged resource pages,
  updated one inspector link and matched 611 Python/Docker files. Published 30
  local files with preimages `out/ffta-selection-before-vsxhvd_5`. Native
  Windows remains deferred. [Source contract](../rom_data_gameplay.md#european-selection-byte-table-source-contract)
- [x] **Task-cycle delta playback.** Browse the 48 signed source bytes on a
  separate World and other data page, with a graph and source table. Choose a
  starting phase, unsigned 16-bit value and add/subtract branch; step or play
  the documented bounded sequence, optionally continuing across cycles. The
  calculation wraps at 16 bits and the next phase wraps after 47. The view
  does not infer a coordinate, task lifetime or original timing. It rejects
  inconsistent source bytes and carries a dated section badge. Synthetic
  `viewer-task-cycle-synthetic-r34mgrr2` passed 41 projection, source-guard,
  Chrome, focus, playback and regression tests. Protected
  `viewer-task-cycle-real-a8nvlfpd` checked 48 source bytes and 9,216 arithmetic
  steps, exercised Chrome navigation/playback, preserved 24 prior scopes apart
  from moving the resource entry and adding navigation metadata, retained 173
  resource pages unchanged and updated one inspector link, and matched 608
  Python/Docker files. Published 28 local files with preimages
  `out/ffta-task-cycle-before-05w2semu`. Native Windows remains deferred.
  [Source contract](../rom_data_gameplay.md#conditional-european-task-cycle-delta-source-and-model)
- [x] **Screen-update grid movement.** Join the two 24-row compact source
  grids to their documented byte-curve pointers. A collapsible view plots stored
  X against the controlled reader's signed 16-bit Y calculation, displays the
  source row and delta at each step, follows the curve playback and links to
  the corresponding source table and animation references. The table reuses
  its rows during playback. The view labels step zero as a formula preview and
  does not claim complete sprite, scaling, timing or scene reconstruction. A
  dated update badge marks the changed section. Synthetic
  `viewer-e361-grid-synthetic-z5wgg4cl` passed 38 projection, browser, badge
  and regression tests. Protected `viewer-e361-grid-real-o3yceyn2` compared 48
  rows, their source pointers and 7,248 values with the original source, tested
  Chrome playback and the source link, preserved 24 scoped payloads and 174
  resource pages, and matched 607 Python/Docker files. Published 25 local pages
  with preimages `out/ffta-e361-grid-before-3ns4jbk9`. Native Windows remains
  deferred. [Controlled reader](../rom_data_compact_catalog.md#e616-european-screen-update-table-runtime)
- [x] **Screen effect motion and e376 refresh.** A dated, separate Interface
  page browses 22 scheduled and 64 physical second-sprite rows, 12 motion
  records, seven timed records, five background modes and 264 stored offset,
  scale and scanline samples. A selectable, playable marker illustrates the
  stored endpoints with linear interpolation; curves play their actual stored
  samples. Source offsets, bytes, scanline branch values and the separate blend
  alias are visible. Loop and preview rate survive selection changes. The
  original sprite base, scheduler, interpolation, live second-row domain,
  graphics, composition and scene identity are not established by this view.
  Synthetic `viewer-e376-effect-synthetic-vhybnqra` passed three focused source
  and browser tests; 14 existing cases passed in `claym7qj`, whose new fixtures
  initially omitted an empty manifest list and focused a hidden select.
  Protected `viewer-e376-effect-real-jjzu_1p9` checked all 1,089 bytes and 26
  source spans and all 374 displayed rows, preserved 43 scoped payloads and 188
  resource pages, and matched 1,118 Python/Docker files. Published 47 private
  pages with preimages `out/ffta-e376-effect-before-4166wgmt`. Native Windows
  remains deferred. [Source and readers](../rom_data_graphics.md#e648-european-screen-effect-motion-at-runtime)
- [x] **Screen motion curves and e373 refresh.** A dated Interface page
  displays and plays seven authenticated X/Y motion curves through step 60,
  with 21 source records and 30 position rows. The last step is labelled as a
  source alias only for a selected nonconstant curve; null pointers use 256.
  Missing prior position owners leave their bytes explicitly unavailable. Rate
  and loop settings are retained across controller and record changes.
  Synthetic `viewer-e373-curves-synthetic-dzyeapjr` passed 17 source, Chrome,
  playback and regression tests. Protected EUR
  `viewer-e373-curves-real-qs8buc10` matched 1,450 new source bytes, 427 curve
  values, all 21 records and 30 position rows, and 2,562 browser values. It
  preserved 40 scoped payloads and 185 resource pages and matched 1,112
  Python/Docker files. The local e373-d372 wiki was published with preimages
  `out/ffta-e373-curves-before-qwqm0kjn`. The first browser run
  `964n81vk` passed but displayed an alias label for two constant axes; the
  corrected unit above supersedes it. Preview speed, complete screen effects,
  the second controller scheduler and native Windows are not established.
  [Source evidence](../rom_data_graphics.md#e642-european-screen-curves-at-runtime)
- [x] **European screen-update curves and e361 refresh.** The local wiki now
  uses the verified e361-d360 dataset and has a separate, dated Interface page
  for both screen-update banks. Browse four 16-bit scale curves and thirteen
  8-bit row curves per bank, inspect raw values and source offsets in a graph
  or table, and play from the selected step with optional looping. Step 150 is
  explicitly marked as an alias of the next source value; this is a numerical
  preview, not a reconstruction of a complete screen effect or game timing.
  Synthetic `viewer-e361-curves-synthetic-pvgvkb0m` passed 35 projection, source
  guard, Chrome, badge, navigation and regression tests. Protected
  `viewer-e361-curves-real-43n9ldm9` checked 5,134 source values, exercised
  Chrome navigation and playback, preserved 23 earlier scoped payloads and 173
  resource pages, and matched 607 Python/Docker files. Published 27 local files
  with preimages `out/ffta-e361-curves-before-sp3tsw1w`. Native Windows and
  ordinary callback context remain unverified. [Curve source and reader](../rom_data_compact_catalog.md#e615-european-screen-update-curve-banks)
- [x] **Link witnessed and physical compact sequences.** Navigate in both
  directions between matching physical banks and the witnessed catalog, where
  existing source-table references remain available. Links are emitted only
  when roots, selectors, frame durations, list IDs and referenced object lists
  agree, including sparse list indices; ambiguous or inconsistent sources are
  left unlinked. Each page retains only its own data and a small link index.
  Both modified sections display dated update badges. Synthetic
  `viewer-compact-crosslinks-synthetic-ymnjg7bz` passed 31 projection, Chrome,
  badge, scope and regression tests. Protected
  `viewer-compact-crosslinks-real-j5q5lo5i` compared all 212 linked sequences
  across 63 banks, 1,032 frames and 651 object lists with source; Chrome followed
  a source-table round trip, 23 scoped payloads and 173 prior resource pages
  remained intact, and 605 Docker/Python files matched. Published 24 local pages
  with preimages `out/ffta-compact-crosslinks-before-tlpkw1i6`. This does not
  infer graphics/palette pairings or additional scene meanings; native Windows
  remains deferred. [Physical source](../rom_data_compact_catalog.md#e613-european-physical-compact-obj-region)
- [x] **Physical compact descriptors and observed contexts.** Browse 548
  source-backed 24-byte descriptors with their stored halfwords, filter by bank,
  selector or value, and follow links to the selected physical animation and
  back. The descriptors cover 219 bank/selector pairs across 107 banks.
  Authenticated European sources show nine recorded context references,
  including Red Spring and Blue Screw selectors and the documented Soil
  Evidence, Freezeblink, Stardust and Star Cross observations. These labels
  describe observed contexts, not official names for every sequence or proof
  of the complete animation; unverified sources do not receive them. The new
  descriptor section and changed animation section have dated badges.
  Protected `viewer-compact-descriptors-source-vkcs68mc` reconstructed all 548
  descriptors and checked 13,152 source bytes; synthetic
  `viewer-compact-descriptors-synthetic-1u5n0gxx` passed 27 projection,
  filtering, context, navigation, badge and regression tests. Protected
  `viewer-compact-descriptors-real-w3m9wh2t` compared every descriptor with
  original source, checked a two-way Chrome link, preserved 23 scoped payloads
  and 173 resource pages, and matched 605 Python/Docker files. Published 24
  local pages with preimages `out/ffta-compact-descriptors-before-_jrm_zst`. Other caller paths, full scene meanings,
  palettes and native Windows remain unverified.
  [Descriptor source](../rom_data_compact_catalog.md#e613-european-physical-compact-obj-region),
  [observed Red Spring and Blue Screw](../rom_data_compact_catalog.md#european-6c1e78-ordinary-route-candidates),
  [Freezeblink](../rom_data_compact_catalog.md#european-freezeblink-ordinary-route),
  [Stardust](../rom_data_compact_catalog.md#european-stardust-ordinary-route)
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
- [x] **Compact-cell bank animations.** Decode individually authenticated
  compact records into byte and halfword cell frames, retaining source offsets,
  run controls, placement bytes, sequence flags and second bytes. Browse each
  frame or preview stored sequence order from a selected entry, with adjustable
  rate, looping and persistent controls across records. Preview cell indices or
  select tile graphics and palettes manually. Invalid records are reported and
  isolated; unavailable sequence frames cannot play. These previews do not
  establish original timing, transparency, palette pairing or scene composition.
  Synthetic Chrome unit `rom-data-viewer-compact-cell-bank-synthetic-bqw7ty2z`
  passed 47 projection, playback, isolation and regression checks, including
  focus and manual palette selection. Protected EUR e337/d336 unit
  `rom-data-viewer-compact-cell-real-9dcxp6zh` authenticated all 205 source
  ranges, independently decoded 91 records, 493 frames, 563 sequence entries
  and 35,124 cells, and checked every frame and entry in Chrome. Eleven prior
  scope payloads and 145 resource pages were preserved; 18 Docker/Python files
  matched. Twelve ignored wiki pages were published. Initial synthetic unit
  `d121w53o` exposed a fixture count error, corrected before the successful
  run; the first real launch used a temporary filesystem below the admission
  floor, so the accepted unit ran under the project target. Windows is deferred.
  [Frame source](../rom_data_graphics.md#e492-european-compact-cell-frame-decoder)
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
- [x] **Separate battle sprite catalogs.** Normal and alternate battle actors
  have individual navbar pages and resource links, each embedding only its own
  catalog. Switching catalogs preserves the selected sprite where available,
  preview playback, frame, palette, loop, rate and job-reference controls.
  Existing alternate sprite URLs and saved catalog selections redirect to the
  new page. Protected e347 `9ldjwfz9` preserved 20 other scope payloads and
  156 resource entries, exercised navigation and settings in Chrome, and matched
  28 Python/Docker files on a synthetic dataset. The former 18,447,874-byte page
  became 14,028,535 bytes for normal and 5,216,729 for alternate. Published 23
  ignored pages with preimages `out/ffta-battle-split-before-k0hxyj5g`. Synthetic
  `1xee6_cr` passed 19/21 cases; the two corrected cases passed in `cm2uwz_2`,
  and 13 related checks passed in `jz0agsys`. Windows remains deferred.
- [x] **On-demand battle and script sprite tiles.** Split normal, alternate
  and script-actor pages retain sprite bindings, frames, object metadata,
  palettes and references while placing tile arrays in content-addressed local
  assets of up to 64 blocks. The current pose loads its needed block; playing an
  animation first loads the blocks used by its sequence. Stop, changed selection,
  frame, palette, loop and cross-page settings remain usable while loading.
  Missing or invalid assets report an error and offer retry. Single-page output
  and opcode-39 pose catalogs continue to use their existing inline data.
  Synthetic unit `viewer-lazy-sprite-synthetic-bjdgcvk4` passed 26 checks,
  including selection, animation cancellation, retry, job links, palettes,
  script actors and PCM regressions. Protected e354 unit
  `viewer-lazy-sprite-real-h8dfovay` authenticated seven catalogs and 10,366
  source ranges, reconstructed all 7,318 catalog blocks, preserved unrelated
  payloads and lazy assets, and matched 147 Python/Docker files. Focused unit
  `viewer-lazy-sprite-pixels-agydzq3r` compared 46,080 RGBA bytes across two
  real poses exactly. The normal, alternate and script pages fell from
  14,084,183 / 5,272,377 / 13,921,622 bytes to 6,090,724 / 2,732,116 /
  6,953,795 bytes. Published 23 ignored pages and 118 assets with preimages
  `out/ffta-lazy-sprite-before-bpj8fef5`. Live composition, other sprite
  families and native Windows remain unverified or outside this change.
- [x] **Battle sprite job references.** The normal and alternate battle sprite
  catalogs show a collapsible, filterable and paginated list of the direct job
  metadata rows that reference the selected sprite. Verified localized names and
  the selected language accompany links to the exact metadata row on the world
  page. Language and preview settings survive catalog changes. Missing or
  unauthenticated name sources show IDs only. Shared sprite IDs do not identify
  a live actor, job, palette or scene. Synthetic `5hofjxci` passed 20 checks.
  Protected EUR e347 `owdfkcg_` authenticated six source files and five ROM
  ranges, checked all 210 stored references, exercised shared IDs and links in
  Chrome, and matched 27 Python/Docker files on a synthetic dataset. Two ignored
  local pages were published with preimages `out/ffta-battle-jobs-before-8iywf1en`.
  Windows remains deferred.
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
- [x] **Opcode-39 pose sources.** Browse and play the registry and special-unit
  source sequences by physical sequence and index-cell offsets. Frames join
  only compatible extracted object and pixel ranges; missing or conflicting
  inputs remain identified and are never replaced with guessed graphics.
  Existing prior-owner sequences referenced by new cells are included when
  their sources agree. The palette and preview rate remain manual; source
  relationships do not establish live actor, scene, facing or task behavior.
  Synthetic unit `_li_0wt2` passed 42 checks, including partial, conflicting,
  earlier-owner and missing-sequence cases, pixels and playback controls.
  Protected EUR unit `6nb4p1y3` compared all 693 cells and sequences (519 new,
  174 earlier), 879 frames and 862 graphic joins against authenticated files;
  checked two Chrome pixels; preserved 143 resource pages and matched 17
  Docker/Python files. Two resource links moved to the script-actors topic and
  fourteen ignored local wiki files were published. Initial unit `b7hg8dzc`
  exposed a missing optional objects field; `wtkudkvp` expected all 693 targets
  to be new. Corrected checks pass. Windows is deferred.
  [Source evidence](../rom_data_unit_obj.md#low-selector-special-unit-pose-projection)
- [x] **Opcode-39 source index cells.** A collapsible, paginated table shows
  all recorded physical index offsets, including nulls, and filters by target
  kind or decimal/hex offset. Sequence targets open the corresponding preview;
  unavailable sequences remain identified. Filters persist across catalogs;
  the table does not imply live actor or scene use. Synthetic unit `pyo47ub8`
  passed 42/43 checks; its test incorrectly assumed the separate script page
  also contained the equipment catalog. The corrected split/single-page check
  passed in `alynb5hx`. Protected EUR unit `3i5uayb1` matched all 7,924 cells
  (7,231 null and 693 sequence targets), exercised both catalogs in Chrome,
  preserved ten other scope payloads and 145 resource pages, and matched 17
  Docker/Python files. Eleven ignored local wiki pages were published; Windows
  remains deferred. [Source evidence](../rom_data_unit_obj.md#low-selector-special-unit-pose-projection)
- [x] **Maps.** Plot the tile coordinates from `world-nodes.json`. Let the
  reader select a node to see its ID, coordinates and recorded neighbors.
  EUR data and Linux Chrome selection passed in unit `rom-data-viewer-map-povy0tl_`.
  [World map data](../rom_data_maps.md)
- [x] **Observed world location names.** For authenticated EUR world-node
  and pointer-text files, a collapsible list links six recorded region/node
  placements to their localized text entries. Select a node and open its exact
  source string on the text page. These associations come from one recorded
  placement and can differ during play; other profiles or changed source hashes
  do not acquire the names. The initial Giza association had this evidence: Synthetic
  unit `22pyjgbt` passed 38 of 39 checks; its link-click test targeted a wrapped
  text gap and passed after correction in `zg52_9wm`. The tests also cover
  unknown glyph bitmaps, partial languages and both split/single-page links.
  Protected EUR unit `hl7y6830` matched all five name token sequences to the
  source, verified browser navigation, preserved ten existing scope payloads
  and 145 resource pages, and matched 15 Docker/Python files. Eleven ignored
  wiki pages were published. Earlier unit `iet5gnim` used an incorrect German
  fallback expectation; its candidate and Docker outputs were reused. The four
  additional observed associations use regions 1, 2, 6 and 11. Synthetic unit
  `rom-data-viewer-world-names-expanded-synthetic-_2mq6zty` passed 41 checks.
  Protected EUR unit `rom-data-viewer-world-names-expanded-real-g07anua6`
  matched all five observed node/region pairs to retained placement evidence
  and all 25 localized token sequences to the extracted text, checked names,
  node selection and source navigation in Chrome, preserved 11 scope payloads
  and 145 resource pages, and matched 16 Docker/Python files. The ignored world
  page was published. At that checkpoint, other names and placements remained
  open; Windows was deferred. [Placement evidence](../rom_data_maps.md#world-selector-and-placed-region-coordinates)
- [x] **World region name references.** For the authenticated EUR text catalog,
  a separate collapsible directory filters 31 region IDs and their localized
  name references, links directly to each source string, and connects six
  recorded region placements to the map. The region-list builder conditionally
  calls reader `080165e8` with `718 + region_id` for IDs 0–30; this does not
  prove permanent or complete placement. The sixth association (region 16,
  node 2) comes from the existing conditional placement snapshot. Missing
  languages remain explicitly unavailable, and the directory does not require
  world-node data. Synthetic unit `t1s_3vy8` passed 44 checks. Protected EUR
  e339/d338 unit `mo2faka0` authenticated both source windows, checked all 155
  localized token sequences and six recorded node joins, exercised filtering
  and navigation in Chrome, preserved the other scope data and 145 resource
  pages, and matched 16 Docker/Python files. Twelve ignored wiki pages were
  published. Standalone test discovery initially missed the scripts module;
  the import was corrected and unit `yq73x3mi` passed all three focused checks.
  Native Windows and other placements remain unverified.
  [Source and placement evidence](../rom_data_maps.md#world-selector-and-placed-region-coordinates)
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
- [x] **Encounter template job references.** Resolve a template's stored job
  selector against authenticated European metadata and show its localized name
  where the original name reader is available. A zero selector follows the
  documented descriptor fallback only when its source is verified. Each job
  opens its metadata row and available battle sprites; the reverse template
  list is paginated and opens the selected encounter and unit. Unverified or
  missing inputs leave associations unavailable; routed names and live overrides
  are not inferred. Synthetic unit `viewer-encounter-jobs-synthetic-vx2gge9m`
  passed 51 checks including language changes, fallbacks and reverse navigation.
  Protected e345/d344 unit `viewer-encounter-jobs-real-5cpqf0qy` reconstructed
  136,800 source bytes and verified 2,850 references to 101 metadata IDs, the
  header associations and cross-page sprite navigation; 27 Python/Docker files
  matched. Sixteen ignored pages were published with backup
  `out/ffta-encounter-jobs-before-20j7vgzj`; Windows remains deferred.
  [Template source](../rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership)
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
- [x] **World records and mission references.** Browse all fields in each
  physical 38-byte record and inspect its 31 threshold bytes as numbers and
  0–255 bars. In the European profile, each mission links to the world record
  selected by its documented packed index; the reverse list can be filtered
  and paged. Missing, invalid and non-European inputs remain independent.
  These source references do not identify map locations or establish live use.
  Synthetic unit `world-records-synthetic-zggmm8of` passed 47 projection,
  Chrome and regression checks. Protected EUR unit
  `world-records-recheck-mq38y35o` reconstructed 31 rows and all 961 threshold
  bytes from the authenticated ROM, verified 512 packed mission links in both
  directions in Chrome, and matched 20 Python/Docker files. Initial unit
  `world-records-real-swi3tfcl` stopped at a test-only DOM iteration error;
  the corrected unit reused its candidate and Docker output. Fifteen ignored
  local pages were published with backup `out/ffta-world38-before-ktc6iktb`.
  Native Windows remains deferred.
  [Source contract](../rom_data_gameplay.md#observed-european-world-records-with-38-byte-stride)
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
- [x] **Timed sound sources.** Browse the two observed schedules and all 681
  later source fragments by kind and offset, with raw ticks, channels, opaque
  bytes and sound words. For nonzero words, the low 12 bits are shown as a
  candidate directory ID and high bits remain visible as flags. Compatible IDs
  link to the existing approximate player and can be auditioned individually.
  A separate schedule audition starts unambiguous candidates at their stored
  ticks, including simultaneous starts. It exposes a schedule tick rate, a
  per-sound duration, playback progress and stop control; the song player's
  voice, tempo, track controls and mutes also apply. It reports skipped or
  ambiguous IDs and bounds work to 64 starts and 180 seconds. Live scheduler
  state, original channel behavior, flags and exact timing are not reproduced.
  Older and partially invalid optional groups remain independently available.
  Synthetic unit `qfvhn_5h` passed 38/39 tests; its invalid-bank fixture shared a
  row with the observed schedule. The corrected partial-data test passed
  `i1crr9m5`. Protected EUR unit `wpjg4odq` reconstructed 685 authenticated source
  spans, matched 681 fragments and 569 rows, checked browser filtering and song
  navigation, preserved ten scope payloads and 145 resource pages, and matched
  16 Docker/Python files. Eleven ignored wiki pages were published. Audible
  physical output and Windows remain unverified. Synthetic Chrome unit
  `viewer-timed-mix-synthetic-pqk1y3w0` passed 15 tests, including simultaneous
  starts, stored-tick delays, both mute types, stop, focus and bounded work.
  Protected e343 unit `viewer-timed-mix-real-xythz3co` matched all 683 source
  groups, exercised both observed schedules (11 and 15 scheduled notes),
  preserved all 16 page payloads and matched 21 Python/Docker files. The ignored
  local wiki was updated with guarded preimages; physical audio remains untested.
  [Source contract](../rom_data_audio.md#e539-european-timed-sound-census)
- [x] **Song-to-timed-source references.** A selected song exposes a
  collapsible, paginated list of nonzero sound words whose low 12 bits match
  its directory ID. Entries preserve each source group, offset, tick, channel,
  slot, raw word and high flags; duplicate IDs and source groups remain
  distinguishable. These are candidate references, not proof of live use.
  Links reveal and highlight the exact source row and slot, including when a
  copied bookmark is reopened. Synthetic Chrome unit `gae1ekm7` passed 11 of
  13 cases; its new bookmark wait returned an element instead of a boolean and
  a timed-mix case timed out during Chrome startup. The corrected bookmark
  test passed `mkb33kr3`; the timed-mix case passed `dbdxtwd5`.
  Protected e347 unit `5wm8kf2h` checked 1,142 source ranges, 569 rows and
  570 candidate words for 284 IDs, exercised Chrome navigation and bookmarks,
  preserved the 21 scope payloads and matched 26 Python/Docker files. The 21
  ignored local pages were updated with guarded preimages in
  `out/ffta-audio-references-before-evrfn7k2`. Original scheduler behavior,
  exact audio fidelity, physical output and Windows remain unverified.
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
  scheduling falls behind. Exact game timing, original modulation, CGB mixing and
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
  reproduce exact driver timing, full modulation behavior or effects. Synthetic unit
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
- [x] **Physical rhythm-tone audition.** The authenticated European rhythm
  root exposes 128 physically present key-indexed children in the instrument
  view. Its 127 direct PCM/CGB children can be auditioned using each stored
  pitch key; the one nested map child is marked unavailable, matching the
  original reader. The same lookup is used by sequence, timed audition and WAV
  paths when a source selects it; CGB mute remains effective. Source domains
  describe physical candidates, not established ordinary key use. Original
  reader audit `viewer-rhythm-reader-audit-8vnzhg5i` checked 44 instruction
  decodings. Synthetic `viewer-rhythm-synthetic-jmfuhf51` passed 27 source,
  Chrome, PCM/CGB, sequence, timed, WAV and regression tests. Protected EUR
  `viewer-rhythm-real-kb9j9o23` checked 57,131 physical bytes and all 128 child
  bindings, exercised PCM and muted CGB playback, preserved 41 scoped payloads
  and 186 resource pages, and matched 1,112 Python/Docker files. Published 43
  private wiki files with preimages `out/ffta-rhythm-before-x9sunwx4`. Rhythm
  pan, complete channel/mixer behavior, ordinary reachability and native Windows
  remain unverified. [Source contract](../rom_data_audio.md#e611-european-physical-tone-and-direct-wave-banks)
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
- [x] **Export the selected song mix.** Render the selected song or individual
  track as a 32 kHz, 16-bit stereo WAV using the current bounded preview,
  estimated or manual timing, instruments and sine fallbacks, optional PCM
  envelope and track controls, and the current CGB and individual-track mutes.
  The settings and file name are captured when export starts. The output ends
  at the last scheduled note and reports unavailable rendering or automation
  limits; it does not reproduce unmodeled driver behavior. Synthetic Chrome
  unit `rom-data-viewer-sequence-wav-synthetic-zwqbzr4d` passed 42 audio and
  regression checks, including real offline stereo output, pan and mute, and
  comparison with live scheduling. Unit `rom-data-viewer-sequence-wav-errors-9vk7mg2l`
  passed three focused checks including unavailable-renderer recovery. Protected
  EUR e336/d335 unit `rom-data-viewer-sequence-wav-real-rb20l36j` exported a
  two-second song-312 preview with eight resolved PCM notes, decoded its WAV
  independently, verified track-mute silence, preserved ten scope payloads
  and 145 resource pages, and matched 16 Docker/Python fixture outputs. Eleven
  ignored wiki pages were published. Physical audio, exact game mixer behavior
  and Windows remain unverified.
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
  tone word, with a bounded release and mix scaling. CGB square envelopes use
  the separate option below; wave and sine fallbacks remain generic. Exact
  tempo, modulation, effects and mixer behavior are not reproduced. Synthetic unit
  `rom-data-viewer-audio-fidelity-synthetic-kleo1rm5` checked extension wrapping,
  600 integer pitch calculations, envelope scheduling and prior controls.
  Protected EUR unit `rom-data-viewer-audio-fidelity-eur-cv4woo4e` checked all
  180 PCM keys, 12 bases and 429 tone words, observed 1,006 stored extensions,
  and verified an extended PCM note's pitch and envelope with live Web Audio in
  Linux Chrome. Physical audio output remains unverified.
- [x] **Frame-stepped PCM envelope.** The default PCM audition holds each
  extracted attack, decay, sustain and release level for a nominal mixer frame.
  A note-off takes effect at the next frame update; pseudo-echo and live channel
  changes are not modeled. The setting applies to live songs, timed-source
  auditions and WAV exports, including late starts. A separate checkbox restores
  the earlier smooth interpolation and remains selected across songs.
  Synthetic Chrome unit `0jzed1v3` passed 11 audio and regression tests,
  including manual and frame tempo, both envelope modes, timed playback and
  WAV export. Protected e347 unit `bfzbdt8b` authenticated 736 source spans and
  all 429 tone words, including 416 PCM tones with 58 distinct envelopes. It
  compared 20,880 frame levels in 232 cases against a no-echo model of the
  project's HLE mixer, checked actual offline Web Audio samples and a real PCM
  note's browser scheduling, and matched 26 Python/Docker files. The ignored
  local audio page was published with preimage and evidence at
  `out/ffta-pcm-envelope-before-cbn5yxdz`. The original mixer's gain, channel
  allocation, effects, live echo, physical output and Windows remain unverified.
- [x] **CGB square envelope preview.** Song, timed-source and WAV previews
  optionally apply extracted square-channel attack, decay, sustain and release
  with a 64 Hz approximation and a normalized 15-level peak. The checkbox is
  independent of PCM envelopes and CGB mute, defaults on and survives song
  changes and browser reloads. Missing tone words keep the generic fallback;
  CGB wave and standalone instrument envelopes remain generic. Synthetic unit
  `viewer-cgb-envelope-synthetic-8h613hd5` passed 14 browser/regression tests,
  including late starts, the timed player, persisted settings, an actual WAV,
  CGB mute and both PCM envelope modes. An earlier fixture without a PCM wave
  failed initialization; the corrected fixture passed. Protected e354 unit
  `viewer-cgb-envelope-real-x_eltxpp` authenticated 2,693 source spans, 16
  driver instruction halfwords and all four produced square-tone words, checked
  two real tracks in Chrome, preserved 22 topic payloads and lazy assets, and
  matched 28 Python/Docker outputs. The 23 ignored pages were published with
  verified preimages under `out/ffta-cgb-envelope-before-45qx2r6i`. Hardware
  clock phase, changing gain, allocation, effects, physical output and native
  Windows remain approximate or unverified.
- [x] **Estimated song tempo.** A checkbox follows decoded tempo commands from
  all song tracks, including when auditioning just one track; disabling it uses
  the manual tick rate throughout. In the continuous estimate, the manual rate
  is also the fallback before the first tempo command or when none is found.
  Tempo changes affect note starts and ends, PCM envelope timing and the
  seconds-based note timeline. A zero tempo leaves active note gates open until
  the preview ends; the frame model first consumes any already accumulated
  ticks. The tempo pass is
  bounded and reports incomplete track scans. The rate follows the continuous
  average implied by a [generic M4A command and accumulator reference](https://github.com/pret/pokeemerald/blob/master/src/m4a_1.s#L901-L910);
  the continuous estimate does not model frame quantization, live tempo
  controls or exact FFTA-driver behavior. Synthetic Chrome units `rom-data-viewer-sequence-tempo-synthetic-0bksevth`
  and `rom-data-viewer-sequence-tempo-timeline-lfnlvmos` checked cross-track
  changes, same-tick ordering, zero tempo, fallback and playhead alignment.
  Protected EUR unit `rom-data-viewer-sequence-tempo-eur-5k5nafnt` counted 1,150
  source commands and checked a real cross-track note in automatic and manual
  modes with live Web Audio in Chrome. Physical audio remains unverified.
- [x] **Frame-by-frame song tempo.** By default, following song tempo uses the
  European driver's initial 150-unit increment, 256/256 multiplier and zero
  accumulator. At each nominal GBA frame, the model adds the increment and
  processes one tick for every 150 accumulated units. A tempo command changes
  the increment on the next frame, while accumulated ticks in the current
  frame still execute. The same timeline drives live auditions, timed-source
  auditions, note positions and WAV exports. The frame option remains selected
  across songs and tracks; disabling it restores the continuous estimate.
  Synthetic unit `viewer-frame-tempo-synthetic-q1i8siih` passed nine Chrome
  cases covering the two timing modes, playback, WAV, timed sources, mutes and
  source links. Protected EUR command `jrgtis9i` authenticated 808 source spans,
  projected 1,610 tracks and 105,253 instructions, and compared 10,604 ticks in
  88 scenarios, including 24 real song headers, against 135,025 executed Thumb
  instructions from the original initializer, command and accumulator snippets.
  Its command passed but cleanup failed because the temporary root lacked Git;
  protected recheck `j4qhcysn` passed after reusing the candidate and 26 equal
  Python/Docker fixture files. The local audio page was published with a guarded
  preimage and evidence at `out/ffta-frame-tempo-before-xbsyxes0`.
  This models the default multiplier and nominal frame clock, not live changes,
  the full original mixer, effects or exact audible playback. Physical output
  and native Windows remain unverified.
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
- [x] **Track modulation.** The sequence player exposes speed, delay, depth
  and target commands and can apply their bounded tick-based triangle to pitch,
  volume or stereo position during live playback, WAV export and timed sound
  auditions. The toggle persists across songs and, when available, browser
  storage; disabling track controls also disables modulation. The model follows
  the original EUR byte arithmetic, delay and note-reset structure, but assumes
  each resolved preview note is allocated. Browser pitch, volume and pan remain
  approximations of the original mixer. A per-track tick limit is reported.
  Synthetic `viewer-lfo-synthetic-89w8n4fh` passed ten browser/regression tests,
  including 500 independent tick traces and live/WAV/timed scheduling. Protected
  e354 `viewer-lfo-real-x3ovsy1g` authenticated 2,693 source ranges, four
  command targets and 20 original instruction halfwords, counted 1,066 modulation
  commands, checked two real tracks in Chrome, preserved 22 topic payloads and
  matched 28 Python/Docker files. Published 23 ignored pages with preimages
  `out/ffta-lfo-before-f3wpyaxr`. Voice allocation, exact mixing, physical
  output and native Windows remain unverified.
  [Sequence sources](../rom_data_audio.md#producer-bounded-sequence-instructions-and-driver-tables)
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
- [x] **CGB noise-code reference.** The audio page browses all 60 stored
  driver noise codes and displays their source offset, key range, register,
  LFSR width and shift rate. A one-second synthesized preview and WAV use
  the project's area-averaged noise-generator model; optional repeat restarts
  that preview. The CGB mute stays synchronized with the instrument and song
  controls. Driver codes are not mapped to songs or instruments: the current
  EUR tone catalog contains no recognized type-4 tone. Synthetic unit
  `94zs89xc` passed 40 of 41 checks; the remaining test had an invalid DOM
  selector, corrected in `_mfk8r2v` (three focused checks PASS). Protected EUR
  e340 unit `kc7uabxu` matched all 60 codes to the authenticated ROM range,
  checked real Chrome Web Audio, preserved 13 other topic payloads, and
  matched 19 Python/Docker files. Fifteen ignored wiki pages were published.
  This is an audition of generator settings, not original song playback; game
  envelopes, channel scheduling, mixer effects and native Windows are unverified.
  [Driver table evidence](../rom_data_audio.md#original-pitch-rate-noise-and-volume-lookup-tables)
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
- [x] **CRC32 table and e375 refresh.** A dated Checksums page browses all
  256 authenticated lookup words and calculates the same reflected CRC32 over
  user-supplied UTF-8, hexadecimal bytes or a locally selected file. File reads
  use bounded chunks and can be cancelled; no file leaves the browser. A seed
  can be supplied. The tool does not apply save-header transformations or prove
  save validity. Synthetic `viewer-e375-crc-synthetic-ysafq76z` passed 18
  projection, Chrome, chunking, cancellation and regression tests. Protected
  EUR `viewer-e375-crc-real-8jup3fg0` checked all 1,032 source bytes, 256 table
  words and 256 independent browser vectors, plus a 200,000-byte seeded file;
  it preserved 42 scoped payloads and 187 resource pages and matched 1,116
  Python/Docker files. Published 46 private files with preimages
  `out/ffta-e375-crc-before-ywo3d9ed`. Ordinary save behavior and native Windows
  remain unverified. [Reader evidence](../rom_data_runtime.md#e646-european-save-crc32-table-at-runtime)
- [x] **Search extracted file contents.** Search local files for text or hex
  bytes with bounded reads, highlighting and cancellation. The real-time
  checks above supersede the removed initial prototype whose browser test did
  not complete in `rom-data-viewer-search-t2dkxkfs`.
- [ ] **Further previews.** Identify other extracted graphical, text and audio
  resources that can be rendered or played from their documented formats.
