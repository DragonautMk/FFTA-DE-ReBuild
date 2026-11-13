---
name: ffta-cleanup
description: Compresses FFTARecomp artifacts into target/cold-storage and removes obsolete generated files. Use when asked to clean the project or target, reclaim disk space, compress test output, or repeat storage maintenance.
---

# Clean project artifacts

Carry out the requested cleanup using the existing storage tools. Keep useful
historical payloads compressed inside `target/cold-storage`; delete disposable
generated copies and caches once their ownership and lack of consumers are
established. A request to create/edit this skill alone does not start cleanup.
Run commands from the repository root. This skill covers the root workspace;
do not extend a sweep into `AndroidPort/`, other worktrees or host caches.

## Select the work cheaply

- Read `AGENTS.md`, the current `docs/development/TASK_HANDOFF.md` including its
  final handover, and `git status --short`. Record the selection checkpoint
  through `scripts/task_handoff.py update` as required by the memory workflow.
- Check free space with `df -B1 target`; take one bounded directory-size summary
  if needed. Consult the cold catalog and existing receipts before descending
  into payloads. Batch independent reads; summarize JSON instead of printing
  thousands of paths or loading historical documentation.
- Derive exclusions from current task references, unit receipts, leases and
  named consumers. Preserve current datasets for each ROM/profile, named
  comparison baselines, live/failed/interrupted/suspended work and source
  preimages. Hardcoded revision exclusions in the scanner can become outdated.
- Protect ROM/BIOS, personal saves, checkpoints, recordings, private exports,
  tracked files, current applications, canonical executables and shared Wine
  prefixes. Untracked or old does not mean disposable. Do not follow symlinks
  into other locations or reclaim another process's lease.

Proceed within the user's cleanup authorization after reviewing the concrete
inventory; do not ask again for each eligible archive or generated scratch
copy. Unclear ownership or dependencies exclude that candidate while other
eligible work continues. Sandbox process listings cannot establish host
inactivity; use the existing host-aware checks and required execution access.

## Compress automatically

Use the Docker wrapper on supported local Linux Docker Engine installations;
otherwise use the native Python CLI with Git, Zstandard and host PID visibility.
The native fallback must satisfy the same activity checks.

```sh
bash scripts/linux/cold_storage_docker.sh batch --apply --completed-only
# Native equivalent:
python3 scripts/cold_storage.py batch --apply --completed-only
```

Choose one backend. Add repeatable `--exclude 'target/EXACT-UNIT*'` arguments
derived above, protecting the scope itself and its descendants. These are
patterns to replace with actual reviewed paths, not a fixed exclusion list.
`--completed-only` is the routine policy: the default idle-history fallback
does not establish completion. Omit `--apply` when only an assessment is requested.

The batch supplies exact inventories, completion hashes, scope/store leases,
host checks and verified publication before removing originals. Reuse these
checks rather than scripting a second archiver. Archives, catalog and readable
evidence remain together under `target/cold-storage`. Do not gzip files in place
or delete originals before verified publication.

Follow the process handle and its `target/cold-storage-batches/.../batch.json`.
A quiet terminal is not proof of failure; keep progress updates brief and do
not start overlapping batches. Stop retries on an interrupted publication,
inspect its state and use explicit recovery only after ownership is established.
Do not silently lower the space floor: an exception needs a recorded reason
and sufficient headroom for the selected archive's worst-case temporary size.

### Finish the remaining candidates

Read the receipt's exclusions/errors and inspect only the remaining large
uncompressed paths. `Selected 0 paths` means no candidates under that policy,
not that every file in `target` is compressed.

An old `keep_hot_reason` may refer to a superseded current dataset. Compare its
ROM identity/profile and revision with the actual retained replacement and
named baseline. If its completed unit has no remaining consumer, archive it
through an explicit review; keep its original result unchanged. Do not infer
supersession from revision numbers or age alone.

For a reviewed completed payload, substitute real paths in this example:

```sh
python3 scripts/cold_storage.py plan target/completed-unit/work/resources \
  --scope target/completed-unit \
  --completion target/completed-unit/unit-result.json \
  --output target/completed-unit/cold-review.json
python3 scripts/cold_storage.py pack target/completed-unit/work/resources \
  --review target/completed-unit/cold-review.json \
  --reason 'Superseded dataset retained for on-demand inspection'
```

Keep completion evidence and the review outside the payload. The same arguments
work through the Docker wrapper. Review larger exclusions before lowering the
1 MiB batch threshold; compress small files only when the aggregate gain merits
catalog overhead. Never recompress existing archives. Cataloged manual restores
need explicit `retire PATH` after checking consumers and unchanged contents.

## Delete files that are no longer useful

For successful legacy `target/rom-data-*` units, verify PASS evidence and fresh
host activity, then use the existing preview and apply commands:

```sh
python3 scripts/local_artifacts.py complete target/rom-data-completed-unit
python3 scripts/local_artifacts.py complete target/rom-data-completed-unit \
  --apply --report target/rom-data-completed-unit/artifact-cleanup.json
```

Multiple reviewed units can be passed together. This removes only the generated
`windows-managed-europe/interrupted` and `moved resources` copies. The helper
checks inventory and takes leases, but a result filename alone does not prove
PASS and the caller must establish host inactivity. Add `--retire-executables`
only after verifying the exact old executables are superseded by retained ones
and are not pinned for comparisons. Keep results, logs and manifest identities.

For substantial remaining rebuildable caches, follow
[the cache retirement procedure](references/cache-retirement.md). Useful old
datasets and evidence go into cold storage rather than being discarded as caches.
Do not run `cargo clean`, delete all of `target`, prune Docker globally, or
remove arbitrary files by extension/age. Skip candidates whose completion,
ownership, replacement or host activity cannot be established.

## Verify and hand off

- Use archive integrity checks and receipts. Do not rerun test suites, gameplay,
  builds, extractions or compression benchmarks for routine maintenance.
- Sum actual reclaimed file blocks from completed operations, accounting for
  compression/metadata overhead and external hard links. Report overall free
  disk space separately; concurrent work can change it. Keep failed/partial
  receipts and host visibility limitations visible.
- Recheck only unresolved candidates or new output; stop when the reviewed
  inventory is handled. State material exclusions rather than claiming all of
  `target` is compressed. Keep the final answer short: reclaimed space, remaining
  free space, preserved large items and receipt locations.
- Close this maintenance selection through the coordinated handoff/archive
  protocol. A sweep does not close unrelated historical hygiene or runtime
  tasks. Use `ffta-project-memory` for meaningful tool/workflow changes; routine
  runs reuse receipts and record their outcome without inventing code changes.

For later consultation, use `python3 scripts/cold_storage.py list`,
`run --input PATH -- COMMAND`, or explicit `restore PATH`, `verify PATH` and
`retire PATH`. Prefer the leased `run` wrapper for temporary inspection; it
retires unchanged restored inputs after success. Modified or failed inputs stay.
See [artifact retention](../../../docs/development/LOCAL_ARTIFACTS.md#verified-cold-archives-and-on-demand-restoration)
for tool semantics and [Docker support](../../../docs/docker.md#compress-local-validation-artifacts-linux)
when the container backend needs configuration.
