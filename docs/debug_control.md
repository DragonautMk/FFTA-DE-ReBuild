<a id="controllo-locale-pausa-e-avanzamento--dbg-1020"></a>

# Local control, pause and stepping — DBG-10/20

In [isolated sessions](debug_sessions.md), **F6** pauses or resumes, **F7**
advances one frame and **F8** ten. The **Debug** menu offers Step 1 frame,
Step 10 frames and Resume. Stepping requires an already paused copy: the request
does not rewind the machine and stops after the requested number of completed
frames. F6/F7/F8 are consumed before text, guest bindings, Fast and shortcut
capture. They are reserved only in the debug copy; autorepeat generates no
additional steps. Menu commands also work with gamepad navigation.

Menu/keyboard steps use **neutral guest input** (`3ff`, all released). Menu
confirmation and held physical keys do not become guest input. Use the CLI to
select an explicit mask. Game clicks are not executed during a step. The normal
menu continues to pause the main game too. The new commands do not control it.

The copy's panel beside the menu shows guest frame, clock, reason
(`PAUSE-COMMAND`, `MENU-PAUSE`, `STEPPING`, `STEP-COMPLETE`, `STATE-RESTORED`)
and remaining frames. A frame is a completed PPU boundary, not an instruction
breakpoint. Backends retain their own return granularity.

<a id="cli-sulla-stessa-finestra"></a>

## CLI on the same window

Use a runtime containing this feature; old copies retain their pinned executable.
Create a new session with that runtime without replacing the executable in an
already authenticated session. It can open directly in pause:

```powershell
.\recomp.exe debug-session open 'target\debug-case' --paused
```

In another terminal:

```powershell
.\recomp.exe debug-control status 'target\debug-case'
.\recomp.exe debug-control pause 'target\debug-case'
.\recomp.exe debug-control step 'target\debug-case' 3
.\recomp.exe debug-control step 'target\debug-case' 1 --keys 3fe
.\recomp.exe debug-control resume 'target\debug-case'
```

`step N` accepts **1–600 frames**. `--keys HEX` accepts **000–3ff**, active-low:
`3fe` holds A, `3ff` releases all. The mask applies to every frame of the
request. `resume` returns to normal execution and current inputs. For a recorded
variable sequence use [recording/replay](debug_recordings.md). An active recording
in the copy can include steps and pauses: host pause time adds no recorded frames.

For a paused EUR interpreter copy, a Wine/Xvfb comparison started three sessions
from the same seed and used the local-control CLI, parameter forms and command
console respectively to inspect 16 ROM bytes and step one neutral frame. The
complete inspection result and the resulting frame, clock, registers and state
hash were identical; inspection itself did not change guest state.
Another isolated Wine/Xvfb check confirmed that zero-length inspection and a
zero-frame step produce the same refusal in the parameter form and command
parser. The interactive console returned the same JSON errors as `debug-command`
for those cases, ambiguous numbers and an unknown command; guest state remained
unchanged throughout.

An isolated EUR Wine/Xvfb session also resumed and paused through local control,
the menu and F6, and the command console. Each resume advanced frames and each
pause returned to a stopped state with no remaining step budget. Frame counts in
free-running checks depend on timing and are not compared for equality.

The workbench parity tests also compare nondefault values and selected boundary
refusals against the console and local-control parsers for step, inspect,
audio, code, gameplay and Recent; graphics compare-last is included in the
nondefault action comparison. These are parser tests, not additional runtime
coverage for every combination of settings.

Each command receives JSON with run/request identity, success/error,
frame/clock/PC, reason, remaining budget, full-state hash and guest/host audio
queue sizes. The hash is computed only on request and command completion,
without saving an additional snapshot. Returned state is at the observed
boundary: after `resume` the game continues. An incompatible request is refused
without being applied and returns a nonzero outcome; during free execution the
game still advances. A partial execution error is not counted as a completed frame.

Only one client can await an operational command. **status**, **content-list**
and **cancel** use a second mailbox with separate lock/response: they remain
available during a step or search without overwriting the pending result.
No automatic retry. F6/Escape and Load/Undo/Restart can still interrupt the
budget; during a separate replay status is readable, but pause/step/resume are refused.
When a replay was started from the menu and no control request is pending,
readonly inspection remains available through the menu, CLI and console. Direct
form validation checks the parameters; execution uses the same operation-specific
session guards as local control. Profile preparation and separate host-tool
workflows retain their stricter idle-session requirement. While an operational
control request is pending, a direct form can still validate its parameters but
Start is refused until that request finishes or is cancelled by its exact ID.
If a separate workbench host process is running, direct form Start remains
available and uses the shared action guards; the host process remains tracked
and cancellable. Background form validation and new host/profile jobs wait for
the existing workbench worker to finish.

The console and `debug-command` can discover and select content without opening
the selector:

~~~text
content refresh
content list --offset 0 --limit 16
content list --offset 16 --limit 16 --generation GENERATION
catalog select ID
~~~

`content refresh` explicitly scans roots already declared in the copy, in a
worker with the same selector limits; it requires pause, no active step, search
or replay and no other selector job. The request stays pending until completion
and can be cancelled by ID: a cancelled result or one for a changed context is
not applied. `content list` reads the current index without rescanning, even
while another job is pending. The result is in `status.content_page`: generation,
total, next offset, partial-scan indicator and up to 16 IDs with type, hash,
availability and compatibility. Specify the received generation for later pages:
a different one is refused. Long names and paths are shortened with explicit
indicators; the ID retains the complete reference and selection rechecks it.
Equivalent direct transport forms are `debug-control content-refresh SESSION` and
`debug-control content-list SESSION [--offset N --limit N --generation N]`.
An older server without this capability is refused before sending. A Wine/Xvfb
check scanned a synthetic session and selected a catalog from one-item pages,
without menu navigation or game advancement.

A separate Wine/Xvfb check scanned 2,048 inert files in an isolated copy. While
refresh was pending, the prior generation remained listable; cancellation of a
wrong request ID was refused, and cancellation of the exact ID returned
`cancelled` with no pending request. The old index remained unchanged after the
worker finished. Parent state/frame/clock and protected inputs stayed exact.

The cheat catalog can be browsed through the same local channel:

~~~text
content refresh
content list
cheats select CONTENT_ID
cheats refresh
cheats list --offset 0 --limit 16
cheats show CH-0001 --offset 0 --generation SHA_SNAPSHOT
~~~

Selection uses a `cheat-catalog` ID discovered by the selector; the text language
does not accept a path. `cheats refresh` requires a paused isolated copy, rechecks
the selected file in a worker and publishes an immutable snapshot identified by
source hash, ROM and generation. Cancel with the exact ID or a deadline discards
the result: the worker finishes reading before the final response. If selection
or context changes, the previous snapshot is no longer queryable. `list` and
`show` queries use the readonly mailbox and remain available during other jobs.
The file is not reread for every query: the snapshot certifies checking at refresh,
not the absence of later disk changes.

A page contains at most 16 rows with truncation indicators. Original JSON detail
is returned in UTF-8 parts of at most 4096 bytes: concatenate `json_part` following
`next_offset_bytes` and verify `entry_sha256`. A different generation or invalid
offset is refused. Direct forms are `debug-control cheat-select SESSION ID`, `cheat-refresh
SESSION`, `cheat-list SESSION [--offset N --limit N --generation SHA]` and
`cheat-show SESSION CH-ID [--offset N --generation SHA]`. A server without
`cheat-catalog-v1` is refused before sending. Refresh assesses bounded
source-linked plans; these catalog actions do not enable or apply a code.
Use `cheats capabilities CH-ID [--offset N --generation SHA]` (or
`debug-control cheat-capabilities SESSION CH-ID ...`) to read the same bounded
source-capability report as the CLI index. Concatenate its UTF-8 `json_part`
pieces using `next_offset_bytes` and verify `capability_sha256`; the immutable
snapshot and its generation are reported with every part. The read-only query
requires `cheat-capabilities-v1` and does not verify the actual ROM checksum,
device or gameplay. Servers lacking this capability are refused before sending.
Frozen Windows GNU builds and focused Wine tests passed, including multi-part
UTF-8 reconstruction, invalid boundaries and old-server refusal. An isolated
synthetic Wine/Xvfb session returned the same report via console, local control
and CLI, with matching hash; a stale generation was refused and the paused
guest, source and protected inputs remained unchanged.

The explicit actions `cheats prepare CH-ID INDEX MASTER GENERATION` and
`cheats apply-once CH-ID INDEX MASTER GENERATION TOKEN` use the same local control
and require `cheat-once-v1`. The first returns the plan, ranges and a token bound
to selection, content, ROM and paused state; the second rechecks everything and
applies once. Retrying with the same identity recovers the outcome without
repeating writes. Prerequisites and limits, including interpreter, allowed memory
regions and guards on recording and new saves, are in
[cheats](cheats.md#manual-one-shot-in-an-isolated-session--dbg2-19-in-progress).
`cheats arm CH-ID INDEX MASTER GENERATION TOKEN` and `cheats disarm CH-ID|all`
use the separate `cheat-maintain-v1` capability. Arm consumes a current preview;
disarm stops subsequent frame applications without restoring earlier memory.
`cheats preset-save NAME`, `cheats preset-prepare NAME` and `cheats preset-arm
NAME TOKEN` use `cheat-presets-v1`. The first saves the current armed set as a
new inert named file, the second returns a complete group preview, and the last
rechecks and arms all members atomically. Direct control uses the equivalent
`cheat-preset-*` actions. See [preset limits and workflow](cheats.md#manual-one-shot-in-an-isolated-session--dbg2-19-in-progress).
The console completes these command names and `all`, and presents the relevant
English or Italian argument hint without guessing a source, master or token.
A frozen Windows GNU build and four focused Wine completion tests passed. In a
separate paused Wine/Xvfb console, keyboard completion displayed `cheats arm`
and `cheats disarm all` with their hints without submitting either draft or
changing the guest. The session pins and protected inputs stayed unchanged.

The selected replay profile can start with `replay run`, or `replay run audit`,
`replay run capture` or `replay run compare`. Direct transport uses
`debug-control replay-run SESSION [--mode plain|audit|capture|compare]`.
The copy must be paused and a compatible profile already selected: the command
does not open a selector. Execution uses the same isolated supervisor as the
menu. `status.replay` distinguishes active worker, received progress and latest
result; its frames are the worker's, not the main copy's. The request result
retains the report and stays pending until worker termination. An older server
without `replay-job-v1` is refused before sending.
In a Wine/Xvfb check with copied synthetic profiles, replay returned `passed`,
capture `captured` and compare `matched`; audit correctly reported endpoint
divergence without claiming success. A replay retry returned the same JSON and
worker run. A wrong ID was refused while another replay was pending. The separate
cancellation/deadline check uses a slow synthetic process, not a worker executing
the game: it confirms supervisor interruption and terminal `cancelled` and
`timeout`, respectively. The main copy stayed at its initial frame and clock,
with unchanged state. A second synthetic session sent `replay run` from the
Command console window and copied terminal JSON identical to the channel response;
clipboard and display were isolated for the check.

A separate copied EUR DS-20 replay confirmed exact-request cancellation while
the real interpreter worker was executing: progress reported 10 of 102 frames
before cancellation, and the same worker reported `interrupted` at frame 21
with the request outcome `cancelled`. The paused parent state/frame/clock,
source profile and protected inputs remained unchanged. In a second copied
DS-20 run, progress reached 9/102 frames before a 1,400 ms deadline. The client
reported an unacknowledged timeout; the later server result for the same request
confirmed `timeout`, no pending request, and the same worker interrupted at
13/102. The paused parent and protected inputs remained unchanged. These checks
do not establish native Windows behavior.

In another isolated EUR DS-20 session, the command API refreshed discovery,
selected the profile by ID and completed the 102-frame replay twice. Both runs
reached the same state hash and clock; retrying the first request identity
returned its exact original JSON and worker run. The second execution was an
explicit new request. The paused parent and protected files stayed unchanged.

At a Windows client size of 480×360 under Wine/Xvfb, the console showed a
replay's full test request ID and parameters while pending, then its result,
frame/clock and copy hint. The session label was abbreviated. A profile using
the previously corrected replay worker completed 102/102 frames, and the
paused parent and protected inputs stayed unchanged. This is one narrow view,
not proof that every long label or parameter fits.

Press **F2** in the Command console to inspect the session label, full ROM
hash, run, pending request context and recent result lines in a wrapped,
scrollable read-only view.
Use Up/Down or Page Up/Down to scroll, then F2 or Esc to return to the draft.
The view preserves the protocol's bounded request and content identifiers;
larger unexpected values are abbreviated. It does not submit the draft. A
separate Wine/Xvfb keyboard check used a verified 480×360 Windows client,
showed a long session label and ROM hash, and returned to submit the unchanged
`status` draft. The guest state and protected inputs stayed unchanged. Long
pending parameters and scrolling were covered by focused tests. A later Wine/Xvfb
run verified a real pending bounded search in a 480×360 Win32 client: F2 showed
the request ID, progress, PC/mode, hit count, frame window, RAM guard and action.
A second live named-search check also displayed the complete trigger name and
definition hash. Each exact request was then cancelled and the windows closed normally. Input was
simulated and protected inputs stayed unchanged. Native Windows and physical
input remain unverified.

The command panel shows bounded summaries for inspections, discovered content,
cheat previews and state, search results, replay results and saved reports.
Summaries are selected by operation; a failed command does not present retained
data from a previous success as its own result. Full responses remain available
as copied JSON. Replay progress shows the latest frontend observation, not a
real-time guarantee. In a 480×360 Wine/Xvfb test the panel showed 11/102 and
52/102 while the same worker reported 63/102 and 84/102, respectively; the
worker then completed 102/102 and the paused parent state stayed unchanged.

The first narrow replay used an older pinned worker and stopped at frame 45 with
`Access denied`. The profile with the corrected worker differs only in runtime
hash and passed under the same kind of polling. That comparison is consistent
with the earlier progress-publication sharing fix, but the error did not name
the denied OS operation, so the precise cause remains unconfirmed. An initial
GUI helper also waited on the wrong response channel and did not attest a normal
parent close; the corrected sessions closed normally.

<a id="richieste-cancellazione-e-scadenza--dbg-20"></a>

<a id="typed-command-recipes--dbg2-16-in-progress"></a>
## Typed command recipes — DBG2-16

The command console and `debug-command` can save and inspect an inert sequence:

```text
recipe save example 'status' 'step 3 --keys 0x3ff'
recipe preview example
recipe prepare example
recipe conclude RUN-ID
recipe prepare example --reopen-reason 'The checked behavior changed'
recipe checkpoint RUN-ID
recipe run RUN-ID
recipe cancel RUN-ID
recipe resume RUN-ID
recipe recover RUN-ID
```

Quote each command, assertion or wait as a separate argument. The shared command
parser converts each command to a typed action before any file is created. A
recipe has at most 16 entries in total across commands, assertions and waits, with at
least one command,
a combined budget of at most 3,600 frames for explicit `step` commands and
worst-case bounded searches, with a 120-second execution deadline and 1 MiB
checkpoint/output limit. Unbounded guest `resume`, cancellation, retry,
help and nested recipe commands are rejected. Saving never executes a step.
Names use 1–48 ASCII letters, digits, hyphens or underscores. At most 32 files
are stored in the selected session's `Data/config/debug-command-recipes/`;
existing names are not overwritten.

An explicitly terminal checkpoint (completed, failed, assertion-failed or
wait-timeout) can be marked concluded with
`recipe conclude RUN-ID`; this records its hash without dispatching an action.
Later preparations of a matching recipe in the same session require a nonblank
reopening reason of at most 512 printable UTF-8 bytes. The new checkpoint stores
the reason and the concluded run IDs and hashes. Matching uses the ROM pin and
ordered typed actions, so renaming the recipe or changing assertions or waits
does not avoid the check. Ordinary completed but unmarked runs remain repeatable. Preview
shows matching conclusions; the Scripts and tools form has a reopening-reason
field. A changed conclusion, reason or recipe after Details requires a new
review, and a conclusion added after preparation is checked before first start.
An isolated Linux/Xvfb GUI test selected the recipe and session, displayed the
conclusion, refused an empty reason, required a new review after editing the
reason and completed one subsequent frame. It used simulated keyboard and mouse
input; native Windows and physical input remain unverified.
The local files are trusted evidence and can be moved or removed by their owner;
semantically equivalent checks with different commands are not inferred.

Typed command recipes do not dispatch campaign `run-case` or discover campaign
evidence. To rerun a concluded campaign check, use the explicit
[campaign workflow](debug_campaigns.md), which requires its own reopening
reason. Arbitrary manually repeated checks cannot be identified automatically.

An assertion immediately follows the action whose terminal response it checks:

```text
recipe save checked 'status' 'assert /status/paused true' 'step 1' 'assert /status/frame 42'
```

The pointer uses JSON Pointer syntax and is limited to 256 bytes. Expected
values are `true`, `false`, `null`, an integer, or `text:VALUE` for a string;
objects, arrays and floating-point values are rejected. A missing pointer
does not equal `null`. The checkpoint records each comparison. A false
assertion stops the sequence with `assertion-failed`; a later resume does not
resend the action that produced the response. Use values appropriate to the
selected session rather than assuming the example frame number.

A wait after an action polls the original session's read-only `status` response
until a scalar matches, or a bounded time or poll limit is reached:

```text
recipe save observed 'step 1' 'wait /status/paused true 2000 100 10' 'step 1'
```

The numbers are timeout in milliseconds (20–30,000), interval in milliseconds
(20–1,000) and maximum polls (1–128). There can be at most one wait per action;
assertions attached to the action are checked against its terminal response
before its wait, regardless of their textual order. The wait checks the same
scalar and JSON Pointer rules as assertions. It observes the current state: it
does not advance a paused guest or wait for a particular external job itself.

The action's terminal result and wait deadline are stored before polling. New
servers anchor that deadline to the server's terminal completion time, so a
delayed client cannot extend it; older servers retain same-run compatibility
with a deadline measured when the client receives the result. Each
poll is reserved in the checkpoint before the read; an interruption may consume
a poll without recording its response. A resume explicitly continues with the
remaining polls and original deadline and never resends the completed action.
The checkpoint stores the last bounded scalar observation and response digest.
New observations also retain an idle guest boundary when one is available.
A missing or non-scalar value cannot match. A timeout or exhausted poll count
stops later actions with `wait-timeout`; an unconfirmed read stops with
`unconfirmed`. Each status observation can use the remaining original wait
deadline, including when serializing state takes more than one second. Cancellation
is checked while a poll is in flight as well as between polls; an already published
query may still complete. Status polls are fresh, read-only observations and may
see external changes; this runner does not own
exclusive control of the session. The deadline uses the system clock.

A Windows GNU test binary and 30 focused Wine tests passed, including a
matching wait, exhausted poll budget, cancellation and resume without repeating
the completed action, parser limits and existing recipe/control contracts.
These use a synthetic server; real interrupted sessions and native Windows remain
unverified. Two sandboxed Wine attempts could not create the server lock; their
owned leases were released after host process checks, and diagnostics retained.
A separate native Linux build attempt lacked ALSA development metadata and hit
a temporary-file quota; it did not run tests.

Windows GNU builds and 27 focused Wine tests covered passing and failing
assertions, malformed values, missing versus explicit `null`, altered
checkpoint checks and recovery after a lost action response. A separate
Wine/Xvfb EUR interpreter run passed a status assertion, advanced one frame,
then failed an assertion and confirmed that neither the next step nor a resume
advanced the guest. Protected inputs stayed unchanged. A previously completed
checkpoint without assertions was also reread successfully without modifying
it. The live assertion unit retains its private test copies as a named
current-proof exception. An initial test run failed when the synthetic server
treated a temporarily denied result publication as fatal; its outbox retries
on later polls, and the corrected test passed. Native Windows and interactive
console entry were not tested for assertions.

`recipe prepare` creates an inert checkpoint with a unique run ID and a
planned request ID and action hash for each step. It binds these to the recipe,
session manifest and control run in the readiness metadata; it sends no
request. `recipe checkpoint RUN-ID` reloads and validates that plan, refusing
changed recipe content or altered identities, and reports whether readiness
metadata still matches the original run. That metadata alone does not prove the
process is alive. At most 32 checkpoints are retained in the session's
`Data/config/debug-command-runs/` directory. Preparation does not dispatch;
`run` is a separate explicit action.

The runner accepts `status`, `inspect`, bounded `step`, `content refresh`,
`content list`, `profile select`, `catalog select`, `cheats select`, `cheats
refresh`, `cheats list`, `cheats show`, `cheats capabilities`, `ui-start`,
`ui-report`, `graphics`, `audio-start`, `audio-report`, `code-start`,
`code-report`, `gameplay-start`, `gameplay-report`, `pause`, recent-event
capture/freeze/report, issue report, guarded cheat preparation/application,
arming/disarming and presets, bounded `break`/`watch` searches and capped replay
jobs. Discovery and selection
wait for their terminal responses before the next entry. These
operations keep their existing idle-session, catalog-generation and validation
guards. Cheat operations retain their generation and preview-token requirements;
the recipe does not interpolate a token from a prior result. For example:

```text
recipe save indexed 'content refresh' 'content list --limit 4' 'status'
recipe save ui-case 'ui-start' 'step 1' 'ui-report'
recipe save recent-case 'recent-start' 'step 1' 'recent-freeze' 'recent-report'
recipe save replay-case 'replay run plain --max-frames 102' 'step 1'
```

Select the replay profile before the replay action. Each replay requires an
explicit `--max-frames` cap; the profile's execution window must fit that cap.
For a comparison this is its selected window, not the length of the full
recording. The recipe reserves
that many frames against the shared 3,600-frame budget, or twice that number
for a comparison because both machines execute frames. The supervisor pins the
profile hash and the worker rechecks both hash and cap before executing. A
bounded plain replay refuses a comparison profile so its cost cannot be
underestimated. The selected worker executable must match the current frontend
executable: another or older worker might ignore the bounded-worker protocol.
Create a new profile with the current runtime when a prior profile uses a
different executable; do not alter the pinned original. Ordinary uncapped replay
outside a recipe retains its existing runtime options.
Older runtimes without `bounded-replay-v1` are refused before
dispatch; legacy saved recipes with uncapped replays remain readable but cannot
run. A recovered cross-run recipe does not restore an old replay worker.

In an isolated Linux/Xvfb EUR session, a copied 102-frame profile was refused
before starting a worker with a cap of 101; the same profile completed under a
cap of 102 and the following main-session step executed once. An explicit
second resume left the checkpoint and guest state unchanged. Protected inputs
were unchanged and the window closed normally.

An isolated Linux/Xvfb EUR session ran pause, recent start, one frame, freeze
and report in order. The report stayed unchanged and was not duplicated on
resume. A separate recipe attempted to disarm an inactive cheat: the frontend
refused it, the following step stayed planned and the guest state was unchanged.
The windows closed normally and protected inputs were unchanged. Other cheat
operations and the issue report have synthetic ordering coverage here; this does
not establish device-level cheat behavior.

Capture start/report actions retain their own time, frame, event, report-size
and output-directory limits; the recipe's 1 MiB limit applies to its checkpoint
and responses, not the separately saved reports. Cancellation does not discard
an already started capture. External controls can affect a capture while the
recipe is running. Unsupported actions, including unbounded resume, are still
refused before dispatch.

Each `break` or `watch` search reserves its existing worst-case ceiling of 600
completed frames against the shared 3,600-frame budget, even when its instruction
limit is reached earlier. A capture-on-hit search additionally reserves one frame
for its completed-frame snapshot. The search also keeps its 1..5,000,000 instruction
limit and existing interpreter and feature guards. Legacy saved recipes without
the new budget field remain readable; the runner recomputes their worst-case
budget before sending any action. A search that completes without a hit is a
successful bounded search result, so use an assertion such as
`assert /status/probe/outcome text:hit` when a hit is required before continuing.
A search can stop inside a guest frame and an unconfirmed result retains its
request identity for ledger recovery.

Windows GNU build and 33 focused Wine tests passed, including aggregate budget,
legacy identity and assertion-on-miss checks. In an isolated EUR Wine/Xvfb
interpreter session, a one-instruction search returned `not-reached`: an
assertion requiring a hit stopped the following step, while an assertion accepting
`not-reached` allowed one frame. Explicit resumes did not duplicate actions;
protected inputs stayed unchanged and the GUI closed normally. An initial
supervisor overwrote its process variable after the functional check; its owned
Xvfb and leases were recovered after host inspection and independent input
hashes, and the corrected unit passed with an automatic receipt. In a separate
isolated EUR Wine/Xvfb session, a Thumb breakpoint reached the observed paused
PC after 40,948 interpreter steps, before executing the matched instruction. An
assertion accepted the hit, the following step advanced one frame, and explicit
resume did not repeat either action. The checkpoint recorded both steps, the
window closed normally and protected inputs stayed unchanged. In an isolated
Linux/Xvfb EUR interpreter session, an audit-enabled build ran a recipe watch
for a RAM read, asserted the hit and executed its next step; the first match was
an HLE/DMA-inclusive interpreter step, not an attributed guest instruction.
A second resume left the checkpoint and guest state unchanged. With the feature
disabled, the same recipe recorded the feature refusal, left the next step
planned and preserved the guest state. Both windows closed normally and the
input copies were protected. Native Windows watch behavior remains unverified.

A Windows GNU build and 31 focused Wine tests covered capture action order,
report-response recovery without duplicate execution and existing recipe
contracts. An isolated EUR Wine/Xvfb interpreter run executed `ui-start`, one
frame and `ui-report`, producing one complete UI report; a second resume did not
change guest state or create another report. The window closed normally and
protected inputs stayed unchanged. That run used a stripped copy of the built
runtime. A subsequent isolated EUR Wine/Xvfb run exercised audio, code,
gameplay and graphics in separate recipes. Each produced one report with the
expected kind and no duplicate on explicit resume; the first three each advanced
one frame, graphics advanced none, and repeated resumes preserved guest state.
Protected inputs stayed unchanged and the window closed normally. These runs
validate one interpreter session, not capture fidelity or physical audio.

A Windows GNU build and 29 focused Wine tests covered catalog action ordering,
selection refusal, recovery of a lost selection response with the original
identity and refusal to retry an ambiguous catalog read. A separate EUR
Wine/Xvfb interpreter run refreshed content and listed four entries in order;
an invalid selection stopped the next step, and explicit resume left guest state
unchanged. The GUI closed normally and protected inputs stayed unchanged. The
first test wrapper expected a nonexistent response `limit` field and was
corrected to inspect the returned entries; its failed diagnostics were retained.
A further isolated EUR Wine/Xvfb run selected a discovered synthetic cheat
catalog and then refreshed, listed, showed CH-0001 and read its capability
report through one recipe. The selection ID, source hash and snapshot generation
matched across results; the detail and capability digests matched their returned
content. A second resume and guest state were unchanged, protected inputs stayed
unchanged and the GUI closed normally. This verifies the channel workflow, not
cheat behavior or device equivalence.

The runner checks session input pins, persists the original deadline and each request ID before
sending, and journals terminal results in order. A retry of an unresolved
ledger-protected step keeps the same ID, action and deadline; completed steps
are skipped. A pending read query has no server retry ledger, so an ambiguous
result stops with `unconfirmed` instead of being repeated automatically. A
server run change, expired deadline, refusal or output limit also stops the
sequence. The deadline uses the system clock and includes input validation,
but a blocked filesystem read cannot be interrupted by this runner. Concurrent
external controls are not excluded by a recipe-wide session lock.

`cancel` requests a stop between steps; a step already reserved may finish.
The runner reports `cancellation-pending` when that step's outcome is unknown,
and `cancelled` only after reaching a boundary. `resume` explicitly clears the
cancellation marker and continues within the original deadline. Cancellation
does not roll back effects already applied. Checkpoint files
are synced and atomically replaced, but power-loss durability is not claimed.

Windows GNU builds and 16 focused Wine tests checked distinct planned IDs,
checkpoint reload, changed recipe/session handling and rejection of altered
identities, alongside the existing retry-ledger tests. A synthetic CLI check
reloaded the same plan in a separate process without publishing a request. Its
first wrapper failed to enumerate the generated directory under Wine; the
corrected wrapper passed. These checks do not execute a recipe or test recovery.

The file records the session manifest hash and its declared ROM/runtime hashes.
Preview validates the typed actions and limits and refuses a different manifest
or pins; it does not independently hash the session payloads. Preview returns
the complete typed sequence and identifies the runner's supported subset.
In **Scripts and tools → Debug command recipe**, select the recipe and its
original session, then choose **Details** to review it. The picker validates
the selected file and session; a changed selection or session is refused. The
tool shows the complete bounded recipe in its scrollable details. **Start** now requires a
successful Details preview for the current view and unchanged selection. It
rechecks the recipe and session, prepares a new checkpoint and uses the same
bounded runner as the console. The session's saved recipe of that name must
match the reviewed selection; unsupported actions are refused before dispatch.
The tool displays the run ID and outcome, and the full checkpoint remains in
the session for inspection or explicit console resume. **Cancel process**
requests a stop at the next action or wait boundary; an already reserved action
may finish, and no prior effects are rolled back. If its response is still
pending, the client stops waiting and reports `cancellation-pending`; the
checkpoint and original request identity are retained for explicit recovery.
The typed recipe's own limits
apply; the readonly adapter's runtime and timeout fields apply only to its
separate format. Focused synthetic Wine tests exercised Details, Start, cancellation and explicit resume. A menu rendering test checked that a long preview retains its tail and scrolls at a narrow size. An isolated EUR Wine/Xvfb session with simulated keyboard and mouse selected a saved one-frame recipe and its original session through the interactive picker. Start was refused before Details without creating a run; after Details, Start completed one frame and the checkpoint recorded the step as completed. Protected inputs stayed unchanged and the window closed normally. This verifies one interactive path, not native Windows or physical input.

At that earlier authoring stage, action coverage and interrupted execution were
still incomplete; the subsequent checks above and below cover the implemented
runner paths. Wider picker and platform scenarios remain open. Windows GNU builds
and 19 focused Wine tests passed for authoring, validation, parser, completion and
operation metadata. A synthetic session without ROM or control mailbox also
passed save, preview and duplicate refusal through the Windows CLI under Wine.
Native Windows and the console GUI were not tested for this workflow.

A separate synthetic Windows GNU/Wine integration check selected and reviewed
a typed recipe through the tool's input and Details handlers, verified that
Start was refused without creating a control mailbox or host run, and checked
the long-preview warning. Eighteen focused tests passed; they did not exercise
the interactive picker. An
unrelated link-safety test could not materialize its symlink fixture under Wine;
the scoped test rerun excluded that case and retained its failed unit.

For the limited runner, 21 focused Wine tests passed. A synthetic control server
checked cancellation after one completed step, explicit resumption without
repeating it, a lost terminal response recovered through the same request ID,
first-error stop, expired deadline, output cap and refusal to retry an ambiguous
read query. A separate isolated EUR Wine/Xvfb interpreter session cancelled before
execution, resumed a two-frame step followed by an inspect, and confirmed that
a second resume did not advance the guest again. Protected inputs stayed
unchanged. That live test did not interrupt between completed steps; the
between-step interruption and lost-response checks used the synthetic server.
Its log contains two `Access denied` control messages despite the successful
terminal result; the exact denied operation is unconfirmed. The successful
unit retains its private test copies as a named current-proof exception.

A later isolated EUR interpreter session under Wine/Xvfb completed one frame,
entered a bounded status wait, and had its own client process terminated. An
external one-frame step satisfied the wait; explicit resume retained the first
request ID, completed the following step and left the guest paused. Observed
frame offsets were 0, 1, 2, 3 and 3 after a second resume; the second resume also
left state hash, clock, keys and registers unchanged. The session closed normally
and protected inputs stayed unchanged. This covers an interrupted client during
a wait, not a crash during a pending mutating action or a session restart. Its
private test copy is retained as the current interrupted-proof exception.

A second isolated EUR Wine/Xvfb run terminated its own client after the server
reported a 60-frame step pending. The checkpoint retained that request ID and
kept the following step planned. Explicit resume recovered the same request,
completed exactly 60 frames and then one more; another resume left frame,
clock, registers, keys and state hash unchanged. The window closed normally and
protected inputs stayed unchanged. An initial 600-frame attempt exceeded the
120-second deadline in this interpreter and returned `unconfirmed`; its
checkpoint and diagnostics were retained. A separate 60-frame launch encountered
another owner's Wine lease and started no test; the successful run followed its
release. This does not establish automatic recovery after restarting the session itself.

A separate EUR Wine/Xvfb test closed and reopened the isolated session while a
checkpoint retained a completed action followed by an outstanding wait. The
new control run had a different identity: inspecting the old checkpoint remained
possible, but resume was refused before dispatch. Checkpoint bytes, the command
channel and the reopened guest state stayed unchanged; both session windows
closed normally and protected inputs remained unchanged. This test predates the
limited explicit cross-restart recovery described below.

`recipe recover RUN-ID` can rebind a checkpoint to a new control run without
dispatching an action. A pending step can first be resolved from its successful
terminal response with the exact run, request, deadline, operation and
fingerprint. Current servers attempt to save a create-once `terminal-NN.json`
receipt in the owning recipe run before publishing the shared mailbox. Each
receipt is limited to 128 KiB; at most 16 step receipts belong to a run and
32 runs are retained. Receipt publication is best-effort: if it fails, the
normal mailbox still works. Recovery prefers a present receipt and refuses it
if invalid; when no receipt exists, older sessions can use the mailbox. It does
not resend the action. Receipts and checkpoints are trusted local evidence, not
signed records; protect the session directory as private. No guarantee is made
for a crash before receipt publication or sudden power loss. If steps remain,
inspect the result and invoke `recipe recover RUN-ID` again to rebind them. A
missing, mismatched or failed response is not retried. For an attached wait, recovery
requires a valid original terminal completion time, retains the earlier of its
wait deadline and the recipe deadline, and reports `wait-timeout` if it has
already elapsed. Responses from older servers without that time cannot recover
an attached wait. Invalid or out-of-budget timestamps are refused.

Completed UI, audio, code and gameplay start/report pairs can precede the
recovery boundary; their recorded results and request identities remain intact
and neither action is repeated. The report files remain ordinary local artifacts:
recovery does not verify that they have not been moved or modified. An open UI
trace can be retained on normal window close when exactly one stopped recipe
owns it, its remaining steps are unsent, and the paused guest boundary and
selected inputs still match. The private snapshot is bounded and linked to the
checkpoint by hashes. A missing, altered or ambiguous snapshot is refused.
The frontend checks the boundary again and restores the trace immediately
before the first remaining action; a server without this capability refuses
the request. Previous events retain their original session, and the report
marks the interval between sessions. The original 30-second limit includes
time spent between close and recovery. This does not reconstruct crashes before
the snapshot, old checkpoints without one, interrupted waits with an open UI
trace, or open audio, code and gameplay collectors.

Rebinding requires a completed action, optionally followed by its recorded
wait, and only unsent later steps, unchanged session inputs and an unexpired
original deadline. It preserves completed request identities, assigns new identities to unsent steps,
and records the last result's paused guest boundary. Restore that exact state
in the new session, then explicitly use `recipe resume RUN-ID`: the frontend
checks its full state hash, frame, clock, backend, absence of armed cheats,
pending control/search work, replay and retained UI, audio, recent-event,
code and gameplay collectors
immediately before the first action. A mismatch refuses the action. The old
deadline is not extended, and a runtime without the guard refuses before the
mailbox write. Completed steps may use status, inspection, bounded stepping,
search with no action or pause, content discovery and selection. For each
completed selection, recovery retains the role, anchored location, kind and
content hash, rechecks the input outside the frontend thread and guards the
restored selection immediately before dispatch. Temporary catalog IDs may
change after reopening; the stable pin must match. At most three selected
inputs and 2 KiB of pin metadata fit in the guarded request. Older selection
results without a recovery pin are refused. Recovery pins can contain explicit
external paths; treat the checkpoint and copied JSON as private. The catalog
index itself is not restored: remaining catalog list/selection commands
require a new explicit refresh, and cheat queries require a new cheat refresh.
Input contents are checked during recovery and again by the consuming action's
preflight; there is no continuous filesystem watch between those operations.
Remaining steps may
also start a new audio, code, gameplay or UI collector before its report,
capture graphics before a comparison, or use a search capture action.
An unresolved action without a matching response, an unsupported open earlier
collector, a report without its collector and event-freeze searches are refused;
collector contexts are not restored by a guest-state hash alone. A recorded wait can be
rebound if its original deadline and poll budget remain, and its most recent
poll has a recorded idle guest boundary; before any poll, the action result
provides that boundary. The first new status poll checks that
boundary; the prior action is not resent and its deadline, wait deadline and
consumed poll count are retained. An unconfirmed last poll or an older wait
observation without a recoverable boundary is refused. A guarded or ordinary poll may
wait within the remaining original deadline, and a cancellation marker remains
observable while it is in flight. An isolated Linux
interpreter session under Xvfb completed a status step, stopped before its next
step, closed and reopened normally, rebound the checkpoint, and resumed one
frame. The original status response and request identity were preserved; a
second resume left the guest unchanged. Protected inputs stayed unchanged.
Another isolated Linux/Xvfb run started a UI collector after the restart,
stepped one frame and wrote its report; the prior status result and a second
resume remained unchanged. A further Linux/Xvfb run interrupted a recorded
wait, closed and reopened normally, then observed the restored boundary before
an external step satisfied the wait and the recipe executed its next step.
The original action response and both deadlines remained unchanged; the poll
count advanced from one to three and a second resume did not change state.
Another isolated Linux/Xvfb run retained a synthetic catalog selection across
two normal restarts, despite a changed temporary ID. The completed selection
and intervening status responses stayed unchanged, and a second final resume
did not change state. Another run stopped its own client after the server had
written a terminal status response but before the checkpoint recorded it;
after a normal restart, recovery recorded that exact response and continued
without resending the status. A separate Wine/Xvfb test interrupted its own
client after a receipt was written, issued a new status that overwrote the old
mailbox, restarted normally and recovered the receipt without resending the
action. Its following frame ran once and a second resume was unchanged. These
runs used simulated cancellation or client interruption; the Linux runs used
silent audio. A further isolated Linux/Xvfb run interrupted a
pending status with an attached wait, then recovered its exact response and
original deadlines across a normal restart. A guarded observation preceded an
external frame and the recipe's next frame; a second resume changed nothing.
Windows GNU/Wine focused tests cover the
collector ordering, selected inputs, wait recovery, retained terminal
responses, delayed response, cancellation and refusals. An isolated EUR
Windows GNU/Wine/Xvfb restart also preserves a completed status response and
request identity, recovers without guest mutation, executes the following
one-frame step once and leaves state unchanged on a second resume. Both game
windows close normally. Its native driver runs through
`validation_unit.py --wine-host`, under one prefix lease through both runs and
the final server drain; protected inputs remain unchanged. The observed GPU
failure used CPU presentation, so this is not GPU-path or native Windows
acceptance. A separate Wine/Xvfb normal restart recovered an interrupted
recorded wait: the guarded observation preserved the original boundary and
deadlines, an external frame satisfied the wait, and the following recipe frame
ran once. The completed action response and second resume remained unchanged.
Another isolated Wine/Xvfb interpreter session recovered a synthetic catalog
selection across two normal restarts: its temporary ID changed, while role,
location, hash, completed responses and the original deadline stayed consistent.
The following recipe frame ran once and a second resume left the checkpoint
unchanged. Protected inputs and the prefix lease were checked. These runs used
a private debug-symbol-stripped runtime, simulated cancellation and CPU
presentation; native Windows, broader Wine wait/selection/collector coverage and
broader host-context recovery remain open.

An isolated Wine/Xvfb run stopped a recipe after UI trace start, closed and
reopened normally, and recovered the saved trace before one guest frame and its
report. The original start response remained unchanged, the report contained
events from both control runs and one recorded gap, and a second resume did not
change the checkpoint. The runner protected its inputs and drained its leased
Wine prefix; the test used simulated cancellation and CPU presentation.

Another isolated Linux/Xvfb run completed a UI report, interrupted before its
next recipe step, and restarted normally. Recovery kept both original request
identities and responses; the report JSON and HTML hashes remained unchanged,
only one report existed, and the following step ran once. Synthetic checks cover
the same completed-pair rule for audio, code, gameplay and UI captures.

Two initial attempts exited before opening the session because the test bridge
inherited validation variables forbidden by the isolated-session guard. The
bridge was corrected; the same D-Bus warning also appears in the earlier passing
run and did not identify the cause. Both failed diagnostics were retained.

## Requests, cancellation and deadlines — DBG-20

Access explicitly targets the isolated copy's folder. No port, socket or network
service is opened; the main game does not create the channel. Anyone able to
write the folder can control the copy: this is not an authentication boundary
against other system users.

**ready.json** declares schema1, a unique run per start, PID, capabilities and
session identity (manifest SHA256 and bounded label). Every request/response
carries run and ID; the new client also compares manifest identity. An old copy
without these capabilities requires a new session created with the current
runtime; do not replace already authenticated executables.

To query a long command and cancel only the one observed:

~~~powershell
.\recomp.exe debug-control step 'target\debug-case' 600 --timeout-ms 120000
# From another terminal, read status.pending_control.request_id:
.\recomp.exe debug-control status 'target\debug-case'
.\recomp.exe debug-control cancel 'target\debug-case' --request 'EXACT_ID'
~~~

**cancel** stops a CLI step/search budget at the next frontend return and leaves
the copy paused. An old/different ID is refused without cancelling another
command. The operational command ends with outcome **cancelled**, remaining 0
and actual partial state; the cancel query confirms separately. It does not
rewind completed frames. On shutdown a response without final state does not
invent a guest boundary: status is null and the outcome is closed. For replay
started from the channel, cancel requires the exact ID and confirms only the
request: the terminal result confirms any interruption. At the deadline,
cancellation is requested and the result stays pending until worker termination.
Replay started from the menu uses its own Cancel replay; a menu/keyboard budget
remains interruptible with F6/Escape.

All commands accept **--timeout-ms N**, 20–120,000 ms, default 120,000.
The request includes deadline_unix_ms: an already expired request is not
executed; the frontend stops an expired pending budget with control-timeout.
Client waiting uses a monotonic clock. The server deadline uses wall-clock time
and may be affected by clock changes; it does not interrupt an instruction,
frame, blocked callback or synchronous operation halfway through. A synchronous
operation already completed after expiry returns timeout with the observed
state, without rollback.

The client returns JSON for local errors or lack of confirmation too:
outcomes **ok / refused / cancelled / timeout / closed**, plus
**identity-conflict / result-unavailable / capacity** for retries, with ok,
error, run/id/session when available. Before sending, invalid syntax or arguments
produce `parse-error`, an incompatible protocol, manifest, capability or run
produces `incompatible`, and an unavailable session or lock produces `unavailable`:
these local outcomes indicate `delivery: not-sent`. A transport or worker error
may instead report `delivery: unknown`: it does not prove the server did not
receive the request. The server continues to report `refused` for its refusals,
including content-specific compatibility, with details in the error. Also
possible are `invalid-request` for an invalid typed action and `composition-active`
when the console blocks Enter during composition. A client timeout with
server_acknowledged=false does not prove the frontend has stopped work. Use
status and the ID to check before retrying. A later server response may be in
result.json; only the latest per mailbox is retained. Concurrent queries are
refused by their own OS lock; client death releases it automatically. A job's
terminal response reports `pending_control: null`: the completed request no
longer appears active. If publication must be retried, correlation retains
the original job ID.

The current runtime also offers explicit retry with the same identity:

~~~powershell
.\recomp.exe debug-control retry 'target\debug-case' --run RUN --request ID --deadline-unix-ms DEADLINE --timeout-ms 5000 -- step 1 --keys 0x3ff
~~~

`RUN`, `ID` and `DEADLINE` come from the original request; after `--` repeat
the exact action and arguments. They are present in results and client timeout.
They may also be chosen before sending and retained for a possible retry. The
server compares a fingerprint of the complete request: an ID with different
parameters is refused, a pending one is not reexecuted and a completed one
returns its retained result. The per-start limit is 4096 identities, with space
reserved for cancelling the active job; at most 128 responses, 128 KiB each
and 2 MiB serialized total remain cached. If the response is no longer available,
the server states so without reexecuting the action. Memory applies only to the
current run: a different run refuses retry. A new command with another ID is a
distinct action. A Wine/Xvfb check verified that retrying a completed step does
not execute another frame and a pending search retains its owner; different
parameters and runs were refused. Responses include `request_sha256`, which the
client compares before accepting a response from the updated runtime.
Retry requires a server supporting this feature; `status` is an unrecorded read
and is queried with a new request. The language shared by the console and
`debug-command` also accepts, for example,
`retry RUN ID DEADLINE -- step 1 --keys 0x3ff`, or after `--` a
`profile select ID` or `catalog select ID` command. The deadline is the original
decimal `deadline_unix_ms` number. JSON response fields can be copied from the
console with Ctrl+Shift+C. The request is always explicit: if original identity
or action is unavailable, do not invent an ID for recovery. Nested retries,
help and status are not allowed; the console waits up to 120 seconds. A Wine/Xvfb
check retried a completed step from the console: same JSON, no second frame and
changed parameters refused.

Schema1 requests without the new optional fields remain decodable for local
compatibility, without a guaranteed deadline. The current client always
publishes deadline and manifest. Primary and query channels refuse incompatible
actions; stale/malformed requests are consumed without application and logged
in the copy. Reopening changes run, so an earlier result does not confirm a new request.

<a id="audio-pacing-e-file-locali"></a>

## Audio, pacing and local files

Pause invalidates frontend queues and DSP history and silences host output.
Each step drains guest taps without queueing them to the device. Resume starts
with empty queues, new audio priming and the current host deadline; pause time
creates no frames to catch up. Guest audio-generator state remains part of normal
emulation. Listening and physical-device delay are separate from queue checks.

The local channel lives in `Data/config/debug-control/` of the portable copy:
`ready.json`, `request.json`, `result.json` and client lock; query uses query.json,
query-result.json and query.lock. Atomic publication, per-process and request
identity and OS locks separate starts and clients; old results do not confirm a
new command. Only the latest response is retained, without timeline, new slots
or snapshot collections. All files stay in the private Git-ignored session.
The channel closes with the window and is recreated with a new identity on reopening.

Before reading a request the server atomically moves it to its own
`request.claim` or `query.claim`, one per channel. A later published request
cannot be removed in place of an already claimed one; if reading or removal
fails temporarily, claiming is retried before taking another. A remnant of an
old run is discarded by identity verification and not executed. Oversized or
malformed inputs are consumed and logged without guaranteeing a client response;
a nonordinary file may require intervention. Errors distinguish channel and phase.
The mailbox remains single-slot: after an unconfirmed timeout, an unclaimed
request may be replaced and is not confirmation of execution.

<a id="ambito-della-verifica"></a>

## Validation scope

This unit's checks are recorded in its change record. The isolated Windows
candidate is `target/debug-tools-step/recomp.exe`; normal executables remain
separate. No new ROM/DS acceptance or native parity, benchmark, Godot frontend
or Android change follows from these commands.

<a id="verifiche-del-28-ottobre-2026"></a>

## Validation on 28 October 2026

Windows release feature/default builds without warnings from `07be4d2` plus five
selected files, with locked/offline dependencies and outputs separate from
production. **69 Wine tests** pass: four control, six session, twelve frontend,
one audio callback reset and 46 menu. The first sandbox attempt could not create
the Wine socket: no tests had started; the same executables pass with the socket
exception. No product behavior changed for that environment limit.

**24 CLI/GUI checks**, nine on the default worker and two controlled stepping
processes pass. F7 held beyond autorepeat, deliberately also bound to A, produces
only one frame with neutral input. F8 and the two menu commands produce ten/one/ten
frames: the total of 22 matches a single 22-frame CLI command in complete state.
Explicit A/release masks work. Pause and waiting preserve hash, frame and clock,
with empty guest/host audio queues. Recordings of **22 stepped frames** and
**80 frames with pause/resume** pass independent complete-state/clock verification
against continuous interpreter execution.

Refusals for limits/incompatible masks, concurrent client, command on closed
session, Escape interruption, owner shutdown with active budget, F6/CLI/menu
resume and refusal to step while running freely are verified. The default build
executes the feature without audit. The revision added refusal of Replay/Choose
profile/Audit/Capture/Compare during a budget, before opening the selector or
starting a worker. After only that guard and a comment, both candidates were
rebuilt and the new branch checked in the default GUI; previous results for
remaining behavior are retained.

The first GUI script assumed the initial menu cursor was on Resume. Normal hover
had selected another row and the script entered bindings: the attempt was
rejected and retained. The corrected test uses an explicit Debug hit and parks
the pointer; final screenshots were inspected. All **202 checked original inputs**,
including normal Windows/Linux executables, retain hashes and timestamps. Three
accepted sessions close normally; private results and payloads remain under
`target/debug-tools-step/`. Managed publication directories were not created;
the associated ROM helper does not apply. Diagnostic copies and preimages are
retained, reusing the build cache and replacing only superseded local candidates.

Native Windows, Linux/Wayland, physical controllers, other scenes/ROMs/datasets
and steps with native translation remain unverified. Audio checks cover buffers
and callbacks, without listening or physical-device latency evidence. In this
initial DBG-10 validation, a real 120-second client timeout and forced server
death were not simulated; results from a killed process may remain local until reopening.

To stop on PC/mode or bus access and read registers and memory without I/O, use
[DBG-11: breakpoints, watchpoints and inspection](debug_probes.md). These are
bounded interpreter searches to a point that can be inside a frame, distinct
from the completed-frame stepping described here.

To gather framebuffer, layers, palette/tile/OAM and windows in an offline report,
use [DBG-12: graphics inspector](debug_graphics.md), from the menu or with
`debug-control graphics SESSION [--compare-last]`. Capture remains readonly.

<a id="ui-testo-e-input"></a>

## UI, text and input

The command editor clears only pending composition when the IME is disabled or
the console is hidden; an empty IME commit also preserves the draft and its
selection. On Wine, the command window forwards an actual late IME result
through the ordinary editor path; it never substitutes uncommitted preedit text.
An isolated Cangjie test verified repeated commits, cancellation and focus
changes. Native Windows and broader input-method coverage remain unverified.
The console also refuses commands before dispatch when the session lacks a
required capability; a Wine GUI test verified the copyable `not-sent` result.
If the platform provides text for an unidentified physical key, the console
accepts that text subject to the usual modifier, composition and context guards.
An Italian-layout `è` key was verified in Wine without changing the guest state.
The draft and composition lines use the bundled Droid Sans Fallback face when
the primary font lacks a glyph. Cangjie `日` was visually checked in Wine;
characters absent from both faces may still display a missing-glyph box.

`debug-control ui-start SESSION` begins bounded collection;
`ui-report SESSION` saves focus, font metrics and observed sequence in an offline
report. `status.host_menu` also exposes the dropdown and actually active page.
See [DBG-13: use and limits](debug_ui.md).

In isolated Windows GNU/Wine sessions, direct control, the menu and the command
console each started a UI trace, refused a second start while it was active and
saved a report. The CLI and console returned the same refusal text; the menu
displayed it. Guest state/frame/clock and protected inputs stayed unchanged.
The reports contain different UI events, so they were not compared byte for byte.

<a id="cattura-audio"></a>

## Audio capture

`debug-control audio-start SESSION [--frames 1..180] [--streams mixed,psg,hle,fifo0,fifo1]`
arms a short window; `audio-report SESSION` publishes the report after pausing.
`status.audio_diagnostic` distinguishes collection, limits and stop reason,
while `audio_report` identifies the latest HTML. Menu and CLI share collection.
See [DBG-14: guest buffers, callback, comparison and local WAVs](debug_audio.md).

<a id="codice-label-e-fallback"></a>

## Code, labels and fallback

`debug-control code-start SESSION [--frames N --events N --start HEX --end HEX --mode arm|thumb|all --kind TYPE]`
arms the prefix; `code-report SESSION` saves PC/mode/backing/disassembly/label
and native/interpreter choice. `status.code_trace` reports limits and filters.
See [DBG-15: dispatch meaning and limits](debug_code.md).

<a id="report-locale-del-problema"></a>

## Local issue report

`debug-control issue-report SESSION [--profile FILE --symptom TEXT]` uses the
selected profile and saves a local index without starting replay or sending
attachments. `status.issue_report` identifies the file. [DBG-16](debug_issues.md)
describes reproducible command, metadata and private attachments.

<a id="task-script-e-unità"></a>

## Tasks, scripts and units

`debug-control gameplay-start SESSION [--frames N --events N --family all|task|script|unit --context HEX]`
arms collection in the paused copy; `gameplay-report SESSION` saves events with
per-field provenance and resource correlations. `status.gameplay_trace` exposes
filters and limits; `gameplay_report` identifies the HTML.
[DBG-17](debug_gameplay.md) distinguishes RAM contexts, ROM projections,
sentinels and unknown fields without inventing persistent unit identities.

<a id="finestra-recente-e-trigger-derrore"></a>

## Recent window and error trigger

recent-start SESSION [--events N --reads on|off --start HEX --end HEX --on-error on|off]
arms the ring log. recent-freeze SESSION stops collection; recent-report SESSION
saves in the paused copy. Recent status distinguishes window, eviction/drop totals,
stop and write; recent_report retains the latest path. [DBG-19](debug_recent.md)
describes reads and partial coverage, automatic save on error and limits.

<a id="verifica-dbg-20-del-29-ottobre-2026"></a>

## DBG-20 validation on 29 October 2026

The audit reuses completed DBG-10/11/12/16 results for pause/step, inspection,
capture, menu and moved profiles. It adds only actual gaps: queries while
waiting, targeted cancellation, budget deadline and manifest identity.

**30 distinct Wine tests passed** (16 protocol, 14 frontend, one frontend already
ignored): separate locks/responses, ID/run/session/schema, actions in wrong
channel, expired request/client timeout, late ack and retry of query response
only. GUI-test revision corrects the shutdown label from refused to closed;
only the affected protocol test is rerun on the final build, without repeating
GUI frames or the 30 completed tests.

**19 CLI invocations on the default Wine/Xvfb GUI**: query while a client awaits
step 600, wrong ID refused while retaining the budget, exact cancellation after
10 frames, stopped state and empty/muted audio. A 250 ms deadline returns an
unconfirmed client timeout; the subsequent query proves control stopped,
remaining 0/control-timeout and server response with the same ID. Readonly
protocol graphics capture, different manifest refused before publication,
shutdown with pending work and refusal on closed window verified.

The first GUI runtime remains authenticated in the copy; the only final
differences are closed-response classification, help spacing and its test.
No guest-loop or cancellation/deadline behavior change. Screenshot inspected;
six normal inputs/runtimes retain hashes/mtime. Sources, preimages, results,
reports and logs retained under **target/debug-tools-control/**; no new disposable
ROM fixture or old acceptance recreated. No network server added.

Windows GNU release feature/default locked/offline builds without warnings.
Native Windows/native Linux GUI, physical controllers, cancellation during native
execution and a real clock change remain unverified; the long 120 s timeout was not
waited again, with a new real 250 ms check. Search cancellation uses the same
return with interrupt_probe; the new GUI check focused on steps.
[DBG-20 record](changes/2026-10-29-complete-local-session-control.json).
