# U31-09: Steam Deck Y menu shortcut

Status: **source comparison; DEFER device-specific default pending conflict
policy**, with contextual configuration as a possible adaptation. Compared
upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` with fork `decomp`
`83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02. See
[U31-02](U31-02-scrollbars.md) for the upstream tag distinction. No tests or
physical device checks were run.

## Upstream behavior

Upstream `crates/recomp/Cargo.toml` defines a `steamdeck` feature. On Linux,
`play.rs::linux_menu_shortcut` accepts configured menu control, Start+Select,
or Gilrs North when the **binary is built with that feature**. This is a
build-wide condition, not evidence of runtime detection of a physical Steam
Deck. The source test `steam_deck_y_shortcut_is_build_specific_and_mapping_is_present`
checks the conditional helper and bundled Linux mappings for Deck and Steam
Virtual Gamepad. It does not establish Steam Input's actual active mapping,
controller ownership or collision-free operation on hardware.

## Fork behavior and conflict

The fork's input config defaults `pad.menu` to `Select+Start`, supports
persisted configurable combinations, and permits North among recognized
controls. `play.rs` evaluates configured shortcuts and Mode, uses latching,
blocks gameplay/zoom/fast-forward during binding capture or menu shortcut
ownership, and suppresses menu/game input during the focused command console
handoff. The Atlas uses North/Y to switch Debug views when appropriate, as
`docs/menu_clan_atlas.md` describes. North can also be assigned by the user to
a gameplay or fast-forward control; existing input-config tests explicitly
exercise a North fast-forward binding. A build-wide extra North shortcut would
therefore risk reopening/closing the menu, stealing Debug input or consuming a
user binding unless context and precedence are resolved. The fork currently
has no upstream `steamdeck` feature in its `recomp` feature list.

## Conditional proposal and acceptance

First define a persisted, opt-in or context-specific mapping and how an
identified physical Deck, a Steam virtual controller and other pads are
handled. Honor an existing user binding and make shortcut precedence explicit
for gameplay, paused Atlas/Debug, binding capture, command console, fast
forward and other host controls. Do not assume the same raw button mapping for
all Steam Input configurations; avoid changing the current defaults as part of
a comparison-only audit.

Use synthetic edge/latch/capture tests for simultaneous and held buttons,
Y-assigned game and fast-forward bindings, menu open/close, Debug view,
controller disconnect and focus transitions. Then verify both a Deck physical
controller and a declared Steam Input virtual mapping, plus a non-Deck pad,
with actual mapping/configuration recorded. No current physical-device,
SteamOS or native Windows result is claimed.
