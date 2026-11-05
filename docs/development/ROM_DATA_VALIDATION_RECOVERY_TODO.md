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

- [x] **RVR-01 — Resource counts:** replace stale hard-coded resource/revision
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
  - [x] Correct table-graphics revision/resource/file reporting and compare
    the complete generated file set with its manifest. The Wine test and an
    independent result/input check pass; its original unit is FAIL because
    retention rejected a test-created save state. Diagnostics remain retained.
    [Recovery](../changes/2026-11-04-verify-e127-table-graphics.json).
  - [x] Correct the conditional four-byte lookup report and verify its full
    e127 EUR/Wine filter against the authenticated e95 manifest and payloads.
    The report matches 100 current and 85 prior resources.
    [Recovery](../changes/2026-11-04-verify-e127-conditional-lookup.json).
  - [x] Correct the packed lookup report and verify its e127 EUR/Wine filter
    against authenticated e96 inputs. It reports 100 current/86 prior resources.
    [Recovery](../changes/2026-11-04-verify-e127-packed-lookup.json).
  - [x] Correct descriptor-bank, bank-zero and property-mask report counts and
    verify their complete e127 EUR/Wine filters. Reports match the manifests:
    100 current and 87, 88 and 89 prior resources, respectively.
    [Recovery](../changes/2026-11-04-verify-e127-descriptor-mask-regressions.json).
  - [x] Correct the primary compact OBJ report counts and run its complete
    e127 EUR/Wine filter. Its report matches 100 current and 90 authenticated
    e100 predecessor resources. [Recovery](../changes/2026-11-04-verify-e127-primary-compact-obj.json).
  - [x] Correct and validate the secondary through septenary compact OBJ
    reports. Their complete e127 EUR/Wine filters match the authenticated
    current and predecessor manifests. [Recovery](../changes/2026-11-04-recover-compact-obj-report-counts.json).
  - [x] Correct the original compact-catalog report and its aggregate counts;
    the complete e127 EUR/Wine filter passes under RVR-18.
    [Recovery](../changes/2026-11-04-verify-e127-original-compact-catalog.json).
  - [x] Correct the separate five-loop assertion to the 81 objects present in
    e127 and the accepted e111 report. Its complete current EUR/Wine filter
    passes against the authenticated e110 catalog. [Recovery](../changes/2026-11-04-verify-e127-five-loop-catalog.json).

## 2. Run the older tests whose revision expectations changed

The e128-d127 integration of selectors 4/6 changes the current extraction
revision and catalog projection. The accepted e127 results below remain valid
for e127.

- [x] **RVR-21 — e128 affected regressions:** primary through octonary full
  filters pass with authenticated e100..e107 predecessors; RVR-09..13 pass with
  authenticated e95..e99 predecessors. Dynamic fragments, table graphics,
  clan alias, UI context and RVR-19/20 lifecycle filters also pass. The complete
  catalog and selected new-selector filters pass, and a previous-bank report
  distinguishes 61 identical banks from 62 preserved projections. Inputs were
  guarded and successful runtime scratch removed.
  [Evidence](../changes/2026-11-04-integrate-default-selectors-6c1e78.json).
- [x] **RVR-22 — Updated historical catalog filters:** the changed `6c1e78`
  and five-loop full filters pass on e128 under EUR Wine with e118 and e110
  catalog reconstructions authenticated against their retained manifest hashes.
  The former covers 118 frames and five native cases; the latter verifies the
  previous-bank report and 48 native cases. Guarded inputs passed and successful
  scratch was removed. [Evidence](../changes/2026-11-04-integrate-default-selectors-6c1e78.json).

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

- [x] **RVR-02 — Primary compact OBJ:** the complete
  `european_compact_obj_managed_native_reads` filter passes on e127-d126 under
  EUR Wine with authenticated e100 predecessor files. It checks 2,028 bytes,
  mixed boundaries, coherent-tamper rejection, ten typed selectors and raw gap
  selector 8 with full state/clock equality; 40 typed native bytes. Protected
  inputs passed and successful runtime scratch was removed.
  [Evidence](../changes/2026-11-04-verify-e127-primary-compact-obj.json).
- [x] **RVR-03 — Secondary compact OBJ:** the full e127-d126 EUR/Wine filter
  checks 334 bytes, boundaries, coherent-tamper rejection and five selector
  cases with full state/clock equality and 16 typed native bytes. Its report
  matches 100 current and 91 authenticated e101 predecessor resources; guarded
  inputs passed and runtime scratch was removed.
  [Evidence](../changes/2026-11-04-recover-compact-obj-report-counts.json).
- [x] **RVR-04 — Tertiary compact OBJ:** full e127-d126 EUR/Wine filter passes 108 source bytes, boundaries, tamper rejection and three full state/clock selector comparisons with eight typed native bytes. Report counts match 100 current and 92 authenticated e102 resources; guarded inputs passed and scratch was removed. [Evidence](../changes/2026-11-04-recover-compact-obj-report-counts.json).
- [x] **RVR-05 — Quaternary compact OBJ:** full e127-d126 EUR/Wine filter passes 202 source bytes, boundaries, tamper rejection and five full state/clock selector comparisons with 16 typed native bytes. Report counts match 100 current and 93 authenticated e103 resources; guarded inputs passed and scratch was removed. [Evidence](../changes/2026-11-04-recover-compact-obj-report-counts.json).
- [x] **RVR-06 — Quinary compact OBJ:** full e127-d126 EUR/Wine filter passes 140 source bytes, boundaries, tamper rejection and three full state/clock selector comparisons with eight typed native bytes. Report counts match 100 current and 94 authenticated e104 resources; guarded inputs passed and scratch was removed. [Evidence](../changes/2026-11-04-recover-compact-obj-report-counts.json).
- [x] **RVR-07 — Senary compact OBJ:** full e127-d126 EUR/Wine filter passes 756 source bytes, boundaries, tamper rejection and eight full state/clock selector comparisons with 28 typed native bytes. Report counts match 100 current and 95 authenticated e105 resources; guarded inputs passed and scratch was removed. [Evidence](../changes/2026-11-04-recover-compact-obj-report-counts.json).
- [x] **RVR-08 — Septenary compact OBJ:** full e127-d126 EUR/Wine filter passes 20 source bytes, boundaries, tamper rejection and two full state/clock selector comparisons with four typed native bytes. Report counts match 100 current and 96 authenticated e106 resources; guarded inputs passed and scratch was removed. [Evidence](../changes/2026-11-04-recover-compact-obj-report-counts.json).
- [x] **RVR-09 — Four-byte conditional lookup:** the complete
  `european_conditional_lookup_managed_native_reads` filter passes on e127
  under EUR Wine. It checks all 296 bytes, three native prefixes, six indices,
  state/clock equality and tamper rejection. The e95 predecessor copy was
  authenticated against its retained manifest and removed with test scratch.
  [Evidence](../changes/2026-11-04-verify-e127-conditional-lookup.json).
- [x] **RVR-10 — Packed conditional lookup:** the full
  `european_conditional_packed_managed_native_reads` filter passes on e127
  under EUR Wine. It checks 132 bytes, boundaries, metadata14 alias composition,
  two native prefixes, full state/clock equality and tamper rejection. Its e96
  predecessor copy was authenticated and removed with test scratch.
  [Evidence](../changes/2026-11-04-verify-e127-packed-lookup.json).
- [x] **RVR-11 — Conditional descriptor banks:** the complete e127 EUR/Wine
  filter passes: 5,564 source bytes, five native prefixes and eight cases each,
  full state/clock equality and tamper rejection. The 87-resource e97
  predecessor was authenticated and removed with test scratch.
  [Evidence](../changes/2026-11-04-verify-e127-descriptor-mask-regressions.json).
- [x] **RVR-12 — Descriptor bank zero:** the complete e127 EUR/Wine filter
  passes: 124 source bytes, three native prefixes and five cases each, full
  state/clock equality and tamper rejection. The 88-resource e98 predecessor
  was authenticated and removed with test scratch.
  [Evidence](../changes/2026-11-04-verify-e127-descriptor-mask-regressions.json).
- [x] **RVR-13 — Property masks:** the complete e127 EUR/Wine filter passes:
  140 source bytes, one native prefix and six boundary indices, full state/clock
  equality and tamper rejection. The 89-resource e99 predecessor was
  authenticated and removed with test scratch.
  [Evidence](../changes/2026-11-04-verify-e127-descriptor-mask-regressions.json).
- [x] **RVR-14 — Dynamic raw fragments:** the full
  `european_dynamic_fragments_managed_reads` filter passes in EUR/Wine with
  e127-d126. It checks all 1,304 bytes, seven source spans and boundaries, a
  raw/managed DMA state-clock pair, predecessor preservation and transactional
  tamper rejection. The other filters were not selected by this recovery.
  [Evidence](../changes/2026-11-04-verify-e127-dynamic-fragments.json).
- [x] **RVR-15 — Table-selected graphics:** the complete
  `european_table_graphics_managed_native_and_lifecycle` test passes on e127
  under EUR Wine; independent verification confirms the corrected manifest
  report and protected inputs. The original unit is FAIL because its cleanup
  guard retained a test-created save state. The failed unit and diagnostics are
  preserved. [Evidence](../changes/2026-11-04-verify-e127-table-graphics.json).
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
- [x] **RVR-18 — Original compact-catalog projection:** the complete original
  filter passes with e127-d126 under EUR Wine. Its manifest-derived report
  covers 63 banks, 330 spans and 20,257 bytes; all 205 represented selectors
  and 63 out-of-projection probes pass full state/clock comparisons. The first
  99 e126 resources match, guarded inputs pass, and scratch was removed.
  Earlier extension-specific proofs remain separate.
  [Evidence](../changes/2026-11-04-verify-e127-original-compact-catalog.json).

## 3. Complete targeted lifecycle evidence

These are unproved integration criteria, rather than known space failures.
Reuse older lifecycle evidence where its implementation and inputs still apply.

- [x] **RVR-19 / ROMDATA-03/05 — Failed catalog replacement:** the e127
  EUR/Wine disk reload test rejects both a missing catalog and a structurally
  valid edit with an updated manifest hash. The active owner and complete
  machine state remain unchanged; that owner still serves a catalog byte and
  the untouched committed generation can be reattached and read. Inputs were
  guarded and test scratch removed. This tests disk attachment, not an injected
  interruption during generation writing. [Evidence](../changes/2026-11-04-verify-catalog-disk-replacement-failures.json).
- [x] **RVR-20 / ROMDATA-05 — UI context resource lifecycle:** an e127
  EUR/Wine test makes a selected dataset copy unavailable, verifies failed
  reattachment, exact Load/Undo/Restart states and retained owner, and reads
  all 220 typed bytes after each operation. A fresh owner is reconstructed from
  the restored copy and also reads all 220 bytes. Guarded inputs passed and
  synthetic state files and scratch were removed. [Evidence](../changes/2026-11-04-verify-ui-context-resource-lifecycle.json).

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
- [x] **ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY:** White Flowers entry
  reads three aggregated groups and passes a 700-frame EUR Wine raw/managed
  pair. The separate CLI does not identify exact groups; all 40 banks, eight
  tables and consumers are not claimed as ordinary coverage.
  [Evidence](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).
- [ ] **ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY:** identify a source-backed
  ordinary invocation before attempting a new scene. Keep conditional fragment
  acceptance and global-domain uncertainty separate; do not repeat Unit320 or
  the closed state2/state4/selector-witness attempts.
  [Current operational scope](TASK_HANDOFF.md#rom-data-full-eur).
- [x] **ROMDATA-04, compact OBJ e101–e107:** IN-08 Kent's Ball scene observes
  senary bank `6c6928` selector 2 and passes a 600-frame EUR Wine raw/managed
  pair. Other selectors, banks, consumers and full animations remain open.
  [Evidence](../rom_data_obj.md#ordinary-european-senary-selector-2).
- [x] **ROMDATA-04, UI context templates:** extend ordinary coverage beyond the
  prior template 5 copy: IN-08 Action observes template 2 and passes a 64-frame
  EUR Wine raw/managed pair. Other records and consumers remain open.
  [Evidence](../rom_data_ui.md#category-selected-ui-context-templates).
- [x] **ROMDATA-04-COMPACT-CATALOG-ORDINARY:** IN-08 Red Spring observes
  bank `6c1e78` selector 5 and passes a 600-frame EUR Wine raw/managed pair.
  The accepted 480-frame Star Cross proof remains intact; other banks, selectors,
  animations and consumers remain open.
  [Evidence and limits](../rom_data_compact_catalog.md#european-6c1e78-ordinary-route-candidates).
- [x] **ROMDATA-04-LOOP-COMPACT-CATALOG-ORDINARY:** a distinct IN-08 Blue Screw
  scene observes `6c1e78` selector 6 and passes a 600-frame EUR Wine pair,
  extending the previously accepted selector 5 and Soil Evidence, Freezeblink
  and Stardust scenes. Other roots, selectors and complete animations remain
  open; the conditional `6c4c50` Alpha lead still lacks a suitable EUR scene.
  [Evidence and limits](../rom_data_compact_catalog.md#european-soil-evidence-ordinary-route).
- [x] **ROMDATA-04, octonary 6cc124:** observe Roulette selectors 0 and 1 and
  extend the EUR Wine raw/managed comparison to 1,200 frames. Selector 2,
  complete animation and other consumers remain open.
  [Evidence](../rom_data_obj.md#ordinary-roulette-producer-and-eur-validation).

## 5. Resolve prerequisites before selecting further proofs

- [x] **ROMDATA-02-04-6C1E78-DEFAULT-SELECTORS:** reconcile source geometry and
  current-manifest conflicts for caller-derived selectors 4/6, then integrate
  and validate them if supported. Current conditional selectors 3/5 do not
  establish the missing branches. Retained EUR/Wine source evidence confirms
  four-selector geometry (524 bytes) and no e127 non-catalog resource conflict.
  EUR e128-d127 integrates the additional selectors; Wine create/reuse, native
  readers and complete catalog checks, regression filters and 12-frame CLI
  startup pass. Ordinary use, other callers and deferred platforms remain open.
  [Evidence](../changes/2026-11-04-integrate-default-selectors-6c1e78.json).
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
