<a id="todo--bug-fix-e-funzionalità-desktop"></a>

# TODO — Desktop bug fixes and features

Analysis and progress as of October 21, 2026, based on the code and the capture
provided by the user (the image was removed from Git history after diagnosis).
Checkboxes distinguish verified work from still open tasks; proposals in later
sections remain to be developed.

Scope: desktop workspace, with Windows as the validation priority.
`AndroidPort/` is a separate tree. To resume development, consult
[TASK_HANDOFF.md](TASK_HANDOFF.md): reuse checks already completed in
[PERFORMANCE_TODO.md](PERFORMANCE_TODO.md), without reopening rejected experiments.

**BUG-01-LAWS** is fixed for the observed layout; **BUG-02-FPS** is explained
for the confirmed Fast case. **BUG-03-FAST-AUDIO** has transition reset,
effective consumption and a bounded Normal queue; listening/event/hardware
coverage remains. **FEAT-01-MUTE** and **FEAT-02-FULLSCREEN** are implemented,
with explicit validation limits. **FEAT-03-TACTICAL-OVERLAY** remains open
(P3, verified data and display first, then prediction). The latest request adds
the **FEAT-04-ROM-DATA** plan: extraction, decoding and direct data use, then
mods. The ROM-data plan is separate from the fixes described here.
**FEAT-RESTART-ROM** is implemented and verified on Linux/Wine.
An open parent does not reopen completed children.

<a id="bug-01-laws--schermata-delle-leggi-in-battaglia"></a>

## BUG-01-LAWS — Battle laws screen

**Status:** fix implemented for the earlier layout and the compact character
assignment variant reproduced from slot3; extended coverage below remains open.
**Priority:** P1. **Complexity:** medium.

**Verified cause.** The laws table uses BG2=5C08/BG3=4A09, like the action menu,
but does not enable WIN0 for help. The old `Notice` recognition did not admit
it. Without an edge HUD the separation failed; with the HUD the action layout
and X=160 fallback could trigger, producing the reported split content and
empty right panel.

**Implemented solution.** `laws_bounds` in
[ppu.rs](../../crates/gba-core/src/ppu.rs) recognizes the table frame, rows and
divider without reading translated words. It reuses the central `Notice` layer
for the table and help, including the selector and continuation indicator; the
remaining HUD stays separate. The host subtype `laws_layout` in
[ffta_dialogue.rs](../../crates/gba-core/src/ffta_dialogue.rs) is recalculated
every frame and is not serialized. Full geometry is unaffected by Length or the
side-panel clamp. Existing ROM, terrain and register checks remain; incomplete
signatures retain the old path. See [documentation and validation](../ffta_laws_layout.md).

- [x] Reproduce list and help with the EUR ROM and local saves, in Italian and
  English; observe BG, windows and sprites. Keep data and captures under
  `target/laws-fix/`, ignored by Git.
- [x] Fix classification, pixel ownership and both indicators, reusing modal
  panel layout and mouse mode.
- [x] Verify text on both sides of the old cut, HUD present or absent, three
  widths, exact native recomposition and unchanged VRAM/OAM/framebuffer; the
  regression fails when recognition is disabled.
- [x] Verify 72 snapshot cases: list/help, 3:2/16:9/ultrawide, zoom 1 and 2,
  Size 0.5/1/2 and Length 1/2. Native recomposition: zero different pixels.
  GPU/software parity: zero differences in 24 synthetic cases, including
  resizing between even/odd dimensions and changes in Size.
- [x] Run native core and runtime suites, Windows/Wine core suite and final
  snapshot byte regressions; inspect list/help in Linux and Windows/Wine
  frontends and verify loading and normal exit.
- [x] **BUG-01-LAWS-DEPLOYMENT**: reproduce the user's slot3 during assignment;
  recognize WIN0/compact table and WIN1/HELP without splitting columns, include
  selector 037C and keep the HUD separate. Linux/Wine suites, real screenshots
  and scene Load/Undo/reload verified.
- [ ] Complete additional coverage: native Windows, USA ROM, other law
  arrangements/counts and zoom above 2.

**Closure criterion:** the verified layout keeps list, text and indicators
intact and consistent with the original framebuffer. The additional checks
listed remain explicit and do not reopen the validated fix. Portable packages
were not updated.

<a id="bug-02-fps--differenza-tra-framerate-interno-ed-esterno"></a>

## BUG-02-FPS — Difference between internal and external frame rates

**Status:** diagnosis completed for the reported case: the user confirms the
difference only with Fast active; the behavior is expected. An explanation was
added to the menu FPS item. **Priority:** P1 for the completed diagnosis.

**Evidence and analysis.** [fps.rs](../../crates/recomp/src/fps.rs) counts
completed guest frames (**INT**) and successful host presentations (**EXT**)
over wall time. EXT does not measure the monitor's physical refresh. The loop
in `play.rs` can simulate multiple frames before presenting one: the difference
is therefore expected in fast-forward. At 4×, if load permits, approximately
239 INT and 60 EXT are consistent as a theoretical example. Local 4× captures
show 128.9 INT / 32.2 EXT and 130.0 / 32.5, returning to approximately
59.7 / 59.7 without Fast. The ratio remains 4:1 even when the PC does not reach
239 INT. Block-based audio pacing and skipped presentations can also separate
the two counters.

The attached capture shows **59.7 INT / 59.7 EXT**: it documents the graphical
issue, not the reported divergence. Counter tests,
[FPS documentation](../fps_overlay.md) and cadence diagnostics completed in
`PERF-10-PRESENTATION-CADENCE` already exist. `clock` pacing is already the
default on Linux/Windows; it must not be proposed as a missing feature.

**Implemented solution.** The Video > FPS counter row explains simulation and
presentations and states that rates can differ in Fast. The reported case does
not require pacing or GBA clock fixes.

- [x] Clarify the case: the user confirms “Only with Fast active”. Existing 4×
  audio-test captures and counters show the expected 4:1 ratio.
- [x] Reuse completed counter tests and PERF-10/audio checks: multiple frames
  per presentation, pause reset and skipped presentations; no new cadence
  matrix is needed for this report.
- [x] Add an explanation accessible from Video > FPS counter:
  “INT: simulation; EXT: presentations; they can differ in Fast”.
- [x] Close diagnosis of the expected case with explanation and captures. No
  divergence at 1× was reported and equality is not forced.
- [x] Reuse the independent Fast/pause reset/unpresented-frame test. VSync/
  refresh comparisons on device and native Windows remain general PERF-10
  hardware coverage, not a fix required by this case.

**Closure criterion:** every reproduced divergence has a measurable
explanation; pathological divergences are corrected without slowing simulation
or changing counter definitions. Numerical equality is not required.

<a id="bug-03-fast-audio--audio-fuori-sincrono-in-modalità-fast"></a>

## BUG-03-FAST-AUDIO — Out-of-sync audio in Fast mode

**Status:** reported case clarified (Fast audio, 4×); transition reset,
consumption based on actual production and Normal backlog limit implemented.
Listening and hardware/event coverage remain open.
**Priority:** P1. **Complexity:** medium-high. Related to PERF-09/10.

**Evidence and analysis.** `AvConfig` and the Fast forward menu already expose
`silent`, `fast` and `normal`. `set_fast_forward` already emptied queues when
changing mode, but left filter memories and the last callback sample active. The
synthetic regression confirms residual output with empty queues. The new host
epoch also invalidates those states; pause and device loss additionally discard
Normal backlog. See [behavior and evidence](../fast_forward_audio.md).
`fast` keeps the selected multiplier as the requested limit. The callback in
[main.rs](../../crates/recomp/src/main.rs) now adapts consumption to production
measured by the device clock and the queued reserve; the frontend continues to
run the frame group independently of audio.

`normal` instead uses [NormalAudio](../../crates/recomp/src/ff_audio.rs): it
plays PCM produced by the accelerated game at 1×. The old temporary file had
no duration or size limit; a preallocated queue now limits PCM to 2 seconds/
512 KiB and jumps to recent audio with a fade when full. This differs from the
`fast` defect: if the PC cannot sustain the requested multiplier, nominal
consumption and actual production can diverge. The user's case concerns Fast
audio at 4×, not Normal. In the local Linux/CPAL/llvmpipe test, Faithful and
Enhanced reach only about 122–128 INT: all 497/499 Fast callbacks have holds,
with zero producer drops and about 47% of PSG/HLE requests empty. Nominal
consumption therefore exceeds production; at 1× playing intervals have no holds.
Hold/release, toggle and pause/resume were performed with captures inspected.
Zero volume and an inactive scene do not prove sound quality or the latency of
an event on the user's device. Reset alone did not close this second cause, so
the consumption correction below was added.

**Implemented Fast solution.** `fast_audio_rate::Clock` estimates production
over about 200 ms, corrects the reserve with a 250 ms response and adjusts
consumption common to five paths over about 100 ms. Target reserve 60 ms beyond
the callback buffer, range 0.05×..multiplier+2%; the old control remains at 1×.
The sinc filter uses continuous fractional cutoffs; priming/refills emit explicit
silence, counted by `FFTA_FAST_AUDIO`, without repeating old samples. No new
lock, I/O, allocation or host clock in the callback.

**Implemented Normal solution.** Menu “delayed 1×”, 2-second queue, jump to the
last 250 ms with a 10 ms stereo fade and fixed transition tail. No temporary file
or callback I/O. The budget concerns host PCM, not backend-added latency. A
future accelerated mode preserving pitch would require dedicated time-stretch:
that is a further choice, not a property of the 1× queue.

- [x] Identify the reported mode and speed: the user confirms **Fast audio at
  4×**, suspecting playback of delayed audio.
- [ ] Reproduce at 2×/4×/8× with Faithful/Enhanced, hold/toggle transitions and
  a recognizable audiovisual event. Measure initial latency and its evolution.
- [ ] Reuse existing audio/cadence profiles: correlate guest frames, consumption,
  fill, holds/drops and callback time; distinguish priming, pause and Fast. Do
  not use a benchmark without a callback as sound evidence.
- [x] **BUG-03-EFFECTIVE-RATE:** fix nominal consumption when requested speed
  is not reached: smoothed common clock, reserve, fractional cutoff and
  observable priming. 4× callback/2× production regression, 72-case matrix at
  2×/4×/8×, production changes/stopping/resumption and filtering. No change to
  guest time or serialization.
- [x] Verify 4× with real Linux callbacks and guest window 9400:11400:
  Faithful 357/Enhanced 359 callbacks, zero holds/PSG-HLE empty/drops/refills and
  zero priming silence. Mean consumption 2.2032/2.1906× follows production
  2.2030/2.1906×; maximum estimated queue 107.3/108.0 ms. Zero volume and an
  inactive scene: not a sound-latency measurement on native Windows.
- [x] **BUG-03-NORMAL-BOUNDS:** 2 seconds/512 KiB budget, jump to the last
  250 ms and 10 ms stereo fade. Explicit menu, 1× Faithful consumption even
  with Enhanced configured, unchanged resets, no callback file/I/O.
  Native 78 runtime/33 menu and Wine 79 runtime/33 menu suites, warning-free
  build; real Linux callbacks 365/376, maxima 130287/130783 pairs below 131072,
  10 jumps per mode without holds/drop/over-period. Persistence, pause, return
  to 1×, unchanged slots and exit verified; zero volume.
- [x] **BUG-03-TRANSITION-RESET:** also invalidate DAC, interpolators, filters
  and crossfade when entering/leaving Fast and changing multiplier/mode; empty
  audio on pause and device loss. Load/Undo retains stream recreation.
  Regressions on residual output and new samples compared with a freshly
  initialized callback, native and Windows/Wine.
- [ ] Complete listening and device measurements: hold/toggle, pause, load/Undo
  and physical reconnection; host reset cannot retract samples already in the
  backend.
- [x] Synthetic consumption, backlog limit, fade and transition tests:
  6000 ticks at 2×/4×/8× without capacity growth and identical observed/
  unobserved callback, with audible synthetic output and 1× consumption without holds.
- [ ] Verify on Windows with real audio. Before comparing, set a latency
  threshold consistent with the device buffer and check that it does not grow
  during a long session; verify memory and file stability.

**Closure criterion:** `fast` does not accumulate drift relative to video in the
reproduced case, and return to 1× does not drag delayed audio. `normal` has
declared, bounded delay; `silent` remains silent. The report alone does not
justify changing guest mix, serialization or timers.

<a id="feat-01-mute--comando-mute-nel-menu-audio"></a>

## FEAT-01-MUTE — Mute control in the Audio menu

**Status:** implemented and verified within the limits below. **Priority:** P2.

**Implemented solution.** Audio > Mute On/Off saves `audio.mute`, defaulting to
Off for older configurations. Volume remains independent and can change while
Mute is active. The callback applies the union of user mute and temporary Fast
Silent/fallback mute to final output while continuing consumption; the existing
ramp closes the tail to zero below -100 dB. Audio configuration is shared
between startup and live changes: saved volume also applies immediately.
Audio-only edits do not recreate video history. No change to guest input, timers,
save format or pacing. See [audio controls](../audio_controls.md).

- [x] Default, parsing, serialization and compatibility of `audio.mute` in
  AvConfig, with installed/portable paths unchanged.
- [x] Mute row, value and shared keyboard/gamepad/mouse menu handling; volume
  changes during Mute and restoration to the chosen level.
- [x] Separate user and temporary state; apply a ramp without stopping
  consumption/stream/pacing. Unmute does not cancel Fast Silent/fallback;
  pause/Fast/load resets preserve the user's choice.
- [x] Faithful/Enhanced/Fast/Normal callback regression: queues and consumption
  equal the reference, ramp verified, exact silence at steady state, audible
  synthetic output on return and changed volume preserved. Native 27 config/
  34 menu/79 runtime and Wine 27/34/80 pass, 4 optional runtime tests ignored
  per target.
- [ ] Listening on native Windows and a physical controller remain hardware
  coverage; shared menu tests and Wine do not constitute that validation.

**Functional closure criterion satisfied:** mute is accessible and persistent,
volume retained, silence at steady state and consumption unchanged. Volume 0
remains distinct.

<a id="feat-02-fullscreen--fullscreen-onoff-nel-menu-video"></a>

## FEAT-02-FULLSCREEN — Fullscreen On/Off in the Video menu

**Status:** implemented and verified on Linux and Windows/Wine; physical
coverage separate. **Priority:** P2. **Complexity:** low-medium.

**Implemented solution.** Video > Fullscreen (F11) and F11 share the borderless
setter and `video.fullscreen`, default Off and applied at next startup. The
controller reconciles state reported by winit, waiting up to 2 seconds for
asynchronous requests, without reading the clock when no request is pending.
Winit/WM retain geometry and maximization; a fallback repositions the window if
the old area no longer intersects an available monitor.

The real test also found and fixed a menu resize race: GPU upload and software
reading must use the size of the canvas already drawn, not a window resized in
the meantime. See [behavior, evidence and limits](../fullscreen.md).

- [x] Default/old-file/roundtrip configuration, Video row, navigation, mouse
  and scrolling to Back after the increase to 11 items.
- [x] Single startup/F11/menu request; auto-repeat ignored, preference and row
  synchronized with winit state. Delayed response/refusal/cancellation tested;
  external changes not exposed by the backend remain an API limitation.
- [x] Restore 640x360 window with unchanged coordinates and normal/maximized
  geometry verified on X11/Openbox and Windows/Wine. Pure test for removed
  monitor, negative coordinates and window limits; physical hotplug open.
- [x] Reuse resize/DPI/cursor/presenter without recreating guest machine, video
  history or audio, or changing aspect/zoom/speed. Fix the canvas/window race
  found by the test, preserving the negative test.
- [x] Menu and F11 while paused, a single long press, fullscreen restart,
  VSync On, fullscreen minimize/restore and return to windowed mode. Native
  28 config/35 menu/81 runtime and Wine 28/35/82 suites, 4 optional runtime
  tests ignored per target; warning-free build, unchanged slots and normal
  exits. PERF-04 matrix reused.
- [ ] Native Windows, physical controller, Wayland/macOS, hotplug/DPI/refresh
  and physical Alt-Tab remain hardware coverage; Wine uses the interpreter frontend.

**Functional criterion satisfied:** menu, F11 and window-reported state agree;
entry/exit and preference restoration work without restarting the game or
losing session geometry. No package updated.

<a id="feat-03-tactical-overlay--riepilogo-battaglia-e-previsione-turni"></a>

## FEAT-03-TACTICAL-OVERLAY — Battle summary and turn prediction

**Status:** design pending; data sources still need verification.
**Priority:** P3. **Complexity:** high, especially prediction.

**Preserved request.** Square shows laws, number of turns, mission and turn
order; holding it pins the overlay. Before confirming an action the pinned
overlay shows any changes to the order.

**Evidence and analysis.** The project has host bindings, UI layers and EUR/USA
ROM profiles, but the inspected paths do not reveal a verified reader exposing
these data together or an order predictor. The laws screen can help compare
values, but pixels are not a structured source. The physical token to assess
for Square is `West` (X position on Xbox); the current bindings UI uses Xbox
labels. A keyboard binding is also needed and conflicts with saved remappings
must be handled.

**Proposed solution, first phase: data and advisory overlay.** Introduce a typed,
read-only battle-state reader enabled for verified ROM profiles. Expose a
consistent snapshot to the frontend with data validity and provenance; render
it on a host layer independent of zoom. Do not infer EUR/USA addresses by
analogy alone.

- [ ] Identify and verify active laws, mission/objective, units, turn counter
  and scheduler data; define what counts as a “turn” relative to individual
  activations. Compare with real screens and sequences; show “unavailable” for
  unverified data.
- [ ] Design the reader near [ffta_rom.rs](../../crates/gba-core/src/ffta_rom.rs)
  and battle modules, with bounds, identity and phase checks; update at frame/
  event end without writing guest RAM or changing RNG, input or clock.
- [ ] Add a remappable host action in Input, with Square/`West` as the initial
  proposal; resolve conflicts without overwriting GBA bindings. Verify XInput,
  DirectInput and Gilrs and make the label understandable for a PlayStation controller.
- [ ] Implement a press state machine. Proposal to validate: visible on press,
  hidden on short release; holding at least 400 ms pins it after release; a new
  short press closes it. Use wall time so Fast does not change the threshold;
  ignore auto-repeat.
- [ ] Draw laws, mission, turn and current order in a readable panel that does
  not cover confirmations and essential information. Reuse
  [ui_layer](../../crates/screen/src/ui_layer.rs) and software composition.
  Handle resize, DPI, fullscreen and long text; show the “pinned” state.
- [ ] Hide during host menus and outside battle; on focus loss, pad disconnect,
  load/Undo, ROM change or battle end reset presses and stale data. Recalculate
  only from a valid scene before reappearing.

**Proposed solution, second phase: prediction before confirmation.** First
verify how the game determines order and which actions change it. Prefer
prediction data already prepared by the guest, if available; otherwise extract
a pure function on a snapshot. Do not run speculative actions on the active
machine or consume RNG to build the preview.

- [ ] Identify action/target selection, confirmation phase and actual scheduler
  rules, including tie-breaks and status effects; delimit actions for which a
  deterministic prediction is possible.
- [ ] Calculate current and predicted order from immutable data; highlight only
  movements when the overlay is pinned. For random outcomes or insufficient
  data show conditional/unavailable prediction, without presenting an assumed
  order as certain.
- [ ] Update preview on action/target change and remove it on cancellation;
  after confirmation replace it with the new authoritative state.
- [ ] Compare prediction and observed order for actions that leave it unchanged
  and that change it, removed units and relevant states; add regressions after
  verifying rules, not before.
- [ ] Compare replay with overlay on/off: same guest state, RNG, native frames
  and deterministic audio for the same GBA input; measure reading/rendering
  cost and verify that the press does not also confirm a game action.

**Closure criterion:** first phase usable and verified separately; the complete
feature closes only with a reliable preview in supported cases, visible limits
and no overlay effect on guest execution. The BUG-01 fix helps compare laws,
but does not replace verification of data structures and the scheduler.

<a id="feat-04-rom-data--estrazione-e-uso-diretto-dei-dati-poi-mod"></a>

## FEAT-04-ROM-DATA — Extract and directly use data, then mods

**Status:** TODO written; implementation not started. **Complexity:** high.
The internal priority is to extract and decode data from the selected ROM,
prepare it on first startup and have the runtime actually use it. Mod support
and logic changes follow this main deliverable.

The [dedicated TODO](ROM_DATA_TODO.md) contains dependencies and criteria for:

- [ ] ROMDATA-01 — inventory of formats, resource identities and contracts.
- [ ] ROMDATA-02 — verified extraction and decoding by family.
- [ ] ROMDATA-03 — automatic preparation, reuse and dataset restoration.
- [ ] ROMDATA-04 — direct consumption in the runtime, with provenance evidence.
- [ ] ROMDATA-05 — equivalence, saves and performance measurements.
- [ ] ROMDATA-06 — mods as resource overrides, with conflicts and compatibility.
- [ ] ROMDATA-07 — content extensions and logic changes.

**Main criterion:** dataset actually used by the game for the intended
families, with automatic extraction, equivalence without mods and explicit
remaining ROM dependencies. The ROM is already read into RAM: a performance
advantage must be measured. Exporting files or recomposing a ROM alone is not enough.

<a id="consegna-di-ciascuna-attività-futura"></a>

## Delivery of each future task

- [ ] Record the selected ID, what is already complete and the missing criterion
  in the handoff; close each subphase without reopening it on the next resume.
- [ ] Run checks targeted to the change; run core suites serially when they
  share globals. Record native Windows, Wine and Linux separately.
- [ ] Update relevant documentation and record in `docs/changes/`, regenerate
  `CHANGELOG.md` and run `python3 scripts/project_memory.py check`. Keep ROMs,
  saves and recordings out of commits and packages.

<a id="feat-restart-rom--riavvio-rapido"></a>

## FEAT-RESTART-ROM — Quick restart

- [x] Added **Restart ROM** to the main menu: single confirmation, same window
  and translation, current cartridge save synchronized before reset, slots and
  backups retained. Errors in the menu; refused during recording.
- [x] Reset and failed publication tests, menu navigation/drawing, Linux/Windows
  builds and real Linux/Wine checks from battle to the initial screen.

See [ROM restart](../restart_rom.md). Native Windows remains to be verified.
