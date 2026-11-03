<a id="todo--debug-integrato-selettori-console-e-cheat"></a>

# TODO — Integrated debug: selectors, consoles and cheats

Created **29 October 2026**. **Implementation started** from the request
“esegui il todo debugtools v2”; the checkmarks below indicate completed units only.
The [first DBG-01…DBG-22 plan](DEBUG_TOOLS_TODO.md) remains complete within its
scope; this document defines extensions with new evidence without reopening it.

The goal is to use debug tools entirely from the integrated menu, selecting
content and parameters without typing paths or opening file pickers. Add a cheat
menu for preparing debug cases, an on-demand event console and a text command
console useful to the assistant. Windows remains the primary reference; Wine,
native Windows and Linux results are distinct. The current frontend is
sufficient: Godot and Android are not dependencies.

**DBG2 identifies a development task, CH a cheat-catalog entry, DS a scene.**
Evidence obtained with cheats remains a modified variant with provenance; it
does not automatically promote a DS scene or establish runtime parity.

<a id="basi-e-lacune-osservate"></a>

## Foundations and observed gaps

- The [22 existing tools](../debug_tools.md) provide isolated copies, a DS
  catalog, recordings, replay, audit, comparison, inspectors, reports and
  retention. The [menu](../../crates/menu/src/debug_catalog.rs) catalog is
  already navigable: reuse its selection, preview, filters and error handling.
- In [play.rs](../../crates/recomp/src/play.rs), start_replay opens a FileDialog
  when the profile is missing; open_debug_catalog does the same without a
  default catalog; CreateIssueReport may request a file automatically. Choose
  replay profile and Choose catalog still open the external picker.
- Several capture menu actions use Settings::default or fixed parameters. The
  CLI exposes filters and limits that need to become configurable in the menu.
- The [DBG-20 local channel](../debug_control.md) supplies session identity,
  queries, cancellation by ID and deadlines. The [UI trace](../debug_ui.md)
  observes input/focus/dispatch; the [recent log](../debug_recent.md) already
  has caps, drop counters and freeze on error. These are foundations for the
  consoles, not existing interactive consoles. Current UI hooks do not cover
  every raw input.
- The [local cheat catalog](../cheats.md), at cheats/catalogo.json schema 1,
  contains 341 blocks: 23 EUR Codebreaker, 112 Codebreaker with unknown region,
  20 GameShark USA, 111 GameShark JPN and 75 Action Replay with unknown region.
  All entries are not-runtime-tested. Listing an entry does not prove that its
  format, device version or addresses work in the runtime.

These references describe the starting point read on 29 October. Check the
code and selected criterion when starting development.

<a id="comportamento-richiesto"></a>

## Required behavior

- **Internal selection by default.** Available replay, scripts, catalogs,
  recordings, snapshots, datasets, runtimes, libraries, labels, known plans,
  campaigns and reports are selected in menu lists with filters and details.
  Opening a screen or encountering an empty list or missing file never opens a
  system dialog automatically or requires a typed path.
- **Explicit external access.** Only a clearly named “Browse external file…”,
  “Import…” or “Choose folder…” button may open a dialog. Cancel preserves the
  selection and state. Selection is not execution: use separate “Select”,
  “Start”, “Apply” and “Show report” actions.
- **Three meanings of script.** Distinguish debug-command recipes, host
  Python/PowerShell/Bash tools and game scripts derived from verified models.
  The first two require explicit launch and prerequisites; the third is selected
  for inspection/provenance, without treating ROM data as host programs.
- **Menu, console and assistant share operations.** Use the same validation,
  parameters, session control, outcomes and limits. Catalog commands actually
  available first; syntax examples below are proposals, not usable commands yet.
- **Two distinct, optional consoles.** The event console is read-only, with each
  event from enabled categories and every effective host selection change, its
  cause and outcome. The command console provides an editor and text/JSON
  responses. Explicitly show missing hooks, filters, aggregation and drops;
  “All” cannot claim to observe uninstrumented CPU/guest events.
- Observational operations preserve state/clock. Cheats and commands that change
  the game declare their changes and run in the debug copy. Identity, inputs,
  saves and logs follow existing portable and retention rules. Disabling a cheat
  stops future applications; it does not promise to undo effects already made.
  Restoration uses an explicit checkpoint.

<a id="p0--selezione-e-configurazione-dal-menu"></a>

## P0 — Selection and configuration from the menu

- [x] **DBG2-01 — Action inventory and shared contract.**
  Inventory every existing DBG action and CLI parameter, identifying required
  selection, prerequisites, effects, allowed state and result. Identify all
  paths opening dialogs and those with fixed settings. Define reusable menu/
  help/console descriptors over existing handlers without creating a second
  guest executor. **Completion:** every action has a destination in the new
  menu and a readable/structured outcome; parameters not yet exposed are listed
  as open work, not hidden. **Depends on:** DBG-01/20 foundations.
  Contract and usage: [debug operations](../debug_operations.md), including the
  three dialog sites and still fixed/unexposed parameters. Closed with a Windows
  build, synthetic Wine tests, positive/negative queries and public-payload
  checks; [record](../changes/2026-10-29-add-shared-debug-operation-contract.json).

- [x] **DBG2-02 — Shared selectable-content catalog.**
  Index declared portable roots and explicitly added folders: sessions,
  recordings, profiles, scripts/recipes, campaigns, snapshots, runtimes,
  datasets, native libraries, labels, known manifests and reports. Retain stable
  ID, type, name, description, ROM/region, hash, compatibility and availability.
  Bound scans and make refresh explicit; distinguish listing from complete
  verification and execution. Do not scan the disk indiscriminately.
  **Completion:** empty catalog, duplicates, non-ordinary links, changed/missing
  files and incompatible types have explicit outcomes; indexing executes no
  entry. **Depends on:** DBG2-01.
  Implemented: [content catalog](../debug_content.md), `debug-content scan/check`;
  synthetic Wine fixtures, refusal of real links, limits, duplicate names and a
  moved folder verified; [record](../changes/2026-10-29-add-bounded-debug-content-catalog.json).

- [x] **DBG2-03 — Reusable in-menu selector.**
  Provide a list with search, categories, filters, sorting, favorites/recents and
  a details pane; preview where already available. Support keyboard, mouse and
  controller navigation, scrolling, focus, long text and consistent return.
  Separate internal selection, refresh and external import buttons.
  **Completion:** choose two same-name items with different identities, change
  filters, cancel and handle an empty list without automatic OS dialogs or
  accidental actions. **Depends on:** DBG2-02.
  Implemented and verified: [internal selector](../debug_selector.md),
  keyboard/mouse/simulated XInput, same-name entries, search/filters, empty list,
  favorites/recents and preview. Escape propagation after dialog Cancel was
  corrected; a later test preserved complete state/frame0/clock0.
  [Record](../changes/2026-10-29-add-integrated-debug-content-selector.json).

- [x] **DBG2-04 — Migrate all existing tool selections.**
  Move Choose catalog/profile and Replay/Audit/Capture/Compare/Create issue
  fallbacks to the internal selector. Extend selection to sessions,
  snapshots/input, datasets, A/B runtimes, libraries, labels and known plans
  used by the respective flows. Keep the current selection visible and
  compatible. If input is missing, open an internal page explaining what to
  select. **Completion:** exercise each DBG2-01 entry with present, absent,
  changed and incompatible input; zero automatic dialogs. Only the external
  button opens a picker, once, and Cancel starts nothing. **Depends on:** DBG2-03.
  Implemented: [typed selections](../debug_selector.md), twelve roles, zero
  automatic dialogs and launch separate from confirmation. Missing/present
  inputs, changed/absent files, incompatible ROM and schema, single explicit
  dialogs and Cancel verified; [record](../changes/2026-10-29-migrate-debug-tool-input-selection.json).

- [x] **DBG2-05 — Parameters, presets and profile preparation in the menu.**
  Expose frame/timeout, input masks, ranges, filters, audio streams, triggers,
  previous comparison, issue symptom and trace options. Create and save
  profiles/presets with selected-input summaries; distinguish editing a draft
  from starting and do not overwrite an authenticated profile. Show missing
  features and unsupported combinations before starting.
  **Completion:** prepare recording → replay → audit/comparison/report without
  an external editor or typed path; parameters and outcomes match CLI.
  **Depends on:** DBG2-01/04.
  Implemented [parameter workbench](../debug_selector.md): drafts/presets,
  typed inputs, distinct check/prepare/start and new profiles. GUI chain and
  four CLI-equivalent profiles verified with equal outcomes/state/clock;
  [record](../changes/2026-10-29-add-debug-parameter-workbench.json).

- [x] **DBG2-06 — Runnable scripts and tools library.**
  Provide a “Scripts and tools” menu with debug recipes and declared adapters
  for useful host scripts, including homepage, campaigns and artifacts. Each
  entry has version/hash, typed arguments, Python/PowerShell/Bash/Docker
  prerequisites, expected output and Details/Start/Cancel buttons. Pass arguments
  as an array without building shell commands from names or paths. Add a
  separate known-guest-script view linked to the gameplay inspector; inspection
  does not execute those scripts. **Completion:** internal selection and setup,
  changed script or missing host runtime reported, explicitly started process
  cancellable and log traceable; no start on import/hover/selection alone.
  **Depends on:** DBG2-04/05.
  Implemented: five tabs, 17 typed roles, declared read-only adapters,
  status/inspect recipes and separate guest view. Wine GUI/frame0, XInput cancel
  with descendant stopped, timeout/refusals and UTF8 fix verified;
  [record](../changes/2026-10-30-add-declared-debug-script-library.json).

- [x] **DBG2-07 — Persistent selections and moved portable folder.**
  Save UI preferences and references relative to the correct root; distinguish
  internal entries and externally added references. After a ROM/session change,
  invalidate only incompatible selections. Recents and favorites never start
  replay, scripts or cheats automatically. **Completion:** reopen a moved copy
  with spaces/Unicode from another cwd, find valid selections and explain missing
  ones without automatic dialogs. **Depends on:** DBG2-04/06.
  Implemented: bounded state with relative roots, external provenance, pins and
  views/history; read-only restore without launch. Wine GUI reopens the same
  moved copy from `/tmp`, retains favorites and explains missing/changed entries
  at frame0; [record](../changes/2026-10-30-persist-debug-content-choices.json).

<a id="p1--console-degli-eventi-opzionale"></a>

## P1 — Optional event console

- [x] **DBG2-08 — Event schema and coverage map.**
  Unify already observed host events with sequence ID, host time, guest
  frame/clock, session, category, severity, origin and request/job ID. List
  supported and missing producers; connect UI/recent to the same events without
  double dispatch or recursive console logging. **Completion:** reconstruct
  request, selection changes, execution and outcome order from a correlated
  sequence; distinguish host order from aggregated guest events and unavailable
  hooks. **Depends on:** DBG2-01, DBG-13/19.

- [x] **DBG2-09 — Complete host selection and action trace.**
  Observe page, item ID and label, old/new selection, dropdown, changed value,
  opening/closing, confirm, cancel, captured binding and actually executed or
  refused command. Cover keyboard, mouse and available controller backends,
  distinguishing raw, remapped and consumed input. Log hover when it actually
  changes selection. **Completion:** each effective menu change has an event;
  held press, autorepeat, click and controller confirmation produce no duplicate
  commands. Backend gaps are declared. **Depends on:** DBG2-03/08.
  Implemented: [semantic changes and input](../debug_events.md), IDs/values/
  causes, editor and bindings; unchanged hover without duplicates. Six Wine
  captures with simulated XInput, single commands/correlated outcomes and
  unchanged state/frame0; [record](../changes/2026-10-30-observe-effective-host-menu-changes.json).

- [x] **DBG2-10 — Runtime and tool events.**
  Connect session/ROM, pause/step, Load/Undo/reset, recordings/replay,
  captures/reports, scripts, dataset/cache preparation, errors and job progress.
  Add recognized guest transitions and code/read/audio observations only within
  the actually instrumented scope. Future cheats publish activation, application,
  refusal and deactivation on the same channel. **Completion:** each family
  declares origin, frequency, prerequisites and coverage; errors and related
  requests are traceable, without new I/O in audio callbacks or continuous
  machine dumps. **Depends on:** DBG2-08.
  Implemented: [runtime events](../debug_events.md), explicit origin/frequency/
  requirements/scope and job/error correlation. Windows GNU build and Wine
  tests, GUI family tests and targeted verification with the current menu;
  unselected dataset/cache coverage and future cheat coverage declared.
  [Record](../changes/2026-10-30-connect-debug-runtime-lifecycles.json).

- [x] **DBG2-11 — Live event-console panel.**
  Toggle “Event console” from the menu or a configurable shortcut, initially
  off. Provide an overlay or adjacent view with scrolling, search, categories/
  levels, view-only pause, autoscroll, copy and explicit local save. “All
  available events” includes all enabled supported categories; sampling/
  aggregation is visible and can be disabled within caps. Show retained window,
  lost events and collector status even when the view is frozen. A filter does
  not change the original log. **Completion:** read a menu/command/error
  sequence while the game is open, hide/reopen the panel and export the window
  with consistent identities. **Depends on:** DBG2-09/10.
  Implemented: [event console](../debug_events.md#finestra-eventi-live--dbg2-11),
  separate bounded window, filters/view pause, copy/export and configurable
  polls. Windows/Wine and GUI tests with consistent identities and unchanged
  state; [record](../changes/2026-10-31-begin-live-debug-event-window.json).

- [x] **DBG2-12 — Limits, observable costs and absence of guest effects.**
  Set memory/row/byte caps, an update budget and overflow counters. Limit
  serialization/formatting on the game thread; no heavy collection with the
  console off and no accumulated audio with it open. Distinguish game pause,
  collector freeze and view pause. **Completion:** sustained synthetic load
  respects caps and counts losses; a new short selected sequence preserves
  state/clock with console off/on. Record observed cost without reopening
  historical benchmark campaigns. **Depends on:** DBG2-11.
  Implemented distinct budgets and counters, window bounds and shared snapshot;
  synthetic load, off/on comparison and brief Wine audio observation verified.
  The snapshot audio fallback without tap preserves state.
  [Details and limits](../debug_events.md#limiti-e-costo--dbg2-12),
  [record](../changes/2026-10-31-bound-event-console-update-work.json).

<a id="audit-documentale-del-1-novembre-2026"></a>

## Documentation audit of 1 November 2026

Criteria, current sources and existing records/evidence were compared without a
build, ROM extraction, gameplay or new runtime tests. None of the DBG2-13..23
parents has sufficient evidence for full closure: 7 partial (13/15/17/18/19/20/21),
14 integrated with incomplete input verification, 3 not started under the
requested contract (16/22/23), despite reusable foundations. No checkmark was
promoted. Notes below distinguish accepted substeps from missing criteria. Older
record limitations apply at their date and do not negate later results.

Resolved discrepancies: the checklist did not describe the console/cheat
increments; the old queue still requested profile selection, clipboard,
cancel/deadline and retry already verified after those checkpoints. Read-only
import of the 341 entries is integrated and tested, but does not complete the
runtime/verification index; decoder and one-shot exist but do not equal
maintenance/toggle/provenance. Catalog::summary/menu_entries also retain the
fixed reason “no decoder/handler implemented”, now inconsistent with
source-bound inspection and manual one-shot: per-CH capability integration
remains to be completed and was not fixed in this audit. The guard that refuses
recording/save after a one-shot alone does not implement DBG2-23.
  Subsequent work added the bounded source-capability index and maintained
  interpreter plans described under DBG2-17..19 below; this audit is historical.
No other development was selected. On resumption read only the individual ID in
[TASK_HANDOFF.md](TASK_HANDOFF.md); consult history only for a specific doubt.

<a id="p2--console-di-comandi-e-ricette-per-lassistente"></a>

## P2 — Command console and recipes for the assistant

- [ ] **DBG2-13 — Command language over shared handlers.**
  Define stable syntax, tokenization/quoting, numbers/addresses and argument
  schema. Connect help, status, pause/step, inspect, break/watch, captures/
  reports, profile/catalog selection and jobs to DBG2-01 contracts. Provide
  declared selection commands, for example profile select ID, without requiring
  paths. This is a debugger console, not an OS shell. **Completion:** a menu/
  CLI/console command with the same arguments yields identical validation,
  action and outcome; malformed text, ambiguous numbers and unknown commands
  do not mutate the session. **Depends on:** DBG2-01/05/08.

  **Audit 2026-11-01 — partial.** [Parser/dispatch](../../crates/recomp/src/debug_command.rs)
  reuses Action and the local channel; ID selection, targeted cancel/deadline,
  discovery, retry and shared replay are present and verified in substeps.
  [Profile selection](../changes/2026-10-31-prepare-debug-content-id-selection.json)
  includes the later positive CLI/panel test; [shared replay](../changes/2026-10-31-connect-shared-replay-commands.json)
  includes JSON copied from the console. UI trace start, duplicate-start refusal
  and report now have matching menu/CLI/console outcome evidence. Full parity
  across other operations remains missing; these paths do not need reimplementation.

- [ ] **DBG2-14 — Integrated editor, completion and contextual help.**
  Provide a “Command console” panel with explicit submission, history,
  cursor/selection, copy/paste, completion of names/arguments/IDs and contextual
  help. Support Unicode text, keyboard layouts and input composition; controller
  navigation to recent/preset commands, with an on-screen keyboard when needed.
  Multiline paste remains a draft to review, with no implicit submission.
  **Completion:** type, correct, recall and cancel a command; while the editor
  has focus, keys do not reach the guest or activate global shortcuts.
  **Depends on:** DBG2-13.

  **Audit 2026-11-01 — integrated, verification incomplete.** [Panel/editor](../../crates/recomp/src/debug_command_panel.rs),
  [completion](../../crates/recomp/src/debug_command/completion.rs) and the
  controller keyboard are integrated. [Editor/clipboard](../changes/2026-10-31-begin-debug-command-console.json),
  [controller](../changes/2026-10-31-add-debug-command-controller-keyboard.json) and
  [completion](../changes/2026-10-31-add-debug-command-context-completion.json)
  have Wine/Xvfb evidence; multiline input does not submit, and focus protects
  guest/menu. Real IME composition/layout and composition/capability guards in
  the GUI remain; physical hardware/native Windows/Linux are untested. Do not
  repeat the accepted clipboard check.

- [ ] **DBG2-15 — Structured responses, jobs and assistant access.**
  Show ROM/session target and resolved parameters, request ID, progress, short
  result and copyable JSON. Reuse DBG-20 local channel for command/capability
  listing, selection by ID, execution, status and cancellation of the same
  command, without automating coordinates or adding a network server. Separate
  parse error, incompatibility, refusal, unconfirmed timeout, cancellation and
  completion; status/cancel remain available during jobs. **Completion:** the
  assistant selects content and repeats a procedure through the console's API;
  wrong identity and cancellation of another ID are refused. Retrying a lost
  response does not duplicate execution. **Depends on:** DBG2-02/13/14, DBG-20.

  **Audit 2026-11-01 — partial.** [Local control](../../crates/recomp/src/debug_control.rs)
  and [panel observation](../../crates/recomp/src/debug_command/observation.rs)
  expose request/run/target, parameters/progress and outcomes. Records:
  [job context](../changes/2026-10-31-show-debug-command-job-context.json),
  [errors](../changes/2026-10-31-classify-debug-command-local-errors.json),
  [retry](../changes/2026-10-31-add-debug-command-retry.json),
  [discovery](../changes/2026-10-31-add-debug-content-discovery-commands.json) and
  [replay](../changes/2026-10-31-connect-shared-replay-commands.json).
  pending null after cancel is proven. A copied EUR replay later confirmed
  exact-ID cancellation after real worker progress (10/102 observed, stopped at
  21). A separate deadline run observed 9/102 and confirmed the server timeout
  with the worker interrupted at 13/102 after the client's unacknowledged
  timeout. An isolated EUR procedure also repeated discovery, ID selection and
  a complete 102-frame replay twice with equal endpoints; retry of the first
  identity returned the same JSON and worker. At 480×360, a corrected-worker
  replay showed its pending ID/parameters and successful result; long session
  labels remain abbreviated. A slow refresh over 2,048 inert files allowed
  listing the old index, refused wrong-ID cancellation and discarded the exact
  cancelled scan. Full operation parity and comprehensive narrow-window
  presentation remain.

- [ ] **DBG2-16 — Repeatable recipes and bounded batches.**
  Save a sequence of typed commands from the console, selectable in Scripts and
  tools: parameters, versions/pins, outcome waits, assertions and stop on first
  error. Set explicit step/frame/duration/output limits; preview before start
  and permit cancellation between commands. No unbounded loops. Recipes reusing
  concluded evidence require a reopening reason. **Completion:** one short and
  one interrupted recipe retain consistent order, identity, log and checkpoint;
  resumption does not automatically repeat successful operations.
  **Depends on:** DBG2-06/15, DBG-21.

  **Audit 2026-11-01 — not started under the console contract.** [Workbench](../../crates/recomp/src/debug_workbench.rs)
  and [declared library](../changes/2026-10-30-add-declared-debug-script-library.json)
  already have bounded read-only status/inspect recipes (DBG2-06 foundation).
  The [console parser](../../crates/recomp/src/debug_command.rs) does not save/
  execute typed batches with assertions, waits and checkpoints; the editor keeps
  multiline input as a draft. Console serialization, resume without reexecution
  and the requested short/interrupted tests are missing. Do not duplicate the
  existing host runner.

<a id="p3--menu-cheat-utile-alla-preparazione-del-debug"></a>

## P3 — Cheat menu for preparing debug cases

- [ ] **DBG2-17 — Local catalog import and compatibility states.**
  Read the existing catalog preserving CH, complete block order, declared
  format/region, aliases, notes, provenance and source master. Build a separate
  runtime index for exact ROM/device version, decoder and verification; the
  original remains browsable without rewriting. Everything is disabled by
  default: unknown, unsupported and unverified have distinct states with visible
  reasons. **Completion:** all 341 entries are browsable where the local catalog
  exists; a copy without cheats shows an empty list and explicit Import. No
  third-party payload enters public packages. **Depends on:** DBG2-02/03.

  **Audit 2026-11-01 — partial, import integrated and verified.** [Catalog](../../crates/recomp/src/debug_cheats.rs)
  preserves order/provenance/master; [CLI index](../changes/2026-10-31-index-local-cheat-catalog-readonly.json),
  [menu](../changes/2026-10-31-browse-local-cheat-catalog-in-menu.json),
  [target/guide/package exclusion](../changes/2026-10-31-describe-cheat-target-and-public-guide.json)
  and [local channel](../changes/2026-10-31-expose-cheat-snapshot-local-control.json)
  have 341-entry/empty/import/query/unchanged-state evidence. EUR/USA hashes
  mean comparison with the declared region, not compatibility. A bounded index
  now assesses each linked CodeBreaker source against its declared master and
  reports a one-shot plan, refusal or unevaluated state; the menu and local
  queries distinguish this from verification. Device version and actual ROM
  checksum still require separate verification. The seven static source/master
  checks do not verify all entries or behavior.

- [ ] **DBG2-18 — Format decoder and actual operations.**
  Start with the documentable EUR Codebreaker subset; define typed operations
  for supported writes, conditions, repetitions and continuations. Check widths,
  alignment, endianness, master/enable and ordering. Unknown GameShark/Action
  Replay versions/encryption remain non-executable until decoded and tested;
  do not treat strings as RAM writes. Placeholders require validated parameters,
  never guessed substitutions. **Completion:** public synthetic fixtures test
  every supported opcode and atomic refusal of incomplete/unknown blocks; each
  candidate CH reports exactly what it can execute. **Depends on:** DBG2-17.

  **Audit 2026-11-01 — subset verified, parent partial.** [Decoder](../../crates/recomp/src/debug_cheat_decoder.rs)
  implements bounded write/modify/condition/slide with atomic refusal;
  [plaintext](../changes/2026-10-31-inspect-assumed-plaintext-codebreaker.json),
  [explicit seed](../changes/2026-10-31-inspect-explicit-seeded-codebreaker.json) and
  [source/master](../changes/2026-10-31-inspect-source-bound-codebreaker-masters.json)
  record synthetic fixtures and mGBA/CRC comparison for seven EUR priorities.
  The bounded per-source index now exposes assumed decoded operations and the
  one-shot footprint, with explicit unevaluated and unresolved cases. Complete
  per-CH capability and ROM/device verification remain open. Unknown opcodes,
  formats and placeholders may remain refused as planned; the verified subset
  must not be expanded merely to claim closure. Manual one-shot is distinct
  from inspection. Do not implement a second decoder.

- [ ] **DBG2-19 — Controlled application in the debug copy.**
  Reuse the isolated session: arm, apply once, maintain and disarm are distinct
  operations with deterministic timing. Show affected operations/areas before
  activation and detect cheat overlaps. Writes follow the correct memory path,
  including any I/O effects; do not bypass it with raw access. Block unsupported
  native ROM/cache or explicitly handle invalidation and backend; no implicit
  compatibility claim. **Completion:** enabling/disabling and applying during
  pause/step have defined effects, errors leave no partially applied block, and
  the original game is not written. **Depends on:** DBG2-18, DBG-02/10.

  **Audit 2026-11-01 — partial.** [One-shot](../changes/2026-10-31-apply-guarded-cheat-once.json)
  and CheatPrepare/CheatApply handlers in [play.rs](../../crates/recomp/src/play.rs)
  verify source/state/ROM and preflight before bus writes; synthetic Wine tests
  and bounded CH-0334. A later bounded interpreter set supports explicit arm,
  application before each frame, overlap refusal and disarm, with synthetic
  Wine/Xvfb evidence. Native backend, I/O writes, ROM and mirrors are refused;
  device hook equivalence and gameplay remain unproven. A later isolated menu
  restart, save-state load and Undo each cleared the armed set in isolated
  Wine/Xvfb checks; load and Undo restored the expected machine states.
  Deactivation and checkpoint restoration remain distinct operations.

- [ ] **DBG2-20 — Cheats page with values, groups and profiles.**
  Search by CH/name/category, filter ROM/format/state, describe debug use, actual
  target and notes; select values from allowed list/range. Provide toggles for
  verified codes, Apply once, Disable all, named presets and a consistently
  visible active-cheat indicator. Masters are dependencies of the relevant
  decoder, not arbitrary codes selected automatically. Each non-executable
  entry explains what is missing. **Completion:** perform the full flow via
  menu/controller or an equivalent text command, without an implicit picker;
  conflicts and changed codes prevent silent activation. **Depends on:** DBG2-04/15/19.

  **Audit 2026-11-01 — partial.** The [catalog view](../changes/2026-10-31-browse-local-cheat-catalog-in-menu.json)
  provides search/details/filters and a later explicit source/preview/arm/disarm
  page flow. The cheat filters now use source-plan, declared-region and armed
  metadata rather than localized labels; synthetic Wine GUI checks include
  arm/disarm updates. Named source-bound groups can now be saved, reviewed and
  armed atomically through explicit text commands, with synthetic conflict and
  drift refusals. Page preset controls, values/ranges and complete game-effect
  evidence remain missing; retain the integrated view instead of rebuilding it.

- [x] **DBG2-21 — First EUR selection useful for debugging.**
  Priorities, subject to decoding/verification: access to Card Shop/Item Shop/
  Monster Bank (CH-0323/0324/0325) to reach UI and transitions; funds/resources
  (CH-0331/0334/0336) to repeat actions; HP/MP/stats (CH-0321) to keep the case
  alive. CH-0321 is a combined block: do not present it as simple HP without
  testing a distinct variant. Identify the observed target without equating
  “first character” with a global identity. **Completion:** at least one UI
  access and one resource modifier have isolated activation, effect,
  deactivation and restoration evidence; report each other CH individually,
  without marking a whole group verified. **Depends on:** DBG2-19/20.
  Completed within a bounded EUR/Wine interpreter scope: CH-0324 reached Item
  Shop after one Select+Right frame and disarm; the same-seed/input/frame
  control remained on the map, and seed restoration was exact. At the same shop
  purchase checkpoint, a 500-gil purchase produced 7,249 funds without a cheat
  and 99,999,499 with CH-0334 armed; disarm and exact checkpoint restoration
  were checked. [Details](../cheats.md#manual-one-shot-in-an-isolated-session--dbg2-19-in-progress).

  Individual status: CH-0324 has the bounded shop-access evidence above;
  CH-0334 has the bounded purchase evidence above, while a map redraw showed no
  visible change. CH-0336 reported 24 writes but Marsh remained at JP 4 after
  bounded next/previous inputs; the intermediate selection was not captured and
  the redraw is unconfirmed. Other units are untested. CH-0321 (combined block),
  CH-0323 (Card Shop), CH-0325 (Monster Bank) and CH-0331 (gil reward) retain
  static source/master decoding only; their gameplay effects are unverified.

  **Audit 2026-11-01 — partial.** [Static source/master](../changes/2026-10-31-inspect-source-bound-codebreaker-masters.json)
  covers seven priorities; [application](../changes/2026-10-31-apply-guarded-cheat-once.json)
  includes CH-0334: two RAM values and exact restore from seed. The local result
  target/debug-tools-v2-cheat-eur-resource/result.json was reread: no evidence
  of visible/gameplay effect, toggle/deactivation or UI access. All these criteria
  remain, dependent on 19/20; do not promote the group.

- [ ] **DBG2-22 — Advanced cheats for abilities, rules and transitions.**
  Plan small groups for abilities/AP/level (CH-0320/0330/0340), battle rules/
  cards (CH-0338/0339) and mission end/victory (CH-0329/0341). Only afterward
  evaluate global unlocks, completed missions, inventory/monsters, clan stats
  and speed in other EUR CH entries. The Clear Mission note is missing in the
  source: preserve uncertainty. Record persistent effects and shortcuts that
  skip the scene being diagnosed. **Completion:** check before/after state and
  side effects for each selected CH; others remain explicitly unverified. A
  forced victory is not evidence of the ordinary sequence. **Depends on:** DBG2-21.

  Progress: in one isolated EUR/Wine Party scenario, CH-0340 changed Marsh's
  displayed EXP 78→99 while level stayed 5; a same-seed/input/frame no-cheat
  control remained at 78. The plan recorded 24 writes; the observed effect
  persisted after disarm and a confirmed selection change, and both seeds were
  restored exactly. Montblanc showed 99 versus 92 in a control captured at a
  different frame; this is secondary evidence. In another EUR Party scenario,
  CH-0320 changed Marsh's Steal: Ability display from 180 to MASTER! after one
  armed frame and disarm; the same-seed/input/frame control remained at 180.
  Both seeds were restored exactly. Other abilities and side effects, actual
  level-up, CH-0330, rules/cards and transition entries remain unverified.

  **Audit 2026-11-01 — not started.** Catalog entries and the generic decoder
  are not evidence of abilities/AP/rules/transitions. No acceptance record for
  before/after/side effects of the requested groups; DBG2-21 remains open. The
  original uncertainty around the Clear Mission note is retained.

- [ ] **DBG2-23 — Cheats, checkpoints, replay and provenance.**
  Record cheat set/hash/parameters, decoder, order and frame of each application
  in compatible snapshots/profiles/recordings/reports. Define Load/Undo/reset/
  ROM change and replay with a different set: explicit restore or reasoned
  refusal, never a persistent toggle applied unexpectedly. “Restore checkpoint”
  remains distinct from “Disable all”. Handle state versioning while retaining
  expected compatibility; do not save host pointers in the guest.
  **Completion:** a new short cheat recording reproduces exact state/clock; an
  incompatible set is refused. The report identifies the variant, originals
  remain unchanged and no ordinary DS case is promoted. **Depends on:** DBG2-19/21, DBG-04/05/16.

  **Audit 2026-11-01 — not started under the provenance contract.** The
  [one-shot](../changes/2026-10-31-apply-guarded-cheat-once.json) refuses new
  recordings/saves after application (last_cheat_once guard in play.rs).
  [Recording](../../crates/recomp/src/debug_recording.rs) and [replay](../../crates/recomp/src/debug_replay.rs)
  do not record set/decoder/order/frame or prove state-clock cheat replay,
  Load/Undo/reset/ROM behavior and incompatible-set refusal. The preventive
  refusal does not equal the feature; no new recording was created in the audit.

<a id="p4--ulteriori-potenziamenti-e-chiusura"></a>

## P4 — Further improvements and completion

- [ ] **DBG2-24 — Named trigger and breakpoint editor.**
  Manage multiple saved break/watch/capture definitions from menu/console:
  name, enabled, PC/mode, range/value, hit count, frame window and supported
  action (pause, capture, event freeze). Allow declared simple combinations;
  show interpreter/native granularity and observer conflicts. **Completion:**
  one reached and one unreached trigger respect limits, identity and safe
  boundary, with an evaluated condition explainable in the report.
  **Depends on:** DBG2-05/15, DBG-08/11.

- [ ] **DBG2-25 — Pinned observations and differences.**
  Permit pins for registers, read-only memory windows and already decoded
  task/script/unit fields, with label/owner and previous value. Refresh at a
  bounded cadence or on step; invalidate addresses/context after Load/reset/
  ROM change. Colors and filters highlight changes without inventing persistent
  identities or reading registers with I/O effects. **Completion:** explain a
  known change between two points with a source for each field, complete state
  unchanged by inspection and recognizable stale context.
  **Depends on:** DBG2-03/15, DBG-11/17.

- [ ] **DBG2-26 — Issue timeline and exportable recipe.**
  Connect menu selection → command/script → observed guest event → capture/
  difference/error, with bookmarks and access to the corresponding report. Add
  recipe, selected IDs and any active cheats to the issue report; copy the
  repeatable command and distinguish available context from actually proven
  causation. **Completion:** user and assistant reconstruct the same case
  without manually searching for paths; gaps/drops and private attachments are
  explicit. **Depends on:** DBG2-11/16/23, DBG-09/16.

- [ ] **DBG2-27 — Targeted integration and regression checks.**
  Exercise new selection, console off/on, macros and cheats with public synthetic
  fixtures and a few declared real cases. Test keyboard/mouse/controller,
  button-only dialogs, caps, cancel, close, errors and moved folder; compare the
  same commands via menu/console/local channel. Reuse completed DBG evidence,
  selecting new checks only for changed behavior. Active cheats and read-only
  diagnostics have different equivalence expectations, declared before testing.
  **Completion:** Wine/native Windows/Linux and simulated/physical controller
  matrix with actual outcomes, protected originals, no error mistaken for
  success and no automatic repetition of all cases. **Depends on:** selected
  DBG2-04…26 increments; begin checks with each increment.

- [ ] **DBG2-28 — Guide, distribution and version compatibility.**
  Update help and guide with complete menu/console paths, actually supported
  commands, host-tool prerequisites and local index migration. Distribute only
  public adapters/schemas/fixtures; third-party cheat catalog, personal macros,
  history and game data remain excluded. Update the packager whitelist and
  Docker contexts only for necessary new files; declare build and local-channel
  capabilities. **Completion:** clean bundle, understandable empty states,
  commands from a moved folder and accessible logs; an old version receives a
  clear incompatibility. Checklist/record/state/handoff describe only what was
  proven. **Depends on:** DBG2-27 for distributed features.

<a id="riferimenti-e-anomalie-dei-cheat-da-conservare"></a>

## Cheat references and anomalies to preserve

For content use locally cheats/README.md, cheats/catalogo.json,
cheats/controllo-qualita.md and cheats/tabelle-modificatori.md. Versioned
documentation lists useful IDs and categories without copying code blocks.

Remaining issues for their respective entries: differing CB/XP source masters,
CH-0048/0049 P3/P4 labels, possible CH-0236 typo, placeholders and unknown
hardware versions. The 25 empty EUR sections on the saved page are missing
content, not imported cheats; do not invent or automatically download them.
Unlock instructions without a device code remain guides, not executable
switches. JPN and unknown regions may be browsed even when unsupported by the
selected ROM.

<a id="ordine-di-lavoro-e-regola-della-spunta"></a>

## Work order and checkmark rule

First group: **DBG2-01 → 02 → 03 → 04 → 05**, so existing tools become usable
without mandatory pickers. Then library/persistence **06/07**, event console
**08…12**, command console **13…16** and cheats **17…23**. Read-only import
**17** may start after the selector; activation waits for decoder and isolation.
Further improvements **24…26** reuse these contracts. **27/28** accompany each
distributable increment.

Each checkmark requires implementation, user flow, positive and negative
checks, declared limits/caps, input preservation and a memory record. If an
entry includes multiple CH or backends, record sub-outcomes: an incomplete parent
does not reopen completed children. Run necessary new checks, not old campaigns
merely to reconstruct context. Record the actual selection in
[TASK_HANDOFF.md](TASK_HANDOFF.md) according to the [workflow](MEMORY_WORKFLOW.md).
Creating this checklist does not automatically start any entry.
