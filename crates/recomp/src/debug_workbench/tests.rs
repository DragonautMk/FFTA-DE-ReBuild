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

#[test]
fn typed_recipe_tool_requires_current_preview_and_cancels_between_actions() {
    use std::sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}};
    use std::time::{Duration,Instant};
    let t=tempfile::tempdir().unwrap();let root=t.path().join("session");fs::create_dir_all(root.join("Data/config")).unwrap();
    let exe=if cfg!(windows){"recomp.exe"}else{"recomp"};
    for (name,bytes) in [("portable.txt",b"portable".as_slice()),("ffta.gba",b"PUBLIC ROM".as_slice()),(exe,b"PUBLIC executable".as_slice())]{fs::write(root.join(name),bytes).unwrap();}
    let hash=|name:&str|gba_core::ffta_rom::digest(&fs::read(root.join(name)).unwrap());
    let rom=hash("ffta.gba");let pins=json!({"portable.txt":hash("portable.txt"),"ffta.gba":rom,(exe):hash(exe)});
    fs::write(root.join("debug-session.json"),serde_json::to_vec(&json!({"schema":1,"kind":"ffta-debug-session","label":"Synthetic","interp":true,"real_bios":false,"has_state":false,"has_dataset":false,"pins":pins})).unwrap()).unwrap();
    assert_eq!(crate::debug_command::execute(&root,"recipe save tool 'step 1' 'step 2'")["ok"],true);
    let path=root.join("Data/config/debug-command-recipes/recipe-tool.json");
    let recipe_ref=crate::debug_inputs::Reference::explicit(&path,Slot::Recipe,&rom).unwrap();
    let session_ref=crate::debug_inputs::Reference::explicit(&root.join("debug-session.json"),Slot::Session,&rom).unwrap();
    let draft=||Draft{tool:Tool::Recipe,values:defaults(Tool::Recipe),refs:BTreeMap::from([(Slot::Recipe,recipe_ref.clone()),(Slot::Session,session_ref.clone())]),rom:rom.clone(),config:root.join("Data/config")};
    let mut menu=menu::Menu::new(true,true);menu.set_debug_enabled(true);let mut workbench=Workbench::default();workbench.open(&mut menu,Tool::Recipe);
    let finish=|w:&mut Workbench,m:&mut menu::Menu|{let deadline=Instant::now()+Duration::from_secs(10);while w.busy() && Instant::now()<deadline {w.poll(m);std::thread::sleep(Duration::from_millis(5));}assert!(!w.busy());};
    workbench.request(&mut menu,draft(),ToolRequest::Run).unwrap();finish(&mut workbench,&mut menu);assert!(workbench.last.contains("Review this typed recipe"));assert!(!root.join("Data/config/debug-command-runs").exists());
    workbench.request(&mut menu,draft(),ToolRequest::Validate).unwrap();finish(&mut workbench,&mut menu);assert!(workbench.last.contains("Start requires"));assert!(workbench.recipe_preview.is_some());
    let mut server=crate::debug_control::Server::new(&root).unwrap();let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));
    workbench.request(&mut menu,draft(),ToolRequest::Run).unwrap();let cancel=workbench.cancel.as_ref().unwrap().clone();let end=stop.clone();let calls=count.clone();
    let worker=std::thread::spawn(move||{while !end.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){let n=calls.fetch_add(1,Ordering::SeqCst)+1;if n==1{cancel.store(true,Ordering::SeqCst);}let _=server.reply(&request,json!({"paused":true,"frame":n,"remaining":0}),None);}std::thread::sleep(Duration::from_millis(2));}});
    finish(&mut workbench,&mut menu);assert!(matches!(workbench.observation.as_ref().unwrap()["outcome"].as_str(),Some("cancelled"|"cancellation-pending")));assert_eq!(count.load(Ordering::SeqCst),1);
    let id=workbench.observation.as_ref().unwrap()["result"]["checkpoint_id"].as_str().unwrap();
    assert_eq!(crate::debug_command::execute(&root,&format!("recipe resume {id}"))["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),2);
    workbench.cancel_view();workbench.request(&mut menu,draft(),ToolRequest::Run).unwrap();finish(&mut workbench,&mut menu);assert!(workbench.last.contains("Review this typed recipe"));assert_eq!(count.load(Ordering::SeqCst),2);
    stop.store(true,Ordering::SeqCst);worker.join().unwrap();
}
