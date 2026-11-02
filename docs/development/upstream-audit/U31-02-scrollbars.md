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
