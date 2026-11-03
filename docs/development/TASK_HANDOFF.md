# Task handoff

## Operational status

Updated 2026-11-02. User confirmed English-first project with optional Italian. Runtime/offline
reports, guide reviews and portable/private exporter instructions are accepted
within their recorded scopes; the cross-tree audit remains open.

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

### ROMDATA-03-04-E105-SENARY-COMPACT-OBJ

- State: active
- Outcome: integrated_unverified
- Verified: source/756-byte sparse model PASS Wine, two spans/gap2; e105 has 95 resources. Read-only CI repair found Windows test compilation PASS/exit0/no messages in build-result.json, with both integration hashes matching selection.json. This supersedes recorded build session51589's stale next action, not proof of host inactivity.
- Missing: runtime acceptance of publication, loader authentication and composition preserving gaps, inputs and previous resources; no runtime result.json exists in the unit. Compiling an ignored test is not executing it.
- Resume: only on a relevant ROM request, check owner/handle/output before prepared Wine runtime test. target/rom-data-compact-obj-senary-runtime retains evidence; fallback81 remains statically verified. Scope: docs/changes/2026-11-02-record-partial-senary-compact-obj-integration.json. No runtime work selected here.

### ROM-DATA-FULL-EUR

- State: suspended
- Outcome: partial
- Verified: e105-d104, 95 resources/96 files; CLI create/reuse/startup and 98 protected files PASS in `target/rom-data-compact-obj-quinary-cli/result.json`, read during the audit. Neutral 24755→24756 with new counters zero does not prove ordinary use. Do not repeat accepted source/model/runtime/CLI checks or closed negative attempts.
- Missing: unmigrated resources/consumers, missing ordinary-use evidence, ABI/global extent/indirect calls and mod phases ROMDATA-06/07; Linux/USA/native Windows/new32:9 remain distinct. EUR, reduced Wine scope, no performance/space measurements; originals and personal saves unchanged.
- Resume: [ROM_DATA_TODO](ROM_DATA_TODO.md), only the selected criterion's section (ROMDATA-01/02/03/04/05/06/07); [inventory](ROM_DATA_INVENTORY.md). Do not read the whole TODO. The following children retain their IDs and limits, with no new attempts selected here:
  - `ROMDATA-04-PARAMETER-RECORDS27-ORDINARY`: source/model/runtime/CLI accepted; 59 records observed, full domain unproven; positive normal use and complete paired evidence missing. Eleven negative captures closed; archive legacy lines 17624..17672 and `docs/rom_data_gameplay.md`, ordinary sparse parameter record reachability section.
  - `ROMDATA-04-LOCALIZED-TABLE-GRAPHICS-ORDINARY`: 40 banks/8 tables source/model/runtime/CLI accepted; positive normal trigger and paired evidence missing; negative attempts remain closed. `docs/rom_data_ui_graphics.md`, ordinary table extension reachability section; archive 17844..17864.
  - `ROMDATA-05-DYNAMIC-RAW-ORDINARY-REACHABILITY` and `ROMDATA-01-LOCALIZED-RAW-DYNAMIC-DOMAIN`: conditional fragments/paths accepted; global domain and ordinary invocation unproven. Unit320 negative closed; later state2/state4/selector-witnesses diagnosis closed without a scene. Archive 59..75 and 789..866; no blind navigation.
  - `ROMDATA-01-OBJ-REMAINING-DRAW-ROOTS`, `ROMDATA-01-OBJ-REMAINING-CONSTRUCTOR-SOURCES`, `ROMDATA-01-OBJ-GRAPHICS-PAYLOADS`: authenticated increments accepted; global inventory/remaining consumers open. Do not repeat the broad experiment with 61 truncations. Archive 2853..2874, then only the selected family's record.
  - `ROMDATA-05-OBJ-VALIDATION`: local equivalence/costs, GUI, archive and USA initializer have later accepted outcomes; native Windows gates, further catalogs/scenes and normal enablement remain. Archive 3945..4087. Old artificial RSS/space constraints and `ROMDATA-05-BG-SCENES-COSTS` do not authorize further measurements: superseded by EUR/no-metrics steering.
  - Compact OBJ e101..e105: conditional prefixes/model/runtime/CLI accepted; ordinary use still open. Record222/candidate-table and selector33 have only static conditions and bounded negatives. Archive 11..65; do not infer a scene from its name.

## Incomplete tasks and open criteria

DBG2 audit of 2026-11-01: all eleven parents 13..23 remain unchecked;
8 have partial/integrated work, 3 lack the requested implementation.
The checks below predate this session and were inspected without rerunning.
Original criteria and audit notes: [DEBUG_TOOLS_V2_TODO](DEBUG_TOOLS_V2_TODO.md#p2--console-di-comandi-e-ricette-per-lassistente).
DBG2-01..12 remain complete within their scope. Read historical details only
for a specific question: archive legacy lines 179..744 or the named record below.

### DBG2-13

- State: suspended
- Outcome: partial
- Verified: typed parser, catalog/profile selection, cancellation, retry, discovery and shared replay; UI trace start, duplicate refusal and report have matching menu/CLI/console outcome evidence. `docs/changes/2026-11-03-verify-ui-trace-channel-parity.json`.
- Missing: full menu/CLI/console parity for validation, actions and results across other operations.
- Resume: select a missing group without repeating accepted profile/replay or UI-trace paths.

### DBG2-14

- State: suspended
- Outcome: integrated_unverified
- Verified: Unicode editor, history/selection, completion, simulated XInput, focus/hide and clipboard; October 31 records. New maintained-cheat hints/completions pass Wine tests and synthetic console GUI; `2026-11-03-complete-maintained-cheat-console-hints.json`.
- Missing: real IME composition/layout and GUI guards; native Windows/Linux/physical input unverified.
- Resume: select a missing input criterion; do not repeat accepted clipboard, editor or completion checks.

### DBG2-15

- State: suspended
- Outcome: partial
- Verified: structured context, retry, discovery and replay/panel; real EUR replay cancel/deadline and repeat procedure; 480×360 replay view; slow live refresh allows prior listing, rejects wrong ID, cancels exact ID without replacing index. `docs/changes/2026-11-03-cancel-slow-content-refresh.json`.
- Missing: full operation parity and comprehensive narrow-window presentation for long labels/parameters.
- Resume: select a missing criterion; do not repeat accepted replay, narrow replay view or live refresh.

### DBG2-16

- State: waiting
- Outcome: not_started
- Verified: DBG2-06 host runner/observational recipes and supervised replay exist; these do not constitute console batches with checkpoints.
- Missing: saved typed command sequence, pins, assertions/waits/caps, cancellation and resumption without repeating successful steps.
- Resume: depends on DBG2-15; no new development selected during the audit.

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

- State: suspended
- Outcome: partial
- Verified: catalog/source/preview, apply/arm/disarm, filters and named pinned groups save/preview/atomic arm via explicit commands. CH-0226..0241 have source placeholders referencing modifier tables, but their region and values are unvalidated; no substitutions inferred. `docs/changes/2026-11-03-save-and-review-cheat-presets.json`.
- Missing: validated values/ranges, page preset controls and complete game-effect evidence; device/ROM limits remain.
- Resume: select a safe missing criterion; inspect `cheats/tabelle-modificatori.md` only for a specific source and do not treat its declarations as verified.

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
- Outcome: not_started
- Verified: preceding foundations accepted; no recorded closure of parent 24, outside audit 13..23.
- Missing: named triggers; dependencies DBG2-05/15 and DBG-08/11.
- Resume: only criterion DBG2-24 in TODO section P4 upon a relevant request; no development selected now.

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

- A read-only coverage HTML character diff yielded with no retained handle on 2026-11-03; outcome/host status unknown. Do not restart or infer termination.

- Async work: the collaboration inventory observed on 2026-11-01 showed only `/root`; ps visibility is limited to the sandbox namespace. Recent isolated Linux test builds completed; no Wine/runtime job was started or signaled. Host/other-thread state is unobservable: do not assert global inactivity. Historical results prove their children's closure, not today's host state. Before resuming, check handle/process, output and owner; no response or a lost handle does not establish failure.
- Shared resources: checkout, handoff/archive/CHANGELOG, canonical targets, Wine prefixes, display, datasets and fixtures. Do not stop others' processes, restore source/binaries without ownership or clean live/failed tasks. Historical Wine retention audit: deleting a prefix requires coordination/host visibility, not sandbox ps. Disk nearly full in an old sweep; current availability not measured here.
- Memory writes: one coordinated writer; proposals with both preimage hashes and common lock through scripts/task_handoff.py update. If changed, reread and merge before publication. No background writer; no lock authorizes interference with other jobs. Details in MEMORY_WORKFLOW.
- Complete history: [TASK_COMPLETED_ARCHIVE](TASK_COMPLETED_ARCHIVE.md#legacy-handoff-snapshot), a provenance snapshot, not an operational queue. Old partial descriptions remain to prevent information loss: open tasks remain here, not promoted to closed. `TASK_HANDOFF_MIGRATION.json` records sections, hashes and terminal-outcome index. Search an ID and read only the necessary range.
- Godot: planned assessment complete, implementation deferred, not a current dependency. AndroidPort is separate; originals/ROM/BIOS/private catalogs/saves/recordings stay out of commits/packages.

## Final handover

- Task: debugtools v2 objective paused; DBG2 selections suspended. Other active tasks retain their sections and ownership.
- Last verified result: named source-bound cheat presets and typed filters passed Windows GNU/Wine checks; EUR cheat effects and live replay cancel/deadline/retry, slow refresh and UI trace channel parity recorded within bounded scopes.
- Remaining work: DBG2-13..20 and 22..28 retain open criteria; values, page preset controls, broader parity, provenance/recipes and device/platform verification remain. DBG2-21 closed within its EUR/Wine scope.
- Errors or blockers: cleanup helper refused non-ROM-data units without deletion. One old replay worker returned Access denied; fixed-worker comparison passed, precise OS cause unconfirmed. Initial narrow helper did not attest normal parent close; failed fixture retained.
- Next action: do not select work while the goal is paused; upon authorized resumption read this handoff and checkpoint a specific missing criterion.
- References: `docs/development/DEBUG_TOOLS_V2_TODO.md`, `docs/changes/2026-11-03-save-and-review-cheat-presets.json`, `docs/changes/2026-11-03-verify-ui-trace-channel-parity.json`.
