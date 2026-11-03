use super::*;

fn fixture() -> (tempfile::TempDir, Request) {
    let dir = tempfile::tempdir().unwrap();
    let rom = dir.path().join("test cartridge.gba");
    let mut bytes = vec![0; 0xc0]; bytes[3] = 0xea;
    std::fs::write(&rom, bytes).unwrap();
    let save = dir.path().join("test-slot-20.sav");
    std::fs::write(&save, vec![0x55; 65536]).unwrap();
    (dir, Request { rom, save: Some(save), no_bios: true, interp: true, ..Default::default() })
}

fn codes(report: &Value) -> Vec<&str> {
    report["issues"].as_array().unwrap().iter().map(|v| v["code"].as_str().unwrap()).collect()
}

#[test]
fn preflight_validates_snapshot_input_and_preserves_inputs_without_executing() {
    let (dir, mut req) = fixture();
    let mut machine = crate::make_machine(std::fs::read(&req.rom).unwrap(), None);
    machine.bus.frames = 123; machine.bus.clock = 456;
    let state = dir.path().join("checkpoint.ffstate");
    std::fs::write(&state, machine.save_state().unwrap()).unwrap();
    let input = dir.path().join("input.txt");
    std::fs::write(&input, "gba-input v1\n0 3fe\n2 3ff\n").unwrap();
    req.state = Some(state.clone()); req.input = Some(input.clone());
    let paths = [&req.rom, req.save.as_ref().unwrap(), &state, &input];
    let before: Vec<_> = paths.iter().map(|p| std::fs::read(p).unwrap()).collect();
    let report = collect(&req, None);
    assert_eq!(report["ok"], true, "{report:#}");
    assert_eq!(report["schema"], 1);
    assert_eq!(report["game_save_slot"], 20);
    assert_eq!(report["context"], "preflight-no-execution");
    assert_eq!(report["build"]["executable"]["sha256"].as_str().unwrap().len(), 64);
    assert!(report["execution"]["frame"].is_null());
    for (p, bytes) in paths.iter().zip(before) { assert_eq!(std::fs::read(p).unwrap(), bytes); }
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 4);
    assert_eq!(machine.bus.frames, 123); assert_eq!(machine.bus.clock, 456);
}

#[test]
fn failures_are_structured_and_do_not_replace_or_create_input_files() {
    let (dir, mut req) = fixture();
    let input = dir.path().join("bad.txt");
    std::fs::write(&input, "gba-input v1\n0 400\n").unwrap();
    let state = dir.path().join("broken.ffstate");
    std::fs::write(&state, b"broken").unwrap();
    req.input = Some(input.clone()); req.state = Some(state.clone());
    req.dataset = Some(dir.path().join("missing dataset"));
    req.bios = Some(dir.path().join("missing BIOS")); req.no_bios = false;
    req.require_audit = true;
    let report = collect(&req, None);
    assert_eq!(report["ok"], false);
    for code in ["bios", "snapshot", "input-script", "dataset"] { assert!(codes(&report).contains(&code), "{report:#}"); }
    assert_eq!(codes(&report).contains(&"feature-rom-audit"), !cfg!(feature="rom-data-read-audit"));
    assert!(!req.dataset.as_ref().unwrap().exists());
    assert!(!req.bios.as_ref().unwrap().exists());
    assert_eq!(std::fs::read(state).unwrap(), b"broken");
    assert_eq!(std::fs::read_to_string(input).unwrap(), "gba-input v1\n0 400\n");
    assert!(report["issues"].as_array().unwrap().iter().all(|i| !i["remedy"].as_str().unwrap().is_empty()));
}

#[test]
fn live_report_uses_loaded_bytes_and_never_modifies_guest_state() {
    let (_dir, mut req) = fixture();
    let machine = crate::make_machine(std::fs::read(&req.rom).unwrap(), None);
    let before = machine.save_state().unwrap();
    req.slot = Some(99);
    let report = collect(&req, Some(&machine));
    assert_eq!(report["ok"], true, "{report:#}");
    assert_eq!(report["snapshot"]["file"]["exists"], false);
    assert_eq!(report["snapshot"]["meaning"], "picker-selection-not-live-state");
    assert_eq!(report["rom"]["effective_sha256"], gba_core::ffta_rom::digest(&machine.bus.rom));
    assert_eq!(machine.save_state().unwrap(), before);
    std::fs::write(&req.rom, b"changed on disk").unwrap();
    let changed = collect(&req, Some(&machine));
    assert!(codes(&changed).contains(&"rom-changed"));
    assert_eq!(changed["rom"]["effective_sha256"], report["rom"]["effective_sha256"]);
    assert_eq!(machine.save_state().unwrap(), before);
}

#[test]
fn report_refresh_is_bounded_atomic_and_refuses_other_files() {
    let (dir, mut req) = fixture();
    let path = dir.path().join("reports/session.json"); req.out = Some(path.clone());
    let report = collect(&req, None);
    write_report(&path, &report).unwrap();
    write_report(&path, &report).unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()).unwrap(), report);
    assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    let original = std::fs::read(&req.rom).unwrap();
    assert!(write_report(&req.rom, &report).is_err());
    assert_eq!(std::fs::read(&req.rom).unwrap(), original);
    let mut big = report.clone(); big["oversize"] = Value::String("x".repeat(MAX_REPORT as usize));
    let previous = std::fs::read(&path).unwrap();
    assert!(write_report(&path, &big).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), previous);
}

#[test]
fn parser_rejects_ambiguous_options_before_opening_inputs() {
    for args in [vec!["rom.gba", "--bios", "bios.bin", "--no-bios"],
        vec!["rom.gba", "--state", "s", "--slot", "1"], vec!["rom.gba", "--slot", "100"],
        vec!["rom.gba", "--dataset", "d", "--rom-data"], vec!["rom.gba", "--typo"]] {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        assert!(parse(&args).is_err());
    }
}
