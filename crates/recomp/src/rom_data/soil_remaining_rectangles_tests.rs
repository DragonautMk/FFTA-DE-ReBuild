//! EUR Soil rectangles, alternative literal, original readers and transfers.
use super::*;
use gba_core::{capi::RT_API, rom_data::soil_remaining_rectangles as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn source_spans() -> Vec<(usize, usize)> {
    let mut spans = vec![(model::ALT_LITERAL as usize, 4)];
    spans.extend(
        model::RECTS
            .into_iter()
            .map(|(at, h, w)| (at as usize, 2 + 2 * usize::from(h) * usize::from(w))),
    );
    spans
}
fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, len) in source_spans() {
        for at in start..start + len {
            machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E247 predecessor; Wine only, Linux/USA pending"]
fn european_soil_remaining_rectangles_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_REMAINING_RECTANGLES_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_SOIL_REMAINING_RECTANGLES_PREVIOUS").unwrap());
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
        (247, 246, 135)
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
        .find(|entry| entry.id == SOIL_REMAINING_RECTANGLES_ID)
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
        vec![(model::ALT_LITERAL, 4), (0xae7810, 100), (0xae7874, 8)]
    );
    let data = &source.soil_remaining_rectangles.as_ref().unwrap().decoded;
    assert_eq!(
        data.rectangles
            .iter()
            .map(|r| (r.height, r.width))
            .collect::<Vec<_>>(),
        vec![(7, 7), (1, 3)]
    );
    for (offset, bytes, _) in data.source_spans().unwrap() {
        let at = offset as usize;
        assert_eq!(bytes, &rom[at..at + bytes.len()]);
    }
    assert_eq!(
        parse_soil_remaining_rectangles(
            &fs::read(prepared.directory.join(SOIL_REMAINING_RECTANGLES_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.rectangles[0].width += 1;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.rectangles[0].cells[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_soil_remaining_rectangles(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone();
    altered
        .resources
        .iter_mut()
        .find(|e| e.id == SOIL_REMAINING_RECTANGLES_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.soil_remaining_rectangles = Some((&forged, &forged_bytes));
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
    for (start, len) in source_spans() {
        for at in start..start + len {
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
    for (start, len) in source_spans() {
        for edge in [start, start + len] {
            for at in edge.saturating_sub(3)..edge + 3 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    assert!(
        changed.contains(&0xae787c),
        "unselected neighbor remains ROM-backed"
    );
    for (start, len) in source_spans() {
        for edge in [start, start + len] {
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

    let readers = [0x080d_811c, 0x080d_8136, 0x080d_8148];
    let roots = [
        (0x080e_617e, 0x0e6184usize),
        (0x080e_5fe8, 0x0e5ff0),
        (0x080e_60c8, model::ALT_LITERAL as usize),
    ];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut addresses = BTreeSet::new();
    for pc in readers {
        addresses.insert(pc);
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
    let object = root.join("soil-remaining-rectangles-native.o");
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
        for pc in readers {
            for (offset, _, _) in model::RECTS {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[7] = 0x0800_0000 + offset;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[15] = pc;
                    machine.cpu.cpsr = 0xbf;
                }
                set_backing(&mut managed, &rom, true);
                for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                    machine.bus.frame_ready = false;
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
                set_backing(&mut managed, &rom, false);
                assert!(
                    raw.save_state().unwrap() == managed.save_state().unwrap(),
                    "reader pc={pc:08x} offset={offset:06x} native={native}"
                );
                completed += 1;
            }
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
            set_backing(&mut managed, &rom, true);
            for at in literal..literal + 4 {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner
                .soil_remaining_rectangles_stats()
                .unwrap()
                .literal_bytes;
            for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                machine.bus.frame_ready = false;
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
            if literal == model::ALT_LITERAL as usize {
                assert!(
                    owner
                        .soil_remaining_rectangles_stats()
                        .unwrap()
                        .literal_bytes
                        > before
                );
            }
            managed.bus.rom[literal..literal + 4].copy_from_slice(&rom[literal..literal + 4]);
            set_backing(&mut managed, &rom, false);
            assert!(
                raw.save_state().unwrap() == managed.save_state().unwrap(),
                "root pc={pc:08x} native={native}"
            );
            completed += 1;
        }
    }
    assert_eq!(completed, 18);
    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    let mut transfers = 0;
    for (group, (offset, h, w)) in model::RECTS.into_iter().enumerate() {
        for kind in 0..2 {
            let len = 2 + 2 * usize::from(h) * usize::from(w);
            let count = (len / 2) as u32;
            for machine in [&mut transfer_raw, &mut transfer_managed] {
                machine.load_state(&transfer_initial).unwrap();
                if kind == 0 {
                    machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                    machine.bus.write32(0x0400_00d8, 0x0203_2000);
                    machine.bus.write16(0x0400_00dc, count as u16);
                } else {
                    machine.cpu.regs[..3].copy_from_slice(&[
                        0x0a00_0000 + offset,
                        0x0203_2400,
                        count,
                    ]);
                    machine.cpu.regs[15] = 0x0200_0000;
                    machine.cpu.cpsr = 0xbf;
                    machine.bus.write16(0x0200_0000, 0xdf0b);
                }
            }
            set_backing(&mut transfer_managed, &rom, true);
            let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
            let before = transfer_owner
                .soil_remaining_rectangles_stats()
                .unwrap()
                .rectangle_bytes[group];
            for machine in [&mut transfer_raw, &mut transfer_managed] {
                if kind == 0 {
                    machine.bus.write16(0x0400_00de, 0x8000);
                } else {
                    machine.step();
                }
            }
            assert!(
                transfer_owner
                    .soil_remaining_rectangles_stats()
                    .unwrap()
                    .rectangle_bytes[group]
                    > before
            );
            set_backing(&mut transfer_managed, &rom, false);
            assert!(
                transfer_raw.save_state().unwrap() == transfer_managed.save_state().unwrap(),
                "group={group}, transfer={kind}"
            );
            transfers += 1;
        }
    }
    assert_eq!(transfers, 4);
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","revision":[248,247],"resources":136,"new_source_bytes":112,"rectangle_source_bytes":108,"alternate_literal_bytes":4,
        "previous_resources_unchanged":135,"existing_roots_preserved":true,"unselected_neighbors_fall_back":true,
        "poisoned_mirrors_and_mixed_boundaries":true,"original_interpreter_generated_cases":completed,"dma_cpuset_cases":transfers,
        "full_state_clock_equal":true,"coherent_edit_rejected":true,"missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
