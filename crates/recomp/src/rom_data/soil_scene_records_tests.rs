//! EUR Soil Evidence indexed scene records and their original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::soil_scene_records as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    let start = model::RANGE_OFFSET as usize;
    let end = model::SEQUENCE_OFFSET as usize + model::SEQUENCE_COUNT * model::RECORD_BYTES;
    for at in start..end {
        machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
    }
}

#[test]
#[ignore = "exact EUR ROM and E242 predecessor; Wine only, Linux/USA pending"]
fn european_soil_scene_records_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_SCENE_RECORDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_SCENE_RECORDS_PREVIOUS").unwrap());
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
        (242, 241, 130)
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
        .find(|entry| entry.id == SOIL_SCENE_RECORDS_ID)
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
        vec![(model::RANGE_OFFSET, 20), (model::SEQUENCE_OFFSET, 44)]
    );
    let data = &source.soil_scene_records.as_ref().unwrap().decoded;
    assert_eq!((data.ranges.len(), data.sequence.len()), (5, 11));
    for (offset, bytes, _) in data.source_spans().unwrap() {
        let at = offset as usize;
        assert_eq!(bytes, &rom[at..at + bytes.len()]);
    }
    assert_eq!(
        parse_soil_scene_records(
            &fs::read(prepared.directory.join(SOIL_SCENE_RECORDS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.sequence[0].range_index = 5;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.sequence[0].field1 ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_soil_scene_records(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone();
    altered
        .resources
        .iter_mut()
        .find(|e| e.id == SOIL_SCENE_RECORDS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.soil_scene_records = Some((&forged, &forged_bytes));
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
    let start = model::RANGE_OFFSET as usize;
    let end = model::SEQUENCE_OFFSET as usize + model::SEQUENCE_COUNT * model::RECORD_BYTES;
    for at in start..end {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
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
    for edge in [start, model::SEQUENCE_OFFSET as usize, end] {
        for at in edge.saturating_sub(3)..edge + 3 {
            if !owned(at) {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for edge in [start, model::SEQUENCE_OFFSET as usize, end] {
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
    let mut addresses = BTreeSet::new();
    for (start, end) in [
        (0x080f_0574, 0x080f_057e),
        (0x080f_063a, 0x080f_0646),
        (0x080f_0a6e, 0x080f_0a7a),
    ] {
        addresses.extend((start..end).step_by(2));
    }
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("soil-scene-records-native.o");
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
            for selector in [0u32, 5, 10, 11, 255] {
                let (pc, stop, literal, base) = [
                    (0x080f_0574, 0x080f_057e, 0x0f05f0, model::SEQUENCE_OFFSET),
                    (0x080f_063a, 0x080f_0646, 0x0f06ac, model::RANGE_OFFSET),
                    (0x080f_0a6e, 0x080f_0a7a, 0x0f0aa0, model::SEQUENCE_OFFSET),
                ][mode];
                let at = (base + selector * 4) as usize;
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[7] = 0x0201_8000;
                    machine.cpu.regs[5] = 0x0201_8000;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    if mode == 0 {
                        machine.bus.write8(0x0201_8002, selector as u8);
                    }
                    if mode == 1 {
                        machine.bus.write8(0x0201_8001, selector as u8);
                    }
                    if mode == 2 {
                        machine.bus.write16(0x0201_8002, selector as u16);
                    }
                    machine.cpu.regs[15] = pc;
                    machine.cpu.cpsr = 0xbf;
                    if selector == 255 {
                        assert!((at..at + 4).all(|off| !owned(off)));
                        for off in at..at + 4 {
                            machine.bus.rom[off] = rom[off] ^ 0x5a;
                        }
                    }
                }
                set_backing(&mut managed, &rom, true);
                for off in literal..literal + 4 {
                    managed.bus.rom[off] = !rom[off];
                }
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
                if selector == 255 {
                    if mode == 0 {
                        assert_eq!(raw.cpu.regs[0], u32::from(rom[at] ^ 0x5a));
                    }
                    if mode == 1 {
                        assert_eq!(raw.cpu.regs[2], u32::from(rom[at] ^ 0x5a));
                    }
                    if mode == 2 {
                        assert_eq!(raw.cpu.regs[0], u32::from(rom[at + 3] ^ 0x5a));
                    }
                    raw.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
                    managed.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
                }
                managed.bus.rom[literal..literal + 4].copy_from_slice(&rom[literal..literal + 4]);
                set_backing(&mut managed, &rom, false);
                assert!(
                    raw.save_state().unwrap() == managed.save_state().unwrap(),
                    "reader mode={mode} selector={selector} native={native}"
                );
                completed += 1;
            }
        }
    }
    assert_eq!(completed, 30);
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[257,256], "resources":142, "new_source_bytes":64, "range_records":5, "sequence_records":11,
        "previous_resources_unchanged":130, "roots_remain_in_existing_owner":true, "unselected_indices_fall_back":true,
        "poisoned_mirrors_and_mixed_boundaries":true, "original_interpreter_generated_cases":completed,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
