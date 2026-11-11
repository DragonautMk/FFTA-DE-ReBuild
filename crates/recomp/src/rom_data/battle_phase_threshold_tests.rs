//! EUR battle-task conditional threshold source and original live phase handling.
use super::*;
use gba_core::{Bus, Machine};

fn original_prefix(machine: &mut Machine, phase: i8, old_counter: u8) {
    const TASK: u32 = 0x0202_0000;
    machine.bus.write32(0x0200_918c, TASK);
    machine.bus.write8(TASK + 0x38, phase as u8);
    machine.bus.write8(TASK + 0x39, old_counter.wrapping_add(1));
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[1] = u32::from(old_counter);
    machine.cpu.regs[3] = 0x0200_918c;
    machine.cpu.regs[15] = 0x0801_31d6;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..100 {
        if machine.cpu.regs[15] == 0x0801_320c { return; }
        machine.step();
    }
    panic!("original battle phase prefix did not reach its continuation");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E147 predecessor, retained battle state and translation; Wine only"]
fn european_battle_phase_thresholds_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const ROOT: usize = 0x37101c;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_PHASE_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_PHASE_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_BATTLE_PHASE_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "097d7ccc5a9ee0e5315a91b9e574c7d34f22ebd96813ec98b5a24f61e06c11fe");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (147, 146));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (148, 147));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e148-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e148-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBSERVED_TASK_PARAMETERS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_observed_task_parameters(&old_bytes).unwrap();
        let mut current = parse_observed_task_parameters(&new_bytes).unwrap();
        assert!(prior.battle_phase_thresholds.is_none());
        assert_eq!(current.battle_phase_thresholds.take().unwrap(), rom[ROOT..ROOT + 3]);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "battle-phase-threshold");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.observed_task_parameters_stats().unwrap();
    managed.bus.rom[ROOT..ROOT + 3].fill(0xcd);
    for i in 0..3 { assert_eq!(managed.bus.read8(0x0800_0000 + (ROOT + i) as u32), rom[ROOT + i]); }
    let after = owner.observed_task_parameters_stats().unwrap();
    assert_eq!(after.battle_phase_threshold_bytes - before.battle_phase_threshold_bytes, 3);
    assert_eq!(after.corrections_used, before.corrections_used);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [ROOT, ROOT + 3] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[ROOT..ROOT + 3].copy_from_slice(&rom[ROOT..ROOT + 3]);
    for phase in [-1i8, 0, 1, 2, 3] {
        for old_counter in [0u8, 255] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let initial = raw.save_state().unwrap();
            managed.load_state(&initial).unwrap();
            managed.bus.rom[ROOT..ROOT + 3].fill(0xcd);
            original_prefix(&mut raw, phase, old_counter);
            original_prefix(&mut managed, phase, old_counter);
            let threshold = rom[ROOT.wrapping_add_signed(phase as isize)];
            let advanced = old_counter >= threshold;
            let incremented = phase.wrapping_add(1);
            let expected_phase = if advanced { if incremented > 2 { 0 } else { incremented } } else { phase };
            let expected_counter = if advanced { 0 } else { old_counter.wrapping_add(1) };
            assert_eq!(raw.bus.read8(0x0202_0038), expected_phase as u8);
            assert_eq!(raw.bus.read8(0x0202_0039), expected_counter);
            managed.bus.rom[ROOT..ROOT + 3].copy_from_slice(&rom[ROOT..ROOT + 3]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "phase prefix {phase}, counter {old_counter}");
        }
    }
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.observed_task_parameters_stats().unwrap();
        let mut typed = 0u64;
        let mut raw_reads = 0u64;
        let mut dispatches = 0u64;
        for frame in 0..3 {
            managed.bus.rom[ROOT..ROOT + 3].fill(0xcd);
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(ROOT as u32, (ROOT + 3) as u32)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else { managed.bus.frame_ready = false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            typed += report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum::<u64>();
            raw_reads += report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum::<u64>();
            managed.bus.rom[ROOT..ROOT + 3].copy_from_slice(&rom[ROOT..ROOT + 3]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary battle phase state/clock native={native}, frame={frame}");
        }
        let bytes = owner.observed_task_parameters_stats().unwrap().battle_phase_threshold_bytes - before.battle_phase_threshold_bytes;
        assert!(typed > 0 && raw_reads == 0 && bytes > 0);
        if native { assert!(dispatches > 0); }
        results.push((typed, raw_reads, bytes, dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[148,147], "new_source_bytes":3, "e147_projection_preserved":true,
        "source_rom_poisoned":true, "controlled_original_phase_cases":10,
        "ordinary_interpreter_and_native_dispatch_pairs":results, "full_state_clock_frames_each":3,
        "ordinary_phase_zero":"pending", "per_read_native_attribution":"pending", "frontend_ui":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
