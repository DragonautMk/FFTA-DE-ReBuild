# Debug Tools v2 integration matrix

This is the evidence map for [DBG2-27](DEBUG_TOOLS_V2_TODO.md). It records
specific past outcomes and their boundaries; it is not a claim that every
combination or the current binary has been retested. An accepted feature does
not by itself complete the integration matrix. Use the linked records and
receipts to avoid rerunning accepted scopes. Preserve originals in every new
isolated check.

## Platform and input

- **Windows GNU under Wine/Xvfb, simulated keyboard, mouse and XInput: observed
  passes in selected GUI paths.** Selector search, draft cancellation, previews
  and favorites were exercised; the controller keyboard submitted status and
  closed with confirm held without changing the guest. The host-tool GUI used
  simulated XInput to cancel a running process and confirmed its descendant
  stopped. These are separate scenarios, not one full end-to-end matrix. See
  [selector](../changes/2026-10-29-add-integrated-debug-content-selector.json),
  [controller keyboard](../changes/2026-10-31-add-debug-command-controller-keyboard.json)
  and [tool cancellation](../changes/2026-10-30-add-declared-debug-script-library.json).
- **Linux native, simulated keyboard and pointer: observed passes in selected
  GUI paths.** A concluded one-frame recipe was refused without a reason and
  executed with one; the picker displayed the evidence and required a renewed
  review after editing the reason. Protected inputs and normal window closure
  were recorded. This is not full Linux integration coverage. See
  [concluded recipe](../changes/2026-11-08-require-reopening-for-concluded-recipes.json).
- **Native Windows: accepted by user waiver, not executed.** On 2026-11-20
  the user requested treating this criterion as verified despite native Windows
  being unavailable. This is an acceptance decision, not an observed test;
  Windows GNU/Wine outcomes remain separate from native execution.
- **Physical controller on Linux: selected menu paths passed.** A connected
  Generic X-Box pad moved the Debug selection down one row, opened the content
  selector and returned with confirm/cancel in paused synthetic sessions. Real
  device events were observed; guest state/frame/clock stayed unchanged and
  windows closed normally. The cancel test allowed a repeated press and does
  not establish which press caused the return. Other devices, dialogs and Windows
  remain unverified. An earlier navigation attempt timed out without input; the
  first confirm/cancel attempt timed out waiting for return. See
  [physical navigation](../changes/2026-11-20-verify-physical-debug-navigation.json)
  and [confirm/cancel](../changes/2026-11-20-verify-physical-selector-controls.json).

## Operations and expected equivalence

- **Read-only inspection and one-frame step:** a single EUR interpreter seed
  matched full initial and endpoint state, frame, clock, registers and result
  across local control, menu forms and command console. Inspection left guest
  state unchanged. Zero bounds and selected malformed commands were refused
  without guest mutation. This covers the recorded default inputs, not all
  commands or values. A Linux/Xvfb synthetic-seed comparison also matched the
  complete inspection and one neutral frame across these three channels, with
  protected inputs and normal closure. See [Wine channel parity](../changes/2026-11-04-verify-step-inspect-channel-parity.json)
  and [Linux channel parity](../changes/2026-11-20-verify-linux-debug-channel-parity.json).
- **Live audit-enabled watch:** an existing Linux audit-feature interpreter
  stopped after a synthetic 32-bit write with the expected address and value;
  the resulting state matched a separate next-instruction breakpoint control.
  Inputs were protected and both sessions closed normally. This proves one
  write case, not all access sites or configurations. See
  [Linux live watch](../changes/2026-11-20-verify-linux-live-watch.json).
- **Event console off/on:** ten controlled frames from a public synthetic seed
  produced matching complete state, registers, keys, frame and clock in Wine.
  A bounded synthetic burst recorded drops and evictions. The separate live
  audio observations and their earlier failed finding retain their stated limits;
  they do not establish audible quality or long-term cost. A new protected
  Linux/Xvfb pair used the same public synthetic seed for ten frames with the
  console off and visibly collecting: initial and final state hashes, frames,
  clocks, registers and keys matched, and both windows closed normally. The
  console image was reviewed after redraw (`dbg27-linux-event-console-redraw-wx0bn7tu`).
  See [console limits](../changes/2026-10-31-bound-event-console-update-work.json)
  and [Linux pair](../changes/2026-11-20-verify-linux-event-console-parity.json).
- **Cheats:** an active cheat is expected to change its declared target, so it
  must not be judged against an unchanged no-cheat state. A protected EUR/Wine
  matched pair applied CH-0320 for one frame: 144 first-record bytes changed,
  two subsequent records matched and the difference persisted after disarm.
  Two bytes lie beyond the previously examined candidate progress span, with
  meaning unknown. Other entries and scenes have their own bounded records;
  do not promote this to general device compatibility. See
  [mastery comparison](../changes/2026-11-18-compare-european-mastery-record-boundary.json)
  and [cheat limits](../cheats.md).
- **Cheat recording/replay:** Wine frontend evidence from an SL3 origin with
  observed ancestry reproduced the exact endpoint and clock; a separate
  frontend run refused an incompatible active set. The source-bound Linux test
  used a synthetic catalog and a protected EUR ROM. These results are distinct
  from no-cheat console equivalence and from original hardware. See
  [cheat-derived origins](../changes/2026-11-09-record-cheat-derived-origins.json).
- **Moved folder and error outcomes:** a Wine GUI reopened a moved portable copy
  with spaces and Unicode, restored valid choices and reported missing/changed
  references without launching a tool. A synthetic GUI audit reported an
  endpoint divergence; it was retained as divergence, not converted to a pass.
  See [portable choices](../changes/2026-10-30-persist-debug-content-choices.json)
  and [typed selection](../changes/2026-10-29-migrate-debug-tool-input-selection.json).
- **Recipes, reports and caps:** a Linux GUI tested an exact replay frame cap,
  including refusal below the required cap and completion at the required cap;
  a Wine frontend issue report connected a bookmark, selected recipe and active
  synthetic cheat. Its original runner failed a wording assertion; a separate
  retained-input audit and moved replay passed. See [bounded replay](../changes/2026-11-08-bound-replay-jobs-in-recipes.json)
  and [issue timeline](../changes/2026-11-09-connect-issue-timeline-and-recipe.json).

## Acceptance and limits

DBG2-27 is accepted for the selected scenarios mapped above, with the user's
explicit waiver of native Windows execution. The waiver supplies no Windows
runtime result. The checks do not cover every combination of operation, input,
controller or platform; in particular, the physical cancel test allowed a
repeat, and other watch types remain unverified. The synthetic GUI audit
endpoint divergence and earlier timeouts retain their recorded outcomes.
DBG2-20 remains suspended by the user; this acceptance does not reopen it or
claim its remaining parameter domains. DBG2-28 is assessed separately.
