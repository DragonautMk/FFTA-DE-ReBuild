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

Both commands print the generated page path. By default the page is
`rom_data_viewer_generated.html` in the project root and is ignored by Git.
An optional second argument chooses an output directory containing `index.html`.
Docker downloads the Python
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
output after adding the script sprite catalogs. Both generators use the template, including the suspended
large-JSON prototype; the local root page excludes that prototype.

Open the resulting HTML file locally. The output contains private extracted
data. The contents sidebar links to the views available for that dataset;
each section can be opened or closed independently, and the sidebar can open or
close all sections. Resource rows link directly to their dedicated views when
available. On narrow screens the sidebar and resource rows stack vertically.
Linux Chrome units `rom-data-viewer-wiki-legs66p4` and
`rom-data-viewer-wiki-eur-s3qt2yel` checked navigation and section controls,
including the compact layout, EUR text selection, palette and audio controls.

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
- [x] **Background tile sheets.** View decoded `bg-main-graphics.json` words
  as 4-bit 8×8 tiles, choosing a descriptor, base palette bank or grayscale,
  and page. The view labels destination tile indices and does not invent a
  missing destination prefix or assemble a scene. Synthetic unit
  `rom-data-viewer-bgtiles-r4lnzrf7` checked nibble order, RGB colors, aliases,
  paging and invalid data. Protected EUR unit `rom-data-viewer-bgtiles-eur-7pw4d56v`
  compared all 162 descriptor links and 50 graphic word arrays and checked an
  independently calculated palette pixel and the last selection in Chrome.
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
- [x] **Items.** Display `items.json`, labeling unclassified fields by offset
  and preserving signed values. The EUR records and a synthetic negative value
  passed in Linux Chrome unit `rom-data-viewer-items-ycfow_31`.
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
- [x] **Script text archives.** Browse the physical banks and programs in
  `script-text.json` by language and draw their referenced glyphs from
  `font-bank.json`. Original command and operand bytes are labeled inline;
  dynamic substitutions and formatting are not executed. A missing font glyph
  is labeled explicitly. Synthetic unit `rom-data-viewer-script-text-gcw9ab5e`
  checked selectors, a glyph pixel, command bytes and missing glyphs. EUR unit
  `rom-data-viewer-script-text-eur-0liwj5t4` checked five languages, all 385
  bank counts, 1,155 sampled programs, all 147 referenced font glyphs and the
  final program selection in Linux Chrome. The dataset remained protected.
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
- [x] **Sampled world roads.** Overlay uniquely linked roads from
  `world-roads.json` on the node plot using each sample as a pixel offset from
  the lower-index node. Select a road or its node link to inspect and animate
  stored samples at a preview rate of 10 samples per second; no endpoint is
  added and this is not game timing. Synthetic unit
  `rom-data-viewer-roads-io3__c78` checked short endpoints, reverse node links,
  sample selection and animation. Protected EUR unit
  `rom-data-viewer-roads-eur-h9_e9pfp` compared all 45 road sample arrays and
  node pairs and checked a short endpoint and marker in Chrome.
- [x] **Audio.** Draw one waveform from the signed PCM values in
  `audio-tones.json`. Show its length and loop start alongside the source entry.
  EUR sample values and Linux Chrome canvas passed in unit `rom-data-viewer-audio-6k9qlyf2`.
  [Audio format](../rom_data_audio.md)
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
- [x] **CGB wave instruments in sequence audition.** Resolve supported CGB
  wave tones through the same program and key maps, and play their extracted
  32-step waveforms in a loop. Pitch is an equal-tempered approximation; CGB
  square channels, fixed-frequency PCM and unknown tones use sine fallbacks.
  Synthetic unit `rom-data-viewer-cgbvoice-e6iutnqa` checked mapping, waveform
  cache separation, pitch, loops and fallbacks. Protected EUR unit
  `rom-data-viewer-cgbvoice-eur-9_1pifpq` compared both source waves and checked
  a mapped sequence note, Web Audio start and stop in Linux Chrome. Physical
  audio and original-driver timing, tuning, envelopes and effects remain
  unverified.
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
