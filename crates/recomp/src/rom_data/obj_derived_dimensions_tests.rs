//! EUR shape/size lookup views derived from the existing OBJ dimensions file.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{obj_derived_dimensions as model, Resource, OBJ_DIMENSIONS_ID},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{path::Path, sync::Arc};

pub(super) fn assert_previous_obj_dimensions_preserved(
    previous: &Path,
    current_directory: &Path,
    prior: &Resource,
    current: &Resource,
) {
    assert_eq!(
        (
            &current.id,
            &current.path,
            &current.format,
            &current.sha256,
            &current.dependencies
        ),
        (
            &prior.id,
            &prior.path,
            &prior.format,
            &prior.sha256,
            &prior.dependencies
        )
    );
    assert_eq!(
        &current.sources[..prior.sources.len()],
        prior.sources.as_slice()
    );
    assert_eq!(current.sources.len(), prior.sources.len() + 2);
    assert_eq!(
        current.sources[prior.sources.len()].offset,
        model::TABLE_OFFSETS[0]
    );
    assert_eq!(
        current.sources[prior.sources.len() + 1].offset,
        model::TABLE_OFFSETS[1]
    );
    assert_eq!(
        fs::read(previous.join(&prior.path)).unwrap(),
        fs::read(current_directory.join(&current.path)).unwrap()
    );
}

fn setup(machine: &mut Machine, shape: u32, size: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[2] = (shape << 14) | (size << 30);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0801_fb2a;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E215 predecessor; Wine only, Linux/USA pending"]
fn european_obj_dimension_views_publication_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_OBJ_DERIVED_DIMENSIONS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_OBJ_DERIVED_DIMENSIONS_PREVIOUS").unwrap());
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
        (215, 214, 127)
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
        if prior.id == OBJ_DIMENSIONS_ID {
            assert_previous_obj_dimensions_preserved(
                &previous,
                &prepared.directory,
                prior,
                current,
            );
        } else if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
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
    let projections = model::projections(&source.dimensions);
    for (i, start) in model::TABLE_OFFSETS.into_iter().enumerate() {
        assert_eq!(projections[i], rom[start as usize..start as usize + 48]);
    }
    let mut forged = source.dimensions.clone();
    forged.entries.swap(0, 1);
    forged.validate().unwrap();
    let bytes = toml::to_string_pretty(&forged).unwrap().into_bytes();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == OBJ_DIMENSIONS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.dimensions = (&forged, &bytes);
    assert!(
        source.validate_files(files).is_err(),
        "coherent dimension file/hash edits must be rejected"
    );
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for start in model::TABLE_OFFSETS {
        for at in start as usize..start as usize + 48 {
            managed.bus.rom[at] = !rom[at];
        }
    }
    let before = owner.stats();
    for start in model::TABLE_OFFSETS {
        for at in start..start + 48 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.stats();
    assert_eq!(after.derived_source_bytes - before.derived_source_bytes, 96);
    assert_eq!(after.derived_table_mask, 3);
    assert_eq!(after.derived_rows_mask, [0x0fff, 0x0fff]);
    // The preceding bytes belong to the display-control owner; the following
    // bytes remain raw, including when the guest uses the reserved shape.
    for at in 0x14d2f8..0x14d2fc {
        managed.bus.rom[at] = !rom[at];
    }
    for at in 0x14d35c..0x14d360 {
        managed.bus.rom[at] = rom[at] ^ 0x5a;
    }
    for edge in [0x14d2fcusize, 0x14d32c, 0x14d35c] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| {
                let x = at + n;
                if (0x14d35c..0x14d360).contains(&x) {
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
    managed.bus.rom[0x14d2f8..0x14d360].copy_from_slice(&rom[0x14d2f8..0x14d360]);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = (0x0801_fb2a..0x0801_fb4c)
        .step_by(2)
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("obj-derived-dimensions-native.o");
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
        for shape in 0..4 {
            for size in 0..4 {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(machine, shape, size);
                }
                for start in model::TABLE_OFFSETS {
                    for at in start as usize..start as usize + 48 {
                        managed.bus.rom[at] = !rom[at];
                    }
                }
                let before = owner.stats().derived_source_bytes;
                let stop = 0x0801_fb4c;
                for _ in 0..32 {
                    if raw.cpu.regs[15] == stop {
                        break;
                    }
                    raw.bus.frame_ready = false;
                    raw.step();
                }
                for _ in 0..32 {
                    let pc = managed.cpu.regs[15];
                    if pc == stop {
                        break;
                    }
                    managed.bus.frame_ready = false;
                    if native {
                        table
                            .get(pc | 1)
                            .unwrap_or_else(|| panic!("Missing generated reader at {pc:08x}"))(
                            &RT_API,
                            &mut managed as *mut Machine as *mut std::ffi::c_void,
                        );
                    } else {
                        managed.step();
                    }
                }
                assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop));
                assert_eq!(
                    owner.stats().derived_source_bytes - before,
                    if shape == 3 { 2 } else { 4 }
                );
                for start in model::TABLE_OFFSETS {
                    managed.bus.rom[start as usize..start as usize + 48]
                        .copy_from_slice(&rom[start as usize..start as usize + 48]);
                }
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "shape {shape}, size {size}, native {native}"
                );
            }
        }
    }

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let start = model::TABLE_OFFSETS[0];
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + start);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + start,
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
        for at in start as usize..start as usize + 48 {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.stats().derived_source_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(transfer_owner.stats().derived_source_bytes > before);
        dma_managed.bus.rom[start as usize..start as usize + 48]
            .copy_from_slice(&rom[start as usize..start as usize + 48]);
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
        "status":"PASS", "revision":[257,256], "derived_tables":2, "entries_per_table":12, "source_bytes":96,
        "dimensions_file_unchanged":true, "prior_other_resources_unchanged":117, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_switch_tables_preserved":true, "prior_thumb_literals_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_prefix_cases":32,
        "reserved_shape_original_physical_reads_preserved":true,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
