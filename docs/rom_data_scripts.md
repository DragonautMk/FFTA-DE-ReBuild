# European script resources

The bounded EUR source projection contains 115 six-byte command records at
`3acebc..3ad16e` (690 bytes). Each has a little-endian u16 stream advance and an
unaligned little-endian u32 callback at +2. All 115 callbacks are distinct, non-null
ROM Thumb entries. Their low bit is preserved for the original BX trampoline;
they differ from the even MOV-PC dispatch words in the
[effect/event directory](rom_data_events.md).

## Boundaries and original consumers

The original interpreter at `081265cc` reads a signed 16-bit stream cursor,
adds it to the live stream pointer, reads an unsigned byte opcode and computes
`3acebc + opcode*6`. Four byte loads at `081265e4..081265f2` assemble the callback.
The call goes through `08145c14` (BX r3). The byte loads at `08126600/08126602`
assemble the advance, which is added to the live cursor and stored as a halfword.
A callback result of `0x110` repeats the interpreter loop; other results return.
None of this control flow is replaced by the host model.

Two independent callbacks root a different byte lookup at `3ad16e` through
literals `1263c4/126468`. This bounds the 115 command records without inferring an
extent from the appearance of callback pointers. An initial 116-record candidate
ending at the existing selector root `3ad174` was rejected before the source plan
or model was created: its last record would include the distinct byte lookup.
The lookup and intervening bytes remain outside this projection; its complete
semantic domain has not been established.

Relative commands 0 and 1 separately read the same advance field, add their stream
operand and truncate the target to 16 bits before the original call. These readers
are at `08126628..0812664c` and `08126664..08126688`.

The source projection is not a guest opcode bound. The original unsigned byte
opcode arithmetic has no local 115-entry cap; values 115..255 must retain reads
from the existing owner or raw ROM. The signed cursor, u16 wrap, callbacks,
return values, live operands, waits and sequencing remain original. Observed
advance values 1..10 do not create a new allowed-value restriction.

## Typed model and current scope

`rom_data::script_commands` stores a canonical opcode, u16 advance and typed
Thumb callback per record. Strict JSON rejects unknown fields, overflowing
integers, reordered or missing records and malformed callback offsets. It
reconstructs all 690 source bytes without keeping a ROM-byte shadow. Source
authentication requires the exact European ROM, 106 original-instruction halfwords
and six root/successor words, plus exact source reconstruction and SHA-256.
The range is disjoint from all 43 published e49 resource owners; this directory
has no asset dependency because its callbacks refer to executable code.

A warning-free Windows test build and one corrected Wine source oracle pass:
117 callback-load prefixes, 117 advance prefixes (including raw opcodes 115/255),
115 BX transfers, four signed-cursor prefixes, ten relative-target prefixes and
five callback-return loop branches. All 690 owned source bytes are observed at
original load sites. Seven shape, four source-edit, six JSON and two ROM-identity
guards pass. The 2,905 instructions and 67 PCs count coverage only.

The first Wine attempt failed a test expectation: the original ADDS instruction
at `12660a` writes r0, while the test expected the sum in r1. Only those expected
register values were corrected, then the affected Windows test binary and the
same new filter were rerun. The failed evidence is retained. No production code
or typed-model correction was needed, and no old family proof was repeated.

The source/model criterion above is complete. Managed publication/direct guest
consumption are implemented below; native integration, production CLI and
positive ordinary use have separate completed checks below. USA stays at e8-d7. New Linux,
USA and native Windows checks remain pending. No performance or space
measurements are part of this work.

Private, reproducible local evidence is under `target/rom-data-script-commands/`:
the retained disassemblies, `source-contract.json`,
`static-source-verification.json`, `windows-model-fixed-result.json` and
`windows-model-fixed-europe/result.json`. Game data is not included in tracked docs.

## Managed publication and direct command fields

EUR e50-d49 adds `script-command-directory.json` / `engine.script-command-directory`
as file45/resource44, retaining all43previous decoded files and the e49generation.
The directory has no asset dependency; the resource set also requires the
preceding effect/event owner before activation. One authenticated690-byte source
binding serves the original advance and callback loads from typed commands,
without a ROM-byte shadow. Pages3ac/3ad merge the preceding event bytes, following
selector bytes and raw lookup gap at mixed access boundaries.

Original stream selection, signed cursor, unsigned opcode arithmetic, relative
targets, u16 advance wrap, callback BL/BX and return-controlled loops remain guest
instructions. Save and native ABIs do not change. Aggregate advance/callback
bytes and command-use counters remain outside machine serialization. The ordinary
scene assertion requires actual reads of both advances and callbacks.

A warning-free Windows build and two reduced Wine filters pass on the first run.
Managed validation covers13coherent/source-set guards, failure-atomic publication,
create/reuse/repair/move, all43prior decoded files/e49 retention, transactional
missing-file activation and resident ownership through unavailable files, Load,
Undo, backups and Restart. All690typed bytes and115commands agree with source
while the isolated machine's raw source is altered.

Native validation passes26original/interpreter/native complete-state/clock
prefixes, including raw115/255, signed cursors, u16 advance wrap, relative targets,
BL/BX call prefixes and callback-return loops. Compiled API reads account for
14advance/28callback aggregate bytes. Adjacent old event4B/selector4B remain typed
with their raw sources altered; previous2+new2/new2+raw2 accesses, all widths,
ROM mirrors, real BIOS policy, CpuSet/FastSet and DMA pass.314instructions/70PCs/
32native dispatches count coverage only. The prefix endpoint guard is reused;
production emitter and source model remain unchanged. Ordinary scene acceptance is recorded below, independently of the source/model
and targeted checks above. No closed source or older family proof is
repeated for context; Linux/USA/native Windows stay pending.

## Production CLI and positive ordinary scene

A warning-free Windows production build and reduced Wine actual CLI checks pass:
create/reuse without writes of47managed files, an unrelated working directory,
and loading the existing native cache. All45generation files match the accepted
e50dataset. The canonical executable is restored byte-for-byte.

One clean boot frame first establishes activation/cache compatibility with zero
command reads; it is not ordinary script acceptance. A later locator reuses the
genuine retained EUR suspend boot-to-battle input and save through8476frames at
classic240x160. It positively reads42command bytes, comprising14advance and
28callback bytes from7commands. Visual dialogue or an idle snapshot alone did
not establish this consumer; the source-use counters do.

One Wine fullpair from that actual CLI-published e50generation then passes all
8476complete states/clocks/central-wide image buffers/OAM/dispatch and every
mixed/PSG/FIFO/bitwise HLE audio sample, with the same42bytes/7commands. Audio
sample counts are18599928/18599928/2236876/2236876/18578590. The already built test
executable, native cache, input and suspend save are reused. No source/native/
lifecycle/CLI acceptance is repeated and no new fixture/navigation/payload edit
is needed. The other108commands remain targeted-only; this does not establish
a whole callback/interpreter ABI or new32:9scene acceptance. The preceding event/
selector owner reads zero bytes, so its ordinary criterion remains open.

Private results include `windows-build.json`, `windows-cli-summary.json`,
`windows-cli-no-write.json`, `windows-cli-battle-entry8476.json`,
`windows-scene-result.json` and `windows-battle-entry-script-full-state-audio.log`.
Original data, prior generations and316hash-mtime guards remain unchanged.

The producer audit confirms1265cc←125b6a in manager125ac8, dynamically assigned
through literals125ed4/125f10. Producer125e8c resolves the relative script archive
atB946EC using an event key returned by08009aa0 (sourceD68C2C). The complete archive
and event-key projections were still pending at that audit; the following
contract, model and runtime sections now cover them. A read-only audit of63distinct retained EUR transitional boot
snapshots had queued root0 and did not establish an active context by itself.

## Relative script archive and event-key source contract

The next bounded EUR source contains the relative archive at`B946EC..BB064C`
(114528bytes) and234four-byte event keys at`D68C2C..D68FD4` (936bytes), totaling
115464bytes in two disjoint ranges. Both ends come from independently rooted
readers: the localized-text fallback at`08009b04` names`BB064C`, and the coordinate
index reader at`08128922` names`D68FD4`. Neither extent is guessed from byte values.
The ranges are disjoint from all44published e50resource owners.

`08009aa0` returns`D68C2C + (u16(id)*4)`. Each key contains a script bank byte,
script index byte, text-bank selector byte and one preserved raw byte. The script
producer at`08125e98..08125eae` selects top-level bank`(bank-1)*4`, adds its u32
relative offset to`B946EC`, then adds a second u32 relative offset to that bank's
base. Key0is a null source record; this does not create a guest ID0guard. Raw IDs
234/239/65535 and all original truncation/address behavior remain outside the
projection contract.

All234key domains independently agree with62top-level banks and all233nested
slot domains. The self-delimiting first relative offsets agree with the same
index lengths:248top-level bytes and932nested-index bytes. There are213distinct
target addresses and20repeated links. Five links name four other bank headers;
two additional links (bank46:slot1 and bank49:slot1) name only FFalignment fill.
Those target views remain explicit. They are not rejected, flattened into copied
programs or declared unreachable.

The remaining physical storage partitions into207nonempty command segments,
35048stored commands using110opcodes,78212raw parameter bytes and45alignment-fill
spans totaling88bytes. Instruction lengths come from the authenticated
[command directory](#boundaries-and-original-consumers). The one-to-three-byte
FFfills end at independently established bank/end alignment boundaries. This is
a storage partition, not a linear execution graph, complete argument semantics
or proof that a guest can never read alignment bytes.

There are493canonical command5a references to49of the existing51effect selectors:
its u8 parameter q references selector`q-1`. These links and every command's
opcode/length establish actual dependencies on`engine.effect-event-directories`
and`engine.script-command-directory`. Other script arguments remain uninterpreted
byte parameters; relative branches and live cursors keep their original semantics.
No new guest q/opcode/ID cap is inferred from this decoded source projection.

The original key byte2consumer at`08009b28` selects a bank in one of five localized
text archives (`BB064C`, `C076F0`, `C5FBF0`, `CB7C48`, `D10644`). These archives are
not yet migrated. Text bank61also has a live variant path at`08009b3a..08009b64`;
it is not flattened into a fixed text choice. Byte3is preserved without assigning
unproved semantics.

Independent static verification reconstructs and hashes every115464source byte
exactly once, verifies234key records and233non-null two-level address formulas, all493selector
links,128original-instruction halfwords and five root words. The first private
builder incorrectly required every nonheader target to contain commands; its
assertion exposed the two FFfill targets and was corrected before writing the
source plan. No model/code/build/guest test was produced from that assumption.
The earlier candidate end`BB0000` was also excluded: the last script head is at
`BB048B`, before the independently rooted end`BB064C`.

This closes the bounded static inventory. The model and runtime results below
complete later criteria; this static check itself left EUR e50-d49/USAe8-d7 unchanged. No new Linux/USA test
or performance/space measurement is added. Private evidence is in
`target/rom-data-script-archive/source-contract.json`,
`static-source-verification.json` and the retained reader disassemblies.

## Typed relative archive model and reduced Wine source oracle

`rom_data::script_archive` now models the entire115464-byte source contract:
234event keys,62banks/233slots,207physical programs/35048stored instructions and
45alignment records. Slots refer to typed Program, Header or Fill targets;
relative source words are reconstructed from those references. Shared payloads
are stored once, and the model has no ROM-byte shadow. Source ownership remains
a physical partition, while header/fill target views remain explicit aliases.

Structural validation checks canonical indices, reference targets, instruction
continuity, nonoverlapping complete physical coverage and strict JSON. Dependency
validation reads actual command lengths and selector domains from the existing
command/effect assets. Raw arguments, the text-bank selector and keybyte3remain
preserved; q0/255and raw key IDs do not gain new guest bounds. Base authentication
rejects edited source data independently of those structural checks.

A warning-free Windows build and one combined Wine oracle pass on the first run:
238key-address prefixes,234two-level targets (including the raw0unaligned case),
234text-bank byte loads,35054opcode prefixes (all35048stored instructions plus
four header and two fill heads),2397relative-target prefixes and493selector
parameter prefixes. All115464bytes reconstruct exactly. Original load sites
observe42333distinct owned bytes;72811raw parameter bytes,234keybyte3bytes and
86fill bytes remain preserved/hashchecked but unread in these prefixes. That is
not a claim that the guest can never read them.

Sixteen shape, six source-edit, three dependency,13JSON and two identity guards
pass. The224382instructions/62PCs count coverage only. This does not establish a
replacement interpreter/callback ABI, native full-state/clock integration or
ordinary reachability. Source-only model work left e50-d49/USAe8-d7 unchanged;
publication and runtime integration are covered separately below.
Five linked localized text archives remain unmigrated. New Linux/USA/native
Windows checks and performance/space measurements are not added.

Private model evidence under`target/rom-data-script-archive/` includes
`windows-model-tests.json`, `windows-model-result.json`, `windows-model.log`,
`windows-model-europe/script-archive.json` and `windows-model-europe/result.json`.
The earlier global memory failure from concurrent Docker coverage work is
resolved by that work's own record; those files and external commits are preserved.


## Managed relative archive and typed guest reads

EUR `v1/e51-d50` adds `script-archive.json` / `engine.script-archive` as file46 /
resource45. All44 earlier decoded files and the e50 generation remain; USA stays
e8-d7. The manifest declares actual command-directory and effect-directory
dependencies. Activation validates command lengths and selector links against
those loaded assets, as well as source authentication and source-set coherence.

Two source bindings cover115464bytes. Guest reads reconstruct top and nested
index words, opcodes, raw parameters, four-byte keys and explicit fill values
from typed records. Mixed reads merge previous owners and raw ROM per byte,
including the earlier palette-effect owner on pageB94. There is no ROM shadow
or host script sequencer. Original IDs, key0 unaligned rotation, header/fill
views, live RAM, branches and callbacks retain their behavior. Counters and
bank/program/key/fill coverage masks remain outside save states.

Two reduced Wine filters pass on their first run after a warning-free Windows
build. The managed check passes13guards, atomic create/reuse/repair/move,
retention of all44 earlier decoded files/e50, failed activation isolation and
resident reads through unavailable files/Load/Undo/backups/Restart. With raw
source bytes altered, all115464typed bytes agree:1180index,35048opcode,
78212parameter,936key and88fill, covering62banks/207programs/234keys/45fills.

The native check passes28original/interpreter/native complete-state/clock
prefixes. Compiled APIs read37index/4opcode/7parameter/12key/1fill aggregate
bytes. Cases include raw IDs0/234/239/65535, low16 truncation, raw0 unaligned
rotation, header/fill targets, q0/255, relative targets and the return loop.
Previous palette-effect callback reads4bytes correctly on the same page; mixed
raw2+new2 and new2+raw2 reads, widths, mirrors, real BIOS, CpuSet/FastSet and DMA
agree. The169instructions/66PCs/28native dispatches count coverage only. These
prefixes do not prove the entire interpreter or callback ABI. Production emitter,
source model and save/native ABIs are unchanged.

## Archive production CLI and ordinary acceptance

The warning-free Windows production CLI creates and reuses48managed files from
an unrelated working directory, without rewriting them, and loads the retained
native cache. All46generation files match the accepted e51dataset. The canonical
production executable is restored byte-for-byte. Candidate SHA-256 is
`2a2ea73929aa9e8c097a2379ce1b025fa05396f44dd5fe9e49a425815b6b4d80`;
dataset SHA-256 is
`2a1f9aa937b20aadc24801adf4326dffc71bb7cba8329865c15440be6b4994dd`.

The genuine EUR suspend-boot-to-battle locator uses retained save/input files
for8476frames in classic240x160 mode. It reads27aggregate archive bytes:
8index/7opcode/10parameter/2key, one bank/program/key record, no fill. These are
new archive counters, independent of the preceding command-directory proof.
One Wine fullpair from the actual CLI-published e51generation passes every
one of8476full states/clocks, central and wide images, OAM, dispatch and mixed/
PSG/FIFO/bitwise-HLE samples. Observed sample counts are
[18599928,18599928,2236876,2236876,18578590]. Archive usage matches the locator.
All366hash-mtime guards pass; source-model, targeted and preceding e50proofs
were not repeated. `windows-scene-result.json` and
`windows-battle-entry-archive-full-state-audio.log` retain terminal evidence.

Other61banks/206programs/233keys/all45fills currently retain targeted coverage;
five localized text archives are subsequently migrated in
[e52](rom_data_text.md#managed-script-text-and-direct-typed-reads), and the
[ordinary event-or-selector criterion](rom_data_events.md#positive-ordinary-book-transition)
is subsequently closed by a new EUR book scene. No whole ABI or new32:9 scene claim. New Linux/USA/native
Windows checks remain pending; no performance or space measurements, new
fixtures, navigation or save/RAM/payload edits. Evidence is under
`target/rom-data-script-archive/`: `windows-managed-results.json`,
`windows-managed-europe/{managed-result,native-result}.json`, `windows-build.json`,
`windows-cli-{processes,no-write,summary}.json` and the corresponding logs.

## European script coordinate paths source contract

The EUR coordinate source at`D68FD4..D69BC4` contains3056bytes. Original reader
`08128914` names the first root at literal`128994`; the independent two-level
archive reader`0800a1f8` names the successor at literal`a234`. The first payload
offset`348` fences an840-byte/210-slot relative directory. All210strictly ordered
targets, u16count prefixes and four-byte record alignment partition the physical
storage through that independent successor. This is a bounded source projection,
not a global live index maximum inferred from a shape or caller subset.

The physical lists contain856unsigned XY pairs:420count bytes and1712coordinate
bytes, followed by42explicit two-byte FFalignment spans/84bytes. Lists have3–18
stored points and coordinates0–13 in this source; these observations do not add
guest count/coordinate caps. The original reader truncates its index to16bits,
loads a u32relative target and copies each pair into a four-byte live record.
The third byte comes from the original map-height reader`0801dd88`, using the
live EWRAM map pointer/width at`02007f10`; the fourth is zero. Height is not ROM
coordinate data and must not be precomputed into this asset.

Destination selection uses original`08125768`: u8destination maps to queued
root +`58 +132*destination`. The count is stored as u32at buffer+`80`; the
public return is its low byte. Original raw indices/destinations/counts remain
guest-controlled; the32-record buffer layout is not a newly proved count guard.

There are68canonical producer links to56paths in the existing primary script
archive. Command`44` has66links: it reads only byte2as the path ID, leaves byte3
raw/ignored and routes through`0812b6d0`→callback`0812acb4`→the coordinate reader
with destination15. Command`45` has2links, both ID1: it reads a full u16from
bytes1–2and a destination byte3. Inferring a u16index for command44from its
five-byte storage length was rejected before the contract; all66ignored bytes
happen to be zero, which does not establish a wider argument. Actual dependency
validation uses`engine.script-archive` and`engine.script-command-directory`.

Independent static verification reconstructs/hashes every3056byte exactly once,
checks68references from original bytes,194halfword/five-word reader/producer/
successor fingerprints and disjointness from all46current e52resource owners.
The previous script-key source ends exactly at this root; later runtime merging
must preserve its owner on the sharedD68page. No guest execution, model/runtime
acceptance, ordinary reachability or global210-path consumer coverage is inferred
from this source inventory. Linux/USA/native Windows checks remain pending;
no performance/space measurements. Private source evidence is under
`target/rom-data-script-coordinates/`.

## Typed coordinate path source model

`rom_data::script_coordinates` now represents the relative directory, count-derived
XYlists, exact alignment and68typed script links. Strict JSON objects reject
unknown fields. Structural validation preserves the complete u8coordinate range;
base-source authentication rejects coherent edits. Dependency validation checks
the actual archive links and command44/45lengths/callbacks, retaining the ignored
command44byte3semantics. All3056source bytes reconstruct and hash exactly.

A warning-free Windows test build and one new Wine source filter pass on their
first execution:24shape/JSON/provenance/dependency guards, every independently
recorded source field and194halfword/five-word fingerprints. Original execution
copies all210paths, plus two calls after changing live map-height cells:212calls
match the entire EWRAM output, count, return and callee-saved registers. The2972
index/count/XYsource bytes are observed at original loadsites;84alignment bytes
are preserved and hash-checked but unread by these cases.

Thirteen additional original index/destination prefixes cover low16/low8wrap and
raw210/255/65535, and13producer prefixes cover byte44versus u16byte45, ignored
arguments, nibbles and the secondary task's fixed destination15. The40116
instructions/134PCs measure coverage only. The controlled scratch map/arguments
are source-oracle setup, without gameplay snapshots or navigation. This does not
establish native/full-state-clock/whole movement-task/allocator or ordinary ABI.

The model-only stage left e52-d51publication/runtime unchanged. The managed,
direct-consumption and ordinary results follow below. New
Linux/USA/native Windows checks remain pending, with no performance/space
measurements. Evidence includes `windows-model-build.json`,
`windows-model-result.json`, `windows-model-europe/{result,script-coordinates}.json`
and `windows-model.log` under the source evidence directory above.


The five linked localized text archives now have a [bounded source contract](rom_data_text.md#script-selected-localized-text-source-contract):
1803744bytes/five ranges,63top entries and26live variants per language,
385physical banks/41010programs,147glyphs already owned by the complete font
asset. The source preserves division/remainder selection, aliases, all program
headers/commands/encoded tails and exact alignment bytes. Static roundtrip,
dependency and earlier-owner checks pass without guest tests. Their [strict
model and new Wine source oracle](rom_data_text.md#typed-script-selected-text-model-and-reduced-wine-source-oracle)
now pass whole-source reconstruction, guards and new selector/index/header/
expansion cases. Their [managed/runtime and CLI integration](rom_data_text.md#managed-script-text-and-direct-typed-reads)
now passes e52-d51publication, typed source reads and targeted compiled checks.
One positive school3000Wine pair now passes every full state/clock/image/audio
with new text source use; other records retain targeted coverage. e51archive
acceptance above stays closed.

## Managed coordinate path publication and typed reads

EUR e53-d52 adds `script-coordinate-paths.json` /
`engine.script-coordinate-paths` as file48/resource47, retaining every46prior
decoded file and e52generation. USA stays e8-d7. Actual archive/command
dependencies validate producer links before activation. One3056-byte binding on
D68/D69serves typed relative indices, counts, XYpairs and explicit alignment.
The original guest reader still selects its raw index/destination, copies the
path and obtains height from live RAM. Guest instructions, clocks, save/native
ABI and movement/allocator control remain unchanged.

One immutable authenticated owner serves interpreter/compiled reads. The D68page
merges earlier text, script keys and the new coordinate directory per byte;
D69BC4and later bytes remain raw. Counters outside guest state distinguish
index/count/coordinate/alignment bytes and used path/alignment records. Data is
owned as typed fields, without a ROM byte shadow or host movement routine.

The affected Windows core test build passes. An initial frontend compilation
finds a new test counter array5versus4; only that test is corrected and the
successful core build reused. The corrected frontend is warning-free. One Wine
managed filter passes first execution:48files/47resources/all46previous decoded
files/e52retention, injected publication failure, create/reuse/repair/move,
14coherent/missing-file guards and both actual dependencies. Failed attachment
retains the previous owner/state; unavailable files, Load/Undo, cartridge backup
restore and restart retain it. All3056typed bytes/four classes/210paths/856points/
42alignment records pass with their raw source span altered.

The first native filter stops in its new observer's remaining hardcoded five-
element loop. It is corrected to zip the four typed counters; failure logs,
test source, executable and ELF are retained. Only the frontend observer rebuild
and native filter repeat; successful managed/core/source checks stay closed.
The corrected Wine native filter passes24original/interpreter/compiled complete-
state-clock cases, including four full coordinate copies and changed live map
heights. Compiled API reads[28index,85count,76XY,2alignment]aggregate bytes.
Raw indices/destinations, earlier text/key owners, upper raw boundary, widths,
mirrors, real-BIOS reads, CpuSet/FastSet and DMA pass.1443instructions/87PCs/
143dispatches count coverage only; whole allocator/movement-task ABI and ordinary
reachability are separate. Evidence is under `target/rom-data-script-coordinates/`.

## Coordinate path production CLI and ordinary scene

A warning-free Windows production build freezes a private e53CLI and restores the
canonical executable's exact hash/mtime. The established Cargo invalidation
touches only main.rsmtime, leaving its content unchanged. Actual Wine CLI
preparation from an unrelated working directory creates/reuses50managed files
without writes, loads the old native cache and matches all48generation files to
the accepted dataset
`0d3cf2cb3f8caa30f658ec2390f481da22f3c635830fc83aa5a77df8918a9841`.

The retained genuine EUR frame21420snowball snapshot and its normal6000-frame
input supply a positive new-family locator:25bus reads/28aggregate source bytes
=4index +14count +10XY, one path, zero alignment. The old e49negative counter
result is not reused as coordinate evidence; this check uses the actual newly
published coordinate owner. No new fixture or gameplay navigation is needed.

One matching Wine6000-frame pair passes every full state/clock, central/wide
image, OAM, dispatch and mixed/PSG/FIFO/bitwise-HLE sample. Audio counts are
[13167000,13167000,1584000,1584000,13166910]; coordinate use matches the locator.
All581hash-mtime guards pass. Other209paths/all42alignment spans retain targeted
coverage. No whole movement/allocator ABI or new32:9scene is claimed; new
Linux/USA/native Windows checks remain pending, without performance/space
measurements. Terminal evidence includes `windows-managed-europe/{managed-result,
native-result}.json`, `windows-native-retry-result.json`, `windows-cli-summary.json`,
`windows-scene-result.json` and `windows-snow-coordinate-full-state-audio.log`.


## European auxiliary script archive source contract

The independent EUR reader `0800a1f8` names root `D69BC4` at literal `a234`.
It loads an unsigned byte action and slot from its context, reads a relative
`u16` bank offset at `root + 2*action`, then a relative `u16` program offset at
`bank + 2*slot`. It stores the resulting program pointer at context+4, zeros
the cursor at context+2 and calls `0800a240`. This is a separate archive from
the primary `B946EC` archive; its index width and dispatch table differ.

The declared storage projection has eight banks and 442 slots:
63/63/1/63/63/63/63/63. The first top offset16 and all explicit bank targets,
inner minimum-offset fences and the original constant action7 corroborate the
layout. There are 126 distinct slot targets, including30 interior instruction
entries, in133 physical zero-terminated programs. Linear root walks alone omit
69 physical instructions; the complete physical parse retains them. The913
instructions comprise913 opcode bytes and2275 parameter bytes.900 index bytes
and five internal FF alignment spans/10bytes complete the4098-byte archive
projection `D69BC4..D6ABC6`. Stored sizes are no new guest domain guards.

`D6ABC6` is the end after the last linked zero terminator. The following two
FF bytes remain outside this projection. The independently consumed graphics
base `D6B4C8` (literal `12fad4`, load `12fa5a`) is only an upper ceiling:
including everything up to that base would incorrectly claim2306 further
bytes. No independently rooted successor at `D6ABC8` has been established.

Interpreter `0800a240` uses a separate directory at `370F94`, named by literal
`a288`. The canonical17 six-byte rows contain `u16` advance and a Thumb callback
pointer:102 bytes through `370FFA`. The next independently consumed table is
`370FFC` (literal `131b0`, load `13142`); the two intervening zero bytes remain
unowned. All17 callbacks are distinct original functions. Zero opcode returns
before dispatch/advance; its stored directory row is still preserved. Nonzero
opcodes reconstruct the pointer bytewise and use original BX dispatch, then
add the advance to the live `u16` cursor. Reads use a signed `i16` cursor.
The archive depends on this directory; no primary-script dependency is inferred.

The original relative branch callbacks add their operand to the cursor before
the directory advance. All489 canonical relative targets land on instruction
boundaries in the same bank; opcode13 delegates to opcode12's callback helper.
A structural graph with both predicate outcomes covers908 of913 instructions;
the other five remain preserved. This graph does not prove actual reachability
or predicate semantics. Parameters and all original RAM predicates remain guest
work; no host interpreter or branch replacement is introduced.

Wrapper `0800a1a4` truncates action and selection flag to their low bytes. A
nonzero flag selects the live byte at `02002191`; zero chooses slot0. The outer
`0800a158` compares the full action against2 before calling that wrapper and can
select action7; `0800a6d8` explicitly supplies action2/slot0. These producers do
not establish global live maxima action7, slot62 or opcode16. Raw aliases and
out-of-projection reads must retain original guest arithmetic and ROM fallback.

Static verification independently reassembles all4200 bytes exactly once from
typed indices, commands, callbacks, physical instructions and exact gaps. It
checks723 original halfwords/four literal words, independently recomputes all489
relative target formulas, and proves disjointness from all47 published e53
owners. The directory's two zero bytes and archive's final two FF bytes are
explicitly excluded. Evidence is in `target/rom-data-auxiliary-scripts/`:
`source-contract.json`, `static-source-verification.json`, and original consumer
windows. No build or guest execution was needed for this inventory. The source-only inventory did not include a model or guest execution. The
models and reduced Wine source oracle are now verified below; publication,
direct runtime, actual CLI and a positive ordinary scene remain pending. EUR stays e53-d52 and USA
stays e8-d7; new Linux/USA/native Windows checks remain pending. No new gameplay
fixture/navigation or performance/space measurement was performed.


## Typed auxiliary archive and reduced Wine source oracle

`gba-core::rom_data::auxiliary_scripts` now provides strict lossless models for
the separate17-row directory and4098-byte archive projection. Typed bank/slot
references identify a program and an instruction ordinal, preserving all30
interior entries without duplicate payloads. All133 physical programs/913
instructions, raw parameters and five internal alignment spans are retained.
Source views reconstruct both levels of `u16` relative indices, advance fields,
Thumb pointers, opcodes and parameters. No full ROM byte shadow is stored.

Shape checks validate ordering, exact physical coverage, same-bank instruction
entries, program terminators and gaps. Dependency checks require the actual
auxiliary callbacks/advances and instruction-boundary relative targets. All
nested JSON objects reject unknown fields. Coherent parameter/entry/callback
edits remain distinguishable from authenticated source; malformed relative
operands return an error. Public raw-address helpers retain low-byte arithmetic
without imposing stored action/slot/opcode counts on guest execution.

The first Windows build is warning-free. Its new Wine filter passes all source
and canonical assertions, then fails an expectation for raw action8/slot0: that
lookup creates an odd bank address. The original LDRH loads an aligned halfword
and rotates it; the initial test used an unaligned byte slice. Only the new test
observer/expectation is corrected, with all failure sources/executable/logs
retained. The frontend-only rebuild/filter retry passes. Reading the unchanged
CPU implementation then establishes that this rotation acts on the promoted
32-bit value. A further test-only correction and raw8/slot1case verify the high
bits: aligned word000C atD69E98 produces guest target14D69E97. The final frontend
build is warning-free and the same affected Wine filter passes. The model/core
are unchanged across these retries; no old source/runtime test is repeated.

The final single-filter scope passes40 shape/provenance/dependency/JSON/identity
guards and independent fields/hashes/723halfword/four-word fingerprints. Original
execution checks442 context lookups,913 opcode/zero-stop/dispatch prefixes, one
RAM opcode3prefix for the unused canonical command row,80 advance prefixes,
489 relative-update suffixes,10 wrap cases,15 raw-index/signed-cursor/opcode
prefixes and11 wrapper/full-action-before-low-byte probes. Complete EWRAM checks
cover the intended context writes; predicate helpers and whole interpreter/
callback/native/full-state-clock/ordinary ABI are outside this test's scope.

Exactly2887 distinct canonical source bytes are read at the original loadsites:
900indices +913opcodes +978relative operands +96directory bytes. The remaining
1313bytes are preserved/reconstructed/hash-checked:1297other parameter bytes,
10alignment bytes and the6-byte opcode0directory row (zero returns before its
lookup). The31839instructions/177PCs count coverage only. Controlled scratch
arguments are used without loading or editing a gameplay fixture/save/input.

Final evidence is `target/rom-data-auxiliary-scripts/windows-model-final-build.json`,
`windows-model-final-result.json` and `windows-model-final-europe/result.json`;
accepted typed JSON files are in the latter directory. Earlier failure and
narrower passed retry evidence is retained separately. Only two module
registrations changed in existing implementation files at this model stage;
its dataset stayed EUR e53-d52/USAe8-d7. Managed publication/direct consumption
are now verified below. Actual production CLI/positive ordinary scene and new
Linux/USA/native Windows tests remain pending, without performance/space
measurements or new gameplay navigation.


## Managed auxiliary script fields

EUR e54-d53 publishes `auxiliary-script-command-directory.json` /
`engine.auxiliary-script-command-directory` and `auxiliary-script-archive.json` /
`engine.auxiliary-script-archive`:50 generation files/49 resources, preserving all
47 prior decoded files and e53 generations. USA remains e8-d7. The directory has
no asset dependency; the archive requires its actual auxiliary directory. Both
file hashes, bindings, source identities, typed shapes and callback/advance
relationship authenticate before a single shared immutable owner activates.

The owner reconstructs34advance/68callback directory bytes and900index/913opcode/
2275parameter/10alignment archive bytes from typed fields. Source indices retain
interior instruction entries. There is no ROM shadow, host interpreter, new
guest action/slot/opcode guard, second activation or save/native ABI change.
Original low-byte arguments, signed cursor, promoted32-bit unaligned LDRH
rotation, callback predicates and control flow remain guest execution. Shared
page370 retains earlier audio owners; D69 now merges coordinate and auxiliary
fields. From e54 onward the auxiliary projection atD69BC4 is typed, while
D6ABC6and later bytes and directory370FFAand later bytes retain raw fallback.

One warning-free Windows build compiles the affected core/frontend tests; core
tests are not executed. The two new exact Wine filters both pass on their first
execution. Managed checks cover26 coherent/missing-file guards, actual directory
and complete-source-set checks, injected publication failure, create/reuse/
repair/folder move, e53retention and failed-attach isolation. Unavailable files,
Load/Undo, cartridge backup recovery and Restart retain the same owner. All4200
source bytes and all17commands/8banks/133physical programs/five alignment spans
are served correctly after raw source bytes are altered.

The native filter compares44 original/interpreter/compiled prefixes using
complete serialized state and guest clock. Compiled API source-byte counts are
[6advance,65callback,18index,19opcode,15parameter,2alignment]. Cases include
interior entries, signed cursor and advance wrap, raw opcodes/indices, action2
versus102 and the raw8:1high-bit LDRH target. Mixed prior/current/raw owners,
widths, ROM mirrors, real-BIOS mode, CpuSet/FastSet and DMA pass. The624original
instructions/92PCs/52native dispatches count coverage only. This does not prove
whole predicate/interpreter/callback ABI or ordinary reachability.

The accepted dataset ID is
`c3eb7042e1b5d98d2b553db5019eb3fd7d40e01ccde5362fdd7b263a6aa85f10`.
File hashes are `b8ba30cd6f52a97da7cde1b464b8184c2cb54157843c348cd09ceffab8080544`
for the directory and
`bd410f3fde23fb68bec936da0ac5bed89dde2897a640c2e008d6b920351907b3` for the archive.
All587hash-mtime guards pass. Evidence under
`target/rom-data-auxiliary-scripts/` includes `windows-managed-build.json`,
`windows-managed-results.json` and `windows-managed-europe/{managed,native}-result.json`.
The targeted checks do not themselves prove production CLI or ordinary behavior;
those separate criteria are now verified below. New Linux/USA/native Windows/
32:9 checks remain pending. No performance/space
measurement or new gameplay fixture/navigation was performed.


## Auxiliary script production CLI and positive ordinary scene

A single warning-free Windows production build freezes the private e54CLI
`c0cc57a578e015b50f5df0a075dd3f71ef81e8891c872c2cf5941d84ad47afdf` and restores
the canonical executable's exact hash/mtime. Established Cargo invalidation
touches only main.rsmtime; its content remains unchanged. Actual Wine CLI
prepare/reuse from an unrelated working directory creates/reuses52managed
files without rewriting them, loads the old EUR native cache, and matches all
50generation files to the accepted e54dataset
`c3eb7042e1b5d98d2b553db5019eb3fd7d40e01ccde5362fdd7b263a6aa85f10`.

New static caller cues locate the action2/slot0 wrapper at615B4/63914 and the
boolean wrapper atD52FA/D5558. The retained genuine frame21420snowball snapshot
and its normal6000-frame input supply one positive new-family locator. The
auxiliary directory has96bus reads/96aggregate source bytes =32advance +64callback across
six commands. The archive has74bus reads/82aggregate source bytes =16index +20opcode
+46parameter across two banks/two physical programs; alignment is0. Old
source-family scene results are not recounted as auxiliary evidence. No new
gameplay navigation, fixture, save override or checkpoint/input edit is needed.

One corresponding Wine6000-frame pair from that actual CLI generation passes
every complete state/guest clock, central/wide image, OAM, dispatch and mixed/
PSG/FIFO/bitwise-HLE sample, absolute frame21420..27420/classic widescreen0.
Audio counts are[13167000,13167000,1584000,1584000,13166910]; new resource use
matches the CLI locator exactly. All640hash-mtime guards pass. The same frozen
e54test executable/native cache/state/input are reused; no production build,
prepare, locator or old source/targeted/scene acceptance is repeated for context.

Other11directory commands, six banks,131physical programs and all five alignment
spans retain targeted coverage. Whole predicate/interpreter/callback ABI and
new32:9/native Windows/Linux/USA coverage are not inferred from this scene.
No performance/space measurement is made. Terminal evidence under
`target/rom-data-auxiliary-scripts/` includes `windows-build.json`,
`windows-cli-{processes,no-write,summary}.json`, `windows-scene-result.json` and
`windows-snow-auxiliary-full-state-audio.log`. CLI and positive ordinary children
are closed; remaining map/terrain, other assets/GUI/archive/mod work stays open.
