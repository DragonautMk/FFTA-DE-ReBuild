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
