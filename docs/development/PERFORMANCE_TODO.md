<a id="todo-ottimizzazioni-desktop-fftarecomp"></a>

# FFTARecomp desktop optimization TODO

Plan from October 20, 2026, based on code at revision `e39cdb6`.
Priorities: stability, game correctness, Linux/Windows compatibility and code
simplicity. The reported problem concerns Ubuntu 26 and possible slowdowns during
enemy turns. Checkboxes indicate completion of every criterion for each task;
partial progress is recorded below.

The recommended sequence is to **measure the actual slowdown, eliminate
redundant work, address the dominant subsystem, then assess further GPU work**.
Increased GPU utilization alone is not a useful result.

<a id="stato-dellesecuzione"></a>

## Execution status

Status reconciled with current criteria and evidence on October 21. Read
[TASK_HANDOFF.md](TASK_HANDOFF.md) before selecting work: an open parent does
not reopen completed substeps or rejected experiments. The early test sections
below are historical; their counts/exports apply to that phase.

- **PERF-01 — partial:** enemy replay, worldmap and zoom6 already available and
  measured; the rare original slowdown is not reproduced/isolated. Do not
  recreate fixtures and baselines to reconstruct context.
- **PERF-02 — partial:** CPU/PPU/GPU profiles, audio callback, frame gate, raw
  enhanced requests, PipeWire observation and presentation cadence are
  implemented. Attribution of the actually slow scene, controlled realtime
  overhead and hardware/native Windows coverage remain. Empty requests do not
  equal audible defects; ERR PipeWire does not cover every hardware XRUN.
- **PERF-03 — completed:** A/B procedure with hash/configuration, verification
  separate from timing, state/frame/audio comparison and rollback; incomplete
  guard stopped before temporal history and frontend counters. Reuse
  [dispatch checks](../changes/2026-10-20-preserve-native-dispatch-in-cost-profiling.json)
  and complete candidate checks without reimplementing them.
- **PERF-04 — partial:** recovery, skipped uploads, controlled device
  invalidation and X11/Wine lifecycle verified; Wayland/native Windows and the
  physical display/DPI/GPU matrix specified in the entry remain.
- **PERF-05/06 — completed:** LUT/FPS revisions, UI input-packing cache,
  call-local shared world upload and FIFO reuse. Rejected prototypes stay rejected.
- **PERF-07 — partial:** CPU inventory and sampling completed; no new
  translation accepted. Guest counts/cycles do not isolate host cost.
- **PERF-08 — completed:** scanline/UI reductions retained with CPU reference,
  full state and headless/GUI benefits; worldmap/zoom6/lifecycle verified.
  [Latest retained reduction](../changes/2026-10-21-evaluate-world-only-ui-pixels.json).
  Remaining CPU share does not automatically reopen this item.
- **PERF-09 — partial:** diagnostics/gates/sources/backend already available;
  current samples show no contention that justifies SPSC or more buffering.
- **PERF-10 — partial:** observer and cadence matrix completed; composition
  before waiting rejected (worse p99 in 4/5 pairs). Criteria for physical
  60/high-Hz monitors, audio drift and input responsiveness remain.
- **PERF-11/12/13 — conditional, not implemented/accepted:** available costs do
  not yet justify new GPU filters or a GPU PPU. The optional zoom pool already
  exists; do not replace it with another or create threads per frame.

Current exports and hashes are in the handoff: 344 Windows/Wine tests and 345
Linux tests, 15 optional tests excluded per target. These are compatibility/
export results, not native Windows performance or proof that the original lag
is resolved. The [measurement workflow](PERFORMANCE_WORKFLOW.md) and
`docs/changes/` records retain details and limits. This audit repeats no builds
or benchmarks.


<a id="fixture-locali-e-prima-verifica-della-battaglia"></a>

## Local fixtures and first battle check

The new local `saves.zip` archive and any extraction to `saves/` are ignored by
Git; `.sav` and `.ffstate` are already excluded elsewhere too. Checks use
copies in `target/`: no personal progress enters project records, Docker context
or packages.

The supplied slot was actually loaded successfully with the Linux runtime
exported from the container. The cartridge save also allows the world map to be
opened through Saved Game > Load. Both scenes were visually recognized, not
inferred solely from file names.

An exploratory battle session explicitly resumed from the host menu and allowed
to advance for 20 seconds produced 1,074 core samples, 171,840 scanlines and no
invalid PPU intervals. The profile also includes startup: core median 23.084 ms
and scanline median 17.545 ms; p95 25.520 and 19.434 ms respectively. Zoom ran
no additional rows. This is a **preliminary Xvfb/llvmpipe measurement**, not a
Radeon baseline or an identified enemy-turn replay. Attempts that stayed in the
host menu are excluded from this evaluation.

CPU PPU is therefore a priority for further investigation in PERF-02/08:
separate BG/OBJ rendering, composition and panel extraction; particularly
check repeated per-row passes in `capture_dialogue_part`. That function sorts
BG/OBJ candidates for every pixel despite constant per-row BG priority: measure
this work separately and compare a top-two-layer selection with the reference
before changing it. Neither the dominant function nor a benefit from moving to
GPU is yet demonstrated. The runner now supports `--state` in Python comparison
too: 600 additional frames, 120 warm-up and five native pairs on the same build
produced identical framebuffer/OAM, final state and counts. Work medians are
20.950/20.856 ms; the two diagnostic captures attribute about 76% of core time
to scanlines. Loading and startup are excluded from the sample. A separate
check also verifies zoom relative to the snapshot. This is an idle-battle
baseline without an identified enemy turn: it is not a performance gain and
PERF-01/02/03 remain open.

<a id="correzione-delle-condizioni-di-misura-e-selettore-ui-valutato"></a>

## Corrected measurement conditions and evaluated UI selector

An automatic VS Code npm search in the workspace was also following ignored
trees and had run for over half an hour, occupying about ten cores. It was
stopped after its automatic origin was identified; `npm.autoDetect` was
turned off only in ignored local workspace settings. No global setting or
preexisting preference changed. The portable build with original renderer then
passed load, Undo, reload and closure in Xvfb/llvmpipe: the observed idle-battle
counter is approximately 59–60 FPS INT/EXT. This is not a Radeon or AI-turn
measurement. The [workflow](PERFORMANCE_WORKFLOW.md#host-load-before-timing)
describes the check.

With **the same 1,200-frame interval**, snapshot, baseline binary, translation,
input, audio and CPU affinity, baseline median is 18.655 ms under that load and
10.777 ms after removing it. Previous 600-frame checks around 21 ms remain
historical data under contention, not an interference-free baseline; do not
compare different lengths directly. This is a development-environment change,
not game-code acceleration.

Two selectors for only the top two UI layers were evaluated in place of two
per-pixel sorts. Both retain exact results across 196,608 priority/presence
combinations and replays. On the system without the scan: the compact-key variant
worsens median by 1.64%; the simple variant improves median by 1.56%, but the
diagnostic capture shows no peak advantage (higher p95/p99). The overall benefit
does not justify replacing the standard sort: **neither variant was retained in
the runtime**. Individual peak profiles do not prove a universal causal
regression; they are nevertheless insufficient to accept a pacing optimization.

The world map was reached again from a `.sav` copy, recording input and creating
a visually classified local snapshot. Replay from that snapshot passes five
1,200-frame pairs on the original renderer: median about 2.1 ms/frame, identical
final state and images, no over-budget frames in the two profiles. This calibrates
the same binary; it is not an improvement. Files remain under `target/`, outside
Git and packages. Before other PPU/GPU changes, identify the slow enemy turn on
the system without this interference; PERF-01/02/03/08 remain open.

<a id="replay-con-attivazione-nemica-identificata"></a>

## Replay with identified enemy activation

A new local sequence was prepared from a copy of the supplied snapshot: suspend
save using the in-game command, new start with recording, resume battle and wait
for player units. The sequence includes an enemy monster's activation and
movement, then return to a player-character menu. These steps were observed in
the window; this is no longer only the idle-battle replay.

An intermediate snapshot and rebased subsequent inputs bound 3,530 frames. The
headless replay reaches the same final frame and framebuffer hash as the GUI
session; the final image was inspected too. Both A/B checks agree on framebuffer
and OAM for every frame. The
[workflow](PERFORMANCE_WORKFLOW.md#record-a-battle-from-a-supplied-snapshot)
describes the procedure without removing the guard against loading a snapshot
while recording. All fixtures remain ignored.

The full comparison passes five alternating pairs on the same code, with 120
warm-up frames: medians 8.346/8.290 ms for the 3,410 measured frames, identical
final machine state and execution counts. The two separate profiles have zero
frames over 16.74 ms, p99 10.055/10.129 ms and maxima 12.535/16.518 ms.
Scanlines occupy about 73% of core time (6.07/6.08 ms mean); the rest about
2.25/2.24 ms, with no additional zoom rows. This is a new baseline, not a
speedup: the difference between identical binaries calibrates noise. The first
run interrupted by an output error remains retained as partial; these results
come from the later completed comparison.

This interval includes player input, animations and waits: it does not measure AI
computation alone and does not prove this is the originally reported slow turn.
PERF-01 remains open for other scenarios and slowdown characterization; no
runtime acceleration is yet attributed to the replay.

<a id="attribuzione-delle-fasi-ppu-completata"></a>

## PPU stage attribution completed

`FFTA_PPU_STAGES=1` adds disjoint intervals for BG, OBJ, composition, UI
extraction, UI setup and remainder. Comparison of the new diagnostic code with
the baseline passes five pairs on the existing replay, per-frame images/OAM,
machine state and counts. Medians with collection off 8.289/8.317 ms (+0.34%
difference in this test). Detailed capture uses 8.373 ms/frame, about +0.67%
relative to the same candidate's median: a single estimate subject to noise, not
a universal overhead guarantee.

Means per frame: BG 0.246 ms, OBJ 0.288 ms, composition 0.531 ms, UI capture
4.912 ms, UI setup 0.071 ms, scanline remainder 0.078 ms. UI extraction occupies
about 80% of scanlines and 59% of core time. No invalid intervals or over-budget
frames in the two profiles; baseline/candidate core p99 10.237/10.116 ms. This
closes the PERF-02-PPU-STAGES diagnostic substep on Linux, not all of PERF-02 or
a new optimization. The profile guided precomputation of windows per row in
`capture_dialogue_part`, already completed in the next section; rejected front
selectors were not repeated.

<a id="prima-ottimizzazione-dellestrazione-ui-mantenuta"></a>

## First UI extraction optimization retained

PERF-08-UI-ROW-INVARIANTS is complete: window registers and vertical membership,
native origin and modal flag are computed once per call. WIN0/WIN1/OBJ/outside
precedence and circular intervals remain identical; 2,457,600 cases compared with
the previous reference, 169 core and 48 runtime tests passed. No persistent cache,
guest timing change or new state version.

Five pairs on the existing replay: median 8.412 → 8.229 ms/frame, approximately
**2.18% less host work**, per-frame images/OAM, final state and counts identical.
The separate profile attributes mean UI capture 4.941 → 4.779 ms. Core p95/p99
improve (10.117/10.540 → 9.890/10.346 ms); rare over-budget frames are 2/4 in
individual captures, maxima 18.489/18.634 ms. This does not prove peak
elimination or resolution of the originally reported lag.

The new code also passes five world-map pairs: medians 1.958/1.967 ms (+0.43%),
identical state/images/counts and zero over-budget frames in profiles. An
Xvfb/llvmpipe session with the new runtime passes load, Undo, reload, resume and
closure; previews and messages inspected, detailed profiling without invalid
intervals, original slots and ZIP unchanged. Native Windows, Radeon and physical
controller remain outside this validation. Do not repeat these tasks without a
new change or uncovered criterion.

<a id="riuso-dei-bg-nativi-ui-mantenuto"></a>

## Native UI BG reuse retained

PERF-08-UI-PREPARED-BG-ROWS is complete: native BG2/BG3 are prepared once per
scanline and shared among panels/passes. Backing filters a private text copy;
Stats retains its null mask. The reference with fresh preparation per pass
matches final machine state across 48 configurations and all 160 rows, with
nonempty planes, filters and BG enabled in different combinations. 170 core and
48 runtime tests passed.

Five pairs on the existing replay against the precalculated-window version:
**8.244 → 7.903 ms/frame, 4.14% less host work**. Per-frame images/OAM, final
state and counts identical. Mean UI capture 4.781 → 4.476 ms; core p95/p99
9.895/10.328 → 9.487/9.909 ms. No over-budget frames in the two profiles, but
isolated maximum rises from 10.862 to 12.277 ms: no universal promise of peak
elimination.

Five world-map pairs: medians 1.962/1.950 ms, small difference, identical
images, state and counts, zero over-budget frames. New runtime also verified for
load/Undo/reload/resume/closure in Xvfb/llvmpipe, inspecting previews and game;
989 frames and zero invalid PPU intervals. Original slots and ZIP unchanged.
No new persistent data, ABI or state format; no native Windows or Radeon
measurement is inferred from these results.

<a id="righe-ui-vuote-ordinamento-e-blending-evitati"></a>

## Empty UI rows: sorting and blending avoided

PERF-08-UI-EMPTY-ROWS is complete. After preparation, sprite capture and flag
reset, a row with all UI sources empty is explicitly cleared and does not sort/
blend world candidates: these cannot become UI pixels. No frames, hardware
scanlines, cycles or guest decisions are skipped. The old complete loop remains
the test reference; nonempty rows retain it unchanged. 171 core and 48 runtime
tests passed, including previous content to clear, Stats masks, modals and world
object blending.

Five pairs on the existing replay against the shared-BG version:
**7.891 → 6.194 ms/frame, 21.50% less host work**. Per-frame images/OAM, final
state and counts identical. Mean UI capture 4.467 → 2.770 ms; core p95/p99
9.481/9.906 → 7.497/7.793 ms, zero over-budget frames in the two profiles.
Isolated maximum rises 10.770 → 14.502 ms: do not infer a guarantee on rare
peaks or resolution of the originally reported lag.

Five world-map pairs: 1.978/1.942 ms, equal images/state/counts and zero
frames over budget. The new runtime passes load/Undo/reload/resume/closure in
Xvfb/llvmpipe, with previews and game inspected; 987 frames and zero invalid
PPU intervals. Original slots and archive remain unchanged. Current Windows/
Linux build validation was completed in the next section; updated Docker
archives incorporate the current renderer.

<a id="build-e-archivi-desktop-aggiornati"></a>

## Desktop builds and archives updated

PERF-03-CURRENT-DESKTOP-BUILDS is complete within its scope: Windows GNU/MinGW
in Docker with **317 Wine tests**, Linux Bookworm with **318 tests**; 9 optional
cases ignored per platform. Archive integrity, paths and contents verified:
only empty Data scaffold, no ROM/save/recording/game cache or test DLL, TinyCC
and licenses included. PE resources and imports unchanged; executable ELFs with
maximum GLIBC 2.35 requirement. Recorded input hashes unchanged.

The new exported Windows runtime passes the existing 3,530-frame replay:
per-frame framebuffer/OAM, final state and counts match both previous Windows
runtime and already validated Linux build. This is a correctness check under
Wine, not a native Windows performance comparison. The new Linux archive passes
cold-cache startup, load/Undo/reload/resume/closure in Xvfb/llvmpipe: previews
and game inspected, 1,001 frames and zero invalid PPU intervals; original slots
and ZIP unchanged.

Current archives and BUILD-INFO are updated in `dist/docker-windows/` and
`dist/docker-linux/`; previous versions remain in the local validation directory.
No remote publication. Native Windows, physical controllers, hardware GPU/
pacing and scenarios not yet characterized, including high zoom, remain open:
do not repeat the ordinary baseline to cover them.

<a id="osservazioni-sulla-revisione-iniziale-storiche"></a>

## Initial revision observations (historical)

- The GPU is already used for scaling, grid and UI-layer composition:
  [present.rs](../../crates/screen/src/present.rs), `Presenter::present`, and
  [ui_layer.rs](../../crates/screen/src/ui_layer.rs), `Gpu::draw`. GBA rendering,
  layer preparation and temporal filters remain on CPU.
- `Gpu::update` reloads UI, world and LUT textures on every layer update. The
  LUT contains 32,768 colors; the [FPS](../../crates/recomp/src/fps.rs) counter,
  redrawn approximately once per second, also goes through this upload on every
  presentation. Layers may have different worlds/LUTs: do not unify them blindly.
- In the [play.rs](../../crates/recomp/src/play.rs) baseline, `emu_ema_ms`
  includes execution, UI update and `Temporal::push`, but excludes
  `Temporal::compose`, presentation, waits and part of audio/persistence work.
  The `native %` value compares native blocks with individual interpreted
  instructions: it does not measure CPU time percentage. The first tranche
  replaces this percentage with separate counts and adds per-stage CPU profiling.
- An error returned by `Presenter::present` causes the GPU presenter to be
  abandoned for the rest of the session. Frame-acquisition timeout currently
  takes this path too. This is a possible cause of sudden degradation to check
  in the log; it is not yet the diagnosis of the reported lag.
- [Temporal](../../crates/screen/src/blend.rs) processes emulated frame history,
  with six frames for Smart mode. Blending uses colors already encoded for the
  display: moving it to linear space would change the image.
- The [PPU](../../crates/gba-core/src/ppu.rs) already has a parallel path only
  for additional zoom rows, with equivalence tests. It is disabled by default
  on desktop; the persistent pool requires the `parallel-ppu` feature,
  otherwise scoped threads are created during rendering.
- The [previous benchmark](../windows_runtime_performance.md) measures a
  headless introductory sequence under Wine. It does not demonstrate enemy-turn
  cost, window smoothness or native Windows behavior.

<a id="p0--rendere-il-problema-riproducibile"></a>

## P0 — Make the problem reproducible

- [ ] **PERF-01 — Record representative scenes and baselines.**
  Low risk; first step, no dependency. Prepare separate replays for a slow enemy
  turn, battle movement/effects, dialogue/menus and high zoom. Keep ROM, saves
  and recordings only in ignored local directories. Record build, backend,
  resolution, zoom, filters, VSync, audio mode, driver and cold/warm cache.
  Extend [benchmark_windows_runtime.py](../../scripts/benchmark_windows_runtime.py)
  for correct platform detection and reuse deterministic comparison.
  **Complete when:** the same turn repeats from controlled input/initial state
  and there are at least five alternating runs for comparison. Distinguish host
  slowdown from a wait already present in the game when emulation maintains
  approximately 59.73 frames per second.

- [ ] **PERF-02 — Measure times, spikes and waits separately.**
  Low risk; depends on PERF-01. Add opt-in diagnostics in `play.rs`, `main.rs`,
  PPU and presenter: guest execution, PPU, UI preparation, filters, audio handoff,
  save, upload, surface acquisition, command submission and clock wait. Avoid
  per-instruction timers and synchronous per-frame logs. Do not add nested
  intervals as if independent. Count native blocks, fallback, intentionally
  interpreted hooks and incomplete frames separately.
  **Progress:** the `RECOMP_COST_FROM` inventory now uses the runtime's same
  native dispatch, preserving audio/sprite/mouse hooks. It distinguishes native
  blocks, missing translations, system steps and interpreted hooks; it attributes
  guest cycles, not host CPU time, and remains separate from timing comparisons.
  Per-frame CPU times for GBA scanlines, zoom rows and the rest of the core
  (`--ppu-profile` / `FFTA_PROFILE=1`) are also available, with count checks,
  bounded samples and no guest clock/state change.
  **New completed substep:** opt-in callback profiling separates mutex wait,
  rendering, total and arrival interval, with faithful hold/drop counters for
  play, priming, pause and fast-forward. Bounded storage and report after
  teardown; DSP/sample/queue parity verified for different formats and channels.
  The Linux CPAL 44.1 kHz check measures total p99 0.500 ms and mutex wait
  0.000912 ms during play, with no callback exceeding its period. The 1790
  faithful holds in 6 callbacks include transitions/resumption: they are not
  evidence of audible underrun or isolated analysis of the AI turn. Median
  synthetic overhead is about 129 ns/callback (+0.34%). Later substeps
  (PERF-09/10) added frame gate, enhanced counters, backend observation and cadence.
  Original slow scene, controlled realtime overhead, audible meaning of source
  data and full hardware/native Windows coverage remain. Exports pass 328
  Windows/Wine and 329 Linux tests; profile on/off, loads/Undo/FF and benchmark
  audio bypass pass on portable Linux.
  **Complete when:** report contains median, p95, p99, maximum, frames over
  16.74/33.49 ms, turn duration and audio underruns; diagnostic overhead measured.
  GPU time uses optional timestamps and deferred read where supported: `submit`
  time is not GPU execution time. [wgpu 26 Queue APIs](https://docs.rs/wgpu/26.0.1/wgpu/struct.Queue.html)
  describe submission, transfers and timestamps.

- [x] **PERF-03 — Establish correctness checks and A/B procedure.**
  **PERF-03-FRACTIONAL-UI-TEXELS — completed.** Integer RG32Uint maps computed
  using existing CPU formulas eliminate fractional-crop differences without
  changing zoom/FOV. Full keys and device limits; conservative transparency
  bounding box and scissor avoid unnecessary work. Radeon matrix of 32
  zoom/resize/DPI states and shared-resource/invalidation tests pass with exact
  CPU parity. 3530-frame replay and worldmap preserve images/state/counters;
  GUI load/Undo/reload/close and Wayland pass. Five pairs improve 8.230 ->
  7.775 ms (-5.53%); the previous prototype without scissor (+4.17%) remains
  rejected. This result does not prove AI acceleration or native Windows
  performance. Docker export: 332 Windows/Wine tests, 333 Linux (14 optional
  excluded each); integrity, privacy, PE imports and GLIBC 2.35 verified.
  Portable Linux with cold cache: load/Undo/reload/resume/closure and screens
  checked.
  Low risk; depends on PERF-01. Reuse replay, interpreter/native comparison,
  framebuffer/OAM and existing savestate tests. For core changes also compare
  registers, RAM, IO, VRAM, guest cycles, saves and deterministic audio samples
  before the device. Block counters can change with a different translation:
  do not confuse them with game state.
  **Complete when:** each comparison has configuration and binary hashes,
  verification separate from timing and a procedure to return to baseline.
  Check that the `max_steps` limit does not advance temporal history and
  completed-frame counters when `run_frame_native` does not complete a frame.

<a id="p1--correzioni-circoscritte-e-lavoro-ridondante"></a>

## P1 — Bounded fixes and redundant work

- [ ] **PERF-04 — Recover from temporary graphics errors.**
  Medium risk; depends on PERF-02/03. In `Presenter::present` and
  `PlayApp::present_rgba`, distinguish timeout, surface needing reconfiguration,
  device loss and permanent failure. An isolated timeout should be able to skip
  that presentation and retry without permanently disabling the GPU. Define
  bounded retries, counters and a working software fallback; avoid recreation
  loops or continuous logs.
  **Progress:** timeouts implemented with skipped frame without losing GPU, one
  reconfiguration attempt per frame on Lost/Outdated and fallback after three
  consecutive failed recoveries. Out-of-memory/unknown errors trigger immediate
  fallback; only successful presentations count in FPS EXT. Tests simulate
  prolonged timeouts, recovery, budget and permanent failures. The initial
  Xvfb/Radeon-GL Invalid surface case is now handled with balanced error scopes:
  failed configuration returns to software renderer without panic. The same
  check covers resize, VSync and recovery reconfiguration; VSync returns a Result
  to the caller. Verified actual fallback case, resize/VSync ON-OFF on valid
  Vulkan and replay/closure on Radeon Wayland.
  **PERF-04-SKIPPED-UPLOADS:** fixed pending uploads on skipped frames: submit
  transfers only, without draw/present/timestamp or GPU waits. The frontend
  avoids new preparation for zero-size windows and preserves the benchmark
  error when a presentation is skipped. Batch consumption tests and Presenter
  check on 1200 zero-size batches pass; 492 MB logical traffic, stable RSS
  between batches 600 and 1200. Normal GUI load/Undo/reload/close passes; final
  exports verified (333 Windows/Wine tests, 334 Linux, 14 optional excluded
  each), with cold-cache GUI check on portable Linux and screen inspection.
  This is not a measurement of AI-turn acceleration or compositor minimization.
  **PERF-04-MINIMIZE-RESTORE — completed for Linux X11 and Wine/Vulkan.**
  New ROM-free diagnostic: three minimizations/restores observed by the WM,
  three resizes and VSync ON/OFF, successful rendering after every restore and
  correct teardown. Independent Iconic/Normal state verification. Linux
  Openbox/llvmpipe: 27 Presented, zero Skipped; Windows GNU under Wine with
  Vulkan: 18 Presented, 9 Skipped, zero dimensions during minimization. The
  first Wine D3D12 attempt fails device creation: it remains a negative result,
  not native Windows coverage. Current portable Linux also passes three cycles
  in battle followed by Undo/reload/resume/closure. No runtime change or package
  regeneration required.
  **PERF-04-MONITOR-TRANSITIONS — completed for virtual X11/Wine outputs.**
  Diagnostic `--monitors` mode: two distinct monitor handles, A/B/A/B/A sequence
  observed and confirmed by X positions, actual resizes between 640x360 and
  640x480, VSync ON/OFF and correct teardown. Xorg/dummy with 1280x720/1280x960
  outputs: 20 Presented, zero Skipped on Linux and Windows GNU/Wine/Vulkan.
  Portable Linux unchanged: battle, four moves, Undo/reload/resume/closure
  verified with screens. No runtime change or new benchmark. The first scenario
  with unconfigured keyboard was not accepted.
  **PERF-04-DEVICE-LOSS — completed for controlled device destruction.**
  Private test with actual device destruction: existing frames/UI returned an
  error, but creating a new UI caused panic on invalid textures. Presenter now
  retains the first uncaptured/loss error from its Device and passes it to
  software fallback; no per-frame GPU allocations or waits. Tested plain/cached/
  new UI and resize, error-scope isolation, persistent error, refusal of new
  writes/VSync and software presentation. Native and Windows/Wine/Vulkan pass;
  normal GUI without unexpected fallback and Radeon/Wayland 3530-frame replay/
  state/counters unchanged with correct closure. Docker export: 333 Windows/Wine
  tests and 334 Linux (15 optional excluded), archive/privacy/DLL import/GLIBC
  2.35 checks and cold-cache portable Linux GUI pass; previous copy retained.
  Controlled destruction does not prove every asynchronous driver reset or
  physical fault. Wayland and native Windows minimize/restore, physical screens,
  mixed-DPI/GPU migration/hotplug and device loss during execution remain.
  **Complete when:** resize, minimize/restore, monitor change and simulated
  transient error recover without stalls. Also verify Wayland closure after
  recovery and fallback, preserving release order in `exiting`.

- [x] **PERF-05 — Update UI textures and LUTs only when they change.**
  Low/medium risk; depends on PERF-02/03. In `ui_layer.rs`, `fps.rs` and
  `refresh_ui_layers`, introduce explicit content revisions: LUT when colors
  change, FPS when text changes, UI when it actually changes. Separate layout
  changes from pixel changes. Share world upload only for layers with the same
  source, dimensions and generation; the statistics panel has its own background.
  **Outcome:** LUT/FPS revisions, UI packing input cache and world reuse within
  a single presentation are implemented and verified. Worlds with the same
  source and dimensions share texture/upload; distinct sources, including Stats,
  remain separate. All unique worlds update on every call: no persistent
  address-based cache. Payload 530 -> 316 MB in replay, five pairs 8.251 ->
  8.219 ms (-0.39%) with better p95/p99. Independent/shared GPU, mutations,
  source graphs, dimensions, recreation and GUI load/Undo/zoom pass; export 329
  Windows/Wine and 330 Linux tests, including GUI on cold-cache portable Linux.
  The later fractional-coordinate change is described in PERF-03.
  **Complete when:** byte/upload counters demonstrate reduction and images,
  HUD, colors, menu, zoom and load/Undo remain equivalent. Invalidate revisions
  on ROM change, state load and GPU recreation. Assess reusable staging only if
  justified by profile: uploads also have copy/allocation costs
  ([wgpu documentation](https://docs.rs/wgpu/26.0.1/wgpu/struct.Queue.html#method.write_texture)).

- [x] **PERF-06 — Reuse buffers and intermediate results measured as costly.**
  Low/medium risk; depends on PERF-02/03. Check `handoff_audio`: `mem::take`
  consumption of `fifo_tap` abandons the buffer after transfer. Assess draining
  that retains capacity. Also measure repeated panel packing/bounds and retain
  them by content revision. Avoid caches based only on addresses or allocation
  optimizations present solely in tests.
  **Experiment evaluated:** scanning only visible extremes per row reduces
  function cost on synthetic panels, but two 3,000-frame end-to-end comparisons
  showed a candidate regression. Not enabled in runtime: retained only in tests
  with equivalence and a five-alternating-pair release microbenchmark.
  **Final outcome:** FIFO reuse already verified; UI packing cache retained with
  complete keys and memory bounded to current buffers, invalidation on restore
  and tests against direct packing. Five battle pairs measure -1.41% in GUI loop,
  UI preparation 0.157 -> 0.122 ms. Replay, world map, GUI/Undo/style and
  Linux/Windows-Wine builds pass; no audio event flow change. Evidence: October 21
  UI input cache record.
  **Complete when:** allocations or time decrease in the measured path without
  continuous memory growth, lost audio events or stale panels.


<a id="p2--ottimizzare-il-collo-di-bottiglia-dimostrato"></a>

## P2 — Optimize the demonstrated bottleneck

- [ ] **PERF-07 — Reduce expensive fallback during enemy turns.**
  Medium/high risk; depends on PERF-01/02/03. Profile `run_frame_native`,
  `BlockTable`, interpreter and generated code in
  [recomp-core](../../crates/recomp-core/src). ROM/IWRAM lookup is already
  direct: do not replace it without evidence. Use the fallback inventory to find
  missing entries, distinguishing them from intentionally interpreted hooks.
  **PERF-07-FALLBACK-CANDIDATES — triage completed, no translation changed.**
  New inventory on the already recorded enemy replay: after 120 warm-up frames,
  22.63M native blocks, 79.14M missing-translation steps, 55.91M hooks and 6820
  system steps; 3804 distinct missing entries over the entire replay. Parity of
  3530 frames/state/counters verified. Guest cycles are not host cost.
  Optimized native CPU sampling with symbols, 3928 samples/zero lost, indicates
  capture_dialogue_part 31.52%, sprites 8.63%, composition 7.76%; interpreter
  step_inner 6.21% and Machine::step 4.56%. It does not isolate the cost of
  individual missing entries. Prioritize remaining UI extraction work (PERF-08),
  without repeating rejected selectors or rewriting lookup/ABI based only on counts.
  **PERF-07-POST-PPU-HOST-TRIAGE — completed.** New sample after retained PPU
  changes, without recreating fixtures/inventory/baseline: 3798 samples, zero
  lost, identical final state/counters. UI capture 31.20%, sprites 9.19%, compose
  8.85%, step_inner 7.64% self; cost percentages, not speedup or cost of
  individual missing entries. Isolated binary with symbols, normal runtime
  restored by hash. Rejected selectors and caches remain closed.
  **PERF-07-IMMUTABLE-ROM-SEEDS — rejected for production.** 18 already observed
  ROM entries, no new RAM seeds: 28859->29196 blocks. Baseline reconstructed
  byte-for-byte identical to previous translation. All 3530 frames/OAM,
  initial state and warm-up scene equal, but final full-state hash different.
  Test stopped before timing; runtime/cache/gamedb/archives unchanged.
  [Evidence](../changes/2026-10-21-evaluate-immutable-rom-entry-seeds.json).
  Reduced fallback is not evidence of speed or correctness. Before expanding
  coverage, fix and verify audio/timer order without weakening checks.
  **PERF-07-ROM-SEED-STATE-DIVERGENCE — diagnosis completed:** first difference
  at relative frame 38 (guest 8514), equal CPU/clock; two mixer samples and HLE
  verifier state differ. Diagnostic reproduces both original final hashes.
  ROM-free fixture: tick600 and 600*tick1 have equal final clock and DAC but
  different samples and timer bookkeeping. This sensitivity supports the event
  ordering hypothesis without yet proving the entire causal chain in battle.
  No core fix or new translation accepted; next criterion PERF-07-TIMER-AUDIO-ORDER,
  checking DMA, waits, cycles and audio before resuming performance measurements.
  [Diagnosis](../changes/2026-10-21-isolate-rom-coverage-audio-divergence.json).
  **PERF-07-TIMER-AUDIO-ORDER — completed and delivered.** Events at their own
  timestamp, audio/timer order equal to the single-cycle reference, DMA without
  video re-entry and subtracted cycles added to CPU work; halt/IntrWait uses the
  same path. Five new regressions, 179 native/Wine core and 66 runtime tests
  pass; native 3530-frame/OAM/counter replay identical to baseline. Correct state
  verified separately: only final audio/HLE/timer bookkeeping changes intentionally,
  not CPU/clock. Five pairs measure median +0.066607 ms (+1.157%): correctness
  fix, not speedup. Slot v1 and GUI load/zoom/Undo/reload/closure pass. Docker
  export 349 Windows/Wine and 350 Linux tests (15 optional excluded), integrity/
  privacy/compatibility verified; extracted Linux archive passes 3530 frames/
  corrected state and cold-cache GUI with slot v1/load/zoom/Undo/reload/closure.
  Dist archives updated only after checks, with previous versions retained. No
  commit or public release. New seeds still excluded: after the fix the first
  divergence is CPU/clock at frame 212; fixing audio alone does not justify
  expanding translation.
  [Fix and limits](../changes/2026-10-21-order-timer-audio-events.json).
  **PERF-07-NATIVE-FRAME-BOUNDARY — correctness-verified candidate.** Native
  code continued to the back-edge with IRQ already pending: reproduced without
  ROM too, with differing IRQ stack bytes. Budget guards at each entry and after
  store/shell eliminate the problem, retaining native batching away from events
  and separation from RAM-content checks. Translation cache 7, same savestate/
  ABI schema. Native 181 core/66 runtime tests and 11-case TinyCC regression
  pass; also Windows/Wine. Both ROM coverages now match in all state at every
  one of 3530 frames, including continuous queues with 7.7M samples per mixer/
  PSG/HLE and 931920 events per FIFO. Real CLI and previous images verified.
  Cost/GUI/export still to be accepted; timing triplets underway. No automatic
  seed expansion or new portable published.
  [Candidate and evidence](../changes/2026-10-21-preserve-native-event-boundaries.json).
  Extend translation only for code with guaranteed validity; mutable RAM and
  overlays require checks and invalidation.
  **Complete when:** AI replay takes less host time beyond measurement noise and
  preserves decisions, RNG, cycles, interrupts and final state. Update cache/ABI
  revision when the generated-code contract changes.

- [x] **PERF-08 — Reduce CPU PPU cost without changing timing.**
  Medium risk; depends on PERF-02/03. Profile `render_scanline`, tiles, sprites
  and `compose_into`, separating the original 240×160 pixels from widescreen/
  zoom extension. Reuse invariant data within the same scanline and limit
  processing to necessary areas. Assess decoded-tile caching only with complete
  invalidation for VRAM/palette writes, DMA, loading and restoration.
  **Complete when:** raw images and guest state are identical with windows,
  blending, semitransparent sprites and scanline effects too.
  **PERF-08-UI-PANEL-ROW-CLASSIFICATION — completed and retained.** Banner/
  bounds rules cached per call and membership computed only for the first
  eligible BG of each pixel. Lazy bounds, windows, blending, portraits, sort and
  state/ABI preserved. Native and Windows/Wine 173 core tests/5 optional excluded;
  parity of 3530 frames/state/counters even at zoom6. Five headless pairs:
  6.002->5.841 ms (-2.69%); five full GUI pairs: 7.767->7.591 ms (-2.26%), all
  better, including p95/p99. Worldmap and GUI load/Undo/zoom/reload/close pass.
  Distinct from already completed WIN0/WIN1 cache and rejected selectors; not
  evidence of resolving rare original lag or native Windows performance.
  Export 335 Windows/Wine and 336 Linux tests (15 optional excluded each),
  archive/privacy/DLL import/GLIBC 2.35 checks and cold-cache portable Linux
  load/Undo/reload/closure and screens verified. Previous versions retained;
  saves and translation cache compatible.
  **PERF-08-UI-WORLD-ONLY-PIXELS — completed and retained.** Skip sort/blend only
  when no candidate can enter the UI texture, still writing UiPixel::default to
  clear previous data. Distinct from already completed fully empty rows and
  rejected front selectors; implemented with full-state evidence for portraits,
  blending onto the world and previous data to clear. Native/Wine 174 core tests
  pass; 3530 identical headless frames with state/counters, five pairs
  5.846->5.600 ms (-4.20%). GUI 3530 identical frames and five pairs
  7.584->7.382 ms (-2.67%), all better. Worldmap 240 frames/state/counters and
  two diagnostic pairs pass; zoom6 identical over 3530 frames. Normal GUI
  load/Undo/reload/zoom/closure and screens pass without fallback. Docker export
  336 Windows/Wine and 337 Linux tests (15 optional excluded each); archives/
  privacy/DLL/GLIBC 2.35 and extracted cold-cache portable Linux pass. Local dist
  updated with previous archives retained.
  **PERF-08-COMPACT-UI-LAYER-ID — rejected, baseline restored.** Local u8 tags
  instead of usize, same algorithms. Native/Wine 174 core tests and all 3530 GUI/
  headless frames/state/counters identical. Five headless pairs: -1.348%; one
  diagnostic profile opposite (+1.367%). GUI with CPU profile: -0.454%, without
  diagnostics: -0.774%; both with one unfavorable pair and worse maxima. Modest/
  inconsistent GUI benefit relative to variability: no robust useful advantage
  demonstrated for changing already validated code. Previous source/binary
  restored byte-for-byte, evidence retained; archives unchanged. Do not repeat
  the experiment for context or merely because the parent is open. Off-screen
  effects affecting GBA state must remain active.

- [ ] **PERF-09 — Reduce contention and blocking work in audio.**
  Medium/high risk; depends on PERF-02/03. Measure the mutex shared between
  `build_stream` in [main.rs](../../crates/recomp/src/main.rs) and
  `handoff_audio`: the callback holds it while processing samples. Also check
  fast-forward path in [ff_audio.rs](../../crates/recomp/src/ff_audio.rs).
  The first measurement with the new observer shows mutex wait p99 below
  1 microsecond in the Linux sample: this alone does not justify an SPSC
  refactor. Reuse these data and the testable callback; do not recreate diagnostics.
  **PERF-09-INTERACTIVE-GATING — completed.** Optional guest-frame window for
  one traversal, pause/priming/fast-forward separated; load/Undo during capture
  interrupts it. Same idle battle, frames 8596:9196 after load 8476: 236/235
  faithful/enhanced callbacks, zero faithful source hold/drop/over-period; total
  p99 0.548/0.665 ms, mutex 0.000912/0.000711 ms. These are not AI times,
  inter-mode performance comparison or enhanced starvation/XRUN measurement.
  GUI with zoom/Undo instead observes 1309 holds in 7 playing callbacks
  (transitions and concurrent compilation): not a steady-state comparison or
  evidence of an audible defect. Native 60 runtime tests and container 338
  Wine/339 Linux pass, 15 optional excluded in exports; cold-cache portable
  Linux and off/benchmark behavior verified. Reuse the gate without repeating
  this implementation.
  **PERF-09-ENHANCED-SOURCE-EVENTS — completed.** Raw counters and configured
  destination for PSG/HLE/FIFO, with existing gate/context and per-request
  counters excluded from normal DSP specialization. Sample/queue/interpolator
  state parity verified with partial pairs, DAC hold, routing, inactive HLE and
  backlog too; 63 runtime tests pass. In idle window 8596:9196, 236 callbacks
  per mode and no empty reads from any source. In enhanced mode PSG/HLE are
  configured destinations, FIFO only warm; no enhanced destination in faithful
  mode. GUI with transitions instead observes 2741 empty PSG/HLE and 660 per-FIFO
  requests in one playing callback, without enhanced destination: data distinct
  from idle measurement, not audible defects inferred from counters. Export 341
  Wine/342 Linux tests (15 optional excluded each), archive/privacy/compatibility
  and cold-cache portable Linux load/Undo/off/benchmark pass.
  **PERF-09-BACKEND-OBSERVABILITY — completed for local PipeWire.** One pw-top
  with line output, PID->client->stream->link->sink association and stable
  serials. In running samples of idle battle (~9s/10 samples), pause (~2s/3)
  and resume (~5s/6), stream and sink ERR deltas are 0; cadence verified with
  maximum gap <1.001s. Sink already has ERR 1 at first idle sample: initial zeros
  in creating state are not a running baseline. This does not prove zero ALSA/
  hardware XRUNs or the cause of that count. The first buffered-output trace is
  excluded from phase attribution and retained as negative. Full app gate 235
  playing callbacks without hold/drop/over-period/device errors, battle and
  resume displayed and clean closure; private data unchanged. No runtime/CPAL/
  configuration change or rebuild required. Tested scenes do not justify SPSC
  queues/larger buffering; native Windows, hardware and genuinely slow scenes
  remain. Reassess CPU cost with PPU now optimized before adding audio changes.
  Do not confuse counts with cost. First reduce critical sections and
  allocations; consider bounded SPSC rings only if contention remains dominant.
  Preserve event order, capacity, reset, crossfade and device recovery.
  **Complete when:** spikes/underruns improve without increasing latency, with
  checks for pause, mode change, fast-forward, load/Undo and closure.

- [ ] **PERF-10 — Fix pacing only after distinguishing work and waits.**
  Medium risk; depends on PERF-02/03.
  **PERF-10-PRESENTATION-CADENCE — completed.** Optional observer on existing
  FPS timestamp: intervals of successful presentations and guest advances,
  maximum 18000 samples, frame window and lifecycle anchors. Surface skips visible
  as longer intervals; no gap across pause/priming/FF/zero-size, restore/zoom/
  resize or different backends. No pacing or clock changes when off.
  CPAL/Xvfb/llvmpipe matrix, frames 8596:9196: clock 600 presentations/599
  intervals, mean 16.7445/16.7427 ms, p99 ~17.96 ms; audio 237/236, multiple
  guest frames in every interval, mean 42.6843/42.6692 ms, p99 52.77/52.31 ms.
  Requested VSync is not hardware 60/high-Hz coverage; clock is already the
  Linux/Windows default. Silent fallback with ALSA invalid only in the process,
  66 runtime tests and 344 Wine/345 Linux tests pass (15 optional excluded in
  exports). Archive/privacy/compatibility and cold-cache portable Linux load/
  Undo/off/benchmark pass; archives updated. Do not repeat tool/matrix to
  reconstruct context.
  **PERF-10-POST-WAIT-COMPOSE — rejected, baseline restored.** Purity verified;
  five realtime pairs have worse p99 in 4/5 cases despite small p95 improvement.
  Previous source/runtime restored exactly, archives unchanged; no candidate
  verification remains pending.
  [Decision and measurements](../changes/2026-10-21-evaluate-composition-before-deadline.json).
  Do not reopen the attempt merely because the parent is open.
  Compare `FFTA_FRAME_PACING=clock`/`audio` modes, VSync and surface acquisition
  times. Verify interaction between sleep and presentation blocking before
  changing scheduling. Assess an event-loop-managed wait instead of sleep in the
  callback while retaining input handling. Separately measure any pauses caused
  by autosave every 300 frames: moving it to a worker requires write ordering
  and reliable flush.
  **Complete when:** cadence p99 improves on 60 and high-frequency monitors
  without audio drift, guest slowdown or less responsive input. Do not tie the
  GBA clock to monitor refresh or skip simulation.


<a id="p3--gpu-e-parallelismo-incrementali"></a>

## P3 — Incremental GPU work and parallelism

- [ ] **PERF-11 — Prototype LUT and temporal filters on the GPU.**
  Medium/high risk; depends on PERF-02/03/05. Proceed only if these stages
  occupy a significant share of the CPU budget. Keep core and BGR555 framebuffer
  on CPU; upload the raw frame and use LUT/history in textures. Start with
  Off/Simple, then Smart and finally Persistence. Prefer a simple graphics pass;
  a compute shader is not mandatory. Preserve rounding and blending in the
  already encoded color space.
  **Complete when:** total CPU+transfer+GPU cost and p99 improve, with CPU/GPU
  comparisons and CPU fallback. History advances for every guest frame, even if
  a presentation is skipped or fast-forward produces multiple frames. Pause,
  load, Undo, zoom and LUT change must retain current behavior. No synchronous
  per-frame readback; previews and saves must work without introducing GPU waits
  in the normal game loop.

- [ ] **PERF-12 — Evaluate existing zoom parallelism on desktop.**
  Medium risk; depends on PERF-02/03/08. Try `render_zoom_parallel` only on
  additional rows with guest state frozen in VBlank. Use a persistent pool, a
  measured threshold and the serial path for small jobs. Do not add threads per
  frame. **Complete when:** raw equivalence including panels and blending, no
  concurrent guest mutation and advantage on CPUs with few and many cores,
  without worsening audio or p99. The `parallel_rows_match_serial...` test is a
  starting point, not desktop performance evidence.

<a id="p4--esperimenti-da-rimandare"></a>

## P4 — Experiments to defer

- [ ] **PERF-13 — Extended GPU PPU, only with strong residual evidence.**
  High risk; depends on measurements after PERF-08/11/12. First assess an
  isolated portion of host extensions at a stable snapshot. A complete GPU PPU
  must correctly represent register, VRAM, palette, window and DMA changes
  during the frame: an end-of-frame snapshot is not generally sufficient.
  CPU reference comparison and an experimental path disabled by default are
  required. **Complete when:** substantial repeatable advantage including
  synchronization costs, equivalence and recovery across backends. Reject the
  experiment if complexity exceeds benefit.

The initial plan would not put guest AI on the GPU: the current path executes
ARM/Thumb instructions with memory, RNG and interrupt dependencies. Turning it
into a kernel or replacing AI with a host rewrite would require new equivalence
proof. Arbitrarily parallelizing CPU, PPU and audio, using fast-math or removing
safety checks to increase FPS also introduces disproportionate risk relative to
the preceding options.

<a id="regole-per-accettare-ogni-intervento"></a>

## Rules for accepting each change

One change at a time, with a small diff and clear responsibility. Extract
modules only when helpful to the current change; avoid massive `play.rs`/PPU
refactors alongside an algorithm change. Reuse `wgpu`, existing tools and tests;
new dependencies or `unsafe` require a concrete need.

- Correctness: replay and PERF-03 checks pass; serial core tests for shared
  display globals. For GPU filters, specify justified tolerances beforehand;
  no tolerance for guest state or saves.
- Performance: repeatable benefit beyond run-to-run variability in the target
  scene, without significant regressions elsewhere. Retain unfavorable
  measurements too; reject changes without useful benefit.
- Compatibility: Ubuntu Wayland/RADV and X11, software fallback, native Windows
  where available; Wine is a separate check. Untested alternative backends and
  hardware must remain explicitly unverified.
- Persistence and lifecycle: load/Undo, backup/restore, ROM change, portable
  relocation, resize and clean closure according to the affected area. Host
  caches and resources must be reconstructible and invalidatable. Change
  savestate version only when its contract changes, not for a filter.
- Project memory: record actual results, limitations and keep/reject decision in
  `docs/changes`, then regenerate the changelog. A TODO remains open until
  implementation and verification; AndroidPort requires separate work.

**Recommended first tranche:** PERF-01/02/03, then PERF-04/05/06. Reorder later
priorities using data: PERF-07 if turn execution dominates, PERF-08/12 if zoom
dominates, PERF-09 if audio dominates, PERF-10 if waits dominate, PERF-11 if LUT
and filters dominate.
