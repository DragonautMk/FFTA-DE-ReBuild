<a id="campagne-mirate--dbg-21"></a>

# Targeted campaigns — DBG-21

A campaign declares a small set of cases and selects the DS, families or transitions affected by a change. **report** only reads the specified JSON files and reuses retained evidence: it does not run the game, create snapshots or reopen completed acceptance checks. The report interface is in English.

The generator's authored messages use English. Labels, tags and recorded
outcome messages imported from evidence retain their original text and identity.

An explicit **run-case** dispatches one profile to the [existing replay supervisor](debug_replays.md). There is no automatic execution of every case.

<a id="manifest-e-selezione"></a>

## Manifest and selection

Create a private JSON file outside commits with this format:

~~~json
{
  "schema": 1,
  "kind": "ffta-debug-campaign",
  "label": "Change to selected reader",
  "cases": [
    {
      "id": "task-entry-EUR",
      "ds": ["DS-20"],
      "families": ["task"],
      "transitions": ["entry"],
      "profile": {"path": "profili/task/replay.json", "sha256": "EXACT_SHA256_64_HEX"},
      "evidence": {
        "result": {"path": "profili/task/Data/config/replay/result.json", "sha256": "EXACT_SHA256_64_HEX"},
        "audit": {"path": "profili/task/Data/config/replay/rom-audit.json", "sha256": "EXACT_SHA256_64_HEX"}
      }
    },
    {"id": "DS03-unprepared", "ds": ["DS-03"], "families": ["scene-transition"]}
  ]
}
~~~

Replace the placeholders with real hashes. Paths are relative to the manifest and use `/`; they may refer to a sibling directory through `../`. Moving the common ancestor of the manifest, profiles and reports preserves the links. DS, family and transition labels are declarations: they do not prove reachability or use of a family by observed reads. Do not confuse a DS case with a slot.

Omit evidence for a profile that has yet to run; omit profile for an unprepared case. Do not change an old hash to hide an altered file. After a new explicit run, declare new pins in a new manifest version while retaining the previous provenance. The tool does not infer a complete historical association between profile and outcome from an mtime.

~~~powershell
python3 scripts/debug_campaign.py report campagne/modifica.json --ds DS-20 --family task --out target/campaign-reports/task-entry
~~~

At least one of **--case ID / --ds DS / --family TAG / --transition TAG** is required. Repeated values of one filter are combined with OR; different filters are combined with AND. An unknown ID is rejected; a filter without matches selects zero cases. Evidence for excluded cases is not opened.

<a id="interpretare-gli-esiti"></a>

## Interpreting outcomes

The report distinguishes:

- **positive**: the retained result declares passed, matched or captured, with its window complete and reported identities consistent. Captured/matched remain distinct from a full endpoint pass.
- **negative**: an already recorded diverged, error, timeout, interrupted or not-reached outcome; it does not imply failure of every feature or scene.
- **incompatible**: a missing or altered profile/evidence, inconsistent identity/window/endpoint, unsupported schema or a recorded incompatible result.
- **not-executed**: outside the selection, unprepared profile or no evidence specified.

**reused-pinned** means reuse of retained bytes authenticated by hashes, not a new run with the current runtime. The report does not open ROMs, snapshots, SRAM or libraries and does not revalidate payload availability, current compatibility or old DS/ROM/native classifications.

For an associated audit, run, case, identity, filter and window must match. **zero-in-selected-window** requires zero requests, a complete window and no truncation or drops. **partial-zero** retains the incomplete collection; **unavailable** means no authenticated audit, never zero reads. Even a complete zero applies only to the selected instrumented readers: **global feature usage remains unknown**.

The report limits the audit to useful metadata; it does not copy all events. Sessions using directory-v1 coordinate reads. For older evidence the pin authenticates bytes without inventing exclusion of an old process unaware of that protocol.

<a id="eseguire-un-solo-caso-scelto"></a>

## Running one selected case

~~~powershell
python3 scripts/debug_campaign.py run-case campagne/modifica.json task-entry-EUR --reason "Modificato il reader di ingresso" --reopen-reason "La modifica invalida questa specifica verifica" --out target/campaign-intents/task-entry
~~~

**--reopen-reason** is required if the manifest contains prior evidence or result.json already exists in the profile, even if undeclared. A new case still requires --reason. Do not use lost context or an open parent TODO as a reason to repeat work; follow the [memory rules](development/MEMORY_WORKFLOW.md#resume-and-repeat-decisions).

The tool verifies profile/runtime pins, saves **intent.json** and replaces its own process with **recomp debug-replay run PROFILO --json**. The same runner handles payload preflight, locking, timeouts, progress and cancellation. An intent is not a completed result. The worker retains logs/results in the profile; a subsequent campaign may explicitly reference those new pins.

On POSIX, a Windows profile requires **--wine /percorso/reale/wine64**. Native Windows and Linux runtimes do not require the launcher. Dispatch does not compile a ROM or runtime. If exec fails, the receipt remains dispatch-error. Final exit codes come from replay. No alternative supervisor is written.

<a id="launcher-limiti-e-privacy"></a>

## Launchers, limits and privacy

**scripts/windows/debug_campaign.ps1** and **scripts/linux/debug_campaign.sh** resolve the root from their location and always show the log under target/build-logs. With --docker (Bash) or -Docker (PowerShell), they use the existing ffta-debug-tools:local image, disabled networking, a read-only container filesystem and a project mount for output/coordination. Docker allows only report/help; dispatch uses host Python. No automatic build is performed.

~~~bash
bash scripts/linux/debug_campaign.sh --docker report campagne/modifica.json --case task-entry-EUR --out target/campaign-reports/docker-entry
~~~

Python 3.11 and the standard library are required, together with artifact_coordinator.py and the HTML template. Limits: 32 cases, 256 KiB manifest, 128 KiB profile/result, 16 MiB audit read, 64 KiB audit summary and 2 MiB report. Duplicate or non-finite JSON is rejected. At most 8 directories per report/intent store, including failed attempts: choose a dedicated parent directory. No automatic overwrite or cleanup occurs. Links/symlinks, reparse points and shared files are rejected.

Outputs and paths are private: a new folder contains .gitignore. Keep the manually created manifest under an already ignored path. There is no upload, promotion of acceptance results or modification of the original collection. The [homepage](report_home.md) finds campaign.json files after regeneration; the [artifact manager](debug_artifacts.md) shows limits and retains these reports.

<a id="verifica-del-29-ottobre-2026"></a>

## Verification on October 29, 2026

**23 distinct Python tests passed**: 12 campaign tests (11 in the refined test plus one added for strict JSON/types/undeclared previous evidence) and 11 artifact manager tests affected by the limits. They cover explicit selection/OR/AND, exclusion without opening evidence, four outcomes, pins/identity/endpoint/window, zero/partial/unavailable, limits, active lease, links/shared files, relocation, reopening reason and receipt. The real test dispatch uses only a public synthetic executable, without game data.

A private report reuses two retained DS20 profiles/evidence (positive audit and historical native divergence), plus synthetic zero/incompatible metadata and an unprepared case. **Zero new guest frames**; no old acceptance was rerun. Eleven normal metadata/input/runtime guards preserve hash/mtime.

The Bash launcher from another directory and offline Docker using the existing image produce selections 1 and 2; no new image/build/runtime. Four Chrome views (desktop/narrow/empty selection/homepage) verify filters/search/reset, outcomes, reuse, zero distinct from unknown global use, and relative links without overflow. Screenshots inspected; the homepage had 72 indexes/4 campaigns at the time.

A final verifier authenticates 8 public sources and four HTML/JSON pairs, pins/links and guards. Only 4 finished browser profiles were retired; all metadata, manifests, reports, sources/preimages, logs and original evidence remain under **target/debug-tools-campaign/**. No managed/native disposable fixture: the ROM completion helper does not apply. Windows/PowerShell, actual Wine or Linux dispatch and physical controllers were not verified in this unit; the already validated supervisor was reused without invoking it on game data. [DBG-21 record](changes/2026-10-29-add-targeted-debug-campaigns.json).

In the PowerShell launcher -Docker precedes the command. Tool flags, including --out, are forwarded literally without PowerShell common parameter interception. The [DBG-22 distribution](debug_tools.md) adds 13 public launcher checks on PowerShell 7.4.6/Linux; native Windows/PowerShell 5.1 remain unverified.
