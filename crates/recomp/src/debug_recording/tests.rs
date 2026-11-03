use super::*;

fn machine() -> Machine {
    let mut rom = vec![0; 0x200];
    rom[..4].copy_from_slice(&0xeaff_fffeu32.to_le_bytes()); // Original-format ARM loop fixture.
    rom[0x100..0x106].copy_from_slice(b"SRAM_V");
    let mut m = crate::make_machine(rom, None);
    m.bus.frames = 17;
    m.bus.keys = 0x3fe; // A already held at the origin, before any new press event.
    m
}
fn start(path: &Path, machine: &Machine) -> Capture {
    Capture::start(
        path,
        machine,
        true,
        None,
        &input_config::AvConfig::default(),
        &input_config::InputConfig::default(),
    )
    .unwrap()
}
fn step(m: &mut Machine) {
    m.run_frame(5_000_000);
    m.bus.audio_buf.clear();
    m.bus.psg_tap.clear();
    m.bus.hle_tap.clear();
    m.bus.fifo_tap[0].clear();
    m.bus.fifo_tap[1].clear();
}
fn complete(path: &Path) -> Machine {
    let mut m = machine();
    let mut c = start(path, &m);
    for n in 0..6 {
        m.bus.keys = if n < 2 {
            0x3fe
        } else if n == 4 {
            0x37f
        } else {
            0x3ff
        };
        c.before_frame(&m, &[]).unwrap();
        step(&mut m);
    }
    c.finish(&m, "test stopped", true).unwrap();
    m
}

#[test]
fn snapshot_origin_held_keys_and_relative_events_replay_exact_state_clock() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("capture");
    let endpoint = complete(&root);
    let text = fs::read_to_string(root.join("input.input")).unwrap();
    assert_eq!(text, "gba-input v1\n0 3fe\n2 3ff\n4 37f\n5 3ff\n");
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("recording.json")).unwrap()).unwrap();
    assert_eq!(
        (manifest.first_frame, manifest.initial_keys, manifest.frames),
        (17, 0x3fe, 6)
    );
    let report = verify(&root, &mut machine()).unwrap();
    assert_eq!(report["ok"], true, "{report}");
    assert_eq!(report["clock"], endpoint.bus.clock);
}

#[test]
fn start_stop_observe_only_and_refuse_existing_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("capture");
    let m = machine();
    let original = m.save_state().unwrap();
    let c = start(&root, &m);
    assert_eq!(m.save_state().unwrap(), original);
    assert!(Capture::start(
        &root,
        &m,
        true,
        None,
        &input_config::AvConfig::default(),
        &input_config::InputConfig::default()
    )
    .is_err());
    c.finish(&m, "no frames", true).unwrap();
    assert_eq!(m.save_state().unwrap(), original);
    assert_eq!(verify(&root, &mut machine()).unwrap()["ok"], true);
}

#[test]
fn rewind_incomplete_frame_and_output_failure_never_publish_complete() {
    let temp = tempfile::tempdir().unwrap();
    let mut m = machine();
    let root = temp.path().join("rewind");
    let mut c = start(&root, &m);
    m.bus.frames -= 1;
    assert!(c.before_frame(&m, &[]).unwrap_err().contains("timeline"));
    assert!(c.finish(&m, "rewind", false).is_err());
    assert!(verify(&root, &mut machine())
        .unwrap_err()
        .contains("incomplete"));
    assert!(!root.join("end.ffstate").exists());
    let m = machine();
    let root = temp.path().join("partial-frame");
    let mut c = start(&root, &m);
    c.before_frame(&m, &[]).unwrap();
    assert!(c.finish(&m, "step guard", true).is_err());
    let root = temp.path().join("write-failure");
    let c = start(&root, &m);
    fs::create_dir(root.join("end.ffstate")).unwrap();
    assert!(c.finish(&m, "stop", true).is_err());
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("recording.json")).unwrap()).unwrap();
    assert_eq!(manifest.status, "recording");
}

#[test]
fn input_order_same_frame_events_and_limits_are_explicit() {
    let text = b"gba-input v1\n0 3fe\n0 3ff\n0 37f\n1 mouse:4\n1 mouse:1\n1 3ff\n";
    let s = script(text, 2).unwrap();
    assert_eq!(s.keys_at(0), 0x37f);
    assert_eq!(s.1, [(1, 4), (1, 1)]);
    for text in [
        "gba-input v1\n1 3ff\n",
        "gba-input v1\n0 mouse:4\n",
        "gba-input v1\n0 fff\n",
        "gba-input v1\n0 3ff\n2 3fe\n1 3ff\n",
        "gba-input v1\n0 3ff\n3 3fe\n",
    ] {
        assert!(script(text.as_bytes(), 2).is_err());
    }
    let temp = tempfile::tempdir().unwrap();
    let m = machine();
    let mut c = start(&temp.path().join("limit"), &m);
    c.manifest.events = MAX_EVENTS;
    assert!(c.event("0 3ff\n").unwrap_err().contains("limit"));
    c.bytes = MAX_INPUT;
    assert!(c.write(b"x").unwrap_err().contains("limit"));
    c.next_frame = m.bus.frames + MAX_FRAMES;
    let mut m = m;
    m.bus.frames = c.next_frame;
    assert!(c.before_frame(&m, &[]).unwrap_err().contains("limit"));
}

#[test]
fn corrupt_files_wrong_rom_build_and_native_mode_fail_preflight() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("capture");
    complete(&root);
    let input = fs::read(root.join("input.input")).unwrap();
    fs::write(root.join("input.input"), b"changed").unwrap();
    assert!(verify(&root, &mut machine())
        .unwrap_err()
        .contains("checksum"));
    fs::write(root.join("input.input"), input).unwrap();
    let mut m = machine();
    m.bus.rom[0x180] = 9;
    assert!(verify(&root, &mut m).unwrap_err().contains("ROM/BIOS"));
    let p = root.join("recording.json");
    let mut manifest: Manifest = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    manifest.executable_sha256 = "wrong build".into();
    fs::write(&p, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(verify(&root, &mut machine())
        .unwrap_err()
        .contains("executable"));
    manifest.interp = false;
    fs::write(&p, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(verify(&root, &mut machine())
        .unwrap_err()
        .contains("interpreter"));
}

#[test]
fn changed_endpoint_is_a_reported_divergence_not_a_success() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("capture");
    let mut end = complete(&root);
    end.bus.ewram[0x180] ^= 1;
    let bytes = end.save_state().unwrap();
    fs::write(root.join("end.ffstate"), &bytes).unwrap();
    let p = root.join("recording.json");
    let mut manifest: Manifest = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    manifest.end_sha256 = Some(gba_core::ffta_rom::digest(&bytes));
    fs::write(&p, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert_eq!(verify(&root, &mut machine()).unwrap()["ok"], false);
}

#[test]
fn recording_folder_is_movable_with_relative_payload_names() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("capture");
    complete(&root);
    let moved = temp.path().join("moved capture");
    fs::rename(root, &moved).unwrap();
    assert_eq!(verify(&moved, &mut machine()).unwrap()["ok"], true);
}
