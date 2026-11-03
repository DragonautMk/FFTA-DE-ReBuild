# European static gameplay records

Current EUR schema 1 / extractor 27 / decoder 26 adds fixed
`linked-metadata4.json`, resource `gameplay.linked-metadata4`, format
`linked-metadata4-v1-json`: 20 two-pair records / 80 source bytes. All 22 files /
21 resources authenticate together; all 20 earlier decoded files and e26
generations remain unchanged, USA stays e8-d7. Actual parsed metadata52 indices
and pair selectors validate against each other. Typed selector/value bytes feed
original CPU/DMA/native code, preserving counting, contiguous walking, unsigned
values/signed comparisons, tally wrapping, live ability scans, clock and
save/native ABI. Two focused Wine integration checks and a positive 540-frame Change Jobs
full-state/image/audio/actual CLI pair pass with 32 selector / 6 value bytes.

EUR schema 1 / extractor 26 / decoder 25 added fixed
`linked-metadata52.json`, resource `gameplay.linked-metadata52`, format
`linked-metadata52-v1-json`: 116 records / 6032 source bytes. All 21 files /
20 resources authenticate together; all 19 prior decoded files and e25
generations remain unchanged, USA stays e8-d7. Typed header/routing/packed/tail/
opaque fields feed original CPU/DMA/native reads while the original getter,
alternate indices, redirect behavior, live records, guest clock and save/native
ABI stay intact. Two focused Wine integration checks and one positive 540-frame Change Jobs
full-state/image/audio/actual CLI pair pass with 664 newly typed source bytes. Detailed packed-field semantics retain structural
names.

EUR schema 1 / extractor 25 / decoder 24 added fixed
`linked-metadata16.json`, resource `gameplay.linked-metadata16`, format
`linked-metadata16-v1-json`: 128 sixteen-byte records / 2048 source bytes. The
full dataset has 20 files / 19 resources; all 18 previous decoded files and e24
generations remain unchanged, USA stays e8-d7. Original CPU/DMA/native readers
consume typed fields while live-list lookup, indices, RNG state, arithmetic,
shifts, guest clock and save/native ABI remain original. Focused Wine source,
managed/native/lifecycle and one positive 520-frame Clan Mission Item
full-state/image/audio/actual CLI pair pass. That ordinary scene reads 6 typed
display bytes; broader field names remain structural.

EUR schema1/extractor24/decoder23added fixed `item-links.json`, resource
`gameplay.item-links`, format `item-links-v1-json`:225counted pair records/
4500source bytes. It depends on the authenticated item-record resource and
checks all376parsed item owners. That e24dataset has19files/18resources;
every17previous decoded file and e23/older generation remains unchanged.
Typed count/selector/value/opaque bytes feed original CPU/DMA/native logic.
Retained reduced Wine source/managed/native/lifecycle and ordinary headgear
scene/CLI acceptance remain valid. New Linux/USA/native Windows and wider
gameplay catalogs stay pending.

EUR schema1/extractor23/decoder22introduced fixed `items.json`, resource
`gameplay.item-records`, format `item-records-v1-json`. Its376static records
own12032source bytes; that e23dataset had18files/17resources and retained all16
previous decoded files/e22. USA stays e8-d7. Live unit/inventory/task state and
property modifiers remain original RAM/code. The source contract and retained
Wine acceptance for that earlier resource remain below.

## Item record source

The next sixteen-byte metadata table has a separate
[source-only contract](#sixteen-byte-linked-metadata-source-contract). Its
semantic family remains unproved; managed/runtime acceptance follows below.

Following the original shop list071030through category classifier0cf794roots
property reader080ce1ec. Its checked literal080ce20ccontains086e0bd0and
original instructions compute base + (u16item_id <<5). Independent list builder
080cf820uses the same source via literal080cf8dc; it enumerates1..361or
362..375for the separate mission category. Original inventory helper080cec58
independently uses boundary IDs361/375and mission count14, but returns live
EWRAM inventory02001940, not static item records.

The next independently rooted table is086e3ad0. Reader08072134loads the item
base through08072150, fetches item byte1dand uses it as a20byte index into that
next table through08072154. Together the independent roots and enumerated IDs
bound376records ×32bytes =12032bytes at ROM6e0bd0..6e3ad0(exclusive), including
record0without inventing a meaning for it. This span overlaps no e22resource.
A raw pointer-word search finds70matches to the item root; these are discovery
candidates, not70verified code readers.

## Original field access contract

Property reader080ce1ectruncates item ID to u16and property ID to u8. It has
no local item-ID bounds check. Properties0..18dispatch through a code jump
table at080ce214; other property values return0. That table remains code.

Properties0/1/2read unsigned halfwords at offsets00/02/04;3..8read unsigned
bytes08..0d;9reads halfword0e;10..17read signed bytes10..17;18reads unsigned
byte1d. Every32byte record must be preserved, including fields not exposed by
this getter. A lossless candidate representation separates four halfwords00..06,
eight bytes08..0f, eight signed bytes10..17and eight bytes18..1f. Semantic names
for most fields remain unproved; do not infer item names, prices, abilities or
job rules solely from plausible numeric values.

Properties10..13are dynamic. Original getter080ccf00reads live EWRAM bytes at
02002030+u16index. With byte13nonzero, the property reader returns the raw
signed field. With that byte zero, item IDs0033/012e/0094/011eadd live bytes
1/3/2/4to properties10/11/12/13respectively, then apply the original unsigned
comparison/clamp to255. Negative raw values and that unsigned clamp must not
be normalized into a new rule. Live modifiers remain guest code/state, never
frozen into extracted ROM records.

## Evidence and next criterion

Private read-only evidence stays under ignored target/rom-data-gameplay-items:
shop-inventory-071030.txt, callee-080cf794.txt, callee-080ce1ec.txt,
callee-080cec3c.txt, callee-080cec58.txt, item-reader-*.txt,
item-live-modifier-reader.txt, inventory-items.py, item-inventory.json and
item-records-independent.json. Independent static checks verify the exact EUR
hash, selected LDR literals/stride instructions/code targets, both enumeration
bounds, nonoverlap and complete12032byte typed-field roundtrip.

The main-record static inventory is closed. The item-linked20byte table has
the source and managed/runtime acceptance below; jobs/abilities/laws/AI, other
item semantics/readers and regional catalogs remain separate work.

## Typed file and direct consumption

Strict JSON stores four u16halfwords00..06, eight u8bytes08..0f, eight i8fields
10..17and eight u8tail bytes18..1fper record. Fixed array types, exact source
extent/count and original root/stride/enumeration/code-target checks preserve
all32bytes and record0. Source equality rejects coherent payload edits,
reordering, missing/extra records and foreign ROM images. A2MiB file-read guard
bounds malformed inputs; this is not an extracted-space optimization budget.

All18files authenticate before transactional attachment and contribute to whole
identity. Managed preparation retains e22and earlier generations through create/
reuse/repair/move. CPU/DMA/native and guest-BIOS reads project the exact source
byte from typed fields, including reads crossing record/family/source edges.
Source/typed-prefix-halfword/signed/remaining-byte counters are nonserialized.
Original property dispatch, live02002030bonuses, signed loads, unsigned255clamp,
secondary table reads and inventory updates keep executing with their clocks.
No property routine or host ROM consumer is replaced. Save/native ABI and base
ROM identity remain; ownership survives Load/Undo/backup/restart with dataset
files unavailable. PPU/audio continue consuming original live outputs.

## Reduced Wine verification

One original-source check passes14371calls: all376records/19properties under
both live-modifier branches, special-item bonuses, u16/u8truncation and invalid
property handling; independent12032byte roundtrip/source/hash/strict JSON guards
and stack/callee/live-RAM guards pass. The shipped signed fields all happen to
be nonnegative. The first fixture wrongly assumed a negative exists and failed
before executing original code; corrected evidence is retained.240isolated
in-memory negative-byte cases then prove sign extension, bonuses and unsigned
clamp without changing/authenticating/publishing real source data.354477original
instructions/139distinct PCs are coverage only. An initial permission-review
timeout created no process; one permitted retry started the test.

Two focused integration checks pass publication interruption/create/reuse/
repair/move/e22retention, all16old decoded files unchanged, coherent prefix/
signed/tail/reorder/source/dependency/path/role transactional rejection, and
unavailable-copy Load/Undo/backup/restart.86sampled original/interpreter/native
calls with only12032new raw bytes altered match complete states/clocks. Native
reads consume100typed source bytes(24prefix-halfword/40signed/36remaining),
with live bonuses/clamp/truncation active. All bytes/edges/widths/mirrors/real-BIOS
bus/CpuSet/FastSet and4typed DMA bytes pass. The full source-only14371+240oracle
is not repeated. Windows test/production builds are warning-free; canonical
executable restored. Private evidence additionally includes windows-original-
result.json, windows-original-europe/{items.json,result.json},
windows-managed-results.json, windows-managed-europe/{managed-result.json,
native-result.json} and Windows build logs in the same ignored directory.

Actual CLI through Wine creates/reuses e23without dataset writes from unrelated
cwd using the existing native cache. The retained320-frame worldmap→Party→Unit
input positively consumes40new source bytes, all signed fields. One320-frame
ordinary Wine pair matches every complete state/clock/central-wide image/OAM/
dispatch and mixed/PSG/FIFO/HLE audio sample with the same positive consumption.
Prefix/tail ordinary use is not claimed; targeted original/native checks prove
those fields separately. Final Unit menu image is inspected. No new input,
direct RAM/language fixture edit or extra locator was needed. Protected ROM/
states/inputs/e22dataset/cache/loader/canonical remain unchanged; new Linux/USA/
native Windows, wider GUI/archive/gameplay/mod gates remain pending. No
performance/startup/RSS/space campaign or AndroidPort change. Additional evidence:
windows-cli-summary.json, windows-scene-result.json,
windows-unit-items-full-state-audio.log and party-unit-items.png.

## Item-linked pair source contract

A standalone typed source catalog now owns ROM6e3ad0..6e4c64(exclusive):
225records ×20bytes =4500bytes. Twelve checked LDR literals root the table;
next table6e4c64is independently loaded at080d5e80through080d5e9cwith16byte
index arithmetic. All376item records select by byte1d, referencing220distinct
rows0..224; unreferenced rows214..217/219are retained. This source directly
adjoins the existing item table and overlaps no e23resource.

Each record contains a u8count0..9, opaque u8byte1and nine fixed pairs of
u8selector/u8value. All20bytes remain exact, including every slot beyond count.
The current header byte1and all unused pairs are zero; the model preserves them
rather than treating them as absent. There are472active pairs, eight with
selectorff. Semantic names beyond the original byte contracts remain unproved.
Links::validate_items requires every parsed item to have a table owner; raw
for_item uses the original u16item-ID truncation. The source-only phase kept the managed dataset at e23; e24integration is
described below.

Original item-chain080d0cb2uses property18and20byte arithmetic. Bounded query
080d0cc2reads count, ignores selector0, compares a nonzero pair selector with
live unit byte7and pair value withr5, and stops on the first match. It does not
read slots beyond count. Original normal publication prefix080d09ccORs80into
live unit byte40+value before its downstream call. A context mismatch with
selectorffinstead follows080d09f0and stores the value at live unit byte3d.
These are original bounded prefixes, not a reconstructed full job/ability
system; later calls, the52byte table at6e5464and live unit state remain separate.

One focused Wine source check passes378original item selectors(includingu16
truncation),1394queries covering all225rows,464normal and8isolatedffpublication
prefixes.2244cases/61906instructions/66PCs, independent full4500byte roundtrip,
source/JSON/count/extent/owner/coherent opaque/unused/active/reorder guards and
live-unit/outer-RAM/stack guards pass. Four isolated in-memory queries prove
ignored zero selector, duplicate first match and nonzero unused slots without
publishing/authenticating synthetic data. A separate eight-case Wine check
covers the original selector load/context mismatch/ffbranch/unit store from
080d09c4with live unit selector2;80instructions, without repeating the main
source oracle. Both Windows test builds are warning-free. No performance
measurement, new Linux/USA/native Windows or ordinary pair-table use is claimed.

Private evidence under ignored target/rom-data-item-links: links-inventory.json,
link-records-independent.json, inventory-links.py, candidate-literal-readers.json,
item-detail-072158.txt, candidate-reader-*.txt, secondary-reader-*.txt,
windows-original-result.json, windows-original-europe/{item-links.json,result.json,
sentinel-gate-result.json}, windows-sentinel-gate-result.json and Windows logs.
Source-only and ordinary earlier-item proofs remain closed; the subsequent
e24integration below does not repeat those oracles. Wider gameplay/maps/audio/
GUI/archive/mod work and new Linux/USA verification remain pending.

## Managed and direct item-link consumption

EUR e24-d23appends item-links.json to the existing source-authenticated set,
using fixed filenames and an explicit gameplay.item-records dependency. Every
17previous decoded file remains byte-identical; e23/older generations remain
retained. Strict file parsing and source/whole-manifest equality precede
transactional attachment; owner validation uses the actual parsed item resource.
A2MiB file-read guard bounds malformed inputs, not extracted-space acceptance.

Typed count/selector/value/opaque fields project every source byte for original
CPU/DMA/native/guest-BIOS reads, including slots beyond count. Live unit byte7,
normal bit7flags, FFbyte3dpublication, later job/ability calls and original
branch/copy/clock work remain active. Immutable session ownership survives
Load/Undo/backup/restart with dataset files unavailable. Nonserialized counters
distinguish count/selector/value/opaque consumption; save/native ABI remains.

Two focused EUR Wine checks pass publication interruption/create/reuse/repair/
move/e23retention, all17old file equality, coherent count/opaque/unused/reorder/
source/dependency/path/role transactional guards and unavailable-copy lifecycle.
62sampled actual selector/query/publication paths with all4500new raw bytes
altered match complete interpreter/native states/clocks. Native compiled reads
consume36count/114selector/56value bytes, covering all eight observed count
shapes and actual FFmismatch gates. Original bounded paths do not read opaque
byte1; all source bytes including that field are separately bus-proved.
Selector fixtures use a separate native table excluding theirD0CC2stop, which
is also a query entry; no chained overshoot or source-oracle repeat is needed.

All4500bytes/edges/widths/mirrors/real-BIOS bus/CpuSet/FastSet and4typed DMA
bytes pass. A narrow word crossing the adjacent old/new source boundary with
both raw spans altered proves2typed old item bytes+2typed new link bytes.
Windows test/production builds are warning-free, canonical executable restored,
protected e23data/originals/cache/loader intact. New integration evidence in
ignored target/rom-data-item-links includes windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json} and Windows
build logs. Source-only final evidence remains source-final-summary.json and
source-final-source-hashes.json.

Actual Wine CLI creates/reuses e24without dataset writes from unrelated cwd,
using the existing native cache. The first400-frame ordinary Equipment slot
entry reads0new link bytes; it is retained as negative evidence without a full
pair/repeat. Its image shows five equipped slots. A justified normal Down/A
extension opens an equipment inventory, and the560-frame final image explicitly
shows headgear(Wizard Hat/Cross Helm/Diamond Helm/Green Beret/Feather Cap).
Private artifact names retain weapon-list as a locator; no weapon-list scene
is claimed from that filename.

The560-frame CLI reads574new source bytes(287count/205selector/82value,0opaque).
One560-frame ordinary Wine pair matches every complete state/clock/central-wide
image/OAM/dispatch and mixed/PSG/FIFO/HLE audio sample with identical positive
count/selector/value use. No source/native/lifecycle proof is repeated, and no
RAM fixture edit is used. Protected original ROM/states/inputs/e23data/cache/
loader/canonical remain intact. Additional evidence: windows-cli-summary.json,
windows-cli-weapon-list.json, windows-scene-result.json,
windows-weapon-list-item-links-full-state-audio.log and weapon-list.png.
New Linux/USA/native Windows/wider gameplay/GUI/archive/mod gates stay pending;
no performance/startup/RSS/space campaign or AndroidPort change is included.


## Sixteen-byte linked metadata source contract

Standalone `metadata16::Catalog` preserves128records/2048bytes at EUR
6e4c64..6e5464: a raw u16alias and fourteen u8bytes02..0f. Twenty-three
checked original LDRreferences and sixteen-byte indexing identify the root;
CBEDA/CBEF0independently roots the following52byte table and fixes the end.
Independent field reconstruction roundtrips every byte without e24overlap.
Strict JSON rejects unknown fields, overflow, wrong array/count/extent and
coherent source edits. No law/mission/ability name is inferred from numeric
patterns. At source-only closure this standalone model added no managed file or resource;
EUR remained e24-d23/19files/18resources and USA e8-d7. Subsequent e25direct
consumption is documented below.

Rows1..127have alias375+row. Original39070loads that halfword and uses
(base-375*16)+alias*16to read byte2; alias0would point before this table, so
its caller precondition is unproved and no zero fallback is invented.
Original161e4reads byte2directly; both bounded paths produce display index0
for zero and394+value otherwise. This is a display helper index, not a proved
text/string ID. The opaque bytes and record0remain intact.

Four original readers use live linked records: head020028c8points to nodes
with record pointer+0/next+8; identity is byte0|((byte1&3)<<8). Record bytes4/5
select two metadata rows, with zero meaning skip. D5E20truncates input IDu16,
resolves the list and sums unsigned metadata byte0d, returning0on missing/null.
D5EA0does the same lookup for byte0e: zero yields the other value, then1yields
the other value; when both exceed1it calls the original02868LCG, updates live
IWRAM030034b0and selects using the low bit of the returned value. The source
proof checks the actual RNGstate transition, not a frozen choice.

D6704/D67D4instead read activeu16ID02003c28and return0when inactive. For a
nonzero IDthey require an existing linked record; unlike the first two readers,
no missing-record fallback is assumed. D6704truncates the selectoru8and sums
bytes09..0cfor selectors10..13, wrappingu8after each addition. D67D4tests raw
byte07using original register-ASR(value,selector-1)&1and ORs the two results;
zero/large selector shift behavior stays exact. All live records/indices/list
links and RNGremain guest state/code. The preceding D5DC0function has an
unproved r7base and is excluded from this table's contract.

One reduced Wine original-code check passes2581real-source cases and19isolated
synthetic cases:128direct display paths,127nonzero alias paths, every row in
pair sums/choice/flags, first/second list matches, first-two-reader missing/null,
inactive IDs, input truncation and selector edges. Shipped byte0ehas no value1;
isolated machine-owned fixtures prove that priority branch plus unsigned sum
wrap and flag shifts. Synthetic records fail authentication and are never
published. Complete live-list/callee/outer-RAM/stack guards and strict source/
JSON/extent/coherent-edit checks pass.156038instructions/319distinct PCs are
coverage counts only. Initial Windows test compilation omitted the Bus trait
import; retained diagnostics all report that fixture issue. Corrected build16512
is warning-free; Wine16242passes the new test only. All27protected ROM/e24file/
cache/loader/canonical/candidate/fixture hashes remain unchanged.

Evidence under ignored target/rom-data-metadata16: metadata16-inventory.json,
metadata16-records-independent.json, inventory-metadata16.py, reader-*.txt,
windows-{initial-failed-tests-build,tests-build}.{jsonl,log}, windows-original.log,
windows-original-result.json and windows-original-europe/{metadata16.json,
result.json}. Managed preparation, direct CPU/DMA/native projection and lifecycle were
subsequently completed below; ordinary scene/CLI remains a separate criterion.
New Linux/USA/native Windows checks remain pending; no performance or space
campaign, no AndroidPort change and no old source/scene oracle repeated.


## Managed and direct metadata16 consumption

EUR e25-d24adds fixed linked-metadata16.json/resource gameplay.linked-metadata16,
format linked-metadata16-v1-json with the base-dimensions dependency. All20files
are authenticated before transactional activation;18prior decoded files and
e24generations remain unchanged. A2MiB read guard bounds malformed input,
not a space optimization budget. Typed u16alias/raw bytes serve CPU/DMA/native
reads and merge prior owners on crossing accesses. Nonserialized counters
separate alias/display/flag/value/opaque source bytes. Original routines keep
resolving live list nodes/IDs/indices and executing sums, shifts, branches and
LCGupdates; no reader is replaced by a host shortcut. Save/native ABI is intact.

Warning-free Windows test build23604and two focused Wine80528checks pass
publication interruption/create/reuse/repair/move/e24retention, coherent alias/
opaque/value/reorder/source/dependency/path/role transactional guards, and
unavailable-copy Load/Undo/backup/restart.42sampled original/interpreter/native
paths run with all2048new raw ROMbytes altered and match complete states and
clocks:2133instructions/76dispatches as coverage,8compiled alias/17display/
22flag/24value bytes. Opaque fields are not read by these sampled original paths
and are separately bus-proved. All source bytes, record/source edges, widths,
ROMmirrors, real-BIOS bus reads, CpuSet/CpuFastSet and4typed DMAbytes pass.
A narrow crossing word with old-link and new-metadata raw spans both altered
proves2typed bytes from each owner. Source2581+19oracle stays closed.

Evidence:target/rom-data-metadata16/{windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-managed-tests-build.{jsonl,log}}. Ordinary scene/actual CLI acceptance
was subsequently completed below. No old acceptance/performance/space/Linux/
USA/native Windows/Android campaign was run.


## Ordinary metadata16 use and CLI

Warning-free Windows production build77128restores the original canonical
executable. Actual Wine CLI15733creates/reuses e25from an unrelated directory
with the old native cache. Its first replay fails before execution because the
new private input lacked the required gba-input v1header; the failed input/log
remain retained. Corrected replay15780selects Area List and reads0new bytes.
Start-only99771and Party→Start11378inspect actual menu contexts and also read0;
Party inventory-tab cycle24270covers ordinary equipment/consumables and reads0.
These negative locators have no full pair and are not repeated.

The already retained Clan image visibly identifies Mission Item at row3.
Extending that input with three Down/Aselects the distinct new scene: CLI42237
passes520frames with6new display-field bytes and an inspected final Chocobo Egg
entry(1/64). One520-frame Wine pair72249matches every complete state/clock,
central/wide framebuffer, OAM, dispatch and mixed/PSG/FIFO/HLE audio stream,
with the same6positive display bytes. Ordinary alias/flag/value/opaque use is
not claimed; source/native/bus proofs above cover those fields. The observed
screen identifies Mission Item usage; the catalog keeps structural field names.

Initial CLIharness aborted before its final no-write assertion, so a narrowly
justified prepare-reuse-only invocation59534completes that uncovered criterion:
all22managed file hashes/timestamps and the file set remain unchanged. Functional
prepare/reuse and the positive scene were not repeated. No direct RAM edit or
new recording was used. Source/managed/native/lifecycle acceptance remains closed.
Evidence:target/rom-data-metadata16/{windows-build.json,windows-cli-initial-summary.json,
windows-cli-{menu-down2,start-menu,party-start,inventory-tabs,clan-mission-item}.json,
windows-cli-no-write.json,windows-scene-result.json,
windows-clan-mission-item-metadata16-full-state-audio.log,clan-mission-item.png}.
New Linux/USA/native Windows, broader gameplay/maps/audio/GUI/archive/mod work
remain pending. No performance/space campaign or AndroidPort change was made.


## Fifty-two-byte linked metadata source contract

The standalone `metadata52::Catalog` owns EUR 6e5464..6e6bf4: 116 records of
52 bytes, 6032 source bytes. Eight checked LDR literals root the table, the
original CCE28 enumerator visits 0..115 (CCE82 compares 0x73), and two independent
readers root the following four-byte table at 6e6bf4. All bytes roundtrip from
five prefix bytes, an explicit routing byte, 26 middle bytes and 20 tail bytes;
there is no overlap with e25 resources. The structural name avoids assigning
unproved meanings to packed fields. At source-only closure this catalog added no managed file/resource: EUR
remained e25-d24 with 20 files / 19 resources, USA e8-d7. The subsequent
e26 managed/direct integration follows below.

Getter CBEC8 truncates the row, alternate row and property inputs to u8 and
dispatches 48 properties through checked code targets at CBEF8. Property 2
returns raw byte05 immediately. Other properties read their field when byte05
is zero, redirect to the caller's alternate row when it is FF, and redirect to
the named row for any other nonzero value. The shipped table has 112 zero-route
records and four FF records (80, 81, 82, 85), with no other nonzero routes.
Alternate indices remain caller inputs. Properties at least 48 busy-loop at
CBEE2; FF and ordinary redirect cycles can also remain in the getter. The
standalone comparison helper returns None for a nonreturning/out-of-contract
path; it is never an emulated fallback and is not called by the runtime.

The property map preserves unsigned assembled 16-bit fields, bytes, nibbles,
three-bit fields crossing byte boundaries and four 12-bit fields. Property order
is not source order. Unknown and unexposed bytes stay exact. Original 1344B4
uses live record byte05 as an index and reads raw offset33; 137DB4 with its
second argument truncated to u16 zero reads raw offset32 instead. Its nonzero
argument path addresses another table and is outside this bounded proof. The
other surrounding live record/ability/unit logic is not frozen into ROM data.

The literal 6e6850 at 36976 is a discovery false lead for ownership: its original
path uses a 14-byte stride and an index at least 128, giving minimum actual
address 6e6f50. It is not an internal metadata52 owner or an earlier boundary.
The retained counted-pair source independently has 464 normal selectors in
2..43, all within the 116-record owner, plus eight FF selectors with the separate
original publication contract. This does not prove all surrounding FF caller
preconditions. Parsed-owner validation is implemented in the subsequent integration below.

One focused Wine source oracle passes 6369 returning real-source cases: all
116 records and 48 properties with valid alternate inputs, multiple alternates
for each FF row, u8 truncation and the two raw live-index tail readers. Four
bounded original probes retain invalid-property and FF-cycle nonreturning
behavior. Because ordinary nonzero routes are absent, 294 isolated synthetic
calls prove zero/high/mixed packed bits, a two-step ordinary redirect chain,
an FF-to-chain path and raw tail bytes; two further bounded probes retain
ordinary/FF-to-ordinary cycles. Synthetic records fail authentication and are
never published. Full source/JSON/fixed-array/count/extent/coherent-edit and
callee/stack/live-record/outer-RAM guards pass. The 223421 instructions and
524 distinct PCs are coverage counts only. Windows build21334 is warning-free;
Wine20105 runs only this new source test. All 28 protected ROM/e25 files/cache/
loader/canonical/candidate/fixture hashes remain unchanged.

Evidence remains ignored under target/rom-data-metadata52: candidate-roots.json,
metadata52-inventory.json, metadata52-records-independent.json, reader-*.txt,
item-link-cross-reference.json, windows-tests-build.{jsonl,log},
windows-original-result.json, windows-original-europe/{metadata52.json,result.json}.
Managed preparation, direct CPU/DMA/native ownership and lifecycle were
subsequently completed below; ordinary scene/CLI is an independent criterion. New Linux/USA/native Windows
checks remain pending; no old source/integration/scene proof or performance/
space campaign was repeated and AndroidPort is unchanged.


## Managed and direct metadata52 consumption

EUR e26-d25 adds linked-metadata52.json / gameplay.linked-metadata52, format
linked-metadata52-v1-json, with an explicit item-link dependency. Attachment
validates the actual parsed counted-pair owner: active non-FF selectors must
name one of 116 rows, while FF retains its separate original contract. All 21
files authenticate before activation; 19 previous decoded files and e25
immutable generations remain unchanged. A 2 MiB read guard bounds malformed
inputs rather than setting a space optimization target. Exact typed byte views
merge prior metadata16 ownership across source boundaries. Nonserialized
counters separate header/routing/packed/tail/opaque bytes. No host property
shortcut replaces original field extraction, caller-alternate selection,
redirect loops, raw live-index tail readers or guest work/clock. Save/native
ABI and original ROM identity remain intact.

Warning-free Windows build26387 and Wine96091 pass two focused checks:
publication interruption/create/reuse/repair/move/e25 retention, coherent
routing/opaque/packed/reorder/source/dependency/path/role transactional guards,
normal-selector116 rejection with separate FF acceptance, and unavailable-copy
Load/Undo/backup/restart. Seventy-five sampled original/interpreter/native paths
cover all 48 properties, four FF rows with alternate inputs, truncation and raw
tails. With all 6032 new raw ROM bytes altered, complete states and clocks match;
2414 instructions / 169 dispatches are coverage only. Compiled reads consume
18 header / 83 routing / 68 packed / 8 tail bytes. Opaque bytes are not read by
these sampled original paths and are separately bus-proved. Every source byte,
record/source edges, widths, mirrors, real-BIOS bus, CpuSet/FastSet and 4 typed
DMA bytes pass. A narrow crossing word with both adjacent raw owners altered
proves 2 typed metadata16 bytes plus 2 typed metadata52 bytes. Closed source
6369+294 returning calls and bounded nonreturning probes were not repeated.

Evidence:target/rom-data-metadata52/{windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-managed-tests-build.{jsonl,log}}. Positive ordinary scene/actual CLI
acceptance was subsequently completed below. No new Linux/USA/native Windows, performance/space
campaign or AndroidPort change was made.


## Ordinary metadata52 use and CLI

Warning-free Windows production build71119 restores the original canonical
executable. Actual Wine CLI96132 prepares e26, reuses it without changing file
hashes/timestamps, and runs from an unrelated cwd using the old native cache.
The retained worldmap→Party→Unit context gains two ordinary Down presses and
A to open Change Jobs. Its 540-frame replay consumes 664 new bytes: 74 header,
335 routing and 255 packed bytes. The final Marsh/Thief job-selection image is
inspected; ordinary tail/opaque use is not claimed.

One 540-frame Wine pair6205 matches every complete state/clock, central/wide
framebuffer, OAM, dispatch and mixed/PSG/FIFO/HLE audio stream with the same
664 typed bytes. This is a distinct job-change scene; older Unit/Equip and
Mission Item acceptance was not repeated. No source/native/lifecycle oracle,
negative locator, failed harness or test was repeated in this integration.
No direct RAM edit or new recording was used. Read-only fixture/ROM/cache/
loader/canonical/candidate hashes remain protected.

Evidence:target/rom-data-metadata52/{windows-build.json,
windows-cli-{processes,no-write,summary}.json,windows-scene-result.json,
windows-change-jobs-metadata52-full-state-audio.log,change-jobs.png}.
New Linux/USA/native Windows and broader gameplay/maps/audio/GUI/archive/mod
work remain pending. No performance/space campaign or AndroidPort change.


## Four-byte two-pair source contract

Standalone `metadata4::Catalog` owns EUR 6e6bf4..6e6c44: 20 records of four
bytes, 80 source bytes. Two checked LDR literals root this table, and two
independent readers root the following two-byte map at 6e6c44. Each row preserves
two selector/value byte pairs. All 116 metadata52 records' raw offset30 indices
cover the complete range 0..19; all 40 new selectors name metadata52 rows.
There are 26 nonzero selectors, in 2..42, with values 1..5. Zero selectors and
unused payload bytes remain exact. There is no overlap with e26 resources.
At source-only closure this catalog added no managed file/resource: EUR
remained e26-d25 with 21 files / 20 resources, USA e8-d7. Subsequent e27direct
consumption follows below.

Both original consumers count nonzero selectors across both slots: CC424→CC44E
stores the count on the shared stack; 642FE→64330 keeps it in r10. Walking is a
different contract: CC474 and 64350 start only when the first selector is nonzero,
then stop at the first zero or after two pairs. An isolated hole can therefore
have count1 but no leading pair; it must not be normalized into a compact list.
The new model exposes both `active_count` and `leading_count` for comparison.
Neither is a replacement for original runtime instructions.

Selector prefixes CC41A and 642F4 call original metadata52 property39 and then
index the four-byte table. Their actual outer loops enumerate job IDs 2..43
with the same alternate index; FF rows 80/81/82/85 are not included in that
caller contract. The original threshold prefixes CC4EC and 643C2 compare a
live qualified-count register as signed against an unsigned value byte. Passing
increments the success tally with u8 wrapping, advances by two bytes and either
enters the next pair or exits. Deeper scans of live unit ability/progress fields
remain original code; the bounded oracle does not freeze those scans or claim
whole surrounding functions/global caller preconditions.

One focused Wine source check passes 320 real cases across both versions of
Count/Select/Gate/Threshold: all 20 rows, actual caller job IDs 2..43 and counts
below/equal/above each active value. Twenty-four isolated synthetic cases prove
holes, nonzero unused payload, unsigned value255, the signed comparison edge
and success-tally u8 wrapping. Synthetic records fail authentication and are
never published. Shared-stack, preserved prefix-register and outer-RAM guards
pass, as do strict JSON/arrays/count/extent/coherent-source and parsed metadata52
owner checks. These are bounded mid-function prefixes, not whole-function ABI
proofs. The 9560 instructions / 136 distinct PCs are coverage only. Windows
build21229 is warning-free and Wine74773 runs only this new test; no old source,
native/lifecycle or ordinary-scene acceptance is repeated.

Evidence remains ignored under target/rom-data-metadata4: candidate-roots.json,
reader-*.txt, inventory-metadata4.py, metadata4-inventory.json,
metadata4-records-independent.json, windows-tests-build.{jsonl,log},
windows-original-result.json and windows-original-europe/{metadata4.json,result.json}.
Managed preparation, direct CPU/DMA/native ownership and lifecycle were
subsequently completed below; ordinary scene/CLI is an independent criterion. New Linux/USA/native Windows checks stay
pending; no performance/space campaign or AndroidPort change was made.


## Managed and direct metadata4 consumption

EUR e27-d26 adds linked-metadata4.json / gameplay.linked-metadata4, format
linked-metadata4-v1-json, with an explicit metadata52 dependency. Actual parsed
metadata52 raw offset30 indices must name the new rows; all new selector bytes
must name metadata52 rows. All 22 files authenticate before transactional
activation; 20 previous decoded files and e26 immutable generations remain
unchanged. A 2 MiB read guard bounds malformed input rather than a space target.
Typed selector/value bytes preserve both slots and unused values and merge
prior metadata52 ownership on crossings. Counters are not serialized. The
runtime does not replace original count/leading-walk or qualification logic,
signed comparisons, unsigned loads, tally wrapping or guest work/clock. Deeper
ability scans keep running with live inputs; save/native ABI remains intact.

Warning-free Windows build42571 and Wine33626 pass two focused checks:
publication interruption/create/reuse/repair/move/e26 retention, coherent
selector/unused value/reorder/source/dependency/path/role transactional guards,
parsed cross-owner bounds, and unavailable-copy Load/Undo/backup/restart.
Fifty-six sampled original/interpreter/native count/selection/gate/threshold
prefixes cover count shapes0/1/2, values1/2/3/5, second-slot checks, signed
comparison and tally wrapping. With all 80 new raw ROM bytes altered, complete
states and clocks match; 1307 instructions / 81 dispatches are coverage only.
Compiled reads consume 54 selector / 34 value bytes. Every byte, edges, widths,
mirrors, real-BIOS bus, CpuSet/FastSet and 4 typed DMA bytes pass. A narrow
crossing word with both raw owners altered proves 2 typed metadata52 bytes
plus 2 metadata4 bytes. These remain bounded prefix proofs, not whole-function
ABI proofs; source320+24 acceptance was not repeated.

Evidence:target/rom-data-metadata4/{windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-managed-tests-build.{jsonl,log}}. Positive ordinary scene/actual CLI
acceptance was subsequently completed below. New Linux/USA/native Windows, performance/space
campaign and AndroidPort work remain outside this integration.


## Ordinary metadata4 use and CLI

Warning-free Windows production build59395 restores the original canonical
executable. Actual Wine CLI36972 prepares e27, reuses it without modifying file
hashes/timestamps, then replays from an unrelated cwd using the old native cache.
The retained ordinary 540-frame Change Jobs input is used for the newly migrated
80-byte family, with its own positive-use gate; prior e26 metadata52 acceptance
is not reopened. The replay reads 38 new bytes: 32 selector and 6 value bytes.
The final Marsh/Thief job-selection image is inspected.

One 540-frame Wine pair48436 matches every complete state/clock, central/wide
framebuffer, OAM, dispatch and mixed/PSG/FIFO/HLE audio stream with the same
38 typed bytes. Deeper original live ability scans keep executing in that scene;
targeted tests remain bounded prefix proofs. No direct RAM edit, new recording,
old source/native/lifecycle oracle or failed locator was repeated. The private
generator's multiline-template mismatch was corrected before any build; all
integration builds and runtime checks pass.

Evidence:target/rom-data-metadata4/{windows-build.json,
windows-cli-{processes,no-write,summary}.json,windows-scene-result.json,
windows-change-jobs-metadata4-full-state-audio.log,change-jobs.png}.
New Linux/USA/native Windows and broader gameplay/maps/audio/GUI/archive/mod
work remain pending. No performance/space campaign or AndroidPort change.


## Two-byte forward and inverse source contract

Standalone `metadata2::Catalog` owns EUR 6e6c44..6e6c9c: 44 records of two
bytes, 88 source bytes. Checked LDRs CCD4C/CCD5E root it; independent literal
127E46 roots the following 48-byte copy source at 6e6c9c. The last pair
[43,19] is a real record, rather than padding. All stored keys currently equal
their indices and fit the 116 parsed metadata52 owners; both bytes remain
explicit fields. Semantic names for the values remain unproved.

Original CCD48 truncates the index to u8 and reads byte1 without a local bound.
The owned-source proof covers indices 0..43, including row43. `forward` returning
None outside this extent is a diagnostic, never an emulated fallback. Original
CCD58 truncates the query to u8, scans only rows 0..42, returns the first
matching stored key byte and returns zero on a miss. Duplicate values and this
unequal extent must not be normalized. The live wrapper CBD74 loads byte7 from
a live record and calls the direct getter; its bounded proof stops at CBD7C
with the caller frame retained. Four direct callers and one inverse caller are
checked statically; broader caller bounds and whole functions remain unclaimed.

Warning-free Windows build99948 and one Wine56417 source oracle pass 348 real
cases: all44 direct/live indices, all256 inverse values and truncation cases.
Thirteen isolated synthetic cases prove high bytes, duplicate first-match
stored keys and a last-row-only value readable forward but excluded inversely.
Synthetic records fail authentication and parsed-owner validation and are never
published. Exact results, inverse cursor, callee registers, stack/live-record/
outer-RAM guards, strict JSON, extent and coherent-source rejection pass.
75352 instructions / 27 PCs are coverage only. All31 protected files remain
unchanged; no old source/native/lifecycle/scene proof was repeated and no
build/test failure occurred.

Evidence:target/rom-data-metadata2/{metadata2-inventory.json,
metadata2-records-independent.json,reader-*.txt,caller-*.txt,
windows-tests-build.{jsonl,log},windows-original-result.json,
windows-original-europe/{metadata2.json,result.json}}. Source-only closure
leaves EUR e27-d26/22files/21resources and USA e8-d7 unchanged. Managed
publication, typed runtime ownership, lifecycle and ordinary scene/CLI were
subsequently completed below. New Linux/USA/native Windows and broader TODO remain pending; no
performance/space campaign or AndroidPort change.


## Managed and direct metadata2 consumption

EUR e28-d27 adds linked-metadata2.json / gameplay.linked-metadata2, format
linked-metadata2-v1-json, with metadata52 dependency and actual parsed stored-key
owner validation. All23files/22resources authenticate before transactional
activation; all21previous decoded files and e27generations remain unchanged.
A2MiB read guard bounds malformed input rather than a space target. Typed key/
value bytes merge earlier ownership at crossings, while original forward44/
inverse43 instructions, duplicate/first stored-key/miss0/u8truncation and live
byte7caller work keep executing. No new guest bound/fallback, instruction skip,
clock or save/native ABI change is introduced; counters are not serialized.

Initial Windows95465compile caught three existing budget-test calls missing the
new zero argument; corrected without changing their behavior, failed logs
retained. Corrected Windows91568build is warning-free; Wine80052two focused
checks pass publication interruption/create/reuse/repair/move/e27retention,
coherent key/value/lastrow/reorder/source/dependency/path/role transactional
rejection, parsed metadata52key bounds and unavailable-copy Load/Undo/backup/
restart.21sampled original/interpreter/native direct/inverse/live-byte paths
with all88raw ROM bytes altered match complete states/clocks;1705instructions/
28native dispatches are coverage only. Compiled reads use7key/235value bytes.
Allbytes/edges/widths/mirrors/real-BIOS bus/CpuSet/FastSet/DMA4B and a narrow
crossing with2oldmetadata4/2newmetadata2typed bytes pass. The source348+13oracle
stays closed; whole surrounding functions/global caller bounds remain unproved.

Evidence:target/rom-data-metadata2/{windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-managed-tests-build.{jsonl,log},windows-initial-failed-managed-tests-build.*}.
New Linux/USA/native Windows and broader TODO remain pending; no performance/
space campaign or AndroidPort change.

## Ordinary metadata2 mission use and CLI

Warning-free Windows production98775build restores the canonical executable.
Actual CLI16539prepares/reuses e28without writes from an unrelated cwd with the
old native cache. Its retained540frame Change Jobs input reads0new bytes and
is coverage-negative. Pub560navigation instead enters Rumors; bounded230/290/
365prefixes and B-return480observations identify the branch, with the native
separated Pub root absent from PPM. The existing measured Pub contract identifies
Rumors/Missions/Quit Mission/Leave order. Corrected Down210/A300reaches the
Missions LIST at620; A380is too early for that list. These locator results are
preserved, with no full differential or unchanged locator rerun.

After the stable list, three Down presses and A860/A1060enter the selected
mission path; its1260frame probe still reads0bytes. Additional ordinary
A1300/A1600confirmations yield one1840frame Pub mission-requirement replay
with4new typed value bytes and0key bytes. Actual CLI24738and one full Wine
pair2490match every complete state/clock, central/wide framebuffer, OAM,
dispatch and mixed/PSG/FIFO/HLE audio stream, with the same4new bytes. Inverse
key reads are proved by focused native checks, not claimed in this scene.
The final Pub background image is inspected; separated panels, selected unit
and complete mission outcome are not inferred from that image. No direct RAM
edit, new recording or persisted save write: runc never writes its preload.
Retained Pub snapshot/input reuse is justified by the new88Bfamily and separate
positive-use gate; old source/native/lifecycle/scene acceptance is not reopened.

Evidence:target/rom-data-metadata2/{windows-build.json,windows-cli-{processes,
no-write,summary,pub-mission-details,pub-menu-prefix,pub-options-prefix,
pub-selection-prefix,pub-return-options,pub-missions,pub-dispatch-mission,
pub-dispatch-confirm}.json,windows-scene-result.json,
windows-pub-mission-metadata2-full-state-audio.log,pub-dispatch-confirm.png}.
The CLI managed25file set/hashes/timestamps and protected original inputs,
e27dataset/candidates/cache/loader/canonical remain unchanged. Source and new
native/lifecycle checks are not repeated for the ordinary scene. New Linux/USA/
native Windows and broader gameplay/maps/audio/GUI/archive/mod remain pending;
no performance/space campaign or AndroidPort change.


## Default constructor template source contract

Standalone `default48::Catalog` owns EUR6e6c9c..6e6ccc, exactly48bytes from
checked127E46→127EDCliteral and original copy length48. It stores byte00/01,
bytes02..04, argument bytes05..07,17full halfwords08..28, flags2a and two full
halfwords2c..2e; semantic names outside the observed patches remain unproved.
The following240bytes resemble five48byte shapes, but have no independently
found root/consumer and are not owned or normalized into extra records.

Constructor127DC4truncates input bytes and copies only when mode is1or2;
other modes return0. IDs20..23dereference ROM70cd58→6eca78 and select four
separate48byte sources. Every other u8ID uses the default template. The common
patch writes byte00only when the signed ID is nonnegative, byte01only when the
selector exceeds1, and bytes05..07from call arguments. Flag1/2/3ORs8000/2000/
1000into flags2a, preserving preexisting bits; other flags leave it intact.
The new `patched` helper is diagnostic; it does not replace runtime guest work.
The proof stops at127EB0before allocation/live-unit lookup, retaining the caller
frame. Invalid-mode checks additionally prove actual return0/callee/SP restore.

Existing original initializer027EE..027FEexecutes10instructions and installs
ROMd96838..d968e8atIWRAM03005e78..03005f28. Its copy entry03005ee9is selected
through ROM370eb4and original BXr3trampoline145C14. No snapshot, new fixture or
old acceptance rerun is needed. Windows54305is warning-free and one new Wine
source oracle passes271real cases: all256IDs/default-vs-four-alternate sources,
mode1/2/rejection/truncation, signed-ID edge, selector0/1/2/255and flags0/1/2/3/
255.30isolated nonzero/high-bit synthetic cases prove untouched fields and OR
preservation; they fail authentication and are never published. All48source
bytes reconstruct independently; strict JSON/fixed arrays/source/coherent-edit,
scratch/frame/incoming-argument/invalid-callee/outer-RAM guards pass.48812guest
instructions/155PCs are coverage only. All32protected original/e28/candidate/
cache/loader/canonical/source fixtures remain unchanged; no build/test failure.

Evidence:target/rom-data-default48/{constructor-127d60.txt,
constructor-tail-127f00.txt,nearby-literal-candidates.json,
raw-pointer-candidates.json,inventory-default48.py,default48-inventory.json,
default48-fields-independent.json,windows-tests-build.{jsonl,log},
windows-original-result.json,windows-original-europe/{default48.json,result.json}}.
Source-only closure leaves EUR e28-d27/23files/22resources and USAe8-d7unchanged.
Managed publication/runtime ownership/lifecycle/ordinary scene/actualCLI for
this template were subsequently completed below. Four alternate192bytes and their ROM pointer remain
unowned. New Linux/USA/native Windows/broader TODO remain pending; no performance/
space campaign or AndroidPort change.


## Managed and direct default constructor template

EUR e29-d28 adds default-constructor-template.json /
gameplay.default-constructor-template, format default-constructor-template-v1-json,
with no invented semantic owner dependency. All24files/23resources authenticate
before transactional activation; all22prior decoded files/e28generations remain
unchanged. A2MiB read guard bounds malformed input rather than a space target.
Typed header/argument/full-halfword/flag/tail bytes merge previous metadata2
ownership at crossings. Original ROM/IWRAM copy, signed-ID/selector/live argument/
ORflag patches, mode and alternate selection, allocation work, clock/save/native
ABI remain; counters are not serialized. The diagnostic patched helper is not
used to shortcut guest execution.

Initial Windows21352compile caught a stale two-element thread-local macro
counter alongside the new five-field counters; corrected, failed logs retained.
Corrected55456Windows build is warning-free. Wine5227two focused checks pass
publication interruption/create/reuse/repair/move/e28retention, coherent header/
opaque/flag/tail/source/spurious dependency/path/role transactional rejection and
unavailable-copy Load/Undo/backup/restart.16sampled original/interpreter/native
constructor prefixes include real copied IWRAM helper blocks and all48rawbytes
altered; complete states/clocks match. Eight alternate/rejected paths read0new
source bytes. Compiled header/argument/halfword/flag/tail use is39/21/272/16/32B;
2319instructions/1061dispatches are coverage only. Allbytes/edges/widths/mirrors/
real-BIOS bus/CpuSet/FastSet/DMA4B and a narrow crossing with2oldmetadata2/
2newdefault48typed bytes pass. Source271+30proof stays closed. Allocation/live
unit tail remains original, not a whole-function ABI proof; four alternate192B/
ROMpointer/following240B remain unowned.

Warning-free production83840build restores the canonical executable. Actual
WindowsCLI33995prepares/reuses e29without dataset writes from unrelated cwd and
replays the retained397frame Pub entry with old native cache. New-family use
is96B:10header/6argument/68halfword/4flag/8tail. One397frame Wine63615pair
matches every complete state/clock, central/wide framebuffer, OAM, dispatch and
mixed/PSG/FIFO/HLEaudio stream with those same96typed bytes. The final Pub image
is inspected. This retained scene/input is justified by the newly migrated48B
family and independent positive gate; older graphics/text/metadata2acceptance
is not reopened. No negative locator, RAM edit, new recording or persisted
save write was needed; no source/native/lifecycle oracle was repeated.

Evidence:target/rom-data-default48/{windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-managed-tests-build.{jsonl,log},windows-initial-failed-managed-tests-build.*,
windows-build.json,windows-cli-{processes,no-write,summary}.json,
windows-scene-result.json,windows-pub-default48-full-state-audio.log,pub.png}.
Original inputs/e28data/candidates/cache/loader/canonical remain protected; CLI
26file-set/hash/mtime guards pass. New Linux/USA/native Windows and broader
maps/gameplay/audio/GUI/archive/mod remain pending; no performance/space campaign
or AndroidPort change.


## Next rooted constructor sources

Static inventory now separates four alternate constructor records at
6eca78..6ecb38/192B and their4Bpointer70cd58..70cd5c,196Btotal. Checked
127E1A→127E38loads header70cd54; LDR+4at127E28selects the base. Gatedu8IDs
20..23and original48byte indexing/copy bound the four records without owning
the neighboring header. All byte/halfword fields reconstruct independently,
with no e29overlap. Original CCF9C..CD046reads five equipment halfwords08..10
through item properties6/3; all20references fit actual parsed376item owners.
Byte01values104..107remain structural, with no new semantic-owner assertion.

Evidence:target/rom-data-constructor-alternates/{inventory-constructor-alternates.py,
constructor-alternates-inventory.json,constructor-alternates-records-independent.json,
template-consumer-0ccf9c.txt}. The private inventory's misplaced CD034LDRHassertion
was corrected toCD032before output/build; static checks pass. Source model/oracle were subsequently completed below; no managed resource or
runtime projection yet at this boundary. Current EUR e29-d28/
24files/23resources and USAe8-d7stay unchanged. Next is the reduced source-only
proof for these four records; closed default271+30/native16/397scene proofs
are not reopened. New Linux/USA/native Windows remain pending; no metrics.


## Typed constructor alternates source proof

Strict `constructor_alternates::{Record,Alternates,Catalog}` preserves all four
IDs20..23, their fixed source offsets and full byte/halfword fields, plus parsed
base pointer70cd58. Two disjoint ranges own192record bytes and4pointer bytes.
The first five halfwords of each record validate against actual parsed376item
owners. Source-offset/count/order/ID/fixed-array/JSON/coherent-edit guards retain
all196bytes. Byte01meaning and neighboring header ownership remain unclaimed.

Warning-free Windows39935and one Wine61563source oracle pass24real four-ID
copy/live-patch cases coveringu8truncation, selector0/1/2/255and flag/mode edges.
Eight isolated synthetic records retain high bits, unused fields, valid high
item IDs and preexisting ORflags; they fail source authentication and are never
published. Existing original initialization executes10instructions to install
the real IWRAMcopy helper; original constructor127DC4→127EB0keeps its scratch/
caller frame and live incoming arguments. Outer RAM guards pass.5560instructions/
137PCs are coverage only; allocation/live-unit tails remain unclaimed. Closed
default271+30/native16/397scene acceptance was not repeated. All32protected
original/e29/item-owner/candidate/cache/loader/canonical/source files stay intact;
no build/test failure. HEAD advanced externally toe180772during this unit,
committing prior e29content without changing this source scope; no agent commit.

Evidence:target/rom-data-constructor-alternates/{windows-tests-build.{jsonl,log},
windows-original-result.json,windows-original-europe/{constructor-alternates.json,
result.json}}. Managed preparation, direct typed pointer/record ownership,
lifecycle and positive ordinary scene/actualCLI were subsequently completed below; EUR e29-d28/
24files/23resources and USAe8-d7unchanged. New Linux/USA/native Windows and the
rest of the TODO remain pending; no performance/space campaign/AndroidPort change.


## Managed and direct constructor alternates

EUR e30-d29 adds constructor-alternate-templates.json /
gameplay.constructor-alternate-templates, format constructor-alternate-templates-v1-json,
with actual parsed Items dependency for all20equipment references. All25files/
24resources authenticate before transactional activation; all23previous decoded
files/e29generations remain unchanged. A2MiB read guard bounds malformed input,
not an extracted-space target. Typed4Bbase pointer and192Brecord bytes project
through pages6ec/70c, preserving ID/order/offsets and all header/argument/full-
halfword/flag/tail fields. Per-record masks/counters are nonserialized. Original
ROM/IWRAMcopy, pointer/index/mode selection, live patches, allocation work,
clock/save/native ABI remain; no guest routine or fallback is replaced.

Initial Windows89684build is warning-free; pre-Wine review corrects an inherited
zero-use guard direction for the new family and adds an old-default positive
coexistence check. Corrected Windows61725and production92748builds are warning-
free; no compiler/runtime failure. Wine39423two focused checks pass publication
interruption/create/reuse/repair/move/e29retention, coherent header/opaque/flags/
tail/byte01swap/pointer/source/dependency/path/both-source-role transactional
rejection, parsed item bounds and unavailable-copy Load/Undo/backup/restart.
14sampled original/interpreter/native paths include real IWRAMcopy blocks and
all196rawbytes altered during guest execution; complete states/clocks match.
Assertions that read raw records run only after execution stops and spans are
restored. All four records are positively typed(mask15); compiled pointer/
header/argument/halfword/flag/tail use is36/45/27/306/18/36B. Five default or
rejected paths read0newB; old default ownership remains positive.2151instructions/
979dispatches are coverage only. All196bytes/edges/record boundaries/widths/
mirrors/real-BIOS bus, both source spans in CpuSet/FastSet and DMA4B each pass.
Raw-header/pointer and final-record/following-source words each preserve2typed
and2rawbytes. Source24+8/default271+30/native16/397proofs stay closed.

## Ordinary alternate-record use and CLI

Production92748restores the canonical executable. Actual WindowsCLI52141creates/
reuses e30without dataset writes from an unrelated cwd and uses the old native
cache. The retained3000frame clean-boot input reaches the school courtyard but
reads0newbytes; it is a coverage-negative locator, with no full pair or timing.
Local text lookup for metadata52rows104/105yields Judge and106Judgemaster; this
is scene-discovery evidence without renaming the structural file fields.
Retained worldmap hover/A200plus A500/A900/A1300reaches a Lutia battle-map laws
NOTICE at1700, still0newbytes; this negative locator also remains closed.

Ordinary A1800/A2100/A2400/A2700and Start3000/A3300advance the NOTICE/deployment.
Actual CLI26234and one3600frame Wine68173pair pass with45newbytes:4pointer/
6header/3argument/28halfword/2flag/2tail, record mask1(ID20only). Every complete
state/clock, central/wide framebuffer, OAM, dispatch and mixed/PSG/FIFO/HLEaudio
sample matches. The final Lutia battle image is inspected. This ordinary proof
covers the observed direct pointer/field reads; it does not claim every48B or
the whole constructor runs in that scene. All four records/full-copy behavior
remain targeted-only. Reused input/snapshot are justified by the new196Bfamily
and separate positive gate; no direct RAM edit, new recording, persisted save
write or old source/native/lifecycle acceptance repeat. No full pair is run on
negative locators. Existing shared scene tooling supports both cleanboot and
snapshots, so no additional fixture or oracle feature is added.

Evidence:target/rom-data-constructor-alternates/{windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json},
windows-managed-tests-build.{jsonl,log},windows-managed-tests-pre-guard-review-build.*,
windows-build.json,windows-cli-{processes,no-write,summary,engage-battle,deploy-start}.json,
scene-class-label-locator.json,windows-scene-result.json,
windows-deploy-constructor-alternates-full-state-audio.log,deploy-start.png}.
Protected originals/e29/item-owner/candidates/cache/loader/canonical and CLI
27file-set/hash/mtime guards pass. HEAD advanced externally toe180772during the
source unit; unrelated user todo2.md additions remain untouched. New Linux/USA/
native Windows and broader maps/gameplay/audio/GUI/archive/mod remain pending;
no performance/space campaign or AndroidPort change.


## Next bounded fourteen-byte records

Static discovery separates physical6e6f6c..6e7482/1302Binto93fourteen-byte
records with logical IDs2..94. Original getters use biased base6e6f50, rooted
by CD05C→CD07Cand ten other selected literals. CBAD4routes nonnull liveunit+4>1
to these records; IDs0/1keep their original normal job/property branch. A separate
12Bindexed source at6e7484is independently rooted by13762E→137640; only whole
14Brecords before it are owned, leaving two zero bytes unowned. Global caller
upper bounds and constructorID0remain unproved; no fallback is invented.

Fields preserve bytes00..03, full halfwords04/06, packed08..0a, byte0b and bytes
0c/0d. CF144/180return halfwords4/6, CF1BCbyte11and CF1F0byte12foru8argument1or
byte13otherwise. CF08Cretains original packed palette bank/selector lookup/copy.
All186byte01/02values fit parsed116metadata52rows, but that range alone is not
semantic owner proof forbyte01. Original initializer copies byte02to liveunit+7
when descriptor override01is zero; this supports the byte02job link. All1302B
reconstruct independently without overlap with e30resources.

The earlier6e6dbcroot hypothesis was rejected: its reader uses4Bindexing,
and6e6ee4is also separate. Supposed14Baliases6e7484/6e78d4actually have12Bstride.
Evidence:target/rom-data-metadata14/{reader-*.txt,readers-*.txt,
predicate-0cbab0.txt,inventory-metadata14.py,metadata14-inventory.json,
metadata14-records-independent.json}. Model/source oracle were subsequently completed below; managed/runtime remain
next at this boundary, EUR e30-d29/25files/24resources and USAe8-d7unchanged. No new
build/test or closed acceptance repeat; newLinux/USA/nativeWindows and other
families remain pending, no metrics/space work.


## Typed fourteen-byte records source proof

Strict `metadata14::{Metadata14,Record,Catalog}` separates biased address6e6f50
from physical6e6f6c..6e7482, retaining93records/1302B/logicalIDs2..94and every
byte/full-halfword/packed field. Byte02validates against actual parsed116metadata52
job owners. Byte01has no proven semantic owner: a synthetic value250or255is
structurally accepted but fails source authentication. IDs0/1/95plus and the
2Btrailing gap are not owned; diagnostic recordNone never becomes a guest fallback.

Warning-free Windows18151and one Wine66692oracle pass504real cases:471getter
calls over all93IDs and u8argument truncation,22packed-palette cases across live
books0/1/2/default3and11initializer prefixes. Getter results preserve unsigned
u16/u8values. Palette tests observe the original source returned byCF5DC, verify
original32Bcopy/destination bank/512Bguards and keep the live book byte02002fc2.
No palette payload ownership is inferred. Initialization stops atCD0B6after the
original CCF9Cprefix, checking byte02fallback/descriptor overrides/live auxiliary
byte/flags/264Bunit contents and retained mid-function frame, not allocation tails.

13isolated synthetic cases preserve highu16/u8fields, bank15, unused bytes,
unprovedbyte01and a valid highjob selector. StrictJSON/fixedarray/bias/ID/order/
count/source/coherent-edit/byte02owner/callee/scratch/descriptor/unit/palette/
outer-RAM guards pass; synthetic records unauthenticated/unpublished.23627guest
instructions/380PCs and10original helper-init instructions are coverage only.
All33protected original/e30/metadata52owner/candidate/cache/loader/canonical/source
fixtures remain unchanged; no build/runtimefailure or old acceptance repeat.
HEAD2d66955advanced externally during this unit, committing prior e30/cleanup/
user-doc changes; inspected/preserved without agent commit.

Evidence:target/rom-data-metadata14/{windows-tests-build.{jsonl,log},
windows-original-result.json,windows-original-europe/{metadata14.json,result.json}}.
Managed publication/direct CPU-DMA-native ownership/lifecycle/positive ordinary
scene/actualCLI were subsequently completed below; this source-only boundary
retained EUR e30-d29/25files/24resources and USAe8-d7.
Normal live job/property branches, global caller bounds, neighboring families,
newLinux/USA/nativeWindows and the broader TODO remain pending. No performance/
space campaign or AndroidPort change.

## Managed and direct fourteen-byte special-unit metadata

EUR `v1/e31-d30` adds `linked-metadata14.json`, resource
`gameplay.linked-metadata14`, format `linked-metadata14-v1-json`. All 26 files /
25 resources authenticate together; all 24 earlier decoded files and e30
generations are retained. The declared dependency is the actual parsed
`gameplay.linked-metadata52` owner for byte02. Byte01 remains unconstrained by
that dependency. USA retains e8-d7. The 2 MiB file limit guards malformed input;
this change does not enforce a performance or extraction-space target.

Typed fields serve exactly 1,302 bytes at 6e6f6c..6e7482 through CPU, compiled
native, DMA and SWI reads on pages 6e6/6e7. Reads crossing a source boundary
merge typed and original bytes; IDs0/1, the following two-byte gap and higher
indices retain their existing sources. Nonserialized field counters and two
64-bit record masks distinguish positive use of these 93 records. Original
getter routing, live palette books and 32-byte copies, descriptor overrides,
initialization/allocation, guest clock and save/native ABI remain active.

The warning-free Windows test build and two Wine checks (75387) pass managed
publication interruption/create/reuse/repair/move, e30 retention, coherent
field/source/dependency/path/role edits, parsed byte02 owner validation and
transactional attachment. With the dataset copy made unavailable, Load/Undo,
backup restoration and restart retain the in-memory resources. Byte01=255 is
structurally valid but cannot authenticate as original data.

37 original/interpreter/compiled paths with all 1,302 raw source bytes altered
match complete states and clocks. IDs2/20/28/94 consume compiled byte/halfword/
packed/tail fields (13/12/35/9 bytes); two normal-ID0/1 paths consume no new
source bytes. All bytes, record-mask boundary, edges, widths, mirrors, real-BIOS
bus, CpuSet/FastSet, cross-record DMA and typed/raw padding crossings pass.
4,793 instructions and 2,502 dispatches describe coverage only. The source
504+13 oracle and earlier runtime acceptance remain closed and were not repeated.

Production build7465 is warning-free and restores the canonical executable.
ActualCLI9489 creates and reuses the dataset without file/hash/mtime changes,
then replays the retained 397-frame Pub transition with the old native cache
from an unrelated working directory. It reads 27 new bytes from two records:
4 byte fields, 8 halfword bytes, 12 packed bytes and 3 tail bytes. The final Pub
image is inspected. One Wine24398 pair matches every complete state/clock,
central and wide image, OAM, dispatch and mixed/PSG/FIFO/HLE audio stream with
the same positive source counters. No additional scene locator is needed.

Evidence: `target/rom-data-metadata14/` contains windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json}, windows-build.json,
windows-cli-{processes,no-write,summary}.json, windows-scene-result.json,
windows-pub-metadata14-full-state-audio.log and pub.png. Protected originals,
e30 data, source proof, snapshots, input, cache, loader and canonical executable
are retained. Global caller bounds, byte01 semantic meaning, palette payloads,
allocation tails, adjacent families and broader TODO coverage remain open.
New Linux/USA/native Windows checks are pending; no performance, space or
AndroidPort work is included.

## Indexed twelve-byte packed pair source contract

Two distinct indexed banks occupy 6e7484..6e78d4 (92 rows / 1,104 bytes) and
6e78d4..6e7994 (16 rows / 192 bytes). Original literals13762e→137640 and
137652→137664, both original multiply-by-12 paths and shared reader135a1c
establish the contract. The following independent root1377a2→1377b0 points to
6e7994 and is excluded. All 1,296 bytes reconstruct without e31 overlap.
These are separate from the earlier fourteen-byte special-unit source.

Each row stores 48 two-bit values. The strict `metadata12` model keeps twelve
fixed groups of four values0..3, IDs, offsets, bank order and exact92/16counts.
No semantic label or foreign owner is inferred. Existing caller loops visit
slots0..43; stored pairs44..47 retain every bit. Original code chooses the low
bit when the mode's low byte is zero and the high bit for any other low byte.
Rows use the full32-bit input and slots retain signed/wrapping shift arithmetic;
there is no local row/slot bound. A diagnostic None never supplies a guest
return value or changes original access outside the catalog.

One reduced Wine88437 source oracle passes 1,640 real calls over every row,
boundary slots and all48pairs in first/last rows, plus ten raw outside/wrapping
calls: next-bank/global-source reads, row256 distinct from row0, large row wrap,
negative slot and slot48. 384 isolated synthetic calls cover all pair values,
high bits and mode0/0x101/0xff/0x100. Every call observes the exact original
byte address/one LDRB and checks return value, callee, stack and outer RAM.
Strict JSON/fixed shapes/pair range/order/count/ID/source/coherent edits pass.
Synthetic sources fail authentication and remain unpublished. 75,515 guest
instructions / 54 PCs describe coverage only.

Initial Windows99149 compile exposed test setup field/type errors; these are
corrected and the22815build is warning-free. Wine has no failure. All34protected
ROM/e31/candidate/cache/loader/canonical/sourcefixture hashes remain unchanged;
no earlier source/runtime/scene proof was repeated. Evidence lives in
`target/rom-data-metadata12/`: metadata12-inventory.json, independent records,
bounded reader/caller disassembly, windows-initial-failed-tests-build.*,
windows-tests-build.*, windows-original-result.json and
windows-original-europe/{metadata12.json,result.json}.

At this source-only boundary managed EUR e31-d30 / 26 files / 25 resources and
USAe8-d7 remain unchanged. Managed publication, direct CPU/DMA/native ownership,
lifecycle and positive ordinary use were subsequently completed below. Global caller row bounds,
meaning of pair values, the separately rooted global bit source, newLinux/USA/
nativeWindows and wider gameplay/maps/audio/mod phases remain open. No
performance, space or AndroidPort work is included.

## Managed and direct twelve-byte packed pair banks

EUR `v1/e32-d31` publishes `linked-metadata12.json`, resource
`gameplay.linked-metadata12`, format `linked-metadata12-v1-json`. All27files /
26resources authenticate together. Both source roles retain exact1104/192-byte
bank extents; every earlier25decoded file and e31 generation stays unchanged.
The original semantic owner of these flags is unproved, so no foreign-key
dependency is invented. USA retains e8-d7. Parsing limits guard malformed input;
no extraction-space or performance target is measured.

Typed bytes from the 48two-bit fields in each row serve original CPU/native,
SWI and DMA reads on page6e7. Cross-bank words combine both banks; outside
bytes retain existing owners/ROM. Nonserialized bank counters and two64-bit
record masks gate the108records. Full32row, signed/wrapping slot, low8mode,
original routines, clock and save/native ABI remain active. Diagnostic None
never replaces a guest read or return value.

Warning-free Windows36670 and two Wine22271 checks pass publication failure,
create/reuse/repair/move/e31 retention, coherent pair and unused-tail edits,
source/dependency/path/both-role guards, transactional attachment and unavailable
copy Load/Undo/backup/restart.49original/interpreter/compiled paths with all
1296raw bytes altered match complete states/clocks. Four records are positive;
compiled primary/secondary reads are23/22bytes. Three outside paths use0new
bytes; full32, wrapping and next-bank accesses retain their original behavior.
All1296bytes/edges/mask boundaries/widths/mirrors/real-BIOS, both-bank CpuSet/
FastSet and cross-bank4typed/trailing-global2typed2raw DMA checks pass.
1828instructions/153dispatches describe coverage only. Only a comment and
injected-error wording were cleaned after compiling the test; assertions and
production behavior were unchanged. The source1640+10+384 and e31 acceptance
were not repeated.

Windows94316 production build is warning-free and restores the canonical
executable. ActualCLI prepares/reuses e32 without managed-file/hash/mtime changes,
then uses the existing native cache from an unrelated working directory.
The positive ordinary sample is 2200-frame Echo Grass confirmation: ROM_DATA_METADATA12_SCENE frames=2200 bus_reads=44 source_bytes=44 records_used=1 primary_bytes=44 secondary_bytes=0.
One Wine pair matches every complete state/clock, central-wide image, OAM,
dispatch and mixed/PSG/FIFO/HLE audio stream with the same counters; the final
image is inspected. Pub397, Lutia deployment3600 and the earlier3530enemy-movement replay are
coverage-negative and retained without full pairs. Bounded original action
callers guide menu navigation from a retained frame12006baseline snapshot,
avoiding3530prefix repeats. Its serialized separate-UI setting overrides the
pre-load locator environment; existing BG-layer diagnostics expose Steal and
Item menus without changing runtime code. EchoGrass1200reaches Do it/Cancel,
so it is a pre-effect locator. One extra physical A1300commits the action and
makes the2200sequence positive. No RAM/snapshot edit or new fixture is needed;
the old rejected ROM-seed experiment is not reopened when reusing its baseline.
Ordinary coverage is44primary-byte reads/one record/0secondary; the secondary
bank remains targeted-only.

Evidence: `target/rom-data-metadata12/` contains windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json}, windows-build.json,
windows-cli-*.json, windows-scene-result.json and the full-state/audio log.
Protected ROM/e31/sourceproof/fixtures/input/cache/loader/canonical/candidates
remain unchanged. Global caller bounds, flag semantics and the separately rooted
global bank remain unclaimed. NewLinux/USA/nativeWindows, other gameplay/maps/
audio/GUI/archive/mod coverage remain pending; no performance, space or
AndroidPort work is included.

## Global packed pair flags source proof

The original13779c wrapper roots6e7994 through1377a2→1377b0 and discards its
incoming r0: there is no row stride. Its only direct caller1377e0 belongs to
the1377b4 live-RAM mask loop over slots0..43, mode0. This proves the minimum
44two-bit fields / 11bytes at6e7994..6e799f. The independent next root6e79a2
(D2DB0→D2F20, original byte-pair reads) leaves three zero bytes unowned.
No12-byte shape, extra slots or padding ownership is inferred from adjacency.

`global_pair_flags::{GlobalPairFlags,Catalog}` keeps eleven fixed groups of
four values0..3, exact source offset and every stored bit. Diagnostic flags
accept slots0..43 and retain u8mode selection; original signed/wrapping slot
arithmetic and raw adjacent accesses remain guest code. High-mode direct
wrapper behavior is proved; ordinary callers' high-mode use is unclaimed.
The live-mask filter scans only set RAM bits, exits at the first false low
flag and accepts an empty mask. High live-mask tail bits beyond43 are ignored.

Warning-free Windows91155 and one Wine35647 oracle pass102real cases,10raw
outside/wrapping cases and91isolated synthetic cases. These cover all44fields/
bothbits, u8mode, ignoredr0, live empty/all/sparse/tail-only/single masks, source
padding/next-root/negative slot/wrap, high bits, full44scan and early rejection.
Exact source-load address sequences/return values, callee/stack/liveRAM guards,
strictJSON/fixedgroups/pairrange/offset/source/coherent edits pass. Synthetic
sources fail authentication and remain unpublished.13601instructions/70PCs
are coverage only; there is no build or Wine failure or old oracle repeat.

Evidence: `target/rom-data-global-pair-flags/` contains bounded reader/next-root
disassembly, global-pair-flags-inventory.json, independent fields,
windows-tests-build.*, windows-original-result.json and
windows-original-europe/{global-pair-flags.json,result.json}. Protected ROM/e32/
candidate/cache/loader/canonical/sourcefixtures remain unchanged. At this
source-only boundary EUR e32-d31 /27files/26resources and USAe8-d7 are unchanged.
Managed publication/direct CPU-DMA-native/lifecycle were subsequently completed
below; positive ordinary global use remains pending. Three adjacent bytes, semantic labels, other callers/conditions and new
Linux/USA/nativeWindows checks remain open; no performance/space/AndroidPort work.

## Managed and direct global packed pair flags

EUR `v1/e33-d32` adds `global-pair-flags.json`, resource
`gameplay.global-pair-flags`, format `global-pair-flags-v1-json`. All28files /
27resources authenticate together; all26earlier decoded files/e32 generations
remain unchanged. The exact11Bsource and44two-bit fields retain their single
root and original live-mask filter; no semantic dependency, row stride or
ownership of the following three bytes is invented. USA retains e8-d7.

Typed source bytes serve CPU/native/SWI/DMA reads on page6e7, merging prior
metadata12ownership and raw tail bytes. Nonserialized source counters/11-bit
byte-position mask distinguish new global use. Original ignoredr0, signed-
wrapping slot, low8mode, live RAM-mask/earlyexit, clock and save/nativeABI remain.
A byte mask observes source-byte positions, not which bit-pair a caller selected.

Warning-free Windows61276 and two Wine84782 checks pass publication interruption,
create/reuse/repair/move/e32retention, coherent pair/tail/sourceoffset/hash/
unexpecteddependency/path/role edits, transactional attachment and unavailable
copy Load/Undo/backup/restart.28original/interpreter/compiled paths with all11raw
bytes altered match complete states/clocks;17compiled byte reads/mask1059prove
new use. Eight outside paths use0new bytes. All11B/edges/widths/mirrors/real-BIOS,
prior2metadata12+2global typed crossing with both raw sides altered, CpuSet and
FastSet exact11typed bytes and DMA3typed1raw/1typed3raw pass.3076instructions /
231dispatches describe coverage only. Source102+10+91/e32proofs were not repeated.

Windows73677 production is warning-free and restores the canonical executable.
ActualCLI13165 creates/reuses e33 without managed-file/hash/mtime changes, using
the old native cache from an unrelated working directory. Its only new-global
ordinary locator reuses2200EchoGrass:0global bytes/mask0, while metadata12still
reads44bytes, confirming the action executes. This is coverage-negative; no
full pair is run on it. Bounded original131db8..131de8gates the global filter
behind nonzero descriptor halfword4 and liveunit+e8mask. A positive ordinary
scene and corresponding full-state/image/audio pair remain pending; further
selection needs concrete evidence of those conditions. High-mode ordinary
use also remains unclaimed.

Evidence: `target/rom-data-global-pair-flags/` contains windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json}, windows-build.json,
windows-cli-{processes,no-write,summary}.json and caller-131d8c.txt. Protected ROM,
e32/sourceproof/fixtures/input/cache/loader/canonical/candidates are retained.
Three adjacent bytes, semantics/other callers/conditions, newLinux/USA/nativeWindows
and wider graphics/text/maps/gameplay/audio/GUI/archive/mod coverage remain open.
No performance, space or AndroidPort work is included.

## Evaluation parameters source proof

The original D2DB0→D2F20 literal roots 6e79a2. Its score prefix D2DB0→D2E6A
uses seven divisor/weight pairs and two conditional bonus pairs, exactly18bytes
through6e79b4. The first quotient uses unsigned division and u8 truncation;
the remaining six use signed32 division by positive u8 divisors. Products and
sum retain32-bit wrap. The two bonuses inspect separate calls to the original
live predicate, comparing its low byte with1 and0 respectively; the first
byte of each bonus pair remains structural and unused by these branches.
Live unit attributes and original equipment/getter work are not cached here.

The full D2D44 function returns0..5; D483A calls it before the D4840 zero gate
and D4844 multiply-by-four threshold path. Literal D4846→D4888 supplies address
bias6e79c2. Active classes1..5 own five4-byte records at6e79c6..6e79da: one
threshold byte and three opaque bytes each. Class0 reads no threshold and calls
no RNG. The14-byte score gap and4-byte row0 remain unowned. The next independent
root6e79da (04B45C→04B49C, original*10pointer publication) corroborates separation.
Earlier whole32/24-byte gap assumptions are rejected;38owned bytes reconstruct
without overlap with e33. No actor/action name or foreign dependency is inferred.

The strict `evaluation_parameters` model keeps fixed7/2/5arrays, IDs, order,
offsets, all coefficient/bonus bytes and opaque tails. Its finite diagnostics
retain original arithmetic. A zero divisor yields diagnosticNone; the original
guest helper remains unchanged and its zero-divisor path is outside this proof.
Threshold comparison uses signed low16 draw modulo256, then a signed comparison
with the u8 threshold. Actual02868LCGupdates030034b0 and returns15bits, so negative
draws are tested only in isolated original-code suffixes with injected arguments.

Warning-free Windows60074 and one Wine42263 oracle pass132actual-source cases:
12live score prefixes,49arithmetic/12bonus fragments,39RNG/class0 prefixes and
20raw-draw suffixes.52mutated-source cases cover high divisors/weights, signed
values, u8 truncation/wrap, conditional arguments, thresholds0/255 and opaque
high bits. Exact source addresses, values, branches, original RNG state, register/
stack/liveunit/context guards and strictJSON/fixedshape/ID/order/offset/source/
coherent edits pass. Synthetic records fail authentication and are unpublished.
24147instructions/442PCs describe coverage only; no build/Winefailure or earlier
source/native/scene oracle repeat. All protected originals/e33/candidates/cache/
loader/canonical/source fixtures retain hashes.

Evidence: `target/rom-data-numeric-pairs/` contains the independent inventory,
records, bounded readers/calls/division/RNG/predicate/next-root disassembly,
windows-tests-build.*, windows-original-result.json and
windows-original-europe/{evaluation-parameters.json,result.json}. At this
source-only boundary EUR e33-d32/28files/27resources and USAe8-d7 are unchanged.
Managed publication/direct CPU-DMA-native/lifecycle/positive ordinary score use
were subsequently completed below; class1 threshold ordinary use is now verified in the acceptance prefix linked below. Classes2..5 remain targeted-only. Whole-function tails, semantic names/other consumers, unused gaps and new
Linux/USA/nativeWindows remain open; no performance/space/AndroidPort work.

## Managed and direct evaluation parameters

EUR `v1/e34-d33` adds `evaluation-parameters.json`, resource
`gameplay.evaluation-parameters`, format `evaluation-parameters-v1-json`.
All29files/28resources authenticate together; every27earlier decoded file and
e33generation remains unchanged. Both source roles preserve18score and20threshold
bytes with exact IDs/order/offsets. No semantic foreign dependency is inferred.
The14-byte score gap/4-byte row0 stay unowned. USA retains e8-d7. Parsing limits
bound malformed input; no extraction-space or performance target is measured.

Typed bytes serve original CPU/native/SWI/DMA reads on page6e7, merging raw
bytes at all four boundaries. Nonserialized divisor/weight/bonus/opaque/threshold
counters and a five-class mask distinguish new use. Original live producers,
unsigned-u8 first quotient, six signed32 divisions, wrapping score, conditional
low8predicates, RNG/signed-low16 threshold comparison, clock and save/nativeABI
remain active. Diagnostic scoreNone for a zero divisor supplies no guest fallback;
that helper edge remains unclaimed, with the guest routine unchanged.

Initial Windows51951compilation succeeds with an unused-model import warning,
which the strict wrapper rejects. Pre-Wine review also replaces isolated native
fragments whose stop addresses overlap the complete score trace with27complete
score/shared-threshold prefixes. Corrected68404build is warning-free; the source
132+52oracle already proves isolated arithmetic/bonus fragments and is not rerun.
Wine32104two checks pass publication interruption/create/reuse/repair/move/e33
retention, coherent divisor/weight/unused-bonus/opaque-threshold/source/dependency/
path/both-role guards, transactional attachment and unavailable-copy Load/Undo/
backup/restart. No Wine failure occurs.

27original/interpreter/compiled prefixes with all38raw bytes altered match
complete states/clocks. Compiled reads consume82divisor/84weight/12bonus/
14threshold bytes and0opaque bytes; thresholdmask31covers all five classes.
Class0 consumes no new bytes. All38bytes/edges/widths/mirrors/real-BIOS, four
typed/raw word crossings, both-source CpuSet/FastSet (18/20typed bytes) and
2typed2raw DMA checks pass. All18gap/row0bytes remain raw without new counters.
13068instructions/342dispatches describe coverage only. Unused fields retain
exact bytes through typed all-byte tests, not invented original field reads.

Warning-free Windows87827production restores the canonical executable.
ActualCLI7961creates and reuses e34without managed-file/hash/mtime changes,
then uses the old native cache from an unrelated working directory. Bounded
063768→D2D44candidate evaluation justifies the retained1840Pubmission input.
The first sample is positive:15score reads,7divisors/7weights/1bonus,0opaque/
threshold bytes and thresholdmask0. One Wine71943pair matches every complete
state/clock, central-wide image, OAM, dispatch and mixed/PSG/FIFO/HLE audio
stream with those counters. The final Pub background is inspected; separated
panels do not independently establish selected-unit or completed-mission state.
That score pair reads no thresholds. The later short acceptance proof below
closes class1 ordinary use. No negative scene locator, new fixture, directRAMedit,
persisted save write or old source/e28/e33acceptance repeat.

Evidence: `target/rom-data-numeric-pairs/` contains windows-initial-warning-managed-
tests-build.*, windows-managed-tests-build.*, windows-managed-results.json,
windows-managed-europe/{managed-result.json,native-result.json}, windows-build.json,
windows-cli-{processes,no-write,summary}.json, windows-scene-result.json,
windows-pub-evaluation-parameters-full-state-audio.log, pub-dispatch.png,
class-callers.json and caller-063724.txt. Protected originals/e33/sourceproof/
fixtures/input/cache/loader/canonical/candidates remain unchanged. Whole-function
tails, semantic labels/other consumers/unused gaps, the separate11Bglobal ordinary
condition gate, newLinux/USA/nativeWindows and broader gameplay/maps/audio/
GUI/archive/mod phases remain open. No performance, space or AndroidPort work.

## Next ten-byte progression and price records

Static discovery roots6e79da through CF68E→CF6D8,04B45C→04B49C and
04B4B2→04B4DC. Original multiply-by-ten publishes a selected ROM row through
0201F530; the default branch publishes row0. Normal index producer04B3E0..
04B416reads020021b4, increments only an old value at most98 and writes a value
at most99. This establishes100normal records/1000bytes through6e7dc2. The
candidate101st row is rejected as ownership evidence; twelve zero bytes before
the independent6e7dce root (0487F6→048868) remain unowned.01A5A6displays an
adjacent progress byte and does not establish the index bound. Invalid imported
states and out-of-range indices retain original raw access; no fallback is added.

Fields00..07are eight u8 deltas. Checked original callers04B56E/580/592/5A4/
5B6/5C8/5DA/5F0pass them through original04870c to live level/progress slots1..8
with flag bits2..256. That routine retains live growth, u8addition/wrap, carry,
level99cap and behavior for an already invalid old level. Specific skill names
and global initialization preconditions remain unproved. Field08is the discount
byte read by CF674 from the live index; byte09is structural and unused here.

The price reader uses the same6e0bd0base as actual parsed Items, truncates mode
tou8/itemtou16and gates discount on itembyte0c&8. Its live predicate can add10
to the discount with u8wrap. Original signed32 base*(100-discount)/100,
unsigned comparison with minimum price and final zero→1remain distinct. These
records store no item IDs or other proven foreign keys, so no owner dependency
is inferred merely from the consumer. Strict `ProgressionRewards`/`Record` now
preserves fixed IDs0..99/source offsets/eight delta bytes/discount08/opaque09.
Validation does not constrain deltas/discount to the values observed in this ROM.
`record` returns diagnostic None for unowned IDs without changing guest access.

Windows13783build is warning-free; one reduced Wine62460original-code oracle
passes362real cases:7default/indexed publisher prefixes,232original caller→
full04870cdelta helpers and123completeCF674prices. Every row participates;
all eight delta fields, original live growth/progress/carry/u8wrap/cap99/flags
and old99/255 behavior are checked. Prices use actual source-authenticated
parsed Items, original mode2..6mask lookup/item getter and live predicate, plus
u8mode/u16item truncation, ungated reads and raw outside indices100/255.
68isolated high-delta/opaque/discount/u8+10wrap/signed quotient/unsigned minimum/
zero cases pass. Synthetic copies remain unauthenticated/unpublished. Strict
JSON/fixed arrays/IDs/order/offset/coherent edits/all1000Broundtrip/callee/SP/
complete EWRAM and stack guards pass;64148instructions/404PCs are coverage only.
The arithmetic suffix stops before its epilogue and does not claim whole ABI.
Whole progression caller, semantic skill names, managed preparation/direct
consumption/positive ordinary use and newLinux/USA/nativeWindows remain pending.

All1000bytes reconstruct without overlap with e34. Evidence is under
`target/rom-data-metadata10/`: progression-rewards-inventory.json, independent
records, discovery.json and bounded selector/field/price/live-consumer/next-root
disassembly. Current EUR e34-d33/29files/28resources and USAe8-d7remain unchanged.
New source evidence: `windows-tests-build.{jsonl,log}`,
`windows-original-result.json`, `windows-original-europe/{progression-rewards,
result}.json`, source-final-summary.json and source-final-source-hashes.json.
All37protected ROM/e34dataset/cache/loader/canonical/candidate/source fixtures
remain unchanged; no old source/native/scene proof is repeated. External
HEADe634dd8committed earlier e34/static work; preserved without agent commit.
No performance/space campaign or AndroidPort change is included.


## Managed and direct progression rewards

EUR `v1/e35-d34` adds `progression-rewards.json`, resource
`gameplay.progression-rewards`:30files/29resources with all28prior decoded
files and the retained e34generation unchanged; USA remains e8-d7. Preparation,
bounded parsing, manifest/file/source-role authentication and immutable runtime
ownership bind the exact1000Brow fields. No item foreign-key dependency is added:
these records store no item IDs. Original index publication, live growth/level/
progress/flags, mode masks/item getter/predicate/prices, instructions, clock,
save/native ABI remain unchanged. Reads outside0..99and twelve trailing bytes
retain raw access. Typed views compose with earlier resources at shared edges.

Warning-free Windows84154build and Wine80590two focused checks pass. Managed
publication interruption/create/reuse/repair/move/e34retention, coherent edits
to delta/lastrow/discount/opaque/reordered records/source/dependency/path/role,
transactional owner/state preservation and unavailable-file Load/Undo/backup/
restart pass. All1000typed bytes and all100record-mask bits are verified.
27selected original/interpreter/native paths match complete states/clocks with
all1000raw source bytes altered: compiled17delta/4discount/0opaque bytes;
6publisher/ungated/outside paths use0newbytes. Original whole-price paths and
nonoverlapping delta fragments share safe exits; isolated arithmetic suffixes
from the closed source oracle are not added to that native object.
Allbytes/record boundaries/8-16-32bit widths/three mirrors/real-BIOS flag,
12unowned trailing bytes, crossing2old evaluation+2new progression bytes,
CpuSet/FastSet32typed bytes each and DMA2typed+2raw bytes pass.3792instructions/
96native dispatches are coverage only. Source362+68proof is closed, not repeated.

Evidence: `target/rom-data-metadata10/windows-managed-results.json` and
`windows-managed-europe/{managed-result,native-result}.json`. ActualCLI25924prepare/reuse/no-write/old-cache/unrelated-cwd checks pass;
all32managed files retain hashes/mtimes. First600frame town→Shop→Buy locator
reads7discount bytes/record4(mask16), with no delta/opaque reads. One Wine52469
full pair matches every complete state/clock/central-wide image/OAM/dispatch
and mixed/PSG/FIFO/HLE audio stream, with the same7typed discount bytes.
Final shop background is visually inspected; the separated inventory panel
is absent from the raw PPM, so item selection/transactions are not inferred
from that image. Delta use at this earlier600frame boundary is targeted-only; the later e36
1000frame dispatch completion below proves all8delta fields of row0. No negative locator, new
recording/RAM edit/persisted save write or old accepted proof is repeated.
Production33726build is warning-free and canonical binary restored by hash.
Evidence: windows-build.json, windows-cli-{processes,no-write,summary}.json,
windows-scene-result.json, windows-shop-progression-rewards-full-state-audio.log,
shop-buy.png and final-summary/final-source-hashes/final-protected.json in the
same ignored directory. All protected fixtures remain unchanged.
NewLinux/USA/nativeWindows,
whole progression caller/semantic names and the broader TODO remain pending.
No performance/space measurements or AndroidPort changes are included.


## Next progression gates and returned receipts

Static inventory bounds64records×12bytes at6e7dce..6e80ce (768B). Original
0487F6→048868roots the table; offset+=12/index0..63in0487c8establish its finite
extent. Independent D23E6→D24A8roots exactly6e80cefor another stride16consumer,
corroborating the exclusive end without owning any next-table bytes. The
12zero bytes between e35progression rewards and this source remain unowned;
none of the768bytes overlap existing e35resources.

Each record preserves byte00, little-endian halfword01assembled from bytes1/2,
byte03and eight u8requirements04..0b. Byte00currently equals the physical row
index but is not read by this consumer; do not use that observed equality to
normalize/drop it or select the received-bit index. Original0487c8reads a live
u64mask at02002c08, scans unmarked physical indices in ascending order and
compares each of eight live bytes020021b6+2*k(k0..7)unsigned against the minima.
The first comparison is separate from the following seven-step loop. First
eligible record sets its physical bit in the original mask through the original
145bd0u64shift helper, writes byte03to the caller output and returns halfword01.
A miss writes output0and returns0without marking. These results are distinct:
row63has zero requirements/return/output but still marks bit63 on success.
A scalar0is therefore insufficient to diagnose a miss.

Caller04B60Epasses its stack output byte, truncates/tests the returned halfword
and only enters047670when nonzero. That receiver retains its <=375and larger
encoded-value branches. All actual64returned values fit the source-authenticated
376Items (0..375), but their complete downstream receipt semantics/foreign-key
contract is not yet proved; keep the two-byte field structural. Specific skill/
reward names and whole receipt UI behavior remain unclaimed.

Exact EUR root/stride/count/field/load/store/mask/return/caller/next-root opcodes,
all768Broundtrip and no-e35overlap pass statically. Private inventory initially
compared a decoded BL ROM offset with a guest address; the address base was
corrected before producing its successful result. The earlier047678tail locator
is superseded by the true047670function entry. That inventory did not add a
model/build/runtime check; the source contract is now proved below. Managed/
runtime/ordinaryuse/newLinux/USA/nativeWindows remain pending, current
e35-d34/30files/29resources unchanged.
Evidence: `target/rom-data-progression-gates/{inventory.py,discovery.json,
progression-gates-inventory.json,progression-gates-independent.json,
consumer-0487c8.txt,result-consumer-047670.txt,next-root-0d23b0.txt,
next-0d2420.txt,static-summary.json,static-source-hashes.json}`. No performance/
space or AndroidPort work is included; e35source/native/600Shop checks stay closed.


## Progression gates source contract

Strict `ProgressionGates`/`Record` preserves64physical IDs/source offsets,
byte00, halfword01, byte03and fixed eight-byte requirements without constraining
byte00to its observed ID, minima to0..50or returned halfword to item IDs. The
finite diagnostic `Outcome` carries matched ID separately from returned value,
output byte and updated received mask. Original0487c8and its u64shift helper
remain responsible for guest state; no routine or event work is replaced.

Warning-free Windows75099and one Wine11775original full-function oracle pass:
80actual-source cases select every physical row at its exact eight minima,
fail below each dimension, exercise multiple eligible rows/physical ordering,
bits31/32/63/fullmask/levels0-255and distinguish zero-valued row63success from
miss.25isolated copies with unrelated high byte00/high u16receipts/output128-255/
unsigned128minima test above/equal/below and early failures; synthetic copies
remain unauthenticated/unpublished. All704used source bytes (eleven per row)
are read; all64unused byte00fields are preserved by the768Broundtrip and never
read by this original consumer. Strict JSON/fixed-array/count/ID/order/offset/
coherent-edit/source/hash guards and complete EWRAM (only received mask/output
changed), callee registers/SP/stack guards pass.121113instructions/132PCs are
coverage only. No downstream receipt UI/foreign-key/semantic skill claim.

Evidence: `target/rom-data-progression-gates/windows-tests-build.{jsonl,log}`,
`windows-original-result.json`, `windows-original-europe/{progression-gates,
result}.json`, source-final-summary.json and source-final-source-hashes.json.
All38protected ROM/e35dataset/cache/loader/canonical/candidate/source fixtures
remain unchanged. External HEAD4423b91committed prior e35work and this source
implementation during compilation; its diff is inspected/preserved without an
agent commit. New managed publication/direct reads/lifecycle/ordinaryuse and
Linux/USA/nativeWindows remain pending; no old accepted proof is repeated.


## Managed and direct progression gates

EUR `v1/e36-d35` adds `progression-gates.json`/`gameplay.progression-gates`:
31files/30resources, all29prior decoded files/e35generation retained; USAe8-d7
unchanged. Strict parsed immutable records/source-role/file/manifest checks bind
all768B, with fixed-page routing on6e7and6e8and typed requirements/return/output/
unused fields. Physical record-mask diagnostics cover all64bits. Original live
levels/mask/firsteligible/64bitshift/receipt caller/instructions/clock/save-native
ABI remain unchanged;12Bpreceding gap/nextroot at6e80ceremain raw. No invented
item/receipt foreign-key dependency or byte00-ID equality constraint is added.

Warning-free Windows68535and Wine97635two reduced checks pass. Publication
interruption/create/reuse/repair/move/e35retention, coherent requirement/return/
output/unused/lastrow/reorder/source/dependency/path/role tampering and failed
attach owner/state preservation pass. An unavailable-copy Load/Undo/backup/
restart retains owned resources; all768typed bytes/all64record bits are checked.
19original/interpreter/native full0487c8paths share the same return exit and
match complete states/clocks with768raw bytes altered. Compiled counters read
369requirement/18return/9output/0unused bytes;two fullmask paths read0newbytes,
two zero-valued successes still set the physical received bit. Allbytes/record
edges/8-16-32widths/mirrors/real-BIOS flag/bothpages/pagecrossing/12rawgap bytes
and oldreward2/newgate4typed coexistence pass. Cross-page CpuSet/FastSet read
32typed bytes each; DMA reads2typed+2raw.28874instructions/158native dispatches
are coverage only. Source80+25and earlier e35proof are closed, not repeated.

Evidence: `target/rom-data-progression-gates/windows-managed-results.json`,
`windows-managed-europe/{managed-result,native-result}.json` and
managed-final-summary.json/managed-final-source-hashes.json. Production and
ordinary scene05are now verified below. The original ordinary call04B60Eis gated by
04B5FC..04B606checking live0201f504&0x1feafter the eight delta calls; context
pointer0201f508and selected row pointer0201f530remain original. It must be
reached through normal gameplay, not a RAM edit or unrelated blind Shop/Pub
replay. Source/native proof does not establish ordinary receipt/UI behavior.
NewLinux/USA/nativeWindows/downstream receipt semantics/FK/other consumers and
wider TODO remain pending; no performance/space or AndroidPort work included.


## Ordinary progression gates dispatch completion

Windows65421productionbuild is warning-free and restores the canonical binary
by hash. ActualCLI70968create/reuse/no-write/unrelated-cwd and a one-frame
old-native-cache/battle-snapshot startup pass;33managed file hashes/mtimes stay
unchanged. That startup reads0newbytes and is compatibility-only. A read-only
retained-snapshot locator inspects174EURheaders/49checksum-verified representative
payloads: none has the needed newlevel flags. It includes old targeted and
rejected-candidate states and provides no ordinary acceptance. A1840prefix
rendering probe still lacks detached panels because snapshot DialogueState
restores separation after runc's environment setting; no product code is changed.

One isolated Wine/Xvfb/Vulkan GUI session follows normal gameplay. Mad Alchemist
requires10dispatch days/no items/jobs/skills; Sigmund is sent for600gil on day9.
Normal map travel through the western node/Nubswood reaches day19. An observed
encounter on the eastern route is avoided with normal Load of the new day9slot;
no ROM/RAM/snapshot payload is edited. New private UI slots2(day9accepted),
3(day18pre-return),4(day19MissionCleared) support this previously missing
criterion; the original copied slot1and all source fixtures remain unchanged.
The inspected GUI shows Mission Cleared/unit returned,3400gil/80AP/MythrilPick/
TerreRod and skill levels8,1,11,3,2,8,42,2→8,2,11,4,2,8,42,2. Only isolated
storage/config/test cartridge files may change; no personal save is written.
GUI counters read209new requirement bytes/8delta bytes/1evaluation threshold
(class1). This does not infer newly unlocked receipts: new return/output reads0.

First new1000frame CLI replay29523from day18slot3 is positive209requirements/
8deltas. One Wine57387full pair matches every complete state/clock/central-wide
image/OAM/dispatch/mixed-PSG-FIFO-HLEaudio stream with identical counts. New
record mask6155840078674362367shows observed unmarked-row tests, not all64ordinary
rows. e35delta ordinary use is now proved for row0/all8delta fields; earlier600
Shop/362+68source/27native proof is not repeated. e36return/output and all-record
success coverage remain targeted-only. The signing/fee sequence occurs before
slot3, so this pair reads0thresholds; its separate paired acceptance prefix is
now verified below. No fullpair runs on negative
coverage and no benchmark/profiling/performance/space measurement is performed.

The shared test executable was overwritten by parallel Debug-save work. Its
e36scene validator/ROM-data implementation is checked unchanged; the current
Windows test artifact is frozen as recomp-tests-scene.exe (9e5e6f0cfa4db6c9c4af8469
11e7012a6fef2fe29db859454709a4c351623e17) before this new full pair. It includes
that parallel compiled code; the normal9-slot format/loader used here is
unchanged. The e36production candidate is46c8cb78cadb9c7e2d7d41f236135afaee39ca8e6
05b89fc504eaec412daff1d. Source/native19proof remains on its original artifact;
no current binary is falsely identified as that earlier proof. External3ea9b61
commits e36source/runtime; other Debug-save source/docs/skill changes are
inspected/preserved without agent edits or commit. Normal-play host diagnostics
remain in GUI logs without performance interpretation; volume0is not an audible
quality test. NewLinux/USA/nativeWindows/whole receipt UI/FK/other consumers/
remaining TODO stay pending.

Evidence under `target/rom-data-progression-gates/`: windows-build.json,
windows-cli-{processes,no-write,summary,dispatch-original-ui,dispatch-completion}.json,
retained-state-locator.json, windows-dispatch/{commands,result,scene-inputs}.json
and inspected captures, dispatch-completion.input, scene-executable.json,
windows-scene-result.json, windows-dispatch-progression-gates-full-state-audio.log,
final-summary.json/final-protected.json/final-source-hashes.json. Private data
and states remain ignored and excluded from commits/packages.

## Ordinary evaluation threshold acceptance

One new normal Wine GUI checkpoint captures Mad Alchemist/Sigmund immediately
before Accept conditions Yes. The isolated normal9-slot menu saves slot2;
confirmation shows the600gil fee/funds7109, then the GUI closes0. Source ROM,
original states and copied slot1 retain their hashes; no ROM/RAM/state payload
or personal save is edited. This GUI stops at the fee dialogue and reads no
threshold: the observed boundary includes confirmation and fee acknowledgement,
not the Yes press alone.

The first400frame actualCLI70012from this checkpoint reads16evaluation bytes:
7divisors/7weights/1bonus/1threshold, thresholdmask1/class1, no opaque bytes.
One Wine96264pair matches all400complete states/clocks/central-wide images/OAM/
dispatch and every mixed/PSG/FIFO/HLE audio stream, with the same positive
threshold read. The normal Yes/fee sequence closes the class1 ordinary gap;
classes2..5, whole caller and downstream outcomes retain their earlier scopes.
No gate/reward use is inferred from this short scene.

Frozen e36production46c8cb78/test9e5e6f0c, e36dataset, native cache and loader
are reused without a build. Source132+52/native27/1840score and1000dispatch
completion proofs remain closed. Evidence in `target/rom-data-progression-gates/`:
windows-accept-threshold/{commands,result,scene-inputs}.json and inspected captures,
threshold-accept.input, threshold-scene-inputs.json, windows-cli-threshold-accept.json,
windows-threshold-scene-result.json and windows-threshold-accept-full-state-audio.log.
CLI33managed hashes/mtimes and all scene input hashes remain unchanged. NewLinux/
USA/nativeWindows remain pending; no performance/space/AndroidPort work.

## Progression selection gates source contract

Original D2390 roots6e80ce and scans physical indices1..64 with stride16,
therefore the owned source is6e80de..6e84de/1024bytes. Its unused row0/16bytes
remain unowned. Independent16192→161B0roots the next source at6e84e0, leaving
another2bytes unowned. The structural name avoids assigning a rank/title meaning
to the returned selector without tracing its display consumers.

Each record preserves selector00, unsigned minimum01, little-endian unsigned
minimum02/minimum04, eight any_minima06_0d and a little-endian flag0e. Nonzero
minimum01compares live020021b4; minimum02compares02001f6c; minimum04compares the
u16result of original D22FC. Every hard minimum must pass. For the eight live
bytes020021b6+2*k, if any corresponding minimum exists, at least one active
minimum must pass. Zero minima do not count as passes. Nonzero flag0e must pass
original CCE98. No selector00==physical ID or observed-range constraint is added.

Physical order is significant. The first eligible row with at least one condition
writes selector00to020021c6. A row with no conditions explicitly writes0and stops,
even if selector00is nonzero. Exhausting64rows performs no output store, retaining
the old byte. Actual row52is the zero-condition terminal;53..64remain lossless
storage without an ordinary reachability claim. Diagnostic Outcome keeps the
matched physical index and optional output store separate.

The original function's low8input, when nonzero, first clears flags5a1/5a2/5a3/5a9
through CCECC. The original D22FCaggregate counts live flags for a separate
300-entry raw source plus flag36or a linked017fitem fallback. Those live helpers
and that separate raw source remain original; the diagnostic model takes their
result and the post-reset flag view. Six original callers pass reset0or1,
including04B464/04B4B8and063C54/063CC8. No model is published or attached to guest
reads in this step; EUR e36-d35/31files/30resources and USAe8-d7 remain unchanged.

Static opcode/root/extent/no-overlap and independent1024byte roundtrip pass.
The strict source-only Catalog requires the exact EUR ROM; fixed64records,
physical IDs1..64/order/offset, exact arrays and unknown-field guards preserve
all fields. Windows21065testbuild is warning-free. One Wine82907oracle passes
138real-source cases and80unauthenticated/unpublished mutated-source cases through
full original D2390and its original reset/count/flag helpers. It observes831distinct
original field bytes through row52; all64rows are selectable in synthetic scope.
Unsigned u8/u16/higher-bit inputs, any one of8active minima, 300live-flag counts,
flag36/linked017ffallback, low8reset0/1/0x100/0x101, selectors0/255, no-condition
zero store and exhausted no-store match. Full EWRAM changes only by original
reset flags/output; SP/callee registers/stack guards pass. 2657243instructions/
248PCs describe coverage only. Strict JSON/ID/order/source/coherent-edit guards
and all1024bytes roundtrip pass. The raw aggregate-helper source is not migrated.

Protected39ROM/e36dataset/candidate/cache/loader/canonical/inventory inputs stay
unchanged. No build/Wine failure or closed source/native/scene proof is repeated.
Managed publication, direct consumption, lifecycle and ordinary scene acceptance
remain pending. NewLinux/USA/nativeWindows/semantic display consumers are
unverified; no performance/space/AndroidPort work.

Private evidence: `target/rom-data-rank-gates/` contains independent records,
inventory, bounded consumer/helper/caller/next-source disassembly, discovery,
windows-tests-build.*, windows-tests.json, recomp-tests-original.exe,
windows-original-result.json, windows-original.log and
windows-original-europe/{progression-selection-gates.json,result.json}.

## Managed and direct progression selection gates

EUR `v1/e37-d36` adds `progression-selection-gates.json`, resource
`gameplay.progression-selection-gates`, format `progression-selection-gates-v1-json`.
All32files/31resources authenticate together; every30prior decoded file and the
e36generation remain intact. USA stays e8-d7. The same fixed physical IDs1..64,
1024byte span, source binding and source-equal JSON are verified transactionally.
No rank/title interpretation, selector00==ID or semantic foreign dependency is
inferred. Row0/16bytes and the next2byte gap retain raw ROM behavior.

Typed CPU/native/SWI/DMA reads use the model fields on source page6e8 and merge
raw/prior-family bytes at each boundary. Nonserialized counters separate
selector00/minimum01/minimum02/minimum04/any_minima06_0d/flag0e and a64bit physical
record mask. Original D2390reset/count/flag helpers, unsigned comparisons,
ANY thresholds, physical order, terminal store/no-store and every guest
instruction/clock remain unchanged. Save/native identities stay unchanged.

Two focused Wine73517checks pass publication interruption/create/reuse/repair/
move/e36retention; coherent fields/lastrow/reorder/source/dependency/path/role/
missing binding/hash and transactional attachment; unavailable-copy Load/Undo/
backup/restart; all1024bytes/64recordmask. Initial48254test compile misses the
32nd argument in an existing aggregate-guard fixture; its zero placeholder is
corrected, failed logs retained and corrected77778build is warning-free.

18full original D2390/interpreter/native paths with all1024raw bytes altered
match complete states/clocks. Compiled counts are selector15/minimum01 282/
minimum02 505/minimum04 462/ANY1635/flag162. All bytes/record edges/widths/mirrors/
real BIOS,16raw row0bytes/2raw trailing gap bytes, CpuSet/FastSet32typed bytes
each, CpuSet oldgate2/raw16/newselection2coexistence and DMA2typed2raw pass.
211068instructions/11893dispatches describe coverage only. Original setup/run
is factored unchanged to share these18new paths; the closed138+80source oracle
is not repeated. No original helper or output semantics are replaced by host code.

ProductionCLI and ordinary scene are now verified below; newLinux/USA/
nativeWindows/semantic consumers remain pending. No performance/space/AndroidPort work. Private evidence
under `target/rom-data-rank-gates/`: windows-managed-tests-build.*,
windows-initial-failed-managed-tests-build.*, windows-managed-results.json,
recomp-tests-managed.exe, windows-managed-europe/{managed-result,native-result}.json
and protected manifests. External99fb9fdcommits prior source/threshold/Debugcatalog
work; new coverage-tool changes are preserved without agent edits or commit.

## Ordinary progression selection dispatch completion

Warning-free production34174restores the canonical Windows binary by hash;
private e37candidate SHA870a964806a6d6b19e458280aa375b5a190ee8b05ccec655d31e3a192f3fd20a.
ActualCLI43381creates and reuses e37from an unrelated working directory without
changing34managed file hashes/mtimes, then replays with the existing native cache.
The first400frame Yes/fee locator reads0selection bytes; it is closed negative,
with no full pair and no change to the earlier class1threshold acceptance.

The existing normal-UI day18checkpoint and1000frame dispatch-completion input
reach original04B464/4B4B8. FirstCLI9298reads277new selection bytes: selector1,
minimum01 26/minimum02 46/minimum04 42/ANY152/flag10. Mask67108863observes the
first26physical rows, not all64ordinary selections. One Wine3138full pair matches
all1000complete states/clocks/central-wide images/OAM/dispatch and every
mixed/PSG/FIFO/HLE audio stream with the same277bytes/all6fields positive.
The inspected final image shows Sigmund/MadAlchemist success and skill levels
08,02,11,04,02,08,42,02. No new GUI/fixture, ROM/RAM/state-payload edit or personal
save write is needed. This new pair proves e37only; source138+80/native18 and
accepted e36/8delta/class1threshold proofs remain closed.

The frozen corrected77778test executable is37a243230d39db1e34a40ac3b7782b5565efe5c99
bca89ca0ca2aa23077358ed, validator84b5d4230b8386b2b50d52746ddb1b3edfe8287809e4567e26b
0fc742c461e5c. Evidence under `target/rom-data-rank-gates/`: windows-build.json,
windows-cli-{processes,no-write,summary,dispatch-completion}.json,
windows-scene-result.json, windows-dispatch-selection-gates-full-state-audio.log,
scene-executable.json and dispatch-completion.png. ROM/state/input/dataset/cache/
loader/candidates/canonical hashes are protected, CLI also checks file mtimes.
NewLinux/USA/nativeWindows/all64ordinary rows/other callers/semantic display
consumers and the rest of the TODO remain pending; no performance/space/
AndroidPort work or agent commit/push.

## Aggregate flag keys source contract

The raw ordered table at6e9a5c..6e9cb6owns602bytes:300little-endian u16slots
and terminalffffat6e9cb4. Original D2300→D2344and13FB22→13FB74independently
root the two counters. Independent29AF4→29B0Croots the next table at exactly
the exclusive end6e9cb6. All602bytes roundtrip without overlap with e37.
Other indexed lookup/publication readers50354/5165C/51E50/52110are statically
located; this does not prove their complete UI contracts or arbitrary indices.

Original D22FCcounts global flags at02001f70;13FB1Ccounts flags in the passed
context atctx+1f70. Both add0x2ffto each stored key and truncate to u16before
their original flag predicate. Count is by occurrences, preserving order and
duplicates. Affffinside a slot stops immediately; later slots remain lossless
storage. The explicit terminalffffkeeps the declared stream bounded. The exact
base contains300unique keys3..315with gaps; uniqueness/value==ID/range/foreign-key
constraints are not imposed on the model. Synthetic high/wrapping keys are
separate unauthenticated cases and are never published.

After the key loop, either live flag36or an original linked list containing
masked ten-bit ID017fadds one, with OR/short-circuit behavior. Global list head
is020028c8; context list head isctx+28c8. Original pointer chasing/flag getters/
return-u16 remain active. A diagnostic model reads a flag closure and a linked
match input; it does not replace guest code, bind an unrelated context or write
RAM. Strict fixed300slot IDs/order/offset/terminal/unknown-field/source checks
and exact602byte authentication preserve the structural contract.

Windows5092testbuild is warning-free. One Wine38100oracle passes32real-source
and26mutated-source cases through both complete counters and original flag/
linked predicates. All602source bytes are observed. Global/context flag isolation,
key+2ffu16wrap, duplicate occurrences, early slotffffat0/1/149/299, flag36ORmasked
linked first/second matches and short circuit pass. Full EWRAM stays read-only;
SP/callee/stack guards pass. 506977instructions/177PCs describe coverage only.
The previous e37source/native/scene proofs are not repeated. Protected ROM/
e37data/states/inputs/cache/loader/candidates/canonical retain hashes.

This step adds a source-only model; EUR e37-d36/32files/31resources and USAe8-d7
remain unchanged. Managed publication/direct consumption/lifecycle/ordinary
scene and newLinux/USA/nativeWindows/whole other UI readers remain pending.
Private evidence: `target/rom-data-aggregate-flag-keys/` contains independent
records/inventory/literal readers/bounded disassembly, windows-tests-build.*,
windows-tests.json, recomp-tests-original.exe, windows-original-result.json,
windows-original.log and windows-original-europe/{aggregate-flag-keys.json,result.json}.
No performance/space/AndroidPort work or agent commit/push is included.

## Managed and direct aggregate flag keys

EUR `v1/e38-d37` adds `aggregate-flag-keys.json`, resource
`gameplay.aggregate-flag-keys`, format `aggregate-flag-keys-v1-json`.
All33files/32resources authenticate together; all31prior decoded files/e37
remain unchanged. USA stays e8-d7. Strict fixed300slots/terminal/source binding
and source-equal JSON attach transactionally. Readability is retained without
an extraction-space target; malformed-file parsing guards remain bounded.

Direct typed reads project602bytes on page6e9, merging neighboring raw/prior
bytes. Nonserialized counters separate600key bytes from2terminal bytes and
track all300physical slots in five64bit words. An in-slotffffis still a keyfield;
the original counter decides its early-stop behavior. Original global/context
arguments, key+2ffu16wrap, flag36ORlinked017f/count/return-u16, every instruction/
clock and save/nativeABI remain active. No host replacement of live code occurs.

Windows76657testbuild is warning-free. Two focused Wine87184checks pass
publication interruption/create/reuse/repair/move/e37retention; coherent key/
duplicate/earlyffff/terminal/ID/order/binding/source/dependency/path/role/hash/
coverage and atomic owner guards; unavailable-copy Load/Undo/backup/restart;
all602bytes/300entry mask including the last44bits of the fifth word.

16full original/interpreter/native global and context counter paths with
all602raw bytes altered match complete states/clocks. Compiled key/terminal
counters are9540/30bytes. All bytes/entry edges/widths/mirrors/real BIOS,
preceding8/following4raw bytes, terminal-word2typed2raw, CpuSet/FastSet32typed
each plus terminal CpuSet2typed2raw, and DMA2typed2raw pass. 168888instructions/
10357dispatches describe coverage only. Original key/terminal reads and every
live predicate/context/linked pointer remain original code; source32+26is not
repeated. No build or Wine failure occurs.

ProductionCLI and ordinary scene are now verified below; newLinux/USA/
nativeWindows/whole other UI readers remain pending. No performance/space/AndroidPort work. Evidence under
`target/rom-data-aggregate-flag-keys/`: windows-managed-tests-build.*,
windows-managed-results.json, recomp-tests-managed.exe,
windows-managed-europe/{managed-result,native-result}.json and protected manifests.

## Ordinary aggregate keys dispatch completion

Warning-free production59636restores the canonical Windows executable by hash;
private candidate SHA2dfdd6096c948234e34b33cf3e3af3757771b882868e7001af436e1b63428f25.
ActualCLI44401creates and reuses e38from an unrelated working directory with
35managed file hashes/mtimes unchanged, then replays with the old native cache.
The first1000frame replay of the existing normal-UI day18dispatch checkpoint
reads1204new source bytes:1200key/4terminal, with all300physical slots in the
five-word mask (last44bits). Counts include rereads;1204is not a distinct-ROM-
byte count and does not establish both counter entrypoints in ordinary play.

One Wine11196full pair matches all1000complete states/clocks/central-wide images/
OAM/dispatch and every mixed/PSG/FIFO/HLE audio stream with the same counters/
mask. The final image shows Sigmund/MadAlchemist success and skill levels
08,02,11,04,02,08,42,02. Existing state/input suffice: no new GUI/fixture, ROM/RAM/
state-payload edit or personal save write. The new pair proves e38only;
source32+26/native16/e37same-input and earlier accepted proofs remain closed.

The frozen76657test artifact is4369f09cfb2ba859cc0d1513e50d9fd0d1916aba3ec2920b261
f4d5ce77e5af4, validatora3838598b36ba49861a5257904713ab1f270cb0e6e77c0c2af78b4d950589074.
Private evidence in `target/rom-data-aggregate-flag-keys/`: windows-build.json,
windows-cli-{processes,no-write,summary}.json, windows-scene-result.json,
windows-dispatch-aggregate-keys-full-state-audio.log, scene-executable.json and
dispatch-completion.png. Protected ROM/data/state/input/cache/loader/canonical/
candidate hashes and35CLIfile hash/mtime guards pass. NewLinux/USA/nativeWindows/
whole other UI readers/both ordinary counter entrypoints and the rest of the
TODO remain pending; no performance/space/AndroidPort work or agent commit/push.


## Adjacent halfword lookup discovery

The table rooted at6e9cb6 has16 verified literal references. The next independent
root is6e9d8a, leaving a212-byte candidate interval. Its106halfwords alone do not
prove the natural indexed domain. Generic readers dereference RAM indices;
62818mode1 uses a signed low16argument plus41, with no established0..64guard.
Fixed indices64/67 and bounded lists16..20 and21,22,4,23 are verified, but the
live producers of the other UI indices remain unresolved. An apparent6B4C0
reference is not accepted without control-flow evidence. No model or owner is
created from numeric patterns or the neighboring boundary. Reopen this partial
inventory only when a producer bound or another ownership proof is available.
Private static evidence stays under `target/rom-data-adjacent-halfword/`.

## Trade parameters source contract

Original selectorD0744 truncates its input to u8, accepts physical IDs2..6 and
returns five explicit pointers at6e85bc/6e861c/6e867c/6e86dc/6e873c. Other IDs
return null. The price readerD26AC independently uses biased root6e84fc plus
96times the u8ID. The next independent rootCFDE2→CFE14 is6e879c, establishing
five96-byte records/480owned bytes at6e85bc..6e879c. Earlier biased rows and
other indices remain unowned. A nearby counter over30live RAM rows bounds an
internal discount argument; the initial30-ROM-record hypothesis is rejected.

The strict source-only model stores physical ID, source offset,64opaque bytes,
a little-endian u16markup at40/41 and30discount bytes42..5f. Discount count0
reads byte41, also the high byte of markup. Counts1..30 read the tail. This
alias has one stored owner. The model imposes no percentage limit, header
meaning, equality across records or semantic foreign key. All480bytes roundtrip;
320opaque bytes have no price-field-read claim.

D26AC truncates its first argument to u16 and passes it to original D1F30 with
property36. That getter uses a separate70-byte source at713b3c; property36
returns byte3e times200. It is not the already decoded32-byte item catalog,
and its source is not owned by this model. Zero returns before reading any
trade parameters or live flags. Otherwise flags at02002cb4+12*ID, byte1, select
one of two paths. Both bits0and1 set select byte41+u8count and subtract
(base times percentage)/100 from base; other flags select u16markup and add
the same quotient. Multiplication and add/subtract wrap at32bits; original
146474 divides unsigned. This diagnostic arithmetic does not replace guest
code, RAM flags, division, source getter or clocks. Out-of-bank source and
discount counts above30 return no diagnostic value, never an invented guest
fallback; the zero-base early return remains defined even for those inputs.

Windows73640testbuild is warning-free. One Wine52757source oracle passes499
real-source cases,160unauthenticated/unpublished edited-source cases and15
separate arithmetic cases that explicitly inject the getter return operand.
The latter are arithmetic-tail evidence, not authenticated ROM behavior. All256
low8descriptor inputs/null outside2..6, price flags/truncation/zero skip,
count0alias/count1..30tail and all31possible live RAM counts pass. Full original
D0744/D26AC/D1F30/146474/D2728 execute. High synthetic markup proves unsigned
quotients; isolated injected operands prove32bit product/add/subtract wrapping.
All160price-field bytes are read, while320opaque bytes roundtrip only. Strict
JSON/ID/order/offset/arrays/coherent source rejection and full EWRAM read-only/
SP/callee/stack guards pass.72810instructions/184PCs describe coverage only.
146protected-file hashes including the frozen test binary match; prior source/
runtime/ordinary proofs are not repeated. No build or Wine failure occurs.

Managed publication/direct consumption/lifecycle/ordinary scene and newLinux/
USA/nativeWindows are separate pending criteria.
EUR e38-d37/33files/32resources and USAe8-d7 remain unchanged. No performance,
space or AndroidPort work is included. Static evidence remains under
`target/rom-data-stride96/`; independent fixture and the new original oracle
are under `target/rom-data-trade-parameters/`.


## Managed and direct trade parameters

EUR `v1/e39-d38` adds `trade-parameters.json`, resource
`gameplay.trade-parameters`, format `trade-parameters-v1-json`. All34files/
33resources authenticate before transactional attachment; all32previous decoded
files and e38generations remain unchanged. USA remains e8-d7. No dependency
on items.json or the separate70B source is invented. Fixed five physical IDs,
source-equal fields, source binding/role/path/hash/coverage and publication
failure guards preserve the declared480byte owner.

Typed projections on page6e8 serve CPU/native/guest-BIOS/SWI/DMA reads, merging
prior owners and raw gaps. Nonserialized counters distinguish64opaque bytes,
markup low byte40, shared byte41 and30discount tail bytes per record, plus a
five-bit physical record mask. Byte41 counts its address without inferring
which branch read it. Earlier biased rows and other indices remain raw.
Original descriptor selection, separate D1F30getter, zero early return, live
flags/count, arithmetic/division and every instruction/clock remain active.
Save/native ABI stays unchanged; immutable ownership survives Load/Undo/backup/
restart without dataset files. A shared original setup/run helper skips its
raw-source assertion only when typed ownership is attached, with no added
observer read or change to the closed source-only oracle.

Windows21694testbuild is warning-free. Two focused Wine20164checks pass
publication interruption/create/reuse/repair/move/e38retention;20coherent field/
ID/order/source/dependency/path/role/hash/coverage guards, missing-file failure
and transactional ownership; unavailable-copy Load/Undo/backup/restart. All480
bytes match after altering raw source, with320opaque/5markup-low/5shared41/
150discount-tail bytes and physical mask31.

28full original/interpreter/native paths match complete states/clocks with only
480new raw bytes altered. Compiled opaque/markup-low/shared41/discount-tail counts
are0/5/10/10: no original opaque-field price read is inferred. Descriptor/null/
zero skip/flags/count/getter remain original. All bytes, record/field boundaries,
widths, mirrors, real BIOS,222preceding raw gap bytes/4following raw bytes,
tail word2typed2raw, CpuSet/FastSet32typed each, tail CpuSet2typed2raw, earlier
selection2/rawgap222/newtrade2coexistence and DMA2typed2raw pass.
3377instructions/109dispatches describe coverage only. The closed source-only
499+160+15oracle is not repeated. No build or Wine failure occurs.

ProductionCLI and ordinary scene are verified below; newLinux/USA/nativeWindows
remain separate pending criteria.
Evidence is retained under `target/rom-data-trade-parameters/`.


## Ordinary trade parameter fee acceptance

Production46740is warning-free and restores the canonical Windows executable
by hash. The private candidate SHA is59892c74d251ff35c5144e5773d4110c69a12bd8a441
fbf67da167581e3defbe. ActualCLI17191creates/reuses e39from an unrelated working
directory with36managed file hashes/mtimes unchanged, then replays the existing
normal-UI pre-Yes slot2/threshold-accept input through the old native cache.
The first400frames read14new source bytes:7markup-low/7shared41, mask1 for
physical ID2. Counts include rereads; no14distinct-byte claim is made.

One Wine89539full400pair matches every complete state/clock/central-wide image/
OAM/dispatch and all mixed/PSG/FIFO/HLE audio samples, with the same14bytes/mask.
The final inspected image shows the Pub mission list and7109funds. No new GUI,
fixture, ROM/RAM/state-payload edit or personal save write is needed. This pair
proves e39only: the earlier class1threshold400/e38/source499+160+15/native28/
managed/lifecycle proofs remain closed and are not repeated. Ordinary discount
tail/opaque fields/all5records/other callers remain targeted-only or unverified.

The frozen21694test SHA is4070866db2c4fb6825f3f0df8e9d17dd2bf76157397ef4b677761531
f8a9bc0c, validator SHA ded241a5520283cb3b658244afd3e306f61c598e49feeda8485cbe0518cc7ed5.
Evidence under `target/rom-data-trade-parameters/`: windows-build.json,
windows-cli-{processes,no-write,summary}.json, windows-scene-result.json,
windows-fee-accept-trade-parameters-full-state-audio.log, scene-executable.json
and threshold-accept.png. Protected original/canonical/cache/loader/fixture/data/
executable hashes and CLI file mtimes remain unchanged. NewLinux/USA/native
Windows and the wider TODO remain pending; no performance/space/AndroidPort work.


## Item effect parameter static contract

Eleven verified literals root6e879c. Original getterCFE28 and helpers CFDA8/
5AAAC use7-byte records. Four independent readers root the next table at6e87fe,
exactly14records/98bytes later. Existing original item enumeration362..375/
14records and explicit subtraction of362 agree with physical IDs0..13. This
reuses the closed item enumeration proof; it does not assign item names or
reclassify the existing catalog. Arbitrary other indices remain raw/unclaimed.
The new source begins exactly at the e39trade parameter end, without overlap.

Each56-bit record stores nine5-bit coefficients in bits0..44, an8-bit cap in
bits45..52 and three opaque bits53..55. All98bytes roundtrip independently;
the contiguous bit representation matches each original split-byte getter.
Low8properties0..19 map to coefficient indices
0,0,1,1,1,2,2,3,3,3,4,4,5,5,6,6,7,7,8,8. Higher low8property values return0 before
source access. The getter truncates its record index to u8 without a local
14-record guard; the declared bank does not invent behavior outside its owner.

CFDA8 truncates slot to u8, item to u16 and quantity to u8. Its cap lookup uses
the full (item−362)u16index, while the coefficient getter uses that index's low8.
These are distinct address rules. With live current byte02002e7f+16*slot, it
returns coefficient×quantity when the sum fits the cap,0when current is already
at/above cap, otherwise (cap−current)u16. It writes no live value itself.
The UI wrapper5AAAC/caller5DA44 has extra cap>99 branches and live selection;
its complete write behavior is not yet verified.

The source model and original oracle are verified below. EUR e39-d38 and
USAe8-d7 remain unchanged; managed/runtime/ordinary/newLinux/USA/nativeWindows
remain pending. Private evidence: `target/rom-data-stride7/`, including
item-effect-parameters-{inventory,independent}.json, literal/caller maps and
bounded original disassemblies. No previous item oracle, fixture or benchmark
is repeated.


## Item effect parameter source model and original oracle

The strict source-only model stores14fixed physical IDs0..13, item IDs362..375,
source offsets, nine5-bit coefficients, cap45..52u8 and opaque53..55three bits.
JSON rejects unknown/missing fields, wrong arrays/IDs/item mapping/order/offsets
and values exceeding packed widths. Source equality rejects coherent field
changes; parsed376item-owner validation confirms all declared item identities.
The diagnostic getter preserves both low8arguments and invalid-property early
zero. The delta diagnostic requires the fullu16cap source index, leaving other
indices unowned; live current/quantity/cap arithmetic remain guest code/RAM.

Windows94173testbuild is warning-free. One Wine94185full originalCFE28/CFDA8
oracle passes540real cases,4explicit raw-cap alias probes and166isolated
unauthenticated/unpublished synthetic cases. All98bytes are observed in original
reads and roundtrip. Getter20property mapping/truncation/invalid earlyzero,
fullu16item-minus362cap index versus low8getter, live current byte, quantityu8,
fit/zero/remaining and u16return pass. Synthetic coefficients0/1/15/16/31,
caps0/1/99/100/255 and all opaque high-bit values preserve the original packed
rules. Four observed raw cap addresses outside the declared source are neither
owned nor a diagnostic fallback. Full EWRAM stays read-only; SP/callee/stack
guards pass.31740instructions/156PCs describe coverage only.

No build/Wine failure or old14371item/e39proof repetition occurs. Existing ROM,
owner datasets, saves/inputs/cache/loader/canonical and frozen original binary
retain258protected hashes. Private evidence under
`target/rom-data-item-effect-parameters/`: windows-tests-build.*,
windows-tests.json, recomp-tests-original.exe, windows-original-result.json,
windows-original.log and windows-original-europe/{item-effect-parameters,result}.json.
The static artifacts remain under `target/rom-data-stride7/`.

This source-only step retains e39-d38/34files/33resources/USAe8-d7. Managed
publication/direct consumption/lifecycle/ordinary/newLinux/USA/nativeWindows
and the full UI wrapper remain pending. No performance/space/AndroidPort work,
new gameplay fixture or personal save write is included.


## Managed and direct item effect parameters

EUR `v1/e40-d39` adds `item-effect-parameters.json`, resource
`gameplay.item-effect-parameters`, format `item-effect-parameters-v1-json`.
All35files/34resources authenticate together; all33previous decoded files and
e39generations remain unchanged. USA stays e8-d7. The original item IDs give an
explicit `gameplay.item-records` dependency, validated against the parsed376
item owner. Fixed98-byte source/ID/item/order/packed-width/source-binding/
path/role/hash/coverage guards attach transactionally.

Typed page6e8 reads merge the immediately adjacent e39trade owner and following
raw source. Exclusive nonserialized counters track bytes00..04 (coefficients),
byte05 (coefficient+cap), byte06 (cap+opaque), plus a14-record mask. Shared bits
retain one stored byte owner; counters do not invent semantic bit-read counts.
Original CFE28getter, fullu16cap index versus low8getter, out-of-bank raw cap,
live current/quantity/cap arithmetic, all instructions/clocks and save/nativeABI
remain active. Immutable ownership survives unavailable files and Load/Undo/
backup/restart; no guest routine or source outside98bytes is replaced.

Windows93110testbuild is warning-free. Two reduced Wine21177checks pass
publication interruption/create/reuse/repair/move/e39retention;22coherent field/
bitwidth/ID/item/order/source/dependency/path/role/hash/coverage guards, missing
file and transactional owner; unavailable-copy Load/Undo/backup/restart. All98
bytes match with raw source altered, counters70/14/14 and record mask16383.

50full original/interpreter/native paths with all98raw bytes altered match
complete states/clocks, including the explicit raw-cap alias distinction.
Compiled coefficient/shared05/shared06 counts are36/37/23. All7-byte record and
field edges/widths/mirrors/real BIOS,4following raw bytes, tail word2typed2raw,
CpuSet/FastSet32typed each, tail CpuSet2typed2raw, adjacent trade2/itemeffect2
owned crossing and DMA2typed2raw pass.2406instructions/134dispatches describe
coverage only. The closed540+4+166source oracle is not repeated; no build or
Wine failure occurs.

ProductionCLI and one EUR ordinary path are now verified below. Other items/
monsters/cap>99UI paths/newLinux/USA/nativeWindows remain pending. Existing bank
references identify EUR5AEE4/5E7B4; the earlier USA DS-22proof is not reused as
EUR evidence. A new ordinary EUR checkpoint is exported through guest input. Private evidence under
`target/rom-data-item-effect-parameters/`: windows-managed-tests-build.*,
windows-managed-results.json, recomp-tests-managed.exe,
windows-managed-europe/{managed-result,native-result}.json and protected guards.
No performance/space/AndroidPort work is included.


## Ordinary item effect parameters in the European Monster Bank

Production4620is warning-free and restores canonical Windows SHA f3e277db…;
private candidate SHA9f1093246640762e89383f566a55b2748ce6c219e91c227d14b45f0209b64927.
ActualCLI59404creates/reuses e40from an unrelated working directory with37
managed file hashes/mtimes unchanged. CLI50707then loads the old native cache
and replays400frames from an ordinary EUR Goblin Play checkpoint. It reads3
new source bytes: coefficient1/shared051/shared061, physical record mask1.

The retained EUR IN-13snapshot comes from the earlier ordinary boot/load of
its unchanged imported SRAM. A frozen existing Windows replay/export helper
uses only game buttons: travel to Cyril, MonsterBank→List, actual Goblin Lv4,
then Play/Release with Play selected. Normal guest state exports supply
`bank-eur-IN13/open-feed-1.ffstate`; four spaced A presses form bank-play.input.
No guest RAM/state payload, ROM, original cartridge file or personal save is
edited. The actual native-size captures show the owned monster and Play menu.

Initial BagubaPort(Pub/Shop) and Sprohm(Pub/Shop/Prison) probes did not reach a
bank. The guessed64,184target/constant long-hold speed was rejected. Existing
accepted Cyril pre-Yes state proves120,216/place18; short key pulses reach it
from IN13world112,168 without repeating boot/old asset checks. Those failed
navigation guesses remain terminal evidence, not an extraction/runtime failure.

One Wine5779full400pair matches every complete state/clock/central-wide image/
OAM/dispatch and all mixed/PSG/FIFO/HLE audio samples, with the same3bytes/mask.
The ordinary replay mixes1379881native and2741618fallback dispatches; these are
coverage context, not an all-native or performance claim. Source540+4+166/
native50/e39/earlier proofs are not repeated; this pair proves newly migrated
e40only. The final bank-background image is inspected, but serialized separate
UI omits its panels from CLI PPM (the existing capture limitation). No final
food count/confirmation is inferred from that image.

The frozen93110test SHA isa6e7037b4c6aa567945a303f9a659e1fa47114c1a9a74a920e0894154e0c11d3; validator SHA is7a1356a3218159919a4b0137737c5c11267c71caa89099c49cd359caa34ef93c.

Evidence under `target/rom-data-item-effect-parameters/`: windows-build.json,
windows-cli-{processes,no-write,summary}.json, windows-scene-result.json,
windows-bank-play-item-effect-parameters-full-state-audio.log,
scene-executable.json, bank-play.png, bank-locator-protected.json and
bank-eur-IN13/{known-cyril-*,enter-bank-*,open-monster-*,open-feed-*}.
All original/SPS/SRAM/snapshot/cache/loader/canonical/candidate/dataset hashes
and37managed CLI file mtimes remain unchanged. Other14item/20monster/cap>99UI
ordinary paths/newLinux/USA/nativeWindows and the rest of the TODO remain
pending; no performance/space/AndroidPort work is included.


## Twenty-slot job lookup source contract

The separate root `08399750` supplies one unsigned job byte per selected slot.
Six original EUR readers share it: `0D0538` returns the byte directly;
`0D0548/0D0568/0D0588/0D05A8/0D05C8` pass it as both job arguments to `0CBEC8`
with metadata52 properties1/4/6/9/0. The first four property wrappers keep their
low8result; the property0wrapper continues through the original name helper
`0165E8`. This classifies the lookup without inventing game-specific slot names.
The prior UI property-getter contract explicitly left this producer table out.

The source domain is chosen before table inspection from original `12CA98`.
It scans20live16-byte slots at `02002E78`, adding each nonnull slot's index to a
20-byte stack list. Mode3 selects the maximum live slot+5 value, with inclusive
ties; conditional on a single occupied slot, each index0..19can be selected.
The chosen index stays in r6 across `0CFF70`, which clears the live slot while
preserving r6. `12CB64/66` forwards that index to `0D05C8`. This is a declared
producer projection, with no universal gameplay reachability, whole physical
table extent, global20input cap, neighboring-root end or replacement of live
selection/RNG/slot state inferred. The readers' other u8inputs remain outside
this selected source domain.

268halfwords across three complete original code blocks,27critical pins,
nine literals and eight BL edges authenticate that source path. The original
six shared-root references are explicit. Narrow raw Thumb BL candidates for
other callers are retained separately; raw matches, including the one to the
property9wrapper, are not automatically treated as authenticated semantic
callers or universal input bounds.

The selected range is `399750..399764`,20bytes in one span, SHA-256
`969b2cd56b835c15f5bec9d962d9f007a8746ea192857bf0ac25527a44019a38`.
All20job references are distinct and resolve to20rows in the authenticated
`gameplay.linked-metadata52` resource. Each full row reconstructs its original
52bytes. The new range is disjoint from all64current manifest owners, using the
accepted e70-d69 dataset and its existing source-authentication evidence.
No sprite/animation target is followed and no duplicate metadata is selected.

This source-contract child is closed. The strict lossless model and focused
original Wine reader proof are recorded below; managed publication and typed
consumption remain pending. EUR remains e70-d69/65files64resources; USA remains e8-d7.
These checks use static reads/disassembly only, with ROM/tool/contract/model/
dataset hash/mtime guards. No build, game/test execution, fixture, navigation,
private gameplay edit, performance/storage measurement, Android/package/
assistant commit/push occurs. Linux, USA, native Windows and new32:9 checks
remain pending. Evidence: `target/rom-data-ui-job-lookup-source/`.


## Typed twenty-slot job lookup and original reader proof

The [slot-job model](../crates/gba-core/src/rom_data/slot_job_lookup.rs) preserves
exactly20ordered(slot,u8job)references at the authenticated source base. Strict
JSON/source/span/byte lookup validation and direct metadata52row references keep
lossless numeric values. The canonical Catalog authenticates the exact EUR ROM,
three original producer/reader blocks, source hash and actual metadata52data.
The twenty-entry structural projection is not a guest input limit; no original
selection, slot mutation, redirect, property extraction or name reader is replaced.

This model phase connected only two module declarations and left production
e70-d69/65files64resources and USAe8-d7 unchanged. Versioned extraction/runtime
ownership are recorded below; all earlier resource contents remain unchanged.

The first affected Windows frontend build passes warning-free, frozen
`110342c94d99d30ef88b3d9263b2e7fb29440d86e2d29ae1d39d0fcc95bcb7a5`.
One new exact [Wine test](../crates/recomp/src/rom_data/slot_job_lookup_model_tests.rs)
passes independent20entry/source comparison, JSON roundtrip, twelve structural/
JSON/dependency/source rejections and disjointness from all64current owners.
It executes123original reader paths:21bare job lookups,81complete low8property
wrappers and21property0prefixes preserving the full identifier before `0165E8`.
Three high-input aliases verify the original low8argument behavior. All20source
bytes are observed at the six actual original LDRBsites. Callee registers,
full EWRAM/IWRAM outside the original20-byte stack footprint, VRAM/OAM/palette
remain equal to their guarded inputs. These are source/reader checkpoints,
without a native-clock/full allocator/name-reader/ordinary equivalence claim.

All 231 build/232 Wine hash/mtime guards and
39 compiled input preimages remain exact. Canonical model JSON
SHA-256 is `2cea5e7cd140ce7cb19e179ce4997cf627cd514ee5410d5d6b86a9b9d7adfdb6`.

The closed268-pin/source contract and old metadata52/FFFF/A4/managed/native/CLI/
scene acceptances are reused without replay. Publication/runtime/actualCLI/
ordinary and Linux/USA/native Windows/new32:9 remain pending. No fixture,
navigation, private gameplay edit or performance/storage measurement occurs.
Evidence: `target/rom-data-slot-job-lookup-model/`.


## Managed twenty-slot job lookup and typed bus consumption

EUR e71-d70 publishes `gameplay.slot-job-lookup` in `slot-job-lookup.json`,
format `slot-job-lookup-v1-json`, with the actual `gameplay.linked-metadata52`
dependency. The generation has66files/65resources. All64previous resource
records and decoded files remain byte-identical, and the retained e70generation
is unchanged. The canonical twenty-entry model remains intact; USA stays e8-d7.

The [immutable owner](../crates/gba-core/src/rom_data/runtime/slot_job_lookup.rs)
serves exactly20u8job references at399750..399764from typed fields. Page399enters
the shared source path; counters report owned bus bytes and twenty record-mask
bits. Reads crossing either edge combine the file-derived bytes with prior
owners or raw ROM. Source addresses outside the selected projection retain
their original path. Original low8argument casts, metadata52properties/name
identifier, live slot selection, clearing, RNG and guest clocks remain live.
The twenty-entry projection introduces no guest input cap or host replacement.

One affected Windows core/frontend test build passes warning-free; the core is
compile-only. Two exact [Wine filters](../crates/recomp/src/rom_data/slot_job_lookup_runtime_tests.rs)
pass after the external disk failure described below:

- Interrupted publication/create/reuse/repair/move, all64prior records/files and
  retained e70,18transactional attach rejections and three full-set dependency
  rejections. Coherent changes to a job byte and source/file hashes still fail
  canonical authentication. Every failed attach keeps the owner and full state.
- Load/Undo, backup restore and restart share the immutable owner after its
  source directory becomes unavailable. All20altered raw source bytes still read
  correctly from the file and reach all20mask bits. Prior FFFFproperty reads keep
  their own owner and do not charge the new counters.
- Sixteen original/interpreter/compiled paths agree on complete state and guest
  clock with all20new raw bytes altered. They cover getter/property/name-ID
  prefixes, high-input alias, outside-slot20raw lookup, dynamic EWRAM/IWRAM, and
  the original12CA98mode3/single-occupied-slot19selection through CFF70clear and
  D05C8before165E8. The selected index and cleared live slot are checked; the
  name helper and ordinary actor/allocator ABI are not claimed complete.
- Both edges, widths1/2/4, three ROM mirrors and real-BIOS bus mode pass. The
  old64owner map confirms the neighboring bytes are raw. Two mixed API windows
  preserve edited raw neighbors and each charge2new bytes. CpuSet reads20new+
  4raw, FastSet20new+12raw and DMA20new+4raw, with complete state/clock equality.

The native filter observes 16 compiled job-source bytes.
All 233 build/235 Wine input hash/mtime guards and
41 compiled input preimages remain exact. Frozen frontend:
`5f80cdaaa0215b0935283cf801fe83f0ec4573eac25e28280b3154cde3c2e46a`.
Manifest SHA-256: `7ca7bef37999223b2475917c71c3cdbec164c5a6c5524c3119c2fa17a57356b7`;
dataset identity: `aa4588755ddad92406d9082335f96ca125e4c0ebad9e9bdcee0978ed06bbf97c`.

The first managed Wine attempt stopped with `StorageFull` while copying the
retained e70generation; the native filter had not started. Regenerable Linux
compiler libraries were relocated reversibly to a separate temporary filesystem,
with hash/mtime preservation. The failed log/result/scratch remain archived.
The two uncompleted filters then ran with the same frozen binaries, sources and
inputs, using temporary dataset and scratch directories. No product change or
rebuild was needed. A workspace link records the local validation root; these
private artifacts remain local and can be unavailable after temporary cleanup.
This recovery addresses an actual filesystem failure, not a storage-budget or
performance measurement campaign.

The closed source268-pin/model123-reader proof and previous families' results
are reused without replay. This closes managed publication and declared typed
bus/native paths. Actual production CLI is recorded below; ordinary positive
use/equivalence remain pending. Linux, USA, native Windows and new32:9 remain to be checked.
No fixture/navigation/private gameplay edit/performance-storage measurement/
Android/package/assistant commit/push occurs. Evidence:
`target/rom-data-slot-job-lookup-runtime/`.


## Actual production CLI for the twenty-slot job lookup

The first changed Windows production build passes warning-free, frozen
`720843b54da9089975c48ddc695cecf23c7670adc5d1f5ce3a03914a25c0c765`. The canonical executable's original hash and mtime
are restored exactly; main content is unchanged, with mtime-only Cargo
invalidation. The actual CLI is used for these checks.

The first three Wine calls pass prepare/create, no-write reuse and one-frame
old-native-cache attachment from an unrelated working directory. The isolated
resource root has66generation files/68managed files/65resources with the actual
metadata52dependency. All66generation files match the accepted e71-d70runtime
dataset, including its canonical twenty-entry job model. Reuse keeps each hash
and mtime unchanged. A workspace link to a separate temporary filesystem avoids
the actual ENOSPC encountered in the runtime stage; no further cleanup, format
change, build retry or performance/storage campaign is introduced.

The authentic read-only F09 checkpoint loads with the previous native cache and
new dataset. This single-frame result proves production attachment; it does not
establish positive ordinary use or full scene equivalence.

Generation `generation-GmxCy2`, dataset
`aa4588755ddad92406d9082335f96ca125e4c0ebad9e9bdcee0978ed06bbf97c`,
manifest `7ca7bef37999223b2475917c71c3cdbec164c5a6c5524c3119c2fa17a57356b7`.
Observed counters: `bus_reads=0 source_bytes=0 slots_used=0`.
All 304 active input hash/mtime guards and 41 compiled
input preimages remain exact, including the ROM/checkpoint/recording/native
cache/TinyCC/canonical executable and accepted runtime evidence.

Closed source/model/runtime and previous-family acceptances are not replayed.
The declared Bank120ordinary result is recorded below. Linux, USA, native
Windows and new32:9 remain pending. No fixture/navigation/private gameplay edit/Android/
package/assistant commit/push occurs. Private temporary artifacts can become
unavailable after cleanup; the recorded outcome remains separate from that
availability. Evidence: `target/rom-data-slot-job-lookup-cli/`.


## Ordinary Bank scene use of the twenty-slot job lookup

The existing EURbank checkpoint copy and its original Down8..20input are reused
for120frames,16:9/zoom0, with the actual e71production CLI and accepted dataset
`generation-GmxCy2`. The copy still matches its protected source checkpoint;
neither is rewritten, and cartridge saves are not selected. The prior A4negative/
B2positive observation remains closed; the new criterion is file-derived job
lookup use after the new owner was introduced.

The first new Bank120CLI call is positive:2bus reads/2owned job bytes/one slot
record. It selects the exact same input/state/dataset/frame range for one new
Wine fullpair. Only a five-line optional
`FFTA_STATE_TEST_REQUIRE_SLOT_JOB_LOOKUP` guard/report is added to the generic
[scene validator](../crates/recomp/src/rom_data/validation_tests.rs). Runtime,
production defaults, all previous guards, state formats and data remain intact.

One first changed Windows frontend test build passes warning-free, frozen
`f6c4291038785e3887677132e313a2a7b6f0a6313cf42180238c8d4f57553e03`.
The first exact Wine filter passes every120complete serialized Machine states
and guest clocks from frame3660through3780, central and wide framebuffers, OAM,
per-frame native/fallback dispatch and mixed/PSG/FIFOleft/FIFOright/HLE audio.
HLE floating samples are compared by bit pattern. Allfive actual sample counts
are `[263340, 263340, 31680, 31680, 263264]`. Execution is mixed; this is not an all-native claim.
The required new-owner report is2reads/2B/one slot, exactly matching the actual
CLI lead. The other19slot records retain their targeted source/runtime proof.

The CLI capture was inspected:284×160city/Bank exterior background; the bank
UI panels are absent from that raw framebuffer capture. No fully composed GUI,
whole actor/name/allocator ABI, all20ordinary slots or new32:9claim is inferred.
The fullpair compares the declared captured buffers and complete state.

All 308 active scene input hash/mtime guards and
41 compiled input preimages remain exact. The e71dataset,
checkpoint/copy/input/native cache/TinyCC/canonical executable and accepted
source/model/runtime/CLI evidence stay unchanged.

The declared ordinary child is closed. Source268-pin/model123-reader/managed/
native/CLI/F09/A4/previous scene acceptances are not replayed for their old
criteria. No new fixture/navigation/private gameplay edit/performance-storage
campaign/Android/package/assistant commit/push occurs. Linux, USA, native Windows,
new32:9 and the broader ROM TODO remain open. Evidence:
`target/rom-data-slot-job-lookup-ordinary/`.


## Live state helpers following the slot-job lookup

The exact EURentries `0D05E8`, `0D0618` and `0D0650` have a separate live-memory
classification. Their source code is authenticated independently of the preceding
job lookup; code adjacency is a discovery locator, not a data-table boundary.

- `0D05E8` returns1when any of the20first words at
  `02002E78 + 16 * index`, index0..19, is nonzero; otherwise it returns0.
  It stops at the first nonzero word. This tests nonnull words without validating
  pointed objects or proving actor liveness.
- `0D0618` counts nonzero first words of those same20live16-byte slots and
  returns0..20. It reads twenty4-byte windows, not the entire320-byte slot region.
- `0D0650` reads the live byte at `02001F68` and increments it only if its old
  value is at most98. Existing99..255values remain unchanged. No blanket clamp99
  or meaningful scalar return value is inferred.

All three ROMentry bodies ignore incoming arguments for these addresses. Their
six literal words supply fixed RAMbase/offset constants; those are executable
context support, with no referenced immutable ROMasset root in these helpers.
The mutable slot contents and counter remain guest state; exporting them into a
base resource would freeze live behavior. No new decoder/resource/owner or
revision is needed. The20loop bound does not establish the full physical extent
of the preceding ROMjob table or a global getter input limit.

68halfwords across three complete code blocks,30critical pins and six literal
words pass against the exact EURimage. Narrow even-Thumb BLencodings below370000
produce3/2/1caller candidates, retained as raw references; they are not assumed
to be every semantic caller or proof of ordinary reachability. The partial next
entry `0D0670` is outside this classification.

The source child is closed for these three entries. e71-d70/66files65resources
and USAe8-d7 remain unchanged. Existing source/model/runtime/CLI/Bank results stay
closed without replay. No build/game/test/fixture/navigation/private payload
edit/performance-storage campaign/Android/package/assistant commit/push occurs.
Static disassembly is not Linux runtime validation; Linux, USA, native Windows
and new32:9 remain pending. Evidence:
`target/rom-data-post-slot-lookup-readers/source-contract.json`.


## Isolated model for observed task parameter fragments

This e73 milestone describes the earlier two-phase model. The current e75
[phase-zero extension](#conditional-phase-zero-threshold-extension) below adds
phase0 and supersedes its current query/source-span counts; its old acceptance
remains closed.

`rom_data::observed_task_parameters` models two independent EUR fragments in
`ObservedTaskParameters`: two `PhaseThresholds` rows for phases1/2 at371065..67,
and four `VerticalCorrections` rows for indices4..7 at394E14..1C. Entries retain
their explicit phase/index, unsigned threshold or signed16 correction, plus
fixed source identity. Strict structural validation, JSON and source-byte
projection preserve all ten bytes over two disjoint spans. Exact EUR identity,
the two existing authenticated reader windows/literals and source hashes bind
the model; coherent edited fields fail source verification.

The original018DCA..18DF2 prefix increments the RAM counter first, compares its
OLD u16 value against the selected u8 threshold and advances/wraps the phase
only after that comparison. No duration units or global prelookup phase cap are
inferred. Phase0 and every other unmodeled byte remain outside this fragment.
Original0264EA..F8 shifts the task byte right once and subtracts a signed16
correction from projected r2. Both low-bit aliases are preserved. Earlier
indices0..3 retain the existing byte-property owner; indices8..127 remain outside
the model. The four-row fragment does not establish a physical-table extent or
new guest cap.

One first Windows frontend build passes without diagnostics; frozen hash
is8ce94211f4b4469de4e5a0471eb09969ede481ddf6de998d6cb157985aba8122. One reduced
Wine filter passes independent fields, all ten reconstructed bytes/two spans/
boundaries, strict JSON/negative-signed roundtrip and27structural, coherent,
unknown-field, range, source and truncation guards. Eighteen original prefixes
pass: ten phase1/2 cases with old counters0/7/8/9/65535, and eight task bytes8..15
for correction aliases and subtraction across signed/unsigned32 boundaries.
The expected RAM counter/phase changes, all other RAM, OAM/VRAM/palette and
callee/control registers agree with the model contract.

The private canonical model hash is
4b52c5f12f7633de0b1f790c5182ec5a727150a9460eb358eac3f41374788618.
All79selected input/frozen-executable guards and compiled implementations stay
exact. Both existing hubs differ only by module declarations. The production
extraction/owner pipeline, EUR e72-d71/66files65resources/current76OBJlists and
USAe8-d7 remain unchanged; these ten bytes are disjoint all current owners and
are not yet published. Completion helper retires zero paths.

The isolated model milestone is closed. Its later managed/runtime integration
is recorded below; actual CLI and ordinary new-owner checks remain pending. The closed source contract and two-
list/source/model/runtime/CLI/scene proofs are reused without replay. The452
lower audit bytes remain separate. Linux/USA/native Windows/new32:9 remain
unverified; there is no new fixture/navigation or performance/storage campaign.
Evidence: `target/rom-data-observed-task-parameters-model/`.


## Managed observed task parameters and typed bus consumption

EUR e73-d72 publishes `observed-task-parameters.json` as the bounded resource
`gameplay.observed-task-parameters`, format `observed-task-parameters-v1-json`.
The two independent field groups keep their established model and source
identities: two unsigned phase thresholds and four signed16 vertical corrections,
over exactly ten bytes/two spans. There are no fabricated foreign dependencies.
The generation now has67files/66resources; every65earlier file/record, the current
76OBJlists and the e72generation are retained. USA stays e8-d7.

The new immutable owner reconstructs only371065..67 and394E14..1C on candidate
pages371/394. It combines each byte with the preceding owner's result or raw
ROM, preserving phase0/other phase queries, old correction owners and unmodeled
indices. Original increment/OLD-counter compare/phase-wrap and signed-load/
subtraction instructions remain live. Guest timing, CAPI/native ABI, mutable
machine fields and save format do not change. Coverage counters distinguish
threshold/correction bytes and the two/four used rows; they are outside snapshots.

The first Windows build compiles the core but finds two omitted arguments in
historical frontend budget tests. Only those two zero arguments are corrected;
the archived failure and source preimage are retained. The incremental frontend
retry passes without diagnostics, frozen hash
1823c1ab2f14e02fc9e016a073c268e5ef71454a8c1b969fd75f46f04276d694.
Two first exact Wine filters pass. Managed checks cover interruption, create/
no-write reuse, repair of damaged/missing data, relocation,13transactional attach
guards and three full-set guards. Coherent edits to either field kind, malformed
or omitted rows, source plans/hashes/dependencies/revisions preserve the active
owner and complete guest state. Directory-unavailable Load/Undo/backup/restart
retain the owner; all ten bytes and six rows remain readable with raw altered.

Twenty-four original/interpreter/native prefixes have identical complete state
and guest clock with all ten raw source bytes altered. Cases include threshold
boundaries/old-u16 overflow/phase wrap, all eight low-bit correction aliases,
phase0/3/255 and correction inputs0/16/255. Both edges, all widths/three mirrors/
real-BIOS bus and six DMA/CpuSet/FastSet transfers pass. Mixed native API reads
combine2new+2old typed bytes even when both raw sources are altered, and2new+
2edited raw bytes around the phase fragment. Native API serves26typed bytes;
316original instructions/24native dispatches describe coverage only.

All67file/source checks pass. Dataset identity:
fe0839b7f50a40333d72318a573a05a85c9bb319a1f38c2060949022710ff207;
manifest SHA256ced88a054f0f999c4476605d886e3a2ba9b32b21aac96fc3bb198f87290c6bdd.
The published JSON exactly matches the model's4b52c5f12f7633de0b1f790c5182ec5a727150a9460eb358eac3f41374788618.
All74selected input/frozen-executable guards and compiled implementations stay
exact. Required completion retires only interrupted/moved fixture copies;
accepted offline data, current executable and logs/results/preimages remain.

Managed/runtime, actual production CLI and the declared ordinary12frame
comparison below are closed. Broader ordinary coverage remains separate. No completed model/source/ABI/scene oracle,
fixture/navigation or performance/storage campaign is repeated. Broader inputs,
full task/allocator ABI,452lower audit bytes and Linux/USA/native Windows/new32:9
remain separate or unverified. Evidence:
`target/rom-data-observed-task-parameters-runtime/`.


## Actual CLI for observed task parameters

CLI acceptance finishes on2026-10-27 using one warning-free Windows production
build, frozen hashc7f47bd4fdf6263845fdf0cfc3440b3e30fd79fc19a15079231d9417d4112f94.
The canonical executable's original hash/mtime are restored exactly; main source
content stays unchanged with mtime-only Cargo invalidation.

A user interruption leaves the first prepare without a current pointer or final
result. Read-only process/artifact checks confirm incompleteness; its generation
and log are retained in `aborted-cli-prepare`. The same frozen CLI then passes
create/reuse/oldcache-oneframe, all child exits0, from an unrelated directory.
All67generation files match the accepted runtime dataset, with69managed files,
66resources/two new spans10B/no dependencies. Reuse preserves every managed hash
and mtime. No accepted command or build is repeated to reconstruct context.

The first F09 frame reads one threshold byte/one phase and no corrections. Its
both-kind presence gate is explicitly negative and preserved. A new call at the
12frame bound from the retained source audit uses20reads/28typed bytes:
12threshold bytes/16correction bytes, both phases and all four correction rows.
It uses the same original checkpoint/input/cache and16:9. This establishes the
ordinary bound; it is not a complete-state equivalence result.

All145selected input/output/frozen-executable guards and implementation content
stay exact. The two userAtlas edits remain unchanged and unattributed. Artifact
completion retires zero paths and retains the interrupted preparation. Actual
CLI and the later12frame ordinary full-state pair are closed. Linux/USA/
native Windows/new32:9 and broader scenes stay open; no new fixture/navigation
or performance/storage campaign. Evidence:
`target/rom-data-observed-task-parameters-cli/`.


## Ordinary twelve-frame equivalence for observed task parameters

The declared ordinary child uses F09guest frames8476→8488 because the actual
CLI positively reaches both parameter kinds and all six rows at that12frame
bound. The first-frame correction-negative and interrupted preparation remain
closed evidence. Only five optional guard/report lines are added to the existing
scene validator; its frame loop is unchanged. The first affected Windows frontend
build passes without diagnostics, frozen hash
isde380af6bf577eafd28c2b7e3c0a87dacb22bf3dae53b922f8b29a95e3c68ae3.

One exact Wine filter with the current e73 CLI dataset, original checkpoint/
input/cache and16:9 passes every complete serialized Machine state and guest
clock, central/wide captured buffer, OAM, native/fallback dispatch and all mixed/
PSG/FIFO/bitwise-HLE audio. All five streams have actual sample counts
`[26334, 26334, 3168, 3168, 26258]`. The new guard requires both kinds/all six
rows. Its20reads/28source bytes/12threshold bytes/16correction bytes/2phases/4rows
exactly match CLI.88123native/279744fallback dispatches describe coverage only.

All76selected input/frozen-executable guards and original state/input/cache/
current67file dataset/canonical executable remain exact. The source preimage
confirms the exact five-line addition and no other implementation change; the
two userAtlas edits remain unchanged and unattributed. Required artifact
completion retires zero paths.

Source/model/managed/runtime/actual CLI and this declared ordinary child are
closed. The proof is bounded to twelve frames; broader scenes/indirect sources,
full task/allocator ABI, final composed GUI and new32:9 remain open. No prior
source/model/runtime/CLI/collector/scene acceptance, fixture/navigation or
performance/storage campaign is repeated.452lower audit bytes and Linux/USA/
native Windows remain separate or pending. Evidence:
`target/rom-data-observed-task-parameters-ordinary/`.


## Conditional phase-zero threshold extension

EUR e75-d74 extends the same `gameplay.observed-task-parameters` file with the
conditional phase0 byte at371064. The already authenticated original18DCA–18DF8
reader and18DF4 literal establish base371064; the three conditional phases0/1/2
have unsigned thresholds16/8/8. The source span grows from2B at371065 to3B at
371064. The earlier two rows and four signed correction rows are exact, giving
11source bytes in the same two independent spans3B/8B. Source SHA256 for the
three thresholds isee8357f0303dac80be96e10b1b49913c294c163771824fca6152057a6e1da957.
This neither establishes a prelookup global phase cap nor changes the original
OLD-u16-counter comparison, increment/store, wrap or outside-query behavior.

`PhaseThresholds::threshold` now supports0–2 and returnsNone for3–255. Runtime
coverage reserves three phase bits before the four correction bits, so phase0
and correction4 remain independent. The owner composes raw edges with allthree
file-derived bytes; the correction source/earlier owners and guest logic remain
unchanged. File format and count remain68files/67resources. All66other e74
resource records/files,80OBJlists and the earlier10parameter bytes are retained.
USA stays e8-d7; no Machine/CAPI/save/native-cache ABI changes.

One first Windows frontend build is warning-free. One Wine filter passes four
new original phase0 prefixes (old counters0/15/16/65535), seven new attach guards,
six original/interpreter/native complete-state-and-clock prefixes (four phase0,
two outside3/255), four explicit compiled phase0 bytes with raw371064 altered,
disjoint3phase/4correction coverage, all11typed bytes/raw edges and three DMA/
CpuSet/CpuFastSet full-state comparisons. Existing prefix helpers changed only
visibility; old1/2/vertical/constructor acceptance was not replayed.

One first Windows production build preserves the canonical executable's hash/
mtime and main content exactly. A single actual Wine CLI auto-prepare/runc call
uses the existing F09 snapshot/input/cache for a NEW28frame bound:8476–8504,
68generation/70managed files, all68equal the accepted dataset. The CLI child exits0
and uses36reads44B/28threshold16correction bytes/all3phases4rows. Its private log
matcher initially expected the explicit prepare-subcommand format; read-only
finalization recognizes `resources: created` for auto-prepare without rerunning
the command. These counts prove new phase0 use after the old1/2 wrap.

The same frontend executable then passes one Wine28frame fullpair: every
serialized state/clock, central/wide framebuffer, OAM, dispatch and all mixed/
PSG/FIFO-left/FIFO-right/HLE-bit audio comparisons. Actual samples are
[61446,61446,7392,7392,61370];207600native/650135fallback are coverage only. All
seven parameter rows are required; counts match the actual CLI exactly. The
optional five-line `FFTA_STATE_TEST_REQUIRE_PHASE_ZERO_THRESHOLD` guard leaves
the existing comparison loop byte-identical when removed. This is the newly
positive28frame criterion, not a repeat of the closed12frame milestone.

All215selected input/output/frozen hash-mtime guards and compiled source content
pass. Required artifact completion finds zero disposable paths. Dataset identity
eefc7e329ae9bb6104c63867e5ba6331f2466a41752e1237fce92757694aac86,
manifestb43670c8212648ee249874f261b10e4ce5bbaa01bcfeb102e0aa6d8b065dd351,
parameterJSON9c9de34b938d53b4a2fba244d615226b52e3f03d71e03ec0928b01db824a8bee.
No new gameplay fixture/navigation or performance/storage campaign belongs to
this phase-zero unit. Constructor-animation ordinary discovery remains a
separate partial child;371034signed-byte source, broader scenes/all-use/physical
archive extents and Linux/USA/native Windows/new32:9 remain open. Evidence:
`target/rom-data-phase-zero-threshold/` (`source-contract.json`,
`windows-phase-result.json`, `windows-cli-positive28-result.json`,
`windows-ordinary28-result.json`, `preservation-result.json`, `artifact-cleanup.json`).


## Conditional European task-cycle delta source and model

`task_cycle_deltas::Catalog` authenticates a conditional48-byte projection at
EUR371034–371064 as48signed-i8 increments. It checks the exact EUR image, the
complete18DAC–18E5C mode gate/reader/literals and the1464EC–1465A2 nonzero-divisor
helper. The source hash is
a586d9892013a2a3c8dce2bd1629b4f6c611fed7b53e6dbb4a6ba542f13db020.
Both source literals18E18/18E58 point to08371034. The full48-byte projection is
disjoint from all67e75 resource owners; immediately following371064–371067
already belongs to the independent phase-threshold resource.

The entry18DAC copies its task argument to r4 and flags from task+10hex to r5.
A nonzero task firstword and flag mask02 enable the update. Mask04 selects the
separate threshold path; when clear, the signed delta path tests mask08. It
loads the OLD unsigned byte at task+12hex, stores its increment truncated to u8,
and uses the OLD byte for the unchecked lookup. Mask08 set uses LDRSB at18E10
and subtracts the delta; clear uses LDRB at18E2A followed by LSL/ASR24 and adds
it. Both store the low16bits into task halfword+8. No coordinate axis, physical
unit or task lifetime is inferred from that field.

Afterward the reader reloads the already stored u8 increment, passes it as r0
with r1=48 to1464EC and stores the return as the new phase. This helper returns
the unsigned remainder in r0: inputs below48 return directly; larger inputs
use scaled subtraction, track speculative fractional subtractions in r2 and
reverse those fractions at146574–14659C before returning. The fixed nonzero
argument excludes the divisor-zero error branch. MOV PC,LR preserves Thumb.
Thus newphase is `old.wrapping_add(1) % 48`, including old255→0, rather than a
remainder computed before the byte wrap. The0–47domain follows normalization;
it is not a check before the original lookup or proof of every initializer.

The lossless JSON model has a fixed source offset and ordered phase0–47 entries
with signed-i8 delta fields, rejecting unknown fields and invalid signed ranges.
`delta(48..255)` and byte queries outside the projection return None. Source
verification rejects coherent value edits. Guest lookup indices48/49/50 read the
previous phase-threshold owner; other indices may reach other owners/raw ROM.
No guest clamp or arithmetic replacement is introduced. Managed publication and
direct runtime consumption are separate remaining children; this isolated model
keeps e75-d74/68files67resources and USA8-7 unchanged.

One first Windows test build is warning-free; the first reduced Wine filter
passes all48rows,23rejection guards,58original reader prefixes and eight helper
boundaries. These include signed add/subtract and low16overflow, prelookup
outside indices48/49/50/254/255, stored-u8 wrap and complete RAM/callee-register
guards. All77selected input/frozen hash-mtime guards and compiled sources pass. No ordinary scene or managed/native consumption is
claimed by this source/model test. Linux/USA/native Windows/new32:9 remain to do;
no performance or storage measurements. Private source/model evidence is under
`target/rom-data-task-vertical-deltas/`.


## Managed task-cycle deltas and typed consumption

EUR e76-d75 registers `gameplay.task-cycle-deltas` in
`task-cycle-deltas.json`/`task-cycle-deltas-v1-json`, one48-byte source span and
no immutable dependencies. It appends the resource after the67prior entries,
leaving their file bytes, records and coverage order intact:69generation files/
68resources. USA stays e8-d7. Preparation, source equality, file hashes, coverage
and ordered source binding use the existing transaction boundary. Coherent
signed-value edits and metadata/source edits must fail without replacing the
attached owner or changing serialized guest state.

The private typed owner serves only371034–371064 on page371, composing every
byte with an earlier owner or raw memory. In particular an unaligned C-API
read32at08371062 includes the final two new delta bytes and the first two old
phase thresholds; each owner supplies its own bytes. Indices48/49/50 still reach
that earlier owner. The original task's sign handling, byte increment, wrapping,
helper call and stores execute unchanged; there is no guest input clamp. A
48bit usage mask tracks observed rows outside guest serialization, alongside
bus-read and source-byte counts. These are coverage counters, not performance
measurements. No Machine/C-API/save-state/native-cache ABI changes.

One first Windows frontend build is warning-free. One first focused Wine
filter passes69files/68resources with all67previous resource records/files exact,
injected interruption at the new file, nine transactional attach failures and
two full-set presence checks. Eight original/typed-interpreter/compiled-native
prefixes match complete serialized state and guest clock with all48raw source
bytes altered; five explicit compiled new-byte reads prove native consumption.
Both signs and stored-byte/helper wrap are covered; outside48/254/255 preserve
prior/raw reads. The106compiled instructions/37dispatches are coverage only.

The same filter verifies all48typed rows, mixed read32at371062 with two bytes
from each owner, raw outer edges and ROM mirrors, plus three fullstate-clock
DMA/CpuSet/CpuFastSet transfers crossing into the prior threshold owner. The
attached owner survives unavailable files, failed reattachment, state load and
slot Load/Undo, with continued typed reads from altered raw bytes. All154actual
input/output/frozen hash-mtime guards and compiled implementation hashes pass.
Required artifact completion removes the disposable interrupted copy, retaining
the69-file accepted dataset, original inputs, logs/results, preimages and frozen
executable. Dataset identity:
d4f66549ea79aa8c45be7f44484056d2f92daeb754a459f09826473635368254;
manifestbd0881e0c16abbf9602a1faefd2243defbd64be3c884b37962b24081060ddf3b;
cycle JSON2e5b3fab16b60356084606eba7ed856a7e6c0bae5cde6e1abcbd7a9a333a3f45.

The completed source/model filter above remains closed. Actual production CLI
and a positive ordinary48frame scene are verified below.
Linux/USA/native Windows/new32:9 remain pending. Evidence locator:
`target/rom-data-task-cycle-runtime/`.


## Actual CLI for task-cycle deltas

One first Windows production build is warning-free; the frozen CLI hash is
d83ade6ca106d2dfe95945326d48188e79148e3a591e65e368f0dbd3ad30e2bb.
The canonical executable hash/mtime is restored exactly and main source content
is unchanged (only Cargo invalidation mtime updated). Three first Wine commands
pass: explicit prepare, reuse and runc with zero frames. All69generation files
match the accepted Rust dataset, all68runtime resource IDs/hashes match the
manifest, the new owner registers48rows and the existing native cache loads.
The entire71-file managed path set, hashes and mtimes remain unchanged after
reuse and startup. All226actual input/output/frozen guards and compiled source
hashes pass. There are zero guest frames/native dispatches/fallback steps and
zero cycle-read counters: this proves attachment, not positive ordinary use.

Dataset identity/manifest/cycle-file hashes match the runtime evidence above.
Private evidence under `target/rom-data-task-cycle-cli/` includes
`windows-build.json`, `windows-cli-processes.json`, `windows-cli-no-write.json`,
`windows-cli-summary.json`, `preservation-result.json` and cleanup receipt.
A source-backed normal scene reaching the signed cycle mode is now verified
in the ordinary48frame criterion below. No task flag was manually forced. Linux/USA/native Windows/new32:9 stay
pending, with no new gameplay fixture or measurements in this CLI unit.


## Ordinary European task-cycle scene

The signed-cycle family now has one complete ordinary EUR/Wine acceptance on
a newly selected48frame neutral-input Bank bound, guest3660–3708. It uses an
exact byte/hash/mtime copy of an existing normal Bank snapshot, the current
actual-CLI69-file e76-d75 dataset, and the unchanged original native cache. No
navigation, guest RAM/ROM/task mutation, cartridge save write or replay of the
old Bank12/F09/source/model/runtime acceptance was needed.

Source inspection found the initializer at18D6C: low-u8 r3zero selects default
flags03 and resets phase0, with callbackr2/context r1. The generic015E48 caller
selects03or07 according to owner halfword+14mask0100 and adds08 for maskC0==80.
The raw Thumb-BL inventory has11initializer,3update and89flag-update candidates;
these are leads with inspected windows, not a complete authenticated call graph.
Read-only header/payload-hash-validated inspection of three existing normal
snapshots finds Banktask02023CA4=root02022F64+D40, flags03/phase21/context02023C84/
callback08018FC5. Its address matches the5C1F8 flag-update caller. That caller's
argument03alone is not a universal signed-mode proof: helper18E5C preserves old
mode04/sign08 when the new flags include01. The snapshot supplies actual03mode
evidence. F09start/end candidates instead have07/threshold mode. RAM candidates
alone are not liveness proof, and no particular task is attributed to each read.

One existing frozen production CLI observation of the selected neutral48frames
exits0 and consumes48reads/48source bytes/all48phases. The managed71-file pathset,
hashes and mtimes remain unchanged. No production rebuild is repeated. One first
Windows frontend build for a five-line optional positive-use guard is warning-free,
frozen843b6f39bfc277e7b934b1e8ab1b803753f75d645de3f088a04a7604c8cfe77d.
Removing only `FFTA_STATE_TEST_REQUIRE_TASK_CYCLE_DELTAS` restores the old
comparison module byte-for-byte.

One first exact Wine fullpair passes every serialized state/guest clock,
central/wide framebuffer, OAM, dispatch and all mixed/PSG/FIFO-left/FIFO-right/
HLE-bit audio comparisons across those48frames. Actual sample counts are
[105336,105336,12672,12672,105260];96165native/333103fallback are coverage only.
The guard requires all48rows and its48/48/48counts exactly match the actual CLI.
All97selected input/output/frozen hash-mtime guards, compiled implementations
and the old-loop preimage pass. Required artifact completion finds zero
eligible paths. Private evidence under `target/rom-data-task-cycle-ordinary/`
includes source/caller/snapshot reports, `windows-cli-positive48-result.json`,
`windows-scene-build.json`, `windows-ordinary48-result.json`, preservation and
cleanup receipts; snapshots/input/image remain outside Git.

This closes the declared cycle source/model/publication/runtime/CLI/ordinary
children, while full ROM migration remains open. Coordinate axis/unit, every
initializer/task lifetime, complete physical extent/all-use and other scenes
are unproved. Constructor-animation ordinary coverage remains independently
partial; its four negative routes are not reopened. Linux/USA/native Windows/
new32:9 remain pending; no performance or storage campaign.


## European encounter unit templates and shared ownership

The authenticated442 encounter-header backlinks now define a complete physical
partition of2,850 unit templates at `6EA048..70B6A8`:48 bytes per record,
136,800 bytes with SHA-256 `15cc1b488912bf0609d6e2bc45cd3f8b28fcb5eb414b1aa3b32fac60199d636e`. The cached header counts and
pointers match every contiguous group and the original CE18C importer advances
by48. Observed counts1..13 are data values, not a guest limit. The importer
still uses its live `r2` bound and skips occupied264-byte unit slots. This is the
header-linked catalog; no census of all possible template sources is claimed.

All records reconstruct independently, preserving the following layout:

- Five separate bytes00..04, with descriptor/job selectors and original zero
  fallback/override/generated-level branches retained.
- Three bytes05..07 copied to live unit fieldsF6..F8; five equipment halfwords
  08..10, also copied as exactly10 bytes by the original helper; one full
  halfword12 loaded before truncation to live unit byte105.
- Twenty physical bytes14..27 used by the dynamic bit reader, separate bytes28
  and29 for optional live substructure fields, and full16-bit flags2A ORed into
  the existing live flags.
- Separate control bytes2C and2D: nonzero2C gates the original callback that
  replaces live unit word0 and sets flag106; nonzero2D writes live unit byte100.
  Bytes2E..2F remain opaque for the inspected importer/helpers. Their original
  zero values do not authorize rejecting or normalizing edited nonzero bytes.

The bit reader uses `template[0x14 + (i >> 3)] & (1 << (i & 7))`, with `i`
derived from the live unit/job count. There is no proven20-byte or160-bit guest
cap; retain out-of-projection prior/raw behavior. The physical20-byte field is
not permission to replace the original loop, randomness or live rules. Full
importer/all-caller execution remains a later verification scope.

All14,250 equipment references fit the existing376 parsed items. Actual byte00
values are special1 or valid metadata14 IDs2..94; actual byte01 values fit the
116 metadata52 job owners. These are source/reference checks, not observed-value
caps for a lossless model. Byte01 zero is still an original fallback branch.
The later unconditional metadata14 lookup for descriptor1 reaches the unowned
row1 prefix; no typed fallback or extra metadata row is invented.

The existing `gameplay.constructor-alternate-templates` owner already has
`6ECA78..6ECB38` (192 bytes), records225..228 in zero-based physical order,
all four in encounter header29. Their full48-byte projections match. The new
ownership plan must skip these bytes, retain their prior owner/statistics and
validate full-record coherence, with mandatory encounter-header and constructor
alternate dependencies. Its two new fragments are:

- `6EA048..6ECA78`,10,800 bytes,
  SHA-256 `4bc593217bf764f851ffd363838e7463d70ccbeac02eba50d534e0833ead53c5`.
- `6ECB38..70B6A8`,125,808 bytes,
  SHA-256 `bbbead1d73e9cf995a8f3a15a1d81c3dc5210fd628c4c353bf2415cb6b00e7d5`.

Together they add136,608 bytes disjoint from all49,795 source spans of the72
accepted e80 owners. The full source authentication includes the192 shared
bytes; the new byte projection must omit them. Item, metadata14 and metadata52
owners are identified separately from shared physical ownership.

Five new bounded windows complete the importer tail and its two template
helpers, reusing three closed header/constructor/metadata14 windows. All39
instruction pins, four direct BL targets, complete record projections, shared
coherence and41 actual preservation guards pass. No new build, Wine/game test,
checkpoint, global root census, old acceptance replay or performance measurement
occurs. Private evidence is in `target/rom-data-encounter-unit-templates/`:
`source-contract.json`, `reader-fields.json`, `source-windows.json`,
`partition-analysis.json`, reconstructable `template-records.json`, header groups,
two-span plan and final/preservation/cleanup receipts.

The next criterion is a strict lossless model with header/shared/reference
coherence and one focused Wine original-reader proof, followed by managed
publication, typed consumption and ordinary use. Schema1/e80-d79 and USAe8-d7
are unchanged. Coordinate payloads, global/all-caller/full-ABI proof, Linux,
USA, native Windows, new32:9 and the full ROM TODO remain pending.


## Typed encounter unit templates and original-reader proof

`encounter_unit_templates::{Record,EncounterUnitTemplates,Catalog}` now models
all2,850 header-linked48-byte records losslessly. Each record retains its
physical index/offset and encounter-header/ordinal metadata. Separate u8, u16,
fixed arrays and opaque-tail fields reconstruct the full136,800 source bytes.
Strict JSON rejects unknown fields, wrong fixed-array lengths and numeric types;
structural validation checks fixed physical count/order/offsets. It does not
require observed value ranges, zero opaque bytes or canonical flags.

Dependency validation is separate:442 encounter headers must partition the
records with matching pointers/counts/header IDs/ordinals; all four constructor
alternate records must match their complete48 bytes; equipment, descriptor and
resolved job selectors must reference actual parsed item/metadata14/metadata52
owners. Byte01 zero retains the descriptor fallback or descriptor1's direct
job0 path. Descriptor1's later raw metadata14-row1 prefix stays unowned. Full
source authentication checks the exact EUR ROM, original reader windows, full
source digest and decoded model. No guest constructor, live importer, random
rule or dynamic bit loop is replaced.

Two source spans contain136,608 newly owned bytes. `byte_at` deliberately
returns `None` for the old192-byte constructor range, preserving prior ownership.
A shared-record edit leaves both new spans unchanged, but fails full48-byte
constructor coherence and source authentication. Header grouping/reference
checks also remain distinct from lossless shape checks. The physical `record`
lookup is diagnostic; it does not introduce guest index truncation or a cap.

One warning-free core-only Windows build `e0ae0d5c..` uses512 stable source files
(224 core inputs), based on the accepted ordinary source copy plus
only the new model/test and module wiring. Only the affected core-test fingerprint
is isolated; no global clean, old source edit or frontend build occurs. One
focused Wine filter passes38 guards and2,100 selected original prefixes:
1,020 actual single-load cases across distinct source field values and boundary/
shared records, plus1,080 original dynamic-bit prefixes. The latter include120
indices beyond the physical20-byte bit field, exercising control/tail/next-record
and raw-after-catalog bytes without inventing a160-bit guest cap. Opaque means
no dedicated field interpretation: the generic dynamic reader can still alias
these bytes at out-of-field indices.

Actual addresses/widths/destinations/values, single-load register/flag/PC
preservation and dynamic-bit results/register bounds are checked. RAM, VRAM,
OAM, palette and the ROM remain exact. The selected reads cover1,531 distinct
newly owned bytes,102 prior-owned shared bytes and four raw bytes outside the
catalog; this sample does not establish execution coverage of every field in
all2,850 records. The full source projection is independently checked for all
136,800 bytes, including the192 prior-owned bytes. The17,232 guest instructions
are coverage only, not performance measurements or full importer ABI proof.

All34 actual model-unit guards,512 frozen source hashes and
41 closed source-contract guards remain exact; artifact completion retires zero
paths. Evidence is retained under `target/rom-data-encounter-unit-template-model/`:
independent model/probes/coverage/spans, source snapshot, build and Wine results,
static probe-input check and final/preservation/cleanup receipts. No new ordinary
fixture/navigation, prior constructor/header/source/ordinary acceptance replay
or performance campaign occurs.

Managed publication and typed consumption are next: e81-d80/index72,
`encounter-unit-templates.json`, with explicit header/constructor/item/metadata14/
metadata52 dependencies, source pages6EA..70B and the shared192-byte hole.
Actual CLI and an ordinary positive pair follow. Current schema1/e80-d79 and
USAe8-d7 remain unchanged. Linux, USA, native Windows, new32:9, coordinate
payloads, global/all-caller/full-ABI verification and the full ROM TODO stay open.


## Managed encounter unit templates and typed guest reads

EUR e81-d80 adds `encounter-unit-templates.json`, resource
`gameplay.encounter-unit-templates` at index72:74 generation files and73 resources.
All72 previous resource records/files remain exact, and USAe8-d7/schema1 remain
unchanged. Five explicit dependencies bind encounter headers, constructor
alternates, item records, metadata14 descriptors and metadata52 jobs. Strict
parsing, full source authentication, header partition/full48 shared coherence and
reference checks precede transactional activation.

The immutable owner projects the two authenticated fragments (136,608 bytes),
leaving the192 constructor bytes with their existing owner. Nine byte classes
separate selectors, argument triples, equipment, auxiliary halfwords, ability
bits, optional bytes, flags, controls and opaque bytes. Record/header usage masks
count newly owned reads; a shared-only read does not claim a new record or header.
Per-byte composition retains prior owners/raw bytes through mixed reads and ROM
mirrors. Both the global routing gate and local owner cover source pages6EA..70B.
Original guest algorithms, dynamic bit indexing, live importer limits, randomness,
Machine/C API, native-cache and save ABI remain unchanged. The64MiB parser input
guard bounds malformed files; no storage/performance campaign is performed.

One warning-free Windows integration build `3d4b2af2..` uses514 stable source
files from the accepted512-file model copy plus integration paths. Parallel state
diagnostics/Debug/frontend work is excluded. Only affected core-library/test
fingerprints are isolated, preserving old sources and executables. Private edit
preparation stopped on mismatched inherited presence/constructor anchors and an
isolated-script variable; only unfinished edits resumed from saved preimages.
Budget-call arity was checked statically before this first build. No failed build
or replay of the closed source/model/header/constructor/ordinary proofs occurs.

One reduced Wine filter passes new-file interrupted publication,14 attach guards
and seven full-set guards, retaining the prior owner and complete machine state
on activation failure. The27 original-ROM/interpreter/compiled-native prefixes
match complete states and guest clocks with all136,800 raw template bytes
poisoned after Load/setup. They include actual byte/halfword loads and dynamic
bit indices beyond the physical20-byte field, shared/next-record/raw-after-end
reads and ROM mirrors. Native calls explicitly consume
25 newly owned bytes and8 prior-owned bytes;
32 compiled instructions are coverage only, not full importer ABI proof.

The full bus proof reads all136,608 new bytes plus192 old bytes while raw source
is poisoned. Exact class counts are14,230 selector,8,538 argument,28,460 equipment,
5,692 auxiliary,56,920 bit,5,692 optional,5,692 flags,5,692 control and5,692 opaque
bytes. It touches2,846 new records and441 headers; the four old records/header29
stay with the constructor owner. Widths, mirrors, shared-hole crossings, page
boundaries, changed raw outer bytes and an older owner also pass. Load, slot
save/load and Undo retain the immutable owner while dataset files are unavailable.

DMA3, CpuSet and CpuFastSet match complete states/clocks across the full template
range and a raw suffix. DMA uses34,202 words in32-bit mode, CpuSet68,404 halfwords
and CpuFastSet34,208 words. All destinations start at EWRAM10000 and fit; no
oversized16-bit DMA length or wrapping destination is used. All215 actual
guards,514 integration source hashes and512 closed model source preimages remain
exact. Artifact completion retires only the interrupted dataset copy, retaining
the accepted offline dataset, source preimages, executable and results.

Private evidence is retained under `target/rom-data-encounter-unit-template-runtime/`:
build/filter results, native and bus/lifecycle result, final/preservation/cleanup
receipts, accepted dataset and source snapshot. Dataset identity is
`2b624164053d24680f9a7914a5a1469ef38a62f813c78217456488379079af90`, manifest SHA-256
`7b2e21ae7690aa07028599957faaa74dfe4fe46d4e395632b088ddbb94824d53` and new JSON SHA-256
`037cd9f9783fc98b95fb9b69d05fb1e09bf2f3888072c7bd17f9ef6e42153788`.
Actual production CLI/create-reuse/no-write/startup and one positive ordinary
pair are next. This integration is not full importer/ABI, all-caller/global
archive or coordinate-payload proof. Linux, USA, native Windows, new32:9 and the
full ROM TODO remain pending; no new ordinary fixture/navigation or metrics.


## Actual encounter unit-template CLI and ordinary consumption

The actual e81-d80 Windows production CLI now passes three Wine commands:
prepare creates the generation, a second prepare reuses it without writes, and
normal `runc` enters White Flowers from the retained Giza menu. All74 generation
files match the accepted runtime dataset; all73 logged resource IDs/hashes match
the manifest. All76 managed files retain hashes and mtimes through reuse and
normal startup. The five dependencies and two source fragments retain the192
prior constructor bytes. The original28859 translated blocks load.

Exact normal state/input copies cover guest19409→20109 (700 frames). Template
usage is `{"ability_bit_bytes": 41, "argument_bytes": 15, "auxiliary_bytes": 10, "bus_reads": 186, "control_bytes": 10, "equipment_bytes": 80, "flag_bytes": 10, "headers_used": 1, "opaque_bytes": 0, "optional_bytes": 10, "records_used": 5, "selector_bytes": 35, "source_bytes": 211}`. Positive byte classes are
`selector_bytes, argument_bytes, equipment_bytes, auxiliary_bytes, ability_bit_bytes, optional_bytes, flag_bytes, control_bytes`; unobserved byte classes are `opaque_bytes`.
The selector/argument/equipment/flag positive-use criterion is satisfied before
selecting one ordinary full equivalence pair. Usage counts are correctness
coverage, not performance measurements. This single battle-entry case does not
establish all2,850 records, all442 headers, every branch or full importer ABI.
Any zero class remains covered separately by the targeted bus/source/native
proofs, without adding navigation merely to increase a counter.

One warning-free production build `afeaef25..` uses the accepted514-file runtime
source copy, excluding unrelated Debug/frontend changes. Only the affected
production fingerprint is isolated; unchanged core artifacts are reused and the
canonical executable's hash and mtime are restored exactly. All184 actual
guards and514 frozen source hashes pass; artifact completion retires zero paths.
Evidence is retained under `target/rom-data-encounter-unit-template-cli/`: build,
three command logs, no-write/summary/final/preservation/cleanup receipts, exact
normal copies and the production executable. The actual dataset is
`target/rom-data-encounter-unit-template-cli/windows-cli/resources/8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632/v1/e81-d80/generation-GJRgcu`.

No new fixture/checkpoint/navigation, closed model/runtime/header/constructor
acceptance replay or performance campaign occurs. One ordinary full pair with
all13 usage fields equal to this CLI run is next. Linux, USA, native Windows,
new32:9, coordinate payloads and the remaining ROM TODO stay pending.


## Ordinary encounter unit-template battle-entry equivalence

One Wine ordinary equivalence pair now closes the retained700-frame Giza menu
→White Flowers deployment case, guest19409→20109, using the actual e81-d80 CLI
dataset. Every complete machine state and guest clock matches the original-ROM
path. Classic, central and wide images, OAM, native/fallback dispatch and all
five audio streams also match. Dispatch counts are `[3316146, 8054486]` and audio
sample counts are `[1536150, 1536150, 184800, 184800, 1536026]`. These counts are correctness/coverage
evidence; no performance measurement or comparison is made.

All13 unit-template usage fields equal the production CLI exactly:
`{"ability_bit_bytes": 41, "argument_bytes": 15, "auxiliary_bytes": 10, "bus_reads": 186, "control_bytes": 10, "equipment_bytes": 80, "flag_bytes": 10, "headers_used": 1, "opaque_bytes": 0, "optional_bytes": 10, "records_used": 5, "selector_bytes": 35, "source_bytes": 211}`. The eight semantic byte classes are positive,
covering five template records in one encounter header. Opaque ordinary usage
remains zero, covered separately by the source/full-bus proof. This is one
positive battle-entry case, not all2,850 records, all442 headers, every conditional
initializer path, full importer ABI or global archive coverage.

An optional five-line `FFTA_STATE_TEST_REQUIRE_ENCOUNTER_UNIT_TEMPLATES` block
requires positive selector, argument, equipment, auxiliary, bit, optional, flag
and control-byte use, and reports all13 usage fields. Removing only that block
reproduces the prior validator byte-for-byte, preserving every older guard. One
warning-free514-source Windows test build `33c61082..` and one Wine filter pass.
Only the affected test fingerprint is isolated; core code is unchanged from the
accepted runtime. All104 ordinary guards/514 frozen pair-source hashes and
184 closed CLI guards/514 production-source hashes remain exact.
Artifact completion retires zero paths.

Evidence is retained under `target/rom-data-encounter-unit-template-battle-ordinary/`:
validator preimage/added block, stable source copy, build, full-pair result and
final/preservation/cleanup receipts. No new fixture, checkpoint or navigation,
closed source/model/runtime/CLI-registration/header/constructor acceptance replay
or metrics occur. Linux, USA, native Windows, new32:9, coordinate payloads and
the remaining ROM TODO stay pending.


## European indexed 28-byte metadata source contract

The EUR table rooted at710E04 now has a declared physical source contract:
347 records of28 bytes, indices0..346,9716 bytes ending at7133F8, SHA-256
`68cea1f29a7c4a6a50e2184538d2df954c00d792f84eb59405643221410fa192`. Seventeen exact root literals and their PC-relative
loaders use28-byte arithmetic. Two independent consumers root the following
family at7133F8 and use four-byte indexing and different byte fields. These
start/stride/successor-type witnesses establish the declared physical partition;
347 is not a guest-input limit. The proposed resource name is
`gameplay.linked-metadata28`, without inferring an ability, law or mission name.
All records reconstruct byte-for-byte and are disjoint from all74 accepted
resources/50488 source spans in e82-d81.

The original getter D07A4 narrows its index to low16 and selector to low8.
Selectors0/1 read unsigned halfword+0/byte+2;2..8 read bytes4..10;9 returns the
original source address+12;10 returns the full word+16. Selectors11..31 test
bits0..20 of that word and normalize to0/1. Selectors32/33 read halfword+20 and
byte+11. Selectors34..255 return0 without a record-data load. Address computation
remains unchecked; index347 and higher must preserve raw or other-owner behavior.
No logical index cap or global null-record fallback is introduced. Row0 is
physical data and has byte3=1, even though many of its other bytes are zero.

Direct readers additionally establish unsigned halfword+22, branch-dependent
byte+24 dispatch, byte+25 comparisons with2/3 and byte+26 reads. The latter has a
separate zero-alias path through existing metadata52. Bytes3 and27 remain opaque.
Selector9 exposes an inline four-byte region12..15; one original consumer loops
through its first three byte indices0..2 and uses the following four-byte family.
Byte15 remains uninterpreted in the inspected loads. The returned view itself
has no new three-byte bound. All full flags, raw values and opaque bytes stay
exact; observed values do not become enums, Boolean storage or zero policies.

Twenty authenticated windows (14 new/six reused) and69 instruction/literal/
jump-table pins pass. Additional successor/neighbor windows remain exploratory
diagnostics. All122 actual guards pass; artifact completion retires zero paths.
Evidence under `target/rom-data-linked-metadata28/` retains the source contract,
reconstructable records, span, complete getter and direct-reader windows,
successor candidates, pins and final/preservation/cleanup receipts. This unit
uses static original-code inspection and byte/hash checks. It runs no build,
Wine/game test, checkpoint, navigation or performance measurement.

Next is a strict lossless model and focused Wine original-reader proof, then
managed publication, typed consumption, actual CLI and ordinary use. Schema1/
e82-d81 and USAe8-d7 remain unchanged. The following four-byte family, all indirect
callers/full ABI/global archive, Linux, USA, native Windows, new32:9 and the full
ROM TODO remain pending. Closed coordinate/header/template proofs stay closed.


## Typed indexed 28-byte metadata and original-reader proof

The standalone core `rom_data::metadata28` model now retains347 ordered records
and every9716 source byte in typed unsigned fields, fixed byte arrays, a full
32-bit flag word and explicit opaque bytes. Shape checks validate the fixed
source/count/physical index order; they impose no observed value caps, enums,
opaque-zero policy or global null fallback. Exact EUR ROM identity, all20
source-contract windows and source hash authenticate `Catalog::from_rom`.
`verify_source` separately checks that decoded values equal the authenticated
base. JSON rejects unknown/missing fields, wrong array sizes and wrong numeric
types or widths. `source_spans` publishes one disjoint source plan; managed
publication and runtime attachment remain the next integration criterion.

`record` narrows to low16 and returns `None` outside physical ownership; this is
not a guest fallback. `lookup` narrows the selector to low8, preserves scalar/
full-word/21 bit-query behavior and distinguishes `SourceAddress` from `Scalar`.
Selector9 returns the encoded original address+12 even outside the owned table;
selectors34..255 return0 without requiring record data. Other unowned scalar
reads remain `None` for later raw or prior-owner composition. The helper neither
turns the inline view into a host pointer nor adds a three-byte bound.

One warning-free Windows core-only build `358dba2c..` uses a frozen520-file copy
of the accepted518-source ordinary unit with only the core module declaration
and two new leaves. Only the exact core-test fingerprint is isolated; parallel
Debug work is excluded from this proof. One focused Wine filter passes27 shape,
JSON and authentication guards and14517 actual original-code prefixes:
12088 full D07A4 getter calls (all347 records ×34 selectors, all222 default
selectors once, low16/low8 aliases and raw outside indices),1388 unsigned
single-load tail prefixes and1041 five-instruction inline/successor
prefixes. They observe all8675 interpreted source bytes. The other1041 bytes
(positions3/15/27 per record) retain exact serialization and bus projection.
The selected inline prefix preserves its raw following-family lookup.

Nine isolated source variants, never ordinary checkpoints, exercise unsigned
high bytes/halfwords, full-word bit31, queried bit20 and changed opaque bytes.
All342 additional original prefixes agree and source authentication rejects
these modified models. Original RAM outside the retained four-byte stack word,
callee registers, SP/LR, VRAM, OAM, palette and final ROM bytes remain exact.
The `347169` executed instructions describe test coverage only;
no host performance or guest-clock equivalence claim follows from this model
oracle. Native full-state/clock proof belongs to runtime integration.

All36 actual model guards/520 frozen source hashes and122 closed source
contract guards remain exact; artifact completion retires zero paths. Evidence
under `target/rom-data-linked-metadata28-model/` includes independent values and
probe plans, isolated variants, stable sources, build/test logs, executable and
final/preservation/cleanup receipts. Schema1/e82-d81 and USAe8-d7 stay unchanged.
Next are managed e83-d82 publication and typed consumption, then actual CLI and
ordinary use. Full caller ABI, the following four-byte family, global archive,
Linux, USA, native Windows, new32:9 and the full ROM TODO remain pending.


## Managed indexed 28-byte metadata and typed guest reads

EUR e83-d82 adds `gameplay.linked-metadata28` at resource index74, serialized as
`linked-metadata28.json` / `linked-metadata28-v1-json`. The generation contains
76 files and75 resources. All74 previous resource records and decoded files
remain exact; schema1 and USAe8-d7 stay unchanged. Its single9716-byte source
span710E04..7133F8 is disjoint from previous owners and declares no invented
cross-resource dependency. Source reconstruction, file hash, source span/role
and exact decoded-base equality are checked before transactional attachment.
Existing fixed input guards retain their parser-safety role; no space campaign
or compression optimization is performed.

The immutable owner holds the typed model and six record-mask words. Its eight
counters report reads, source bytes, halfword-field bytes, scalar-byte bytes,
inline bytes, flag bytes, opaque bytes and distinct records used. Five classes
partition each physical record6/12/3/4/3 bytes. The inline fourth byte15 remains
opaque alongside bytes3/27; all values stay exact. Local and global source-page
selection include710..713. Every read composes per byte with the prior owner or
raw ROM, so the header to the left and raw four-byte successor to the right keep
their behavior and counters. Low16/low8 indexing, scalar/address/default getters,
bit queries, caller gates and live algorithms remain original guest work.
Machine/C API, native cache and save-state ABI are unchanged.

One warning-free522-source Windows build `040133f5..` and one focused Wine filter
pass publication76/prior74 equality, interruption before manifest publication,
14 attachment and two complete-set guards. Every failed attach preserves the
existing resource owner and complete guest state. The accepted520-source model
copy is the build base; only integration hubs and two new leaves are overlaid,
excluding parallel Debug work. Only affected core-library/test fingerprints are
isolated; old source copies and the canonical executable stay exact.

Twenty-three original/interpreter/native cases compare complete serialized
states and guest clocks with all9716 raw bytes poisoned after Load/setup:
17 load prefixes cover five classes, record boundaries, mirrors, the prior
header and raw successor; six complete D07A4 calls cover source-address/no-data
branches, full flags, bit queries, aliases and raw outside indices. The compiled
path actually consumes31 new bytes and1 prior-header byte.
Two analysis families keep full-getter and single-load stops disjoint, asserting
that neither family compiles its stop; Thumb BL pairs remain fused.
The56 compiled instructions and28 dispatches describe coverage only.
This is selected native behavior, not full caller ABI or ordinary reachability.

The bus proof poisons all9716 source bytes and reads them through a ROM mirror:
2082 halfword-field,4164 scalar-byte,1041 inline,1388 flag and1041 opaque bytes,
all347 records. Mixed16/32-bit reads cross all347 record ends, page boundaries,
prior-header and raw successor edges; other-owner reads leave these counters
unchanged. Unavailable files/failed reattachment, Load and private-slot Load/Undo
retain the same owner. DMA3 and CpuSet4860-halfword copies and CpuFastSet2432-word
copy cover the full source and raw tail; all three compare complete state/clock
against the ROM path. Source poison is restored before state comparison.

All216 actual guards,522 integration source hashes and520 closed model preimages
remain exact. Artifact completion retires only the disposable interrupted dataset
copy; accepted data, logs, executable, compiled objects and source preimages stay
available under `target/rom-data-linked-metadata28-runtime/`. Dataset identity is
`66bc5dbd8fe08af1b949de272215870a7f74fe929eb6e2041b498e8f16becb0f`, manifest `16015edba717322a7839e517b9ba0edfb4ac5e78557f2a8b2a6efc34dace2920` and metadata JSON
`590ba29559b1f5a44212f3ac9e09eab723c14dcc32c5a8e4642097ec64b60966`. Build/runtime/final/preservation/cleanup receipts
record actual scope. The closed source and14517+342 model oracle are not replayed.

Next are actual production CLI create/reuse/startup with positive field use, then
one ordinary equivalence pair. The retained Giza700 window is a candidate whose
new usage counters must be observed first. Full caller ABI/global archive, the
following four-byte family, Linux, USA, native Windows, new32:9 and the full ROM
TODO remain pending. No new checkpoint, ordinary fixture, navigation or game run,
performance measurement, Android change, package, assistant commit or push occurs.


## Actual indexed metadata28 CLI registration and ordinary locators

The production e83-d82 Windows CLI passes generation creation, reuse and normal
startup. All76 generation files match the accepted runtime dataset; all75 logged
resource IDs/hashes match the manifest. All78 managed files retain hashes and
mtimes through reuse and every observed startup. The original
28859 translated blocks load with the single9716-byte source span and
empty dependency list intact. Registration is closed. Positive ordinary usage and one complete ordinary
pair are now verified by the [Steal menu case](#ordinary-metadata28-steal-menu-equivalence)
below.

Four bounded normal locators all report zero in every metadata28 counter:
Giza menu-to-deployment700 frames19409→20109; retained Change Jobs540 frames
4519→5059; Ability-set selection700 frames4519→5219; and Battle Tech R-info800
frames4519→5319. These are negative observations, not positive consumption or
per-frame equivalence proofs. The last two use normal inputs derived on a copied
checkpoint after inspecting the Unit menu (Equipment0/Ability1/Change Jobs2)
and the visible R-info command. The final image actually shows First Aid,
Powerbreak, Mug, Shieldbearer and Mindbreak; that menu still does not read this
physical table. No input/guest-state value is forced to make a counter positive.
Do not repeat these locators. The later dry-ground Steal combat menu supplies positive field-use evidence.
The four unrelated menu/deployment views and basic index0/default attack alone
do not establish that use.

One warning-free production build `0acb352e..` uses a frozen522-file copy of the
accepted runtime. Its only source overlay corrects three metadata28 error labels
that previously named coordinates. Reversing those three string edits restores
the accepted core file byte-for-byte; all other521 source files are exact.
Only affected core-library/production fingerprints are isolated; the canonical
executable hash and mtime are restored exactly. There are six recorded CLI
commands (prepare/reuse/four locators), plus one230-frame menu observation.
Source/model/runtime acceptances are not replayed for the diagnostic text.

All220 actual guards and522 CLI source hashes remain exact; artifact completion
retires zero paths. Evidence under `target/rom-data-linked-metadata28-cli/` keeps
normal input derivation/provenance, source copy, three-label preimage/inverse
receipt, build/command logs, zero-case histories, menu images and registration/
preservation/cleanup receipts. Dataset: `target/rom-data-linked-metadata28-cli/windows-cli/resources/8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632/v1/e83-d82/generation-GuUcax`. No new checkpoint,
manual guest RAM/task/ROM/cartridge edit or performance measurement occurs.
Linux, USA, native Windows, new32:9, the following four-byte family, full caller
ABI/global archive and the full ROM TODO remain pending.


## Ordinary metadata28 Steal menu equivalence

Normal game inputs on copies of retained EUR checkpoints now provide a positive
metadata28 case: opening the Steal command list on dry ground in Giza. The actual
menu shows Steal Armor, Steal Weapon, Steal Gil, Steal JP and Steal Ability.
The accepted suffix spans42 frames, guest22669→22711, and serves five records:
`{"bus_reads": 15, "byte_bytes": 10, "flag_bytes": 0, "halfword_bytes": 10, "inline_bytes": 0, "opaque_bytes": 0, "records_used": 5, "source_bytes": 20}`. These are served-byte counters, not an inferred
distinct-byte union. Halfword and byte fields are positive; inline, flag and
opaque ordinary use remain zero and retain their separate targeted proofs.
No ability effect or steal result is executed or claimed by this menu window.

The existing production CLI `0acb352e..` replays that suffix
with exactly the same eight counters, all75 logged resource IDs/hashes matching
e83-d82 and all78 managed paths retaining hashes and mtimes. It loads the same
28859-block native cache. One Wine full pair on the actual CLI dataset then
compares every complete serialized state and guest clock against the original-ROM
path, with classic/central/wide images, OAM and all five audio streams equal at
every frame. Native/fallback dispatch counts are `[205895, 591653]` and audio sample
counts are `[92168, 92168, 11088, 11088, 92082]`; these describe correctness/coverage, not speed.

An optional five-line `FFTA_STATE_TEST_REQUIRE_METADATA28` block requires positive
reads, source bytes, records and at least one interpreted byte class, reporting
all eight counters. Removing only that block restores the prior validator
byte-for-byte. A test-only normal-input observer loads a copied checkpoint,
advances the original native frame runner with keyboard inputs and observes
resource counters. It retains a rolling16-frame pre-read checkpoint, input suffix,
end state/image and up to32 positive continuation frames within the input bound.
ROM identity is asserted unchanged. There is no manual guest RAM, register,
task, ROM or cartridge edit, original save writer or forced action index.

One warning-free523-source Windows test build `5a6c81be..` supplies both capture
and comparison; core code is unchanged from the accepted522-source CLI. Only
the affected test fingerprint is isolated. Seven normal capture steps execute
1182 frames, plus one neutral CLI frame inspects the full viewport. The initial
water branch has Action disabled and is abandoned without an equivalence claim.
A retained pre-move dry checkpoint has Move selected/Action enabled; ordinary
Down/A opens Action and then Steal. The final capture observes the first new
read and32 continuation frames, retaining the42-frame suffix. No old source,
model, runtime, CLI registration, four negative locators or constructor acceptance
is replayed. No production/core rebuild is needed for discovery or the full pair.

All177 current guards/523 frozen source hashes and220 closed CLI guards/522
production source hashes remain exact, including original seeds, cache, loader,
canonical executable and all78 managed files. Artifact completion retires zero
paths. Evidence under `target/rom-data-linked-metadata28-combat/` includes normal
input plans and capture endpoints, `07-open-steal/before-activation.ffstate`,
`activation.input`, CLI and42-frame pair logs/results, validator preimage/guard,
frozen sources/executable and final/preservation/cleanup receipts.

This closes the declared source/model/managed/runtime/CLI and one positive
ordinary case for metadata28. It does not prove every347 records, inline/flag/
opaque ordinary use, executed ability effects, full caller ABI or global archives.
The following four-byte family's declared source, model, runtime, CLI and one
ordinary case are described below. Linux, USA, native Windows, new32:9 and the
full ROM TODO remain pending. No performance measurement, space campaign, Android
change, package, assistant commit or push occurs.

## European linked four-byte record source contract

**ROMDATA-01-LINKED-RECORDS4 — source contract complete, 2026-10-28.** The declared
EUR physical partition `7133F8..713744` contains844 bytes, retained as211 ordered
four-byte slots, with SHA-256
`85d556e39da8baaec49308c815125532aa8947a476e2c59131532a5bf0334fa9`.
Six exact root literals and their original PC-relative loads establish the
four-byte indexing. The independently rooted next family at713744 uses an
eight-byte step and its own loop. This establishes a physical partition, not a
guest index cap or the semantic number of active records. The neutral proposed
resource ID is `gameplay.linked-records4`.

All four fields are individually loaded as unsigned bytes. Field00 appears in
the B8548 reader and the global/instance getters134CBC/134CD6. Field01 is loaded
at C5524, C5A42 and132FE6; its branch and dispatch comparisons belong to those
callers. Original134E24/134E26 and134E40/134E42 load fields03/02 separately as
arguments to live logic. Getter13510C reads field02. Other consumers use field00,
field02 and field03 as indices into separate encoded code-pointer tables rooted
at3AD638,3AD6AC and3AD72C. Those external tables and their extents are not part of
this source contract. No ability/effect names, enums, value caps, Boolean
conversion or signed reinterpretation are introduced.

The selected original loops obtain the metadata28 getter's encoded inline
address and read its first three positions. Across347 parent rows these give
1041 references to194 distinct four-byte slots, all inside the declared
partition. Three references select index0. Repeated indices alias the same
original row; equal row contents remain separate physical identities. The
observed maximum208 is a property of these references, not an accepted index
limit. The fourth inline byte is not newly classified as a reference.

Original context initialization retains the inline address at live+2C and
`087133F8 + 4 * inline_index` at live+30. The two later selector setters store and
reread the low byte of the inline position before loading the row index. Their
address calculation has no universal three-position or211-record guard. The
eventual model/runtime must retain original encoded addresses and leave reads
outside its owned partition to the appropriate existing owner or raw ROM.

Slots0,1,209 and210 contain identical zero bytes and are preserved individually.
The inspected loops read index0 normally; no global null, terminator or fallback
rule follows from a zero row. The two trailing zero slots209/210 have no proven
reserved/padding meaning and remain undecided. The next family's first-byte
zero check skips an eight-byte entry and advances its loop; it is not a
terminator for either catalog.

All211 slots reconstruct the complete844 source bytes exactly. The partition
does not overlap any of the75 accepted e83-d82 resources or their50489 source
spans. Thirteen authenticated instruction/literal windows, six new and seven
cached, and45 exact pins support the contract. Two additional exploratory
windows are retained separately. All112 input guards preserve the ROM, static
disassembler, accepted76 dataset files, cached witnesses and completed proof
receipts. Artifact completion retires zero paths. An expected ADD encoding in
the private checking helper was corrected before acceptance and a missing
literal window was added; no build or runtime test failed.

Evidence under `target/rom-data-linked-records4/` includes the source contract,
ordered private records, reference list/counts, root/loader inventory,
authenticated windows and pins, source spans and final/preservation/cleanup
receipts. This unit uses static inspection only: no build, Wine invocation,
gameplay, fixture derivation or previous completed proof is repeated. The strict
typed model, managed/runtime binding, actual CLI and one ordinary Wine case are
now complete below for this declared partition. Linux, USA, native
Windows, new32:9, full caller ABI, indirect catalogs and the full ROM TODO remain
pending; no performance or storage campaign is performed.

## Typed four-byte linked records and original reader proof

**ROMDATA-02-LINKED-RECORDS4 — complete for EUR/Wine, 2026-10-28.** The standalone
core `rom_data::records4` model preserves all211 ordered physical slots and all
844 bytes, including distinct zero slots0,1,209 and210. `Record` contains its
physical `u8` index and four separate unsigned byte fields. Strict JSON parsing
rejects unknown/missing fields and invalid numeric types or widths. Validation
checks the declared source, slot count, order and indices without inventing
enums, a null row, value caps or meaning for the trailing zeros. One source span
has role `linked-records4-slots` and reconstructs the complete partition.

`source_address(index: u8)` returns the original encoded guest address, including
indices211..255 outside this model's ownership. `record` returns `None` for an
unowned slot and `byte_at` does so outside7133F8..713744. These are ownership
diagnostics, not guest zero/fallback behavior. The original dynamic inline
selector is not constrained to three positions. Shape validation remains
separate from `verify_source`: base authentication requires the exact full EUR
ROM, source fingerprint and all13 authenticated reader/boundary windows, then
compares decoded content. Modified but well-shaped values are not accepted as
the original base dataset.

One warning-free core-only Windows build produces executable SHA-256
`a727058074a5c1e546a2528538cf9ff81507e147c5d62c4c8a35bdf740d43bb7`.
Its frozen525-file source snapshot derives from the accepted523-file metadata28
ordinary snapshot with only the new model/test leaves and module declaration
overlaid. It has233 core/build inputs. Only the exact affected core-test
fingerprint is isolated; the current parallel debug sources and all accepted
snapshots remain untouched.

One new Wine filter passes20 model/parser/authentication guards and1567 calls
or prefixes through original guest instructions:

- 1055 field probes run five readers for every211 slots: global and instance
  field00 getters, field01 prefix, global field02 getter and the field03/02
  argument prefix. Actual LDRB effective addresses cover all844 owned source
  bytes, including unreferenced and zero slots. The argument prefixes stop
  before calls into live game logic.
- 512 full setter calls run both original global and instance setters for all
  256 byte indices. High bits in the input selector are discarded by the
  original byte store/read. Checks include the selected inline byte, encoded
  row address and all five permitted EWRAM write bytes, including addresses for
  unowned211..255. No three-position or211-record cap is introduced.
- Two isolated high-bit variants run10 additional field probes, exercising
  unsigned values128..255 in every field. They pass shape validation, fail
  original-base authentication and remain separate synthetic oracle inputs.

Every field probe preserves EWRAM; setter probes preserve the full buffer apart
from the expected selector byte and pointer word. All probes preserve callee
registers, SP/LR, IWRAM, VRAM, OAM and palette; the final ROM equals its original
copy after the isolated variants are restored. The9808 executed instructions
are coverage evidence only. This oracle does not compare native clock behavior,
full caller ABI, ordinary gameplay or managed runtime consumption.

All125 actual input guards,525 frozen source hashes, the three current
implementation hashes and112 closed source-unit guards remain exact. Artifact
completion retires zero paths. Private evidence under
`target/rom-data-linked-records4-model/` retains independent model/source/probe
inputs, isolated variants, source preimage/snapshot, current executable,
build/Wine logs and final/preservation/cleanup receipts. Source classification
and all older model/runtime/ordinary proofs are not replayed. Schema1/e83-d82,
save/native/C API layouts and the existing managed dataset are unchanged.

Managed publication, typed bus ownership, targeted native/Load/Undo/DMA checks,
actual CLI and one positive ordinary case are now complete below. Linux, USA, native Windows, new32:9, full caller ABI,
external dispatch-table extents, indirect catalogs and the full ROM TODO remain
pending. No gameplay, checkpoint derivation, performance measurement, storage
campaign, Android change, package, assistant commit or push occurs in this unit.


## Managed four-byte linked records and typed runtime reads

**ROMDATA-03-04-LINKED-RECORDS4-RUNTIME — complete for EUR/Wine, 2026-10-29.**
EUR e84-d83 adds `gameplay.linked-records4` at resource index75, serialized as
`linked-records4.json` / `linked-records4-v1-json`. The generation contains77
files and76 resources. Every preceding75 resource record and payload remains
exactly equal to the accepted e83-d82 dataset. The new844-byte source span
7133F8..713744 is disjoint, with no invented dependencies. Dataset identity is
`e285ffd1c2b595b07515d6d300e668d6b8fb16027b97c305348af40efce75917`, manifest SHA-256
`5a9456200ae5b219e9dc7f3805a403e6dbc2c42c18b6dfabdf4ef6952637fd4b` and new JSON SHA-256
`e9c1dc49c709f8821d7702b067a68590c11c904b27c812113d122a1b17362c20`.

The runtime owns an immutable typed `Records4` value. Guest reads obtain its four
unsigned fields directly; encoded indices, source pointers, selectors and live
dispatch remain original guest code. Seven diagnostic counters report reads,
served source bytes, four field-byte totals and distinct physical slots. A
four-word mask retains all211 slot identities, including the zero rows. These
are use/correctness counters, not performance measurements. Neither a semantic
record count nor a new index, null or selector limit is introduced.

Local and global source-page gates include page713. Each byte of a wide access
composes the existing metadata28 owner on the left, the new typed owner inside
the partition and raw ROM on the right. Unowned indices211..255 retain original
address calculation and read behavior. No raw shadow image is reconstructed for
the new family. File parsing/binding, base-source authentication and publication
are integrated with the existing transactional loader; save, native and C API
layouts are unchanged. The USA extraction branch remains8/7 without new claims.

Three Windows test builds were required. The first failed on a new test's
`&&Call` predicate and a stale cached menu library. Exact cache evidence showed
the old menu artifact marked fresh despite the frozen current source containing
the required APIs. The predicate was corrected and only the affected menu/host
fingerprints invalidated. The second build was warning-free and never run.
Before Wine, static inspection confirmed that guest LDR word aligns its bus
address before rotating; two test expectations were corrected accordingly.
The third build is warning-free, with accepted executable SHA-256
`33144a8c39e141e46d78ecce60a8128353c7a96aaffb7f3fa3cfe4088beec716`. Its551-file frozen snapshot includes the
current committed workspace and the seven integration paths. Both earlier551
snapshots and the unused second executable are retained as diagnostics. The
525 accepted model source preimages remain exact; no production debug/menu code
is changed and no debug test queue is executed.

One new Wine filter passes publication/interrupted-write rejection,14 attach
guards, two complete-set guards and retention of the same owner and complete
machine state after failed attachment. Recomputed file hashes cannot admit
modified fields, wrong physical indices/count/source offsets, unknown JSON
fields or counterfeit source bindings, dependencies, formats and coverage.

The same filter compares21 original-ROM interpreter, file-derived interpreter
and compiled native cases using complete serialized states and guest clocks.
All844 raw source bytes are altered **after** Load/setup in the file-derived
arms. Cases comprise13 loads, four complete original getters, one two-field
argument prefix and three complete original pointer setters, across four
analysis families that each exclude their stop boundaries. The41 compiled
instructions read21 new typed bytes and six prior metadata28 bytes. Widths,
mirrors, all fields, outside-owner reads, low-byte selector aliases and original
encoded pointers are covered. Unaligned guest word loads use aligned physical
bus reads; separate direct C API checks cover byte composition across owner
edges. The21 native dispatches are coverage evidence only.

Full-bus checks recover every844 byte from the typed owner despite altered raw
data, count211 bytes for each field and211 distinct slots, and verify every211
record-end read32 crossing plus width/mirror and prior/raw boundaries. Reads of
unrelated owners leave the new counters unchanged. Unavailable files and failed
attachment preserve the owner; Load and private slot Load/Undo retain the same
Arc and still serve typed bytes. DMA3 and CpuSet transfer424 halfwords and
CpuFastSet216 words, including raw tail bytes, with complete state/clock equality
against original ROM in all three cases.

All237 actual guards,551 accepted source hashes, both551 diagnostic hash
sets,525 model preimages and seven current implementation hashes remain exact.
Artifact completion retires only the disposable interrupted-write copy. Evidence
under `target/rom-data-linked-records4-runtime/` retains accepted
`windows-managed-europe/offline dataset`, native objects, source preimages and
snapshots, executables, build/Wine logs and final/preservation/cleanup receipts.

This closes managed/runtime integration and targeted lifecycle/native-clock
equivalence for the declared family. Actual production CLI create/reuse/startup
and one positive ordinary case are now verified below. No source,
model or previous ordinary acceptance is replayed. Linux, USA, native Windows,
new32:9, full caller ABI, external dispatch catalogs and the full ROM TODO remain
pending. No gameplay/checkpoint derivation, performance measurement, storage
campaign, Android change, package, assistant commit or push occurs in this unit.


## Actual linked four-byte record CLI registration

**ROMDATA-04-LINKED-RECORDS4-CLI — registration complete, 2026-10-29.** The later
positive ordinary case below closes the remaining consumption criterion. One warning-free Windows production build from551 source
files, byte-identical to the accepted runtime snapshot, produces executable
SHA-256 `bcf632fa30c6cb5c31c3d2f34e49dbb4ad3064cec87fcdc2a8dff6c3a1c3b64a`.
There is no source overlay or diagnostic-label correction. Only the exact
core-library/production fingerprints are isolated. The canonical executable's
hash and modification time are restored exactly after the build. Unfinished
parallel audio/debug sources remain separate from the accepted snapshot.

Three actual CLI Wine children exit0: dataset creation, reuse and a new normal
120-frame command selection. All77 generation files match the accepted e84-d83
runtime dataset exactly; all76 resource IDs/file hashes appear during startup.
The same79 managed paths retain their hashes and modification times across reuse
and the normal run. The old native cache loads28859 blocks. This verifies actual
CLI registration, publication and startup, not consumption of every resource.

The normal locator starts from a copy of the retained dry-ground Steal menu at
guest frame22711, with Steal Armor visibly selected. A single ordinary A press
at relative8, held12 frames, reaches the target screen by22831. All seven new
records4 counters are zero. That observation is retained as a closed negative;
no equivalence pair is run for it and no positive ordinary consumption is
claimed. It does not reopen the earlier Steal42 menu-opening acceptance or the
four closed metadata28 negative locators. No new endpoint checkpoint is saved
by this CLI command, and target pixels alone are not a new machine checkpoint.

All187 actual guards,551 frozen CLI source hashes and551 accepted runtime source
hashes remain exact, including original/copy checkpoints, ROM, native cache,
TinyCC loader, canonical executable and77 accepted dataset inputs. Artifact
completion retires zero paths. Evidence under
`target/rom-data-linked-records4-cli/` retains the frozen sources/executable,
canonical preimage, three CLI logs/process journal, no-write receipts, normal
input/seed copy and target PPM/PNG, manifests and final/preservation/cleanup
receipts. The generation is
`windows-cli/resources/8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632/v1/e84-d83/generation-ginqZq`.

The later following-turn case below now verifies positive records4 reads and
complete per-frame state/guest-clock/pixel/audio equivalence, reusing this
production executable and dataset. Do not repeat
this identical negative120-frame locator or source/model/native proofs merely
to reconstruct context. Linux, USA, native Windows, new32:9, full caller ABI,
external catalogs and the full ROM TODO remain pending. No performance or
storage campaign, manual guest edit, new checkpoint derivation, Android change,
package, assistant commit or push occurs in this unit.


## Ordinary linked four-byte turn equivalence

**ROMDATA-04-05-LINKED-RECORDS4-COMBAT — one EUR/Wine ordinary case complete,
2026-10-29.** Normal game controls on authorized checkpoint copies produce a
positive following-turn case after Fight and final-facing confirmation. The
retained40-frame suffix runs guest24755..24795 with neutral keys, before and
after the first new source read; the endpoint visibly shows Lalle's turn.
Capture, actual production CLI and the original-ROM/file-derived comparison
agree in all seven counters:223 bus reads/223 served bytes, field00=64,
field01=132, field02=11, field03=16 and two physical slots. Served bytes are not a
distinct-byte union. All four fields are observed; this does not prove every211
slot or assign semantic names to the numeric fields.

The actual CLI reuses executable `bcf632fa..` and the existing e84-d83 generation
without a new production build or preparation. All76 logged IDs/file hashes
match the manifest, all79 managed paths remain unchanged and the old28859-block
native cache is loaded. One complete Wine equivalence pair passes every40
serialized machine states and guest clocks, classic and wide pixels, OAM,
dispatch and all five audio buffers. All seven counters match the CLI and
capture exactly. Native/fallback dispatch counts are[325898, 579939], and audio
sample counts are[87780, 87780, 10560, 10560, 87686]; these are correctness/coverage evidence only.
Both ordinary arms use the same existing native cache. Interpreter/native
clock equivalence remains the separately closed targeted21-case runtime proof,
not a new whole-game interpreter/native claim.

An optional five-line `FFTA_STATE_TEST_REQUIRE_RECORDS4` guard rejects a scene
without new source/field/slot use and reports all seven counts. Removing only
that block restores the previous validator byte-for-byte. The new ignored
`records4_scene_tests::normal_gameplay_observes_records4` observer loads copied
states and normal input scripts, uses the existing native frame runner, and
retains a rolling pre-activation state followed by32 positive continuation
frames. It changes only normal input and host audio observation buffers; it
does not manually edit guest RAM, registers, task flags, ROM or cartridge saves.
The original ROM hash remains exact after every capture.

One warning-free Windows host-test build from552 frozen files produces
`a5b2fc8428c13280aa4d7910d6b89fb4f23a3b6b391de9ca068114caf0636522`. It derives from the accepted551 CLI sources with
only the observer leaf, test-module declaration and validator guard overlaid.
Core code is unchanged. The exact host-test and menu dependency fingerprints
are isolated so concurrent audio/menu work cannot supply a mismatched cached
dependency; no parallel source is changed or its test queue executed. The same
executable handles all discovery and the full pair, with no subsequent rebuild.

Ten captures execute2084 frames through normal cancellation, a verified dry
approach from6,5 to3,5, Fight selection, adjacent target Dandree at3,6, HIT
prediction, final Do it and facing confirmation. Earlier zero-use phases remain
closed observations and are not equivalence claims. The attack-success outcome
is not inferred from the prediction screen. One additional classic CLI frame
was only a UI observation. Subsequent menu checks use read-only extraction of
the already serialized classic framebuffer: FFSTATE checksum and RGB555 vector
shape are checked, and the accompanying complete wide pixels must equal the
saved native PPM byte-for-byte. This supplies current menu labels without
additional game execution, Wine launches, checkpoint edits or helper rebuilds.
The last capture first observes reads at relative152 and continues32 frames;
rolling relative144 supplies the40-frame accepted suffix. Discovery then stops.

All210 unique actual input guards/552 frozen hashes/three current implementation
hashes and187 closed CLI guards/551 CLI hashes remain exact. Original seeds,
input copies, cache, loader, canonical executable and managed data are preserved.
Artifact completion retires zero paths. Evidence under
`target/rom-data-linked-records4-combat/` retains normal plans/endpoints,
`10-finish-facing-next-turn/before-activation.ffstate`, `activation.input`,
observations/framebuffer projections, CLI/full-pair logs and results, source
preimages/snapshot, current executable and final/preservation/cleanup receipts.

This closes the declared source/model/managed/runtime/CLI and one ordinary
case for linked four-byte records. Source, model, targeted native/lifecycle,
registration and earlier negative acceptances are not replayed. All211 ordinary
slots, full caller ABI, attack/effect success semantics, indirect catalogs,
Linux, USA, native Windows, new32:9 and the full ROM TODO remain pending.
The following independently rooted eight-byte family has its source contract
documented below; its migration remains pending. No performance measurement, storage campaign,
Android change, package, assistant commit or push occurs in this unit.

## European conditional eight-byte record source contract

**ROMDATA-01-CONDITIONAL-RECORDS8 — source contract complete, 2026-10-29.**
The original scan at `D56A0..D57B4` starts at `713744`, advances by eight bytes
and visits at most127 positions, with loop ordinals1..127. Its1016-byte source
partition ends exactly at `713B3C`, with SHA-256
`d52e829c2ae2e3f8a519fc1fe28797a1005fcdc0e2afc81201e872024608b18b`.
Two independently inspected readers of the next root use a different70-byte
stride. This closes the eight-byte extent without declaring that successor's
extent or interpreting the data as missions, jobs or rumors. The neutral
proposed resource is `gameplay.conditional-records8`.

Byte00 is a nonzero gate. Zero skips that position and continues scanning;
it never terminates the table. Byte01 is either zero for a wildcard or an
unsigned equality filter against the full incoming32-bit context. The first
encoded key occupies bytes02..03, assembled little-endian from separate unsigned
loads, and its unsigned operand is byte04. The second key occupies bytes05..06
and its unsigned operand is byte07. All eight bytes are preserved. These keys
select live predicates, rather than ROM payload pointers.

Keys0000..05FF select a normalized live flag at
`02001F70 + (key >> 3)`, bit `key & 7`. Higher keys select an unsigned live
counter at `02002030 + low16(key - 0600)`. The first counter predicate requires
counter greater than or equal to its operand; the second requires counter less
than its operand. Their flag predicates also have different polarity: the first
rejects flag1/operand0 and flag0/operand1, while the second rejects flag1/operand1
and flag0/operand0. Other operand values pass those Boolean branches without
coercion. Only the second key0 bypasses the predicate. A first key0 still reads
flag0, and the original first flag path may call its reader twice. The scan
does not invoke the neighboring live-state setters.

The source contains76 nonzero-gate rows and51 entirely zero rows. Each zero
position retains its physical identity. All byte01 values happen to be zero,
and each nonzero byte00 happens to equal its physical index plus one; neither
observation becomes a validation restriction or trusted row index. The enabled
rows contain105 live references,104 to flags and one to a counter, reaching63
distinct live keys with24 repeated-key groups. Repeated keys alias live
predicates while the source rows remain distinct. The counter reference uses
operand2, which must remain numeric. Forty-seven enabled rows bypass their
second predicate using key0. Mutable flag/counter memory is not a base asset.

Accepted rows produce their original encoded ROM pointers, in physical order;
the return value is the accepted count. The original routine passes its output
pointer and `capacity << 2` to an executable helper before scanning. It then
decrements the full32-bit capacity only after writing each accepted pointer.
There is no entry guard making capacity0 an empty-list request. Both inspected
direct callers pass64 and narrow their context to a byte; that does not impose
a new restriction on the routine itself. The helper vector at370EB0 and the
145C10 `BX r2` trampoline retain their earlier executable-support classification.
They are not a zero fill-value resource or part of this partition, and their
mutable IWRAM body/full ABI is not reauthenticated here.

One aligned32-bit pointer inventory over encoded addresses71373C..713B44 finds
the exact source root only atD570C, the successor root at38 locations, and no
interior or origin-minus-eight pointer in that representation. Two candidate
direct Thumb calls are verified in their caller windows. This does not establish
all indirect/unaligned references or full caller ABI. All127 slots reconstruct
the1016 bytes exactly, with no overlap against76 accepted e84-d83 resources and
their50490 source spans. Eight authenticated windows, six new and two reused,
and93 exact instruction/literal pins support the contract. All96 input guards
preserve ROM, static disassembler, accepted77 dataset files, cached witnesses
and closed receipts. Artifact completion retires zero paths.

Private evidence under `target/rom-data-conditional-records8/` retains ordered
records, live references/aliases, root/caller inventory, source contract/spans,
authenticated windows/pins and final/preservation/cleanup receipts. This is
static source inspection only: no build, game/Wine test, checkpoint, navigation,
previous completed proof or helper classification is repeated. Schema1/e84-d83
and the76-resource dataset remain unchanged. The strict typed model and original
selection-loop Wine proof are now complete below. Managed/runtime publication,
actual CLI and ordinary use of this family remain pending, together with Linux,
USA, native Windows, new32:9 and the full ROM TODO. No performance or storage
campaign occurs.

## Typed conditional records and original selection-loop proof

**ROMDATA-02-CONDITIONAL-RECORDS8 — complete for EUR/Wine, 2026-10-29.** The
standalone `rom_data::conditional_records8` model preserves127 ordered physical
slots and all1016 source bytes. Each record retains its physical index, unsigned
gate/filter/operands and two little-endian16-bit encoded keys. Strict JSON
parsing rejects unknown or missing fields, wrong numeric types and widths.
Shape validation requires the declared extent, count, physical order and indices;
it does not impose the observed header=index+1, zero filters, Boolean operands
or semantic names. Equal zero slots remain distinct. `byte_at` and `record`
return `None` outside physical ownership, without introducing a guest fallback.
One source span reconstructs the entire source partition.

`LivePredicates` supplies mutable flags and counters separately from the base
asset. `eligible` retains literal first/second polarity, numeric thresholds,
the first flag reader's repeated query and the second-key-zero bypass.
`select` returns original encoded row pointers in physical order and decrements
the full32-bit capacity after acceptance, including its wrap from zero. It
models selection only; it does not initialize a guest output buffer or execute
the external clear helper. The filter compares against the full32-bit context.
Shape validation is separate from exact base authentication: `verify_source`
requires the exact EUR ROM, source fingerprint and all eight authenticated
reader/caller/boundary windows, then compares decoded content. Well-shaped
modified values are not authenticated as base data.

One warning-free Windows core-only build from554 frozen files/239 selected
core inputs produces executable SHA-256
`c6f5406d1f0e91595791aa90d606bf018edd555af4e8d07d2281679962485b44`.
The snapshot derives from the accepted552-file preceding ordinary proof with
only this module, its test leaf and one core-hub module declaration added.
Removing that declaration restores the hub preimage byte-for-byte. Only the
affected core-test fingerprint is isolated; parallel frontend/audio/debug source
is preserved and not included in this unit's validation.

One focused Wine filter passes24 parser/shape/authentication guards and109
original-loop cases. Sixteen base cases vary live flag patterns, counter values
and capacities0/1/64/FFFFFFFF. Ninety-three isolated synthetic cases cover flag
keys0/05FF, second-key-zero bypass, counter keys0600/0682/FFFF, numeric boundary
operands through255, unsigned header255, filters128/255 and full-context mismatch,
and a zero header that suppresses all live calls. An independent Python oracle
provides expected pointers and live-call sequences. The actual original loop
`D56CE..D57A4` and its real `CCE98`/`CCF00` callees agree with both that oracle and
the typed selection. Pointer order, accepted count, remaining capacity, visited
physical positions and exact live-call traces match. The original entry clear
helper and return epilogue are excluded explicitly; full caller ABI is not proven.

The original base cases actually load612 distinct source bytes. Synthetic cases
load134 distinct positions and cover all eight bytes of their final physical
slot. These observations are separate from lossless reconstruction of all1016
base bytes, rather than a claim that the scan reads every field of disabled rows.
All EWRAM matches expected pointer writes exactly, including untouched output;
IWRAM is exact except the real flag reader's four-byte return-link scratch.
Stack pointer, r11/r12, VRAM, OAM, palette and the final restored ROM pass.
The198372 executed instructions are coverage only; no time, RSS, profiling or
other performance/storage measurement is interpreted.

All120 unit input guards,554 frozen hashes, three current implementation hashes
and96 closed source guards remain exact. Original seeds, managed77-file dataset,
canonical executable and closed proof receipts remain unchanged. Artifact
completion retires zero paths. Evidence under
`target/rom-data-conditional-records8-model/` retains the independent model,
cases/source spans, preimage/snapshot/core hashes, executable, build/Wine logs
and final/preservation/cleanup receipts. No gameplay, ordinary checkpoint,
navigation, previous completed proof or IWRAM helper reconstruction is repeated.

Schema1/e84-d83,76 managed resources, Machine/save/native-cache/C API remain
unchanged in that model unit. Managed publication, immutable typed bus service
and affected native/state-clock/lifecycle/DMA now pass below. Actual CLI and
ordinary consumption remain pending. Linux, USA, native Windows, new32:9,
full caller ABI, indirect references, successor70-byte extent and the full ROM
TODO remain pending. No Android/package/assistant commit/push occurs.

## Managed conditional records and typed runtime reads

**ROMDATA-03-04-CONDITIONAL-RECORDS8-RUNTIME — complete for EUR/Wine,
2026-10-29.** EUR schema1/e85-d84 appends resource index76,
`gameplay.conditional-records8`, file `conditional-records8.json`, format
`conditional-records8-v1-json`. The dataset has78 files/77 resources; every
preceding76 resource record and payload remains byte-for-byte exact. One
1016-byte span covers713744..713B3C with role `conditional-records8-slots` and no
invented dependencies. USA remains e8-d7 and unverified.

Extraction, strict parsing/source binding, bounded input accounting, publication,
loading and runtime logs share the same model. An immutable typed owner serves
all eight bytes of every physical slot. Six logical classes distinguish the
gate, filter, first key, first operand, second key and second operand; the keys
retain both little-endian bytes. A two-word mask tracks127 physical slots.
Nine counters report bus requests, served source bytes, the six field classes
and distinct slots used. Page713 is registered in both routing layers. Each
byte composes with preceding records4 ownership or the raw successor, including
unaligned C API reads across the two boundaries. Guest selection, live flag/
counter reads and original encoded pointer identity remain unchanged.

One initial Wine filter verifies publication, a deliberately interrupted new
file write,14 failed-attach cases and two missing/full-set guards. Every failure
retains the exact machine state and prior owner. Its first16 native cases pass
complete serialized-state/clock comparison against raw interpretation:14 loads
cover the eight original byte readers, wider reads, mirrors and prior/raw edges;
two prefixes assemble the encoded keys. All1016 raw source bytes are altered
after Load/setup in both typed interpreter and native candidates, then restored
before full-state comparison. The original code and source authentication are
preserved, and native case groups exclude their own stop boundaries.

The final original selection-loop case initially stops on an observer discrepancy:
direct read8/16/32 callbacks account607 bytes while609 are expected. A remaining
case-only diagnosis proves raw/typed/native complete state and clock agree, and
actual typed/native owners both serve609 bytes. Two grouped native dispatches
atD5732 andD5772 each contain one additional source read through the real
`interp_one` fallback. The test observer now delegates to that original callback
and accounts its source delta separately. A final case-only Wine proof passes
the original strict609 expectation, exact typed/native actual counts and full
state/clock, with607 direct bytes plus two fallback bytes and no unattributed
dispatch. Runtime, translation, guest logic and C API implementation are not
changed to repair the observer.

The combined acceptance covers17 distinct native cases. Their native paths serve
631 new bytes in total,629 through direct callbacks and two through the existing
fallback, plus five prior records4 bytes. These are served-byte counts, not a
distinct-byte union or a claim that every instruction is compiled. Five retained
native objects contain118 emitted entry points across the separate case groups;
the full selection case uses100 entries/392 dispatches as coverage only.
Publication, attach guards and the first16 passing cases are not repeated during
the remaining-case work.

The remaining diagnosis also verifies all1016 typed bus bytes, all six field
classes and127 slots, every127 record end, wider/mirrored reads and prior/raw
composition. Temporarily unavailable files, failed attachment, state Load,
slot Load/Undo and owner `Arc` identity pass. DMA3, CpuSet510 halfwords and
CpuFastSet256 words cross the successor boundary and each preserve the complete
raw-reference state and clock. These checks are not repeated in the final
observer-only run.

Four Windows builds are warning-free from568 frozen current, mutually consistent
core/menu/frontend inputs. The first is never executed after static review finds
an inherited4096 dispatch guard too small for the full scan. The second supplies
the initial observer failure and accepted earlier scopes, the third supplies the
remaining diagnosis/lifecycle proof, and the fourth supplies the complete
callback observer. Only the test harness changes after the first build. Exact
core-library, host-test and initially menu fingerprints are isolated; no global
clean or parallel-source edits occur. The accepted executable SHA-256 is
`f60055e6d77ae4039bc0fdfa872a3182c47f14d40991c10307ae604576eb1371`.

All261 actual input guards,568 accepted hashes, three diagnostic568-file snapshots,
554 model preimages, seven current implementation hashes and77 old/78 accepted
dataset files remain exact. Artifact completion retires only the disposable
interrupted copy. The accepted dataset is
`target/rom-data-conditional-records8-runtime/windows-managed-europe/offline dataset`,
identity `3f13d6e5432b0dbff85284d15013b5c6179d877f3f9c4532f953acb81e5db004`.
Its manifest SHA-256 is
`2b4221213ebed5a65d55642e66743cf9ce0b02cb27563c30906b2e82167abbb6`;
the new JSON SHA-256 is
`ed324120046a3e854da483d311c23822cdb68aa860449ca986887fa99836c254`.
Private evidence retains all build/run receipts, object/preimage copies,
diagnostic and final source traces, final/remaining results, preservation and
cleanup receipts. Acceptance is combined from these explicit scopes; the
initial failed full-filter receipt is retained as a failure.

No ordinary game/checkpoint/navigation, closed source/model proof, performance
or storage campaign is repeated. Machine/save/native-cache/C API ABI remain
unchanged. Actual production CLI registration now passes below; ordinary positive
consumption remains pending. Linux, USA, native Windows, new32:9, full caller ABI,
indirect references, successor70-byte extent and the full ROM TODO remain pending.

## Actual conditional-record CLI registration

**ROMDATA-04-CONDITIONAL-RECORDS8-CLI — registration/startup complete for
EUR/Wine, 2026-10-29.** One warning-free Windows production build uses568 files
byte-identical to the accepted runtime snapshot, with no source overlays.
Its executable SHA-256 is
`72d44985a5e6531605990cb7769378195c5eb65cf1894de099b8764ad4d22c9f`.
Only the affected core-library/production fingerprints are isolated; the menu
source matches the selected runtime. The canonical executable's original hash
and modification time are restored in the build's `finally` path.

Three real CLI children under Wine exit0: `rom-data prepare` creates e85-d84,
a second invocation reuses it, and `runc --rom-data` loads that generation for
one neutral frame. All78 generation files, including the manifest, equal the
accepted runtime dataset. All77 logged resource IDs/hashes match its manifest.
All80 managed paths retain hashes and modification times during reuse/startup,
and the existing28859-block native library is loaded unchanged. These counts
establish registration and reuse rather than an extracted-code percentage.

After the successful first preparation, the private orchestration helper stops
on a stale index75 assertion against the new resource at76. The actual CLI
command has succeeded and the generation is valid. The helper is corrected,
its initial preimage is retained, and only the two unexecuted children resume
against that same generation and binary. Preparation and compilation are not
repeated. No runtime/resource implementation changes follow from this assertion.

Startup uses an exact local copy of the retained normal before-activation state
at guest24755, with one neutral input frame to24756. The seed SHA-256 is
`79be0fcf4f674f69975c915f17410c0610df8bb4b1e7f899cbc4251f373e27ff`.
All nine new-family counters are zero in this single frame. No action,
navigation, new ordinary checkpoint or saved endpoint is derived, and neither
positive ordinary consumption nor full-pair equivalence is claimed here.

All189 actual guards,568 CLI hashes and568 accepted-runtime hashes remain exact.
Original seeds/input copies, ROM, native cache, TinyCC loader, canonical
executable and accepted78 files are preserved. Artifact completion retires zero
paths. Private evidence under `target/rom-data-conditional-records8-cli/` includes
build/CLI process and no-write results, initial/corrected helpers, final summary,
source snapshot, preimage/current executable, seed copy and preservation/cleanup
receipts. The generation is `windows-cli/resources/<EUR SHA>/v1/e85-d84/generation-hw22uM`.

The source, model, native/lifecycle and earlier-family proofs remain closed.
One positive ordinary use and matching complete state/clock/image/OAM/audio pair
now pass below. Linux, USA, native Windows, new32:9,
full caller ABI, indirect references, successor70-byte extent and the full ROM
TODO remain pending. No performance/storage campaign, Android/package, assistant
commit or push occurs in this unit.

## Ordinary conditional-record Rumors-entry equivalence

**ROMDATA-04-05-CONDITIONAL-RECORDS8-PUB — complete for one ordinary EUR/Wine
case, 2026-10-29.** Normal Pub input reads the new conditional-record family.
The accepted42-frame suffix runs guest15181..15223 from a checkpoint derived
only through normal game controls. It serves468 new bytes in468 requests:
gate127, filter76, first key152, first operand76, second key34 and second
operand3. All six field classes are positive. The127 physical slots touched
include the51 zero-gate headers; this is not127 active records, all fields of
every slot, a distinct-byte union, or all predicate branches/caller contexts.
Capture, actual CLI and the complete-pair test report identical nine counters.

The seed is the already-retained normal Missions-list endpoint from the eligible
quest locator, at guest14969 with SHA-256
`b3195906e512e163cce1a88c051d5bce39e7865c4a11daf2da3be31dc3acd5a0`.
Its old620-frame route and older397-frame Pub acceptance are not replayed. The
authenticated classic image shows Desert Peril selected in the mission list.
One normal B press returns to Pub options after180 frames; all nine new counters
remain zero. Its image shows Missions selected with Rumors immediately above.
The next capture presses Up for three frames and A for12, first reads the new
source at relative42, and stops after32 frames of continuation. It executes74
frames, with the rolling relative32 checkpoint supplying the42-frame accepted
suffix. Discovery stops once all six field classes are observed.

The final image contains the NPC prompt “Heard the latest rumors?”; no displayed
rumor list, selected rumor or completed mission outcome is inferred. Both normal
captures execute254 frames on local copies. No guest RAM/register/task/ROM edit,
cartridge writer or forced guest identifier is used. The zero-use B phase is a
closed observation and is not repeated.

Image inspection uses read-only extraction from the saved FFSTATE. The full
payload checksum, source ROM hash, unique38400-pixel classic vector, following
1228800-pixel wide vector and valid RGB555 values are checked. The older Pub PPM
is240x160 classic with conversion `component * 255 / 31`, unlike the recent
wide/bit-replication profile initially assumed by the helper. Its complete pixels
match the authenticated classic vector exactly. The helper now verifies the
actual matching framebuffer/conversion and records that profile explicitly.
The old FFSTATE/observation retain their earlier guards; the old PPM/PNG are
treated as auxiliary outputs, not falsely claimed to appear in that guard map.
All three classic projections use saved data without another Wine UI run,
guest execution or mutation, and the earlier helper preimage is retained.

One warning-free Windows host-test build from569 frozen files produces
`63584945ab7a1ff6d0cdbf9ddc98b6b274ae0b86ec6640caf8915ee0250e54ba`.
It derives from the accepted568 CLI sources with only the passive observer leaf,
its test-module declaration and optional five-line positive-use guard overlaid.
Removing the declarations/guard restores both preimages byte-for-byte. Core and
runtime code are unchanged, and only the affected host-test fingerprint is
isolated. The same executable performs both captures and the full pair.

The existing72d44985 production CLI and e85-d84 generation-hw22uM replay this
new42-frame suffix without preparation, rebuilding or navigation. All77 logged
ID/hashes match, all80 managed paths retain hashes/mtimes, the existing28859-block
native cache stays exact, and all nine counters match the capture. One Wine
full pair using the same63584945 test executable then compares every42 complete
serialized states/guest clocks, classic/wide pixels, OAM and five audio streams.
All pass with those exact counters. Both ordinary arms use the same old native
cache; this is data-service equivalence rather than whole interpreter/native
parity. Dispatch counts202704/276411 and audio sample counts
`[92168, 92168, 11088, 11088, 92106]` are coverage only.

All143 actual guards,569 frozen source hashes,568 CLI hashes, three current
implementation hashes and80 managed files remain exact. Original normal seeds,
ROM, canonical executable, TinyCC loader and native cache are preserved.
Artifact completion retires zero paths. Private evidence under
`target/rom-data-conditional-records8-pub/` retains normal plans/endpoints,
`02-select-rumors/before-activation.ffstate`, `activation.input`, observations,
framebuffer projections, source preimages/snapshot, executable, CLI/pair results
and final/preservation/cleanup receipts.

This closes the declared source/model/managed/runtime/actual CLI chain and one
ordinary case for conditional-records8. No closed proof is replayed, and no
performance/storage campaign, Android change, package, assistant commit or push
occurs. Full caller ABI, all indirect references, successor70-byte extent, Linux,
USA, native Windows, new32:9 and the full ROM TODO remain pending. The next
independent source lead is713B3C with a70-byte reader stride; it requires its own
extent/field/overlap contract before a model or runtime claim.

## European mission-record source contract

**ROMDATA-01-MISSION-RECORDS70 — complete for the declared EUR source contract,
29 October 2026.** The catalog occupies `0x713B3C..0x71C73C`: 512 ordered records
of 70 bytes, totaling 35,840 bytes. Its SHA-256 is
`bbdbc955b9b3f7ed678123aa6bea8b6098f25f6a13a4ab94de669d1e103253dc`.
The mission association reuses the already-closed normal mission-selection and
event-join evidence; other field names remain structural.

Two independent scans start at `0x713B82`, index 1, advance by 70 and include
index 511. The getter directly addresses index 0. The resulting end is also
an independently rooted byte array at `0x71C73C`; the separate 30-byte array at
`0x71C740` has stride-one readers. These witnesses establish the physical
extent. The separate accessor returning 300 does not establish a catalog cap.

The original getter at `0x0D1F30` narrows its index to the low 16 bits and its
selector to the low 8 bits. Selectors 0 through 64 dispatch through 65 entries;
selectors 65 through 255 return zero without reading record data. There is no
guest index check against 512. Its unsigned scalar loads compose little-endian
values from individual bytes, including unaligned fields. All offsets below
are hexadecimal; selector numbers are decimal:

- Selector 0 reads LE16 at `+00`. Selectors 8/10/12 read LE16 at `+05/+08/+0B`,
  with selectors 9/11/13 reading the following unsigned operand bytes.
- Selectors 1/2 extract three bits at bit positions 0/3 of byte `+02`;
  selector 3 extracts six bits starting at bit 6 across `+02/+03`;
  selector 4 reads bits 4..6 of `+03`, and selector 7 extracts three bits
  starting at bit 7 across `+03/+04`.
- Selectors 14..18 extract widths 3/8/8/3/8 from starting byte/bit positions
  `+0E:0`, `+0E:3`, `+0F:3`, `+10:3`, `+10:6`.
- Selectors 19 and 20 return the same original encoded address, record `+12`.
  Selectors 21/22/23/24 return addresses at `+16/+1A/+1E/+22`.
  The getter does not read the pointed bytes or impose an element count.
  A separate direct reader composes LE16 from `+12/+13`.
- Selectors 25..28 read four LE16 values at `+22/+24/+26/+28`, sharing storage
  with the view returned by selector 24.
- Selectors 29/31 read unsigned bytes at `+35/+34`. Selector 30 returns
  byte `+33` multiplied by 200, and selector 54 similarly uses `+3E`.
- Selector 32 returns the original encoded address at `+2A`; selectors 33..41
  read the nine unsigned bytes at `+2A..+32`, sharing that storage.
- Selectors 42/44/53 map bytes `+36/+37/+3D` through the pure helper at
  `0x0CEEEC`: zero remains zero, otherwise add `0x0177` (375). The getter
  narrows the result to 16 bits. This helper has no live-memory dependency.
- Selectors 46..52 extract widths 4/7/6/4/7/6/6 from starting byte/bit positions
  `+38:0`, `+38:4`, `+39:3`, `+3A:1`, `+3A:5`, `+3B:4`, `+3C:2`.
- Selectors 5/6/55/56/43/45/61/62/63/64 expose bits 0 through 9 of LE16
  storage at `+41`. Selector 63 reads only byte `+42`; the others read both
  bytes. The complete two-byte storage remains intact.
- Selectors 57/58 read bytes `+3F/+40`; selector 59 reads LE16 at `+43`.
  Selector 60 reads the event-row byte at `+45` (69 decimal).

All storage survives reconstruction, including the 16 inline bytes at
`+12..+21`, unused getter bits in byte `+04`, byte `+11` and the flag word,
raw mapped/scaled operands, record zero, and every physical record identity.
The observed low ten bits of `+00` equal each physical index in the base ROM;
this observation does not become a parser constraint or a new guest bound.
No stored word is classified as a ROM pointer merely from its numeric value.

The already-closed normal selected-mission producer reads byte `+45` and
uses `0x71C760 + 12 * event_row`. The 512 numeric references have 173 distinct
values and four repeated-value groups. The zero value retains caller-specific
handling; it does not establish a universal null rule or the event catalog's
extent. Computed inline views preserve their original encoded addresses and
aliases rather than becoming separate owners.

All 35,840 bytes reconstruct exactly in original order. The span is disjoint
from all 77 accepted e85-d84 resources and their 50,491 source spans. Nineteen
authenticated windows (nine new, ten cached), 157 instruction/literal pins and
110 input hash/mtime guards pass. Legacy cached text is checked instruction by
instruction against its protected ROM window and pinned with its current text
hash; an unavailable historical text guard is not claimed. The aligned encoded
word census finds 38 base literals, two origin-plus-stride literals and two
interior data-word candidates without a proven reader role. It does not claim
all indirect or unaligned references.

This unit performs static inspection only: no build, Wine runtime test, game
execution, checkpoint, navigation or previous-proof replay. Schema 1, e85-d84
and the accepted 78 files remain unchanged; artifact completion retires zero
paths. Private evidence under `target/rom-data-mission-records70/` includes
the source contract, selectors, lossless records, numeric references/aliases,
root census, authenticated windows, pins and preservation/cleanup receipts.

The strict model and original-getter Wine proof now pass below; managed runtime,
actual CLI and ordinary use remain pending. Full caller ABI, all unnamed field meanings,
global inline-view element counts, successor-array and event-catalog contracts,
Linux, USA, native Windows, new32:9 and the full ROM TODO are unverified. No
performance or storage campaign, Android change, package, assistant commit or
push occurs here.

## Typed mission records and original-getter proof

**ROMDATA-02-MISSION-RECORDS70 — complete for the declared model and EUR/Wine
getter proof, 29 October 2026.**
[`mission_records70`](../crates/gba-core/src/rom_data/mission_records70.rs)
adds a strict, lossless core model of all 512 records and one 35,840-byte source
span. Scalar integers, fixed byte arrays, packed residual bits, mapped/scaled
raw operands and physical order remain intact. Unknown JSON fields, wrong
numeric types/widths, array lengths and physical shape are rejected. Shape
validation is separate from authentication against the exact EUR ROM, complete
source hash and all 19 source-contract windows; it does not impose new value
enums or require the observed header/index relationship.

`Records70::query` narrows index/selector to their original low 16/8 bits.
Invalid selectors return `Some(0)` without a record lookup. The seven address
selectors compute their original encoded addresses even for indices outside
this physical owner. An unowned scalar returns `None`, meaning that the owner
cannot answer that original read; it introduces no guest bounds check or null
fallback. Inline aliases, zero mapping, multiplication by 200 and every unsigned
selector projection retain the source contract.

One warning-free Windows core-test build uses 571 frozen source files and 242
selected core inputs, preserving all 569 accepted preimage hashes. Only the
new model/test leaves and one public module declaration are overlaid; removing
that declaration restores the core hub byte-for-byte. The executable SHA-256 is
`663e8b87330b587bae353fb103acdf0bf39f3ef55c153cd218c31de6500ae72a`.
No frontend or parallel menu/content/debug change is compiled in this unit.

One focused Wine filter passes 21 format/authentication guards and 698 calls
through the original getter, from its entry at `0x0D1F30` through its return,
including the real pure helper at `0x0CEEEC`. The 438 base cases cover all 65
selectors at physical indices 0/1/58/299/300/511, high-bit normalization,
invalid-selector zero, computed views beyond this owner and scalar reads at
indices 512/65535. Those scalar probes verify the real raw getter result while
the model correctly reports no owned value. The 260 synthetic calls use four
isolated 70-byte patterns at record 511 with high incoming index/selector bits.
No ordinary checkpoint, cartridge save or game fixture is edited.

Independent Python expectations, the typed model and actual return values
agree. Each observed unsigned-byte load has the expected effective address and
order, including the one-byte path for flag bit 8. All EWRAM, VRAM, OAM and
palette bytes are unchanged. IWRAM matches exactly, including the expected
main/helper return-link words; SP, R4 through R12, Thumb return state and final
ROM bytes are checked. The original getter reads 324 distinct owned source
bytes across the base cases and 54 across the synthetic cases. These are
separate from complete 35,840-byte reconstruction: address-returning selectors
do not dereference the 16 inline bytes at `+12..+21`. The 17,213 executed
instructions are coverage only.

All 139 preservation guards, 571 frozen hashes, three current implementation
hashes, 569 prior accepted hashes and 110 closed-source guards remain exact.
Artifact completion retires zero paths. Private evidence in
`target/rom-data-mission-records70-model/` retains independent model/cases/source
expectations, source preimages/snapshot, executable, Windows build and Wine
results, read-coverage counts and final/preservation/cleanup receipts.

Schema 1, e85-d84, 77 managed resources and 78 accepted files remain unchanged.
Managed runtime/native-clock/lifecycle checks now pass below; actual CLI and
ordinary use are the next work. Full caller ABI, other field meanings, indirect references,
successor arrays/event-catalog contracts, Linux, USA, native Windows, new32:9 and
the full ROM TODO remain pending. No completed proof is replayed, and no
performance/storage measurement, Android/package change, assistant commit or
push occurs.

## Managed mission records and typed runtime reads

**ROMDATA-03-04-MISSION-RECORDS70-RUNTIME — complete for combined EUR/Wine
publication and runtime scopes, 29 October 2026.** e86-d85 adds resource index
77, `gameplay.mission-records70`, file `mission-records70.json`, format
`mission-records70-v1-json`. Its single 35,840-byte source span has no
dependencies. The generation contains 79 files and 78 resources; all 77 prior
resource records and payload files remain exact.

The immutable owner serves original source bytes on pages `0x713..0x71C`,
composing the previous conditional-record owner on the left and raw successor
bytes on the right. Original guest selectors, encoded addresses, event joins
and timing remain guest code. Ten disjoint physical storage classes count word,
packed, operand, inline, byte-list, scaled, scalar, mapped, flag and event bytes.
Together with request/served-byte counts and physical records used, these form
13 counters; eight 64-bit masks retain all 512 physical identities. These names
classify storage and do not assign new game semantics.

Two warning-free Windows host-test builds use 573 matched frozen sources and
243 selected core inputs. The first executable, `73326ad9..`, passes creation,
all 77 prior payload/manifest identities and interrupted publication of the new
file. Its first positive attach then fails the historical 1 KiB shared/OBJ
allocation cap, before attach guards, native cases or lifecycle checks execute.
That calculation includes the entire growing `RuntimeResources` structure and
its resource handles; it is not a measurement of the new payload or process RSS.

Following the requested deferral of space limits, attach no longer rejects the
authenticated owner using the historical 1 KiB shared/OBJ or 64 KiB BG allocation
caps. Three legacy allocation assertions are removed. Input parsing bounds,
complete ROM/source authentication, typed bindings and transactional attach
remain active. No capacity optimization or performance/RSS campaign is run.

The second executable is
`455bdb1cd8aef0ccd72df16ceafbb0f483d6b4cebb36891c492251efaccf7276`.
A remaining-only Wine filter reuses the same published generation, hydrates only
its file hashes into independently ROM-derived metadata and validates every
source/format/dependency/order identity. It passes 14 rejected-attach cases,
two incomplete-resource-set cases and unchanged complete state/owner after
failure. Creation and interrupted publication are not repeated. The initial
failed filter remains a failure; acceptance combines its closed publication
scope with the second filter's runtime scope.

Forty-one original-code cases compare complete serialized states and guest
clocks across raw interpreter, typed interpreter and typed compiled execution:
16 load prefixes and 25 getter calls, including the pure helper, normalized
inputs, an invalid selector, an unowned scalar and an unbounded inline address.
All 35,840 raw source bytes are altered **after** Load/setup. Three saved native
objects retain libraries through use, fuse Thumb BL pairs and exclude each
family's stop boundary. Every native dispatch accounts for direct read callbacks
and the real `interp_one` callback. Native execution serves 52 new bytes and five
previous-family bytes, all attributed; its fallback-source deltas are zero.
These are served-byte totals, not a distinct-byte union or whole-catalog native
coverage. The 168 compiled entries and 69 dispatches are coverage only.

Separate bus checks read all 35,840 altered bytes through the typed owner, all
ten classes and 512 identities, 512 record boundaries, widths, ROM mirrors and
typed/prior/raw edge compositions. Missing files and failed attach preserve the
same owner. Load, slot Load and Undo retain the same Arc. DMA and CpuSet copy
17,922 halfwords; CpuFastSet copies 8,968 words. All three transfers compare
complete states and guest clocks, including the raw bytes beyond this owner.
The generation's 79 payload/manifest hashes still match the first publication.

The dataset SHA-256 is
`0c170fce4b8a04b6a5beb839ec7dbfca810824fb4da87b7417f728b0825fc575`,
the manifest hash is
`8dc9d972ff9d6603522f879f77c7c04676dbd17565db5a4d4bdf1ec98055eeaa`,
and the new JSON hash is
`e41044165ac3391e091d3cd4174f3b98c9d96e7c62645d7251e482c6752ce5ce`.
All 860 guards, accepted/diagnostic 573-file snapshots, 571 model preimages,
eight current implementation hashes and 78 prior files remain exact. Artifact
completion retires only the disposable interrupted-publication copy. Private
evidence under `target/rom-data-mission-records70-runtime/` retains the accepted
dataset, both executables/snapshots/logs, allocation-fix preimages, native
objects/observations and combined `close-final.py` receipts. The original
`close-runtime.py` assumptions are obsolete and that script is not executed.

No source/model/old ordinary proof, normal checkpoint, navigation or cartridge
write is replayed. Machine/save/native-cache/C API ABI remain unchanged. Actual
production CLI registration now passes below; one positive ordinary suffix/full
pair is next. Full caller ABI, indirect references, successor-array/event-catalog
contracts, Linux, USA, native Windows, new32:9 and the full ROM TODO remain
pending. Parallel menu/content/debug/package work is preserved and excluded
from these frozen builds. No performance/storage campaign, Android change,
package, assistant commit or push occurs.


## Actual mission-record CLI registration

**ROMDATA-04-MISSION-RECORDS70-CLI — registration/startup complete for EUR/Wine,
29 October 2026.** One warning-free production Windows build uses 573 files
byte-identical to the accepted runtime snapshot, without source overlays. Its
executable SHA-256 is
`738b7f204da2fb45b0615c3673114ec604fb72a654810edbb3567be3d996f382`.
Only the affected core-library/production and matching-menu fingerprints are
isolated. The canonical executable's original hash and modification time are
restored in the build's `finally` path. Parallel menu/content/debug/package
changes are preserved and excluded from the frozen source snapshot.

Three actual CLI children under Wine exit zero: creation of e86-d85, reuse of
that same generation, and `runc --rom-data` startup for one neutral frame. All 79
generation files equal the accepted runtime dataset, all 78 logged resource
ID/hash pairs match, and all 81 managed paths retain hashes and modification
times during reuse/startup. The old 28,859-block native library loads unchanged.
The new entry is index 77, `gameplay.mission-records70`, with its single
35,840-byte source span and no dependencies.

The private generator initially matches the wrong dictionary-index token and
stops on its own assertion before any Wine child. Only its generator section is
corrected; the manifest's actual new-resource index is checked before launch.
The 573-file snapshot, canonical preimage and ongoing valid build remain intact.
No source copy, compilation or actual CLI preparation is repeated.

Startup uses an exact local copy of the retained normal guest24755 state and
one neutral frame to24756. Its original SHA-256 is
`79be0fcf4f674f69975c915f17410c0610df8bb4b1e7f899cbc4251f373e27ff`.
All 13 mission-record counters are zero. No action, navigation, new checkpoint
or saved endpoint is produced. This proves registration/startup, without
accepting a positive ordinary case, a new negative locator phase or full-pair
equivalence.

All 195 guards, 573 CLI and 573 accepted-runtime source hashes, original inputs,
79 accepted files, canonical executable, native library and TinyCC loader remain
exact. Artifact completion retires zero paths. Private evidence under
`target/rom-data-mission-records70-cli/` retains source/canonical preimages, the
corrected generator, executable, CLI child/no-write receipts and final summary.
The generation is `windows-cli/resources/<EUR SHA>/v1/e86-d85/generation-hu7Ei8`.

Source, model, native and lifecycle proofs remain closed. One positive ordinary
mission-information suffix and complete state/clock/image/OAM/audio pair now pass
below. Linux, USA, native
Windows, new32:9, full caller ABI, indirect references, successor-array/event
catalog contracts and the full ROM TODO remain pending. No performance/storage
campaign, Android/package change, assistant commit or push occurs.


## Ordinary mission-information equivalence

**ROMDATA-04-05-MISSION-RECORDS70-MISSIONS — complete for one ordinary EUR/Wine
case, 29 October 2026.** Normal input opens the Giza Plains mission-information
page. The accepted33-frame suffix runs guest15017..15050 and serves five new
bytes in five requests, all in the packed-byte class, touching one physical
record. All13 counters match capture, actual CLI and full-pair replay. The other
nine field classes are zero: this does not prove all fields of that record, all
512 records, five distinct bytes, acceptance of a mission or a battle outcome.

The original normal Missions-list checkpoint at14969 retains SHA-256
`b3195906e512e163cce1a88c051d5bce39e7865c4a11daf2da3be31dc3acd5a0`.
Its already-authenticated classic PNG and projection receipt are read and
inspected without rendering or replaying the earlier620/397-frame routes.
Desert Peril is selected, with Giza Plains immediately below. One new capture
presses Down for three frames at relative8 and A for12 at relative40. The first
new-source read is at relative49, guest15019. The observer stops after its
32-frame activation window, executing81 frames in total. Its rolling relative48
checkpoint supplies the33-frame suffix: A held for four frames, then29 neutral.
Discovery stops after this single positive capture.

The capture's actual Wine child and test pass. Its private parent helper then
fails while registering completed outputs because a steps list was absent.
Only existing output registration is repaired; no second capture, Wine child,
game input or build is run for that bookkeeping error. The endpoint's complete
wide284x160 PPM matches the authenticated FFSTATE framebuffer with bit-replicated
RGB conversion. The derived classic PNG shows Giza Plains, rank1, information
page1/3. The old seed receipt separately retains its classic240x160/scaled-color
profile. No guest RAM/register/task/ROM edit, forced identifier or cartridge
write is used.

One warning-free Windows host-test build uses574 frozen files: the accepted573
CLI sources plus one passive observer leaf, its test-module declaration and an
optional five-line positive-use guard. Removing the module/guard restores both
hubs byte-for-byte, and core/runtime sources are unchanged. Its SHA-256 is
`3d21e89cd252485aed02065e58c158712772c6e677582ff284e7af800172528e`.
The same executable performs the capture and complete pair.

The existing738b7f20 production CLI and e86-d85 generation-hu7Ei8 run the new
33-frame suffix without preparation, rebuilding or navigation. All78 logged
ID/hash pairs match, all81 managed paths retain hashes/mtimes, and the old
28,859-block native cache remains exact. One Wine full-pair test using the same
3d21e89c executable compares every33 complete serialized states/guest clocks,
classic/wide pixels, OAM and five audio streams. All pass with the exact13
counters above. Both arms use the same old native cache; this is data-service
equivalence, not whole interpreter/native parity. Dispatch168998/217502 and
audio sample counts `[72418, 72418, 8712, 8712, 72342]` are coverage only.

All133 guards,574 frozen hashes,573 CLI hashes, three current implementation
hashes, original seeds and81 managed files remain exact. Artifact completion
retires zero paths. Private evidence under
`target/rom-data-mission-records70-missions/` retains the seed/projection
receipts, normal plan, accepted `01-select-mission/before-activation.ffstate`,
`activation.input`, endpoint/observations, source preimages/snapshot, executable,
CLI/pair results and final preservation/cleanup receipts.

This closes the declared mission-record source/model/managed/runtime/actual CLI
chain and this ordinary case. Full caller ABI, all indirect references, other
ordinary field classes, successor byte-array/event-catalog contracts, Linux,
USA, native Windows, new32:9 and the full ROM TODO remain pending. No closed
proof, old route or fixture is recreated; no performance/storage campaign,
Android/package change, assistant commit or push occurs.


## European selection-byte-table source contract

**ROMDATA-01-SELECTION-BYTE-TABLES — complete for the declared EUR source
contract, 29 October 2026.** Two unsigned-byte views occupy34 bytes:
`0x71C73C..0x71C740` contains four count thresholds, and
`0x71C740..0x71C75E` contains30 ordinal weights. The combined SHA-256 is
`d758af5f1a07613f1c2047257d42ef9cbfac83b63783ee6fd0e7b3879370b8dd`.
The proposed neutral resource is `gameplay.selection-byte-tables`; semantic
mission/region names are not inferred from neighboring assets.

The original routine at `0x0D3228` counts nonzero low-three-bit headers in five
live six-byte records at `0x02002E34`. It examines all five headers and permits
only counts0..3 to read the first array; counts4/5 exit before that ROM load.
The independent second root begins exactly four bytes later. The threshold is
loaded unsigned and compared against a signed16 result from external numeric
helpers, with equality/below continuing. Their second argument is100 at this
stage. This establishes the data's role without imposing a percentage range,
RNG distribution, modulo formula or new guest-wide count limit.

Two stride-one scans independently reload the weight root and use ordinals
1..30, corresponding to physical positions0..29. Each queries the real helper
at `0x0377BC`. That helper narrows its input to eight bits and returns the raw
live byte at `0x02002CC3 + 12 * (low8(ordinal) - 1)`, using32-bit address
arithmetic. The helper itself has no0/30 guard and does not normalize to bool;
its callers treat any nonzero low byte as eligible. Mutable EWRAM is excluded
from the exported base data.

The first scan adds each eligible unsigned weight and still visits every
ordinal when a value is zero. A zero total exits before the next numeric stage.
The second stage retains the full32-bit helper result as a rank; its second
argument is the first-scan total. A fresh live query precedes each eligible
weight read. The original subtracts the unsigned weight with32-bit wrap and
advances while the resulting signed rank is nonnegative. The first negative
result selects its ordinal; exhaustion at31 exits the routine path. No extra
negative-rank rejection, zero-weight shortcut or cached eligibility is invented.
These describe the observed stages, not a complete routine/RNG model.

All34 bytes reconstruct exactly in physical order. Equal values remain distinct
positions; there are no stored ROM pointers or implied pointer aliases. The
aligned encoded-word census finds one literal for each root and36 for the next
root at `0x71C760`, whose independent reader uses stride12. The two bytes at
`0x71C75E..0x71C760` remain opaque and unowned; no padding/header or event-catalog
extent is inferred. The new spans are disjoint from78 accepted e86-d85 resources
and50,492 existing source spans.

Six authenticated static windows (two new, four cached),95 instruction/literal
pins and96 input hash/mtime guards pass. The additional windows establish the
live-header base and exact eligibility reader; the already-closed mission
source/ordinary proofs are not repeated. The cached text is authenticated
against both its exact ROM window and opcode bytes. Artifact completion retires
zero paths. Private evidence under `target/rom-data-selection-byte-tables/`
retains arrays, per-position/live-input references, equal-value groups, two
source spans, root census, authenticated windows, pins and preservation receipts.

This is static inspection only: no build, Wine/game execution, fixture or
navigation. Schema1/e86-d85/79 files remain unchanged. The strict typed model
and original-prefix Wine proof now pass below; managed runtime also passes.
Actual CLI and one ordinary threshold-use case now pass below. Full routine ABI, numeric-helper internals/distribution, all indirect
or unaligned references, global ordinal semantics, the opaque gap, event-catalog
extent, Linux, USA, native Windows, new32:9 and the full ROM TODO remain pending.
No performance/storage campaign, Android/package change, assistant commit or
push occurs.


## Typed selection-byte tables and original-stage proof

**ROMDATA-02-SELECTION-BYTE-TABLES — complete for the declared EUR model and
Wine stages, 29 October 2026.**
[`selection_byte_tables`](../crates/gba-core/src/rom_data/selection_byte_tables.rs)
retains the four thresholds and thirty weights as fixed unsigned-byte arrays,
with their two physical source offsets. Strict JSON rejects unknown fields,
wrong lengths/types and values outside u8. Shape validation checks offsets
without inventing value enums or percentage caps. Exact EUR ROM/source and
six reader-window fingerprints are authenticated separately. The two-byte gap
after the arrays is excluded from byte ownership; equal values keep their
physical positions. All34 bytes and both source spans reconstruct exactly.

Diagnostic methods count all five low-three-bit live headers, sum eligible
weights, and scan for the first signed-negative result after a wrapping32-bit
subtraction. Each scan requests the original raw live status byte in ordinal
order. A zero weight still participates; a supplied negative rank is not
rejected. These methods describe individual stages, not the numeric helpers
or the complete routine. The guest's zero-total exit precedes the second
numeric stage. Negative-rank/zero-total combinations in isolated choice probes
do not establish that the complete routine can produce them.

One warning-free Windows core build uses576 frozen files/245 selected core
inputs. It overlays only two leaves and one module declaration onto574 accepted
source hashes; removing the declaration restores the core hub byte-for-byte.
Executable SHA-256:
`b40503b87caafceae22f9b023a498df868e6fa31a7e49c71868f9d0b187d1ebc`.
One Wine filter passes18 shape/JSON/authentication guards,45 base and24
isolated synthetic stage cases. It executes original count/threshold code
from `0x0D323C`, sum/zero-total code from `0x0D327E`, and choice code from
`0x0D32B4`, including actual live reader `0x0377BC`. Independent Python
expectations, the typed projections, actual results, live-query order and
unsigned ROM-load order agree. Cases include counts0..5, masked high bits,
zero/nonzero live bytes, exhaustion, zero/all255/alternating/numeric source
patterns and signed32 extremes with wrap.

The original stages observe all34 base source bytes and32 synthetic bytes;
complete reconstruction remains a separate check. EWRAM/IWRAM/VRAM/OAM/palette
remain byte-exact; SP/R8..R12 and final ROM bytes are checked. The30,480 executed
instructions are coverage only. All123 preservation guards,576 frozen hashes,
three current hashes,574 accepted preimages and96 closed-source guards pass.
Artifact completion retires zero paths. Private evidence under
`target/rom-data-selection-byte-tables-model/` retains independent cases/model/
source expectations, preimages, immutable snapshot, executable, build/Wine
results and final preservation/cleanup receipts.

This model unit leaves Schema1/e86-d85/79 managed files unchanged. Managed
publication and typed bus/native state-clock/lifecycle checks now pass below
as e87-d86. Actual CLI and one ordinary threshold-use case now pass below. Numeric-helper internals/distribution, prologue/
epilogue/full routine ABI, all indirect references, opaque gap/event-catalog
extent, Linux, USA, native Windows, new32:9 and the full ROM TODO remain open.
Parallel frontend edits are preserved outside this core snapshot. No ordinary
checkpoint is changed or navigation repeated, and no performance measurement
is performed.


## Managed selection-byte tables and typed runtime reads

**ROMDATA-03-04-SELECTION-BYTE-TABLES-RUNTIME — complete for the declared
EUR/Wine managed and runtime scope, 29 October 2026.** e87-d86 adds index78,
`gameplay.selection-byte-tables`, in `selection-byte-tables.json` with format
`selection-byte-tables-v1-json`, two source spans of4/30 bytes and no dependency.
All78 prior resource records/coverage entries and payloads remain exact; there
are80 files/79 resources. Schema1 and USAe8-d7 remain unchanged.

The immutable fixed-array owner supplies only34 bytes on physical page71C.
Reads compose with the previous mission-record owner and raw bytes beyond
`0x71C75E`, including the opaque gap and next event root. Original count, live
eligibility, numeric helpers, wrapping rank arithmetic and guest control flow
remain original instructions. Six nonserialized counters distinguish bus reads,
served bytes, threshold/weight bytes and physical threshold/weight positions.
Repeated equal values keep separate position bits. No guest Machine, save-state,
native-cache or C API ABI changes. Strict bounded parsing and complete source/
format/dependency/full-set authentication precede transactional attachment; no
allocation cap is introduced.

One warning-free Windows host-test build uses578 frozen files/246 core inputs
from576 accepted model sources plus two new leaves. Executable SHA-256:
`e5b4566884e6e4d3c41565c5e0aefef0c808842c3f3bd181e6e04c8826ebd29d`.
One Wine filter passes new80-file publication, all78 prior records/files exact,
and failure while writing the new JSON without publishing a valid manifest.
Fourteen forged payload/offset/source length/hash/role/extra span/dependency/
format/coverage/JSON cases and two incomplete-set cases are rejected. Failed
attachment preserves the same owner and complete serialized guest state.

Twenty original/interpreted/native cases compare complete state and guest clock:
12 byte/halfword/word prefixes cover both arrays, mirrors, the prior owner and
raw right border; eight count/sum/choice stages include count rejection, zero
weight total, selection and exhaustion with the real live helper. Four separate
native families exclude their stop boundaries. All34 raw source bytes and four
prior-owner bytes are changed only after Load/setup; restoring the ROM before
state comparison preserves exact identities. Actual typed service totals76 new
bytes and five previous-owner bytes, all attributed to read8/16/32 or interp_one
callbacks, with zero fallback-source deltas. These are served-byte counts, not
a distinct-byte union or full routine/ABI proof. The91 compiled entries and119
dispatches are coverage only.

Bus checks read every34 bytes, all4/30 physical positions and both storage
classes, then test unaligned C API widths, mirrors, both array boundaries and
prior/raw composition. An unrelated old owner leaves the new counters unchanged.
With dataset files unavailable, failed reattach, snapshot Load, slot Load and
Undo retain the same Arc and exact states. DMA3/CpuSet20 halfwords and CpuFastSet
16 words match complete states/clocks, including the raw surrounding bytes.

All243 guards,578 frozen hashes,576 model hashes, seven current implementation
hashes and79 prior files pass. Cleanup retires only the successful interrupted
publication copy; the80-file accepted dataset, source preimages, executable,
four native objects and build/Wine/results remain under
`target/rom-data-selection-byte-tables-runtime/`. Dataset identity:
`2721d6d198605abc19a9f073ca3c82117171bfafb916b3015f6951212c3f2405`.
Manifest SHA-256:
`3915947729adcea74b01c2017ec55d5d1f42a2317860a8973141ca13618707c7`.
JSON SHA-256:
`b02a0019a8fe571869d4c08dea47b92b64e6b345abd5fe9cffe7c7bf9fd1ccdb`.

Actual production CLI registration/reuse and one ordinary threshold-use case now
pass below. Numeric-helper internals/distribution, full routine ABI, all indirect
references, opaque gap/event-catalog extent, Linux, USA, native Windows, new32:9
and the full ROM TODO remain pending. Source/model/old acceptance proofs are not
repeated; ordinary checkpoints/navigation and parallel frontend edits stay
outside this unit. No performance or storage campaign is performed.


## Actual selection-byte CLI registration

**ROMDATA-04-SELECTION-BYTE-TABLES-CLI — complete for actual EUR CLI
registration/startup, 29 October 2026.** One warning-free production Windows
build uses the same578 accepted runtime sources, with no source overlays.
Its SHA-256 is
`22af4132047c497ec11c461cd911f7a531b50062113578b841c4aa5be37da1d1`.
The canonical executable is restored with its original hash and modification
time after the build. Parallel frontend edits remain outside this snapshot.

One Wine orchestration runs three actual commands: create the e87-d86 dataset,
reuse it and start one neutral frame from a copy of the retained normal
frame24755 checkpoint. All exit0. All80 generation files match the accepted
runtime dataset, all79 runtime ID/hash pairs match the manifest, and all82
managed paths retain their hashes/mtimes during reuse and startup. Index78 has
the correct ID/format, no dependencies and two source spans totaling34 bytes.
The old28,859-block native cache is loaded unchanged; no translation is rebuilt.

The guest advances24755→24756 with all six new usage counters zero. This
verifies attachment/startup, not positive ordinary consumption or scene
equivalence. No action, navigation, new checkpoint derivation, saved endpoint,
source/model/runtime proof replay or performance measurement occurs. All192
preservation guards,578 CLI and578 runtime source hashes,80 accepted payloads,
ROM, native cache, TinyCC, original seed and canonical executable pass. Artifact
completion retires zero paths. Private evidence under
`target/rom-data-selection-byte-tables-cli/` retains preimages, immutable source
snapshot, executable, three command logs, no-write/summary/preservation/cleanup
receipts and the e87-d86 generation-ZloxMx dataset.

One ordinary positive threshold-use suffix and complete state/clock/image/OAM/
audio comparison now pass below. Full routine ABI, numeric-helper internals/distribution, indirect
references, opaque gap/event-catalog extent, Linux, USA, native Windows, new32:9
and the full ROM TODO remain open. No unrelated parallel work is attributed
to this validation.


## Ordinary selection-threshold travel equivalence

**ROMDATA-04-SELECTION-BYTE-TABLES-ORDINARY — complete for one ordinary EUR
threshold-use case on Wine, 29 October 2026.** New static caller inspection
finds the direct Thumb call at `0x0D3724` into `0x0D3228`. Its caller narrows
the incoming argument to eight bits and skips this path when zero. Two other
direct callers supply zero; wrapper `0x0D6970` supplies one and has three direct
call candidates. Eight bounded new windows/354 opcode pins authenticate this
chain. This is a direct-call census, not complete indirect reachability.

Readonly decoding of four selected, checksum-authenticated EUR snapshots finds
initial guard byte6 and one/two nonzero low-three-bit headers. The existing
normal Giza map checkpoint at guest19049 is selected, with two headers and
seven nonzero live status bytes. Its cached map image is inspected without
rerendering or regenerating the checkpoint. The old Giza travel proof remains
closed; this capture supplies the new owner's missing ordinary-use criterion.

A test-only passive observer leaf, one module declaration and an optional
five-line positive-use guard are overlaid onto the accepted578-source CLI
snapshot. The579-file Windows build is warning-free; executable SHA-256:
`8f3bee9797a1bb3666697ba487ae67b83584537646d9ef05ba9d2c7642b0d7de`.
Inverse hub/validator edits reproduce the preimages exactly; core/runtime
sources remain unchanged. Parallel frontend work is excluded.

One normal capture on an exact seed copy uses A for12frames from relative1,
then neutral input, bounded to360frames. It stops after129frames,19049→19178,
with first source use at relative97/guest19147. The retained rolling point at
relative96 gives a33-frame suffix19145→19178. Discovery stops after this one
positive capture. ROM, original checkpoints and cartridge saves remain unchanged;
there is no manual RAM/register/task/ID edit or numeric/RNG forcing.

All six counters agree across capture, the same22af4132 production CLI and the
same8f3bee97 test executable's complete pair:

- bus reads1; served source bytes1; threshold bytes1; weight bytes0;
- physical threshold positions1; physical weight positions0.

The ordinary path therefore proves one threshold position, not all four, any
weight position, successful selection or an event/mission outcome. Weight use
and wider source coverage retain the targeted source/model/native evidence.
The authenticated fullwide284×160 PPM matches the complete saved framebuffer;
a readonly classic projection shows the world-map/calendar screen near Giza.

The actual CLI reuses the same e87-d86 generation-ZloxMx without preparation,
rebuilding or navigation. All79 ID/hash pairs and82 managed path hashes/mtimes
remain exact, with the old28,859-block native cache unchanged. One Wine full
pair compares all33 complete serialized states/clocks, central/wide pixels,
OAM and five audio streams; every comparison passes with identical six counts.
Both arms use the same old native cache: this is data-service equivalence, not
a claim of whole interpreter/native parity. Dispatch203258/519441 and audio
counts `[72418, 72418, 8712, 8712, 72342]` are coverage only.

All140 guards,579 frozen source hashes,578 CLI hashes, three current paths,
original seed and82 managed files remain exact. Artifact completion retires
zero paths. Private evidence under
`target/rom-data-selection-byte-tables-ordinary/` retains caller windows/gates,
normal seed copies/input, accepted33-frame checkpoint/suffix, endpoint images/
projection receipt, source preimages/snapshot, executable and CLI/pair/final
preservation/cleanup results.

This closes the declared source/model/managed/runtime/actual CLI chain and one
ordinary case. Other ordinary threshold/weight positions, full routine ABI,
numeric-helper internals/distribution, indirect references, opaque gap/event-
catalog extent, Linux, USA, native Windows, new32:9 and the full ROM TODO remain
pending. No closed source/model/native/registration proof is replayed, and no
performance/storage campaign, Android/package change, assistant commit or push
occurs.


## European event-record source contract

**ROMDATA-01-EVENT-RECORDS12 — complete for the declared EUR source contract,
29 October 2026.** `0x71C760..0x71D360` contains256 ordered12-byte records,
3,072 bytes, SHA-256
`e74541202207ed2249fcec14f6ca56073ce3b786acaa08f8b0efbdec1d858f24`.
The proposed resource is `gameplay.event-records12`, with one physical owner.
It is disjoint from79 accepted e87-d86 resources/50,494 source spans.

The extent combines three witnesses: the explicit backward scan of indices
255..1 from `base + 255*12`, a live unsigned-byte selector addressing0..255,
and the independently rooted next catalog at `0x71D360`, used with stride27.
This establishes physical storage, not a new global guest index cap. Another
reader loads an unsigned16 index and computes `base + 12*index` without checking
256; queries beyond physical ownership retain their original outside reads.
Index0 is retained even though this script scan skips it.

The lossless layout is structural: unsigned header00/key01 bytes, unsigned
little-endian primary02/alternate04 halfwords, unsigned parameter06/byte07,
raw flags08 halfword and two opaque bytes at0A..0B. The original flags reader
masks8; byte07 is tested for zero/nonzero. Residual bits and all opaque bytes
remain intact. Zero tails are not declared padding, and no value enum, global
null rule, new foreign-key cap or stored ROM-pointer interpretation is added.
All3,072 bytes reconstruct in order; repeated values/records retain positions.

The pointer at `0x71D1EC` is exactly physical row225 of this owner, with31
remaining physical positions. It is a shared interior view, not another asset.
The conditional reader at `0x009D04` narrows its argument to eight bits. When
the supplied record's alternate04 is nonzero, it reads primary02 from
`0x71D1EC + 12*low8(argument)` and keeps the supplied alternate04; otherwise it
uses direct primary02 and alternate0. That helper has no31-position guard,
so high arguments may refer outside this owner. The selection suffix separately
uses ordinals1..30 into this view, corresponding to physical rows226..255.
No host cap or duplicate child ownership is introduced.

The script callback at `0x127388` reads a complete16-bit key from two operand
bytes, then compares it against unsigned key01 while scanning255..1. It stops
on the first backward match, so repeated keys choose their highest physical
row. A key above255 cannot match; row0 is excluded regardless of its contents.
A match stores the physical byte index at `0x02003C2A` and passes primary02 with
alternate0 to the original live setter. Eight duplicate-key groups and one
identical-record group are retained. This is static behavior, not new script
or event execution acceptance.

The closed normal mission-selection evidence supplies a separate join from
mission byte45 to an unsigned event-row index. Event key01 feeds original event
start logic, including the special4A live-selector path. Parameter06 feeds
the independently rooted27-byte catalog at `0x71D360`; its extent and field
semantics remain a separate task. Primary/alternate targets retain their own
owners and outside behavior; this contract imposes no unconditional target cap.

Fifteen authenticated windows (ten new/five cached),67 explicit instruction/
literal pins and112 hash/mtime guards pass. Cached windows are checked against
their declared ROM hashes and every opcode; the old initial preservation map
does not contain every later window, so no missing historical text stamp is
invented. A private helper stopped before writing outputs on that missing-key
assumption; it was corrected to authenticate those cached windows directly.
No disassembly, build, test or old acceptance is repeated to repair that helper.

The full-ROM aligned encoded-word census is reused:36 base literals, two
interior-view literals and25 next-root literals; there are no other candidates
inside this range in that representation. This is not all indirect or unaligned
references. Artifact completion retires zero paths. Private evidence in
`target/rom-data-event-records12/` retains ordered records, references, interior
views, source span, root census, authenticated windows, pins and preservation
receipts. Source tracing uses no build, Wine/game execution, fixture or
navigation. Schema1/e87-d86/80 managed files remain unchanged.

The strict typed model/original-prefix Wine proof and managed runtime now pass
below; actual CLI and one ordinary mission-entry case also pass below. Full routine ABI, live script/
event state effects, indirect references, the preceding two-byte opaque gap,
next27-byte catalog extent, Linux, USA, native Windows, new32:9 and the full
ROM TODO remain pending. No performance/storage campaign, Android/package
change, assistant commit or push occurs.


## Typed event records and original-prefix proof

**ROMDATA-02-EVENT-RECORDS12 — complete for the declared EUR model/prefix scope
on Wine, 30 October 2026.**
[`event_records12`](../crates/gba-core/src/rom_data/event_records12.rs) retains
256 ordered12-byte records, unsigned fields, all flags and the two opaque tail
bytes. Strict JSON rejects unknown fields, wrong types/widths/array lengths;
shape validation checks source offset, count and ordered physical index only.
It imposes no observed-value enum, zero-null rule or foreign-key cap. Exact EUR
ROM/source and all15 source-window fingerprints are authenticated separately.
All3,072 bytes and one source span reconstruct exactly.

Physical `record`/`byte_at` queries return absence outside this owner. The
computed view narrows its supplied index to16bits and retains the original
encoded address beyond physical index255. The shared interior view narrows
its argument to8bits; physical indices225..255 answer arguments0..30, while
higher arguments retain an encoded address and unanswered owned data. This
introduces no guest bounds check or raw-zero fallback. The conditional pair
uses the supplied record's alternate04 as the gate: direct primary/zero when
zero, alias primary/original alternate when nonzero. The script projection
compares the full16-bit key and finds the first match in backward255..1 order,
with row0 excluded even for synthetic data. Live setters/event start remain
original guest work.

One warning-free Windows core build uses581 frozen files/248 core inputs from
579 accepted preimages plus two leaves. Removing the one module declaration
restores the core hub byte-for-byte. Executable SHA-256:
`a4b88a69137416bfcfbecc75f1817146b637091ca9709293c2b011e1de2425e2`.
One Wine filter passes20 format/shape/authentication guards,51 base and37
isolated synthetic original-prefix cases. Independent Python expectations,
typed projections, actual results and byte-load order agree.

The original pair executes from `0x009D04` to immediately before its live
setter. The script scan executes from `0x127388` to immediately before the
matched setter or unmatched continuation. Seven unsigned byte/halfword load
prefixes cover header/key/primary/alternate/parameter/byte07/flags; another
prefix verifies the live-u16 computed view, including indices256/65535 and
high incoming values narrowed through the original halfword input. Pair cases
cover alias arguments30/31/255 and high-bit truncation; base keys include
values above255. Four synthetic byte patterns exercise full unsigned values,
nonzero opaque tails and repeated-key highest-match behavior. A separate
synthetic key confined to row0 proves its exclusion. These isolated patches
do not derive or modify an ordinary game checkpoint.

The original prefixes observe302 distinct base bytes and268 synthetic bytes;
this is separate from complete3,072-byte reconstruction. EWRAM matches exactly
except the expected matched script-index store. IWRAM matches including the
actual pair/scan prologue pushes; VRAM/OAM/palette remain exact. Prefix-final
SP/R8..R12 and restored ROM bytes are checked. Whole-routine entry/return ABI,
epilogues, live continuations and native clock equivalence are outside this
unit. The9,959 executed instructions are coverage only.

All133 preservation guards,581 frozen hashes, three current implementation
hashes,579 accepted preimages and112 closed-source guards pass. Artifact
completion retires zero paths. Private evidence under
`target/rom-data-event-records12-model/` retains independent model/cases/source
expectations, preimages, immutable source snapshot, executable, build/Wine
results and final preservation/cleanup receipts.

This model unit leaves Schema1/e87-d86/80 managed files unchanged. Managed
publication/typed bus/native-clock/lifecycle checks now pass below as e88-d87;
actual CLI and one ordinary case now pass below. Full routine ABI/live script-event effects, indirect references,
the preceding opaque gap, next27-byte catalog extent, Linux, USA, native
Windows, new32:9 and the full ROM TODO remain open. No closed proof, ordinary
fixture/navigation or performance measurement is repeated.


## Managed event records and typed runtime reads

**ROMDATA-03-04-EVENT-RECORDS12-RUNTIME — complete for the declared EUR/Wine
managed/runtime scope, 30 October 2026.** e88-d87 adds index79,
`gameplay.event-records12`, file `event-records12.json`, format
`event-records12-v1-json`, one3,072-byte source span and no dependency. There
are81 files/80 resources, with all79 prior resource records/coverage entries
and payloads exact. Schema1 and USAe8-d7 remain unchanged.

The immutable owner covers256 physical12-byte records on pages71C..71D. Its
row225 interior alias shares the same owner, without another asset/span or a
new guest bounds check. Reads compose with the two preceding selection-weight
bytes, the opaque two-byte gap and raw bytes beyond the end. Eight disjoint
storage classes cover header, key, primary, alternate, parameter, byte07, flags
and opaque tails. Eleven nonserialized counters include bus/served bytes,
those eight classes and physical records used; four mask words retain all256
positions independently of repeated values. Original script matching, live
setters/event start, alias arithmetic and outside reads remain guest work.
No Machine/save-state/native-cache/C API ABI change or allocation cap is added.

One warning-free Windows host-test build uses583 frozen files/249 core inputs,
from581 accepted model sources plus two new leaves. Its SHA-256 is
`9225a3bc69ac7ad586592653d203099b18594995b7b4a0f6bd2b64d5f882f698`.
New concurrent menu/debug-input changes remain outside this source snapshot.
One Wine filter passes publication of81 files, preservation of all79 prior
records/payloads and interrupted new-file publication without a valid manifest.
Fourteen payload/shape/source/hash/role/dependency/format/coverage/JSON forgeries
and two incomplete-set cases are rejected, preserving the same active owner
and complete guest state. Strict bounded parsing and full source authentication
precede transactional attachment.

Twenty-two controlled original/interpreted/native prefix cases compare complete
state and clock:14 load probes cover all eight storage classes, widths/mirrors,
physical borders and the interior view; eight pair/scan/u16-view prefixes cover
direct versus aliased pairs, outside alias arguments31/255, full16 nonmatching
keys and computed views beyond index255. Operand registers are assigned only
for isolated unit probes; this is not ordinary reachability. Four native
families exclude stop boundaries. All3,072 raw bytes and two prior-owner bytes
are changed after Load/setup, then restored before state comparison.

Actual service totals296 new bytes and three prior selection bytes. All deltas
are attributed to real read8/16/32 or interp_one callbacks; two of the296 new
bytes are served through interp_one fallback, zero prior bytes use that callback.
This is not296 direct native bytes, a distinct-byte union or whole-catalog
native coverage. The51 compiled entries/30 dispatches are coverage only.

Bus checks read every3,072 bytes, all eight classes and all256 physical records,
then verify unaligned C API widths, mirrors, all256 record boundaries, shared
alias and prior/raw-gap/next-root composition. An unrelated old owner leaves
the new counters unchanged. With dataset files unavailable, failed reattach,
snapshot Load, slot Load and Undo retain the same Arc and expected complete
states. DMA3/CpuSet1,538 halfwords and CpuFastSet776 words match complete states
and clocks, including the raw bytes after the resource.

All253 guards,583 frozen hashes,581 model hashes, seven current implementation
hashes and80 prior files pass. Cleanup retires only the successful interrupted
publication copy. Accepted81-file dataset identity:
`9cc80461861935b25821d297af7a57320dd365bf30f82b07c014a67a53d87326`.
Manifest SHA-256:
`7aaae9f77aae81f95f01b98cffac156ee2f44599fbbdd3369776ddb11f033834`.
JSON SHA-256:
`2983a4918e74524edc4ae3e06744979d8c3b4d08e4c5816b7154ca138e157d6c`.
Private evidence under `target/rom-data-event-records12-runtime/` retains that
dataset, source preimages/snapshot, executable, four native objects and build/
Wine/final preservation/cleanup receipts.

Actual production CLI registration/reuse and one ordinary positive suffix/full
pair now pass below. Full routine ABI/live script-event effects, indirect
references, the preceding opaque gap, next27-byte catalog extent, Linux, USA,
native Windows, new32:9 and the full ROM TODO remain pending. No closed model/source/selection
proof, ordinary checkpoint/navigation or performance/storage measurement is
repeated.


## Actual event-record CLI registration

**ROMDATA-04-EVENT-RECORDS12-CLI — complete for actual EUR registration/startup,
30 October 2026.** One warning-free Windows production build uses the same583
accepted runtime sources without overlays. SHA-256:
`b9a60501ab6ef92011eadd81d79b2cd3720c25be7468ed392ba1ea8d243844bb`.
The canonical executable retains its original hash/mtime after restoration.
Concurrent debug-tool edits remain outside the selected snapshot.

One Wine orchestration runs three actual commands: create e88-d87, reuse it,
and start one neutral frame from a copy of the retained normal frame24755
checkpoint. All exit0. All81 generation files equal the runtime dataset, all80
logged ID/hash pairs match the manifest, and all83 managed paths preserve
hashes/mtimes during reuse/startup. Index79 has the correct ID/format, no
dependencies and one3,072-byte span. The old28,859-block native cache loads
unchanged, without rebuilding a translation.

The guest advances24755→24756 with all11 event-resource counters zero. This
is registration/attachment evidence, not ordinary positive consumption or
scene equivalence. No action/navigation/new checkpoint endpoint or closed
source/model/native/CLI proof is repeated. All194 guards,583 CLI/runtime source
hashes,81 accepted files, ROM/native cache/TinyCC/original seed and canonical
executable pass. Artifact completion retires zero paths. Private evidence under
`target/rom-data-event-records12-cli/` retains preimages, source snapshot,
executable, command logs, no-write/summary/preservation/cleanup receipts and
the e88-d87 generation-3iczRx dataset.

One ordinary positive suffix/full state-clock/image/OAM/audio comparison now
passes below. Full routine ABI/live script-event effects, indirect references,
the preceding opaque gap, next27-byte catalog extent, Linux, USA, native Windows,
new32:9 and the full ROM TODO remain pending. No performance/storage measurement is performed.


## Ordinary event-record mission-entry equivalence

**ROMDATA-04-EVENT-RECORDS12-ORDINARY — complete for one EUR mission-entry case
on Wine, 30 October 2026.** The existing normal checkpoint at guest19409 shows
the Giza Plains menu with White Flowers selected. Its guarded original state
and cached image are reused on a copy; the already-closed selected-mission
producer/event-reader witnesses establish this route. No old700-frame battle
route or source/model/native/CLI preparation proof is replayed.

A passive eleven-counter observer leaf, module declaration and optional
five-line positive-use guard are overlaid onto the accepted583-source CLI
snapshot. Core/runtime sources remain exact; inverse hub/validator edits
restore their preimages byte-for-byte. One584-file Windows build is warning-free:
`0b6581cbf4f09f3da2b10accd63e9c81d7332830865b3172c5abb139c08d8042`.
Concurrent debug-tool work stays outside this snapshot.

One capture uses normal A input for12frames starting at relative1, then neutral,
with a240-frame bound and stop31frames after first source use. It stops after
34frames,19409→19443, with the first read at relative2/guest19412; the rolling
point is relative0, so the accepted suffix is the same34frames. Discovery stops
after this one positive capture. No manual guest RAM/register/task/ID/ROM or
cartridge edit, numeric/RNG forcing or new route search occurs.

All eleven counters agree across capture, the same b9a60501 production CLI and
the same0b6581cb full-pair test executable:

- bus reads4; served source bytes6; key bytes2; primary bytes2; alternate bytes2;
- header/parameter/byte07/flags/opaque bytes0; physical records used1.

The two key reads are counted as service, not a distinct-byte union. This
proves three storage classes in one physical record, not all eight classes,
all256 records, an alias branch or a battle/mission outcome. Other source
coverage retains its targeted proofs. The fullwide284×160 PPM is authenticated
against the complete saved framebuffer; the readonly classic projection shows
the Giza world map during early mission entry.

The actual CLI executes the same34-frame suffix without preparation, rebuilding
or navigation. Its child exits0 and the identity/no-write/frame checks pass.
The local collector then raises a TypeError because it selects the previous
selection-byte resource's usage line. A new offline registration helper checks
the existing log's event-resource ID and all11 counters, then records the result.
The original helper/log are retained; no CLI command or guest frame is repeated
to repair collection. All80 ID/hash pairs and83 managed file hashes/mtimes
remain exact, with the28,859-block old native cache unchanged.

One Wine full pair passes all34 complete serialized states/guest clocks, central
and wide pixels, OAM and five audio streams, with the exact11 counters above.
Both arms use the same old native cache: this verifies data-service equivalence,
not whole interpreter/native parity. Dispatch128240/218562 and audio counts
`[74612, 74612, 8976, 8976, 74488]` are coverage only.

All126 guards,584 frozen source hashes,583 CLI hashes, three current paths,
original seed, ROM/native cache/TinyCC/canonical executable and83 managed files
remain exact. Artifact completion retires zero paths. Private evidence under
`target/rom-data-event-records12-ordinary/` retains guarded seed copies, normal
input, accepted34-frame checkpoint/suffix, endpoint images/projection receipt,
source preimages/snapshot, executable, original collector failure, offline
registration and CLI/pair/final preservation/cleanup receipts.

This closes the declared event-record source/model/managed/runtime/actual CLI
chain and one ordinary case. Wider ordinary fields/records/alias use, full
routine ABI/live script-event effects, indirect references, the preceding
opaque gap, next27-byte catalog extent, Linux, USA, native Windows, new32:9
and the full ROM TODO remain pending. No performance/storage campaign,
Android/package change, assistant commit or push occurs.


## Observed European parameter records with 27-byte stride

**ROMDATA-01-PARAMETER-RECORDS27 — observed source fragments complete,
30 October 2026; whole catalog extent remains unknown.** The authenticated
256 event records supply exactly59 distinct `parameter06` indices:0..50 and
115..122. Original getters multiply this unsigned byte by27 and add
`0x0871D360`. These selected physical records form two source runs,
`0x71D360..0x71D8C1` and `0x71DF81..0x71E059`, containing1593 bytes in total.
Their ordered concatenation has SHA256
`40d4e406d08f42caaa9c87da76a143166a185656d7e8bf987828c562bd3edf75`.
The proposed neutral resource is `gameplay.parameter-records27`.

The59 indices describe the observed fragment's ownership. Separate readers
use a raw unsigned live byte at view offset`0xA8` and compute the same address
without checking127,128 or59. Other indices must retain raw reads until their
sources are classified. The independent next root`0x71E0E0` uses stride38.
Its distance from the27-byte root is insufficient to prove128 records, so that
candidate count is rejected. In particular, the holes and trailing zeros are
neither padding nor implicitly owned records.

Byte00 is an unsigned header. Bytes01..06 are retained as individual packed
bytes, including shared nibbles: original readers compose
`(byte01 >> 5) | ((byte02 & 31) << 3)`,
`(byte03 >> 5) | ((byte04 & 15) << 3)`, and the adjacent nibble pairs04/05 and
05/06. Bits0 and1 of byte01 and masks`0x30`/`0xC0` of byte0D have separate
consumers. All other bits remain intact; no enum is inferred from current values.

Offsets07,0A and12 are unsigned little-endian16-bit values assembled from byte
loads. Bytes0C and15 are explicitly read with `LDRSB`:0C can be multiplied
by200, while15 is subsequently encoded in the low16 bits of an argument.
Byte09 is unsigned and has a separate scale-by200 path. Bytes0E/0F and10/11
are separate operands: a marker`0x8000` is ORed into the even byte's operand,
while the odd byte is passed independently. They are not declared pointers or
combined halfwords. Byte14 and bytes16..1A remain opaque and lossless.

All1593 bytes roundtrip, all256 event links resolve into the observed fragment,
and neither source run overlaps the80 accepted e88-d87 resources/50495 source
spans. Twenty-seven static windows(23 new/four cached),70 instruction/literal
pins and125 hash/mtime guards authenticate the source contract. All25 exact
root literals in the cached aligned-word census are covered; that census does
not establish the absence of indirect or unaligned aliases. Shared packed
views retain one owner. Repeated values and records retain their physical indices.
No build, Wine run, game input, checkpoint creation, old acceptance replay or
performance measurement was needed for this source-only unit.

The strict model, original-prefix Wine proof, managed publication/runtime,
actual CLI and ordinary use are pending. Full catalog extent, the unowned
indices, helper implementations, whole-routine ABI/live effects, indirect
references, Linux, USA, native Windows, new32:9 and the full ROM TODO also remain
open. The completed event12 chain remains closed.


## Sparse parameter-record model and original-prefix Wine proof

**ROMDATA-02-PARAMETER-RECORDS27 complete for the59 observed EUR records,
30 October 2026.** `gba_core::rom_data::parameter_records27` exposes a strict
`ParameterRecords27` model with ordered physical indices0..50 and115..122,
individual unsigned/packed fields, three little-endian16-bit values, two signed
bytes and exact opaque bytes. Shape validation allows edited field values;
`verify_source` separately requires the authenticated EUR ROM and source data.
Unknown/missing JSON fields, invalid integer types/ranges, wrong opaque lengths,
incorrect source root and missing/reordered/duplicated/unobserved indices fail.

`record` and `byte_at` answer only for the two observed runs. Neither fills the
unowned gap nor truncates a high physical index to a byte. `encoded_view`
separately preserves the original low8-bit address computation for every input,
including indices51,114,123,127,128,255 and wrapping high arguments. This
separation adds no guest bound. The model emits two spans totaling1593 bytes;
shared packed projections read the same underlying fields.

One warning-free Windows core build from586 frozen files(accepted584-file
ordinary snapshot plus two new leaves) produced executable SHA256
`49c72f02f144924f84e952a318e7c4121b5d1aefda608c825a8fc96295156dcd`.
One Wine filter passed25 validation guards and73 original-ROM/48 isolated
synthetic prefix cases. An independent Python oracle supplies the decoded
model, source spans, results and exact read order. Original unsigned, packed,
nibble-pair, unsigned16, signed/scaled, marker-operand and computed-view
instructions match, including signed127/-128 and negative low16 encoding.
Every RAM region checked(EWRAM/IWRAM/VRAM/OAM/palette), SP, R8..R12 and final
ROM matches its expected preimage. These prefixes stop before external helpers,
prologues/epilogues and live continuation.

The tests observe95 distinct owned bytes from the base cases and20 from
synthetic cases; the separate roundtrip checks all1593 bytes. The739 executed
instructions are coverage only. They do not establish whole-routine ABI,
native/interpreter guest-clock parity or ordinary scene equivalence. No game
checkpoint or normal input was created. Preservation verifies143 hash/mtime
guards,586 frozen source hashes,251 core inputs,584 accepted preimages,
125 source-contract guards and the inverse module edit. Artifact retention
removes no files. Parallel DBG/menu/script/input changes remain excluded.

Managed publication/runtime, actual CLI and positive ordinary use remain
pending at schema1/e88-d87. The full catalog extent, unowned indices, external
helpers, indirect references, Linux, USA, native Windows, new32:9 and the full
ROM TODO remain open. No performance or storage campaign was run.


## Managed sparse parameter records and typed runtime reads

**ROMDATA-03-04-PARAMETER-RECORDS27-RUNTIME complete for the59 observed EUR
records, 30 October 2026.** Schema1/e89-d88 adds `parameter-records27.json`,
resource `gameplay.parameter-records27`, format `parameter-records27-v1-json`
at manifest index80. The dataset contains82 files/81 resources; all80 earlier
resource records and payloads remain exact. Two source spans own1593 bytes in
total. The file is self-contained and declares no dependencies.

The immutable owner composes with prior event records on page71D and covers
the second observed run on page71E. Unclassified indices/gaps and the next
independent root retain their actual raw bytes, including mixed-width reads
across each border. Packed projections share the same physical storage.
Twelve disjoint storage classes expose15 counters: bus reads, served bytes,
header, packed bytes01..06, value07, byte09, value0A, signed0C, flags0D,
operands0E/0F, operands10/11, value12, signed15, opaque bytes and records used.
Four mask words retain physical indices, with59 observed positions. This adds
no guest index, allocation or file-count cap and changes no Machine/save/native
cache/C API ABI. Dataset parse/binding remains transactional and authenticated.

The reduced Wine validation combines two scopes. The first warning-free588-file
build(`51e19ef0..`) passes publication, all80 prior payloads, interrupted-file
publication,14 failed-attach/two full-set guards, and the first17 native
full-state/clock cases. Its test oracle then incorrectly expects two prior-owner
bytes for an aligned32-bit load that actually serves four. Typed/native/API
counts all agree on four. This failure and its initial source snapshot,
executable, objects and observations are retained.

Only the test leaf changes: the prior-owner predicate now covers all3072 event
bytes, and a remaining mode selects the16 unfinished native cases plus bus/
lifecycle tests. The second warning-free588-file build has SHA256
`278d6d23bda33d888462e1fc8c28df39a0ab0e0ca760687586e8134cb88b852e`. One remaining-only Wine filter passes,
reusing the same82 payloads and13 native objects with exact hashes/mtimes.
Publication, attach guards and the first17 native cases are not repeated.
The combined result is33 complete-state/clock cases(20 controlled loads and13
original arithmetic/load prefixes), with all callback deltas accounted.
They serve37 new bytes and five prior event bytes; these are served totals,
not distinct-byte coverage. The remaining16 cases report zero source bytes
through `interp_one`; no fallback breakdown exists for the first17 cases, so
this is not an all-direct-native claim. Remaining75 visited instructions and16
dispatches are coverage only.

The independent bus checks cover all1593 bytes, all12 classes/59 records and
59 record borders, mirrors/widths, deliberately changed raw gap/next-root bytes,
prior-owner composition, failed attachment, unavailable files and Load/slot/Undo
owner retention. DMA/CpuSet1664 halfwords and CpuFastSet832 words match complete
state and guest clock across both sparse runs and their raw gap. No ordinary
checkpoint or game input was created. Preservation verifies291 hash/mtime
guards, the accepted and diagnostic588-file snapshots,252 core inputs,
586 model preimages, seven current implementation paths and81 prior files.
The artifact helper retires only the disposable interrupted publication copy.
An inherited uppercase task label in a private collector is corrected offline;
no build or test is repeated for metadata. `accepted-selection.json` identifies
the accepted snapshot; the original `selection.json` remains diagnostic.

Dataset SHA256:`13c76015188dc8a80c10d060d650d0d9ec2ae1f06c508140958b3c65713fd61b`.
Manifest SHA256:`1b523648ad9d0fd9aa72fb7668053504356bf8b419445c1983b1bbc76e93359f`.
JSON SHA256:`fc94e9aa59110f2ae0f72e3a7351f14e8c461711f41f3a16e81c24c239549a48`.
Private evidence: `target/rom-data-parameter-records27-runtime/`.

Actual production CLI preparation/reuse/startup and positive ordinary use
remain pending. Whole catalog extent, unowned indices, full routine ABI/live
helpers/effects, indirect references, Linux, USA, native Windows, new32:9 and the
full ROM TODO remain open. No performance or storage campaign was run.


## Actual sparse parameter-record CLI registration

**ROMDATA-04-PARAMETER-RECORDS27-CLI complete for creation/reuse/startup,
30 October 2026.** One warning-free Windows production build uses the same588
accepted runtime source files with no overlays, including the corrected test
leaf that is excluded from production. Executable SHA256:
`bd99ac5be727adf1153a28888bb1097d41a504c4041132031fd0bcf6b4c663c8`.
The canonical executable is restored in a `finally` block with exact hash/mtime.
Parallel DBG/menu/script/input changes remain outside the frozen source set.

One Wine orchestration runs the real CLI three times: `rom-data prepare`
creates the e89-d88 generation, a second preparation reuses it, and `runc`
loads a retained normal frame24755 checkpoint copy with neutral input for one
frame. All three processes exit0. The82 generated payloads match the accepted
runtime dataset byte for byte; all81 logged resource ID/hash pairs match the
manifest, including index80/two source spans/1593 bytes and no dependencies.
All84 managed paths retain exact hashes/mtimes during reuse and startup, and
the unchanged native cache loads28859 translated blocks.

The startup reaches frame24756 with all15 parameter-record counters zero.
This proves registration and compatibility, not positive ordinary consumption
or scene equivalence. No game navigation, input action or new checkpoint is
created. Preservation checks197 guards,588 CLI source hashes and588 runtime
source hashes, the ROM/original seed/native cache/TinyCC/canonical executable
and all82 accepted files. Artifact retention removes no paths. No source/model/
runtime/older ordinary proof or performance campaign is repeated.
Private evidence: `target/rom-data-parameter-records27-cli/`.

A positive ordinary suffix/full state-clock/image/OAM/audio pair remains
pending, as do whole catalog extent, unowned indices, full routine ABI/live
helpers/effects, indirect references, Linux, USA, native Windows, new32:9 and
the full ROM TODO.


## Ordinary sparse parameter-record reachability remains partial

**ROMDATA-04-PARAMETER-RECORDS27-ORDINARY remains open, 30 October 2026.**
A passive observer reports all15 counters and records normal input/checkpoints
without modifying guest RAM/registers/ROM/task IDs or RNG. A five-line optional
positive-use guard in the existing scene validator requires source reads,
physical record use and the sum of all12 storage classes. The observer module
and guard remove exactly to their588-file CLI preimages. One warning-free
589-file Windows test build has SHA256
`8f0c02b7b9010247d0db4e4e69758a8e45841a122dc3c1370f0354f18344920b`;
core/runtime sources remain identical to the accepted CLI snapshot.

The first two Wine captures on authorized copies complete with all15 counters zero:
normal R12 from the existing Giza mission-info page1/3 checkpoint runs180 frames,
15050→15230; the endpoint still shows page1/3. Normal A12 at relative1/161 from
the retained visible White Flowers opening-dialogue checkpoint runs240 frames,
21889→22129; the endpoint shows the ensuing camera scene. These are closed
negative bounds, not positive fixtures or equivalence acceptance. No positive
suffix, positive CLI or full scene pair is run. Both sets of checkpoints,
images, inputs and logs remain diagnostic evidence.

Nine bounded static windows identify the header getter `D2588`, its text
formatter/task callers, constructor `12C280` and script callback `127324`.
The latter passes instruction byte1 as a raw mode. The task copies that mode
to its byte06/live formatter byteB2 before calling the getter. Seven selected
old checkpoints have formatter gateB4=0, which is only a read-only gate
observation, not an execution claim.

A new read-only join of the accepted command directory, script archive and
event records identifies opcode80/advance2/callback127324. Five direct mode2
instructions belong to programs53/65/74/93/95 and keys56/68/77/96/98; the
matching event rows are216..220 and parameter indices116..120. White Flowers
uses event58/parameter0/program163. Further guesses in that mission-info or
intro route are stopped. This direct join does not rule out indirect script
links or mode0 reads when the live override is zero.

Nine further normal captures reuse the same observer without rebuilding. A180
from the authenticated IN13 Sprohm checkpoint opens the town information menu
(3020→3200), B32 closes it, and five source-guided cursor steps reach the
visible Nubswood node. A360 travels there (3448→3808), ending at the visible
The Cheetahs menu. A240 enters the Cyril Pub from a retained town-menu copy
(3315→3555), and A120 advances its entry (3555→3675). All15 source counters
remain zero in all eleven captures. No mission ID is inferred from the visible
label, no further encounter is selected and no RNG resampling is attempted.

Six authenticated existing EUR bank snapshots expose nonzero event/live-view
parameter candidates but no formatter gate. These read-only values are not
reachability proof. Cached original node geometry corrects the initial movement
estimates; the source hitbox is inclusive and the actual endpoint label is
checked before travel. An additional80-byte initializer window authenticates
40 original halfwords without proving which displayed mission is active.
The initial broad RAM entity projection is unproved outside its source-supported
entries and is not used as a clan census or an event selector.

The Pub endpoint provides no readable choices. One additional neutral Wine
frame uses the existing production CLI with host dialogue layout disabled only
for render inspection; exit0, no resulting state is saved or adopted. Read-only
classic/wide projection authenticates the complete saved framebuffer, and a
source-ordered projection of the saved76800-pixel native UI layer shows only
the Gil counter. Neither diagnostic establishes a menu choice, positive use or
equivalence. Blind Pub/world navigation is stopped.

Preservation checks233 hash/mtime guards,589 observer source hashes,588 CLI
source hashes and the three current observer/module/validator paths, including
84 managed paths and the original ROM/seeds/native cache/canonical executable.
Initial two-capture receipts remain retained alongside the updated eleven-capture
summary. The partial unit retains all diagnostics; no completion cleanup is
requested. Private evidence: `target/rom-data-parameter-records27-ordinary/`.

Resume this ordinary criterion only with a concrete normal scene and a source-
specific trigger, reusing this observer and the existing bd99ac5b CLI/e89
dataset. The eleven captures and render diagnostics are not rerun to reconstruct
context. Independent source contracts may progress while this criterion is open.
Source-positive ordinary use and full state/clock/image/OAM/audio equivalence
remain required. Linux, USA, native Windows, new32:9, whole catalog extent and
the full ROM TODO remain pending. No performance campaign or forced fixture
was used.


## Observed European world records with 38-byte stride

**ROMDATA-01-WORLD-RECORDS38 — declared observed source catalog complete,
30 October 2026.** All512 authenticated mission records supply exactly31
six-bit indices0..30 through `(packed02[0] >> 6) | ((packed02[1] & 15) << 2)`.
Their source root is`0x71E0E0`, stride38, with1178 bytes ending at`0x71E57A`.
The complete byte roundtrip has SHA256
`cd2b33e95a5b46a53703f8a32f89a6f1c0eea4d1b562dfd86ee7c4836084d483`.
The proposed neutral resource is`gameplay.world-records38`.

The initializer`D347C` takes the argument's low8 bits, multiplies by38 and
adds the root. A separate selector at`D4C4C` retains its full32-bit R2 argument
and computes the wrapping address without truncation. Neither introduces a31
cap. The accepted source catalog and the original encoded address domain stay
distinct; unclassified outside reads remain raw. The two inspected flag-range
callers map flags1415..1438 to physical indices7..30. Their full routine ABI and
ordinary execution are not established by this static join.

An interior literal`0x71E106` points at physical row1 of this same owner.
The bounded updater`D3508` iterates ordinals6..29, selecting physical rows7..30;
it does not introduce a second overlapping resource. A distinct following root
at`0x71E57C` indexes32-bit pointers. The intervening two zero bytes remain
unowned; neither their value nor the distance between roots proves padding or
an additional record. The512 mission links and bounded updater provide the
extent evidence, not distance alone.

Byte01 is unsigned. Its initializer uses an external helper with`byte01 >> 1`,
adds the original byte, keeps the low8 result and replaces zero with1. This
records the observed projection without replacing RNG or helper behavior.
Bytes02/03 compose a little-endian16-bit operand, while byte02 also has an
independent unsigned consumer. Bytes04/05 retain their complete16-bit value;
the observed copy writes only its low10 bits. Bytes07..25 form31 unsigned
thresholds, indexed by a live count explicitly clamped to0..30; the original
branch compares the signed external result against`100 - threshold`. No100
cap is imposed on stored thresholds. Bytes00 and06 remain opaque, with all
zeros, FF values and unused bits preserved and no enum or pointer invented.

Static verification authenticates12 windows (ten new, two cached),58 explicit
instruction/literal pins and107 hash/mtime guards. All1178 bytes roundtrip,
all512 links resolve and the span overlaps none of the81 accepted e89-d88
owners/50497 source spans. All two exact-root literals and the interior literal
in the cached aligned census are covered; indirect/unaligned references are
not exhaustively excluded. The existing82-file managed dataset remains exact.
No build, Wine run, gameplay input, checkpoint creation, earlier acceptance
replay or performance/space measurement occurs in this source unit.
Private evidence:`target/rom-data-world-records38/`.

A strict model/original-prefix Wine proof, managed publication/runtime, actual
CLI and ordinary use remain pending. Opaque semantics, outside/indirect readers,
external helpers/RNG, full routine ABI/live effects, Linux, USA, native Windows,
new32:9 and the full ROM TODO remain open. Sparse27 ordinary reachability stays
partial independently; its negative navigation bounds are not reopened.


## World-record model and original-prefix Wine proof

**ROMDATA-02-WORLD-RECORDS38 complete for the declared EUR catalog,
30 October 2026.** The strict lossless model retains31 ordered records, one
1178-byte source span and all opaque bits. Unknown/missing JSON fields, numeric
width violations, malformed threshold arrays and reordered/missing/extra rows
are rejected. Shape validation permits changed field values, including raw255
thresholds; exact source verification independently requires the authenticated
16MiB EUR ROM,12 reader windows and the full source digest.

Physical`record`/`byte_at` ownership ends at`0x71E57A`, leaving the two-byte gap
and next root raw. Separate encoded views reproduce the initializer's low8
argument, the selector's full32-bit wrapping arithmetic and the updater's
interior-row1 address. None caps the guest address to31 records. Model methods
retain byte02 both separately and within the LE16 operand, all16 bits of
value04 with its low10 copy projection, the byte01 divisor and post-helper
wrap/zero-to-one arithmetic, and the original live-count clamp/signed threshold
comparison. External RNG and helper execution remain guest behavior.

One warning-free590-file Windows core-test build has SHA256
`ed4d65c8df45523e5d5c66362427a2222e7664c50d8797ffdc775a1f6ba42029`.
Its single reduced Wine filter passes19 parser/source guards,41 base and19
isolated synthetic original prefixes. These execute actual unsigned loads,
LE16 composition, low10 destination writes, count clamp and signed branches,
post-helper wrapping/zero handling and all three address views. Synthetic cases
cover zero/one/128/255 byte01, helper result wrap, high16-bit storage, raw255
thresholds with negative/equal comparison edges, count31/255 and u32 address
overflow. No ordinary checkpoint, task ID, RNG or game input is forced.

All1178 source bytes roundtrip. Actual original reads cover23 distinct base
bytes and six distinct synthetic bytes; these are separate coverage sets,
not a claim that every source byte executes. The353 instructions are coverage
only. Results and load order, complete EWRAM/IWRAM/VRAM/OAM/palette, expected
isolated destination writes, SP/R8..R12 and final ROM are exact. Register
checks account for the selected prefix's intentional R9 assignment. This is
not a full routine ABI, cycle/native, helper/RNG or ordinary scene proof.

Preservation checks26 unit hash/mtime guards,107 closed-source guards,590
frozen build sources/254 core inputs,588 accepted CLI preimages and three
current implementation paths. Removing the new module declaration restores
the exact prior core hub; unrelated frontend edits are excluded. Artifact
retention removes no paths. Private evidence:
`target/rom-data-world-records38-model/`.

Schema1/e89-d88 and the accepted82-file managed dataset remain unchanged.
Managed publication/typed runtime, native full state-clock/lifecycle/DMA, actual
CLI and ordinary use are separate pending criteria. Opaque/outside/indirect
semantics, full helpers/RNG/live effects, Linux, USA, native Windows, new32:9 and
the full ROM TODO remain open. No completed proof, navigation or performance
campaign was repeated.


## Managed world records and typed runtime reads

**ROMDATA-03-04-WORLD-RECORDS38 complete for EUR/Wine,30 October 2026.**
Revision e90-d89/schema1 adds`gameplay.world-records38` at index81 in
`world-records38.json` (`world-records38-v1-json`, no dependencies). The managed
dataset has83 files/82 resources; all81 prior resource records, coverage entries
and payloads remain exact. Its single1178-byte source span is the declared
31-record catalog, not a new bound on original guest addresses.

An immutable typed owner on page71E composes the existing sparse27 owner,
the unowned preceding gap, its own records, the following two-byte gap and the
independent next root. Storage counters partition byte01, bytes02/03, bytes04/05,
31 threshold bytes and opaque00/06. Eight counters and one physical-record mask
observe served bytes/31 slots without reinterpreting unknown fields. Original
live arithmetic, RNG/helper calls and outside reads remain guest behavior.
No Machine/save-state/native-cache/C API ABI or allocation cap is introduced.

One warning-free592-file Windows host-test build has SHA256`271dcebaea48bd973d492534e9623b4f9a2efc8f02b1d64335c3ab560766a995`.
The compiler child completed successfully; the wrapper's final guard caught a
private runner start-label edit made while that wrapper held its old selection.
Offline registration rechecked the frozen/current sources and inputs, then
adopted the same executable. No compilation or accepted test was repeated.

One reduced Wine filter passes83-file publication, prior81 equality, interrupted
new-file publication,14 forged/missing attachment cases and two complete-set
guards. Failed attachment retains the owner and complete machine state.
Twenty-seven complete-state/clock comparisons use17 controlled original loads
and ten original prefixes in ten stop-safe native families. All actual read8/
16/32 and real interpreter-fallback callbacks are accounted. Native callbacks
serve23 new bytes and5 prior parameter bytes, including fallback
counts[0, 0]; these are served counts, not distinct-byte unions or a claim
that every callback is a direct native load. Visited instructions and dispatch
counts are coverage only. All1178 new raw source bytes and five prior bytes are
poisoned only after isolated Load/setup, then restored before state comparison.

Bus checks cover all1178 bytes/five classes/31 records, all31 record ends,
mirrors/widths and prior/raw-gap/next-root composition. Load/slot Undo retains
the owner while files are unavailable. DMA, CpuSet704 halfwords and
CpuFastSet352 words match complete state/clock across both owners and raw gaps.
These are isolated probes; no ordinary checkpoint, task ID or RNG is forced.

Preservation checks269 guards,592 frozen files/255 core inputs,
590 model-source hashes, seven current implementation paths and82 prior files.
The current sparse27 observer module is preserved; only the frozen host overlay
excludes it and retains the accepted validator preimage. Parallel DBG/menu work
is preserved and excluded. Artifact retention retires only the disposable
interrupted copy, retaining the accepted dataset/logs/source preimages/objects.
Private evidence:`target/rom-data-world-records38-runtime/`.

Actual production CLI creation/reuse/startup and positive ordinary scene
equivalence remain separate pending criteria. Full routine ABI/helpers/RNG/live
effects, opaque/outside/indirect semantics, Linux, USA, native Windows, new32:9
and the full ROM TODO remain open. Sparse27 ordinary use stays partial. No old
source/model/native/ordinary proof, route or performance campaign is repeated.


## Actual world-record CLI registration

**ROMDATA-04-WORLD-RECORDS38-CLI — registration/startup complete,
30 October 2026.** One warning-free592-file production build has SHA256
`6d95f7050cbddf6e8143961f81f66928224e44e8c6c0529f9af4538d76374f7c`. It uses the exact accepted runtime snapshot without overlays; the
canonical executable's original hash and mtime are restored in a finally block.

One Wine orchestration runs three actual CLI commands: prepare creates an
e90-d89 generation, prepare reuses it, and runc resumes an authorized copied
checkpoint at24755 with one neutral frame, ending24756. All children exit0.
All83 generation files equal the accepted runtime dataset; all82 logged
resource ID/hash pairs match the manifest, and all85 managed paths preserve
hash/mtime during reuse and startup. Index81 binds`gameplay.world-records38`,
the expected format/no dependencies and one1178-byte source span. The old
native cache loads28859 blocks without modification.

All eight new counters are zero in this startup. This proves registration and
compatibility, not positive ordinary consumption or scene equivalence. No game
navigation, input action, new recording or saved endpoint is created.
Preservation checks201 guards,592 CLI and592 runtime source hashes, the
ROM/original seed/native cache/TinyCC/canonical executable and83 accepted files.
Artifact retention removes no paths. No closed source/model/runtime/older CLI
or ordinary proof or performance campaign is repeated.
Private evidence:`target/rom-data-world-records38-cli/`.

Positive ordinary use with a selected suffix/CLI/full scene pair remains open,
as do full routine ABI/helpers/RNG/live effects, opaque/outside/indirect
semantics, Linux, USA, native Windows, new32:9 and the full ROM TODO. Sparse27
ordinary reachability remains partial independently.


## Ordinary world-record counter-expiration equivalence

**ROMDATA-04-WORLD-RECORDS38-ORDINARY complete for one EUR scene,
30 October 2026.** A passive eight-counter observer and optional five-line
positive-use guard use one warning-free593-source Windows host-test build,
SHA256`ee93928887dcac3a02a7682678c4e0753c7d4492519a2bdce93a04cedb51103e`.
Core/runtime match the592-file production CLI exactly. Removing the new module
and guard restores their preimages; the existing sparse27 observer/guard remain
in the current tree and are excluded only from this frozen build overlay.

The first normal A12 travel from the retained Nubswood-selection copy runs
360 frames,3448→3808, with all eight counters zero. This bound is closed,
not a positive fixture. Read-only source/snapshot checks explain the relevant
branch: `CCF00` reads a live byte at`EWRAM2030 + low16(counter index)`.
The only enabled row in this context has its counter fall32→30, remaining
outside the source-read branch. The seven source-supported path slots0..6
become FF at the endpoint. These RAM observations support input selection;
they are not direct PC execution proof or a broad entity/clan census.

Four selected existing EUR world-bank snapshots are authenticated. IN08 has
an enabled physical13 live row with counter1, making its threshold branch a
candidate after two original daily updates. A copied IN08 world checkpoint
starts at3020 with Aisenfield selected. One normal Down48/96-frame cursor
capture ends3116 at the actual placed node18, beyond intended Cyril; the
source graph has at least three edges from player node25 to that destination.
The movement estimate is corrected from the observed coordinates, and the
actual destination is used without replaying navigation. No town/mission
identity is inferred from the omitted label in the world PPM.

One subsequent normal A12 capture stops after189 frames,3116→3305. The first
new source read occurs at relative157/guest3274; the rolling checkpoint yields
a45-frame suffix3260→3305. Discovery stops immediately after the required31
following frames. No additional travel, RNG resampling, manual RAM/register/
ROM/task/ID editing or old route reconstruction occurs. The new trigger is
live-counter expiration, not a claim that any world travel reads this catalog.
Bounded static inspection reuses three cached windows and authenticates the
new entry/gate/counter windows; the gate window overlaps an already cached
caller and is diagnostic inspection, not a replay of its old acceptance.

The SAME6d95f705 production CLI runs that suffix once on the accepted e90-d89
generation. All82 logged resource ID/hash pairs match;85 managed paths and the
old28859-block native cache retain hash/mtime. The SAME593-source test binary
runs one raw-ROM/managed pair for all45 frames. Every complete serialized
state/clock, classic/central-wide pixels, OAM and all five audio streams match.
Capture, actual CLI and pair report exactly two bus reads/two served bytes:
one threshold byte and one byte01, one physical record, other three classes
zero. These are served counts; the counters do not identify the record's
index or prove all five classes/31 records execute in ordinary gameplay.

The authenticated framebuffer projection matches the complete284×160 wide PPM
with bit-replicated RGB555 conversion. Its classic image shows ordinary world
travel/Madmoon Day4, without proving arrival, a mission choice or outcome.
Dispatch261766 native/702117 fallback and audio sample counts
[98754,98754,11880,11880,98670] are coverage only. Both pair arms use the same
old native cache; this is not global interpreter-versus-native parity.

Preservation checks165 guards,593 observer/592 CLI source hashes, three current
implementation paths/inverse hubs and all85 managed paths, original seeds,
inputs, ROM/cache/TinyCC/canonical executable. Successful-unit retention removes
no paths; diagnostics for the earlier negative bound remain retained.
Private evidence:`target/rom-data-world-records38-ordinary/`.
Other ordinary records/classes, full routine ABI/helpers/RNG/live effects,
opaque/outside/indirect semantics, Linux, USA, native Windows, new32:9 and the
full ROM TODO remain open. Sparse27 ordinary reachability stays partial.
No performance or space campaign is run.

## European conditional four-byte lookup projection

The independently rooted table at `6e6dbc` is read with four-byte indexing;
the earlier hypothesis that it belonged to the 14-byte records remains
rejected. Original readers reconstruct two little-endian 16-bit fields from
each row. Eleven aligned root literals were found; other indirect references
and the semantic names of the fields remain unproved.

The metadata52 search at `0cce28` enumerates rows 0–115. For suitable original
search inputs, property 12 supplies each nonzero index 1–73 to a list consumed
by the reader at `080300`. The authenticated metadata has four `ff` routing
rows: a same-row redirect can fail to return and must not become a successful
fallback. A separate conditional original path can clear the selected object's
byte `36`; its reader accesses row 0 without a zero guard. This is source
reasoning under the stated conditions, not an observed ordinary refresh.

These witnesses support a bounded 74-row, 296-byte source projection at
`6e6dbc..6e6ee4`, SHA256
`76c83105709e84c7852de2d88b19c7b50b7862b443b57a478de4854ebb3a0ea1`,
without overlap against the accepted e95 manifest. They do not establish a
global index bound: other original readers take live values, and reads at 74
or above retain their original raw behavior. Exact EUR ROM, source windows,
metadata and manifest checks are retained in `target/rom-data-6e6dbc-lookup/`.
The source milestone itself added no runtime owner. The later model and managed
publication are documented below. Ordinary use and Linux/USA checks remain
open. No gameplay or performance measurement was performed.

## Typed conditional four-byte lookup model

The EUR source projection has a strict model of 74 rows with two independent
`u16` fields. It reconstructs all 296 original bytes and authenticates both the
exact ROM and the source digest. The encoded-address helper retains original
32-bit wrapping arithmetic; looking up an index outside the declared projection
returns no modeled row and does not add a guest-side check.

One isolated Windows build and focused Wine test passed source and JSON checks,
malformed/coherent-edit rejection, and original field and byte-reader prefixes
for indices 0, 1, 73, 74, 255 and `u32::MAX`. The original metadata52 getter
also returned 73 for row 77/property 12. Protected inputs and frozen sources
remained unchanged; evidence is in `target/rom-data-6e6dbc-model/`.

This model milestone was followed by managed publication below; it is not an
ordinary gameplay proof. Linux, USA and native Windows remain open.


## Managed conditional four-byte lookup

The EUR e96-d95 dataset adds `gameplay.conditional-four-byte-lookup`, with
74 authenticated two-field records and one 296-byte source binding. Strict
activation checks the decoded model against the exact original source, even
when an edited file and its manifest hash agree. Runtime reads reconstruct
source bytes from the typed fields, composing at span boundaries with earlier
owners and the original ROM. The original selectors and arithmetic still run;
indices beyond this conditional projection are not capped. The resource
reports read and per-field byte counters.

One isolated Windows build and focused Wine test passed: all 85 previous
resource records and files remained identical; poisoned source bytes, mirrors
and mixed-width span edges read correctly; coherent tampering was rejected
without replacing the existing owner or state. Three translated original
prefixes were compared with the raw interpreter at indices 0, 1, 73, 74, 255
and `u32::MAX`: complete state and clock matched, with 15 in-projection native
bytes served and none for the outside cases. Protected inputs and frozen sources
were unchanged. Evidence is in `target/rom-data-6e6dbc-runtime-retry/`; the
initial failed build and budget-test correction are retained separately in
`target/rom-data-6e6dbc-runtime/`. This is not ordinary gameplay or full caller
ABI coverage. Production CLI, ordinary use, Linux, USA and native Windows
remain to verify. No performance measurements were made.


## Conditional lookup production CLI

An isolated production Windows binary under Wine created and reused the EUR
e96-d95 generation, then advanced one frame from an existing neutral input and
state. All 87 generation files matched the accepted managed test dataset, all
86 runtime resource IDs and hashes matched the manifest, and 89 managed files
remained unchanged during reuse and startup. The ROM, input, state, native cache,
canonical executable and frozen build sources were protected and unchanged.
Evidence is in `target/rom-data-6e6dbc-cli/`. This establishes registration and
startup, not ordinary use of the lookup or full gameplay equivalence. Linux,
USA and native Windows remain to verify; no performance test was performed.


## Conditional lookup ordinary-route candidate

The existing normal-input EUR Ability navigation is a source-supported
candidate, not an observed lookup read. The retained frame-4519 state and input
reach an Ability screen whose saved screenshot shows Item as the first category.
Original code copies the four-byte map `01 02 03 05`, uses the selected category
in state 0 and calls the list builder with the mapped value; its value-1 branch
consumes indices through the lookup at `08080306` and `08080316`. Entry into
that state, a nonempty list and actual reads remain to be observed. The existing
input's A press at frame 500 and release at 512 justify one bounded 540-frame
observation. Source check/disassembly are retained under
`target/rom-data-6e6dbc-ordinary-route/`, with state/input provenance under
`target/rom-data-linked-metadata28-cli/`. The old metadata28 negative concerns
a different resource; neither it nor the zero-read one-frame CLI startup proves
that this lookup is unused. No gameplay replay or performance measurement was
performed for this source selection.


## Ordinary conditional lookup use in Ability navigation

One Wine run of the existing normal-input Ability route, frame 4519 through
5059, reported 55 typed bus reads and 55 source bytes: 27 from the first field
and 28 from the second. This is positive aggregate use of the EUR projection;
it does not identify each instruction or establish all gameplay paths. The run
used the e96 production runtime and existing native cache; 199 guarded files and
691 frozen source hashes were unchanged. Evidence is in
`target/rom-data-6e6dbc-ordinary/`. The full same-input raw/typed scene pair,
Linux, USA and native Windows remain to verify; no performance was measured.


## Ordinary Ability scene differential

The same existing 540-frame Ability route passed a Wine differential between
raw ROM and the e96 managed runtime. At every frame the complete state and
clock, central and wide video, OAM, dispatch counts and five audio outputs
matched. An opt-in assertion inside this same comparison recorded 55 typed
lookup reads: 27 first-field bytes and 28 second-field bytes. The 108 guarded
files and 691 frozen source hashes stayed unchanged. Both arms shared the
existing native cache; this does not cover every scene or establish general
interpreter/native equivalence. Evidence is in
`target/rom-data-6e6dbc-ordinary-pair/`. Linux, USA and native Windows remain
unverified; no performance measurements were taken.


## European conditional packed four-byte projection

Three original readers use root `086e6ee4` and four-byte indexing, with the
index taken from `u16(descriptor+4)`. The original `080d0ed4` getter selects
an eight-byte descriptor from the pointer table at `086df4d4` using two live
`u8` inputs. Reader `08134168` checks `descriptor[6] == 5`, then reads the low
five bits of row byte 1. Another reader combines the upper three bits of byte 1
with the low two bits of byte 2; a third reads byte 3. Unused bits and byte 0
remain structural until their meanings are established.

The UI category-3 path maps to type 5, builds a list filtered by descriptor
type and can assign the selected descriptor to the unit. Availability, unit
count/progress/equipment and a valid confirmation gate the actual selections.
Within 23 consecutive source pointer intervals, 42 type-5 descriptors provide
static witnesses covering indices 1–33. This supports the conditional 33-row
projection `6e6ee8..6e6f6c` (132 bytes, SHA-256
`5f8c27c41576800b3eded5d5c7e57c7ceb10f5a1ca9e31d8c07fb72c19cfcae9`),
which has no e96 owner overlap. It does not prove every witness ordinarily
reachable or establish a global bound; the final pointer interval is unbounded.
There is no source witness for row 0 in the examined intervals. Other descriptor
types can supply indices above 33: one static type-1 index 251 points into the
existing metadata14 owner. The two readers lacking a type guard preserve their
original address arithmetic and any prior owner. No new fallback is implied.

Exact EUR ROM, scoped source census, per-index witnesses and original reader,
producer and assignment disassemblies are retained under
`target/rom-data-6e6ee4-root/`. This is a source classification only, with no
new runtime owner or ordinary-use proof. Linux, USA and native Windows remain
to verify; no performance measurement was made.


## Typed conditional packed lookup model

The source-authenticated EUR model preserves 33 rows at indices 1–33 as byte 0,
byte 1 split into low five and high three bits, byte 2 split into low two and
high six bits, and byte 3. It reconstructs all 132 original bytes and rejects
wrong source, shape, bit ranges and coherent edits. Address calculations retain
original 32-bit wrapping; row 0 and indices outside the projection do not gain
a guest-side check.

One isolated Windows build and focused Wine test passed the model roundtrip and
original descriptor getter, packed reader and type guard on authenticated
descriptors for indices 0, 1, 33 and 251. Index 251 remains outside the model
and reaches bytes also owned by metadata14. Evidence is under
`target/rom-data-6e6ee4-model/`. The model is not yet a managed/runtime resource
or ordinary-use proof; Linux, USA and native Windows remain open. No performance
measurement was made.


## Managed conditional packed lookup

The EUR e97-d96 dataset adds `gameplay.conditional-packed-lookup`, with 33
authenticated four-byte records at indices 1–33. Strict activation compares
the parsed data against the exact original source even if a modified file and
manifest hash agree. The runtime reconstructs the declared bytes from their
typed fields and composes them with preceding owners and the raw ROM. Guest
selectors and arithmetic are unchanged; row 0 and indices beyond 33 are not
capped. Per-byte-field counters expose actual use.

One isolated Windows build and focused Wine test passed: all 86 previous
resource records and files stayed identical, all 132 source bytes and mixed
width/mirror boundaries were correct, and coherent tampering was rejected
without replacing the existing owner or state. Two translated original reader
prefixes over descriptor indices 0, 1, 33 and 251 matched the raw interpreter's
complete state and clock. With the source bytes poisoned, index 251 was served
by the existing metadata14 owner; the adjacent owner also supplied the mixed
boundary. Evidence is under `target/rom-data-6e6ee4-runtime/`; protected inputs
and frozen sources were unchanged. Production CLI, ordinary scene, Linux, USA
and native Windows remain to verify. No performance measurements were made.


## Conditional packed lookup production CLI

An isolated production Windows binary under Wine created and reused EUR
e97-d96, then advanced a retained neutral frame. All 88 generation files
matched the accepted dataset, 87 runtime resource identities and hashes matched
the manifest, and 90 managed files were unchanged during reuse and startup.
ROM, input, state, native cache, canonical executable and frozen build sources
were protected. Evidence is in `target/rom-data-6e6ee4-cli/`. This verifies
registration and startup, not ordinary use. Linux, USA and native Windows
remain to verify; no performance measurements were made.


## Conditional packed lookup ordinary-route candidate

A retained EUR Polly-to-Hildegard Fight checkpoint and its existing 500-frame
input are a source-backed candidate for ordinary use, not yet a positive read.
At the initial forecast state, a combat wrapper points to a unit whose selected
descriptor has type 5, lookup index 1 and active progress; the actor uses a
different wrapper. The original node adapter passes wrappers to the combat
callback, which can reach the byte-3 reader through the flag-8 path. The
authenticated state contains bit 47 in the inspected bitset, outside the
original 0–43 scan; an earlier inference that bit 43 blocked this path was
incorrect. Whether the retained attack takes the constructor branch and admits
that wrapper remains unobserved.

Evidence, corrected gate check and source windows are under
`target/rom-data-6e6ee4-ordinary-route/`; the existing input and state are
under `target/rom-data-equipment-ordinary/`. One bounded Wine observation can
distinguish use in this particular segment. No gameplay replay or performance
measurement was performed for this source selection; Linux and USA remain open.

Another retained EUR battle seed has Marche's bank-1 type-5 descriptor row 11
(packed index 1) learned and already assigned to slot `3c`; the Ability
screenshot shows Combat Combo. Its saved unit and global bytes satisfy the
inspected Combo menu gates, unlike a previously documented disabled Combo in a
different scene. A conditional combat prefix would also pass its inspected
gates if invoked for this unit. Original command 17 installs callback
`0802a231`, but its continuation has not yet been connected to a packed
reader for that unit. This is a static candidate, not observed packed-lookup use or
justification to replay the prior negative; the dispatch and a normal input
remain to establish. No performance measurements were made.

An independent Wine disassembly of the command-17 installation and callback
confirms that the inspected callback initializes menu context, dispatches via
a six-way jump table and writes the result state. It contains no direct call
to the packed reader in the inspected body. The later execution and participant
identity are still unconnected; this bounded inspection does not establish
global absence of a route. Windows are under
`target/rom-data-6e6ee4-combo-dispatch/`; no gameplay replay was performed.


## Conditional packed lookup F09 observation

One bounded Wine run of the retained 500-frame F09 confirmed Fight, from guest
frame 8668 to 9168, reported zero bus reads and zero bytes for the packed
lookup. It used the e97 production runtime and the existing state, input and
native cache; 206 guarded files and 695 frozen source hashes were unchanged.
This result closes only that observed segment. The candidate source chain has
conditional branches and callbacks, and the zero counters do not establish
global absence or identify which branch was taken. No full scene pair followed
the negative. Evidence is in `target/rom-data-6e6ee4-ordinary/`; positive
ordinary use, Linux, USA and native Windows remain open. No performance
measurements were taken.


## European conditional eight-byte descriptor banks

The original getter at `080d0ed4` selects a bank pointer at `086df4d4` and
an eight-byte descriptor using two live `u8` values. UI and other original
readers access the three little-endian halfwords at offsets 0, 2 and 4 and
the bytes at 6 and 7. Their full meanings and all indirect callers remain
open; the getter itself does not cap either index.

A separate original producer calls `080cd938` for bank values 1–23 and stores
its result in the unit's live count byte. Those counts support a conditional
projection of 23 pointer words at `6df4d8..6df534` and 684 eight-byte records:
`6df5b4..6e0acc` and `6e0afc..6e0b44`. The bank-23 count is nine and its end
coincides with an independently rooted pointer table. The bank-22 count is 11,
although 17 records fit before the next pointer; the 48-byte gap remains raw.
The count producer returns zero for bank 0. A separate mode-3 path suggests
a conditional use of bank-0 rows 1–15, but its complete producer and ordinary
reachability are not yet established, so bank 0 and pointer 0 remain raw.
These counts describe this producer, not a global guest-side index limit or
proof that every record is reachable. None of the declared source ranges
overlaps e97 owners.

Exact EUR source hashes, the original count jump table and per-bank counts,
next-root and reader disassemblies are retained under
`target/rom-data-6df4d4-descriptors/`. This is a source classification only;
model, runtime, ordinary use, Linux, USA and native Windows remain to verify.
No performance measurement was made.


## Typed conditional descriptor bank model

The EUR model represents the 23 selected bank pointers and 684 records as
three `u16` fields and two bytes per record. It strictly validates bank IDs,
source offsets, count bindings and exact ROM/source hashes, and reconstructs
all 5,564 bytes in the three declared source spans. The original getter's
`u8` truncation and address arithmetic remain available without a new guest
limit; bank 0, the bank-22 gap and other out-of-projection addresses are not
modeled.

One isolated Windows build and focused Wine test passed strict JSON/shape and
coherent-edit guards, every selected source byte, the original count dispatcher
for banks 0–24, seven original unit-count stores and eleven original descriptor
address cases including bank 0 and the bank-22 gap. Evidence is under
`target/rom-data-6df4d4-model/`; protected inputs and frozen source hashes were
unchanged. Managed publication, runtime use, ordinary scenes, Linux, USA and
native Windows remain open. No performance measurement was made.


## Managed conditional descriptor banks

The EUR e98-d97 dataset adds `gameplay.conditional-descriptor-banks` with
23 pointer words and 684 eight-byte records in three authenticated source
spans. Strict activation compares decoded data against the exact original
source even when an edited file and its manifest hash agree. Runtime reads
reconstruct those bytes from typed fields and compose with prior owners and
raw ROM; the original `u8` pointer and row arithmetic is unchanged. Bank 0
and the bank-22 gap remain raw. Per-field counters expose actual use.

One isolated Windows build and focused Wine test passed: all 87 previous
resource records and files were identical; the source bytes, mirrors and
mixed-width boundaries read correctly; coherent tampering was rejected without
replacing the active owner or state. Five translated original pointer and field
prefixes over eight inside/outside bank-row cases matched the raw interpreter's
complete state and clock. Poisoned selected bytes proved typed reads, while
separately poisoned pointer 0 and the 48-byte bank-22 gap remained raw. Evidence
is under `target/rom-data-6df4d4-runtime/`; guarded inputs and frozen sources
were unchanged. Production CLI, ordinary scenes, Linux, USA and native Windows
remain to verify. No performance measurements were made.


## Conditional descriptor banks production CLI

An isolated production Windows binary under Wine created and reused EUR
e98-d97, then advanced one retained neutral frame. All 89 generation files
matched the accepted dataset, all 88 runtime resource identities and hashes
matched the manifest, and 91 managed files remained unchanged on reuse and
startup. The guarded ROM, input, state, native cache, canonical executable and
frozen sources were unchanged. New descriptor counters were zero in this one
frame; registration does not establish ordinary use. Evidence is under
`target/rom-data-6df4d4-cli/`. Linux, USA and native Windows remain to verify;
no performance measurements were made.


## Conditional descriptor banks ordinary-route candidate

The retained normal-input Ability route is a source-backed candidate for the
new descriptor owner. The original `08077074` panel renderer loops over five
entries; entries 2, 3 and 4 load the bank pointer using unit byte 6 and read
descriptor halfwords 0 and 2 using slots `3a`, `3b` and `3c`, without a zero
or type guard. The retained state has selected unit bank 1, and the existing
540-frame input reaches this panel. This connects a real scene and original
code, but e98 descriptor reads and instruction PCs have not yet been observed.
The visible Fighter Tech text is a list row and must not be mistaken for the
separate internal category-3/type-5 selector.

The original renderer window and authenticated literals are under
`target/rom-data-6df4d4-ordinary-route/`; state/input provenance is under
`target/rom-data-linked-metadata28-cli/`. One bounded Wine observation of the
existing input is the next discriminator. No gameplay replay or performance
measurement was made for this source selection; Linux and USA remain open.


## Ordinary descriptor use in Ability navigation

One Wine run of the existing normal-input Ability route, frames 4519–5059,
reported 19 typed reads and 64 bytes: 52 pointer bytes, six from field 0 and six
from field 2. Field 4, kind and byte 7 were not exercised. This is aggregate
positive use of the EUR descriptor projection, not instruction attribution or
coverage of every field. The e98 production runtime used the existing native
cache; 212 guarded files and 699 frozen source hashes were unchanged. Evidence
is under `target/rom-data-6df4d4-ordinary/`. The full raw/typed scene pair,
other fields, Linux, USA and native Windows remain open. No performance was
measured.


## Ordinary Ability descriptor scene differential

The same existing 540-frame Ability route passed a Wine differential between
raw ROM and the e98 managed runtime. At every frame complete state and clock,
central and wide video, OAM, dispatch counts and five audio outputs matched.
An opt-in assertion in this comparison recorded 52 typed pointer bytes and six
bytes each from fields 0 and 2; field 4, kind and byte 7 were not exercised.
The 118 guarded files and 699 frozen source hashes remained unchanged. Both
arms shared the existing native cache; this does not cover all scenes or
establish general interpreter/native equivalence. Evidence is under
`target/rom-data-6df4d4-ordinary-pair/`. Other fields, Linux, USA and native
Windows remain unverified; no performance measurements were made.

The separately retained e99 battle Item-menu observation also logged two
reads of `gameplay.conditional-descriptor-banks`: four pointer bytes and one
kind byte, with its other fields zero. This establishes aggregate kind use in
that bounded scene, without attributing a specific record or instruction. The
same-scene raw/e99 pair passed, but its assertion covered bank 0 rather than
this secondary resource. The accepted manifest identifies the descriptor-banks
file as SHA-256 `31f6b49dc4ca7c7efc8138a1ec46bef0b325bcf2011606aabdf799294ea370c7`;
the retained log is under `target/rom-data-6df4d4-bank0-field4/`. At that
point field 4 and byte 7 for this resource remained ordinarily unobserved.
No replay or performance measurement was made for this reconciliation.

## Ordinary Battle Tech descriptor information

The original information builder reads fields 0, 2, kind and byte 7 of
enabled non-Item descriptors. The retained EUR Battle Tech R-info screenshot
shows a populated list. One e99 Wine observation of its existing 800-frame
state and input served 128 typed descriptor-bank bytes: 76 pointer, 16 field 0,
16 field 2, 10 kind and 10 byte 7; field 4 was zero. All 219 guarded files and
703 frozen sources were unchanged. The aggregate counters do not identify
individual records or executed PCs. Evidence is in
`target/rom-data-6df4d4-banks-info/`.

The existing Windows differential harness was reused for a Wine raw/e99 pair
on the same 800 frames. Full state and clock, central and wide video, OAM,
dispatch and all five audio outputs matched per frame. The harness asserted
pointer and field-0/2 use, and its emitted counters were separately checked for
ten kind and ten byte-7 bytes; field 4 remained zero. All 123 guarded files and
703 frozen sources were unchanged. Both arms shared the existing native cache;
this does not establish general native/interpreter equivalence. Evidence is in
`target/rom-data-6df4d4-banks-info-pair/`. Ordinary field-4 use, Linux, USA and
native Windows remain to verify. No performance measurements were made.

## Conditional property-selected mask table

Original EUR readers at `080ce64a`, `080cee9e` and `080ceff4` use the
literal `086e0b44`. They obtain metadata52 property `0x24`, multiply it by
four and load a 32-bit mask used in a bit test. All 116 authenticated physical
metadata52 records were reconstructed from the accepted e99 dataset and the
original ROM; property `0x24` corresponds to byte 45 and its values cover
every index from 0 through 34. The resulting 35 words occupy the exact
140-byte span `6e0b44..6e0bd0`, SHA-256
`89fa534fce962ae4e48c9812619d9f41467ce37442cfb598d4beebf81fc6e065`.
It begins at the end of descriptor bank 23, ends at the existing Item table,
and does not overlap an e99 resource source. Evidence is in
`target/rom-data-6e0b44-masks/` and the retained original Wine disassembly in
`target/rom-data-6df4d4-descriptors/`.

This is a conditional source classification, not proof that dynamic or invalid
selectors are globally limited to those values. Original arithmetic and
out-of-span raw behavior remain unchanged; a typed model, runtime publication,
ordinary use, Linux, USA and native Windows remain to verify. No performance
measurements were made.

The standalone typed model now preserves the 35 little-endian u32 masks and
reconstructs the 140 authenticated source bytes. It checks extent and rejects
coherent edits against the original source without adding a guest selector
limit. One isolated Windows build and focused Wine test verified every byte
and boundary, strict JSON and malformed shapes, the original load prefix for
indices 0, 1, 34, 35, 255 and u32 maximum, and original metadata property
results 0, 1 and 34 for three rows. Input and frozen-source guards remained
unchanged. Evidence is under `target/rom-data-6e0b44-masks-model/`. This is
conditional prefix evidence; managed publication, ordinary use, Linux, USA
and native Windows remain to verify. No performance measurements were made.

EUR e100-d99 publishes these masks as `gameplay.conditional-property-masks`.
Strict activation authenticates the decoded words against the original source,
and the runtime serves their bytes through the existing CPU/DMA/native reads
with separate usage counters. Guest instructions and selector arithmetic remain
unchanged; reads outside the projection retain their existing behavior.

An isolated Windows build and focused Wine test verified that all 89 e99
resources and files remain identical, every new source byte and mixed-width
edge is correct, and separately poisoned adjacent descriptor and Item bytes
still compose with their earlier owners. A coherent edited file plus updated
manifest hash was rejected without replacing the active owner or state. One
translated original load prefix for six inside/outside indices matched raw
interpreter state and clock, serving 12 typed native bytes. Inputs and frozen
sources were unchanged. Evidence is under
`target/rom-data-6e0b44-masks-runtime-retry/`; production CLI, ordinary use,
Linux, USA and native Windows remain to verify. No performance measurements
were made. The initial build's missing test trait import was corrected before
the successful retry.

An isolated production Windows build subsequently created and reused e100
under Wine and advanced one retained neutral frame. All 91 generation files
matched the accepted dataset, 90 runtime resource identities matched the
manifest, and 93 managed files remained unchanged on reuse and startup.
The mask counters were zero in this frame; it does not establish ordinary
mask use. Evidence is under `target/rom-data-6e0b44-masks-cli/`. Linux, USA
and native Windows remain to verify; no performance measurements were made.

The original headgear inventory path can call the equipment check that reads
these masks; its retained EUR state has five nonzero equipped items and the
documented input opens the headgear list. One e100 Wine observation of that
existing 560-frame scene recorded 91 typed reads and 364 source bytes; all
224 guarded files and 707 frozen sources were unchanged. These aggregate
counts do not identify executed PCs, selectors or distinct source bytes.
Original windows and observation are in
`target/rom-data-6e0b44-masks-ordinary/`.

One isolated Windows test build and Wine raw/e100 comparison then passed for
the same 560 frames: complete state and clock, central and wide video, OAM,
dispatch and five audio outputs matched at every frame. The test asserted the
same 91 typed reads and 364 source bytes; all 128 guarded files and 707 frozen
sources remained unchanged. Both arms shared the native cache, so this is not
a general native/interpreter comparison or a count of distinct mask bytes.
Evidence is under `target/rom-data-6e0b44-masks-ordinary-pair/`. Linux, USA and
native Windows remain to verify. No performance measurements were made.

## Ordinary Steal-menu descriptor field 4

The original non-Item battle menu obtains the selected unit's descriptor bank,
checks descriptor kind and learned progress, then reads field 4. The retained
EUR seed selects bank 1 and has learned Steal records; its existing screenshot
shows Marche's populated Steal list. One e99 Wine observation of the existing
280-frame input served 131 typed descriptor-bank bytes: 48 pointer, 10 field 0,
60 field 4 and 13 kind; field 2 and byte 7 were zero. All 218 guarded files
and 703 frozen sources were unchanged. These aggregate counters do not
identify exact records or PCs. Evidence and authenticated original source
windows are in `target/rom-data-6df4d4-banks-field4/`.

An isolated Windows test build and Wine raw/e99 pair then passed on the same
280 frames: complete state and clock, central and wide video, OAM, dispatch
and five audio outputs matched every frame. The narrow test assertion
confirmed pointer and field-4 use, including the same 60 field-4 bytes.
All 123 guarded files and 703 frozen sources were unchanged. Both arms shared
the existing native cache; this is not a general native/interpreter comparison
or coverage of all records and scenes. Evidence is in
`target/rom-data-6df4d4-banks-field4-pair/`. Linux, USA and native Windows
remain to verify. No performance measurements were made.


## Conditional descriptor bank 0 source

The original mode-3 producer `080d08b4` reads pointer 0 from `086df4d4`
even when the selected unit's byte `37` is zero. It takes the interval 1–15
from authenticated metadata52 row 1. Caller `081379f0` tests unit byte `37`
before scanning those descriptors; all 15 records in this interval have kind 1
and field-4 values 251–265. The original initializer can set byte `37` to 1
when a nonnull unit has byte `7 == 24` and the normal-count branch is entered.
Its false branch skips that store; a separate producer can clear it. Metadata52
row 24 exists, but an ordinary instance reaching this initializer has not been
proved. A bounded read-only census of 62 retained snapshots found no matching
nonzero slot or predicate witness; that is not global absence.

This supports two conditional source bindings: pointer 0 at
`6df4d4..6df4d8` and records 1–15 at `6df53c..6df5b4`. Record 0 and other
indices remain raw; the original selector arithmetic and bounds are unchanged.
Neither binding overlaps e98 owners. Exact hashes, source windows, predicate
and state checks are under `target/rom-data-6df4d4-bank0/`. This is source
classification only: model, runtime, ordinary use, Linux, USA and native
Windows remain open. No performance measurement was made.


## Typed conditional descriptor bank 0 model

A strict standalone EUR model reuses the descriptor fields for pointer 0 and
records 1–15, reconstructing the two authenticated spans (124 bytes). Record
0 and other indices remain outside the model; no guest-side bound or fallback
is introduced. Shape and coherent source edits are rejected.

One isolated Windows build and focused Wine test passed the original predicate
for null and unit-byte values 23/24/25; the original initializer preserved the
false-branch slot and stored 1 for 24. Original mode 3 returned pointer 0 and
bounds 1–15 with slot 0 or 1, and the original caller found the first and last
field-4 values only with a nonzero slot. These are conditional original-code
checks, not a proof of ordinary use. Evidence is under
`target/rom-data-6df4d4-bank0-model/`; managed publication, runtime, Linux,
USA and native Windows remain open. No performance measurement was made.


## Managed conditional descriptor bank 0

The EUR e99-d98 dataset adds `gameplay.conditional-descriptor-bank-zero` for
pointer 0 and records 1–15 in two authenticated spans. Strict activation
compares decoded data against the original source even when a modified file
and manifest hash agree. Runtime reads reconstruct these bytes from typed
fields and compose with the existing descriptor owner; original selector
arithmetic is unchanged and record 0 remains raw.

One isolated Windows build and focused Wine test passed: all 88 previous
resource records and files remained identical; source bytes, mirrors and mixed
width boundaries were correct; coherent tampering was rejected without
replacing the owner or state. Poisoned adjacent pointer 1 and bank-1 record 0
proved composition with the earlier owner; separately poisoned bank-0 record 0
remained raw. Three translated original pointer/field prefixes over five
inside/outside cases matched raw interpreter state and clock. Evidence is under
`target/rom-data-6df4d4-bank0-runtime/`; guarded inputs and frozen sources were
unchanged. Production CLI, ordinary use, Linux, USA and native Windows remain
to verify. No performance measurements were made.


## Conditional descriptor bank 0 production CLI

An isolated production Windows binary under Wine created and reused EUR
e99-d98, then advanced one retained neutral frame. All 90 generation files
matched the accepted dataset, 89 runtime resource IDs and hashes matched the
manifest, and 92 managed files stayed unchanged during reuse and startup.
Guarded ROM, state, input, native cache, canonical executable and frozen build
sources were unchanged. This proves registration and startup, not ordinary
bank-0 use. Evidence is under `target/rom-data-6df4d4-bank0-cli/`; Linux, USA
and native Windows remain open. No performance measurements were made.


## Ordinary Item-info access to descriptor bank 0

The original Item-info path provides another route to bank 0: the selected
Item set is copied to unit byte 36 and passed to the mode-2 descriptor getter.
Its slot-1 fallback selects pointer 0 and records 1–15. In the Item branch,
the information constructor bypasses the progress/type filters and reads
record fields 0, 2, 6 and 7. This route does not require unit byte 37 or job 24.

One bounded EUR Wine run opened Item information from the retained frame-4519
state, using the existing R-info input with its intervening Down press removed.
Over 800 frames, typed bank-0 counters recorded 4 pointer bytes, 30 field-0
bytes, 30 field-2 bytes, 15 kind bytes and 30 byte-7 bytes; field 4 remained
unobserved. All 219 guarded files and 703 frozen source files were unchanged.
Evidence is in `target/rom-data-6df4d4-bank0-ordinary/`. These aggregate
counters do not identify executed PCs or establish coverage of every scene.

An isolated Windows test build and one Wine raw-versus-e99 comparison then
passed on the same 800 frames: full state and clock, central and wide video,
OAM, dispatch and all five audio outputs matched at every frame. The test
asserted the same pointer, field-0, field-2, kind and byte-7 use; field 4
remained zero. All 123 guarded files and 703 frozen sources were unchanged.
Both arms used the existing native cache, so this is not a general comparison
of native and interpreter execution. Evidence is in
`target/rom-data-6df4d4-bank0-ordinary-pair/`. Ordinary field-4 use, Linux,
USA and native Windows remain to verify. No performance measurements were made.


## Ordinary battle Item-menu bank-0 field 4

The original battle Item-menu builder receives the selected unit's mode-2
descriptor bank. For slot 36 equal to 1, the getter selects bank 0; the builder
adds record 15 and reads its field at offset 4. The retained EUR battle seed
has Marche's slot 36 set to 1, and the retained Item-menu screenshot shows his
Item list. This identifies a candidate without requiring slot 37 or job 24.

One Wine observation of that existing 340-frame input served 100 typed bank-0
bytes: 56 pointer, 12 field 0 and 32 field 4. Field 2, kind and byte 7 were
zero in this scene. All 218 guarded files and 703 frozen sources were
unchanged; the counters do not establish exact executed PCs or every record.
Evidence is in `target/rom-data-6df4d4-bank0-field4/`.

An isolated Windows test build and Wine raw-versus-e99 comparison passed for
the same 340 frames. Full state and clock, central and wide video, OAM,
dispatch and all five audio outputs matched per frame; the test asserted
pointer and field-4 use and observed the same 32 field-4 bytes. All 123
guarded files and 703 frozen sources were unchanged. Both arms shared the
existing native cache, so this does not establish general native/interpreter
equivalence. Evidence is in `target/rom-data-6df4d4-bank0-field4-pair/`.
Linux, USA and native Windows remain to verify. No performance measurements
were made.
