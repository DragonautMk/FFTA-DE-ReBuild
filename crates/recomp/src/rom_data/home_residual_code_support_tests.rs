//! EUR home-scene scoped literal and switch-table readers.
use super::*;
use gba_core::rom_data::thumb_switch_tables as switches;
use gba_core::{capi::RT_API, rom_data::thumb_pc_literals as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, reader: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + reader;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E220 predecessor; Wine only, Linux/USA pending"]
fn european_home_residual_code_support_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_HOME_RESIDUAL_CODE_SUPPORT_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_HOME_RESIDUAL_CODE_SUPPORT_PREVIOUS").unwrap());
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
        (220, 219, 130)
    );
    let old_json: serde_json::Value =
        serde_json::from_slice(&fs::read(previous.join(THUMB_PC_LITERALS_FILE)).unwrap()).unwrap();
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
        if prior.id == THUMB_PC_LITERALS_ID {
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
                &current.sources[..model::LITERAL_COUNT
                    + model::ADDITIONAL_COUNT
                    + model::OBSERVED_COUNT
                    + model::HOME_ANCHORED_COUNT],
                prior.sources.as_slice()
            );
            assert_eq!(
                current.sources.len(),
                model::LITERAL_COUNT
                    + model::ADDITIONAL_COUNT
                    + model::OBSERVED_COUNT
                    + model::HOME_ANCHORED_COUNT
                    + model::HOME_OBSERVED_COUNT
                    + model::RED_SPRING_OBSERVED_COUNT
                    + model::RED_SPRING_ANCHORED_COUNT
                    + model::POST_INDEXED_OBSERVED_COUNT
                    + model::POST_INDEXED_ANCHORED_COUNT
                    + model::BANK_OBSERVED_COUNT + model::BLUE_SCREW_OBSERVED_COUNT + model::SOIL_EVIDENCE_OBSERVED_COUNT + model::SOIL_146_OBSERVED_COUNT
            );
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous,
                &prepared.directory,
                prior,
                current,
            );
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
    let data = &source.thumb_pc_literals.as_ref().unwrap().decoded;
    assert_eq!(
        old_json["literals"],
        serde_json::to_value(&data.literals).unwrap()
    );
    assert_eq!(
        old_json["additional"],
        serde_json::to_value(&data.additional).unwrap()
    );
    assert_eq!(
        old_json["observed"],
        serde_json::to_value(&data.observed).unwrap()
    );
    assert_eq!(
        old_json["home_anchored"],
        serde_json::to_value(&data.home_anchored).unwrap()
    );
    assert_eq!(data.home_observed.len(), 40);
    for literal in &data.home_observed {
        let at = literal.source_offset as usize;
        assert_eq!(literal.value.to_le_bytes(), rom[at..at + 4]);
    }
    let switch_data = &source.thumb_switch_tables.as_ref().unwrap().decoded;
    assert_eq!(switch_data.additional_tables.len(), switches::ADDITIONAL_TABLE_COUNT);
    assert_eq!(
        parse_thumb_pc_literals(
            &fs::read(prepared.directory.join(THUMB_PC_LITERALS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.home_observed.swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["home_observed"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ThumbPcLiterals>(invalid).is_err());
    let mut forged = data.clone();
    forged.home_observed[0].value ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_thumb_pc_literals(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.thumb_pc_literals = Some((&forged, &bytes));
    assert!(
        source.validate_files(files).is_err(),
        "coherent file/hash edits must be rejected"
    );
    let mut forged_switch = switch_data.clone();
    forged_switch.additional_tables[4].destinations[0].address ^= 2;
    forged_switch.validate().unwrap();
    assert!(forged_switch.verify_source(&rom).is_err());
    let switch_bytes = encode_thumb_switch_tables(&forged_switch).unwrap();
    let mut switch_manifest = dataset.manifest.clone();
    switch_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == THUMB_SWITCH_TABLES_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&switch_bytes);
    let mut switch_files = dataset.files();
    switch_files.manifest = &switch_manifest;
    switch_files.thumb_switch_tables = Some((&forged_switch, &switch_bytes));
    assert!(
        source.validate_files(switch_files).is_err(),
        "coherent switch file/hash edits must be rejected"
    );
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for literal in data
        .literals
        .iter()
        .chain(&data.additional)
        .chain(&data.observed)
        .chain(&data.home_anchored)
        .chain(&data.home_observed)
    {
        let start = literal.source_offset as usize;
        for at in start..start + 4 {
            managed.bus.rom[at] = !rom[at];
        }
    }
    let before = owner.thumb_pc_literals_stats().unwrap();
    for literal in data
        .literals
        .iter()
        .chain(&data.additional)
        .chain(&data.observed)
        .chain(&data.home_anchored)
        .chain(&data.home_observed)
    {
        for at in literal.source_offset..literal.source_offset + 4 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.thumb_pc_literals_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 3804);
    assert_eq!(
        after.literals_mask,
        [u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, 0x3f]
    );
    assert_eq!(
        after.additional_source_bytes - before.additional_source_bytes,
        468
    );
    assert_eq!(after.additional_mask, [u64::MAX, (1u64 << 53) - 1]);
    assert_eq!(
        after.observed_source_bytes - before.observed_source_bytes,
        1772
    );
    assert_eq!(
        after.observed_mask,
        [
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            (1u64 << 59) - 1
        ]
    );
    assert_eq!(after.home_anchored_bytes - before.home_anchored_bytes, 100);
    assert_eq!(after.home_anchored_mask, (1u64 << 25) - 1);
    assert_eq!(after.home_observed_bytes - before.home_observed_bytes, 160);
    assert_eq!(after.home_observed_mask, (1u64 << 40) - 1);
    // Selected boundaries include previous owners, adjacent words and raw bytes.
    let prior_owned = |at: usize| {
        old.resources
            .iter()
            .flat_map(|entry| &entry.sources)
            .any(|span| {
                (span.offset as usize..span.offset as usize + span.length as usize).contains(&at)
            })
    };
    let boundary_indices = [0usize, 1, 10, 15, 20, 25, 30, 35, 39];
    let mut changed = BTreeSet::new();
    for i in boundary_indices {
        let offset = model::HOME_OBSERVED[i].0 as usize;
        for at in (offset - 3..offset).chain(offset + 4..offset + 8) {
            if data.byte_at(at as u32).is_none() {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for i in boundary_indices {
        let offset = model::HOME_OBSERVED[i].0 as usize;
        for edge in [offset, offset + 4] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    let x = at + n;
                    if data.byte_at(x as u32).is_some()
                        || switch_data.byte_at(x as u32).is_some()
                        || prior_owned(x)
                        || !changed.contains(&x)
                    {
                        rom[x]
                    } else {
                        rom[x] ^ 0x5a
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
    for literal in data
        .literals
        .iter()
        .chain(&data.additional)
        .chain(&data.observed)
        .chain(&data.home_anchored)
        .chain(&data.home_observed)
    {
        let start = literal.source_offset as usize;
        managed.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }

    let indices = [
        0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
        24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    ];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses: Vec<u32> = indices
        .iter()
        .map(|&i| 0x0800_0000 + model::HOME_OBSERVED[i].1)
        .collect();
    assert!(addresses
        .iter()
        .all(|a| !addresses.iter().any(|b| a != b && a.abs_diff(*b) == 2)));
    let blocks = addresses
        .iter()
        .map(|&addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("home-observed-thumb-literals-native.o");
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
        for i in indices {
            let (offset, reader, opcode) = model::HOME_OBSERVED[i];
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, reader);
            }
            for at in offset as usize..offset as usize + 4 {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner.thumb_pc_literals_stats().unwrap().home_observed_bytes;
            raw.bus.frame_ready = false;
            raw.step();
            managed.bus.frame_ready = false;
            if native {
                table.get((0x0800_0000 + reader) | 1).unwrap()(
                    &RT_API,
                    &mut managed as *mut Machine as *mut std::ffi::c_void,
                );
            } else {
                managed.step();
            }
            assert_eq!(
                owner.thumb_pc_literals_stats().unwrap().home_observed_bytes - before,
                4
            );
            let expected = u32::from_le_bytes(
                rom[offset as usize..offset as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(raw.cpu.regs[usize::from((opcode >> 8) & 7)], expected);
            assert_eq!(
                (raw.cpu.regs[15], managed.cpu.regs[15]),
                (0x0800_0000 + reader + 2, 0x0800_0000 + reader + 2)
            );
            managed.bus.rom[offset as usize..offset as usize + 4]
                .copy_from_slice(&rom[offset as usize..offset as usize + 4]);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "literal {offset:x}, native={native}"
            );
        }
    }

    // Both home switch tables retain the original guarded paths and out-of-range behavior.
    let new_specs = &switches::ADDITIONAL_TABLES[4..6];
    assert_eq!(new_specs.len(), 2);
    assert_eq!(
        switch_data.additional_tables[4..6]
            .iter()
            .map(|t| t.destinations.len())
            .sum::<usize>(),
        35
    );
    let switch_before = owner.thumb_switch_tables_stats().unwrap();
    for spec in new_specs.iter().copied() {
        for at in spec.start() as usize..spec.end() as usize {
            managed.bus.rom[at] = !rom[at];
        }
        for at in spec.start()..spec.end() {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let switch_after = owner.thumb_switch_tables_stats().unwrap();
    assert_eq!(
        switch_after.additional_source_bytes - switch_before.additional_source_bytes,
        140
    );
    assert_eq!(switch_after.additional_destinations_mask & 0x30, 0x30);
    let mut switch_changed = BTreeSet::new();
    for spec in new_specs.iter().copied() {
        for at in (spec.start() as usize - 3..spec.start() as usize)
            .chain(spec.end() as usize..spec.end() as usize + 4)
        {
            if data.byte_at(at as u32).is_none()
                && switch_data.byte_at(at as u32).is_none()
                && !prior_owned(at)
            {
                switch_changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for spec in new_specs.iter().copied() {
        for edge in [spec.start() as usize, spec.end() as usize] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    let x = at + n;
                    if switch_changed.contains(&x) {
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
                        "switch read16 {at:x}"
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]),
                        "switch read32 {at:x}"
                    );
                }
            }
        }
    }
    for spec in new_specs.iter().copied() {
        managed.bus.rom[spec.start() as usize..spec.end() as usize]
            .copy_from_slice(&rom[spec.start() as usize..spec.end() as usize]);
    }
    for at in switch_changed {
        managed.bus.rom[at] = rom[at];
    }

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut addresses = BTreeSet::new();
    for (which, spec) in new_specs.iter().copied().enumerate() {
        let start = if which == 0 {
            spec.guard
        } else {
            spec.code_start
        };
        addresses.extend((0x0800_0000 + start..0x0800_0000 + spec.mov).step_by(2));
    }
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("home-switch-destinations-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_switch_library, switch_table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool, spec: switches::AdditionalTableSpec| {
        machine.bus.frame_ready = false;
        let pc = machine.cpu.regs[15];
        if native && pc != 0x0800_0000 + spec.mov {
            switch_table
                .get(pc | 1)
                .unwrap_or_else(|| panic!("Missing generated switch reader {pc:08x}"))(
                &RT_API,
                machine as *mut Machine as *mut std::ffi::c_void,
            );
        } else {
            machine.step();
        }
    };
    for native in [false, true] {
        for (which, spec) in new_specs.iter().copied().enumerate() {
            for index in [0u32, spec.count as u32 - 1] {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(
                        machine,
                        if which == 0 {
                            spec.guard
                        } else {
                            spec.code_start
                        },
                    );
                    machine.cpu.regs[0] = index + u32::from(which == 1);
                }
                for at in spec.root as usize..spec.end() as usize {
                    managed.bus.rom[at] = !rom[at];
                }
                let before = owner
                    .thumb_switch_tables_stats()
                    .unwrap()
                    .additional_source_bytes;
                let root_before = owner.thumb_pc_literals_stats().unwrap().home_observed_bytes;
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
                    owner.thumb_pc_literals_stats().unwrap().home_observed_bytes - root_before,
                    4
                );
                managed.bus.rom[spec.root as usize..spec.end() as usize]
                    .copy_from_slice(&rom[spec.root as usize..spec.end() as usize]);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "switch={}, index={index}, native={native}",
                    spec.root
                );
            }
            // Out-of-range guarded selector follows the original branch without table reads.
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(
                    machine,
                    if which == 0 {
                        spec.guard
                    } else {
                        spec.code_start
                    },
                );
                machine.cpu.regs[0] = if which == 0 { spec.count as u32 } else { 0 };
            }
            let stop = if which == 0 { 0x0809_efbc } else { 0x0809_f3d2 };
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

            // Enter the original load sequence beyond the projection: no host guard is added.
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, spec.reader);
                machine.cpu.regs[0] = spec.count as u32;
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
        }
    }

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let offset = model::HOME_OBSERVED[0].0;
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
            .thumb_pc_literals_stats()
            .unwrap()
            .home_observed_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .thumb_pc_literals_stats()
                .unwrap()
                .home_observed_bytes
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
        "status":"PASS", "revision":[248,247], "home_observed_literals":40, "home_observed_source_bytes":160,
        "home_switch_tables":2, "home_switch_destinations":35, "home_switch_source_bytes":140,
        "prior_literals_preserved":911, "prior_switch_tables_preserved":27, "prior_other_resources_unchanged":120, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_copied_halfwords_preserved":true, "legacy_masks_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":80,
        "original_guarded_switch_cases":8, "outside_guard_cases":4, "outside_load_cases":4, "indirect_mov_interpreter_fallback":true,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
