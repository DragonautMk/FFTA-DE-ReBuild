# U31-08: Additional save cards and in-game UI

Status: **source comparison; do not import upstream save subsystem**. Upstream
`6bdddbc559ac459c4a18325c15c28d040d87f407` is compared with fork `decomp`
`83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02. See
[U31-02](U31-02-scrollbars.md) for the tag distinction and
[U31-07](U31-07-machine-autosave.md) for state-format incompatibility.
No build, new test, save or game session was run for this comparison.

## Upstream UI and actual storage semantics

The upstream desktop save screen is owned by `AutoSave::screen_active` and
`route_entry` in `crates/recomp/src/autosave.rs`, with input routed by
`play.rs::activate_save_card` and the save-screen navigation loop. The host
owns a three-card carousel: loading can show QuickSave and the original two
slots, additional `SaveN` entries appear when present, and saving presents an
add card. Manual added cards contain machine states and preview metadata,
rather than separate ordinary cartridge `.sav` files. Their state is prepared
on entering a clean world map and queued to the writer; `save_and_return`
waits for the matching writer result before queuing a load. The load path can
also invoke the original handler for an unconverted legacy card.

`shared_loader` and `is_saving` require the Europe profile and specific guest
callbacks. `route_entry` also writes guest fields and invokes the guest handler;
the host suppresses overlapping original input. Delete uses confirmation,
tombstones and a recoverable Deleted directory. Legacy conversion may archive
an original `.sav` once recognized entries have corresponding cards. These are
coupled UI, guest and persistence behaviors, not a cosmetic card selector. The
upstream source includes synthetic carousel, additional-slot restart and
save-entry phase tests; none was executed here. Android has a separate save
screen implementation; its lifecycle and UI were not independently audited.

## Fork comparison

The fork's [Extended saves](../../extended_saves.md) provides 9 or 99 manual
snapshot slots with previews and one Undo, plus an opt-in selection of up to 99
numbered ordinary cartridge files. The latter are separate paths; switching
validates and protects the destination, flushes the prior cartridge, restarts
on the selected save, and returns to the game's ordinary Load flow. This is
not the same operation as saving an upstream map checkpoint into an in-game
card. The fork's existing evidence record describes Windows/Wine keyboard GUI
checks, including snapshot slot 99, Undo and cartridge slot 99; it explicitly
excludes native Windows hardware, physical gamepad and Android acceptance.
Those checks were not repeated for this audit.

The relevant fork code is `crates/recomp/src/game_saves.rs`,
`crates/recomp/src/save_state.rs`, `crates/recomp/src/play.rs` and menu pages in
`crates/menu/src/lib.rs`. The selection UI is in the host pause menu, not the
upstream replacement of the original in-game save/load screen. Therefore the
higher number of fork slots does not establish UX equivalence.

## Decision and acceptance

**SKIP the wholesale subsystem; DEFER a separate in-game UX investigation.**
Before proposing any adaptation, compare a specific user journey and its
storage semantics, including original save/load, Resume Battle, empty slots,
previews and cancellation. Preserve ordinary `.sav` files, 9/99 snapshots,
Undo, backups, recording guards and ROM/BIOS identity; do not rename or
migrate originals as a side effect of an audit. A purely presentational
improvement must not silently turn cartridge saves into machine checkpoints.

Any later change requires synthetic input/selection tests, recovery and
interruption checks, paired file and machine-state preservation checks,
region-specific behavior and an isolated GUI/controller test. Validate
Android separately; no current real-game or cross-region card equivalence is
claimed. A concrete useful in-game UX difference and its acceptable semantics
remain unproven.
