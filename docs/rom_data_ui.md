# European UI choice list sources

The original widget initializer `0808BBF0` has eighteen direct Thumb callers in
the exact EUR image. Sixteen pass immutable ROM tables; two build a filtered
list in RAM from a shared immutable template. The bounded source contract now
has a strict lossless public decoder and one focused Wine original-reader check. Managed publication and typed runtime
consumption now pass targeted EUR/Wine verification below. EUR is v1/e56-d55;
USA remains e8-d7. Other UI tables and indirect callers are not claimed complete.

## Source domains and original consumers

Twelve direct tables contain36entries. Their original caller constants bound
these exact extents; proximity does not authorize exporting intervening gaps:

- `3979BC`: four entries; `3979DC`: three; `3979F4`: four; `397A14`: three.
- `397A2C`: five entries, used by two callers; `397A54`: four, used by two.
- `397F50`: six entries.
- `3ADF68`: two entries, used by two callers; `3ADF78`: two, used by two.
- `3ADF88`, `3ADF90` and `3ADF98`: one entry each. Identical content at distinct
  physical roots retains distinct provenance.

The shared template at `397A9C` has six entries. Thirteen ranges therefore
contain42entries/336source bytes, disjoint from all fifty current managed source
owners. An all-byte pointer census accounts for eighteen literal occurrences
of these thirteen roots, agreeing with the eighteen declared producer paths.
No full game-wide UI catalog or absence of computed/indirect roots is inferred.

Each entry is eight bytes in source order:

- Byte0 is the identifier returned by selected-entry getter `0808CA64` from
  `widget+4+8*current_index`. It is a value, without a pointer interpretation.
- Bytes1/2 supply unsigned cursor X/Y through original byte reads and live
  cursor-coordinate stores in `0808BDEC`.
- Byte3 is copied intact. All42current values are zero; a semantic read or
  padding-removal rule is not established.
- Bytes4/5/6/7 are raw left/right/up/down navigation bytes. The original input
  routine maps input bits20/10/40/80respectively to these byte reads. High bit
  checks depend on live widget flags; the low seven bits become the stored
  selection index. Preserve the entire raw byte: coordinate reads after a
  move can use the unmasked raw value, including its high bit. Do not normalize
  it into an in-range host link or clamp a target to the exported table count.

The initializer stores the low count byte at `widget+104`, independently of
its original signed full-word `count > 0` copy loop at `0808BC18`. It copies
both original words with `LDMIA` into `widget+4+8*i`. Catalog extents do not
introduce a guest count cap. Current selection, count, input flags, cursor
coordinates, task scheduling and subsequent modifications remain live RAM.

## Filtered live lists

Callers `0808256A` and `0808270A` pass `0200F250`, not a ROM table, to the
initializer. Their producers at `080824E0` and `08082680` read five live
availability slots. They conditionally copy consecutive source template entries
starting at entry1, compacting the source offset only when a slot is included.
They overwrite each included entry's identifier, Y coordinate and up/down
links, and retain the base entry0. The input count is `selected_slots+1`.

The source projection over all32availability masks reads entries0..5, with six
entries needed when all slots are included. This bounds the immutable template;
it does not replace the live predicates or copied list with a precomputed ROM
result. Original write ordering matters: the last-entry down byte is cleared
before copying base entry0, then base up is set to the selected-slot count.
With no selected slots, the base copy overwrites the clear, retaining down1.
Do not impose a host navigation bound or change this raw behavior.

## Verification and remaining work

The static audit authenticates the exact EUR ROM and accepted e55 manifest,
reconstructs all336bytes from eight typed byte fields, checks all50prior owners,
all16direct source/count setups, both template producers, eighteen root literals
and93original instruction fingerprints. The ordered source SHA-256 is
`74323d5436d4fa7dadbb51f571222031410c89fd708276f39b6adf36c7400f7f`.
The private projection was refined during source review to retain the original
zero-selection base-copy overwrite; no implementation or prior proof changed.

Evidence is under ignored `target/rom-data-ui-list-entries/`: `roots.json`,
`source-contract.json`, `template-projections.json`, `widget-body.txt` and
`dynamic-producers.txt`. This unit runs no build or guest execution. The public decoder and focused Wine original-reader criterion are completed
below; targeted managed/native verification is also completed below. Actual production CLI and one positive ordinary Party scene are now verified
below; wider ordinary contexts remain pending.
New Linux/USA/native Windows checks remain pending. No performance/space
measurement, fixture/navigation, Android change, package, agent commit or push.


## Lossless model and original Wine readers

[`ui_choice_lists.rs`](../crates/gba-core/src/rom_data/ui_choice_lists.rs) exposes
`Catalog::from_rom`, shared thirteen-table source definitions and strict JSON
`UiChoiceLists`. Each list has a physical-root ID, exact source offset, direct
or filtered-template kind and typed entries. Navigation retains four separate
raw bytes; byte3 remains explicit `opaque`. `byte_at` and `source_spans` reproduce
all336original bytes and thirteen source roles/hashes. Shape validation binds
the declared physical catalog; it permits the full byte domains for identifiers,
coordinates, opaque data and navigation. Equal data at `3ADF88`/`3ADF98` keeps
distinct IDs and provenance. Source verification independently requires the
exact EUR image and matches the complete decoded model.

One needed affected Windows frontend test build passes without warnings. One
exact ignored test in
[`ui_choice_list_model_tests.rs`](../crates/recomp/src/rom_data/ui_choice_list_model_tests.rs)
passes on Wine, without repeating earlier model/source/runtime/scene checks:

- All336bytes match the independent static source plan, hashes, JSON roundtrip
  and original loadsite coverage. Seventeen shape/width/unknown-field/identity/
  source-content guards pass. A synthetic all-byte-field variant preserves
  nonzero opaque data, high navigation values and full-byte ID/X/Y values while
  failing source authentication as expected.
- Eighteen original initializer copy prefixes pass: thirteen source lists, one
  synthetic variant, and raw counts0/-1/-65536/257. The full signed copy loop and
  low-byte live count remain distinct, including original metadata overwrite
  when copying257entries beyond the declared table projection. No host guard is
  inferred from this probe or the physical source counts.
- Forty-four original selected-entry getter calls and43idle cursor calls pass,
  including all42source entries, the synthetic variant and raw current index255.
  Both live filter producers pass all32availability masks (64prefixes), retaining
  compacted source copies and ordered live overrides, including down1for zero
  available slots. Two post-sound move prefixes confirm that raw high-bit values
  80/86address the original unmasked coordinate locations.

Every probe compares complete EWRAM/IWRAM outside original stack writes; returning
functions also retain SP and callee registers. This is a component source/copy/
reader oracle, not full widget/native/full-machine-state-clock/ordinary parity.
The12231instructions/212PCs are coverage counts. All703hash/mtime guards pass.
No production resource, extractor/decoder revision or existing file changes in
this source-model phase: its EUR e55-d54/USAe8-d7 evidence remains terminal;
managed publication subsequently advances EUR below. New Linux/USA/native
Windows checks remain pending. Evidence: ignored
`target/rom-data-ui-choice-model/windows-model-{build,result}.json`,
`windows-model.log` and `windows-model-europe/result.json`.

## Managed publication and typed runtime reads

EUR v1/e56-d55 publishes `ui-choice-lists.json` as `engine.ui-choice-lists`, with
52generation files/51resources. All50prior decoded files are unchanged and e55
is retained; USA remains e8-d7. The new resource has no asset dependency. Current
full-set validation requires its preceding resource set to remain coherent;
this does not reinterpret other assets as choice-table dependencies.

The common decoder/source plan authenticates thirteen disjoint336byte bindings.
The immutable owner in
[`runtime/ui_choice_lists.rs`](../crates/gba-core/src/rom_data/runtime/ui_choice_lists.rs)
serves typed entry fields to byte/halfword/word guest bus and compiled API reads,
including mirrors and copies through the original BIOS/DMA paths. It composes
with earlier typed owners and raw bytes individually. It does not keep a
second raw ROM copy or replace live counts, filters, selection, cursor input,
copy ordering or task execution. The same owner survives Load/Undo/backup/
Restart after source files become unavailable. Save/native identities and ABI
are unchanged. Usage reports bus/source bytes and identifier/coordinate/opaque/
navigation categories plus the count of physical lists touched.

One affected Windows core/frontend test compile passes without warnings; core
tests were compiled only. Two exact new Wine filters pass on their first runs:

- Publication failure leaves no manifest; create/reuse/repair/move and retained
  e55files pass. Seventeen malformed/missing-file guards reject content, width,
  source span/hash/role, path, dependency, coverage, kind and shape edits while
  retaining the previous owner and complete machine state. Complete-set
  coherence, unavailable-files Load/Undo/backup/Restart and shared owner pass.
  With all336new raw bytes and an old14byte OBJ source altered, direct reads
  still recover all336typed bytes:42ID/84coordinate/42opaque/168navigation, all
  thirteen lists. Four old same-page OBJ bytes still come from their typed owner.
- Twenty-nine original/interpreter/compiled component paths compare complete
  serialized machine state and guest clock with poisoned raw sources. They
  include all thirteen source copies, raw counts0/-1/257, a changed RAM source,
  RAM getter/idle paths and both live filter prefixes at zero/all availability.
  Compiled API reads consume92identifier/183coordinate/92opaque/368navigation
  aggregate bytes. These are read counts, not distinct source coverage.
- Byte/halfword/word boundaries and mirrors pass in both HLE and real-BIOS bus
  modes. Three actual CpuSet/FastSet SWIs and one immediate DMA preserve full
  state/clock. A six-byte copy crosses2old OBJ bytes,2raw bytes and2new UI bytes;
  both SWI and DMA charge2bytes to each owner. An unaligned API word preserves
  two edited raw bytes beside two new typed bytes.

All706hash/mtime guards pass. The3010instructions/186PCs/38native dispatches are
coverage counts, not performance measurements or a complete widget ABI proof.
Source/model/original-reader and older family checks were not repeated.
The targeted dataset identity is
`20de59719c526d547cbea3edaf83920b7d2feac0147c4297329263afde547886`;
new file SHA-256 is
`95f7ccb90cb1d48595f1ace3826d417d3ec2aef5b93f3686ae03444f5fd8abfb`.
Evidence: ignored `target/rom-data-ui-choice-runtime/runtime-targeted-outcome.json`,
`windows-managed-{build,results}.json`, and
`windows-managed-europe/{managed,native}-result.json`.

Actual production CLI preparation/reuse and one positive ordinary Party scene
are now completed below. New Linux/USA/native Windows/new32:9 checks and wider
ordinary contexts remain pending. No performance/space measurement, gameplay fixture/
navigation, Android change, package, agent commit or push was performed.


## Production CLI and positive ordinary Party scene

One needed Windows production CLI build passes without warnings. Its frozen
candidate is `444c2db98c8fa675039e440d8d1414ef236d39b1ae78422e099dd4367ba7352f`;
the canonical executable is restored with its exact original hash/mtime. Main
source content is unchanged; only its mtime is touched for the established Cargo
output invalidation after restoration. No source/model/test/runtime rebuild or
old acceptance is repeated in this stage.

The actual Wine CLI prepares and reuses e56-d55 from an unrelated working
directory with the old native cache. All52generation files exactly match the
accepted managed dataset. All54managed files retain hashes and mtimes on reuse.
The actual generation is recorded in `windows-cli-summary.json` and the later
positive locator; its identity remains
`20de59719c526d547cbea3edaf83920b7d2feac0147c4297329263afde547886`.

The retained Pecorelle1000and Montblanc menu600contexts read zero new UI source
bytes. Both are terminal coverage-negative observations with no full pairs or
longer repeat. A visual four-row battle menu uses a different cursor/text-list
component, so its appearance does not establish use of these copied choice
entries. Read-only authentication/inspection of110existing EUR snapshots finds
no declared direct-list copies; this does not prove global unreachability.
Existing Party unit screenshots and Start/A/A input provenance identify the
relevant component without creating a fixture or editing guest RAM/state.

The single320frame worldmap→Party→unit CLI locator is positive:8bus reads and
32aggregate source bytes (4identifier/8coordinate/4opaque/16navigation), one
physical list. It uses the existing `worldmap.ffstate` snapshot and unchanged
`worldmap-party-unit.input`, with configured widescreen0. The snapshot hash is
`7701a360c2836d8ce1bac59288786cd34922046163048aa32dfa1f09fadcac9a`; input hash is
`bf757e6957e596ba823c9063284c51b6e8766408dbb29247d625869aabae1dfc`.
These are pre-existing private fixtures; no new gameplay recording/navigation,
PC/RAM/save payload override or fixture is introduced.

One exact Wine full-pair test then reuses the same actual CLI generation,
old cache, snapshot, input and320frames with the already frozen targeted test
executable `0f942581b1aee1b00ce314d526499c4bb9badd9365e916d07cb0caac965187df`.
It passes initial-state equality and every complete machine state/guest clock
from frame4519through4839, central/wide image, OAM, dispatch and mixed/PSG/FIFO/
bitwise HLE audio. All five audio streams have actual samples:
`[702240,702240,84480,84480,702164]`. The same32aggregate UI bytes/one list are
observed. All769hash/mtime guards pass. Native and fallback execution both occur;
this is not all-native, new32:9, whole-widget ABI or all thirteen lists' ordinary
coverage. Only targeted checks cover the other twelve lists; their ordinary use remains
unverified.

Evidence under ignored `target/rom-data-ui-choice-runtime/`:
`windows-build.json`, `windows-cli-{processes,no-write,summary}.json`,
`windows-choice-{menu,party}-locator.json`, `existing-ui-snapshot-copies.json`,
`windows-scene-result.json` and `windows-party-unit-full-state-audio.log`.
All source/model/managed/native/actualCLI/positiveordinary children are complete
for the declared EUR scope. New Linux/USA/native Windows/new32:9 and other
ordinary contexts remain pending. No performance/space measurements, Android
change, package, agent commit or push.


## Category-selected UI context templates

A fresh e108 native API address inventory over the accepted EUR Roulette Do it
seed, guest frames 11960..11972, preserves complete state, clock, video, OAM,
dispatch and all five audio streams with its observer active. It records 20,835
unique read contexts, 21,845 image bytes and 732 unowned bytes in 87 merged
ranges. Those ranges are observations, not catalog extents. Interpreter,
host/HLE/direct DMA and other scenes remain outside this native API sample.

Five observed words at 394ea8..394ebc are data copied from template 5, rather
than a jump table. The original selector 08028f4c..08028fda reads the live category
byte through [0200f438]+4. Categories 1..15 select indices
0,6,7,1,2,3,3,3,0,1,4,5,8,9,10; other categories return 0.
The constructor at 08028fdc computes 08394e44+20*index, sets copy count 20
and copies through callback [08370eb4] into its live context +80. This complete
selector image and explicit byte count support eleven 20-byte templates,
394e44..394f20 (220 bytes). Neighbor spacing and the observed five words do not
supply the bound. The projection has no overlap with the 98 e108 owners.

The copied memcpy is executable support and remains in guest memory. A read-only
query of the existing frame 11960 checkpoint executes no guest instructions
and preserves its complete serialized state/clock. Its IWRAM 03005ee8..03005f28
matches ROM d968a8..d968e8 exactly. The observed 03005f04 context corresponds to
ROM d968c4 LDMIA r1!,{r0}; the word-store/subtract-four/loop instructions agree
with a 20-byte copy. The callback's original initializer provenance is already
documented in [UI palette provenance](rom_data_ui_palettes.md). Exported PC/outer dispatch remain context
hints; an exact dynamic caller stack is not inferred.

The second source reader 08029446..0802948c uses the same selector/root and
scales bytes 00/01 by eight for coordinates. The constructor reads LE16 +04 for
n*4 and n allocations and tests byte 09 bit 3. The model therefore retains x/y
in units of eight, that allocation entry count, all flag bits and fifteen
explicitly opaque bytes (02..03,06..08,0a..13). Opaque does not mean padding.
The source contract passes 48 independent opcode pins, selector mapping,
lossless reconstruction, copied-reader identity and current-owner exclusion
under Wine. SHA256:
f3f20b6c259a2353b0db6189b8596c37dabdb312850b7d92a8b0b9d23898c16f.

EUR `e109-d108` publishes `ui.context-templates` in
`ui-context-templates.json`, preserving all 98 earlier resource records and
file bytes. It retains original guest selection, copies, allocation, timing
and live UI state; out-of-owner reads keep their previous semantics. Typed
source counters distinguish coordinates, count, flags and opaque data. No
global UI catalog boundary or guest index cap is introduced. Save format,
native cache and C API are unchanged.

Canonical Windows test and production CLI builds pass without warnings; all
705 source hashes remain stable. CLI create/reuse passes for one retained
current generation: 99 resources, 100 generation files and 102 managed files.
Every previous file matches e108; reuse preserves file sets, hashes and times.
No complete dataset copy is created for each test.

One focused Wine test passes the lossless 220-byte model, JSON/shape/source
checks, poisoned-ROM typed byte reads, mirrored mixed boundaries, field
counters and in-memory coherent-edit rejection. The canonical generation stays
read-only during corruption checks. Original interpretation covers the
selector's complete 256-category byte domain. Eleven original constructor/copy
prefixes and eleven coordinate prefixes compare raw interpretation against
native execution with complete state and clock equality; 308 aggregate native
bytes come from the typed owner. These are conditional source-path proofs,
not all templates' ordinary use or complete constructor execution. Creating
a rejected owner in memory leaves the existing machine/owner unchanged; this
check does not claim a new disk-based failed-reload/lifecycle matrix.

For the later e127-d126 validation, the test report derives current and prior
resource counts from the corresponding manifests. The full EUR/Wine filter
passes: all 256 categories, eleven copy and eleven coordinate native prefixes,
308 typed native bytes and state/clock equality. Coherent edits are rejected in
memory. The report matches its manifests (100 current resources, 98 prior), and
a separate check verifies all 98 prior records and current file hashes.
Evidence: `target/validation/units/rvr17-ui-context-wine-bezw4jap/` and
[recovery record](changes/2026-11-04-prepare-ui-context-validation.json).
This does not establish every template's ordinary use or a disk reload matrix.

A production CLI replay of the existing ordinary IN-19 EUR Roulette Do it
seed advances guest 11960..11972. The new owner serves five reads/20 bytes:
two coordinate, two count, one flag and fifteen opaque bytes. All 99 runtime
resource identities match the manifest. The existing raw/managed native scene
test passes the same twelve frames, comparing complete serialized state,
guest clock, central/wide images, OAM, dispatch and all five exact audio
streams. Positive audio lengths are [26334,26334,3168,3168,26230]. This proves
one ordinary template copy; it does not prove ordinary use of all eleven
records, every consumer, complete UI behavior or global catalog extent.
The previously accepted 600-frame Roulette animation proof was not rerun.

Evidence is under `target/rom-data-e108-next-source-20261103/`. Wine units:
`rom-data-e108-roulette-residue-07n4xsdy`,
`rom-data-ui-templates-source-okgmc0c8`,
`rom-data-ui-templates-contract-j0v50ag4`,
`rom-data-ui-templates-current-1y75gnqo`,
`rom-data-ui-templates-native-h95faz1x` and
`rom-data-ui-templates-ordinary-dk6lf1zi`. Successful diagnostic/native/ordinary
scratch is automatically removed; metadata, logs, the small native object and
source preimages remain. The accepted e109 dataset was retained for comparison until its e110 successor
passed; that payload is now retired with its manifest/logs/results preserved.
The current dataset is under
`target/validation/units/rom-data-loop-compact-current-o_yv6xep/work/resources/`;
[the compact catalog](rom_data_compact_catalog.md) records the e111 successor
and retirement of the later e110 comparison after exact prior-resource checks.
The older e108 payload was likewise retired only after successor equality,
input guards, unit/prefix leases and host activity checks. The original
ROM, personal saves and the one current Do it seed are unchanged. Retired
payloads do not reopen accepted evidence. Linux, USA, native Windows, broader
UI coverage and the remaining ROM/mod phases stay open. No benchmark was run.
