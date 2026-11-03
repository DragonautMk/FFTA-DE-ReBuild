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
The separate context at `3710d4` remains unowned.

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
