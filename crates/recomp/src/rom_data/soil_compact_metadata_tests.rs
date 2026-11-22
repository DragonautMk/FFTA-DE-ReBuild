//! EUR Soil Evidence compact metadata and original byte/halfword readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::{compact_obj_catalog as model, obj_lists}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, path::Path, sync::Arc};

pub(super) fn assert_previous_compact_catalog_preserved(
    previous: &Path,
    current_directory: &Path,
    prior: &gba_core::rom_data::Resource,
    current: &gba_core::rom_data::Resource,
) {
    assert_eq!((&current.id, &current.path), (&prior.id, &prior.path));
    assert_eq!(prior.format, "compact-obj-catalog-v12-json");
    assert_eq!(current.format, "compact-obj-catalog-v13-json");
    let mut dependencies = prior.dependencies.clone();
    dependencies.push(OBJ_STATIC_LISTS_ID.into());
    assert_eq!(current.dependencies, dependencies);
    for source in &prior.sources { assert!(current.sources.iter().any(|candidate| candidate == source)); }
    let old: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let new: serde_json::Value = serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    for (name, value) in old.as_object().unwrap() { assert_eq!(value, &new[name]); }
}

fn setup(machine: &mut Machine, pc: u32, address: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = 0x0800_0000 + address;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + pc;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E235 predecessor; Wine only, Linux/USA pending"]
fn european_soil_compact_metadata_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_COMPACT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_COMPACT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (235, 234, 130));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (248, 247, 136));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == COMPACT_OBJ_CATALOG_ID {
            assert_previous_compact_catalog_preserved(&previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 2);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID || prior.id == OBSERVED_TILEMAP_ID {
            super::soil_transfer_records_tests::assert_previous_soil_transfer_resource_preserved(
                &previous, &prepared.directory, prior, current);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let data = &source.compact_obj_catalog.as_ref().unwrap().decoded;
    let metadata = data.soil_evidence_metadata.as_ref().unwrap();
    assert_eq!((metadata.list_offsets.len(), metadata.frames.len()), (20, 20));
    let lists = &source.obj_lists.as_ref().unwrap().decoded.lists;
    for (index, relative) in metadata.list_offsets.iter().enumerate() {
        let at = model::SOIL_EVIDENCE_ROOT + 4 + u32::from(*relative);
        assert_eq!(at, obj_lists::SOIL_EVIDENCE_LISTS[index].0);
        assert!(lists.iter().any(|list| list.source_offset == at));
        assert_eq!(usize::from(metadata.frames[index].list), index);
    }
    let pieces = metadata.source_pieces().unwrap();
    assert_eq!(pieces.iter().map(|(_, bytes)| bytes.len()).sum::<usize>(), 88);
    for (at, bytes) in &pieces { assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]); }
    assert_eq!(parse_compact_obj_catalog(&fs::read(prepared.directory.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.soil_evidence_metadata.as_mut().unwrap().list_offsets.swap(0, 1); assert!(bad.validate().is_err());
    let mut bad = serde_json::to_value(data).unwrap(); bad["soil_evidence_metadata"]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::CompactObjCatalog>(bad).is_err());
    let mut forged = data.clone(); forged.soil_evidence_metadata.as_mut().unwrap().frames[0].duration ^= 1; forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_compact_obj_catalog(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == COMPACT_OBJ_CATALOG_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.compact_obj_catalog = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for (start, end) in model::SOIL_EVIDENCE_METADATA_RANGES { for at in start as usize..end as usize { managed.bus.rom[at] = !rom[at]; } }
    let before = owner.compact_obj_catalog_stats().unwrap();
    for (start, end) in model::SOIL_EVIDENCE_METADATA_RANGES { for at in start..end { assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]); } }
    let after = owner.compact_obj_catalog_stats().unwrap();
    for i in 0..2 { assert_eq!(after.soil_metadata_bytes[i] - before.soil_metadata_bytes[i], 44); }
    assert_eq!(after.soil_metadata_mask, [(1u64 << 44) - 1; 2]);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for (start, end) in model::SOIL_EVIDENCE_METADATA_RANGES { for at in (start as usize - 3..start as usize).chain(end as usize..end as usize + 4) {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } }
    for (start, end) in model::SOIL_EVIDENCE_METADATA_RANGES { for edge in [start as usize, end as usize] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |n: usize| if changed.contains(&(at + n)) { rom[at + n] ^ 0x5a } else { rom[at + n] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
            assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
        }
    } } }
    for (start, end) in model::SOIL_EVIDENCE_METADATA_RANGES { managed.bus.rom[start as usize..end as usize].copy_from_slice(&rom[start as usize..end as usize]); }
    for at in changed { managed.bus.rom[at] = rom[at]; }

    // These are original scoped LDRB and LDRH instructions from E274.
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let pcs = [0x0801_0cc6, 0x0801_0ea2];
    let blocks = pcs.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("soil-compact-metadata-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] { for (pc, width, dest) in [(0x10cc6, 1usize, 1usize), (0x10ea2, 2, 0)] {
        for at in [model::SOIL_EVIDENCE_ROOT, model::SOIL_EVIDENCE_ROOT + 42, model::SOIL_EVIDENCE_SEQUENCE_ROOT, model::SOIL_EVIDENCE_SEQUENCE_ROOT + 42] {
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, pc, at); }
            for x in at as usize..at as usize + width { managed.bus.rom[x] = !rom[x]; }
            let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
            raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
            if native { table.get((0x0800_0000 + pc) | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); } else { managed.step(); }
            assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, width as u64);
            let expected = if width == 1 { u32::from(rom[at as usize]) } else { u32::from(u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())) };
            assert_eq!((raw.cpu.regs[dest], managed.cpu.regs[dest]), (expected, expected));
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (0x0800_0000 + pc + 2, 0x0800_0000 + pc + 2));
            managed.bus.rom[at as usize..at as usize + width].copy_from_slice(&rom[at as usize..at as usize + width]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "pc={pc:x}, at={at:x}, native={native}");
            cases += 1;
        }
    } }
    assert_eq!(cases, 16);

    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    for (group, (offset, end)) in model::SOIL_EVIDENCE_METADATA_RANGES.iter().copied().enumerate() { for kind in 0..3 {
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 16);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + offset, 0x0203_2400, if kind == 2 { 8 } else { 16 }]);
                machine.cpu.regs[15] = 0x0200_0000; machine.cpu.cpsr = 0xbf;
                machine.bus.write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        for at in offset as usize..end as usize { transfer_managed.bus.rom[at] = !rom[at]; }
        let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.compact_obj_catalog_stats().unwrap().soil_metadata_bytes[group];
        for machine in [&mut transfer_raw, &mut transfer_managed] { if kind == 0 { machine.bus.write16(0x0400_00de, 0x8000); } else { machine.step(); } }
        assert!(transfer_owner.compact_obj_catalog_stats().unwrap().soil_metadata_bytes[group] > before);
        transfer_managed.bus.rom[offset as usize..end as usize].copy_from_slice(&rom[offset as usize..end as usize]);
        assert_eq!(transfer_raw.save_state().unwrap(), transfer_managed.save_state().unwrap(), "group={group}, transfer={kind}");
    } }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[248,247], "resources":136, "previous_compact_catalog_preserved":true,
        "previous_other_resources_unchanged":123, "previous_copied_halfwords_preserved":true, "previous_literals_preserved":true, "previous_switch_tables_preserved":true, "linked_obj_lists":20, "metadata_source_bytes":88,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":16,
        "dma_cpuset_cpufastset_state_clock_cases":6, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
