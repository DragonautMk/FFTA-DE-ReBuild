//! EUR rectangle roots, original readers and adjacent resource ownership.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn set_roots(machine: &mut Machine, rom: &[u8], invert: bool) {
    for (offset, length) in [(model::RECTANGLE_CHAIN_OFFSET, model::RECTANGLE_CHAIN_LENGTH),
                             (model::RECTANGLE_TABLE_OFFSET, model::RECTANGLE_TABLE_LENGTH)] {
        for at in offset as usize..offset as usize + length { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
}

#[test]
#[ignore = "exact EUR ROM, E183 predecessor and Wine; Linux/USA pending"]
fn european_tilemap_rectangle_roots_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ROOTS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_ROOTS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (183, 182, 120));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (184, 183, 120));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    assert_eq!(&source.manifest.resources[117].sources[..3], &old.resources[117].sources);
    assert_eq!(source.manifest.resources[117].sources.len(), 5);
    for (offset, bytes, _) in data.source_spans() { let at = offset as usize; assert_eq!(bytes, &rom[at..at + bytes.len()]); }
    assert_eq!(parse_observed_tilemap(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.rectangle_roots.descriptor_pointer += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.rectangle_roots.entry_count = 2; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.rectangle_roots.second_rectangle_pointer += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.rectangle_roots.descriptor_parameters[0] ^= 1;
    let bad_bytes = encode_observed_tilemap(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone(); altered_manifest.resources[117].sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent data/file-hash edits must be rejected");
    assert!(model::Catalog::from_rom(&rom, &source.manifest.resources).is_err());
    drop(dataset);

    let mut visited: BTreeSet<u32> = (0x080db392..0x080db3aa).step_by(2).collect();
    visited.insert(0x081094fa);
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("observed-tilemap-roots-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    set_roots(&mut managed, &rom, true);
    // Alter the beginning of the adjacent E181 descriptor as well, proving
    // that a cross-owner read cannot succeed through the original ROM.
    let adjacent = (model::RECTANGLE_CHAIN_OFFSET as usize) + model::RECTANGLE_CHAIN_LENGTH;
    for at in adjacent..adjacent + 2 { managed.bus.rom[at] = !rom[at]; }
    for (offset, length) in [(model::RECTANGLE_CHAIN_OFFSET, model::RECTANGLE_CHAIN_LENGTH),
                             (model::RECTANGLE_TABLE_OFFSET, model::RECTANGLE_TABLE_LENGTH)] {
        let start = offset as usize; let end = start + length;
        for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, end] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
            }
        }
    }
    let before_root = owner.observed_tilemap_stats().unwrap().root_bytes[0];
    let before_descriptor = owner.observed_tilemap_descriptor_stats().unwrap().descriptor_bytes;
    let at = adjacent - 2; let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    assert_eq!(owner.observed_tilemap_stats().unwrap().root_bytes[0] - before_root, 2);
    assert_eq!(owner.observed_tilemap_descriptor_stats().unwrap().descriptor_bytes - before_descriptor, 2);
    managed.bus.rom[adjacent..adjacent + 2].copy_from_slice(&rom[adjacent..adjacent + 2]);
    set_roots(&mut managed, &rom, false);
    let initial = raw.save_state().unwrap();
    for index in 0..2 {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[0] = 0x08000000 + model::RECTANGLE_TABLE_OFFSET;
            machine.cpu.regs[5] = 0; machine.cpu.regs[6] = 0x08000000 + model::RECTANGLE_CHAIN_OFFSET + 8;
            machine.cpu.regs[7] = 0; machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.regs[15] = if index == 0 { 0x080db392 } else { 0x081094fa }; machine.cpu.cpsr = 0xbf;
        }
        set_roots(&mut managed, &rom, true);
        let before = owner.observed_tilemap_stats().unwrap().root_bytes;
        let stop = if index == 0 { 0x080db3aa } else { 0x081094fc };
        let mut steps = 0;
        while raw.cpu.regs[15] != stop { assert!(steps < 20); raw.bus.frame_ready = false; raw.step(); steps += 1; }
        let mut steps = 0;
        while managed.cpu.regs[15] != stop {
            assert!(steps < 20); managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        let after = owner.observed_tilemap_stats().unwrap().root_bytes;
        assert_eq!(after[index] - before[index], if index == 0 { 13 } else { 4 });
        assert_eq!(after[1 - index] - before[1 - index], 0);
        assert_eq!(raw.cpu.regs[0], 0x08000000 + data.rectangles[index].source_offset);
        set_roots(&mut managed, &rom, false);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[184,183], "resources":120, "preserved_other_resources":119,
        "previous_tilemap_preserved":true, "root_spans":2, "new_source_bytes":20,
        "poisoned_mirrored_and_mixed_owner_boundaries":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":2, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
