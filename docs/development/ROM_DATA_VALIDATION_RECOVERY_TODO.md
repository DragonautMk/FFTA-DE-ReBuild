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

- [ ] **ROMDATA-01-POST-INDEXED-LOWER-READERS:** attribute the remaining 536
  lower-ROM bytes not matched by prior source witnesses in the E170 post-indexed
  Wine inventory (`108ejhdv`, reconciliation `c8paxoub`). Thirty-two bytes at
  `14d2dc..14d2fc` have a bounded table contract (`9ei8p1cy`, `s94ilj14`) and
  E171 managed ownership (`8bb1ykny`). Remaining unmatched addresses may be
  executable support; the inventory alone does not prove an asset.
  [Evidence](../rom_data_runtime.md#e170-post-indexed-rom-read-inventory).
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
- [ ] **ROMDATA-01-E172-REDSPRING-OTHER-RAW:** classify the 8,789 previously
  observed unowned bytes outside the E173 bank increment. E174 migrates 2,578
  and E175 another 2,291. E176 owns another 64, E177 another 100, E178
  another 7, E179 another 12, E180 another 48, E181 another 28, E182
  another 58 and E183 another 86; the other 3,517 require classification before
  claiming the action has no remaining raw source. Other scenes,
  consumers, Linux, USA and native Windows remain pending.
- [x] **ROMDATA-01-E172-REDSPRING-AF271C scoped EUR/Wine:** E177 identifies a 102-byte
  compressed tilemap, including 100 sampled unowned bytes. Source `yiosygyx`,
  Wine trace `r7r9ungh`, original reader/publication `798svo0e` and altered-
  source 600-frame action `mpurdf2y` and production CLI `it5komaw` pass.
  [Evidence](../rom_data_graphics.md#e177-observed-red-spring-tilemap).
- [ ] **ROMDATA-01-E177-TILEMAP-OTHER-CONSUMERS:** identify other assets,
  modes and readers without assuming guest bounds or a complete tilemap family;
  the table at `3a4d84` and other descriptor rows remain raw.
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
- [ ] **ROMDATA-01-E172-REDSPRING-AE789C:** migrate or classify the raw pointer
  cells at `3a4b08` and `3a4d3c`, and identify other rectangles and consumers.
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
