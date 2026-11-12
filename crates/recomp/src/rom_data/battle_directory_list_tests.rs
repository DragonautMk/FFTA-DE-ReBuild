//! EUR conditional directory-backed OBJ lists and their original draw reader.
use super::*;
use gba_core::{Bus, Machine};
use std::collections::BTreeSet;

fn draw_original(machine: &mut Machine, source: u32) {
    const RETURN: u32 = 0x0800_0300;
    const STACK: u32 = 0x0300_7c00;
    const BUFFER: u32 = 0x0203_0010;
    machine.cpu.regs[..4].copy_from_slice(&[0x0800_0000 + source, 100, 70, 0]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0800_1500;
    machine.cpu.cpsr = 0xbf;
    for (i, value) in [7u32, 2, 3, BUFFER, 128, 0].into_iter().enumerate() {
        machine.bus.write32(STACK + (i as u32) * 4, value);
    }
    for offset in 0..128 * 8 { machine.bus.write8(BUFFER + offset, 0x5a); }
    for _ in 0..20_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN { assert_eq!(machine.cpu.regs[13], STACK); return; }
    }
    panic!("original OBJ draw did not return");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E149 predecessor, retained battle continuation and translation; Wine only"]
fn european_battle_directory_lists_publication_and_original_draws() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_LISTS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_LISTS_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_BATTLE_LISTS_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "e6c84afbc6f732cb6636f6ad28ad49303b3af8ceb450c054884fed55b403f3e1");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (149, 148));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (150, 149));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e150-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e150-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    let mut new_spans = Vec::<(u32, usize)>::new();
    let mut indexed_roots = Vec::new();
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
        let directory = current.battle_directory.as_ref().unwrap();
        assert_eq!(directory.offsets.len(), 103);
        indexed_roots = directory.offsets.iter().map(|&relative| directory.source_offset + 4 + u32::from(relative)).collect();
        let prior_roots: BTreeSet<u32> = prior.lists.iter().map(|list| list.source_offset).collect();
        for list in &current.lists {
            if !prior_roots.contains(&list.source_offset) {
                let bytes = list.source_bytes().unwrap();
                let at = list.source_offset as usize;
                assert_eq!(bytes, rom[at..at + bytes.len()]);
                new_spans.push((list.source_offset, bytes.len()));
            }
        }
        assert_eq!(new_spans.len(), 101);
        assert_eq!(new_spans.iter().map(|(_, len)| *len).sum::<usize>(), 3574);
        current.lists.retain(|list| prior_roots.contains(&list.source_offset));
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| old_resource.sources.iter().any(|prior| prior.offset == s.offset && prior.length == s.length && prior.role == s.role));
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    assert_eq!(indexed_roots.len(), 103);
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.obj_list_stats().unwrap();
    for &(at, len) in &new_spans { managed.bus.rom[at as usize..at as usize + len].fill(0xcd); }
    for &(at, len) in &new_spans {
        for i in 0..len { assert_eq!(managed.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
    }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before.source_bytes, 3574);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for index in [0usize, 31, 33, 61, 63, 102] {
        let at = indexed_roots[index] as usize;
        for delta in [-3isize, -1, 0, 1] {
            let offset = at.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + offset as u32), u32::from(u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + offset as u32), u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap()));
        }
    }
    // Index 84 straddles the newly enabled 40d/40e source-page boundary.
    for offset in [0x40dffdusize, 0x40dffe, 0x40dfff] {
        assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + offset as u32), u32::from(u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap())));
        assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + offset as u32), u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap()));
    }
    for &(at, len) in &new_spans { managed.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
    let initial = raw.save_state().unwrap();
    for index in [0usize, 31, 33, 61, 63, 102] {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        for &(at, len) in &new_spans { managed.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        draw_original(&mut raw, indexed_roots[index]);
        draw_original(&mut managed, indexed_roots[index]);
        for &(at, len) in &new_spans { managed.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "original draw state/clock index {index}");
    }
    let new_roots: BTreeSet<u32> = new_spans.iter().map(|(at, _)| *at).collect();
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&state).unwrap();
    let mut checkpoint = None;
    let mut selected = None;
    let mut observed_frame = 0u32;
    'observe: for frame in 0..120u32 {
        observer.bus.keys = 0x3ff;
        observer.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if observer.bus.frame_ready { break; }
            let pc = observer.cpu.regs[15];
            let argument = observer.cpu.regs[0];
            if pc == 0x0800_1500 && argument >= 0x0800_0000 && new_roots.contains(&(argument - 0x0800_0000)) {
                checkpoint = Some(observer.save_state().unwrap());
                selected = Some(argument - 0x0800_0000);
                observed_frame = frame;
                break 'observe;
            }
            observer.step();
        }
        assert!(observer.bus.frame_ready, "ordinary battle list observation frame limit");
    }
    let checkpoint_dir = root.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let mut ordinary = Vec::new();
    if let (Some(state), Some(selected)) = (&checkpoint, selected) {
        fs::write(checkpoint_dir.join("pre-new-draw.ffstate"), state).unwrap();
        let selected_len = new_spans.iter().find(|(at, _)| *at == selected).unwrap().1;
        let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
        for native in [false, true] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let mut managed = crate::make_machine(rom.clone(), None);
            attach_from_directory(&mut managed, &prepared.directory).unwrap();
            raw.load_state(state).unwrap();
            managed.load_state(state).unwrap();
            for &(at, len) in &new_spans { managed.bus.rom[at as usize..at as usize + len].fill(0xcd); }
            let owner = managed.bus.rom_resources().unwrap().clone();
            let before = owner.obj_list_stats().unwrap();
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(selected, selected + selected_len as u32)] }, 10000).unwrap();
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
            let bytes = owner.obj_list_stats().unwrap().source_bytes - before.source_bytes;
            assert!(owner.obj_list_source_used(selected).unwrap().0 && typed > 0 && raw_reads == 0 && bytes > 0);
            if native { assert!(dispatches > 0); }
            for &(at, len) in &new_spans { managed.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary new battle list state/clock native={native}");
            ordinary.push((native, typed, raw_reads, bytes, dispatches));
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[150,149], "new_lists":101, "new_source_bytes":3574,
        "e149_projection_preserved":true, "source_rom_poisoned":true, "controlled_original_draws":6,
        "ordinary_new_source":selected.map(|v| format!("{v:06x}")), "ordinary_observation_frame":selected.map(|_| observed_frame),
        "ordinary_one_frame_pairs":ordinary, "other_ordinary_sources":"pending", "per_read_native_attribution":"pending",
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "retained EUR state, E150 dataset and translation; Wine only"]
fn european_battle_directory_index56_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const SOURCE: usize = 0x40dc0a;
    const LEN: usize = 44;
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_LISTS_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_BATTLE_LISTS_DATASET").unwrap());
    let input = fs::read(std::env::var_os("FFTA_BATTLE_LISTS_STATE").unwrap()).unwrap();
    let expected_sha = std::env::var("FFTA_BATTLE_LISTS_STATE_SHA")
        .unwrap_or_else(|_| "dc60cbe14fd375d9aefdedee9baeb65b2834cae3476be5a068feab737d4838f9".into());
    let max_frames: u32 = std::env::var("FFTA_BATTLE_LISTS_MAX_FRAMES").ok().map(|v| v.parse().unwrap()).unwrap_or(3);
    assert!((1..=3).contains(&max_frames));
    assert_eq!(gba_core::ffta_rom::digest(&input), expected_sha);
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (150, 149));
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    assert_eq!(u32::from_le_bytes(observer.bus.ewram[0x30fbc..0x30fc0].try_into().unwrap()), 0x0800_0000 + SOURCE as u32);
    if expected_sha == "31b837594b1ede7f2c0d521d3db939a4cc96a1880946bd631e9dc9c027c4f409" {
        assert_eq!(u32::from_le_bytes(observer.bus.ewram[0x30f80..0x30f84].try_into().unwrap()), 0x0802_e379);
        assert_eq!(u16::from_le_bytes(observer.bus.ewram[0x30f94..0x30f96].try_into().unwrap()), 0x0102);
        assert_eq!(observer.bus.ewram[0x30faa], 0x02);
    }
    let mut checkpoint = None;
    let mut reached_frame = 0u32;
    'observe: for frame in 0..max_frames {
        observer.bus.keys = 0x3ff;
        observer.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if observer.bus.frame_ready { break; }
            if observer.cpu.regs[15] == 0x0800_1500 && observer.cpu.regs[0] == 0x0800_0000 + SOURCE as u32 {
                checkpoint = Some(observer.save_state().unwrap());
                reached_frame = frame;
                break 'observe;
            }
            observer.step();
        }
        assert!(observer.bus.frame_ready, "ordinary index-56 observation frame limit");
    }
    let checkpoints = root.join("checkpoints");
    fs::create_dir_all(&checkpoints).unwrap();
    let mut results = Vec::new();
    if let Some(state) = &checkpoint {
        fs::write(checkpoints.join("pre-index56-draw.ffstate"), state).unwrap();
        let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
        for native in [false, true] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let mut managed = crate::make_machine(rom.clone(), None);
            attach_from_directory(&mut managed, &dataset).unwrap();
            raw.load_state(state).unwrap();
            managed.load_state(state).unwrap();
            managed.bus.rom[SOURCE..SOURCE + LEN].fill(0xcd);
            let owner = managed.bus.rom_resources().unwrap().clone();
            let before = owner.obj_list_stats().unwrap();
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else { raw.bus.frame_ready = false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(SOURCE as u32, (SOURCE + LEN) as u32)] }, 10000).unwrap();
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
            let bytes = owner.obj_list_stats().unwrap().source_bytes - before.source_bytes;
            assert!(owner.obj_list_source_used(SOURCE as u32).unwrap().0 && typed > 0 && raw_reads == 0 && bytes >= LEN as u64);
            if native { assert!(dispatches > 0); }
            managed.bus.rom[SOURCE..SOURCE + LEN].copy_from_slice(&rom[SOURCE..SOURCE + LEN]);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary index-56 state/clock native={native}");
            results.push((native, typed, raw_reads, bytes, dispatches));
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR retained continuation, bounded neutral frames", "state_sha256":expected_sha, "max_frames":max_frames,
        "candidate_index":56, "source":"40dc0a", "draw_reached":checkpoint.is_some(),
        "draw_frame":checkpoint.as_ref().map(|_| reached_frame), "one_frame_state_clock_pairs":results,
        "other_ordinary_lists":"pending", "per_read_native_attribution":"pending", "frontend_ui":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
