<a id="riprodurre-un-caso-con-un-profilo--dbg-05"></a>

# Replay a case with a profile — DBG-05

A portable profile retains the case, runtime, ROM/BIOS, optional dataset,
starting snapshot, input, expected endpoint and timeout. Menu and CLI start
the same isolated worker; it has no writer for SRAM, slots or backups.
The normal game stays paused and resumes only by explicit choice.

A profile can include a [ROM-read audit](debug_rom_audit.md):
`debug-rom-audit create` configures the preset, window, ranges and cap. Run,
Cancel and replay outcomes remain shared. The audit has its own truncation
and partial-window indicators, separate from endpoint comparison.

<a id="preparare-il-profilo"></a>

## Prepare the profile

Start with a [complete recording](debug_recordings.md) and its closed
[debug copy](debug_sessions.md):

```powershell
.\recomp.exe debug-replay create `
  --recording 'C:\Debug\DS-20\Data\config\recordings\recording-…' `
  --session 'C:\Debug\DS-20' --dir 'C:\Debug\replay-DS-20' `
  --case 'DS-20-EUR' --timeout-ms 120000
```

The destination must be new. The command authenticates the recording, checks
origin/endpoint and ROM/BIOS/dataset, then copies only the required inputs into
a new session. `replay.json` is published last: an incomplete copy is not an
executable profile. The DS collection and source recording remain unchanged.
The case name is a description, not new DS evidence.

The selected runtime is the executable running create; `--runtime FILE`
selects another. The profile separately records hashes of the selected runtime
and the binary that recorded the case. A new build can therefore be compared
against an old endpoint without renaming the build or promoting a divergent
outcome. The selected runtime must support `debug-replay worker`; old
executables without the command exit with an explicit error and retained log.

The default path uses the interpreter. `--library FILE` selects an already
available compatible native translation: the library and TinyCC are copied
and authenticated. The ROM is not compiled, nor is an external cache consulted
to replace the selected library. A library with incompatible ABI/ROM/BIOS may
fail or diverge; the separate worker retains its outcome. Compatibility is
not inferred from the `.dll` name.

Recordings with an exact cheat timeline or cheat-derived starting ancestry
currently require the plain interpreter profile: native libraries, audits,
trigger captures and comparisons are refused. Starting ancestry is pinned and
reported, while its already-applied effects come from the starting snapshot.
The private catalog and timeline are copied into the profile and checked against
the ROM and source-bound plans before replay. A replay started from the menu is
refused while any separate maintained cheat set is armed. The result identifies
recorded variants and links the complete private application timeline. See
[cheat recording limits and validation](cheats.md).

The dataset is inherited from the source session and must match the recording.
All payload paths are relative; moving the whole profile folder preserves
the case. Personal files remain private, Git-ignored and outside packages.

<a id="dalla-cli"></a>

## From the CLI

```powershell
.\recomp.exe debug-replay run 'C:\Debug\replay-DS-20\replay.json' --json
```

Output is schema-1 JSON Lines, kind `ffta-debug-replay-result`: rows with
`outcome = running`, completed/total frames and message, then a terminal outcome.
Each launch has a distinct identifier. The result identifies case, runtime,
ROM, dataset, input and library; at the endpoint it adds state hash and clock.
`--json` is optional: output semantics remain the same.

The stop condition is `exact-endpoint`: replay exactly the recorded frames
and compare the complete expected serialized state and clock. A similar image
is insufficient. Guest-frame origin remains intact; input is relative to the
snapshot, with explicit initial keys. Mouse semantics follow existing runners.

Exit codes:

- **0, passed:** complete endpoint and clock identical.
- **2, diverged:** endpoint or clock differs.
- **3, incompatible:** profile/input/build missing, changed or incompatible.
- **4, timeout:** time limit reached.
- **5, interrupted:** cancellation or replay-owner shutdown.
- **6, error:** worker error or incomplete frame.

Timeout ranges from 1 to 120,000 ms; the maximum is 360,000 frames. The
supervisor also bounds worker startup/preflight. Two runs of the same profile
are refused through exclusive locks; distinct profiles have separate copies
and results.

To cancel from a second terminal:

```powershell
.\recomp.exe debug-replay cancel 'C:\Debug\replay-DS-20\replay.json'
```

The worker checks the request between frames; if it does not respond within
two seconds, the supervisor terminates it. Ctrl+C requests the same stop when
the terminal delivers the signal. An already completed result is not cancelled
retroactively.

<a id="dal-gioco"></a>

## From the game

Open **Debug → Replay case** and select `replay.json`. The case starts in a
separate process. The menu shows progress and result; **Cancel replay** requests
a stop. **Choose replay profile** changes selection without starting it. The
profile stays selected for the game process's lifetime, so the next Replay case
needs only one button.

To preselect it at startup:

```powershell
.\recomp.exe play 'C:\Games\FFTA EUR.gba' --no-bios --interp `
  --debug-replay 'C:\Debug\replay-DS-20\replay.json'
```

Resume is suspended during replay; Quit also closes the worker. When finished,
choose Resume to continue the original game. Stop an active recording before
starting a replay. The complete result is also written to the game log.

<a id="output-locale"></a>

## Local output

Within the profile, `Data/config/replay/` contains only the latest progress,
result, cancellation request and `worker.log`, replaced at each launch.
Each JSON contains the attempt ID: an old result does not apply to a new run.
Publication is atomic with bounded retries for brief Windows contention with
the reader. Copies do not accumulate at every run.

The worker loads the snapshot and consumes audio like the player, without
opening audio or a window and without measuring performance. If execution,
host/RTC, build or input produces differences, the outcome remains visible.
This tool does not automatically update DS status, ROM coverage or extraction evidence.

<a id="verifiche-del-28-ottobre-2026"></a>

## Validation on 28 October 2026

Offline Windows release build on frozen public base `2fbf9f7` with fifteen
overlaid tool files and a separate Cargo cache. Final candidate:
`target/debug-tools-replay/recomp.exe`, SHA-256
`29b135eb5f267909b4b05d0bb4e92aca5d952a3783f23acd4a1a3dbc6557047e`.
Normal executables, the source DLL and concurrent extraction work remain unchanged.

- **60 distinct tests under Wine:** 8 replay, 7 shared recording-preflight
  and 45 menu tests. They cover complete-state repeatability, origin and
  initial keys, timeout/cancellation progress and codes, divergence, invalid
  inputs, limits, active session, portable paths and relocation.
- **Real CLI:** the EUR DS-20 case of 102 frames repeats identical state and clock;
  it also passes after profile relocation. Two concurrent launches yield one
  pass and one refusal without interrupting the first. Missing, changed or
  incorrectly ordered files return code 3; valid alternative inputs yield divergence 2;
  timeout and cancellation yield 4 and 5 respectively. The 16 behavior checks
  passed; the native comparison is described separately below.
- **Wine/Xvfb GUI:** Replay case reaches the CLI endpoint, Cancel replay
  interrupts the worker and closing the parent terminates it. Parent
  state/frame/clock and progress are unchanged; 171 selected files from the
  collection, ROM, recording sources, DLL and TinyCC retain hashes/mtime.
  Final result, cancellation and diagnosis screenshots were inspected.

The second initial replay detected Windows contention on the progress JSON.
The fix adds an atomic retry bounded to 32 ms: the Windows test deliberately
holds a reader open without delete sharing, then releases it. That test and
the eight replay tests pass; repeatability/concurrency/GUI are verified on
the corrected build. Initial GUI navigation with short presses did not reach
Cancel before the case ended; the final check uses queued real clicks.
Attempts, logs and profiles are retained locally.

The existing EUR DLL (`ab7aa1808c4c14550c46ec2410f18ef47e301dbcf07a6ed021dbd015cf8f38ca`)
was copied without recompiling it. It completes the 102 frames with the same
clock `6269795841`, but the complete state differs from the interpreter endpoint:
`76a654dc…` versus `d8c4dfc8…`. The runner returns **diverged, code 2**.
The initial test expectation was a pass: that comparison remains failed and
is not turned into parity evidence. The cause and parity of that DLL require
separate investigation; no byte or comparison was corrected to make it pass.

Evidence under `target/debug-tools-replay/`: `test-results.json`,
`publication-test-results.json`, `cli-results.json`, `cli-summary.json`,
`gui-results.json` and retained attempts. Native Windows, Linux/Wayland,
physical controllers, Ctrl+C in real terminals and replay with a nonempty
dataset are not verified by this unit. No benchmark, new DS acceptance,
ROM extraction, package or Android/Godot change.

[DBG-08 capture profiles](debug_captures.md) extend the format with an explicit
condition and `stop: "capture-trigger"`. They share leases, supervision,
cancellation and runtime selection but stop at the safe boundary after the
first match or at the limit. `captured` (0) and `not-reached` (7) are neither
complete endpoint verification nor new DS acceptance. Profiles without a
condition retain the previous `exact-endpoint` behavior.

[DBG-09 comparisons](debug_comparisons.md) extend the profile with `comparison`
and `stop: "first-divergence"`. The worker advances two independent machines
from the same state/input and compares every frame: interpreter/raw reference,
interpreter/native candidate with an optional dataset. `matched` (0) certifies
only the declared prefix; `diverged` (2) publishes fields, images and audio at
the first differing boundary. The recorded endpoint remains a separate check.
Supervision, locks, cancellation and timeout are shared; simultaneous audit
and triggers are refused. Ordinary profiles retain `exact-endpoint`.
