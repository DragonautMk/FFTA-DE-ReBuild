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

The same managed filter was later run under EUR Wine with e127-d126 and its
100-resource manifest, using the retained 84-resource e93-d92 predecessor.
It passes all 1,304 bytes, seven source spans and mixed boundaries, the DMA
full-state/clock comparison and transactional rejection. ROM, predecessor,
selected source and test executable remained protected; successful scratch was
removed. Evidence: `target/validation/units/rvr14-dynamic-raw-e127-fn5b_rwk/`.
The selected test source matches its retained build snapshot; six unrelated
source files and two other test modules differ from the current workspace.
This does not establish ordinary reachability or native Windows, Linux or USA
behavior.

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

A later guarded read-only scan extends the saved-state sample beyond the 14
previous catalog entries. Of 605 retained paths, 345 distinct payloads pass the
EUR ROM identity and state checksum/parser checks; 72 distinct payloads are
non-EUR or invalid for this parser, and remaining paths duplicate payloads.
All 345 have flag `02001f78` bit 0 and gate `0200203d` clear, and no current or
pending dynamic selector 7–9. The word at `0200f450` does not resolve to an
in-range EWRAM task pointer in this sample. This is a saved-value bound, not a
claim that the task cannot be scheduled later, that other pointer regions are
invalid, or that no ordinary use exists. Both older recorded negative scenes
remain closed and were not replayed. Source pins and the protected inputs pass;
successful scratch was retired. Evidence:
`target/validation/units/e128-dynamic-retained-state-context-1z_0p0ni/`.

A subsequent source-pinned trace resolves why the recorded Area List observation
cannot establish the `selection == 2` branch. At dispatch entry `080454e8`,
that recording has `r4 = 3` at guest frame 4620. Original code compares that
register with 2; only the matching path sets flag 40 and requests world state
20, whose table entry queues script key 162. Entry samples cannot expose every
instruction inside a native block. A separate retained one-Down/A Clan input
reaches the visible Clan menu at frame 4759. The first trace's positive-hit
guard failed because none of its selected dispatch entries was sampled; it is
retained as a failed diagnostic. A corrected isolated observer includes the
preceding comparison: it samples `r4 = 1` at `080454c4` on frame 4620, sees
flag40 and gate13 clear at every frame boundary, and reports zero dynamic
owner reads. A separate production CLI run of the same 240-frame input also
reports zero dynamic reads and retains the final image. These two menu items
therefore do not identify a normal input that produces selection 2. Frame or
native-block sampling cannot exclude transitions within a frame or block, and
zero owner reads are a bounded result, not a global absence claim. No full
pair was run. Evidence: `target/validation/units/`
`e128-dynamic-world-state20-source-vcsy0wyf/`,
`e128-dynamic-world-menu-branch-trace-_bkkdyf5/`,
`e128-dynamic-clan-branch-trace-xlx9oirq/` (failed guard),
`e128-dynamic-clan-flag-trace-kdtitdxe/` and
`e128-dynamic-clan-cli-observation-mx0nssxb/`. Inputs and accepted previous
proofs were guarded; test builds are isolated and kept for follow-up.


The entry producing selection 2 is conditionally available when packed flag
`559h` is set (`0200201b` bit 1). In the same authenticated saved-state sample,
63 of 345 distinct valid EUR payloads have that flag set; this does not by
itself prove that their menus are live. Normal input from the IN-08 checkpoint
opened the entry labelled **Link**. Its 240-frame observation samples `r4 = 2`,
sets flag 40, requests world state 20 and script key 162, and reaches the Link
task, with zero dynamic-fragment reads in that window.

From the visible Link menu, separate normal-input selections of **Team Up**
and **Trade Units** each reach callback `08064678`: gate 13 becomes 1 and the
current selector is 1 at subsequent sampled frame boundaries. Both 240-frame
observations report zero dynamic-fragment owner reads. The Team Up screen
then shows an ACCESS search; after 600 further neutral frames it displays
“Link broken. Returning to world map.” This single attempt establishes neither
the cause of the connection failure nor the behavior with a connected peer.
The uninstrumented neutral continuation is not a read-count observation.

The gate activation is a source-backed ordinary task invocation, but not yet
ordinary consumption of selectors 7–9. A positive resource read and its
raw/managed pair remain open, as do the global domain and deferred platforms.
Entry and frame sampling can miss changes within blocks or frames. No
performance measurements were made. Evidence: `target/validation/units/`
`e128-dynamic-menu-flag559-sl3la67j/`,
`e128-dynamic-in08-menu-selection-asumwnjo/`,
`e128-dynamic-in08-link-trace-ng__2ehq/`,
`e128-dynamic-in08-link-endpoint-jc327675/`,
`e128-dynamic-link-dialogue-oa45m74y/`,
`e128-dynamic-link-menu-ksa30jun/`, `e128-dynamic-teamup-trace-379yt0al/`,
`e128-dynamic-teamup-endpoint-kyxi8tfx/`, `e128-dynamic-teamup-wait-jvegqsbs/`,
`e128-dynamic-trade-units-selection-bg0cq6nj/` and
`e128-dynamic-trade-units-trace-3sc2vj6d/`. A prior Link endpoint preflight
failed on a path typo before guest execution and is retained separately.

A guarded source/snapshot join connects the Link registration to another
original callback, `0809a594`, whose dispatcher reads the context pointer at
`0200f4a8`. Its state 26 reaches `08096b72 → 080c4ce4 → 080c3be4`, which can
request selector 8 or 9. Five saved Link boundaries, including the Team Up
ACCESS endpoint and its later failure screen, contain dispatcher state 8 and a
zero context argument at offset 4; the ACCESS endpoint has gate 13 set and the
later boundary has it clear. These are saved boundary values, not a continuous
trace or proof that state 26 cannot be reached. The ordinary transition to that
state remains unproved. All source pins and protected checkpoints passed; no
guest instructions ran. Evidence:
`target/validation/units/e128-dynamic-link-ui-dispatch-source-rvmgtvib/`.

A further guarded scan checks that source-pinned dispatcher root in the same
345 distinct authenticated EUR payloads. In 240 the root value falls within
the tested EWRAM range; none of those saved context fields contains state 24,
25 or 26. The other 105 root values fall outside that range. An in-range value
does not prove a live task or even that the bytes are currently a dispatcher
context; out-of-range values do not exclude other valid regions. These saved
values do not exclude later transitions or any unsampled checkpoint. No guest
instructions ran. Evidence:
`target/validation/units/e128-dynamic-dispatch-retained-states-_9tifc06/`.

A source-pinned path narrows one way to reach dispatcher state 26. In state 1
the original code obtains the context at offset 4 through `080a2f68`; state 4
can queue state 23 only after `08095c10` returns nonzero and `080cbbf0` reports
bit `1000h` set in the pointed object's halfword at offset `28h`. State 23
sets state 24, and a further conditional path in state 24 queues state 26 and
state 76. The queued states and helper results do not prove a normal scene,
their timing or a completed call to the fragment setter.

To look for other ways to enable gate 13, an aligned Thumb BL-pattern scan
found 51 candidate calls to setter `080ccf18` in the EUR ROM. Of these, 27
have consecutive immediate index/value loads immediately before the call. Six
of those pairs name index 13: one supplies value 1 at `0806467e`, and five
supply 0. The 24 other patterns, indirect calls, alternate entry points and
direct memory stores are not excluded by this comparison. The scan can include
data that resembles code; it is not a whole-program control-flow proof.

Two source-authenticated script directories have callbacks that could pass a
data-supplied index to that setter. The declared main EUR archive has 207
programs and 35,048 instructions, none using main opcode 27; the declared
auxiliary archive has 133 programs and 913 instructions, none using auxiliary
opcode 3. A separate source-pinned read of the two mission-record consumers
finds no encoded key `060dh` among the four inline fields of each of the 512
physical EUR mission records. These bounded results do not exclude other
programs, runtime-generated data or indirect calls, and no guest instructions
ran. Protected inputs passed and successful scratch was removed. Evidence:
`target/validation/units/e128-dynamic-state26-source-path-kmcl952k/`,
`e128-dynamic-aux-script-setter-e3wmfre7/`,
`e128-dynamic-main-script-setter-j7bx5tqe/` and
`e128-dynamic-mission-setter-scan-w64t0936/`.

For the source-pinned direct caller of `080d0330`, the five in-range table
branches select indices 14–18. Its out-of-range branch retains the caller's
`r6`; the inspected direct caller supplies either `0839797c` or `00004bfc`,
and the setter narrows that index to 16 bits. Neither case selects index 13.
The helper called along the way saves and restores `r6`. An aligned BL scan
found the one inspected direct call and an exact-byte scan found no Thumb
pointer to the function; neither excludes other indirect entry mechanisms.
Eight further source-pinned normal-entry sites use indices 1/2/4, 12, 20–275
or 27–50, depending on the caller. Those bounds depend on the inspected entry
paths and do not prove the values on alternate entries or corrupted control
flow. No guest instructions ran. Evidence:
`target/validation/units/e128-dynamic-selector-index-domain-l8in_mf0/`,
`e128-dynamic-callee-preserves-r6-tmshu7g4/` and
`e128-dynamic-other-setter-domains-o5w42x52/`.

The Link task's state 5 calls `080cac2c`; a nonzero result tests bit 1 of
`020157e6` and takes the branch that writes state 6. Four authenticated saved
boundaries all have this status byte zero: the two pre-selection Link screens
have task states 52 and 2, the Team Up ACCESS endpoint has state 5 with gate 13
set, and the later Link-broken endpoint has state 52 with that gate clear.
This is not a continuous trace, a diagnosis of the failure, or a proof that a
peer is necessary. It identifies a source-backed condition for investigating a
distinct continuation. Evidence:
`target/validation/units/e128-dynamic-link-state5-status-y6ruykyg/`.

Original `080cabf8` passes object `02015738` to the separate state machine at
`080c9230`; its state 11 sets bit 1 at object offset `ae`, the bit tested by
Link task state 5. One pinned state-2 path queues state 11 after a separate
bit-4 condition, but queued states and intervening conditions remain active.
At the saved Team Up ACCESS endpoint that state machine has state 4, status
byte 0, and gate 13 set; at the later Link-broken endpoint it has state 13,
status 0 and gate 13 clear. The state-4 source tests three IWRAM fields and
has a conditional counter branch that can select state 13. These are two
boundary values, not a continuous trace or a diagnosis of why Link failed.
Source and protected checkpoints passed; no guest instructions ran. Evidence:
`target/validation/units/e128-dynamic-link-bit-producer-b4opp9l0/` and
`e128-dynamic-transport-state2-queue-r0k83gsd/`.

Original `08003f80` reads `04000128` and, on a conditional path, updates the
IWRAM counters at `030029c0` and `030029c2` that transport state 4 tests;
`08004e74` also reads that register. In the two Team Up checkpoints, byte
`04000128` is `33h`; the ACCESS boundary has counter values 0 and 12, while
the later Link-broken boundary has both zero. These are boundary values, not
a continuous trace or a diagnosis of the link failure.

The hash-verified source frozen for the earlier Windows observer routes byte
accesses to offset `128h` through the ordinary I/O backing-store arms in
`MemMap::io_read8` and `io_write8`; the observer executable hash also matches
its build receipt. This observation is limited to those byte dispatch arms and
does not establish all link behavior, the presence or absence of a peer, or a
causal explanation for the observed result. Protected sources, executable and
checkpoints passed; no guest instructions ran. Evidence:
`target/validation/units/e128-dynamic-link-serial-path-hi825gpr/`.

## Conditional fixed OBJ transfer (E135)

The EUR caller at `081449fc` queues a direct halfword DMA from ROM offset
`b80b5c` to VRAM `06010000`, with original control `80004000`: the bounded
source extent is 32,768 bytes through `b88b5c`. It is independent of the
localized selector tables and has no overlap with E134 manifest sources.
The caller instructions, literal words and queue helper passed protected Wine
source checks (`target/validation/units/obj-context-graphics-consumer-wine-rcruc6ln/`,
`target/validation/units/obj-context-graphics-source-wine-9sny93_2/`). This is
a conditional source contract, not an observed ordinary-game scene.

E135 extends the existing raw-transfer model with one separately identified
fixed bank. Localized bank/group usage counters retain their prior meaning;
the fixed bank contributes only raw payload bytes. A named clean-profile Wine
unit verifies CLI create/reuse/verify and one managed startup frame, preserving
102 unaffected E134 files. The original queue and DMA pass full-state/clock
comparisons in both interpreter and translated execution, with the source ROM
span poisoned, the complete VRAM destination checked and machine-state reload
covered (`target/validation/units/e135-clean-profile-first-start-dma-wine-9qxljiuw/`).
The temporary clean profile was drained and removed after PASS; the separate
shared prefix and its interrupted lease were not changed. The startup frame
does not prove ordinary use of this bank. Ordinary gameplay, full UI save/Load
lifecycle, Linux, USA and native Windows remain unverified. No performance
measurement was made.

## Additional context transfer candidates

A read-only review of the retained `081449fc` disassembly identifies three
further calls to the same halfword queue helper `08000ad0`. At calls `08144b48`,
`08144b5a` and `08144b6a`, the literal sources and declared DMA lengths imply
these conditional source ranges:

- `b88c18..b8b9f8`: 5,872 halfwords, 11,744 bytes.
- `b8dcb4..b8e4b4`: 1,024 halfwords, 2,048 bytes.
- `b8e4b4..b8ecb4`: 1,024 halfwords, 2,048 bytes.

The destination values are loaded from live IWRAM and are not classified by
this source review. A read-only comparison with the retained E135 manifest
found no overlap with its declared sources. The reviewed disassembly
`target/rom-data-obj-remaining/new-caller-1449fc.txt` has SHA-256
`deb229e4599db0c7def6929e714658d3cdd16a6c991ffb1669ed5a301992989d`;
the compared E135 manifest has SHA-256
`43f80c4e270b183874a4d7fe37528bcab53c57e04ac3de91f638e70a480d91c4`.
These are bounded static candidates, not new extracted owners or proof of
ordinary execution. Wine validation of the source fingerprints and managed
consumer checks remains open, as do other aliases, live destination ordering,
Linux, USA and native Windows. No guest instructions, new tests or performance measurements ran in
this review.

A further bounded source inspection identifies initializer `0803227c` writing
`06008000`, `06007000` and `06007800` into the three respective live fields.
If those values are still present at the caller, the corresponding DMA
destination ranges are `06008000..0600ade0`, `06007000..06007800` and
`06007800..06008000`. The instruction and literal windows were compared with
the exact EUR ROM and are fingerprinted by the E136 extractor. The preserved
disassembly `target/rom-data-obj-remaining/31c00-disassembly.txt` has SHA-256
`157508a6e6ae8a8b999e68dbcb2e17fba8d7f96b29bf38cb8becde0761e58767`.
Ordering between the initializer and ordinary callers, and intervening writes,
remain unproved. Prepared Wine checks inject these conditional values rather
than claim ordinary reachability.

The caller also has a five-entry dispatch table at `08144a28`, indexed by the
halfword at object offset `2a8`. State 1 enters `08144b3c`, which queues all
three transfers; state 0 enters `08144a3c` and later writes state 1 while saving
state 2 in the object's state stack. If the same object is invoked again with
state 1 and the relevant fields intact, the three queue calls follow; state 1
then restores the preceding state from that stack. This is a conditional source
path, not evidence that an ordinary scene reaches it or that its live
destination fields still hold the initializer's values.

The table pointer and all five destinations were checked against the exact EUR
ROM; instruction windows `144a0a..144a22`, `144af0..144b0e` and
`144b3c..144b8c` have SHA-256 respectively
`c3a6f830dfef381fbf37b35f51b99b6b79c769bc84be493023da3bd347979907`,
`5ecb720fe4ed52ac592a886e670cab4fbed388204cad9e10bd97335e00c7043e` and
`a40b5689a5513eeec8ffd884d601ecaf74e36984dd33420940bd68665438a4b5`.
An eventual Wine observation can distinguish the live path by recording the
object identity, state and three destination fields at those call sites.

A further static link leads to constructor `08144dfc`: it stores `083ae47c`
at object offset `2f8` and the halfword value 1 at offset `2fc`. The word at
`083ae47c` is `081449fd`, the Thumb pointer to the state-machine caller above.
The constructor and table word in the exact EUR ROM have SHA-256
`c14d721c31d00b534a30dce9d28b9e47d41984639ba0ec467c1087fd9a4d0781`
for `144dfc..144e3c` and
`322c20c2476dbcaa4ceb7a498278565523650de82836f97df26e8ea53130ebca`
for `3ae47c..3ae480`. Observing this constructor can provide an object
identity for the later state-1 check, but its ordinary callers and the actual
dispatch sequence remain unverified.

A further conditional predecessor is `08144808`: when its input object's first
byte is zero, its branch at `0814481a` reaches a direct call to `08144dfc` at
`08144828`. The returned object pointer is stored at `0200f3c0`, and the input
state byte is set to 1. EUR ROM window `144808..14483c` has SHA-256
`433c23a1e6f5ab7f06a2c8475da16c7d20af143166f0595536458a1e9493ef76`.
This supplies an earlier conditional observation point, not an identified
ordinary scene or proof that the constructed object reaches state 1 above.

The earlier registration path is also bounded statically. When the byte in
its input context is zero, `080036c4` writes the Thumb pointer `081404e1` to
`03000854`. In state 2, `081404e0` initializes a structure at `0200f3a0` and
uses `08006b40` to place `08142169` into its first pointer slot at `0200f3b8`.
The wrapper at `08142168` directly calls `08144808`, forwarding its second
argument. These are conditional registrations and a direct call; the evidence
does not establish that either registered pointer is invoked in a scene.

For the exact EUR ROM, the SHA-256 values of windows `0036c4..003712`,
`1404e0..1404fa`, `140578..1405ac` and `142168..142176` are respectively
`67bedcc75c48af9fe05aa21872aa996f6dec04f7f5b6e1f0f4b117c9589cffd0`,
`96f809509ae444f8430b994e47e2b3bdeb8b2b65996ddd358f8b2391465191da`,
`f1639ccad6c2ac1cc07edbe815b2d626aa7a1b9efbe0c483d6fb69b71e1698ce` and
`4692e6515a3d9b743841d4077b6601f2672535b89f1112fd315bf26c07e746ab`.
The callback literals at `003738..003764` hash to
`c9f6763ffbadc3c769712ae743f48c2df38416e51387f8bf4174544c6b1b18c5`;
initializer `006ae0..006af8` hashes to
`1326879814188ee0d868326b7b3312d8e9e83a61568852ab66c4c022d0b533f7`,
and pointer writer `006b40..006b5c` hashes to
`adfcb3744861f340a5e92fd533d4d2906eae4c1878541e53f574283adcc0814e`.

On a later branch of `080036c4`, the halfword at offset 4 of the object pointed
to by `030034b8` selects an entry in the `03000830` pointer table; values above
11 select entry 0. If that halfword is 9 and the earlier table assignment is
still intact, the selected pointer is `081404e1`. The branch passes the pointer
to `08006b5c` for storage in another structure; this still does not establish
invocation or identify a game scene. EUR ROM windows `003780..0037c6`,
`0037d8..0037f8` and `006b5c..006b76` hash to
`cf37724bb72e591607959b227218c0f303d310c3666b4d376073fe094e84a8e8`,
`92dea8943fad69d0b3dbe354cad7282b826ae7654d4056402234d2345fb7fe17` and
`f6eadf47b02db91b84275720c4f86f3961e096385b432f4d5ba175d5136a87e7`.

Two conditional paths can request index 9. At `08025374`, a call with argument
`44h` tests bit 4 of byte `02001f78`; if set, `08025382` calls `0800394c`
with 9, which stores that halfword at `030034d2`. Another path at `08140362`
calls `0800392c` with 9, which stores `8009h` at the same address. The
transition logic handles the high-bit form by clearing bit 15 and writing the
result to offset 4 of the currently selected object. These paths require their
own callers, state and subsequent transition processing; neither is proof of a
scene or of the transfer itself. The bit's gameplay meaning remains unknown.

The previously authenticated read-only scan recorded byte `02001f78` for 14 EUR
checkpoint entries representing seven distinct snapshot hashes. All recorded
values were `08h` or `28h`, so bit 4 was clear in those snapshots at scan time.
The retained result is
`target/rom-data-dynamic-ordinary-reachability/snapshot-peek-flag40-corrected.json`
(SHA-256 `78403f4b664a865d33a756a4d55af020084dfcce4d2c0e10181c1763cf0612e8`).
This rules out the bit-gated request at those exact captured values only; the
bit can change later, and the separate `8009h` request is unaffected. No new
snapshot, replay or runtime test was needed for this comparison.

The bit-gated request is not by itself a reliable route to the transfers.
In state 1, callback `081404e0` reads the state produced by its preliminary
callback; if flag `44h` is set and that state is 2, it writes `ffh` instead
of proceeding to the state-2 registration. Separately, completion at
`081418bc..081418c4` writes 2 to `0200f390`; what makes ordinary execution
reach that completion remains unknown.

An earlier startup path provides a better candidate for observation:
initialization at `080036c4` registers callback `081401e9` in slot 11 and
sets the initial selector to 11. That callback can request `8009h` at
`08140362`. When `030009c4` is initially zero, its first-entry path invokes
`0813fe44` and subsequently waits for `0813fecc`. The source does not prove
that this completes without input or that a cold start reaches the transfers.
Once Wine is available, an ordinary EUR cold-start observation can record the
selector and pending transitions, readiness at `0813fecc`, and hits at
`081401e8`, `08140362`, `08140578` and `08144b3c`, without forcing guest state.

A bounded observer for that next check has been cross-compiled for Windows
(`target/validation/units/e136-cold-observer-build-1iux4706/`); its executable
and protected build-source hashes were checked after the build. It is prepared
to run 180 frames of the raw EUR HLE interpreter with all keys released,
capture selected executed Thumb PCs and frame-boundary selector samples, and
retain the final raw GBA framebuffer. Its event limit and dropped count are
explicit. The launcher is `target/rom-data-e136-next/run-cold-observer.py`.
No Wine observation has run yet. This raw, no-input observation would identify
a candidate path or prompt; it would not establish managed E136 consumption,
translated execution or other scenes.

The retained disassembly further narrows the first-entry wait: `0813fecc`
decrements an initial countdown in state 0 before entering state 1. In state 1,
the branch at `0813ff40` tests `03000002 & 0009h`; when nonzero it sets the
local state to 2 and a countdown to `20h`. The state-2 code returns ready once
that countdown reaches zero. This source path supports observing the prompt
and input state, but does not prove an ordinary startup reaches it or establish
which on-screen choice should be made. Other writes and paths remain possible.
Exact EUR windows `13fecc..13ff32`, `13ff3c..13ff72` and `140190..1401b0`
hash to `6ff9770c4ad9728839b7666cf842d6adbdda88611ae35986022edd5bbc8724cc`,
`60eb6b89acb5e7771bfb9f7f1e571008355334ce32bd703264c89ef0e5238582` and
`0ac19226af636b1d157d79e25381b222aa89a2699254ce8f085dab0810cc9ad7`.

The exact EUR ROM windows `00370e..003736`, `1401e8..140252`,
`14054e..140570` and `1418bc..1418c6` hash respectively to
`1fd62decbffe197e240ff7d173a1cab57bddbab1f930c0f50a74d8a1b2ba988e`,
`33338fe42240c2a81311114c16e0940ce70071f3ed31adb6f2e46f82af48a5e5`,
`7f874eb9793923a72ac4b283341487d2cf30e8ca5464c154e91114cbf02557b4` and
`52a1146b3fb3060eebd2ab75394c1d2b0860f430faed3459f0e29817f2035f5f`.
The bounded source interpretation was independently suggested by the focused
Sol consultation and checked against these ROM bytes and call targets.

A subsequent protected, read-only inspection of those same 14 entries
(`target/validation/units/e136-eur-index-checkpoint-inspect-dubn20o1/`)
found current indices 1 or 2, pending value `ffffh`, and the expected slot-9
pointer `081404e1` in each snapshot. These are observations at capture only:
they do not exclude a later index-9 transition or show invocation. The ROM
identity, payload checksums and recorded snapshot hashes were checked; the
inputs remained unchanged. This inspection did not execute guest code or
replace the pending Wine tests.

EUR source windows `025374..02538e`, `0cce98..0cceca`, `00392c..003962`,
`003884..0038a6`, `0038c8..0038d2` and `140354..140366` hash respectively to
`5a844a34260abde4eb6bc8dec36cadd42f8d5498e12274e5f7be7e54fd6720a5`,
`d7f7e83cf5a4610e8ba85a212bb0751b73c350ac13064a4775fc9e919e682ab4`,
`6441820a2043d959d2e378d1b1d2e3e6fd3dc7083e57d7b38aad5e6bfc2d10a0`,
`45f6de5347b4b62b543a97a37c5071195e30b888bef6e2677c9927ad643677f9`,
`46fb8582eddbd655be2fa3524e972a70282b7a245b64aa5f137d9a2414f39cd5` and
`0d8ce76124a0033287fb51b9f5dc170387e5ad932832ab3a4745a95ffbc1b553`.

E136 adds these three views to the existing fixed raw-transfer model, with
separate `fixed-context-transfer` provenance roles; the E135 fixed OBJ role and
the localized group identifiers remain unchanged. The exact EUR ROM identity,
source ranges, source hashes, caller instructions and literals are checked by
the extractor. Their destination semantics are deliberately left to guest code.
The implementation and its focused Wine test are integrated but not yet
verified; E135 remains the accepted current dataset. Original and translated
runtime, the production CLI, ordinary use, Linux, USA and native Windows remain
open. No performance measurement was made.
