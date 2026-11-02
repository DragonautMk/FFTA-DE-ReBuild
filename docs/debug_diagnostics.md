<a id="diagnosi-della-sessione--dbg-01"></a>

# Session diagnosis — DBG-01

Diagnosis collects the identities and paths needed to reproduce a problem.
The menu and `diagnose` command use the same collector and versioned JSON
(`schema = 1`, `kind = ffta-session-diagnosis`). Collection is explicit:
it does not add logs every frame.

<a id="dal-gioco"></a>

## From the game

Open **Debug → Copy diagnosis**. Extended saves need not be enabled.
The game stays paused: the report is copied to the clipboard and written to
`diagnostics/session.json` under the configuration directory. In portable mode
the file is `Data/config/diagnostics/session.json` beside the executable.
Each request updates the same file without creating a series of folders.

The report contains its folder's full path. If the clipboard is unavailable,
the menu reports this and retains the file; the startup log shows its path.
The JSON may contain personal paths: review it before sharing.

Session diagnosis uses the hashes of the ROM/BIOS bytes actually loaded,
the actual mode, guest frame and clock, and the attached dataset's identity.
It reports whether the original files or on-disk dataset have been replaced.
The snapshot slot is the **menu selection**, not a promise that the current
state still matches the file. An active recording is identified without
claiming it is already ready for replay.

<a id="prima-dellavvio-dalla-cli"></a>

## Before startup, from the CLI

PowerShell example, from the selected executable's directory:

```powershell
.\recomp.exe diagnose 'C:\Giochi\FFTA EUR.gba' --no-bios --interp --json
```

For a recorded case with an explicit dataset:

```powershell
.\recomp.exe diagnose 'C:\Giochi\FFTA EUR.gba' --no-bios --interp `
  --sav 'C:\Debug\partita.sav' --state 'C:\Debug\scena.ffstate' `
  --input 'C:\Debug\scena.input' --dataset 'C:\Debug\dataset' `
  --out 'C:\Debug\diagnosis.json' --json
```

On Linux the command is `./recomp diagnose ...`. `recomp help diagnose` lists
the options. `--bios FILE` selects a real BIOS; without either BIOS option,
the runtime's normal installed-image search applies. `--rom-data` checks the
managed generation selected by `current`; `--dataset DIR` checks an explicit
directory. The check neither prepares nor repairs data: if missing,
the report suggests `recomp rom-data prepare <rom>`.

`--slot N` checks one of the 99 snapshots beside the selected save;
it is an alternative to `--state FILE`. `--sav` and `FFTA_SAVE_PATH` avoid
ambiguous save selection. Resolution follows the normal rules but does not
create folders or open a picker: multiple candidates require an explicit choice.
An explicit `--sav` path must exist; without an explicit selection, a new
game is valid and the report identifies the missing file.

Without `--out` the CLI writes only to stdout. Without `--json` it displays a
brief summary with causes and remedies. Invalid inputs produce `ok = false`,
structured issues and exit code **1**; a valid diagnosis returns **0**.
Syntax errors or report-write failures are reported on stderr.

<a id="contenuto-e-limiti"></a>

## Contents and limits

- Build: version, revision and working-tree modification status at build time,
  target/profile, available features, path and SHA-256 of the actual binary.
  An unknown revision remains explicit in builds without Git metadata.
- Files: hashes and sizes of ROM, BIOS, SRAM, snapshot, input and configuration;
  EUR/USA profile only for exactly recognized images.
- Dataset: directory and manifest hash, extractor/decoder revision, resources
  and identity authenticated against the ROM through the runtime validator.
- Snapshot: full CLI decoding with the normal validator on a temporary
  machine; no guest instruction is executed.
- Input: format/value parsing; it does not establish scene reachability,
  timing compatibility with the snapshot or endpoint equality.
- Native mode: before startup this is a request, not an already loaded library
  or successful compilation. The report states this; from the menu it identifies
  the mode actually active.
- `--require-rom-audit` fails when the compiled feature
  `rom-data-read-audit` is absent. It does not start a capture: the audit interface is DBG-06.

The report is bounded to 512 KiB and published atomically; it replaces only a
previous report of the same kind/schema, never an arbitrary file or link.
No ROM, BIOS, RAM, SRAM, screenshot or asset content is embedded in the JSON.
The check does not start native translation, replay or benchmarks, or save the
game. [Isolated sessions](debug_sessions.md) add portable copies of the selected
case; replay profiles remain a later item in the
[checklist](development/DEBUG_TOOLS_TODO.md).

<a id="verifica-del-27-ottobre-2026"></a>

## Validation on 27 October 2026

Windows build with locked dependencies and no network; 47 tests passed under
Wine (5 diagnosis, 40 menu, 2 save selection). Eight real EUR CLI cases verify
explicit/managed dataset identity, an incompatible old generation, malformed
input, a different snapshot BIOS, missing audit feature, protection of a save
used as the report destination, and an out-of-range slot.

A Wine/Xvfb session with portable configuration and a copied save verifies
the menu entry, clipboard JSON identical to the file, unchanged frame/clock
between two collections, updating a single report and normal close. The menu
screenshot was also inspected. The 139 selected protected input files, including
ROM, snapshot, recording, save and dataset, retain original hashes and timestamps;
both Windows/Linux production executables are also preserved.

The first GUI launch with a prefix already started without a display stopped
before creating the window; the check passed with a dedicated graphical Wine
prefix. Native Windows, Linux, Wayland, physical controllers and a session with
active native translation were not verified in this unit. The report identifies
SRAM without validating the game's internal checksums. The checks do not include
benchmarks, new DS scenes or replaying old ROM acceptance tests.
Local evidence is in `target/debug-tools-diagnosis/`.
