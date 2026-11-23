//! EUR Firaga scoped indexed fields and their original load instructions.
use super::*;
use gba_core::{capi::RT_API, rom_data::firaga_compact_cells as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn poison(machine: &mut Machine, rom: &[u8], enabled: bool) {
    for (start, len) in model::INDEXED_FIELD_SPANS {
        for at in start as usize..start as usize + len {
            machine.bus.rom[at] = if enabled { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E270 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_index2_fields_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_INDEX2_FIELDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_INDEX2_FIELDS_PREVIOUS").unwrap());
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
        (270, 269, 146)
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
        (271, 270, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        if prior.id != FIRAGA_COMPACT_CELLS_ID {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(previous.join(&prior.path)).unwrap(),
                fs::read(prepared.directory.join(&current.path)).unwrap()
            );
        } else {
            let old_data =
                parse_firaga_compact_cells(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_firaga_compact_cells(
                &fs::read(prepared.directory.join(&current.path)).unwrap(),
            )
            .unwrap();
            assert!(old_data.indexed_fields.is_none());
            assert!(new_data.indexed_fields.take().is_some());
            assert_eq!(new_data, old_data);
            let mut metadata = current.clone();
            metadata.sources.truncate(prior.sources.len());
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
        }
    }
    let data = &source.firaga_compact_cells.as_ref().unwrap().decoded;
    assert_eq!(data.source_spans().unwrap().len(), 9);
    let mut forged = data.clone();
    forged.indexed_fields.as_mut().unwrap().byte_cc96 ^= 1;
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_firaga_compact_cells(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut manifest = dataset.manifest.clone();
    manifest
        .resources
        .iter_mut()
        .find(|r| r.id == FIRAGA_COMPACT_CELLS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &manifest;
    files.firaga_compact_cells = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let before = owner.firaga_compact_cells_stats().unwrap();
    poison(&mut managed, &rom, true);
    for (start, len) in model::INDEXED_FIELD_SPANS {
        for at in start as usize..start as usize + len {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
            }
        }
    }
    let after = owner.firaga_compact_cells_stats().unwrap();
    assert_eq!(after.source_bytes, before.source_bytes);
    assert_eq!(after.indexed_source_bytes, [3, 6, 3, 3, 12]);
    let owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| (s.offset as usize..(s.offset + s.length) as usize).contains(&at))
    };
    let mut changed = BTreeSet::new();
    for (start, len) in model::INDEXED_FIELD_SPANS {
        for edge in [start as usize, start as usize + len] {
            for at in edge.saturating_sub(3)..edge + 4 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    for (start, len) in model::INDEXED_FIELD_SPANS {
        for edge in [start as usize, start as usize + len] {
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
                        u32::from(u16::from_le_bytes([expected(0), expected(1)]))
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)])
                    );
                }
            }
        }
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }
    poison(&mut managed, &rom, false);

    let readers = [
        (0x080e_b93eu32, 0x39cc94u32, 1u32, 2usize, 2usize),
        (0x080e_b8c0, 0x39cca4, 2, 2, 2),
        (0x080e_bc10, 0x39ccb8, 1, 0, 0),
        (0x080e_b962, 0x39ccd0, 1, 2, 3),
        (0x080e_b990, 0x39ccf4, 4, 0, 0),
    ];
    let roots = [
        (0x080e_b93au32, 0x0eb9ccusize),
        (0x080e_b8ba, 0x0eb9b0),
        (0x080e_bc0a, 0x0ebc30),
        (0x080e_b95c, 0x0eb9d4),
        (0x080e_b988, 0x0eb9d8),
    ];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses: BTreeSet<_> = readers
        .iter()
        .map(|x| x.0)
        .chain(roots.iter().map(|x| x.0))
        .collect();
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-index2-fields-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for (field, (pc, base, width, src, dst)) in readers.into_iter().enumerate() {
            for index in [1u32, 2, 3] {
                let at = base + index * width;
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[src] = 0x0800_0000 + at;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[15] = pc;
                    machine.cpu.cpsr = 0xbf;
                }
                poison(&mut managed, &rom, true);
                let before = owner
                    .firaga_compact_cells_stats()
                    .unwrap()
                    .indexed_source_bytes[field];
                for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                    let mut steps = 0;
                    while machine.cpu.regs[15] != pc + 2 {
                        assert!(steps < 8, "reader escaped at {:08x}", machine.cpu.regs[15]);
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
                let expected = (0..width).fold(0u32, |v, i| {
                    v | (u32::from(rom[(at + i) as usize]) << (8 * i))
                });
                assert_eq!(raw.cpu.regs[dst], expected);
                assert_eq!(
                    owner
                        .firaga_compact_cells_stats()
                        .unwrap()
                        .indexed_source_bytes[field]
                        - before,
                    if index == 2 { u64::from(width) } else { 0 }
                );
                poison(&mut managed, &rom, false);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "pc={pc:08x}, index={index}, native={native}"
                );
                cases += 1;
            }
        }
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
            let before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
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
            }
            assert_eq!(
                owner.thumb_pc_literals_stats().unwrap().source_bytes - before,
                4
            );
            managed.bus.rom[literal..literal + 4].copy_from_slice(&rom[literal..literal + 4]);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "root pc={pc:08x}, native={native}"
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 40);
    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let start = 0x39ccfcusize;
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
        poison(&mut dma_managed, &rom, true);
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .firaga_compact_cells_stats()
            .unwrap()
            .indexed_source_bytes[4];
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .firaga_compact_cells_stats()
                .unwrap()
                .indexed_source_bytes[4]
                > before
        );
        poison(&mut dma_managed, &rom, false);
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
        "status":"PASS", "revision":[271,270], "resources":146, "new_source_bytes":9,
        "prior_other_resources_unchanged":145, "prior_representation_preserved":true,
        "forged_source_rejected":true, "poisoned_mixed_boundaries_and_mirrors":true,
        "original_interpreter_generated_reader_and_root_cases":cases, "dma_cpuset_cpufastset_cases":3,
        "full_state_clock_equal":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
