# Custom UI graphics resources

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
Source/model/runtime/registration acceptance stays closed. Positive ordinary
use/full scene equivalence remains open until a concrete constructor/context
is identified; blind gameplay discovery is stopped. Private diagnostics stay
under `target/rom-data-localized-table-graphics-ordinary/`.

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
