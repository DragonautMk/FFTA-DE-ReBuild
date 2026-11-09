<a id="contratto-delle-operazioni-di-debug"></a>

# Debug operation contract

DBG2-01 adds a browsable inventory of existing operations. Contract version 1
contains 110 operations and 376 parameters (65/242 in the first DBG2-01 unit,
plus catalog, selector and roles from DBG2-02/03/04/05/06): diagnosis, isolated
copies, catalog, recordings, replay/audit/capture/compare profiles, local
control, inspectors, traces, reports, retention, campaigns and checkpoints.

```sh
recomp debug-operations list
recomp debug-operations show control.step
recomp debug-operations show replay.create --json
recomp debug-operations list --json
```

These queries read only descriptors embedded in the build. They do not open
sessions, scan folders, load game data or start the listed tools. An unknown
ID, extra arguments or an unsupported form produces JSON with
`outcome: invalid-request` and exit 1. Valid queries return exit 0;
`--json` must be the last argument.

Each descriptor reports stable ID, title, source and handler, current CLI
syntax, required inputs, parameters and their constraints/defaults, allowed
state, effects, result and limits. `menu.current` identifies the existing
entry; `menu.destination` identifies the intended integrated-menu destination.
`menu.implemented: false` distinguishes a destination not yet implemented.
Inventory presence does not make an operation executable through new channels.

`parameters[].current_menu` distinguishes selected values, handler-fixed
values, missing controls and options irrelevant to the menu, such as the local
client deadline. `open_work` retains DBG2 IDs responsible for gaps. For example,
the menu allows steps of 1 or 10 frames with released keys; the CLI accepts
1…600 frames and an explicit mask. The parameter editor remains DBG2-05 work.

Operation IDs also appear in new UI-trace `host-action` events, local-control
responses and the pending command. The link is descriptive: menu and control
continue to use existing handlers and validation. Profile selection has the
distinct ID `replay.select`; starting is `replay.run`. Replay-kind IDs retain
the distinction between replay, audit, capture and compare.

Query format is `schema: 1`, `kind: ffta-debug-operations`, with
`contract_version`, `outcome`, `message`, `operations` and the result contract.
Execution results retain existing formats, described per operation. The future
shared request/outcome envelope is explicitly planned in DBG2-13/15; this is
not a new execution API. The local channel retains schema 1 and adds only
`operation_id` to result and pending status. Timeout and cancellation requests
still require checking status.

<a id="dialoghi-e-impostazioni-ancora-da-migrare"></a>

## Dialogs and settings still to migrate

The initial inventory had three automatic frontend sites: replay profile,
catalog and issue-report fallback. DBG2-04 replaces them with the internal page
and does not start the action after selection. Only
`content_picker.rs::Picker::browse_file` and `browse_folder` remain, through
explicit buttons. New migration cases remain under verification in the TODO.
Descriptors also report fixed values for code/gameplay/recent traces, audio
stream/frames, previous graphics comparison, Inspect PC address/length, issue
symptom and profile preparation. Host tools have an intended destination and
require explicit DBG2-06 adapters; a script name does not become a shell command.

<a id="verifica-e-manutenzione"></a>

## Validation and maintenance

Contract tests check valid queries and refusals, unique IDs and parameters,
destinations, declared gaps and menu/control correspondence. Rust enum matches
are exhaustive. Python checks compare parser flags, menu variants and dialog
sites with the inventory; flags passed to child processes are distinguished
from parent-command options. Update the descriptor and check when adding an operation.
Separate additive reference cards describe the shared break/watch window,
guard and hit-action options, the local-control replay frame cap, and the
typed-recipe reopening reason. The response still uses the existing
mode-specific operation identity. Historical Atlas contracts remain unchanged.

This unit's validation covers metadata and local protocol with synthetic
fixtures; it needs no ROM, replay or new scene checks. Actual build/test results
are in the record linked from the [DBG2 TODO](development/DEBUG_TOOLS_V2_TODO.md).
The selector, typed roles and internal parameters are available; console and
cheats follow the DBG2 TODO status.

`workbench.*` operations distinguish opening/editing a draft from verification,
presets, preparation and starting. Previous audio/code/gameplay/recent/graphics/
inspection/issue entries open parameters; their opening ID is separate from
`control.*`, which continues to identify the actual handler. DBG2-05 validation
is linked from the TODO; shared consoles remain DBG2-13/15.

IDs `tools.homepage`, `tools.campaign-report`, `tools.artifact-preview`,
`tools.recipe` and `tools.guest-scripts` identify the five tabs of
`Debug > Scripts and tools`. Details reads metadata/hashes; only Start launches
the declared adapter or readonly recipe. Cancel stops the supervised process
and retains outcome/log; the guest view only reads data.
See [the selector and library](debug_selector.md) for prerequisites and limits.

The [Clan Atlas menu](menu_clan_atlas.md) groups paths into eight Debug sections
and offers the readonly Menu / CLI reference page. The registry retains IDs,
parameters, handlers and protections; only menu paths change. Reference cards
use the selected menu language for their field labels and route presentation;
registry paths and the CLI/JSON descriptors remain unchanged. Descriptor prose
itself is English. The separate planned-task reference is generated from the
English checklist and reflects its recorded completion states and audit notes.

`cheats.open`, `cheats.choose`, `cheats.refresh`, `cheats.list`, `cheats.show`
and `cheats.info` describe local catalog browsing. `cheats.inspect-plaintext`
describes CLI inspection based on an explicit assumption; `cheats.inspect-seeded`
describes inspection with a caller-supplied key. Neither automatically verifies
device or encryption, nor enables codes. `cheats.inspect-source` requires
explicit provenance and master codes and can compare the checksum with a selected
ROM; the result remains advisory. `cheats.prepare-once` and `cheats.apply-once`
describe preview and manual state-bound application in an isolated copy; they
are neither maintained toggles nor verification of original-device effects.
`control.retry` describes explicit recovery of a request with unchanged identity.
Descriptors expose no new local-channel actions beyond those actually supported.
Selection by ID, asynchronous refresh and paginated catalog reads are available
through the local channel and console language; refreshing the menu view still
uses a separate path. Details in [debug control](debug_control.md).

<a id="linguaggio-debugger--dbg2-13-in-sviluppo"></a>

## Debugger language — DBG2-13 in progress

The single-line text parser supports quotes and hexadecimal numbers with a
`0x` prefix; it rejects ambiguous decimal counts and does not interpret a shell.
The experimental `recomp debug-command SESSION 'COMMAND'` CLI translates
supported commands into the same types and authenticated channel as
`debug-control`. `help control.step`, `status` and `step 1 --keys 0x3ff` were
verified under Wine; malformed input produces `parse-error` without mutating
the session. In the console the draft stays editable and Ctrl+Shift+C copies
the current error JSON, not the previous result. Distinct local failures and
possibly unknown delivery are described in [debug control](debug_control.md).
`profile
select ID` and `catalog select ID` are connected to the selector. A Wine/Xvfb
session verified synthetic-catalog selection and rejection of wrong kinds or
unknown IDs without changing state, frame or clock. Another Wine/Xvfb check
selected two distinct complete synthetic replay-profile IDs, one from the CLI
and one from the console, with correct hashes and unchanged state. Selection
checks references without starting replay; the full equivalence required by
DBG2-13 is not covered. `replay run` and the `audit`, `capture` and `compare`
modes now call the shared supervisor on the selected profile, with status,
outcome and cancellation by ID through the local channel. Instructions and
limits are in [debug control](debug_control.md). `content refresh` and
`content list` use the selector's same scan and index through the authenticated
channel: reads are paginated and can pin the generation. A Wine check on a
synthetic copy discovered and selected a catalog by ID without opening the
menu; details and limits are in [debug control](debug_control.md).

The selector has an internal preparation job keyed by ID: it reuses typed
reference checks, keeps the result pending until completion and discards
cancellations or changed contexts/catalogs. It is now connected to command
transport; the outcome remains pending until the job result. In a synthetic
Wine copy, a pending search rejected a wrong ID and was cancelled from the
console with the exact ID; another reached its deadline, distinguishing client
timeout from server confirmation. The corrected terminal response no longer
reports the completed job as pending; a new Wine check verified the result
and subsequent status. The `debug-control retry` CLI recovers a result with
the same run, ID, deadline and action: a Wine check confirmed that retrying a
completed step or pending search does not execute it again. The console and
`debug-command` accept `retry RUN ID DEADLINE -- COMMAND`; a Wine check confirmed
console recovery of the step result and rejection of changed parameters.
Per-run memory details and limits are in [debug control](debug_control.md).

A Command console window is in progress, opened from Debug / Overview in an
isolated debug copy. The draft supports cursor, selection, history, IME and
clipboard. Tab proposes names, options, values, content IDs and the pending
request ID; a row shows contextual syntax. With multiple suggestions, Tab
cycles them in the draft. Completion respects quoting and cursor position
and does not send commands. The panel shows the session label, ROM hash prefix
and run, along with pending-job ID and parameters; steps and searches show
their respective remaining budgets, while scans and selections show only
that they are running. Data comes from the frontend, updated at most four
times per second, without automatic channel requests or state hashing at every
update. Displayed identity is checked against the run manifest on opening;
long values may be abbreviated. Copyable `jobs status` JSON retains complete
pending-job ID and parameters.
A Wine/Xvfb check showed a search in progress and, after cancellation, the
on-screen keyboard in a 480×360 window. Submission uses the local-channel
client used by the CLI. Multiline pasted text stays in the draft and is
rejected by the parser until corrected. Under Wine/Xvfb, `step 1` produced
the same complete state as the CLI and the F9 shortcut remained isolated.
Closing the panel briefly filters synthetic focus-return presses; a traced
check verified reopening and ordinary Esc without altering the game. Another
Wine/Xvfb check with isolated prefix and clipboard verified selection copy/cut,
multiline Unicode paste and result-JSON copying. Submitting the multiline draft
was refused without publishing commands or changing game state. Real IME,
physical controllers and full equivalence remain unverified: these checks do
not close DBG2-13/14.

The console also offers an on-screen keyboard navigable by controller:
directions and confirm select characters, history, presets and actions;
`Send` submits explicitly. After leaving, the pad must return to neutral before
the main menu accepts new input. A Wine/Xvfb check with simulated XInput
verified the `status` preset, history, cancellation and closing while holding
confirm without changing state, frame or clock. Physical controllers remain unverified.
