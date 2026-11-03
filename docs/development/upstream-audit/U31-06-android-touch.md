# U31-06: Android touch selection

Status: **partial source audit; ADAPT candidate for one verified prompt**.
Upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` is compared with fork
`decomp` HEAD `83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02.
This report traces the two-answer dialogue path; it does not claim coverage of
all scenes or approval of a runtime port. The upstream `3.1` tag and the
comparison target differ as explained in [U31-02](U31-02-scrollbars.md).

## Event path in both trees

`AndroidPort/android/app/.../GameSurface.kt` accepts a completed single-pointer
short gesture only while the game is active and the relevant touch option is
enabled. It rejects cancelled/multitouch gestures, maps against the displayed
view, caps the tap queue at eight and sends normalized view coordinates. The
worker drains the queue through `NativeBridge.gbaTap`; `gba-android/src/jni.rs`
locks the engine and calls `Engine::tap`. The fork's
`AndroidPort/docs/EXPERIMENTAL_TOUCH.md` documents this path as experimental,
off by default, with unsupported targets falling back to A.

The pinned upstream surface also permits naming and save-screen interactions
when the experimental option is off, with separate conditions. That wider
behavior is outside the proposed dialogue unit. Do not treat tap delivery,
input overlay exclusion or a passing APK build as proof that a guest choice was
selected correctly.

## Upstream two-answer prompt

Upstream `AndroidPort/crates/gba-android/src/touch.rs` checks
`ffta_battle_mouse::DialogueChoice::read` before the inactive menu underneath.
For a recognized choice it uses the captured dialogue/notice panel and host
layout, including motion, or the world rectangle if no panel is found. A hit
selects the matching answer; an empty-space tap returns without generating A.

The recognizer in upstream `AndroidPort/crates/gba-core/src/ffta_battle_mouse.rs`
requires touch enabled, two exact ROM instruction signatures, an EWRAM pointer,
text state 9, valid selection fields and on-screen geometry. Bounds derive from
the original cursor fields. `select` rechecks ownership, writes the target
selection field, and either queues a local dialogue-answer request or, for
`PanelKind::Notice`, uses the existing A-pulse path. The local answer is
consumed only when the expected guest PC, registers, owner and deadline match.
While an answer request exists, `State::interprets` routes the relevant ROM
address range through the interpreter rather than a native block. Exact ROM
signatures do not by themselves establish which fork region/revision is
supported. Changes here affect guest state and hook routing, not just UI layout.

Upstream's synthetic `dialogue_answers_select_the_touched_option_and_ignore_empty_space`
test calls `Engine::tap` with constructed signatures and memory, then checks
selection and no leaked A at that point. It does not execute a complete real
dialogue or prove native/physical-device equivalence.

## Fork comparison and a concrete candidate

The fork has no `DialogueChoice` recognizer or corresponding local-answer
request in its Android core. Its `Engine::tap` maps to the existing semantic
mouse or battle handler and falls back to a guest A pulse if unhandled. The
fork **does** already recognize binary Notice menus: `MenuScene::notice_menu`
identifies visible choice bullets, `aim` queues selection, and the existing
synthetic `notice_choices_follow_repeated_bullets_and_confirm_after_selection`
checks cursor selection and delayed confirmation. The generic touch fallback
still matters when a tap does not hit a recognized target. Notice support
therefore cannot be equated either with complete two-answer dialogue coverage
or with no existing support.

A useful next investigation is one **non-Notice, two-answer dialogue** with a
known ROM identity and reproducible input state, to establish whether the fork
currently selects the touched answer or merely confirms the existing one.
Compare the captured panel's actual transform with its hit-test coordinates;
upstream's `ffta_ui`, `host_layout` and `ui_motion` do not exist as drop-in
fork equivalents. Assess safe guest-input or existing semantic-selector options
before copying direct EWRAM mutation or new PC hooks. Preserve fork EUR/USA
profile gates, native/interpreter behavior and the existing Notice route.

## Evidence and acceptance required

The fork's existing documentation records historical touch engine/core/Kotlin
and device checks, scoped there to general event delivery and other behavior;
full manual touch coverage is explicitly unclaimed. The fork source includes
synthetic Notice tests and disabled-touch/coordinate tests. Upstream contains
the synthetic dialogue test described above. These were inspected, **not run**
for this audit; no current device, emulator, real game, platform build or
replay was exercised.

A proposed implementation must first document the scene, ROM signatures,
current and expected behavior, ownership boundaries, coordinate transforms and
how the target action matches pad input. Require synthetic positive and
negative coverage for both choices, empty area, missing/changed signatures,
stale ownership, cancellation, disabled touch and overlapping Notice menus;
verify no underlying selection or unwanted A. Exercise scaling/zoom and
panel transitions, guest state/clock effects and native versus interpreter
hooks. Then test the selected scene on a named Android device or emulator with
an isolated authorized fixture, separately reporting region and limitations.
Preserve controls, pause and tap-queue isolation; do not infer broader Android
scene coverage from this one test.

The [Party gallery comparison](U31-06-android-party-gallery.md) traces one
additional Android scene and its interpreter boundary. A paired host test of
the actual Android `Engine::tap` demonstrates different member selection in
one EUR gallery. A targeted Linux native-dispatch check also exercises the
upstream hook, but a subsequent normal warm-cache host run failed its
six-frame selection assertion. A bounded observer found the pending request
and execution of the native hook instruction without a dispatcher hook call;
Android ARM64 and device behavior remain untested.
Remaining U31-06 audit: other roster/Party, Area, shop, battle, naming and
scrolling paths, independent Android lifecycle and UI transform details, as
well as broader scene evidence. Those paths are not accepted by this report.
