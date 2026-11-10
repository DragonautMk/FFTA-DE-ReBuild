<a id="todo--estrazione-rom-dati-decodificati-e-mod"></a>

# TODO — ROM extraction, decoded data and mods

Request updated October 22, 2026. **FEAT-04-ROM-DATA** is in progress;
inventory and initial contract are defined. Runtime migrations remain open.
Scope: desktop workspace, Windows as the primary validation platform; Linux as
an additional check. `AndroidPort/` requires separate work.

The [validation recovery checklist](ROM_DATA_VALIDATION_RECOVERY_TODO.md) tracks
skipped current-revision reruns, unproved scene/lifecycle criteria, recovered
storage failures and explicitly deferred platform checks. Use it to select a
bounded recovery without reopening accepted evidence.

The [Godot frontend](GODOT_FRONTEND_TODO.md) is only an assessment with its TODO
deferred until agreed extraction is complete. It is not a new execution step
of this TODO and must not be developed during data migration.

The [debug tools TODO](DEBUG_TOOLS_TODO.md) collects improvements for user and
assistant: case replay, read audits, divergence comparison and ROM coverage
links. It is a checklist selected as needed, not a new parallel campaign
mandatory for extraction.

<a id="obiettivo-e-ordine-di-lavoro"></a>

## Objective and work order

The main objective is to unpack the selected ROM into resource files, decode
them into usable formats and have the runtime read that data. The dataset is
prepared on first startup; later startups reuse the valid version. Mod support
and broader modifications follow.

Updated user priority: for now **do not measure performance regressions**;
optimization comes later. Do not start benchmarks, profiles or startup/RSS
campaigns, or compress or alter formats to satisfy old space budgets. Existing
measurements remain historical; they do not block functional work.
Updated regional priority: **focus on the European version**; new USA checks
remain open. Keep completed USA evidence as historical without repeating it
during this EUR phase.
Updated platform priority: **reduce tests and run them only on Wine**; mark new
Linux checks as pending. Completed Linux checks remain historical without
repeating them for new integrations. Equivalence evidence remains necessary,
including guest clock, actual file use and native/save/Load compatibility.

Order: **inventory → extraction and decoding → startup preparation → direct
consumption → verification → mods → extended changes**. Creating files the game
does not use does not satisfy the objective. Reconstructing a ROM from extracted
files alone may help comparison, but does not complete reading decoded data.

Files can be loaded at startup or before use, with an in-memory cache: “reading
from files” does not require disk access per pixel, audio sample or instruction.
Speed improvements remain to be measured.

<a id="punto-di-partenza-verificato-e-vincoli"></a>

## Verified starting point and constraints

- [play.rs](../../crates/recomp/src/play.rs) already reads the entire ROM with
  `std::fs::read` and passes it to the machine. In
  [mem.rs](../../crates/gba-core/src/mem.rs), `MemMap.rom` is a `Vec<u8>`;
  normal guest reads access that buffer in RAM.
- A ROM contains code, tables, pointers and resources that can be compressed or
  shared. It is not an archive with a filesystem to extract generically: formats
  and boundaries must be identified for each resource family.
- [ffta_rom.rs](../../crates/gba-core/src/ffta_rom.rs) recognizes EUR/USA
  profiles by exact hashes. [ensure_native](../../crates/recomp/src/main.rs) ties
  the translation cache to the image and other compilation parameters. Changing
  a declared hash alone does not make new data compatible.
- [Machine states](../../crates/gba-core/src/save_state.rs) and
  [desktop slots](../../crates/recomp/src/save_state.rs) use ROM identity. The
  new dataset must remain consistent with saves, load and Undo.
- Guest executable code and structures not yet migrated may require the original
  image. Completely removing the `.gba` dependency is a separate criterion to
  assess after inventorying those dependencies; it is not an automatic result of
  resource extraction.

<a id="romdata-01--inventario-e-contratto-dei-dati"></a>

## ROMDATA-01 — Data inventory and contract

**Dependencies:** none. **Priority:** mandatory first phase.

- [x] Inventory graphics, palettes, tiles/maps, sprites/animations, text, game
  tables and audio. For each family record format, compression, boundaries,
  pointers, cross-references, read routines and verification level. Distinguish
  static ROM data from dynamic RAM state.
- [ ] Verify EUR/USA and languages separately. Reuse existing readers as partial
  evidence; do not infer offsets by analogy alone.
- [x] Define stable resource IDs, versioned schema and manifest with source ROM
  hash, extractor/decoder version, files and content hashes, dependencies and
  coverage. Retain provenance from original intervals.
- [x] Choose a first family verifiable from extraction through the game, based
  on identified routines. Separate decoded content and necessary metadata: for
  example, one image does not preserve indexed palettes, priority, transparency
  and animations; PCM does not preserve audio sequencing.
- [x] Publish the migration matrix: for each family identify extractor, decoder,
  runtime reader, checks and remaining ROM dependencies.

**Closure:** contract and first family defined with evidence; still-unknown
families explicitly listed. Inventory is not complete migration.

**ROMDATA-01-CONTRACT completed; ROMDATA-01 partial:**
[inventory and contract](ROM_DATA_INVENTORY.md).
First family: `graphics.obj-dimensions`, static metadata verified on both exact
profiles. Inventory distinguishes known sources from unverified boundaries/
catalogs/languages; it does not claim a complete game resource catalog.
EUR/USA verification is complete for the first family; regional and language
catalogs of other families remain to be identified and verified.

**ROMDATA-01-GRAPHICS-LZ77 completed for initial observed producers:** four LZ77
sources per region verified on clean 1200-frame Linux and Windows/Wine startups;
stream boundaries and guest outputs match. Two regional roots of 88-byte
descriptors also identified. Catalog limits, internal fields, packet
interpretation and languages remain open.
[Contract and evidence](../rom_data_graphics.md).

**ROMDATA-01-BG-DESCRIPTOR-PREFIX completed for verified contiguous block:**
162 records/14256 bytes, all references within bounds, first payload distinct
from records and identical metadata in both profiles. Conditional selectors
verified with original routines for every record, sentinels and fallback.
Relative references, transfer triples, variant pointer and uninterpreted bytes
are distinguished. Graphics family 01 remains partial for linked assets/programs,
other catalogs and languages.
[Contract](../rom_data_graphics.md#authenticated-descriptor-metadata).

**ROMDATA-01-BG-PALETTES completed for declared base catalog:** all 162 entries
point to 127 sources per region, 125 LZ77 and 2 raw. Intervals, aliases and
decoded buffers match between EUR/USA. Four original Linux/Wine oracles pass
1296 calls/72792 instructions each, complete output/guards and bit 15 preserved.
Alternate-palette streams are separately classified below; prerequisites of the
five FF indices remain to be clarified. Typed model and palette consumption are
completed separately in units 02/04 below.
[Contract and evidence](../rom_data_graphics.md#base-background-palette-catalog).

**ROMDATA-01-BG-ALTERNATE-PALETTE-PROGRAMS completed for valid indices:** two
catalogs of 157 keys 0..156 per region, 314 streams with BE headers and LZ/RLE
format distinct from BIOS; each produces 256 B. Tables, sources and outputs match
between EUR/USA; four original Linux/Wine oracles pass 1256 calls each, complete
output/guards and unchanged EWRAM/SP. Five FF records remain distinct: no
invented dereference or fallback, caller prerequisites still to verify. Typed
parser 02 is now verified below; managed extraction and consumption 04 are
completed in units 02/04 below. No performance measurement.
[Contract](../rom_data_graphics.md#alternate-background-palette-catalogs).

**ROMDATA-01-BG-ALTERNATE-CALLER-CONTRACT: direct inventory and EUR modes
completed; global FF prerequisites open.** Four Thumb callers per region,
original setter and related confirmation identified. Options menu applies LCD
A=0, LCD B=1 (second catalog), TV=2 (first catalog) only after Save changes→Yes.
EUR native GUI snapshots confirm values without forcing RAM. No local check of
byte 56 FF in inspected paths; ID 00ff check is separate. Map/Party and
deployment GUI do not read alternate streams: pub scenes with positive use are
completed below; FF prerequisites remain.
[Contract and limits](../rom_data_graphics.md#alternate-palette-callers-and-color-mode).

**ROMDATA-01-BG-PACKET-CATALOG completed for declared fields 04/08/10:** 352
distinct sources per region (122 paired, 117 halfword, 113 auxiliary), 01 raw/
11 LZ77 wrappers with optional parent, initialization and dictionaries verified.
All references/aliases/hashes match EUR/USA, no overlaps/cycles, chains at most
2 nodes. Four Linux/Wine cases with original readers pass 486 invocations each,
complete buffers/guards/SP. The decoder now also exposes dictionary read extent.
Typed model/managed file and actual consumption remain 02/03/04; field 00 and
transfer/variant programs separate.
[Contract](../rom_data_graphics.md#complete-descriptor-packet-catalogs).

**ROMDATA-01-BG-MAIN-GRAPHICS completed for declared EUR catalog:** 162 links/
50 disjoint sources, 23 tag 20 and 27 tag 22, 621923 source bytes and 835232
output. Staged producer 0801b688 and custom LZ/RLE codec 080054a0 verified;
tag 22 retains destination offset and nested tag 20 header. All 50 outputs/
cursors/guards/SP match original on Wine; both DMA transfers verified for one
20 and one 22 case. New Linux/USA checks pending.
[Contract](../rom_data_graphics.md#european-main-background-graphics).

**ROMDATA-01-BG-TRANSFER-VARIANT completed for declared EUR catalog:** 97 active
triples in five slots (83/6/5/3/0), 42 controls and 34 shared raw tile banks,
76 disjoint intervals. All 162 variant pointers are null in this block; no
variant program invented. Original EUR producers confirm duration, frame order
and DMA with dynamic limit; Linux/USA pending.
[Contract](../rom_data_graphics.md#european-background-transfer-programs).

**ROMDATA-01-TEXT-AUXILIARY-ROOTS / ROMDATA-02-FONT-KEYBOARD-DATA completed
for declared EUR/Wine scope:** inventory of 16 direct callers and separate name
keyboard archive, three programs/18 row widths/180 cells. Last glyph 3174
verifies the end of the full bank of 3175 glyphs/metrics. Two targeted Wine
checks pass: JSON/source/guards and 3047 new original glyphs, three pages/
180 selectors (3230 calls); no repetition of the already closed 128 glyphs.
Unicode/other dynamic pointers and new Linux/USA checks pending.
[Contract](../rom_data_text.md#complete-european-font-and-name-keyboard).

**ROMDATA-01-OBJ-STATIC-LISTS / ROMDATA-02-OBJ-STATIC-DATA completed for declared
EUR/Wine scope:** 107 direct literal references identify 66 disjoint lists,
160 objects/1092 source bytes. Signed coordinates and all attribute fields
retained. Two targeted Wine checks pass: fields/boundaries/JSON and 1056 calls
to the two original routines with clipping/flip/wrap/capacity/early return,
entire OAM/matrix/guard buffers equal. Other indirect roots, animations and OBJ
graphics remain open; Linux/USA pending. [Contract](../rom_data_obj.md).

**ROMDATA-01-GAMEPLAY-ITEM-ROOTS: declared inventory completed, original contract
now verified below.** Property reader 080ce1ec/table 6e0bd0, 376 records × 32
bytes/12032 bytes to independent root 6e3ad0. Shop/classifier callers and
original enumeration 1..361/362..375 confirm bounds; 19 properties expose
halfword/byte/signed and live modifiers 02002030. All 32 bytes per record
retained, no overlap and e22; unproven semantics not invented. EUR oracle/model/
management/consumption now completed below; following 20-byte table, other
families and Linux/USA remain open. [Contract](../rom_data_gameplay.md).


<a id="romdata-02--estrattori-e-decoder"></a>

## ROMDATA-02 — Extractors and decoders

**ROMDATA-02-OBJ-CONTEXT-UPDATER scoped EUR progress:** E134 adds one
38-frame sequence and a 21-list OBJ bank. Wine verifies CLI create/reuse/verify/
startup, source bytes and three updater/draw full-state/clock cases each in
the interpreter and translated execution, including machine-state reload.
Ordinary use, UI save/Load, Linux, USA and native Windows remain open; no
performance measurement.
[Evidence](../rom_data_obj.md#conditional-context-updater-obj-bank-e134).

**ROMDATA-02-OBJ-BRANCH-LISTS scoped EUR progress:** three branch-selected
static OBJ lists (36 bytes) are added in E133. Wine interpreter and translated
draws match full state/clock in six cases each, including machine-state reload;
production CLI create/reuse/verify and managed startup pass. Ordinary scenes,
UI save/Load, other sources, Linux, USA and native Windows remain open. No
performance measurement. [Evidence](../rom_data_obj.md#branch-selected-static-obj-lists-e133).

**ROMDATA-02/03/04-GAMEPLAY-6E84E0 scoped EUR progress.** The 31 four-byte
records selected by authenticated item byte18 are modeled and attached with an
item-owner dependency; E132 managed Wine publication and 194 original prefix
comparisons pass. Live-u16 indices outside the projection preserve existing
owners/raw behavior. Actual Wine CLI create/reuse/verify and one-frame startup
pass with zero new-owner reads. A stocked IN-08 EUR Trade Items scene then
passes a 600-frame Wine native raw/managed state/clock/image/OAM/dispatch/audio
pair with positive byte01 and value02 reads. A separate Wine lifecycle check
verifies E132 owner retention across Load/Undo, backup restore and restart.
Earlier source-save ancestry, global selector extent, further scenes, Linux and USA remain open. No
performance measurements.
[Contract](../rom_data_gameplay.md#conditional-item-derived-four-byte-parameters).

**ROMDATA-02/03/04-AUDIO-EXTENDED-DISPATCH completed for the conditional
EUR fragment.** A separate typed resource owns ten callback words for selectors
0..7 and 10..11; existing selectors 8/9 and the sequence grammar remain intact.
Wine original-prefix, managed/interpreter/translated full-state, mixed-boundary
and transactional checks pass; a later Wine check covers guest widths/mirrors
and mixed-owner DMA3/CpuSet transfers. E131 CLI create/reuse/verify and one-frame startup
pass; that startup has zero reads of the new fragment. The guest selector is not
bounded by this projection; full handler behavior, ordinary production and table
extent remain open. Linux/USA/native Windows remain pending; no performance
measurements. [Contract and evidence](../rom_data_audio.md#conditional-extra-extended-dispatch-words).

**ROMDATA-01-TEXT-REMAINING-CATALOGS / ROMDATA-02-TEXT-CATALOG-DATA completed
for the declared EUR reader block:** 9 pointer catalogs/45 tables/16100 strings,
29 relative selectors/145 archives/17035 programs. Flags, commands/operands,
exact custom LZ/RLE, decoded padding and 128 referenced glyphs retained; 33581
disjoint intervals. Two targeted Wine tests pass, including complete independent
inventories and 80 pointer boundaries/145 roots/300 script outputs/63 new
original glyphs (968 calls). Valid dynamic context explicit in the oracle; no
scene evidence inferred. Managed integration/extended runtime consumption still
open, e11 retains the previous subset. Linux/USA pending.
[Contract and limits](../rom_data_text.md#broad-pointer-catalogs-and-relative-text-archives).

**ROMDATA-02-TEXT-CATALOG-MANAGED completed for EUR/Wine:** tenth fixed file
text-catalogs.json, revision 1/12/11, 33581 intervals and set authentication/
identity. Publication/reuse, atomic error/repair, actual EUR checks on data/
consistent changes/metadata/paths/dependencies/limits, move and transactional
failed activation pass in three reduced Wine checks. Actual CLI prepare/reuse
without writes/one-frame activation with old cache pass. e11 and eight previous
files unchanged, USA remains e8. Extended 04 consumption verified-only and
lifecycle/scenes 05 still open; Linux/USA pending.

**ROMDATA-01-TEXT-SHORT-STRINGS / ROMDATA-02-TEXT-SHORT-DATA completed for the
declared EUR catalog:** five original tables of 107 entries, 535 distinct strings
(478 compact/57 extended), 5468 source bytes. Typed model with exact glyph codes/
control bytes and only 65 packed 4bpp glyphs/metrics used; 670 intervals with
provenance/hash. Two targeted Wine tests pass: codec/guards and all original
selection/copy/glyph/width routines, 1331 calls/89446 instructions. Strict JSON
and consistent changes rejected. Managed publication/startup/runtime consumption
remain open, schema e10 unchanged. Other catalogs, complete font/Unicode/commands,
Linux/USA pending. [Contract and evidence](../rom_data_text.md).

**ROMDATA-02-TEXT-SHORT-MANAGED completed for EUR/Wine:** ninth fixed file
short-texts.json, revision 1/11/10 and 670 intervals with provenance. Complete
set/identity authentication, atomic publication, reuse/repair/move and
transactional failed activation pass in three targeted Wine tests. Actual CLI:
prepare/reuse without writes/one-frame activation with retained native cache
pass. e10 and seven previous data files unchanged; USA remains e8. New catalog
is verified-only; consumption 04 and scenes/lifecycle 05 open, Linux/USA pending.

**Dependencies:** ROMDATA-01. Proceed by family without declaring the rest of
the catalog finished when the first prototype works.

- [x] Implement parsers with checks on lengths, offsets, pointers, decoded sizes
  and references; reject truncated or inconsistent input.
- [x] Extract and decode the first family in the agreed format, retaining
  information needed to reproduce original behavior. Unknown blocks may remain
  raw, marked as undecoded.
- [x] Make output deterministic and inspectable: same inputs and version produce
  equivalent manifest and data without absolute paths.
- [x] Verify with synthetic fixtures and local comparisons with the original
  decoder or data observed in game. Where the format permits, also verify
  roundtrip without requiring byte-identical compression when multiple encodings
  represent the same data.
- [ ] Extend extractors and decoders to other matrix families, recording
  supported content and limits for each.

**Closure per family:** correct decoded files, sufficient metadata and invalid
inputs handled. Exporting binary blocks alone is insufficient.

**ROMDATA-02-OBJ-DIMENSIONS completed:** extractor and typed decoder, CLI
`rom-data extract|verify`, manifest and provenance checks. First four items are
complete for this family (without pointers or compression); parsers for other
families remain to be implemented. Linux and Windows/Wine tests pass with both
local ROMs too. Details and limits in [decoded resources](../rom_data.md).
Direct consumption of the first family is now connected in opt-in path 04.

**ROMDATA-02-LZ77-DECODER completed as an offline component:** bounded input
and memory, overlapping references, exact consumption without padding and
malformed input rejection. Four synthetic tests per target and comparison with
already captured guest outputs pass on Linux/Wine. It does not yet add a family
to the manifest: graphics-catalog semantic contracts are needed first.

**ROMDATA-02-BG-PACKETS completed as an offline component:** typed parsers for
16-bit packets and indexed pairs, with destination, write and packet-count
limits. Three synthetic tests per target and comparisons with original EUR/USA
routines pass on Linux/Wine: 7 invocations/87160 instructions per case, entire
EWRAM compared and block continuation verified. The literal-pair branch retains
syntax, with destination guard after an original check contradicted the initial
assumption. [Contract, evidence and limits](../rom_data_graphics.md#typed-background-packets).
Catalog and integration of new resources remain open.

**ROMDATA-02-BG-DESCRIPTOR-METADATA completed as an offline component:** typed
decoder, bounds checks, source authentication, exact roundtrip and deterministic
TOML. Three synthetic tests per target and four EUR/USA cases on Linux/Wine pass:
2612 original calls/134629 instructions per case. The 106309-byte test files
match across the four paths. Manifest integration, preparation and runtime
consumption were selected separately as follows.

**ROMDATA-02-BG-MANAGED completed for extraction/verification/preparation:** new
descriptor file in manifest, schema 1/extractor 3/decoder 2 and v1/e3-d2
directory, retaining previous versions. Limits 16 KiB manifest/OBJ, 256 KiB BG
and 272 KiB total; atomic publication of three files, full reuse, recovery,
whole-dataset identity and authenticated activation. 25 Linux/23 Wine tests,
18 real CLI checks per target and tar/zip privacy scenarios pass.
[Evidence and limits](../rom_data_graphics.md#managed-descriptor-extraction-and-preparation).
This unit proves preparation; subsequent direct consumption 04 is described below.

**ROMDATA-02-BG-BASE-PALETTE-DATA completed:** model of banks of 16 u16 values,
bit 15 retained, bounded raw 0/LZ77 10 parser and exact intervals without
padding. All 162 entries extracted to readable JSON; manifest with all
provenance, four files and revision 1/5/4, old generations retained. 39 Linux/
37 Wine tests and 22 CLI checks per target pass, including original oracles,
integrity/repair/relocation and compatibility of already consumed resources.
[Contract and limits](../rom_data_graphics.md#typed-palette-extraction-and-preparation).
This unit covers preparation; subsequent consumption 04 is completed below
with revision 1/6/5 and compatibility metadata. No compression/space work.


**ROMDATA-02-BG-ALTERNATE-PALETTE-CODEC completed for the verified catalog:**
bounded custom codec, BE header, all tokens and original checks 03..0F, exact
consumption and rejection of truncation, out-of-history references and overrun.
Model with complete u16 banks, source/link metadata and five FF values retained.
Linux/Wine: 4 core tests, 314 buffers per region equal to retained oracles,
70 synthetic streams/1120 original calls per regional case/target and negative
check of original overrun. Deterministic readable JSON across four paths; no
performance measurement. Managed extraction/preparation and alternative-file
consumption are selected separately below.
[Contract and evidence](../rom_data_graphics.md#typed-alternate-palette-codec).

**ROMDATA-02-BG-ALTERNATE-PALETTE-MANAGED completed:** fifth readable JSON file
with all banks/keys/links and 316 provenance intervals; schema 1/extractor 7/
decoder 6, old e6 generations retained. Atomic set write/verification, no-write
reuse, repair and authenticated activation with full identity. Linux 23 default
+ 4 private and Wine 21 + 4 pass; 23 real CLI checks per target, including
EUR/USA, recovery and relocation. No performance measurement or space reduction.
Alternate activation is still `verified-only` at the end of this unit;
consumption 04 is completed below.
[Evidence and limits](../rom_data_graphics.md#managed-alternate-palette-datasets).

**ROMDATA-02-BG-PACKET-DATA completed for EUR; USA verification deferred:**
offline model with 352 unique programs/486 links, typed ordered writes, parents/
aliases and original wrapper/body metadata. Bounds, cycles, IDs, overlaps,
inconsistencies and consistent base changes are checked. Eight core tests, one
JSON test and one EUR case with all retained buffers pass on Linux/Wine; readable
JSON identical. No new original replay, active schema or runtime consumer. Sixth
managed file completed below; consumption 04 remains.
[Contract and evidence](../rom_data_graphics.md#typed-descriptor-packet-programs).

**ROMDATA-02-BG-PACKET-MANAGED completed for EUR/Wine:** sixth fixed file
`bg-packet-programs.json`, manifest with 352 intervals and BG dependency, revision
1/8/7 with e7 generations retained. Input guards 64 MiB per packet file/set,
whole-dataset parse/verification before publication/activation. Four targeted
Wine cases and seven real Windows commands pass; canonical files/originals
protected. Runtime status verified-only at publication; EUR consumption 04 is
now completed below. New Linux/USA checks pending, no performance campaign or
broad suite. [Contract and evidence](../rom_data_graphics.md#managed-packet-program-datasets).

**ROMDATA-02-BG-GRAPHICS-DATA completed for EUR/Wine:** offline typed model with
complete little-endian words, offsets, exact source, aliases and canonical IDs;
bounded parser with ROM authentication. Codec shared with alternate palettes,
retaining their 256/4096-byte guards. Nine targeted Wine tests pass, including
readable JSON/roundtrip and 54 original staged calls. No new manifest revision or
runtime migration; managed extraction and consumption remain to be connected.
Linux/USA pending, no performance measurement.
[Contract and evidence](../rom_data_graphics.md#european-main-background-graphics).

**ROMDATA-02-BG-GRAPHICS-MANAGED completed for EUR/Wine:** seventh fixed file
`bg-main-graphics.json`, revision 1/9/8 and 50 authenticated intervals; USA
retains six files/e8-d7 without new checks. Atomic publication, no-write reuse,
repair, full identity and previous e8 generations retained. Four targeted Wine
tests and seven real Windows CLI commands pass, including activation with
previous native cache. New graphics runtime verified-only at that boundary; no
use evidence inferred from the first frame. Consumption 04 completed below.
Linux/USA pending, no benchmark or space reduction.
[Contract and evidence](../rom_data_graphics.md#managed-european-main-graphics).

**ROMDATA-02-BG-TRANSFER-DATA completed offline for EUR/Wine:** controls with
u16 duration/32-byte tile offset, transfer size and byte 3 retained; tiles as
complete u32 words, independent aliases/IDs and typed destinations. Five
targeted Wine checks pass (3 core/1 JSON/1 private oracle): four original
animations, two full cycles, VRAM/guards after every copy and 34 partial DMA
chunks; 634 calls/80766 instructions. Full provenance and roundtrip; consistent
changes rejected. No new manifest or consumer at that boundary: managed file
completed below, consumption 04 subsequent. Linux/USA pending, no performance
measurement. [Contract and evidence](../rom_data_graphics.md#european-background-transfer-programs).

**ROMDATA-02-BG-TRANSFER-MANAGED completed for EUR/Wine:** eighth fixed file
`bg-transfer-programs.json`, revision 1/10/9, 76 authenticated intervals and
full set identity. Four targeted Wine tests and three real Windows CLI commands
pass preparation/reuse/repair/relocation/transactional activation and previous
native cache. e9 generations and six prior decoded files retained; USA remains
with six files e8-d7. Transfer runtime verified-only at that boundary, no use
evidence inferred from first frame; consumption 04 completed below. Linux/USA
pending, no benchmark.
[Contract and evidence](../rom_data_graphics.md#managed-european-transfer-programs).


**ROMDATA-02-FONT-KEYBOARD-MANAGED / ROMDATA-04-FONT-KEYBOARD-CONSUMPTION
completed for declared EUR/Wine scope:** two new fixed files font-bank.json and
name-keyboard.json, dataset 1/13/12 with 12 files/11 resources; e12 and prior
files retained, USA remains e8-d7. Authentication of seven provenance sources
and the entire set; immutable typed owner for glyphs/metrics/indices/programs/
rows. Three targeted Wine checks pass publication/atomic error/reuse/repair/
relocation, change rejection and transactional activation; five original/native
calls with all seven sources altered and equal states/clock; Load/Undo/backup/
restart with temporary copies unavailable. No save/native ABI change or guest
instruction skipped. Linux/USA pending.

**ROMDATA-05-FONT-KEYBOARD-SCENE-CLI completed for one EUR/Wine transition:**
397-frame Cyril pub with equal states/clock/images/OAM/dispatch/audio and 8064
glyph bytes/93 metrics from the complete bank. Actual CLI prepares/reuses e13
without writes and old cache, same retained legacy result. The scene does not
use the keyboard: ordinary name scene/GUI/archives remain pending. No performance
measurement or space optimization.
[Evidence](../rom_data_text.md#complete-european-font-and-name-keyboard).

**ROMDATA-02-OBJ-STATIC-MANAGED / ROMDATA-04-OBJ-STATIC-CONSUMPTION completed
for declared EUR/Wine scope:** fixed file obj-static-lists.json, 1/14/13 with
13 files/12 resources and 66 provenance sources; e13 and prior files retained,
USA remains e8-d7. Three targeted Wine checks pass publication/reuse/repair/
relocation, change rejection and transactional activation, eight original/
native calls with all 66 sources altered and equal states/clock, boundaries/
mirrors/SWI/DMA, eight host draw/publication cases and Load/Undo/backup/restart.
No ABI change or guest instruction skipped. An additional Wine check passes
792 host draw/visibility/battle cases and live changes to RAM/ROM lists outside
the catalog; new Linux/USA checks pending.

**ROMDATA-05-OBJ-STATIC-SCENE-CLI completed for one EUR/Wine transition:**
397-frame pub with every state/clock/image/OAM/dispatch/audio equal and 5470
typed OBJ bytes/958 host reads. Three actual CLI processes from an unrelated cwd
prepare/reuse without writes and replay with old cache, retained legacy result
equal. No performance/space measurement. Broader GUI/archives and other OBJ
catalogs remain open. [Evidence](../rom_data_obj.md#verification).

**ROMDATA-01-OBJ-UI-ANIMATION-ROOTS / ROMDATA-02-OBJ-UI-ANIMATION-DATA completed
for declared EUR/Wine scope:** 30 literal pairs identify 10 OBJ banks/sequences,
167 lists/284 objects/95 sequences/184 frames and 282 intervals. Relative
indices, OBJ fields and u16 durations retained; counters remain RAM. Two Wine
checks pass JSON/source/guards, 115 original constructors and 2292 original
ticks with equal task, loop/last frame, OAM/matrices/guards. Three empty sequences
retained, direct domains now classified under ROMDATA-01-OBJ-EMPTY-CALLERS;
indirect/live scheduling unverified. The seven additional direct lists are now
migrated under e16; other indirect callers remain open. Linux/USA pending.

**ROMDATA-02-OBJ-ANIMATION-MANAGED / ROMDATA-04-OBJ-ANIMATION-CONSUMPTION
completed for declared EUR/Wine scope:** obj-animation-banks.json, 1/15/14 with
14 files/13 resources and 282 provenance sources; e14 and all prior files
retained. Three targeted Wine checks pass authentication/publication/reuse/
repair/relocation, 49 original/native constructor/body calls with altered
sources and equal states/clock, all six types read by compiled code, boundaries/
mirrors/SWI/DMA, ten host banks and Load/Undo/backup/restart without files. ABI
and guest work unchanged. The short 240-frame battle scene matches completely but
uses 0 bytes from these banks: the positive gate rejects it. A 160-frame map+
Start scene now passes every state/clock/image/OAM/dispatch/audio with 36513
typed bytes/3200 host reads; three real CLIs prepare/reuse without writes and
replay with old cache. Final menu inspected. Linux/USA pending.
[Contract and evidence](../rom_data_obj.md#indexed-ui-animation-banks).


**ROMDATA-02-OBJ-EXTRA-DIRECT-LISTS / ROMDATA-04-OBJ-EXTRA-CONSUMPTION /
ROMDATA-05-OBJ-EXTRA-SCENE-CLI completed for declared EUR/Wine scope:** sixteen
additional references/seven new lists bring the static catalog to 123 references,
73 lists, 173 objects and 1184 source bytes. e16-d15 retains e15, all 66 prior
records and other decoded files; 14 files/13 resources remain. Three targeted
Wine checks pass authentication/publication/reuse/repair/relocation, 14 original/
interpreter/native calls with seven sources altered and equal states/clock,
boundaries/mirrors/SWI/DMA, 14 host cases and Load/Undo/backup/restart without
files. Per-source guest/host flags prevent prior lists from satisfying the new
gate. Battle/Party/character-menu observations are retained as negative coverage;
EUR language selection at startup uses three new lists. One 240-frame comparison
passes every state/clock/image/OAM/dispatch/audio; actual CLI prepares/reuses
without writes and replays with old cache from an unrelated cwd, final image
inspected. No closed test repeated. Other scenes/GUI/archives, new Linux/USA
checks and switch/indirect roots pending.
[Evidence](../rom_data_obj.md#additional-direct-lists-in-e16).

**ROMDATA-01-OBJ-ZERO-COUNT-PAIR-CONTRACT / ROMDATA-02-OBJ-ZERO-COUNT-DATA /
ROMDATA-04-OBJ-ZERO-COUNT-CONSUMPTION / ROMDATA-05-OBJ-ZERO-COUNT-SCENE-CLI
completed for declared EUR/Wine nonrepeat scope:** eleventh bank b4d8e8/b4f4e8,
one list/two objects/five zero counts. Explicit selectors and duration references
retain zero headers and reuse typed owners of subsequent headers/indices. Eight
new intervals/58 bytes; e17-d16 retains the ten prior records, other files and
e16 without ABI change. One original Wine check passes 14 constructors/104 ticks
and addresses read/task states/OAM/matrices/guards. Two targeted Wine checks pass
authentication/aliases/consistent changes/publication/reuse/repair/relocation/
Load/Undo/backup/restart, eight interpreter/native comparisons with sources and
shared duration altered and equal states/clock, boundaries/mirrors/SWI/DMA and
host. A 240-frame Party scene passes every state/clock/image/OAM/dispatch/audio
with 288 new selector bytes and 288 new host reads; actual CLI prepares/reuses
without writes and replays with prior cache, image inspected. Direct domains of
the three prior empty sequences now classified under ROMDATA-01-OBJ-EMPTY-CALLERS;
repeat1/different prerequisites and other roots remain open. Linux/USA pending.
[Contract](../rom_data_obj.md#zero-count-hold-bank-in-e17).

**ROMDATA-02-OBJ-LOCALIZED-UI-BANKS / ROMDATA-04-OBJ-LOCALIZED-CONSUMPTION /
ROMDATA-05-OBJ-LOCALIZED-SCENE-CLI completed for declared EUR/Wine scope:** five
language-selected banks through two pointer tables; 149 lists/574 objects/144
frames and 166 additional intervals/4984 bytes. e18-d17 retains 11 prior banks,
other files and e17 generations. The catalog now has 16 pairs/317 lists/105
sequences/328 frames, still 14 files/13 resources. Original selections and u16
256 duration with u8 counter retained. One Wine check passes 15 selections/10
constructors/1589 ticks and task states/OAM/matrices/guards; two targeted checks
pass authentication/reuse/repair/relocation/Load/Undo/backup/restart, 30 original/
interpreter/native comparisons with 166 sources altered, new boundaries/mirrors/
SWI/DMA and five host publications. The retained battle and first directional
map attempts have zero coverage, retained without duplicate comparison or repeat.
The actual “Engage!” scene from the map uses normal mouse targeting at Lutia Pass
and A: 400 frames with every state/clock/image/OAM/dispatch/audio equal, 8 table
bytes/5906 bank bytes/228 new host reads. Actual CLI prepares/reuses without
writes and uses old cache from an unrelated cwd; final image inspected. New
Linux/USA checks pending. Linked compressed OBJ graphics remain open; direct
domains of the three old empty records now classified under
ROMDATA-01-OBJ-EMPTY-CALLERS, indirect/live entries unverified.
[Evidence](../rom_data_obj.md#language-selected-ui-banks-in-e18).


**ROMDATA-01-OBJ-GRAPHICS-PAYLOADS / ROMDATA-02-OBJ-CUSTOM-GRAPHICS /
ROMDATA-04-OBJ-CUSTOM-CONSUMPTION / ROMDATA-05-OBJ-CUSTOM-SCENE-CLI completed
for the declared EUR/Wine catalog:** 39 custom banks and six tables of five
pointers, 45 intervals/62674 source bytes/131872 decoded bytes; thirteen direct
references and six canonical indexed paths. EUR e19-d18 adds obj-graphics.json
(15 files/14 resources), retains all 13 prior decoded files and e18 generations.
Typed literals/pointers feed original decoder/copies; scratch, transformations,
partial transfers and live RAM tail remain active. Wine verifies 39 original
decoders/30 selectors/6 copies, publication/reuse/recovery/relocation/Load/Undo/
backup/restart and 39 interpreter/native decoders/30 selectors/4 copies with
altered sources and equal states/clock. A DMA fixture address fix is verified
separately with 20 positive typed bytes. A 400-frame Engage scene passes states/
images/OAM/dispatch/audio with 1194 source bytes/460 literals/selector: 4.
Actual Windows CLI creates/reuses without writes/executes from an unrelated cwd
with previous native cache; canonical executable and original sources unchanged.
Other OBJ payloads/palettes, indirect/live scheduling of the three empty records
and Linux/USA remain open; direct domains now classified under
ROMDATA-01-OBJ-EMPTY-CALLERS; no performance measurement.
[Contract and evidence](../rom_data_obj.md#custom-obj-word-banks-in-e19).

**ROMDATA-01-OBJ-PALETTE-SOURCES / ROMDATA-02-UI-RAW-PALETTE-MANAGED /
ROMDATA-04-UI-RAW-PALETTE-CONSUMPTION / ROMDATA-05-UI-RAW-PALETTE-SCENE-CLI
completed for the declared EUR/Wine catalog:** 21 banks/23 direct, derived or
Color Mode references, 1328 u16 values/2656 bytes without overlaps, for both
BG/OBJ halves of the palette buffer. EUR e20-d19 adds ui-palettes.json, 16 files/
15 resources; all 14 prior decoded files/e19 unchanged. Wine verifies original
176-byte initializer, 27 copies/10 Color Mode cases, JSON/sources/bit 15,
publication/reuse/recovery/relocation/Load/Undo/backup/restart and full
interpreter/native parity with all 21 sources altered and 3528 positive compiled
bytes. Bus/DMA 160 bytes and publication of live RAM changes pass. Ordinary
240-frame startup/actual CLI match states/clock/images/OAM/dispatch/audio with
33 reads/130 palette bytes. Other indirect sources and Linux/USA/native Windows
remain open; no performance measurement. [Contract and evidence](../rom_data_ui_palettes.md).

**ROMDATA-01-RETURN-LENGTH-CUSTOM-GRAPHICS / ROMDATA-02-UI-CUSTOM-GRAPHICS /
ROMDATA-04-UI-CUSTOM-CONSUMPTION / ROMDATA-05-UI-CUSTOM-SCENE-CLI completed for
the declared EUR/Wine catalog:** nine custom banks (eight BG/one OBJ), ten
original literal/decoder/upload paths, 57038 source bytes/84160 decoded. Counts
derived from r0 and the 544/2944-byte partial copy remain original instructions.
EUR e21-d20 adds ui-graphics.json (17 files/16 resources), retains all 15 prior
decoded files/e20. Wine passes sources/JSON/metadata, publication/reuse/recovery/
relocation/Load/Undo/backup/restart and all ten interpreter/native paths with
nine intervals altered and equal states/clock; 68906 compiled bytes/39861
literals and all banks positive. Bus/mirrors/BIOS/CpuSet/FastSet/DMA 32 bytes
pass. First Party→Unit 320-frame locator negative and retained; positive 320-frame
Clan scene with 1642 source bytes/764 literals, equal states/clock/images/OAM/
dispatch/audio and actual CLI create/no-write reuse/old cache. Other indirect
sources/variants remain open, Linux/USA pending, no performance measurement.
[Contract](../rom_data_ui_graphics.md).

**ROMDATA-01-UI-LANGUAGE-GRAPHICS / ROMDATA-02-UI-LOCALIZED-GRAPHICS /
ROMDATA-04-UI-LOCALIZED-CONSUMPTION / ROMDATA-05-UI-LOCALIZED-SCENE-CLI
completed for declared EUR/Wine scope:** seven new banks/two selectors for five
languages/two links to existing OBJ owners. 17 new intervals/14436 bytes
(14396 payload + 40 pointers)/35008 decoded; total 16 banks/26 intervals/71474
source bytes/119168 decoded. EUR e22-d21 retains 17 files/16 resources, nine UI
records/all 15 other decoded files/e21 unchanged. Live language, code jump table,
English fallback versus skip and 5056 OBJ/2688 BG copy remain original. Three
targeted Wine checks pass 14 selector paths/seven new producers/two shared BG
suffixes only, authentication/owners/transactions/publication/reuse/repair/
relocation/Load/Undo/backup/restart and native with 17 intervals altered/equal
states-clock; 48 compiled selector bytes/14310 new source bytes/6449 literals/
all 7 banks positive, bus/BIOS/SWI/DMA 4 bytes. Only failed new fixtures fixed:
duplicate module and native chain beyond selector-only stops; evidence retained,
no runtime change for those issues. CLI create/no-write reuse passes; 320-frame
Clan is negative for new data and not compared again. Normal 1200-frame startup
with existing input uses 4 selector bytes/2347 new source bytes b7d9e4/1124
literals: one full states/clock/images/OAM/dispatch/audio pair passes, final
image inspected. Other ordinary language scenes/indirect sources remain open,
Linux/USA/native Windows pending; no performance/space measurement.
[Contract](../rom_data_ui_graphics.md#localized-ui-selectors-in-e22).


**ROMDATA-02-GAMEPLAY-ITEM-SOURCE-ORACLE / ROMDATA-02-GAMEPLAY-ITEM-DATA /
ROMDATA-04-GAMEPLAY-ITEM-CONSUMPTION completed for declared EUR/Wine scope:**
376 records/32 bytes/12032 source bytes, items.json/resource gameplay.item-records,
EUR e23-d22/18 files/17 resources, all 16 prior decoded files/e22 unchanged.
Halfword/byte/signed/tail fields preserve every bit; properties/live bonuses/
inventory/unsigned clamp and secondary table remain original. One Wine oracle
passes 14371 calls across all records/19 properties/two bonus branches, special/
truncated/invalid cases and 240 isolated negative signed cases; all distributed
ROM fields are nonnegative. First incorrect fixture assumption corrected before
execution, failed log retained. Two Wine integrations pass management/recovery/
retention/transactions/provenance/Load/Undo/backup/restart and 86 native calls
with 12032 raw bytes altered/equal states-clock; 100 compiled bytes (24 prefix/
40 signed/36 other). All bytes/edges/mirrors/BIOS/SWI/DMA 4 bytes pass.
ROMDATA-05-GAMEPLAY-ITEM-SCENE-CLI is complete for one 320-frame Unit scene:
CLI create/no-write reuse/old cache and one full states/clock/images/OAM/dispatch/
audio pair pass with 40 new signed bytes; final image inspected. Ordinary prefix/
tail not claimed; other catalogs and Linux/USA/native Windows remain open, no
performance/space measurement. [Contract](../rom_data_gameplay.md).

**ROMDATA-01-GAMEPLAY-ITEM-SECONDARY-ROOTS / ROMDATA-02-GAMEPLAY-ITEM-LINK-SOURCE-ORACLE
completed for declared EUR/Wine contract:** 6e3ad0..6e4c64, 225 records × 20
bytes/4500 bytes, count 0..9/opaque byte/nine pairs, 472 active/eight FF. Twelve
literal LDRs and independent next root confirm boundaries; 376 items reference
220 rows, all 225 retained. Lossless typed catalog/JSON/source/owner/extent/
coherent edit passes Wine: 378 selectors/1394 searches/464 normal prefixes/
8 FF prefixes, 4 synthetic zero/duplicate/unused queries and 8 cases separate
from the earlier comparison at the FF branch. 2244 cases/61906 main instructions
+ 80 in the gate, no performance measurement. Unit bit 7/byte 3d remain live;
subsequent job/ability calls not claimed. The contract precedes subsequent e24
integration reported below; new Linux/USA/native Windows checks remain pending.
[Contract](../rom_data_gameplay.md#item-linked-pair-source-contract).

**ROMDATA-02-GAMEPLAY-ITEM-LINK-DATA / ROMDATA-04-GAMEPLAY-ITEM-LINK-CONSUMPTION
completed for declared EUR/Wine scope:** e24-d23 adds item-links.json/resource
gameplay.item-links with explicit dependency on the 376 parsed items; 19 files/
18 resources, all 17 prior decoded files/e23 unchanged. 225 records/4500 bytes
of count/selector/value/opaque feed CPU/DMA/native readers; queries, unit bit 7/
FF byte 3d, subsequent calls, clock/save/native ABI remain original. Two Wine
checks pass publication/failure/reuse/repair/relocation/retention/transactions/
lifecycle and 62 native paths with altered source/equal states-clock: 36 count/
114 selector/56 value bytes. Opaque is not read by these paths but is verified
through the bus with all 4500 bytes; edges/mirrors/BIOS/SWI/DMA 4 bytes and a
crossing of 2 old bytes + 2 new bytes with both raw sources altered pass. No old
oracle repeated.
ROMDATA-05-GAMEPLAY-ITEM-LINK-SCENE-CLI is complete for the declared EUR/Wine
scene: CLI create/no-write reuse/old cache passes; 400-frame Equip slot locator
reads 0 new bytes, retained without pair/repeat. Ordinary Down/A open the
headgear list (final image inspected); 560-frame CLI and one full states/clock/
images/OAM/dispatch/audio pair pass with 574 new bytes (287 count/205 selector/
82 value). Private weapon-list names are locators, not evidence of a weapon
scene. Linux/USA/native Windows and the rest of the TODO remain open, no
performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-item-link-consumption).

**ROMDATA-01-GAMEPLAY-METADATA16-ROOTS / ROMDATA-02-GAMEPLAY-METADATA16-SOURCE-ORACLE
completed for declared EUR/Wine contract:** 128 records × 16 bytes/2048 bytes,
6e4c64..6e5464, 23 literal LDRs and independent next root. Lossless u16 alias +
14-byte model/JSON/source/extent and independent inventory pass. One Wine check
passes 2581 original display/nonzero-alias/live-list/sum/flag/choice cases and
19 synthetic cases for priority 1 absent in data/u8 sum/shift. Original LCG state
and RAM/stack/callee guards verified; no law/mission/ability name invented, alias
0 not claimed. 156038 instructions/319 PCs are coverage, not performance
measurements. EUR managed remains e24-d23/19 files/18 resources; preparation,
direct consumption/lifecycle/scene/CLI of this new table open. Linux/USA/native
Windows pending. [Contract](../rom_data_gameplay.md#sixteen-byte-linked-metadata-source-contract).

**ROMDATA-02-GAMEPLAY-METADATA16-DATA / ROMDATA-04-GAMEPLAY-METADATA16-CONSUMPTION
completed for declared EUR/Wine scope:** e25-d24 adds linked-metadata16.json/
resource gameplay.linked-metadata16, 20 files/19 resources; all 18 prior decoded
files/e24 retained, USA e8-d7. Typed alias/display/flag/value/opaque fields
feed CPU/DMA/native while retaining original live list/RNG/sums/shifts/clock/
save/native ABI. Two Wine checks pass publication/failure/reuse/repair/
relocation/retention/transactions/lifecycle and 42 native paths with 2048 raw
bytes altered/equal states-clock: 8 alias/17 display/22 flag/24 value compiled
bytes. Opaque verified on the bus with all bytes/edges/mirrors/BIOS/SWI/DMA 4
bytes and crossing 2 old link + 2 new metadata bytes with both altered. Source
oracle 2581+19 not repeated. ROMDATA-05-GAMEPLAY-METADATA16-SCENE-CLI is
complete: CLI creates/reuses and 22-file hash/mtime no-write check passes;
Area List/Start/Party/normal inventory locators read 0 new bytes and are neither
repeated nor paired. Clan→Mission Item 520 frames reads 6 new display bytes,
Chocobo Egg image inspected; one states/clock/images/OAM/dispatch/audio pair
passes with the same 6 bytes. Ordinary alias/flag/value not claimed. Initial
input-header error corrected before execution; only corrected replay and then
the previously unmet no-write criterion were run. New Linux/USA/native Windows
and the rest of the TODO remain open; no performance/space measurement.
[Contract](../rom_data_gameplay.md#managed-and-direct-metadata16-consumption).


**ROMDATA-01-GAMEPLAY-METADATA52-ROOTS / ROMDATA-02-GAMEPLAY-METADATA52-SOURCE-ORACLE
completed for the stated EUR/Wine contract:** 116 records of 52 bytes /
6032 bytes, 6e5464..6e6bf4, eight LDR literals, enumerator 0..115 and an
independent next root. Lossless model with an explicit route and three fixed
arrays; 48 unsigned/bitfield properties preserve FF referrals to the caller's
index, nonzero referrals and original cycles, without invented fallbacks. A Wine
check passes 6369 real calls / 4 bounded non-returning probes and 294 synthetic
high-bit/referral calls / 2 cyclic probes; tail32/33 readers from a live index and
JSON/source/extent/RAM/stack/callee guards pass. The previous 464 normal
selectors fit within the new owner; eight FF remain distinct. No old oracle was
repeated; 223421 instructions / 524 PCs are coverage only. The dataset remains
e25-d24 / 20 files / 19 resources: preparation, direct/lifecycle/scene/CLI
consumption of this table and new Linux/USA/native Windows checks remain open.
[Contract](../rom_data_gameplay.md#fifty-two-byte-linked-metadata-source-contract).

**ROMDATA-02-GAMEPLAY-METADATA52-DATA / ROMDATA-04-GAMEPLAY-METADATA52-CONSUMPTION
completed for the stated EUR/Wine scope:** e26-d25 adds linked-metadata52.json /
resource gameplay.linked-metadata52, 21 files / 20 resources; all 19 previous
decoded files and the e25 generations preserved, USA e8-d7. The parsed owner of
the normal selectors is validated within 116, with FF distinct. Typed header/
routing/packed/tail/opaque reads preserve original getters, alternate indices,
referrals, live records, clock and save/native ABI. Two Wine checks pass
publication/failure/reuse/repair/movement/retention/transactions/lifecycle and
75 native paths with all 6032 raw bytes altered and equal state-clock results:
18 header / 83 routing / 68 packed / 8 tail bytes compiled. Opaque at the bus
with all bytes/edges/mirrors/BIOS/SWI/DMA4 and the crossing of 2 metadata16 bytes
+ 2 metadata52 bytes both altered. Source/cycle oracle not repeated.
ROMDATA-05-GAMEPLAY-METADATA52-SCENE-CLI complete: CLI creates/reuses without
changing hash/mtime, uses the old cache from a different cwd and opens Change Jobs
with ordinary input. A 540-frame pair passes state/clock/images/OAM/dispatch/audio
with 664 new bytes (74 header / 335 routing / 255 packed); final Marsh/Thief
image inspected. Ordinary tail/opaque use is not claimed. No failed locator,
failed harness/test or previous check repeated. New Linux/USA/native Windows
checks and the rest of the TODO remain open; no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-metadata52-consumption).

**ROMDATA-01-GAMEPLAY-METADATA4-ROOTS / ROMDATA-02-GAMEPLAY-METADATA4-SOURCE-ORACLE
completed for the stated EUR/Wine prefixes:** 20 four-byte records /
80 bytes, 6e6bf4..6e6c44, two selector/value pairs, 26 active. Two literals and
an independent next root; parsed cross-owner indices with metadata52 pass.
A Wine check passes 320 real / 24 synthetic Count/Select/Gate/Threshold cases
for both original versions: all records, calling jobs 2..43, below/equal/above
thresholds, gaps, unused payload, value255, signed comparison and u8 tally.
Counting both slots and walking to the first zero remain distinct; live ability
scans and entire functions are not claimed or frozen.
9560 instructions / 136 PCs are coverage; no old oracle repeated.
The dataset remains e26-d25 / 21 files / 20 resources: preparation, direct
consumption, lifecycle, scene/CLI and new Linux/USA/native Windows checks remain open.
[Contract](../rom_data_gameplay.md#four-byte-two-pair-source-contract).

**ROMDATA-02-GAMEPLAY-METADATA4-DATA / ROMDATA-04-GAMEPLAY-METADATA4-CONSUMPTION
completed for the stated EUR/Wine scope:** e27-d26 adds linked-metadata4.json /
resource gameplay.linked-metadata4, 22 files / 21 resources; all 20 previous
decoded files/e26 preserved, USA e8-d7. Parsed metadata52 owner and validated
indices/selectors; typed bytes preserve counting/contiguous walking, live scans,
unsigned values/signed comparison/u8 tally and original clock/ABI.
Two Wine checks pass publication/failure/reuse/repair/movement/retention/
transactions/lifecycle and 56 native prefixes with 80 raw bytes altered and
equal state-clock results: 54 selector / 34 value bytes compiled; all bytes/edges/
mirrors/BIOS/SWI/DMA4 and the crossing of 2 metadata52 + 2 metadata4 bytes pass.
ROMDATA-05-GAMEPLAY-METADATA4-SCENE-CLI complete: CLI creates/reuses without changing
hash/mtime, old cache/different cwd; one 540-frame Change Jobs pair passes state/
clock/images/OAM/dispatch/audio with 32 new selector / 6 value bytes, Marsh/Thief
image inspected. Preserved input justified by the new family/new gate; e26 not
reopened. Source oracle 320+24 not repeated; prefixes not claimed as entire
functions. New Linux/USA/native Windows checks and the rest of the TODO remain
open; no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-metadata4-consumption).

**ROMDATA-01-GAMEPLAY-METADATA2-MAP-ROOTS / ROMDATA-02-GAMEPLAY-METADATA2-SOURCE-ORACLE
completed for the stated EUR/Wine scope:** 44 key/value pairs / 88 bytes at
6e6c44..6e6c9c, last real record 43 and the independent next 48B source.
Original forward truncates to u8 and reads index 0..43 without a local bound;
inverse truncates to u8, searches only 0..42 and returns the first stored key /
zero if absent. Parsed metadata52 owner/JSON/source/extent and 348 real + 13
synthetic Wine cases pass, including aliases, high byte, keys unequal to the
index and a match only in the last record excluded from the inverse search.
Live byte 7 wrapper tested through getter return with the frame retained; entire
functions, global bounds and value semantics are not claimed. Dataset
e27-d26/22 files/21 resources unchanged; preparation/consumption/lifecycle/
scene-CLI and new Linux/USA/native Windows checks remain to do. No performance/
space measurements, no old oracle repeated.
[Contract](../rom_data_gameplay.md#two-byte-forward-and-inverse-source-contract).

**ROMDATA-02-GAMEPLAY-METADATA2-DATA / ROMDATA-04-GAMEPLAY-METADATA2-CONSUMPTION
completed for the stated EUR/Wine scope:** e28-d27 adds linked-metadata2.json /
resource gameplay.linked-metadata2, 23 files/22 resources, all 21 previous files/
e27 preserved; USA e8-d7. Parsed metadata52 owner/authenticated key-value bytes
preserve original forward 44/inverse 43 instructions, alias/first stored key/
miss 0/u8 and live byte 7/clock/save/native ABI. Two Wine checks pass publication/
transactions/reuse/repair/movement/retention/lifecycle and 21 original/native
paths, 88 raw bytes altered/equal state-clock results, 7 key/235 value compiled,
all bytes/edges/mirrors/BIOS/SWI/DMA4/crossing 2 metadata4 + 2 metadata2.
Corrected build warning-free (after three missing zero arguments in old budget
tests); source 348+13 closed.
ROMDATA-05-GAMEPLAY-METADATA2-SCENE-CLI complete: CLI prepares/reuses without writes
with old cache/different cwd. ChangeJobs 540/Rumors 560/prefixes and Missions 620/
1260 are preserved negative locators; additional ordinary confirmations yield a
Pub 1840 pair with equal state/clock/images/OAM/dispatch/audio and 4 new value /
0 key B. Inverse key tested only in targeted tests; final Pub image with separate
native panels does not prove mission unit/outcome. No old oracle/full pair on the
negatives, no performance/space measurements. New Linux/USA/native Windows checks
and the rest of the TODO remain open.
[Contract](../rom_data_gameplay.md#managed-and-direct-metadata2-consumption).

**ROMDATA-01-GAMEPLAY-DEFAULT48-TEMPLATE / ROMDATA-02-GAMEPLAY-DEFAULT48-SOURCE-ORACLE
completed for the stated EUR/Wine scope:** 48 structural bytes at 6e6c9c..6e6ccc,
original root and copy length. Constructor mode 1/2, four IDs 20..23 with
alternate sources, distinct signed ID/selector >1/arguments/OR flag patches.
A Wine check passes 271 real + 30 synthetic cases with the original IWRAM
initializer, all 256 IDs, mode/truncation/signed boundaries/selector/flag and
preservation of unused bits/bytes. 48B roundtrip/JSON/array/source/stack/args/
callee-invalid/outerRAM pass; no old oracle repeated, 48812 instructions /
155 PCs are coverage only. Allocation/live unit after 127EB0 excluded; alternate
192B/ROM pointer and the following 240B not yet owned. Dataset e28-d27/23 files/
22 resources unchanged; preparation/consumption/lifecycle/scene CLI and new
Linux/USA/native Windows checks remain to do, no performance/space measurements.
[Contract](../rom_data_gameplay.md#default-constructor-template-source-contract).

**ROMDATA-02-GAMEPLAY-DEFAULT48-DATA / ROMDATA-04-GAMEPLAY-DEFAULT48-CONSUMPTION
completed for the stated EUR/Wine scope:** e29-d28 adds default-constructor-template.json /
resource gameplay.default-constructor-template, 24 files/23 resources, 22 previous
files/e28 preserved; USA e8-d7. Header/argument/halfword/flag/tail bytes preserve
original ROM/IWRAM copying, live patches, mode/alternatives/allocation/clock/save/
native ABI. Two Wine checks pass publication/transactions/lifecycle/retention and
16 original/interpreter/native prefixes with 48 raw B altered/equal state-clock
results; 8 alternate/rejected branches, 0 new reads, 39/21/272/16/32 B compiled,
all bytes/edges/BIOS/mirror/SWI/DMA4/crossing 2 metadata2 + 2 default48.
Build corrected and warning-free after a counter macro remained at 2 elements
instead of 5; initial log preserved. Source 271+30 not repeated.
ROMDATA-05-GAMEPLAY-DEFAULT48-SCENE-CLI complete: CLI creates/reuses without
writes using old cache/different cwd; one 397 Pub pair passes state/clock/images/
OAM/dispatch/audio with 96 new B = 10 header/6 argument/68 halfword/4 flag/8 tail,
image inspected. New family/gate justifies preserved input; no negative locator,
new RAM edit/recording or old oracle repeated. Allocation tail, alternate 192B/
ROM pointer and following 240B not yet owned. New Linux/USA/native Windows checks/
rest of TODO open; no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-default-constructor-template).

**ROMDATA-01-GAMEPLAY-CONSTRUCTOR-ALTERNATES statically completed:** 4 records
48B/192B at 6eca78..6ecb38 + 4B pointer at 70cd58, 196B total/no e29 overlap.
Root LDR, LDR+4, gated u8 IDs 20..23/stride 48/copy 48 and independent roundtrip
pass. Consumer CCF9C..CD046 reads five equipment halfwords 08..10 per record via
item properties 6/3; 20 links within 376 parsed items. Byte 01 and neighboring
header bytes are not assigned unproven semantics. Model/source oracle/preparation/
consumption/scene remain to do; EUR e29-d28 unchanged, default 271+30/native 16/
397 proofs closed.
[Contract](../rom_data_gameplay.md#next-rooted-constructor-sources).

**ROMDATA-02-GAMEPLAY-CONSTRUCTOR-ALTERNATES-SOURCE-ORACLE completed for EUR/Wine:**
strict model of 4 records/192B + 4B pointer/196B, IDs 20..23/offset/byte/halfword
fields and five equipment halfwords per record with 376-item parsed owner. A Wine
check passes 24 real + 8 synthetic cases of original copy/patch, u8 truncation,
selector/flag/mode, preserved high bit/unused/high item/OR and JSON/source/array/
order/ID/extent/stack/args/outerRAM. 5560 instructions/137 PCs are coverage only;
allocation tail excluded, synthetic cases not authenticated/published.
Default 271+30/native 16/397 not repeated. Dataset e29-d28/24 files/23 resources
unchanged; preparation/consumption/lifecycle/scene CLI and new Linux/USA/native
Windows checks remain to do, no performance/space measurements.
[Contract](../rom_data_gameplay.md#typed-constructor-alternates-source-proof).

**ROMDATA-02-GAMEPLAY-CONSTRUCTOR-ALTERNATES-DATA / ROMDATA-04-GAMEPLAY-CONSTRUCTOR-ALTERNATES-CONSUMPTION
completed for the stated EUR/Wine scope:** e30-d29 adds constructor-alternate-templates.json /
resource gameplay.constructor-alternate-templates, 25 files/24 resources, 23 previous
files/e29 preserved; USA e8-d7. Parsed Items owner, typed 4B pointer/192B records
and per-record mask preserve ROM/IWRAM copy/live patches/mode/allocation/clock/save/
native ABI. Two Wine checks pass publication/transactions/lifecycle/retention and
14 paths with 196 raw B altered/equal state-clock results, all 4 records mask 15/
pointer compiled, 5 default/rejected with 0 new B/old default positive. All bytes/
record edges/BIOS/mirror/SWI/DMA4 for both sources/crossing 2 typed + 2 raw pass.
Build warning-free; zero-use guard corrected in review before Wine, no failure.
Source 24+8 and old default 271+30/native 16/397 proofs closed.
ROMDATA-05-GAMEPLAY-CONSTRUCTOR-ALTERNATES-SCENE-CLI complete: CLI creates/reuses
without writes/old cache/different cwd. Intro 3000 school courtyard and Lutia 1700
NOTICE are preserved negative locators, no full pair. Normal confirmations/
deployment yield a 3600 Lutia pair with equal state/clock/images/OAM/dispatch/audio
and 45 new B: 4 pointer/6 header/3 arguments/28 halfword/2 flag/2 tail, mask 1
only ID 20. All 4 records and full copy remain targeted proofs; final image
inspected, no RAM edit/new recording/persisted save/old oracle repeated.
New Linux/USA/native Windows checks/neighboring headers/following 240B/rest of
TODO open, no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-constructor-alternates).

**ROMDATA-01-GAMEPLAY-METADATA14-ROOTS statically completed in the stated catalog:**
93 records/1302 physical B at 6e6f6c..6e7482, logical IDs 2..94/biased base 6e6f50.
Predicate >1 in the getters keeps 0/1 on the live job branch; independent next
root 6e7484 uses stride 12 and leaves 2B unowned. Halfword 4/6, byte 11/12/13
and packed palette 8..10 getters identified; roundtrip/no e30 overlap/186 values
01/02 within 116 metadata52 pass. Byte 02 copies to the live job with override 0;
the semantic owner of byte 01 is not proved by range alone. Hypothesis of 14B at
6e6dbc rejected (reader *4), alleged 7484/78d4 aliases rejected (stride 12).
Model/source oracle/consumption remain to do, e30 unchanged; no build/test/repeat
of old acceptance, new Linux/USA/native Windows checks remain to do.
[Contract](../rom_data_gameplay.md#next-bounded-fourteen-byte-records).

**ROMDATA-02-GAMEPLAY-METADATA14-SOURCE-ORACLE completed for the stated EUR/Wine scope:**
strict model of 93 records/1302B, IDs 2..94/biased base 6e6f50/physical
6e6f6c..6e7482, all byte/halfword/packed fields, parsed metadata52 owner for byte
02 only; byte 01 is not semantically constrained by range alone. A Wine check
passes 504 real cases: 471 getters/all 93 IDs/u8 truncation, 22 palettes with RAM
book 0/1/2/default 3/32B copy/512B guards and 11 initialization prefixes
CCF9C→CD0B6/override/flag/264B unit frame. 13 synthetic high u16/u8/bank 15/
unused/unproved byte 01/high valid job cases pass together with JSON/source/array/
bias/ID/order/count/owner/callee/stack/descriptor/unit/palette/outerRAM.
23627 instructions/380 PCs are coverage only, no old oracle repeated. Allocation
and global bounds excluded; 0/1/95 plus/2B padding unowned, no fallback. Dataset
e30-d29/25 files/24 resources unchanged; preparation/consumption/scene CLI/
lifecycle and new Linux/USA/native Windows checks remain to do, no performance/
space measurements.
[Contract](../rom_data_gameplay.md#typed-fourteen-byte-records-source-proof).

**ROMDATA-02-GAMEPLAY-METADATA14-DATA / ROMDATA-04-GAMEPLAY-METADATA14-CONSUMPTION /
ROMDATA-05-GAMEPLAY-METADATA14-SCENE-CLI completed for the stated EUR/Wine scope:**
e31-d30 adds linked-metadata14.json/gameplay.linked-metadata14, 26 files /
25 resources; all 24 previous files/e30 preserved, USA e8-d7. Parsed metadata52
owner only for byte 02; byte 01 has no semantic constraint. Typed projection of
1302B, counters and masks for the 93 records preserve original getters, RAM book,
palette copies, override, clock and native/save ABI. Two Wine checks pass
publication/reuse/repair/movement/retention/transaction guards and Load/Undo/
backup/restart with dataset copy unavailable. 37 original/interpreter/native paths
with 1302 raw B altered have identical state-clock results; IDs 2/20/28/94
positive, normal 0/1 without new reads. Bus/edge/width/mirror/BIOS/SWI/DMA and
typed/raw boundaries pass. Real CLI creates/reuses without writes and uses the old
cache from an unrelated cwd; one complete 397 Pub pair passes state/clock/images/
OAM/dispatch/audio with 27B/two records/all field groups, image inspected. Source
oracle 504+13 and previous proofs not repeated. Global bounds, palette payload,
allocation tails, adjacent families and new Linux/USA/native Windows checks remain
to do; no performance/space measurements or AndroidPort changes.
[Contract](../rom_data_gameplay.md#managed-and-direct-fourteen-byte-special-unit-metadata).

**ROMDATA-01-GAMEPLAY-METADATA12-ROOTS / ROMDATA-02-GAMEPLAY-METADATA12-SOURCE-ORACLE
completed for the stated EUR/Wine source scope:** two banks with stride 12 at
6e7484..6e78d4/92 records and 6e78d4..6e7994/16 records, total 108 records/1296B/
5184 bit pairs. Root/callee 135a1c/independent next source 6e7994 verified;
strict model of 12×4 groups with values 0..3 preserves 48 pairs even though loops
use 0..43. Original row32 and signed/wrapping slot indices, mode truncated to
u8, no new guest bound or fallback. A Wine check passes 1640 real/10 raw-outside-wrap/
384 synthetic cases, exact address/LDRB, JSON/source/callee/stack/RAM. Initial
build with test setup errors corrected, Windows rebuild warning-free; no Wine error.
EUR e31-d30 unchanged; preparation/consumption/lifecycle/scene, field meaning,
global bounds and new Linux/USA/native Windows checks remain to do. No performance/
space measurements or previous proof repeated.
[Contract](../rom_data_gameplay.md#indexed-twelve-byte-packed-pair-source-contract).

**ROMDATA-02-GAMEPLAY-METADATA12-DATA / ROMDATA-04-GAMEPLAY-METADATA12-CONSUMPTION /
ROMDATA-05-GAMEPLAY-METADATA12-SCENE-CLI completed for the stated EUR/Wine scope:**
e32-d31 adds linked-metadata12.json/gameplay.linked-metadata12, 27 files/26 resources,
25 previous files/e31 preserved; USA e8-d7. Typed two-bit fields, two source
owners, bank counters/108-record masks preserve original row32/slot signed-wrap/
mode u8/clock/ABI without invented semantic dependencies. Two Wine checks pass
publication/reuse/repair/movement/retention/transaction guards/Load-Undo-backup-
restart with dataset copy unavailable. 49 original/interpreter/native paths with
1296 raw B altered have identical state-clock results; both banks positive, three
external paths with 0 new reads. Bus/edge/width/mirror/BIOS/SWI/DMA and raw/typed/
between-bank boundaries pass. One positive 2200-Echo Grass confirmation pair for
state/clock/images/OAM/dispatch/audio/CLI completes the stated ordinary scope.
Source oracle 1640+10+384/e31 not repeated. Global bounds/flag semantics/separate
global bank/other families and new Linux/USA/native Windows checks remain to do;
no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-twelve-byte-packed-pair-banks).

**ROMDATA-01-GAMEPLAY-GLOBAL-PAIR-FLAGS / ROMDATA-02-GAMEPLAY-GLOBAL-PAIR-FLAGS-SOURCE-ORACLE
completed for the stated EUR/Wine source scope:** single root 6e7994, 44 pairs/11B
up to 6e799f from RAM loop 0..43/mode 0; no stride 12, 3B before independent root
6e79a2 unowned. Strict 11×4 model with values 0..3 preserves both bits. A Wine
check passes 102 real/10 raw-outside-wrap/91 synthetic cases, wrapper with ignored
r0/signed-wrap slot/mode u8/live mask filter/early exit/44 scan/JSON/source/callee/
stack/RAM. 13601 instructions/70 PCs are coverage only; no build/Wine failure or
previous proof repeated. EUR e32-d31 unchanged; preparation/consumption/lifecycle/
scene, ordinary high mode, semantics/padding and new Linux/USA/native Windows
checks remain to do.
[Contract](../rom_data_gameplay.md#global-packed-pair-flags-source-proof).

**ROMDATA-02-GAMEPLAY-GLOBAL-PAIR-FLAGS-DATA / ROMDATA-04-GAMEPLAY-GLOBAL-PAIR-FLAGS-CONSUMPTION
completed for EUR/Wine; ROMDATA-05 CLI concluded, ordinary global use remains to do:**
e33-d32 adds global-pair-flags.json/gameplay.global-pair-flags, 28 files/27 resources,
26 previous files/e32 preserved, USA e8-d7. Typed 11B/byte-position mask preserve
ignored r0/slot wrap/mode u8/live RAM/early exit/clock/ABI without invented owner/
padding. Two Wine checks pass publication/retention/transaction guards/Load-Undo-
backup-restart; 28 native/interpreter paths with 11 raw B altered have identical
state-clock results, 17B compiled/8 external paths with 0 new B. All 11B/bus/
mirror/BIOS/boundary 2 old 2 new/SWI 11 typed/DMA 3 typed 1 raw and 1 typed 3 raw
pass. CLI creates/reuses without writes, old cache/unrelated cwd; only scene
2200 Echo Grass negative (0 global/44 metadata12), no complete pair for it.
Positive ordinary global use and its complete pair are missing; reader 131db8
requires descriptor hword 4 and a nonzero live mask, concrete conditions are needed
before further replays. High mode/new Linux/USA/native Windows/3 adjacent B remain
to do. Oracles 102+10+91/e32 not repeated; no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-global-packed-pair-flags).

**ROMDATA-01-GAMEPLAY-NUMERIC-PAIR-ROOTS / ROMDATA-02-GAMEPLAY-EVALUATION-PARAMETERS-SOURCE-ORACLE
completed for the EUR/Wine source scope:** 38B in two sources: 7 divisor/weight
pairs and 2 conditional bonuses at 6e79a2..6e79b4; 5 threshold classes 1..5 with
stride 4 at 6e79c6..6e79da. Class 0 skips RNG/reading, 14B between sources and
4B row 0 unowned; 32/24B hypothesis rejected. Strict model of 7/2/5 arrays/ID/
offset/fields/opaque and a Wine check pass 132 original source cases (12 live score
prefixes, arithmetic/bonus/RNG/suffix fragments with injected arguments) and 52
mutated sources. First unsigned/u8 division, 6 signed 32-bit divisions, wrap, bonus low 8 ==1/
==0, original LCG/signed low 16 %256 threshold comparison preserved. JSON/source/
ID/order/offset/callee/stack/unit/context guards pass; no build/Wine failure or
previous proof repeated. 24147 instructions/442 PCs are coverage only. Zero
divisor is diagnostic None only/guest unchanged, not executed here; whole function
not claimed. e33 unchanged; preparation/consumption/lifecycle/scene/semantics and
new Linux/USA/native Windows checks remain to do, no performance/space measurements.
[Contract](../rom_data_gameplay.md#evaluation-parameters-source-proof).

**ROMDATA-02-GAMEPLAY-EVALUATION-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-EVALUATION-PARAMETERS-CONSUMPTION /
ROMDATA-05-GAMEPLAY-EVALUATION-PARAMETERS-SCENE-CLI completed for the stated EUR/Wine score scope:**
e34-d33 adds evaluation-parameters.json/gameplay.evaluation-parameters,
29 files/28 resources/27 previous files and e33 preserved, USA e8-d7. Typed 38B
and class/field counters preserve live producers/unsigned-u8 and signed divisions/
wrap/bonus/RNG/clock/ABI. Two Wine checks pass preparation/retention/transaction
guards/Load-Undo-backup-restart and 27 native/interpreter prefixes with 38 raw B
altered/identical state-clock results; class mask 31, class 0 zero, opaque 0
compiled. Bus/edge/mirror/BIOS/SWI 18-20 typed/DMA 2 typed 2 raw/18B gap-row 0
raw pass. Initial build with unused import warning rejected by wrapper; overlapping
native samples corrected before Wine, warning-free rebuild, no Wine error. CLI
creates/reuses without writes/old cache/unrelated cwd; first 1840 Pub mission
positive with 15 score B (7 divisors/7 weights/1 bonus), one complete state/clock/
images/OAM/dispatch/audio pair passes. The subsequent 400 signing/fee closes
class 1; classes 2..5 remain targeted-only, source 132+52/proofs e28/e33 not
repeated. Whole function/semantics/gap/ordinary global 11B/new Linux/USA/native
Windows remain to do; no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-evaluation-parameters).

**ROMDATA-01-GAMEPLAY-METADATA10-ROOTS completed within the stated static scope:**
root 6e79da/stride 10/default 0 and producer 020021b4 in 04B3E0..04B416 establish
0..99, 100 records/1000B up to 6e7dc2. Candidate 101 rejected; 12 zero B before
root 6e7dce unowned, 01A5A6 display is not evidence of a setter. Eight u8 deltas
are passed to consumer 04870c with live state/carry/wrap/cap 99/flag; byte 08 used
in Items price with optional +10 u8, signed division/unsigned minimum/zero→1,
byte 09 structurally unused. Roundtrip/no e34 overlap/root/stride/field opcodes
pass. Model and source oracle completed below. Preparation/consumption/scene/field
names/new Linux-USA checks remain to do.
[Contract](../rom_data_gameplay.md#next-ten-byte-progression-and-price-records).

**ROMDATA-02-GAMEPLAY-PROGRESSION-REWARDS-SOURCE-ORACLE completed for EUR/Wine:**
strict model of 100 records/1000B with 8 deltas/discount 08/opaque 09, without
constraints based only on observed values. Build 13783 warning-free; single Wine
62460 passes 362 real cases (7 publisher/232 caller+helper 04870c/123 full prices)
and 68 synthetic cases. Parsed Items authenticated, original mask/predicate,
u8 overflow/carry/cap 99/old invalid/flag, signed division/unsigned minimum/zero,
JSON/roundtrip/ID/order/source offset/SP/callee/EWRAM pass. Outside 100/255 remains
raw; synthetic cases not authenticated/published; 37 protected files unchanged.
Source closed, preparation/consumption/scene/new Linux-USA-native Windows checks
remain; no performance measurements.

**ROMDATA-02-GAMEPLAY-PROGRESSION-REWARDS-DATA / ROMDATA-04-GAMEPLAY-PROGRESSION-REWARDS-CONSUMPTION completed for EUR/Wine:**
e35-d34 adds progression-rewards.json/gameplay.progression-rewards,
30 files/29 resources, all 28 previous decoded files/e34 unchanged, USA e8-d7.
Build 84154 warning-free/Wine 80590 two targeted checks pass publication/reuse/
repair/move/coherent edits/transactional/load-undo-backup-restart. 27 original/
interpreter/native paths with 1000 raw B altered have identical state-clock results,
17 delta/4 discount/0 opaque compiled, 6 publisher/ungated/outside paths zero.
All 1000B/100 records/boundary/width/mirror/BIOS/SWI 32 typed/DMA 2 typed 2 raw/
crossing 2 eval 2 new pass. Source 362+68 closed/not repeated; CLI/scene completed
below, new Linux-USA-native Windows checks remain to do.
[Contract](../rom_data_gameplay.md#managed-and-direct-progression-rewards).

**ROMDATA-01-GAMEPLAY-PROGRESSION-GATES-ROOTS completed within the static scope:**
0487F6→048868/stride 12/index 0..63 establish 64 records/768B 6e7dce..6e80ce,
independent next root D23E6→D24A8 at the same end. Byte 00 unused, halfword 01
return, byte 03 output, 8 u8 minimums against 020021b6+2*k; original u64 mask/
first eligible unmarked row order/physical bit/zero return with bit 63 success
remain distinct from a miss. All 768B roundtrip/no e35 overlap/root/field/stride/
count/caller/next opcodes pass; current returns 0..375 compatible with Items,
complete receipt/FK not proved. Inventory alone did not add model/build/Wine;
source oracle completed below, managed/runtime/new Linux-USA-native Windows remain.
[Contract](../rom_data_gameplay.md#next-progression-gates-and-returned-receipts).

**ROMDATA-02-GAMEPLAY-PROGRESSION-GATES-SOURCE-ORACLE completed for EUR/Wine:**
strict model of 64 records/768B byte 00/halfword 01/byte 03/8 minimums, no
byte 00==ID/minimum<=50/receipt==item constraint. Outcome distinguishes eligible
ID/value/output/mask. Build 75099 warning-free/single Wine 11775 passes 80 real +
25 synthetic cases, complete original 0487c8 + u64 helper: all 64 rows/equal-below
8 dimensions/unsigned 128/bits 31-32-63/first eligible order/full mask/row 63
success value 0; 704B fields read, 768B roundtrip/JSON/ID-offset-order/coherent
edit/SP/callee/EWRAM pass. 38 protected files unchanged, e35 unchanged; managed/
runtime/lifecycle/ordinary use/receipt UI-FK/new Linux-USA-native Windows remain
to do. No old test repeated.
[Contract](../rom_data_gameplay.md#progression-gates-source-contract).

**ROMDATA-02-GAMEPLAY-PROGRESSION-GATES-DATA / ROMDATA-04-GAMEPLAY-PROGRESSION-GATES-CONSUMPTION completed for EUR/Wine:**
e36-d35 adds progression-gates.json/gameplay.progression-gates, 31 files/
30 resources/all 29 previous decoded files/e35 retained/USA e8-d7. Windows 68535
warning-free/Wine 97635 two checks pass publication/interruption/create-reuse-
repair-move/retention/coherent requirement-return-output-unused-lastrow-reorder-
source-dependency-path-role/transactional owner state/load-undo-backup-restart.
19 original/interpreter/native paths with 768 raw B altered/identical state-clock;
compiled 369 requirements/18 return/9 output/0 unused, 2 successes with value 0
mark a bit, 2 full mask zero. All 768B/64 records/boundary/width/mirror/BIOS/
two pages/SWI cross-page 32 typed/DMA 2 typed 2 raw/12B raw gap/old reward 2 new
gate 4 coexistence pass. Source 80+25 closed/not repeated. CLI/scene 05 completed
below; receipt UI-FK/new Linux-USA-native Windows remain, no performance measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-progression-gates).

**ROMDATA-02-OCTONARY-COMPACT-OBJ completed for the declared EUR/Wine source:**
caller-witnessed selectors 0/1/2 at `6cc124` cover 21 frames, five lists/four
objects and 102 contiguous bytes, including the empty list. Source geometry,
lossless JSON, original reader prefixes and modification rejection pass on
Wine. This does not establish a global selector cap or ordinary use; Linux/USA
remain pending. [Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6cc124).

<a id="romdata-03--preparazione-automatica-e-riuso-allavvio"></a>

## ROMDATA-03 — Automatic preparation and reuse at startup

**Dependencies:** ROMDATA-02 for the first family.

- [x] Integrate preparation after ROM selection/verification and before game
  startup: extract when the dataset is missing or version changes; reuse when
  identity and contents are valid.
- [x] Define paths through existing installed/portable storage. Proposal to
  confirm: `Data/resources/<rom-hash>/<schema>/` in portable mode; follow
  [relocation rules](../portable.md). Keep regenerable base data, future user
  mods, native caches and saves separate.
- [x] Write to a temporary directory, validate and publish only a complete
  dataset; handle cancellation, insufficient space, concurrent startup and
  crash without replacing the last valid version with a partial result.
- [x] Show understandable progress and errors; support regeneration of missing/
  corrupted data from the source ROM without overwriting mods or saves.
- [x] Verify first startup, restart without re-extraction, ROM/region change,
  decoder update and portable-folder relocation. Define how to detect a changed
  source before reusing its dataset.

**Closure:** first startup prepares resources; subsequent starts reuse a valid
dataset. Extraction remains local and does not modify the source ROM.

**ROMDATA-03-OBJ-STARTUP completed in the first family's opt-in path:**
`rom-data prepare` and `play --rom-data`, immutable generations, atomic
selection, OS lock, reuse/verification and recovery. The checks above concern
`graphics.obj-dimensions`; other families are not implicitly migrated. Linux
and Windows/Wine tests pass, including forced stops and two GUI starts before/
after portable relocation. Space exhaustion simulated; physical power loss and
native Windows unverified. Details in [decoded resources](../rom_data.md).
Direct consumption is now connected in 04; normal enablement awaits gate 05.

<a id="romdata-04--uso-diretto-dei-dati-estratti-nel-runtime"></a>

## ROMDATA-04 — Direct runtime use of extracted data

**Dependencies:** ROMDATA-01/02/03. **Core of the main objective.**

- [x] Introduce resource access through IDs and typed structures, with loading/
  preloading from decoded files. Define memory budget, data lifetime and
  invalidation; avoid I/O in audio callbacks and per-pixel or per-instruction paths.
- [ ] For each family follow all consumers: translated native code, interpreter,
  CPU/DMA reads and host paths such as rendering and audio. Identify where the
  game requires raw bytes, decompresses, copies or interprets resources.
- [x] Connect the first family to the loader and replace consumption of the
  corresponding ROM resource with extracted data. If data must be transferred to
  guest RAM/VRAM, preserve it with correct layout, registers and observable
  effects; do not apply generic lookups to every bus read.
- [ ] When replacing guest routines, preserve cycles, IRQ, DMA, RNG and event
  order. Demonstrate how guest time is retained if decompression is moved earlier
  to extraction: skipping it without accounting for its effects is not equivalent.
- [x] Distinguish base ROM, dataset and compiled code identity. Review embedded
  constants, hooks and native caches: no library may reuse data or assumptions
  incompatible with the dataset actually loaded.
- [x] Add provenance diagnostics and use/fallback counters per family. In tests
  of migrated resources make access to the old path detectable; also verify
  missing files are not silently ignored in favor of the ROM.
- [ ] Progressively migrate all inventoried families. Record code, still-raw
  data and remaining ROM reads separately; temporary fallback preserves
  compatibility but leaves the corresponding migration open.

**Closure per family:** the game uses the decoded file and provenance is
verified. **Overall closure:** all data families planned in the matrix are
migrated and verified; justified exclusions are declared. One working example
is a partial delivery, not completion.

**ROMDATA-04-OBJ-CONSUMPTION completed for implementation and initial checks:**
the first family is read from files in `play/frames/runc --rom-data`; fixed
projections for CPU/DMA/native APIs, typed host reads, modified RAM preserved,
counters and nonserialized resources retained through Load/Undo/backup. Measured
budget 432 B; no instruction or decompression skipped. EUR host reader also
fixed by verifying ROM source against original instructions. Extractor 2 records
the additional EUR alias and retains old generations. Details and evidence in
[runtime consumption](../rom_data_runtime.md).
Checked items cover infrastructure and the first family, not others. Original
routine matrix covers 48 EUR/USA cases with actual guest reads; 600-frame native
comparison proves host use and state/image/audio parity without attributing guest
reads not executed on that initial screen. New tranche 05 verifies EUR scenes/
transition and local costs described below; USA trace is now verified separately
as described below. Other gates and families remain open.

**ROMDATA-04-BG-CONSUMPTION completed for metadata and initial checks:** 162
authenticated descriptors retained as immutable typed data; separate CPU/DMA/
native API projections and BG counters. Schema/extractor/decoder 1/3/2, ROM/
save/native identity and ABI unchanged. Measured 440 B shared/OBJ + 28672 B BG,
within 1 KiB/64 KiB limits declared before tests. Linux/Wine 204 core and 98/97
frontend tests pass; four regional cases of 2608 calls compare original,
interpreter and TinyCC even with source ROM in memory altered after activation.
3867 native BG reads per case; Load/Undo/backup and missing file verified. Two
native EUR 1200-frame startups match for all states/images/OAM/audio, with 53
actual BG reads. [Evidence and limits](../rom_data_graphics.md#direct-descriptor-consumption).
Scenes/GUI/costs 05 BG, linked assets/programs and other families remain open.

**ROMDATA-04-BG-BASE-PALETTE-CONSUMPTION completed for base catalog:** 162
palettes retained in immutable owner; decoded banks for HLE LZ77 and exact
source representation for CPU/DMA/native APIs/guest BIOS. Revision 1/6/5, e5
retained; writes, clock and ROM/save/native identity unchanged. Linux/Wine 210
core and 101/100 frontend tests pass. Four original/native cases of 1296 calls/
72792 instructions match even when altering all old ROM intervals after
activation; 1272 LZ77 calls read banks per case. Load/Undo/backup/removed files
pass. New native EUR 1200-frame startups match for states/images/OAM/audio with
one 160 B palette actually read from files; CLI e5 refusal/retention and
interpreter/native pairs pass. Owner 123833 B on both targets; no space optimization.
[Evidence and limits](../rom_data_graphics.md#direct-base-palette-consumption).
Alternate programs, other families/languages and further gate 05 checks remain open.

**ROMDATA-04-BG-ALTERNATE-PALETTE-CONSUMPTION completed for valid catalog:**
immutable owner and CPU/DMA/native reads from decoded banks for literals, typed
links and exact control metadata. No instruction/copy/cycle skipped, gameplay I/O
or schema/native cache change; dataset still e7-d6. Linux 216 core/104 runtime
and Wine 216/103 pass, plus four regional original/native producer checks with
poisoned ROM data: 1256 calls/case, equal full state and clock, positive bank use
and separate literal-byte evidence via native API. Load/Undo/backup/restart
retain owner/counters even with files removed. Actual alternate scenes, FF
prerequisites, GUI/archives and native Windows remain; no performance measurement.
[Contract and evidence](../rom_data_graphics.md#direct-alternate-palette-consumption).

**ROMDATA-04-BG-PACKET-CONSUMPTION completed for reduced EUR/Wine scope:** 352
typed programs retained in immutable owner; raw/literal LZ77 values from decoded
words, typed body for HLE, exact original checks. CPU/DMA/native API reads and
EUR page 895 connected; mixed palette/packet reads compose correctly. Dynamic
initialization, parents, writes and guest clock retained; schema 1/8/7 and save/
native identity unchanged. Three targeted Wine tests pass: synthetic reads/
SWI/DMA/owner, nine original/native EUR calls with altered packet ROM intervals
and equal full state, Load/Undo/backup/restart with temporary file copies made
unavailable. 18 typed bytes read by native instructions and 10506 HLE bytes.
USA remains verified-only; new Linux/USA checks pending, no benchmark or broad suite.
[Contract and evidence](../rom_data_graphics.md#direct-european-packet-consumption).

**ROMDATA-05-BG-PACKET-SCENE completed for one EUR/Wine transition:** retained
slot 5 LCD B/input 397 frames, entering Cyril pub BG62. Every full state/clock,
image/OAM/dispatch and audio sample matches; three HLE calls consume 4720 body/
2640 typed bytes. The 12 wrapper reads contain no raw values: native 04 evidence
covers that path separately. Other scenes, GUI/archives, new Linux/USA checks
and native Windows remain open.

**ROMDATA-04-BG-GRAPHICS-CONSUMPTION completed for reduced EUR/Wine scope:**
immutable words of seventh file e9 and literal maps of original codec; CPU/DMA/
native API reads, exact header/controls and EUR pages 769..895. No guest
instruction/write/cycle skipped. Three targeted Wine tests pass: synthetic with
altered duplicate sources/literals, four original/native calls with all 50 ROM
sources altered and equal full state/clock, owner retained across Load/Undo/
backup/restart with only temporary copies unavailable. 29023 typed bytes read
by native instructions. Schema 1/9/8 and save/native identity unchanged.
Linux/USA pending, no performance measurement.
[Contract and evidence](../rom_data_graphics.md#direct-european-main-graphics-consumption).

**ROMDATA-05-BG-GRAPHICS-SCENE completed for one EUR/Wine transition:** same
retained slot 5 LCD B/input 397 frames, entering Cyril pub BG62. Every full
state/clock, image/OAM/dispatch and audio sample matches; 14815 graphics reads/
16209 source bytes/10334 decoded literal bytes. Other scenes/GUI/archives, new
Linux/USA checks and native Windows remain open.

**ROMDATA-04-BG-TRANSFER-CONSUMPTION completed for reduced EUR/Wine scope:**
file e10 controls/tiles retained as immutable structures; typed CPU/DMA/native
reads on pages 778..897 and separate counters. Countdown/flags/DMA progress/
guest clock unchanged. Three targeted Wine tests pass, including two original/
native cases with all 76 ROM sources altered, equal full state after 60 calls,
66 compiled control bytes and 9984 tile bytes via native DMA. Load/Undo/backup/
restart retain owner with temporary copies unavailable. Schema 1/10/9 and save/
native identity unchanged. Linux/USA pending.
[Contract and evidence](../rom_data_graphics.md#direct-european-transfer-consumption).

**ROMDATA-05-BG-TRANSFER-SCENE/CLI completed for declared EUR/Wine scope:** LCD B
397-frame transition/Cyril pub BG62 with every state/clock/image/OAM/dispatch/
audio equal, 1203 control/21504 actual tile bytes. A new decoded CLI reuses e10
without writes and the native cache, same result as retained legacy reference
without repeating it. Other scenes/GUI/archives, Linux/USA and native Windows
remain open; no performance measurement.


**ROMDATA-04-OCTONARY-COMPACT-OBJ completed for combined EUR/Wine substeps:**
e108-d107 adds the 102-byte typed bank and preserves all 97 previous resources.
Owned bytes, mixed boundaries, transactional rejection and original native/
interpreter state-clock equality pass for selectors 0/1/2 plus a retained-input
continuation for selector 256. That selector reads a raw zero offset returning
to two owned count bytes; no guest cap or all-raw fallback is inferred. The
initial full filter failed on the counter oracle; a later complete e127-d126
filter passed with the corrected oracle, 100 resources and 97 prior resources
preserved. That run's JSON summary has a stale fixed resources=98 field; a
later corrected full-filter rerun reports the actual 100 from its manifest.
[Full-filter recovery](../changes/2026-11-04-verify-e127-octonary-full-filter.json).
The e108 production CLI create/reuse/startup also passes: 99 files match runtime,
98 identities and 101 managed files remain unchanged. Neutral new-bank counters are zero;
the later ordinary replay is recorded below. Linux/USA remain pending, with no
measurements.
**ROMDATA-04-OCTONARY-ORDINARY completed for one positive EUR/Wine replay:**
normal IN-19 Marche Blue Magic→Roulette→Do it in Mage Tourney serves 4,744
new-bank reads/5,831 bytes. Existing native raw/managed verification passes all
600 frames, 11960..12560 with full state/clock/images/OAM/dispatch/five audio
streams. Canonical outputs/current dataset reused; inputs unchanged, no build
or extraction. Scratch auto-retired, 45 superseded navigation copies retired
with evidence and one current seed retained. Animation completion, ordinary
selector/list completeness and global domains remain unproved; Linux/USA and
native Windows pending, no benchmark.
[Evidence](../rom_data_obj.md#ordinary-roulette-producer-and-eur-validation).

**ROMDATA-01/02/03/04/05-UI-CONTEXT-TEMPLATES completed for scoped EUR/Wine:**
source-backed category selection and explicit 20-byte copy define eleven UI
records/220 bytes at 394e44, without a global bound or guest cap. e109-d108
adds the typed file/runtime owner and field counters; all 98 earlier resources
and bytes stay identical. Warning-free canonical Windows builds, one-generation
CLI create/reuse and focused Wine checks pass: 256 original categories,
eleven copy plus eleven coordinate native/raw state-clock prefixes, typed
boundaries and coherent rejection in memory (no dataset copies). One ordinary
11960..11972 CLI/native raw-managed pair serves five new-owner reads/20 bytes
with full state/clock/images/OAM/dispatch/five-stream audio equality. Ordinary
all-record/all-consumer coverage, global UI extent and Linux/USA/native Windows
remain open. Superseded e108 payload retired with evidence/current inputs
preserved; no benchmark.
[Contract](../rom_data_ui.md#category-selected-ui-context-templates).

**ROMDATA-01/02/03/04-COMPACT-CATALOG completed for scoped EUR/Wine:**
one sparse typed resource covers 29 new literal-root banks/38 selectors,
207 frames/160 list identities/888 objects, 6,650 bytes in 65 spans. The
118-reader retained-census query and 37 original windows prove the declared
projection; gaps/unwitnessed selectors/global CFG remain unproved. e110-d109
publishes 100 resources/101 files and preserves all 99 prior payloads. Focused
Wine source/bytes/mixed-boundaries/in-memory rejection passes; all new reader
fields and 67 native/interpreter state-clock cases pass, including actual
out-of-projection owned intersections. CLI create/reuse/startup recognizes
100 identities without managed writes. Its 12-frame new counters are zero;
A later source-backed IN-08 Star Cross case serves 15,830 catalog reads/19,330
source bytes and passes one 480-frame Wine native raw/managed complete-state,
clock, render/OAM/dispatch/five-stream audio pair. Physical human Illusionist9
and raw rosterunit6 are independently bound to the normal deployment/menu.
`ROMDATA-04-COMPACT-CATALOG-ORDINARY` retains complete-bank/selector/consumer
coverage as open; this single case does not complete the animation or global
catalog. Canonical builds/current dataset reused; successful scratch retired,
current seed/evidence/failures kept. Linux/USA/native Windows pending; no benchmark.
[Contract](../rom_data_compact_catalog.md).

**ROMDATA-01-E110-RESIDUAL-COMPACT-SOURCES / ROMDATA-02-04-E111-LOOP-COMPACT-CATALOG
completed for scoped EUR/Wine:** five explicit original byte-counter loops prove
43 selectors for roots 6c56a8/6c4c50/6c5800/6c88a8/6c894c. Fourteen original
Windows windows and all 19 BL/index-preservation flows pass. Sparse geometry
adds 60 frames/44 lists/81 objects/974 bytes without current-owner overlap.
The existing resource reaches 34 banks/81 selectors/7,624 bytes, e111-d110;
100 resources/101 files, 99 prior payloads and 29 prior banks unchanged.
One new-only Wine filter passes poisoned bytes/mixed boundaries/in-memory
rejection and 48 native state-clock cases, serving 180 typed bytes including
eight actual outside-projection intersections. CLI create/reuse and 12-frame
startup pass. Two live-source CLI guards reject concurrent debug edits; a
stable disposable source snapshot builds cleanly and is removed. The e110
comparison is retired after acceptance, with real inputs/evidence retained.
Ordinary use of these five roots, 29 residual census roots, global domains,
Linux/USA/native Windows and remaining ROM/mod phases stay open. No old reader
or ordinary proof repeated, no benchmark or dataset test copy.
[Contract](../rom_data_compact_catalog.md#finite-loop-european-source-extension).

<a id="romdata-05--equivalenza-salvataggi-e-prestazioni"></a>

## ROMDATA-05 — Equivalence, saves and performance

**ROMDATA-04-TEXT-CATALOG-CONSUMPTION completed for the stated EUR/Wine scope:**
complete model from the tenth file, typed CPU/DMA/native reads for pointers,
strings, relative indices, headers, programs, glyphs/metrics; LZ/RLE literals
from decoded units, exact original checks. Three Wine checks pass: synthetic with
altered duplicate sources/literals, 15 original/native chains with 33581 altered
ROM ranges and 90 equal states/clock results, Load/Undo/backup/restart with file
copies unavailable. Positive compiled use in all 8 types; explicit valid dynamic
context, no ordinary scene inferred. Old subset authenticated without double owner/
counters. e12-d11/save/native ABI unchanged. Scenes/CLI 05, other roots/fonts/
Unicode, Linux/USA remain to do.
[Evidence](../rom_data_text.md#broad-direct-consumption).

**ROMDATA-04-TEXT-SHORT-CONSUMPTION completed for the stated EUR/Wine scope:**
immutable owner with indices/codes/glyphs/metrics from the ninth file, fixed
source pages and separate counters. Three Wine checks pass: synthetic widths/
mirror/edges/SWI/DMA/Load, five original/native chains with all 670 ROM ranges
altered and 20 equal calls/states/clock results, Load/Undo/backup/restart with
temporary file copies unavailable. Native APIs read 20 table bytes/124 strings/
636 glyphs/6 metrics as typed data; fixed mode skips metrics like the original.
e11-d10/save/native ABI unchanged. Ordinary gameplay scenes, other catalogs/fonts/
Unicode/commands, Linux/USA remain to do. [Evidence](../rom_data_text.md#direct-consumption).

**Dependencies:** each ROMDATA-04 integration, before normal enablement.

- [ ] Compare original and unmodified extracted dataset with the same input:
  guest state at end of frame, RNG, clock, RAM/VRAM/OAM, images and deterministic
  audio. Cover relevant intro, map, battle, menu and transitions; reuse valid
  existing evidence without confusing coverage with completeness.
- [ ] Verify native and interpreter paths, Fast, pause, load/Undo and backup
  restoration. Rebuild host resources and caches after loading without serializing
  pointers or duplicating all extracted files in machine states.
- [ ] Define identity and compatibility of `.ffstate` and `.sav`: the equivalent
  base dataset should preserve existing states where possible. If layout or
  semantics change, version and reject incompatible combinations before changing
  the machine or writing the save.
- [ ] **Deferred by the user:** separately measure initial extraction, startup with
  a ready dataset, disk space, RAM and frame/loading/audio times against the current
  in-memory path. Set regression limits before comparison. Also record a neutral
  or worse result: extraction does not prove a speedup.
- [ ] Verify native Windows; treat Windows/Wine and Linux as separate evidence.
  Update launcher/documentation/packager without including ROMs, extracted
  resources, personal saves or generated code in repositories and distributed packages.

**Main deliverable:** automatic extraction, startup reuse and direct consumption
verified for the stated matrix, equivalence without mods and remaining dependencies
documented. Cost measurements are deferred and do not block functional work. The
following mod phases do not block this deliverable.

**ROMDATA-05-OBJ-SCENES-COSTS completed within the stated local scope:**
[protocol and results](ROM_DATA_VALIDATION.md),
[record](../changes/2026-10-21-validate-decoded-obj-scenes-and-costs.json).
Linux and Windows/Wine comparisons on map 240, battle/menu 3530 and startup to
battle 8476 frames: every full state, image/OAM, count and audio stream matches;
actual EUR alias read at frame 5400. Base state and save compatibility preserved,
without a new version. Five Linux pairs against the corrected legacy path pass the
limits set before measurement: +1.0081%/-0.2355% mean cost; neutral result. Seven
first starts/reuses: medians 32.416/21.019ms Linux and 109.222/91.891ms Wine;
maximum Wine reuse 183.130ms retained, with a median gate. Linux RSS and disk space
meet budgets. Synthetic tar/zip archive test excludes private data and includes the
new instructions.

**ROMDATA-05-OBJ-GUI completed within the EUR Linux/Windows-Wine scope:**
Fast held and toggled, pause/resume while retaining Fast, return to 1x, Load/Undo/
reload and backup restoration from the menu pass with resource use, inspected
captures, real 44.1/48kHz callbacks and unchanged protected files.
[Record and limits](../changes/2026-10-21-validate-decoded-resource-gui-lifecycle.json).
Final gain is 0: no proof of audibility or callback A/B performance.
**ROMDATA-05-OBJ-ARCHIVES completed within the stated scope:** corrected Docker
copies/allowlist to include guides required by the packager; real archives passed
395 Linux/392 Windows-Wine tests (24 optional tests ignored each). Privacy, PE/ELF,
automatic TinyCC and GUI of the actually extracted runtime with new portable
preparation/Load/Undo/reload pass.
[Record and limits](../changes/2026-10-21-validate-decoded-resource-portable-archives.json).
Previous archives/binaries unchanged; one BUILD-INFO metadata change explicitly
recorded.

**ROMDATA-05-OBJ-USA-INITIALIZER completed on Linux and Windows/Wine:** clean startup
without saves or forced PC through the original USA copier, frame 791. All
13 instructions match in full state; CpuSet reads 24 halfwords from the file and
correctly copies 3648 bytes, including 48 metadata bytes. The test with only the
old alias altered after authentic activation also passes.
[Evidence and limits](ROM_DATA_VALIDATION.md#original-usa-initializer).
This is not a complete USA scene/audio verification. ROMDATA-05 remains partial
for native Windows, unavailable to the user: Linux/Wine only for now. Normal
enablement and other families/mods remain open.

**ROMDATA-05-BG-SCENES-COSTS partial:** full Linux/Wine parity on map 240,
battle/menu 3530 and transition 8476 frames; 53 BG reads in the transition. Five
Linux pairs pass the frame limits (+1.3600%/-0.6017% median mean), as do startup/
reuse and disk space. Peak RSS exceeds the 2MiB limit: +2864KiB median/+2944KiB
maximum. Result retained, no normal enablement. Attribution identifies TOML
preparation/parsing; alternate and inline TOML prototypes rejected. The JSON
prototype identified direct parsing with 22528B of heap; the subsequent revision
adopts readable standard JSON, verified in the runtime, following the updated user
preference below.
[Protocol and results](ROM_DATA_VALIDATION.md#bg-metadata-e3-results-and-rss-failure).
Completed scene/timing checks must not be repeated to reconstruct context; RSS,
BG GUI/archives and native Windows remain open.

**ROMDATA-02-BG-JSON completed:** revision 1/4/3, BG in readable standard JSON
with direct parsing into typed structures; manifest/OBJ remain TOML. Previous e3
generations retained; authentication, input guards, reuse/repair and native/
interpreter consumption pass on Linux/Wine.
[Evidence](../rom_data_graphics.md#readable-json-datasets).
**Updated preference:** no work to compress or save space in extracted data. The
149261 BG bytes/150818 dataset bytes are accepted for readability; parser limits
protect against malformed input. Cost and prototype results remain informative,
without new cycles to meet the former disk limit. Next priority: payloads and
catalogs still in ROM.

**ROMDATA-05-BG-BASE-PALETTE-SCENES completed for the stated EUR scenes:**
new comparisons with dataset e6 pass 240 map frames, 3530 battle/menu and 8476
transition on Linux/Wine. All full states, images/OAM, dispatch and audio streams
match. Stable scenes do not load palettes; the transition reads a decoded 160B
palette and 53 BG descriptors. Previous fixtures and caches reused and protected.
[Results](ROM_DATA_VALIDATION.md#base-palette-e6-scene-acceptance).
Costs/GUI/archives and native Windows remain separate gates; neither normal
enablement nor migration of other families is inferred from these scenes.

**ROMDATA-05-BG-ALTERNATE-SCENES / GUI completed within the stated EUR scope:**
LCD B/TV confirmed in the original menu, entry to Cyril's pub with BG62. Four
native 397-frame Linux/Wine pairs match for every full state, center/wide image,
OAM, dispatch and all nonempty audio streams. Positive alternate use: LCD B 165
literal bytes/2 links, TV 153/2. Native Linux/Wine GUIs present both modes and
exit 0 with 318 literal bytes/4 links. Snapshot/ROM/cache/dataset/canonical files
protected; no forced RAM/PC or performance measurement. Broader scenes/regions/
languages, global FF, archives and native Windows remain.
[Evidence](ROM_DATA_VALIDATION.md#alternate-palette-e7-scene-and-gui-acceptance).

**ROMDATA-05-GAMEPLAY-PROGRESSION-REWARDS-SCENE-CLI completed within the EUR/Wine scope:**
production 33726 warning-free/canonical restored; CLI 25924 create/reuse/no-write/
old cache/unrelated cwd pass, 32 managed files unchanged by hash/mtime. First
600 town→Shop→Buy positive with 7 discount B/record 4 (mask 16), one Wine 52469
pair has identical state/clock/image/OAM/dispatch/audio throughout. Shop background
inspected; separate inventory panel not visible in the PPM, selection/transaction
not inferred. Ordinary delta remains targeted-only. No negative locator, new recording,
RAM edit or save writes; 362+68/27 native proofs not repeated. New Linux-USA-native
Windows checks/whole caller/field semantics remain to do. No performance/space
measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-progression-rewards).

**ROMDATA-05-GAMEPLAY-PROGRESSION-GATES-SCENE-CLI completed within the stated EUR/Wine scope:**
production 65421 warning-free/canonical restored; CLI 70968 prepare/reuse-no-write/
old cache/unrelated cwd/1 frame battle compatibility pass, 33 files unchanged by
hash-mtime, zero new use not mistaken for a positive scene. Static locator of 174
EUR headers/49 representative payloads has no eligible flags, no false ordinary
proof. Single GUI Wine 92614 normal Mad Alchemist 10 days/Sigmund/fee 600/day
9→19 with necessary new UI checkpoints 2-3-4 (no payload edit/personal save);
Mission Cleared/rewards/2 skill level-ups inspected, 209 requirements/8 delta/
1 threshold in GUI. New 1000 CLI 29523 positive 209/8; single Wine 57387 full
1000 pair has identical state-clock-images-OAM-dispatch-audio. E35 row 0 eight
ordinary deltas closed; e36 ordinary return/output targeted-only, GUI threshold 1
occurs before slot 3; the subsequent 400-frame Yes/fee pair below closes class 1.
Shared test exe changed by parallel Debug work: e36 verifier unchanged, pinned copy
9e5e6f0c with that parallel compiled code; no false identity with the old 19 native.
Old source/native/lifecycle proofs not repeated, no full pair on negative coverage.
New Linux/USA/native Windows/receipt UI-FK remain, no performance/space/Android.
[Contract](../rom_data_gameplay.md#ordinary-progression-gates-dispatch-completion).

**ROMDATA-05-GAMEPLAY-EVALUATION-THRESHOLD-ACCEPT-SCENE completed for EUR/Wine class 1:**
single GUI 17436 saves normally in slot 2 before Yes, confirms 600 gil/funds 7109
and exits 0; no payload edit or personal save, originals/copy 1 protected. GUI
stopped at the fee reads 0 thresholds: the effective boundary includes confirmation
and fee acknowledgement. First CLI 70012 400 frames positive with 16 evaluation B
(7 divisor/7 weight/1 bonus/1 threshold), mask 1. Single Wine 96264 full 400 pair
has identical state-clock-images-OAM-dispatch-all audio, same threshold 1.
Binaries 46c8cb78/9e5e6f0c and dataset/cache reused; no build/132+52/native 27/
1840 score/1000 completion repeated. Classes 2..5/whole caller/new Linux/USA/native
Windows remain to do; no gates/rewards use or outcome inferred.
[Evidence](../rom_data_gameplay.md#ordinary-evaluation-threshold-acceptance).

**ROMDATA-01-GAMEPLAY-PROGRESSION-RANK-GATES-ROOTS / ROMDATA-02-GAMEPLAY-PROGRESSION-SELECTION-GATES-SOURCE-ORACLE completed for the EUR/Wine source scope:**
64 records ×16/1024B at 6e80de..6e84de from D2390/shift 4/indices 1..64;
row 0/16B unowned, independent next root 6e84e0 leaves 2B. Unsigned minimums
01/02/04 all required; 06..0d requires ANY active threshold, original flag 0e,
first eligible writes selector 00. With no conditions it writes 0 and stops;
exhausting 64 does not write; row 52 sentinel/53..64 storage. Original low 8/
input reset and count/flag helpers, no selector==ID schema. Strict source-only
model/1024B roundtrip and single warning-free build/Wine oracle 82907 pass 138 real
+ 80 synthetic full D2390/reset-count-flag cases: 831 original field B/ANY/
unsigned/zero-terminal-vs-exhausted-no-store/full RAM-SP-callee/JSON-source guards.
39 protected inputs unchanged; no old proof repeated. E36 unchanged, publication/
runtime/lifecycle/scene/new Linux-USA checks remain to do.
[Contract](../rom_data_gameplay.md#progression-selection-gates-source-contract).

**ROMDATA-02-GAMEPLAY-PROGRESSION-SELECTION-GATES-DATA / ROMDATA-04-GAMEPLAY-PROGRESSION-SELECTION-GATES-CONSUMPTION completed for EUR/Wine:**
e37-d36 adds progression-selection-gates.json/gameplay.progression-selection-gates,
32 files/31 resources/all 30 previous/e36 retained/USA e8-d7. Two Wine checks
73517 pass publication/retention/coherent guards/binding-source-role-path-hash/
transactions/Load-Undo-backup-restart and 18 full D2390 native-interpreter paths
with 1024 raw B altered/identical state-clock results. Compiled selector 15/min01
282/min02 505/min04 462/ANY 1635/flag 162; all 64 mask/all bytes-boundaries-mirror-
BIOS/row 0 raw 16 + gap raw 2/SWI 32 typed each/old gate 2-raw 16-new 2/DMA 2
typed 2 raw. First 48254 compile E0061 budget fixture 32 argument corrected,
77778 warning-free; no Wine failure. Source 138+80 closed/not repeated, helper
instructions/clock/ABI unchanged. Production CLI/ordinary scene/new Linux-USA-
native Windows remain to do.
[Contract](../rom_data_gameplay.md#managed-and-direct-progression-selection-gates).

**ROMDATA-05-GAMEPLAY-PROGRESSION-SELECTION-GATES-SCENE-CLI completed for EUR/Wine:**
production 34174 warning-free/canonical restored; CLI 43381 create/reuse-no-write/
old cache/unrelated cwd passes, 34 files unchanged by hash-mtime. First 400 Yes-fee
0 new B, negative closed/no full pair. First 1000 day 18 checkpoint dispatch
completion CLI 9298 positive 277B (selector 1/min01 26/min02 46/min04 42/ANY 152/
flag 10), mask 67108863: first 26 records observed. Single Wine 3138 full 1000
pair has identical state-clock-images-OAM-dispatch-audio; Sigmund/Mad Alchemist
success/skill 08-02-11-04-02-08-42-02 image inspected. No new GUI/fixture/payload
edit/personal save. New e37 proof, source 138+80/native 18/e36/delta 8/threshold
class 1 closed/not repeated. New Linux-USA-native Windows/all 64 ordinary/display
semantics remain to do; no performance/space measurements.
[Evidence](../rom_data_gameplay.md#ordinary-progression-selection-dispatch-completion).

**ROMDATA-01-GAMEPLAY-AGGREGATE-FLAG-KEYS-ROOTS / ROMDATA-02-GAMEPLAY-AGGREGATE-FLAG-KEYS-SOURCE-ORACLE completed for the EUR/Wine source scope:**
602B at 6e9a5c..6e9cb6, 300 u16 slots + ffff terminator, independent next root
29AF4 at the exact end. Two complete counters D22FC global/13FB1C context preserve
(key+2ff) u16, order/duplicates/early ffff, live flag 36 OR linked 017f extra 1/
u16 return. Strict lossless model and single Wine 38100 pass 32 real + 26 synthetic
cases, all 602B observed, context isolation/overflow/duplicates/early termination/
short circuit/readonly full EWRAM-SP-callee-stack/JSON-ID-order-offset-source
guards. Build 5092 warning-free, no error/old proof repeated. E37 unchanged;
preparation/consumption/lifecycle/scene and new Linux-USA-native Windows checks
remain to do; other 4 UI readers static-only.
[Contract](../rom_data_gameplay.md#aggregate-flag-keys-source-contract).

**ROMDATA-02-GAMEPLAY-AGGREGATE-FLAG-KEYS-DATA / ROMDATA-04-GAMEPLAY-AGGREGATE-FLAG-KEYS-CONSUMPTION completed for EUR/Wine:**
e38-d37 adds aggregate-flag-keys.json/gameplay.aggregate-flag-keys, 33 files/
32 resources/all 31 previous/e37 retained/USA e8-d7. Windows 76657 warning-free
and two Wine checks 87184 pass publication/retention/coherent key-duplicate-early
ffff-terminal-binding-source-role-path-hash-coverage/transactional owner/Load-Undo-
backup-restart; 16 full global-context counter paths in original/interpreter/native
with 602 raw B altered have identical state-clock results. Compiled key 9540/
terminal 30; all 602B/300-slot mask 5 words/boundaries-mirror-BIOS/preceding 8-
following 4 raw/SWI 32 typed each + terminal 2 typed 2 raw/word terminal 2 typed
2 raw/DMA 2 typed 2 raw pass. Source 32+26 closed/not repeated, original
instructions-context-predicates-wrapping-clock-ABI. CLI/ordinary scene/new Linux-
USA-native Windows remain to do, no performance/space measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-aggregate-flag-keys).

**ROMDATA-05-GAMEPLAY-AGGREGATE-FLAG-KEYS-SCENE-CLI completed for EUR/Wine:**
production 59636 warning-free/canonical restored; CLI 44401 create/reuse-no-write/
old cache/unrelated cwd passes, 35 files protected by hash-mtime. First 1000 day 18
dispatch positive with 1204 source B (1200 key/4 terminal)/all 300 slot mask
(5 words last 44 bits). Rereads included; neither 1204 distinct bytes nor ordinary
use of both counters inferred. Single Wine 11196 full 1000 pair has identical
state-clock-images-OAM-dispatch-all audio with the same counters; final Sigmund/
Mad Alchemist success/skills 08-02-11-04-02-08-42-02 inspected. Existing state/input,
no new GUI/fixture/payload edit/personal save. New proof only for e38; source
32+26/native 16/e37 same input closed/not repeated. New Linux-USA-native Windows/
other UI readers/ordinary use of both counters/rest of TODO remain to do; no
performance/space measurements or AndroidPort.
[Evidence](../rom_data_gameplay.md#ordinary-aggregate-keys-dispatch-completion).

**ROMDATA-01-GAMEPLAY-ADJACENT-HALFWORD-LOOKUP-ROOTS partial, selection closed:**
16 verified references at 6e9cb6 and next root 6e9d8a bound 212 candidate B,
but do not prove a domain of 106 halfwords. Fixed indices 64/67/lists 16..20 and
21,22,4,23 verified; RAM/UI producers remain to be proved. No model from values
or the next boundary alone; reopen only with domain/ownership evidence.

**ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE96-ROOTS statically completed:**
D0744 selects five 96B records/IDs 2..6 at 6e85bc..6e879c, 480B; biased price
root D26AC and independent next root agree. Hypothesis of 30 records rejected:
D2728 counts 30 RAM rows for internal argument 0..30. Strict/lossless source
model with 64 opaque B/u16 markup 40-41/30 discounts 42-5f, count 0 aliases
byte 41. Original D1F30 uses another 70B table/property 36, not items.json.
**ROMDATA-02-GAMEPLAY-TRADE-PARAMETERS-SOURCE-ORACLE completed for EUR/Wine:**
Windows 73640 warning-free/single Wine 52757 pass 499 real + 160 synthetic source
+ 15 separately injected arithmetic operand cases; all 160B price fields observed/
320 opaque B roundtrip only/all 480B lossless. Originals D0744/D26AC/D1F30/146474/
D2728, low8-nulloutside/zero-skip/alias 41/count 0..30/unsigned divide/wrap32/
full EWRAM readonly/SP-callee-stack/strict JSON-ID-order-offset-source pass.
72810 instructions/184 PCs are coverage only, 146 protected files unchanged.
Publication/consumption/lifecycle/scene and new Linux-USA-native Windows checks
remain to do; previous proofs closed/not repeated. EUR e38-d37 unchanged, no
performance/space measurements.
[Contract](../rom_data_gameplay.md#trade-parameters-source-contract).

**ROMDATA-02-GAMEPLAY-TRADE-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-TRADE-PARAMETERS-CONSUMPTION completed for EUR/Wine:**
e39-d38/trade-parameters.json/gameplay.trade-parameters/34 files/33 resources,
all 32 previous/e38 retained/USA e8-d7. Windows 21694 warning-free and two Wine
checks 20164 pass publication-failure/reuse-repair-movement/retention/20 coherent
edits/missing file-owner-transaction/Load-Undo-backup-restart/all 480B/mask 31.
28 original/interpreter/native paths with 480 raw B altered have identical
state-clock results; compiled opaque/markup-low/shared 41/discount 0-5-10-10,
opaque price semantics not inferred. All bytes/field-record edges/mirror-BIOS/
gap 222 raw/word tail 2 typed 2 raw/SWI 32 typed each + tail 2 typed 2 raw + old
selection 2/raw 222/new 2/DMA 2 typed 2 raw pass. Source 499+160+15 closed/not
repeated; original instructions/getter/live flag-count/clock/ABI. CLI/ordinary
scene/new Linux-USA-native Windows checks remain to do, no performance/space
measurements.
[Contract](../rom_data_gameplay.md#managed-and-direct-trade-parameters).

**ROMDATA-05-GAMEPLAY-TRADE-PARAMETERS-SCENE-CLI completed for EUR/Wine:**
production 46740 warning-free/canonical restored, CLI 17191 create/reuse-no-write/
old cache/unrelated cwd passes, 36 managed files protected by hash-mtime. First
400 normal pre-Yes/threshold-accept checkpoint positive with 14B (markup-low 7/
shared 41 7), mask 1 ID 2; rereads included, no 14 distinct B claimed. Single
Wine 89539 full 400 pair: all state-clock-images-OAM-dispatch/mixed-PSG-FIFO-HLE
audio identical, same counters. Final Pub list/7109 funds inspected, no new GUI/
fixture/payload edit/personal save. New e39 proof, source 499+160+15/native 28/
managed/e34 threshold 400/e38 closed/not repeated. New Linux-USA-native Windows/
other callers/discount tail-opaque-all 5 ordinary remain to do; no performance/
space measurements or AndroidPort.
[Evidence](../rom_data_gameplay.md#ordinary-trade-parameter-fee-acceptance).

**ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE7-ROOTS statically completed:**
11 literal roots at 6e879c, independent next root 6e87fe and item enumeration
362..375/subtract 362 establish 14 records of 7B/98B, no e39 overlap. Nine 5-bit
coefficients/8-bit cap/3 opaque bits roundtrip with independent getters. CFE28
property 0..19/index low8; CFDA8 cap uses full (item u16−362) u16, not low8.
Original current RAM/quantity/cap clamp; UI cap >99 static only, arbitrary other
indices raw/not claimed. Model/source oracle/managed/runtime/scene/new Linux-USA-
native Windows checks remain to do, e39 unchanged.
[Contract](../rom_data_gameplay.md#item-effect-parameter-static-contract).

**ROMDATA-02-GAMEPLAY-ITEM-EFFECT-PARAMETERS-SOURCE-ORACLE completed for EUR/Wine:**
strict 14 records 98B/9 coefficients 5bit-cap8-opaque3/ID-item-order-offset-array-
bitwidth/item owner 376/source guards pass. Windows 94173 warning-free; single
Wine 94185 original CFE28/CFDA8 passes 540 real + 4 raw cap probes outside owner +
166 isolated synthetic cases. All 98B read/roundtrip; low8 getter/invalid property
early zero/full u16 cap index vs getter low8/live current/quantity u8/cap fit-zero-
remaining/u16 return/readonly full RAM/SP-callee-stack pass. 31740 instructions/
156 PCs are coverage only, 258 protected files. Cap probes outside owner are not
new typed data/fallback. No old item oracle 14371/e39 repeated; e39-d38/34 files/
33 resources/USA e8-d7 unchanged. Publication/consumption/lifecycle/scene/new
Linux-USA-native Windows/full UI wrapper remain to do, no performance/space
measurements.
[Evidence](../rom_data_gameplay.md#item-effect-parameter-source-model-and-original-oracle).

**ROMDATA-02-GAMEPLAY-ITEM-EFFECT-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-ITEM-EFFECT-PARAMETERS-CONSUMPTION completed for EUR/Wine:**
e40-d39/item-effect-parameters.json/gameplay.item-effect-parameters, 35 files/
34 resources/all 33 previous/e39 retained/USA e8-d7, real items owner. Windows
93110 warning-free and two Wine checks 21177 pass publication-failure/reuse-repair-
movement/retention/22 coherent edits/bitwidth-ID-item-dependency-source-role-path-
hash-coverage/missing file-transactional owner/Load-Undo-backup-restart/all 98B/
mask 16383. 50 original-interpreter-native paths with 98 raw B altered have
identical state-clock results; compiled coefficient/shared05/shared06 bytes
36-37-23. All 7B record-field edges/mirror-BIOS/following 4 raw/word tail 2 typed
2 raw/SWI 32 typed each + tail 2 typed 2 raw + trade 2-item effect 2 crossing/
DMA 2 typed 2 raw pass. Source 540+4+166 closed/not repeated, original instructions/
index widths/raw cap alias/live RAM/clock/ABI. CLI/ordinary/new Linux-USA-native
Windows/full UI remain to do. DS22 populated bank is USA-only, not EUR evidence.
[Contract](../rom_data_gameplay.md#managed-and-direct-item-effect-parameters).

**ROMDATA-05-GAMEPLAY-ITEM-EFFECT-PARAMETERS-SCENE-CLI completed for EUR/Wine:**
production 4620 warning-free/canonical restored; CLI 59404 create/reuse-no-write
37 files protected by hash-mtime and 50707 old cache/unrelated cwd pass. New EUR
checkpoint IN13 with guest input only: Cyril Bank/List/Goblin Lv4/Play, valid
source header-hash; no RAM/state payload edit or personal save. First 400 CLI
positive 3B (coefficient 1/shared05 1/shared06 1), mask 1 record 0. Single Wine
5779 full 400 pair with all state-clock-images-OAM-dispatch/mixed-PSG-FIFO-HLE
audio identical/same counters. Mixed native/interpreter replay, no all-native/
performance claim. Old source/native/e39 proofs closed/not repeated. Rejected
Baguba/Sprohm navigation/hypothesized coordinates retained; Cyril coordinates
120,216 from existing accepted state, reached with short pulses. Final CLI bank
background inspected but separate UI panels absent as in the prior limitation:
no food-count/confirm claim. Other 14 items/20 monsters/cap >99 UI/new Linux-USA-
native Windows remain to do; rest of TODO open, no performance/space measurements.
[Evidence](../rom_data_gameplay.md#ordinary-item-effect-parameters-in-the-european-monster-bank).

<a id="romdata-06--mod-tramite-sostituzione-delle-risorse"></a>

## ROMDATA-06 — Mods through resource replacement

**ROMDATA-05-TEXT-CATALOG-SCENE-CLI completed for declared EUR/Wine scope:**
one ordinary 397-frame LCD B/Cyril pub BG62 entry pair reuses slot/input/cache.
Every state/clock/image/OAM/dispatch/audio matches, with 52 pointer bytes/
185 strings/2 indices/4 headers/decoded program: 27/8064 glyphs/93 metrics
read from files. This is a raw program: compressed use proven separately in 04.
One new decoded CLI reuses e12 without writes and the old cache, same result as
the retained legacy reference without repeating it. Warning-free Windows binary/
canonical restored; other gates/GUI/archives/Linux/USA open.
[Evidence and limits](../rom_data_text.md#ordinary-transition-and-production-cli).

**Dependencies:** complete main ROMDATA-01…05 delivery.

- [ ] Define a `mods/` directory in application storage and a versioned manifest
  with ID, version, compatible profiles/datasets, dependencies and replaced
  resources. Resolve relative paths within the package, with size and format
  limits; preserve user files during base regeneration.
- [ ] Define deterministic order and conflict handling. Validate overrides
  against the same base-resource contracts before game startup; show dependency/
  version errors instead of loading a partial combination.
- [ ] Add persistent enable/disable and indication of active mods. For the first
  version apply changes on session restart; hot reload is not required to
  complete initial support.
- [ ] Tie content hashes and mod order to caches and saves; isolate incompatible
  slots and protect original `.sav` files when a mod changes game data. Disabling
  mods must restore base resources.
- [ ] Test original/minimal mods for supported families, visible in-game change,
  conflicts, missing files and restoration. Distribute only original examples or
  descriptions/patches, not assets extracted from the user's ROM.

**Closure:** mods actually loaded through the resource path, reproducible
selection, recoverable base, compatibility and errors verified.

<a id="romdata-07--modifiche-estese-a-contenuti-e-logica"></a>

## ROMDATA-07 — Extended content and logic changes

**Dependencies:** ROMDATA-06 and verification of affected structures.

- [ ] Extend formats for longer text, new resources, maps, animations and gameplay
  tables, addressing ID, size, pointer and interface limits. Distinguish
  replacement of existing content from addition of new content.
- [ ] Design scripts, code patches and game rules separately: determine need for
  recompilation, ABI revision, cache invalidation and disabling incompatible
  hooks. A graphics override does not automatically enable logic changes.
- [ ] Define author tools and documentation, schema migrations and package
  validation; add original examples and relevant regressions.

**Closure:** every new capability has a contract, examples and verified
compatibility; unsupported changes are declared.

<a id="regole-di-avanzamento"></a>

## Progress rules

Select a subphase in [TASK_HANDOFF.md](TASK_HANDOFF.md), recording results and
missing criteria by family. Do not close the parent while its criteria remain
open or reopen completed children without new evidence. Follow
[MEMORY_WORKFLOW.md](MEMORY_WORKFLOW.md) for documentation, records and checks;
keep ROM, extracted resources and personal evidence out of Git.


**ROMDATA-01-AUDIO-TYPED-CATALOG-ROOTS completed for the initial EUR roots:**
Original/live M4A verified without a new replay; 3 players 36B, directory of
640 entries/5120B, 455 distinct headers/10080B, 186 empty-header aliases,
1610 track roots and 4 banks. Independent roundtrip of all fields; no invented
640/128-tone guard, header 150940/WaveData 150944 alias explicit. Complete
banks/samples/bytecode remain open. [Contract](../rom_data_audio.md).

**ROMDATA-02-AUDIO-M4A-DIRECTORY-SOURCE-ORACLE completed for EUR/Wine:** strict
model of 3 players/640 entries/455 headers/457 ranges/15236B, header ID is the
minimum songID and aliases are explicit; exact source/roundtrip/17 JSON-shape-link-
source guards pass. Final Windows build warning-free; single Wine oracle passes
640 selections + 9 targeted inputs, 1 raw lookup 640 stopped before the null
header dereference and 6 unpublished synthetic variants. Complete EW-RAM/IW-RAM
outside scratch stack/traces/SP/callee match independent expectations, 13477B
actually read/1759B retained only. First compile E0283 and first incorrect
word 20 tracks expectation retained; corrected zero expectation proved by
backedge 1492e6. Model unchanged; no ordinary/native/clock acceptance claimed.
Dataset e40-d39/USA e8-d7 unchanged.

**ROMDATA-02-AUDIO-M4A-DIRECTORY-DATA / ROMDATA-04-AUDIO-M4A-DIRECTORY-CONSUMPTION
completed for the stated EUR/Wine scope:** e41-d40 adds audio-directory.json/
resource audio.m4a-directory, 36 files/35 resources/34 previous decoded files
and e40 retained, USA e8-d7. Typed owner of 457 spans/15236B without raw overlay,
four category counters and masks for 3 players/640 songs/455 headers. Windows
build 5021 warning-free and two Wine checks 51686 pass publication/25 coherent
guards/reuse-repair-move-retention/Load-Undo-backup-Restart without folder,
44 original-interpreter-native paths with 15236 raw B altered/equal full
state-clock results. Every byte/span/edge/width/mirror/BIOS/SWI-DMA passes,
including 2 typed 2 raw at the dummy-wave boundary and CPUSet 2 directory +
8 header + 2 raw. Mixer/sequencer/RAM/ABI remain; source 649+1+6 and e40 not
repeated, no new build/Wine error.

**ROMDATA-05-AUDIO-M4A-DIRECTORY-SCENE-CLI completed for the stated EUR/Wine scope:**
production 78108 warning-free/canonical restored; CLI 14580 creates/reuses e41
without rewriting 38 files, 27876 reuses old cache from an unrelated cwd. Existing
EUR Goblin Play checkpoint/input 400 without new navigation/fixture/RAM edit:
179 new B (36 directory/79 header/64 pointers), 3 songs/headers/0 player configs
read. Single Wine 91327 full 400 pair has identical state-clock-images-OAM-
dispatch-mixed PSG FIFO HLE audio/same 179B; mixed execution, not all-native.
568×160 Bank background image inspected; panels omitted by serialized separateUI
as already known, no new confirmation/food count claimed. Source 649+1+6/native
44/e40 scene closed/not repeated; no build/Wine error in this subphase.

**ROMDATA-01-AUDIO-M4A-TONE-SEQUENCE-PRODUCERS completed for the stated produced
domains:** graph of 1610 tracks/177047B/805 ranges/zero errors, 36 copy+patch
commands matching retained RAM. 390 bank-program pairs, 429 tones/5148B, 348 keymap
links/347B/1 alias, 277 PCM headers/1931528 nominal payload bytes; 276 extra
interpolated endpoints/1 fixed8 within length. 2 static 16B CGB sources. Plan of
736 ranges/1941759 new B excludes 4B already in e41. Build 32264 warning-free/
single Wine 8457 passes 390 BD/348 map/276+1 endpoints; note/kernel are partial
checkpoints, not complete mixer or scenes, not all PCM read. 128-keymap sweep
rejected as ownership: it invades adjacent tables. Parser/model/new publication-
runtime remain to do; e41 unchanged.
[Contract](../rom_data_audio.md#produced-tone-keymap-and-pcm-source-contracts).

**ROMDATA-02-AUDIO-M4A-TONE-MODEL completed for EUR/Wine (2026-10-24):** shared
bounded parser reproduces 1610 graphs/147424 states/390 programs/2065 key pairs/
805 ranges 177047B; strict model of 429 tones/347 map bytes-348 links/277 PCM/
2 CGB, exact roundtrip of 736 ranges 1941759B. Signed i8 PCM + optional endpoint/
CGB 32 nibbles, IDs from first references/canonical aliases, no mixer rewritten.
Build 61632 warning-free/single Wine 19316 passes domains/all independent field-
hashes/JSON/source roundtrip/28 guards and coherent source/alias edit with the
same new spans rejected by the real directory dependency. 2 new original CGB
copies pass 32B wave RAM/RAM/SP in an internal checkpoint, not the complete mixer.
492 originals/evidence protected; no 390 BD/348 map/277 endpoint/e41 replay
repeated, e41-d40/USA e8-d7 unchanged.

**ROMDATA-02-AUDIO-M4A-TONE-DATA / ROMDATA-04-AUDIO-M4A-TONE-CONSUMPTION
completed for the stated EUR/Wine scope (2026-10-24):** e42-d41/37 files/
36 resources, all 35 previous files/e41 retained/USA e8-d7. 736 disjoint sources
1941759B/real directory dependency 4B. Two Wine checks 84702/3937 pass 26 guards/
publication-reuse-repair-move/lifecycle/all bytes and masks; 25 original/
interpreter/native paths with raw altered/full state-clock, 736 boundaries/mirror/
BIOS/SWI-DMA/alias 2+2/raw fixed8. Both MP2K hook/tick call sites read 4428 header B
+ 35458 PCM B typed from 277 waves, nonzero bit-exact audio and equal full state/
clock. 276 direct host endpoints separate from existing HLE clamp; not all PCM read
by mixer. Internal note/header/ARM/CGB checkpoints do not equal the entire original
mixer. Final build 49176 warning-free; two test preparation errors corrected and
retained, no model/production change; no closed check reopened.

**ROMDATA-05-AUDIO-M4A-TONE-SCENE-CLI completed for EUR/Wine (2026-10-24):**
production 16173 warning-free/canonical restored, CLI 79505 creates/reuses without
writing 39 files/old cache/unrelated cwd/400 bank Play uses 516956 guest B
(247 tone/5 map/11362 header/504912 PCM/430 endpoint), 0 CGB. Default CLI shadow
off explicit: single Wine 36911 pair arms it after load and passes 400 states-clock/
images/OAM/dispatch/audio mixed-PSG-FIFO-HLE with same guest B and 3093208 host B
(1056 header/3092152 PCM). Both required uses positive; existing snapshot/input
unchanged, no new navigation/fixture. Bank background capture 568x160 inspected;
separateUI omits panels as already known, no new food count confirmation.

**ROMDATA-02-AUDIO-M4A-SEQUENCE-MODEL completed for EUR/Wine (2026-10-24):**
105253 physical instructions/single interpretation/no overlap, previous 1610 roots/
805 ranges/177047B unchanged. Strict enum for wait/note/parameter/pattern/end/
return/jump/endtie/extended, canonical IDs for jumps/patterns/tracks; explicit
versus inherited opcode and operands preserved without a host sequencer/frozen RAM.
49 clock B/36 command words 144B/only CD8-9 dispatch 8B add 3 spans 201B:
808 ranges 177248B disjoint from the 36 e42 resources. Final build 76630 warning-
free/single Wine 4313 passes all field-hashes/sources/JSON/27 guards/inherited
metadata with unchanged bytes/real directory. 404 new original calls/checkpoints:
161 prefix dispatch/158 note args/16 jump-pattern-return/12 parameter/4 CD8-9/
49 wait/3 CE/1 copy 36 words; RAM/cursors/SP/stack pass. No full sequencer/mixer/
state-clock/native/ordinary acceptance. First Wine 29418 failed on two PCs
transcribed +2 in driver assertions, corrected from ROM bytes; log/exe/source
retained. 598 inputs/evidence protected; old proofs not repeated.

**ROMDATA-02-AUDIO-M4A-SEQUENCE-DATA / ROMDATA-04-AUDIO-M4A-SEQUENCE-CONSUMPTION
completed for EUR/Wine (2026-10-24):** e43-d42 adds audio-sequences.json/
audio.m4a-sequences, 38 files 37 resources/all 36 previous decoded files/e42
retained/USA e8-d7. Real directory+tone dependencies/390 program-key domains from
147424 states, inherited metadata/banks with the same bytes rejected. Two Wine
checks 49125/build 11482 warning-free pass publication/25 guards/lifecycle/all
177248B/masks for 105253 instructions-49 clock-36 command-2 ext and 35 original/
interpreter/native paths with raw altered/full state-clock. 808 boundaries/mirror/
BIOS/SWI-DMA/raw fallback/tone 2 + sequence 2 guest-host/live RAM running status
pass. Generic host is not a host sequencer; internal checkpoints/narrow readers do
not prove full sequencer/mixer ABI. No new build/Wine error; old 404/e42 proofs
not repeated, 627 old protected files unchanged.

**ROMDATA-05-AUDIO-M4A-SEQUENCE-SCENE-CLI completed for EUR/Wine (2026-10-24):**
production 48965 warning-free/canonical restored, CLI 48325 creates/reuses without
rewriting 40 files/old cache/unrelated cwd/400 bank Play reads 819 new B
(366 opcode/262 operand/191 clock), 185 source records observed including lookahead,
not a count of executed instructions. Single Wine 29123 pair passes 400 states-
clock/images/OAM/dispatch/identical mixed-PSG-FIFO-HLE audio/same 819B. 0 command/
extended/generic-host in this scene, no ordinary initialization. Existing checkpoint/
input unchanged, no new navigation/fixture. 568x160 Bank background capture
inspected, separateUI omits panels as known, no new confirmation/food count.
Source 404/native 35/e42 proofs not repeated; no new build/Wine error in this
subphase.

**ROMDATA-01-AUDIO-M4A-DRIVER-LUTS / ROMDATA-02-AUDIO-M4A-DRIVER-MODEL completed
for the stated EUR/Wine scope (2026-10-24):** 7 tables 370b20..370d04/484B
disjoint from e43: 180 PCM keycodes/12 u32 bases/12 u16 sample counts/132 CGB
keycodes/12 i16 bases/60 noise/16 volume. Decomposed nibbles/sign/opaque bits/
aliases preserved. PCM key low8 clamp 178 + endpoint 179/frac 255 and unsigned
UMULL; CGB tonal key <=35→0 frac 0/minus 36 clamp 130 + endpoint 131/signed ASR;
noise type 4/key <=20→0/minus 21 clamp 59. Sample frequency nibble 0..15 without
clamp 12: 0 aliases last PCM base, 13..15 alias CGB; public SoundMode skips zero.
Raw RAM volume 16/64 delegate to previous clock/255 raw, no invented guest guard.
Build 87853 warning-free/single Wine 60589 passes 1138 new original calls
(541 PCM/300 tonal/259 noise full return, 18 rate/19 volume prefix/1 zero mode
full return), all 484B at observed load sites, RAM/SP/callee/stack, source+JSON/
15 guards/coherent source/sequence boundary owner. No full init/mixer/state-clock/
native/ordinary acceptance; e43-d42/USA e8-d7 unchanged. 879 inputs/evidence
protected; no old acceptance repeated/build-Wine error. Final reconnaissance
corrects global cap 15 hypothesis: balanced branch 149640 skips clamp 14964c.
Additional Wine 27372/build 59773 runs only 8 new gain returns and passes
128/128→16, 255/128→23, 255/255→31; unilateral branch clamp 15, exact RAM/SP/
callee/stack 8B. No new bytes/model change/1138 repeated; ordinary global domain
16 not proved, aliases must be preserved. Producer proof/correction closed.

**ROMDATA-02-AUDIO-M4A-DRIVER-DATA / ROMDATA-04-AUDIO-M4A-DRIVER-CONSUMPTION
completed for EUR/Wine (2026-10-24):** e44-d43 publishes audio-driver-tables.json/
audio.m4a-driver-tables, 39 files 38 resources/all 37 previous decoded files/e43
retained, USA e8-d7. 7 spans 484B/real sequence dependency. Build 70748 warning-
free/Wine 51847 two reduced checks pass 19 guards/publication-reuse-repair-move/
retention/lifecycle/all 484B/424 entry bits and 28 original-interpreter-native paths
with raw altered/exact state-clock, UMULL Thumb-ARM/all compiled categories positive
[10,42,6,18,24,4,2]. Sample aliases 0/13..15 and volume 16/31/64 owner clock/
255 raw, all 7 boundaries/width/mirror/BIOS/SWI/DMA/guest-host crossing pass.
844 instructions/150 PCs/70 dispatches coverage only; no full init/mixer ABI.
917 protected + exe unchanged; source 1138/gain 8 and old proofs closed not
repeated. No build/Wine error.

**ROMDATA-05-AUDIO-M4A-DRIVER-SCENE-CLI completed for EUR/Wine (2026-10-24):**
Build 85643 warning-free/canonical restored; Wine 30529 real CLI creates/reuses
without writing 41 managed files/old cache/unrelated cwd 400 EUR bank. 756 aggregate
byte reads [132 PCM key,528 PCM base,0 sample count,32 CGB key,64 CGB base,0 noise,
0 volume]; 67 physical entries [31,12,0,16,8,0,0], host 0. Single Wine 21679 pair
400 exact state-clock/images-OAM/dispatch/mixed PSG FIFO HLE audio, same positive
use. Not distinct bytes/performance/all-native. 568x160 background image inspected,
separate UI omitted as known, no new food count/confirmation. Existing state/input
unchanged/no new GUI-fixture-RAM alteration. Source 1138/gain 8/native 28/old
proofs not repeated, no new build/Wine error/live job.

**ROMDATA-01-AUDIO-FFTA-WRAPPER-SOURCES completed, reclassified (2026-10-24):**
370eb0 is an executable vector of IWRAM clear/copy/fill helpers, not audio; b94690
is graphics palette-effect dispatch. Verified 7 literals/5 fixed producers
1/2/4/8/16, state 1 reader/unmasked u16/stride 4, projection of 17 words 68B with
12 null holes/disjoint from 38 e44 owners. No global cap 16/total extent inferred;
higher indices raw. Three 32B prefixes in the old EUR state uniquely match ROM
code, without new gameplay/fixture/build/repeated old proofs. Audio catalog unchanged.

**ROMDATA-02-PALETTE-EFFECT-DISPATCH-MODEL completed for EUR/Wine (2026-10-24):**
Model of 17 slots 68B/canonical u16 kinds/5 Thumb targets as code offsets/12 null.
Build 71666 warning-free/single Wine 22360 passes field-hashes/JSON-source/
12 guards + wrong ROM and 31 original prefixes (23 lookup/3 state gate/5 producer
store), all 68B observed at 0814ac70. High word truncated/live u16/null/raw
17-31-255-65535/RAM-SP-callee/2B write exact. 134 instructions 18 PCs coverage
only, not full callback/native/state-clock/ordinary. 1216 protected unchanged/no
build-Wine error/old proofs repeated; e44 unchanged.

**ROMDATA-02-PALETTE-EFFECT-DISPATCH-DATA / ROMDATA-04-PALETTE-EFFECT-DISPATCH-CONSUMPTION
completed for EUR/Wine (2026-10-24):** e45-d44 publishes 40 files 39 resources/all
38 previous decoded files/e44 retained/USA e8-d7. Build 67564 warning-free/Wine
46138 two reduced checks pass 12 guards/publication-retention-lifecycle/all 68B/
17-slot mask and 10 original/interpreter/native prefixes with raw altered/exact
state-clock, 24 callback/4 null compiled. Live u16 kind/truncation/null/state gate/
raw 17-65535/all edges-width-mirror/BIOS/SWI/DMA/2 typed-2 raw pass. 48 instructions
8 PCs 10 dispatches coverage only, not whole callback/allocator ABI. 1226 protected
+ exe unchanged; source 31/e44 old proofs not repeated/no build-Wine error.

**ROMDATA-05-PALETTE-EFFECT-DISPATCH-SCENE-CLI / SCENE-TRANSITION completed for
the stated EUR/Wine scope (2026-10-24):** build 95401 warning-free/canonical
restored, Wine 56076 real CLI creates/reuses without writing 42 files/old cache/
unrelated cwd. Boot 240 neutral 0 use closed/no full pair. New normal input confirms
English A 240/release 252, then neutral 1000: Wine 60304 CLI positive 1486 reads/
5944 aggregate callback bytes/null 0/slot mask 2 (kind 1 only). Single Wine 35467
pair 1000 passes all state-clock/images-OAM/dispatch/mixed PSG FIFO HLE audio,
same use. Not 5944 distinct bytes/performance/all-native. 568x160 intro ProductDevelopmentDivision-4/snow image inspected. Other 4 callbacks/null targeted-only,
not whole ABI. 10 protected snapshots inspected statically with empty list, no
replay/new snapshots/RAM-state payload-save altered. Source 31/native 10/old proofs
closed not repeated; no new build-Wine error/live job.

**ROMDATA-01-PALETTE-EFFECT-PROGRAM-PRODUCERS partial (2026-10-24):**
Initial path main 080d7d24→0814b9f8→callback 4 closed. Real palette task
080d7fac/public 080d8214-080d8234 distinguished from nearby task 080d79ac/79d0
(248 candidates, not palette). 308 direct BL; 285 exact argument expressions/
179 ROM programs. 3 lists with counts 3/3/2 and 8 rows 8B add 5 programs; 2 LDM/STM
copies of 3 words add 1. Retained R4 bridge reuses an already known root. Total
185 programs/1000 commands/16135 candidate B in 124 ranges + 64 list/24 stack =
16223B disjoint from 39 e45 owners. Header 6B (count u8), commands 6B/relative
u16 offset, direct versus queued color copy and minimum 1 sequence documented;
parameter lengths still need comparison with original Wine readers.
20 indexed producers now resolved as bounded projections from task ID/message
copies/constant stores/counted loops: 97 cells/388B, 45 programs added. Total
230 programs/1135 commands/18221 program B + 476 producer B = 18697B/165 ranges,
disjoint from 39 owners. Previous 185 records identical; 39bcc4 only 2 pointers,
null/scalar tail excluded. No new RAM cap/global domain/full physical extent. No
model/runtime/e46/guest/build/test; e45/USA e8-d7 unchanged. Closed proofs not repeated.

**ROMDATA-01-PALETTE-EFFECT-INDEXED-PRODUCER-DOMAINS completed for the 20 projections**
(2026-10-24). Task source 080d7018 uses distinct segments ID 1..341 and ID 342+;
ID 354/355 do not derive from a single extrapolated base. Explicit proofs/fingerprint
in indexed-producer-proofs; new indexed-program-discovery authoritative 230 records.

**ROMDATA-02-PALETTE-EFFECT-PROGRAM-MODEL completed for EUR/Wine (2026-10-24):**
Lossless model of 230 programs 1135 commands/25 tables, 165 ranges 18697B,
fingerprint 120, canonical IDs/null pointers/aliases/delay/reserved and parameters
preserved. Windows 23752 warning-free; single Wine 96989 passes all 18221 program B/
18681B total observed on original readers, 16B reserved list retained/not read;
97 cells/8 rows/2 stack. 6 copy wrap-clamp limits/3 min count-period/1 direction 255
and 16 shape-JSON-source guards pass. No full allocator/callback/native/state-clock/
ordinary acceptance. 1645 artifacts + new test exe unchanged; e45-d44/USA e8-d7
unchanged, no performance measurement.

**ROMDATA-03-PALETTE-PROGRAM-RUNTIME integration/targeted completed for EUR/Wine**
(2026-10-24): e46-d45/41 files 40 resources/all 39 previous decoded files/e45
preserved. Binding 165/18697B from typed fields; header/command/parameter/producer
and canonical IDs, no new RAM index guard/sequencer/save-native ABI. Wine 78294
managed passes 12 guards/lifecycle/all 18697B/230 programs 25 tables. Initial
native/diagnostic tests failed for an endpoint inside a concatenated graph; a guard
in the test corrects the boundary, production backend unchanged. Native-only 33298
passes 24 full state-clock prefixes, generated API [16 header 52 command 66 parameter
12 producer] B, raw command 65535/offset 65534/index 2/BIOS-SWI-DMA/mirror/edges.
Source/managed already closed not repeated.

**ROMDATA-04-PALETTE-PROGRAM-CLI completed for EUR/Wine (2026-10-24):** Windows
19390 warning-free/canonical restored; Wine 60759 create/reuse-no-write 43 managed
files and old cache from a different cwd pass. Normal English confirm 1000 reads
0 new bytes: negative locator closed, no full pair/do not repeat. Separate static
scan of new task 080d7fac in 10 protected worldmap/bank snapshots: manager zero/
no callback, no replay/new snapshot. Do not repeat for context.

**ROMDATA-05-PALETTE-PROGRAM-SCENE completed for EUR/Wine (2026-10-24):**
Montblanc Richiamo/Pecorelle, from a confirmation checkpoint reached with normal
EUR input, passes real CLI and a single 1000-frame comparison in 240x160 mode:
full state/clock/images/OAM/dispatch/mixed-PSG-FIFO-HLE audio identical. 170 aggregate
byte reads = 32 header + 90 command + 48 parameter, 4 programs, 0 producer. The
other 226 programs and producer tables retain targeted proof. First comparison had
completed all 1000 frames and the new-family assertion, then failed only the old
generic host OBJ dimensions requirement. The test now accepts typed bus or host
reads; positive header/command/parameter requirement unchanged. Only the Windows
test rebuilt warning-free and only this comparison repeated, passing. Production
runtime unchanged; no new source build/retest, managed/native/CLI create-reuse.
Both outcomes retained. No changes to original RAM/state payload/save. F09 and river
preparation remain private navigation; locators without execution are not positive
proofs. Model 230/source Wine/20 projections/native 24/managed/CLI/ordinary closed;
intro 1000 negative and 10 snapshots are not next actions. New Linux/USA/native
Windows and a new 32:9 scene remain to do.

**ROMDATA-01-BLEND-PROGRAM-PRODUCERS completed for the stated EUR projections
(2026-10-24):** task 080d77c4/ctor 080d79ac-080d79d0 control BLDCNT,
BLDALPHA, BLDY. Classified 248 direct calls: 235 constant/2 conditional/
6 indexed/1 inline/1 mutable RAM template/3 dynamic RAM. 160 programs/345 commands,
960B header + 1380B commands + 88B producer (22 pointers/6 tables) = 2428B/
158 ranges, disjoint from all 40 e46 resources. Header 6B (u16 control/coeff/count),
command 4B (u16 duration,u8 first/second). No cap 16/RAM index guard/full physical
extent inferred. False code root 080e2ec8 rejected: caller copies 10B from 39a5f0
to RAM and overwrites duration. Independent reconstruction/hashes, 44 consumer
opcodes and DMA register mapping pass statically; no new guest run, build or
runtime test. Source and Wine behavior remain distinct criteria.
[Contract](../rom_data_graphics.md#blend-program-producer-inventory).

**ROMDATA-02-BLEND-PROGRAM-DATA completed for EUR/Wine (2026-10-24):** strict
lossless model of 160 programs/345 commands/6 tables 22 pointers, 2428B/158 ranges;
bitfields and reserved fields preserved, no new cap 16/RAM guard. Exact EUR identity,
110 opcodes and canonical references authenticated. Windows 12341 warning-free;
single Wine 94935 passes 160 headers/345 commands/22 cells/5 instant chains/
template copy+RAM override/2 maximum headers-mode 0/6 durations/3 delays/index
65535 raw. 17 shape guards/3 edited source/2 JSON/2 ROM identity checks pass.
2376B observed; 52 second-target bytes for nonalpha timed cases remain retained/
hash-verified for these initial-mode probes, no global unreachability inferred.
52943 instructions/264 PCs coverage only.
[Model and limits](../rom_data_graphics.md#typed-blend-program-source-model).

**ROMDATA-03-BLEND-PROGRAM-RUNTIME integration/targeted completed for EUR/Wine
(2026-10-24):** e47-d46/42 files 41 resources/all 40 previous decoded files/e46
preserved. 158 bindings/2428B from typed header-command-producer, 13 pages; no new
RAM guard/cap 16/sequencer/save-native ABI. Two reduced Wine checks pass 12 guards/
lifecycle/all 2428B/160 programs 6 tables and 20 original-interpreter-native
prefixes full state/clock, with [34 header 70 command 16 producer] B from compiled
API. IWRAM template/live RAM/mode 2→1/raw command 65535/index 3/previous palette
owner index 5/width-mirror-BIOS-SWI-DMA/new 2 raw 2/new 2 old 2 pass.
1034 instructions 295 PCs 93 dispatches coverage only. After review, corrected the
extracted set consistency gate: one additional negative Wine check passes without
repeating native/lifecycle. Affected Windows builds warning-free; first test arity
error retained. No complete allocator/callback/scheduler ABI or ordinary proof yet.
[Consumption](../rom_data_graphics.md#managed-blend-programs-and-typed-guest-reads).

**ROMDATA-04-BLEND-PROGRAM-CLI / ROMDATA-05-BLEND-PROGRAM-SCENE completed for
EUR/Wine (2026-10-24):** Windows 81247 production warning-free/canonical restored.
Wine 46190 CLI create/reuse-no-write 44 managed files/old cache/different cwd passes;
all 42 generation files match the accepted dataset. Real EUR scene Montblanc
Richiamo/Pecorelle from retained checkpoint/input: 18 aggregate byte reads =
10 header + 8 command, 1 program, 0 producer. Single Wine 91145 comparison of
1000 frames on the CLI-published generation passes all full states/clock/240x160
images/OAM/dispatch/mixed-PSG-FIFO-HLE audio. Other 159 programs and 6 producer
tables retain targeted coverage; new 32:9 scene remains to do. No test rebuild,
navigation/fixture/payload edit or repeated source 160/inventory 248/e46/native 20/
lifecycle. USA e8-d7 unchanged; new Linux/USA checks remain to do.

**ROMDATA-01-TASK-DESCRIPTOR-SOURCE completed for the stated EUR projection
(2026-10-24):** 341 primary/20 auxiliary/51 script records, 412 records 8B/3296B/
3 ranges, disjoint from 41 e47 owners. Nullable Thumb callback/u16 context size/
raw u16 metadata; 268 distinct nonzero callbacks/3 null records. Boundary 39a490
proved by independent graphics consumer and directory row kind 0/ID 413 selecting
39a488. Primary ID 362 ≡ secondary 363, last 412 ≡ 413; no global selector guard.
59 halfword/12 word fingerprint and static roundtrip/hashes pass; 53 BL candidates
retained, incomplete CFGs remain unknown. Hypothesis of 456 records rejected, do
not reopen. [Contract](../rom_data_tasks.md).

**ROMDATA-02-TASK-DESCRIPTOR-MODEL completed for EUR/Wine (2026-10-24):** strict
lossless model of 341+20+51 records/3296B/3 ranges, ordinal/nullable Thumb/context
size/raw metadata and two exact ID formulas. Windows 24937 warning-free; single Wine
82576 passes 415 primary/59 secondary loads/346 metadata halfword/346 byte/18 low16/
9 branch and 23 guards. 3156 distinct B observed/140 metadata B retained unread in
these checks; aliases and raw indices 0/65535/413-414 preserved. 7191 instructions
39 PCs coverage only, not complete allocator/callback/scheduler ABI/native/state-
clock/ordinary scene. e47-d46/USA e8-d7 unchanged; no closed proofs repeated/
performance measurement. Linux/USA remain to do.
[Model and evidence](../rom_data_tasks.md#reduced-wine-source-validation).

**ROMDATA-03-TASK-DESCRIPTOR-RUNTIME completed for targeted EUR/Wine criteria
(2026-10-24):** e48-d47 publishes task-descriptors.json/engine.task-descriptors,
43 files 42 resources/all 41 previous decoded files/e47 retained/USA e8-d7 unchanged.
3 bindings 3296B/pages 399-39a, typed callback/context size/metadata, selectors/ABI
unchanged. Windows 72886 warning-free/Wine 32369 two checks pass 13 guards/lifecycle/
all 3296B/412 records 3 catalogs and 29 original/interpreter/native prefixes with
full state-clock. Compiled API [44 callback 22 context 9 metadata] B, alias/raw
0-65535-413/414/low16/special metadata/old blend owner 6B altered, edges-width-
mirror-BIOS-SWI-DMA/new 2 raw 2 pass. 206 instructions 41 PCs 29 dispatches coverage
only, not full ABI/ordinary scene. Source and closed proofs not repeated.
[Consumption](../rom_data_tasks.md#managed-task-descriptors-and-typed-guest-reads).

**ROMDATA-04-TASK-DESCRIPTOR-CLI / ROMDATA-05-TASK-DESCRIPTOR-SCENE completed for
EUR/Wine (2026-10-24):** Windows 87514 production warning-free/canonical restored;
Wine 38310 real CLI creates/reuses without writing 45 managed files/loads old cache
from a different cwd; all 43 generation files match the accepted dataset. Authentic
EUR Pecorelle 1000/classic 240x160 scene passes 28 aggregate B = 16 callback
8 context 4 metadata/4 records 2 catalogs. Single Wine 58452 pair from CLI generation
passes all 1000 state-clock/images-OAM-dispatch/mixed PSG FIFO bitwise HLE audio.
Other 408 records/third catalog targeted only; not full ABI/new 32:9. No fixture
recreation/navigation/RAM-payload-save alteration/repeated old source-native-
lifecycle proofs. [CLI and scene](../rom_data_tasks.md#production-cli-and-ordinary-scene).

**ROMDATA-01-EFFECT-EVENT-DIRECTORIES / ROMDATA-02-EFFECT-EVENT-DIRECTORY-MODEL
completed for the stated EUR/Wine scope (2026-10-24):** 75 events 16B/51 selectors
4B/28 even Thumb same-state dispatch words, 1516B/4 ranges disjoint from 42 e48
owners. Increment/comparison 74 and independent catalog 3acebc prove 1200B;
independent reader 3ad240 proves 204B selectors without inferring extent from
51 tasks. Corrected candidate 128b4c (literal→table 128b50), retained initial
overly narrow guard 125394→125398. 19+51 logical references to task catalog;
q0/index 65535/ID 362/jump RAM/default remain original. Windows 78776 warning-free/
single Wine 43580 passes 77 header 77 delay/53 selector 43 param/32 dispatch-default/
19 low16-wrap and 27 guards. 1106 distinct B observed, 300 event raw + 110 param B
retained-hash-checked/unread in these checks. 2582 instructions/70 PCs coverage
only; not interpreter-allocator/native/state-clock/ordinary ABI or reachability
proof 124e1c. e48-d47/USA e8-d7 unchanged, Linux/USA remain to do.
[Contract](../rom_data_events.md).

**ROMDATA-03-EFFECT-EVENT-DIRECTORY-RUNTIME completed for targeted EUR/Wine
criteria (2026-10-24):** e49-d48 publishes 44 files 43 resources/all 42 previous
decoded files/e48 retained/USA e8-d7 unchanged, real engine.task-descriptors
dependency. 4 bindings 1516B/pages 124/128/3ac/3ad from events/selectors/even
dispatch, no new ROM shadow/guest cap/sequencer/save-native ABI. Windows 93869
warning-free/Wine 91663 two checks first PASS: 13 guards/lifecycle/all 1516B/
75 events 51 selectors 2 tables and 29 original/interpreter/native prefixes with
full state and clock. Generated API [24 event 6 selector 24 dispatch] B; index 75/
51-65535/q0/wrap/default/even Thumb, event 65535→previous font 11B, previous palette
on same page 4B, edges-widths-mirrors-BIOS-SWI-DMA/new 2 raw 2 pass. 186 instructions
70 PCs/29 dispatches coverage only, not full ABI/ordinary/reachability.
[Consumption](../rom_data_events.md#managed-publication-and-typed-guest-reads).

**ROMDATA-04-EFFECT-EVENT-DIRECTORY-CLI completed for EUR/Wine (2026-10-24):**
Windows 48886 production warning-free/canonical restored; Wine 76313 real CLI
creates/reuses without writing 46 managed files/loads old cache from a different
cwd; all 44 generation files match the accepted e49 dataset.

**ROMDATA-05-EFFECT-EVENT-DIRECTORY-SCENE completed for EUR/Wine (2026-10-24):**
Authentic EUR locators Pecorelle 1000, school cold boot 3000 and suspend boot-to-
battle 8476/classic 0 end with 0 new-family reads; closed negative, no full pair
required for events. Opcode trace 59/5a→128af4 and authenticated Pecorelle/F09
snapshot read with root 20159d4 zero narrow the context; USA identity candidate
excluded before inspection/replay. New normal EUR startup/name/snow/street/room
progression produces 19 authentic snapshots, exact roundtrips and verified input/
artifact hashes; no RAM/state/save patch or original changes. Two new bounded
locators 3180 tutorial/6000 turn-transition closed negative, no full pair. Subsequent
state 39420 room/book gives a new 1800 locator, Wine 59085 positive: 2 bus reads/
5B = 0 event + 1 selector + 4 dispatch, one selector/one table. Single Wine 91753
pair 1800 same state/input/CLI generation e52/cache/test exe passes all state-clock/
center-wide images/OAM/dispatch/mixed PSG FIFO bitwise HLE audio, samples
[3950100,3950100,475200,475200,3950026], first use 39946/same 5B/470 hash-mtime
guards. Event-OR-selector criterion closed with selector read; 0 event rows/other
50 selectors/other table remain targeted. No product/core/prepare rebuild/old
proofs/performance repeated. Linux-USA-native Windows/new 32:9/full ABI/
reachability 124e1c unverified.
[Evidence](../rom_data_events.md#positive-ordinary-book-transition).

**ROMDATA-01-SCRIPT-COMMAND-DIRECTORY / ROMDATA-02-SCRIPT-COMMAND-DIRECTORY-MODEL
completed for the stated EUR/Wine scope (2026-10-24):** 115 commands of 6B/690B at
3acebc..3ad16e, three original roots and an independent bound from two lookup
readers 1263c4/126468. Initial candidate 116 rejected before plan/model: it included
the distinct table at 3ad16e. Unique non-null Thumb BX callbacks and u16 advance,
no asset dependency; disjoint from 43 e49 owners. Strict lossless model and
106 halfword/6 word fingerprint/hash roundtrip pass. First Wine 49512 fails test
expectation ADDS r1 instead of r0; only two assertions corrected, failed proof
retained. Windows 12070 warning-free and single corrected Wine filter 67188 pass
117 callback/117 advance/115 BX/4 signed cursor/10 relative-target/5 loop-return,
all 690 distinct B observed and 19 guards. 2905 instructions/67 PCs coverage only;
not interpreter/native/full state-clock/ordinary ABI. e49-d48/USA e8-d7 unchanged;
publication/runtime consumption/CLI/positive scene completed in units below.
[Contract](../rom_data_scripts.md).

**ROMDATA-03-SCRIPT-COMMAND-DIRECTORY-RUNTIME completed for targeted EUR/Wine
criteria (2026-10-24):** e50-d49 publishes 45 files 44 resources/all 43 previous
decoded files/e49 retained/USA e8-d7 unchanged. No asset dependency, source-set
consistency requires previous owner. 690B/pages 3ac-3ad from 115 typed commands,
without ROM shadow/new guest caps/save-native ABI. Windows 93228 warning-free;
Wine 6355 two filters pass first attempt 13 guards/lifecycle/all 690B and 26
original-interpreter-native prefixes full state/clock. Compiled API 14 advance/
28 callback B; raw 115/255, signed cursor/u16 wrap/relative targets/BL-BX/call
return loop, previous owners event 4B/selector 4B, previous 2 new 2/new 2 raw 2,
edges-width-mirror-BIOS-SWI-DMA pass. 314 instructions/70 PCs/32 dispatches coverage
only, not full script/callback ABI/ordinary scene.
[Consumption](../rom_data_scripts.md#managed-publication-and-direct-command-fields).

**ROMDATA-04-SCRIPT-COMMAND-DIRECTORY-CLI completed for EUR/Wine (2026-10-24):**
Windows 18103 production warning-free/canonical restored; Wine 20133 real CLI
creates/reuses without writing 47 managed files from a different cwd, loads old
cache. All 45 generation files match the accepted dataset. One activation boot
frame, 0 command reads: this does not constitute ordinary scene acceptance.

**ROMDATA-05-SCRIPT-COMMAND-DIRECTORY-SCENE completed for EUR/Wine (2026-10-24):**
Trace of producer 1265cc←125b6a/manager 125ac8 and dynamic callbacks; 63 retained
authentic transient EUR snapshots have queued root 0, no replay or modification.
Single Wine 81783 CLI locator with retained authentic input+suspend save through
battle entry 8476/classic 240x160 reads 42B = 14 advance 28 callback/7 commands.
Single Wine 93633 full pair from CLI generation passes all 8476 state-clock/images/
OAM/dispatch and bitwise mixed-PSG-FIFO-HLE audio, same positive coverage. Reuses
test exe/cache/input/save, no rebuild/prepare/oracle/native/lifecycle/old proofs
or fixture creation/navigation/payload edit. Other 108 commands targeted only;
not full ABI/new 32:9. e49 event/selector/dispatch 0: third negative locator
closed, no full pair required for events. 316 hash-mtime guards pass.
[Scene](../rom_data_scripts.md#production-cli-and-positive-ordinary-scene).

**ROMDATA-01-SCRIPT-ARCHIVE completed in the static EUR inventory (2026-10-24):**
115464B/2 ranges: relative archive 114528B B946EC..BB064C and 234 keys 4B/936B
D68C2C..D68FD4. Bounds from independent localized text/coordinate directory roots.
Key domains and first offsets agree with 62 banks/233 slots, 248+932B indices.
213 distinct targets/20 repeated links include 5 links to header and 2 to FF fill:
retained, not rejected/marked dead. 207 physical segments/35048 commands/
110 opcodes/78212 param B; 45 fill spans 88B. 493 links 5a to 49 existing
selectors and real dependency on commands. Key byte 2 selects five text archives
not yet migrated; raw byte 3 retained. Independent reconstruction/all 115464B/
234 keys/233 non-null target formulas/128 halfword 5 word guards/disjoint from
44 owners pass without guest execution. Initial nonempty program assumption
rejected on 46:1/49:1; corrected before plan. No CFG/full argument semantics/
reachability/new guest caps. Inventory and model closed; runtime/CLI/scene
integration covered in units below. Inventory left e50-d49/USA e8-d7 unchanged.
[Contract](../rom_data_scripts.md#relative-script-archive-and-event-key-source-contract).

**ROMDATA-02-SCRIPT-ARCHIVE-MODEL completed for EUR/Wine (2026-10-24):** strict
lossless model of entire 115464B scope, 234 keys/62 banks 233 slots/207 physical
programs 35048 instructions/45 fill; Program/Header/Fill targets and relative words
reconstructed, no ROM shadow/alias copies. Lengths and 493 links depend on actual
command/effect assets. Windows 91283 warning-free/single Wine 85436 passes first
run: 238 key addresses/234 2-level targets (including raw 0 unaligned)/234 text
banks/35054 opcodes/2397 relative targets/493 selector params. All 115464B
reconstructed; 42333 distinct B read at original load sites, 72811 param + 234 key
raw + 86 fill = 73131B retained/hash-checked unread in this scope. 40 guards pass;
224382 instructions 62 PCs coverage only. Not full ABI/interpreter/native/state-
clock/ordinary. e50-d49/USA e8-d7 unchanged; new Linux/USA/native Windows checks
remain to do.
[Model](../rom_data_scripts.md#typed-relative-archive-model-and-reduced-wine-source-oracle).

**ROMDATA-03-SCRIPT-ARCHIVE-RUNTIME completed for targeted EUR/Wine criteria
(2026-10-24):** e51-d50 publishes script-archive.json/engine.script-archive,
46 files 45 resources/all 44 previous decoded files/e50 retained/USA e8-d7 unchanged.
Real command+effect dependencies validated; two bindings 115464B reconstruct typed
indices/opcodes/parameters/keys/fill, without shadow/new guest limits/sequencer/ABI.
Windows 22414 warning-free/Wine 76182 two filters pass on first attempt 13 guards/
lifecycle/all 115464B = 1180 index 35048 opcode 78212 parameter 936 key 88 fill
and 28 original/interpreter/native prefixes full state-clock. Compiled API 37 index
4 opcode/7 parameter 12 key 1 fill B; raw 0/234/239/65535/low16/rotation/header-
fill-target/q0/relative-target/return loop/old palette 4B same page/mixed raw-new/
edges-width-mirror-real BIOS-CpuSet-FastSet-DMA pass. 169 instructions 66 PCs
28 dispatches coverage only; not full ABI/ordinary.
[Consumption](../rom_data_scripts.md#managed-relative-archive-and-typed-guest-reads).

**ROMDATA-04-SCRIPT-ARCHIVE-CLI completed for EUR/Wine (2026-10-24):** Windows
48522 production warning-free/canonical restored; Wine 66454 real CLI creates/
reuses without writing 48 managed files from a different cwd/loads old cache.
All 46 generation files match accepted e51 dataset. Authentic suspend boot-to-
battle 8476/classic 240x160 locator from retained input/save positive: 27B =
8 index 7 opcode 10 parameter 2 key, one bank/program/key, 0 fill. 365 hash-mtime
guards pass; no rebuild old proofs/fixture/navigation/payload edit.

**ROMDATA-05-SCRIPT-ARCHIVE-SCENE completed for EUR/Wine (2026-10-24):** single
Wine 44039 full 8476-frame pair from CLI e51 generation, same existing authentic
input/save/test exe/cache, passes all state-clock/center-wide images/OAM/dispatch/
mixed PSG FIFO bitwise HLE audio, unchanged 27B use and 366 hash-mtime guards.
Does not repeat e50/source-model/native/lifecycle/CLI/prepare/build acceptance.
No failed guest test/fixture/navigation/payload edit. Other 61 banks/206 programs/
233 keys/all 45 fill targeted only; not full ABI/new 32:9. e49 ordinary use positive
remains open after three negative locators; Linux/USA/native Windows remain to do.

**ROMDATA-01-SCRIPT-LOCALIZED-TEXT completed in the static EUR inventory
(2026-10-24):** five roots BB064C/C076F0/C5FBF0/CB7C48/D10644, bounds from
independent language roots and script-key D68C2C: 1803744B/5 ranges disjoint from
45 e51 owners. 63 top entries per language from key max 62/min offset FC, first
entry aliases bank 0, not length. 11 normal links shared per language, 26 bank 61
variants selected by dividing the live byte by 10; separate remainder remaps the
message by 24 with u16 truncation. Original full-u16/low-byte/special/fallback/
root key branches, no new cap. 385 banks/41395 u16 indices including 385 sentinels/
41010 programs 8202 per language; existing Program/customLZ grammar. 9813 compressed
extended/5709 raw extended/25488 raw compact, 212011 commands 11 forms/147 glyphs
1..3112 already in font; real script key and font dependencies. 1260 top +
520 variant + 82790 bank index + 1684407 program + 34767 alignment B. 34314 gaps
1..3B, four mixed: uniform hypothesis rejected before plan/model/Wine, exact bytes
retained. Full reconstruction/hashes 1803744B/15195 formulas/349 halfword 13 word
guards/dependencies/glyphs/disjointness pass without guest execution/build/test or
repeated previous proofs. e51-d50/USA e8-d7 unchanged; model/Wine closed in the
unit below, runtime/CLI/ordinary remain open children.
[Contract](../rom_data_text.md#script-selected-localized-text-source-contract).

**ROMDATA-02-SCRIPT-LOCALIZED-TEXT-MODEL completed for EUR/Wine (2026-10-24):**
strict model of all 1803744B with typed Bank/Variants references, physical payloads
not duplicated, u16 indices/sentinels reconstructed and exact gaps. Existing Program
codec/16-bit header/arguments/encoded-decoded tails/key-font dependencies preserved.
Windows 42377 warning-free/first Wine 95147 fails JSON: unit variant accepted an
extra field; corrected to empty struct retaining assertion, before guest cases.
All first-attempt artifacts retained. Windows 55123 warning-free/single corrected
Wine 34745 passes all data/hashes/1803744B/385 banks 41010 programs and 45 guards:
22 shape 6 source-edit 3 dependency 12 JSON-shape 2 identity. New original cases
7 languages/1170 key-bank/1280 live quotient/256 live remainder/90 raw-special/
1170 u16 indices/837 headers/837 source-vs-typed output+guard, including 249
decompressed bodies/tails. 2116900 instructions 834 PCs coverage only, not native
state-clock/full ABI/ordinary. Initialized checkpoint retained and unchanged,
scratch arguments checked; no gameplay fixture/navigation/checkpoint-save-input
changes or repeated old proofs. 4060 prior artifacts/first attempt/new exe and
368 hash-mtime protected. Only two module declarations in existing files;
e51-d50/USA e8-d7/ABI unchanged.
[Model](../rom_data_text.md#typed-script-selected-text-model-and-reduced-wine-source-oracle).

**ROMDATA-03-SCRIPT-LOCALIZED-TEXT-RUNTIME completed for targeted EUR/Wine criteria
(2026-10-24):** e52-d51/script-text.json/text.script-archives, 47 files 46 resources,
all 45 previous decoded files/e51/USA e8-d7 retained, real key/font dependencies.
Five bindings 1803744B for indices/header/control/glyphs/commands/operands/terminator/
slack, literal map shared with previous text; ABI/save/live guest unchanged. Decoded
tail retained, 0 source literals because generated by LZ/backref: only private
all-class-positive hypothesis rejected. Windows 23146 core warning-free; first
frontend fails only test array 5 vs 11/unused import, test corrected and only
frontend 39830 rebuilt warning-free, initial log retained. Wine 95863 passes three
filters on first launch: single core literal-poison for changed helper, managed
13 guards/lifecycle/all 1803744B/5 languages 385 banks 41010 programs 130 variants
34314 gaps plus 163B previous text 0/1/2, native 35 original/interpreter/compiled
prefixes full state-clock. Compiled API [21,17,17,13,6,4,5,1,3,0,3] B; raw 255/
wrap/sentinel/BB0-D68 previous-new/raw after key owner/width-mirror-real BIOS-CpuSet-
FastSet-DMA pass. 379 instructions 65 PCs 71 dispatches coverage only, not full
ABI/reachability.
[Consumption](../rom_data_text.md#managed-script-text-and-direct-typed-reads).

**ROMDATA-04-SCRIPT-LOCALIZED-TEXT-CLI completed for EUR/Wine (2026-10-24):**
Windows 51427 production warning-free/canonical restored. Wine 71122 real CLI
creates/reuses without writing 49 managed files/different cwd/old cache, all
47 generation files equal accepted dataset; 468 hash-mtime protected. Suspend boot
8476 locator reads 0 new texts: negative closed/no full pair, do not recount old
e49 context.

**ROMDATA-05-SCRIPT-LOCALIZED-TEXT-LOCATOR completed for EUR/Wine (2026-10-24):**
Static audit of 74 EUR snapshots on text fields gives no manager cue; does not prove
dead code, four USA headers excluded before payload/replay. New Wine 24902 locator
school cold boot 3000/classic 0 from retained authentic input, without save/state:
289B = 8 top 4 bank index 8 header 93 control 136 glyph 39 command 1 terminator,
decoded 176B, one language/bank/two programs, 0 variant/operand/tail/slack.
468 hash-mtime pass. No build/prepare/fixture/navigation or checkpoint-save-input
changes.

**ROMDATA-05-SCRIPT-LOCALIZED-TEXT-PAIR completed for EUR/Wine (2026-10-24):**
single Wine 43805 pair 3000 frames from CLI e52 generation/same input/no save-state
override/existing test exe-cache passes every state-clock/center-wide image/OAM/
dispatch/mixed PSG FIFO bitwise HLE audio, same 289B use/468 hash-mtime guards.
No rebuild/prepare/old proofs/new gameplay fixture/navigation/checkpoint-save-input
changes. Other 4 languages 384 banks 41008 programs/130 variants 34314 gaps remain
targeted; 0 ordinary operand/tail/slack.

**ROMDATA-01-SCRIPT-COORDINATE-INDEX-SOURCE / ROMDATA-02-SCRIPT-COORDINATE-PATHS-MODEL
completed for the stated EUR/Wine scope (2026-10-24):** reader 128914/root D68FD4
and reader A1F8/root D69BC4 bound 3056B/210 lists/856 XY pairs/42 FF padding 84B.
Relative directory 840B/u16 count 420B/XY 1712B reconstructed exactly, disjoint
from 46 e52 owners; full 194 halfword 5 word/hashes. 68 script links/56 lists:
66 opcode 44 index byte 2/byte 3 ignored (u16 hypothesis rejected before model),
2 opcode 45 u16. Real script-archive/script-command-directory dependencies; height
from RAM map, original low16 index/low8 destination/u32 count/low8 return, no new
guest cap. Strict lossless model; Windows 25513 warning-free/single Wine 70105
passes on first launch all fields/hashes/24 guards and original execution 212 copies
(all 210 lists + two changed RAM height cells), 13 raw index/destination prefixes
and 13 producer-bytewidth/ignored/nibble/taskargs. 2972 source B observed, 84 padding
preserved-unread; 40116 instructions 134 PCs coverage only. Not native/full state-
clock/full ABI/allocator/task movement/ordinary. e52-d51/USA e8-d7 unchanged,
no turn/save/fixture/navigation fix; new Linux-USA-native Windows checks remain.
[Contract and model](../rom_data_scripts.md#typed-coordinate-path-source-model).

**ROMDATA-03-SCRIPT-COORDINATE-PATHS-RUNTIME completed for targeted EUR/Wine criteria
(2026-10-24):** e53-d52/script-coordinate-paths.json/engine.script-coordinate-paths,
48 files 47 resources/all 46 previous decoded files/e52/USA e8-d7 retained, real
archive/command dependencies. Shared immutable owner for index/count/XY/alignment,
no new ROM shadow/host movement/guest cap/ABI; triple D68 text/keys/coordinates and
raw D69BC4 preserved. First build 10180 passes core/fails only test counter array
5 vs 4; corrected frontend 27578 warning-free/reused core. Wine 30996 managed on
first launch passes 14 guards/lifecycle/all 3056B/210 lists 856 points 42 alignment/
all 46 previous/e52. First native stops on test observer loop 0..5; corrected to
zip 4, retained failed log/EXE/ELF/source, rebuilt only frontend 75147 warning-free.
Wine 31012 native only passes 24 original/interpreter/compiled prefixes full state-
clock, four whole copies/RAM heights/raw args/all classes [28 index 85 count 76 XY
2 alignment] B, mixed owners/edges-width-mirror-real BIOS-CpuSet-FastSet-DMA.
1443 instructions 87 PCs 143 dispatches coverage only, not full allocator/
movement task ABI/ordinary.
[Consumption](../rom_data_scripts.md#managed-coordinate-path-publication-and-typed-reads).

**ROMDATA-04-SCRIPT-COORDINATE-PATHS-CLI / ROMDATA-05-SCRIPT-COORDINATE-PATHS-PAIR
completed for EUR/Wine (2026-10-24):** Windows 29223 production warning-free/
canonical restored exactly. Wine 45975 real CLI creates/reuses without writing
50 managed files/different cwd/old cache, all 48 generation files equal accepted
dataset. Authentic 6000-frame locator from retained state 21420/input positive:
25 bus reads/28B = 4 index 14 count 10 XY/one list, 0 alignment. Single Wine 76926
pair same 6000 frames/state/input/CLI generation passes all state and clock/center-
wide images/OAM/dispatch/mixed PSG FIFO bitwise HLE audio, samples [13167000,
13167000,1584000,1584000,13166910], same 28B/581 hash-mtime guards. No new fixture/
navigation/prepare/locator/old proof repeated. Other 209 lists/all 42 alignment
remain targeted; new 32:9/Linux-USA-native Windows/full ABI unverified, no space/
performance measurements.
[CLI and scene](../rom_data_scripts.md#coordinate-path-production-cli-and-ordinary-scene).

**ROMDATA-01-AUXILIARY-SCRIPT-ARCHIVE-SOURCE completed for the stated EUR
projection (2026-10-24):** reader A1F8/root D69BC4 distinct from the primary archive,
u16 relative indices/u8 action-slot. 4200B: 4098 archive + 102 directory 17 commands
at 370F94. 8 banks/442 slots/126 distinct targets/30 internal entries/133 physical
programs 913 instructions, 900 index + 913 opcode + 2275 param + 10 alignment B.
Root walk does not cover 69 instructions: retained by physical parsing. 489 relative
targets at instruction boundaries; structural CFG 908/913 is not reachability,
other 5 retained. Projection ends D6ABC6/last terminator; following 2 FF excluded.
Graphics base D6B4C8 is only a ceiling, 2306B outside projection; independent
directory successor 370FFC, 2 zero gap bytes excluded. Reconstruction/hashes
4200B/723 halfword 4 word/489 formulas/disjoint from 47 e53 owners pass without
build/guest execution/Linux test. No guest cap/host interpreter/primary dependency/
full ABI. e53-d52/USA e8-d7 unchanged; model/Wine oracle verified below; managed/
runtime/CLI/ordinary still open.
[Contract](../rom_data_scripts.md#european-auxiliary-script-archive-source-contract).

**ROMDATA-02-AUXILIARY-SCRIPT-MODELS completed for the stated EUR/Wine scope (2026-10-24):**
strict directory/archive model 4200B/133 programs/913 instructions with 30 internal
program+instruction entries, parameters/gap/zero row preserved; real dependency on
its own callback/advance, no guest cap/ROM shadow/host interpreter. Windows 66583
warning-free; first Wine 20886 fails test expectation raw 8:0/odd address. Only test
observer/expectation corrected, frontend 54240 warning-free/Wine 3148 passes. Reading
exec.rs establishes 32-bit rotation: additional case 8:1 verifies high bits/target
14D69E97; frontend 52724 warning-free/final Wine 77162 passes, model/core unchanged,
failure/executables/log/intermediate proof retained. 40 guards/all fields-hashes/
723 halfword 4 word, 442 lookup/913 dispatch + RAM opcode 3/80 advance/489 relative
suffixes/10 wrap/15 raw/11 wrapper. 2887 distinct B observed; 1313B = 1297 param +
10 gap + 6 directory zero retained hash-checked-unread. 31839 instructions 177 PCs
coverage only, not predicates/full ABI/native/full state-clock/ordinary. No fixture/
save/input/navigation or repeated old proof; e53-d52/USA e8-d7 unchanged, new
Linux-USA-native Windows checks remain to do.
[Model](../rom_data_scripts.md#typed-auxiliary-archive-and-reduced-wine-source-oracle).

**ROMDATA-03-AUXILIARY-SCRIPT-RUNTIME completed for targeted EUR/Wine criteria
(2026-10-24):** e54-d53 publishes auxiliary directory/archive, 50 files 49 resources/
all 47 previous decoded files/e53 retained/USA e8-d7. Real archive→own directory
dependency, single immutable owner 4200B without ROM shadow/host script/guest cap/
new save-native ABI. Pages 370 audio/D69 coordinates and raw 370FFA+/D6ABC6+
preserved, original control/live cursor/predicates/32-bit LDRH. Windows 44279 core
+ frontend warning-free; Wine 61473 two new filters pass first run 26 guards/
lifecycle/create-reuse-repair-move/failure transaction/e53 retained/unavailable file/
Load-Undo-backup-Restart/same owner/all 4200B/17 commands 8 banks 133 programs 5 gaps
and 44 original-interpreter-compiled prefixes full state-clock. Compiled API
[6 advance 65 callback 18 index 19 opcode 15 param 2 alignment] B, internal entries/
signed cursor/raw opcode/8:1 high bits/full action 2 vs 102/mixed owners/edges-width-
mirror-real BIOS-CpuSet-FastSet-DMA pass. 624 instructions 92 PCs 52 dispatches
coverage only, not full predicates/interpreter/callback ABI/ordinary. 587 hash-mtime
guards pass. No repeated old test/new fixture/navigation/performance/space; real CLI/
positive ordinary use verified in units below.
[Consumption](../rom_data_scripts.md#managed-auxiliary-script-fields).

**ROMDATA-04-AUXILIARY-SCRIPT-CLI / ROMDATA-05-AUXILIARY-SCRIPT-PAIR completed for
EUR/Wine (2026-10-24):** Windows 10336 production warning-free/canonical hash-mtime
restored exactly, main.rs content unchanged/mtime-only Cargo invalidation. Wine
23153 real CLI creates/reuses without writing 52 managed files/different cwd/old
cache, all 50 generation files equal accepted e54 dataset. Single authentic
6000-frame locator from retained state 21420/input positive for both resources:
directory 96B = 32 advance 64 callback/6 commands, archive 82B = 16 index 20 opcode
46 param/2 banks 2 programs, 0 alignment. Single Wine 93545 pair same 6000 frames/
state/input/CLI generation/retained test exe-cache passes every full state-clock/
center-wide image/OAM/dispatch/mixed PSG FIFO bitwise HLE audio, samples [13167000,
13167000,1584000,1584000,13166910], same new use/640 hash-mtime guards. Other
11 commands 6 banks 131 programs/5 alignment spans targeted only; not full ABI/
new 32:9/Linux-USA-native Windows. No repeated rebuild/prepare/locator/old proof/
new fixture/navigation or performance/space measurement.
[CLI and scene](../rom_data_scripts.md#auxiliary-script-production-cli-and-positive-ordinary-scene).

**ROMDATA-01-TERRAIN-MAP-SOURCE completed for the stated EUR path
(2026-10-24):** original loader 1B688/state 6→1AE88 uses root 7662A8/162 records
and all 486 field 04/08/10 links of the 352 already extracted and consumed BG
programs. Verified 14256 descriptor B/398227 packet B/134 alias links, hashes and
exclusive ownership against 49 owners, 40 halfword 11 literal 9 BL. Field 10
initializes height/flag grid 16×16 at 02007CB0; live pointer/dimensions/flags and
EWRAM/VRAM planes remain dynamic. No new resource/decoder/revision or repeated old
test/build; e54-d53/USA e8-d7 unchanged. Accessor 10B1DFB4/other contexts/flag
semantics remain unknown. [Classification and limits](../rom_data_maps.md).

**ROMDATA-01-MAP-RASTER-PALETTE-SOURCE completed EUR static (2026-10-24):**
root 4872B0..489BB0/10496B, 32 uncompressed records/33 ranges/324 selector 54/55
links, 64 index 192 header 10240 color B. Header 3 u16 and 160 u16 colors per
record; 26 different contents but 32 physical identities. Static decoder reconstructs
all file bytes/hashes/links, verifies 67 halfword 13 literal 10 BL and no overlap
with 49 owners. Originals 1EAF0 and 1EDEC preserve different unsigned/signed indices
and FFFF fallback; no new guest cap 32. Original copy 161 halfword/322B reads the
next header or first word 489BB0 already owned by alternate palettes, then rewrites
destination word 160 with word 0. No adjacent owner duplication or read/clock
elimination. No build/guest execution; e54-d53/USA e8-d7 unchanged.
[Contract](../rom_data_maps.md#map-raster-palette-source-contract).

**ROMDATA-02-MAP-RASTER-PALETTE-MODEL completed EUR/Wine (2026-10-24):**
isolated lossless model 32 records/33 ranges/10496B/324 descriptor-dependent links;
header 3 u16/u16 colors/derived index/physical identity without ROM shadow. Windows
30586 frontend test build warning-free; single Wine 39763 new filter first PASS.
All fields/hashes/JSON/15 guards, 64 initializations 32 canonical getters 11 raw
getters 7 unsigned prefixes/3 disabled/2 deferred updates 2 explicit setters 1 DMA
setup pass. 36 CpuSet observations compare all 322B before wrap, including 013C
from the old adjacent owner. Synthetic bit 15/word range/full RAM outside stack/SP/
callee variant pass; 10496 distinct B read at load sites/HLE copies verified.
4753 instructions 202 PCs coverage only, not full state-clock/native/ordinary or
executed raster DMA; 643 hash-mtime guards. e54-d53/USA e8-d7 unchanged, new
Linux-USA-native Windows pending.
[Model and evidence](../rom_data_maps.md#typed-raster-palette-model-and-original-reader-oracle).

**ROMDATA-03-MAP-RASTER-PALETTE-RUNTIME completed for targeted EUR/Wine scope
(2026-10-24):** e55-d54 publishes `map-raster-palettes.json` /
`graphics.map-raster-palettes`, 51 files 50 resources/all 49 previous identical/
e54 retained/USA e8-d7 unchanged. Real BG descriptor dependency/single typed owner
serves 33 ranges 10496B without ROM shadow; previous owner retains word 489BB0/
cross-read. Windows 82612 core/frontend warning-free; single Wine 41457 two new
filters first PASS: 17 guards/lifecycle/Load-Undo-backup-Restart/unavailable files/
all 10496B/32 programs; 28 original-interpreter-native paths full state-clock/
compiled API [15 index 46 header 24 color] B, raw signed/unsigned/sentinels/window/
322B copies/wrap/queued DMA/old owner-raw boundaries. Width/mirror/BIOS bus mode/
four SWI/one immediate DMA pass; 646 hash-mtime guards. 638 instructions 204 PCs
55 dispatches coverage only, not full camera/raster DMA/ordinary ABI. Source/model
already closed not rerun.
[Runtime and limits](../rom_data_maps.md#managed-raster-palette-publication-and-typed-reads).

**ROMDATA-04-MAP-RASTER-PALETTE-CLI / ROMDATA-05-MAP-RASTER-PALETTE-PAIR
completed for the stated EUR/Wine scope (2026-10-24):** Windows 26915 production
warning-free/canonical hash-mtime restored/main content unchanged-Cargo mtime.
Wine 40160 real CLI creates/reuses without writing 53 managed files/different cwd/
old cache, all 51 generation files equal accepted e55. Single authentic cold boot
1500 retained demo input/no save-state/override locator positive 50880 aggregate B =
2 index 322 header 50556 color/25440 bus reads/mask 2 records (includes neighbor's
header, not two complete palettes or distinct bytes). Single Wine 90978 pair same
1500 frames 0..1500/CLI generation/test exe-cache passes every state-clock/image/
OAM/dispatch/mixed PSG FIFO bitwise HLE audio, samples [3291096,3291096,395212,
395212,3269758], same new use/701 hash-mtime guards. Source/model/targeted and old
proofs not rerun; no new fixture/navigation/performance/space. Other contexts/full
camera-raster ABI/new 32:9/Linux-USA-native Windows remain pending.
[CLI and scene](../rom_data_maps.md#raster-palette-production-cli-and-positive-ordinary-boot).

**ROMDATA-01-OBJ-EMPTY-CALLERS completed within the direct EUR static scope
(2026-10-24); unproved hold extension rejected:** the three old empty headers
B4BD20/bank0seq1, B4BD34/bank1seq1, B4FA10/bank14seq6 remain unchanged with
frames=[]/without zero_count_hold. Twelve constructors of the first two banks use
0; 18 callers of initializer 8BBF0 produce modeword 0..3→sequences 0..3; two updates
use 2/3. All 30 root literals/47 instruction fingerprints/50 owners verified;
constants across calls according to preserved r4..r11 ABI. Six B of possible
selectors remain raw; durations 8/16/4 alias already typed indices. Raw mode
7/65539→6/7, repeat 1: no guest cap or global unreachability proof. New producers/
ordinary trace with established scheduling are the only reason to reopen; indirect
calls/RAM mutations unverified. No new model/resource/revision/build/guest execution;
e55-d54/USA e8-d7 unchanged.
[Contract](../rom_data_obj.md#earlier-empty-sequence-caller-domains).

**ROMDATA-01-UI-LIST-ENTRIES-SOURCE completed within the EUR static scope
(2026-10-24):** 18 direct callers 8BBF0→16 ROM entries/12 lists 36 entries and
two RAM filters 0200F250 with 6-entry template 397A9C. Thirteen ranges 42 entries
336B, disjoint from 50 owners; all 18 literals/93 fingerprints and byte
reconstruction pass. u8 ID/X/Y/opaque fields and raw left/right/up/down; no
normalized pointer/cap/count/target. Signed copy count word distinct from live byte;
getter 8CA64/input 8BDEC with coordinates from unmasked raw index. Projections of
32 live masks establish template extent; ID/Y/up/down and selection remain RAM.
clear-down→base copy→up order preserves down 1 with zero slots, corrected during
private static review. No new model/resource/revision/build/guest execution;
e55-d54/USA e8-d7 unchanged. [Contract](../rom_data_ui.md).

**ROMDATA-02-UI-CHOICE-LISTS-DATA completed for EUR/Wine (2026-10-24):** public
strict lossless/shared source plan model for 13 tables 42 entries 336B/physical IDs,
byte 3 opaque and 4 raw navigation fields without guest cap. One targeted Windows
build warning-free plus a single Wine filter pass 17 guards/all 336B original load
sites/18 copy prefixes (13 sources + 1 variant + 4 raw counts 0/-1/-65536/257)/
44 getters/43 idle/64 prefixes two filters 32 masks/2 coordinate prefixes unmasked
flags 80-86. EWRAM/IWRAM outside stack and SP/callee returns equal; not full widget/
native/full state-clock/ordinary. 703 hash-mtime guards; 12231 instructions 212 PCs
coverage only. Variant all 8 u8 fields passes shape and fails authentication; copy
257 preserves original metadata overwrite. No old proof/source census repeated or
new fixture/navigation/performance/space. e55-d54/USA e8-d7 unchanged.
[Model and evidence](../rom_data_ui.md#lossless-model-and-original-wine-readers).

**ROMDATA-03-UI-CHOICE-LISTS-RUNTIME completed for targeted EUR/Wine scope
(2026-10-24):** e56-d55 publishes ui-choice-lists.json/engine.ui-choice-lists,
52 generation files 51 resources/all 50 previous decoded equal/e55 retained/
USA e8-d7 unchanged. No asset dependency, coherent full set. Immutable owner
13 spans 336B serves ID/X/Y/opaque/raw navigation to CPU/native APIs and BIOS/DMA
copies; RAM counts/filters/copies/input/cursors/tasks and save-native ABI unchanged.
Windows 36248 single core/frontend build warning-free; core tests compiled only.
Wine 82471 two exact filters first PASS 17 guards/lifecycle/all 336 typed B =
[42 ID 84 coordinates 42 opaque 168 nav]/13 lists/transactional failed attach/
failed publication-create-reuse-repair-move/e55 retention/unavailable Load-Undo-
backup-Restart/shared owner. Native 29 original-interpreter-compiled paths full
state-clock/compiled [92 ID 183 coordinates 92 opaque 368 nav] aggregate B; all
336 new raw + 14 old OBJ raw B altered. Counts 0/-1/257/RAM copy/getter-idle/
4 filter prefixes zero-all, widths/mirrors/real BIOS bus/3 SWI/1 DMA pass. Mixed
copy 2 old + 2 raw + 2 new B/SWI-DMA and API word 2 raw + 2 new, old OBJ same
page 4B; 706 hash-mtime guards. 3010 instructions 186 PCs 38 dispatches coverage
only, not full widget/ordinary ABI. Source/model/old proof not repeated; no new
fixture/navigation/performance/space.
[Consumption and evidence](../rom_data_ui.md#managed-publication-and-typed-runtime-reads).

**ROMDATA-04-UI-CHOICE-LISTS-CLI / ROMDATA-05-UI-CHOICE-LISTS-PAIR completed for
the stated EUR/Wine scope (2026-10-24):** one Windows production build 60923 first
PASS warning-free/frozen 444c2db9..; canonical hash/mtime exactly restored, main
content unchanged/mtime-only Cargo invalidation. CLI 36895 creates/reuses 52
generation files 54 managed no-write/all 52 equal accepted e56/unrelated cwd/old
cache. Pecorelle 1000 0 new UI, Moogle-menu 6000 new UI: terminal negatives/no full
pairs/extensions. Readonly inspection of 110 EUR snapshots without direct copies
does not prove unreachability; battle menu is a different component. Existing
Partyunit screenshot/input guide single CLI 54355 320 positive 32 aggregate B =
[4 ID 8 coordinates 4 opaque 16 nav]/8 bus reads/1 list. Worldmap snapshot
7701a360../Partyunit input bf757e69.. unchanged, no new fixture/navigation/payload
edit. Single Wine 95582 full pair same 320 frames 4519→4839/every state-clock/
center-wide image/OAM/dispatch/mixed PSG FIFO bitwise HLE audio matches; audio
[702240,702240,84480,84480,702164]. Same 32 new B/1 list/769 hash-mtime guards.
Frozen 0f942581 test exe/actual CLI DQ1nbu/old cache/width 0, mixed native-fallback.
Source/model/managed/native/old scene not repeated; no all-native/full widget/
new 32:9. Ordinary use of other 12 lists pending; targeted checks closed; new
Linux/USA/native Windows/32:9 remain pending.
[CLI and scene](../rom_data_ui.md#production-cli-and-positive-ordinary-party-scene).

**ROMDATA-01-UNIT-OBJ-INITIAL-POSE / ROMDATA-02-UNIT-OBJ-INITIAL-POSE completed
within the EUR/Wine source/model scope (2026-10-24):** original 2D394→233A8→CF144
produces IDs from already migrated metadata52 property 4/metadata14 halfword 04;
2D3F8→22788 passes fullword animation 0/part 0. 110 job route 0 + 93 special project
121 IDs, without guest cap or complete physical catalog; job 0/1 value 65535
remains unresolved/unexported. Directory 394834→first cells→121 sequences
477 frames 20B operation 1, OAM base A50020/6 lists 31 objects→original tile count
221D4→333 requests 135168B raw 4bpp from base 898A40. 00B14 queues DMA, does not
decode; 00800 performs actual DMA3. 415 spans 146358B disjoint from 51 owners;
94 instruction 13 BL 6 literal static fingerprints PASS. Strict lossless/shared
source plan model/4224 8x8 palette-index tiles preserves header/tail/opaque
attributes. Windows 20506 single frontend test build warning-free/frozen 1d759b0d;
Wine 19197 single filter first PASS 121 original resets 22E20/1 constructor prefix
22788/477 scheduler→queue→DMA upload/all VRAM/all 135168 pixel B/16 shape-size
including shape 3/11 JSON-auth guards/SP-callee/outside RAM. 9393 distinct load-site
B + 135168 DMA = 144561B; 1797B header/tail/OAM preserved unread; 245830 instructions
499 PCs coverage only. 703 hash-mtime guards; no full unit ABI/native/full state-clock/
ordinary, production build/old test/fixture/navigation/performance/space. EUR e56-d55/
USA e8-d7 unchanged. [Contract/model](../rom_data_unit_obj.md).

**ROMDATA-03-UNIT-OBJ-INITIAL-POSE completed within the targeted EUR/Wine scope
(2026-10-24):** e57-d56 publishes unit-obj-initial-pose.json/graphics.unit-obj-initial-
pose with metadata52/14 dependencies; 53 files 52 resources/all 51 previous decoded/
manifest coverage records identical/e56 retained/USA e8-d7. Immutable owner returns
directory/index/header/frame/OAM from typed fields and pixels from 4bpp indices;
no altered ROM shadow/RAM-scheduler-DMA-VRAM/save-native ABI. Windows 73480 single
core/frontend build warning-free; core tests compiled only 7a95a911../frontend
d0995de8.. frozen. Wine 19825 two exact filters first PASS 17 guards/lifecycle/all
146358 typed B = [484 directory 484 index 484 header 9540 frame 198 OAM 135168 pixel]/
121 sequences, transactional failed attach/failed publication-create-reuse-repair-
move/unavailable Load-Undo-backup-Restart/shared owner. Native 30 original-interpreter-
compiled paths full state-clock/API [28 dir 20 index 8 header 127 frame 164 OAM
1 pixel] aggregate B; pixel API probe distinct from actual compiled DMA executors.
Raw 146358 new + 20 old OBJ B altered, ID 65535/animation 65537/live RAM frame pass.
415 edges/width/mirrors/BIOS bus/2 SWI/1 DMA: mixed copy 2 new + 1276 raw + 8 old
OBJ with charges 2/8; API word 2 new + 2 raw edited; old OBJ same page 4B without
new charges. 706 hash-mtime guards/8 compiled inputs unchanged; 3542 instructions
496 PCs 177 dispatches coverage only. Source/model/old proof not repeated; no full
unit/allocator ABI/ordinary/fixture/navigation/performance/space.
[Consumption and evidence](../rom_data_unit_obj.md#managed-publication-and-direct-runtime-reads).

**ROMDATA-04-UNIT-OBJ-INITIAL-POSE-CLI / ROMDATA-05-UNIT-OBJ-INITIAL-POSE-PAIR
completed within the stated EUR/Wine scope (2026-10-24):** Windows 49887 single
production build warning-free/frozen d4c782c6..; original canonical hash/mtime
restored, main content unchanged/mtime-only Cargo invalidation. Wine 57046 three
real CLI create/reuse-no-write 55 managed files/all 53 generation files equal accepted/
unrelated cwd/old cache. Actual e57 generation-ZV0TqS/ab9286b2..; original worldmap
7701a360../Partyunit input bf757e69.. 320 without new fixture/navigation/payload
edit. Positive locator 10962 aggregate B = [8 dir 16 index 4 header 252 frame
7162 OAM 3520 pixel]/8170 bus reads/2 sequences. Single Wine 12785 full pair same
320 frames 4519→4839/every state-clock/center-wide image/OAM/dispatch/mixed PSG
FIFO bitwise HLE audio matches; audio [702240,702240,84480,84480,702164]. Same
10962B/3520 pixel B/2 sequences/764 hash-mtime guards; frozen d0995de8 test exe/
actual CLI dataset/old cache/width 0, mixed native-fallback. Small wrapper process-
exit preflight corrected before any Wine, no functional test repeated. Source/model/
managed/native/old scene/build/prepare/locator not repeated; Partyunit endpoint with
animated 240x160 preview inspected. No all-native/full unit-allocator/new 32:9 claim.
Other 119 ordinary sequences targeted only; new Linux/USA/native Windows remain
pending. [CLI and scene](../rom_data_unit_obj.md#production-cli-and-positive-ordinary-party-scene).

**ROMDATA-01-UNIT-OBJ-PALETTE-SOURCES / ROMDATA-02-COLOR-MODE-PALETTES
completed within the EUR/Wine source/model scope (2026-10-24):** five helpers
CF49C/CF4E4/CF51C/CF5A0/CF5DC and 70 direct calls prove 30 spans 5750B = 2873 u16
colors + 4 u8 slice selectors without overlap with 52 owners. Complete 512B copies,
fragmentary supplemental copies, 12B shared and 9 special palettes from 51 metadata14
projections verified; no extent inferred from adjacency or guest cap. 86 instruction
19 literal 11 BL pins pass. Strict lossless/shared source plan model and corrected
Wine filter pass 165 original pointer/99 copy/30 staged reader/54 special copy/
12 slice prefixes/all 5750B/eight guards. First filter stopped by incorrect r5/r6
seed in test 30632; model unchanged; correction uses original MOV/LDR. Two Windows
frontend builds warning-free, no production build or old proof repeated. 764 hash-
mtime guards/four compiled inputs intact; 33654 instructions 770 PCs coverage only.
EUR e57-d56/USA e8-d7 unchanged; no full state-clock/native/ordinary claim.
[Contract and evidence](../rom_data_color_palettes.md).

**ROMDATA-03-COLOR-MODE-PALETTES completed within targeted EUR/Wine scope
(2026-10-24):** e58-d57 publishes color-mode-palettes.json/graphics.color-mode-
palettes with metadata14 dependency/54 files 53 resources/all 52 previous decoded/
manifest coverage records identical/e57 retained/USA e8-d7. Immutable u16 colors/
u8 selectors owner preserves raw gaps/prior owner by byte, live mode/copies/fades/
RAM/save-native ABI. First build 16832 stopped on 3 E0061 in test constructors
before Wine; three zero arguments added, no old test run. Build 13589 core/frontend
warning-free/core compiled only/frozen frontend 6725b65c... Wine 12913 two exact
filters first PASS 17 guards/lifecycle/all 5750 typed B = [1536 unit 1024 companion
2286 supplemental 36 shared 864 special 4 selector]/30 fragments/transactional
failed attach/failed publication-create-reuse-repair-move/unavailable Load-Undo-
backup-Restart shared owner. Native 28 original-interpreter-compiled paths full
state-clock; aggregate API [1532,1536,158,44,192,7] B six categories. Raw 5750 new
+ 4 old UI B altered; low8 index 65535/mode 3/live RAM/30 edges/width/mirrors/BIOS
bus/2 SWI/1 DMA pass. Mixed copy 2 new + 20 raw + 4 old UI B with exact charges
2/4; API word 2 new + 2 raw edited; old UI 40FCB0 same page 4B without new charges.
767 build guards 769 runtime guards/nine compiled inputs intact; 6805 instructions
451 PCs 5177 dispatches coverage only. No full allocator-scheduler/ordinary claim;
source/model/old proofs not repeated.
[Consumption and evidence](../rom_data_color_palettes.md#managed-publication-and-typed-runtime-reads).

**ROMDATA-04-COLOR-MODE-PALETTES-CLI / ROMDATA-05-COLOR-MODE-PALETTES-LUTIA-PAIR
completed within the stated EUR/Wine scope (2026-10-24):** Windows 28568 single
production build warning-free/frozen a052914b..; canonical hash/mtime exactly
restored, main content unchanged/mtime-only Cargo invalidation. Wine 22334 CLI
create/reuse-no-write 56 managed/all 54 generation files equal accepted/unrelated
cwd/old cache, actual e58 generation-XbOkZs/e7abf8bb... Partyunit 320 0 new colors;
neutral boot 240 Wine 46838 0 new colors: terminal negatives/no full pairs/no new
input/fixture. Existing Lutia map load recording guides single Wine 22903 1700
positive 1102 aggregate B = [512 unit 512 companion 78 supplemental 0 shared
0 special 0 selector]/277 bus reads/4 fragments. Worldmap 7701a360../worldmap-
engage-battle.input unchanged, no new navigation/payload edit. New test guard
REQUIRE_COLOR_MODE_PALETTES requires positive typed colors; Windows 46600 single
frontend build warning-free/frozen 449a1d2c... Single Wine 16883 full pair same
1700 frames 4519→6219/every state-clock/center-wide image/OAM/dispatch/mixed PSG
FIFO bitwise HLE audio matches; audio [3730650,3730650,448800,448800,3730574].
Same 1102B/4 fragments/828 guards/ten compiled inputs intact; old 9 runtime inputs
unchanged. Lutia NOTICE 240x160 endpoint inspected; mixed native-fallback, no
all-native/full allocator-scheduler/new 32:9. Source/model/managed/native/old proofs
not repeated. Other 26 ordinary fragments/other modes/shared-special-selector
remain targeted-only; new Linux-USA-native Windows pending.
[CLI and scene](../rom_data_color_palettes.md#production-cli-and-ordinary-lutia-palette-consumption).

**ROMDATA-01-UNIT-OBJ-EQUIPMENT / ROMDATA-02-UNIT-OBJ-EQUIPMENT completed within
the EUR/Wine source/model scope (2026-10-24):** original 9BA84→CE1EC properties
3/9 select 252 items/61 sprites 124..184, bases 0/4/8, parts 0/1; explicit overrides
185/bases 0-4 and 124-126/base 16 included. 748 cells/86 null/258 sequences
3439 frames: 1911 op1/609 op6/888 op8/31 op0, only op1 produces graphics references.
593 ranges 110666B, no overlap with all 53 e58 owners: 248 directory 2992 indices
69812 sequences 174 OAM 37440 pixel 4bpp/1170 tiles. 73 instruction/4 BL/4 literal/
2 dispatch static; typed lossless model with item dependency and real nulls, no
guest cap. Single Windows test build warning-free; single new Wine filter passes
first attempt 10 guards/1496 constructor prefixes/86 null resets/258 sequence
resets/six commands/280 scheduler-DMA/all 37440 pixel B. 828 previous guards
intact, seven compiled preimages retained; later action 12/13 comment-only correction
does not change implementation. No old proof/build/fixture repeated. Managed
publication/runtime/ordinary scene still to do, EUR e58-d57/USA e8-d7 unchanged.
Normal/alternate actor and script 49 census remains partial: no script animation ×
sprite Cartesian product. Linux/USA/native Windows remain to do, no performance
measurement. [Contract and evidence](../rom_data_unit_obj.md#equipment-poses-source-model).

**ROMDATA-03-UNIT-OBJ-EQUIPMENT completed within targeted EUR/Wine scope
(2026-10-24):** e59-d58 publishes 55 files 54 resources, unit-obj-equipment.json/
graphics.unit-obj-equipment with actual item-records dependency; all 53 previous
files/resource records/coverage identical, e58 retained, USA e8-d7. Immutable owner
reads fields and 4bpp indices for 593 ranges 110666B without ROM shadow, with real
nulls and byte-by-byte composition with previous owners/raw holes. Task/commands/
facing/RAM copies/scheduler/DMA/VRAM/save-native ABI remain original. Single Windows
core/frontend test build warning-free, core compiled only. Two new Wine filters
first PASS 17 guards/lifecycle/all 110666 typed B/258 sequences and 26 original-
interpreter-compiled paths with equal full state-clock, null/parts 0-1/fullword
ID-animation/commands 0-6-8/live RAM frame/scheduler-DMA. Compiled APIs aggregate
[32 directory 24 index 6 header 47 frame 32 OAM 1 pixel] B; actual DMA compared
separately. 593 edges/width/mirror/BIOS/2 SWI/1 DMA pass; mixed 2 new + 1028 raw +
8 old OBJ with charges 2/8 and unaligned API 2 new + 2 raw edited. 831 build guards/
833 runtime/11 source inputs intact. 1511 instructions 502 PCs 97 dispatches coverage
only, no full allocator-scheduler/ordinary claim; source/model/old proofs not
repeated. Dataset identity 2c200038../manifest file SHA aa8bba05.. distinct; CLI
and positive ordinary still open. Linux/USA/native Windows/new 32:9 remain to do;
no metrics. [Publication and consumption](../rom_data_unit_obj.md#equipment-managed-publication-and-runtime-reads).

**ROMDATA-04-UNIT-OBJ-EQUIPMENT-CLI completed (2026-10-24):** single Windows 85707
production build warning-free/frozen 0795d057..; canonical hash/mtime exactly
restored, main content unchanged/mtime-only Cargo invalidation. Wine 87287 actual
CLI create/reuse-no-write 55 generation/57 managed/all 55 equal accepted/unrelated
cwd/old cache; e59 generation-nXJuGC/2c200038... Sheep 1000 locator and Wine 22920
Fight 700 locator are terminal equipment-negative: 0 all categories/0 sequences,
no full pair/test guard build/unchanged repetition. Old palette program scene proofs
are not equipment-use evidence. Spell snapshot precondition initially assumed desktop
wrapper; actual raw FFTA-ST/version 1/EUR/payload verified without previous build/
guest. F09 preview/core/hash pass. 833 runtime/891 CLI/893 Fight guards/11 source
inputs/all 7623 previous artifacts intact. Static inspection reuses 211 candidate
9BA84 call sites, does not prove liveness/unique definitions. dump_frame shows guest
framebuffer, not necessarily native UI.
[CLI and ordinary limits](../rom_data_unit_obj.md#equipment-production-cli-and-ordinary-coverage-limits).

**ROMDATA-05-UNIT-OBJ-EQUIPMENT-SCENE completed within the stated EUR/Wine scope
(2026-10-25).** The user explicitly authorizes local checkpoints from a EUR copy
using only normal commands, originals/cartridges unchanged and artifacts outside
Git; previous automatic refusal superseded, no RAM/HP/status/PC changes. Derivation
F09 frame 8476→8668 in 192 normal frames confirms Action/Fight/ally Hildegard. Two
empty squares and first A 500 stopping at Do it are closed negatives, not repeated.
New A input at 0/48 (release 4/52) actually confirms the attack: actual e61 CLI
500 positive 4477B = 8 dir 16 index 4 header 276 frame 2765 OBJ 1408 pixel/two
sequences. Only positive pixel+sequence guard in scene test; single Windows frontend
build warning-free/frozen a13b3aa6..; single Wine 500 pair first PASS frames
8668→9168 every state-clock/center-wide image/OAM/dispatch/audio, equipment counters
identical to CLI (3269 reads). Source/model/managed/native/actual CLI/ordinary
equipment closed, other 256 ordinary sequences remain targeted-only. Old Sheep
1000/Fight 700/source/native/scene/build not repeated; e61/USA e8 unchanged.
General actors/alternate/script/other palette poses and new Linux/USA/native Windows/
32:9 remain; main 01..05 partial/mod 06-07 dependent. No performance/space
measurement. [Authorized scene and limits](../rom_data_unit_obj.md#authorized-ordinary-equipment-attack).


**ROMDATA-01-UNIT-OBJ-NORMAL-BATTLE / ROMDATA-02-UNIT-OBJ-NORMAL-BATTLE completed
for the stated EUR/Wine projection (2026-10-24):** 121 normal metadata52/14 IDs +
42 explicit, 29 body bases/8 extra parts resolve 10980 keys/8374 physical cells/
5293 null/2581 sequences 17894 frames. 150 lists 676 objects and 3216 4bpp requests
preserve indexed pixels and commands 0/1/2/3/6/7/8. Shared views 146358B initial +
6984B equipment verified; 6021 new ranges 1455474B, no other overlap among current
54 owners/no guest limit or adjacency-based extension. Single warning-free Windows
build/single new Wine filter first PASS: 12 guards/16474 constructor prefixes over
8237 new cells/5293 null resets/2451 sequence resets/100 commands/2858 scheduler-DMA/
all 1063200 new pixel B/full VRAM and guards. Original checks of old owners not
repeated. Finite projection, not proof of reachability of every actor/action
combination. Unrestricted product with alternate IDs rejected: sprite 247/animation
104 reads directory 394834 as index, no frame count filter resolves missing linkage.
Alternate/script 49/RAM stream remain open. e59-d58/55 files 54 resources and USA
e8-d7 unchanged; next 03 managed publication/typed consumption, then CLI/ordinary
use. Linux/USA/native Windows/new 32:9 remain to do, no measurement.
[Contract and evidence](../rom_data_unit_obj.md#normal-mode-battle-poses-source-projection-and-typed-model).


**ROMDATA-03-UNIT-OBJ-NORMAL-BATTLE completed within targeted EUR/Wine scope
(2026-10-24):** e60-d59 publishes/consumes unit-obj-normal-battle.json with real
metadata52/14/initial/equipment dependencies, 56 files 55 resources/all 54 previous
records and decoded files unchanged/e59 retained/USA e8 unchanged. Reader intersects
semantic views with 6021 new ranges 1455474B; no duplicate owner or raw shadow.
One warning-free Windows build/two new Wine filters first PASS: 17 guards/lifecycle/
all 1455474 typed B/2451 sequences and 32 original/interpreter/compiled prefixes
with equal guest states and clock. Parts 0/1/2/nulls/fullword ID/live RAM/commands
0-2-3-6-7-8/two uploads + two DMA/all 6 API classes/6021 edges/mirror/BIOS/SWI-DMA
pass. Mixed window 2 new 1036 raw 316 initial 248 equipment 8 previous OBJ preserves
exact charges; unaligned API 2 new + 2 altered raw passes. Model/source and previous
checks remain closed/not repeated. Next 04 real CLI create/reuse-without-writes/
old cache and 05 positive ordinary use of the new normal body family; alternate/
script/RAM stream and Linux/USA/native Windows/new 32:9 remain open. No navigation/
new checkpoint or performance measurement.
[Contract and evidence](../rom_data_unit_obj.md#normal-battle-managed-publication-and-runtime-reads).


**ROMDATA-04-UNIT-OBJ-NORMAL-BATTLE-CLI / ROMDATA-05-UNIT-OBJ-NORMAL-BATTLE-PAIR
completed within the stated EUR/Wine scope (2026-10-24):** single warning-free
Windows production build/canonical hash-mtime exactly restored; real CLI creates and
reuses without writes 56 generation files/58 managed, all equal to accepted e60,
from a different cwd/old cache. Existing unchanged F09 Wait 3530 recording with
monster movement is positive: 842236 new B = [40 index 10 header 26826 frame
815360 pixel]/5 sequences. One test-only positive guard/single frontend build/
single full Wine pair 3530 first PASS: every full state/guest clock/center-wide
image/OAM/dispatch and mixed/PSG/FIFO/HLE audio sample bitwise equal; counters match
CLI exactly. All 958 guards/14 compiled inputs/preimages/8083 previous artifacts
preserved. Other 2446 new sequences remain targeted/not an exhaustive playthrough.
Source/model/managed/native/CLI/ordinary children closed, no repetition or new
fixture/navigation/checkpoint/payload/save. Alternate/script/RAM stream remain open;
05 equipment positive later closed on October 25 after explicit authorization and
single new Fight 500 pair. Linux/USA/native Windows/new 32:9 remain to do, no
performance/space measurement.
[Evidence and limits](../rom_data_unit_obj.md#normal-battle-actual-cli-and-ordinary-equivalence).


**ROMDATA-01-UNIT-OBJ-ALTERNATE-TARGET-VIEWS — static contract completed
(2026-10-24), model/oracle still to do:** 65 alternate IDs/5850 keys,
5682 physical cells = 4709 null + 953 with 706 sequences 2771 frames + 20 shared
index-root views. All 20 have independent initial-pose directory/index fields as
witnesses; sole internal overlap 4B is 247/anim 104→directory/root sprite 0. Old
blanket interpretation as sequences remains rejected: no count/cap filter, removed
key or adjacency bound. 30 OBJ lists 108 objects/1203 pixel requests 387008B;
3321 new ranges 410086B after 6232 initial + 52604 normal-battle + 22 equipment
shared, no other overlap among 55 e60 owners. New 606 sequences 2210 frames/5252
cells 4428 null/1079 pixel requests 342112B. Contract and SHA bdf15f7d.. verified
statically; next 02 model with tagged targets and explicit dependencies, then single
new Wine oracle of original readers. No new generation/runtime/scene claimed; e60/
USA e8 unchanged. Actor/action reachability, script 49/RAM stream/Linux/USA/native
Windows/new 32:9 remain open, no measurement.
[Contract and limits](../rom_data_unit_obj.md#alternate-battle-selectors-and-explicit-shared-target-views).


**ROMDATA-02-UNIT-OBJ-ALTERNATE-BATTLE completed within targeted EUR/Wine scope
(2026-10-24):** public lossless model with tagged null/sequence/shared-index-root
targets preserves all 5850 keys/20 views tested against initial-pose fields,
706 sequences 2771 frames/30 OBJ lists/1203 4bpp blocks. 3321 new ranges 410086B
after validation of the three shared owners; sole 4B directory/index overlap with
exact witness. One warning-free Windows frontend build/single Wine filter first
PASS: 15 guards/10544 constructor prefixes (40 shared aliases)/4428 null resets/
606 sequence resets/five new commands 2-3-8/1079 scheduler-DMA/all 342112B new
pixels/full VRAM. No false frame parsed or scheduled; previous original opcode 6
and others not repeated. Source/model closed; next 03 managed publication/typed
consumption, then CLI/ordinary use. e60/USA e8 unchanged; Linux/USA/native Windows/
new 32:9 remain to do, no space/performance measurement or new gameplay checkpoint.
[Model and evidence](../rom_data_unit_obj.md#typed-alternate-target-model).


**ROMDATA-03-UNIT-OBJ-ALTERNATE-BATTLE completed within targeted EUR/Wine scope
(2026-10-24):** e61-d60 publishes/consumes unit-obj-alternate-battle.json with five
real dependencies metadata52/14/initial/equipment/normal; 57 files 56 resources/
all 55 previous records/coverage/decoded unchanged/e60 retained/USA e8 unchanged.
Only 3321 new ranges 410086B are owned; twenty shared index roots remain non-sequences
and previous owners retain responsibility. One warning-free Windows core/frontend
build (core compiled only), two new Wine filters first PASS: 20 guards/lifecycle/
all 410086 typed B/606 sequences; 34 original/interpreter/compiled paths with equal
full state and clock, part 0 sequence/part 1-2 null/shared-root prefixes/fullword
ID/live RAM/commands 2-3-8/two uploads + two DMA/all 6 APIs/3321 edges/mirror/BIOS/
SWI-DMA. Mixed window 2 new 780 raw 8 previous OBJ and API 2 new + 2 altered raw
preserve exact charges. Target::Null empty struct rejects extra fields previously
ignored by serde; valid JSON identical cad7771d.., no repeat of physical oracle.
All 8216 previous artifacts/15 compiled inputs/preimages/961 build + 963 Wine guards
preserved. Source/model/managed/native children closed; next 04 real CLI and 05
positive ordinary use of new family. Reachability/script 49/RAM stream/05 positive
equipment and Linux/USA/native Windows/new 32:9 open; no performance-space measurement
or new gameplay checkpoint.
[Contract and evidence](../rom_data_unit_obj.md#alternate-managed-publication-and-typed-runtime-reads).


**ROMDATA-04-UNIT-OBJ-ALTERNATE-BATTLE-CLI completed EUR/Wine; 05 positive open
(2026-10-24):** single warning-free production build/frozen d3db6585../canonical
hash-mtime exactly restored; real CLI creates/reuses 57 generation files 59 managed
without rewrites, all equal to accepted e61, from unrelated cwd. One frame with old
cache and unchanged authentic F09 snapshot passes: 0 alternate use, activation smoke
only and no repeat of already closed normal pair 3530. Static/authenticated read of
81 existing EUR snapshots finds no current task with a new alternate sequence that
provides a concrete positive-gate candidate. Does not exclude future use in
recordings; no long locator/scene guard/test build/full pair or new navigation/
checkpoint/payload edit. First static helper confused guest pointer/file offset:
preimage/invalid results retained, only cheap static inspection corrected, no game
acceptance invalidated. 8543 previous artifacts/1023 CLI guards/15 compiled runtime
inputs-preimages unchanged. Source/model/managed/native/actual CLI closed; 05 positive
ordinary alternate use still open; 05 positive equipment later closed on October 25
with authorized checkpoints and single new Fight 500 pair. Script 49/live/new
Linux-USA-native Windows-32:9 remain open, no measurement.
[CLI and limits](../rom_data_unit_obj.md#alternate-actual-cli-and-remaining-ordinary-use-gate).


**ROMDATA-01-SCRIPT-OBJ-ACTOR-PAIRING: selector classification completed,
sprite pairing still open (2026-10-24).** Decimal 49 (0x31), callback
126CC0, retains 167 instructions/18 selectors/13 bases. 151 lookups in RAM units
(12+24 records stride 264/identity byte 4), 16 in the live registry/fallback; 0
immediate sprite IDs. The context lookup compares unit pointers and uses current
sprite +34. The u16×4 low16 group/signed facing +1F/body flag 0/same extra base
are original. 31 reuses existing units without changing byte 01; 32 only
repositions them, so file order does not prove dominance or an unchanged sprite.
Existing scripts and provenance/67 halfwords 9 literals 18 BL authenticated; no
new ownership/model/runtime/review/build/guest test. Earlier hex/decimal notation
corrected and one BL pin moved to the first halfword, without repeating closed
acceptances. Next useful evidence: only sprite/animation/part queries resolved
during 49 in an unchanged EUR recording, retaining the instruction/actor/reader
link. No indiscriminate scripts×sprites product. e61/USA e8 unchanged; new Linux-
USA-native Windows-32:9 to do, no measurement or new fixture/navigation/checkpoint.
[Live selector contract](../rom_data_unit_obj.md#script-animation-selectors-require-live-actor-bindings).


**ROMDATA-01-SCRIPT-OBJ-RESOLVED-QUERIES completed for the targeted EUR recording
(2026-10-24):** test-only observer/cfg(test) recording, no production code/source
owner/ABI/save/cache modified. First Wine preflight fails before snapshot/frame
because it assumed all entry points were native; evidence/preimage retained. Only the observer
was corrected: native function forwarding where available, original Machine::step
where cache was missing. Corrected frontend build warning-free/frozen 4342f2..;
sole actual Wine census 6000 frames 21420→27420 passes with six handler 49/six body
SP-LR linked/18 real constructor queries. Six selector 16/group 2 instructions
resolve sprite 46/animation 8/parts 0-1-2; native availability [false,false,true],
instrumented accounting does not prove dispatch equivalence. ROM/all 56 owners
audit: body already normal, two null 4B cells 37C760/37C764 not owned, 8B/
SHA af5570f5.., no new sequence/pixel. 1025 guards/6 input preimages intact.
A new stable static contract covers these null cells; another 161 instructions/state
bindings remain open: no universal selector 16→sprite 46, model/resource or
scripts×all-sprites product. No new fixture/navigation/checkpoint/payload edit/full
pair or repeated old acceptance. e61/USA e8 unchanged; Linux-USA-native Windows-32:9
to do, no performance/space measurement.
[Actual queries and limits](../rom_data_unit_obj.md#source-linked-script-query-observation).


**ROMDATA-01-SCRIPT-OBJ-SPECIAL-NORMAL-PAIRS completed as a conditional EUR source
contract (2026-10-25).** CBD24 compares unit+4 with the selector; CBAD4 nonnull
unit+4>1 and CF144 bind its actual logical metadata14 ID to normal halfword 04;
9B824 assigns it to actor+34 before override 42. Explicit conditions selected before
the audit: actor retains the same unit's normal assignment, override 20 clear,
facing 0..3. Alternate/42-230/high16/different actor state remain separate, no
groups×all sprites product or removed row. All 151 low instructions → 20 selector/
base pairs 14 logical IDs → 18 sprite/base pairs 12 sprites/108 queries 108 cells
76 null/32 sequences 241 frames/11 OBJ 64 objects/95 pixel requests 35968B.
56-owner audit: 4780 initial + 30586 normal shared and zero other overlap/conflicts/
mixed items/variant extensions. New 33 spans 6436B/SHA 6c3ef9c9.. = [240 index
28 header 920 frame 5248 pixel], 60 cells 53 null/7 sequences 46 frames (all op 1)/
17 pixel requests. Observed null cells 37C760/64 included in stable ROM plan, no
save-dependent dataset. 27 halfword/1 literal/3 BL pins pass, no build/guest test.
Contract 01 closed; next 02 lossless model with 151 refs/20 witnesses/108 keys and
only new Wine oracle 60 cells/7 resets/17 uploads, then 03/04/05. Old acceptances/
6000 census not repeated; e61/USA e8 unchanged, Linux-USA-native Windows-32:9 to do,
no performance/space/fixture/navigation/checkpoint.
[Contract and limits](../rom_data_unit_obj.md#conditional-normal-script-poses-for-special-unit-selectors).


**ROMDATA-02-SCRIPT-OBJ-SPECIAL-NORMAL completed in targeted EUR/Wine scope
(2026-10-25).** Lossless model 20 logical metadata14 witnesses/151 script refs/
108 keys with shared physical formats; validates initial and normal independently
even on overlapping ranges, subtracting their union. Full 41802B/SHA f64755ef..
and new 33 spans 6436B/SHA 6c3ef9c9.. match the independent plan. Single warning-free
Windows frontend build/frozen 2eaa5653..; single new Wine filter first PASS 18 guards/
120 prefixes/53 null resets 7 proper/17 scheduler-DMA3/all 5248 new pixel B/whole
VRAM. New frames all op 1, no replay of already tested commands 2/6/8; old acceptances/
6000 census closed. Only module registrations and new files, e61/USA e8 unchanged.
Managed/runtime/actual CLI/ordinary and alternate/42-230/high16/actor-state conditions
remain; new Linux/USA/native Windows/32:9 to do. No performance/space measurement.
After memory closure, ordinary equipment derivation was subsequently completed with
explicit authorization and a single positive Fight 500 pair; 03 script-special
pipeline continues below.
[Model and evidence](../rom_data_unit_obj.md#typed-normal-script-special-pose-model).


**ROMDATA-03/04-SCRIPT-OBJ-SPECIAL-NORMAL managed/runtime completed within targeted
EUR/Wine scope (2026-10-26).** e62-d61 publishes 58 files 57 resources with
metadata14/script archive/initial/normal, 151 refs 20 authentic logical witnesses;
all 56 previous records/files/coverage and e61 preserved, USA e8-d7. Immutable owner
serves only 33 spans 6436B typed, shared initial/normal with previous owners, without
ROM shadow/ID cap/task rewrite/ABI-save-native changes. One warning-free Windows
core/frontend build (core compiled only); two new Wine filters first PASS 23 attach
guards + 6 dependencies/lifecycle/all 6436 typed B/7 sequences and 23 original-
interpreter-compiled paths with equal full state-clock. Part 0 proper/part 1-2 null/
two orientations/fullword raw ID/live RAM/two uploads-DMA/all 33 edges/mirror/BIOS/
SWI-DMA pass; new API [0 dir 36 index 6 header 43 frame 0 OBJ 1 pixel] B, exact
mixed 2 new 4 normal and API 2 new 2 altered raw. All 41802 graph B poisoned, old
commands 2-6-8/source/census/scenes not repeated. Dataset f131b439../manifest file
2b84c40c../model file 02e7362e.. with distinct identities. 04 real production CLI
subsequently completed below; 05 ordinary source/new pixels of the family and
alternate/override/high registry/actor-state conditions remain; Linux-USA-native
Windows-new 32:9 to do. No performance/space measurement/new fixture.
[Publication and runtime](../rom_data_unit_obj.md#conditional-script-special-normal-managed-runtime).


**ROMDATA-04-SCRIPT-OBJ-SPECIAL-NORMAL-CLI completed EUR/Wine (2026-10-26).**
Single warning-free Windows production build/frozen 6455958e..; canonical exactly
restored (hash+mtime), main content unchanged/mtime-only Cargo invalidation.
Actual Wine 26817 create/reuse 58 generation 60 managed/all 58 equal accepted/no
rewrite, unrelated cwd, generation-SHz1vX/f131b439... One unchanged F09 frame/old
cache loads owner but zero new bytes: compatibility smoke only, no 05 ordinary/new
pixels/full pair claim. Old 3530/500/census 6000/source/native/CLI/scene evidence
not repeated. 05 source-use/new sequence/pixel ordinary still open: Snow 6000 has
two actual new null cells, a lead that does not prove new pixels. Linux-USA-native
Windows-new 32:9/other source cases remain to do; no measurement or new fixture/
navigation.
[CLI and limits](../rom_data_unit_obj.md#conditional-script-special-normal-actual-production-cli).


**ROMDATA-05-SCRIPT-SPECIAL ordinary: positive null locator, pixels still open
(2026-10-26).** Actual e62 Wine Snow 6000 unchanged reads 48 new index B and 0 new
sequences/pixels; it is real source use, not a full pair or pixel evidence. No
census/guard build/full pair repeated. New static read of 81 EUR snapshots for
seven new sequences finds no compatible tasks. RIVER Start/A618 already recorded
on the new consumer reads 0 new bytes: terminal negative, do not repeat. Previously
active context is program 158/key 168; 159/key 169 contains BAD829→2/group 34 but
the trigger is unproven, end/start of battle interpretations remain provisional.
A short B44 from authorized DS20 EUR copy shows only Polly menu/two private
checkpoints, not victory/new pixels/new DS20 acceptance. Originals/cart unchanged.
05 new sequences/pixels/equivalence remain open; next independent criterion can be
script-special alternate source contract, keeping the gate open. New Linux-USA-
native Windows-32:9 to do, no measurement or production code.
[Locator and limits](../rom_data_unit_obj.md#script-special-ordinary-source-locators-and-remaining-pixel-gate).


**ROMDATA-01-SCRIPT-OBJ-SPECIAL-ALTERNATE-PAIRS completed as a conditional EUR
source contract (2026-10-26).** Same 151 low 49 references/20 logical witnesses,
metadata14 half 06 under original current alternate assignment/override 20 clear/
facing 0..3. Four sprites 188/231/232/234, 14 sprite-base pairs, 84 cells 68 null,
14 sequences 64 frames/four OBJ lists 20 objects/26 pixel requests 7232B. All 57
owners e62 authenticated, no shared-root target/conflict/mixed item/variable extents.
New 20 spans 816B = 216 index 8 header 80 frame 512 pixel, 54 cells 52 null/two
sequences/four op 1 frames/two pixel requests; SHA 029ff8a0... **ROMDATA-02-SCRIPT-OBJ-SPECIAL-ALTERNATE completed in targeted EUR/Wine scope:** typed model,
warning-free Windows build and single new Wine filter pass 19 guards/108 original
prefixes/52 null + 2 proper resets/two DMA3 uploads/full VRAM 512 new pixel B.
First Wine start fails before test due to sandbox lock server; same binary restarted
with necessary access, first actual filter PASS/no rebuild. Already covered commands
2/8 not repeated. Managed/runtime/actual CLI/ordinary and high16/override/current
state remain open; e62/USA e8 unchanged, Linux/native Windows/new 32:9 to do, no
performance/space measurement.
[Contract and model](../rom_data_unit_obj.md#conditional-alternate-script-special-source-and-typed-model).


**ROMDATA-03/04-SCRIPT-OBJ-SPECIAL-ALTERNATE managed/runtime completed within
targeted EUR/Wine scope (2026-10-26).** e63-d62 publishes 59 files 58 resources
with actual metadata14/script archive/initial/normal/alternate, all 57 previous
data files/resource records/coverage unchanged/e62 retained/USA e8-d7 unchanged.
Immutable owner serves only 20 new spans 816B from typed fields; old owners and raw
holes compose each read. Warning-free Windows core/frontend build, core compiled
only; two new Wine filters first PASS 24 attach + 7 dependency checks/lifecycle/all
816 typed B/two sequences and 24 original-interpreter-compiled paths with full state/
clock, part 0 proper/parts 1-2 null/facing/fullword raw ID/live RAM frame/uploads-
DMA/all 20 edges/mirrors/BIOS-SWI-DMA/mixed 2 new 4 alternate + 2 new 2 raw. All
9048B graph altered, including shared owners; previous source/model/54 cells/source
DMA/commands 2-8 evidence not repeated. Actual production CLI/ordinary new pixels/
equivalence and high16/42-230/current actor remain open; new Linux/USA/native
Windows/new 32:9 to do, no performance/space measurement/new fixture.
[Publication and runtime](../rom_data_unit_obj.md#conditional-script-special-alternate-managed-runtime).


**ROMDATA-04-SCRIPT-OBJ-SPECIAL-ALTERNATE-CLI completed EUR/Wine (2026-10-26).**
Single warning-free Windows production build/frozen 36cba67d.., canonical restored
with exact hash+mtime/main content unchanged. Real Wine CLI creates/reuses 59
generation/61 managed files, all 59 byte-identical to the accepted dataset, no hash/
mtime rewritten from unrelated cwd. One authentic F09 frame/old cache load passes
with zero new uses: compatibility only. Source/model/managed/native/actual CLI closed/
no repeat. Ordinary new pixels/full scene, high16/42-230/current actor and new
Linux/USA/native Windows/32:9 to do; no performance/space measurement/new fixture.
[CLI and limits](../rom_data_unit_obj.md#conditional-script-special-alternate-actual-production-cli).


**ROMDATA-05-SCRIPT-SPECIAL-ALTERNATE-TWO-SEQUENCE-LEAD closed negative
(2026-10-26):** 81 authenticated EUR snapshots, zero enabled/dormant tasks for the
two new sequences AD4950/AD4980/18832-33, original hash+mtime unchanged. Targets
differ from old 606 alternate/seven normal scans, not repeated. No Wine game/build/
navigation/new checkpoint/full pair selected; do not retry unchanged. Ordinary new
sequence/pixel/equivalence remain open; next independent source contract override
42/230 or high registry with concrete original witnesses.
[Lead and limits](../rom_data_unit_obj.md#conditional-script-special-alternate-actual-production-cli).


**ROMDATA-01-SCRIPT-OBJ-OVERRIDE-PAIRS completed as a conditional EUR source
contract (2026-10-26).** Original 9B734 flag 20 forces 42 normal/230 alternate;
only these two IDs and 13 actual groups from 167 script 49 refs: 26 mode-base
witnesses/334 conditional references/156 cells 126 null/28 sequences 140 frames/
8 OBJ lists 27 objects/42 pixel requests 8000B. Original current assignment/flag 20
set/facing 0..3 conditions before targets, no static low/high registry identity or
all-sprite product. All 58 owners and 19 halfword/two BL/five code fingerprints
pass, zero conflicts/mixed items/shared roots/varying extents. New 40 spans 1050B =
384 index 24 header 360 frame 26 OBJ 256 pixel; 96 cells 90 null/six sequences
18 frames (16 op 1/two op 2)/one list 4 objects/two pixel requests.
**ROMDATA-02-SCRIPT-OBJ-OVERRIDE completed in targeted EUR/Wine scope:** offline
model, warning-free Windows build and single new Wine filter first PASS 19 guards/
all 156 keys/192 constructor aliases/90 null + 6 proper resets/two new op 2
argument copies/two actual DMA uploads 256B/full VRAM. e63/USA e8/runtime unchanged.
03 managed/runtime/CLI/ordinary and high registry/current actor/new Linux-USA-native
Windows-32:9 open, no performance measurement.
[Contract and model](../rom_data_unit_obj.md#conditional-forced42-and230script-pose-source-model).


**ROMDATA-03/04-SCRIPT-OBJ-OVERRIDE managed/runtime completed within targeted
EUR/Wine scope (2026-10-26).** e64-d63 adds unit-obj-script-override.json /
graphics.unit-obj-script-override with actual script archive/normal battle/alternate
battle: 60 files 59 resources/all 58 previous data-record-coverage/e63 retained/
USA e8-d7. New immutable owner only 40 spans 1050B including 26 OBJ and two new
op 2 argument pairs; original selector/flag 20/task/command/DMA/VRAM/save/native ABI
active. Single warning-free Windows core/frontend build/core compiled only; two new
Wine filters first PASS 26 attach + 5 dependency guards/lifecycle/all 1050 typed B/
six seq and 27 original-interpreter-compiled paths full state/clock/both new op 2/
new OBJ/null/proper/live RAM/two uploads + DMA/40 edges/mirrors/BIOS-SWI-DMA/mixed
2 new 4 alternate + 2 new 2 raw. All 11722 graph B altered; source/model/192 prefixes/
96 cells/source DMA/old evidence not repeated. Real CLI/ordinary new pixel/
equivalence and high registry/current actor/new Linux-USA-native Windows-32:9 open,
no measurement or new fixture.
[Publication and runtime](../rom_data_unit_obj.md#forced-script-override-managed-runtime).


**ROMDATA-04-SCRIPT-OBJ-OVERRIDE-CLI completed EUR/Wine (2026-10-26).** Single
warning-free Windows production build/frozen 8a987e99.., canonical hash+mtime
restored/main content unchanged. Real Wine CLI create/reuse 60 generation 62 managed/
all 60 byte-equal accepted/no-write from unrelated cwd/old cache passes. One
authentic F09 frame reads zero new bytes, compatibility smoke only. Source/model/
managed/native/CLI closed/no repeat. Ordinary new sequence/pixel/equivalence and
high registry-current actor/new Linux-USA-native Windows-32:9 open, no measurement
or new fixture/navigation/payload.
[CLI and limits](../rom_data_unit_obj.md#forced-script-override-actual-production-cli).


**ROMDATA-05-SCRIPT-OVERRIDE-SIX-SEQUENCE-LEAD closed negative (2026-10-26).**
81 authenticated EUR snapshots/zero enabled or dormant tasks for 42/(32,33,40,41)
and 230/(32,33), all part 0; input hash+mtime unchanged. Targets differ from old
606 alternate/seven normal/two alternate, not repeated. No guest/build/navigation/
checkpoint/payload edit/full pair, do not retry unchanged. Ordinary new sequence/
pixel/equivalence remain open while independent high registry/current actor source
work continues.
[Lead and limits](../rom_data_unit_obj.md#forced-script-override-actual-production-cli).


**ROMDATA-01-SCRIPT-REGISTRY-SPECIAL-PAIRS completed as a conditional EUR source
contract (2026-10-26).** 16 high 49 refs/seven exact selector-base pairs, unit
byte 4 in the 93 known metadata14 rows/same-unit current CF144-CF180 assignment/
override 20 clear/facing 0..3 selected before targets. 1302 witnesses 2976 refs/
74 sprites 370 sprite-base/2220 logical queries 1908 cells 1537 null/353 sequences
2081 frames/60 OBJ lists 279 objects/841 pixel requests 321088B. All 59 owners and
41 halfword/four literal/eight BL pins pass, no known index root/conflict/mixed item/
varying extent or query pruning. New 272 spans 15276B = 2392 index 84 header 1920
frame 10880 pixel: 598 cells 567 null/21 seq 96 op 1 frames/37 pixel requests, zero
new OBJ. Source only closed; model/Wine closed as below. Managed-runtime-CLI-ordinary
to do, e64/USA e8 unchanged. No invented static registry/generic job/redirect/out
of catalog/current actor identity, no measurement or guest/fixture/navigation.
[Contract and limits](../rom_data_unit_obj.md#registry-selectors-and-known-special-unit-source-projection).


**ROMDATA-02-SCRIPT-REGISTRY-SPECIAL model/oracle completed EUR (2026-10-26).**
Lossless model retains 1302 witnesses 2976 refs/2220 logical keys 1908 physical
cells/353 sequences 2081 frames and all 272 new spans 15276B. Six previous views
validated independently before union subtraction; identical physical aliases
coalesced, conflicts rejected. Initial frontend build E0308 test-only corrected
with lookup in the initial type; warning-free retry. First Wine filter passes
26 guards/1196 constructor aliases/567 null + 21 proper resets/37 scheduler-DMA
uploads 10880B/full VRAM. No old command/source/managed/native/scene replay.
03 managed/runtime, real CLI, ordinary new pixels/full scene and generic job/out
of catalog/redirect/changed actor/global registry reachability remain open; Linux/
USA/native Windows/new 32:9 to verify. No performance/space measurement or new
fixture/navigation/payload edit.
[Model and limits](../rom_data_unit_obj.md#typed-registry-knownspecial-script-pose-model).


**ROMDATA-03-SCRIPT-REGISTRY-SPECIAL managed/runtime completed EUR (2026-10-26).**
e65-d64 publishes 61 files 60 resources, 59 previous/e64 preserved; metadata14/
archive and six previous views are real dependencies. Typed owner only 272 new
spans 15276B, shared bytes/holes/raw/original tasks-DMA preserved. One warning-free
Windows core/frontend build and first two targeted Wine filters pass 33 attach +
10 dependency guards/lifecycle/15276 typed B 21 seq and 25 original-interpreter-
compiled full state-clock paths with 373842B raw graph altered. 272 edges/width-
mirrors/real BIOS-bus/SWI-DMA/live RAM and mixed 2 new 4 alternate + 2 new 2 edited
raw pass; shared 4B for each of 6 owners, no old command/source/previous acceptance
replay. Real CLI/ordinary new pixels/full scene remain separate, no whole actor/
allocator ABI. Generic job/out of catalog/redirect/changed actor/global registry
reachability and Linux/USA/native Windows/new 32:9 open; no performance/space
measurement or fixture/navigation/payload edit.
[Consumption and limits](../rom_data_unit_obj.md#managed-registry-knownspecial-poses-and-typed-guest-reads).


**ROMDATA-04-SCRIPT-REGISTRY-SPECIAL-ACTUAL-CLI completed EUR (2026-10-26).**
Production build once warning-free/frozen 661c5af7..; canonical restored with
exact hash+mtime/main content unchanged-mtime-only Cargo invalidation. First Wine
CLI prepare/reuse from different cwd: 61 generation 63 managed/60 resources/eight
deps/all 61 files equal accepted/reuse hash-mtime unchanged. One authentic F09
frame old cache attach passes, new consumption 0/smoke only; ordinary new sequence/
pixel/full scene equivalence open. No old scene/source-runtime evidence/fixture/
navigation repeated, no performance/space measurement. Linux/USA/native Windows/
new 32:9 and global registry/generic job/out of catalog/redirect/changed actor
remain to verify.
[CLI and limits](../rom_data_unit_obj.md#registry-knownspecial-actual-production-cli).


**ROMDATA-05-SCRIPT-REGISTRY-SPECIAL-21-SEQUENCE-LEAD closed negative (2026-10-26).**
New readonly scan 81 authenticated snapshots/21 new sequences (new bindings all
part 0) finds 0 active or dormant original tasks; header/payload/ROM and hash+mtime
of all 81 inputs before/after pass. Old 606 alternate/seven normal/two alternate/
six override scans not repeated. No guest/build/fixture/navigation/payload edit/
full pair; lead closed/no retry unchanged. Ordinary new sequence/pixel/full scene
remain open while independent high registry generic-current-assignment source work
continues.
[Lead and limits](../rom_data_unit_obj.md#registry-knownspecial-actual-production-cli).


**ROMDATA-01-SCRIPT-REGISTRY-GENERIC-DIRECT source contract completed EUR (2026-10-26).**
16 high refs/seven pairs, unit+4=0/1/same-unit current assignment/override 20 clear/
facing 0..3, job unit+5/alternate+7/property 4-5/direct route 0 authenticated before
targets with 83 halfword/6 literal-dispatch/11 BL pins. All 116 rows retained:
100 both concrete/10 normal only/2 both FFFF/4 route FF; 110 normal 100 alternate
concrete → 2940 witnesses 6720 refs. Initial 110 joint assumption corrected before
reading targets; 2 normal 12 alternate FFFF zero-extend u16 without invented fallback.
All 60 e65 owners authenticated before root audit. 107 sprites 535 pairs/3210 keys
3210 cells 2420 null/777 proper seq 5093 frames/45 OBJ lists 231 objects/1895 pixel
requests 677280B. 12 targets are initial indices 48..59 with exact directory/
first-cell witnesses, no count heuristic. Full 796992B/shared 697168B/new 659 spans
99824B = 5544 index 504 header 11760 frame 82016 pixel: 1386 cells 1260 null/126 new
seq 588 op 1/268 new pixel, no new OBJ/mixed item/role conflict/variable extent.
Model/Wine/managed-runtime/CLI/ordinary to do; e65/USA e8 unchanged. Redirect FFFF
out of catalog/current changed/registry identity and Linux/USA/native Windows/new
32:9 separate. No guest/build/fixture/navigation or performance/space measurement.
[Contract and limits](../rom_data_unit_obj.md#registry-selectors-and-generic-direct-job-source-projection).


**ROMDATA-02-SCRIPT-REGISTRY-GENERIC model/oracle completed EUR (2026-10-26).**
Lossless model retains 2940 witnesses 6720 refs/107 roots 3210 keys/777 seq 5093
frames, 12 explicit initial root targets 48..59 and all 659 new spans 99824B. Six
previous views validated independently before union subtraction; metadata52/archive
are real dependencies, 116 rows/sentinel/redirect retained in existing source/metadata.
First Windows frontend build warning-free/frozen bd8727ee..; first Wine filter passes
34 guards/2772 new cell facing aliases/1260 null 126 proper resets/268 scheduler-DMA
uploads 82016B/full VRAM. No old command/source/managed/native/scene replay; no
native complete-state-clock/whole-actor ABI claim. 03 managed/runtime/CLI/ordinary
new pixels and FFFF/redirect/out of catalog/changed actor/global identity open;
Linux/USA/native Windows/new 32:9 to verify. No performance/space measurement or
new fixture/navigation/private payload edit; e65/USA e8 unchanged.
[Model and limits](../rom_data_unit_obj.md#typed-registry-generic-direct-job-model).


**ROMDATA-03-SCRIPT-REGISTRY-GENERIC managed/runtime completed EUR (2026-10-26).**
e66-d65 publishes 62 files 61 resources/all 60 previous-e65 preserved. Metadata52/
archive and six previous views are real dependencies; typed owner only 659 new spans
99824B, 12 initial root views/shared bytes/holes/raw/original tasks-DMA preserved.
One Windows core/frontend build first PASS warning-free and first two Wine filters
pass 36 attach + 10 dependency guards/lifecycle/99824 typed B 126 seq and 25 original-
interpreter-compiled full state-clock paths with 796992B raw graph altered. 659 edges/
width-mirrors/real BIOS-bus/SWI-DMA/live RAM/mixed 2 new 4 alternate + 2 new 2 edited
raw pass; 4 shared B for each of six owners, no old command/source/previous acceptance
replay. Real CLI/ordinary new pixels/full scene, FFFF/redirect/out of catalog/changed
actor/global registry identity remain open; no whole actor/allocator ABI. Linux/USA/
native Windows/new 32:9 to verify, no performance/space measurement or fixture/
navigation/payload edit.
[Consumption and limits](../rom_data_unit_obj.md#managed-registry-generic-direct-job-poses-and-typed-guest-reads).


**ROMDATA-04-SCRIPT-REGISTRY-GENERIC-ACTUAL-CLI completed EUR (2026-10-26).**
First Windows production build warning-free/frozen d5092be6..; canonical restored
with exact hash+mtime/main content unchanged-mtime-only Cargo invalidation. First
Wine CLI prepare/reuse from different cwd: 62 generation 64 managed/61 resources/
eight deps/all 62 files equal accepted/reuse hash-mtime unchanged. One authentic F09
frame old cache attach passes, new consumption 0/smoke only; ordinary new sequence/
pixel/full scene equivalence open. No old source/model/runtime/scene proof replay,
fixture/navigation/private payload edit or performance/space measurement. Linux/USA/
native Windows/new 32:9 and FFFF/redirect/out of catalog/changed actor/global
registry identity remain to verify.
[CLI and limits](../rom_data_unit_obj.md#registry-generic-direct-job-actual-production-cli).


**ROMDATA-05-SCRIPT-REGISTRY-GENERIC-126-SEQUENCE-LEAD closed negative (2026-10-26).**
New readonly scan 81 authenticated snapshots/126 new proper sequences (new bindings
all part 0/body link +44) finds 0 active or dormant original body tasks. Current
pointer inside/at end/outside/misaligned retained without filter/end-of-frame pruning.
Model/source/body link code and header-payload-ROM/hash+mtime of all 81 inputs before/
after pass. Old 21 known special/six override/seven normal/two alternate/606 alternate
scans not repeated. No guest/build/fixture/navigation/private payload/full pair; lead
closed/no retry unchanged. Ordinary new sequence/pixel/full scene remain open,
independent high registry route FF/alternate job and other remaining cases continue.
No performance/space measurement, Linux/USA/native Windows/new 32:9 pending.
[Lead and limits](../rom_data_unit_obj.md#registry-generic-direct-job-actual-production-cli).


**ROMDATA-01-SCRIPT-REGISTRY-REDIRECT-KNOWN-JOB completed as an EUR source alias (2026-10-26).**
The four current jobs 80/81/82/85 route FF redirect to the concrete alternate
job+7 row route 0 for property 4 normal/5 alternate; they do not use the primary
sprite field. Domain selected before targets: 16 high refs/seven pairs/unit kind
0-1/same-unit original assignment/actual mode/override 20 clear/facing 0-3. All
116 rows/928 actual classifications retained: 840 concrete 56 FFFF 32 loop;
110 normal 100 alternate produce 11760 witnesses 26880 refs. 98 halfword 6 literal
11 BL pins pass; r4 remains the alternate job, CC35A returns to the same property.
All 61 current resources/sources authenticated before targets, each of 2940 direct
witnesses has four preimages: same 3210 cells/777 seq/12 initial root views/full
796992B/seven owners, zero new bytes/resources/revisions. Metadata52/e66 Wine proofs
reused, no build/guest/fixture/navigation or performance/space measurement. e66-d65/
USA e8-d7 unchanged; ordinary/global registry identity/FFFF/nonreturning/out of
catalog/changed actor and Linux/USA/native Windows/new 32:9 open.
[Contract and limits](../rom_data_unit_obj.md#registry-generic-jobs-redirected-to-a-known-alternate-job).


**ROMDATA-01-SCRIPT-REGISTRY-FFFF-READ completed as an EUR reader classification (2026-10-26).**
196 direct witnesses + 784 route FF/2240 refs retain FFFF results normal 0/1 and
alternate 0/1/46-50/64-65/69-71; domain selected before targets. 71 reader pins/
three code blocks/two literals verify zero extension/actor u16/fullword directory
without FFFF guard. Property 083B4C12=DC86, directory 083D4830=14AE3F1E: 60 queries/
30 words in unmapped region 14, CPU aligns/rotates 16 and MemMap depends on
livebios_open. No invented ROM sequence/constant null/universal nonreturn, cap/
fallback/pointer fix or extension. These two ROM reads are 6B without an owner:
raw fallback preserved, 6B migration and ordinary reachability still open. e66-d65/
USA e8-d7 unchanged, no build/guest/fixture/navigation/private payload/performance
or space measurement. Linux/USA/native Windows/new 32:9 and global identity/unknown/
changed actor open.
[Classification and limits](../rom_data_unit_obj.md#known-generic-job-ffff-results-and-raw-bus-reads).


**ROMDATA-01-UNIT-OBJ-PROPERTY-WORDS EUR source contract completed (2026-10-26).**
221C4 reads 394C14+2*sprite and returns u16; 9AD0E passes it via r8 to constructor,
allocation and task+16. Domain before targets: 398 normal/alternate witnesses with
concrete metadata52/14 and override 42/230, 182 sprites/queries; concrete redirects
already equivalent, no inferred equipment/body/global identity. 98 halfword 15 BL/
four code blocks/one literal and all 61 owners/files/sources pass before the words.
Four unowned ranges 364B, SHA ad328c8e.., 139 words 16/43 words 24; raw u16/holes
preserved, no invented extension 248/ID cap/flag bitfield or VRAM unit. Source
closed, model/Wine oracle/managed-runtime-CLI-ordinary open; e66-d65/USA e8-d7
unchanged. FFFF raw 6B/unknown/changed actor/global identity and new Linux-USA-
native Windows-new 32:9 separate. No build/guest/fixture/navigation/measurements.
[Contract and limits](../rom_data_unit_obj.md#actor-property-words-from-known-sprite-assignments).


**ROMDATA-02-UNIT-OBJ-PROPERTY-WORDS model/oracle completed EUR/Wine (2026-10-26).**
Lossless model 398 witnesses/182 words/4 ranges 364B, actual metadata52/14
dependencies, raw u16/holes/identity preserved without 16/24 cap. First warning-free
Windows frontend build and first exact Wine run pass 20 guards/all 182 getters 364B/
3 raw fullword getters, four actor→new task prefixes (two original/two synthetic
highword), two reuse prefixes and four original u16 setters. New tasks receive the
property, existing ones retain their own +16; all RAM outside scratch/destinations
and VRAM/OAM/palette pass. 82 build 83 Wine hash-mtime guards/11 compiled inputs
unchanged; no old oracle repeated/full allocator-constructor/native-clock/ordinary
claim. e66-d65/USA e8-d7/production unchanged, managed-runtime-CLI-ordinary still
open. No fixture/navigation/private payload/performance/space measurement; Linux/USA/
native Windows/new 32:9 and FFFF raw 6B/unknown/changed actor/global registry separate.
[Model and limits](../rom_data_unit_obj.md#typed-actor-property-words-and-original-reader-validation).


**ROMDATA-03-UNIT-OBJ-PROPERTY-WORDS managed/runtime completed EUR (2026-10-26).**
e67-d66 publishes 63 files 62 resources with actual metadata52/14 dependencies and
all 61 previous files/records-e66 preserved. Immutable owner only 4 spans 364B/
182 words, 398 witnesses, 3 u64 masks per record; old owners/holes 112-117/FFFF
raw 6B and guest work preserved. One warning-free Windows core/frontend build, first
two Wine filters pass 17 attach + 4 full-set dependency guards/lifecycle/all 364
typed B 182 mask records and 16 original-interpreter-compiled full state-clock paths
with 364 raw B altered. Four edges/width-mirrors/real BIOS bus/raw fullword/live RAM/
SWI-DMA pass; API 2 new 2 old/2 new 2 raw, CpuSet 4 new 2 old/FastSet 28 new 4 old/
DMA 4 new 2 old exact. 84 build 86 Wine hash-mtime guards/20 compiled inputs unchanged,
no old source/model/family proof repeated or full allocator/whole actor/ordinary
claim. Production CLI/positive ordinary, raw 6B/unknown-changed actor/global identity
and Linux/USA/native Windows/new 32:9 open. USA e8-d7 unchanged, no gameplay fixture/
navigation/private payload or performance/space measurement.
[Consumption and limits](../rom_data_unit_obj.md#managed-actor-property-words-and-typed-runtime-reads).


**ROMDATA-04-UNIT-OBJ-PROPERTY-WORDS-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen 2ce56e03.., canonical restored
with exact hash+mtime/main content unchanged-mtime-only Cargo invalidation. First
Wine CLI prepare/reuse from different cwd: 63 generation 65 managed/62 resources/
two deps metadata52-14/all 63 accepted/reuse hash-mtime unchanged. One frame from
immutable authentic F09 loads old native cache and attaches the resource, new bus/
source/word counters all 0: smoke only, positive ordinary/full scene equivalence
still open. 154 active CLI guards/20 compiled inputs unchanged, no repeated accepted
source/model/runtime/scene, gameplay fixture/navigation/private payload or performance/
space measurement. USA e8-d7, Linux/USA/native Windows/new 32:9 and raw 6B/unknown-
changed actor/global identity separate.
[CLI and limits](../rom_data_unit_obj.md#actor-property-words-in-the-actual-production-cli).


**ROMDATA-05-UNIT-OBJ-PROPERTY-WORDS-ORDINARY completed within declared EUR/Wine scope (2026-10-26).**
One new ordinary CLI F09 Wait 3530 on already authenticated inputs/checkpoint/cache
uses 10 reads/20 aggregate B/two words from the new owner. Only an optional positive
guard/report added to the generic validator; first warning-free Windows frontend
build/frozen a3e255e6... First full Wine pair with only the new flag passes every
frame state-clock/central-wide buffer/OAM/dispatch/mixed PSG-FIFO-HLE audio; exact
10/20/2 counts as production CLI 2ce56e03../generation-gpXTBA/e67. 155 active
hash-mtime guards/21 compiled inputs unchanged, other 180 words retain targeted
source-reader coverage. Old normal-battle 3530 child remains closed: same inputs for
new property-word criterion, no source/model/runtime/old-family/cost proof repeated
or gameplay fixture/navigation/private payload/performance-space measurement. Declared
scene closed, no whole allocator/actor ABI/all-word ordinary/new 32:9 claim; Linux/
USA/native Windows, raw 6B/unknown-changed actor/global identity and other families
remain open.
[Scene and limits](../rom_data_unit_obj.md#ordinary-actor-propertyword-use-and-scene-equivalence).


**ROMDATA-01-UNIT-OBJ-PROPERTY-GETTER-CALLERS completed as an EUR source contract (2026-10-26).**
The five already inventoried BL calls are authenticated; four missing paths
reconstructed with 680 halfwords, 25 BL and 21 literal/control-flow pins. The two
u16 callers use 233A8/CF144 or 9B734 and update the property with 22DF4 while
retaining the allocation; known concrete assignments reuse the existing 182 words.
The two UI callers load context+1085 with LDRB and compare the result with 16 to
select a descriptor. Builder, truncations and tag/animation conditions verified.
The full window of 256 byte arguments is selected before targets, without inferring
a 256 catalog, a getter cap or universal ordinary reachability. All 62 owners/files/
sources authenticated: 184 alias queries (182 properties + two words in the four
palette bytes) and 72 new words/144B in four disjoint ranges, SHA 9d1239cc... Both
shared views reconstructed; no query discarded or duplicated. Source child closed;
lossless 72-word model and Wine oracle closed in the following entry. Runtime/CLI/
ordinary remain to do. No build/guest/fixture/navigation/measurement; e67-d66/
USA e8-d7 unchanged. FFFF raw 6B, unknown u16/changed actor/global identity/computed
indirect and Linux/USA/native Windows/new 32:9 remain separate and open.
[Callers and limits](../rom_data_unit_obj.md#additional-property-getter-callers-and-the-ui-byte-input-window).


**ROMDATA-02-UNIT-OBJ-BYTE-PROPERTY-WORDS model/oracle completed EUR/Wine (2026-10-26).**
New lossless model of only 72 words/144B; input byte/source/u16 identity, four
ranges and strict JSON. The two property words/color palettes dependencies provide
184 words without duplicate copies: exact coverage of the 256-value byte input window. One
warning-free Windows frontend build and one new Wine filter on first attempt pass
14 guards, 512 original UI prefixes/all 512B read, eight exclusions without getter,
four synthetic u16 comparisons, four low-byte writers and six prefixes of the two
additional setters. All RAM outside scratch/destinations and VRAM/OAM/palette checked.
109 build 110 Wine guards/13 compiled inputs unchanged, model SHA ac6a296a.. . No
old test repeated or full UI/allocator/native-clock/ordinary claim. Source/model
closed; new managed-runtime/CLI/ordinary remain pending, e67-d66/USA e8-d7 unchanged,
Linux/USA/native Windows/new 32:9 to verify. No fixture/navigation/private payload
edit/performance-space measurement.
[Model and limits](../rom_data_unit_obj.md#typed-ui-byte-property-words-and-original-caller-validation).


**ROMDATA-03-UNIT-OBJ-BYTE-PROPERTY-WORDS managed/runtime completed EUR (2026-10-26).**
e68-d67 publishes 64 files/63 resources, retains all 62 previous records/63 files
and e67; USA e8-d7 unchanged. Actual property words/color palettes dependencies
retain 184 shared words; new owner only 72 words/144B/four ranges, bitmap per record.
One warning-free Windows core/frontend build; two new Wine filters on first run pass
17 attach guards/four dependency guards, create/reuse/repair/move/interrupted
publication, failed-attach owner+full state isolation and lifecycle with unavailable
directory. All 144B altered raw return typed bytes, all 72 records used. Original/
interpreter/compiled: equal full state and clock on getter, UI prefix, fullword/live
RAM/mixed reads. Four edges, width/mirrors/real BIOS bus/SWI/DMA verified. API 2 new
2 property/2 new 2 palette/2 new 2 raw; CpuSet 2 new 4 property/FastSet 28 new
4 property/DMA 4 new 2 palette. Source/model/old proof closed without replay; new
production CLI and ordinary still to do. Linux/USA/native Windows/new 32:9 and full
UI/allocator/actor ABI not claimed; no fixture/navigation/private payload/performance-
space measurement/Android/package/commit.
[Runtime and limits](../rom_data_unit_obj.md#managed-ui-byte-property-words-and-typed-runtime-reads).


**ROMDATA-04-UNIT-OBJ-BYTE-PROPERTY-WORDS-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen 0c0f5082..; canonical restored
with exact hash+mtime, main content unchanged/mtime only to invalidate Cargo. First
Wine CLI prepare/reuse from different cwd pass 64 generation 66 managed files/
63 resources/two real dependencies/all 64 equal to dataset 03/reuse without writes.
One authentic F09 frame loads old native cache and uses 1 read/2 aggregate B/1 new
word: positive attachment check, full ordinary equivalence still open. 182 active
hash-mtime guards/23 compiled inputs unchanged; no old oracle/scene child repeated,
fixture/navigation/private payload/performance-space measurement. e68-d67/USA e8-d7,
Linux/USA/native Windows/new 32:9 and other families separate.
[CLI and limits](../rom_data_unit_obj.md#ui-byte-property-words-in-the-actual-production-cli).


**ROMDATA-05-UNIT-OBJ-BYTE-PROPERTY-WORDS-ORDINARY completed within declared EUR/Wine scope (2026-10-26).**
F09 Wait 3530 reuses authentic checkpoint/inputs/cache for new owner e68; old normal/
e67 property scene children remain closed. CLI one frame already positive: no second
CLI 3530 locator. Only five optional guard/report lines added to generic validator;
first warning-free Windows build and first full Wine pair pass every frame full
state/clock/central-wide buffer/OAM/dispatch/mixed PSG-FIFO-HLE audio, all five
streams with actual samples. New use 476 reads/952 aggregate B/4 words; other
68 words have targeted reader evidence. 183 active scene guards/24 compiled inputs
unchanged. No source/model/runtime/CLI/old scene/cost proof repeated or fixture/
navigation/private payload/measurement. Declared scene closed; full UI/allocator/
actor ABI/all-word ordinary/new 32:9 and Linux/USA/native Windows, unknown/changed
actor/global identity/computed indirect/raw 6B and other families remain open.
[Scene and limits](../rom_data_unit_obj.md#ordinary-ui-byte-property-use-and-scene-equivalence).


**ROMDATA-01-UI-PROPERTY-OBJ-LIST-SOURCES EUR source contract completed (2026-10-26).**
Reused already recorded CFG candidates and completed getter/UI evidence. Four
u16==16 property conditional branches lead A4/B2→2AE26/1F6C→1500; 48 halfword/
four literal/four BL verify control, pointer and stride 6. All 63 owners/files/
sources authenticated before targets: A4 has 2 objects and produces 14B new
and disjoint, SHA 1f9cfc55..; B2 has 2/14B and matches the already owned model,
no duplication. Packed bits/coordinates reconstructed. Two A4 references and
one B2 alias add three records: projection 126 refs/74 lists 175 objects 1198B,
production still 123 refs/73 lists/e68-d67. Source child closed; model extension
and publication/runtime must be coupled with revision change because Catalog feeds
Extraction directly. CLI/ordinary A4 still to do, no new family needed. No broad
CFG/old oracle/build/guest/fixture/navigation/measurement. Linux/USA/native Windows/
new 32:9 and global families/indirect calls remain open.
[Contract](../rom_data_obj.md#property-selected-ui-list-source-extension).


**ROMDATA-02-03-UI-PROPERTY-OBJ-LIST-EXTENSION completed EUR (2026-10-26).**
e69-d68 integrates three references in the same Catalog/List/Object/owner/format,
126 refs/74 lists 175 objects 1198B; new A4 only 14B/two objects, B2 shared. Same
64 files/63 resources and size dependency; all 73 previous records, 62 other files/
records and e68 unchanged. Corrected warning-free Windows build (first error only an
unqualified test constant). First two Wine filters pass: roundtrip/eight guards/
publication-repair-move management/lifecycle; two original-interpreter-compiled
draws full state-clock/709 instructions/4 dispatch/36 compiled B, two host
publications, edges-width-mirror-BIOS and mixed API 2 new 2 B2/DMA-CpuSet 14 new
2 B2/FastSet 4 prior 14 new 14 B2. Runtime handlers unchanged. 77 build 79 Wine
guards/29 compiled inputs exact. Real CLI 04 and ordinary A4 guest-host 05 still to
do; USA e8-d7/Linux/native Windows/new 32:9 pending. No old evidence/scene replay,
fixture/navigation or performance-space measurement.
[Integration and limits](../rom_data_obj.md#managed-ui-property-list-extension-and-typed-consumption).


**ROMDATA-04-UI-PROPERTY-OBJ-LIST-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen 17488009..; canonical restored
hash+mtime, main content unchanged. First Wine prepare/reuse/old-cache one frame
from /tmp pass 64 generation 66 managed/63 resources/74 spans 1198B/all 64 equal to
dataset 03, reuse without writes. 148 active CLI guards/29 compiled inputs exact.
Ordinary A4 scene equivalence still open; no old oracle/scene repeated, fixture/
navigation/private payload or performance-space measurement. Other platforms remain
to do.
[CLI and limits](../rom_data_obj.md#ui-property-lists-in-the-actual-production-cli).


**ROMDATA-05-UI-PROPERTY-OBJ-LIST-LOCATORS: negative observations closed, ordinary open (2026-10-26).**
F09 one frame/Party unit 320 do not use A4. One bank 120 locator on an authorized
exact EUR copy plus normal Down uses B2 guest-host, A4 absent; endpoint inspected,
151 guards/29 compiled inputs exact. No full pair or extended repeat/other slots.
Four snapshots read with checksum and first 7 visible rows → property 24; no global
coverage inferred. New setters 02AE70 (input/animation/tag), 02AF14 (animation),
02AF5C (state only) distinguished. First BL census for two new targets: 4 updaters/
8 setters; prefix 0578EE assigns animation 24 to human preview. Full path and
ordinary A4 guest-host/full scene witness still open; read control before further
input. Old oracles/CFG/scenes closed, no new build/gameplay fixture/measurement;
Linux/USA/native Windows/new 32:9 to do.
[Locator and next criterion](../rom_data_obj.md#ordinary-ui-property-list-locators).


**ROMDATA-05-UI-PROPERTY-OBJ-LIST-HUMAN-ANIMATION completed for the declared EUR scene (2026-10-26).**
Original join 25 halfword/6 switch words: evaluator D2D44→6375C→1092→063230/02AF14
sets 24 for results 1/2, tag 1 + property 16 selects A4. First Wine CLI Dispatch 1840
positive A4 guest-host/15 evaluation B; negative 1400 retained because it stopped
before event 1600, corrected boundary justified by already positive original 1840
evidence. First warning-free Windows test build/frozen 427845f2..; only 5 optional
A4 guard/report lines in validator, product runtime unchanged. First Wine full pair
1840 passes all state-clock/central-wide/OAM/dispatch/all audio and new A4 guest-host
gate; 5 streams with real samples. 152 guards/29 compiled inputs exact; authentic
originals/inputs/private copy, no cart edit or new derived checkpoint. Two private
preflights corrected before Wine, no guest retry. 01/02/03/04/05 declared closed;
old oracles/scenes and negative locators not repeated. Linux/USA/native Windows/new
32:9, full UI/global indirect calls/other families remain open; no performance-space
measurement. [Scene and limits](../rom_data_obj.md#positive-dispatch-use-and-scene-equivalence).


**ROMDATA-02-UNIT-OBJ-FFFF-READER-FIELDS model completed EUR (2026-10-26).**
New lossless model: query 65535, encoded property u16 word + directory u32 word,
two ranges 6B, 980 conditional witnesses/2240 references (196 direct 784 redirect)
with metadata52/archive49 dependencies. Previous valid models and byte aliases
unchanged; no interpreted root/sequence/corrected pointer/cap. First warning-free
Windows build/frozen 83ffcd75.. and first single Wine filter pass 13 guards/3 numeric
roundtrips/980 records/6B and two complete original readers 28 instructions,
registers/RAM guard. 156 build 157 Wine guards/35 compiled inputs exact. 71-pin/
60-query classification and old A4/model/runtime/scene evidence closed without
replay. Publication/owner runtime 03, native clock/ordinary pending; production
e69-d68/64 files 63 resources and USA e8-d7 unchanged. Linux/USA/native Windows/
new 32:9 to do, no fixture/navigation/performance-space measurement.
[Model and limits](../rom_data_unit_obj.md#typed-model-for-known-ffff-reader-fields).


**ROMDATA-03-UNIT-OBJ-FFFF-READER-FIELDS managed/runtime completed EUR (2026-10-26).**
e70-d69: 65 files 64 resources, all 63 previous decoded records/files and e69
preserved; USA e8-d7 unchanged. New numeric resource with real metadata52/script
archive dependencies, model 980 witnesses 2240 refs unchanged. Immutable owner
serves only 2B property + 4B directory from typed fields; gate 3B4/3D4, counters and
two bit fields. Full pointer, live open BIOS/unaligned LDR rotate 16 and guest
control preserved, no invented sequence/target/null/cap. First warning-free Windows
core/frontend build and first two new Wine filters PASS: 20 attach guards, four
dependency guards, publication/lifecycle unavailable directory, all 6B raw altered/
two fields and distinct old owner. Original/interpreter/native 14 paths equal full
state/clock: getter/directory/zero early return/three BIOS prefixes, live RAM/mixed
reads. Two edges/width/mirrors/real BIOS bus; API 2 new 2 raw, CpuSet 2 new 6 raw/
FastSet 4 new 28 raw/DMA 4 new 4 raw preserved. Source/model/A4/old proof closed
without replay. Production CLI 04 and ordinary 05 remain to do, no full sprite/
allocator ABI claimed. Linux/USA/native Windows/new 32:9 pending. No fixture/navigation/
private payload/performance-space/Android/package/commit/push.
[Consumption and limits](../rom_data_unit_obj.md#managed-ffff-reader-fields-and-typed-bus-consumption).


**ROMDATA-04-UNIT-OBJ-FFFF-READER-FIELDS-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen candidate; canonical exact hash+
mtime restored/main content unchanged. First three Wine calls prepare/reuse/old-cache
one frame from unrelated cwd pass 65 generation 67 managed/64 resources/all 65 equal
to dataset 03/two real dependencies/reuse without writes. Original F09 checkpoint and
previous cache intact: attachment proof only, positive ordinary/equivalence not
claimed. Source/model/runtime/old scene closed without replay; USA e8-d7 and Linux/
native Windows/new 32:9 to verify. No fixture/navigation/private payload/performance-
space/Android/package/commit/push.
[CLI and limits](../rom_data_unit_obj.md#actual-production-cli-for-ffff-numeric-fields).


**ROMDATA-05-UNIT-OBJ-FFFF-READER-FIELDS-EXISTING-CONTEXT-LEAD closed, positive ordinary open (2026-10-26).**
New readonly locator on the 81 existing EUR checkpoints: actor+34 FFFF/unit kind
0-1/current job-alternate job predicate from 980 witnesses, both modes/override/
null-dormant-outside retained, without requiring a nonexistent sequence target.
159 potential candidates in 61 snapshots, all kind 0/job 0/alt 0/override 20 clear:
157 null body pointers and two 01480168 outside EWRAM. No body task available in
EWRAM, no registry identity/same assignment/actor validity inferred. This is not
global unreachability evidence and provides no positive scene: ordinary reader/full
pair remain open. Hash/mtime/header/payload/ROM/input/model/closed contract exact.
Source/model/runtime/CLI/126 sequence/old scene closed without replay; no new build/
Wine/gameplay/fixture/navigation/private payload/performance-space. Linux/USA/native
Windows/new 32:9 pending.
[Locator and limits](../rom_data_unit_obj.md#existing-checkpoint-locator-for-ffff-numeric-readers).


**ROMDATA-01-UI-JOB-LOOKUP-SOURCE completed for EUR 20-slot projection (2026-10-26).**
Root 399750 shared by six readers D0538/D0548/D0568/D0588/D05A8/D05C8: job byte
or CBEC8 property 1/4/6/9/0, original truncations and name helper 165E8. Domain
selected before data from 12CA98: index list of 20 RAM slots 02002E78/stride 16,
mode 3 maximum +5/inclusive tie; one occupied slot allows any 0..19. R6 preserved
through CFF70 clear before D05C8. 268 halfword/27 critical pins/9 literal/8 BL
authenticated. No invented global cap/full physical extent/adjacency/game identity;
other raw caller matches are not automatically semantic code. Twenty distinct u8
jobs/20B/one range SHA 969b2cd5.., 20 metadata52 rows reconstructed, disjoint from
all 64 current owners. No sprite target followed. Source child closed; model/Wine
oracle/managed/runtime/CLI/ordinary open. e70-d69/65 files 64 resources and USA 8-7
unchanged; no build/guest/test/fixture/navigation/private payload/performance-space/
Android/package/commit/push. Linux/USA/native Windows/new 32:9 pending.
[Contract and limits](../rom_data_gameplay.md#twenty-slot-job-lookup-source-contract).


**ROMDATA-02-SLOT-JOB-LOOKUP-MODEL model completed EUR (2026-10-26).**
Strict model 20 (slot,u8 job)/20B/source base/hash/byte_at/direct metadata52; Catalog
checks EUR/three producer-reader blocks. Only two module declarations, no Extraction/
runtime registration: e70-d69/65 files 64 resources/USA 8-7 unchanged. First warning-
free Windows frontend build and first single Wine filter PASS: 20 independent entries/
20B/12 JSON-shape-source-dependency guards/roundtrip/disjoint from all 64 owners.
123 original paths: 21 bare job/81 complete property wrappers/21 property 0 prefixes
before 165E8, three high input aliases. All 20B at six original LDRB, callee/full
RAM outside 20B stack/VRAM-OAM-palette preserved. No guest cap 20/null/host fallback,
code parser 268 pins/metadata52/FFFF/A4/old proof not repeated. Managed/runtime 03/
CLI 04/ordinary 05 open; no new fixture/navigation/private payload/performance-space/
Android/package/commit/push. Linux/USA/native Windows/new 32:9 pending.
[Model and limits](../rom_data_gameplay.md#typed-twenty-slot-job-lookup-and-original-reader-proof).


**ROMDATA-03-SLOT-JOB-LOOKUP-RUNTIME managed/runtime completed EUR (2026-10-26).**
e71-d70: 66 files 65 resources/gameplay.slot-job-lookup/slot-job-lookup.json, real
metadata52 dependency; all 64 previous records/files and e70 preserved, USA 8-7
unchanged. Immutable owner 20 u8/20B/page 399/20-record mask, original low 8 input/
slot selection/clear/properties/name/raw outside preserved, no guest cap 20. First
warning-free Windows core/frontend build and two new Wine filters PASS after ENOSPC
stop and retry on temporary filesystem: 18 attach guards/3 dependency guards/
publication/lifecycle unavailable directory/all 20B raw altered/all 20 bits/distinct
FFF owner. Sixteen original/interpreter/native paths equal full state-clock: getter/
properties/name ID prefix/high input/slot 20 raw/live RAM and 12CA98 mode 3 slot 19
→ CFF70 clear → D05C8 before 165E8. Edges/width/mirrors/real BIOS bus; API 2 new
2 raw, CpuSet 20 new 4 raw/FastSet 20 new 12 raw/DMA 20 new 4 raw preserved. Old
64 owners confirm raw neighbors. Source 268 pins/model 123 readers/old proof closed
without replay. CLI 04/ordinary 05 open; Linux/USA/native Windows/new 32:9 pending.
No fixture/navigation/private payload/performance-space/Android/package/commit/push.
[Consumption and limits](../rom_data_gameplay.md#managed-twenty-slot-job-lookup-and-typed-bus-consumption).


**ROMDATA-04-SLOT-JOB-LOOKUP-ACTUAL-CLI completed EUR/Wine (2026-10-26).**
First warning-free Windows production build/frozen candidate; canonical exact hash+
mtime restored/main content unchanged. First three Wine calls prepare/reuse/old-cache
one frame from unrelated cwd pass 66 generation 68 managed/65 resources/all 66 equal
to dataset 03/metadata52 dependency/reuse without writes. Private root on temporary
filesystem via workspace link after ENOSPC 03, no further cleanup/budget formatting/
build retry. F09/original cache intact: attachment proof only, positive ordinary/
equivalence open. Source/model/runtime/old proof closed without replay; Linux/USA/
native Windows/new 32:9 to verify. No fixture/navigation/private payload/performance-
space/Android/package/commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-production-cli-for-the-twenty-slot-job-lookup).


**ROMDATA-05-SLOT-JOB-LOOKUP-BANK-ORDINARY completed for declared EUR scene (2026-10-26).**
First new CLI bank 120 positive 2 reads/2B/1 slot, with existing copied checkpoint/
Down 8..20 input and production e71 dataset generation-GmxCy2. Sole scope new job
lookup, old A4 negative/B2 positive closed. Only 5 optional guard/report lines in scene
validator, runtime/product/old guards/states unchanged. First warning-free Windows
frontend build and first exact Wine filter PASS: 120 full state-clock from 3660 to
3780/central-wide/OAM/native-fallback dispatch and bit-exact mixed PSG-FIFO left-right-
HLE. All 5 sample counts [263340,263340,31680,31680,263264] positive; new use 2B
1 slot matches CLI. Other 19 ordinary slots targeted only. Inspected CLI capture
284x160: Bank background, UI panels absent from raw framebuffer; no final composite
GUI/full name-actor-allocator ABI/new 32:9 claimed. Input/copy/original/cache/dataset
intact; no fixture/navigation/private payload/performance-space/old proof replay.
Source/model/03/04/05 declared closed, full TODO active/Linux-USA-native Windows-
new 32:9 pending.
[Scene and limits](../rom_data_gameplay.md#ordinary-bank-scene-use-of-the-twenty-slot-job-lookup).


**ROMDATA-01-POST-SLOT-LOOKUP-READERS completed for three EUR RAM APIs (2026-10-26).**
D05E8 any nonnull and D0618 count read only the first words of 20 RAM slots 02002E78/
stride 16; they do not validate pointers/liveness, D05E8 returns early. D0650 increments
byte 02001F68 only if old<=98, leaves 99..255 unchanged (no global clamp). All ignore
incoming args for these addresses; six literals are executable RAM base/offset support,
no new ROM asset root. 68 halfword/30 critical pins/6 literals/three complete blocks
verified; 3/2/1 raw BL matches remain candidates/not semantic completeness. No live RAM
export/new decoder-resource-owner/revision/ROM table cap 20. e71-d70/66 files 65
resources/USA 8-7 unchanged. Next partial D0670 excluded. Source child closed,
old evidence/Bank not repeated; no build/guest/test/fixture/navigation/private
payload/performance-space. Linux/USA/native Windows/new 32:9 pending.
[Live state and limits](../rom_data_gameplay.md#live-state-helpers-following-the-slot-job-lookup).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE completed for the EUR Bank12 sample
(2026-10-26).** A test-only observer wraps native read8/16/32 APIs and subtracts the
20756 merged ranges from the 65 e71 source plans. First warning-free Windows build
and one Wine filter: 3166 context reads/3683 image bytes; 198 reads include 676 bytes
outside the plans, 79 ranges observed. Full state/clock, central-wide/OAM/dispatch
and five audio streams match at each frame 3660–3672. The 676 bytes are leads to
authenticate, including five words 370E70/74/80/84/EB0; no new resource/revision
or global completeness. Exported PC may be stale; interpreter/HLE/DMA/native
constants and other scenes outside sample. 73 input hash-mtime exact; main/C API/
old validators unchanged, only new test module with cfg declaration. No new fixture/
navigation/performance measurement. Old Bank 120/source/model/runtime/CLI
acceptances remain closed. Linux/USA/native Windows/new 32:9 to do.
[Contract, transparency and limits](../rom_data_runtime.md#bounded-native-rom-api-residue-inventory).


**ROMDATA-01-NATIVE-RESIDUE-HIGH-WORD-READERS statically completed EUR
(2026-10-26).** The five observed words 370E70/74/80/84/EB0 support execution:
two DMA register address directories and one Thumb IWRAM entry passed to BX r2.
392 halfword/47 pins/5 literal/4 BL in six complete blocks authenticated; actual
loads 0846/09B2/0D56/3A7E/11DA/11EE distinguished from exported PC. Original callers
prove two 4x4B projections 370E68/78 and one word 370EB0, 36B disjoint from 65
owners; 20B observed classified, 656B lower still to classify. No inferred global
cap, RAM body/ABI/scheduler or new asset/owner/revision; explicit ROM/code dependency
and 676B still outside managed plans. No build/guest/test/fixture/navigation/
measurement/old acceptance repeated. e71-d70/USA 8-7 unchanged; Linux/USA/native
Windows/new 32:9 pending.
[Hardware directories and branch control](../rom_data_runtime.md#high-offset-native-residue-is-hardware-and-branch-support).


**ROMDATA-01-NATIVE-RESIDUE-LOWER-READERS completed for EUR static witnesses
(2026-10-26).** 656B observed: 158 literal words/632B with 184 linear Thumb paths
within 96B of saved contexts (507 distinct halfwords), and 6 words/24B in 5 jump
tables with 82 halfwords of bounds/index/load/MOV pc prefix authenticated.
8/35/69/48/5-entry projections; only 0/0/27/4-6/1 observed. This is not a dynamic
instruction/mode trace or proof of all uses. Together with the 20B high hardware/
branch support, all 676B outside plans have static code-support witnesses; they
remain outside the 65 owners/original ROM dependency. No inferred new family/asset
root/owner/revision/complete migration. No build/guest/test/fixture/navigation/
measurement/old acceptance repeated. e71-d70/USA 8-7 unchanged; interpreter/HLE/
DMA/native constants/other scenes and Linux/USA/native Windows/new 32:9 separate.
[Literals and jump tables](../rom_data_runtime.md#lower-native-residue-has-literal-and-switch-table-witnesses).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE-F09 completed for second EUR sample
(2026-10-26).** Same frozen observer executable, no build: one Wine filter 12 frames
8476→8488 from existing F09 checkpoint/inputs preserves state/clock/buffer/OAM/
dispatch and five audio streams. 13671 context reads/13961B image; 604B outside
plans, 584 shared with Bank and 20 new B/5 words. All 5 have linear Thumb PC-relative
witnesses (20 halfwords); static source, not a dynamic trace/mode/all uses. Union of
two samples 696B still outside 65 owners, no asset/revision inferred. 76 input
hash-mtime and compiled code exact, retention helper 0 paths. No fixture/navigation/
measurement or old F09/Bank acceptance repeated. Other interpreter/HLE/DMA/constant
paths and Linux/USA/native Windows/new 32:9 remain separate.
[Second sample](../rom_data_runtime.md#second-native-api-sample-from-the-existing-f09-checkpoint).


**ROMDATA-01-HOST-ROM-READ-RESIDUE completed for collector and EUR F09 sample
(2026-10-26).** Optional non-default Cargo feature rom-data-read-audit: thread-local
address/reader/length collector on resolved bus and selected host audio/sprite/
tables/keyboard; interpreter instruction fetch excluded, execution/clock/API/
savestate unchanged. First build core compiled/frontend failed due to missing c_void
qualification in test; test-only correction and warning-free incremental build. One
Wine filter: ARM/Thumb/width/mirror/audio/lifecycle probe and 12 frames 8476→8488
state-clock-buffer-OAM-dispatch-five audio identical. 23061 requests/19984B image,
1146B outside 65 owners; includes all 604B outside plans from old API report and adds
542B. 90 new high B/6 ranges remain sources to authenticate, 64B also from SpritePeek.
No format/asset/extent/owner/revision inferred. 76 execution inputs + 1 comparison
report hash-mtime exact, no old evidence/fixture/navigation/measurement. e71-d70/
USA 8-7 unchanged; KeyboardWidths inactive/other threads-scenes/host accesses/typed
HLE fast path out of scope; Linux/USA/native Windows/new 32:9 pending.
[Collector, proof and limits](../rom_data_runtime.md#optional-bus-and-host-rom-read-audit).


**ROMDATA-01-BUS-HOST-HIGH-RESIDUE completed for EUR source contracts
(2026-10-26).** 90 new B classified: 64B two indirect OBJ lists 394F4C/26B/4 objects
and 40D89E/38B/6 objects; 10B parameters (2 phase thresholds, 4 signed vertical
words); 16B Thumb IWRAM control pointers. 295 window halfwords/53 pins/7 literals +
ABI hash reused. Original copy 0924E8 proves D968E8..D97728→03005F28..03006D68,
constructor D9752C writes list+158; raw ROM disassembly at empty RAM target rejected.
One existing F09 producer-args Wine frame confirms 2 readers/10 objects, no old
equivalence repeated. 40D89E from task+24 is an indirect observation, not an invented
literal; command creation not reconstructed. Timing compares OLD counter, post-update
phase 0..2 not a global cap; corrected byte>>1 query has 128 possible indices, only
4..7 new here. 64B lists + 10B parameters still to model/publish; 16B control remain
ROM/code dependency; 452B lower separate. e71-d70/65 owners/USA 8-7 unchanged, no
build/fixture/navigation/measurement. Linux/USA/native Windows/new 32:9 pending.
[Indirect lists and evidence](../rom_data_obj.md#observed-indirect-obj-list-sources-beyond-the-direct-catalog),
[parameters and control](../rom_data_runtime.md#high-bus-and-host-residue-source-contracts).


**ROMDATA-02-OBSERVED-OBJ-LISTS-MODEL completed for two EUR lists
(2026-10-26).** Isolated obj_lists::observed API reuses List/Object: 2 lists/
10 objects/64B, identity/count/shape/field width/strict JSON/source consistency
authenticated, constructor-bound root distinct from ordinary-task argument. First
warning-free Windows frontend build and one Wine filter: 64B/2 spans/14 guards/
6 original paths ROM 1500/170C/copied RAM 030065A8, output/matrices/stack/callee/
external RAM/OAM-VRAM-palette exact. Canonical JSON 4530552b..; 75 input-artifact
guards and compiled code unchanged. Only 2 declarations in existing files; old
Catalog 74/175 objects 1198B/e71-d70/66 files 65 resources/USA 8-7 unchanged. No
publication/owner/revision or ordinary evidence to infer; model closed, managed
runtime/CLI/new use from file still open. No old 74/ABI/copier/scene check repeated,
new fixture/navigation/measurement. 10B parameters and 452B lower separate; Linux/
USA/native Windows/new 32:9 pending.
[Model and checks](../rom_data_obj.md#isolated-typed-model-for-the-two-observed-indirect-lists).


**ROMDATA-03-OBSERVED-OBJ-LISTS-RUNTIME completed for two EUR lists
(2026-10-26).** e72-d71 integrates the two indirect identities into the existing
OBJ resource: 76 lists, 185 objects and 1262 source bytes. The 126 direct references,
66 files/65 resources, same size dependency, all 74 previous lists, other 64 files/
records and e71 remain. USA 8-7 unchanged. First warning-free Windows build and two
Wine filters pass publication/repair/move, 12 attach guards and Load/Undo/backup/
restart; six original/interpreter/native paths have equal full state and clock with
all 64 new ROM bytes altered. Edges/widths/mirror/BIOS, six DMA-CpuSet-FastSet
transfers, API reads 2B old + 2B new and 2B altered raw + 2B new, six host publications
pass. 5571 instructions/2418 dispatch are coverage only; 265 typed bytes served by
native APIs. File exactly matches accepted model, 77 input/executable guards and
compiled code unchanged; artifact completion removes only interrupted/moved copies.
Production CLI and new ordinary equivalence remain open; ten parameter bytes and
452 lower bytes separate. No old model/ABI/scene repeated, new fixture/navigation/
measurement. Linux, USA, native Windows and new 32:9 to verify.
[Integration and evidence](../rom_data_obj.md#managed-observed-indirect-lists-and-typed-consumption).


**ROMDATA-04-OBSERVED-OBJ-LISTS-ACTUAL-CLI completed EUR (2026-10-26).**
First warning-free Windows production build; separate CLI c6d674d7.., canonical
executable hash/mtime exactly restored, main content unchanged with mtime-only Cargo
invalidation. First three Wine commands from unrelated cwd pass create/reuse/old-cache
one frame: 66 generation files/68 managed, all 66 equal to runtime dataset, 65 resources/
76 OBJ spans/size dependency. Reuse without hash/mtime changes. Original F09/inputs/
cache at 16:9: both new lists 00394f4c/0040d89e guest=true host=true; OBJ totals
147 reads/147B/14B count/133B attributes/12 host reads. Positive use, full ordinary
equivalence still open. All 143 CLI guards and implementation/canonical/originals
exact; artifact helper zero paths. No old source/model/runtime/producer logging/
scenario repeated, new fixture/navigation or measurements. Linux/USA/native Windows/
new 32:9 to verify; ten parameter bytes and 452 lower separate.
[CLI and evidence](../rom_data_obj.md#actual-cli-for-the-observed-indirect-lists).


**ROMDATA-05-OBSERVED-OBJ-LISTS-ORDINARY completed for the first EUR F09 frame
(2026-10-26).** Only five optional lines in existing validator require both new lists
guest+host. First warning-free Windows build and one Wine filter: frame 8476→8477,
full state/clock/central-wide images/OAM/dispatch and all audio equal. Five streams
with real samples [2196,2196,264,264,2120]; OBJ counters 147 reads/147B/14 count/
133 attributes/12 host exactly match CLI. 6803 native/22849 fallback coverage only;
75 input/executable guards, originals/cache/dataset/canonical exact. Five-line delta
checked against preimage, other implementations unchanged; artifact helper zero paths.
Source/model/managed/runtime/CLI and declared ordinary closed; other scenes/indirect
calls/composed GUI/ABI/new 32:9 open. No old 12/3530/scenario/model/runtime repeated,
new fixture/navigation or measurements. Ten parameter bytes/452 lower and Linux/USA/
native Windows separate or to verify.
[Equivalence and limits](../rom_data_obj.md#ordinary-first-frame-equivalence-for-the-observed-lists).


**ROMDATA-02-OBSERVED-TASK-PARAMETERS-MODEL completed EUR (2026-10-26).**
Isolated observed_task_parameters API for two independent fragments: phase 1/2 u8
thresholds at 371065..67 and four signed16 corrections indices 4..7 at 394E14..1C.
Identity/indices/strict JSON/10B reconstruction/two spans/code-literal-source hash;
no inferred duration/global cap/physical extent. Phase 0/other bytes outside model;
corrections 0..3 retain previous owner, 8..127 remain external. First warning-free
Windows build and one Wine filter: 27 guards and 18 original prefixes (10 old-counter/
phase-wrap, 8 low-bit alias/signed-sub), RAM/callee/OAM-VRAM-palette exact. Canonical
model 4b52c5f1..; 79 input/executable guards and implementation exact. Only two
declarations in old hubs; pipeline/e72-d71/66 files 65 resources/76 OBJ lists/USA 8-7
unchanged. New 10B disjoint from all owners and not yet published; parameter managed/
runtime/CLI/ordinary open. Artifact helper zero paths. No source/295 pins/old model-
ABI-runtime-CLI-scene repeated, new fixture/navigation/measurement. 452 lower separate;
Linux/USA/native Windows/new 32:9 to verify.
[Model and limits](../rom_data_gameplay.md#isolated-model-for-observed-task-parameter-fragments).


**ROMDATA-03-OBSERVED-TASK-PARAMETERS-RUNTIME completed EUR (2026-10-26).**
e73-d72 publishes gameplay.observed-task-parameters/observed-task-parameters.json:
67 files/66 resources, all 65 previous files/records/e72/76 OBJ lists retained. Two
independent fields, no invented dependency, only 10B/two spans 371065..67 and
394E14..1C; pages 371/394 compose previous owners and ROM, without new cap/ABI/clock/
state. First two Wine filters pass 13 attach refusals/3 full-set guards, publication/
repair/move/lifecycle and 24 original-interpreter-native prefixes with equal full
state/clock and all 10B raw altered. Includes phases 0/3/255, corrections 0/16/255,
edges/mirror/BIOS, six DMA-CpuSet-FastSet, API reads 2 new + 2 old with old raw
altered and 2 new + 2 modified raw. 26B native; 316 instructions/24 dispatch coverage
only. Model 4b52c5f1.. exact, 74 guards and compiled code exact. First build core
compiled, frontend omitted two test arguments: only two zeros corrected, warning-free
retry; no Wine filter failed. Artifact helper removes only interrupted/moved. Production
CLI and ordinary parameters remain open; Linux/USA/native Windows/new 32:9 pending,
452 lower separate. No old model/source/ABI/scenario repeated, new fixture/navigation/
measurement.
[Runtime and evidence](../rom_data_gameplay.md#managed-observed-task-parameters-and-typed-bus-consumption).


**ROMDATA-04-OBSERVED-TASK-PARAMETERS-ACTUAL-CLI completed EUR (2026-10-27).**
Warning-free Windows production build/c7f47bd4..; canonical hash/mtime exactly restored,
main content unchanged. First prepare interrupted by user: runner absent, manifest
without current/result; copy/log archived and same binary reused. Three accepted
commands exit 0: create/reuse/old-cache one frame, 67 generation files/69 managed/
66 resources/two spans 10B/no deps/all 67 exact, reuse without writes. First frame
1B threshold/1 phase but 0 corrections: presence gate explicitly negative, retained.
One new runc 12 frames, motivated by that negative and existing source lead, exit 0
with 20 reads/28B/12 thresholds 16 corrections/2 phases 4 rows. 145 guards and
implementation/originals/cache/canonical/user Atlas unchanged; artifact helper zero
paths, interrupted copy retained. CLI closed; ordinary full state 12 frames to verify.
No prepare/reuse/build/old source-model-runtime-scenario repeated, new fixture/
navigation or measurements. Linux/USA/native Windows/new 32:9 and 452 lower separate
or pending.
[CLI and limits](../rom_data_gameplay.md#actual-cli-for-observed-task-parameters).


**ROMDATA-05-OBSERVED-TASK-PARAMETERS-ORDINARY completed EUR (2026-10-27).**
Only five optional lines in existing validator require both groups and all six rows.
First warning-free Windows build and one Wine filter: 12 F09 guest frames 8476→8488,
every full state/clock/central-wide image/OAM/dispatch/audio equal. Five streams with
real samples [26334,26334,3168,3168,26258]; 20 reads/28B/12 thresholds 16 corrections/
2 phases 4 rows exactly match CLI. 88123 native/279744 fallback coverage only.
76 guards, originals/cache/67-file dataset/canonical/code/user Atlas exact. Five-line
delta checked against preimage; artifact helper zero paths. Source/model/managed/
runtime/CLI/declared ordinary closed. First correction-negative frame and interrupted
prepare remain closed; no old source-model-runtime-CLI-collector-scenario repeated,
new fixture/navigation/measurement. Other scenes, indirect calls/ABI/composed GUI/
new 32:9 open; 452 lower and Linux/USA/native Windows separate or pending.
[Equivalence and limits](../rom_data_gameplay.md#ordinary-twelve-frame-equivalence-for-observed-task-parameters).


**ROMDATA-01-BUS-HOST-LOWER-RESIDUE completed EUR (2026-10-27).**
452 bytes/113 words/62 ranges classified with 114 contextual reads: 404B from
102 Thumb PC-relative LDRs on 101 literals, 48B from 12 observed switch entries 7–18.
Load 080A1146/MOV PC 080A1148 retains Thumb; original prefix normalizes u16 zero/>24
to 1 before index, without new global limit. New optional SiteCapture retains previous
schema/keys; PC/mode scope requires opcode/address/width verification, not automatic
attribution for nested HLE/DMA. One warning-free incremental Windows build after
helper error only in test; one Wine filter with new probes and one current F09 arm
12 frames observes all 113 words. 85 guards/cfg preimage/code/Atlas exact; artifact
helper zero paths. No old acceptance repeated, fixture or measurement. The 452B support
code, e73-d72/67 files 66 resources unchanged. New sources 394FBC/371064 phase 0/
371034 require their own contracts; no format/extent inferred from literals. Linux/
USA/native Windows/new 32:9 to verify, global TODO open.
[Contract and limits](../rom_data_runtime.md#lower-bus-and-host-residue-source-contracts).


**ROMDATA-01-ADDITIONAL-CONSTRUCTOR-OBJ source/model completed EUR (2026-10-27).**
394FBC is a table of four conditional 32-bit threshold/list pointer pairs, 32B;
original readers 9DB56/9DBCA and already authenticated constructor prove four lists
394F82/90/9E/AC, 56B/eight objects. Total 88B, no owner overlap. Increment/store/
reload u8 before comparison; 255→0, new phase >3 disables. Old phase not bounded
before lookup: model 0–3, no new global cap. Isolated model with strict JSON/source
projection/list dependency verification. First warning-free Windows build/single
Wine filter passes 13 guards, 15 counter prefixes, four pointer-constructor prefixes
and four new draws with existing copied oracle. 94 guards/code exact, hub declarations
only/helper visibility only; artifact helper zero paths. Retained F09 does not read
these data: no ordinary use claimed. Production e73-d72/67 files 66 resources unchanged;
extraction/runtime owner/positive use to do. Linux/USA/native Windows/new 32:9 pending,
no old acceptance repeated/fixture/navigation/measurement.
[Contract and model](../rom_data_obj.md#conditional-constructor-obj-animation-source-and-model).


**ROMDATA-03-CONSTRUCTOR-ANIMATION-RUNTIME completed EUR (2026-10-27).**
e74-d73/68 files 67 resources publishes 32B table with OBJ dependency; catalog 80 lists/
193 objects 1318B, all 76 previous lists/126 direct refs/65 other files-records exact.
Exact table owner with previous/raw composition, lists served by existing OBJ owner.
First Wine run passes 10 guards/2 halfsets/lifecycle/publication and 13 original-
interpreter-native state-clock prefixes with 88B raw altered: 36B typed native/
20 thresholds 16 pointers/4 phases, 4 external queries without charges, 708 bus and
composition cases. It then stops on test setup error: load_state before ROM restoration.
Failure retained, no full PASS claimed. Only test order corrected; remaining filter on
already validated dataset passes 6 DMA/SWI and 4 host publications/two actual objects
each, without repeating passed portions. Production identical between filters. Final
builds warning-free, 169 guards/code/Atlas exact; artifact helper 0 paths, failed
diagnostics and offline 68 retained, only superseded never-run exe retired. Real CLI/
positive ordinary use to do; retained inactive F09 insufficient. Linux/USA/native
Windows/new 32:9 pending; no new fixture/measurement/replay.
[Consumption and evidence](../rom_data_obj.md#managed-constructor-obj-animation-and-typed-consumption).


**ROMDATA-04-CONSTRUCTOR-ANIMATION-CLI completed EUR (2026-10-27).**
First warning-free Windows production build/1e98de03..; canonical hash/mtime restored
and main content unchanged. Three Wine CLI calls exit 0: prepare/reuse without writes/
runc 0 frames. 68 generation files/70 managed/67 resources, all 68 hashes identical
to accepted runtime and all 67 runtime IDs/hashes exact; old cache loaded. Reuse and
startup hash/mtime unchanged, 145 guards/439 sources exact; helper 0 paths. Zero frames/
dispatch/fallback/new counters: real registration/attach evidence, no positive ordinary
use inferred. No F09/source-model-runtime-scenario repeated, new fixture/navigation/
measurement. Positive ordinary use and Linux/USA/native Windows/new 32:9 remain to verify.
[CLI and limits](../rom_data_obj.md#actual-cli-for-constructor-obj-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE partial EUR (2026-10-27).**
Passive Wine-only observer implemented and optional positive guard in validator
(seven lines; previous loop exact without guard). First warning-free Windows build;
four normal observations exit 0 but NEGATIVE: two movements 160/200 frames from
copied EUR 12006 checkpoint, delayed selection 160, confirmation 200 from new selection
state. Table 0/phases 0/lists unused, no full pair or positive fixture claimed.
84 guards/code/originals exact; previous normal provenance retained without 3530-frame
replay. No manual RAM/task/phase/ROM mutation. Flag setter 9D054/call 6 verified as
lead, normal action/semantics still to prove; no further blind route or forced flag.
Unit open, fixture/diagnostics retained; independent 371064 phase 0 source separate.
Linux/USA/native Windows/new 32:9 pending, no measurement.
[Evidence still open](../rom_data_obj.md#ordinary-constructor-animation-discovery-remains-partial).


**ROMDATA-01..05-CONDITIONAL-PHASE-ZERO completed EUR (2026-10-27).**
371064 phase 0 now typed: conditional thresholds 0/1/2=16/8/8, 3B span plus
8 corrections, 11B in same file/68 files 67 resources/e75-d74. Old 2 rows +
4 corrections/all 66 other records-files exact. No global cap before lookup or ABI
change. Disjoint coverage partition 3 phases/4 corrections. First warning-free Windows
build/single Wine run passes 4 new original prefixes/7 attach/6 native state-clock/
3 DMA and raw edges. First production build/canonical restored, one auto-prepare
28-frame CLI passes all 68 files/3 phases 4 corrections/36 reads 44B/28 thresholds
16 corrections. Only offline log matcher corrected after CLI exit 0; command not
repeated. Same frontend exe: one new full pair 28 frames 8476→8504 passes every
state-clock/image/OAM/dispatch/audio with samples [61446,61446,7392,7392,61370],
exact CLI counts. 215 guards/code/5-line unguarded loop exact; helper 0 paths. No old
12 frames/source/model/runtime/constructor repeated or new fixture/measurement.
Ordinary constructor remains partial; 371034 and Linux/USA/native Windows/new 32:9
platforms still to verify.
[Contract and evidence](../rom_data_gameplay.md#conditional-phase-zero-threshold-extension).


**ROMDATA-01-TASK-CYCLE-DELTAS / ROMDATA-02-TASK-CYCLE-MODEL completed EUR
(2026-10-27).** Conditional projection 371034–371064/48 signed-i8 deltas, disjoint
from all 67 e75 resources. Readers 18E10/18E2A use OLD u8 before normalization;
increment stored as u8 before helper 1464EC/remainder 48, 255→0. Signed addition/
subtraction on halfword task+8 with low 16 store; axis/unit/lifetime not inferred.
No cap introduced: queries 48..255 None, 48/49/50 belong to old owner. First
warning-free Windows build/single Wine filter passes 23 guards/58 original prefixes/
8 helper edges/all 48 rows and RAM-callee guards, 77 frozen inputs exact. Lossless
model/strict JSON; revision e75-d74/68 files 67 resources unchanged. Management/
runtime consumption/CLI/ordinary 48 completed in units below; source test alone did
not claim consumption or scene. Previous constructor ordinary still partial; Linux/
USA/native Windows/new 32:9 pending, no measurement or new ordinary fixture.
[Contract and evidence](../rom_data_gameplay.md#conditional-european-task-cycle-delta-source-and-model).


**ROMDATA-02/04-TASK-CYCLE-RUNTIME completed EUR/Wine (2026-10-27).**
New resource gameplay.task-cycle-deltas/file task-cycle-deltas.json/e76-d75:
69 files 68 resources, 48B signed-i8/one span, all 67 previous resources/files
exact. Typed owner composes new bytes/prior/raw, no cap or ABI change; USA 8-7.
First warning-free Windows build/first Wine filter passes 9 transactional attach
refusals/2 full-set/interrupted new file, 8 original-interpreter-native prefixes
with equal full state/clock and 48 raw B altered, 5 explicitly compiled reads. All
48 rows, read32 boundary with 2 new + 2 old bytes, edges/mirror, 3 DMA/SWI transfers
across owners, unavailable files/Load/Undo with owner retained. 154 guards/code exact;
helper retires only interrupted copy. No source filter or old acceptances repeated.
Production CLI completed below; positive ordinary 48 deltas completed below;
constructor ordinary partial. Linux/USA/native Windows/new 32:9 pending, no measurement.
[Consumption and evidence](../rom_data_gameplay.md#managed-task-cycle-deltas-and-typed-consumption).


**ROMDATA-05-TASK-CYCLE-CLI completed EUR/Wine (2026-10-27).**
First warning-free Windows production build/canonical restored with exact hash mtime;
three Wine commands prepare/reuse/runc 0 exit 0. All 69 files equal accepted Rust
dataset, all 68 runtime IDs/hashes equal manifest, 48 delta rows registered, existing
native cache loaded. Entire 71-file pathset/hash mtime unchanged on reuse and startup;
226 guards/code exact, helper 0 paths. Zero frames/dispatch/fallback/delta reads:
attachment only; ordinary positivity and full pair 48 completed below. No blind route/
manual flag. Linux/USA/native Windows/new 32:9 pending, no measurement.
[CLI and limits](../rom_data_gameplay.md#actual-cli-for-task-cycle-deltas).


**ROMDATA-05-TASK-CYCLE-ORDINARY completed EUR/Wine (2026-10-27).**
Neutral Bank 48 scene 3660→3708 selected from source and normal snapshot: initializer
18D6C/default 03, generic 015E48/03-07 and candidate Bank root+D40 flags 03 phase 21,
address consistent with caller 5C1F8. Setter 18E5C retains old 04/08 with new 01:
argument 03 alone does not prove mode. 11/3/89 raw BL are candidates, no call graph/
liveness inferred from RAM alone. One existing CLI without rebuild uses 48 reads 48B/
all 48 phases, dataset 71 paths/hash mtime unchanged. First warning-free frontend build
for 5 positive-guard lines; old loop byte-exact without guard. One Wine full pair passes
every full state/clock/image/OAM/dispatch/five audio streams, samples
[105336,105336,12672,12672,105260], exact CLI counts 48/48/48. 97 guards/code/preimage
exact, helper 0 paths. No navigation/RAM-task-ROM/cartridge tampering or old Bank 12/F09
acceptance repeated. Declared cycle children 01/02/04/CLI 05/ordinary 05 closed, total
migration still open; distinct constructor ordinary partial. Linux/USA/native Windows/
new 32:9 pending, no measurements.
[Scene and evidence](../rom_data_gameplay.md#ordinary-european-task-cycle-scene).


**ROMDATA-01-BANK-BUS-HOST-RESIDUE: EUR/Wine census completed, 90B sources open
(2026-10-27).** Optional SiteCapture::start_all mode preserves empty/single filters
and old keys. First warning-free Windows feature build/single Wine run: new probes and
one decoded Bank 12 arm 3660–3672 with existing normal input, no old full pair repeated.
7430 reads/7532 sites, 497 with interpreter context/7035 None; 7083B image, 1434B
outside 68 owners. All 3683B of old NativeAPI included; 992B already classified,
80 literals/320B and 8 dispatch cells/32B verified now: 1344B code support, no global
127 cap from branch with helper. 90B data remain: 18 program/12 directory/60 OBJ lists,
root 4165A0+4114FC via 5E6F4/8, readers 2C5E0/2C60C with tick-phase-selector u16,
distinct from already typed u8 banks. Contract and model of four authenticated prefixes
completed below; management/consumption remain open. No new resource/revision from
census alone. Twelve typed deltas 21–32 read by interpreter LDRB 18E2A explain the
limit of old NativeAPI. 95 guards/code/default preimage exact, helper 0 paths. e76-d75/
USA 8-7 unchanged, user Atlas preserved; Linux/USA/native Windows/new 32:9 pending,
no measurements/fixture.
[Inventory and new sources](../rom_data_runtime.md#bank-bus-and-selected-host-rom-inventory).


**ROMDATA-01-U16-TASK-OBJ-BANK / ROMDATA-02-U16-TASK-OBJ-MODEL completed within
selected EUR prefix scope (2026-10-27).** Four 2C5E0 producers authenticate roots
41634C/411478, 41647C/4114AC, 4165A0/4114FC, 4166C4/41154C: 50 lists 140 OBJ/
7 seq 49 frames/65 spans 1378B disjoint from 68 owners. Relative-index/payload format
verified, padding excluded; seven unreferenced lists and three empty seq remain data,
not reachability evidence/invented holds. u16 counter-phase-selector, low u8 repeat,
return old list before increment, 255→256/65535→0, clamp/repeat/done latch; no cap
before lookup or global archive boundary. Lossless strict JSON model reuses pure
fields, not u8 timing. First warning-free core build: Wine passes 23 guards/1378B
projection then fails test expectation on ctor return r0. Test-only correction; second
warning-free core build/remaining-only filter passes 23 ctor/93 update/6 producer,
RAM-callee/alias/edges. First filter not declared PASS and 23 guards not repeated.
88 guards/199 core sources exact; helper 0 paths. Core dev serde_json preserves parallel
frontend changes without compiling them. Revision e76-d75/USA 8-7 unchanged: management/
consumption/CLI/ordinary for these banks still open; Linux/USA/native Windows/new 32:9
pending, no measurements/fixture.
[Contract and model](../rom_data_obj.md#european-obj-banks-with-u16-task-counters).


**ROMDATA-02/04-U16-TASK-OBJ-RUNTIME completed in EUR/Wine phases (2026-10-27).**
e77-d76 adds graphics.word-task-obj-animations/file word-task-obj-animations:
70 files 69 resources/all 68 previous records-files identical, 65 spans 1378B/
4 banks/50 lists/140 OBJ/7 seq 49 frames. Strict binding/transactional attach; shared
pure projection retains old owner body and gate, new 411/416 gate and host lookup.
Indexing/counters/guest old-list return and ABI/states/cache unchanged. First warning-
free Windows build; Wine passes publication/interrupted file/9 attach guards/2 full-set,
27 original prefixes 7 ctor 20 update/202B compiled/full state-clock, 1378B/6 classes/
217 mixed reads/66B raw gap/old owner/Load-slot Undo without file/DMA3. Then process
without terminal result: no full PASS claimed, receipts retained, no passed phase
repeated. Remaining build compiled clean but rejected by source guard for 2 parallel
DBG changes; stable snapshot. Host test draft isolated per frame; test-only private
field compilation error and OAM/register origin corrected, product unchanged. Final
warning-free build/single host-only Wine run passes 50 lists/140 OBJ/95 host reads with
65 ROM spans altered, accepted OAM per list and equal raw/typed descriptors. 165 guards/
496 snapshot sources/current ROM inputs exact; helper retires only expected interrupted
copy, retains dataset/logs/preimages/executables. Real CLI subsequently verified below;
ordinary subsequently verified in Bank window below, no old model/census/scene acceptance
repeated. USA e8-d7/Linux/native Windows/new 32:9 pending; no measurements or new
fixture/navigation.
[Runtime contract and evidence](../rom_data_obj.md#managed-runtime-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-CLI completed EUR/Wine (2026-10-27).**
One warning-free Windows production build from accepted runtime snapshot copy;
CLI candidate b3bfe6f425923ce461793771fe6c84b86eb5da7dda69b3b9c7e0f5208eab51ea,
canonical executable hash/mtime restored. One Wine script/3 real commands prepare
created/reused/runc 0 pass: 70 generation files identical to runtime/69 registered
resources with exact hashes/72 managed paths. Reuse and startup without hash mtime
writes; original cache loaded, 0 guest frames/dispatch/fallback and 9 new owner fields
all 0. 154 guards/496 copied sources/current ROM inputs exact/helper 0 paths. Dataset
27463ce4f7ffccc3d0b9c8ab5ee78160a3e1c844ed80f68768f9adea87cb7757.
CLI child closed, separate positive ordinary verified below; no old acceptance or
fixture/navigation repeated. Linux/USA/native Windows/new 32:9 pending, no measurements/
Android/package/commit/push. Parallel DBG changes preserved.
[Real CLI and evidence](../rom_data_obj.md#actual-cli-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-ORDINARY completed EUR/Wine (2026-10-27).**
Bank window 3660→3672 chosen from already accepted census/root 4165A0+4114FC,
byte-identical copies of normal snapshot/inputs, no navigation/new checkpoint/
RAM-task-ROM/cartridge tampering. One existing e77 CLI passes positive use:
292 reads/346B = [44 index 20 count 240 attr 2 seq header 20 frame-list 20 duration]/
20 host, 72 managed paths/hash mtime unchanged. Optional 5-line guard on new owner in
full-pair test; old validator/loop byte-exact with it removed. First warning-free Windows
snapshot frontend build/single Wine full pair first PASS: all 12 full states/clock/
central-wide images/OAM/dispatch and 5 audio streams identical, 9 exact CLI counters.
Dispatch [19962,85770], samples [26334,26334,3168,3168,26258]. 94 guards/496 snapshot
sources/current ROM inputs exact/helper 0 paths. No previous model/runtime/scene/census
acceptance repeated or new production build. Declared source/model/management-consumption/
CLI/ordinary children for this family closed; no evidence for all 4 banks/50 lists/
7 unreferenced lists/empty seq update/indirect index/full archive/full game. Distinct
constructor ordinary partial; whole TODO and Linux/USA/native Windows/new 32:9 open,
no measurements/Android/package/commit/push, parallel DBG preserved.
[Scene and evidence](../rom_data_obj.md#ordinary-european-u16-task-obj-bank-scene).


**ROMDATA-01/05-CONSTRUCTOR-FLAG-REACHABILITY closed for EUR/Wine flag
(2026-10-28); ordinary animation still partial.** Twenty-one source windows link
95EA0/wrapper 9A594/global 0200F4A8/state DC/root 960F0: Move 27/A after B cancel/
validation 9D6C8/different destination → count 1A4/28-29, signed count >1 → enable
99F18. Other 3A→99300 and A5544 case 6→A57BE/case 1→A570A are distinct paths, not
global liveness. Two authenticated normal snapshots read only to choose A-only,
without old Right: one reused Wine observer 160 closes fifth negative test, table 0/
phases 0/lists unused; no full pair/positive fixture or new build. Endpoint state 25/
count 3 does not imply invalid destination. New missing flag/slot criterion during
transition: one existing readonly 64-frame CLI captures 448 RAM-image files, normal
Move and flag 20 active 12186–12230/45 postframes; both slots 394/3A4 always inactive/
counter-phase 0/table 0. Ordinary flag reachability proved, slot creation/callback
still to authenticate; no PC/intraframe attribution. 587 guards/496 frozen sources/
21 window hashes exact, originals/e77 dataset 72 paths unchanged/helper 0 paths. No old
acceptance repeated or RAM/ROM/cart tampered with; static disassembler is not a Linux
game test. New full pair only after positive use of 4 phases/4 lists guest-host; no
more blind Move attempts. Linux/USA/native Windows/new 32:9/full TODO open, no
measurements/Android/package/commit/push.
[Flag and slots](../rom_data_obj.md#ordinary-constructor-flag-activation-and-inactive-slots).


**ROMDATA-01/05-CONSTRUCTOR-SLOT-ACTIVATION closed for source gate and map exclusion
(2026-10-28); ordinary still partial.** Eight new windows authenticate 9DB0C→owner+10/
pair [owner,9B11D]→9E300/145C10→9B11C(actor,owner); same slots 394/3A4 as counter
and 9DBA4. 9DAD4 is a distinct earlier helper. Beyond movement and flag 20, normal
branch 9B460 requires unit FD2, signed XY 0..15, valid cell/flags 02/S8 height > actor
S16 height >>4, anim 36!=40; entry-exit bit 8000, skip mask 10000, free slot. Live
cell root 02007F10, width+8/base+4, stride 2 height-flags, no new ROM asset. Readonly
inspection of existing 64 RAM dumps: FD2/actor flags 0/anim 0, three traversed cells
height 2/flags 0; all 256 declared cells without 02, stable 512B hash. This unchanged
map offers no destinations for the gate; no further blind Move or repeat of old five
negatives/flag capture. 87 guards/8 window hashes exact/helper 0 paths, no new gameplay/
build/Wine-Linux test/capture/schema/runtime/ABI/cache/save. Next criterion: readonly
inventory of existing normal EUR snapshots, suitable map and actor and provenance; no
synthetic states or recreation of retired copies. Full pair only after positive use of
4 phases/4 lists guest-host; ordinary/Linux/USA/native Windows/new 32:9/full TODO open,
no measurements/Android/package/commit/push.
[Cell and slot gate](../rom_data_obj.md#constructor-slot-creation-depends-on-live-cell-flags-and-height).


**ROMDATA-05-CONSTRUCTOR-ELIGIBLE-MAP-INVENTORY closed negative (2026-10-28).**
New readonly criterion on 81 normal EUR snapshots/already verified provenance: all
hash/payload/ROM/16x16 domains valid, only 2 map projections 512B (1 F09/80 river),
zero 02 cells/candidates. Old 606 alternate sequence scan not repeated. 88 snapshot-
report-parser-contract-output guards exact; no build/gameplay/Wine-Linux test/capture/
checkpoint/tampering. Terminal negative bound on 2 retained maps, not all maps/future
script; do not repeat same 81 states or Move here. Parent ordinary still partial; next
source lead cell-flag producer 02007F10/terrain ROM before choosing eligible map. Full
pair only after positive use of 4 phases/4 lists guest-host. Linux/USA/native Windows/
new 32:9/full TODO open, no measurements/deployment.
[Retained maps](../rom_data_obj.md#retained-normal-maps-have-no-eligible-constructor-animation-cells).


**ROMDATA-01-CELL-FLAG-WRITERS closed for EUR census/candidate seeds (2026-10-28).**
Old TERRAIN-MAP-SOURCE from the 24th already closed/352 BG packages reused, no new
archive. New direct census 1DE98 no Thumb BL/aligned pointer (not dead code/indirect),
sole BL 1DD68 to 1D798 reads flags 02. New semantic query of authenticated e77 BG
dataset: 113 auxiliary programs/162 links/zero fill-parent-child/ordered u16 writes,
25 programs and 44 IDs initialize 02; flags 0/1/2/3/4/8/9. No decoder/resource/
revision or old codec/loader acceptance repeated. 24 guards/4 source windows exact/
helper 0 paths, no build/gameplay/Wine-Linux test/capture/checkpoint. Private quote
typo corrected before execution; early cleanup rejects unit without result, then passes
after summary, product unchanged. Asset candidates do not prove normal battle/place
name; next actual map selection script/caller → eligible ID/normal reachable battle.
81 snapshots/5 Move bound/flag capture remain closed. Ordinary 4 phases/4 lists
guest-host/full pair and Linux/USA/native Windows/new 32:9/full TODO open, no
measurements/deployment.
[Seeds and sources](../rom_data_maps.md#constructor-animation-cell-flag-seeds).


**ROMDATA-01-CONSTRUCTOR-ELIGIBLE-MAP-SELECTION: source link closed,
reachable battle still partial (2026-10-28).** 253C8 state 10/2549C reads CCF00(7)
→02002037 low u8→1B270/1B689 loader/state 11; 2540C is dispatch table. Opcode 12
advance 2/callback 126831: byte 1 nonzero→CCF18 variable 7; zero→70C8CC+40*U16(02002170)
+2→variable 7, then sets 0200F390=10. 51 CCF18 candidates, two literal 7 arms
authenticated, no all-writer claim. Typed script query 204 instructions 56 default/
148 literal: 19 eligible literals/10 map args [31,33,34,73,75,89,91,109,115,129];
exact references private, not place names/ordinary reachability or event key = active
index. Two retained normal examples active 35/197 map args 99/85→descriptor 98/84/
consistent default/ineligible, not a repeated 81 scan. 37 guards 9 windows exact, no
build/gameplay/Wine-Linux test/capture/checkpoint/runtime/schema/ABI/cache/save. Next
active-index producer/40B record 70C8CC/map byte 2→normal quest/event, no forced ID/
guessed navigation. Positive 4 phases/4 lists guest-host/full pair and whole TODO/
Linux/USA/native Windows/32:9 open, no measurements/deployment.
[Script selection and limits](../rom_data_maps.md#script-map-selection-and-remaining-battle-reachability).


**ROMDATA-01-ACTIVE-ENCOUNTER-RECORDS: producer and conditional join closed EUR
(2026-10-28), complete catalog and ordinary battle still partial.** Setter 9CF0 writes
low16 r0/r1 to 02002170/172; two direct BL 9D2C/1273A8, no aligned Thumb pointer,
without claiming all writers. 9D04 chooses primary/alternate from row+2 or 71D1EC+
12*low8(arg1), according to row+4. 63944 links live selector 219E, record 70B713B3C+45
and 12B row 71C760; 127388 searches event key on rows 255..1, first descending match,
duplicates distinct. New query over caller u8 domain: 208 direct-primary rows, 54 with
seed 02, 44 also with default-map script. Does not prove mission availability, names
or 40B catalog extent; max reference 442 and separate constant 300 are not limits.
Retained normal pub selector 0/row 0/map arg 11 ineligible, readonly. 42 guards/
23 windows exact; private Python import corrected after stop before output; originals
unchanged. No build/gameplay/Wine-Linux test/capture/checkpoint/new resource. Next
criterion: eligible mission/event actually available from normal checkpoint and
original UI. No forced ID/RAM or further blind Move; 4 phases/4 lists guest-host and
ordinary full pair remain open. Old acceptances not repeated; Linux/USA/native Windows/
new 32:9/full TODO pending, no measurements or deployment.
[Producer and evidence](../rom_data_maps.md#active-index-producer-and-event-record-links).


**ROMDATA-05-ELIGIBLE-QUEST-LOCATOR closed EUR/Wine (2026-10-28): eligible mission
present in normal menu.** Observer 309ba31c/e77 reused, no build: a known 620-frame
path retains missing-menu endpoint 14969; then 300 frames 4 Down+A open White Flowers
detail 15269. Old 620 scene not repeated as acceptance; new checkpoint/selectors
criterion. Originals and copies exact, no RAM/task/ROM/cart edit. Live F44C 02024D5C/
count+1B14/pointers+20+4*ordinal, low 10 mission ID from 62126..5E. New 14-entry
query finds 2 direct-primary/default-script/cell 02 candidates: 58/72. Fifth row and
visible detail confirm 58; F448 0202389C +1450→02002268/ID 58 with 60208..28. Event
58/primary 315/map arg 69/descriptor 68 eligible; Giza Plains in description, but
acceptance and battle not yet reached. 130 guards/496 frozen sources/23 windows exact/
helper 0 paths. Candidate layout 031C/03D0 03001028 rejected because not linked to live
menu. Both Wine workers pass, constructor counters 0 in pub expected, not positive
proof/full pair. Next normal confirmation and verify acceptance/battle entry, then
actor/cells before guided Move. 4 phases/4 lists guest-host/full pair, complete 40B
catalog, Linux/USA/native Windows/new 32:9/full TODO pending; old acceptances closed,
no measurements/deployment.
[Normal mission and evidence](../rom_data_maps.md#normal-pub-menu-identifies-an-eligible-quest).


**ROMDATA-05-WHITE-FLOWERS-BATTLE-NAVIGATION partial EUR/Wine (2026-10-28);
normal acceptance closed.** Exact copy of ID 58 detail→loadout OK→cost/objective→
conditions Yes; 300 gil payment/funds 7709→7409 and subsequent Fee:Paid prove active
mission. Entry initially read as QuitPub is QuitMission: only A:Cancel detail opened,
no cancellation confirmation; B/B/Down/A on Leave leads to map. 27 normal captures/
4840 frames, existing observer 309ba31c/e77, no build/manual RAM-task-ROM/cart write.
All Wine workers pass, table 0, not 27 equivalence acceptances/no full pair. World
labels Cyril/LutiaPass/Nubswood/Sprohm, central patch excluded; camera scroll/overshoot
documented. Giza Plains not proved reachable/absent/not placed. 337 guards/496 sources/
17 exploratory window hashes exact. 92524 init/stale task/031C 03001028 do not prove
world dispatch; 165E8 catalog index 725 Giza and base 718/list 3EE60 are only leads,
not coordinates. Partial unit retains all diagnostic checkpoints, no fixture retirement.
Next source selector/placed locations from retained states, no further blind cursor or
27 bound/5 Move/81 map/old acceptances repeated. Battle/actor/4 phases 4 lists guest-host/
full pair and full TODO/Linux/USA/native Windows/new 32:9 remain open, no measurements
or deployment.
[Acceptance and limits](../rom_data_maps.md#normal-white-flowers-acceptance-and-open-battle-entry).


**ROMDATA-01-WORLD-LOCATION-SELECTOR closed EUR/Wine (2026-10-28).** Corrected
inspection assumptions: 254B2→6B40, count task byte+2 not word+4; literal 2810/0300
is 03002810, not 03001028. Optional passive test: one warning-free Windows build
from 497-source snapshot, one neutral Wine frame 18829→18830/2232 reads 2276 sites
323 scoped, actual RAM matches parser. Parent[03002810]=0200F3A4; nodes[parent+67C]
=08395CA8. 375F8 loop 30/stride 20, byte 0/1*8+8 center, conditional hitbox and
1-pixel snap; 37D6A..70 cursor view+6/+8, 37A44 display Y=world Y−scroll C.
385CC..38668 save base+B0+12*i enabled byte 1 bit 0/byte 3 one-based selector→node
(selector−1). Giza index 7/catalog 725/node 20/source 395E24 is placed at 112,296.
Readonly two southern endpoints: 21 cursor 128,287, 22 cursor 128,330; old 27 bound
not repeated. 152 guards/497 sources/32 windows exact/helper 0 paths. New ROM family
30 nodes 20B: only 2 coordinate B classified, 18B/schema/model/extraction/runtime
open. Next short Left from 21, verify Giza label before A; no forced coordinates/RAM/
map ID. Battle and 4 phases 4 lists guest-host/full pair/full TODO/Linux/USA/native
Windows/new 32:9 open, no measurements/deployment/old acceptances repeated.
[Selector and evidence](../rom_data_maps.md#world-selector-and-placed-region-coordinates).


**ROMDATA-05-GIZA-BATTLE-ENTRY closed EUR/Wine (2026-10-28).** Guided short Left
leads to Giza Plains label; A travel/menu White Flowers/A deployment. Primary 315/
map 69/descriptor 68, live grid 512B e8427b5e.. matches seed packet 56/20 cells 02.
Marsh deployed with position/direction confirmed; START To Battle/Yes/dialogue leads
to Move. Actor FD2 (6,5), Left/Down/Down actual cursor (5,7), normal A activates
4 phases/4 lists guest-host: 75 frames/22 reads 88B = 44 threshold 44 pointer.
Checkpoint 22713 + suffix 11 retained, not yet full pair. 16 Wine captures/3875 frames,
existing 309ba31c/e77/no build; 221 guards 496 sources exact/helper 0 paths. Keys
only on copies, no manual guest RAM/task/ROM/cart save modification. First STARTs before
facing confirmation inert/B returns to placement: corrected and closed, do not repeat.
Next single positive CLI 11 and full state-clock-images-OAM-dispatch-audio comparison
from suffix, without repeating discovery/navigation. 40B catalogs/world nodes 18B/full
TODO/Linux/USA/native Windows/new 32:9 pending; no measurements/deployment/old
acceptances repeated.
[Entry and positive evidence](../rom_data_maps.md#normal-giza-battle-entry-and-positive-constructor-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE completed EUR/Wine
(2026-10-28).** Exact copy of new normal Giza checkpoint 22713/suffix 11→22724.
One existing production CLI b3bfe6/e77 reuses dataset/cache and passes 22 reads 88B/
44 threshold 44 pointer/all 4 phases. One Wine full pair 309ba31c passes each of
11 state-clock/classic-central-extended images/OAM/dispatch/five audio streams and all
4 lists guest-host; counters match CLI, dispatch 57833/223470, audio
24138/24138/2904/2904/23966. 91 guards 496 sources/72 managed paths exact/helper 0;
no build/preparation/navigation/discovery/old acceptances repeated, guest edit/cart
write/measurements. Constructor ordinary case closed, not coverage of all actors/maps/
full game. Linux/USA/native Windows/new 32:9/full TODO remain pending; 40B catalogs and
new world nodes 18B to complete. u16/DBG work preserved, no deployment.
[Full ordinary evidence](../rom_data_obj.md#ordinary-constructor-animation-positive-giza-scene).


**ROMDATA-01-WORLD-NODE-RECORDS EUR source projection closed (2026-10-28).**
395CA8..395F00/30 records 20B/600B, count from already proven 375F8 loop; no runtime
cap 30 inferred (low u8 getter/work array 32). Producers 31488/32BE6 write root
[parent 03002810+67C]. Bytes 0/1 coordinates; four slots 2/6/10/14 path ID/neighbor/
opaque 2B plus opaque 2B tail: 300 semantic B/300 opaque B retained. 120 slots/
90 active links/30 path 0/45 paths each in 2 directions. Valid node 0 distinct from
path 0; 4DAD0 matches neighbor before path (22 target 0→0), 46EB0 same node or path!=0/
neighbor. 4DB80 graph copies 4 slots/skips path 0/gate 8-27; reference 397018+(path−1)*8
external, road catalog/payload open. 25 windows (21 new 4 reused)/44 pins/42 guards
pass, 600B disjoint from 49746 spans 69 owners. New criterion on existing trace:
35 reads 42 sites coordinates only, no new edge coverage or repeated audit. No build/
game/test/fixture/guest edit/measurement. Next typed lossless model/source plan and
one targeted Wine oracle; extraction/runtime/CLI/ordinary, Linux/USA/native Windows/
new 32:9/full TODO pending.
[Schema and readers](../rom_data_maps.md#world-node-record-source-projection).


**ROMDATA-02-WORLD-NODE-MODEL completed EUR/Wine (2026-10-28).** world_nodes
catalog authenticates exact image/6 windows/600B hash. Model 30 ordered nodes,
4 Link path/neighbor/opaque 2B + opaque 2B tail, lossless source plan 1 span 600B;
strict JSON/shape distinct from authenticity, no inferred 45/29 cap/opaque zero.
One warning-free Windows core-only build/6c2ef15a.., one Wine filter passes 23 guards/
566 original calls with independent return values/read addresses, 300 semantic B
observed/300 opaque B preserved unread/16 byte aliases/24 outside projection. Callee/
SP/RAM outside stack/VRAM/OAM/palette checked, not native/full state-clock/BFS/
ordinary. 24 guards 215 frozen sources/helper 0 exact. No gameplay/frontend build/
old oracles repeated; e77-d76 and schema 1 unchanged. Next managed publication/typed
guest consumption/real CLI/retained world scene. Linux/USA/native Windows/new 32:9/
full TODO pending, no measurements/deployment.
[Model and proof](../rom_data_maps.md#typed-world-nodes-and-original-reader-proof).


**ROMDATA-03/04-WORLD-NODE-RUNTIME completed EUR/Wine (2026-10-28).** e78-d77
publishes world-nodes.json/gameplay.world-nodes/index 69: 71 files 70 resources, all
69 previous records/files exact, USA e8-d7 unchanged. 600B binding/7 statistics
fields, coordinates/path/neighbor/opaque read from model, prior/raw/edge composition
without cap on guest input or replacement of placement/pathfinder/ABI/save/cache.
One Wine filter passes 12 attach/2 full-set/interrupted publication, 15 native full
state-clock prefixes/32B compiled 124 instructions/600B raw altered after Load, alias/
mirror/outside 30-31/equal 255/root RAM. Bus all 600B/4 classes 30 nodes/width/edges/
prior owner, Load/Undo/unavailable files and 3 DMA/CpuSet/CpuFastSet full state-clock
pass. First 2 builds rejected by stale core cache: dep-info points to old copy, touching
new copy ineffective; moved only specific fingerprint. Third warning-free build/
fresh=false/2cf1d882.. from 502 stable sources. 200 guards/502 sources + 215 model
preimages exact; helper retires only interrupted copy, accepted dataset/executable
retained. Real CLI/ordinary scene still to do. No new gameplay/old model or proof
repeated/measurement; Linux/USA/native Windows/new 32:9/road 397018/full TODO pending.
Parallel DBG/u16 work preserved.
[Consumption and evidence](../rom_data_maps.md#managed-world-nodes-and-typed-guest-reads).


**ROMDATA-04-WORLD-NODE-CLI completed EUR/Wine (2026-10-28); ordinary partial.**
Warning-free Windows production build/e0491cdc.. from 502 stable sources; canonical
hash/mtime restored. Three Wine CLI exit 0 prepare created/reused/runc normal 16,
71 files identical to runtime/70 ID hashes exact/73 managed paths unchanged on reuse-
startup, cache 28859 blocks loaded. Normal Giza 01/02 input copies, 19049→19065:
402 reads 402B coordinates/22 nodes; path/neighbor/opaque 0, no full pair. Private
matcher expected relative frame 16, CLI shows absolute 19065: corrected offline after
all 3 exit 0/guards, no repeat. Bound 16 without path/neighbor closed, next source
callers/justified normal state before new observation. 169 guards/502 sources exact/
helper 0. No new fixture/navigation/previous proof/measurement. Ordinary 3 classes/
full pair/road 397018/full TODO/Linux/USA/native Windows/new 32:9 pending; DBG/u16 work
preserved.
[CLI and ordinary limits](../rom_data_maps.md#actual-world-node-cli-and-ordinary-coordinate-use).


**ROMDATA-01-WORLD-NODE-ORDINARY-CALLERS and ROMDATA-05-WORLD-NODE-TRAVEL-ORDINARY
completed EUR/Wine (2026-10-28).** New direct BL query/10 windows 27 guards links
3B012→4DB1C→4DB80 to view 1A (dest−1) different from parent 7BB; output 7C1.
47034 progress!=FF →4DAD0; entity parent 7B8+52*i. Two existing normal checkpoints
17/selected 20→19/progress FF and entity 1 node 0→1 motivate bound 360, not PC/frame
or all-caller inference. One positive CLI 360 and one full pair 360 pass 6841B =
4820 coordinates 864 path 1157 neighbor/29 nodes, all state-clock/classic-central-
extended images/OAM/dispatch/five audio and CLI counters identical. Opaque 0 distinct
from targeted tests. Dispatch 2006894/3620371, audio 790020/790020/95040/95040/789942.
One warning-free Windows build/dabb1ac6.. from 502 sources with optional 5-line guard;
old validator exact without guard, runtime/core unchanged. 103 guards/502 sources/
27 caller/169 CLI exact/helper 0. First 16 path/neighbor negative remains closed; no
new fixture/navigation/guest edit/old proof repeated/measurements. Ordinary case closed,
not all nodes/maps/opaque ordinary/full pathfinder. Linux/USA/native Windows/new 32:9/
road 397018/full TODO pending, DBG/u16 work preserved.
[Ordinary evidence and gates](../rom_data_maps.md#ordinary-world-node-travel-and-caller-gates).


**ROMDATA-01-WORLD-ROAD-PATHS EUR source projection completed (2026-10-28).**
45 path IDs from 90 active links/30 nodes select 45 headers 8B 397018..397180:
u8 count, opaque 3B, u32 ROM pointer. 45 distinct contiguous payloads
395F00..397018/2188 signed i8 XY pairs/4376B, 46 spans 4736B = 4601 semantic
135 opaque. Root store parent 680 reused; 4DC cost byte 0, 4DD0C flag/index/low u8
step/no cap 45, outside step count output zero 32 without pointer/sample load,
reverse order/no negation, two packed i16/return pointer. 4DD70 flag current>target/
4DD84 min node; caller adds node XY*8 to offsets, 46B64 differences max 5 samples/
facing not full ABI. 3 root literals/3 point calls/1 dir/1 min candidate query,
not global absence of indirect calls. 5 endpoints 1 pixel short 4/6/8/10/12 retained;
zero points absent/count 32..85 not constraints. 6 windows 60 pins 17 guards/
70 owners 49747 spans disjoint/helper 0. No build/game/test/fixture/old proof
repeated/measurement. Next lossless model/source plan and targeted Wine oracle;
extraction/runtime/CLI/ordinary/global catalog/opaque meaning/Linux/USA/native
Windows/new 32:9/full TODO pending. DBG/u16 work preserved.
[Road contract](../rom_data_maps.md#world-road-path-source-contract).


**ROMDATA-02-WORLD-ROAD-MODEL completed EUR/Wine (2026-10-28).** Standalone
world_roads model: 45 ordered headers with u8 count/opaque 3/encoded pointer,
2.188 signed XY pairs and shared source plan 46 spans/4.736 B. Strict JSON,
consistent partition and separate authenticity; five short endpoints, possible
zero sample and structural count 0/255 retained without inferred 32..85 cap.
One warning-free Windows core-only build/de9d1e11.. and one Wine filter pass
29 guards/2.537 original calls: 2.188 forward points, 135 reverse, 180 exits,
6 u8 aliases, 6 external headers and 22 order helpers. Exact outputs/returns/read
sequences; all 4.601 semantic B read, 135 opaque B reconstructed/unread.
32 preservation guards/504 build files (218 core) exact/helper 0. No ordinary scene/
fixture/frontend build/measurement or already closed node proof repeated. Schema 1/
e78-d77 unchanged. Next managed extraction/typed guest reads, real CLI and one
positive ordinary scene. Linux/USA/native Windows/new 32:9, global catalog extension
and full TODO pending.
[Model and proof](../rom_data_maps.md#typed-sampled-world-roads-and-original-reader-proof).


**ROMDATA-03-04-WORLD-ROAD-RUNTIME completed EUR/Wine (2026-10-28).**
e79-d78/index 70/gameplay.world-roads/world-roads.json: 72 files 71 resources, all
70 previous records/files exact, USA 8-7/schema 1 unchanged. Immutable count/pointer/
sample/opaque owner from 46 spans 4736B; composes world node at left boundary and
raw right/header 45..255, no movement/facing/pathfinding algorithm replaced. Two
warning-free Windows builds/accepted 52be2981.. from 506 stable files; first filter
closes publication/attach, then fails native case 3 due to omitted page 396. Page gate
fix and remaining native/bus/lifecycle filter without republishing: 12 attach guards/
2 full-set, 12 original/interpreter/native full state-clock prefixes, 67 compiled
instructions/44B explicit, 4736B sources poisoned after Load; all 4736B/4 classes
45 paths/widths/mirrors/edges/prior owner/Load Undo and DMA3 PASS. 221 guards 506
source hashes + 504 model preimages exact; helper retires only interrupted copy. No
scene/model/nodes/fixture repeated, measurement or ABI change. Real CLI and one
positive ordinary scene next; Linux/USA/native Windows/new 32:9/global extent/full
TODO pending.
[Consumption and evidence](../rom_data_maps.md#managed-world-roads-and-typed-guest-reads).


**ROMDATA-04-WORLD-ROAD-CLI completed EUR/Wine (2026-10-28).** Warning-free Windows
production build/1a86d341.. from 506 accepted sources, canonical hash mtime restored.
Three Wine commands prepare created/reused/runc 360: 72 files exact to runtime/71 ID
hashes/74 paths unchanged hash mtime, old 28859 cache blocks loaded. Normal Giza
01 state/02 input copies 19049→19409: 2976 reads/5067B = 885 count/2788 pointer/
1394 sample/0 opaque/40 paths, 3 semantic classes positive. 180 guards 506 hashes/
helper 0; no new scene, navigation, measurement or already closed proof repeated.
Next one ordinary full pair with identical CLI counts; not all 45 payloads/maps/
opaque ordinary/global extent. Linux/USA/native Windows/new 32:9/full TODO remain pending.
[CLI and consumption](../rom_data_maps.md#actual-sampled-road-cli-and-ordinary-consumption).


**ROMDATA-05-WORLD-ROAD-TRAVEL-ORDINARY completed EUR/Wine (2026-10-28).**
One normal Giza 01/02 Wine full pair 360 with CLI dataset e79-d78, 19049→19409:
all 360 state-clock/classic-central-wide/OAM/dispatch/five audio exact. Seven counts
match CLI: 2976 reads 5067B = 885 count 2788 pointer 1394 sample 0 opaque, 40 paths
(any field, not 40 payloads). Optional 5-line guard preserves old validator byte exact;
one warning-free Windows build/4c453f07.. from 506 stable files. 100 guards 506 hashes
+ 180 CLI guards exact/helper 0; no new scene/fixture/navigation/measurement or already
closed source/model/runtime/node proof repeated. Road ordinary case closed; all 45
payloads/maps/opaque ordinary/full ABI/global extent, Linux/USA/native Windows/new 32:9
and full TODO still pending.
[Ordinary equivalence](../rom_data_maps.md#ordinary-sampled-road-travel-equivalence).


**ROMDATA-01-ENCOUNTER-CATALOG-READERS EUR contract completed (2026-10-28).**
Declared physical catalog 442 headers 40B 1..442: origin 70C8CC, first 70C8F4 from
two independent roots, next 710E04 independent 28B family. 17680B, no assumption
that max 442 is count/nor constant 300/nor guest cap. 442 contiguous template links
2850 records 48B/136800B corroborate partition. 17 semantic fields/33B and 7 opaque
per header; original low 16 indices without 442 guard. Index 0 is coordinate prefix:
last 16B payload of header 442, no invented valid header. Header 29+4 pointer to
70CD58 already owned by constructor-alternates: dependency/consistency/skip 4B,
2 new spans 17676B, 14582 semantic/3094 opaque disjoint from 71 owners 49793 spans.
Already owned 192B template retained; unit/coordinate payload separate, not migrated.
27 loaders from cached root/40 windows (39 new 1 reused)/54 pins 68 guards/helper 0
PASS. No build/game/test/fixture/previous proof repeated. Next lossless 442-header
model/shared pointer + one Wine oracle; extraction/runtime/CLI/ordinary, all indirect/
global extent/Linux/USA/native Windows/new 32:9/full TODO pending, no measurement.
[Headers and ownership](../rom_data_maps.md#european-encounter-headers-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-HEADERS-MODEL completed EUR/Wine (2026-10-28).**
Lossless encounter_headers model 442 headers 1..442: 17 fields/opaque 7, 17680B full;
2 spans 17676B exclude 70CD58 len 4 from constructor-alternates owner. Strict JSON/
shape separate from authenticity/dependency consistency; modified shared pointer leaves
spans unchanged but is rejected. Low u16 alias/None 0-443..65535/prior raw, no inferred
count cap/null pointer/enum/opaque zero. One warning-free Windows core-only build
5855af92.. and one Wine filter pass 29 guards/7523 probes: 7072 single-load prefixes
and 451 complete C0034 calls (442 + 6 aliases + 3 outside), all 14586 semantic B incl
4 shared observed, 3094 opaque B reconstructed/unread. Reg/flags/PC prefix, callee/
SP/LR gate, RAM outside stack 4/VRAM/OAM/palette checked; not full ABI/importer/native
clock/ordinary. 33 guards 508 files (221 core) exact/helper 0, e79-d78 unchanged, no
old oracles/scene/fixture/measurements. Next managed/typed bus/source page 70C..710/
dependency, then CLI/ordinary; nested payload/all callers/global/Linux/USA/native
Windows/new 32:9/full TODO still pending.
[Model and proof](../rom_data_maps.md#typed-encounter-headers-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-HEADERS-RUNTIME completed EUR/Wine (2026-10-28).**
e80-d79/index 71/encounter-headers.json: 73 files 72 resources, all 71 previous exact,
constructor-alternates dependency and shared-pointer consistency. Owner 2 spans 17676B
excludes 4 old B, source pages 70C..710, 8 classes/442 header mask/11 stats; guest
algorithms/indices/ABI/cache/save unchanged, USA 8-7/schema 1. One warning-free Windows
build 1c4135f5.. from 510 stable files, one Wine filter: 14 attach guards/3 full-set/
failed owner-state, 26 original/interpreter/native full state-clock prefixes with 17680B
raw poison, 31 compiled instructions/37 new B + 4 old B. All 17676 new + 4 shared B/
8 classes/442 headers/widths/mirrors/edges/prior owner/Load Undo/DMA3 PASS. 206 guards
510 hashes + 508 model preimages exact/helper retires only interrupted. No scene/fixture/
measurement/7523 model proof/already closed node-road-constructor repeated. Next real
CLI/ordinary; nested payload/all callers/global/Linux/USA/native Windows/new 32:9/full
TODO pending.
[Consumption and evidence](../rom_data_maps.md#managed-encounter-headers-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-HEADERS-CLI completed EUR/Wine (2026-10-28).** One warning-free
Windows production build/9d0a772d.. from 510 accepted sources; canonical hash mtime
restored. Three Wine commands prepare created/reused/runc 700 pass: 73 files exact to
runtime/72 ID hashes/75 paths no-write, 28859 cache blocks loaded. Normal Giza 02 menu/
03 input copies 19409→20109: 17 reads/41B, six positive semantic classes/1 header/
0 opaque. 184 guards 510 hashes/helper 0; no new navigation, checkpoint, measurement
or closed proof repeated. Next one full pair with identical CLI counts; not all 442
headers/payload/opaque ordinary/full ABI/global catalog. Linux/USA/native Windows/new
32:9/full TODO pending.
[CLI and consumption](../rom_data_maps.md#actual-encounter-header-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-HEADERS-BATTLE-ORDINARY completed EUR/Wine (2026-10-28).**
One normal full pair 700 Giza 02 menu/03 input 19409→20109 with CLI dataset e80-d79:
every full state/clock/classic-central-wide image/OAM/dispatch/5 audio passes. All 11
counts match CLI: 17 reads 41B = 12 count/1 map/4 template pointer/0 parameter/
1 placement count/20 position pointer/3 control/0 opaque, one header; six positive
classes. Parameter ordinary and opaque not observed, previous targeted evidence distinct.
One optional 5-line guard/old validator byte exact, one warning-free Windows build/
43fcc614.. from 510 sources and one Wine filter. 103 ordinary + 184 CLI guards/510
hashes each exact/helper 0. No new fixture/navigation/closed proof repeated or
measurement. Ordinary case closed; all 442 headers/payload/full ABI/global catalog,
Linux/USA/native Windows/new 32:9/full TODO remain pending.
[Equivalence](../rom_data_maps.md#ordinary-encounter-header-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-UNIT-TEMPLATES completed for the EUR source contract
(2026-10-28).** The 442 already authenticated header links partition 2.850 records
48B, 6EA048..70B6A8/136.800B; independent complete projection. New readers separate
selectors/arguments, five u16 equipment values, halfword 12, physical bitset 14..27
with dynamic index without 20B/160-bit cap, controls 28/29/flags 2A/2C/2D and opaque
2E..2F. All 14.250 equipment references within 376 items, selectors consistent with
metadata14/metadata52 owners; observed values are not caps. Old 192B constructor
6ECA78..6ECB38/records 225..228/header 29 excluded from new 136.608B/two spans;
full 48 coherence and header/constructor dependencies required, no duplication among
49.795 spans/72 owners. Eight windows 5 new 3 reused/39 pins/4 BL/41 guards pass,
no build/game/Wine-Linux test/fixture/old proof or census repeated/measurement. Next
strict model + one targeted Wine oracle; managed/runtime/CLI/ordinary/coordinate pool/
global/full ABI/Linux-USA/native Windows/new 32:9/full TODO pending.
[Contract](../rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-UNIT-TEMPLATE-MODEL completed EUR/Wine (2026-10-28).**
Strict lossless model 2.850 records 48B, full 136.800B/two spans 136.608 new,
192B previous constructor excluded from byte_at but compared in full. Shape/JSON/
header partition/shared coherence/item-metadata14-job references and source auth
separate; full u8/u16/opaque 2, no invented value caps or 160-bit guest cap. One
warning-free Windows core-only build/e0ae0d5c.. from 512 sources/224 core inputs,
one Wine filter passes 38 guards/2.100 selected original prefixes: 1.020 LDR,
1.080 bit including 120 indices beyond 20B. 1531 new/102 old/4 raw distinct bytes
observed, registers/addresses/values and all RAM/VRAM/OAM/palette/ROM exact;
independent full projection, not full ABI/semantic coverage of all 2.850 records.
34 guards 512 hashes + 41 source guards exact/helper 0; no new fixture, measurement
or closed proof repeated. Next e81-d80/index 72 publication + typed consumption,
then CLI/ordinary; Linux-USA/native Windows/new 32:9/coordinate pool/global/full ABI/
full TODO pending.
[Model and evidence](../rom_data_gameplay.md#typed-encounter-unit-templates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-UNIT-TEMPLATE-RUNTIME completed EUR/Wine (2026-10-28).**
e81-d80/index 72/encounter-unit-templates.json, 74 files 73 resources/all 72 previous
exact/five dependencies; 192B constructor remain with old owner. Immutable owner
136.608B/two spans, nine classes and record/header stats, local/global 6EA..70B routing.
One warning-free Windows build/3d4b2af2.. from 514 sources and one Wine filter pass
14 attach/7 full-set/transactional failures, 27 original-interpreter-native prefixes
with full state/clock and 136800B raw altered after Load, 25 new B/8 prior B compiled.
Every 136608 new B + 192 old/9 classes/2846 records 441 headers, widths/mirrors/
boundaries/Load Undo and DMA word 34202/CpuSet 68404 halfword/FastSet 34208 word
pass. 215 guards 514 hashes 512 model preimages exact/helper only interrupted. ABI/
algorithms unchanged; USA 8-7/schema 1. Next real CLI + ordinary; coordinate pool/all
callers/full ABI/global/Linux-USA/native Windows/new 32:9/full TODO pending, no
measurements.
[Consumption and evidence](../rom_data_gameplay.md#managed-encounter-unit-templates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-UNIT-TEMPLATE-CLI completed EUR/Wine (2026-10-28).**
One warning-free Windows production build/afeaef25.. from 514 accepted runtime sources,
canonical hash mtime restored. Three Wine commands prepare created/reused/runc 700
pass: 74 files exact to runtime/73 ID hashes/76 paths no-write, 28859 cache blocks.
Normal Giza 02 menu/03 A 700 input copies 19409→20109: 186 reads/211B, 5 records/
1 header, 8 positive classes and 1 unobserved. Positive selector/argument/equipment/
flag criterion satisfied; all 13 counts retained for subsequent full pair. 184 guards
514 hashes/helper 0, no new fixture/navigation/measurement or closed proof repeated.
Ordinary full pair/all 2850 records 442 headers/full ABI/coordinate pool/global/
Linux-USA/native Windows/new 32:9/full TODO pending.
[CLI and consumption](../rom_data_gameplay.md#actual-encounter-unit-template-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-UNIT-TEMPLATE-BATTLE-ORDINARY completed EUR/Wine
(2026-10-28).** One normal Giza 02 menu/03 input full pair 700 19409→20109 with
CLI dataset e81-d80: every full state/clock/classic-central-wide image/OAM/dispatch/
5 audio passes. All 13 counts match CLI: 186 reads 211B, 35 selector/15 argument/
80 equipment/10 auxiliary/41 bit/10 optional/10 flag/10 control/0 opaque, five
records and one header; eight semantic classes positive. Opaque ordinary not observed,
separate bus proof. One optional 5-line guard/old validator byte exact, one warning-
free Windows build/33c61082.. from 514 sources and one Wine filter. 104 ordinary +
184 CLI guards/514 hashes each exact/helper 0; no new fixture/navigation/measurement/
closed proof repeated. Case closed; all 2850 records 442 headers/full ABI/coordinate
pool/global/Linux-USA/native Windows/new 32:9/full TODO pending.
[Equivalence](../rom_data_gameplay.md#ordinary-encounter-unit-template-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-COORDINATES completed for the EUR source contract
(2026-10-28).** 691 lists/2342 XY pairs/4684B 70B6A8..70C8F4 exactly partitioned
by header pointer-count links: no gap/alias/overlap among 49797 spans 73 owners.
323 placement 0 lists 3272B/34 random 680B/12 placement 2 = 88B/322 default 644B,
323 headers with payload 119 all null. Raw u8 XY preserve signed search and unsigned
reads; RNG 02868 nonnegative 15-bit/remainder 146314/mask u16 remain original, no
invented 10/16 cap or safe null fallback. Nine reused + three new helper/RNG windows,
41 pins/38 guards pass; no build/Wine-Linux test/game/fixture/navigation/closed proof
repeated/measurement. Next strict 691-list model + header coherence and one targeted
Wine oracle, then extraction/runtime/CLI/ordinary. Schema 1/e81-d80/USA 8-7 unchanged;
Linux-USA/native Windows/new 32:9/all callers/full ABI/global/full TODO open.
[Contract](../rom_data_maps.md#european-encounter-coordinate-lists-and-original-readers).


**ROMDATA-02-ENCOUNTER-COORDINATE-MODEL completed EUR/Wine (2026-10-28).**
Strict model 691 lists 2342 raw u8 XY pairs/4684B/691 spans; shape/header coherence/
authenticity separate, diagnostic signed view, no safe null fallback/value cap.
First Windows core build fails due to Bus import in test; only import corrected,
second 516-source copy accepted/2fe3e236.. warning-free/227 core inputs, preimages
retained. One Wine filter passes 30 guards/8044 original signed-unsigned prefixes
across all 4684B, 12 isolated high bits and 102 original random prefixes (34 lists *
3 seeds). RNG/index/addresses/u16 mask/writes/callee-SP/outer RAM-VRAM-OAM-palette-ROM
exact; not full ABI, repeated random/exhaustion/native clock/ordinary. 40 guards
516 accepted + 516 failed hashes/38 source guards exact/helper 0; no new fixture/
navigation/closed proof repeated/measurement. Next managed e82-d81/index 73 + typed
bus, then CLI ordinary; Linux-USA/native Windows/new 32:9/all callers/global/full TODO
pending.
[Model and proof](../rom_data_maps.md#typed-encounter-coordinates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-COORDINATE-RUNTIME completed EUR/Wine (2026-10-28).**
e82-d81/index 73/encounter-coordinates.json, 75 files 74 resources/all 73 previous
exact/header dependency. Immutable owner 691 spans 4684B/four roles + XY/list-header
statistics, local and global 70B..70C routing/composition of old owners at edges.
Three warning-free Windows builds, accepted 0f85fbf7.. from 518 sources. First Wine
filter passes publication/14 attach/3 full-set and stops in native harness; second
isolates concatenation beyond stop. Only test corrected with BL pair and two families
that exclude their respective stops. Third filter reuses dataset, retains all 22 cases
and completes native/lifecycle without repeating publication or attach. The 22 original-
interpreter-native cases match full state and clock with 4684B raw altered after Load
(18 loads/index 0 gate/3 random prefixes), 21 new B/2 template/13 header compiled.
All 4684B/691 lists 323 headers/XY roles/691 boundaries/width-mirror/Load Undo/DMA-
CpuSet-FastSet pass. 253 guards/518 hashes 516 model preimages exact/helper only
interrupted; algorithms/ABI unchanged. Next real CLI + ordinary; Linux-USA/native
Windows/new 32:9/full ABI/all callers/global/full TODO pending. No fixture/navigation/
measurement/old proof repeated.
[Consumption and evidence](../rom_data_maps.md#managed-encounter-coordinates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-COORDINATE-CLI completed EUR/Wine (2026-10-28).** One
warning-free Windows production build/b467d4ad.. from 518 accepted sources, canonical
hash mtime restored. Three Wine commands prepare created/reused/runc 700 pass: 75 files
exact to runtime/74 ID hashes/77 paths no-write/28859 cache blocks loaded. Normal Giza
02 menu/03 input copies 19409→20109: 20 reads 20B/1 list 1 header, placement 0-X-Y
positive; all 10 counts retained. 186 guards 518 hashes/helper 0, no new fixture/
navigation/measurement/closed proof repeated. Full pair next, not all 691 lists/roles/
headers/ABI. Linux-USA/native Windows/new 32:9/global/full TODO pending.
[CLI and consumption](../rom_data_maps.md#actual-encounter-coordinate-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-COORDINATE-BATTLE-ORDINARY completed EUR/Wine
(2026-10-28).** One normal Giza 02 menu/03 input full pair 700 19409→20109 with
CLI dataset e82-d81: every full state/clock/classic-central-wide image/OAM/5 audio
passes; native and fallback dispatch executed. All 10 counts match CLI: 20 reads 20B,
placement 0=20, X 10/Y 10, one list and one header. Random/placement 2/default not
observed here, previously completed targeted tests remain separate. Optional 5-line
guard/old validator byte exact, one warning-free Windows build/de67e015.. from 518
sources and one Wine filter. 104 ordinary + 186 CLI guards/518 hashes each exact/
helper 0. Case closed; all 691 lists 323 headers with payload/placement-RNG/full ABI/
global/Linux-USA/native Windows/new 32:9/full TODO pending. No new fixture, navigation
or performance measurement.
[Equivalence](../rom_data_maps.md#ordinary-encounter-coordinate-battle-entry-equivalence).


**ROMDATA-01-LINKED-METADATA28 completed for the EUR source contract
(2026-10-28).** 347 records 28B/9716B 710E04..7133F8, 17 roots and 28B arithmetic,
next 4B family with two independent readers. Physical boundary, no guest cap 347.
Getter D07A4 low16/low8: scalars, internal view +12, word flag/21 bits, selectors
34..255→0 without data reads. Direct readers u16+22/byte 24..26; opaque bytes 3/15/27
retained. All 347 records reconstructed, no overlap with 74 owners/50488 spans,
20 windows 14 new 6 reused/69 pins/122 guards/helper 0. Static contract only;
model/Wine/runtime/CLI/ordinary next. Schema 1/e82-d81 unchanged. Linux-USA/native
Windows/new 32:9/global/full TODO pending; no build/game/checkpoint/navigation/
measurement or closed proof repeated.
[Contract](../rom_data_gameplay.md#european-indexed-28-byte-metadata-source-contract).


**ROMDATA-02-LINKED-METADATA28 completed EUR/Wine (2026-10-28).** Lossless model
347 records/9716B, unsigned fields/flag32/opaque bytes, shape/source auth separate;
low16/low8 lookup retains scalars/address view +12/21 bits/default without read,
no invented guest cap or null fallback. One warning-free Windows core build/
358dba2c.. from 520 stable sources and one Wine filter: 27 guards/14517 original
prefixes (12088 complete getters 1388 tail 1041 inline), 8675 interpreted bytes
observed/1041 opaque reconstructed, 9 isolated variants 342 prefixes; RAM/callee/
SP-LR/VRAM/OAM/palette/ROM exact. 36 model guards 520 hashes + 122 source exact/
helper 0. Schema 1/e82-d81 unchanged; managed/runtime next then CLI/ordinary. Full ABI/
next 4B family/global/Linux-USA/native Windows/new 32:9/full TODO pending, no
measurement or new fixture/navigation/old closed proof repeated.
[Model and proof](../rom_data_gameplay.md#typed-indexed-28-byte-metadata-and-original-reader-proof).


**ROMDATA-03-04-LINKED-METADATA28-RUNTIME completed EUR/Wine (2026-10-28).**
e83-d82/index 74/linked-metadata28.json: 76 files 75 resources/all 74 previous exact,
one span 9716B/no invented dependency. Immutable typed owner 5 classes/347 records/
8 counters, local + global pages 710..713, previous header/raw successor boundaries.
One warning-free Windows build/040133f5.. from 522 sources, one Wine filter publication/
interruption 14 attach 2 full-set/23 native full state-clock cases with 9716 raw bytes
altered after Load: 17 loads + 6 D07A4 getters, 31 new bytes + 1 header compiled.
All 9716 bytes/5 classes 347 records/all 347 boundaries/Load Undo/DMA-CpuSet-FastSet
3 state-clock pass. 216 guards 522 hashes 520 model preimages exact/helper retires
only interrupted copy. Actual CLI/ordinary next; ABI/USA 8-7/schema 1 unchanged, no
new game/checkpoint/navigation/measurement or old closed proof repeated. Full ABI/
global/next 4B family/Linux-USA/native Windows/new 32:9/full TODO pending.
[Direct consumption](../rom_data_gameplay.md#managed-indexed-28-byte-metadata-and-typed-guest-reads).


**ROMDATA-04-LINKED-METADATA28-CLI registration completed EUR/Wine;
positive ordinary use still open (2026-10-28).** One warning-free Windows production
build/0acb352e.. from 522 sources: only correction of 3 metadata28 messages, inverse
byte exact/other 521 files exact/canonical hash mtime restored. Create/reuse/normal
starts: 76 files exact to runtime/75 ID hashes/78 paths no-write/28859 cache blocks.
Four normal locators Giza 700/Jobs 540/Abilityset 700/BattleTechRinfo 800 have ALL
8 counters 0, retained as negatives not to repeat; no full pair executed. Menus
observed, new input from normal commands on authorized copy; no new checkpoint or
manual guest edit. 220 guards 522 hashes/helper 0. Next: combat path with nonzero
action before positive/full pair. Linux-USA/native Windows/new 32:9/full ABI/global/
full TODO pending; no measurement.
[CLI and limits](../rom_data_gameplay.md#actual-indexed-metadata28-cli-registration-and-ordinary-locators).


**ROMDATA-04-05-LINKED-METADATA28-COMBAT completed EUR/Wine (2026-10-28).**
Normal opening of Steal menu on dry ground at Giza: 42 frames 22669→22711, 15 reads
20 bytes served/five records, 10 halfword + 10 byte; inline/flag/opaque 0. Capture,
existing CLI 0acb352e.. and one Wine full pair match all 8 counts; every full state/
clock/classic-central-wide image/OAM/5 audio passes. No ability effect executed or
all 347 records claimed. One warning-free Windows test build/5a6c81be.. from 523
sources, optional 5-line guard/old validator byte exact and normal-input-only observer;
seven captures 1182 frames on authorized copies, no manual guest/original save edit.
177 guards 523 hashes + 220 CLI/522 hashes exact/78 paths no-write/helper 0. Four old
negative locators remain closed; source/model/runtime/registration or other family
evidence not repeated. Ordinary case closed; effects/all 347/full ABI/global/next 4B
family/Linux-USA/native Windows/new 32:9/full TODO pending, no performance measurement.
[Equivalence](../rom_data_gameplay.md#ordinary-metadata28-steal-menu-equivalence).


**ROMDATA-01-LINKED-RECORDS4 completed for the EUR source contract
(2026-10-28).** Range 7133F8..713744, 844 bytes in 211 physical 4-byte positions;
six roots and original readers with u8 index multiplied by 4, distinct boundary with
next family at stride 8. All four fields read unsigned; original addresses retained
in live context. 1041 references from first three inline positions of 347 metadata28
records reach 194 distinct positions, three references to index 0 and none outside.
No invented 211/max 208 cap, global limit on three indices, enum or null fallback.
Zeros 0/1/209/210 remain distinct physical records; reserved/padding meaning of final
two still to clarify. Full 844-byte reconstruction, no overlap with 75 resources/
50489 spans e83-d82; 13 authenticated windows (6 new, 7 reused), 45 pins and 112
guards exact/helper 0. Static analysis only, without build/test/game/fixture or repeated
closed evidence. Next model and targeted original-reader Wine proof; management/runtime/
CLI/ordinary use of THIS family still to implement. Linux/USA/native Windows/new 32:9/
full ABI/indirect catalogs/full TODO pending; no measurement.
[Contract](../rom_data_gameplay.md#european-linked-four-byte-record-source-contract).


**ROMDATA-02-LINKED-RECORDS4 completed EUR/Wine (2026-10-28).** Lossless core model:
211 physical positions with four u8 fields/844 bytes/one span; strict shape, order/
indices/parser separate from EUR authentication and 13 windows. Original guest address
calculated for all 256 indices, including external 211..255; diagnostic None does not
become guest fallback. Distinct zeros and meaning of final two still open, no invented
enum/cap/global selector limit. One warning-free Windows core build/a7270580.. from
525 stable sources (233 core inputs), single Wine filter: 20 guards/1567 original calls
or prefixes, 1055 readers cover all 844 bytes and 512 full setters cover both contexts/
all 256 indices/low8 aliases/external addresses. Two isolated high-bit variants pass
10 unsigned prefixes and are rejected by base authentication. Full RAM except expected
5 setter bytes/callee/SP-LR/IWRAM/VRAM/OAM/palette/final ROM exact; 9808 instructions
coverage only. 125 guards 525 hashes and 112 source exact/helper 0. Schema 1/e83-d82/
save/native/CAPI unchanged. Next managed/runtime then CLI/ordinary use of this family;
native clock/full ABI/Linux/USA/native Windows/new 32:9/indirect catalogs/full TODO
pending. No game/checkpoint/measurement/repeated previous closed evidence.
[Model and readers](../rom_data_gameplay.md#typed-four-byte-linked-records-and-original-reader-proof).


**ROMDATA-03-04-LINKED-RECORDS4-RUNTIME completed EUR/Wine (2026-10-29).**
e84-d83/index 75/linked-records4.json: 77 files 76 resources, all 75 previous records/
payload exact, one span 844 bytes without invented dependencies. Immutable typed
owner/four fields/211 positions/seven counters; page 713 local + global, bytewise
composition with previous metadata28 and raw successor. No new cap/null/selector limit,
save/native/CAPI ABI and USA 8-7 unchanged. Three Windows builds: first failed due to
test reference and stale menu cache, second warning-free never executed, third warning-
free 33144a8c.. from 551 current sources; only LDR alignment expectations corrected
before Wine. One Wine filter passes publication/interruption 14 attach 2 full-set/
owner-state intact/21 native full state-clock cases with all 844 raw bytes altered
after Load: 13 loads 4 getters, one double prefix and 3 setters/four groups; 41
instructions/21 new bytes 6 previous. All 844 bytes/four fields 211 positions/all
211 boundaries/Load Undo same Arc and DMA3/CpuSet 424 halfword/FastSet 216 word pass.
237 guards 551 hashes + two 551 diagnostics/525 model preimages/seven current hashes
exact; helper retires only interrupted copy. Next real CLI/ordinary use; Linux/USA/
native Windows/new 32:9/full ABI/external catalogs/full TODO pending. No measurement/
new game/fixture/repeated old closed evidence.
[Direct consumption](../rom_data_gameplay.md#managed-four-byte-linked-records-and-typed-runtime-reads).


**ROMDATA-04-LINKED-RECORDS4-CLI registration completed EUR/Wine;
positive ordinary use pending (2026-10-29).** One warning-free Windows production
build/bcf632fa.. from 551 runtime-identical sources, without overlay; only isolated
corelib/prod fingerprints and canonical hash mtime restored. Three Wine CLI children
exit 0 (create/reuse/new 120 start): 77 files exact to runtime, 76 IDs/hashes loaded,
79 managed paths hash mtime no-write, 28859 cache blocks. Steal menu 22711 copy with
Armor selected, A 8 hold 12→22831 shows targeting but all 7 new counters are 0.
Negative case closed, no full pair or positive use inferred; final checkpoint not
saved by CLI command. 187 guards/551 CLI + 551 runtime hashes exact/helper 0, originals
unchanged; no new fixture or repeated old closed evidence. Next normal combat/target
resolution with positive reads then full equivalence. Linux/USA/native Windows/new
32:9/full ABI/external catalogs/full TODO pending, no measurement or assistant change
to Android/package/commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-linked-four-byte-record-cli-registration).


**ROMDATA-04-05-LINKED-RECORDS4-COMBAT completed for one EUR/Wine ordinary case
(2026-10-29).** Normal Fight/Doit/facing commands lead to next turn; suffix 40 frames
24755→24795, 223 reads/223 bytes served, fields 64/132/11/16 and two physical
positions. All 7 counters identical between capture, existing BCF CLI and one Wine
full pair; each of 40 full state/clock/classic-wide pixels/OAM/5 audio passes. One
warning-free Windows build/a5b2fc84.. from 552 sources: normal observer, test module
and optional 5-line guard only; inverse validator exact/core unchanged. Ten captures
2084 frames on authorized copies, no manual guest/ROM/cart edit; one classic CLI frame
UI only, then authenticated readonly framebuffer projections without further execution.
All 76 ID hashes/79 paths no-write/28859 cache, 210 guards 552 hashes + 187 CLI guards
551 hashes exact/helper 0. Previous zero cases remain closed; no source/model/native/
lifecycle/registration repeated. All 4 fields observed, not all 211 positions or attack
success/full ABI. New 8B family/indirect catalogs/Linux-USA/native Windows/new 32:9/full
TODO pending, no measurement.
[Equivalence](../rom_data_gameplay.md#ordinary-linked-four-byte-turn-equivalence).


**ROMDATA-01-CONDITIONAL-RECORDS8 EUR source contract completed
(2026-10-29).** D56A0 scan at root 713744, stride 8 and ordinals 1..127:
127 positions/1016 bytes up to distinct root 713B3C, whose two readers use stride 70.
Byte 00 zero skips position, byte 01 zero wildcard; LE16 keys at +2/+5 and u8
operands at +4/+7. Live flags up to 05FF and u8 counters for key-0600: first flag
requires equality for operands 0/1 and counter>=operand; second flag requires
inequality and counter<operand, only second key 0 bypasses. No invented Boolean
coercion/cap/header index. 76 enabled rows/51 zero, 105 live references 104 flag
1 counter/63 keys/24 alias groups; operand 2 retained, 47 second key 0. Original
ROM pointer outputs in order, count; u32 capacity/filter without capacity 0 entry
guard, both observed callers pass 64/u8 filter. Helper 370EB0/145C10 remains
already classified executable support, not new asset. SHA256
d52e829c2ae2e3f8a519fc1fe28797a1005fcdc0e2afc81201e872024608b18b,
full reconstruction/disjoint from 76 owners 50490 spans; 8 windows 6 new 2 reused/
93 pins/96 guards exact/helper 0. Aligned pointer-word query: 1 root/38 successor,
none internal/origin-8 in that representation; 2 call sites verified. Static analysis
only, no build/Wine/game/fixture/old proof repeated; schema 1/e84-d83/77 files
76 resources unchanged. Next family model and Wine readers; managed/runtime/CLI/
ordinary use/Linux-USA/native Windows/new 32:9/full ABI/indirect callers/successor
extent/full TODO pending. No measurement.
[Contract](../rom_data_gameplay.md#european-conditional-eight-byte-record-source-contract).


**ROMDATA-02-CONDITIONAL-RECORDS8 EUR/Wine model/oracle completed
(2026-10-29).** Lossless core model 127 positions/1016 bytes/one span; strict parser,
shape/order separate from base ROM + 8-window authentication. Gate/filter/LE16 keys/
u8 operands preserved without invented enum/cap/null; select retains original pointers/
order/u32 zero-wrap capacity/full u32 filter and live call multiplicity, different
first/second polarity. One warning-free Windows core build c6f5406d.. from 554 sources/
239 core inputs; only new leaf + test + module, inverse hub byte exact, isolated core
test fingerprint. One Wine filter passes 24 guards and 109 original D56CE..D57A4
cycles with real CCE98/CCF00 readers: 16 base scenarios, 93 isolated synthetic variants
with keys 0/05FF/0600/0682/FFFF, operands 0..255, header 255, filter 128/255 vs full
context, capacity 0/1/64/FFFFFFFF. Python oracle and model agree with actual output/
count/capacity/positions/calls; 612 base bytes and 134 synthetic positions observed,
not all 1016 bytes read. EWRAM exact except expected output/IWRAM except 4-byte
return-link scratch/stack-r11r12/VRAM-OAM-palette/final ROM exact; 198372 instructions
coverage only. 120 guards 554 hashes/3 current hashes + 96 source guards exact/helper 0;
no game/checkpoint/navigation/old proof or reconstructed IWRAM helper. Helper entry/
epilogue/full ABI excluded; schema 1/e84-d83/76 resources/save/native/CAPI unchanged.
Next managed/runtime then CLI/ordinary use; Linux-USA/native Windows/new 32:9/indirect
calls/70B successor/full TODO pending. No performance or space measurement.
[Model and original loop](../rom_data_gameplay.md#typed-conditional-records-and-original-selection-loop-proof).


**ROMDATA-03-04-CONDITIONAL-RECORDS8-RUNTIME completed EUR/Wine with combined
proofs (2026-10-29).** e85-d84/index 76/conditional-records8.json: 78 files 77 resources,
76 previous exact/one span 1016B/no deps. Immutable owner 6 classes/127 slots/9 counters,
page 713 local + global and previous records4/raw successor composition; guest/live/
address logic and save/native/CAPI ABI unchanged. Four warning-free Windows builds from
568 consistent current sources; first never run after pre-Wine correction of test-only
iteration limit. First Wine filter passes publication/interruption 14 attach 2 full-set +
16 native full state-clock cases, then fails in the 17th for observer 607 vs 609. Diagnosis
of remaining case alone proves identical raw/typed/native state-clock and 609 actual bytes,
completes bus/lifecycle; final observer-only run passes 609 callbacks = 607 direct +
2 interp_one fallback, no unattributed bytes. Only test corrected; publication/attach/
16 cases/bus not repeated. 17 distinct cases: 14 loads 2 LE prefixes 1 scan; all 1016 raw
bytes altered AFTER Load/setup. Total 631 bytes served by native path = 629 direct
2 fallback and 5 records4 bytes, not distinct union or all compiled. All 1016 bus B/
6 classes 127 slots/127 boundaries/failed owner-full state/Load Undo same Arc and DMA3/
CpuSet 510 halfword/FastSet 256 word full state-clock pass. 261 guards 568 accepted
hashes + three 568 diagnostics/554 model preimages/7 current hashes/77 old 78 new files
exact; helper retires only interrupted copy. Accepted exe f60055e6.., first failure
retained; no game/checkpoint/navigation/previous proof or performance measurement.
Next real CLI and ordinary use; Linux-USA/native Windows/new 32:9/full ABI/indirect
calls/70B successor/full TODO pending.
[Consumption and combined evidence](../rom_data_gameplay.md#managed-conditional-records-and-typed-runtime-reads).


**ROMDATA-04-CONDITIONAL-RECORDS8-CLI registration/startup EUR/Wine completed
(2026-10-29).** One warning-free Windows production build 72d44985.. from 568 sources
byte-identical to runtime/no overlay, isolated corelib + prod fingerprints and canonical
hash mtime restored. Three Wine CLI children exit 0: create e85-d84/reuse/neutral
startup 1; 78 files exact to runtime/77 ID hashes loaded/80 managed paths no-write/
28859 old blocks. First prepare 0 succeeds; local helper checked index 75 instead of 76:
corrected and only two remaining children resumed, without repeating prepare or build.
Normal before-activation copy 24755→24756 with neutral input, all 9 new counters 0;
no action/navigation/new checkpoint/saved endpoint, no positive ordinary/full pair
inferred. 189 guards 568 CLI + 568 runtime hashes exact/helper 0, originals unchanged.
Source/model/native/lifecycle/old scenes remain closed. Next positive ordinary use and
full comparison; Linux-USA/native Windows/new 32:9/full ABI/indirect calls/70B
successor/full TODO pending, no measurement or assistant change to Android/package/
commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-conditional-record-cli-registration).


**ROMDATA-04-05-CONDITIONAL-RECORDS8-PUB completed for one EUR/Wine ordinary
case (2026-10-29).** Normal Rumors entry in pub; suffix 42f 15181→15223, 468 reads/
468 bytes SERVED, six classes 127/76/152/76/34/3 and 127 physical positions including
51 zero headers, not 127 active records/all fields/all branches/byte union. All 9
counters exact across capture/CLI 72d44985 and full pair 63584945; each of 42 state/
clock/classic-wide pixels/OAM/5 audio equal. One warning-free Windows build from
569 sources: CLI 568 + passive observer + module + optional 5-line guard; inverse
hub-validator byte exact/core-runtime unchanged. Two normal captures 254 frames:
B 180 negative then Up 3/A 12 positive 74, first read rel 42 + 32 continuation,
rolling rel 32→suffix 42. Seed Missions LIST 14969 already retained, no old 620/397
route replay; old PPM classic 240x160 color scale *255//31 authenticated against
FFSTATE, not recent wide format. Readonly projections and images verified, no Wine UI/
guest edit/cart write; final prompt HeardlatestRumors, list/selection/outcome not
inferred. 143 guards 569 hashes + 568 CLI/3 root hashes/80 managed paths no-write/
28859 cache exact, helper 0. CLI e85 generation hw22uM reused without prepare/build/
navigation; first B 0 case remains closed. Declared source/model/runtime/CLI/ordinary
8B closed; next new root 713B3C/stride 70 requires its own contract. Linux-USA/native
Windows/new 32:9/full ABI/indirect calls/full TODO pending; no measurement or assistant
change to Android/package/commit/push.
[Equivalence](../rom_data_gameplay.md#ordinary-conditional-record-rumors-entry-equivalence).


**ROMDATA-01-MISSION-RECORDS70 EUR source contract completed
(2026-10-29).** Catalog 713B3C..71C73C/512 records x 70/35840B, two independent
scans 1..511 plus index 0 getter/successor boundary with own readers; separate constant
300, no new guest cap 512. Getter 65 selectors low16 index/low8 selector: unsigned LE
from byte, packed bits, flags, zero or +0177 mapping and 200 scale, 7 address selectors/
6 inline offsets, aliases 19/20 and views 24/32 with shared scalars. Remaining bytes/
raw values/identity order retained. Already closed mission link reused: field +45 HEX
69 decimal event row *12+71C760; 512 references/173 values/four alias groups, zero
depends on caller; distinct event catalog still to classify. All 35840B reconstructed,
SHA256 bbdbc955b9b3f7ed678123aa6bea8b6098f25f6a13a4ab94de669d1e103253dc,
disjoint from 77 owners 50491 spans/e85-d84/78 files unchanged. 19 windows 9 new
10 cache/157 pins 110 guards exact; old text authenticated opcode-by-opcode against
ROM and current hash, no false historical text guard. 38 base literals/2 origin+70/
2 internal candidates in data not classified readers, aligned query not exhaustive
for indirect calls. Static only, no build/Wine/game/fixture/navigation/previous proof
repeated; helper retires zero paths. Next strict lossless model/Wine getter oracle,
then managed/runtime/CLI/ordinary. Full ABI/unknown field semantics/view counts/
successors/Linux-USA/native Windows/new 32:9/full TODO pending. No performance or
space measurement; parallel menu/content/DBG/package work preserved.
[Contract](../rom_data_gameplay.md#european-mission-record-source-contract).


**ROMDATA-02-MISSION-RECORDS70 EUR/Wine model/oracle completed
(2026-10-29).** New lossless core mission_records70 512 records/35840B/one span,
strict shape/order/JSON separate from exact ROM + 19-window authentication. Remaining
bytes/inline arrays/raw operands and all 65 selectors retained; low16 index/low8
selector query, Some 0 invalid without lookup, addresses calculated even outside owner,
unowned scalar None diagnostic without new guest cap. One warning-free Windows core
build 663e8b87.. from 571 sources/242 core inputs, 569 preimages exact and inverse hub
byte exact; only two leaves/public module, no parallel frontend. One Wine filter passes
21 guards and 698 calls of original D1F30 getter to return with real CEEEC helper:
438 base/all 65 at indices 0/1/58/299/300/511/normalization/default/views outside
owner/scalars 512-65535, 260 isolated synthetic variants on four 70B patterns at record
511. Independent Python/model/results and actual LDRB read order exact; 324 distinct
base bytes/54 synthetic versus 35840B separately reconstructed, views do not dereference
16B inline. EWRAM/VRAM/OAM/palette unchanged, IWRAM exact including expected return
links, SP/R4..R12/Thumb/final ROM exact; 17213 instructions coverage only. 139 guards
571 hashes/3 current 569 preimages + 110 source guards/helper 0. No game/checkpoint/
cart write/navigation/already closed proof or measurement. Schema 1/e85-d84/77 resources
78 files/save/native/CAPI unchanged. Next managed/runtime/native clock/lifecycle then
CLI/ordinary; full ABI/unknown fields/indirect calls/successors/Linux-USA/native Windows/
new 32:9/full TODO pending. Parallel work preserved.
[Model and getter](../rom_data_gameplay.md#typed-mission-records-and-original-getter-proof).


**ROMDATA-03-04-MISSION-RECORDS70-RUNTIME completed EUR/Wine with combined
evidence (2026-10-29).** e86-d85/index 77/mission-records70.json/79 files 78 owners,
77 previous exact/one span 35840B/no deps; immutable owner 10 classes 13 counters/
8 mask words 512 positions/pages 713..71c, previous 8B and raw successors composition.
Two warning-free Windows builds 573 sources/243 core inputs: 73326ad9 first publication/
prior 77/interruption PASS then first attach FAIL historic OBJ cap 1024, other tests
not started. With request for no space limits removed attach caps OBJ 1024/BG 64KiB
and three allocation assertions, parse bounds/source auth/binding unchanged. 455bdb1c
second build/Wine filter ONLY remaining on SAME 79-file generation: 14 attach 2 full-set/
failed owner full state/41 native state-clock (16 loads 25 getters) with 35k raw altered
AFTER load/setup/all 35k bus 10 classes 512 slots/512 edges/Load Undo same Arc/DMA3 pass.
Publication not repeated, first failure retained; no cap fix interpreted as optimization
or RSS measurement. All callbacks including interp_one attributed: 52 new bytes served
5 previous/fallback delta 0, not distinct union/all catalog native; 3 stop-safe families/
168 entries 69 dispatch coverage only. DMA/CpuSet 17922 halfword/FastSet 8968 word full
state-clock identical. 860 guards 573 accepted + 573 diagnostics/571 model/8 current
hashes/78 old 79 new files exact; helper retires only interrupted copy. close-final.py
PASS-combined-scopes, old close-runtime.py not run. Source/model/old scenes not repeated,
no game/checkpoint/navigation/cart write/changed Machine-save-native-CAPI ABI. Next real
CLI then ordinary; Linux-USA/native Windows/new 32:9/full ABI/indirect calls/successors/
full TODO pending. No performance/space measurement, parallel work excluded from build
and preserved.
[Consumption and evidence](../rom_data_gameplay.md#managed-mission-records-and-typed-runtime-reads).


**ROMDATA-04-MISSION-RECORDS70-CLI registration/startup EUR/Wine completed
(2026-10-29).** One warning-free Windows production build 738b7f20.. from 573 sources
byte-identical to runtime/no overlay, consistent corelib-prod-menu fingerprints and
canonical hash mtime restored. Three Wine children all 0: prepare created/reuse/neutral
startup 1, 79 files exact to runtime/78 ID hashes/81 managed paths no-write/28859 old
native blocks. Private generator stopped on token-index assertion before any Wine: only
helper generation corrected with verified index 77, no source copy/build/CLI preparation
repeated. Normal copy 24755→24756 all 13 counters 0; no action/navigation/new checkpoint/
saved endpoint, positive ordinary full pair or new negative locator phase inferred.
195 guards 573 CLI + 573 runtime/79 files/originals/native cache/loader/canonical exact;
helper 0. Source/model/native/lifecycle/old proofs remain closed. Next positive ordinary
and full comparison; Linux-USA/native Windows/new 32:9/full ABI/indirect calls/successors/
full TODO pending, no measurement/Android/package/assistant commit/push.
[CLI and limits](../rom_data_gameplay.md#actual-mission-record-cli-registration).


**ROMDATA-04-05-MISSION-RECORDS70-MISSIONS completed for one EUR/Wine ordinary
case (2026-10-29).** Normal opening of Giza Plains information, suffix 33 frames
15017→15050: 5 reads/5 bytes SERVED only packed class/one physical position, other
9 classes 0. All 13 counters exact across capture/CLI 738b7f20/full pair 3d21e89c;
each of 33 full state-clock/classic-wide pixels/OAM/5 audio equal, both arms same old
native cache/not global interpreter-native parity. One warning-free Windows build
574 sources = CLI 573 + passive observer/public module/optional 5-line guard, inverse
hub byte exact/core-runtime unchanged. ONE capture 81f from already retained Missions
LIST 14969: Down 3 at 8/A 12 at 40, first read rel 49/guest 15019, rolling 48→suffix
33 A 4 + neutral 29; search STOP. Wine test child 0 PASS; post-capture helper KeyError
steps after complete output repaired only recording, no capture/input/Wine/build repeated.
Already authenticated classic seed 240x160/scale reused; new wide PPM 284x160/bitrep
verified against full FFSTATE, classic PNG shows Giza MISSION info page 1/3 Rank 1. No
mission accepted/battle/outcome/all fields/all 512/union of 5 distinct bytes inferred.
133 guards 574 hashes + 573 CLI/3 current/78 ID hashes 81 managed paths no-write 28859
cache exact, helper 0/originals unchanged. Old 620/397 route/source-model-runtime/
registration not repeated, no forced ID/RAM/cart write. Declared 70B chain closed;
new C73C/C740 arrays and C760 event require own contracts. Linux-USA/native Windows/
new 32:9/full ABI/indirect calls/other ordinary classes/full TODO pending; no performance/
space measurement/Android/package/assistant commit/push.
[Equivalence](../rom_data_gameplay.md#ordinary-mission-information-equivalence).


**ROMDATA-01-SELECTION-BYTE-TABLES EUR source contract completed
(2026-10-29).** C73C..C740 four count thresholds and C740..C75E thirty ordinal
weights: 34 bytes/two spans, SHA256
d758af5f1a07613f1c2047257d42ef9cbfac83b63783ee6fd0e7b3879370b8dd. Five live
headers 02002E34+6*i counted only if low 3 bits nonzero, all examined; guard <=3 before
u8 threshold load, 4/5 exit. Second independent root, two stride 1 loops ordinals 1..30/
positions 0..29. Real helper 377BC reads raw live u8 02002CC3+12*(low8 ordinal-1),
no global 0/30 guard or Boolean normalization; caller tests low8 nonzero. Sum 0 exits
before second stage; full32 helper rank minus unsigned weight with wrap and signed
comparison, first negative selects, 31 exhausted exits; no invented negative guard/
zero-weight/cached eligibility. Numeric helper 2868/146314/RNG/distribution/full ABI
not modeled. All 34B reconstructed/positions retained/repeated values not deduplicated/
no data pointers; disjoint from 78 owners 50492 spans/e86-d85/79 files unchanged.
6 windows 2 new 4 cache/95 pins 96 guards/helper 0, only 2 new count context and live
helper; no build/Wine/game/fixture/navigation/old proof repeated or measurement. Gap
C75E..C760 two opaque B unowned, next 12B root C760 independent/extent pending. Neutral
ID gameplay.selection-byte-tables. Next model/Wine oracle then managed/runtime/CLI/
ordinary; Linux-USA/native Windows/new 32:9/indirect calls/full TODO pending.
[Contract](../rom_data_gameplay.md#european-selection-byte-table-source-contract).


**ROMDATA-02-SELECTION-BYTE-TABLES EUR/Wine model/stages completed
(2026-10-29).** Strict model 4+30 u8/34B/two spans, shape separate from exact ROM +
six windows; gap 2B excluded/repeated values distinct. One warning-free Windows core
build b40503b8.. 576 files 245 inputs/574 preimages; one Wine filter 18 guards/45 base +
24 synthetic executes D323C count/D327E sum-zero/D32B4 choice with real 377BC. Python/
model/results/live query order and LDRB exact; wrap32/signed rank including extremes,
no weight 0 shortcut or new cap. All 34B base/32 synthetic observed, 34 reconstruction
separate, all RAM/SP-R8..R12/final ROM exact; 30480 instructions coverage only. 123 guards
576 hashes/3 current/574 preimages/96 source guards PASS, helper 0. Schema 1/e86-d85/
79 files unchanged; managed runtime/bus/native clock/Load Undo/DMA/CLI/ordinary next,
numeric helpers/RNG/full ABI/Linux/USA/native Windows/new 32:9/global TODO pending. No
ordinary fixture or closed proof repeated/metrics.
[Model and proof](../rom_data_gameplay.md#typed-selection-byte-tables-and-original-stage-proof).


**ROMDATA-03-04-SELECTION-BYTE-TABLES-RUNTIME completed EUR/Wine
(2026-10-29).** e87-d86/index 78/selection-byte-tables.json/80 files 79 resources,
two spans 4/30B/no deps, all 78 previous records/files identical/schema 1-USA e8
unchanged. Immutable page 71C owner composes prior 70B and raw gap/event root;
6 counters/4+30 distinct positions, no allocation cap or ABI change. One warning-free
Windows build e5b45668.. 578 files 246 core/576 model, one Wine filter publication/
interruption/14 attach 2 full-set/20 native full state-clock/all 34 bus/Load Undo/DMA3
PASS. 12 loads + 8 stages/real 377BC, raw 34 + prior 4 altered only after Load;
76 new B served + 5 prior all attributed to callbacks, fallback source 0, 91 entries
119 dispatch coverage only. Width/mirror/edges/prior/raw, all 4+30 positions and
DMA-CpuSet 20 halfword/FastSet 16 word exact state-clock. 243 guards 578 hashes 576
model/7 current/79 prior exact; helper retires only successful interrupted copy. Real
CLI/ordinary next; numeric helpers/RNG/full ABI/gap-event extent/Linux/USA/native
Windows/new 32:9/global TODO pending, no old proof or fixture repeated/metrics.
[Consumption](../rom_data_gameplay.md#managed-selection-byte-tables-and-typed-runtime-reads).


**ROMDATA-04-SELECTION-BYTE-TABLES-CLI EUR/Wine registration-startup completed
(2026-10-29).** One warning-free production build 22af4132.. same 578 runtime sources/
no overlay/canonical hash mtime restored; one Wine orchestration three commands create-
reuse-neutral 1 exit 0. 80 identical dataset files/79 ID hashes/82 paths hash mtime
unchanged/cache 28859 unchanged. e87-d86/index 78/two spans 34B/no deps verified;
24755→24756 all 6 usage 0, no ordinary acceptance. 192 guards 578 CLI + 578 runtime/
ROM-native cache-TinyCC-original seed-canonical exact/helper 0. No navigation/new
checkpoint/old proof/metrics. Positive ordinary and full pair next; Linux-USA-native
Windows/new 32:9/full ABI/RNG/indirect calls/gap-event/global TODO pending.
[CLI](../rom_data_gameplay.md#actual-selection-byte-cli-registration).


**ROMDATA-04-SELECTION-BYTE-TABLES-ORDINARY completed one EUR/Wine case
(2026-10-29).** Eight new windows/354 direct opcode pins D3724→D3228/low8 argument
nonzero guard; wrapper D6970 passes 1, other 2 callers 0 skip, indirect unknown. Four
readonly seeds guard 6/count 1-2; normal Giza 19049/cached UI selected. One 579-source
8f3bee97 build observer 6 stats + 5-line guard/no core-runtime changes/inverse hub exact.
ONE capture A 12/129f 19049→19178/first 97-19147/rolling 96→suffix 33-19145→19178.
SAME 22af CLI/e87 ZloxMx positive 33 and SAME 8f3bee97 full pair 33 all complete
states-clock/classic-wide/OAM/5 audio PASS; 6 exact counts 1 bus 1 SERVED threshold B
1 position/weights 0, not all 4/30 positions or event/mission outcome. Discovery STOP
after one. Full wide 284x160 bitrep authenticated/classic map calendar PNG inspected.
140 guards 579 hashes 578 CLI/3 current/82 managed/native cache-ROM-seed-canonical
exact/helper 0. Dispatch 203258/519441 and audio [72418,72418,8712,8712,72342] coverage
only; same cache in both arms/not global interpreter-native parity. No closed proof or
fixture recreated/metrics. Source-model-runtime-CLI/one ordinary chain closed; event
12B root C760/alias 71D1EC/next 71D360 requires own contract. Linux-USA-native Windows/
new 32:9/full ABI/RNG/indirect calls/gap-event/global TODO pending.
[Ordinary use](../rom_data_gameplay.md#ordinary-selection-threshold-travel-equivalence).


**ROMDATA-01-EVENT-RECORDS12 EUR source contract completed (2026-10-29).**
71C760..71D360/256x12/3072B/one span SHA e7454120.. disjoint from 79 owners
50494 spans; backscan 255..1/stride 12 + live u8 zero + independent next root stride
27 prove extent, not fit-only/global cap. Layout u8 header 00-key 01/u16 primary 02-
alternate 04/u8 param 06-byte 07/u16 flags 08/opaque 0A 2B, all 3072 roundtrip/no
invented enums-FK caps-zero padding. 71D1EC=base+225x12/31 physical positions
INTERNAL view/same owner, helper 9D04 low8 arg not capped at 31/can leave owner;
ordinal choice 1..30→rows 226..255. Callback 127388 full u16 key/vs byte 01/highest
match 255..1/zero excluded/8 key groups 1 record group. Alt 04 nonzero→alias primary
02 + original alt 04, zero→direct primary/alt 0; key 4A live special/param 06 next
27 root/live effects original. 15 windows 10 new 5 cached/67 pins/112 guards/80
accepted files PASS/helper 0; census 36 base 2 alias 25 next and not all indirect.
Static only/no build-Wine-game-fixture-navigation-old proof replay; schema 1/e87-d86
unchanged. Model/original Wine prefixes next, then managed/runtime/CLI/ordinary;
full ABI/live effects/indirect calls/gap/next 27 extent/Linux-USA-native Windows/
new 32:9/global TODO pending/no metrics.
[Contract](../rom_data_gameplay.md#european-event-record-source-contract).


**ROMDATA-02-EVENT-RECORDS12 EUR/Wine model/prefixes completed (2026-10-30).**
Strict 256x12/3072B/one span/raw flags/opaque 2B, shape order distinct from exact
ROM + 15 windows. Physical None outside owner vs u16 encoded view/uncapped low8 alias,
direct/alias-original alt pair and full16 script highest 255..1/row 0 excluded. One
warning-free core build a4b88a69.. 581 files 248 core/579 preimages/inverse hub exact,
one Wine filter 20 guards 51 base 37 synthetic prefixes 9D04 pair/127388 scan/seven
unsigned loads/u16 view PASS; Python-model-results/LDR order exact. 302 base B 268
synthetic B observed vs separate 3072 roundtrip; RAM exact with expected index store/
push, SP-R8..R12/final ROM verified, 9959 instructions coverage only. 133 guards
581 hashes/3 current/579 preimages 112 source exact/helper 0. Schema 1/e87-d86/80 files
unchanged; managed/native clock/lifecycle/CLI/ordinary next. Full ABI/live effects/
indirect calls/gap/next 27 extent/Linux-USA-native Windows/new 32:9/global TODO pending.
No old proof/ordinary fixture/navigation repeated/metrics.
[Model](../rom_data_gameplay.md#typed-event-records-and-original-prefix-proof).


**ROMDATA-03-04-EVENT-RECORDS12-RUNTIME completed EUR/Wine (2026-10-30).**
e88-d87/index 79/event-records12.json/81 files 80 resources/79 prior exact, 3072B/
one span/no deps/alias row 225 SAME owner; pages 71c-71d/8 classes 11 stats/4 mask
words 256 positions, prior selection 2B + raw gap 2/next root composed, no allocation
cap/guest ABI change. One warning-free build 9225a3bc.. 583 sources 249 core 581 model,
one Wine filter publication-interruption/14 attach 2 full-set/22 native full state-clock/
all 3072 bus 8 classes 256 records 256 borders/Load Undo/DMA3 PASS. 14 loads + 8 pair-
scan-u16 view/4 stop-safe families; raw 3072 + prior 2 altered after Load, 296 SERVED
new B 3 prior all attributed, 2 new B interp_one fallback (not 296 direct native or
distinct union)/51 entries 30 dispatch coverage only. DMA-CpuSet 1538 halfword/FastSet
776 word exact state-clock. 253 guards 583 hashes 581 model/seven current 80 prior exact;
helper only successful interrupted. Actual CLI/ordinary next, full ABI-live effects-
indirect calls-gap-next 27/Linux-USA-native Windows/new 32:9/full TODO pending; no old
proof/fixture/metrics.
[Consumption](../rom_data_gameplay.md#managed-event-records-and-typed-runtime-reads).


**ROMDATA-04-EVENT-RECORDS12-CLI EUR/Wine registration-startup completed
(2026-10-30).** One warning-free production build b9a60501.. SAME 583 runtime sources/
no overlays/canonical hash mtime restored; one Wine orchestration create-reuse-neutral 1
three exit 0. 81 identical payloads 80 ID hashes 83 no-write paths/cache 28859 exact;
e88-d87/index 79/one 3072 span/no deps. 24755→24756 all 11 counts 0/compatibility only,
not positive ordinary. 194 guards 583 CLI + 583 runtime/81 accepted/ROM-native cache-
TinyCC-seed-canonical exact/helper 0. Positive ordinary/full pair next; Linux-USA-native
Windows/new 32:9/full ABI-live effects/indirect calls/gap-next 27/full TODO pending.
No source-model-native/old proof/fixture-navigation replay or metrics.
[CLI](../rom_data_gameplay.md#actual-event-record-cli-registration).


**ROMDATA-04-EVENT-RECORDS12-ORDINARY completed one EUR/Wine case (2026-10-30).**
Original normal Giza White Flowers 19409 seed/cached UI + producer witness reused,
not old 700f route proof. One 584-source 0b6581cb build observer 11 stats + 5-line
guard/inverse hub-validator exact/core-runtime unchanged. ONE A 12 capture 34f
19409→19443/first rel 2-19412/rolling 0 suffix 34; discovery STOP. SAME b9a605 CLI/
e88 generation 3iczRx actual child 0/34f, collector TypeError for OLD usage ID fixed
OFFLINE log registration only/no CLI replay. SAME 0b6581cb full pair 34 all complete
states-clock/classic-wide/OAM/5 audio PASS. 11 counts exact 4 bus 6 SERVED B/key 2-
primary 2-alternate 2/one physical record/other 5 classes 0; not distinct union/all
8-all 256/alias branch/outcome. Full wide 284x160 bitrep authenticated/classic Giza
map mission entry inspected. 126 guards 584 hashes 583 CLI/3 current/83 no-write paths/
80 ID hashes/cache 28859-ROM-seed-canonical exact/helper 0; dispatch 128240/218562/
audio [74612,74612,8976,8976,74488] coverage only. Event source-model-runtime-CLI/
one ordinary chain closed; next 27 root 71D360 contract extent/fields pending,
Linux-USA-native Windows/new 32:9/full ABI-live effects/indirect calls/gap/full TODO
open/no metrics.
[Ordinary use](../rom_data_gameplay.md#ordinary-event-record-mission-entry-equivalence).


**ROMDATA-01-PARAMETER-RECORDS27 completed for observed EUR fragments
(2026-10-30); total extent still open.** The 256 event records prove 59 indices
0..50 and 115..122, stride 27/root 71D360: two ranges 71D360..71D8C1 and
71DF81..71E059/1593B, roundtrip SHA 40d4e406.., 256 links and no overlap with
80 owners/50495 spans e88-d87. 27 windows 23 new 4 reused/70 pins/125 guards,
all 25 root literals covered in cached census. Shared packed fields 01..06,
u16 LE 07-0A-12, explicit i8 0C-15, byte 09 and flags 0D, separate operands
0E0F-1011, opaque 14+16..1A retained. No enum/pointer/padding inferred from values.
Live +A8 readers use u8 without 127/128/59 limit; next root 71E0E0 stride 38 does
not prove 128 records by distance alone. 128 hypothesis rejected; gaps/further indices
raw and unowned. Static only, no build/Wine/input/fixture/old proof repeated or metric.
Wine model/proof, runtime, CLI and ordinary to do; global extent/full ABI/live effects/
indirect calls/Linux-USA-native Windows/new 32:9/full TODO open. Event 12 chain closed.
[Contract](../rom_data_gameplay.md#observed-european-parameter-records-with-27-byte-stride).


**ROMDATA-02-PARAMETER-RECORDS27 completed for 59 observed EUR records
(2026-10-30).** Strict model with physical indices 0..50/115..122, u8/packed fields,
u16 LE, i8/opaque, two spans 1593B; modification validation separate from source auth.
record/byte_at do not respond in gaps or for high indices; encoded_view retains original
low8 without 59/128 limit. One 586-source/251-core build 49c72f02 warning-free, one
Wine filter 25 guards 73 base 48 synthetic: 12 classes of original prefixes, exact
results and read order, signed 127/-128/low16 encoding included. All RAM/SP-R8..R12/
final ROM exact; 95 base B 20 synthetic B distinct observed vs 1593B roundtrip.
739 instructions coverage only. 143 guards/584 preimages/125 source guards/inverse
hub retained/helper 0. No old proof or fixture repeated, nor metrics. Managed runtime
e89-d88/CLI/ordinary to do; global extent/full ABI/helper/live effects/indirect calls/
Linux-USA-native Windows/new 32:9/full TODO open. e88-d87 unchanged.
[Model](../rom_data_gameplay.md#sparse-parameter-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-PARAMETER-RECORDS27-RUNTIME EUR/Wine completed for 59 observed
records (2026-10-30).** e89-d88/index 80/parameter-records27.json/82 files 81 resources,
80 previous exact/no deps/two spans 1593B total; sparse owner 71D..71E composes previous
event and raw gaps, 12 classes 15 stats/four physical masks. No new cap/ABI/save/native
cache change. First 588 build 51e19ef0 passes publication, interruption, 14 attach
2 full-set/17 native full state-clock; test oracle expects 2 previous bytes instead of
4 aligned bytes served, typed/native/API agree on 4. Only test leaf corrected: second
588 build 278d6d23 warning-free/one Wine run only remaining 16 native + bus/lifecycle/
DMA3 PASS, same 82 files 13 objects hash mtime exact; no publication/attach/first 17
cases repeated. Total 33 full state-clock/20 loads 13 prefixes/13 families, 37 SERVED
new B 5 event; not distinct union or all direct native, fallback 0 only for remaining 16/
previous 17 not reported. All 1593B, 12 classes 59 records/59 borders/raw gap-next root/
Load Undo, DMA-CpuSet 1664 halfword/FastSet 832 word exact. 291 guards 588 accepted +
588 diagnostic/252 core/586 model/7 current 81 prior retained; helper only interrupted.
Collector task label corrected offline without build/test replay. accepted-selection.json
authoritative, selection.json diagnostic. Actual CLI/ordinary/global extent/full ABI/
helper-live effects/indirect calls/Linux-USA-native Windows/new 32:9/full TODO open.
[Runtime](../rom_data_gameplay.md#managed-sparse-parameter-records-and-typed-runtime-reads).


**ROMDATA-04-PARAMETER-RECORDS27-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 588-source production bd99ac5b warning-free, SAME corrected runtime/
no overlays/canonical hash mtime finally restored. One Wine orchestration: prepare creates
e89-d88, prepare reuses, runc copied normal checkpoint 24755 neutral 1→24756; three
exit 0/82 runtime-exact payloads/81 ID hashes/84 managed paths hash mtime unchanged/
28859 native cache. Index 80/two spans 1593B total/no deps correct. All 15 counts 0,
compatibility/registration only, not positive ordinary use or equivalence. 197 guards
588 CLI + 588 runtime/ROM-seed-cache-TinyCC-canonical exact/helper 0. No new navigation/
checkpoint or old tests or metrics repeated. Positive ordinary/full pair, global extent/
full ABI/helper-live effects/indirect calls/Linux-USA-native Windows/new 32:9/full TODO open.
[CLI](../rom_data_gameplay.md#actual-sparse-parameter-record-cli-registration).


**ROMDATA-04-PARAMETER-RECORDS27-ORDINARY partial (2026-10-30), positive still to
obtain.** Passive observer 15 stats + module/optional 5-line guard; 589-source 8f0c02b7
warning-free/core-runtime 588 CLI unchanged/inverse preimages exact. Two normal Wine
captures on copies: R 12 GizaInfo 15050→15230/180f page 1/3 unchanged; A 12 at 1/161
WhiteFlowers dialogue 21889→22129/240f. All 15 counts 0: NEGATIVE bounds closed, no
positive suffix/positive CLI/full pair executed. 9 static windows/7 readonly snapshot
gates; accepted script directory/archive/events join identifies opcode 80 mode 2 in
programs 53/65/74/93/95, keys 56/68/77/96/98, events 216..220/params 116..120.
WhiteFlowers 58 param 0/program 163; stop further blind attempts in page or intro,
indirect calls/mode 0 override 0 not excluded. 130 guards 589 + 588 hashes/3 current
84 managed/originals retained, diagnostics kept/no full cleanup. Reuse SAME observer
and bd99 CLI/e89 dataset when relevant scene known; positive/full equivalence/global
extent/Linux-USA-native Windows/new 32:9/full TODO open/no metrics. Update to same unit:
eleven total normal captures, all 15 stats 0, no rebuild. New contexts SprohmInfo 180/
B 32/Nubswood cursor/travel 360 and CyrilPub 240+120; no mission selection inferred
from name or RNG resampling. Six authenticated snapshot gates do not prove liveness.
One neutral 1 Wine inspection/already compiled CLI/layout off exit 0 does not save/adopt
state; readonly framebuffer/UI 76800-pixel projections show only Gil, no readable choice.
233 guards 589 + 588 hashes/3 current 84 managed exact, first receipts retained. Blind
navigation stopped; positive ordinary still open, independent source contracts may
continue without repeating these captures.
[Status and evidence](../rom_data_gameplay.md#ordinary-sparse-parameter-record-reachability-remains-partial).


**ROMDATA-01-WORLD-RECORDS38 completed for the observed EUR catalog
(2026-10-30).** Root 71E0E0/31x38/1178B up to 71E57A,
SHA cd2b33e95a5b46a53703f8a32f89a6f1c0eea4d1b562dfd86ee7c4836084d483.
All 512 missions project exactly indices 0..30; interior root 71E106 and loop 6..29
use SAME owner/records 7..30, next 71E57C reads u32 pointer/unowned gap 2B. No
count-by-distance or cap 31: D347C low8 and D4C4C full R2 u32 retain original
addresses. Byte 01/u16 LE 02 with separate byte 02/u16 LE 04 with low10 copy/
31 thresholds 07 + count clamped 0..30/opaque 00-06 retained. 12 windows 10 new
2 cached/58 pins 107 guards, 1178 roundtrip 512 links/no overlap 81 owners
50497 spans/e89-d88/82 files exact. Static only/no build-Wine-input-fixture/old
evidence or metrics. Wine model/oracle/runtime/CLI/ordinary to do; helper-RNG/
full ABI/live effects/indirect calls/opaque/Linux-USA-native Windows/new 32:9/
full TODO open. Sparse 27 ordinary remains partial, navigation not reopened.
[Contract](../rom_data_gameplay.md#observed-european-world-records-with-38-byte-stride).


**ROMDATA-02-WORLD-RECORDS38 completed EUR/Wine (2026-10-30).** Strict model
31 records/1178B/one span, opaque and all bits retained; editable shape separate
from exact EUR auth/12 window hashes. Physical record/byte_at does not own gap 2B
or indices 31+, original encoded low8/full u32/interior row 1 without cap 31.
Projections byte 01 divisor + post-helper wrap/zero 1, byte 02/u16 LE 02, full
u16/low10 value 04, live count clamp 0..30/signed threshold comparison with raw
255. One 590-source/254-core build ed4d65c8 warning-free, one Wine filter
19 guards 41 base 19 synthetic original PASS; 23 base B 6 synthetic B distinct
observed vs 1178 roundtrip/353 instructions coverage only. Read order/results/
all RAM + expected destinations/SP-R8..R12/final ROM exact. 26 unit guards
107 source/588 preimages/3 current/inverse hub exact, helper 0. No external
helpers-RNG/full ABI/native clock/ordinary claim; no old proof/navigation/fixture
repeated or metric. Schema 1/e89-d88/82 files unchanged. Managed runtime e90-d89/
CLI/ordinary and Linux-USA-native Windows/new 32:9/full TODO to do.
[Model](../rom_data_gameplay.md#world-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-WORLD-RECORDS38-RUNTIME completed EUR/Wine (2026-10-30).**
e90-d89/index 81/world-records38.json/83 files 82 resources/prior 81 exact/no deps/
one span 1178B. Immutable owner 71E composes sparse 27/raw gap/next root, 5 classes
8 stats/one 31 mask without new cap or ABI. One 592 build 271dceba warning-free;
collector guard-label corrected offline on SAME exe, no rebuild. One Wine filter
passes publication/interruption/14 attach 2 full-set/27 native full state-clock
(17 loads, 10 prefixes/10 stop-safe families), 23 SERVED new B 5 prior/fallback
[0, 0], not distinct union/all direct native. All 1178B/5 classes 31 records/
31 borders/mirrors-widths/prior + raw gap-next root/Load slot Undo exact; DMA-CpuSet
704 halfword/FastSet 352 word full state-clock. 269 guards 592 frozen/255 core/
590 model/7 current 82 prior retained, current observer 27 excluded ONLY from frozen
hub. Helper only interrupted copy. Actual CLI/ordinary/full ABI/helper-RNG/live
effects/opaque-indirect/Linux-USA-native Windows/new 32:9/full TODO open. No old
proof/navigation/fixture or metrics repeated.
[Runtime](../rom_data_gameplay.md#managed-world-records-and-typed-runtime-reads).


**ROMDATA-04-WORLD-RECORDS38-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 592-source production 6d95f705 warning-free, SAME runtime/no
overlays/canonical hash mtime restored. One Wine orchestration/three exit 0: prepare
creates e90-d89, prepare reuses, runc copied normal checkpoint 24755 neutral 1→24756.
83 runtime-exact files/82 ID hashes/85 managed paths hash mtime unchanged/cache
28859 exact; index 81/no deps/one span 1178B. Eight counts 0, registration/compatibility
only, not positive ordinary/equivalence. 201 guards 592 CLI + 592 runtime/ROM-seed-
cache-TinyCC-canonical exact/helper 0. No new navigation/fixture/old proof or metric
repeated. Positive ordinary/full pair/full ABI/helper-RNG/live effects/opaque-indirect/
Linux-USA-native Windows/new 32:9/full TODO open.
[CLI](../rom_data_gameplay.md#actual-world-record-cli-registration).


**ROMDATA-04-WORLD-RECORDS38-ORDINARY completed for one EUR/Wine case
(2026-10-30).** One 593-source ee939288 warning-free/passive 8 stats + 5-line guard,
core-runtime 592 CLI unchanged/inverse hubs exact/current 27 observer retained. First
A 12 Nubswood 360f 3448→3808 NEGATIVE closed; readonly CCF00/path 0..6/counter 32→30
explains branch not ready. Four authenticated EUR banks: IN08 physical 13 counter 1,
normal Down 48/96f 3020→3116 ends actual node 18; A 12 single positive capture 189f
3116→3305, first source 157/3274, rolling 45 suffix 3260→3305/discovery STOP. SAME
6d95 CLI one 45/SAME 593 test one full pair 45: each state-clock/classic-wide/OAM/
5 audio exact, 8 identical stats 2 bus 2 SERVED B = 1 threshold + 1 byte 01/one
physical record, other 3 classes 0/not all 5-31. 82 ID hashes 85 managed/cache 28859
no-write, authenticated framebuffer projection/world travel MadmoonDay4, no mission/
arrival inferred. 165 guards/593 + 592 sources/3 current/inverse/originals exact/helper 0.
Normal inputs on copies only, no reset-RNG-RAM-ID/route recreation, no old proof or
metric. Other ordinary/full ABI/helper-RNG/live effects/opaque-indirect/Linux-USA-native
Windows/new 32:9/full TODO open; sparse 27 ordinary remains partial.
[Case and limits](../rom_data_gameplay.md#ordinary-world-record-counter-expiration-equivalence).


**ROMDATA-01-INDEXED-GRAPHICS-ARCHIVES completed for the EUR source contract
(2026-10-30).** Four pointer groups 71E57C/71E61C/71E630/71E658, five language
slots each, 20 archives 649 programs; 24 disjoint spans 74907B source (74827 archives
+ 80 pointer), 278336B output/15631B dictionary prefixes. Codec 05228/0537C with BE
index-offset and fixed external dictionary, distinct 054A0; opaque 4137 is not a
discriminator, raw u8 language without fallback and original low16 start-count. All
source/output bytes reconstructed exactly, 22 windows 20 new 2 cached/101 pins/10 root
literals/12 merged BL/121 guards/no overlap 82 owners 50498 spans/83 accepted files
exact. Other 16 nearby groups/code-dictionary tails/context 3710D4/global extent not
classified. No build-Wine-game-fixture/old proof/metric repeated; schema 1/e90-d89
unchanged. Model/Wine oracle/runtime/CLI/ordinary/full ABI/indirect/Linux-USA-native
Windows/new 32:9/full TODO open; sparse 27 ordinary partial.
[Contract and limits](../rom_data_indexed_graphics.md).


**ROMDATA-02-INDEXED-GRAPHICS-ARCHIVES completed EUR/Wine (2026-10-30).** Strict
model 20 archives/four five-slot groups/649 programs: opaque header, BE index and
offset/literal-dictionary packets/dictionary prefixes/LE words exact, editable shape
separate from exact EUR auth. Host mode 0 width 5 only, no new guest cap; raw language
encoded view remains original for 5/255, low16 start-count/zero-oversize-clamp/invalid
start before init retained. One 595-source/257-core build d78a74d9 warning-free from
593 ordinary + 2 leaves/one public module; one Wine filter 37 guards 20 whole archive
calls + 9 boundary + 9 synthetic + 28 raw selector prefixes PASS. All 74907B/24 spans/
278336 decoded B reconstructed; output/status/globals/SP-R4..R11 and RAM outside 68B
original stack/VRAM-OAM-palette/final ROM exact. 5993129 instructions coverage only.
26 unit guards 121 source/593 preimages/3 current/inverse hub exact. No closed proof/
checkpoint/navigation/performance replay; schema 1/e90-d89 unchanged. Managed runtime/
native clock/CLI/ordinary/mode 3-wider width/full ABI-live uploads/indirect/Linux-USA-
native Windows/new 32:9/full TODO open.
[Model and proof](../rom_data_indexed_graphics.md#typed-model-and-original-wrapper-proof).


**ROMDATA-03-04-INDEXED-GRAPHICS-RUNTIME completed EUR/Wine (2026-10-30).**
e91-d90/index 82/graphics.indexed-archives/indexed-graphics.json/84 files 83 resources/
all 82 prior records-files exact/no deps/24 spans 74907B; immutable owner reconstructed
from header-index-packets-dictionary/7 classes 12 stats/20 banks 649 programs 4 group
masks. No decoder-upload shortcut/new ABI or guest cap. First 597 build fails only Bus
import test-only, diagnostics retained; corrected 597/258 core 0732b3b9 warning-free/
production unchanged. One Wine filter PASS publication-interruption/14 attach 2 full-set/
35 complete state-clock (28 loads + one original 1-program wrapper + 6 raw selectors)/
3 stop-safe families; 272 SERVED new B 4 prior world/fallback [1,0], not distinct union/
all direct native; 232 instructions 59 dispatch coverage only. All 74907 bus B/7 classes
20 archives 649 programs 4 groups/all 24 borders-mirrors-widths/prior world + raw gap/
Load slot Undo/DMA-CpuSet-FastSet mixed 64B full state-clock exact. 282 guards 597
accepted 597 diagnostic/595 model/7 current 83 prior retained. Accepted-selection.json
authoritative, selection.json first FAIL diagnostic. Actual CLI/ordinary/full ABI-live
uploads/mode 3-wider width/global extent/indirect/Linux-USA-native Windows/new 32:9/full
TODO open. No old proof/fixture/navigation/metric repeated.
[Runtime and evidence](../rom_data_indexed_graphics.md#managed-source-views).


**ROMDATA-04-INDEXED-GRAPHICS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 597-source production d8767645 warning-free, SAME runtime/no
overlays/canonical hash mtime finally restored. One Wine orchestration/three exit 0:
prepare creates e91-d90, prepare reuses, runc copied normal checkpoint 24755 neutral 1
→24756. 84 runtime-exact files/83 ID hashes/86 managed paths hash mtime unchanged/
cache 28859 exact; index 82/no deps/24 spans 74907B. All 12 counts 0, registration/
compatibility only, no positive ordinary/full pair. 201 guards 597 CLI + 597 runtime/
84 accepted/ROM-seed-cache-TinyCC-canonical exact. No new navigation/fixture/old proof/
metric repeated. Ordinary/full ABI-live uploads/mode 3-wider width/global extent/indirect/
Linux-USA-native Windows/new 32:9/full TODO open.
[CLI and limits](../rom_data_indexed_graphics.md#actual-cli-registration).


**ROMDATA-04-INDEXED-GRAPHICS-ORDINARY completed for one EUR/Wine case
(2026-10-30).** One 598-source 6d40a6a5 warning-free/passive 12 stats + 5-line guard,
597 CLI core-runtime unchanged/inverse hubs exact/current parameter-world observers
retained. 8 windows 6 new 2 cached/453E2→1967C context 3→program 2 MENU; one Start 12
normal capture 41f 4519→4560/first source 9 guest 4529/rolling 41 suffix/discovery STOP.
SAME d876 CLI one 41/all 83 ID hashes 86 managed no-write/cache 28859 exact and SAME
test one full pair every 41 state-clock/classic-wide/OAM/5 audio exact, 12 identical
stats 274 reads 277 SERVED B = selector 4/header 6/index 4/token 67/literal 17/
dictionary header 4/dictionary 175, one archive-program-group/all 7 classes positive/
not all 20-649-4. 144 guards/598 frozen 597 CLI/3 current/inverse/originals exact.
Readonly full wide 284x160 projection/classic PNG shows MENU Party-Clan-AreaList-System/
MoonDay9, no choice/mission outcome. Normal inputs only, no RAM-ID-RNG-ROM forcing or
old 320 Down-A/world 38 daily 45/negative/proof replay/metric. Other ordinary/full ABI/
mode 3-wider width/global extent/indirect/Linux-USA-native Windows/new 32:9/full TODO
open; sparse 27 ordinary remains partial.
[Case and limits](../rom_data_indexed_graphics.md#ordinary-world-map-menu-equivalence).


**ROMDATA-01-LOCALIZED-TABLE-GRAPHICS EUR source contract completed
(2026-10-30).** Eight independent groups 71E590/5A4/5B8/5CC/644/680/6BC/6D0
raw u8 language 02001F6E×4/LE32 pointer→054A0, 40 new custom LZ-RLE/nonindexed
banks. 48 spans 53667 source B (53507 bank + 160 selector)/114880 decoded B, all
tokens/source/output reconstructed exactly; 14 windows 10 new 4 cached/48 pins/
8 root literals/111 guards/no overlap 83 owners 50522 spans/all 84 accepted files
exact. Output per group five banks 1472/2688/768/4928/1920/7488/1344/2368B, no
invented count-by-distance or guest cap/fallback skip. Live RAM/helper transfer/
later work remain original, full transfer ABI not closed. Known 71E5E0 old UI
banks/table alias separate and seven other raw-looking groups/tails/indirect calls/
global extent unclassified. Static only/no build-Wine-game-fixture-navigation/old
proof or metric; schema 1/e91-d90 unchanged. Model/Wine oracle/runtime/CLI/ordinary/
Linux-USA-native Windows/new 32:9/full TODO open.
[Contract and limits](../rom_data_ui_graphics.md#additional-european-table-selected-source-contract).


**ROMDATA-02-LOCALIZED-TABLE-GRAPHICS completed EUR/Wine (2026-10-30).**
New core ui_graphics::table_selected reuses Graphic/custom LZ: strict 8 tables × 5
contiguous BankPointers/40 ordered banks/raw encoded byte + LE words/group outputs,
editable shape distinct from exact EUR auth/14 fingerprints. Physical select None
for 5/255 but original raw encoded view. Separate catalog not yet enabled in managed
UI resource: old 16 banks/two inline switches/e91 unchanged. One 600 frozen/260 core
build cb46ccb1 warning-free from 598 ordinary + two leaves/one public module; one
Wine run 23 guards/40 new original 054A0 calls 56 raw selector prefixes PASS. All
53667 source B/48 spans/114880 decoded B/cursor-produced-returned-size/SP-R4..R11/
full EWRAM/IWRAM outside 40B original stack/VRAM-OAM-palette/final ROM exact.
1335470 instructions coverage only. 26 unit 111 source guards/600 frozen 598 preimages/
3 current/inverse hub exact; no old 16 UI/20 indexed/native 35/41 ordinary/CLI proof
replay/fixture-navigation/metric. Offline control label 23B2E corrected to 0150 word/
1344B, not group set 02A0 halfword, originals/inputs unchanged/no proof replay. Managed
runtime/native clock/CLI/ordinary/full helper ABI/live contexts/71E5E0 alias/seven raw
groups/global extent/indirect calls/Linux-USA-native Windows/new 32:9/full TODO open.
[Model and evidence](../rom_data_ui_graphics.md#typed-table-model-and-new-original-decoder-proof).


**ROMDATA-03-04-LOCALIZED-TABLE-GRAPHICS-RUNTIME completed EUR/Wine
(2026-10-30).** e92-d91/same 84 files 83 IDs/UI index 15 56 banks 10 selectors
106 bindings, all other 82 records-files/old 16 banks-two inline-shared OBJ exact.
40 banks + 40 pointers 80 new runtime bindings byte union 48 canonical spans 53667B
exact; 6 new table counters separate from old 6 UI stats, original decode-scratch-
helper-upload work/ABI retained. One 601-source/260-core d4a73eb2 warning-free from
600 model + one host leaf; ONE Wine PASS publication-changed UI interruption/
14 attach 2 full-set/22 complete state-clock (12 loads + 1 new bank decoder + 9 raw
selectors)/3 stop-safe families. 499 SERVED new B 5 prior indexed/fallback [3,0],
not distinct union/all direct native; 356 instructions 71 dispatch coverage only.
All 53667 bus B 24328 literal 160 selector 40 banks 8 groups/all 48 edges/widths-
mirrors-prior indexed-raw-old UI/Load slot Undo/DMA3 mixed full state-clock exact.
267 guards 601 frozen 600 model/seven current 84 prior exact/helper interrupted only.
No old 40 model/16 UI/20 indexed/native 35/41 ordinary/CLI prepare proofs replay or
fixture-navigation/metric. Actual CLI/ordinary/full helper ABI-live contexts/71E5E0
alias/seven raw groups/global extent-indirect-tails/Linux-USA-native Windows/new 32:9/
full TODO open.
[Runtime and evidence](../rom_data_ui_graphics.md#managed-table-selected-graphics-and-runtime-consumption).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 601 runtime-identical production 31c0f298 warning-free/no overlays/
canonical hash mtime finally exact; ONE Wine orchestration 3 commands child 0 creates-
reuses-neutral 1 normal 24755 copy→24756. 84 runtime-exact files 83 ID hashes 86 managed
no-write/cache 28859 exact, e92-d91/UI index 15 56 banks 10 selectors 106 bindings/same
84 files 83 IDs. Six new table counts 0/registration-compatibility only/no positive
ordinary-full pair. 202 guards 601 CLI + 601 runtime/84 accepted/ROM-seed-cache-TinyCC-
canonical exact. No old 40 model/16 UI/indexed 20/native 22-35/41 ordinary/CLI prepare
proof replay/new checkpoint derivation-navigation/metric. Positive ordinary/full helper
ABI/live contexts/global extent-indirect/71E5E0 alias/seven raw groups/tails/Linux-USA-
native Windows/new 32:9/full TODO open.
[CLI and limits](../rom_data_ui_graphics.md#actual-table-extension-cli-registration).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY partial; two negative bounds
closed EUR/Wine (2026-10-30).** Cached/raw source 244D6→2C8C4/root 71E6D0 and old
normal Fight prediction image select 64f 23851→23915; distinct 71E590 COMBOMENU
asset/30608 constructor + 30FDA redraw and Action seed 23611 two Down+A select 160f
→23771. SAME 31c0 CLI both child 0/all 6 new counts 0/83 ID hashes 86 managed
no-write/cache 28859 exact. Static initializer link does not prove new load; PPM
omits separate HUD/menu, Combo not claimed reached. No observer build/guard/full pair/
endpoint state/longer retry/RNG resampling/old route recreation. Diagnostics retained/
discovery stopped; positive ordinary/full equivalence open pending concrete constructor
context. Source-model-runtime-CLI registration remain closed; Linux-USA-native Windows/
new 32:9/full TODO open/no metrics.
[Status and limits](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).


**ROMDATA-01-LOCALIZED-RAW-TABLE-SOURCES completed for fixed EUR views
(2026-10-30).** Seven independent roots 71E5F4/608/66C/694/6A8/6E4/6F8, five
physical slots each and unbounded u8 language selector: 35 views/42 spans/38220B
(38080 payload + 140 selector), SHA
e11ef4d5f16feea8ef90e7fce58056e19d689004b7ca973270285e3cd31b8f80. Six prefixes
224/160/768/1600/2048/2432B and only +0200..0380/384B of seventh; lengths from
original copies/alternate branches, not distance/header. Fused BL 12EE24→145230/
CpuSet halfword and 6F8→000AD0/control 80000040 halfword verified, first hypothesis
corrected before output. 32 windows 23 new 9 cached/193 pins/13 root literals/
13 fixed copies/130 guards/no overlap 83 owners 50602 bindings/all 84 accepted files
exact. Four dynamic 6F8 views/u8 task+2−4/u16 offset<<5 do not establish a cap:
metadata/lower prefix/tail and full domain remain open. Static analysis only, schema
1/e92-d91/runtime unchanged; model/Wine oracle/runtime/CLI/ordinary/full ABI/71E5E0
alias/indirect/full TODO open; new Linux-USA-native Windows-32:9 to do, no metric/
build/game/fixture/navigation/old proof repeated.
[Contract and limits](../rom_data_ui_raw_transfers.md#declared-source-contract).


**ROMDATA-02-LOCALIZED-RAW-TRANSFERS completed EUR/Wine (2026-10-30).**
New core ui_raw_transfers/RawTransfers+BankView: seven physical tables/35 ordered
views, 42 spans 38220B/LE words and seventh sparse group +0200..0380 only. Editable
shape distinct from EUR authentication/32 fingerprints; u8 language addresses outside
0..4 raw, no fallback/cap. Build 603 sources/262 core 35bdc5fd warning-free; three
builds and two Wine attempts, one final filter PASS 21 guards/91 original prefixes/
60 isolated helper copies/25 original DMA queue executor copies. All bytes, EWRAM/
VRAM/OAM/palette, IWRAM outside original 36B stack when needed, SP/callee/final ROM
exact. First corrections test-only: stack 20→36B before Wine, boundary between adjacent
tables incorrectly expected raw; diagnostics and preimage retained, model/source
unchanged. 44 unit 130 source guards/601 baseline 603 frozen/3 current/inverse hub exact;
6635 instructions coverage only. Schema e92-d91 and 84 accepted files unchanged;
runtime/native clock/CLI/ordinary/full caller ABI/6F8-lower prefix-metadata-tail domain/
71E5E0 alias/full TODO open. New Linux-USA-native Windows-32:9 to do; no old proof/
game/fixture/checkpoint/navigation/metric repeated.
[Model and evidence](../rom_data_ui_raw_transfers.md#typed-model).


**ROMDATA-03-04-LOCALIZED-RAW-TRANSFERS-RUNTIME completed EUR/Wine
(2026-10-30).** e93-d92/index 83 graphics.ui-raw-transfers/85 files 84 IDs, all
83 previous records/payload exact. 42 spans 38220B from typed raw words and pointers,
six separate counters; original selectors/copies/DMA queue/ABI-clock preserved. Two
builds 605 sources/263 core, accepted 4b219adc warning-free; one Wine filter PASS
publication-interruption/14 attach 2 full-set/23 native full states-clock (12 loads +
1 CpuSet + 1 queue executor + 9 language prefixes)/4 stop-safe families. 939 SERVED
new B 3 previous/fallback [768,0], not distinct union or all direct native; 115
instructions 25 dispatch coverage only. All 38220B/35 views 7 groups/42 edges/width-
mirror/adjacency between new and old/raw gaps/Load slot Undo/four mixed 64B transfers
DMA16-DMA32-CpuSet-FastSet pass. 309 guards 605 frozen 603 model/7 current 84 old/
inverse hub exact. Before Wine only test corrected: 441608 starts previous indexed
archive, not raw gap; product code unchanged between builds and first diagnostic
950bea97 retained. Runner labels e92/DMA3 obsolete metadata only: JSON proves [1,93,92]/
4 transfers, no replay. Real CLI/ordinary/full caller ABI/6F8-metadata-prefix-tail domain/
71E5E0 alias/indirect/full TODO open; new Linux-USA-native Windows-32:9 to do. No old
proof/source-model/game-fixture-checkpoint-navigation/metric repeated.
[Runtime and evidence](../rom_data_ui_raw_transfers.md#managed-raw-transfer-resource).


**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-CLI EUR/Wine create/reuse/startup completed
(2026-10-30).** One 605 runtime-identical production build ffda6c59 warning-free/no
overlays/canonical hash mtime finally exact; one Wine orchestration three child 0
commands create-reuse-neutral 1/normal 24755 copy→24756. 85 runtime-identical files/
84 ID hashes 87 managed no-write/cache 28859 exact. e93-d92/index 83/35 views 7 tables/
42 bindings 38220B. Six new counters 0: registration/compatibility only, not positive
ordinary use or full pair. 203 guards 605 CLI + 605 runtime/85 accepted/ROM-seed-cache-
TinyCC-canonical exact; no new endpoint/derived checkpoint/action input/navigation/old
proof repeated/metric. Positive ordinary/full ABI/6F8-metadata-prefix-tail domain/
71E5E0 alias/indirect/global catalog/full TODO open; new Linux-USA-native Windows-32:9
to do.
[CLI and limits](../rom_data_ui_raw_transfers.md#actual-cli-registration).

**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-ORDINARY completed for the selected EUR/Wine
startup (2026-10-31).** CLI e93 uses the retained normal startup input: 1.372 reads,
5.488 bytes served (16 selectors, 5.472 payload), four banks and four groups. A
1.200-frame pair with the same dataset compares full state, clock, video, OAM, dispatch
and audio: all equal. Both arms use the same native cache; 92 protected files unchanged.
No new build or performance measurement. Other paths, dynamic domain, full ABI, Linux,
USA and native Windows remain open.
[Evidence and limits](../rom_data_ui_raw_transfers.md#ordinary-european-boot-equivalence).

**ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN partial (2026-10-31).** Original callers
explicitly supply selectors 7, 8 and 9; two others read a live byte without a proven
limit. Six rows and five candidate prefixes cover 1.304 bytes without overlap with
e93 resources. No global limit, ordinary use or new runtime consumption is inferred
from static analysis alone. Subsequent model and integration are described below;
remaining domain and Linux/USA remain open.
[Details](../rom_data_ui_raw_transfers.md#bounded-dynamic-view-producer-discovery).

**ROMDATA-02-LOCALIZED-RAW-DYNAMIC-FRAGMENTS completed for the declared EUR/Wine
model (2026-10-31).** Seven ranges/1.304 bytes reconstructed and verified from ROM;
targeted shape, JSON, authentication and boundary check passed on Wine. No new runtime
owner, guest limit or execution test of original routines. Subsequent integration is
described below; global domain and Linux/USA checks remain open.
[Model and limits](../rom_data_ui_raw_transfers.md#dynamic-fragment-source-model).

**ROMDATA-03/04-DYNAMIC-RAW-FRAGMENTS completed for declared EUR/Wine publication
and reads (2026-10-31).** e94-d93 adds independent resource index 84, seven ranges/
1.304 bytes, retaining the previous 84 resources. Wine proof checks bytes, boundaries,
composition and one DMA with full state/clock, plus rejection of altered data. First
attempt identified a missing page, corrected before passing. Production CLI, native
instructions, ordinary use, global domain and Linux/USA/native Windows checks remain;
no performance measurement.
[Evidence](../rom_data_ui_raw_transfers.md#managed-dynamic-fragment-resource).

**ROMDATA-04-DYNAMIC-RAW-FRAGMENTS-CLI completed for EUR/Wine registration
(2026-10-31).** Production CLI creates and reuses e94-d93, loads all 85 resources
with verified hashes and advances one neutral frame without changing managed files.
The 86 generation files match the accepted dataset. This does not prove dynamic
ordinary use; native execution, global domain and Linux/USA/native Windows remain
to verify. No performance measurement.
[Evidence](../rom_data_ui_raw_transfers.md#dynamic-fragment-production-cli).

**ROMDATA-04-DYNAMIC-RAW-ORIGINAL-NATIVE completed for isolated EUR/Wine cases
(2026-10-31).** Original routine, both queues and subsequent DMA pass 30 combinations
(selectors 7/8/9, two branches, five languages). Pointers and transferred bytes match;
full state/clock match between interpreter with ROM, interpreter with dataset and
translated code with dataset, with dynamic sources poisoned in both copies. Ordinary
use, global domain, full ABI and new Linux/USA/native Windows checks remain open; no
performance measurement.
[Evidence](../rom_data_ui_raw_transfers.md#original-dynamic-routine-and-translated-execution).

**ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY partial (2026-10-31).** Source
identifies guest gate `0200203d` and two callbacks that activate and deactivate it.
Fourteen EUR checkpoints show no ready selectors; an existing worldmap→Party→Unit
320-frame recording has no bus reads of the seven ranges or observed caller entries.
Native sampling may miss changes within blocks. No global absence is inferred:
positive ordinary use, state/clock comparison, global domain and Linux/USA/native
Windows remain open. No performance measurement.
[Evidence](../rom_data_ui_raw_transfers.md#ordinary-dynamic-reachability-investigation).

**ROMDATA-01-DYNAMIC-RAW-DISPATCH-LIFECYCLE partial EUR/Wine (2026-10-31).** The
dispatcher goes through callback `063878`, conditionally registered by flag `40h`
(bit 0 of `02001f78`); selection 2 can set it. One Wine observation of the existing
Area List/240-frame path reaches the comparison once, but shows neither an active bit
at sampled points nor bus reads of dynamic ranges. This is a bounded negative:
positive ordinary use, full comparison, global domain and Linux/USA checks remain
open; no performance measurement.
[Evidence and limits](../rom_data_ui_raw_transfers.md#dispatcher-lifecycle-and-area-list-observation).

**ROMDATA-01-LOCALIZED-TABLE-ALIAS bounded EUR source (2026-10-31).** Table
`71e5e0` has 20 unmanaged bytes and points to the five already managed clan selector
banks. Five literal references and six original selection sequences verified; live
u8 language index without inferred fallback. Model, publication, runtime consumption,
ordinary use and Linux/USA remain open.
[Contract](../rom_data_ui_graphics.md#european-alias-of-the-existing-clan-banks).

**ROMDATA-03-04-LOCALIZED-TABLE-ALIAS completed for EUR/Wine publication and
interpreter prefixes (2026-10-31).** e95-d94 adds the 20 table bytes as five typed
pointers to the existing UI resource, without duplicating clan banks; the other
84 resources remain identical. Boundary reads, 42 original prefix combinations with
full state/clock and transactional rejection of an altered file verified. Production
CLI, native execution, positive ordinary use and Linux/USA remain to verify. No
performance measurement.
[Evidence](../rom_data_ui_graphics.md#managed-european-clan-table-alias).

**ROMDATA-04-CLAN-TABLE-ALIAS-NATIVE EUR/Wine (2026-10-31).** Six translated
original prefixes, 24 cases with indices 0/4/5/255: full state/clock equal to ROM
interpreter, 48 bytes served from dataset with altered ROM table. CLI, ordinary use,
full ABI and Linux/USA checks remain open.
[Evidence](../rom_data_ui_graphics.md#translated-clan-alias-selector-prefixes).

**ROMDATA-04-CLAN-TABLE-ALIAS-CLI EUR/Wine (2026-10-31).** Production CLI creates
and reuses e95-d94 and advances one frame: 86 files identical to accepted dataset,
85 runtime identities verified and no change to managed files. Positive ordinary use,
full comparison and Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_ui_graphics.md#clan-alias-production-cli).

**ROMDATA-05-CLAN-TABLE-ALIAS-ORDINARY bounded negative EUR/Wine
(2026-10-31).** Existing 1.200-frame startup path showed no alias bus reads in
instrumented readers; no events lost. This does not prove global absence: positive
ordinary use, full comparison and Linux/USA checks remain open. No performance
measurement.
[Evidence and limits](../rom_data_ui_graphics.md#clan-alias-ordinary-boot-observation).

**ROMDATA-05-CLAN-TABLE-ALIAS-BANK-ENTRY EUR/Wine (2026-10-31).** An already
recorded Bank entry shows two typed alias reads; one is attributed to original
instruction `0805e73a`. A separate 120-frame pair with the same state and input
compares full state/clock, video, OAM, dispatch and audio: all equal. Other paths
and languages, full ABI and Linux/USA checks remain open. No performance measurement.
[Evidence](../rom_data_ui_graphics.md#ordinary-clan-alias-use-during-bank-entry).

**ROMDATA-01-E95-HISTORICAL-READ-RESIDUE bounded (2026-10-31).** Comparison of
already retained F09 reads against e95 finds 74 bytes now managed and 1.072 still
outside owners; all latter have previous code-support evidence reauthenticated on
EUR ROM. This is not a new execution or global census: other scenes and data remain
open. Linux/USA to do; no performance measurement.
[Evidence](../rom_data_runtime.md#historical-f09-residue-against-e95).

**ROMDATA-01-GAMEPLAY-6E6DBC-LOOKUP bounded EUR source (2026-10-31).** Original
readers use two little-endian fields per row, stride four; documented producers support
a conditional projection 0..73 of 296 bytes. This is not a global limit: other indices
retain original reads. Model, publication, runtime and scene remain open; Linux/USA
to do, no performance measurement.
[Contract](../rom_data_gameplay.md#european-conditional-four-byte-lookup-projection).

**ROMDATA-02-GAMEPLAY-6E6DBC-LOOKUP EUR/Wine model (2026-10-31).** Strict model
of 74 rows/296 bytes with source authentication and original wrapping address; one
Wine test passes original readers, inside and outside projection indices, JSON and
rejection of changes. Managed extraction, runtime and ordinary use remain open;
Linux/USA to do, no performance measurement.
[Evidence](../rom_data_gameplay.md#typed-conditional-four-byte-lookup-model).


**ROMDATA-03-04-GAMEPLAY-6E6DBC-LOOKUP managed EUR/Wine (2026-10-31).**
e96-d95 publishes the conditional projection of 74 rows: 85 previous resources
identical, typed reads and edges verified, transactional rejection of altered data.
Three original native prefixes and six indices match interpreter state/clock; indices
outside projection are not limited. Production CLI, ordinary use and Linux/USA checks
remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-four-byte-lookup).


**ROMDATA-04-GAMEPLAY-6E6DBC-CLI EUR/Wine (2026-10-31).** Production CLI creates
and reuses e96-d95 and advances one frame: 87 files identical to accepted dataset,
86 runtime identities verified and no change to managed files. Ordinary use and
Linux/USA checks remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-lookup-production-cli).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY-ROUTE EUR candidate (2026-10-31).** An
already retained Abilità input and original first-category branch indicate a candidate
for typed read; reached state and nonempty list are hypotheses to observe. One Wine
prefix of 540 frames is the next check; Linux/USA remain open. No performance
measurement.
[Evidence](../rom_data_gameplay.md#conditional-lookup-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY positive EUR/Wine (2026-10-31).** Already
retained ordinary Abilità input produces 55 typed reads/bytes, 27 of first field and
28 of second, in the 540-frame segment. The counter is aggregate; full scene comparison
and Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#ordinary-conditional-lookup-use-in-ability-navigation).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY-PAIR EUR/Wine (2026-10-31).** In the same
540-frame Abilità sequence, full state/clock, video, OAM, dispatch and five audio
outputs match between original ROM and managed runtime; 55 typed bytes cover both
fields in the same test. Other scenes and Linux/USA checks remain open; no performance
measurement.
[Evidence](../rom_data_gameplay.md#ordinary-ability-scene-differential).


**ROMDATA-01-GAMEPLAY-6E6EE4-ROOT conditional EUR source (2026-10-31).**
Three original readers use a u16 index from a stride-eight descriptor; the type 5
branch has static witnesses for indices 1..33, totaling 132 bytes without overlap
e96. Ordinary reachability, row 0 and global limit are unproven; other types may read
previous owners. Model and runtime remain open; Linux/USA to do, no performance
measurement.
[Contract](../rom_data_gameplay.md#european-conditional-packed-four-byte-projection).


**ROMDATA-02-GAMEPLAY-6E6EE4-PACKED EUR/Wine model (2026-10-31).** Strict model
of 33 typed rows, reconstruction of 132 bytes and original readers checked for zero,
endpoints 1/33 and alias 251 to metadata14. It introduces no guest limits; managed
publication, runtime and Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-packed-lookup-model).


**ROMDATA-03-04-GAMEPLAY-6E6EE4-PACKED managed EUR/Wine (2026-10-31).**
e97-d96 publishes 132 typed bytes: 86 previous resources identical, shared boundaries
and alias 251 still served by metadata14, transactional rejection of changes and two
native prefixes equivalent to interpreter. CLI, ordinary use and Linux/USA checks
remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-packed-lookup).


**ROMDATA-04-GAMEPLAY-6E6EE4-CLI EUR/Wine (2026-10-31).** Production CLI creates
and reuses e97-d96 and advances one frame: 88 files identical to accepted dataset,
87 runtime identities verified and no change to managed files. Ordinary use and
Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-packed-lookup-production-cli).


**ROMDATA-05-GAMEPLAY-6E6EE4-ORDINARY-ROUTE EUR candidate (2026-10-31).** An
already recorded Polly→Hildegard attack contains a wrapper with type 5/index 1
descriptor and an original chain compatible with the reader. Some branches remain
to observe; an earlier suspicion of bit 43 blocking was wrong. One Wine observation
limited to the existing segment can distinguish use.
[Evidence](../rom_data_gameplay.md#conditional-packed-lookup-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6E6EE4-ORDINARY bounded negative EUR/Wine
(2026-10-31).** Existing 500-frame F09 combat shows no packed lookup reads. This is
a negative for the observed scene alone, not proof of global absence; positive
ordinary use and Linux/USA remain open. No performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-packed-lookup-f09-observation).


**ROMDATA-01-GAMEPLAY-6DF4D4-DESCRIPTORS conditional EUR source
(2026-10-31).** Original count producer supports 23 pointers and 684 eight-byte
records in banks 1..23. Count 11 of bank 22 excludes 48 bytes before the next pointer;
bank 0 remains separate and raw. No global guest limit; model/runtime and Linux/USA
remain open.
[Contract](../rom_data_gameplay.md#european-conditional-eight-byte-descriptor-banks).


**ROMDATA-02-GAMEPLAY-6DF4D4-DESCRIPTORS EUR/Wine model (2026-10-31).**
Lossless model of 23 banks and 684 records, with original counts and writes verified
on Wine; bank 0 and bank 22 gap remain raw and without invented guest limits.
Publication/runtime and Linux/USA checks remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-descriptor-bank-model).


**ROMDATA-03-04-GAMEPLAY-6DF4D4-DESCRIPTORS managed EUR/Wine
(2026-10-31).** e98-d97 publishes 23 pointers and 684 typed descriptors: 87 previous
resources identical, bank 0 and bank 22 gap remain raw, consistent edit rejected and
five native prefixes equivalent to interpreter. CLI, ordinary use and Linux/USA remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-descriptor-banks).


**ROMDATA-04-GAMEPLAY-6DF4D4-CLI EUR/Wine (2026-10-31).** Production CLI creates
and reuses e98-d97 and advances one frame: 89 files identical to accepted dataset,
88 runtime identities verified and managed files unchanged. Descriptor counters zero
in the frame; ordinary use and Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#conditional-descriptor-banks-production-cli).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY-ROUTE EUR candidate (2026-10-31).** Original
Abilità panel renderer reads pointers and descriptors from unit bank; already retained
state and input reach the panel. e98 use is not yet observed. One 540-frame Wine segment
checks it.
[Evidence](../rom_data_gameplay.md#conditional-descriptor-banks-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY positive EUR/Wine (2026-10-31).** The
540-frame Abilità sequence reads 64 typed bytes: 52 pointers, 6 field 0, 6 field 2;
other fields remain unobserved. Full comparison, other scenes and Linux/USA remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-descriptor-use-in-ability-navigation).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY-PAIR EUR/Wine (2026-10-31).** In the same
540-frame Abilità sequence, state/clock, video, OAM, dispatch and audio match; in the
same test 52 pointer bytes and 6+6 bytes of fields 0/2 are typed. Field 4/type/byte 7
and Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-ability-descriptor-scene-differential).


**ROMDATA-01-GAMEPLAY-6DF4D4-BANK0 conditional EUR source (2026-10-31).** Original
producer reads pointer 0 and may enumerate records 1..15 when unit+37 is nonzero;
an initializer may set it for unit+7=24. No ordinary witness in 62 snapshots examined;
record 0 and other indices remain raw, with no invented guest limit. Model/runtime and
Linux/USA remain open.
[Contract](../rom_data_gameplay.md#conditional-descriptor-bank-0-source).


**ROMDATA-02-GAMEPLAY-6DF4D4-BANK0 EUR/Wine model (2026-10-31).** Strict model for
pointer 0 and records 1..15; original predicate, store, mode 3 and scan pass on Wine
for zero/nonzero slots. Record 0 remains raw; ordinary use, publication/runtime and
Linux/USA remain open.
[Evidence](../rom_data_gameplay.md#typed-conditional-descriptor-bank-0-model).


**ROMDATA-03-04-GAMEPLAY-6DF4D4-BANK0 managed EUR/Wine (2026-10-31).** e99-d98
publishes pointer 0 and 15 descriptors: 88 previous resources identical, boundaries
with bank 1 composed and record 0 raw, consistent edit rejected, three native prefixes
equivalent. CLI, ordinary use and Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#managed-conditional-descriptor-bank-0).


**ROMDATA-04-GAMEPLAY-6DF4D4-BANK0-CLI EUR/Wine (2026-10-31).** Production CLI
creates and reuses e99-d98 and advances one frame: 90 files identical to accepted
dataset, 89 runtime identities verified and managed files unchanged. Ordinary use and
Linux/USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-descriptor-bank-0-production-cli).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANK0-ORDINARY-PAIR EUR/Wine (2026-10-31).** Item
information screen uses pointer 0 and fields 0/2/type/byte 7 of records. On the same
800-frame ordinary sequence raw ROM and e99 dataset comparison matches state/clock,
video, OAM, dispatch and audio; field 4 not observed. Both sides share native cache:
Linux, USA and native Windows remain to verify. No performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-item-info-access-to-descriptor-bank-0).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANK0-FIELD4-PAIR EUR/Wine (2026-10-31).** In battle
Item menu an existing 340-frame sequence reads 32 typed bytes of bank 0 field 4. Raw
ROM/e99 comparison matches state/clock, video, OAM, dispatch and audio, using the same
native cache on both sides. Linux, USA and native Windows remain to verify; no performance
measurement.
[Evidence](../rom_data_gameplay.md#ordinary-battle-item-menu-bank-0-field-4).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-KIND-RECONCILE EUR/Wine (2026-10-31).** Already
retained battle Item scene log also records 4 pointer bytes and 1 type byte of the
banks 1..23 descriptor resource. This is aggregate use, without individual record/PC
attribution; scene comparison verifies bank 0 without asserting this secondary resource.
Ordinary field 4/byte 7, Linux and USA remain open. No replay or performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-ability-descriptor-scene-differential).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-INFO-PAIR EUR/Wine (2026-10-31).** Already
recorded Battle Tech screen reads 10 typed type bytes and 10 byte 7 values of descriptors
in banks 1..23. Over 800 frames raw ROM/e99 comparison matches state/clock, video, OAM,
dispatch and audio; field 4 not observed and native cache shared. Linux, USA and native
Windows remain to verify; no performance measurement.
[Evidence](../rom_data_gameplay.md#ordinary-battle-tech-descriptor-information).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-FIELD4-PAIR EUR/Wine (2026-10-31).** In Steal
menu a retained 280-frame sequence reads 60 typed bytes of field 4 in banks 1..23.
Raw ROM/e99 comparison matches state/clock, video, OAM, dispatch and audio; both sides
use same native cache. Linux, USA and native Windows remain to verify; no performance
measurement.
[Evidence](../rom_data_gameplay.md#ordinary-steal-menu-descriptor-field-4).


**ROMDATA-01-GAMEPLAY-6E0B44-MASKS conditional EUR source (2026-10-31).**
Three original readers index u32 masks using metadata52 property 0x24. The 116
authenticated rows produce all indices 0..34; source range 6e0b44..6e0bd0 contains
35 words and is distinct from e99 resources. Dynamic/invalid selectors are not globally
bounded: no guest cap. Model, runtime, ordinary use and Linux/USA remain open; no
performance measurement.
[Contract](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-02-GAMEPLAY-6E0B44-MASKS EUR/Wine model (2026-10-31).** Strict lossless
model of 35 u32 masks/140 bytes, without added guest limits; Wine checks bytes,
boundaries, original selectors and external indices. Publication/runtime, ordinary
use, Linux and USA remain open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-03-04-GAMEPLAY-6E0B44-MASKS managed EUR/Wine (2026-10-31).** e100-d99
publishes 35 typed masks, retains exactly 89 previous resources, composes boundaries
with adjacent owners and rejects consistent edits; one original native prefix matches
the interpreter in the six selected cases. CLI, ordinary use, Linux and USA remain
open; no performance measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-04-GAMEPLAY-6E0B44-MASKS-CLI EUR/Wine (2026-10-31).** Production CLI
creates and reuses e100-d99 and advances one neutral frame: 91 files identical to
accepted dataset, 90 runtime identities verified and managed files unchanged. Masks
are not read in the frame; ordinary use and Linux/USA remain open. No performance
measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-05-GAMEPLAY-6E0B44-MASKS-ORDINARY-PAIR EUR/Wine (2026-10-31).**
Retained headgear list reads typed masks 91 times (364 aggregate bytes). Raw ROM/e100
comparison over 560 frames matches state/clock, video, OAM, dispatch and audio; both
sides share native cache. Linux, USA and native Windows remain to verify; no performance
measurement.
[Evidence](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-04-DYNAMIC-FRAGMENT-USAGE-CLI EUR/Wine (2026-10-31).** Dynamic fragments
expose separate counters for requests, selector bytes and payload. One neutral frame
of Windows CLI under Wine emits the new line with zero counters, reusing e100 dataset
without changes; ordinary use, Linux and USA remain open. No performance measurement.
[Evidence](../rom_data_ui_raw_transfers.md#managed-dynamic-fragment-resource).

**EUR e101-d100 progress (2026-10-31).** The attested compact OBJ projection is
published and checked with targeted Wine tests for sources, boundaries, transactional
loading and one original native/interpreter prefix. Production CLI, ordinary use,
unattested sequence 8, Linux and USA remain to verify. No performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).

The e101-d100 production CLI passed creation, reuse and one Wine startup; ordinary
use of the new compact OBJ resource remains to be demonstrated.

**EUR e102-d101 progress (2026-10-31).** A second sparse compact OBJ bank is published
and checked on Wine for sources, boundaries, transactional loading and one native/
interpreter prefix. Production CLI, ordinary use, unattested entries, Linux and USA
remain to verify. No performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6c6460).

The e102-d101 production CLI passed creation, reuse and one Wine startup; ordinary
use of the second compact OBJ bank remains to be demonstrated.

**EUR e103-d102 progress (2026-10-31).** A third sparse compact OBJ bank, including
an empty list, is published and checked on Wine for sources, boundaries, transactional
loading and one native/interpreter prefix. CLI, ordinary use, unattested entries, Linux
and USA remain to verify. No performance measurement.
[Contract](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6bf490).

The e103-d102 production CLI passed creation, reuse and one Wine startup; ordinary
use of the third compact OBJ bank remains to be demonstrated.
