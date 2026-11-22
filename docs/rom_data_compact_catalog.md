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

The original complete catalog regression now passes against e127-d126 under
EUR Wine. It verifies all 20,257 represented bytes and 330 source spans, 63
banks, 205 selectors, boundary reads and coherent edits rejected in memory.
The original interpreter readers and 268 native count cases agree in complete
state and clock, serving 925 typed bytes; the first 99 e126 resource records
match. The corrected report agrees with the tested manifest and catalog. This
does not establish ordinary use of every bank, disk-reload failure behavior,
native Windows, Linux, USA or performance. [Recovery record](changes/2026-11-04-verify-e127-original-compact-catalog.json).

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

A focused e127 EUR/Wine disk-attachment test now covers a missing catalog file
and a structurally valid catalog edit whose file hash is updated in the copied
manifest. Both replacement attempts fail while retaining the exact active owner
and machine state. The existing owner still serves catalog bytes, and a fresh
machine can attach and read the unchanged committed generation. This does not
test an interrupted generation write or complete lifecycle/platform coverage.
[Recovery record](changes/2026-11-04-verify-catalog-disk-replacement-failures.json).

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

The five-loop filter was rerun on e127-d126 under EUR Wine after correcting
its stale 103-object assertion to the 81 objects in both the current catalog
and the original accepted projection. It verifies all 974 selected source
bytes, 43 selectors and 48 native full state/clock cases, with 184 typed bytes
under current ownership. The 29-bank e110 catalog was reconstructed only in
scratch and matched its retained manifest hash; 99 prior resource records
matched and input preservation passed. The earlier e111 result remains
unchanged. [Recovery record](changes/2026-11-04-verify-e127-five-loop-catalog.json).

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

A later source-pinned lead for root `0x6c4c50` corrects an adjacency
assumption: primary task 322/action 327 has a separate callback and does not
establish this loop. Descriptor bank 22 slot 6 instead maps to action 328 and
task 323, callback `0x0811c23c`. Under its conditional state `0x0102` and
context gates, that callback can create child `0x0811c0a0`; the child's state
`0x0100` enters the three-selector loop at `0x0811c0fe`. Source-derived action
name glyphs visually read **Alpha**. A guarded read-only scan found no species
22 unit among the 24 clan slots in six retained EUR checkpoints. That limited
negative does not exclude enemies, other scenes, indirect dispatch or other
callers. No ordinary Alpha execution or exact root read is established.
Evidence: `target/validation/units/e128-6c4c50-caller-source-louifozx/`,
`e128-6c4c50-caller-xref-y29q60ah/`,
`e128-6c4c50-corrected-producer-0ccxi4aj/` and
`e128-6c4c50-action-label-uummv8dq/`. No guest instructions or performance
measurements were used in these source/checkpoint checks.

A guarded importer/data join then followed the original unit-property path for
zero-routing metadata52 rows. Among the 2,850 header-linked templates, 20
physical records in 12 headers resolve through that specific path to bank 22;
the corresponding metadata52 rows are 88 or 108. The candidate headers carry
map arguments 38, 69 or 162. This does not establish mission identity, an
available Alpha action, a live enemy, other routing branches or a usable scene.
The first query had two transcription errors in its opcode pins and failed
before guest execution; the corrected query passes. Evidence:
`target/validation/units/e128-6c4c50-unit-property-n2ooa0l5/`,
`e128-6c4c50-property-case-l24jm8aw/`,
`e128-6c4c50-encounter-candidates-_ubh8_yr/` and failed diagnostic
`e128-6c4c50-encounter-candidates-gnyhzazn/`. A separate protected read-only
scan of ten retained EUR checkpoints found none with a matching `U16(02002170)`
header index. The field has a documented original default-map reader, but its
saved value alone does not establish the active encounter or exclude other
paths. A source-authenticated join of the 256 bounded event rows links candidate
headers 440, 60 and 59 to direct-primary rows 159, 216 and 217 respectively;
mission record 379 references row 159. Its event key 124 and conditional script
program 120 references are not evidence of a playable mission or Alpha use.
None of the ten snapshots had selected mission index 379. A further six
distinct cataloged EUR checkpoints likewise contained neither a candidate
header index nor selected mission 379; duplicate payloads were skipped. These
saved values are bounded leads, not absence or availability claims. Evidence:
`target/validation/units/e128-6c4c50-live-header-scan-cbg06mc9/`,
`e128-6c4c50-live-mission-scan-4uzvf8x7/`,
`e128-6c4c50-mission-event-join-vam1bdgo/` and
`e128-6c4c50-catalog-checkpoints-auseo9z5/`.

The retained IN-08 world snapshot has a source-backed placed region 29 at
world `(216,72)`; original name glyphs visually read Ambervale. A bounded
normal-input route reached the visible Ambervale label and moved the clan to
that node. A further A input, observed at 8 and 120 frames, left the captured
central image on the map; no finale or Alpha action was established. This does
not rule out a prompt in another layer or another route. No guest fields, RNG
or original saves were edited. Evidence:
`target/validation/units/e128-alpha-in08-world-location-bcnaexe8/`,
`e128-alpha-ambervale-hitbox-1ctdmhrx/`, `e128-alpha-ambervale-travel-cktf_hxy/`,
`e128-alpha-ambervale-enter-2whyqug9/` and
`e128-alpha-ambervale-prompt-immediate-5gyadesd/`.

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

The caller also derives selectors 4 and 6 from the other flag branch. EUR
e128-d127 extends this bank to selectors 3/4/5/6 and 524 represented bytes,
preserving the previous lists and sequences. Source geometry and conflict
checks, Windows builds, Wine create/reuse, the selected native-reader test and
a 12-frame production startup pass. The full e128 catalog and affected recovery
regressions also pass. This conditional source
coverage does not establish ordinary use or all callers. [Integration record](changes/2026-11-04-integrate-default-selectors-6c1e78.json).

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

## European 6c1e78 ordinary-route candidates

A guarded EUR source join links bank 5 descriptors 55–62 to actions 243–250,
primary task IDs 239–246 and the shared callback `0x08109440`. The callback
branches to its `0x6c1e78` constructor path only under state `0x0103` and
further conditions; the source join alone does not demonstrate execution. The
original panel source indexes descriptor-bank pointers by unit byte +6, while
the secondary producer passes unit byte +8 directly as the physical metadata52
row. Original ROM bytes and the e128 dataset agree that rows 41, 42 and 43
contain respectively `[55, 65]`, `[66, 77]` and `[78, 87]` at offsets
`+0x2e..+0x2f`. These are source fields, not proof of menu availability.

Read-only inspection of six authenticated EUR checkpoints found IN-08 unit 13
with species 5, primary job 41 and learned bits for slots 55–62; IN-19 unit 4
has species 5, secondary job 41 and learned bits for 55–59 and 61–62. These
are source candidates; their learned bits alone do not establish execution.

The documented IN-08 input prefix was replayed to its retired deployment
selector solely to branch into this new scene, with its recorded state hash
reproduced. Normal input selected the observed slot-13 pointer, deployed
Skimble and reached the first turn. The visible Action → Pandora → Red Spring
route reaches a `Do it` confirmation at frame 7657. This establishes menu
availability for that scene. A 600-frame native raw/managed EUR Wine pair on
e128 matches complete state and clock, central and wide images, OAM, dispatch
and five audio streams at every frame. A read audit observed represented root
byte `0x086c1e7a` at frame 7876; aggregate catalog source bytes were 12,983.
A second, snapshot-only audit watches the selector-offset span and observes
`0x086c218a` at frame 8031: `(0x6c218a - 0x6c2180) / 2 = 5`, so the observed
selector is 5. This second 600-frame pair also passes all comparisons. Its
initial 300-frame window did not reach the selector read; failed diagnostics
are retained. Evidence: `target/validation/units/`
`e128-6c1e78-redspring-watch-pair-yj07anhh/`,
`e128-6c1e78-selector-watch-600-2emk682f/` and
`e128-6c1e78-selector-watch-pair-03yu4i2w/`. Complete animation and all
consumers remain unverified.
A distinct ordinary input from the retained Pandora menu selects Blue Screw
and reaches its `Do it` confirmation at guest frame 7673. A source-matched
snapshot-only audit of selector-offset addresses observed the represented
halfword at `0x086c218c` on frame 8048, identifying selector 6. The e128 EUR
Wine native raw/managed comparison passed all 600 frames, with complete state,
guest clock, central/wide images, OAM, dispatch and five audio streams equal
at each frame. Catalog source-byte counters total 12,953 across the scene and
are aggregate rather than per-bank counts. Inputs were guarded and successful
runtime scratch removed. Evidence:
`target/validation/units/e128-pandora-blue-blue-screw-target-lvshesyo/` and
`e128-blue-screw-selector-pair-2v9x4fr7/`. E255 later inventories the same
recorded continuation against E230; 24 sampled bytes lay outside declared
sources. E259 subsequently serves them. See the [bounded inventory](rom_data_runtime.md#e255-blue-screw-rom-read-inventory).
Selectors 5 and 6 are observed in separate scenes; selectors 3/4, all callers and complete animations remain open.

The retained branch is `target/validation/units/`
`e128-6c1e78-redspring-slot13-rwsxgiz7/`; the prefix/selection evidence is in
`e128-6c1e78-deployment-branch-2_ca0kz8/`. The first launch failed before
guest execution due to a relative checkpoint path; the corrected run passed.

The first source/roster script failed on an off-by-one task ordinal; the corrected
run passed without guest execution. Evidence: `target/validation/units/`
`e128-6c1e78-source-rosters-blgppao9/`, `e128-6c1e78-job-mapping-hxmci6q6/`
and failed diagnostic `e128-6c1e78-source-rosters-rqgukn11/`. Linux, USA and
native Windows remain open; no performance measurements were made.

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
and `target/rom-data-e127-loop-timer-watch-20261104/`. E260 later inventories
the same continuation against E231; see the
[bounded inventory](rom_data_runtime.md#e260-soil-evidence-rom-read-inventory).

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

## Conditional primary-task alias for 6cb278

The existing E125 projection covers secondary task IDs 363–366 and lookup
indices 0–3. The original primary constructor instead maps ID 362 to the
first of the same task descriptors and stores that ID in its message. If its
callback `0811eaa8` receives that message in state 0, the original byte
subtraction stores index 255. If invoked in state `80h`, subject to the original
helper result, the callback reads the selector byte at `3aba8b` and two signed
position halfwords at `3abd78..3abd7c`. The selected compact sequence already
has an owner. Source audit `e172-compact-primary-source-0k922apv` authenticates
the constructor, callback, literals and descriptor, and verifies these five
bytes have no E171 owner. This is conditional source flow: no ordinary call to
primary ID 362 or complete callback execution is established.

EUR E172-d171 publishes `graphics.compact-primary-alias` with those two exact
source spans. Its declared dependencies are the existing task descriptors and
compact catalog. The runtime reconstructs the selector and signed position
fields from the authenticated resource, composing neighboring bytes with
prior owners or raw ROM. It does not change guest IDs, arithmetic, mutable
task state or the existing compact bank.

Wine `e172-compact-primary-wine-_5qb5ej3` verifies create/reuse, preservation
of all 114 previous resource records and files, mixed-width/mirror boundaries
with backing bytes inverted, dependency and overlap checks, coherent-edit
rejection and transactional failed attachment. Five separately initialized
original prefixes compare interpreter and compiled execution against the raw
reference (ten complete state/clock comparisons); these include the primary
ID store, descriptor load, callback slot calculation and the two source
readers. The existing compact sequence is also read through its prior owner
with its backing bytes inverted. Production CLI Wine
`e172-compact-primary-cli-wine-lhle9dmz` creates, reuses and verifies the
115-resource generation; all 116 files match the tested dataset and reuse and
verification leave it unchanged. Both Wine units drained. The prefixes do not
prove ordinary reachability, a full callback ABI or a global selector domain.
Linux, USA and native Windows remain pending. No performance measurement ran.

## E172 Red Spring ROM-read inventory

The retained normal-input Red Spring confirmation was replayed on EUR/Wine
from guest frame 7657 through 8257 against E172-d171. The original-ROM and
managed machines matched complete state, guest clock, central/wide video, OAM,
dispatch and five nonempty audio streams at each of the 600 frames. The
existing compact-root watch observed `6c1e78` at frame 7876; its owner served
12,983 aggregate source bytes over the window. This is one ordinary action,
not complete animation, all-native execution or all-consumer coverage.

A test-only, feature-gated address observer collected reads from the managed
arm in the same comparison. The resulting 317,927 unique reader/address/width
requests cover 184,004 distinct image bytes; 9,423 bytes have no E172 source
owner, of which 6,075 lie at or above `370000`. Among the uncovered addresses
are ranges within the vicinity of compact root `6c1e78`, including
`6c1eb0..6c1f0a`, `6c1f0c..6c1fae`, `6c209e..6c2186` and
`6c218e..6c220a`. Bus reads cover all four ranges; selected host sprite peeks
also overlap the first three. At E172 these were observed bytes rather than
proved asset boundaries or per-instruction attribution; the subsequent E173
bank extension below authenticates the matching compact-bank geometry. Other uncovered ranges
include `b0e2b8..b0ecca` and `b151d8..b1550c`. Their readers, formats and
relationships to assets require separate source validation.

Wine unit `e172-redspring-audit-wine-c9uprkbl` completed the child test and
drained its server, but its unit receipt is FAIL because the 40 MiB address
report exceeded the runner's 16 MiB evidence-copy limit. Recovery unit
`e172-redspring-audit-recovery-05iu77v6` independently rechecked all 26
original input guards, the test log and report against the authenticated E172
manifest, retained the complete report and produced a compact uncovered-read
index. It did not replay the game. Large reports should be retained with
`--keep-path` rather than requested as small `--evidence` files. The observer
misses unselected host paths and folded constants, and unowned bytes can be
code. Linux, USA and native Windows remain pending; no performance measurement
was performed.

### E203 cross-reference with earlier code-support observations

After E201, 3,276 bytes in the E172 address sample still have no extracted
owner. Static reconciliation `e203-redspring-code-support-usda5fn4` matches
420 of those bytes to 105 four-byte words previously attributed to original
instruction sites in the separate EUR/Wine lower-residue investigation:
93 PC-relative literals and 12 entries in the `0a1150` control table. The
reconciliation checks the original source and instruction hashes, prior site
records and the current E201 manifest; it does not replay the game. Its
temporary source script is no longer retained, but the PASS receipt, result and
protected evidence remain available locally.

At E201 these 420 bytes remained unowned ROM reads. E211 owns the `0a1150`
control table and its `0a114c` root, and E212 owns all 105 words in this prior-site
set (`e221-redspring-prior-sites-so67hagh`). The remaining E172 ownership has
not been fully reclassified against E212. The earlier instruction attribution
comes from another scene: the Red Spring capture records addresses, not the
executing PCs, and does not exclude other uses of those words. At E201 the other
2,856 sampled bytes had no match in this particular prior-site set; neither
figure establishes full-ROM coverage.

### E231 Red Spring E221 source reconciliation

Static unit `e231-redspring-e221-reconcile-wt2lyljd` independently joins the
retained 9,423-byte E172 sample to the E201 and E221 declared source ranges.
It confirms the E201 residual of 3,276 bytes and verifies that all 420 E203
prior-site bytes have E221 declarations. Of the E201 residual, E221 declares
2,324 bytes: 1,948 literal, 348 switch-table, 20 copied-halfword and eight OBJ
dimension bytes. The other 952 bytes form 166 ranges between `003698` and
`14d854`, all sampled by the bus reader; none was sampled by the selected
sprite-peek reader. It verifies the ROM and original report hashes and each of
57 E201 and 563 E221 source spans intersecting the original sample. No game
was replayed, and the inputs were protected by the validation unit.

The result is an address and manifest comparison, not a proof of executing
PCs, runtime ownership dispatch, complete source extents, other callers or
full-ROM coverage. The 952 bytes still need source attribution and consumer
validation; other scenes, Linux, USA and native Windows remain open. No
performance measurement was made.

### E232 Red Spring residual reader scopes

Wine unit `e232-redspring-residual-sites-av1hj_7p` follows the retained E172
input and state for 600 frames with the E221 dataset, targeting the 238 aligned
words (952 bytes) outside its declared source ranges. All targeted bytes were
observed. Its distinct captured site records comprise 216 instruction-scoped
reads and 32 outside a scope. For 196 words (784 bytes), an original Thumb
PC-relative LDR opcode and its effective address match the observed four-byte
read, with conflicts against existing literal-reader instructions excluded.
The remaining 42 words do not have that eligible match in this capture. The
site plan and capture are retained with the unit; Wine drained cleanly with
the local Bluetooth overlay.

An instruction scope can contain nested HLE/DMA accesses: the match is a
source-selection witness, not proof of every direct caller. This observation
alone does not extract or consume these words, prove complete source extents,
show full scene equivalence, or classify the other 42 words. Other scenes,
Linux, USA and native Windows remain open. No performance comparison was made.
E233 subsequently publishes the 196 source-selected words and verifies their
readers, transfers, altered-source scene and CLI under Wine, leaving 42 sampled
words outside declared source ranges. [Evidence](rom_data_runtime.md#e233-red-spring-observed-thumb-literals).

## E173 Red Spring compact bank and host sprite lists

The authenticated EUR bank at `6c1e78` has seven represented selectors and
34 referenced lists. Original relative offsets and record lengths cover the
continuous interval `6c1e78..6c22fe`: 1,158 bytes, including 634 bytes that
were outside the E172 catalog. The added selectors 0, 1 and 2 have 8, 39 and
12 frames; their referenced lists account for 21 newly represented lists.
The earlier 600-frame inventory observed all 634 new bytes through the guest
bus and all 462 new list-body bytes through host sprite peeks. These checks
establish the source geometry and observed use, not individual selector timing
or a bound on arbitrary guest arithmetic. Source evidence:
`e173-compact-bank-source-j2cu37bc`.

EUR E173-d172 publishes the extended existing compact catalog. Its guest bus
serves the added fields, and host sprite-list lookups now consult this catalog
after the existing list owners. The focused Wine check `e173-redspring-wine-s73gcfva`
preserves the previous projections of 63 banks and all 114 other resources,
checks the 634 new bytes with altered backing ROM, mirrored and mixed-boundary
reads, three original interpreter/compiled count prefixes, and 33 nonempty
host lists with the whole bank altered. The first attempt
`e173-redspring-wine-k2ma7lbz` failed because the general game object did not
contain the isolated compiled reader; the corrected test reused the previously
authenticated compact-reader object. Both Wine servers drained.

The normal-input Red Spring action `e173-redspring-scene-wine-czzprb8z`
matched original and managed complete state, guest clock, central/wide video,
OAM, dispatch and five nonempty audio streams for all 600 frames, while all
634 new source bytes were altered during each managed frame. The selected
address audit recorded 610 unique guest-bus requests and 42 typed sprite-list
requests overlapping those bytes, with no raw sprite-peek request in that
set. Production CLI create/reuse/verify passed on Wine
(`e173-redspring-cli-wine-u2r2goyn`), and its 116 files match the tested
generation. One action and the selected prefixes do not establish all scenes,
readers, native execution paths or total ROM independence. Of the earlier
E172 inventory, 8,789 unowned observed bytes lie outside this bank increment;
[E174](rom_data_graphics.md#e174-observed-red-spring-lz77-block) subsequently
integrates 2,578 of them. Linux, USA and native Windows are pending.
No performance measurement was performed.

## E186 Red Spring compact-bank records

EUR E186-d185 extends `graphics.conditional-compact-obj-catalog` with three
24-byte records at `3a4ca0..3a4ce8`. Each points to the E173 compact bank at
`6c1e78`; the selected bank sequences are 2, 1 and 0. The remaining halfwords
retain their exact source values without assigning unproved meanings. Source
`e186-compact-records-source-ae4wgq4a` authenticates all 72 bytes, including
60 bytes observed unowned in the selected E172 sample. The catalog validates
each bank/selector link while leaving other record indices to the guest.

Wine `e186-records-trace-wine-2ibf_qi4` observes the original tail reader
`080d8974` for all three records, while `e186-fields-trace-wine-bldajpbj`
observes the bank pointer and parameter reads at `080d8b60` and `080d8bac`.
These traces used interpreter fallback. Wine `e186-records-wine-k6d5krm5`
preserves the prior catalog fields and other 119 resources, rejects coherent
edits, checks poisoned mirrors and record boundaries, and compares nine
original-interpreter/separately compiled reader prefixes with equal complete
state and guest clock. Wine `e186-records-scene-wine-1zw44c41` matches
complete state, clock, central/wide video, OAM, dispatch and audio for the
recorded 600-frame action with all 72 record bytes and 634 previously managed
bank bytes altered in the managed machine. All three records are used, and
the bank is read through both the guest bus and typed host sprite lists; no
raw sprite peeks are observed in the selected spans. Wine production CLI
`e186-records-cli-wine-y0aiuls6` creates, reuses and verifies 121 files
matching the tested generation. Other records, banks, selectors, scenes and
consumers, Linux, USA and native Windows remain open. No performance
measurement was performed.


## E274 Soil Evidence compact metadata plan

Wine source-site capture `e274-soil-compact-sites-wine-d95mdss3` observes all
88 sampled metadata bytes over 600 E235 Soil Evidence frames, with 71 selected
read rows inside interpreter scopes. Protected static plan
`e274-soil-compact-metadata-plan-tmxn23ol` verifies a compact projection rooted
at `6c173c`: flags 12, sequence-relative offset `278`, 20 list offsets mapping
to the E271 typed list roots, and selector 0 with 20 frames referencing those
lists in order. Both selected metadata spans are 44 bytes; the list payloads
remain owned by the OBJ resource and must not be declared twice.

This is one sampled projection, not a complete selector/index domain, producer,
all readers or all scenes. Interpreter scopes may include nested accesses. The
plan publishes no owner. Linux, USA and native Windows remain open; no
performance comparison was made.


## E275 Soil Evidence compact metadata runtime

EUR E236-d235 adds the E274 projection to the authenticated conditional
compact OBJ catalog. The two 44-byte source spans contain the flags, 20
relative list offsets and the selected 20-frame sequence. The catalog declares
a dependency on `graphics.obj-static-lists`, which retains sole ownership of
the 20 list payloads; the metadata spans do not duplicate those sources.
Previously decoded catalog fields and 129 other resource files are preserved.

Wine reader `e275-soil-compact-readers-wine-myffsalx` checks the exact source ranges, links, altered-source
mirrors and mixed boundaries, original interpreted and generated reader
instructions, transfers, coherent edit rejection and snapshot ownership. Wine
scene `e275-soil-compact-scene-wine-gmfuh78b` compares 600 recorded Soil Evidence frames with both metadata
spans altered; all 88 metadata bytes are consumed and state, guest clock,
video, OAM, dispatch and five nonempty audio streams match each frame. Wine
CLI `e275-soil-compact-cli-wine-jak27la2` creates, reuses and verifies 131 matching files. Static join
`e275-soil-compact-sample-residual-j6x30g5v` leaves 397 E260 sampled bytes outside E236 source owners. Wine
drained with the local Bluetooth overlay.

This covers one observed selector and continuation, not the full selector
domain, producers, all readers or other scenes. Real BIOS, UI save/Load,
Linux, USA and native Windows remain open. Older revision-sensitive tests were
updated but not rerun; no performance comparison was made.


## E296 Soil Evidence compact record plan

Protected plan `e296-soil-compact-record-source-plan-vrirvg0i` joins the
E285 selected Wine reads with the established E186 24-byte record field
pattern and E274 Soil Evidence metadata. It authenticates the 24-byte
record at `39a91c..39a934` and the observed pointer word at `39aa34`;
20 record bytes and all four pointer bytes occur in the E245 residual sample.
The record points to the already-owned compact metadata root `6c173c` and
selects its represented sequence zero.

The original reader at `080e4afc` loads the pointer at index four from a
table rooted at `39aa24`; its PC literal `e4b80` is already owned. Prior
Wine disassembly `e293-soil-indirect-roots-disassembly-wine-8lucbl5h` and
the E186 reader traces support this field geometry. These static links and
scoped reads do not establish the sampled caller ancestry or the complete
pointer table. In particular, adjacent `39aa38` remains unowned. The plan
publishes no runtime owner. Other records, callers, indices and scenes,
Linux, USA and native Windows remain open. No performance comparison was made.


## E297 Soil Evidence compact record runtime

EUR E246-d245 serves the E296 24-byte compact record and its observed
four-byte indexed pointer through `graphics.soil-compact-record`. The record
links to the existing Soil compact metadata and selector zero; the pointer
reader uses the existing Thumb literal. Those prior resources remain separate
owners. Unselected pointer entries and neighboring bytes continue through
prior-owner or ROM fallback; in particular `39aa38` is not claimed.

Wine reader `e297-soil-compact-record-readers-wine-5w49a8bi` verifies preparation and reuse, all 133 prior
resource definitions and files, poisoned mirrors and mixed boundaries,
rejection of a coherently edited file, retained snapshot ownership and 28
original interpreter/generated record and pointer reader cases. Wine scene
`e297-soil-compact-record-scene-wine-1ti7yhlx` compares 600 retained Soil Evidence frames with all 28 source
bytes altered and verifies reads of both spans. State, guest clock, video,
OAM, dispatch and five nonempty audio streams match each frame. Wine CLI
`e297-soil-compact-record-cli-wine-fr1p_15v` creates, reuses and verifies 135 matching files. Static join
`e297-soil-compact-record-sample-residual-_hg2aqh0` leaves 38 E260 sampled bytes outside E246 sources, none below
`300000`. Wine drained using the local Bluetooth overlay.

The static links do not establish actual ancestry of sampled reads or the
complete pointer table. Other callers, pointers, scenes, real BIOS and UI
save/Load remain open. Older revision-sensitive fixtures were updated but
not rerun; Linux, USA and native Windows remain pending. No performance
comparison was made.
