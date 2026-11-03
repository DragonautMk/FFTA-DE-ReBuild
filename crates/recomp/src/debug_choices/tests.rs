use super::*;
#[test]
fn internal_locations_move_but_explicit_external_paths_do_not_and_resolving_never_reads() {
    let t = tempfile::tempdir().unwrap();
    let app = t.path().join("Original è app");
    let config = app.join("Data/config");
    std::fs::create_dir_all(&config).unwrap();
    let external = t.path().join("Outside è.py");
    std::fs::write(&external, b"raise RuntimeError('NEVER START')").unwrap();
    let file = config.join("recipe.json");
    std::fs::write(&file, b"PUBLIC").unwrap();
    let anchors = Anchors {
        app: app.clone(),
        config,
    };
    assert_eq!(
        Location::capture(&anchors.app, &anchors).unwrap(),
        Location::App(".".into())
    );
    let internal = Location::capture(&file, &anchors).unwrap();
    let outside = Location::capture(&external, &anchors).unwrap();
    assert!(matches!(internal, Location::App(_)));
    assert!(matches!(outside, Location::External(_)));
    let moved = t.path().join("Moved folder è");
    std::fs::rename(app, &moved).unwrap();
    let rebased = Anchors {
        app: moved.clone(),
        config: moved.join("Data/config"),
    };
    assert_eq!(
        internal.resolve(&rebased).unwrap(),
        moved.join("Data/config/recipe.json")
    );
    assert_eq!(
        outside.resolve(&rebased).unwrap(),
        external.canonicalize().unwrap()
    );
    let missing = Location::App("Data/config/missing.py".into());
    let path = missing.resolve(&rebased).unwrap();
    assert!(!path.exists());
    assert_eq!(
        std::fs::read_to_string(outside.resolve(&rebased).unwrap()).unwrap(),
        "raise RuntimeError('NEVER START')"
    );
}
#[test]
fn persisted_schema_rejects_traversal_unknown_slots_duplicates_caps_and_unknown_fields() {
    for bad in [
        "../file",
        "/absolute",
        "a/../b",
        "a\\b",
        "a:b",
        "a//b",
        "./a",
        "",
    ] {
        assert!(Location::App(bad.into()).validate().is_err());
    }
    let mut s = State::default();
    s.inputs.push(SavedInput {
        slot: "HostRuntime".into(),
        file: Location::App("python.exe".into()),
        sha256: "a".repeat(64),
        kind: Kind::Runtime,
    });
    let bytes = s.encode().unwrap();
    let got = State::decode(&bytes).unwrap();
    assert_eq!(got.inputs[0].slot, "HostRuntime");
    s.inputs.push(s.inputs[0].clone());
    assert!(s.encode().is_err());
    s.inputs.pop();
    s.inputs[0].slot = "UnknownFutureRole".into();
    assert!(s.encode().is_err());
    assert!(State::decode(&vec![b' '; MAX_BYTES + 1]).is_err());
    let mut v = serde_json::to_value(State::default()).unwrap();
    v["execute"] = serde_json::json!(true);
    assert!(State::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}

#[test]
fn pins_restore_after_move_preserve_neutral_tools_and_refuse_changed_missing_or_foreign_rom() {
    let t = tempfile::tempdir().unwrap();
    let app = t.path().join("Before è");
    std::fs::create_dir(&app).unwrap();
    let anchors = Anchors {
        app: app.clone(),
        config: app.join("Data/config"),
    };
    std::fs::create_dir_all(&anchors.config).unwrap();
    let runtime = app.join("runtime.exe");
    std::fs::write(&runtime, b"PUBLIC NOT EXECUTABLE").unwrap();
    let rom = "a".repeat(64);
    let reference =
        crate::debug_inputs::Reference::explicit(&runtime, menu::ContentSlot::HostRuntime, &rom)
            .unwrap();
    let pin = SavedInput::capture(menu::ContentSlot::HostRuntime, &reference, &anchors).unwrap();
    assert_eq!(pin.file, Location::App("runtime.exe".into()));
    assert!(pin.restore(&anchors, &"b".repeat(64)).is_ok());
    let labels = app.join("debug-labels.toml");
    std::fs::write(
        &labels,
        format!("gba-labels v1\nrom-sha256 {rom}\nrom 08000000 a\n"),
    )
    .unwrap();
    let reference =
        crate::debug_inputs::Reference::explicit(&labels, menu::ContentSlot::Labels, &rom).unwrap();
    let label_pin = SavedInput::capture(menu::ContentSlot::Labels, &reference, &anchors).unwrap();
    assert!(label_pin.restore(&anchors, &"b".repeat(64)).is_err());
    let mut file = FileState::load(&anchors).unwrap();
    assert!(!file.path.exists());
    file.state.inputs.push(pin);
    file.save().unwrap();
    let moved = t.path().join("After move è with spaces");
    std::fs::rename(&app, &moved).unwrap();
    let rebased = Anchors {
        app: moved.clone(),
        config: moved.join("Data/config"),
    };
    let loaded = FileState::load(&rebased).unwrap();
    assert!(loaded.state.inputs[0].restore(&rebased, &rom).is_ok());
    std::fs::write(moved.join("runtime.exe"), b"CHANGED PUBLIC").unwrap();
    assert!(loaded.state.inputs[0].restore(&rebased, &rom).is_err());
    std::fs::remove_file(moved.join("runtime.exe")).unwrap();
    assert!(loaded.state.inputs[0].restore(&rebased, &rom).is_err());
}
#[test]
fn atomic_preferences_refuse_detected_external_edits_and_do_not_rewrite_unchanged_bytes() {
    let t = tempfile::tempdir().unwrap();
    let a = Anchors {
        app: t.path().into(),
        config: t.path().join("config"),
    };
    let mut f = FileState::load(&a).unwrap();
    assert!(!a.config.exists());
    f.save().unwrap();
    let before = std::fs::metadata(&f.path).unwrap().modified().unwrap();
    let bytes = std::fs::read(&f.path).unwrap();
    f.save().unwrap();
    assert_eq!(
        std::fs::metadata(&f.path).unwrap().modified().unwrap(),
        before
    );
    assert_eq!(std::fs::read(&f.path).unwrap(), bytes);
    std::fs::write(&f.path, b"USER CHANGED PREFERENCES").unwrap();
    assert!(f.save().is_err());
    assert_eq!(std::fs::read(&f.path).unwrap(), b"USER CHANGED PREFERENCES");
    assert!(FileState::load(&a).is_err());
}

#[test]
fn selecting_before_first_config_write_does_not_require_a_config_directory() {
    let t = tempfile::tempdir().unwrap();
    let app = t.path().join("App è");
    std::fs::create_dir(&app).unwrap();
    let a = Anchors {
        app: app.clone(),
        config: app.join("Data/config"),
    };
    let selected = app.join("runtime.exe");
    std::fs::write(&selected, b"PUBLIC NEVER RUN").unwrap();
    assert_eq!(
        Location::capture(&selected, &a).unwrap(),
        Location::App("runtime.exe".into())
    );
    let external = t.path().join("External.py");
    std::fs::write(&external, b"NEVER RUN").unwrap();
    assert!(matches!(
        Location::capture(&external, &a).unwrap(),
        Location::External(_)
    ));
    assert!(!a.config.exists());
}

#[cfg(windows)]
#[test]
fn preferences_publish_through_an_actual_canonical_verbatim_windows_root() {
    let t = tempfile::tempdir().unwrap();
    let app = std::fs::canonicalize(t.path()).unwrap();
    assert!(app.to_string_lossy().starts_with("\\\\?\\"));
    let anchors = Anchors {
        config: app.join("Data/config"),
        app,
    };
    let mut file = FileState::load(&anchors).expect("a drive prefix is not a directory to stat");
    assert!(!file.path.exists());
    file.save().unwrap();
    let loaded = FileState::load(&anchors).unwrap();
    assert_eq!(loaded.state.kind, "ffta-debug-choices");
    assert_eq!(loaded.state.encode().unwrap(), file.state.encode().unwrap());
}
