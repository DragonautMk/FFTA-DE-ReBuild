//! EUR direct tilemap descriptors and their original field reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap_descriptor as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E198 predecessor and Wine; Linux/USA pending"]
fn european_direct_tilemap_descriptors_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_DIRECT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_DIRECT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (198, 197, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_DESCRIPTOR_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (199, 198, 123));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_DESCRIPTOR_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_DESCRIPTOR_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 7);
    let entries = data.additional_tilemap_descriptors.as_ref().unwrap();
    let start = model::SOURCE_OFFSET as usize + model::SOURCE_LENGTH; let end = start + 7 * model::SOURCE_LENGTH;
    for entry in entries { let at = entry.source_offset as usize; assert_eq!(entry.source_bytes(), rom[at..at + model::SOURCE_LENGTH]); }
    assert_eq!(parse_observed_tilemap_descriptor(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_DESCRIPTOR_FILE)).unwrap()).unwrap(), *data);
    let tilemap = &source.observed_tilemap.as_ref().unwrap().decoded;
    data.validate_dependencies(tilemap).unwrap();
    let mut missing = tilemap.clone(); missing.additional_streams = None;
    assert!(data.validate_dependencies(&missing).is_err());
    let mut bad = data.clone(); bad.additional_tilemap_descriptors.as_mut().unwrap()[0].parameter16 ^= 1;
    let bad_bytes = encode_observed_tilemap_descriptor(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_DESCRIPTOR_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap_descriptor = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let visited: BTreeSet<u32> = (0x080db318..0x080db320).step_by(2).collect();
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("direct-tilemap-descriptors-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]); let initial = raw.save_state().unwrap();
    for (index, entry) in entries.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.bus.write32(0x02018024, 0x020242bc);
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[3] = 0x08000000 + entry.source_offset; machine.cpu.regs[4] = 0x02018000;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[15] = 0x080db318; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.observed_tilemap_descriptor_stats().unwrap().additional_tilemap_descriptor_bytes;
        for _ in 0..4 { raw.bus.frame_ready = false; raw.step(); }
        let mut steps = 0;
        while managed.cpu.regs[15] != 0x080db320 {
            assert!(steps < 8); managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        assert_eq!(owner.observed_tilemap_descriptor_stats().unwrap().additional_tilemap_descriptor_bytes - before, 7);
        assert_eq!((raw.cpu.regs[0],raw.cpu.regs[1],raw.cpu.regs[2],raw.cpu.regs[3]),
            (0x020242bc,entry.tilemap_pointer,u32::from(entry.parameter16),u32::from(entry.parameter8)));
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "descriptor {index}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[199,198], "resources":123, "preserved_other_resources":122,
        "previous_descriptors_preserved":true, "descriptors":7, "source_bytes":56, "tilemap_dependencies_verified":true,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":7, "full_state_clock_equal":true,
        "ordinary_scene":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
