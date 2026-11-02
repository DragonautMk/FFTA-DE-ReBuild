# Switch ROM native picker

2026-10-18. The existing pause-menu Switch ROM submenu retains Return to game
and now labels its selection action Choose ROM.... It opens rfd's native file
picker in the current ROM/save directory, filtered to GBA images.

Cancel leaves the paused game open. The selected file must match an exact
supported FFTA USA/EUR SHA256. Read/hash errors show a dialog and do not exit.
The current save is flushed before starting the selected ROM. Save/start
errors also retain the paused game. Only a successful process spawn triggers
the existing clean-exit path. On Windows the replacement has no extra console.
The executable restarts directly with play and the selected path; installer
and ROM/save contents are not repackaged or deleted. Explicit title-pinned
packages still reject another hash before exit because their native library
belongs to one ROM; development/dynamic builds can select either region.

The file picker uses the same rfd 0.15 version already present in the project.
Windows uses the system dialog; Linux uses rfd's portal backend.
NOT MEASURED: manual interaction with the dialog and full process handoff.

MEASURED: Windows release build succeeded. Submenu and invalid/missing-file
tests passed. Explicit read-only validation tests accepted both exact user
ROMs (USA and EUR), with launch=false and save=false. No original saves
were written during validation.
