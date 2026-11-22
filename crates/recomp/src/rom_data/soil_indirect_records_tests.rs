//! EUR indirect Soil records, boundaries and original Thumb readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::soil_indirect_records as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for start in model::OFFSETS {
        for at in start as usize..start as usize + model::RECORD_BYTES {
            machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E243 predecessor; Wine only, Linux/USA pending"]
fn european_soil_indirect_records_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_INDIRECT_RECORDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_INDIRECT_RECORDS_PREVIOUS").unwrap());
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
        (243, 242, 131)
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
        if prior.id == OBJ_STATIC_LISTS_ID {
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(
                &previous, &prepared.directory, prior, current);
            continue;
        }
        if prior.id == OBSERVED_LZ_ID {
            super::soil_evidence_lz_tests::assert_previous_observed_lz_preserved(
                &previous, &prepared.directory, prior, current);
            continue;
        }
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let resource = source
        .manifest
        .resources
        .iter()
        .find(|entry| entry.id == SOIL_INDIRECT_RECORDS_ID)
        .unwrap();
    assert_eq!(
        resource.dependencies,
        vec![THUMB_PC_LITERALS_ID.to_string()]
    );
    assert_eq!(
        resource
            .sources
            .iter()
            .map(|s| (s.offset, s.length))
            .collect::<Vec<_>>(),
        model::OFFSETS.iter().map(|&at| (at, 8)).collect::<Vec<_>>()
    );
    let data = &source.soil_indirect_records.as_ref().unwrap().decoded;
    for (offset, bytes, _) in data.source_spans().unwrap() {
        let at = offset as usize;
        assert_eq!(bytes, &rom[at..at + bytes.len()]);
    }
    assert_eq!(
        parse_soil_indirect_records(
            &fs::read(prepared.directory.join(SOIL_INDIRECT_RECORDS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.records[0].offset += 1;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.records[0].values[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_soil_indirect_records(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone();
    altered
        .resources
        .iter_mut()
        .find(|e| e.id == SOIL_INDIRECT_RECORDS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.soil_indirect_records = Some((&forged, &forged_bytes));
    assert!(
        source.validate_files(files).is_err(),
        "coherent modified file and hash must be rejected"
    );
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    set_backing(&mut managed, &rom, true);
    for start in model::OFFSETS {
        for at in start as usize..start as usize + model::RECORD_BYTES {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
            }
        }
    }
    let owned = |at: usize| {
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
    for start in model::OFFSETS {
        for edge in [start as usize, start as usize + model::RECORD_BYTES] {
            for at in edge.saturating_sub(3)..edge + 3 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    for start in model::OFFSETS {
        for edge in [start as usize, start as usize + model::RECORD_BYTES] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |i: usize| {
                    if changed.contains(&(at + i)) {
                        rom[at + i] ^ 0x5a
                    } else {
                        rom[at + i]
                    }
                };
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!(
                    (RT_API.read16)(ptr, 0x0a00_0000 + at as u32),
                    u32::from(u16::from_le_bytes([expected(0), expected(1)]))
                );
                assert_eq!(
                    (RT_API.read32)(ptr, 0x0800_0000 + at as u32),
                    u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)])
                );
            }
        }
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }
    set_backing(&mut managed, &rom, false);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let ranges = [
        (0x080e_3682, 0x080e_368c),
        (0x080e_36a0, 0x080e_36ac),
        (0x080e_36b0, 0x080e_36b4),
    ];
    let roots = [
        (0x080e_4b10, 0x0e4b84),
        (0x080e_4b46, 0x0e4b90),
        (0x080e_4c6e, 0x0e4c88),
        (0x080f_0a02, 0x0f0a24),
        (0x080f_05d2, 0x0f0600),
    ];
    let mut addresses = BTreeSet::new();
    for (start, end) in ranges {
        addresses.extend((start..end).step_by(2));
    }
    for (pc, _) in roots {
        addresses.insert(pc);
    }
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("soil-indirect-records-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut completed = 0;
    for native in [false, true] {
        for mode in 0..3 {
            for &record in &model::OFFSETS {
                for lane in 0..2u32 {
                    let (pc, stop) = ranges[mode];
                    for machine in [&mut raw, &mut managed] {
                        machine.load_state(&initial).unwrap();
                        machine.bus.ffta_sprites.enabled = false;
                        machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                        machine.cpu.regs[5] = 0x0201_8000;
                        machine.cpu.regs[6] = lane;
                        machine.cpu.regs[13] = 0x0300_7c00;
                        if mode == 0 {
                            machine
                                .bus
                                .write32(0x0201_800c, 0x0800_0000 + record - lane * 8);
                            machine.cpu.regs[2] = lane;
                        }
                        if mode == 1 {
                            machine.bus.write32(0x0201_800c, 0x0800_0000 + record);
                            machine.cpu.regs[8] = 0;
                        }
                        if mode == 2 {
                            machine.cpu.regs[1] = 0x0800_0000 + record;
                        }
                        machine.cpu.regs[15] = pc;
                        machine.cpu.cpsr = 0xbf;
                    }
                    set_backing(&mut managed, &rom, true);
                    for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                        let mut steps = 0;
                        while machine.cpu.regs[15] != stop {
                            assert!(steps < 24, "reader escaped at {:08x}", machine.cpu.regs[15]);
                            machine.bus.frame_ready = false;
                            if use_native {
                                table.get(machine.cpu.regs[15] | 1).unwrap()(
                                    &RT_API,
                                    machine as *mut Machine as *mut std::ffi::c_void,
                                );
                            } else {
                                machine.step();
                            }
                            steps += 1;
                        }
                    }
                    set_backing(&mut managed, &rom, false);
                    assert!(
                        raw.save_state().unwrap() == managed.save_state().unwrap(),
                        "reader mode={mode} record={record:06x} lane={lane} native={native}"
                    );
                    completed += 1;
                }
            }
        }
    }
    let fallback = 0x39e22cusize;
    assert!((fallback..fallback + 8).all(|at| !owned(at)));
    for native in [false, true] {
        for mode in 0..3 {
            let (pc, stop) = ranges[mode];
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                machine.bus.ffta_sprites.enabled = false;
                machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                machine.cpu.regs[5] = 0x0201_8000;
                machine.cpu.regs[6] = 0;
                machine.cpu.regs[13] = 0x0300_7c00;
                if mode == 0 {
                    machine.bus.write32(0x0201_800c, 0x0839_e21c);
                    machine.cpu.regs[2] = 2;
                }
                if mode == 1 {
                    machine.bus.write32(0x0201_800c, 0x0839_e22c);
                    machine.cpu.regs[8] = 0;
                }
                if mode == 2 {
                    machine.cpu.regs[1] = 0x0839_e22c;
                }
                machine.cpu.regs[15] = pc;
                machine.cpu.cpsr = 0xbf;
                for at in fallback..fallback + 8 {
                    machine.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
            set_backing(&mut managed, &rom, true);
            for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                let mut steps = 0;
                while machine.cpu.regs[15] != stop {
                    assert!(steps < 24);
                    machine.bus.frame_ready = false;
                    if use_native {
                        table.get(machine.cpu.regs[15] | 1).unwrap()(
                            &RT_API,
                            machine as *mut Machine as *mut std::ffi::c_void,
                        );
                    } else {
                        machine.step();
                    }
                    steps += 1;
                }
            }
            let bytes = |at| [rom[at] ^ 0x5a, rom[at + 1] ^ 0x5a];
            if mode == 0 {
                assert_eq!(
                    raw.cpu.regs[0],
                    u32::from(u16::from_le_bytes(bytes(fallback)))
                );
            }
            if mode == 1 {
                assert_eq!(
                    raw.cpu.regs[2],
                    u32::from(u16::from_le_bytes(bytes(fallback + 4)))
                );
            }
            if mode == 2 {
                assert_eq!(raw.cpu.regs[0], u32::from(rom[fallback + 2] ^ 0x5a));
            }
            raw.bus.rom[fallback..fallback + 8].copy_from_slice(&rom[fallback..fallback + 8]);
            managed.bus.rom[fallback..fallback + 8].copy_from_slice(&rom[fallback..fallback + 8]);
            set_backing(&mut managed, &rom, false);
            assert!(
                raw.save_state().unwrap() == managed.save_state().unwrap(),
                "fallback mode={mode} native={native}"
            );
            completed += 1;
        }
    }
    for native in [false, true] {
        for (pc, literal) in roots {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                machine.bus.ffta_sprites.enabled = false;
                machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                machine.cpu.regs[13] = 0x0300_7c00;
                machine.cpu.regs[15] = pc;
                machine.cpu.cpsr = 0xbf;
            }
            for at in literal..literal + 4 {
                managed.bus.rom[at] = !rom[at];
            }
            for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                if use_native {
                    table.get(pc | 1).unwrap()(
                        &RT_API,
                        machine as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    machine.step();
                }
                assert_eq!(machine.cpu.regs[15], pc + 2);
            }
            managed.bus.rom[literal..literal + 4].copy_from_slice(&rom[literal..literal + 4]);
            assert!(
                raw.save_state().unwrap() == managed.save_state().unwrap(),
                "root pc={pc:08x} native={native}"
            );
            completed += 1;
        }
    }
    assert_eq!(completed, 76);
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[257,256], "resources":142, "new_source_bytes":40, "records":5,
        "previous_resources_unchanged":131, "roots_remain_in_existing_owner":true, "unselected_indices_fall_back":true,
        "poisoned_mirrors_and_mixed_boundaries":true, "original_interpreter_generated_cases":completed,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
