<a id="primo-punto-di-divergenza-dbg-09"></a>

# First point of divergence (DBG-09)

**Debug > Compare first difference** and `debug-compare create/run/cancel`
compare every completed-frame boundary from recorded snapshots and input. The
reference uses **interpreter/raw**; the candidate may use the interpreter or an
existing native translation, optionally with a typed dataset. The worker holds
two independent machines and advances them sequentially under the contract used
by existing raw/typed ROM checks. View settings are shared. The main game remains
paused.

The verified isolated Windows candidate is
`target/debug-tools-compare/recomp.exe`; the normal executable is not replaced.
The feature is also available without `rom-data-read-audit`. Windows validation
described here uses Wine/Xvfb.

<a id="preparare-ed-eseguire"></a>

## Prepare and run

Use a [complete recording](debug_recordings.md) and its
[closed isolated session](debug_sessions.md). Creation checks the same identities
as [DBG-05](debug_replays.md): selected runtime and recording runtime, ROM/BIOS,
dataset, origin, input and retained endpoint. The destination must be new.

PowerShell example, interpreter/raw versus interpreter/typed:

```powershell
.\recomp.exe debug-compare create `
  --recording 'target/session/Data/config/recordings/recording' `
  --session 'target/session' --dir 'target/comparison' --case DS-20-EUR `
  --dataset 'target/authenticated-dataset' --frames 100
.\recomp.exe debug-compare run 'target/comparison/replay.json'
```

To compare interpreter/raw with native/raw, replace `--dataset` with
`--library 'out/existing-cache.dll'`. `--backend native` is selected automatically
when a library is present; it can also be declared explicitly. The cache must be
the intended one for the ROM and compatible with the selected runtime. It is copied,
identified and loaded through TinyCC; no new translation is compiled. Without a
library the candidate is the interpreter. A backend inconsistent with library
presence is refused.

Shared `--runtime` and `--timeout-ms` parameters retain replay semantics. `--frames`
is between 1 and the minimum of recorded length and **3,600**; the default is that
minimum. No new input is generated beyond the recording. The selected dataset is
authenticated, copied into the profile and active on the candidate. The reference
always recreates the raw view from the same state; ROM/BIOS and initial guest data
remain identical.

In the menu, select `replay.json` with **Choose replay profile**, then **Compare
first difference**. This action refuses an ordinary or capture profile. **Cancel
replay** and closing the owner window also interrupt comparison. The equivalent
command is:

```powershell
.\recomp.exe debug-compare cancel 'target/comparison/replay.json'
```

Supervision retains exclusive locks, run identity, deadlines, progress/result JSON
and interruption handling. The worker has no personal SRAM, slot or backup writers.
Comparison, audit inventory and capture triggers cannot be combined in one run.

<a id="cosa-viene-confrontato"></a>

## What is compared

Origin is checked before starting. At each boundary aligned by **frame number**,
the comparison checks:

- complete serialized CPU/Bus state, with ROM/BIOS identity;
- guest clock, frame counter and `frame_ready`;
- complete CPU, central RGB555 framebuffer, entire wide framebuffer allocation
  and OAM;
- mixed and PSG i16, HLE by f32 bits, FIFO A/B events as sample and period.

Observation reads structures and buffers; it does not perform diagnostic I/O
reads, advance the clock or change state to force a match. Frontend audio queues
are observed before draining and drained symmetrically before the next frame,
including origin. Only the latest matching point is retained in memory, without a
timeline of dumps.

`--audio preserve` retains source settings. The report records HLE presence, active
taps and actual counts for each stream. `--audio arm-hle` explicitly enables taps
and HLE shadow symmetrically before the window: the added setup is declared in the
report. Zero counts or disabled paths do not prove audio equivalence for those paths.
Counts are **values/events**, distinct from file bytes.

Frames share origin and index; backends may return at different granularity, by
instruction or native block. Observed clock and PC therefore remain in the
comparison without artificial synchronization. The first differing frame locates a
boundary, not the responsible instruction or, by itself, cause or backend correctness.

<a id="risultato-e-report"></a>

## Result and report

The profile retains `comparison` and `stop: "first-divergence"`. Supervisor JSON
continues under `Data/config/replay/`, with `comparison_report` relative to the
profile directory:

- **0, `matched`:** all declared components match in the selected prefix;
- **2, `diverged`:** first differing boundary, with index and components;
- **3:** incompatible input/profile, **4:** timeout, **5:** interrupted,
  **6:** worker error.

The menu shows **DIFF FRAME N** or **MATCHED N/N**. An incomplete step exceeding
the guard is not presented as the first differing frame. Comparison does not
certify the full recording endpoint: that remains the ordinary DBG-05 operation.
A matching prefix does not promote global parity, resources, data properties or
DS-case status.

Open `comparisons/<run>/index.html` in a browser. Its fixed interface uses English,
works offline and includes `comparison.json`, actual identities, settings, scope,
counters and hashes. When a difference appears, only at that point it retains:

- `left.ffstate`, `right.ffstate` and the last matching `previous.ffstate` when present;
- side-by-side central/wide PNGs, with view selection and expandable previous image;
- files for five nonempty streams: little-endian interleaved stereo i16 for
  mixed/PSG, little-endian IEEE f32 for HLE, five-byte FIFO records
  (`i8 sample`, `u32le period`);
- differing CPU/Bus fields, general-purpose registers, EWRAM/IWRAM/OAM vector,
  pixel and stream differences, with at most 16 examples per entry.

PNGs represent active PPU buffers. Frontend overlay/composition remain outside
comparison; padding pixels in wide memory remain included in state/full-buffer
comparisons. Images may match when only CPU, clock, pixel metadata or audio differs.

Field names come from the **actual Serialize implementation of the current schema**.
Values are postcard-encoded as in saves, including private members and float bits.
Offsets shown in this section belong to each encoded value: they are not guest
addresses or offsets in the full snapshot file. Registers and declared-vector indexes
are exposed separately. The structure is not guessed from save varint bytes.

A match or interruption publishes only HTML/JSON, without images, snapshots or
streams. An initial divergence may have no previous point. Artifacts remain
diagnostic, without catalog or DS promotion; retain the profile with ROM/BIOS,
input and dataset needed to reproduce them. At most **eight reports per profile**:
the ninth start is refused before the worker. Old reports are not overwritten;
archive explicitly or create a new profile. Payloads are private and Git-ignored,
outside commits/packages. Associate the newest result with its own run rather than
confusing it with retained older reports.

<a id="verifiche-effettuate-e-limiti"></a>

## Checks performed and limits

The isolated build uses public commit `7bb07bf` (**e80-d79**) plus ten selected files;
concurrent extraction changes remain excluded. Feature/default builds compiled
without warnings. **67 Wine tests** passed: eight comparison, two field serializer,
twelve replay and 45 menu. A synthetic fault at the second frame checked field,
register, memory, images, last matching point and exact frame recognition; checks
cover NaN bits, audio counts, limits, incomplete frames, cancellation and publication
without overwrite. The initial synthetic input lacked the `gba-input v1` header;
the validator correctly rejected it. Only the fixture was fixed; logs were retained.

**20 CLI checks**, one controlled cancellation process and three default/feature
checks passed. In the two selected DS-20 frames, raw/raw, raw/typed and symmetric
audio setup matched. Default without audit hooks produced the same hashes as the
feature candidate. Diagnosis of a native cache already known to diverge stops at
the **first frame**, without rerunning the old 102-frame endpoint comparison:

- guest frame 22,220 from origin 22,219;
- interpreter clock 6,241,425,344, native 6,241,425,348, delta **4 cycles**;
- only serialized `bus.clock` and `cpu.regs` fields differ;
- `r15`: `0x0800045c` / `0x0800044c`;
- central/wide framebuffer, OAM and five observed streams match: 2,194 mixed/PSG/HLE
  values and 264 events per FIFO on each side.

These are observed boundary results, without repairing the cache or newly accepting
its parity. Typed comparison uses the accepted e80 dataset; it is not an inventory
of bytes consumed per resource, which remains [DBG-06](debug_rom_audit.md). Wine/Xvfb
GUI checks the same first difference, matching prefix, cancellation and owner
closure; main-window state/frame/clock and progress remain unchanged. The browser
was checked at 1,400/620 px, including wide images and a matched report without
heavy payloads. All **276 selected original inputs**, including the two normal
executables, retained hashes and mtimes.

Comparison uses one runtime with two guest machines; comparison between two distinct
executable versions is not implemented. Native Windows, Linux/Wayland, physical
controllers, other ROMs/datasets/scenes and instruction-granularity search remain
unverified or outside scope. No new benchmark, ROM/DS acceptance, complete native
correctness or Godot work is inferred. Local evidence: `target/debug-tools-compare/`;
its possible unavailability does not reopen closed checks.

To inspect PPU buffers and frontend composition in a stopped copy, use the
[DBG-12 inspector](debug_graphics.md): offline report with palette/tile/OAM,
windows and backing pixels. Backend frame comparison here remains separate from
readonly capture and frozen-register projections.
