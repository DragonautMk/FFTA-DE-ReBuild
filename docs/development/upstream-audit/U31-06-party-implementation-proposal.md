# U31-06: Proposed Android Party gallery touch unit

Status: **proposal for review; not implemented or approved**. This proposal
selects one original-game Party gallery on Android. Its source and measured
baseline are in the [Party comparison](U31-06-android-party-gallery.md). It
does not authorize a broad upstream merge. The project TODO requires approval
of the specific unit before modifying runtime code.

## User-visible contract proposed

When experimental touch selection is enabled and the verified EUR Party gallery
is active, a short tap on a visible member selects and opens that same member,
with the game's ordinary portrait, highlight and confirmation behavior. A tap
on the currently selected member opens it once. A tap outside all visible
member hit boxes while this gallery owns input is consumed without confirming
or affecting an underlying screen. In particular, the top tool strip is not a
member target in this unit; implementing its separate tools is future work.
The empty-area rule is a proposed behavior, not an existing or approved
policy. Unsupported ROMs and unrecognized scenes retain their existing paths.

The measured EUR host baseline is one 24-member gallery: pad Right then A
opens member 1; the fork's tap currently opens member 0, while upstream's
interpreter opens member 1. The upstream normal native cache can bypass its
own Party hook, so its implementation is not ready to copy. The user-facing
contract above is broader than that one observation and must be validated for
every supported visible target and relevant gallery state before acceptance.

## Proposed mechanism: observed pad navigation

Prefer driving the original selector with bounded directional button pulses,
then A only after the guest has published the requested selection. This uses
the existing input path instead of adding guest PC hooks, changing guest RAM
directly, modifying the native emitter or adopting upstream's roster module.
It is a design candidate: the only measured navigation so far is Right from
member 0 to member 1. Wraparound, vertical motion, disabled members, scrolling
and partially visible rows are not established by that observation.

1. Recognize the gallery only under the existing exact EUR ROM identity and
   touch-enable checks, a verified owner/task and child state, valid bounded
   pointers, count, selected member, visible geometry and unobscured ownership.
   Reject invalid, stale, ambiguous or unsupported state. Derive hit boxes
   from live member positions and scroll offset; clip to the actual visible
   world rectangle. Resolve overlap deterministically only when its intended
   ownership is demonstrated. A higher-priority modal or overlay owns input.
2. A recognized gallery tap has three outcomes: member target, consumed empty
   space, or unavailable/invalid gallery. The caller must not convert the
   first two into the existing generic A fallback. Do not globally change the
   fallback for other scenes. Process only the newest permitted tap under a
   documented queue policy, and never queue multiple confirmations.
3. Record a bounded request tied to the current gallery instance and target.
   At each eligible guest frame, check ownership, current selected index and
   input state; issue at most one verified directional pulse and release it
   before deciding the next step. Confirm exactly once only after the target
   is the live, stable guest selection and its dependent view has reached the
   chosen readiness condition. Any unsupported transition, timeout or lack of
   progress cancels without A. No geometric distance or arbitrary index
   arithmetic may be treated as proof of a legal pad route. The timeout,
   pulse/release durations, readiness condition and maximum work need evidence
   for the selected gallery; the measured 4-frame press plus 20-frame release
   is one successful example, not a universal setting.
4. Physical/gamepad input always takes ownership and cancels the request
   before a synthetic key can be applied. Cancel on Back, disabled touch,
   game pause/background, scene change, reset, load or ROM switch, and on a
   stale or replaced owner. Pausing must explicitly clear pending synthetic
   input: the existing Android worker clears queued taps while paused but does
   not run guest frames there, so a frame-only timeout cannot provide this
   guarantee. Cancellation must not itself emit A, B or a stale direction.
   If a gallery instance can close and reopen with indistinguishable pointers
   before cancellation, the identity scheme is insufficient: refuse or stop
   rather than confirm in the new instance.

The existing `ffta_battle_mouse::keys` cancels queued mouse work when physical
input is present and owns a button queue; `press_sequence` cancels existing
mouse requests before filling that queue. A Party state machine must be
integrated with that ownership rather than blindly appending a fixed sequence.
The Android worker drains tap events before the next `gbaFrame` supplies the
current physical key mask, so arbitration must be checked at the frame that
applies a pulse, not only when a tap is received. Existing queue bounds,
Notice/shop priority and single-pointer gesture rules must be preserved.

## Boundaries and alternatives

Expected affected areas are the Android `gba-core` input state/recognizer,
`gba-android` tap and cancellation boundaries, and, if necessary, the Android
worker/JNI pause path. Exact file changes depend on tests. English remains the
default; this unit requires no new user setting, save format, desktop behavior
or global key binding. The experimental touch option stays opt-in. Source
changes affect Android's compiler cache tag, so cold and warm cache tests are
required even though this proposal makes no emitter or native ABI change.

Upstream's direct owner hook is an alternative only if the pad route cannot
provide a complete and responsive contract. That alternative first needs a
separately reviewed native-boundary design covering direct intra-group jumps,
overlapping entries and interpreter fallback, not just the outer dispatcher.
The observed bypass is from upstream on host; it is not proof that every fork
hook is broken. The desktop fork's event-budget guards and Android's chain
gate have different semantics; neither should be replaced without independent
timing, cache, input and ROMDATA evidence. If the pad route cannot safely reach
all targets in the proposed gallery, stop and revise the unit rather than
shipping a silently partial selector or silently importing the upstream hook.

## Acceptance and stop conditions

- **Scene and geometry:** verify all visible members in representative EUR
  galleries, changed order/count, first/last and multi-row navigation, scrolling,
  overlap, clipping, screen resize/zoom and stable hit transforms; a tap must
  open the same member as an equivalent pad sequence.
- **Input safety:** empty and top-strip taps cause no confirmation; physical
  input, Back, touch disable, pause/background, owner changes, repeated taps,
  invalid or expired requests never leave a direction or delayed A behind.
  Test overlays, Notice/shop priority and unsupported/USA ROMs separately.
- **State and execution:** compare selected member, dependent portrait and
  highlight and resulting screen against pad behavior. Verify guest frame and
  clock behavior appropriate to a sequence of actual pad pulses; do not require
  identical timelines for gestures that take different numbers of frames. Test
  interpreter, normal cold/warm native cache and native group boundaries;
  preserve cartridge saves and existing Android input/recording behavior.
- **Platform:** run synthetic/core and engine tests, then the documented scene
  on an identified Android emulator or device with an isolated authorized
  save. Distinguish host, emulator, physical device, EUR and USA evidence; a
  build alone is not scene acceptance.

Stop before runtime changes without specific-unit approval, or during work if
the guest owner, correct target, pad route, cancellation, cache or platform
evidence cannot be established. No implementation of this proposal,
post-change application test or device acceptance is claimed by this document.
