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
        Self { base, current }
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
        Ok(self.base.with_file_name(name))
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
        let target = read_save(&path, old.len())?;
        let erased = vec![0xff; old.len()];
        let fresh = super::save_backups::prepare_boot(
            machine,
            boot,
            Some(target.as_deref().unwrap_or(&erased)),
        )?;
        let protected = super::save_backups::Backups::new(&path).capture_startup()?;
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
        if path.try_exists().map_err(|e| e.to_string())? {
            if !overwrite {
                return Err("Game save is occupied. Confirm replacement first.".into());
            }
            // Protect existing progress before an explicitly confirmed replacement.
            read_save(&path, bytes.len())?;
            super::save_backups::Backups::new(&path).capture_startup()?;
        }
        super::save_backups::atomic_write(&path, bytes, overwrite)
    }
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
