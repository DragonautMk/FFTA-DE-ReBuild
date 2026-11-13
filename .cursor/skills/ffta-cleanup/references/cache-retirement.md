# Retire rebuildable caches

Read this only when significant caches remain after ordinary compression and
legacy scratch cleanup. A missing cache is already handled; do not rebuild it
to validate removal. This is an exact-inventory procedure, not a blanket sweep.

Eligible candidates include compiler intermediates from completed isolated
builds and Cargo `incremental/` directories with no active compiler consumer.
Do not classify a whole build directory as disposable if it contains the only
retained executable, generated source, private input or diagnostic evidence.
Canonical `deps/`, `build/`, executables, shared Wine and archive contents are
outside the automatic cache selection; review a specific additional need
separately rather than deleting these roots.

1. Select exact repository-relative directories/files under `target`, with a
   reason, completed-unit evidence where applicable, and any retained replacement
   executable. Use `scripts/local_artifacts.py` when its narrow selection fits.
   Otherwise keep one bounded driver and an exact JSON plan/receipt under
   `target/local-artifact-cleanup-<unique-run>/`, outside selected payloads.
2. Inspect the relevant `scripts/artifact_coordinator.py` lease API and
   `scripts/cold_storage.py` host activity checker. Require host PID visibility,
   check handles/owners and lease the selected unit for the entire mutation.
   For canonical Cargo `incremental/`, also hold a nonblocking exclusive
   `fcntl.flock` on the **existing** parent profile `.cargo-lock` throughout;
   never replace or unlink that lock. If unavailable, skip that cache. Check
   for Cargo/rustc/build consumers of the enclosing target/profile, not only
   files open within `incremental/`. Cooperative locks do not cover every tool.
3. Inventory with `lstat`, recording device/inode, type, mode, owner, link count,
   size, nanosecond times and allocated blocks. Refuse tracked/private files,
   unexpected ownership, special files, symlink/reparse roots or descendants
   and nested activity markers. Bound every selected path to its reviewed root.
   Record identities of retained executables/inputs outside the selection.
4. Immediately before removal, recheck the inventory, unchanged completion
   evidence, held locks and fresh host activity. Busy, changed or unreadable
   critical ownership/activity evidence means skip. Retain any host inspection
   limitations in the receipt; never report global inactivity from partial data.
5. Remove only unchanged planned entries. For a whole disposable directory,
   rename it to a unique retirement sibling under the same parent while holding
   locks, then delete only that owned tree without following links. Never pick
   up a pre-existing retirement directory from another run. On interruption,
   preserve the receipt and inspect ownership before any recovery.
6. Verify retained identities are unchanged and record removed entries/errors.
   Count each inode once; blocks with hard links remaining outside the selection
   are not reclaimed. Removing a cache alias must only unlink it, never truncate
   or overwrite shared contents. Do not count sparse logical bytes as disk gain.

Rebuilding may take longer next time; no build is required now. If safe cache
retirement cannot be established, finish other eligible work and report the
specific remaining cache and reason. Never stop or signal other sessions to
make a cleanup candidate eligible.
