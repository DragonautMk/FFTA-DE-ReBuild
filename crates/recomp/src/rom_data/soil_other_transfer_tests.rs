//! EUR Soil Evidence list and mode-7 sources with their original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::{observed_tilemap as tilemap, observed_tilemap_descriptor as descriptor, observed_tilemap_row as row}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn selected_ranges() -> Vec<(usize, usize)> {
    let mut spans = vec![
        (tilemap::SOIL_RECTANGLE_ENTRIES_OFFSET as usize, 16), (tilemap::SOIL_RECTANGLE_LIST_OFFSET as usize, 8),
        (tilemap::SOIL_MODE7_OFFSET as usize, 6), (tilemap::SOIL_MODE7_OFFSET as usize + 6, 4),
        (descriptor::SOIL_MODE7_DESCRIPTOR_OFFSET as usize, 8),
    ];
    for &(start, height, width, _) in &tilemap::SOIL_RECTANGLE_SPECS {
        spans.push((start as usize, 2 + usize::from(height) * usize::from(width) * 2));
    }
    for &(start, _, _, _) in &row::SOIL_OTHER_ROW_SPECS { spans.push((start as usize, 12)); }
    spans
}

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, len) in selected_ranges() {
        for at in start..start + len { machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] }; }
    }
}

#[test]
#[ignore = "exact EUR ROM and E241 predecessor; Wine only, Linux/USA pending"]
fn european_soil_other_transfer_sources_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_OTHER_TRANSFER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_OTHER_TRANSFER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (241, 240, 130));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (257, 256, 142));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == OBJ_STATIC_LISTS_ID {
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(
                &previous, &prepared.directory, prior, current);
            continue;
        }
        if prior.id == OBSERVED_LZ_ID {
            super::soil_evidence_lz_tests::assert_previous_observed_lz_preserved(
                &previous, &prepared.directory, prior, current);
            continue;
        }
        if [OBSERVED_TILEMAP_ID, OBSERVED_TILEMAP_ROW_ID, OBSERVED_TILEMAP_DESCRIPTOR_ID].contains(&prior.id.as_str()) {
            super::soil_transfer_records_tests::assert_previous_soil_transfer_resource_preserved(&previous, &prepared.directory, prior, current);
            let added = &current.sources[prior.sources.len()..];
            let expected = match prior.id.as_str() {
                OBSERVED_TILEMAP_ID => (6, 72), OBSERVED_TILEMAP_ROW_ID => (2, 24), _ => (1, 8),
            };
            assert_eq!((added.len(), added.iter().map(|s| s.length).sum::<u32>()), expected);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let map = &source.observed_tilemap.as_ref().unwrap().decoded;
    let desc = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    let rows = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    desc.validate_dependencies(map).unwrap(); rows.validate_dependencies(desc).unwrap(); rows.validate_tilemap_dependency(map).unwrap();
    let soil = map.soil_transfer_sources.as_ref().unwrap();
    assert_eq!(soil.entry_count, 2); assert_eq!(soil.rectangles.iter().map(|r| (r.height, r.width)).collect::<Vec<_>>(), vec![(2, 7), (1, 3)]);
    assert_eq!(soil.mode7.first_run.control, 0x8004);
    let mut invalid = map.clone(); invalid.soil_transfer_sources.as_mut().unwrap().entries[0].rectangle_pointer += 2; assert!(invalid.validate().is_err());
    let mut invalid = map.clone(); invalid.soil_transfer_sources.as_mut().unwrap().mode7.first_run.control = 0; assert!(invalid.validate().is_err());
    let mut invalid = rows.clone(); invalid.soil_other_rows.as_mut().unwrap()[1].descriptor_pointer += 4; assert!(invalid.validate_dependencies(desc).is_err());
    let mut forged = map.clone(); forged.soil_transfer_sources.as_mut().unwrap().rectangles[0].cells[0] ^= 1;
    let forged_bytes = encode_observed_tilemap(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap(); let mut altered = dataset.manifest.clone();
    altered.resources.iter_mut().find(|e| e.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.observed_tilemap = Some((&forged, &forged_bytes));
    assert!(source.validate_files(files).is_err(), "coherent modified file and hash must be rejected"); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap(); let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap(); set_backing(&mut managed, &rom, true);
    let spans = selected_ranges(); assert_eq!(spans.iter().map(|(_, n)| n).sum::<usize>(), 104);
    for &(start, len) in &spans { for at in start..start + len {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    } }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    let edges: Vec<_> = spans.iter().flat_map(|&(start, len)| [start, start + len]).collect();
    for &edge in &edges { for at in edge.saturating_sub(3)..edge + 3 {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } }
    assert!(changed.contains(&0xaf2716) && changed.contains(&0xaf2717), "the unclassified gap must remain ROM-backed");
    for edge in edges { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
    } }
    for at in changed { managed.bus.rom[at] = rom[at]; } set_backing(&mut managed, &rom, false);

    // Exercise actual load instructions at the observed list, descriptor and nested reader sites.
    let cases: &[(u32, usize, u32)] = &[
        (0x080d_b388, 1, 0x0839_e0a0), (0x080d_b38e, 1, 0x0839_e0a0),
        (0x080d_b392, 6, 0x0839_e088), (0x080d_b39a, 6, 0x0839_e088),
        (0x080d_b39e, 4, 0x0839_e078), (0x080d_b3a0, 4, 0x0839_e078),
        (0x080d_b39e, 4, 0x0839_e080), (0x080d_b3a0, 4, 0x0839_e080),
        (0x080d_8136, 7, 0x08ae_71bc), (0x080d_8136, 7, 0x08ae_777c),
        (0x080d_b2ba, 1, 0x0839_a704), (0x080d_b316, 0, 0x0839_a704),
        (0x080d_b318, 3, 0x0839_a6e8), (0x080d_b31c, 3, 0x0839_a6e8),
        (0x080d_a560, 7, 0x08af_270c), (0x080d_a75c, 7, 0x08af_270c),
        (0x080d_a15a, 2, 0x08af_2712), (0x080d_a182, 4, 0x08af_2712),
    ];
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses: BTreeSet<_> = cases.iter().map(|case| case.0).collect();
    let blocks = addresses.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("soil-other-transfer-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut completed = 0;
    for native in [false, true] { for &(pc, register, address) in cases {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
            machine.cpu.regs[register] = address; machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[15] = pc; machine.cpu.cpsr = 0xbf;
        }
        set_backing(&mut managed, &rom, true); managed.bus.frame_ready = false;
        if native { table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
        else { managed.step(); }
        // TinyCC can join the two adjacent selected loads into one block.
        let stop = if native && matches!(pc, 0x080d_b39e | 0x080d_b316) { pc + 4 } else { pc + 2 };
        assert_eq!(managed.cpu.regs[15], stop, "unexpected generated reader extent at {pc:08x}");
        for _ in 0..2 {
            if raw.cpu.regs[15] == stop { break; }
            raw.bus.frame_ready = false; raw.step();
        }
        assert_eq!(raw.cpu.regs[15], stop, "interpreter reader extent at {pc:08x}");
        set_backing(&mut managed, &rom, false);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "state differs at pc={pc:08x} native={native}"); completed += 1;
    } }
    assert_eq!(completed, 36);
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[257,256], "new_source_bytes":104, "type2_rows":1, "type4_rows":1,
        "previous_fields_and_other_resources_preserved":true, "gap_remains_rom_backed":true,
        "poisoned_mirrors_and_mixed_boundaries":true, "original_interpreter_generated_cases":completed,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
