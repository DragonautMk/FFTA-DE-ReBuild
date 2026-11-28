# Custom UI graphics resources

The original custom UI decoder at `080054a0` reads the stream selected by
the guest. The extracted catalog supplies the encoded stream and selection
data through normal cartridge reads. Original decompression, VRAM transfers
and display setup continue to run in the guest.

## E438 Additional European custom stream source plan

Protected reconstruction `e438-eur-five-custom-stream-source-plan-av4gexqa`
decodes all five `3adfb0` streams to 1,664 bytes each. It records the exact
consumed packet extents, source and output fingerprints, table and original
caller literal. The seven selected spans total 3,061 bytes and have no E301
owner. Two compressed streams end exactly where an E437 embedded-prefix owner
begins; the other three leave one byte between those sources.

The original selector and `080054a0` decoder are identified by E435, but this
plan does not execute those streams or establish ordinary use. Trailing bytes,
other readers, unrestricted guest language values, Linux, USA and native
Windows remain open. No performance comparison was made.

## E499 Additional European custom stream runtime

EUR generation `e312-d311` extends `graphics.ui-custom-banks` with the
five streams selected by the table at `3adfb0`. The catalog stores each
block's encoded bytes and decoded words. The manifest adds five encoded
stream ranges, five selector words and the four-byte table literal at
`13c014`. Those eleven spans account for the seven E438 ranges and their
3,061 bytes. The selector has five physical entries. The original guest
address calculation also applies to language values beyond those entries.

Each original call at `0813bf7c` decodes 1,664 bytes into the guest's
scratch buffer. The next original call at `0813bf88` transfers them to
`0600b180`. The catalog excludes bytes after each consumed stream.
An explicit `e311-d310` dataset selection uses the earlier catalog.

Wine `afav76ye` runs one focused test against the E438 plan and the
E491 manifest. The production command handler creates, reuses and verifies
the dataset. The 155 other resource entries match the predecessor exactly.
The predecessor attaches with its original UI file. A rehashed edit of
the table literal is rejected while the existing runtime owner stays active.

The original selector, decoder and VRAM transfer run for each language
with the new 3,061 bytes inverted in the managed ROM copy. All planned
bytes are read from the extracted catalog. The original decoder consumes
the expected span for each stream and produces the recorded 1,664-byte
output. Raw and managed registers, guest clock, RAM, video memory, OAM and
palette agree. The two selector prefixes for values 5 and 255 preserve
their original address and value. All 31 protected paths pass and Wine
drains. Ordinary invocation, translated dispatch and deferred platforms
remain open.

Cargo succeeded in `ux5zla0w` and reused the canonical test output path
that held the E493 harness. The validation guard retained the replacement
diagnostic. The Wine unit independently verifies the new executable,
checks all 14 immutable build inputs and records both binary hashes.
The previous exact executable has no located local copy. The earlier
E493 through E498 results retain their recorded evidence.

## E500 Literal-selected European custom streams

Protected census `e500-eur-literal-custom-source-plan-ao1dmjoc`
authenticates all 73 retained Thumb BL patterns targeting `080054a0`.
Thirty calls load a fixed source address from a PC-relative word,
including one that preserves it in `r7` across three calls. Those
30 words select 24 distinct custom streams. Independent decoding
determines each consumed extent and output hash.

In `e312-d311`, 18 complete streams and four pointer words have
owners. The unowned projection consists of 26 pointer words and six
streams at `3b1108`, `3ba838`, `3bb394`, `40e28c`, `476214` and
`b75abc`. The six streams decode to 63,744 bytes. The property word
at `3b4c12` remains with `graphics.unit-obj-ffff-reader-fields` and
splits the `3b1108` source projection. The other 47,843 bytes form
33 disjoint spans. The plan retains the reader fingerprints and
source and output hashes with the previous owners.

This evidence bounds the literal arguments in the retained census.
The other 43 call patterns, computed operands and indirect callers
remain outside this projection. Ordinary execution and additional
platforms stay open.

## E501 Literal-selected European graphics runtime

EUR generation `e313-d312` adds six complete decoded streams and 26
original source pointers to `graphics.ui-custom-banks`. Its 33 new
source ranges claim all 47,843 bytes from the E500 plan. The original
property word at `3b4c12` remains owned by
`graphics.unit-obj-ffff-reader-fields`, which the manifest declares
as a dependency. The original decoder receives the complete source
across the two owners. Existing OBJ streams referenced by the new
literal cells retain their original owners.

Wine `e501-eur-literal-ui-wine-ms1i9nsv` passes the production command
handler for preparation, reuse and verification. All 155 other resource
entries and the earlier UI source records match `e312-d311`. The earlier
manifest attaches with its original UI file. Rehashing an edited source
pointer does not allow it to replace the attached resources.

The single targeted test runs the original reader loads for all 26
new pointer cells and the original decoder for each of the six new
streams. One caller loads into `r7` and moves the value to `r1` after
heap setup. The test runs those original instructions with scratch
preallocated. All selected backing bytes, including the two shared
bytes, are inverted in the comparison machine. Each selected read is
served by managed resources. The original outputs match 63,744 bytes
of independently decoded data, and paired registers, guest clock and
RAM match. The OBJ owner serves the two shared bytes during decoding.
Word reads crossing that owner boundary match on all three ROM mirrors.
Protected inputs pass and Wine drains.

Windows cross-build `e501-eur-literal-ui-windows-build-lqcru_bi`
keeps the exact E499 test executable under `work/e499-test.exe` before
refreshing the canonical output. The test scope covers controlled original
readers and decoder output. Ordinary invocation, downstream transfers,
translated execution, Linux, USA and native Windows remain open.

## E502 Computed European decoder sources

Protected source inventory `e502-eur-computed-custom-source-plan-67k4v45y`
accounts for the remaining 43 calls from the E500 census. Thirty-four
select from 16 five-slot language tables. Four select from the two
157-entry alternate BG palette tables. Two select the tag-20 and
tag-22 streams linked from the 162 BG descriptors, and two use the
103-entry relative OBJ directory at `3d63d4`. The remaining call
decodes relative text programs through `08013f8c`.

The audited physical inputs include 80 language streams, 314 alternate
palettes, 50 main BG graphics wrappers, 103 relative OBJ streams and
8,898 compressed programs from the 145 declared relative text archives.
The audit reconstructs the selected boundaries and matches 9,659 occupied
source ranges to the `e313-d312` manifest. It also resolves 40 distinct
source-base literals. Of these, 31 words totaling 124 bytes still come
from the ROM, including 28 language-table words and three alternate
palette roots.

The physical lengths refer to the declared source families. Wider live
index values, arbitrary text archive arguments and callers outside the
retained census still need independent evidence. This is a source audit.

## E503 Source-base literals for the European custom decoder

EUR generation `e314-d313` adds all 31 residual E502 words to
`engine.thumb-pc-literals`. The original Thumb loads receive the same
table base addresses from the extracted file. The 155 other resource
entries, the existing literal records and the stream owners match
`e313-d312`.

Wine `e503-eur-custom-decoder-roots-wine-bc73k4y1` verifies 32 original
reader paths, including two with a shared source word. For each path it
executes the first and last declared table lookups with the selected
index and address registers established. All 64 cases match the original
registers, flags, table results and guest clock with the new 124 bytes
inverted. The trace attributes all 31 words to managed reads, and the
existing Star Cross counter stays at zero during these calls.

The command handler creates, reuses and verifies the new generation.
The earlier manifest attaches with its original literal file, and a
rehashed pointer edit is rejected while the active dataset stays usable.
Wine drains. Windows build `e503-eur-custom-decoder-roots-windows-build-ca54p6yc`
retains the exact E501 executable under `work/e501-test.exe`.
Ordinary invocation, wider index domains, translated execution, Linux,
USA and native Windows remain open.

## Additional European table-selected source contract

`ROMDATA-01-LOCALIZED-TABLE-GRAPHICS` declares eight additional, independently
rooted five-slot language pointer groups at `71e590`, `71e5a4`, `71e5b8`,
`71e5cc`, `71e644`, `71e680`, `71e6bc` and `71e6d0`. They select 40 distinct
custom LZ/RLE banks, all currently outside the managed catalog. The decoder is
original `080054a0`, distinct from the indexed archive decoder `05228/0537c`
and BIOS LZ77. This is a source contract for extending
`graphics.ui-custom-banks`; schema1/e91-d90 and the accepted84-file dataset
remain unchanged in this unit.

Each selector reads the live unsigned byte at `02001f6e`, scales it by four,
adds its root and loads a little-endian bank pointer. All eight actual selector
sequences and fused calls to `054a0` are authenticated. Physical slots0..4
are declared; no language fallback, skipped load or guest cap is added for
outside5/255 addresses. The cached aligned root census has exactly one literal
for each selected root, and every such reference is covered.

The five bank outputs per group are respectively1472,2688,768,4928,1920,7488,
1344 and2368 bytes. Their sizes come from big-endian stream headers and exact
packet production, not distances or alignments. Independent decoding preserves
every token/literal/source byte, exact consumed extent and little-endian output
words. The declared48 disjoint spans contain53507 bank bytes and160 selector
bytes,53667 total, producing114880 decoded bytes. Source SHA256 is
`df69408f23205472246a37ab071670913bdb2ceebe151a91b18fadf9e297d5b4`;
decoded bytes concatenated by increasing bank offset have SHA256
`11ba607260a1ae16c6eced9c1855ba3f98bd41b11d033b5950f3abb66f16c8fd`.

Some callers decode into fixed scratch`02003cb0`; others use allocated task
pointers or task-owned RAM. Original downstream helpers, dynamic destinations
and later scratch work remain distinct from static source ownership. In
particular, the helper at`12c4f8` uses scratch beyond the4928-byte decoded block;
that later RAM work does not establish another ROM source extent. Established
halfword-copy calls show full768/7488/1344-byte transfers for three groups;
full transfer/helper ABI and ordinary liveness for the complete set are pending.

Fourteen authenticated static windows (ten new/four reused),48 instruction/
literal pins and111 hash/mtime guards pass. No span overlaps the83 accepted
owners/50522 source spans; all84 accepted files and actual immutable inputs
remain unchanged. Private evidence lives under ignored
`target/rom-data-localized-table-graphics/`. This unit performs no build,
Wine/game run, fixture creation, navigation or older acceptance replay.

The neighboring`71e5e0` table points to five already owned UI banks; its table
alias remains a separate unowned criterion. Seven other nearby raw-looking
groups, indirect references/global extent, gaps/tails and live helper contexts
remain unclassified. Strict table modeling, new-bank original-decoder/selector
Wine proof, managed/runtime/native-clock integration, actual CLI and ordinary
use remain pending. Linux/USA/native Windows/new32:9/mod phases/fullROMTODO
remain pending; no performance or storage campaign is included.

## Typed table model and new original-decoder proof

`ROMDATA-02-LOCALIZED-TABLE-GRAPHICS` adds
`gba_core::rom_data::ui_graphics::table_selected`, reusing the established
`Graphic`/custom LZ implementation. Its strict JSON stores eight tables with
five contiguous `BankPointer` records each and40 ordered banks with exact
encoded bytes/decoded words. Validation checks physical root/order/count,
contiguous pointer positions, unique bank linkage, group-specific output sizes,
stream consistency and disjoint source spans. Exact EUR ROM/reader fingerprints
and source verification remain distinct from editable shape validation: a
coherently edited literal/output passes shape and fails source authentication.

Physical `select` returns no bank for language5/255, while `encoded_view` retains
the original raw address. The separate new Catalog is not yet called by the
existing managed UI Catalog, so e91 and all old16 banks/two inline switches
remain unchanged until the runtime integration unit.

One warning-free Windows GNU core build uses600 frozen sources: the accepted
598 ordinary snapshot plus two model/test leaves and one UI module addition.
One focused Wine filter passes23 shape/JSON/source guards,40 new original
`054a0` calls and56 actual selector prefixes (eight tables × languages0..4/5/255).
All53667 source bytes/48 spans/114880 decoded bytes match the independent model.
Every new call matches its complete output and guards, consumed source cursor,
produced/returned size, restored SP/R4..R11, complete EWRAM, IWRAM outside the
original40-byte stack scratch, VRAM/OAM/palette and final ROM bytes. The1335470
executed instructions are coverage only.26 model guards/111 source guards,
600 frozen/598 accepted source preimages and three current implementation paths
remain exact; previous decoder/native/ordinary/CLI acceptance is not repeated.

An offline source metadata correction records actual`23b2e→14522c` control
`0150` (`a8<<1`),336 FastSet words/1344 bytes. An earlier private downstream
label used the equivalent halfword count`02a0`; original opcodes, source
extents, decoded bytes and model inputs were unchanged, so no proof was rerun.
The guarded original label and correction receipt are both retained.

Managed publication/runtime/native-clock, actual CLI and positive ordinary use
of these40 banks remain pending. Full downstream helper ABI/live contexts,
neighboring table alias/raw groups/global extent/indirect/tails and deferred
platforms remain open; no game checkpoint or navigation is added by this unit.

## Managed table-selected graphics and runtime consumption

The e127-d126 regression test verifies the exact generated manifest file set
and reports its actual revision, 100 resources and 101 files. In EUR Wine the
complete test passed publication/interruption, 14 attachment guards, 22 native
state/clock cases and three transfer comparisons, including Load/Undo with the
resource files unavailable. A separate guarded unit confirmed its result and
inputs. The original validation unit remains FAIL because its cleanup guard
found the test-created `.ffstate`; its work and diagnostics are retained.
Evidence: `target/validation/units/rvr15-table-graphics-e127-retry2-iri_b8pe/`
and `target/validation/units/rvr15-table-graphics-result-check-qey5s95l/`.
This is targeted Wine evidence; ordinary table-extension reachability and
native Windows, Linux and USA coverage remain open.

`ROMDATA-03-04-LOCALIZED-TABLE-GRAPHICS-RUNTIME` enables the new table Catalog
inside the existing `graphics.ui-custom-banks` resource at index15. EUR is now
schema1/extractor92/decoder91. The dataset still has84 files/83 resource IDs;
only `ui-graphics.json` and its resource record change. All82 other resource
records/files, the prior16 complete banks by source offset, the two inline
selectors and shared OBJ links remain exact. The UI resource contains56 banks,
ten selectors and106 source bindings.

The eight raw-language groups use existing `Selector` storage with stable
`table-<root>` IDs and no fallback. Forty bank spans and40 four-byte pointer
bindings add80 runtime spans whose byte union equals the48 canonical source
spans/53667 bytes exactly. This splits only pointer provenance, adding no bytes
or ownership. The dedicated table model retains original raw encoded queries.
Immutable bindings compose typed literal bytes from decoded words with exact
header/token metadata, earlier indexed owners and raw ROM gaps. Original
`054a0`, scratch writes, live task/helper work and uploads continue executing;
no decompression shortcut or native/save ABI change is introduced.

Six `table_graphics_stats` counters identify new-family read/source-byte/
decoded-literal/selector-byte totals and distinct bank/group usage. Existing
six UI counters retain their meanings, including the historical non-English
localized subset. Complete new-family coverage is53667 source bytes,
24328 decoded literal bytes,160 selector bytes,40 banks and eight groups.

One601-source Windows host-test build from the accepted600-model snapshot plus
one test leaf is warning-free. One reduced Wine filter passes publication and
changed-UI-file interruption,14 failed-attachment guards/two full-set guards,
22 complete-state/clock cases comparing raw interpreter, typed interpreter and
generated execution, and complete new-source bus/lifecycle checks. The22 cases
include12 controlled loads, one new-bank original decoder call and nine raw
selector suffixes in three stop-safe native families. All53667 new ROM source
bytes and20 prior indexed bytes are altered only after Load/setup in isolated
probes. Actual callbacks serve499 new bytes/five prior indexed bytes, including
three new bytes through `interp_one` fallback; these are served counts, not a
distinct-byte union or an all-direct-native claim. The356 visited instructions
and71 dispatches are coverage only.

Every new source byte/literal/selector/bank/group and all48 canonical boundaries,
widths/mirrors, prior-indexed/raw/old-UI composition, unavailable-file Load/slot
Undo and three mixed DMA/CpuSet/CpuFastSet full-state/clock cases pass. The267
preservation guards cover601 frozen/600 model sources, seven current paths,
84 accepted old files and actual immutable inputs. Existing parameter observer
source stays current and is excluded only from the frozen host module, matching
the accepted baseline. No old decoder/native/ordinary acceptance or navigation
is repeated. Actual production CLI and positive ordinary use of the40-bank
extension remain pending, as do full helper ABI/live effects/global extent/
indirect/tails and deferred platforms. Private evidence is retained under
`target/rom-data-localized-table-graphics-runtime/`.

## Actual table-extension CLI registration

`ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-CLI` passes one warning-free production
Windows GNU build from the same601 accepted runtime sources without overlays.
The canonical executable hash/mtime is restored in a finally block. One Wine
orchestration executes three successful commands from an unrelated working
directory: dataset creation, no-write reuse, then one neutral frame on a copy
of the retained normal24755 checkpoint. All84 generation files equal the
accepted runtime dataset, all83 logged resource ID/hash pairs match, all86
managed paths retain their hash/mtime and the old native cache loads28859 blocks.

Guest24755→24756 has all six new-family counters zero. This proves registration
and compatibility only; positive ordinary use and full scene equivalence of
the40-bank extension remain pending.202 preservation guards cover601CLI/601
runtime sources,84 accepted files and ROM/normal checkpoint/input/cache/TinyCC/
canonical executable. No new checkpoint derivation/navigation/older acceptance
or metric is performed. Private evidence and the selected31c0f298 production
executable are retained under `target/rom-data-localized-table-graphics-cli/`.

## Ordinary table-extension reachability remains partial

`ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY` has two closed negative bounds.
The exact `244d6→2c8c4` initializer/root`71e6d0` link and a retained normal
Fight prediction image selected a64-frame target-confirm input from the
authenticated23851 checkpoint. The same e92 production CLI exits0 at23915,
with all six new-family counters zero. The static initializer link does not
prove that confirming a target initializes the bank again.

A distinct `71e590` accepted bank preview spells COMBO MENU/COMMAND;
original`30608` is called by the`30828` constructor and`30fda` redraw path,
with a task context distinct from the forecast object. A copied normal23611
Action checkpoint and two separated Down presses/one A press select a bounded
160-frame attempt to open Combo. That same CLI also exits0 with all six
counters zero. The headless PPMs omit separate HUD/menu layers, so they do not
prove that Combo was reached or that the prediction was absent. No endpoint
state is saved/adopted and no missing menu state is inferred.

Both probes preserve all83 resource ID/hash pairs,86 managed paths, original
checkpoints/inputs/ROM/cache/TinyCC and canonical executable. Only normal keys
on copies are used. No observer build, positive guard, full pair, longer-bound
retry, RNG resampling or old combat route recreation follows either negative.
Source/model/runtime/registration acceptance stays closed. These two earlier
bounds did not establish positive ordinary use or full scene equivalence. Blind
gameplay discovery was stopped. Private diagnostics stay under
`target/rom-data-localized-table-graphics-ordinary/`.

A later source-guided selection uses the retained EUR White Flowers entry before
battle initialization. Original task registration references callback `08024491`;
its state-zero branch calls the `0802c8c4` initializer associated with root
`71e6d0`. This source link motivates the scene but does not identify which groups
were read. From the recorded Giza Plains mission menu, the existing normal A
input runs frames 19409–20109. The e128 production CLI reports 2,627 table
reads, 2,755 source bytes, 1,118 decoded literal bytes, 12 selector bytes,
three banks and three groups. These are aggregate counters; exact group or bank
identities are not attributed by this measurement.

A separate e128 native raw/managed pair using the same protected checkpoint,
input and dataset passes all 700 frames: full serialized state and clock,
central and wide images, OAM, dispatch and five audio streams match at each
frame. The positive CLI and pair are separate executions; the pair itself did
not instrument group identities. Both guarded validation units passed and
retired successful scratch. Evidence:
`target/validation/units/e128-localized-ui-white-flowers-entry-7ewe6mwj/` and
`e128-localized-ui-white-flowers-pair-2rcuaiy1/`. This closes the scoped ordinary
trigger and paired proof, not all 40 banks, eight groups, selectors, consumers,
Linux, USA or native Windows.

A later E275 Wine inventory of the same retained continuation is documented
separately in the [E362 selected-reader audit](rom_data_runtime.md#e362-white-flowers-rom-read-inventory).
It records broader ROM addresses without attributing them to the table groups.
[E366](rom_data_runtime.md#e366-white-flowers-localized-indexed-archive-runtime)
serves five separate indexed archives, their pointer group and caller literals;
one archive is consumed in that retained continuation. Other ordinary language
scenes, readers and platforms remain open.

EUR schema1/extractor21/decoder20 introduced fixed `ui-graphics.json`, resource
`graphics.ui-custom-banks`, format `ui-custom-banks-v1-json`. The declared
catalog has nine raw BE-size custom LZ/RLE banks, ten independently rooted
literal/decode/upload paths, nine disjoint source ranges/57038encoded bytes
and84160decoded bytes. Eight unique sources feed BG VRAM; one source feeds
OBJ VRAM through two callers. Other indirect/custom sources remain outside
this declared catalog.

## Source contract

The earlier constant-count OBJ upload filter intentionally omitted callers
whose copy count derives from decoder r0. Original decoder `080054a0` returns
the declared decoded byte length. New callers shift/mask that result into
CpuSet/FastSet control, preserving full guest decoding and transfer work.
Bounded analysis reuses the73retained decoder callers, propagates only this
independently proved return value and checks both BG/OBJ VRAM destinations.
Its23transfer references/18streams include10references/nine sources absent
from the earlier39OBJbank catalog. New original caller disassemblies and
independent token/source/output hashes establish the selected scope.

The raw source offsets are `3bade0`, `43bc28`, `b6b434`, `b6d074`, `b6e848`,
`b6f9c0`, `b70ea8`, `b7233c` and `b7a9d0`. None overlaps any earlier resource.
The two banks atb70ea8/b7233care exactly adjacent; provenance retains separate
owners. Source43bc28produces2944bytes but original caller08ed9ccopies only544
bytes to BG VRAM. Other selected transfers copy the complete decoded extent,
including b7a9d0's25664bytes to OBJ VRAM. Decoder-derived counts, selected
scratch pointers and partial prefixes remain original instructions.

The model reuses `obj_graphics::Graphic`'s exact little-endian decoded u32words
and consumed encoded metadata. This shares the codec/storage contract, without
changing the older OBJ resource ID or file bytes. Internal mixed subregions
remain exact words; no universal pure-pixel/RGBA interpretation is invented.
Strict JSON and bounded decoding reject unknown fields, truncation/trailing
bytes, invalid history, wrong decoded words, ordering and overlap. Exact ROM
root/literal/BL validation and source equality reject coherent encoded/decoded
edits. The per-bank32KiB output/128KiB encoded guards and16MiB file-read guard
bound malformed inputs; no extracted-space acceptance budget is introduced.

## Managed and runtime behavior

All17files/16resources are authenticated before transactional attachment and
contribute to dataset identity. All15earlier decoded files remain byte-identical,
e20/older generations stay retained and USA remains e8-d7. Source bindings
project literal byte positions from typed decoded words; exact header/token
positions retain authenticated encoded metadata. CPU/DMA/native and guest-BIOS
bus reads use this source view, merging crossing reads with existing owners.
Every original decoder/count/scratch/copy instruction and clock remains active.
PPU reads live VRAM after those uploads; no added host ROM graphics consumer or
decompression shortcut is introduced. Nonserialized family and per-bank
source/literal counters prove ownership, preserving save/native identity and
ABI. Immutable ownership survives Load/Undo/backup/restart with files unavailable.

## Verification

One reduced EUR Wine original-source check passes independent source/output
hashes, strict deterministic JSON/source/bounds/coherent-edit rejection and
all ten actual decoder→upload paths. It verifies exact original r7consumed
cursor/r9produced count/r0returned length, source/destination/control registers,
full scratch output/guards and whole VRAM, including the544of2944byte partial
copy.1368457instructions are coverage only. The first new-source fixture seeded
r5as scratch at143f0cinstead of inheriting the preceding original mask literal
143f6c; that failed check is retained. The corrected fixture uses the actual
mask, and only that failed new-source oracle was repeated. No old bank oracle
was reopened.

Two focused EUR Wine integration tests pass publication interruption/create/
reuse/repair/move/e20retention, equality of all15previous decoded files,
coherent payload/missing-bank/source/hash/path guards and transactional rejection,
and unavailable-copy Load/Undo/backup/restart. All ten actual interpreter/native
paths with all nine raw spans altered match complete states/clocks;68906compiled
source bytes/39861literal bytes and every bank's positive counter prove typed
ownership. All source bytes/edges/widths/mirrors, real-BIOS bus, CpuSet/FastSet,
32typed DMA bytes and reads crossing adjacent new banks pass. Test and
production Windows builds are warning-free, canonical executable restored and
protected originals/e20data/cache/loader unchanged.

Actual CLI create/no-write reuse from unrelated cwd with old native cache
passes. The first retained320-frame Party→Unit replay reads0new bytes; it is
coverage-negative and is not used as an ordinary-use acceptance. Its final
image is an Equip/Abilities/Jobs submenu. The new bank43bc28preview instead
contains CLAN MBR/stat labels, and a bounded normal worldmapStart→Down→A input
opens Clan. Actual CLI replay320frames consumes1642source bytes/764typed
literal bytes in1579reads, all attributed to43bc28. One320-frame ordinary
Wine pair then matches every complete state/clock/central-wide image/OAM/
dispatch/audio sample with the same positive new-resource counters. Final
Clan status image is inspected; no RAM fixture edit, old source/native/lifecycle
or old-scene repeat was used to locate this new-family scene.

Private evidence stays under ignored `target/rom-data-return-length-graphics/`:
`new-ui-bank-inventory.json`, `caller-*.txt`, `windows-original-result.json`,
`windows-original-europe/result.json`, `windows-managed-results.json`,
`windows-managed-europe/*-result.json`, `windows-cli-summary.json`,
`windows-cli-clan.json`, `windows-scene-result.json`,
`windows-clan-ui-graphics-full-state-audio.log` and build logs. New Linux/USA/native Windows, other callers/scenes and broader GUI/archive/
mod phases remain pending. No performance/startup/RSS/space campaign,
AndroidPort change, commit or push is included.

## Localized UI selectors in e22

EUR schema1/extractor22/decoder21 extends the same file/resource to16banks,
two five-entry selector groups and two links to existing OBJ banks. The original
nine bank records and all15other decoded files stay identical; e21and earlier
generations remain retained. There are still17files/16resources. The UI resource
now explicitly depends on graphics.obj-custom-banks as well as OBJ dimensions;
linked banks must exist in the authenticated parsed OBJ resource.

Original Clan switch0808ed28selects43bc28/44d22c/45ed38/46fdc8/4818b8for
English/French/German/Italian/Spanish. It reads live language byte02001f6eand
uses English for values above4. Switch08143cbaselects
b7d9e4/b7e310/b7edfc/b7f874/b80140; it instead skips localized uploads for
values above4. Its5056byte CpuFastSet OBJ copy and2688byte CpuSet BG copy
exactly cover7744decoded bytes. The last two banks stay owned by the existing
OBJ resource; they are referenced rather than copied into UI JSON. English
Clan likewise reuses the existing43bc28record and its544byte partial upload.

The seven new banks own14396encoded bytes/35008decoded bytes. Ten4byte asset
pointer literals add40bytes, making17new disjoint ranges/14436bytes. The whole
UI catalog owns26ranges/71474source bytes and119168decoded bytes. Code jump
tables remain code; selector records authenticate only their asset pointers.
Exact root instruction/jump-target checks retain both original fallback rules.
Strict JSON, ordered/nonoverlapping sources, owner presence and source equality
reject coherent pointer/fallback/owner changes. Runtime typed pointer reads and
new-source/literal counters distinguish selector use from localized payload use.
Live language selection and every original decoder/copy/count/clock remain.

One reduced Wine source oracle passes14original selector paths(two groups,
values0..4/5/255), all seven new decoder→upload paths and isolated2688byte BG
tails for the two already proved shared OBJ banks. Exact source cursor, decoded
return length, transfer controls, scratch guards and whole VRAM match independent
inventory. It preserves all nine old records and verifies strict JSON/source/
selector/fallback/shared-owner guards. No old decoder campaign was repeated.

Two focused Wine integration checks pass publication interruption/create/reuse/
repair/move/e21retention, all15other decoded-file equality, coherent payload/
selector/fallback/shared alias/source/dependency/path transactional rejection,
and unavailable-copy Load/Undo/backup/restart. Native execution covers14selector
paths/seven new producers/two isolated shared tails with17new spans altered,
matching complete states/clocks. It consumes48compiled selector bytes,
14310new-source bytes/6449new literal bytes and every new bank positively.
All new bytes/edges/widths/mirrors/real-BIOS bus/CpuSet/FastSet and4typed DMA
selector bytes pass. Initial duplicate test-module compile failure is corrected.
Initial native fixture let chained full-producer blocks overshoot selector-only
stops; a separate selector-only table excludes those three stops. Failed evidence
is retained; only the failed native criterion is repeated, without runtime or
original-source proof changes. Test/production builds are warning-free and the
canonical executable is restored.

Actual Wine CLI create/no-write reuse/old-cache replay from unrelated cwd passes.
The first320-frame Clan locator uses only the prior English bank and0new
selector/localized bytes, retained as negative evidence with no full pair.
One bounded1200-frame cold-boot replay reuses existing demo input and positively
reads4selector bytes/2347new b7d9e4source bytes/1124new typed literal bytes.
One1200-frame ordinary Wine pair matches every complete state/clock/central-wide
image/OAM/dispatch and mixed/PSG/FIFO/HLE audio stream with the same positive
counters. Final intro-transition image is inspected. This English intro scene
proves new data use; other ordinary language scenes remain pending. No direct
RAM/language edit, new recording or closed old oracle repeat was used.

Private e22evidence stays under ignored target/rom-data-ui-localized:
localized-inventory.json, selector-*.txt, windows-original-result.json,
windows-original-europe/result.json, windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-cli-summary.json, windows-cli-boot-intro.json, windows-scene-result.json,
windows-boot-intro-localized-full-state-audio.log and Windows build logs.
Protected original ROM/states/inputs/e21data/cache/loader/canonical stay intact.
New Linux/USA/native Windows, other indirect sources and wider TODO gates stay
pending. No performance/startup/RSS/space campaign or AndroidPort change.

## European alias of the existing clan banks

The separate table at `71e5e0` contains five little-endian pointers to the same
banks used by the existing `clan` selector. Each bank decodes to 2,944 bytes;
the 20 table bytes do not overlap any source binding in the current EUR e94
manifest. The banks are already managed, but this table is not. Its source
SHA256 is `1e9b0d6368856eecea893011919583717b2699959a2cee45f93c8b23f23c60c9`.

An aligned EUR ROM census found five literal references at `023c70`, `03278c`,
`04fd3c`, `05e770` and `070fcc`; each has one corresponding PC-relative Thumb
load in its literal range. Six selector sequences, including two in the last
reader, load the live unsigned byte at `02001f6e`, multiply it by four, read
the table and call original decoder `054a0`. No fallback or limit for values
outside the five declared physical slots is implied. Five source windows were
authenticated against the ROM, including two disassembled under Wine; the
ROM, current manifest/UI file and windows remained unchanged. Private evidence
is in `target/rom-data-localized-table-alias/`.

This contract covers the identified direct references, not every indirect
reader, ordinary reachability or the complete transfer helper ABI. The next
integration must add the table's bytes while reusing the existing bank objects
and preserving the old inline selector. Model, managed publication, runtime
consumption and Linux/USA/native Windows verification remain open. No gameplay
or performance measurement was performed.

## Managed European clan table alias

The EUR dataset now adds the five pointers at `71e5e0` to the existing
`graphics.ui-custom-banks` resource as `clan-table-alias`, with no duplicated
bank objects. The table bytes are authenticated against the original source,
and their targets are checked against the existing `clan` selector. The alias
has no fallback; the original guest code still computes addresses from the live
language byte. Extractor/decoder revisions are e95-d94. The previous inline
selector, all bank contents and the eight-table usage counters retain their
existing meaning.

One isolated Windows build and focused Wine test published the dataset and
verified the other 84 resource records and files against e94. It checked all
20 alias bytes, boundary reads and mirrors, then compared full serialized state
and clock for six original interpreter selector prefixes with language values
0–5 and 255. The five alias pointers were poisoned only in the typed machine.
A coherently edited UI file was refused without replacing the attached owner or
changing the machine state. Protected inputs and the frozen source snapshot
remained unchanged. The first build failed because the new test lacked a trait
import; the corrected build and Wine test passed. Evidence is retained in
`target/rom-data-clan-table-alias-retry/`.

This initial prefix comparison used the interpreter and did not establish an
ordinary game scene. The separate checks below cover selected translated
prefixes, production CLI registration and one ordinary Bank entry; the full
helper ABI, other paths, Linux, USA and native Windows remain open. No performance
measurement was performed.

The full managed clan alias filter was subsequently rerun with e127-d126 under
EUR Wine. It verifies preservation of the 85-resource e94-d93 predecessor,
all 20 alias bytes and mixed boundaries, 42 selector-prefix cases with full
state/clock equality, and transactional rejection of coherent tampering. Its
generated manifest has 100 resources. Guarded ROM, predecessor and test inputs
remained unchanged. Evidence: `target/validation/units/rvr16-clan-alias-e127-0mpw8bok/`.
The retained test artifact matches this test's source snapshot; six unrelated
source files and two other test modules differ from the current workspace.
This is targeted EUR/Wine evidence, not a new ordinary scene or native Windows,
Linux or USA validation.

## Translated clan alias selector prefixes

A separate Wine check compiled the six original selector prefixes and compared
translated execution using the e95 typed owner against the raw-ROM interpreter.
For 24 cases (language values 0, 4, 5 and 255 at each site), complete serialized
state and clock matched. The original table bytes were altered in the typed
machine; its runtime counters confirm 48 alias bytes served by typed callbacks
for the in-range cases. The accepted e95 dataset, ROM and frozen sources remained
unchanged. Private build and test evidence is in
`target/rom-data-clan-table-alias-native/`.

This proves the selected instruction prefixes, not the complete decoder or
transfer ABI, every language in translated execution, or ordinary gameplay.
Production CLI registration and one positive ordinary Bank entry are checked
below; other paths, Linux, USA and native Windows remain pending. No performance measurement was performed.

## Clan alias production CLI

An isolated Windows production build from the selected tracked revision passes
one Wine create/reuse/startup sequence for e95-d94. The generated 86 files
match the accepted e95 dataset, including the UI resource's five alias pointer
bindings. Reuse and one neutral frame leave all 88 managed files unchanged; the
runtime reports all 85 resource identities and loads the existing native cache.
ROM, input, checkpoint, accepted dataset, cache, canonical executable and
frozen sources remain unchanged. Evidence is in
`target/rom-data-clan-table-alias-cli/`.

This verifies registration and startup for the selected frame. A separate
ordinary Bank entry and scene comparison are documented below; Linux, USA and
native Windows remain to verify; no performance measurement was performed.

## Clan alias ordinary boot observation

The original `023bcc` branch can read the alias, but its input gate and an
earlier positive aggregate for another raw-transfer resource do not establish
that this branch runs during boot. One isolated Windows audit build and Wine
observation used the existing 1,200-frame clean-boot recording with e95-d94.
The bus filter recorded no requests for `71e5e0..71e5f4`, with no dropped or
truncated events; the ROM, input, cache, dataset and frozen sources remained
unchanged. Evidence is in `target/rom-data-clan-table-alias-observe/`.

This is a bounded negative for that recording and instrumented bus readers:
folded constants and unselected direct host readers are outside the audit.
It does not prove global absence. A separate Bank entry below subsequently
shows positive use and a matching scene comparison; other paths, Linux, USA
and native Windows checks remain open. No guest state was forced and no performance measurement ran.

## Ordinary clan alias use during Bank entry

A source-backed investigation traced callback `05aee4` through `05af9e` to
`05e728` and its original alias load at `05e73a`. The preserved Bank entry
snapshot at guest frame 3354 and the existing final 120-frame neutral input
lead to the registered callback without changing guest memory. A focused Wine
bus audit recorded two four-byte **Typed** reads of `71e5e0`, at guest frames
3366 and 3409. The latter has interpreter instruction `0805e73a`; the first
has no attributed instruction. No requests were dropped or truncated.

A separate Wine pair using the same initial state, input, e95 dataset and
native cache matched all 120 complete serialized states and clocks, central and
wide frames, OAM, dispatch counts and five audio outputs. Both arms use the
same native cache, and the positive audit is a separate observation. ROM,
input, checkpoint, cache, accepted dataset and frozen sources remained
unchanged. Private evidence is in `target/rom-data-clan-alias-bank-entry/`.

This establishes one ordinary EUR Bank entry, not every language or caller, the
full helper ABI or independent native-cache equivalence. Linux, USA and native
Windows remain to verify. No performance measurement was performed.

## E622 European fixed-width screen maps

The original helper `081441f8` takes a tilemap pointer, a destination,
a halfword count, a width and a tile value to add. It reads consecutive
halfwords and writes each row at a stride of 32 cells. The ten direct
calls use a zero tile value and read nine distinct maps. Two localized
callers use the same 30 by 20 map at `b8d354`. The existing e22 selector
uses the stored language byte at `02001f6e`.

Eight complete maps occupy `b8c778..b8dcb4`, between the existing raw
palette and DMA transfer owners. In source order they contain 12 by 16,
10 by 17, 15 by 14, 22 by 11, 10 by 32, 12 by 32 and two 30 by 20
rectangles. The same helper reads a 17 by 13 map at `6df2dc..6df496`
immediately after the E610 background screen maps. The extents come
from the original counts and widths. The eight bytes after the 17 by
13 map are outside this reader.

Wine `oi7b1y59` attests 964 original instruction halfwords and checks
the 2,939 distinct map cells and ten source literals against e363. The
ownership audit finds 5,918 unowned bytes in twelve spans. All 15 input
guards pass and Wine drains. Result SHA-256
`497567aca5ab4690d3628adcc991681799a504755b4ecbb360f41be9032fecc4`.
Runtime publication, ordinary scenes, translated paths and deferred
platforms are tracked in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E623 European fixed-width screen map runtime

Dataset `e364-d363` publishes the E622 projection in
`graphics.fixed-screen-maps`, stored as `fixed-screen-maps.bin`.
The 5,918-byte file supplies both map regions and ten source literals
at their original guest addresses. The two localized callers continue
to share the map at `b8d354`. The e363 dataset still attaches, and all
164 earlier resources are byte-identical.

Wine `h7701ipw` executes the original fetches and row copier
from all ten caller sites with altered ROM backing. Controlled
destinations verify 3,539 total copied cells, including all 2,939
distinct cells, and 2,721 unwritten guard cells. Each path matches
the guest clock. The complete final state matches, with 7,118 managed
bytes read and 1,152 mixed and mirrored boundary checks. Two invalid
candidates leave the current attachment in place.

Windows GNU build `pwcxixfo` and the Wine CLI creation, verification
and reuse pass. Stage `kdqkk4qk` keeps the 166-file generation.
All 17 Wine input guards pass, and the server drains. Stage result
SHA-256 `2d6317648a350c0fa85fcc7976a36a235058cb1f5558f495fcab1f0b9d2a7623`; manifest SHA-256
`370363197c5245254fab98acf895733224f34f80c7fa61b96b483935ac859d40`.
Ordinary scene and translated execution, Linux, USA and native Windows
remain in the [recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E624 European localized layout bank

Four original builders select five language tables each using the
stored language byte at `02001f6e`. The tables at `39810c..398d90`
contain 267 twelve-byte records. The builder loops provide all 20
counts, which match the physical table boundaries. Each builder uses
four copies of the language selection sequence. The row readers load
an OBJ list pointer and halfwords at offsets 4 and 6. The later layout
loop at `0809367c` reads the halfword at offset 8 with a twelve-byte
stride. All 267 rows share the one-object list at `398102`.

The preceding selector at `08092746` reads a five-word allocation size
table. The task setup selects four-object and three-object lists at
`3980d4` and `3980ee`, and the original draw path uses the same counted
six-byte OBJ format as `398102`. Two alignment bytes at `39810a` are
outside the three lists. The four builders select individual fields
from the five four-byte records at `398d90`. The transition reader at
`0809573a` bounds 17 records at `398da4`, with complementary values
from 16 to 0 and 0 to 16.

Wine `gg6jocgf` verifies 6,218 original halfwords, all 20 layout bounds
and 88 literal sites. The e364 ownership audit finds 3,718 unowned bytes
in 91 spans. They cover the 3,366-byte source region with its two
alignment bytes excluded, plus the literals. All 16 input guards pass
and Wine drains. Result SHA-256
`6fe639b8b18027de64d92bd6570cbc4f161800f427d89ce6736654676496727f`.
Runtime publication, scene triggers, translated readers and deferred
platforms are tracked in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E625 European localized layout runtime

Dataset `e365-d364` publishes the E624 projection in
`graphics.localized-layout-tables`, stored as `localized-layout-tables.bin`.
The 3,718-byte file supplies all 91 source spans at their original guest
addresses. The 267 rows continue to share one OBJ list. The e364 dataset
still attaches, and all 165 earlier resources are byte-identical.

Wine `_2d3h_eq` enters the original builders with the five stored
language values. The comparisons cover all 20 layouts and 267 rows,
the allocation table, the four per-language parameters, the secondary
position loop, eight objects drawn from the three OBJ lists, 17
transition rows and 88 literal sites. With the new source bytes
altered, the original instructions read 6,650 managed bytes.
Complete guest state and clock match, and 8,736 mixed and mirrored
boundary checks pass. Two invalid candidates leave the attachment
in place.

Windows GNU build `92ac4p6_` and the Wine CLI subreceipt from
`roudsfiq` verify creation, verification and reuse of the 167 files
retained in `v6z9rj49`. The first reader test reached its target on the
fixture's final step. Build `x3wytrre` raises the test budget and reuses
the verified dataset and CLI. All 21 retry input guards pass, and Wine
drains. Retry result SHA-256 `79f7f3ae1977c8c093a66be65b447bf125a42ce3d43134aef00f84bcb0bfa020` and manifest SHA-256
`3848a386380b56b64e1aa741cc4f92f69afb5982fdaeb6fe4dd8158fd2675c7d`.
Ordinary scenes, translated execution, additional indirect callers,
Linux, USA and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E628 European indexed UI tilemaps

The original copier at `08036a54` takes a source, destination, height
and width. It adds the selected tile value and advances the destination
by the caller's stride between rows. The selector at `0803ae9c` uses
the stored language byte at `02001f6e`. Value 1 selects two tilemaps,
one 27 by 2 and one 27 by 1. The other branch selects a 27 by 2 map.

The four contiguous ten-slot directories at `395a44..395ae4` select
20 distinct 15 by 2 maps and eight distinct 15 by 1 maps. The first
directory serves two original copier calls and a separate fill path.
Duplicates in the four directories preserve the original addresses.
The directories end at the following independent language table.

The 31 maps contain 855 cells spanning `b543a8..b54a56`.
Two alignment bytes precede the next transfer bank at `b54a58`.
Wine `stqxruch` verifies 971 original halfwords, all 40 directory
slots and eight literal words. The e366 ownership audit finds
1,902 unowned bytes in eight spans. All 18 input guards pass and
Wine drains. Result SHA-256
`d72e37815c41e945bfa1c245ba560cd8fd6b8a8d8c2ba1fba4bd9a2ec40cacfe`.
Runtime publication, ordinary producer bounds, translated readers
and deferred platforms are tracked in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E629 European indexed UI tilemap runtime

Dataset `e367-d366` publishes the 1,902 E628 bytes as
`graphics.indexed-ui-tilemaps` in `indexed-ui-tilemaps.bin`.
The file serves the 31 maps, four directories and eight source
literals at their original addresses. Shared targets keep the
same addresses. The e366 dataset still attaches, and all 167
earlier resources are byte-identical.

Wine `kuqcbc2p` executes the original three direct copy sites and
all ten physical indices at each of the five indexed sites.
With the selected ROM backing altered, the callers copy 1,485
cells including all 855 distinct cells. The original 27-cell stride,
tile addend and untouched cells match. Complete guest state and clock,
3,382 managed bytes and 768 mixed and mirrored boundary reads
pass. Two invalid candidates leave the attachment active.

Windows GNU build `l23bh7kw` and Wine CLI creation, verification and
reuse pass. Stage `9m_lcvkw` retains 169 files. All 17 Wine input guards
pass, and the server drains. Stage result SHA-256 `48818e49f6b429de3bafd2c1ef6e210f48968b16cf48be7a20debe6354b41e21`;
manifest SHA-256 `361ec88752acbcf8e9c66844dcedc463f9f157a3853c4207ecec38a8c789cc94`.
Ordinary triggers, index producers, translated execution, Linux, USA
and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E632 European queued UI map bank

Seven original calls to `08000be4` select the five tilemaps in
`3ba358..3ba838` through nine literal sites. The map at `3ba358`
has 20 columns and three rows. Four callers share the 30 by 4 map
at `3ba3d0`. The final two maps have ten columns and seven rows.
A branch selects between their two distinct addresses.

The copies at `0812d666` use 19 columns and a 38-byte source
stride. The next original root at `3ba720` bounds 16 physical rows
starting at `3ba4c0`. Both original branches use the live row count
from task offset 16. The physical height does not change that live
reader. Every map uses a 64-byte destination stride.

Wine `zcysqkbo` attests 3,273 original halfwords and verifies the
624 cells and all nine literal words. The e368 ownership audit finds
1,284 unowned bytes in nine spans. The neighboring White Flowers
transfer and compressed UI bank keep their existing owners. All 15
input guards pass and Wine drains. Result SHA-256
`46481134b1dd7d42643470d759c2a76e126dda1e70bcd0339ab0e6a174ed3bd7`.
Runtime publication, the live row domain, ordinary scenes, translated
readers and deferred platforms are tracked in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E633 European queued UI map runtime

Dataset `e369-d368` publishes the 1,284 E632 bytes as
`graphics.queued-ui-maps` in `queued-ui-maps.bin`. The original map
addresses, nine literal words and the live row count remain in the
guest paths. e368 still attaches, and all 169 earlier resources
are byte-identical.

Wine `uo7t0zos` executes the nine original source paths and eleven
DMA copies with altered backing. Each live-height branch is checked
with one row and all sixteen physical rows. Both conditional maps
are selected through the original branch. All 1,326 copied cells,
source and destination strides, destination guards, complete guest
state and clock match. The original queues consume 2,700
managed bytes. The 864 mixed and mirrored boundary reads and
rejection of two invalid candidates pass.

Windows GNU build `8llomu57` and Wine CLI creation, verification
and reuse pass. Stage `xh8fmuqm` retains the 171-file generation.
All 17 input guards pass, and the server drains. Stage result
SHA-256 `9adbfdb5f35c581028b2e970cd7e3858f9c67e6a1f220424151db608c59713a5`; manifest SHA-256
`a3ed0a5bbb9afca9a98e2efa9faf843f07a8c3769a1e15789b2100afdf7943cb`.
Live row bounds, ordinary triggers, translated readers, Linux, USA
and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E653 European queued matrix sources

Six original paths select `448278` when the saved language byte is
1 and `437194` otherwise. Each physical map contains seven rows of 26
halfwords with a 52-byte source pitch. The original queue calls at
`0802f328` and `0802f732` use two-row slices starting at rows zero
and three. The edge reader at `0802fa86` uses row two, and the loop at
`08030134` copies all 52 cells in rows five and six into the task
buffer. The neighboring indexed archives independently bound both maps
at `437300` and `4483e4`.

The original copy at `0802df02` takes 144 bytes from `3ba048`.
It writes four 18-cell rows at task offset 916. The seven-row queue
at `0802e0a0` starts at task offset 808, so the initializer supplies its
last four rows before further task edits. The existing Firaga matrix
ends at `3ba048`, and the next observed tilemap begins at `3ba0d8`.

Wine `tuz9kwhe` attested 817 original halfwords and reported 924
unowned bytes against e378. The first verifier stopped after the Wine
outputs because one interval list used lists and the other used tuples.
Protected reconciliation `uhq5u4zf` checks the original outputs and
passes all 23 input guards without repeating the Wine commands. The
source result SHA-256 is
`ce81038f7df560e187098905ebc20c79aa47914f166efebb89c559efe3f8ad75`.
The three maps account for 436 cells and 872 bytes. The 13 literal
words bring the complete projection to 924 bytes across ten spans.
Sixteen call sites in the 39-call census still need source ancestry.
The recovery checklist tracks ordinary and translated use, the
remaining producers and deferred platform checks.

## E654 European queued matrices at runtime

`e379-d378` publishes the 924 E653 bytes as
`graphics.queued-matrix-sources` in `queued-matrix-sources.bin`.
The cartridge bus maps the three source blocks and their original
literal words at the original addresses. The manifest links the
existing Firaga copied matrices for the adjacent initializer.

Windows GNU build `yk73m09a` and Wine `oe5aqd3x` pass. The test
alters all 924 source bytes in the managed ROM copy. It exercises the
six original selectors with all five saved language IDs. The two
source choices reach the original two-row queue argument suffix, and
both complete maps pass DMA through the original queue helper.
The original CPU loop copies 104 cells from the last two rows across
both maps. The original initializer copies its 144 bytes into task RAM
and the original queued transfer reads the resulting seven-row map
with its first three rows supplied by the fixture.

All destination guards, 960 mixed and mirrored boundary checks,
complete guest state and clock match. The original paths read 1,436
managed bytes. The e378 predecessor still attaches and both invalid
attachments are rejected. The Wine CLI creates, verifies and reuses
181 files with all 179 previous resources byte-exact. All 17 Wine
input guards pass and the server drains. Stage `w6058qkm` retains the
dataset with manifest SHA-256
`13bcba7fe32e9fa0eba10d32c8f67f91f3e10813dc142f4bc07c3556db58fb78`.

Ordinary task preparation, translated execution, native Linux, the
USA ROM and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E655 European queued grid shapes

The `006270` callback copies 16-byte descriptors from the event-zero
message into its context at offset `0x30`. On the later preparation
event it reads the selected descriptor's flags, obtains the allocation
size from `370e90` and stores the allocator result at context offset
`0x8`. The resize helper at `006910` uses the same allocator and field.
The fill and render paths write cells into that buffer before the
32-row queue call at `006622`. Its four halfword shapes cover one,
two horizontal, two vertical or four 32-by-32 screen blocks. Each
block transfers 32 halfwords per row with a 64-byte destination
pitch and a source pitch of twice the shape's width. The byte branch
at `00666c` instead submits linear chunks from the same buffer.

The table at `370e90..370eb0` contains eight records of two dimension
bytes and one little-endian allocation size. Four shapes have two
bytes per cell and four have one. The original reads through literals
`006358`, `0064b0` and `006628` select the allocation size, dimensions
and DMA source pitch. The table ends at the existing IWRAM helper vector.
The 32 table bytes and 12 literal bytes are unowned in `e379-d378`.

Wine `2bxsvzw_` attests 645 original halfwords, passes the ownership
join and drains its server. The source result SHA-256 is
`1927cc6c1e7e3f614c84a5e681d989c966c0512b5933dfac064d7fe3d59a1193`.
E588's accepted event-zero copy establishes the incoming descriptor
shape. Its live writer, the flag modes actually used and the immutable
inputs to rendering still require original producer evidence.
Publication, runtime use and deferred checks are in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E656 European menu selection banks

The interval `39576c..3959a4` contains 12 separately rooted tables.
The original handlers near `03e02e` and `03e908` select five
36-byte rows at one root and eight 8-byte rows at four roots. The
entry builder at `03e8cc` uses five 8-byte descriptors at
`395a1c..395a44`. Their nonzero selector fields refer to those
five banks with one-based indices. The existing E219 indexed words
occupy the intervening `3959a4..395a1c`, and the indexed tilemap
directories start at `395a44`.

The original matching loops at `03c2bc` and `03c308` establish
28, five, six and eight rows at `395908`, `3958bc`, `3958d0` and
`3958e8` respectively. Four earlier dispatch branches read the
halfword at offset 2 of the selected four-byte row. The high-bit
cases use the separate 22-halfword physical table at `395978`.
The label selector at `03e1c6` reads byte pairs from 8-byte rows at
`395874`, and the numerical handlers at `04974c` and `04989c`
read fields of four-byte rows at `395860`. The rooted neighbors
bound the physical tables. Live indices from task state still
require producer evidence.

Wine `hwoyyk2k` attests 1,415 original halfwords and matches 32
reader sites to 30 distinct pointer words. The 13 physical tables
and their pointer words account for 728 unowned bytes in 27 spans
against e379. The join confirms the existing owner of all 120
intervening bytes. All 14 guards pass and Wine drains.
Source result SHA-256
`a1e4ddfb37ad91fffa049632d19211c79025aecd141ba328f65c0363b97e2c48`.
Runtime publication, ordinary paths and deferred platforms are
tracked in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E657 European menu selection banks at runtime

`e380-d379` publishes the 728 E656 bytes as
`gameplay.menu-selection-banks` in `menu-selection-banks.bin`.
The cartridge bus serves all 27 original spans, including the
descriptor and pointer words. The intervening indexed words keep
their independent owner. `e379-d378` still attaches.

Windows GNU build `u3uiazeh` and Wine `sjms_oc_` pass. With the
backing bytes altered, the original readers load all 32 pointers,
exercise selected fields in all 13 tables, match all 28 pairs and
traverse the complete no-match loops. The original code also reads
the high-bit values, label pairs and numerical adjustments. It
fetches 3,051 managed bytes in these cases. All 30 neighboring
indexed words and 2,592 mixed and mirrored boundary checks pass.
Complete state and guest clock match, and both invalid attachments
leave the active dataset in place.

The Wine CLI creates, verifies and reuses 182 files. Each of the
180 earlier resources matches e379 byte-for-byte. All 17 input
guards pass and the Wine server drains. Stage `vjtb2m5a` retains
the dataset with manifest SHA-256
`085955893b3fdf27f8f06571f085a35bab8fb5339e0eb0608c447e5679746af0`.

Ordinary menu triggers and live index domains, translated execution,
Linux, USA and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E658 European world-map label tables and OBJ lists

The two world-map label handlers near `037b7c` and `037ddc` read
the saved language byte at `02001f6e`. The byte table at
`395ae4..395b7a` contains five 30-entry rows of variant indices.
The parallel table at `395b7a..395ca6` contains five rows of
30 halfwords. The original callers halve the selected halfword
to obtain the starting program and add five to the variant index
to obtain the requested count. Both use the original indexed
decoder at `00537c`.

The 150 slices partition all 1,218 programs in the five archives
selected by `3955d8`. The 35,686 archive bytes, their directory
and both archive root words retain their existing
`graphics.indexed-archives` owner. The two zero bytes at
`395ca6..395ca8` precede the independently owned world-node bank.

The variant selects one of the eight pointers at
`39573c..39575c`. Slots 6 and 7 share the final list. The parent
field at offset `0x678` supplies the original draw wrapper at
`001f6c`. Its renderer at `001500` reads a halfword count and
advances by six bytes per OBJ entry. The seven physical lists
contain 18 entries and exactly fill `3956c2..39573c`.

Wine `6778uuyd` attests 499 original halfwords and joins the
complete selected source. The two language tables, variant
directory, OBJ lists and six reader literal words contribute
628 unowned bytes across ten spans against e380. All 13 guards
pass and the server drains. The source result SHA-256 is
`390af148ec21ea0c9403dcd15216ae3fc2c2a5819dda4424cb3704e9334f6c79`.

The second path uses the original bounded node helper at
`03784c`, which returns zero or a one-based index through 30.
The first path forms its index from the view byte at offset
`0x8c` and masks it to seven bits. Its live writer and domain,
runtime publication, ordinary and translated use and deferred
platforms remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E659 European world-map labels at runtime

`e381-d380` adds the two selection tables, eight variant slots,
seven counted OBJ lists and six reader literals to the typed
`graphics.indexed-archives` resource. Its new fields reproduce all
628 E658 source bytes at the original addresses. The original
archive groups and their encoded source bytes are preserved.

Windows GNU build `foxfdoz3` and Wine `hawsy1fk` pass. With the
selected ROM bytes altered, both original paths select all 150
language and node pairs and pass the expected arguments to the
original decoder. Five decoder calls reproduce the expected
bytes. The original OBJ draw routine emits 18 shadow OAM entries
from the seven lists. All eight pointer slots, 960 boundary
checks across the three ROM mirrors, the raw alignment halfword
and complete guest state and clock match the reference behavior.
The e380 dataset still attaches. Both invalid attachments leave
the current dataset in place.

The Wine CLI creates, verifies and reuses all 182 files. The
180 other resources are byte-identical to e380. All 17 guards
pass and Wine drains. Stage `gi448653` retains the dataset with
manifest SHA-256
`faa14e5a117fd9f368d546889350fa751b5e44379c14f3712e344d8b403b8a58`.
Ordinary and translated use, the first reader's live index domain,
Linux, USA and native Windows remain in the
[recovery checklist](development/ROM_DATA_VALIDATION_RECOVERY_TODO.md).

## E643 European fixed 30-by-20 screen-map source

The original caller at `0804f894` invokes the loader at `0804fd68`.
Its copy loop at `0804fd8e` reads exactly 600 halfwords from
`3b53fc..3b58ac` in 20 rows of 30. The source row pitch is 60 bytes.
The destination pitch is 64 bytes. The original code sets bit 8 in the
halfword at `03000940` after the copy. Three literal words select the map,
the destination context and that shadow-register address. The preceding
tileset literal remains with its custom UI resource.

A bounded aligned Thumb census through `146000` finds three candidates
with the copy and dimension immediates. `0802be50` uses the screen
dimensions to clip a variable-width board and takes its source from a
caller argument. The census records all 11 direct callers in this
interval. At `080e9890` the immediate is a stack offset. The scope
covers this fixed loop form. Derived sources and other copy forms
require separate proofs.

Wine source unit `jbniu5w1` attests 301 original halfwords and the e373
ownership audit. The selected graph contains 1,216 bytes. Of these,
the 1,200 cell bytes and 12 literal bytes remain unowned. All 14 input
guards pass, and the Wine server drains. Result SHA-256
`426cd10b844e1f5eec84f993a14dcaac3c14291987382a77b3e93c5e69d909f0`.

The compressed archive's adjacent tile attribute table at
`3b58ac..3b5acc` still needs its original consumer. Runtime publication
and altered-source copy parity follow in E644. The e373 interval audit
ranks `493c05..4954dc` as the largest nonuniform gap between declared
owners beginning in the data area. Two values, `bb` and `cc`, account for
6,350 of its 6,359 bytes. The interval still needs an original source
root and extent before classification.

## E644 European full-screen map at runtime

`e374-d373` publishes the 600 cells and three literal words through
`graphics.full-screen-map` in `full-screen-map.bin`. Its 1,212 bytes
retain their original addresses. The preceding tileset literal remains
owned by `graphics.ui-custom-banks`.

Windows build `_piq8s1k` and Wine unit `gd0efuq7` pass. The original
copy reads every packed byte with altered ROM backing. All 600 copied
halfwords, 42 destination guards and the shadow-register bit match.
The 192 ROM mirror boundary checks pass, and complete guest state and
clock match. e373 attaches, and two invalid candidates leave the active
attachment in place.

The CLI creates, verifies and reuses 176 files. All 174 previous resources
are byte-identical. All 17 protected inputs pass, and the Wine server
drains and releases its lease. Stage `lm4fie3o` retains the dataset.
Result SHA-256 `6f962ba3ff4629419e116400e169319770480f23119b251734a923ac165eba6a`.

The test supplies the destination context and executes the original copy.
The ordinary screen route, other copy forms, the neighboring attribute
reader, translated execution and deferred platforms remain in recovery.

The static dispatcher at `0804f830` calls initializer `0804f990` before
`0804fd68` on successive indexed states. Its masked low two bits from
`02002fc2` choose the 160-byte palette source: values 0 and 3 select
`40fcb0`, value 1 selects `411310`, and value 2 selects `410830`.
The source is copied to shadow RAM at `03003860` through the original
copy dispatcher, and its first halfword is then cleared. The reader and
caller audits `viewer-screen-init-audit-lz4xdapt` and
`viewer-screen-caller-audit-n8dyno0p` preserve the disassembly. The viewer
binds these sources only when their palette catalog and source hashes match.
It renders this initial palette as a source preview; zero-index transparency,
other layers, later palette changes and a named scene are not established.
