//! Per-ROM slots (nine by default, 99 with Extended saves).
use gba_core::{save_state::MAX_STATE_BYTES, Machine};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const SLOT_COUNT: u8 = 9;
pub const EXTENDED_SLOT_COUNT: u8 = 99;
pub const PREVIEW_W: usize = 240;
pub const PREVIEW_H: usize = 160;
const PREVIEW_BYTES: usize = PREVIEW_W * PREVIEW_H * 4;
const SLOT_MAGIC: &[u8; 8] = b"FFTA-SL1";
const TAGGED_MAGIC: &[u8; 8] = b"FFTA-SL2";
const MAX_METADATA_BYTES: usize = 256 * 1024;
const HEADER: usize = 8 + 8 + 32;
const PREFIX: usize = HEADER + PREVIEW_BYTES;
pub(crate) const MAX_FILE_BYTES: usize = PREFIX + MAX_STATE_BYTES + 36 + MAX_METADATA_BYTES;

pub struct Slots {
    directory: PathBuf,
    count: u8,
    undo: Option<Undo>,
}

enum Undo {
    Save {
        slot: u8,
        previous: Option<Vec<u8>>,
        written: [u8; 32],
    },
    Load {
        slot: u8,
        previous: Vec<u8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoEffect {
    Save(u8),
    Load(u8),
}

pub struct SlotInfo {
    pub occupied: bool,
    pub preview: Vec<[u8; 4]>,
    pub status: String,
}

impl Slots {
    pub fn new(save: &Path, rom: &[u8]) -> Self {
        let sha: String = Sha256::digest(rom)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let parent = save
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Self {
            directory: parent.join("SaveStates").join(sha),
            count: SLOT_COUNT,
            undo: None,
        }
    }

    pub fn set_extended(&mut self, enabled: bool) {
        self.count = if enabled { EXTENDED_SLOT_COUNT } else { SLOT_COUNT };
    }

    pub fn count(&self) -> u8 { self.count }

    pub fn path(&self, slot: u8) -> Result<PathBuf, String> {
        if !(1..=self.count).contains(&slot) {
            return Err(format!("State slot must be between 1 and {}", self.count));
        }
        Ok(self.directory.join(format!("slot-{slot}.ffstate")))
    }

    pub fn info(&self, slot: u8) -> SlotInfo {
        let read = || -> Result<Option<(Vec<[u8; 4]>, bool)>, String> {
            let path = self.path(slot)?;
            let mut file = match File::open(&path) {
                Ok(f) => f,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.to_string()),
            };
            if file.metadata().map_err(|e| e.to_string())?.len() > MAX_FILE_BYTES as u64 {
                return Err("State exceeds the size limit".into());
            }
            let mut magic = [0; 8];
            file.read_exact(&mut magic).map_err(|e| e.to_string())?;
            if &magic == b"FFTA-ST\0" {
                return Ok(Some((Vec::new(), false)));
            } // Older, preview-less checkpoint.
            if &magic == TAGGED_MAGIC {
                return preview(&read_file(&path).map_err(|e| e.to_string())?).map(|p|Some((p, true)));
            }
            if &magic != SLOT_MAGIC {
                return Err("Unrecognized state format".into());
            }
            let mut prefix = vec![0; PREFIX];
            prefix[..8].copy_from_slice(&magic);
            file.read_exact(&mut prefix[8..])
                .map_err(|e| e.to_string())?;
            let state_len = validate_prefix(&prefix)?;
            if file.metadata().map_err(|e| e.to_string())?.len() != (PREFIX + state_len) as u64 {
                return Err("Incomplete state file".into());
            }
            Ok(Some((
                prefix[HEADER..]
                    .chunks_exact(4)
                    .map(|p| p.try_into().unwrap())
                    .collect(), false,
            )))
        };
        match read() {
            Ok(None) => SlotInfo {
                occupied: false,
                preview: Vec::new(),
                status: "Empty".into(),
            },
            Ok(Some((preview, tagged))) => SlotInfo {
                occupied: true,
                status: if preview.is_empty() {
                    "Saved - no preview"
                } else if tagged {
                    "Saved - cheat ancestry"
                } else {
                    "Saved"
                }
                .into(),
                preview,
            },
            Err(_) => SlotInfo {
                occupied: true,
                preview: Vec::new(),
                status: "Unreadable".into(),
            },
        }
    }

    #[cfg(test)]
    pub fn save(
        &mut self,
        machine: &Machine,
        slot: u8,
        preview: &[[u8; 4]],
        overwrite: bool,
    ) -> Result<(), String> {
        self.save_with_history(machine, slot, preview, overwrite, &Default::default())
    }

    pub(crate) fn save_with_history(
        &mut self, machine: &Machine, slot: u8, preview: &[[u8; 4]], overwrite: bool,
        history: &crate::cheat_provenance::History,
    ) -> Result<(), String> {
        let path = self.path(slot)?;
        let bytes = encode(machine, preview, history)?;
        let previous = match read_file(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("Cannot preserve previous slot: {e}")),
        };
        std::fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        atomic_write(&path, &bytes, overwrite)?;
        // Only a successful operation replaces the single undo entry.
        self.undo = Some(Undo::Save {
            slot,
            previous,
            written: Sha256::digest(&bytes).into(),
        });
        Ok(())
    }

    /// Exact checkpoint import with transactional validation and one-step file undo.
    pub fn import(&mut self, machine: &Machine, slot: u8, bytes: &[u8]) -> Result<(), String> {
        let mut scratch = crate::make_machine(machine.bus.rom.clone(), machine.bus.real_bios.then_some(machine.bus.bios.as_slice()));
        load_bytes(&mut scratch, bytes)?;
        let path = self.path(slot)?;
        if path.exists() { return Err("Import requires an empty snapshot slot".into()); }
        std::fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        atomic_write(&path, bytes, false)?;
        self.undo = Some(Undo::Save { slot, previous: None, written: Sha256::digest(bytes).into() });
        Ok(())
    }

    #[cfg(test)]
    pub fn load(&mut self, machine: &mut Machine, slot: u8) -> Result<(), String> {
        self.load_with_history(machine, slot).map(|_| ())
    }

    pub(crate) fn load_with_history(&mut self, machine: &mut Machine, slot: u8) -> Result<crate::cheat_provenance::History, String> {
        let path = self.path(slot)?;
        let previous = machine.save_state()?;
        let bytes = read_file(&path).map_err(|e| e.to_string())?;
        let history = load_bytes_with_history(machine, &bytes)?;
        self.undo = Some(Undo::Load { slot, previous });
        Ok(history)
    }

    pub fn undo_description(&self) -> Option<(&'static str, u8)> {
        self.undo.as_ref().map(|undo| match undo {
            Undo::Save { slot, .. } => ("Undo save (slot {slot})", *slot),
            Undo::Load { slot, .. } => ("Undo load (slot {slot})", *slot),
        })
    }

    #[cfg(test)]
    pub fn undo_label(&self) -> Option<String> {
        self.undo_description().map(|(key, slot)| key.replace("{slot}", &slot.to_string()))
    }

    pub fn undo_restores_machine(&self) -> bool {
        matches!(self.undo, Some(Undo::Load { .. }))
    }

    pub fn undo(&mut self, machine: &mut Machine) -> Result<UndoEffect, String> {
        let effect = match self.undo.as_ref().ok_or("Nothing to undo.")? {
            Undo::Save {
                slot,
                previous,
                written,
            } => {
                let path = self.path(*slot)?;
                let current =
                    read_file(&path).map_err(|e| format!("Cannot verify slot before undo: {e}"))?;
                if Sha256::digest(&current)[..] != written[..] {
                    return Err("Slot changed outside this session; undo refused.".into());
                }
                match previous {
                    Some(bytes) => atomic_write(&path, bytes, true)?,
                    None => std::fs::remove_file(&path).map_err(|e| e.to_string())?,
                }
                UndoEffect::Save(*slot)
            }
            Undo::Load { slot, previous } => {
                machine.load_state(previous)?;
                UndoEffect::Load(*slot)
            }
        };
        self.undo = None; // Undo is consumed; it is not a redo stack.
        Ok(effect)
    }
}

fn read_file(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "State exceeds the size limit",
        ));
    }
    Ok(bytes)
}

fn atomic_write(path: &Path, bytes: &[u8], overwrite: bool) -> Result<(), String> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing slot directory")?)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    if overwrite {
        file.persist(path)
    } else {
        file.persist_noclobber(path)
    }
    .map_err(|e| {
        if e.error.kind() == std::io::ErrorKind::AlreadyExists {
            "Slot is occupied. Select Save state again to confirm replacement.".into()
        } else {
            e.error.to_string()
        }
    })?;
    Ok(())
}

fn validate_prefix(data: &[u8]) -> Result<usize, String> {
    if data.len() < PREFIX || (&data[..8] != SLOT_MAGIC && &data[..8] != TAGGED_MAGIC) {
        return Err("Invalid state preview header".into());
    }
    let len = u64::from_le_bytes(data[8..16].try_into().unwrap());
    if len > MAX_STATE_BYTES as u64 || len < 108 {
        return Err("Invalid state size".into());
    }
    if data[16..48] != Sha256::digest(&data[HEADER..PREFIX])[..] {
        return Err("State preview is damaged".into());
    }
    Ok(len as usize)
}

/// SL2 stores bounded cheat ancestry with the snapshot in one atomic file.
pub(crate) fn encode(machine: &Machine, preview: &[[u8; 4]], history: &crate::cheat_provenance::History) -> Result<Vec<u8>, String> {
    if preview.len() != PREVIEW_W * PREVIEW_H { return Err("Invalid state preview size".into()); }
    let state = machine.save_state()?;
    let pixels: Vec<u8> = preview.iter().flat_map(|p| p.iter().copied()).collect();
    let mut bytes = Vec::with_capacity(PREFIX + state.len());
    bytes.extend_from_slice(if history.is_empty() { SLOT_MAGIC } else { TAGGED_MAGIC });
    bytes.extend_from_slice(&(state.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&Sha256::digest(&pixels));
    bytes.extend_from_slice(&pixels);
    bytes.extend_from_slice(&state);
    if !history.is_empty() {
        history.validate()?;
        let metadata = serde_json::to_vec(history).map_err(|e| e.to_string())?;
        if metadata.len() > MAX_METADATA_BYTES { return Err("Cheat ancestry exceeds the size limit".into()); }
        bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        let mut hash = Sha256::new(); hash.update(&state); hash.update(&metadata);
        bytes.extend_from_slice(&hash.finalize());
        bytes.extend_from_slice(&metadata);
    }
    Ok(bytes)
}

fn parts(bytes: &[u8]) -> Result<(&[u8], crate::cheat_provenance::History), String> {
    if bytes.len() > MAX_FILE_BYTES { return Err("State exceeds the size limit".into()); }
    if !bytes.starts_with(SLOT_MAGIC) && !bytes.starts_with(TAGGED_MAGIC) {
        return Ok((bytes, Default::default()));
    }
    let len = validate_prefix(bytes)?;
    let end = PREFIX.checked_add(len).ok_or("Invalid state size")?;
    if bytes.len() < end { return Err("Incomplete state file".into()); }
    let history = if bytes.starts_with(TAGGED_MAGIC) {
        let tail = &bytes[end..];
        if tail.len() < 36 { return Err("Missing cheat ancestry".into()); }
        let n = u32::from_le_bytes(tail[..4].try_into().unwrap()) as usize;
        if n > MAX_METADATA_BYTES || tail.len() != 36 + n { return Err("Invalid cheat ancestry length".into()); }
        let mut hash = Sha256::new(); hash.update(&bytes[PREFIX..end]); hash.update(&tail[36..]);
        if tail[4..36] != hash.finalize()[..] { return Err("Cheat ancestry checksum mismatch".into()); }
        let history: crate::cheat_provenance::History = serde_json::from_slice(&tail[36..]).map_err(|e| format!("Invalid cheat ancestry: {e}"))?;
        history.validate()?;
        history
    } else {
        if bytes.len() != end { return Err("Incomplete state file".into()); }
        Default::default()
    };
    Ok((&bytes[PREFIX..end], history))
}

/// Check envelope integrity without loading a machine or reading guest I/O.
pub(crate) fn preview(bytes: &[u8]) -> Result<Vec<[u8;4]>, String> {
    let (core, _) = parts(bytes)?;
    let preview = if bytes.starts_with(SLOT_MAGIC) || bytes.starts_with(TAGGED_MAGIC) {
        bytes[HEADER..PREFIX].chunks_exact(4).map(|p|p.try_into().unwrap()).collect()
    } else { Vec::new() };
    if core.len() < 108 || core.len() > MAX_STATE_BYTES || &core[..8] != b"FFTA-ST\0"
        || u32::from_le_bytes(core[8..12].try_into().unwrap()) != 1 {
        return Err("Invalid or incompatible snapshot header".into());
    }
    if core[76..108] != Sha256::digest(&core[108..])[..] { return Err("Snapshot checksum mismatch".into()); }
    Ok(preview)
}

/// Authenticate a selectable snapshot's envelope and ROM without loading it.
/// The eventual load still checks BIOS, decoded fields and live compatibility.
pub(crate) fn reference_identity(bytes:&[u8],rom:Option<&str>)->Result<(),String>{
    preview(bytes)?;
    let (core, _) = parts(bytes)?;
    let actual=core[12..44].iter().map(|b|format!("{b:02x}")).collect::<String>();
    if rom.is_some_and(|r|r!=actual){return Err("Snapshot belongs to a different ROM".into());}
    Ok(())
}

/// Read a desktop slot or legacy raw snapshot through the same transactional validator.
pub(crate) fn load(machine: &mut Machine, path: &Path) -> Result<(), String> {
    let bytes = read_file(path).map_err(|e| e.to_string())?;
    load_bytes(machine, &bytes)
}

pub(crate) fn load_bytes(machine: &mut Machine, bytes: &[u8]) -> Result<(), String> {
    let history = load_bytes_with_history(machine, bytes)?;
    if !history.is_empty() { eprintln!("Snapshot has observed cheat ancestry; do not treat results as ordinary-play evidence"); }
    Ok(())
}

pub(crate) fn load_bytes_with_history(machine: &mut Machine, bytes: &[u8]) -> Result<crate::cheat_provenance::History, String> {
    let (core, history) = parts(bytes)?;
    machine.load_state(core)?;
    Ok(history)
}

pub(crate) fn load_with_history(machine: &mut Machine, path: &Path) -> Result<crate::cheat_provenance::History, String> {
    let bytes = read_file(path).map_err(|e| e.to_string())?;
    load_bytes_with_history(machine, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn preview() -> Vec<[u8; 4]> {
        vec![[12, 34, 56, 255]; PREVIEW_W * PREVIEW_H]
    }

    #[test]
    fn cheat_snapshot_roundtrip_integrity_load_and_undo() {
        let dir = tempfile::tempdir().unwrap();
        let mut machine = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &machine.bus.rom);
        let mut history = crate::cheat_provenance::History::default();
        history.record("once", serde_json::json!({"id":"synthetic","plan_sha256":"test"}), 12, 34, 1);
        machine.cpu.regs[4] = 42;
        slots.save_with_history(&machine, 1, &preview(), false, &history).unwrap();
        let bytes = read_file(&slots.path(1).unwrap()).unwrap();
        assert!(bytes.starts_with(TAGGED_MAGIC));
        assert_eq!(super::preview(&bytes).unwrap(), preview());
        assert!(slots.info(1).status.contains("cheat"));
        reference_identity(&bytes, Some(&gba_core::ffta_rom::digest(&machine.bus.rom))).unwrap();
        machine.cpu.regs[4] = 77;
        let loaded = slots.load_with_history(&mut machine, 1).unwrap();
        assert_eq!(machine.cpu.regs[4], 42);
        assert_eq!(loaded.events, history.events);
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Load(1));
        assert_eq!(machine.cpu.regs[4], 77);
        let before = machine.save_state().unwrap();
        for offset in [16, PREFIX, bytes.len()-1] {
            let mut damaged = bytes.clone(); damaged[offset] ^= 1;
            assert!(load_bytes_with_history(&mut machine, &damaged).is_err());
            assert_eq!(machine.save_state().unwrap(), before);
        }
        let mut extra = bytes.clone(); extra.push(0);
        assert!(load_bytes_with_history(&mut machine, &extra).is_err());
        assert_eq!(machine.save_state().unwrap(), before);
        let state_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let tail = PREFIX + state_len;
        let mut unsupported = bytes.clone();
        let mut metadata: serde_json::Value = serde_json::from_slice(&unsupported[tail + 36..]).unwrap();
        metadata["schema"] = serde_json::json!(2);
        let metadata = serde_json::to_vec(&metadata).unwrap();
        unsupported.truncate(tail);
        unsupported.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        let mut hash = Sha256::new(); hash.update(&unsupported[PREFIX..tail]); hash.update(&metadata);
        unsupported.extend_from_slice(&hash.finalize()); unsupported.extend_from_slice(&metadata);
        assert!(load_bytes_with_history(&mut machine, &unsupported).is_err());
        assert_eq!(machine.save_state().unwrap(), before);
        let mut other = Machine::new(vec![1; 256]); let original = other.save_state().unwrap();
        assert!(load_bytes_with_history(&mut other, &bytes).is_err());
        assert_eq!(other.save_state().unwrap(), original);
        slots.save_with_history(&machine, 1, &preview(), true, &history).unwrap();
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Save(1));
        assert_eq!(read_file(&slots.path(1).unwrap()).unwrap(), bytes);
    }

    #[test]
    fn undo_save_restores_original_preview_and_empty_slot_without_rewinding_game() {
        let dir = tempfile::tempdir().unwrap();
        let mut machine = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &machine.bus.rom);
        assert!(slots.undo_label().is_none());
        slots.save(&machine, 9, &preview(), false).unwrap();
        machine.cpu.regs[4] = 7;
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Save(9));
        assert_eq!(machine.cpu.regs[4], 7);
        assert!(!slots.info(9).occupied);
        assert!(slots.undo(&mut machine).is_err()); // No redo or older undo.
        slots.save(&machine, 9, &preview(), false).unwrap();
        let original = read_file(&slots.path(9).unwrap()).unwrap();
        let replacement = vec![[100, 200, 50, 255]; PREVIEW_W * PREVIEW_H];
        machine.cpu.regs[4] = 42;
        slots.save(&machine, 9, &replacement, true).unwrap();
        machine.cpu.regs[4] = 99;
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Save(9));
        assert_eq!(read_file(&slots.path(9).unwrap()).unwrap(), original);
        assert_eq!(slots.info(9).preview, preview());
        assert_eq!(machine.cpu.regs[4], 99);
        assert!(slots.undo_label().is_none());
        assert!(Slots::new(&dir.path().join("test.sav"), &machine.bus.rom)
            .undo_label()
            .is_none());
    }

    #[test]
    fn undo_load_restores_exact_preload_machine_even_after_playing() {
        let dir = tempfile::tempdir().unwrap();
        let mut machine = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &machine.bus.rom);
        slots.save(&machine, 1, &preview(), false).unwrap();
        let file = read_file(&slots.path(1).unwrap()).unwrap();
        machine.cpu.regs[4] = 42;
        machine.bus.ewram[10] = 9;
        machine.bus.tick(123);
        let before = machine.save_state().unwrap();
        slots.load(&mut machine, 1).unwrap();
        assert_eq!(machine.cpu.regs[4], 0);
        assert!(slots.undo_restores_machine());
        assert_eq!(slots.undo_label().as_deref(), Some("Undo load (slot 1)"));
        machine.cpu.regs[4] = 77;
        machine.bus.tick(4000);
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Load(1));
        assert_eq!(machine.save_state().unwrap(), before);
        assert_eq!(read_file(&slots.path(1).unwrap()).unwrap(), file);
        assert!(slots.undo_label().is_none());
        assert!(slots.undo(&mut machine).is_err());
        assert_eq!(machine.save_state().unwrap(), before);
    }

    #[test]
    fn only_successful_operations_replace_undo_across_slots_and_action_kinds() {
        let dir = tempfile::tempdir().unwrap();
        let mut machine = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &machine.bus.rom);
        slots.save(&machine, 1, &preview(), false).unwrap();
        machine.cpu.regs[4] = 42;
        let before = machine.save_state().unwrap();
        slots.load(&mut machine, 1).unwrap();
        assert!(slots.save(&machine, 1, &preview(), false).is_err());
        assert!(slots.load(&mut machine, 9).is_err());
        assert!(slots.save(&machine, 2, &[], false).is_err());
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Load(1));
        assert_eq!(machine.save_state().unwrap(), before);
        slots.load(&mut machine, 1).unwrap();
        slots.save(&machine, 2, &preview(), false).unwrap();
        let after_save = machine.save_state().unwrap();
        assert!(slots.load(&mut machine, 9).is_err());
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Save(2));
        assert_eq!(machine.save_state().unwrap(), after_save);
        assert!(slots.info(1).occupied);
        assert!(!slots.info(2).occupied);
        assert!(slots.undo(&mut machine).is_err());
    }

    #[test]
    fn failed_undo_preserves_external_changes_and_can_be_retried() {
        let dir = tempfile::tempdir().unwrap();
        let mut machine = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &machine.bus.rom);
        slots.save(&machine, 1, &preview(), false).unwrap();
        let path = slots.path(1).unwrap();
        let written = read_file(&path).unwrap();
        std::fs::write(&path, b"external changes").unwrap();
        assert!(slots.undo(&mut machine).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"external changes");
        assert!(slots.undo_label().is_some());
        std::fs::remove_file(&path).unwrap();
        assert!(slots.undo(&mut machine).is_err());
        assert!(slots.undo_label().is_some());
        std::fs::write(&path, written).unwrap();
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Save(1));
        slots.save(&machine, 1, &preview(), false).unwrap();
        slots.load(&mut machine, 1).unwrap();
        let mut other = Machine::new(vec![1; 256]);
        let original = other.save_state().unwrap();
        assert!(slots.undo(&mut other).is_err());
        assert_eq!(other.save_state().unwrap(), original);
        assert!(slots.undo_restores_machine());
        assert_eq!(slots.undo(&mut machine).unwrap(), UndoEffect::Load(1));
    }

    #[test]
    fn undo_overwrite_restores_legacy_file_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let mut machine = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &machine.bus.rom);
        std::fs::create_dir_all(&slots.directory).unwrap();
        let legacy = machine.save_state().unwrap();
        std::fs::write(slots.path(1).unwrap(), &legacy).unwrap();
        slots.save(&machine, 1, &preview(), true).unwrap();
        slots.undo(&mut machine).unwrap();
        assert_eq!(read_file(&slots.path(1).unwrap()).unwrap(), legacy);
        assert!(slots.info(1).preview.is_empty());
    }

    #[test]
    fn nine_slots_are_stable_isolated_and_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let rom = vec![0; 256];
        let mut slots = Slots::new(&dir.path().join("cartridge.sav"), &rom);
        let renamed = Slots::new(&dir.path().join("renamed.sav"), &rom);
        let other = Slots::new(&dir.path().join("cartridge.sav"), &[1; 256]);
        assert_eq!(slots.path(1).unwrap(), renamed.path(1).unwrap());
        assert_ne!(slots.path(1).unwrap(), other.path(1).unwrap());
        assert!(slots.path(0).is_err());
        assert!(slots.path(10).is_err());
        let mut m = Machine::new(rom.clone());
        for slot in 1..=SLOT_COUNT {
            assert!(!slots.info(slot).occupied);
            m.cpu.regs[4] = slot as u32;
            slots.save(&m, slot, &preview(), false).unwrap();
        }
        let mut fresh = Machine::new(rom);
        let mut slots = Slots::new(&dir.path().join("cartridge.sav"), &fresh.bus.rom);
        for slot in 1..=SLOT_COUNT {
            slots.load(&mut fresh, slot).unwrap();
            assert_eq!(fresh.cpu.regs[4], slot as u32);
            assert_eq!(slots.info(slot).preview, preview());
        }
        assert_eq!(std::fs::read_dir(&slots.directory).unwrap().count(), 9);
    }

    #[test]
    fn extended_slots_preserve_original_paths_and_high_slot_undo_across_toggles() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = Machine::new(vec![0;256]);
        let mut slots = Slots::new(&dir.path().join("ffta.sav"), &m.bus.rom);
        let first = slots.path(1).unwrap();
        slots.set_extended(true);
        assert_eq!(slots.count(),99);
        assert_eq!(slots.path(1).unwrap(),first);
        assert!(slots.path(0).is_err());
        assert!(slots.path(100).is_err());
        for slot in [9,10,99] {
            m.cpu.regs[4] = slot as u32;
            slots.save(&m,slot,&preview(),false).unwrap();
        }
        let high = slots.path(99).unwrap();
        let bytes = std::fs::read(&high).unwrap();
        assert!(slots.save(&m,99,&preview(),false).is_err());
        slots.set_extended(false);
        assert!(slots.path(99).is_err());
        assert!(slots.undo(&mut m).is_err());
        assert_eq!(std::fs::read(&high).unwrap(),bytes);
        assert!(slots.undo_label().unwrap().contains("99"));
        slots.set_extended(true);
        assert_eq!(slots.undo(&mut m).unwrap(),UndoEffect::Save(99));
        assert!(!high.exists());
        m.cpu.regs[4] = 55;
        slots.load(&mut m,10).unwrap();
        assert_eq!(m.cpu.regs[4],10);
        assert_eq!(slots.undo(&mut m).unwrap(),UndoEffect::Load(10));
        assert_eq!(m.cpu.regs[4],55);
        assert_eq!(std::fs::read_dir(&slots.directory).unwrap().count(),2);
    }

    #[test]
    fn overwrite_requires_confirmation_and_preview_matches_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &m.bus.rom);
        slots.save(&m, 1, &preview(), false).unwrap();
        let original = std::fs::read(slots.path(1).unwrap()).unwrap();
        m.cpu.regs[4] = 42;
        assert!(slots.save(&m, 1, &preview(), false).is_err());
        assert_eq!(std::fs::read(slots.path(1).unwrap()).unwrap(), original);
        let replacement = vec![[89, 65, 43, 255]; PREVIEW_W * PREVIEW_H];
        slots.save(&m, 1, &replacement, true).unwrap();
        assert_eq!(slots.info(1).preview, replacement);
        slots.load(&mut m, 1).unwrap();
        assert_eq!(m.cpu.regs[4], 42);
        assert_eq!(std::fs::read_dir(&slots.directory).unwrap().count(), 1);
    }

    #[test]
    fn damage_missing_and_wrong_rom_do_not_mutate_machine() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &m.bus.rom);
        let original = m.save_state().unwrap();
        assert!(slots.load(&mut m, 1).is_err());
        slots.save(&m, 1, &preview(), false).unwrap();
        let bytes = std::fs::read(slots.path(1).unwrap()).unwrap();
        for offset in [0, 8, HEADER, PREFIX + 12, bytes.len() - 1] {
            let mut bad = bytes.clone();
            bad[offset] ^= 1;
            std::fs::write(slots.path(1).unwrap(), &bad).unwrap();
            assert!(slots.load(&mut m, 1).is_err());
            assert_eq!(m.save_state().unwrap(), original);
        }
        std::fs::write(slots.path(1).unwrap(), &bytes).unwrap();
        let mut other = Machine::new(vec![1; 256]);
        let before = other.save_state().unwrap();
        assert!(slots.load(&mut other, 1).is_err());
        assert_eq!(other.save_state().unwrap(), before);
    }

    #[test]
    fn old_states_load_without_preview_and_bad_preview_never_clobbers_slot() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = Machine::new(vec![0; 256]);
        let mut slots = Slots::new(&dir.path().join("test.sav"), &m.bus.rom);
        std::fs::create_dir_all(&slots.directory).unwrap();
        m.cpu.regs[4] = 99;
        let original = m.save_state().unwrap();
        std::fs::write(slots.path(1).unwrap(), &original).unwrap();
        let info = slots.info(1);
        assert!(info.occupied);
        assert!(info.preview.is_empty());
        m.cpu.regs[4] = 0;
        slots.load(&mut m, 1).unwrap();
        assert_eq!(m.cpu.regs[4], 99);
        assert!(slots.save(&m, 1, &[], true).is_err());
        assert_eq!(std::fs::read(slots.path(1).unwrap()).unwrap(), original);
        std::fs::write(slots.path(2).unwrap(), b"broken").unwrap();
        assert_eq!(slots.info(2).status, "Unreadable");
        assert!(slots.info(2).occupied);
    }
}
