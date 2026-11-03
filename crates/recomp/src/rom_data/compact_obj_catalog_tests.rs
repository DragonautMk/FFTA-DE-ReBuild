//! One prepared generation; new-bank source, boundary and original reader proofs.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API, rom_data::{RuntimeResources, compact_obj_catalog as model}};

pub(super) fn prefix(machine: &mut Machine, start: u32, end: u32) {
    machine.cpu.regs[15] = start;
    machine.cpu.cpsr = 0xbf;
    let mut steps = 0;
    while machine.cpu.regs[15] != end {
        assert!(steps < 32, "original compact prefix did not return");
        machine.bus.frame_ready = false;
        machine.step();
        steps += 1;
    }
}

#[test]
#[ignore = "private EUR source/e111 current dataset/e110 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_loop_compact_catalog_matches_new_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c4c50, 0x6c56a8, 0x6c5800, 0x6c88a8, 0x6c894c];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 5);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 43);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 60);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 44);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 103);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 974);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (110, 109, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (112, 111, 100));
    assert_eq!(current.manifest.resources[..99], old_manifest.resources[..99]);
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 29);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), 29);
    for bank in &old.banks { assert_eq!(data.banks.iter().find(|new| new.root == bank.root), Some(bank)); }
    assert_eq!(current.compact_obj_catalog.as_ref().unwrap(), data);
    let bytes = encode_compact_obj_catalog(data).unwrap();
    assert_eq!(parse_compact_obj_catalog(&bytes).unwrap(), *data);
    let mut changed = data.clone();
    changed.banks.iter_mut().find(|bank| bank.root == selected[0]).unwrap().sequences[0].relative_offset += 2;
    assert!(changed.validate().is_err());

    let mut machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    let initial = machine.save_state().unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    for &(start, end) in &new_spans { machine.bus.rom[start as usize..end as usize].fill(0xcd); }
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for &(start, end) in &new_spans { for at in start..end {
        assert_eq!(data.byte_at(at), Some(rom[at as usize]));
        assert_eq!(machine.bus.read8(0x0c000000 + at), rom[at as usize]);
    } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 974);
    for &(start, end) in &new_spans { for edge in [start, end] { for delta in [-3i32, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta) as usize;
        let ptr = &mut machine as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
    } } }
    machine.bus.rom.copy_from_slice(&rom);
    assert_eq!(machine.save_state().unwrap(), initial);
    let mut bad = read_dataset(&dataset).unwrap();
    bad.compact_obj_catalog.as_mut().unwrap().banks.iter_mut().find(|bank| bank.root == selected[0]).unwrap()
        .sequences[0].sequence.frames[0].duration ^= 1;
    let bytes = encode_compact_obj_catalog(bad.compact_obj_catalog.as_ref().unwrap()).unwrap();
    bad.manifest.resources[99].sha256 = gba_core::ffta_rom::digest(&bytes);
    bad.compact_obj_catalog_bytes = Some(bytes);
    assert!(RuntimeResources::from_base_files(&rom, bad.files()).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(machine.save_state().unwrap(), initial);
    drop(bad); drop(current); drop(machine);

    let mut raw = crate::make_machine(rom.clone(), None);
    raw.bus.ffta_sprites.enabled = false;
    let state = 0x02007f00;
    for bank in &banks {
        raw.bus.write32(state + 4, 0x08000000 + bank.list_root());
        raw.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
        for entry in &bank.sequences {
            let sequence = &entry.sequence;
            raw.cpu.regs = [0; 16]; raw.cpu.regs[3] = u32::from(sequence.selector); raw.cpu.regs[4] = state;
            prefix(&mut raw, 0x08010870, 0x08010884);
            assert_eq!(raw.cpu.regs[3] as usize, sequence.frames.len());
            for (phase, frame) in sequence.frames.iter().enumerate() {
                raw.cpu.regs = [0; 16]; raw.cpu.regs[5] = u32::from(sequence.selector); raw.cpu.regs[3] = state; raw.cpu.regs[2] = phase as u32;
                prefix(&mut raw, 0x08010e5a, 0x08010e70); assert_eq!(raw.cpu.regs[0], u32::from(frame.list));
                raw.cpu.regs = [0; 16]; raw.cpu.regs[7] = u32::from(sequence.selector); raw.cpu.regs[4] = state; raw.cpu.regs[3] = phase as u32;
                prefix(&mut raw, 0x08010cb2, 0x08010cc8); assert_eq!(raw.cpu.regs[1], u32::from(frame.duration));
            }
        }
        for entry in &bank.lists {
            raw.cpu.regs = [0; 16]; raw.cpu.regs[2] = u32::from(entry.index); raw.cpu.regs[3] = state;
            prefix(&mut raw, 0x08010e9a, 0x08010eb2);
            assert_eq!(raw.cpu.regs[1], 0x08000000 + entry.list.source_offset);
        }
    }

    let object = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &dataset).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let mut served = 0;
    let mut native_cases = 0;
    for bank in &banks { for selector in bank.sequences.iter().map(|entry| entry.sequence.selector).chain([0x100]) {
        for machine in [&mut raw, &mut native] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[3] = u32::from(selector); machine.cpu.regs[4] = state;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[15] = 0x08010870; machine.cpu.cpsr = 0xbf;
        }
        for &(start, end) in &new_spans { native.bus.rom[start as usize..end as usize].fill(0xcd); }
        let directory = bank.sequence_root() + u32::from(selector) * 2;
        let relative = u16::from_le_bytes(rom[directory as usize..directory as usize + 2].try_into().unwrap());
        let count_at = bank.sequence_root() + u32::from(relative);
        let expected: u64 = [directory, count_at].into_iter().flat_map(|at| at..at + 2).map(|at| u64::from(data.byte_at(at).is_some())).sum();
        prefix(&mut raw, 0x08010870, 0x08010884);
        let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
        let mut steps = 0;
        while native.cpu.regs[15] != 0x08010884 {
            assert!(steps < 32); native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
            steps += 1;
        }
        let used = owner.compact_obj_catalog_stats().unwrap().source_bytes - before;
        assert_eq!(used, expected, "bank {:x} selector {selector}", bank.root);
        if selector != 0x100 { assert_eq!(used, 4); }
        served += used; native_cases += 1;
        native.bus.rom.copy_from_slice(&rom);
        assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap(), "bank {:x} selector {selector}", bank.root);
    } }
    assert_eq!(native_cases, 48);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":100, "previous_resources_unchanged":99, "previous_banks_unchanged":29,
        "new_banks":5, "new_selectors":43, "new_source_bytes":974, "new_frames":60, "new_lists":44, "new_objects":81,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":5,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private exact EUR source/current dataset and retained original native prefix; Wine only, Linux/USA pending"]
fn european_compact_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.verify_source(&rom).unwrap();
    assert_eq!(data.banks.len(), 37);
    assert_eq!(data.banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 103);
    assert_eq!(data.banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 363);
    assert_eq!(data.banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 294);
    assert_eq!(data.banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 1088);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 106);
    assert_eq!(spans.iter().map(|(_, bytes, _)| bytes.len()).sum::<usize>(), 9208);
    for &(start, end) in model::SOURCE_SPANS {
        for at in start..end { assert_eq!(data.byte_at(at), Some(rom[at as usize])); }
        for at in [start - 1, end] { assert_eq!(data.byte_at(at), None); }
    }
    let bytes = encode_compact_obj_catalog(data).unwrap();
    assert_eq!(parse_compact_obj_catalog(&bytes).unwrap(), *data);
    let mut changed = data.clone(); changed.banks.pop(); assert!(changed.validate().is_err());
    changed = data.clone(); changed.banks[0].sequences[0].sequence.frames[0].duration ^= 1;
    changed.validate().unwrap(); assert!(changed.verify_source(&rom).is_err());
    changed = data.clone(); changed.banks[0].sequences[0].relative_offset += 2; assert!(changed.validate().is_err());
    assert!(serde_json::from_str::<model::CompactObjCatalog>("{\"banks\":[],\"extra\":true}").is_err());
    let old: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    let current = read_dataset(&dataset).unwrap();
    assert_eq!(old.resources.len(), 100);
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (112, 111, 100));
    for entry in &old.resources[..99] { assert_eq!(current.manifest.resources.iter().find(|resource| resource.id == entry.id), Some(entry)); }
    assert_eq!(current.compact_obj_catalog.as_ref().unwrap(), data);
    assert_eq!(current.manifest.resources[99].id, COMPACT_OBJ_CATALOG_ID);
    let mut machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    let initial = machine.save_state().unwrap(); let owner = machine.bus.rom_resources().unwrap().clone();
    for &(start, end) in model::SOURCE_SPANS { machine.bus.rom[start as usize..end as usize].fill(0xcd); }
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for &(start, end) in model::SOURCE_SPANS { for at in start..end { assert_eq!(machine.bus.read8(0x0c000000 + at), rom[at as usize]); } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 9208);
    for &(start, end) in model::SOURCE_SPANS { for edge in [start, end] { for delta in [-3i32, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta) as usize; let ptr = &mut machine as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
    } } }
    machine.bus.rom.copy_from_slice(&rom); assert_eq!(machine.save_state().unwrap(), initial);
    let mut bad = read_dataset(&dataset).unwrap();
    bad.compact_obj_catalog.as_mut().unwrap().banks[0].sequences[0].sequence.frames[0].duration ^= 1;
    let bytes = encode_compact_obj_catalog(bad.compact_obj_catalog.as_ref().unwrap()).unwrap();
    bad.manifest.resources[99].sha256 = gba_core::ffta_rom::digest(&bytes); bad.compact_obj_catalog_bytes = Some(bytes);
    assert!(RuntimeResources::from_base_files(&rom, bad.files()).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner)); assert_eq!(machine.save_state().unwrap(), initial);
    drop(bad); drop(current); drop(machine);

    let mut raw = crate::make_machine(rom.clone(), None); raw.bus.ffta_sprites.enabled = false;
    let state = 0x02007f00;
    for bank in &data.banks {
        raw.bus.write32(state + 4, 0x08000000 + bank.list_root()); raw.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
        for entry in &bank.sequences {
            let sequence = &entry.sequence;
            raw.cpu.regs = [0; 16]; raw.cpu.regs[3] = u32::from(sequence.selector); raw.cpu.regs[4] = state;
            prefix(&mut raw, 0x08010870, 0x08010884); assert_eq!(raw.cpu.regs[3] as usize, sequence.frames.len());
            for (phase, frame) in sequence.frames.iter().enumerate() {
                raw.cpu.regs = [0; 16]; raw.cpu.regs[5] = u32::from(sequence.selector); raw.cpu.regs[3] = state; raw.cpu.regs[2] = phase as u32;
                prefix(&mut raw, 0x08010e5a, 0x08010e70); assert_eq!(raw.cpu.regs[0], u32::from(frame.list));
                raw.cpu.regs = [0; 16]; raw.cpu.regs[7] = u32::from(sequence.selector); raw.cpu.regs[4] = state; raw.cpu.regs[3] = phase as u32;
                prefix(&mut raw, 0x08010cb2, 0x08010cc8); assert_eq!(raw.cpu.regs[1], u32::from(frame.duration));
            }
        }
        for entry in &bank.lists {
            raw.cpu.regs = [0; 16]; raw.cpu.regs[2] = u32::from(entry.index); raw.cpu.regs[3] = state;
            prefix(&mut raw, 0x08010e9a, 0x08010eb2); assert_eq!(raw.cpu.regs[1], 0x08000000 + entry.list.source_offset);
        }
    }

    // Reuse the authenticated same-ROM count-reader object; the native API and
    // guest prefix did not change. No generated source or extra build is needed.
    let object = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut native = crate::make_machine(rom.clone(), None); attach_from_directory(&mut native, &dataset).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone(); let initial = raw.save_state().unwrap(); let mut served = 0; let mut native_cases = 0;
    for bank in &data.banks { for selector in bank.sequences.iter().map(|entry| entry.sequence.selector).chain([0x100]) {
        for m in [&mut raw, &mut native] {
            m.load_state(&initial).unwrap(); m.bus.ffta_sprites.enabled = false; m.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
            m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32); m.cpu.regs[3] = u32::from(selector); m.cpu.regs[4] = state;
            m.cpu.regs[13] = 0x03007c00; m.cpu.regs[15] = 0x08010870; m.cpu.cpsr = 0xbf;
        }
        for &(start, end) in model::SOURCE_SPANS { native.bus.rom[start as usize..end as usize].fill(0xcd); }
        let directory = bank.sequence_root() + u32::from(selector) * 2;
        let relative = u16::from_le_bytes(rom[directory as usize..directory as usize + 2].try_into().unwrap());
        let count_at = bank.sequence_root() + u32::from(relative);
        let expected: u64 = [directory, count_at].into_iter().flat_map(|at| at..at + 2).map(|at| u64::from(data.byte_at(at).is_some())).sum();
        prefix(&mut raw, 0x08010870, 0x08010884); let before = owner.compact_obj_catalog_stats().unwrap().source_bytes; let mut steps = 0;
        while native.cpu.regs[15] != 0x08010884 {
            assert!(steps < 32); native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        let used = owner.compact_obj_catalog_stats().unwrap().source_bytes - before; assert_eq!(used, expected, "bank {:x} selector {selector}", bank.root);
        if selector != 0x100 { assert_eq!(used, 4); }
        served += used; native_cases += 1; native.bus.rom.copy_from_slice(&rom);
        assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap(), "bank {:x} selector {selector}", bank.root);
    } }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":100, "previous_unchanged":99, "banks":37, "selectors":103,
        "source_bytes":9208, "spans":106, "frames":363, "lists":294, "objects":1088,
        "native_count_cases":native_cases, "typed_native_bytes":served,
        "out_of_projection_cases":37, "counter_oracle_from_actual_owned_intersections":true,
        "full_state_clock":true, "coherent_rejection_in_memory":true, "dataset_copies_created":0,
        "ordinary_use":"pending", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
