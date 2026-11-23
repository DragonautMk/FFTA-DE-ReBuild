//! EUR White Flowers byte-indexed switch with its original guest dispatch.
use super::*;
use gba_core::{capi::RT_API, rom_data::thumb_switch_tables as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, pc: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + pc;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E285 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_switch_and_original_dispatch() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_SWITCH_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_SWITCH_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (285, 284, 146));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (286, 285, 146));
    let spec = model::ADDITIONAL_TABLES[34];
    assert_eq!((spec.root, spec.start(), spec.end(), spec.count), (0x32e10, 0x32e14, 0x33214, 256));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(&previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 1);
            let old_data = parse_thumb_switch_tables(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            assert_eq!(old_data.additional_tables.len(), model::FIRAGA_ADDITIONAL_TABLE_COUNT);
            old_data.validate().unwrap();
            assert!(old_data.byte_at(spec.start()).is_none());
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let data = &source.thumb_switch_tables.as_ref().unwrap().decoded;
    assert_eq!(data.additional_tables.len(), model::ADDITIONAL_TABLE_COUNT);
    assert_eq!(data.additional_tables[34].destinations.len(), 256);
    assert_eq!(data.additional_tables[34].destination_bytes(), rom[spec.start() as usize..spec.end() as usize]);
    assert_eq!(parse_thumb_switch_tables(&fs::read(prepared.directory.join(THUMB_SWITCH_TABLES_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    invalid.additional_tables[34].destinations.pop();
    assert!(invalid.validate().is_err());
    let mut invalid = data.clone();
    invalid.additional_tables[34].destinations[0].address |= 1;
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["additional_tables"][34]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ThumbSwitchTables>(invalid).is_err());
    let mut forged = data.clone();
    forged.additional_tables[34].destinations[0].address ^= 2;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_thumb_switch_tables(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == THUMB_SWITCH_TABLES_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.thumb_switch_tables = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for at in spec.root as usize..spec.end() as usize { managed.bus.rom[at] = !rom[at]; }
    let before = owner.thumb_switch_tables_stats().unwrap();
    let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
    for at in spec.start()..spec.end() {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]);
        }
    }
    for at in spec.root..spec.start() {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]);
        }
    }
    let after = owner.thumb_switch_tables_stats().unwrap();
    assert_eq!(after.additional_source_bytes - before.additional_source_bytes, 3072);
    assert_eq!(after.additional_destinations_mask, 1u64 << 34);
    assert_eq!(owner.thumb_pc_literals_stats().unwrap().source_bytes - root_before, 12);
    let current_owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for at in (spec.start() as usize - 3..spec.start() as usize).chain(spec.end() as usize..spec.end() as usize + 4) {
        if !current_owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    }
    for edge in [spec.start() as usize, spec.end() as usize] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| if changed.contains(&(at+n)) { rom[at+n] ^ 0x5a } else { rom[at+n] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])), "read16 {at:x}");
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]), "read32 {at:x}");
            }
        }
    }
    managed.bus.rom[spec.root as usize..spec.end() as usize].copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
    for at in changed { managed.bus.rom[at] = rom[at]; }

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x0800_0000 + spec.code_start..0x0800_0000 + spec.mov).step_by(2)
        .map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("white-flowers-switch-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool| {
        machine.bus.frame_ready = false;
        let pc = machine.cpu.regs[15];
        if native && pc != 0x0800_0000 + spec.mov {
            table.get(pc | 1).unwrap_or_else(|| panic!("Missing generated reader {pc:08x}"))(&RT_API, machine as *mut Machine as *mut std::ffi::c_void);
        } else { machine.step(); }
    };
    let mut dispatch_cases = 0;
    let mut guard_cases = 0;
    let mut fallback_cases = 0;
    for native in [false, true] {
        for index in [0u8, 5, 45, 254, 255] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, spec.code_start);
                machine.cpu.regs[1] = 0x0203_0100;
                machine.bus.write8(0x0203_0100, index);
            }
            for at in spec.root as usize..spec.end() as usize { managed.bus.rom[at] = !rom[at]; }
            let before = owner.thumb_switch_tables_stats().unwrap().additional_source_bytes;
            let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
            let at = spec.start() as usize + usize::from(index) * 4;
            let destination = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
            for _ in 0..16 { if raw.cpu.regs[15] == destination { break; } execute(&mut raw, false); }
            for _ in 0..16 { if managed.cpu.regs[15] == destination { break; } execute(&mut managed, native); }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (destination, destination));
            assert_eq!(owner.thumb_switch_tables_stats().unwrap().additional_source_bytes - before, 4);
            assert_eq!(owner.thumb_pc_literals_stats().unwrap().source_bytes - root_before, 4);
            managed.bus.rom[spec.root as usize..spec.end() as usize].copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "index={index}, native={native}");
            dispatch_cases += 1;
        }
        // Bypass the byte load only to exercise the original guard's out-of-domain branch.
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, spec.guard); machine.cpu.regs[0] = 256; }
        let before = owner.thumb_switch_tables_stats().unwrap().additional_source_bytes;
        // A native chain can cross the BL halves in one call, so compare at
        // the original call destination rather than at an interior label.
        let high_pc = 0x0800_0000 + spec.guard + 4;
        let high = armv4t::decode_thumb(view.read16(high_pc), high_pc);
        let low = armv4t::decode_thumb(view.read16(high_pc + 2), high_pc + 2);
        let (stop, _) = armv4t::fuse_thumb_bl(&high, &low).unwrap();
        assert_eq!(stop, 0x0803_60d2);
        for _ in 0..8 { if raw.cpu.regs[15] == stop { break; } execute(&mut raw, false); }
        for _ in 0..8 { if managed.cpu.regs[15] == stop { break; } execute(&mut managed, native); }
        assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop));
        assert_eq!(owner.thumb_switch_tables_stats().unwrap().additional_source_bytes - before, 0);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        guard_cases += 1;

        // A direct reader past the projection must still fall back to the original owner.
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, spec.reader); machine.cpu.regs[0] = 256; }
        for at in spec.root as usize..spec.end() as usize { managed.bus.rom[at] = !rom[at]; }
        let before = owner.thumb_switch_tables_stats().unwrap().additional_source_bytes;
        let stop = 0x0800_0000 + spec.mov;
        for _ in 0..8 { if raw.cpu.regs[15] == stop { break; } execute(&mut raw, false); }
        for _ in 0..8 { if managed.cpu.regs[15] == stop { break; } execute(&mut managed, native); }
        assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop));
        assert_eq!(owner.thumb_switch_tables_stats().unwrap().additional_source_bytes - before, 0);
        managed.bus.rom[spec.root as usize..spec.end() as usize].copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        fallback_cases += 1;
    }
    assert_eq!((dispatch_cases, guard_cases, fallback_cases), (10, 2, 2));
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[286,285], "resources":146, "new_tables":1, "new_destinations":256, "new_source_bytes":1024,
        "prior_switch_representation_preserved":true, "prior_other_resources_unchanged":145, "root_owner_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_dispatch_cases":dispatch_cases,
        "outside_guard_cases":guard_cases, "outside_projection_fallback_cases":fallback_cases,
        "indirect_mov_interpreter_fallback":true, "full_state_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
