//! EUR rectangle pointer aliases and source-backed original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn spans() -> impl Iterator<Item = (usize, usize)> {
    std::iter::once((model::ADDITIONAL_RECTANGLE_POINTER_OFFSET as usize, model::ADDITIONAL_RECTANGLE_POINTER_COUNT * 4))
        .chain(model::ADDITIONAL_RECTANGLE_SPECS.iter().map(|&(offset, _)| (offset as usize, 26)))
}

fn poison(machine: &mut Machine, rom: &[u8], invert: bool) {
    for (start, length) in spans() {
        for at in start..start + length { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
}

fn verify_live_owner(machine: &mut Machine, rom: &[u8]) {
    let owner = machine.bus.rom_resources().unwrap().clone();
    let before = owner.observed_tilemap_stats().unwrap();
    poison(machine, rom, true);
    assert_eq!(machine.bus.read32(0x083a4d40), 0x08ae7908);
    assert_eq!(machine.bus.read8(0x08ae7909), rom[0xae7909]);
    assert_eq!(machine.bus.read16(0x08ae790a), u16::from_le_bytes(rom[0xae790a..0xae790c].try_into().unwrap()));
    poison(machine, rom, false);
    let after = owner.observed_tilemap_stats().unwrap();
    assert_eq!(after.additional_rectangle_pointer_bytes - before.additional_rectangle_pointer_bytes, 4);
    assert_eq!(after.additional_rectangle_bytes[0] - before.additional_rectangle_bytes[0], 3);
}

#[test]
#[ignore = "exact EUR ROM, E202 dataset and private Wine scratch; Linux/USA pending"]
fn european_additional_rectangles_survive_save_lifecycle_and_missing_files() {
    let root = PathBuf::from(std::env::var_os("FFTA_EXTRA_RECTANGLES_ROOT").unwrap());
    let selected = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_EXTRA_RECTANGLES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old = parse_observed_tilemap(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    assert!(old.additional_rectangles.is_none() && old.additional_rectangle_pointers.is_none());
    let copied = root.join("lifecycle dataset"); let unavailable = root.join("unavailable dataset");
    fs::create_dir(&copied).unwrap();
    for entry in fs::read_dir(&selected).unwrap() {
        let entry = entry.unwrap(); assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), copied.join(entry.file_name())).unwrap();
    }
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &copied).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    assert_eq!(machine.save_state().unwrap(), boot);
    verify_live_owner(&mut machine, &rom);
    fs::rename(&copied, &unavailable).unwrap();
    assert!(attach_from_directory(&mut machine, &copied).is_err());
    assert_eq!(machine.save_state().unwrap(), boot);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    let save = root.join("private.sav");
    let mut slots = crate::save_state::Slots::new(&save, &rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    machine.cpu.regs[4] = 99; let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap(); assert_eq!(machine.save_state().unwrap(), boot);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    verify_live_owner(&mut machine, &rom);
    slots.undo(&mut machine).unwrap(); assert_eq!(machine.save_state().unwrap(), changed);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    verify_live_owner(&mut machine, &rom);
    let len = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; len]).unwrap();
    let backups = crate::save_backups::Backups::new(&save); backups.capture_startup().unwrap();
    backups.restore(backups.choices(len)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    verify_live_owner(&mut machine, &rom);
    machine.bus.load_save_data(&vec![9; len]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(fs::read(&save).unwrap(), vec![9; len]);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    verify_live_owner(&mut machine, &rom);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[202,201], "previous_tilemap_parsed":true,
        "missing_copy_failed_attachment_transactional":true, "load_undo_backup_restart_owner_retained":true,
        "first_pointer_and_rectangle_used_at_each_stage":true, "scratch_only_saves":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM, E201 predecessor and Wine; Linux/USA pending"]
fn european_additional_rectangle_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_EXTRA_RECTANGLES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_EXTRA_RECTANGLES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (201, 200, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (202, 201, 123));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 6);
    let rectangles = data.additional_rectangles.as_ref().unwrap();
    let pointers = data.additional_rectangle_pointers.as_ref().unwrap();
    for (i, &(offset, hash)) in model::ADDITIONAL_RECTANGLE_SPECS.iter().enumerate() {
        let bytes = rectangles[i].source_bytes();
        assert_eq!(bytes, rom[offset as usize..offset as usize + 26]);
        assert_eq!(gba_core::ffta_rom::digest(&bytes), hash);
    }
    for (pointer, &target) in pointers.iter().zip(&model::ADDITIONAL_RECTANGLE_TARGETS) {
        assert_eq!(*pointer, 0x08000000 + rectangles[target].source_offset);
    }
    assert_eq!(pointers[2], pointers[4]); assert_eq!(pointers[3], pointers[6]);
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut incomplete = data.clone(); incomplete.additional_rectangles = None; assert!(incomplete.validate().is_err());
    let mut invalid = data.clone(); invalid.additional_rectangle_pointers.as_mut().unwrap()[4] += 4; assert!(invalid.validate().is_err());
    let mut invalid = data.clone(); invalid.additional_rectangles.as_mut().unwrap()[0].cells.pop(); assert!(invalid.validate().is_err());
    let mut bad = data.clone(); bad.additional_rectangles.as_mut().unwrap()[0].cells[0] ^= 1;
    let bad_bytes = encode_observed_tilemap(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected"); drop(dataset);

    let mut visited: BTreeSet<u32> = (0x081094f2..0x081094fc).step_by(2).collect();
    visited.extend((0x080d8136..0x080d8142).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("additional-rectangles-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    poison(&mut managed, &rom, true);
    for (start, length) in spans() {
        let end = start + length;
        for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
            for at in start..end { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
        }
        for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        } }
    }
    // The new pointer span touches the prior root and the separately owned selector.
    for at in [0x3a4d3eusize, 0x3a4d3f, 0x3a4d5c, 0x3a4d5d] { managed.bus.rom[at] = !rom[at]; }
    let before = owner.observed_tilemap_stats().unwrap();
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x083a4d3e), u32::from_le_bytes(rom[0x3a4d3e..0x3a4d42].try_into().unwrap()));
    assert_eq!((RT_API.read32)(pointer, 0x083a4d5a), u32::from_le_bytes(rom[0x3a4d5a..0x3a4d5e].try_into().unwrap()));
    let after = owner.observed_tilemap_stats().unwrap();
    assert_eq!(after.root_bytes[1] - before.root_bytes[1], 2);
    assert_eq!(after.additional_rectangle_pointer_bytes - before.additional_rectangle_pointer_bytes, 4);
    assert_eq!(after.alternate_selector_bytes - before.alternate_selector_bytes, 2);
    for at in [0x3a4d3eusize, 0x3a4d3f, 0x3a4d5c, 0x3a4d5d] { managed.bus.rom[at] = rom[at]; }
    poison(&mut managed, &rom, false); let initial = raw.save_state().unwrap();

    for index in (0u8..=8).chain([255]) {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.bus.write8(0x02018008, index);
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[6] = 0x02018000; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[15] = 0x081094f2; machine.cpu.cpsr = 0xbf;
        }
        poison(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().additional_rectangle_pointer_bytes;
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x081094fc { assert!(steps < 8); raw.bus.frame_ready = false; raw.step(); steps += 1; }
        let mut steps = 0;
        while managed.cpu.regs[15] != 0x081094fc { assert!(steps < 8); managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1; }
        assert_eq!(owner.observed_tilemap_stats().unwrap().additional_rectangle_pointer_bytes - before, if (1..=7).contains(&index) { 4 } else { 0 });
        let at = model::RECTANGLE_TABLE_OFFSET as usize + usize::from(index) * 4;
        assert_eq!(raw.cpu.regs[0], u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        poison(&mut managed, &rom, false);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "pointer reader {index}");
    }
    for (index, rect) in rectangles.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[5] = 0x08000000 + rect.source_offset + 2;
            machine.cpu.regs[6] = 0x03003862; machine.cpu.regs[7] = 0x08000000 + rect.source_offset;
            machine.cpu.regs[9] = 0x08370eb4; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[15] = 0x080d8136; machine.cpu.cpsr = 0xbf;
        }
        poison(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().additional_rectangle_bytes[index];
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x080d8142 { assert!(steps < 16); raw.bus.frame_ready = false; raw.step(); steps += 1; }
        let mut steps = 0;
        while managed.cpu.regs[15] != 0x080d8142 { assert!(steps < 16); managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1; }
        assert_eq!(owner.observed_tilemap_stats().unwrap().additional_rectangle_bytes[index] - before, 1);
        assert_eq!(raw.cpu.regs[2], 24); assert_eq!(raw.cpu.regs[1], 0x08000000 + rect.source_offset + 2);
        poison(&mut managed, &rom, false);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "rectangle reader {index}");
    }
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap();
        machine.bus.write32(0x040000d4, 0x08000000 + model::ADDITIONAL_RECTANGLE_SPECS[0].0 + 2);
        machine.bus.write32(0x040000d8, 0x02010000); machine.bus.write16(0x040000dc, 12);
    }
    poison(&mut managed, &rom, true);
    let before = owner.observed_tilemap_stats().unwrap().additional_rectangle_bytes[0];
    raw.bus.write16(0x040000de, 0x8000); managed.bus.write16(0x040000de, 0x8000);
    assert_eq!(owner.observed_tilemap_stats().unwrap().additional_rectangle_bytes[0] - before, 24);
    poison(&mut managed, &rom, false); assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "DMA");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[202,201], "resources":123, "preserved_other_resources":122,
        "previous_tilemap_preserved":true, "pointer_words":7, "rectangles":5, "source_bytes":158,
        "pointer_aliases_and_gaps_preserved":true, "coherent_edit_rejected":true,
        "poisoned_mirrored_and_mixed_owner_boundaries":true, "original_interpreter_native_pointer_cases":10,
        "original_interpreter_native_rectangle_cases":5, "dma_halfwords":12, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
