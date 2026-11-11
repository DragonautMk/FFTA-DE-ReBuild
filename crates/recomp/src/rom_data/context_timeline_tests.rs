//! EUR context timeline table and its original two halfword-copy sources.
use super::*;
use gba_core::{capi::RT_API, rom_data::layered_obj_animations::{
    CONTEXT_TIMELINE_ROOT, CONTEXT_TIMELINE_SOURCE_A, CONTEXT_TIMELINE_SOURCE_B,
}, Bus, Machine};

const SPANS: [(u32, usize); 3] = [
    (CONTEXT_TIMELINE_ROOT, 100),
    (CONTEXT_TIMELINE_SOURCE_A, 32),
    (CONTEXT_TIMELINE_SOURCE_B, 32),
];

#[test]
#[ignore = "exact EUR ROM, E137 predecessor and ordinary startup checkpoint; Wine only"]
fn european_context_timeline_preserves_and_serves_ordinary() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_TIMELINE_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_TIMELINE_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_TIMELINE_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "47a211b4e18f8dd8f84fbc2b8b437a29a39086e234e1ef1197970d18233d5e41");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (137, 136));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (138, 137));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e138-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e138-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new_resource = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new_resource.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old_resource.sha256);
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID {
            assert_eq!(new_resource, old_resource);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let old_model = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut model = parse_layered_obj_animations(&new_bytes).unwrap();
        assert!(old_model.context_timeline.is_none());
        let timeline = model.context_timeline.take().unwrap();
        assert_eq!(timeline.records.len(), 5);
        for (index, record) in timeline.records.iter().enumerate() {
            let at = CONTEXT_TIMELINE_ROOT as usize + index * 20;
            assert_eq!(record.source_bytes(), rom[at..at + 20]);
        }
        for (at, words) in [(CONTEXT_TIMELINE_SOURCE_A, timeline.source_a), (CONTEXT_TIMELINE_SOURCE_B, timeline.source_b)] {
            let bytes: Vec<_> = words.into_iter().flat_map(u16::to_le_bytes).collect();
            assert_eq!(bytes, rom[at as usize..at as usize + 32]);
        }
        assert_eq!(model, old_model);
        assert_eq!(encode_layered_obj_animations(&model).unwrap(), old_bytes);
        let mut metadata = new_resource.clone();
        metadata.sources.retain(|s| !s.role.starts_with("context-timeline-"));
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let poison = |machine: &mut Machine| {
        for (at, len) in SPANS { machine.bus.rom[at as usize..at as usize + len].fill(0xcd); }
    };
    let restore = |machine: &mut Machine| {
        for (at, len) in SPANS { machine.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
    };
    let before = owner.layered_obj_animations_stats().unwrap();
    poison(&mut managed);
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for (at, len) in SPANS {
        for i in 0..len { assert_eq!(managed.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
        for edge in [at, at + len as u32] {
            for delta in [-3i32, -1, 0, 1] {
                let p = edge.wrapping_add_signed(delta) as usize;
                assert_eq!((RT_API.read16)(pointer, 0x0a00_0000 + p as u32), u32::from(u16::from_le_bytes(rom[p..p + 2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x0800_0000 + p as u32), u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()));
            }
        }
    }
    let after = owner.layered_obj_animations_stats().unwrap();
    assert!(after.timeline_table_bytes - before.timeline_table_bytes >= 100);
    assert!(after.timeline_source_bytes - before.timeline_source_bytes >= 64);
    restore(&mut managed);
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let mut observations = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!(raw.cpu.regs[15], 0x0814_0986);
        assert_eq!(raw.cpu.regs[5], 0x083a_e038);
        let initial = owner.layered_obj_animations_stats().unwrap();
        let mut copied = false;
        let mut native_blocks = 0u64;
        let mut native_timeline_entry = false;
        poison(&mut managed);
        for frame in 0..3 {
            if native {
                let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
                let (count, _) = crate::run_frame_native_traced(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD,
                    |machine, kind| {
                        if kind == crate::NativeDispatch::Native && ((0x0814_0964..0x0814_0a30).contains(&machine.cpu.regs[15]) || (0x0814_3028..0x0814_3100).contains(&machine.cpu.regs[15])) { native_timeline_entry = true; }
                        if owner.layered_obj_animations_stats().unwrap().timeline_source_bytes - initial.timeline_source_bytes >= 64 {
                            copied |= machine.bus.ewram[0x1bdd8..0x1bdf8] == rom[CONTEXT_TIMELINE_SOURCE_A as usize..CONTEXT_TIMELINE_SOURCE_A as usize + 32]
                                && machine.bus.ewram[0x1bdf8..0x1be18] == rom[CONTEXT_TIMELINE_SOURCE_B as usize..CONTEXT_TIMELINE_SOURCE_B as usize + 32];
                        }
                    }, |_, _, _| {});
                native_blocks += count;
            } else {
                raw.bus.frame_ready = false;
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    assert_eq!(raw.bus.frame_ready, managed.bus.frame_ready);
                    if raw.bus.frame_ready { break; }
                    assert_eq!((raw.cpu.regs[15], raw.cpu.thumb()), (managed.cpu.regs[15], managed.cpu.thumb()));
                    let event = raw.step();
                    assert_eq!(matches!(event, gba_core::StepEvent::Instr(_)), matches!(managed.step(), gba_core::StepEvent::Instr(_)));
                    if owner.layered_obj_animations_stats().unwrap().timeline_source_bytes - initial.timeline_source_bytes >= 64 {
                        copied |= managed.bus.ewram[0x1bdd8..0x1bdf8] == rom[CONTEXT_TIMELINE_SOURCE_A as usize..CONTEXT_TIMELINE_SOURCE_A as usize + 32]
                            && managed.bus.ewram[0x1bdf8..0x1be18] == rom[CONTEXT_TIMELINE_SOURCE_B as usize..CONTEXT_TIMELINE_SOURCE_B as usize + 32];
                    }
                }
            }
            assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
            restore(&mut managed);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "ordinary state and clock, native={native}, frame={frame}");
            poison(&mut managed);
        }
        let stats = owner.layered_obj_animations_stats().unwrap();
        let table_bytes = stats.timeline_table_bytes - initial.timeline_table_bytes;
        let source_bytes = stats.timeline_source_bytes - initial.timeline_source_bytes;
        assert!(table_bytes > 0 && source_bytes >= 64 && copied);
        if native { assert!(native_blocks > 0); }
        restore(&mut managed);
        observations.push((table_bytes, source_bytes, native_timeline_entry));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[138,137], "records":5, "table_bytes":100,
        "copied_source_bytes":64, "previous_projection_preserved":true, "ordinary_pairs":true,
        "ordinary_copy_exact":true, "full_state_clock_frames_each":3, "source_rom_poisoned":true,
        "ordinary_table_source_native_entry_observed":observations,
        "all_five_rows_original_observed":"separate observer", "CLI":"pending", "other_paths":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
