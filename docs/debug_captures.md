<a id="cattura-al-raggiungimento-di-un-caso-dbg-08"></a>

# Capture when a case is reached (DBG-08)

`debug-capture` and **Debug > Capture on trigger** run an existing complete
recording in an isolated worker. They observe a condition and produce a derived
point at the first subsequent **completed-frame boundary**. The normal game
stays paused. Conditions do not alter input, tasks, registers, clock or scene
state to make it appear.

Use a runtime built with `--features rom-data-read-audit`. The locally verified
Windows candidate is `target/debug-tools-capture/recomp.exe`; the normal
executable is not replaced. The Windows checks described here run under Wine,
not native Windows.

<a id="preparare-un-profilo"></a>

## Prepare a profile

A [complete snapshot/input recording](debug_recordings.md) and its
[closed isolated session](debug_sessions.md) are required. Common parameters
follow [debug-replay](debug_replays.md). Each profile uses a new directory;
creation refuses incompatible or changed sources.

PowerShell example, capture after the fiftieth replay frame:

```powershell
.\recomp.exe debug-capture create `
  --recording 'target/sessione/Data/config/recordings/registrazione' `
  --session 'target/sessione' --dir 'target/cattura-caso' --case DS-20-EUR `
  --frame 50 --limit-frames 100 --timeout-ms 120000
.\recomp.exe debug-capture run 'target/cattura-caso/replay.json'
```

The recording must contain at least 100 frames for this example. The default
limit is the recorded length; execution does not continue past available input.
Choose exactly one of these conditions:

- **First resource read:** `--resource ID --manifest FILE`, with optional
  `--preset all|bus|host`. The manifest must belong to the same ROM and every
  resource range must authenticate by hash. `--dataset DIR` can supply both
  the manifest and an explicit runtime dataset; without a dataset the trigger
  also observes the normal raw path. The plan is copied and identified in the
  profile as `capture-plan.toml`. Capture occurs on the first instrumented read
  intersecting a range, retaining reader, width, Typed/Raw service and any
  interpreter context.
- **Entry into a verified ROM instruction:** `--pc HEX --mode arm|thumb
  --opcode HEX`. PC, alignment, width, expected opcode and decoding are
  checked against the selected ROM. Obtain PC and opcode from analysis of your
  own ROM; a different value is refused. The trigger observes actual interpreter
  entry **with the condition passed**, before execution/HLE of that instruction.
  A PC inherited from an HLE/DMA read is not mistaken for a new entry.
  Instructions with failed conditions, native instructions, BIOS and RAM code
  are not covered.
- **Known frame event:** `--frame N`, relative to replay origin,
  with `1 <= N <= --limit-frames`. Capture occurs exactly after N completed frames.

Resource and PC conditions require the feature's hooks; frame profiles also
use the same capture contract. Profiles are **interpreter only**: `--library`
and simultaneous audits are refused. No native translation is compiled.
Selected runtime and original recording identities remain distinct.

<a id="eseguire-dal-menu-o-dalla-cli"></a>

## Run from the menu or CLI

In the normal game, open Debug, select `replay.json` with **Choose replay
profile**, then **Capture on trigger**. The menu uses the same worker as the
CLI. This action refuses a profile without a condition. **Cancel replay** also
cancels a capture; closing the owner window terminates the worker.

```powershell
.\recomp.exe debug-capture cancel 'target/cattura-caso/replay.json'
```

The profile retains `capture.condition`, `capture.limit_frames` and
`stop: "capture-trigger"`. DBG-05 checks and locks remain active: ROM/BIOS,
runtime, dataset, origin and input are identified, two simultaneous uses are
refused, and the worker has no writer for personal SRAM, slots or backups.
The client can supervise a different explicitly selected runtime; the worker
itself must have the required hooks.

`progress.json`, `result.json` and `worker.log` remain under
`Data/config/replay/`. The result declares the run and effective limit:

- `captured`, exit **0**: trigger observed and bundle published;
- `not-reached`, exit **7**: frame limit reached, no snapshot;
- **3** incompatible, **4** timeout, **5** interrupted, **6** worker error,
  following replay supervision.

The menu shows **CAPTURED** or **NOT REACHED**. “Not reached” means only that it
was not observed in the selected collectors, thread and window; it does not
prove a resource is never read. Interruption or timeout produces no new capture.
Old bundles remain independent of the latest result: always compare the run.

<a id="riutilizzare-il-punto"></a>

## Reuse the point

Each success publishes a new `captures/<run>/` directory in the profile:

- `capture.ffstate`: complete state at the safe boundary, loadable normally;
- `origin.ffstate` and `input.input`: copies of the authenticated buffers
  actually used, retained even if profile files change after execution;
- `capture.json`: reason, first observation, effective boundary/frame/clock,
  snapshot, ROM/BIOS/runtime/dataset and input hashes, origin and limits;
- `debug-catalog.json`: ready local catalog, with a **variant** entry.

The `profile_settings_sha256` fingerprint identifies the compact serialization
of the settings actually loaded, not a late reread of the file.
`input_end_exclusive` identifies how many recorded frames were applied;
the original input file may contain later events. Retain the profile with
`capture-plan.toml` if the resource's source plan needs to be reexamined.

Open the catalog from [Debug catalog](debug_catalog.md), or:

```powershell
.\recomp.exe debug-catalog list 'target/cattura-caso/captures/RUN/debug-catalog.json' --json
.\recomp.exe debug-catalog open 'target/cattura-caso/captures/RUN/debug-catalog.json' `
  --entry 'CAP/RUN' --rom 'rom/la-propria-rom.gba' --no-bios --interp `
  --dir 'target/punto-riutilizzato'
```

Use the BIOS matching the origin when present, instead of `--no-bios`.
Opening creates another isolated session; slot import uses the existing path
with a free slot and Undo. The catalog retains **variant** status without
promoting the named DS scene. First observation and snapshot time are distinct:
a read in the observation frame is retained before stopping at the next
boundary, without interrupting a DMA/HLE callback.

At most **eight bundles per profile** are retained. The ninth launch is refused
before the worker starts; existing points are not overwritten or deleted.
Archive bundles explicitly or create a new profile. Partial copies are prepared
in a temporary directory and renamed only at publication. All payloads remain
private under `target/` or another ignored destination; do not put them in
commits or packages. The personal catalog is not rewritten.

<a id="ambito-dellosservazione"></a>

## Observation scope

The first-read collector is separate from the audit inventory: it retains a
single match, without an event list or cap-related drops. It recognizes the
same six readers as DBG-06, excludes fetches and normalizes physical mirrors.
Ranges are sorted/merged with offset lookup; the limit is 100,000 resource
ranges and 128 MiB total to authenticate, within a 16 MiB manifest. It does not
observe other threads, uninstrumented host/HLE accesses, native constants,
out-of-image reads or EEPROM/GPIO paths excluded by the hooks.

The condition selects authenticated ranges; it does not prove new data
ownership, extraction completeness or semantic presence of a scene. No machine
state or ABI was extended to retain the collector.

<a id="verifiche-effettuate"></a>

## Validation performed

The isolated Windows build starts from public commit `acd086e` plus eight
selected DBG-08 files; concurrent extraction changes remain excluded.
Feature and default builds compile without warnings. Under Wine, **66 tests**
pass: five capture, eleven replay, three trigger collector, two bounded-audit
regressions and 45 menu tests. The publication correction uses already
authenticated buffers, with a check that changes the input file externally
after execution; core/menu tests with identical binaries were retained without repetition.

**25 CLI checks**, a controlled cancellation process and four client/worker
checks with and without the feature verify creation, capture, limit, wrong
identities/opcodes, altered plan, timeout, native exclusion, locks, cancellation
and retention. On the existing DS-20 replay, frame, resource and PC triggers
produce the same complete state after one frame, with clock 6,241,425,344.
The first resource read is `SpritePeek`: the initial test expectation of
`ObjSourceTable` was corrected without rerunning already valid captures. The
catalog recognizes the point as a variant and opens an isolated session with
an identical seed. The not-reached case stops at two frames, with state/clock
equal to the second-frame capture control and no new snapshot.

The Wine/Xvfb GUI verifies **CAPTURED**, **NOT REACHED**, cancellation and owner
shutdown; main-window state/frame/clock and progress remain unchanged. Final
screenshots were inspected. The 203 selected original inputs, including the
personal collection and both normal executables, retain hashes and modification
times. Old DBG-04/05/06, native or extraction acceptance checks are not rerun,
and no new DS scene is promoted.

Native Windows, Linux/Wayland, physical controllers and capture with an explicit
typed dataset are not verified in this unit. Real checks use an already
retained e76 source plan and the raw path; the later committed runtime model
remains distinct. Logs, derived points and local results are under
`target/debug-tools-capture/`; future unavailability does not require recreating
or repeating already closed checks.

To locate an interpreter/native or raw/typed difference, use
[DBG-09 comparisons](debug_comparisons.md). The captures described here produce
reusable variants on a trigger; comparison instead retains state and streams
observed before the frontend drain only at the first differing boundary, with
field names and side-by-side images. The two profile types remain distinct.
