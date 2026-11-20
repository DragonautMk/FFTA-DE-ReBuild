//! EUR OBJ byte groups and original Thumb readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::obj_lists as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, E195 predecessor and Wine; Linux/USA pending"]
fn european_obj_layout_offsets_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_OBJ_LAYOUT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_OBJ_LAYOUT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (195, 194, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (196, 195, 123));
    for prior in &old.resources {
        if prior.id == OBJ_STATIC_LISTS_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.obj_lists.as_ref().unwrap().decoded; let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap();
    let without_new: Vec<_> = current.sources.iter().filter(|entry| entry.offset != model::LAYOUT_OFFSETS_ROOT).cloned().collect();
    assert_eq!(without_new, prior.sources); assert_eq!(current.sources.len(), prior.sources.len() + 1);
    let offsets = data.layout_offsets.as_ref().unwrap(); let start = model::LAYOUT_OFFSETS_ROOT as usize; let end = start + model::LAYOUT_OFFSETS_LENGTH;
    assert_eq!(offsets.source_bytes(), rom[start..end]); data.verify_source(&rom).unwrap();
    assert_eq!(parse_obj_lists(&fs::read(prepared.directory.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap(), *data);
    let mut malformed = serde_json::to_value(data).unwrap(); malformed["layout_offsets"]["first_pair"].as_array_mut().unwrap().pop();
    assert!(serde_json::from_value::<model::Lists>(malformed).is_err());
    let mut bad = data.clone(); bad.layout_offsets.as_mut().unwrap().first_pair[0] ^= 1;
    let bad_bytes = encode_obj_lists(&bad).unwrap(); let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone(); altered_manifest.resources.iter_mut().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.obj_lists = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected"); drop(dataset);

    let cases = [
        (0x0809c06c, 2usize, 3usize, 0usize, None, false, 0usize),
        (0x0809ae64, 3, 3, 2, None, false, 1),
        (0x080a334c, 2, 1, 4, Some(1usize), true, 2),
        (0x080a33ac, 1, 1, 4, None, false, 2),
    ];
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = cases.iter().map(|&(addr, _, _, _, _, _, _)| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("obj-layout-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap(); let owner = managed.bus.rom_resources().unwrap().clone();
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, start + 2, start + 4, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]); let initial = raw.save_state().unwrap();
    for (case, &(reader, base, target, relative, index_reg, signed, group)) in cases.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            if let Some(index_reg) = index_reg { machine.cpu.regs[index_reg] = 0; }
            machine.cpu.regs[base] = 0x08000000 + model::LAYOUT_OFFSETS_ROOT + relative as u32;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x08000301;
            machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.obj_list_stats().unwrap().layout_offset_bytes[group];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.obj_list_stats().unwrap().layout_offset_bytes[group] - before, 1);
        let byte = rom[start + relative + usize::from(reader == 0x080a33ac)];
        let expected = if signed { (byte as i8 as i32) as u32 } else { u32::from(byte) };
        assert_eq!(raw.cpu.regs[target], expected);
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "reader case {case}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[196,195], "resources":123, "preserved_other_resources":122,
        "previous_obj_lists_preserved":true, "groups":3, "new_source_bytes":8,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":4, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
