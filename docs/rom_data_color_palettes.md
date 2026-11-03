# European Color-Mode unit and UI palettes

This bounded family follows the five original palette helpers at `080CF49C`,
`080CF4E4`, `080CF51C`, `080CF5A0` and `080CF5DC`. It complements the
[unit initial-pose graphics](rom_data_unit_obj.md) and the21 already migrated
[raw UI palette banks](rom_data_graphics.md). EUR e58-d57 now publishes and
consumes this typed family; USA remains e8-d7. Actual production CLI and one
positive ordinary Lutia1700-frame state/clock/image/audio pair pass.

## Original source contract

Each helper reads the low two live bits at `02002FC2`. Mode1 selects variant1,
mode2 selects variant2; mode0 and mode3 take the default. Indexed helpers narrow
their full argument to low8, then multiply by32. These operations stay guest
work; the owned source extents are not new live bounds. `CF4E4` mode1 explicitly
aliases the default physical source, rather than selecting a third bank.

`CF49C` selects the base OBJ block: default40F290, mode14108F0, mode240FE10.
The three original initializers at23B46/23C3A/4FAD4 copy512 bytes to the live
OBJ shadow at03003A60. The16 full palettes per variant are established by these
actual copies. `CF4E4` selects the companion BG block: default/mode140F490,
mode2410010. The same initializers copy512 bytes to03003860. Other nibble and
constant consumers fit these independently proved blocks.

`CF51C` selects supplemental unit/UI colors. Its default/mode1/mode2 bases are
40F690/410CF0/410210. Index16 and above use separate literals512 bytes later;
the code retains its original split and has no local upper bound. All40 direct
calls provide concrete indices. Memcpy consumes its byte count, CpuSet consumes
16 halfwords, and the staged palette reader14B6F0 reads16 u16 colors before
allocation/scheduling. The resulting physical projection per variant is:

- Rows0..3 in full.
- Rows4/5 only bytes4..31 of each row.
- Rows8..22 in full.
- Row23 only bytes24..25.
- Rows24..26 in full.

These holes are preserved. Rows6/7, the first four bytes of rows4/5 and the rest
of row23 are not exported. No following rows are inferred from adjacency.
Original23DB4 selects14-byte slices using bytes at394E04. Its four direct
callers supply arguments1/2,1/2,4/4 and3/3, proving four selector bytes and the
four observed slices. Zero and other low8 arguments retain original behavior;
this caller projection does not make them invalid guest inputs.

`CF5A0` selects only six shared colors: default410810, mode14112F0,
mode240FC90. Both A95E4/A9CBA consumers copy12 bytes. The mode2 root is
intentionally different from the ordering of the other books; no full32-byte
palette or gap is inferred.

`CF5DC` selects special-unit override palettes: default40FA90, mode14110F0,
mode2410610. OriginalCF08C and73C20 derive their selector and destination bank
from the existing metadata14 packed bytes08..0A. Argument low8==1 combines the
high nibble of byte08 with the low nibble of byte09; other arguments use byte0A.
A zero selector skips the copy. Otherwise selector-1 selects a32-byte palette.
All93 authenticated records provide51 active projections, covering source
indices0..8 and destination banks0..4. These are producer projections, not live
record/selector limits. CF08C writes03003A60+32*bank;73C20 writes the live context
at[03002818]+228+32*bank. Ordinary metadata52 jobs return bank nibbles without
this override copy and do not create another palette owner.

The contract contains30 nonoverlapping spans:2873 full u16 BGR555 storage words
and four u8 slice selectors,5750 bytes total. Adjacent requests merge only
within the same book and variant; distinct physical roots remain distinct even
when contents are equal. Every span is disjoint from all52 authenticated e57
owners. The ordered source SHA-256 is
`2e7a4ae78f6841669ed2f11fddd3635d119c9af6edd6602fb3e34f607314485c`.
Static checks pass86 instruction,19 literal and11 consumer-call fingerprints,
the70-call direct Thumb census, concrete copy extents and metadata projections.
This census is not a global indirect-call or reachability claim.

The pointer at370EB4 and copied-IWRAM memory helper are executable support,
already classified. They are not exported as palette data. Guest copies,
publication, fades, allocation, final palette RAM and save/native ABI remain
separate from this immutable source model.

## Lossless source model

The [public module](../crates/gba-core/src/rom_data/color_mode_palettes.rs)
retains physical IDs, book, canonical variant, source offsets, full u16 colors
including bit15 and all u8 selector values. Strict JSON rejects unknown fields;
validation checks exact ordered extents and identity metadata. Authentication
checks the canonical EUR source separately from valid lossless field values.
The shared source plan reconstructs every byte and keeps unowned gaps raw.

The bounded source oracle is in
[color_mode_palette_model_tests.rs](../crates/recomp/src/rom_data/color_mode_palette_model_tests.rs).
The reduced Wine result is recorded below.

## Original Wine readers

The source-model Windows frontend test build passes without warnings. The
first Wine attempt stopped at a test destination guard:30632 usesr5 for its
live destination andr6 for the callback, unlike several surrounding callers.
The model had not reported a difference. The corrected oracle derives these
registers from each nearby original MOV/LDR instead of a guessed common pair.
One changed-oracle Windows build is warning-free; the corrected exact Wine
filter passes. Both failed and successful evidence remain separate.

The accepted filter checks all30 reconstructed spans against the independent
static plan, strict JSON roundtrip and eight shape/authentication guards.
A coherent edit of full color bits and a raw255 slice selector remains a valid
lossless shape and fails canonical-source authentication. All165 original
pointer cases preserve SP, callee registers and RAM, covering modes0/1/2/3/FE,
index15/16 branch boundary, index255 and high-word argument truncation.

Original consumer prefixes pass99 ordinary copies,30 staged source reads,
54 special-unit copies and12 slice-picker calls. The54 special prefixes cover
all nine physical selectors through both CF08C shadow and73C20 context paths,
with representatives exercising both packed metadata branches. All51 active
metadata projections are authenticated statically; no global guest cap follows.
The copies execute the original relocated IWRAM helper or real CpuSet. Staged
reads stop before14A818 allocation/scheduling; slice paths stop before the
subsequent graphics calls. Complete declared destination RAM, outside RAM and
stack/callee properties match at these endpoints. All5750 owned bytes are
observed at actual load/copy sites, including the four selector bytes. The
33654 instructions/770 PCs are coverage only, with no host performance claim.

The accepted frozen executable SHA-256 is
`deb4ed42418dfe256f6d3951c2ed0bd1cec45b27c392b960f791e5e28933012d`;
the decoded model SHA-256 is
`98068136afc44d24640c2c27027ccc34954aa0d92a01a603db767b07cfaab382`.
Evidence includes `windows-model2-build.json`, `windows-model2-result.json`,
`windows-model2.log` and `windows-model2-europe/result.json`. The first attempt
uses the corresponding `windows-model-*` names and is not recorded as passing.
The764 prior build/runtime hash-mtime guards and four compiled source inputs
remain unchanged after the successful check.

Private static evidence is under `target/rom-data-unit-palettes/`:
`source-contract.json`, `audit-sources.py` and original disassembly. These local
files are evidence, not portable fixtures; no game data enters Git or packages.

## Managed publication and typed runtime reads

EUR schema1/extractor58/decoder57 publishes `color-mode-palettes.json`, resource
`graphics.color-mode-palettes`, with an actual `gameplay.linked-metadata14`
dependency. There are54 generation files/53 resources. All52 earlier decoded
files, resource records and coverage entries are identical; e57 remains
available and USA remains e8-d7. The new file matches the accepted source model,
SHA-256 `98068136afc44d24640c2c27027ccc34954aa0d92a01a603db767b07cfaab382`.
The declared special-unit producers must have all their selected source colors
in this owner. This source-set check never caps a live guest argument.

The [immutable owner](../crates/gba-core/src/rom_data/runtime/color_mode_palettes.rs)
retains u16 color fields and the four u8 slice selectors. It derives bytes from
those fields; temporary reconstruction buffers used for authentication are
discarded. Candidate pages394/40F/410/411 do not confer ownership: exact byte
lookups preserve the30 source fragments and every raw gap, composing with all
earlier owners in the same pages. Live Color-Mode values, pointer arithmetic,
RAM copies, fades, allocation, palette publication and save/native ABI remain
original. Transient usage counters are outside machine state serialization.

The first affected Windows build found three old input-guard test constructors
missing the new file argument; no Wine test had started. Adding a zero for the
absent resource preserves those tests. The corrected affected core/frontend
build is warning-free. Core tests are compiled only. Frozen frontend SHA-256:
`6725b65c2850d67822e5a257c72c75e5b1c45035d44f17c05c98d2758e47738a`.
Two new exact Wine filters in the
[runtime oracle](../crates/recomp/src/rom_data/color_mode_palette_runtime_tests.rs)
pass on their first executions:

- Seventeen publication/source/hash/path/dependency/JSON guards reject edits
  transactionally, preserving the previous owner and complete machine state.
  Interrupted publication, create/reuse/repair/move and e57 retention pass.
  Load/Undo, backup recovery and Restart retain the shared owner when its files
  are unavailable.
- All5750 owned bytes come from typed fields with raw source bytes altered:
  1536unit-base,1024companion,2286supplemental,36shared,864special-unit and
  four slice-selector bytes. All30 physical fragments are used. A four-byte
  read from the earlier UI palette owner at40FCB0 charges that owner only.
- Twenty-eight original/interpreter/compiled paths match complete machine
  states and guest clocks. They cover all three modes, base shadow copies,
  short shared colors, partial supplemental copies and CpuSet, staged source
  readers, slice selectors, both metadata14 branches and both destinations,
  raw index65535/mode3, byte/halfword/word APIs and a modified live RAM palette.
  Compiled APIs charge aggregate bytes `[1532,1536,158,44,192,7]` in the six
  categories above. These are repeated read counts, not distinct source bytes.
- All30 source edges, widths, three ROM mirrors, real-BIOS bus mode,
  CpuSet/FastSet and DMA pass. A26-byte mixed transfer contains two new bytes,
  twenty raw bytes and four previous UI bytes; exact owner charges are2/4.
  An unaligned API word preserves two new colors bytes and two independently
  edited raw bytes. All5750 new raw source bytes and four old UI bytes are
  altered in these checks, proving file-derived ownership rather than ROM fallback.

The6805 instructions/451 PCs/5177 native dispatches describe coverage only.
This is acceptance of the declared source-copy/staged-reader prefixes, not a
whole allocator/scheduler or ordinary-scene ABI claim. Source/model proofs and
old family tests were not rerun. The767 prior build-stage hash-mtime guards,
769 runtime guards and nine compiled source inputs remain unchanged.

Private evidence under `target/rom-data-color-mode-runtime/` includes
`windows-managed2-build.json`, `windows-managed2-results.json`, the two Wine
logs and `windows-managed-europe/{managed,native}-result.json`. The failed first
compile is retained separately. The production CLI and ordinary gate were
selected separately after this targeted acceptance, as recorded below.

## Production CLI and ordinary Lutia palette consumption

One changed-code Windows production build is warning-free; the frozen CLI
candidate SHA-256 is
`a052914b4dfbb7b8253d0faf494ac205dd4606fd4cc6753afead1eda68da9e8c`.
The canonical executable's original hash and nanosecond mtime are restored.
Cargo invalidation touches main.rs's mtime only; its contents are unchanged.
Actual Wine CLI create/reuse from an unrelated working directory with the old
native cache verifies all54 generation files against the accepted set. Reuse
leaves all56 managed files' hashes and mtimes unchanged. Actual e58 generation
`generation-XbOkZs` has dataset identity
`e7abf8bb028a77209412f90d8db0eddd91825190f3af23e86a1bcf06889afa97` and
manifest-file SHA-256
`8cb3410f37d67fc8f4e2de0b3752d649e00a3a62b3409de55c75b78619f6823b`.

Two short candidate contexts are terminal coverage negatives: the existing
Party-unit320 replay and neutral languageboot240 each read zero new palette
bytes. Neither received a full pair or repeated build. Other resource use in
those scenes does not establish use of these palette sources.

The retained genuine EUR worldmap snapshot and unchanged
worldmap-engage-battle.input reach Lutia's map laws NOTICE at1700 frames.
One new-family locator reads1102 aggregate color bytes:512unit-base,
512companion and78supplemental,277bus reads/four physical fragments. This real
map load supplies the positive criterion absent from the loaded Party buffers
and initial language screen. No new gameplay input/state fixture or RAM/PC/save
payload override is used.

A test-only `FFTA_STATE_TEST_REQUIRE_COLOR_MODE_PALETTES` guard in the
[ordinary oracle](../crates/recomp/src/rom_data/validation_tests.rs) requires
positive typed color consumption. One affected Windows frontend test build is
warning-free; frozen executable SHA-256
`449a1d2c31eb0adaf78fc6708aba2cbbeac8ddf2494e0b23a9826a1c4a3e85f1`.
One exact Wine1700-frame full pair passes on its first execution using the same
actual CLI dataset, snapshot, input, old cache and width0. Every initial/frame
4519→6219 complete machine state/guest clock, central/wide image, OAM, native
and fallback dispatch, mixed audio, PSG, both FIFOs and bitwise HLE audio match.
Audio sample totals are `[3730650,3730650,448800,448800,3730574]`.
The family guard reads the same1102bytes/277bus reads/four fragments as the CLI.
The final240×160Lutia NOTICE image is inspected; PPM-to-PNG conversion is only
for viewing, preserving every pixel.

This is a mixed native/fallback ordinary proof, not all-native or whole
allocator/scheduler acceptance. The remaining26 physical fragments, shared
colors, special overrides, slice selectors and other Color-Modes remain
ordinary-unverified, with their targeted proofs retained. New Linux/USA/native
Windows/new32:9 remain pending. Source/model/managed/native and previous family
acceptance are not rerun. The828 scene hash-mtime guards and ten compiled source
inputs remain unchanged; the nine earlier runtime inputs are still identical.

Private evidence under `target/rom-data-color-mode-cli/` includes
`windows-build.json`, `windows-cli-summary.json`, `windows-cli-no-write.json`,
`windows-boot-locator.json`, `windows-lutia-locator.json`,
`windows-scene-build.json`, `windows-scene-result.json` and
`windows-lutia-full-state-audio.log`. Negative locators and older source/runtime
maps remain separate. No performance/space campaign, new gameplay fixture or
navigation, private payload edit, Android change, package, agent commit or push.
