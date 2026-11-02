<a id="todo--strumenti-di-debug-condivisi"></a>

# TODO — Shared debug tools

Updated **October 29, 2026**. This checklist tracks design and development:
checks identify completed tools. The tools must serve both the user, through
menus and short procedures, and the assistant, through reproducible commands
and structured results.

**DBG identifies a tool; DS identifies a test scene.** DS-01…DS-26 remain in
the [debug saves TODO](debug%20saves.md), with their respective states and
limits. Do not create a second list of validated scenes.

Current priority: help [ROM extraction](ROM_DATA_TODO.md), focusing on EUR.
Windows is the desktop reference; new functional checks follow the current Wine
path, separately identifying native checks still missing. This plan requires no
benchmark. The [Godot frontend](GODOT_FRONTEND_TODO.md) remains deferred and is
not a prerequisite.

The successor [Integrated debugging — DBG2](DEBUG_TOOLS_V2_TODO.md) plans
internal selectors, menu parameters/scripts, event/command consoles and debug
cheats. Its items are new and do not reopen DBG-01…DBG-22.

<a id="basi-già-disponibili-da-riutilizzare"></a>

## Existing foundations to reuse

Checks here identify existing capabilities within the scope documented in the
links; they do not mean these are already collected in one debugger.

- [x] **Debug slots:** up to 99 snapshots and 99 standard saves, copies,
  previews and Undo; [Extended saves](../extended_saves.md). The local catalog
  retains case provenance, region and status, but the menu uses generic names.
- [x] **Recording and replay:** `RECOMP_RECORD_INPUT`, `frames`/`runc`/`verify`
  with recorded input; `runc --state` resumes snapshots read-only. The
  [replay workflow](PERFORMANCE_WORKFLOW.md#record-a-battle-from-a-supplied-snapshot)
  documents the current limit on loads during recording.
- [x] **Comparisons and captures:** CLI frame-hash verification, targeted
  full-state/clock/audio tests and framebuffer/PPU/audio dumps. These have
  different scopes: `verify` alone is not equivalent to all targeted-test
  checks. References: [CLI](../../crates/recomp/src/main.rs),
  [comparisons](../../scripts/compare_native_backends.py),
  [log analysis](../../scripts/ffta_log_analysis.py).
- [x] **Code and fallback:** `dis` disassembler, interpreter trace, collection
  and import/export of [labels](../labels.md), native/fallback counters.
- [x] **Dataset and map:** extraction/verification/preparation and offline
  [ROM coverage](ROM_COVERAGE.md) through Docker, with identity, revision and
  generation date separate from ordinary-use evidence.
- [x] **Optional ROM audit:** bus/host collector and filtered captures with
  execution context, behind the `rom-data-read-audit` feature;
  [capabilities and limits](../rom_data_runtime.md#optional-bus-and-host-rom-read-audit).
  Now exposed through menu and CLI in DBG-06; Bank extensions retain separate
  bounded evidence described in the same document.
- [x] **Artifact management:** existing helpers and
  [local retention](LOCAL_ARTIFACTS.md) rules. Closing validation of the
  interrupted cleanup remains distinct from their existence; improvements are
  collected in DBG-18.

<a id="regole-comuni-per-i-nuovi-strumenti"></a>

## Common rules for new tools

- Same operation and meaning from the menu and equivalent command; the CLI must
  work without automating screen clicks and coordinates.
- Every run identifies the actual binary, ROM/BIOS, dataset, snapshot, input and
  configuration. Commit and modified working tree are insufficient to identify
  an old binary: build hash and provenance are required.
- Short readable output, versioned JSON for the assistant and distinct exit
  codes for success, divergence, timeout or incompatible input.
- Diagnostics disabled in the normal path; explicit activation, duration,
  filters and maximum output. Truncated logs and uncovered hooks must be
  recognizable, never interpreted as absence of problems.
- ROMs, snapshots, SRAM, recordings, dumps and assets stay local and ignored by
  Git. Sessions work on copies and do not save over the personal game.
- Observation must preserve guest state and clock; diagnostic reads must not
  change I/O registers, test counts or game flow.
- A check requires implementation, usage instructions, evidence of the expected
  result and error case, evidence and limits. A flag or correct image alone
  does not close validation.

<a id="p0--rendere-i-problemi-riproducibili-e-aiutare-lestrazione"></a>

## P0 — Make problems reproducible and help extraction

- [x] **DBG-01 — Session diagnosis and initial check.**
  Menu “Copy diagnosis” and CLI check without starting the scene: show profile,
  build, native/interpreter mode, active dataset, ROM/BIOS, slot, input, actual
  data path and report folder. Report missing files, incompatible generations
  and tools not compiled into the chosen build.
  **Closure:** the same report is obtained from menu/CLI; bad input is identified
  before execution and the cause indicates how to correct it.
  **Completed:** [session diagnosis](../debug_diagnostics.md), menu
  “Copy diagnosis” and CLI `diagnose`, shared JSON and a single atomic report.
  Verified 47 Windows tests under Wine, eight CLI cases on EUR data and menu/
  clipboard in an isolated Wine/Xvfb session; 139 protected files unchanged.
  Native Windows and Linux remain unverified. Native before startup is an explicit
  request, not a compilation already performed.
  [Record](../changes/2026-10-27-add-shared-session-diagnosis.json).

- [x] **DBG-02 — Isolated debug session.**
  Start a working copy of the chosen case, clearly identifying the active game.
  Snapshot load, Undo, autosave, backup and exit must remain in the session;
  return to the normal game must be explicit.
  **Closure:** original hashes unchanged after Load/Undo/restart/error and a
  transferable portable path. **Depends on:** DBG-01; reuses existing slots.
  **Completed:** [isolated sessions](../debug_sessions.md), Start/Close debug
  session menu and CLI `debug-session create/open/inspect`. Portable copies with
  authenticated inputs, local writes and external overrides excluded. Verified
  64 Wine tests, 41 final menu tests, seven CLI cases and GUI checks of
  operations, relocation and explicit return to the normal game; 73 protected
  inputs unchanged. Native Windows/Linux and native gameplay in the copy remain
  unverified. [Record](../changes/2026-10-27-add-isolated-debug-sessions.json).

- [x] **DBG-03 — Browsable and queryable DS catalog.**
  Show scene name, region, slot, variant, source, status and available evidence;
  filter from the menu and obtain the same data as JSON. Import into a free slot
  with preview, without confusing DS, IN, GEN and variant numbers.
  **Closure:** selecting DS-20 opens the expected point in the EUR copy;
  DS-03/DS-15 remain uncovered until new evidence exists. Detect catalogs with
  missing files or different checksums. **Depends on:** DBG-01/02.
  **Completed:** [shared catalog](../debug_catalog.md), Debug catalog menu and
  CLI `debug-catalog list/open/import`, filters, provenance, checksums and
  previews. Verified 61 Wine tests, 15 CLI cases and DS-20 opening/import into
  slot 2/loading/return in Wine/Xvfb GUI; 140 protected inputs unchanged. The
  92 entries keep DS, variants and sources distinct; DS-03/15 remain uncovered.
  Native Windows, Linux and physical controllers remain unverified.
  [Record](../changes/2026-10-28-add-shared-debug-save-catalog.json).

- [x] **DBG-04 — Simple recording from a snapshot.**
  “Record/Stop” commands and corresponding CLI start: associate the initial
  snapshot, frame origin, already-held keys and ordered input sequence. On
  Load/Undo during recording, stop with a message or start an explicit segment;
  do not weaken the current guard by producing an apparently valid recording.
  **Closure:** replay reaches the expected endpoint with consistent state/clock,
  including when starting with a held key. **Depends on:** DBG-02/03.
  **Completed:** [recording from a snapshot](../debug_recordings.md), Start/Stop
  input recording menu and CLI `debug-record start/verify`. Initial snapshot,
  relative frames, explicit initial keys, ordered events and authenticated
  endpoint; limits without automatic deletion. Load/Undo load save the capture
  and refuse the first rewind. Verified 57 Wine tests and five real DS-20
  captures with identical full-state/clock replay, plus relocation, altered
  files, active session, closure and write failure. 140 original inputs
  unchanged. Native validation, native Windows and physical controllers remain
  open. [Record](../changes/2026-10-28-add-snapshot-input-recording.json).

- [x] **DBG-05 — Replay with one command or button.**
  A local profile contains the selected case, binary and dataset, input, frame
  count, stop condition and timeout. The user chooses “Replay case”; the
  assistant runs the same profile and receives progress and JSON outcome.
  Do not enable performance measurements for a functional check.
  **Closure:** repeatable endpoint; clear errors on incompatible input, timeout
  and interruption, without personal writes. **Depends on:** DBG-01…04.
  **Completed:** [replay profiles](../debug_replays.md), Replay case/Choose
  replay profile/Cancel replay menu and CLI `debug-replay create/run/cancel`.
  Isolated worker without game writer, exact endpoint, JSON progress/result and
  distinct codes. Verified 60 Wine tests, repeatability/relocation, errors/
  timeout/cancellation/double start and menu with unchanged parent; 171 selected
  inputs unchanged. The old local native library completes the case with
  identical clock and different state: divergence reported, parity not accepted.
  Native Windows/Linux/controllers remain unverified.
  [Record](../changes/2026-10-28-add-supervised-debug-replay-profiles.json).

- [x] **DBG-06 — ROM read audit usable without ad hoc tests.**
  Expose existing collectors with presets, frame interval, address filter and
  event limit. Distinguish bus/host, typed/raw reads, excluded fetches and
  unobserved accesses. Report PC/mode when available, keeping context inherited
  from DMA/HLE separate from the instruction that reads directly.
  **Closure:** report physical intervals and distinct bytes, compare known data
  and discard duplicate mirrors; check state/clock with audit off and on. A read
  outside an owner is a lead, not a newly proven family.
  **Depends on:** DBG-01/05 and completion of audit extensions already underway.
  **Completed:** [ROM audit](../debug_rom_audit.md), CLI
  `debug-rom-audit create/run/cancel` and Audit replay menu on isolated profiles.
  Filters, window, bounded events, actual typed/raw service, PC/mode scope,
  physical unions and comparison with an identified plan. Truncation and partial
  windows explicit; missing feature rejected. Verified 61 Wine tests, raw/typed
  CLI cases with identical state/clock, independent counts, menu/cancel and
  unchanged parent; 238 protected inputs unchanged. Two DS20 frames: 12,416
  distinct B, 11,495 B served typed with dataset e76. No new ROM/DS acceptance.
  [Record](../changes/2026-10-28-add-bounded-rom-read-audit-tools.json).

- [x] **DBG-07 — Connect scenes, reads and ROM coverage.**
  Import audits by DS case and expose “identified”, “served by runtime”, “read in
  selected scenes”, “shared” and “unknown” levels. Click a region → resource,
  scenes and evidence; compare two generations with visible date/identity.
  Keep code and data candidates separate.
  **Closure:** count unions without adding rereads, distinguish positive-family
  intervals from observed bytes, reject reports with a different ROM and declare
  collector coverage. **Depends on:** DBG-06.
  **Completed:** [map and instructions](ROM_COVERAGE.md#scene-letture-e-confronto-dbg-07),
  explicit import via Docker, unions reconstructed from events, report selection
  and resource/case/evidence detail. Schema 3 with dated comparison, identity
  and added/removed intervals; filters, gaps, TRONCATO/PARZIALE (truncated/partial) visible.
  26 generator tests, eight Bash and 12 PowerShell checks pass; real Docker and
  browser at 1400/620 px verified. Three existing DS20 audits: 31,061 B as a
  truncated minimum; short complete captures 12,416 B, typed 11,495 B. No new
  replay, extracted data or promoted DS case; snapshot e76 explicitly historical.
  [Record](../changes/2026-10-28-link-rom-coverage-to-scene-audits.json).

- [x] **DBG-08 — Stop and capture when a case is reached.**
  Allow conditions such as first read of a resource, entry into a verified PC
  or known event. Stop at a safe boundary and create a derived snapshot with
  origin, input and capture reason, ready for the catalog.
  **Closure:** a known trigger produces a reusable point; a never-reached trigger
  ends at the declared limit without promoting the case. Ordinary scenes are
  reached through real input. **Depends on:** DBG-03/05/06.
  **Completed:** [trigger captures](../debug_captures.md), CLI
  `debug-capture create/run/cancel` and “Capture on trigger” menu. First read
  of authenticated resource, interpreter entry into a verified PC/opcode or frame
  event; snapshot at safe boundary with origin/input/reason and catalog variant.
  Limit without match → `not-reached` (7), no snapshot; eight bundles per
  profile, without overwrite. 66 Wine tests, 25 CLI cases plus cancellation and
  four feature checks, GUI reached/not-reached/cancel/owner-close pass. DS20:
  three triggers agree on full state/clock after one frame; reusable catalog.
  203 originals unchanged, no new DS acceptance.
  [Record](../changes/2026-10-28-add-bounded-debug-trigger-captures.json).


<a id="p1--capire-dove-e-perché-compare-una-differenza"></a>

## P1 — Understand where and why a difference appears

- [x] **DBG-09 — Compare at the first divergence.**
  Compare interpreter/native or raw/typed data using existing checks: full
  state, clock, central/wide framebuffer, OAM and audio stream. Highlight the
  first frame and differing fields, with side-by-side images and JSON.
  **Closure:** locate a known difference beyond the final hash; indicate exactly
  which components were compared. **Depends on:** DBG-05; heavy output only
  around the divergence.
  **Implemented (2026-10-28):** [frame comparison](../debug_comparisons.md),
  `debug-compare create/run/cancel` and “Compare first difference” menu.
  Interpreter/raw reference and interpreter/native candidate with optional
  dataset; exact serialized CPU/Bus fields, registers, side-by-side images and
  audio stream. Maximum window 3,600 frames, eight reports per profile;
  snapshot/PNG/audio only at the first differing boundary, metadata for equal or
  interrupted prefix. 67 Wine tests, 20 CLI cases plus cancellation, three
  default checks, GUI and desktop/mobile browser pass. DS20: first differing
  frame, clock +4 cycles and only r15; images/OAM/audio agree. This locates the
  boundary without a verdict on cause or acceptance of native parity. All 276
  checked originals and normal executables remain unchanged.
  [Record](../changes/2026-10-28-add-first-frame-debug-comparisons.json).

- [x] **DBG-10 — Pause, frame stepping and controlled resume.**
  Provide shortcuts and equivalent commands to stop and advance N frames. Show
  guest frame, clock and stop reason; prevent the debugger key from also becoming
  game input. **Closure:** exact stepping, resumption without state skips or audio
  buildup caused by host pause. **Depends on:** DBG-02/05.
  **Implemented (2026-10-28):** [isolated copy control](../debug_control.md),
  Step 1/10 and Resume menu, F6/F7/F8 consumed before guest input and CLI
  `debug-control status/pause/step/resume`; start with `debug-session open DIR --paused`.
  Budget of 1–600 frames, neutral input or CLI mask, visible frame/clock/reason,
  audio queues flushed and pacing restored on resume. 69 Wine tests, 24 CLI/GUI
  cases and nine defaults pass; 22 menu/keyboard frames match the CLI in full
  state and 22/80-frame recordings match the continuous interpreter.
  Interruptions, concurrent client and replay during a budget verified. 202
  originals unchanged; native platforms/controllers/native steps unverified.
  [Record](../changes/2026-10-28-add-controlled-debug-frame-stepping.json).

- [x] **DBG-11 — Breakpoints, watchpoints and read-only memory.**
  Select PC/ARM/Thumb, read/written interval or observed value; show registers
  and memory with labels, ROM offset and owner. Distinguish instruction stop
  from native-block stop: do not claim unsupported granularity.
  **Closure:** two targeted triggers verified and diagnostic reads without I/O
  effects; no arbitrary RAM modification required. **Depends on:** DBG-10/06.
  **Implemented (2026-10-28):** [read-only search and inspection](../debug_probes.md),
  `debug-control break/watch/inspect` and Inspect PC / registers menu in the copy.
  PC/mode before the step, bus/value accesses after the step with HLE/DMA explicit;
  bounded interpreter search, native refused, watch requires audit feature.
  Registers, 1–256-byte windows, mirror/offset/label and owner from authenticated
  source-plan. 88 distinct Wine tests, 21 CLI/GUI cases and nine defaults plus
  interruptions pass: Thumb PC and two real IRQ accesses, reads without mutation,
  guards and closure. Also fixed lost confirmation on a replacement temporarily
  denied by Windows: republish the same response without re-executing the command.
  202 originals unchanged; no new ROM/DS/native acceptance.
  [Record](../changes/2026-10-28-add-bounded-debug-breakpoints-and-memory-inspection.json).

- [x] **DBG-12 — Graphics inspector and visual comparison.**
  Combine existing dumps: BG/OBJ, OAM, palettes, tiles, windows, original
  framebuffer, widescreen extension and final composition. Link a selected pixel
  to its source when provenance is demonstrated.
  **Closure:** explain a known palette/clipping/layout case with reproducible
  view and report, without altering guest registers or rendering.
  **Depends on:** DBG-05/09; reuses extracted data and current graphics comparisons.
  **Outcome:** [offline inspector](../debug_graphics.md) from a paused copy,
  menu/CLI, up to eight private reports with buffer/PNG/JSON and prior comparison.
  BG/OBJ/OAM/palette/tile/windows, central/wide and CPU composition with/without
  UI; explicit pixel backing, ROM provenance and final winner unknown. DS-20
  frame 22220 explains UI padding/clipping: 852×480 window, canvas at x=-294,
  actual content 89,856 pixels in [114,48,738,192), inside clip. 58 Wine tests,
  21 CLI/GUI cases, desktop/mobile browser and five reports verified; 202
  originals and normal runtimes unchanged. GPU effects, native platforms and
  other scenes not certified.
  [Record](../changes/2026-10-28-add-readonly-graphics-inspector.json).

- [x] **DBG-13 — Menu, text and input diagnostics.**
  Show recognized menu, focus, host/guest action, bbox and text metrics; trace
  press/release, remapping and menu-consumed confirmation. Retain a keyboard/
  gamepad check and a CLI check with recorded input.
  **Closure:** attribute a double confirmation or clipped text to a precise step
  without relying on a screenshot alone. **Depends on:** DBG-04/12.
  **Outcome:** [UI/input trace](../debug_ui.md), Start/Save from menu and CLI,
  focus in `status` including dropdown options/geometry, same-font metrics, bbox
  and diagnostic PNG without modifying rendering or input. Limits: 30 s/1,024
  events/600 frames, eight private reports. Keyboard autorepeat: 14 repeats,
  one dispatch; simulated XInput: one A remapping, no double confirmation or
  guest frame. CLI A/release recording verified over two frames after closure.
  Clipped text attributed to 545.89 px required/472 available, 64/75 characters,
  false canvas clip. 69 distinct Wine tests, desktop/mobile browser and six
  reports; 202 originals/normal runtimes unchanged. Test navigation errors kept
  separate from accepted prefixes; physical controllers/guest text not certified.
  [Record](../changes/2026-10-28-add-bounded-menu-text-input-diagnostics.json).

- [x] **DBG-14 — Bounded audio diagnostics.**
  Make mixed/PSG/FIFO/HLE captures, their events and a short interval around the
  defect selectable; compare and play dumps locally. Expose relevant mute, Fast
  and resets without starting a benchmark.
  **Closure:** locate a known divergence and distinguish generated audio from
  audio actually delivered to the device; I/O outside the audio callback.
  **Depends on:** DBG-05/09; declare byte/duration limits and hardware listener.
  **Outcome:** [audio captures and comparison](../debug_audio.md), Start/Save
  menu and CLI with stream/window selection, private WAV/raw/HTML/JSON. Guest
  buffer, producer decision and actual callback buffer remain distinct; mute/
  Fast/gain/queues/resets explicit. No new callback I/O/clock/allocation/lock.
  Divergence preserved at frame 8514: mixed 81918/81919 −704 versus −576, without
  rerunning. 66 distinct Wine tests and two targeted reruns, four-frame recording
  with exact endpoint, menu/default and CLI/feature, desktop/mobile browser with
  WAV/seek and homepage 37 indexes/8 audio pass; 197 originals and normal
  runtimes unchanged. Final report also identifies seed/BIOS/dataset/build/input.
  Native Windows/Linux, hardware/listener and other scenes remain unverified.
  [Record](../changes/2026-10-29-add-bounded-audio-diagnostics.json).

- [x] **DBG-15 — Code, labels and fallback in one report.**
  Connect observed PC, disassembly, label, compiled block and fallback reason,
  with filters and bounded captures. Distinguish ROM code, RAM code, static
  candidates and observed execution.
  **Closure:** reconstruct an interpreter/native transition and flag label import
  for a different ROM; do not infer code-extraction percentage from entry count
  alone. **Depends on:** DBG-01/06/11.
  **Outcome:** [code/label/fallback trace](../debug_code.md), Start/Save menu/CLI,
  PC/mode/backing/disassembly and actual choice, filters and bounded prefix.
  Different-ROM label rejected; ROM/RAM and static candidates remain distinct.
  In the real native prefix events 17/18 transition from compiled 08145c10 Thumb
  to missing 0801bde8 Thumb: 41 native/470 missing/1 IRQ. The same library with
  observer off/on preserves full state and 6547 blocks/14991 steps in the entire
  frame. 74 Wine tests, menu/CLI with exact two-frame recorded endpoint, errors/
  capacities, desktop/mobile browser and four reports pass; six original inputs/
  runtimes unchanged. No new interpreter/native parity or code-extraction percentage.
  [Record](../changes/2026-10-29-add-bounded-code-dispatch-diagnostics.json).

- [x] **DBG-16 — Issue report ready to read together.**
  Generate a local HTML index, summary and JSON with symptom, case, frame, input,
  identities, comparison and suggested next check. Buttons to open the folder
  and copy the exact command; distinguish private attachments from shareable metadata.
  **Closure:** the user can point to a report and the assistant reproduce it
  without manually reconstructing environment variables; no automatic file upload.
  **Depends on:** DBG-01/05/09; report also present on error.
  **Outcome:** [local issue report](../debug_issues.md), menu/CLI and HTML/JSON
  index with symptom, identity, recorded/window/completed frames, first difference,
  private attachments and next check. Incompatible preflight still saves; eight
  reports/2 MiB with lock, no automatic replay/upload. Relative launcher after
  relocation with spaces/apostrophe/Unicode reaches exact two-frame endpoint;
  real clipboard/folder and desktop/mobile browser pass. 72 distinct Wine tests
  plus one targeted rerun, menu/CLI with guest state unchanged, 28 original inputs
  intact; homepage 49 indexes/8 issues. Native PowerShell/Windows and Linux
  remain unverified.
  [Record](../changes/2026-10-29-add-reproducible-local-issue-reports.json).

<a id="p2--estendere-la-diagnosi-senza-accumulare-lavoro-e-dati"></a>

## P2 — Extend diagnosis without accumulating work and data

- [x] **DBG-17 — Task, script and unit inspector.**
  Expose already verified structures, cursors, actors and resource links, with
  per-field provenance. Leave undecoded fields unknown and distinguish ROM data
  from RAM state and sentinel values.
  **Closure:** reconstruct a known sequence from observed events without
  inventing global bounds or fixed unit identities. **Depends on:** DBG-06/11
  and already completed ROM-specific contracts.
  **Outcome:** shared menu/CLI, family/context filters and offline report with
  per-field provenance, authenticated EUR model resources and unknown fields.
  Limits 30 s/60 frames/512 events/1 MiB, eight folders/2 MiB JSON, no new
  snapshot. 79 Wine tests, GUI with simultaneous code trace and identical full
  state: 70 events reconstruct countdown 10→9→8 of the same task address in two
  frames, without persistent identity. Menu/empty-filter/selected-context report
  and browser pass; no script observed in this window, without inferring global
  non-use. Native Windows/Linux and native gameplay capture remain open.
  [Usage](../debug_gameplay.md), [record](../changes/2026-10-29-add-bounded-gameplay-inspection.json).

- [x] **DBG-18 — Debug session and artifact manager.**
  Reuse existing cleanup and complete its verification/distribution: Docker
  contexts with all required modules, removal receipt, selection of sessions
  to retain, estimated space and trace/capture limits. Protect active sessions
  and in-use caches through explicit coordination; preserve errors, originals,
  inputs and evidence while deleting completed redundant copies.
  **Closure:** consistent preview/apply, new session and concurrent cleanup do
  not damage each other, error/interruption and paths-with-spaces tests pass.
  **Depends on:** DBG-01; applicable from the first P0 tools.
  **Outcome:** preview/apply/--keep with estimated space, limits and receipts;
  common leases for creation/runtime/cleanup/cache and historical read-only
  reads. Legacy, failed, active/interrupted contexts and units with nested markers
  protected; no automatic removal of leases or originals. Complete Docker modules,
  Git-derived image and launcher with log. 33 Python and 34 Wine tests pass,
  real Windows/Python contention while paused, separate concurrent creation,
  error/interruption, Docker keep/apply/stale plan/read-only reader and browser
  at 1400/620. All new removals are synthetic fixtures; no old sweep or ROM
  replay rerun. Native PowerShell/Windows remain unverified.
  [Usage](../debug_artifacts.md), [record](../changes/2026-10-29-coordinate-debug-artifact-retention.json).

- [x] **DBG-19 — Recent events ring buffer.**
  Keep a bounded amount of input, scene changes, selected reads, fallback and
  errors; freeze on request or trigger without continuous whole-machine dumps.
  Indicate discarded events and actual window.
  **Closure:** respect limits even in a long run and save useful context on a
  controlled error. **Depends on:** DBG-08/16/18.
  **Verified (October 29):** rolling window 1,024 events/256 KiB, 8 KiB/event;
  actual input, partial UI signature, 16 ROM keys/frame and dispatch/errors.
  Freeze/Save menu/CLI and error trigger save HTML/JSON without snapshot. One
  million synthetic events respect caps; GUI remains active after 31 s, evicts
  60 events and saves useful context on controlled error without changing guest
  state. 81 Windows/Wine and 11 Python tests pass, two-frame interpreter observer
  identical and desktop/narrow browser verified. Default unavailable reads
  distinguished from zero; native platforms unverified.
  [Usage](../debug_recent.md), [record](../changes/2026-10-29-add-recent-event-ring.json).

- [x] **DBG-20 — Local control of an open session.**
  Assess a structured local channel to query state, pause, advance and capture,
  shared with menu commands. Prefer the smallest solution allowing the assistant
  to operate without fragile clicks.
  **Closure:** versioned commands, timeout/cancellation, session identification
  and structured response; explicit local access and no service exposed by default.
  **Depends on:** DBG-01/02/10/16.
  **Verified (October 29):** extended the file-based channel already shared by
  the menu, without networking. Status/cancel queries separate from the waiting
  operational client; cancellation by exact ID, deadline and configurable
  20–120000 ms timeout, manifest/run/request identity and JSON responses even
  for local errors. 30 distinct Wine tests and one targeted rerun for closed
  classification; real GUI with default 19 CLI checks exercises status while
  waiting, wrong ID, partial cancellation, 250 ms deadline with server stop,
  read-only capture and wrong/closed session. DBG10/11/12/16 checks reused; no
  old replay reopened.
  [Usage](../debug_control.md), [record](../changes/2026-10-29-complete-local-session-control.json).

- [x] **DBG-21 — Targeted case and regression campaigns.**
  Select DS, families or transitions affected by a change; show positive/
  negative/incompatible/not-run outcomes and reused earlier evidence. Game data
  remain local: public runner tests use synthetic fixtures.
  **Closure:** the report distinguishes “zero reads in this scene” from “feature
  unused in the game”; no automatic replay of all completed checks to reconstruct
  context. **Depends on:** DBG-03/05/07/09/18.
  **Verified (October 29):** manifest/pins and explicit DS/family/transition
  selection, four outcomes and evidence reuse. Report without execution;
  run-case delegates one profile to the existing supervisor with reason/receipt
  and explicit reopening for previous evidence. 32 cases/8 outputs/2 MiB, zero
  in the window distinct from unknown global use. 23 Python tests, synthetic
  public dispatch, two DS20 checks reused read-only, Bash/Docker and browser
  pass; zero new guest frames, no build/runtime/old acceptance repeated.
  [Usage](../debug_campaigns.md), [record](../changes/2026-10-29-add-targeted-debug-campaigns.json).

- [x] **DBG-22 — Distribution and shared tool guide.**
  Document a few complete workflows: report a bug, record a scene, compare two
  runtimes, find uncovered reads and free temporary files. Align help/menu/CLI;
  provide PowerShell and Docker launch for offline tools where useful, with
  error logs always traceable.
  **Closure:** repeatable procedures from relocated portable path, explicit
  prerequisites, packages without personal data and declared Wine/native
  Windows/Linux matrix. **Depends on:** tools actually selected; does not require
  the entire plan or Godot frontend to be complete.
  **Verified (October 29):** [shared guide](../debug_tools.md) with five complete
  workflows, prerequisites/matrix and persistent logs; aligned help, PS/Bash/
  Python/Docker launchers. Ten public packager tests, 13 simulated PowerShell
  7.4.6 Linux tests, two synthetic TinyCC tests and offline read-only ZIP pass.
  Bundle relocated with spaces/Unicode: recording/menu, two runtimes, bug/audit/
  cleanup, browser 1400/620; five new frames total, full endpoint/clock identical
  even with final remapped binary. 76 public files/75 authenticated payloads,
  empty Data, no ROM/BIOS/progress/log/fixture/translation, personal prefixes
  refused. Native Windows/PS5.1 and recent Linux runtime not certified; normal
  runtimes and original inputs unchanged.
  [Record](../changes/2026-10-29-distribute-shared-debug-tools.json).

<a id="mantenere-gli-strumenti-completati"></a>

## Maintain completed tools

DBG-01…DBG-22 are complete within the scopes and platforms declared in their
records. Start from the [shared guide](../debug_tools.md) to choose a workflow;
numbering remains stable. Native/hardware limitations and reported divergences
are not passed checks and do not automatically reopen completed units.

For new work select a missing criterion or concrete regression, recording the
reason, changed input and next check in [TASK_HANDOFF.md](TASK_HANDOFF.md).
Reuse existing evidence; do not repeat every case to reconstruct context.
Update the document and records consistently with the actual result. This
checklist does not replace ongoing ROM priorities.
