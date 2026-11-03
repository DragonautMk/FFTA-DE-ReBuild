# Restart the current ROM

Open the pause menu and choose **Restart ROM**. Confirming immediately restarts
from the session's initial boot, keeping the same window, ROM, BIOS, native
translation, settings and selected cartridge-save path. It resumes automatically.
The main menu fits eleven entries at the usual desktop sizes.

Restart first flushes the **current cartridge memory** to the selected `.sav`.
It then boots with those bytes, like saving and relaunching the emulator. This
preserves progress already written by the game into cartridge memory; unsaved
battle/RAM progress is discarded. It does not load the cartridge bytes from the
beginning of the session. See [cartridge backups](save_backups.md) to recover
older progress instead.

The two startup backups and all nine state-slot files stay unchanged. Restart
clears the session's slot Undo, releases guest keys and pending mouse actions,
resets Fast, and rebuilds audio queues, temporal history and separated UI from
the new timeline. Loading a slot after restart remains available.

The clean machine is prepared and its cartridge file atomically published
before replacing the live machine. An invalid boot or failed write leaves the
live session intact, paused with an error in the menu. Active input recording
refuses restart with an in-menu instruction to stop recording first. No new
process, translation or snapshot format is introduced. On `decomp`, restart
and backup restore retain the same immutable decoded-resource owner, including
OBJ metadata, BG descriptors and base palettes, without reopening their files.

## Validation

Native Linux release: 36 menu and 83 runtime tests pass. Windows GNU under
Wine: 36 menu and 84 runtime tests pass; four optional runtime tests ignored
per platform. The restart tests cover boot CPU/RAM/clock restoration, current
cartridge bytes on disk and in memory, unchanged backup files, invalid boot
and failed atomic publication without modifying the live session. Menu tests
cover keyboard confirmation, mouse hit testing/drawing and error recovery from
240×160 through 4K. Production builds are warning-free.

Actual Linux and Windows/Wine normal-play checks load the supplied deployment
slot, exercise laws/HELP and Load/Undo/reload, then select Restart ROM and reach
the initial language screen in the same process/window. Slot files and original
ROM/save-state/executables stay unchanged; inspected menu captures show the new
entry. Private evidence: ignored `target/restart-deployment-ui/`.

The integrated `decomp` branch additionally passes211core/36menu/103runtime
on Linux and211/36/102under Wine. A private USA/EUR test on both targets deletes
copied dataset files, then verifies restart and backup recovery retain the same
OBJ/BG/palette resource owner/counters and correct cartridge bytes. Seven core
and24runtime optional tests are skipped by the ordinary suites; this new
lifecycle test is explicitly selected separately for both regions.

Native Windows and physical controllers remain unverified. No performance
measurements, AndroidPort changes or portable archive rebuilds were selected.
