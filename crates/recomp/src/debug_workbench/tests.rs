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
        let Action::RecentStart { settings } = d.control().unwrap() else { panic!("Expected recent settings") };
        assert_eq!(settings.reads, reads == "on");
        if reads == "on" && !cfg!(feature = "rom-data-read-audit") {
            assert!(d.validate().unwrap().contains("feature-unavailable"));
        }
    }
    d.tool = Tool::Audio;
    d.values = defaults(Tool::Audio);
    d.values.insert("--streams".into(), "mixed,mixed".into());
    assert!(d.control().is_err());
}
#[test]
fn common_menu_drafts_match_console_and_local_control_actions() {
    let root=tempfile::tempdir().unwrap();
    let cases=[
        (Tool::Step,"step 1 --keys 0x3ff"),
        (Tool::Inspect,"inspect 0x08000000 16"),
        (Tool::Audio,"audio-start --frames 60 --streams mixed,psg,hle,fifo0,fifo1"),
        (Tool::Code,"code-start --frames 1 --events 512 --start 0x0 --end 0xffffffff --kind all --mode all"),
        (Tool::Gameplay,"gameplay-start --frames 1 --events 512 --family all"),
        (Tool::Recent,"recent-start --events 1024 --reads on --start 0x0 --end 0x02000000 --on-error on"),
        (Tool::Graphics,"graphics"),
    ];
    for (tool,text) in cases {
        let draft=Draft{tool,values:defaults(tool),refs:BTreeMap::new(),rom:"a".repeat(64),config:root.path().into()};
        let menu=draft.control().unwrap();
        let crate::debug_command::Command::Control(console)=crate::debug_command::parse(text).unwrap() else{panic!("{tool:?}")};
        let mut args=crate::debug_command::tokenize(text).unwrap();args.insert(1,"session".into());
        let cli=crate::debug_control::parse(&args).unwrap().1;
        assert_eq!(serde_json::to_value(&menu).unwrap(),serde_json::to_value(&console).unwrap(),"{tool:?}");
        assert_eq!(serde_json::to_value(&menu).unwrap(),serde_json::to_value(&cli).unwrap(),"{tool:?}");
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(),0);
}
#[test]
fn nondefault_form_bounds_match_console_and_local_control() {
    let root=tempfile::tempdir().unwrap();
    let cases: &[(Tool,&[(&str,&str)],&str)] = &[
        (Tool::Step,&[("frames","600"),("--keys","000")],"step 600 --keys 0x000"),
        (Tool::Inspect,&[("address","02000000"),("length","256")],"inspect 0x02000000 256"),
        (Tool::Audio,&[("--frames","180"),("--streams","mixed,psg")],"audio-start --frames 180 --streams mixed,psg"),
        (Tool::Code,&[("--frames","60"),("--events","1"),("--start","08000000"),("--end","0a000000"),("--kind","interpreter"),("--mode","thumb")],"code-start --frames 60 --events 1 --start 0x08000000 --end 0x0a000000 --kind interpreter --mode thumb"),
        (Tool::Gameplay,&[("--frames","60"),("--events","1"),("--family","unit"),("--context","02000000")],"gameplay-start --frames 60 --events 1 --family unit --context 0x02000000"),
        (Tool::Recent,&[("--events","1"),("--reads","off"),("--start","10"),("--end","20"),("--on-error","off")],"recent-start --events 1 --reads off --start 0x10 --end 0x20 --on-error off"),
        (Tool::Graphics,&[("--compare-last","on")],"graphics --compare-last"),
    ];
    for &(tool,changes,text) in cases {
        let mut values=defaults(tool);
        for &(key,value) in changes {values.insert(key.into(),value.into());}
        let draft=Draft{tool,values:checked_values(tool,values).unwrap(),refs:BTreeMap::new(),rom:"a".repeat(64),config:root.path().into()};
        let menu=draft.control().unwrap();
        let crate::debug_command::Command::Control(console)=crate::debug_command::parse(text).unwrap() else {panic!("{tool:?}")};
        let mut args=crate::debug_command::tokenize(text).unwrap();args.insert(1,"session".into());
        let cli=crate::debug_control::parse(&args).unwrap().1;
        assert_eq!(serde_json::to_value(&menu).unwrap(),serde_json::to_value(&console).unwrap(),"{tool:?}");
        assert_eq!(serde_json::to_value(&menu).unwrap(),serde_json::to_value(&cli).unwrap(),"{tool:?}");
    }
    let invalid: &[(Tool,&str,&str,&str)] = &[
        (Tool::Step,"frames","601","step 601"),
        (Tool::Inspect,"length","257","inspect 0x08000000 257"),
        (Tool::Audio,"--frames","181","audio-start --frames 181"),
        (Tool::Code,"--events","0","code-start --events 0"),
        (Tool::Gameplay,"--frames","61","gameplay-start --frames 61"),
        (Tool::Recent,"--events","1025","recent-start --events 1025"),
    ];
    for &(tool,key,value,text) in invalid {
        let mut values=defaults(tool);values.insert(key.into(),value.into());
        let draft=Draft{tool,values:checked_values(tool,values).unwrap(),refs:BTreeMap::new(),rom:"a".repeat(64),config:root.path().into()};
        assert!(draft.control().is_err(),"{tool:?}");
        assert!(crate::debug_command::parse(text).is_err(),"{text}");
        let mut args=crate::debug_command::tokenize(text).unwrap();args.insert(1,"session".into());
        assert!(crate::debug_control::parse(&args).is_err(),"{text}");
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(),0);
}
#[test]
fn recent_form_console_and_cli_share_typed_validation() {
    let t=tempfile::tempdir().unwrap();
    let mut draft=Draft{tool:Tool::Recent,values:defaults(Tool::Recent),refs:BTreeMap::new(),rom:"a".repeat(64),config:t.path().into()};
    for reads in ["on","off"] {
        draft.values.insert("--reads".into(),reads.into());
        let menu=draft.control().unwrap();
        let args=vec!["recent-start","session","--events","1024","--reads",reads,"--start","0","--end","02000000","--on-error","on"].into_iter().map(str::to_owned).collect::<Vec<_>>();
        let cli=crate::debug_control::parse(&args).unwrap().1;
        let text=format!("recent-start --events 1024 --reads {reads} --start 0x0 --end 0x02000000 --on-error on");
        let crate::debug_command::Command::Control(console)=crate::debug_command::parse(&text).unwrap() else {panic!()};
        assert_eq!(serde_json::to_value(&menu).unwrap(),serde_json::to_value(&cli).unwrap());
        assert_eq!(serde_json::to_value(&menu).unwrap(),serde_json::to_value(&console).unwrap());
    }
    draft.values.insert("--events".into(),"0".into());
    assert!(draft.control().is_err());
    assert!(crate::debug_command::parse("recent-start --events 0").is_err());
    assert!(!t.path().join("debug-presets").exists());
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
