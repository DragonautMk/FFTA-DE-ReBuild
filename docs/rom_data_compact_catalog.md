# Witnessed European compact OBJ catalog

This catalog extends direct runtime consumption of EUR compact OBJ animations
without duplicating the eight earlier managed compact banks. It is one sparse
resource for the new root/selector pairs supported by the retained reader census.
It does not establish the complete game catalog, indirect call graph or ordinary
use of every sequence.

## Source and scope

The existing e100 candidate census contains 71 roots and 118 possible Thumb
literal readers in the historical 6b3428..6df2dc search interval. Its atlas and
ownership labels are historical. The new query starts only at those retained
reader offsets, examines at most 0x120 bytes below code offset 150000, stops at
branches or unsupported instructions and tracks known register values through
supported straight-line instructions. Calls clobber volatile registers; root
values retained across the selected constructor calls use their authenticated
save/restore contract. This is not a new whole-ROM control-flow analysis.

The query finds 65 literal-root/constant-selector calls into original
constructors 080d97d0 and 080d98e4, covering 36 roots. Twenty-four calls belong
to seven earlier owners and are excluded; the eighth earlier compact owner also
stays excluded. The remaining 41 calls establish 29 new roots and 38 selectors.
Thirty-seven exact original windows were independently disassembled under Wine;
all root literals, BL targets and root/selector transfers were checked. Twelve
constructor prologue/epilogue pins authenticate the saved-root registers; there
are no other calls within these selected windows.

The old preliminary 6c30a4 projection omitted selector 1. Original 080ff670 is
MOVS r1,1 before 080ff674 calls 080d97d0. The new contract includes selectors
0/1/2 and 152 bytes. The old 104-byte suggestion was never accepted as a managed
contract and is not reused.

## Sparse format

For each attested pair, the original format uses a four-byte header: flags and
a little-endian relative sequence-directory offset. The list directory starts
at root+4. A sequence directory cell is a relative u16 offset to a u16 count
followed by exactly that many two-byte list/duration frames. Each referenced
list directory cell is a relative u16 offset to a count-prefixed six-byte OBJ
list. Signed coordinates, packed attributes and all duration bits are retained
using the existing shared typed representations.

Only witnessed sequence directory cells and their records are included. Only
lists referenced by those records, their directory cells and their actual
objects are included. Directory lengths are not inferred from first offsets,
neighbor distances or physically plausible entries. Empty lists retain their
count bytes. Gaps and unwitnessed selectors keep their existing guest reads;
no new guest bound or selector guard is introduced.

The projection contains 29 banks, 38 sequences, 207 frames, 160 list identities
and 888 objects. Its 6,650 bytes form 65 merged source spans. Each selected
header has flags 12; no projected byte overlaps another selected bank or any
of the 99 e109 owners. The exact bank/selector/frame/list metadata is maintained
in [the contract specification](../crates/gba-core/src/rom_data/compact_obj_catalog/contracts.rs).
The source projection SHA256 is
`463fa5a1f075ce6e7fd705dcb0ec3772552d63e53ed8293ce2fa0b90bd2647c7`.
These source extents are not a claim that all their bytes execute in gameplay.

## Integration and validation status

EUR e110-d109 adds
`graphics.conditional-compact-obj-catalog` / `compact-obj-catalog.json` after
the 99 existing resources. One generic typed model authenticates the exact EUR
ROM and reconstructs the sparse spans. The immutable runtime owner composes
its represented bytes with previous owners and raw bytes across mixed reads;
its counters distinguish requests from source bytes. Original instructions,
copies, timing and live task state remain guest behavior. This is base-resource
authentication; it does not enable mod overrides. Save format, C API and native
translation compatibility are unchanged.

Canonical Windows test and production CLI builds pass without warnings; all
709 source hashes remain stable. The first test-build wrapper ended with signal
143 before publishing a result. Host checks found no matching descendants and
the accepted e109 binary remained unchanged. Its logs were retained; the
same-source continuation completed. The cause of that interruption is unproved.

One retained current e110 generation passes production CLI create/reuse:
100 resources, 101 generation files and 103 managed files. All 99 earlier
resource records and payload bytes are identical, and reuse leaves file sets,
hashes and modification times unchanged. The new resource matches the 65
independently derived source bindings. There is no full dataset copy per test.

One focused Wine filter passes source-byte/JSON/shape checks, all represented
bytes with their duplicate ROM payload poisoned, mirrored mixed boundaries and
coherent-edit rejection formed in memory. The canonical generation remains
read-only. Original reader prefixes verify all 38 sequence counts, 207 frame
list/duration pairs and 160 list pointers. A retained native count-reader
object is reused because its original code and C API are unchanged. All 38
witnessed selectors and one out-of-projection selector per bank produce 67
native/interpreter complete-state/clock matches, serving 187 typed bytes.
Valid cases serve 152 bytes; the other 35 are actual owned intersections from
out-of-projection directory/count reads. Those cases are not assumed all-raw,
and do not introduce a guest cap. This is conditional reader coverage, not
complete constructor/scheduler or ordinary gameplay equivalence. In-memory
failed-owner creation is not a new disk failed-reload/lifecycle matrix.

A short production CLI startup reuses the existing EUR Roulette Do it seed,
advances guest 11960..11972 and recognizes all 100 exact resource identities.
Its new-catalog counters are zero. This proves startup registration/reuse,
not ordinary consumption or global absence. The separate ordinary case below
adds a full-frame pair; the previous 600-frame Roulette acceptance was not repeated.

The selected ordinary leads remain unavailable from the declared human bank.
Metadata28 actions 303/310/312 select tasks 298/305/307 and callbacks
081110a0/08111f5c/08113708; exact dataset/ROM comparisons pass. None has an active
descriptor in the human bank's 142 declared records. Original child-creation
windows show 08113806 passing callback 08112a1d, then 08112b5e passing
08112715; their literal words and the bank calls are separately witnessed.
The earlier nearest-descriptor/action-305 guess is rejected as producer proof.
No cast was attempted for these rejected leads. This is not a claim that no
other class, actor or ordinary path can use the catalog. A supported producer
and appropriate scene are required before another ordinary attempt.

## Finite-loop European source extension

Five additional original callers initialize a byte-sized induction register to
zero before the retained literal reader. Each passes that register as selector,
increments it with explicit byte truncation and repeats through an unsigned
CMP/BLS back edge. The earlier query began at the root literal with that register
unknown, so it could not authenticate these selectors.

The source-supported roots and selector domains are 6c56a8 (0–2), 6c4c50 (0–2),
6c5800 (0–10), 6c88a8 (0–9) and 6c894c (0–15). Fourteen independent original
Windows windows cover the five loops, seven bounded helper bodies and two
unchanged constructor preservation pins. The query authenticates all 19 BL
instructions, the direct argument copies and exact initialization/tails. No
selected body has another branch. Reviewed helpers preserve the induction
registers; the previously accepted constructor saves/restores r4–r7. Conditional
loop entry is not a proof of global or ordinary reachability.

Independent geometry for the 43 selectors gives 60 frames, 44 referenced lists,
81 objects and 974 bytes in seven union spans. No selected byte overlaps another
new bank or any current e110 owner. The projection SHA256 is
`82c918e0689a542496708401309022974b72fdf5aa1ffe41b2bf739542b32935`.
Directory length is not inferred from physical layout. Only the loop-selected
cells, sequences and referenced lists are represented; other reads retain their
original semantics.

The existing generic catalog metadata is extended to 34 banks, 81 selectors,
267 frames, 204 list identities, 969 objects and 7,624 bytes in 71 union spans.
The resource ID, file, schema and 100-resource count are retained; extractor and
decoder revisions become e111-d110. The Windows test build passes without
warnings with 710 unchanged source hashes. Two production compiles finish cleanly
but are rejected because unrelated debug sources change during compilation.
Their logs and source pins remain. A single disposable snapshot of 841 source/
asset files then builds the production CLI with all 710 code hashes unchanged;
the selected ROM-data files match the accepted test build. The canonical Cargo
target and executable are reused. The successful source snapshot is removed
automatically; two earlier launch guards stop before capture/compiler execution
when another compiler is active. No other writer or job is stopped.

Production create/reuse passes with 100 resources and 101 generation files.
The 99 other resource records/payloads and all 29 prior bank structures are
identical. Reuse changes no managed file hashes, times or file set. One new-only
Wine filter verifies all 974 represented bytes with the duplicate ROM payload
poisoned, mirrored mixed boundaries and coherent edits rejected in memory.
Original reader prefixes verify 43 counts, 60 list/duration pairs and 44 list
pointers. The retained native count object serves 43 witnessed selectors plus
five out-of-projection cases: all 48 complete-state/clock comparisons pass,
serving 180 typed bytes (172 valid plus eight actual owned intersections).
An outside selector is not assumed all-raw, and no guest bound is added.
Old native reader cases are not repeated; no full dataset test copy is created.

The changed production CLI loads all 100 exact identities and advances the
existing Star Cross Do it seed for 12 frames, 7857..7869. Aggregate catalog
counters are zero in this short window. This proves changed-generation startup,
not new-five-root ordinary use or absence. The earlier 480-frame Star Cross
acceptance remains separate and is not repeated. Complete constructor/scheduler
behavior, ordinary use of these five banks and full bank domains remain open.

The successful native/CLI scratch is removed automatically. After acceptance,
the e110 comparison's 103 managed files are retired under old/new unit and Wine
prefix leases, retaining its manifest/logs and checking ten real-input guards.
The preservation map guards the successor, original ROM/seed/input/native helper,
TinyCC, old evidence and source preimages. Unrelated compiler outputs are outside
the cleanup operation; code-only compilers are left intact, while runtime or
selected-unit readers prevent retirement. One e111 generation remains under
`target/validation/units/rom-data-loop-compact-current-o_yv6xep/work/resources/`.
Earlier ordinary proofs and retired payloads do not reopen accepted checks.

Source evidence: `target/rom-data-compact-residual-sources-20261103/`,
`source-contract-result.json`, and Wine unit
`rom-data-compact-residual-source-6dt5o4n2`. Its successful scratch is removed,
with the loop specifications, metadata and original windows retained. The other
29 residual census roots and the global/ordinary/Linux/USA/native Windows/mod
criteria remain open. No performance or space measurement is performed.

## European selector tables and compact banks

Three further original producers select compact animation rows through ROM
parameter tables. Source branches and explicit bounds establish 14 rows at
3a80c8 (stride 8, selector byte 0), 10 rows at 39f7a4 (stride 8, selector byte 1)
and 15 rows at 3a6b10 (stride 4, selector byte 1). Their unique selector sets
are 0–13 for 6c58b8, 1–5 for 6c6e6c, and 0–2 for 6cbaf8. Conditional skip
paths and the third producer's variable starting index remain guest behavior.
The last entry guard rejects indices above 14 before table access; the typed
owner does not impose an additional guest bound.

Four new original Windows windows authenticate the three producer flows and
one leaf helper, with seven BLs and their exact branch destinations. Four
unchanged constructor/helper preservation windows reuse the accepted earlier
pins. The three bank projections add 22 selectors, 96 frames, 90 list identities,
119 objects and 1,366 bytes in nine spans, without existing-owner overlap.

The three selector tables were also unowned. Their proven fields add 218 bytes:
byte selectors/origins, signed halfword destinations, comparison bytes and
signed positions. The first table's byte 3 and the second table's bytes 6/7
are not read by the selected producers. These 34 bytes remain unowned and
read from the original ROM. No meaning or padding value is invented for them.
The third producer compares its byte key with a RAM halfword; it retains that
width and uses constructor Y=8 from code.

The catalog's v2 JSON adds typed `selector_tables` alongside `banks`. Historical
v1 JSON can be decoded without that field for bank identity comparisons, while
current validation requires all three bound tables. The resource format becomes
`compact-obj-catalog-v2-json`, extractor/decoder e112-d111; the resource ID and
100-resource count stay unchanged. The complete current projection has 37 banks,
103 selectors and 9,208 source bytes in 106 spans. Save format, original guest
instructions and native C API are unchanged.

One guarded source capture at a stable build path compiles the Windows CLI and
test harness in a single Cargo invocation. Both pass without warnings with
713 stable code hashes. The 844 source/assets files are removed after success;
the stable package path and canonical Cargo cache avoid per-version build copies.
A single e112 generation passes create/reuse with all 99 other payloads and
34 previous bank structures identical. The three typed tables match an
independent field decoder and do not claim the 34 unknown bytes.

One new-only Wine filter passes all 1,584 represented new bytes with duplicate
ROM bytes poisoned, mixed mirrored boundaries, source/JSON/shape validation and
coherent-edit rejection in memory. The 34 unknown bytes remain raw and do not
increment typed-read counters. The 99 other resources and all 34 prior banks
remain unchanged; their accepted cases are not repeated.

Original reader prefixes verify the 22 new sequence counts, 96 frame list/duration
pairs and 90 list pointers. The retained native count object covers the 22 new
selectors plus three outside cases, serving 90 typed bytes across 25 complete
state/clock matches. Original field prefixes cover 71 table and skip cases with
248 typed bytes. These include the RAM-halfword comparison, occupied-slot and
key-mismatch paths, and index-15 guard. All 96 native/interpreter comparisons
pass. They are conditional prefix checks, not an ordinary gameplay proof.

The first native wrapper selected zero tests through an incorrect Rust module
path. Its diagnostics remain; the corrected filter passes without a rebuild.
Successful native scratch is removed automatically; no dataset copy is created.
Evidence is `target/rom-data-e111-next-sources-20261103/native-unit.json` and
Wine unit `rom-data-table-compact-native-hw7pz62y`.

The goal is paused before the short production startup check. The accepted e111
generation remains the named comparison for the provisional e112 successor;
retirement follows its remaining acceptance check. Ordinary use of the three
new roots/tables, full catalog/consumer coverage, the other 26 census roots,
Linux, USA, native Windows and mod support remain unverified. No performance
or space measurements are performed.
Source evidence: `target/rom-data-e111-next-sources-20261103/`,
`source-contract-result.json`, and Wine source unit
`rom-data-table-compact-source-9sx6eqy3`. Ordinary use, complete banks/global
reachability, the remaining 26 census roots, Linux/USA/native Windows and the
ROM/mod phases remain open. No benchmark or space measurement is performed.

## Ordinary European Star Cross use

Original metadata28 action 46 selects primary task 46, whose descriptor at
399918 names callback 080eece4. At 080eedba the callback loads child 080eead9
from 080eede4; 080eedc2 calls the original task creator. Child 080eead8
initializes four compact objects through 080d97d0 at 080eeb44, using root
6bacd8 and selector 2. Its later selector 3 path is conditional. The callback
parameter can select an alternate path. The earlier action-45 adjacency lead
is rejected: its callback returns before this separate child function begins.

Star Cross is active human bank 1 descriptor 84. Human Illusionist is physical
metadata52 row 9, with descriptors 81–91; Nu Mou Illusionist is row 23, with
slot 37 in its bank. Raw byte 2d is a command/category identifier, not the
physical job selector. Original secondary producer 080d0900 passes roster
byte +8 directly to 080cbec8; there is no added row. Using command ID 8 as the
human job would incorrectly identify the existing Black Mage as Illusionist.

Seven original Windows windows and exact ROM/dataset identities support this
producer. Read-only Wine queries of six existing EUR checkpoints identify
Douglas, human unit 6 in IN-08, with secondary job 9 and learned descriptor84
byte 9e. The source-derived postcard-prefix reader agrees with the retained
Machine loader on all 24 roster identity/job fields and the earlier flag.
These observations execute no guest instructions. Deployment order differs
from raw roster order: six Right presses initially select raw unit 17, whose
missing secondary command is not evidence against unit 6. The corrected
selection observes both live displayed-unit pointers as 020006b0 before
normal placement, matching unit 6 and name pointer 087261a9.

Normal input continues from an existing accepted IN-08 travel endpoint to an
Elut Sands clan encounter, selects Engage, deploys Douglas and opens
Action → Phantasm Skill → Star Cross → Do it. The game displays a 64 MP cost
with 269 MP available. No job, learned flag, task, RNG, guest register or ROM
byte is forced. The existing travel proof and earlier Roulette pair are not
repeated. The normal-input route is consolidated from its recorded edges and
hashes without replaying it to reconstruct context.

One reduced Wine case advances frames 7857..8337. The production native CLI
serves 15,830 reads and 19,330 source bytes from the new catalog. The existing
native raw/managed filter passes complete state and clock, central/wide pixels,
OAM, dispatch and all five exact audio streams at every one of the 480 frames.
All streams contain actual samples. Canonical e110 CLI/test binaries, the
existing native translation, shared Wine prefix and current dataset are reused;
there is no rebuild, re-extraction or dataset copy. The exact source/input hashes
and managed tree remain unchanged. A retained generic A-eight-frame input is
reused under its legacy filename.

The catalog counter aggregates its represented banks; it is not per-bank or
per-selector telemetry. This proves ordinary consumption of the catalog during
one source-backed Star Cross confirmation. It does not establish every bank,
selector, constructor branch, scheduler path or the complete animation. The
endpoint still shows the spell animation. Linux, USA, native Windows, wider
ordinary coverage and the remaining ROM/mod phases are open; no benchmark.

One diagnostic precheck incorrectly required the list's first/top index to be
the selected entry. It fails before guest execution and is retained. The original
confirmation reader combines top index 0 with visible cursor row 1; the corrected
Engage check passes with count 2 and row offset 16. The failed sandbox Wine socket
launch is also retained; its exact owned leases were recovered only after host
checks. These are harness failures, without a demonstrated game regression.
Successful source/pair scratch is removed automatically. Superseded successful
navigation checkpoints are retired with guards for the current Do it seed,
original saves, canonical artifacts and dataset. Keep the current seed, source
preimages, small evidence and failed diagnostics; retired payloads do not reopen
accepted checks.

Ordinary evidence: `target/rom-data-compact-catalog-ordinary-20261103/`,
`ordinary-unit.json`, `normal-route-result.json` and
`navigation-retirement-final.json`; Wine pair unit
`rom-data-catalog-star-cross-ordinary-pair-l8_s2yrr`.

Source evidence: `target/rom-data-compact-catalog-20261103/` and Wine units
`rom-data-compact-catalog-query-410il0gd`,
`rom-data-compact-catalog-windows-xuq32lwl`,
`rom-data-compact-catalog-current-rdbcje1e`,
`rom-data-compact-catalog-native-9zzz4djk` and
`rom-data-compact-catalog-cli-a8rqf6ha`. Successful source/native/CLI scratch is
removed automatically; metadata, original disassembly and logs remain. The
e110 dataset was retained for the later e111 comparison, then retired after
that successor passed. The e111 current locator is recorded above. The e109
comparison payload was retired
only after exact successor acceptance, real-input guards, old/new unit and
Wine-prefix leases, and host activity checks. Its manifest/logs/results and
the native helper remain; original ROM and private saves/inputs are unchanged.
Retired payloads do not reopen accepted criteria.

Linux, USA, native Windows, additional selectors/roots, global extent, broader
ordinary use and remaining ROM/mod phases are open. No benchmark was run. Game data,
personal saves, recordings and generated native game code stay out of commits
and packages.
