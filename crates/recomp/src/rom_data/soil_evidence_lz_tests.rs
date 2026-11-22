//! European Soil Evidence LZ77 publication and source consumption.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{lz77, observed_lz::{ObservedLzBlock, ADDITIONAL_SPECS}},
    Bus, Machine,
};
use std::{path::Path, sync::Arc};

pub(super) fn assert_previous_observed_lz_preserved(
    previous: &Path,
    current_directory: &Path,
    prior: &gba_core::rom_data::Resource,
    current: &gba_core::rom_data::Resource,
) {
    assert_eq!((&current.id, &current.path, &current.format, &current.dependencies),
        (&prior.id, &prior.path, &prior.format, &prior.dependencies));
    assert_eq!(&current.sources[..prior.sources.len()], prior.sources.as_slice());
    let old: ObservedLzBlock = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let new: ObservedLzBlock = serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    assert_eq!((&new.source_offset, &new.encoded_source, &new.decoded),
        (&old.source_offset, &old.encoded_source, &old.decoded));
    assert_eq!(&new.additional_blocks[..old.additional_blocks.len()], old.additional_blocks.as_slice());
}

#[test]
#[ignore = "exact EUR ROM and E231 predecessor; Wine only, Linux/USA pending"]
fn european_soil_evidence_lz_publication_and_swi() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_LZ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_LZ_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (231, 230, 130));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (248, 247, 136));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == OBSERVED_LZ_ID {
            assert_previous_observed_lz_preserved(&previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 3);
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == INDEXED_WORDS_3959_ID {
            super::soil_e3838_tests::assert_previous_indexed_words_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == OBJ_STATIC_LISTS_ID {
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == COMPACT_OBJ_CATALOG_ID {
            super::soil_compact_metadata_tests::assert_previous_compact_catalog_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
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
    let data = &source.observed_lz.as_ref().unwrap().decoded;
    let specs = &ADDITIONAL_SPECS[13..16];
    assert_eq!(data.additional_blocks.len(), 16);
    assert_eq!(specs.iter().map(|s| s.encoded_length).sum::<usize>(), 1335);
    assert_eq!(specs.iter().map(|s| s.decoded_length).sum::<usize>(), 1640);
    for (stream, spec) in data.additional_blocks[13..16].iter().zip(specs) {
        assert_eq!(stream.source_offset, spec.offset);
        assert_eq!(&stream.encoded_source, &rom[spec.offset as usize..spec.offset as usize + spec.encoded_length]);
        assert_eq!(stream.decoded.len(), spec.decoded_length);
    }
    assert_eq!(parse_observed_lz(&fs::read(prepared.directory.join(OBSERVED_LZ_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.additional_blocks.swap(13, 14); assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.additional_blocks[13].decoded[0] ^= 1; assert!(bad.validate().is_err());
    let mut forged = data.clone();
    forged.additional_blocks[13].encoded_source[5] ^= 1;
    forged.additional_blocks[13].decoded = lz77::decode(&forged.additional_blocks[13].encoded_source, specs[0].decoded_length).unwrap().bytes;
    forged.validate().unwrap();
    let bytes = encode_observed_lz(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBSERVED_LZ_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.observed_lz = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    for spec in specs { for at in spec.offset as usize..spec.offset as usize + spec.encoded_length { managed.bus.rom[at] = !rom[at]; } }
    let before = owner.observed_lz_stats().unwrap();
    for spec in specs { for at in spec.offset as usize..spec.offset as usize + spec.encoded_length {
        assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]);
    } }
    let after = owner.observed_lz_stats().unwrap();
    for i in 0..3 { assert_eq!(after.soil_source_bytes_by_block[i] - before.soil_source_bytes_by_block[i], specs[i].encoded_length as u64); }
    for spec in specs { for edge in [spec.offset as usize, spec.offset as usize + spec.encoded_length] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
            }
        }
    } }
    for spec in specs { let start = spec.offset as usize; managed.bus.rom[start..start + spec.encoded_length].copy_from_slice(&rom[start..start + spec.encoded_length]); }
    let initial = raw.save_state().unwrap();
    for (index, (spec, stream)) in specs.iter().zip(&data.additional_blocks[13..16]).enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs[0] = 0x0800_0000 + spec.offset;
            machine.cpu.regs[1] = 0x0202_42bc;
            machine.cpu.regs[15] = 0x0814_5254;
            machine.cpu.cpsr = 0xbf;
        }
        let start = spec.offset as usize;
        for at in start..start + spec.encoded_length { managed.bus.rom[at] = !rom[at]; }
        let before = owner.observed_lz_stats().unwrap().soil_hle_calls_by_block;
        raw.step(); managed.step();
        let after = owner.observed_lz_stats().unwrap().soil_hle_calls_by_block;
        for i in 0..3 { assert_eq!(after[i] - before[i], u64::from(i == index)); }
        assert_eq!(&raw.bus.ewram[0x242bc..0x242bc + stream.decoded.len()], &stream.decoded);
        assert_eq!(&managed.bus.ewram[0x242bc..0x242bc + stream.decoded.len()], &stream.decoded);
        managed.bus.rom[start..start + spec.encoded_length].copy_from_slice(&rom[start..start + spec.encoded_length]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    }
    assert!(managed.bus.decoded_observed_lz(0x0200_0000).is_none());
    assert!(managed.bus.decoded_observed_lz(0x0800_0000 + specs[0].offset + 1).is_none());

    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    for (index, spec) in specs.iter().enumerate() { for kind in 0..3 {
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + spec.offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 32);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + spec.offset, 0x0203_2400, if kind == 2 { 16 } else { 32 }]);
                machine.cpu.regs[15] = 0x0200_0000; machine.cpu.cpsr = 0xbf;
                machine.bus.write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        let start = spec.offset as usize;
        for at in start..start + spec.encoded_length { transfer_managed.bus.rom[at] = !rom[at]; }
        let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.observed_lz_stats().unwrap().soil_source_bytes_by_block[index];
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            if kind == 0 { machine.bus.write16(0x0400_00de, 0x8000); } else { machine.step(); }
        }
        assert!(transfer_owner.observed_lz_stats().unwrap().soil_source_bytes_by_block[index] > before);
        transfer_managed.bus.rom[start..start + spec.encoded_length].copy_from_slice(&rom[start..start + spec.encoded_length]);
        assert_eq!(transfer_raw.save_state().unwrap(), transfer_managed.save_state().unwrap(), "stream {index}, transfer {kind}");
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
        "status":"PASS", "revision":[248,247], "resources":136, "previous_other_resources_unchanged":120, "previous_copied_halfwords_preserved":true, "previous_switch_tables_preserved":true, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "previous_literals_preserved":true,
        "previous_streams_preserved":14, "new_streams":3, "new_encoded_bytes":1335, "new_decoded_bytes":1640,
        "poisoned_mirrored_and_boundary_reads":true, "original_swi_state_clock_cases":3,
        "dma_cpuset_cpufastset_state_clock_cases":9, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
