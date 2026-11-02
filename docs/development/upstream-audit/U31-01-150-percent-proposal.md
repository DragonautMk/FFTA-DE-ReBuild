# U31-01: Proposed desktop 150% fast forward

Status: **proposal for review; not implemented or approved**. The
[source audit](U31-01-fast-forward.md) identifies the missing 150% setting.
The upstream TODO requires specific-unit approval before runtime changes. This
candidate retains the fork's integer percentages and does not import upstream's
floating-point configuration or change Android.

## Proposed interaction

Offer every multiple of 10 from 100% through 800% in the desktop speed choice:
71 ordered values, including all 36 existing values. Selecting 150% sets
exactly 150%, without enabling fast forward by itself. Retain the existing
20-point speed shortcuts and row adjustment: from 150%, down gives 130% and up
gives 170%; limits clamp to 100% and 800%. For example, 110% down reaches 100%
and a subsequent up reaches 120%; clamping at the boundary is not reversible.
The dropdown's own navigation moves between adjacent choices, hence by 10
points, and retains its existing wrap behavior. Toggle/hold XOR semantics and
the three audio modes remain unchanged.

The current menu opens a dropdown that scrolls to keep its selected choice visible;
its keyboard/controller navigation wraps, and its mouse wheel sends the same
up/down events. Source inspection alone does not prove usable navigation through
71 choices or correct pointer hit testing. Test scrolling and selection near
both ends and 150% at the minimum window size as well as a normal size. The
existing test that assumes option index 35 is 800% must instead locate the
value or use the new last index; index 35 would become 450%.

## Configuration and pacing contract

Accept `ff.percent` only for integral multiples of 10 in 100–800; retain the
existing ignore behavior for invalid values. Round-trip every accepted new and
old percentage. Preserve existing `ff.speed = 2`, `4` and `8` meanings and
serialization; do not start accepting `ff.speed = 1.5` as part of this unit.
The same admissible set must be used by parsing, normalization, choices and
pacing: otherwise a selection can silently fall back to 400% or be rejected
after a restart. Current releases ignore newly accepted values such as 150%,
so downgrade compatibility cannot mean retaining that setting; document this
explicitly without rewriting older configuration files.

For a freshly reset clock, 150% must produce frame batches `[1, 2, 1, 2]` and
three guest frames per two pacing ticks. Preserve clock reset and audio buffer
handling when activation, speed or audio mode changes. Fast audio should use
the selected percentage, while silent and normal modes retain their existing
contracts. A simple arithmetic check of the proposed integer grid confirms
that clamped ±20 stays within the 71-value set; it does not execute product
code or establish wall-clock pacing and audio quality.

## Implementation and acceptance gates

If this specific unit is approved, develop it separately against pinned fork
and upstream revisions. Expected locations are `crates/input-config` for the
list, normalizer and parser; `crates/menu` for the list, shortcuts and tests;
and desktop integration tests as necessary. Inspect ownership and current
preimages before editing. Do not conflate this with a replay, Android, device
mapping or autosave change.

1. Test all 71 valid values and malformed, fractional, off-grid and out-of-range
   inputs; preserve exact legacy parsing and serialization, including existing
   old configurations. Verify selection is stable after save and reload.
2. Test dropdown selection/cancel and its visible range at 100%, 150% and 800%,
   including pointer, wheel and keyboard/controller navigation. Test ±20 from
   150%, 110% and 790%, bounds and existing even-grid values. Confirm neither
   setting selection nor shortcut unintentionally activates fast forward.
3. Test 150% frame batches, cumulative totals, rate-change reset and all
   hold/latch combinations, plus the previous rates and all audio modes. Check
   the menu's displayed value agrees with the effective value after reload.
4. Run appropriate targeted tests and an isolated desktop GUI check for input,
   pacing transitions and audio. State the measured environment and limitations;
   Wine/synthetic input, native Windows, physical controller and Android are
   distinct evidence. Preserve original ROMs, saves and existing configuration.

Stop if percent validation and presentation cannot be kept consistent, if
the enlarged dropdown cannot be navigated safely, or if existing configuration,
audio or clock behavior regresses. No implementation, measurement or platform
acceptance follows from this proposal.
