<a id="audit-delle-letture-rom--dbg-06"></a>

# ROM-read audit — DBG-06

The audit observes ROM data reads during isolated replay. It does not extract
new assets, alter game bytes or update coverage or DS evidence. It reuses
existing bus/host collectors, adding filters, a frame window, event cap and
actual typed/raw service path.

An executable built with **`--features rom-data-read-audit`** is required.
The feature remains disabled in normal builds. If absent, the request is
refused: an empty report is not presented as an absence of reads.

<a id="preparare-e-avviare"></a>

## Prepare and run

Start with a [complete recording](debug_recordings.md) and its
[closed copy](debug_sessions.md), as for [replay profiles](debug_replays.md):

```powershell
.\recomp.exe debug-rom-audit create `
  --recording 'C:\Debug\DS-20\Data\config\recordings\recording-…' `
  --session 'C:\Debug\DS-20' --dir 'C:\Debug\audit-DS-20' --case 'DS-20-EUR' `
  --preset all --from 0 --to 102 --limit 20000 `
  --known-manifest 'C:\Dati\FFTA\manifest.toml'

.\recomp.exe debug-rom-audit run 'C:\Debug\audit-DS-20\replay.json'
```

Create uses a new destination and retains the audit profile with the inputs.
Run and Cancel use the same worker, timeout, cancellation and outcomes as the
`debug-replay` CLI. No ad hoc Rust tests, Python on the user's computer or
coordinate automation for the assistant are needed.

Filters:

- **`--preset all|bus|host`**: all instrumented points, bus only or selected
  host points only. The default preset is all.
- **`--from N --to N`**: frames relative to the snapshot, interval `[from,to)`.
  Replay still proceeds to the recorded endpoint; only selected frames are
  observed. Without to, the window ends at the recording's end.
- **`--range START:END`**: **physical** offset interval `[start,end)`, decimal
  or `0x` hexadecimal. Repeatable up to 32 times, for example
  `--range 0x370000:0x380000`. These are not guest `0x08…` addresses. The filter
  includes reads intersecting the selection, retaining their full width.
- **`--limit N`**: 1 to 50,000 distinct events, default 20,000. Retained
  duplicates increase request counts. New events beyond the cap increment
  dropped_requests and make truncated explicit.

To compare an already available generation of decoded data, `--dataset DIR`
copies and authenticates its files as resources of the selected runtime;
the source recording remains intact. The endpoint must continue to match.
Without an override, the source session's pinned resources apply. No dataset
is published or regenerated.

`--known-manifest FILE` supplies a source plan for comparison. Schema, ROM
identity/size and range bounds are checked, with metadata copied into the
profile. It is a provenance reference, separate from consumption evidence.
The ROM is not reextracted to verify the plan. Without the flag, the selected
dataset's manifest is used if present; otherwise known_plan.compared is false
and out-of-plan bytes are not invented.

<a id="dal-menu"></a>

## From the menu

Preselect the profile with `play --debug-replay FILE` or **Choose replay profile**,
then choose **Debug → Audit replay**. The menu requires a profile with an audit
configured. Replay case also runs the audit when the profile contains one.
Cancel replay and closing the window stop the worker. The normal game stays
paused and retains its progress.

The replay's JSON outcome identifies the report, truncation and window
completion. **Passed concerns the replay endpoint**; a truncated or partial
audit remains so even when that endpoint matches.

<a id="leggere-il-report"></a>

## Read the report

The profile retains only the latest `Data/config/replay/rom-audit.json`,
schema 1, kind `ffta-rom-read-audit`, with case, run and input identities.
An error or interruption before collection may produce no new audit: check
that its run matches the result's run. An old report is not a new observation,
and a missing report does not mean zero reads.

Ranges are physical, sorted and merged: overlapping reads, different widths
and ROM mirrors do not count observed_distinct_bytes twice. Bus and host
counters may overlap; bus_host_shared_distinct_bytes measures that intersection.
Plan-owner totals may overlap and must not be added to reconstruct the global union.

Each event distinguishes reader and actual **Typed**, **Raw** or **Unclassified**
service, independently of whether the manifest has an owner. Typed/raw totals
may overlap on the same bytes through different paths. The report retains
request count and first/last guest frame per key. Events beyond the cap do not
contribute to retained ranges; those bytes are not proven absent or unconsumed.

PC and ARM/Thumb mode, when available, are an **interpreter scope**: HLE and DMA
may inherit it. They do not prove that instruction executed a direct LDR at
the address. None indicates native/no observed scope without inventing a PC.
Attributing an operand still requires checking the reader.

The bus and selected AudioValue, SpritePeek, SpriteList, ObjSourceTable and
KeyboardWidths helpers are covered. Limits remain explicit:

- interpreter instruction fetches excluded; the count does not measure native fetches;
- other direct host accesses, uninstrumented typed HLE fast paths and other threads;
- embedded native constants, code/signature authentication and failed reads;
- GPIO/EEPROM, out-of-image regions, filtered ranges/frames and dropped events.

An access outside the known plan is a lead to investigate, not a newly extracted
family. These reports are not the complete ROM map: linking scenes, resources
and ROM-coverage layers remains DBG-07.

<a id="verifiche-del-28-ottobre-2026"></a>

## Validation on 28 October 2026

Offline Windows release build on frozen public base `2fbf9f7` with only selected
tool files and audit hooks. The large runtime hub is compiled from its public
version plus four hook replacements; concurrent extraction changes remain in
the workspace and outside the build. Final candidate:
`target/debug-tools-audit/recomp.exe`, SHA-256
`84f2bdc64e1fa6180d89b357204f6a23a572769436ccd4054b6cc042714a3343`.
Normal Windows/Linux executables are preserved.

- **61 tests under Wine:** 6 audit, 2 bounded-collector, 8 replay and 45 menu.
  They verify equal complete state and clock with observation off/on, fetch
  exclusion, scope, typed/raw, filters, cap, mirrors, known plan, duration/drop
  and unchanged projection of historical collectors.
- **DS-20 CLI:** replay of 102 frames with identical endpoint and clock. The
  complete two-frame window observes 12,416 B both raw and with the e76 dataset:
  18,755 raw and 18,754 typed events. The typed path serves 11,495 distinct B;
  another 997 B are observed raw, with 76 B overlap between paths. The e76 plan
  contains 11,495 B of the union; 921 observed B remain outside that plan,
  without automatic classification as new resources.
- **Truncation:** 102 frames reach the 50,000-event cap: 2,557,121 matching
  requests, 1,251,772 dropped and 31,061 B retained. The replay result passes;
  the audit remains explicitly truncated. A cap of 8 events is also checked
  without changing state or clock.
- **15 final CLI cases**, beyond accepted references, cover raw/typed, host/bus
  presets, ranges and window, cap, a profile without audit and missing/changed
  references. An independent Python check reconstructs unions, plan
  intersections, filters and request/drop arithmetic. Two further checks confirm
  create refusal without the feature and explicit use of an enabled worker
  through a supervisor without the feature.
- **Wine/Xvfb GUI:** Audit replay, TRUNCATED/PARTIAL warnings, cancellation and
  worker shutdown pass. Parent-game state/frame/clock and progress unchanged;
  238 selected collection/ROM/recording/dataset inputs retain hashes/mtime.
  Final screenshots inspected.

The real check found the e76 manifest of 7,926,499 B exceeded the initial local
4 MiB cap: shared readers now use the project's existing 16 MiB cap. The first
check incorrectly assumed 50,000 events would suffice for all 102 frames:
that report remains truncated and the complete check uses a two-frame window.
Those results and attempts are retained. The first GUI launch in the shared
Wine prefix failed before the window; the dedicated graphical prefix passes
without a runtime change. Accepted tests were not repeated to reconstruct context.

Evidence in `target/debug-tools-audit/`: `test-results.json`, `cli-results.json`,
`cli-summary.json`, `default-cli-results.json`, `gui-results.json`,
`default-hook-verification.json` and retained attempts. Both builds have no
warnings; code for the four core paths with the feature off equals its preimage
after removing hooks and normalizing line endings.

Native Windows, Linux/Wayland and physical controllers remain unverified.
Native parity comparisons, Bank/F09 campaigns, extraction, new DS checks,
performance or Android/Godot were not reopened. Linking reports to ROM coverage
remains the subsequent DBG-07.

<a id="collegamento-alla-mappa-rom"></a>

## Link to the ROM map

[ROM coverage](development/ROM_COVERAGE.md#scene-letture-e-confronto-dbg-07)
imports retained reports through `--audit` / `-AuditPath` without a new replay.
It allows selecting windows, viewing distinct bytes per resource/case, tracing
evidence from the map and comparing two snapshots. Observed ranges remain
separate from positive families; limits, filters, truncation and different
manifests stay visible. Reports must belong to the same ROM.

To stop at the first resource read or verified PC entry, use
[DBG-08 captures](debug_captures.md). They reuse hooks with an independent
collector retaining only the first match and publishing the snapshot at the
next frame boundary. No complete event inventory is needed; capture and audit
profiles remain distinct.
