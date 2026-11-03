use super::*;
fn settings() -> Settings {
    Settings {
        preset: "all".into(),
        from: 0,
        to: 2,
        ranges: Vec::new(),
        limit: 50000,
        known_sha256: None,
    }
}
fn m() -> gba_core::Machine {
    let mut rom = vec![0; 0x200];
    for (offset, word) in [
        (0, 0xe59f0010u32),
        (4, 0xe5901000),
        (8, 0xeafffffc),
        (24, 0x08000100),
        (256, 0x12345678),
    ] {
        rom[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
    }
    crate::make_machine(rom, None)
}
#[test]
fn physical_interval_union_and_known_overlap_do_not_double_count_mirrors_or_widths() {
    let actual = merge(vec![(0, 4), (0, 1), (2, 6), (8, 10), (6, 8)]);
    assert_eq!(actual, [(0, 10)]);
    assert_eq!(bytes(&actual), 10);
    let known = merge(vec![(2, 5), (4, 7), (9, 12)]);
    assert_eq!(intersection(&actual, &known), [(2, 7), (9, 10)]);
    assert_eq!(bytes(&intersection(&actual, &known)), 6);
}
#[test]
fn invalid_windows_ranges_and_event_limits_are_explicit_errors() {
    let mut s = settings();
    assert!(validate(&s, 2).is_ok());
    s.to = 3;
    assert!(validate(&s, 2).is_err());
    s.to = 2;
    s.from = 2;
    assert!(validate(&s, 2).is_err());
    s.from = 0;
    s.limit = 0;
    assert!(validate(&s, 2).is_err());
    s.limit = 50001;
    assert!(validate(&s, 2).is_err());
    s.limit = 8;
    s.ranges = vec![(4, 4)];
    assert!(validate(&s, 2).is_err());
}
fn known(rom: &[u8]) -> Vec<u8> {
    let m = gba_core::rom_data::Manifest {
        schema: 1,
        extractor_revision: 1,
        decoder_revision: 1,
        source_rom_sha256: gba_core::ffta_rom::digest(rom),
        source_rom_size: rom.len() as u32,
        profile: "fixture".into(),
        coverage: vec!["fixture-source".into()],
        resources: vec![gba_core::rom_data::Resource {
            id: "fixture-source".into(),
            path: "metadata.json".into(),
            format: "fixture".into(),
            sha256: String::new(),
            dependencies: vec![],
            sources: vec![gba_core::rom_data::SourceRange {
                offset: 24,
                length: 4,
                sha256: String::new(),
                role: "fixture literal".into(),
            }],
        }],
    };
    toml::to_string(&m).unwrap().into_bytes()
}
#[test]
fn known_manifest_checks_identity_and_ranges_without_running_an_extractor() {
    let machine = m();
    let bytes = known(&machine.bus.rom);
    assert!(manifest(&bytes, &machine.bus.rom).is_ok());
    let mut wrong = machine.bus.rom.clone();
    wrong[0x180] = 1;
    assert!(manifest(&bytes, &wrong).is_err());
    let mut parsed: gba_core::rom_data::Manifest =
        toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
    parsed.resources[0].sources[0].length = u32::MAX;
    assert!(manifest(
        toml::to_string(&parsed).unwrap().as_bytes(),
        &machine.bus.rom
    )
    .is_err());
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
fn bounded_trace_filters_drops_and_legacy_projection_remain_independent() {
    use gba_core::{
        rom_data::read_audit::{Capture, Reader, Service, SiteCapture, TraceCapture, TraceFilter},
        Bus,
    };
    let mut machine = m();
    let legacy = Capture::start();
    let sites = SiteCapture::start_all();
    let capture = TraceCapture::start(
        TraceFilter {
            readers: vec![Reader::Bus],
            ranges: vec![(0x100, 0x102)],
        },
        1,
    )
    .unwrap();
    machine.bus.read32(0x08000100);
    capture.frame(Some(42));
    machine.bus.read16(0x08000100);
    machine.bus.read16(0x08000100);
    machine.bus.read8(0x0a000100);
    machine.bus.read8(0x08000110);
    let report = capture.finish();
    let legacy = legacy.finish();
    let sites = sites.finish();
    assert_eq!(
        sites.iter().map(|s| s.read).collect::<BTreeSet<_>>(),
        legacy
    );
    assert_eq!(report.events.len(), 1);
    assert_eq!(report.events[0].requests, 2);
    assert_eq!(report.events[0].first_guest_frame, 42);
    assert_eq!(report.events[0].key.service, Service::Raw);
    assert!(report.events[0].key.instruction.is_none());
    assert!(report.truncated);
    assert_eq!(report.dropped_requests, 1);
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
fn observer_preserves_each_complete_state_clock_and_excludes_instruction_fetches() {
    let mut plain = m();
    let mut observed = m();
    let temp = tempfile::tempdir().unwrap();
    let mut s = settings();
    s.from = 1;
    let mut audit = Audit::start(temp.path(), &s, &observed.bus.rom).unwrap();
    for relative in 0..2 {
        audit.capture.frame(None);
        plain.run_frame(5_000_000);
        audit.frame(relative, observed.bus.frames);
        observed.run_frame(5_000_000);
        assert_eq!(plain.save_state().unwrap(), observed.save_state().unwrap());
    }
    let report = audit.finish(2, "passed", "fixture", "run", "fixture-rom");
    assert_eq!(report["window"]["frames_entered"], 1);
    assert_eq!(report["window"]["complete"], true);
    assert!(report["fetch_bus_requests_excluded"].as_u64().unwrap() > 0);
    assert_eq!(report["observed_distinct_bytes"], 8);
    assert_eq!(report["known_plan"]["compared"], false);
    let events = report["events"].as_array().unwrap();
    assert!(events
        .iter()
        .all(|e| e["key"]["instruction"]["thumb"] == false));
    assert!(events.iter().all(|e| [4u64, 0].contains(
        &e["key"]["instruction"]["pc"]
            .as_u64()
            .unwrap()
            .saturating_sub(0x08000000)
    )));
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
fn known_plan_and_truncation_reports_describe_retained_evidence_only() {
    use gba_core::Bus;
    let mut machine = m();
    let temp = tempfile::tempdir().unwrap();
    let bytes = known(&machine.bus.rom);
    std::fs::write(temp.path().join("known-plan.toml"), &bytes).unwrap();
    let mut s = settings();
    s.known_sha256 = Some(gba_core::ffta_rom::digest(&bytes));
    s.limit = 1;
    let mut audit = Audit::start(temp.path(), &s, &machine.bus.rom).unwrap();
    audit.frame(0, 0);
    machine.bus.read32(0x08000018);
    machine.bus.read8(0x08000100);
    let report = audit.finish(1, "interrupted", "fixture", "run", "fixture-rom");
    assert_eq!(report["truncated"], true);
    assert_eq!(report["window"]["complete"], false);
    assert_eq!(report["known_plan"]["within_distinct_bytes"], 4);
    assert_eq!(report["known_plan"]["outside_distinct_bytes"], 0);
    assert_eq!(report["observed_distinct_bytes"], 4);
    assert_eq!(report["known_plan"]["resources"][0]["id"], "fixture-source");
}
