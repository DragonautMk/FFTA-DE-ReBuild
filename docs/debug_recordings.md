<a id="registrare-gli-input-da-uno-snapshot--dbg-04"></a>

# Record input from a snapshot — DBG-04

The new recorder associates a starting snapshot, a relative input sequence
and a final state. Use it inside a [debug copy](debug_sessions.md), including
one opened from the [DS catalog](debug_catalog.md). The personal game is not
a recording destination.

<a id="dal-menu"></a>

## From the menu

Pause the copy and choose **Debug → Start input recording**.
The current point is preserved without advancing the machine. Choose Resume,
reproduce the problem, then return to **Debug → Stop input recording**.
The window title shows `[REC]` while recording. Closing the copy normally
completes the recording; a runtime error leaves it incomplete.

The folder is `Data/config/recordings/recording-…` in the portable copy. The log
shows the full path and the menu confirms the saved folder. At most eight
attempts are retained before asking you to move them: none is deleted
automatically. Each attempt is limited to 360,000 frames, 200,000 events and
8 MiB of input. Write errors, timeline changes or render-dimension changes
stop the attempt and are reported in the menu.

Load and Undo load complete the recording and **refuse that first command**:
repeat the choice after reading the message. Rewinding is not inserted into a
linear sequence. Saving a snapshot and undoing a file write remain possible.
Restarting, switching saves and creating another copy require stopping the
recording first.

<a id="dalla-cli"></a>

## From the CLI

Prepare a copy with a snapshot, for example with `debug-catalog open` or
`debug-session create --state`. Then, with the copy closed:

```powershell
.\recomp.exe debug-record start 'C:\Debug\DS-20'
```

The command opens the copy with the same pinned inputs and starts recording
before the first game frame. Recording starts at the actual point after
frontend initialization, preserved in `start.ffstate`. The menu can stop it;
the supervisor process waits for the copy to close. Game overrides and a
second concurrent opening are not accepted.

To check the behavior of an interpreter recording:

```powershell
.\recomp.exe debug-record verify 'C:\Debug\DS-20\Data\config\recordings\recording-…' `
  --session 'C:\Debug\DS-20'
```

The check uses the ROM/BIOS and dataset pinned in the closed copy. It verifies
hashes, event origin and order, then replays on a temporary machine without
writing saves. The `ffta-recording-verification` JSON, schema 1, compares the
complete final serialized state and clock. Divergence or error exits with
code 1; the replay timeout is 120 seconds. This is not a benchmark.

The binary must match the one used for recording. This bounded check requires
an interpreter recording. [DBG-05 profiles](debug_replays.md) allow choosing
a runtime, starting from the menu, cancelling and distinguishing replay
outcomes. The old `frames/runc/verify --input` commands continue to read
the `gba-input v1` file.

<a id="contenuto-e-semantica"></a>

## Contents and semantics

- `start.ffstate`: complete starting state, bound to ROM and BIOS.
- `input.input`: `gba-input v1`, with relative frames starting at zero, an
  explicit initial mask of already held keys and subsequent changes. Masks
  are active-low: `3ff` releases all keys. Events in the same frame retain
  file order; mouse and keys follow the existing replay semantics.
- `end.ffstate`: final state of a recording completed normally.
- `recording.json`: schema 1, kind `ffta-input-recording`, status recording/
  complete/incomplete, binary/ROM/BIOS/dataset and payload hashes, absolute
  start/end frames and clocks, render dimensions and stop reason.
- `av.cfg`, `input.cfg`: starting configuration, useful for reproducing context.
- When cheats are applied during a supported interpreter recording,
  `cheats-timeline.json` records exact application order, frame, clock,
  identities and plans; `catalogo.json` is the pinned private source catalog.
  The manifest hashes the timeline, which in turn pins the catalog. Verification
  re-decodes the plans before applying them on the isolated machine. Recordings
  can start with observed cheat ancestry: `cheat-ancestry.json` retains that
  bounded private history and its hash is pinned in the manifest. The initial
  state already contains earlier effects, so replay does not apply them again.
  Legacy ancestry can be unknown. See [cheat recording limits and validation](cheats.md).

Payload names are relative. The folder can move with the copy or on its own,
while retaining a compatible session for verification. An interrupted process
does not publish a complete result. An intact checkpoint alone does not prove
the inputs reach the desired outcome: final comparison is required, and a
divergence remains visible.

The historical `RECOMP_RECORD_INPUT` path remains available and separate: it
records absolute frames and an initial SRAM copy without the new manifest.
Stop can close it but does not turn it into a snapshot recording. Its
Load/Undo/restart blocks remain valid. Historical and new recordings cannot overlap.

Snapshots, input, configuration and reports contain private data. They remain
in the Git-ignored copy and are not included in packages.

<a id="verifiche-del-28-ottobre-2026"></a>

## Validation on 28 October 2026

Offline Windows release build from frozen public base `2fbf9f7` with thirteen
overlaid tool files and a separate cache. Final candidate:
`target/debug-tools-recording/recomp.exe`, SHA-256
`fcd271732ca1a898392ef3a589133a1a45268a8fc9fd98048f109d060c641db4`.
Normal executables and concurrent extraction work are preserved.

- **57 Wine tests:** seven recorder, 44 menu and six session tests.
  They cover nonzero origin and an initially held A key, exact replay, event
  order, limits, rewind, failed writes, checksum/ROM/build, reported divergence
  and moving the folder.
- **Five real EUR DS-20 captures** reproduce identical complete serialized
  state and clock: CLI start, menu start/stop, stopping for Undo load and Load,
  and closing while recording. An additional replay after relocation passes.
- **15 CLI test commands**, besides GUI processes: copy creation, endpoint
  comparison, refusals for altered files, incomplete capture, an already open
  session and a session without a snapshot.
- An induced publication failure for `end.ffstate` in the copy alone
  returns code 1 and a failed session result; it does not publish a complete
  recording. Hashes/mtime of the 140 original inputs and the pinned inputs in
  the copies remain unchanged. Start/Stop and Load/Undo refusal screenshots inspected.

The first test build required converting the mouse target from u8 to u16.
After the tests and first three captures, only messages and close-error
propagation were refined; the final build and other two captures verify
those paths. Local evidence in `target/debug-tools-recording/`:
`test-results.json`, `gui-results.json`, `final-gui-results.json`, `cli-results.json`
and `final-cli-results.json`.

Native Windows, Linux/Wayland, physical controllers, new in-game mouse paths
and native recordings are not verified by this unit. No new DS acceptance,
ROM extraction, performance test or Android/Godot change.
