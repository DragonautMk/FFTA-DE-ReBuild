//! EUR signed halfword physical projection and its original byte-indexed readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::copied_halfword_lookups as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn poison(machine: &mut Machine, rom: &[u8], enabled: bool) {
    let start = model::SIGNED_PROJECTION_OFFSET as usize;
    for at in start..start + model::SIGNED_PROJECTION_BYTES {
        machine.bus.rom[at] = if enabled { !rom[at] } else { rom[at] };
    }
}

#[test]
#[ignore = "exact EUR ROM and E269 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_signed_halfword_projection_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_SIGNED_HALFWORDS_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_FIRAGA_SIGNED_HALFWORDS_PREVIOUS").unwrap());
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
        (269, 268, 146)
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
        (270, 269, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        if prior.id != COPIED_HALFWORD_LOOKUPS_ID {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(previous.join(&prior.path)).unwrap(),
                fs::read(prepared.directory.join(&current.path)).unwrap()
            );
        } else {
            let old_data =
                parse_copied_halfword_lookups(&fs::read(previous.join(&prior.path)).unwrap())
                    .unwrap();
            let mut new_data = parse_copied_halfword_lookups(
                &fs::read(prepared.directory.join(&current.path)).unwrap(),
            )
            .unwrap();
            assert!(old_data.signed_byte_index_projection.is_none());
            let projection = new_data.signed_byte_index_projection.take().unwrap();
            assert_eq!(
                (projection.source_offset, projection.values.len()),
                (
                    model::SIGNED_PROJECTION_OFFSET,
                    model::SIGNED_PROJECTION_VALUES
                )
            );
            assert_eq!(new_data, old_data);
            let mut metadata = current.clone();
            assert_eq!(
                metadata.sources.pop().unwrap().length as usize,
                model::SIGNED_PROJECTION_BYTES
            );
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
        }
    }
    let data = &source.copied_halfword_lookups.as_ref().unwrap().decoded;
    let mut invalid = data.clone();
    invalid
        .signed_byte_index_projection
        .as_mut()
        .unwrap()
        .values
        .pop();
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.signed_byte_index_projection.as_mut().unwrap().values[18] ^= 1;
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_copied_halfword_lookups(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut manifest = dataset.manifest.clone();
    manifest
        .resources
        .iter_mut()
        .find(|r| r.id == COPIED_HALFWORD_LOOKUPS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &manifest;
    files.copied_halfword_lookups = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let start = model::SIGNED_PROJECTION_OFFSET as usize;
    let end = start + model::SIGNED_PROJECTION_BYTES;
    poison(&mut managed, &rom, true);
    for at in start..end {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
        }
    }
    let stats = owner.copied_halfword_lookups_stats().unwrap();
    assert_eq!(
        stats.signed_projection_bytes,
        (model::SIGNED_PROJECTION_BYTES * 3) as u64
    );
    assert_eq!(stats.signed_projection_regions_mask, 3);
    assert_eq!(stats.table_mask & 0x10, 0x10);
    let owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| (s.offset as usize..(s.offset + s.length) as usize).contains(&at))
    };
    let mut changed = BTreeSet::new();
    for edge in [start, end] {
        for at in edge.saturating_sub(3)..edge + 4 {
            if !owned(at) {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for edge in [start, end] {
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
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }
    poison(&mut managed, &rom, false);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let readers = [
        (0x0800_7ab0u32, 0x0800_7abc, model::SIGNED_PROJECTION_OFFSET),
        (
            0x0800_7ac4,
            0x0800_7ad2,
            model::SIGNED_PROJECTION_SECOND_VIEW,
        ),
    ];
    let blocks = readers
        .into_iter()
        .flat_map(|(entry, stop, _)| (entry..stop).step_by(2))
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-signed-halfwords-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let inputs = [
        0u32,
        1,
        17,
        18,
        23,
        24,
        25,
        26,
        32,
        63,
        64,
        127,
        128,
        191,
        192,
        254,
        255,
        0x100,
        0x112,
        0x1ff,
        u32::MAX,
    ];
    let mut cases = 0;
    for native in [false, true] {
        for (entry, stop, base) in readers {
            for input in inputs {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[0] = input;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[14] = 0x0800_0301;
                    machine.cpu.regs[15] = entry;
                    machine.cpu.cpsr = 0xbf;
                }
                poison(&mut managed, &rom, true);
                let before = owner
                    .copied_halfword_lookups_stats()
                    .unwrap()
                    .signed_projection_bytes;
                let root_before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
                for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                    let mut steps = 0;
                    while machine.cpu.regs[15] != stop {
                        assert!(steps < 16, "reader escaped at {:08x}", machine.cpu.regs[15]);
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
                let at = (base + 2 * (input & 255)) as usize;
                let expected =
                    i16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) as i32 as u32;
                assert_eq!(raw.cpu.regs[0], expected);
                assert_eq!(
                    owner
                        .copied_halfword_lookups_stats()
                        .unwrap()
                        .signed_projection_bytes
                        - before,
                    2
                );
                assert_eq!(
                    owner.thumb_pc_literals_stats().unwrap().source_bytes - root_before,
                    4
                );
                poison(&mut managed, &rom, false);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "reader={entry:08x}, input={input:x}, native={native}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 84);
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
        poison(&mut dma_managed, &rom, true);
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .copied_halfword_lookups_stats()
            .unwrap()
            .signed_projection_bytes;
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
                .signed_projection_bytes
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
        "status":"PASS", "revision":[270,269], "resources":146, "new_source_bytes":640,
        "prior_other_resources_unchanged":145, "prior_representation_preserved":true,
        "invalid_and_forged_source_rejected":true, "poisoned_mixed_boundaries_and_mirrors":true,
        "original_interpreter_generated_reader_cases":cases, "dma_cpuset_cpufastset_cases":3,
        "full_state_clock_equal":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
