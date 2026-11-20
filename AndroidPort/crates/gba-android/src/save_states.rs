//! App-private, ROM-specific save-state slots and one-level session undo.
use std::{fs, io::Read, path::{Path, PathBuf}};
use gba_core::{Machine, save_state::MAX_STATE_BYTES};
use sha2::{Digest, Sha256};
use crate::cheat_provenance::{ExactHistory, History};

const PREVIEW_HEADER: usize = 48;
const PREVIEW_BYTES: usize = 240 * 160 * 4;
const PREFIX: usize = PREVIEW_HEADER + PREVIEW_BYTES;
const MAX_METADATA: usize = 256 * 1024;
pub const MAX_SLOT_BYTES: usize = MAX_STATE_BYTES + PREFIX + 36 + MAX_METADATA;

/// Validate the desktop envelope and retain its bounded cheat ancestry.
fn parts(bytes: &[u8]) -> Result<(&[u8], History), String> {
    if bytes.len() > MAX_SLOT_BYTES { return Err("Save state exceeds the size limit".into()); }
    let tagged = bytes.starts_with(b"FFTA-SL2") || bytes.starts_with(b"FFTA-SL3");
    if !tagged && !bytes.starts_with(b"FFTA-SL1") { return Ok((bytes, History::default())); }
    if bytes.len() < PREFIX { return Err("Incomplete desktop save-state preview".into()); }
    let length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    if length > MAX_STATE_BYTES as u64 || length < 108 { return Err("Invalid desktop save-state size".into()); }
    let end = PREFIX.checked_add(length as usize).ok_or("Invalid desktop save-state size")?;
    if bytes.len() < end { return Err("Incomplete desktop save state".into()); }
    if bytes[16..48] != Sha256::digest(&bytes[PREVIEW_HEADER..PREFIX])[..] {
        return Err("Desktop save-state preview is damaged".into());
    }
    let history = if tagged {
        let tail = &bytes[end..];
        if tail.len() < 36 { return Err("Missing cheat ancestry".into()); }
        let n = u32::from_le_bytes(tail[..4].try_into().unwrap()) as usize;
        if n > MAX_METADATA || tail.len() != 36 + n { return Err("Invalid cheat ancestry length".into()); }
        let mut hash = Sha256::new();hash.update(&bytes[PREFIX..end]);hash.update(&tail[36..]);
        if tail[4..36] != hash.finalize()[..] { return Err("Cheat ancestry checksum mismatch".into()); }
        let history: History = serde_json::from_slice(&tail[36..]).map_err(|e| format!("Invalid cheat ancestry: {e}"))?;
        history.validate()?;
        if (bytes.starts_with(b"FFTA-SL3") && history.schema != 2) || (bytes.starts_with(b"FFTA-SL2") && history.schema != 1) {
            return Err("Snapshot ancestry version does not match envelope".into());
        }
        history
    } else {
        if bytes.len() != end { return Err("Unexpected data after save state".into()); }
        History::default()
    };
    Ok((&bytes[PREFIX..end], history))
}

fn encode(machine: &Machine, history: &History) -> Result<Vec<u8>, String> {
    let state = machine.save_state()?;
    if history.is_empty() { return Ok(state); }
    history.validate()?;
    let metadata = serde_json::to_vec(history).map_err(|e| e.to_string())?;
    if metadata.len() > MAX_METADATA { return Err("Cheat ancestry exceeds the size limit".into()); }
    let mut pixels = Vec::with_capacity(PREVIEW_BYTES);
    for &pixel in &machine.bus.framebuffer {
        pixels.extend_from_slice(&[
            ((pixel & 31) * 255 / 31) as u8,
            (((pixel >> 5) & 31) * 255 / 31) as u8,
            (((pixel >> 10) & 31) * 255 / 31) as u8, 255]);
    }
    if pixels.len() != PREVIEW_BYTES { return Err("Invalid state preview size".into()); }
    let mut bytes = Vec::with_capacity(PREFIX + state.len() + 36 + metadata.len());
    bytes.extend_from_slice(if history.schema == 2 { b"FFTA-SL3" } else { b"FFTA-SL2" });
    bytes.extend_from_slice(&(state.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&Sha256::digest(&pixels));
    bytes.extend_from_slice(&pixels);
    bytes.extend_from_slice(&state);
    bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    let mut hash = Sha256::new();hash.update(&state);hash.update(&metadata);
    bytes.extend_from_slice(&hash.finalize());bytes.extend_from_slice(&metadata);
    Ok(bytes)
}

pub enum Undo {
    Save { slot: u8, previous: Option<Vec<u8>>, digest: String },
    Load { snapshot: Vec<u8>, exact: Option<ExactHistory> },
}

pub struct Slots { directory: PathBuf, prefix: Option<String>, rom_key: String }
impl Slots {
    pub fn new(save_path: &Path, rom_key: &str) -> Result<Self, String> {
        Ok(Self { directory: save_path.parent().ok_or("Save directory missing")?.join("SaveStates").join(rom_key),
            prefix:None,rom_key:rom_key.to_owned() })
    }
    pub fn beside_rom(save_path:&Path,rom_key:&str)->Result<Self,String> {
        let directory=save_path.parent().ok_or("Save directory missing")?.to_owned();
        let prefix=save_path.file_stem().and_then(|s|s.to_str()).filter(|s|!s.is_empty()).ok_or("Invalid save name")?.to_owned();
        Ok(Self{directory,prefix:Some(prefix),rom_key:rom_key.to_owned()})
    }
    fn path(&self, slot: u8) -> Result<PathBuf, String> {
        if !(1..=9).contains(&slot) { return Err("Invalid save-state slot".into()); }
        let name=match &self.prefix {Some(prefix)=>format!("{prefix}.slot-{slot}.ffstate"),None=>format!("slot-{slot}.ffstate")};
        Ok(self.directory.join(name))
    }
    fn read(&self, slot: u8) -> Result<Option<Vec<u8>>, String> {
        let mut file = match fs::File::open(self.path(slot)?) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Cannot read save state: {e}")),
        };
        let mut bytes = Vec::new();
        (&mut file).take((MAX_SLOT_BYTES + 1) as u64).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_SLOT_BYTES { return Err("Save state exceeds the size limit".into()); }
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
    #[cfg(test)]
    fn save(&self, machine: &Machine, slot: u8, overwrite: bool) -> Result<Undo, String> {
        self.save_with_history(machine, slot, overwrite, &History::default())
    }
    pub fn save_with_history(&self, machine: &Machine, slot: u8, overwrite: bool, history: &History) -> Result<Undo, String> {
        let previous = self.read(slot)?;
        if previous.is_some() && !overwrite { return Err("Slot already occupied; confirm overwrite".into()); }
        let bytes = encode(machine, history)?;
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        crate::save_backups::atomic_write(&self.path(slot)?, &bytes, previous.is_some())?;
        Ok(Undo::Save { slot, previous, digest: gba_core::ffta_rom::digest(&bytes) })
    }
    #[cfg(test)]
    fn load(&self, machine: &mut Machine, slot: u8) -> Result<Undo, String> {
        self.load_with_history(machine, slot, None).map(|(undo, _)| undo)
    }
    pub fn load_with_history(&self, machine: &mut Machine, slot: u8, exact: Option<ExactHistory>) -> Result<(Undo, History), String> {
        let bytes = self.read(slot)?.ok_or("Save-state slot is empty")?;
        let (state, history) = parts(&bytes)?;
        let snapshot = machine.save_state()?;
        machine.load_state(state)?;
        Ok((Undo::Load { snapshot, exact }, history))
    }
    pub fn export(&self, slot: u8) -> Result<Vec<u8>, String> {
        self.read(slot)?.ok_or_else(|| "Save-state slot is empty".into())
    }
    pub fn import(&self, machine: &Machine, slot: u8, bytes: &[u8], overwrite: bool) -> Result<Undo, String> {
        if bytes.len() > MAX_SLOT_BYTES { return Err("Save state exceeds the size limit".into()); }
        let previous = self.read(slot)?;
        if previous.is_some() && !overwrite { return Err("Slot already occupied; confirm overwrite".into()); }
        let mut probe = if machine.bus.real_bios {
            Machine::new_with_bios(machine.bus.rom.clone(), &machine.bus.bios)
        } else { Machine::new(machine.bus.rom.clone()) };
        let (state, _) = parts(bytes)?;
        probe.load_state(state)?;
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        if let Some(old)=&previous {
            // Keep a durable recovery copy even if the app exits before Undo.
            let backups=if self.prefix.is_some() {self.directory.join("SaveStateImportBackups").join(&self.rom_key)}
                else {self.directory.join("ImportBackups")};
            fs::create_dir_all(&backups).map_err(|e|e.to_string())?;
            let backup=backups.join(format!("slot-{slot}-{}.ffstate",gba_core::ffta_rom::digest(old)));
            if backup.exists() {
                let metadata=fs::metadata(&backup).map_err(|e|e.to_string())?;
                if metadata.len()!=old.len() as u64 || fs::read(&backup).map_err(|e|e.to_string())?!=*old {
                    return Err("Existing import backup is damaged".into());
                }
            } else {
                crate::save_backups::atomic_write(&backup,old,false)?;
            }
            if self.read(slot)?!=previous {return Err("Save-state slot changed during import; try again".into());}
        }
        crate::save_backups::atomic_write(&self.path(slot)?, bytes, previous.is_some())?;
        Ok(Undo::Save { slot, previous, digest: gba_core::ffta_rom::digest(bytes) })
    }
    pub fn undo(&self, machine: &mut Machine, undo: &Undo) -> Result<Option<ExactHistory>, String> {
        match undo {
            Undo::Load { snapshot, exact } => { machine.load_state(snapshot)?; Ok(exact.clone()) },
            Undo::Save { slot, previous, digest } => {
                let bytes = self.read(*slot)?.ok_or("The saved slot was removed; undo refused")?;
                if gba_core::ffta_rom::digest(&bytes) != *digest { return Err("The saved slot changed; undo refused".into()); }
                let path = self.path(*slot)?;
                if let Some(bytes) = previous { crate::save_backups::atomic_write(&path, bytes, true)?; }
                else { fs::remove_file(path).map_err(|e| e.to_string())?; }
                Ok(None)
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
    #[test]
    fn desktop_preview_import_is_verified_and_undo_restores_original() {
        let root = std::env::var_os("FFTA_VALIDATION_WORK").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
            .join(format!("state-import-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let slots = Slots::new(&root.join("game.sav"), "synthetic-rom").unwrap();
        let mut machine = Machine::new(vec![0; 512]);
        machine.cpu.regs[4] = 123;
        let raw = machine.save_state().unwrap();
        let preview = vec![9u8; PREVIEW_BYTES];
        let hash = gba_core::ffta_rom::digest(&preview);
        let mut wrapped = b"FFTA-SL1".to_vec();
        wrapped.extend_from_slice(&(raw.len() as u64).to_le_bytes());
        for pair in hash.as_bytes().chunks_exact(2) {
            wrapped.push(u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap());
        }
        wrapped.extend_from_slice(&preview);
        wrapped.extend_from_slice(&raw);
        let undo = slots.import(&machine, 1, &wrapped, false).unwrap();
        assert_eq!(slots.export(1).unwrap(), wrapped);
        machine.cpu.regs[4] = 456;
        slots.load(&mut machine, 1).unwrap();
        assert_eq!(machine.cpu.regs[4], 123);
        let mut damaged = wrapped.clone(); damaged[PREVIEW_HEADER] ^= 1;
        assert!(slots.import(&machine, 1, &damaged, true).is_err());
        assert_eq!(slots.export(1).unwrap(), wrapped);
        let mut tagged = wrapped.clone(); tagged[..8].copy_from_slice(b"FFTA-SL3");
        assert!(slots.import(&machine, 1, &tagged, true).is_err());
        let other = Machine::new(vec![1; 512]);
        assert!(slots.import(&other, 1, &wrapped, true).is_err());
        assert_eq!(slots.export(1).unwrap(), wrapped);
        slots.undo(&mut machine, &undo).unwrap();
        assert_eq!(slots.occupied().unwrap(), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cheat_ancestry_survives_import_load_save_and_undo() {
        let root = std::env::var_os("FFTA_VALIDATION_WORK").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
            .join(format!("state-history-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let slots = Slots::new(&root.join("game.sav"), "synthetic-rom").unwrap();
        let mut machine = Machine::new(vec![0; 512]);
        let mut older = History::default();
        older.record("armed-at-snapshot", serde_json::json!({"id":"A"}), 1, 2, 1);
        let sl2 = encode(&machine, &older).unwrap();
        assert!(sl2.starts_with(b"FFTA-SL2"));
        let (_, decoded) = parts(&sl2).unwrap();
        assert_eq!(decoded.events, older.events);
        let mut newer = History::default();
        newer.record("once", serde_json::json!({"id":"B"}), 3, 4, 1);
        assert_eq!(newer.status()["history"]["events"].as_array().unwrap().len(), 1);
        assert_eq!(newer.recording_summary()["observed_variants"], serde_json::json!(["B"]));
        let sl3 = encode(&machine, &newer).unwrap();
        assert!(sl3.starts_with(b"FFTA-SL3"));
        slots.import(&machine, 1, &sl3, false).unwrap();
        let (undo, loaded) = slots.load_with_history(&mut machine, 1, older.exact.clone()).unwrap();
        assert_eq!(loaded.events, newer.events);
        assert_eq!(serde_json::to_value(&loaded.exact).unwrap(), serde_json::to_value(&newer.exact).unwrap());
        let mut merged = older.clone();
        merged.include(&loaded); merged.restore_exact(loaded.exact);
        slots.save_with_history(&machine, 2, false, &merged).unwrap();
        let saved = slots.export(2).unwrap();
        let (_, reread) = parts(&saved).unwrap();
        assert_eq!(reread.events, merged.events);
        assert_eq!(serde_json::to_value(&reread.exact).unwrap(), serde_json::to_value(&merged.exact).unwrap());
        assert!(slots.undo(&mut machine, &undo).unwrap().is_none());
        let mut damaged = sl3.clone(); *damaged.last_mut().unwrap() ^= 1;
        assert!(slots.import(&machine, 1, &damaged, true).is_err());
        assert_eq!(slots.export(1).unwrap(), sl3);
        fs::remove_dir_all(root).unwrap();
    }

}
