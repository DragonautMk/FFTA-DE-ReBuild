//! EUR White Flowers final switches and their original guarded dispatches.
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

fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}

fn set_input(machine: &mut Machine, rom: &[u8], spec: model::AdditionalTableSpec, index: u32) {
    setup(machine, spec.code_start);
    match spec.root {
        0x02a7b0 => machine.cpu.regs[3] = index + 1,
        0x125fe4 => { machine.cpu.regs[4] = 0x0203_0100; machine.bus.write8(0x0203_0106, (index + 3) as u8); }
        0x1279b8 => machine.cpu.regs[5] = index,
        _ => panic!("Unknown switch {:x}", spec.root),
    }
    let _ = rom;
}

fn outside_target(rom: &[u8], spec: model::AdditionalTableSpec) -> u32 {
    let branch = half(rom, spec.guard + 2);
    if branch & 0xff00 == 0xd800 {
        let displacement = i32::from(branch as u8 as i8) * 2;
        return (0x0800_0000i64 + i64::from(spec.guard) + 6 + i64::from(displacement)) as u32;
    }
    assert_eq!(branch & 0xff00, 0xd900);
    let pc = 0x0800_0000 + spec.guard + 4;
    let first = armv4t::decode_thumb(half(rom, spec.guard + 4), pc);
    let second = armv4t::decode_thumb(half(rom, spec.guard + 6), pc + 2);
    if let Some((target, _)) = armv4t::fuse_thumb_bl(&first, &second) { return target; }
    let unconditional = half(rom, spec.guard + 4);
    assert_eq!(unconditional & 0xf800, 0xe000);
    let displacement = ((i32::from(unconditional & 0x7ff)) << 21) >> 20;
    (0x0800_0000i64 + i64::from(spec.guard) + 8 + i64::from(displacement)) as u32
}

#[test]
#[ignore = "exact EUR ROM and E288 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_final_switches_and_original_dispatch() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_FINAL_SWITCHES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_FINAL_SWITCHES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (288, 287, 146));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (289, 288, 146));
    let specs = &model::ADDITIONAL_TABLES[47..50];
    assert_eq!(specs.iter().map(|s| s.count).sum::<usize>(), 30);
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(&previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 3);
            let old_data = parse_thumb_switch_tables(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            assert_eq!(old_data.additional_tables.len(), model::WHITE_FLOWERS_MIXED_TABLE_COUNT);
            old_data.validate().unwrap();
            for spec in specs { assert!(old_data.byte_at(spec.start()).is_none()); }
        } else if prior.id == THUMB_PC_LITERALS_ID {
            let old_data = parse_thumb_pc_literals(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_thumb_pc_literals(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            assert!(old_data.white_flowers_final.is_none());
            assert!(new_data.white_flowers_final.take().is_some());
            assert_eq!(new_data, old_data);
            assert_eq!(&current.sources[..prior.sources.len()], prior.sources.as_slice());
            assert_eq!(current.sources.len() - prior.sources.len(), 3);
            assert_eq!((&current.id, &current.path, &current.format, &current.dependencies), (&prior.id, &prior.path, &prior.format, &prior.dependencies));
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let data = &source.thumb_switch_tables.as_ref().unwrap().decoded;
    assert_eq!(data.additional_tables.len(), model::ADDITIONAL_TABLE_COUNT);
    for (table, spec) in data.additional_tables[47..50].iter().zip(specs) {
        assert_eq!(table.destinations.len(), spec.count);
        assert_eq!(table.destination_bytes(), rom[spec.start() as usize..spec.end() as usize]);
    }
    assert_eq!(parse_thumb_switch_tables(&fs::read(prepared.directory.join(THUMB_SWITCH_TABLES_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    invalid.additional_tables[47].destinations.pop();
    assert!(invalid.validate().is_err());
    let mut invalid = data.clone();
    invalid.additional_tables[47].destinations[0].address |= 1;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.additional_tables[47].destinations[0].address ^= 2;
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
    let literal_data = &source.thumb_pc_literals.as_ref().unwrap().decoded;
    let words = literal_data.white_flowers_final.as_ref().unwrap();
    assert_eq!(words.len(), 3);
    for word in words {
        let at = word.source_offset as usize;
        assert_eq!(word.value.to_le_bytes(), rom[at..at + 4]);
    }
    assert_eq!(parse_thumb_pc_literals(&fs::read(prepared.directory.join(THUMB_PC_LITERALS_FILE)).unwrap()).unwrap(), *literal_data);
    let mut invalid = literal_data.clone();
    invalid.white_flowers_final.as_mut().unwrap().swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut forged = literal_data.clone();
    forged.white_flowers_final.as_mut().unwrap()[0].value ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_thumb_pc_literals(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == THUMB_PC_LITERALS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.thumb_pc_literals = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent literal/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for spec in specs { for at in spec.root as usize..spec.end() as usize { managed.bus.rom[at] = !rom[at]; } }
    for at in 0x02a7ac..0x02a7b0 { managed.bus.rom[at] = !rom[at]; }
    let before = owner.thumb_switch_tables_stats().unwrap();
    let mixed_before = owner.thumb_pc_literals_stats().unwrap().white_flowers_final_bytes;
    let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
    for spec in specs {
        for at in spec.start()..spec.end() {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]); }
        }
        for at in spec.root..spec.start() {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]); }
        }
    }
    for at in 0x02a7ac..0x02a7b0 {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    }
    let after = owner.thumb_switch_tables_stats().unwrap();
    assert_eq!(after.additional_source_bytes - before.additional_source_bytes, 360);
    assert_eq!(after.additional_destinations_mask, 0x7u64 << 47);
    let literals = owner.thumb_pc_literals_stats().unwrap();
    assert_eq!(literals.source_bytes - root_before, 48);
    assert_eq!(literals.white_flowers_final_bytes - mixed_before, 36);
    assert_eq!(literals.white_flowers_final_mask, 0x7);
    let current_owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for spec in specs {
        for at in (spec.start() as usize - 3..spec.start() as usize).chain(spec.end() as usize..spec.end() as usize + 4) {
            if !current_owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
        }
    }
    for spec in specs {
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
    }
    for spec in specs { managed.bus.rom[spec.root as usize..spec.end() as usize].copy_from_slice(&rom[spec.root as usize..spec.end() as usize]); }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    managed.bus.rom[0x02a7ac..0x02a7b0].copy_from_slice(&rom[0x02a7ac..0x02a7b0]);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let mut addresses = BTreeSet::new();
    for spec in specs { addresses.extend((0x0800_0000 + spec.code_start..0x0800_0000 + spec.mov).step_by(2)); }
    addresses.insert(0x0802_a78c);
    let blocks = addresses.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("white-flowers-final-switches-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool, spec: model::AdditionalTableSpec| {
        machine.bus.frame_ready = false;
        let pc = machine.cpu.regs[15];
        if native && pc != 0x0800_0000 + spec.mov {
            table.get(pc | 1).unwrap_or_else(|| panic!("Missing generated reader {pc:08x}"))(&RT_API, machine as *mut Machine as *mut std::ffi::c_void);
        } else { machine.step(); }
    };
    let sampled: [&[u32]; 3] = [&[0], &[0], &[1]];
    let mut dispatch_cases = 0;
    let mut guard_cases = 0;
    let mut fallback_cases = 0;
    for native in [false, true] {
        for (which, &spec) in specs.iter().enumerate() {
            let mut indices = BTreeSet::from([0, spec.count as u32 - 1]);
            indices.extend(sampled[which].iter().copied());
            for index in indices {
                for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); set_input(machine, &rom, spec, index); }
                for at in spec.root as usize..spec.end() as usize { managed.bus.rom[at] = !rom[at]; }
                let before = owner.thumb_switch_tables_stats().unwrap().additional_source_bytes;
                let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
                let at = spec.start() as usize + index as usize * 4;
                let destination = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
                for _ in 0..24 { if raw.cpu.regs[15] == destination { break; } execute(&mut raw, false, spec); }
                for _ in 0..24 { if managed.cpu.regs[15] == destination { break; } execute(&mut managed, native, spec); }
                assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (destination, destination), "root={:x}, index={index}, native={native}", spec.root);
                assert_eq!(owner.thumb_switch_tables_stats().unwrap().additional_source_bytes - before, 4);
                assert_eq!(owner.thumb_pc_literals_stats().unwrap().source_bytes - root_before, 4);
                managed.bus.rom[spec.root as usize..spec.end() as usize].copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
                assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "root={:x}, index={index}, native={native}", spec.root);
                dispatch_cases += 1;
            }
            // Observe the original out-of-domain transfer before executing its destination.
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); set_input(machine, &rom, spec, spec.count as u32); }
            let before = owner.thumb_switch_tables_stats().unwrap().additional_source_bytes;
            let stop = outside_target(&rom, spec);
            for _ in 0..12 { if raw.cpu.regs[15] == stop { break; } execute(&mut raw, false, spec); }
            for _ in 0..12 { if managed.cpu.regs[15] == stop { break; } execute(&mut managed, native, spec); }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop), "guard root={:x}, native={native}", spec.root);
            assert_eq!(owner.thumb_switch_tables_stats().unwrap().additional_source_bytes - before, 0);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            guard_cases += 1;

            // Bypass the guard to confirm out-of-projection reads retain their original owner.
            let selector = usize::from((half(&rom, spec.reader) >> 3) & 7);
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, spec.reader); machine.cpu.regs[selector] = spec.count as u32; }
            for at in spec.root as usize..spec.end() as usize { managed.bus.rom[at] = !rom[at]; }
            let before = owner.thumb_switch_tables_stats().unwrap().additional_source_bytes;
            let stop = 0x0800_0000 + spec.mov;
            for _ in 0..12 { if raw.cpu.regs[15] == stop { break; } execute(&mut raw, false, spec); }
            for _ in 0..12 { if managed.cpu.regs[15] == stop { break; } execute(&mut managed, native, spec); }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop), "fallback root={:x}, native={native}", spec.root);
            assert_eq!(owner.thumb_switch_tables_stats().unwrap().additional_source_bytes - before, 0);
            managed.bus.rom[spec.root as usize..spec.end() as usize].copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            fallback_cases += 1;
        }
    }
    assert_eq!((dispatch_cases, guard_cases, fallback_cases), (14, 6, 6));
    for native in [false, true] {
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, 0x02a78c); }
        for at in 0x02a7ac..0x02a7b0 { managed.bus.rom[at] = !rom[at]; }
        let before = owner.thumb_pc_literals_stats().unwrap().white_flowers_final_bytes;
        raw.step();
        if native { table.get(0x0802_a78d).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
        else { managed.step(); }
        assert_eq!(owner.thumb_pc_literals_stats().unwrap().white_flowers_final_bytes - before, 4);
        managed.bus.rom[0x02a7ac..0x02a7b0].copy_from_slice(&rom[0x02a7ac..0x02a7b0]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[289,288], "resources":146, "new_tables":3, "new_destinations":30, "new_literal_words":3, "new_source_bytes":132,
        "prior_switch_and_literal_representations_preserved":true, "prior_other_resources_unchanged":144, "root_owners_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_dispatch_cases":dispatch_cases,
        "outside_guard_cases":guard_cases, "outside_projection_fallback_cases":fallback_cases, "standalone_literal_reader_cases":2,
        "indirect_mov_interpreter_fallback":true, "full_state_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
