//! Resolve legacy saves without renaming, deleting or copying the user's files.
use std::path::{Path, PathBuf};

fn preference() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("FFTARecomp").join("shared-save-path.txt"))
}

pub fn remember(save: &Path) {
    if input_config::storage::portable_root().is_some() { return; }
    let result = (|| -> std::io::Result<()> {
        let Some(file) = preference() else { return Ok(()); };
        std::fs::create_dir_all(file.parent().unwrap())?;
        std::fs::write(file, std::path::absolute(save)?.to_string_lossy().as_bytes())
    })();
    if let Err(e) = result { eprintln!("save: cannot remember shared save: {e}"); }
}

/// Whether the installed-mode selection came from the remembered family.
/// Explicit environment paths always take precedence over a remembered slot.
pub fn is_remembered_family(save: &Path) -> bool {
    if std::env::var_os("FFTA_SAVE_PATH").is_some() || input_config::storage::portable_root().is_some() {
        return false;
    }
    preference().and_then(|p| std::fs::read_to_string(p).ok())
        .is_some_and(|value| Path::new(value.trim()) == save)
}

fn candidates(rom: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = rom.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut saves = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|s| s.eq_ignore_ascii_case("sav")) {
            saves.push(path);
        }
    }
    saves.sort();
    Ok(saves)
}

fn available_or_backed_up(save: &Path) -> bool {
    save.is_file() || crate::save_backups::Backups::new(save).has_copies()
}

fn available_with_namespace(save: &Path, namespace: Option<&Path>) -> bool {
    available_or_backed_up(save) || namespace.is_some_and(|root| crate::save_backups::Backups::at_namespace(save, root).has_copies())
}

pub fn resolve_routed(rom: &Path, namespace: Option<&Path>) -> Result<PathBuf, String> {
    resolve_with(rom, true, namespace)
}

/// Same save selection, without creating directories or opening a picker.
pub fn inspect(rom: &Path) -> Result<PathBuf, String> {
    resolve_with(rom, false, None)
}

fn resolve_with(rom: &Path, interactive: bool, namespace: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("FFTA_SAVE_PATH") {
        return Ok(PathBuf::from(path));
    }
    if let Some(root) = input_config::storage::portable_root() {
        let dir = root.join("Data").join("saves");
        if interactive {
            std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        // Persist only the numeric choice: the complete portable folder can move.
        let slot = input_config::AvConfig::load().debug_game_save_slot;
        return crate::game_saves::GameSaves::new(&dir.join("ffta.sav")).path(slot);
    }
    if let Some(saved) = preference().and_then(|p| std::fs::read_to_string(p).ok()) {
        let saved = PathBuf::from(saved);
        if available_with_namespace(&saved, namespace) { return Ok(saved); }
    }
    let expected = rom.with_extension("sav");
    if available_with_namespace(&expected, namespace) { return Ok(expected); }
    let saves = candidates(rom)?;
    match saves.as_slice() {
        [] => Ok(expected),
        [only] => {
            eprintln!("save: using {} (independent of ROM name)", only.display());
            Ok(only.clone())
        }
        _ => {
            if !interactive {
                return Err("Multiple cartridge saves found; choose one with --sav FILE (or FFTA_SAVE_PATH).".into());
            }
            // Other games may share this directory. Never guess which file to overwrite.
            let selected = rfd::FileDialog::new()
                .set_title("Choose your FFTA save - Cancel to start a new save")
                .add_filter("Game Boy Advance save", &["sav"])
                .set_directory(rom.parent().unwrap_or(Path::new(".")))
                .pick_file();
            Ok(selected.unwrap_or(expected))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn private_backup_keeps_missing_primary_discoverable() {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("ffta.sav");
        std::fs::write(&save, vec![7; 65536]).unwrap();
        let namespace = crate::save_backups::Backups::debug_namespace(&dir.path().join("private"), b"synthetic");
        let backups = crate::save_backups::Backups::with_namespace(&save, &namespace).unwrap();
        backups.capture_startup().unwrap();
        std::fs::remove_file(&save).unwrap();
        assert!(!super::available_or_backed_up(&save));
        assert!(super::available_with_namespace(&save, Some(&namespace)));
    }

    #[test]
    fn missing_primary_keeps_its_location_when_recovery_copies_exist() {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("custom-progress.sav");
        assert!(!super::available_or_backed_up(&save));
        std::fs::write(&save, vec![7;65536]).unwrap();
        let backups = crate::save_backups::Backups::new(&save);
        backups.capture_startup().unwrap();
        std::fs::remove_file(&save).unwrap();
        assert!(super::available_or_backed_up(&save));
        assert!(!save.exists());
        assert!(!super::available_or_backed_up(&dir.path().join("unrelated.sav")));
    }

    #[test]
    fn arbitrary_names_and_multiple_saves_are_preserved() {
        let dir = std::env::temp_dir().join(format!("ffta-save-resolve-{}",std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rom = dir.join("Europe.gba");
        assert!(super::candidates(&rom).unwrap().is_empty());
        let save = dir.join("my progress.SAV");
        std::fs::write(&save,b"test progress").unwrap();
        assert_eq!(super::candidates(&rom).unwrap(), vec![save.clone()]);
        let other = dir.join("another.sav");
        std::fs::write(&other,b"other progress").unwrap();
        assert_eq!(super::candidates(&rom).unwrap().len(),2);
        assert_eq!(std::fs::read(&save).unwrap(),b"test progress");
        assert_eq!(std::fs::read(&other).unwrap(),b"other progress");
        std::fs::remove_file(save).unwrap();
        std::fs::remove_file(other).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
