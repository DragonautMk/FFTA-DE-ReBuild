# Extended saves (desktop debug option)

First unlock Debug in the pause menu with **Up Up Down Down Left Left Right Right**,
or set `debug = true` in `av.cfg` and restart. The unlock is persistent.
See [menu visibility](menu_clan_atlas.md#sblocco-permanente-di-debug).

Open the pause menu, choose **Game > Saves** (also available under
**Debug > Sessions and cases**), then enable **Extended saves** with
Confirm, choose ON in the selector, then Confirm again. The default is OFF. The choice
is stored as `debug.extended_saves` in `av.cfg`, including portable
`Data/config/av.cfg`.

Standard-save availability and metadata messages follow the selected menu
language, with English as the default and fallback. Save forms use English
source keys while retaining existing translated labels and compatibility keys.

The [debug scene checklist](development/debug%20saves.md) assigns stable
DS-01 through DS-26 names/numbers to test situations, with evidence and explicit
validated/partial/uncovered status. The local ignored `saves/debug-cases/`
collection uses those same snapshot-slot numbers; partial slots remain candidates.
The expanded local collection is `debug_saves/ready/`, with separate USA/EUR
folders, an Italian usage guide and a provenance/status catalog. Its `IN-NN`
numbers select imported standard saves; `DS-NN` numbers select scene snapshots.
The entire root `debug_saves/` tree is ignored, including original containers,
converted saves, derived checkpoints and local metadata. The checklist records
region-specific validation and the remaining untested internal File No.2 entries.
Generated standard saves use GEN IDs: **GEN-30** is Sprohm before Decision
Time; **GEN-31** has Decision Time plus Prison Break active and White Magic
assigned; **GEN-33** resumes Prison Break through the game's **Resume Battle**.
File No.1 loads map progress for GEN-30/31. GEN-33 requires confirming the
resume summary, the warning and final Yes; an actual battle load is verified.
Master copies remain under ignored `debug_saves/generated/`, separately from
the numbered working family. **NEW-32** is factory-empty memory for naming
snapshots: select standard slot32 before loading DS-23 or variants53/54.
Loading snapshots restores SRAM and may flush it to the active standard file.
The guide lists **18 validated, 6 partial, 2 uncovered cases**, plus menu-ready
variant snapshots 51–64 for USA plus 65 for EUR. The new user-provided EUR DS-20
checkpoint loads in slot 20 and reaches a failed dispatched-unit return through
the flee/Defeat/map transition; its original bytes are retained under a
descriptive ignored filename. Variant numbers stay distinct from DS IDs.

## Snapshot slots

The existing **Save states** page offers slots **1–99** while enabled, and
**1–9** otherwise. Confirm on Slot opens the picker; directions move the pending
choice, Confirm applies it and Cancel keeps the previous slot. Within this
picker Left/Right wraps, so Left from slot 1 highlights slot 99.
Previews, overwrite confirmation and the session's single Undo work as before.
Slots 1–9 remain in `SaveStates/<ROM-SHA-256>/` beside the normal cartridge
save. When the private debug-save Git worktree is available, slots 10–99 use
`<asset root>/debug_saves/runtime/SaveStates/<ROM-SHA-256>/`. The runtime
copies an existing legacy high slot to an empty new location without deleting
the original or replacing an existing destination. If both copies exist, the
private copy is used and the old one is retained for review.

Disabling the option hides slots 10–99 without deleting or renaming them.
The picker moves back into the 1–9 range. Undo save for a hidden slot requires
enabling Extended saves again. State format, ROM/BIOS checks and native
translation compatibility are unchanged. See [Save states](save_states.md).

## Standard game saves

With Extended saves enabled, **Game > Saves > Game saves** selects up to 99 cartridge
save files. Confirm on Slot opens the numbered picker; directions highlight a
number and Confirm applies it. Cancel discards the pending number. The page shows the filename, whether the file is saved, empty or
unreadable, and which slot is active.

Slot 1 retains its existing location and filename. Slots 2–99 add the suffix
`-slot-NN` and use the private debug-save worktree when available:

```text
Data/saves/ffta.sav                                      (slot 1)
<asset root>/debug_saves/runtime/GameSaves/<ROM-SHA-256>/ffta-slot-02.sav
...
<asset root>/debug_saves/runtime/GameSaves/<ROM-SHA-256>/ffta-slot-99.sav
```

An installed or explicitly selected `my.progress.sav` keeps its primary path;
the debug directory uses `my.progress-slot-02.sav`, etc. The two-digit suffix
`-slot-02` through `-slot-99` is reserved for numbering; reopening such a file
returns to the same family instead of appending another suffix. Legacy
numbered saves are copied without replacing an existing private version; the
originals remain available. Existing unnumbered saves are not renamed, moved or
converted. Files placed in the debug paths appear on the next menu refresh.

Choose **Game > Saves > Asset folder** to select the folder containing the
private `debug_saves` and `FFTARecomp-private-recordings` Git worktrees; restart
the game to apply the selection. The default is `../FFTARecomp-private`
relative to the project or portable application root. A selection on the same
filesystem root is saved relatively. Without the debug-save worktree the
runtime retains legacy save locations. An explicit `FFTA_SAVE_PATH` pointing
directly to a numbered save retains its legacy family for that launch.
Git shows changes but does not commit or push them automatically.

- **Copy current save** copies the cartridge memory to the selected file.
  Save using the game's own command first: this copies ordinary saved progress,
  not the exact current scene or unsaved play. The active slot does not change.
  Replacing an occupied file requires explicit confirmation, initially on
  Cancel, and first protects its old contents in its own backup folder.
- **Use and restart** asks for confirmation, initially on Cancel, flushes the
  current cartridge memory to its old file, and boots the game on the selected
  file in the same window. Resume, then choose the game's normal Load command.
  An empty slot starts with erased cartridge memory for a new game.

Selection validates the destination's size and prepares a clean machine before
changing the active save. A bad file, failed backup, bad boot state or failed
source write leaves the current machine and active slot unchanged. An existing
destination is backed up before loading; it is not overwritten by the old
machine. The normal autosave and exit paths subsequently use only the active
filename. Switching clears snapshot Undo and resets frontend input/audio/video
history like Restart ROM; it is refused during input recording.

Each numbered file has its own [two backups](save_backups.md). The first nine
snapshot slots are shared per ROM and save directory; higher slots are shared
per ROM and private asset root. Loading a snapshot also restores cartridge memory,
which may later be written to the currently active standard slot.

Portable mode remembers only `debug.game_save_slot = 1..99`, so moving the
bundle keeps the selection valid. Installed mode remembers the selected path
through the existing shared-save preference. `FFTA_SAVE_PATH` still overrides
startup selection. Disabling Extended saves keeps the active standard file;
it hides the selection tools and preserves every file.

The filename and size checks do not validate the game's internal checksums.
The original game still decides whether an ordinary save is usable. This
feature changes the desktop workspace only; AndroidPort is separate.

## Validation

On Linux, 39 configuration and 154 menu tests passed, along with 57 focused
runtime tests for saves, backups, state slots, asset references, sessions and
recordings. One optional test in each group was ignored. These include synthetic
legacy-copy and private Git visibility checks. An optimized Linux runtime also
resolved a synthetic sibling catalog before and after moving both folders. An
isolated Linux X11 session with synthetic data and a simulated folder chooser
verified the menu's relative selection, copying slot 10 and numbered save 2 to
the default debug worktree, and leaving the ordinary save and slot 9 in place.
Another synthetic X11 startup after moving sibling folders confirmed that a
persisted custom relative root is used for extended saves and startup backups.
The updated frontend has not been verified with a real desktop folder chooser,
real saves or native Windows.

The Windows checks below cover the earlier extended-save implementation, before
private asset routing.

The Windows GNU release builds of the affected tests and production CLI passed
without compiler warnings. Wine runs pass 29 input/config tests, 39 menu tests
and 22 targeted runtime tests (3 numbered-save, 10 snapshot, 7 backup and
2 save-path tests); one existing optional resource-backed backup check is
ignored. The tests cover high-slot undo/preservation, exact numbered paths,
custom names containing dots, failed switch isolation and overwrite guards.
Picker rendering and mouse hit geometry are checked from 240x160 to 3840x2160.

Three normal Windows/Wine GUI sessions with keyboard input verify default nine
slots, enabling Extended saves, snapshot99 save/load/Undo and overwrite cancel,
disabling without deleting it, copying cartridge progress to slot99, safe
switch confirmation, an empty slot's erased memory, returning to slot99,
restart and moving the portable folder with an unrelated working directory.
The screenshots were inspected and source ROM/save/snapshot hashes remain
unchanged. Evidence is local under `target/extended-saves/`, including test
logs, `windows-build.json`, GUI captures and `gui-results.json`.

These earlier GUI checks use Wine, interpreter execution and BIOS HLE, not
native Windows hardware or a physical gamepad. Real-save checks for the new
routing, the optional resource-backed check, release packaging and AndroidPort
remain unverified.
