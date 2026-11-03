# Local validation artifacts

ROMs, saves, input recordings, generated datasets and test evidence have distinct
lifetimes. Do not delete the entire `target/` tree or run a global Docker prune
to clean one unit. The root workspace and AndroidPort remain separate.

## Bounded retention policy

Historical acceptance does not require keeping every generated dataset forever.
Keep the current compatible dataset per ROM/profile and anything actually needed
by active or explicitly planned work; retain an older generation only for a
specific documented need. Automatic coverage retains its existing two-generation
limit. Check current work and consumers before selecting a prior dataset, and
record its ROM identity, revision, replacement and completed result. A retained
newer dataset is not byte-identical to an old revision: future checks requiring
the exact old generation must regenerate it or use an existing archived copy.

Never discard original ROMs, BIOS, personal saves, checkpoints or recordings,
or the private debug export, as a consequence of cache cleanup. Preserve live,
failed or interrupted scopes until their diagnosis is complete, and retain
small results, manifest identities, source preimages and logs as evidence. Old, completed
Wine prefixes, rebuildable test binaries and caches may be retired after checking
for live users, unique data and concrete dependencies; do not assume a prefix is
empty or unused merely because it is old.

For a reviewed historical dataset or other category outside the existing helper,
first create a bounded explicit preview of exact paths, sizes, identity and
retained replacements. Exclude tracked content, symlinked roots and private
inputs; never follow links. The existing helpers refuse linked trees. Hold the
applicable activity lease and recheck the preview immediately before each
removal. Record successes and failures in a local receipt. Never silently delete
an explicit external dataset or use a global clean as a substitute. Cleanup
receipts describe missing local payloads; they do not reopen completed checks.

## Default lifecycle for new validation

Use `scripts/validation_unit.py` for new local build/test units instead of
writing another per-revision runner with permanent output directories. It
creates `target/validation/units/<name>-<unique>/work`, runs one foreground
command there and automatically retires that scratch after successful checks.
The command must finish all its children and write an explicit `status: PASS`
JSON for every selected `--result`. A harness may run several related phases
and assert their results before writing that final JSON.

Keep only these durable outputs from a successful unit:

- The command log, PASS results, source/input identities and cleanup receipt.
  The runner retains these outside `work`; manifests and result/summary JSON
  are copied to `evidence/` automatically. Select additional small evidence
  with `--evidence`; each evidence file is limited to 16 MiB.
- One current reusable dataset per ROM/profile, plus a previous dataset only
  while a named comparison actually needs it. Test extraction, mutation,
  repair and relocation copies belong in `work`. A build-only/source-only
  change does not require extracting another dataset. Reuse authenticated
  read-only inputs for unchanged extraction revisions.
- Canonical Cargo outputs and shared compiler caches. Resolve test binaries
  from Cargo's artifact output; avoid copying another executable into every
  unit unless preserving that exact binary is an acceptance requirement.
- One runner-owned Wine prefix, leased across all participating units. Per-test
  saves/configuration still need explicit isolation. Prefix contents are not
  a per-version snapshot. Use a separate prefix only for a named clean-profile
  or incompatible-configuration test, with a documented lifetime.

Example for an existing ignored EUR runtime test (replace the binary, test name
and its output environment variable with the selected test's actual values):

```sh
python3 scripts/validation_unit.py --name rom-data-family-runtime --wine \
  --protect path/to/current-dataset --protect path/to/private-eur.gba \
  --result runtime/result.json \
  --env 'FFTA_SELECTED_RUNTIME_ROOT={wine_work}\runtime' \
  -- 'Z:\path\to\current-test.exe' 'rom_data::family::selected_test' \
  --exact --ignored --nocapture --test-threads=1
```

Paths in `--result` and `--evidence` are relative to `work`. Command arguments
and `--env NAME=value` accept `{repo}`, `{unit}`, `{work}` and their Wine
counterparts `{wine_repo}`, `{wine_unit}`, `{wine_work}`. The child also receives
`FFTA_VALIDATION_WORK` and `FFTA_VALIDATION_UNIT`. Inherited FFTA/RECOMP/GBA
overrides are cleared; pass every required test setting explicitly. Protect
the actual ROM, dataset, state, recording and other read-only inputs with
repeatable `--protect` options. Complete file sets, identities, hashes and
modification times are checked after the command, before retirement.
Read-only input guards permit hard-linked files, as used by canonical Cargo
executables; they never unlink those inputs. Scratch and lease paths retain
their strict shared-file checks. Symlinks and junctions remain refused.

With `--wine`, the runner owns `target/validation/wine-prefix`, refuses adoption
of an unmarked legacy prefix and holds its lease through `wineserver -w`. Use
the matching `--wineserver /path/to/wineserver` if it is absent from `PATH`.
All consumers of this prefix must use the runner; never launch a separate
uncoordinated Wine process against it. Existing personal/shared prefixes are
neither adopted nor removed. On native Windows omit `--wine` and use native
path placeholders. Native Linux validation remains a separate gate.

Nonzero exit, missing/negative results, changed protected inputs, linked or
shared scratch paths, private input extensions and tracked scratch content
prevent retirement. A timeout or interruption retains protective leases too,
because descendants may still be alive; inspect the recorded owner and process
before manual recovery. There is no automatic stale-lease reclamation.
`--keep-work 'named current-input or diagnostic reason'` is an explicit retention
exception; record its consumer and retire it after replacement. It must not
become a default flag on each revision. Source preimages and personal inputs
belong outside disposable scratch. Failures keep their work and evidence.

This lifecycle applies to new units, not a background sweep of historical
artifacts. Existing completed scopes still use the reviewed cleanup procedures
below; failed/interrupted work is never selected merely to meet a disk quota.

### Wine test prefixes

Wine prefixes are generated environments, not evidence that their entire tree
must be retained forever. A completed test's isolated prefix is a candidate only
after its results are recorded and its actual consumers have finished. Preserve
shared prefixes used by current builds or tests, and check the test scripts and
current handoff rather than inferring ownership from the directory name.

Inventory without following links. Check the user directories and Wine registry
for configuration or unique game data, not only known save-file extensions;
preserve or relocate data that is not safely reproducible. Wine commonly creates
`dosdevices/z:` pointing to `/` and user-folder links to real personal folders:
never traverse these links or touch their destinations. The existing cleanup
tools refuse linked trees; do not bypass that protection or delete part of a
prefix while leaving an apparently usable, broken prefix behind. A future prefix
cleanup tool must inventory and recheck the links themselves, remove only the
links and the selected prefix tree, and prove that deletion cannot follow them.

Check live users in the host process namespace or through a coordination
mechanism shared by every consumer. An empty `ps` result inside an isolated PID
namespace, an absent lease on a legacy prefix, or an old timestamp is not proof
that the prefix is idle. If that evidence is unavailable, leave the prefix in
place and record the limitation. Never remove another process's live prefix.

The local read-only inventory `target/local-wine-prefix-audit-20261031/` found
21 prefixes totalling up to about 24.9 GiB of allocated file blocks. This is
an upper bound because hard links may be counted more than once, not an estimate
of guaranteed savings. Most files are Wine's own Windows libraries, but absence
of recognised save extensions is not proof that a prefix is disposable.

## Preserve local files when aligning branches

Aligning branches does not select a validation unit for cleanup. Preserve
ignored and untracked local files, including files newly covered by the target
branch's `.gitignore`. Inventory them before switching, check incoming tracked
paths for collisions, and use `git switch --no-overwrite-ignore` so Git refuses
to overwrite an ignored file. Do not use `git clean -fdx` or `git clean -fdX`.

When replacing a divergent local branch tip, retain a recovery reference first.
If that branch is not checked out in any worktree, update only its reference
with an expected-old-value guard; this leaves the checkout and local files in
place. Create the recovery reference and update the branch atomically.
Let checkout remove only obsolete tracked paths absent from the target tree;
verify local files remain afterwards. Delete disposable local artifacts only
through a separately selected, successful unit and the preview/receipt rules
below. An ignored file is not evidence that it is no longer needed.

## Session manager and activity coordination

[DBG-18](../debug_artifacts.md) adds bounded preview/apply with keep-selection,
HTML/JSON estimates and per-operation receipts. It preserves every session's
inputs/reports/errors and selects only generated native cache libraries; explicit
completed ROM units reuse the disposable-copy rules below. New sessions and
Python tools share directory-v1 activity leases across Wine/native/container
processes. Active/interrupted/legacy/unattested/failed scopes remain protected.
Automatic coverage preparation/read/pruning holds one cache lease and retains
receipts in `.retention/`;64receipts stops further pruning until archived.
Old uncoordinated tools/readers must not run against an evictable shared cache.

The `complete` CLI below now takes the same unit lease; duplicate/concurrent
cleanup refuses before removal. No automatic stale-lease reclamation, global
prune or deletion of original data. Portable Python/PowerShell/Docker
entry points, logs, retained payload availability and exact limits are in the
linked guide. Root/staged coverage Docker contexts include every Python import.

## Complete a validation unit

Managed validation deliberately creates several large dataset copies to test
failed publication, repair and relocation. After successful managed/native tests,
record their results and run preservation checks, then retire the disposable
copies. Keep the accepted `offline dataset` while it is needed for CLI/scene
work or an active investigation; review superseded datasets under the bounded
retention policy above. Failed or running tests retain all diagnostic data.

Preview from the repository root (Python and Git required):

```sh
python3 scripts/local_artifacts.py complete target/rom-data-example-runtime
```

After reviewing the paths, apply and keep a receipt:

```sh
python3 scripts/local_artifacts.py complete target/rom-data-example-runtime \
  --apply --report target/rom-data-example-runtime/artifact-cleanup.json
```

The helper removes only `windows-managed-europe/interrupted` and
`windows-managed-europe/moved resources`. It keeps the accepted offline dataset,
source/model preimages, screenshots, logs, results, canonical executable and
shared Wine/Cargo caches. It refuses tracked content, links, private inputs and
unexpected file types. It rechecks candidate sizes, times and file identities
before removal. Symlinked validation roots, including the current `/tmp` backed
slot-job unit, are skipped.

`--retire-executables` additionally selects known old unit-local `recomp`,
`recomp-tests-*`, `recomp-rom-data` and `gba-core-tests-*` executables. Use this
only for superseded units after checking that later work uses a retained helper.
It does not discover or remove arbitrary binaries, release packages or Wine
prefixes. Every command previews unless `--apply` is present.

Preservation maps should contain inputs actually used by the new unit. Old
expected-output maps remain historical evidence; do not require retired mutable
test copies to remain on disk forever. A cleanup receipt records local artifact
availability and does not invalidate a completed acceptance. Do not recreate old
fixtures or replay closed tests just because these payloads were retired.

## Automatic coverage cache retention

Both ROM coverage Docker launchers pass `--cache-keep 2`: after successful HTML
publication, retain the current dataset plus the newest other generation for
that ROM. Version ordering is numeric. Cache deletion is restricted to the
automatic cache's ROM/revision layout and identity-matching manifest. Unknown,
linked or private content is refused; unrelated ROM directories remain intact.
Cache-cleanup errors are reported without failing an already published report.
Never share a cache with a concurrently running historical coverage analysis
when enabling eviction; use an explicit dataset/output outside that cache.

Explicit `--dataset` inputs are untouched. Direct Python invocation keeps
retention disabled by default (`--cache-keep 0`), including read-only cache
validation; developers may opt into a chosen positive count. Docker continues
to bundle Python, so the host needs no Python for coverage generation.

## 26 October cleanup

The reviewed local sweep retired 80 disposable test trees and 160 superseded
unit-local executables: 7,138 files / about 50.3 GB allocated. All 3,186 private
ROM/save/recording hashes and 53,887 retained-file sizes/times were unchanged.
The newest UI-property/FFFF/slot-job/post-slot units were preserved, as were
accepted offline datasets, actual CLI datasets, Wine prefixes, build caches,
source preimages and proof logs/results. Source files were unchanged by the
deletion. Local manifests are under `target/local-artifact-cleanup-2026-10-26/`.

The lifetime of build caches is separate from these per-test payloads:
there is no scheduled background deletion, Cargo clean or global Docker prune.

## 3 November cleanup

The reviewed 36-path plan retired 23 superseded EUR datasets (e94–e105) and
13 isolated Wine prefixes from completed debug-tool tests: 28,723,036,160 bytes
of allocated file blocks (26.75 GiB). The current e107 datasets, USA datasets,
shared environments, active/failed/interrupted scopes, logs, results and source
preimages were retained. The 4,931 protected ROM/save/checkpoint/recording files
kept their hashes; 295,715 retained files kept their identities, sizes and times.
The cleanup also retained 101 manifest and Wine configuration preimages.

Prefix payloads matched a retained generated reference apart from the backed-up
configuration. Links were inventoried without traversal; a deletion probe
confirmed that an external link destination remained intact. Unit leases,
fresh host process checks and exact inventory rechecks guarded each removal.
No relevant active process was observed; four process-detail reads were denied,
as recorded in the receipt. No game tests, builds or performance measurements
were repeated for this cleanup.

The exact plan, preservation inventories, preimages and `PASS` receipt remain
under `target/local-artifact-cleanup-20261103/`. Retired payloads are no longer
available locally; historical acceptance remains valid within its original
scope. This sweep does not close the older interrupted cleanup task.
