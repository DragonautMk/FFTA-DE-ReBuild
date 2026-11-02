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
completati per EUR/Wine dichiarato:**48byte strutturali6e6c9c..6e6ccc, root e
lunghezza copia originali. Costruttore mode1/2, quattro ID20..23con sorgenti
alternate, patch signedID/selector>1/argomenti/ORflag distinti. Un controllo Wine
passa271casi reali+30sintetici con inizializzatore IWRAM originale, tutti256ID,
mode/troncamento/bordi signed/selector/flag e preservazione bit/byte inutilizzati.
48Broundtrip/JSON/array/source/stack/args/callee-invalid/outerRAM passano; nessun
oracolo vecchio ripetuto,48812istruzioni/155PCsolo copertura. Allocazione/liveunit
dopo127EB0esclusi; alternate192B/puntatore ROM e successivi240Bnon ancora owned.
Dataset e28-d27/23file/22risorse invariato; preparazione/consumo/lifecycle/scenaCLI
e nuovi Linux/USA/Windows nativo da fare, nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#default-constructor-template-source-contract).

**ROMDATA-02-GAMEPLAY-DEFAULT48-DATA / ROMDATA-04-GAMEPLAY-DEFAULT48-CONSUMPTION
completati per EUR/Wine dichiarato:**e29-d28aggiunge default-constructor-template.json /
resource gameplay.default-constructor-template,24file/23risorse,22file precedenti/
e28conservati; USAe8-d7. Byte header/argomenti/halfword/flag/tail conservano copia
originale ROM/IWRAM, patch live, mode/alternative/allocazione/clock/save/native ABI.
Due controlli Wine passano pubblicazione/transazioni/lifecycle/retention e16prefissi
originali/interprete/nativi con48rawBalterati/stati-clock uguali;8rami alternativi/
rifiutati0newreads,39/21/272/16/32Bcompilati, tutti byte/bordi/BIOS/mirror/SWI/DMA4/
crossing2metadata2+2default48. Build corretta warning-free dopo macrocontatore
rimasta a2elementi anziché5; log iniziale conservato. Sorgente271+30non ripetuta.
ROMDATA-05-GAMEPLAY-DEFAULT48-SCENE-CLI completo: CLI crea/riusa senza scritture
con vecchia cache/cwd diverso; una coppia397Pub stati/clock/immagini/OAM/dispatch/
audio passa con96Bnuovi=10header/6argomenti/68halfword/4flag/8tail, immagine
ispezionata. Nuova famiglia/gate giustifica input conservato; nessun locator
negativo, RAMedit/recording nuovo o vecchio oracolo ripetuto. Tail allocazione,
alternate192B/puntatore ROM e successivi240Bnon ancora owned. Nuovi Linux/USA/
Windows nativo/restoTODOaperti; nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-default-constructor-template).

**ROMDATA-01-GAMEPLAY-CONSTRUCTOR-ALTERNATES completato staticamente:**4record
48B/192Ba6eca78..6ecb38+puntatore4Ba70cd58,196Btotali/noe29overlap. RootLDR,
LDR+4, IDu8gated20..23/stride48/copia48 e roundtrip indipendente passano.
Consumatore CCF9C..CD046legge cinque equipment halfword08..10perrecord attraverso
item proprietà6/3;20linkentro376item parsed. Byte01e header vicini non ricevono
semantiche non provate. Modello/oracolo sorgente/preparazione/consumo/scena restano
da fare; EUR e29-d28invariato, prove default271+30/native16/397chiuse.
[Contratto](../rom_data_gameplay.md#next-rooted-constructor-sources).

**ROMDATA-02-GAMEPLAY-CONSTRUCTOR-ALTERNATES-SOURCE-ORACLE completato per EUR/Wine:**
modello strict4record/192B+puntatore4B/196B, ID20..23/offset/campi byte/halfword e
cinque equipment halfword perrecord con owner376item parsed. Un controllo Wine
passa24casi reali+8sintetici di copia/patch originali, troncamento u8, selector/
flag/mode, highbit/unused/highitem/ORpreservati e JSON/source/array/order/ID/extent/
stack/args/outerRAM.5560istruzioni/137PCsolo copertura; tail allocazione esclusa,
synthetic non autenticati/non pubblicati. Default271+30/native16/397non ripetuti.
Dataset e29-d28/24file/23risorse invariato; preparazione/consumo/lifecycle/scenaCLI
e nuovi Linux/USA/Windows nativo da fare, nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#typed-constructor-alternates-source-proof).

**ROMDATA-02-GAMEPLAY-CONSTRUCTOR-ALTERNATES-DATA / ROMDATA-04-GAMEPLAY-CONSTRUCTOR-ALTERNATES-CONSUMPTION
completati per EUR/Wine dichiarato:**e30-d29aggiunge constructor-alternate-templates.json /
resource gameplay.constructor-alternate-templates,25file/24risorse,23file precedenti/
e29conservati; USAe8-d7. Owner Items parsed, puntatore4B/record192Btipizzati e mask
perrecord conservano copia ROM/IWRAM/patch live/mode/allocazione/clock/save/native ABI.
Due controlli Wine passano pubblicazione/transazioni/lifecycle/retention e14percorsi
con196rawBalterati/stati-clock uguali, tutti4record mask15/puntatore compilati,
5default/rifiutati0newB/vecchio default positivo. Tutti byte/bordi record/BIOS/mirror/
SWI/DMA4per entrambe le sorgenti/crossing2typed+2raw passano. Build warning-free;
guardia zero-use corretta in review prima di Wine, nessun failure. Source24+8e
vecchie prove default271+30/native16/397chiuse.
ROMDATA-05-GAMEPLAY-CONSTRUCTOR-ALTERNATES-SCENE-CLI completo: CLI crea/riusa senza
scritture/vecchia cache/cwd diverso. Intro3000cortile scuola e Lutia1700NOTICE sono
locator negativi conservati, nessuna coppia completa. Conferme/schieramento normali
danno una coppia3600Lutia stati/clock/immagini/OAM/dispatch/audio uguali e45nuoviB:
4pointer/6header/3argomenti/28halfword/2flag/2tail, mask1soloID20. Tutti4record e
copia completa restano prove mirate; immagine finale ispezionata, nessun RAMedit/
recording nuovo/salvataggio persistito/vecchio oracolo ripetuto. Nuovi Linux/USA/
Windows nativo/header vicini/successivi240B/restoTODOaperti, nessuna misura performance/
spazio. [Contratto](../rom_data_gameplay.md#managed-and-direct-constructor-alternates).

**ROMDATA-01-GAMEPLAY-METADATA14-ROOTS completato staticamente nel catalogo dichiarato:**
93record/1302Bfisici6e6f6c..6e7482,IDlogici2..94/base biased6e6f50. Predicate>1
nei getter mantiene0/1sul ramo job live; root successivo indipendente6e7484usa
stride12e lascia2Bnonowned. Getter halfword4/6,byte11/12/13 e palette packed8..10
individuati; roundtrip/noe30overlap/186valori01/02entro116metadata52passano. Byte02
copia al job liveconoverride0; owner semantico byte01non provato solo dal range.
Ipotesi6e6dbc14Brespinta(reader*4), presunti alias7484/78d4respinti(stride12).
Modello/oracolo sorgente/consumo da fare,e30invariato; nessun build/test/ripetizione
vecchia accettazione, nuovi Linux/USA/Windows nativo da fare.
[Contratto](../rom_data_gameplay.md#next-bounded-fourteen-byte-records).

**ROMDATA-02-GAMEPLAY-METADATA14-SOURCE-ORACLE completato per EUR/Wine dichiarato:**
modello strict93record/1302B,ID2..94/basebiased6e6f50/fisici6e6f6c..6e7482,
tutti byte/halfword/packed, owner metadata52parsed per byte02solo; byte01non
vincolato semanticamente dal solo range. Un controllo Wine passa504casi reali:
471getter/tutti93ID/troncamento u8,22palette con book RAM0/1/2/default3/copia32B/
guardie512B e11prefissi inizializzazioneCCF9C→CD0B6/override/flag/frame264Bunit.
13sintetici highu16/u8/bank15/unused/byte01unproved/altojob valido passano insieme
JSON/source/array/bias/ID/order/count/owner/callee/stack/descriptor/unit/palette/
outerRAM.23627istruzioni/380PCsolo copertura, nessun vecchio oracolo ripetuto.
Allocazione e bound globali esclusi;0/1/95plus/2Bpadding nonowned, nessun fallback.
Dataset e30-d29/25file/24risorse invariato; preparazione/consumo/scenaCLI/lifecycle
e nuoviLinux/USA/Windows nativo da fare, nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#typed-fourteen-byte-records-source-proof).

**ROMDATA-02-GAMEPLAY-METADATA14-DATA / ROMDATA-04-GAMEPLAY-METADATA14-CONSUMPTION /
ROMDATA-05-GAMEPLAY-METADATA14-SCENE-CLI completati per EUR/Wine dichiarato:**
e31-d30 aggiunge linked-metadata14.json/gameplay.linked-metadata14, 26 file /
25 risorse; tutti i 24 file precedenti/e30 conservati, USAe8-d7. Owner metadata52
parsed solo per byte02; byte01 senza vincolo semantico. Proiezione tipizzata di
1302B, contatori e maschere dei 93 record preservano getter originali, book RAM,
copie palette, override, clock e ABI native/save. Due controlli Wine passano
pubblicazione/riuso/riparazione/spostamento/retention/guardie transazionali e
Load/Undo/backup/restart con copia dataset indisponibile. 37 percorsi originali/
interprete/native con raw1302B alterati hanno stati-clock identici; ID2/20/28/94
positivi, normali0/1 senza nuove letture. Bus/edge/width/mirror/BIOS/SWI/DMA e
confini typed/raw passano. CLI reale crea/riusa senza scritture e usa la vecchia
cache da cwd estraneo; una sola coppia397Pub completa stato/clock/immagini/OAM/
dispatch/audio con27B/due record/tutti i gruppi di campi, immagine ispezionata.
Oracolo sorgente504+13 e prove precedenti non ripetuti. Bound globali, payload
palette, code di allocazione, famiglie adiacenti e nuoviLinux/USA/Windows nativo
da fare; nessuna misura performance/spazio o modificaAndroidPort.
[Contratto](../rom_data_gameplay.md#managed-and-direct-fourteen-byte-special-unit-metadata).

**ROMDATA-01-GAMEPLAY-METADATA12-ROOTS / ROMDATA-02-GAMEPLAY-METADATA12-SOURCE-ORACLE
completati per EUR/Wine sorgente dichiarato:** due banchi con stride12 a
6e7484..6e78d4/92record e6e78d4..6e7994/16record, totale108record/1296B/5184coppie
di bit. Root/callee135a1c/sorgente successiva indipendente6e7994 verificati;
modello strict gruppi12×4 valori0..3 conserva48coppie anche se i loop usano0..43.
Indici row32 e slot signed/wrapping originali, mode troncato u8, nessun bound o
fallback guest nuovo. Un controllo Wine passa1640reali/10raw-fuori-wrap/384
sintetici, indirizzo/LDRB esatti, JSON/source/callee/stack/RAM. Build iniziale con
errori setup test corretto, rebuild Windows senza avvisi; nessun errore Wine.
EUR e31-d30 invariato; preparazione/consumo/lifecycle/scena, significato campi,
bound globali e nuoviLinux/USA/Windows nativo da fare. Nessuna misura performance/
spazio o prova precedente ripetuta.
[Contratto](../rom_data_gameplay.md#indexed-twelve-byte-packed-pair-source-contract).

**ROMDATA-02-GAMEPLAY-METADATA12-DATA / ROMDATA-04-GAMEPLAY-METADATA12-CONSUMPTION /
ROMDATA-05-GAMEPLAY-METADATA12-SCENE-CLI completati per EUR/Wine dichiarato:**
e32-d31 aggiunge linked-metadata12.json/gameplay.linked-metadata12,27file/26risorse,
25file precedenti/e31 conservati; USAe8-d7. Campi a due bit tipizzati, due owner
sorgente, contatori di banco/maschere108record mantengono row32/slot signed-wrap/
modeu8/clock/ABI originali senza dipendenze semantiche inventate. Due controlli
Wine passano pubblicazione/riuso/riparazione/spostamento/retention/guardie
transazionali/Load-Undo-backup-restart con copia dataset indisponibile.
49percorsi originali/interprete/native con1296rawB alterati hanno stati-clock
identici; entrambi i banchi positivi, tre percorsi esterni0nuove letture.
Bus/edge/width/mirror/BIOS/SWI/DMA e confini raw/typed/tra banchi passano.
Una coppia2200-Echo Grass confirmation stato/clock/immagini/OAM/dispatch/audio/CLI positiva
completa il perimetro ordinario dichiarato. Oracolo sorgente1640+10+384/e31 non
ripetuti. Bound globali/semantica flag/banco globale separato/altre famiglie e
nuoviLinux/USA/Windows nativo da fare; nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-twelve-byte-packed-pair-banks).

**ROMDATA-01-GAMEPLAY-GLOBAL-PAIR-FLAGS / ROMDATA-02-GAMEPLAY-GLOBAL-PAIR-FLAGS-SOURCE-ORACLE
completati per EUR/Wine sorgente dichiarato:** root singolo6e7994,44coppie/11B
fino6e799f da loop RAM0..43/mode0; nessuno stride12,3Bprima root indipendente
6e79a2 nonowned. Modello strict11×4 valori0..3 conserva entrambi i bit. Un controllo
Wine passa102reali/10raw-fuori-wrap/91sintetici, wrapper conr0ignorato/slot signed-
wrap/modeu8/filtro maschera live/uscita anticipata/scansione44/JSON/source/callee/
stack/RAM.13601istruzioni/70PC solo copertura; nessun build/Winefailure/prova
precedente ripetuta. EUR e32-d31 invariato; preparazione/consumo/lifecycle/scena,
highmode ordinario, semantica/padding e nuoviLinux/USA/Windows nativo da fare.
[Contratto](../rom_data_gameplay.md#global-packed-pair-flags-source-proof).

**ROMDATA-02-GAMEPLAY-GLOBAL-PAIR-FLAGS-DATA / ROMDATA-04-GAMEPLAY-GLOBAL-PAIR-FLAGS-CONSUMPTION
completati per EUR/Wine; ROMDATA-05CLI concluso, uso ordinario globale da fare:**
e33-d32 aggiunge global-pair-flags.json/gameplay.global-pair-flags,28file/27risorse,
26file precedenti/e32conservati, USAe8-d7.11Btipizzati/maschera posizioni byte
mantengono ignoredr0/slotwrap/modeu8/liveRAM/uscita anticipata/clock/ABI senza
owner/padding inventati. Due controlli Wine passano pubblicazione/retention/
guardie transazionali/Load-Undo-backup-restart;28percorsi native/interprete con
raw11Balterati hanno stati-clock identici,17Bcompiled/8esterni0nuoviB.
Tutti11B/bus/mirror/BIOS/confine2vecchi2nuovi/SWI11typed/DMA3typed1raw e1typed3raw
passano. CLI crea/riusa senza scritture, vecchia cache/cwd estraneo; unica scena
2200EchoGrass negativa(0globali/44metadata12), nessuna coppia completa su di essa.
Mancano uso ordinario globale positivo/relativa coppia completa; reader131db8
richiede descriptorhword4 e maschera live nonzero, servono condizioni concrete
prima di altri replay. Highmode/newLinux/USA/Windows nativo/3Badiacenti da fare.
Oracoli102+10+91/e32 non ripetuti; nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-global-packed-pair-flags).

**ROMDATA-01-GAMEPLAY-NUMERIC-PAIR-ROOTS / ROMDATA-02-GAMEPLAY-EVALUATION-PARAMETERS-SOURCE-ORACLE
completati per EUR/Wine sorgente:**38B in due sorgenti:7coppie divisore/peso e2
bonus condizionali a6e79a2..6e79b4;5classi soglia1..5con stride4 a6e79c6..6e79da.
Classe0saltaRNG/lettura,14Btra sorgenti e4Brow0nonowned; ipotesi32/24B respinta.
Modello strict7/2/5array/ID/offset/campi/opaque e un controllo Wine passano132casi
sorgente originale (12prefissi score live, frammenti aritmetici/bonus/RNG/suffix
con argomenti iniettati) e52sorgenti mutate. Prima divisioneunsigned/u8,6signed32,
wrap, bonuslow8==1/==0, LCG originale/sogliacomparison signedlow16%256 preservati.
Guardie JSON/source/ID/order/offset/callee/stack/unit/context passano; nessun
build/Winefailure/prova precedente ripetuta.24147istruzioni/442PC solo copertura.
Zerodivisor solo diagnosticNone/guest invariato, non eseguito qui; wholefunction
nonclaimed. e33invariato; preparazione/consumo/lifecycle/scena/semantica e nuovi
Linux/USA/Windows nativo da fare, nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#evaluation-parameters-source-proof).

**ROMDATA-02-GAMEPLAY-EVALUATION-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-EVALUATION-PARAMETERS-CONSUMPTION /
ROMDATA-05-GAMEPLAY-EVALUATION-PARAMETERS-SCENE-CLI completati per EUR/Wine score dichiarato:**
e34-d33 aggiunge evaluation-parameters.json/gameplay.evaluation-parameters,
29file/28risorse/27file precedenti ed e33conservati, USAe8-d7.38Btipizzati e
contatori classi/campi preservano produttori live/divisioni unsigned-u8 e signed/
wrap/bonus/RNG/clock/ABI. Due controlli Wine passano preparazione/retention/
guardie transazionali/Load-Undo-backup-restart e27prefissi native/interprete con
raw38Balterati/stati-clock identici; classmask31,classe0zero,opaque0compiled.
Bus/edge/mirror/BIOS/SWI18-20typed/DMA2typed2raw/18Bgap-row0raw passano.
Build iniziale con avviso import inutilizzato rifiutato dal wrapper; campioni
native sovrapposti corretti prima di Wine, rebuild senza avvisi, nessun errore
Wine. CLI crea/riusa senza scritture/vecchia cache/cwd estraneo; primo1840Pubmission
positivo15scoreB(7divisori/7pesi/1bonus), una sola coppia completa stato/clock/
immagini/OAM/dispatch/audio passa. La successiva400firma/fee chiudeclass1; classi2..5restano targeted-only,
fonte132+52/provee28/e33 non ripetute. Wholefunction/semantica/gap/global11Bordinary/
nuoviLinux/USA/Windows nativo da fare; nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-evaluation-parameters).

**ROMDATA-01-GAMEPLAY-METADATA10-ROOTS completato nel perimetro statico dichiarato:**
root6e79da/stride10/default0 e producer020021b4in04B3E0..04B416 fissano0..99,
100record/1000Bfino6e7dc2. Candidato101respinto;12Bzero prima root6e7dcenonowned,
01A5A6display non è prova setter. Otto u8delta sono passati al consumer04870c
con stato live/carry/wrap/cap99/flag; byte08usato in prezzo Items con optional+10u8,
divisione signed/minimounsigned/zero→1, byte09structuralunused. Roundtrip/noe34overlap/
root/stride/field opcodes passano. Modello e sourceoracle completati sotto.
Preparazione/consumo/scena/nomi campi/newLinux-USA da fare.
[Contratto](../rom_data_gameplay.md#next-ten-byte-progression-and-price-records).

**ROMDATA-02-GAMEPLAY-PROGRESSION-REWARDS-SOURCE-ORACLE completato per EUR/Wine:**
modello strict100record/1000Bcon8delta/discount08/opaque09, senza vincoli basati
sui soli valori osservati. Build13783senza avvisi; unico Wine62460passa362casi
reali (7publisher/232caller+helper04870c/123prezzi completi) e68sintetici.
Items parsed autenticati, mask/predicato originali, overflowu8/carry/cap99/
oldinvalid/flag, signeddivisione/minimounsigned/zero, JSON/roundtrip/ID/ordine/
sourceoffset/SP/callee/EWRAM passano. Fuori100/255 resta raw; sintetici non
autenticati/pubblicati;37file protetti invariati. Fonte chiusa, preparazione/
consumo/scena/nuoviLinux-USA-Windowsnativo restano; nessuna misura prestazionale.

**ROMDATA-02-GAMEPLAY-PROGRESSION-REWARDS-DATA / ROMDATA-04-GAMEPLAY-PROGRESSION-REWARDS-CONSUMPTION completati per EUR/Wine:**
e35-d34 aggiunge progression-rewards.json/gameplay.progression-rewards,
30file/29risorse, tutti28decoded precedenti/e34invariati, USAe8-d7. Build84154
senza avvisi/Wine80590due controlli mirati passano pubblicazione/riuso/repair/
move/coherent edits/transactional/load-undo-backup-restart.27percorsi originali/
interprete/native con1000Braw alterati hanno stati-clock identici,17delta/
4discount/0opaque compiled,6percorsi publisher/ungated/outsidezero. Tutti1000B/
100record/boundary/width/mirror/BIOS/SWI32typed/DMA2typed2raw/crossing2eval2new
passano. Fonte362+68chiusa/non ripetuta; CLI/scena completati sotto, newLinux-USA-
Windowsnativo da fare.
[Contratto](../rom_data_gameplay.md#managed-and-direct-progression-rewards).

**ROMDATA-01-GAMEPLAY-PROGRESSION-GATES-ROOTS completato nel perimetro statico:**
0487F6→048868/stride12/index0..63fissano64record/768B6e7dce..6e80ce,
nextrootD23E6→D24A8indipendente allo stessoend. Byte00unused, halfword01return,
byte03output,8u8minimi contro020021b6+2*k; originale masku64/ordine prima
riga unmarked idonea/bitfisico/zero-return con bit63successo distinti dal miss.
Tutti768Broundtrip/noe35overlap/root/field/stride/count/caller/next opcodes
passano; ritorni attuali0..375compatibili Items, receipt/FK completo nonprovato.
Il solo inventario non aggiungeva modello/build/Wine; sourceoracle completato sotto,
managed/runtime/newLinux-USA-Windowsnativo restano.
[Contratto](../rom_data_gameplay.md#next-progression-gates-and-returned-receipts).

**ROMDATA-02-GAMEPLAY-PROGRESSION-GATES-SOURCE-ORACLE completato per EUR/Wine:**
modello strict64record/768Bbyte00/halfword01/byte03/8minimi, nessun vincolo
byte00==ID/minimo<=50/receipt==item. Outcome distingue ID idoneo/valore/output/
maschera. Build75099senza avvisi/unicoWine11775passa80casi reali+25sintetici,
originale0487c8completo+helperu64: tutti64row/equal-below8dimensioni/unsigned128/
bit31-32-63/ordine primo idoneo/fullmask/row63successo valore0;704Bcampi letti,
768Broundtrip/JSON/ID-offset-order/coherentedit/SP/callee/EWRAM passano.
38file protetti invariati, e35unchanged; managed/runtime/lifecycle/ordinaryuse/
receiptUI-FK/newLinux-USA-Windowsnativo da fare. Nessun vecchio test ripetuto.
[Contratto](../rom_data_gameplay.md#progression-gates-source-contract).

**ROMDATA-02-GAMEPLAY-PROGRESSION-GATES-DATA / ROMDATA-04-GAMEPLAY-PROGRESSION-GATES-CONSUMPTION completati per EUR/Wine:**
e36-d35aggiunge progression-gates.json/gameplay.progression-gates,31file/
30risorse/all29decoded precedenti/e35retained/USAe8-d7. Windows68535senzaavvisi/
Wine97635duecontrolli passano pubblicazione/interruzione/create-reuse-repair-move/
retention/coherent requirement-return-output-unused-lastrow-reorder-source-
dependency-path-role/transactionalownerstate/load-undo-backup-restart.19percorsi
originale/interprete/native con768rawBalterati/stato-clock identici; compiled369
requirements/18return/9output/0unused,2successivalore0marcanobit,2fullmaskzero.
All768B/64record/boundary/width/mirror/BIOS/twopages/SWIcrosspage32typed/DMA2typed2raw/
12Brawgap/oldreward2newgate4coexistence passano. Fonte80+25chiusa/nonripetuta.
CLI/scena05completati sotto; receiptUI-FK/newLinux-USA-Windowsnativo restano,
nessunamisuraperformance.
[Contratto](../rom_data_gameplay.md#managed-and-direct-progression-gates).

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


## ROMDATA-05 — Equivalenza, salvataggi e prestazioni

**ROMDATA-04-TEXT-CATALOG-CONSUMPTION completato per EUR/Wine dichiarato:**
modello completo dal decimo file, letture tipizzate CPU/DMA/native per puntatori,
stringhe, indici relativi, header, programmi, glifi/metriche; letterali LZ/RLE
da unità decodificate, controlli originali esatti. Tre controlli Wine passano:
sintetico con sorgenti/letterali duplicati alterati,15catene originali/native
con33581intervalli ROM alterati e90stati/clock uguali, Load/Undo/backup/riavvio
con copie dei file indisponibili. Uso compilato positivo negli8tipi; contesto
dinamico valido esplicito, nessuna scena ordinaria dedotta. Sottoinsieme vecchio
autenticato senza doppio owner/contatori. e12-d11/save/native ABI invariati.
Scene/CLI05, altre radici/font/Unicode, Linux/USA da fare.
[Prove](../rom_data_text.md#broad-direct-consumption).

**ROMDATA-04-TEXT-SHORT-CONSUMPTION completato per EUR/Wine dichiarato:**
owner immutabile con indici/codici/glifi/metriche dal nono file, pagine sorgente
fisse e contatori distinti. Tre controlli Wine passano: sintetico larghezze/
mirror/bordi/SWI/DMA/Load, cinque catene originali/native con tutti670intervalli
ROM alterati e20chiamate/stati/clock uguali, Load/Undo/backup/riavvio con copie
temporanee dei file indisponibili. API native leggono20byte tabelle/124stringhe/
636glifi/6metriche tipizzati; modalità fissa salta le metriche come l'originale.
e11-d10/save/native ABI invariati. Scene gameplay ordinarie, altri cataloghi/font/
Unicode/comandi, Linux/USA da fare. [Prove](../rom_data_text.md#direct-consumption).

**Dipendenze:** ogni integrazione ROMDATA-04, prima di abilitarla normalmente.

- [ ] Confrontare originale e dataset estratto senza modifiche, con stessi
  input: stato guest a fine frame, RNG, clock, RAM/VRAM/OAM, immagini e audio
  deterministico. Coprire intro, mappa, battaglia, menu e transizioni pertinenti;
  riutilizzare le prove già valide senza confondere copertura e completezza.
- [ ] Verificare percorsi nativi e interprete, Fast, pausa, load/Undo e ripristino
  backup. Ricostruire risorse host e cache dopo il caricamento senza serializzare
  puntatori o duplicare tutti i file estratti negli stati macchina.
- [ ] Definire identità e compatibilità di `.ffstate` e `.sav`: il dataset base
  equivalente deve preservare gli stati esistenti quando possibile. Se cambiano
  layout o semantica, versionare e rifiutare combinazioni incompatibili prima di
  modificare la macchina o scrivere il salvataggio.
- [ ] **Rinviato dall’utente:** misurare separatamente estrazione iniziale, avvio con dataset pronto,
  spazio su disco, RAM e tempi di frame/caricamento/audio rispetto al percorso
  attuale in memoria. Fissare i limiti di regressione prima del confronto.
  Registrare anche un risultato neutro o peggiore: l'estrazione non prova velocità.
- [ ] Verificare Windows nativo; usare Windows/Wine e Linux come prove distinte.
  Aggiornare launcher/documentazione/packager senza includere ROM, risorse estratte,
  save personali o codice generato nei repository e nei pacchetti distribuiti.

**Consegna principale:** estrazione automatica, riuso all'avvio e consumo diretto
verificati per la matrice dichiarata, equivalenza senza mod e dipendenze residue
documentate. Le misure dei costi sono rinviate e non bloccano il lavoro funzionale. Le fasi mod seguenti non bloccano questa consegna.

**ROMDATA-05-OBJ-SCENES-COSTS completato nel perimetro locale dichiarato:**
[protocollo e risultati](ROM_DATA_VALIDATION.md),
[record](../changes/2026-10-21-validate-decoded-obj-scenes-and-costs.json).
Confronti Linux e Windows/Wine su mappa240, battaglia/menu3530 e avvio fino alla
battaglia8476frame: ogni stato completo, immagine/OAM, conteggio e flusso audio
coincide; lettura effettiva dell'alias EUR al frame5400. Compatibilità base degli
stati e dei save preservata, senza nuova versione. Cinque coppie Linux contro il
legacy corretto passano i limiti fissati prima della misura: +1,0081%/-0,2355%
di costo medio; risultato neutro. Sette primi avvii/riusi: mediane32,416/21,019ms
Linux e109,222/91,891ms Wine; massimo riuso Wine183,130ms conservato, con gate
sulla mediana. RSS Linux e spazio disco rispettano i budget. Test sintetico
archivi tar/zip esclude dati privati e include le nuove istruzioni.

**ROMDATA-05-OBJ-GUI completato nel perimetro EUR Linux/Windows-Wine:**
Fast tenuto e commutato, pausa/ripresa mantenendo Fast, ritorno1x, Load/Undo/
ricaricamento e ripristino backup dal menu passano con uso delle risorse,
catture ispezionate, veri callback44,1/48kHz e file protetti invariati.
[Record e limiti](../changes/2026-10-21-validate-decoded-resource-gui-lifecycle.json).
Il gain finale è0: nessuna prova di udibilità o prestazioni callback A/B.
**ROMDATA-05-OBJ-ARCHIVES completato nel perimetro dichiarato:** corrette le
copie/allowlist Docker per includere le guide richieste dal packager; archivi
reali con395test Linux/392Windows-Wine passati (24opzionali ignorati ciascuno).
Privacy, PE/ELF, TinyCC automatico e GUI del runtime effettivamente estratto con
nuova preparazione portable/Load/Undo/ricaricamento passano.
[Record e limiti](../changes/2026-10-21-validate-decoded-resource-portable-archives.json).
Archivi/binari precedenti invariati; variazione di un metadato BUILD-INFO
registrata esplicitamente.

**ROMDATA-05-OBJ-USA-INITIALIZER completato su Linux e Windows/Wine:** avvio
pulito senza save né PC forzato fino al copier originale USA, frame791. Tutte
le13istruzioni coincidono per stato completo; CpuSet legge24halfword dal file
e copia correttamente3648byte, inclusi48di metadati. Anche la prova con il solo
vecchio alias alterato dopo l'attivazione autentica passa.
[Prova e limiti](ROM_DATA_VALIDATION.md#original-usa-initializer).
Non è una verifica completa delle scene/audio USA. ROMDATA-05 resta parziale
per Windows nativo, non disponibile all'utente: per ora solo Linux/Wine.
Abilitazione normale e altre famiglie/mod restano aperte.

**ROMDATA-05-BG-SCENES-COSTS parziale:** parità completa Linux/Wine su mappa240,
battaglia/menu3530e transizione8476frame; letture BG53nella transizione. Cinque
coppie Linux passano i limiti di frame (+1,3600%/-0,6017%mediana media), così
come avvio/riuso e spazio disco. Il picco RSS invece supera il limite2MiB:
+2864KiBmediana/+2944KiBmassimo. Risultato conservato, nessuna abilitazione
normale. L'attribuzione individua la preparazione/parsing TOML; i prototipi
TOML alternativo e inline sono respinti. Il prototipo JSON ha individuato il parsing diretto con22528Bdi heap; la
revisione successiva adotta JSON standard leggibile, verificato nel runtime,
seguendo la nuova preferenza utente descritta sotto.
[Protocollo e risultati](ROM_DATA_VALIDATION.md#bg-metadata-e3-results-and-rss-failure).
Le prove scene/tempi concluse non vanno ripetute per ricostruire il contesto;
RSS, GUI/archivi BG e Windows nativo restano aperti.


**ROMDATA-02-BG-JSON completato:** revisione1/4/3, BG in JSON leggibile standard
con parsing diretto nelle strutture tipizzate; manifest/OBJ restano TOML.
Generazioni e3precedenti conservate; autenticazione, guardie sugli input,
riuso/riparazione e consumo nativo/interprete passano su Linux/Wine.
[Prove](../rom_data_graphics.md#readable-json-datasets).
**Preferenza aggiornata:** nessun lavoro per comprimere o risparmiare spazio
nei dati estratti. I149261byte BG/150818del dataset sono accettati per
leggibilità; limiti del parser proteggono da input malformati. I risultati dei
costi e dei prototipi restano informativi, senza nuovi cicli per centrare il
precedente limite disco. Prossima priorità: payload e cataloghi ancora in ROM.

**ROMDATA-05-BG-BASE-PALETTE-SCENES completato per le scene EUR dichiarate:**
nuovi confronti con dataset e6passano240frame mappa,3530battaglia/menu e8476
transizione su Linux/Wine. Tutti gli stati completi, immagini/OAM, dispatch e
flussi audio coincidono. Le scene stabili non caricano palette; la transizione
legge una palette decodificata da160B e53descrittori BG. Fixture e cache precedenti
riusate e protette. [Risultati](ROM_DATA_VALIDATION.md#base-palette-e6-scene-acceptance).
Costi/GUI/archivi e Windows nativo restano gate separati; nessuna abilitazione
normale o migrazione di altre famiglie è dedotta da queste scene.

**ROMDATA-05-BG-ALTERNATE-SCENES / GUI completati nel perimetro EUR dichiarato:**
LCD B/TV confermati nel menu originale, ingresso nel pub di Cyril con BG62.
Quattro coppie native397frame Linux/Wine coincidono per ogni stato completo,
immagine centrale/wide, OAM, dispatch e tutti gli stream audio non vuoti.
Uso alternativo positivo: LCD B165byte letterali/2link, TV153/2. Le GUI native
Linux/Wine presentano entrambe le modalità e chiudono0con318byte letterali/4link.
Snapshot/ROM/cache/dataset/canonici protetti; nessuna forzatura RAM/PC o misura
prestazionale. Restano scene/regione/lingue più ampie, FFglobali, archivi e Windows
nativo. [Prove](ROM_DATA_VALIDATION.md#alternate-palette-e7-scene-and-gui-acceptance).

**ROMDATA-05-GAMEPLAY-PROGRESSION-REWARDS-SCENE-CLI completato nel perimetro EUR/Wine:**
produzione33726senza avvisi/canonicalripristinato; CLI25924create/reuse/no-write/
vecchia cache/cwd estraneo passano,32file gestiti invariati perhash/mtime.
Primo600town→Shop→Buy positivo7discountB/record4(mask16), una sola coppiaWine52469
ha ogni stato/clock/immagine/OAM/dispatch/audio identici. SfondoShop ispezionato;
pannello inventory separato non visibile nel PPM, selezione/transazione non
inferite. Delta ordinary resta targeted-only. Nessun negativo/nuovo recording/
RAMedit/scritture save/prove362+68o27native ripetute; nuoviLinux-USA-Windowsnativo/
wholecaller/campi semantici da fare. Nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-progression-rewards).

**ROMDATA-05-GAMEPLAY-PROGRESSION-GATES-SCENE-CLI completato nel perimetro EUR/Wine dichiarato:**
produzione65421senzaavvisi/canonicalripristinato; CLI70968prepare/reuse-no-write/
oldcache/unrelatedcwd/1framebattlecompat passano,33file hash-mtime invariati,
zero newuse non scambiato per scena positiva. Locatorstatic174EURheaders/49payload
rappresentativi non ha flagsidonei, nessunafalsaprovaordinary. UnicoGUIWine92614
normalMadAlchemist10giorni/Sigmund/fee600/day9→19concheckpointUI2-3-4 nuovi
necessari (nessunpayloadedit/savepersonale); MissionCleared/rewards/2skilllevelup
ispezionati,209requirements/8delta/1thresholdGUI. Nuovo1000CLI29523positivo209/8;
unicoWine57387fullpair1000stati-clock-immagini-OAM-dispatch-audio identici. E35
row0otto deltaordinary chiusi; e36return/outputordinary targeted-only, threshold1
GUIavviene prima slot3; il successivo400frame Yes/fee pair sotto chiudeclass1. Sharedtestexe
cambiato dal lavoroDebug parallelo: verificatoree36immutato, copiapinnata9e5e6f0c
conquelcodicecompilatoparallelo; nessunafalsaidentitàcolvecchio19native. Provefonte/
native/lifecycle vecchie nonripetute, nessunfullpairsuicoperturanegativa. Nuovi
Linux/USA/Windowsnativo/receiptUI-FK restano, niente performance/spazio/Android.
[Contratto](../rom_data_gameplay.md#ordinary-progression-gates-dispatch-completion).

**ROMDATA-05-GAMEPLAY-EVALUATION-THRESHOLD-ACCEPT-SCENE completato per EUR/Wine class1:**
unicoGUI17436salva normale slot2 prima di Yes, conferma600gil/funds7109 e chiude0;
nessuneditpayload o savepersonale, originali/copia1protetti. Il GUIfermo alla fee
legge0threshold: confineeffettivo include conferma e acknowledgementfee. Primo
CLI70012400framepositivo16evaluationB(7divisor/7weight/1bonus/1threshold), mask1.
UnicoWine96264fullpair400stati-clock-immagini-OAM-dispatch-tuttiaudio identici,
stesso threshold1. Binari46c8cb78/9e5e6f0c e dataset/cache riusati; nessunbuild/
132+52/native27/1840score/1000completion ripetuto. Classi2..5/wholecaller/newLinux/
USA/Windowsnativo ancora dafare; nessunuso gates/rewards o outcomeinferito.
[Prova](../rom_data_gameplay.md#ordinary-evaluation-threshold-acceptance).

**ROMDATA-01-GAMEPLAY-PROGRESSION-RANK-GATES-ROOTS / ROMDATA-02-GAMEPLAY-PROGRESSION-SELECTION-GATES-SOURCE-ORACLE completati EUR/Wine sorgente:**
64record×16/1024B6e80de..6e84de daD2390/shift4/indici1..64; riga0/16Bnonowned,
nextrootindipendente6e84e0lascia2B. Minimiunsigned01/02/04tutti necessari;
06..0d richiedeANYsogliattiva, flag0e originale, primoeligible scriveselector00.
Senza condizioni scrive0eferma, esaurire64non scrive; riga52sentinel/53..64storage.
Resetlow8/input e count/flaghelper originali, nessunselector==IDschema. Modello
strictsource-only/roundtrip1024B e singolobuild senzaavvisi/oracleWine82907passano
138reali+80sintetici fullD2390/reset-count-flag:831fieldBoriginali/ANY/unsigned/
zero-terminal-vs-exhausted-no-store/fullRAM-SP-callee/guardieJSON-source.39input
protetti invariati; nessunprova vecchia ripetuta. E36immutato, pubblicazione/
runtime/lifecycle/scena/nuoviLinux-USA dafare.
[Contratto](../rom_data_gameplay.md#progression-selection-gates-source-contract).

**ROMDATA-02-GAMEPLAY-PROGRESSION-SELECTION-GATES-DATA / ROMDATA-04-GAMEPLAY-PROGRESSION-SELECTION-GATES-CONSUMPTION completati EUR/Wine:**
e37-d36aggiunge progression-selection-gates.json/gameplay.progression-selection-gates,
32file/31risorse/all30precedenti/e36conservati/USAe8-d7. DueWine73517passano
pubblicazione/retention/guardiecoherenti/binding-source-role-path-hash/transazioni/
Load-Undo-backup-restart e18fullD2390native-interprete/raw1024Balterati/stati-clock
identici. Compiledselector15/min01282/min02505/min04462/ANY1635/flag162; all64mask/
tuttibyte-confini-mirror-BIOS/row0raw16+gapraw2/SWI32typedciascuno/oldgate2-raw16-
new2/DMA2typed2raw. Primo48254compileE0061fixturebudget32argcorretto,77778senza
avvisi; nessunWinefailure. Fonte138+80chiusa/nonripetuta, helperstesseistruzioni/
clock/ABI. ProduzioneCLI/scenaordinaria/nuoviLinux-USA-Windowsnativo dafare.
[Contratto](../rom_data_gameplay.md#managed-and-direct-progression-selection-gates).

**ROMDATA-05-GAMEPLAY-PROGRESSION-SELECTION-GATES-SCENE-CLI completato EUR/Wine:**
produzione34174senzaavvisi/canonicalripristinato; CLI43381create/riuso-no-write/
vecchiacache/cwdestraneo passa,34file hash-mtime invariati. Primo400Yes-fee0newB,
negativochiuso/nofullpair. Primo1000checkpointday18dispatchcompletion CLI9298
positivo277B(selector1/min0126/min0246/min0442/ANY152/flag10), mask67108863:
prim26recordosservati. UnicoWine3138fullpair1000stati-clock-immagini-OAM-dispatch-
audio identici; immagineSigmund/MadAlchemistsuccess/skill08-02-11-04-02-08-42-02
ispezionata. NessunGUI/fixture/payloadedit/savepersonale nuovo. Nuova prova e37,
fonte138+80/native18/e36/delta8/thresholdclass1chiusi/nonripetuti. NuoviLinux-USA-
Windowsnativo/all64ordinary/semanticadisplay dafare; niente performance/spazio.
[Prova](../rom_data_gameplay.md#ordinary-progression-selection-dispatch-completion).

**ROMDATA-01-GAMEPLAY-AGGREGATE-FLAG-KEYS-ROOTS / ROMDATA-02-GAMEPLAY-AGGREGATE-FLAG-KEYS-SOURCE-ORACLE completati EUR/Wine sorgente:**
602B6e9a5c..6e9cb6,300slotu16+terminatoreffff, nextrootindipendente29AF4esattoend.
Duecontatori completi D22FCglobale/13FB1Ccontesto mantengono(key+2ff)u16,
ordine/duplicati/earlyffff, liveflag36ORlinked017fextra1/returnu16. Modello strict
lossless e unicoWine38100passano32reali+26sintetici, tutti602B osservati,
isolamentocontesto/overflow/duplicati/terminazioneprecoce/shortcircuit/readonly
fullEWRAM-SP-callee-stack/JSON-ID-order-offset-source guards. Build5092senzaavvisi,
nessun errore/ripetizioneprova vecchia. E37invariato; preparazione/consumo/
lifecycle/scena e nuoviLinux-USA-Windowsnativo dafare; altri4readerUIstatic-only.
[Contratto](../rom_data_gameplay.md#aggregate-flag-keys-source-contract).

**ROMDATA-02-GAMEPLAY-AGGREGATE-FLAG-KEYS-DATA / ROMDATA-04-GAMEPLAY-AGGREGATE-FLAG-KEYS-CONSUMPTION completati EUR/Wine:**
e38-d37aggiunge aggregate-flag-keys.json/gameplay.aggregate-flag-keys,33file/
32risorse/all31precedenti/e37conservati/USAe8-d7. Windows76657senzaavvisi e due
Wine87184passano pubblicazione/retention/coherentkey-duplicate-earlyffff-terminal-
binding-source-role-path-hash-coverage/transactionalowner/Load-Undo-backup-restart;
16fullcounterglobale-contesto originali/interprete/native con raw602Balterati
hanno stati-clockidentici. Compiledkey9540/terminal30; all602B/300slotmask5word/
confini-mirror-BIOS/preceding8-following4raw/SWI32typed ciascuno+terminal2typed2raw/
wordterminal2typed2raw/DMA2typed2raw passano. Fonte32+26chiusa/nonripetuta,
istruzioni-contesto-predicati-wrapping-clock-ABI originali. CLI/scenaordinaria/
nuoviLinux-USA-Windowsnativo dafare, nessunmisuraperformance/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-aggregate-flag-keys).

**ROMDATA-05-GAMEPLAY-AGGREGATE-FLAG-KEYS-SCENE-CLI completato EUR/Wine:**
produzione59636senzaavvisi/canonicalripristinato; CLI44401crea/riusa-no-write/
vecchiacache/cwdestraneo passa,35filehash-mtime protetti. Primo1000dispatchday18
positivo1204sourceB(1200key/4terminal)/all300slotmask(5wordultimi44bit). Riletture
incluse, nessun1204byte distinti o entrambicounterordinary inferiti. UnicoWine11196
fullpair1000stati-clock-immagini-OAM-dispatch-tuttiaudio identici con stessi
contatori; finalSigmund/MadAlchemistsuccess/skills08-02-11-04-02-08-42-02ispezionato.
Stato/inputesistenti, nessunGUI/fixture/payloadedit/savepersonale nuovo. Nuova
prova solo e38; fonte32+26/native16/e37sameinput chiusi/nonripetuti. NuoviLinux-
USA-Windowsnativo/altrireaderUI/entrambicounterordinary/restTODO dafare; nessuna
misuraperformance/spazio o AndroidPort.
[Prova](../rom_data_gameplay.md#ordinary-aggregate-keys-dispatch-completion).

**ROMDATA-01-GAMEPLAY-ADJACENT-HALFWORD-LOOKUP-ROOTS parziale, selezione chiusa:**
16riferimenti verificati a6e9cb6 e nextroot6e9d8a delimitano212B candidati,
ma non provano dominio106halfword. Indici fissi64/67/liste16..20 e21,22,4,23
verificati; produttori RAM/UI ancora da dimostrare. Nessun modello da soli
valori o confine successivo; riaprire solo con prova di dominio/ownership.

**ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE96-ROOTS completato staticamente:**
D0744 seleziona cinque record96B/ID2..6 a6e85bc..6e879c,480B; root biased prezzo
D26AC e nextroot indipendente concordano. Ipotesi30record respinta: D2728conta
30righeRAM per l'argomento interno0..30. Modello sorgente strict/lossless con
64Bopachi/u16markup40-41/30discount42-5f, count0 alias byte41. D1F30 originale
usa un'altra tabella70B/property36, non items.json.
**ROMDATA-02-GAMEPLAY-TRADE-PARAMETERS-SOURCE-ORACLE completato EUR/Wine:**
Windows73640senzaavvisi/unicoWine52757passano499reali+160sorgentesintetica+
15operandiaritmeticiiniettati separati; all160Bcampo prezzo osservati/320Bopachi
solo roundtrip/tutti480Blossless. Originali D0744/D26AC/D1F30/146474/D2728,
low8-nulloutside/zero-skip/alias41/count0..30/unsigneddivide/wrap32/fullEWRAMreadonly/
SP-callee-stack/strictJSON-ID-order-offset-source passano. 72810istruzioni/184PC
solo coverage,146file protetti invariati. Pubblicazione/consumo/lifecycle/scena
e nuoviLinux-USA-Windowsnativo dafare; prove precedenti chiuse/nonripetute.
EUR e38-d37 invariato, nessuna misura performance/spazio.
[Contratto](../rom_data_gameplay.md#trade-parameters-source-contract).

**ROMDATA-02-GAMEPLAY-TRADE-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-TRADE-PARAMETERS-CONSUMPTION completati EUR/Wine:**
e39-d38/trade-parameters.json/gameplay.trade-parameters/34file/33risorse,
all32precedenti/e38conservati/USAe8-d7. Windows21694senzaavvisi e dueWine20164
passano pubblicazione-failure/riuso-riparo-spostamento/retention/20coherentedits/
missingfile-owner-transaction/Load-Undo-backup-restart/all480B/mask31. 28percorsi
originali/interprete/native/raw480Balterati hanno stati-clock identici; compiled
opaque/markup-low/shared41/discount0-5-10-10, opachi prezzo non inferiti. Tutti
byte/field-recordedges/mirror-BIOS/gap222raw/wordtail2typed2raw/SWI32typed each+
tail2typed2raw+oldselection2/raw222/new2/DMA2typed2raw passano. Fonte499+160+15
chiusa/nonripetuta; istruzioni/getter/liveflag-count/clock/ABI originali.
CLI/scenaordinaria/nuoviLinux-USA-Windowsnativo dafare, nessuna misura prestazioni/spazio.
[Contratto](../rom_data_gameplay.md#managed-and-direct-trade-parameters).

**ROMDATA-05-GAMEPLAY-TRADE-PARAMETERS-SCENE-CLI completato EUR/Wine:**
produzione46740senzaavvisi/canonicalripristinato, CLI17191crea/riusa-no-write/
vecchiacache/cwdestraneo passa,36filemanaged hash-mtime protetti. Primo400normale
checkpointpre-Yes/threshold-accept positivo14B(markup-low7/shared417),mask1ID2;
rilettureincluse, nessun14Bdistinti. UnicoWine89539full400pair: tutti stati-clock-
immagini-OAM-dispatch/mixed-PSG-FIFO-HLEaudio identici, stessicontatori. FinalPublist/
7109fondi ispezionato, nessunGUI/fixture/payloadedit/savepersonale nuovo. Prova
nuovae39, fonte499+160+15/native28/managed/e34threshold400/e38chiusi/nonripetuti.
NuoviLinux-USA-Windowsnativo/altricaller/discounttail-opachi-tutti5ordinary dafare;
nessuna misura prestazioni/spazio o AndroidPort.
[Prova](../rom_data_gameplay.md#ordinary-trade-parameter-fee-acceptance).

**ROMDATA-01-GAMEPLAY-ADJACENT-STRIDE7-ROOTS completato staticamente:**
11literalroot6e879c, nextindipendente6e87fe e enumerazioneitem362..375/subtract362
fissano14record7B/98B, nessunoverlape39. Nove coefficienti5bit/cap8bit/3bitopachi
roundtrip con getter indipendenti. CFE28property0..19/index low8; CFDA8cap usa
full(itemu16−362)u16, non low8. CurrentRAM/quantity/capclamp originali; UIcap>99
solo statico, arbitrarialtriindici raw/nonclaimed. Modello/oracolosorgente/
managed/runtime/scena/nuoviLinux-USA-Windowsnativo dafare, e39invariato.
[Contratto](../rom_data_gameplay.md#item-effect-parameter-static-contract).

**ROMDATA-02-GAMEPLAY-ITEM-EFFECT-PARAMETERS-SOURCE-ORACLE completato EUR/Wine:**
strict14record98B/9coeff5bit-cap8-opaque3/ID-item-order-offset-array-bitwidth/
itemowner376/source guards passano. Windows94173senzaavvisi; unicoWine94185
originalCFE28/CFDA8 passa540reali+4sondecaprawfuoriowner+166sinteticiisolati.
Tutti98B letti/roundtrip; low8getter/invalidpropertyearlyzero/fullu16capindex
vsgetterlow8/livecurrent/quantityu8/capfit-zero-remaining/returnu16/readonlyfullRAM/
SP-callee-stack passano. 31740istruzioni/156PCsolo coverage,258file protetti.
Sonde capfuoriowner non sono nuovi dati typed/fallback. Nessun vecchio itemoracle
14371/e39ripetuto; e39-d38/34file/33risorse/USAe8-d7invariati. Pubblicazione/
consumo/lifecycle/scena/nuoviLinux-USA-Windowsnativo/fullUIwrapper dafare,
nessuna misura prestazioni/spazio.
[Prova](../rom_data_gameplay.md#item-effect-parameter-source-model-and-original-oracle).

**ROMDATA-02-GAMEPLAY-ITEM-EFFECT-PARAMETERS-DATA / ROMDATA-04-GAMEPLAY-ITEM-EFFECT-PARAMETERS-CONSUMPTION completati EUR/Wine:**
e40-d39/item-effect-parameters.json/gameplay.item-effect-parameters,35file/
34risorse/all33precedenti/e39conservati/USAe8-d7, owner items reale. Windows93110
senzaavvisi e dueWine21177passano pubblicazione-failure/riuso-riparo-spostamento/
retention/22coherentedits/bitwidth-ID-item-dependency-source-role-path-hash-coverage/
missingfile-transactionalowner/Load-Undo-backup-restart/all98B/mask16383. 50percorsi
originali-interprete-native/raw98Balterati hanno stati-clock identici; compiled
coefficient/shared05/shared06bytes36-37-23. Tutti7Brecord-fieldedges/mirror-BIOS/
following4raw/wordtail2typed2raw/SWI32typed each+tail2typed2raw+trade2-itemeffect2
crossing/DMA2typed2raw passano. Fonte540+4+166chiusa/nonripetuta, istruzioni/
indexwidths/rawcapalias/liveRAM/clock/ABI originali. CLI/ordinary/nuoviLinux-USA-
Windowsnativo/fullUI dafare. DS22bancapopolata èUSA-only, non prova EUR.
[Contratto](../rom_data_gameplay.md#managed-and-direct-item-effect-parameters).

**ROMDATA-05-GAMEPLAY-ITEM-EFFECT-PARAMETERS-SCENE-CLI completato EUR/Wine:**
produzione4620senzaavvisi/canonicalripristinato; CLI59404crea/riusa-no-write37file
hash-mtimeprotetti e50707vecchiacache/cwdestraneo passano. Nuovo checkpointEUR
IN13con soli inputguest: CyrilBank/List/GoblinLv4/Play, sourceheader-hashvalidi;
nessunRAM/statepayloadedit o savepersonale. Primo400CLIpositivo3B(coefficiente1/
shared051/shared061),mask1record0. UnicoWine5779full400pair con tutti stati-clock-
immagini-OAM-dispatch/mixed-PSG-FIFO-HLEaudio identici/stessicontatori. Replay
misto native/interprete, nessunall-native/performanceclaim. Vecchieprovesorgente/
native/e39chiuse/nonripetute. NavigazioneBaguba/Sprohm/coordinateipotizzaterespinta
conservata; coordCyril120,216da statoaccettatogiàesistente, raggiunta con impulsi
brevi. FinalCLIbankbackground ispezionato ma pannelli separateUIassenti come
limiteprecedente: nessunfood-count/confirmclaim. Altri14item/20monstri/cap>99UI/
nuoviLinux-USA-Windowsnativo dafare; restoTODO aperto, niente performance/spazio.
[Prova](../rom_data_gameplay.md#ordinary-item-effect-parameters-in-the-european-monster-bank).

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


**ROMDATA-01-AUDIO-TYPED-CATALOG-ROOTS completato per le radici iniziali EUR:**
M4A originale/live verificato senza nuovo replay;3player36B, directory640voci/
5120B,455header distinti/10080B,186alias header vuoto,1610radici tracce e4banchi.
Roundtrip indipendente di tutti i campi; nessun guard640/128toni inventato,
alias header150940/WaveData150944 esplicito. Banchi/campioni/bytecode completi
restano aperti. [Contratto](../rom_data_audio.md).

**ROMDATA-02-AUDIO-M4A-DIRECTORY-SOURCE-ORACLE completato EUR/Wine:** modello
strict3player/640voci/455header/457range/15236B, ID header minimo songID e alias
espliciti; fonte esatta/roundtrip/17guard JSON-shape-link-source passano. Build
Windows finale senza warning; unico oracolo Wine passa640selezioni+9input mirati,
1lookup raw640 fermato prima del deref header null e6varianti sintetiche non
pubblicate. EW-RAM completa/IW-RAM fuori scratch stack/tracce/SP/callee uguali
alle attese indipendenti,13477B effettivamente letti/1759B solo conservati.
Primo compileE0283 e prima attesa errata word20 tracce conservati; correzione
attesa zero provata dal backedge1492e6. Modello invariato; nessuna accettazione
ordinaria/native/clock rivendicata. Dataset e40-d39/USAe8-d7 invariati.

**ROMDATA-02-AUDIO-M4A-DIRECTORY-DATA / ROMDATA-04-AUDIO-M4A-DIRECTORY-CONSUMPTION
completati EUR/Wine dichiarato:** e41-d40aggiunge audio-directory.json/risorsa
audio.m4a-directory,36file/35risorse/34decoded precedenti ed e40conservati,
USAe8-d7. Proprietario tipizzato457span/15236Bsenza overlay raw, contatori
quattro categorie e maschere3player/640song/455header. BuildWindows5021senza
warning e due controlliWine51686passano pubblicazione/25guard coerenti/riuso-
riparazione-move-retention/Load-Undo-backup-Restart senza cartella,44percorsi
originale-interprete-native con raw15236Balterati/stati-clock completi uguali.
Ogni byte/span/bordo/width/mirror/BIOS/SWI-DMA passa, inclusi2typed2raw al bordo
dummy-wave e CPUSet2directory+8header+2raw. Mixer/sequencer/RAM/ABI restano;
fonte649+1+6ed e40non ripetuti, nessun nuovo errore build/Wine.

**ROMDATA-05-AUDIO-M4A-DIRECTORY-SCENE-CLI completato EUR/Wine dichiarato:**
production78108senza warning/canonico ripristinato; CLI14580crea/riusa e41senza
riscrivere38file,27876riusa vecchia cache da cwd estranea. Vecchi checkpoint
EURGoblinPlay/input400senza nuova navigazione/fixture/RAM edit:179nuoviB
(36directory/79header/64puntatori),3song/header/0configplayer letti. Unico
Wine91327fullpair400stati-clock-immagini-OAM-dispatch-mixedPSGFIFOHLEaudio
identici/stessi179B; esecuzione mista, non all-native. Immagine568×160sfondo
Bank ispezionata; pannelli omessi da separateUI serializzata come già noto,
nessuna nuova conferma/conteggio cibo rivendicata. Fonte649+1+6/native44/
e40scene chiusi/non ripetuti; nessun errore build/Wine in questa sottofase.

**ROMDATA-01-AUDIO-M4A-TONE-SEQUENCE-PRODUCERS completato per i domini prodotti
dichiarati:** grafo1610tracce/177047B/805range/zeroerrori,36comandi copia+patch
uguali a RAM conservata.390coppie banco-programma,429tone/5148B,348linkkeymap/
347B/1alias,277headerPCM/1931528payload nominali;276endpoint extra interpolati/
1fixed8entro lunghezza.2fontiCGB16Bstatiche. Piano736range/1941759nuoviBesclude
4Bgià e41. Build32264senza warning/unicoWine8457passa390BD/348map/276+1endpoint;
note/kernel sono checkpoint parziali, non mixer o scene completi, non tutti i
PCM letti. Sweep128keymap respinto come ownership: invade tabelle adiacenti.
Parser/model/nuova pubblicazione-runtime da fare; e41invariato.
[Contratto](../rom_data_audio.md#produced-tone-keymap-and-pcm-source-contracts).

**ROMDATA-02-AUDIO-M4A-TONE-MODEL completato EUR/Wine (2026-10-24):** parser
condiviso bounded riproduce1610grafi/147424stati/390programmi/2065coppie-key/
805range177047B; modello strict429tone/347mapbyte-348link/277PCM/2CGB,
736range1941759Bexact roundtrip. PCM signedi8+endpoint opzionale/CGB32nibble,
ID da prime referenze/alias canonici, nessun mixer riscritto. Build61632senza
warning/unicoWine19316passa domini/tutti i campi-hash indipendenti/JSON/source
roundtrip/28guard e fonte coerente/alias edit con stessi nuovi span respinto
dalla dipendenza directory reale.2nuove copie CGBoriginali passano32BwaveRAM/
RAM/SP in checkpoint interno, non mixer completo.492originali/evidenze protetti;
nessun390BD/348map/277endpoint/e41replay ripetuto, e41-d40/USAe8-d7invariati.

**ROMDATA-02-AUDIO-M4A-TONE-DATA / ROMDATA-04-AUDIO-M4A-TONE-CONSUMPTION
completati EUR/Wine dichiarato (2026-10-24):** e42-d41/37file/36risorse, tutti
35file precedenti/e41conservati/USAe8-d7.736source1941759Bdisgiunti/dipendenza
reale directory4B. DueWine84702/3937passano26guard/pubblicazione-riuso-ripara-
sposta/lifecycle/tutti i byte e mask;25percorsi original/interpreter/native con
rawalterato/fullstate-clock,736bordi/mirror/BIOS/SWI-DMA/alias2+2/rawfixed8.
Entrambi i callsite MP2Khook/tick leggono4428headerB+35458PCM Btipizzati da277onde,
audio nonzero bitexact e stato/clock completi uguali.276endpoint hostdiretti
separati dal clampHLE esistente; non tutti i PCM letti dal mixer. Checkpoint
interni note/header/ARM/CGBnon equivalgono a mixer originale intero. Buildfinale
49176senza warning; due errori di preparazione test corretti e conservati,
nessun cambio modello/produzione; nessun controllo già chiuso riaperto.

**ROMDATA-05-AUDIO-M4A-TONE-SCENE-CLI completato EUR/Wine (2026-10-24):**
production16173senza warning/canonico ripristinato, CLI79505crea/riusa senza
scrivere39file/vecchia cache/cwdestraneo/400bankPlay usa516956guestB(247tone/
5map/11362header/504912PCM/430endpoint),0CGB. DefaultCLIshadowoff esplicito:
unicoWine36911pair loarma dopo load e passa400stati-clock/immagini/OAM/dispatch/
audio mixed-PSG-FIFO-HLE con stessi guestB e3093208hostB(1056header/3092152PCM).
Entrambi gli usi obbligatori positivi; snapshot/input esistenti invariati,
nessuna navigazione/fixture nuova. Capturebankbackground568x160ispezionata;
separateUIomette pannelli come già noto, nessuna nuova conferma foodcount.

**ROMDATA-02-AUDIO-M4A-SEQUENCE-MODEL completato EUR/Wine (2026-10-24):**
105253istruzioni fisiche/unica interpretazione/nessun overlap,1610root/805range/
177047Bprecedenti invariati. Enum strict per wait/note/parameter/pattern/end/
return/jump/endtie/extended, ID canonici per salti/pattern/tracce; opcode esplicito
vs ereditato e operandi conservati senza hostsequencer/RAM congelata.49clockB/
36commandword144B/soloCD8-9dispatch8B aggiungono3span201B:808range177248Bdisgiunti
dalle36risorse e42. Buildfinale76630senza warning/unicoWine4313passa tutti i
campi-hash/fonti/JSON/27guard/metadata ereditati a byte invariati/directory reale.
404nuove chiamate/checkpoint originali:161prefixdispatch/158noteargs/16jump-
pattern-return/12parameter/4CD8-9/49wait/3CE/1copy36words; RAM/cursori/SP/stack
passano. Non fullsequencer/mixer/stati-clock/native/ordinary. Prima Wine29418
fallita su due PCtrascritti+2 nelle asserzioni driver, corretti dai byteROM;
log/exe/source conservati.598input/evidenze protetti; vecchie prove non ripetute.

**ROMDATA-02-AUDIO-M4A-SEQUENCE-DATA / ROMDATA-04-AUDIO-M4A-SEQUENCE-CONSUMPTION
completati EUR/Wine (2026-10-24):** e43-d42aggiunge audio-sequences.json/
audio.m4a-sequences,38file37risorse/tutti36decoded precedenti/e42conservati/
USAe8-d7. Dipendenze directory+tone reali/390domini programma-tasto da147424stati,
metadata ereditati/banchi a stessi byte respinti. DueWine49125/build11482senza
warning passano pubblicazione/25guard/lifecycle/tutti177248B/mask105253istruzioni-
49clock-36command-2ext e35percorsi original/interpreter/native con rawalterato/
fullstate-clock. Bordi808/mirror/BIOS/SWI-DMA/fallbackraw/tone2+sequence2guest-host/
runningstatusRAM vivo passano. Generic host non è hostsequencer; checkpoint
interni/lettori stretti non fullsequencer/mixerABI. Nessun nuovo errore build/Wine;
vecchio404/e42prove non ripetuti,627vecchi file protetti invariati.

**ROMDATA-05-AUDIO-M4A-SEQUENCE-SCENE-CLI completato EUR/Wine (2026-10-24):**
production48965senza warning/canonico ripristinato, CLI48325crea/riusa senza
riscrivere40file/vecchia cache/cwdestraneo/400bankPlay legge819nuoviB(366opcode/
262operand/191clock),185record sorgente osservati incluso lookahead, non conteggio
istruzioni eseguite. UnicoWine29123pair passa400stati-clock/immagini/OAM/dispatch/
audio mixed-PSG-FIFO-HLE identici/stessi819B.0command/extended/generic-host in
questa scena, non inizializzazione ordinaria. Checkpoint/input esistenti invariati,
nessuna navigazione/fixture nuova. Capture568x160sfondoBankispezionata, separateUI
omette pannelli come noto, nessuna nuova conferma/foodcount. Fonte404/native35/
e42prove non ripetuti; nessun nuovo errore build/Wine in questa sottofase.

**ROMDATA-01-AUDIO-M4A-DRIVER-LUTS / ROMDATA-02-AUDIO-M4A-DRIVER-MODEL completati
EUR/Wine dichiarato (2026-10-24):**7tabelle370b20..370d04/484Bdisgiunti e43:
180PCMkeycode/12u32base/12u16samplecount/132CGBkeycode/12i16base/60noise/16volume.
Nibbles decomposti/segno/bit opachi/alias preservati. PCMkeylow8clamp178+endpoint179/
frac255 e UMULLunsigned; CGBtonalkey<=35→0frac0/meno36clamp130+endpoint131/signedASR;
noise type4/key<=20→0/meno21clamp59. Samplefreq nibble0..15senza clamp12:0alias
ultimabasePCM,13..15aliasCGB; SoundModepubblicoskipzero. VolumerawRAM16/64delegano
clockprecedente/255raw, nessun guardguest inventato. Build87853senza warning/
unicoWine60589passa1138nuove chiamate originali (541PCM/300tonal/259noise fullreturn,
18rate/19volume prefix/1zeromode fullreturn), tutti484B alle loadsite osservate,
RAM/SP/callee/stack, fonte+JSON/15guard/fonte coerente/ownerconfine sequenze.
Non fullinit/mixer/stati-clock/native/ordinary; e43-d42/USAe8-d7invariati.
879input/evidenze protetti; nessuna vecchia accettazione ripetuta/errorebuild-Wine.
Ricognizione finale corregge ipotesi cap15globale: ramo bilanciato149640salta
clamp14964c. Ulteriore Wine27372/build59773esegue solo8nuovi return gain e passa
128/128→16,255/128→23,255/255→31; ramo unilaterale clamp15, RAM/SP/callee/stack8B
esatti. Nessun byte nuovo/cambio modello/1138ripetuto; dominio ordinario globale16
non provato, aliases da preservare. Prova/correzione produttore chiusa.

**ROMDATA-02-AUDIO-M4A-DRIVER-DATA / ROMDATA-04-AUDIO-M4A-DRIVER-CONSUMPTION
completati EUR/Wine (2026-10-24):** e44-d43pubblica audio-driver-tables.json/
audio.m4a-driver-tables,39file38risorse/tutti37decoded precedenti/e43conservati,
USAe8-d7.7span484B/dipendenza sequenze reale. Build70748senza warning/Wine51847
due controlli ridotti passano19guard/pubblicazione-riuso-riparazione-spostamento/
retention/lifecycle/tutti484B/424entrybits e28percorsi original-interpreter-native
con rawalterato/stati-clock esatti, UMULLThumb-ARM/categorie compilate tuttepositive
[10,42,6,18,24,4,2]. Alias sample0/13..15 e volume16/31/64ownerclock/255raw,
tutti7bordi/width/mirror/BIOS/SWI/DMA/guest-host crossing passano.844istruzioni/
150PC/70dispatch coverageonly; non fullinit/mixerABI.917protetti+exe immutati;
fonte1138/gain8 e vecchie prove chiuse non ripetute. Nessun errorebuild/Wine.

**ROMDATA-05-AUDIO-M4A-DRIVER-SCENE-CLI completato EUR/Wine (2026-10-24):**
Build85643senza warning/canonicalripristinato; Wine30529CLIrealecrea/riusa senza
scrivere41filegestiti/vecchiocache/unrelatedcwd400bancoEUR.756letturebyteaggregate
[132PCMkey,528PCMbase,0samplecount,32CGBkey,64CGBbase,0noise,0volume];67entryfisiche
[31,12,0,16,8,0,0], host0. UnicoWine21679pair400stati-clock/immagini-OAM/dispatch/
mixedPSGFIFOHLEaudio esatti, stesso uso positivo. Non byte distinti/performance/
allnative. Immagine568x160background ispezionata, UIseparata omessa come noto,
nessunfoodcount/conferma nuovo. Stato/input esistentiimmutati/nessuna GUI-fixture-
RAMalterazione. Fonte1138/gain8/native28/vecchie prove non ripetuti, nessun nuovo
errorebuild/Wine/livejob.

**ROMDATA-01-AUDIO-FFTA-WRAPPER-SOURCES completato, riclassificato (2026-10-24):**
370eb0è vettore eseguibile di helperIWRAMclear/copy/fill, non audio; b94690è
palette-effect dispatch grafico. Verificati7literal/5produttorifissi1/2/4/8/16,
lettore state1/u16nonmascherato/stride4, proiezione17word68Bcon12buchinull/disgiunta
38owner e44. Nessun cap16globale/totalextent dedotto; indici superiori raw.
Tre prefissi32Bnel vecchio statoEUR corrispondono univocamente a ROMcode, senza
nuovo gioco/fixture/build/prove vecchie ripetute. Catalogo audio invariato.

**ROMDATA-02-PALETTE-EFFECT-DISPATCH-MODEL completato EUR/Wine (2026-10-24):**
Modello17slot68B/kindu16canonici/5targetThumb come offsetcodice/12null. Build71666
senza warning/unicoWine22360passa campi-hash/JSON-source/12guard+wrongROM e31prefix
originali (23lookup/3stategate/5producerstore), tutti68B osservati a0814ac70.
Highwordtroncato/liveu16/null/raw17-31-255-65535/RAM-SP-callee/write2B esatti.
134istruzioni18PC coverageonly, non fullcallback/native/stateclock/ordinary.
1216protetti immutati/nessun errorebuild-Wine/vecchie prove ripetute; e44invariato.

**ROMDATA-02-PALETTE-EFFECT-DISPATCH-DATA / ROMDATA-04-PALETTE-EFFECT-DISPATCH-CONSUMPTION
completati EUR/Wine (2026-10-24):** e45-d44pubblica40file39risorse/tutti38decoded
precedenti/e44conservati/USAe8-d7. Build67564senza warning/Wine46138due checkridotti
passano12guard/pubblicazione-retention-lifecycle/tutti68B/17slotmask e10prefix
original/interpreter/native con rawalterato/stati-clock esatti,24callback/4null
compilati. Liveu16kind/troncamento/null/stategate/raw17-65535/tutti bordi-width-
mirror/BIOS/SWI/DMA/2typed-2raw passano.48istruzioni8PC10dispatch coverageonly,
non wholecallback/allocatorABI.1226protetti+exe immutati; fonte31/e44vecchie prove
non ripetute/nessun errorebuild-Wine.

**ROMDATA-05-PALETTE-EFFECT-DISPATCH-SCENE-CLI / SCENE-TRANSITION completati
EUR/Wine dichiarato (2026-10-24):** build95401senza warning/canonicalripristinato,
Wine56076CLIrealecrea/riusa senza scrivere42file/oldcache/unrelatedcwd. Boot240
neutro0uso chiuso/no fullpair. Nuovo inputnormale confermaEnglish A240/release252,
poi neutral1000: Wine60304CLIpositivo1486letture/5944bytecallbackaggregate/null0/
slotmask2(kind1solo). UnicoWine35467pair1000passa tutti stati-clock/immagini-OAM/
dispatch/mixedPSGFIFOHLEaudio, stesso uso. Non5944byte distinti/performance/allnative.
Immagine568x160intro ProductDevelopmentDivision-4/snow ispezionata. Altri4callback
/nulltargeted-only, non wholeABI.10snapshotprotetti ispezionati staticamentecon
lista vuota, nessun replay/nuovi snapshot/RAM-statepayload-savealterati. Fonte31/
native10/vecchieprove chiuse non ripetute; nessun nuovo errorebuild-Wine/livejob.

**ROMDATA-01-PALETTE-EFFECT-PROGRAM-PRODUCERS parziale (2026-10-24):**
Chiuso il percorso iniziale main080d7d24→0814b9f8→callback4. Taskpalette reale
080d7fac/public080d8214-080d8234 distinto dal vicino task080d79ac/79d0 (248candidati,
non palette).308BLdiretti;285espressioniargomento esatte/179programmiROM.3listecon
conteggi3/3/2 e8row8B aggiungono5programmi;2copieLDM/STM3word aggiungono1. BridgeR4
conservato riusa rootgiànota. Totale185programmi/1000comandi/16135Bcandidate in124
range+64liste/24stack=16223B disgiunti39owner e45. Header6B(countu8),comandi6B/
offsetrelativou16, copycolori diretto vs queued e min1sequenza documentati;
lunghezzeparametri ancora da confrontareconlettori originali Wine.
20produttoriindicizzati ora risolti come proiezioni bounded da taskID/copiemessaggi/
storecostanti/loopcontati:97celle/388B,45programmi aggiunti. Totale230programmi/
1135comandi/18221programB+476producerB=18697B/165range, disgiunti39owner.185record
precedenti identici;39bcc4solo2puntatori, coda null/scalari esclusa. Nessun nuovo
capRAM/domino globale/estensione fisica completa. Nessun modello/runtime/e46/
guest/build/test; e45/USAe8-d7invariati. Prove chiuse nonripetute.

**ROMDATA-01-PALETTE-EFFECT-INDEXED-PRODUCER-DOMAINS completato per le20proiezioni**
(2026-10-24). Fonte task080d7018usa segmenti distinti ID1..341eID342+;
ID354/355non derivano da un'unica base estrapolata. Prove/fingerprint espliciti
in indexed-producer-proofs; nuovo indexed-program-discovery autorevole230record.

**ROMDATA-02-PALETTE-EFFECT-PROGRAM-MODEL completato per EUR/Wine (2026-10-24):**
Modello lossless230programmi1135comandi/25tabelle,165range18697B, fingerprint120,
IDcanonici/puntatorinull/alias/delay/reserved e parametri conservati. Windows23752
senzaavvisi; unicoWine96989passa tutti18221programB/18681Btotali osservati sui
lettori originali,16Breservedlista conservati/nonletti;97celle/8row/2stack.6limiti
copywrap-clamp/3mincount-period/1direction255 e16guardshape-JSON-source passano.
Non fullallocator/callback/native/stato-clock/ordinary.1645artefatti+nuovotestexe
immutati; e45-d44/USAe8-d7 invariati, nessuna misura performance.

**ROMDATA-03-PALETTE-PROGRAM-RUNTIME integrazione/targeted completati EUR/Wine**
(2026-10-24): e46-d45/41file40risorse/all39decodedprecedenti/e45preservati. Binding
165/18697B dai campitipizzati; Header/command/parameter/producer e IDcanonici,
nessun guardindice RAM/sequencer/save-nativeABI nuovo. Wine78294managedpassa
12guard/ciclodivita/tutti18697B/230program25table. Nativeiniziale/diagnostico
fallivano per endpoint dentrograph concatenato; guardneltest correggeilconfine,
backendproduzione invariato. Native-only33298passa24prefissi stato-clock completi,
APIgenerate[16header52command66parameter12producer]B, rawcommand65535/offset65534/
index2/BIOS-SWI-DMA/mirror/bordi. Fonte/managed giàchiusi nonripetuti.

**ROMDATA-04-PALETTE-PROGRAM-CLI completato EUR/Wine (2026-10-24):** Windows19390
senzaavvisi/canonico ripristinato; Wine60759create/reuse-no-write43filemanaged e
cache vecchia da cwddiversa passano. NormalEnglishconfirm1000legge0nuovi byte:
locator negativo chiuso, nessun fullpair/nonripetere. Scansione staticadistinta
nuovotask080d7fac nei10snapshotprotetti worldmap/bank: managerzero/nocallback,
nessun replay/nuovosnapshot. Non ripetere percontesto.

**ROMDATA-05-PALETTE-PROGRAM-SCENE completato EUR/Wine (2026-10-24):**
Montblanc Richiamo/Pecorelle, da checkpoint di conferma ottenuto con input normali
EUR, passa CLI reale e un unico confronto1000frame in modalità240x160: stato
completo/clock/immagini/OAM/dispatch/audio misto-PSG-FIFO-HLE identici.170letture
byte aggregate=32header+90command+48parameter,4programmi,0producer. Gli altri226
programmi e le tabelle producer mantengono la prova targeted. Il primo confronto
aveva completato tutti1000frame e l'assert della nuova famiglia, poi falliva solo
il vecchio requisito generico hostOBJdimensions. Il test accetta ora letture
bus o host tipizzate; requisito header/command/parameter positivo invariato.
Ricompilato solo il test Windows senza avvisi e ripetuto solo questo confronto,
che passa. Runtime di produzione invariato; nessun nuovo build/retest fonte,
managed/native/CLI create-reuse. Conservati entrambi gli esiti.
Nessuna modifica a RAM/statepayload/save originali. F09 e preparazione sul fiume
restano navigazione privata; i locator senza esecuzione non sono prove positive.
Modello230/sourceWine/20proiezioni/native24/managed/CLI/ordinary sono chiusi;
intro1000negativo e snapshot10non sono nextaction. NuoviLinux/USA/Windowsnativo
ed una nuova scena32:9 restano da fare.

**ROMDATA-01-BLEND-PROGRAM-PRODUCERS completato per le proiezioni dichiarate
EUR (2026-10-24):** task080d77c4/ctor080d79ac-080d79d0controllano BLDCNT,
BLDALPHA, BLDY. Classificate248chiamate dirette:235costanti/2condizionali/
6indicizzate/1inline/1templateRAMmutabile/3RAMdinamiche.160programmi/345comandi,
960Bheader+1380Bcomandi+88Bproducer(22puntatori/6tabelle)=2428B/158range,
disgiunti da tutte40le risorse e46. Header6B(u16control/coeff/count), comando4B
(u16duration,u8first/second). Nessun cap16/guardindice RAM/estensione fisica
completa dedotto. Falso rootcodice080e2ec8respinto: il caller copia10B da39a5f0
in RAM e sovrascrive la durata. Ricostruzione/hash indipendenti,44opcode dei
consumatori e mapping DMAregistri passano staticamente; nessun nuovo guest run,
build o test runtime. Fonte e comportamento Wine restano criteri distinti.
[Contratto](../rom_data_graphics.md#blend-program-producer-inventory).

**ROMDATA-02-BLEND-PROGRAM-DATA completato EUR/Wine (2026-10-24):** modello
strict lossless160programmi/345comandi/6tabelle22puntatori,2428B/158range;
bitfield e riservati conservati, nessun cap16/guardRAM nuovo. Identità EUR esatta,
110opcode e riferimenti canonici autenticati. Windows12341senzaavvisi; unico
Wine94935passa160header/345comandi/22celle/5cateneinstant/templatecopy+RAMoverride/
2header massimi-mode0/6durate/3delay/index65535raw.17guardforma/3fonteeditata/
2JSON/2identitàROM passano.2376B osservati;52byte secondtarget nonalpha timed
restano conservati/hashverificati per questi initial-mode probe, nessuna
irraggiungibilità globale dedotta.52943istruzioni/264PC solo copertura.
[Modello e limiti](../rom_data_graphics.md#typed-blend-program-source-model).

**ROMDATA-03-BLEND-PROGRAM-RUNTIME integrazione/targeted completati EUR/Wine
(2026-10-24):** e47-d46/42file41risorse/all40decodedprecedenti/e46preservati.
158binding/2428B da header-command-producer tipizzati,13pagine; nessun guardRAM/
cap16/sequencer/save-nativeABI nuovo. Due prove Wine ridotte passano12guard/
ciclodivita/tutti2428B/160program6table e20prefissi original-interpreter-native
stato completo/clock, con[34header70command16producer]B daAPI compilate.
TemplateIWRAM/liveRAM/mode2→1/rawcommand65535/index3/precedenteownerpaletteindex5/
width-mirror-BIOS-SWI-DMA/new2raw2/new2old2 passano.1034istruzioni295PC93dispatch
solo copertura. Dopo review, corretto il gate di coerenza dell'insieme estratto:
una sola verifica Wine negativa aggiuntiva passa senza ripetere native/lifecycle.
Build Windows interessati senza avvisi; primo errore d'arità nei test conservato.
Nessuna prova di ABI completa allocator/callback/scheduler oordinary ancora.
[Consumo](../rom_data_graphics.md#managed-blend-programs-and-typed-guest-reads).

**ROMDATA-04-BLEND-PROGRAM-CLI / ROMDATA-05-BLEND-PROGRAM-SCENE completati
EUR/Wine (2026-10-24):** Windows81247production senza avvisi/canonico ripristinato.
Wine46190CLI create/reuse-no-write44filemanaged/cachevecchia/cwddiversa passa;
tutti42file della generazione coincidono col dataset accettato. Scena EUR reale
Montblanc Richiamo/Pecorelle da checkpoint/input conservati:18letture byte
aggregate=10header+8command,1programma,0producer. Unico confronto Wine91145di
1000frame sulla generazione pubblicata dal CLI passa tutti stati completi/clock/
immagini240x160/OAM/dispatch/audio misto-PSG-FIFO-HLE. Altri159programmi e6tabelle
producer mantengono copertura targeted; nuova scena32:9 ancora da fare.
Nessun rebuild test, navigazione/fixture/payload edit o repeat fonte160/inventario
248/e46/native20/lifecycle. USAe8-d7 invariato; nuoviLinux/USA da fare.

**ROMDATA-01-TASK-DESCRIPTOR-SOURCE completato per la proiezione EUR dichiarata
(2026-10-24):**341record primari/20ausiliari/51script,412record8B/3296B/3range,
disgiunti41owner e47. CallbackThumbnullable/contextsizeu16/metadatau16grezzi;
268callback nonzero distinti/3recordnull. Confine39a490provato dal consumatore
grafico indipendente e dalla riga directory kind0/ID413che seleziona39a488.
IDprimario362≡secondario363, ultimo412≡413; nessun guardglobale ai selettori.
59halfword/12wordfingerprint e roundtrip/hash statici passano;53BLcandidati
conservati, CFG incompleti restanoignoti. Ipotesi456record respinta, non riaprire.
[Contratto](../rom_data_tasks.md).

**ROMDATA-02-TASK-DESCRIPTOR-MODEL completato EUR/Wine (2026-10-24):** modello
strict lossless341+20+51record/3296B/3range, ordinale/Thumbnullable/contextsize/
rawmetadata e due formule ID esatte. Windows24937senzaavvisi; unicoWine82576passa
415loadprimari/59secondari/346metadatahalfword/346byte/18low16/9branch e23guard.
3156Bdistinti osservati/140metadataBconservati nonletti inquesteprove; alias e
indici raw0/65535/413-414preservati.7191istruzioni39PCsolocopertura, nonABI completa
allocator/callback/scheduler/native/stato-clock/scenaordinaria. e47-d46/USAe8-d7
invariati; nessuna ripetizione provechiuse/performance. Linux/USA dafare.
[Modello e prove](../rom_data_tasks.md#reduced-wine-source-validation).

**ROMDATA-03-TASK-DESCRIPTOR-RUNTIME completato nei criteri mirati EUR/Wine
(2026-10-24):** e48-d47pubblica task-descriptors.json/engine.task-descriptors,
43file42risorse/all41decodedprecedenti/e47conservati/USAe8-d7invariato.3binding
3296B/pages399-39a, callback/contextsize/metadata tipizzati, selettori/ABI invariati.
Windows72886senzaavvisi/Wine32369duecheck passano13guard/lifecycle/all3296B/
412record3cataloghi e29prefix original/interpreter/native stati-clock completi.
CompiledAPI [44callback22context9metadata]B, alias/raw0-65535-413/414/low16/special
metadata/oldblendowner6B alterato, bordi-width-mirror-BIOS-SWI-DMA/new2raw2 passano.
206istruzioni41PC29dispatch solocopertura, non ABIintera/scenaordinaria. Fonte e
provechiuse nonripetute. [Consumo](../rom_data_tasks.md#managed-task-descriptors-and-typed-guest-reads).

**ROMDATA-04-TASK-DESCRIPTOR-CLI / ROMDATA-05-TASK-DESCRIPTOR-SCENE completati
EUR/Wine (2026-10-24):** Windows87514production senzaavvisi/canonicalripristinato;
Wine38310CLI realecrea/riusa senza scrivere45filemanaged/carica cachevecchia da
cwddiversa; tutti43filegenerazione coincidono coldatasetaccettato. Scena autentica
EURPecorelle1000/classic240x160passa28Baggregate=16callback8context4metadata/
4record2cataloghi. UnicoWine58452pair dallagenerazioneCLIpassa tutti1000stati-clock/
immagini-OAM-dispatch/audio mistoPSGFIFObitwiseHLE. Altri408record/terzocatalogo
mirati solamente; nonABIintera/new32:9. Nessuna ricreazionefixture/navigazione/
modifica RAM-payload-save/ripetizione source-native-lifecycle vecchieprove.
[CLI e scena](../rom_data_tasks.md#production-cli-and-ordinary-scene).

**ROMDATA-01-EFFECT-EVENT-DIRECTORIES / ROMDATA-02-EFFECT-EVENT-DIRECTORY-MODEL
completati per EUR/Wine dichiarato (2026-10-24):**75eventi16B/51selettori4B/
28worddispatch pariThumbsame-state,1516B/4range disgiunti42owner e48. Incremento/
confronto74 e catalogo indipendente3acebc provano1200B; lettore indipendente3ad240
prova204Bselettori senza dedurreestensione dai51task. Corretto candidato128b4c
(literal→tabella128b50), conservata guardia iniziale troppo stretta125394→125398.
19+51riferimenti logici al catalogo task; q0/index65535/ID362/jumpRAM/default
restanooriginali. Windows78776senzaavvisi/unicoWine43580passa77header77delay/
53selector43param/32dispatch-default/19low16-wrap e27guard.1106Bdistinti osservati,
300eventraw+110paramBconservati-hashchecked/nonletti inquesteprove.2582istruzioni/
70PCsolocopertura; noninterpreter-allocator/native/stato-clock/ordinaryABI o prova
reachability124e1c. e48-d47/USAe8-d7invariati, Linux/USA dafare. [Contratto](../rom_data_events.md).

**ROMDATA-03-EFFECT-EVENT-DIRECTORY-RUNTIME completato nei criteri mirati
EUR/Wine (2026-10-24):** e49-d48pubblica44file43risorse/all42decodedprecedenti/
e48conservati/USAe8-d7invariato, dipendenza realeengine.task-descriptors.4binding
1516B/pages124/128/3ac/3ad daeventi/selettori/dispatch pari, senzaROMshadow/guestcap/
sequencer/save-nativeABI nuovo. Windows93869senzaavvisi/Wine91663duecheckfirstPASS:
13guard/lifecycle/all1516B/75event51selector2table e29prefix original/interpreter/
native statieclockcompleti. APIgenerated[24event6selector24dispatch]B; index75/
51-65535/q0/wrap/default/evenThumb, event65535→fontprecedente11B, paletteprecedente
stessapagina4B, bordi-widths-mirrors-BIOS-SWI-DMA/new2raw2 passano.186istruzioni70PC/
29dispatchsolocopertura, nonABIintera/ordinary/reachability. [Consumo](../rom_data_events.md#managed-publication-and-typed-guest-reads).

**ROMDATA-04-EFFECT-EVENT-DIRECTORY-CLI completato EUR/Wine (2026-10-24):**
Windows48886production senzaavvisi/canonicalripristinato; Wine76313CLI realecrea/
riusa senza scrivere46filemanaged/carica cachevecchia da cwddiversa; tutti44file
generazione coincidono coldatasetaccettato e49.

**ROMDATA-05-EFFECT-EVENT-DIRECTORY-SCENE completato EUR/Wine (2026-10-24):** locator autenticiEUR
Pecorelle1000, schoolcoldboot3000 e suspendboot-to-battle8476/classic0 terminano
con0letture nuovafamiglia; chiusi negativi, nessun fullpair richiesto per eventi. Tracciaopcode59/5a→128af4 e letturaautenticata
snapshotPecorelle/F09 conroot20159d4zero restringono ilcontesto; candidatocon
identitàUSA esclusoprima dell'ispezione/replay. Nuova progressione normale EUR
avvio/nome/neve/strada/camera produce19snapshot autentici, roundtrip esatti e
hash input/artefatti verificati; nessuna patch RAM/state/save o modifica originali.
Due nuovi locator limitati3180tutorial/6000turn-transition negativi chiusi,
nessunfullpair. Il successivo stato39420camera/libro dà un nuovo locator1800
Wine59085positivo:2bus read/5B=0event+1selector+4dispatch, unselector/unatable.
UnicoWine91753pair1800stesso stato/input/generazioneCLI e52/cache/testexe passa
tutti stati-clock/immagini central-wide/OAM/dispatch/audio mistoPSGFIFObitwiseHLE,
campioni[3950100,3950100,475200,475200,3950026], primo uso39946/stessi5B/
470hash-mtime guard. Criterio event-OR-selector chiuso con lettura selector;
eventrows0/altri50selector/altratable restano mirati. Nessun rebuild prodotto/
core/prepare/vecchie prove/performance. Linux-USA-Windowsnativo/new32:9/ABIintera/
reachability124e1c non verificati. [Prova](../rom_data_events.md#positive-ordinary-book-transition).

**ROMDATA-01-SCRIPT-COMMAND-DIRECTORY / ROMDATA-02-SCRIPT-COMMAND-DIRECTORY-MODEL
completati per EUR/Wine dichiarato (2026-10-24):** 115 comandi da6B/690B a
3acebc..3ad16e, tre root originali e limite indipendente da due lettori lookup
1263c4/126468. Candidato iniziale116 respinto prima del piano/modello: includeva
la tabella distinta a3ad16e. Callback Thumb BX non-null unici e advanceu16,
nessuna dipendenza asset; disgiunto da43owner e49. Modello strict lossless e
106halfword/6wordfingerprint/hashroundtrip passano. Primo Wine49512 fallisce
aspettativa testADDSr1 invece di r0; corrette solamente due asserzioni, prova
fallita conservata. Windows12070 senzaavvisi e unico filtro Wine67188 corretto
passano117callback/117advance/115BX/4signedcursor/10relative-target/5loop-return,
tutti690B distinti osservati e19guard.2905istruzioni/67PC solo copertura; non
interpreter/native/fullstate-clock/ordinaryABI. e49-d48/USAe8-d7 invariati;
pubblicazione/consumo runtime/CLI/scena positiva completati nelle unità sotto.
[Contratto](../rom_data_scripts.md).

**ROMDATA-03-SCRIPT-COMMAND-DIRECTORY-RUNTIME completato nei criteri mirati
EUR/Wine (2026-10-24):** e50-d49 pubblica45file44risorse/all43decodedprecedenti/
e49conservati/USAe8-d7 invariato. Nessuna dipendenza asset, coerenza source-set
richiede owner precedente.690B/pages3ac-3ad da115comandi tipizzati, senzaROMshadow/
nuovi cap guest/ABI save-native. Windows93228 senzaavvisi; Wine6355duefiltri
passano al primo tentativo13guard/lifecycle/all690B e26prefix original-interpreter-
native stati/clock completi. APIcompiled14advance/28callback B; raw115/255, cursore
signed/wrapu16/targetrelativi/BL-BX/callreturnloop, ownerprecedenti event4B/selector4B,
previous2new2/new2raw2, bordi-width-mirror-BIOS-SWI-DMA passano.314istruzioni/70PC/
32dispatch solo copertura, non intera ABI script/callback/scena ordinaria.
[Consumo](../rom_data_scripts.md#managed-publication-and-direct-command-fields).

**ROMDATA-04-SCRIPT-COMMAND-DIRECTORY-CLI completato EUR/Wine (2026-10-24):**
Windows18103production senzaavvisi/canonicalripristinato; Wine20133CLI realecrea/
riusa senza scrivere47filemanaged da cwddiversa, carica cachevecchia. Tutti45file
generazione coincidono coldatasetaccettato. Unsolo frame boot di attivazione,
0letturecomandi: non equivale ad accettazione di una scena ordinaria.

**ROMDATA-05-SCRIPT-COMMAND-DIRECTORY-SCENE completato EUR/Wine (2026-10-24):**
Traccia produttore1265cc←125b6a/manager125ac8 e callback dinamici;63snapshot EUR
transitori autentici conservati hanno rootqueued0, nessun replay né modifica.
Unico locatorCLI Wine81783 con input+suspend save autentici già conservati fino
all'ingresso battaglia8476/classic240x160 legge42B=14advance28callback/7comandi.
Unico Wine93633 fullpair dallagenerazioneCLI passa tutti8476stati-clock/immagini/
OAM/dispatch e audio misto-PSG-FIFO-HLE bitwise, stessa copertura positiva.
Riusa testexe/cache/input/save, nessun rebuild/prepare/oracolo/native/lifecycle/
vecchieprove o creazionefixture/navigazione/payloadedit. Altri108comandi mirati
solamente; non intera ABI/new32:9. e49event/selector/dispatch0: terzo locator
negativo chiuso, niente fullpair richiesto per eventi.316hash-mtimeguards passano.
[Scena](../rom_data_scripts.md#production-cli-and-positive-ordinary-scene).

**ROMDATA-01-SCRIPT-ARCHIVE completato nell'inventario statico EUR (2026-10-24):**
115464B/2range: archivio relativo114528B B946EC..BB064C e234chiavi4B/936B
D68C2C..D68FD4. Limiti da root indipendenti testo localizzato/coordinatedirectory.
Domini delle chiavi e primi offset concordano con62banchi/233slot,248+932B indici.
213target distinti/20link ripetuti includono5link aheader e2aFFfill: conservati,
non respinti/marcati morti.207segmenti fisici/35048comandi/110opcode/78212paramB;
45fillspan88B.493link5a verso49selettori esistenti e dipendenza reale dai comandi.
Byte2chiave seleziona cinque archivi testo ancora nonmigrati; byte3raw conservato.
Ricostruzione indipendente/tutti115464B/234chiavi/233formule target non-null/128halfword5wordguard/
disgiunto44owner passano senza guestexecution. Prima assunzione di programma
nonvuoto respinta su46:1/49:1; corretta prima del piano. Non CFG/intera semantica
argomenti/reachability/nuovi cap guest. Inventario e modello chiusi; integrazione
runtime/CLI/scena trattata nelle unità sotto. Inventario lasciava e50-d49/USAe8-d7 invariati. [Contratto](../rom_data_scripts.md#relative-script-archive-and-event-key-source-contract).

**ROMDATA-02-SCRIPT-ARCHIVE-MODEL completato EUR/Wine (2026-10-24):** modello
strict lossless dell'intero ambito115464B,234chiavi/62banchi233slot/207programmi
fisici35048istruzioni/45fill; target Program/Header/Fill e parole relative
ricostruite, nienteROMshadow/copiealias. Lunghezze e493link dipendono dagli asset
comandi/effetti reali. Windows91283senzaavvisi/unicoWine85436 passa alprimo giro:
238keyaddress/234target2livelli(compreso raw0unaligned)/234textbank/35054opcode/
2397relative-target/493selectorparam. Tutti115464B ricostruiti;42333Bdistinti letti
ai loadsite originali,72811param+234keyraw+86fill=73131Bconservati/hashchecked non
letti inquesto ambito.40guard passano;224382istruzioni62PC solocopertura. Nonintera
ABI/interpreter/native/stato-clock/ordinary. e50-d49/USAe8-d7 invariati; nuovi
Linux/USA/Windowsnativo dafare. [Modello](../rom_data_scripts.md#typed-relative-archive-model-and-reduced-wine-source-oracle).

**ROMDATA-03-SCRIPT-ARCHIVE-RUNTIME completato nei criteri mirati EUR/Wine
(2026-10-24):** e51-d50 pubblica script-archive.json/engine.script-archive,
46file45risorse/all44decodedprecedenti/e50conservati/USAe8-d7 invariato. Dipendenze
reali comandi+effetti validate; due binding115464B ricostruiscono indici/opcode/
parametri/chiavi/fill tipizzati, senza shadow/nuovi limiti guest/sequencer/ABI.
Windows22414senzaavvisi/Wine76182duefiltri passano alprimo tentativo13guard/
lifecycle/all115464B=1180index35048opcode78212parameter936key88fill e28prefix
original/interpreter/native stati-clock completi. APIcompiled37index4opcode/
7parameter12key1fillB; raw0/234/239/65535/low16/rotation/header-fill-target/q0/
relative-target/returnloop/oldpalette4B stessapagina/mixedraw-new/bordi-width-
mirror-realBIOS-CpuSet-FastSet-DMA passano.169istruzioni66PC28dispatch solocopertura;
noninteraABI/ordinary. [Consumo](../rom_data_scripts.md#managed-relative-archive-and-typed-guest-reads).

**ROMDATA-04-SCRIPT-ARCHIVE-CLI completato EUR/Wine (2026-10-24):** Windows48522
production senzaavvisi/canonicalripristinato; Wine66454CLI realecrea/riusa senza
scrivere48filemanaged da cwddiversa/carica cachevecchia. Tutti46filegenerazione
coincidono coldatasetaccettato e51. Locator autentico suspendboot-to-battle8476/
classic240x160 da input/save conservati positivo:27B=8index7opcode10parameter2key,
un banco/programma/chiave,0fill.365hash-mtimeguard passano; nessun rebuild vecchie
prove/fixture/navigazione/payload edit.

**ROMDATA-05-SCRIPT-ARCHIVE-SCENE completato EUR/Wine (2026-10-24):** unico
Wine44039fullpair8476frame dalla generazioneCLI e51, stessi input/save autentici/
testexe/cache già esistenti, passa tutti stati-clock/immagini central-wide/OAM/
dispatch/audio mistoPSGFIFObitwiseHLE, uso27B invariato e366hash-mtimeguard.
Non ripete l'accettazione e50/source-model/native/lifecycle/CLI/prepare/build.
Nessun test guest fallito/fixture/navigazione/payload edit. Altri61banchi/206programmi/233chiavi/
tutti45fill mirati solamente; noninteraABI/new32:9. e49usoordinario positivo
resta aperto dopo tre locator negativi; Linux/USA/Windowsnativo dafare.

**ROMDATA-01-SCRIPT-LOCALIZED-TEXT completato nell'inventario statico EUR
(2026-10-24):** cinque root BB064C/C076F0/C5FBF0/CB7C48/D10644, limiti dalle root
lingua indipendenti e script-keyD68C2C:1803744B/5range disgiunti45owner e51.
63voci top perlingua da keymax62/minoffsetFC, prima voce aliasbanco0nonlunghezza.
11link normali condivisi/lingua,26varianti banco61 selezionate da divisione del
byte vivo per10; resto separato rimappa il messaggio per24con troncamento u16.
Branch full-u16/low-byte/special/fallback/rootkey originali, nessun nuovo cap.
385banchi/41395indici-u16 inclusi385sentinel/41010programmi8202perlingua; grammar
Program/customLZ già presente.9813compressed extended/5709raw extended/25488raw
compact,212011comandi11forme/147glifi1..3112 già nel font; dipendenze reali chiavi
script e font.1260top+520variant+82790bankindex+1684407program+34767alignment B.
34314gap1..3B, quattro misti: respinta ipotesi uniforme prima del piano/modello/
Wine, bytes esatti conservati. Ricostruzione/hash interi1803744B/15195formule/
349halfword13wordguard/deps/glifi/disgiunzione passano senza guestexecution/build/
test o repeat prove precedenti. e51-d50/USAe8-d7 invariati; modello/Wine chiusi
nell'unità sotto, runtime/CLI/ordinary restano figli aperti. [Contratto](../rom_data_text.md#script-selected-localized-text-source-contract).

**ROMDATA-02-SCRIPT-LOCALIZED-TEXT-MODEL completato EUR/Wine (2026-10-24):**
modello strict interi1803744B con riferimenti Bank/Variants tipizzati, payload
fisici non duplicati, u16indici/sentinel ricostruiti e gap esatti. Program codec
esistente/header16bit/argomenti/encoded-decoded tails/deps chiavi-font conservati.
Windows42377senzaavvisi/primoWine95147fallisce JSON: variante unit accettava
campoextra; corretto a emptystruct mantenendo asserzione, prima di casi guest.
Tutti artefatti prima prova conservati. Windows55123senzaavvisi/unicoWine34745
corretto passa tutti dati/hash/1803744B/385banchi41010programmi e45guard:
22shape6source-edit3dependency12JSON-shape2identity. Nuovi casi originali7lingue/
1170key-bank/1280quotient-vivo/256remainder-vivo/90raw-special/1170u16indici/
837header/837source-vs-typed output+guard, inclusi249corpi/code decompressi.
2116900istruzioni834PCsolo copertura, non native stato-clock/ABIintera/ordinary.
Checkpoint inizializzato conservato e immutato, argomenti scratch controllati;
no fixture gameplay/navigazione/modifica checkpoint-save-input o vecchie prove
ripetute.4060artefatti precedenti/prima prova/newexe e368hash-mtime protetti.
Solo due dichiarazioni modulo in file esistenti; e51-d50/USAe8-d7/ABI invariati.
[Modello](../rom_data_text.md#typed-script-selected-text-model-and-reduced-wine-source-oracle).

**ROMDATA-03-SCRIPT-LOCALIZED-TEXT-RUNTIME completato nei criteri mirati EUR/Wine
(2026-10-24):** e52-d51/script-text.json/text.script-archives,47file46risorse,
all45decodedprecedenti/e51/USAe8-d7 conservati, dipendenze reali chiavi/font.
Cinque binding1803744B per indici/header/control/glifi/comandi/operandi/terminator/
slack, literalmap condiviso con testi precedenti; ABI/save/live guest invariati.
Coda decodificata conservata,0literal sorgenti perché generata da LZ/backref:
respinta solo ipotesi privata all-class-positive. Windows23146core senzaavvisi;
primo frontend fallisce solo array test5vs11/importunused, corretto test e
ricompilato solo frontend39830senzaavvisi, log iniziale conservato. Wine95863
passa tre filtri alprimo avvio: unico core literal-poison per helper cambiato,
managed13guard/lifecycle/all1803744B/5lingue385banchi41010programmi130varianti34314gap
più163Btesti precedenti0/1/2, native35prefissi original/interpreter/compiled stati-
clock completi. CompiledAPI[21,17,17,13,6,4,5,1,3,0,3]B; raw255/wrap/sentinel/
BB0-D68previous-new/rawdopoownerchiavi/width-mirror-realBIOS-CpuSet-FastSet-DMA
passano.379istruzioni65PC71dispatchsolo copertura, nonABIintera/reachability.
[Consumo](../rom_data_text.md#managed-script-text-and-direct-typed-reads).

**ROMDATA-04-SCRIPT-LOCALIZED-TEXT-CLI completato EUR/Wine (2026-10-24):**
Windows51427production senzaavvisi/canonicalripristinato. Wine71122CLI realecrea/
riusa senza scrivere49filemanaged/cwddiversa/cachevecchia, tutti47filegenerazione
uguali aldatasetaccettato;468hash-mtimeprotetti. Locator suspendboot8476legge0
nuovitesti: negativochiuso/niente fullpair, nonricontarevecchiocontesto e49.

**ROMDATA-05-SCRIPT-LOCALIZED-TEXT-LOCATOR completato EUR/Wine (2026-10-24):**
Audit statico74snapshot EUR su campi testo non dà cue manager; nonprova deadcode,
quattro header USA esclusi prima delpayload/replay. Nuovo locator Wine24902
coldboot school3000/classic0 da input autentico conservato, senza save/state:
289B=8top4bankindex8header93control136glyph39command1terminator,decoded176B,
unalingua/banco/dueprogrammi,0variant/operand/tail/slack.468hash-mtime passano.
Nessun build/prepare/fixture/navigazione o modifica checkpoint-save-input.

**ROMDATA-05-SCRIPT-LOCALIZED-TEXT-PAIR completato EUR/Wine (2026-10-24):**
unicoWine43805pair3000frame dalla generazioneCLI e52/stesso input/nessun sav-state
override/testexe-cache esistenti passa ogni stato-clock/immagine central-wide/
OAM/dispatch/audio mistoPSGFIFObitwiseHLE, stesso uso289B/468hash-mtimeguard.
Nessun rebuild/prepare/vecchie prove/nuova fixture gameplay/navigazione/modifica
checkpoint-save-input. Altre4lingue384banchi41008programmi/
130varianti34314gap restano mirati;0operand/tail/slack ordinari.

**ROMDATA-01-SCRIPT-COORDINATE-INDEX-SOURCE / ROMDATA-02-SCRIPT-COORDINATE-PATHS-MODEL
completati EUR/Wine dichiarato (2026-10-24):** reader128914/rootD68FD4 e
readerA1F8/rootD69BC4 delimitano3056B/210liste/856coppieXY/42paddingFF84B.
Directoryrelativa840B/countu16 420B/XY1712B ricostruiti esattamente, disgiunti
da46owner e52;194halfword5word/hash completi.68linkscript/56liste:66opcode44
indicebyte2/byte3ignorato (respinta ipotesiu16 prima delmodello),2opcode45u16.
Dipendenze reali script-archive/script-command-directory; altezza da mappa RAM,
low16indice/low8destinazione/countu32/ritornolow8 originali, nessuncapguest nuovo.
Modellostrict lossless; Windows25513senzaavvisi/unicoWine70105passa alprimo avvio
tuttiicampi/hash/24guard ed esecuzioneoriginale212copie (tutte210liste+due celle
altezzaRAMvariate),13prefissi rawindice/destinazione e13producer-bytewidth/
ignored/nibble/taskargs.2972Bsorgente osservati,84padding preservati-nonletti;
40116istruzioni134PC solo copertura. Non native/fullstate-clock/ABIintera/
allocator/taskmovement/ordinary. e52-d51/USAe8-d7 invariati, nessunafixturno/save/
fixture/navigazione; nuove verificheLinux-USA-Windowsnativo dafare.
[Contratto e modello](../rom_data_scripts.md#typed-coordinate-path-source-model).

**ROMDATA-03-SCRIPT-COORDINATE-PATHS-RUNTIME completato nei criteri mirati EUR/Wine
(2026-10-24):** e53-d52/script-coordinate-paths.json/engine.script-coordinate-paths,
48file47risorse/all46decodedprecedenti/e52/USAe8-d7 conservati, dipendenze reali
archive/command. Owner immutabile condiviso per index/count/XY/alignment, nessuna
ROMshadow/hostmovement/capguest/ABI nuova; triploD68testo/chiavi/coordinate e raw
D69BC4 preservati. Primo build10180passacore/fallisce solo arraycontatoretest5vs4;
corretto frontend27578senzaavvisi/riusato core. Wine30996managed alprimo avvio
passa14guard/lifecycle/all3056B/210liste856punti42alignment/tutti46precedenti/e52.
Primo native si ferma su looposservatoretest0..5; corretto a zip4, conservati log/
EXE/ELF/sorgente fallito, ricompilato solo frontend75147senzaavvisi. Wine31012
solo native passa24prefissi original/interpreter/compiled stati-clock completi,
quattrocopie intere/altezzeRAM/rawarg/allclassi[28index85count76XY2alignment]B,
mixedowner/bordi-width-mirror-realBIOS-CpuSet-FastSet-DMA.1443istruzioni87PC143
dispatch solo copertura, non intera ABI allocator/movementtask/ordinary.
[Consumo](../rom_data_scripts.md#managed-coordinate-path-publication-and-typed-reads).

**ROMDATA-04-SCRIPT-COORDINATE-PATHS-CLI / ROMDATA-05-SCRIPT-COORDINATE-PATHS-PAIR
completati EUR/Wine (2026-10-24):** Windows29223production senzaavvisi/canonical
ripristinato esattamente. Wine45975CLI realecrea/riusa senza scrivere50filemanaged/
cwddiversa/cachevecchia, tutti48filegenerazione uguali aldatasetaccettato.
Locator autentico6000frame da21420/stato/input giàconservati positivo:25busreads/
28B=4index14count10XY/unalista,0alignment. UnicoWine76926pair stessi6000frame/
stato/input/generazioneCLI passa tutti statieclock/immagini central-wide/OAM/
dispatch/audio mistoPSGFIFObitwiseHLE, campioni[13167000,13167000,1584000,1584000,
13166910], stessi28B/581hash-mtimeguard. Nessuna nuova fixture/navigazione/prepare/
locator/vecchia prova ripetuta. Altre209liste/tutti42alignment restano mirati;
nuovi32:9/Linux-USA-Windowsnativo/ABIintera non verificati, nessuna misura spazio/
performance. [CLI e scena](../rom_data_scripts.md#coordinate-path-production-cli-and-ordinary-scene).

**ROMDATA-01-AUXILIARY-SCRIPT-ARCHIVE-SOURCE completato per la proiezione EUR
 dichiarata (2026-10-24):** readerA1F8/rootD69BC4 distinto dall'archivio primario,
indici relativiu16/action-slotu8.4200B:4098archivio+102directory17comandi a370F94.
8banchi/442slot/126targetdistinti/30entryinterne/133programmi fisici913istruzioni,
900index+913opcode+2275param+10alignmentB. Rootwalknoncopre69istruzioni: preservate
colparsefisico.489targetrelativi su confiniistruzione; CFGstrutturale908/913nonè
reachability, altre5preservate. FineproiezioneD6ABC6/ultimoterminator; FF2seguenti
esclusi. GraphicsbaseD6B4C8 è solo ceiling,2306Bfuoriproiezione; directorysuccessor
370FFC indipendente, zero2gap esclusi. Ricostruzione/hash4200B/723halfword4word/
formule489/disgiunto47owner e53passano senza build/guestexecution/testLinux.
Nessuncapguest/hostinterpreter/dipendenzaprimario/ABIintera. e53-d52/USAe8-d7
invariati; modello/oracoloWine verificati sotto; managed/runtime/CLI/ordinary
ancoraaperti.
[Contratto](../rom_data_scripts.md#european-auxiliary-script-archive-source-contract).

**ROMDATA-02-AUXILIARY-SCRIPT-MODELS completato EUR/Wine dichiarato (2026-10-24):**
modellostrict directory/archivio4200B/133programmi/913istruzioni con30entryinterne
program+instruction, parametri/gap/zero-row preservati; dipendenza reale callback/
advance propri, nessun capguest/ROMshadow/interpretehost. Windows66583senzaavvisi;
primoWine20886fallisce aspettativatest raw8:0/indirizzodispari. Soloosservatore/
expectationtest corretti, frontend54240senzaavvisi/Wine3148passa. Letturaexec.rs
stabilisce rotazione32-bit: ulteriorecaso8:1 verifica highbits/target14D69E97;
frontend52724senzaavvisi/Wine77162finalepassa, modello/coreimmutati, conservati
fallimento/eseguibili/log/provaintermedia.40guard/tutticampi-hash/723halfword4word,
442lookup/913dispatch+RAMopcode3/80advance/489suffissirelativi/10wrap/15raw/
11wrapper.2887Bdistinti osservati;1313B=1297param+10gap+6directoryzero conservati
hashchecked-nonletti.31839istruzioni177PCsolocopertura, nonpredicati/interaABI/
native/fullstate-clock/ordinary. Nessuna fixture/save/input/navigazione o vecchia
prova ripetuta; e53-d52/USAe8-d7 invariati, nuoviLinux-USA-Windowsnativo dafare.
[Modello](../rom_data_scripts.md#typed-auxiliary-archive-and-reduced-wine-source-oracle).

**ROMDATA-03-AUXILIARY-SCRIPT-RUNTIME completato nei criteri mirati EUR/Wine
(2026-10-24):** e54-d53pubblica directory/archivio ausiliari,50file49risorse/
all47decoded precedenti/e53retained/USAe8-d7. Dipendenza reale archivio→propria
 directory, ownerimmutabile unico4200B senzaROMshadow/hostscript/capguest/ABI
save-native nuova. Pagine370audio/D69coordinate e raw370FFA+/D6ABC6+ preservate,
control/livecursor/predicati/LDRH32-bit originali. Windows44279core+frontend
senzaavvisi; Wine61473duefiltri nuovi passanoalprimogiro26guard/lifecycle/
create-reuse-repair-move/failuretransaction/e53retained/unavailablefile/Load-Undo-
backup-Restart/stessoowner/all4200B/17comandi8banchi133programmi5gap e44prefissi
original-interpreter-compiled stati-clock completi. APIcompiled[6advance65callback
18index19opcode15param2alignment]B, entryinterne/signedcursor/rawopcode/8:1highbits/
fullaction2vs102/mixedowners/bordi-width-mirror-realBIOS-CpuSet-FastSet-DMA passano.
624istruzioni92PC52dispatch solocopertura, noninteraABI predicati/interpreter/
callback/ordinary.587hash-mtimeguard passano. Nessun vecchiotest ripetuto/nuova
fixture/navigazione/performance/spazio; CLIreale/ordinario positivo verificati
nelle unità sotto.
[Consumo](../rom_data_scripts.md#managed-auxiliary-script-fields).

**ROMDATA-04-AUXILIARY-SCRIPT-CLI / ROMDATA-05-AUXILIARY-SCRIPT-PAIR completati
EUR/Wine (2026-10-24):** Windows10336production senzaavvisi/canonicalhash-mtime
ripristinato esattamente, main.rscontenutoimmutato/mtime-only invalidazioneCargo.
Wine23153CLI realecrea/riusa senza scrivere52filemanaged/cwddiversa/vecchiacache,
tutti50filegenerazione uguali aldatasetaccettato e54. Unicolocator autentico6000
frame da21420/stato/inputgiàconservati positivoentramberisorse: directory96B=
32advance64callback/6comandi, archivio82B=16index20opcode46param/2banchi2programmi,
0alignment. UnicoWine93545pair medesimi6000frame/stato/input/generazioneCLI/
testexe-cacheconservati passa ogni statocompleto-clock/immaginecentral-wide/OAM/
dispatch/audio mistoPSGFIFObitwiseHLE, campioni[13167000,13167000,1584000,1584000,
13166910], stesso usonuovo/640hash-mtimeguard. Altri11comandi6banchi131programmi/
5alignmentspan miratisolamente; noninteraABI/new32:9/Linux-USA-Windowsnativo.
Nessun rebuild/prepare/locator/vecchiaprova ripetuta/nuovafixture/navigazione o
misura performance/spazio. [CLI e scena](../rom_data_scripts.md#auxiliary-script-production-cli-and-positive-ordinary-scene).

**ROMDATA-01-TERRAIN-MAP-SOURCE completato per il percorso EUR dichiarato
(2026-10-24):** il loader originale1B688/state6→1AE88 usa root7662A8/162record
e tutti486link field04/08/10dei352programmi BG già estratti e consumati.
Verificati14256descrittoriB/398227packetB/134aliaslink, hash e proprietà esclusiva
contro49owner,40halfword11literal9BL. Field10inizializza griglia altezze/flag
16×16a02007CB0; puntatore/dimensioni/flag vivi e piani EWRAM/VRAM restano dinamici.
Nessuna nuova risorsa/decoder/revisione o vecchio test/build rieseguito; e54-d53/
USAe8-d7 invariati. Accessor10B1DFB4/altri contesti/semantica flag restano ignoti.
[Classificazione e limiti](../rom_data_maps.md).

**ROMDATA-01-MAP-RASTER-PALETTE-SOURCE completato EUR statico (2026-10-24):**
root4872B0..489BB0/10496B,32record noncompressi/33range/324link selettori54/55,
64index192header10240colorB. Header3u16e160coloriu16perrecord;26contenuti diversi
ma32identitàfisiche. Decoderstatico ricostruisce tuttifilebyte/hash/link, verifica
67halfword13literal10BL e nessun overlap49owner. Originali1EAF0e1EDECconservano
diversi indiciunsigned/signed e fallbackFFFF; nessun nuovo cap guest32. Copia
originale161halfword/322Blegge headerseguente o prima word489BB0già appartenente
alle palettealternative, poi riscrive word160dest con word0. Nessuna duplicazione
dell'owner adiacente o eliminazione lettura/clock. Nessun build/esecuzione guest;
e54-d53/USAe8-d7 invariati. [Contratto](../rom_data_maps.md#map-raster-palette-source-contract).

**ROMDATA-02-MAP-RASTER-PALETTE-MODEL completato EUR/Wine (2026-10-24):**
modelloisolato lossless32record/33range/10496B/324link dipendenti dai descrittori;
header3u16/coloriu16/indexderivato/identitàfisica senzaROMshadow. Windows30586
frontendtestbuild senzaavvisi; unicoWine39763nuovofiltro primoPASS. Tutti campi/
hash/JSON/15guard,64inizializzazioni32getter canonici11rawgetter7prefix unsigned/
3disabled/2update differiti2setterespliciti1setupDMA passano.36osservazioni
CpuSetconfrontano tutti322Bprima del wrap, inclusa013Cdel vecchioowner adiacente.
Variante sintetica bit15/rangeword/RAMcompletafuori stack/SP/callee passano;
10496B distinti letti ai loadsite/copieHLEverificate.4753istruzioni202PCsolocopertura,
non fullstate-clock/native/ordinary o DMA raster eseguito;643hash-mtimeguard.
e54-d53/USAe8-d7 invariati, nuovoLinux-USA-Windowsnativo pending.
[Modello e prova](../rom_data_maps.md#typed-raster-palette-model-and-original-reader-oracle).

**ROMDATA-03-MAP-RASTER-PALETTE-RUNTIME completato nel perimetro mirato EUR/Wine
(2026-10-24):** e55-d54pubblica `map-raster-palettes.json` /
`graphics.map-raster-palettes`,51file50risorse/tutti49precedenti identici/e54
conservato/USAe8-d7invariato. Dipendenza realeBGdescriptor/unicoownertipizzato
serve33range10496BsenzaROMshadow; ownerprecedente conserva word489BB0/crossread.
Windows82612core/frontend senzaavvisi; unicoWine41457due nuovifiltri primoPASS:
17guard/lifecycle/Load-Undo-backup-Restart/fileindisponibili/all10496B/32programmi;
28percorsi original-interpreter-native stato-clock completi/APIcompiled[15index
46header24color]B, raw signed/unsigned/sentinelle/window/copie322B/wrap/queuedDMA/
confini-oldowner-raw. Width/mirror/BIOSbusmode/quattroSWI/unDMAimmediato passano;
646hash-mtimeguard.638istruzioni204PC55dispatch solocopertura, non intera camera/
rasterDMA/ordinaryABI. Fonte/modello già chiusi non rieseguiti.
[Runtime e limiti](../rom_data_maps.md#managed-raster-palette-publication-and-typed-reads).

**ROMDATA-04-MAP-RASTER-PALETTE-CLI / ROMDATA-05-MAP-RASTER-PALETTE-PAIR
completati EUR/Wine dichiarato (2026-10-24):** Windows26915production senza
avvisi/canonico hash-mtime ripristinato/maincontenutoimmutato-mtimeCargo.
Wine40160CLI realecrea/riusa senza scrivere53filemanaged/cwddiversa/vecchiacache,
tutti51filegenerazione uguali all'e55accettato. Unicolocator autentico coldboot
1500inputdemo conservato/senza stato-save/override positivo50880aggregateB=
2index322header50556color/25440busreads/mask2record (include header del vicino,
non due palette complete o byte distinti). UnicoWine90978pair medesimi1500frame
0..1500/generazioneCLI/testexe-cache passa ogni stato-clock/immagine/OAM/dispatch/
audio mistoPSGFIFObitwiseHLE, campioni[3291096,3291096,395212,395212,3269758],
stesso uso nuovo/701guardhash-mtime. Source/model/targeted e vecchieprove non
rieseguite; nessuna nuova fixture/navigazione/performance/spazio. Altri contesti/
intera camera-rasterABI/new32:9/Linux-USA-Windowsnativo restano pending.
[CLI e scena](../rom_data_maps.md#raster-palette-production-cli-and-positive-ordinary-boot).

**ROMDATA-01-OBJ-EMPTY-CALLERS completato nel perimetro statico diretto EUR
(2026-10-24); estensione hold non provata respinta:** i tre vecchi header vuoti
B4BD20/bank0seq1, B4BD34/bank1seq1, B4FA10/bank14seq6 restano invariati con
frames=[]/senza zero_count_hold. Dodici costruttori dei primi due banchi usano0;
18chiamanti dell'inizializzatore8BBF0producono modeword0..3→sequenze0..3;
due aggiornamenti usano2/3. Tutti30letterali delle radici/47fingerprint istruzioni/
50owner verificati; costanti attraverso chiamate secondo ABI r4..r11preservati.
SeiBdi possibili selettori restano raw; durate8/16/4alias di indici già tipizzati.
Rawmode7/65539→6/7, repeat1: nessun cap guest o prova globale di irraggiungibilità.
Nuovi producer/trace ordinaria con scheduling stabilito sono il solo motivo per
riaprire; chiamate indirette/mutazioni RAM non verificate. Nessun nuovo modello/
risorsa/revisione/build/esecuzione guest; e55-d54/USAe8-d7 invariati.
[Contratto](../rom_data_obj.md#earlier-empty-sequence-caller-domains).

**ROMDATA-01-UI-LIST-ENTRIES-SOURCE completato nel perimetro statico EUR
(2026-10-24):**18chiamanti diretti8BBF0→16ingressiROM/12liste36voci e due
filtriRAM0200F250con template397A9Cda6voci. Tredici intervalli42voci336B,
disgiunti50owner; tutti18letterali/93fingerprint e ricostruzione byte passano.
Campiu8ID/X/Y/opaque e left/right/up/downraw; niente pointer/cap/count/target
normalizzato. Countwordfirmato della copia distinto dal byte live; getter8CA64/
input8BDECcon coordinate da indice raw non mascherato. Proiezioni32maschere live
stabiliscono estensione template; ID/Y/up/down e selezione restano RAM. Ordine
clear-down→copia base→up preserva down1con zero slot, corretto durante revisione
statica privata. Nessun nuovo modello/risorsa/revisione/build/guestexecution;
e55-d54/USAe8-d7 invariati. [Contratto](../rom_data_ui.md).

**ROMDATA-02-UI-CHOICE-LISTS-DATA completato per EUR/Wine (2026-10-24):**
modello pubblico strictlossless/sharedsourceplan per13tabelle42voci336B/IDfisici,
byte3opaque e4navigationrawsenza cap guest. Un Windows build mirato warning-free
più un solo filtro Wine passano17guardie/all336Boriginal loadsites/18copy prefix
(13fonti+1variante+4count raw0/-1/-65536/257)/44getter/43idle/64prefixduefiltri32
maschere/2prefixcoordinateflag80-86nonmascherate. EWRAM/IWRAMfuoristack e SP/callee
ritorni uguali; niente intera widget/native/fullstate-clock/ordinario.703guardhash-
mtime;12231istruzioni212PCsolo copertura. Variante tutti8campiu8passaforma e
fallisce autenticazione; copia257preserva overwrite metadata originale. Nessuna
vecchia prova/sourcecensus ripetuta o nuova fixture/navigazione/performance/spazio.
e55-d54/USAe8-d7 invariati. [Modello e prove](../rom_data_ui.md#lossless-model-and-original-wine-readers).

**ROMDATA-03-UI-CHOICE-LISTS-RUNTIME completato nel perimetro mirato EUR/Wine
(2026-10-24):** e56-d55pubblica ui-choice-lists.json/engine.ui-choice-lists,
52filegenerazione51risorse/tutti50precedenti decodificati uguali/e55conservato/
USAe8-d7 invariato. No dipendenza asset, fullsetcoerente. Owner immutabile13span
336Bserve ID/X/Y/opaque/navigationraw a CPU/API native e copie BIOS/DMA; counts/
filtriRAM/copie/input/cursori/task e save-nativeABI invariati. Windows36248unico
build core/frontend warning-free; coretest solo compilati. Wine82471duefiltri
esatti firstPASS17guardie/lifecycle/all336typedB=[42ID84coordinate42opaque168nav]/
13liste/failedattachtransazionale/pubblicazione fallita-create-reuse-repair-move/
e55retention/unavailableLoad-Undo-backup-Restart/sharedowner. Native29percorsi
originale-interprete-compilato fullstate-clock/compiled[92ID183coordinate92opaque
368nav]Baggregati; tutti336newraw+14oldOBJrawBalterati. Counts0/-1/257/RAMcopy/
getter-idle/4prefixfiltri zero-tutti, widths/mirrors/realBIOSbus/3SWI/1DMA passano.
Copiamista2old+2raw+2newB/SWI-DMA e APIword2raw+2new, oldOBJstessapagina4B;
706guardhash-mtime.3010istruzioni186PC38dispatch solo copertura, niente intera
widget/ordinaryABI. Source/model/oldproof non ripetuti; nessuna nuova fixture/
navigazione/performance/spazio. [Consumo e prove](../rom_data_ui.md#managed-publication-and-typed-runtime-reads).

**ROMDATA-04-UI-CHOICE-LISTS-CLI / ROMDATA-05-UI-CHOICE-LISTS-PAIR completati
per il perimetro EUR/Wine dichiarato (2026-10-24):** un Windowsproductionbuild
60923firstPASSsenzaavvisi/frozen444c2db9..; canonico hash/mtime esatti ripristinati,
maincontentinvariato/mtime-onlyCargoinvalidation. CLI36895crea/riusa52filegenerazione
54gestitino-write/tutti52uguali accettati e56/cwd estranea/oldcache. Pecorelle1000
0newUI, Moogle-menu6000newUI: negativiterminali/no fullpairs/estensioni. Ispezione
readonly110snapshotEURsenza copie dirette non prova irraggiungibilità; battlemenu
componente diverso. Screenshot/inputPartyunit esistenti guidano unicoCLI54355
320positivo32Baggregati=[4ID8coordinate4opaque16nav]/8busreads/1lista. Snapshot
worldmap7701a360../inputPartyunitbf757e69.. invariati, nessuna fixture/navigazione
nuova/payloadedit. UnicoWine95582fullpair stessi320frame4519→4839/ogni stato-
clock/immaginecentrale-wide/OAM/dispatch/mixedPSGFIFObitwiseHLEaudio coincide;
audio[702240,702240,84480,84480,702164]. Stesso32Bnuovi/1lista/769guardhash-mtime.
Frozen0f942581testexe/actualCLI DQ1nbu/oldcache/width0, misto native-fallback.
Source/model/managed/native/oldscene non ripetuti; niente allnative/interaWidget/
new32:9. Usoordinario altre12liste pending; verifichemirate chiuse; nuovi Linux/USA/Windowsnativo/32:9
restano pending. [CLI e scena](../rom_data_ui.md#production-cli-and-positive-ordinary-party-scene).

**ROMDATA-01-UNIT-OBJ-INITIAL-POSE / ROMDATA-02-UNIT-OBJ-INITIAL-POSE completati
nel perimetro fonte/modello EUR/Wine (2026-10-24):** originale2D394→233A8→CF144
produce ID da metadata52property4/metadata14halfword04 già migrati;2D3F8→22788
passa fullwordanimation0/part0.110jobroute0+93special proiettano121ID, senza
capguest o catalogofisicocompleto; job0/1valore65535resta irrisolto/inesportato.
Directory394834→primecelle→121sequenze477frame20Boperation1, OAMbaseA50020/
6liste31oggetti→tilecountoriginale221D4→333richieste135168B4bppgrezzi dalla
base898A40.00B14accodaDMA, non decodifica;00800esegueDMA3reale.415span146358B
disgiuntida51owner;94instruction13BL6literal fingerprintsstaticiPASS. Modello
strictlossless/sharedsourceplan/4224tile8x8indicepalette conserva header/tail/
attributiopachi. Windows20506unicofrontendtestbuildwarning-free/frozen1d759b0d;
Wine19197unicofiltro firstPASS121resetoriginali22E20/1constructorprefix22788/
477scheduler→queue→DMAupload/tuttaVRAM/all135168pixelB/16shape-sizeinclusoshape3/
11JSON-authguard/SP-callee/RAMesterna.9393loadsiteBdistinti+135168DMA=144561B;
1797Bheader/tail/OAMpreservatinonletti;245830istruzioni499PCsolocopertura.
703guardhash-mtime; niente fullunitABI/native/fullstate-clock/ordinario,
productionbuild/oldtest/fixture/navigazione/performance/spazio. EUR e56-d55/
USAe8-d7invariati. [Contratto/modello](../rom_data_unit_obj.md).

**ROMDATA-03-UNIT-OBJ-INITIAL-POSE completato nel perimetro mirato EUR/Wine
(2026-10-24):** e57-d56pubblica unit-obj-initial-pose.json/graphics.unit-obj-initial-
pose con dipendenzemetadata52/14;53file52risorse/tutti51precedentidecodificati/
recordmanifestcoperturaidentici/e56conservato/USAe8-d7. Ownerimmutabile restituisce
directory/index/header/frame/OAM da campitipizzati e pixel da indici4bpp; niente
ROMshadow/RAM-scheduler-DMA-VRAM/save-nativeABIalterati. Windows73480unicobuild
core/frontend warning-free; coretestsolocompilati7a95a911../frontendd0995de8..
frozen. Wine19825duefiltriesatti firstPASS17guardie/lifecycle/tutti146358typedB=
[484directory484index484header9540frame198OAM135168pixel]/121sequenze, failedattach
transazionale/pubblicazionefallita-create-reuse-repair-move/unavailableLoad-Undo-
backup-Restart/sharedowner. Native30percorsioriginale-interprete-compilato stato-
clockcompleti/API[28dir20index8header127frame164OAM1pixel]Baggregati; pixelAPIprobe
distinto dai veri executorDMAcompilati. Raw146358new+20oldOBJBalterati, ID65535/
animation65537/liveRAMframe passano.415edges/width/mirrors/BIOSbus/2SWI/1DMA:
copiamista2new+1276raw+8oldOBJconaddebiti2/8; APIword2new+2raweditati; oldOBJsamepage
4Bsenzaaddebitionuovi.706guardhash-mtime/8compiledinputsimmutati;3542istruzioni496PC
177dispatchsolocopertura. Source/model/oldproofnonripetuti; nessuna interaunità/
allocatorABI/ordinario/fixture/navigazione/performance/spazio.
[Consumo e prove](../rom_data_unit_obj.md#managed-publication-and-direct-runtime-reads).

**ROMDATA-04-UNIT-OBJ-INITIAL-POSE-CLI / ROMDATA-05-UNIT-OBJ-INITIAL-POSE-PAIR
completati nel perimetro EUR/Wine dichiarato (2026-10-24):** Windows49887unico
productionbuildwarning-free/frozend4c782c6..; canonico hash/mtimeoriginali ripristinati,
maincontentinvariato/mtime-onlyCargo invalidation. Wine57046treCLI reali create/
reuse-no-write55filegestiti/all53generazioneugualiaccettati/cwdestranea/oldcache.
Actuale57generation-ZV0TqS/ab9286b2..; originalworldmap7701a360../Partyunitinput
bf757e69..320senza nuova fixture/navigazione/payloadedit. Locatorpositivo10962B
aggregati=[8dir16index4header252frame7162OAM3520pixel]/8170busreads/2sequenze.
UnicoWine12785fullpair stessi320frame4519→4839/ogni stato-clock/immaginecentrale-
wide/OAM/dispatch/mixedPSGFIFObitwiseHLEaudio coincide; audio[702240,702240,84480,
84480,702164]. Stessi10962B/3520pixelB/2sequenze/764guardhash-mtime; frozen d0995de8
testexe/actualCLIdataset/oldcache/width0, misto native-fallback. Piccolo preflight
wrapperprocess-exit corretto prima di qualsiasiWine, nessun testfunzionaleripetuto.
Source/model/managed/native/oldscene/build/prepare/locator non ripetuti; endpoint
Partyunit con preview animata240x160ispezionato. No allnative/interaunità-allocator/
new32:9claim. Altre119sequenzeordinary target-only; nuovi Linux/USA/Windowsnativo
restano pending. [CLI e scena](../rom_data_unit_obj.md#production-cli-and-positive-ordinary-party-scene).

**ROMDATA-01-UNIT-OBJ-PALETTE-SOURCES / ROMDATA-02-COLOR-MODE-PALETTES
completati nel perimetro fonte/modello EUR/Wine (2026-10-24):** cinquehelper
CF49C/CF4E4/CF51C/CF5A0/CF5DC e70directcall provano30span5750B=2873u16colors+
4u8slice selectors senza overlap52owner. Copiecomplete512B, supplemental
frammentarie,12Bshared e9palette speciali da51proiezionimetadata14 verificate;
nessuna estensione daadiacenza o capguest.86instruction19literal11BLpins passano.
Modello strictlossless/sharedsourceplan e filtroWinecorretto passano165original
pointer/99copy/30stagedreader/54specialcopy/12slice prefix/tutti5750B/eightguards.
Primofiltrofermo da seedr5/r6errato neltest30632; modello senza differenze;
correzione usa MOV/LDRoriginali. DuebuildfrontendWindows senzaavvisi, nessun
productionbuild o vecchia prova ripetuta.764guardhash-mtime/fourcompiledinputs
intatti;33654istruzioni770PCsolo copertura. EUR e57-d56/USAe8-d7 invariati;
nessun fullstate-clock/native/ordinaryclaim. [Contratto e prove](../rom_data_color_palettes.md).

**ROMDATA-03-COLOR-MODE-PALETTES completato nel perimetro mirato EUR/Wine
(2026-10-24):** e58-d57pubblica color-mode-palettes.json/graphics.color-mode-
palettes con metadata14dependency/54file53risorse/tutti52precedenti decodificati/
recordmanifestcoperturaidentici/e57conservato/USAe8-d7. Ownerimmutabileu16colors/
u8selectors perbyte conserva gapraw/priorowner, liveMode/copie/fades/RAM/save-
nativeABI. Primobuild16832fermo3E0061nei costruttoritestprimaWine; trezeroargomenti
aggiunti, nessun vecchiotest eseguito. Build13589core/frontendwarning-free/core
solocompilato/frozenfrontend6725b65c... Wine12913duefiltriesatti firstPASS17guardie/
lifecycle/tutti5750typedB=[1536unit1024companion2286supplemental36shared864special
4selector]/30frammenti/failedattachtransazionale/pubblicazionefallita-create-
reuse-repair-move/unavailableLoad-Undo-backup-Restartsharedowner. Native28percorsi
originale-interprete-compilato stato-clockcompleti; APIaggregati[1532,1536,158,44,
192,7]B sei categorie. Raw5750new+4oldUIBalterati; low8index65535/mode3/liveRAM/
30edges/width/mirrors/BIOSbus/2SWI/1DMA passano. Copiamista2new+20raw+4oldUIB con
addebitiesatti2/4; APIword2new+2raweditati; oldUI40FCB0stessapagina4Bsenza nuovi
addebiti.767buildguard769runtimeguard/ninecompiledinputsintatti;6805istruzioni
451PC5177dispatchsolocopertura. Niente interoallocator-scheduler/ordinaryclaim;
fonte/modello/vecchieprove nonripetuti. [Consumo e prove](../rom_data_color_palettes.md#managed-publication-and-typed-runtime-reads).

**ROMDATA-04-COLOR-MODE-PALETTES-CLI / ROMDATA-05-COLOR-MODE-PALETTES-LUTIA-PAIR
completati nel perimetro EUR/Wine dichiarato (2026-10-24):** Windows28568unico
productionbuildwarning-free/frozena052914b..; canonico hash/mtimeesatti ripristinati,
maincontentinvariato/mtime-onlyCargoinvalidation. Wine22334CLIcreate/reuse-no-write
56gestiti/all54generazioneugualiaccettati/cwdestranea/oldcache, actuale58generation-
XbOkZs/e7abf8bb.. . Partyunit3200newcolors; neutralboot240Wine468380newcolors:
negativiterminali/no fullpairs/niente nuovi input/fixture. Recordesistente Lutia
maploadguida unicoWine229031700positivo1102Baggregati=[512unit512companion78
supplemental0shared0special0selector]/277busreads/4frammenti. Worldmap7701a360../
worldmap-engage-battle.input invariati, nessuna nuova navigazione/payloadedit.
Nuovo guardtest REQUIRE_COLOR_MODE_PALETTES richiede colori typedpositivi;
Windows46600unicobuildfrontendwarning-free/frozen449a1d2c.. . UnicoWine16883fullpair
stessi1700frame4519→6219/ogni stato-clock/immaginecentrale-wide/OAM/dispatch/
mixedPSGFIFObitwiseHLEaudio coincide; audio[3730650,3730650,448800,448800,3730574].
Stessi1102B/4frammenti/828guardie/tencompiledinputsintatti; vecchi9runtimeinputs
immutati. EndpointLutiaNOTICE240x160ispezionato; mistonative-fallback, niente
allnative/interoallocator-scheduler/new32:9. Fonte/modello/managed/native/vecchie
prove non ripetuti. Altri26frammenti ordinary/altreMode/shared-special-selector
restano targeted-only; nuoviLinux-USA-Windowsnativo pending.
[CLI e scena](../rom_data_color_palettes.md#production-cli-and-ordinary-lutia-palette-consumption).

**ROMDATA-01-UNIT-OBJ-EQUIPMENT / ROMDATA-02-UNIT-OBJ-EQUIPMENT completati nel
perimetro fonte/modello EUR/Wine (2026-10-24):** originale9BA84→CE1EC proprietà
3/9 seleziona252item/61sprite124..184, basi0/4/8, parti0/1; overrideespliciti
185/basi0-4 e124-126/base16 inclusi.748celle/86null/258sequenze3439frame:
1911op1/609op6/888op8/31op0, soloop1produce riferimenti grafici.593intervalli
110666B, senza overlap con tutti53owner e58:248directory2992indici69812sequenze
174OAM37440pixel4bpp/1170tile.73instruction/4BL/4literal/2dispatch statici;
modello tipizzato lossless con dipendenzaitem e nullreali, senza capguest.
Unico buildtestWindows warning-free; unico nuovo filtroWine passa al primo
tentativo10guardie/1496prefixcostruttore/86resetnull/258resetsequenze/seicomandi/
280scheduler-DMA/tutti37440pixelB.828guardieprecedenti intatte, settepreimage
compilati conservati; correzione successiva del solo commento action12/13 non
cambia l'implementazione. Nessuna vecchia prova/build/fixture ripetuta.
Pubblicazione gestita/runtime/scenaordinary ancora dafare, EUR e58-d57/USAe8-d7
invariati. Censimento attori normali/alternativi e script49 resta parziale:
nessun prodotto cartesiano animazionescript×sprite. Linux/USA/Windowsnativo da
fare, nessuna misura prestazionale.
[Contratto e prove](../rom_data_unit_obj.md#equipment-poses-source-model).

**ROMDATA-03-UNIT-OBJ-EQUIPMENT completato nel perimetro mirato EUR/Wine
(2026-10-24):** e59-d58pubblica55file54risorse, unit-obj-equipment.json/
graphics.unit-obj-equipment con dipendenza effettiva item-records; tutti53file/
recordrisorsa/coperture precedenti identici, e58 conservato, USAe8-d7. Owner
immutabile legge campi e indici4bpp per i593intervalli110666B senza ROMshadow,
con nullreali e composizione byteperbyte con owner precedenti/rawholes. Task/
comandi/facing/copieRAM/scheduler/DMA/VRAM/save-nativeABI restano originali.
Unico buildtestWindows core/frontend warning-free, core solo compilato. Due
nuovi filtriWine firstPASS17guardie/lifecycle/tutti110666typedB/258sequenze e
26percorsi originali-interprete-compilato con stati-clock completi uguali,
null/parti0-1/fullwordID-animazione/comandi0-6-8/liveRAMframe/scheduler-DMA.
APIcompilate aggregano[32directory24indici6header47frame32OAM1pixel]B; DMAreale
confrontato separatamente.593edges/width/mirror/BIOS/2SWI/1DMA passano; misto
2new+1028raw+8oldOBJ con addebiti2/8 e APIunaligned2new+2raweditati.831guardie
build/833runtime/11sourceinputintatti.1511istruzioni502PC97dispatchsolocopertura,
nessun interoallocator-scheduler/ordinaryclaim; fonte/modello/vecchieprove non
ripetuti. Identitàdataset2c200038../manifestfileSHAaa8bba05.. distinti; CLI e
positiveordinary ancora aperti. Linux/USA/Windowsnativo/new32:9 dafare; no metriche.
[Pubblicazione e consumo](../rom_data_unit_obj.md#equipment-managed-publication-and-runtime-reads).

**ROMDATA-04-UNIT-OBJ-EQUIPMENT-CLI completato (2026-10-24):** unico
Windows85707productionbuildwarning-free/frozen0795d057..; canonico hash/mtime
esatti ripristinati, maincontent invariato/mtime-onlyCargoinvalidation. Wine87287
actualCLIcreate/reuse-no-write55generazione/57gestiti/tutti55ugualiaccettati/
cwdestranea/oldcache; e59generation-nXJuGC/2c200038.. . Sheep1000locator e
Wine22920Fight700locator sono equipment-negative terminali:0tuttecategorie/
0sequenze, niente fullpair/testguardbuild/ripetizione invariata. Vecchie prove
paletteprogrammiscene non sono prova d'uso dell'equipaggiamento. Precondizione
snapshotspell inizialmente assume wrapperdesktop; effettivo rawFFTA-ST/version1/
EUR/payload verificato senza build/guest precedente. F09preview/core/hashpass.
833runtime/891CLI/893Fightguardie/11sourceinput/tutti7623artifactprecedenti intatti.
Ispezione staticariutilizza211callsitecandidati9BA84, non provavita/definizioni
univoche. dump_frame mostraframebufferguest, non necessariamente nativeUI.
[CLI e limiti ordinary](../rom_data_unit_obj.md#equipment-production-cli-and-ordinary-coverage-limits).

**ROMDATA-05-UNIT-OBJ-EQUIPMENT-SCENE completato nel perimetro EUR/Wine dichiarato
(2026-10-25).** L'utente autorizza esplicitamente checkpoint locali da una copia
EUR tramite soli comandi normali, originali/cartucce invariati e artefatti fuori
Git; precedente rifiuto automatico superato, nessuna modifica RAM/HP/status/PC.
Derivazione F09frame8476→8668in192frame normali conferma Action/Fight/alleata
Hildegard. Due caselle vuote e primoA500che si ferma su Do it sono negativi
chiusi, non ripetuti. Input nuovoAat0/48(release4/52) conferma realmente l'attacco:
actuale61CLI500positivo4477B=8dir16index4header276frame2765OBJ1408pixel/twosequenze.
Solo guardia positiva pixel+sequenze nel testscena; unica buildfrontendWindows
warning-free/frozena13b3aa6..; unica coppiaWine500firstPASS frame8668→9168ogni
stato-clock/immagine centrale-wide/OAM/dispatch/audio, contatori equipment identici
alCLI(3269letture). Fonte/modello/managed/native/actualCLI/ordinary equipment
chiusi, altre256sequenzeordinary restano targeted-only. VecchiSheep1000/Fight700/
source/native/scene/build non ripetuti; e61/USAe8invariati. Restano generale
attori/alternate/script/altreposepalette e nuoviLinux/USA/nativeWindows/32:9;
principale01..05parziale/mod06-07dipendenti. Nessuna misura performance/spazio.
[Scena autorizzata e limiti](../rom_data_unit_obj.md#authorized-ordinary-equipment-attack).


**ROMDATA-01-UNIT-OBJ-NORMAL-BATTLE / ROMDATA-02-UNIT-OBJ-NORMAL-BATTLE completati
per la proiezione EUR/Wine dichiarata (2026-10-24):**121IDnormali metadata52/14+
42esplicito,29basi corpo/8parti extra risolvono10980chiavi/8374celle fisiche/
5293nulle/2581sequenze17894frame.150liste676oggetti e3216richieste4bpp conservano
pixel indicizzati e comandi0/1/2/3/6/7/8. Viste condivise146358Binitial+6984B
equipment verificate; nuovi6021intervalli1455474B, nessun altro overlap nei54
owner correnti/nessun limite guest o estensione per adiacenza. Unica build
Windows senza warning/unico filtro Wine nuovo al primoPASS:12guardie/16474
prefissi costruttore sulle8237celle nuove/5293reset nulli/2451reset sequenze/
100comandi/2858scheduler-DMA/tutti1063200Bpixel nuovi/VRAM e guardie intere.
I controlli originali dei vecchi owner non sono ripetuti. Proiezione finita,
non prova di raggiungibilità di ogni combinazione attore/azione. Il prodotto
indiscriminato con gli IDalternativi è respinto: sprite247/animazione104legge
la directory394834come indice, nessun filtro sul numero frame risolve il legame
mancante. Alternate/script49/streamRAM restano aperti. e59-d58/55file54risorse e
USAe8-d7 invariati; prossimo03pubblicazione gestita/consumo tipizzato, poiCLI/
uso ordinario. Linux/USA/nativeWindows/nuovo32:9 da fare, nessuna misura.
[Contratto e prove](../rom_data_unit_obj.md#normal-mode-battle-poses-source-projection-and-typed-model).


**ROMDATA-03-UNIT-OBJ-NORMAL-BATTLE completato nel perimetro EUR/Wine mirato
(2026-10-24):** e60-d59pubblica/consuma unit-obj-normal-battle.json con dipendenze
reali metadata52/14/initial/equipment,56file55risorse/tutti54precedenti record e
filedecodificati invariati/e59conservato/USAe8invariato. Il lettore interseca le
viste semantiche con i6021nuovi intervalli1455474B; nessun owner duplicato o
shadowgrezzo. Una build Windows senza warning/due nuovi filtri Wine al primoPASS:
17guardie/lifecycle/tutti1455474Btipizzati/2451sequenze e32prefissi originali/
interprete/compilato con stati e clock guest uguali. Parti0/1/2/nulli/IDfullword/
RAMlive/comandi0-2-3-6-7-8/dueupload+dueDMA/tutte6classi API/6021bordi/mirror/BIOS/
SWI-DMA passano. Finestramista2nuovi1036grezzi316initial248equipment8OBJprecedenti
preserva addebiti esatti; API disallineata2nuovi+2grezzi modificati passa. Modello/
sorgente e controlli precedenti restano chiusi/non ripetuti. Prossimi04CLIreale
create/riuso-senza-scritture/vecchiacache e05uso ordinario positivo della nuova
famiglia corpo normale; alternate/script/streamRAM eLinux/USA/nativeWindows/
nuovo32:9restano aperti. Nessuna navigazione/nuovocheckpoint o misura performance.
[Contratto e prove](../rom_data_unit_obj.md#normal-battle-managed-publication-and-runtime-reads).


**ROMDATA-04-UNIT-OBJ-NORMAL-BATTLE-CLI / ROMDATA-05-UNIT-OBJ-NORMAL-BATTLE-PAIR
completati nel perimetro EUR/Wine dichiarato (2026-10-24):** unica buildproduzione
Windows senza warning/canonico hash-mtimeesatto ripristinato; CLIreale crea e
riusa senza scritture56filegenerazione/58gestiti, tutti uguali all'e60accettato,
da cwddiversa/vecchiacache. RegistrazioneF09Wait3530esistente/immutata conmovimento
mostri è positiva:842236Bnuovi=[40index10header26826frame815360pixel]/5sequenze.
Un solo guardpositivo test-only/unica buildfrontend/unico fullpair Wine3530al
primoPASS: ogni stato completo/clockguest/immagine centrale-wide/OAM/dispatch e
campione audio mixed/PSG/FIFO/HLEbitwise uguale; contatori esatti uguali alCLI.
Tutti958guardie/14inputcompilati/preimage/8083artefatti precedenti preservati.
Altre2446nuovesequenze restano mirate/non playthrough esaustivo. Figli source/
model/managed/native/CLI/ordinary chiusi, nessuna ripetizione o nuova fixture/
navigazione/checkpoint/payload/save. Alternate/script/streamRAM rimangono aperti;
05equipmentpositivo poi chiuso il25ottobre dopo autorizzazione esplicita e
singola nuova coppiaFight500.
Linux/USA/nativeWindows/nuovo32:9 da fare, nessuna misura prestazionale/spazio.
[Prove e limiti](../rom_data_unit_obj.md#normal-battle-actual-cli-and-ordinary-equivalence).


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
