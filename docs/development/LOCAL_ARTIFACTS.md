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
  An explicitly needed isolated build belongs in `work`; select only the
  executable needed by the next consumer with `--keep-path`, so its compiler
  cache does not become permanent. A build and its immediate test can share
  one foreground harness and retire together after PASS.
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
executables; they never unlink those inputs. Owned scratch accepts Cargo's
internal hard links only when the inventory contains every name for each inode
(matching its link count). A link outside scratch still blocks retirement.
The complete inventory, including link counts, is rechecked before removal.
Symlinks, junctions and nested activity leases remain refused.

With `--wine`, the runner owns `target/validation/wine-prefix`, refuses adoption
of an unmarked legacy prefix and holds its lease through `wineserver -w`. Use
the matching `--wineserver /path/to/wineserver` if it is absent from `PATH`.
For a native Python/shell GUI driver that launches Wine children, use
`--wine-host` instead of `--wine`. The runner starts that driver directly in
the foreground, with native `FFTA_VALIDATION_WORK`/`FFTA_VALIDATION_UNIT` paths
and the same managed `WINEPREFIX`/`WINESERVER` environment. The driver must pass
that environment to every Wine child, keep all phases/restarts inside its
foreground lifetime and finish its children before returning. The runner keeps
the prefix lease through the final `wineserver -w`, including nonzero driver
exit; a driver timeout or failed server drain retains diagnostic leases.
Receipts record the mode, lease owner and whether server drain completed.
`--wine` still prepends Wine and supplies Windows paths; the modes are mutually
exclusive. Wine path placeholders remain available in either mode.

For example, with a reviewed native driver and its actual protected inputs:

```sh
python3 scripts/validation_unit.py --name wine-gui-restart --wine-host \
  --protect path/to/gui_driver.py --protect path/to/runtime.exe \
  --protect path/to/input-state --result result.json --reserve-gib 1 \
  -- python3 '{repo}/path/to/gui_driver.py'
```

All consumers of this prefix must use the runner; never launch a separate
uncoordinated Wine process against it or keep a bridge alive while an external
driver uses its lease. A held lease refuses a new run; inspect its recorded
owner instead of removing it or stopping unrelated processes. Existing
personal/shared prefixes are neither adopted nor removed. On native Windows
omit both Wine modes and use native path placeholders. Native Linux validation
remains a separate gate.

Nonzero exit, missing/negative results, changed protected inputs, external
shared scratch paths, unselected private input extensions and tracked scratch content
prevent retirement. A timeout or interruption retains protective leases too,
because descendants may still be alive; inspect the recorded owner and process
before manual recovery. There is no automatic stale-lease reclamation.
Failures keep their work and evidence.

### Retain selected outputs instead of the whole unit

Use repeatable `--keep-path REL` with `--keep-reason 'consumer and retirement
condition'` to keep an exact file or subtree after PASS. Selected outputs preserve their original logical paths; unselected siblings
are removed. Eligible retained outputs are then compressed by default. The
runner restores declared `--protect` inputs to those paths before use. Add
`--keep-hot REASON` for the current dataset or other frequently used output. For example, append these options before `--` when a
harness creates the current dataset and a pinned executable:

```sh
--keep-path resources --keep-path cargo-target/release/ffta-cli.exe \
--keep-reason 'Current EUR dataset and CLI until the next accepted generation'
```

Select the actual executable path reported by the build. Keep each executable's
required runtime files too. The cleanup receipt retains the existing `kept`
token and adds `retention_mode: selected`, `kept_paths` and unique-inode logical
byte totals before/after retention; these are not allocated disk-block savings.
Missing selections fail without cleanup. A generated checkpoint can be selected
explicitly, but any private file outside the selection blocks all retirement.
Original personal inputs and source preimages still belong outside scratch.

`--keep-work 'consumer and retirement condition'` remains a whole-unit diagnostic
exception and cannot be combined with selective retention. It must not become a
default flag on each revision. Neither form automatically retires an older unit;
review its dependents and close the retention exception after replacement.

### Check space before starting

The runner checks free space before input hashing/unit creation and again just
before launching the child. Defaults require **6 GiB free**: a **5 GiB floor**
plus **1 GiB expected peak growth**. Set `--reserve-gib N` for the actual new
build outputs, dataset copies, logs and evidence expected at peak. Reusing a
canonical build or current dataset can make that budget much smaller than a
fresh isolated build. A refusal starts no child and deletes nothing.

`--min-free-gib N` adjusts the floor. Reducing either default requires a concrete
`--space-exception REASON`, recorded in a successful launch's receipt with the
observed free space and requested budget. Use this only for a measured bounded
check, not as the usual response to a disk-full condition. First review retained
completed units and dependencies, then retire eligible artifacts separately.

This is an admission check, **not a reservation or filesystem quota**. Concurrent
processes or growth beyond the estimate can still fill the disk. The runner does
not kill processes, silently discard failures or sweep historical work to meet
the budget. A hard cap would require a separately managed filesystem/quota and
coordination with all writers, including Cargo and Wine outside unit scratch.

### Storage assessment and follow-up

The read-only 7 November 2026 inventory counted about **165.5 GiB** of allocated
regular-file blocks in `target/`, deduplicating hard links and never following
symlinks. Approximate categories were 39.7 GiB standalone binaries/native caches,
33.3 GiB Wine environments, 25.1 GiB dataset trees, 23.6 GiB isolated Cargo caches,
13.8 GiB canonical Cargo outputs, and 30.0 GiB other data/evidence/compiler files
and private inputs. Categories describe storage, not cleanup eligibility; private
files inside a classified tree inherit that tree's category. Directory metadata,
filesystem reflink sharing and concurrent mutations are not resolved by this scan.

Among 1,196 runner receipts, 354 successful units kept their whole work and 170
failed units retained diagnostics, accounting for about 17.5 GiB and 15.9 GiB of
work respectively. Two receipts still said running; no host inactivity was
inferred. At least three large Cargo units had child exit zero but failed the
old blanket hard-link guard. The revised runner prevents that specific failure
for newly owned scratch; it does not rewrite those historical outcomes.

Next historical cleanup should review superseded successful datasets and pinned
binaries against current consumers, then isolate evidence needed to close failed
units. Completed private Wine prefixes require the separate ownership/process
review below. Preserve current E132 and the named E131 comparison, personal data
and active/suspended work. No amount above is a promise of safe reclaimable space.
Evidence and limitations: [workflow change record](../changes/2026-11-07-bound-validation-storage.json).

This lifecycle applies to new units, not a background sweep of historical
artifacts. Existing completed scopes still use the reviewed cleanup procedures
below; failed/interrupted work is never selected merely to meet a disk quota.

### Verified cold archives and on-demand restoration

Eligible retained outputs from successful new units now become `.tar.zst`
archives under `target/cold-storage/` by default. Install the **zstd CLI** on the
host; the implementation uses level 3 with one worker. `--keep-hot REASON` keeps
current or frequently used outputs available directly. Canonical Cargo output
roots and the shared validation Wine prefix cannot be archived by this tool.
Private input extensions, foreign ownership, external hard links, privileged/ACL
attributes, special files and live nested leases are refused; skipped retained
outputs remain available with a reason in the validation receipt.

The store records each original path, file hashes, modes, owner/group IDs,
nanosecond modification times, internal hard links, symbolic-link targets and
supported `user.*` extended attributes (including Wine DOS attributes). Small
manifests and result/summary files are copied to readable `evidence/` beside the
archive. The existing `cleanup: kept` receipt token means the payload is retained;
`retention_storage` distinguishes cold, hot and mixed storage. Historical result
receipts are never rewritten to pretend a test was rerun.

The validation runner resolves archived **declared `--protect` inputs** before
hashing or launching the test. It restores the original paths under an exclusive
store lease, checks space for their uncompressed size plus the test budget,
and retires unchanged restored copies after PASS. A failure keeps the restored
diagnostics; a timeout keeps protective leases because descendants may still
exist. All consumers of an evictable archived path must coordinate through the
runner or the standalone wrapper. The exclusive store lease serializes archive
consumers and operations; it is not a filesystem quota or a background service.

For legacy commands and read-only inspection:

```sh
python3 scripts/cold_storage.py list
python3 scripts/cold_storage.py run --input target/completed-check/resources -- COMMAND
python3 scripts/cold_storage.py restore target/completed-check/resources
python3 scripts/cold_storage.py verify target/completed-check/resources
python3 scripts/cold_storage.py retire target/completed-check/resources
```

`run` restores its selected inputs for a foreground command and retires unchanged
copies on success. Manual `restore` deliberately leaves the files available.
Modified copies are preserved and cannot silently replace their original archive;
retain them as new artifacts before retiring the old generation. Validation's
ordinary `--protect` guard still refuses symlinked trees: use the standalone
wrapper or explicit restoration for historical Wine trees containing links.
Archive extraction itself validates links without following their destinations.

For automatic historical adoption, run one batch command:

```sh
python3 scripts/cold_storage.py batch --apply
```

With local Linux Docker Engine, Python/Git/Zstandard can run in a dedicated
container instead:

```sh
bash scripts/linux/cold_storage_docker.sh batch --apply
```

The image is built on first use and reused until its packaged sources change.
The launcher preserves host paths and UID/GID, mounts only `target` writable,
and validates host process visibility before opening the store. It shares the
native catalog and leases. Docker Desktop, remote and rootless daemons are not
supported for this host-aware operation. See the
[Docker guide](../docker.md#compress-local-validation-artifacts-linux) for
mounts, AppArmor behavior, platform limits and other CLI commands.

The batch scans completed managed work and legacy scopes with recognized
successful result receipts. It also selects idle historical legacy artifacts
without recognized success receipts: every selected member must have unchanged
mtime and ctime for at least 24 hours, and host consumer checks must pass.
This is reversible storage retention, not a declaration of test completion.
Missing or malformed success receipts no longer exclude an entire historical
scope; explicit failures, unfinished states, leases and protected scopes still do.
Outcome metadata stays readable and hash-bound, and is rechecked before retirement.
Use `--completed-only` to retain the former receipt-only selection policy.

The scanner prints progress and updates its receipt approximately every five
seconds while traversing paths. Catalog membership uses indexed lookups to
avoid repeatedly walking the whole catalog for every file. It selects eligible
files and directories of at least
1 MiB, and compresses them sequentially through the same verified store. It
does not launch tests, builds, gameplay or benchmarks. Omit `--apply` for a
preview; `--min-mib N` changes the minimum and repeatable `--exclude 'target/GLOB'`
adds paths to preserve. Shell globs should be quoted.

Each invocation writes a separate `target/cold-storage-batches/<run>/batch.json`
with candidates, exclusions, completed archives, errors and reclaimed file
blocks. The receipt is updated after each archive. A subsequent invocation
skips cataloged paths, including manually restored copies, and discovers the
remaining work. Interrupted publications stop the batch for explicit recovery;
busy or changed sources remain available. Current E132/E131, current packaged
application, canonical/shared outputs, explicit `--keep-hot` work, private
inputs, source preimages and known suspended scopes remain excluded. Conflicting
failure receipts require separate review; unrecognized completion formats may
use the idle-history policy without changing their original outcome.
The current exceptions are maintained in `scripts/cold_storage_batch.py`.

Legacy successful exit-code lists and Windows result records are recognized
without modifying historical receipts. Their hashes and the source inventory
are bound to the batch review; original results and host activity are rechecked
before retirement. No elapsed-time or file-age heuristic establishes completion.
Other historical paths can still use an explicit review:

```sh
python3 scripts/cold_storage.py plan target/completed-check/resources \
  --scope target/completed-check --completion target/completed-check/result.json \
  --output target/completed-check/cold-review.json
python3 scripts/cold_storage.py pack target/completed-check/resources \
  --review target/completed-check/cold-review.json \
  --reason 'Completed comparison retained for occasional reproduction'
```

A review binds the exact inventory to completion evidence outside the payload.
`pack` rechecks both, leases the parent activity scope, and checks host-visible
process commands, environments, working directories, open files and mappings
before compression and before retirement. It refuses a sandbox-only PID view;
unreadable process details remain recorded limitations, not proof of global
inactivity. Current, failed, live and suspended scopes must be excluded by the
reviewer. A later positive report for another scope does not close an earlier
failed test or establish that its prefix is disposable.

Publication streams directly to a temporary compressed archive, verifies all
members and hashes, durably publishes the archive/catalog, and rechecks the
source before retiring it. No full intermediate tar is written. Restoration uses
staging and refuses conflicting paths; it never calls `extractall` or follows
archived symlinks. Enough free space for the uncompressed payload is required
before extraction. The catalog must not be moved independently of its payloads.

`recover PATH` handles interrupted retirement/restoration while preserving the
verified archive. Changed retirement residue and conflicting destinations block
recovery. Incomplete `preparing` archives retain their original sources and
require inspection; the tool does not guess whether an unfinished archive is
usable. Never delete a stale store/unit lease without inspecting its recorded
owner and descendants. Ordinary terminal failures release only their own leases.

The host implementation and regression tests are validated on Linux. Archives
with Wine colon-named links or Linux extended attributes are not promised to
restore on native Windows. Compression is an application workflow on this ext4
workspace; ext4 does not provide transparent file compression
([kernel documentation](https://www.kernel.org/doc/html/latest/filesystems/ext4/inodes.html)).
Arbitrary unwrapped commands cannot read a compressed directory by its former
path. Inspect the catalog before regenerating a supposedly missing fixture.

The initial 7 November 2026 feasibility sample contained seven complete files:
three generated JSON files, one Cargo archive, one executable and two Wine DLLs.
Zstandard 1.5.7 reduced 369.85 MiB to 47.30 MiB at level 3 (87.21%) or 43.01 MiB
at level 6 (88.37%). All eight group/level content round trips matched SHA256;
originals were unchanged and temporary archives/extractions were removed. This
nonrandom sample does not estimate whole-target savings. Single local timings
with potentially warm caches were about 0.81/2.06 seconds for compression and
0.54 seconds for extraction, excluding hash checks.
Evidence: `target/validation/units/cold-storage-assessment-_br49tbe/` and the
[assessment record](../changes/2026-11-07-assess-validation-cold-storage.json).
Production tooling and historical rollout have their own validation and receipts.

Routine archival and consultation use catalog lookup, content-integrity checks
and existing completion evidence. They do not require rerunning test suites,
builds, gameplay or compression benchmarks. Regression testing is for changes
to the archive implementation or a specific unresolved fault.

The 7 November rollout archived **382 reviewed paths**: 29.33 GiB of original
regular-file blocks became 7.01 GiB of compressed payloads plus 97.37 MiB of
catalog/evidence metadata. This reclaimed **23,871,709,184 file-block bytes
(22.23 GiB / 23.87 GB)**, before small external review receipts and filesystem
directory metadata. All 6,426 inventoried private-file identities, sizes and
modification times remained unchanged. Current E132/E131 inputs, canonical
builds, shared Wine, the current packaged application and failed/suspended
scopes were retained directly. The observed disk-free figure also reflects
unrelated writers and is not used to attribute savings.

Every archive was verified before its source was retired. Final checks verified
all 382 payload/catalog hashes and actual dataset/Wine restoration at the
original paths, followed by re-retirement; no game was executed. Three host
process-detail reads were inaccessible and remain an explicit limitation.
Reviewed plans, exclusions, source/tool provenance and the application receipt
are under `target/cold-storage-migration-20261107/`. The final managed evidence
is `target/validation/units/cold-storage-final-verification-z8k2a8bt/`.
See the [implementation record](../changes/2026-11-07-enable-validation-cold-storage.json).

The subsequent automatic batch archived **272 additional paths** from 142
completed scopes. It reclaimed **9,378,770,944 file-block bytes (8.73 GiB /
9.38 GB)**: 10.63 GiB of original blocks became 1.75 GiB of payloads plus
146.34 MiB of catalog/evidence metadata. All selected archives completed without
errors; their catalog manifests match, original payload copies are absent, and
the previous 6,426 private-file stat guards remain unchanged. Packing performed
the content-integrity checks; no suite, build, gameplay, benchmark or additional
restore exercise was run. Three inaccessible host process details remain a
visibility limit. Receipts: `target/cold-storage-batches/20261107-113755-51f0fa64/`.
See the [batch automation record](../changes/2026-11-07-automate-remaining-target-compression.json).

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

## 4 November build-cache and dataset cleanup

The completed menu-localization unit's isolated `target/menu-i18n-en-it/build`
Cargo cache was retired after checking its completion result, exact inventory,
unit lease and host process users. The removal reclaimed 3,933,757,440 bytes
of file blocks (3.66 GiB), counting hard-linked files once. All 158 protected
private-input hashes and 1,798 retained-file identities, sizes and modification
times remained unchanged. Logs, screenshots, results, source evidence and the
unit's Wine prefix remain available. Current canonical Cargo outputs, shared
Wine, ROM datasets and live/failed/interrupted work were excluded.

The `PASS` receipt and exact build-only plan are under
`target/local-artifact-cleanup-20261104/build-cache-applied.json` and
`build-cache-plan.json`. Four host process-detail reads were denied; no selected
unit consumer was observed. Other sessions were active, so changes in overall
disk free space cannot all be attributed to this cleanup. No build, game test,
fixture extraction or performance measurement was repeated.

Automatic approval review initially refused the extended command because it
included retained validation datasets under general cleanup authorization.
The user then explicitly authorized the 15 e111–e125 EUR datasets listed in
`target/local-artifact-cleanup-20261104/dataset-retirement-proposal.json`.
After fresh inventory/activity checks and unit leases, these superseded payloads
were retired, reclaiming another 7,947,784,192 file-block bytes (7.40 GiB).
Their source-ROM identity, profile and coverage were checked against the retained
e127 replacement; e126 was also retained for the current investigation. All
158 protected private-input hashes and 393 retained-file identities, sizes and
times were unchanged. Fifteen manifest preimages, unit results and logs remain.

Both cleanup receipts are `PASS`; the dataset receipt is
`target/local-artifact-cleanup-20261104/dataset-applied.json`. Combined retirement
reclaimed 11,881,541,632 file-block bytes (11.07 GiB). Exact historical datasets
would require regeneration for reuse. All failed/interrupted/live units were
excluded. This cleanup does not close the older interrupted hygiene task.

## 5 November compiler-intermediate cleanup

The reviewed plan retired 3,771 `.rlib`, `.rmeta`, `.o`, `.a` and `.d`
compiler intermediates from six successful isolated validation builds, reclaiming
5,599,150,080 file-block bytes (5.21 GiB), counting hard links once and excluding
blocks still linked outside the selected files. Every executable, dynamic
library, source snapshot, result and other retained file remains in place.
Original ROM/BIOS, saves, checkpoints, recordings, private exports, current and
historical datasets, canonical Cargo outputs, Wine prefixes and
failed/interrupted units were excluded from removal. Future compilation may
rebuild dependencies; existing runtime artifacts remain available.

The exact plan, host preflight, protected hashes, retained-file inventory and
`PASS` receipt are under `target/local-artifact-cleanup-20261105/`. Unit leases,
fresh host process scans and exact inventory checks guarded removal. All 250
protected input/export/executable hashes and 13,115 retained file identities,
sizes and modification times remained unchanged. No selected-unit consumer was
observed; three host process-detail reads were denied. Free space increased
from 7,818,182,656 to 13,406,445,568 bytes during the operation; selected
file-block counts identify the cleanup's contribution independently of other
disk activity. No build, gameplay test, fixture extraction or performance
measurement was repeated. The older interrupted hygiene task remains separate.

## 7 November completed Wine-prefix cleanup

Retired seven completed isolated Wine test prefixes from the audit, session,
comparison, workbench and selector units, plus one rebuildable Rust archive from
the old isolated debug-tools Cargo cache. Removed file blocks total
8,987,373,568 bytes (8.37 GiB). The 42 retained Wine configuration copies occupy
23,523,328 file-block bytes, leaving 8,963,850,240 bytes (8.35 GiB) before small
plan, log and receipt metadata. Overall free-space changes also reflect other
disk activity and are not attributed entirely to this cleanup.

Every nonconfiguration prefix file matched the retained generated Wine
reference. Exact inventories included all links; hardened removal unlinked them
without touching their destinations. Unit leases, fresh host process scans and
completion/inventory rechecks guarded the seven removals. No selected consumer
was observed; four host process-detail reads were denied. All 520 protected
hashes and 4,229 retained file identities, sizes and modification times remained
unchanged. Current E132/E131 datasets, shared Wine, source preimages, results,
private saves/checkpoints/recordings and exports remain available.

The disk reached zero writable space before plan creation. A bounded plan and
receipt in `/tmp`, both existing Cargo output locks and a host process check
guarded retirement of one 70,402,048-byte intermediate, allowing normal
directory-lease coordination to resume. Those bootstrap records were then
preserved in the project. No runtime or executable was removed in that step.

Plans, configuration preimages and both `PASS` receipts are under
`target/local-artifact-cleanup-20261107/`. Final validation unit
`target/validation/units/local-space-cleanup-20261107-0dxs5vzo/` passed all
presence checks and 1,749 retained reference hashes, with guarded inputs unchanged
and automatic scratch removal. Its first invocation refused the linked reference
tree before creating a unit; the corrected invocation protected individual
ordinary files. No build, gameplay, extraction or performance test was repeated.
Historical acceptance remains valid; exact former prefixes require restoration
or recreation. The older interrupted hygiene task remains separate.
