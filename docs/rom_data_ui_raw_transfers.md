# European localized raw UI transfers

## Declared source contract

`ROMDATA-01-LOCALIZED-RAW-TABLE-SOURCES` authenticates seven independently
rooted language pointer groups: `71e5f4`, `71e608`, `71e66c`, `71e694`,
`71e6a8`, `71e6e4` and `71e6f8`. Each contains five declared little-endian
pointers selected by live unsigned byte `02001f6e` multiplied by four. All
13 aligned root literals in the retained ROM census are covered. Physical
slots0..4 describe ownership; the original uncapped address calculation for
language5/255 remains separate. No fallback or skipped guest read is inferred.

These are raw transfer views. Their lengths come from original helper controls,
not headers, pointer spacing, tile names or visual interpretation. In root order:

- `71e5f4`:224 bytes from each bank start. Three original callers copy through
  `145c14`, which branches throughr3 to the copied IWRAM memcpy callback.
- `71e608`:160 bytes from each bank start, copied to scratch+`e0` through the
  same callback. The subsequent merged scratch upload adds no ROM ownership.
- `71e66c`:768 bytes from each bank start. The fused call at`12ee24` targets
  `145230`, SWI0B/CpuSet with`0180` halfwords.
- `71e694`:1600 bytes from each bank start, the union of two conditional views:
  first832 bytes, or768 bytes at+`340`. The original helper result's low byte
  chooses the branch; no meaning for that mode is invented.
- `71e6a8`:2048 bytes from each bank start. A live task queues two1024-byte
  copies through`000b14`, control`84000100`; the zero-task branch calls
  SWI0C/CpuFastSet with`0200` words for the same full extent.
- `71e6e4`:2432 bytes from each bank start, SWI0C with`0260` words.
- `71e6f8`:only384 bytes at bank+`0200..0380`. Three separate state handlers
  queue128-byte copies at+`200`,+`280` and+`300` through`000ad0`, each with
  control`80000040` (halfwords).

The `000ad0`/`000b14` helpers store low16 count bits plus their respective
DMA controls in the linked transfer queue. The original executor preserves
the ROM source, chooses DMA3, and writes source/destination/control registers.
The queue, guest timing, live destinations and later uploads remain original.
The callback mapping through ROM`370eb4` to IWRAM`03005ee9` reuses the closed
[raw palette proof](rom_data_ui_palettes.md#source-and-consumer-contract).
Copied executable code is not a data resource.

The declared35 views and seven selector tables form42 disjoint spans,
38220 source bytes (38080 payload+140 selector). Exact little-endian word
reconstruction passes; concatenation by source offset has SHA256
`e11ef4d5f16feea8ef90e7fce58056e19d689004b7ca973270285e3cd31b8f80`.
No span overlaps the83 accepted e92-d91 owners/50602 runtime bindings.
All84 accepted dataset files and guarded inputs remain byte/mtime exact.

## Dynamic views remain open

Four additional calls through`000ad0` use root`71e6f8` with64-byte transfers
at offsets from two little-endian u16-pair tables, rooted at`3996f4` and
`39970c`. Live byte task+2 minus4 selects a pair; each value is shifted left5.
Bit3 at`030028a0` chooses the table. The raw calculation has no observed upper
bound, and the caller dispatcher does not establish one. A six-row count from
24-byte spacing or convenient values is therefore rejected. Those metadata,
any lower bank prefix and offsets outside the declared fixed union remain
unowned. This source contract does not close the entire seventh bank catalog,
its ordinary liveness, all producers or the full helper ABI.

## Verification and remaining work

Static authentication covers32 windows (23 new/nine reused),193 instruction/
literal pins,13 root literals,13 fixed transfer calls and the four dynamic
calls. One classifier attempt stopped before producing outputs when the fused
`12ee24` target contradicted an initial word-copy assumption. The corrected
classifier also authenticates `000ad0` halfword controls rather than borrowing
`000b14` word units. No incorrect extent was published or tested. An unrelated
`04fc70` search window is retained as diagnostic evidence, not consumer proof.
The successful result preserves130 hash/mtime guards. Evidence stays private
under ignored `target/rom-data-localized-raw-table-sources/`.

This is source analysis only: schema1/e92-d91, runtime code and accepted data
are unchanged. Typed modeling, a reduced original-source Wine check, managed
consumption/native state-clock comparison, actual CLI and positive ordinary use
remain pending. Dynamic source domains, the separate known`71e5e0` alias,
indirect/global extent and gaps/tails remain open. New Linux/USA/native Windows/
32:9 and the broader ROM TODO remain pending. No build, game, new checkpoint,
navigation, older acceptance replay or performance/storage measurement ran.

## Typed model

`rom_data::ui_raw_transfers` now provides a strict `RawTransfers` model with
seven tables and35 ordered `BankView` values. A view preserves its bank pointer,
relative source offset and little-endian words. It reuses the existing
physical five-slot table representation while preserving the original raw
language address calculation. The seventh group's lower prefix and tails
remain outside its view. Shape validation and exact EUR provenance are
independent: a raw word edit can retain valid shape and fail source
authentication. The catalog authenticates the32 retained reader windows and
all38220 declared source bytes. The model unit left e92-d91 and its84-file generation unchanged; the following
runtime unit now registers the resource separately.

The focused EUR Wine check passes all38220 source bytes/42 spans, strict JSON,
21 shape/source guards and91 original selector prefixes (13 consumers, each
with language0..4/5/255). Isolated unit inputs execute60 original transfer
helper calls, including25 real queue-executor/DMA copies. Callback byte copies,
CpuSet halfword copies, CpuFastSet word copies and both queued DMA widths agree
with independent source bytes. Complete EWRAM, VRAM, OAM and palette guards,
IWRAM outside the actual stack scratch, SP and callee registers, and final ROM
all match. These are unit inputs, not ordinary game fixtures or full caller ABI
proofs. The original IWRAM callback code is installed from its authenticated ROM
copy in the isolated unit machine; the closed initializer proof is not rerun.

The accepted603-source/262-core-input Windows build is warning-free, SHA256
`35bdc5fd15d2c458df63aadecfabfeea766c44ef2924b92d5dafc57c9105f69c`.
Three Windows builds and two Wine attempts were needed. Before Wine, a test-only
stack guard was corrected from20 to36 bytes for the original DMA wait helper.
The first Wine attempt then stopped at an incorrect expectation that every span
edge was raw: three adjacent table boundaries correctly select another owned
table. The corrected oracle uses independent spans. The model and source
contract did not change for either test correction; diagnostic executables,
logs and exact source preimages remain retained. The accepted result preserves
44 unit guards,130 source guards,601 baseline source hashes and three current
implementation paths; the hub inverse matches exactly. All84 accepted e92 files
remain unchanged. The6635 executed instructions count coverage only.

Private evidence is under`target/rom-data-localized-raw-transfers-model/`.
Managed registration and selected native state-clock checks are completed
below. Actual CLI/ordinary use, dynamic index domains and previously listed
platform/catalog limitations remain pending.
No gameplay/checkpoint/navigation or older acceptance proof was replayed; no
performance or storage measurement ran.

## Managed raw transfer resource

The implementation now registers `graphics.ui-raw-transfers`, fixed file
`ui-raw-transfers.json`, format `ui-raw-transfers-v1-json`, at EUR schema1/
extractor93/decoder92. It adds index83 to the previous83 resource IDs, making
85 files including the manifest. The prior83 entries and payloads are retained.
Strict source/model/binding validation precedes transactional attachment.

The immutable owner stores typed bank words and pointer tables, resolving
each source byte independently and composing crossing reads with earlier
owners or raw ROM. The seventh group's unowned prefix and tails remain raw.
Separate nonserialized counters expose bus reads, served source bytes, selector
bytes, payload bytes and distinct banks/groups used. Original language loads,
raw index arithmetic, CPU copies, transfer queues, DMA destinations and guest
clock work remain active. No direct host rendering or transfer shortcut is
introduced. The fixed parser guard follows the existing bounded-input policy;
no storage optimization or allocation cap for these views is added.

The reduced EUR Wine integration check passes publication of85 files/84 IDs,
retaining all83 previous records and payloads exactly. New-file interruption
leaves no published manifest. Fourteen source-coherent shape/hash/path/binding/
coverage rejection cases and two full-set dependency checks preserve the prior
attached owner and complete machine state after failed attachment.

Twenty-three original/interpreter/native paths match complete states and guest
clocks with all38220 new source bytes and four preceding indexed-selector bytes
altered after Load/setup. These cover12 loads, one original CpuSet call, one
original transfer-queue executor and nine raw-language prefixes, split into
four native families that exclude their stop boundaries. Read8/16/32,
interpreter fallback and the DMA-causing write32 callback are all accounted.
939 served new bytes and three prior indexed bytes are observed; fallback
accounts for768 of the new bytes, so this is not an all-direct-native or distinct
catalog-coverage claim. The115 visited instructions and25 dispatches count
coverage only.

Every declared byte,35 views/seven groups,140 selector bytes and38080 payload
bytes match the ROM through the bus. All42 span edges/widths/mirrors, adjacent
new tables, prior indexed data and sparse raw prefix holes pass. The archive
at`441608` starts exactly at the new sparse view's end; it remains served by
its previous indexed owner. Load/slot Undo retain the immutable owner while
files are unavailable. Four64-byte transfers, mixing prior indexed, prior UI
and new selectors through DMA16, DMA32, CpuSet and CpuFastSet, match complete
states and guest clocks.

The accepted605-source/263-core-input Windows build is warning-free, SHA256
`4b219adc54111f10b01657afe08092fb88f366dd4fecda609055b3413dbd966f`.
Two builds and one Wine filter were used. Before Wine, only the new test leaf
was corrected to distinguish the unowned lower prefix from the prior-owned
successor archive; product runtime code was identical between builds. The first
warning-free executable/log/source preimage remain diagnostic evidence.
309 preservation guards cover605 frozen sources,603 model inputs, seven
current implementation paths and all84 old accepted files. The current-only
parameter27 observer is preserved, excluded only from the frozen overlay.

Accepted dataset SHA256 is
`309b6eaf5af3c3ab26dc7b87050723b79f205fcb83dee0054625f38a7cdd1a60`;
manifest SHA256 is
`a022df159d73ef3b7f1e2f1ec5b62d3024507898386138d95d87cbd888e821b0`;
new JSON SHA256 is
`2a89044c7724107e32360279ff44d801e2f73f9802309dcf853394c29f73ef65`.
Private evidence is under`target/rom-data-localized-raw-transfers-runtime/`.
The runner's inherited console e92/DMA3 labels are stale metadata; accepted
JSON asserts e93-d92 and four mixed transfer cases. No build or test is repeated
for those labels.

Actual CLI/ordinary use, full caller ABI, dynamic index domains and the broader
catalog/platform limitations remain pending. The closed source/model proofs
are not replayed, no ordinary game fixture is created, and no performance or
storage campaign runs.

## Actual CLI registration

The actual Windows CLI now creates and reuses EUR e93-d92 under Wine, then
loads a retained authorized normal EUR checkpoint copy at24755 and advances
one neutral frame to24756. All three child processes exit0. All85 generation
files are byte-identical to the accepted runtime dataset; all84 logged resource
ID/hash pairs match. Reuse and startup preserve hash/mtime for87 managed paths
and the old native cache with28859 loaded blocks. The new six raw-transfer
counters are zero, establishing registration/startup compatibility only. No
positive ordinary consumption or full scene equivalence is claimed.

One warning-free production build uses exactly the accepted605 runtime source
files, without overlays. Its SHA256 is
`ffda6c595cfd38eae46e8f9ac2ee709ba077b4198a053f25b58ba09be214c483`.
The canonical executable's exact hash/mtime is restored in the build's finally
block.203 preservation guards cover the605 CLI and605 runtime source hashes,
all85 accepted files, original ROM/checkpoint/cache/TinyCC and canonical
executable. The private adopted generation is e93-d92/generation-5FQy4l under
`target/rom-data-localized-raw-transfers-cli/windows-cli/resources/`; receipts
and logs are retained in that ignored unit. No new checkpoint endpoint, game
action/navigation, earlier acceptance replay or performance/space campaign
is involved. At this stage positive ordinary use, dynamic index domain/full
helper ABI and all previously listed platform/catalog limitations remained pending.

## Ordinary European boot equivalence

The existing EUR e93-d92 production CLI now demonstrates ordinary consumption
on the retained 1200-frame clean-boot input under Wine. The six resource counters
report 1,372 reads and 5,488 served source bytes: 16 selector bytes and 5,472
payload bytes, with four banks and four groups observed. These counts include
repeated reads; they do not establish distinct-byte or whole-catalog coverage.

One Wine comparison using the already accepted runtime test executable and
the same 605 source files, dataset and input matches every complete serialized
state and clock, central and wide image, OAM, dispatch and five audio streams
for all 1,200 frames. Both arms use the same retained native translation cache;
this does not establish global interpreter/native parity. The 92 guarded ROM,
cache, loader, input, executable and managed files retain their hashes and
modification times. The sandbox initially prevented Wine from creating its
server lock; that failed invocation produced no game result, and the successful
checks used the necessary sandbox escalation. No new build or earlier proof was
repeated. Private receipts are in `target/rom-data-raw-ordinary-boot/`.

This closes the selected ordinary EUR/Wine case, not all ordinary paths. The
dynamic seventh-group domain, remaining groups, full caller ABI, global catalog,
new Linux and USA verification, native Windows and the broader TODO remain open.
No performance measurements were taken.

## Bounded dynamic-view producer discovery

The dynamic `71e6f8` transfer path has a concrete producer chain. Original
`0cac18` passes the object at `02015744` to `0c9194`; transitions preserve a
pending selector in byte3 and copy it to byte2. Setter `0caafc` truncates its
input to a byte, and its 24 direct Thumb callers include six explicit calls
with values 7, 8 or 9. Two other callers load a live structure byte, so these
examples do not establish a global upper bound. Original helper and task-state
gates remain relevant; static callers do not prove ordinary execution.

For values 7, 8 and 9, each of the two original tables at `3996f4` and
`39970c` supplies one authenticated four-byte row. The four resulting distinct
64-byte offsets cover bank-relative `0100..0200`; the five declared language
slots therefore provide five candidate 256-byte prefix fragments. The six rows
and five prefixes total 1,304 bytes, disjoint from all e93 resource source
spans. Values 7..9 and those candidates are a bounded source discovery, not a
new runtime owner, a six-row table limit or proof of all possible indices.
The other live-byte producers, full table extent and remaining lower prefix and
tails stay open. Private source evidence is in
`target/rom-data-raw-dynamic-domain/source-result.json`; the exact ROM and 86
other guarded inputs remain unchanged. No build, Wine test or guest replay was
needed for this static inspection. Linux and USA verification remain pending.

## Dynamic fragment source model

`ui_raw_transfers::dynamic_fragments` models the six discovered selector rows
and five bank prefixes as seven explicit source spans. It preserves the full
little-endian halfwords and words, validates that each described 64-byte view
fits the declared prefix, and returns no byte for positions outside these
fragments. Shape validation and exact EUR source authentication are separate;
the latter requires the exact ROM identity. The model adds no runtime owner,
guest selector cap or new managed dataset revision.

One core-only Windows build and one focused Wine check pass exact reconstruction
and byte lookup of all 1,304 bytes, seven boundaries, strict JSON roundtrip,
shape rejection and independently authenticated source/ROM rejection. The
test uses isolated source inputs and does not execute the original transfer
routine. A preparation guard was corrected before compilation; no earlier proof
was repeated. Private receipts are in `target/rom-data-raw-dynamic-domain/`.
Managed extraction, typed bus reads and production CLI registration are covered
below; bounded original/native execution is covered further below. Ordinary use,
other indices and Linux/USA checks remain open.

## Managed dynamic fragment resource

EUR schema 1 / extractor 94 / decoder 93 publishes the independent
`graphics.dynamic-raw-fragments` resource at index 84 in
`dynamic-raw-fragments.json`. It authenticates the seven bounded source spans
and composes typed bytes with preceding owners and raw ROM, preserving the
original selector arithmetic and guest transfer work. Previous 84 resource
records and payloads remain unchanged. No global selector bound or ownership
of other fragments is implied.

The focused Wine integration test publishes and reloads 85 resources, checks
all 1,304 bytes with their ROM backing poisoned, tests read widths and mirrors
at all seven boundaries, and confirms the adjacent earlier raw-transfer owner.
One DMA16 transfer matches complete machine state and clock against the raw
ROM reference. A source-coherent altered JSON is rejected without replacing the
attached owner or changing state. ROM, previous accepted dataset, build and
implementation inputs retain their guarded hashes and timestamps.

The first Wine attempt identified a missing page for the final bank prefix;
the corrected build and one retry pass. A separate first compilation found
missing test arguments and a trait import, fixed before execution. The system
disk filled before the retry could start; two disposable copies from a
previously passed unit were retired by the project retention tool and its
receipt retained. Temporarily relocated executables were restored with their
recorded hashes and timestamps. Evidence is in
`target/rom-data-dynamic-raw-runtime/`; failed diagnostic data remains intact.

The subsequent production CLI registration is described below. The original
dynamic routine/native execution is tested below for bounded isolated cases;
positive ordinary use, other indices and the full selector domain remain
unverified. Linux,
USA and native Windows checks remain pending. No performance measurement ran.

The existing `graphics.dynamic-raw-fragments` owner now exposes independent
request, total source-byte, selector-byte and payload-byte counters. Production
diagnostics emit them under that resource ID; these counts include repeated
reads and do not establish distinct-byte coverage. The data schema and original
guest behavior are unchanged. A focused Windows build and Wine test against the
accepted e100 dataset verified poisoned selector and payload reads, a mixed
boundary and an outside read: three requests served seven source bytes, split
into three selector and four payload bytes, while the outside read did not
increment the counters. Guarded inputs and frozen sources stayed unchanged.
Evidence is under `target/rom-data-dynamic-fragment-usage/`. The actual
production log and ordinary use remain to verify; Linux, USA and native
Windows are pending. No performance measurements were made.

An isolated production Windows build subsequently ran one retained neutral
frame under Wine against the already prepared e100 directory. The CLI emitted
the new resource usage line with all four counters zero; 93 managed files and
the other guarded inputs stayed unchanged. This confirms the log and startup,
not absence in other scenes. Evidence is under
`target/rom-data-dynamic-fragment-usage-cli/`. Ordinary use, Linux, USA and
native Windows remain to verify. No performance measurements were made.

## Dynamic fragment production CLI

The production runtime now identifies `graphics.dynamic-raw-fragments` and
its file hash in the same resource log as the other decoded families; the
extraction catalog also describes its bounded source scope. One isolated
Windows production build from 681 frozen source inputs succeeded without
compiler diagnostics and left the shared executable unchanged.

Under Wine, the actual CLI created an EUR e94-d93 dataset, reused it and
started one existing neutral frame. All 86 generation files match the accepted
managed runtime dataset; all 85 loaded resource identities and hashes match
the manifest. All 88 managed files and the guarded ROM, seed, native cache,
accepted dataset, implementation inputs and canonical executable stayed
unchanged during reuse and startup. This is registration and loading evidence,
not proof that this frame reads the dynamic fragments. Results and logs remain
in `target/rom-data-dynamic-raw-cli/`.

Bounded original dynamic routine/native execution is verified below; positive
ordinary use, global index domain and Linux/USA/native Windows verification
remain pending. No performance measurement was performed.

## Original dynamic routine and translated execution

One focused EUR Wine test executes original Thumb routine `0c9060`, its two
queue calls and the original queue executor at `0800`. Isolated inputs cover
selectors 7, 8 and 9, both flag-bit3 table branches and all five declared
language slots, for 30 combinations. For each, it verifies the two computed
source pointers, destinations and queue controls, then checks both 64-byte
VRAM transfers against the original ROM.

Complete serialized machine state and guest clock match between the raw-ROM
interpreter, the interpreter attached to the decoded dataset, and native
translated instruction dispatch attached to that dataset. All 1,304 dynamic
source bytes are poisoned in each decoded arm after attachment; correct rows
and transfer payloads therefore require the typed owner. ROM, accepted dataset,
TCC library, executables and relevant source inputs retain their guarded hashes
and timestamps. The focused Windows test build and Wine result are retained in
`target/rom-data-dynamic-original-native/`.

This is an isolated unit setup, not ordinary gameplay, a global selector bound,
full original helper ABI, or a claim that every native instruction uses a direct
callback without interpreter fallback. Other selectors, Linux, USA and native
Windows remain to verify; no performance measurement was performed.

## Ordinary dynamic reachability investigation

Original getter `0ccf00` reads byte `02002030 + u16(index)`, so the index 13
check in setter `0caafc` depends on `0200203d`. Original callback `064678`
requests that byte be set to 1 and callback `066588` requests 0. The wrapper
`0c4ce4` passes context `020101f8` into initializer `0c3be4`; the semantic
name of this UI lifecycle is not yet established. These are source facts, not
a proof that a particular ordinary scene reaches the dynamic transfer.

A readonly scan of 14 existing EUR catalog checkpoint entries found zero for
the gate, current and pending selectors and phase in those snapshots. The
snapshot parser checked ROM identity and payload checksum; its decoded DS-20
registers, CPSR and IWRAM `7ffc` match an earlier Wine inspection. A first
scan had a region-hash lookup error and is retained separately as invalid.

One Wine observation of the existing 320-frame worldmap→Party→Unit input
found no bus requests in the seven dynamic spans and no selected callback entry
in the sampled native dispatches; gate, selectors and phase stayed zero at
those sampled boundaries. No events were dropped. Native dispatch sampling
can miss changes inside a block; the bus audit also excludes folded constants
and unselected direct host readers. Thus this is a bounded negative for the
retained route, not proof of absence throughout the game. Source, ROM, input,
checkpoint, cache and dataset guards passed. Evidence remains under
`target/rom-data-dynamic-ordinary-reachability/` and
`target/rom-data-dynamic-ordinary-observe/`.

A normal positive dynamic use and its full state/clock comparison remain open,
as do the global selector domain and Linux/USA/native Windows checks. No
performance measurement ran.

Further read-only source inspection links original `08096b72` through
`080c4ce4` to initializer `080c3be4`, which can request selectors 8 and 9;
the setter still returns without a request when gate `0200203d` is zero.
Dispatcher `08063a74` selects the gate-enabling callback `08064679` at task
state 4 through root `0200f450`, with the separate flag-40 registration gate.
Three additional authenticated retained states (worldmap, battle and F09
forecast) had zero gate, current/pending selectors and phase, with flag 40
clear. This extends only the snapshot negative: the event producing task state
4 and a normal positive scene remain unestablished. No gameplay replay or
performance measurement was performed for this inspection.

The original state-4 producer is now identified conditionally. The state-2
handler `080644f8` reads a signed result from menu object data at offset 6;
results 1, 2 or 3 preserve the value 4 written to the task at `080645a6`.
It pushes states 4, 52 and 53, sets state 50, and later pops through
50 → 53 → 52 → 4; the final pop is gated by an original helper. In the normal
menu callback, a new A press on an enabled entry writes selected index plus 1
to that result. Eight original Windows disassembly windows were independently
checked under Wine and retained in `target/rom-data-dynamic-state4-producer/`.

This establishes a conditional input and task transition, not which user-facing
scene opens that menu or whether a retained state reaches it with an eligible
entry. Ordinary positive dynamic use, Linux, USA and native Windows remain
open. No gameplay replay or performance measurements were made.

The initial dispatcher states also have conditional paths into state 2. The
callback at `080638ce` clears the task state; state 0 sets task byte `4d` to 1
and, when its flag-8 branch chooses message `a6`, queues state 2. Otherwise it
queues state 3; state 3 queues state 2 if byte `02000072` is nonzero and state
1 otherwise. Five authenticated original Wine disassembly windows are under
`target/rom-data-dynamic-state2-entry/`. These instructions do not establish
which user-facing scene registers the task or reaches an eligible menu; the
older bounded negatives remain unchanged. No gameplay replay or performance
measurement was performed.

An additional bounded inspection revisited the 24 known direct setter calls.
Their immediate values outside the already modeled 7–9 are 0–3, which do not
enter the dynamic transfer path. Two calls load task byte `50`; the inspected
stores to that byte produce 3, 7 or 8. In particular `0806491e` writes 7,
`0806498c` writes a register set to 8, and `08064dd4` writes 8. Original Wine
windows are under `target/rom-data-dynamic-selector-witnesses/`. These findings
do not justify adding indices 4–6 or 10 and above to the owned fragments, and
the local inspection does not exclude alias, indexed or external writes. No
gameplay replay or performance measurement was performed.

## Dispatcher lifecycle and Area List observation

An aligned Thumb call census of the authenticated EUR ROM finds one direct call
of dispatcher `063a74`, at `0638ac` inside callback `063878`. Its pointer occurs
at `127bc0`; the code at `127b8c` tests packed flag `40h` using getter `0cce98`
and, when set, registers callback `063879` in slot 5. The getter tests bit 0
of byte `02001f78`. At `0454e8`, the selected value is compared with 2; the
input-gated branch calls setter `0ccecc(40h, 1)` at `045508`. Thirteen original
instruction pins and four fused call targets were verified. An earlier probe
used the wrong address and another the wrong bit; neither is valid evidence.
A corrected readonly scan found bit 0 clear in 14 existing EUR checkpoints.

The retained worldmap input reaches the menu labelled Area List after 240
frames, making it a source-backed candidate, not proof of the callback's UI
meaning. One isolated Windows audit build and Wine observation of that input
sampled `0454e8` once, but byte `02001f78` remained `08` (bit 0 clear) and the
gate, selectors and phase remained zero at sampled dispatch boundaries. There
were no bus requests in the seven dynamic spans and no dropped or truncated
events. ROM, input, checkpoint, dataset, native cache and frozen sources passed
preservation checks. Results are in `target/rom-data-dynamic-dispatch-lifecycle/`
and `target/rom-data-dynamic-area-observe/`.

This is a bounded negative for that recording: native samples can miss changes
inside blocks, and the bus audit excludes folded constants and unselected direct
host reads. Ordinary positive dynamic use and its full state/clock comparison,
the global selector domain, and Linux/USA/native Windows checks remain open.
No performance measurement was performed.
