//! EUR adjacent tilemap-row publication and original indexed readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_tilemap_row as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E179 predecessor and Wine; Linux/USA pending"]
fn european_observed_tilemap_rows_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ROWS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TILEMAP_ROWS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (179, 178, 120));
    let old_row: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (180, 179, 120));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ROW_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let new_row = serde_json::to_value(data).unwrap();
    for key in ["source_offset", "field0", "field1", "field2", "descriptor_pointer", "field8"] {
        assert_eq!(new_row[key], old_row[key], "previous row field {key}");
    }
    assert_eq!(source.manifest.resources[119].sources[0], old.resources[119].sources[0]);
    assert_eq!(source.manifest.resources[119].sources.len(), 5);
    for (offset, bytes, _) in data.source_spans() {
        let start = offset as usize;
        assert_eq!(bytes, &rom[start..start + model::SOURCE_LENGTH]);
    }
    assert_eq!(parse_observed_tilemap_row(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.additional_rows[0].descriptor_pointer += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.additional_rows[0].field8 ^= 1;
    let bad_bytes = encode_observed_tilemap_row(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources[119].sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap_row = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent data/file-hash edits must be rejected");
    assert!(model::Catalog::from_rom(&rom, &source.manifest.resources).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut visited: BTreeSet<u32> = (0x080db30a..0x080db318).step_by(2).collect();
    visited.extend((0x080db4a0..0x080db4b2).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("observed-tilemap-rows-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::ADDITIONAL_SPECS[0].0 as usize;
    let end = model::SOURCE_OFFSET as usize + model::SOURCE_LENGTH;
    assert_eq!(end - start, 60);
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    let before = owner.observed_tilemap_row_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    assert_eq!(owner.observed_tilemap_row_stats().unwrap().source_bytes - before, 60);
    for edge in [start, model::SOURCE_OFFSET as usize, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    let mut served = 0;
    for index in 0..5 {
        for field_reader in [false, true] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                machine.bus.ffta_sprites.enabled = false;
                machine.bus.write32(0x02018000, 0x08000000 + start as u32);
                machine.bus.write8(0x02018007, index as u8);
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                machine.cpu.regs[4] = 0x02018000;
                machine.cpu.regs[13] = 0x03007c00;
                machine.cpu.regs[15] = if field_reader { 0x080db4a0 } else { 0x080db30a };
                machine.cpu.cpsr = 0xbf;
            }
            for at in start..end { managed.bus.rom[at] = !rom[at]; }
            let before = owner.observed_tilemap_row_stats().unwrap().source_bytes;
            let stop = if field_reader { 0x080db4b2 } else { 0x080db318 };
            let mut steps = 0;
            while raw.cpu.regs[15] != stop {
                assert!(steps < 16, "original row reader did not reach the stop");
                raw.bus.frame_ready = false; raw.step(); steps += 1;
            }
            let mut steps = 0;
            while managed.cpu.regs[15] != stop {
                assert!(steps < 16, "native row reader did not reach the stop");
                managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
                steps += 1;
            }
            let used = owner.observed_tilemap_row_stats().unwrap().source_bytes - before;
            assert_eq!(used, if field_reader { 1 } else { 4 });
            served += used;
            let offset = start + index * model::SOURCE_LENGTH;
            if field_reader { assert_eq!(raw.cpu.regs[0], u32::from(rom[offset + 1] & 1)); }
            else { assert_eq!(raw.cpu.regs[3], u32::from_le_bytes(rom[offset+4..offset+8].try_into().unwrap())); }
            managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[180,179], "resources":120, "preserved_other_resources":119,
        "previous_row_preserved":true, "additional_rows":4, "additional_source_bytes":48,
        "poisoned_mirrored_and_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":10, "native_reader_source_bytes":served,
        "full_state_clock_equal":true, "ordinary_use":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
