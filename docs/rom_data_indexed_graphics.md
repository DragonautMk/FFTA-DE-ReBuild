# Indexed graphics archives

## European source contract

`ROMDATA-01-INDEXED-GRAPHICS-ARCHIVES` classifies four independently rooted
five-slot pointer groups at ROM offsets `71e57c`, `71e61c`, `71e630` and
`71e658`. Their 20 distinct archives contain 649 indexed programs. The declared
source has 24 disjoint spans: 74827 archive bytes and 80 pointer bytes, totaling
74907 bytes. Independently reconstructed output totals 278336 bytes. This
source contract does not yet add a managed resource or change schema1/e90-d89.

The first group contains 30 programs per language, each producing 192 bytes.
The next two groups each contain four programs per language, producing 7168
and 3840 bytes per program respectively. The fourth group contains
85/91/93/91/99 programs, each producing 64 bytes. These counts come from each
archive's big-endian count and actual original readers, rather than distances
between roots. No overlap exists with the 82 previously accepted resources or
their 50498 source spans.

Original pointer selection loads the live unsigned byte at `02001f6e`, scales
it by four and reads a little-endian pointer. Slots 0 through 4 are the declared
physical banks. Typography/layout fallbacks elsewhere in the callers do not
normalize this pointer index; `0ac2a8` loads the pointer before comparing with
four. Queries outside the physical slots retain their original raw address.
The separate context at `3710d4` was unowned at this revision; its later
EUR integration is described in [the context extension](#european-indexed-graphics-contexts).

## Archive and fixed dictionary encoding

Original wrapper `0800537c` reads two opaque header bytes, a big-endian u16
program count, a big-endian u32 dictionary-relative offset and one big-endian
u32 archive-relative offset per program. All 20 declared headers happen to
contain `4137`; the original wrapper does not check this value, so it is not
a sufficient format discriminator. This encoding is distinct from the custom
LZ decoder at `080054a0`.

Original entry decoder `08005228` reads a mode and width from the dictionary
header. All declared archives use mode0/width5 and a big-endian u16 decoded
size shared by their programs. A token with its high bit set encodes
`(low7 + 1)` following literal bytes. Otherwise a big-endian u16 encodes a
fixed dictionary reference: length `(word >> 10) + 3`, offset `word & 1023`.
References read an external shared dictionary, rather than previously decoded
output. Every source byte, including unreferenced holes inside the observed
dictionary prefix, is preserved.

The declared dictionary extent ends at the highest actual referenced
offset-plus-length across that archive's programs. Those prefixes total
15631 bytes. Later tails, interbank alignment and gaps remain raw/unowned.
Canonical program streams are contiguous after their index and end exactly
at the dictionary header; this was checked for every program, not inferred.

The wrapper truncates start/count arguments to their low16 bits. An invalid
start returns failure1 before initializing destination or output globals.
Requested count0 or a count greater than the archive count initially means
the whole count, then clamps to the remaining programs. Selected outputs are
concatenated. Entry success is1/failure0; wrapper success is0/failure1.
Mode3 reads a size from each stream; other modes fail. These noncanonical paths
and wider width/ARM register-shift behavior still need model/original-reader
proof. No arbitrary width cap is established by the canonical width5 data.

## Static verification and remaining work

The source classifier authenticates 22 disassembly windows (20 new, two reused),
101 instruction/literal pins, all ten exact root literals and 12 fused Thumb
BL calls to the wrapper. All 74907 source bytes reconstruct exactly, SHA256
`ab21ada958cc5bcb13f06b59218bf9b4757155684c794881dd9ca7c0f4859006`.
All program outputs reconstruct exactly, combined SHA256
`b380cbb85eb5606a94622362822e09d2d15761343e3b4a088a294a314a862559`.
The 121 hash/mtime guards preserve all 83 previously accepted dataset files
and actual immutable inputs. Private evidence is retained under ignored
`target/rom-data-indexed-graphics-archives/`.

This unit is static only: no build, Wine/game run, checkpoint derivation,
navigation or older acceptance replay was performed. The other 16 nearby
pointer groups remain unclassified. Global archive extent, indirect aliases,
opaque contexts, strict editable model, original-reader Wine proof, managed
publication/runtime/native clock, actual CLI and ordinary use remain pending.
USA, Linux, native Windows and new32:9 verification remain pending; no
performance or storage campaign is included.

## Typed model and original wrapper proof

`ROMDATA-02-INDEXED-GRAPHICS-ARCHIVES` adds
`gba_core::rom_data::indexed_graphics`. Strict JSON preserves the opaque header,
BE index/dictionary offsets, packet boundaries, literal bytes, dictionary
length/offset references, complete observed dictionary prefix and exact decoded
little-endian words. Physical groups/archives/program order, counts, sizes,
contiguous source layout, dictionary reference bounds, produced lengths,
decoded words and disjoint extents are checked. Editable shape validation is
separate from exact EUR ROM, reader fingerprints and source authentication.
An opaque-header edit can pass shape validation while source verification fails.

This host model deliberately covers declared mode0/width5 archives. Rejecting
other modes/widths during host parsing does not add a check or cap to original
guest execution. `PointerGroup::select` returns no physical bank for languages
5/255; `encoded_view` still retains the original raw address. Archive selection
uses the same low16 start/count projection, zero/oversize-count interpretation
and remaining-program clamp as original `0537c`. An invalid start returns no
selected slice rather than implying destination initialization.

One warning-free Windows GNU core build uses 595 frozen source files: the
accepted 593-source ordinary snapshot plus two new model/test leaves and one
module addition. Parallel frontend/container/package edits are excluded.
One focused Wine filter passes 37 malformed-shape/JSON/source guards, all
20 complete archive calls (all 649 programs/278336 decoded bytes), nine
low16 start/count boundary calls, nine isolated synthetic streams and 28 raw
language selector suffixes. Synthetic cases cover fixed/far dictionary reads,
maximum 128-byte literal and 34-byte reference packets, zero output, output
overrun and unsupported modes. They use controlled test inputs, without
deriving or modifying ordinary game checkpoints.

Actual original wrapper results, complete destination/output guards, output
globals, restored SP and R4 through R11 match. Complete EWRAM and IWRAM outside
the original 68-byte stack scratch, VRAM, OAM and palette match their expected
contents. Original stack scratch is permitted to retain its writes; final ROM
bytes are unchanged. Selector proof executes the shared raw-byte/scale/load
suffix with each group's base, rather than claiming full caller execution.
The 5993129 executed instructions are a coverage count only. Source/model
proofs and older acceptance were not repeated. Managed publication, native
clock/bus integration, CLI and ordinary use remain separate unfinished units;
mode3/wider widths, full universal ABI/live uploads and deferred platforms
remain unverified.

## Managed source views

The new EUR registration is schema1/extractor91/decoder90, resource
`graphics.indexed-archives`, file `indexed-graphics.json`, format
`indexed-graphics-v1-json`, index82 with no dependencies. It appends one resource
to the 82-resource e90 dataset, for 84 files/83 resources. USA retains its
previous revision and remains unverified. Older accepted generations remain
separate evidence.

Attachment authenticates the complete selected dataset before publishing an
immutable owner. Runtime source bytes are rebuilt from validated typed
headers, index offsets, packets and dictionary storage. Original guest
decoding, scratch writes, copy counts and uploads remain active. Exact byte
membership composes with earlier owners and raw ROM on the same page, including
the two-byte gap between world records and the first new selector group.
Source bytes are retained by the owner when files become unavailable and
across Load/Undo; counters and source storage do not alter serialized state or
the native ABI.

Twelve counters expose read/source-byte totals, selector/header/index/token/
literal/dictionary-header/dictionary byte counts and distinct archive/program/
group usage. Full physical coverage is 80 selector bytes, 160 archive-header
bytes, 2596 index bytes, 39955 token bytes, 16405 literal bytes, 80 dictionary
header bytes and 15631 dictionary bytes, with 20 archives/649 programs/four
groups. Coverage counters are not performance measurements.

Runtime verification passes one reduced Wine filter. The first host test
compilation failed because the new test lacked the `Bus` trait import;
production code compiled. The corrected 597-source build is warning-free and
changes only that import, retaining both snapshots and first diagnostics.
All 82 previous resource records/files remain identical. Publication interruption,
14 transactional attachment rejection cases and two full-set guards pass,
including unchanged prior owner and complete state on failed attachment.

Thirty-five complete-state/clock cases compare raw interpreter, typed interpreter
and generated native execution with all 74907 new ROM source bytes and 1178
prior world-record bytes altered after Load/setup. They include 28 controlled
loads, one original `0537c` call decoding one 192-byte program, and six raw
language suffixes. Three native families exclude their stop boundaries and
retain loaded libraries. All actual memory callbacks, including `interp_one`,
match the independent observed load counts: 272 served new bytes/four prior
world bytes, including one new byte through fallback. These are served counts,
not a distinct-byte union or an all-direct-native claim. The 232 visited
instructions and 59 dispatches are coverage only.

Every source byte/seven classes/20 archives/649 programs/four groups, all
24 span boundaries/widths/mirrors, prior-world/raw-gap composition and unavailable
files across Load/slot Undo pass. DMA, CpuSet and CpuFastSet each copy the same
64-byte mixed prior-world/gap/new-selector range with exact full state/clock.
The 282 preservation guards cover 597 accepted and 597 diagnostic sources,
595 model preimages, seven current paths and 83 prior dataset files.
Private accepted evidence is `target/rom-data-indexed-graphics-runtime/` with
`accepted-selection.json` authoritative; `selection.json` retains the first
compilation diagnostic. Actual production CLI and ordinary game use remain
pending. No ordinary fixture/navigation or earlier acceptance was repeated.


## Actual CLI registration

`ROMDATA-04-INDEXED-GRAPHICS-CLI` passes one warning-free production Windows
GNU build from the same accepted 597 runtime sources, without overlays.
The canonical executable hash/mtime is restored in a finally block. One Wine
orchestration runs three successful commands from an unrelated working
directory: managed creation, no-write reuse and one neutral frame from a copy
of the retained normal24755 checkpoint. All 84 generation files equal the
accepted runtime dataset, all 83 logged resource ID/hash pairs match, and all
86 managed paths retain their hash/mtime during reuse/startup. The unchanged
old native cache loads 28859 blocks.

The frame reaches24756 with all twelve new usage counters zero. This verifies
registration and compatibility; positive ordinary use and full scene
equivalence remain pending. The 201 guards preserve both 597-source snapshots,
84 accepted files, ROM/seed/cache/TinyCC and canonical executable. No ordinary
checkpoint was derived and no navigation, earlier proof or metric was repeated.
Private CLI evidence is `target/rom-data-indexed-graphics-cli/`; its selected
production executable SHA256 is
`d8767645ab8cbc7203e0ac6bb48e810feaab2003a4dcad7300361f01c09ffeb4`.


## Ordinary world-map menu equivalence

`ROMDATA-04-INDEXED-GRAPHICS-ORDINARY` verifies one normal EUR world-map menu
case under Wine. Eight authenticated caller windows (six new/two reused) and
the fused call at `0453e2` join context3 at `3710e0` to archive program2, whose
accepted decoded tiles spell MENU. Context index and program index are separate;
this does not establish ownership or an enum for the context table. The retained
normal4519 world-map checkpoint and old input receipt authenticate the trigger.
Only Start is used: press at relative8, release at20, neutral otherwise. The
older Down/A/320-frame Clan route and other completed proofs are not replayed.

One warning-free 598-source Windows host-test build adds a passive observer
and optional five-line positive-usage guard to the accepted 597-source CLI
snapshot. Existing parameter/world observers stay in current source; only the
parameter observer/guard is excluded from this frozen baseline to match the CLI.
Core/runtime remain unchanged and both module/validator inverses are exact.
A private preparation variable collision was recovered offline using the same
already copied seed/source snapshot, before any build or Wine execution.

One normal capture becomes positive at relative9/guest4529 and stops at4560,
after41frames from4519. Its rolling41-frame suffix is accepted immediately;
discovery performs no further navigation. The same production CLI runs this
suffix once with all83 ID/hash pairs and86 managed paths unchanged and the old
28859-block native cache intact. The same test executable runs one full pair:
all41 complete states/clocks/classic-wide images/OAM/five audio streams match.
Both arms use the same old native cache; this is not global interpreter/native
equivalence. A nonfatal Wine D-Bus warning remains in the successful pair log;
no retry was performed.

All twelve counters match capture, CLI and full pair:274 reads/277 served bytes,
including selector4, archive header6, index4, token67, literal17, dictionary
header4 and dictionary175, with one archive/program/group used. All seven byte
classes are positive; the case does not cover all20archives/649programs/four
groups. The205083native/283493fallback dispatches and five audio sample counts
are coverage only. Complete284x160 wide framebuffer pixels authenticate the
readonly classic projection showing MENU with Party/Clan/Area List/System and
MoonDay9. No option selection or mission outcome is claimed.

The144 guards preserve598 frozen/597CLI sources, three current implementation
paths, original ROM/checkpoint/input/cache/TinyCC/canonical executable and86
managed files. Game controls on copies are the only new guest inputs; no manual
RAM/register/task/ID/RNG/ROM or cartridge-save edits were made. Source/model/
runtime/CLI registration acceptance, old ordinary routes and performance
measurements were not repeated. Other ordinary archives/programs/languages,
mode3/wider widths/full ABI/live effects/indirect/global extent, Linux/USA/
native Windows/new32:9 and the whole ROM TODO remain pending. Private evidence
is retained under `target/rom-data-indexed-graphics-ordinary/`.

## European indexed graphics contexts

`ROMDATA-01-CONTEXT-3710D4` extends `graphics.indexed-archives` with 33
four-byte context records at `3710d4..371158`. Original helper `019670`
returns `3710d4 + 4 * index`; the caller at `01967c` uses fields 2 and 1 as
archive program indices, and `01972c` reads field 0 after truncating its
argument to a byte. The fourth byte is preserved as opaque. The explicit
index 32 supplied at `13cc48` and the independently rooted table at `371158`
support the selected extent. They do not prove that all 33 records are
reachable in ordinary play, or that other callers cannot calculate addresses
outside it. The original arithmetic and raw fallback remain unchanged.

EUR schema 1/e159-d158 adds a strict context projection to the existing JSON
resource, with one 132-byte source binding. The complete indexed resource now
owns 25 spans and 75,039 bytes; its prior archive/group projection is unchanged.
The context source SHA-256 is
`3461a0d663b5f2a27d74ecaaf4670d3519a7e3eaad4f4161d41202866807d593`.
The runtime supplies those bytes from the typed fields, composes boundary reads
with prior owners or raw ROM, and exposes `context_bytes` in its existing
usage report. This counter records source use, not performance.

One focused Windows GNU/Wine test passed preparation and reuse, comparison
with all e158 resource files, 132 poisoned-source byte reads, boundary/mirror
reads, malformed-model and edited-source guards, and 24 original-reader cases.
The latter compare complete state and clock for interpreter and generated
dispatch; generated dispatch can fall back to interpretation. The same test
reuses an authenticated ordinary world-map input for 11 interpreter frames:
four typed requests hit context 3, none hit raw context bytes, and both arms
match in state/clock, central and wide framebuffer, OAM and mixed audio. This
is one bounded input window, not a full menu or all-context proof. Evidence:
`target/validation/units/e159-indexed-context-wine-8dbsgppp/`.

The E170 dataset also passed the complete 41-frame recorded world-map input in
Wine (`indexed-context-route-wine-5eyv63je`), with all 132 source bytes altered
in the managed machine. Both arms matched full state, clock, central and wide
framebuffer, OAM, mixed audio, PSG, both FIFOs and HLE audio each frame; all
five audio streams contained samples. Four typed requests consumed four bytes
of context 3; no raw or unclassified requests and no dropped trace requests
were observed. There were 205,083 generated and 283,493 fallback dispatches;
the aggregate trace does not attribute individual reads to native instructions.
Other contexts and the full reader ABI are not covered by this input.

Wine `indexed-context-direct-wine-ax_5va8t` repeats that same recorded route
with a test-only generated-API observer to resolve the attribution left open
above. All four traced context-3 requests, serving four typed bytes, pass
directly through generated API calls; none is nested inside `interp_one` and
no raw or unclassified context requests occur. The 41 raw/managed frames again
match complete state, clock, video, OAM, dispatch and five nonempty audio
streams with all 132 context-source bytes altered. The wrappers do not identify
exact guest load PCs. Other contexts, callers, reader ABI and platforms remain
open. Protected inputs passed, Wine drained and successful scratch was removed.

A separate controlled E170 Wine test (`indexed-context-domain-wine-9l5p79sq`)
compares getter `01972c` for all 256 byte indices and their `0x100` aliases,
using both interpreter and generated instruction-block dispatch: 1,024 cases.
It also compares the three original field-load instructions for every one of
the 33 records in both modes: 198 cases. All complete states and clocks match
with the 132 managed source bytes altered. The generated blocks may fall back
to interpretation; controlled registers establish neither ordinary index
reachability nor every caller or the complete reader ABI. The first attempt
`dyn2l97k` failed because the test object omitted the called helper `019670`;
the corrected build includes it. An intervening build ended with a broken-pipe
unit error after its terminal handle was interrupted; it was not used as proof.

The same getter can address the separately owned bounded-range source at
indices 206..227. E170 Wine `bounded-range-context-overlap-wine-3lrpx1l2`
checks those indices, both neighboring indices and `0x100` aliases in both
dispatch modes: 96 full-state/clock comparisons pass with all 88 range-source
bytes altered. The 88 in-range cases each serve one range byte, and the eight
neighbor cases serve none. This controlled test does not establish ordinary
index reachability or all callers; generated blocks may fall back.

The production Windows CLI also passed under Wine in
`e159-indexed-context-cli-wine-27uqnksb`: it created, reused and verified an
e159 dataset whose 104 files matched the dataset above. Reuse and an 11-frame
world-map run left the generation unchanged; the CLI reported four served
context bytes. This run uses the retained translation cache and does not add a
full-scene equivalence claim. A nonfatal Wine D-Bus diagnostic appeared, but
the command and validation unit completed successfully and the server drained.

Additional ordinary contexts, full reader ABI/indirect consumers, broader
affected Wine regressions, native Windows, Linux, USA and the remaining
ROM-data TODO stay open. No performance measurement was performed.

## Direct European archive at 3ae484

The original caller at `08023c98` loads `083ae484` from its literal at
`08023d08`, then calls the existing indexed-archive wrapper `0800537c` with
start and count both zero. The next direct call uses the separate archive at
`3ae5a4`, classified below. The caller also passes
the decoded outputs to the original upload helper. These instructions support
the archive interpretation independently of its opaque header. A bounded
ordinary-input continuation later observes both archives, as described below.

Static EUR/E164 audit `e169-direct-indexed-source-audit-j80q8jx4` authenticates
the caller and both wrapper calls. The first archive at `3ae484..3ae5a4` has
four 64-byte literal programs, mode 0/width 5 and no referenced dictionary
payload. Its 288 source bytes decode to 256 bytes. The source SHA-256 is
`967d4b1bdbba077df4ea9b5b4aa490bef3ec5a9a7509f728e99ba8f999999da5`;
the decoded SHA-256 is
`32f36ef4c8f2a5eef8780010587399e0199a75fe72c771f2bc071e783a7d6e4e`.
Neither the four-byte caller literal nor the archive overlaps an E164 source
owner. E165-d164 adds a separate typed resource for these spans, retaining
the original guest decoder and the existing language-group contract. Wine
test `e170-direct-indexed-runtime-wine-qk3xqc4z` creates and reuses the dataset,
preserves all 107 previous resources, and serves all 292 typed source bytes
with their ROM copies altered. Mixed-width boundaries and a malformed decoded
word are checked. The controlled original caller and decoder produce identical
full states and guest clocks for raw and managed reads using interpreter and
generated instruction dispatch; generated blocks may use interpreter fallback.
The Wine server drained after the test. Production Wine CLI test
`e170-direct-indexed-cli-wine-e87ug9pq` creates, reuses and verifies all 109
files matching that dataset. Its 11-frame world-map window leaves the dataset
unchanged and reads no bytes from the new archive, so it does not establish
ordinary use. The Wine server drained. A later ordinary continuation observes
both archives; other scenes, Linux, USA and native Windows remain pending. No
performance measurement was made.

## Adjacent direct European archive at 3ae5a4

The next original caller loads `083ae5a4` from `08023d10` and passes it to
the same indexed wrapper. Static EUR/E165 audit
`e171-adjacent-indexed-source-audit-zy8p2uxf` parses 41 programs of 64 decoded
bytes, using 63 literal and 89 dictionary packets. The exact used archive
extent is `3ae5a4..3aed7c`: 2,008 source bytes including a 96-byte referenced
dictionary prefix, producing 2,624 decoded bytes. Its archive source SHA-256
is `170a986e3eb78464227cc6a14f71c5e102c355746f4a3e8ecc41a97feb2cd83c`;
the decoded SHA-256 is
`668112f0c7946c0ad46564c6c79649a615560c1054116807fb6113d8395ef1cd`.
Neither this archive nor its four-byte caller literal overlaps an E165 owner.
E166-d165 publishes these spans as a separate typed resource. Wine test
`e171-adjacent-indexed-runtime-wine-puxz2kz7` creates and reuses the dataset,
preserving all 108 previous resources. It checks all 2,012 typed source bytes
with their ROM copies altered, mixed-width reads across both the new source
boundaries and the previous archive, and an invalid decoded word. A controlled
original caller and decoder match raw and managed full states and guest clocks
through interpreter and generated instruction dispatch; generated blocks may
use interpreter fallback. Production Wine CLI test
`e171-adjacent-indexed-cli-wine-ozq4vcii` creates, reuses and verifies all 110
files matching the tested dataset. Its 11-frame world-map window leaves the
dataset unchanged and reads no bytes from this archive. The Wine server drained
after both tests. A later ordinary continuation observes both archives; other
scenes, Linux, USA and native Windows remain open. No performance measurement
was made.

### Source-guided ordinary-use lead

A static EUR search (`m5katm09`, `bckq9lmb`) finds three syntactic Thumb calls
to loader `08023c98`; it does not establish a complete caller census. One is
inside script command `0x49`, which calls the loader only when its operand and
the original flag check permit it. Source join `p0qt7o6q` identifies such an
instruction in program 1 between the recorded cursor positions of two genuine
normal-input checkpoints. That relationship is a lead, not proof that the
instruction, its branch or either archive was used in the recorded interval.
Wine locator `e174-indexed-ordinary-wine-dh34xwul` follows the authenticated
recorded input for 6,000 frames from that checkpoint and observes zero source
bytes from either new archive. It therefore performs no full-state pair and
does not prove whether the command was skipped or its original flag check
failed. The Wine server drained; Linux and USA remain deferred. Other scenes
and callers remain open.

The original flag-`0x40` getter tests bit 0 at `02001f78`; that bit is clear at
both recorded endpoints of the negative window, though its value during the
command is unknown. A subsequent source check corrects the proposed separate
program-2 route: the dispatcher subtracts one from the request before indexing
the table at `0812557c`. Entry 11 selects `08125638`, which calls the loader,
so it requires request 12; entry 12 selects `081255b0` for request 13. The
program-2 instruction `0x17 0d` does not by itself select the loader path.
Disassembly evidence is retained in `e175-indexed-program2-source-lead-lt7b6kiv`;
its original PASS did not assert this table index and does not validate the
rejected hypothesis. A focused Sol consultation was requested as `gpt-6.1-sol`
at `xhigh`; its effective settings were not independently confirmed. Guarded
static audit `e175-request12-source-audit-eduynil3` authenticates the dispatch
and finds four `0x17 0c` instructions in programs 146, 148, 150 and 160, reached
by event keys 154, 156, 158 and 170 respectively. These are conditional source
associations, not evidence that any corresponding event or loader call runs in
ordinary play. Other request-12 producers and ordinary execution remain open.

Another original caller is associated with mode-table slot 2. Protected static
audit `e177-indexed-mode2-source-audit-0ivxkuzk` follows registration of
`08025115`, the selector table and its state-zero call through `080251e4` to
the same loader. It finds eight syntactic direct request-2 candidates. An
authenticated normal-input cold recording reaches a checkpoint at frame 9,000
with selector 2 and that slot populated. This is a bounded source and snapshot
association: it does not prove that the callback or either archive was read
during the recording. The recording is a possible future observation, not a
replacement for the completed negative window. Offline audit `acjbv2hh` also
verified that an independently retained cold-origin preselection is at frame
180 with neutral input; a time-shifted copy of the recording is retained for a
bounded continuation. It did not independently prove that the two recordings
have identical states at frame 180 at that point in the investigation.

Wine locator `e178-indexed-ordinary-cold-wine-abz4lk0q` continues that cold-origin
checkpoint with the shifted recorded keys. It first observes managed source
reads at relative frame 967 and then records 302 and 3,292 source bytes from
the two archives respectively. A rolling checkpoint allows an 11-frame
raw/managed comparison through this use with the managed ROM copies of both
archives and caller literals altered. Each frame matches complete state and
guest clock, central and wide framebuffers, OAM and audio; audio is nonempty.
Generated dispatch can fall back to interpretation. The Wine server drained
and protected inputs remained unchanged.

The original authenticated acquisition helper and input were subsequently
replayed from cold for 180 frames under Wine
(`e178-cold-prefix-identity-snmplz7o`). The raw snapshots differ: the original
helper drains its audio output each frame, while the retained checkpoint has
131,072 silent output samples. Their headers and checksums are valid; every
other payload byte matches. Loading both snapshots with the same retained
decoder and draining only the audio output yields identical complete serialized
states (`e178-prefix-audio-normalized-5i77hh79`). The accepted archive test
already drains that output immediately after loading, before guest execution.
Independent verification also confirms the recorded input is neutral before
frame 180 and the shifted events exactly match the original continuation
(`e178-prefix-join-proof-v89740hr`). The initial exact-byte assertion therefore
failed for the explained output-queue difference, not a demonstrated guest-state
divergence. This establishes the tested cold-origin join after the existing
audio drain; it does not prove the entire 9,000-frame capture, all scenes,
archive consumers or request-12 execution. Linux, USA and native Windows
remain unverified. Protected inputs passed and Wine drained; the mismatch unit
and its diagnostic state remain retained. A Sol consultation was requested as
GPT-6.1 Sol at xhigh; effective settings and the primary model/effort were not
independently confirmed.

EUR/Wine `indexed-full-route-wine-klmjn8sv` now compares the complete remaining
8,820 frames of that recorded input, from frame 180 through frame 9,000, with
the E170 dataset. The two archive spans and their caller literals are inverted
in the managed ROM during guest execution. Both archives are read: counters
record 302 and 3,292 source bytes respectively. At every frame raw and managed
states, guest clocks, central/wide images, OAM, dispatch and five nonempty
audio streams match. Source bytes are restored only to serialize the managed
state for comparison. Inputs were protected, Wine drained and successful
scratch was removed. Wine logged a D-Bus assertion, but the test completed.
The explicitly selected translation uses generated dispatch with interpreter
fallback. The result does not compare the endpoint to the original historical
capture or attribute individual reads to native PCs; other routes, consumers,
Linux, USA and native Windows remain open.

A retained EUR DS-20 checkpoint contains program 160 before its request-12
instruction. Its flag `0244` is clear, so the original conditional can route
through request 7 first; cursor proximity does not establish the subsequent
control flow. The checkpoint is a legacy `FFTA-SL1` state with no recorded
ancestry metadata, so it cannot independently prove a cheat-free history. A
previously recorded neutral continuation is available for a bounded diagnostic;
this is a source lead, not an ordinary-use witness.
