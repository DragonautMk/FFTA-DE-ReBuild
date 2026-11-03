# Configurable desktop menu shortcuts

For the desktop portable editions, settings are under `Data/config` and the
cartridge save is `Data/saves/ffta.sav`, with backups and state slots beside it.
See [portable storage and migration](portable.md). Installed-mode paths below
apply when `portable.txt` is absent.


The default keyboard shortcut is **Escape**. The default gamepad shortcut
is **Start + Back/Select** on XInput, DirectInput and standard gamepads.
Guide/Mode remains an additional way to open the menu where the OS exposes it.

The [native Clan Atlas](menu_clan_atlas.md) keeps the current shortcut visible
in the lower-left sidebar and adds menu-only shoulder shortcuts for volume and
Fast Forward.

While the menu is open, Tab (or pad West/X) switches sidebar/content focus;
Shift+Tab (or North/Y) opens the Debug reference view. Selectors keep local Tab
navigation and use Ctrl+Tab for the sidebar. F1, RightThumb/R3 or **Details**
opens complete readonly values and returns to the previous selection. Capture,
text editors, dropdowns and busy host forms keep priority. Supported configured
menu-key members take precedence over utility keys. Escape cancels a nested
context before returning to the game; held-key repeats cannot cancel then resume.

L1/R1 adjust volume by 10, with L1+R1 toggling mute; L2/R2 adjust the preferred
Fast Forward percentage by 20, with L2+R2 toggling activation. Chords preserve
the selected value, win within 80 ms and require full release before rearming.
Capture/editing owns these inputs; closed-menu gameplay bindings are unchanged.

To change the shortcut, open **Game > Controls**, choose **Keyboard**, **Mouse & Keyboard**
or **Gamepad**, then select **Open menu**. Release the key/button that activated
capture, hold the desired single control or simultaneous combination, and
release all its controls. The largest simultaneously held set is saved;
sequential unrelated presses are not merged into a combination. Escape cancels
capture. Keyboard and mouse/keyboard share the keyboard menu shortcut.

Bindings persist in `%APPDATA%\FFTARecomp\input.cfg` on Windows:

```ini
key.menu = LeftControl+M
pad.menu = LeftTrigger+RightTrigger
```

The pad example means LB+RB in the menu's Xbox labels. Existing profiles get
the defaults without requiring a reset. Restoring the default gamepad layout
also restores Start+Back. Escape always remains available as a recovery key.

A completed shortcut toggles once and stays consumed through partial releases,
until its last member is released. A second complete press can close the menu.
Gamepad capture waits for a neutral state and records one controller's controls;
disconnecting or switching devices discards an incomplete capture. Individual
controls still retain their ordinary game bindings before a full shortcut is
recognized.

## Validation

For the Atlas migration, Wine release validation passed 31 input-config,
86 menu and 11 focused frontend tests. Actual EUR GUI checks covered supported
Tab menu-key priority, keyboard capture and F1 exclusivity, R3/mouse details,
shoulder chords and partial release, dropdown cancellation and held Escape.
The controller was simulated XInput. See [Atlas validation](menu_clan_atlas.md)
for viewport, presenter, FF/audio coverage and remaining platform limits.
The results below describe the earlier shortcut implementation.

Windows GNU release tests executed through Wine: 25 input/config tests,
26 menu tests and 27 runtime tests passed. Coverage includes capture activation
release, overlapping presses, partial release/re-arm, device changes, old
profiles, persistence and keyboard/mouse/gamepad menu entry points.

An interactive smoke run used an isolated XInput test double and test save.
It opened the menu with Start+Back, captured LB+RB through the actual gamepad
page, opened and closed the menu with the new combination, and kept the menu
unchanged while releasing only one bumper. It also captured Ctrl+M through
the keyboard page, reopened the menu with it and verified both saved bindings.
The process exited cleanly. This is a Windows/Wine test with simulated input,
not a claim of manual testing with a physical controller on native Windows.

Logs, test doubles and screenshots are under ignored `target/combo-ui-smoke/`
and `target/windows-combo-*.log`. Test controller DLLs are never packaged.
