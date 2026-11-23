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

## Selecting a recovery

Use the [ROM-data progress rules](ROM_DATA_TODO.md#progress-rules) to choose
one authorized criterion. Before a build, replay or other expensive step,
record the deliverable, exact required evidence, risks and stopping condition.
Name the tested revision, path or filter and any missing prerequisite.

Check linked terminal results and protected inputs first. Reuse accepted proof
where the implementation, dependencies, inputs and assumptions still support
the selected claim. If a change invalidates that proof, run the smallest
sufficient affected regression set under the
[validation protocol](ROM_DATA_VALIDATION.md#risk-based-validation-for-new-work).
Keep the stated filter, ordinary-use, equivalence and lifecycle gates for each
selected item. Preserve known mismatches and resolve them before accepting the
affected criterion, regardless of the planned budget.

Stop when the selected criterion passes. Follow up on a concrete correctness
risk to that criterion or record an unavailable proof and its prerequisite.
Keep unproved broader coverage open and record optional leads in the
[optional investigation backlog](ROM_DATA_TODO.md#optional-investigation-backlog).
Honor existing pauses and ownership. Obtain authorization before resuming a
paused task, restarting interrupted work, changing shared Wine resources or
changing another agent's selected goal.

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
- [x] **ROMDATA-04-CONTEXT-3710D4-CLI:** the EUR e159 production CLI creates,
  reuses and verifies 104 matching files; its bounded Wine world-map run
  observes four context bytes (`27uqnksb`). Other consumers remain open.
- [x] **ROMDATA-04-SUBSTITUTION-371158-CLI:** the EUR e160 production CLI
  creates, reuses and verifies 104 matching files under Wine (`mgaon9d6`).
  Its bounded world-map scene reads no substitution callback bytes.
- [x] **ROMDATA-04-RANGE-37140C-CLI:** the EUR e161 production CLI creates,
  reuses and verifies 105 matching files under Wine (`emoffrjv`). Its bounded
  world-map scene reads no range-lookup bytes.
- [x] **ROMDATA-04-ADJACENT-371464-CLI:** the EUR e162 production CLI creates,
  reuses and verifies 106 matching files under Wine (`j4r9sopl`). Its bounded
  world-map scene reads no tilemap geometry bytes.
- [x] **ROMDATA-04-SCRIPT39-REGISTRY-CLI:** the EUR e163 production CLI creates,
  reuses and verifies 107 matching files under Wine (`vgo1cewu`). Its bounded
  world-map window reads no opcode-39 pose bytes.
- [x] **ROMDATA-04-SCRIPT39-SPECIAL-CLI:** the EUR e164 production CLI creates,
  reuses and verifies 108 matching files under Wine (`72_aby76`). Its bounded
  world-map window reads no new special-pose bytes.
- [x] **ROMDATA-05-E162-SCENES scoped:** compare four retained checkpoints for
  three frames each (`ig6dmb65`) and an eight-frame production cold boot
  (`dcq3ml1a`) on EUR/Wine. No selected new-source reads occurred in the
  checkpoint windows; the cold boot read no geometry bytes.

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
for e127. The later conditional marker integration changes the current
revision and static OBJ catalog again; the accepted e128 results remain valid
for e128.

- [x] **RVR-23 — e129 affected regressions:** 38 EUR/Wine filters pass: eight
  compact OBJ, five conditional lookup, seven data/lifecycle, full catalog and
  default selectors, and sixteen historical catalog filters. Predecessor
  projections are authenticated against retained manifests; reports separate
  unchanged resources from the preserved previous OBJ projection. Inputs pass
  and successful scratch is removed. Ordinary marker use and deferred platforms
  remain separate. [Evidence](../changes/2026-11-05-publish-conditional-ram-slot-obj-markers.json).

- [x] **RVR-24 — e130 affected regressions:** 38 EUR/Wine full filters pass: 13 compact/conditional, seven data/lifecycle, two complete catalog/default selector and 16 historical catalog comparisons. Historical projections are authenticated against retained manifests; prior OBJ projections remain preserved. Protected inputs pass and successful scratch is removed. Ordinary use and deferred platforms remain separate. [Evidence](../changes/2026-11-05-verify-e130-historical-regression-matrix.json).

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

- [x] **ROMDATA-05-E170-BASE-SAVE-IDENTITY:** E170 EUR/Wine `_mjaz8sl`
  verifies equal snapshot bytes and bidirectional snapshot/cartridge-save
  interchange with and without the unmodified base dataset. Invalid snapshot
  identity, version, checksum or payload and a wrong-ROM dataset are rejected
  transactionally; invalid restart boot state cannot overwrite a synthetic
  save. Cartridge files remain raw game-format data without a dataset-identity
  header. GUI, game-save semantics, mods and deferred platforms remain open.
  [Evidence](../rom_data_runtime.md#identity-snapshots-and-diagnostics).
- [x] **ROMDATA-05-E170-GUI-LOAD-UNDO:** Wine/Xvfb `o9pxy5_a` exercises actual
  Load, Undo and reload on the Bank route in isolated raw/managed sessions.
  Complete-state hashes and clocks match at all six endpoints, including the
  repeated 120-frame route after reload; the managed range counter totals 46
  bytes versus 23 in the single-pass run. This uses aggregate counters and
  endpoint checks, not per-frame video/audio or native Windows evidence.
  [Evidence](../rom_data_runtime.md#european-bounded-range-lookup).
- [x] **ROMDATA-05-E170-GUI-BACKUP-RESTORE:** Wine/Xvfb `xssda1xe` restores a
  synthetic backup over a different test save using the actual menu, loads the
  Bank slot and repeats its 120-frame input. Raw and managed state hashes and
  clocks match at five endpoints; 46 range bytes are counted across both
  passes. Other scenes, real saves, per-frame audio/video, native Windows,
  mods and the broader lifecycle criteria remain open.
  [Evidence](../rom_data_runtime.md#european-bounded-range-lookup).
- [x] **ROMDATA-05-E170-UNALIGNED-SOURCE-READS:** EUR/Wine `8_1wpiix` checks
  5,459,217 odd halfword and 8,159,411 non-word-aligned word offsets fully
  covered by the E170 source union. Bus and native API match original bytes
  across three mirrors with inverted backing ROM. Mixed covered/uncovered
  accesses beyond sampled boundaries, guest instructions, scenes and other
  platforms remain open. [Evidence](../rom_data_runtime.md).

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

- [ ] **ROMDATA-04-PARAMETER-RECORDS27-ORDINARY:** establish cheat-free ordinary
  provenance for the positive full-state pair already observed from a save of
  unknown ancestry. Preserve the 59 observed records and closed negative
  captures; the live domain, other scenes and complete consumer set remain
  unproved. Do not repeat accepted pairs to reconstruct context.
  [Gap](../rom_data_gameplay.md#ordinary-sparse-parameter-record-reachability-remains-partial).
- [x] **ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY:** White Flowers entry
  reads three aggregated groups and passes a 700-frame EUR Wine raw/managed
  pair. The separate CLI does not identify exact groups; all 40 banks, eight
  tables and consumers are not claimed as ordinary coverage.
  [Evidence](../rom_data_ui_graphics.md#ordinary-table-extension-reachability-remains-partial).
- [ ] **ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY:** identify a source-backed
  ordinary invocation before attempting a new scene. IN-08 Link selections
  activate the gate, but two 240-frame observations have zero fragment reads;
  a positive read and paired scene remain open. Keep conditional acceptance and
  global-domain uncertainty separate; do not repeat closed negative routes.
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

### Additional open ROM-data work through E158

These unchecked entries preserve incomplete work; they do not reopen any
accepted child result. Select a source-backed criterion and inspect its
current evidence before running another check.

- [ ] **ROMDATA-01 / ROMDATA-02 / ROMDATA-03 / ROMDATA-04:** complete the EUR
  source and consumer inventory, extract/decode remaining families, connect
  them to preparation and managed runtime, and account for raw bytes, host
  and translated consumers, DMA, indirect calls, ABI compatibility and guest
  timing. E158 is a partial increment;
  the unchecked parent criteria in [ROM_DATA_TODO](ROM_DATA_TODO.md) remain
  authoritative.
- [ ] **ROMDATA-01-CONTEXT-3AE42C:** establish remaining language-byte writers,
  index extent and other ordinary publication paths. Title IDs 1–4 have
  ordinary publication witnesses; confirming already-active ID 0 takes the
  unchanged branch (`9k_mye2m`). Returning from French and publishing ID 0
  now passes an ordinary EUR/Wine interpreter pair (`1riwnrce`). Other writers,
  native attribution beyond the observed choices, the global value domain,
  Linux, USA and native Windows remain open. The E164 conditional full-byte
  index audit (`s7fuvptv`) found
  39 already-owned and 222 unbound physical pairs. E166 recheck (`a1j2lsxv`)
  found 256 owned and five unowned pairs after the two archives were added.
  E167 recheck `60xqqy83` finds 260 owned; the final two-byte gap at
  `3ae472..3ae474` remains unclassified. Bounded audit `s82earyr` identifies
  conditional reader inputs 35 and 30 and neighboring independent list roots;
  its alignment and zero bytes do not prove padding. E168-d167 adds a separate
  opaque projection; publication and controlled original-reader Wine checks
  pass (`mncnsstk`). E168 assigns all 261 conditional physical pairs (`4ami2zcc`)
  without establishing their live input domain. Production Wine CLI creation,
  reuse and verification pass (`wklf6sch`); ordinary reachability of inputs
  30/35, Linux, USA and native Windows remain pending.
- [x] **ROMDATA-01-CONTEXT-3AE42C-NATIVE-LANGUAGES scoped:** E170 Wine
  `4yp7q7yd` follows ordinary selections 1–4 and return to 0 with generated
  dispatch. Each value changes as expected, both selected title-cursor
  coordinates receive direct generated API reads, and full state, clock,
  video, OAM, dispatch and audio match per frame. Direct native reads of the
  separate `3ae42c` bank were not covered by that check; the global writer/value
  domain remains open.
  [Evidence](../rom_data_obj.md).
- [x] **ROMDATA-01-CONTEXT-3AE42C-DIRECT-NATIVE scoped:** E170 Wine `arfmrs12`
  follows five ordinary startup selections and attributes both bytes of each
  position pair to direct generated API reads. All 2,120 raw/managed frames
  match state, clock, video, OAM, dispatch and audio with source bytes altered.
  Other readers, values, scenes and platforms remain open.
  [Evidence](../rom_data_obj.md).
- [ ] **ROMDATA-01-CONTEXT-CALLBACK-3AE47C:** validate the drafted E167
  independently rooted callback descriptor. The controlled original
  dispatcher state-4 Wine retry `md0zxb9k` and production CLI `555lr8qm`
  pass; static join `1rjf486k` links an ordinary retained trace and checkpoint
  to the descriptor. Wine continuation `wklf6sch` reads 14 managed descriptor
  bytes and matches three framebuffer/OAM frames and the final serialized
  state. E170 Wine `hf5czvkg` additionally compares three frames of full state,
  clock, video, OAM and nonempty audio with the 12 source bytes altered; 14
  typed descriptor bytes are read. Initial `yxtwldmj` failed an intermediate
  call-PC observation hidden by compiled chaining. The final descriptor byte is opaque; other scenes, Linux, USA and
  native Windows remain open.
- [ ] **ROMDATA-01-INDEXED-DIRECT-3AE484:** extend coverage of the EUR
  four-program archive identified by `08023c98`; source audit (`j80q8jx4`),
  controlled reader (`qk3xqc4z`), production CLI (`e87ug9pq`) and one bounded
  ordinary-input continuation (`abz4lk0q`) pass on Wine. Other scenes and
  consumers, Linux, USA and native Windows remain open.
- [ ] **ROMDATA-01-INDEXED-DIRECT-3AE5A4:** finish validation of the adjacent
  EUR archive. Static source audit `zy8p2uxf` and E166 controlled Wine test
  `puxz2kz7` pass for 41 programs; production CLI Wine `ozq4vcii` passes.
  Its bounded world-map window reads no archive bytes; a separate continuation
  (`abz4lk0q`) reads both archives and passes an 11-frame pair. Other scenes
  and consumers, Linux, USA and native Windows remain pending.
- [ ] **ROMDATA-04-INDEXED-DIRECT-ORDINARY:** extend ordinary coverage of both
  independently rooted archives. Wine `abz4lk0q` reads both in a cold-origin
  continuation and compares 11 frames of state, clock, video, OAM and nonempty
  audio with managed ROM sources altered. The original cold input prefix now
  joins the checkpoint after the test's existing audio-output drain
  (`v89740hr`); raw snapshots differ in that silent output queue. The full
  8,820-frame recorded continuation now passes on E170 (`klmjn8sv`). A separate
  replay using the guarded translation matches the normalized historical
  endpoint (`0gj2cglj`). A separate bounded guarded E170 raw/managed pair
  reads both archives and matches 11 frames (`u75gcclc`). Other scenes and
  consumers remain open.
  The source-guided program-1 checkpoint
  (`p0qt7o6q`) yields zero archive reads in 6,000 recorded Wine frames
  (`dh34xwul`); no scene pair ran. Determine whether the command or flag gate
  was reached before selecting another route; do not repeat this negative
  window. The proposed program-2 command requests 13, but the direct loader
  branch requires 12; the previous program-2 lead is rejected. Static audit
  `eduynil3` finds four request-12 script instructions (event keys 154, 156,
  158 and 170). DS-20 slot 65 is a possible program-160 precursor but an
  intervening conditional and unknown legacy ancestry limit its evidence.
  A separate source audit `0ivxkuzk` associates the loader with mode slot 2 and
  an existing cold-input checkpoint, without proving the call occurred there.
  Offline audit `acjbv2hh` authenticates the frame-180 preselection and retains
  shifted input; `v89740hr` verifies the cold prefix join after draining the
  differing output queue. Use the DS-20
  diagnostic only for a distinct missing route, since its legacy ancestry is
  unknown. Linux and USA remain deferred.
- [x] **ROMDATA-04-INDEXED-DIRECT-COLD-PREFIX:** the original acquisition helper
  replays the authenticated neutral input from cold to frame 180 under Wine;
  `v89740hr` verifies the retained checkpoint matches after draining only its
  131,072 silent output samples, exactly as the accepted continuation already
  does. The shifted input matches the original events. Other routes and the
  full recorded acquisition remain outside this result.
  [Evidence](../rom_data_indexed_graphics.md).
- [x] **ROMDATA-04-INDEXED-DIRECT-FULL-RECORDED-ROUTE:** E170 Wine `klmjn8sv`
  compares all 8,820 frames from the authenticated frame-180 origin through
  the recorded frame-9,000 endpoint. With both archive spans and caller literals
  inverted during execution, the archives serve 302 and 3,292 bytes; full state,
  clock, video, OAM, dispatch and five nonempty audio streams match each frame.
  The pair itself uses a legacy translation and does not establish historical
  endpoint identity. Other scenes, consumers and platforms remain open.
  [Evidence](../rom_data_indexed_graphics.md).
- [x] **ROMDATA-04-INDEXED-HISTORICAL-ENDPOINT scoped:** the recorded Wine
  continuation using retained guarded translation `qpbgtmmw` matches the
  historical frame-9,000 complete state after normalization of audio output,
  tap enablement and host mixer shadows (`0gj2cglj`). The earlier legacy-object
  attempt `tagk77bt` remains FAIL and retains its guest-state differences.
  This separate replay does not repeat the legacy raw/managed pair with the
  guarded object; other routes and platforms remain open.
- [x] **ROMDATA-04-INDEXED-GUARDED-PAIR scoped:** E170 Wine `u75gcclc` uses
  the guarded translation and matches 11 ordinary raw/managed frames with the
  two archive sources and caller literals altered in the managed ROM copy.
  Both archives have positive typed reads; state, clock, video, OAM, dispatch
  and audio match, with nonempty mixed audio. Other scenes, consumers and
  platforms remain open. [Evidence](../rom_data_indexed_graphics.md).
- [x] **ROMDATA-01-TITLE-INVENTORY-RECONCILE scoped static:** `d83mbwp2`
  joins the retained title-language/main/opening Wine samples to E228. All
  62/8/8 previously unowned sampled bytes have declared sources; earlier Wine
  already exercised the five title-selector choices. This does not prove other
  routes or current per-read dispatch.
  [Evidence](../rom_data_runtime.md#e248-title-and-opening-inventory-reconciliation).
- [ ] **ROMDATA-04-TITLE-MAIN-BANK-OTHER / FIRSTBANK-ZERO:** investigate other
  scenes, producers and native-dispatch attribution. All five nonempty lists
  in the E158 title main-bank sequence have one ordinary Wine interpreter
  witness. E170 Wine `vbye2pnv` passes a 180-frame mixed-dispatch state/video/
  audio pair and records all five headers at interpreter site `0800153c`, with
  no direct native API header reads. First-bank sequence zero, other scenes and
  configurations, and the global producer domain remain open.
- [x] **ROMDATA-04-TITLE-MAIN-BANK-NATIVE scoped:** E170 Wine `8j504dl0`
  starts from a protected EUR save with `FFTA_PATCH_SPR=0`, follows the retained
  boot input and compares 3,885 raw/managed frames. All five nonempty headers
  are read through the generated API with their sources altered; full state,
  clock, video, OAM, dispatch and audio match. The default hook route remains
  separately interpreter-scoped. [Evidence](../rom_data_obj.md).
- [x] **ROMDATA-04-TITLE-MAIN-BANK-COMPLETE-ANIMATION scoped:** E170 Wine
  `ntggzh1c` verifies all six original sequence phases and their 63 consecutive
  exclusive header frames, then the animator completion flag. A 180-frame
  raw/managed pair matches state, clock, video, OAM, dispatch and nonempty
  audio with altered list sources. Other banks, producers and scenes remain
  open. [Evidence](../rom_data_obj.md).
- [ ] **ROMDATA-04-BATTLE-40D300-OTHER:** establish source-backed ordinary use
  of the remaining directory-selected battle lists; index 56 has a scoped
  Wine witness, not full-directory ordinary coverage.
- [ ] **ROMDATA-04-METADATA14-ID1-ALIAS:** verify the full importer, other
  scenes/consumers and native dispatch on the ordinary route. The controlled
  Wine interpreter/native prefix (`56wpztn8`) and one ordinary interpreter
  frame (`gpir8qjh`) read the row through the packed owner and match raw state.
  E170 Wine `3s1xbvpx` passes a 240-frame raw/managed generated-dispatch pair
  with the row altered, five typed reads and matching state/video/OAM/audio;
  Wine `_hfk7lti` now compares one complete ordinary `080ccf9c` converter
  invocation through return, with the actual alias read typed and the row
  altered; state, clock, video, OAM and mixed audio match. The batch wrapper
  `080ce18c` now passes five controlled E170 Wine cases (`g2cugem4`) through
  return, including bounds and an occupied destination slot, using an
  authenticated two-unit header. Wine `049hqes5` observes all five ordinary
  row requests during generated
  dispatches without interpreter instruction scope. Wine `1gs1xgeu` confirms
  all five requests directly invoke the generated-code memory API and none is
  nested in `interp_one`. Static join `7xxd2a4i` connects script key 161,
  program 151 and opcode 74 to the batch wrapper. E170 event audit `dlkjby8u`
  finds no direct key-161 row; event row 22 is the sole special key-74 row and
  mission record 22 references it. The selected RAM table value is not 161 in
  279 valid retained states (`3xgls5b2`); this does not establish the value
  during event execution. The event-index byte is also not 22 in the same
  retained states (`_13erapb`); that byte can be stale and is not a complete
  mission-entry indicator. Source audit `6d1p8x1c` pins two operand-based
  selector-table writers and finds no 161 operand among their 46 main and
  145 auxiliary declared script instances; other writers, aliases and
  dynamic streams remain open. Conditional source audit `o9iv4u54` finds the
  queue initializer clears four slots, sets selector 1 and writes slot 0;
  without intervening writes the special event reads zero from slot 1. The
  copied IWRAM body and absence of later mutations are assumptions, not live
  observations. Ordinary batch reachability, other headers, exact
  guest PCs, other calls and scenes remain open.
- [x] **ROMDATA-04-METADATA14-ID1-ALIAS-NATIVE-READS scoped:** attribute all
  five row requests on the recorded EUR battle route to direct generated-code
  memory API calls while preserving the 240-frame raw/managed comparison.
  Wine `1gs1xgeu` and the unwrapped regression `0zpte7lm` pass; this does not
  identify the individual guest PCs or cover other routes.
- [ ] **ROMDATA-01-CONTEXT-3710D4:** verify additional ordinary contexts,
  remaining consumers and full reader ABI. EUR e159 Wine publication, original
  reader cases and one ordinary context-3 window pass (`8dbsgppp`); production
  CLI passes (`27uqnksb`). E170 generated-dispatch Wine pair `5eyv63je` matches
  41 world-map frames with altered sources and four typed context-3 reads;
  controlled Wine `9l5p79sq` additionally compares all 256 getter byte values
  and their 0x100 aliases plus three field loads across all 33 records, using
  interpreter and generated instruction dispatch. Other callers, ordinary
  reachability, native attribution beyond the observed context and broader
  private-ROM regressions remain pending. Current Wine `cpyhx1cy` passes all
  31 non-ignored synthetic ROM-data tests; 451 ignored tests were not run.
- [x] **ROMDATA-01-CONTEXT-3710D4-DIRECT-NATIVE scoped:** Wine `ax_5va8t`
  attributes all four context-3 requests and four typed bytes on the recorded
  world-map route to direct generated API calls. The 41-frame raw/managed pair
  matches state, clock, video, OAM, dispatch and audio with altered source bytes.
  Other contexts, callers and the full reader ABI remain open.
  [Evidence](../rom_data_indexed_graphics.md).
- [ ] **ROMDATA-01-SUBSTITUTION-371158:** establish remaining callback behavior,
  other parser-to-callback paths, ordinary text use and unselected consumers. EUR e160
  typed candidate slots and dispatcher prefixes pass under Wine (`2bltt60c`);
  callbacks 0/3 and their truncated aliases now execute fully with controlled
  RAM inputs (`rdulyaqw`). One real EUR program reaches callback 0 and returns
  through the original handler in a controlled task context (`h9c0peco`).
  Production CLI passes (`mgaon9d6`). A cold-origin source join (`gk6kl05w`)
  identifies a conditional message-119 callback-0 candidate in the recorded
  home continuation. Wine locator `zonc4h0q` reads seven text programs but
  zero callback bytes over 1,800 frames; no scene pair ran. This negative is
  limited to that window. Other ordinary use/callbacks, broader Wine
  regressions, Linux, USA and native Windows remain open.
- [ ] **ROMDATA-01-TEXT-READER-REGISTRY-371068:** audit the separately rooted
  text-reader callback registry immediately before `3710d4` against the current
  E168 manifest, establish its conditional entry extent and consumers, and
  integrate it if the source contract is supported. Static audit `mj5buj5x`
  authenticates 27 callback words and two literals without E168 overlap;
  original indexed reads precede later switch guards, so no guest bound is
  implied. E169 publication and 12 controlled interpreter prefixes pass Wine
  (`j15jcf44`), preserving all 111 earlier resources and checking mixed-owner
  boundaries. Production Wine CLI `mr5vn2od` passes creation/reuse/verify and
  a recorded 1,800-frame home continuation with 28 managed callback bytes,
  matching framebuffer/OAM hashes and final state. E170 Wine `8672kdim` compares
  all 1,800 frames of full state, clock, video, OAM and nonempty audio with 116
  source bytes altered. Seven typed table reads address only null slot 3; no
  callback invocation is established. Kind 11 conditionally selects `08016c00`,
  but its ordinary use remains unproved. Other scenes, direct native attribution,
  Linux, USA and native Windows remain open.
- [x] **ROMDATA-01-TEXT-READER-COMPLETE-PREFIX-DOMAIN:** both original reader
  prefixes match raw state and clock over all 256 byte indices and `0x100`
  aliases in interpreter and generated instruction dispatch: 2,048 E170 Wine
  cases, each word served exactly once from typed resources with all table
  backing bytes inverted (`u43_buss`). Prefixes stop before callback invocation;
  ordinary indices, callback semantics and other consumers remain open, and
  generated blocks may fall back to interpretation. [Evidence](../rom_data_text.md).
- [x] **ROMDATA-01-TEXT-READER-FULL-BYTE-DOMAIN:** audit physical ownership
  for the original registry readers' full byte-index address range against
  E169, preserving independently rooted neighboring data and distinguishing
  conditional addresses from ordinary index reachability. Static audit
  `nw66w3v6` finds 158 assigned and 98 unowned words, all in `371158..37140c`;
  93 are zero and five resemble Thumb ROM pointers. Source audit `xf27x8sa`
  authenticates their complement to the 75 earlier words and dispatcher
  literal. E170 controlled Wine test `sjojlovh` publishes these as a separate
  projection and matches 66 original interpreter/generated prefixes. E170
  recheck `thuu27et` assigns all 256 conditional registry-reader words and
  preserves the previous 158 owners. Production Wine CLI `alfjrf8b` passes
  create/reuse/verify; its three-frame scene reads zero new words. Callback
  semantics, ordinary indices and other dispatcher domains remain open. No
  guest bound is imposed. The stated ownership audit is complete; the remaining
  execution and semantic questions belong to the separate registry and
  substitution tasks above. [Evidence](../changes/2026-11-17-audit-european-text-reader-e170-ownership.json).
- [ ] **ROMDATA-01-RANGE-37140C:** establish ordinary use and remaining reader
  contexts. EUR e161 publication, typed bytes and original scan pass under Wine
  (`t2mh2ei8`); production CLI passes (`emoffrjv`). E170 caller census finds
  two Thumb wrapper calls and 41 candidate upstream calls (`yeizv0zb`,
  `bpuinw0u`, `tq88gscc`); no direct ARM BL candidate (`odbnk6j8`) and no
  matching candidate pointer in the scoped checkpoint scan (`50n30lt7`).
  Code execution, indirect calls and other entries remain unproved. Three
  distinct E170 Wine locators read zero source bytes: Giza battle entry
  (`dpeuvczs`), Royal Valley Mission (`hn5vy0hm`), Bank entry (`bfqintcy`).
  Do not repeat these windows. A source-guided Bank Select scene reads 23 bytes
  across five records (`5n6dsl9_`); the 120-frame full state/clock/video/OAM/
  audio pair with altered sources passes (`acebndyu`); interpreter attribution
  confirms `05be8a → 01abfc → 01abc0` (`hbu4s4d0`). Mixed native/interpreter
  frame clocks differ by 10 cycles with a legacy cache (`6s36aeat`,
  `je_s2es2`, `7tgvy6tq`); the retained guarded translation matches interpreter
  state/clock for that frame (`u_28pjng`) and passes the 120-frame managed pair
  (`jyionxk1`). Both direct wrapper prefixes pass 92 controlled cases across
  all range bounds and an unmatched input (`xfsk4jgx`). Literal census
  `_s7xtdr9` finds only the reader's aligned source pointer; full-ROM exact
  byte-offset census `l4aomd4j` finds no unaligned pointer into the range on
  mirrors 08/0a/0c. Computed or differently encoded pointers remain open.
  E170 mixed-owner
  boundary regression `pbj0c8fy` passes 852 typed reads over three mirrors.
  E170 lifecycle `p55pua3g` verifies failed replacement, slot Load/Undo and
  all 88 range bytes through a retained or freshly attached owner. Direct
  backup restoration and restart with an unavailable copy also pass for this
  owner (`tg0_or33`); these synthetic checks do not cover GUI or Fast.
  Both text-registry prefixes pass all 22 conditional range indices, neighbors
  and high-byte aliases in 192 controlled cases (`44qwhqj_`); the substitution
  dispatcher passes the corresponding 96 cases (`knxnixuk`). The indexed
  context getter passes its 96 range-overlap and neighbor cases (`3lrpx1l2`).
  An isolated portable call to the production cache selector cold-builds and
  reuses the same guarded object, and its 120-frame Bank pair matches. Wine
  unit `kitubxih` remains FAIL because a private ROM copy was retained; its
  zero-exit results and unchanged inputs are verified by read-only audit
  `s_g0xk37`. Actual GUI startup cold-builds and loads the same object under
  Wine/Xvfb (`wamu6k3z`, audit `p7tuq7iz`), but its replay fails before any
  verified frame because no GPU presenter is available. A later normal GUI
  interpreter pair `t6rwu_qk` executes the Bank input for 120 frames and
  matches state/clock at four endpoints, with 23 managed range bytes and
  normal close. The native-dispatch GUI pair `g3fnb1x5` matches the same
  endpoints and reads with the guarded cache. Neither compares video/audio
  per frame; native dispatch may fall back to interpretation. A traced GUI
  Fast continuation `a6u0qv7x` matches the raw reference after 12 neutral
  frames, eight after the toggle; aggregate range reads cannot be attributed
  specifically to Fast. A later GUI run `hr3m809u` enables Fast before any
  guest frame and observes 23 range bytes during 48 traced frames; an exact
  input-matched raw reference has equal initial/final state and clock. It
  does not compare video/audio per frame or alter ROM bytes.
  Nearby aligned-pointer
  census `wdb_l7ok` and disassembly `7zra4rc1`/`rm4e5mhp` bound the shown
  additional byte-index and geometry reads before the range in their normal
  paths. The four words without scoped PC-relative candidates lie in one
  E170 PCM wave and three BG graphics sources, with no exact ROM pointer to
  their bytes in the scoped mirror audit (`eqni5tiq`); other address
  constructions remain unexamined. Other computed
  consumers, ordinary indices, legacy IRQ equivalence and broader regressions
  remain open.
- [ ] **ROMDATA-01-ADJACENT-371464:** establish remaining readers, selector
  domains and broader Wine regression coverage. EUR e162 publishes
  six conditional geometry groups; the original-reader comparison passes
  (`5zpi8vam`) and its CLI scene has no positive ordinary read (`j4r9sopl`). A
  later E169 Wine home continuation reads 56 managed bytes across two aggregate
  groups and matches 1,800 framebuffer/OAM frames and final state (`mr5vn2od`,
  evidence audit `db6l43_0`). E170 Wine `lby31j3d` identifies state-width and
  state-height groups and compares full state, clock, video, OAM and nonempty
  audio per frame with geometry and special-pose index sources altered.
- [ ] **E134 ordinary updater:** establish production-cache entry availability,
  other ordinary paths and translated drawing. Current explicitly labeled
  translation has one-frame interpreter/native parity (`9ymu0twl`) and
  three-frame raw/managed updater-dispatch evidence (`yw62he9p`); captured
  dispatch may include nested work. An isolated no-extra-label build has none
  of the 43 observed updater symbol names (`qpbgtmmw`); its ordinary paired
  Wine frames pass without an observed native updater dispatch (`botzl4b6`),
  A later isolated portable test of the production selector chooses the same
  object (`kitubxih`, evidence audit `s_g0xk37`); the original Wine unit is
  FAIL due to a retained private ROM copy. The GUI also loads that object
  (`wamu6k3z`, audit `p7tuq7iz`), but its replay fails without a GPU
  presenter. The old artifact's failed
  comparison (`ve41xbr3`) is not explained; its later IRQ comparison crossed
  the reference frame boundary (`jui0l9oq`).
- [x] **E134-COMPLETE-FIRST-SEQUENCE-CONTROLLED:** for all 38 frames of the
  original first sequence, compare independent original and translated
  updater/draw calls and verify the selected list and duration. E170 Wine
  `b6uugr9g` passes full state/clock with altered managed backing, covering all
  19 lists used by the sequence. Continuous ordinary animation, the other two
  lists, production-cache coverage and other consumers remain open.
  [Evidence](../rom_data_obj.md#conditional-context-updater-obj-bank-e134).
- [x] **E134-ORDINARY-FIRST-SEQUENCE-SELECTIONS:** the retained EUR startup
  route reads all 38 selector/duration pairs, and one original updater context
  follows the modeled index/countdown sequence. E170 Wine `jusromci` matches
  state, clock, video, OAM, dispatch and five audio streams for 420 frames with
  bank backing altered. A separate raw interpreter observation `n455br6k`
  follows the caller into state 4 with 26 ticks still active, explaining why
  this startup instance does not complete its final hold. An ordinary instance
  that completes it, default-hook/default-cache translated drawing and other
  consumers remain open.
  [Evidence](../rom_data_obj.md#conditional-context-updater-obj-bank-e134).
- [x] **E134-ORDINARY-TRANSLATED-DRAW-PATCH-OFF:** on the recorded EUR startup
  continuation, Wine `an9unydr` attributes 133 direct generated-API header
  reads to the draw dispatch for all 19 sequence-selected lists. The 160-frame
  raw/managed pair matches state, clock, video, OAM, dispatch and audio with
  bank backing altered. This disables the sprite patch after checkpoint load
  and supplements nine observed draw entries; default production-cache and
  default-hook attribution, the final hold and other consumers remain open.
  [Evidence](../rom_data_obj.md#conditional-context-updater-obj-bank-e134).
- [ ] **ROMDATA-01-PARAMETER-RECORDS27-DYNAMIC-SOURCES:** resolve the live
  source domain, save-load behavior and aliases beyond the observed 59 rows.
- [ ] **ROMDATA-01-SCRIPT39-REGISTRY-POSES:** authenticate and model the
  original opcode-39 task producers and their conditional registry/actor/job
  joins, then connect any new disjoint sources to the EUR resource path and
  verify them under Wine. E162 contains 71 distinct high-registry selector/group
  pairs. The typed source model and controlled task join pass under Wine
  (`z2eftm1l`); e163 publication, typed reads and 12 controlled reader
  comparisons also pass (`ga87pps5`), as does the production CLI (`vgo1cewu`).
  Ordinary use and other producer domains remain open. Source witness `B97ACB` can conditionally select
  four null sprite-0 cells through normal job 2; this is not a guest index bound.
  See [the inventory](../rom_data_unit_obj.md#opcode-39-registry-pose-inventory).
- [ ] **ROMDATA-01-SCRIPT39-SPECIAL-POSES:** finish EUR low-selector
  special-unit ordinary/remaining reader coverage. The conditional
  E163 source plan (`stme4uml`) identifies 964 new index cells, 109 sequences,
  seven OBJ lists and 87 pixel blocks. The typed model and a controlled original
  unit-to-actor join pass under Wine (`aysnyiei`); e164 publication and six
  controlled reader comparisons pass (`jrp5fb0y`), as does the production CLI
  (`72_aby76`) without a positive ordinary read in that window. The later E169
  Wine home continuation reads 48 index bytes, zero sequence/OBJ/pixel bytes,
  and matches 1,800 framebuffer/OAM frames and final state (`mr5vn2od`, audit
  `db6l43_0`). E170 Wine `lby31j3d` identifies 12 null index cells and passes
  the full 1,800-frame state/clock/video/OAM/audio pair with index sources
  altered. Static join `pot_cegy` finds one modeled opcode-39 query per cell,
  grouped as selectors/groups 15/22, 16/4, 16/16, 17/6, 17/16 and 78/5.
  Wine `bxmz57hb` locates a read at relative frame 345; interpreter replay reads
  two cells in Thumb `080227a8` scope and matches native frame state, image and
  OAM. Follow-up `b56w86ji` observes caller `0809ae70`, sprite 48, animation
  65 and parts 1/2 in the interpreter. Wine `_uuybpg0` observes the upstream
  `0812a8c4` task → `0809d9c4` → `0809ae1c` chain, with animation base 64.
  Wine `v6ydjwg7` observes task selector 17, group 16, state 0 and flag 0.
  Wine `7hy1jqxu` observes opcode39 `b95917`, its matching creator and task
  entry in that frame. Wine `s4quofvj` and static join `1zjiaoiz` connect the
  allocator return and synchronous callback to the same task. Wine `lwnzoodb`
  observes typed reads of the matching body index, one-frame sequence, OBJ and
  pixels and passes a 1,800-frame state/clock/video/OAM/audio pair with those
  sources altered. Complete animation lifecycle and per-request task attribution
  remain unproved. Two
  excluded group-9 instructions (`B96A1E`, `BA0841`) call `0809BA84` with argument 13; their
  conditional index and downstream sources are already owned (`wnmiows2`,
  `4p7yr_3v`). The original task prefix selects the expected callees in three
  controlled Wine comparisons (`uto86fc6`). Two separately seeded callee
  continuations return with matching raw/managed state and guest clocks on Wine
  (`_mvdi9t3`). The uninterrupted allocator path, alternate-sprite behavior,
  actor overrides, action-13 ordinary reachability and other ordinary paths,
  broader consumers, Linux, USA and native Windows remain open.
- [ ] **ROMDATA-04-GAMEPLAY-6E84E0-BYTE00:** find a source-backed ordinary
  trigger for the unresolved byte, including generated/indirect sources.
  The bounded direct-dispatcher scan finds only two text-sourced candidates
  (`1quaoj7v`); RAM substitution includes callback table `371158` with live
  index/output unproved (`8bksdmpz`). Declared text yields 75 opcode-25
  candidate operands, four with zero table words (`t03ndh68`); execution is
  unproved. Callback 0 copies a RAM buffer with conditional short-string and
  live context+312 producers (`1o2klth5`, `ds4kthpq`). Valid keyboard-only
  sequences exclude the target bytes (`hlejcxyw`). State `0100` initializes
  the field through either `08016648(2)` or the distinct localized selector
  `08016828(0)` (`tdombygl`, `y865vpe4`). All five generated strings omit
  the target through the terminator (`dgzrlx3q`); on the exact unmodified
  field-to-callback path the aligned zero blocks trailing bytes (`m0m6rtgn`).
  Candidate callback 28 selects `08016768` or `08016828` (`nxd4bals`,
  `6zw8vl88`): its event-derived branch is within the catalog for all 256
  ordinals (`k7p3b_ey`). Two active-halfword stores are bounded to 0..511
  (`tcle99g9`); mode-2 input to another store is a byte conditional on
  intervening state (`_emiu3kp`, `kqba3yet`). Ten-bit values 512..1023
  select words outside primary ROM (`fnub_9js`); eight satisfy two necessary
  helper conditions (`76wmjx5f`, `7vncx8y0`), with execution/mapping
  unproved. One insertion constructs IDs 1..511 (`4ajfe58a`, `aizksk7l`);
  another has IDs 0..358 for its 31 authenticated rows (`m8am6hhs`). Its two
  direct candidates pass a byte index (`a4_woxfy`); outside catalog, 75
  values exceed 511 but none matches the eight helper candidates (`sli5ztgr`).
  Bounded immediate-store windows trace view+1d from view+1b and restrict
  specific source stores to 0..127 (`hrbvrx_w`, `t_vqnwj1`, `l4_ach_9`);
  26 indices 31..127 still yield IDs above 511. Two view+8c writers read
  live variable 80 (`acxbx3u4`); a dynamic setter candidate instead targets
  1b..32 (`7ic5mznx`). Two script setter opcodes have no instructions in the
  authenticated projections (`svjlcy7s`). Mission key writers have 27 owned
  matches, mode 3 values 1..30 (`jnkbjfyx`); examined direct helper paths
  exclude variable 80 (`pdzm4ife`, `q0h2f10_`, `dce65adz`, `st534xw1`).
  One dynamic-base store cannot overlap variable 80 under a successful,
  intact in-pool allocation, including EWRAM mirrors (`q9cpd9yg`); actual
  allocation state, outside rows, other writers and restoration remain open.
- [x] **ROMDATA-05-E170-ALL-RESOURCE-LIFECYCLE scoped:** synthetic EUR/Wine
  `i73r4f54` verifies all 10,948,338 declared source bytes with inverted ROM
  backing after failed replacement, Load, Undo, backup restore and restart.
  The owner and expected states persist; individual resource attribution, GUI,
  Fast, guest execution and other platforms remain outside this result.
  [Evidence](../rom_data_runtime.md).
- [ ] **ROMDATA-05:** complete required equivalence, saves, audio and rendering
  checks for each remaining migrated family and preserve guest timing and
  side effects; the four short E162 scene windows and cold boot do not establish
  full-scene, all-input, all-consumer or global equivalence. E170 Wine
  `yeg0bdt_` checks byte/halfword/word composition at all 74,225 distinct
  manifest source boundaries with inverted backing ROM and three mirrors;
  E170 Wine `ffhtpvms` additionally scans all 10,948,338 distinct declared
  source bytes through three mirrors and the native API and compares aligned,
  fully covered words with byte composition against inverted backing ROM.
  E170 Wine `8_1wpiix` additionally checks all fully covered unaligned
  halfword/word offsets through bus and native API. These are read regressions,
  not guest execution or full equivalence. The separate E170 Wine `9b1qo9ie` test matches all
  8,476 frames of one retained EUR suspend-to-battle route, including complete
  state, clock, video, OAM, dispatch and five audio streams; 42 script-command
  bytes from seven commands are read. ROM sources were not altered, and this
  one route does not close all-family or all-consumer equivalence.

## 5. Resolve prerequisites before selecting further proofs

- [x] **ROMDATA-01-SOIL-EVIDENCE-RAW scoped EUR/Wine:** `tfe8gnqy` captures
  E231 selected-reader addresses during 600 matching Soil Evidence frames;
  3,066 sampled bytes are outside declared sources. This does not attribute PCs.
  [Evidence](../rom_data_runtime.md#e260-soil-evidence-rom-read-inventory).
- [x] **ROMDATA-01-SOIL-EVIDENCE-LZ-PLAN scoped static:** `frkft5lf`
  authenticates three complete sampled LZ77 streams: 1,335 encoded bytes and
  1,640 decoded bytes. The plan publishes no owner.
  [Evidence](../rom_data_graphics.md#e261-soil-evidence-lz77-source-plan).
- [x] **ROMDATA-04-SOIL-EVIDENCE-LZ scoped EUR/Wine:** E232 serves the three
  E261 streams. Wine reader `e262-soil-lz-readers-wine-v7ya7ht6`, scene `e262-soil-lz-scene-wine-knhlea70`, CLI `e262-soil-lz-cli-wine-6ld8ewbe` and
  sample join `e262-soil-lz-sample-residual-twyv42n7` pass. [Evidence](../rom_data_graphics.md#e262-soil-evidence-lz77-runtime-sources).
- [ ] **ROMDATA-01-SOIL-EVIDENCE-RESIDUAL:** attribute and integrate the
  E260 sampled bytes (zero outside E248 owners after E301); attribution, other scenes and full coverage remain open. E272 classifies
  112 lower and 373 upper bytes, all sampled on the Bus. E263 classifies address/width
  groups and finds 151 possible lower Thumb literal words. E264 observes
  matching instruction-scoped PC-relative loads for 135 distinct words, now
  served by E266. E267 groups 58 remaining lower words; E269 serves 30, leaving 28 at that
  stage. E284 leaves no sampled bytes below `300000`; other upper sources remain open. [Evidence](../rom_data_runtime.md#e267-soil-evidence-remaining-lower-reader-groups).
- [x] **ROMDATA-01-SOIL-UPPER-READER-SITES scoped EUR/Wine:** E285 captures
  all 285 E240 residual bytes; 245 are covered by interpreter-scoped groups
  and 40 are outside those scopes. [Evidence](../rom_data_runtime.md#e285-soil-evidence-upper-residual-reader-sites).
- [x] **ROMDATA-01-SOIL-12BYTE-RECORDS scoped EUR/Wine/static:** E286
  authenticates six selected 12-byte-shaped rows and original stride/branches.
  [Evidence](../rom_data_runtime.md#e286-soil-evidence-twelve-byte-record-plan).
- [x] **ROMDATA-01-SOIL-TYPE3-LINKS scoped EUR/Wine/static:** E287 joins four
  rows, four descriptors and four LZ wrappers to already owned streams; one
  wrapper is shared. [Evidence](../rom_data_runtime.md#e287-soil-evidence-type-3-source-links).
- [x] **ROMDATA-04-SOIL-12BYTE-RECORDS scoped EUR/Wine:** E241 serves the four
  E287 type-3 rows and links without duplicate wrapper/stream ownership. Reader
  `e288-soil-transfer-readers-wine-3f10sxyc`, scene `e288-soil-transfer-scene-wine-ixp_q9mt`, CLI `e288-soil-transfer-cli-wine-ztjvp9z5`, sample join `e288-soil-transfer-sample-residual-tn85nxvm` and prior
  copied-halfword regression `e288-soil-copied-regression-wine-5_4m68dh` pass. [Evidence](../rom_data_runtime.md#e288-soil-evidence-type-3-transfer-runtime).
- [ ] **ROMDATA-01-SOIL-OTHER-TRANSFER-TYPES:** establish remaining type-2 and
  type-4 consumers, complete guest domains and the unclassified `af2716..af2718`
  gap; no relationship to the adjacent tilemap is inferred.
- [x] **ROMDATA-01-SOIL-TYPES24-SCOPED-PLAN EUR/Wine/static:** E289 authenticates
  104 selected source bytes across both original reader paths, including 78
  sampled bytes. [Evidence](../rom_data_runtime.md#e289-soil-evidence-type-2-and-type-4-source-plan).
- [x] **ROMDATA-04-SOIL-TYPES24-OBSERVED scoped EUR/Wine:** E242 serves the
  E289 selected paths without owning the gap or duplicating the adjacent
  tilemap. Reader `e290-soil-types24-readers-wine-oan65r_2`, scene `e290-soil-types24-scene-wine-lrcwewjg`, CLI `e290-soil-types24-cli-wine-okzybzbi` and sample join
  `e290-soil-types24-sample-residual-tmqi3aq8` pass. [Evidence](../rom_data_runtime.md#e290-soil-evidence-type-2-and-type-4-runtime-sources).
- [ ] **ROMDATA-04-SOIL-TYPES24-LINUX:** validate selected readers, scene and
  CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-TYPES24-USA:** identify and validate corresponding USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E242-WINE:** rerun other updated revision-sensitive
  Wine fixtures when selected; their earlier evidence remains valid.
- [ ] **ROMDATA-04-SOIL-TRANSFER-OTHER:** validate other callers/scenes, real
  BIOS and UI save/Load; guest index domains remain unbounded.
- [ ] **ROMDATA-04-SOIL-TRANSFER-LINUX:** validate E241 readers, selected
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-TRANSFER-USA:** identify and validate corresponding
  USA sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E241-WINE:** rerun other updated revision-sensitive
  Wine fixtures when selected; only the prior copied-halfword fixture was rerun.
- [x] **ROMDATA-01-SOIL-39E1-READERS scoped EUR/Wine/static:** E291 identifies
  five four-byte range records and eleven four-byte sequence records, including
  30 sampled bytes. [Evidence](../rom_data_runtime.md#e291-soil-evidence-indexed-four-byte-record-plan).
- [x] **ROMDATA-04-SOIL-39E1-RECORDS scoped EUR/Wine:** E243 serves all 64
  E291 source bytes with existing literal and adjacent owners preserved. Wine
  reader `e292-soil-scene-records-readers-wine-3ob0bs_v`, scene `e292-soil-scene-records-scene-wine-vg1kiwv5`, CLI `e292-soil-scene-records-cli-wine-xhag7oue` and
  static join `e292-soil-scene-records-sample-residual-xzombbo8` pass. [Evidence](../rom_data_runtime.md#e292-soil-evidence-indexed-scene-record-runtime).
- [ ] **ROMDATA-04-SOIL-39E1-OTHER:** establish other callers, initial state,
  guest domains and scenes; validate real BIOS and UI save/Load.
- [ ] **ROMDATA-04-SOIL-39E1-LINUX:** validate selected readers, scene and CLI
  on Linux when selected.
- [ ] **ROMDATA-04-SOIL-39E1-USA:** identify and validate corresponding USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E243-WINE:** rerun the 35 updated older revision-
  sensitive fixtures when selected; their earlier evidence remains valid.
- [x] **ROMDATA-01-SOIL-UPPER-INDIRECT scoped EUR/Wine/static:** E293
  authenticates five eight-byte records, including 35 sampled bytes, and
  compatible original reader/initializer links. [Evidence](../rom_data_runtime.md#e293-soil-evidence-indirect-eight-byte-record-plan).
- [x] **ROMDATA-04-SOIL-UPPER-INDIRECT scoped EUR/Wine:** E244 serves all 40
  E293 source bytes with existing literal and adjacent owners preserved. Wine
  reader `e294-soil-indirect-records-readers-wine-cp1uz_wj`, scene `e294-soil-indirect-records-scene-wine-y9ussbh4`, CLI `e294-soil-indirect-records-cli-wine-ahnerpan` and
  static join `e294-soil-indirect-records-sample-residual-vrbl28sb` pass. [Evidence](../rom_data_runtime.md#e294-soil-evidence-indirect-record-runtime).
- [ ] **ROMDATA-04-SOIL-INDIRECT-OTHER:** establish sampled caller ancestry,
  other callers, guest domains and scenes; validate real BIOS and UI save/Load.
- [ ] **ROMDATA-04-SOIL-INDIRECT-LINUX:** validate selected readers, scene and
  CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-INDIRECT-USA:** identify and validate corresponding USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E244-WINE:** rerun the 36 updated older revision-
  sensitive fixtures when selected; their earlier evidence remains valid.
- [x] **ROMDATA-04-SOIL-INDEXED-BYTES scoped EUR/Wine:** E245 serves the six
  witnessed bytes after the E244 indirect record. Wine reader `e295-soil-indexed-bytes-readers-wine-eb4g2qel`,
  scene `e295-soil-indexed-bytes-scene-wine-_fgc_4sp`, CLI `e295-soil-indexed-bytes-cli-wine-ttlz5pdo` and static join `e295-soil-indexed-bytes-sample-residual-hgpyj2zs`
  pass. [Evidence](../rom_data_runtime.md#e295-soil-evidence-indexed-byte-runtime).
- [ ] **ROMDATA-04-SOIL-INDEXED-BYTES-OTHER:** establish full table and index
  domain, other callers/scenes and real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-INDEXED-BYTES-LINUX:** validate selected reader, scene
  and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-INDEXED-BYTES-USA:** identify and validate corresponding
  USA sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E245-WINE:** rerun the 37 updated older revision-
  sensitive fixtures, including the relocated fallback probe, when selected.
- [x] **ROMDATA-01-SOIL-COMPACT-RECORD scoped EUR/Wine/static:** E296
  authenticates a 24-byte Soil compact record and one indexed pointer word,
  including 24 sampled bytes. [Evidence](../rom_data_compact_catalog.md#e296-soil-evidence-compact-record-plan).
- [x] **ROMDATA-04-SOIL-COMPACT-RECORD scoped EUR/Wine:** E246 serves the
  E296 record and observed pointer with prior bank/literal owners. Wine reader
  `e297-soil-compact-record-readers-wine-5w49a8bi`, scene `e297-soil-compact-record-scene-wine-1ti7yhlx`, CLI `e297-soil-compact-record-cli-wine-fr1p_15v` and static join
  `e297-soil-compact-record-sample-residual-_hg2aqh0` pass. [Evidence](../rom_data_compact_catalog.md#e297-soil-evidence-compact-record-runtime).
- [ ] **ROMDATA-04-SOIL-COMPACT-RECORD-OTHER:** establish sampled caller
  ancestry, complete pointer table, other callers/scenes and real BIOS and
  UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-COMPACT-RECORD-LINUX:** validate readers, scene and
  CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-COMPACT-RECORD-USA:** identify and validate the USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E246-WINE:** rerun the 38 updated older revision-
  sensitive fixtures when selected; earlier evidence remains valid.
- [x] **ROMDATA-01-SOIL-REMAINING-SCOPED scoped EUR/Wine/static:** E298
  authenticates six individually observed indexed values, 16 sampled bytes,
  and their existing literal roots. [Evidence](../rom_data_runtime.md#e298-soil-evidence-scoped-indexed-values-plan).
- [x] **ROMDATA-04-SOIL-SCOPED-VALUES scoped EUR/Wine:** E247 serves all 16
  E298 bytes with prior literal owners and fallback. Wine reader `e299-soil-scoped-values-readers-wine-4zlbm5q6`,
  scene `e299-soil-scoped-values-scene-wine-qeu3cp12`, CLI `e299-soil-scoped-values-cli-wine-r4c7x_zi` and static join `e299-soil-scoped-values-sample-residual-248p0qae`
  pass. [Evidence](../rom_data_runtime.md#e299-soil-evidence-scoped-indexed-values-runtime).
- [ ] **ROMDATA-04-SOIL-SCOPED-VALUES-OTHER:** establish complete tables, guest
  domains, actual callback invocation and ancestry, other callers/scenes,
  real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-SCOPED-VALUES-LINUX:** validate selected readers,
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-SCOPED-VALUES-USA:** identify and validate corresponding
  USA sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E247-WINE:** rerun the 39 updated older revision-
  sensitive fixtures when selected; earlier evidence remains valid.
- [ ] **ROMDATA-01-SOIL-UPPER-STRUCTURES:** establish original reader/source
  formats for the scoped `39a` and `39e` page clusters and integrate only
  authenticated, nonoverlapping extents.
- [x] **ROMDATA-01-SOIL-REMAINING-RECTANGLES scoped EUR/Wine/static:** E300
  authenticates two rectangles and one alternative literal, including all 22
  E247 residual sample bytes. [Evidence](../rom_data_graphics.md#e300-soil-evidence-remaining-rectangle-plan).
- [x] **ROMDATA-04-SOIL-REMAINING-RECTANGLES scoped EUR/Wine:** E248 serves
  two rectangles and their alternate literal. Wine reader `e301-soil-remaining-rectangles-readers-wine-ibf9tm98`,
  scene `e301-soil-remaining-rectangles-scene-wine-1o7mbmv_`, CLI `e301-soil-remaining-rectangles-cli-wine-vcu4hnwd` and static join `e301-soil-remaining-rectangles-sample-residual-a7umgmlt`
  pass. [Evidence](../rom_data_graphics.md#e301-soil-evidence-remaining-rectangle-runtime).
- [ ] **ROMDATA-04-SOIL-REMAINING-RECTANGLES-OTHER:** establish sampled pointer
  ancestry, other callers/scenes and real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-REMAINING-RECTANGLES-LINUX:** validate readers, scene
  and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-REMAINING-RECTANGLES-USA:** identify and validate
  corresponding USA sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E248-WINE:** rerun the 40 updated older revision-
  sensitive fixtures when selected; earlier evidence remains valid.
- [x] **ROMDATA-01-FIRAGA-SCENE-INVENTORY scoped EUR/Wine:** E302 compares
  606 recorded battle frames and inventories selected readers against E248;
  8,283 distinct observed bytes are outside its source spans.
  [Evidence](../rom_data_gameplay.md#e302-european-firaga-scene-rom-read-inventory).
- [x] **ROMDATA-01-FIRAGA-LZ-PLAN scoped static:** E303 authenticates three
  complete streams covering 3,290 E302 sampled bytes and 5,968 decoded bytes.
  [Evidence](../rom_data_graphics.md#e303-firaga-battle-lz77-source-plan).
- [x] **ROMDATA-04-FIRAGA-LZ scoped EUR/Wine:** E249 publishes the three E303
  streams. Wine readers `6taiimnv`, scene `ugltru8s`, CLI `a19gxpua` and
  static join `higcopsd` pass. [Evidence](../rom_data_graphics.md#e304-firaga-battle-lz77-runtime-sources).
- [ ] **ROMDATA-04-FIRAGA-LZ-OTHER:** establish game-level asset identities,
  other callers/scenes and real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-FIRAGA-LZ-LINUX:** validate publication, readers, transfers,
  affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-FIRAGA-LZ-USA:** identify and validate corresponding USA
  sources when selected; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E249-WINE:** rerun the 41 revision-sensitive older
  fixtures updated for E249 and check their reports when selected. The older
  E174/E175 observed-LZ fixtures also need current-revision adaptation before
  a rerun; earlier accepted results remain valid.
- [x] **ROMDATA-01-FIRAGA-SPRITE-PLAN scoped static:** E305 finds 44 fully
  sampled, disjoint count/payload candidates covering 2,170 bytes.
  [Evidence](../rom_data_runtime.md#e305-firaga-spritepeek-list-candidates).
- [x] **ROMDATA-04-FIRAGA-SPRITE-LISTS scoped EUR/Wine:** E250 serves all
  44 E305 lists. Wine reader `zlctezch`, scene `pa5aamwj`, CLI `h5s9hkgs`
  and static join `qgpr0gm3` pass. [Evidence](../rom_data_runtime.md#e306-firaga-typed-sprite-lists).
- [ ] **ROMDATA-04-FIRAGA-SPRITE-LISTS-OTHER:** establish pointer producers,
  other callers/scenes and real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-FIRAGA-SPRITE-LISTS-LINUX:** validate publication, guest and
  host reads, transfers, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-FIRAGA-SPRITE-LISTS-USA:** identify and validate corresponding
  USA sources when selected; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E250-WINE:** rerun the 42 revision-sensitive older
  fixtures updated for E250 and check their reports when selected; older
  revision-pinned fixtures still need adaptation before rerunning.
- [x] **ROMDATA-01-FIRAGA-3B9E80-SOURCE scoped EUR/static:** E307 authenticates
  two adjacent copied matrices and four literal words, covering 472 sampled
  bytes outside E250 sources. [Evidence](../rom_data_graphics.md#e307-firaga-copied-matrix-source-plan).
- [x] **ROMDATA-04-FIRAGA-COPIED-MATRICES scoped EUR/Wine:** E251 publishes
  both matrices and four literal words; reader `43bpfeqh`, scene `_e603cr2`,
  CLI `e308-firaga-matrices-cli-wine-73v6pnom` and join `q5otx2ur` pass.
  [Evidence](../rom_data_graphics.md#e308-firaga-copied-matrix-runtime).
- [ ] **ROMDATA-04-FIRAGA-COPIED-MATRICES-OTHER:** establish the dynamic producer,
  semantics, other readers/scenes, real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-FIRAGA-COPIED-MATRICES-LINUX:** validate publication,
  original readers, transfers, the affected scene and CLI on Linux.
- [ ] **ROMDATA-04-FIRAGA-COPIED-MATRICES-USA:** identify and validate
  corresponding USA sources; native Windows also remains unverified.
- [ ] **ROMDATA-REGRESSION-E251-WINE:** rerun the 43 updated older
  revision-sensitive fixtures when selected; older pinned fixtures need
  adaptation before rerunning. Earlier accepted evidence remains valid.
- [x] **ROMDATA-01-FIRAGA-RESIDUAL-SOURCE scoped EUR/static:** E309 authenticates
  626 + 22 sampled bytes of compact cell data and the associated original
  reader paths. [Evidence](../rom_data_graphics.md#e309-firaga-compact-cell-source-plan).
- [x] **ROMDATA-04-FIRAGA-COMPACT-CELLS scoped EUR/Wine:** E252 serves the E309
  frames, sequence, pointer cell and caller literal. Wine reader `5kbmgbk2`,
  scene `e310-firaga-compact-scene-wine-q89549qz`, CLI `e310-firaga-compact-cli-wine-0ha_kvmp` and join `5soh9_aa` pass.
  [Evidence](../rom_data_graphics.md#e310-firaga-compact-cell-runtime).
- [ ] **ROMDATA-REGRESSION-E252-WINE:** rerun the 44 updated older revision-
  sensitive fixtures when selected; older pinned fixtures need adaptation.
- [ ] **ROMDATA-04-FIRAGA-COMPACT-CELLS-OTHER:** establish other selectors,
  field meanings, scenes, real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-FIRAGA-COMPACT-CELLS-LINUX:** validate publication, readers,
  affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-FIRAGA-COMPACT-CELLS-USA:** identify and validate corresponding
  USA sources when selected; native Windows also remains unverified.
- [x] **ROMDATA-01-FIRAGA-412050-SOURCE scoped EUR/static:** E311 authenticates
  176 candidate bytes, 172 sampled, and the original indexed OBJ readers.
  [Evidence](../rom_data_obj.md#e311-firaga-indexed-obj-source-plan).
- [x] **ROMDATA-04-FIRAGA-OBJ-BANK scoped EUR/Wine:** E253 serves the E311
  descriptor, sequence and directory, reusing 18 lists and adding the empty
  list to guest/host ownership. Reader `za6f9c7v`, scene `e312-firaga-obj-scene-wine-3d6gbyaw`, CLI `e312-firaga-obj-cli-wine-qe3bcmsl`
  and join `0hdjkda7` pass.
  [Evidence](../rom_data_obj.md#e312-firaga-indexed-obj-runtime).
- [ ] **ROMDATA-REGRESSION-E253-WINE:** rerun the 45 updated older revision-
  sensitive fixtures when selected; older pinned fixtures need adaptation.
- [ ] **ROMDATA-04-FIRAGA-OBJ-BANK-OTHER:** establish producer ancestry, other
  selectors/scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-OBJ-BANK-LINUX:** validate publication, guest/host
  readers, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-FIRAGA-OBJ-BANK-USA:** identify and validate corresponding
  USA sources when selected; native Windows also remains unverified.
- [x] **ROMDATA-01-FIRAGA-6B368E-SOURCE scoped EUR/static:** E313 authenticates
  a compact OBJ bank, three empty lists and selected indirect roots covering
  66 sampled bytes. [Evidence](../rom_data_obj.md#e313-firaga-compact-obj-source-plan).
- [x] **ROMDATA-04-FIRAGA-COMPACT-OBJ-BANK scoped EUR/Wine:** E254 serves the
  E313 compact bank and roots, reusing seven lists and adding three empty
  lists to guest/host ownership. Reader `e314-firaga-compact-obj-readers-wine-uag8crl9`, scene `e314-firaga-compact-obj-scene-wine-00m4jnor`, CLI
  `e314-firaga-compact-obj-cli-wine-dlvxuop3` and join `e314-firaga-compact-obj-sample-residual-h1qk2fn_` pass.
  [Evidence](../rom_data_obj.md#e314-firaga-compact-obj-runtime).
- [ ] **ROMDATA-REGRESSION-E254-WINE:** rerun the 46 updated older revision-
  sensitive fixtures when selected; older pinned fixtures need adaptation.
- [ ] **ROMDATA-04-FIRAGA-COMPACT-OBJ-BANK-OTHER:** establish descriptor and
  producer ancestry, other selectors/scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-COMPACT-OBJ-BANK-LINUX:** validate publication,
  guest/host readers, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-FIRAGA-COMPACT-OBJ-BANK-USA:** identify and validate
  corresponding USA sources; native Windows also remains unverified.
- [x] **ROMDATA-04-FIRAGA-ADJACENT-COMPACT-OBJ-SOURCES scoped EUR/static:**
  E315 authenticates 115 source bytes, 107 sampled, two descriptors, a
  17-frame sequence, ten prior lists and one empty list.
  [Evidence](../rom_data_obj.md#e315-firaga-adjacent-compact-obj-source-plan).
- [x] **ROMDATA-04-FIRAGA-ADJACENT-COMPACT-OBJ-BANK scoped EUR/Wine:** E255
  serves the selected descriptors, count, compact sequence and empty list,
  retaining earlier owners. Reader `e316-firaga-adjacent-obj-readers-wine-5zq084xm`, scene `e316-firaga-adjacent-obj-scene-wine-vgv_p24h`, CLI `e316-firaga-adjacent-obj-cli-wine-823mmm5g`
  and join `e316-firaga-adjacent-obj-sample-residual-rk9t89t2` pass.
  [Evidence](../rom_data_obj.md#e316-firaga-adjacent-compact-obj-runtime).
- [ ] **ROMDATA-REGRESSION-E255-WINE:** rerun the 47 updated older revision-
  sensitive fixtures when selected; older pinned fixtures need adaptation.
- [ ] **ROMDATA-04-FIRAGA-ADJACENT-COMPACT-OBJ-OTHER:** establish remaining
  selectors/sequences, producer ancestry, other scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-ADJACENT-COMPACT-OBJ-LINUX:** validate publication,
  readers, selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-ADJACENT-COMPACT-OBJ-USA:** identify and validate
  corresponding sources; native Windows remains unverified.
- [x] **ROMDATA-04-FIRAGA-6C12C8-SOURCES scoped EUR/static:** E317
  authenticates 85 source bytes, 81 sampled, a 13-frame sequence, nine
  prior lists and one empty list.
  [Evidence](../rom_data_obj.md#e317-firaga-6c12c8-compact-obj-source-plan).
- [x] **ROMDATA-04-FIRAGA-6C12C8-BANK scoped EUR/Wine:** E256 serves the
  selected count, pointer, descriptor, compact sequence and empty list,
  retaining earlier owners. Reader `e318-firaga-6c12c8-obj-readers-wine-hjwi7j_n`, scene `e318-firaga-6c12c8-obj-scene-wine-vav9bzv2`, CLI `e318-firaga-6c12c8-obj-cli-wine-maq3te3q`
  and join `e318-firaga-6c12c8-obj-sample-residual-loxk_hkh` pass.
  [Evidence](../rom_data_obj.md#e318-firaga-6c12c8-compact-obj-runtime).
- [ ] **ROMDATA-REGRESSION-E256-WINE:** rerun the 48 updated older revision-
  sensitive fixtures when selected; older pinned fixtures need adaptation.
- [ ] **ROMDATA-04-FIRAGA-6C12C8-OTHER:** establish other selectors, pointers,
  producers and scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-6C12C8-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-6C12C8-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-04-FIRAGA-EB9C0-LITERALS scoped EUR/Wine:** E257 adds seven
  source-backed original Thumb LDR operands to the existing literal owner.
  Reader `e319-firaga-eb9c0-literals-readers-wine-ej_mhq65`, scene `e319-firaga-eb9c0-literals-scene-wine-g9opsvws`, CLI `e319-firaga-eb9c0-literals-cli-wine-kq1iltos` and join `e319-firaga-eb9c0-literals-sample-residual-r8fvlc_h` pass.
  [Evidence](../rom_data_runtime.md#e319-firaga-observed-thumb-literals).
- [ ] **ROMDATA-REGRESSION-E257-WINE:** adapt and rerun affected older
  revision-sensitive fixtures, including unchanged-literal assumptions, when
  selected; 49 updated fixtures have not been rerun.
- [ ] **ROMDATA-04-FIRAGA-EB9C0-OTHER:** establish other readers, tables,
  scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-EB9C0-LINUX:** validate readers, selected scene and
  CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-EB9C0-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-39CB9C-SOURCES scoped EUR/static:** E320 authenticates
  four transfer records, their roots and wrappers, and three previously owned
  LZ streams. [Evidence](../rom_data_runtime.md#e320-firaga-transfer-program-source-plan).
- [x] **ROMDATA-04-FIRAGA-39CB9C-PROGRAM scoped EUR/Wine:** E258 publishes
  typed controls and reuses the existing LZ streams. Reader `e321-firaga-transfer-program-readers-wine-tfkrk5a_`, scene
  `e321-firaga-transfer-program-scene-wine-sh2x442o`, CLI `e321-firaga-transfer-program-cli-wine-hj1_inpx` and join `e321-firaga-transfer-program-sample-residual-h_gw4p3y` pass.
  [Evidence](../rom_data_runtime.md#e321-firaga-transfer-program-runtime).
- [ ] **ROMDATA-REGRESSION-E258-WINE:** adapt and rerun affected older
  revision-sensitive fixtures for E258, including E257 literal assumptions,
  when selected.
- [x] **ROMDATA-01-FIRAGA-39CB9C-PALETTE-ROW-SOURCES scoped EUR/static:** E322
  proves two adjacent eight-byte row prefixes, 14 bytes sampled. [Evidence](../rom_data_graphics.md#e322-firaga-palette-row-source-plan).
- [x] **ROMDATA-04-FIRAGA-39CB9C-PALETTE-ROWS scoped EUR/Wine:** E259 serves
  the selected prefixes. Reader `e323-firaga-palette-rows-readers-wine-rqv2iq3y`, scene `e323-firaga-palette-rows-scene-wine-05hjmqhm`, CLI `e323-firaga-palette-rows-cli-wine-7y0zezfv`
  and join `e323-firaga-palette-rows-sample-residual-uui0fp3b` pass.
  [Evidence](../rom_data_graphics.md#e323-firaga-palette-row-runtime).
- [ ] **ROMDATA-REGRESSION-E259-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-01-FIRAGA-39CB9C-PALETTES:** establish other consumers and
  selected or full use of the additional rows declared at `ae6db8`; the E322
  source plan covers only the first row of each selected palette.
- [ ] **ROMDATA-04-FIRAGA-39CB9C-OTHER:** establish other selectors, producers,
  consumers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-39CB9C-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-39CB9C-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-14D480-SOURCES scoped EUR/static:** E324 authenticates
  five original literals and 44 bytes copied to the stack; all 64 source bytes
  appear in E302. [Evidence](../rom_data_runtime.md#e324-firaga-stack-initialization-source-plan).
- [x] **ROMDATA-04-FIRAGA-14D480-STACK scoped EUR/Wine:** E260 publishes
  typed stack data and literals. Reader `e325-firaga-stack-initialization-readers-wine-g2cv2hbo`, scene `e325-firaga-stack-initialization-scene-wine-w0ru_ree`,
  CLI `e325-firaga-stack-initialization-cli-wine-_t8_0ngi` and join `e325-firaga-stack-initialization-sample-residual-smedangh` pass.
  [Evidence](../rom_data_runtime.md#e325-firaga-stack-initialization-runtime).
- [ ] **ROMDATA-REGRESSION-E260-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-14D480-OTHER:** establish live selector meaning and
  domain, other callers and scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-14D480-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-14D480-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-40E844-SOURCES scoped EUR/Wine:** E326 identifies
  one queued halfword-transfer projection and its original producer; 264 source
  bytes are sampled. [Evidence](../rom_data_runtime.md#e326-firaga-queued-halfword-transfer-source-plan).
- [x] **ROMDATA-04-FIRAGA-40E844-TRANSFER scoped EUR/Wine:** E261 publishes
  the selected transfer view. Reader `e327-firaga-queued-transfer-readers-wine-eas1b41j`, scene `e327-firaga-queued-transfer-scene-wine-9nspb3cr`,
  CLI `e327-firaga-queued-transfer-cli-wine-4jv_uinh` and join `e327-firaga-queued-transfer-sample-residual-nrrlw5qt` pass.
  [Evidence](../rom_data_runtime.md#e327-firaga-queued-halfword-transfer-runtime).
- [ ] **ROMDATA-REGRESSION-E261-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-40E844-OTHER:** establish other indices, source
  allocation/format, consumers and scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-40E844-LINUX:** validate publication, original
  transfer, selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-40E844-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-2D-2E-LITERALS scoped EUR/static:** E328 authenticates
  41 original Thumb LDR instruction/source pairs, 164 sampled bytes.
  [Evidence](../rom_data_runtime.md#e328-firaga-2d2e-thumb-literal-source-plan).
- [x] **ROMDATA-04-FIRAGA-2D-2E-LITERALS scoped EUR/Wine:** E262 publishes
  the 41 words. Reader `e329-firaga-2d-2e-literals-readers-wine-537ey132`, scene `e329-firaga-2d-2e-literals-scene-wine-ag5y0o6_`, CLI `e329-firaga-2d-2e-literals-cli-wine-stvyq0wt` and
  join `e329-firaga-2d-2e-literals-sample-residual-yfp0eet3` pass.
  [Evidence](../rom_data_runtime.md#e329-firaga-2d2e-thumb-literal-runtime).
- [ ] **ROMDATA-REGRESSION-E262-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-2D-2E-OTHER:** establish other readers, values,
  scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-2D-2E-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-2D-2E-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-EB83-EBC-LITERALS scoped EUR/static:** E330
  authenticates ten original LDR instruction/source pairs, 40 sampled bytes.
  [Evidence](../rom_data_runtime.md#e330-firaga-eb83ebc-thumb-literal-source-plan).
- [x] **ROMDATA-04-FIRAGA-EB83-EBC-LITERALS scoped EUR/Wine:** E263 publishes
  the ten words. Reader `e331-firaga-eb83-ebc-literals-readers-wine-u78tirug`, scene `e331-firaga-eb83-ebc-literals-scene-wine-uzod2dru`, CLI `e331-firaga-eb83-ebc-literals-cli-wine-t7kh3_rm` and
  join `e331-firaga-eb83-ebc-literals-sample-residual-s6x3bxvv` pass.
  [Evidence](../rom_data_runtime.md#e331-firaga-eb83ebc-thumb-literal-runtime).
- [ ] **ROMDATA-REGRESSION-E263-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-EB83-EBC-OTHER:** establish other readers, values,
  scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-EB83-EBC-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-EB83-EBC-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-CODE-LITERALS scoped EUR/static:** E332
  authenticates 14 original LDR instruction/source pairs, 56 sampled bytes.
  [Evidence](../rom_data_runtime.md#e332-firaga-0130190cf-thumb-literal-source-plan).
- [x] **ROMDATA-04-FIRAGA-CODE-LITERALS scoped EUR/Wine:** E264 publishes
  the 14 words. Reader `e333-firaga-code-literals-readers-wine-0j4qqizf`, scene `e333-firaga-code-literals-scene-wine-lcs3hcvx`, CLI `e333-firaga-code-literals-cli-wine-kijs396o` and
  join `e333-firaga-code-literals-sample-residual-we6mxexq` pass.
  [Evidence](../rom_data_runtime.md#e333-firaga-0130190cf-thumb-literal-runtime).
- [ ] **ROMDATA-REGRESSION-E264-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-CODE-LITERALS-OTHER:** establish other readers,
  values, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-CODE-LITERALS-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-CODE-LITERALS-USA:** identify and validate
  corresponding sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-MIXED-LITERALS scoped EUR/static:** E334
  authenticates 57 original LDR instruction/source pairs, 228 sampled bytes.
  [Evidence](../rom_data_runtime.md#e334-firaga-mixed-thumb-literal-source-plan).
- [x] **ROMDATA-04-FIRAGA-MIXED-LITERALS scoped EUR/Wine:** E265 publishes
  the 57 words. Reader `e335-firaga-mixed-literals-readers-wine-8ipblmws`, scene `e335-firaga-mixed-literals-scene-wine-ixvjw955`, CLI `e335-firaga-mixed-literals-cli-wine-7jyq36ot` and
  join `e335-firaga-mixed-literals-sample-residual-77a2186_` pass.
  [Evidence](../rom_data_runtime.md#e335-firaga-mixed-thumb-literal-runtime).
- [ ] **ROMDATA-REGRESSION-E265-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-MIXED-LITERALS-OTHER:** establish other readers,
  values, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-MIXED-LITERALS-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-MIXED-LITERALS-USA:** identify and validate
  corresponding sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-RESIDUAL-LITERALS scoped EUR/static:** E336
  authenticates 52 original LDR instruction/source pairs, 208 sampled bytes.
  [Evidence](../rom_data_runtime.md#e336-firaga-residual-thumb-literal-source-plan).
- [x] **ROMDATA-04-FIRAGA-RESIDUAL-LITERALS scoped EUR/Wine:** E266 publishes
  the 52 words. Reader `e337-firaga-residual-literals-readers-wine-tmmrcebi`, scene `e337-firaga-residual-literals-scene-wine-m1fas74s`, CLI `e337-firaga-residual-literals-cli-wine-qgtz2kbo` and
  join `e337-firaga-residual-literals-sample-residual-29h_ckxv` pass; build evidence `06cq110j` reconciles the original FAIL.
  [Evidence](../rom_data_runtime.md#e337-firaga-residual-thumb-literal-runtime).
- [ ] **ROMDATA-REGRESSION-E266-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [x] **ROMDATA-VALIDATION-STDOUT-BROKEN-PIPE scoped Wine:** E341 keeps a
  successful receipt and exit when only the final output pipe is closed. Wine
  CLI proof `e341-closed-stdout-wine-proof-v6cjp6xu` passed and drained; its
  inner unit is `e341-closed-stdout-cli-wine-zf76dich`. The earlier proof
  `e341-closed-stdout-wine-proof-qsu55wxi` remains FAIL: its inner receipt
  passed and drained, but Python's final flush returned 120.
- [ ] **ROMDATA-VALIDATION-STDOUT-BROKEN-PIPE-LINUX:** run dedicated host-side
  runner regressions, including true retirement and output failure cases, when
  Linux testing is selected.
- [ ] **ROMDATA-04-FIRAGA-RESIDUAL-LITERALS-OTHER:** establish other readers,
  values, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-RESIDUAL-LITERALS-LINUX:** validate publication, readers,
  selected scene and CLI when that platform is selected.
- [ ] **ROMDATA-04-FIRAGA-RESIDUAL-LITERALS-USA:** identify and validate
  corresponding sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-VERTICAL-EXTENSION scoped EUR/static:** E338
  authenticates the 22-byte physical projection for indices 10..20; prior
  indices 4..9 remain intact. [Evidence](../rom_data_gameplay.md#e338-firaga-vertical-lookup-source-projection).
- [x] **ROMDATA-04-FIRAGA-VERTICAL-EXTENSION scoped EUR/Wine:** E267 publishes
  indices 10..20 with isolated accounting. Reader `e339-firaga-vertical-extension-readers-wine-tzkb51gc`, scene `e339-firaga-vertical-extension-scene-wine-kf23c29b`,
  CLI `e339-firaga-vertical-extension-cli-wine-huxmebx6` and join `e339-firaga-vertical-extension-sample-residual-o6wga3mh` pass.
  [Evidence](../rom_data_gameplay.md#e339-firaga-vertical-lookup-runtime-projection).
- [ ] **ROMDATA-REGRESSION-E267-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-VERTICAL-OTHER:** establish other indices, callers,
  scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-FIRAGA-VERTICAL-LINUX:** validate publication and the
  selected readers, scene and CLI when Linux is selected.
- [ ] **ROMDATA-04-FIRAGA-VERTICAL-USA:** identify and validate corresponding
  sources; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-SCOPED-INDEX2 scoped EUR/static:** E340
  authenticates five sampled bytes as physical index-2 projections of two
  existing Soil scoped-reader bases. [Evidence](../rom_data_runtime.md#e340-firaga-scoped-index-2-source-plan).
- [x] **ROMDATA-04-FIRAGA-SCOPED-INDEX2 scoped EUR/Wine:** E342 publishes
  the selected word and byte with previous values and fallback preserved.
  Reader `e342-firaga-scoped-index2-readers-wine-4cdn3xwg`, scene `e342-firaga-scoped-index2-scene-wine-5o3bgt5b`, CLI `e342-firaga-scoped-index2-cli-wine-54wdcjp2` and join `e342-firaga-scoped-index2-sample-residual-81qecn8v` pass.
  [Evidence](../rom_data_runtime.md#e342-firaga-scoped-index-2-runtime).
- [ ] **ROMDATA-REGRESSION-E268-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-SCOPED-INDEX2-OTHER:** establish other indices,
  callers and scenes, real BIOS and UI save/Load; the sample lacks PCs.
- [ ] **ROMDATA-04-FIRAGA-SCOPED-INDEX2-LINUX-USA:** validate the changed
  projection on Linux and identify/validate USA; native Windows is unverified.
- [x] **ROMDATA-01-FIRAGA-PALETTE-SWITCH-PLAN scoped EUR/static:** E344
  authenticates 104 bytes in two guarded 12/14-way tables, including 12
  sampled bytes. [Evidence](../rom_data_runtime.md#e344-firaga-palette-switch-source-plan).
- [x] **ROMDATA-04-FIRAGA-PALETTE-SWITCH scoped EUR/Wine:** E345 publishes
  both destination arrays, keeping previous literal roots and guest guards.
  Reader `e345-firaga-palette-switch-readers-wine-n12uhnkp`, scene `e345-firaga-palette-switch-scene-wine-19we6cc1`, CLI `e345-firaga-palette-switch-cli-wine-lkro3br9` and join `e345-firaga-palette-switch-sample-residual-no7zdoby` pass.
  [Evidence](../rom_data_runtime.md#e345-firaga-palette-switch-runtime).
- [ ] **ROMDATA-REGRESSION-E269-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-PALETTE-SWITCH-OTHER:** establish other readers,
  scenes, real BIOS and UI save/Load; sample executing PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-PALETTE-SWITCH-LINUX-USA:** validate the changed
  sources on Linux and identify/validate USA; native Windows is unverified.
- [ ] **ROMDATA-01-FIRAGA-RESIDUAL-CONTEXT-606:** select a targeted longer
  Wine capture or obtain independent source contracts before attributing the
  remaining E302 addresses. E346's 12-frame native-API observer intersects
  none of the E269 residual; its blind spots and stale-PC limits remain.
  [Evidence](../rom_data_runtime.md#e346-firaga-short-native-residual-context).
- [x] **ROMDATA-01-FIRAGA-14CF-READERS scoped EUR/static:** E347 identifies
  two compatible signed-halfword readers and six paired sampled indices.
  [Evidence](../rom_data_runtime.md#e347-firaga-signed-halfword-reader-candidates).
- [x] **ROMDATA-01-FIRAGA-14CF-PROJECTION-PLAN scoped EUR/static:** E348
  authenticates both readers and the 640-byte union of their low-byte physical
  domains. [Evidence](../rom_data_runtime.md#e348-firaga-signed-halfword-physical-projection-plan).
- [x] **ROMDATA-04-FIRAGA-14CF-PROJECTION scoped EUR/Wine:** E349 publishes
  the 640-byte physical union, retaining literal roots and guest indexing.
  Reader `e349-firaga-signed-halfwords-readers-wine-lh4xw9gy`, scene `e349-firaga-signed-halfwords-scene-wine-0w4y_z_n`, CLI `e349-firaga-signed-halfwords-cli-wine-q2nr5p9n` and join `e349-firaga-signed-halfwords-sample-residual-io3xsjg4` pass.
  [Evidence](../rom_data_runtime.md#e349-firaga-signed-halfword-physical-projection-runtime).
- [ ] **ROMDATA-REGRESSION-E270-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-14CF-PROJECTION-OTHER:** establish input reachability,
  other readers, scenes, real BIOS and UI save/Load; sample PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-14CF-PROJECTION-LINUX-USA:** validate the changed
  sources on Linux and identify/validate USA; native Windows is unverified.
- [x] **ROMDATA-01-FIRAGA-INDEX2-FIELDS scoped EUR/static:** E350 identifies
  five compatible original readers for nine sampled bytes.
  [Evidence](../rom_data_runtime.md#e350-firaga-indexed-field-candidates).
- [x] **ROMDATA-01-FIRAGA-INDEX2-FIELDS-PLAN scoped EUR/static:** E351
  authenticates the five readers, prior literal roots and nine source bytes.
  [Evidence](../rom_data_runtime.md#e351-firaga-indexed-field-source-plan).
- [x] **ROMDATA-04-FIRAGA-INDEX2-FIELDS scoped EUR/Wine:** E352 publishes
  five physical fields, retaining literal roots and guest fallback. Reader
  `e352-firaga-index2-fields-readers-wine-enxi8h4h`, scene `e352-firaga-index2-fields-scene-wine-jn68cocs`, CLI `e352-firaga-index2-fields-cli-wine-zbyfd8qq` and join `e352-firaga-index2-fields-sample-residual-pxnn7sao` pass.
  [Evidence](../rom_data_runtime.md#e352-firaga-indexed-field-runtime).
- [ ] **ROMDATA-REGRESSION-E271-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-INDEX2-FIELDS-OTHER:** establish other indices,
  readers/scenes, real BIOS and UI save/Load; sample PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-INDEX2-FIELDS-LINUX-USA:** validate the changed
  sources on Linux and identify/validate USA; native Windows is unverified.
- [x] **ROMDATA-01-FIRAGA-INDIRECT-INDEX2-PLAN scoped EUR/static:** E353
  authenticates the eight-byte physical record, including seven sampled bytes.
  [Evidence](../rom_data_runtime.md#e353-firaga-indirect-index-2-source-plan).
- [x] **ROMDATA-04-FIRAGA-INDIRECT-INDEX2 scoped EUR/Wine:** E354 publishes
  the eight-byte record with existing roots and guest fallback. Reader
  `e354-firaga-indirect-index2-readers-wine-b7cio2t3`, scene `e354-firaga-indirect-index2-scene-wine-lx36oq5k`, CLI `e354-firaga-indirect-index2-cli-wine-_s7qf3ro` and join `e354-firaga-indirect-index2-sample-residual-61erx_sh` pass.
  [Evidence](../rom_data_runtime.md#e354-firaga-indirect-index-2-runtime).
- [ ] **ROMDATA-REGRESSION-E272-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-INDIRECT-INDEX2-OTHER:** establish other indices,
  callers and scenes, real BIOS and UI save/Load; sample PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-INDIRECT-INDEX2-LINUX-USA:** validate the changed
  source on Linux and identify/validate USA; native Windows is unverified.
- [x] **ROMDATA-01-FIRAGA-LINKED-TRANSFER-CANDIDATE scoped EUR/Wine:** E355
  identifies a conditional index-2 one-row path to a type-3 row and linked
  descriptor. Disassembly `e355-firaga-residual-readers-disassembly-wine-fvczyws1` passes and drains.
  [Evidence](../rom_data_runtime.md#e355-firaga-linked-transfer-candidates).
- [x] **ROMDATA-01-FIRAGA-LINKED-TRANSFER-PLAN scoped EUR/static:** E356
  authenticates the 20-byte row/descriptor, original readers, prior
  wrapper/stream owners and 17-byte sampled intersection.
  [Evidence](../rom_data_runtime.md#e356-firaga-linked-transfer-source-plan).
- [x] **ROMDATA-04-FIRAGA-LINKED-TRANSFER scoped EUR/Wine:** E357 publishes
  the row and descriptor, reusing prior wrapper/stream owners. Reader
  `e357-firaga-linked-transfer-readers-wine-be8l1_ah`, scene `e357-firaga-linked-transfer-scene-wine-izp2e4pq`, CLI `e357-firaga-linked-transfer-cli-wine-6dug_n2c` and join `e357-firaga-linked-transfer-sample-residual-ah1pd9hx` pass.
  [Evidence](../rom_data_runtime.md#e357-firaga-linked-transfer-runtime).
- [ ] **ROMDATA-REGRESSION-E273-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-LINKED-TRANSFER-OTHER:** establish other indices,
  callers and scenes, real BIOS and UI save/Load; sample PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-LINKED-TRANSFER-LINUX-USA:** validate the changed
  source on Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-PALETTE-PROJECTION-PLAN scoped EUR/static:** E358
  authenticates a conditional 16-halfword projection and two sampled bytes.
  [Evidence](../rom_data_runtime.md#e358-firaga-caller-bounded-palette-projection-plan).
- [x] **ROMDATA-04-FIRAGA-PALETTE-PROJECTION scoped EUR/Wine:** E359 publishes
  the 32-byte projection with prior root and guest fallback. Reader
  `e359-firaga-palette-projection-readers-wine-4vc70dqr`, scene `e359-firaga-palette-projection-scene-wine-mrf6k1gq`, CLI `e359-firaga-palette-projection-cli-wine-x3vrl3f5` and join `e359-firaga-palette-projection-sample-residual-w9mdf6yf` pass.
  [Evidence](../rom_data_runtime.md#e359-firaga-palette-projection-runtime).
- [ ] **ROMDATA-REGRESSION-E274-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-PALETTE-PROJECTION-OTHER:** establish other callers
  and scenes, real BIOS and UI save/Load; sample PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-PALETTE-PROJECTION-LINUX-USA:** validate on Linux
  and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-FIRAGA-CCEC-INDIRECT-PLAN scoped EUR/static:** E360
  authenticates the eight-byte physical record, existing index-2 pointer,
  original consumer and all seven sampled residual bytes.
  [Evidence](../rom_data_runtime.md#e360-firaga-ccec-indirect-record-source-plan).
- [x] **ROMDATA-04-FIRAGA-CCEC-INDIRECT scoped EUR/Wine:** E361 publishes
  the eight-byte record with prior pointer/root owners and guest fallback.
  Reader `e361-firaga-ccec-indirect-readers-wine-zcuhebx4`, scene `e361-firaga-ccec-indirect-scene-wine-_jvgs94x`, CLI `e361-firaga-ccec-indirect-cli-wine-z4yf7_da` and join `e361-firaga-ccec-indirect-sample-residual-idrflo71` pass.
  [Evidence](../rom_data_runtime.md#e361-firaga-ccec-indirect-record-runtime).
- [ ] **ROMDATA-REGRESSION-E275-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-FIRAGA-CCEC-INDIRECT-OTHER:** establish other indices,
  callers and scenes, real BIOS and UI save/Load; sample PCs remain unknown.
- [ ] **ROMDATA-04-FIRAGA-CCEC-INDIRECT-LINUX-USA:** validate on Linux and
  identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-STAR-CROSS-RAW scoped EUR/Wine:** E403 inventories selected
  readers during 480 matching frames; 7,359 sampled bytes are outside E291
  declared sources. [Evidence](../rom_data_runtime.md#e403-star-cross-rom-read-inventory).
- [x] **ROMDATA-01-STAR-CROSS-RESIDUAL-GROUPS scoped EUR/static:** E404 groups
  7,359 sampled bytes into 37 intervals, none in the known copied-IWRAM block;
  original readers and extents remain unproved. [Evidence](../rom_data_runtime.md#e404-star-cross-residual-grouping).
- [x] **ROMDATA-01-STAR-CROSS-BANK-PLAN scoped EUR/static:** E405 authenticates
  selectors 0/1, lists 0–22 and 2,116 unowned source bytes, all observed by
  bus; 2,014 list bytes also appear in sprite peeks. [Evidence](../rom_data_compact_catalog.md#e405-star-cross-compact-bank-source-plan).
- [x] **ROMDATA-01-STAR-CROSS-BANK-RUNTIME scoped EUR/Wine:** E406 serves the
  2,116 added compact-bank bytes through guest and typed host paths; original
  readers, altered-source 480-frame scene and CLI pass with Wine drained.
  [Evidence](../rom_data_compact_catalog.md#e406-star-cross-compact-bank-runtime).
- [ ] **ROMDATA-01-STAR-CROSS-BANK-OTHER:** establish other callers, selectors
  outside the projection, scenes and full animation without imposing guest bounds.
- [x] **ROMDATA-01-STAR-CROSS-E292-RESIDUAL scoped EUR/static:** E407 joins
  E403 to E292: 2,116 sampled bytes now owned, 5,243 still unowned in 33
  intervals. [Evidence](../rom_data_runtime.md#e407-star-cross-sample-reconciliation).
- [x] **ROMDATA-01-STAR-CROSS-LZ-PLAN scoped EUR/static:** E408 authenticates
  two observed unowned LZ77 sources totaling 4,960 encoded/9,296 decoded bytes.
  [Evidence](../rom_data_graphics.md#e408-star-cross-lz77-source-plan).
- [x] **ROMDATA-01-STAR-CROSS-LZ-RUNTIME scoped EUR/Wine:** E409 serves both
  streams, preserves earlier owners and passes original SWI/transfer,
  altered-source 480-frame scene and CLI checks with Wine drained.
  [Evidence](../rom_data_graphics.md#e409-star-cross-lz77-runtime).
- [ ] **ROMDATA-01-STAR-CROSS-LZ-OTHER:** identify asset/callers and verify
  other scenes, real BIOS, Linux, USA and native Windows.
- [x] **ROMDATA-01-STAR-CROSS-E293-RESIDUAL scoped EUR/static:** E410 joins
  E403 to E293: 4,960 further sampled bytes now owned, leaving 283 in 31
  intervals. [Evidence](../rom_data_runtime.md#e410-star-cross-sample-reconciliation).
- [x] **ROMDATA-01-STAR-CROSS-LITERALS-PLAN scoped EUR/Wine:** E411 authenticates
  19 original Thumb readers and 76 unowned sampled bytes.
  [Evidence](../rom_data_runtime.md#e411-star-cross-literal-source-plan).
- [x] **ROMDATA-04-STAR-CROSS-LITERALS-RUNTIME scoped EUR/Wine:** E412 serves
  19 words; original readers, altered-source 480-frame scene and CLI pass
  with Wine drained. [Evidence](../rom_data_runtime.md#e412-star-cross-literal-runtime).
- [ ] **ROMDATA-04-STAR-CROSS-LITERALS-OTHER:** establish other readers, scenes,
  real BIOS, Linux, USA and native Windows.
- [ ] **ROMDATA-REGRESSION-E292-WINE:** adapt and rerun affected older
  revision-sensitive compact-bank fixtures when selected.
- [ ] **ROMDATA-REGRESSION-E293-WINE:** adapt and rerun affected older
  revision-sensitive observed-LZ fixtures when selected.
- [ ] **ROMDATA-REGRESSION-E294-WINE:** adapt and rerun affected older
  revision-sensitive literal fixtures when selected.
- [x] **ROMDATA-01-STAR-CROSS-E294-RESIDUAL scoped EUR/static:** E413 joins
  E403 to E294: 76 further sampled bytes now owned, leaving 207 in 22
  intervals. [Evidence](../rom_data_runtime.md#e413-star-cross-sample-reconciliation).
- [x] **ROMDATA-01-STAR-CROSS-COPIED-TABLES-PLAN scoped EUR/static:** E414
  authenticates four copied spans totaling 60 sampled unowned bytes, their
  roots and original copy paths. [Evidence](../rom_data_runtime.md#e414-star-cross-copied-tables-source-plan).
- [ ] **ROMDATA-04-STAR-CROSS-COPIED-TABLES-RUNTIME:** integrate the four spans
  with original fallback and neighboring ownership, then verify copied readers,
  altered-source scene and CLI on Wine.
- [ ] **ROMDATA-01-STAR-CROSS-RESIDUAL:** classify the 207 remaining sampled bytes,
  authenticate original readers and integrate concrete data dependencies;
  distinguish executable/copied support and retain other scenes as open.
- [ ] **ROMDATA-01-STAR-CROSS-LINUX-USA:** verify selected coverage on Linux,
  identify and validate USA and check native Windows separately.
- [x] **ROMDATA-01-ROULETTE-NATIVE-RESIDUAL scoped EUR/static:** E402 joins
  the retained 12-frame E108 native inventory to E291; all 732 previously
  unowned bytes now have source owners. [Evidence](../rom_data_runtime.md#e402-roulette-native-sample-reconciliation).
- [ ] **ROMDATA-01-ROULETTE-OTHER-READERS:** investigate interpreter, host,
  other scenes and full-ROM coverage separately; native PCs remain hints.
- [x] **ROMDATA-01-WHITE-FLOWERS-RAW scoped EUR/Wine:** E362 inventories
  selected readers during 700 matching frames; 8,039 sampled bytes are outside
  E275 declared sources. [Evidence](../rom_data_runtime.md#e362-white-flowers-rom-read-inventory).
- [x] **ROMDATA-01-WHITE-FLOWERS-RESIDUAL-GROUPS scoped static:** E363 groups
  8,039 addresses; 3,600 lie in previously documented copied-code support and
  4,439 require classification. [Evidence](../rom_data_runtime.md#e363-white-flowers-residual-grouping).
- [x] **ROMDATA-01-WHITE-FLOWERS-SOURCE-CANDIDATES scoped static:** E364
  identifies an indexed archive candidate and separate halfword-root leads.
  [Evidence](../rom_data_runtime.md#e364-white-flowers-indexed-archive-candidate).
- [x] **ROMDATA-01-WHITE-FLOWERS-INDEXED-ARCHIVE-PLAN scoped EUR/static:** E365
  authenticates five archives, original code, roots, owners and sampled
  intersection. [Evidence](../rom_data_runtime.md#e365-white-flowers-localized-indexed-archive-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-INDEXED-ARCHIVES scoped EUR/Wine:** E366
  publishes five archives, group and literals; original readers, altered-source
  scene and CLI pass. [Evidence](../rom_data_runtime.md#e366-white-flowers-localized-indexed-archive-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-INDEXED-ARCHIVES-OTHER:** establish other
  ordinary language scenes, callers, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E276-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-INDEXED-ARCHIVES-LINUX-USA:** validate on
  Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-3BA0D8-ROOTS scoped EUR/static:** E367
  authenticates the conditional queued transfer, two 20 × 8 halfword sources,
  three literals and prior owners. [Evidence](../rom_data_runtime.md#e367-white-flowers-queued-halfword-rows-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-QUEUED-ROWS scoped EUR/Wine:** E368
  publishes both projections and literals; original selector/queue/DMA,
  altered-source scene and CLI pass. [Evidence](../rom_data_runtime.md#e368-white-flowers-queued-halfword-rows-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-QUEUED-ROWS-OTHER:** establish the other
  ordinary selector path, callers and scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E277-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-QUEUED-ROWS-LINUX-USA:** validate on Linux
  and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-RESIDUAL-E277 scoped static:** E369 groups
  6,437 remaining sampled bytes and identifies a conditional indexed archive
  lead. [Evidence](../rom_data_runtime.md#e369-white-flowers-remaining-source-candidates).
- [x] **ROMDATA-01-WHITE-FLOWERS-3C94BC-PLAN scoped EUR/static:** E370
  authenticates the archive, original caller, source reconstruction, owners
  and sample. [Evidence](../rom_data_runtime.md#e370-white-flowers-directly-rooted-indexed-archive-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-3C94BC scoped EUR/Wine:** E371 publishes
  the archive and literal; original wrapper/decoder, altered-source scene and
  CLI pass. [Evidence](../rom_data_runtime.md#e371-white-flowers-directly-rooted-indexed-archive-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-3C94BC-OTHER:** establish other ordinary
  programs, callers and scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E278-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-3C94BC-LINUX-USA:** validate on Linux and
  identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-FOUR-LITERAL-POOLS scoped EUR/static:**
  E372 authenticates 49 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e372-white-flowers-four-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-FOUR-LITERAL-POOLS scoped EUR/Wine:** E373
  publishes 49 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e373-white-flowers-four-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-FOUR-LITERAL-POOLS-OTHER:** establish other
  readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E279-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-FOUR-LITERAL-POOLS-LINUX-USA:** validate on
  Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-EIGHT-LITERAL-POOLS scoped EUR/static:**
  E374 authenticates 62 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e374-white-flowers-eight-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-EIGHT-LITERAL-POOLS scoped EUR/Wine:** E375
  publishes 62 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e375-white-flowers-eight-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-EIGHT-LITERAL-POOLS-OTHER:** establish other
  readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E280-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-EIGHT-LITERAL-POOLS-LINUX-USA:** validate on
  Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-TEN-LITERAL-POOLS scoped EUR/static:**
  E376 authenticates 62 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e376-white-flowers-ten-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-TEN-LITERAL-POOLS scoped EUR/Wine:** E377
  publishes 62 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e377-white-flowers-ten-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-TEN-LITERAL-POOLS-OTHER:** establish other
  readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E281-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-TEN-LITERAL-POOLS-LINUX-USA:** validate on
  Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-THIRTEEN-LITERAL-POOLS scoped EUR/static:**
  E378 authenticates 60 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e378-white-flowers-thirteen-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-THIRTEEN-LITERAL-POOLS scoped EUR/Wine:** E379
  publishes 60 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e379-white-flowers-thirteen-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-THIRTEEN-LITERAL-POOLS-OTHER:** establish other
  readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E282-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-THIRTEEN-LITERAL-POOLS-LINUX-USA:** validate on
  Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-TWENTY-EIGHT-LITERAL-POOLS scoped EUR/static:**
  E380 authenticates 92 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e380-white-flowers-twenty-eight-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-TWENTY-EIGHT-LITERAL-POOLS scoped EUR/Wine:** E381
  publishes 92 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e381-white-flowers-twenty-eight-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-TWENTY-EIGHT-LITERAL-POOLS-OTHER:** establish
  other readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E283-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-TWENTY-EIGHT-LITERAL-POOLS-LINUX-USA:**
  validate on Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-SIXTY-TWO-LITERAL-POOLS scoped EUR/static:**
  E382 authenticates 128 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e382-white-flowers-sixty-two-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-SIXTY-TWO-LITERAL-POOLS scoped EUR/Wine:** E383
  publishes 128 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e383-white-flowers-sixty-two-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-SIXTY-TWO-LITERAL-POOLS-OTHER:** establish
  other readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E284-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-SIXTY-TWO-LITERAL-POOLS-LINUX-USA:**
  validate on Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-REMAINING-LITERAL-POOLS scoped EUR/static:**
  E384 authenticates 182 original readers and disjoint source words.
  [Evidence](../rom_data_runtime.md#e384-white-flowers-remaining-sampled-literal-pools-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-REMAINING-LITERAL-POOLS scoped EUR/Wine:** E385
  publishes 182 words; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e385-white-flowers-remaining-sampled-literal-pools-runtime).
- [ ] **ROMDATA-04-WHITE-FLOWERS-REMAINING-LITERAL-POOLS-OTHER:** establish
  other readers, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-REGRESSION-E285-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-REMAINING-LITERAL-POOLS-LINUX-USA:**
  validate on Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-3320C-READER scoped EUR/static:** E386
  identifies original byte-indexed switch dispatch and its 256-word source;
  candidate `032eb8` is jump-table data. Sampled PC/ancestry remains unknown.
  [Evidence](../rom_data_runtime.md#e386-white-flowers-byte-indexed-switch-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-032E-SWITCH scoped EUR/Wine:** E387
  publishes 256 destinations; original dispatch, altered-source scene and CLI
  pass. [Evidence](../rom_data_runtime.md#e387-white-flowers-byte-indexed-switch-runtime).
- [ ] **ROMDATA-REGRESSION-E286-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-032E-SWITCH-OTHER:** verify other readers,
  selector ancestry, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-WHITE-FLOWERS-032E-SWITCH-LINUX-USA:** validate on Linux
  and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-RESIDUAL-SWITCHES scoped EUR/static:** E388
  authenticates seven switches, 373 destinations and disjoint sources.
  [Evidence](../rom_data_runtime.md#e388-white-flowers-residual-switch-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-RESIDUAL-SWITCHES scoped EUR/Wine:** E389
  publishes seven arrays; original dispatch, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e389-white-flowers-residual-switches-runtime).
- [ ] **ROMDATA-REGRESSION-E287-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-RESIDUAL-SWITCHES-OTHER:** verify other readers,
  selector ancestry, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-WHITE-FLOWERS-RESIDUAL-SWITCHES-LINUX-USA:** validate on
  Linux and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-MIXED-SWITCHES scoped EUR/static:** E390
  authenticates five switches, 89 destinations and five unowned literals.
  [Evidence](../rom_data_runtime.md#e390-white-flowers-mixed-switch-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-MIXED-SWITCHES scoped EUR/Wine:** E391
  publishes five switches and five literals; original readers, altered-source
  scene and CLI pass. [Evidence](../rom_data_runtime.md#e391-white-flowers-mixed-switches-runtime).
- [ ] **ROMDATA-REGRESSION-E288-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-MIXED-HELPER-RETURN:** investigate the
  original `0bbffc` out-of-domain helper return and any resulting later reads.
- [ ] **ROMDATA-04-WHITE-FLOWERS-MIXED-SWITCHES-OTHER:** verify other readers,
  selector ancestry, scenes, real BIOS and UI save/Load.
- [ ] **ROMDATA-04-WHITE-FLOWERS-MIXED-SWITCHES-LINUX-USA:** validate on Linux
  and identify/validate USA; native Windows remains unverified.
- [x] **ROMDATA-01-WHITE-FLOWERS-FINAL-SAMPLE scoped EUR/static:** E392 maps
  30 sampled bytes and authenticates three switches, 30 destinations and three
  literals covering 24 bytes. [Evidence](../rom_data_runtime.md#e392-white-flowers-final-sampled-switch-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-FINAL-SWITCHES scoped EUR/Wine:** E393 serves
  three arrays and literals; original readers, altered-source scene and CLI pass.
  [Evidence](../rom_data_runtime.md#e393-white-flowers-final-sampled-switches-runtime).
- [ ] **ROMDATA-REGRESSION-E289-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-FINAL-SWITCHES-OTHER:** verify other readers,
  selector ancestry, scenes, real BIOS and UI save/Load.
- [x] **ROMDATA-01-WHITE-FLOWERS-ISOLATED-SITES scoped EUR/Wine:** E394 observes
  all six bytes, with two interpreter scopes covering three bytes; remaining
  three are unscoped. [Evidence](../rom_data_runtime.md#e394-white-flowers-isolated-sampled-reader-scopes).
- [x] **ROMDATA-01-WHITE-FLOWERS-397B1E-CONTEXT scoped EUR/Wine:** E395 records
  three native non-DMA byte reads with shared dispatch context; exact load and
  extent remain unproved. [Evidence](../rom_data_runtime.md#e395-white-flowers-397b1e-native-context).
- [x] **ROMDATA-01-WHITE-FLOWERS-397B1E-SOURCE scoped EUR/static:** E396
  fingerprints the original byte-reader sequence and a non-overlapping 48-byte
  projection; [evidence](../rom_data_runtime.md#e396-white-flowers-397b1e-bounded-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-397B1E-PROJECTION scoped EUR/Wine:** E397
  publishes 48 signed bytes; original reader/helper, fallback, altered-source
  scene and CLI pass. [Evidence](../rom_data_runtime.md#e397-white-flowers-signed-byte-projection-runtime).
- [ ] **ROMDATA-REGRESSION-E290-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-397B1E-LINUX-USA:** validate on Linux and
  identify/validate USA; native Windows remains unverified.
- [ ] **ROMDATA-01-WHITE-FLOWERS-397B1E-DOMAIN:** identify producer ancestry,
  true asset extent, other readers and all possible initial cursor values.
- [x] **ROMDATA-01-WHITE-FLOWERS-ISOLATED-OPERANDS scoped EUR/Wine:** E398
  proves direct byte and halfword operands at the two sampled interpreter
  instructions. [Evidence](../rom_data_runtime.md#e398-white-flowers-isolated-original-operands).
- [x] **ROMDATA-01-WHITE-FLOWERS-ISOLATED-SOURCE-PLAN scoped EUR/static:** E399
  selects 20 halfwords and one byte with prior root ownership and original
  fallback. [Evidence](../rom_data_runtime.md#e399-white-flowers-isolated-source-plan).
- [x] **ROMDATA-04-WHITE-FLOWERS-ISOLATED-VALUES scoped EUR/Wine:** E400
  publishes both projections; original readers, outside fallback, 700-frame
  scene and CLI pass. [Evidence](../rom_data_runtime.md#e400-white-flowers-isolated-values-runtime).
- [ ] **ROMDATA-REGRESSION-E291-WINE:** adapt and rerun affected older
  revision-sensitive fixtures when selected.
- [ ] **ROMDATA-04-WHITE-FLOWERS-ISOLATED-VALUES-LINUX-USA:** validate on Linux
  and identify/validate USA; native Windows remains unverified.
- [ ] **ROMDATA-01-WHITE-FLOWERS-ISOLATED-SAMPLE:** determine full source
  extents, ancestry and other readers for the E397/E400 bounded projections.
- [ ] **ROMDATA-04-WHITE-FLOWERS-FINAL-SWITCHES-LINUX-USA:** validate on Linux
  and identify/validate USA; native Windows remains unverified.
- [ ] **ROMDATA-01-WHITE-FLOWERS-LOW-LITERAL-CANDIDATES:** authenticate the
  remaining lower-ROM candidate instruction contexts and source ownership.
- [x] **ROMDATA-01-WHITE-FLOWERS-SAMPLE-RECONCILIATION scoped EUR/static:**
  E401 confirms the E362 residual matches E246's 3,600 bytes inside the
  copied-IWRAM source block; its 48-byte metadata alias is already owned.
  [Evidence](../rom_data_runtime.md#e401-white-flowers-copied-code-support-reconciliation).
- [ ] **ROMDATA-01-WHITE-FLOWERS-RESIDUAL:** investigate other data readers and
  scenes; the selected address-only sample has no unclassified bytes outside
  the copied-code support block and does not prove full-ROM coverage.
- [ ] **ROMDATA-01-COPIED-IWRAM-CODE-DEPENDENCY:** the inventory excludes
  executable code from decoded assets and retains this block in ROM. A separate
  code-migration design and validation are required for ROM-free startup; do
  not publish the block as a decoded data asset.
- [ ] **ROMDATA-01-WHITE-FLOWERS-LINUX-USA:** repeat selected validation on
  Linux and identify/validate USA; native Windows remains unverified.
- [ ] **ROMDATA-01-FIRAGA-SCENE-RESIDUAL:** attribute the E302 sampled
  addresses to original readers and source contracts; zero remain outside E275
  source ownership in the selected E302 sample, but individual executing PCs
  and conditional ancestry remain open. Preserve code or other justified
  exclusions separately.
  Other scenes, consumers and full-ROM coverage remain open.
- [ ] **ROMDATA-01-FIRAGA-SCENE-LINUX:** reproduce the selected inventory and
  equivalent scene on Linux when that platform is selected.
- [ ] **ROMDATA-01-FIRAGA-SCENE-USA:** identify a corresponding USA scene and
  validate it when that region is selected; native Windows also remains open.
- [ ] **ROMDATA-01-SOIL-UPPER-NATIVE-CONTEXTS:** determine reader and caller
  ancestry for the 40 E285 outside-scope bytes near `ae71be..ae787c`; E248
  source ownership does not establish attribution or other scenes.
- [x] **ROMDATA-01-SOIL-14D6D8-SITES scoped EUR/Wine:** E282 attributes the
  eight sampled bytes to native read API calls with one outer dispatch context.
  [Evidence](../rom_data_runtime.md#e282-soil-evidence-14d6d8-native-read-contexts).
- [x] **ROMDATA-01-SOIL-14D6D8-READER scoped EUR/Wine/static:** E283
  authenticates an eight-byte copy and subsequent halfword indexing.
  [Evidence](../rom_data_runtime.md#e283-soil-evidence-copied-halfword-source-plan).
- [x] **ROMDATA-04-SOIL-14D6D8-HALFWORDS scoped EUR/Wine:** E240 serves the
  four copied halfwords; reader `e284-soil-copied-halfwords-readers-wine-r9a1m88c`, scene `e284-soil-copied-halfwords-scene-wine-6xd5_pj5`, CLI `e284-soil-copied-halfwords-cli-wine-af_b0kli` and
  sample join `e284-soil-copied-halfwords-sample-residual-z7jjst5p` pass. [Evidence](../rom_data_runtime.md#e284-soil-evidence-copied-halfword-runtime).
- [ ] **ROMDATA-01-SOIL-14D6D8-OTHER:** establish other callers/scenes, real
  BIOS and UI save/Load behavior; no complete guest index domain is inferred.
- [ ] **ROMDATA-04-SOIL-14D6D8-LINUX:** validate readers, transfers, scene and
  CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-14D6D8-USA:** identify corresponding USA sources;
  native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E240-WINE:** rerun affected older revision-sensitive
  tests when selected; earlier acceptance is preserved but E240 expectations
  were not rerun.
- [x] **ROMDATA-01-SOIL-146-LITERAL-SITES scoped EUR/Wine:** E280 attributes all
  13 earlier unscoped words to matching Thumb PC-relative loads under a bounded
  interpreter window. [Evidence](../rom_data_runtime.md#e280-soil-evidence-146xxx-original-literal-sites).
- [x] **ROMDATA-04-SOIL-146-LITERALS scoped EUR/Wine:** E239 serves 13
  observed words; reader `e281-soil-146-literals-readers-wine-h8dj_z7q`, scene `e281-soil-146-literals-scene-wine-fivwh4bx`, CLI `e281-soil-146-literals-cli-wine-f80mzg0p` and sample
  join `e281-soil-146-literals-sample-residual-anz9pb_y` pass. [Evidence](../rom_data_runtime.md#e281-soil-evidence-146xxx-literal-runtime).
- [ ] **ROMDATA-01-SOIL-146-LITERALS-OTHER:** establish other callers/scenes,
  real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-146-LITERALS-LINUX:** validate readers, transfers,
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-146-LITERALS-USA:** identify corresponding USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E239-WINE:** rerun affected older revision-sensitive
  tests when selected; earlier acceptance is preserved but E239 expectations
  were not rerun.
- [x] **ROMDATA-01-SOIL-REMAINING-SWITCH-PLAN scoped EUR/Wine/static:** E278
  authenticates nine guarded tables, 120 destinations and ten sampled words.
  [Evidence](../rom_data_runtime.md#e278-soil-evidence-remaining-switch-source-plan).
- [x] **ROMDATA-04-SOIL-REMAINING-SWITCHES scoped EUR/Wine:** E238 serves nine
  destination arrays without duplicating roots; reader `e279-soil-remaining-switches-readers-wine-tvvu9o31`, scene
  `e279-soil-remaining-switches-scene-wine-tgdwq7za`, CLI `e279-soil-remaining-switches-cli-wine-7oaysfzd` and sample join `e279-soil-remaining-switches-sample-residual-vdq2xyeo` pass.
  [Evidence](../rom_data_runtime.md#e279-soil-evidence-remaining-switch-runtime).
- [ ] **ROMDATA-01-SOIL-REMAINING-SWITCHES-OTHER:** establish selector writers,
  other callers/scenes, real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-REMAINING-SWITCHES-LINUX:** validate readers, transfers,
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-REMAINING-SWITCHES-USA:** identify corresponding USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E238-WINE:** rerun affected older revision-sensitive
  tests when selected; earlier acceptance is preserved but E238 expectations
  were not rerun.
- [x] **ROMDATA-01-SOIL-A83C2-WORDS scoped EUR/Wine/static:** E276 authenticates
  a 15-entry switch with existing literal root and three sampled indices.
  [Evidence](../rom_data_runtime.md#e276-soil-evidence-a83c2-switch-source-plan).
- [x] **ROMDATA-04-SOIL-A83C2-SWITCH scoped EUR/Wine:** E237 serves 15
  destinations without duplicating the existing root; reader `e277-soil-a83c2-switch-readers-wine-syjfuorl`,
  scene `e277-soil-a83c2-switch-scene-wine-76pzfrqd`, CLI `e277-soil-a83c2-switch-cli-wine-bx2ywffb` and sample join `e277-soil-a83c2-switch-sample-residual-ab5klphb` pass.
  [Evidence](../rom_data_runtime.md#e277-soil-evidence-a83c2-switch-runtime).
- [ ] **ROMDATA-01-SOIL-A83C2-OTHER:** establish selector writers, other
  callers/scenes, real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-A83C2-LINUX:** validate readers, transfers, scene and
  CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-A83C2-USA:** identify corresponding USA sources;
  native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E237-WINE:** rerun affected older revision-sensitive
  tests when selected; earlier acceptance is preserved but E237 expectations
  were not rerun.
- [x] **ROMDATA-01-SOIL-E3838-PLAN scoped EUR/Wine/static:** E268 authenticates
  the reader sequence, root and 30 sampled words/120 bytes; no complete index
  bound is inferred. [Evidence](../rom_data_runtime.md#e268-soil-evidence-e3838-indexed-source-plan).
- [x] **ROMDATA-04-SOIL-E3838-OBSERVED-WORDS scoped EUR/Wine:** E234 serves
  the 30 E268 words, preserving signed arithmetic and fallback. Wine reader
  `e269-soil-e3838-readers-wine-dpssxyfq`, scene `e269-soil-e3838-scene-wine-t22ztsv4`, CLI `e269-soil-e3838-cli-wine-oon2tik4` and join `e269-soil-e3838-sample-residual-5aymku00` pass.
  [Evidence](../rom_data_runtime.md#e269-soil-evidence-sparse-indexed-words).
- [ ] **ROMDATA-04-SOIL-E3838-OTHER:** establish other consumers and scenes,
  real BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-SOIL-E3838-LINUX:** validate readers, fallback, transfers,
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-E3838-USA:** identify corresponding USA sources;
  native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E234-WINE:** rerun affected older revision-sensitive
  tests when selected; prior acceptance remains valid but E234 expectations
  were not rerun.
- [ ] **ROMDATA-01-SOIL-E235-UPPER-READS:** identify source formats and readers
  for the remaining 285 upper sampled bytes. E273 finds 20 list-offset matches;
  E274/E275 identify and integrate the two 44-byte metadata intervals.
  Other formats, readers and ownership remain open. [Evidence](../rom_data_runtime.md#e273-soil-evidence-list-adjacent-layout).
- [x] **ROMDATA-01-SOIL-6C173C-BANK scoped EUR/Wine/static:** E274 verifies
  88 metadata bytes, 20 existing list references and one 20-frame selector 0;
  all selected bytes are observed in the Wine source-site capture.
  [Evidence](../rom_data_compact_catalog.md#e274-soil-evidence-compact-metadata-plan).
- [x] **ROMDATA-04-SOIL-6C173C-METADATA scoped EUR/Wine:** E236 serves the
  two metadata spans without duplicating E271 list sources; reader `e275-soil-compact-readers-wine-myffsalx`,
  scene `e275-soil-compact-scene-wine-gmfuh78b`, CLI `e275-soil-compact-cli-wine-jak27la2` and sample join `e275-soil-compact-sample-residual-j6x30g5v` pass.
  [Evidence](../rom_data_compact_catalog.md#e275-soil-evidence-compact-metadata-runtime).
- [ ] **ROMDATA-01-SOIL-6C173C-OTHER:** establish other selectors, producers,
  readers and scenes; real BIOS and UI save/Load remain open.
- [ ] **ROMDATA-04-SOIL-6C173C-LINUX:** validate metadata readers, transfers,
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-6C173C-USA:** identify corresponding USA metadata and
  validate its use; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E236-WINE:** rerun affected older revision-sensitive
  tests when selected; earlier acceptance is preserved but E236 expectations
  were not rerun.
- [x] **ROMDATA-01-SOIL-SPRITE-LISTS scoped EUR/static:** E270 identifies 20
  paired SpritePeek count/payload candidates, 586 sampled unowned bytes, with
  no candidate overlaps. [Evidence](../rom_data_runtime.md#e270-soil-evidence-spritepeek-list-candidates).
- [x] **ROMDATA-04-SOIL-SPRITE-LISTS scoped EUR/Wine:** E235 serves the 20
  E270 lists. Wine reader `e271-soil-sprite-lists-readers-wine-gywh4sti`, scene `e271-soil-sprite-lists-scene-wine-b3g8olpn`, CLI `e271-soil-sprite-lists-cli-wine-duz93oci` and
  sample join `e271-soil-sprite-list-sample-residual-ki3vs5jv` pass.
  [Evidence](../rom_data_runtime.md#e271-soil-evidence-typed-sprite-lists).
- [ ] **ROMDATA-01-SOIL-SPRITE-LISTS-PRODUCER:** establish pointer producers,
  complete asset domain and other guest/host consumers; other scenes, real BIOS
  and UI save/Load remain open.
- [ ] **ROMDATA-04-SOIL-SPRITE-LISTS-LINUX:** validate guest and host readers,
  transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-SPRITE-LISTS-USA:** identify corresponding USA sources;
  native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E235-WINE:** rerun affected older revision-sensitive
  tests when selected; prior acceptance remains valid but E235 expectations
  were not rerun.
- [ ] **ROMDATA-01-SOIL-E3838-DOMAIN:** establish index writers, complete domain,
  other readers and source extent without treating observed indices as a guard.
- [x] **ROMDATA-01-SOIL-LITERAL-PLAN scoped EUR/static:** E265 authenticates
  135 words/540 bytes from the E264 witnesses against E232, excluding reader
  and source overlaps. [Evidence](../rom_data_runtime.md#e265-soil-evidence-literal-source-plan).
- [x] **ROMDATA-04-SOIL-EVIDENCE-OBSERVED-LITERALS scoped EUR/Wine:** E233
  serves the 135 E265 words. Wine reader `e266-soil-literals-readers-wine-itca7x6l`, scene `e266-soil-literals-scene-wine-8wxjny7m`, CLI
  `e266-soil-literals-cli-wine-ptvqd89o` and join `e266-soil-literal-sample-residual-u2vrc_4i` pass.
  [Evidence](../rom_data_runtime.md#e266-soil-evidence-observed-literal-sources).
- [ ] **ROMDATA-04-SOIL-EVIDENCE-LITERALS-OTHER:** establish other callers,
  source domains and scenes, real-BIOS behavior and UI save/Load.
- [ ] **ROMDATA-04-SOIL-EVIDENCE-LITERALS-LINUX:** validate affected readers,
  boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-EVIDENCE-LITERALS-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E233-WINE:** rerun affected older revision-sensitive
  tests when selected; their previous acceptance remains valid but updated
  E233 expectations are unverified.
- [ ] **ROMDATA-04-SOIL-EVIDENCE-LZ-OTHER:** establish asset identity, other
  scenes and consumers, real-BIOS behavior and UI save/Load.
- [ ] **ROMDATA-01-SOIL-EVIDENCE-LINUX:** validate affected readers, transfers,
  scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-SOIL-EVIDENCE-LZ-USA:** identify and authenticate USA
  sources; native Windows remains unverified.
- [ ] **ROMDATA-REGRESSION-E232-WINE:** rerun affected older revision-sensitive
  reader tests when selected, including the interleaved LZ test; their earlier
  accepted evidence remains valid; E232 and later expectations were not rerun.
- [x] **ROMDATA-01-BLUE-SCREW-RAW scoped EUR/Wine:** `jyhm2s_n` captures
  E230 selected-reader addresses during 600 matching Blue Screw frames; 24
  sampled bytes are outside declared sources. This does not attribute PCs.
  [Evidence](../rom_data_runtime.md#e255-blue-screw-rom-read-inventory).
- [x] **ROMDATA-01-BLUE-SCREW-LITERAL-ANCHORS scoped static:** `0iwncua_`
  finds no bounded, straight-line translated-block-anchored Thumb LDR for the
  six sampled words; other reader forms and paths remain possible.
  [Evidence](../rom_data_runtime.md#e256-blue-screw-literal-anchor-scan).
- [x] **ROMDATA-01-BLUE-SCREW-SITES scoped EUR/Wine:** `m3aveg4f` captures all
  six words: three have matching PC-relative LDR witnesses and three share an
  indirect LDR scope. Nested reads and other callers remain possible.
  [Evidence](../rom_data_runtime.md#e257-blue-screw-residual-read-sites).
- [x] **ROMDATA-01-BLUE-SCREW-SOURCE-PLAN scoped EUR/Wine:** `jkaucm5o`
  authenticates three literals and a 15-entry indirect switch array covering
  all E255 sampled words; the plan itself publishes no owner.
  [Evidence](../rom_data_runtime.md#e258-blue-screw-source-plan).
- [x] **ROMDATA-01-BLUE-SCREW-RESIDUAL scoped EUR/Wine:** E231 serves the
  E258 literals and switch array. Wine readers `t4cf550o`, 600-frame scene
  `dk9h50xm`, CLI `9fermhwm` and sample join `uo47n9gw` pass.
  [Evidence](../rom_data_runtime.md#e259-blue-screw-literal-and-switch-sources).
- [ ] **ROMDATA-04-BLUE-SCREW-SOURCES-OTHER:** establish other readers, source
  domains and scenes, real-BIOS and UI save/Load; scopes may nest.
- [ ] **ROMDATA-04-BLUE-SCREW-SOURCES-LINUX:** validate affected readers,
  boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-BLUE-SCREW-SOURCES-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [ ] **ROMDATA-01-BLUE-SCREW-LINUX:** validate affected readers and scene on
  Linux when selected; USA and native Windows remain unverified.
- [x] **ROMDATA-04-BANK-OBSERVED-LITERALS scoped EUR/Wine:** E229-d228 serves
  77 witnessed words. Reader `biu5a8ky`, 12 altered-source Bank frames
  `fwu8_cee`, CLI `g8lsy8kq` and reconciliation `duj05bki` pass.
  [Evidence](../rom_data_runtime.md#e250-bank-observed-thumb-literals).
- [x] **ROMDATA-01-BANK-OTHER-SAMPLED-ANCHORS scoped static:** `hfew8swv`
  finds no straight-line translated-block-anchored Thumb LDR for these words
  within the scanned bounds; other reader forms and paths are not excluded.
  [Evidence](../rom_data_runtime.md#e251-bank-unscoped-literal-anchor-scan).
- [x] **ROMDATA-01-BANK-OTHER-SAMPLED-SITES scoped static:** `rm1pwc1a`
  authenticates the retained site capture: one 32-bit Bus read per word has
  a Thumb `LDR r0, [r0]` instruction scope. Nested reads remain possible.
  [Evidence](../rom_data_runtime.md#e252-bank-residual-read-sites).
- [x] **ROMDATA-01-BANK-OTHER-SAMPLED-SWITCH-PLAN scoped EUR/Wine:** `in0jqgky`
  authenticates three indexed switch sources with 5, 8 and 256 destination
  words and accounts for the three sampled words. The plan publishes no owner.
  [Evidence](../rom_data_runtime.md#e253-bank-switch-source-plan).
- [x] **ROMDATA-01-BANK-OTHER-SAMPLED-WORDS scoped EUR/Wine:** E230 serves all
  three E253 arrays; Wine reader `akk2y3mv`, 12 Bank frames `hcng484s`, CLI
  `oxj8x5e8` and sample join `re_5dymo` pass.
  [Evidence](../rom_data_runtime.md#e254-bank-switch-destinations).
- [ ] **ROMDATA-04-BANK-SWITCH-DESTINATIONS-OTHER:** establish other readers,
  selector domains and scenes, real-BIOS and UI save/Load; scopes may nest.
- [ ] **ROMDATA-04-BANK-SWITCH-DESTINATIONS-LINUX:** validate affected readers,
  boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-BANK-SWITCH-DESTINATIONS-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [ ] **ROMDATA-04-BANK-OBSERVED-LITERALS-OTHER:** establish other readers,
  source extents, scenes, real-BIOS and UI save/Load; scopes may nest.
- [ ] **ROMDATA-04-BANK-OBSERVED-LITERALS-LINUX:** validate affected readers,
  boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-BANK-OBSERVED-LITERALS-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-01-F09-HISTORICAL-RECONCILE scoped static:** `__2ozlaa`
  checks the retained 12-frame F09 inventory against E228. All 1,146 bytes
  previously outside E71 source ranges now have declared owners; no replay or
  current-reader claim is made.
  [Evidence](../rom_data_runtime.md#e249-historical-f09-inventory-reconciliation).
- [ ] **ROMDATA-01-F09-HISTORICAL-OTHER:** establish other readers, source
  domains and scenes; validate affected paths on Linux and USA when selected.
- [ ] **ROMDATA-01-POST-INDEXED-LOWER-READERS:** establish other readers,
  executing PCs, source domains and scenes. All 1,068 lower bytes in the E170
  sample now have declared sources (`aqz4qh3s`); 3,600 high bytes lie in the
  copied IWRAM code-support block retained in ROM by the existing contract.
  Linux, USA and native Windows remain open.
  [Evidence](../rom_data_runtime.md#e246-full-post-indexed-inventory-reconciliation).
- [x] **ROMDATA-01-POST-INDEXED-FULL-RECONCILE scoped static:** `aqz4qh3s`
  authenticates intersecting E228 source spans for the complete E170 unowned
  sample: 1,080 bytes are declared and 3,600 remain in the copied-code block.
  Individual byte semantics and full-ROM coverage are not established.
- [x] **ROMDATA-01-POST-INDEXED-UNSCOPED-ANCHORS scoped static:** `0uvhmsty`
  identifies original LDRs and straight-line exported-block anchors for all 18
  remaining sampled words. The E242 reads were unscoped; actual executing PCs
  and ownership are not established by the scan.
  [Evidence](../rom_data_runtime.md#e244-post-indexed-unscoped-literal-anchors).
- [x] **ROMDATA-04-POST-INDEXED-ANCHORED-LITERALS scoped EUR/Wine:** E228
  serves 18 anchored words. Reader `nssz0c5s`, 12 altered-source post-indexed
  frames `fc5v9tzl`, CLI `i52g_6ot` and reconciliation `rhmbe_o9` pass.
  [Evidence](../rom_data_runtime.md#e245-post-indexed-anchored-thumb-literals).
- [ ] **ROMDATA-04-POST-INDEXED-ANCHORED-LITERALS-OTHER:** establish executing
  PCs, other callers, source extents, scenes, real-BIOS and UI save/Load.
- [ ] **ROMDATA-04-POST-INDEXED-ANCHORED-LITERALS-LINUX:** validate affected
  readers, boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-POST-INDEXED-ANCHORED-LITERALS-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-01-POST-INDEXED-LOWER-SITES scoped EUR/Wine:** `qpmxbv2e`
  captures all 27 words in the 12-frame E170 continuation. Nine words have
  matching original Thumb LDR site/address witnesses; 19 read records are
  outside instruction scopes. No source is published by the capture.
  [Evidence](../rom_data_runtime.md#e242-post-indexed-lower-reader-sites).
- [x] **ROMDATA-04-POST-INDEXED-OBSERVED-LITERALS scoped EUR/Wine:** E227 serves
  nine witnessed words. Reader `6gxwak7q`, 12 altered-source post-indexed frames
  `fbe_qfgm`, CLI `uydq8tce` and reconciliation `n2okilgf` pass.
  [Evidence](../rom_data_runtime.md#e243-post-indexed-observed-thumb-literals).
- [ ] **ROMDATA-04-POST-INDEXED-OBSERVED-LITERALS-OTHER:** establish other
  callers, source extents, scenes, real-BIOS and UI save/Load; scopes may nest.
- [ ] **ROMDATA-04-POST-INDEXED-OBSERVED-LITERALS-LINUX:** validate affected
  readers, boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-POST-INDEXED-OBSERVED-LITERALS-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-01-POST-INDEXED-LOWER-RECONCILE scoped static:** of the 568
  E170 bytes unmatched by earlier witnesses, E226 declares 460: 408 literals,
  20 switch bytes and 32 display-control bytes. The remaining 108 bytes form
  14 ranges; `wxrezwcp` validates intersecting source hashes without replay.
  Prior witness and high copied-IWRAM subsets are outside this reconciliation.
- [x] **ROMDATA-01-E207-HOME-INVENTORY scoped EUR/Wine:** `kkbk45be` inventories
  600 recorded home frames with the E202 dataset and equal state, clock, video,
  OAM, dispatch and audio. Of 8,544 unowned sampled bytes, 6,436 are at or above
  `300000`; the address-only report does not attribute PCs or asset extents.
  [Evidence](../rom_data_runtime.md#e207-home-scene-rom-read-inventory).
- [x] **ROMDATA-01-E207-HOME-UNOWNED scoped EUR/Wine:** E221 owns all 8,544
  bytes previously unowned in this address-only sample (`dnr49xgd`).
- [ ] **ROMDATA-01-E207-HOME-OTHER:** establish other readers, source domains,
  scenes, real-BIOS and UI save/Load; this sample does not prove full coverage.
- [ ] **ROMDATA-01-E207-HOME-LINUX:** validate affected readers, transfers,
  home scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E207-HOME-USA:** identify and authenticate corresponding USA
  sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-HOME-OBSERVED-CODE scoped EUR/Wine:** E221-d220 adds 40
  literal words and two switch arrays. Reader `gwh_mblj`, 600 altered-source
  home frames `18841zsj`, CLI `q0do0j4t` and reconciliation `dnr49xgd` pass.
  [Evidence](../rom_data_runtime.md#e230-home-scene-observed-literals-and-switch-destinations).
- [ ] **ROMDATA-04-EUR-HOME-OBSERVED-CODE-OTHER:** establish other callers,
  selectors, scenes, real-BIOS and UI save/Load; scopes may include nested reads.
- [ ] **ROMDATA-04-EUR-HOME-OBSERVED-CODE-LINUX:** validate publication, readers,
  boundaries, transfers, home scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-HOME-OBSERVED-CODE-USA:** identify and authenticate the
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-HOME-THUMB-LITERALS scoped EUR/Wine:** E220-d219 adds
  25 anchored words. Reader `yc40amn_`, 600 altered-source home frames
  `29mql8p4`, CLI `__imn0tp` and reconciliation `ypnkepk0` pass.
  [Evidence](../rom_data_runtime.md#e229-home-scene-anchored-thumb-literals).
- [ ] **ROMDATA-04-EUR-HOME-THUMB-LITERALS-OTHER:** establish other readers,
  scenes, literal pools, real-BIOS and UI save/Load; anchors are static.
- [ ] **ROMDATA-04-EUR-HOME-THUMB-LITERALS-LINUX:** validate publication, readers,
  boundaries, transfers, home scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-HOME-THUMB-LITERALS-USA:** identify and authenticate the
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-01-E215-LLEDNAR-RAW scoped EUR/Wine:** E206 600-frame Llednar
  continuation matches state, clock, video, OAM, dispatch and audio. The Wine
  child passed but its unit retained generated input as private; independent
  recovery `nocdwh09` verifies the inputs and address inventory without replay.
  [Evidence](../rom_data_runtime.md#e215-llednar-battle-rom-read-inventory).
- [x] **ROMDATA-01-E215-LLEDNAR-UNOWNED scoped EUR/Wine:** E216–E228 own all
  4,281 previously unowned bytes in this address-only sample (`3blfucoi`).
  Other readers, scenes and source extents remain open.
- [ ] **ROMDATA-01-E215-LLEDNAR-LINUX:** validate the selected battle path and
  any migrated consumers on Linux when selected; USA remains unverified.
- [x] **ROMDATA-02-E216-BATTLE-OBJ scoped EUR/Wine:** E207-d206 consumes 442
  authenticated bytes in 17 lists and three sequences. Original reader,
  boundaries, host lists and transfers pass `drtfh3ff`; the altered-source
  600-frame battle continuation passes `5op6sk14`, and the production CLI
  create/reuse/verify passes `h524hpqf`.
  [Evidence](../rom_data_obj.md#e216-european-battle-number-obj-bank).
- [ ] **ROMDATA-02-E216-BATTLE-OBJ-OTHER:** establish remaining readers,
  sequence/index domains, scenes, real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-02-E216-BATTLE-OBJ-LINUX:** validate publication, readers,
  boundaries, host/transfer paths, scene use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E216-BATTLE-OBJ-USA:** identify and authenticate any
  corresponding USA source; native Windows remains unverified.
- [x] **ROMDATA-02-E217-BATTLE-OBJ-SUPPORT scoped EUR/Wine:** E208-d207 consumes
  seven callback words and four additional actor lookup entries. Original
  interpreter/generated readers and mixed boundaries pass `_pxxvoql`; the
  altered-source 600-frame battle passes `sk5simea` and production CLI
  create/reuse/verify passes `ayi5qr7q`.
  [Evidence](../rom_data_obj.md#e217-european-battle-obj-support-fields).
- [ ] **ROMDATA-02-E217-BATTLE-OBJ-SUPPORT-OTHER:** identify the `397fb4` word,
  other callback/lookup consumers, index domains, scenes, real-BIOS and UI save/Load.
- [ ] **ROMDATA-02-E217-BATTLE-OBJ-SUPPORT-LINUX:** validate publication,
  reader/branch ABI, boundaries, scene use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E217-BATTLE-OBJ-SUPPORT-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-02-E218-INDEXED-RECORDS16 scoped EUR/Wine:** E209-d208 serves
  all 86 authenticated records; original byte/halfword readers, adjacent/raw
  boundaries and transfers pass `68wakt20`, and the altered-source 600-frame
  battle continuation passes `z_mmhop1`. Production CLI create/reuse/verify
  passes `r1aazv8q`.
  [Evidence](../rom_data_gameplay.md#e218-european-indexed-16-byte-records).
- [ ] **ROMDATA-02-E218-INDEXED-RECORDS16-OTHER:** establish remaining field
  meanings, readers, index domains, scenes, real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-02-E218-INDEXED-RECORDS16-LINUX:** validate publication,
  readers, boundaries, transfers, scene use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E218-INDEXED-RECORDS16-USA:** identify and authenticate
  corresponding USA records; native Windows remains unverified.
- [x] **ROMDATA-04-3959A4 scoped EUR/Wine:** E210-d209 serves 30 words;
  source, original readers, transfers and boundaries pass `hdglegnr`, the
  altered-source 600-frame battle passes `des5bafu`, and CLI create/reuse/verify
  passes `snb6pezg`.
  [Evidence](../rom_data_gameplay.md#e219-european-indexed-word-lookup).
- [ ] **ROMDATA-04-3959A4-OTHER:** establish value meanings, remaining guest
  indices/readers and scenes, real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-04-3959A4-LINUX:** validate publication, original readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-3959A4-USA:** identify and authenticate any corresponding
  USA source; native Windows remains unverified.
- [x] **ROMDATA-04-THUMB-SWITCHES scoped EUR/Wine:** E211-d210 serves 23
  authenticated tables/696 destinations and roots. Source, original readers,
  boundaries and transfers pass `83yciuax`, 600 altered-source battle frames
  pass `01z89owc`, and production CLI create/reuse/verify passes `lr21g7ro`.
  [Evidence](../rom_data_runtime.md#e220-european-thumb-switch-table-resources).
- [ ] **ROMDATA-04-THUMB-SWITCHES-OTHER:** attribute other control tables and
  literals, additional indices/consumers/scenes, real-BIOS and UI save/Load.
- [ ] **ROMDATA-04-THUMB-SWITCHES-LINUX:** validate publication, readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-THUMB-SWITCHES-USA:** identify and authenticate corresponding
  USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-THUMB-LITERALS scoped EUR/Wine:** E212-d211 serves 326
  original PC-relative words. Source, original readers, mixed boundaries and
  transfers pass `e3auifw5`; 600 altered-source battle frames pass `qct03qw7`;
  production CLI create/reuse/verify passes `t5x79vsh`.
  [Evidence](../rom_data_runtime.md#e221-european-thumb-pc-relative-literals).
- [ ] **ROMDATA-04-THUMB-LITERALS-OTHER:** attribute other literal words,
  consumers, scenes, real-BIOS and UI save/Load; do not infer pointed-to assets.
- [ ] **ROMDATA-04-THUMB-LITERALS-LINUX:** validate publication, readers,
  mixed boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-THUMB-LITERALS-USA:** identify and authenticate corresponding
  USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-THUMB-LITERALS-ANCHORED scoped EUR/Wine:** E213-d212 serves
  117 additional words. Source `gzj3f255`, readers and boundaries `0qe1983j`,
  600 altered-source battle frames `osmbx9xm`, and production CLI `ajwb625v`
  pass. [Evidence](../rom_data_runtime.md#e222-additional-statically-anchored-thumb-literals).
- [ ] **ROMDATA-04-THUMB-LITERALS-ANCHORED-OTHER:** attribute other literal
  words, consumers, scenes, real-BIOS and UI save/Load; static anchors do not
  prove executing PCs in E215 or identify pointed-to assets.
- [ ] **ROMDATA-04-THUMB-LITERALS-ANCHORED-LINUX:** validate publication,
  readers, boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-THUMB-LITERALS-ANCHORED-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-LLEDNAR-LITERAL-SITES scoped EUR/Wine:** E214-d213 adds 443
  words. Scope `744c5av7`, readers/boundaries `qeb1_wme`, 600 altered-source
  battle frames `v5vdxrct` and CLI `73o_xfzb` pass.
  [Evidence](../rom_data_runtime.md#e223-battle-observed-thumb-literal-words).
- [ ] **ROMDATA-04-LLEDNAR-LITERAL-SITES-OTHER:** attribute remaining words,
  readers, scenes, real-BIOS and UI save/Load; scoped observations can include
  nested accesses and do not establish complete pools or pointed-to assets.
- [ ] **ROMDATA-04-LLEDNAR-LITERAL-SITES-LINUX:** validate publication, readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-LLEDNAR-LITERAL-SITES-USA:** identify and authenticate the
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-01-E215-LLEDNAR-RESIDUE-READERS scoped EUR/Wine:** E215-d214
  adds four switch arrays/163 destinations. Trace `qyrfym7w`, original windows
  `rit3fxua`, readers `6k14siel`, 600 altered-source battle frames `lzpuxs2a`,
  production CLI `4j66od25` and residual reconciliation `03amsoxp` pass.
  [Evidence](../rom_data_runtime.md#e224-additional-battle-switch-destinations).
- [ ] **ROMDATA-01-E215-LLEDNAR-RESIDUE-OTHER:** establish remaining source
  domains and other callers, scenes, real-BIOS and UI save/Load. The completed
  address-only sample does not prove full coverage. Do not impose a guest cap.
- [ ] **ROMDATA-01-E215-LLEDNAR-RESIDUE-LINUX:** validate publication, readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E215-LLEDNAR-RESIDUE-USA:** identify and authenticate the
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-OBJ-DERIVED-TABLES scoped EUR/Wine:** E216-d215 adds
  two dimension-derived views. Readers `riylj8n_`, 600 altered-source battle
  frames `o9_mw7ee`, production CLI `e_jxiaof` and reconciliation `53vwjg84`
  pass. [Evidence](../rom_data_runtime.md#e225-derived-obj-dimension-views).
- [ ] **ROMDATA-04-EUR-OBJ-DERIVED-TABLES-OTHER:** verify other readers, scenes,
  real-BIOS and UI save/Load; preserve reserved-shape physical reads.
- [ ] **ROMDATA-04-EUR-OBJ-DERIVED-TABLES-LINUX:** validate publication, readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-OBJ-DERIVED-TABLES-USA:** identify and authenticate the
  corresponding USA views; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-14CF18-MASKS scoped EUR/Wine:** E217-d216 serves two
  copied halfword tables and two roots. Reader `j75qi78l`, 600 altered-source
  battle frames `jv89wjir`, CLI `if6d22yl` and reconciliation `u6ti5mxt` pass.
  [Evidence](../rom_data_runtime.md#e226-copied-halfword-lookup-tables).
- [ ] **ROMDATA-04-EUR-14CF18-MASKS-OTHER:** verify other callers, index domains,
  scenes, real-BIOS and UI save/Load; preserve original outside-index behavior.
- [ ] **ROMDATA-04-EUR-14CF18-MASKS-LINUX:** validate publication, readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-14CF18-MASKS-USA:** identify and authenticate the
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-39A5C8 scoped EUR/Wine:** E218-d217 serves three
  indexed halfwords. Reader `ebfw8yn_`, 600 altered-source battle frames
  `jv5_43gi`, CLI `w3f33_xp` and reconciliation `l5w20wvc` pass.
  [Evidence](../rom_data_runtime.md#e227-indexed-halfwords-at-39a5c8).
- [ ] **ROMDATA-04-EUR-39A5C8-OTHER:** verify other callers, selector paths,
  scenes, real-BIOS and UI save/Load; preserve original outside-range behavior.
- [ ] **ROMDATA-04-EUR-39A5C8-LINUX:** validate publication, readers, boundaries,
  transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-39A5C8-USA:** identify and authenticate the corresponding
  USA source; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-3AD16E-LOOKUP scoped EUR/Wine:** E219-d218 serves the
  bounded physical window and second root. Readers `cu0__ocg`, 600 altered-source
  battle frames `frcsvz7a`, CLI `a2w8hrj6` and reconciliation `3blfucoi` pass.
  [Evidence](../rom_data_runtime.md#e228-script-byte-lookup-window).
- [ ] **ROMDATA-04-EUR-3AD16E-LOOKUP-OTHER:** establish semantic domain and other
  callers, scenes, real-BIOS and UI save/Load; retain outside-window behavior.
- [ ] **ROMDATA-04-EUR-3AD16E-LOOKUP-LINUX:** validate publication, readers,
  boundaries, transfers, battle scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-3AD16E-LOOKUP-USA:** identify and authenticate the
  corresponding USA source; native Windows remains unverified.
- [x] **ROMDATA-02-E214-SIGNED-LOOKUP scoped EUR/Wine:** E206-d205 serves four
  physical signed lookup bytes without limiting the original guest byte index.
  Reader and mixed boundaries `qo5za07r`, altered home scene `aj4omulx` and
  production CLI `yozdoyrs` pass.
  [Evidence](../rom_data_gameplay.md#e214-european-signed-task-lookup-projection).
- [ ] **ROMDATA-02-E214-SIGNED-LOOKUP-OTHER:** establish remaining indices,
  consumers, ordinary reader PCs, scenes, real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-02-E214-SIGNED-LOOKUP-LINUX:** validate publication, readers,
  boundaries, scene use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E214-SIGNED-LOOKUP-USA:** identify and authenticate any
  corresponding USA source; native Windows remains unverified.
- [x] **ROMDATA-02-E213-TILEMAP-LOOKUP scoped EUR/Wine:** E205-d204 serves the
  signed eight-row tile lookup and composes with the adjacent task-cycle owner.
  Original readers `y3wk6uev`, altered home scene `7fmxa2bo`, production CLI
  `ga0p99vm` and legacy-counter check `li2pps9d` pass.
  [Evidence](../rom_data_runtime.md#e213-european-eight-row-tile-lookup).
- [ ] **ROMDATA-02-E213-TILEMAP-LOOKUP-OTHER:** establish other consumers,
  alternate entries, scenes, real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-02-E213-TILEMAP-LOOKUP-LINUX:** validate publication, original
  drawing, boundaries, scene use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E213-TILEMAP-LOOKUP-USA:** identify and authenticate any
  corresponding USA source; native Windows remains unverified.
- [x] **ROMDATA-01-E211-HOME-INDEXED-SOURCE scoped EUR:** `mwuqtdap` verifies
  the three source roots, original caller, 495 programs, decoded lengths and
  dictionary prefixes. The third archive overlaps the previously owned four
  bytes at `3d4830..3d4834`; the scene capture remains address-only.
  [Evidence](../rom_data_indexed_graphics.md#e211-european-color-mode-archive-source-contract).
- [x] **ROMDATA-02-E211-HOME-INDEXED scoped EUR/Wine:** E204-d203 extracts
  three color-mode archives and composes with the existing four-byte owner.
  Original readers and mixed boundaries `d88bld36`, altered home scene
  `t6q5y978` and production CLI `vnsgav7l` pass.
  [Evidence](../rom_data_indexed_graphics.md#e212-european-color-mode-archive-publication).
- [ ] **ROMDATA-02-E211-HOME-INDEXED-OTHER:** establish additional ordinary
  consumers, modes and scenes, real-BIOS and UI save/Load behavior.
- [ ] **ROMDATA-02-E211-HOME-INDEXED-LINUX:** validate publication, original
  readers, mixed boundaries, scene use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E211-HOME-INDEXED-USA:** identify and authenticate any
  corresponding USA archives; native Windows remains unverified.
- [x] **ROMDATA-01-E208-HOME-RANGES scoped EUR source lead:** `rwsqv1y7`
  authenticates those three ranges as custom streams selected by entries 69,
  71 and 73 of the kind-4 directory at `3d63d4`, with a pinned original
  resolver and decoder path. The scene's executing PCs remain unproved.
  [Evidence](../rom_data_graphics.md#e208-home-scene-relative-archive-streams).
- [x] **ROMDATA-02-E208-HOME-STREAMS scoped EUR/Wine:** E203-d202 extracts and
  serves the full 103-entry directory. Reader `af7l9c_2`, altered-source home
  scene `deee1ha0` and production CLI `473fvdeq` pass; the scene reads entries
  69, 71 and 73. [Evidence](../rom_data_graphics.md#e210-relative-graphics-archive-publication).
- [ ] **ROMDATA-02-E210-RELATIVE-GRAPHICS-OTHER:** establish ordinary use of
  other indices and scenes, remaining callers, real-BIOS paths and UI save/Load.
- [ ] **ROMDATA-02-E210-RELATIVE-GRAPHICS-LINUX:** validate publication, readers,
  boundaries, ordinary use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E210-RELATIVE-GRAPHICS-USA:** identify and authenticate any
  corresponding USA directory and streams when regional work resumes; native
  Windows remains pending.
- [x] **ROMDATA-02-04-DISPLAY-CONTROL-TABLES scoped EUR/Wine:** model and publish the two
  four-halfword display-control arrays and four IWRAM pointers at
  `14d2dc..14d2fc`, then serve their bytes through the EUR runtime. Preserve
  live destination structures and raw fallback outside the three exact spans;
  verify affected source, mixed-boundary and original-reader behavior on Wine.
  E171 Wine `8bb1ykny` passes publication, boundaries, original-reader cases
  and a 12-frame ordinary pair. Production Wine CLI create/reuse/verify passes
  (`ikwqgxwg`).
  Other readers and selectors, Linux, USA and native Windows remain pending.
  [Source contract](../rom_data_runtime.md#european-display-control-source-tables-at-14d2dc).
- [x] **ROMDATA-01-E172-REDSPRING-RAW scoped EUR/Wine:** the retained
  600-frame action matches raw/managed state, clock, video, OAM, dispatch and
  audio, with positive compact-root use. `c9uprkbl` failed only while copying
  its oversized evidence; `05iu77v6` verified the original inputs and report
  and retained the complete inventory without a replay.
  [Evidence](../rom_data_compact_catalog.md#e172-red-spring-rom-read-inventory).
- [x] **ROMDATA-01-E172-REDSPRING-6C1E78-RAW scoped EUR/Wine:** E173 extends
  the authenticated compact bank by 634 observed bytes and connects its host
  sprite lists. Source/reader `s73gcfva`, altered-source 600-frame action
  `czzprb8z` and production CLI `u2r2goyn` pass. The first reader attempt
  `k2ma7lbz` used an object lacking the selected block and was corrected.
  [Evidence](../rom_data_compact_catalog.md#e173-red-spring-compact-bank-and-host-sprite-lists).
- [x] **ROMDATA-01-E173-REDSPRING-3A4CA0 scoped EUR/Wine:** E186 owns three
  records selecting the E173 compact bank. Source `ae4wgq4a`, Wine traces
  `2ibf_qi4`/`bldajpbj`, readers `k6d5krm5`, 600-frame scene `1zw44c41` and
  CLI `y0aiuls6` pass.
  [Evidence](../rom_data_compact_catalog.md#e186-red-spring-compact-bank-records).
- [ ] **ROMDATA-01-E186-COMPACT-RECORDS-LINUX:** repeat publication, original
  readers, boundaries, affected scene and CLI on Linux.
- [ ] **ROMDATA-01-E186-COMPACT-RECORDS-USA:** locate and authenticate equivalent
  USA records and validate their consumers; native Windows remains pending too.
- [x] **ROMDATA-01-E187-REDSPRING-SOURCE-TABLES scoped EUR/Wine:** two timed
  sound schedules and their literal roots are owned by the audio directory.
  Source `dybrm98g`, trace `t7gxddev`, readers `63mjpshl`, 600-frame scene
  `vrwrh7f0` and CLI `2xep4wwf` pass.
  [Evidence](../rom_data_audio.md#e187-red-spring-timed-sound-schedules).
- [ ] **ROMDATA-01-E187-SOUND-SCHEDULES-LINUX:** run publication, original
  readers, mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E187-SOUND-SCHEDULES-USA:** locate and authenticate the
  corresponding USA tables and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E187-SOUND-SCHEDULES-OTHER:** classify the separately rooted
  adjacent rows, the E186 record root `109628`, other schedules and consumers.
- [ ] **ROMDATA-01-E187-AUDIO-DIRECTORY-FIXTURES:** adapt revision-pinned older
  audio directory oracle/publication fixtures to E187 before rerunning them;
  keep Linux runs pending until Linux validation is selected.
- [x] **ROMDATA-01-E188-IWRAM-CONTROL-397F80 scoped EUR/Wine:** five IWRAM
  callback words are owned by the OBJ-list resource. Source `rm56pdw9`, Wine
  traces `sr60q_xk`/`fo65p2xu`, branches `ryj41yi9`, scene `ch94duvy` and CLI
  `3z2jda18` pass.
  [Evidence](../rom_data_runtime.md#e188-observed-iwram-obj-callback-words).
- [ ] **ROMDATA-01-E188-IWRAM-CONTROL-LINUX:** run publication, reader/branch,
  mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E188-IWRAM-CONTROL-USA:** locate and authenticate equivalent
  USA callback words and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E188-IWRAM-CONTROL-OTHER:** classify other callbacks, callers
  and live IWRAM code paths. E191 separately owns `397fa4` and `397fb8`.
- [ ] **ROMDATA-01-E188-OBJ-LIST-FIXTURES:** adapt older revision-pinned OBJ
  list fixtures to E188 before rerunning them; leave Linux runs pending.
- [x] **ROMDATA-01-E189-DMA-DIRECTORIES-370E68 scoped EUR/Wine:** source
  `_zu8l44g`, trace `vgsu5kue`, readers/four DMA writes `598qsw1v`, 600-frame
  scene `dgpightt` and CLI `98p9_ms6` pass.
  [Evidence](../rom_data_runtime.md#e189-dma-register-directories).
- [ ] **ROMDATA-01-E189-DMA-DIRECTORIES-LINUX:** validate publication, readers,
  DMA effects, boundaries, scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E189-DMA-DIRECTORIES-USA:** locate and authenticate equivalent
  USA directories and callers; native Windows remains pending.
- [ ] **ROMDATA-01-E189-DMA-DIRECTORIES-OTHER:** cover other DMA callers without
  restricting guest indices. E190 separately owns the `370eb0` helper vector.
- [x] **ROMDATA-01-E190-IWRAM-MEMORY-HELPERS-370EB0 scoped EUR/Wine:** source
  `2muj949h`, trace `1jrexdnx`, readers/branches `r9rce5o_`, 600-frame scene
  `94ao1rz2` and CLI `h_owgk16` pass.
  [Evidence](../rom_data_runtime.md#e190-iwram-memory-helper-vector).
- [ ] **ROMDATA-01-E190-IWRAM-HELPERS-LINUX:** validate publication, reader/branch,
  mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E190-IWRAM-HELPERS-USA:** locate and authenticate equivalent
  USA helpers and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E190-IWRAM-HELPERS-OTHER:** cover other callers and live
  IWRAM execution without imposing guest index or code immutability assumptions.
- [x] **ROMDATA-01-E191-IWRAM-CALLBACKS-397FA4 scoped EUR/Wine:** source
  `m460uxk9`, Wine trace `35oh5q8o`, publication/branches `gj6h0gcr`, 600-frame
  altered-source scene `_vpbz8be` and CLI `fttzwwhr` pass.
  [Evidence](../rom_data_runtime.md#e191-independently-rooted-iwram-obj-callbacks).
- [ ] **ROMDATA-01-E191-IWRAM-CALLBACKS-LINUX:** validate publication,
  reader/branch, mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E191-IWRAM-CALLBACKS-USA:** locate and authenticate equivalent
  USA callbacks and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E191-IWRAM-CALLBACKS-OTHER:** investigate other callbacks,
  callers and mutable IWRAM paths without adding guest index restrictions.
- [x] **ROMDATA-01-E192-LINKED-CALLBACK-TABLES-3AD638 scoped EUR/Wine:**
  source `vgus1zn1`, Wine trace `i99518dm`, publication/branches `rg_ejxyn`,
  600-frame altered-source scene `ditx_b0y` and CLI `yzo92bp7` pass. Reader and
  scene logs contain D-Bus assertions before the completed tests.
  [Evidence](../rom_data_gameplay.md#e192-linked-record-callback-tables).
- [ ] **ROMDATA-01-E192-LINKED-CALLBACKS-LINUX:** validate publication, readers,
  branches, mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E192-LINKED-CALLBACKS-USA:** identify and authenticate the
  corresponding USA tables and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E192-LINKED-CALLBACKS-OTHER:** investigate other consumers and
  paths without treating physical table extents as guest index bounds.
- [ ] **ROMDATA-01-E192-RECORDS4-FIXTURES:** adapt older revision-pinned records4
  fixtures for the new callback fields; leave Linux runs pending.
- [x] **ROMDATA-01-E193-ACTION-CALLBACK-RECORDS-3AD7E4 scoped EUR/Wine:**
  source `2po3wuk9`, Wine trace `fqhwgje1`, readers `jcz5bcc5`, 600-frame
  altered-source scene `vys970r2` and CLI `5zlmal_2` pass for 93 records.
  Reader and scene logs contain D-Bus assertions before successful completion.
  [Evidence](../rom_data_gameplay.md#e193-action-callback-records).
- [ ] **ROMDATA-01-E193-ACTION-CALLBACKS-LINUX:** validate publication, readers,
  mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E193-ACTION-CALLBACKS-USA:** locate and authenticate equivalent
  USA records and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E193-ACTION-CALLBACKS-OTHER:** investigate other consumers and
  preserve opaque words and unbounded guest indices; update old records4 fixtures.
- [ ] **ROMDATA-05-WINE-DBUS-PENDING-CALL:** the assertions in passing Wine
  reader/scene logs include `rg_ejxyn`/`ditx_b0y`, `jcz5bcc5`/`vys970r2`,
  `ug76r3cd`/`dh8ic7ya`, `yxymrems`, `54zxpyzj`/`s0azwl5i`, `i23_yk35` and
  `d97f6l58`, `29if0sg2` and `0vq42lsj`. Process trace
  `qxwtl8he` reproduces the assertion in `winedevice.exe`; static audit
  `ijsvoxgk` confirms the installed Wine 10.0 Bluetooth module has the NULL-call
  defect addressed by [Wine MR7302](https://list.winehq.org/hyperkitty/list/wine-gitlab@list.winehq.org/thread/ZEBOUKFLYJZWNZYMRFPFXAXU33I5OBWB/).
  This strongly implicates that defect, although the exact triggering stack and
  Bluetooth impact were not captured. A minimal Wine smoke `zkjnk_35` did not
  reproduce it. A local matching backport and opt-in leased wrapper are now
  available; see the completed local subtask below. System Wine still contains
  the defect. When a compatible corrected system runtime is available and a
  shared-resource change is authorized, validate the affected Wine path with
  no assertion and a drained server. Physical Bluetooth and asynchronous
  property updates remain unverified. Do not hide the warning or disable the
  service as a substitute for a fix.
- [x] **ROMDATA-05-WINE-DBUS-LOCAL-OVERLAY scoped EUR/Wine:** corrected module
  `s10dafvr` and loader proof `v50bntvr` pass. Blue Screw `hrc9ivgq` passes
  600 frames without the assertion and drains Wine; the unit FAIL from a split
  trace-parser record is reconciled by `qxfzqz9z`. The reusable leased wrapper
  passes Wine loader proof `25z4y70w`. System Wine and prefix settings remain
  unchanged. [Procedure](ROM_DATA_VALIDATION.md#wine-bluetooth-d-bus-diagnostic).
- [x] **ROMDATA-01-E194-ADJACENT-TABLES-3AD460 scoped EUR/Wine:** source
  `r364fmf9`, trace `uxtnxes3`, readers `ug76r3cd`, 600-frame altered-source
  scene `dh8ic7ya` and CLI `7ind1kk9` pass. Four tables were observed in the
  scene; reader and scene logs contain D-Bus assertions before successful completion.
  [Evidence](../rom_data_runtime.md#e194-adjacent-offset-tables).
- [ ] **ROMDATA-01-E194-ADJACENT-TABLES-LINUX:** validate publication, readers,
  mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E194-ADJACENT-TABLES-USA:** locate and authenticate equivalent
  USA tables and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E194-ADJACENT-TABLES-OTHER:** establish ordinary use of the
  fifth table and investigate other consumers without imposing guest bounds.
- [x] **ROMDATA-01-E195-TILEMAP-ALTERNATES-AE6E14 scoped EUR/Wine:** source
  `n6nl69bd`, trace `9x55uknb`, readers/branches `9sclx54d`, 600-frame
  altered-source scene `yxymrems` and CLI `3j_wfn2z` pass. The scene log has a
  D-Bus assertion before successful completion.
  [Evidence](../rom_data_graphics.md#e195-alternate-tilemap-rectangles).
- [ ] **ROMDATA-01-E195-TILEMAP-ALTERNATES-LINUX:** validate publication, readers,
  branches, mixed boundaries, affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E195-TILEMAP-ALTERNATES-USA:** locate and authenticate the
  corresponding USA sources and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E195-TILEMAP-ALTERNATES-OTHER:** prove ordinary alternate-branch
  copy and other consumers without imposing guest index bounds.
- [x] **ROMDATA-01-E196-OBJ-BYTES-398DE8 scoped EUR/Wine:** source `v9z3u37q`,
  Wine trace `jcuo75ia`, readers `54zxpyzj`, altered-source scene `s0azwl5i` and
  CLI `u7pcy_df` pass. Reader and scene logs contain D-Bus assertions before
  successful completion. [Evidence](../rom_data_obj.md#e196-obj-layout-byte-groups).
- [ ] **ROMDATA-01-E196-OBJ-BYTES-LINUX:** validate publication, readers, mixed
  boundaries, the affected scene and CLI on Linux when selected.
- [ ] **ROMDATA-01-E196-OBJ-BYTES-USA:** locate and authenticate corresponding
  USA sources and consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E196-OBJ-BYTES-OTHER:** establish ordinary use of the other
  byte groups and investigate other callers without imposing guest bounds.
- [ ] **ROMDATA-01-E172-REDSPRING-OTHER-RAW:** establish other callers, source
  extents and scenes beyond the reconciled E172 sample. The original 8,789
  bytes outside the E173 bank increment were addressed in later increments:
  E174 migrates 2,578
  and E175 another 2,291. E176 owns another 64, E177 another 100, E178
  another 7, E179 another 12, E180 another 48, E181 another 28, E182
  another 58, E183 another 86, E184 another 17, E185 another 6 and E186
  another 60, E187 another 57, E188 another 20, E189 another 24, E190 another
  12, E191 another eight, E192 another 16, E193 another six, E194 another five
  E195 another eight and E196 another two. E203 joined 420 of the 3,276
  remaining at E201 to code-support sites from another scene; E212 owns all
  420. Static E221 reconciliation `wt2lyljd` attributes 2,324 of the E201
  residual to declared source ranges, leaving 952 sampled bytes at E221.
  E222 adds 784, leaving 168 sampled bytes (`pz3zxelt`); E223 adds 64,
  leaving 104 (`c_2f3g_b`); E224 adds 88, leaving 16 (`55zvsm8r`); E225
  adds eight (`n1l57jp6`); E226 adds the last eight, leaving zero unowned bytes
  in the retained E172 sample (`_z04mity`). Executing PCs and alternate uses
  remain open; so do other scenes, consumers, Linux, USA and native Windows.
  [Evidence](../rom_data_runtime.md#e240-final-red-spring-copy-source).
- [x] **ROMDATA-01-E172-REDSPRING-E221-RECONCILE scoped static:** protected
  E172 sample, E201/E221 manifests and source hashes reconcile (`wt2lyljd`);
  952 sampled bus bytes remain outside E221 declared source ranges. No replay
  or complete-ROM coverage is claimed.
- [x] **ROMDATA-01-E172-REDSPRING-RESIDUAL-SITES scoped EUR/Wine:** `av1hj_7p`
  observes all 238 residual words; 196 have matching original Thumb LDR
  opcode/address witnesses. Scopes may include nested accesses and do not
  establish every caller. [Evidence](../rom_data_compact_catalog.md#e232-red-spring-residual-reader-scopes).
- [x] **ROMDATA-01-E172-REDSPRING-INDIRECT-SOURCES scoped EUR/Wine/static:**
  ten original guarded switch arrays, 148 destinations and 592 source bytes
  are authenticated (`3ubkkgml`/`rtqw8txm`); 64 overlap unowned E172 sample
  bytes. No destination ownership or complete caller/domain claim is made.
  [Evidence](../rom_data_runtime.md#e234-red-spring-indirect-switch-source-plan).
- [x] **ROMDATA-01-E172-UNSCOPED-LITERAL-ANCHORS scoped static:** `rlq73_nk`
  identifies 22 unowned sampled words with original LDRs and straight-line
  exported-block anchors. The addresses were observed outside an instruction
  scope; executing PCs and ownership are not established.
  [Evidence](../rom_data_runtime.md#e236-red-spring-unscoped-literal-anchors).
- [x] **ROMDATA-01-E172-FINAL-READERS scoped source plan:** Wine `jqg8wh29`/
  `56f3ef_x` and static `b39y_6rn` authenticate two guarded destination arrays
  and one eight-byte original copy source spanning the 16 E224-unowned sampled
  bytes. The address-only sample does not prove the executing PCs or publish
  any source. [Evidence](../rom_data_runtime.md#e238-red-spring-remaining-source-plan).
- [x] **ROMDATA-04-E172-FINAL-SWITCH-ARRAYS scoped EUR/Wine:** E225 serves the
  two arrays; reader `2zdfj3c8`, 600 altered-source Red Spring frames `6vjw7n51`,
  CLI `q_skv5h9` and sample reconciliation `n1l57jp6` pass.
  [Evidence](../rom_data_runtime.md#e239-final-red-spring-switch-arrays).
- [ ] **ROMDATA-04-E172-FINAL-SWITCH-ARRAYS-OTHER:** establish other callers,
  selectors, scenes, real-BIOS and UI save/Load; the sampled executing PCs and
  complete source families remain open.
- [x] **ROMDATA-04-E172-FINAL-COPIED-SOURCE scoped EUR/Wine:** E226 serves the
  eight-byte copy source; reader `e2rslaz3`, 600 altered-source Red Spring frames
  `xwjchks1`, CLI `_jk7dwh3` and sample reconciliation `_z04mity` pass.
  [Evidence](../rom_data_runtime.md#e240-final-red-spring-copy-source).
- [ ] **ROMDATA-04-E172-FINAL-COPIED-SOURCE-OTHER:** establish other callers,
  scenes, real-BIOS and UI save/Load; the sampled executing PCs and complete
  source families remain open.
- [ ] **ROMDATA-04-E172-FINAL-SOURCES-LINUX:** validate affected sources,
  original readers, boundaries, transfers, scene and CLI on Linux.
- [ ] **ROMDATA-04-E172-FINAL-SOURCES-USA:** identify and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-REDSPRING-ANCHORED-LITERALS scoped EUR/Wine:** E224
  serves 22 words. Reader `d0vf7iwd`, 600 altered-source Red Spring frames
  `ao5jx0xh`, CLI `dl95px1f` and reconciliation `55zvsm8r` pass.
  [Evidence](../rom_data_runtime.md#e237-red-spring-anchored-thumb-literals).
- [ ] **ROMDATA-04-EUR-REDSPRING-ANCHORED-LITERALS-OTHER:** establish other
  callers, literal pools, scenes, real-BIOS and UI save/Load; static anchors do
  not prove E172 executing PCs. E239/E240 serve the four other sampled words;
  broader families remain open.
- [ ] **ROMDATA-04-EUR-REDSPRING-ANCHORED-LITERALS-LINUX:** validate affected
  readers, boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-REDSPRING-ANCHORED-LITERALS-USA:** locate and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-REDSPRING-SWITCH-DESTINATIONS scoped EUR/Wine:** E223
  serves ten arrays/148 destinations. Reader `wd1z4xka`, 600 altered-source
  Red Spring frames `hg3vf9nv`, CLI `fzyln36o` and reconciliation `c_2f3g_b`
  pass. [Evidence](../rom_data_runtime.md#e235-red-spring-switch-destinations).
- [ ] **ROMDATA-04-EUR-REDSPRING-SWITCH-DESTINATIONS-OTHER:** establish other
  callers, selectors, scenes, real-BIOS and UI save/Load; original guards do not
  establish global selector bounds. Other families and source extents remain open.
- [ ] **ROMDATA-04-EUR-REDSPRING-SWITCH-DESTINATIONS-LINUX:** validate affected
  readers, boundaries, transfers, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-REDSPRING-SWITCH-DESTINATIONS-USA:** locate and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-04-EUR-REDSPRING-OBSERVED-LITERALS scoped EUR/Wine:** E222-d221
  publishes and serves 196 words. Reader `4sf2ed90`, 600 altered-source Red
  Spring frames `8ze5sj_t`, CLI `wdd7lpip` and reconciliation `pz3zxelt` pass.
  [Evidence](../rom_data_runtime.md#e233-red-spring-observed-thumb-literals).
- [ ] **ROMDATA-04-EUR-REDSPRING-OBSERVED-LITERALS-OTHER:** establish other
  callers, source extents, scenes, real-BIOS and UI save/Load; scopes may nest.
  E223–E226 serve the other 42 sampled words; broader families remain open.
- [ ] **ROMDATA-04-EUR-REDSPRING-OBSERVED-LITERALS-LINUX:** validate affected
  extraction, readers, boundaries, scene and CLI on Linux when selected.
- [ ] **ROMDATA-04-EUR-REDSPRING-OBSERVED-LITERALS-USA:** locate and authenticate
  corresponding USA sources; native Windows remains unverified.
- [x] **ROMDATA-01-E203-REDSPRING-CODE-SUPPORT scoped static join:** `usda5fn4`
  verifies source hashes and earlier executed sites for 93 literal words and
  12 control-table words overlapping the E172 sample. It does not establish
  the E172 execution sites or remove the raw dependency. The temporary source
  script is no longer retained. [Evidence](../rom_data_compact_catalog.md#e203-cross-reference-with-earlier-code-support-observations).
- [ ] **ROMDATA-01-E203-REDSPRING-OTHER:** classify the remaining sampled reads
  and establish their actual consumers and any alternative uses as needed;
  Linux, USA and native Windows remain deferred.
- [x] **ROMDATA-01-E172-REDSPRING-AF271C scoped EUR/Wine:** E177 identifies a 102-byte
  compressed tilemap, including 100 sampled unowned bytes. Source `yiosygyx`,
  Wine trace `r7r9ungh`, original reader/publication `798svo0e` and altered-
  source 600-frame action `mpurdf2y` and production CLI `it5komaw` pass.
  [Evidence](../rom_data_graphics.md#e177-observed-red-spring-tilemap).
- [ ] **ROMDATA-01-E177-TILEMAP-OTHER-CONSUMERS:** identify other assets,
  modes and readers without assuming guest bounds or a complete tilemap family;
  E197 owns eight words at `3a4d84`; other indices and descriptor rows remain open.
- [x] **ROMDATA-01-E197-TILEMAP-3A4D84 scoped EUR/Wine:** source and original
  reader audit `wrir7wj7`, publication/indexed readers `i23_yk35` and CLI
  `q7a_34j2` pass. The reader log contains a D-Bus assertion before successful
  completion. The eight physical words do not bound the original guest index.
  [Evidence](../rom_data_graphics.md#e197-indexed-tilemap-pointers).
- [ ] **ROMDATA-01-E197-TILEMAP-LINUX:** validate publication, indexed readers,
  mixed boundaries and CLI on Linux when selected.
- [ ] **ROMDATA-01-E197-TILEMAP-USA:** locate and authenticate the corresponding
  USA table and readers; native Windows remains pending.
- [ ] **ROMDATA-01-E197-TILEMAP-OTHER:** establish ordinary use and other
  consumers without imposing guest bounds. E198 owns the seven pointed-to
  streams; ordinary use through this indexed-pointer path is still open.
- [x] **ROMDATA-01-E198-TILEMAP-PAYLOADS-AF2780 scoped EUR/Wine:** original
  decoder `nuq8mgl_`, source `2llwimo3`, readers `d97f6l58` and CLI `gfc_c1yi`
  pass for seven streams. The reader log contains a D-Bus assertion before
  successful completion. [Evidence](../rom_data_graphics.md#e198-additional-compressed-tilemaps).
- [ ] **ROMDATA-01-E198-TILEMAP-LINUX:** validate publication, compressed
  readers, mixed boundaries and CLI on Linux when selected.
- [ ] **ROMDATA-01-E198-TILEMAP-USA:** locate and authenticate the corresponding
  USA streams and readers; native Windows remains pending.
- [ ] **ROMDATA-01-E198-TILEMAP-OTHER:** establish ordinary use of the other six
  streams and investigate other readers and tilemaps without assuming a complete
  catalog; update older
  revision-pinned tilemap fixtures when their checks are selected.
- [x] **ROMDATA-01-E199-TILEMAP-DESCRIPTORS-3A4B78 scoped EUR/Wine:** source
  and extracted dependencies `uottnw22`, Wine readers `9ofg9nm8` and CLI
  `0bqdvd05` pass. [Evidence](../rom_data_graphics.md#e199-direct-tilemap-descriptors).
- [ ] **ROMDATA-01-E199-TILEMAP-DESCRIPTORS-LINUX:** validate publication,
  dependencies, readers, mixed boundaries and CLI on Linux when selected.
- [ ] **ROMDATA-01-E199-TILEMAP-DESCRIPTORS-USA:** locate and authenticate the
  corresponding USA descriptors; native Windows remains pending.
- [ ] **ROMDATA-01-E199-TILEMAP-DESCRIPTORS-OTHER:** establish ordinary use of
  the other six descriptors and investigate other rows and consumers without
  imposing guest bounds. E200
  publishes seven rows selecting the E199 descriptors.
- [x] **ROMDATA-01-E200-TILEMAP-ROWS-3A4C04 scoped EUR/Wine:** source and
  extracted descriptor dependencies `u4ror4sy`, Wine readers `29if0sg2` and
  CLI `irh5mvpj` pass. The reader log contains a D-Bus assertion before
  successful completion. [Evidence](../rom_data_graphics.md#e200-direct-tilemap-rows).
- [ ] **ROMDATA-01-E200-TILEMAP-ROWS-LINUX:** validate publication, descriptor
  dependencies, readers, mixed boundaries and CLI on Linux when selected.
- [ ] **ROMDATA-01-E200-TILEMAP-ROWS-USA:** locate and authenticate equivalent
  USA rows and readers; native Windows remains pending.
- [ ] **ROMDATA-01-E200-TILEMAP-ROWS-OTHER:** establish ordinary use of the other
  six rows and investigate other consumers without imposing guest bounds. E201 publishes the seven
  adjacent interleaved LZ rows.
- [x] **ROMDATA-01-E201-INTERLEAVED-LZ scoped EUR/Wine:** source `w13041vc`,
  readers/HLE `0vq42lsj` and CLI `7l8ix18e` pass for seven interleaved chains.
  The reader log contains a D-Bus assertion before successful completion.
  [Evidence](../rom_data_graphics.md#e201-interleaved-lz-chains).
- [ ] **ROMDATA-01-E201-INTERLEAVED-LZ-LINUX:** validate publication,
  dependencies, readers, LZ HLE, mixed boundaries and CLI on Linux.
- [ ] **ROMDATA-01-E201-INTERLEAVED-LZ-USA:** locate and authenticate the
  corresponding USA chain; native Windows remains pending.
- [ ] **ROMDATA-01-E201-INTERLEAVED-LZ-OTHER:** establish ordinary use of the other
  six chains and investigate other consumers and real-BIOS/DMA paths; update
  revision-pinned LZ fixtures.
- [x] **ROMDATA-04-E202-INTERLEAVED-ORDINARY scoped EUR/Wine:** source audit `9jydyc_i` traces
  one EUR initializer to the row-list task. Under a zero initial row index and
  no later changes to its pointer, count or index, its nonempty path covers
  indices 0–3; the first E201/E200 rows are indices 6/7. This does not exclude
  other producers, computed pointers, later writers or contexts. The subsequent
  second-stage selector chooses a two-row pair; `_impqiwz` pins its arithmetic
  and the original task-ID parameter store. Retained Blue Screw Wine audit
  `dyl4e4jy` finds the first pair; `9qneu93s` confirms 600-frame equivalence
  with altered backing and positive resource/HLE counters. Both Wine logs
  contain a D-Bus assertion; both servers drained. Callback attribution remains
  unproved. Linux, USA and native Windows remain pending.
  [Evidence](../rom_data_graphics.md#e202-conditional-row-list-dataflow).
- [ ] **ROMDATA-04-E202-INTERLEAVED-OTHER:** verify other row pairs and scenes,
  callback attribution and remaining consumers without inventing inputs or
  imposing guest bounds; real-BIOS/DMA paths remain open.
- [ ] **ROMDATA-04-E202-INTERLEAVED-LINUX:** repeat the selected ordinary
  altered-backing proof on Linux when selected.
- [ ] **ROMDATA-04-E202-INTERLEAVED-USA:** authenticate and verify corresponding
  USA data and ordinary use; native Windows remains pending.
- [x] **ROMDATA-01-E204-BLUESCREW-RAW scoped source lead:** `isdh6mnt` and
  `t31eyd12` identify the observed pointer and rectangle and authenticate seven
  adjacent pointer cells with five distinct targets. The capture is address-only.
  [Evidence](../rom_data_graphics.md#e204-blue-screw-rectangle-source-lead).
- [x] **ROMDATA-02-E204-RECTANGLE-RESOURCES scoped EUR/Wine:** E202-d201
  extracts and serves the seven pointer cells and five rectangles. Wine
  `pf3lqjl7` covers original reader prefixes, aliases, boundaries and one DMA;
  `qfr83rk5` verifies the first pointer/rectangle in the recorded 600-frame
  Blue Screw scene with altered backing; production CLI `c3n9eh_b` passes.
  [Evidence](../rom_data_graphics.md#e205-additional-rectangle-pointers-and-targets).
- [ ] **ROMDATA-02-E205-RECTANGLE-OTHER:** establish ordinary use of the other
  rectangles, other readers and selectors, real-BIOS and other lifecycle behavior;
  update revision-pinned fixtures when their checks are selected.
- [x] **ROMDATA-05-E206-RECTANGLE-LIFECYCLE scoped EUR/Wine:** the targeted
  test passes Load/Undo, backup restoration and restart with an unavailable
  private dataset copy, retaining the owner and serving the selected resources.
  Original unit `fivnb6pa` remains FAIL because its private-input cleanup gate
  refused a synthetic `.sav`; recovery `za2m7ukk` verifies the child PASS and
  drained Wine without replay. [Evidence](../rom_data_graphics.md#e206-rectangle-resource-lifecycle).
- [ ] **ROMDATA-05-E206-RETAINED-SCRATCH:** separately review the retained
  failed unit's test-owned scratch if cleanup is later selected; preserve the
  safety gate and do not rerun gameplay merely to retire it.
- [ ] **ROMDATA-02-E204-RECTANGLE-LINUX:** validate extraction, readers,
  boundaries, ordinary use and CLI on Linux when selected.
- [ ] **ROMDATA-02-E204-RECTANGLE-USA:** authenticate corresponding USA data;
  native Windows remains pending.
- [x] **ROMDATA-01-E178-TILEMAP-ROW-3A4BEC scoped EUR/Wine:** E179 publishes the observed
  12-byte row selecting the E178 descriptor. Wine trace `xvj_6rmw`, source
  `cc9b5_to`, original reader/publication `fhgchuk_` and altered-source
  600-frame scene `4y6kd_rg` and production CLI `n7qzldpv` pass.
  [Evidence](../rom_data_graphics.md#e179-observed-tilemap-row).
- [ ] **ROMDATA-01-E179-ROW-OTHER-CONSUMERS:** identify other rows, selectors
  and readers without assuming a global guest bound.
- [x] **ROMDATA-01-E179-TILEMAP-ROWS-3A4BBC scoped EUR/Wine:** E180 extends the selected row
  resource with four adjacent observed rows. Source `5w43a4ri`, Wine trace
  `atgqm66z`, original reader/publication `jcmdanqw` and 600-frame scene
  `4nxkgjnp` and production CLI `ilg230s3` pass.
  [Evidence](../rom_data_graphics.md#e180-adjacent-observed-tilemap-rows).
- [x] **ROMDATA-01-E180-ROW-TARGETS scoped EUR/Wine:** E181 migrates the four
  descriptor targets and linked LZ wrappers. Source `l3pwspy8`/`e90r8zmw`,
  Wine trace `0_8a03d2`, reader `fd2bih7u`, 600-frame scene `pdfkq9g5` and
  CLI `3tjo6qvr` pass.
  [Evidence](../rom_data_graphics.md#e181-observed-tilemap-descriptors-and-lz-wrappers).
- [ ] **ROMDATA-01-E180-ROW-TARGETS:** identify other consumers, scenes and
  descriptors outside the selected EUR/Wine evidence; aggregate scene counters
  do not attribute ordinary use to every migrated descriptor.
- [x] **ROMDATA-01-E172-REDSPRING-399774 scoped EUR/Wine:** E182 owns three
  transfer rows, descriptors and linked LZ wrappers. Source `g0ce9o5s`, Wine
  trace `qu75rzi6`, reader `fjrrnhsl`, 600-frame scene `fx2_p2kt` and CLI
  `8dilrn0i` pass. The scene log contains a D-Bus assertion; the test completed
  and passed. Earlier wildcard, protected-source and reader-assertion failures
  are retained in the evidence.
  [Evidence](../rom_data_graphics.md#e182-observed-transfer-rows-and-lz-descriptors).
- [ ] **ROMDATA-01-E172-REDSPRING-399774:** identify other consumers and
  transfer records outside the selected EUR/Wine evidence.
- [x] **ROMDATA-01-E172-REDSPRING-AE789C scoped EUR/Wine:** E183 owns two
  authenticated halfword rectangles. Source `n257xucg`, Wine trace `re5n0s9w`,
  reader `rxrkgl9k`, 600-frame scene `qz9v5fto` and CLI `grtrq8pm` pass. The
  scene log contains a D-Bus assertion; the test completed and passed.
  [Evidence](../rom_data_graphics.md#e183-observed-tilemap-rectangles).
- [x] **ROMDATA-01-E183-RECTANGLE-ROOTS scoped EUR/Wine:** E184 owns the
  selected descriptor/list and second rectangle pointer. Source `46yea8p1`,
  Wine traces `_vvgz2qn`, readers `5kk84qdw`, 600-frame scene `kavnisxk`
  and CLI `vdp3mixx` pass.
  [Evidence](../rom_data_graphics.md#e184-observed-rectangle-roots).
- [ ] **ROMDATA-01-E172-REDSPRING-AE789C:** identify other roots, rectangles,
  selectors and consumers outside the selected EUR/Wine evidence.
- [x] **ROMDATA-01-E184-RECTANGLE-LIST-ROW scoped EUR/Wine:** E185 owns the
  selected row and validates its E184 list dependency. Source `a9cjlk1d`, Wine
  trace `2iw6p25i`, readers `ho6q2xxd`, 600-frame scene `xrk2v6ks` and CLI
  `e42p10cn` pass.
  [Evidence](../rom_data_graphics.md#e185-observed-rectangle-list-row).
- [ ] **ROMDATA-01-E185-LIST-ROW-LINUX:** repeat publication, original readers,
  adjacent boundary, affected scene and CLI on Linux.
- [ ] **ROMDATA-01-E185-LIST-ROW-USA:** locate and authenticate an equivalent
  USA row and validate its consumers; native Windows remains pending too.
- [ ] **ROMDATA-01-E184-RECTANGLE-ROOTS-LINUX:** repeat publication, original
  readers, mixed boundaries, affected scene and CLI on Linux.
- [ ] **ROMDATA-01-E184-RECTANGLE-ROOTS-USA:** locate and authenticate equivalent
  USA roots and validate their consumers; native Windows remains pending too.
- [ ] **ROMDATA-01-E183-RECTANGLES-LINUX:** repeat publication, original reader,
  boundaries, affected scene and CLI on Linux.
- [ ] **ROMDATA-01-E183-RECTANGLES-USA:** locate and authenticate equivalent USA
  rectangles and validate their consumers; native Windows remains pending too.
- [ ] **ROMDATA-01-E182-TRANSFER-LINUX:** after EUR/Wine acceptance, repeat
  publication, original readers, mixed boundaries, affected scene and CLI on Linux.
- [ ] **ROMDATA-01-E182-TRANSFER-USA:** locate and authenticate equivalent USA
  sources and validate their consumers; native Windows remains pending too.
- [ ] **ROMDATA-01-E181-DESCRIPTORS-LINUX:** repeat publication, original
  readers, mixed wrapper/LZ boundaries, affected scene and CLI on Linux.
- [ ] **ROMDATA-01-E181-DESCRIPTORS-USA:** locate and authenticate equivalent
  USA descriptors and wrappers and validate their consumers; native Windows
  remains pending too.
- [ ] **ROMDATA-01-E180-ROWS-LINUX:** repeat row publication, readers, scene
  and CLI checks when Linux coverage resumes.
- [ ] **ROMDATA-01-E180-ROWS-USA:** authenticate any equivalent USA sources
  and validate their consumers; native Windows remains pending too.
- [ ] **ROMDATA-01-E179-ROW-LINUX:** repeat row publication, reader, scene and
  CLI checks when Linux coverage resumes.
- [ ] **ROMDATA-01-E179-ROW-USA:** authenticate any equivalent USA source and
  validate its consumers; native Windows remains pending too.
- [x] **ROMDATA-01-E177-TILEMAP-DESCRIPTOR-3A4B70 scoped EUR/Wine:** E178 authenticates the
  8-byte descriptor and publishes it with its tilemap dependency. Wine trace
  `i1a1chh7`, source `vapbrkwo`, reader/publication `ve3ggqgj` and 600-frame
  scene `7tvsftjk` and production CLI `2aiu_s0o` pass.
  [Evidence](../rom_data_graphics.md#e178-observed-tilemap-descriptor).
- [ ] **ROMDATA-01-E178-DESCRIPTOR-LINUX:** repeat descriptor publication,
  original reader, affected scene and CLI checks when Linux coverage resumes.
- [ ] **ROMDATA-01-E178-DESCRIPTOR-USA:** locate and authenticate an equivalent
  USA descriptor and validate its consumers; native Windows remains pending.
- [ ] **ROMDATA-01-E177-TILEMAP-LINUX:** repeat publication, original reader,
  affected scene and CLI checks on Linux when platform coverage resumes.
- [ ] **ROMDATA-01-E177-TILEMAP-USA:** locate and authenticate any equivalent
  USA source and validate its consumers; native Windows remains pending too.
- [x] **ROMDATA-01-E175-REDSPRING-14D1F0 scoped EUR/Wine:** E176 owns the
  authenticated 64-byte constructor template. Source `n33tj6pd`, poisoned
  native/interpreter copy and publication `nibv6hif`, and the altered-source
  600-frame action `2efs1kas`, and production CLI `0qyhd6ao` pass. Static constructor evidence and the scene
  inventory do not by themselves identify the scene's dynamic caller.
  [Evidence](../rom_data_gameplay.md#e176-european-constructor-template-at-14d1f0).
- [ ] **ROMDATA-01-E176-CONSTRUCTOR64-OTHER-CALLERS:** identify live-record
  field meanings and additional callers/scenes; the original guest retains
  allocation and writes. Do not infer a caller from the address alone.
- [ ] **ROMDATA-01-E176-CONSTRUCTOR64-LINUX:** repeat publication, copy and
  affected scene/CLI checks on Linux when platform coverage is resumed.
- [ ] **ROMDATA-01-E176-CONSTRUCTOR64-USA:** locate and authenticate any
  equivalent USA resource and validate its consumers; no USA equivalence is
  assumed. Native Windows validation remains separately pending.
- [x] **ROMDATA-01-E172-REDSPRING-OTHER-LZ scoped EUR/Wine:** E175 publishes
  six additional LZ77 streams. Source `zi6u7skx`, publication/SWI and input
  guards `_42f690d`, altered-source 600-frame action `6jqo8wyd` and CLI
  `fonccar1` pass. The initial publication `f0e28tia` reached the earlier
  aggregate input bound; the corrected finite bound and per-file guards pass.
  [Evidence](../rom_data_graphics.md#e175-additional-red-spring-lz77-streams).
- [ ] **ROMDATA-01-E172-REDSPRING-OTHER-LZ-CONSUMERS:** identify the six
  assets and other callers; verify direct DMA and real-BIOS consumption, plus
  Linux, USA and native Windows, before claiming wider coverage.
- [x] **ROMDATA-01-E172-REDSPRING-B0E2B8 scoped EUR/Wine:** E174 publishes
  the observed LZ77 block and serves both guest source reads and decoded SWI11
  output. Publication/SWI `xlo4jdo9`, altered-source 600-frame action
  `c_ce4pye` and production CLI `mlc8meas` pass.
  [Evidence](../rom_data_graphics.md#e174-observed-red-spring-lz77-block).
- [ ] **ROMDATA-01-E172-REDSPRING-B0E2B8-OTHER-CONSUMERS:** identify the asset
  and other callers; verify direct DMA and real-BIOS consumption, plus Linux,
  USA and native Windows, before claiming wider coverage of this block.
- [x] **ROMDATA-01-OBJ-6CB278-PRIMARY scoped EUR/Wine:** E172 Wine
  `_5qb5ej3` passes publication, mixed boundaries and ten controlled original
  prefixes for the five conditional primary-alias bytes. Production Wine CLI
  create/reuse/verify passes (`lhle9dmz`).
  [Evidence](../rom_data_compact_catalog.md#conditional-primary-task-alias-for-6cb278).
- [ ] **ROMDATA-04-OBJ-6CB278-PRIMARY-ORDINARY:** establish an ordinary primary
  ID-362 invocation and its complete callback/consumers before claiming normal
  use. Other selectors, Linux, USA and native Windows remain open.
- [ ] **ROMDATA-E172-DEFERRED-REGRESSIONS:** update or explicitly preserve
  E171-pinned Wine fixtures before rerunning them with E172, then run affected
  Linux checks when Linux validation is selected. USA and native Windows remain
  separate pending platforms.
- [ ] **ROMDATA-E171-DEFERRED-REGRESSIONS:** update or explicitly preserve the
  revision-pinned historical E170 Wine fixtures before rerunning them against
  E171; run the corresponding affected Linux checks when Linux validation is
  selected. USA and native Windows remain separate pending platforms.
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
  payloads. Conditional RAM-slot lists `395172`/`39517a` now have source and
  original-writer model evidence, plus an isolated original slot-consumer draw
  comparison for both lists. E129-d128 publishes them, with guarded EUR/Wine
  lifecycle and original-interpreter and translated-native managed-read
  checks. The affected E129 regression matrix passes under RVR-23; ordinary
  use remains open. A distinct source-pinned discovery identifies 16 layered
  OBJ/sequence pairs through descriptor table `3ae1a4`; a strict source model
  and 16 original initializers pass under Wine. E130 managed publication and
  source reads, initializers, initial-frame interpreter/translated draw and the
  affected regression matrix pass; complete animation and ordinary evidence
  remain open. Do not reopen the broad experiment with 61
  truncations. [Source contract](../rom_data_obj.md#conditional-layered-obj-animation-source-at-3ae1a4).
  [Current selections and limits](TASK_HANDOFF.md#rom-data-full-eur).

## 6. Keep explicitly deferred scope visible

These items remain open outside the current EUR/Wine functional selection.
They require an explicit scope change or their stated prerequisite; free disk
space by itself does not select them.

- [ ] **Linux:** validate changed ROM integrations, including the E134 ordinary
  updater, title-language routes, descriptor-1 alias and indexed graphics
  contexts, text substitution callbacks, bounded range lookup, tilemap
  geometry and the E163–E170 pose, archive, callback and text-reader resources
  on native Linux when selected; retain older accepted Linux results.
- [ ] **Wine sandbox access:** if sandbox-only Wine execution is required,
  provide access to its runtime directory and Unix-domain socket binds. The
  runner now refuses these known restrictions before leasing the prefix;
  automatic approval-review availability remains outside the project. An
  interrupted E170 launch left a separate cold-storage lease; its exact marker
  was preserved after host reboot and catalog review (`lease-recovery-20261118`).
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
