<a id="sessioni-di-debug-isolate--dbg-02"></a>

# Isolated debug sessions — DBG-02

A session is a portable copy of the selected case. It contains an executable,
ROM, optional BIOS and dataset, configuration, save and starting snapshot.
The copy works in its own folder: periodic saves, exit, slots, Undo, backups,
settings and cache do not use the original game's paths.

To start from a collection scene, use the [DS catalog](debug_catalog.md):
**Debug → Debug catalog → Open isolated case** or `debug-catalog open`.
The copy retains the name, provenance and selected entry; snapshots can also
be imported into a free slot in the copy without loading them automatically.
To record a reproduction from the current point, use
[Start/Stop input recording](debug_recordings.md); `debug-record start` starts
the same recording from the CLI in a closed copy with a starting snapshot.
A complete recording can become a [replay profile](debug_replays.md): menu and
CLI run the case in a separate worker without writing progress to the source
session or normal game.

<a id="dal-gioco"></a>

## From the game

Pause and open **Debug → Start debug session**. The runtime captures the
current point and save memory, copies the current configuration and opens a
second window. The normal game stays paused. The new window is titled
**FFTARecomp - DEBUG: name**; its menu also shows a **DEBUG COPY** indicator in
the margin (abbreviated in narrow windows), visible in fullscreen.

The copy can load snapshots, save, undo and restart with the existing features.
**Debug → Close debug session** closes the copy; return to the original window
and choose Resume to continue the normal game. Debug progress is not promoted
automatically to the personal game. Start debug session and Switch ROM are
blocked in the copy: close it before changing cases. Creation is refused
during input recording.

When a valid user catalog is present at `cheats/catalogo.json` beside the
source runtime, session creation copies it into the private session at the
same relative path. The catalog is pinned in the session manifest and the
Debug / Cheat catalog page can discover it without another import. A malformed
or changing catalog prevents creation with an error; an absent catalog leaves
the usual empty state. Existing sessions are not silently modified. Cheat
activation retains its interpreter and preview requirements described in
[the cheat guide](cheats.md).

The menu-created copy retains the current game's backend. When the game is
using the native backend, create an interpreter copy with `debug-session
create ... --interp --open` to activate supported cheats; opening the catalog
alone does not switch the backend.

Targeted Linux and Windows Wine tests cover the copy, pin, relocation and
tamper checks. Separate isolated Linux/X11 and Windows Wine GUI sessions opened
the inherited catalog and exercised a supported cheat through the shared
control channel. Native Windows and Wayland remain untested for this change.

Menu-created sessions are under `Data/debug-sessions/` in portable mode, or
`debug-sessions/` under the installed configuration directory. The log shows
the full folder path. After eight folders, the menu asks you to reopen a
session or move those to retain: it does not delete evidence or failed attempts.

<a id="dalla-cli"></a>

## From the CLI

PowerShell example:

```powershell
.\recomp.exe debug-session create 'C:\Games\FFTA EUR.gba' `
  --dir 'C:\Debug\DS-20' --sav 'C:\Saves\game.sav' `
  --state 'C:\Debug\DS-20.ffstate' --no-bios --interp --open
```

`--open` is optional. Without this flag, the folder is created and a JSON
result printed without starting the game. The destination must be new.
The source save and snapshot are read and copied, never moved.
If `--sav` is omitted, the `diagnose` check's selection rules apply;
an ambiguous choice requires an explicit path.

To reopen or inspect a folder:

```powershell
.\recomp.exe debug-session inspect 'C:\Debug\DS-20'
.\recomp.exe debug-session open 'C:\Debug\DS-20'
```

`inspect` returns the manifest and verifies files without running the game.
`open` waits for the copy to close and propagates the process outcome. You can
also open `recomp.exe` directly in the folder. Linux uses the same syntax with
`./recomp`; the copied binary retains execute permission. This unit's documented
validation covers Windows under Wine.

`--bios FILE` copies a real BIOS; `--no-bios` selects HLE. `--dataset DIR` copies
an already authenticated dataset; `--rom-data` selects the current managed one
without extracting or repairing it. These are alternative options. Without a
dataset the runtime uses the ROM normally. The menu copies the dataset actually attached.

`--interp` selects the interpreter. A native copy needs the source runtime's
TinyCC: the library is copied into the session and translations are created
in its cache. Creation requires the development runtime; full-recomp packages
with a distribution manifest are not converted by this tool.

<a id="riapertura-e-spostamento"></a>

## Reopening and moving

The starting snapshot remains in `seed.ffstate` and is also copied to slot 1.
Each opening of a session with a snapshot restarts there; other slots and save
changes stay in the copy and can be loaded from the menu. Restart ROM uses the
normal reboot with current cartridge memory, without implicitly loading the
seed or rotating backups. Closing and reopening the process instead creates
the normal startup backup of the copy's save.

Close the session before moving **the whole folder**. Manifest and inputs use
relative names; `portable.txt` locates the directory from the actual executable,
even if the command starts from another working directory. Do not change files
or links while the session is open.

The `debug-session.json` manifest identifies the case and retains hashes for
ROM, BIOS, executable, seed, dataset, optional cheat catalog and TinyCC. Saves, configuration and cache
are mutable. `session.lock` prevents two simultaneous openings of the same copy;
`last-result.json` retains the latest startup/exit outcome. The
[diagnosis](debug_diagnostics.md) report is in the copy's configuration directory.

Launches through `open` or the menu remove inherited `FFTA_*`, `RECOMP_*` and
`GBA_*` variables: overrides cannot reconnect personal saves, recordings or
dumps. Direct launch with those variables present is refused. `play` overrides
in the copy are refused: use `debug-session create` to configure a new case.

The new folder immediately receives a `.gitignore` excluding all its contents.
ROM, BIOS, dataset, snapshots and saves remain private and must not be included
in packages. The initial copy is bounded to 768 MiB; inspection rejects trees
over 2 GiB, 4096 entries or 16 levels, and linked files. A copy left incomplete
by an I/O error remains for diagnosis without a valid manifest: it is neither
launched nor removed automatically.

<a id="verifica-del-27-ottobre-2026"></a>

## Validation on 27 October 2026

Offline Windows release build with locked dependencies from a stable public
source copy plus this tool's changes. Ongoing extraction and production
executables stay separate from build outputs.

Under Wine, six session, 41 menu, ten slot and seven backup tests passed; the
existing backup test requiring a specific dataset remains ignored. The final
41 menu tests also passed after the indicator's visual correction. Seven real
EUR CLI cases verify creation, inspection, TinyCC copying and rejection of
incompatible destinations/inputs/overrides.

Wine/Xvfb checks verify starting from the snapshot, copied dataset, exclusion
of external overrides, exclusive opening, Save/Load/Undo, restart, backup
restoration and closing. The moved folder reopens from an unrelated working
directory, preserving the snapshot and two backups. The menu-created copy
closes normally; on returning to the original window, frame, clock and progress
file are unchanged. All 73 protected source files retain hashes and timestamps,
as do both Windows/Linux production executables.

X11 detection and waiting for reports required corrections in the harness
alone. When returning to the original window, XSetInputFocus alone showed a
black image: a Windows probe verified that the window was visible, not
minimized and responsive; ShowWindow/SetForegroundWindow, like taskbar
activation, completes the return correctly. Final screenshots were inspected;
the indicator no longer covers slot titles.

Native Windows, Linux/Wayland, physical controllers and native-translation
gameplay in the copy remain unverified. The native check covers copying and
identity of the TinyCC library. No benchmarks, new DS scenes or old ROM
acceptance tests were run. Local evidence is in `target/debug-tools-sessions/`;
public sources and reusable cache are in `target/debug-tools-build/`.

To pause the copy, advance an exact number of frames and resume, use
[DBG-10: menu, F6/F7/F8 and debug-control CLI](debug_control.md).
`debug-session open DIR --paused` starts from the seed without executing frames
before the first request; normal `open DIR` retains direct startup.

<a id="coordinamento-con-la-pulizia--dbg-18"></a>

## Cleanup coordination — DBG-18

New copies declare the directory-v1 host protocol and retain a lease during
session creation and use. An active or interrupted context remains protected;
an old copy without the protocol is not considered safe for automatic cleanup.
The normal session lock remains in place.
[Manager, receipts and lease recovery](debug_artifacts.md).
