# Typed audio source contracts

This is an EUR source inventory for ROMDATA-01, authenticated against
`ffta_rom::EUROPE_SHA256`. The source model and its original-reader Wine oracle
are implemented. EUR e44-d43 publishes and consumes the typed directory,
produced tones/keymaps/PCM/CGB waves, sequence instructions and their proved driver
tables, including seven pitch/rate/noise/volume lookup tables. Production CLI
and one new 400-frame ordinary Wine differential pass for
the sequence family; earlier tone/sample and MP2K shadow acceptance stays closed.
New Linux, USA and native Windows validation are also pending. There are no
performance measurements or space targets in this work.

## Driver identity and live evidence

The strongest `mp2k::detect` signature occurs once, at file offset147f40:
the literal pool contains SoundInfo pointer03007ff0, identifier68736d53,
Thumb mixer entry030034dd, VCOUNT04000006 and PCM buffer displacement0350.
The original Thumb SoundMain entry is **08147ed4**, distinct from that pool
offset. Its code checks the identifier, locks it and branches through the RAM
entry. Do not infer a function entry from `Mp2kSig.sound_main_off`: the primary
detector currently stores a pool offset in that field.

Static parsing of the retained ordinary EUR Goblin/Play checkpoint confirms
SoundInfo020084e0, idle identifier68736d53, eight channels and PCM frequency
15768. Its active channel headers point into ROM, and its three live player
headers match directory IDs20,114,101. This reuses an existing snapshot; it is
neither a new guest run nor proof that all audio assets have been cataloged.
Driver selection is M4A/MP2K, so generic GAX/RDRV structures cannot establish
this image's asset contracts.

## Player configuration and song directory

Original initialization08148b00 loads literal count3 at148b6c and source
0814f51c at148b70. It advances by12bytes per player. This proves the36-byte
source14f51c..14f540 and the following fields:

- Word00: mutable player destination address, in EWRAM for this image.
- Word04: mutable track-array destination address, in IWRAM.
- Byte08: track capacity, passed to081491f8 and clamped there to16.
- Byte09: retained padding with no observed reader.
- Halfword0a: read by08148b3e and truncated to player byte0b, which gates
  priority replacement in08149270. Preserve the full halfword.

Capacities are16,6,3. Destination addresses describe guest state; they are
immutable configuration values, never host pointers or decoded mutable state.

The adjacent directory occupies14f540..150940:640eight-byte entries/5120B.
Seven checked literals in08144f04/08144f34 and08148b84/08148bb0/08148bfc/
08148c50/08148c84 reference its root. The original08148b84 path computes
`8 * (argument & 0xffff)`, reads header word00 and player-index halfword04,
then indexes the player configuration with stride12. Halfword06 is preserved
without a demonstrated reader; although all its values equal halfword04 in
this image, that equality does not establish the same meaning.

There is **no local640-entry bounds check**. The declared physical catalog
ends at the shared empty header150940; an independently followed tone record
at14da14 points to WaveData150944, four bytes after that header. The directory
has231,327,82entries selecting players0,1,2, respectively. Catalog membership
must never be used to replace the original behavior of other u16 indices.

## Header aliases and outgoing roots

The640entries reference455distinct headers:454nonempty and one zero-track
header,150940, aliased by186entries. Nonempty headers have1..16tracks, each
within its selected player's capacity. Original08149270 reads priority byte02,
bank pointer word04, track count byte00 and track pointers at08+4*i; it starts
only `min(header count, player capacity)` tracks and clears remaining player
tracks. Byte03 can request the original reverb update via bit80. Byte01 is
zero throughout this catalog but has no proved reader here.

These headers occupy10080source bytes and expose1610distinct track roots.
Each header has an8-byte prefix followed by exactly4bytes per declared track.
The zero-track header still has its word04 loaded by the original routine:
that word is40000000, overlapping the beginning of WaveData150944. This alias
must remain explicit; do not reject it as an invalid bank pointer or assign
the overlapping bytes two independently editable owners.

Nonempty headers reference four bank roots:14d9fc,14df78,14e578,14eb6c.
Original voice command081483e4 reads a raw byte selector and copies three
words from `bank + 12 * selector` into the track. There is no local127mask;
128tone entries cannot be assumed from a generic SDK layout. Note setup
081487cc also follows type40/80 split/keymap links and copies a tone's wave
pointer and envelope word. Bank extents, reachable selectors, nested tones,
keymaps and sample payload extents still require their own producer proof.

Original sequencing081485ac retains running status, waits80..b0, commands
b1..ce and notescf..ff. Its command dispatch is mutable RAM configured at
initialization, not a license to classify the entire neighboring ROM as data.
Track bytecode extents, branches/pattern returns and extended commands remain
unmodeled. In particular, generic shadow-HLE DPCM support does not prove this
image uses compressed waves; the inspected original mixer reads signed bytes.

## Retained evidence and next criteria

Private artifacts under `target/rom-data-audio-m4a/` include exact engine/live
identification, bounded original disassemblies, checked pointer/call candidates
and `directory-independent.json`/`inventory-summary.json`. Independent field
decoding reproduces all36player bytes,5120directory bytes and10080header bytes,
including padding, unused halfwords, dummy pointers and aliases. No ROM, save,
sample or sequence bytes belong in Git or packages.

## Source model and original-reader Wine oracle

`gba_core::rom_data::audio_directory::Catalog::from_rom` authenticates the exact
EUR ROM,14checked root literals, original count/stride/index/header operations
and the empty-header/first-wave boundary. `AudioDirectory` contains three player
records,640song records and455headers ordered by source offset. Header IDs are
the lowest song ID referring to each header; repeated directory entries share
one header owner. JSON rejects unknown fields and validates counts, IDs, order,
alignment, guest destination extents, track lengths, links and player capacity.
`verify_source` rejects coherent semantic edits as well as broken structure.

The457source ranges total15236bytes. Every source field has exact little-endian
reconstruction, including all unused bytes and the overlapping empty header's
40000000bank word. `selected(argument)` uses low16; `None` denotes an unowned
lookup and never substitutes a guest validity check. No sample or sequence bytes
are decoded, and no source model operation changes the original player state.

The focused private test in `crates/recomp/src/rom_data/audio_directory_tests.rs`
runs the original08148b84→08149270→return path under Wine for all640entries.
Another nine targeted cases exercise high argument bits, live track capacities
0/1/6/16, priority rejection and the player lock. One explicit raw index640probe
stops at the actual start entry, before its unowned null header is dereferenced;
its original directory loads at08150940/08150944 are recorded, not modeled as
another song. Six unauthenticated/unpublished source variants exercise priority,
reverb, unused header/directory/player fields and high priority-gate bits.

All649real-source/targeted-input cases and6synthetic cases pass independently
predicted complete EWRAM and IWRAM outside call-stack scratch, track destinations,
SP/callee registers and stack sentinels. Zero-track aliases, low16index behavior,
live `min(count, capacity)`, locking, priority and reverb agree with original code.
Original distinct reads cover13477bytes:3840directory bytes,12player-address
bytes and9625header/track-pointer bytes. The other1759bytes are preserved but
not falsely claimed read. All1610track pointers are read by the original start
path. These are targeted source checks, not ordinary-gameplay or native/full-
state/clock differential acceptance.

The final Windows test compilation is warning-free; the single focused Wine
oracle passes17JSON/shape/link/source guards too. Initial compile E0283 in a new
test assertion and an initial oracle expectation failure are retained privately.
The latter expected a per-track index at track word20; original1492e4 captures
zero once and the1492e6backedge skips recapture, so the corrected oracle expects
zero. The asset model was unchanged by that correction. Final evidence is
`windows-retry2-tests.*`, `windows-original-retry1-{result.json,log}` and
`windows-original-retry1-europe/{audio-directory,result}.json` under the same
private directory. All364protected inputs/evidence/canonical executables remain
unchanged. Instruction/PC counts are coverage only; no performance measurement.

## Managed publication and typed source consumption

EUR `v1/e41-d40` adds `audio-directory.json`, resource `audio.m4a-directory`,
format `audio-directory-v1-json`. All36files/35resources authenticate together;
the34prior decoded files and retained e40generation stay unchanged. USA remains
e8-d7. The457audio source spans do not overlap any prior resource. Song/header
links are internal to this resource; it has no external decoded dependency.

The runtime retains the typed model and an ordered source index. Player/song/
header fields reconstruct each byte directly; there is no second raw overlay.
Original CPU, DMA, BIOS copies and native bus calls receive typed bytes only
within owned spans. Fragmented header gaps and bytes after the dummy header
remain raw. The first four WaveData bytes at150944are also the empty header's
bank word, so this existing alias retains one owner. The mixer, sequencer,
mutable RAM, guest clock and save/native ABI are unchanged. Current shadow
audio readers consume live voices/waves, not this directory; full tone/sample/
sequence migration is separate unfinished work.

Nonserialized usage counters distinguish player, directory, header-prefix and
track-pointer bytes. Bitsets separately observe3players,640songs and455headers.
They survive Load/Undo with the immutable owner and are absent from guest saves.
CLI logs expose family totals and observed record counts; these are aggregate
usage/coverage evidence, never a performance or distinct-byte measurement.

Warning-free Windows build5021and two reduced Wine checks51686pass publication
failure/create/reuse/repair/move/e40retention,25coherent decoded/link/source/
dependency/path/role/hash/coverage guards and transactional missing-file failure.
Load/Undo/backup/Restart retain the owner after its copied dataset disappears;
all15236bytes and all3/640/455record masks read correctly with raw spans altered.

The new native check covers44original/interpreter/native start paths, using
15236altered raw ROM bytes and comparing complete serialized states/clocks.
Compiled reads are positive in all four categories:176player/264directory/
539header/884track-pointer bytes, including rereads. All457spans, every byte,
boundary widths/mirrors/real-BIOS views, SWI copies and DMA pass. Explicit checks
prove two typed/two raw bytes after the dummy header and a CPUSet crossing of
2directory+8dummy-header+2raw bytes. Source oracle649+1+6and older family checks
were not rerun. There were no build or Wine failures in this integration unit.
Private evidence: `windows-managed-tests.*`, `windows-managed-results.json`,
`windows-managed-europe/{managed-result,native-result}.json`, and the frozen
`recomp-tests-managed.exe`.14926instructions/181PCs/608native dispatches are
coverage only. Full bank/sample/sequence extents and new Linux/USA/native Windows
checks remain open.

## Ordinary European directory use and production CLI

Production Windows build78108is warning-free; its private candidate is
`target/rom-data-audio-m4a/recomp-rom-data.exe`, SHA-256
`700009cc615f516bd31d98fdf982e4970a3dc64b6f470db0ee0045c7ddb060ec`.
The prior canonical executable is restored exactly. Actual Wine CLI14580creates
e41-d40and reuses it without changing hashes or modification times of its38
managed files. CLI27876runs from an unrelated working directory with the old
native cache and the retained ordinary EUR IN13-derived Goblin/Play checkpoint
and bank-play input. No new GUI navigation, snapshot construction, RAM/state
payload edits, personal save writes or USA/Linux execution are involved.

The400-frame scene reads179new-family source bytes in77bus calls:36directory,
79header-prefix and64track-pointer bytes, including rereads. Three song records
and three headers are observed. Player configuration reads are zero in this
replay; their ordinary initialization is not inferred from targeted coverage.
Title wrappers can select an existing RAM player directly, so positive audio
directory use does not require a configuration-table read.

One new Wine91327pair passes every complete serialized state/clock, central/
wide image, OAM, dispatch count and mixed/PSG/FIFO/HLE audio sample, with the same
179byte category totals. The replay uses mixed native/interpreted execution
(1379881native/2741618fallback dispatches, coverage context only); it is not an
all-native or performance claim. Frozen managed tests and validation helper are
identified in `scene-executable.json`. The source649+1+6, native44and earlier
family scene proofs remain closed; this pair validates the new e41directory.

The final568×160CLIimage shows the bank background. The retained snapshot's
serialized separate-UI setting omits the panels in this capture, as previously
observed; it does not establish a new food-count or confirmation result.
Private `windows-{build,cli-processes,cli-no-write,cli-summary,scene-result}.json`,
`windows-bank-play-audio-directory-full-state-audio.log` and `bank-play.png`
retain the evidence. Full tone-bank selectors/extents, nested tones/keymaps,
sample payloads and complete sequence bytecode remain the next audio work.

## Produced tone, keymap and PCM source contracts

A bounded source traversal now follows all 1,610 track roots, preserving
running status, pattern returns, repeat state, selected program and last note
key. Its 177,047 consumed instruction/operand bytes in 805 ranges do not overlap
the 455 headers, tone records, PCM extents or CGB wave sources. All graphs reach
a stop or repeated control state without a discovery failure; observed pattern
depth is at most one. This is source discovery, with no host scheduling or new
managed resource. EUR remains e41-d40.

Original 081482f0 copies 36 command words from 370a90 to 02008e60; 08148dbc
patches several slots. Seven checked literal readers and the reconstructed
patches reproduce the retained ordinary state's entire table. B9 and CD differ
from the ROM defaults. Main 081485ac dispatches waits 80..b0, commands b1..ce
and notes cf..ff, remembering running status only for commands >=bd. No B5 or
B9 command is reached in these tracks. CD selects only extended commands 8/9,
at four occurrences, with no extended wave-pointer change. Eighteen note events
use the initialized RAM tone before a program selection; they have no ROM owner.

There are 390 produced bank/program pairs. Bank roots 14d9fc/14df78/14e578/
14eb6c have 35/121/121/113 produced selectors, with maxima 115/127/126/126.
These are discrete domains, not complete arrays or guest bounds checks.
Original BD reader 081483e4 takes the raw operand byte and copies three words
from `bank + 12 * selector` into track 24/28/2c.

Tracking note keys yields 2,065 bank/program/key pairs. Seven type40 map tones
have 348 reached key links backed by 347 distinct bytes: two maps share byte
14f3b8. They select 40 nested records, 39 additional to the main 390, yielding
429 canonical 12-byte tones / 5,148 bytes. A generic 128-key sweep reaches
adjacent tables, including selector 255; it is rejected as physical ownership
proof. Other keys or changed RAM bank pointers remain raw outside this plan.

The tones include 415 type00 PCM records, one type08 fixed-frequency record,
seven type40 maps and two each of CGB types 1/2/3. The 416 PCM references share
277 unique 16-byte WaveData headers: all type0, with 129 flags4000 and 148
flags0. Frequency word, loop start and declared length remain exact fields.
Nominal signed 8-bit payloads total 1,931,528 bytes. No reached type has bit20;
generic shadow-HLE DPCM support does not prove compressed assets here.

For 276 interpolated waves, ARM 081481e0/081481e4 reads the final nominal
sample and the following signed endpoint while one interval remains. Preserve
that endpoint's actual value; it can be nonzero. The exclusively type08 wave's
direct reader 08148140 stays within nominal length, leaving its following byte
unowned. The final interpolated extent ends exactly at first track byte 3371dc.
Two CGB type3 sources at 14f4dc/14f4ec have 16 bytes each; static original
081497b4..081497c8 copies four words to wave RAM. Their execution remains pending.

The resulting plan has 736 nonoverlapping new ranges / 1,941,759 bytes: 429
tones, 28 merged keymap spans, 277 wave spans and two CGB spans. It excludes
150944..150948, already owned by the e41 empty header; the new family must
depend on the real audio-directory owner for that alias. Unselected records
and gaps remain unclassified; the whole audio inventory stays open.

Warning-free Windows build 32264 and one focused Wine check 8457 pass 390 BD
copies with complete EWRAM/SP/callee checks, 348 mapped note selections, 276
interpolated endpoint probes and one fixed-frequency endpoint probe. Note/kernel
probes stop at explicit internal checkpoints, so they are not full mixer,
state/clock/native or ordinary-scene acceptance. The check reads 4,680 program
bytes and 347 map bytes, not every PCM byte. All 480 protected inputs/evidence
stay unchanged; 38,926 instructions / 109 PCs describe coverage only. An initial
static assertion confused the handler at 148e2a; the accepted CD replacement
root is 148e10/148ea8. No ROM or runtime code changed for that correction.

Private evidence is under `target/rom-data-audio-tones/`: command-table and
sequence producer JSON/scripts, `nested-tone-producer-domains.json`,
`tone-wave-independent.json`, `producer-independent-checks.json`,
`producer-source-plan.json`, `original-source-cases.json`, Windows build logs
and `windows-original-europe/result.json`. Next: shared bounded ROM producer
parser and lossless tone/keymap/PCM/CGB model, followed by managed/runtime and
ordinary Wine acceptance. The closed 390/348/277 reader proof and e41 acceptance
are retained; new Linux/USA/native Windows checks remain pending.

## Lossless tone and sample model

`audio_tones::Catalog::from_rom` now authenticates the exact EUR directory and
uses the shared bounded `audio_producers` traversal to derive the declared
domains. It reproduces all 1,610 graphs / 147,424 states, 390 program domains,
2,065 program/key pairs and the independent 805 consumed bytecode ranges.
That traversal analyzes source only; guest sequencing and timing remain live.
Extended producer commands outside the proved CD8/9 domain fail explicitly.

`AudioTones` contains four banks, 429 typed tone records, 347 canonical map-byte
records with 348 links, 277 PCM waves, two CGB waves and the directory alias.
Bank IDs use the first referring song ID. Tone IDs follow first bank/program
references, then parent/key references; wave order follows first referring tone.
Source offsets remain provenance, and aliased records have a single owner.

PCM headers retain the original type, flags, frequency word, loop start and
declared length. Samples are `i8` values with a separate optional signed endpoint
for interpolated waves. CGB waves contain 32 amplitudes in 0..15, packed back
high nibble first. These typed fields reconstruct all 736 new source spans /
1,941,759 bytes exactly, including every signed sample and nonzero endpoint.
No resampling or pitch/envelope/mixer rewrite occurs in this decoder.

Strict JSON and structural checks reject unknown fields, invalid IDs/order,
source extents, link/domain mismatches, sample widths/lengths, orphaned waves,
missing or extra interpolation endpoints, invalid CGB amplitudes and overlaps.
`validate_directory` checks the actual bank roots and the four-byte alias
against the parsed directory. A coherent edit to that shared wave's flags
leaves every newly owned span unchanged, but fails the real owner check.
`verify_source` also rejects structurally valid sample edits.

Warning-free Windows test build 61632 and one Wine check 19316 pass all retained
independent program/key domains, 429 tone fields, 277 header/payload hashes and
736 source hashes, lossless source/JSON roundtrip, 28 structural/JSON/sample
guards and coherent-source/directory-alias failures. The two previously static
CGB sources now pass the original 081497b0..081497cc four-word copy to wave RAM,
with exact RAM writes and SP; 28 total instructions describe coverage only.
Those are explicit internal copy checkpoints, not complete CGB mixer acceptance.
All 492 protected inputs/evidence remain unchanged. The old 390 BD / 348 map /
277 endpoint probes and e41 scene checks were not rerun.

Private output is `target/rom-data-audio-tones/windows-model-europe/audio-tones.json`
and `result.json`, with `windows-model-tests.*`, `windows-model-result.json` and
the frozen `recomp-tests-model.exe`. The catalog is now registered as `audio-tones.json` / `audio.m4a-tones`,
format `audio-tones-v1-json`, in EUR e42-d41. Its manifest dependency names
`audio.m4a-directory`; activation validates the actual parsed directory owner,
including the four excluded alias bytes. All 35 prior decoded files and e41
remain intact; USA stays e8-d7. Managed/runtime validation passes below.

## Typed tone and sample consumption

The runtime stores typed tones, canonical map bytes, signed PCM headers/samples/
endpoints and CGB amplitudes, with a sorted source index. Owned values are
reconstructed from those fields for original CPU, DMA, BIOS and native reads;
unselected records and gaps keep their raw behavior. A read crossing 150946
merges two directory-owned bytes with two wave-owned bytes. The shared first
four wave-header bytes retain their existing directory owner.

The MP2K `MemView` now borrows the same immutable runtime owner for ROM value
reads. Both actual `MemMap` callsites pass it: the per-frame header hook and
sample rendering during `tick`. Header and PCM/DPCM fetch helpers use typed
values; RAM voice structs remain live. GAX/RDRV views keep their existing path.
The original HLE interpolation, pitch, looping and envelope behavior is retained,
including its existing boundary clamp. The separate original ARM interpolation
endpoint remains available through the typed guest/host provider; a direct
host endpoint probe does not claim that current HLE interpolation reads it.

Guest and host counters distinguish tone, keymap, header, PCM, endpoint and CGB
reads. Guest bitsets cover 429 tones, 347 map bytes, 277 waves and two CGB waves;
a separate host wave bitset prevents guest coverage from satisfying the sampler
gate. All counters/indices are immutable-owner metadata outside save states.
No host pointer is serialized, and guest clock/save/native ABI is unchanged.

Warning-free Windows test builds 10352/87016/49176 and two focused Wine checks
84702/3937 pass publication failure/create/reuse/repair/move/e41 retention,
26 coherent field/source/alias/dependency/path/hash guards, transactional missing
files and unavailable-copy Load/Undo/backup/Restart. All 1,941,759 typed bytes
and all record masks read correctly with their raw source bytes altered.

The second check compares complete states/clocks for 25 representative original,
interpreter and compiled paths under the same alteration, including unproduced
program raw behavior. Compiled category reads [115 tone, 7 map, 14 header,
3 PCM, 2 endpoint, 32 CGB] are positive. All 736 span edges, widths, mirrors,
BIOS modes, SWI/DMA, shared two-owner boundaries and fixed8 raw tail pass.
Internal note/header/ARM/CGB checkpoints are explicit, not full mixer ABI proof.
903 instructions, 135 PCs and 55 dispatches describe coverage only.

Both actual MP2K callsites read typed headers and PCM samples for all 277 waves:
4,428 header bytes and 35,458 sample bytes, including rereads. Shadow samples
are nonzero and match bit for bit, with equal complete host state and clocks.
This does not claim every PCM byte was read by the sampler. The 276 direct-host
endpoint probes are separate from HLE rendering's existing clamp. Live RAM and
unowned ROM host fallback remain unchanged. All 499 prior protected files and
frozen test executables remain intact.

Two test-only setup errors are retained: the first managed attempt read the
previous deliberately forged tone file when preparing the next directory guard;
the second test left its BIOS-mode flag set before restoring a baseline state.
Corrections use original directory bytes and reset the test flag. Production and
model code were unchanged; only failed checks were rebuilt/rerun. The successful
managed check was not repeated, nor were old decoder/source/e41 campaigns.
Evidence: `target/rom-data-audio-tones/windows-managed-results.json`,
`windows-managed-europe/{managed-result,native-result}.json`,
`windows-managed-retry{1,2}-tests.*`, `windows-{managed,native}.log` and the
frozen `recomp-tests-managed-retry{1,2}.exe`.

Production CLI and ordinary acceptance are closed below. Complete sequence
export, driver LUTs, unselected assets and new Linux/USA/native Windows remain open.

## Ordinary European tone/sample use and e42 production CLI

Warning-free Windows build 16173 produces the private
`target/rom-data-audio-tones/recomp-rom-data.exe`, SHA-256
`719db176a0d2672de5bc8e0712b6dcdefb6c3b91bec62157e3116e91dd7047d2`.
The canonical executable is restored exactly. Wine CLI 79505 creates e42-d41,
reuses it with all 39 managed file hashes/mtimes unchanged, and runs the existing
400-frame IN13-derived Goblin/Play checkpoint/input from an unrelated cwd using
the old native cache. No new navigation, fixture or state-payload edits occur.

The CLI reads 516,956 new guest source bytes, including rereads: 247 tone,
5 keymap, 11,362 header, 504,912 PCM and 430 endpoint bytes; CGB reads are zero.
That default headless replay retains a disabled shadow sampler, so its zero host
count is explicit. It is not replayed merely to obtain another mode; the existing
scene differential already arms the normal shadow after loading its snapshot.

One new Wine 36911 pair passes all 400 complete states/clocks, central/wide
images, OAM, dispatch counts and mixed/PSG/FIFO/HLE samples. It reads the same
516,956 guest bytes plus 3,093,208 host bytes: 1,056 header and 3,092,152 PCM,
including rereads. The gate requires both positive guest and host PCM use. Mixed
1,379,881 native and 2,741,618 fallback dispatches describe coverage only.
Source/model/targeted native-host and e41 scene checks remain closed.

The final 568×160 CLI capture shows the bank background. The retained snapshot's
separate-UI setting omits panels here, as already documented; this proves no new
food count or confirmation. All protected originals, old generations/cache,
canonical runtime and replay inputs remain unchanged. Private evidence is
`windows-{build,cli-processes,cli-no-write,cli-summary,scene-result}.json`,
`scene-executable.json`, `windows-bank-play-audio-tones-full-state-audio.log`
and `bank-play.png` under `target/rom-data-audio-tones/`.

This closes the declared produced-tone/sample family's managed/runtime/ordinary
scope. It leaves complete sequence export, driver LUTs, unselected bank/sample
records, wider scenes, other asset families, GUI/archive/mod gates and new
Linux/USA/native Windows checks open. No performance or space campaign ran.

## Producer-bounded sequence instructions and driver tables

`audio_sequences::Catalog::from_rom` extends the existing bounded source graph
with physical instruction observations. The new independent inventory confirms
105,253 starts, one interpretation each, and no instruction overlap. The prior
1,610 tracks / 147,424 states and 805 consumed ranges / 177,047 bytes are unchanged.
This new output describes instruction ownership; it does not replay the game or
recreate the old tone/sample evidence.

The strict `AudioSequences` model contains 1,610 track roots and 105,253
instructions: 50,616 waits, 36,579 notes, 12,732 parameter commands, 2,337 pattern
calls, 1,209 ends, 1,094 returns, 401 jumps, 281 end-tie records and four extended
commands. Canonical instruction IDs follow source order; jumps/patterns and track
roots reference those IDs. Source offsets remain provenance. Each instruction
has one physical owner, even when reached through several paths.

Typed fields retain explicit versus inherited opcode encoding, wait/note clock
indices, optional note key/velocity/extension bytes, optional end-tie keys,
parameter command/value pairs and extended selector/value pairs. For an inherited
note, the clock index is authenticated source-graph metadata; only its original
operand bytes are reconstructed. The guest's live running-status byte still
controls execution. A metadata-only clock edit leaves all source bytes unchanged
but fails canonical source authentication. No host sequencer is introduced.

Original wait and note readers use the same 49-byte clock table at 370d04,
indexed by raw wait80..b0 or note cf..ff. Initialization copies exactly 36 words
from 370a90 into mutable command RAM; the model preserves all 144 original bytes,
including six non-command callback slots. Existing RAM patches for B9/CD stay in
original initialization. The produced CD commands select only slots8/9 at
370d58..370d60; only those eight extended dispatch bytes acquire ownership.
The other slots and wider driver LUTs remain separate unowned work.

These three table spans add 201 bytes: 808 source ranges / 177,248 bytes in total,
all disjoint from the 36 existing e42 resource owners. Strict JSON/shape checks
validate counts/order/IDs, nonoverlap, links, opcode/optional-byte grammar, source
extents, table widths/code addresses and actual directory track roots. An implicit
parameter cannot begin with a byte>=80. Canonical source verification also rejects
coherent instruction edits and changes to inherited metadata that encode no bytes.
The original directory root and dynamic bank state remain authoritative.

Windows retry1 build 76630 is warning-free; one focused Wine check 4313 passes all
independent instruction fields/hashes, track roots, source/table roundtrip, strict
JSON and 25 structural guards, coherent byte/metadata edits and the actual parsed
directory dependency. Its 404 new original-reader calls cover 161 dispatch prefixes,
158 note-operand prefixes, 16 jump/pattern returns with stack depth0..2, 12 generic
parameter reads, four CD8/9 returns, 49 wait prefixes, three optional-key CE returns
and one complete 36-word command-table copy. EWRAM/cursor/key/length/SP/outside-stack
and callee checks pass for those declared probes. The 7,020 instructions / 142 PCs
are coverage only. These are explicit source checkpoints or narrow reader returns,
not full sequencer/mixer/state-clock/native or ordinary-scene acceptance.

The first Wine attempt stopped at two wrongly transcribed instruction addresses
in the source-contract assertions, before model/oracle execution. Raw ROM confirms
1c08 at1485ca and38cf at1485cc; the corrected test passes. Initial no-run builds,
failed log/executable/source and final results stay private. All 598 protected
inputs/evidence remain unchanged, and closed tone/directory/e42 scene tests were
not repeated. Evidence and the private `audio-sequences.json` are under
`target/rom-data-audio-sequences/`, including `sequence-instructions-independent.json`,
`sequence-source-plan.json`, `windows-model-retry1-{result.json,log}` and
`windows-model-europe/result.json`.

The managed/runtime integration below advances EUR to e43-d42 / USA e8-d7.
Full driver LUTs, unselected sequences/assets, other families, GUI/archive/mod
criteria and new Linux/USA/native Windows checks remain open.

## Managed sequence publication and typed guest reads

EUR e43-d42 registers `audio-sequences.json` / `audio.m4a-sequences`, format
`audio-sequences-v1-json`. The 38 files / 37 resources include all 36 previous
decoded files, and e42 is retained; USA stays e8-d7. The manifest names both
`audio.m4a-directory` and `audio.m4a-tones` as dependencies. `validate_owners`
walks the typed instructions and actual directory roots, preserving opcode
inheritance, key/program state and pattern stack rules, and compares all 390
program/key domains with the actual parsed tone banks. A bank note-key metadata
edit can leave all encoded tone bytes unchanged yet still fail this dependency.

The immutable runtime owner reconstructs bytes directly from typed instruction
and driver-table fields. CPU, BIOS copies, DMA and native bus calls retain the
original code/RAM/clock behavior. Boundary reads merge the final e42 PCM bytes
with the first sequence bytes; unowned padding and other extended dispatch slots
stay raw. The generic audio host view also merges the new owner so an owned byte
cannot silently fall back to its ROM copy. This adds no host sequencer.

Counters distinguish opcode, operand, clock, command-table and extended-table
bytes. Instruction bits and table masks separately observe all 105,253 records,
49 clock entries, 36 command words and two extended slots. These counters/index
fields and generic host counts belong to the immutable owner outside snapshots;
no save or native-cache ABI changes.

Warning-free Windows build 11482 and two focused Wine checks 49125 pass publication
failure/create/reuse/repair/move/e42 retention, 25 coherent source/field/owner/
dependency/path/hash guards, transactional missing files and unavailable-copy
Load/Undo/backup/Restart. All 177,248 bytes read correctly with their raw source
copies altered: 91,663 opcode / 85,384 operand / 49 clock / 144 command / 8 extended
bytes. All 105,253 instruction bits and table masks are observed. Direct generic
host reads after lifecycle restoration use the same typed bytes without guest
usage or clock changes.

Thirty-five representative original/interpreter/native paths with raw source
altered have equal complete states/clocks. Compiled category reads [15 opcode,
36 operand, 10 clock, 144 command, 8 extended] are positive. The live RAM running
status can override the metadata interpretation, and unowned raw operands retain
their original behavior. All 808 source edges, widths, mirrors, BIOS modes,
SWI/DMA, raw padding and the two-tone/two-sequence guest/generic-host boundary
pass. Dispatch/note/wait internal checkpoints and narrow branch/operand/CD/CE/
copy returns are explicit; these are not whole sequencer/mixer ABI proof.
953 instructions / 142 PCs / 81 dispatches describe coverage only.

There were no new build or Wine failures. Private generator preview corrections
were applied before repository output. All 627 previous protected inputs/evidence
and the frozen test executable remain unchanged. The source404 and earlier e42
proofs were not repeated. Evidence is `windows-managed-tests.*`,
`windows-managed-results.json`, `windows-{managed,native}.log` and
`windows-managed-europe/{managed-result,native-result}.json` under
`target/rom-data-audio-sequences/`.

Production CLI and ordinary acceptance are closed below; wider driver LUTs,
unselected assets, other families and GUI/archive/mod criteria remain open.

## Ordinary European sequence use and e43 production CLI

Warning-free Windows build 48965 creates the private
`target/rom-data-audio-sequences/recomp-rom-data.exe`, SHA-256
`355ebc024334d6d0791931692ee0fd8f2eeeb4107f58e8becc91b272668da5e6`.
The canonical executable is restored exactly. Wine CLI 48325 creates e43-d42,
reuses it with all 40 managed file hashes/mtimes unchanged, and runs the existing
400-frame IN13-derived bankPlay checkpoint/input from an unrelated cwd with the
old native cache. No new navigation, fixture, RAM/state-payload or personal-save
edits are involved.

The CLI reads 819 new source bytes, including rereads: 366 opcode, 262 operand
and 191 clock bytes. Reads observe 185 instruction source records, including
lookahead; this is not a count of executed instructions. Command-table, extended
and generic-host reads are zero in this loaded scene. Original initialization
and other extended commands are not inferred from positive sequence use.

One new Wine 29123 pair passes all 400 complete states/clocks, central/wide
images, OAM, dispatch counts and mixed/PSG/FIFO/HLE samples, with the same
819 bytes / 185 source records. All three opcode/operand/clock gates are required
positive. Mixed 1,379,881 native / 2,741,618 fallback dispatches describe coverage
only. The wider declared source family contains 105,253 instructions / 177,248
bytes; this ordinary scene does not claim to read them all. Source404/native35
and earlier e42 acceptance were not repeated.

The final 568×160 CLI image shows the bank background. Its known serialized
separate-UI setting omits the panels here; it establishes no new food count or
confirmation. Protected originals, generations/cache, canonical runtime and replay
inputs remain unchanged. Private evidence is `windows-{build,cli-processes,
cli-no-write,cli-summary,scene-result}.json`, `scene-executable.json`,
`windows-bank-play-audio-sequences-full-state-audio.log` and `bank-play.png` under
`target/rom-data-audio-sequences/`.

This closes e43 managed/runtime/ordinary acceptance for the declared sequence and
initial table scope. Remaining driver pitch/volume/other LUTs need their original
reader/index/bounds contracts, alongside unselected audio assets and the other
families/GUI/archive/mod TODO. New Linux/USA/native Windows checks remain pending;
no benchmark or space campaign ran.

## Original pitch, rate, noise and volume lookup tables

Seven adjacent source tables at370b20..370d04 add 484 bytes, with no overlap
with any of the 37 e43 owners. Their independent roots and original readers
prove 180 packed PCM key codes, 12 unsigned 32-bit PCM bases, 12 unsigned
16-bit sample counts, 132 packed CGB key codes, 12 signed 16-bit CGB bases,
60 noise-register codes and 16 wave-volume codes. The region follows the
already owned 36-word command table and ends at the already owned clock table.
The queued370eb0/b94690 roots are reclassified by their original readers:
370eb0 is a general copied-IWRAM memory-helper vector; b94690 is palette-effect
dispatch. They are not additional M4A lookup tables.
[Classification](rom_data_graphics.md#palette-effect-dispatch-source-contract).

PCM helper08148a5c truncates key/fraction to their low bytes. Keys above178
become178 with fraction255. It reads key and key+1, so the physical map includes
endpoint179. Each packed byte supplies a base index and right-shift nibble;
the current source uses exactly12 base indices. Interpolation uses two unsigned
multiply-high operations through Thumb147ec4 / ARM147ec8. The model preserves
raw frequency/base words and that arithmetic without inventing frequency units.

CGB helper08149510 truncates type/key/fraction to low bytes. Type4 selects
noise: keys<=20 choose0, then key-21 clamps at59. Other types use signed tonal
bases: keys<=35 choose index0/fraction0; otherwise key-36 clamps at130 with
fraction255. Reading the next entry proves endpoint131. Signed halfword loads,
arithmetic shifts and integer interpolation stay distinct from the PCM path.
These rules describe the helper interface, including targeted type4 coverage;
they do not claim every tone type occurs in ordinary gameplay.

The direct frequency initializer08148fcc masks a four-bit frequency selector,
then reads a halfword at `sample_counts + 2*(selector-1)`. Twelve physical
entries are bounded by the next independent CGB-map root. There is no local
12-entry clamp: selector0 reads the high halfword of the final PCM base, and
selectors13..15 read the first six CGB-map bytes. All have a single typed owner.
Public SoundMode08149070 skips the helper when the selector is zero. A direct
lookup diagnostic therefore must not be confused with that public gate.

Wave-volume reader08149a32 uses the unmasked live channel byte09. Its physical
map has sixteen entries, bounded by the independent clock root. That does not
prove a universal sixteen-level producer domain: original gain helper08149608
clamps at15 in the single-sided route, but balanced routing branches at149640
directly to149652 and skips the clamp. Targeted live inputs128/128 produce16,
255/128 produce23, and255/255 produce31; ordinary reachability of those inputs
is not established. Levels16/64 read e43 clock entries0/48, and255 reaches an
unowned byte. No new runtime index check is inferred. `wave_volume_code` reports
membership in the physical sixteen-entry table only; guest execution is not
routed through that diagnostic.

`AudioDriver` stores strict typed tables, decomposed key nibbles, unsigned/signed
integer bases and complete raw hardware codes, including unused bits. Table name
and index are stable logical identifiers; source offsets remain explicit
provenance. Source-byte reconstruction keeps internal sample-selector aliases
without a duplicate raw overlay. Sequence-owner validation checks the actual
neighboring command/clock boundaries. Diagnostic integer helpers support source
comparison, while guest mixer/sequencer behavior remains original.

Warning-free Windows build87853 and one focused Wine check60589 pass all
independent fields/seven hashes/484-byte source and JSON roundtrip,15 structural/
JSON guards, coherent signed-base/unsigned-unused-volume-bit edits and the actual
sequence boundary owner. The1,138 new original-reader probes include541 complete
PCM returns,300 complete tonal CGB returns,259 complete noise returns,18 sample
count prefixes,19 volume prefixes and one complete public zero-frequency return.
EWRAM is read-only or matches predicted prefix writes; SP/callee/outside-stack
guards pass. The observed original load sites read all484 distinct owned bytes
and three adjacent aliases at370d04/370d34/370df3. The unsigned ARM UMULL helper
executes;52,869 instructions/189 PCs describe coverage only.

A final static review found that the initial unconditional cap15 inference
missed the balanced-route branch. A separate narrowed Windows build59773 and
Wine27372check executes only eight new gain-helper returns, confirming levels
16/23/31, routing masks, sustain values and exact EWRAM/SP/callee/outside-eight-
byte-stack guards. It adds no table bytes and does not repeat the successful
1,138-case check.290instructions/52PCs describe coverage only. The model needed
no change: it already preserves out-of-table aliases instead of guarding guest
indices. `windows-gain-{result.json,log}`, `windows-gain-europe/result.json` and
`gain-contract-correction.json` retain this corrected producer contract.

These probes stop at explicit sample-count/volume checkpoints and do not prove
full initialization, CGB mixing, state-clock/native or ordinary equivalence.
All879 protected inputs/evidence remain unchanged, with no build/Wine failure.
Closed source404/native35/e43ordinary/tone277 checks were not repeated. Evidence,
including the private `audio-driver-tables.json`, is under
`target/rom-data-audio-driver/`: `driver-independent.json`, checked disassemblies,
`windows-model-{result.json,log}` and `windows-model-europe/result.json`.

Managed extraction remains e43-d42 / USA e8-d7. Next: publish this driver resource
with its real sequence boundary dependency, route original CPU/DMA/native and
generic host byte reads, preserve internal/external/raw aliases and validate
lifecycle plus one new positive ordinary Wine pair. Other wrapper roots/assets,
GUI/archive/mod criteria and new Linux/USA/native Windows checks remain open.

## Managed driver tables and original integer reads

EUR e44-d43 adds `audio-driver-tables.json` / `audio.m4a-driver-tables`, format
`audio-driver-tables-v1-json`. The 39 files / 38 resources retain all 37 prior
decoded files and e43; USA stays e8-d7. The seven spans own 484 bytes, with the
actual parsed sequence resource as dependency: its command table ends at the
first PCM key and its clock table begins immediately after the volume codes.

Runtime reads derive each byte from the validated typed fields. CPU, DMA,
BIOS, native dispatch and generic audio host views merge owners at boundaries.
Packed key nibbles, unsigned32 PCM bases, signed16 CGB bases and raw register
codes retain their original widths and bits. Counters track seven byte classes
and 424 physical entries outside save state. Guest arithmetic and RAM indices
remain original; no host pitch/mixer rewrite or serialized pointer is added.

Sample selector0 reads the final PCM base's high half; selectors13–15 read
CGB key bytes. Volume16/31/64 read the existing clock owner, while255 reads
unowned raw ROM. These physical aliases are deliberate: the conditional gain
cap does not justify a universal volume15 clamp. Out-of-family bytes delegate
to their existing owner or remain live raw ROM.

Windows70748 builds the managed tests without warnings. Wine51847 runs two
focused checks, both terminal/pass: publication failure/create-reuse-repair-move,
e43 retention/all37prior decoded files,19coherent authentication guards, missing
file activation isolation and unavailable-directory Load/Undo/backup/Restart.
All484typed bytes and424physical entry bits pass with raw source altered; the
post-lifecycle generic host view reads28typed bytes without guest counter use.

The second check covers28representative original/interpreter/native paths with
complete states and clocks equal after restoring poisoned ROM bytes. Compiled
byte categories are [10PCMkey,42PCMbase,6samplecount,18CGBkey,24CGBbase,4noise,
2volume], all positive. The original Thumb/ARM unsigned multiply-high helper
executes. Sample0/13/14/15 aliases and volume16/31/64 prior clocks/255raw pass;
all7span boundaries/widths/mirrors/BIOS/SWI/DMA and two-owner guest/host words
pass.844instructions/150PCs/70dispatches are coverage counts only. Pitch full
returns and rate/volume internal checkpoints do not establish the whole ABI.

917old protected files plus the frozen test executable remain unchanged. No
build/Wine failure; closed source/model1138/gain8/earlier proofs are not repeated.
Private evidence: `target/rom-data-audio-driver/windows-managed-results.json`,
`windows-managed-europe/{managed-result,native-result}.json` and associated logs.
Production CLI/ordinary e44 acceptance is complete below; new Linux/USA/native
Windows checks remain pending. Generic host access alone
does not establish host driver arithmetic or a complete initializer/mixer ABI.


## Ordinary European driver-table use and e44 production CLI

Warning-free Windows build85643 produces the private
`target/rom-data-audio-driver/recomp-rom-data.exe`, SHA-256
`376f98f15e1504c97d17e515e04ad9c13a8c9ade25c1fab9ce521a3a012afbd8`.
The canonical Windows executable is restored. Wine30529 passes actual CLI
create/reuse without writing41managed files, old native cache loading and the
retained EUR bankPlay scene from an unrelated working directory.

The 400-frame CLI reads756source bytes from the new family:132PCM key bytes,
528PCM base bytes,32CGB key bytes and64CGB base bytes. These are aggregate reads,
not distinct source bytes.67physical entries are observed, with per-table counts
[31,12,0,16,8,0,0]. Sample-count/noise/volume/generic-host ordinary reads are zero;
their targeted interface checks remain the evidence, without wider scene claims.

One new Wine21679 full differential compares every400complete states/clocks,
central/wide images, OAM, dispatch behavior and mixed/PSG/FIFO/HLE audio samples.
All match with the same756source-byte reads/67entries and required positive PCM
key/base use. First frame3753; mixed1379881native/2741618fallback dispatches are
coverage counts only. The existing EUR checkpoint/input are reused unchanged;
there is no new GUI navigation, fixture creation or guest RAM/statepayload edit.
Source1138/gain8/native28/older family acceptance stays closed and is not repeated.

The final568x160CLIbank background is inspected. The known separate-UI omission
remains; this is not a new food-count or confirmation proof. No build/Wine failure
or live job remains. Evidence under `target/rom-data-audio-driver/`:
`windows-{build,cli-processes,cli-no-write,cli-summary,scene-result}.json`,
`windows-bank-play-audio-driver-tables-full-state-audio.log` and `bank-play.png`.
Other wrapper roots/LUTs/unselected assets and graphics/text/maps/gameplay,
GUI/archive/mod criteria and new Linux/USA/native Windows checks remain open.
No performance or space measurement was taken.
