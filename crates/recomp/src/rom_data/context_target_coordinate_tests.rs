//! EUR context target rectangle and its original interpolation setup.
use super::*;
use gba_core::{rom_data::layered_obj_animations::CONTEXT_TARGET_COORDINATE, Bus, Machine};

const CONTEXT: u32 = 0x0202_0000;
const RETURN: u32 = 0x0800_0300;
const STACK: u32 = 0x0300_7c00;

fn original_target(machine: &mut Machine) {
    machine.cpu.regs[0] = CONTEXT;
    machine.cpu.regs[1] = 0x0800_0000 + CONTEXT_TARGET_COORDINATE;
    machine.cpu.regs[2] = 180;
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0814_3238;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..1000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN { return; }
    }
    panic!("original target-coordinate helper did not return");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E142 predecessor, ordinary state and translation; Wine only"]
fn european_context_target_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_TARGET_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TARGET_PREVIOUS").unwrap());
    let input = fs::read(std::env::var_os("FFTA_TARGET_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (142, 141));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (143, 142));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e143-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e143-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
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
        assert!(prior.context_target_coordinate.is_none());
        let target = current.context_target_coordinate.take().unwrap();
        assert_eq!(target.source_offset, CONTEXT_TARGET_COORDINATE);
        assert_eq!(target.source_bytes(), rom[CONTEXT_TARGET_COORDINATE as usize..CONTEXT_TARGET_COORDINATE as usize + 8]);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "context-target-coordinate");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let at = CONTEXT_TARGET_COORDINATE as usize;
    let before = owner.layered_obj_animations_stats().unwrap();
    managed.bus.rom[at..at + 8].fill(0xcd);
    for index in 0..8 { assert_eq!(managed.bus.read8(0x0800_0000 + CONTEXT_TARGET_COORDINATE + index as u32), rom[at + index]); }
    assert_eq!(owner.layered_obj_animations_stats().unwrap().target_coordinate_bytes - before.target_coordinate_bytes, 8);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [at, at + 8] {
        for delta in [-3isize, -1, 0, 1] {
            let index = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + index as u32), u32::from(u16::from_le_bytes(rom[index..index + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + index as u32), u32::from_le_bytes(rom[index..index + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[at..at + 8].copy_from_slice(&rom[at..at + 8]);
    let initial = raw.save_state().unwrap();
    managed.load_state(&initial).unwrap();
    managed.bus.rom[at..at + 8].fill(0xcd);
    original_target(&mut raw);
    original_target(&mut managed);
    for index in 0..8 {
        assert_eq!(raw.bus.read8(CONTEXT + 0x0c + index as u32), rom[at + index]);
        assert_eq!(managed.bus.read8(CONTEXT + 0x0c + index as u32), rom[at + index]);
    }
    assert_eq!(raw.bus.read8(CONTEXT), 2);
    assert_eq!(raw.bus.read32(CONTEXT + 0x1c), 0);
    assert_eq!(raw.bus.read32(CONTEXT + 0x20), 180);
    managed.bus.rom[at..at + 8].copy_from_slice(&rom[at..at + 8]);
    assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "original target helper state and clock");

    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    observer.bus.frame_ready = false;
    let mut frames = 0u32;
    let mut steps_in_frame = 0u32;
    let mut checkpoint = None;
    let mut context = 0u32;
    let mut copy_arguments = false;
    let mut copied = false;
    while frames < 40 && !copied {
        let pc = observer.cpu.regs[15];
        if pc == 0x0814_0f12 && observer.cpu.regs[1] == 0x0800_0000 + CONTEXT_TARGET_COORDINATE {
            assert_eq!(observer.cpu.regs[2], 180);
            context = observer.cpu.regs[0];
            let heap = u32::from_le_bytes(observer.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
            assert_eq!(context, heap + 0x8b8c);
            checkpoint = Some(observer.save_state().unwrap());
        }
        let copy = checkpoint.is_some() && pc == 0x0814_5230 && observer.cpu.regs[0] == 0x0800_0000 + CONTEXT_TARGET_COORDINATE
            && observer.cpu.regs[1] == context + 0x0c && observer.cpu.regs[2] == 4;
        let event = observer.step();
        if matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) {
            if copy { copy_arguments = true; }
            if pc == 0x0814_0f16 && copy_arguments {
                for index in 0..8 { assert_eq!(observer.bus.read8(context + 0x0c + index as u32), rom[at + index]); }
                assert_eq!(observer.bus.read8(context), 2);
                assert_eq!(observer.bus.read32(context + 0x1c), 0);
                assert_eq!(observer.bus.read32(context + 0x20), 180);
                copied = true;
            }
        }
        steps_in_frame += 1;
        assert!(steps_in_frame < 5_000_000, "ordinary target observation frame limit");
        if observer.bus.frame_ready { frames += 1; steps_in_frame = 0; observer.bus.frame_ready = false; }
    }
    assert!(copy_arguments && copied, "ordinary target copy was not observed within 40 frames");
    let state = checkpoint.expect("ordinary target helper checkpoint");
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    let mut interpolation_seen = false;
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
            managed.bus.rom[at..at + 8].fill(0xcd);
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    if raw.bus.frame_ready { break; }
                    let pc = raw.cpu.regs[15];
                    let event = raw.step();
                    if matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) && pc == 0x0814_21ee { interpolation_seen = true; }
                }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_TARGET_COORDINATE, CONTEXT_TARGET_COORDINATE + 8)] }, 10000).unwrap();
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
            managed.bus.rom[at..at + 8].copy_from_slice(&rom[at..at + 8]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary target state and clock: native={native}, frame={frame}");
        }
        let bytes = owner.layered_obj_animations_stats().unwrap().target_coordinate_bytes - before.target_coordinate_bytes;
        assert!(bytes >= 8 && typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, bytes, native_dispatches));
    }
    assert!(interpolation_seen, "ordinary first interpolation update was not observed");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[143,142], "source_bytes":8, "prior_resources_preserved":true,
        "controlled_original_helper":true, "ordinary_copy_arguments_destination_and_state":true,
        "ordinary_interpolation_update_observed":true, "frames_to_copy":frames, "source_rom_poisoned":true,
        "ordinary_interpreter_and_native_dispatch_pairs":counts, "full_state_clock_frames_each":3,
        "other_consumers":"pending", "per_read_native_attribution":"pending", "frontend_ui":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR checkpoint, E143 dataset and translation; Wine only"]
fn european_context_target_full_interpolation_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_TARGET_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TARGET_DATASET").unwrap());
    let input = fs::read(std::env::var_os("FFTA_TARGET_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (143, 142));
    let at = CONTEXT_TARGET_COORDINATE as usize;
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    observer.bus.frame_ready = false;
    let mut prelude_frames = 0u32;
    let mut steps_in_frame = 0u32;
    while !(observer.cpu.regs[15] == 0x0814_0f12
        && observer.cpu.regs[1] == 0x0800_0000 + CONTEXT_TARGET_COORDINATE) {
        observer.step();
        steps_in_frame += 1;
        assert!(steps_in_frame < 5_000_000, "ordinary target prelude frame limit");
        if observer.bus.frame_ready {
            prelude_frames += 1;
            assert!(prelude_frames < 40, "ordinary target call not reached");
            steps_in_frame = 0;
            observer.bus.frame_ready = false;
        }
    }
    assert_eq!(observer.cpu.regs[2], 180);
    let context = observer.cpu.regs[0];
    let heap = u32::from_le_bytes(observer.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
    assert_eq!(context, heap + 0x8b8c);
    let context_offset = context.checked_sub(0x0200_0000).unwrap() as usize;
    let state = observer.save_state().unwrap();
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut native_dispatches = 0u64;
        let mut held_frames = 0u32;
        let mut matched_frames = 0u32;
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        for frame in 0..190 {
            managed.bus.rom[at..at + 8].fill(0xcd);
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_TARGET_COORDINATE, CONTEXT_TARGET_COORDINATE + 8)] }, 10000).unwrap();
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
            assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer, "framebuffer at {frame}");
            assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer, "wide framebuffer at {frame}");
            assert_eq!(raw.bus.oam, managed.bus.oam, "OAM at {frame}");
            managed.bus.rom[at..at + 8].copy_from_slice(&rom[at..at + 8]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary full interpolation state and clock: native={native}, frame={frame}");
            matched_frames += 1;
            let context_bytes = &raw.bus.ewram[context_offset..context_offset + 0x24];
            assert_eq!(context_bytes[0], 2, "target interpolation state changed before saturation");
            assert_eq!(&context_bytes[0x0c..0x14], &rom[at..at + 8], "target geometry changed");
            let elapsed = u32::from_le_bytes(context_bytes[0x1c..0x20].try_into().unwrap());
            let duration = u32::from_le_bytes(context_bytes[0x20..0x24].try_into().unwrap());
            assert_eq!(duration, 180);
            if elapsed == 181 {
                assert_eq!(&context_bytes[0x14..0x1c], &rom[at..at + 8], "saturated geometry differs from target");
                held_frames += 1;
                if held_frames == 2 { break; }
            } else {
                assert!(elapsed <= 180 && held_frames == 0, "unexpected elapsed value or lost hold");
            }
        }
        let bytes = owner.layered_obj_animations_stats().unwrap().target_coordinate_bytes - before.target_coordinate_bytes;
        assert_eq!(held_frames, 2, "target did not saturate and hold within 190 frames");
        assert!(bytes >= 8 && typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        results.push((matched_frames, typed, raw_reads, bytes, native_dispatches));
    }
    assert_eq!(results[0].0, results[1].0, "interpreter and native-dispatch completion frames");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"ordinary no-input EUR target interpolation through saturation and hold",
        "prelude_frames":prelude_frames, "duration":180, "saturated_elapsed":181, "held_frames":2,
        "source_rom_poisoned":true, "ordinary_interpreter_and_native_dispatch_pairs":results,
        "full_state_clock_framebuffer_oam_each_frame":true, "audio":"not checked",
        "other_consumers":"pending", "per_read_native_attribution":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
