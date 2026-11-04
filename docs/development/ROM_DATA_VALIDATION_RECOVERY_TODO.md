# ROM data validation recovery checklist

Reviewed on November 4, 2026. This checklist tracks skipped validation reruns
and known gaps from recent ROM-data work. It supplements
[ROM_DATA_TODO](ROM_DATA_TODO.md); the original task IDs and accepted evidence
remain authoritative. It is not an exhaustive inventory of every unmigrated
ROM resource.

Current execution scope: **EUR, Windows executables under Wine, functional
checks**. Native Windows, Linux, USA and performance measurements have separate
deferred items below. A build PASS establishes compilation; a runtime PASS
requires executing the selected test and checking its result.

An unchecked regression item means the updated revision expectations have not
been verified by a recorded rerun. Earlier accepted versions remain accepted.
The evidence does not attribute every unchecked item to lack of disk space.

## Recovered items

- [x] **ROMDATA-04-LOOP-AUDIT-BUILD-ENOSPC:** complete the audit CLI compilation
  against the retained 845-file snapshot. Cross-build and independent artifact
  verification passed. The original unit's cleanup refused Cargo hard links and
  its output remains retained. [Result](../changes/2026-11-04-recover-rom-data-audit-cli-build.json).
- [x] **ROMDATA-04-E127-OCTONARY-FULL-FILTER:** execute the complete corrected
  octonary runtime filter with e127-d126 expectations. Owned bytes, boundary
  reads, coherent-tamper rejection and four native/interpreter state-clock
  cases passed together. [Result](../changes/2026-11-04-verify-e127-octonary-full-filter.json).
- [x] Preserve the recovered slot-job runtime/CLI and seventh compact OBJ
  model/runtime/CLI outcomes after their earlier storage failures. These have
  subsequent PASS evidence; the old failures do not create another rerun task.
  [Slot-job evidence](../changes/2026-10-26-consume-european-slot-job-lookup.json),
  [seventh model](../changes/2026-11-03-model-european-compact-obj-seventh-root.json),
  [seventh runtime](../changes/2026-11-03-publish-european-compact-obj-seventh-root.json),
  [seventh CLI](../changes/2026-11-03-verify-european-compact-obj-seventh-cli.json).

## 1. Correct validation reports

- [ ] **RVR-01 — Resource counts:** replace stale hard-coded resource/revision
  summaries in the affected private tests with values from the actual tested
  manifest. The earlier full octonary e127 report said 98; its correction is
  recorded below. Audit the other checklist filters for the same discrepancy.
  Close when the changed reports agree with the manifest and their relevant
  reruns pass. Preserve old evidence verbatim.
  [Observed discrepancy](../changes/2026-11-04-verify-e127-octonary-full-filter.json),
  [octonary test](../../crates/recomp/src/rom_data/compact_obj_animations_octonary_runtime_tests.rs).
  - [x] Correct the octonary report's current and prior resource counts and
    rerun its full EUR/Wine filter. Its report and generated e127 manifest
    both say 100, all 97 prior resources match, and the report retains its
    existing fields. [Recovery](../changes/2026-11-04-fix-octonary-validation-report.json).
  - [x] Correct the UI-context report counts and verify its complete e127
    EUR/Wine filter. The corrected report matches the manifests (100 current,
    98 prior). [Recovery](../changes/2026-11-04-prepare-ui-context-validation.json).
  - [ ] Correct and validate the other affected reports. The source audit
    found fixed historical resource counts in the primary through septenary
    compact OBJ tests, the four-byte and packed lookup, both descriptor-bank
    tests and property masks. Table graphics also has old
    revision, file and resource summaries; reconcile its assertion described
    in RVR-15. The original compact-catalog report has old aggregate counts.

## 2. Run the older tests whose revision expectations changed

The [finite-loop change record](../changes/2026-11-03-extend-european-compact-catalog-from-finite-loops.json)
explicitly records updated older test expectations without their reruns.
The following source files now contain e127-d126/100-resource expectations.
The octonary full filter above is already recovered.

For each item, select the exact filter, verify its compiled source identity,
and inspect required predecessor inputs before launching. If a predecessor
dataset was retired, determine the smallest authenticated comparison required
by this test; do not recreate every historical dataset. Closed scene replays
and accepted new-only filters are not implicitly selected by this list.

Close each item only after its complete selected filter passes, preservation
checks pass, and the record names the tested revision and precise scope.

- [ ] **RVR-02 — Primary compact OBJ:** run
  `european_compact_obj_managed_native_reads` with the current expectations.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_runtime_tests.rs).
- [ ] **RVR-03 — Secondary compact OBJ:** run
  `european_secondary_compact_obj_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_secondary_runtime_tests.rs).
- [ ] **RVR-04 — Tertiary compact OBJ:** run
  `european_tertiary_compact_obj_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_tertiary_runtime_tests.rs).
- [ ] **RVR-05 — Quaternary compact OBJ:** run
  `european_quaternary_compact_obj_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_quaternary_runtime_tests.rs).
- [ ] **RVR-06 — Quinary compact OBJ:** run
  `european_quinary_compact_obj_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_quinary_runtime_tests.rs).
- [ ] **RVR-07 — Senary compact OBJ:** run
  `european_senary_compact_obj_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_senary_runtime_tests.rs).
- [ ] **RVR-08 — Septenary compact OBJ:** run
  `european_septenary_compact_obj_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/compact_obj_animations_septenary_runtime_tests.rs).
- [ ] **RVR-09 — Four-byte conditional lookup:** run
  `european_conditional_lookup_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/conditional_four_byte_lookup_runtime_tests.rs).
- [ ] **RVR-10 — Packed conditional lookup:** run
  `european_conditional_packed_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/conditional_packed_lookup_runtime_tests.rs).
- [ ] **RVR-11 — Conditional descriptor banks:** run
  `european_conditional_descriptor_banks_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/conditional_descriptor_banks_runtime_tests.rs).
- [ ] **RVR-12 — Descriptor bank zero:** run
  `european_conditional_descriptor_bank_zero_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/conditional_descriptor_bank_zero_runtime_tests.rs).
- [ ] **RVR-13 — Property masks:** run
  `european_conditional_property_masks_managed_native_reads`.
  [Test](../../crates/recomp/src/rom_data/conditional_property_masks_runtime_tests.rs).
- [ ] **RVR-14 — Dynamic raw fragments:** inspect the changed assertions in
  `european_dynamic_fragments_managed_reads`, then execute the affected filter.
  Record any additional affected filter explicitly before selecting it.
  [Tests](../../crates/recomp/src/rom_data/dynamic_raw_fragments_runtime_tests.rs).
- [ ] **RVR-15 — Table-selected graphics:** run
  `european_table_graphics_managed_native_and_lifecycle` against the updated
  complete-set expectations. First reconcile its 94-file directory assertion
  with the current e127 generation, recorded as 101 files, and derive the
  result summary from the actual manifest and file set.
  [Test](../../crates/recomp/src/rom_data/table_graphics_runtime_tests.rs).
- [x] **RVR-16 — Clan table alias:** the full
  `european_clan_alias_managed_reads` filter passes with e127-d126 under EUR
  Wine. It preserves the e94-d93 predecessor resources, checks 20 alias bytes,
  42 selector cases with full state/clock equality and transactional rejection.
  [Evidence](../changes/2026-11-04-verify-e127-clan-alias-regression.json).
- [x] **RVR-17 — UI context templates:** the full
  `european_ui_context_templates_match_source_and_native_readers` filter passes
  in EUR/Wine with e127-d126: all 256 category inputs, eleven copy and eleven
  coordinate prefixes, 308 typed native bytes and full state/clock equality.
  Coherent edits are rejected in memory; 98 prior resource records match. The
  corrected report matches both manifests (100 current, 98 prior).
  [Evidence and limits](../changes/2026-11-04-prepare-ui-context-validation.json).
- [ ] **RVR-18 — Original compact-catalog projection:** reconcile the original
  `european_compact_catalog_matches_source_and_native_readers` assumptions
  with the extended catalog, then run the affected original projection checks.
  Keep previously accepted extension-specific filters separately recorded.
  [Tests](../../crates/recomp/src/rom_data/compact_obj_catalog_tests.rs).

## 3. Complete targeted lifecycle evidence

These are unproved integration criteria, rather than known space failures.
Reuse older lifecycle evidence where its implementation and inputs still apply.

- [ ] **RVR-19 / ROMDATA-03/05 — Failed catalog replacement:** select a current
  compact-catalog disk reload/publication failure case. Verify that a missing or
  coherently altered replacement is rejected without changing the active owner
  or machine state. Record whether the old committed generation remains usable.
  In-memory owner rejection alone does not close this item.
  [Current limit](../rom_data_compact_catalog.md).
- [ ] **RVR-20 / ROMDATA-05 — UI context resource lifecycle:** establish the
  missing current-owner Load/Undo/restart scope, including a selected test copy
  becoming unavailable. Verify owner reconstruction/retention and exact state
  restoration for the operations actually exercised. Reuse existing compatible
  lifecycle proofs and state their limits.
  [UI context contract](../rom_data_ui.md#category-selected-ui-context-templates),
  [parent criteria](ROM_DATA_TODO.md#romdata-05--equivalenza-salvataggi-e-prestazioni).

## 4. Fill ordinary-use gaps with source-backed scenes

Each item requires a justified EUR trigger, positive reads of the selected
resource/root, and a raw/managed pair with equal complete state, guest clock,
images/OAM/dispatch and deterministic audio for the selected frame window.
Aggregate catalog counters alone do not identify a particular new root.
Document the observed subset; one scene does not prove every selector or a
complete animation. Rejected leads and closed negative locators remain closed.

- [ ] **ROMDATA-04-PARAMETER-RECORDS27-ORDINARY:** establish positive ordinary
  consumption and its paired comparison. Preserve the 59 observed records and
  eleven closed negative captures; the full domain remains unproved.
  [Gap](../rom_data_gameplay.md#ordinary-sparse-parameter-record-reachability-remains-partial).
- [ ] **ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY:** establish an ordinary
  trigger for the additional 40 banks/eight tables and its paired evidence.
  [Gap](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).
- [ ] **ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY:** identify a source-backed
  ordinary invocation before attempting a new scene. Keep conditional fragment
  acceptance and global-domain uncertainty separate; do not repeat Unit320 or
  the closed state2/state4/selector-witness attempts.
  [Current operational scope](TASK_HANDOFF.md#rom-data-full-eur).
- [ ] **ROMDATA-04, compact OBJ e101–e107:** select one remaining bank with a
  justified ordinary caller and complete its positive paired scene. Record the
  exact bank/selector; conditional reader tests do not close ordinary use.
  [Contracts](../rom_data_obj.md).
- [ ] **ROMDATA-04, UI context templates:** extend ordinary coverage beyond the
  accepted single 20-byte copy toward the remaining records/consumers.
  [Gap](../rom_data_ui.md#category-selected-ui-context-templates).
- [ ] **ROMDATA-04-COMPACT-CATALOG-ORDINARY:** verify a remaining bank/selector
  beyond the accepted Star Cross case. Preserve that closed 480-frame proof.
  [Evidence and limits](../rom_data_compact_catalog.md#ordinary-european-star-cross-use).
- [ ] **ROMDATA-04-LOOP-COMPACT-CATALOG-ORDINARY:** verify a remaining root or
  animation window beyond Soil Evidence, Freezeblink and Stardust. Preserve
  their accepted exact-root watches and paired scenes.
  [Evidence and limits](../rom_data_compact_catalog.md#european-soil-evidence-ordinary-route).
- [ ] **ROMDATA-04, octonary 6cc124:** extend ordinary selector/list or animation
  coverage beyond the accepted IN-19 Roulette 600-frame case.
  [Existing scope](../rom_data_obj.md#ordinary-roulette-producer-and-eur-validation).

## 5. Resolve prerequisites before selecting further proofs

- [ ] **ROMDATA-02-04-6C1E78-DEFAULT-SELECTORS:** reconcile source geometry and
  current-manifest conflicts for caller-derived selectors 4/6, then integrate
  and validate them if supported. Current conditional selectors 3/5 do not
  establish the missing branches.
  [Current selection](TASK_HANDOFF.md#rom-data-full-eur).
- [ ] **ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN:** establish further source-backed
  dynamic domains and consumers before making a global coverage claim.
- [ ] **ROMDATA-01-OBJ-REMAINING-DRAW-ROOTS / CONSTRUCTOR-SOURCES /
  GRAPHICS-PAYLOADS:** select individual authenticated missing consumers or
  payloads. Do not reopen the broad experiment with 61 truncations.
  [Current selections and limits](TASK_HANDOFF.md#rom-data-full-eur).

## 6. Keep explicitly deferred scope visible

These items remain open outside the current EUR/Wine functional selection.
They require an explicit scope change or their stated prerequisite; free disk
space by itself does not select them.

- [ ] **Linux:** validate changed integrations on native Linux when that platform
  is selected; retain older accepted Linux results.
- [ ] **USA:** validate regional extraction/readers/scenes when USA work resumes;
  EUR evidence does not prove USA compatibility.
- [ ] **Native Windows:** run the selected build/runtime/input matrix on an
  actual Windows host; Wine evidence remains separately labelled.
- [ ] **New 32:9 scenes:** verify applicable new-resource render coverage when
  that scene scope is selected; existing 16:9 pairs do not close it.
- [ ] **Performance/storage measurements:** resume only after the user restores
  this scope; define comparison conditions and limits before measurements.
- [ ] **ROMDATA-06/07:** begin resource-replacement mods and extended changes
  after the relevant extraction/consumption/verification prerequisites.

## Recording each recovery

Use `scripts/validation_unit.py`, guard the actual ROM/dataset/state/input/cache
inputs, and require explicit PASS results. Inspect Wine/process ownership before
recovering an interrupted lease. Retain failed diagnostics and preserve source
preimages outside disposable scratch. Follow
[local artifact retention](LOCAL_ARTIFACTS.md#default-lifecycle-for-new-validation).

After each completed item, add its evidence and limits, tick only its verified
criterion, reconcile the original TODO/handoff selection, and use the
[memory publication protocol](MEMORY_WORKFLOW.md#coordinated-memory-publication).
Do not infer parent completion from a passing child.
