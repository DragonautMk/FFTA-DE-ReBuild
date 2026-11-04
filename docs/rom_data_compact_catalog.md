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

The production Windows CLI passed a 12-frame EUR startup on Wine using the e112
generation. All 100 runtime resource identities matched the manifest and the
protected inputs and generation remained unchanged. Catalog usage was zero, so
this establishes registration and startup only; ordinary use of the three new
roots/tables remains unverified. The e111 comparison is retained; no retirement
was attempted. Full catalog/consumer coverage, the other 26 census roots, Linux,
USA, native Windows and mod support remain unverified. No performance or space
measurements were performed. Evidence: `target/rom-data-e111-next-sources-20261103/cli-unit.json`;
Wine unit `rom-data-table-compact-cli-tycqx900`.
A later EUR source query identifies one additional conditional producer for root
`0x6ca5f8`. Its original loop initializes selector 0, calls the previously
verified constructor, increments an unsigned byte and repeats through selector
2. The Wine disassembly and opcode checks establish this loop, not all possible
producers or ordinary reachability. Its independent projection covers 174
unowned source bytes, three sequences with 13/12/12 frames and nine lists with
eight objects; list 8 is empty. The e113-d112 generation now extracts the bank
and serves those fields at runtime; the catalog has 38 banks, 106 selectors,
400 frames, 303 lists, 1,096 objects and 9,382 represented bytes. Its creation
and no-write reuse preserve the preceding 37 banks and 99 other resources.

A focused Wine check poisons duplicate ROM bytes, checks reconstruction and
mixed boundaries, preserves the empty list, rejects a coherent edit, and checks
the original reader prefixes for three counts, 37 frame pairs and nine list
pointers. Four original/native count comparisons, including one outside selector,
match complete state and clock. A 12-frame production CLI startup matches all
100 resource identities; catalog reads were zero, so ordinary use of this root
is still unproven. Source, build, generation, native and CLI evidence is under
`target/rom-data-e112-residual-20261104/`; Linux, USA and native Windows remain
pending. No performance or space measurements were performed.

A further conditional producer uses compact root `0x6c992c` and a 20-row
selector table at `0x3a86d0`. The original bounds check admits row indices
0–19; a halfword threshold gates each constructor call. The selector byte in
those rows takes values 0–7. The authenticated source projection covers a
200-byte bank with 40 frames and nine lists (list 1 is empty), plus six read
bytes per table row. The other 40 row bytes remain unowned. The new bank ends
exactly where an existing bank begins, which must be preserved when merging
source spans. E114-d113 now extracts the bank and table in the v3 catalog
and serves their known fields at runtime. Creation and no-write reuse preserve
38 earlier banks and 99 other resources; the complete catalog has 39 banks,
114 selectors, 440 frames, 312 lists, 1,104 objects and 9,702 represented bytes.

Wine validation checks the 320 new bytes, mixed boundaries, raw fallback for
40 unread row bytes, empty list 1, original reader prefixes and both threshold
paths. Nine native count comparisons match full state and clock, including an
outside selector. A 12-frame production startup matches 100 resource identities;
zero catalog reads mean ordinary use is not proved. Source, build, dataset,
native and CLI receipts are under `target/rom-data-e113-table-root-20261104/`.
The first source query incorrectly expected list 8 to be empty; the corrected
query checks list 1. Other producers, Linux, USA and native Windows remain
unverified. No performance or space measurements were performed.

Another conditional EUR producer uses root `0x6c95d4` and a 16-row,
12-byte table at `0x3a8610`. Its guarded table selector has values 0–2;
two original helper calls preserve the copied selector before the constructor.
Source analysis identifies 412 bank bytes, 160 read table bytes and 32 unread
row bytes. The bank has 33 frames, 33 lists and 33 objects. The table ends at
the prior timed-position table; owned spans must merge only at actual adjacency.
E115-d114 extracts these fields into the v4 catalog and serves them at runtime,
leaving the 32 unread bytes raw. Its 40 banks cover 117 selectors, 473 frames,
345 lists, 1,137 objects and 10,274 represented bytes. Wine create/reuse retains
39 earlier banks and 99 other resources. A focused Wine test checks all 572 new
bytes, raw fallback, boundary reads, original guarded table paths, 33 frame pairs,
33 list pointers and four native count cases with matching full state and clock.
A 12-frame production CLI startup matched all 100 resource identities; zero
catalog reads leave ordinary use unproved. Source, build, dataset and test
receipts are under `target/rom-data-e114-root-20261104/`. Other producers, Linux,
USA and native Windows remain unverified; no performance or space measurements.

A conditional EUR caller at `0x080e22a8` passes selector 0 and root
`0x6c4910` to the alternate `0x080d9990` constructor. Retained original
initializer and reader instructions authenticate its compact sequence path.
The source projection has eight frames, six lists and 19 objects, including
empty list 5, in 162 bytes; the two-byte gap remains raw. This is not a global
selector bound. E116-d115 now extracts and serves this projection. Wine creation/reuse
preserves 40 earlier banks and 99 other resources. A focused test checks 162
source bytes, raw-gap boundaries, the empty list, eight original frame pairs,
six list pointers and two original/native count cases with full state and clock.
A 12-frame Wine CLI startup matches all 100 identities but has zero catalog
reads, so ordinary use remains unproved. The complete catalog has 41 banks,
118 selectors, 481 frames, 351 lists, 1,156 objects and 10,436 represented bytes.
Evidence is under `target/rom-data-e115-root-20261104/`; other selectors and
producers, Linux, USA and native Windows remain unverified. No performance or
space measurements were performed.

A bounded follow-up query for the authenticated alternate constructor finds
ten previously unowned EUR roots with 12 explicit selector values. Their caller
windows load the literal roots and set selectors before direct calls; source
geometry is 1,818 bytes, 120 frames, 95 lists and 185 objects. The windows
have been reviewed as conditional call paths; other callers and selectors are
not bounded by this query. Two already-owned roots are excluded from this set.
Wine evidence: `target/rom-data-e116-alternate-census-20261104/` and unit
`rom-data-e116-alternate-geometry-1k042bio`. Other callers and selectors remain pending.

The E117-d116 integration extracts and serves these ten conditional banks.
Windows cross-build and EUR Wine creation/reuse pass; 99 other resource payloads
and the 41 prior banks remain unchanged. The focused Wine reader test compares
1,818 source bytes and boundaries, including 22 original/native count cases
with full state and clock. A 12-frame production Wine startup matches all 100
resource identities but records zero catalog reads; ordinary use of these roots
is not established. Receipts are under
`target/rom-data-e116-alternate-census-20261104/` and the managed E117 dataset
is retained in `rom-data-e117-alternate-current-5mvjt67m`. Other callers and
selectors, ordinary use, Linux, USA and native Windows remain unverified; no
performance measurements were made.

The retained E111 list has 29 candidate roots. Comparing its root addresses
with E117 `BankSpec` declarations found 17 represented and 12 without a
matching declared bank: `6babc4`, `6bac28`, `6bb574`, `6c0614`, `6c1e78`,
`6c316c`, `6ca580`, `6cac40`, `6cb278`, `6cca8c`, `6ccae4`, `6ccdec`.
This comparison does not establish full selector or consumer coverage for any
represented root and is not a global census.

A EUR Wine source window for candidate `6c0614` shows a bounded 44-row,
four-byte-stride table at `39e164`; byte 1 supplies the selector at the call
site and contains values 0–5 in those rows. The candidate at `6ca580` receives
its selector from a mutable structure; no bound is established. For `6c0614`,
a Wine source geometry check identifies 42 frames, 19 lists and 554 source bytes
with no conflict against the E117 manifest. Evidence:
`target/rom-data-e117-residual-census-20261104/geometry-unit.json` and
`target/validation/units/rom-data-e117-residual-source-aztqb7xo/evidence/`.
The original source window confirms that the bounded timer loop supplies those
fields to the compact constructor; the E117 manifest has no table or bank
overlap. E118-d117 extracts and serves this bank and all 44 table rows as v5
catalog fields. A Windows cross-build and EUR Wine create/reuse pass with 99
other resources and the previous 51 banks unchanged. Focused Wine checks cover
730 new bytes, boundaries, the original selector/coordinate prefix for all 44
rows and seven native count comparisons with full state and clock. A 12-frame
Wine CLI startup matches all 100 resource identities but records zero catalog
reads; ordinary use is not established. The E111 census now has 11 roots without
a declared bank; the other selectors and consumers remain unbounded.
Source evidence is in `target/rom-data-e117-residual-census-20261104/`;
E118 receipts are in `target/rom-data-e118-timer-20261104/`. Linux, USA and
native Windows remain unverified; no performance measurements were made.

A conditional EUR caller at `0x0810965a` derives selector 3 or 5 for root
`0x6c1e78`; its two helper paths preserve the selected register, including the
`0x15` jump-table case. Source geometry covers 56 frames, 13 lists and 392
bytes without E118 overlap. E119-d118 extracts and serves the bank. Windows
cross-build and EUR Wine create/reuse pass, preserving 99 other resource payloads
and all 52 prior banks. A focused Wine check covers all 392 new bytes, boundary
reads, both selector paths, original frame/list readers and three native count
comparisons with full state and clock. A 12-frame CLI startup matched all 100
resource identities but made zero catalog reads, leaving ordinary use unproved.
Other callers and selector values are not bounded. Source receipts are under
`target/rom-data-e118-residual-20261104/` and E119 checks under
`target/rom-data-e119-root-20261104/`. Linux, USA and native Windows remain
unverified; no performance measurements were made.

A conditional EUR caller at `0x080f01e8` uses a 12-row table at `0x39e0c4`
and root `0x6bac28`; the loop starts at zero and the table supplies selectors
0–8. Its represented bank covers 172 source bytes; the full table contributes
96 bytes with no E119 overlap. E120-d119 extracts and serves both as v6 fields.
Windows cross-build and EUR Wine create/reuse pass, retaining the 53 previous
banks and 99 other resources. Focused Wine checks cover 268 new bytes and
boundaries, the original field-reading instructions for all 12 rows, and ten
native count comparisons with full state and clock. A 12-frame Wine CLI startup
matched all 100 resource identities but recorded zero catalog reads; ordinary
use remains unproved. Other callers and selectors remain unbounded. Source
receipts are under `target/rom-data-e119-6bac28-20261104/` and execution checks
under `target/rom-data-e120-schedule-20261104/`. Linux, USA and native Windows
remain unverified; no performance measurements were made.

Two conditional EUR callers use root `0x6babc4` with 30-row and 11-row tables
at `0x39dc50` and `0x39f6cc`. Their bounded selectors are 0–5, and the bank
and table projections cover 428 source bytes without E120 overlap. E121-d120
extracts and serves both tables and the bank as v7 fields. Windows cross-build
and EUR Wine create/reuse pass, preserving 99 other resources and all 54 prior
banks. Focused Wine checks cover the 428 bytes and boundaries, original field
readers on all 41 table rows and seven native count comparisons with full state
and clock. A 12-frame Wine CLI startup matched all 100 resource identities but
recorded zero catalog reads; ordinary use remains unproved. Other callers and
selectors remain unbounded. Source receipts are under
`target/rom-data-e120-table-leads-20261104/` and execution checks under
`target/rom-data-e121-flight-20261104/`. Linux, USA and native Windows remain
unverified; no performance measurements were made.

A conditional EUR caller at `0x080ef024` visits ten type-dependent rows at
`0x39defc` and uses root `0x6bb574` for selectors 0–2. The bank projection
covers 52 bytes; 141 table bytes are read on the observed branches. E122-d121
extracts and serves the bank and sparse typed rows as v8 fields, leaving
branch-inactive bytes raw. Windows cross-build and EUR Wine create/reuse pass,
retaining 99 other resources and all 55 prior banks. Focused Wine checks cover
193 represented bytes and boundaries, the original field readers for all ten
rows and all three row types, raw fallback for inactive fields, and four native
count cases with full state and clock. A 12-frame Wine CLI startup matched all
100 resource identities but recorded zero catalog reads; ordinary use remains
unproved. Other callers and selector values remain unbounded. Source evidence
is under `target/rom-data-e121-6bb574-20261104/` and execution checks under
`target/rom-data-e122-effects-20261104/`. Linux, USA and native Windows remain
unverified; no performance measurements were made.

Two EUR callers at `0x0811a024` and `0x0811b75c` each traverse 24 rows
and calculate selectors 0–7 for root `0x6cac40`. A protected Wine source
analysis bounds this caller-specific projection to 132 bank bytes and 594
conditionally read table bytes, with no E122 resource overlap. E123-d122
extracts and serves those fields while leaving unread row bytes on the raw ROM
path. The Windows build and EUR Wine create/reuse pass, preserving 99 other
resources and all 56 previous banks. Focused Wine checks cover 726 represented
bytes and boundaries, the original field readers for all 48 rows, raw fallback
for unread fields, coherent rejection and nine native count comparisons with
full state and clock. A 12-frame Wine startup matched all 100 resource
identities but recorded zero catalog reads. This does not bound other
callers, establish ordinary gameplay use, or verify Linux, USA or native
Windows. No performance measurements were made. Source evidence:
`target/rom-data-e122-6cac40-20261104/`; execution checks:
`target/rom-data-e123-grid-20261104/`.

The EUR caller at `0x080e255c` initializes three records pointing to root
`0x6c316c` with indices 0–2; callback `0x080d88fc` forwards each index through
`0x08011434` to the object creator. Wine source geometry for these initial
indices identifies 188 represented bytes with no E123 overlap. E124-d123
extracts and serves the scoped bank. The Windows GNU build and EUR Wine
create/reuse pass, preserving 99 other resources and all 57 previous banks.
Focused Wine checks cover 188 bytes and boundaries, original callback header
and index reads, coherent rejection and four native count comparisons with
full state and clock. A 12-frame Wine startup matched all 100 runtime identities
but recorded zero catalog reads. The records are mutable and other consumers are not bounded; unwitnessed
accesses retain raw fallback. Ordinary gameplay use, Linux, USA and native
Windows remain unverified, and no performance measurements were made.
Evidence: `target/rom-data-e123-6c316c-20261104/`; execution checks:
`target/rom-data-e124-initial-20261104/`.

The first four script task descriptors select callback `0x0811eaa8`. In the
secondary constructor, IDs 363–366 are copied to the callback template; their
low bytes yield initial indices 0–3 into the lookup at `0x3ab98c`. The four
lookup bytes select 0–3 in root `0x6cb278`; Wine source analysis identifies
1088 bank bytes and no E124 overlap. E125-d124 code represents this initial
secondary-path projection. Windows GNU build and EUR Wine create/reuse pass,
preserving 99 other resources and all 58 previous banks. Focused Wine checks
cover 1092 represented bytes and boundaries, original lookup reads and raw
fallback, coherent rejection and five native count comparisons with full state
and clock. A 12-frame Wine startup matched all 100 resource identities but
recorded zero catalog reads. The primary constructor aliases these descriptors with different IDs and can
produce another index. Mutable state and other invocations remain unbounded;
raw fallback remains available. Ordinary gameplay, Linux, USA and native
Windows remain unverified; no performance measurements were made. Evidence:
`target/rom-data-e124-6cb278-20261104/`; execution checks:
`target/rom-data-e125-secondary-20261104/`.

A EUR parent task uses the documented clear helper on its context and then
runs a five-by-four loop. Its bounded initial path passes outer indices 0–4 to
root `0x6cca8c` and fields of 20 rows at `0x3ac944` to roots `0x6ccae4` and
`0x6ccdec`; row bytes 6–7 remain raw. Wine source geometry found no E125
resource overlap. E126-d125 code represents these three bank projections and
20 sparse rows. Windows GNU build and EUR Wine create/reuse pass, preserving
99 other resources and all 59 previous banks. Focused Wine checks cover 3870
source bytes and boundaries, the original field readers for all 20 rows, raw
fallback, coherent rejection and 11 sampled native count comparisons with full
state and clock. A 12-frame Wine startup matched all 100 runtime identities
but recorded zero catalog reads. This is conditional on the original context
clear and loop, and does not bound mutable or other
call paths. Ordinary gameplay, Linux, USA and native Windows remain unverified;
no performance measurements were made. Evidence:
`target/rom-data-e125-6cca-20261104/`; execution checks:
`target/rom-data-e126-scene-20261104/`.

For EUR root `0x6ca580`, the dispatcher initializes the selector from a
four-bit field and passes it through a type-13 callback. Wine source analysis
found selectors 0–6 structurally decodable as a 116-byte compact projection
without E126 overlap. Candidates 7–15 exceed this analysis' structural guards;
that does not make those guest accesses invalid, and unrepresented bytes retain
ROM fallback. E127-d126 extracts and serves the seven-selector projection.
Windows GNU build and EUR Wine create/reuse pass, preserving 99 other resources
and all 62 previous banks. Focused Wine checks cover the 16 nibble values, 116
represented bytes and boundaries, raw fallback, coherent rejection and four
sampled native count comparisons with full state and clock. A 12-frame Wine
startup matched all 100 resource identities but recorded zero catalog reads.
Other callers, mutable state, ordinary
gameplay, Linux, USA and native Windows remain unverified. No performance
measurements were made. Evidence: `target/rom-data-e126-6ca580-20261104/`;
execution checks: `target/rom-data-e127-nibble-20261104/`.

Source evidence: `target/rom-data-e111-next-sources-20261103/`,
`source-contract-result.json`, and Wine source unit
`rom-data-table-compact-source-9sx6eqy3`. Ordinary use, complete banks/global
reachability, the remaining census roots, Linux/USA/native Windows and the
ROM/mod phases remain open. No benchmark or space measurement is performed.

## European Soil Evidence ordinary route

The retained IN-08 unit 6 is a human with secondary Illusionist job 9 and the
learned flag for descriptor slot 87. Action 49 maps through task callback
`0x080f089c` and child `0x080f01e8` to compact root `0x6bac28`. Starting from
the retained Star Cross confirmation, ordinary inputs returned to the skill
menu, selected Soil Evidence and reached its confirmation. An E127 EUR Wine
pair over 480 frames passed full state/clock, framebuffer, OAM, dispatch and
five audio-stream comparisons; the native CLI observed positive aggregate
catalog reads. The aggregate counter alone cannot prove which bank was read. A separate
feature-gated Wine watch on the decoded machine observed a read of represented
address `0x086bac2a` at frame 8391, within root `0x6bac28`; the same 480-frame
raw/managed comparison passed. This is one ordinary invocation, not full
animation or global selector/consumer coverage. Other banks, Linux, USA and
native Windows remain open; no performance measurements were made. Evidence:
`target/rom-data-e127-loop-ordinary-20261104/` and
`target/rom-data-e127-loop-watch-20261104/`.

The same task49 parent later creates child `0x080f07ec`, whose timer loop uses
root `0x6c0614`. Retained EUR Wine source windows identify this connection.
A feature-gated Wine watch observed represented address `0x086c0616` at frame
8511; the native raw/managed comparison passed for all 600 frames with full
state/clock, images, OAM, dispatch and five audio streams. The selected test
executable was emitted and its 845-file source snapshot verified, although the
overall Cargo command failed while compiling the CLI because the filesystem
ran out of space; the prior CLI remained unchanged. Other callers, complete
animation, Linux, USA and native Windows remain open; no performance
measurements were made. Evidence: `target/rom-data-e127-loop-ordinary-20261104/`
and `target/rom-data-e127-loop-timer-watch-20261104/`.

The previously failed audit CLI build was retried against the retained, verified
845-file snapshot in an isolated Windows cross-build. The CLI compiled without
warnings, and a separate guarded verification checked Cargo's successful finish,
artifact hash and unchanged snapshot. The build unit itself remains marked FAIL
because its cleanup refused Cargo-created hard links; its diagnostics and output
are retained. This closes the missing compilation only: the existing Wine scene
results were not repeated, and the wider coverage limits above remain. Evidence:
`target/validation/units/rom-data-audit-cli-verify-61zo549_/evidence/result.json`
and `target/validation/units/rom-data-recover-audit-cli-fd8jxksj/`.

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

## European Freezeblink ordinary route

The retained IN-08 unit 6 also has learned human descriptor slot 83, mapped
through action 45 to task45 callback `0x080ee898`, child `0x080ee6f0` and
compact root `0x6babc4`. Ordinary menu input selected Freezeblink and reached
its confirmation. In Wine, a feature-gated watch observed represented address
`0x086babc6` at frame 8361; the 600-frame native raw/managed pair matched full
state/clock, images, OAM, dispatch and five audio streams. This uses the same
verified test executable emitted before the audit Cargo build failed on its CLI
with ENOSPC; the preceding CLI stayed unchanged. This single invocation does not
establish complete animation, selector or consumer coverage. Linux, USA and
native Windows remain unverified. No performance measurements were made.
Evidence: `target/rom-data-e127-loop-ordinary-20261104/rosters-45-unit.json`,
`target/rom-data-compact-catalog-ordinary-20261103/task45-select.png`,
`target/rom-data-compact-catalog-ordinary-20261103/task45-target.png` and
`target/rom-data-e127-loop-freeze-watch-20261104/watch-unit.json`.

## European Stardust ordinary route

The retained IN-08 unit 6 has learned human descriptor slot 85. Its action 47
maps through task47 callback `0x080ef250` and child `0x080ef024` to compact
root `0x6bb574`. Ordinary menu input selected Stardust and reached confirmation.
In Wine, a feature-gated watch observed represented address `0x086bb576` at
frame 8450; the 600-frame native raw/managed pair matched full state/clock,
images, OAM, dispatch and five audio streams. The verified test executable was
emitted before the audit Cargo build failed on its CLI with ENOSPC; the earlier
CLI was unchanged by that build. One invocation does not establish complete
animation, selector or consumer coverage. Linux, USA and native Windows remain
unverified. No performance measurements were made. Evidence:
`target/rom-data-e127-loop-ordinary-20261104/rosters-47-unit.json`,
`target/rom-data-compact-catalog-ordinary-20261103/task47-select.png`,
`target/rom-data-compact-catalog-ordinary-20261103/task47-target.png` and
`target/rom-data-e127-loop-stardust-watch-20261104/watch-unit.json`.
