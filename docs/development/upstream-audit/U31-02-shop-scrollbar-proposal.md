# U31-02: Proposed desktop EUR shop inventory scrollbar

Status: **proposal for review; not implemented or approved**. This is one
original-game list, not an upstream subsystem merge. The upstream target is
`6bdddbc559ac459c4a18325c15c28d040d87f407`; recheck the fork HEAD and the
working tree when selecting an implementation owner. The
[U31-02 audit](U31-02-scrollbars.md) records the source comparison, EUR/USA
wheel baselines, upstream adapter and limitations. The upstream TODO requires
explicit selection and approval of this specific unit before runtime changes.

## Proposed behavior

With desktop mouse input enabled, show a draggable scrollbar on the verified
EUR shop inventory only when the selected category contains more entries than
visible rows. Dragging its thumb or clicking the track requests a valid first
row; releasing it ends the drag. The selected item must remain coherent with
the displayed page and a drag must not purchase, confirm or select an unrelated
item. Ordinary wheel, keyboard and controller navigation continue to work.
Shop dialogs, quantity/confirmation overlays, an invalid or unrecognized scene,
and disabled mouse input do not expose an active scrollbar. The initial unit
has no Android implementation. USA retains its existing wheel and menu paths;
adding a USA scrollbar requires its own recognition and validation.

This adds direct page access rather than making an unscrollable list scrollable:
in isolated core tests both EUR and USA lists already moved with the wheel.
The concrete EUR sample has seven entries and four visible rows. Upstream's
list adapter recognized copied EUR fields and accepted a page request, but its
guest redraw and full frontend were not exercised. A USA request accepted as
soon as its list first appeared did not visibly move; after 40 neutral frames
wheel traversal succeeded. That observation does not prove the cause, and a
fixed 40-frame delay is not an approved readiness rule.

## Candidate implementation and dependencies

Owner: the session explicitly selected to implement this unit, in a separate
worktree/branch after checking concurrent work and current file preimages. A
minimal candidate is:

1. Add or adapt bounded scrollbar geometry under `crates/screen` (track, thumb,
   hit, grab offset, clamped position and painting). Reuse the fork's existing
   packed UI and cache invalidation rather than import the upstream renderer.
   Upstream's conditional shop canvas `[8,0,216,88]` and sample thumb are
   evidence for a design, not proof of the correct final fork window transform.
2. In desktop `crates/recomp/src/play.rs`, recognize the chosen list and route
   pointer press/move/release ahead of item clicks. Map the painted geometry
   through the same current panel transform, capture the drag until release,
   and cancel on focus/cursor loss, pause, scene/owner change or disablement.
   A drag or track click must not leak to generic A or underlying item input.
3. In `crates/gba-core/src/ffta_shop_mouse.rs` and associated mouse state,
   expose only an authenticated, bounded list description and a safe page
   request for this EUR context. Derive readiness and ownership from evidence;
   cancel stale or ambiguous requests without confirmation. The exact request
   mechanism remains to be chosen after evaluating the original guest behavior.
   Driving original Up/Down pulses requires a bounded, observable convergence
   policy and physical-input arbitration; upstream's direct redraw instead
   redirects guest execution and changes native dispatch boundaries. Neither is
   established safe by the current tests. Stop and revise if neither can meet
   the contract without unsafe hooks, writes, timing assumptions or native/cache
   regressions.
4. The existing category producer emits `mouse:1024..1029`, but the recorded
   input parser and replay dispatch do not support those values. Fix or explicitly
   block the affected recording flow as a separately reviewed prerequisite before
   claiming a complete shop recording/replay contract. Any new semantic event
   also needs a versioned producer/parser/replay design; simply accepting a token
   is insufficient. Existing recordings and ordering must remain compatible.

Expected concurrent files are `play.rs`, `ffta_shop_mouse.rs`,
`ffta_battle_mouse.rs`, a screen geometry module, and recording/replay files
if the selected approach requires them. At proposal time the first three and
`crates/screen/src` had no tracked diff; ownership may have changed since.
Check the worktree again before editing. Do not change the existing save, debug,
ROMDATA or cache contracts to make this unit easier.

## Acceptance and stop conditions

- **Baseline and readiness:** reproduce the recognized EUR scene in an owned
  isolated GUI; establish the visible list, paint coordinates and pointer
  mapping across scale/resize. Determine a defensible readiness/cancellation
  predicate, including early entry and transition frames. Do not use recognition
  alone as proof that a queued gesture can be applied.
- **Geometry and input:** no bar without overflow; proportional thumb and both
  endpoints; track click and grab offset; transformed hit tests; capture,
  release, focus loss and leaving the track; no leaked A or double action.
  Exercise empty/nonoverflow/invalid lists, stale owner, overlays, pause,
  disabled mouse and changing categories, plus wheel, keyboard and controller.
- **Guest result:** compare the chosen item and list page against an equivalent
  sequence of original inputs in the same scene, including first/last rows,
  scrolling back and subsequent explicit selection. Different-duration input
  sequences need an agreed comparison boundary; do not claim identical clocks
  for unequal numbers of guest frames. Verify frame/clock and unaffected guest
  state where equivalence is expected, including cancellation and interrupted
  requests.
- **Recording and execution:** verify the relevant pre-existing category-event
  issue is addressed or explicitly blocked; record and replay drag, ordinary
  wheel and subsequent item selection with matching terminal state and input
  order. Test interpreter and normal native cold/warm cache separately, including
  native block boundaries if any guest hook or redirect is introduced. Exercise
  existing ROMDATA-backed reads and preserve snapshots, cartridge saves and
  other protected inputs.
- **Platforms:** positive EUR shop GUI evidence; distinct USA regression
  evidence that its existing input still works, plus unsupported ROM and
  invalid-context refusal. Report Wine and native Windows separately. Android
  is a separate future unit, not accepted by a desktop result.

The likely source diff is modest only if existing input can safely reach and
stabilize every page; the readiness, replay, native and isolated GUI checks
are the dominant costs and their durations are unknown. Do not start runtime
changes or merge without explicit unit approval. Stop if a safe deterministic
page request, nonconfirming ownership, replay contract, protected fixture or
owned GUI environment cannot be established. The existing modeled evidence is
not a post-change acceptance result.
