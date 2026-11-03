<a id="strumenti-di-debug-portable--guida-comune"></a>

# Portable debug tools — common guide

The Debug page is initially hidden: in the menu press
**↑ ↑ ↓ ↓ ← ← → →**, or set `debug = true` in `av.cfg` and restart.
The unlock persists; in portable mode the file is `Data/config/av.cfg`.
See [unlocking the menu](menu_clan_atlas.md#sblocco-permanente-di-debug).

Use an isolated copy and an explicitly selected case. These workflows do not
automatically change DS evidence status or start general campaigns. Game data
is added locally after extracting the package.

The [integrated Debug TODO](development/DEBUG_TOOLS_V2_TODO.md) describes
planned extensions: menu choices and scripts without automatic pickers,
event/command consoles and cheats. Implementation begins with the
[operation contract](debug_operations.md): `debug-operations list/show` lists
entries, parameters and current gaps without executing them. The
[content catalog](debug_content.md), `debug-content scan/check`, adds a readonly
index with explicit roots, identities and states.
[Debug > Content selector](debug_selector.md) offers the new internal choice;
typed roles and `Parameters / prepare profiles` offer selection and drafts for
existing handlers. `Debug > Scripts and tools` adds declared adapters for the
homepage, campaign reports and artifact previews, readonly recipes and separate
guest catalogs. Start and cancellation are explicit; the three adapters require
Python 3.11+ selected in the menu, with output and logs in the current config root.
Selector choices and views are saved in the config root and follow portable
relocation, with pins and explanations for missing/changed files. Console and
cheats follow the new TODO.

To create a **private** portable copy with the local debug collection and EUR
slots ready, use the [asset export script](private_debug_assets.md). This export
is separate from distribution packages without private data.

<a id="prerequisiti-e-cartella-spostabile"></a>

## Prerequisites and movable folder

Extract the entire package into a writable folder; spaces/Unicode are allowed.
You need your own FFTA USA/EUR ROM and, when selected, compatible SRAM,
snapshot, BIOS, recording, dataset or library. These files are not distributed.
Keep executables, portable.txt, tcc, Licenses, TinyCC, scripts and docs together.

The runtime handles sessions/recording/replay/reports without Python or Docker.
Offline tools require Python 3.11 available as python3, or Docker Desktop in
Linux mode with an existing image: ffta-debug-tools:local for artifacts/campaigns,
ffta-rom-coverage:local for the homepage. For PowerShell artifact/campaign
launchers, -Docker must precede the command; Python/runtime flags are forwarded
literally, including --out. Launchers do not build or download images. Docker
builds are checkout operations, not prerequisites to run each time.

Windows PowerShell 5.1+, after replacing the example path:

~~~powershell
$Bundle = 'D:\FFTARecomp-portable'
$Dbg = Join-Path $Bundle 'scripts/windows/debug_runtime.ps1'
& $Dbg help
& $Dbg help debug-control
~~~

All relative launcher arguments resolve from Bundle, even when the terminal is
elsewhere. On Linux use bash scripts/linux/debug_runtime.sh COMMAND...; for a
Windows package under Wine add --wine /path/to/wine64. Wine runtime path
arguments use Windows paths, for example Z:/path/to/case. The launcher
does not convert arbitrary names/arguments.

The runtime launcher always prints the log path in Data/config/debug-logs.
Offline launchers print the log in target/build-logs. The player retains
play-error.log for startup errors; replays retain worker.log/progress.json/
result.json in the profile. An error does not authorize deleting its evidence.
In the manifest, source_sha256 identifies the packager input file before document
normalization; for a binary it is the binary hash, not the Rust tree hash.
PACKAGE-MANIFEST.json identifies payloads; it does not certify platform or case validation.

<a id="1-registrare-una-scena"></a>

## 1. Record a scene

Prepare a case with a new destination. Replace the three paths with your own
inputs; --state is optional when starting from boot:

~~~powershell
$Case = 'Data/debug-sessions/case'
& $Dbg debug-session create 'C:/Games/FFTA EUR.gba' --dir $Case --sav 'C:/Debug/progress.sav' --state 'C:/Debug/scene.ffstate' --no-bios --interp
& $Dbg debug-session open $Case --paused
~~~

The second command stays open until the window closes. In the copy choose
**Debug > Start input recording**, then Resume and reproduce the scene;
pause again and choose **Stop input recording**. F6 pauses/resumes, F7 advances
one neutral frame, F8 ten. The menu/controller uses the same commands.

From a second terminal, with Bundle/Dbg/Case set as above:

~~~powershell
& $Dbg debug-control status $Case
& $Dbg debug-control step $Case 1 --keys 3fe
& $Dbg debug-control step $Case 1 --keys 3ff
~~~

Masks are active-low: 3fe=A, 3ff=released. Do not mix rewind/Load with a linear
sequence. Stop shows the exact folder name; retain it explicitly instead of
automatically selecting another test's latest case. Close the copy normally
before preparing the profile:

~~~powershell
$Recording = "$Case/Data/config/recordings/recording-EXACT_NAME"
$Profile = 'Data/debug-profiles/replay'
& $Dbg debug-replay create --recording $Recording --session $Case --dir $Profile --case 'Locale-EUR' --timeout-ms 120000
& $Dbg debug-replay run "$Profile/replay.json" --json
~~~

A passed outcome compares the complete final state and clock with the recorded
endpoint. Diverged/incompatible/timeout/interrupted/error are distinct; do not
convert them into acceptance. A replay creates local results but does not write
the personal game. To cancel: debug-replay cancel PROFILE/replay.json.
[Recordings](debug_recordings.md), [profiles](debug_replays.md).

<a id="2-segnalare-un-bug-con-contesto-riproducibile"></a>

## 2. Report a bug with reproducible context

Use the profile and recording from workflow 1. Report creation is readonly and
does not start another replay:

~~~powershell
& $Dbg debug-report create "$Profile/replay.json" --symptom 'What happened, what was expected, last input and observed point'
~~~

Open the HTML identified by the JSON: it is in Data/config/issues/ inside that
profile. It shows identities, retained result, next check and relative
reproduce.ps1/reproduce.sh launchers. Moving the whole profile retains the
recipe; the HTML alone does not carry the inputs. Review summary.json, paths
and attachments before sharing. Nothing is sent automatically.
Exit 0 means a compatible report was created, not that the bug is fixed; exit 3
may publish a report with incompatible inputs.

In the isolated window, **Copy diagnosis** also saves the diagnosis if the
clipboard is unavailable. Select a profile with **Choose replay profile**, then
return to the menu and choose **Create issue report**. To collect events
preceding the error, in the paused copy:

~~~powershell
& $Dbg debug-control recent-start $Case --events 1024
# Reproduce the problem, then pause with F6 or debug-control pause.
& $Dbg debug-control recent-report $Case
~~~

Errors observed by the frontend freeze/save the window by default. A crash
without a callback cannot export memory; drops, window limits and unavailable
reads remain explicit. Status queries do not fill the log.
[Issue report](debug_issues.md), [recent events](debug_recent.md).

<a id="3-confrontare-due-versioni-del-runtime"></a>

## 3. Compare two runtime versions

Prepare two profiles from the same recording, closed session and inputs,
explicitly selecting A and B. Do not replace a binary inside an already
authenticated profile:

~~~powershell
& $Dbg debug-replay create --recording $Recording --session $Case --dir 'Data/debug-profiles/runtime-A' --case 'Version-A' --runtime 'C:/Build-A/recomp.exe'
& $Dbg debug-replay create --recording $Recording --session $Case --dir 'Data/debug-profiles/runtime-B' --case 'Version-B' --runtime 'C:/Build-B/recomp.exe'
& $Dbg debug-replay run 'Data/debug-profiles/runtime-A/replay.json' --json
& $Dbg debug-replay run 'Data/debug-profiles/runtime-B/replay.json' --json
~~~

Compare outcome, runtime/ROM/BIOS/dataset/input/endpoint and clock in the two
result.json files. The reference is the recording endpoint: A=passed/B=diverged
is a reproducible difference, not a speed measurement. If both diverge, the
reference is unconfirmed; incompatibility does not mean parity. Recording inputs
and AV remain common and the runtimes are identified separately from the one
that recorded the case.

To locate a difference between interpreter/raw and native or typed **within the
same selected runtime**, use frame-by-frame comparison:

~~~powershell
& $Dbg debug-compare create --recording $Recording --session $Case --dir 'Data/debug-profiles/first-difference' --case 'Raw-vs-typed' --dataset 'C:/Debug/verified-dataset' --frames 100
& $Dbg debug-compare run 'Data/debug-profiles/first-difference/replay.json'
~~~

Choose frames within the recording; for native select an existing library with
--library instead of the dataset. It does not compile new translations. This is
not a frame-by-frame comparison of two different executables. In the menu:
**Choose replay profile > Compare first difference**; Cancel replay interrupts.
[Comparison and scope](debug_comparisons.md).

<a id="4-cercare-letture-fuori-dal-piano-noto"></a>

## 4. Find reads outside the known plan

A runtime with rom-data-read-audit is required: the ordinary build refuses the
audit rather than returning zero reads. Diagnosis JSON exposes
build.features.rom_data_read_audit. In the checkout, the corresponding native
build is cargo build --locked --release -p recomp -p ffta-play --features recomp/rom-data-read-audit;
the packager cannot enable features in an already built binary. Select a
recording and a TOML manifest already verified for the same ROM; no extraction
or regeneration is implicit. Choose a window within the recorded frame count:

~~~powershell
& $Dbg debug-rom-audit create --recording $Recording --session $Case --dir 'Data/debug-profiles/reads' --case 'Read-window' --preset all --from 0 --to 100 --limit 20000 --known-manifest 'C:/Debug/verified-dataset/manifest.toml'
& $Dbg debug-rom-audit run 'Data/debug-profiles/reads/replay.json'
~~~

In the profile's rom-audit.json read known_plan.outside_distinct_bytes, physical
ranges, typed/raw/unclassified service, reader, window, drops and truncation.
Without an authenticated plan the outside comparison is unknown. --range uses
physical ROM offsets, not guest PC addresses. Zero requests applies only to the
observed readers/filter/window; it does not prove absence of use in the game.

The **Audit replay** menu action uses a selected audit profile. In the checkout,
the coverage map can import the retained audit without repeating the case.
The package contains no private coverage or dataset; the JSON report is already
available locally. [Audit](debug_rom_audit.md).

<a id="5-liberare-temporanei-preservando-prove-e-input"></a>

## 5. Free temporary files while preserving evidence and inputs

Close the session and choose a new destination for the plan. Python:

~~~powershell
python3 "$Bundle/scripts/debug_artifacts.py" preview --session "$Bundle/$Case" --out "$Bundle/Data/debug-artifact-plans/revisione"
# Read index.html and plan.json, then apply THAT unchanged plan:
python3 "$Bundle/scripts/debug_artifacts.py" apply "$Bundle/Data/debug-artifact-plans/revisione/plan.json" --receipt "$Bundle/Data/debug-artifact-plans/apply-receipt.json"
~~~

With PowerShell and Docker already prepared, use the launcher with the same
arguments, relative to the bundle root:

~~~powershell
& "$Bundle/scripts/windows/debug_artifacts.ps1" -Docker preview --session $Case --out 'Data/debug-artifact-plans/revisione-docker'
~~~

On Linux: bash scripts/linux/debug_artifacts.sh [--docker] ARGUMENTS.
The plan preserves ROM/snapshot/SRAM/input, reports/errors, datasets,
authenticated runtimes and unknown files; it selects only recognized native
caches and, in the checkout, disposable copies of completed ROM units.
--keep DIR protects an entire session. An active/interrupted lease, changed
plan or unexpected files blocks removal; read the error receipt. Do not remove
a lease just because it is old. Moving or updating the tool requires a new plan.

The tool does not delete entire sessions or reports to free capacity: archive
them explicitly while the game is closed. [Retention and limits](debug_artifacts.md).

<a id="trovare-i-report-e-selezionare-il-lavoro"></a>

## Find reports and select work

Generate the local index after creating/moving reports:

~~~powershell
& "$Bundle/scripts/windows/report_home.ps1" -Python -OpenReport
# Or, with the coverage image already present:
& "$Bundle/scripts/windows/report_home.ps1" -OpenReport
~~~

Linux: bash scripts/linux/report_home.sh --python or without the flag for Docker.
The page lists existing files; presence/mtime does not imply acceptance.
Campaigns select --case/--ds/--family/--transition and reuse pinned JSON;
**report** does not replay the game. **run-case** starts one explicit case
through the existing supervisor and requires a reopening reason for previous
evidence. [Campaigns](debug_campaigns.md).

To control an open window without coordinates, use debug-control. status works
while a command is pending; cancel SESSION --request ID cancels only the observed
ID. --timeout-ms N (20..120000 ms) limits the wait and adds a budget deadline at
the next frontend return. An unconfirmed client timeout does not prove work has
already stopped: check status before retrying. No network server is exposed.
[Local channel](debug_control.md).

<a id="matrice-delle-verifiche-e-limiti"></a>

## Validation matrix and limits

- **Windows GNU through Wine/Xvfb:** DBG01–21 tools verified in documented units;
  optional-audit and default builds have separate scopes. For DBG22 see the
  package validation below. Wine is not native Windows.
- **Native Windows / PowerShell 5.1:** not verified. PowerShell 7.4.6 in a Linux
  container passes 13 launcher checks with simulated public applications; this
  is not native Windows execution. No physical-controller or hardware audio test.
- **Linux host:** Python/Bash, offline reports, networkless Docker, public fixtures
  and packaging tar verified. This is not new acceptance of the current Linux
  runtime; recent interactive tools need dedicated native verification.
- **Docker:** offline tools on existing images. It does not run the GUI, distribute
  datasets or supply missing Wine/Linux runtimes.
- **Android/Godot:** outside the scope of these desktop tools.

The package includes only explicit public inputs: selected binaries, public
database, TinyCC/source/licenses, scripts/templates and guides. Data is empty;
no ROM/BIOS, saves, snapshots, recordings, generated translations, fixtures,
logs or results are included. Distributed documents retain links to included
files; references to absent checkout evidence and sources become explicit text.

<a id="verifica-della-distribuzione--29-ottobre-2026"></a>

## Distribution validation — 29 October 2026

Locked/offline Windows GNU audit release build on frozen public sources plus
explicit overlays; the launcher comes from the same selection. No guest-loop
change: the final Rust delta only aligns help/synopsis with cancellation and
deadlines already implemented.

Ten public packager tests verify ZIP/tar, whitelist and manifest, absence of
personal files even beside inputs, unchanged licenses, document links, Linux
executable permissions, rejected symlinks/ROM headers, errors preserving the
previous archive and logs from a moved folder. The tenth also verifies rejection
of UTF-8/UTF-16LE private prefixes. Thirteen simulated PowerShell 7.4.6 checks
in a Linux container also pass --out forwarding, nonzero outcome, stderr,
readonly/networkless Docker and previous homepage preservation. Null Replace
parameter conversion, hidden temporary removal and flag interception by common
parameters were corrected.

From the extracted Windows package moved to a path with spaces/apostrophe/
Unicode, with the terminal in another directory: 19 CLI invocations through the
launcher, menu Start/Stop for a 1-frame recording, two distinct runtimes replaying
that single endpoint, an issue report and a 1-frame audit. These are 4 new frames
only for guide integration, not rerun old DS/native acceptance or benchmarks.
Both replay clock/endpoints match; the audit distinguishes 27,403 requests/885
bytes outside the known plan in that window only.

Initial test diagnosis looked for the audit flag in the wrong field: the script
was corrected, and three commands already successful before the GUI were reused.
A transient Windows JSON contention was recovered by the existing channel;
the recent window can freeze on that observed error before a later refusal.
No command or frame is rerun to reconstruct context.

Ten offline checks from the moved bundle verify host/Docker, three reused checks
without more frames, repeat refusal without a reason, homepage, plan/apply and
stale receipt without removal. Only 66 bytes of SYNTHETIC cache created after
shutdown and never loaded were removed; 28 real copy files and 11 source input/
runtime guards retain hashes/mtime. GUI and browser screenshots at 1400/620 of
6 new reports were inspected; filters/search/reset/links pass.

Initial binary scanning found personal builder paths. Final binaries were
rebuilt with Rust --remap-path-prefix and TinyCC -ffile-prefix-map, with identical
source selection; binaries are not modified afterwards. Two synthetic TinyCC
checks with Wine pass: compile error with lock/context release and ELF cache with
callback/large stack. No game frames. The packager accepts repeatable
--private-prefix PREFIX, rejects UTF-8/UTF-16LE in payloads and retains the
previous archive on error. Recording the prefix is unnecessary: the manifest
retains only the number of checks.

The distribution record retains identity and CRC/manifest verification of the
final archive, without ROM/BIOS/personal Data. The final extracted/moved binary
passes one additional replay of the retained new recording, with identical
complete state/clock: 5 new frames in total for DBG22. Documentation-only
corrections do not require more frames. Historical DBG01–21 checks remain
references with their own scopes; local availability does not authorize automatic
repetition. [Record](changes/2026-10-29-distribute-shared-debug-tools.json).

[DBG2-08 shared events](debug_events.md) link UI and recent log with shared
identity and correlation; the published map declares partial and missing hooks.
Live consoles and more extensive producers remain in the DBG2 TODO.
