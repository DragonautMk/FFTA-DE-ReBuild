use super::*;
fn machine() -> Machine {
    let mut rom = vec![0; 512];
    rom[..4].copy_from_slice(&0xeafffffeu32.to_le_bytes());
    rom[0x100..0x106].copy_from_slice(b"SRAM_V");
    let mut m = crate::make_machine(rom, None);
    m.bus.frames = 17;
    m
}
fn script() -> crate::InputScript {
    crate::debug_recording::script(b"gba-input v1\n0 3ff\n", 4).unwrap()
}
fn settings() -> Settings {
    Settings {
        frames: 3,
        backend: "interp".into(),
        audio: "preserve".into(),
    }
}
#[test]
fn observation_does_not_change_machine_and_float_payload_bits_stay_exact() {
    let mut m = machine();
    m.bus.hle_tap = vec![f32::from_bits(0x7fc00001)];
    let before = m.save_state().unwrap();
    let (a, _) = observe(&m).unwrap();
    assert_eq!(m.save_state().unwrap(), before);
    m.bus.hle_tap[0] = f32::from_bits(0x7fc00002);
    let (b, _) = observe(&m).unwrap();
    assert!(changed(&a, &b).contains(&"hle".into()));
    assert_ne!(a.hashes["full_state"], b.hashes["full_state"]);
}
#[test]
fn complete_matching_prefix_has_correct_previous_and_counts_each_stream_once() {
    let (mut left, mut right, mut reference) = (machine(), machine(), machine());
    let d = loop_compare(
        &mut left,
        &mut right,
        &settings(),
        &script(),
        |m| {
            m.run_frame(5_000_000);
            Ok((0, 0))
        },
        |_| None,
    )
    .unwrap();
    assert_eq!(d.outcome, "matched");
    assert_eq!(d.completed, 3);
    assert_eq!(d.previous.as_ref().unwrap().0.guest_frame, 19);
    let mut values = 0;
    for _ in 0..3 {
        drain(&mut reference);
        reference.run_frame(5_000_000);
        values += reference.bus.audio_buf.len();
    }
    assert_eq!(right.save_state().unwrap(), reference.save_state().unwrap());
    assert_eq!(d.audio_totals[0]["mixed_i16_values"], values);
    assert_eq!(d.audio_totals[0], d.audio_totals[1]);
}
#[test]
fn first_changed_frame_locates_named_fields_memory_registers_and_paired_artifacts() {
    let t = tempfile::tempdir().unwrap();
    let (mut left, mut right) = (machine(), machine());
    let d = loop_compare(
        &mut left,
        &mut right,
        &settings(),
        &script(),
        |m| {
            m.run_frame(5_000_000);
            if m.bus.frames == 19 {
                m.cpu.regs[0] = 7;
                m.bus.ewram[9] = 42;
                m.bus.oam[4] = 12;
                m.bus.framebuffer[3] = 31;
            }
            Ok((0, 0))
        },
        |_| None,
    )
    .unwrap();
    assert_eq!(d.outcome, "diverged");
    assert_eq!(d.completed, 2);
    assert_eq!(d.previous.as_ref().unwrap().0.guest_frame, 18);
    assert_eq!(d.left.clock, d.right.clock);
    let path = publish(
        t.path(),
        "known-diff",
        &settings(),
        json!({"case":"synthetic"}),
        Value::Null,
        &d,
        &left,
        &right,
    )
    .unwrap();
    let dir = t.path().join(path).parent().unwrap().to_owned();
    let report: Value =
        serde_json::from_slice(&fs::read(dir.join("comparison.json")).unwrap()).unwrap();
    assert_eq!(report["first_difference"], 2);
    assert_eq!(report["fields"]["registers"][0]["register"], "r0");
    let names: Vec<_> = report["fields"]["different"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["field"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"cpu.regs") && names.contains(&"bus.ewram") && names.contains(&"bus.oam")
    );
    assert_eq!(report["value_differences"]["ewram"]["first"][0]["index"], 9);
    assert_eq!(
        report["value_differences"]["ewram"]["first"][0]["right"],
        42
    );
    for file in [
        "left.ffstate",
        "right.ffstate",
        "previous.ffstate",
        "left-central.png",
        "right-central.png",
        "left-wide.png",
        "right-wide.png",
        "previous-wide.png",
    ] {
        assert!(dir.join(file).is_file(), "{file}");
    }
    assert!(publish(
        t.path(),
        "known-diff",
        &settings(),
        Value::Null,
        Value::Null,
        &d,
        &left,
        &right
    )
    .is_err());
    assert!(publish(
        t.path(),
        "../escape",
        &settings(),
        Value::Null,
        Value::Null,
        &d,
        &left,
        &right
    )
    .is_err());
}
#[test]
fn origin_difference_is_frame_zero_without_a_fictitious_previous_point() {
    let (mut left, mut right) = (machine(), machine());
    right.cpu.regs[1] = 1;
    let d = loop_compare(
        &mut left,
        &mut right,
        &settings(),
        &script(),
        |_| panic!("No frame should execute"),
        |_| None,
    )
    .unwrap();
    assert_eq!(d.outcome, "diverged");
    assert_eq!(d.completed, 0);
    assert!(d.previous.is_none());
}
#[test]
fn cancellation_and_timeout_are_partial_without_heavy_or_first_difference_claim() {
    for (name, at) in [("interrupted", 0), ("timeout", 1)] {
        let t = tempfile::tempdir().unwrap();
        let (mut left, mut right) = (machine(), machine());
        let d = loop_compare(
            &mut left,
            &mut right,
            &settings(),
            &script(),
            |m| {
                m.run_frame(5_000_000);
                Ok((0, 0))
            },
            |n| (n == at).then(|| (name, "controlled stop".into())),
        )
        .unwrap();
        assert_eq!(d.outcome, name);
        assert_eq!(d.completed, at);
        let path = publish(
            t.path(),
            name,
            &settings(),
            Value::Null,
            Value::Null,
            &d,
            &left,
            &right,
        )
        .unwrap();
        let dir = t.path().join(path).parent().unwrap().to_owned();
        let report: Value =
            serde_json::from_slice(&fs::read(dir.join("comparison.json")).unwrap()).unwrap();
        assert!(report["first_difference"].is_null());
        assert!(report["artifacts"].is_null());
        assert_eq!(fs::read_dir(dir).unwrap().count(), 2);
    }
}
#[test]
fn incomplete_candidate_frame_is_error_not_first_aligned_divergence() {
    let (mut left, mut right) = (machine(), machine());
    let d = loop_compare(
        &mut left,
        &mut right,
        &settings(),
        &script(),
        |_| Ok((0, 0)),
        |_| None,
    )
    .unwrap();
    assert_eq!(d.outcome, "error");
    assert_eq!(d.completed, 0);
}
#[test]
fn profile_bounds_mode_library_and_retention_are_explicit() {
    let mut s = settings();
    for n in [0, 5, 3601] {
        s.frames = n;
        assert!(validate(&s, 4, false).is_err());
    }
    s.frames = 2;
    s.backend = "native".into();
    assert!(validate(&s, 4, false).is_err());
    assert!(validate(&s, 4, true).is_ok());
    s.audio = "unknown".into();
    assert!(validate(&s, 4, true).is_err());
    let t = tempfile::tempdir().unwrap();
    assert!(capacity(t.path()).is_ok());
    let p = t.path().join("comparisons");
    fs::create_dir(&p).unwrap();
    for n in 0..8 {
        fs::create_dir(p.join(n.to_string())).unwrap();
    }
    assert!(capacity(t.path()).is_err());
    assert_eq!(fs::read_dir(p).unwrap().count(), 8);
}
#[test]
fn matched_run_publishes_only_bounded_metadata_and_no_scene_acceptance() {
    let t = tempfile::tempdir().unwrap();
    let mut right = machine();
    let seed = right.save_state().unwrap();
    let result = run(
        t.path(),
        "matched",
        &settings(),
        json!({"case":"synthetic"}),
        &mut right,
        &seed,
        &script(),
        None,
        |_| None,
    )
    .unwrap();
    assert_eq!(result.outcome, "matched");
    let dir = t.path().join(result.report).parent().unwrap().to_owned();
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
    let report: Value =
        serde_json::from_slice(&fs::read(dir.join("comparison.json")).unwrap()).unwrap();
    assert!(report["artifacts"].is_null());
    assert!(report["fields"].is_null());
    assert_eq!(report["completed"], 3);
}
