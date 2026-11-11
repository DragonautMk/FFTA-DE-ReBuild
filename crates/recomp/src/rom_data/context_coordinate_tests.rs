//! EUR adjacent context coordinates, managed source projection and original consumer.
use super::*;
use gba_core::{capi::RT_API, rom_data::layered_obj_animations::CONTEXT_COORDINATE_OFFSETS, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

const TASK: u32 = 0x0201_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;
const END: u32 = 0x0814_372a;
const SEED: u32 = 0x1234_5678;

fn setup(machine: &mut Machine, source: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[..4].copy_from_slice(&[TASK, 0x0800_0000 + source, 0, 0]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0814_36b4;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write32(0x0300_34b0, SEED);
    for offset in [0, 4, 8] { machine.bus.write32(STACK + offset, 0); }
}

fn interpret(machine: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == END { return; }
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("context coordinate prefix did not finish");
}

fn translated(machine: &mut Machine, table: &crate::BlockTable) {
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == END { return; }
        machine.bus.frame_ready = false;
        table.get(machine.cpu.regs[15] | u32::from(machine.cpu.thumb()))
            .expect("original traced instruction compiled")(&RT_API, machine as *mut Machine as *mut std::ffi::c_void);
    }
    panic!("translated context coordinate prefix did not finish");
}

#[test]
#[ignore = "exact EUR ROM, E136 predecessor and TinyCC; Windows executable under Wine"]
fn european_context_coordinate_records_preserve_and_consume() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_COORDINATE_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_COORDINATE_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (136, 135));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (137, 136));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e137-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e137-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
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
        assert_eq!(old_model.context_coordinate_records.len(), 0);
        assert_eq!(model.context_coordinate_records.len(), 6);
        for (record, offset) in model.context_coordinate_records.iter().zip(CONTEXT_COORDINATE_OFFSETS) {
            assert_eq!(record.source_offset, offset);
            assert_eq!(record.source_bytes(), rom[offset as usize..offset as usize + 8]);
        }
        model.context_coordinate_records.clear();
        assert_eq!(model, old_model);
        assert_eq!(encode_layered_obj_animations(&model).unwrap(), old_bytes);
        let mut metadata = new_resource.clone();
        metadata.sources.retain(|s| s.role != "context-coordinate-record");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let poison = |machine: &mut Machine| {
        for at in CONTEXT_COORDINATE_OFFSETS { machine.bus.rom[at as usize..at as usize + 8].fill(0xcd); }
    };
    let restore = |machine: &mut Machine| {
        for at in CONTEXT_COORDINATE_OFFSETS { machine.bus.rom[at as usize..at as usize + 8].copy_from_slice(&rom[at as usize..at as usize + 8]); }
    };
    poison(&mut managed);
    let initial_stats = owner.layered_obj_animations_stats().unwrap();
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for at in CONTEXT_COORDINATE_OFFSETS {
        for delta in 0..8 { assert_eq!(managed.bus.read8(0x0800_0000 + at + delta), rom[at as usize + delta as usize]); }
        for edge in [at, at + 8] {
            for delta in [-3i32, -1, 0, 1] {
                let p = edge.wrapping_add_signed(delta) as usize;
                assert_eq!((RT_API.read16)(pointer, 0x0a00_0000 + p as u32), u32::from(u16::from_le_bytes(rom[p..p + 2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x0800_0000 + p as u32), u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()));
            }
        }
    }
    assert!(owner.layered_obj_animations_stats().unwrap().coordinate_bytes - initial_stats.coordinate_bytes >= 48);
    let mut visited = BTreeSet::new();
    for at in CONTEXT_COORDINATE_OFFSETS {
        raw.load_state(&initial).unwrap();
        restore(&mut managed);
        managed.load_state(&initial).unwrap();
        poison(&mut managed);
        setup(&mut raw, at);
        setup(&mut managed, at);
        interpret(&mut raw, &mut visited);
        interpret(&mut managed, &mut BTreeSet::new());
        restore(&mut managed);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "original coordinate prefix {at:x}");
    }
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&(addr, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(addr), addr) } else { armv4t::decode_arm(view.read32(addr), addr) };
        let mut instrs = vec![first];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr, thumb, instrs }
    }).collect();
    let object = root.join("context-coordinate-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for at in CONTEXT_COORDINATE_OFFSETS {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        poison(&mut managed);
        setup(&mut raw, at);
        setup(&mut managed, at);
        interpret(&mut raw, &mut BTreeSet::new());
        translated(&mut managed, &table);
        restore(&mut managed);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "translated coordinate prefix {at:x}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[137,136], "records":6, "source_bytes":48,
        "prior_resource_projection_preserved":true, "interpreter_prefix_cases":6,
        "translated_prefix_cases":6, "source_rom_poisoned":true, "original_full_initializer":"pending",
        "ordinary_scene":"pending", "CLI":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "ordinary EUR startup pre-call state, E137 dataset and translation; Wine only"]
fn european_context_coordinate_ordinary_initializer_pairs() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_COORDINATE_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_COORDINATE_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_COORDINATE_STATE").unwrap()).unwrap();
    let expected_sha = std::env::var("FFTA_CONTEXT_COORDINATE_STATE_SHA")
        .unwrap_or_else(|_| "2112f1b6f04a0be42db0a78418386ae0a9165f6b75eff693b7bf52fec4ca60cf".into());
    assert_eq!(gba_core::ffta_rom::digest(&state), expected_sha);
    let address = |name: &str, default: u32| std::env::var(name).ok()
        .map(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).unwrap())
        .unwrap_or(default);
    let call = address("FFTA_CONTEXT_COORDINATE_CALL", 0x0814_0e1e);
    let context = address("FFTA_CONTEXT_COORDINATE_CONTEXT", 0x0201_9a54);
    let source = address("FFTA_CONTEXT_COORDINATE_SOURCE", 0x003a_e164);
    assert!(CONTEXT_COORDINATE_OFFSETS.contains(&source));
    let destination = context.checked_sub(0x0200_0000).unwrap() as usize + 0x10;
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let mut bytes_used = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!(raw.cpu.regs[15], call);
        assert_eq!([raw.cpu.regs[0], raw.cpu.regs[1]], [context, 0x0800_0000 + source]);
        let owner = managed.bus.rom_resources().unwrap().clone();
        let initial_bytes = owner.layered_obj_animations_stats().unwrap().coordinate_bytes;
        let poison = |machine: &mut Machine| {
            for at in CONTEXT_COORDINATE_OFFSETS { machine.bus.rom[at as usize..at as usize + 8].fill(0xcd); }
        };
        let restore = |machine: &mut Machine| {
            for at in CONTEXT_COORDINATE_OFFSETS { machine.bus.rom[at as usize..at as usize + 8].copy_from_slice(&rom[at as usize..at as usize + 8]); }
        };
        let mut copied = false;
        let mut native_blocks = 0u64;
        poison(&mut managed);
        for frame in 0..3 {
            if native {
                let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
                let (count, _) = crate::run_frame_native_traced(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD,
                    |machine, _| {
                        if owner.layered_obj_animations_stats().unwrap().coordinate_bytes - initial_bytes >= 8 {
                            copied |= machine.bus.ewram[destination..destination + 8] == rom[source as usize..source as usize + 8];
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
                    let raw_event = raw.step();
                    assert_eq!(matches!(raw_event, gba_core::StepEvent::Instr(_)), matches!(managed.step(), gba_core::StepEvent::Instr(_)));
                    if owner.layered_obj_animations_stats().unwrap().coordinate_bytes - initial_bytes >= 8 {
                        copied |= managed.bus.ewram[destination..destination + 8] == rom[source as usize..source as usize + 8];
                    }
                }
            }
            assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
            restore(&mut managed);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "ordinary full state and clock, native={native}, frame={frame}");
            poison(&mut managed);
        }
        let count = owner.layered_obj_animations_stats().unwrap().coordinate_bytes - initial_bytes;
        assert!(count >= 8);
        assert!(copied, "ordinary initializer did not copy the selected record, native={native}");
        if native { assert!(native_blocks > 0); }
        bytes_used.push(count);
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[137,136], "ordinary_source":format!("{source:06x}"),
        "ordinary_interpreter_pair":true, "ordinary_native_dispatch_pair":true,
        "exact_context_copy":true, "source_rom_poisoned":true,
        "full_state_clock_frames_each":3, "coordinate_bytes":bytes_used,
        "translated_initializer_execution":"not separately established", "other_records":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
