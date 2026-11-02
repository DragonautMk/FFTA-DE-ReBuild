# U31-02: Proposed shop-category recording replay repair

Status: **approved; isolated candidate under validation**. This unit repairs
an existing desktop recording incompatibility found while evaluating the shop
scrollbar. It does not implement the scrollbar. The
[U31-02 audit](U31-02-scrollbars.md#follow-up-real-usa-shop-recording-refused-by-verification)
contains the real USA Wine recording and the earlier EUR interpreter models.
The upstream TODO requires an explicitly selected integration unit before
runtime changes.

## Reproduced problem

A complete isolated USA shop recording changes category from headgear to armor
and contains `285 mouse:1025` followed by a key latch at the same frame. The
current `debug-record verify` rejects that input with `mouse target out of
range`. The producer uses values `1024 + slot` for shop category slots 0–5;
the `gba-input v1` parser accepts neither that range nor a shop-category replay
branch. In one EUR isolated interpreter model, applying category events through
the existing shop selection API together with recorded keys matched serialized
states and clocks over three spaced changes. That model bypassed the real
parser/dispatcher and is not acceptance of this repair.

## Proposed behavior and minimal candidate

The desktop recording and existing `gba-input v1` readers should accept the
already-produced values 1024–1029 and replay them as the corresponding shop
category **only in a valid recognized shop context**. Other previously accepted
inputs retain their meaning. Invalid values outside that exact range remain
rejected; a recognized token in an incompatible scene or invalid slot yields
an explicit replay mismatch, rather than selecting a battle menu or silently
succeeding. The original recording files are never rewritten to make them pass.

Candidate scope is `crates/recomp/src/main.rs`: teach `InputScript::parse` the
exact existing token range and place a bounded shop-category branch in
`apply_mouse` before the broader `index >= 512` branch, enabling the existing
replay mouse state and using `ffta_shop_mouse::select_category`. This order
matters: only widening the parser would otherwise route 1024–1029 through the
battle-menu branch, including a narrowing cast. Existing producer and token
numbers should remain unchanged unless a test demonstrates a necessary reason
and that change is reviewed separately. Inspect whether applying a category
request emits a new pending semantic event and whether that state can affect
later input; do not discard unrelated events or alter key ordering blindly.

`debug_recording::script`, recording verification, replay profiles and older
CLI input consumers share this parser or `apply_mouse`; assess each caller's
frame origin and behavior. The recorder latches keys before writing semantic
mouse events and key changes; verifier sets the recorded mask and applies mouse
before the frame. The modeled case covered that order for a few EUR changes,
not physical-input contention, rapid gestures, overlays or later item
selection. Invalid ROM signatures, stale owner and unavailable category must
remain safely refused. No change to Android, scrollbar geometry, native emitter,
save format or global input settings is implied.

The focused verifier also binds a recording to the executable hash. Therefore a
new executable must not silently alter or bypass that identity check to verify
the old USA recording. Preserve it as a negative baseline. Generate a **new**
recording with the selected new binary and verify that recording normally. A
separate read-only compatibility test may feed a protected copy of the old
input and associated matching machine state to the new parser/dispatch under a
clearly documented test harness, but must not rewrite manifest provenance or
claim that the old binary identity matched. An older executable may still
reject these already-produced tokens; document that limitation accurately.

Owner: the session explicitly selected to implement this unit, in an isolated
worktree/branch. Expected changed files are desktop `main.rs` and targeted test
modules; changes to shared core, `debug_recording` or other files require a
specific reason discovered during validation. At proposal time these files had
no tracked diff; check ownership and preimages again before editing. User ROMs,
saves and recordings are inputs, never committed or packaged.

## Acceptance and stop conditions

1. Parser and dispatch tests cover all six category slots, range boundaries,
   existing battle/world/language tokens, stable equal-frame order and malformed
   input. A shop event cannot enter the battle-menu branch. Wrong/disabled ROM,
   missing owner, overlay, stale state and unavailable category fail explicitly
   without an unrelated action.
2. Exercise the real core producer → actual parser → actual dispatch → guest
   frames, including same-frame recorded Right, repeated category changes,
   cancellation and physical keys, item selection after a change, and no extra
   delayed input or duplicate confirmation. Compare full serialized state,
   guest clock and relevant input/event outcomes against live interpreter
   execution at identical frame boundaries.
3. Start a **new** isolated USA GUI recording with the new binary, change a
   category and run the actual `debug-record verify`; require its final state
   and clock to match. Keep original ROM, source save, snapshot and the earlier
   failed recording hash-stable. Exercise EUR separately, and preserve a
   previously accepted compatible recording. Report Wine/Xvfb and native
   Windows separately; a simulated pointer is not a physical mouse test.
4. Run targeted core and desktop tests and check native cache/interpreter
   dispatch where replay uses native execution; the focused verifier is
   interpreter-only. Do not claim broad native, Android or all-shop coverage
   from an interpreter replay.

The code change may be small, but targeted replay, recording, platform and
compatibility checks dominate cost; duration is unknown. Stop before runtime
changes without specific approval, or if event ownership, deterministic ordering,
identity checks, invalid-context refusal or preservation cannot be established.
This proposal is distinct from the separate
[EUR shop scrollbar proposal](U31-02-shop-scrollbar-proposal.md).

A later [key-only comparison](U31-02-scrollbars.md#follow-up-key-only-comparison-for-the-recorded-usa-change)
reproduced the captured USA recording's serialized endpoint with and without
modeled event dispatch for the observed adjacent 0→1 transition. The current
verifier still rejects the original event. Acceptance must include nonadjacent
category changes and not infer that deleting semantic events is generally safe.

A later [nonadjacent USA recording](U31-02-scrollbars.md#follow-up-nonadjacent-usa-category-recording)
showed the key-only replay diverging from the actual endpoint, while a modeled
`1028` dispatch matched it. This supports retaining and correctly dispatching
the semantic event; it is not an implementation or full replay acceptance.

A further [recorded item-selection and cancellation sequence](U31-02-scrollbars.md#follow-up-item-selection-and-cancellation-after-a-category-change)
also matched its endpoint with modeled category dispatch, but diverged with
keys alone. The application verifier still rejects the event, and the model
does not exercise the actual parser or dispatcher.

## Isolated implementation and partial validation

The [recorded approval](U31-approved-decisions.md) permits this bounded unit.
On 2026-11-02 an isolated checkout at fork commit `c680b29517ac3dc99890a9d856122289c97f623f`
received a candidate that accepts only 1024–1029 and dispatches those values
to the existing shop category selector ahead of the battle-menu branch. An
invalid context returns an explicit shop replay mismatch. The main checkout's
runtime has not been changed or merged. The local candidate patch, harness and
logs are retained in `target/u31-shop-replay-repair-B9AOtX/`; the isolated
checkout is `/tmp/u31-replay-repair-R7vMxO/worktree` while available.

Linux desktop test compilation and six focused replay tests passed, including
new range/order and invalid-scene tests. The first compilation attempt lacked
ALSA development metadata; the successful retry used a local pkg-config file
and link to the system ALSA runtime. Five existing shop-core tests passed
serially. A separate release harness compiled the exact candidate
`InputScript` implementation with the core and replayed the authenticated
529-frame USA category/item/cancel recording: the final serialized state and
clock matched. At the recorded shop frame all six available slots were accepted;
an overlay, missing owner and duplicate category IDs were refused without
changing serialized state. Original recording inputs and copied ROM hashes
remained unchanged. One harness compile first failed on an out-of-range test
literal and was corrected; that diagnostic is retained. The successful harness
reported one unused `load` warning.

The harness also observed a pending `Some(1028)` recording event after replay.
The examined CLI verifier, replay worker and comparison each own isolated
machines; no live-recording handoff was established. Do not discard a possibly
unrelated event blindly: ownership and downstream effects remain a validation
gate. The actual newly built Linux application rejected a copied old recording
with `Recording was made with a different executable`, as intended; neither its
recording manifest nor input was rewritten. A first cross-platform session
attempt was rejected because the Windows session lacked the Linux `recomp`
pin; a freshly created Linux session was then used. A first debug binary also
exceeded the session executable size limit; the isolated copy was stripped of
debug symbols for this negative identity check.

The parser/dispatcher harness is not a successful application verification.
Fresh recordings with the changed executable and actual verification, EUR,
native dispatch/cache, physical-input contention and Windows/Wine GUI checks
remain outstanding. The current environment exposed Wine but no MinGW linker
or Xvfb in `PATH`; no Windows build or new GUI test is claimed. This candidate
is partial and must meet the acceptance gates above before integration.
