<a id="todo--estrazione-rom-dati-decodificati-e-mod"></a>

# TODO — ROM extraction, decoded data and mods

Request updated October 22, 2026. **FEAT-04-ROM-DATA** is in progress;
inventory and initial contract are defined. Runtime migrations remain open.
Scope: desktop workspace, Windows as the primary validation platform; Linux as
an additional check. `AndroidPort/` requires separate work.

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


**ROMDATA-01-UNIT-OBJ-ALTERNATE-TARGET-VIEWS — contratto statico completato
(2026-10-24), modello/oracolo ancora da fare:**65IDalternativi/5850chiavi,
5682celle fisiche=4709nulle+953con706sequenze2771frame+20vistedi radiciindice
condivise. Tutte20hanno campidirectory/indice initial-pose indipendenti come
testimoni; l'unico overlapinterno4B è247/anim104→directory/rootsprite0. La vecchia
interpretazione blanketcome sequenze rimane respinta: nessun filtro count/cap,
chiave rimossa o limiteperadiacenza.30listeOBJ108oggetti/1203richieste pixel
387008B; nuovi3321intervalli410086B dopo6232initial+52604normal-battle+22equipment
condivisi, nessun altro overlap nei55owner e60. Nuove606sequenze2210frame/5252
celle4428nulle/1079richieste pixel342112B. Contratto eSHA bdf15f7d.. verificati
staticamente; prossimo02modello con targettaggati e dipendenze esplicite, poi
unico nuovo oracolo Wine dei lettori originali. Nessuna nuova generazione/runtime/
scena dichiarata; e60/USAe8invariati. Raggiungibilità attore/azione, script49/
streamRAM/Linux/USA/nativeWindows/nuovo32:9 restano aperti, nessuna misura.
[Contratto e limiti](../rom_data_unit_obj.md#alternate-battle-selectors-and-explicit-shared-target-views).


**ROMDATA-02-UNIT-OBJ-ALTERNATE-BATTLE completato nel perimetro EUR/Wine mirato
(2026-10-24):** modello pubblico lossless con targettaggati null/sequence/shared-
index-root conserva tutte5850chiavi/20vistetestate contro i campi initial-pose,
706sequenze2771frame/30listeOBJ/1203blocchi4bpp.3321nuovi intervalli410086B dopo
validazione dei treownercondivisi; unico overlap4Bdir/indice con testimoneesatto.
Una buildfrontend Windows senza warning/unico filtro Wine al primoPASS:15guardie/
10544prefissi costruttore (40alias condivisi)/4428reset nulli/606reset sequenze/
cinquecomandi nuovi2-3-8/1079scheduler-DMA/tutti342112Bpixel nuovi/VRAMintera.
Nessun falsoframe parsato o schedulato; opcode6e precedenti originali non
ripetuti. Source/modello chiusi; prossimo03pubblicazione gestita/consumo tipizzato,
poiCLI/uso ordinario. e60/USAe8invariati; Linux/USA/nativeWindows/nuovo32:9da fare,
nessuna misura/spazio/performance o nuovo checkpoint di gioco.
[Modello e prove](../rom_data_unit_obj.md#typed-alternate-target-model).


**ROMDATA-03-UNIT-OBJ-ALTERNATE-BATTLE completato nel perimetro EUR/Wine mirato
(2026-10-24):** e61-d60pubblica/consuma unit-obj-alternate-battle.json con cinque
reali dipendenze metadata52/14/initial/equipment/normal;57file56risorse/tutti55
precedenti record/coverage/decodificati invariati/e60conservato/USAe8invariato.
Solo3321nuovi intervalli410086B sono owned; venti radiciindice condivise restano
nonsequenze e gli owner precedenti mantengono la responsabilità. Una buildcore/
frontend Windows senza warning (core solo compilato), due nuovi filtri Wine al
primoPASS:20guardie/lifecycle/tutti410086Btipizzati/606sequenze;34percorsi originali/
interprete/compilato con stati completi e clock uguali, part0sequenza/part1-2nulli/
prefissi sharedroot/IDfullword/RAMlive/comandi2-3-8/dueupload+dueDMA/tutte6API/
3321bordi/mirror/BIOS/SWI-DMA. Finestramista2nuovi780grezzi8OBJprecedenti e API
2nuovi+2grezzi modificati preservano addebiti esatti. Target::Null emptystruct
rifiuta campi extra prima ignorati da serde; JSON valido identico cad7771d..,
nessuna ripetizione dell'oracolo fisico. Tutti8216artefatti precedenti/15input
compilati/preimage/961build+963Wineguard preservati. Figli source/model/managed/
native chiusi; prossimi04CLIreale e05uso ordinario positivo della nuova famiglia.
Raggiungibilità/script49/streamRAM/05equipment positivo eLinux/USA/nativeWindows/
nuovo32:9aperti; nessuna misura performance-spazio o nuovo checkpoint di gioco.
[Contratto e prove](../rom_data_unit_obj.md#alternate-managed-publication-and-typed-runtime-reads).


**ROMDATA-04-UNIT-OBJ-ALTERNATE-BATTLE-CLI completato EUR/Wine;05positivo aperto
(2026-10-24):** unica buildproduzione senza warning/frozend3db6585../canonico
hash-mtimeesatto ripristinato; CLIreale crea/riusa57filegenerazione59gestiti senza
riscritture, tutti uguali all'e61accettato, da cwdestranea. Un frameconcachevecchia
ed autentico snapshotF09immutato passa:0usoalternativo, solo smokeattivazione e
nessuna ripetizione della coppia normale3530già chiusa. Lettura statica/autenticata
81snapshotEUResistenti non trova taskcorrente con sequenzanuovaalternativa che
fornisca un concreto candidato per il gatepositivo. Non esclude usofuturo nelle
registrazioni; nessun locatorlungo/guardscena/buildtest/fullpair o nuova navigazione/
checkpoint/payloadedit. Primohelperstatico confondeva puntatoreguest/offsetfile:
preimage/risultatiinvalidi conservati, corretta solo ispezionestatica economica,
nessuna accettazionegioco invalidata.8543artefattiprecedenti/1023guardCLI/15input
runtimecompilati-preimage immutati. Source/model/managed/native/actualCLI chiusi;
05usoordinario alternativo positivo ancora aperto;05equipmentpositivo poi chiuso
il25ottobre con checkpoint autorizzati e singola nuova coppiaFight500.
Script49/live/nuoviLinux-USA-nativeWindows-32:9 restano aperti, nessuna misura.
[CLI e limiti](../rom_data_unit_obj.md#alternate-actual-cli-and-remaining-ordinary-use-gate).


**ROMDATA-01-SCRIPT-OBJ-ACTOR-PAIRING: classificazione selettori completata,
accoppiamento sprite ancora aperto (2026-10-24).** Decimale49(0x31), callback
126CC0, conserva167istruzioni/18selettori/13basi.151ricerche nelle unitàRAM
(12+24recordstride264/identitàbyte4),16nel registro/fallback vivo;0ID sprite
immediati. La ricercacontext confronta puntatoriunità e usa spritecorrente+34.
Gruppou16×4low16/facingsigned+1F/bodyflag0/medesima baseextra sono originali.
31riusa unitàesistenti senza cambiare byte01;32solo riposiziona, quindi l'ordine
nel file non prova dominanza o spriteimmutato. Script e provenienza esistenti/
67halfword9literal18BL autenticati; nessuna nuovaownership/modello/runtime/
revisione/build/testguest. Prima notazionehex/decimale corretta e un pinBLspostato
alla prima halfword, senza ripetere accettazioni chiuse. Prossima evidenza utile:
sole querysprite/animazione/parte risolte durante49in una registrazioneEUR
immutata, mantenendo collegamentoistruzione/attore/lettore. Nessun prodotto
indiscriminato scripts×sprites. e61/USAe8invariati; nuoveLinux-USA-nativeWindows-
32:9 da fare, nessuna misura o nuova fixture/navigazione/checkpoint.
[Contratto dei selettori vivi](../rom_data_unit_obj.md#script-animation-selectors-require-live-actor-bindings).


**ROMDATA-01-SCRIPT-OBJ-RESOLVED-QUERIES completato per la registrazioneEUR
mirata (2026-10-24):** osservatoresolo-test/registrazione cfg(test), nessun codice
produzione/sourceowner/ABI/save/cache modificato. Primo preflightWine fallisce
prima di snapshot/frame per ipotesi tuttiingressinativi; prova/preimage conservati.
Corretto solo osservatore: forwarding funzione nativa doveesiste, originale
Machine::step dovecachemancava. Buildfrontendcorretta warning-free/frozen4342f2..;
unico censimento effettivo Wine6000frame21420→27420passa con sixhandler49/sixbody
SP-LRlinked/18querycostruttore reali. Seiistruzioni selector16/group2risolvono
sprite46/anim8/parti0-1-2; disponibilitànative[false,false,true], contabilità
strumentata non prova equivalenzadispatch. AuditROM/tutti56owner: corpo giànormal,
duecelle4Bnulle37C760/37C764nonowned,8B/SHAaf5570f5.., nessuna nuova sequenza/pixel.
1025guardie/6input-preimage corretti intatti. Nuovo contratto staticostabile per
queste nullcelle ed altre161istruzioni/associazionistato restano aperti: nessun
selector16→sprite46 universale, modello/risorsa o prodotto scripts×all-sprites.
Nessuna nuova fixture/navigazione/checkpoint/payloadedit/fullpair o ripetizione
vecchiaaccettazione. e61/USAe8invariati; Linux-USA-nativeWindows-32:9da fare,
nessuna misura performance/spazio.
[Query effettive e limiti](../rom_data_unit_obj.md#source-linked-script-query-observation).


**ROMDATA-01-SCRIPT-OBJ-SPECIAL-NORMAL-PAIRS completato come contratto fonte
condizionale EUR (2026-10-25).** CBD24confronta unit+4col selettore; CBAD4nonnull
unit+4>1eCF144legano il suo veroIDlogico metadata14al normalhalfword04;9B824lo
assegna ad actor+34prima dell'override42. Condizioni esplicite selezionateprima
dell'audit: attore mantiene assegnazionenormale della stessaunità, override20clear,
facing0..3. Alternate/42-230/high16/statoattore diverso restano separati, nessun
prodotto gruppi×tuttisprite o riga rimossa. Tutte151istruzionilow→20coppieselettore/
base14IDlogici→18coppie sprite/base12sprite/108query108celle76nulli/32sequenze241
frame/11OBJ64oggetti/95richieste pixel35968B. Audit56owner:4780initial+30586normal
condivisi e zeroaltrooverlap/conflitti/itemmisti/estensionivarianti. Nuovi33span
6436B/SHA6c3ef9c9..=[240index28header920frame5248pixel],60celle53nulle/sequenze7
frame46(tuttiop1)/17richiestepixel. Nullcelle37C760/64osservate incluse nel piano
ROMstabile, nessundatasetdipendentedal save.27halfword/1literal/3BLpins passano,
nessun build/testguest. Contratto01chiuso; prossimo02modello lossless con151refs/
20testimoni/108chiavi e solo nuovo oracoloWine60celle/7reset/17upload, poi03/04/05.
Vecchieaccettazioni/censimento6000nonripetuti; e61/USAe8invariati, Linux-USA-
nativeWindows-32:9da fare, nessun performance/spazio/fixture/navigazione/checkpoint.
[Contratto e limiti](../rom_data_unit_obj.md#conditional-normal-script-poses-for-special-unit-selectors).


**ROMDATA-02-SCRIPT-OBJ-SPECIAL-NORMAL completato EUR/Wine mirato (2026-10-25).**
Modello lossless20testimoni metadata14logici/151refs script/108chiavi con formati
fisici condivisi; valida initial e normal indipendentemente anche su intervalli
sovrapposti, sottraendone l'unione. Full41802B/SHAf64755ef.. e nuovi33span6436B/
SHA6c3ef9c9.. uguali al piano indipendente. Unica buildfrontendWindows senza
warning/frozen2eaa5653..; unico nuovo filtroWine firstPASS18guardie/120prefix/
53resetnull7proper/17scheduler-DMA3/all5248newpixelB/wholeVRAM. Nuoviframe tuttiop1,
nessun replay dei comandi2/6/8già provati; vecchie accettazioni/censimento6000chiusi.
Solo registrazioni moduli e nuovi file, e61/USAe8invariati. Restano managed/runtime/
actualCLI/ordinary e condizioni alternate/42-230/high16/statoattore; nuoveLinux/
USA/nativeWindows/32:9da fare. Nessuna misura performance/spazio. Dopo la chiusura
memoria, derivazione ordinary equipment poi conclusa con autorizzazione esplicita
ed unica coppiaFight500positiva; pipeline03script-special prosegue sotto.
[Modello e prove](../rom_data_unit_obj.md#typed-normal-script-special-pose-model).


**ROMDATA-03/04-SCRIPT-OBJ-SPECIAL-NORMAL managed/runtime completati nel
perimetro EUR/Wine mirato (2026-10-26).** e62-d61pubblica58file57risorse con
metadata14/scriptarchive/initial/normal,151refs20testimoni logici autentici;
tutti56record/file/coperture precedenti ed e61 conservati, USAe8-d7. Ownerimmutabile
serve solo33span6436B typed, condivisi initial/normal agliowner precedenti, senza
ROMshadow/capID/riscrittura task/ABI-save-native. Unica buildWindowscore/frontend
warning-free(core solo compilato); due nuovi filtriWine firstPASS23attachguardie+
6dipendenze/lifecycle/tutti6436typedB/sequenze7 e23percorsi originali-interprete-
compilato con stato-clock completo uguale. Part0proper/part1-2null/twoorientations/
fullwordrawID/liveRAM/two uploads-DMA/all33edges/mirror/BIOS/SWI-DMA passano;
nuoviAPI[0dir36index6header43frame0OBJ1pixel]B, misto2new4normal esatto e API
2new2raweditati. Tutti41802graphB avvelenati, vecchi comandi2-6-8/source/censimento/
scene non ripetuti. Datasetf131b439../manifestfile2b84c40c../modelfile02e7362e..
con identità distinte.04CLIproduzione reale poi concluso sotto; restano05ordinary
source/nuovepixel della famiglia e condizioni alternate/override/highregistry/
statoattore; Linux-USA-
nativeWindows-new32:9 da fare. Nessuna misura performance/spazio/nuovafixture.
[Pubblicazione e runtime](../rom_data_unit_obj.md#conditional-script-special-normal-managed-runtime).


**ROMDATA-04-SCRIPT-OBJ-SPECIAL-NORMAL-CLI completato EUR/Wine (2026-10-26).**
Unica buildproduzioneWindows warning-free/frozen6455958e..; canonico esattamente
ripristinato(hash+mtime), maincontent invariato/mtime-onlyCargoinvalidation.
ActualWine26817create/reuse58generation60managed/all58ugualiaccettati/no rewrite,
cwdestranea, generation-SHz1vX/f131b439.. . Un frameF09immutabile/vecchiacache
caricaowner mazero nuovibyte: solosmokecompatibilità, nessun05ordinary/nuovipixel/
fullpairclaim. Vecchieprove3530/500/census6000/source/native/CLI/scene non ripetute.
05source-use/newsequence/pixelordinary ancora aperto: Snow6000ha due celle nulle
nuove effettive, è un lead e non prova nuovipixel. Linux-USA-nativeWindows-new32:9/
altri casi fonte restano da fare; nessuna misura o nuova fixture/navigazione.
[CLI e limiti](../rom_data_unit_obj.md#conditional-script-special-normal-actual-production-cli).


**ROMDATA-05-SCRIPT-SPECIAL ordinary: locatornulli positivo, pixel ancora aperti
(2026-10-26).** Actuale62WineSnow6000immutabile legge48nuoviindexB e0nuove
sequenze/pixel; è usosorgente reale, non coppia completa o prova pixel. Nessun
censimento/guardbuild/fullpair ripetuto. Nuova lettura statica81snapshotEUR per
sette nuove sequenze non trova taskcompatibili. RIVERStart/A618già registrato
sul nuovo consumer legge0nuovibyte: negativo terminale, non ripetere. Contesto
attivo prima èprogram158/key168;159/key169contieneBAD829→2/group34 ma il trigger
non è dimostrato, interpretazioni fine/inizio battaglia restano provvisorie.
Un breve B44da copiaDS20EURautorizzata mostra solo menuPolly/duecheckpointprivati,
non vittoria/nuovepixel/nuova accettazioneDS20. Originali/cart invariati.
05nuovesequenze/pixel/equivalenza resta aperto; prossimo criterio indipendente
può essere contratto fonte script-special alternate, mantenendo il gate aperto.
NuoviLinux-USA-nativeWindows-32:9da fare, nessuna misura o codiceproduzione.
[Locator e limiti](../rom_data_unit_obj.md#script-special-ordinary-source-locators-and-remaining-pixel-gate).


**ROMDATA-01-SCRIPT-OBJ-SPECIAL-ALTERNATE-PAIRS completato come contratto fonte
condizionato EUR (2026-10-26).** Stessi151riferimenti low49/20witness logici,
metadata14half06sotto assegnazione alternate corrente originale/override20clear/
facing0..3. Quattro sprite188/231/232/234,14sprite-base pairs,84celle68null,
14sequenze64frame/quattro OBJlists20oggetti/26pixelrequests7232B. Tutti57owner
e62autenticati, nessun targetsharedroot/conflitto/mixeditem/extentsvariabili.
Nuovi20span816B=216index8header80frame512pixel,54celle52null/two sequences/four
op1frames/two pixelrequests; SHA029ff8a0.. . **ROMDATA-02-SCRIPT-OBJ-SPECIAL-ALTERNATE completato EUR/Wine mirato:** modello
tipizzato, buildWindows senza avvisi e unico nuovo filtroWine passano19guardie/
108prefix originali/52null+2proper reset/two DMA3uploads/interaVRAM512newpixelB.
Il primo avvioWine fallisce prima del test per lockserver nella sandbox; stessa
binary riavviata con accesso necessario, primo filtroeffettivoPASS/no rebuild.
Comandi2/8già coperti non ripetuti. Managed/runtime/
actualCLI/ordinary e high16/override/currentstate restano aperti; e62/USAe8
invariati, Linux/nativeWindows/new32:9da fare, nessuna misura performance/spazio.
[Contratto e modello](../rom_data_unit_obj.md#conditional-alternate-script-special-source-and-typed-model).


**ROMDATA-03/04-SCRIPT-OBJ-SPECIAL-ALTERNATE managed/runtime completati nel
perimetro EUR/Wine mirato (2026-10-26).** e63-d62pubblica59file58risorse con
metadata14/scriptarchive/initial/normal/alternate effettive, tutti57precedenti
filedati/resource records/coverage invariati/e62conservato/USAe8-d7invariato.
Ownerimmutabile serve solo20nuovi span816B da campi tipizzati; vecchiowner e
rawholes compongono ogni lettura. BuildWindows core/frontend senza avvisi,
core compilato soltanto; due nuovi filtriWine primoPASS24attach+7dependency
checks/lifecycle/all816typedB/two sequences e24percorsi original-interpreter-
compiled stato-completo/clock, part0proper/parts1-2null/facing/fullwordrawID/
liveRAMframe/uploads-DMA/all20edges/mirrors/BIOS-SWI-DMA/mixed2new4alternate+
2new2raw. Tutto9048Bgraph alterato, compresi owner condivisi; prove fonte/modello/
54celle/DMAfonte/comandi2-8precedenti non ripetute. ActualproductionCLI/ordinary
newpixel/equivalence e high16/42-230/currentactor restano aperti; nuoveLinux/
USA/nativeWindows/new32:9da fare, nessuna misura performance/spazio/nuova fixture.
[Pubblicazione e runtime](../rom_data_unit_obj.md#conditional-script-special-alternate-managed-runtime).


**ROMDATA-04-SCRIPT-OBJ-SPECIAL-ALTERNATE-CLI completato EUR/Wine (2026-10-26).**
Buildproduction Windows unico/senza avvisi/frozen36cba67d.., canonico ripristinato
hash+mtime esatti/maincontentunchanged. CLIreale Wine crea/riusa59generation/
61managedfiles, tutti59byte-identici al datasetaccettato, nessun hash/mtime
riscritto da cwd estranea. Oneframe autenticoF09/caricamento cachevecchia passa
conzero nuovi usi: compatibilitàsoltanto. Fonte/modello/managed/native/actualCLI
chiusi/no repeat. Ordinarynewpixel/fullscene, high16/42-230/currentactor e nuove
Linux/USA/nativeWindows/32:9da fare; nessuna misura performance/spazio/nuovafixture.
[CLI e limiti](../rom_data_unit_obj.md#conditional-script-special-alternate-actual-production-cli).


**ROMDATA-05-SCRIPT-SPECIAL-ALTERNATE-TWO-SEQUENCE-LEAD chiuso negativo
(2026-10-26):**81snapshotEURautenticati, zero taskenabled/dormant per le due
nuove sequenze AD4950/AD4980/18832-33, hash+mtimeoriginali invariati. Target
diversi dalle vecchie scansioni606alternate/settenormal, non ripetute. Nessun
Winegame/build/navigazione/nuovocheckpoint/fullpair selezionato; non riprovare
unchanged. Ordinarynewsequence/pixel/equivalence resta aperto; prossimo contratto
fonte indipendente override42/230o highregistry con witnessoriginali concreti.
[Lead e limiti](../rom_data_unit_obj.md#conditional-script-special-alternate-actual-production-cli).


**ROMDATA-01-SCRIPT-OBJ-OVERRIDE-PAIRS completato come contratto fonte EUR
condizionato (2026-10-26).** Original9B734flag20forza42normal/230alternate;
solo questi due ID e13gruppi effettivi da167script49refs:26mode-base witnesses/
334riferimenti condizionati/156celle126null/28sequenze140frame/8OBJlists27oggetti/
42pixelrequests8000B. Condizioni currentassignment originale/flag20set/facing0..3
prima dei target, nessuna identità staticalow/highregistry o allspriteproduct.
Tutti58owner e19halfword/twoBL/five code fingerprints passano, zero conflitti/
mixeditem/sharedroot/varyingextent. Nuovi40span1050B=384index24header360frame26OBJ
256pixel;96celle90null/six sequences18frames(16op1/twoop2)/one list4objects/two
pixelrequests. **ROMDATA-02-SCRIPT-OBJ-OVERRIDE completato EUR/Wine mirato:** modellooffline,
buildWindows senza avvisi e unico nuovo filtroWine primoPASS19guardie/all156keys/
192constructor aliases/90null+6proper resets/two newop2argumentcopies/two actual
DMAuploads256B/interaVRAM. e63/USAe8/runtime invariati.03managed/runtime/CLI/ordinary e highregistry/
currentactor/NewLinux-USA-nativeWindows-32:9aperti, nessuna misura performance.
[Contratto e modello](../rom_data_unit_obj.md#conditional-forced42-and230script-pose-source-model).


**ROMDATA-03/04-SCRIPT-OBJ-OVERRIDE managed/runtime completati nel perimetro
EUR/Wine mirato (2026-10-26).** e64-d63aggiunge unit-obj-script-override.json /
graphics.unit-obj-script-override con scriptarchive/normalbattle/alternatebattle
effettive:60file59risorse/tutti58precedenti dati-record-coverage/e63conservato/
USAe8-d7. Nuovoownerimmutabile solo40span1050B inclusi26OBJ e due nuove coppie
argomentiop2; originali selector/flag20/task/command/DMA/VRAM/save/nativeABI attivi.
Unica buildWindows core/frontend senza avvisi/core compilato soltanto; due nuovi
filtriWine primoPASS26attach+5dependency guards/lifecycle/all1050typedB/sixseq e
27percorsi original-interpreter-compiled stato-completo/clock/bothnewop2/newOBJ/
null/proper/liveRAM/two uploads+DMA/40edges/mirrors/BIOS-SWI-DMA/mixed2new4alternate+
2new2raw. Tutti11722graphB alterati; fonte/modello/192prefix/96cell/sourceDMA/
vecchieprove non ripetuti. CLIreale/ordinarynewpixel/equivalence e highregistry/
currentactor/newLinux-USA-nativeWindows-32:9aperti, nessuna misura o nuovafixture.
[Pubblicazione e runtime](../rom_data_unit_obj.md#forced-script-override-managed-runtime).


**ROMDATA-04-SCRIPT-OBJ-OVERRIDE-CLI completato EUR/Wine (2026-10-26).** Unica
buildproductionWindows senzaavvisi/frozen8a987e99.., canonicalhash+mtime ripristinati/
maincontentunchanged. CLIrealeWinecreate/reuse60generation62managed/all60byteuguali
accettati/no-write da cwd estranea/vecchiacache passa. OneframeautenticoF09 legge
zero nuovibyte, solosmokecompatibilità. Fonte/modello/managed/native/CLI chiusi/no
repeat. Ordinarynewsequence/pixel/equivalence e highregistry-currentactor/newLinux-
USA-nativeWindows-32:9aperti, nessuna misura o nuova fixture/navigazione/payload.
[CLI e limiti](../rom_data_unit_obj.md#forced-script-override-actual-production-cli).


**ROMDATA-05-SCRIPT-OVERRIDE-SIX-SEQUENCE-LEAD chiuso negativo (2026-10-26).**
81snapshotEURautenticati/zero taskenabled o dormant per42/(32,33,40,41) e230/(32,33),
tutti part0; hash+mtimeinput invariati. Targetdiversi dai vecchi606alternate/
settenormal/duealternate, non ripetuti. Nessun guest/build/navigazione/checkpoint/
payloadedit/fullpair, non riprovareunchanged. Ordinarynewsequence/pixel/equivalence
resta aperto mentre prosegue fontehighregistry/currentactor indipendente.
[Lead e limiti](../rom_data_unit_obj.md#forced-script-override-actual-production-cli).


**ROMDATA-01-SCRIPT-REGISTRY-SPECIAL-PAIRS completato come contratto fonte EUR
condizionato (2026-10-26).**16high49refs/sevenexactselector-basepairs, unitbyte4
nelle93righemetadata14note/sameunitcurrentCF144-CF180assignment/override20clear/
facing0..3 scelti prima dei target.1302witnesses2976refs/74sprite370sprite-base/
2220logicalqueries1908cells1537null/353sequenze2081frame/60OBJlists279objects/
841pixelrequests321088B. Tutti59owner e41halfword/fourliteral/eightBLpins passano,
nessun knownindexroot/conflict/mixeditem/varyingextent o querypruning. Nuovi272span
15276B=2392index84header1920frame10880pixel:598cells567null/21seq96op1frames/37pixel
requests, zero nuoviOBJ. Fonteonly chiuso; modello/Wine chiusi come sotto.
Managed-runtime-CLI-ordinary da fare, e64/USAe8invariati. Nessuna identità statica registry/genericjob/redirect/
outofcatalog/currentactor inventata, nessuna misura o guest/fixture/navigazione.
[Contratto e limiti](../rom_data_unit_obj.md#registry-selectors-and-known-special-unit-source-projection).


**ROMDATA-02-SCRIPT-REGISTRY-SPECIAL completato modello/oracolo EUR (2026-10-26).**
Modello lossless conserva1302witnesses2976refs/2220logicalkeys1908physicalcells/
353sequenze2081frame e tutti272nuovispan15276B. Sei visteprecedenti validate
indipendentemente prima della sottrazioneunione; aliasfisici identici coalesciati,
conflitti rifiutati. Buildfrontend iniziale E0308 test-only corretto con lookup
nel tipoinitial; retry warning-free. Primo filtroWine passa26guards/1196alias
costruttore/567null+21properreset/37scheduler-DMAuploads10880B/interaVRAM. Nessun
vecchio comando/source/managed/native/scene replay.03managed/runtime, CLIreale,
ordinarynewpixel/fullscene e genericjob/outofcatalog/redirect/changedactor/global
registryreachability restano aperti; Linux/USA/nativeWindows/new32:9 da verificare.
Nessuna misura performance/spazio o nuova fixture/navigazione/payload edit.
[Modello e limiti](../rom_data_unit_obj.md#typed-registry-knownspecial-script-pose-model).


**ROMDATA-03-SCRIPT-REGISTRY-SPECIAL completato managed/runtime EUR (2026-10-26).**
e65-d64pubblica61file60risorse,59precedenti/e64preservati; metadata14/archive e
sei visteprecedenti sono dipendenze reali. Owner tipizzato solo272nuovispan15276B,
byte condivisi/holes/raw/originaltasks-DMA preservati. Un buildWindowscore/frontend
warning-free e primi dueWine mirati passano33attach+10dependencyguards/lifecycle/
15276typedB21seq e25percorsi original-interpreter-compiled fullstate-clock con
373842Brawgraph alterati.272bordi/width-mirrors/realBIOS-bus/SWI-DMA/liveRAM e
mixed2new4alternate+2new2editedraw passano; shared4B per ognuno dei6owner, nessun
oldcommand/source/previousacceptance replay. CLIreale/ordinarynewpixel/fullscene
restano separati, nessun wholeactor/allocatorABI. Genericjob/outofcatalog/redirect/
changedactor/globalregistryreachability e Linux/USA/nativeWindows/new32:9 aperti;
nessuna misura performance/spazio o fixture/navigazione/payload edit.
[Consumo e limiti](../rom_data_unit_obj.md#managed-registry-knownspecial-poses-and-typed-guest-reads).


**ROMDATA-04-SCRIPT-REGISTRY-SPECIAL-ACTUAL-CLI completato EUR (2026-10-26).**
Buildproduzione una volta warning-free/frozen661c5af7..; canonicale ripristinato
hash+mtime esatti/maincontenutoinvariato-mtime-onlyCargoinvalid. Primi CLIWine
prepare/reuse da cwddiversa:61generation63managed/60risorse/eightdeps/all61file
ugualiaccepted/reusehash-mtimeinvariati. OneframeauthenticF09oldcacheattach passa,
nuovoconsumo0/smokeonly; ordinarynewsequence/pixel/fullsceneequivalence aperti.
Nessuna vecchia scena/prova source-runtime/fixture/navigazione ripetuta, nessuna
misura performance/spazio. Linux/USA/nativeWindows/new32:9 e globalregistry/
genericjob/outofcatalog/redirect/changedactor rimangono da verificare.
[CLI e limiti](../rom_data_unit_obj.md#registry-knownspecial-actual-production-cli).


**ROMDATA-05-SCRIPT-REGISTRY-SPECIAL-21-SEQUENCE-LEAD chiuso negativo (2026-10-26).**
Nuovo scanreadonly81snapshot autenticati/21nuovesequenze (nuovibindingtuttipart0)
trova0taskoriginali attivi o dormienti; header/payload/ROM e hash+mtime di tutti81
input prima/dopo passano. Vecchi606alternate/sevennormal/twoalternate/sixoverride
scan non ripetuti. Nessun guest/build/fixture/navigazione/payloadedit/fullpair;
leadchiuso/no retryinvariato. Ordinarynewsequence/pixel/fullscene resta aperto
mentre prosegue la fonte indipendente highregistry generic-currentassignment.
[Lead e limiti](../rom_data_unit_obj.md#registry-knownspecial-actual-production-cli).


**ROMDATA-01-SCRIPT-REGISTRY-GENERIC-DIRECT completato contratto fonte EUR (2026-10-26).**
16highrefs/sevenpairs, unit+4=0/1/sameunitcurrentassignment/override20clear/facing0..3,
jobunit+5/alternate+7/property4-5/directroute0 autenticati prima dei target con
83halfword/6literal-dispatch/11BLpins. Tutte116righe conservate:100bothconcrete/
10normalonly/2bothFFFF/4routeFF;110normal100alternateconcrete→2940witnesses6720refs.
Ipotesi110joint iniziale corretta prima di leggere target;2normal12alternateFFFF
zeroextendu16 senza fallback inventato. All60e65owner autenticati prima rootaudit.
107sprite535pairs/3210keys3210cells2420null/777properseq5093frames/45OBJlists231objects/
1895pixelrequests677280B.12target sono indiciinitial48..59con witnesses exactdirectory/
firstcell, nessuncountheuristic. Full796992B/shared697168B/nuovi659span99824B=
5544index504header11760frame82016pixel:1386cells1260null/126newseq588op1/268newpixel,
no nuoviOBJ/mixeditem/roleconflict/extentvariabile. Modello/Wine/managed-runtime/
CLI/ordinary da fare; e65/USAe8invariati. RedirectFFFFoutofcatalog/currentchanged/
registryidentity e Linux/USA/nativeWindows/new32:9 separati. Nessun guest/build/
fixture/navigazione o misura performance/spazio.
[Contratto e limiti](../rom_data_unit_obj.md#registry-selectors-and-generic-direct-job-source-projection).


**ROMDATA-02-SCRIPT-REGISTRY-GENERIC completato modello/oracolo EUR (2026-10-26).**
Modello lossless conserva2940witnesses6720refs/107roots3210keys/777seq5093frames,
12esplicititargetinitialroot48..59 e tutti659newspan99824B. Sei visteprecedenti
validate indipendentemente prima della sottrazioneunione; metadata52/archive
dipendenze reali,116righe/sentinel/redirect mantenute nella fonte/metadata esistente.
Primo buildfrontendWindows warning-free/frozenbd8727ee..; primo filtroWine passa
34guards/2772newcell facingaliases/1260null126proper resets/268scheduler-DMAuploads
82016B/interaVRAM. Nessun oldcommand/source/managed/native/scene replay; no native
complete-stateclock/wholeactorABI claim.03managed/runtime/CLI/ordinarynewpixel e
FFFF/redirect/outofcatalog/changedactor/globalidentity aperti; Linux/USA/native
Windows/new32:9 da verificare. Nessuna misura performance/spazio o nuova fixture/
navigazione/privatepayload edit; e65/USAe8invariati.
[Modello e limiti](../rom_data_unit_obj.md#typed-registry-generic-direct-job-model).


**ROMDATA-03-SCRIPT-REGISTRY-GENERIC completato managed/runtime EUR (2026-10-26).**
e66-d65pubblica62file61risorse/all60precedenti-e65preservati. Metadata52/archive e
sei visteprecedenti sono dipendenze reali; owner tipizzato solo659newspan99824B,
12initialrootviews/sharedbytes/holes/raw/originaltasks-DMA preservati. Un build
Windowscore/frontend firstPASSwarning-free e primi dueWine passano36attach+
10dependencyguards/lifecycle/99824typedB126seq e25percorsi original-interpreter-
compiled fullstate-clock con796992Brawgraphalterati.659bordi/width-mirrors/realBIOS-
bus/SWI-DMA/liveRAM/mixed2new4alternate+2new2editedraw passano;4sharedB per ognuno
seiowner, nessun oldcommand/source/previousacceptance replay. CLIreale/ordinary
newpixel/fullscene, FFFF/redirect/outofcatalog/changedactor/globalregistryidentity
restano aperti; nessun wholeactor/allocatorABI. Linux/USA/nativeWindows/new32:9
da verificare, nessuna misura performance/spazio o fixture/navigazione/payloadedit.
[Consumo e limiti](../rom_data_unit_obj.md#managed-registry-generic-direct-job-poses-and-typed-guest-reads).


**ROMDATA-04-SCRIPT-REGISTRY-GENERIC-ACTUAL-CLI completato EUR (2026-10-26).**
Primo buildproduzioneWindows warning-free/frozend5092be6..; canonicale ripristinato
hash+mtimeesatti/maincontenutoinvariato-mtime-onlyCargoinvalid. Primi CLIWine
prepare/reuse da cwddiversa:62generation64managed/61risorse/eightdeps/all62file
ugualiaccepted/reusehash-mtimeinvariati. OneframeauthenticF09oldcacheattach passa,
nuovoconsumo0/smokeonly; ordinarynewsequence/pixel/fullsceneequivalence aperti.
Nessun oldsource/model/runtime/sceneproof replay, fixture/navigazione/privatepayload
edit o misura performance/spazio. Linux/USA/nativeWindows/new32:9 e FFFF/redirect/
outofcatalog/changedactor/globalregistryidentity rimangono da verificare.
[CLI e limiti](../rom_data_unit_obj.md#registry-generic-direct-job-actual-production-cli).


**ROMDATA-05-SCRIPT-REGISTRY-GENERIC-126-SEQUENCE-LEAD chiuso negativo (2026-10-26).**
Nuovo scanreadonly81snapshot autenticati/126nuovepropersequenze (nuovibindingtutti
part0/bodylink+44) trova0taskbody originali attivi o dormienti. Currentpointer
inside/atend/outside/misaligned conservato senza filtro/fineframe pruning. Modello/
fonte/bodylinkcode e header-payload-ROM/hash+mtime di tutti81input prima/dopo passano.
Vecchi21knownspecial/sixoverride/sevennormal/twoalternate/606alternate scans non
ripetuti. Nessun guest/build/fixture/navigazione/privatepayload/fullpair; leadchiuso/
no retryinvariato. Ordinarynewsequence/pixel/fullscene resta aperto, proseguono
contratti indipendenti highregistry routeFF/altjob e altri casi rimasti. Nessuna
misura performance/spazio, Linux/USA/nativeWindows/new32:9 pending.
[Lead e limiti](../rom_data_unit_obj.md#registry-generic-direct-job-actual-production-cli).


**ROMDATA-01-SCRIPT-REGISTRY-REDIRECT-KNOWN-JOB completato come alias fonte EUR (2026-10-26).**
I quattro currentjob80/81/82/85routeFF reindirizzano alla riga alternatejob+7
concreta route0per property4normal/5alternate; non usano il campo sprite primario.
Dominio scelto prima dei target:16highrefs/sevenpairs/unitkind0-1/sameunit original
assignment/actualmode/override20clear/facing0-3. Tutte116righe/928classificazioni
effettive conservate:840concrete56FFFF32loop;110normal100alternate producono11760
witnesses26880refs.98halfword6literal11BLpins passano; r4resta il job alternativo,
CC35Aritorna alla stessa property. Tutte61risorse/fonti correnti autenticate prima
dei target, ogni2940witnessdiretto ha quattro preimmagini: stesse3210celle/777seq/
12initialrootviews/full796992B/setteowner, zero nuovi byte/risorse/revisioni.
Metadata52/e66Wine proofs riusate, nessun build/guest/fixture/navigazione o misura
performance/spazio. e66-d65/USAe8-d7invariati; ordinary/globalregistryidentity/
FFFF/nonreturning/outofcatalog/changedactor e Linux/USA/nativeWindows/new32:9 aperti.
[Contratto e limiti](../rom_data_unit_obj.md#registry-generic-jobs-redirected-to-a-known-alternate-job).


**ROMDATA-01-SCRIPT-REGISTRY-FFFF-READ completato come classificazione lettore EUR (2026-10-26).**
196witnessdiretti+784routeFF/2240refs conservano i risultatiFFFFnormal0/1 e
alternate0/1/46-50/64-65/69-71; dominio scelto prima dei target.71readerpins/
treblocchicodice/twoliterals verificano zeroextension/actoru16/fullworddirectory
senza guardFFFF. Proprietà083B4C12=DC86, directory083D4830=14AE3F1E:60query/
30word nella regione14unmapped, CPUallinea/ruota16 e MemMap dipende da livebios_open.
Nessuna sequenzaROM/nullcostante/nonritornouniversale, cap/fallback/pointerfix o
estensione inventati. Queste due lettureROM sono6Bsenza owner: rawfallback
preservato, migrazione6B e ordinaryreachability ancora aperti. e66-d65/USAe8-d7
invariati, nessun build/guest/fixture/navigazione/privatepayload/misura performance
o spazio. Linux/USA/nativeWindows/new32:9 e globalidentity/unknown/changedactor aperti.
[Classificazione e limiti](../rom_data_unit_obj.md#known-generic-job-ffff-results-and-raw-bus-reads).


**ROMDATA-01-UNIT-OBJ-PROPERTY-WORDS completato contratto fonte EUR (2026-10-26).**
221C4legge394C14+2*sprite e restituisce u16;9AD0Elo passa via r8al costruttore,
allocazione e task+16. Dominio prima dei target:398witnessnormal/alternate
metadata52/14concreti e override42/230,182sprite/query; redirectsconcreti già
equivalenti, nessuna equipment/body/globalidentity inferita.98halfword15BL/
fourcodeblocks/oneliteral e tutti61owner/file/source passano prima dei word.
Quattro intervalli364Bnon posseduti, SHAad328c8e..,139word16/43word24; rawu16/
buchi preservati, nessuna estensione248/IDcap/flagbitfield o unitàVRAM inventata.
Fontechiusa, modello/oracoloWine/managed-runtime-CLI-ordinary aperti; e66-d65/
USAe8-d7invariati. FFFFraw6B/unknown/changedactor/globalidentity e nuoviLinux-USA-
nativeWindows-new32:9 separati. Nessun build/guest/fixture/navigazione/misure.
[Contratto e limiti](../rom_data_unit_obj.md#actor-property-words-from-known-sprite-assignments).


**ROMDATA-02-UNIT-OBJ-PROPERTY-WORDS completato modello/oracolo EUR/Wine (2026-10-26).**
Modello lossless398witness/182word/4range364B, dipendenze metadata52/14effettive,
rawu16/buchi/identità preservati senza cap16/24. Primo buildWindowsfrontend senza
warning e primoWineesatto passano20guards/all182getters364B/3rawfullwordgetter,
quattroprefissi actor→newtask (dueoriginali/duehighword sintetici), duereuseprefixes
e quattrosetter originali u16. Tasknuovi ricevono proprietà, esistenti conservano
proprio+16; interaRAM fuori scratch/destinazioni e VRAM/OAM/palette passano.
82build83Winehash-mtimeguards/11compiledinputs invariati; nessun vecchio oracolo
ripetuto/fullallocator-constructor/native-clock/ordinary claim. e66-d65/USAe8-d7/
produzione invariati, managed-runtime-CLI-ordinary ancora aperti. Nessuna fixture/
navigazione/privatepayload/misura performance/spazio; Linux/USA/nativeWindows/
new32:9 e FFFFraw6B/unknown/changedactor/globalregistry separati.
[Modello e limiti](../rom_data_unit_obj.md#typed-actor-property-words-and-original-reader-validation).


**ROMDATA-03-UNIT-OBJ-PROPERTY-WORDS completato managed/runtime EUR (2026-10-26).**
e67-d66pubblica63file62risorse con metadata52/14dipendenze reali e tutti61file/
record precedenti-e66preservati. Ownerimmutabile solo4span364B/182word,398witnesses,
3u64mask perrecord; vecchiowner/buchi112-117/FFFFraw6B e lavoro guest preservati.
Un buildWindowscore/frontend senza warning, primi dueWine passano17attach+
4fullsetdependencyguards/lifecycle/all364typedB182maskrecord e16percorsi
original-interpreter-compiled fullstate-clock con364rawB alterati. Quattrobordi/
width-mirrors/realBIOSbus/rawfullword/liveRAM/SWI-DMA passano; API2new2old/2new2raw,
CpuSet4new2old/FastSet28new4old/DMA4new2old esatti.84build86Winehash-mtimeguards/
20compiledinputs invariati, nessun vecchio source/model/familyproof ripetuto o
fullallocator/wholeactor/ordinary claim. CLIproduzione/ordinarypositivo, raw6B/
unknown-changedactor/globalidentity e Linux/USA/nativeWindows/new32:9 aperti.
USAe8-d7invariato, nessuna gameplayfixture/navigazione/privatepayload o misura
performance/spazio.
[Consumo e limiti](../rom_data_unit_obj.md#managed-actor-property-words-and-typed-runtime-reads).


**ROMDATA-04-UNIT-OBJ-PROPERTY-WORDS-ACTUAL-CLI completato EUR/Wine (2026-10-26).**
Primo buildproduzioneWindows senza warning/frozen2ce56e03.., canonicale ripristinato
hash+mtimeesatti/maincontenutoinvariato-mtime-onlyCargo invalidation. PrimiCLIWine
prepare/reuse da cwdiversa:63generation65managed/62risorse/twodepsmetadata52-14/
all63accepted/reusehash-mtimeinvariati. Un frame da F09autentico immutabile carica
vecchiocachenativo e allega la risorsa, nuovi contatori bus/source/word tutti0:
smokeonly, ordinarypositivo/fullsceneequivalence ancora aperti.154activeCLIguards/
20compiledinputs invariati, nessuna ripetizione source/model/runtime/scene accettata,
gameplayfixture/navigazione/privatepayload o misura performance/spazio. USAe8-d7,
Linux/USA/nativeWindows/new32:9 e raw6B/unknown-changedactor/globalidentity separati.
[CLI e limiti](../rom_data_unit_obj.md#actor-property-words-in-the-actual-production-cli).


**ROMDATA-05-UNIT-OBJ-PROPERTY-WORDS-ORDINARY completato nel perimetro EUR/Wine dichiarato (2026-10-26).**
Una nuova CLIordinary F09Wait3530sugli input/checkpoint/cache già autenticati usa
10letture/20Baggregati/dueword del nuovoowner. Solo un guard/reportpositivo opzionale
aggiunto al validatore generico; primo buildfrontendWindows senza warning/frozen
a3e255e6.. . Primo fullpairWine con solonuovoflag passa ogniframestate-clock/
central-widebuffer/OAM/dispatch/mixed-PSG-FIFO-HLEaudio; conteggi10/20/2 esatti come
CLIproduzione2ce56e03../generation-gpXTBA/e67.155activehash-mtimeguards/21compiled
inputs invariati, altre180word mantengono copertura source-reader mirata. Vecchio
normal-battle3530child resta chiuso: stessiinput per nuovocriteriopropertyword,
nessun source/model/runtime/oldfamily/costproof ripetuto o gameplayfixture/
navigazione/privatepayload/misura performance-spazio. Scenadichiarata chiusa,
non wholeallocator/actorABI/allwordordinary/new32:9; Linux/USA/nativeWindows,
raw6B/unknown-changedactor/globalidentity e altre famiglie restano aperti.
[Scena e limiti](../rom_data_unit_obj.md#ordinary-actor-propertyword-use-and-scene-equivalence).


**ROMDATA-01-UNIT-OBJ-PROPERTY-GETTER-CALLERS completato come contratto fonte EUR (2026-10-26).**
I cinque BL già censiti sono autenticati; ricostruiti i quattro percorsi mancanti
con 680 halfword, 25 BL e 21 literal/control-flow pin. I due chiamanti u16 usano
233A8/CF144 o 9B734 e aggiornano la proprietà con 22DF4 conservando l'allocazione;
le assegnazioni concrete note riusano le 182 parole esistenti. I due chiamanti
UI caricano context+1085 con LDRB e confrontano il risultato con 16 per scegliere
un descrittore. Builder, troncamenti e condizioni tag/animazione sono verificati.
Prima dei target è selezionata la finestra completa dei 256 argomenti byte,
senza dedurre un catalogo256, un cap del getter o reachability ordinaria universale.
Tutti62 owner/file/fonti autenticati:184 query alias (182 proprietà + due parole
nei quattro byte palette) e72 parole/144B nuovi in quattro intervalli disgiunti,
SHA9d1239cc.. . Entrambe le viste condivise sono ricostruite; nessuna query scartata
o duplicazione. Sourcechild chiuso; modello lossless72word e oracolo Wine sono
chiusi nel punto seguente. Runtime/CLI/ordinary restano da fare.
Nessun build/guest/fixture/navigazione/misura; e67-d66/USAe8-d7 invariati.
FFFFraw6B, u16 ignoti/changedactor/globalidentity/computedindirect e Linux/USA/
nativeWindows/new32:9 restano separati e aperti.
[Chiamanti e limiti](../rom_data_unit_obj.md#additional-property-getter-callers-and-the-ui-byte-input-window).


**ROMDATA-02-UNIT-OBJ-BYTE-PROPERTY-WORDS completato modello/oracolo EUR/Wine (2026-10-26).**
Nuovo modello lossless delle sole72word/144B; identità inputbyte/source/u16,
quattro intervalli e JSON rigorosi. Le due dipendenze propertywords/colorpalettes
forniscono184word senza copie duplicate: copertura esatta della finestra byte256.
Un buildfrontendWindows senza warning e un nuovo filtro Wine al primo tentativo
passano14guard,512prefissi UI originali/tutti512B letti,otto esclusioni senza
getter,quattro confronti u16 sintetici,quattro writer lowbyte e sei prefissi dei
due setter aggiuntivi. RAM intera fuori scratch/destinazioni e VRAM/OAM/palette
verificati.109build110Wineguard/13compiledinput invariati, modelloSHAac6a296a..
Nessun vecchio test ripetuto o fullUI/allocator/native-clock/ordinary claim.
Source/modello chiusi; managed-runtime/CLI/ordinary nuovi restano pendenti,
e67-d66/USAe8-d7 invariati, Linux/USA/nativeWindows/new32:9 da verificare.
Nessuna fixture/navigazione/privatepayload edit/misura performance-spazio.
[Modello e limiti](../rom_data_unit_obj.md#typed-ui-byte-property-words-and-original-caller-validation).


**ROMDATA-03-UNIT-OBJ-BYTE-PROPERTY-WORDS completato managed/runtime EUR (2026-10-26).**
e68-d67 pubblica64file/63risorse, mantiene tutti62record/63file precedenti ed e67;
USAe8-d7 invariato. Dipendenze reali propertywords/colorpalettes conservano184word
condivise; nuovo owner solo72word/144B/quattro intervalli, bitmap per record.
Un buildWindowscore/frontend senza warning; due nuovi filtri Wine al primo
avvio passano17attachguard/quattro dependencyguard, create/reuse/repair/move/
interruptedpublication, failed-attach owner+fullstate isolation e lifecycle con
directory indisponibile. Tutti144B raw alterati restituiscono byte tipizzati,
tutti72record usati. Original/interpreter/compiled: stato completo e clock uguali
su getter, UI-prefix, fullword/liveRAM/mixed reads. Quattro bordi, width/mirrors/
realBIOSbus/SWI/DMA verificati. API2new2property/2new2palette/2new2raw; CpuSet
2new4property/FastSet28new4property/DMA4new2palette. Source/modello/oldproof chiusi
senza replay; produzioneCLI e ordinary nuovi ancora da fare. Linux/USA/
nativeWindows/new32:9 e fullUI/allocator/actorABI non dichiarati; nessuna fixture/
navigazione/privatepayload/misura performance-spazio/Android/package/commit.
[Runtime e limiti](../rom_data_unit_obj.md#managed-ui-byte-property-words-and-typed-runtime-reads).


**ROMDATA-04-UNIT-OBJ-BYTE-PROPERTY-WORDS-ACTUAL-CLI completato EUR/Wine (2026-10-26).**
Primo buildproduzioneWindows senza warning/frozen0c0f5082..; canonicale ripristinato
hash+mtime esatti, main invariato nel contenuto/solo mtime per invalidare Cargo.
Primi CLI Wine prepare/reuse da cwdiversa passano64generation66managedfile/
63risorse/due dipendenze reali/tutti64 uguali al dataset03/riuso senza scritture.
Un frame F09 autentico carica vecchiocachenativo e usa1lettura/2Baggregati/1word
nuova: positivo nel controllo di allegato, equivalenza ordinary completa ancora
aperta.182activehash-mtimeguard/23compiledinput invariati; nessun vecchio oracolo/
scenechild ripetuto, fixture/navigazione/privatepayload/misura performance-spazio.
e68-d67/USAe8-d7, Linux/USA/nativeWindows/new32:9 e altre famiglie separati.
[CLI e limiti](../rom_data_unit_obj.md#ui-byte-property-words-in-the-actual-production-cli).


**ROMDATA-05-UNIT-OBJ-BYTE-PROPERTY-WORDS-ORDINARY completato nel perimetro EUR/Wine dichiarato (2026-10-26).**
F09Wait3530 riusa checkpoint/input/cache autentici per il nuovo owner e68; i
vecchi scenechild normal/e67property restano chiusi. CLIoneframe già positiva:
nessun secondo locatorCLI3530. Solo cinque righe di guard/report opzionale aggiunte
al validatore generico; primo buildWindows senza warning e primo fullpairWine
passano ogni frame stato completo/clock/central-widebuffer/OAM/dispatch/mixed-PSG-
FIFO-HLEaudio, tutti cinque i flussi con campioni effettivi. Nuovo uso
476letture/952Baggregati/4word; altre68word con prove lettore mirate.
183sceneactiveguard/24compiledinput invariati. Nessuna ripetizione source/model/
runtime/CLI/oldscene/costproof o fixture/navigazione/privatepayload/misura.
Scenadichiarata chiusa; fullUI/allocator/actorABI/allwordordinary/new32:9 e
Linux/USA/nativeWindows, unknown/changedactor/globalidentity/computedindirect/
raw6B e altre famiglie restano aperti.
[Scena e limiti](../rom_data_unit_obj.md#ordinary-ui-byte-property-use-and-scene-equivalence).


**ROMDATA-01-UI-PROPERTY-OBJ-LIST-SOURCES completato contratto fonte EUR (2026-10-26).**
Riutilizzati candidati CFG già registrati e prove getter/UI concluse. Quattro
rami condizionali della proprietà u16==16 portano A4/B2→2AE26/1F6C→1500;
48halfword/quattro literal/quattro BL verificano controllo, puntatore e stride6.
Tutti63owner/file/fonti autenticati prima dei target: A4 conta2 oggetti e produce
14B nuovi disgiunti, SHA1f9cfc55..; B2 conta2/14B e coincide col modello già posseduto,
nessuna duplicazione. Bit packed/coordinate ricostruiti. Due riferimenti A4 e
un alias B2 aggiungono tre record: proiezione126refs/74liste175oggetti1198B,
produzione ancora123refs/73liste/e68-d67. Sourcechild chiuso; estensione modello e
pubblicazione/runtime vanno accoppiate con cambio revisione perché Catalog alimenta
Extraction direttamente. CLI/ordinary A4 ancora da fare, nessuna nuova famiglia
necessaria. Nessun broadCFG/vecchio oracolo/build/guest/fixture/navigazione/misura.
Linux/USA/nativeWindows/new32:9 e famiglie/indiretti globali restano aperti.
[Contratto](../rom_data_obj.md#property-selected-ui-list-source-extension).


**ROMDATA-02-03-UI-PROPERTY-OBJ-LIST-EXTENSION completato EUR (2026-10-26).**
e69-d68 integra tre riferimenti nello stesso Catalog/List/Object/owner/formato,
126refs/74liste175oggetti1198B; nuova A4 solo14B/twoobjects, B2 condivisa.
Stessi64file/63risorse e dipendenza dimensioni; tutti73record precedenti,
62altri file/record ed e68 invariati. Build Windows corretto senza warning
(primo errore solo costante test non qualificata). Due primi filtri Wine passano:
roundtrip/otto guard/gestione pubblicazione-riparazione-spostamento/lifecycle;
due draw original-interpreter-compiled fullstate-clock/709istruzioni/4dispatch/
36compiledB, due host publication, bordi-width-mirror-BIOS e mixedAPI2new2B2/
DMA-CpuSet14new2B2/FastSet4prior14new14B2. Runtime handler invariati.
77build79Wineguard/29compiledinput esatti. CLI reale04 e ordinaryA4guest-host05
restano da fare; USAe8-d7/Linux/nativeWindows/new32:9 pendenti. Nessun replay
vecchie prove/scene, fixture/navigazione o misura performance-spazio.
[Integrazione e limiti](../rom_data_obj.md#managed-ui-property-list-extension-and-typed-consumption).


**ROMDATA-04-UI-PROPERTY-OBJ-LIST-CLI completato EUR/Wine (2026-10-26).**
Primo buildproduzione Windows senza warning/frozen17488009..; canonicale ripristinato
hash+mtime, main contenuto invariato. Primi prepare/reuse/oldcacheoneframeWine da
/tmp passano64generation66managed/63risorse/74span1198B/tutti64uguali al dataset03,
riuso senza scritture. 148activeCLIguard/29compiledinput esatti.
OrdinaryA4sceneequivalence ancora aperta; nessun vecchio oracolo/scena ripetuto,
fixture/navigazione/privatepayload o misura performance-spazio. Altre piattaforme
restano da fare.
[CLI e limiti](../rom_data_obj.md#ui-property-lists-in-the-actual-production-cli).


**ROMDATA-05-UI-PROPERTY-OBJ-LIST-LOCATORS: osservazioni negative chiuse, ordinary aperto (2026-10-26).**
F09oneframe/Partyunit320 non usano A4. Un solo locator banca120 su copiaEUR
esatta autorizzata più normaleDown usa B2guest-host, A4assente; endpoint ispezionato,
151guard/29compiledinput esatti. Nessun fullpair o ripetizione allungata/altri slot.
Quattro snapshot letti conchecksum e prime7righe visibili→proprietà24; nessuna
portata globale inferita. Nuovi setter02AE70(input/animation/tag),02AF14(animation),
02AF5C(solo stato) distinti. Prima censusBL per due nuovi target:4updater/8setter;
prefisso0578EE assegnaanimation24 al preview umano. Percorso completo e witness
ordinaryA4guest-host/fullscene ancora aperti; leggere controllo prima di altroinput.
Vecchi oracoli/CFG/scene chiusi, nessun nuovo build/fixturegiocata/misura; Linux/
USA/nativeWindows/new32:9 da fare.
[Locator e prossimo criterio](../rom_data_obj.md#ordinary-ui-property-list-locators).


**ROMDATA-05-UI-PROPERTY-OBJ-LIST-HUMAN-ANIMATION completato per la scena dichiarata EUR (2026-10-26).**
Join originale25halfword/6switchword: valutatoreD2D44→6375C→1092→063230/02AF14
imposta24 per risultati1/2, tag1+proprietà16 seleziona A4. Primo CLI WineDispatch1840
positivo A4guest-host/15evaluationB;1400negativo conservato perché fermava prima
dell'evento1600, confine corretto giustificato da prova originale1840 già positiva.
Primo buildWindows test senza warning/frozen427845f2..; solo5righe guard/report
opzionale A4 nel validator, runtimeprodotto invariato. Primo fullpairWine1840 passa
tutti stati-clock/central-wide/OAM/dispatch/tutti audio e nuovo gateA4guest-host;
5stream con campioni reali.152guard/29compiledinput esatti; originali/input/copia
privata autentici, nessun cartedit o checkpoint derivato nuovo. Due preflightprivati
corretti prima di Wine, nessun guestretry. 01/02/03/04/05 dichiarati chiusi; vecchi
oracoli/scene e locatornegativi non ripetuti. Linux/USA/nativeWindows/new32:9,
fullUI/indiretti globali/altre famiglie restano aperti; nessuna misura performance-
spazio. [Scena e limiti](../rom_data_obj.md#positive-dispatch-use-and-scene-equivalence).


**ROMDATA-02-UNIT-OBJ-FFFF-READER-FIELDS completato modello EUR (2026-10-26).**
Nuovo modello lossless: query65535, wordu16proprietà+wordu32directory encoded,
due range6B,980witnesscondizionali/2240riferimenti (196diretti784redirect) con
dipendenze metadata52/archive49. Modelli validi precedenti e aliasbyte invariati;
nessun root/sequenza/puntatore corretto/cap interpretato. Primo buildWindows senza
warning/frozen83ffcd75.. e primo singolo filtroWine passano13guard/3numericroundtrip/
980record/6B e due lettori originali completi28istruzioni, registri/RAMguard.
156build157Wineguard/35compiledinput esatti. Classificazione71pin/60query e vecchie
prove A4/modello/runtime/scene chiuse senza replay. Pubblicazione/ownerruntime03,
nativeclock/ordinary pendenti; produzione e69-d68/64file63risorse e USAe8-d7 invariati.
Linux/USA/nativeWindows/new32:9 da fare, nessuna fixture/navigazione/misura performance-
spazio. [Modello e limiti](../rom_data_unit_obj.md#typed-model-for-known-ffff-reader-fields).


**ROMDATA-03-UNIT-OBJ-FFFF-READER-FIELDS completato managed/runtime EUR (2026-10-26).**
e70-d69:65file64risorse, tutti63record/file decodificati precedenti ed e69
conservati; USAe8-d7 invariato. Nuova risorsa numerica con dipendenze reali
metadata52/scriptarchive, modello980witness2240refs invariato. Owner immutabile
serve solo2Bproperty+4Bdirectory da campi tipizzati; gate3B4/3D4, contatori e due
bitcampo. Puntatore completo, BIOSaperto vivo/unalignedLDRrotate16 e controllo
guest preservati, nessuna sequenza/target/null/cap inventata. Primo buildWindows
core/frontend senza warning e primi due nuovi filtri Wine PASS:20attachguard,
quattro dependencyguard, publication/lifecycle directory-indisponibile, tutti6B
raw alterati/duecampi e vecchioowner distinto. Original/interpreter/native14path
stato completo/clock uguali: getter/directory/zeroearlyreturn/treBIOSprefix,
liveRAM/mixed reads. Due bordi/width/mirrors/realBIOSbus; API2new2raw, CpuSet
2new6raw/FastSet4new28raw/DMA4new4raw preservati. Source/modello/A4/oldproof chiusi
senza replay. ProduzioneCLI04 e ordinary05 restano da fare, nessuna fullsprite/
allocatorABI dichiarata. Linux/USA/nativeWindows/new32:9 pendenti. Nessuna fixture/
navigazione/privatepayload/performance-spazio/Android/package/commit/push.
[Consumo e limiti](../rom_data_unit_obj.md#managed-ffff-reader-fields-and-typed-bus-consumption).


**ROMDATA-04-UNIT-OBJ-FFFF-READER-FIELDS-ACTUAL-CLI completato EUR/Wine (2026-10-26).**
Primo buildproduzioneWindows senza warning/candidato congelato; canonicale hash+
mtime esatti ripristinati/main contenuto invariato. Prime tre chiamate Wine
prepare/reuse/oldcache-oneframe da cwd estranea passano65generation67managed/
64risorse/tutti65uguali al dataset03/due dipendenze reali/riuso senza scritture.
Checkpoint F09 originale e cache precedente integri: solo prova attachment,
ordinarypositive/equivalenza non dichiarate. Source/modello/runtime/oldscene
chiusi senza replay; USAe8-d7 e Linux/nativeWindows/new32:9 da verificare.
Nessuna fixture/navigazione/privatepayload/performance-spazio/Android/package/
commit/push. [CLI e limiti](../rom_data_unit_obj.md#actual-production-cli-for-ffff-numeric-fields).


**ROMDATA-05-UNIT-OBJ-FFFF-READER-FIELDS-EXISTING-CONTEXT-LEAD chiuso, ordinary positivo aperto (2026-10-26).**
Nuovo locator readonly sugli81checkpoint EUR esistenti: predicato actor+34FFFF/
unitkind0-1/currentjob-alternatejob da980witness, entrambe modalità/override/
null-dormant-outside conservati, senza richiedere targetsequenza inesistente.
159candidati potenziali in61snapshot, tutti kind0/job0/alt0/override20clear:
157bodypointer null e due01480168fuoriEWRAM. Nessun taskcorpo disponibile inEWRAM,
nessuna identità registry/sameassignment/actorvalidity dedotta. Non è prova di
irraggiungibilità globale e non fornisce scena positiva: ordinaryreader/fullpair
restano aperti. Hash/mtime/header/payload/ROM/input/modello/closedcontract esatti.
Source/modello/runtime/CLI/126sequence/oldscene chiusi senza replay; nessun nuovo
build/Wine/gameplay/fixture/navigazione/privatepayload/performance-spazio.
Linux/USA/nativeWindows/new32:9 pendenti. [Locator e limiti](../rom_data_unit_obj.md#existing-checkpoint-locator-for-ffff-numeric-readers).


**ROMDATA-01-UI-JOB-LOOKUP-SOURCE completato per proiezione EUR20slot (2026-10-26).**
Root399750 condiviso da sei lettori D0538/D0548/D0568/D0588/D05A8/D05C8: bytejob
oppureCBEC8property1/4/6/9/0, troncamenti e namehelper165E8 originali. Dominio
selezionato prima dei dati dal12CA98: lista indici dei20slotRAM02002E78/stride16,
mode3massimo+5/tieinclusivo; un singolo slot occupato consente ogni0..19. R6
preservato attraverso CFF70clear prima di D05C8.268halfword/27criticalpin/9literal/
8BL autenticati. Nessun capglobale/extentfisicocompleto/adjacency/identitàdi gioco
inventato; altricallermatch raw non automaticamente codicesemantico. Venti job
u8/20B/unarange SHA969b2cd5.. distinti,20righemetadata52 ricostruite, disgiunti
tutti64owner attuali. Nessun target sprite seguito. Sourcechild chiuso; modello/
Wineoracle/managed/runtime/CLI/ordinary aperti. e70-d69/65file64risorse e USA8-7
invariati; nessun build/guest/test/fixture/navigazione/privatepayload/performance-
spazio/Android/package/commit/push. Linux/USA/nativeWindows/new32:9 pendenti.
[Contratto e limiti](../rom_data_gameplay.md#twenty-slot-job-lookup-source-contract).


**ROMDATA-02-SLOT-JOB-LOOKUP-MODEL completato modello EUR (2026-10-26).**
Modello strict20(slot,u8job)/20B/sourcebase/hash/byte_at/metadata52diretto; Catalog
verifica EUR/treblockproducer-reader. Due solemoduledecls, nessuna registrazione
Extraction/runtime: e70-d69/65file64risorse/USA8-7 invariati. Primo buildWindows
frontend senza warning e primo unico filtro Wine PASS:20entry indipendenti/20B/
12guard JSON-shape-source-dependency/roundtrip/disgiuntitutti64owner.123percorsi
originali:21barejob/81propertywrappercompleti/21property0prefix prima di165E8,
trealiasinputhigh. Tutti20B ai sei LDRB originali, callee/fullRAMfuori20Bstack/
VRAM-OAM-palette preservati. Nessun capguest20/null/hostfallback, codeparser268pin/
metadata52/FFFF/A4/oldproof non ripetuti. Managed/runtime03/CLI04/ordinary05 aperti;
nessuna nuova fixture/navigazione/privatepayload/performance-spazio/Android/
package/commit/push. Linux/USA/nativeWindows/new32:9 pendenti.
[Modello e limiti](../rom_data_gameplay.md#typed-twenty-slot-job-lookup-and-original-reader-proof).


**ROMDATA-03-SLOT-JOB-LOOKUP-RUNTIME completato managed/runtime EUR (2026-10-26).**
e71-d70:66file65risorse/gameplay.slot-job-lookup/slot-job-lookup.json, dipendenza
reale metadata52; tutti64record/file precedenti ed e70 conservati, USA8-7 invariato.
Owner immutabile20u8/20B/page399/maschera20record, originali inputlow8/slotselection/
clear/properties/name/rawoutside preservati, nessun cap20guest. Primo buildWindows
core/frontend senza warning e due nuovi filtri Wine PASS dopo stop ENOSPC e retry su filesystem temporaneo:18attachguard/3
dependencyguard/publication/lifecycle directory-indisponibile/tutti20Brawalterati/
tutti20bit/ownerFFFdistinto. Sedici percorsi original/interpreter/native fullstate-
clock uguali: getter/properties/nameIDprefix/highinput/slot20raw/liveRAM e12CA98
mode3slot19→CFF70clear→D05C8prima165E8. Bordi/width/mirrors/realBIOSbus; API2new2raw,
CpuSet20new4raw/FastSet20new12raw/DMA20new4raw preservati. Vecchi64owner confermano
rawneighbors. Source268pin/model123reader/oldproof chiusi senza replay. CLI04/
ordinary05 aperti; Linux/USA/nativeWindows/new32:9 pendenti. Nessuna fixture/
navigazione/privatepayload/performance-spazio/Android/package/commit/push.
[Consumo e limiti](../rom_data_gameplay.md#managed-twenty-slot-job-lookup-and-typed-bus-consumption).


**ROMDATA-04-SLOT-JOB-LOOKUP-ACTUAL-CLI completato EUR/Wine (2026-10-26).**
Primo buildWindowsproduzione senza warning/candidato congelato; canonicale hash+
mtime esatti ripristinati/main contenuto invariato. Prime tre chiamate Wine
prepare/reuse/oldcache-oneframe da cwd estranea passano66generation68managed/
65risorse/tutti66uguali al dataset03/dipendenzametadata52/riuso senza scritture.
Rootprivato su filesystemtemporaneo mediante linkworkspace dopoENOSPC03, nessun
ulteriorecleanup/formatobudget/buildretry. F09/cacheoriginali integri: sola
provaattachment, ordinarypositivo/equivalenza aperti. Source/modello/runtime/
oldproof chiusi senza replay; Linux/USA/nativeWindows/new32:9 da verificare.
Nessuna fixture/navigazione/privatepayload/performance-spazio/Android/package/
commit/push. [CLI e limiti](../rom_data_gameplay.md#actual-production-cli-for-the-twenty-slot-job-lookup).


**ROMDATA-05-SLOT-JOB-LOOKUP-BANK-ORDINARY completato per scena EUR dichiarata (2026-10-26).**
Primo nuovo CLIbank120positivo2read/2B/1slot, con checkpointcopiato/inputDown8..20
esistenti e datasetproductione71generation-GmxCy2. Unicoscopo nuovojoblookup,
vecchioA4negativo/B2positivo chiuso. Solo5lineoptionalguard/report nelvalidatore
scene, runtime/prodotto/vecchiguard/stati invariati. Primo buildWindowsfrontend
senza warning e primo filtro Wine esatto PASS:120fullstate-clock da3660a3780/
central-wide/OAM/dispatchnative-fallback e mixed-PSG-FIFOleft-right-HLEbitexact.
Tutti5samplecount[263340,263340,31680,31680,263264]positivi; nuovouso2B1slot uguale
CLI. Altri19slot ordinari targeted-only. CatturaCLI284x160ispezionata:sfondoBank,
pannelliUI assenti dalrawframebuffer; nessuna GUIcompositafinale/fullname-actor-
allocatorABI/new32:9 dichiarata. Input/copia/originale/cache/dataset integri;
nessuna fixture/navigazione/privatepayload/performance-spazio/oldproof replay.
Source/model/03/04/05dichiarati chiusi, fullTODOattivo/Linux-USA-nativeWindows-
new32:9pendenti. [Scena e limiti](../rom_data_gameplay.md#ordinary-bank-scene-use-of-the-twenty-slot-job-lookup).


**ROMDATA-01-POST-SLOT-LOOKUP-READERS completato per tre API RAM EUR (2026-10-26).**
D05E8anynonnull eD0618count leggono solo le primeword di20slotRAM02002E78/stride16;
non validano puntatori/liveness, D05E8ritorna presto. D0650incrementa byte02001F68
solo seold<=98, lascia99..255invariati (nessun clampglobale). Tutti ignorano arg
entranti perquestiindirizzi; sei literal sono supporto eseguibile RAMbase/offset,
nessuna nuova radice ROMasset.68halfword/30criticalpin/6literal/treblock completi
verificati;3/2/1BLmatch raw restano candidati/non completezzasemantica. Nessun
export diRAM viva/nuovadecoder-risorsa-owner/revisione/cap20tabellaROM. e71-d70/
66file65risorse/USA8-7 invariati. ProssimoD0670parziale escluso. Sourcechildchiuso,
vecchieprove/Bank non ripetute; nessun build/guest/test/fixture/navigazione/
privatepayload/performance-spazio. Linux/USA/nativeWindows/new32:9 pendenti.
[Stato vivo e limiti](../rom_data_gameplay.md#live-state-helpers-following-the-slot-job-lookup).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE completato per il campione EUR Bank12
(2026-10-26).** Un osservatore solo di test avvolge le API native read8/16/32 e
sottrae i20756intervalli uniti dei65piani sorgente e71. Primo build Windows senza
avvisi e un filtro Wine:3166letture di contesto/3683byte immagine;198letture
includono676byte fuori piano,79intervalli osservati. Stato completo/clock,
central-wide/OAM/dispatch e cinque flussi audio coincidono a ogni frame3660–3672.
676byte sono indizi da autenticare, inclusi cinque word370E70/74/80/84/EB0;
nessuna nuova risorsa/revisione o completezza globale. PC esportato può essere
stale; interprete/HLE/DMA/costanti native e altre scene fuori campione.73input
hash-mtime esatti; main/CAPI/vecchi validatori invariati, solo nuovo modulo test
con dichiarazione cfg. Nessuna nuova fixture/navigazione/misura prestazioni.
Le vecchie accettazioni Bank120/source/model/runtime/CLI restano chiuse.
Linux/USA/nativeWindows/new32:9 da fare.
[Contratto, trasparenza e limiti](../rom_data_runtime.md#bounded-native-rom-api-residue-inventory).


**ROMDATA-01-NATIVE-RESIDUE-HIGH-WORD-READERS completato staticamente EUR
(2026-10-26).** I cinque word osservati370E70/74/80/84/EB0 sono supporto di
esecuzione: due directory di indirizzi registri DMA e un ingresso ThumbIWRAM
passato aBXr2.392halfword/47pin/5literal/4BL in sei blocchi interi autenticati;
loadreali0846/09B2/0D56/3A7E/11DA/11EE distinti dalPCesportato. I chiamanti
originali provano due proiezioni4x4B370E68/78 e un word370EB0,36Bdisgiunti da65owner;
20Bosservati classificati,656Blower ancora da classificare. Nessun capglobale,
RAMbody/ABI/scheduler o nuovoasset/owner/revisione inferito; dipendenza ROM/codice
esplicita e676Bancora fuori piani gestiti. Nessun build/guest/test/fixture/
navigazione/misura/vecchia accettazione ripetuti. e71-d70/USA8-7 invariati;
Linux/USA/nativeWindows/new32:9 pendenti.
[Directory hardware e controllo di salto](../rom_data_runtime.md#high-offset-native-residue-is-hardware-and-branch-support).


**ROMDATA-01-NATIVE-RESIDUE-LOWER-READERS completato per testimoni statici EUR
(2026-10-26).**656Bosservati:158wordliteral/632B con184percorsi Thumblineari entro96B
dai contestisalvati (507halfworddistinti), e6word/24B in5tabelle salto con82halfword
di prefisso bounds/indice/load/MOVpc autenticati. Proiezioni8/35/69/48/5entry;
solo0/0/27/4-6/1osservate. Non è una traccia dinamica istruzione/modo né prova di
tutti gli usi. Insieme ai20Bhardware/branchhigh, tutti676Bfuori piano hanno riscontri
statici di supporto codice; restano fuori dai65owner/dipendenzaROMoriginale.
Nessuna nuova famiglia/radiceasset/owner/revisione/completa migrazione inferita.
Nessun build/guest/test/fixture/navigazione/misura/vecchia accettazione ripetuti.
e71-d70/USA8-7 invariati; interprete/HLE/DMA/costanti native/altre scene e
Linux/USA/nativeWindows/new32:9 separati.
[Literal e tabelle di salto](../rom_data_runtime.md#lower-native-residue-has-literal-and-switch-table-witnesses).


**ROMDATA-01-NATIVE-ROM-DATA-RESIDUE-F09 completato per secondo campione EUR
(2026-10-26).** Stesso eseguibile osservatore congelato, nessun build: un filtro
Wine12frame8476→8488 da checkpoint/inputF09esistenti preserva stato/clock/buffer/
OAM/dispatch e cinque audio.13671letture contesto/13961Bimmagine;604Bfuori piano,
584condivisi conBank e20nuoviB/5word. Tutti5hanno testimoni ThumbPC-relative
lineari (20halfword); fonte statica, non traccia dinamica/modo/tutti gli usi.
Unione due campioni696Bancora fuori65owner, nessun asset/revisione inferiti.
76inputhash-mtime e codice compilato esatti, helperretention0percorsi. Nessuna
fixture/navigazione/misura o vecchia accettazione F09/Bank ripetuta. Altri percorsi
interprete/HLE/DMA/costanti e Linux/USA/nativeWindows/new32:9 restano separati.
[Secondo campione](../rom_data_runtime.md#second-native-api-sample-from-the-existing-f09-checkpoint).


**ROMDATA-01-HOST-ROM-READ-RESIDUE completato per collector e campione EURF09
(2026-10-26).** FeatureCargo opzionale non-default rom-data-read-audit: collector
thread-local indirizzo/lettore/lunghezza su bus risolto e hostaudio/sprite/tabelle/
keyboard selezionati; fetchistruzioni interprete escluso, esecuzione/clock/API/
savestate invariati. Primo build corecompilato/frontendfallito per qualifica
c_void mancante nel test; correzione solo test e buildincrementale senzaavvisi.
Un filtro Wine: probeARM/Thumb/width/mirror/audio/lifecycle e12frame8476→8488
stato-clock-buffer-OAM-dispatch-cinqueaudio identici.23061richieste/19984Bimmagine,
1146Bfuori65owner; include tutti604Bfuoripiano del vecchioAPIreport e aggiunge542B.
90nuoviBalti/6intervalli restano sorgenti da autenticare,64Banche daSpritePeek.
Nessun formato/asset/extent/owner/revisione inferito.76inputesecuzione+1report
confrontohash-mtime esatti, nessuna vecchia prova/fixture/navigazione/misura.
e71-d70/USA8-7 invariati; KeyboardWidthsnonattivo/altrethread-scene/accessihost/
fastpathHLEtipizzati fuori scope; Linux/USA/nativeWindows/new32:9 pendenti.
[Collector, prova e limiti](../rom_data_runtime.md#optional-bus-and-host-rom-read-audit).


**ROMDATA-01-BUS-HOST-HIGH-RESIDUE completato per contratti sorgente EUR
(2026-10-26).**90Bnuovi classificati:64Bdue listeOBJ indirette394F4C/26B/4oggetti e
40D89E/38B/6oggetti;10Bparametri (2thresholdphase,4wordverticali signed);16Bpuntatori
ThumbIWRAM di controllo.295halfwordfinestra/53pin/7literal+hashABIriusati. Copia
originale0924E8 provaD968E8..D97728→03005F28..03006D68, costruttoreD9752C scrive
lista+158; rawROMdis aRAMtargetvuoto rifiutato. Un solo frame Wineproducerargs
esistenteF09 conferma2lettori/10oggetti, nessuna equivalenzavecchia ripetuta.
40D89Eda task+24 è osservazioneindiretta, nonliteralinventato; creazionecomando
nonricostruita. TimingconfrontaOLDcounter, fasepostupdate0..2 nonglobalcap;
correzionequerybyte>>1 ha128possibiliindici, solo4..7nuovi qui.64Blist+10Bparametri
ancora da modellare/pubblicare;16Bcontrollo restano dipendenzaROM/codice;452B
lower separati. e71-d70/65owner/USA8-7 invariati, nessun build/fixture/navigazione/
misura. Linux/USA/nativeWindows/new32:9 pendenti.
[Liste indirette e prove](../rom_data_obj.md#observed-indirect-obj-list-sources-beyond-the-direct-catalog),
[parametri e controllo](../rom_data_runtime.md#high-bus-and-host-residue-source-contracts).


**ROMDATA-02-OBSERVED-OBJ-LISTS-MODEL completato per due liste EUR
(2026-10-26).** API isolataobj_lists::observed riusaList/Object:2liste/10oggetti/
64B, identità/count/shape/fieldwidth/JSONstrict/sourcecoerente autenticati, root
constructor-bound distinto da ordinary-task-argument. Primo buildfrontendWindows
senzaavvisi e un filtro Wine:64B/2span/14guardie/6percorsi originali ROM1500/170C/
copiedRAM030065A8, output/matrici/stack/callee/RAMesterna/OAM-VRAM-palette esatti.
JSONcanonico4530552b..;75guardieinput-artefatti e codicecompilato invariati.
Solo2dichiarazioni nei fileesistenti; vecchioCatalog74/175oggetti1198B/e71-d70/
66file65risorse/USA8-7 invariati. Nessuna pubblicazione/owner/revisione o prova
ordinary da dedurre; modellochiuso, managedruntime/CLI/nuovo uso dafile ancora
aperti. Nessun vecchio74/ABI/copier/scenecheck ripetuto, nuova fixture/navigazione/
misura.10Bparametri e452Blower separati; Linux/USA/nativeWindows/new32:9 pendenti.
[Modello e verifiche](../rom_data_obj.md#isolated-typed-model-for-the-two-observed-indirect-lists).


**ROMDATA-03-OBSERVED-OBJ-LISTS-RUNTIME completato per due liste EUR
(2026-10-26).** e72-d71 integra le due identità indirette nella risorsa OBJ
esistente: 76 liste, 185 oggetti e 1262 byte sorgente. Restano 126 riferimenti
diretti, 66 file/65 risorse, stessa dipendenza dimensioni, tutte le 74 liste
precedenti, gli altri 64 file/record ed e71. USA8-7 invariata. Primo build
Windows senza avvisi e due filtri Wine passano pubblicazione/riparazione/move,
12 guardie di attivazione e Load/Undo/backup/restart; sei percorsi originali/
interprete/nativo hanno stato completo e clock uguali con tutti i 64 nuovi byte
ROM alterati. Bordi/larghezze/mirror/BIOS, sei trasferimenti DMA-CpuSet-FastSet,
letture API 2B vecchi+2B nuovi e 2B raw modificati+2B nuovi, sei pubblicazioni
host passano. 5571 istruzioni/2418 dispatch sono solo copertura; 265 byte typed
serviti da API native. File esattamente uguale al modello accettato, 77 guardie
input/eseguibile e codice compilato invariati; completamento artefatti elimina
solo copie interrupted/moved. CLI di produzione e nuova equivalenza ordinary
restano aperte; dieci byte parametri e 452 byte lower separati. Nessun vecchio
modello/ABI/scena ripetuto, nuova fixture/navigazione/misura. Linux, USA, Windows
nativo e nuovo32:9 da verificare.
[Integrazione e prove](../rom_data_obj.md#managed-observed-indirect-lists-and-typed-consumption).


**ROMDATA-04-OBSERVED-OBJ-LISTS-ACTUAL-CLI completato EUR (2026-10-26).**
Primo build Windows di produzione senza avvisi; CLI separata c6d674d7..,
eseguibile canonico hash/mtime ripristinati esattamente, contenuto main invariato
con sola invalidazione Cargo via mtime. Tre primi comandi Wine da cwd estranea
passano create/reuse/oldcache-oneframe: 66 file generazione/68 managed, tutti i
66 uguali al dataset runtime, 65 risorse/76 span OBJ/dipendenza dimensioni.
Riuso senza modifiche hash/mtime. F09/input/cache originali a16:9: entrambe le
nuove liste00394f4c/0040d89e guest=true host=true; totali OBJ147letture/147B/
14B count/133B attributi/12letture host. Uso positivo, equivalenza ordinary
completa ancora aperta. Tutte143guardie CLI e implementazione/canonico/originali
esatti; helper artefatti zero percorsi. Nessun vecchio source/model/runtime/
producer logging/scenario ripetuto, nuova fixture/navigazione o misure.
Linux/USA/Windows nativo/nuovo32:9 da verificare; dieci byte parametri e452lower
separati. [CLI e prove](../rom_data_obj.md#actual-cli-for-the-observed-indirect-lists).


**ROMDATA-05-OBSERVED-OBJ-LISTS-ORDINARY completato per il primo frame F09 EUR
(2026-10-26).** Solo cinque righe opzionali al validatore esistente richiedono
entrambe le nuove liste guest+host. Primo build Windows senza avvisi e un solo
filtro Wine: frame8476→8477, stato completo/clock/immagini centrale-larga/OAM/
dispatch e tutti gli audio uguali. Cinque flussi con campioni reali
[2196,2196,264,264,2120]; contatori OBJ147letture/147B/14count/133attributi/
12host esattamente uguali alla CLI. 6803native/22849fallback solo copertura;
75guardie input/eseguibile, originali/cache/dataset/canonico esatti. Delta di
cinque righe verificato sul preimage, altre implementazioni invariate; helper
artefatti zero percorsi. Source/model/managed/runtime/CLI e ordinary dichiarato
chiusi; altre scene/indiretti/composedGUI/ABI/nuovo32:9 aperti. Nessun vecchio
12/3530/scenario/modello/runtime ripetuto, nuova fixture/navigazione o misure.
Dieci byte parametri/452lower e Linux/USA/Windows nativo separati o da verificare.
[Equivalenza e limiti](../rom_data_obj.md#ordinary-first-frame-equivalence-for-the-observed-lists).


**ROMDATA-02-OBSERVED-TASK-PARAMETERS-MODEL completato EUR (2026-10-26).**
API isolata observed_task_parameters per due frammenti indipendenti: soglie u8
fasi1/2 a371065..67 e quattro correzioni signed16 indici4..7 a394E14..1C.
Identità/indici/JSONstrict/ricostruzione10B/due span/code-literal-source hash;
nessuna durata/cap globale/estensione fisica dedotta. Fase0/altri byte fuori
modello; correzioni0..3 conservano l'owner precedente,8..127 restano esterne.
Primo build Windows senza avvisi e un filtro Wine:27guardie e18prefissi originali
(10old-counter/phase-wrap,8alias-bitbasso/signed-sub), RAM/callee/OAM-VRAM-palette
esatti. Modello canonico4b52c5f1..;79guardie input/eseguibile e implementazione
esatte. Solo due dichiarazioni nei vecchi hub; pipeline/e72-d71/66file65risorse/
76listeOBJ/USA8-7 invariati. Nuovi10B disgiunti da tutti gli owner e non ancora
pubblicati; managed/runtime/CLI/ordinary dei parametri aperti. Helper artefatti
zero percorsi. Nessun source/295pin/vecchio modello-ABI-runtime-CLI-scena
ripetuto, nuova fixture/navigazione/misura.452lower separati;
Linux/USA/Windows nativo/nuovo32:9 da verificare.
[Modello e limiti](../rom_data_gameplay.md#isolated-model-for-observed-task-parameter-fragments).


**ROMDATA-03-OBSERVED-TASK-PARAMETERS-RUNTIME completato EUR (2026-10-26).**
e73-d72 pubblica gameplay.observed-task-parameters/observed-task-parameters.json:
67file/66risorse, tutti65file/record precedenti/e72/76listeOBJ conservati. Due
campi indipendenti, nessuna dipendenza inventata, solo10B/due span371065..67 e
394E14..1C; pagina371/394 compone owner precedenti e ROM, senza cap/ABI/clock/
stato nuovi. Due primi filtri Wine passano13rifiuti attach/3guardie fullset,
pubblicazione/riparazione/move/lifecycle e24prefissi originali-interprete-nativo
con stato completo/clock uguali e tutti10B raw alterati. Inclusi fasi0/3/255,
correzioni0/16/255, bordi/mirror/BIOS, sei DMA-CpuSet-FastSet, letture API
2nuovi+2vecchi con raw vecchi alterati e2nuovi+2raw modificati.26B nativi;
316istruzioni/24dispatch solo copertura. Modello4b52c5f1.. esatto,74guardie e
codice compilato esatti. Primo build core compilato, frontend due argomenti di
test omessi: corretti solo due zeri, retry senza avvisi; nessun filtro Wine
fallito. Helper artefatti elimina solo interrupted/moved. CLI di produzione e
ordinary parametri restano aperti; Linux/USA/Windows nativo/nuovo32:9 pendenti,
452lower separati. Nessun vecchio modello/source/ABI/scenario ripetuto, nuova
fixture/navigazione/misura.
[Runtime e prove](../rom_data_gameplay.md#managed-observed-task-parameters-and-typed-bus-consumption).


**ROMDATA-04-OBSERVED-TASK-PARAMETERS-ACTUAL-CLI completato EUR (2026-10-27).**
Build Windows produzione senza avvisi/c7f47bd4..; canonico hash/mtime ripristinati
esattamente, contenuto main invariato. Prima prepare interrotta dall'utente:
runner assente, manifest senza current/risultato; copia/log archiviati e stesso
binario riusato. Tre comandi accettati exit0: create/reuse/oldcache-oneframe,
67file generazione/69managed/66risorse/due span10B/nodeps/all67esatti, riuso senza
scritture. Primo frame1B soglia/1fase ma0correzioni: gate presenza esplicitamente
negativo, conservato. Una nuova runc12frame, motivata da quel negativo e dal
lead sorgente esistente, exit0 con20letture/28B/12soglie16correzioni/2fasi4righe.
145guardie e implementazione/originali/cache/canonico/Atlas utente invariati;
helper artefatti zero percorsi, copia interrotta conservata. CLI chiusa; ordinary
fullstate12frame da verificare. Nessun prepare/reuse/build/vecchio source-model-
runtime-scenario ripetuto, nuova fixture/navigazione o misure. Linux/USA/Windows
nativo/nuovo32:9 e452lower separati o pendenti.
[CLI e limiti](../rom_data_gameplay.md#actual-cli-for-observed-task-parameters).


**ROMDATA-05-OBSERVED-TASK-PARAMETERS-ORDINARY completato EUR (2026-10-27).**
Solo cinque righe opzionali al validatore esistente richiedono entrambi i gruppi
e tutte le sei righe. Primo build Windows senza avvisi e un solo filtro Wine:
12frame F09 guest8476→8488, ogni stato completo/clock/immagine centrale-larga/
OAM/dispatch/audio uguale. Cinque flussi con campioni reali
[26334,26334,3168,3168,26258];20letture/28B/12soglie16correzioni/2fasi4righe
esattamente uguali alla CLI.88123native/279744fallback solo copertura.76guardie,
originali/cache/dataset67file/canonico/codice/Atlas utente esatti. Delta cinque
righe verificato sul preimage; helper artefatti zero percorsi. Source/model/
managed/runtime/CLI/ordinary dichiarato chiusi. Primo frame correction-negative
e prepare interrotta restano chiusi; nessun vecchio source-modello-runtime-CLI-
collector-scenario ripetuto, nuova fixture/navigazione/misura. Altre scene,
indiretti/ABI/composedGUI/nuovo32:9 aperti;452lower e Linux/USA/Windows nativo
separati o pendenti.
[Equivalenza e limiti](../rom_data_gameplay.md#ordinary-twelve-frame-equivalence-for-observed-task-parameters).


**ROMDATA-01-BUS-HOST-LOWER-RESIDUE completato EUR (2026-10-27).**
452byte/113word/62intervalli classificati con114letture contestuali:404B da
102LDR Thumb PC-relative su101literal,48B da12voci switch osservate7–18.
Caricamento080A1146/MOVPC080A1148 conserva Thumb; prefisso originale normalizza
u16zero/>24a1 prima dell'indice, senza nuovo limite globale. Il nuovo SiteCapture
opzionale conserva schema/chiavi precedenti; scopePC/mode richiede verifica
opcode/indirizzo/larghezza, non attribuzione automatica per HLE/DMA annidati.
Un build Windows incrementale senza avvisi dopo errore helper solo nel test;
un filtro Wine con nuove sonde e un solo arm F09 corrente12frame osserva tutti
113word.85guardie/preimage cfg/codice/Atlas esatti; helper artefatti zero percorsi.
Nessuna vecchia accettazione ripetuta, fixture o misura. I452B sono supporto
codice, e73-d72/67file66risorse invariati. Nuove sorgenti394FBC/371064fase0/
371034 richiedono contratti propri; nessun formato/estensione dedotto dai literal.
Linux/USA/Windows nativo/nuovo32:9 da verificare, TODO globale aperto.
[Contratto e limiti](../rom_data_runtime.md#lower-bus-and-host-residue-source-contracts).


**ROMDATA-01-ADDITIONAL-CONSTRUCTOR-OBJ source/modello completato EUR (2026-10-27).**
394FBC è una tabella di quattro coppie condizionali soglia32bit/puntatorelista,
32B; lettori originali9DB56/9DBCA e costruttore già autenticato provano quattro
liste394F82/90/9E/AC,56B/otto oggetti. Totale88B, nessuna sovrapposizione owner.
Incremento/store/reloadu8 prima del confronto;255→0, fase nuova>3 disattiva.
Vecchia fase non limitata prima del lookup: modello0–3, nessun nuovo cap globale.
Modello isolato con JSON rigoroso/proiezione sorgente/verifica dipendenza liste.
Primo build Windows senza avvisi/unico filtro Wine passa13guardie,15prefissi
contatore, quattro prefissi puntatore-costruttore e quattro nuovi disegni con
oracolo copiato esistente.94guardie/codice esatto, hub solo dichiarazioni/helper
solo visibilità; helper artefatti zero percorsi. F09 conservata non legge questi
dati: nessun uso ordinario dichiarato. Produzione e73-d72/67file66risorse invariata;
estrazione/owner runtime/uso positivo da fare. Linux/USA/Windows nativo/nuovo32:9
pendenti, nessuna vecchia accettazione ripetuta/fixture/navigazione/misura.
[Contratto e modello](../rom_data_obj.md#conditional-constructor-obj-animation-source-and-model).


**ROMDATA-03-CONSTRUCTOR-ANIMATION-RUNTIME completato EUR (2026-10-27).**
e74-d73/68file67risorse pubblica tabella32B con dipendenza OBJ; catalogo80liste/
193oggetti1318B, tutte76liste precedenti/126ref dirette/65altri file-record esatti.
Owner tabella esatta con composizione precedente/raw, liste servite da owner OBJ
esistente. Primo Wine supera10guardie/2halfset/lifecycle/pubblicazione e13prefissi
originale-interprete-native stato-clock con88B grezzi alterati:36B typed native/
20soglie16puntatori/4fasi,4query esterne senza addebito,708casi bus e composizione.
Si ferma poi su errore setup test: load_state prima del ripristino ROM. Fallimento
conservato, non dichiarato PASS completo. Corretto solo l'ordine nel test; filtro
restante su dataset già validato passa6DMA/SWI e4pubblicazioni host/due oggetti
reali ciascuna, senza ripetere le parti superate. Produzione identica fra i filtri.
Build finali senza avvisi,169guardie/codice/Atlas esatti; helper artefatti0percorsi,
diagnostica fallita e offline68 conservati, solo exe mai eseguito sostituito ritirato.
CLI reale/uso ordinario positivo da fare; F09 conservata inattiva non basta.
Linux/USA/Windows nativo/nuovo32:9 pendenti; nessuna nuova fixture/misura/replay.
[Consumo e prove](../rom_data_obj.md#managed-constructor-obj-animation-and-typed-consumption).


**ROMDATA-04-CONSTRUCTOR-ANIMATION-CLI completato EUR (2026-10-27).**
Primo build produzione Windows senza avvisi/1e98de03..; canonico hash/mtime
ripristinati e contenuto main invariato. Tre CLI Wine exit0: prepare/riuso senza
scritture/runc0frame.68file generazione/70managed/67risorse, tutti68hash identici
al runtime accettato e tutti67ID/hash runtime esatti; vecchia cache caricata.
Hash/mtime riuso e startup invariati,145guardie/439sorgenti esatte; helper0percorsi.
Zero frame/dispatch/fallback/contatori nuovi: prova registrazione/attach reale,
nessun uso ordinario positivo dedotto. Nessun F09/source-model-runtime-scenario
ripetuto, nuova fixture/navigazione/misura. Uso ordinario positivo e Linux/USA/
Windows nativo/nuovo32:9 restano da verificare.
[CLI e limiti](../rom_data_obj.md#actual-cli-for-constructor-obj-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE parziale EUR (2026-10-27).**
Osservatore passivo Wine-only implementato e guardia positiva opzionale nel
validatore (sette righe; loop precedente esatto senza guardia). Primo build
Windows senza avvisi; quattro osservazioni normali exit0 ma NEGATIVE: due
spostamenti160/200frame dal checkpoint EUR12006 copiato, selezione ritardata160,
conferma200 dal nuovo stato di selezione. Tabella0/fasi0/liste inutilizzate,
nessun fullpair o fixture positiva dichiarati.84guardie/codice/originali esatti;
provenienza normale precedente conservata senza replay3530frame. Nessuna
mutazione manuale RAM/task/fase/ROM. Setter flag9D054/call6 verificati come lead,
azione normale/semantica ancora da provare; niente altra rotta cieca o flag forzato.
Unità aperta, fixture/diagnostica conservate; sorgente indipendente371064fase0
separata. Linux/USA/Windows nativo/nuovo32:9 pendenti, nessuna misura.
[Prova ancora aperta](../rom_data_obj.md#ordinary-constructor-animation-discovery-remains-partial).


**ROMDATA-01..05-CONDITIONAL-PHASE-ZERO completato EUR (2026-10-27).**
371064fase0 ora tipizzata: soglie condizionali0/1/2=16/8/8, span3B più8correzioni,
11B nello stesso file/68file67risorse/e75-d74. Vecchie2righe+4correzioni/all66altri
record-file esatti. Nessun cap globale prima lookup o cambio ABI. Partizione
copertura3fasi/4correzioni disgiunta. Primo build Windows senza avvisi/unico Wine
passa4prefissi originali nuovi/7attach/6stati-clock nativi/3DMA e bordi raw.
Primo build produzione/canonico ripristinato, una CLI auto-prepare28frame passa
all68file/3fasi4correzioni/36letture44B/28soglie16correzioni. Corretto solo matcher
log offline dopo CLIexit0; comando non ripetuto. Stesso exe frontend: un nuovo
fullpair28frame8476→8504 passa ogni stato-clock/immagine/OAM/dispatch/audio con
campioni[61446,61446,7392,7392,61370], conteggi esatti CLI.215guardie/codice/loop
senza guardia5righe esatti; helper0percorsi. Nessun vecchio12frame/source/modello/
runtime/constructor ripetuto o nuova fixture/misura. Ordinaryconstructor rimane
parziale;371034e piattaforme Linux/USA/Windows nativo/nuovo32:9 ancora da verificare.
[Contratto e prove](../rom_data_gameplay.md#conditional-phase-zero-threshold-extension).


**ROMDATA-01-TASK-CYCLE-DELTAS / ROMDATA-02-TASK-CYCLE-MODEL completati EUR
(2026-10-27).** Proiezione condizionale371034–371064/48delta signed-i8,
disgiunta da tutte67risorse e75. Lettori18E10/18E2A usano OLD-u8 prima della
normalizzazione; incremento memorizzato come u8 prima helper1464EC/resto48,255→0. Somma/
sottrazione signed su halfwordtask+8 con storelow16; asse/unità/lifetime non
inferiti. Nessun cap introdotto: query48..255None,48/49/50 sono vecchio owner.
Primo build Windows senza avvisi/unico filtro Wine passa23guardie/58prefissi
originali/8bordi helper/tutte48righe e guardie RAM-callee,77input-frozen esatti.
Modello lossless/JSON rigoroso; revisione e75-d74/68file67risorse invariata.
Gestione/consumo runtime/CLI/ordinary48 completati nelle unità sotto; il solo
test sorgente non dichiarava consumo o scena. Constructorordinary precedente ancora parziale; Linux/USA/
Windows nativo/nuovo32:9 pendenti, nessuna misura o nuova fixture ordinaria.
[Contratto e prove](../rom_data_gameplay.md#conditional-european-task-cycle-delta-source-and-model).


**ROMDATA-02/04-TASK-CYCLE-RUNTIME completato EUR/Wine (2026-10-27).**
Nuova risorsa gameplay.task-cycle-deltas/filetask-cycle-deltas.json/e76-d75:
69file68risorse,48B signed-i8/uno span, tutte67precedenti risorse/file esatte.
Owner tipizzato compone nuovi byte/prior/raw, niente cap o cambio ABI; USA8-7.
Primo build Windows senza avvisi/primo filtro Wine passa9rifiuti attach
transazionali/2insieme completo/interruzione nuovo file,8prefissi originali-
interprete-nativi con stato completo/clock uguali e48rawB alterati,5letture
esplicitamente compilate. Tutte48righe, confine read32con2nuovi+2vecchi byte,
bordi/mirror,3trasferimenti DMA/SWI tra owner, file indisponibili/Load/Undo con
owner conservato.154guardie/codice esatti; helper ritira solo copia interrotta.
Nessuna ripetizione filtro sorgente o vecchie accettazioni. CLI produzione completata sotto;
ordinary positiva48 dei delta completata sotto; constructorordinary parziale.
Linux/USA/Windows nativo/nuovo32:9 pendenti, nessuna misura.
[Consumo e prove](../rom_data_gameplay.md#managed-task-cycle-deltas-and-typed-consumption).


**ROMDATA-05-TASK-CYCLE-CLI completato EUR/Wine (2026-10-27).**
Primo build produzione Windows senza avvisi/canonico ripristinato conhashmtime
esatti; tre comandi Wine prepare/riuso/runc0exit0. Tutti69file uguali al dataset
Rust accettato, tutti68ID/hash runtime uguali al manifest,48righe delta registrate,
cache nativa esistente caricata. Tutto pathset71file/hashmtime invariato su riuso
ed avvio;226guardie/codice esatti, helper0percorsi. Zero frame/dispatch/fallback/
letture delta: solo attacco; positività ordinaria e fullpair48 completati sotto. Niente
rotta cieca/manualflag. Linux/USA/Windows nativo/nuovo32:9 pendenti, nessuna misura.
[CLI e limiti](../rom_data_gameplay.md#actual-cli-for-task-cycle-deltas).


**ROMDATA-05-TASK-CYCLE-ORDINARY completato EUR/Wine (2026-10-27).**
Scena Bank48neutra3660→3708 scelta da sorgente e snapshotnormale: initializer
18D6C/default03, generic015E48/03-07 e candidato Bankroot+D40flags03phase21,
indirizzo coerente col caller5C1F8. Setter18E5C conserva vecchi04/08 con nuovo01:
argomento03non basta a provare il modo.11/3/89BL grezzi sono candidati, niente
callgraph/liveness dedotti dalla sola RAM. Una CLI esistente senza rebuild usa
48letture48B/all48fasi, dataset71path/hashmtime invariato. Primo build frontend
senza avvisi per5righe di guardia positiva; vecchio loop byteesatto senza guardia.
Un solo fullpair Wine passa ogni stato completo/clock/immagine/OAM/dispatch/
cinque flussi audio, campioni[105336,105336,12672,12672,105260], conteggi48/48/48
esatti CLI.97guardie/codice/preimage esatti, helper0percorsi. Nessuna navigazione/
manomissione RAM-task-ROM/cartridge o vecchia accettazione Bank12/F09 ripetuta.
Child dichiarati01/02/04/CLI05/ordinary05 ciclo chiusi, migrazione totale ancora
aperta; constructorordinary distinto parziale. Linux/USA/Windows nativo/nuovo32:9
pendenti, niente misure.
[Scena e prove](../rom_data_gameplay.md#ordinary-european-task-cycle-scene).


**ROMDATA-01-BANK-BUS-HOST-RESIDUE: censimento EUR/Wine completato, sorgenti90B
aperte (2026-10-27).** Modalità opzionale SiteCapture::start_all conserva filtri
vuoti/singoli e vecchie chiavi. Primo build feature Windows senza avvisi/unico
Wine: probe nuovi e un braccio Bank12decodificato3660–3672 con input normale
esistente, niente vecchio fullpair ripetuto.7430letture/7532site,497con contesto
interprete/7035None;7083B immagine,1434B fuori68owner. Tutti3683B del vecchio
NativeAPI inclusi;992B già classificati,80literal/320B e8celle dispatch/32B
verificati ora:1344B supporto codice, niente cap127globale da branchconhelper.
Restano90B dati:18programma/12directory/60listeOBJ, root4165A0+4114FC via5E6F4/8,
lettori2C5E0/2C60C con tick-fase-selettoreu16, distinti dai banchi u8già tipizzati.
Contratto e modello dei quattro prefissi autenticati completati sotto; gestione/
consumo restano aperti. Nessuna nuova risorsa/revisione dal solo censimento. Dodici delta tipizzati21–32 letti da
LDRB18E2A interprete spiegano il limite del vecchio NativeAPI.95guardie/codice/
preimage default esatti, helper0percorsi. e76-d75/USA8-7 invariati, Atlasutente
preservato; Linux/USA/Windows nativo/nuovo32:9 pendenti, niente misure/fixture.
[Inventario e nuove sorgenti](../rom_data_runtime.md#bank-bus-and-selected-host-rom-inventory).


**ROMDATA-01-U16-TASK-OBJ-BANK / ROMDATA-02-U16-TASK-OBJ-MODEL completati nel
prefisso EUR dichiarato (2026-10-27).** Quattro produttori2C5E0 autenticano root
41634C/411478,41647C/4114AC,4165A0/4114FC,4166C4/41154C:50liste140OBJ/7seq49frame/
65span1378B disgiunti da68owner. Formato relative-index/payload verificato,
padding escluso; sette liste non referenziate e tre seqvuote restano dati,
non prova di raggiungibilità/hold inventati. Contatore-fase-selettoreu16,
repeatlowu8, ritorno listavecchia prima incremento,255→256/65535→0, clamp/repeat/
done latch; nessun cap prima lookup o confine globale archivio. Modello lossless
JSON rigoroso riusa campi puri, non tempistica u8. Primo build core senza avvisi:
Wine passa23guardie/proiezione1378B poi fallisce aspettativa test sul r0di ritorno
ctor. Correzione solo test; secondo build core senza avvisi/filtro solo-rimanente
passa23ctor/93update/6producer, RAM-callee/alias/bordi. Primo filtro non dichiarato
PASS e23guardie non ripetute.88guardie/199sorgenti core esatte; helper0percorsi.
Coredevserde_json preserva modifiche frontend parallele senza compilarle. Revisione
e76-d75/USA8-7 invariata: gestione/consumo/CLI/ordinary di questi banchi ancora
aperti; Linux/USA/Windows nativo/nuovo32:9 pendenti, niente misure/fixture.
[Contratto e modello](../rom_data_obj.md#european-obj-banks-with-u16-task-counters).


**ROMDATA-02/04-U16-TASK-OBJ-RUNTIME completato in fasi EUR/Wine (2026-10-27).**
e77-d76 aggiunge graphics.word-task-obj-animations/fileword-task-obj-animations:
70file69risorse/all68record-file precedenti identici,65span1378B/4banchi/50liste/
140OBJ/7seq49frame. Binding rigoroso/aggancio transazionale; proiezione pura
condivisa mantiene corpo e gate del vecchio owner, nuovo gate411/416 e lookuphost.
Indicizzazione/contatori/ritorno listavecchia guest e ABI/stati/cache invariati.
Primo build Windows senzaavvisi; Wine passa pubblicazione/interruzione-file/
9guardattach/2fullset,27prefissi originali7ctor20update/202B compilati/fullstate-
clock,1378B/6classi/217mixedread/66B rawgap/vecchioowner/Load-slotUndo senzafile/
DMA3. Poi processo senza risultato terminale: non dichiarato fullPASS, ricevute
conservate, nessuna fase passata ripetuta. Build rimanente compilato pulito ma
rifiutato dal guardiasorgenti per2modifiche DBG parallele; snapshot stabile.
Drafttest host isolato perframe; errorecompilazione test-only campo privato e
origine OAM/register corretti, prodotto invariato. Buildfinale senzaavvisi/
unico Wine solo-host passa50liste/140OBJ/95letture host con65spanROM alterati,
OAM accettato perlista e descrittori raw/tipizzati uguali.165guardie/496sorgenti
snapshot/input ROM correnti esatti; helperritira sola copia interrupted prevista,
conserva dataset/log/preimage/eseguibili. CLIreale successivamente verificata
sotto; ordinary successivamente verificata nella finestra Bank sotto,
nessuna vecchia accettazione modello/censimento/scena ripetuta. USAe8-d7/Linux/
Windows nativo/nuovo32:9 pendenti; niente misure o nuove fixture/navigazione.
[Contratto runtime e prove](../rom_data_obj.md#managed-runtime-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-CLI completato EUR/Wine (2026-10-27).**
Un buildproduction Windows senzaavvisi da copia snapshot runtime accettata;
candidatoCLI b3bfe6f425923ce461793771fe6c84b86eb5da7dda69b3b9c7e0f5208eab51ea,
eseguibilecanonico hash/mtime ripristinato. Un unico scriptWine/3comandi reali
preparecreated/reused/runc0 passa:70filegenerazione identici al runtime/69risorse
registrate conhash esatti/72path managed. Riuso e avvio senza scritture hashmtime;
cacheoriginale caricata,0frameguest/dispatch/fallback e9campi nuovoowner tutti0.
154guardie/496sorgenti copiate/inputROMcorrenti esatti/helper0percorsi. Dataset
27463ce4f7ffccc3d0b9c8ab5ee78160a3e1c844ed80f68768f9adea87cb7757.
ChildCLI chiuso, ordinary positiva separata verificata sotto; nessuna accettazione
vecchia o fixture/navigazione ripetuta. Linux/USA/Windows nativo/nuovo32:9 pendenti,
niente misure/Android/package/commit/push. Modifiche DBG parallele preservate.
[CLI reale e prove](../rom_data_obj.md#actual-cli-for-the-u16-task-obj-banks).


**ROMDATA-05-U16-TASK-OBJ-ORDINARY completato EUR/Wine (2026-10-27).**
FinestraBank3660→3672 scelta dal censimento già accettato/root4165A0+4114FC,
copie byteidentiche di snapshot/inputnormali, nessuna navigazione/nuovocheckpoint/
manomissione RAM-task-ROM/cartridge. Una CLIe77 esistente passa uso positivo:
292letture/346B=[44index20count240attr2seqheader20frame-list20duration]/20host,
72pathmanaged/hashmtime invariati. Guardiaopzionale5righe sulnuovoowner nel test
fullpair; vecchio validator/loop byteesatto rimuovendola. Primo buildfrontend
Windows snapshot senzaavvisi/unicoWinefullpair primaPASS: tutti12stati completi/
clock/immagini central-wide/OAM/dispatch e5flussi audio identici,9contatori esatti
CLI. Dispatch[19962,85770], campioni[26334,26334,3168,3168,26258].94guardie/
496sorgenti snapshot/inputROMcorrenti esatti/helper0percorsi. Nessuna precedente
accettazione modello/runtime/scena/censimento ripetuta o buildproduction nuovo.
Child dichiarati sorgente/modello/gestione-consumo/CLI/ordinary diquestafamiglia
chiusi; nessunaprova all4banks/50lists/7liste nonreferenziate/seqvuote update/
indiceindiretto/fullarchivio/fullgioco. Constructorordinary distinto parziale;
wholeTODO e Linux/USA/Windows nativo/nuovo32:9 aperti, niente misure/Android/
package/commit/push, DBG parallelo preservato.
[Scena e prove](../rom_data_obj.md#ordinary-european-u16-task-obj-bank-scene).


**ROMDATA-01/05-CONSTRUCTOR-FLAG-REACHABILITY chiuso per il flag EUR/Wine
(2026-10-28); animazione ordinary ancora parziale.** Ventuno finestre sorgente
legano95EA0/wrapper9A594/global0200F4A8/statoDC/root960F0: Move27/A dopo Bcancel/
validazione9D6C8/destinazione diversa→count1A4/28-29, count signed>1→enable99F18.
Altri3A→99300 eA5544case6→A57BE/case1→A570A sono percorsi distinti, nonliveness
globale. Due snapshotnormali autenticati letti soltanto per scegliere A-only,
senza vecchioRight: unWineobserver160 riusato chiude quinta prova negativa,
table0/phases0/listeinutilizzate; nessunfullpair/fixturepositiva o buildnuovo.
Endpointstato25/count3 non implica destinazioneinvalidata. Nuovocriterio mancante
flag/slot duranteilpassaggio: unaCLIsolalettura64frame giàesistente cattura448
fileRAM-immagine, normaleMove e flag20attivo12186–12230/45postframe; entrambi
slot394/3A4 sempreinattivi/counter-phase0/table0. Flagordinaryraggiungibile provato,
creazioneslot/callback ancora da autenticare; niente attribuzionePC/intraframe.
587guardie/496sorgenti frozen/21hashfinestre esatti, originali/e77dataset72path
invariati/helper0percorsi. Nessuna vecchia accettazione ripetuta o RAM/ROM/cart
manomessa; staticodisassembler non è testgameLinux. Nuovafullpair solo dopo
usopositivo4fasi/4listeguest-host; nientealtri tentativiMoveallacieca. Linux/USA/
Windowsnativo/nuovo32:9/fullTODO aperti, niente misure/Android/package/commit/push.
[Flag e slot](../rom_data_obj.md#ordinary-constructor-flag-activation-and-inactive-slots).


**ROMDATA-01/05-CONSTRUCTOR-SLOT-ACTIVATION chiuso per gate sorgente ed esclusione
mappa (2026-10-28); ordinary ancora parziale.** Otto finestre nuove autenticano
9DB0C→owner+10/pair[owner,9B11D]→9E300/145C10→9B11C(actor,owner); stessi slot394/
3A4del counter e9DBA4.9DAD4 è helper precedente distinto. Oltre movimento eflag20,
ramonormale9B460 richiedeunitFD2, XYsigned0..15, cellavalida/flags02/altezzaS8>
altezzaattoreS16>>4, anim36!=40; bit8000entrata-uscita, mask10000skip, slotlibero.
Rootcellelive02007F10,width+8/base+4,stride2altezza-flags, non nuovoassetROM.
Solo lettura dei64RAMdump esistenti: FD2/flagsattore0/anim0, trecellepercorse
height2/flags0; tutte256celle dichiarate senza02,512Bhash stabile. Questa mappa
immutata nonoffre destinazioni per ilgate; nientealtroMoveallacieca o vecchie
cinquenegative/flagcapture ripetute.87guardie/8hashfinestre esatti/helper0percorsi,
nessun nuovo gameplay/build/testWine-Linux/capture/schema/runtime/ABI/cache/save.
Prossimocriterio: inventariosolalettura snapshotEURnormali esistenti, mappaeattore
idonei eprovenienza; niente stati sintetici o ricreazione copie ritirate. Fullpair
solo dopo uso positivo4fasi/4listeguest-host; ordinary/Linux/USA/Windowsnativo/
nuovo32:9/fullTODO aperti, niente misure/Android/package/commit/push.
[Gate cella e slot](../rom_data_obj.md#constructor-slot-creation-depends-on-live-cell-flags-and-height).


**ROMDATA-05-CONSTRUCTOR-ELIGIBLE-MAP-INVENTORY chiuso negativo (2026-10-28).**
Nuovocriterio solalettura81snapshotEURnormali/provenienza giàverificata: tutti
hash/payload/ROM/domìni16x16validi, soltanto2proiezionimappa512B(1F09/80river),
zero celle02/candidati. Vecchioscan606sequenzealternatenonripetuto.88guardie
snapshot-report-parser-contract-output esatte; nessunbuild/gameplay/testWine-
Linux/capture/checkpoint/manomissione. Boundnegativoterminale su2mappeconservate,
nonallmap/futurescript; nonripetere81statiuguali oMovequi. Parentordinaryancora
parziale; prossimaleadsorgente produttoreflagcelle02007F10/terrainROM prima di
scegliereunamappaidonea. Fullpairsolo dopousopositivo4fasi/4listeguest-host.
Linux/USA/Windowsnativo/nuovo32:9/fullTODO aperti, niente misure/deployment.
[Mappe conservate](../rom_data_obj.md#retained-normal-maps-have-no-eligible-constructor-animation-cells).


**ROMDATA-01-CELL-FLAG-WRITERS chiuso per census/seed candidati EUR (2026-10-28).**
VecchioTERRAIN-MAP-SOURCEdel24giàchiuso/352pacchettiBGriusato, nonnuovoarchivio.
Nuovocensus diretto1DE98nessunBLThumb/puntatoreallineato (nondeadcode/indiretto),
unicoBL1DD68a1D798legge flags02. Nuovaquerysemantica datasetBGautenticato e77:
113auxprogrammi/162link/zerofill-parent-child/orderedu16writes,25programmi e44ID
inizializzano02; flags0/1/2/3/4/8/9. Nessun decoder/risorsa/revisione o accettazione
codec/loader vecchia ripetuta.24guardie/4finestresorgente esatte/helper0percorsi,
nessunbuild/gameplay/testWine-Linux/capture/checkpoint. Quote typo privato corretto
prima esecuzione; cleanupanticipato rifiutaunitasenzarisultato, poipassa dopo
summary, prodottoimmutato. Candidatiasset nonprovano battaglianormale/luogonome;
prossimoactualmapselectionscript/caller→IDidoneo/normalreachablebattle.81snapshot/
5Movebound/flagcapture restanochiusi. Ordinary4fasi/4listeguest-host/fullpair e
Linux/USA/Windowsnativo/nuovo32:9/fullTODO aperti, niente misure/deployment.
[Seed e sorgenti](../rom_data_maps.md#constructor-animation-cell-flag-seeds).


**ROMDATA-01-CONSTRUCTOR-ELIGIBLE-MAP-SELECTION: collegamento sorgente chiuso,
battaglia raggiungibile ancora parziale (2026-10-28).**253C8state10/2549Clegge
CCF00(7)→02002037lowu8→1B270/1B689loader/state11;2540Cètabelladispatch. Opcode12
advance2/callback126831: byte1nonzero→CCF18variable7; zero→70C8CC+40*U16(02002170)
+2→variable7, poimette0200F390=10.51candidatiCCF18, duearmiliteral7autenticati,
nessunallwriterclaim. Queryscriptstyped204istruzioni56default/148literal:
19literalidonee/10argmappa[31,33,34,73,75,89,91,109,115,129]; riferimenti esatti
privati, nonnomiluogo/ordinaryreachability oeventkey=activeindex. Dueesempinormali
retainedactive35/197 maparg99/85→descriptor98/84/defaultcoerenti/inidonei, non81
scanripetuto.37guardie9finestre esatte, nessunbuild/gameplay/testWine-Linux/
capture/checkpoint/runtime/schema/ABI/cache/save. Prossimoactiveindexproducer/
40Brecord70C8CC/mapbyte2→normalquest/event, nienteIDforzato/navigazioneguess.
Positive4fasi/4listeguest-host/fullpair ewholeTODO/Linux/USA/Windowsnativo/32:9
aperti, niente misure/deployment.
[Selezione script e limiti](../rom_data_maps.md#script-map-selection-and-remaining-battle-reachability).


**ROMDATA-01-ACTIVE-ENCOUNTER-RECORDS: produttore e join condizionale chiusi EUR
(2026-10-28), catalogo completo e battaglia ordinary ancora parziali.** Setter9CF0
scrive low16r0/r1 a02002170/172; due BL diretti9D2C/1273A8, nessun puntatore Thumb
allineato, senza pretesa di tutti gli scrittori.9D04 sceglie primary/alternate da
row+2 oppure71D1EC+12*low8(arg1), secondo row+4.63944 collega selettore live219E,
record70B713B3C+45 e row12B71C760;127388 cerca event key su righe255..1, prima
corrispondenza decrescente, duplicati distinti. Query nuova sul dominio u8 del
chiamante:208 righe direct-primary,54 con seed02,44 anche con script default-map.
Non prova disponibilità missioni, nomi o extent del catalogo40B; maxreferenza442
e costante separata300 non sono limiti. Pub normale conservato selettore0/row0/
maparg11 inidoneo, sola lettura.42guardie/23finestre esatte; import Python privato
corretto dopo stop prima di output; originali invariati. Nessun build/gameplay/
testWine-Linux/capture/checkpoint/nuova risorsa. Prossimo criterio: missione/evento
idoneo realmente disponibile da checkpoint normale e UI originale. Niente ID/RAM
forzato o altri Move ciechi;4fasi/4liste guest-host e fullpair ordinary restano
aperti. Vecchie accettazioni non ripetute; Linux/USA/Windows nativo/nuovo32:9/full
TODO pendenti, niente misure o deployment.
[Produttore e prove](../rom_data_maps.md#active-index-producer-and-event-record-links).


**ROMDATA-05-ELIGIBLE-QUEST-LOCATOR chiuso EUR/Wine (2026-10-28): missione idonea
presente nel menu normale.** Osservatore309ba31c/e77 riusato, nessun build: un
percorso noto620frame conserva endpoint menu mancante14969; poi300frame4Down+A
aprono dettaglio White Flowers15269. Non ripetuta come accettazione la vecchia
scena620; nuovo criterio checkpoint/selettori. Originali e copie esatte, nessun
RAM/task/ROM/cart edit. Live F44C02024D5C/count+1B14/puntatori+20+4*ordinal, low10
missionID da62126..5E. Query nuova14entry trova2candidati direct-primary/default-
script/cell02:58/72. Quinta riga e dettaglio visibile confermano58; F4480202389C
+1450→02002268/ID58 con60208..28. Evento58/primary315/maparg69/descriptor68 idoneo;
Giza Plains nella descrizione, ma accettazione e battaglia non ancora raggiunte.
130guardie/496sorgenti frozen/23finestre esatte/helper0percorsi. Layout candidato
031C/03D03001028 scartato perché non legato al menu live. Entrambi workerWine
passano, contatori constructor0 nel pub attesi, non prova positiva/fullpair.
Prossimo conferma normale e verifica accettazione/ingresso battaglia, poi attore/
celle prima di Move guidato.4fasi/4liste guest-host/fullpair, catalogo40Bcompleto,
Linux/USA/Windows nativo/nuovo32:9/fullTODO pendenti; vecchie accettazioni chiuse,
niente misure/deployment.
[Missione normale e prove](../rom_data_maps.md#normal-pub-menu-identifies-an-eligible-quest).


**ROMDATA-05-WHITE-FLOWERS-BATTLE-NAVIGATION parziale EUR/Wine (2026-10-28);
accettazione normale chiusa.** Copia esatta dettaglio ID58→loadoutOK→costo/
obiettivo→condizioniYes; pagamento300gil/fondi7709→7409 e successivo Fee:Paid
provano missione attiva. Voce inizialmente letta QuitPub è QuitMission: aperto
solo dettaglio A:Cancel, nessuna conferma annullamento; B/B/Down/A su Leave porta
alla mappa.27capture normali/4840frame, osservatore309ba31c/e77 già esistenti,
nessun build/manualRAM-task-ROM/cart write. Tutti workerWine passano, table0,
non27accettazioni equivalenza/nessun fullpair. Label mondo Cyril/LutiaPass/
Nubswood/Sprohm, patch centrale esclusa; camera scroll/overshoot documentati.
Non provati Giza Plains raggiungibile/assente/non piazzata.337guardie/496sorgenti/
17hash finestre esplorative esatti.92524init/staletask/031C03001028 non provano
worlddispatch; catalogo165E8indice725Giza e base718/lista3EE60 sono solo lead,
non coordinate. Unità parziale conserva tutti checkpoint diagnostici, niente
ritiro fixture. Prossimo source selector/località piazzate da stati conservati,
nessun altro cursore cieco o27bound/5Move/81map/vecchie accettazioni ripetute.
Battaglia/attore/4fasi4liste guest-host/fullpair e fullTODO/Linux/USA/Windows
nativo/nuovo32:9 restano aperti, niente misure o deployment.
[Accettazione e limiti](../rom_data_maps.md#normal-white-flowers-acceptance-and-open-battle-entry).


**ROMDATA-01-WORLD-LOCATION-SELECTOR chiuso EUR/Wine (2026-10-28).** Corrette
ipotesi ispezione:254B2→6B40, count taskbyte+2 nonword+4; letterale2810/0300
è03002810, non03001028. Test opzionale passivo: un buildWindows senzaavvisi da
snapshot497sorgenti, un frameWine neutro18829→18830/2232read2276site323scoped,
RAM effettiva ugualeparser. Parent[03002810]=0200F3A4; nodes[parent+67C]=08395CA8.
375F8loop30/stride20, byte0/1*8+8centro, hitbox e snap1pixel condizionali;
37D6A..70 cursorview+6/+8,37A44displayY=worldY−scrollC.385CC..38668 savebase+B0+
12*i enabledbyte1bit0/byte3selectoronebased→node(selector−1). Giza index7/catalog
725/node20/source395E24 è piazzata a112,296. Solo lettura dueendpointmeridionali:
21cursor128,287,22cursor128,330; vecchi27bound nonripetuti.152guardie/497sorgenti/
32finestreesatte/helper0percorsi. NuovafamigliaROM30node20B: solo2coordinateB
classificati,18B/schema/modello/estrazione/runtime aperti. Prossimo Leftbreve da
21, verificarelabelGizaprimaA; nientecoordinate/RAM/mapIDforzati. Battaglia e
4fasi4liste guest-host/fullpair/fullTODO/Linux/USA/Windows nativo/nuovo32:9 aperti,
niente misure/deployment/vecchie accettazioni ripetute.
[Selettore e prove](../rom_data_maps.md#world-selector-and-placed-region-coordinates).


**ROMDATA-05-GIZA-BATTLE-ENTRY chiuso EUR/Wine (2026-10-28).** ShortLeft guidato
porta alla label Giza Plains; A viaggio/menu White Flowers/A schieramento.
Primary315/map69/descriptor68, griglia live512B e8427b5e.. uguale seed packet56/
20celle02. Marsh schierato con posizione/direzione confermate; START To Battle/
Yes/dialogo porta a Move. AttoreFD2(6,5), Left/Down/Down cursore effettivo(5,7),
A normale attiva4fasi/4liste guest-host:75frame/22read88B=44threshold44pointer.
Checkpoint22713+suffix11 conservati, non ancora fullpair.16captureWine/3875frame,
309ba31c/e77 esistenti/no build;221guardie496sorgenti esatti/helper0percorsi.
Solo tasti su copie, nessuna modifica manuale guestRAM/task/ROM/cart save.
Primi START prima conferma facing inerti/B torna a placement: corretti e chiusi,
non ripetere. Prossimo singolo CLI positivo11 e confronto completo stato-clock-
immagini-OAM-dispatch-audio dal suffix, senza ripetere scoperta/navigazione.
Cataloghi40B/worldnodes18B/fullTODO/Linux/USA/Windows nativo/nuovo32:9 pendenti;
niente misure/deployment/vecchie accettazioni ripetute.
[Ingresso e prova positiva](../rom_data_maps.md#normal-giza-battle-entry-and-positive-constructor-animation).


**ROMDATA-05-CONSTRUCTOR-ANIMATION-ORDINARY-POSITIVE completato EUR/Wine
(2026-10-28).** Copia esatta nuovo checkpoint normale Giza22713/suffix11→22724.
Una CLI produzione esistente b3bfe6/e77 riusa dataset/cache e passa22read88B/
44threshold44pointer/tutte4fasi. Un fullpair Wine309ba31c passa ogni11stati-clock/
immagini classiche-centrali-estese/OAM/dispatch/cinqueaudio e tutte4liste guest-
host; contatori identiciCLI, dispatch57833/223470,audio24138/24138/2904/2904/23966.
91guardie496sorgenti/72pathmanaged esatti/helper0; niente build/preparazione/
navigazione/discovery/vecchie accettazioni ripetute, guestedit/cartwrite/misure.
Caso ordinario constructor chiuso, non copertura tutti attori/mappe/fullgame.
Linux/USA/Windows nativo/nuovo32:9/fullTODO restano pendenti; cataloghi40B e nuovi
worldnodes18B da completare. Preservato lavoro u16/DBG, niente deployment.
[Prova ordinaria completa](../rom_data_obj.md#ordinary-constructor-animation-positive-giza-scene).


**ROMDATA-01-WORLD-NODE-RECORDS proiezione sorgente EUR chiusa (2026-10-28).**
395CA8..395F00/30record20B/600B, count dal loop375F8già provato; nessun cap
runtime30 dedotto (getterlowu8/workarray32). Producer31488/32BE6scrivono root
[parent03002810+67C]. Byte0/1 coordinate; quattro slot2/6/10/14 pathID/neighbor/
opaque2B più tailopaque2B:300semanticB/300opaqueB conservati.120slot/90linkattivi/
30path0/45path ciascuno in2direzioni. Nodo0valido distinto da path0;4DAD0match
neighbor prima delpath (22target0→0),46EB0stessonodo oppurepath!=0/neighbor.
4DB80graphcopia4slot/salta path0/gate8-27; riferimento397018+(path−1)*8 esterno,
roadcatalog/payload aperti.25finestre(21nuove4riusate)/44pin/42guardie passano,
600B disgiunti da49746span69owner. Nuovo criterio sulla traccia esistente:
35read42site solo coordinate, niente nuova copertura edge o audit ripetuto.
Nessun build/game/test/fixture/guestedit/misura. Prossimo modello tipizzato
lossless/sourceplan e un oracoloWine mirato; estrazione/runtime/CLI/ordinary,
Linux/USA/Windows nativo/nuovo32:9/fullTODO pendenti.
[Schema e lettori](../rom_data_maps.md#world-node-record-source-projection).


**ROMDATA-02-WORLD-NODE-MODEL completato EUR/Wine (2026-10-28).** Catalogo
world_nodes autentica immagine esatta/6finestre/hash600B. Modello30nodi ordinati,
4Link path/neighbor/opaque2B +tailopaque2B, sourceplan1span600B lossless; JSON
stretto/shape distinto da autenticità, niente cap45/29/opaquezero dedotto.
Un buildWindows core-only senzaavvisi/6c2ef15a.., un filtroWine passa23guardie/
566chiamate originali con ritorni/indirizzi letture indipendenti,300semanticB
osservati/300opaqueB preservati nonletti/16aliasbyte/24fuoriproiezione. Callee/
SP/RAM fuori stack/VRAM/OAM/palette verificati, non native/fullstate-clock/BFS/
ordinary.24guardie215sorgenti congelate/helper0 esatti. Nessun gameplay/frontend
build/vecchi oracoli ripetuti; e77-d76 e schema1 invariati. Prossimo pubblicazione
managed/consumo guest tipizzato/CLI reale/scena mondo conservata. Linux/USA/
Windows nativo/nuovo32:9/fullTODO pendenti, niente misure/deployment.
[Modello e prova](../rom_data_maps.md#typed-world-nodes-and-original-reader-proof).


**ROMDATA-03/04-WORLD-NODE-RUNTIME completato EUR/Wine (2026-10-28).** e78-d77
pubblica world-nodes.json/gameplay.world-nodes/index69:71file70risorse, tutti69
record/file precedenti esatti, USAe8-d7 invariato. Binding600B/7campi statistica,
coordinate/path/neighbor/opaque letti dal modello, composizione prior/raw/bordi
senza cap su input guest o sostituzione di piazzamento/pathfinder/ABI/save/cache.
Un filtroWine passa12attach/2fullset/pubblicazione interrotta,15prefissi nativi
fullstate-clock/32Bcompilati124istruzioni/600Braw alterati dopoLoad, alias/mirror/
fuori30-31/equal255/rootRAM. Bus tutti600B/4classi30nodi/width/bordi/priorowner,
Load/Undo/file indisponibili e DMA/CpuSet/CpuFastSet fullstate-clock3 passano.
Primi2build rifiutati da cachecorevecchia: dep-info punta a vecchia copia, touch
nuova copia inefficace; spostato solo fingerprint specifico. Terzo build senza
avvisi/fresh=false/2cf1d882.. da502sorgenti stabili.200guardie/502sorgenti+215
preimmagini modello esatti; helper ritira solo copia interrupted, dataset/eseguibile
accettati conservati. CLI reale/scena ordinaria ancora da fare. Nessun nuovo
gameplay/vecchio modello o proof ripetuto/misura; Linux/USA/Windows nativo/
nuovo32:9/road397018/fullTODO pendenti. Lavoro parallelo DBG/u16 preservato.
[Consumo e prove](../rom_data_maps.md#managed-world-nodes-and-typed-guest-reads).


**ROMDATA-04-WORLD-NODE-CLI completato EUR/Wine (2026-10-28); ordinary parziale.**
Buildproduzione Windows senzaavvisi/e0491cdc.. da502sorgenti stabili; canonico
hash/mtime ripristinati. TreCLIWine exit0 preparecreated/reused/runc normale16,
71file identici runtime/70IDhash esatti/73path managed invariati riuso-avvio,
cache28859block caricata. Copie normali Giza01/02input,19049→19065:402read402B
coordinate/22nodi; path/neighbor/opaque0, nessun fullpair. Matcher privato voleva
frame relativo16, CLI mostra assoluto19065: corretto offline dopo tutti3exit0/
guardie, nessuna ripetizione. Bound16 senza path/neighbor chiuso, prossimo
sourcecallers/stato normale motivato prima di nuova osservazione.169guardie/
502sorgenti esatti/helper0. Nessun nuovo fixture/navigazione/proof precedente/
misura. Ordinary3classi/fullpair/road397018/fullTODO/Linux/USA/Windows nativo/
nuovo32:9 pendenti; lavoro DBG/u16 preservato.
[CLI e limiti ordinari](../rom_data_maps.md#actual-world-node-cli-and-ordinary-coordinate-use).


**ROMDATA-01-WORLD-NODE-ORDINARY-CALLERS e ROMDATA-05-WORLD-NODE-TRAVEL-ORDINARY
completati EUR/Wine (2026-10-28).** Nuovoquery directBL/10finestre27guardie lega
3B012→4DB1C→4DB80 a view1A(dest−1) diverso da parent7BB; output7C1.47034progress!=FF
→4DAD0; entity parent7B8+52*i. Duecheckpoint normali esistenti17/selected20→19/
progressFF e entity1node0→1 motivano bound360, non deduzione PC/frame o tutti
caller. Una CLI360 positiva e un fullpair360 passano6841B=4820coordinate864path
1157neighbor/29nodi, tutti stati-clock/immagini classiche-centrali-estese/OAM/
dispatch/cinqueaudio e contatoriCLI identici. Opaque0 distinto da prove mirate.
Dispatch2006894/3620371,audio790020/790020/95040/95040/789942. Una buildWindows
senzaavvisi/dabb1ac6.. da502sorgenti con guardia opzionale5righe; vecchio validatore
esatto senza guardia, runtime/core invariati.103guardie/502sorgenti/27caller/
169CLI esatti/helper0. Primo16negativo path/neighbor resta chiuso; niente nuovi
fixture/navigazione/guestedit/vecchi proof ripetuti/misure. Casoordinary chiuso,
non tutti nodi/mappe/opaqueordinary/fullpathfinder. Linux/USA/Windows nativo/
nuovo32:9/road397018/fullTODO pendenti, lavoro DBG/u16 preservato.
[Prova ordinaria e gate](../rom_data_maps.md#ordinary-world-node-travel-and-caller-gates).


**ROMDATA-01-WORLD-ROAD-PATHS proiezione sorgente EUR completata (2026-10-28).**
45IDpath dai90link attivi/30nodi selezionano45header8B397018..397180:countu8,
opaque3B,puntatoreROMu32. Payload45distinti contigui395F00..397018/2188coppieXY
signedi8/4376B,46span4736B=4601semantic135opaque. Rootstoreparent680 riusato;
4DCcost byte0,4DD0Cflag/index/steplowu8/nocap45, fuori stepcount outputzero32
senza pointer/sampleload, reverseordine/nonnegazione, duei16packed/returnptr.
4DD70flagcurrent>target/4DD84minnode; calleraggiungeXYnode*8 agli offset,
46B64differenze max5sample/facing nonfullABI.3rootliteral/3pointcalls/1dir/1min
candidatequery, non assenza globaleindiretti.5endpoint1pixelshort4/6/8/10/12
preservati; zero pointassenti/count32..85 nonvincoli.6finestre60pin17guardie/
70owner49747span disgiunti/helper0. Nessun build/game/test/fixture/vecchi proof
ripetuti/misura. Prossimo modello lossless/sourceplan e oracoloWine mirato;
estrazione/runtime/CLI/ordinary/catalogoglobale/opaque meaning/Linux/USA/
Windows nativo/nuovo32:9/fullTODO pendenti. Lavoro DBG/u16 preservato.
[Contratto dei percorsi](../rom_data_maps.md#world-road-path-source-contract).


**ROMDATA-02-WORLD-ROAD-MODEL completato EUR/Wine (2026-10-28).** Modello
world_roads standalone: 45 header ordinati con countu8/opaque3/puntatore codificato,
2.188 coppie XY con segno e sourceplan condiviso 46 span/4.736 B. JSON stretto,
partizione coerente e autenticità distinta; preservati cinque endpoint corti,
zero campione possibile e count0/255 strutturali senza cap32..85 dedotto.
Un build Windows core-only senza avvisi/de9d1e11.. e un filtro Wine passano
29 guardie/2.537 chiamate originali: 2.188 punti forward, 135 reverse, 180 uscite,
6 aliasu8, 6 header esterni e 22 helper ordine. Output/ritorni/sequenze letture
esatti; tutti4.601 B semantici letti, 135 B opachi ricostruiti/non letti.
32 guardie conservazione/504 file build(218 core) esatti/helper0. Nessuna scena
ordinaria/fixture/build frontend/misura o prova nodo già chiusa ripetuta.
Schema1/e78-d77 invariati. Prossimo estrazione managed/letture guest tipizzate,
CLI reale e una scena ordinaria positiva. Linux/USA/Windows nativo/nuovo32:9,
estensione globale catalogo e fullTODO pendenti.
[Modello e prova](../rom_data_maps.md#typed-sampled-world-roads-and-original-reader-proof).


**ROMDATA-03-04-WORLD-ROAD-RUNTIME completato EUR/Wine (2026-10-28).**
e79-d78/index70/gameplay.world-roads/world-roads.json:72file71risorse, tutti70
record/file precedenti esatti, USA8-7/schema1 invariati. Owner immutabile count/
puntatore/sample/opaque da46span4736B; compone nodo mondo al bordo sinistro e
rawdestro/header45..255, nessun algoritmo movimento/facing/pathfinding sostituito.
Due build Windows senza avvisi/accettato52be2981.. da506file stabili; primo filtro
chiude publication/attach, poi fallisce nativecase3 per pagina396 omessa. Fix
pagegate e filtro rimanente native/bus/lifecycle senza ripubblicare:
12guardie attach/2fullset, 12prefix original/interpreter/native fullstate-clock,
67istruzioni compilate/44B espliciti, sorgenti4736B poison dopoLoad;
tutti4736B/4classi45percorsi/widths/mirrors/bordi/priorowner/LoadUndo e DMA3 PASS.
221guardie506sourcehash+504preimage modello esatti; helper ritira solo copia
interrotta. Nessuna scena/modello/nodi/fixture ripetuti, misura o ABI modificata.
CLI reale e una scena ordinaria positiva sono prossimi; Linux/USA/Windows nativo/
nuovo32:9/globalextent/fullTODO pendenti.
[Consumo e prove](../rom_data_maps.md#managed-world-roads-and-typed-guest-reads).


**ROMDATA-04-WORLD-ROAD-CLI completato EUR/Wine (2026-10-28).** Build produzione
Windows senzaavvisi/1a86d341.. da506sorgenti accettati, canonicalhashmtime ripristinati.
Tre comandi Wine preparecreated/reused/runc360:72file esatti alruntime/71IDhash/
74path invariati hashmtime, vecchi28859blocchi cache caricati.
Copie normali Giza01state/02input19049→19409: 2976letture/5067B=
885count/2788puntatore/1394sample/0opaque/40percorsi,
3classi semantiche positive. 180guardie506hash/helper0; nessuna nuova scena,
navigazione, misura o prova giàchiusa ripetuta. Prossimo unico fullpair ordinario
con conteggiCLI identici; non tutti45payload/mappe/opaque ordinari/globalextent.
Linux/USA/Windows nativo/nuovo32:9/fullTODO restano pendenti.
[CLI e consumo](../rom_data_maps.md#actual-sampled-road-cli-and-ordinary-consumption).


**ROMDATA-05-WORLD-ROAD-TRAVEL-ORDINARY completato EUR/Wine (2026-10-28).**
Un fullpair Wine360 normale Giza01/02 con datasetCLI e79-d78,19049→19409:
tutti360stati-clock/classic-central-wide/OAM/dispatch/cinqueaudio esatti.
Sette conteggi uguali CLI:2976reads5067B=885count2788pointer1394sample0opaque,
40percorsi(anyfield, non40payload). Guard opzionale5righe conserva vecchio
validator byteexact; un buildWindows senzaavvisi/4c453f07.. da506file stabili.
100guardie506hash +180guardieCLI esatte/helper0; nessuna nuova scena/fixture/
navigazione/misura o prova source/model/runtime/nodi giàchiusa ripetuta.
Caso ordinario road chiuso; tutti45payload/mappe/opaqueordinari/fullABI/globalextent,
Linux/USA/Windows nativo/nuovo32:9 e fullTODO ancora pendenti.
[Equivalenza ordinaria](../rom_data_maps.md#ordinary-sampled-road-travel-equivalence).


**ROMDATA-01-ENCOUNTER-CATALOG-READERS contratto EUR completato (2026-10-28).**
Catalogo fisico dichiarato442header40B1..442: origin70C8CC, first70C8F4 da due
root indipendenti, next710E04 famiglia28B indipendente.17680B, nonmax442assunto
come count/né costante300/né capguest.442linktemplate contigui2850record48B/
136800B corroborano partizione.17campi/33B semantici e7opachi perheader; indici
originali low16senza guardia442. Index0è prefisso coordinate: ultimi16B payload
header442, niente header valido inventato. Puntatoreheader29+4a70CD58giàowned
constructor-alternates: dependency/coerenza/skip4B,2nuovispan17676B,14582semantic/
3094opaque disgiunti71owner49793span. Template192Bgiàowned preservati; payload
unità/coordinate separati, nonmigrati.27loader da rootcached/40finestre(39new1reuse)/
54pin68guardie/helper0 PASS. Nessun build/game/test/fixture/proofprecedente ripetuto.
Prossimo modello lossless442header/sharedpointer+unoracoloWine; estrazione/runtime/
CLI/ordinary, allindirect/globalextent/Linux/USA/Windows nativo/nuovo32:9/fullTODO
pendenti, nessuna misura.
[Header e ownership](../rom_data_maps.md#european-encounter-headers-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-HEADERS-MODEL completato EUR/Wine (2026-10-28).**
Modelloencounter_headers lossless442header1..442:17campi/opaque7,17680Bfull;
2span17676B escludono70CD58len4delownerconstructor-alternates. JSONstretto/shape
separato daautenticità/coerenzadependency; sharedptrmodificato lascia span
immutati maè rifiutato. lowu16alias/None0-443..65535/priorraw, nessun capcount/
ptrnull/enum/opaquezero dedotto. Un buildWindows core-only senzaavvisi5855af92..
e un filtroWine passano29guardie/7523probes:7072prefixsingolocaricamento e451
C0034completi(442+6alias+3fuori), tutti14586semanticB incl4shared osservati,
3094opaqueB ricostruiti/nonletti. Reg/flags/PCprefix, callee/SP/LRgate, RAM fuori
stack4/VRAM/OAM/palette verificati; nonfullABI/importer/nativeclock/ordinary.
33guardie508file(221core) esatti/helper0, e79-d78invariato, niente vecchioracoli/
scena/fixture/misure. Prossimo managed/typedbus/sourcepage70C..710/dependency,
poiCLI/ordinary; nestedpayload/allcaller/global/Linux/USA/Windows nativo/nuovo32:9/
fullTODO ancora pendenti.
[Modello e prova](../rom_data_maps.md#typed-encounter-headers-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-HEADERS-RUNTIME completato EUR/Wine (2026-10-28).**
e80-d79/index71/encounter-headers.json:73file72risorse, tutti71precedenti esatti,
dependencyconstructor-alternates e coerenzasharedpointer. Owner2span17676B
esclude4Bvecchi, sourcepage70C..710,8classi/442headermask/11stats; guestalgoritmi/
indici/ABI/cache/save invariati, USA8-7/schema1. Un buildWindows senzaavvisi
1c4135f5.. da510file stabili, un filtroWine:14guardieattach/3fullset/failedowner-
state,26prefix original/interpreter/native fullstate-clock con17680Brawpoison,
31istruzionicompilate/37Bnuovi+4Bvecchi. Tutti17676nuovi+4sharedB/8classi/442header/
widths/mirrors/bordi/priorowner/LoadUndo/DMA3 PASS. 206guardie510hash+508preimage
modello esatti/helperritira solo interrupted. Nessuna scena/fixture/misura/prova
modello7523/nodi-road-constructor giàchiusa ripetuta. Prossimo CLIreale/ordinary;
nestedpayload/allcaller/global/Linux/USA/Windowsnativo/nuovo32:9/fullTODO pendenti.
[Consumo e prove](../rom_data_maps.md#managed-encounter-headers-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-HEADERS-CLI completato EUR/Wine (2026-10-28).** Un build
produzione Windows senzaavvisi/9d0a772d.. da510sorgenti accettati; canonicalhashmtime
ripristinati. Tre comandi Wine preparecreated/reused/runc700 passano:73file esatti
alruntime/72IDhash/75path no-write, 28859blocchi cache caricati. Copie normali
Giza02menu/03input19409→20109: 17letture/41B, sei classi semantiche
positive/1header/0opaque. 184guardie510hash/helper0; nessuna nuova navigazione,
checkpoint, misura o prova chiusa ripetuta. Prossimo unico fullpair con conteggiCLI
identici; non tutti442header/payload/opaqueordinari/fullABI/globalcatalogo.
Linux/USA/Windows nativo/nuovo32:9/fullTODO pendenti.
[CLI e consumo](../rom_data_maps.md#actual-encounter-header-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-HEADERS-BATTLE-ORDINARY completato EUR/Wine (2026-10-28).**
Un solo fullpair700 normale Giza02menu/03input19409→20109 con datasetCLI e80-d79:
ogni stato completo/clock/immagine classic-central-wide/OAM/dispatch/5audio passa.
Tutti11conteggi esatti allaCLI:17letture41B=12count/1map/4templateptr/0parameter/
1placementcount/20positionptr/3control/0opaque, unheader; sei classi positive.
Parameterordinary e opaque non osservati, prove mirate precedenti distinte.
Una guardia facoltativa5righe/vecchio validator byteexact, un buildWindows senza
avvisi/43fcc614.. da510sorgenti e un filtroWine. 103guardieordinarie+184CLI/510hash
ciascuno esatti/helper0. Nessuna nuova fixture/navigazione/prova chiusa ripetuta
oppure misura. Caso ordinario chiuso; tutti442header/payload/fullABI/globalcatalogo,
Linux/USA/Windows nativo/nuovo32:9/fullTODO restano pendenti.
[Equivalenza](../rom_data_maps.md#ordinary-encounter-header-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-UNIT-TEMPLATES completato per il contratto sorgente EUR
(2026-10-28).** I442link header già autenticati partizionano2.850record48B,
6EA048..70B6A8/136.800B; proiezione completa indipendente. Nuovi lettori separano
selettori/argomenti, cinque equipmentu16, halfword12, bitsetfisico14..27 con
indice dinamico senza cap20B/160bit, controlli28/29/flags2A/2C/2D e opaque2E..2F.
Tutti14.250riferimenti equipment entro376item, selettori coerenti con proprietari
metadata14/metadata52; valori osservati non sono cap. Vecchi192B constructor
6ECA78..6ECB38/record225..228/header29 esclusi dai nuovi136.608B/two-span;
full48coherence e dipendenze header/constructor richieste, nessuna duplicazione
tra49.795span/72owner. Ottofinestre5nuove3riusate/39pin/4BL/41guardie passano,
nessun build/game/testWine-Linux/fixture/vecchia prova o censimento ripetuto/misura.
Prossimo modello rigoroso+un oracoloWine mirato; managed/runtime/CLI/ordinary/
coordinatepool/global/fullABI/Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti.
[Contratto](../rom_data_gameplay.md#european-encounter-unit-templates-and-shared-ownership).


**ROMDATA-02-ENCOUNTER-UNIT-TEMPLATE-MODEL completato EUR/Wine (2026-10-28).**
Modello rigoroso lossless2.850record48B, full136.800B/two-span136.608nuovi,
192B constructorprecedenti esclusi da byte_at ma confrontati integralmente.
Shape/JSON/partitionheader/sharedcoherence/riferimenti item-metadata14-job e
sourceauth separati; fullu8/u16/opaque2, nessun capvalori o160bitguest inventato.
Un buildWindows core-only senzaavvisi/e0ae0d5c.. da512sorgenti/224coreinputs,
un filtroWine passa38guardie/2.100prefissi originali selezionati:1.020LDR,
1.080bit incl120indici oltre20B.1531byte nuovi/102old/4raw distinti osservati,
registri/indirizzi/valori e tuttaRAM/VRAM/OAM/palette/ROM esatti; proiezioneintera
indipendente, nonfullABI/coverage semantica ditutti2.850record. 34guardie512hash+
41sourceguard esatti/helper0; niente nuova fixture, misura o prova chiusa ripetuta.
Prossima pubblicazione e81-d80/index72+consumo tipizzato, poiCLI/ordinary;
Linux-USA/Windowsnativo/nuovo32:9/coordinatepool/global/fullABI/fullTODO pendenti.
[Modello e prove](../rom_data_gameplay.md#typed-encounter-unit-templates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-UNIT-TEMPLATE-RUNTIME completato EUR/Wine (2026-10-28).**
e81-d80/index72/encounter-unit-templates.json,74file73risorse/tutti72precedenti
esatti/cinque dipendenze;192B constructorrestano al vecchioproprietario. Owner
immutabile136.608B/two-span, noveclassi e record/headerstats, routing6EA..70B
locale/globale. Un buildWindows senzaavvisi/3d4b2af2.. da514sorgenti e un filtroWine
passano14attach/7fullset/fallimentitransazionali,27prefissi original-interpreter-
native con stato completo/clock e136800Brawalterati dopoLoad, 25Bnuovi/8Bprior
compilati. Ogni136608Bnuovo+192old/9classi/2846record441header, larghezze/mirror/
confini/LoadUndo e DMAword34202/CpuSet68404halfword/FastSet34208word passano.
215guardie514hash512modelpreimages esatti/helper solo interrupted. ABI/algoritmi
invariati; USA8-7/schema1. ProssimiCLI reale+ordinary; coordinatepool/allcaller/
fullABI/global/Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti, niente misure.
[Consumo e prove](../rom_data_gameplay.md#managed-encounter-unit-templates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-UNIT-TEMPLATE-CLI completato EUR/Wine (2026-10-28).**
Un buildWindows produzione senzaavvisi/afeaef25.. da514sorgenti runtime accettati,
canonicalhashmtime ripristinati. TrecomandiWine preparecreated/reused/runc700
passano:74file esatti alruntime/73IDhash/76path no-write, 28859blocchi cache.
Copie normali Giza02menu/03A700input19409→20109: 186letture/211B,
5record/1header, 8classi positive e1nonosservate. Criterio positivo
selector/argument/equipment/flag soddisfatto; tutti13conteggi conservati perun
fullpair successivo. 184guardie514hash/helper0, niente nuovafixture/navigazione/
misura o proof chiuso ripetuto. Ordinaryfullpair/tutti2850record442header/fullABI/
coordinatepool/global/Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti.
[CLI e consumo](../rom_data_gameplay.md#actual-encounter-unit-template-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-UNIT-TEMPLATE-BATTLE-ORDINARY completato EUR/Wine
(2026-10-28).** Un solo fullpair700 normale Giza02menu/03input19409→20109 con
il datasetCLI e81-d80:ogni stato completo/clock/immagine classic-central-wide/
OAM/dispatch/5audio passa. Tutti13conteggi esatti allaCLI:186letture211B,
35selector/15argument/80equipment/10auxiliary/41bit/10optional/10flag/10control/
0opaque, cinque record e unheader; otto classi semantiche positive. Opaqueordinary
nonosservato, prova bus distinta. Una guardia facoltativa5righe/vecchiovalidator
byteexact, un buildWindows senzaavvisi/33c61082.. da514sorgenti e un filtroWine.
104guardieordinarie+184CLI/514hash ciascuno esatti/helper0; nessuna nuova fixture/
navigazione/misura/proof chiuso ripetuto. Caso chiuso; tutti2850record442header/
fullABI/coordinatepool/global/Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti.
[Equivalenza](../rom_data_gameplay.md#ordinary-encounter-unit-template-battle-entry-equivalence).


**ROMDATA-01-ENCOUNTER-COORDINATES completato per il contratto sorgente EUR
(2026-10-28).**691liste/2342coppieXY/4684B70B6A8..70C8F4 partizionati esattamente
dai linkpointer-count header:nessun buco/alias/overlap fra49797span73owner.
323placement0liste3272B/34random680B/12placement2=88B/322default644B,323headercon
payload119tuttinulli. Rawu8XY preservano letture signedsearch e unsigned; RNG02868
nonnegative15bit/remainder146314/masku16 restano originali, nessuncap10/16 o
fallbacksafe sui null inventato. Novefinestre riusate+tre nuove helper/RNG,41pin/
38guardie passano; niente build/testWine-Linux/game/fixture/navigazione/prove
chiuse ripetute/misura. Prossimo modello rigoroso691liste+coerenzaheader e un
oracoloWine mirato, poi estrazione/runtime/CLI/ordinary. Schema1/e81-d80/USA8-7
invariati; Linux-USA/Windowsnativo/nuovo32:9/allcaller/fullABI/global/fullTODO aperti.
[Contratto](../rom_data_maps.md#european-encounter-coordinate-lists-and-original-readers).


**ROMDATA-02-ENCOUNTER-COORDINATE-MODEL completato EUR/Wine (2026-10-28).**
Modello rigoroso691liste2342coppie rawu8XY/4684B/691span; shape/coerenzaheader/
autenticità separati, vista signed diagnostica, nessunfallbacksafe null/capvalori.
Primo buildcoreWindows fallisceperimportBus neltest; solaimport corretta, seconda
copia516sorgenti accettata/2fe3e236.. senzaavvisi/227coreinputs, preimmagini conservate.
Un filtroWine passa30guardie/8044prefissi originali signed-unsigned su tutti4684B,
12highbit isolati e102prefissi randomoriginali(34liste*3seed). RNG/index/indirizzi/
masku16/scritture/callee-SP/outerRAM-VRAM-OAM-palette-ROM esatti; nonfullABI,
repeatedrandom/exhaustion/nativeclock/ordinary.40guardie516accepted+516failedhash/
38sourceguard esatti/helper0; nessuna nuova fixture/navigazione/prova chiusa
ripetuta/misura. Prossimo managed e82-d81/index73+typedbus, poiCLIordinary;
Linux-USA/Windowsnativo/nuovo32:9/allcaller/global/fullTODO pendenti.
[Modello e prova](../rom_data_maps.md#typed-encounter-coordinates-and-original-reader-proof).


**ROMDATA-03-04-ENCOUNTER-COORDINATE-RUNTIME completato EUR/Wine (2026-10-28).**
e82-d81/index73/encounter-coordinates.json,75file74risorse/tutti73precedenti
esatti/dipendenza header. Owner immutabile691span4684B/quattro ruoli+XY/statistiche
liste-header, routing locale eglobale70B..70C/composizione vecchiowner ai bordi.
Tre build Windows senza avvisi, accettato0f85fbf7.. da518sorgenti. Il primo filtro
Wine passa pubblicazione/14attach/3fullset e si ferma nel harness nativo; il
secondo isola la concatenazione oltre il punto di arresto. Corretto solo il test
con BLpair e due famiglie che escludono i rispettivi stop. Il terzo filtro riusa
il dataset, mantiene tutti22casi e completa native/lifecycle senza ripetere
pubblicazione o attach. I22casi original-interpreter-native coincidono nello
stato e clock completi con4684Braw alterati dopoLoad (18load/indice0gate/3randomprefix),
21Bnuovi/2template/13header compilati. Tutti4684B/691liste323header/
ruoliXY/691confini/width-mirror/LoadUndo/DMA-CpuSet-FastSet passano. 253guardie/
518hash516modelpreimages esatti/helper solo interrupted; algoritmi/ABI invariati.
ProssimiCLI reale+ordinary; Linux-USA/Windowsnativo/nuovo32:9/fullABI/allcaller/
global/fullTODO pendenti. Nessuna fixture/navigazione/misura/vecchio proof ripetuto.
[Consumo e prove](../rom_data_maps.md#managed-encounter-coordinates-and-typed-guest-reads).


**ROMDATA-04-ENCOUNTER-COORDINATE-CLI completato EUR/Wine (2026-10-28).** Un build
Windows produzione senza avvisi/b467d4ad.. da518sorgenti accettati, canonicalhashmtime
ripristinati. TrecomandiWine preparecreated/reused/runc700 passano:75file esatti
alruntime/74IDhash/77path no-write/28859blocchi cache caricati. Copie normali
Giza02menu/03input19409→20109: 20letture20B/1liste1header,
placement0-X-Y positivi; tutti10conteggi conservati. 186guardie518hash/helper0,
nessuna nuova fixture/navigazione/misura/prova chiusa ripetuta. Fullpair prossimo,
non tutte691liste/ruoli/header/ABI. Linux-USA/Windowsnativo/nuovo32:9/global/
fullTODO pendenti.
[CLI e consumo](../rom_data_maps.md#actual-encounter-coordinate-cli-and-ordinary-consumption).


**ROMDATA-05-ENCOUNTER-COORDINATE-BATTLE-ORDINARY completato EUR/Wine
(2026-10-28).** Un fullpair700 normale Giza02menu/03input19409→20109 con datasetCLI
e82-d81: ogni stato completo/clock/immagine classic-central-wide/OAM/5audio passa;
dispatch nativo e fallback eseguiti. Tutti10conteggi esatti allaCLI:20letture20B,
placement0=20, X10/Y10, una lista e unheader. Random/placement2/default nonosservati
qui, restano distinte le prove mirate già concluse. Guardia facoltativa5righe/
vecchiovalidator byteexact, un buildWindows senzaavvisi/de67e015.. da518sorgenti e
un filtroWine. 104guardieordinarie+186CLI/518hash ciascuno esatti/helper0.
Caso chiuso; tutte691liste323headerconpayload/placement-RNG/fullABI/global/
Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti. Nessuna nuova fixture,
navigazione o misura di performance.
[Equivalenza](../rom_data_maps.md#ordinary-encounter-coordinate-battle-entry-equivalence).


**ROMDATA-01-LINKED-METADATA28 completato per il contratto sorgente EUR
(2026-10-28).**347record28B/9716B710E04..7133F8,17root e aritmetica28B,
famiglia successiva4B con due lettori indipendenti. Confine fisico, nessun cap
347 guest. GetterD07A4 low16/low8: scalari, vista interna+12, wordflag/21bit,
selettori34..255→0 senza letture dati. Lettori direttiu16+22/byte24..26;
byte3/15/27 opachi conservati. Tutti347record ricostruiti, nessuna sovrapposizione
con74owner/50488span,20finestre14nuove6riusate/69pin/122guardie/helper0.
Solo contratto statico; modello/Wine/runtime/CLI/ordinario prossimi. Schema1/
e82-d81 invariati. Linux-USA/Windowsnativo/nuovo32:9/global/fullTODO pendenti;
nessun build/gioco/checkpoint/navigazione/misura o prova chiusa ripetuta.
[Contratto](../rom_data_gameplay.md#european-indexed-28-byte-metadata-source-contract).


**ROMDATA-02-LINKED-METADATA28 completato EUR/Wine (2026-10-28).** Modello
lossless347record/9716B, campi unsigned/flag32/byteopachi, shape/sourceauth distinti;
lookup low16/low8 conserva scalari/vista indirizzo+12/21bit/default senza lettura,
nessun cap guest o fallback nullo inventato. Un buildWindows core senzaavvisi/
358dba2c.. da520sorgenti stabili e un filtroWine:27guardie/14517prefissi originali
(12088getter completi1388tail1041inline),8675byte interpretati osservati/1041opachi
ricostruiti,9varianti isolate342prefissi; RAM/callee/SP-LR/VRAM/OAM/palette/ROM
esatti. 36guardiemodello520hash+122sorgente esatti/helper0. Schema1/e82-d81
invariati; prossimi managed/runtime poiCLI/ordinario. FullABI/nextfamiglia4B/
global/Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti, nessuna misura o
nuova fixture/navigazione/vecchia prova chiusa ripetuta.
[Modello e prova](../rom_data_gameplay.md#typed-indexed-28-byte-metadata-and-original-reader-proof).


**ROMDATA-03-04-LINKED-METADATA28-RUNTIME completato EUR/Wine (2026-10-28).**
e83-d82/index74/linked-metadata28.json:76file75risorse/all74precedenti esatti,
uno span9716B/nessuna dipendenza inventata. Owner tipizzato immutabile5classi/
347record/8contatori, pagine locali+globali710..713, bordi headerprecedente/
successor grezzo. Un buildWindows senzaavvisi/040133f5.. da522sorgenti, un filtroWine
pubblicazione/interruzione14attach2fullset/23casi nativi stato-clock completi con
9716byte grezzi alterati dopoLoad:17load+6getterD07A4,31byte nuovi+1header compilati.
Tutti9716byte/5classi347record/tutti347bordi/LoadUndo/DMA-CpuSet-FastSet3stati-clock
passano. 216guardie522hash520preimmagini modello esatti/helper ritira solo copia
interrotta. ActualCLI/ordinario prossimi; ABI/USA8-7/schema1 invariati, nessun
nuovo gioco/checkpoint/navigazione/misura o vecchia prova chiusa ripetuta.
FullABI/global/nextfamiglia4B/Linux-USA/Windowsnativo/nuovo32:9/fullTODO pendenti.
[Consumo diretto](../rom_data_gameplay.md#managed-indexed-28-byte-metadata-and-typed-guest-reads).


**ROMDATA-04-LINKED-METADATA28-CLI registrazione completata EUR/Wine;
uso ordinario positivo ancora aperto (2026-10-28).** Un buildWindows produzione
senzaavvisi/0acb352e.. da522sorgenti: sola correzione di3messaggi metadata28, inverse
byteexact/altri521file esatti/canonicalhashmtime ripristinati. Creazione/riuso/
avvii normali:76file esatti alruntime/75IDhash/78path no-write/28859blocchi cache.
Quattro locatori normali Giza700/Jobs540/Abilityset700/BattleTechRinfo800 hanno
TUTTI8contatori0, conservati come negativi da non ripetere; nessun fullpair
eseguito. Menu osservati, nuovi input da normali comandi su copia autorizzata;
nessun checkpoint nuovo o edit manuale guest. 220guardie522hash/helper0.
Prossimo: percorso di combattimento con azione non nulla prima di positivo/fullpair.
Linux-USA/Windowsnativo/nuovo32:9/fullABI/global/fullTODO pendenti; nessuna misura.
[CLI e limiti](../rom_data_gameplay.md#actual-indexed-metadata28-cli-registration-and-ordinary-locators).


**ROMDATA-04-05-LINKED-METADATA28-COMBAT completato EUR/Wine (2026-10-28).**
Apertura normale del menu Steal su terreno asciutto a Giza:42frame22669→22711,
15letture20byte serviti/cinque record,10halfword+10byte; inline/flag/opachi0.
Cattura, CLI esistente0acb352e.. e un fullpairWine coincidono in tutti8conteggi;
ogni stato completo/clock/immagine classic-central-wide/OAM/5audio passa. Nessun
effetto abilità eseguito o tutti347record rivendicati. Un buildWindows test senza
avvisi/5a6c81be.. da523sorgenti, guardia facoltativa5righe/vecchiovalidator byteexact
ed osservatore di soli input normali; sette catture1182frame su copie autorizzate,
nessun edit manuale guest/salvataggio originale. 177guardie523hash+220CLI/522hash
esatti/78path no-write/helper0. Quattro vecchi locatori negativi restano chiusi;
non si ripetono source/model/runtime/registrazione o prove di altre famiglie.
Caso ordinario chiuso; effetti/tutti347/fullABI/global/nextfamiglia4B/Linux-USA/
Windowsnativo/nuovo32:9/fullTODO pendenti, nessuna misura di performance.
[Equivalenza](../rom_data_gameplay.md#ordinary-metadata28-steal-menu-equivalence).


**ROMDATA-01-LINKED-RECORDS4 completato per il contratto sorgente EUR
(2026-10-28).** Intervallo7133F8..713744,844 byte in211 posizioni fisiche da4 byte;
sei root e lettori originali con indice u8 moltiplicato per4, confine distinto
con la famiglia successiva a passo8. Tutti i quattro campi sono letti unsigned;
indirizzi originali conservati nel contesto live. 1041 riferimenti dalle prime
tre posizioni inline dei347 record metadata28 raggiungono194 posizioni distinte,
tre riferimenti a indice0 e nessuno esterno. Nessun cap211/max208, limite globale
ai tre indici, enum o fallback nullo inventato. Gli zeri0/1/209/210 restano record
fisici distinti; significato riserva/padding dei due finali ancora da chiarire.
Ricostruzione completa844 byte, nessun overlap con75 risorse/50489 span e83-d82;
13 finestre autenticate (6 nuove,7 riusate),45 pin e112 guardie esatti/helper0.
Solo analisi statica, senza build/test/gioco/fixture o ripetizione di prove chiuse.
Prossimo: modello e prova mirata dei lettori originali su Wine; gestione/runtime/
CLI/uso ordinario di QUESTA famiglia ancora da implementare. Linux/USA/Windows
nativo/nuovo32:9/fullABI/cataloghi indiretti/fullTODO pendenti; nessuna misura.
[Contratto](../rom_data_gameplay.md#european-linked-four-byte-record-source-contract).


**ROMDATA-02-LINKED-RECORDS4 completato EUR/Wine (2026-10-28).** Modello core
lossless:211 posizioni fisiche con quattro campi u8/844 byte/uno span; shape,
ordine/indici/parser stretti separati dall'autenticazione EUR e13 finestre.
Indirizzo guest originale calcolato per tutti256 indici, anche211..255 esterni;
None diagnostico non diventa fallback guest. Zeri distinti e significato dei due
finali ancora aperto, nessun enum/cap/limite globale ai selettori inventato.
Un buildWindows core senza avvisi/a7270580.. da525 sorgenti stabili (233 input
core), un solo filtro Wine:20 guardie/1567 chiamate o prefissi originali,
1055 lettori coprono tutti844 byte e512 setter completi coprono entrambi i
contesti/tutti256 indici/alias low8/indirizzi esterni. Due varianti isolate con
bit alto passano10 prefissi unsigned e sono rifiutate dall'autenticazione base.
RAM completa salvo i5 byte attesi dei setter/callee/SP-LR/IWRAM/VRAM/OAM/palette/
ROM finale esatti;9808 istruzioni come sola copertura. 125 guardie525 hash e112
sorgente esatti/helper0. Schema1/e83-d82/save/native/CAPI invariati. Prossimo:
managed/runtime poi CLI/uso ordinario di questa famiglia; nativeclock/fullABI/
Linux/USA/Windows nativo/nuovo32:9/cataloghi indiretti/fullTODO pendenti. Nessun
gioco/checkpoint/misura/ripetizione di prove precedenti chiuse.
[Modello e lettori](../rom_data_gameplay.md#typed-four-byte-linked-records-and-original-reader-proof).


**ROMDATA-03-04-LINKED-RECORDS4-RUNTIME completato EUR/Wine (2026-10-29).**
e84-d83/index75/linked-records4.json:77 file76 risorse, tutti75 record/payload
precedenti esatti, uno span844 byte senza dipendenze inventate. Owner tipizzato
immutabile/quattro campi/211 posizioni/sette contatori; pagina713 locale+globale,
composizione per byte con metadata28 precedente e successore grezzo. Nessun
nuovo cap/null/limite ai selettori, ABI save/native/CAPI e USA8-7 invariati.
Tre buildWindows: primo fallito per riferimento del test e cache menu obsoleta,
secondo senza avvisi mai eseguito, terzo senza avvisi33144a8c.. da551 sorgenti
correnti; sole attese di allineamento LDR corrette prima di Wine. Un filtro Wine
passa pubblicazione/interruzione14attach2fullset/owner-stato intatti/21casi nativi
stato-clock completi con tutti844 byte grezzi alterati dopoLoad:13load4getter,
un prefisso doppio e3setter/quattro gruppi;41 istruzioni/21byte nuovi6precedenti.
Tutti844byte/quattro campi211posizioni/tutti211bordi/LoadUndo stessoArc e DMA3/
CpuSet424halfword/FastSet216word passano. 237 guardie551hash +due551diagnostici/
525preimmagini modello/sette hash correnti esatti; helper ritira solo copia
interrotta. Prossimo CLI reale/uso ordinario; Linux/USA/Windows nativo/nuovo32:9/
fullABI/cataloghi esterni/fullTODO pendenti. Nessuna misura/gioco/fixture nuova/
ripetizione di vecchie prove chiuse.
[Consumo diretto](../rom_data_gameplay.md#managed-four-byte-linked-records-and-typed-runtime-reads).


**ROMDATA-04-LINKED-RECORDS4-CLI registrazione completata EUR/Wine;
uso ordinario positivo pendente (2026-10-29).** Un buildWindows produzione senza
avvisi/bcf632fa.. da551 sorgenti identiche al runtime, senza overlay; soli
fingerprint corelib/prod isolati e canonical hashmtime ripristinati. Tre figli
CLI Wine escono0 (creazione/riuso/nuovo avvio120):77 file esatti al runtime,
76 ID/hash caricati,79 percorsi gestiti hashmtime no-write,28859 blocchi cache.
Copia del menu Steal22711 con Armor selezionato, A8hold12→22831 mostra puntamento
ma tutti7 contatori nuovi sono0. Caso negativo chiuso, nessun fullpair o uso
positivo dedotto; checkpoint finale non salvato dal comando CLI. 187 guardie/
551 hash CLI+551 runtime esatti/helper0, originali invariati; nessuna nuova
fixture o ripetizione di vecchie prove chiuse. Prossimo: risoluzione normale di
combattimento/bersaglio con letture positive e poi equivalenza completa. Linux/
USA/Windows nativo/nuovo32:9/fullABI/cataloghi esterni/fullTODO pendenti, nessuna
misura o modifica Android/package/commit/push dell'assistente.
[CLI e limiti](../rom_data_gameplay.md#actual-linked-four-byte-record-cli-registration).


**ROMDATA-04-05-LINKED-RECORDS4-COMBAT completato per un caso ordinario EUR/Wine
(2026-10-29).** Normali comandi Fight/Doit/facing portano al turno seguente;
suffix40 frame24755→24795,223 letture/223 byte serviti, campi64/132/11/16 e due
posizioni fisiche. Tutti7 contatori identici tra cattura, CLI BCF esistente e
un fullpairWine; ogni40 stato completo/clock/pixel classic-wide/OAM/5audio passa.
Un buildWindows senza avvisi/a5b2fc84.. da552 sorgenti: solo osservatore normale,
modulo test e guardia facoltativa5righe; inverse validator esatto/core invariato.
Dieci catture2084 frame su copie autorizzate, nessun edit manuale guest/ROM/cart;
un frameCLI classico soloUI, poi proiezioni framebuffer readonly autenticate senza
altra esecuzione. All76IDhash/79path no-write/28859cache, 210 guardie552hash+
187CLIguardie551hash esatti/helper0. Zero-case precedenti restano chiusi; nessuna
ripetizione di source/model/native/lifecycle/registrazione. Tutti4 campi osservati,
non tutte211 posizioni o successo attacco/fullABI. Nuova famiglia8B/cataloghi
indiretti/Linux-USA/Windows nativo/nuovo32:9/fullTODO pendenti, nessuna misura.
[Equivalenza](../rom_data_gameplay.md#ordinary-linked-four-byte-turn-equivalence).


**ROMDATA-01-CONDITIONAL-RECORDS8 contratto sorgente EUR completato
(2026-10-29).** Scansione D56A0 a root713744, passo8 e ordinali1..127:
127 posizioni/1016 byte fino al distinto root713B3C, i cui due lettori usano
passo70. Byte00 zero salta la posizione, byte01 zero wildcard; chiavi LE16 a
+2/+5 e operandi u8 a+4/+7. Flag live fino05FF e contatori u8 per key-0600:
primo flag richiede uguaglianza per operandi0/1 e counter>=operand; secondo flag
richiede disuguaglianza e counter<operand, solo seconda key0 bypassa. Nessuna
coercizione booleana/cap/indice header inventato.76righe abilitate/51zero,
105riferimenti live104flag1counter/63chiavi/24gruppi alias; operand2 preservato,
47secondkey0. Output puntatori ROM originali in ordine, count; capacità/filtro
u32 senza entryguard capacità0, entrambi chiamanti osservati passano64/filtro u8.
Helper370EB0/145C10 resta supporto eseguibile già classificato, non nuovo asset.
SHA256d52e829c2ae2e3f8a519fc1fe28797a1005fcdc0e2afc81201e872024608b18b,
ricostruzione completa/disgiunto76owner50490span;8finestre6nuove2riusate/93pin/
96guardie esatti/helper0. Query parole puntatore allineate:1root/38successore,
nessun interno/origin-8 in quella rappresentazione;2callsite verificati.
Solo analisi statica, nessuna build/Wine/gioco/fixture/vecchia prova ripetuta;
schema1/e84-d83/77file76risorse invariati. Prossimo modello e lettori Wine della
famiglia; managed/runtime/CLI/uso ordinario/Linux-USA/Windows nativo/nuovo32:9/
fullABI/caller indiretti/estensione successore/fullTODO pendenti. Nessuna misura.
[Contratto](../rom_data_gameplay.md#european-conditional-eight-byte-record-source-contract).


**ROMDATA-02-CONDITIONAL-RECORDS8 modello/oracolo EUR/Wine completato
(2026-10-29).** Modello core lossless127posizioni/1016byte/uno span; parser,
shape/ordine stretti separati dall'autenticazione baseROM+8finestre. Gate/filter/
chiavi LE16/operandi u8 conservati senza enum/cap/null inventati; select conserva
puntatori originali/ordine/capacità u32 zero-wrap/filtro fullu32 e molteplicità
chiamate live, polarità primo/secondo diverse. Un buildWindows core senza avvisi
c6f5406d.. da554sorgenti/239inputcore; sola nuova foglia+test+modulo, inversehub
byteexact, isolato fingerprintcoretest. Un filtroWine passa24guardie e109cicli
originali D56CE..D57A4 con veri lettori CCE98/CCF00:16scenari base,93varianti
sintetiche isolate con chiavi0/05FF/0600/0682/FFFF, operandi0..255, header255,
filtro128/255 vs contesto completo, capacità0/1/64/FFFFFFFF. OraclePython e
modello concordano con output/count/capacità/posizioni/chiamate effettive;
612byte base e134posizioni sintetiche osservate, non tutti1016byte letti.
EWRAM esatta salvo output atteso/IWRAM salvo scratch4byte returnlink/stack-r11r12/
VRAM-OAM-palette/ROMfinale esatti;198372istruzioni sola copertura.120guardie554hash/
3hashcorrenti+96guardiesorgente esatti/helper0; nessun gioco/checkpoint/navigazione/
vecchia prova o helperIWRAM ricostruito. Helperentry/epilogo/fullABI esclusi;
schema1/e84-d83/76risorse/save/native/CAPI invariati. Prossimo managed/runtime
poi CLI/uso ordinario; Linux-USA/Windows nativo/nuovo32:9/indiretti/successore70B/
fullTODO pendenti. Nessuna misura di performance o spazio.
[Modello e ciclo originale](../rom_data_gameplay.md#typed-conditional-records-and-original-selection-loop-proof).


**ROMDATA-03-04-CONDITIONAL-RECORDS8-RUNTIME completato EUR/Wine con prove
combinate (2026-10-29).** e85-d84/index76/conditional-records8.json:78file77risorse,
76precedenti esatte/uno span1016B/nodeps. Owner immutabile6classi/127slot/9contatori,
pagina713 locale+globale e composizione records4 precedente/successore grezzo;
logica guest/live/indirizzi e ABI save/native/CAPI invariati. Quattro buildWindows
senza avvisi da568sorgenti correnti coerenti; primo mai eseguito dopo correzione
pre-Wine del solo limite di iterazioni test. Primo filtro Wine passa pubblicazione/
interruzione14attach2fullset+16casi nativi stato-clock completi, poi fallisce nel
17esimo per osservatore607vs609. Diagnosi solo caso residuo prova stato-clock
raw/typed/native e609byte reali identici, completa bus/lifecycle; finale solo
osservatore passa609callback=607diretti+2fallback interp_one, nessun byte non
attribuito. Solo test corretto; non si ripetono pubblicazione/attach/16casi/bus.
17casi distinti:14load2prefissi LE1scansione; tutti1016byte grezzi alterati DOPO
Load/setup. Totale631byte serviti dal percorso nativo=629diretti2fallback e5byte
records4, non unione distinta o tutto compilato. Tutti1016busB/6classi127slot/
127bordi/failedowner-fullstate/LoadUndo stessoArc e DMA3/CpuSet510halfword/
FastSet256word fullstate-clock passano.261guardie568hash accettati+tre568diagnostici/
554preimmagini modello/7hashcorrenti/77vecchi78nuovi file esatti; helper ritira
solo copia interrotta. Exe accettato f60055e6.., primo fallimento conservato;
nessun gioco/checkpoint/navigazione/prova precedente o misura di performance.
Prossimi CLI reale e uso ordinario; Linux-USA/Windows nativo/nuovo32:9/fullABI/
indiretti/successore70B/fullTODO pendenti.
[Consumo e prove combinate](../rom_data_gameplay.md#managed-conditional-records-and-typed-runtime-reads).


**ROMDATA-04-CONDITIONAL-RECORDS8-CLI registrazione/avvio EUR/Wine completati
(2026-10-29).** Un buildWindows produzione senzaavvisi72d44985.. da568sorgenti
byteidentiche alruntime/nooverlay, fingerprintcorelib+prod isolati e canonical
hashmtime ripristinati. Tre figli CLI Wine escono0: creazione e85-d84/riuso/avvio
neutro1;78file esatti alruntime/77IDhash caricati/80path gestiti nowrite/28859blocchi
vecchi. Primo prepare0 ha successo; helperlocale controllavaindice75 anziché76:
corretto e ripresi solo due figli rimanenti, senza rifare prepare o build.
Copia normale before-activation24755→24756 coninputneutro, tutti9contatori nuovi0;
nessuna azione/navigazione/checkpoint nuovo/endpoint salvato, nessun positivo
ordinario/fullpair dedotto.189guardie568CLI+568runtimehash esatti/helper0, originali
invariati. Source/model/native/lifecycle/vecchie scene restano chiusi. Prossimo
uso ordinario positivo e confronto completo; Linux-USA/Windows nativo/nuovo32:9/
fullABI/indiretti/successore70B/fullTODO pendenti, nessuna misura o modifica
Android/package/commit/push dell'assistente.
[CLI e limiti](../rom_data_gameplay.md#actual-conditional-record-cli-registration).


**ROMDATA-04-05-CONDITIONAL-RECORDS8-PUB completato per un caso ordinario
EUR/Wine (2026-10-29).** Ingresso normale Rumors nel pub; suffix42f15181→15223,
468letture/468byte SERVITI, sei classi127/76/152/76/34/3 e127posizioni fisiche
incluse51intestazioni zero, non127record attivi/tutti campi/tutti rami/unione byte.
Tutti9contatori esatti tra cattura/CLI72d44985 efullpair63584945; ogni42stato/clock/
pixel classic-wide/OAM/5audio uguale. Un buildWindows senzaavvisi da569sorgenti:
CLI568+osservatore passivo+modulo+guardia opzionale5righe; inversehub-validator
byteexact/core-runtime invariati. Due catture normali254frame: B180negativo poi
Up3/A12positivo74, primo readrel42+32continuazione, rollingrel32→suffix42.
Seed MissionsLIST14969 giàconservato, nessun vecchio620/397route replay; vecchio
PPMclassic240x160 scalacolore*255//31 autenticato controFFSTATE, nonformato wide
recente. Proiezioni readonly eimmagini verificate, nessun WineUI/guestedit/cart
write; promptfinaleHeardlatestRumors, lista/selezione/esito non dedotti.
143guardie569hash+568CLI/3roothashes/80managedpaths nowrite/28859cache esatti,
helper0. CLIe85generationhw22uM riusata senza prepare/build/navigazione; primo
B0case resta chiuso. Declaredsource/model/runtime/CLI/ordinario8B chiusi;
prossimo nuovo root713B3C/passo70 richiedecontratto proprio. Linux-USA/Windows
nativo/nuovo32:9/fullABI/indiretti/fullTODO pendenti; nessuna misura o modifica
Android/package/commit/push dell'assistente.
[Equivalenza](../rom_data_gameplay.md#ordinary-conditional-record-rumors-entry-equivalence).


**ROMDATA-01-MISSION-RECORDS70 contratto sorgente EUR completato
(2026-10-29).** Catalogo713B3C..71C73C/512recordx70/35840B, due scansioni
indipendenti1..511 più getterindice0/confine successore con lettori propri;
costante300 separata, nessun nuovo cap guest512. Getter65selettori indice low16/
selettore low8: LE unsigned da byte, bit impacchettati, flag, mapping zero oppure
+0177 e scala200,7selettori indirizzo/6offset inline, alias19/20 e viste24/32 con
scalari condivisi. Byte residui/valori grezzi/identità ordine preservati. Mission
link giàchiuso riusato: campo+45HEX69decimale eventrow*12+71C760;512riferimenti/
173valori/quattrogruppi alias, zero dipende dal chiamante; catalogo eventi distinto
ancora da classificare. Tutti35840B ricostruiti, SHA256
bbdbc955b9b3f7ed678123aa6bea8b6098f25f6a13a4ab94de669d1e103253dc, disgiunti
77owner50491span/e85-d84/78file invariati.19finestre9nuove10cache/157pin110guardie
esatti; vecchio testo autenticato opcode-per-opcode contro ROM e hash corrente,
nessuna falsa guardia testo storica.38literalbase/2origin+70/2candidati interni
in dati non classificati lettori, query allineata non esaustiva indiretti. Solo
statica, nessuna build/Wine/gioco/fixture/navigazione/prova precedente ripetuta;
helper ritira zero path. Prossimo modello rigoroso lossless/oracolo getter Wine,
poi managed/runtime/CLI/ordinario. FullABI/semantiche campi ignoti/conteggi viste/
successori/Linux-USA/Windows nativo/nuovo32:9/fullTODO pendenti. Nessuna misura di
performance o spazio; lavoro parallelo menu/content/DBG/package preservato.
[Contratto](../rom_data_gameplay.md#european-mission-record-source-contract).


**ROMDATA-02-MISSION-RECORDS70 modello/oracolo EUR/Wine completato
(2026-10-29).** Nuovo core mission_records70 lossless512record/35840B/unospan,
shape/ordine/JSON rigorosi separati da autenticazione ROM esatta+19finestre.
Byte residui/array inline/operandi grezzi e tutti65selettori preservati; query
indice low16/selettore low8, Some0 invalido senza lookup, indirizzi calcolati
anche fuoriowner, scalare nonpossedutoNone diagnostico senza nuovo cap guest.
Un buildWindows core senzaavvisi663e8b87.. da571sorgenti/242inputcore,569preimmagini
esatte e inversehub byteexact; sole duefoglie/pubmodule, nessun frontend parallelo.
Un filtroWine passa21guardie e698chiamate getter originale D1F30 finoalreturn con
vero helperCEEEC:438base/all65agliindici0/1/58/299/300/511/normalizzazione/default/
viste fuoriowner/scalari512-65535,260varianti sintetiche isolate suquattro70Bpattern
alrecord511. Pythonindipendente/modello/risultati eordine lettureLDRB effettivo
esatti;324byte base distinti/54sintetici contro35840B ricostruiti separatamente,
viste non dereferenziano16B inline. EWRAM/VRAM/OAM/palette invariati, IWRAM esatta
inclusi linkreturn previsti, SP/R4..R12/Thumb/ROMfinale esatti;17213istruzioni
solacopertura.139guardie571hash/3correnti569preimmagini+110guardiesorgente/helper0.
Nessun gioco/checkpoint/cartwrite/navigazione/prova giàchiusa o misura. Schema1/
e85-d84/77risorse78file/save/native/CAPI invariati. Prossimo managed/runtime/
nativeclock/lifecycle poiCLI/ordinario; fullABI/campi ignoti/indiretti/successori/
Linux-USA/Windows nativo/nuovo32:9/fullTODO pendenti. Lavoro parallelo preservato.
[Modello e getter](../rom_data_gameplay.md#typed-mission-records-and-original-getter-proof).


**ROMDATA-03-04-MISSION-RECORDS70-RUNTIME completato EUR/Wine con prove
combinate (2026-10-29).** e86-d85/index77/mission-records70.json/79file78owner,
77precedenti esatti/uno span35840B/nodeps; ownerimmutabile10classi13contatori/
8maskword512posizioni/pagine713..71c, composizione8Bprecedente e successori grezzi.
Due buildWindows senzaavvisi573sorgenti/243inputcore:73326ad9 prima pubblicazione/
prior77/interruzione PASS poi primoattach FAIL cap storicoOBJ1024, altri test
nonpartiti. Con richiesta no-limiti-spazio rimossi capattachOBJ1024/BG64KiB e
treasserzioni di allocazione, parsebounds/sourceauth/binding invariati.455bdb1c
secondo build/filtroWine SOLOresidui sulla STESSAgenerazione79file:14attach2fullset/
failedownerfullstate/41nativi stato-clock(16load25getter) con35kgrezzi alterati
DOPOload/setup/all35kbus10classi512slot/512bordi/LoadUndo stessoArc/DMA3passano.
Pubblicazione nonripetuta, primo fallimento conservato; nessun capfix interpretato
come ottimizzazione o misuraRSS. Tutti callback inclusointerp_one attribuiti:
52byte nuoviserviti5precedenti/fallbackdelta0, nonunione distinta/tuttocatalogo
nativo;3famiglie stop-safe/168entry69dispatch sola copertura. DMA/CpuSet17922
halfword/FastSet8968word statocompleto-clock identici.860guardie573accettati+573
diagnostici/571modello/8hashcorrenti/78vecchi79nuovifile esatti;helper ritira solo
copiainterrotta. close-final.pyPASS-combined-scopes, vecchio close-runtime.py
non eseguito. Source/modello/vecchie scene nonripetuti, nessun gioco/checkpoint/
navigazione/cartwrite/ABI Machine-save-native-CAPI cambiato. ProssimoCLI reale
poiordinario; Linux-USA/Windows nativo/nuovo32:9/fullABI/indiretti/successori/
fullTODO pendenti. Nessuna misura performance/spazio, lavoro parallelo escluso
dabuild e preservato.
[Consumo e prove](../rom_data_gameplay.md#managed-mission-records-and-typed-runtime-reads).


**ROMDATA-04-MISSION-RECORDS70-CLI registrazione/avvio EUR/Wine completati
(2026-10-29).** Un buildWindows produzione senzaavvisi738b7f20.. da573sorgenti
byteidentiche alruntime/nooverlay, fingerprintcorelib-prod-menu coerenti e
canonicalhashmtime ripristinati. Tre figliWine tutti0: preparazionecreata/riuso/
avvio neutro1,79file esatti alruntime/78IDhash/81path gestiti nowrite/28859vecchi
blocchi nativi. Generatoreprivato fermato su assertiontokenindice prima diogni
Wine: correttasola generazionehelper conindice77 verificato, nessuna copia
sorgenti/build/preparazioneCLI ripetuta. Copianormale24755→24756 all13contatori0;
nessuna azione/navigazione/checkpoint nuovo/endpoint salvato, positivoordinario
fullpair o nuova fase negativa del locator dedotti.195guardie573CLI+573runtime/79file/
originali/nativecache/loader/canonical esatti;helper0. Source/model/native/
lifecycle/vecchieprove restanochiusi. Prossimo positivoordinario e confronto
completo; Linux-USA/Windows nativo/nuovo32:9/fullABI/indiretti/successori/fullTODO
pendenti, nessunamisura/Android/package/assistantcommit/push.
[CLI e limiti](../rom_data_gameplay.md#actual-mission-record-cli-registration).


**ROMDATA-04-05-MISSION-RECORDS70-MISSIONS completato per un caso ordinario
EUR/Wine (2026-10-29).** Apertura normale informazioniGizaPlains, suffix33frame
15017→15050:5letture/5byte SERVITI solo classepacked/una posizione fisica, altre
9classi0. Tutti13contatori esatti cattura/CLI738b7f20/fullpair3d21e89c; ogni33stato
completo-clock/pixelclassic-wide/OAM/5audio uguale, entrambelebraccia stessa
vecchiacachenativa/nonparitàglobaleinterprete-nativo. UnbuildWindows senzaavvisi
574sorgenti=CLI573+osservatorepassivo/pubmodule/guardiaopzionale5righe, inversehub
byteexact/core-runtime invariati. UNA cattura81f daMissionsLIST14969 giàconservato:
Down3at8/A12at40, primalettura rel49/guest15019, rolling48→suffix33 A4+neutral29;
ricercaSTOP. TestWinechild0 PASS; helperpostcatturaKeyErrorsteps dopooutputcompleti
riparatosoloregistrazione, nessuna cattura/input/Wine/build ripetuta. Seedclassico
240x160/scale giàautenticato riusato; nuovoPPMwide284x160/bitrep verificatocontro
FFSTATE completo, classicPNG mostraGizaMISSIONinfo page1/3Rank1. Nessuna missione
accettata/battaglia/esito/tutticampi/all512/unione5bytedistinti dedotti.
133guardie574hash+573CLI/3correnti/78IDhash81managedpaths nowrite28859cache esatti,
helper0/originali invariati. Vecchie620/397route/source-model-runtime/registrazione
non ripetute, nessunaforzaturaID/RAM/cartwrite. Declared70Bchain chiusa; nuovi
arrayC73C/C740 eeventC760 richiedonocontratti propri. Linux-USA/Windows nativo/
nuovo32:9/fullABI/indiretti/altreclassiordinarie/fullTODO pendenti; nessunamisura
performance/spazio/Android/package/assistantcommit/push.
[Equivalenza](../rom_data_gameplay.md#ordinary-mission-information-equivalence).


**ROMDATA-01-SELECTION-BYTE-TABLES contratto sorgente EUR completato
(2026-10-29).** C73C..C740 quattro soglie di conteggio eC740..C75E trenta pesi
ordinali:34byte/two span, SHA256d758af5f1a07613f1c2047257d42ef9cbfac83b63783ee6fd0e7b3879370b8dd.
Cinque header live02002E34+6*i contati solo low3bit nonzero, tutti esaminati;
guard<=3 prima dellaloadu8 soglia,4/5escono. Secondoroot indipendente, duecicli
stride1 ordinali1..30/posizioni0..29. Verohelper377BC leggeu8 grezzo live
02002CC3+12*(low8ordinal-1), nessun guardglobale0/30 o normalizzazionebool;
callertesta low8nonzero. Somma0esceprima secondo stadio; rangohelper full32
meno peso unsigned conwrap e confrontosigned, primonegativosceglie,31esaurito
esce; nessunguardnegativo/zero-weight/cachedeligibility inventato. Numerichelper
2868/146314/RNG/distribuzione/fullABI nonmodellati. Tutti34B ricostruiti/posizioni
conservate/valori ripetuti nondeduplicati/nopuntatori dati; disgiunti78owner
50492span/e86-d85/79file invariati.6finestre2nuove4cache/95pin96guardie/helper0,
sole2nuovecontestocount ehelper live; nessuna build/Wine/gioco/fixture/navigazione/
vecchiaprova ripetuta o misura. GapC75E..C760 dueB opachi nonposseduti, next12Broot
C760indipendente/estensione pendente. IDneutrogameplay.selection-byte-tables.
Prossimo modello/oracoloWine poi managed/runtime/CLI/ordinario; Linux-USA/
Windows nativo/nuovo32:9/indiretti/fullTODO pendenti.
[Contratto](../rom_data_gameplay.md#european-selection-byte-table-source-contract).


**ROMDATA-02-SELECTION-BYTE-TABLES completato modello/stadi EUR/Wine
(2026-10-29).** Modello strict4+30u8/34B/due span, shape distinta da ROMesatta
+seiwindow; gap2B escluso/valori ripetuti distinti. Un buildcore Windows senza
warning b40503b8..576file245input/574preimmagini; un filtro Wine18guard/45base+
24sintetici esegue D323Ccount/D327Esomma-zero/D32B4scelta con vero377BC.
Python/modello/risultati/ordine querylive e LDRB esatti; wrap32/rango firmato
anche estremi, nessuno shortcutpeso0 o nuovocap. Tutti34Bbase/32sintetici osservati,
ricostruzione34 separata, tutteRAM/SP-R8..R12/ROMfinale esatti;30480istruzioni
solo copertura.123guard576hash/3correnti/574preimmagini/96sourceguard PASS,
helper0. Schema1/e86-d85/79file invariati; runtime gestito/bus/nativeclock/
LoadUndo/DMA/CLI/ordinario prossimi, helpernumerici/RNG/fullABI/Linux/USA/
Windowsnativo/new32:9/TODOglobale pendenti. Nessuna fixtureordinaria o prova
chiusa ripetuta/metriche.
[Modello e prova](../rom_data_gameplay.md#typed-selection-byte-tables-and-original-stage-proof).


**ROMDATA-03-04-SELECTION-BYTE-TABLES-RUNTIME completato EUR/Wine
(2026-10-29).** e87-d86/index78/selection-byte-tables.json/80file79risorse,
due span4/30B/nodeps, tutti78record/file precedenti identici/schema1-USAe8
invariati. Ownerimmutabile page71C compone prior70B e rawgap/eventroot;
6contatori/4+30posizioni distinte, nessun capallocazione o cambioABI. Un build
Windows senza warning e5b45668..578file246core/576model, un filtro Wine
pubblicazione/interruzione/14attach2fullset/20native fullstate-clock/all34bus/
LoadUndo/DMA3 PASS.12load+8stadi/vero377BC, raw34+prior4 alterati solo dopoLoad;
76nuoviBserviti+5prior tutti attribuiti a callback, fallbacksource0,91entry119
dispatch solo copertura. Width/mirror/confini/prior/raw, tutte4+30posizioni e
DMA-CpuSet20halfword/FastSet16word exactstate-clock.243guard578hash576model/
7correnti/79prior exact; helper ritira solo interrupted riuscito. CLIreale/
ordinario prossimi; numerichelpers/RNG/fullABI/gap-eventextent/Linux/USA/Windows
nativo/new32:9/TODOglobale pendenti, nessuna vecchiaprova o fixture ripetuta/
metriche. [Consumo](../rom_data_gameplay.md#managed-selection-byte-tables-and-typed-runtime-reads).


**ROMDATA-04-SELECTION-BYTE-TABLES-CLI completato EUR/Wine registrazione-avvio
(2026-10-29).** Un buildproduzione senza warning22af4132.. stesse578sorgenti
runtime/nooverlay/canonico hashmtime ripristinato; una orchestrazioneWine tre
comandi create-reuse-neutral1 exit0.80file datasetidentici/79IDhash/82percorsi
hashmtime invariati/cache28859immutata. e87-d86/index78/two spans34B/no deps
verificati;24755→24756 tutte6usage0, nessunaaccettazioneordinaria.192guard
578CLI+578runtime/ROM-nativecache-TinyCC-seedoriginale-canonico exact/helper0.
Nessunanavigazione/nuovocheckpoint/vecchiaprova/metriche. Positivoordinario e
fullpair prossimi; Linux-USA-nativeWindows/new32:9/fullABI/RNG/indirect/gap-event/
TODOglobale pendenti. [CLI](../rom_data_gameplay.md#actual-selection-byte-cli-registration).


**ROMDATA-04-SELECTION-BYTE-TABLES-ORDINARY completato un caso EUR/Wine
(2026-10-29).** Otto nuove window/354opcodepin direttiD3724→D3228/guardarglow8
nonzero; wrapperD6970passes1, altri2callers0skip, indirectignoti. Quattroseed
readonly guard6/count1-2; sceltonormaleGiza19049/cachedUI. Un579source8f3bee97
build observer6stats+5lineguard/no core-runtimechanges/inversehub exact.
UNA captureA12/129f19049→19178/first97-19147/rolling96→suffix33-19145→19178.
SAME22afCLI/e87ZloxMx positivo33 eSAME8f3bee97 fullpair33 tuttecomplete states-
clock/classic-wide/OAM/5audio PASS; counts6exact1bus1SERVEDthresholdB1position/
weights0, non tutte4/30posizioni oevent/missionoutcome. DiscoverySTOPafterone.
Fullwide284x160bitrep authenticated/classicmapcalendarPNG inspected.140guards
579hash578CLI/3correnti/82managed/nativecache-ROM-seed-canonical exact/helper0.
Dispatch203258/519441 eaudio[72418,72418,8712,8712,72342] solo copertura; stessa
cache inentrambi arms/non globalinterpreter-nativeparity. Nessuna prova chiusa
o fixture ricreata/metriche. Catena source-model-runtime-CLI/unordinario chiusa;
event12BrootC760/alias71D1EC/next71D360 richiede contrattoproprio. Linux-USA-
Windowsnativo/new32:9/fullABI/RNG/indirect/gap-event/TODOglobale pendenti.
[Ordinario](../rom_data_gameplay.md#ordinary-selection-threshold-travel-equivalence).


**ROMDATA-01-EVENT-RECORDS12 completato contratto sorgente EUR (2026-10-29).**
71C760..71D360/256x12/3072B/onespan SHAe7454120.. disgiunto79owner50494spans;
backscan255..1/stride12+liveu8zero+nextrootindipendente27stride provanoestensione,
nonfitonly/globalcap. Layoutu8header00-key01/u16primary02-alternate04/u8param06-
byte07/u16flags08/opaque0A2B, tutti3072roundtrip/noenums-FKcaps-zeropadding.
71D1EC=base+225x12/31physicalpositions vistaINTERNA/sameowner, helper9D04low8arg
noncappato31/puòuscireowner; sceltaordinal1..30→righe226..255. Callback127388
keyu16completo/vsbyte01/highestmatch255..1/zeroexcluded/8keygroups1recordgroup.
Alt04nonzero→aliasprimary02+originalalt04, zero→directprimary/alt0; key4Alive
special/param06next27root/liveeffects originali.15window10new5cached/67pin/
112guard/80acceptedfiles PASS/helper0; census36base2alias25next e nonindirectall.
Solo statico/no build-Wine-game-fixture-navigation-oldproofreplay;
schema1/e87-d86 invariati. Modello/originalprefixWine prossimi, poi managed/
runtime/CLI/ordinario; fullABI/liveeffects/indirect/gap/next27extent/Linux-USA-
Windowsnativo/new32:9/TODOglobale pendenti/no metrics.
[Contratto](../rom_data_gameplay.md#european-event-record-source-contract).


**ROMDATA-02-EVENT-RECORDS12 completato modello/prefissi EUR/Wine (2026-10-30).**
Strict256x12/3072B/onespan/rawflags/opaque2B, shapeorder distinta da ROMesatta
+15window. PhysicalNonefuoriowner vs u16encodedview/low8aliasuncapped, pair
direct/alias-altoriginale e scriptfull16highest255..1/row0excluded. Un buildcore
senza warning a4b88a69..581file248core/579preimages/inversehubexact, un filtro
Wine20guards51base37synthetic prefissi9D04pair/127388scan/sevenunsignedloads/
u16view PASS; Python-modello-risultati/ordineLDR esatti.302baseB268syntheticB
osservati vs3072roundtrip separato; RAMesatta con indexstore/pushattesi, SP-
R8..R12/ROMfinale verificati,9959istruzioni solo copertura.133guard581hash/
3correnti/579preimages112source exact/helper0. Schema1/e87-d86/80file invariati;
managed/nativeclock/lifecycle/CLI/ordinario prossimi. FullABI/liveeffects/
indirect/gap/next27extent/Linux-USA-Windowsnativo/new32:9/TODOglobale pendenti.
Nessuna vecchiaprova/fixtureordinaria/navigazione ripetuta/metriche.
[Modello](../rom_data_gameplay.md#typed-event-records-and-original-prefix-proof).


**ROMDATA-03-04-EVENT-RECORDS12-RUNTIME completato EUR/Wine (2026-10-30).**
e88-d87/index79/event-records12.json/81file80risorse/79prior exact,3072B/onespan/
nodeps/aliasrow225 SAMEowner; pages71c-71d/8classes11stats/4maskwords256positions,
priorselection2B+rawgap2/nextroot composti, nessunalloccap/guestABIchange.
Un buildwarningfree9225a3bc..583source249core581model, un Winefilter
pubblicazione-interruzione/14attach2fullset/22nativefullstate-clock/all3072bus
8class256record256borders/LoadUndo/DMA3 PASS.14load+8pair-scan-u16view/4stop-safe
families; raw3072+prior2 alterati dopoLoad,296SERVEDnewB3prior tuttiattribuiti,
2newBinterp_onefallback (non296directnative o distinctunion)/51entry30dispatch
solo copertura. DMA-CpuSet1538halfword/FastSet776word exactstate-clock.253guard
583hash581model/sevencurrent80prior exact; helperonlysuccessfulinterrupted.
ActualCLI/ordinary prossimi, fullABI-liveeffects-indirect-gap-next27/Linux-USA-
Windowsnativo/new32:9/fullTODO pendenti; nessuna vecchiaprova/fixture/metriche.
[Consumo](../rom_data_gameplay.md#managed-event-records-and-typed-runtime-reads).


**ROMDATA-04-EVENT-RECORDS12-CLI completato EUR/Wine registrazione-avvio
(2026-10-30).** Un buildproduzione warningfreeb9a60501.. SAME583sourceruntime/
nooverlays/canonico hashmtime ripristinato; una Wineorchestrazione create-reuse-
neutral1 treexit0.81payloadidentici80IDhash83nowritepaths/cache28859exact;
e88-d87/index79/one3072span/no deps.24755→24756 all11counts0/compatibilityonly,
nonpositiveordinary.194guard583CLI+583runtime/81accepted/ROM-nativecache-TinyCC-
seed-canonico exact/helper0. Ordinarypositive/fullpair prossimi; Linux-USA-
Windowsnativo/new32:9/fullABI-liveeffects/indirect/gap-next27/fullTODO pendenti.
Nessuna source-model-native/oldproof/fixture-navigation replay o metriche.
[CLI](../rom_data_gameplay.md#actual-event-record-cli-registration).


**ROMDATA-04-EVENT-RECORDS12-ORDINARY completato un caso EUR/Wine (2026-10-30).**
SeednormaleGizaWhiteFlowers19409 originale/cachedUI+producerwitness riusati,
nonold700f routeproof. Un584source0b6581cb build observer11stats+5lineguard/
inversehub-validator exact/core-runtime unchanged. UNA A12capture34f19409→19443/
firstrel2-19412/rolling0 suffix34; discoverySTOP. SAMEb9a605CLI/e88generation
3iczRx actualchild0/34f, collectorTypeErrorperOLDusageID correttoOFFLINElog
registration only/noCLIreplay. SAME0b6581cb fullpair34 tuttecomplete states-clock/
classic-wide/OAM/5audio PASS.11counts exact4bus6SERVEDB/key2-primary2-alternate2/
onephysicalrecord/other5classes0; nondistinctunion/all8-all256/aliasbranch/outcome.
Fullwide284x160bitrep authenticated/classicGizamapmissionentry inspected.126guard
584hash583CLI/3correnti/83nowritepaths/80IDhash/cache28859-ROM-seed-canonical
exact/helper0; dispatch128240/218562/audio[74612,74612,8976,8976,74488] onlycoverage.
Catena eventsource-model-runtime-CLI/unordinario chiusa; next27root71D360
contractextent/fields pending, Linux-USA-Windowsnativo/new32:9/fullABI-liveeffects/
indirect/gap/fullTODO aperti/no metrics.
[Ordinario](../rom_data_gameplay.md#ordinary-event-record-mission-entry-equivalence).


**ROMDATA-01-PARAMETER-RECORDS27 completato per i frammenti EUR osservati
(2026-10-30); estensione totale ancora aperta.** I256record evento dimostrano
59indici0..50 e115..122, passo27/root71D360: due intervalli71D360..71D8C1 e
71DF81..71E059/1593B, roundtrip SHA40d4e406..,256link e nessuna sovrapposizione
con80owner/50495span e88-d87.27finestre23nuove4riusate/70pin/125guardie,
tutti25literal root coperti nel censimento cached. Campi packed01..06 condivisi,
u16LE07-0A-12, i8 espliciti0C-15, byte09 eflags0D, operandi separati0E0F-1011,
opaque14+16..1A preservati. Nessun enum/puntatore/padding dedotto dai valori.
I lettori live+A8 usano u8 senza limite127/128/59; nextroot71E0E0 passo38 non
prova128record per sola distanza. Ipotesi128 respinta; buchi/indici ulteriori
raw e non posseduti. Statico soltanto, nessun build/Wine/input/fixture/vecchia
prova ripetuta né metrica. Modello/prova Wine, runtime, CLI e ordinario da fare;
estensione globale/fullABI/liveeffects/indirect/Linux-USA-Windowsnativo/new32:9/
fullTODO aperti. Catena event12 chiusa.
[Contratto](../rom_data_gameplay.md#observed-european-parameter-records-with-27-byte-stride).


**ROMDATA-02-PARAMETER-RECORDS27 completato per i59record EUR osservati
(2026-10-30).** Modello strict con indici fisici0..50/115..122, campi u8/packed,
u16LE, i8/opaque, due span1593B; validazione modifiche distinta da sourceauth.
record/byte_at non rispondono nei buchi o per indici alti; encoded_view conserva
low8 originale senza limite59/128. Un586source/251core build49c72f02 warningfree,
un filtro Wine25guardie73base48sintetici:12classi di prefissi originali,
risultati e ordine letture esatti, signed127/-128/encodinglow16 inclusi.
TuttaRAM/SP-R8..R12/ROM finale esatti;95Bbase20Bsintetici osservati distinti
contro1593B roundtrip.739istruzioni solo coverage.143guardie/584preimage/
125sourceguard/inversehub preservati/helper0. Nessuna vecchia prova o fixture
ripetuta, né metriche. Runtime gestito e89-d88/CLI/ordinario da fare;
estensione globale/fullABI/helper/liveeffects/indirect/Linux-USA-Windowsnativo/
new32:9/fullTODO aperti. e88-d87 invariato.
[Modello](../rom_data_gameplay.md#sparse-parameter-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-PARAMETER-RECORDS27-RUNTIME completato EUR/Wine per i59record
osservati (2026-10-30).** e89-d88/index80/parameter-records27.json/82file81risorse,
80precedenti esatti/nodeps/due span1593B totali; owner sparso71D..71E compone
precedente evento e buchi raw,12classi15stats/quattro mask fisiche. Nessun cap
nuovo/ABI/save/nativecache change. Primo588build51e19ef0 passa pubblicazione,
interruzione,14attach2fullset/17native fullstate-clock; testoracle attende2byte
precedenti invece dei4 allineati serviti, typed/native/API concordano4.
Corretto soltanto testleaf: secondo588build278d6d23 warningfree/unWine solo
16native rimanenti+bus/lifecycle/DMA3 PASS, stessi82file13oggetti hashmtimeesatti;
nessuna ripetizione pubblicazione/attach/primi17casi. Totale33fullstate-clock/
20load13prefix/13famiglie,37SERVEDB nuovi5evento; nonunione distinta o tutti
native diretti, fallback0solo16rimanenti/precedenti17 nonrendicontati. Tutti1593B,
12classi59record/59bordi/rawgap-nextroot/LoadUndo, DMA-CpuSet1664halfword/
FastSet832word esatti.291guardie588accepted+588diagnostic/252core/586model/
7correnti81prior preservati; helper solo interrupted. Labeltask collector
corretta offline senza build/test replay. accepted-selection.json autorevole,
selection.json diagnostico. CLIeffettiva/ordinario/estensione globale/fullABI/
helper-liveeffects/indirect/Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
[Runtime](../rom_data_gameplay.md#managed-sparse-parameter-records-and-typed-runtime-reads).


**ROMDATA-04-PARAMETER-RECORDS27-CLI completato creazione/riuso/avvio EUR/Wine
(2026-10-30).** Un588source productionbd99ac5b warningfree, SAMEruntime corretto/
nooverlays/canonicalhashmtime finalmente ripristinati. Una orchestrazione Wine:
prepare crea e89-d88, prepare riusa, runc checkpointnormale24755copiato neutro1→
24756; treexit0/82payloadruntimeexact/81IDhash/84managedpathhashmtime invariati/
28859nativecache. Index80/two1593Btotalspans/nodeps corretti. Tutti15conteggi0,
solo compatibilità/registrazione, non uso ordinario positivo o equivalenza.
197guardie588CLI+588runtime/ROM-seed-cache-TinyCC-canonical esatti/helper0.
Nessuna nuova navigazione/checkpoint né vecchie prove o metriche ripetute.
Ordinario positivo/fullpair, estensione globale/fullABI/helper-liveeffects/
indirect/Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
[CLI](../rom_data_gameplay.md#actual-sparse-parameter-record-cli-registration).


**ROMDATA-04-PARAMETER-RECORDS27-ORDINARY parziale (2026-10-30), positivo ancora
da ottenere.** Osservatore passivo15stats+modulo/guardia opzionale5righe;
589source8f0c02b7 warningfree/core-runtime588CLI invariati/inversepreimage esatti.
Due catture Wine normali su copie: R12 GizaInfo15050→15230/180f pagina1/3
invariata; A12a1/161 dialogoWhiteFlowers21889→22129/240f. Tutti15conteggi0:
bounds NEGATIVI chiusi, nessun suffisso positivo/CLIpositiva/fullpair eseguiti.
9finestre statiche/7gatesnapshot readonly; join accettato directoryscript/
archivio/eventi identifica opcode80mode2 inprogrammi53/65/74/93/95, chiavi56/
68/77/96/98, eventi216..220/param116..120. WhiteFlowers58 param0/program163;
stop ulteriori tentativi ciechi nella pagina o intro, indiretti/mode0override0
non esclusi.130guardie589+588hash/3correnti84managed/originali preservati,
diagnostica trattenuta/nessun cleanupcompleto. Riutilizzare SAMEobserver ebd99
CLI/e89dataset quando è nota una scena pertinente; positivo/full-equivalenza/
extentglobale/Linux-USA-Windowsnativo/new32:9/fullTODO aperti/no metriche.
Aggiornamento della stessa unità: undici catture normali totali, tutte15stats0,
nessun rebuild. Nuovi contesti SprohmInfo180/B32/cursoreNubswood/travel360 e
CyrilPub240+120; nessuna selezione missione dedotta dal nome o RNGresampling.
Sei gatesnapshot autenticati non provano liveness. Una sola ispezione Wine
neutra1/CLIgiàcompilata/layoutoff exit0 non salva/adotta stato; proiezioni
readonly framebuffer/UI76800pixel mostrano solo Gil, nessuna scelta leggibile.
233guardie589+588hash/3correnti84managed esatti, prime ricevute trattenute.
Navigazione cieca fermata; ordinario positivo ancora aperto, contratti sorgente
indipendenti possono proseguire senza ripetere queste catture.
[Stato e prove](../rom_data_gameplay.md#ordinary-sparse-parameter-record-reachability-remains-partial).


**ROMDATA-01-WORLD-RECORDS38 completato per il catalogo EUR osservato
(2026-10-30).** Root71E0E0/31x38/1178B fino71E57A,
SHAcd2b33e95a5b46a53703f8a32f89a6f1c0eea4d1b562dfd86ee7c4836084d483.
Tutte512missioni proiettano esattamente indici0..30; interiorroot71E106 e
ciclo6..29 usano SAMEowner/record7..30, next71E57C legge pointeru32/gap2B
non posseduto. Nessun count-by-distance o cap31: D347C low8 eD4C4C R2u32
completo conservano gli indirizzi originali. Byte01/u16LE02conbyte02separato/
u16LE04conlow10copy/31threshold07+countclamped0..30/opaque00-06 preservati.
12finestre10nuove2cached/58pin107guardie,1178roundtrip512link/nooverlap81owner
50497span/e89-d88/82file esatti. Statico soltanto/no build-Wine-input-fixture/
vecchie prove o metriche. Modello/oracoloWine/runtime/CLI/ordinario da fare;
helper-RNG/fullABI/liveeffects/indirect/opaque/Linux-USA-Windowsnativo/new32:9/
fullTODO aperti. Sparse27ordinario resta parziale, navigazione non riaperta.
[Contratto](../rom_data_gameplay.md#observed-european-world-records-with-38-byte-stride).


**ROMDATA-02-WORLD-RECORDS38 completato EUR/Wine (2026-10-30).** Modello
strict31record/1178B/onespan, opaque e tutti bit preservati; shape modificabile
distinta da exactEURauth/12windowhash. Physicalrecord/byte_at non possiede gap2B
o indici31+, encodedlow8/fullu32/interiorrow1 originali senza cap31. Proiezioni
byte01divisor+posthelperwrap/zero1, byte02/u16LE02, fullu16/low10value04,
livecountclamp0..30/signedthresholdcomparison con raw255. Un590source/254core
builded4d65c8 warningfree, un filtroWine19guardie41base19sintetici originali
PASS;23Bbase6Bsintetici osservati distinti vs1178roundtrip/353istruzioni solo
coverage. Ordine letture/risultati/tuttaRAM+destinazionipreviste/SP-R8..R12/ROM
finale esatti.26guardieunit107source/588preimage/3correnti/inversehub esatti,
helper0. No externalhelpers-RNG/fullABI/nativeclock/ordinario claim; nessuna
vecchia prova/navigazione/fixture ripetuta o metrica. Schema1/e89-d88/82file
invariati. Runtime gestito e90-d89/CLI/ordinario e Linux-USA-Windowsnativo/
new32:9/fullTODO da fare.
[Modello](../rom_data_gameplay.md#world-record-model-and-original-prefix-wine-proof).


**ROMDATA-03-04-WORLD-RECORDS38-RUNTIME completato EUR/Wine (2026-10-30).**
e90-d89/index81/world-records38.json/83file82risorse/prior81esatti/nodeps/
onespan1178B. Owner immutabile71E compone sparse27/gapraw/nextroot,5classi8stats/
one31mask senza nuovo cap o ABI. Un592build271dceba warningfree; collector
guard-label corretto offline su SAMEexe, nessun rebuild. Un filtroWine passa
pubblicazione/interruzione/14attach2fullset/27native fullstate-clock (17load,
10prefix/10famiglie stop-safe),23SERVEDB nuovi5prior/fallback[0, 0],
non unione distinta/tutti native diretti. Tutti1178B/5classi31record/31bordi/
mirrors-widths/prior+rawgap-nextroot/LoadslotUndo esatti; DMA-CpuSet704halfword/
FastSet352word fullstate-clock. 269guardie592frozen/255core/590model/
7correnti82prior preservati, observer27corrente escluso SOLO dal frozenhub.
Helper solo copia interrupted. CLIeffettiva/ordinario/fullABI/helper-RNG/
liveeffects/opaque-indirect/Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
No vecchie prove/navigazione/fixture o metriche ripetute.
[Runtime](../rom_data_gameplay.md#managed-world-records-and-typed-runtime-reads).


**ROMDATA-04-WORLD-RECORDS38-CLI completato creazione/riuso/avvio EUR/Wine
(2026-10-30).** Un592source production6d95f705 warningfree, SAMEruntime/nooverlays/
canonicalhashmtime ripristinati. Una orchestrazioneWine/treexit0: preparecrea
e90-d89, prepareriusa, runccheckpointnormale24755copiato neutro1→24756.
83file runtimeexact/82IDhash/85managedpathhashmtime invariati/cache28859esatta;
index81/nodeps/onespan1178B. Otto conteggi0, solo registrazione/compatibilità,
non positivo ordinario/equivalenza. 201guardie592CLI+592runtime/ROM-seed-cache-
TinyCC-canonical esatti/helper0. Nessuna nuova navigazione/fixture/vecchia prova
o metrica ripetuta. Ordinario positivo/fullpair/fullABI/helper-RNG/liveeffects/
opaque-indirect/Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
[CLI](../rom_data_gameplay.md#actual-world-record-cli-registration).


**ROMDATA-04-WORLD-RECORDS38-ORDINARY completato per un caso EUR/Wine
(2026-10-30).** Un593sourceee939288 warningfree/passive8stats+5lineguard,
core-runtime592CLI invariati/inversehubs esatti/current27observer preservato.
PrimoA12Nubswood360f3448→3808 NEGATIVO chiuso; readonlyCCF00/path0..6/counter32→30
spiega ramo non pronto. QuattrobankEUR autenticati: IN08physical13counter1,
normalDown48/96f3020→3116 terminaactualnode18; A12unica cattura positiva189f
3116→3305, firstsource157/3274, rolling45suffix3260→3305/discoverySTOP.
SAME6d95CLI una45/SAME593test unfullpair45: ogni stato-clock/classic-wide/OAM/
5audio esatto,8stats identiche2bus2SERVEDB=1threshold+1byte01/onephysicalrecord,
altre3classi0/nonall5-31. 82IDhash85managed/cache28859nowrite, proiezioneframebuffer
autenticata/worldtravelMadmoonDay4, nessuna missione/arrivo dedotti.165guardie/
593+592source/3correnti/inverse/originali esatti/helper0. Normaliinputcopie soltanto,
nessun reset-RNG-RAM-ID/route recreation, nessuna vecchia prova o metrica.
Altri ordinari/fullABI/helper-RNG/liveeffects/opaque-indirect/Linux-USA-Windows
nativo/new32:9/fullTODO aperti; sparse27ordinario resta parziale.
[Caso e limiti](../rom_data_gameplay.md#ordinary-world-record-counter-expiration-equivalence).


**ROMDATA-01-INDEXED-GRAPHICS-ARCHIVES completato per il contratto sorgente EUR
(2026-10-30).** Quattro gruppi pointer71E57C/71E61C/71E630/71E658, cinque
slot lingua ciascuno,20archivi649programmi;24span disgiunti74907B sorgente
(74827archivi+80pointer),278336B output/15631B prefissi dizionario. Codec
05228/0537C con indice-offset BE e dizionario esterno fisso, distinto054A0;
opaque4137 non è discriminatore, rawu8 lingua senza fallback e low16start-count
originali. Tutti byte sorgente/output ricostruiti esatti,22finestre20nuove2cached/
101pin/10rootliteral/12BLfusi/121guardie/nooverlap82owner50498span/83file esatti.
Altri16gruppi vicini/code-dictionary tails/context3710D4/globalextent non
classificati. Nessun build-Wine-game-fixture/vecchia prova/metrica ripetuta;
schema1/e90-d89 invariati. Modello/oracoloWine/runtime/CLI/ordinario/fullABI/
indirect/Linux-USA-Windowsnativo/new32:9/fullTODO aperti; sparse27ordinario parziale.
[Contratto e limiti](../rom_data_indexed_graphics.md).


**ROMDATA-02-INDEXED-GRAPHICS-ARCHIVES completato EUR/Wine (2026-10-30).**
Modello strict20archivi/fourfive-slotgroups/649programmi: opaqueheader, indice
eoffset BE/pacchetti literal-dictionary/prefissi dizionario/LEwords esatti,
shape modificabile distinta da exactEURauth. Hostmode0width5 soltanto, nessun
nuovo guestcap; rawlanguage encodedview resta originale per5/255, low16start-
count/zero-oversize-clamp/invalidstart-before-init conservati. Un595source/257core
buildd78a74d9 warningfree da593ordinary+2leaf/onepubmodule; unWinefilter37guardie
20wholearchivecalls+9boundary+9synthetic+28rawselector prefixes PASS. Tutti74907B/
24span/278336decodedB ricostruiti; output/status/globals/SP-R4..R11 e RAM fuori
68B stackoriginale/VRAM-OAM-palette/finalROM esatti.5993129istruzioni solo coverage.
26guardieunit121source/593preimage/3correnti/inversehub esatti. No closedproof/
checkpoint/navigation/performance replay; schema1/e90-d89 invariati. Runtime
gestito/nativeclock/CLI/ordinario/mode3-widerwidth/fullABI-liveuploads/indirect/
Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
[Modello e prova](../rom_data_indexed_graphics.md#typed-model-and-original-wrapper-proof).


**ROMDATA-03-04-INDEXED-GRAPHICS-RUNTIME completato EUR/Wine (2026-10-30).**
e91-d90/index82/graphics.indexed-archives/indexed-graphics.json/84file83risorse/
all82priorrecords-files esatti/nodeps/24span74907B; owner immutabile ricostruito
da header-indice-pacchetti-dizionario/7classi12stats/20bank649program4group masks.
Nessun decoder-upload shortcut/nuovo ABI o guestcap. Primo597build fallisce
solo importBus test-only, diagnostica conservata; corretto597/258core0732b3b9
warningfree/production invariata. UnWinefilter PASS pubblicazione-interruzione/
14attach2fullset/35complete-state-clock (28load+oneoriginal1programwrapper+6raw
selectors)/3famiglie stop-safe;272SERVEDnuoviB4priorworld/fallback[1,0], non
unione distinta/tutti native diretti;232istruzioni59dispatch solo coverage.
Tutti74907busB/7classi20archivi649program4gruppi/all24bordi-mirrors-widths/prior
world+gapraw/LoadslotUndo/DMA-CpuSet-FastSet mixed64B fullstate-clock esatti.
282guardie597accepted597diagnostic/595model/7correnti83prior preservati.
Accepted-selection.json autorevole, selection.json primoFAILdiagnostico.
CLIeffettiva/ordinario/fullABI-liveuploads/mode3-widerwidth/globalextent/indirect/
Linux-USA-Windowsnativo/new32:9/fullTODO aperti. No vecchia prova/fixture/
navigazione/metrica ripetuta.
[Runtime e prove](../rom_data_indexed_graphics.md#managed-source-views).


**ROMDATA-04-INDEXED-GRAPHICS-CLI completato creazione/riuso/avvio EUR/Wine
(2026-10-30).** Un597source productiond8767645 warningfree, SAMEruntime/nooverlays/
canonicalhashmtime ripristinati finally. Una orchestrazioneWine/treexit0:
preparecrea e91-d90, prepareriusa, runc checkpointnormale24755copiato neutro1
→24756.84file runtimeexact/83IDhash/86managedpathhashmtime invariati/cache28859
esatta; index82/nodeps/24span74907B. Tutti12conteggi0, solo registrazione/
compatibilità, nessun positivoordinario/fullpair.201guardie597CLI+597runtime/
84accepted/ROM-seed-cache-TinyCC-canonical esatti. Nessuna nuova navigazione/
fixture/vecchia prova/metrica ripetuta. Ordinario/fullABI-liveuploads/mode3-
widerwidth/globalextent/indirect/Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
[CLI e limiti](../rom_data_indexed_graphics.md#actual-cli-registration).


**ROMDATA-04-INDEXED-GRAPHICS-ORDINARY completato per un caso EUR/Wine
(2026-10-30).** Un598source6d40a6a5 warningfree/passive12stats+5lineguard,
597CLIcore-runtime invariati/inversehubs esatti/currentparameter-worldobservers
preservati.8finestre6nuove2cached/453E2→1967Ccontext3→program2MENU; unaStart12
cattura normale41f4519→4560/firstsource9guest4529/rolling41suffix/discoverySTOP.
SAMEd876CLI una41/all83IDhash86managednowrite/cache28859esatta eSAMEtest unfullpair
ogni41stati-clock/classic-wide/OAM/5audio esatti,12stats identiche274read277
SERVEDB=selector4/header6/index4/token67/literal17/dictionaryheader4/dictionary
175,onearchive-program-group/all7classespositive/nonall20-649-4.144guardie/
598frozen597CLI/3correnti/inverse/originali esatti. Readonlyfullwide284x160
proiezione/classicPNG mostraMENU Party-Clan-AreaList-System/MoonDay9, nessuna
scelta/missionoutcome. Inputnormali soltanto, nessunRAM-ID-RNG-ROMforcing o
old320Down-A/world38daily45/negative/proofreplay/metrica. Altriordinari/fullABI/
mode3-widerwidth/globalextent/indirect/Linux-USA-Windowsnativo/new32:9/fullTODO
aperti; sparse27ordinario resta parziale.
[Caso e limiti](../rom_data_indexed_graphics.md#ordinary-world-map-menu-equivalence).


**ROMDATA-01-LOCALIZED-TABLE-GRAPHICS completato contratto sorgente EUR
(2026-10-30).** Otto gruppi indipendenti71E590/5A4/5B8/5CC/644/680/6BC/6D0
rawu8lingua02001F6E×4/LE32pointer→054A0,40nuovibanchi customLZ-RLE/nonindexed.
48span53667sourceB(53507bank+160selector)/114880decodedB, tutti token/source/
outputricostruiti esatti;14finestre10nuove4cached/48pin/8rootliteral/111guardie/
nooverlap83owner50522span/all84acceptedfileesatti. Output pergroupfivebanks
1472/2688/768/4928/1920/7488/1344/2368B, nessun count-by-distance o guestcap/
fallbackskipinventato. LiveRAM/helpertransfer/laterworkrestanooriginali,
fulltransferABI nonchiuso. Known71E5E0vecchiUIbanks/tablealias separatoe
settealtrigruppi rawlooking/tails/indirect/globalextent nonclassificati.
Staticosoltanto/no build-Wine-game-fixture-navigation/vecchia prova o metrica;
schema1/e91-d90 invariati. Modello/oracoloWine/runtime/CLI/ordinario/Linux-USA-
Windowsnativo/new32:9/fullTODO aperti.
[Contratto e limiti](../rom_data_ui_graphics.md#additional-european-table-selected-source-contract).


**ROMDATA-02-LOCALIZED-TABLE-GRAPHICS completato EUR/Wine (2026-10-30).**
Nuovo coreui_graphics::table_selected riusaGraphic/customLZ:strict8tables×5
contiguousBankPointers/40orderedbanks/rawencodedbyte+LEwords/groupoutputs,
shape modificabile distinta daexactEURauth/14fingerprints. PhysicalselectNone
per5/255 maencodedviewraworiginale. Catalogseparato nonancoraabilitato nella
risorsaUIgestita:old16banks/twoinline switches/e91 invariati. Un600frozen/260core
buildcb46ccb1 warningfree da598ordinary+twoleaves/onepubmodule; unWine23guardie/
40neworiginal054A0calls56rawselectorprefixes PASS. Tutti53667sourceB/48spans/
114880decodedB/cursor-produced-returned-size/SP-R4..R11/fullEWRAM/IWRAMfuori
40Bstackoriginale/VRAM-OAM-palette/finalROM esatti.1335470istruzioni solocoverage.
26unit111sourceguards/600frozen598preimage/3correnti/inversehub esatti; noold
16UI/20indexed/native35/41ordinary/CLIproof replay/fixture-navigation/metrica.
Offlinecontrollabel23B2Ecorretto0150word/1344B, nongroupset02A0halfword,
originali/inputsinvariati/noproofreplay. Runtimegestito/nativeclock/CLI/ordinario/
fullhelperABI/livecontexts/71E5E0alias/sevenrawgroups/globalextent/indirect/
Linux-USA-Windowsnativo/new32:9/fullTODO aperti.
[Modello e prove](../rom_data_ui_graphics.md#typed-table-model-and-new-original-decoder-proof).


**ROMDATA-03-04-LOCALIZED-TABLE-GRAPHICS-RUNTIME completato EUR/Wine
(2026-10-30).** e92-d91/same84files83IDs/UIindex15 56banks10selectors106bindings,
allother82records-files/old16banks-twoinline-sharedOBJ esatti.40bank+40pointer
80newruntimebindings byteunion48canonicalspans53667B esatta;6newtablecounters
separati daold6UIstats, originaldecode-scratch-helper-uploadwork/ABI preservati.
Un601source/260core d4a73eb2 warningfree da600model+onehostleaf; ONEWine PASS
pubblicazione-changedUIinterruption/14attach2fullset/22complete-state-clock
(12load+1newbankdecoder+9rawselector)/3famiglie stop-safe.499SERVEDnewB5prior
indexed/fallback[3,0], nonunione distinta/alldirectnative;356istruzioni71dispatch
solocoverage. Tutti53667busB24328literal160selector40banks8groups/all48edges/
widths-mirrors-priorindexed-raw-oldUI/LoadslotUndo/DMA3mixedfullstate-clock
esatti.267guards601frozen600model/sevencurrent84prior exact/helperinterrupted
only. No old40model/16UI/20indexed/native35/41ordinary/CLIprepare proofs replay
néfixture-navigation/metrica. ActualCLI/ordinario/fullhelperABI-livecontexts/
71E5E0alias/sevenrawgroups/globalextent-indirect-tails/Linux-USA-Windowsnativo/
new32:9/fullTODO aperti.
[Runtime e prove](../rom_data_ui_graphics.md#managed-table-selected-graphics-and-runtime-consumption).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-CLI completato creazione/riuso/avvio
EUR/Wine (2026-10-30).** Un601runtimeidentical production31c0f298 warningfree/
nooverlays/canonicalhashmtimefinallyexact; ONEWine3commands child0 crea-riusa-
neutro1normal24755copy→24756.84runtimefileexact83IDhash86managednowrite/cache
28859exact, e92-d91/UIindex15 56banks10selectors106bindings/same84files83IDs.
Seinewtablecounts0/registrazione-compatibilitàonly/no positiveordinary-fullpair.
202guards601CLI+601runtime/84accepted/ROM-seed-cache-TinyCC-canonical esatti.
No old40model/16UI/indexed20/native22-35/41ordinary/CLIprepare proofreplay/
newcheckpointderivation-navigation/metrica. Positivoordinario/fullhelperABI/
livecontexts/globalextent-indirect/71E5E0alias/sevenrawgroups/tails/Linux-USA-
Windowsnativo/new32:9/fullTODO aperti.
[CLI e limiti](../rom_data_ui_graphics.md#actual-table-extension-cli-registration).


**ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY parziale; dueboundnegativi
chiusi EUR/Wine (2026-10-30).** Cached/rawsource244D6→2C8C4/root71E6D0 e
oldnormalFightpredictionimage selezionano64f23851→23915; distinct71E590
COMBOMENUasset/30608constructor+30FDAredraw eActionseed23611 dueDown+A
selezionano160f→23771. SAME31c0CLI entrambi child0/all6newcounts0/83IDhash
86managednowrite/cache28859exact. Staticinitializerlinknonprova nuovoload;
PPMometteHUD/menu separati, Combo raggiunto nonasserito. Nessunobserverbuild/
guard/fullpair/endpointstate/longerretry/RNGresampling/oldrouterecreation.
Diagnostica trattenuta/discoverystopped; positivoordinario/full-equivalenza
aperti inattesacontesto costruttoreconcreto. Source-model-runtime-CLIregistro
restanochiusi; Linux-USA-Windowsnativo/new32:9/fullTODO aperti/nometriche.
[Stato e limiti](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).


**ROMDATA-01-LOCALIZED-RAW-TABLE-SOURCES completato per le viste fisse EUR
(2026-10-30).** Sette radici indipendenti71E5F4/608/66C/694/6A8/6E4/6F8,
cinque slot fisici ciascuna e selettore lingua u8 non limitato:35viste/42span/
38220B(38080payload+140selector), SHAe11ef4d5f16feea8ef90e7fce58056e19d689004b7ca973270285e3cd31b8f80.
Sei prefissi224/160/768/1600/2048/2432B e solo+0200..0380/384B del settimo;
lunghezze da copie originali/rami alternativi, non distanza/header. FusedBL
12EE24→145230/CpuSet halfword e6F8→000AD0/control80000040 halfword verificati,
prima ipotesi corretta prima di produrre output.32finestre23nuove9cached/
193pin/13rootliteral/13copie fisse/130guardie/nooverlap83owner50602binding/
all84accepted file esatti. Quattro viste dinamiche6F8/u8task+2−4/u16offset<<5
non stabiliscono un cap: metadata/prefisso inferiore/tail e dominio completi
restano aperti. Solo analisi statica, schema1/e92-d91/runtime invariati;
modello/oracoloWine/runtime/CLI/ordinario/fullABI/alias71E5E0/indirect/fullTODO
aperti; nuoviLinux-USA-Windowsnativo-32:9 da fare, nessuna metrica/build/game/
fixture/navigation/vecchia prova ripetuta.
[Contratto e limiti](../rom_data_ui_raw_transfers.md#declared-source-contract).


**ROMDATA-02-LOCALIZED-RAW-TRANSFERS completato EUR/Wine (2026-10-30).**
Nuovo coreui_raw_transfers/RawTransfers+BankView: sette tabelle fisiche/35viste
ordinate,42span38220B/LEwords e settimo gruppo sparso+0200..0380 soltanto.
Forma modificabile distinta da autenticazione EUR/32fingerprint; indirizzi
lingua u8 fuori0..4 raw, nessun fallback/cap. Build603source/262core35bdc5fd
senza warning; tre build e due tentativi Wine, un filtro finale PASS21guardie/
91prefissi originali/60copie helper isolate/25copie esecutore coda DMA originale.
Tutti byte, EWRAM/VRAM/OAM/palette, IWRAM fuori stack originale36B quando serve,
SP/callee/finalROM esatti. Prime correzioni solo test: stack20→36B primaWine,
bordo tra tabelle adiacenti atteso raw erroneamente; diagnostica e preimage
conservati, modello/fonte invariati.44guardie unità130fonte/601baseline603frozen/
3correnti/inversehub esatti;6635istruzioni solo coverage. Schemae92-d91 e
84accepted file invariati; runtime/nativeclock/CLI/ordinario/fullcallerABI/
dominio6F8-lowerprefix-metadata-tail/alias71E5E0/fullTODO aperti. NuoviLinux-
USA-Windowsnativo-32:9 da fare; nessuna vecchia prova/game/fixture/checkpoint/
navigation/metrica ripetuta.
[Modello e prove](../rom_data_ui_raw_transfers.md#typed-model).


**ROMDATA-03-04-LOCALIZED-RAW-TRANSFERS-RUNTIME completato EUR/Wine
(2026-10-30).** e93-d92/index83 graphics.ui-raw-transfers/85file84ID, tutti83
record/payload precedenti esatti.42span38220B da parole raw e puntatori tipizzati,
sei contatori separati; selettori/copie/coda DMA/ABI-clock originali preservati.
Due build605source/263core, accettata4b219adc senza warning; un filtroWine PASS
pubblicazione-interruzione/14attach2fullset/23stati-clock completi nativi
(12load+1CpuSet+1esecutorecoda+9prefissilingua)/4famiglie stop-safe.939SERVED
nuoviB3precedenti/fallback[768,0], non union distinta né tutti diretti nativi;
115istruzioni25dispatch solo coverage. Tutti38220B/35viste7gruppi/42bordi/
width-mirror/adiacenze tra nuovi e vecchi/buchi raw/LoadslotUndo/quattro trasferimenti
misti64B DMA16-DMA32-CpuSet-FastSet passano.309guardie605frozen603model/
7correnti84vecchi/inversehub esatti. PrimaWine solo test corretto:441608 è
inizio archivio indexed precedente, non buco raw; codice prodotto invariato tra
build e diagnostica prima950bea97 trattenuta. Runnerlabel e92/DMA3 obsolete solo
metadata: JSON prova[1,93,92]/4trasferimenti, nessun replay. CLI reale/ordinario/
fullcallerABI/dominio6F8-metadata-prefix-tail/alias71E5E0/indirect/fullTODO aperti;
nuoviLinux-USA-Windowsnativo-32:9 da fare. Nessuna vecchia prova/source-model/
game-fixture-checkpoint-navigation/metrica ripetuta.
[Runtime e prove](../rom_data_ui_raw_transfers.md#managed-raw-transfer-resource).


**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-CLI completato creazione/riuso/avvio
EUR/Wine (2026-10-30).** Una build produzione605runtimeidentical ffda6c59 senza
warning/nooverlays/canonicalhashmtimefinallyexact; una orchestrazioneWine tre
comandi child0 crea-riusa-neutro1/normal24755copy→24756.85file runtime-identici/
84IDhash87managednowrite/cache28859esatta. e93-d92/index83/35viste7tabelle/
42binding38220B. Sei contatori nuovi0: solo registrazione/compatibilità, non uso
ordinario positivo néfullpair.203guardie605CLI+605runtime/85accepted/ROM-seed-
cache-TinyCC-canonical esatti; nessun nuovo endpoint/checkpoint derivato/input
azione/navigation/vecchia prova ripetuta/metrica. Positivoordinario/fullABI/
dominio6F8-metadata-prefix-tail/alias71E5E0/indirect/globalcatalog/fullTODO aperti;
nuoviLinux-USA-Windowsnativo-32:9 da fare.
[CLI e limiti](../rom_data_ui_raw_transfers.md#actual-cli-registration).

**ROMDATA-04-LOCALIZED-RAW-TRANSFERS-ORDINARY completato per l'avvio EUR/Wine
selezionato (2026-10-31).** CLI e93 usa il normale input di avvio conservato:
1.372 letture, 5.488 byte serviti (16 selettori, 5.472 payload), quattro banchi
e quattro gruppi. Una coppia di 1.200 frame con lo stesso dataset confronta
stato completo, clock, video, OAM, dispatch e audio: tutti uguali. Entrambi i
rami usano la stessa cache nativa; 92 file protetti invariati. Nessuna nuova
build o misura di performance. Altri percorsi, dominio dinamico, ABI completa,
Linux, USA e Windows nativo restano aperti.
[Prove e limiti](../rom_data_ui_raw_transfers.md#ordinary-european-boot-equivalence).

**ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN parziale (2026-10-31).** I chiamanti
originali forniscono esplicitamente selettori 7, 8 e 9; altri due leggono un
byte vivo senza limite dimostrato. Sei righe e cinque prefissi candidati coprono
1.304 byte senza sovrapporsi alle risorse e93. Nessun limite globale, uso
ordinario o consumo runtime nuovo è dedotto dalla sola analisi statica. Modello
e integrazione successivi sono descritti sotto; dominio residuo e Linux/USA restano aperti.
[Dettagli](../rom_data_ui_raw_transfers.md#bounded-dynamic-view-producer-discovery).

**ROMDATA-02-LOCALIZED-RAW-DYNAMIC-FRAGMENTS completato per il modello
EUR/Wine dichiarato (2026-10-31).** Sette intervalli/1.304 byte ricostruiti e
verificati da ROM; controllo mirato di forma, JSON, autenticazione e confini
passato su Wine. Nessun nuovo owner runtime, limite guest o test di esecuzione
delle routine originali. La successiva integrazione è descritta sotto; dominio
globale e verifiche Linux/USA restano aperti.
[Modello e limiti](../rom_data_ui_raw_transfers.md#dynamic-fragment-source-model).

**ROMDATA-03/04-DYNAMIC-RAW-FRAGMENTS completato per pubblicazione e letture
EUR/Wine dichiarate (2026-10-31).** e94-d93 aggiunge la risorsa indipendente
index84, sette intervalli/1.304 byte, conservando le precedenti 84 risorse.
La prova Wine verifica byte, confini, composizione e un DMA con stato/clock
completi, oltre al rifiuto di dati alterati. Il primo tentativo ha identificato
una pagina mancante, corretta prima del passaggio. Restano CLI di produzione,
istruzioni native, uso ordinario, dominio globale e verifiche Linux/USA/Windows
nativo; nessuna misura prestazionale.
[Prove](../rom_data_ui_raw_transfers.md#managed-dynamic-fragment-resource).

**ROMDATA-04-DYNAMIC-RAW-FRAGMENTS-CLI completato per registrazione EUR/Wine
(2026-10-31).** La CLI di produzione crea e riusa e94-d93, carica tutte le
85 risorse con hash verificati e avanza un frame neutro senza modificare i file
gestiti. Gli 86 file della generazione coincidono col dataset accettato. Questo
non prova l'uso ordinario dinamico; esecuzione nativa, dominio globale e
Linux/USA/Windows nativo restano da verificare. Nessuna misura prestazionale.
[Prove](../rom_data_ui_raw_transfers.md#dynamic-fragment-production-cli).

**ROMDATA-04-DYNAMIC-RAW-ORIGINAL-NATIVE completato per casi isolati EUR/Wine
(2026-10-31).** La routine originale, le due code e il DMA successivo passano
30 combinazioni (selettori 7/8/9, due rami, cinque lingue). Puntatori e byte
trasferiti coincidono; stato/clock completi coincidono tra interprete con ROM,
interprete con dataset e codice tradotto con dataset, con sorgenti dinamiche
velenate nelle due copie. Uso ordinario, dominio globale, ABI completa e nuove
verifiche Linux/USA/Windows nativo restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_ui_raw_transfers.md#original-dynamic-routine-and-translated-execution).

**ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY parziale (2026-10-31).**
La sorgente identifica il gate guest `0200203d` e due callback che lo attivano
e disattivano. Quattordici checkpoint EUR non mostrano selettori pronti; una
registrazione esistente worldmap→Party→Unit di 320 frame non ha letture bus
dei sette intervalli né ingressi osservati dei chiamanti. Il campionamento
native può perdere cambiamenti interni ai blocchi. Non si deduce assenza
globale: uso ordinario positivo, confronto stato/clock, dominio globale e
Linux/USA/Windows nativo restano aperti. Nessuna misura di performance.
[Prove](../rom_data_ui_raw_transfers.md#ordinary-dynamic-reachability-investigation).

**ROMDATA-01-DYNAMIC-RAW-DISPATCH-LIFECYCLE parziale EUR/Wine (2026-10-31).**
Il dispatcher passa per callback `063878`, registrata condizionalmente dal flag
`40h` (bit 0 di `02001f78`); la selezione 2 può impostarlo. Una osservazione
Wine del percorso esistente Area List/240 frame raggiunge il confronto una volta,
ma non mostra bit attivo ai punti campionati né letture bus degli intervalli
dinamici. È un negativo circoscritto: uso ordinario positivo, confronto completo,
dominio globale e verifiche Linux/USA restano aperti; nessuna misura prestazionale.
[Prove e limiti](../rom_data_ui_raw_transfers.md#dispatcher-lifecycle-and-area-list-observation).

**ROMDATA-01-LOCALIZED-TABLE-ALIAS fonte EUR delimitata (2026-10-31).**
La tabella `71e5e0` ha 20 byte non gestiti e punta ai cinque banchi già gestiti
del selettore clan. Cinque riferimenti letterali e sei sequenze di selezione
originali verificati; indice lingua u8 vivo senza fallback dedotto. Modello,
pubblicazione, consumo runtime, uso ordinario e Linux/USA restano aperti.
[Contratto](../rom_data_ui_graphics.md#european-alias-of-the-existing-clan-banks).

**ROMDATA-03-04-LOCALIZED-TABLE-ALIAS completato per pubblicazione e prefissi
interprete EUR/Wine (2026-10-31).** e95-d94 aggiunge i 20 byte della tabella
come cinque puntatori tipizzati alla risorsa UI esistente, senza duplicare i
banchi clan; le altre 84 risorse restano identiche. Verificate letture ai confini,
42 combinazioni di prefissi originali con stato/clock completi e rifiuto
transazionale di un file alterato. CLI di produzione, esecuzione nativa, uso
ordinario positivo e Linux/USA restano da verificare. Nessuna misura prestazionale.
[Prove](../rom_data_ui_graphics.md#managed-european-clan-table-alias).

**ROMDATA-04-CLAN-TABLE-ALIAS-NATIVE EUR/Wine (2026-10-31).** Sei prefissi
originali tradotti, 24 casi con indici 0/4/5/255: stato/clock completi uguali
all'interprete ROM, 48 byte serviti dal dataset con tabella ROM alterata.
Rimangono aperti CLI, uso ordinario, ABI completa e verifiche Linux/USA.
[Prove](../rom_data_ui_graphics.md#translated-clan-alias-selector-prefixes).

**ROMDATA-04-CLAN-TABLE-ALIAS-CLI EUR/Wine (2026-10-31).** La CLI di
produzione crea e riusa e95-d94 e avvia un frame: 86 file identici al dataset
accettato, 85 identità runtime verificate e nessuna modifica ai file gestiti.
Uso ordinario positivo, confronto completo e Linux/USA restano aperti; nessuna
misura prestazionale.
[Prove](../rom_data_ui_graphics.md#clan-alias-production-cli).

**ROMDATA-05-CLAN-TABLE-ALIAS-ORDINARY negativo delimitato EUR/Wine
(2026-10-31).** Il percorso di avvio esistente di 1.200 frame non ha mostrato
letture bus dell'alias nei lettori strumentati; nessun evento perso. Non prova
assenza globale: uso ordinario positivo, confronto completo e verifiche
Linux/USA rimangono aperti. Nessuna misura prestazionale.
[Prove e limiti](../rom_data_ui_graphics.md#clan-alias-ordinary-boot-observation).

**ROMDATA-05-CLAN-TABLE-ALIAS-BANK-ENTRY EUR/Wine (2026-10-31).** Un ingresso
Bank già registrato mostra due letture tipizzate dell'alias; una è attribuita
all'istruzione originale `0805e73a`. Una coppia separata di 120 frame con lo
stesso stato e input confronta stato/clock completi, video, OAM, dispatch e audio:
tutti uguali. Altri percorsi e lingue, ABI completa e verifiche Linux/USA restano
aperti. Nessuna misura prestazionale.
[Prove](../rom_data_ui_graphics.md#ordinary-clan-alias-use-during-bank-entry).

**ROMDATA-01-E95-HISTORICAL-READ-RESIDUE delimitato (2026-10-31).** Il
confronto delle letture F09 già conservate con e95 trova 74 byte ora gestiti e
1.072 ancora fuori dagli owner; tutti questi ultimi hanno prove precedenti di
supporto al codice, riautenticate sulla ROM EUR. Non è una nuova esecuzione né
un censimento globale: altre scene e dati rimangono aperti. Linux/USA da fare;
nessuna misura prestazionale.
[Prove](../rom_data_runtime.md#historical-f09-residue-against-e95).

**ROMDATA-01-GAMEPLAY-6E6DBC-LOOKUP fonte EUR delimitata (2026-10-31).**
I lettori originali usano due campi little-endian per riga, passo quattro; i
produttori documentati sostengono una proiezione condizionale 0..73 di 296 byte.
Non è un limite globale: gli altri indici mantengono le letture originali.
Modello, pubblicazione, runtime e scena restano aperti; Linux/USA da fare.
[Contratto](../rom_data_gameplay.md#european-conditional-four-byte-lookup-projection).

**ROMDATA-02-GAMEPLAY-6E6DBC-LOOKUP modello EUR/Wine (2026-10-31).**
Modello rigoroso delle 74 righe/296 byte con autenticazione sorgente e indirizzo
originale con wrap; una prova Wine passa lettori originali, indici interni e
fuori proiezione, JSON e rifiuto di modifiche. Estrazione gestita, runtime e
uso ordinario restano aperti; Linux/USA da fare, nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#typed-conditional-four-byte-lookup-model).


**ROMDATA-03-04-GAMEPLAY-6E6DBC-LOOKUP gestito EUR/Wine (2026-10-31).**
e96-d95 pubblica la proiezione condizionale di 74 righe: 85 risorse precedenti
identiche, letture tipizzate e bordi verificati, rifiuto transazionale di dati
alterati. Tre prefissi nativi originali e sei indici coincidono con stato/clock
dell'interprete; gli indici fuori proiezione non sono limitati. CLI di produzione,
uso ordinario e verifiche Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#managed-conditional-four-byte-lookup).


**ROMDATA-04-GAMEPLAY-6E6DBC-CLI EUR/Wine (2026-10-31).** La CLI di
produzione crea e riusa e96-d95 e avanza un frame: 87 file identici al dataset
accettato, 86 identità runtime verificate e nessuna modifica ai file gestiti.
Uso ordinario e verifiche Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-lookup-production-cli).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY-ROUTE candidato EUR (2026-10-31).**
Un input Abilità già conservato e il ramo originale della prima categoria
indicano un candidato per la lettura tipizzata; stato raggiunto e lista non vuota
sono ipotesi da osservare. Un solo prefisso Wine di 540 frame è il prossimo
controllo; Linux/USA restano aperti. Nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-lookup-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY positivo EUR/Wine (2026-10-31).**
L'input ordinario Abilità già conservato produce 55 letture/byte tipizzati,
27 del primo campo e 28 del secondo, nel segmento di 540 frame. Il contatore
è aggregato; confronto completo della scena e Linux/USA restano aperti.
[Prove](../rom_data_gameplay.md#ordinary-conditional-lookup-use-in-ability-navigation).


**ROMDATA-05-GAMEPLAY-6E6DBC-ORDINARY-PAIR EUR/Wine (2026-10-31).**
Sulla stessa sequenza Abilità di 540 frame, stato/clock completi, video, OAM,
dispatch e cinque uscite audio coincidono fra ROM originale e runtime gestito;
55 byte tipizzati coprono entrambi i campi nella stessa prova. Altre scene e
verifiche Linux/USA restano aperte; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#ordinary-ability-scene-differential).


**ROMDATA-01-GAMEPLAY-6E6EE4-ROOT fonte EUR condizionale (2026-10-31).**
Tre lettori originali usano un indice u16 da un descrittore a passo otto; il
ramo tipo5 ha testimoni statici per indici1..33, pari a132 byte senza overlap
e96. Accessibilità ordinaria, riga0 e limite globale non sono dimostrati; altri
tipi possono leggere owner precedenti. Modello e runtime restano aperti;
Linux/USA da fare, nessuna misura prestazionale.
[Contratto](../rom_data_gameplay.md#european-conditional-packed-four-byte-projection).


**ROMDATA-02-GAMEPLAY-6E6EE4-PACKED modello EUR/Wine (2026-10-31).**
Modello rigoroso delle33 righe tipizzate, ricostruzione di132 byte e lettori
originali verificati per zero, estremi1/33 e alias251 verso metadata14. Non
introduce limiti guest; pubblicazione gestita, runtime e Linux/USA restano aperti.
[Prove](../rom_data_gameplay.md#typed-conditional-packed-lookup-model).


**ROMDATA-03-04-GAMEPLAY-6E6EE4-PACKED gestito EUR/Wine (2026-10-31).**
e97-d96 pubblica132 byte tipizzati: 86 risorse precedenti identiche, bordi
condivisi e alias251 ancora serviti da metadata14, rifiuto transazionale di
modifiche e due prefissi nativi equivalenti all'interprete. CLI, uso ordinario
e verifiche Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#managed-conditional-packed-lookup).


**ROMDATA-04-GAMEPLAY-6E6EE4-CLI EUR/Wine (2026-10-31).** La CLI di
produzione crea e riusa e97-d96 e avanza un frame: 88 file identici al dataset
accettato, 87 identità runtime verificate e nessuna modifica ai file gestiti.
Uso ordinario e Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-packed-lookup-production-cli).


**ROMDATA-05-GAMEPLAY-6E6EE4-ORDINARY-ROUTE candidato EUR (2026-10-31).**
Un attacco Polly→Hildegard già registrato contiene un wrapper con descrittore
tipo5/indice1 e una catena originale compatibile con il lettore. Alcuni rami
restano da osservare; un precedente sospetto di blocco bit43 era errato. Una
sola osservazione Wine limitata al segmento esistente può distinguere l'uso.
[Prove](../rom_data_gameplay.md#conditional-packed-lookup-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6E6EE4-ORDINARY negativo delimitato EUR/Wine
(2026-10-31).** Il combattimento F09 esistente di 500 frame non mostra letture
del lookup impacchettato. È un negativo della sola scena osservata, non una
prova di assenza globale; uso ordinario positivo e Linux/USA restano aperti.
Nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-packed-lookup-f09-observation).


**ROMDATA-01-GAMEPLAY-6DF4D4-DESCRIPTORS fonte EUR condizionale
(2026-10-31).** Il produttore originale di conteggi sostiene23 puntatori e684
record a otto byte nelle banche1..23. Il conteggio11 della banca22 esclude
48 byte prima del puntatore successivo; bank0 resta separata e raw. Nessun
limite globale guest; modello/runtime e Linux/USA restano aperti.
[Contratto](../rom_data_gameplay.md#european-conditional-eight-byte-descriptor-banks).


**ROMDATA-02-GAMEPLAY-6DF4D4-DESCRIPTORS modello EUR/Wine (2026-10-31).**
Modello lossless delle23 banche e684 record, con conteggi e scritture originali
verificati su Wine; banca0 e gap banca22 restano raw e senza limiti guest
inventati. Pubblicazione/runtime e verifiche Linux/USA restano aperte.
[Prove](../rom_data_gameplay.md#typed-conditional-descriptor-bank-model).


**ROMDATA-03-04-GAMEPLAY-6DF4D4-DESCRIPTORS gestito EUR/Wine
(2026-10-31).** e98-d97 pubblica23 puntatori e684 descrittori tipizzati:
87 risorse precedenti identiche, banca0 e gap banca22 restano raw, modifica
coerente rifiutata e cinque prefissi nativi equivalenti all'interprete. CLI,
uso ordinario e Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#managed-conditional-descriptor-banks).


**ROMDATA-04-GAMEPLAY-6DF4D4-CLI EUR/Wine (2026-10-31).** La CLI di
produzione crea e riusa e98-d97 e avanza un frame: 89 file identici al dataset
accettato, 88 identità runtime verificate e file gestiti invariati. Contatori
dei descrittori a zero nel frame; uso ordinario e Linux/USA restano aperti.
[Prove](../rom_data_gameplay.md#conditional-descriptor-banks-production-cli).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY-ROUTE candidato EUR (2026-10-31).**
Il renderer originale del pannello Abilità legge puntatori e descrittori della
banca dell'unità; stato e input già conservati raggiungono il pannello. L'uso
e98 non è ancora osservato. Un solo segmento Wine di540 frame lo verifica.
[Prove](../rom_data_gameplay.md#conditional-descriptor-banks-ordinary-route-candidate).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY positivo EUR/Wine (2026-10-31).**
La sequenza Abilità di540 frame legge64 byte tipizzati:52 puntatori,6 campo0,
6 campo2; gli altri campi restano non osservati. Confronto completo, altre
scene e Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#ordinary-descriptor-use-in-ability-navigation).


**ROMDATA-05-GAMEPLAY-6DF4D4-ORDINARY-PAIR EUR/Wine (2026-10-31).**
Sulla stessa sequenza Abilità di540 frame coincidono stato/clock, video, OAM,
dispatch e audio; nella stessa prova52 byte puntatore e6+6 byte dei campi0/2
sono tipizzati. Campo4/tipo/byte7 e Linux/USA restano aperti; nessuna misura
prestazionale. [Prove](../rom_data_gameplay.md#ordinary-ability-descriptor-scene-differential).


**ROMDATA-01-GAMEPLAY-6DF4D4-BANK0 fonte EUR condizionale (2026-10-31).**
Il produttore originale legge pointer0 e può enumerare record1..15 quando
unit+37 è nonzero; un inizializzatore può impostarlo per unit+7=24. Nessun
testimone ordinario nei62 snapshot esaminati; record0 e altri indici restano
raw, senza limite guest inventato. Modello/runtime e Linux/USA restano aperti.
[Contratto](../rom_data_gameplay.md#conditional-descriptor-bank-0-source).


**ROMDATA-02-GAMEPLAY-6DF4D4-BANK0 modello EUR/Wine (2026-10-31).**
Modello rigoroso per pointer0 e record1..15; predicato, store, modalità3 e
scansione originali passano su Wine per slot zero/nonzero. Record0 rimane raw;
uso ordinario, pubblicazione/runtime e Linux/USA restano aperti.
[Prove](../rom_data_gameplay.md#typed-conditional-descriptor-bank-0-model).


**ROMDATA-03-04-GAMEPLAY-6DF4D4-BANK0 gestito EUR/Wine (2026-10-31).**
e99-d98 pubblica pointer0 e15 descrittori: 88 risorse precedenti identiche,
confini con banca1 composti e record0 raw, modifica coerente rifiutata, tre
prefissi nativi equivalenti. CLI, uso ordinario e Linux/USA restano aperti;
nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#managed-conditional-descriptor-bank-0).


**ROMDATA-04-GAMEPLAY-6DF4D4-BANK0-CLI EUR/Wine (2026-10-31).** La CLI
di produzione crea e riusa e99-d98 e avanza un frame: 90 file identici al
dataset accettato, 89 identità runtime verificate e file gestiti invariati.
Uso ordinario e Linux/USA restano aperti; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-descriptor-bank-0-production-cli).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANK0-ORDINARY-PAIR EUR/Wine (2026-10-31).**
La schermata informazioni Item usa pointer0 e i campi0/2/tipo/byte7 dei
record. Sulla stessa sequenza ordinaria di800 frame il confronto ROM grezza
e dataset e99 coincide per stato/clock, video, OAM, dispatch e audio; il
campo4 non è osservato. Entrambi i lati condividono la cache nativa: Linux,
USA e Windows nativo restano da verificare. Nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#ordinary-item-info-access-to-descriptor-bank-0).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANK0-FIELD4-PAIR EUR/Wine (2026-10-31).**
Nel menu Item in battaglia una sequenza esistente di340 frame legge32 byte
tipizzati del campo4 della banca0. Il confronto ROM grezza/e99 coincide
per stato/clock, video, OAM, dispatch e audio, usando la stessa cache nativa
in entrambi i lati. Linux, USA e Windows nativo restano da verificare; nessuna
misura prestazionale. [Prove](../rom_data_gameplay.md#ordinary-battle-item-menu-bank-0-field-4).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-KIND-RECONCILE EUR/Wine (2026-10-31).**
Il log già conservato della scena Item in battaglia registra anche4 byte
puntatore e1 byte tipo della risorsa descrittori banche1..23. È uso aggregato,
senza attribuzione al singolo record/PC; il confronto della scena verifica
la banca0 senza asserire questa risorsa secondaria. Campo4/byte7 ordinari,
Linux e USA restano aperti. Nessun replay o misura prestazionale.
[Prove](../rom_data_gameplay.md#ordinary-ability-descriptor-scene-differential).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-INFO-PAIR EUR/Wine (2026-10-31).**
La schermata Battle Tech già registrata legge10 byte tipo e10 byte7 tipizzati
dei descrittori banche1..23. Su800 frame il confronto ROM grezza/e99 coincide
per stato/clock, video, OAM, dispatch e audio; campo4 non osservato e cache
nativa condivisa. Linux, USA e Windows nativo restano da verificare; nessuna
misura prestazionale. [Prove](../rom_data_gameplay.md#ordinary-battle-tech-descriptor-information).


**ROMDATA-05-GAMEPLAY-6DF4D4-BANKS-FIELD4-PAIR EUR/Wine (2026-10-31).**
Nel menu Steal una sequenza conservata di280 frame legge60 byte tipizzati del
campo4 nelle banche1..23. Il confronto ROM grezza/e99 coincide per stato/clock,
video, OAM, dispatch e audio; entrambi i lati usano la stessa cache nativa.
Linux, USA e Windows nativo restano da verificare; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#ordinary-steal-menu-descriptor-field-4).


**ROMDATA-01-GAMEPLAY-6E0B44-MASKS fonte EUR condizionale (2026-10-31).**
Tre lettori originali indicizzano maschere u32 con proprietà metadata52 0x24.
Le116 righe autenticate producono tutti gli indici0..34; l'intervallo sorgente
6e0b44..6e0bd0 contiene35 parole ed è distinto dalle risorse e99. Selettori
dinamici/invalidi non sono globalmente limitati: nessun cap guest. Modello,
runtime, uso ordinario e Linux/USA restano aperti; nessuna misura prestazionale.
[Contratto](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-02-GAMEPLAY-6E0B44-MASKS modello EUR/Wine (2026-10-31).**
Modello rigoroso lossless di35 maschere u32/140 byte, senza limiti guest
aggiunti; Wine verifica byte, confini, selettori originali e indici esterni.
Pubblicazione/runtime, uso ordinario, Linux e USA restano aperti; nessuna
misura prestazionale. [Prove](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-03-04-GAMEPLAY-6E0B44-MASKS gestito EUR/Wine (2026-10-31).**
e100-d99 pubblica35 maschere tipizzate, conserva esattamente89 risorse
precedenti, compone i confini con i proprietari adiacenti e rifiuta modifiche
coerenti; un prefisso nativo originale coincide con l'interprete nei sei casi
selezionati. CLI, uso ordinario, Linux e USA restano aperti; nessuna misura
prestazionale. [Prove](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-04-GAMEPLAY-6E0B44-MASKS-CLI EUR/Wine (2026-10-31).**
La CLI di produzione crea e riusa e100-d99 e avanza un frame neutro:91 file
identici al dataset accettato,90 identità runtime verificate e file gestiti
invariati. Le maschere non sono lette nel frame; uso ordinario e Linux/USA
restano aperti. Nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-05-GAMEPLAY-6E0B44-MASKS-ORDINARY-PAIR EUR/Wine (2026-10-31).**
La lista copricapi conservata legge91 volte le maschere tipizzate (364 byte
aggregati). Il confronto ROM grezza/e100 per560 frame coincide per stato/clock,
video, OAM, dispatch e audio; i due lati condividono la cache nativa. Linux,
USA e Windows nativo restano da verificare; nessuna misura prestazionale.
[Prove](../rom_data_gameplay.md#conditional-property-selected-mask-table).


**ROMDATA-04-DYNAMIC-FRAGMENT-USAGE-CLI EUR/Wine (2026-10-31).**
I frammenti dinamici espongono contatori separati per richieste, byte selettore
e payload. Un frame neutro della CLI Windows sotto Wine emette la nuova riga
con contatori zero, riusando il dataset e100 senza modifiche; uso ordinario,
Linux e USA restano aperti. Nessuna misura prestazionale.
[Prove](../rom_data_ui_raw_transfers.md#managed-dynamic-fragment-resource).

**Avanzamento EUR e101-d100 (2026-10-31).** La proiezione OBJ compatta
attestata è pubblicata e verificata con test mirato Wine per sorgenti, confini,
caricamento transazionale e un prefisso originale nativo/interprete. La CLI di
produzione, l'uso ordinario, la sequenza8 non attestata, Linux e USA restano da
verificare. Nessuna misura prestazionale. [Contratto](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6b9b80).

La CLI di produzione e101-d100 ha superato creazione, riuso e un avvio Wine;
l'uso ordinario della nuova risorsa OBJ compatta resta da dimostrare.

**Avanzamento EUR e102-d101 (2026-10-31).** Una seconda banca OBJ compatta
sparsa è pubblicata e verificata su Wine per sorgenti, confini, caricamento
transazionale e un prefisso nativo/interprete. CLI di produzione, uso
ordinario, voci non attestate, Linux e USA restano da verificare. Nessuna
misura prestazionale. [Contratto](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6c6460).

La CLI di produzione e102-d101 ha superato creazione, riuso e un avvio Wine;
l'uso ordinario della seconda banca OBJ compatta resta da dimostrare.

**Avanzamento EUR e103-d102 (2026-10-31).** Una terza banca OBJ compatta
sparsa, inclusa una lista vuota, è pubblicata e verificata su Wine per sorgenti,
confini, caricamento transazionale e un prefisso nativo/interprete. CLI,
uso ordinario, voci non attestate, Linux e USA restano da verificare. Nessuna
misura prestazionale. [Contratto](../rom_data_obj.md#conditional-compact-obj-animation-source-at-6bf490).

La CLI di produzione e103-d102 ha superato creazione, riuso e un avvio Wine;
l'uso ordinario della terza banca OBJ compatta resta da dimostrare.
