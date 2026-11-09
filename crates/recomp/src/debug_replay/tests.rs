use super::*;

#[test]
fn preparation_metadata_is_bounded_best_effort_and_preserves_legacy_reports_and_inputs(){
    let fixture=Fixture::new();let path=fixture.create();let(root,p)=load(&path).unwrap();let before=fixture.inputs();
    let report=Report::new(&p,"metadata-run","running",0,"Synthetic metadata");
    let old=serde_json::to_value(&report).unwrap();assert!(old.get("preparation").is_none());let decoded:Report=serde_json::from_value(old).unwrap();assert!(decoded.preparation.is_empty());
    fs::create_dir_all(outputs(&root)).unwrap();let mut events=Vec::new();let started=Instant::now();
    preparation(&p,&root,"metadata-run",&mut events,started,"dataset","requested",serde_json::json!({"synthetic":true}));
    preparation(&p,&root,"metadata-run",&mut events,started,"cache","loaded",serde_json::json!({"large":"è".repeat(10000)}));
    let published:Report=serde_json::from_slice(&read(&outputs(&root).join("progress.json"),32*1024).unwrap()).unwrap();assert_eq!(published.preparation.len(),2);assert_eq!(published.preparation[1]["details"]["truncated"],true);
    let absent=root.join("absent parent");
    for _ in 0..20{preparation(&p,&absent,"metadata-run",&mut events,started,"dataset","refused",serde_json::json!({"synthetic":true}));}
    assert_eq!(events.len(),8);assert!(events[2].get("publication_error").is_some());
    let mut final_report=report;final_report.preparation=events;let bytes=serde_json::to_vec(&final_report).unwrap();assert!(bytes.len()<32*1024);assert_eq!(fixture.inputs(),before);
    let decoded:Report=serde_json::from_slice(&bytes).unwrap();assert_eq!(decoded.preparation.len(),8);assert_eq!(decoded.preparation[0]["sequence"],0);
}

#[cfg(windows)]
#[test]
fn failed_worker_supervisor_retains_received_preparation_without_reexecuting_worker(){
    let fixture=Fixture::new();let profile=fixture.create();let(root,p)=load(&profile).unwrap();let before=fixture.inputs();fs::create_dir_all(outputs(&root)).unwrap();
    let run="metadata-run";let mut last=Report::new(&p,run,"preparing",0,"Synthetic worker failure");last.preparation=vec![serde_json::json!({"sequence":0,"family":"cache","outcome":"requested"})];
    json(&outputs(&root).join("progress.json"),&last).unwrap();
    let child=std::process::Command::new("cmd").args(["/c","exit 3"]).spawn().unwrap();
    let lease=File::create(root.join("synthetic-supervisor.lock")).unwrap();
    let mut job=Job{profile,root,p,run:run.into(),child,_lease:lease,started:Instant::now(),cancelled:None,last:Some(last),done:false};
    let mut result=None;for _ in 0..200{result=job.poll().unwrap();if result.is_some(){break;}std::thread::sleep(Duration::from_millis(5));}
    let result=result.expect("Synthetic child exited");assert_eq!(result.outcome,"incompatible");assert_eq!(result.preparation.len(),1);assert_eq!(result.preparation[0]["family"],"cache");assert_eq!(fixture.inputs(),before);
}

#[cfg(windows)]
#[test]
fn progress_publication_retries_a_short_windows_reader_sharing_denial() {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: usize,
            creation: u32,
            flags: u32,
            template: usize,
        ) -> usize;
        fn CloseHandle(handle: usize) -> i32;
    }
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    json(&path, &serde_json::json!({"completed":1})).unwrap();
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let handle = unsafe { CreateFileW(name.as_ptr(), 0x80000000, 1, 0, 3, 0, 0) };
    assert_ne!(handle, usize::MAX);
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(12));
        unsafe { CloseHandle(handle) };
    });
    json(&path, &serde_json::json!({"completed":2})).unwrap();
    release.join().unwrap();
    let result: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(result["completed"], 2);
}

fn machine() -> Machine {
    let mut rom = vec![0; 0x200];
    rom[..4].copy_from_slice(&0xeaff_fffeu32.to_le_bytes());
    rom[0x100..0x106].copy_from_slice(b"SRAM_V");
    let mut m = crate::make_machine(rom, None);
    m.bus.frames = 17;
    m.bus.keys = 0x3fe;
    m
}
fn drain(m: &mut Machine) {
    m.bus.audio_buf.clear();
    m.bus.psg_tap.clear();
    m.bus.hle_tap.clear();
    m.bus.fifo_tap[0].clear();
    m.bus.fifo_tap[1].clear();
}
struct Fixture {
    temp: tempfile::TempDir,
    recording: PathBuf,
    session: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let recording = temp.path().join("capture");
        let mut m = machine();
        let av = input_config::AvConfig::default();
        let input = input_config::InputConfig::default();
        let mut c = crate::debug_recording::Capture::start(&recording, &m, true, None, &av, &input)
            .unwrap();
        let initial = m.save_state().unwrap();
        for n in 0..4 {
            m.bus.keys = if n < 2 { 0x3fe } else { 0x3ff };
            c.before_frame(&m, &[]).unwrap();
            m.run_frame(5_000_000);
            drain(&mut m);
        }
        c.finish(&m, "fixture", true).unwrap();
        let session = crate::debug_session::create(
            &temp.path().join("source session"),
            crate::debug_session::Source {
                rom: &m.bus.rom,
                bios: None,
                save: m.bus.save_data(),
                state: Some(&initial),
                av,
                input,
                interp: true,
                dataset: None,
                executable: &std::env::current_exe().unwrap(),
                cheat_favorites: None,
            },
        )
        .unwrap();
        Self {
            temp,
            recording,
            session,
        }
    }
    fn create(&self) -> PathBuf {
        create(
            &self.recording,
            &self.session,
            &self.temp.path().join("replay case"),
            "DS-20-EUR",
            120000,
            &std::env::current_exe().unwrap(),
            None,
        )
        .unwrap()
    }
    fn inputs(&self) -> Vec<Vec<u8>> {
        [
            self.session.join("seed.ffstate"),
            self.session.join("Data/saves/ffta.sav"),
            self.recording.join("start.ffstate"),
            self.recording.join("input.input"),
            self.recording.join("end.ffstate"),
        ]
        .iter()
        .map(|p| fs::read(p).unwrap())
        .collect()
    }
}
fn run_fixture(path: &Path, control: impl FnMut(u64) -> Option<(&'static str, String)>) -> Report {
    let (root, p) = load(path).unwrap();
    let mut m = machine();
    m.load_state(&read(&root.join("seed.ffstate"), 64 * 1024 * 1024).unwrap())
        .unwrap();
    let bytes = read(&root.join("input.input"), MAX_INPUT).unwrap();
    let input = crate::debug_recording::script(&bytes, p.frames).unwrap();
    let end = read(&root.join("expected.ffstate"), 64 * 1024 * 1024).unwrap();
    execute(&p, "fixture-run", &mut m, &input, &end, None, None, None, None, control)
}
#[test]
fn bounded_worker_rechecks_profile_identity_and_frame_limit_before_start(){
    let f=Fixture::new();let path=f.create();let before=f.inputs();let (_,profile,hash)=load_pinned(&path).unwrap();assert_eq!(profile.frames,4);
    assert!(Job::start_bounded(&path,crate::debug_control::ReplayMode::Plain,3).err().unwrap().contains("frame"));
    assert!(worker(&path,"bounded-test",Some(WorkerLimit{max_frames:3,profile_sha256:hash.clone()})).err().unwrap().contains("frame limit"));
    assert!(worker(&path,"bounded-test",Some(WorkerLimit{max_frames:4,profile_sha256:"a".repeat(64)})).err().unwrap().contains("changed"));
    let mut changed:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();changed["runtime_sha256"]=serde_json::json!("a".repeat(64));fs::write(&path,serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(Job::start_bounded(&path,crate::debug_control::ReplayMode::Plain,4).err().unwrap().contains("current runtime"));
    assert!(!outputs(path.parent().unwrap()).exists());assert_eq!(f.inputs(),before);
}
#[test]
fn profile_pins_selected_inputs_and_repeats_full_endpoint_without_save_writes() {
    let f = Fixture::new();
    let before = f.inputs();
    let path = f.create();
    for _ in 0..2 {
        let r = run_fixture(&path, |_| None);
        assert_eq!(r.outcome, "passed");
        assert_eq!(r.completed, 4);
        assert_eq!(r.code(), 0);
    }
    assert_eq!(f.inputs(), before);
    let (root, p) = load(&path).unwrap();
    assert_eq!(p.stop, "exact-endpoint");
    assert!(p.library_sha256.is_none());
    assert_eq!(
        checked(&root, "input.input", MAX_INPUT, &p.input_sha256).unwrap(),
        fs::read(f.recording.join("input.input")).unwrap()
    );
}
#[test]
fn timeout_and_interruption_report_distinct_codes_and_exact_progress() {
    let f = Fixture::new();
    let path = f.create();
    for (kind, code) in [("timeout", 4), ("interrupted", 5)] {
        let r = run_fixture(&path, |done| {
            if done == 2 {
                Some((kind, "fixture stop".into()))
            } else {
                None
            }
        });
        assert_eq!(r.outcome, kind);
        assert_eq!(r.code(), code);
        assert_eq!(r.completed, 2);
    }
    assert_eq!(run_fixture(&path, |_| None).outcome, "passed");
}
#[test]
fn endpoint_divergence_never_becomes_a_success() {
    let f = Fixture::new();
    let path = f.create();
    let (root, p) = load(&path).unwrap();
    let mut m = machine();
    m.load_state(&fs::read(root.join("seed.ffstate")).unwrap())
        .unwrap();
    let input =
        crate::debug_recording::script(&fs::read(root.join("input.input")).unwrap(), p.frames)
            .unwrap();
    let r = execute(
        &p,
        "fixture",
        &mut m,
        &input,
        b"different expected endpoint",
        None,
        None,
        None,
        None,
        |_| None,
    );
    assert_eq!(r.code(), 2);
    assert_eq!(r.outcome, "diverged");
    assert!(r.state_sha256.is_some());
}
#[test]
fn create_refuses_active_sessions_existing_destinations_and_corrupt_capture() {
    let f = Fixture::new();
    let path = f.create();
    assert!(create(
        &f.recording,
        &f.session,
        path.parent().unwrap(),
        "DS20",
        1000,
        &std::env::current_exe().unwrap(),
        None
    )
    .is_err());
    let _lease = crate::debug_session::open(&f.session).unwrap();
    assert!(create(
        &f.recording,
        &f.session,
        &f.temp.path().join("active"),
        "DS20",
        1000,
        &std::env::current_exe().unwrap(),
        None
    )
    .is_err());
    drop(_lease);
    fs::write(f.recording.join("input.input"), "changed").unwrap();
    assert!(create(
        &f.recording,
        &f.session,
        &f.temp.path().join("corrupt"),
        "DS20",
        1000,
        &std::env::current_exe().unwrap(),
        None
    )
    .unwrap_err()
    .contains("checksum"));
}
#[test]
fn profile_bounds_schema_and_changed_input_are_detected() {
    let f = Fixture::new();
    let path = f.create();
    let (root, p) = load(&path).unwrap();
    fs::write(root.join("input.input"), b"changed").unwrap();
    assert!(checked(&root, "input.input", MAX_INPUT, &p.input_sha256)
        .unwrap_err()
        .contains("changed"));
    for (timeout, frames, stop) in [
        (0, 4, "exact-endpoint"),
        (120001, 4, "exact-endpoint"),
        (1000, MAX_FRAMES + 1, "exact-endpoint"),
        (1000, 4, "unknown"),
    ] {
        let mut invalid = p.clone();
        invalid.timeout_ms = timeout;
        invalid.frames = frames;
        invalid.stop = stop.into();
        json(&path, &invalid).unwrap();
        assert!(load(&path).is_err());
    }
}
#[test]
fn moved_profile_preserves_relative_payloads_and_selected_binary() {
    let f = Fixture::new();
    let path = f.create();
    let moved = f.temp.path().join("moved profile");
    fs::rename(path.parent().unwrap(), &moved).unwrap();
    assert_eq!(
        run_fixture(&moved.join(PROFILE), |_| None).outcome,
        "passed"
    );
    let (_, p) = load(&moved.join(PROFILE)).unwrap();
    assert_eq!(
        digest(&read(&moved.join(executable()), 256 * 1024 * 1024).unwrap()),
        p.runtime_sha256
    );
}
#[test]
fn wrong_session_rom_and_malformed_origin_are_rejected_before_destination_creation() {
    let f = Fixture::new();
    let p = f.recording.join("recording.json");
    let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    manifest["rom_sha256"] = "wrong".into();
    json(&p, &manifest).unwrap();
    let destination = f.temp.path().join("wrong");
    assert!(create(
        &f.recording,
        &f.session,
        &destination,
        "DS20",
        1000,
        &std::env::current_exe().unwrap(),
        None
    )
    .unwrap_err()
    .contains("ROM/BIOS/dataset"));
    assert!(!destination.exists());
}

#[cfg(feature="rom-data-read-audit")]
fn trigger_profile(f: &Fixture, name: &str, condition: crate::debug_capture::Condition, limit: u64) -> PathBuf {
    let args=vec!["--recording".into(),f.recording.to_string_lossy().into_owned(),"--session".into(),f.session.to_string_lossy().into_owned(),
        "--dir".into(),f.temp.path().join(name).to_string_lossy().into_owned(),"--case".into(),"DS-20-EUR".into()];
    create_cmd_captured(&args,&crate::debug_capture::Request{settings:crate::debug_capture::Settings{condition,limit_frames:limit},manifest:None,dataset:None}).unwrap()
}
#[cfg(feature="rom-data-read-audit")]
fn triggered(path:&Path) -> (Report, Machine, crate::debug_capture::Observer) {
    let(root,p)=load(path).unwrap();let mut m=machine();m.load_state(&fs::read(root.join("seed.ffstate")).unwrap()).unwrap();
    let input=crate::debug_recording::script(&fs::read(root.join("input.input")).unwrap(),p.frames).unwrap();
    let mut observer=crate::debug_capture::Observer::start(&root,p.capture.as_ref().unwrap(),&m.bus.rom).unwrap();
    let r=execute(&p,"fixture-trigger",&mut m,&input,&[],None,None,Some(&mut observer),None,|_|None);
    (r,m,observer)
}
#[cfg(feature="rom-data-read-audit")]
#[test]
fn frame_capture_is_exact_at_boundary_catalog_ready_reusable_and_never_overwrites() {
    let f=Fixture::new();let original=f.inputs();
    let path=trigger_profile(&f,"frame capture",crate::debug_capture::Condition::Frame{completed:2},3);
    assert!(Job::require_capture(&path).is_ok());
    let(r,m,observer)=triggered(&path);assert_eq!(r.outcome,"captured");assert_eq!(r.code(),0);assert_eq!(r.completed,2);assert_eq!(r.frames,3);
    let mut reference=machine();for _ in 0..2 {reference.run_frame(5_000_000);drain(&mut reference);}
    assert_eq!(m.save_state().unwrap(),reference.save_state().unwrap());assert_eq!(r.clock,Some(reference.bus.clock));
    let root=path.parent().unwrap();let seed=fs::read(root.join("seed.ffstate")).unwrap();let used_input=fs::read(root.join("input.input")).unwrap();
    let provenance=serde_json::json!({"relative_completed":2,"start_sha256":digest(&seed),"input_sha256":digest(&used_input)});
    // A later external edit must not change the bytes that actually produced the point.
    fs::write(root.join("input.input"),b"changed after execution").unwrap();
    let relative=crate::debug_capture::publish(root,"fixture-trigger","DS-20-EUR",&m,&observer,provenance.clone(),crate::debug_capture::Inputs{state:&seed,input:&used_input}).unwrap();
    assert_eq!(fs::read(root.join(&relative).parent().unwrap().join("input.input")).unwrap(),used_input);
    let catalog=crate::debug_catalog::Catalog::load(&root.join(&relative)).unwrap();
    assert_eq!(catalog.entries.len(),1);assert_eq!(catalog.entries[0].status,"variante");assert!(catalog.entries[0].available);
    let bytes=catalog.bytes(0).unwrap();let mut restored=machine();crate::save_state::load_bytes(&mut restored,&bytes).unwrap();
    assert_eq!(restored.save_state().unwrap(),reference.save_state().unwrap());
    restored.bus.keys=0x3ff;reference.bus.keys=0x3ff;
    restored.run_frame(5_000_000);drain(&mut restored);reference.run_frame(5_000_000);drain(&mut reference);
    assert_eq!(restored.save_state().unwrap(),reference.save_state().unwrap());
    assert!(crate::debug_capture::publish(root,"fixture-trigger","DS-20-EUR",&m,&observer,provenance.clone(),crate::debug_capture::Inputs{state:&seed,input:&used_input}).is_err());
    assert_eq!(catalog.bytes(0).unwrap(),bytes);assert_eq!(f.inputs(),original);
    assert!(crate::debug_capture::publish(root,"../escape","DS-20-EUR",&m,&observer,provenance.clone(),crate::debug_capture::Inputs{state:&seed,input:&used_input}).is_err());
}
#[cfg(feature="rom-data-read-audit")]
#[test]
fn real_instruction_entry_triggers_and_never_reached_is_bounded_without_promotion() {
    let f=Fixture::new();let original=f.inputs();
    let path=trigger_profile(&f,"PC capture",crate::debug_capture::Condition::Instruction{pc:0x08000000,thumb:false,opcode:0xeafffffe},4);
    let(r,m,observer)=triggered(&path);assert_eq!(r.outcome,"captured");assert_eq!(r.completed,1);assert!(observer.hit.is_some());
    let mut reference=machine();reference.run_frame(5_000_000);drain(&mut reference);assert_eq!(m.save_state().unwrap(),reference.save_state().unwrap());
    drop(observer);
    let never=trigger_profile(&f,"never PC",crate::debug_capture::Condition::Instruction{pc:0x08000010,thumb:false,opcode:0},2);
    let(r,m,observer)=triggered(&never);assert_eq!(r.outcome,"not-reached");assert_eq!(r.code(),7);assert_eq!(r.completed,2);assert!(observer.hit.is_none());
    reference.run_frame(5_000_000);drain(&mut reference);assert_eq!(m.save_state().unwrap(),reference.save_state().unwrap());
    assert!(r.capture_catalog.is_none());assert!(!never.parent().unwrap().join("captures").exists());assert_eq!(f.inputs(),original);
}
#[test]
fn capture_action_refuses_plain_profile_and_legacy_profile_has_no_capture_side_effects() {
    let f=Fixture::new();let path=f.create();assert!(Job::require_capture(&path).is_err());
    assert!(!fs::read_to_string(&path).unwrap().contains("capture"));
    assert!(!path.parent().unwrap().join("captures").exists());
}

#[test]
fn comparison_profile_is_explicit_bounded_and_separate_from_plain_replay(){
    let f=Fixture::new();let plain=f.create();assert!(Job::require_comparison(&plain).is_err());
    let args=vec!["--recording".into(),f.recording.to_string_lossy().into_owned(),"--session".into(),f.session.to_string_lossy().into_owned(),"--dir".into(),f.temp.path().join("comparison").to_string_lossy().into_owned(),"--case".into(),"DS-20-EUR".into()];
    let profile=create_cmd_comparison(&args,&crate::debug_compare::Request{settings:crate::debug_compare::Settings{frames:2,backend:"interp".into(),audio:"preserve".into()},dataset:None}).unwrap();
    let(_,p)=load(&profile).unwrap();assert_eq!(p.stop,"first-divergence");assert_eq!(p.comparison.as_ref().unwrap().frames,2);assert_eq!(Report::new(&p,"test","running",0,"running").frames,2);assert!(Job::require_comparison(&profile).is_ok());assert!(Job::require_capture(&profile).is_err());
    assert!(Job::start_bounded(&profile,crate::debug_control::ReplayMode::Plain,2).err().unwrap().contains("mode"));assert!(Job::start_bounded(&profile,crate::debug_control::ReplayMode::Comparison,1).err().unwrap().contains("frame"));
    assert_eq!(p.frames,4);assert_eq!(execution_frames(&p),2);assert!(check_bounded_profile(&p,crate::debug_control::ReplayMode::Comparison,2).is_ok());
}

#[test]
#[ignore = "Requires an explicitly protected EUR ROM and validation work directory"]
fn cheat_recording_replays_exact_state_and_refuses_changed_inputs() {
    use crate::debug_cheat_timeline::Entry;
    use serde_json::json;
    let rom=fs::read(std::env::var_os("FFTA_PARAMETER_TEST_ROM").expect("protected EUR ROM")).unwrap();
    let work=PathBuf::from(std::env::var_os("FFTA_VALIDATION_WORK").expect("validation work"));
    let temporary=tempfile::tempdir_in(work).unwrap();let root=temporary.path();
    let catalog_path=root.join("catalogo.json");
    let codes=json!(["AE1934BX 70EY","524D3BD0 7FEE","473AF42B 20FC","07A5CB01 381C"]);
    let catalog=json!({"schema":1,"sources":[{"id":"synthetic","sha256":"a".repeat(64)}],"entries":[
        {"id":"CH-0001","format":"codebreaker","region":"EUR","section":"synthetic","title":"Example","codes":codes,"notes":[],"aliases":[],"status":"not-runtime-tested","provenance":[{"source":"synthetic","master_id":"CH-0002","raw_code_lines":codes,"parameters":[
            {"name":"first","allowed":{"kind":"list","values":[15]},"slots":[{"line":0,"field":"word","offset":7,"digits":1}]},
            {"name":"second","allowed":{"kind":"range","min":6,"max":6,"step":1},"slots":[{"line":0,"field":"value","offset":3,"digits":1}]}]}]},
        {"id":"CH-0002","format":"codebreaker","region":"EUR","section":"synthetic","title":"Master","codes":[],"notes":[],"aliases":[],"status":"not-runtime-tested","provenance":[{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","19A10BC4 AA65","26960CD0 140B"]}]},
        {"id":"CH-0003","format":"codebreaker","region":"EUR","section":"synthetic","title":"Prior additive write","codes":["725D3BDA 7FFE"],"notes":[],"aliases":[],"status":"not-runtime-tested","provenance":[{"source":"synthetic","master_id":"CH-0002","raw_code_lines":["725D3BDA 7FFE"]}]}]});
    fs::write(&catalog_path,serde_json::to_vec(&catalog).unwrap()).unwrap();
    let sha=digest(&rom);let reference=crate::debug_inputs::Reference::explicit(&catalog_path,menu::ContentSlot::CheatCatalog,&sha).unwrap();
    let snapshot=crate::debug_cheats::Snapshot::new(crate::debug_cheats::Catalog::load(&catalog_path).unwrap(),reference.entry.id.clone(),sha.clone()).unwrap();
    let choices=vec![crate::debug_cheat_parameters::Choice{name:"first".into(),value:15},crate::debug_cheat_parameters::Choice{name:"second".into(),value:6}];
    let (plan,info)=snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&snapshot.generation,&choices).unwrap();
    let identity=json!({"id":"CH-0001","provenance_index":0,"master_id":"CH-0002","source_sha256":info["source_sha256"],"rom_sha256":sha,"entry_lines_sha256":info["entry_lines_sha256"],"master_lines_sha256":info["master_lines_sha256"],"plan_sha256":digest(&serde_json::to_vec(&plan).unwrap()),"decoder":"codebreaker-subset-v1","parameters":choices});
    let entry=Entry{identity,plan:serde_json::to_value(&plan).unwrap()};
    let once_identity=entry.identity.clone();
    let maintained_identity=json!([{"id":"CH-0001","identity":entry.identity,"plan_sha256":entry.identity["plan_sha256"],"decoder":"codebreaker-subset-v1"}]);
    let mut history=crate::cheat_provenance::History::default();
    let mut m=crate::make_machine(rom.clone(),None);let av=input_config::AvConfig::default();let input=input_config::InputConfig::default();
    let (prior,prior_info)=snapshot.prepare_once(&reference,&rom,"CH-0003",0,"CH-0002",&snapshot.generation,&[]).unwrap();
    let prior_json=serde_json::to_value(&prior).unwrap();assert_eq!(prior_json["operations"].as_array().unwrap().len(),1);assert_eq!(prior_json["operations"][0]["kind"],"modify");assert_eq!(prior_json["operations"][0]["operation"],"add");
    let prior_identity=json!({"id":"CH-0003","provenance_index":0,"master_id":"CH-0002","source_sha256":prior_info["source_sha256"],"rom_sha256":prior_info["rom_sha256"],"entry_lines_sha256":prior_info["entry_lines_sha256"],"master_lines_sha256":prior_info["master_lines_sha256"],"plan_sha256":digest(&serde_json::to_vec(&prior).unwrap()),"decoder":"codebreaker-subset-v1"});
    let prior_writes=prior.apply_once(&mut m.bus).unwrap();history.record("once",prior_identity,m.bus.frames,m.bus.clock,prior_writes);
    let initial=m.save_state().unwrap();let recording=root.join("recording");
    let mut capture=crate::debug_recording::Capture::start_with_history(&recording,&m,true,None,&av,&input,&history).unwrap();
    capture.reserve_cheats(&m,&catalog_path,"once",vec![entry.clone()]).unwrap();let writes=plan.apply_once(&mut m.bus).unwrap();capture.commit_cheats(writes).unwrap();history.record("once",once_identity.clone(),m.bus.frames,m.bus.clock,writes);
    for n in 0..2 {
        m.bus.keys=if n==0{0x3fe}else{0x3ff};capture.before_frame(&m,&[]).unwrap();
        capture.reserve_cheats(&m,&catalog_path,"maintained",vec![entry.clone()]).unwrap();let writes=plan.apply_once(&mut m.bus).unwrap();capture.commit_cheats(writes).unwrap();history.record("maintained",maintained_identity.clone(),m.bus.frames,m.bus.clock,writes);
        let before=m.bus.frames;m.run_frame(5_000_000);assert_eq!(m.bus.frames,before+1);drain(&mut m);
        if n==0 {
            let saved=crate::save_state::encode(&m,&vec![[0,0,0,255];240*160],&history).unwrap();assert!(saved.starts_with(b"FFTA-SL3"));
            let mut restored=crate::make_machine(rom.clone(),None);let ancestry=crate::save_state::load_bytes_with_history(&mut restored,&saved).unwrap();
            let exact=serde_json::to_value(&ancestry.exact).unwrap();assert_eq!(exact["applications"].as_array().unwrap().len(),3);
            assert_eq!(exact["variants"][1]["parameters"][0]["name"],"first");assert_eq!(restored.save_state().unwrap(),m.save_state().unwrap());
        }
    }
    capture.reserve_cheats(&m,&catalog_path,"once",vec![entry]).unwrap();let writes=plan.apply_once(&mut m.bus).unwrap();capture.commit_cheats(writes).unwrap();history.record("once",once_identity,m.bus.frames,m.bus.clock,writes);
    assert_eq!(&m.bus.ewram[0x3ffe0..0x3ffe2],&[1,0]);
    let end=m.save_state().unwrap();let end_clock=m.bus.clock;capture.finish(&m,"synthetic test",true).unwrap();
    let mut verify_machine=crate::make_machine(rom.clone(),None);let checked=crate::debug_recording::verify(&recording,&mut verify_machine).unwrap();assert_eq!(checked["ok"],true,"{checked}");assert_eq!(checked["clock"],end_clock);assert_eq!(verify_machine.save_state().unwrap(),end);assert_eq!(checked["cheat_timeline"]["applications"],4);assert_eq!(checked["starting_cheat_ancestry"]["summary"]["observed_variants"][0],"CH-0003");
    let session=crate::debug_session::create(&root.join("source"),crate::debug_session::Source{rom:&rom,bios:None,save:m.bus.save_data(),state:Some(&initial),av,input,interp:true,dataset:None,executable:&std::env::current_exe().unwrap(),cheat_favorites:None}).unwrap();
    let profile=create(&recording,&session,&root.join("profile"),"synthetic-cheat",120000,&std::env::current_exe().unwrap(),None).unwrap();
    let profile_root=profile.parent().unwrap();fs::create_dir_all(outputs(profile_root)).unwrap();let result=worker(&profile,"cheat-test",None).unwrap();
    assert_eq!(result.outcome,"passed","{}",result.message);assert_eq!(result.clock,Some(end_clock));assert_eq!(result.cheat_timeline.unwrap()["variants"][0]["id"],"CH-0001");assert_eq!(result.starting_cheat_ancestry.unwrap()["summary"]["observed_variants"][0],"CH-0003");
    let issue=crate::debug_issue::create(&profile,"Synthetic cheat replay").unwrap();assert!(issue.compatible);
    let issue_json:serde_json::Value=serde_json::from_slice(&fs::read(issue.path.parent().unwrap().join("issue.json")).unwrap()).unwrap();
    assert_eq!(issue_json["summary"]["cheat_timeline"]["variants"][0]["id"],"CH-0001");
    assert_eq!(issue_json["summary"]["starting_cheat_ancestry"]["summary"]["observed_variants"][0],"CH-0003");
    assert_eq!(issue_json["retained_result"]["outcome"],"passed");
    assert!(issue_json["attachments"].as_array().unwrap().iter().any(|item|item["path"]=="cheats-timeline.json"));
    let timeline_path=profile_root.join("cheats-timeline.json");let original=fs::read(&timeline_path).unwrap();let mut changed=original.clone();changed.push(b' ');fs::write(&timeline_path,&changed).unwrap();
    assert!(worker(&profile,"changed-timeline",None).err().unwrap().contains("changed"));fs::write(&timeline_path,&original).unwrap();
    // Even replacing both the plan hash and the profile's file hash must not
    // turn a different decoded plan into the selected source-bound variant.
    let mut forged=crate::debug_cheat_timeline::Timeline::parse(&original).unwrap();
    let old=forged.entries[0].plan["operations"][0]["value"].as_u64().unwrap();forged.entries[0].plan["operations"][0]["value"]=json!(old^1);
    let forged_plan=crate::debug_cheat_decoder::Plan::from_recorded(&forged.entries[0].plan).unwrap();
    forged.entries[0].identity["plan_sha256"]=json!(digest(&serde_json::to_vec(&forged_plan).unwrap()));
    let forged_bytes=serde_json::to_vec(&forged).unwrap();fs::write(&timeline_path,&forged_bytes).unwrap();
    let original_profile=fs::read(&profile).unwrap();let mut forged_profile:serde_json::Value=serde_json::from_slice(&original_profile).unwrap();
    forged_profile["cheat_timeline_sha256"]=json!(digest(&forged_bytes));fs::write(&profile,serde_json::to_vec(&forged_profile).unwrap()).unwrap();
    assert!(worker(&profile,"changed-plan",None).err().unwrap().contains("differs from source"));
    let incompatible_issue=crate::debug_issue::create(&profile,"Changed variant").unwrap();assert!(!incompatible_issue.compatible);
    let issue_json:serde_json::Value=serde_json::from_slice(&fs::read(incompatible_issue.path.parent().unwrap().join("issue.json")).unwrap()).unwrap();assert!(issue_json["retained_result"].is_null());
    fs::write(&profile,original_profile).unwrap();fs::write(&timeline_path,original).unwrap();
    let copied_catalog=profile_root.join("catalogo.json");let original=fs::read(&copied_catalog).unwrap();fs::write(&copied_catalog,b"{}").unwrap();assert!(worker(&profile,"changed-catalog",None).is_err());fs::write(&copied_catalog,original).unwrap();
    let initial_history=profile_root.join("cheat-ancestry.json");let original=fs::read(&initial_history).unwrap();let mut changed=original.clone();changed.push(b' ');fs::write(&initial_history,changed).unwrap();
    assert!(worker(&profile,"changed-start-ancestry",None).err().unwrap().contains("changed"));fs::write(&initial_history,original).unwrap();
    let mut changed_profile:serde_json::Value=serde_json::from_slice(&fs::read(&profile).unwrap()).unwrap();changed_profile["library_sha256"]=json!("a".repeat(64));fs::write(&profile,serde_json::to_vec(&changed_profile).unwrap()).unwrap();assert!(load(&profile).err().unwrap().contains("plain interpreter"));
}
