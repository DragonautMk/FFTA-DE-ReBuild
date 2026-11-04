# Task handoff

## Operational status

English-first project with optional Italian. Accepted report/guide/export scopes remain recorded; the cross-tree audit remains open.

Read this file, then only relevant linked sections. Do not preload archives, CHANGELOG, PROJECT_STATE or large TODOs. Retired artifacts do not invalidate accepted evidence. Rules: [MEMORY_WORKFLOW](MEMORY_WORKFLOW.md#resume-and-repeat-decisions).

Execution state and acceptance outcome are separate: `active`, `suspended`,
`interrupted`, `failed`, `waiting`; open outcomes are `partial`,
`integrated_unverified`, `not_started`. Completed/rejected tasks move to the
archive after acceptance checks; an open parent never reopens closed children.
Here `suspended` means outside this session's selection, without authorizing
termination of another owner's process or changing another thread's goal state.

## Running or suspended tasks

### U31-P0-UPSTREAM-AUDIT

- State: suspended
- Outcome: partial
- Verified: U31-02: 7 replay/10 core tests, USA/EUR GUI and Right interruptions; EUR native category, Right and pointer-exit comparisons match; shop hook observed. Decisions: upstream-audit/U31-approved-decisions.md.
- Missing: U31-02 other GUI pointer/native cancellation paths and Windows; menu fix, Party device, autosave and other audit. Approval is not acceptance.
- Resume: branch `codex/u31-shop-replay-wip` in `/tmp/u31-shop-replay-wip`; private evidence `target/u31-shop-replay-repair-B9AOtX/`. Reconciliation deferred; resume only on request.

### ENGLISH-FIRST-PROJECT

- State: active
- Outcome: partial
- Verified: previous runtime/report/guide/export units accepted, including shared debug guides. Command/event consoles and operation cards use English/Italian presentation; targeted tests PASS. Audit: 356 reviewed files, 9 Markdown pending; upstream decision plan translated with legacy anchors.
- Missing: remaining documentation (ROM guide e11–e25, e28–e30, e33–e34, e37–e41, e44–e61 and e66–e76 clarified; rest pending), other dynamic report producers/metadata, helper/test modules, tools/build text, source comments and full audit across both trees. Planned-task checklist/reference translated and synchronized. Preserve optional locales, imported content and immutable history. Focus choice diagnostics follow English/Italian; preserve active DBG2 work. DBG2 checklist and TSV matched at last review; reconcile future concurrent changes. The handoff audit hash is volatile; saved ROM coverage JSON/HTML reconciled. Do not repeat the lost character diff.
- Launcher: foundation and play/input/audio/video authored text reviewed in English; dynamic values and runtime/platform behavior remain unverified. Evidence: `docs/changes/2026-11-03-review-launcher-screens-english.json`.
- Resume: ENGLISH-FIRST-INVENTORY-REVIEW partial: map guide reviewed; ROM guide e26/e27/e31/e32, packet intro and extracted-files/validation text clarified. Earlier inventory and remaining ROM guide sections need review. Evidence: `docs/changes/2026-11-03-clarify-rom-guide-metadata-and-packets.json`.

### ROM-DATA-FULL-EUR

- State: waiting
- Outcome: partial
- Verified: E127 100 resources/101 files, 63 banks; EUR Wine create/reuse, nibble reads/fallback, four native comparisons and 12-frame CLI PASS (zero catalog reads). Current: target/validation/units/rom-data-e127-nibble-current-evuagp74/work/resources/. Prior scoped proofs preserved.
- Missing: unmigrated resources/consumers, missing ordinary-use evidence, ABI/global extent/indirect calls and mod phases ROMDATA-06/07; Linux/USA/native Windows/new32:9 remain distinct. EUR, reduced Wine scope, no performance/space measurements; originals and personal saves unchanged.
- Selection: `ROMDATA-04-LOOP-COMPACT-CATALOG-ORDINARY` active/partial. EUR/Wine, Linux/USA pending, no metrics. Soil Evidence 6bac28/6c0614, Freezeblink 6babc4 and Stardust 6bb574 exact reads and paired Wine scenes PASS. Audit CLI compilation recovered after ENOSPC; emitted test passed. Missing: remaining banks and global coverage. Selection: `ROMDATA-02-04-6C1E78-DEFAULT-SELECTORS` partial. Caller branch also derives 4/6, while only conditional 3/5 are represented. Next: Wine source geometry and current-manifest conflict check; if coherent, extend catalog and validate. EUR/Wine only; no metrics. Model/effort unconfirmed.
- Recovery selection: `RVR-01` active/partial. Octonary, UI-context and table-graphics reports recovered; RVR-14/15/16/17 selected EUR/Wine tests pass, RVR-15 independently verified after unit retention FAIL. Missing: other stale reports and filters. Next: select one and verify predecessor inputs.
- Resume: [validation recovery checklist](ROM_DATA_VALIDATION_RECOVERY_TODO.md) for skipped checks; [ROM_DATA_TODO](ROM_DATA_TODO.md), only the selected criterion's section (ROMDATA-01/02/03/04/05/06/07); [inventory](ROM_DATA_INVENTORY.md). Do not read the whole TODO. Children retain their IDs and limits:
  - `ROMDATA-04-PARAMETER-RECORDS27-ORDINARY`: source/model/runtime/CLI accepted; 59 records observed, full domain unproven; positive normal use and complete paired evidence missing. Eleven negative captures closed; archive legacy lines 17624..17672 and `docs/rom_data_gameplay.md`, ordinary sparse parameter record reachability section.
  - `ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY`: 40 banks/8 tables source/model/runtime/CLI accepted; positive normal trigger and paired evidence missing; negative attempts remain closed. `docs/rom_data_ui_graphics.md`, ordinary table extension reachability section; archive 17844..17864.
  - `ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY` and `ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN`: conditional fragments/paths accepted; global domain and ordinary invocation unproven. Unit320 negative closed; later state2/state4/selector-witnesses diagnosis closed without a scene. Archive 59..75 and 789..866; no blind navigation.
  - `ROMDATA-01-OBJ-REMAINING-DRAW-ROOTS`, `ROMDATA-01-OBJ-REMAINING-CONSTRUCTOR-SOURCES`, `ROMDATA-01-OBJ-GRAPHICS-PAYLOADS`: authenticated increments accepted; global inventory/remaining consumers open. Do not repeat the broad experiment with 61 truncations. Archive 2853..2874, then only the selected family's record.
  - `ROMDATA-05-OBJ-VALIDATION`: local equivalence/costs, GUI, archive and USA initializer have later accepted outcomes; native Windows gates, further catalogs/scenes and normal enablement remain. Archive 3945..4087. Old artificial RSS/space constraints and `ROMDATA-05-BG-SCENES-COSTS` do not authorize further measurements: superseded by EUR/no-metrics steering.
  - Compact OBJ e101..e108: conditional prefixes/model/runtime/CLI accepted. e101..e107 ordinary use remains open; record222/candidate-table and selector33 have only static conditions/bounded negatives. e108/6cc124 has one IN-19 Roulette 600-frame positive native pair; complete animation/ordinary selector-list domain remains unproved. Archive 11..65 and new ordinary-use record; no accepted replay repeated.
  - UI context templates e109: source selector/copy catalog and one ordinary20B copy accepted; complete UI extent/all consumers/all11 records in ordinary play remain unproved. docs/rom_data_ui.md. Do not repeat closed12-frame proof for context.
  - ROMDATA-04-COMPACT-CATALOG-ORDINARY: StarCross ordinary proof closed: IN08Douglas secondary9/mastered84, positive15830reads/19330B and480frame full native pair. Complete29-root/allselector/animation/consumer coverage remains open; counters aggregatecatalog. Prior303/310/312 human leads excluded. docs/rom_data_compact_catalog.md.
  - `ROMDATA-04-LOOP-COMPACT-CATALOG-ORDINARY`: five-loop source/model/native/CLI accepted; Soil Evidence 6bac28/6c0614, Freezeblink 6babc4 and Stardust 6bb574 exact reads with paired Wine scenes PASS. Other banks, complete animation and global domains remain open. docs/rom_data_compact_catalog.md.

## Incomplete tasks and open criteria

DBG2 audit of 2026-11-01 found eleven parents 13..23 unchecked;
8 had partial/integrated work and 3 lacked the requested implementation.
DBG2-13 and DBG2-14 have since completed; current open criteria follow.
Original criteria and audit notes: [DEBUG_TOOLS_V2_TODO](DEBUG_TOOLS_V2_TODO.md#p2--console-di-comandi-e-ricette-per-lassistente).
DBG2-01..12 remain complete within their scope. Read historical details only
for a specific question: archive legacy lines 179..744 or the named record below.

### DBG2-15

- State: active
- Outcome: partial
- Verified: context, retry, discovery, replay/cancel, live refresh and extended live search context at 480x360 with exact cancellation in EUR Wine. `2026-11-04-show-live-extended-search-context.json`.
- Missing: full operation parity and comprehensive narrow-window presentation; native/physical input. DBG2-14 prerequisite accepted.
- Selection: user requests task completion; audit supported action families and presentation against accepted DBG2-13/15 evidence and identify exact gaps.
- Resume: complete remaining parity without repeating accepted replay or search context. Private proofs retained.

### DBG2-16

- State: active
- Outcome: partial
- Verified: assertions, waits, catalog/captures, bounded searches including an EUR Wine Thumb hit, Start/cancel, interrupted recovery, safe restart refusal and interactive recipe/session picker. `2026-11-04-verify-recipe-breakpoint-hit.json`.
- Missing: other action types, feature-gated watch, broader picker scenarios, cross-restart recovery and wider platforms; ambiguous queries not retried.
- Resume: select an unverified criterion without repeating accepted search or picker flows; private and failed proof units retained.

### DBG2-17

- State: suspended
- Outcome: partial
- Verified: readonly catalog/import/menu, bounded source index/CLI, and source-capability local query with generation, chunk/hash and CLI parity; `2026-11-03-index-cheat-source-capabilities.json`, `2026-11-03-expose-source-capability-local-control.json`.
- Missing: complete device/ROM and behavioral verification; source plans are assumptions.
- Resume: `docs/cheats.md`; do not repeat accepted index or channel checks.

### DBG2-18

- State: suspended
- Outcome: partial
- Verified: CodeBreaker subset, explicit plaintext/seed/source/master/CRC and seven static mGBA comparisons; October 31 inspection records. The source-bound index now exposes supported plans and unevaluated limits.
- Missing: complete per-CH ROM/device verification; unsupported opcodes/formats may remain rejected. A decoded master is not an executed hook.
- Resume: `docs/cheats.md`, `debug_cheat_decoder.rs`; do not duplicate the decoder.

### DBG2-19

- State: suspended
- Outcome: partial
- Verified: one-shot/CH-0334, maintained interpreter timing, conflict/disarm and active indicator; isolated System/Restart, save-state load and Undo disarm synthetic plans, load/Undo restore exact expected state. `2026-11-03-maintain-bounded-interpreter-cheats.json`.
- Missing: device/gameplay equivalence and broader memory/backend coverage; I/O writes, native/ROM and mirrors unsupported. Disarm does not itself restore memory.
- Resume: `docs/cheats.md`; do not repeat accepted synthetic checks or restoration paths.

### DBG2-20

- State: active
- Outcome: partial
- Verified: source/preview/arm/disarm, filters and named group save/select/preview/arm via page and commands. `docs/changes/2026-11-03-manage-cheat-presets-in-page.json`.
- Missing: validated values/ranges and broader game/device behavior.
- Resume: select a missing criterion; do not repeat the accepted preset-page flow.

### DBG2-22

- State: suspended
- Outcome: partial
- Verified: CH-0340 EXP and CH-0320 Steal: Ability each accepted in a EUR Party scene with paired control and exact restoration. CH-0330 plan conditional; source has no notes. Documented battle-reward checkpoint DS-13 is USA, not EUR; no suitable EUR reward scene identified.
- Missing: CH-0330 suitable EUR scene, before/after and side effects; other group entries, actual level-up and device-hook equivalence.
- Resume: identify a relevant EUR reward checkpoint before testing CH-0330; no blind scene navigation. `docs/changes/2026-11-03-verify-eur-ability-mastery.json`.

### DBG2-23

- State: waiting
- Outcome: not_started
- Verified: play.rs refuses new recordings/saves after one-shot pending provenance; debug_recording.rs/debug_replay.rs do not implement the requested cheat contract.
- Missing: set/hash/parameters/order/frame/decoder in checkpoints/reports, Load/Undo/reset/ROM semantics, state-clock replay and incompatible-set refusal.
- Resume: after DBG2-19/21; refusal guard is protection, not completed provenance/replay support.

### DBG2-24

- State: waiting
- Outcome: partial
- Verified: named trigger editor, counts/windows, RAM guard, pause/capture/freeze, authenticated capture identity and idempotent retry; EUR Wine hit/miss, actions and GUI PASS. `2026-11-04-authenticate-trigger-capture-and-retry.json`.
- Missing: shared DBG2-15 parent closure; native Windows, physical input and broader platform coverage remain under DBG2-27. Counted watch tested synthetically.
- Resume: finish DBG2-15 prerequisite before parent closure; preserve accepted trigger evidence and private proofs.

### DBG2-25

- State: waiting
- Outcome: not_started
- Verified: preceding foundations accepted; no recorded closure of parent 25, outside audit 13..23.
- Missing: observed values/differences pinned on screen; dependencies DBG2-03/15 and DBG-11/17.
- Resume: only criterion DBG2-25 in TODO section P4 upon a relevant request; no development selected now.

### DBG2-26

- State: waiting
- Outcome: not_started
- Verified: preceding foundations accepted; no recorded closure of parent 26, outside audit 13..23.
- Missing: timeline and exportable recipe; dependencies DBG2-11/16/23 and DBG-09/16.
- Resume: only criterion DBG2-26 in TODO section P4 upon a relevant request; no development selected now.

### DBG2-27

- State: waiting
- Outcome: not_started
- Verified: preceding foundations accepted; no recorded closure of parent 27, outside audit 13..23.
- Missing: targeted integration/regression matrix; dependencies on earlier criteria and distinct platforms/input paths.
- Resume: only criterion DBG2-27 in TODO section P4 upon a relevant request; no development selected now.

### DBG2-28

- State: waiting
- Outcome: not_started
- Verified: preceding foundations accepted; no recorded closure of parent 28, outside audit 13..23.
- Missing: guide, distribution and version compatibility; depends on DBG2-27 for distributed features.
- Resume: only criterion DBG2-28 in TODO section P4 upon a relevant request; no development selected now.

### PERFORMANCE-TODO

- State: suspended
- Outcome: partial
- Verified: infrastructure/replay/baselines and accepted PERF-03/05/06/08 optimizations; timer/audio, hardened frame boundaries, event allowance and selected timing checks have terminal results. Compact-ID/18-ROM-seeds/post-wait/row-bounds experiments remain rejected.
- Missing: rare original lag, real attribution/realtime overhead, hardware/native Windows/Wayland, physical lifecycle and PERF-01/02/04/07/09/10 criteria; PERF-11/12/13 conditional. `PERF-07-EXACT-ROM-HINTS` retains the old integration ID: metadata/cache, default/explicit equivalence, replay/GUI/export were not closed at that checkpoint. None of its execution selected now.
- Resume: only upon a performance request; [PERFORMANCE_TODO](PERFORMANCE_TODO.md#stato-dellesecuzione) for summary, archive legacy lines 4998..5049 and record 2026-10-21-preserve-native-event-boundaries for integration. Do not repeat accepted hardened/allowance/timing checks.

### DEBUG-SAVES-EXPANSION

- State: suspended
- Outcome: partial
- Verified: accepted imports/copies/boot/scenes remain accepted; old inventory of 6 valid/9 partial/11 uncovered is not the current global state. Later increments and user DS-20 closed; a GUI interruption does not prove normal-close.
- Missing: only scene subcases still uncovered/partial in the current catalog; IN05 File2, IN19 EUR wrapper and IN18 conversion remain unaccepted leads to reconcile before acting.
- Resume: only upon a relevant request; docs/development/debug saves.md and local catalog, record 2026-10-23-audit-imported-debug-saves, archive 2551..2573 and 2643..2736. Do not recreate fixtures/imports or explore scenes now.

### LOCAL-ARTIFACT-HYGIENE

- State: interrupted
- Outcome: partial
- Verified: October 26 applied.json records applied cleanup; later sweeps have separate closed receipts. No deletion in this reorganization.
- Missing: original task's final validation/memory closure unattested; later cleanups do not automatically prove all its criteria.
- Resume: only upon a cleanup request, check task receipts/limits before any documentation-only closure; archive 2486..2496 and `target/local-artifact-cleanup-2026-10-26/applied.json`. No new sweep to reconstruct context.

## Blockers, dependencies and shared resources

- DBG2 IME tests touched personal `~/.local/share/ibus-table/tables/cangjie3-user.db`; user informed, do not alter it. Unit de2hg3tl lost an Xvfb handle: host status unknown; do not signal by guess.

- A read-only coverage HTML character diff yielded with no retained handle on 2026-11-03; outcome/host status unknown. Do not restart or infer termination.

- Async work: the collaboration inventory observed on 2026-11-01 showed only `/root`; ps visibility is limited to the sandbox namespace. Recent isolated Linux test builds completed; no Wine/runtime job was started or signaled. Host/other-thread state is unobservable: do not assert global inactivity. Historical results prove their children's closure, not today's host state. Before resuming, check handle/process, output and owner; no response or a lost handle does not establish failure.
- Shared resources: checkout, handoff/archive/CHANGELOG, canonical targets, Wine prefixes, display, datasets and fixtures. Do not stop others' processes, restore source/binaries without ownership or clean live/failed tasks. Historical Wine retention audit: deleting a prefix requires coordination/host visibility, not sandbox ps. The 3 November cleanup removed 26.75 GiB; its receipt records host-visible selected-unit checks and preservation limits. Other tasks still require fresh coordination before cleanup.
- Memory writes: one coordinated writer; proposals with both preimage hashes and common lock through scripts/task_handoff.py update. If changed, reread and merge before publication. No background writer; no lock authorizes interference with other jobs. Details in MEMORY_WORKFLOW.
- Complete history: [TASK_COMPLETED_ARCHIVE](TASK_COMPLETED_ARCHIVE.md#legacy-handoff-snapshot), a provenance snapshot, not an operational queue. Old partial descriptions remain to prevent information loss: open tasks remain here, not promoted to closed. `TASK_HANDOFF_MIGRATION.json` records sections, hashes and terminal-outcome index. Search an ID and read only the necessary range.
- Godot: planned assessment complete, implementation deferred, not a current dependency. AndroidPort is separate; originals/ROM/BIOS/private catalogs/saves/recordings stay out of commits/packages.

## Final handover

- Task: RVR-15 completed for selected test; RVR-01 and ROM-DATA-FULL-EUR remain partial.
- Last verified result: e127 Wine test PASS; 101 manifest files, 100 resources, 14 attach guards, 22 native and three transfer cases, Load/Undo; independent input/result verification PASS.
- Remaining work: other report corrections and checklist filters; ordinary use and deferred platforms remain open.
- Errors or blockers: Wine validation unit FAIL on test-created ffstate during cleanup; work, diagnostics and build artifact retained. Two earlier fixture-omission failures retained; model effort unconfirmed.
- Next action: select another unchecked recovery item and verify predecessor inputs before execution.
- References: docs/changes/2026-11-04-verify-e127-table-graphics.json; target/validation/units/rvr15-table-graphics-result-check-qey5s95l/.
