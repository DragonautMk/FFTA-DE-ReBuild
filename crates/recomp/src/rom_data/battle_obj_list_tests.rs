//! EUR battle constructor OBJ list publication and ordinary source use.
use super::*;
use gba_core::Bus;

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E145 predecessor, retained battle state and translation; Wine only"]
fn european_battle_obj_list_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const ROOT: usize = 0x395088;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_BATTLE_OBJ_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "097d7ccc5a9ee0e5315a91b9e574c7d34f22ebd96813ec98b5a24f61e06c11fe");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (145, 144));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (146, 145));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e146-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e146-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        let list = current.lists.iter().find(|l| l.source_offset == ROOT as u32).unwrap();
        assert_eq!(list.objects.len(), 2);
        assert_eq!(list.source_bytes().unwrap(), rom[ROOT..ROOT + 14]);
        current.lists.retain(|list| list.source_offset != ROOT as u32);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.offset != ROOT as u32);
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut byte_machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut byte_machine, &prepared.directory).unwrap();
    let owner = byte_machine.bus.rom_resources().unwrap().clone();
    byte_machine.bus.rom[ROOT..ROOT + 14].fill(0xcd);
    let before = owner.obj_list_stats().unwrap().source_bytes;
    for i in 0..14 { assert_eq!(byte_machine.bus.read8(0x0800_0000 + (ROOT + i) as u32), rom[ROOT + i]); }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before, 14);
    let ptr = &mut byte_machine as *mut gba_core::Machine as *mut std::ffi::c_void;
    for edge in [ROOT, ROOT + 14] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.obj_list_stats().unwrap().source_bytes;
        let mut typed = 0u64;
        let mut raw_reads = 0u64;
        let mut dispatches = 0u64;
        for frame in 0..3 {
            managed.bus.rom[ROOT..ROOT + 14].fill(0xcd);
            if native {
                let ptr = &mut raw as *mut gba_core::Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(ROOT as u32, (ROOT + 14) as u32)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
                dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else { managed.bus.frame_ready = false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            typed += report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum::<u64>();
            raw_reads += report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum::<u64>();
            managed.bus.rom[ROOT..ROOT + 14].copy_from_slice(&rom[ROOT..ROOT + 14]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary battle state/clock native={native}, frame={frame}");
        }
        assert!(owner.obj_list_source_used(ROOT as u32).unwrap().0);
        assert!(typed > 0 && raw_reads == 0);
        if native { assert!(dispatches > 0); }
        counts.push((typed, raw_reads, owner.obj_list_stats().unwrap().source_bytes - before, dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[146,145], "new_lists":1, "new_source_bytes":14,
        "e145_projection_preserved":true, "source_rom_poisoned":true,
        "ordinary_interpreter_and_native_dispatch_pairs":counts, "full_state_clock_frames_each":3,
        "per_read_native_attribution":"pending", "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E146 predecessor, retained battle state and translation; Wine only"]
fn european_battle_draw_list_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const ROOT: usize = 0x40dcba;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_BATTLE_OBJ_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "097d7ccc5a9ee0e5315a91b9e574c7d34f22ebd96813ec98b5a24f61e06c11fe");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (146, 145));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (147, 146));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e147-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e147-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        let list = current.lists.iter().find(|l| l.source_offset == ROOT as u32).unwrap();
        assert_eq!(list.objects.len(), 8);
        assert_eq!(list.source_bytes().unwrap(), rom[ROOT..ROOT + 50]);
        current.lists.retain(|list| list.source_offset != ROOT as u32);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.offset != ROOT as u32);
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut byte_machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut byte_machine, &prepared.directory).unwrap();
    let owner = byte_machine.bus.rom_resources().unwrap().clone();
    byte_machine.bus.rom[ROOT..ROOT + 50].fill(0xcd);
    let before = owner.obj_list_stats().unwrap().source_bytes;
    for i in 0..50 { assert_eq!(byte_machine.bus.read8(0x0800_0000 + (ROOT + i) as u32), rom[ROOT + i]); }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before, 50);
    let ptr = &mut byte_machine as *mut gba_core::Machine as *mut std::ffi::c_void;
    for edge in [ROOT, ROOT + 50] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.obj_list_stats().unwrap().source_bytes;
        let mut typed = 0u64;
        let mut raw_reads = 0u64;
        let mut dispatches = 0u64;
        for frame in 0..3 {
            managed.bus.rom[ROOT..ROOT + 50].fill(0xcd);
            if native {
                let ptr = &mut raw as *mut gba_core::Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(ROOT as u32, (ROOT + 50) as u32)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
                dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else { managed.bus.frame_ready = false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            typed += report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum::<u64>();
            raw_reads += report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum::<u64>();
            managed.bus.rom[ROOT..ROOT + 50].copy_from_slice(&rom[ROOT..ROOT + 50]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary battle state/clock native={native}, frame={frame}");
        }
        assert!(owner.obj_list_source_used(ROOT as u32).unwrap().0);
        assert!(typed > 0 && raw_reads == 0);
        if native { assert!(dispatches > 0); }
        counts.push((typed, raw_reads, owner.obj_list_stats().unwrap().source_bytes - before, dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[147,146], "new_lists":1, "new_source_bytes":50,
        "e146_projection_preserved":true, "source_rom_poisoned":true,
        "ordinary_interpreter_and_native_dispatch_pairs":counts, "full_state_clock_frames_each":3,
        "per_read_native_attribution":"pending", "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "retained exact EUR battle state; Wine only"]
fn european_battle_draw_list_ordinary_caller() {
    use std::collections::VecDeque;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_ROOT").unwrap());
    let state = fs::read(std::env::var_os("FFTA_BATTLE_OBJ_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "097d7ccc5a9ee0e5315a91b9e574c7d34f22ebd96813ec98b5a24f61e06c11fe");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom, None);
    machine.load_state(&state).unwrap();
    machine.bus.frame_ready = false;
    let mut previous = VecDeque::new();
    let mut found = None;
    for _ in 0..5_000_000 {
        if machine.bus.frame_ready { break; }
        let pc = machine.cpu.regs[15];
        if pc == 0x0800_1500 && machine.cpu.regs[0] == 0x0840_dcba {
            let sp = machine.cpu.regs[13];
            let lr = machine.cpu.regs[14];
            let stack: Vec<String> = (0..16).map(|i| format!("{:08x}", machine.bus.read32(sp + i * 4))).collect();
            found = Some((format!("{lr:08x}"), format!("{sp:08x}"), stack, previous.into_iter().collect::<Vec<String>>()));
            break;
        }
        let event = machine.step();
        if matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) {
            if previous.len() == 160 { previous.pop_front(); }
            previous.push_back(format!("{pc:08x}"));
        }
    }
    let (lr, sp, stack, preceding_pcs) = found.expect("ordinary draw argument not reached in one frame");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR retained battle state, first original draw of 40dcba",
        "reader":"08001500", "source":"40dcba", "lr":lr, "sp":sp, "stack_words":stack,
        "preceding_executed_pcs":preceding_pcs, "pointer_producer":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
