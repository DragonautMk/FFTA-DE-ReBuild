use super::*;

#[test]
fn common_order_survives_guest_rewind_and_context_restores_on_early_return() {
    let clock = Clock::new(Session { run_id: "synthetic".into(), manifest_sha256: Some("a".repeat(64)) });
    let outer = Correlation::request("mailbox-1", "local-control", true);
    let _outer = clock.context(outer);
    let a = clock.event(9, 100, "control-request", json!({})).value(0, 0);
    fn nested(clock: &Clock) {
        let _inner = clock.context(Correlation::request("inner", "menu-navigation", false));
        assert_eq!(clock.current().request_id.as_deref(), Some("inner"));
    }
    nested(&clock);
    let b = clock.event(0, 0, "control-complete", json!({})).value(4, 2);
    assert_eq!(a["request_id"], "mailbox-1");
    assert_eq!(a["job_id"], b["job_id"]);
    assert_eq!(b["event_sequence"], 1);
    assert_ne!(a["event_id"], b["event_id"]);
    assert!(a["host_elapsed_ns"].as_u64().unwrap() <= b["host_elapsed_ns"].as_u64().unwrap());
    assert_eq!(b["frame"], 0);
    assert_eq!(b["session"]["manifest_sha256"], "a".repeat(64));
    assert_eq!(b["origin"]["source"], "local-control");
    assert_eq!(b["observation"], "host-boundary");
}

#[test]
fn poll_changes_are_coalesced_once_and_reset_does_not_reuse_event_ids() {
    let mut hub = Hub::new(Session::default());
    let a = hub.publish(0, 0, "mapped-poll", json!({"keys":1023})).unwrap().value(0, 0);
    for _ in 0..100 { assert!(hub.publish(0, 0, "mapped-poll", json!({"keys":1023})).is_none()); }
    assert_eq!(hub.clock.status()["next_event_sequence"], 1);
    hub.reset_polls();
    let b = hub.publish(0, 0, "mapped-poll", json!({"keys":1023})).unwrap().value(0, 0);
    assert_ne!(a["event_id"], b["event_id"]);
    let read = hub.clock.event(0, 0, "rom-read-batch", json!({"state":"unavailable"})).value(0, 0);
    assert_eq!(read["observation"], "guest-frame-aggregate");
    assert_eq!(read["category"], "guest");
    assert_eq!(read["request_id"], Value::Null);
    let error = hub.clock.event(0, 0, "control-result", json!({"outcome":"refused"})).value(0, 0);
    assert_eq!(error["severity"], "error");
    let unknown = hub.clock.event(0, 0, "new-unregistered-hook", json!({})).value(0, 0);
    assert_eq!(unknown["origin"]["producer"], "unregistered");
}

#[test]
fn real_menu_and_shared_command_metadata_fanout_preserves_machine_and_report_identity() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("debug-session.json"), "{\"synthetic\":true}").unwrap();
    let machine = gba_core::Machine::new(vec![0;16]);
    let before = machine.save_state().unwrap();
    let mut hub = Hub::new(Session::default());
    let mut trace = crate::debug_ui::Trace::default();
    trace.start_shared(json!({"synthetic":true}), hub.clock.clone()).unwrap();
    let mut recent = crate::debug_recent::Buffer::start_shared(root.path(), &machine, crate::debug_recent::Settings { on_error:false, ..Default::default() }, json!({}), json!(null), hub.clock.clone()).unwrap();
    let mut menu = menu::Menu::new(true, false);
    let mut av = input_config::AvConfig::default();
    let correlation = hub.clock.request("menu-navigation");
    let _context = hub.clock.context(correlation.clone());
    let mut emitted = Vec::new();
    let mut emit = |stage: &str, data: Value| {
        let event = hub.publish(0, 0, stage, data).unwrap();
        emitted.push(event.value(99, 99)); trace.accept(&event); recent.accept(&event);
    };
    emit("host-input-request", json!({"input":"Down"}));
    let before_menu = crate::debug_ui::focus(&menu, 940, 620);
    let action = menu.handle(menu::MenuInput::Down, &mut av);
    let after_menu = crate::debug_ui::focus(&menu, 940, 620);
    assert_ne!(before_menu, after_menu);
    emit("host-menu-dispatch", json!({"before":before_menu,"after":after_menu,"action":format!("{action:?}")}));
    emit("host-action", json!({"action":"Pause"}));
    emit("control-request", json!({"command":{"command":"pause"}}));
    let mut frame_control = crate::debug_control::Control::default();
    assert!(frame_control.apply(&crate::debug_control::Action::Pause, true, false).is_ok());
    emit("control-result", json!({"outcome":"accepted"}));
    assert!(frame_control.apply(&crate::debug_control::Action::Step { frames:0, keys:1023 }, true, false).is_err());
    emit("control-result", json!({"outcome":"refused"}));
    drop(emit);
    trace.freeze("synthetic-end");
    let recent_path = recent.save(root.path(), &machine).unwrap();
    let ui_path = trace.save(root.path(), json!({}), &[[0,0,0,255]], 1, 1).unwrap();
    let read = |path: &str, name: &str| -> Value {
        serde_json::from_slice(&std::fs::read(root.path().join(path).parent().unwrap().join(name)).unwrap()).unwrap()
    };
    let ui = read(&ui_path, "ui.json"); let ring = read(&recent_path, "recent.json");
    for original in emitted {
        let rows = |report: &Value| report["events"].as_array().unwrap().iter().filter(|e|e["event_id"]==original["event_id"]).cloned().collect::<Vec<_>>();
        let a = rows(&ui); let b = rows(&ring);
        assert_eq!(a.len(),1); assert_eq!(b.len(),1);
        for field in ["event_sequence","host_elapsed_ns","session","category","severity","origin","request_id","job_id","observation","data"] { assert_eq!(a[0][field],b[0][field],"{field}"); }
        assert_eq!(a[0]["request_id"], correlation.request_id.as_deref().unwrap());
    }
    assert_eq!(ui["event_coverage"],ring["event_coverage"]);
    assert!(!ui["event_coverage"]["missing"].as_array().unwrap().is_empty());
    let next = hub.clock.status()["next_event_sequence"].clone();
    // Reading status/coverage and republishing an already saved report is not
    // a producer and cannot recursively add observations.
    let _ = trace.status(); let _ = recent.status(); let _ = coverage();
    recent.save(root.path(), &machine).unwrap();
    assert_eq!(hub.clock.status()["next_event_sequence"],next);
    assert_eq!(machine.save_state().unwrap(),before);
}

#[test]
fn poll_coalescing_can_be_disabled_and_restored_without_reusing_ids() {
    let mut hub=Hub::new(Session::default());
    let one=hub.publish_with_poll_coalescing(0,0,"mapped-poll",json!({"keys":1023}),true).unwrap().value(0,0);
    assert!(hub.publish_with_poll_coalescing(0,0,"mapped-poll",json!({"keys":1023}),true).is_none());
    let all=hub.publish_with_poll_coalescing(0,0,"mapped-poll",json!({"keys":1023}),false).unwrap().value(1,1);
    assert_ne!(one["event_id"],all["event_id"]);
    assert!(hub.publish_with_poll_coalescing(0,0,"mapped-poll",json!({"keys":1023}),true).is_none());
}
