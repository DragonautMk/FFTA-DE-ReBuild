//! One prepared generation; new-bank source, boundary and original reader proofs.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API, rom_data::{RuntimeResources, compact_obj_catalog as model}};

fn assert_previous_bank_preserved(old: &model::Bank, current: &model::Bank) {
    if old.root != 0x6c1e78 {
        assert_eq!(old, current);
        return;
    }
    assert_eq!((old.root, old.flags, old.sequence_relative_offset),
        (current.root, current.flags, current.sequence_relative_offset));
    assert_eq!(old.lists, current.lists);
    for sequence in &old.sequences {
        assert_eq!(current.sequences.iter().find(|entry| entry.sequence.selector == sequence.sequence.selector), Some(sequence));
    }
}

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
#[ignore = "private EUR e130 current/e127 prior and native reader; Wine only"]
fn european_6c1e78_default_selectors_match_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let current_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let prior_manifest: Manifest = toml::from_str(&fs::read_to_string(prior_dir.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((prior_manifest.extractor_revision, prior_manifest.decoder_revision, prior_manifest.resources.len()), (127, 126, 100));
    let prior: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior_dir.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    let current = read_dataset(&current_dir).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&prior_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    let data = current.compact_obj_catalog.as_ref().unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(data.banks.len(), 63);
    assert_eq!(data.selector_tables, prior.selector_tables);
    for bank in &prior.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let bank = data.banks.iter().find(|bank| bank.root == 0x6c1e78).unwrap();
    assert_eq!(bank.sequences.iter().map(|entry| entry.sequence.selector).collect::<Vec<_>>(), [3, 4, 5, 6]);
    assert_eq!(bank.sequences.iter().map(|entry| entry.sequence.frames.len()).collect::<Vec<_>>(), [13, 19, 43, 43]);
    let spans = model::BANK_SPECS.iter().find(|spec| spec.root == bank.root).unwrap().spans;
    assert_eq!(spans.iter().map(|(a,z)| z-a).sum::<u32>(), 524);
    let mut raw = crate::make_machine(rom.clone(), None);
    for base in [3, 4] { for value in 0..=10 {
        raw.cpu.regs = [0; 16]; raw.cpu.regs[0] = value; raw.cpu.regs[5] = base;
        prefix(&mut raw, 0x08109688, 0x0810969a);
        assert_eq!(raw.cpu.regs[5], base + if [3,5,7].contains(&value) { 2 } else { 0 });
    } }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &current_dir).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = managed.save_state().unwrap();
    for &(a,z) in spans { managed.bus.rom[a as usize..z as usize].fill(0xcd); }
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for &(a,z) in spans { for at in a..z { assert_eq!(managed.bus.read8(0x0c000000 + at), rom[at as usize]); } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 524);
    for &(a,z) in spans { for edge in [a,z] { for delta in [-3i32,-1,0,1] {
        let at = edge.wrapping_add_signed(delta) as usize;
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    } } }
    managed.bus.rom.copy_from_slice(&rom); assert_eq!(managed.save_state().unwrap(), initial);
    let mut bad = read_dataset(&current_dir).unwrap();
    bad.compact_obj_catalog.as_mut().unwrap().banks.iter_mut().find(|bank| bank.root == 0x6c1e78).unwrap().sequences[1].sequence.frames[0].duration ^= 1;
    let bytes = encode_compact_obj_catalog(bad.compact_obj_catalog.as_ref().unwrap()).unwrap();
    bad.manifest.resources[99].sha256 = gba_core::ffta_rom::digest(&bytes); bad.compact_obj_catalog_bytes = Some(bytes);
    assert!(RuntimeResources::from_base_files(&rom, bad.files()).is_err());
    assert!(std::sync::Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    assert_eq!(managed.save_state().unwrap(), initial); drop(bad); drop(managed);

    let object = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut native = crate::make_machine(rom.clone(), None); attach_from_directory(&mut native, &current_dir).unwrap();
    let native_owner = native.bus.rom_resources().unwrap().clone(); let initial = raw.save_state().unwrap();
    let state = 0x02007f00; let mut served = 0;
    for selector in [4u16, 6, 0x100] {
        for m in [&mut raw, &mut native] {
            m.load_state(&initial).unwrap(); m.bus.ffta_sprites.enabled = false;
            m.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
            m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32); m.cpu.regs[3] = u32::from(selector); m.cpu.regs[4] = state;
            m.cpu.regs[13] = 0x03007c00; m.cpu.regs[15] = 0x08010870; m.cpu.cpsr = 0xbf;
        }
        for &(a,z) in spans { native.bus.rom[a as usize..z as usize].fill(0xcd); }
        let directory = bank.sequence_root() + u32::from(selector) * 2;
        let relative = u16::from_le_bytes(rom[directory as usize..directory as usize + 2].try_into().unwrap());
        let count_at = bank.sequence_root() + u32::from(relative);
        let expected: u64 = [directory, count_at].into_iter().flat_map(|at| at..at+2).map(|at| u64::from(data.byte_at(at).is_some())).sum();
        prefix(&mut raw, 0x08010870, 0x08010884);
        let before = native_owner.compact_obj_catalog_stats().unwrap().source_bytes; let mut steps = 0;
        while native.cpu.regs[15] != 0x08010884 {
            assert!(steps < 32); native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        let used = native_owner.compact_obj_catalog_stats().unwrap().source_bytes - before;
        assert_eq!(used, expected); if selector != 0x100 { assert_eq!(used, 4); } served += used;
        native.bus.rom.copy_from_slice(&rom); assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[130,129], "resources":current.manifest.resources.len(),
        "unchanged_other_resources":98, "previous_obj_projection_preserved":true, "preserved_banks":prior.banks.len(), "bank_source_bytes":524,
        "selectors":[3,4,5,6], "new_selectors":[4,6], "branch_cases":22,
        "native_count_cases":3, "typed_native_bytes":served, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "Linux":"pending", "USA":"pending",
        "nativeWindows":"pending", "ordinary_use":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
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
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 81);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 974);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (110, 109, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    let current_resources = current.manifest.resources.len();
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 29);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
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
        "status":"PASS", "resources":current_resources, "previous_resources_unchanged":old_manifest.resources[..99].len()-1, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":banks.len(), "new_selectors":banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(),
        "new_source_bytes":new_spans.iter().map(|(a, z)| z - a).sum::<u32>(),
        "new_frames":banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(),
        "new_lists":banks.iter().map(|bank| bank.lists.len()).sum::<usize>(),
        "new_objects":banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(),
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":banks.len(),
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e113 current dataset/e112 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6ca5f8_compact_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6ca5f8];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 3);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 37);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 9);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 8);
    assert!(banks[0].lists.iter().find(|entry| entry.index == 8).unwrap().list.objects.is_empty());
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 174);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (112, 111, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 37);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 174);
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
    assert_eq!(native_cases, 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":3, "new_source_bytes":174, "new_frames":37, "new_lists":9, "new_objects":8,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e116 current dataset/e115 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6c4910_compact_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c4910];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 1);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 8);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 6);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 19);
    assert!(banks[0].lists.iter().find(|entry| entry.index == 5).unwrap().list.objects.is_empty());
    assert_eq!(data.byte_at(0x6c499e), None);
    assert_eq!(data.byte_at(0x6c499f), None);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 162);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (115, 114, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 40);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 162);
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
    assert_eq!(native_cases, 2);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":1, "new_source_bytes":162, "new_frames":8, "new_lists":6, "new_objects":19,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e117 current dataset/e116 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_alternate_compact_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6b6448, 0x6bde00, 0x6bfca4, 0x6c036c, 0x6c1b28, 0x6c2300, 0x6c2cfc, 0x6c4bf8, 0x6cab64, 0x6cae74];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 10);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 12);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 120);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 95);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 185);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 1818);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (116, 115, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 41);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 1818);
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
    assert_eq!(native_cases, 22);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":10, "new_selectors":12, "new_source_bytes":1818, "new_frames":120, "new_lists":95, "new_objects":185,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":10,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e119 current dataset/e118 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6c1e78_compact_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c1e78];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 4);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 118);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 13);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 36);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 524);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (118, 117, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 52);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.schedule = None; previous_tables.flight = None; previous_tables.burst = None; previous_tables.effects = None; previous_tables.grid_first = None; previous_tables.grid_second = None; previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 524);
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
    // The caller selects base 3 or 4, then conditionally adds two.
    for base in [3, 4] { for value in 0..=10 {
        raw.cpu.regs = [0; 16]; raw.cpu.regs[0] = value; raw.cpu.regs[5] = base;
        prefix(&mut raw, 0x08109688, 0x0810969a);
        assert_eq!(raw.cpu.regs[5], base + if [3, 5, 7].contains(&value) { 2 } else { 0 });
    } }
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
    assert_eq!(native_cases, 5);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":4, "new_source_bytes":524, "new_frames":118, "new_lists":13, "new_objects":36,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e120 current dataset/e119 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6bac28_schedule_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6bac28];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 9);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 9);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 9);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 13);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 172);
    new_spans.push((0x39e0c4, 0x39e124));
    assert_eq!(data.selector_tables.as_ref().unwrap().schedule.as_ref().unwrap().rows.len(), 12);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (119, 118, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 53);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.schedule = None; previous_tables.flight = None; previous_tables.burst = None; previous_tables.effects = None; previous_tables.grid_first = None; previous_tables.grid_second = None; previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 268);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    typed.bus.rom[0x39e0c4..0x39e124].fill(0xcd);
    for row in 0..12u32 {
        for (start, end) in [(0x080f0282, 0x080f0284), (0x080f029e, 0x080f02a4), (0x080f02ba, 0x080f02bc), (0x080f0230, 0x080f0238)] {
            for machine in [&mut raw, &mut typed] {
                machine.cpu.regs = [0; 16]; machine.cpu.regs[4] = 0x0839e0c4 + row * 8; machine.cpu.regs[3] = row * 8;
                prefix(machine, start, end);
            }
            assert_eq!(&raw.cpu.regs[..4], &typed.cpu.regs[..4]);
        }
    }
    drop(typed);
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
    assert_eq!(native_cases, 10);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":9, "new_source_bytes":268, "new_frames":9, "new_lists":9, "new_objects":13, "table_rows":12,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e121 current dataset/e120 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6babc4_flight_burst_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6babc4];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 6);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 6);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 6);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 6);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 100);
    new_spans.extend([(0x39dc50, 0x39dd40), (0x39f6cc, 0x39f724)]);
    assert_eq!(data.selector_tables.as_ref().unwrap().flight.as_ref().unwrap().rows.len(), 30);
    assert_eq!(data.selector_tables.as_ref().unwrap().burst.as_ref().unwrap().rows.len(), 11);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (120, 119, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 54);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.flight = None; previous_tables.burst = None; previous_tables.effects = None; previous_tables.grid_first = None; previous_tables.grid_second = None; previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 428);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    typed.bus.rom[0x39dc50..0x39dd40].fill(0xcd);
    typed.bus.rom[0x39f6cc..0x39f724].fill(0xcd);
    for (base, rows, cases) in [(0x0839dc50, 30u32, &[(0x080ee740, 0x080ee742), (0x080ee756, 0x080ee760), (0x080ee7a0, 0x080ee7a2)][..]), (0x0839f6cc, 11u32, &[(0x080f5e38, 0x080f5e3a), (0x080f5e3e, 0x080f5e40), (0x080f5e44, 0x080f5e4e)][..])] {
        for row in 0..rows { for &(start, end) in cases {
            for machine in [&mut raw, &mut typed] {
                machine.cpu.regs = [0; 16]; machine.cpu.regs[5] = base + row * 8; machine.cpu.regs[3] = base + row * 8;
                prefix(machine, start, end);
            }
            assert_eq!(&raw.cpu.regs[..4], &typed.cpu.regs[..4]);
        } }
    }
    drop(typed);
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
    assert_eq!(native_cases, 7);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":6, "new_source_bytes":428, "new_frames":6, "new_lists":6, "new_objects":6, "table_rows":41,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e122 current dataset/e121 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6bb574_effect_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6bb574];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 3);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 3);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 3);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 3);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 52);
    new_spans.extend([(0x39defc, 0x39df4d), (0x39df4e, 0x39df50), (0x39df5c, 0x39df94), (0x39df98, 0x39df9a)]);
    assert_eq!(data.selector_tables.as_ref().unwrap().effects.as_ref().unwrap().rows.len(), 10);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (121, 120, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 55);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.effects = None; previous_tables.grid_first = None; previous_tables.grid_second = None; previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 193);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
    let rows = &data.selector_tables.as_ref().unwrap().effects.as_ref().unwrap().rows;
    for (row, entry) in rows.iter().enumerate() {
        let mut cases = vec![(0x080ef096, 0x080ef098), (0x080ef09e, 0x080ef0a0)];
        if entry.kind != 1 { cases.extend([(0x080ef0b4, 0x080ef0b6), (0x080ef0b6, 0x080ef0be), (0x080ef110, 0x080ef112)]); }
        if entry.kind == 0 { cases.extend([(0x080ef108, 0x080ef110), (0x080ef11e, 0x080ef126), (0x080ef14a, 0x080ef14c)]); }
        for (start, end) in cases {
            for machine in [&mut raw, &mut typed] {
                machine.cpu.regs = [0; 16]; machine.cpu.regs[6] = 0x0839defc + row as u32 * 16;
                prefix(machine, start, end);
            }
            assert_eq!(&raw.cpu.regs[..4], &typed.cpu.regs[..4]);
        }
    }
    for at in 0x39defc..0x39df9c {
        if data.byte_at(at).is_none() { assert_eq!(typed.bus.read8(0x08000000 + at), rom[at as usize]); }
    }
    drop(typed);
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
    assert_eq!(native_cases, 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":3, "new_source_bytes":193, "new_frames":3, "new_lists":3, "new_objects":3, "table_rows":10,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}



#[test]
#[ignore = "private EUR source/e124 current dataset/e123 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6c316c_initial_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c316c];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 3);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 39);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 10);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 9);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 188);


    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (123, 122, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 57);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 188);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
    let record = 0x02007d00;
    for selector in 0..3 {
        for machine in [&mut raw, &mut typed] {
            machine.bus.write32(record, 0x086c316c); machine.bus.write16(record + 4, selector);
            machine.cpu.regs = [0; 16]; machine.cpu.regs[6] = record; machine.cpu.regs[13] = 0x03007c00;
            prefix(machine, 0x080d89c6, 0x080d89e6);
        }
        assert_eq!(&raw.cpu.regs[..4], &typed.cpu.regs[..4]);
        assert_eq!(raw.bus.read32(0x03007c28), typed.bus.read32(0x03007c28));
        assert_eq!(raw.bus.read32(0x03007c2c), typed.bus.read32(0x03007c2c));
        for machine in [&mut raw, &mut typed] { machine.cpu.regs[6] = record; prefix(machine, 0x080d8a10, 0x080d8a12); }
        assert_eq!(raw.cpu.regs[2], typed.cpu.regs[2]);
        assert_eq!(typed.cpu.regs[2], u32::from(selector));
    }
    drop(typed);
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
    assert_eq!(native_cases, 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":3, "new_source_bytes":188, "new_frames":39, "new_lists":10, "new_objects":9, "initial_records":3,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}




#[test]
#[ignore = "private EUR source/current dataset/e126 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6ca580_nibble_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6ca580];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 7);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 7);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 7);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 7);
    let new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 116);


    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (126, 125, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 62);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let previous_tables = data.selector_tables.clone().unwrap();
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 116);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
    let record = 0x02007d00;
    for selector in 0..16u32 {
        for machine in [&mut raw, &mut typed] {
            machine.cpu.regs = [0; 16]; machine.cpu.regs[5] = record; machine.cpu.regs[2] = (selector << 8) | 13;
            prefix(machine, 0x080e5f3c, 0x080e5f46);
            assert_eq!(machine.bus.read8(record + 2), selector as u8);
            machine.cpu.regs = [0; 16]; machine.cpu.regs[5] = record;
            prefix(machine, 0x080e53ac, 0x080e53b0);
        }
        assert_eq!(raw.cpu.regs[0], typed.cpu.regs[0]);
        assert_eq!(raw.cpu.regs[1], typed.cpu.regs[1]);
    }
    assert_eq!(data.byte_at(0x6ca5f6), None);
    assert_eq!(typed.bus.read8(0x086ca5f6), rom[0x6ca5f6]);
    drop(typed);
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
    for bank in &banks { for selector in [0u16, 3, 6, 0x100] {
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
    assert_eq!(native_cases, 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":7, "new_source_bytes":116, "new_frames":7, "new_lists":7, "new_objects":7, "nibble_values":16,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}



#[test]
#[ignore = "private EUR source/e125 current dataset/e124 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6cb278_secondary_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6cb278];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 4);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 44);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 14);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 154);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 1088);
    new_spans.push((0x3ab98c, 0x3ab990));


    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (124, 123, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 58);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 1092);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
    let record = 0x02007d00;
    let rows = &data.selector_tables.as_ref().unwrap().script_lookup.as_ref().unwrap().rows;
    for (index, &expected) in rows.iter().enumerate() {
        for machine in [&mut raw, &mut typed] {
            machine.bus.write8(record + 2, index as u8);
            machine.cpu.regs = [0; 16]; machine.cpu.regs[5] = record;
            prefix(machine, 0x0811eb86, 0x0811eb8e);
        }
        assert_eq!(raw.cpu.regs[1], u32::from(expected));
        assert_eq!(raw.cpu.regs[1], typed.cpu.regs[1]);
    }
    assert_eq!(data.byte_at(0x3ab990), None);
    assert_eq!(typed.bus.read8(0x083ab990), rom[0x3ab990]);
    drop(typed);
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
    assert_eq!(native_cases, 5);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":4, "new_source_bytes":1092, "new_frames":44, "new_lists":14, "new_objects":154, "initial_records":4,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}



#[test]
#[ignore = "private EUR source/e126 current dataset/e125 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6cca_scene_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6cca8c, 0x6ccae4, 0x6ccdec];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 3);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 27);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 72);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 72);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 533);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 3750);
    new_spans.extend((0..20).map(|index| (0x3ac944 + index * 8, 0x3ac94a + index * 8)));


    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (125, 124, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 59);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 3870);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
    let rows = &data.selector_tables.as_ref().unwrap().scene.as_ref().unwrap().rows;
    assert_eq!(rows.len(), 20);
    for (index, row) in rows.iter().enumerate() {
        let address = 0x083ac944 + index as u32 * 8;
        for (start, end) in [(0x08123f20, 0x08123f2e), (0x08123f4c, 0x08123f5c)] {
            for machine in [&mut raw, &mut typed] {
                machine.cpu.regs = [0; 16]; machine.cpu.regs[5] = address; prefix(machine, start, end);
            }
            assert_eq!(&raw.cpu.regs[..4], &typed.cpu.regs[..4]);
        }
        assert_eq!(data.byte_at(0x3ac944 + index as u32 * 8), Some(row.first_selector));
        for offset in [6, 7] {
            let at = 0x3ac944 + index as u32 * 8 + offset;
            assert_eq!(data.byte_at(at), None);
            assert_eq!(typed.bus.read8(0x08000000 + at), rom[at as usize]);
        }
    }
    drop(typed);
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
    for bank in &banks {
        let mut selectors: Vec<_> = [0, bank.sequences.len() / 2, bank.sequences.len() - 1].into_iter().map(|index| bank.sequences[index].sequence.selector).collect();
        selectors.sort(); selectors.dedup(); selectors.push(0x100);
        for selector in selectors {
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
    assert_eq!(native_cases, 11);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":3, "new_selectors":27, "new_source_bytes":3870, "new_frames":72, "new_lists":72, "new_objects":533, "initial_records":20,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":3,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}


#[test]
#[ignore = "private EUR source/e123 current dataset/e122 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6cac40_grid_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6cac40];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 8);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 8);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 8);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 8);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 132);
    for (start, bytes, _) in data.source_spans().unwrap() {
        let end = start + bytes.len() as u32;
        for base in [0x3a9abc, 0x3aae44] {
            let a = start.max(base); let z = end.min(base + 24 * 16);
            if a < z { new_spans.push((a, z)); }
        }
    }
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 726);
    let tables = data.selector_tables.as_ref().unwrap();
    assert_eq!(tables.grid_first.as_ref().unwrap().rows.len(), 24);
    assert_eq!(tables.grid_second.as_ref().unwrap().rows.len(), 24);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (122, 121, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 56);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap(); previous_tables.grid_first = None; previous_tables.grid_second = None; previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 726);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
    for (base, head, word, flag, signed_flag) in [
        (0x3a9abc, 0x0811a092, 0x0811a0f4, 0x0811a13e, 0x0811a144),
        (0x3aae44, 0x0811b78a, 0x0811b7ec, 0x0811b836, 0x0811b83c),
    ] {
        let rows = if base == 0x3a9abc { &data.selector_tables.as_ref().unwrap().grid_first.as_ref().unwrap().rows } else { &data.selector_tables.as_ref().unwrap().grid_second.as_ref().unwrap().rows };
        for (index, row) in rows.iter().enumerate() {
            let address = 0x08000000 + base + index as u32 * 16;
            for machine in [&mut raw, &mut typed] { machine.cpu.regs = [0; 16]; machine.cpu.regs[0] = address; prefix(machine, head, head + 10); }
            assert_eq!(&raw.cpu.regs[..8], &typed.cpu.regs[..8]);
            for machine in [&mut raw, &mut typed] { machine.cpu.regs = [0; 16]; machine.cpu.regs[10] = index as u32 * 16; prefix(machine, word, word + 8); }
            assert_eq!(raw.cpu.regs[0], typed.cpu.regs[0]);
            if row.flag.is_some() {
                for machine in [&mut raw, &mut typed] { machine.cpu.regs = [0; 16]; machine.cpu.regs[1] = address; prefix(machine, flag, flag + 2); }
                assert_eq!(raw.cpu.regs[0], typed.cpu.regs[0]);
            }
            if row.signed_flag.is_some() {
                for machine in [&mut raw, &mut typed] { machine.cpu.regs = [0; 16]; machine.cpu.regs[1] = address; prefix(machine, signed_flag, signed_flag + 4); }
                assert_eq!(raw.cpu.regs[0], typed.cpu.regs[0]);
            }
        }
        for at in base..base + 24 * 16 { if data.byte_at(at).is_none() { assert_eq!(typed.bus.read8(0x08000000 + at), rom[at as usize]); } }
    }
    drop(typed);
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
    assert_eq!(native_cases, 9);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":8, "new_source_bytes":726, "new_frames":8, "new_lists":8, "new_objects":8, "table_rows":48,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e118 current dataset/e117 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6c0614_timer_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c0614];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 6);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 42);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 19);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 61);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 554);
    new_spans.push((0x39e164, 0x39e214));
    assert_eq!(data.selector_tables.as_ref().unwrap().timer_positions.as_ref().unwrap().rows.len(), 44);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (117, 116, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 51);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    let mut previous_tables = data.selector_tables.clone().unwrap();
    previous_tables.timer_positions = None; previous_tables.schedule = None; previous_tables.flight = None; previous_tables.burst = None; previous_tables.effects = None; previous_tables.grid_first = None; previous_tables.grid_second = None; previous_tables.script_lookup = None; previous_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&previous_tables));
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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 730);
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
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    typed.bus.rom[0x39e164..0x39e214].fill(0xcd);
    for row in 0..44u32 {
        for m in [&mut raw, &mut typed] {
            m.cpu.regs = [0; 16];
            m.cpu.regs[4] = 0x0839e164 + row * 4;
            m.cpu.cpsr = 0xbf;
            prefix(m, 0x080f0820, 0x080f0826);
        }
        assert_eq!(&raw.cpu.regs[1..=3], &typed.cpu.regs[1..=3]);
        for index in 0..3 { assert_eq!(typed.cpu.regs[index + 1], u32::from(rom[0x39e164 + row as usize * 4 + index + 1])); }
    }
    drop(typed);
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
    assert_eq!(native_cases, 7);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":6, "new_source_bytes":730, "new_frames":42, "new_lists":19, "new_objects":61, "table_rows":44,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e115 current dataset/e114 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6c95d4_motion_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c95d4];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 3);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 33);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 33);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 33);
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 412);
    new_spans.extend((0..16).map(|i| (0x3a8610 + i * 12, 0x3a861a + i * 12)));
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 572);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (114, 113, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 39);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    assert_eq!(current.compact_obj_catalog.as_ref().unwrap(), data);
    let mut old_tables = data.selector_tables.clone().unwrap(); old_tables.motion_positions = None; old_tables.timer_positions = None; old_tables.schedule = None; old_tables.flight = None; old_tables.burst = None; old_tables.effects = None; old_tables.grid_first = None; old_tables.grid_second = None; old_tables.script_lookup = None; old_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&old_tables));
    let table = data.selector_tables.as_ref().unwrap().motion_positions.as_ref().unwrap();
    for (i, row) in table.rows.iter().enumerate() {
        let at = 0x3a8610 + 12 * i;
        assert_eq!((row.threshold, row.selector, row.x, row.y, row.dx, row.dy, row.x_rate, row.y_rate),
            (rom[at], rom[at+1], i16::from_le_bytes(rom[at+2..at+4].try_into().unwrap()), i16::from_le_bytes(rom[at+4..at+6].try_into().unwrap()), rom[at+6] as i8, rom[at+7] as i8, rom[at+8], rom[at+9]));
    }

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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 572);
    let before_unknown = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for i in 0..16 { for offset in [10, 11] {
        let at = 0x3a8610 + i * 12 + offset;
        assert_eq!(data.byte_at(at), None);
        assert_eq!(machine.bus.read8(0x08000000 + at), rom[at as usize]);
    } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes, before_unknown);
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

    // Exercise the original guarded table reader and selector copy in each row.
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    let owner = typed.bus.rom_resources().unwrap().clone();
    let baseline = typed.save_state().unwrap();
    for row in 0..16usize {
        let at = 0x3a8610 + 12 * row;
        for accepted in [true, false] {
            let threshold = rom[at];
            if !accepted && threshold == 0 { continue; }
            let mut original = crate::make_machine(rom.clone(), None);
            for m in [&mut original, &mut typed] {
                m.load_state(&baseline).unwrap(); m.bus.ffta_sprites.enabled = false;
                m.cpu.regs = [0; 16]; m.cpu.regs[3] = state; m.cpu.regs[6] = state + 0x100;
                m.cpu.regs[7] = 0x08000000 + at as u32;
                m.bus.write16(state, if accepted { u16::from(threshold) } else { u16::from(threshold) - 1 });
            }
            for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
            let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
            let end = if accepted { 0x08116e0a } else { 0x08116eac };
            prefix(&mut original, 0x08116e00, end); prefix(&mut typed, 0x08116e00, end);
            if accepted {
                prefix(&mut original, 0x08116e12, 0x08116e2c); prefix(&mut typed, 0x08116e12, 0x08116e2c);
                prefix(&mut original, 0x08116e6a, 0x08116e74); prefix(&mut typed, 0x08116e6a, 0x08116e74);
            }
            assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, if accepted { 10 } else { 1 });
            typed.bus.rom.copy_from_slice(&rom);
            assert_eq!(typed.save_state().unwrap(), original.save_state().unwrap(), "motion row {row}, accepted {accepted}");
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
    assert_eq!(native_cases, 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":3, "new_source_bytes":572, "new_frames":33, "new_lists":33, "new_objects":33,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
        "counter_oracle_from_actual_owned_intersections":true, "full_state_clock":true,
        "coherent_rejection_in_memory":true, "dataset_copies_created":0, "old_reader_cases_repeated":false,
        "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR source/e114 current dataset/e113 baseline/native prefix; Wine only, Linux/USA pending"]
fn european_6c992c_table_catalog_matches_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let selected = [0x6c992c];
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let banks: Vec<_> = data.banks.iter().filter(|bank| selected.contains(&bank.root)).collect();
    assert_eq!(banks.len(), 1);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 8);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 40);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 9);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 8);
    assert!(banks[0].lists.iter().find(|entry| entry.index == 1).unwrap().list.objects.is_empty());
    let mut new_spans: Vec<_> = model::BANK_SPECS.iter().filter(|spec| selected.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 200);
    new_spans.extend((0..20).map(|i| (0x3a86d0 + i * 8, 0x3a86d6 + i * 8)));
    assert_eq!(new_spans.iter().map(|(a, z)| z - a).sum::<u32>(), 320);

    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (113, 112, 100));
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for (updated, previous) in current.manifest.resources[..99].iter().zip(&old_manifest.resources[..99]) { super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, previous); }
    // Read the accepted older JSON without applying the new complete bank-set
    // validator to its historical projection. Its protected file is unchanged.
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 38);
    assert_eq!(data.banks.iter().filter(|bank| !selected.contains(&bank.root)).count(), model::BANK_SPECS.len() - selected.len());
    for bank in &old.banks { assert_previous_bank_preserved(bank, data.banks.iter().find(|new| new.root == bank.root).unwrap()); }
    assert_eq!(current.compact_obj_catalog.as_ref().unwrap(), data);
    let mut old_tables = data.selector_tables.clone().unwrap(); old_tables.timed_positions = None; old_tables.motion_positions = None; old_tables.timer_positions = None; old_tables.schedule = None; old_tables.flight = None; old_tables.burst = None; old_tables.effects = None; old_tables.grid_first = None; old_tables.grid_second = None; old_tables.script_lookup = None; old_tables.scene = None;
    assert_eq!(old.selector_tables.as_ref(), Some(&old_tables));
    let table = data.selector_tables.as_ref().unwrap().timed_positions.as_ref().unwrap();
    for (i, row) in table.rows.iter().enumerate() {
        let at = 0x3a86d0 + 8 * i;
        assert_eq!((row.match_value, row.selector, row.x, row.y),
            (rom[at], rom[at + 1], i16::from_le_bytes(rom[at + 2..at + 4].try_into().unwrap()), i16::from_le_bytes(rom[at + 4..at + 6].try_into().unwrap())));
    }

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
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 320);
    let before_unknown = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for i in 0..20 { for offset in [6, 7] {
        let at = 0x3a86d0 + i * 8 + offset;
        assert_eq!(data.byte_at(at), None);
        assert_eq!(machine.bus.read8(0x08000000 + at), rom[at as usize]);
    } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes, before_unknown);
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

    // Execute the original threshold/field reader against raw and typed bytes.
    let mut typed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut typed, &dataset).unwrap();
    let typed_owner = typed.bus.rom_resources().unwrap().clone();
    let baseline = typed.save_state().unwrap();
    for row in 0..20usize {
        let addr = 0x3a86d0 + row * 8;
        let threshold = rom[addr];
        for accepted in [true, false] {
            if !accepted && threshold == 0 { continue; }
            let mut original = crate::make_machine(rom.clone(), None);
            for m in [&mut original, &mut typed] {
                m.load_state(&baseline).unwrap();
                m.bus.ffta_sprites.enabled = false;
                m.cpu.regs = [0; 16]; m.cpu.regs[3] = 0x08000000 + addr as u32; m.cpu.regs[5] = state;
                m.bus.write16(state, if accepted { u16::from(threshold) } else { u16::from(threshold) - 1 });
            }
            for &(a, z) in &new_spans { typed.bus.rom[a as usize..z as usize].fill(0xcd); }
            let end = if accepted { 0x0811731c } else { 0x08117342 };
            let before = typed_owner.compact_obj_catalog_stats().unwrap().source_bytes;
            prefix(&mut original, 0x08117308, end); prefix(&mut typed, 0x08117308, end);
            assert_eq!(typed_owner.compact_obj_catalog_stats().unwrap().source_bytes - before, if accepted { 6 } else { 1 });
            typed.bus.rom.copy_from_slice(&rom);
            assert_eq!(typed.save_state().unwrap(), original.save_state().unwrap(), "table row {row}, accepted {accepted}");
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
    assert_eq!(native_cases, 9);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":101, "previous_resources_unchanged":98, "previous_obj_projection_preserved":true, "previous_banks_unchanged":old.banks.iter().filter(|bank| data.banks.iter().find(|new| new.root == bank.root) == Some(*bank)).count(), "previous_banks_preserved":old.banks.len(),
        "new_banks":1, "new_selectors":8, "new_source_bytes":320, "new_frames":40, "new_lists":9, "new_objects":8,
        "native_count_cases":native_cases, "typed_native_bytes":served, "out_of_projection_cases":1,
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
    assert_eq!(data.banks.len(), 63);
    assert_eq!(data.banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 207);
    assert_eq!(data.banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 949);
    assert_eq!(data.banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 607);
    assert_eq!(data.banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 2171);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 328);
    assert_eq!(spans.iter().map(|(_, bytes, _)| bytes.len()).sum::<usize>(), 20389);
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
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    let current_resources = current.manifest.resources.len();
    for entry in &old.resources[..99] { let updated = current.manifest.resources.iter().find(|resource| resource.id == entry.id).unwrap(); super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, entry); }
    assert_eq!(current.compact_obj_catalog.as_ref().unwrap(), data);
    assert_eq!(current.manifest.resources[99].id, COMPACT_OBJ_CATALOG_ID);
    let mut machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    let initial = machine.save_state().unwrap(); let owner = machine.bus.rom_resources().unwrap().clone();
    for &(start, end) in model::SOURCE_SPANS { machine.bus.rom[start as usize..end as usize].fill(0xcd); }
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for &(start, end) in model::SOURCE_SPANS { for at in start..end { assert_eq!(machine.bus.read8(0x0c000000 + at), rom[at as usize]); } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before,
        spans.iter().map(|(_, bytes, _)| bytes.len() as u64).sum::<u64>());
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
    for bank in &data.banks { assert!(!bank.sequences.iter().any(|entry| entry.sequence.selector == 0x100));
      for selector in bank.sequences.iter().map(|entry| entry.sequence.selector).chain([0x100]) {
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
        "status":"PASS", "resources":current_resources, "previous_unchanged":old.resources[..99].len()-1, "previous_obj_projection_preserved":true,
        "banks":data.banks.len(), "selectors":data.banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(),
        "source_bytes":spans.iter().map(|(_, bytes, _)| bytes.len()).sum::<usize>(), "spans":spans.len(),
        "frames":data.banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(),
        "lists":data.banks.iter().map(|bank| bank.lists.len()).sum::<usize>(),
        "objects":data.banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(),
        "native_count_cases":native_cases, "typed_native_bytes":served,
        "out_of_projection_cases":data.banks.len(), "counter_oracle_from_actual_owned_intersections":true,
        "full_state_clock":true, "coherent_rejection_in_memory":true, "dataset_copies_created":0,
        "ordinary_use":"pending", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR ROM and current dataset; Wine only, Linux/USA pending"]
fn european_compact_catalog_failed_disk_replacement_preserves_owner() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let committed = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let replacement = root.join("replacement");
    fs::create_dir(&replacement).unwrap();
    for entry in fs::read_dir(&committed).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), replacement.join(entry.file_name())).unwrap();
    }
    let original = read_dataset(&replacement).unwrap();
    assert_eq!((original.manifest.extractor_revision, original.manifest.decoder_revision), (130, 129));
    let index = original.manifest.resources.iter().position(|resource| resource.id == COMPACT_OBJ_CATALOG_ID).unwrap();
    let manifest_bytes = fs::read(replacement.join(MANIFEST_FILE)).unwrap();
    let catalog_bytes = fs::read(replacement.join(COMPACT_OBJ_CATALOG_FILE)).unwrap();
    let mut machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut machine, &committed).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let initial = machine.save_state().unwrap();

    let missing = root.join("catalog-held-aside.json");
    fs::rename(replacement.join(COMPACT_OBJ_CATALOG_FILE), &missing).unwrap();
    assert!(attach_from_directory(&mut machine, &replacement).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(machine.save_state().unwrap(), initial);
    fs::rename(&missing, replacement.join(COMPACT_OBJ_CATALOG_FILE)).unwrap();

    let mut modified = original.compact_obj_catalog.as_ref().unwrap().clone();
    modified.banks[0].sequences[0].sequence.frames[0].duration ^= 1;
    let forged_bytes = encode_compact_obj_catalog(&modified).unwrap();
    let mut forged_manifest = original.manifest.clone();
    forged_manifest.resources[index].sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    fs::write(replacement.join(COMPACT_OBJ_CATALOG_FILE), &forged_bytes).unwrap();
    fs::write(replacement.join(MANIFEST_FILE), toml::to_string_pretty(&forged_manifest).unwrap()).unwrap();
    // Parsing succeeds and the manifest hash matches the changed disk bytes;
    // source authentication must reject them before machine publication.
    let parsed = read_dataset(&replacement).unwrap();
    assert_eq!(parsed.compact_obj_catalog.as_ref().unwrap(), &modified);
    assert_eq!(parsed.manifest.resources[index].sha256,
        gba_core::ffta_rom::digest(&fs::read(replacement.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()));
    drop(parsed);
    assert!(attach_from_directory(&mut machine, &replacement).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(machine.save_state().unwrap(), initial);
    fs::write(replacement.join(COMPACT_OBJ_CATALOG_FILE), catalog_bytes).unwrap();
    fs::write(replacement.join(MANIFEST_FILE), manifest_bytes).unwrap();

    // The existing owner continues to serve catalog bytes after both failed
    // replacements, and the untouched committed generation can be attached.
    let offset = 0x6c4c50usize;
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    machine.bus.rom[offset] ^= 0xff;
    assert_eq!(machine.bus.read8(0x08000000 + offset as u32), rom[offset]);
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes, before + 1);
    machine.bus.rom[offset] = rom[offset];
    assert_eq!(machine.save_state().unwrap(), initial);
    let mut fresh = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut fresh, &committed).unwrap();
    let fresh_owner = fresh.bus.rom_resources().unwrap().clone();
    let before = fresh_owner.compact_obj_catalog_stats().unwrap().source_bytes;
    fresh.bus.rom[offset] ^= 0xff;
    assert_eq!(fresh.bus.read8(0x08000000 + offset as u32), rom[offset]);
    assert_eq!(fresh_owner.compact_obj_catalog_stats().unwrap().source_bytes, before + 1);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[130,129], "missing_catalog_rejected":true,
        "coherent_disk_edit_parsed_and_rejected":true, "active_owner_unchanged":true,
        "machine_state_unchanged":true, "active_owner_reads":true,
        "committed_generation_reattached_and_read":true, "Linux":"pending",
        "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
