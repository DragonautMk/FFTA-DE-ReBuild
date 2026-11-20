//! EUR Red Spring compact-bank records and original guest readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::compact_obj_catalog as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E185 predecessor and Wine; Linux/USA pending"]
fn european_red_spring_compact_records_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_RECORDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_COMPACT_RECORDS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (185, 184, 120));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (186, 185, 120));
    for prior in &old.resources {
        if prior.id == COMPACT_OBJ_CATALOG_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.compact_obj_catalog.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let previous_sources: Vec<_> = source.manifest.resources[99].sources.iter().filter(|range| range.offset != model::RED_SPRING_RECORD_OFFSET).cloned().collect();
    assert_eq!(previous_sources, old.resources[99].sources);
    assert_eq!(source.manifest.resources[99].sources.len(), old.resources[99].sources.len() + 1);
    let records = data.red_spring_records.as_ref().unwrap();
    assert_eq!(records.iter().map(|record| record.halfwords[0]).collect::<Vec<_>>(), [2, 1, 0]);
    for record in records { let at = record.source_offset as usize; assert_eq!(record.source_bytes(), &rom[at..at + model::RED_SPRING_RECORD_LENGTH]); }
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_compact_obj_catalog(&fs::read(prepared.directory.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.red_spring_records.as_mut().unwrap()[0].bank_pointer += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.red_spring_records.as_mut().unwrap()[0].halfwords[0] = u16::MAX; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.red_spring_records.as_mut().unwrap()[0].halfwords[2] ^= 1;
    let bad_bytes = encode_compact_obj_catalog(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone(); altered_manifest.resources[99].sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.compact_obj_catalog = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent data/file-hash edits must be rejected");
    drop(dataset);

    let visited: BTreeSet<u32> = [0x080d8974, 0x080d8b60, 0x080d8baa, 0x080d8bac].into_iter().collect();
    assert_eq!(visited.len(), 4);
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("compact-redspring-records-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::RED_SPRING_RECORD_OFFSET as usize; let end = start + model::RED_SPRING_RECORD_LENGTH * 3;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, start + model::RED_SPRING_RECORD_LENGTH, start + model::RED_SPRING_RECORD_LENGTH * 2, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        }
    }
    let before = owner.compact_obj_catalog_stats().unwrap().red_spring_record_bytes;
    let at = start + model::RED_SPRING_RECORD_LENGTH - 2; let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    let after = owner.compact_obj_catalog_stats().unwrap().red_spring_record_bytes;
    assert_eq!((after[0] - before[0], after[1] - before[1]), (2, 2));
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for (index, record) in records.iter().enumerate() {
        for mode in 0..3 {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                machine.cpu.regs[6] = 0x08000000 + record.source_offset; machine.cpu.regs[13] = 0x03007c00;
                machine.cpu.regs[15] = [0x080d8b60, 0x080d8baa, 0x080d8974][mode]; machine.cpu.cpsr = 0xbf;
            }
            for at in start..end { managed.bus.rom[at] = !rom[at]; }
            let before = owner.compact_obj_catalog_stats().unwrap().red_spring_record_bytes;
            let stop = [0x080d8b62, 0x080d8bae, 0x080d8976][mode];
            let mut steps = 0;
            while raw.cpu.regs[15] != stop { assert!(steps < 8); raw.bus.frame_ready = false; raw.step(); steps += 1; }
            let mut steps = 0;
            while managed.cpu.regs[15] != stop {
                assert!(steps < 8); managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1;
            }
            let after = owner.compact_obj_catalog_stats().unwrap().red_spring_record_bytes;
            assert_eq!(after[index] - before[index], [4, 4, 2][mode]);
            if mode == 0 { assert_eq!(raw.cpu.regs[1], record.bank_pointer); }
            if mode == 1 { assert_eq!(raw.cpu.regs[2], u32::from(record.halfwords[0])); assert_eq!(raw.cpu.regs[0], u32::from(record.halfwords[2])); }
            if mode == 2 { assert_eq!(raw.cpu.regs[0], u32::from(record.halfwords[9])); }
            managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[186,185], "resources":120, "preserved_other_resources":119,
        "previous_compact_catalog_preserved":true, "records":3, "new_source_bytes":72,
        "poisoned_mirrored_and_record_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":9, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
