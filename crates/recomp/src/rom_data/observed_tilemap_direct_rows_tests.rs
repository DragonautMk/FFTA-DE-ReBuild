//! EUR tilemap rows linked to the additional direct descriptors.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap_row as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn poison(machine: &mut Machine, rom: &[u8], invert: bool) {
    for &(offset, _, _) in &model::DIRECT_ROW_SPECS {
        for at in offset as usize..offset as usize + model::SOURCE_LENGTH {
            machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM, E199 predecessor and Wine; Linux/USA pending"]
fn european_direct_tilemap_rows_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ROWS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ROWS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (199, 198, 123));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (200, 199, 123));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ROW_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_ROW_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == OBSERVED_TILEMAP_ROW_ID).unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 7);
    let rows = data.direct_tilemap_rows.as_ref().unwrap();
    for row in rows { let at = row.source_offset as usize; assert_eq!(row.source_bytes(), rom[at..at + model::SOURCE_LENGTH]); }
    assert_eq!(parse_observed_tilemap_row(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap(), *data);
    let descriptors = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    data.validate_dependencies(descriptors).unwrap();
    let mut missing = descriptors.clone(); missing.additional_tilemap_descriptors = None;
    assert!(data.validate_dependencies(&missing).is_err());
    let mut bad = data.clone(); bad.direct_tilemap_rows.as_mut().unwrap()[0].field8 ^= 1;
    let bad_bytes = encode_observed_tilemap_row(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_TILEMAP_ROW_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap_row = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected"); drop(dataset);

    let mut visited: BTreeSet<u32> = (0x080db30a..0x080db318).step_by(2).collect();
    visited.extend((0x080db4a0..0x080db4b2).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("direct-tilemap-rows-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    poison(&mut managed, &rom, true);
    for row in rows {
        let start = row.source_offset as usize; let end = start + model::SOURCE_LENGTH;
        for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
        for edge in [start, end] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
            }
        }
    }
    poison(&mut managed, &rom, false); let initial = raw.save_state().unwrap();
    let base = model::DIRECT_ROW_SPECS[0].0;
    for (index, row) in rows.iter().enumerate() {
        for field_reader in [false, true] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                machine.bus.write32(0x02018000, 0x08000000 + base); machine.bus.write8(0x02018007, (index * 2) as u8);
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                machine.cpu.regs[4] = 0x02018000; machine.cpu.regs[13] = 0x03007c00;
                machine.cpu.regs[15] = if field_reader { 0x080db4a0 } else { 0x080db30a }; machine.cpu.cpsr = 0xbf;
            }
            poison(&mut managed, &rom, true);
            let before = owner.observed_tilemap_row_stats().unwrap().direct_tilemap_row_bytes;
            let stop = if field_reader { 0x080db4b2 } else { 0x080db318 };
            let mut steps = 0;
            while raw.cpu.regs[15] != stop { assert!(steps < 16); raw.bus.frame_ready = false; raw.step(); steps += 1; }
            let mut steps = 0;
            while managed.cpu.regs[15] != stop {
                assert!(steps < 16); managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1;
            }
            assert_eq!(owner.observed_tilemap_row_stats().unwrap().direct_tilemap_row_bytes - before, if field_reader { 1 } else { 4 });
            if field_reader { assert_eq!(raw.cpu.regs[0], u32::from(row.field1 & 1)); }
            else { assert_eq!(raw.cpu.regs[3], row.descriptor_pointer); }
            poison(&mut managed, &rom, false);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "row {index} reader {field_reader}");
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[200,199], "resources":123, "preserved_other_resources":122,
        "previous_rows_preserved":true, "rows":7, "source_bytes":84, "descriptor_dependencies_verified":true,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":14, "full_state_clock_equal":true,
        "ordinary_scene":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
