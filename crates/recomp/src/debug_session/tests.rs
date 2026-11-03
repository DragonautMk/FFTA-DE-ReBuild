use super::*;

struct Fixture { dir: tempfile::TempDir, rom: Vec<u8>, save: Vec<u8>, state: Vec<u8>, executable: PathBuf }
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let mut rom = vec![0; 0x200]; rom[3] = 0xea; rom[0x100..0x106].copy_from_slice(b"SRAM_V");
        let save = vec![7; 65536];
        let mut machine = crate::make_machine(rom.clone(), None);
        machine.bus.load_save_data(&save); machine.bus.frames = 17; machine.bus.clock = 100;
        let state = machine.save_state().unwrap();
        let executable = dir.path().join("original runtime");
        for (p, b) in [(dir.path().join("original.gba"), rom.as_slice()), (dir.path().join("original.sav"), save.as_slice()),
            (dir.path().join("original.ffstate"), state.as_slice()), (executable.clone(), b"public executable fixture".as_slice())] {
            fs::write(p, b).unwrap();
        }
        Self { dir, rom, save, state, executable }
    }
    fn source(&self) -> Source<'_> {
        let mut av = input_config::AvConfig::default(); av.debug_game_save_slot = 99;
        Source { rom: &self.rom, bios: None, save: Some(&self.save), state: Some(&self.state),
            av, input: input_config::InputConfig::default(), interp: true, dataset: None, executable: &self.executable }
    }
    fn root(&self) -> PathBuf { self.dir.path().join("isolated case") }
    fn originals(&self) -> Vec<Vec<u8>> {
        ["original.gba", "original.sav", "original.ffstate", "original runtime"].iter().map(|p| fs::read(self.dir.path().join(p)).unwrap()).collect()
    }
}

#[test]
fn load_undo_restart_backup_autosave_and_exit_targets_stay_in_the_copy() {
    let f = Fixture::new(); let originals = f.originals();
    let root = create(&f.root(), f.source()).unwrap();
    let session = open(&root).unwrap();
    let save = root.join("Data/saves/ffta.sav");
    let mut machine = crate::make_machine(fs::read(session.rom()).unwrap(), None);
    let boot = machine.save_state().unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    machine.bus.load_save_data(&backups.capture_startup().unwrap().unwrap());
    crate::save_state::load(&mut machine, &session.state().unwrap()).unwrap();
    assert_eq!(machine.bus.frames, 17);
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots.set_extended(true);
    let preview = vec![[0,0,0,255]; 240*160];
    slots.save(&machine, 99, &preview, false).unwrap();
    assert!(slots.path(99).unwrap().is_file());
    slots.undo(&mut machine).unwrap(); assert!(!slots.path(99).unwrap().exists());
    machine.bus.frames = 23;
    slots.load(&mut machine, 1).unwrap(); assert_eq!(machine.bus.frames, 17);
    slots.undo(&mut machine).unwrap(); assert_eq!(machine.bus.frames, 23);
    machine.bus.load_save_data(&vec![9;65536]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(fs::read(&save).unwrap(), vec![9;65536]);
    assert_eq!(machine.bus.frames, 0);
    // Same writer as periodic autosave/final teardown, with the session-selected path.
    crate::write_sav(save.to_str().unwrap(), machine.bus.save_data().unwrap()).unwrap();
    session.result(&Ok(())); assert!(root.join("last-result.json").is_file());
    assert_eq!(f.originals(), originals);
    assert_eq!(input_config::AvConfig::parse(&fs::read_to_string(root.join("Data/config/av.cfg")).unwrap()).debug_game_save_slot,1);
    assert_eq!(read_manifest(&root).unwrap().label, "isolated case");
}

#[test]
fn move_and_reopen_preserve_relative_inputs_and_exclusive_session_ownership() {
    let f = Fixture::new(); let root = create(&f.root(), f.source()).unwrap();
    let active = open(&root).unwrap(); assert!(open(&root).is_err());
    drop(active); drop(open(&root).unwrap());
    let moved = f.dir.path().join("moved case with spaces"); fs::rename(&root, &moved).unwrap();
    let session = open(&moved).unwrap();
    assert_eq!(fs::read(session.rom()).unwrap(),f.rom);
    assert_eq!(fs::read(session.state().unwrap()).unwrap(),f.state);
    let text = fs::read_to_string(moved.join(MANIFEST)).unwrap();
    assert!(!text.contains(&root.to_string_lossy().replace('\\',"\\\\")));
    assert!(!root.exists());
}

#[test]
fn bad_snapshot_save_or_existing_destination_preserves_all_originals() {
    let f = Fixture::new(); let originals = f.originals();
    let mut source=f.source();source.state=Some(b"broken state");
    assert!(create(&f.root(),source).is_err());assert!(!f.root().exists());
    let mut source=f.source();source.save=Some(b"wrong size");
    assert!(create(&f.root(),source).is_err());assert!(!f.root().exists());
    fs::create_dir(f.root()).unwrap();fs::write(f.root().join("keep"),b"keep").unwrap();
    assert!(create(&f.root(),f.source()).is_err());
    assert_eq!(fs::read(f.root().join("keep")).unwrap(),b"keep");assert_eq!(f.originals(),originals);
}

#[test]
fn source_changes_do_not_retarget_the_session_and_seed_tampering_is_detected() {
    let f=Fixture::new();let root=create(&f.root(),f.source()).unwrap();
    fs::write(f.dir.path().join("original.gba"),b"new original file").unwrap();
    assert_eq!(fs::read(root.join("ffta.gba")).unwrap(),f.rom);
    assert!(read_manifest(&root).is_ok());
    fs::write(root.join("seed.ffstate"),b"changed seed").unwrap();
    assert!(read_manifest(&root).unwrap_err().contains("input changed"));
    assert_eq!(fs::read(f.dir.path().join("original.ffstate")).unwrap(),f.state);
}

#[test]
fn linked_mutable_save_and_manifest_path_escape_are_rejected_before_launch() {
    let f=Fixture::new();let root=create(&f.root(),f.source()).unwrap();
    let save=root.join("Data/saves/ffta.sav");fs::remove_file(&save).unwrap();
    fs::hard_link(f.dir.path().join("original.sav"),&save).unwrap();
    assert!(read_manifest(&root).unwrap_err().contains("unshared"));
    fs::remove_file(save).unwrap();
    let mut manifest=read_manifest(&root).unwrap();
    manifest.pins.insert("../original.sav".into(),gba_core::ffta_rom::digest(&f.save));
    fs::write(root.join(MANIFEST),serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(read_manifest(&root).unwrap_err().contains("relative"));
    assert_eq!(fs::read(f.dir.path().join("original.sav")).unwrap(),f.save);
}

#[test]
fn child_environment_scrubs_inherited_project_paths_and_preserves_host_environment() {
    let mut command=Command::new("fixture");
    let names=["FFTA_SAVE_PATH","GBA_RECOMP_BIOS","RECOMP_RECORD_INPUT","ffta_resources_dir","PATH","DISPLAY"];
    scrub(&mut command,names.iter().map(std::ffi::OsString::from));
    let removed=command.get_envs().filter(|(_,v)|v.is_none()).map(|(n,_)|n.to_string_lossy().into_owned()).collect::<Vec<_>>();
    for key in &names[..4] { assert!(removed.iter().any(|n|n.eq_ignore_ascii_case(key))); }
    assert!(!removed.iter().any(|n|n=="PATH"||n=="DISPLAY"));
}
