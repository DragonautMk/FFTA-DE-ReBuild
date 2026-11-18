//! App-private, ROM-specific save-state slots and one-level session undo.
use std::{fs, io::Read, path::{Path, PathBuf}};
use gba_core::{Machine, save_state::MAX_STATE_BYTES};

pub enum Undo {
    Save { slot: u8, previous: Option<Vec<u8>>, digest: String },
    Load { snapshot: Vec<u8> },
}

pub struct Slots { directory: PathBuf }
impl Slots {
    pub fn new(save_path: &Path, rom_key: &str) -> Result<Self, String> {
        Ok(Self { directory: save_path.parent().ok_or("Save directory missing")?.join("SaveStates").join(rom_key) })
    }
    fn path(&self, slot: u8) -> Result<PathBuf, String> {
        if !(1..=9).contains(&slot) { return Err("Invalid save-state slot".into()); }
        Ok(self.directory.join(format!("slot-{slot}.ffstate")))
    }
    fn read(&self, slot: u8) -> Result<Option<Vec<u8>>, String> {
        let mut file = match fs::File::open(self.path(slot)?) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Cannot read save state: {e}")),
        };
        let mut bytes = Vec::new();
        (&mut file).take((MAX_STATE_BYTES + 1) as u64).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_STATE_BYTES { return Err("Save state exceeds the size limit".into()); }
        Ok(Some(bytes))
    }
    pub fn occupied(&self) -> Result<i32, String> {
        let mut mask = 0;
        for slot in 1..=9 {
            match fs::metadata(self.path(slot)?) {
                Ok(_) => mask |= 1 << (slot - 1),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                Err(e) => return Err(format!("Cannot list save states: {e}")),
            }
        }
        Ok(mask)
    }
    pub fn save(&self, machine: &Machine, slot: u8, overwrite: bool) -> Result<Undo, String> {
        let previous = self.read(slot)?;
        if previous.is_some() && !overwrite { return Err("Slot already occupied; confirm overwrite".into()); }
        let bytes = machine.save_state()?;
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        crate::save_backups::atomic_write(&self.path(slot)?, &bytes, previous.is_some())?;
        Ok(Undo::Save { slot, previous, digest: gba_core::ffta_rom::digest(&bytes) })
    }
    pub fn load(&self, machine: &mut Machine, slot: u8) -> Result<Undo, String> {
        let bytes = self.read(slot)?.ok_or("Save-state slot is empty")?;
        let snapshot = machine.save_state()?;
        machine.load_state(&bytes)?;
        Ok(Undo::Load { snapshot })
    }
    pub fn undo(&self, machine: &mut Machine, undo: &Undo) -> Result<(), String> {
        match undo {
            Undo::Load { snapshot } => machine.load_state(snapshot),
            Undo::Save { slot, previous, digest } => {
                let bytes = self.read(*slot)?.ok_or("The saved slot was removed; undo refused")?;
                if gba_core::ffta_rom::digest(&bytes) != *digest { return Err("The saved slot changed; undo refused".into()); }
                let path = self.path(*slot)?;
                if let Some(bytes) = previous { crate::save_backups::atomic_write(&path, bytes, true) }
                else { fs::remove_file(path).map_err(|e| e.to_string()) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_overwrite_load_and_undo_are_transactional() {
        let root = std::env::var_os("FFTA_VALIDATION_WORK").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
            .join(format!("state-slots-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let slots = Slots::new(&root.join("game.sav"), "synthetic-rom").unwrap();
        let mut machine = Machine::new(vec![0; 512]);
        assert_eq!(slots.occupied().unwrap(), 0);
        machine.bus.ewram[0] = 12;
        let first = slots.save(&machine, 1, false).unwrap();
        assert_eq!(slots.occupied().unwrap(), 1);
        assert!(slots.save(&machine, 1, false).is_err());
        machine.bus.ewram[0] = 34;
        let overwrite = slots.save(&machine, 1, true).unwrap();
        slots.undo(&mut machine, &overwrite).unwrap();
        let load = slots.load(&mut machine, 1).unwrap();
        assert_eq!(machine.bus.ewram[0], 12);
        slots.undo(&mut machine, &load).unwrap();
        assert_eq!(machine.bus.ewram[0], 34);
        slots.undo(&mut machine, &first).unwrap();
        assert_eq!(slots.occupied().unwrap(), 0);
        assert!(slots.save(&machine, 0, false).is_err());
        assert!(slots.save(&machine, 10, false).is_err());
        let undo = slots.save(&machine, 9, false).unwrap();
        let path = slots.path(9).unwrap();
        fs::write(&path, b"changed outside the app").unwrap();
        assert!(slots.undo(&mut machine, &undo).is_err());
        assert!(slots.load(&mut machine, 9).is_err());
        assert_eq!(machine.bus.ewram[0], 34);
        assert_eq!(fs::read(&path).unwrap(), b"changed outside the app");
        fs::remove_dir_all(root).unwrap();
    }
}
