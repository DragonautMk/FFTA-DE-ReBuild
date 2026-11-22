//! EUR Soil Evidence transfer rows, descriptors and shared LZ wrappers.
use super::*;
use gba_core::{capi::RT_API, rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for &(offset, _, wrapper, _) in &descriptor::SOIL_TRANSFER_SPECS {
        for (start, len) in [(offset as usize, 8), (wrapper as usize, 4)] {
            for at in start..start + len { machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] }; }
        }
    }
    for &(offset, _, _) in &row::SOIL_TRANSFER_ROW_SPECS {
        for at in offset as usize..offset as usize + 12 { machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] }; }
    }
}


/// Verify that an extended descriptor or row retains every prior field and source.
pub(super) fn assert_previous_soil_transfer_resource_preserved(
    previous: &std::path::Path, current_directory: &std::path::Path,
    prior: &gba_core::rom_data::Resource, current: &gba_core::rom_data::Resource,
) {
    assert_eq!((&current.id, &current.path, &current.format, &current.dependencies),
        (&prior.id, &prior.path, &prior.format, &prior.dependencies));
    assert_eq!(&current.sources[..prior.sources.len()], prior.sources.as_slice());
    let old: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let new: serde_json::Value = serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    for (key, value) in old.as_object().unwrap() { assert_eq!(&new[key], value, "preserved {key}"); }
}

#[test]
#[ignore = "exact EUR ROM and E240 predecessor; Wine only, Linux/USA pending"]
fn european_soil_transfer_rows_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_TRANSFER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_TRANSFER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (240, 239, 130));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (248, 247, 136));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID {
            let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let new_data: serde_json::Value = serde_json::from_slice(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved {key}"); }
            assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
            let added = &current.sources[prior.sources.len()..];
            assert_eq!((added.len(), added.iter().map(|s| s.length).sum::<u32>()), if prior.id == OBSERVED_TILEMAP_ROW_ID { (6, 72) } else { (8, 52) });
            assert_eq!(current.dependencies, prior.dependencies);
        } else if prior.id == OBSERVED_TILEMAP_ID {
            assert_previous_soil_transfer_resource_preserved(&previous, &prepared.directory, prior, current);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let desc = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    let rows = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let soil_desc = desc.soil_transfer_descriptors.as_ref().unwrap();
    let soil_rows = rows.soil_transfer_rows.as_ref().unwrap();
    desc.validate_lz_dependencies(&source.observed_lz.as_ref().unwrap().decoded).unwrap();
    rows.validate_dependencies(desc).unwrap();
    assert_eq!(soil_desc[1].wrapper, desc.additional_descriptors[2].wrapper);
    assert_eq!(desc.source_spans().iter().filter(|(start, _, _)| *start == 0xb0df28).count(), 1);
    for (offset, bytes, _) in desc.source_spans() { let at = offset as usize; assert_eq!(bytes, &rom[at..at + bytes.len()]); }
    for (offset, bytes, _) in rows.source_spans() { let at = offset as usize; assert_eq!(bytes, rom[at..at + bytes.len()]); }
    let mut bad = rows.clone(); bad.soil_transfer_rows.as_mut().unwrap()[0].descriptor_pointer += 4;
    assert!(bad.validate_dependencies(desc).is_err());
    let mut bad = desc.clone(); bad.soil_transfer_descriptors.as_mut().unwrap()[1].wrapper.opaque_halfword ^= 1;
    assert!(bad.validate().is_err(), "shared wrapper must equal its single owner");
    let mut bad = desc.clone(); bad.soil_transfer_descriptors.as_mut().unwrap()[0].wrapper.flags &= !2;
    assert!(bad.validate().is_err());
    let mut forged = rows.clone(); forged.soil_transfer_rows.as_mut().unwrap()[0].field8 ^= 1;
    let forged_bytes = encode_observed_tilemap_row(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone(); altered.resources.iter_mut().find(|e| e.id == OBSERVED_TILEMAP_ROW_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.observed_tilemap_row = Some((&forged, &forged_bytes));
    assert!(source.validate_files(files).is_err(), "coherent modified file and hash must be rejected"); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    set_backing(&mut managed, &rom, true);
    for &(offset, _, wrapper, _) in &descriptor::SOIL_TRANSFER_SPECS {
        for at in (offset as usize..offset as usize + 8).chain(wrapper as usize..wrapper as usize + 4) {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
        }
    }
    for &(offset, _, _) in &row::SOIL_TRANSFER_ROW_SPECS {
        for at in offset as usize..offset as usize + 12 { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    let edges: Vec<_> = row::SOIL_TRANSFER_ROW_SPECS.iter().flat_map(|&(start, _, _)| [start as usize, start as usize + 12])
        .chain(descriptor::SOIL_TRANSFER_SPECS.iter().flat_map(|&(start, _, wrapper, _)| [start as usize, start as usize + 8, wrapper as usize, wrapper as usize + 4])).collect();
    for edge in &edges { for at in edge.saturating_sub(3)..edge + 3 {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } }
    for edge in edges { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
    } }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    set_backing(&mut managed, &rom, false);

    let mut visited: BTreeSet<u32> = (0x080d_b30a..0x080d_b318).step_by(2).collect();
    visited.extend((0x080d_b4a0..0x080d_b4b2).step_by(2));
    visited.extend((0x080d_b0c6..0x080d_b0de).step_by(2));
    visited.extend((0x080d_b14e..0x080d_b154).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("soil-transfer-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] { for (i, entry) in soil_desc.iter().enumerate() { for mode in 0..4 {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.bus.write32(0x0201_8000, 0x0800_0000 + soil_rows[i].source_offset); machine.bus.write8(0x0201_8007, 0);
            machine.bus.write32(0x0201_8024, 0x0202_42bc);
            machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
            machine.cpu.regs[1] = 0x0800_0000 + soil_rows[i].source_offset; machine.cpu.regs[4] = 0x0201_8000;
            machine.cpu.regs[5] = entry.asset_pointer; machine.cpu.regs[7] = 1; machine.cpu.regs[13] = 0x0300_7c00;
            machine.cpu.regs[15] = [0x080d_b30a, 0x080d_b4a0, 0x080d_b0c6, 0x080d_b14e][mode]; machine.cpu.cpsr = 0xbf;
        }
        set_backing(&mut managed, &rom, true);
        let stop = [0x080d_b318, 0x080d_b4b2, 0x080d_b0de, 0x080d_b154][mode];
        for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
            let mut steps = 0;
            while machine.cpu.regs[15] != stop {
                assert!(steps < 24); machine.bus.frame_ready = false;
                if use_native { table.get(machine.cpu.regs[15] | 1).unwrap()(&RT_API, machine as *mut Machine as *mut std::ffi::c_void); }
                else { machine.step(); } steps += 1;
            }
        }
        set_backing(&mut managed, &rom, false);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "row={i} mode={mode} native={native}"); cases += 1;
    } } }
    assert_eq!(cases, 32);
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[248,247], "soil_rows":4, "soil_descriptors":4, "new_wrappers":3,
        "new_source_bytes":92, "shared_wrapper_owned_once":true, "existing_streams_owned_once":true,
        "previous_fields_and_other_resources_preserved":true, "poisoned_mirrors_and_mixed_boundaries":true,
        "original_interpreter_generated_cases":cases, "full_state_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
