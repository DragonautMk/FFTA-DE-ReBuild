use super::*;
use std::fs;
fn rom() -> String {
    "a".repeat(64)
}
#[test]
fn cheat_catalog_selection_authenticates_type_and_changes_without_execution(){
    let t=tempfile::tempdir().unwrap();let path=t.path().join("catalogo.json");
    let bytes=br#"{"schema":1,"entries":[{"id":"CH-0001","format":"codebreaker","region":"EUR","section":"synthetic","title":"Example","status":"unverified","codes":["A","B"],"notes":[],"aliases":[],"provenance":[]}]}"#;
    fs::write(&path,bytes).unwrap();let reference=Reference::explicit(&path,ContentSlot::CheatCatalog,&rom()).unwrap();assert_eq!(reference.entry.kind,Kind::CheatCatalog);
    assert_eq!(reference.recheck(ContentSlot::CheatCatalog,&rom()).unwrap(),reference.path);assert!(reference.recheck(ContentSlot::Catalog,&rom()).is_err());
    fs::write(&path,br#"{"schema":2,"entries":[]}"#).unwrap();assert!(reference.recheck(ContentSlot::CheatCatalog,&rom()).is_err());
    assert!(Reference::explicit(&path,ContentSlot::CheatCatalog,&rom()).is_err());
}
#[test]
fn typed_references_recheck_identity_missing_kind_and_context_without_execution() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("runtime.exe");
    fs::write(&p, b"PUBLIC synthetic runtime; never execute").unwrap();
    let r = Reference::explicit(&p, ContentSlot::RuntimeA, &rom()).unwrap();
    assert_eq!(r.recheck(ContentSlot::RuntimeB, &rom()).unwrap(), r.path);
    assert!(r.recheck(ContentSlot::Library, &rom()).is_err());
    assert!(r.recheck(ContentSlot::RuntimeA, &"b".repeat(64)).is_err());
    fs::write(&p, b"CHANGED").unwrap();
    assert!(r.recheck(ContentSlot::RuntimeA, &rom()).is_err());
    fs::remove_file(&p).unwrap();
    assert!(r.recheck(ContentSlot::RuntimeA, &rom()).is_err());
    fs::write(
        t.path().join("NEVER.py"),
        b"raise RuntimeError('must not run')",
    )
    .unwrap();
    assert!(
        Reference::explicit(&t.path().join("NEVER.py"), ContentSlot::RuntimeA, &rom()).is_err()
    );
}
#[test]
fn labels_sequences_and_snapshots_have_real_readonly_preflight() {
    let t = tempfile::tempdir().unwrap();
    let labels = t.path().join("debug-labels.toml");
    fs::write(
        &labels,
        format!("gba-labels v1\nrom-sha256 {}\nrom 08000000 a\n", rom()),
    )
    .unwrap();
    assert!(Reference::explicit(&labels, ContentSlot::Labels, &rom()).is_ok());
    assert!(Reference::explicit(&labels, ContentSlot::Labels, &"b".repeat(64)).is_err());
    let input = t.path().join("input.input");
    fs::write(&input, b"gba-input v1\n0 3ff\n1 3fe\n").unwrap();
    assert!(Reference::explicit(&input, ContentSlot::Input, &rom()).is_ok());
    fs::write(&input, b"gba-input v1\n0 1000\n").unwrap();
    assert!(Reference::explicit(&input, ContentSlot::Input, &rom()).is_err());
    let m = crate::make_machine(vec![0; 256], None);
    let before = m.save_state().unwrap();
    let state = t.path().join("a.ffstate");
    fs::write(&state, &before).unwrap();
    assert!(Reference::explicit(
        &state,
        ContentSlot::Snapshot,
        &gba_core::ffta_rom::digest(&m.bus.rom)
    )
    .is_ok());
    assert!(Reference::explicit(&state, ContentSlot::Snapshot, &rom()).is_err());
    assert_eq!(m.save_state().unwrap(), before);
}
#[test]
fn malformed_profiles_catalogs_sessions_recordings_and_plans_are_refused() {
    let t = tempfile::tempdir().unwrap();
    for (slot, name, data) in [
        (
            ContentSlot::Replay,
            "replay.json",
            r#"{"schema":1,"kind":"ffta-debug-replay-profile"}"#,
        ),
        (
            ContentSlot::Catalog,
            "catalog.json",
            r#"{"schema":1,"cases":false}"#,
        ),
        (
            ContentSlot::Session,
            "debug-session.json",
            r#"{"schema":1,"kind":"ffta-debug-session"}"#,
        ),
        (
            ContentSlot::Recording,
            "recording.json",
            r#"{"schema":1,"kind":"ffta-input-recording"}"#,
        ),
        (ContentSlot::KnownPlan, "known-plan.toml", "schema = 1\n"),
        (ContentSlot::Dataset, "manifest.toml", "schema = 1\n"),
    ] {
        let p = t.path().join(name);
        fs::write(&p, data).unwrap();
        assert!(
            Reference::explicit(&p, slot, &rom()).is_err(),
            "{}",
            slot.label()
        );
    }
    assert!(!t.path().join("replay.lock").exists());
    assert!(!t.path().join("session.lock").exists());
}

#[test]
#[ignore = "explicit new public synthetic GUI case for migrated choices; no personal data or guest steps"]
fn write_synthetic_migration_fixture() {
    crate::content_picker::tests::write_synthetic_menu_fixture();
    finish_synthetic_migration_fixture();
}

#[test]
#[ignore = "resume only the explicit public partial fixture; never recreates its seed"]
fn finish_synthetic_migration_fixture() {
    let root = PathBuf::from(std::env::var_os("FFTA_SELECTOR_TEST_ROOT").unwrap());
    let rom = fs::read(root.join("ffta.gba")).unwrap();
    let m = crate::make_machine(rom.clone(), None);
    let identity: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("fixture-identity.json")).unwrap()).unwrap();
    assert_eq!(identity["kind"], "public-synthetic-selector-fixture");
    let _unstarted = crate::debug_session::replay_command(&root).unwrap();
    fs::create_dir_all(root.join("Data/config/recordings")).unwrap();
    let recording = root.join("Data/config/recordings/zero");
    let c = crate::debug_recording::Capture::start(
        &recording,
        &m,
        true,
        None,
        &input_config::AvConfig::default(),
        &input_config::InputConfig::default(),
    )
    .unwrap();
    c.finish(&m, "Explicit zero-frame public fixture", true)
        .unwrap();
    let runtime = PathBuf::from(std::env::var_os("FFTA_SELECTOR_TEST_RUNTIME").unwrap());
    let profile = root.join("Data/config/replay/Valid zero");
    crate::debug_replay::create(
        &recording,
        &root,
        &profile,
        "Public zero",
        120000,
        &runtime,
        None,
    )
    .unwrap();
    fs::write(
        root.join("debug-catalog.json"),
        r#"{"schema":1,"cases":[],"variants":[],"inputs":[]}"#,
    )
    .unwrap();
    fs::write(
        root.join("Data/config/debug-labels.toml"),
        format!(
            "gba-labels v1\nrom-sha256 {}\nrom 08000000 a\n",
            gba_core::ffta_rom::digest(&rom)
        ),
    )
    .unwrap();
    // One controlled public self-loop frame supplies the positive typed replay variants.
    let mut one = crate::make_machine(rom.clone(), None);
    let single = root.join("Data/config/recordings/one");
    let mut c = crate::debug_recording::Capture::start(
        &single,
        &one,
        true,
        None,
        &input_config::AvConfig::default(),
        &input_config::InputConfig::default(),
    )
    .unwrap();
    c.before_frame(&one, &[]).unwrap();
    one.run_frame(5_000_000);
    assert_eq!(one.bus.frames, 1);
    c.finish(&one, "Explicit one-frame public fixture", true)
        .unwrap();
    let args = |name: &str| {
        vec![
            "--recording".into(),
            single.to_string_lossy().into_owned(),
            "--session".into(),
            root.to_string_lossy().into_owned(),
            "--dir".into(),
            root.join("Data/config/replay")
                .join(name)
                .to_string_lossy()
                .into_owned(),
            "--case".into(),
            name.into(),
            "--runtime".into(),
            runtime.to_string_lossy().into_owned(),
            "--timeout-ms".into(),
            "120000".into(),
        ]
    };
    let audit = crate::debug_rom_audit::Request {
        settings: crate::debug_rom_audit::Settings {
            preset: "bus".into(),
            from: 0,
            to: 1,
            ranges: Vec::new(),
            limit: 512,
            known_sha256: None,
        },
        known: None,
        dataset: None,
    };
    crate::debug_replay::create_cmd_audited(&args("Public audit"), Some(&audit)).unwrap();
    let capture = crate::debug_capture::Request {
        settings: crate::debug_capture::Settings {
            condition: crate::debug_capture::Condition::Frame { completed: 1 },
            limit_frames: 1,
        },
        manifest: None,
        dataset: None,
    };
    crate::debug_replay::create_cmd_captured(&args("Public capture"), &capture).unwrap();
    let comparison = crate::debug_compare::Request {
        settings: crate::debug_compare::Settings {
            frames: 1,
            backend: "interp".into(),
            audio: "preserve".into(),
        },
        dataset: None,
    };
    crate::debug_replay::create_cmd_comparison(&args("Public compare"), &comparison).unwrap();
}
