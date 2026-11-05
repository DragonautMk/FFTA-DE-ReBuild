//! Source and conditional native UI readers, reusing one prepared dataset.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::{ui_context_templates as model, RuntimeResources};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn interpreter_until(machine: &mut Machine, end: u32, bound: usize) {
    let mut steps = 0;
    while machine.cpu.regs[15] != end {
        assert!(steps < bound, "original prefix did not return");
        machine.bus.frame_ready = false;
        machine.step();
        steps += 1;
    }
}

fn assert_context_owner_reads(machine: &mut Machine, rom: &[u8]) {
    let owner = machine.bus.rom_resources().unwrap().clone();
    let state = machine.save_state().unwrap();
    let before = owner.ui_context_templates_stats().unwrap().source_bytes;
    machine.bus.rom[model::SOURCE_OFFSET as usize..model::END_OFFSET as usize].fill(0xcd);
    for offset in model::SOURCE_OFFSET..model::END_OFFSET {
        assert_eq!(machine.bus.read8(0x08000000 + offset), rom[offset as usize]);
    }
    assert_eq!(owner.ui_context_templates_stats().unwrap().source_bytes - before, 220);
    machine.bus.rom.copy_from_slice(rom);
    assert_eq!(machine.save_state().unwrap(), state);
}

#[test]
#[ignore = "private EUR ROM and current dataset; Wine only, Linux/USA pending"]
fn european_ui_context_unavailable_copy_load_undo_restart() {
    let root = PathBuf::from(std::env::var_os("FFTA_UI_CONTEXT_TEMPLATES_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let committed = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let copy = root.join("selected dataset");
    fs::create_dir(&copy).unwrap();
    for entry in fs::read_dir(&committed).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), copy.join(entry.file_name())).unwrap();
    }
    let dataset = read_dataset(&copy).unwrap();
    assert_eq!((dataset.manifest.extractor_revision, dataset.manifest.decoder_revision), (130, 129));
    assert!(dataset.ui_context_templates.is_some());
    drop(dataset);
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &copy).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    assert_eq!(machine.save_state().unwrap(), boot);

    let private = root.join("synthetic state files");
    fs::create_dir(&private).unwrap();
    let save = private.join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    machine.cpu.regs[4] = 0x12345678;
    machine.bus.write8(0x02001000, 0x5a);
    let changed = machine.save_state().unwrap();
    assert_ne!(changed, boot);
    let unavailable = root.join("unavailable dataset");
    fs::rename(&copy, &unavailable).unwrap();
    assert!(attach_from_directory(&mut machine, &copy).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(machine.save_state().unwrap(), changed);

    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.save_state().unwrap(), boot);
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_context_owner_reads(&mut machine, &rom);
    slots.undo(&mut machine).unwrap();
    assert_eq!(machine.save_state().unwrap(), changed);
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_context_owner_reads(&mut machine, &rom);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.save_state().unwrap(), boot);
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_context_owner_reads(&mut machine, &rom);

    fs::rename(&unavailable, &copy).unwrap();
    let mut fresh = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut fresh, &copy).unwrap();
    assert!(!std::sync::Arc::ptr_eq(fresh.bus.rom_resources().unwrap(), &owner));
    assert_eq!(fresh.save_state().unwrap(), boot);
    assert_context_owner_reads(&mut fresh, &rom);
    drop(slots);
    fs::remove_dir_all(private).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[130,129], "unavailable_copy_rejected":true,
        "load_undo_restart_full_state_exact":true, "owner_retained":true,
        "typed_bytes_per_stage":220, "restored_copy_owner_reconstructed":true,
        "synthetic_state_files_removed":true, "Linux":"pending", "USA":"pending",
        "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR ROM, prepared current/prior dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_ui_context_templates_match_source_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_UI_CONTEXT_TEMPLATES_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_UI_CONTEXT_TEMPLATES_PRIOR").unwrap());
    let catalog = model::Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.verify_source(&rom).unwrap();
    assert_eq!(data.source_bytes().unwrap(), rom[model::SOURCE_OFFSET as usize..model::END_OFFSET as usize]);
    assert_eq!(data.records.len(), 11);
    for at in model::SOURCE_OFFSET..model::END_OFFSET {
        assert_eq!(data.byte_at(at), Some(rom[at as usize]));
    }
    assert_eq!(data.byte_at(model::SOURCE_OFFSET - 1), None);
    assert_eq!(data.byte_at(model::END_OFFSET), None);
    let encoded = encode_ui_context_templates(data).unwrap();
    assert_eq!(parse_ui_context_templates(&encoded).unwrap(), *data);
    let mut changed = data.clone();
    changed.records[0].flags ^= 1;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    changed.records.pop();
    assert!(changed.validate().is_err());
    assert!(serde_json::from_str::<model::UiContextTemplates>("{\"records\":[],\"extra\":true}").is_err());

    let old: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    let current = read_dataset(&dataset).unwrap();
    let resource_count = current.manifest.resources.len();
    assert_eq!(old.resources.len(), 98);
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (130, 129, 101));
    for resource in &old.resources {
        let updated = current.manifest.resources.iter().find(|entry| entry.id == resource.id).unwrap();
        super::conditional_marker_runtime_tests::assert_previous_resource_metadata(updated, resource);
    }
    assert_eq!(current.manifest.resources[98].id, UI_CONTEXT_TEMPLATES_ID);
    assert_eq!(current.ui_context_templates.as_ref().unwrap(), data);

    let mut machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    let initial = machine.save_state().unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let before = owner.ui_context_templates_stats().unwrap();
    machine.bus.rom[model::SOURCE_OFFSET as usize..model::END_OFFSET as usize].fill(0xcd);
    for at in model::SOURCE_OFFSET..model::END_OFFSET {
        assert_eq!(machine.bus.read8(0x0c000000 + at), rom[at as usize]);
    }
    let after = owner.ui_context_templates_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 220);
    assert_eq!(after.coordinate_bytes - before.coordinate_bytes, 22);
    assert_eq!(after.count_bytes - before.count_bytes, 22);
    assert_eq!(after.flag_bytes - before.flag_bytes, 11);
    assert_eq!(after.opaque_bytes - before.opaque_bytes, 165);
    for edge in [model::SOURCE_OFFSET, model::END_OFFSET] {
        for delta in [-3i32, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta) as usize;
            let ptr = &mut machine as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    machine.bus.rom.copy_from_slice(&rom);
    assert_eq!(machine.save_state().unwrap(), initial);

    // Coherent corruption is formed in memory; the canonical generation stays
    // read-only and no complete dataset copy is required for this rejection.
    let mut bad = read_dataset(&dataset).unwrap();
    bad.ui_context_templates.as_mut().unwrap().records[0].flags ^= 1;
    let bytes = encode_ui_context_templates(bad.ui_context_templates.as_ref().unwrap()).unwrap();
    bad.manifest.resources[98].sha256 = gba_core::ffta_rom::digest(&bytes);
    bad.ui_context_templates_bytes = Some(bytes);
    assert!(RuntimeResources::from_base_files(&rom, bad.files()).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(machine.save_state().unwrap(), initial);
    drop(bad);
    drop(current);
    drop(machine);

    // Test the original selector's complete u8 input domain. This is source
    // verification; production still executes the original guest selector.
    let mapping = [0u32, 6, 7, 1, 2, 3, 3, 3, 0, 1, 4, 5, 8, 9, 10];
    let mut raw = crate::make_machine(rom.clone(), None);
    for category in 0u32..=255 {
        raw.bus.write32(0x0200f438, 0x0203b000);
        raw.bus.write8(0x0203b004, category as u8);
        raw.cpu.regs = [0; 16];
        raw.cpu.regs[13] = 0x03007c00;
        raw.cpu.regs[14] = 0x03007ff1;
        raw.cpu.regs[15] = 0x08028f4c;
        raw.cpu.cpsr = 0xbf;
        interpreter_until(&mut raw, 0x03007ff0, 64);
        let expected = if (1..=15).contains(&category) { mapping[category as usize - 1] } else { 0 };
        assert_eq!(raw.cpu.regs[0], expected, "category {category}");
    }

    // Copy only the authenticated memcpy helper as the original initializer
    // does. Targeted conditional prefixes are not ordinary-gameplay claims.
    raw.bus.iwram[0x5ee8..0x5f28].copy_from_slice(&rom[0xd968a8..0xd968e8]);
    let mut visited: BTreeSet<u32> = (0x08028f4c..0x08028ffe).step_by(2).collect();
    visited.extend((0x0802944a..0x0802948e).step_by(2));
    visited.extend((0x03005ee8..0x03005f28).step_by(2));
    visited.insert(0x08145c14);
    let view = View { rom: &rom, ewram: None, iwram: Some(&raw.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("ui-context-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &dataset).unwrap();
    let baseline = raw.save_state().unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    let mut served = 0;
    let cases = [(1u8, 0usize), (4, 1), (5, 2), (6, 3), (11, 4), (12, 5), (2, 6), (3, 7), (13, 8), (14, 9), (15, 10)];
    for (category, index) in cases {
        for coordinates in [false, true] {
            for m in [&mut raw, &mut native] {
                m.load_state(&baseline).unwrap();
                m.bus.ffta_sprites.enabled = false;
                m.bus.write32(0x0200f438, 0x0203b000);
                m.bus.write8(0x0203b004, category);
                m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                m.cpu.regs[0] = if coordinates { index as u32 } else { 0x02018000 };
                m.cpu.regs[8] = 0x02018000;
                m.cpu.regs[13] = 0x03007c00;
                m.cpu.regs[14] = 0x03007ff1;
                m.cpu.regs[15] = if coordinates { 0x0802944a } else { 0x08028fdc };
                m.cpu.cpsr = 0xbf;
            }
            native.bus.rom[model::SOURCE_OFFSET as usize..model::END_OFFSET as usize].fill(0xcd);
            let end = if coordinates { 0x0802948e } else { 0x08028ffe };
            interpreter_until(&mut raw, end, 256);
            let before = owner.ui_context_templates_stats().unwrap().source_bytes;
            let mut steps = 0;
            while native.cpu.regs[15] != end {
                assert!(steps < 256, "native UI prefix did not return");
                native.bus.frame_ready = false;
                table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void);
                steps += 1;
            }
            let used = owner.ui_context_templates_stats().unwrap().source_bytes - before;
            assert_eq!(used, if coordinates { 8 } else { 20 });
            served += used;
            native.bus.rom.copy_from_slice(&rom);
            assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap(), "template {index}, coordinate prefix {coordinates}");
            if !coordinates {
                assert_eq!(&raw.bus.ewram[0x18080..0x18094], &data.records[index].source_bytes());
            }
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":resource_count, "previous_unchanged":old.resources.len()-1,"previous_obj_projection_preserved":true, "source_bytes":220,
        "records":11, "selector_categories":256, "native_copy_cases":11,
        "native_coordinate_cases":11, "typed_native_bytes":served,
        "coherent_edit_rejected_in_memory":true, "full_state_clock":true,
        "dataset_copies_created":0, "ordinary_use":"pending",
        "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
