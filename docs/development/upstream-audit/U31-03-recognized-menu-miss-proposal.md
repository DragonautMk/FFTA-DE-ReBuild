# U31-03/06: Proposed desktop recognized-menu miss guard

Status: **proposal for review; not implemented or approved**. This is a
candidate desktop input unit, related to the action-panel investigation but
separate from the proposed upstream panel geometry. The pinned upstream target
is `6bdddbc559ac459c4a18325c15c28d040d87f407`; the inspected fork `decomp`
HEAD is `83f245232c7dbd0206d4fea2a556e031e98fbf37`. See the
[action-panel audit](U31-03-panel-visual-fixes.md#follow-up-recognized-menu-miss-and-upstream-background-guard)
for the modeled comparison and its limits. The upstream TODO requires explicit
selection and approval of a specific integration unit before runtime changes.

## Proposed user-visible contract

When a verified battle menu is recognized in the desktop game and an unpaused
left click with a known window position does not hit a valid target, consume
that miss without converting it into a generic A press. A valid click must
continue to select/confirm according to the existing menu path. The proposed
rule does not change ordinary background confirmation when no battle menu is
recognized, nor keyboard, controller, right-click or paused-menu handling.
This is a candidate behavior, not an established product policy or a claim
that every unrecognized transition is safe.

The evidence is one EUR action menu. The isolated fork-core model observed a
click outside the layout queue A and change the subsequent state relative to a
no-click control; it did not deliver a window event. Upstream's background
confirmation guard blocks a recognized-menu miss, but its full recognizers and
coordinate transforms differ and were not run on the same scene. A later [Wine GUI baseline](U31-03-panel-visual-fixes.md#follow-up-isolated-wine-gui-click-and-no-click-control)
reproduced an off-panel click opening Move, with a no-click control and two
verified recordings. This one baseline does not establish event-frame
attribution or the partial-reveal behavior; do not label either fixed.

## Candidate change and boundaries

Owner: the session explicitly selected to implement this unit. Expected scope
is the desktop `crates/recomp/src/play.rs` left-button branch and its focused
tests. No Android, core hook, geometry, renderer, save format, native emitter
or recording schema change is assumed. The current working-tree versions and
concurrent ownership must be rechecked before editing; the relevant desktop
source files had no uncommitted diff when this proposal was prepared.

Immediately before the existing `game_mouse_aim` call, capture whether
`ffta_battle_mouse::MenuScene::read` recognizes a menu, limited to unpaused
input with a known position. If the existing aim call handles the click, keep
its result. If it returns false and that pre-aim menu was recognized, cancel
pending mouse work as required by the selected path and consume the miss
without queuing A. Otherwise retain the current fallback. Capture context before
aim because miss handling can cancel pending work. Recheck the actual handler's
priority and cancellation behavior during implementation: a recognized menu
may coexist with a higher-priority shop, world or facing handler. A handled
higher-priority action must remain handled, and a stale or false-positive menu
recognizer must not suppress an unrelated action. If that cannot be established,
stop and revise the scope rather than applying this predicate globally.

Do not silently add upstream's broader service, language, roster, map-award or
out-of-view conditions. Missing mouse position, partial menu recognition,
transparent pixels within a recognized layout and a click outside the game view
need their own evidence and explicit behavior decisions. The special case of a
paused window with no position is outside this unit; do not alter it indirectly.
The proposed predicate is not intended to fix text alignment, panel animation,
Party touch, or the existing shop-category replay issue.

The existing button queue holds a synthetic press over guest frames; physical
input cancels pending battle-mouse work. Current recording captures latched keys
and semantic mouse events before guest frame execution. A changed miss should
produce no synthetic A or new mouse selection event; valid recorded clicks and
pre-existing recordings must retain their replay semantics. Review the full
recording/replay path instead of assuming that a local boolean test proves it.

## Evidence and acceptance gates

1. **Baseline GUI:** in an owned isolated desktop session, load an authenticated
   EUR action scene and verify a real left-click miss, window and panel
   coordinates, resulting latched keys and guest result against no click. The
   debug frame-step interface suppresses game clicks; use a method that proves
   which completed frame received the window event. Capture window state and
   event/latch evidence. Do not reuse another owner's display or save paths.
2. **Focused implementation tests:** valid target and handled higher-priority
   input still work; a recognized-menu miss generates no A and cancels only
   intended pending work. Cover disabled mouse, scene disappearance, position
   availability, pause, physical-input arbitration and repeated/moved clicks.
   Cover fallback when no menu is recognized and ordinary background A where
   its existing behavior is needed. Negative cases must exercise actual event
   decisions and queued keys, not merely a copied boolean expression.
3. **Timeline and replay:** compare against a no-click control at matching guest
   frame/clock and full state after a miss; validate an intended hit separately.
   Record and replay a miss and a valid selection, checking key and semantic
   event order and terminal state. Verify a pre-existing compatible recording;
   do not imply shop-category replay is repaired.
4. **Platform and scope:** repeat the targeted GUI check with the changed build
   and confirm pointer/paint alignment. Exercise EUR and the existing USA path
   separately where fixtures exist, and interpreter plus normal native dispatch
   when affected. Report Wine versus native Windows, Linux, controller and
   Android as distinct evidence. Retain protected input hashes and isolate
   executables, caches, screenshots and saves.

The expected code change is small, but the GUI, recording and native/region
checks dominate the validation cost; no duration is established. The current
environment lacked an isolated X server and automated pointer tool. Do not
proceed by modifying the shared display or claiming modeled input as GUI proof.
Stop if a reliable owned GUI setup, stable frame/event attribution, menu
ownership or any preserved input path cannot be demonstrated. A successful unit
would address only recognized-menu misses, leaving the partial-reveal and
visual-geometry criteria open.

The Wine GUI baseline above supplies one positive reproduction and a no-click
control; exact event-frame attribution, a changed-build comparison and all
other acceptance gates remain open. The proposal remains unapproved.
