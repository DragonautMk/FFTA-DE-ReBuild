<a id="eventi-comuni--dbg2-08"></a>

# Shared events — DBG2-08

Existing host observations use a single frontend publication point. When the
UI trace, recent log or console is active, a host event is created once and
delivered to active consumers. No observer invokes menu or local-control
handlers. With all consumers off, input/focus payloads are not constructed
and events are not serialized.

Each row retains `stage`, `data`, `frame` and `clock` and adds:

- `event_schema: 1`, `event_id` and `event_sequence`: shared identity and order
  in the same run. The sequence does not restart when another collector starts.
- `host_elapsed_ns`: monotonic time since host-clock creation; neither guest
  time nor a hardware-latency measurement.
- `session.run_id` and `session.manifest_sha256`: local-channel run and
  authenticated copy manifest; outside that channel the manifest is null.
- `category`, `severity`, `origin.producer`, `origin.source` and `observation`:
  declared family, level, observation point, context and granularity.
- `request_id` and `job_id`: null without an observed request. Menu navigation
  and action share context; local commands retain the actual mailbox ID.
  Steps and searches also report that ID as the job.

`sequence` and `host_ms` remain **collector-local** aliases for compatibility
with schema-1 reports. To merge two reports, use `event_id` and sort by
`event_sequence`. Shared time and sequence are assigned at host publication;
guest frame/clock can repeat or move backwards. Gaps may include uncollected
periods, events specific to the other collector and drops. This order does
not imply a guest-instruction history.

The shared channel observes already instrumented Winit tokens, changed XInput
and remapped polls, existing input consumption, bindings, menu dispatch with
before/after focus, shared actions and commands, frame/restore boundaries and
already routed errors. Identical polls are coalesced before collector delivery.
Keyboard repeats retain their flag. Recent observers of guest reads, dispatch
and recognition retain their granularity: `rom-read-batch` and `frame` are
frame aggregates, not an intraframe sequence. Read keys remain sorted and
bounded as before.

Accepted keyboard/pad navigation publishes `host-input-request`, then
`host-menu-dispatch` and, if the handler returns an action, `host-action`.
`host-action-result` means only return from the existing handler. It does not
certify acceptance of a procedure that may already have shown a menu refusal.
`control-result` distinguishes `accepted` and `refused`; acceptance does not
mean an asynchronous job has finished. Local control publishes
`control-complete` with the same request/job ID when the step/search budget
ends, including cancellation or timeout. The `status` query stays silent.

`status.event_coverage` and `event_coverage` in both reports expose the same
versioned map of supported, conditional, partial and missing producers.
`status.event_clock` exposes identity and next number. Missing hooks remain:
raw Gilrs/DirectInput, touch/pen paths absent from the frontend, all
uninstrumented lifecycle data, native-block internals, complete CPU history,
raw audio callbacks and the lifecycle of future maintained cheats.
An unregistered producer is marked `unknown/unregistered`.

Collector limits remain separate: [UI](debug_ui.md) retains an explicit prefix;
[recent](debug_recent.md) keeps a rolling window with eviction and oversize
counters. Payload is not duplicated in the envelope; metadata fits existing
byte caps. A full or frozen collector does not block the other. Starting
collection during a command may retain its outcome without the preceding
request; saving/freezing may exclude its later outcome. Export and redraw
neither produce recursive events nor execute commands.

<a id="verifica-del-30-ottobre-2026"></a>

## Validation on 30 October 2026

Windows GNU locked/offline build without warnings on 606 frozen files/46 host
overlays; 11 distinct Wine tests pass, including identity/fanout, nested
contexts, polling, UI/recent limits, one million events and local control.
The new GUI uses the same accepted public seed with a distinct runtime:
35 UI and 47 recent events, all 35 shared events identical except local aliases.
Three navigation dispatches produce one selector opening; accepted pause and
refused UI restart retain real IDs and outcomes in both reports. Complete
state/frame0/clock0 remain exact. GUI and Xvfb closed; pins and canonical
runtimes preserved. Native Windows, Linux and physical hardware not verified
by this unit; no DS acceptance or guest parity.

<a id="cambi-del-menu--dbg2-09"></a>

## Menu changes — DBG2-09

The menu exposes readonly semantic state separate from geometry: page, entry
ID and label, current value, dropdown, selected slots/availability, binding
capture, editor, content filters/confirmation/favorite, tool fields and actual
displayed feedback. A content or catalog ID remains distinct from its label,
even for identically named items. Ordinary control IDs use page/control
variant; tool fields use tool/key.

`host-menu-baseline` declares a new collection's initial state;
`host-menu-change` records only changed or removed properties, with previous/
current value, previous/current entry and cause. Each property produces a
separate row, so a multi-field update stays within the per-row cap. At most
96 properties: further fields are declared omitted. Long values retain
256 characters, count and complete hash to distinguish texts with the same
prefix. No slot image or guest dump enters the snapshot.

Observations occur after public inputs and existing handlers; editor and
cancellation early returns are included. Internal mutations not presented
between two input boundaries are not implementation history. Identical hover
emits no repeated changes. The renderer and its scroll cells are neither
called nor adjusted to read semantic state.

`mouse-input` distinguishes raw Winit buttons/wheel from menu delivery;
`host-pointer-dispatch` and `host-input-consumed` declare routing and
consumption. Mouse movement produces selection events only when selection
changes. Request context also follows the mouse queue through to the action.
`host-text-input` observes IME preedit/commit already received from Winit;
draft changes remain separate from committed fields. `binding-capture-result`
reports available tokens/configuration and capture outcome. Remapped polls
identify selected backend and device; simultaneous keyboard/pad sources remain
combined. Raw Gilrs/DirectInput tokens are not yet instrumented.

`host-action-refused` uses synchronous errors actually returned; other handlers
retain feedback in the property log and a `handler-returned` outcome.
Asynchronous job completion remains separate. No observer calls a handler again
to reconstruct its outcome. DBG2-09 results distinguish Wine/simulated XInput
from physical hardware and IME composition; code compatibility alone does not
verify the latter.

<a id="verifica-dbg2-09"></a>

### DBG2-09 validation

Windows GNU locked/offline without warnings, 609 frozen sources/50 host overlays.
Eleven distinct Rust cases pass in targeted runs; the incorrect XInput-filter
attempt that selected no tests is excluded from the count. Six GUI captures on
the same public seed contain 580 UI and 739 recent events: all 580 shared events
have equal identities and data, without evictions or oversize. Held confirm,
autorepeat, dropdown, hover, editor, inspection/refusal and bindings are
verified; complete state/frame0/clock0 remain exact. Prefixes completed before
test errors are retained; only missing criteria resume. Keyboard capture keeps
the existing vocabulary: G is assigned, F4 stays waiting.

GUI and isolated display closed; seed pins and canonical runtimes preserved.
The guide is in the public list and explicit Docker COPY/ignore rules;
10 Python tests on synthetic archives pass without generating a release or
rebuilding Docker. These checks do not establish native Windows/Linux,
physical hardware or real IME.

<a id="finestra-eventi-live--dbg2-11"></a>

## Live event window — DBG2-11

Open Event console in Debug's capture section, or use F9 when Debug is enabled.
`key.event_console` in `input.cfg` accepts F2, F3, F4, F5, F9, F10 or F12;
the default is F9. The console is a separate host window: opening it does not
pause the game. Hiding stops collection and retains the collected window;
reopening resumes collection. Events while hidden are not reconstructed.

The window has its own rolling buffer of at most 1024 events, 256 KiB of
serialized JSON and 8 KiB per event. It shows current interval, evicted and
oversize counts and collector status even while the view is paused. P freezes
or resumes only the view; arrows and Page Up/Page Down scroll, End returns to
automatic follow. C and L cycle category and level; `/` edits search, Enter
ends editing. Filters do not change the original buffer. Ctrl+C copies the
JSON view and Ctrl+S explicitly saves it in the copy's `Data/config/event-console`,
at most 32 files. Escape hides the window. Export retains canonical IDs,
filters, collector status and coverage declaration.
The window uses the selected desktop menu language (English by default),
including when reopened after changing that setting. Category and level
tokens, event data and exported JSON remain unchanged by localization.

A toggles coalescing of identical XInput/remapped polls: initially enabled;
disabling publishes every observed poll to active consumers and may reach
their caps faster. Guest-read aggregates remain per frame: there is no
intraframe instruction history here. Producers exclusive to the recent log
appear only while that collector is active. All includes categories actually
published, including `unknown`; it does not enable uninstrumented hooks.
The paused-view snapshot and live buffer are distinct and each remains bounded.

Windows GNU builds and synthetic Wine checks verify menu, shortcuts/focus,
filters, pause, scrolling, export and actual clipboard copying, with complete
state/frame/clock unchanged. Checks use Xvfb and simulated input; native Windows,
native Linux and physical devices are not verified. Limits and sustained-use
cost remain subsequent DBG2-12 work.

<a id="limiti-e-costo--dbg2-12"></a>

## Limits and cost — DBG2-12

The collector accepts at most 32 events per frontend cycle; those beyond budget
are counted separately from evictions and oversized events. The live buffer
caps at 1024 rows, 256 KiB of JSON and 1 MiB total JSON text plus search index;
each event has an 8 KiB ceiling. The frozen view shares immutable rows: it may
retain up to one additional bounded window while the live buffer rotates.
These are data/row limits, not exact measurements of allocator or graphical
window overhead. Drawing, at most four times per second, decodes only visible
rows; filters and export do not mutate the log. View pause, collection and
game pause remain distinct.

A synthetic Wine check of 128,000 observations in 2000 cycles counted 64,000
budget drops and respected limits while leaving the frozen view unchanged;
observed time is 426 ms for the test, not a game benchmark. Two isolated
sessions with the same seed and ten controlled frames produced identical
complete state, registers and clock with the console off and on. Measured
duration includes the client process and establishes no general performance cost.

If a snapshot disabled separate audio taps, the frontend uses the available
faithful mix for pacing/output without changing the snapshot or saved Enhanced
choice. A brief Wine observation with the console open and view frozen showed
bounded audio queues and no drop messages, as did the console-off comparison.
Perceived quality, physical hardware and sustained behavior on native Windows
or Linux remain unverified.

<a id="runtime-e-strumenti--dbg2-10"></a>

## Runtime and tools — DBG2-10

`runtime-event` adds family, phase, outcome and bounded details on the same
clock. `event_coverage.runtime_families` describes origin, frequency,
requirements and scope of 18 families. The cheat family is partial: the manual
handler publishes preparation, one-shot application and refusal; it does not
cover maintained toggles, deactivation or original-hook history. Hook
availability does not equal hardware validation or guest parity.

`start_observed` means a consumer was active at the source; it does not guarantee
record retention beyond consumer limits. Local-channel publication errors
retain only the validated pending response's context; successful status queries
stay silent.

The registry retains at most 16 active jobs, with original request/job IDs even
when the view changes. Cancellation and other interventions retain original
context and report `cause_request_id` separately. Steps and searches reuse the
actual local-channel ID; the same context surrounds existing ticks. The
registry neither starts nor repeats an operation. An accepted replacement
declares `superseded`/`rearmed` without turning it into completion.

Details are bounded to 4096 bytes; larger values retain prefix/hash and a
truncation flag. Outcome remains outside truncated details. Progress is emitted
only on change and at most every 250 ms. UI/recent samples exclude their own
counts and clocks to avoid self-generated progress. Code/gameplay/audio read
existing observer counters from the frontend: no new I/O or clock read in
audio callbacks, nor continuous whole-machine serialization. Baseline ROM
hashing occurs only at explicit collection start.

Save, Load, Undo, reset and backup declare request and outcome at actual handler
sites. Restoration is observed before freezing for discontinuity. Recordings
and workers retain their initial context; handler completion does not replace
job completion. Host results retain actual completed/cancelled/timeout/refused
outcomes without interpreting menu text or rereading files to produce the
event. Results for an old draft remain identified and do not replace the current one.

The replay worker retains up to 8 preparation phases: dataset request/attach
and native/cache input request/load, or not selected/refused. These are
prepared offline inputs; they do not establish new extraction or compilation.
Details per phase are bounded to 512 bytes, preparation publication is best
effort and nonblocking, and records are retained in progress/results and the
supervisor fallback. Readers of these reports have a 32 KiB cap. Worker host
sequence/time remain distinct from frontend publication; worker frames are
not frames executed by the main machine.

CLI history before frontend startup, arbitrary dataset/cache generation and
uninstrumented native/CPU internals remain outside scope. The unsaved recent
log continues to block a separate replay; the UI trace can observe it.
Shutdown emits intent in the already active buffer without implicit export:
save the desired prefix explicitly before exiting. The gameplay inspector
still requires the authenticated European ROM; a refused synthetic ROM is not
reinterpreted as a verified guest model.

Checks on the Windows GNU build under Wine include ten targeted cases and
session, step, status, recording, capture, recipe, cancellation and replay
interactions. Temporary control-response blocking produced errors associated
with the actual request ID; retry succeeded. The recording manifest points
to the existing `recording.json` file, separate from its directory. Replay
observed unselected dataset/cache phases; it does not prove private-input
loading. One controlled frame retained the expected complete state. After the
Clan Atlas revision, a new Windows build and ten Wine tests passed; menu
recording start/stop and a controlled step retain expected identities and
state. Native Windows, physical controllers and IME are not verified.
