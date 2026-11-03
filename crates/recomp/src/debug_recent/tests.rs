use super::*;
fn root()->tempfile::TempDir{
    let d=tempfile::tempdir().unwrap();fs::write(d.path().join("debug-session.json"),"{\"synthetic\":true}").unwrap();d
}
#[test]
fn million_event_run_rolls_past_old_prefix_time_and_both_storage_limits(){
    let mut b=Buffer::new(Settings{events:8,..Default::default()},json!({"synthetic":true}));
    b.started=Instant::now()-std::time::Duration::from_secs(90);
    for i in 0..1_000_000u64{b.push(i,i*3,"synthetic-frame",json!({"keys":i&0x3ff}));}
    assert!(b.enabled());assert_eq!(b.entries.len(),8);assert_eq!(b.evicted,999_992);assert_eq!(b.next,1_000_000);
    assert_eq!(b.entries.front().unwrap().value["sequence"],999_992);
    assert!(b.entries.back().unwrap().value["host_ms"].as_u64().unwrap()>=90_000);
    let before=b.entries.back().unwrap().value.clone();b.push(1_000_001,3_000_003,"oversize",json!({"text":"x".repeat(EVENT_LIMIT)}));
    assert_eq!(b.oversize,1);assert_eq!(b.entries.back().unwrap().value,before);
    let mut bytes=Buffer::new(Default::default(),json!({}));
    for i in 0..1000{bytes.push(i,i,"large",json!({"text":"x".repeat(7000)}));}
    assert!(bytes.entries.len()<1024&&bytes.bytes<=BYTE_LIMIT&&bytes.evicted>0);
    assert_eq!(bytes.bytes,bytes.entries.iter().map(|e|e.bytes).sum::<usize>());
}
#[test]
fn controlled_error_keeps_message_at_min_capacity_and_failed_save_retains_context(){
    let r=root();let m=Machine::new(vec![0;16]);let before=m.save_state().unwrap();
    let mut b=Buffer::start(r.path(),&m,Settings{events:1,..Default::default()},json!({"synthetic":"</script><img>"}),json!({"recognized":null})).unwrap();
    assert!(b.error(&m,"controlled missing-code request"));assert!(!b.enabled());assert_eq!(b.entries.back().unwrap().value["stage"],"error");
    b.event(&m,"post-freeze",json!({}));assert_eq!(b.entries.len(),1);
    let base=r.path().join("Data/config/recent");fs::create_dir_all(&base).unwrap();for i in 0..8{fs::create_dir(base.join(i.to_string())).unwrap();}
    assert!(b.save(r.path(),&m).is_err());assert_eq!(b.entries.back().unwrap().value["data"]["message"],"controlled missing-code request");assert!(b.write_error.is_some());
    for i in 0..8{fs::remove_dir(base.join(i.to_string())).unwrap();}
    let report=b.save(r.path(),&m).unwrap();assert_eq!(b.save(r.path(),&m).unwrap(),report);
    assert_eq!(fs::read_dir(&base).unwrap().count(),1);assert!(!fs::read_to_string(r.path().join(&report)).unwrap().contains("</script><img>"));
    assert_eq!(before,m.save_state().unwrap());
    let mut b=Buffer::new(Settings{on_error:false,..Default::default()},json!({}));assert!(!b.error(&m,"observe without trigger"));assert!(b.enabled());
    b.scene(&m,json!({"oversize":"x".repeat(10000)}));assert_eq!(b.last_scene.as_ref().unwrap()["unknown"],"Scene metadata exceeded2048bytes");
    b.freeze(&m,"explicit");assert_eq!(b.reason,"explicit");
}
#[test]
fn actual_native_dispatch_and_read_observation_preserve_full_state(){
    extern "C" fn translated(_: *const gba_core::capi::RtApi,m:*mut std::ffi::c_void)->u32{gba_core::capi::rt_interp_one(m)}
    let rom=[0xe3a00001u32,0xeafffffdu32].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>();
    let table=crate::BlockTable::from_blocks(&[gba_core::capi::RcgBlock{key:crate::ROM_BASE,func:translated}]).unwrap();
    let mut plain=Machine::new(rom.clone());let mut observed=Machine::new(rom);let r=root();
    let p=&mut plain as *mut Machine as *mut std::ffi::c_void;let expected=crate::run_frame_native(&mut plain,&table,p,12);
    let mut b=Buffer::start(r.path(),&observed,Default::default(),json!({"synthetic":true}),json!(null)).unwrap();
    let mut frame=Frame::start(&mut b,&observed,true);
    let p=&mut observed as *mut Machine as *mut std::ffi::c_void;
    let actual=crate::run_frame_native_traced(&mut observed,&table,p,12,|_,kind|frame.native(kind),|_,_,_|{});
    frame.finish(&mut b,&observed,false);
    assert_eq!(actual,expected);assert_eq!(actual,(6,6));assert_eq!(plain.save_state().unwrap(),observed.save_state().unwrap());
    let event=&b.entries.back().unwrap().value;assert_eq!(event["data"]["actual_dispatch"]["native"],6);assert_eq!(event["data"]["actual_dispatch"]["missing"],6);
    let mut b=Buffer::new(Settings{reads:false,..Default::default()},json!({}));
    Frame::start(&mut b,&observed,false).finish(&mut b,&observed,false);
    assert_eq!(b.entries.iter().find(|e|e.value["stage"]=="rom-read-batch").unwrap().value["data"]["state"],"disabled");
    let mut system_only=Frame::start(&mut b,&observed,true);system_only.native(crate::NativeDispatch::System);system_only.finish(&mut b,&observed,false);
    assert_eq!(b.entries.back().unwrap().value["data"]["native_lookup_performed"],false);
    assert_eq!(b.entries.back().unwrap().value["data"]["actual_dispatch"]["system"],1);

}
#[cfg(feature="rom-data-read-audit")]
#[test]
fn actual_filtered_raw_reads_report_drops_and_collision_without_guest_writes(){
    use gba_core::Bus;
    use gba_core::rom_data::read_audit::{TraceCapture,TraceFilter};
    let mut m=Machine::new(vec![0;256]);let before=m.save_state().unwrap();
    let mut b=Buffer::new(Default::default(),json!({}));let frame=Frame::start(&mut b,&m,false);
    for i in 0..32{m.bus.read8(0x08000000+i);}
    frame.finish(&mut b,&m,false);
    let reads=&b.entries.iter().find(|e|e.value["stage"]=="rom-read-batch").unwrap().value["data"];
    assert_eq!(reads["state"],"observed");assert!(reads["selected"]["dropped_requests"].as_u64().unwrap()>0);
    assert_eq!(reads["selected"]["events"].as_array().unwrap().len(),READ_KEYS);
    assert!(reads["selected"]["events"].as_array().unwrap().iter().all(|e|e["key"]["service"]=="Raw"));
    assert_eq!(before,m.save_state().unwrap());
    let external=TraceCapture::start(TraceFilter{readers:vec![],ranges:vec![]},16).unwrap();
    let mut collision=Buffer::new(Default::default(),json!({}));
    Frame::start(&mut collision,&m,false).finish(&mut collision,&m,false);
    assert_eq!(collision.entries.iter().find(|e|e.value["stage"]=="rom-read-batch").unwrap().value["data"]["state"],"observer-unavailable");assert_eq!(collision.read_unavailable,1);drop(external);
}
#[cfg(not(feature="rom-data-read-audit"))]
#[test]
fn absent_feature_is_unavailable_and_not_zero_reads(){
    let m=Machine::new(vec![0;16]);let mut b=Buffer::new(Default::default(),json!({}));
    Frame::start(&mut b,&m,false).finish(&mut b,&m,false);
    assert_eq!(b.entries.iter().find(|e|e.value["stage"]=="rom-read-batch").unwrap().value["data"]["state"],"feature-unavailable");assert_eq!(b.read_unavailable,1);
}
#[test]
#[ignore="explicit local ROM/seed/output; new observer check, no distributed game fixture"]
fn retained_seed_interpreter_observer_preserves_two_frames(){
    let path=|key|std::path::PathBuf::from(std::env::var_os(key).expect(key));
    let rom=fs::read(path("FFTA_RECENT_ROM")).unwrap();let seed=fs::read(path("FFTA_RECENT_STATE")).unwrap();let out=path("FFTA_RECENT_OUTPUT");
    assert!(!out.exists());fs::create_dir_all(&out).unwrap();fs::write(out.join("debug-session.json"),"{\"validation_fixture\":true}").unwrap();
    let mut plain=crate::make_machine(rom.clone(),None);crate::save_state::load_bytes(&mut plain,&seed).unwrap();
    let mut observed=crate::make_machine(rom,None);crate::save_state::load_bytes(&mut observed,&seed).unwrap();
    let mut b=Buffer::start(&out,&observed,Default::default(),json!({"scope":"Same interpreter observer off/on, not new scene acceptance","seed_sha256":gba_core::ffta_rom::digest(&seed),
        "test_executable_sha256":gba_core::ffta_rom::digest(&fs::read(std::env::current_exe().unwrap()).unwrap())}),json!({"unknown":"No frontend scene recognizer in this fixture"})).unwrap();
    for keys in [0x3fe,0x3ff]{
        plain.bus.keys=keys;observed.bus.keys=keys;
        plain.run_frame(5_000_000);
        let frame=Frame::start(&mut b,&observed,false);observed.run_frame(5_000_000);frame.finish(&mut b,&observed,true);
        assert_eq!(plain.save_state().unwrap(),observed.save_state().unwrap());assert!(observed.bus.frame_ready);
    }
    assert_eq!(b.completed_frames,2);assert!(b.entries.iter().any(|e|e.value["stage"]=="rom-read-batch"));
    assert!(b.error(&observed,"Controlled fixture error after two exact frames"));
    println!("RECENT_REPORT {}",b.save(&out,&observed).unwrap());
}

#[test]
fn direct_recent_events_share_identity_with_console_without_guest_changes(){
    let m=Machine::new(vec![0;16]);let before=m.save_state().unwrap();
    let console=std::rc::Rc::new(std::cell::RefCell::new(crate::debug_console::Window::default()));
    console.borrow_mut().start();let mut recent=Buffer::new(Default::default(),json!({}));
    recent.console=Some(std::rc::Rc::downgrade(&console));
    recent.event(&m,"recognized-ui-transition",json!({"synthetic":true}));
    let value=recent.entries.back().unwrap().value.clone();
    let view=console.borrow();let shown=view.visible();assert_eq!(shown.len(),1);
    assert_eq!(shown[0]["event_id"],value["event_id"]);assert_eq!(shown[0]["data"],value["data"]);
    assert_eq!(m.save_state().unwrap(),before);
}
