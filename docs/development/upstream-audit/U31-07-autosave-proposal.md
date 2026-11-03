# U31-07: Fork-native machine-autosave proposal

Status: **desktop product settings approved; implementation pending**. This is a desktop
design candidate derived from the [source audit](U31-07-machine-autosave.md),
not a claim of implemented behavior or validation. Android has a separate
frontend and needs its own lifecycle and storage analysis. The existing
cartridge save, manual states, backups and debug-session rules remain the
baseline. No upstream bundle, legacy `.sav` migration or card UI is imported.

## Approved desktop settings and open details

On 2026-11-02 the user accepted these desktop settings:

- Autosave is opt-in and disabled by default.
- The configurable interval defaults to five minutes of play. Capture occurs at
  the first safe opportunity after the interval, not necessarily at the exact
  deadline.
- Retain the three most recent valid recovery copies. Retire the oldest only
  after the new copy has been written and verified.
- Offer available recovery copies for explicit selection; never restore one
  automatically. Recovery uses a separate cartridge save and preserves the
  current save.
- A normal exit does not delete recovery copies; users can explicitly delete
  them in settings.

The safe-point predicate, definition of play time, configurable interval bounds,
exact recovery presentation, storage identity, filesystem behavior and other
technical details remain open. Upstream's five-second timer and eligibility
conditions are not adopted. Android requires a separate decision and design.
These settings are not evidence of an implementation or validated behavior.

## Identity, capture and scheduling

Use a dedicated recovery namespace separate from manual slots and cartridge
backups. A candidate stores the fork's raw `FFTA-ST\0` state with a versioned
manifest containing its full digest and size, ROM and BIOS identities, state
version, and a cartridge-family identity and observed cartridge digest. Binding
to the active cartridge identity must distinguish different saves of the same
ROM; ROM identity alone is insufficient. The manifest must not contain save
bytes, secrets or absolute paths. Exact naming and a portable identity that
survives appropriate moves without conflating independent saves are unresolved;
raw state bytes alone do not establish the active cartridge path. Refuse an
ambiguous mapping rather than guessing. Debug copies use only their own isolated
save root; replay and benchmark workers must not publish into a personal root.

Acquire an OS-backed exclusive writer lock for a namespace before publishing and
hold it for the writer's lifetime. An unavailable lock, unsupported locking or
unverified filesystem semantics disables writes with a visible explanation;
stale-looking lockfile names or PIDs alone never authorize breaking a lock.
Release only locks owned by this process. Concurrent recovery and publication
also need coordination so a reader cannot lose the generation it is validating.

At an approved completed-guest-frame boundary, serialize on the machine-owning
thread, then pass immutable bounded bytes to a background writer. This must not
advance guest time or serialize concurrently with mutation. The exact safe
predicate and interaction with pause, rendering, fast-forward and game scenes
remain to be established with evidence. Suppress capture in benchmark/replay
workers and after a cheat one-shot until provenance is supported. Define and
test the recording policy rather than bypassing its existing guards.

Allow at most one publication in flight; either skip or coalesce later eligible
captures into one bounded latest pending item. The choice and memory/write
budget remain open. A sequence/epoch bound to the namespace prevents an old
worker from publishing after a ROM, cartridge save or session switch. Drain or
cancel safely before releasing ownership, and never let a stale completion
replace a newer generation. Failure to serialize, queue, write or publish is
visible to the user and leaves the last valid recovery intact. The timer and
error policy must avoid an unbounded retry or write loop. A normal exit does
not silently erase the last recovery; retention and explicit deletion need a
separate decision.

## Candidate publication and recovery

Use immutable uniquely named generations and a small versioned manifest or
pointer, rather than replacing the only valid state in place. One candidate
publication sequence is:

1. Create a unique temporary file in the destination directory, write the
   bounded snapshot and metadata, flush and sync it; verify any required
   checksums before publication.
2. Atomically publish the new generation without clobbering a prior generation.
   Write and sync a new pointer, then atomically replace the old pointer. Sync
   the parent directory where supported and verify the actual platform
   guarantees. Keep the previous known-good generation until the new one and
   its pointer have passed the required validation and retention policy.
3. On startup or an explicit request, inspect the pointer and available prior
   candidates under coordination. Validate limits, complete content and digest,
   format/version, ROM/BIOS and cartridge identity, then load into a scratch
   machine before any live mutation. If the newest candidate is invalid, offer
   a validated older generation; do not trust a filename or timestamp alone.

An interruption before generation publication leaves the previous pointer
usable; one after generation publication but before pointer replacement leaves
an orphan and the prior pointer; one after pointer replacement leaves either a
validated new generation or a recoverable previous one, subject to verified
filesystem semantics. Orphans are never treated as valid without validation.
Disk-full, sync, rename and directory-sync errors must be reported without
claiming successful durability. Process-kill tests can exercise interruption
boundaries but do not prove survival of sudden power loss. Windows filesystem
semantics, directory sync support and network/removable storage behavior need
explicit investigation; fail closed where the assumptions are unsupported.

## Cartridge chronology and live restore

A machine snapshot includes cartridge memory. The current frontend can flush
that memory to the active `.sav` every 300 frames and on clean shutdown, so
merely validating a snapshot or suppressing the periodic write is insufficient.
The proposal must gate **both** paths before committing a recovered machine,
through every exit and error path. Compare the active save identity and current
disk content with the capture context, and recheck at commit; a matching digest
alone does not prove that overwriting an earlier timeline is safe.

Recovery is to use a new isolated cartridge save, selected explicitly by the
user, with subsequent writes confined there. Writing recovery into the existing
save is outside the approved scope. The exact lock and transaction design is
still open; refuse recovery if preservation cannot be established. Never replace
or rename the original cartridge save as a side effect of discovering recovery.

Before live commit, validate in scratch and preserve a pre-recovery state. Use
the existing `after_state_restore` frontend boundary and core state loader,
including native decode-cache handling, only after all preconditions succeed.
Recording must be stopped or recovery refused with an explicit retry, consistent
with manual loads; replay, active probes and isolated sessions require explicit
guards. Recovery must not silently consume or overwrite the single existing
Undo entry. Either extend Undo with a recovery-aware operation that preserves
its documented semantics, or refuse recovery while an existing Undo is live;
the choice and user interaction remain open. A failed or cancelled recovery
must leave the live machine, active save, manual slots, Undo and backup history
unchanged. The proposed save gate and recovery transaction do not exist yet.

## Implementation increments and acceptance

1. Resolve the remaining identity, timing, filesystem and chronology details;
   specify observable UI and failure behavior consistent with the approved settings.
2. Implement the isolated storage and lock protocol with synthetic bytes and
   injected failures, independently of capture. Test competing processes and
   each publication boundary on supported filesystems.
3. Add bounded capture/scheduling and session-switch coordination. Verify
   recording, replay, debug-copy, cheat and benchmark restrictions and that
   guest state/timing are not advanced by capture.
4. Add explicit, transactional recovery and the cartridge-write gate, including
   clean-exit behavior, Undo and frontend/native restore boundaries. Test
   cancellation and errors before and after each relevant commit point.
5. Validate an isolated desktop scenario on Windows, including a real restart,
   interrupted publication, recovery and preservation of a distinct current
   cartridge save. Report Wine, native Windows and Android evidence separately.

Tests must cover corrupt, truncated, oversized and incompatible files; wrong
ROM/BIOS/cartridge identities; missing or corrupt pointers; older-generation
fallback; disk full and write/sync/rename failures; multiple instances; pending
writers during save switches; active recording and cheat guards; native cache
and frontend reset; and preservation of manual slots, Undo, cartridge saves
and backups. Test on synthetic/isolated data, retain diagnostics for failures,
and never claim a power-loss guarantee from process termination alone. No tests,
application behavior or platform acceptance are claimed by this proposal.

## Android-specific analysis boundary

The [Android lifecycle comparison](U31-07-machine-autosave.md#follow-up-android-lifecycle-and-cartridge-storage)
shows that the fork's existing `gbaSave` writes cartridge data while the pinned
upstream uses that entry point for asynchronous machine-state management.
Android therefore needs a separate contract preserving cartridge writes,
explicit completion/error reporting and save/ROM-switch behavior. A candidate
must address background, process termination, worker ownership, benchmark
isolation and input/audio/render/native reset; a desktop design or the upstream
five-second eligibility timer does not settle those decisions. No Android
runtime, UX or storage policy is approved by this analysis.
