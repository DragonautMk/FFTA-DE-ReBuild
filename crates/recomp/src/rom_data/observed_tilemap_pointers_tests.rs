//! EUR indexed tilemap pointers and their original guest reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, E196 predecessor and Wine; Linux/USA pending"]
fn european_indexed_tilemap_pointers_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_POINTER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_POINTER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (196, 195, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (197, 196, 123));
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
    assert_eq!(current.sources.len(), prior.sources.len() + 1);
    let start = model::INDEXED_POINTER_OFFSET as usize;
    let end = start + model::INDEXED_POINTER_COUNT * 4;
    let pointers = data.indexed_pointers.unwrap();
    assert_eq!(pointers.iter().flat_map(|pointer| pointer.to_le_bytes()).collect::<Vec<_>>(), rom[start..end]);
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut malformed = serde_json::to_value(data).unwrap();
    malformed["indexed_pointers"].as_array_mut().unwrap().pop();
    assert!(serde_json::from_value::<model::ObservedTilemap>(malformed).is_err());
    let mut bad = data.clone(); bad.indexed_pointers.as_mut().unwrap()[2] ^= 4;
    let bad_bytes = encode_observed_tilemap(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let entry = 0x081091e0;
    let instrs = (0..5).map(|i| { let at = entry + i * 2; armv4t::decode_thumb(view.read16(at), at) }).collect();
    let object = root.join("tilemap-pointer-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks: vec![Block { addr: entry, thumb: true, instrs }] }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
        for at in start..end { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    }
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for index in (0u8..=8).chain([255]) {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.bus.write8(0x02018000, index);
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[4] = 0x02018000; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[14] = 0x08000301; machine.cpu.regs[15] = entry; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.observed_tilemap_stats().unwrap().indexed_pointer_bytes;
        for _ in 0..5 { raw.bus.frame_ready = false; raw.step(); }
        managed.bus.frame_ready = false;
        table.get(entry | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.observed_tilemap_stats().unwrap().indexed_pointer_bytes - before, if index < 8 { 4 } else { 0 });
        let at = start + usize::from(index) * 4;
        assert_eq!(raw.cpu.regs[0], u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "original reader index {index}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[197,196], "resources":123, "preserved_other_resources":122,
        "previous_tilemap_preserved":true, "pointer_words":8, "source_bytes":32,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_index_cases":10, "outside_projection_guest_reads_preserved":true,
        "full_state_clock_equal":true, "ordinary_scene":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
