<a id="report-locale-del-problema--dbg-16"></a>

# Local issue report — DBG-16

The report gathers the symptom, case, frame/input, identity and retained result
of a [replay profile](debug_replays.md), with an optional link to the first
divergence comparison and a suggested next check. It does not execute the case,
send files or duplicate ROMs, saves, snapshots or dumps.

<a id="creare-il-report"></a>

## Create the report

Windows CLI with the updated runtime:

```powershell
.\recomp.exe debug-report create '.\profile\replay.json' --symptom 'Describe what happened and what you expected'
```

JSON returns `Data/config/issues/<run>/index.html` **inside the selected profile**.
`issue.json` contains complete context and links; `summary.json` contains only
metadata to review before sharing. The description accepts at most 2,048 bytes.
When omitted, it displays “Not supplied”; the report does not invent a symptom.

In a paused isolated copy, choose **Debug > Create issue report**. It uses the
selected replay or opens the `replay.json` selector. Selecting does not run the
replay. The action uses the latest available replay message as its description;
use the CLI for an explicit description. The equivalent command for the same
window is:

```powershell
.\recomp.exe debug-control issue-report '.\session' --profile '.\profile\replay.json' --symptom 'Observed symptom'
```

`--profile` may be omitted when a selection already exists. `status.issue_report`
exposes the latest path. The action requires pause, without pending budgets,
probes or replays; it observes guest state without changing it. The report belongs
to the selected profile, which may differ from the copy used to open the menu.

In an isolated EUR Wine/Xvfb check, local-control CLI, the issue parameter form
and the command console each created a report for the same selected, pinned
profile with an empty symptom. Their summaries matched, preflight was compatible,
and guest state remained unchanged. Creating these reports did not run replay.

<a id="leggere-e-riprodurre"></a>

## Read and reproduce

The report shows preflight, the retained result and next check. The summary
distinguishes `recorded_frames` from the `selected_frames` window and `completed`
frames; the comparison adds `first_difference` and `different_components`. The
English interface opens report/profile directories in the browser, copies the exact
local-launcher command and, when identified, opens the retained comparison.
Terminal selection distinguishes PowerShell/Windows and shell/Linux; Windows
profiles require `wine64` when run on Linux.

`reproduce.ps1` and `reproduce.sh` resolve the profile relative to themselves.
They execute the **runtime pinned in the profile** with `debug-replay run`, its
`replay.json` and JSON output. They do not consult a newer build or reconstruct
game environment variables. The worker retains normal separation from game-save
writes. Replay exit codes remain those described in DBG-05.

Moving **the entire profile directory**, including reports and payloads, preserves
links and launchers. Moving only the HTML does not transfer the case. Reopening
HTML after a move updates the copyable command for the current location. Report
generation does not alter the original replay. Executing the command remains a
separate, explicit choice.

<a id="errori-e-valore-della-prova"></a>

## Errors and evidentiary value

Preflight checks profile schema/limits, session pins, runtime/ROM/BIOS/dataset/
seed/input/endpoint/library hashes and input parsing. It executes no instructions
and does not load the native library; the worker remains responsible for snapshot
decoding, actual compatibility and endpoint verification.

Changed input or an incompatible profile still produces HTML/JSON with an error
and next check: the standalone CLI returns **3** and the report path. **0** means
a report was created with compatible preflight, not that the case was rerun or
the bug fixed. An invalid destination path, busy lock, malformed parameters,
exhausted capacity or write failure can prevent publication; the CLI explains why.

A previous result is accepted only when schema and available runtime/ROM/input/
library/dataset identities match. Its presence does not authenticate freshness or
every setting of an old profile. A comparison also requires matching ROM/runtime/
input/seed/library and run. These checks concern **already retained bytes**, not a
new correctness acceptance. A missing or incompatible result remains unknown,
not “case never executed”.

<a id="privacy-e-limiti"></a>

## Privacy and limits

- ROMs, BIOS, input, states, datasets, native translation and dumps remain
  **private** attachments referenced by relative links. No automatic upload or
  message occurs.
- `issue.json` includes errors, pins and local paths and is private. `summary.json`
  excludes source paths and game bytes, but symptom/case name can still contain
  personal information. The term “metadata” does not automatically mean safe to share.
- At most eight directories per profile, including incomplete attempts, and 2 MiB
  JSON. No automatic overwrite or cleanup. A separate writer lock coordinates two
  concurrent creations. Creation does not execute the launchers.
- Mailbox limits require a profile path within 1,024 bytes and a serialized request
  within 3,500 bytes; shorten the description/path when escapes exceed the limit.
  The standalone CLI does not use that mailbox.

The [local homepage](report_home.md) recognizes `issue.json` after regeneration.
Prerequisites, actual checks and unverified platforms are in the
[DBG-16 record](changes/2026-10-29-add-reproducible-local-issue-reports.json).

Output-directory checks continue to reject symbolic links and Windows reparse
points. The [October 30 fix](changes/2026-10-30-remove-report-link-flag-warnings.json)
removes the Linux `unused_mut` warning; affected blocks compiled without warnings
for Linux and Windows without rerunning report checks.

<a id="verifiche-del-29-ottobre-2026"></a>

## October 29, 2026 checks

Locked/offline Windows feature/default builds without warnings; 72 Wine tests
(collector, controller, menu and frontend), plus one focused rerun after separating
summary counts. Seven standalone CLI commands and one launcher checked creation
without execution, readonly import of the retained divergence and publication of
an incompatible report. A case created from an already accepted recording was
moved with spaces/apostrophe/Unicode: SH/Wine launcher from an unrelated cwd
reached the exact complete two-frame endpoint.

Menu and CLI in the paused copy preserved state/frame/clock; 37 invocations included
navigation and selection/capacity/free-running/closure refusals. Capacity checks
preserved all report hashes/mtimes; the parent closed normally. All 28 actually
selected original inputs remained intact. Chrome checked actual clipboard against
the previously executed launcher's argv, local folder opening, PowerShell quoting,
error/comparison and desktop/mobile layout. Summary revision checked 102 recorded
frames versus 2 selected, first difference 1 and CPU/state/clock components without
a new replay. Homepage indexed 49 reports, eight issues; screenshots inspected.

Evidence remains under `target/debug-tools-issue/`; private reports/payloads are
Git-ignored. Native PowerShell/Windows, native Linux, other browsers and physical
controllers were not exercised. The preceding native comparison was only imported:
its parity was not reopened or certified here.
