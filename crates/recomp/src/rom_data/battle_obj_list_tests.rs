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

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "retained EUR dialogue continuation and exact ROM; Wine only"]
fn european_battle_draw_list_writer_observation() {
    use gba_core::debug_memory::{Capture, Watch};
    const SLOT: usize = 0x30fbc;
    const SOURCE: u32 = 0x0840_dcba;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_ROOT").unwrap());
    let before = fs::read(std::env::var_os("FFTA_BATTLE_WRITER_BEFORE").unwrap()).unwrap();
    let after = fs::read(std::env::var_os("FFTA_BATTLE_WRITER_AFTER").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&before), "e6c84afbc6f732cb6636f6ad28ad49303b3af8ceb450c054884fed55b403f3e1");
    assert_eq!(gba_core::ffta_rom::digest(&after), "a4f2e3476dbbf95a4ca8d8b39ad3f35bea3d4af896f438e130f1282f9943fb67");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    machine.load_state(&before).unwrap();
    let slot = |m: &gba_core::Machine| u32::from_le_bytes(m.bus.ewram[SLOT..SLOT + 4].try_into().unwrap());
    assert_ne!(slot(&machine), SOURCE);
    let mut endpoint = crate::make_machine(rom, None);
    endpoint.load_state(&after).unwrap();
    assert_eq!(slot(&endpoint), SOURCE);
    drop(endpoint);
    let capture = Capture::start(Watch { start: 0x0203_0fbc, end: 0x0203_0fc0, write: true, width: None, value: None }).unwrap();
    let mut writes = Vec::new();
    let mut resolver_calls = Vec::new();
    let mut candidate_resolver = None;
    let mut candidate_writer = None;
    let mut found = false;
    let mut completed_frame = 0u32;
    'frames: for frame in 0..120u32 {
        machine.bus.keys = 0x3ff;
        machine.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if machine.bus.frame_ready { break; }
            let pc = machine.cpu.regs[15];
            let regs = machine.cpu.regs;
            let old = slot(&machine);
            let resolver = pc == 0x0800_720c && regs[0] == 0x0840_d300 && regs[1] == 62;
            let writer = pc == 0x0802_e110 && regs[6] == 0x0203_0f98 && regs[0] == SOURCE;
            let snapshot = if resolver || writer { Some(machine.save_state().unwrap()) } else { None };
            let event = machine.step();
            let executed = matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc);
            if executed && resolver {
                resolver_calls.push(serde_json::json!({"frame":frame,"pc":format!("{pc:08x}"),"directory":format!("{:08x}", regs[0]),"selector":regs[1]}));
                candidate_resolver = snapshot.clone();
            }
            if let Some(access) = capture.take_hit() {
                let new = slot(&machine);
                writes.push(serde_json::json!({"frame":frame,"pc":format!("{pc:08x}"),"executed_instruction":executed,
                    "address":format!("{:08x}",access.address),"width":access.width,"value":format!("{:08x}",access.value),
                    "slot_before":format!("{old:08x}"),"slot_after":format!("{new:08x}"),
                    "r0":format!("{:08x}",regs[0]),"r6":format!("{:08x}",regs[6])}));
                assert!(writes.len() <= 128, "too many slot write events");
                if new == SOURCE {
                    if executed && writer { candidate_writer = snapshot; }
                    found = true;
                    completed_frame = frame;
                    break 'frames;
                }
            }
        }
        assert!(machine.bus.frame_ready, "writer observation frame limit at {frame}");
    }
    drop(capture);
    assert!(found, "target slot value was not observed within the retained continuation");
    let checkpoints = root.join("checkpoints");
    fs::create_dir_all(&checkpoints).unwrap();
    if let Some(state) = &candidate_resolver { fs::write(checkpoints.join("pre-resolver.ffstate"), state).unwrap(); }
    if let Some(state) = &candidate_writer { fs::write(checkpoints.join("pre-writer.ffstate"), state).unwrap(); }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR retained stage-5 continuation, neutral input until first target slot value",
        "writer_frame":completed_frame, "writes":writes, "matching_resolver_calls":resolver_calls,
        "candidate_resolver_checkpoint":candidate_resolver.is_some(), "candidate_writer_checkpoint":candidate_writer.is_some(),
        "watch_limit":"first overlapping bus access per step; pre-step PC can include nested work",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E148 predecessor, ordinary resolver checkpoint and translation; Wine only"]
fn european_battle_directory_publication_and_ordinary_pairs() {
    use gba_core::rom_data::{obj_lists::{BATTLE_DIRECTORY_COUNT, BATTLE_DIRECTORY_ROOT}, read_audit::{Service, TraceCapture, TraceFilter}};
    const RETURN: u32 = 0x0800_0300;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_DIRECTORY_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_DIRECTORY_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_BATTLE_DIRECTORY_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "4ed38457275308dce29a3e21d14d391015fc4bda1702228cfbda24fad3cee367");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (148, 147));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (149, 148));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e149-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e149-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    let at = BATTLE_DIRECTORY_ROOT as usize;
    let len = 4 + 2 * BATTLE_DIRECTORY_COUNT;
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old_resource.sha256);
        if old_resource.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        assert!(prior.battle_directory.is_none());
        let directory = current.battle_directory.take().unwrap();
        assert_eq!(directory.source_bytes().unwrap(), rom[at..at + len]);
        assert_eq!(directory.offsets.len(), 103);
        assert_eq!(at as u32 + 4 + u32::from(directory.offsets[62]), 0x40dcba);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "battle-relative-directory");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.obj_list_stats().unwrap();
    managed.bus.rom[at..at + len].fill(0xcd);
    for i in 0..len { assert_eq!(managed.bus.read8(0x0800_0000 + (at + i) as u32), rom[at + i]); }
    assert_eq!(owner.obj_list_stats().unwrap().directory_bytes - before.directory_bytes, len as u64);
    let ptr = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
    for edge in [at, at + len] {
        for delta in [-3isize, -1, 0, 1] {
            let index = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + index as u32), u32::from(u16::from_le_bytes(rom[index..index + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + index as u32), u32::from_le_bytes(rom[index..index + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    for selector in [0u32, 62, 102, 103, 65535, 0x1_003e] {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        for machine in [&mut raw, &mut managed] {
            machine.cpu.regs[0] = 0x0800_0000 + BATTLE_DIRECTORY_ROOT;
            machine.cpu.regs[1] = selector;
            machine.cpu.regs[13] = 0x0300_7c00;
            machine.cpu.regs[14] = RETURN | 1;
            machine.cpu.regs[15] = 0x0800_720c;
            machine.cpu.cpsr = 0xbf;
        }
        managed.bus.rom[at..at + len].fill(0xcd);
        for machine in [&mut raw, &mut managed] {
            for _ in 0..200 { if machine.cpu.regs[15] == RETURN { break; } machine.step(); }
            assert_eq!(machine.cpu.regs[15], RETURN, "directory reader did not return");
            let index = selector as u16 as usize;
            let expected = if index < BATTLE_DIRECTORY_COUNT {
                0x0800_0000 + BATTLE_DIRECTORY_ROOT + 4 + u32::from(u16::from_le_bytes(rom[at + 4 + 2 * index..at + 6 + 2 * index].try_into().unwrap()))
            } else { 0 };
            assert_eq!(machine.cpu.regs[0], expected, "directory selector {selector:x}");
        }
        managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "directory state/clock selector {selector:x}");
    }
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        managed.bus.rom[at..at + len].fill(0xcd);
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.obj_list_stats().unwrap();
        if native {
            let ptr = &mut raw as *mut gba_core::Machine as *mut std::ffi::c_void;
            crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
        } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
        assert!(raw.bus.frame_ready);
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(BATTLE_DIRECTORY_ROOT, BATTLE_DIRECTORY_ROOT + len as u32)] }, 10000).unwrap();
        capture.frame(Some(managed.bus.frames));
        let dispatches = if native {
            let ptr = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
            crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0
        } else { managed.bus.frame_ready = false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } 0 };
        assert!(managed.bus.frame_ready);
        let report = capture.finish();
        assert!(!report.truncated);
        let typed: u64 = report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum();
        let raw_reads: u64 = report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
        let bytes = owner.obj_list_stats().unwrap().directory_bytes - before.directory_bytes;
        assert!(typed > 0 && raw_reads == 0 && bytes >= 5);
        if native { assert!(dispatches > 0); }
        assert_eq!(u32::from_le_bytes(raw.bus.ewram[0x30fbc..0x30fc0].try_into().unwrap()), 0x0840_dcba);
        managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary directory state/clock native={native}");
        results.push((typed, raw_reads, bytes, dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[149,148], "directory_bytes":len, "entries":BATTLE_DIRECTORY_COUNT,
        "e148_projection_preserved":true, "controlled_original_selector_cases":6,
        "source_rom_poisoned":true, "ordinary_selector":62, "one_frame_state_clock_pairs":results,
        "other_selectors_ordinary":"pending", "per_read_native_attribution":"pending", "frontend_ui":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
