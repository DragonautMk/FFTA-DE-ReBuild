# U31-01: 150% fast forward

Status: **source comparison; ADAPT only after defining the stepping rule**.
Compared upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` with fork
`decomp` `83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02. See
[U31-02](U31-02-scrollbars.md) for the upstream tag distinction. No application builds, timing runs or game
sessions were executed for this audit.
Three isolated upstream module tests were run as recorded below.

## Upstream and fork behavior

Upstream `crates/input-config/src/lib.rs` parses floating-point `ff.speed` and
accepts 1.5, 2, 3 and 4; it maps old 1 to 1.5 and 8 to 4. The upstream menu's
`FF_SPEEDS` exposes those four values. `FrameBudget` accumulates fractional
frames in `f64`; `recomp/src/play.rs` applies it when fast forward is active.
The upstream source has a 60-tick budget test including 1.5 and an `ff.speed =
1.5` configuration round-trip test; its result is recorded below.

The fork uses `input_config::fast_forward::PERCENTS` (100–800 in steps of 20),
with `selected_percent`, `effective_percent` and integer-remainder `FrameClock`.
`AvConfig` separately parses old `ff.speed = 2 | 4 | 8` and new `ff.percent`
values divisible by 20, preserving old values and their serialization. A
standalone upstream `ff.speed = 1.5` is not accepted by this fork parser and
leaves the default speed in force; `ff.percent = 150` is also rejected. This
is not direct configuration compatibility. A malformed/unsupported entry is
currently ignored rather than necessarily reported to the user.

The fork menu enumerates `PERCENTS`, validates direct selections against that
list and adjusts the selected value by 20 in `menu/src/lib.rs`. Its runtime
resets `FrameClock` and audio buffers on a speed/activation change, computes
fast audio rate from the percentage, and uses `next_batch` for pacing. Toggle
and hold are XORed. These paths are already present and must remain coherent;
replacing them with upstream's floating-point configuration and budget would
change accepted legacy behavior.

## Conditional proposal

Adding 150 requires changing **all** applicable gates: percent validation,
selection normalization, visible options, pacing and serialization. A future
150% `FrameClock` schedule must total three guest frames per two pacing ticks
from a reset clock (for example `[1, 2]`), but the current normalizer maps 150
to its fallback, so the existing implementation does not do that today.
The plan proposes a 10-point-capable scale while retaining shoulder shortcuts
of ±20. Define the neighboring menu options and exact ±20 behavior from 150
before implementation; preserve all currently accepted percent values and the
meaning/round-trip of legacy 2, 4 and 8. Whether upstream floating `ff.speed`
should be imported is a separate explicit compatibility decision, not implied
by providing a fork-native `ff.percent = 150`.

Acceptance: synthetic tests for valid/invalid parsing, round trips, 150% and
legacy frame totals, accumulator reset on rate changes, bounds, dropdown and
shoulder stepping, hold/latch combinations, and silent/fast/normal audio.
Then exercise an isolated GUI and compare measured guest-frame pacing and audio
at 150%, including transitions out of fast forward. Treat controller, native
Windows and Android as separately scoped checks. Existing source tests for
other percentages are not proof of a 150% implementation.

A [specific desktop proposal](U31-01-150-percent-proposal.md) defines a
10-point choice grid and the behavior of ±20 shortcuts, with configuration,
dropdown, pacing and compatibility gates. It remains unapproved and does not
implement or validate 150% in the fork.


## Targeted upstream module validation

On 2026-11-02, `crates/input-config` was extracted from the pinned upstream
Git tree into a new temporary directory. Its unmodified `src/lib.rs` and
`directinput.rs` were compiled together on Linux with `rustc 1.94.0
--edition=2021 --test`, without a Cargo build or the application's other
crates. Three individually selected tests passed (one assertion group each;
21 other tests filtered on each run):

- `pad_gesture_tests::fractional_budget_and_window_preferences`: 60-tick
  budgets including 1.5 and a serialized `ff.speed = 1.5` round trip.
- `tests::fast_forward_toggle_and_hold_are_independent_of_multiplier`:
  activation truth table for 1.5/2/3/4 and the upstream `ff.speed = 1` mapping.
- `tests::av_volume_brightness_and_ff_multiplier_roundtrip`: default/round
  trip at 4 and upstream's legacy `ff.speed = 8` mapping to 4.

Temporary source and test binary are under `/tmp/ffta-u31-input-QaV3Ss` while
available. No ROM, private save, window, controller or audio device was used.
These results apply to the pinned upstream source on Linux; they do not test
the fork, its 150% behavior, full Cargo workspace, menu integration, actual
frame pacing, audio or Windows. Previously accepted fork tests were not rerun.


## Offline Cargo compilation

The complete pinned upstream tree was extracted into an isolated temporary
checkout, and `CARGO_TARGET_DIR=<isolated>/build cargo test --locked --offline
-p input-config --no-run` completed successfully on Linux with Cargo 1.94.0.
It compiled the actual workspace package and its unit-test binary but did not
execute the tests or build the application. The library emitted two
`dead_code` warnings for `normalize_centered` and `changed_pov` in
`directinput.rs`. Temporary tree: `/tmp/ffta-u31-cargo-SdoXlH` while available.
This supplements the three standalone selected test results above; it does not
establish a Windows build, full workspace, menu, audio or runtime acceptance.
