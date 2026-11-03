# U31-07: Automatic machine-state saving

Status: **source audit; ADAPT for a future fork-native design**. This report
is not an implementation specification or proof of runtime behavior. The
upstream target is `6bdddbc559ac459c4a18325c15c28d040d87f407`, the fork is
`decomp` HEAD `83f245232c7dbd0206d4fea2a556e031e98fbf37`, and the common
ancestor is `fcc38cc46084793f29136419a8a15e6914a570ce`. Source was inspected on
2026-11-02; no tests, builds, ROMs or game sessions were run. The `3.1` tag
points to the ancestor, not the target; see [U31-02](U31-02-scrollbars.md).

## Upstream behavior and callers

Desktop [`autosave.rs`](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/recomp/src/autosave.rs)
combines automatic states with the game's save-card UI, legacy cartridge-save
migration, manual state cards, previews and deletion. The desktop frontend
constructs `AutoSave::new_at` in `crates/recomp/src/play.rs`, calls `eligible`
and `tick` after frames, and uses `load_auto_state` to restore. An automatic
capture is attempted after five seconds of continuous eligibility, when not
paused and not skipping rendering; ineligible frames reset the timer. The
eligibility predicate recognizes live field/world-map conditions and excludes
file selectors, forced blank and roster contexts. It is source-specific and is
not an independently validated universal safe-point definition.

`tick` serializes synchronously, then uses a bounded four-item channel and
`try_send` to pass bytes and metadata to a writer thread. It ignores a failed
`try_send` for an automatic write and reports automatic writer failures to the
log; only manual slot completion uses the result channel. The writer packs the
state and image into `QuickSave`, writes a fixed `.tmp`, flushes that file,
copies the existing destination to `.bak` when present, and renames the temp
file. The reader tries the destination and then `.bak`. `Drop` joins the
worker. These source facts do not establish durability under power loss,
concurrent instances, disk exhaustion or every platform's filesystem semantics.
The default path has a shared `Saves/QuickSave` name; ROM and BIOS hashes are
validated by the inner state on load, not included in that filename. A proposed
fork design must separately decide naming and collision behavior.

The Android tree has its own `gba-android/src/save_states.rs` and an
`Engine` integration in `engine.rs` that initializes storage, updates saves and
routes its save UI. It is a separate implementation surface; this audit did not
trace or validate its lifecycle, filesystem or touch behavior.

## Format compatibility

The upstream outer bundle begins with `FFTASV02`, two little-endian `u32`
length/count fields, 64 ASCII bytes of SHA-256 digest of the state-plus-image
body, then the state and up to 38,400 little-endian `u16` image pixels. The
reader also accepts an upstream raw `FFTAST01` state. Its inner state contains
ROM, BIOS and body SHA-256 values and a custom `Codec` serialization of CPU
and hardware; the loader validates these and clears the decode cache.

The fork's `crates/gba-core/src/save_state.rs` instead emits `FFTA-ST\0`, a
version number, ROM/BIOS/payload digests and a `postcard` snapshot. The fork's
`crates/recomp/src/save_state.rs` accepts that raw state or its own `FFTA-SL1`
slot envelope, with a 240×160 RGBA preview and its digest. It does **not**
recognize the upstream bundle or raw magic. Consequently neither direct
loading nor merely stripping the upstream outer bundle establishes
compatibility; the payload layouts and potentially the represented machine
state differ. No conversion is proposed or claimed. A real adapter would need
an explicit versioned mapping and independent equivalence evidence.

## Existing fork protections and risk

Fork `play.rs` periodically writes changed **cartridge backup bytes** to the
active `.sav` every 300 frames outside benchmark mode; this is not a resumable
machine-state autosave. Manual snapshots use per-ROM `SaveStates/<SHA-256>`
slots (9 or 99), overwrite confirmation and a single session Undo. The fork's
slot writer uses a same-directory temporary file and syncs its contents before
persistence. Snapshot load validates version, ROM, BIOS, payload and decoded
state before replacing the machine; it preserves host resources, cancels queued
mouse input and clears the decode cache.

The frontend's `after_state_restore` is a wider timeline boundary: it freezes
active collectors, interrupts probing, resets audio/input and rebuilds visual
temporal state. Manual load and Undo-load guard active input recording;
manual save refuses after a cheat one-shot pending provenance support. An
automatic capture or recovery path must explicitly address those rules, isolated
debug-session paths, replay and native execution, and must not silently bypass
them. The consequences of saving restored cartridge bytes back to the active
`.sav`, switching the active cartridge slot, and recovering after a later manual
save are design questions; existing manual-snapshot behavior is not proof of a
safe automatic policy.

Upstream `archive_legacy` may rename a recognized original `.sav` after its
available legacy slots have corresponding state cards. This behavior and the
upstream card subsystem conflict with the fork's independent cartridge saves,
backups and snapshot model; do not import them as an autosave shortcut.

## Existing test evidence and proposal

Upstream contains synthetic tests named
`five_second_timer_writes_complete_state_and_skips_selectors`,
`bundled_states_validate_picture_checksum_and_recover_previous_file` and
`replacement_keeps_previous_state_backup`. The fork has core round-trip and
wrong-ROM/BIOS/version rejection tests, plus slot Undo and failed-Undo tests.
Their existence and source assertions were inspected; none was executed for
this audit, and none alone proves GUI, interruption, real-game, cross-version or
native Windows acceptance.

A future design should use a dedicated fork-native state location and the
fork's validated snapshot and frontend restore boundary, without consuming or
mutating manual slots, Undo or cartridge backup history. Before implementation,
choose and document activation/default, scheduling and write budget, retention,
recovery UX, collision/locking policy, cartridge-save timeline semantics,
debug/replay/cheat restrictions, error visibility and platform durability.
Five seconds, automatic startup restore and a specific rotation count are not
approved defaults. Android requires its own design and validation.

Acceptance must include synthetic corrupt/truncated/incompatible input and
wrong ROM/BIOS/version refusal without mutation; interrupted writes at each
publication boundary and validated previous-generation recovery; write errors,
full storage and competing instances; recording/replay/debug and cheat guards;
manual slots, Undo, `.sav` and backup preservation; native-cache and frontend
reset; and an isolated Windows recovery check with actual results. Explicitly
separate tests of process termination from power-loss durability. No upstream
format migration should be offered without its own compatibility proof.

The [fork-native autosave proposal](U31-07-autosave-proposal.md) develops a
reviewable candidate lifecycle, recovery protocol and test plan. It is a
proposal, not an approved configuration or an implemented feature. Remaining
work includes its open decisions, Android analysis, implementation and
acceptance, and the other U31 audit families.

## Follow-up: Android lifecycle and cartridge storage

A bounded source comparison on 2026-11-02 examined the pinned upstream Android
`engine.rs`, `save_states.rs`, JNI and Kotlin lifecycle callers against the
current `AndroidPort/` fork. This is static evidence, not an Android build,
device run or verification of a filesystem's crash guarantees.

The fork's `MainActivity` supplies an app-private `ffta.sav` path, or a distinct
`benchmark.sav` path in benchmark mode. JNI verifies the ROM, reads that save
if present, installs an `Engine` under the global mutex and resets keys. The
fork `Engine::save` obtains cartridge bytes from the bus, writes a fixed
`sav.tmp` sibling and renames it over the save; it does not serialize the
machine. In the inspected method there is no explicit file or directory sync,
backup rotation or machine-recovery offer. It returns success without writing
when no save path or cartridge data is present; JNI also returns success when
there is no engine. Therefore a `gbaSave()` success by itself does not prove
that a new cartridge file was written or is durable.

The Kotlin game worker calls `gbaSave()` on its first paused/inactive loop,
every 120 loop iterations, and in `finally`; these calls do not check the
returned status. The explicit switch-ROM action does check for return value 1,
while the import path calls `gbaSave()` without checking it. `ON_PAUSE` marks
the surface inactive and clears input; `ON_RESUME` marks it active. Detaching
sets the worker's running flag to false, interrupts it and joins for at most two
seconds. These are source paths, not a guarantee that the worker finishes or
that a callback runs before Android kills a process. `gbaFrame` and `gbaSave`
use the engine mutex, which serializes their accesses; it does not by itself
make a background publication durable.

In the upstream Android tree, `Engine::save` instead invokes `update_saves`;
that method also runs before each frame batch. It lazily creates `Saves` (or
`benchmark-saves` for `benchmark.sav`), uses the unqualified `QuickSave` path,
observes the session and runs the five-second eligibility timer. It queues a
serialized machine state to a four-item writer channel; a full or disconnected
queue is ignored for automatic writes, and automatic write failures are logged.
The writer syncs a fixed temporary file, copies a prior destination to `.bak`,
and renames the temporary file; a reader tries the main bundle and backup.
`AutoSave::Drop` closes the sender and joins its writer, but this does not prove
completion after process death or that a return from `gbaSave()` means the
queued write reached storage. Manual state cards also use this subsystem, and
successful manual conversion can archive a recognized legacy cartridge file.
Replacing the fork's `save` method with the upstream method would therefore
change the meaning of the existing lifecycle and switch calls, not merely add
a separate recovery file.

Upstream card selection queues a load, processed on a later `update_saves`.
After loading, the Android engine restores UI style/separation and touch enable,
recreates detected MP2K state, invalidates UI motion and packing, cancels mouse
queues, neutralizes keys and resets its PCM/audio buffers. This is a concrete
Android restore boundary, but no fork Android machine-state loader or equivalent
recovery transaction was found in the inspected `gba-core` and `gba-android`
source. The upstream `FFTASV02`/`FFTAST01` formats are distinct from the fork
**desktop** formats described above; this search does not establish any
Android/desktop state compatibility.

The timer uses `Instant` and is reset on ineligible updates, not explicitly by
the Kotlin pause transition. Consequently, if the measured interval advances
past five seconds while updates are suspended and the next update is eligible,
the source permits a capture before the next frame batch. This is an inference
from control flow, not an observed Android lifecycle result; clock behavior,
process suspension and scheduling require device tests. An automatic capture
must not be assumed to happen on every pause: `save()` still applies the
eligibility and timer predicates.

The separate Android design must define background/foreground and process-kill
semantics, capture timing and acknowledgment, queue draining without blocking
lifecycle indefinitely, error delivery, benchmark and ROM-switch isolation,
cartridge chronology and rollback, and complete audio/input/render/native-cache
reset. The shared `ffta.sav` name for accepted ROMs makes identity and region
switching particularly relevant; a ROM hash alone cannot identify a cartridge
timeline. Test pause/resume, detach, forced process death and disk errors on an
isolated device, plus competing or stale writers and ROM switching. Preserve
and hash original cartridge files. Process-kill results must remain distinct
from sudden-power-loss and storage-durability claims. No Android design choice,
implementation, default interval or new recovery behavior is approved here.
