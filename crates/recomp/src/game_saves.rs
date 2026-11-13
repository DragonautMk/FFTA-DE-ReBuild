//! Numbered cartridge saves. Slot 1 keeps the original, unnumbered filename.
use gba_core::Machine;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub const SLOT_COUNT: u8 = 99;

pub struct GameSaves {
    base: PathBuf,
    debug_directory: Option<PathBuf>,
    backup_namespace: Option<PathBuf>,
    pub current: u8,
}

impl GameSaves {
    pub fn new(selected: &Path) -> Self {
        let mut base = selected.to_owned();
        let mut current = 1;
        if let Some((stem, suffix)) = selected
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.rsplit_once("-slot-"))
        {
            if suffix.len() == 2 && suffix.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(slot @ 2..=SLOT_COUNT) = suffix.parse::<u8>() {
                    let mut name = std::ffi::OsString::from(stem);
                    if let Some(ext) = selected.extension() {
                        name.push(".");
                        name.push(ext);
                    }
                    base.set_file_name(name);
                    current = slot;
                }
            }
        }
        Self { base, current, debug_directory: None, backup_namespace: None }
    }

    /// Keep the first cartridge save in its original location. Numbered
    /// debug saves live in a per-ROM directory under the selected asset root.
    pub fn with_debug_root(selected: &Path, rom: &[u8], root: &Path) -> Result<Self, String> {
        let mut saves = Self::new(selected);
        let legacy = Self::new(&saves.base);
        let dir = root.join("runtime/GameSaves").join(gba_core::ffta_rom::digest(rom));
        let backup_namespace = crate::save_backups::Backups::debug_namespace(root, rom);
        saves.debug_directory = Some(dir);
        saves.backup_namespace = Some(backup_namespace.clone());
        for slot in 2..=SLOT_COUNT {
            let legacy_save = legacy.path(slot)?;
            crate::save_state::copy_legacy_file(&legacy_save, &saves.path(slot)?)?;
            crate::save_backups::Backups::with_namespace(&legacy_save, &backup_namespace)?;
        }
        Ok(saves)
    }

    pub fn base_path(&self) -> &Path { &self.base }

    pub fn prepare_backups(&self, save: &Path) -> Result<crate::save_backups::Backups, String> {
        match &self.backup_namespace {
            Some(root) if save == self.base.as_path() => crate::save_backups::Backups::local_with_private_recovery(save, root),
            Some(root) => crate::save_backups::Backups::with_namespace(save, root),
            None => Ok(crate::save_backups::Backups::new(save)),
        }
    }

    pub fn current_backups(&self, save: &Path) -> crate::save_backups::Backups {
        match &self.backup_namespace {
            Some(_) if save == self.base.as_path() => crate::save_backups::Backups::new(save),
            Some(root) => crate::save_backups::Backups::at_namespace(save, root),
            None => crate::save_backups::Backups::new(save),
        }
    }

    pub fn path(&self, slot: u8) -> Result<PathBuf, String> {
        if !(1..=SLOT_COUNT).contains(&slot) {
            return Err("Game save slot must be between 1 and 99".into());
        }
        if slot == 1 {
            return Ok(self.base.clone());
        }
        let mut name = self
            .base
            .file_stem()
            .ok_or("Save filename is missing")?
            .to_os_string();
        name.push(format!("-slot-{slot:02}"));
        if let Some(ext) = self.base.extension() {
            name.push(".");
            name.push(ext);
        }
        Ok(match &self.debug_directory {
            Some(directory) => directory.join(name),
            None => self.base.with_file_name(name),
        })
    }

    pub fn choices(&self, expected: usize) -> Vec<menu::GameSaveSlot> {
        (1..=SLOT_COUNT)
            .map(|slot| {
                let path = self.path(slot).expect("bounded game save slot");
                let (available, status) = match read_save(&path, expected) {
                    Ok(Some(_)) => (true, "Saved".into()),
                    Ok(None) => (true, "Empty - new game".into()),
                    Err(error) => (false, error),
                };
                menu::GameSaveSlot {
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    status,
                    available,
                    occupied: path.exists(),
                    current: self.current == slot,
                }
            })
            .collect()
    }

    /// Prepare and validate the destination before flushing the old cartridge.
    /// A missing destination gets erased memory, never the boot snapshot's save.
    pub fn switch(
        &mut self,
        machine: &mut Machine,
        boot: &[u8],
        slot: u8,
    ) -> Result<PathBuf, String> {
        let path = self.path(slot)?;
        let old_path = self.path(self.current)?;
        let old = machine
            .bus
            .save_data()
            .ok_or("This cartridge has no save memory")?;
        if path == old_path {
            return Ok(path);
        }
        std::fs::create_dir_all(parent(&path)).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(parent(&old_path)).map_err(|e| e.to_string())?;
        let target = read_save(&path, old.len())?;
        let erased = vec![0xff; old.len()];
        let fresh = super::save_backups::prepare_boot(
            machine,
            boot,
            Some(target.as_deref().unwrap_or(&erased)),
        )?;
        let protected = self.prepare_backups(&path)?.capture_startup()?;
        if protected != target {
            return Err("The selected save changed. Select it again.".into());
        }
        super::save_backups::atomic_write(&old_path, old, true)?;
        *machine = fresh;
        self.current = slot;
        Ok(path)
    }

    /// Copy the game's cartridge memory, not a machine snapshot or unsaved play.
    pub fn copy_current(&self, machine: &Machine, slot: u8, overwrite: bool) -> Result<(), String> {
        let path = self.path(slot)?;
        let bytes = machine
            .bus
            .save_data()
            .ok_or("This cartridge has no save memory")?;
        std::fs::create_dir_all(parent(&path)).map_err(|e| e.to_string())?;
        if path.try_exists().map_err(|e| e.to_string())? {
            if !overwrite {
                return Err("Game save is occupied. Confirm replacement first.".into());
            }
            // Protect existing progress before an explicitly confirmed replacement.
            read_save(&path, bytes.len())?;
            self.prepare_backups(&path)?.capture_startup()?;
        }
        super::save_backups::atomic_write(&path, bytes, overwrite)
    }
}

fn parent(path: &Path) -> &Path {
    path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."))
}

fn read_save(path: &Path, expected: usize) -> Result<Option<Vec<u8>>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Cannot read save: {e}")),
    };
    let mut bytes = Vec::new();
    file.take(expected as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if expected == 0 || bytes.len() != expected {
        return Err("Unreadable / incompatible size".into());
    }
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn machine() -> Machine {
        let mut rom = vec![0; 512];
        rom[256..265].copy_from_slice(b"FLASH_V12");
        super::super::make_machine(rom, None)
    }

    #[test]
    fn routed_numbered_saves_preserve_primary_and_existing_private_versions() {
        let dir = tempfile::tempdir().unwrap();
        let normal = dir.path().join("normal");
        std::fs::create_dir_all(&normal).unwrap();
        let base = normal.join("ffta.sav");
        let legacy = GameSaves::new(&base);
        std::fs::write(&base, b"normal").unwrap();
        std::fs::write(legacy.path(2).unwrap(), b"legacy").unwrap();
        let legacy_backup = crate::save_backups::Backups::new(&legacy.path(2).unwrap());
        legacy_backup.capture_startup().unwrap();
        let private = dir.path().join("private/debug_saves");
        let saves = GameSaves::with_debug_root(&base, b"synthetic rom", &private).unwrap();
        assert_eq!(saves.path(1).unwrap(), base);
        assert!(saves.path(2).unwrap().starts_with(&private));
        assert_eq!(std::fs::read(saves.path(2).unwrap()).unwrap(), b"legacy");
        assert_eq!(std::fs::read(legacy.path(2).unwrap()).unwrap(), b"legacy");
        assert!(legacy_backup.has_copies());
        assert_eq!(saves.prepare_backups(&saves.path(2).unwrap()).unwrap().choices(6)[0].generation, Some(1));
        std::fs::write(saves.path(2).unwrap(), b"private edit").unwrap();
        let again = GameSaves::with_debug_root(&base, b"synthetic rom", &private).unwrap();
        assert_eq!(std::fs::read(again.path(2).unwrap()).unwrap(), b"private edit");
        assert_eq!(std::fs::read(&base).unwrap(), b"normal");
    }

    #[test]
    fn primary_backups_stay_local_while_numbered_backups_use_debug_root() {
        let dir = tempfile::tempdir().unwrap();
        let normal = dir.path().join("normal");
        std::fs::create_dir_all(&normal).unwrap();
        let base = normal.join("ffta.sav");
        std::fs::write(&base, vec![1; 65536]).unwrap();
        let private = dir.path().join("private/debug_saves");
        let namespace = crate::save_backups::Backups::debug_namespace(&private, b"synthetic rom");
        let saves = GameSaves::with_debug_root(&base, b"synthetic rom", &private).unwrap();

        saves.prepare_backups(&base).unwrap().capture_startup().unwrap();
        let local = normal.join("SaveBackups/ffta.sav");
        let copies: Vec<_> = std::fs::read_dir(&local).unwrap().map(|entry| entry.unwrap().path()).collect();
        assert_eq!(copies.len(), 1);
        assert_eq!(std::fs::read(&copies[0]).unwrap(), vec![1; 65536]);
        assert!(!namespace.join("ffta.sav").exists());
        assert_eq!(saves.current_backups(&base).choices(65536)[0].generation, Some(1));

        let numbered = saves.path(2).unwrap();
        std::fs::create_dir_all(numbered.parent().unwrap()).unwrap();
        std::fs::write(&numbered, vec![2; 65536]).unwrap();
        saves.prepare_backups(&numbered).unwrap().capture_startup().unwrap();
        assert!(namespace.join("ffta-slot-02.sav").is_dir());
        assert_eq!(saves.current_backups(&numbered).choices(65536)[0].generation, Some(1));
        assert_eq!(std::fs::read(&base).unwrap(), vec![1; 65536]);
        assert!(!namespace.join("ffta.sav").exists());
    }

    #[test]
    fn missing_primary_can_recover_old_private_backups_without_deleting_them() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("ffta.sav");
        std::fs::write(&base, vec![7; 65536]).unwrap();
        let private = dir.path().join("private/debug_saves");
        let namespace = crate::save_backups::Backups::debug_namespace(&private, b"synthetic rom");
        let previous = crate::save_backups::Backups::at_namespace(&base, &namespace);
        previous.capture_startup().unwrap();
        std::fs::remove_file(&base).unwrap();

        let saves = GameSaves::with_debug_root(&base, b"synthetic rom", &private).unwrap();
        let recovered = saves.prepare_backups(&base).unwrap();
        assert!(!base.exists());
        assert_eq!(recovered.choices(65536)[0].generation, Some(1));
        assert_eq!(previous.choices(65536)[0].generation, Some(1));
        let local = dir.path().join("SaveBackups/ffta.sav");
        let copy = std::fs::read_dir(&local).unwrap().next().unwrap().unwrap().path();
        assert_eq!(std::fs::read(copy).unwrap(), vec![7; 65536]);
    }

    #[test]
    fn existing_primary_does_not_import_old_private_backups() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("ffta.sav");
        let private = dir.path().join("private/debug_saves");
        let namespace = crate::save_backups::Backups::debug_namespace(&private, b"synthetic rom");
        std::fs::write(&base, vec![3; 65536]).unwrap();
        let previous = crate::save_backups::Backups::at_namespace(&base, &namespace);
        previous.capture_startup().unwrap();
        std::fs::write(&base, vec![4; 65536]).unwrap();

        let saves = GameSaves::with_debug_root(&base, b"synthetic rom", &private).unwrap();
        let local = saves.prepare_backups(&base).unwrap();
        assert!(!local.has_copies());
        local.capture_startup().unwrap();
        let local_dir = dir.path().join("SaveBackups/ffta.sav");
        let copies: Vec<_> = std::fs::read_dir(local_dir).unwrap().map(|entry| entry.unwrap().path()).collect();
        assert_eq!(copies.len(), 1);
        assert_eq!(std::fs::read(&copies[0]).unwrap(), vec![4; 65536]);
        assert_eq!(previous.choices(65536)[0].generation, Some(1));
    }

    #[test]
    fn existing_local_primary_backups_are_not_merged_with_old_private_history() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("ffta.sav");
        let private = dir.path().join("private/debug_saves");
        let namespace = crate::save_backups::Backups::debug_namespace(&private, b"synthetic rom");
        std::fs::write(&base, vec![3; 65536]).unwrap();
        let previous = crate::save_backups::Backups::at_namespace(&base, &namespace);
        previous.capture_startup().unwrap();
        std::fs::write(&base, vec![4; 65536]).unwrap();
        crate::save_backups::Backups::new(&base).capture_startup().unwrap();
        std::fs::remove_file(&base).unwrap();

        let saves = GameSaves::with_debug_root(&base, b"synthetic rom", &private).unwrap();
        let local = saves.prepare_backups(&base).unwrap();
        assert_eq!(local.choices(65536)[0].generation, Some(1));
        let local_dir = dir.path().join("SaveBackups/ffta.sav");
        let copies: Vec<_> = std::fs::read_dir(local_dir).unwrap().map(|entry| entry.unwrap().path()).collect();
        assert_eq!(copies.len(), 1);
        assert_eq!(std::fs::read(&copies[0]).unwrap(), vec![4; 65536]);
        assert_eq!(previous.choices(65536)[0].generation, Some(1));
    }

    #[test]
    fn numbered_paths_reopen_without_nesting_and_move_with_the_save_directory() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("my progress.sav");
        let saves = GameSaves::new(&base);
        assert_eq!(saves.path(1).unwrap(), base);
        assert!(saves.path(0).is_err());
        assert!(saves.path(100).is_err());
        for slot in 2..=99 {
            let path = saves.path(slot).unwrap();
            assert_eq!(
                path.file_name().unwrap(),
                format!("my progress-slot-{slot:02}.sav").as_str()
            );
            let reopened = GameSaves::new(&path);
            assert_eq!(reopened.current, slot);
            assert_eq!(reopened.path(1).unwrap(), base);
            assert_eq!(reopened.path(99).unwrap(), saves.path(99).unwrap());
        }
        let moved = GameSaves::new(&dir.path().join("moved/my progress-slot-99.sav"));
        assert_eq!(
            moved.path(2).unwrap(),
            dir.path().join("moved/my progress-slot-02.sav")
        );
        let dots = GameSaves::new(&dir.path().join("my.progress-slot-09.SAV"));
        assert_eq!(dots.path(1).unwrap(), dir.path().join("my.progress.SAV"));
        assert_eq!(
            dots.path(99).unwrap(),
            dir.path().join("my.progress-slot-99.SAV")
        );
    }

    #[test]
    fn switch_flushes_only_old_save_and_cleanly_boots_selected_or_erased_memory() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("ffta.sav");
        let mut saves = GameSaves::new(&base);
        let mut m = machine();
        m.bus.load_save_data(&vec![1; 65536]);
        let boot = m.save_state().unwrap(); // Contains old progress: must not leak into an empty slot.
        m.bus.load_save_data(&vec![2; 65536]);
        m.cpu.regs[4] = 123;
        let target = saves.path(99).unwrap();
        std::fs::write(&target, vec![9; 65536]).unwrap();
        assert_eq!(saves.switch(&mut m, &boot, 99).unwrap(), target);
        assert_eq!(m.cpu.regs[4], 0);
        assert_eq!(m.bus.save_data().unwrap(), vec![9; 65536]);
        assert_eq!(std::fs::read(&base).unwrap(), vec![2; 65536]);
        assert_eq!(std::fs::read(&target).unwrap(), vec![9; 65536]);
        assert!(super::super::save_backups::Backups::new(&target).has_copies());
        saves.switch(&mut m, &boot, 2).unwrap();
        assert_eq!(m.bus.save_data().unwrap(), vec![0xff; 65536]);
        assert!(!saves.path(2).unwrap().exists());
        saves.switch(&mut m, &boot, 1).unwrap();
        assert_eq!(
            std::fs::read(saves.path(2).unwrap()).unwrap(),
            vec![0xff; 65536]
        );
        assert_eq!(m.bus.save_data().unwrap(), vec![2; 65536]);
    }

    #[test]
    fn failed_switch_keeps_machine_current_slot_and_files_and_copy_requires_confirmation() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("ffta.sav");
        let mut saves = GameSaves::new(&base);
        let mut m = machine();
        let boot = m.save_state().unwrap();
        m.bus.load_save_data(&vec![7; 65536]);
        let before = m.save_state().unwrap();
        let target = saves.path(99).unwrap();
        std::fs::write(&target, [1, 2, 3]).unwrap();
        let choices = saves.choices(65536);
        assert!(choices[0].current);
        assert!(choices[1].available && !choices[1].occupied);
        assert!(!choices[98].available && choices[98].occupied);
        assert!(saves.switch(&mut m, &boot, 99).is_err());
        assert_eq!(m.save_state().unwrap(), before);
        assert_eq!(saves.current, 1);
        assert!(!base.exists());
        assert_eq!(std::fs::read(&target).unwrap(), [1, 2, 3]);
        std::fs::write(&target, vec![9; 65536]).unwrap();
        assert!(saves.switch(&mut m, b"bad boot", 99).is_err());
        // A failed source write must also leave the live machine alone.
        std::fs::create_dir(&base).unwrap();
        assert!(saves.switch(&mut m, &boot, 99).is_err());
        assert_eq!(m.save_state().unwrap(), before);
        assert_eq!(saves.current, 1);
        assert_eq!(std::fs::read(&target).unwrap(), vec![9; 65536]);
        assert!(saves.copy_current(&m, 99, false).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), vec![9; 65536]);
        saves.copy_current(&m, 99, true).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), vec![7; 65536]);
        assert_eq!(m.save_state().unwrap(), before);
        assert_eq!(saves.current, 1);
        saves.copy_current(&m, 2, false).unwrap();
        assert_eq!(
            std::fs::read(saves.path(2).unwrap()).unwrap(),
            vec![7; 65536]
        );
    }
}
