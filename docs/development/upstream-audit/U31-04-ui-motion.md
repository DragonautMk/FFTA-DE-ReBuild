# U31-04: Host panel transitions and smooth zoom

Status: **partial source audit; ADAPT conditionally**, not an implementation
or acceptance. Compared upstream `6bdddbc559ac459c4a18325c15c28d040d87f407`
with fork `decomp` `83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02.
See [U31-02](U31-02-scrollbars.md) for the upstream tag distinction. No tests,
builds or game sessions were run.

## Upstream behavior and actual callers

`crates/recomp/src/ui_motion.rs` maintains a host panel state machine using
`Instant` and smoothstep interpolation: nominal transition time is 0.15 s,
with 0.22 s for a dialogue exit. It retains the last complete packed pixels and
node during selected exits, translates the layout, and controls whether the
panel accepts input. For example, name entry is interactive only when open;
other panel roles allow input during entry and not during exit. Its `Motion::sync`
uses UI node roles, guest window phase/owner, stage and selected-character
information. `BattleHud` is a separate host state machine in the same module.
`play.rs` calls these after guest frame execution and uses motion-adjusted
layouts for both presentation and mouse targeting; it resets them after
upstream state restore. Android has its own `ui_motion.rs`, engine sync and
`touch.rs` consumers, which need a separate review.

Synthetic source tests include complete-canvas preservation while a dialogue
contracts, entry/exit input ownership and matching layout/hit coordinates.
Those tests were inspected but not executed. They do not prove rendering,
interaction or behavior with the fork's debug/record/replay and UI caches.

Upstream desktop zoom is a **separate** transition in `play.rs`: it interpolates
the displayed zoom over 0.20 s, while allocating the maximum extent of the
transition for the underlying renderer and updating view/crop. Its `sync_view`
also sets core extra dimensions and dialogue view fields. It is therefore
incorrect to assume that the entire zoom change is merely a host overlay or
that guest/full-state equivalence follows from the panel module's stated
host-only intent. Measure the actual fork state and rendered extent before
making any parity claim.

## Fork comparison and risk

The fork currently derives panel layouts directly in `play.rs` from live
`DialogueState` and current pixels, and has no upstream `ui_motion` module or
scene-node/window-phase model. Its `sync_view` applies the target zoom directly
and updates extra dimensions, temporal buffers and dialogue view values. Its
own exact-input panel cache, GPU revision tracking, temporal rendering,
state-restore boundary, input routing and debug traces already have independent
semantics. Importing the upstream module wholesale would couple to different
scene, renderer, input and lifecycle APIs. No general lack of visual
smoothness or real-game defect is established by source comparison alone.

## Conditional proposal and acceptance

Select one observed panel and first capture its current entry, exit and input
behavior. Design an optional, disableable host presentation transition that
preserves the chosen fork scene's ownership and complete-canvas rules; define
what happens on pause, restoration, focus changes, rapid reopen, modal overlap
and repeated input. Reuse the same transformed layout for rendering and hit
testing. Treat zoom as a distinct unit because render extent, temporal history
and PPU-facing view values differ.

Synthetic tests should exercise phase changes, irregular host times, clipping,
alpha, resize, cancellation and transformed hit tests, including disabled mode.
Then compare a targeted scene frame-by-frame with and without the option,
checking guest state/frame/clock and controller/mouse behavior. Verify state
restore, recording/replay and native paths independently; test native Windows
and Android separately where claimed. Performance and frame pacing require
separate measurements if implementation changes them. No input or visual
coverage beyond the exercised scene should be inferred.
