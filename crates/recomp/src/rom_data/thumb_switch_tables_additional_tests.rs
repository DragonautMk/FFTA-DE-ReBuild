//! Four additional EUR switch tables whose root words already have a literal owner.
use super::*;
use gba_core::{capi::RT_API, rom_data::thumb_switch_tables as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, path::Path, sync::Arc};

pub(super) fn assert_previous_switch_tables_preserved(
    previous: &Path,
    current_directory: &Path,
    prior: &gba_core::rom_data::Resource,
    current: &gba_core::rom_data::Resource,
) {
    assert_eq!(
        (
            &current.id,
            &current.path,
            &current.format,
            &current.dependencies
        ),
        (&prior.id, &prior.path, &prior.format, &prior.dependencies)
    );
    assert_eq!(
        &current.sources[..prior.sources.len()],
        prior.sources.as_slice()
    );
    assert_eq!(
        current.sources.len(),
        model::TABLE_COUNT + model::ADDITIONAL_TABLE_COUNT
    );
    let previous_json: serde_json::Value =
        serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let current_json: serde_json::Value =
        serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    assert_eq!(previous_json["tables"], current_json["tables"]);
    if let Some(previous_additional) = previous_json.get("additional_tables") {
        let old = previous_additional.as_array().unwrap();
        let new = current_json["additional_tables"].as_array().unwrap();
        assert_eq!(old.as_slice(), &new[..old.len()]);
    }
}

fn setup(machine: &mut Machine, which: usize, index: u32, guarded: bool) {
    let spec = model::ADDITIONAL_TABLES[which];
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.cpsr = 0xbf;
    match which {
        0 => machine.cpu.regs[1] = index,
        1 | 3 => machine.cpu.regs[2] = index,
        2 => {
            machine.cpu.regs[1] = index;
            machine.cpu.regs[2] = index.wrapping_mul(4);
        }
        _ => unreachable!(),
    }
    machine.cpu.regs[15] = 0x0800_0000 + if guarded { spec.guard } else { spec.reader };
}

#[test]
#[ignore = "exact EUR ROM and E214 predecessor; Wine only, Linux/USA pending"]
fn european_additional_switch_destinations_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ADDITIONAL_THUMB_SWITCHES_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_ADDITIONAL_THUMB_SWITCHES_PREVIOUS").unwrap());
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
        (214, 213, 127)
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
        if prior.id == THUMB_SWITCH_TABLES_ID {
            assert_previous_switch_tables_preserved(&previous, &prepared.directory, prior, current);
        } else if prior.id == gba_core::rom_data::OBJ_DIMENSIONS_ID {
            super::obj_derived_dimensions_tests::assert_previous_obj_dimensions_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(
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
    let data = &source.thumb_switch_tables.as_ref().unwrap().decoded;
    assert_eq!(data.additional_tables.len(), model::ADDITIONAL_TABLE_COUNT);
    assert_eq!(
        data.additional_tables
            .iter()
            .take(4)
            .map(|table| table.destinations.len())
            .sum::<usize>(),
        163
    );
    for (table, spec) in data.additional_tables.iter().take(4).zip(model::ADDITIONAL_TABLES[..4].iter().copied()) {
        assert_eq!(
            table.destination_bytes(),
            rom[spec.start() as usize..spec.end() as usize]
        );
        let root_owner = old
            .resources
            .iter()
            .find(|resource| resource.id == THUMB_PC_LITERALS_ID)
            .unwrap();
        assert!(root_owner
            .sources
            .iter()
            .any(|src| src.offset == spec.root && src.length == 4));
        assert!(old
            .resources
            .iter()
            .flat_map(|resource| &resource.sources)
            .all(|src| src.offset + src.length <= spec.end() || src.offset >= spec.end() + 4));
    }
    assert_eq!(
        parse_thumb_switch_tables(
            &fs::read(prepared.directory.join(THUMB_SWITCH_TABLES_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.additional_tables.swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["additional_tables"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ThumbSwitchTables>(invalid).is_err());
    let mut forged = data.clone();
    forged.additional_tables[0].destinations[0].address ^= 2;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_thumb_switch_tables(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == THUMB_SWITCH_TABLES_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.thumb_switch_tables = Some((&forged, &bytes));
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
    for spec in model::TABLES {
        for at in spec.root as usize..spec.end() as usize {
            managed.bus.rom[at] = !rom[at];
        }
    }
    for spec in model::ADDITIONAL_TABLES[..4].iter().copied() {
        for at in spec.root as usize..spec.end() as usize {
            managed.bus.rom[at] = !rom[at];
        }
    }
    let before = owner.thumb_switch_tables_stats().unwrap();
    let roots_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
    for spec in model::TABLES {
        for at in spec.root..spec.end() {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    for spec in model::ADDITIONAL_TABLES[..4].iter().copied() {
        for at in spec.root..spec.end() {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.thumb_switch_tables_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 2876 + 652);
    assert_eq!(after.roots_mask, (1 << 23) - 1);
    assert_eq!(after.destinations_mask, (1 << 23) - 1);
    assert_eq!(
        after.additional_source_bytes - before.additional_source_bytes,
        652
    );
    assert_eq!(after.additional_destinations_mask, 0xf);
    assert_eq!(
        owner.thumb_pc_literals_stats().unwrap().source_bytes - roots_before,
        16
    );
    for spec in model::ADDITIONAL_TABLES[..4].iter().copied() {
        for at in spec.end() as usize..spec.end() as usize + 4 {
            managed.bus.rom[at] = rom[at] ^ 0x5a;
        }
        for edge in [spec.start() as usize, spec.end() as usize] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    let x = at + n;
                    if (spec.end() as usize..spec.end() as usize + 4).contains(&x) {
                        rom[x] ^ 0x5a
                    } else {
                        rom[x]
                    }
                };
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                    assert_eq!(
                        (RT_API.read16)(ptr, mirror + at as u32),
                        u32::from(u16::from_le_bytes([expected(0), expected(1)])),
                        "read16 {at:x}"
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]),
                        "read32 {at:x}"
                    );
                }
            }
        }
        managed.bus.rom[spec.root as usize..spec.end() as usize + 4]
            .copy_from_slice(&rom[spec.root as usize..spec.end() as usize + 4]);
    }
    for spec in model::TABLES {
        managed.bus.rom[spec.root as usize..spec.end() as usize]
            .copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
    }

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut addresses = BTreeSet::new();
    for spec in model::ADDITIONAL_TABLES[..4].iter().copied() {
        addresses.insert(0x0800_0000 + spec.guard);
        addresses.insert(0x0800_0000 + spec.guard + 2);
        addresses.extend((0x0800_0000 + spec.reader..0x0800_0000 + spec.mov).step_by(2));
    }
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("additional-thumb-switch-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool| {
        machine.bus.frame_ready = false;
        let pc = machine.cpu.regs[15];
        let is_indirect_mov = model::ADDITIONAL_TABLES
            .iter()
            .any(|spec| pc == 0x0800_0000 + spec.mov);
        if native && !is_indirect_mov {
            table
                .get(pc | 1)
                .unwrap_or_else(|| panic!("Missing generated reader at {pc:08x}"))(
                &RT_API,
                machine as *mut Machine as *mut std::ffi::c_void,
            );
        } else {
            machine.step();
        }
    };
    for native in [false, true] {
        for (which, spec) in model::ADDITIONAL_TABLES[..4].iter().copied().enumerate() {
            for index in [0u32, spec.count as u32 - 1] {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(machine, which, index, true);
                }
                for at in spec.root as usize..spec.end() as usize {
                    managed.bus.rom[at] = !rom[at];
                }
                let before = owner
                    .thumb_switch_tables_stats()
                    .unwrap()
                    .additional_source_bytes;
                let roots_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
                let at = spec.start() as usize + index as usize * 4;
                let expected = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
                for _ in 0..24 {
                    if raw.cpu.regs[15] == expected {
                        break;
                    }
                    execute(&mut raw, false);
                }
                for _ in 0..24 {
                    if managed.cpu.regs[15] == expected {
                        break;
                    }
                    execute(&mut managed, native);
                }
                assert_eq!(
                    (raw.cpu.regs[15], managed.cpu.regs[15]),
                    (expected, expected),
                    "root {:x}, index {index}, native {native}",
                    spec.root
                );
                assert_eq!(
                    owner
                        .thumb_switch_tables_stats()
                        .unwrap()
                        .additional_source_bytes
                        - before,
                    4
                );
                assert_eq!(
                    owner.thumb_pc_literals_stats().unwrap().source_bytes - roots_before,
                    4
                );
                managed.bus.rom[spec.root as usize..spec.end() as usize]
                    .copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "root {:x}, index {index}, native {native}",
                    spec.root
                );
            }
            // Enter the original load sequence outside the projected range; no guest guard is changed.
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, which, spec.count as u32, false);
            }
            for at in spec.root as usize..spec.end() as usize {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner
                .thumb_switch_tables_stats()
                .unwrap()
                .additional_source_bytes;
            let stop = 0x0800_0000 + spec.mov;
            for _ in 0..16 {
                if raw.cpu.regs[15] == stop {
                    break;
                }
                execute(&mut raw, false);
            }
            for _ in 0..16 {
                if managed.cpu.regs[15] == stop {
                    break;
                }
                execute(&mut managed, native);
            }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop));
            assert_eq!(
                owner
                    .thumb_switch_tables_stats()
                    .unwrap()
                    .additional_source_bytes
                    - before,
                0
            );
            managed.bus.rom[spec.root as usize..spec.end() as usize]
                .copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "outside root {:x}, native {native}",
                spec.root
            );
        }
    }

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let spec = model::ADDITIONAL_TABLES[2];
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + spec.start());
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + spec.start(),
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
        for at in spec.start() as usize..spec.end() as usize {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .thumb_switch_tables_stats()
            .unwrap()
            .additional_source_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .thumb_switch_tables_stats()
                .unwrap()
                .additional_source_bytes
                > before
        );
        dma_managed.bus.rom[spec.start() as usize..spec.end() as usize]
            .copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
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
        "status":"PASS", "revision":[248,247], "additional_tables":4, "destinations":163, "source_bytes":652,
        "prior_tables_preserved":23, "prior_other_resources_unchanged":117, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_thumb_literals_preserved":true, "prior_obj_dimensions_preserved":true, "legacy_masks_preserved":true,
        "existing_literal_roots_preserved":true, "poisoned_mixed_boundaries_and_mirrors":true,
        "original_guarded_interpreter_generated_prefix_cases":16, "indirect_mov_interpreter_fallback":true, "outside_load_sequence_cases":8,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
