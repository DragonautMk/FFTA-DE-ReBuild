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
not ordinary consumption or global absence. No new full-frame pair is claimed
for this catalog, and no previous 600-frame acceptance was repeated.

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

Source evidence: `target/rom-data-compact-catalog-20261103/` and Wine units
`rom-data-compact-catalog-query-410il0gd`,
`rom-data-compact-catalog-windows-xuq32lwl`,
`rom-data-compact-catalog-current-rdbcje1e`,
`rom-data-compact-catalog-native-9zzz4djk` and
`rom-data-compact-catalog-cli-a8rqf6ha`. Successful source/native/CLI scratch is
removed automatically; metadata, original disassembly and logs remain. The
current dataset is retained under the current unit's `work/resources` for
subsequent migration/ordinary work. The e109 comparison payload is retired
only after exact successor acceptance, real-input guards, old/new unit and
Wine-prefix leases, and host activity checks. Its manifest/logs/results and
the native helper remain; original ROM and private saves/inputs are unchanged.
Retired payloads do not reopen accepted criteria.

Linux, USA, native Windows, additional selectors/roots, global extent, ordinary
use and remaining ROM/mod phases are open. No benchmark was run. Game data,
personal saves, recordings and generated native game code stay out of commits
and packages.
