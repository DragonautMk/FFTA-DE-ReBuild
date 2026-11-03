<a id="sessioni-e-artefatti-di-debug--dbg-18"></a>

# Debug sessions and artifacts — DBG-18

The manager produces an English **HTML/JSON preview**, with estimated space and sessions to retain. Apply uses that plan and a new receipt. It selects generated libraries in completed session caches and disposable copies of already recognized ROM test units covered by the [existing cleanup](development/LOCAL_ARTIFACTS.md).

The generator's authored CLI messages use English. Paths and imported metadata
retain their original values; their language does not change artifact identities.

Sessions and their originals, inputs, saves, recordings, datasets, reports, errors, manifests, authenticated executables and source preimages are retained. `--keep` excludes a session entirely from cleanup. Unknown files are retained; the manager does not perform a global cleanup of `target/`, Docker, Cargo or normal caches. Earlier collector limits remain in effect: eight folders per type and the event/frame/time and size limits documented in the [guides](development/DEBUG_TOOLS_TODO.md) and summarized in the preview.

<a id="python-o-powershell"></a>

## Python or PowerShell

You need **Python 3.11+**, the complete public `scripts/` directory, and Git when the path is in a checkout or ROM units are selected. Building the game is unnecessary. Preview writes only its report and transient coordination leases; it removes no payload.

From the portable/project root, select folders you own:

```powershell
python3 scripts/debug_artifacts.py preview --session 'Data/debug-sessions/case one' --session 'Data/debug-sessions/case two' --keep 'Data/debug-sessions/case one' --out 'target/artifacts-preview-1'
python3 scripts/debug_artifacts.py apply 'target/artifacts-preview-1/plan.json' --receipt 'target/artifacts-receipt-1.json'
```

`--sessions-root FOLDER` lists immediate children without searching the entire project. `--session` and `--unit target/rom-data-NAME` may be repeated; `--unit` declares that unit's checks complete and reuses the old helper's `interrupted` / `moved resources` selection. The accepted dataset, results, originals and evidence are retained. The new manager does not retire unit executables; the old explicit option remains separate.

To always obtain the command log, even when starting from another directory:

```powershell
.\scripts\windows\debug_artifacts.ps1 -ToolArguments @('preview', '--session', 'Data/debug-sessions/case one', '--keep', 'Data/debug-sessions/case one', '--out', 'target/artifacts-preview-2')
```

Or on Linux:

```sh
bash scripts/linux/debug_artifacts.sh preview --session 'Data/debug-sessions/case one' --out 'target/artifacts-preview-3'
```

Launchers resolve the root from their location and identify the file in `target/build-logs/`. Exit **0** means the command succeeded, **1** an error or changed plan; preview can succeed with protected contexts and zero candidates, so read each row's status. No command overwrites an existing preview folder or receipt. After regeneration, the [homepage](report_home.md) indexes local previews.

## Docker

The launcher uses `ffta-debug-tools:local`, derived from the existing coverage image. The derived image adds Git, which the original coverage image lacks. Preparation requires networking for Debian packages; subsequent management commands use `--network none`, a read-only container filesystem and a project-folder bind mount. They do not copy game data into the image or build Cargo.

```sh
docker build -f Dockerfile.debug-tools -t ffta-debug-tools:local .
bash scripts/linux/debug_artifacts.sh --docker preview --session 'Data/debug-sessions/case one' --out 'target/artifacts-docker-1'
bash scripts/linux/debug_artifacts.sh --docker apply 'target/artifacts-docker-1/plan.json' --receipt 'target/docker-receipt-1.json'
```

On PowerShell use `-Docker -ToolArguments @(...)`. Host Python and Git are not required for this path. Docker Desktop/daemon and the coverage image are prerequisites. Preview and apply must use the same launcher: Docker paths are `/project/...`. Generate a new preview after moving the folder or updating the tool. Retained plans describe file locations and identities at selection time.

<a id="coordinamento-e-conservazione"></a>

## Coordination and retention

New isolated copies declare `artifact_coordination: directory-v1` in the manifest. Runtime, session creation, the manager and coverage cache generation/rotation share the protocol. A `.debug-artifacts-lease` directory and its `owner.json` identify the owner; atomic directory creation avoids assumptions about interoperability of Windows, Wine, Python and container locks. The normal session lock remains in place. The lease is acquired before verification/copying and held throughout the operation or execution; a concurrent process refuses an in-use context.

A killed process leaves a protective lease. There is no automatic PID-based recovery: a PID does not identify the same process across host/Wine/container. Before manually removing a residual lease, verify the process and logs that created it and completion of the work. A legacy session without the protocol, an incomplete or failed session, or one containing links/shared files remains protected. Its authenticated executable is not replaced to make it eligible for cleanup.

Apply reconstructs the plan, acquires all required leases and rechecks candidate identities and, for sessions, retained-file identities under coordination before the first removal. Changes, new captures, a started runtime or an altered plan require a new preview. A distinct new session cannot enter the already fixed selection. Git-tracked files are rejected.

Receipts record the plan, pending operation, successful removals and error or completion; they are written and synchronized before and after each removal. An interruption can leave a partially applied prefix: inspect the receipt and files, then create a new preview. Automatic coverage rotation always retains the current generation and writes receipts in the cache's `.retention/`, up to 64 receipts; beyond that limit cleanup stops while retaining reports and datasets. Explicit readers of a `CACHE/SHA256/eN-dN` generation acquire the same lease without enabling pruning. When Docker hides the path behind `/input/dataset`, both launchers mount the coordination root separately and pass `--cache-lease /cache-lease`: the dataset remains read-only. Direct Python invocation recognizes the layout; custom invocations can specify `--cache-lease ROOT_CACHE`. An active marker nested in a ROM copy protects the entire unit, including other diagnostic copies.

All processes sharing a cache must use the updated tools: external readers or older versions do not participate in the protocol.

Manager limits: **32 contexts, 8,192 entries per examined tree, 8,192 candidate files, 16 MiB per plan/receipt**; payloads are not read to estimate space. Logical bytes are exact for the selection; allocated bytes use `st_blocks` when present, otherwise logical bytes with `allocation_estimated: true`. Interrupted leases, incomplete folders and errors remain available for diagnosis. Receipts describe local availability: retired payloads do not reopen completed evidence or authorize rebuilding or replaying to restore them.

<a id="verifiche"></a>

## Verification

[DBG-18 record](changes/2026-10-29-coordinate-debug-artifact-retention.json). Python checks use public synthetic placeholders; Wine checks verify the paused runtime without executing new guest frames. Inputs and earlier evidence remain retained. The matrix distinguishes Python/Linux and Docker from Windows GNU/Wine and PowerShell/native Windows, which were not run.

In the PowerShell launcher -Docker precedes the command. Tool flags, including --out, are forwarded literally without PowerShell common parameter interception. The [DBG-22 distribution](debug_tools.md) adds 13 public launcher checks on PowerShell 7.4.6/Linux; native Windows/PowerShell 5.1 remain unverified.
