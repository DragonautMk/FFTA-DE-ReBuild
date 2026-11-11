//! EUR context activation ticks and conditional gates.
use super::*;
use gba_core::{rom_data::layered_obj_animations::{CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS}, Bus, Machine};

const CONTEXT: u32 = 0x0201_0000;

fn original_loop(machine: &mut Machine, tick: u32, mode: u8) {
    machine.bus.write32(0x0200_f3b0, CONTEXT);
    machine.bus.write32(CONTEXT + 0x8fd8, tick);
    machine.bus.write8(CONTEXT + 0x9234, mode);
    for index in 0..40 { machine.bus.write8(CONTEXT + 0x91d0 + index, 0); }
    machine.cpu.regs[15] = 0x0814_0e6c;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..4000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == 0x0814_0eac { return; }
    }
    panic!("original context activation loop did not finish");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E141 predecessor, ordinary state and translation; Wine only"]
fn european_context_activation_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_ACTIVATION_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_ACTIVATION_PREVIOUS").unwrap());
    let input = fs::read(std::env::var_os("FFTA_ACTIVATION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (141, 140));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (142, 141));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e142-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e142-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    let mut ticks = Vec::new();
    let mut gates = Vec::new();
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
        assert!(prior.context_activation.is_none());
        let activation = current.context_activation.take().unwrap();
        assert_eq!((activation.ticks.len(), activation.gates.len()), (40, 40));
        ticks = activation.ticks;
        gates = activation.gates;
        for index in 0..40 {
            let at = CONTEXT_ACTIVATION_TICKS as usize + 4 * index;
            assert_eq!(ticks[index].to_le_bytes(), rom[at..at + 4]);
            assert_eq!(gates[index], rom[CONTEXT_ACTIVATION_GATES as usize + index]);
        }
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "context-activation-ticks" && s.role != "context-activation-gates");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    assert_eq!((ticks.len(), gates.len()), (40, 40));
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let poison = |machine: &mut Machine| {
        for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { machine.bus.rom[at as usize] = 0xcd; }
    };
    let restore = |machine: &mut Machine| {
        for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { machine.bus.rom[at as usize] = rom[at as usize]; }
    };
    let before = owner.layered_obj_animations_stats().unwrap();
    poison(&mut managed);
    for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 {
        assert_eq!(managed.bus.read8(0x0800_0000 + at), rom[at as usize]);
    }
    let after = owner.layered_obj_animations_stats().unwrap();
    assert_eq!(after.activation_gate_bytes - before.activation_gate_bytes, 40);
    assert_eq!(after.activation_tick_bytes - before.activation_tick_bytes, 160);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS, CONTEXT_ACTIVATION_TICKS + 160] {
        for delta in [-3i32, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta) as usize;
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    restore(&mut managed);
    let initial = raw.save_state().unwrap();
    for (query, mode) in [(ticks[0], 0u8), (ticks[0], 1), (ticks[39], 0), (ticks[39], 1), (u32::MAX, 1)] {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        poison(&mut managed);
        let before = owner.layered_obj_animations_stats().unwrap();
        original_loop(&mut raw, query, mode);
        original_loop(&mut managed, query, mode);
        let after = owner.layered_obj_animations_stats().unwrap();
        assert_eq!(after.activation_tick_bytes - before.activation_tick_bytes, 160);
        if mode != 0 && ticks.contains(&query) { assert!(after.activation_gate_bytes > before.activation_gate_bytes); }
        for index in 0..40 {
            let expected = u8::from(ticks[index] == query && (mode == 0 || gates[index] == 0));
            assert_eq!(raw.bus.read8(CONTEXT + 0x91d0 + index as u32), expected);
            assert_eq!(managed.bus.read8(CONTEXT + 0x91d0 + index as u32), expected);
        }
        restore(&mut managed);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "original loop state and clock, mode={mode}");
    }

    // The retained ordinary checkpoint advances without input to the first due tick.
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    observer.bus.frame_ready = false;
    let mut frames = 0u32;
    let mut steps_in_frame = 0u32;
    let mut pre_activation = None;
    let mut set_flag = false;
    let mut read_flag = false;
    let mut cleared_flag = false;
    while frames < 40 && !cleared_flag {
        let pc = observer.cpu.regs[15];
        let heap = u32::from_le_bytes(observer.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
        let heap_offset = heap.checked_sub(0x0200_0000).map(|v| v as usize);
        let counter = heap_offset.and_then(|at| observer.bus.ewram.get(at + 0x8fd8..at + 0x8fdc))
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()));
        if pc == 0x0814_0e6c && counter == Some(ticks[0]) && pre_activation.is_none() {
            pre_activation = Some(observer.save_state().unwrap());
        }
        let flag_index = heap_offset.and_then(|at| at.checked_add(0x91d0));
        let before_flag = flag_index.and_then(|at| observer.bus.ewram.get(at)).copied();
        let first_store = pc == 0x0814_0ea0 && observer.cpu.regs[6] == 0 && counter == Some(ticks[0]);
        let event = observer.step();
        if matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) {
            let after_flag = flag_index.and_then(|at| observer.bus.ewram.get(at)).copied();
            if first_store && before_flag == Some(0) && after_flag == Some(1) { set_flag = true; }
            if pc == 0x0814_0eb4 && set_flag && observer.cpu.regs[0] == 1 { read_flag = true; }
            if pc == 0x0814_0eee && read_flag && before_flag == Some(1) && after_flag == Some(0) { cleared_flag = true; }
        }
        steps_in_frame += 1;
        assert!(steps_in_frame < 5_000_000, "ordinary activation frame limit");
        if observer.bus.frame_ready { frames += 1; steps_in_frame = 0; observer.bus.frame_ready = false; }
    }
    assert!(set_flag && read_flag && cleared_flag, "ordinary first activation was not observed within 40 frames");
    let state = pre_activation.expect("ordinary pre-activation checkpoint");
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        let mut native_dispatches = 0u64;
        for frame in 0..3 {
            poison(&mut managed);
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS + 160)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            for event in report.events {
                match event.key.service {
                    Service::Typed => typed += event.requests as usize,
                    Service::Raw => raw_reads += event.requests as usize,
                    _ => {}
                }
            }
            restore(&mut managed);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary activation state and clock: native={native}, frame={frame}");
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        let tick_bytes = after.activation_tick_bytes - before.activation_tick_bytes;
        let gate_bytes = after.activation_gate_bytes - before.activation_gate_bytes;
        assert!(tick_bytes >= 160 && typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, tick_bytes, gate_bytes, native_dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[142,141], "source_bytes":200, "prior_resources_preserved":true,
        "original_loop_cases":5, "controlled_gate_case":true, "source_rom_poisoned":true,
        "ordinary_first_flag_set_read_cleared":true, "frames_to_first_activation":frames,
        "ordinary_interpreter_and_native_dispatch_pairs":counts, "full_state_clock_frames_each":3,
        "ordinary_gate_use":"pending", "other_activations":"pending", "per_read_native_attribution":"pending",
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR checkpoint, E142 dataset and translation; Wine only"]
fn european_context_activation_ordinary_gated_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::{BTreeMap, BTreeSet};

    let root = PathBuf::from(std::env::var_os("FFTA_ACTIVATION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_ACTIVATION_DATASET").unwrap());
    let input = fs::read(std::env::var_os("FFTA_ACTIVATION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (142, 141));
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    let read_context = |machine: &Machine| -> Option<(usize, u32, u8)> {
        let heap = u32::from_le_bytes(machine.bus.ewram.get(0xf3b0..0xf3b4)?.try_into().ok()?);
        let offset = heap.checked_sub(0x0200_0000)? as usize;
        let counter = u32::from_le_bytes(machine.bus.ewram.get(offset + 0x8fd8..offset + 0x8fdc)?.try_into().ok()?);
        let mode = *machine.bus.ewram.get(offset + 0x9234)?;
        Some((offset, counter, mode))
    };
    let mut pressed_at = None;
    let mut mode_write_counter = None;
    let mut pre_gated = None;
    let mut gates = BTreeMap::<usize, u8>::new();
    let mut stores = BTreeSet::<usize>::new();
    let mut completed_frame = 0u32;
    let mut loop_complete = false;
    'observe: for frame in 0..40u32 {
        let (_, counter, _) = read_context(&observer).expect("live context");
        if pressed_at.is_none() && counter >= 744 { pressed_at = Some(frame); }
        observer.bus.keys = if pressed_at.is_some_and(|start| frame < start + 2) { 0x3f7 } else { 0x3ff };
        observer.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if observer.bus.frame_ready { break; }
            let pc = observer.cpu.regs[15];
            let (heap, counter, mode) = read_context(&observer).expect("live context");
            if pc == 0x0814_0e6c && counter == 750 && mode != 0 && pre_gated.is_none() {
                pre_gated = Some(observer.save_state().unwrap());
            }
            let index = observer.cpu.regs[6] as usize;
            let event = observer.step();
            if !matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) { continue; }
            if pc == 0x0814_19a4 && observer.bus.ewram[heap + 0x9234] != 0 { mode_write_counter = Some(counter); }
            if counter == 750 && mode != 0 && pc == 0x0814_0e92 {
                assert!(index < 40);
                let gate = observer.cpu.regs[0] as u8;
                assert_eq!(gate, rom[CONTEXT_ACTIVATION_GATES as usize + index]);
                gates.insert(index, gate);
            }
            if counter == 750 && mode != 0 && pc == 0x0814_0ea0 { stores.insert(index); }
            if counter == 750 && mode != 0 && pc == 0x0814_0eac && !gates.is_empty() {
                loop_complete = true;
                completed_frame = frame;
                break 'observe;
            }
        }
        assert!(observer.bus.frame_ready, "ordinary gated observation frame limit at {frame}");
    }
    assert!(pressed_at.is_some(), "Start was not applied");
    let written_at = mode_write_counter.expect("original Start handler did not set mode");
    assert!((743..=750).contains(&written_at), "mode write fell outside the intended tick window");
    assert!(loop_complete, "ordinary gated equality was not reached within 40 frames");
    let permitted = gates.values().filter(|&&gate| gate == 0).count();
    let suppressed = gates.values().filter(|&&gate| gate != 0).count();
    assert!(permitted > 0 && suppressed > 0, "ordinary gate did not cover both outcomes");
    for (&index, &gate) in &gates { assert_eq!(stores.contains(&index), gate == 0, "ordinary gate outcome at {index}"); }
    let state = pre_gated.expect("ordinary pre-gated checkpoint");
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        let mut native_dispatches = 0u64;
        for frame in 0..3 {
            for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { managed.bus.rom[at as usize] = 0xcd; }
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS + 160)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            for event in report.events {
                match event.key.service {
                    Service::Typed => typed += event.requests as usize,
                    Service::Raw => raw_reads += event.requests as usize,
                    _ => {}
                }
            }
            for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { managed.bus.rom[at as usize] = rom[at as usize]; }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary gated state and clock: native={native}, frame={frame}");
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        let tick_bytes = after.activation_tick_bytes - before.activation_tick_bytes;
        let gate_bytes = after.activation_gate_bytes - before.activation_gate_bytes;
        assert!(tick_bytes >= 160 && gate_bytes > 0 && typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, tick_bytes, gate_bytes, native_dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"ordinary EUR checkpoint, brief Start press and release",
        "start_frame":pressed_at, "mode_write_counter":written_at, "gated_frame":completed_frame,
        "ordinary_gate_reads":gates.len(), "permitted":permitted, "suppressed":suppressed,
        "source_rom_poisoned":true, "ordinary_interpreter_and_native_dispatch_pairs":counts,
        "full_state_clock_frames_each":3, "other_activations":"pending", "per_read_native_attribution":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR checkpoint, E142 dataset and translation; Wine only"]
fn european_context_activation_all_ordinary_flags() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_ACTIVATION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_ACTIVATION_DATASET").unwrap());
    let input = fs::read(std::env::var_os("FFTA_ACTIVATION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (142, 141));
    let ticks: Vec<u32> = (0..40).map(|index| {
        let at = CONTEXT_ACTIVATION_TICKS as usize + 4 * index;
        u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
    }).collect();
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    let read_context = |machine: &Machine| -> Option<(usize, u32, u8)> {
        let heap = u32::from_le_bytes(machine.bus.ewram.get(0xf3b0..0xf3b4)?.try_into().ok()?);
        let offset = heap.checked_sub(0x0200_0000)? as usize;
        let counter = u32::from_le_bytes(machine.bus.ewram.get(offset + 0x8fd8..offset + 0x8fdc)?.try_into().ok()?);
        let mode = *machine.bus.ewram.get(offset + 0x9234)?;
        Some((offset, counter, mode))
    };
    let mut mask = 0u64;
    let mut pre_last = None;
    let mut terminal_set = false;
    let mut flag39_read = false;
    let mut flag39_cleared = false;
    let mut completed_frame = 0u32;
    'observe: for frame in 0..1800u32 {
        observer.bus.keys = 0x3ff;
        observer.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if observer.bus.frame_ready { break; }
            let pc = observer.cpu.regs[15];
            if !matches!(pc, 0x0814_0e6c | 0x0814_0ea0 | 0x0814_1832 | 0x0814_183c | 0x0814_1844) {
                observer.step();
                continue;
            }
            let (heap, counter, mode) = read_context(&observer).expect("live context");
            assert_eq!(mode, 0, "no-input route changed the activation mode");
            if pc == 0x0814_0e6c && counter == ticks[39] && pre_last.is_none() {
                pre_last = Some(observer.save_state().unwrap());
            }
            let index = observer.cpu.regs[6] as usize;
            let terminal_address = observer.cpu.regs[2];
            let event = observer.step();
            if !matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) { continue; }
            if pc == 0x0814_0ea0 {
                assert!(index < 40);
                assert_eq!(counter, ticks[index], "ordinary activation tick at index {index}");
                assert_eq!(observer.bus.ewram[heap + 0x91d0 + index], 1);
                mask |= 1u64 << index;
            }
            if pc == 0x0814_1832 && observer.cpu.regs[0] == 1 { flag39_read = true; }
            if pc == 0x0814_183c && flag39_read {
                assert_eq!(observer.bus.read8(terminal_address), 0xff);
                terminal_set = true;
            }
            if pc == 0x0814_1844 && terminal_set && observer.bus.ewram[heap + 0x91d0 + 39] == 0 {
                flag39_cleared = true;
                completed_frame = frame;
                break 'observe;
            }
        }
        assert!(observer.bus.frame_ready, "ordinary activation frame limit at {frame}");
    }
    assert_eq!(mask, (1u64 << 40) - 1, "ordinary activation indices");
    assert!(flag39_read && terminal_set && flag39_cleared, "ordinary terminal sequence was not observed");
    let state = pre_last.expect("ordinary last-activation checkpoint");
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        let mut native_dispatches = 0u64;
        for frame in 0..3 {
            for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { managed.bus.rom[at as usize] = 0xcd; }
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS + 160)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            for event in report.events {
                match event.key.service {
                    Service::Typed => typed += event.requests as usize,
                    Service::Raw => raw_reads += event.requests as usize,
                    _ => {}
                }
            }
            for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { managed.bus.rom[at as usize] = rom[at as usize]; }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary terminal state and clock: native={native}, frame={frame}");
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        let tick_bytes = after.activation_tick_bytes - before.activation_tick_bytes;
        let gate_bytes = after.activation_gate_bytes - before.activation_gate_bytes;
        assert!(tick_bytes >= 160 && typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, tick_bytes, gate_bytes, native_dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"ordinary no-input EUR activation continuation",
        "activation_mask":format!("{mask:010x}"), "terminal_flag_read_state_set_and_clear":true,
        "terminal_frame":completed_frame, "source_rom_poisoned":true,
        "last_event_interpreter_and_native_dispatch_pairs":counts, "full_state_clock_frames_each":3,
        "other_scenes":"pending", "per_read_native_attribution":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
