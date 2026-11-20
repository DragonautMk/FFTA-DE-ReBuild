//! EUR transfer rows, linked descriptors and their existing LZ streams.
use super::*;
use gba_core::{capi::RT_API, rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn set_backing(machine: &mut Machine, rom: &[u8], invert: bool, include_lz_prefix: bool) {
    for &(offset, _, wrapper, _) in &descriptor::TRANSFER_SPECS {
        for (start, length) in [(offset as usize, descriptor::SOURCE_LENGTH),
                                (wrapper as usize, descriptor::WRAPPER_LENGTH + if include_lz_prefix { 4 } else { 0 })] {
            for at in start..start + length { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
        }
    }
    for &(offset, _, _) in &row::TRANSFER_SPECS {
        for at in offset as usize..offset as usize + row::SOURCE_LENGTH { machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] }; }
    }
}

#[test]
#[ignore = "exact EUR ROM, E181 predecessor and Wine; Linux/USA pending"]
fn european_transfer_records_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TRANSFER_RECORDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TRANSFER_RECORDS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (181, 180, 120));
    let old_descriptor: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_DESCRIPTOR_FILE)).unwrap()).unwrap();
    let old_row: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (182, 181, 120));
    for prior in &old.resources {
        if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let desc = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    let rows = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let new_descriptor = serde_json::to_value(desc).unwrap();
    let new_row = serde_json::to_value(rows).unwrap();
    for (old, new) in [(&old_descriptor, &new_descriptor), (&old_row, &new_row)] {
        for (key, value) in old.as_object().unwrap() { assert_eq!(&new[key], value, "preserved field {key}"); }
    }
    assert_eq!(&source.manifest.resources[118].sources[..9], &old.resources[118].sources);
    assert_eq!(&source.manifest.resources[119].sources[..5], &old.resources[119].sources);
    assert_eq!((source.manifest.resources[118].sources.len(), source.manifest.resources[119].sources.len()), (15, 8));
    desc.validate_lz_dependencies(&source.observed_lz.as_ref().unwrap().decoded).unwrap();
    rows.validate_dependencies(desc).unwrap();
    for (offset, bytes, _) in desc.source_spans() { let at = offset as usize; assert_eq!(bytes, &rom[at..at + bytes.len()]); }
    for (offset, bytes, _) in rows.source_spans() { let at = offset as usize; assert_eq!(bytes, &rom[at..at + bytes.len()]); }
    assert_eq!(parse_observed_tilemap_descriptor(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_DESCRIPTOR_FILE)).unwrap()).unwrap(), *desc);
    assert_eq!(parse_observed_tilemap_row(&fs::read(prepared.directory.join(OBSERVED_TILEMAP_ROW_FILE)).unwrap()).unwrap(), *rows);
    let mut bad = rows.clone(); bad.transfer_rows[0].descriptor_pointer += 4; assert!(bad.validate_dependencies(desc).is_err());
    let mut bad = desc.clone(); bad.transfer_descriptors[0].wrapper.flags &= !2; assert!(bad.validate().is_err());
    let mut bad = rows.clone(); bad.transfer_rows[0].field8 ^= 1;
    let bad_bytes = encode_observed_tilemap_row(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone(); altered_manifest.resources[119].sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.observed_tilemap_row = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent data/file-hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut visited: BTreeSet<u32> = (0x080db30a..0x080db318).step_by(2).collect();
    visited.extend((0x080db4a0..0x080db4b2).step_by(2));
    visited.extend((0x080db0c6..0x080db0de).step_by(2));
    visited.extend((0x080db14e..0x080db154).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("observed-transfer-records-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    set_backing(&mut managed, &rom, true, true);
    for &(offset, _, wrapper, _) in &descriptor::TRANSFER_SPECS {
        for at in (offset as usize..offset as usize + descriptor::SOURCE_LENGTH).chain(wrapper as usize..wrapper as usize + descriptor::WRAPPER_LENGTH) {
            assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]);
        }
    }
    for &(offset, _, _) in &row::TRANSFER_SPECS {
        for at in offset as usize..offset as usize + row::SOURCE_LENGTH { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    }
    for edge in [0x399774usize, 0x39978c, 0x3997b0, 0xb02ea0, 0xb02ea4, 0xb030bc, 0xb030c0, 0xb151d4, 0xb151d8] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        }
    }
    let before_desc = owner.observed_tilemap_descriptor_stats().unwrap();
    let before_row = owner.observed_tilemap_row_stats().unwrap().transfer_source_bytes;
    let at = 0x39978a; let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    assert_eq!(owner.observed_tilemap_descriptor_stats().unwrap().transfer_descriptor_bytes - before_desc.transfer_descriptor_bytes, 2);
    assert_eq!(owner.observed_tilemap_row_stats().unwrap().transfer_source_bytes - before_row, 2);
    for &(_, _, wrapper, _) in &descriptor::TRANSFER_SPECS {
        let before_desc = owner.observed_tilemap_descriptor_stats().unwrap().transfer_wrapper_bytes;
        let before_lz = owner.observed_lz_stats().unwrap().source_bytes;
        let at = wrapper as usize + 2; let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
        assert_eq!(owner.observed_tilemap_descriptor_stats().unwrap().transfer_wrapper_bytes - before_desc, 2);
        assert_eq!(owner.observed_lz_stats().unwrap().source_bytes - before_lz, 2);
    }
    set_backing(&mut managed, &rom, false, true);
    let initial = raw.save_state().unwrap();
    for (index, entry) in desc.transfer_descriptors.iter().enumerate() {
        for mode in 0..4 {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                machine.bus.write32(0x02018000, 0x0839978c); machine.bus.write8(0x02018007, index as u8);
                machine.bus.write32(0x02018024, 0x020242bc);
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                machine.cpu.regs[1] = 0x0839978c + index as u32 * 12; machine.cpu.regs[4] = 0x02018000;
                machine.cpu.regs[5] = entry.asset_pointer; machine.cpu.regs[7] = 1; machine.cpu.regs[13] = 0x03007c00;
                machine.cpu.regs[15] = [0x080db30a, 0x080db4a0, 0x080db0c6, 0x080db14e][mode]; machine.cpu.cpsr = 0xbf;
            }
            set_backing(&mut managed, &rom, true, false);
            let before_row = owner.observed_tilemap_row_stats().unwrap().transfer_source_bytes;
            let before_desc = owner.observed_tilemap_descriptor_stats().unwrap();
            let stop = [0x080db318, 0x080db4b2, 0x080db0de, 0x080db154][mode];
            let mut steps = 0;
            while raw.cpu.regs[15] != stop { assert!(steps < 20); raw.bus.frame_ready = false; raw.step(); steps += 1; }
            let mut steps = 0;
            while managed.cpu.regs[15] != stop {
                assert!(steps < 20); managed.bus.frame_ready = false;
                table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); steps += 1;
            }
            let after = owner.observed_tilemap_descriptor_stats().unwrap();
            // The descriptor prefix also reads the row's halfword at offset 2.
            assert_eq!(owner.observed_tilemap_row_stats().unwrap().transfer_source_bytes - before_row, [4, 1, 6, 0][mode]);
            assert_eq!(after.transfer_descriptor_bytes - before_desc.transfer_descriptor_bytes, if mode == 2 { 4 } else { 0 });
            assert_eq!(after.transfer_wrapper_bytes - before_desc.transfer_wrapper_bytes, if mode >= 2 { 1 } else { 0 });
            if mode == 0 { assert_eq!(raw.cpu.regs[3], 0x08000000 + entry.source_offset); }
            if mode == 1 { assert_eq!(raw.cpu.regs[0], u32::from(rows.transfer_rows[index].field1 & 1)); }
            if mode == 2 { assert_eq!(raw.cpu.regs[5], entry.asset_pointer); }
            if mode >= 2 { assert_eq!(raw.cpu.regs[0], u32::from(entry.wrapper.flags & if mode == 2 { 1 } else { 2 })); }
            set_backing(&mut managed, &rom, false, false);
            assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[182,181], "resources":120, "preserved_other_resources":118,
        "previous_descriptor_and_row_preserved":true, "transfer_rows":3, "transfer_descriptors":3, "transfer_wrappers":3, "new_source_bytes":72,
        "poisoned_mirrored_and_mixed_owner_boundaries":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":12, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
