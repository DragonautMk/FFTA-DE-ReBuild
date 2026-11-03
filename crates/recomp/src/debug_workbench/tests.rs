use super::*;
#[test]
fn every_form_has_bounded_unique_fields_and_shared_control_validation() {
    for tool in Tool::ALL {
        let f = fields(tool);
        assert!(f.len() <= 32);
        let keys = f
            .iter()
            .map(|f| &f.key)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(keys.len(), f.len());
        assert!(checked_values(tool, defaults(tool)).is_ok());
    }
    let t = tempfile::tempdir().unwrap();
    let mut d = Draft {
        tool: Tool::Step,
        values: defaults(Tool::Step),
        refs: BTreeMap::new(),
        rom: "a".repeat(64),
        config: t.path().into(),
    };
    d.values.insert("frames".into(), "10".into());
    d.values.insert("--keys".into(), "3fe".into());
    let action = d.control().unwrap();
    let cli = crate::debug_control::parse(&[
        "step".into(),
        "selected-session".into(),
        "10".into(),
        "--keys".into(),
        "3fe".into(),
    ])
    .unwrap()
    .1;
    assert_eq!(
        serde_json::to_value(action).unwrap(),
        serde_json::to_value(cli).unwrap()
    );
    assert!(!t.path().join("debug-presets").exists());
    d.values.insert("--keys".into(), "400".into());
    assert!(d.control().is_err());
    for tool in [
        Tool::Audio,
        Tool::Code,
        Tool::Gameplay,
        Tool::Graphics,
        Tool::Inspect,
    ] {
        d.tool = tool;
        d.values = defaults(tool);
        let result = d.control();
        assert!(result.is_ok(), "{tool:?}: {result:?}");
    }
    d.tool = Tool::Recent;
    d.values = defaults(Tool::Recent);
    for reads in ["off", "on"] {
        d.values.insert("--reads".into(), reads.into());
        match d.control() {
            Ok(Action::RecentStart { settings }) => {
                assert!(reads == "off" || cfg!(feature = "rom-data-read-audit"));
                assert_eq!(settings.reads, reads == "on");
            }
            Err(error) => {
                assert_eq!(reads, "on", "{error}");
                assert!(!cfg!(feature = "rom-data-read-audit"), "{error}");
                assert!(error.contains("lacks rom-data-read-audit"), "{error}");
            }
            Ok(other) => panic!("Unexpected Recent action: {other:?}"),
        }
    }
    d.tool = Tool::Audio;
    d.values = defaults(Tool::Audio);
    d.values.insert("--streams".into(), "mixed,mixed".into());
    assert!(d.control().is_err());
}
#[test]
fn presets_are_unique_bounded_tool_checked_and_do_not_restore_paths_or_execute() {
    let t = tempfile::tempdir().unwrap();
    let d = Draft {
        tool: Tool::Step,
        values: defaults(Tool::Step),
        refs: BTreeMap::new(),
        rom: "a".repeat(64),
        config: t.path().into(),
    };
    for _ in 0..8 {
        save_preset(&d).unwrap();
    }
    assert!(save_preset(&d).is_err());
    assert_eq!(load_preset(t.path(), Tool::Step).unwrap(), d.values);
    assert!(load_preset(t.path(), Tool::Audio).is_err());
    let base = preset_base(t.path(), Tool::Step);
    let mut paths = fs::read_dir(base)
        .unwrap()
        .map(|p| p.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    let last = paths.last().unwrap();
    let original = fs::read(last).unwrap();
    let mut p: Preset = serde_json::from_slice(&original).unwrap();
    p.values.insert("arbitrary-path".into(), "outside".into());
    fs::write(last, serde_json::to_vec(&p).unwrap()).unwrap();
    assert!(load_preset(t.path(), Tool::Step).is_err());
    assert!(fs::read_dir(t.path())
        .unwrap()
        .all(|e| e.unwrap().file_name() == "debug-presets"));
}
#[test]
fn missing_and_foreign_inputs_refuse_before_profile_or_preset_creation() {
    let t = tempfile::tempdir().unwrap();
    for tool in [
        Tool::Replay,
        Tool::Audit,
        Tool::Capture,
        Tool::Compare,
        Tool::Issue,
    ] {
        let d = Draft {
            tool,
            values: defaults(tool),
            refs: BTreeMap::new(),
            rom: "a".repeat(64),
            config: t.path().into(),
        };
        assert!(d.validate().is_err());
    }
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
}

#[test]
fn late_preset_read_after_back_and_reopen_cannot_replace_the_current_draft() {
    let t = tempfile::tempdir().unwrap();
    let mut menu = menu::Menu::new(true, true);
    menu.set_debug_enabled(true);
    let mut w = Workbench::default();
    w.open(&mut menu, Tool::Step);
    let (_, _, revision) = menu.tool_values();
    let mut replacement = defaults(Tool::Step);
    replacement.insert("frames".into(), "600".into());
    w.job = Some(Job {
        request:ToolRequest::LoadPreset,
        tool: Tool::Step,
        revision,
        view: w.view,
        handle: std::thread::spawn(move || Ok(Output::Loaded(replacement))),
    });
    w.cancel_view();
    w.open(&mut menu, Tool::Step);
    for _ in 0..200 {
        w.poll(&mut menu);
        if !w.busy() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let (_, values, _) = menu.tool_values();
    assert_eq!(values.into_iter().collect::<Values>()["frames"], "1");
    assert!(w.last.contains("old draft"));
    let observed=w.observation.as_ref().unwrap();
    assert_eq!(observed["request"],"LoadPreset");
    assert_eq!(observed["current_draft"],false);
    assert_eq!(observed["result"]["outcome"],"completed");
    assert_eq!(std::fs::read_dir(t.path()).unwrap().count(), 0);
}
