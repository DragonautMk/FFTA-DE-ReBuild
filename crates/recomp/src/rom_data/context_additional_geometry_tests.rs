//! EUR context geometry sources and their original copy helpers.
use super::*;
use gba_core::{rom_data::layered_obj_animations::CONTEXT_ADDITIONAL_GEOMETRY, Bus, Machine};

fn call_helper(machine: &mut Machine, helper: u32, source: u32, argument: u32) {
    machine.cpu.regs[0] = 0x0202_0000;
    machine.cpu.regs[1] = 0x0800_0000 + source;
    machine.cpu.regs[2] = argument;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = helper;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..1000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == 0x0800_0300 { return; }
    }
    panic!("original context helper did not return");
}

#[test]
#[ignore = "exact EUR ROM and E143 predecessor; Wine only"]
fn european_additional_geometry_publication_and_helpers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TARGET_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TARGET_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (143, 142));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (144, 143));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e144-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e144-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old_resource.sha256);
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut current = parse_layered_obj_animations(&new_bytes).unwrap();
        assert!(prior.context_additional_geometry.is_empty());
        assert_eq!(current.context_additional_geometry.len(), 6);
        for (record, at) in current.context_additional_geometry.iter().zip(CONTEXT_ADDITIONAL_GEOMETRY) {
            assert_eq!(record.source_offset, at);
            assert_eq!(record.source_bytes(), rom[at as usize..at as usize + 8]);
        }
        current.context_additional_geometry.clear();
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "context-additional-geometry");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.layered_obj_animations_stats().unwrap();
    managed.bus.rom[0x3ae174..0x3ae1a4].fill(0xcd);
    for at in 0x3ae174..0x3ae1a4 {
        assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]);
    }
    assert_eq!(owner.layered_obj_animations_stats().unwrap().additional_geometry_bytes - before.additional_geometry_bytes, 48);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [0x3ae174usize, 0x3ae17c, 0x3ae194, 0x3ae1a4] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[0x3ae174..0x3ae1a4].copy_from_slice(&rom[0x3ae174..0x3ae1a4]);
    for (offset, helper, argument, destinations) in [
        (0x3ae174, 0x0814_31f4, 1, &[4, 0x14][..]),
        (0x3ae17c, 0x0814_31f4, 0, &[4, 0x14][..]),
        (0x3ae184, 0x0814_31f4, 1, &[4, 0x14][..]),
        (0x3ae18c, 0x0814_31f4, 0, &[4, 0x14][..]),
        (0x3ae194, 0x0814_31f4, 0, &[4, 0x14][..]),
        (0x3ae194, 0x0814_3238, 180, &[0x0c][..]),
        (0x3ae19c, 0x0814_3238, 60, &[0x0c][..]),
    ] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let state = raw.save_state().unwrap();
        managed.load_state(&state).unwrap();
        managed.bus.rom[0x3ae174..0x3ae1a4].fill(0xcd);
        call_helper(&mut raw, helper, offset, argument);
        call_helper(&mut managed, helper, offset, argument);
        for &destination in destinations {
            for i in 0..8 {
                assert_eq!(raw.bus.read8(0x0202_0000 + destination + i), rom[offset as usize + i as usize]);
                assert_eq!(managed.bus.read8(0x0202_0000 + destination + i), rom[offset as usize + i as usize]);
            }
        }
        managed.bus.rom[0x3ae174..0x3ae1a4].copy_from_slice(&rom[0x3ae174..0x3ae1a4]);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "helper state and clock for {offset:x}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[144,143], "source_records":6, "source_bytes":48,
        "prior_resources_preserved":true, "source_rom_poisoned":true,
        "controlled_original_helper_cases":7, "ordinary_use":"pending", "native_dispatch":"pending",
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR checkpoint, E144 dataset and translation; Wine only"]
fn european_additional_geometry_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_TARGET_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TARGET_DATASET").unwrap());
    let input = fs::read(std::env::var_os("FFTA_TARGET_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let calls = [
        (0x0814_10ba, 0x3ae174, 1, 0x8bb0),
        (0x0814_131c, 0x3ae17c, 0, 0x8b8c),
        (0x0814_14e8, 0x3ae184, 1, 0x8bb0),
        (0x0814_16d8, 0x3ae18c, 0, 0x8b8c),
        (0x0814_16e4, 0x3ae194, 180, 0x8b8c),
        (0x0814_1780, 0x3ae194, 0, 0x8b8c),
        (0x0814_178c, 0x3ae19c, 60, 0x8b8c),
    ];
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    observer.bus.frame_ready = false;
    let mut checkpoints: Vec<Option<Vec<u8>>> = vec![None; calls.len()];
    let mut frames = 0u32;
    let mut steps = 0u32;
    while checkpoints.iter().any(Option::is_none) && frames < 2000 {
        let pc = observer.cpu.regs[15];
        if let Some((index, &(_, source, argument, context_offset))) = calls.iter().enumerate().find(|(_, c)| c.0 == pc) {
            if checkpoints[index].is_none() {
                assert_eq!(observer.cpu.regs[1], 0x0800_0000 + source);
                assert_eq!(observer.cpu.regs[2], argument);
                let heap = u32::from_le_bytes(observer.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
                assert_eq!(observer.cpu.regs[0], heap + context_offset);
                checkpoints[index] = Some(observer.save_state().unwrap());
            }
        }
        observer.step();
        steps += 1;
        assert!(steps < 5_000_000, "ordinary observation frame limit");
        if observer.bus.frame_ready { frames += 1; steps = 0; observer.bus.frame_ready = false; }
    }
    assert!(checkpoints.iter().all(Option::is_some), "ordinary calls observed: {:?}", checkpoints.iter().map(Option::is_some).collect::<Vec<_>>());
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for (index, checkpoint) in checkpoints.iter().enumerate() {
        for native in [false, true] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let mut managed = crate::make_machine(rom.clone(), None);
            attach_from_directory(&mut managed, &dataset).unwrap();
            raw.load_state(checkpoint.as_ref().unwrap()).unwrap();
            managed.load_state(checkpoint.as_ref().unwrap()).unwrap();
            managed.bus.rom[0x3ae174..0x3ae1a4].fill(0xcd);
            let owner = managed.bus.rom_resources().unwrap().clone();
            let before = owner.layered_obj_animations_stats().unwrap();
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(0x3ae174, 0x3ae1a4)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            let dispatches = if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
                0
            };
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            let typed: u64 = report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum();
            let raw_reads: u64 = report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
            let bytes = owner.layered_obj_animations_stats().unwrap().additional_geometry_bytes - before.additional_geometry_bytes;
            assert!(typed > 0 && raw_reads == 0 && bytes >= 8, "source attribution at case {index}");
            if native { assert!(dispatches > 0); }
            managed.bus.rom[0x3ae174..0x3ae1a4].copy_from_slice(&rom[0x3ae174..0x3ae1a4]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary state and clock case {index}, native={native}");
            counts.push((index, native, typed, raw_reads, bytes, dispatches));
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"seven ordinary EUR call sites from one no-input continuation",
        "observation_frames":frames, "source_rom_poisoned":true, "one_frame_state_clock_pairs":counts,
        "per_read_native_attribution":"pending", "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E144 predecessor, ordinary checkpoint and translation; Wine only"]
fn european_earlier_context_geometry_publication_and_ordinary_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_TARGET_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TARGET_PREVIOUS").unwrap());
    let input = fs::read(std::env::var_os("FFTA_TARGET_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (144, 143));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (145, 144));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e145-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e145-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut current = parse_layered_obj_animations(&new_bytes).unwrap();
        assert_eq!(prior.context_additional_geometry.len(), 6);
        assert_eq!(current.context_additional_geometry.len(), 8);
        for (record, at) in current.context_additional_geometry.iter().take(2).zip([0x3ae154usize, 0x3ae15c]) {
            assert_eq!(record.source_offset, at as u32);
            assert_eq!(record.source_bytes(), rom[at..at + 8]);
        }
        current.context_additional_geometry.drain(..2);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.offset != 0x3ae154 && s.offset != 0x3ae15c);
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.layered_obj_animations_stats().unwrap();
    managed.bus.rom[0x3ae154..0x3ae164].fill(0xcd);
    for at in 0x3ae154..0x3ae164 { assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]); }
    assert_eq!(owner.layered_obj_animations_stats().unwrap().additional_geometry_bytes - before.additional_geometry_bytes, 16);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [0x3ae154usize, 0x3ae15c, 0x3ae164] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[0x3ae154..0x3ae164].copy_from_slice(&rom[0x3ae154..0x3ae164]);
    for (offset, helper, argument, destinations) in [
        (0x3ae154, 0x0814_31f4, 0, &[4, 0x14][..]),
        (0x3ae154, 0x0814_31f4, 1, &[4, 0x14][..]),
        (0x3ae15c, 0x0814_3238, 50, &[0x0c][..]),
    ] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let state = raw.save_state().unwrap();
        managed.load_state(&state).unwrap();
        managed.bus.rom[0x3ae154..0x3ae164].fill(0xcd);
        call_helper(&mut raw, helper, offset, argument);
        call_helper(&mut managed, helper, offset, argument);
        for &destination in destinations {
            for i in 0..8 {
                assert_eq!(managed.bus.read8(0x0202_0000 + destination + i), rom[offset as usize + i as usize]);
            }
        }
        managed.bus.rom[0x3ae154..0x3ae164].copy_from_slice(&rom[0x3ae154..0x3ae164]);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "helper state and clock");
    }
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    observer.bus.frame_ready = false;
    let mut frames = 0;
    let mut steps = 0;
    while !(observer.cpu.regs[15] == 0x0814_0c3c && observer.cpu.regs[1] == 0x083a_e15c) {
        observer.step();
        steps += 1;
        assert!(steps < 5_000_000, "ordinary target observation frame limit");
        if observer.bus.frame_ready { frames += 1; assert!(frames < 500, "ordinary target not reached"); steps = 0; observer.bus.frame_ready = false; }
    }
    assert_eq!(observer.cpu.regs[2], 50);
    let heap = u32::from_le_bytes(observer.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
    assert_eq!(observer.cpu.regs[0], heap + 0x8b8c);
    let checkpoint = observer.save_state().unwrap();
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&checkpoint).unwrap();
        managed.load_state(&checkpoint).unwrap();
        managed.bus.rom[0x3ae154..0x3ae164].fill(0xcd);
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        if native {
            let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
            crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
        } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
        assert!(raw.bus.frame_ready);
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(0x3ae154, 0x3ae164)] }, 10000).unwrap();
        capture.frame(Some(managed.bus.frames));
        let dispatches = if native {
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0
        } else { managed.bus.frame_ready = false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } 0 };
        assert!(managed.bus.frame_ready);
        let report = capture.finish();
        assert!(!report.truncated);
        let typed: u64 = report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum();
        let raw_reads: u64 = report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
        let bytes = owner.layered_obj_animations_stats().unwrap().additional_geometry_bytes - before.additional_geometry_bytes;
        assert!(typed > 0 && raw_reads == 0 && bytes >= 8);
        if native { assert!(dispatches > 0); }
        managed.bus.rom[0x3ae154..0x3ae164].copy_from_slice(&rom[0x3ae154..0x3ae164]);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary target state and clock, native={native}");
        results.push((native, typed, raw_reads, bytes, dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[145,144], "new_source_bytes":16, "e144_projection_preserved":true,
        "source_rom_poisoned":true, "controlled_helper_cases":3, "ordinary_target_call":true,
        "ordinary_target_prelude_frames":frames, "one_frame_state_clock_pairs":results,
        "ordinary_initial_source":"pending", "per_read_native_attribution":"pending", "frontend_ui":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR cold startup, E145 dataset and translation; Wine only"]
fn european_earlier_context_geometry_initial_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_TARGET_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TARGET_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (145, 144));
    let mut observer = crate::make_machine(rom.clone(), None);
    let calls = [(0x0814_0742, 0, 0x8b8c), (0x0814_0752, 1, 0x8bb0)];
    let mut checkpoints: Vec<Option<Vec<u8>>> = vec![None; 2];
    let mut reached_frame = 0u32;
    'frames: for frame in 0..1200u32 {
        observer.bus.keys = if (180..182).contains(&frame) { 0x3fe } else { 0x3ff };
        observer.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if observer.bus.frame_ready { break; }
            let pc = observer.cpu.regs[15];
            if let Some((index, &(_, selector, offset))) = calls.iter().enumerate().find(|(_, c)| c.0 == pc) {
                if checkpoints[index].is_none() {
                    assert_eq!(observer.cpu.regs[1], 0x083a_e154);
                    assert_eq!(observer.cpu.regs[2], selector);
                    let heap = u32::from_le_bytes(observer.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
                    assert_eq!(observer.cpu.regs[0], heap + offset);
                    checkpoints[index] = Some(observer.save_state().unwrap());
                    reached_frame = frame;
                }
            }
            observer.step();
            if checkpoints.iter().all(Option::is_some) { break 'frames; }
        }
        assert!(observer.bus.frame_ready, "ordinary initial observation frame limit");
    }
    assert!(checkpoints.iter().all(Option::is_some), "initial source calls not reached");
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for (index, checkpoint) in checkpoints.iter().enumerate() {
        for native in [false, true] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let mut managed = crate::make_machine(rom.clone(), None);
            attach_from_directory(&mut managed, &dataset).unwrap();
            raw.load_state(checkpoint.as_ref().unwrap()).unwrap();
            managed.load_state(checkpoint.as_ref().unwrap()).unwrap();
            managed.bus.rom[0x3ae154..0x3ae164].fill(0xcd);
            let owner = managed.bus.rom_resources().unwrap().clone();
            let before = owner.layered_obj_animations_stats().unwrap();
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(0x3ae154, 0x3ae15c)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            let dispatches = if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0
            } else { managed.bus.frame_ready = false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } 0 };
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            let typed: u64 = report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum();
            let raw_reads: u64 = report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
            let bytes = owner.layered_obj_animations_stats().unwrap().additional_geometry_bytes - before.additional_geometry_bytes;
            assert!(typed > 0 && raw_reads == 0 && bytes >= 8, "ordinary initial source case {index}");
            if native { assert!(dispatches > 0); }
            managed.bus.rom[0x3ae154..0x3ae164].copy_from_slice(&rom[0x3ae154..0x3ae164]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary initial state and clock case {index}, native={native}");
            results.push((index, native, typed, raw_reads, bytes, dispatches));
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR cold startup with English selection, both initial-source calls",
        "last_call_frame":reached_frame, "source_rom_poisoned":true, "one_frame_state_clock_pairs":results,
        "per_read_native_attribution":"pending", "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
