use super::*;
#[test]
fn recipe_rejects_guest_mutations_unknown_fields_and_unbounded_steps_without_running() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("recipe.json");
    let good = json!({"schema":1,"kind":"ffta-debug-recipe","label":"PUBLIC readonly","steps":[{"operation_id":"control.status"},{"operation_id":"control.inspect","address":"08000000","length":16}]});
    fs::write(&p, serde_json::to_vec(&good).unwrap()).unwrap();
    let r = recipe(&p).unwrap();
    let argv = r.steps[1]
        .args(Path::new("session è ; $x & space"))
        .unwrap();
    assert_eq!(argv[2], "session è ; $x & space");
    assert_eq!(argv[3], "08000000");
    for bad in [
        json!({"operation_id":"control.resume"}),
        json!({"operation_id":"control.inspect","address":"ffffffff","length":256}),
        json!({"operation_id":"control.status","shell":"echo NO"}),
    ] {
        let mut v = good.clone();
        v["steps"] = json!([bad]);
        fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(recipe(&p).is_err());
    }
    let mut v = good;
    v["steps"] = json!(vec![json!({"operation_id":"control.status"}); 17]);
    fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(recipe(&p).is_err());
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 1);
}
#[test]
fn guest_metadata_and_declared_details_are_readonly_and_distinct() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("script-archive.json");
    fs::write(&p,br#"{"schema":1,"kind":"PUBLIC synthetic guest metadata","entries":[{"opcode":"never run"}]}"#).unwrap();
    let s = guest(&p).unwrap();
    assert!(s.contains("1 declared entries"));
    assert!(s.contains("No guest instructions or host code executed"));
    for tool in [
        Tool::Homepage,
        Tool::CampaignReport,
        Tool::ArtifactPreview,
        Tool::Recipe,
        Tool::GuestScripts,
    ] {
        let d = description(tool);
        assert!(d.contains("v1"));
    }
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 1);
}
fn fixture_command(mode: &str) -> Command {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.args([
        "--exact",
        "debug_tools::tests::public_process_fixture",
        "--ignored",
        "--nocapture",
    ]);
    c.env("FFTA_TOOL_TEST_SCENARIO", mode);
    c
}
#[test]
#[ignore = "explicit public subprocess double; only called by bounded runner tests"]
fn public_process_fixture() {
    let Ok(mode) = std::env::var("FFTA_TOOL_TEST_SCENARIO") else {
        return;
    };
    match mode.as_str() {
        "args" => {
            println!("ARG {:?}", std::env::var("FFTA_TOOL_TEST_ARG").unwrap());
        }
        "flood" => {
            let b = vec![b'x'; 65536];
            loop {
                std::io::stdout().write_all(&b).unwrap();
            }
        }
        "tree" => {
            let mut c = fixture_command("sleep");
            // Inherit pipes: an escaped descendant would keep EOF blocked.
            let mut child = c.spawn().unwrap();
            println!("DESCENDANT {}", child.id());
            std::io::stdout().flush().unwrap();
            let _ = child.wait();
        }
        _ => {
            println!("PUBLIC SLEEP");
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_secs(30));
        }
    }
}
#[test]
fn bounded_process_uses_literal_argv_cancels_times_out_and_caps_logs() {
    let t = tempfile::tempdir().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut c = fixture_command("args");
    c.env("FFTA_TOOL_TEST_ARG", "spazi è ; $() & \"quote\"");
    let r = process(
        &mut c,
        t.path(),
        "literal",
        &cancel,
        Instant::now() + Duration::from_secs(10),
    )
    .unwrap();
    assert_eq!(r.outcome, "completed");
    assert!(fs::read_to_string(t.path().join("literal.stdout"))
        .unwrap()
        .contains("spazi è ; $() &"));
    let r = process(
        &mut fixture_command("sleep"),
        t.path(),
        "timeout",
        &cancel,
        Instant::now() + Duration::from_millis(350),
    )
    .unwrap();
    assert_eq!(r.outcome, "timeout");
    let c = cancel.clone();
    let log = t.path().join("cancel.stdout");
    let trigger = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline
            && !fs::read_to_string(&log)
                .unwrap_or_default()
                .contains("DESCENDANT")
        {
            thread::sleep(Duration::from_millis(20));
        }
        c.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    let r = process(
        &mut fixture_command("tree"),
        t.path(),
        "cancel",
        &cancel,
        Instant::now() + Duration::from_secs(20),
    )
    .unwrap();
    trigger.join().unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "descendants must stop before pipe collection"
    );
    assert_eq!(r.outcome, "cancelled");
    assert!(fs::read_to_string(t.path().join("cancel.stdout"))
        .unwrap()
        .contains("DESCENDANT"));
    cancel.store(false, Ordering::SeqCst);
    let r = process(
        &mut fixture_command("flood"),
        t.path(),
        "cap",
        &cancel,
        Instant::now() + Duration::from_secs(10),
    )
    .unwrap();
    assert_eq!(r.outcome, "log-limit");
    assert_eq!(r.stdout, LOG_LIMIT);
    let r = process(
        &mut Command::new(t.path().join("absent-runtime.exe")),
        t.path(),
        "absent",
        &cancel,
        Instant::now() + Duration::from_secs(2),
    );
    assert!(r.unwrap_err().contains("Cannot start"));
}
#[test]
fn bounded_run_retention_and_runtime_version_refusals_preserve_evidence() {
    let t = tempfile::tempdir().unwrap();
    for _ in 0..8 {
        new_run(t.path()).unwrap();
    }
    assert!(new_run(t.path()).is_err());
    fs::write(t.path().join("version.stdout"), "Python 3.10.0\n").unwrap();
    fs::write(t.path().join("version.stderr"), b"").unwrap();
    assert!(verify_version(t.path()).is_err());
    fs::write(t.path().join("version.stdout"), "Python 3.11.9\n").unwrap();
    assert_eq!(verify_version(t.path()).unwrap(), "Python 3.11.9");
}

#[test]
fn extractor_projection_without_envelope_is_selectable_but_not_executed() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("script-commands.json");
    let commands = (0..gba_core::rom_data::script_commands::COMMAND_COUNT)
        .map(|i| json!({"opcode":i,"advance":2,"callback":{"rom_offset":0}}))
        .collect::<Vec<_>>();
    fs::write(&path,serde_json::to_vec(&json!({"source_offset":gba_core::rom_data::script_commands::SOURCE_OFFSET,"commands":commands})).unwrap()).unwrap();
    let r = crate::debug_inputs::Reference::explicit(&path, Slot::GuestScripts, &"a".repeat(64))
        .unwrap();
    assert_eq!(
        r.recheck(Slot::GuestScripts, &"a".repeat(64)).unwrap(),
        r.path
    );
    assert!(guest(&path).unwrap().contains("115 declared entries"));
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 1);
}

#[test]
fn external_source_and_dependency_changes_refuse_and_embedded_choice_can_recover() {
    let t = tempfile::tempdir().unwrap();
    for item in HOME {
        fs::write(t.path().join(item.name), item.data).unwrap();
    }
    let mut d = Draft {
        tool: Tool::Homepage,
        values: std::collections::BTreeMap::from([
            ("--source".into(), "external".into()),
            ("--timeout-ms".into(), "1000".into()),
        ]),
        refs: std::collections::BTreeMap::new(),
        rom: "a".repeat(64),
        config: t.path().into(),
    };
    let p = t.path().join("report_home.py");
    let reference = crate::debug_inputs::Reference::explicit(&p, Slot::HostTool, &d.rom).unwrap();
    d.refs.insert(Slot::HostTool, reference);
    assert!(check_payload(&d).unwrap().contains("Verified external"));
    fs::write(t.path().join("report_home.html"), b"CHANGED PUBLIC").unwrap();
    assert!(check_payload(&d).unwrap_err().contains("dependency"));
    fs::write(t.path().join("report_home.html"), HOME[1].data).unwrap();
    fs::write(&p, b"raise RuntimeError('NEVER EXECUTE')").unwrap();
    assert!(check_payload(&d).is_err());
    d.values.insert("--source".into(), "embedded".into());
    assert!(check_payload(&d).unwrap().contains("Embedded"));
    assert!(details(&d).unwrap_err().contains("runtime absent"));
    assert!(!t.path().join("debug-tool-runs").exists());
}

#[test]
#[ignore = "explicit changed-scope UTF8 host adapter acceptance; closed PUBLIC session only, no guest frames"]
fn public_python_adapter_preserves_unicode_report_encoding() {
    let source = PathBuf::from(
        std::env::var_os("FFTA_HOST_TOOL_ENCODING_SESSION")
            .expect("explicit closed PUBLIC session"),
    );
    let runtime = PathBuf::from(
        std::env::var_os("FFTA_HOST_TOOL_ENCODING_RUNTIME")
            .expect("explicit selected Python runtime"),
    );
    let config = PathBuf::from(
        std::env::var_os("FFTA_HOST_TOOL_ENCODING_OUTPUT").expect("explicit NEW proof output"),
    );
    assert!(!config.exists());
    let identity = read_json(&source.join("fixture-identity.json"), 16384).unwrap();
    assert_eq!(identity["kind"], "public-synthetic-selector-fixture");
    let rom = identity["rom_sha256"].as_str().unwrap().to_owned();
    let d = Draft {
        tool: Tool::ArtifactPreview,
        values: std::collections::BTreeMap::from([
            ("--source".into(), "embedded".into()),
            ("--timeout-ms".into(), "120000".into()),
        ]),
        refs: std::collections::BTreeMap::from([
            (
                Slot::Session,
                crate::debug_inputs::Reference::explicit(
                    &source.join("debug-session.json"),
                    Slot::Session,
                    &rom,
                )
                .unwrap(),
            ),
            (
                Slot::HostRuntime,
                crate::debug_inputs::Reference::explicit(&runtime, Slot::HostRuntime, &rom)
                    .unwrap(),
            ),
        ]),
        rom,
        config: config.clone(),
    };
    let result = run(d, Arc::new(AtomicBool::new(false))).unwrap();
    assert!(result.contains("Host tool completed"), "{result}");
    let root = fs::read_dir(config.join("debug-tool-runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let bytes = fs::read(root.join("report/index.html")).unwrap();
    let html = String::from_utf8(bytes)
        .expect("HTML declares UTF8; dynamic Unicode paths must retain that encoding");
    assert!(html.contains("è"));
    let plan = read_json(&root.join("report/plan.json"), 16 * 1024 * 1024).unwrap();
    assert_eq!(plan["scopes"][0]["status"], "completed");
    println!("PUBLIC UTF8 adapter proof: {}", root.display());
}
