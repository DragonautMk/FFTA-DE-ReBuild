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
shared prefix and its interrupted lease were not changed. The startup frame alone does not prove ordinary use. A later ordinary Wine
startup after English selection reaches the original queue call with source
`08b80b5c`, destination `06010000` and halfword count `4000h`
(`e135-ordinary-observer-wine-s462xxqw`); the state immediately before that
call is retained. From this checkpoint, raw/managed pairs with both the
interpreter and translated original queue/DMA pass exact 32 KiB VRAM comparison
and three full frame state/clock comparisons with the managed ROM source
poisoned (`e135-ordinary-pair-wine-jjgeygm9`). Each run records 48,608 raw
transfer payload bytes in aggregate, including the subsequent E136 transfers.
Other scenes, full UI save/Load, Linux, USA and native Windows remain open. No
performance measurement was made.

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
At this initial review they were bounded static candidates, with no proof of
ordinary execution. No guest instructions, new tests or performance
measurements ran in that review; the later E136 validation is recorded below.

A further bounded source inspection identifies initializer `0803227c` writing
`06008000`, `06007000` and `06007800` into the three respective live fields.
If those values are still present at the caller, the corresponding DMA
destination ranges are `06008000..0600ade0`, `06007000..06007800` and
`06007800..06008000`. The instruction and literal windows were compared with
the exact EUR ROM and are fingerprinted by the E136 extractor. The preserved
disassembly `target/rom-data-obj-remaining/31c00-disassembly.txt` has SHA-256
`157508a6e6ae8a8b999e68dbcb2e17fba8d7f96b29bf38cb8becde0761e58767`.
Ordering between the initializer and ordinary callers, and intervening writes,
remain unproved. The Wine checks inject these conditional values rather than
claim ordinary reachability.

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
The EUR cold-start observation below records selector and pending transitions,
readiness at `0813fecc`, and selected PC entries without forcing guest state.

A bounded observer for that check was cross-compiled for Windows
(`target/validation/units/e136-cold-observer-build-1iux4706/`); its executable
and protected build-source hashes were checked after the build. It ran 180
frames of the raw EUR HLE interpreter with all keys released under Wine,
capture selected executed Thumb PCs and frame-boundary selector samples, and
retain the final raw GBA framebuffer. Its event limit and dropped count are
explicit. The launcher is `target/rom-data-e136-next/run-cold-observer.py`.
The no-input run (`e136-cold-observer-wine-_kyjkycg`) reached the language
selection screen with English highlighted. Selector 11 was observed at frame
9, and readiness returned zero on all 170 observed returns; none of the
selected transfer-path entries was reached. No events were dropped. This raw
observation does not establish managed E136 consumption or other scenes.

An independent raw EUR Wine continuation repeated that observed cold-start
prefix, pressed A on the highlighted English choice during frames 180–181 and
then released all keys (`e136-select-observer-wine-mri3o4xu`). The readiness
return became 1 at frame 213. At frame 360 the raw framebuffer showed the
Square Enix splash; the sampled selector remained 11, no selected transfer
entry was reached and no events were dropped. This is a bounded ordinary-input
observation, not proof that later startup cannot reach the transfers. Inputs
and the prior no-input evidence remained protected.

A longer Wine observation of the same input prefix extended it to frame 1200
and retained its exact endpoint (`e136-long-observer-wine-jt59h5bk`). It
observed the `8009h` request at frame 466 and selector 9 at frame 468; none of
the selected transfer entries was reached, and no events were dropped. The
final raw framebuffer was nearly black. The retained endpoint is for a bounded
continuation, not evidence that the transfers are unreachable. Its ROM, source,
prior observation and build inputs remained protected.

Continuing that endpoint for 600 frames under Wine with no additional input
(`e136-resume-observer-wine-uxsheu99`) retained selector 9 and context state 1
at sampled frame boundaries. The original `081404e0` and `08140670` entries
were observed, but none of the selected transfer entries was reached; no events
were dropped. The final raw framebuffer showed two introductory image panels.
This bounded result does not establish what later inputs or frames do. The
original endpoint stayed unchanged and the continuation endpoint was retained.

A further no-input Wine continuation from that endpoint
(`e136-timeline-observer-wine-sjr5o0bg`) followed the automatic callback
timeline. At the third transfer call, the raw interpreter had observed all
three queue calls with these live source, destination and halfword-count values:

- `08b88c18`, `06000000`, `16f0h`.
- `08b8dcb4`, `06009000`, `0400h`.
- `08b8e4b4`, `06008000`, `0400h`.

These destinations differ from the conditional initializer values used in the
earlier isolated tests. The calls occurred after the ordinary English selection
and automatic timeline, with no further inputs or dropped observations. A
machine state immediately before queueing and the state after the third call
are retained in that unit. The ordinary interpreter pair
(`e136-ordinary-pair-wine-ez7huad9`) confirms all three queue arguments and
the exact resulting VRAM ranges with the managed source ROM spans poisoned.
All three complete frame states and clocks match the raw execution; managed
reads account for the 15,840 source bytes. The corresponding Wine pair with
translated original queue/DMA code also passes on the same checkpoint
(`e136-ordinary-translated-wine-e2v3_rjg`), including the poisoned sources,
exact VRAM, managed reads and three complete frame state/clock comparisons.
Other scenes remain to be verified.

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
inputs remained unchanged. This inspection did not execute guest code.

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
EUR Wine verification passed: `e136-context-transfers-wine-sqk59qhr` checks all
15,840 source bytes, the E135 predecessor and three original queue/DMA cases;
`e136-context-transfers-native-wine-i4lkp88p` checks the three translated
queue/DMA cases after reload. Both compare full machine state and clocks and
use injected conditional VRAM destinations. Production CLI create, reuse,
verify and one-frame managed startup passed in
`e136-context-transfers-cli-wine-jtdsittv`, with 102 unchanged prior files.
The current E136 dataset is retained in the first unit. A Wine lifecycle test
copies the dataset into isolated scratch, makes the copy unavailable, and
confirms failed reattachment is transactional. Slot load, Undo, backup restore
and restart retain the same resource owner and still serve all 48,608 fixed
transfer bytes with ROM sources poisoned
(`e136-context-lifecycle-wine-5ncguagd`). This covers the backend operations;
the frontend UI, other scenes, Linux, USA and native Windows remain open.
No performance measurement was made.


## E428 European 71e6f8 bounded bank plan

Protected source plan `e428-eur-71e6f8-whole-bank-source-plan-8cnwq6rl`
reconciles all five EUR pointers in the `71e6f8` group with E298. Each
source-rooted physical interval is 896 bytes: `441288..441608`,
`4528fc..452c7c`, `464498..464818`, `475460..4757e0` and
`486f30..4872b0`. Their ends coincide exactly with independently rooted
existing resources: four indexed archives and the map raster palette bank.

Each interval's `0100..0200` bytes already belong to
`graphics.dynamic-raw-fragments`, and `0200..0380` to
`graphics.ui-raw-transfers`. All five `0000..0100` prefixes are unowned:
1,280 bytes out of 4,480 bounded bank bytes. The join validates the existing
source hashes and no overlapping owner for every byte. Four prefixes have the
same source hash; the fifth differs. Their words can be preserved as opaque
source data without inventing visual semantics or guest selector bounds.

The original fixed and dynamic readers establish the linked transfer views,
not ordinary use of every prefix or a global live-index domain. Neighbor
roots establish these physical interval ends, not every semantic relationship
between adjacent data. E428 is a source plan only; integration, other readers
and scenes, Linux, USA and native Windows remain open. No performance
comparison was made.

## E429 European five bounded banks runtime

EUR extractor 299 / decoder 298 adds the five previously unowned 256-byte
prefixes to `graphics.dynamic-raw-fragments`. Together with the existing
dynamic and fixed transfer owners, the complete five E428 physical intervals
now comprise 4,480 managed source bytes. Their independently rooted successors
retain their existing owners. Source authentication preserves all previous
fragment fields and the other 147 resource records and files.

Wine reader `e429-localized-whole-bank-readers-wine-qa7y9h7g` passes all bank
bytes with their ROM backing altered, including all three ROM mirrors, mixed
owner boundaries and the fifth bank's page boundary. It rejects a coherent
source edit and preserves the attached owner through a missing dataset and
snapshot restore. Thirty controlled original transfer cases cover selectors
4–6, both flag branches and all five language slots; their source offsets cover
all four quarters of each new prefix. Original interpreted, resource-backed
interpreted and generated execution agree on complete state and guest clock.
The original selector tables are still read from the ROM for those cases, and
the existing raw guest index calculation remains unchanged.

Wine production CLI `e429-localized-whole-bank-cli-wine-4j2h1iqg` creates,
reuses and verifies 149 files matching the reader-tested generation. The
generation and its files remain unchanged on reuse and verification. Both Wine
servers drained, and their logs contain no D-Bus assertion with the local
overlay.

Physical intervals and controlled transfers do not establish ordinary gameplay
use of every prefix or a global guest selector bound. Other consumers and
scenes, real BIOS, Linux, USA and native Windows remain open. No performance
comparison was made.

## E430 European remaining raw bank census

Protected source census `e430-eur-localized-raw-other-banks-census-ynlrsmfq`
reconciles the other six declared pointer groups against the exact EUR ROM and
the E299 manifest. All 30 pointer values and their existing original-transfer
views are source-authenticated; those fixed views total 36,160 bytes. For each
view the census records the next greater target among the 100 retained E427
pointer values and partitions the interval by current source ownership.

All five `71e608` transfer views reach that next known pointer without a gap.
The 25 other intervals contain 115,352 bytes in total: 101,587 currently have
no owner and the remainder already belong to other resources. In particular,
the five `71e66c` intervals include independently owned custom and indexed
archives. Three other groups have equal per-language intervals after their
fixed views: 68 bytes for `71e5f4`, 1,168 for `71e694` and 20 for `71e6e4`.
These observations identify candidates for reader and format investigation;
they do not establish that any interval belongs to the preceding bank.

The census finds candidate physical neighbors, not a global inventory or
logical bank ends. It does not prove the meaning or ordinary use of unowned
bytes, and it does not authorize merging the neighboring owners. More original
reader and root evidence is needed before integrating additional ranges.
Linux, USA and native Windows remain pending; no performance comparison or
gameplay replay was performed.

## E431 Repeated-gap pointer candidates

Protected census `e431-eur-localized-raw-gap-roots-z5tr1bdh` checks all
4,194,304 aligned little-endian words of the exact EUR ROM for `08`-region
values into the 15 equal-sized candidate gaps of E430. Thirty consecutive
values at `397fbc..398034` target the five `71e694` gaps: six per language,
with successive targets 192 bytes apart. A further candidate value into a
`71e5f4` gap lies in an already owned packet program; no candidate targets
the `71e6e4` gaps. The other five `71e5f4` and five `71e6e4` gap contents
have identical hashes within their respective groups.

The 30 consecutive values are a promising source catalog, but the pattern by
itself does not prove a live pointer table, a reader, 192-byte resource lengths
or ownership of the 16 bytes after six such intervals. The scan covers aligned
values only; unaligned, indirect and computed references and other address
mirrors remain open. Original reader/root evidence is required before claiming
or integrating the candidate resources. Linux, USA and native Windows remain
pending; no gameplay replay or performance comparison was made.

## E432 Thirty localized queued-transfer resources

Wine disassembly `e432-eur-gap-catalog-disassembly-wine-a3h2cux_` identifies
the original selector at `0809279c..080927e2`. It indexes the pointer table at
`397fbc` and the parallel length table at `398034` with the live language byte
and a task byte, using `language * 6 + selector`. For the five physical
languages and six listed entries, all 30 lengths are 192 bytes. The original
`08000be4` helper and queue executor use four rows of 24 halfwords, with
48-byte source and 64-byte destination strides. This describes the selected
source projection; it does not impose a bound on the original index arithmetic.

Protected source plan `e432-eur-localized-thirty-resource-source-plan-it7ywjrs`
authenticates the 30 pointers, both 120-byte tables, two original root literals,
reader instructions and five contiguous 1,152-byte payloads against E299. All
6,008 selected source bytes are unowned at that revision. The original guest
selector and queued transfer remain the intended runtime consumers; the plan
does not yet publish them. Sixteen following bytes per language remain outside
the selected projection. Ordinary invocation, other indices/readers/scenes,
Linux, USA and native Windows remain open. No performance comparison was made.

## E433 Thirty-resource queued catalog runtime

EUR extractor 300 / decoder 299 publishes the complete E432 source projection
inside `graphics.dynamic-raw-fragments`. The catalog retains both original
tables, their root literals and all five contiguous banks: 30 resources of
192 bytes, adding 6,008 authenticated source bytes. Existing fragment fields,
147 other resource records and their files remain unchanged. The original
uncapped index arithmetic, queue registration and DMA execution remain guest
work; reads outside the declared spans continue through earlier owners or ROM.

Wine reader `e433-localized-queued-catalog-readers-wine-z19k6u4o` verifies all
new bytes on ROM mirrors, mixed boundaries and page crossings with the backing
ROM altered. It rejects a coherent source edit and preserves the attached owner
across a missing dataset and snapshot restore. For all five languages and six
entries, interpreted and generated original selector paths match complete
state and guest clock; the original interpreter selector, queue and DMA copy
all 30 resources correctly. The Wine server drained.

Wine production CLI `e433-localized-queued-catalog-cli-wine-0vd50_u1` creates,
reuses and verifies 149 files matching the reader-tested generation. The
generation and files remain unchanged on reuse and verification. Both Wine
servers drained and their logs contain no D-Bus assertion with the local overlay.

Ordinary invocation, other readers and scenes, unrestricted guest indices,
the 16 following bytes per bank, Linux, USA and native Windows remain open.
No performance comparison was made.

## E434 Larger-gap pointer candidates

Protected census `e434-eur-large-localized-gap-census-xuhbm61i` reconciles
the unowned portions of ten larger E430 intervals with E300. Across 95,307
unowned bytes it finds 66 aligned little-endian `08`-region values, including
four consecutive five-language candidate tables at `39554c`, `3955d8`,
`397d94` and `3adfb0`. All four table spans are unowned in E300. The first two
target gaps after `71e66c` transfer views and the other two target gaps after
`71e6a8` views; two groups point exactly to the existing view ends.

These are candidate address patterns, not established readers, logical bank
ends or proof that all intervening bytes share an owner. Other owned archives
within the larger intervals remain separate. Unaligned, indirect, computed
and other-mirror references, ordinary use, Linux, USA and native Windows remain
open. No gameplay replay or performance comparison was made.

## E435 Four original-reader classifications

Wine disassembly `e435-eur-four-localized-table-disassembly-wine-lgzvdwq6`
and protected classification `e435-eur-four-localized-family-classification-aigv45cr`
identify original language-selected readers for all four E434 tables. Three
groups use indexed wrapper `0800537c`: `39554c` selects five archives whose
headers declare 30 programs each; `3955d8` selects five declaring
225/249/231/235/278; and `397d94` selects five structures with an archive at
offset `b4`, each declaring 44 programs. Their dictionary headers declare
mode 0 / width 5; decoded program sizes are 64, 64 and 256 bytes respectively.
The 15 headers declare 1,588 programs in total.

The fourth group, `3adfb0`, selects five streams used by the original custom
decoder `080054a0`; each declares 1,664 output bytes, matching the following
copy control. These decoder calls and source headers establish candidate
families, not the full consumed source extents or successful decoding of every
program. The prefixes before the five embedded archives are separate sources.
Ordinary invocation, guest indices, Linux, USA and native Windows remain open.
No performance comparison was made.

## E440 E301 ownership of the remaining gaps

The `audit-ownership` CLI compared all 25 nonempty E430 intervals against
the E301 manifest on Wine (`e440-eur-raw-gap-residual-wine-d8smp7pd`). The
same intervals had 101,587 unowned bytes in E299 and now have 3,607. The
E438 plan accounts for 3,037 of those bytes as five unpublished stream
payloads. Its 24 bytes of selector and caller metadata are outside these
intervals.

The other 570 bytes break down by E430 group as follows. `71e5f4` has 340,
`71e66c` has 38, `71e694` has 80, `71e6a8` has 12 and `71e6e4` has 100.
The evidence records the exact residual spans. Assigning these intervals
requires independent evidence of a source or padding boundary. This check
uses retained metadata and establishes no new runtime or gameplay coverage.

## E441 Shape of the 570 unattributed bytes

The static census `e441-eur-localized-raw-residual-census-retry-xbrt1szy`
checks the exact EUR ROM and E301 manifest for all 41 residual intervals. In
26 intervals, 50 zero bytes extend a bounded source end to the next 4-byte
boundary, where a separate source begins. The other 520 bytes form fifteen
intervals. Each language has a 68-byte `71e5f4` gap, a 16-byte `71e694` gap,
and a 20-byte `71e6e4` gap. The 68-byte and 20-byte contents are identical
across all five languages within their respective groups. The 16-byte
intervals have four distinct hashes.

A census of little-endian words at all four byte phases and the three ROM
mirrors finds five values in the exact residual ranges. It reproduces the
applicable E431 hit. Three of the five word cells intersect existing resource
sources. The other two cells, `6d8f38` and `b176bf`, point into a `71e5f4` gap
if interpreted as addresses. Original reader evidence is still needed for
both. Zero fill and alignment describe the physical layout. They do not
establish that guest reads skip those bytes. The census publishes metadata
only. Runtime, Linux and USA validation remain open.

## E442 Context of the two uncovered address-shaped words

Static inspection `e442-eur-localized-raw-residual-pointer-context-engiyyrd`
checks both word cells from E441 against the exact EUR ROM and E301 manifest.
Neither occurs in a consecutive five-language table at the same relative
offset in each `71e5f4` interval. A full-ROM search for exact addresses of
the cells finds no words in the three ROM mirrors. The aligned cell has no
matching immediate Thumb or ARM PC-relative word load in the encodable
distance. The other cell is unaligned and cannot be the target of those
aligned literal loads.

No selected evidence identifies an original reader for either cell. A live
or source-anchored indirect reader is still required for attribution. Computed
references, other encodings, runtime use, Linux and USA remain open.

## E528 European startup callback table

The original boot code loads the Thumb installer pointer at `0800058e`
and registers it through `08006b40`. The installer at `080036c4` reads
the IWRAM table base at `080036e0`. It loads all twelve consecutive
callback words from `00373c..00376c` and stores them at `03000830..03000860`.
The last store runs at `08003710`. The code addresses authenticate the
table extent and the order of its slots.

Revision `e324-d323` serves the twelve slots and the two boot literals at
`0005c0` and `003738` through `engine.thumb-pc-literals`. The 56 bytes in
source order have SHA-256
`f9250dbfbd1c4994c192a32d7451c6e0aa37b44053f3f0808ceb1a1b3e205a1a`.
The later lookup at `080037b2` already uses the same resource at `0037f0`.
Its source record and the rest of the predecessor manifest are preserved.

The single Wine test `qhrwrz0g` finishes the original installer on frame
8 with BIOS HLE and neutral input. It traces 14 raw reads in the reference
and 14 typed reads at the same instructions with the selected ROM backing
altered in the candidate. All 48 IWRAM table bytes, guest clock, audio buffer
and full saved state agree. The test also attaches the exact `e323-d322`
predecessor, verifies the prior lookup and rejects `e324-d323` with the new
group removed. The failed attachment preserves the active owner.

Build `b_vm92ph` passes and the Wine server drains. The verified manifest
SHA-256 is `f40de38596eb6231777cd920dc32d5eb39181dad01e4549f2cd0ed0cd3360a2e`.
E530 verifies dispatch to slot 5. Other selector values, translated execution,
original BIOS, Linux, USA and native Windows remain open. No performance
measurements were taken.

## E530 European startup dispatcher literal pools

The original dispatcher at `080036c4` loads its 12 callback slots before
running the selection and transition logic. The census covers the remaining
dispatcher instructions and the ten adjacent routines that reset, query or
submit requests, ending before `08003a6c`. Their 58 PC-relative word loads
address the queue fields, task context, initializer pointer and request
sentinels. Nineteen words already belong to `engine.thumb-pc-literals`.
The word at `370eb0` used to call the initializer has its existing owner in
`engine.iwram-memory-helpers`.

Revision `e325-d324` appends the remaining 39 physical words, totaling 156
bytes, to the startup literal group. Extraction authenticates the exact EUR
ROM, the dispatcher and helper window, the original load instructions and
the values. The 156 bytes in source order have SHA-256
`9ce5b5878698a02d487e06cfe9fae1f011d70d2315c6eda897ff46a85d383950`.
All earlier source records and unrelated resources match `e324-d323`.

Wine `9xc0yybf` reaches the original title callback `08139508` at frame 741
with slot 5 selected. In the ordinary comparison, 1,468 typed requests at
26 original sites match the raw reference, including 13 newly owned words.
Both runs agree on the guest clock, audio and full saved state with the new
ROM spans altered in the managed run. A controlled pass executes every one
of the 39 original loads and verifies its value and typed service. The exact
`e324-d323` predecessor attaches. Presenting its literal packet as `e325-d324`
is rejected without replacing the active owner.

Build `bu7hs4zv` passes and the Wine server drains. The verified manifest
SHA-256 is `1e10285efbf1fa439714cc7d4cfe7cdb1527cdac8f2072532241a77da3600920`.
Full execution of the other selector and request branches, translated
execution, original BIOS, Linux, USA and native Windows remain open. No
performance measurements were taken.

## E531 European title callback literal pools

The slot-5 callback at `08139508` registers `08139d29` through the original
task writer. Following both sides of the direct branches within the parent
and child finds 14 parent words and 55 child words. The 71 original load
instructions include two readers each of the shared words at `139efc` and
`13a1cc`. All 69 physical words lie outside the `e325-d324` ownership map.

Revision `e326-d325` appends both pools to `engine.thumb-pc-literals` and
adds page `13a` to the literal lookup. Extraction authenticates the exact
EUR ROM, both callback bodies, the physical source words and the primary
reader of each word. The 276 bytes in source order have SHA-256
`74ec44a8dda83c62dcb4d1ad14e8340a9cb2467f56135d8cd36b22217001c3f9`.

Wine `22985_8p` reaches the actual slot-5 parent at frame 741 and compares
both configurations from that in-memory checkpoint. The original writer
installs `08139d29` at `0200f3bc`, and the child reaches the state-3 loader
at `081399e4` on frame 743. With the 69 source words altered in the managed
ROM, its 16 typed requests match the raw reference at 15 original sites.
The guest clock, audio buffer and full saved state agree. A controlled pass
executes all 71 original loads with the altered backing, including both
readers of each shared word. The exact `e325-d324` predecessor attaches.
The incomplete current packet is rejected without replacing the active
owner.

Build `12nl9ie3` passes and the Wine server drains. The verified manifest
SHA-256 is `a50a73f81fc4fee5761e18d03864256d77ee27cfc19a40b89bf3d47a4fd3e886`.
Ordinary use of the remaining callback branches, translated execution,
original BIOS, Linux, USA and native Windows remain open. No performance
measurements were taken.

## E532 European title frontend source bank

Starting at the two E531 callbacks, the source census follows the original
direct branches and calls within `139508..13ba90`. It also follows every
destination of the two guarded switches. This identifies 33 routines and
347 original literal loads of 314 physical words. Eighty-nine words have
earlier literal or UI transfer owners. The switch at `0813ad1e` selects six
title state destinations from `13ad2c..13ad44`. The switch at `0813ba2e`
selects five stored-language destinations from `13ba3c..13ba50`.

Revision `e327-d326` adds 225 words to `engine.thumb-pc-literals` and all
eleven destinations to `engine.thumb-switch-tables`. The new source spans
total 944 bytes. Extraction authenticates the original EUR code and source
bytes, including each switch's original guard. It preserves the two table
base words at `13ad28` and `13ba38` as literals. It authenticates the literal
bank's 900 bytes in source order with SHA-256
`232b936886973cae169db965b7dea2755385188bec3432951efec63a8b680f53`.

Wine `uor3beuv` follows the E529 English confirmation route and reaches the
original opening routine at `0813a1dc` on frame 3047. Both configurations
start from that ordinary checkpoint and reach the end of the 19-position
loop on frame 3104. With the 944 bytes altered in the managed ROM, all 540
typed requests at 18 original sites match the raw reference. Guest clock,
audio and full saved state agree. Controlled execution verifies all 256 new
literal readers, all eleven switch destinations and four out-of-range cases.
The original fallback branches issue no requests for either switch's sources.

The exact `e326-d325` predecessor attaches. The incomplete `e327-d326`
dataset is rejected without replacing the active owner. Build `422clx82`
passes and the Wine server drains. The verified manifest SHA-256 is
`5d08a717d367b11917de1dcf3dc6606a03a3f2813958f273b3efbc9216cc613a`.
Ordinary use of the remaining menu paths, translated and original-BIOS
execution, Linux, USA and native Windows remain open. No performance
measurements were taken.

## E538 European tilemap rectangles

The original drawing routine at `0802c6e0` reads a row-major rectangle of
little-endian halfwords. Its callers supply the dimensions. It clips the
signed horizontal position to columns 0 through 30. The seven aligned
direct calls include four from live world-map buffers, two from staged RAM
templates and one with a ROM source at `0812ee54`.

The two staged templates occupy a contiguous bank at `3b0e5c..3b0f44`.
The 10-by-8 template starts at `3b0e5c` and the 12-by-3 template starts
at `3b0efc`. The original initializers at `08071510` and `080717c8`
add `0xc0` to each tilemap entry and write the templates to the two
RAM records selected at `0806b210`. The second initializer appends two
zero columns to each row in RAM. A separate draw path at `0806a5c0`
uses both ROM templates directly with a tile base of `0x300`.

The 22-by-17 rectangle at `3ba95c..3bac48` is the ROM source of the
draw at `0812ee54`. Its loader starts at `0812edf4`. The five E516
CPUSet call sites in that loader use decoded graphics from `3bac48`
and `3bade0`, a 768-byte view selected through the `71e66c` language
directory, and two existing color-mode palette blocks. The original code
temporarily sets the tilemap stride to 32 before drawing and restores it
afterward.

EUR `e333-d332` adds `halfword_copies.tilemap_rectangles` to
`graphics.ui-raw-transfers`. The new spans contain 490 halfwords and
six caller words, including the word selecting `71e66c`. The source
hash of the nine spans in order is
`94f4feacf27ec17822a2aa56fb0f195e36532326221e0ce797dd75300eefcfce`.
The source plan with the reader windows and previous owners has SHA-256
`0670cf0c5007482e34c1752a951162eb05e4277e7ec50cfa053e8738411f95aa`.

Windows build `3ckb9xrl` and Wine unit `z2k72eer` pass. The controlled
interpreter runs the original direct draw, both staged initializers and
draws, and five cases of the localized loader. The cases cover all five
stored languages and all four color-mode selector values. The new source
ranges and selected earlier graphics, raw and palette ranges are altered
in the managed ROM. All 2,116 rectangle and pointer reads match the raw
reference. The exact layer buffers, padded RAM rows, horizontal clipping,
three VRAM transfers and two palette copies agree. The raw-transfer
counter records the three rectangles and five localized banks.

The unowned bytes at `3b0e5b` and `3ba95b` continue to use raw fallback
on all three ROM mirrors. Guest clock and complete saved state agree.
The exact `e332-d331` predecessor attaches and the incomplete current
generation is rejected transactionally. The Wine server drains. The
manifest SHA-256 is
`d5aef3344336933e58aa1a7691a35f51a23017962f44e22fad29dd659159e997`.

Ordinary scene entry, additional producers and indirect calls, translated
execution, original BIOS, Linux, USA and native Windows remain open.
No performance measurements were made.
