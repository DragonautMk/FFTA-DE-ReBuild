//! EUR Soil Evidence sparse indexed words and original signed reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::indexed_words_3959 as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, path::Path, sync::Arc};

pub(super) fn assert_previous_indexed_words_preserved(
    previous: &Path,
    current_directory: &Path,
    prior: &gba_core::rom_data::Resource,
    current: &gba_core::rom_data::Resource,
) {
    assert_eq!((&current.id, &current.path, &current.format, &current.dependencies),
        (&prior.id, &prior.path, &prior.format, &prior.dependencies));
    assert_eq!(&current.sources[..prior.sources.len()], prior.sources.as_slice());
    let old: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let new: serde_json::Value = serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    assert_eq!(old["source_offset"], new["source_offset"]);
    assert_eq!(old["records"], new["records"]);
    if let Some(records) = old.get("soil_e3838_records") {
        assert_eq!(records, &new["soil_e3838_records"]);
    }
}

fn setup(machine: &mut Machine, index: i16) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[5] = 0x0300_7b00;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x080e_382e;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write16(0x0300_7b08, index as u16);
}

#[test]
#[ignore = "exact EUR ROM and E233 predecessor; Wine only, Linux/USA pending"]
fn european_soil_e3838_words_and_original_signed_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_E3838_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_E3838_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (233, 232, 130));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (248, 247, 136));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == INDEXED_WORDS_3959_ID {
            assert_previous_indexed_words_preserved(&previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 30);
        } else if prior.id == OBJ_STATIC_LISTS_ID {
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == COMPACT_OBJ_CATALOG_ID {
            super::soil_compact_metadata_tests::assert_previous_compact_catalog_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID || prior.id == OBSERVED_TILEMAP_ID {
            super::soil_transfer_records_tests::assert_previous_soil_transfer_resource_preserved(
                &previous, &prepared.directory, prior, current);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let data = &source.indexed_words_3959.as_ref().unwrap().decoded;
    assert_eq!(data.soil_e3838_records.len(), 30);
    for (record, (id, _)) in data.soil_e3838_records.iter().zip(model::SOIL_E3838_WORDS) {
        assert_eq!(record.id, id);
        assert_eq!(record.source_offset, model::SOIL_E3838_BASE + u32::from(id) * 4);
        let at = record.source_offset as usize;
        assert_eq!(record.value.to_le_bytes(), rom[at..at + 4]);
    }
    assert_eq!(parse_indexed_words_3959(&fs::read(prepared.directory.join(INDEXED_WORDS_3959_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.soil_e3838_records.swap(0, 1); assert!(bad.validate().is_err());
    let mut bad = serde_json::to_value(data).unwrap(); bad["soil_e3838_records"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::IndexedWords3959>(bad).is_err());
    let mut forged = data.clone(); forged.soil_e3838_records[0].value ^= 1; forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_indexed_words_3959(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == INDEXED_WORDS_3959_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.indexed_words_3959 = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let old_start = model::SOURCE_OFFSET as usize;
    let old_end = model::END_OFFSET as usize;
    for at in old_start..old_end { managed.bus.rom[at] = !rom[at]; }
    let old_before = owner.indexed_words_3959_stats().unwrap();
    for at in old_start..old_end { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    let old_after = owner.indexed_words_3959_stats().unwrap();
    assert_eq!(old_after.source_bytes - old_before.source_bytes, (old_end - old_start) as u64);
    assert_eq!((old_after.fields_mask, old_after.record_mask), (0xf, [(1u64 << 30) - 1, 0]));
    assert_eq!(old_after.soil_e3838_source_bytes, old_before.soil_e3838_source_bytes);
    managed.bus.rom[old_start..old_end].copy_from_slice(&rom[old_start..old_end]);
    for record in &data.soil_e3838_records { for at in record.source_offset as usize..record.source_offset as usize + 4 { managed.bus.rom[at] = !rom[at]; } }
    let before = owner.indexed_words_3959_stats().unwrap();
    for record in &data.soil_e3838_records { for at in record.source_offset..record.source_offset + 4 {
        assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
    } }
    let after = owner.indexed_words_3959_stats().unwrap();
    assert_eq!(after.soil_e3838_source_bytes - before.soil_e3838_source_bytes, 120);
    assert_eq!(after.soil_e3838_record_mask, (1u64 << 30) - 1);
    assert_eq!((after.fields_mask, after.record_mask), (before.fields_mask, before.record_mask));
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for record in &data.soil_e3838_records {
        let at = record.source_offset as usize;
        for neighbor in (at - 3..at).chain(at + 4..at + 8) {
            if !owned(neighbor) { changed.insert(neighbor); managed.bus.rom[neighbor] = rom[neighbor] ^ 0x5a; }
        }
    }
    for record in &data.soil_e3838_records { for edge in [record.source_offset as usize, record.source_offset as usize + 4] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| if changed.contains(&(at + n)) { rom[at + n] ^ 0x5a } else { rom[at + n] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
            }
        }
    } }
    for record in &data.soil_e3838_records { let at = record.source_offset as usize; managed.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]); }
    for at in changed { managed.bus.rom[at] = rom[at]; }

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x080e_382e..0x080e_383a).step_by(2).map(|addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("soil-e3838-signed-reader-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for index in model::SOIL_E3838_WORDS.iter().map(|(id, _)| i16::from(*id)).chain([-32768i16, -1, 0, 8, 24, 33, 50]) {
            let at = (i64::from(model::SOIL_E3838_BASE) + i64::from(index) * 4) as usize;
            let selected = data.soil_e3838_byte_at(at as u32).is_some();
            if !selected { assert!((at..at + 4).all(|x| !owned(x))); }
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, index); }
            for x in at..at + 4 { managed.bus.rom[x] = rom[x] ^ 0x5a; if !selected { raw.bus.rom[x] = rom[x] ^ 0x5a; } }
            let before = owner.indexed_words_3959_stats().unwrap().soil_e3838_source_bytes;
            for _ in 0..12 { if raw.cpu.regs[15] == 0x080e_383a { break; } raw.bus.frame_ready = false; raw.step(); }
            for _ in 0..12 {
                if managed.cpu.regs[15] == 0x080e_383a { break; }
                managed.bus.frame_ready = false;
                if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
                else { managed.step(); }
            }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (0x080e_383a, 0x080e_383a));
            assert_eq!(owner.indexed_words_3959_stats().unwrap().soil_e3838_source_bytes - before, if selected { 4 } else { 0 });
            let expected = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) ^ if selected { 0 } else { 0x5a5a_5a5a };
            assert_eq!((raw.cpu.regs[0], managed.cpu.regs[0]), (expected, expected));
            raw.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
            managed.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "index={index}, native={native}");
            cases += 1;
        }
    }
    assert_eq!(cases, 74);

    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    let offset = data.soil_e3838_records[0].source_offset;
    for kind in 0..3 {
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + offset, 0x0203_2400, if kind == 2 { 8 } else { 2 }]);
                machine.cpu.regs[15] = 0x0200_0000; machine.cpu.cpsr = 0xbf;
                machine.bus.write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        for at in offset as usize..offset as usize + 4 { transfer_managed.bus.rom[at] = !rom[at]; }
        let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.indexed_words_3959_stats().unwrap().soil_e3838_source_bytes;
        for machine in [&mut transfer_raw, &mut transfer_managed] { if kind == 0 { machine.bus.write16(0x0400_00de, 0x8000); } else { machine.step(); } }
        assert!(transfer_owner.indexed_words_3959_stats().unwrap().soil_e3838_source_bytes > before);
        transfer_managed.bus.rom[offset as usize..offset as usize + 4].copy_from_slice(&rom[offset as usize..offset as usize + 4]);
        assert_eq!(transfer_raw.save_state().unwrap(), transfer_managed.save_state().unwrap(), "transfer {kind}");
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
        "status":"PASS", "revision":[248,247], "resources":136, "previous_indexed_words_preserved":30,
        "previous_other_resources_unchanged":121, "previous_copied_halfwords_preserved":true, "previous_literals_preserved":true, "previous_switch_tables_preserved":true, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "soil_e3838_words":30, "soil_e3838_source_bytes":120,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":74,
        "negative_and_unobserved_raw_fallback":true, "dma_cpuset_cpufastset_state_and_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
