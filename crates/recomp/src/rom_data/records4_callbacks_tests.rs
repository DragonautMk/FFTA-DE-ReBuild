//! EUR linked-record callback partitions and the original Thumb branch ABI.
use super::*;
use gba_core::{capi::RT_API, rom_data::records4 as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E191 predecessor and Wine; Linux/USA pending"]
fn european_records4_callbacks_publication_and_native_branches() {
    let root = PathBuf::from(std::env::var_os("FFTA_RECORDS4_CALLBACK_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_RECORDS4_CALLBACK_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (191, 190, 122));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(RECORDS4_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (192, 191, 122));
    for prior in &old.resources {
        if prior.id == RECORDS4_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.records4.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == RECORDS4_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == RECORDS4_ID).unwrap();
    let without_new: Vec<_> = current.sources.iter().filter(|entry| !model::CALLBACK_ROOTS.contains(&entry.offset)).cloned().collect();
    assert_eq!(without_new, prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 3);
    let tables = data.callback_tables.as_ref().unwrap();
    for i in 0..3 {
        assert_eq!(tables[i].len(), model::CALLBACK_COUNTS[i]);
        assert_eq!(tables[i].iter().flat_map(|word| word.to_le_bytes()).collect::<Vec<_>>(),
            rom[model::CALLBACK_ROOTS[i] as usize..model::CALLBACK_ENDS[i] as usize]);
    }
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_records4(&fs::read(prepared.directory.join(RECORDS4_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.callback_tables.as_mut().unwrap()[0][0] &= !1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.callback_tables.as_mut().unwrap()[1].pop(); assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.callback_tables.as_mut().unwrap()[0][0] += 2;
    let bad_bytes = encode_records4(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == RECORDS4_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.records4 = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    // The original LDRs feed the existing BX r1 bridge; the guest owns both index and LR.
    let cases = [(0x08134ca2, 0usize, 0usize, 3usize), (0x081350da, 1, 1, 23), (0x081357ba, 0, 2, 0)];
    let bridge = 0x08145c0c;
    let visited: BTreeSet<u32> = cases.iter().map(|&(reader, _, _, _)| reader).chain([bridge]).collect();
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("records4-callbacks-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::CALLBACK_ROOTS[0] as usize; let end = model::CALLBACK_ENDS[2] as usize;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, model::CALLBACK_ROOTS[1] as usize, model::CALLBACK_ROOTS[2] as usize, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for &(reader, base, table_index, element) in &cases {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[base] = 0x08000000 + model::CALLBACK_ROOTS[table_index] + element as u32 * 4;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x08000301;
            machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.records4_stats().unwrap().callback_bytes[table_index];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.records4_stats().unwrap().callback_bytes[table_index] - before, 4);
        assert_eq!(raw.cpu.regs[1], tables[table_index][element]);
        for machine in [&mut raw, &mut managed] { machine.cpu.regs[15] = bridge; }
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(bridge | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(raw.cpu.regs[15], tables[table_index][element] & !1);
        assert_eq!(raw.cpu.regs[14], 0x08000301);
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[192,191], "resources":122, "preserved_other_resources":121,
        "previous_records4_preserved":true, "tables":3, "callback_words":107, "new_source_bytes":428,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_branch_cases":3, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
