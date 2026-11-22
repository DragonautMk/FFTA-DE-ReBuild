//! Selected EUR Firaga palette prefixes and their original copy consumer.
use super::*;
use gba_core::{capi::RT_API, rom_data::firaga_palette_rows as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn poison(machine: &mut Machine, rom: &[u8], enable: bool) {
    for offset in model::ROW_OFFSETS {
        for at in offset as usize..offset as usize + 8 {
            machine.bus.rom[at] = if enable { !rom[at] } else { rom[at] };
        }
    }
}
fn setup(machine: &mut Machine, pc: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
}
#[test]
#[ignore = "exact EUR ROM and E258 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_palette_rows_and_original_copy() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_PALETTE_ROWS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_PALETTE_ROWS_PREVIOUS").unwrap());
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
        (258, 257, 143)
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
        (259, 258, 144)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior, "prior resource {}", prior.id);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let data = &source.firaga_palette_rows.as_ref().unwrap().decoded;
    assert_eq!(
        data.rows
            .iter()
            .map(|r| (r.declared_rows, r.width))
            .collect::<Vec<_>>(),
        vec![(1, 3), (3, 3)]
    );
    for (offset, bytes, _) in data.source_spans().unwrap() {
        assert_eq!(bytes, rom[offset as usize..offset as usize + bytes.len()]);
    }
    assert_eq!(
        parse_firaga_palette_rows(
            &fs::read(prepared.directory.join(FIRAGA_PALETTE_ROWS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut forged = data.clone();
    forged.rows[0].colors[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_firaga_palette_rows(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut manifest = dataset.manifest.clone();
    manifest
        .resources
        .iter_mut()
        .find(|r| r.id == FIRAGA_PALETTE_ROWS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &manifest;
    files.firaga_palette_rows = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    raw.bus.iwram[0x5ee8..0x5f28].copy_from_slice(&rom[0xd968a8..0xd968e8]);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    poison(&mut managed, &rom, true);
    for offset in model::ROW_OFFSETS {
        for at in offset..offset + 8 {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]);
            }
        }
    }
    assert!(owner
        .firaga_palette_rows_stats()
        .unwrap()
        .source_bytes
        .iter()
        .all(|&n| n > 0));
    let owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| (s.offset as usize..s.offset as usize + s.length as usize).contains(&at))
    };
    let mut changed = BTreeSet::new();
    for edge in [0xae6db0usize, 0xae6db8, 0xae6dc0] {
        for at in edge - 3..edge + 3 {
            if !owned(at) {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    // The second header declares three rows, but only its first row is owned.
    assert!(changed.contains(&0xae6dc0));
    for edge in [0xae6db0usize, 0xae6db8, 0xae6dc0] {
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
    // Run the actual selected consumer and its IWRAM copy helper to return.
    let mut complete_copies = 0;
    for (offset, bank) in [(0xae6db8u32, 7u32), (0xae6db0, 6)] {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            setup(machine, 0x080d_80e0);
            machine.cpu.regs[0] = 0x0800_0000 + offset;
            machine.cpu.regs[1] = bank;
            machine.cpu.regs[2] = 1;
            machine.cpu.regs[3] = 0;
            machine.bus.write32(0x0300_7c00, 1);
        }
        poison(&mut managed, &rom, true);
        let before = owner.firaga_palette_rows_stats().unwrap().bus_reads;
        for machine in [&mut raw, &mut managed] {
            let mut steps = 0;
            while machine.cpu.regs[15] != 0x0800_0300 {
                assert!(
                    steps < 5000,
                    "palette copy escaped at {:08x}",
                    machine.cpu.regs[15]
                );
                machine.bus.frame_ready = false;
                machine.step();
                steps += 1;
            }
            let dst = (0x3a60 + bank * 32 + 2) as usize;
            assert_eq!(
                &machine.bus.iwram[dst..dst + 6],
                &rom[offset as usize + 2..offset as usize + 8]
            );
        }
        assert!(owner.firaga_palette_rows_stats().unwrap().bus_reads > before);
        poison(&mut managed, &rom, false);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "complete consumer {offset:x}"
        );
        complete_copies += 1;
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&raw.bus.iwram),
        bios: None,
    };
    let blocks = (0x0300_5ee8..0x0300_5f28)
        .step_by(2)
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-palette-copy-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut native_copies = 0;
    for offset in model::ROW_OFFSETS {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            setup(machine, 0x0300_5ee8);
            machine.cpu.regs[0] = 0x0201_0000;
            machine.cpu.regs[1] = 0x0800_0000 + offset + 2;
            machine.cpu.regs[2] = 6;
        }
        poison(&mut managed, &rom, true);
        let before = owner.firaga_palette_rows_stats().unwrap().bus_reads;
        for (machine, native) in [(&mut raw, false), (&mut managed, true)] {
            let mut steps = 0;
            while machine.cpu.regs[15] != 0x0800_0300 {
                assert!(steps < 3000);
                machine.bus.frame_ready = false;
                if native {
                    table.get(machine.cpu.regs[15] | 1).unwrap()(
                        &RT_API,
                        machine as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    machine.step();
                }
                steps += 1;
            }
            assert_eq!(
                &machine.bus.ewram[0x10000..0x10006],
                &rom[offset as usize + 2..offset as usize + 8]
            );
        }
        assert!(owner.firaga_palette_rows_stats().unwrap().bus_reads > before);
        poison(&mut managed, &rom, false);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "native helper {offset:x}"
        );
        native_copies += 1;
    }
    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let offset = model::ROW_OFFSETS[0];
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 8);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + offset, 0x0203_2400, 8]);
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
            .firaga_palette_rows_stats()
            .unwrap()
            .bus_reads;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .firaga_palette_rows_stats()
                .unwrap()
                .bus_reads
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
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","revision":[259,258],"resources":144,"source_bytes":16,"prior_resources_unchanged":143,
        "mixed_boundaries_fallback_and_mirrors":true,"original_complete_consumer_copies":complete_copies,"original_generated_copy_helper_cases":native_copies,
        "dma_cpuset_cpufastset_cases":3,"full_state_clock_equal":true,"coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true,"Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
