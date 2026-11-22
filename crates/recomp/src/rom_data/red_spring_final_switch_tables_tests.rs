//! EUR final Red Spring switch destinations with original guarded readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::thumb_switch_tables as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, pc: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + pc;
    machine.cpu.cpsr = 0xbf;
}

fn half(rom: &[u8], offset: u32) -> u16 {
    u16::from_le_bytes(
        rom[offset as usize..offset as usize + 2]
            .try_into()
            .unwrap(),
    )
}

fn outside_target(rom: &[u8], spec: model::AdditionalTableSpec) -> u32 {
    let branch = half(rom, spec.guard + 2);
    if branch & 0xff00 == 0xd800 {
        let displacement = i32::from(branch as u8 as i8) * 2;
        (0x0800_0000i64 + i64::from(spec.guard) + 6 + i64::from(displacement)) as u32
    } else {
        assert_eq!(branch & 0xff00, 0xd900);
        let unconditional = half(rom, spec.guard + 4);
        assert_eq!(unconditional & 0xf800, 0xe000);
        let displacement = ((i32::from(unconditional & 0x7ff)) << 21) >> 20;
        (0x0800_0000i64 + i64::from(spec.guard) + 8 + i64::from(displacement)) as u32
    }
}

#[test]
#[ignore = "exact EUR ROM and E224 predecessor; Wine only, Linux/USA pending"]
fn european_red_spring_final_switch_tables_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_RED_SPRING_FINAL_SWITCHES_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_RED_SPRING_FINAL_SWITCHES_PREVIOUS").unwrap());
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
        (224, 223, 130)
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
        (257, 256, 142)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous,
                &prepared.directory,
                prior,
                current,
            );
            assert_eq!(current.sources.len() - prior.sources.len(), 16);
        } else if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
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
    let new_specs = &model::ADDITIONAL_TABLES[16..18];
    assert_eq!(new_specs.len(), 2);
    assert_eq!(
        data.additional_tables[16..18]
            .iter()
            .map(|v| v.destinations.len())
            .sum::<usize>(),
        177
    );
    assert_eq!(
        parse_thumb_switch_tables(
            &fs::read(prepared.directory.join(THUMB_SWITCH_TABLES_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.additional_tables[16].destinations[0].address |= 1;
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["additional_tables"][16]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ThumbSwitchTables>(invalid).is_err());
    let mut forged = data.clone();
    forged.additional_tables[16].destinations[0].address ^= 2;
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
    for spec in model::ADDITIONAL_TABLES[..18].iter().copied() {
        for at in spec.start() as usize..spec.end() as usize {
            managed.bus.rom[at] = !rom[at];
        }
    }
    let before = owner.thumb_switch_tables_stats().unwrap();
    for spec in model::ADDITIONAL_TABLES[..18].iter().copied() {
        for at in spec.start()..spec.end() {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.thumb_switch_tables_stats().unwrap();
    assert_eq!(
        after.additional_source_bytes - before.additional_source_bytes,
        2092
    );
    assert_eq!(after.additional_destinations_mask, 0x3ffff);
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
    for spec in new_specs.iter().copied() {
        for at in (spec.start() as usize - 3..spec.start() as usize)
            .chain(spec.end() as usize..spec.end() as usize + 4)
        {
            if !current_owned(at) {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for spec in new_specs.iter().copied() {
        for edge in [spec.start() as usize, spec.end() as usize] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    if changed.contains(&(at + n)) {
                        rom[at + n] ^ 0x5a
                    } else {
                        rom[at + n]
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
    }
    for spec in model::ADDITIONAL_TABLES[..18].iter().copied() {
        managed.bus.rom[spec.start() as usize..spec.end() as usize]
            .copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut addresses = BTreeSet::new();
    for spec in new_specs.iter().copied() {
        addresses.extend((0x0800_0000 + spec.guard..0x0800_0000 + spec.mov).step_by(2));
    }
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("red-spring-final-switch-destinations-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool, spec: model::AdditionalTableSpec| {
        machine.bus.frame_ready = false;
        let pc = machine.cpu.regs[15];
        if native && pc != 0x0800_0000 + spec.mov {
            table
                .get(pc | 1)
                .unwrap_or_else(|| panic!("Missing generated reader {pc:08x}"))(
                &RT_API,
                machine as *mut Machine as *mut std::ffi::c_void,
            );
        } else {
            machine.step();
        }
    };
    let mut guarded_cases = 0;
    let mut outside_cases = 0;
    let mut outside_load_cases = 0;
    for native in [false, true] {
        for spec in new_specs.iter().copied() {
            let selector = usize::from((half(&rom, spec.guard) >> 8) & 7);
            let shift = half(&rom, spec.reader - 6);
            assert_eq!(shift & 0xffc7, 0x0080);
            let shift_source = usize::from((shift >> 3) & 7);
            assert_eq!(selector, shift_source);
            for index in [0u32, spec.count as u32 - 1] {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(machine, spec.guard);
                    machine.cpu.regs[selector] = index;
                }
                for at in spec.root as usize..spec.end() as usize {
                    managed.bus.rom[at] = !rom[at];
                }
                let before = owner
                    .thumb_switch_tables_stats()
                    .unwrap()
                    .additional_source_bytes;
                let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
                let at = spec.start() as usize + index as usize * 4;
                let destination = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
                for _ in 0..24 {
                    if raw.cpu.regs[15] == destination {
                        break;
                    }
                    execute(&mut raw, false, spec);
                }
                for _ in 0..24 {
                    if managed.cpu.regs[15] == destination {
                        break;
                    }
                    execute(&mut managed, native, spec);
                }
                assert_eq!(
                    (raw.cpu.regs[15], managed.cpu.regs[15]),
                    (destination, destination)
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
                    owner.thumb_pc_literals_stats().unwrap().source_bytes - root_before,
                    4
                );
                managed.bus.rom[spec.root as usize..spec.end() as usize]
                    .copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "switch {:x}, index={index}, native={native}",
                    spec.root
                );
                guarded_cases += 1;
            }
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, spec.guard);
                machine.cpu.regs[selector] = spec.count as u32;
            }
            let stop = outside_target(&rom, spec);
            let before = owner
                .thumb_switch_tables_stats()
                .unwrap()
                .additional_source_bytes;
            for _ in 0..12 {
                if raw.cpu.regs[15] == stop {
                    break;
                }
                execute(&mut raw, false, spec);
            }
            for _ in 0..12 {
                if managed.cpu.regs[15] == stop {
                    break;
                }
                execute(&mut managed, native, spec);
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
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            outside_cases += 1;

            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, spec.reader - 6);
                machine.cpu.regs[shift_source] = spec.count as u32;
            }
            for at in spec.root as usize..spec.end() as usize {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner
                .thumb_switch_tables_stats()
                .unwrap()
                .additional_source_bytes;
            let stop = 0x0800_0000 + spec.mov;
            for _ in 0..12 {
                if raw.cpu.regs[15] == stop {
                    break;
                }
                execute(&mut raw, false, spec);
            }
            for _ in 0..12 {
                if managed.cpu.regs[15] == stop {
                    break;
                }
                execute(&mut managed, native, spec);
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
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            outside_load_cases += 1;
        }
    }
    assert_eq!(
        (guarded_cases, outside_cases, outside_load_cases),
        (8, 4, 4)
    );

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let offset = new_specs[0].start();
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + offset,
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
        for at in offset as usize..offset as usize + 4 {
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
        dma_managed.bus.rom[offset as usize..offset as usize + 4]
            .copy_from_slice(&rom[offset as usize..offset as usize + 4]);
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
        "status":"PASS", "revision":[257,256], "red_spring_tables":2, "red_spring_destinations":177, "red_spring_source_bytes":708,
        "prior_switch_tables_preserved":39, "prior_other_resources_unchanged":120, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_thumb_literals_preserved":true, "prior_copied_halfwords_preserved":true, "legacy_masks_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_guarded_interpreter_generated_cases":guarded_cases,
        "outside_guard_cases":outside_cases, "outside_load_cases":outside_load_cases, "indirect_mov_interpreter_fallback":true,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
