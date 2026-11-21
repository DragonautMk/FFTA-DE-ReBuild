//! EUR indexed halfwords: publication, original reader and physical boundaries.
use super::*;
use gba_core::{capi::RT_API, rom_data::indexed_halfwords_39a5 as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::sync::Arc;

const READER_ENTRY: u32 = 0x080e_21de;
const READER_STOP: u32 = 0x080e_21e6;
const ROOT: usize = 0xe21f4;

fn setup(machine: &mut Machine, index: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[4] = index;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = READER_ENTRY;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E217 predecessor; Wine only, Linux/USA pending"]
fn european_indexed_halfwords_39a5_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_HALFWORDS_39A5_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_INDEXED_HALFWORDS_39A5_PREVIOUS").unwrap());
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
        (217, 216, 128)
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
        (221, 220, 130)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous, &prepared.directory, prior, current);
        } else {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(prepared.directory.join(&current.path)).unwrap(),
                fs::read(previous.join(&prior.path)).unwrap()
            );
        }
    }
    let data = &source.indexed_halfwords_39a5.as_ref().unwrap().decoded;
    let start = model::SOURCE_OFFSET as usize;
    let end = start + model::SOURCE_BYTES;
    assert_eq!(data.source_bytes().unwrap(), rom[start..end]);
    assert_eq!(
        parse_indexed_halfwords_39a5(
            &fs::read(prepared.directory.join(INDEXED_HALFWORDS_39A5_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.source_offset += 1;
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::IndexedHalfwords39a5>(invalid).is_err());
    let mut forged = data.clone();
    forged.values[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_indexed_halfwords_39a5(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == INDEXED_HALFWORDS_39A5_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.indexed_halfwords_39a5 = Some((&forged, &bytes));
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
    for at in start..end {
        managed.bus.rom[at] = !rom[at];
    }
    for at in ROOT..ROOT + 4 {
        managed.bus.rom[at] = !rom[at];
    }
    let before = owner.indexed_halfwords_39a5_stats().unwrap();
    for at in start..end {
        assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]);
    }
    let after = owner.indexed_halfwords_39a5_stats().unwrap();
    assert_eq!(
        after.source_bytes - before.source_bytes,
        model::SOURCE_BYTES as u64
    );
    assert_eq!(after.values_mask, 7);
    assert_eq!(
        managed.bus.read32(0x0800_0000 + ROOT as u32),
        u32::from_le_bytes(rom[ROOT..ROOT + 4].try_into().unwrap())
    );
    for at in (start - 3..start).chain(end..end + 4) {
        managed.bus.rom[at] = rom[at] ^ 0x5a;
    }
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| {
                let x = at + n;
                if (start - 3..start).contains(&x) || (end..end + 4).contains(&x) {
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
    managed.bus.rom[start - 3..end + 4].copy_from_slice(&rom[start - 3..end + 4]);
    managed.bus.rom[ROOT..ROOT + 4].copy_from_slice(&rom[ROOT..ROOT + 4]);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = (READER_ENTRY..READER_STOP)
        .step_by(2)
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("indexed-halfwords-39a5c8-native.o");
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
        for index in [0u32, 1, 2, 3, 255] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, index);
            }
            for at in start..end {
                managed.bus.rom[at] = !rom[at];
            }
            for at in ROOT..ROOT + 4 {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner.indexed_halfwords_39a5_stats().unwrap().source_bytes;
            for _ in 0..16 {
                if raw.cpu.regs[15] == READER_STOP {
                    break;
                }
                raw.bus.frame_ready = false;
                raw.step();
            }
            for _ in 0..16 {
                if managed.cpu.regs[15] == READER_STOP {
                    break;
                }
                managed.bus.frame_ready = false;
                if native {
                    table.get(managed.cpu.regs[15] | 1).unwrap()(
                        &RT_API,
                        &mut managed as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    managed.step();
                }
            }
            assert_eq!(
                (raw.cpu.regs[15], managed.cpu.regs[15]),
                (READER_STOP, READER_STOP)
            );
            assert_eq!(
                owner.indexed_halfwords_39a5_stats().unwrap().source_bytes - before,
                if index < 3 { 2 } else { 0 }
            );
            let at = start + index as usize * 2;
            assert_eq!(
                raw.cpu.regs[1],
                u32::from(u16::from_le_bytes([rom[at], rom[at + 1]]))
            );
            managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            managed.bus.rom[ROOT..ROOT + 4].copy_from_slice(&rom[ROOT..ROOT + 4]);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "index={index}, native={native}"
            );
        }
    }

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine
                    .bus
                    .write32(0x0400_00d4, 0x0800_0000 + model::SOURCE_OFFSET);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 32);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + model::SOURCE_OFFSET,
                    0x0203_2400,
                    if kind == 2 { 16 } else { 32 },
                ]);
                machine.cpu.regs[15] = 0x0200_0000;
                machine.cpu.cpsr = 0xbf;
                machine
                    .bus
                    .write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        for at in start..end {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .indexed_halfwords_39a5_stats()
            .unwrap()
            .source_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .indexed_halfwords_39a5_stats()
                .unwrap()
                .source_bytes
                > before
        );
        dma_managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
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
        "status":"PASS", "revision":[221,220], "values":3, "source_bytes":6,
        "prior_other_resources_unchanged":126, "prior_switch_tables_preserved":true, "prior_thumb_literals_preserved":true, "prior_literal_root_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":10,
        "outside_indices_unchanged":true, "dma_cpuset_cpufastset_state_and_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
