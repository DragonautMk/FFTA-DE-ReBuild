//! EUR Red Spring stack-copy source, original reader and managed ownership.
use super::*;
use gba_core::{capi::RT_API, rom_data::copied_halfword_lookups as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7a00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0810_9630;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E225 predecessor; Wine only, Linux/USA pending"]
fn european_red_spring_final_copy_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_RED_SPRING_FINAL_COPY_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_RED_SPRING_FINAL_COPY_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old.extractor_revision,
            old.decoder_revision,
            old.resources.len()
        ),
        (225, 224, 130)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (248, 247, 136)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(
                &previous,
                &prepared.directory,
                prior,
                current,
            );
            assert_eq!(current.sources.len() - prior.sources.len(), 2);
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == OBSERVED_LZ_ID {
            super::soil_evidence_lz_tests::assert_previous_observed_lz_preserved(
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
        } else if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID || prior.id == OBSERVED_TILEMAP_ID {
            super::soil_transfer_records_tests::assert_previous_soil_transfer_resource_preserved(
                &previous, &prepared.directory, prior, current);
        } else {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(prepared.directory.join(&current.path)).unwrap(),
                fs::read(previous.join(&prior.path)).unwrap()
            );
        }
    }
    let data = &source.copied_halfword_lookups.as_ref().unwrap().decoded;
    let start = model::TABLE_OFFSETS[2];
    let new_span = data
        .source_spans()
        .unwrap()
        .into_iter()
        .find(|(offset, _, _)| *offset == start)
        .unwrap();
    assert_eq!(
        (new_span.0, new_span.1.len(), new_span.2),
        (0x14d84c, 8, "third-halfword-lookup")
    );
    assert_eq!(new_span.1, rom[start as usize..start as usize + 8]);
    assert_eq!(
        parse_copied_halfword_lookups(
            &fs::read(prepared.directory.join(COPIED_HALFWORD_LOOKUPS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.tables.swap(1, 2);
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.tables[2].values[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_copied_halfword_lookups(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == COPIED_HALFWORD_LOOKUPS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.copied_halfword_lookups = Some((&forged, &bytes));
    assert!(
        source.validate_files(files).is_err(),
        "coherent file/hash edits must be rejected"
    );
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let start = start as usize;
    for at in start..start + 8 {
        managed.bus.rom[at] = !rom[at];
    }
    let before = owner.copied_halfword_lookups_stats().unwrap();
    for at in start..start + 8 {
        assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]);
    }
    let after = owner.copied_halfword_lookups_stats().unwrap();
    assert_eq!(after.table_bytes - before.table_bytes, 8);
    assert_eq!(after.table_mask & 4, 4);
    let current_owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|entry| &entry.sources)
            .any(|span| {
                (span.offset as usize..span.offset as usize + span.length as usize).contains(&at)
            })
    };
    let mut changed = BTreeSet::new();
    for at in (start - 3..start).chain(start + 8..start + 12) {
        if !current_owned(at) {
            changed.insert(at);
            managed.bus.rom[at] = rom[at] ^ 0x5a;
        }
    }
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [start, start + 8] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| {
                if changed.contains(&(at + n)) {
                    rom[at + n] ^ 0x5a
                } else {
                    rom[at + n]
                }
            };
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(
                    (RT_API.read16)(ptr, mirror + at as u32),
                    u32::from(u16::from_le_bytes([expected(0), expected(1)]))
                );
                assert_eq!(
                    (RT_API.read32)(ptr, mirror + at as u32),
                    u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)])
                );
            }
        }
    }
    managed.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = [0x0810_9630, 0x0810_9632, 0x0810_9634]
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("red-spring-final-copy-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for native in [false, true] {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            setup(machine);
        }
        for at in start..start + 8 {
            managed.bus.rom[at] = !rom[at];
        }
        for at in 0x109710..0x109714 {
            managed.bus.rom[at] = !rom[at];
        }
        let before = owner.copied_halfword_lookups_stats().unwrap().table_bytes;
        let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
        for _ in 0..512 {
            if raw.cpu.regs[15] == 0x0810_963a {
                break;
            }
            raw.bus.frame_ready = false;
            raw.step();
        }
        for _ in 0..512 {
            let pc = managed.cpu.regs[15];
            if pc == 0x0810_963a {
                break;
            }
            managed.bus.frame_ready = false;
            if native && [0x0810_9630, 0x0810_9632, 0x0810_9634].contains(&pc) {
                table.get(pc | 1).unwrap()(
                    &RT_API,
                    &mut managed as *mut Machine as *mut std::ffi::c_void,
                );
            } else {
                managed.step();
            }
        }
        assert_eq!(
            (raw.cpu.regs[15], managed.cpu.regs[15]),
            (0x0810_963a, 0x0810_963a)
        );
        assert_eq!(
            owner.copied_halfword_lookups_stats().unwrap().table_bytes - before,
            8
        );
        assert_eq!(
            owner.thumb_pc_literals_stats().unwrap().source_bytes - root_before,
            4
        );
        for i in 0..8 {
            assert_eq!(raw.bus.read8(0x0300_7a10 + i), rom[start + i as usize]);
            assert_eq!(managed.bus.read8(0x0300_7a10 + i), rom[start + i as usize]);
        }
        managed.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
        managed.bus.rom[0x109710..0x109714].copy_from_slice(&rom[0x109710..0x109714]);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "original copy native={native}"
        );
    }

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + start as u32);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + start as u32,
                    0x0203_2400,
                    if kind == 2 { 8 } else { 2 },
                ]);
                machine.cpu.regs[15] = 0x0200_0000;
                machine.cpu.cpsr = 0xbf;
                machine
                    .bus
                    .write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        for at in start..start + 8 {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .copied_halfword_lookups_stats()
            .unwrap()
            .table_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .copied_halfword_lookups_stats()
                .unwrap()
                .table_bytes
                > before
        );
        dma_managed.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
        assert_eq!(
            dma_raw.save_state().unwrap(),
            dma_managed.save_state().unwrap(),
            "transfer {kind}"
        );
    }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[248,247], "new_tables":1, "new_source_bytes":8,
        "previous_tables_preserved":2, "prior_other_resources_unchanged":120, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_switch_tables_preserved":true, "prior_thumb_literals_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_and_generated_copy_cases":2,
        "original_helper_and_guest_stack_preserved":true, "dma_cpuset_cpufastset_state_and_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
