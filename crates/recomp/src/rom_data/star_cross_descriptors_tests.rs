//! EUR Star Cross compact descriptors and their original Thumb readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::compact_obj_catalog as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    let start = model::STAR_CROSS_RECORD_OFFSET as usize;
    for at in start..start + 2 * model::STAR_CROSS_RECORD_LENGTH {
        machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
    }
}

#[test]
#[ignore = "exact EUR ROM and E296 predecessor; Wine only, Linux/USA pending"]
fn european_star_cross_descriptors_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STAR_CROSS_DESCRIPTORS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_STAR_CROSS_DESCRIPTORS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (296, 295, 148));
    let old_data: model::CompactObjCatalog = serde_json::from_slice(&fs::read(previous.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert!(old_data.star_cross_records.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (297, 296, 148));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id != COMPACT_OBJ_CATALOG_ID {
            assert_eq!(current, prior);
            assert_eq!(fs::read(previous.join(&prior.path)).unwrap(), fs::read(prepared.directory.join(&current.path)).unwrap());
        } else {
            let mut metadata = current.clone();
            metadata.sources.retain(|span| span.offset != model::STAR_CROSS_RECORD_OFFSET);
            metadata.format = prior.format.clone();
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
            assert_eq!(current.sources.len(), prior.sources.len() + 1);
        }
    }
    let data = &source.compact_obj_catalog.as_ref().unwrap().decoded;
    let mut without_new = data.clone();
    let records = without_new.star_cross_records.take().unwrap();
    assert_eq!(without_new, old_data);
    assert_eq!(records.iter().map(|record| record.halfwords[0]).collect::<Vec<_>>(), [0, 1]);
    for record in &records {
        let at = record.source_offset as usize;
        assert_eq!(record.source_bytes(), &rom[at..at + model::STAR_CROSS_RECORD_LENGTH]);
    }
    let root_owner = source.manifest.resources.iter().find(|r| r.id == THUMB_PC_LITERALS_ID).unwrap();
    assert!(root_owner.sources.iter().any(|span| span.offset <= 0xeee24 && span.offset + span.length >= 0xeee28));
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_compact_obj_catalog(&fs::read(prepared.directory.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone(); invalid.star_cross_records.as_mut().unwrap()[0].bank_pointer += 4; assert!(invalid.validate().is_err());
    let mut invalid = data.clone(); invalid.star_cross_records.as_mut().unwrap()[0].halfwords[0] = 1; assert!(invalid.validate().is_err());
    let mut forged = data.clone(); forged.star_cross_records.as_mut().unwrap()[0].halfwords[2] ^= 1;
    forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_compact_obj_catalog(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone();
    altered.resources.iter_mut().find(|e| e.id == COMPACT_OBJ_CATALOG_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.compact_obj_catalog = Some((&forged, &forged_bytes));
    assert!(source.validate_files(files).is_err(), "coherent modified file and hash must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let start = model::STAR_CROSS_RECORD_OFFSET as usize;
    let end = start + 2 * model::STAR_CROSS_RECORD_LENGTH;
    set_backing(&mut managed, &rom, true);
    for at in start..end {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().star_cross_record_bytes, [72, 72]);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources).any(|span|
        (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for edge in [start, start + model::STAR_CROSS_RECORD_LENGTH, end] {
        for at in edge - 3..edge + 3 {
            if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
        }
    }
    for edge in [start, start + model::STAR_CROSS_RECORD_LENGTH, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |i| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
            assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
        }
    }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    set_backing(&mut managed, &rom, false);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let ranges = [(0x080d_8b60, 0x080d_8b62), (0x080d_8baa, 0x080d_8bae), (0x080d_8974, 0x080d_8976)];
    let caller = (0x080e_edfa, 0x080e_ee12);
    let mut addresses = BTreeSet::new();
    for (a, z) in ranges.into_iter().chain([caller]) { addresses.extend((a..z).step_by(2)); }
    let blocks = addresses.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("star-cross-descriptors-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut completed = 0;
    for native in [false, true] {
        for (mode, (pc, stop)) in ranges.into_iter().enumerate() {
            for record in &records {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[6] = 0x0800_0000 + record.source_offset;
                    machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[15] = pc; machine.cpu.cpsr = 0xbf;
                }
                set_backing(&mut managed, &rom, true);
                for (machine, generated) in [(&mut raw, false), (&mut managed, native)] {
                    let mut steps = 0;
                    while machine.cpu.regs[15] != stop {
                        assert!(steps < 8); machine.bus.frame_ready = false;
                        if generated { table.get(machine.cpu.regs[15] | 1).unwrap()(&RT_API, machine as *mut Machine as *mut std::ffi::c_void); }
                        else { machine.step(); }
                        steps += 1;
                    }
                }
                if mode == 0 { assert_eq!(raw.cpu.regs[1], record.bank_pointer); }
                if mode == 1 { assert_eq!(raw.cpu.regs[2], u32::from(record.halfwords[0])); assert_eq!(raw.cpu.regs[0], u32::from(record.halfwords[2])); }
                if mode == 2 { assert_eq!(raw.cpu.regs[0], u32::from(record.halfwords[9])); }
                set_backing(&mut managed, &rom, false);
                assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "reader mode={mode} native={native}");
                completed += 1;
            }
        }
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            machine.cpu.regs[4] = 0x0201_8000; machine.cpu.regs[6] = 7;
            machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[15] = caller.0; machine.cpu.cpsr = 0xbf;
        }
        for at in 0xeee24..0xeee28 { managed.bus.rom[at] = !rom[at]; }
        for (machine, generated) in [(&mut raw, false), (&mut managed, native)] {
            let mut steps = 0;
            while machine.cpu.regs[15] != caller.1 {
                assert!(steps < 20); machine.bus.frame_ready = false;
                if generated { table.get(machine.cpu.regs[15] | 1).unwrap()(&RT_API, machine as *mut Machine as *mut std::ffi::c_void); }
                else { machine.step(); }
                steps += 1;
            }
            assert_eq!(machine.bus.read32(0x0300_7c00), 2);
            assert_eq!(machine.bus.read32(0x0300_7c08), 0x0800_0000 + model::STAR_CROSS_RECORD_OFFSET);
        }
        managed.bus.rom[0xeee24..0xeee28].copy_from_slice(&rom[0xeee24..0xeee28]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "caller native={native}");
        completed += 1;
    }
    assert_eq!(completed, 14);
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[297,296], "resources":148, "new_source_bytes":48,
        "prior_other_resources_unchanged":147, "prior_representation_preserved":true,
        "root_remains_in_existing_owner":true, "poisoned_mirrors_and_mixed_boundaries":true,
        "original_interpreter_generated_cases":completed, "full_state_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
