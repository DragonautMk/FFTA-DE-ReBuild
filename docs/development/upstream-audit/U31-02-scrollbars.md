# U31-02: Original-game list scrollbars

Status: **partial source audit; ADAPT candidate**, not an implementation or a
runtime acceptance. Audit date: 2026-11-02. This is one family of the P0 audit;
it does not approve a wholesale upstream import or complete the other families.

## Pinned inputs and scope

- Fork: `decomp` HEAD `83f245232c7dbd0206d4fea2a556e031e98fbf37`;
  the relevant source was inspected in the working tree. Existing unrelated
  uncommitted documentation is preserved.
- Upstream comparison target: `6bdddbc559ac459c4a18325c15c28d040d87f407`;
  common ancestor `fcc38cc46084793f29136419a8a15e6914a570ce`, confirmed
  from the local histories. The upstream tag `3.1` points to that ancestor;
  the target is five commits after the tag, including the cumulative v3.1
  content update and two merges. Do not attribute the target tree to the tag
  without distinguishing them. Release notes are claims, not test results.
- Upstream source was fetched into a separate temporary Git repository; no
  references or files in the project checkout were updated by the fetch.

## Upstream behavior and callers

[`ui_scrollbar::Bar`](https://github.com/Jorge262007/FFTARecomp/blob/6bdddbc559ac459c4a18325c15c28d040d87f407/crates/screen/src/ui_scrollbar.rs)
constructs a proportional thumb only for an overflowing list with usable
geometry; it calculates hit testing, grab offset, clamped drag position and
painting. On 2026-11-02 the exact extracted source was compiled independently
with `rustc 1.94.0 --test`; all four geometry/painting tests passed. This
exercises the self-contained upstream module only, not either application.

In upstream `crates/recomp/src/play.rs`, `roster_scrollbar` obtains the list
from `ffta_roster_mouse::scrollbar_list`, while `pub_scrollbar` selects
`pub_list`, `battle_list` or the shop list. The pointer handlers convert
window positions into their respective native or panel coordinates, and pass
the chosen first row to the relevant `scroll_to`. They run before underlying
roster/list click handling. Rendering paints the bar over a detached panel or
frame and invalidates the relevant panel cache. Mouse/keyboard configuration
controls visibility. This is more than a standalone geometry widget.

Upstream `crates/gba-core/src/ffta_roster_mouse.rs` recognizes several list
contexts and implements `scroll_to`; the gallery case writes several guest
state fields and schedules a non-confirming request, while other cases queue a
scroll request. Area List geometry is anchored to its window during row
animation. Upstream `ffta_battle_mouse` and `ffta_shop_mouse` supply other list
adapters. Desktop and `AndroidPort/` have separate roster source copies.
A follow-up inspection found no Android frontend caller of the desktop bar;
see the bounded proposal below. Other Android integration remains unverified.

## Fork comparison

The inspected fork has no `ui_scrollbar` module or `ffta_roster_mouse` module
in either tree. The Atlas menu has its own independent internal scrolling in
`crates/menu/src/lib.rs`; it is not a scrollbar in an original-game list.
Desktop `crates/recomp/src/play.rs` forwards an unpaused mouse wheel to
`ffta_shop_mouse::scroll`; `gba-core/src/ffta_battle_mouse.rs` has existing
menu selection, guest input hooks and replay-key handling. Its generic
`window_list` deliberately accepts only unscrolled instances. Thus a generic
scrollbar implementation alone cannot establish safe scrolling, and the
existing mouse paths do not establish equivalent scrollbar behavior.

Existing fork tests include mouse menu and replay-key tests in
`ffta_battle_mouse.rs` and a Pub window coordinate test in `play.rs`; their
presence is not proof that the proposed scrollbar path works. Native hooks,
recording/replay, ROM identity checks, UI panel layouts and the native render
cache require compatibility review for any selected list. No input recordings,
ROMs, game sessions, application builds or integrated tests were used in this audit.

## Proposed bounded follow-up

**ADAPT, conditional on a demonstrated list-specific gap.** First select one
original-game list with identifiable overflow, safe region-specific recognition
and a reproducible fixture or scene. Compare its adapter and input ownership
against upstream; do not copy the broader roster, battle, shop or Android
subsystems by default. Scope desktop and Android separately. Candidate file
areas are `crates/screen`, `crates/recomp/src/play.rs`, and only the selected
list adapter under `crates/gba-core`; the exact changes depend on the chosen
list and must be reviewed against the fork's debug and replay contracts.

Acceptance should cover no bar without overflow, proportional bounded geometry,
track/thumb hit testing and drag endpoints; click/drag without unintended
confirmation; stable positioning through resize, zoom and list animation;
correct focus, cancellation and controller/keyboard behavior; unchanged replay
and game input outside the selected list; EUR and existing USA paths separately
when supported. Use synthetic tests first, followed by a targeted Windows GUI
check with a verified isolated fixture, recording what was actually exercised.
Do not claim ordinary gameplay or Android acceptance from synthetic tests.
Stop if the list's ownership, ROM identity, render layout or replay semantics
cannot be established, or if evidence requires modifying unrelated systems.

Remaining P0 work: deepen the twelve initial family reports, including further
Android/list paths, build/test evidence and ROMDATA, native cache and debug
interaction for chosen integrations. No integration, performance claim or
platform acceptance follows from this report.


## Follow-up: EUR shop list and Android call sites

A bounded source search of upstream `AndroidPort/crates/gba-android/src` and
`AndroidPort/android/app/src/main` found no `scrollbar`, `scroll_to` or
`scroll_by` call. The Android core does include roster list and scroll helpers,
and its generic battle `aim`, `keys` and hook entry delegate to roster handling;
that is not a frontend scrollbar or proof that a finger can drag one. The
Android `GameSurface` enqueues completed short single-pointer taps rather than
continuous drag samples. This conclusion is limited to the searched pinned
tree; it does not rule out all other forms of list navigation. Desktop and
Android need independent pointer/gesture designs.

A concrete **desktop EUR shop inventory** candidate has narrower dependencies
than the roster families. The fork already recognizes both regional shop
profiles and handles wheel input by queuing the game's Up/Down key pulses.
Upstream's additional `ffta_shop_mouse::list` restricts direct page recognition
to the EUR inventory callback, validates cursor/list context and yields first,
visible and total rows. `scroll_to` queues a bounded target and neutral state.
The upstream desktop `pub_scrollbar_pointer` maps the panel coordinates, then
calls `pub_scroll_to` or the shop `scroll_to`; release and cursor-leave clear
the drag. During its direct shop redraw, upstream `entry` saves CPU registers
and CPSR, redirects execution to an original draw routine with verified bytes,
updates cursor state and restores the saved registers at the return hook. Its
`interprets` range prevents native execution during that request. It also
invalidates panel packing when the bar is painted. These are material guest,
native-dispatch and rendering changes, not a geometry-only patch. Existing
USA wheel/category handling must be retained; upstream's EUR-only direct list
is not a USA implementation.

Fork recording collects semantic battle/language events and latched key states
in `play.rs`/`debug_recording.rs`; headless replay invokes `replay_keys`. A new
direct page request must either be represented and deterministically replayed
under the existing versioned contract, or be explicitly refused during
recording until supported. Existing recorded category selection does not prove
that arbitrary drag events replay. Review exact ROMDATA/native resource and
cache interactions for the chosen callback before implementation.

**Proposed unit, not yet implemented:** use the current fork and pinned
upstream sources, owner the implementing session, and select only the EUR shop
inventory. Files potentially affected are desktop `crates/screen` for geometry,
`crates/recomp/src/play.rs` and input/recording paths, and the smallest needed
`crates/gba-core/src/ffta_shop_mouse.rs`/hook changes. This is an estimate of
scope, not a commitment to copy the upstream hook. First prove an overflowing
shop scene and compare current wheel behavior with desired drag; the exact
fixture, ROM identity and real GUI cost are still unknown. Required tests:
standalone geometry and pixel overlay; verified/invalid profile, empty/nonoverflow
list, both endpoints, stale/cancelled request and no confirm; mouse capture,
release, cursor leave and focus loss; transformed panel hit tests; replay or
explicit recording refusal; native/interpreter and USA fallback; unchanged
manual saves, ROM inputs and unrelated resource/cache state. Then use an
isolated Windows GUI scene and record actual cost and results. Do not infer
completion from the four upstream geometry tests.


## Follow-up: existing recording and dispatch constraint

A source trace found a pre-existing, separate shop-category replay mismatch
that must be addressed before claiming recording compatibility for a shop
scrollbar. When `ffta_shop_mouse::select_category` accepts a different
category, it emits `1024 + slot` for slots 0–5 through
`ffta_battle_mouse::take_event`. `play.rs::run_one_frame` can write that as a
`mouse:<number>` event to either recording path. However,
`InputScript::parse` accepts battle/menu events 256–543, world events
4096–42495 and supported one-byte language targets; 1024–1029 match none.
`debug_recording::script` delegates to that parser, and `export` and `verify`
call it. Thus a recording that actually contains such a category event will
be refused by these readers. This follows from the traced code and a synthetic
parser test; it is not a claim that every recording or shop interaction fails.

A standalone Linux harness compiled the **exact extracted** fork parser with a
minimal `LanguageMouse::valid_target` stub and `InputScript` container. It
asserted rejection of all six 1024–1029 values and acceptance of boundary
values 256, 512, 543, 4096 and 42495; one test passed with one unused-field
warning. All tested values exceed `u8`, so the stub cannot affect those
results. The temporary harness and binary are `/tmp/ffta-u31-replay-parse.rs`
and `/tmp/ffta-u31-replay-tests` while available; generated source SHA-256 is
`1a5399020f8646bf74e093819abf29383678b92f71c50dfc7cd05373f1fe2848`.
No actual shop recording, complete application, game or replay was run.
Even accepting the tokens would not suffice: `apply_mouse` has no matching
shop-category dispatch branch. A complete fix must test producer, parser,
replay application, key ordering and end-state equivalence together; preserve
existing input-file compatibility. This is an independent issue, not a
completed scrollbar implementation.

For native execution, `run_frame_native_traced` checks
`ffta_battle_mouse.interprets(pc)` before dispatching a compiled block and
uses `Machine::step` for matching hook addresses; `Machine::step` calls the
mouse entry hook. Upstream's shop page request adds an interpreter range and a
register/CPSR save/restore redirection into the original EUR redraw routine.
The fork's existing shop hook does not contain that page request/range. A new
range must be checked for entry inside a native block, correct frame/clock and
interrupt handling, rollback/cancel and interactions with ROMDATA-backed reads
and resource lifetime. The presence of authenticated runtime resources and
separate native entry lists is not proof of safe equivalence. No ABI or cache
change is established as necessary from this source trace alone.

## Follow-up: EUR overflowing shop and wheel baseline

On 2026-11-02 a bounded Linux interpreter probe used the catalogued EUR DS-18
checkpoint and the existing `target/rom-data-metadata10/shop-buy.input`
sequence. Its snapshot checksum matches both the catalog and the earlier F-05
slot-5 checkpoint. The verified snapshot preview is a world map, not a shop;
the catalog marks this EUR case partial. The prior ROMDATA evidence documents
this same route to Buy but does not show the detached list in its raw image.
No previous acceptance was reopened or a new fixture constructed.

Two fresh fork-core machines loaded the same authenticated EUR ROM and snapshot
and followed the recorded key sequence for 300 frames with the mouse path
installed and a 22-pixel side extension. At that point the validated shop
callback and list fields reported seven total items, four visible, first index
0 and page-relative selection 0; `MenuScene::read` reported four rows. Their
serialized states matched before the new input. This is a concrete overflowing
EUR list, not proof that every shop category or another region behaves alike.

Calling the fork's `ffta_shop_mouse::scroll` on one machine accepted six
successive down requests, 20 guest frames apart. The page-relative selection
advanced 0→1→2→3, then stayed at 3 while the first index advanced 0→1→2→3.
Six corresponding up requests returned the observed first index and selection
to 0/0. The no-input control stayed at 0/0 throughout. All twelve requests were
accepted; the result concerns these list indices, not full-state equality after
return, a purchase, a physical wheel or a GUI drag. The isolated release-mode
harness and trace are `target/u31-shop-pagination-0WzxAu/`; ROM, checkpoint and
input-sequence hashes were unchanged, and the build produced no warnings.

An earlier exploratory run waited until frame 600, after the route had entered
a different state; its scroll request returned false and both controls remained
unchanged. That is a correctly bounded negative for that later point, not a
wheel failure in the recognized list. The separate one-step check at frame 300
observed selection 0→1, and the twelve-request comparison above adds the page
boundary and return. These traces remain in `target/u31-shop-route-VFNYm2/`
and `target/u31-shop-wheel-og94zr/` respectively. Managed cleanup rejects these
U31 directories because it only accepts `target/rom-data-*`; evidence remains.

This narrows the U31-02 candidate: the fork already has effective wheel
navigation in this overflowing EUR list, while a draggable original-game
scrollbar remains absent in the inspected source. The value and correctness of
an added drag target still need an actual GUI comparison, transformed geometry,
scroll endpoints, selection/confirmation and recording/replay tests, and a
separate USA evaluation. The probe did not run the upstream application,
native dispatch, Android or a real window event and does not authorize a
runtime change.

## Follow-up: upstream adapter on the observed EUR list

A separate isolated Linux comparison replayed the existing 300-frame EUR route
only to capture the guest EWRAM, I/O and frame number at the recognized shop
list. It initialized the exact pinned upstream desktop `gba-core` with the same
verified ROM and those copied fields, rather than attempting to load the fork's
incompatible machine-state format. The upstream `ffta_shop_mouse::list` returned
`total=7`, `visible=4`, `first=0`, matching the fork observation. A direct
`scroll_to(3)` request returned true; its next neutral-key query remained
`0x3ff`, the list still reported first 0, and the copied EWRAM/I/O hashes were
unchanged immediately after the request. With mouse disabled the upstream list
was absent and the same request was refused. This verifies recognition and
request setup only; no upstream guest instructions or redraw were run.

Using those observed list values and the pinned desktop frontend's shop bounds
`[8,0,216,88]` as a **conditional** input, the exact upstream `Bar::new`
produced track `[210,16,214,84]`, thumb `[210,16,214,55]` and maximum first
index 3. A standalone geometry probe checked hit/grab and clamped top/bottom
mapping, and a thumb-grab round trip for all four first indices. It did not
establish that the full frontend selected that node, that the bar was rendered
at these screen coordinates, or that a pointer drag invokes a successful guest
redraw. Those are separate GUI and execution gates.

The input ROM, original snapshot and route file retained their hashes in the
fork capture; the upstream run also retained its ROM and transferred inputs.
The local isolated unit `target/u31-shop-upstream-fTl0sP/` contains private
copied guest memory, harnesses and logs and must not be committed or packaged.
The upstream core build emitted four unused-code warnings; the standalone
geometry build emitted one. No native dispatcher, replay, USA, Android,
physical input or upstream full-game acceptance follows from this comparison.

The managed cleanup helper rejected this U31 unit because it accepts only
`target/rom-data-*`; its private evidence remains isolated.

## Follow-up: USA shop wheel baseline and early recognition

On 2026-11-02 an isolated Linux interpreter harness loaded the catalogued
USA DS-18 checkpoint with the matching USA ROM. The checkpoint is at the shop
greeting. A bounded sequence of A presses and neutral frames reached a
recognized shop inventory after 119 guest frames. The validated USA callback
and menu reader reported seven items, four visible, first index 0 and selected
row 0. This is a separate USA observation, not a reuse of the EUR result.

The first attempt issued down as soon as that list was recognized. All six
requests were accepted, but the first did not move the selection in the next
20 frames; after six requests the first index was 2, not the expected 3, and
the endpoint assertion failed. ROM and checkpoint hashes remained unchanged.
The failed harness, trace and result remain in
`target/u31-shop-usa-6roLpp/`. This does not establish why the first request
was ineffective or prove an application-window defect.

A separate run repeated the bounded route and waited 40 neutral frames after
recognition. Both machines still reported the same 7/4 list and identical
serialized state before the new input. Six down requests, 20 frames apart,
advanced the selected row 0→1→2→3 and then the first index 0→1→2→3; six up
requests returned both indices to 0. The no-input control stayed at 0/0. All
12 requests were accepted. The isolated run passed without build warnings;
ROM and checkpoint hashes matched their preimages. Evidence is in
`target/u31-shop-usa-settle-GwDw9Y/`.

This tests the fork core's wheel adapter and guest list indices for one USA
shop scene. It neither sends a GUI wheel/drag nor establishes full-state
identity after returning, native execution, replay, every category or an
upstream USA scrollbar. In the earlier source audit, upstream's direct shop
list is EUR-only. The early recognition result is a reason to validate a
stable input-readiness condition when designing a scrollbar, not a proven
cause or a license to add a fixed delay universally. Both ignored units
contain private state and must stay out of commits and packages.

Managed cleanup rejected the successful U31 unit because the helper accepts
only `target/rom-data-*`; both the successful and failed units remain.

The [EUR shop scrollbar proposal](U31-02-shop-scrollbar-proposal.md) describes
one reviewable integration unit, including the already-working wheel path,
early-readiness uncertainty and the existing category replay prerequisite.
It is not approved or implemented.

## Follow-up: category event from the EUR shop scene

A bounded Linux interpreter comparison used the existing EUR route to the
recognized shop list at frame 300. Two fork machines had identical serialized
state before input. On one machine `ffta_shop_mouse::select_category` requested
category 1 of six from category 0 and emitted semantic event `1025` through
`take_event`. The core applied the original selection path: at four frames the
requested machine reported category 1 while the no-input control remained at
0, and those observed category indices remained distinct through frame 80.

The harness placed the emitted event into a minimal `gba-input v1` text and
passed it to the current `InputScript::parse` function extracted verbatim from
`crates/recomp/src/main.rs`, using the real `LanguageMouse::valid_target`. It
returned `mouse target out of range`. This combines a real-scene core producer
with the actual parser logic and confirms the previously synthetic range
mismatch for this particular event. It does not show that an application
recording was created, or test the full recorder, exporter or replay runner.
The no-input control and changed category are separate timelines; no complete
state equivalence is claimed.

The local ignored unit `target/u31-shop-event-PneDIy/` retains the harness,
minimal input text, trace and protected-input receipt. The EUR ROM, snapshot
and existing route input hashes were unchanged. The isolated build passed with
one unused-field warning. This is a recording-compatibility prerequisite for a
future shop scrollbar, not a fix or evidence that ordinary wheel events fail.
A complete repair still needs explicit approval, producer/parser/dispatch and
ordering tests, existing-file compatibility and a real recording/replay.

The managed cleanup helper rejected this U31 unit because it accepts only
`target/rom-data-*`; the isolated evidence remains.

## Follow-up: modeled category-event replay ordering

A bounded Linux interpreter comparison reconstructed the same EUR shop
category 0→1 transition in two fresh fork-core machines. At the recognized
frame-300 list, their serialized states matched. The first machine requested
category 1, then recorded its actual semantic event and latched key for each of
80 guest frames. In the second machine a small harness **modeled** a candidate
shop-event dispatch: at the recorded frame it enabled replay and called the
existing `select_category` for event 1025, assigned the recorded key and called
`replay_keys` in the same order used by the current headless verification loop.
No parser or application code was changed or executed for this dispatch.

The recorded first key was `0x3ef`; the guest category became 1 by the fourth
frame. Every one of the 80 completed frames matched the first machine's full
serialized-state hash and guest clock. Both machines started from the same
validated EUR snapshot and route; ROM, snapshot and route hashes were unchanged.
The local ignored unit `target/u31-shop-replay-model-BvUWG4/` retains the
harness and logs; its release build emitted two unused-code warnings.

This is evidence that one candidate dispatch/order can reproduce this **one**
transition in the interpreter. It is not a working input format: the current
parser still rejects 1025, the current dispatcher has no shop-category branch,
and the harness bypassed both. It also did not exercise an application
recording, other category changes, physical-input contention, later event
consumption, native blocks, USA or a GUI. A fix must validate those paths and
preserve old recordings rather than merely expanding the accepted parser range.

Managed cleanup rejected this U31 unit because it accepts only
`target/rom-data-*`; the isolated evidence remains.

## Follow-up: multiple modeled category events

A separate isolated Linux interpreter comparison extends the preceding
single-event model with the EUR category sequence 0→1→2→0. The same validated
shop route produced two identical starting machines. The live side requested
categories 1, 2 and 0 at relative frames 0, 80 and 160, respectively; it
emitted events 1025, 1026 and 1024 and recorded the latched key at each frame.
The replay-model side dispatched those semantic targets through the existing
core selection function and called `replay_keys`, while applying the recorded
keys. The category reached each requested target; all 240 completed frames
matched the live machine's serialized-state hashes and guest clocks. The ROM,
snapshot and existing route input hashes remained unchanged. The isolated
unit `target/u31-shop-replay-multi-wljogL/` retains the harness and result; its
release build emitted two unused-code warnings.

This extends the earlier **one-event** result to three spaced requests and a
return to the starting category. It does not execute the current parser or
application replay dispatcher, which still cannot handle these events. It also
does not cover rapid or simultaneous changes, physical input, wrong context,
subsequent item selection, native execution, USA or GUI. Serialized-state
matching is limited to the fields included by that format; it is not proof of
identical host queues or a complete recording/replay contract. No runtime change
or approval follows from the model.

Managed cleanup rejected this U31 unit because it accepts only
`target/rom-data-*`; the isolated evidence remains.

## Follow-up: real USA shop recording refused by verification

On 2026-11-02 an isolated debug session used the catalogued USA DS-18 snapshot,
matching ROM and cartridge save with the Windows GNU executable previously
built from the fork. Under Wine, private Xvfb and Vulkan llvmpipe, two Z-key
presses advanced from the shop greeting to Buy. The screenshot showed the
headgear category and its list. XTest then moved the pointer to client
coordinate `(185,25)` over the second category and sent a left click; the
subsequent screenshot showed the armor category and a different item list.
Pointer movement preceded the click, so this observation does not distinguish
hover selection from click selection.

The actual snapshot-relative recorder closed normally with status `complete`
after 407 frames. Its input contains `mouse:1025` and the `0x3ef` key at
relative frame 285. The actual `debug-record verify` command exited 1 with
`recording input: mouse target out of range`. This is a concrete application
recording that its own current verifier refuses, extending the earlier EUR
core/parser comparison to a USA GUI path. It does not show a successful replay
or establish that every shop recording fails.

The first attempt reached the shop but its test script encountered EOF while
waiting for a coordinate; the X connection closed, the owned supervisor PID
was absent on inspection and its recording remained incomplete. The corrected
attempt used a new isolated session and prefix. Original USA ROM, source save,
snapshot and test executable hashes were unchanged in both attempts. The
ignored `target/u31-shop-gui-al8X5b/` and
`target/u31-shop-gui-retry-QGxyPX/` retain the incomplete and completed
recordings, screenshots, scripts, logs and hash receipts; they contain private
game data and must not be committed or packaged. This is Wine/X11 with
software Vulkan and synthetic pointer/keys, not native Windows, Android or a
physical input test. It does not authorize a runtime change.

Managed cleanup rejected this U31 unit because it accepts only
`target/rom-data-*`; both isolated attempts remain.

The [focused shop-category replay proposal](U31-02-shop-category-replay-proposal.md)
sets out a bounded candidate and its recording-identity and regression gates.
It is separate from the scrollbar and remains unapproved and unimplemented.

## Follow-up: key-only comparison for the recorded USA change

A separate isolated Linux interpreter harness authenticated the input, start
and end hashes from the real USA shop recording described above. It loaded
the recorded start state into two machines and replayed all 407 recorded
frames. One applied the recorded key masks while ignoring `mouse:1025`; the
other modeled a shop-category dispatch for that event before applying the
existing `replay_keys` logic. Both reproduced the recording's exact serialized
end state and guest clock, and their serialized states matched after every
completed frame. The input, copied ROM, states and manifest remained unchanged.
The isolated release build emitted no warnings. The ignored unit
`target/u31-usa-recording-model-Q9QTtW/` retains the harness and receipt.

This particular recording moves from category 0 to adjacent category 1 and
contains the original Right key at the event frame; the observed result shows
that **in this case** those keys suffice to reproduce the serialized endpoint.
It does not establish that semantic events are redundant for nonadjacent
categories, rapid changes, native execution or other scenes. The actual
`debug-record verify` still rejects the original `mouse:1025` before replay;
the harness bypassed the parser and modeled dispatch and did not change the
recording or its executable identity. Matching serialized state does not prove
all transient host-side queues or GUI outputs match. Do not strip the event as
a general repair based on this adjacent-category observation.

Managed cleanup rejected this U31 unit because it accepts only
`target/rom-data-*`; its isolated evidence remains.

## Follow-up: nonadjacent USA category recording

A new isolated Wine/Xvfb session started from the same catalogued USA shop
checkpoint and the same Windows GNU executable as the preceding GUI test.
After the greeting and Buy selection, XTest moved the pointer and clicked the
fifth category at client `(328,25)`. The screenshots show the category changing
from headgear to the fifth category and the list changing to boots and related
items. As in the earlier test, pointer movement preceded the button press, so
hover versus click was not isolated. The completed 411-frame recording contains
`mouse:1028` and Right (`0x3ef`) at relative frame 289. The actual
`debug-record verify` exited 1 because that mouse target is out of range.
Original ROM, source cartridge save, snapshot and executable hashes remained
unchanged. The isolated Wine/Xvfb unit is
`target/u31-shop-nonadj-7gR1nI/`.

A separate Linux interpreter comparison authenticated the actual recording's
manifest, input and start/end hashes and ran its 411 frames twice. The
**key-only** variant ignored the semantic event; the **modeled-event** variant
called the existing shop category selector for event 1028 before
`replay_keys`. States matched through 291 completed frames and first differed
after frame 292. At the end, the key-only serialized state did **not** match
the recorded endpoint; the modeled-event serialized state and clock **did**
match. The two final guest clocks differed by three cycles. The recording and
copied ROM hashes remained unchanged; the release harness built without
warnings. Its isolated unit is `target/u31-nonadj-model-3GyDMP/`.

This demonstrates why dropping category events is unsafe for this recorded
nonadjacent transition, unlike the earlier adjacent 0→1 example. The
modeled-event run bypassed the current parser and dispatcher, and its endpoint
match is not a successful application replay or proof of identical host queues.
No runtime fix, native Windows, physical mouse, Android or all-category
coverage is claimed. Both ignored units contain private game data and must not
be committed or packaged.

Managed cleanup rejected these U31 units because the helper accepts only
`target/rom-data-*`; the isolated evidence remains.

## Follow-up: item selection and cancellation after a category change

Another isolated USA Wine/Xvfb session started from the same shop checkpoint.
After moving to the fifth category with the synthetic pointer, the recorded
inputs selected an item and then cancelled. Screenshots show the quantity
overlay for Battle Boots and the subsequent return to the list. Pointer motion
preceded the button press, so this does not isolate hover from click. The
529-frame recording contains `mouse:1028` and Right at frame 270, followed by
the A and B inputs. The actual `debug-record verify` exited 1 with `recording
input: mouse target out of range`. The original ROM, source save, snapshot and
executable hashes remained unchanged. The GUI receipt and screenshots are in
`target/u31-shop-item-mElF40/`.

An isolated Linux interpreter comparison authenticated the recording's input
and start/end hashes and ran all 529 frames. Ignoring the category event
produced a different final serialized state; modeling the existing category
selector before `replay_keys` produced the exact recorded final serialized
state and guest clock. The variants first differed after 273 completed frames;
their final clocks differed by one cycle. The recording and copied ROM hashes
remained unchanged. The harness and receipt are in
`target/u31-shop-item-model-zQwxBU/`.

This is evidence for this sequence, not a successful application replay: the
model bypassed the rejecting parser and dispatcher and does not establish
identical host queues or GUI output. Native Windows, physical input, Android
and other shop sequences remain unverified. No runtime fix was made. Both
ignored units contain private data and must not be committed or packaged.
Managed cleanup rejected both units because it accepts only
`target/rom-data-*`; their evidence remains.
