//! EUR copied halfword tables, original consumers and managed ownership.
use super::*;
use gba_core::{capi::RT_API, rom_data::copied_halfword_lookups as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, path::Path, sync::Arc};

pub(super) fn assert_previous_copied_halfwords_preserved(
    previous: &Path,
    current_directory: &Path,
    prior: &gba_core::rom_data::Resource,
    current: &gba_core::rom_data::Resource,
) {
    assert_eq!((&current.id, &current.path, &current.format), (&prior.id, &prior.path, &prior.format));
    let mut dependencies = prior.dependencies.clone();
    if !dependencies.iter().any(|id| id == THUMB_PC_LITERALS_ID) { dependencies.push(THUMB_PC_LITERALS_ID.into()); }
    assert_eq!(current.dependencies, dependencies);
    assert_eq!(&current.sources[..prior.sources.len()], prior.sources.as_slice());
    let old: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let new: serde_json::Value = serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    let old_tables = old["tables"].as_array().unwrap();
    let new_tables = new["tables"].as_array().unwrap();
    assert_eq!(old_tables.as_slice(), &new_tables[..old_tables.len()]);
}

fn setup(machine: &mut Machine, start: u32, selector: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = selector;
    machine.cpu.regs[13] = 0x0300_7a00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + start;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E216 predecessor; Wine only, Linux/USA pending"]
fn european_copied_halfword_lookups_and_original_consumers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COPIED_HALFWORD_LOOKUPS_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_COPIED_HALFWORD_LOOKUPS_PREVIOUS").unwrap());
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
        (216, 215, 127)
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
        if prior.id == THUMB_PC_LITERALS_ID {
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
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
        } else if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(
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
    let data = &source.copied_halfword_lookups.as_ref().unwrap().decoded;
    let spans: Vec<_> = data.source_spans().unwrap().into_iter()
        .filter(|(offset, _, _)| *offset != model::TABLE_OFFSETS[2] && *offset != model::SOIL_EVIDENCE_OFFSET).collect();
    assert_eq!(spans.len(), 4);
    assert_eq!(
        spans.iter().map(|(_, bytes, _)| bytes.len()).sum::<usize>(),
        24
    );
    for (offset, bytes, _) in &spans {
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
    }
    assert_eq!(
        parse_copied_halfword_lookups(
            &fs::read(prepared.directory.join(COPIED_HALFWORD_LOOKUPS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.tables.swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["tables"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::CopiedHalfwordLookups>(invalid).is_err());
    let mut forged = data.clone();
    forged.tables[0].values[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_copied_halfword_lookups(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == COPIED_HALFWORD_LOOKUPS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.copied_halfword_lookups = Some((&forged, &bytes));
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
    let poison = |machine: &mut Machine| {
        for (offset, bytes, _) in &spans {
            for at in *offset as usize..*offset as usize + bytes.len() {
                machine.bus.rom[at] = !rom[at];
            }
        }
        for at in 0x3630..0x3634 {
            machine.bus.rom[at] = !rom[at];
        }
    };
    let restore = |machine: &mut Machine| {
        for (offset, bytes, _) in &spans {
            let start = *offset as usize;
            machine.bus.rom[start..start + bytes.len()]
                .copy_from_slice(&rom[start..start + bytes.len()]);
        }
        machine.bus.rom[0x3630..0x3634].copy_from_slice(&rom[0x3630..0x3634]);
    };
    poison(&mut managed);
    let before = owner.copied_halfword_lookups_stats().unwrap();
    for (offset, bytes, _) in &spans {
        for at in *offset..*offset + bytes.len() as u32 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.copied_halfword_lookups_stats().unwrap();
    assert_eq!(after.table_bytes - before.table_bytes, 16);
    assert_eq!(after.root_bytes - before.root_bytes, 8);
    assert_eq!((after.table_mask, after.root_mask), (3, 3));
    let mut changed = BTreeSet::new();
    for &(offset, ref bytes, _) in &spans {
        let start = offset as usize;
        for at in (start - 3..start).chain(start + bytes.len()..start + bytes.len() + 4) {
            if data.byte_at(at as u32).is_none() && !(0x3630..0x3634).contains(&at) {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for &(offset, ref bytes, _) in &spans {
        for edge in [offset as usize, offset as usize + bytes.len()] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    let x = at + n;
                    if data.byte_at(x as u32).is_some()
                        || (0x3630..0x3634).contains(&x)
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
    restore(&mut managed);
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut addresses = BTreeSet::from([0x0800_3616, 0x0800_363e, 0x0800_3666]);
    addresses.extend((0x0814_7e0c..0x0814_7e1a).step_by(2));
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("copied-halfword-native.o");
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
        for (i, reader) in [0x3616u32, 0x363e, 0x3666].into_iter().enumerate() {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, reader, 0);
            }
            poison(&mut managed);
            let before = owner.copied_halfword_lookups_stats().unwrap().root_bytes;
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
                owner.copied_halfword_lookups_stats().unwrap().root_bytes - before,
                if i == 0 { 0 } else { 4 }
            );
            restore(&mut managed);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "root reader {reader:x}, native={native}"
            );
        }
        for (i, &start) in model::TABLE_OFFSETS.iter().take(2).enumerate() {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, 0x147e0c, 0);
                machine.cpu.regs[1] = u32::MAX;
                machine.cpu.regs[2] = 7;
                machine.cpu.regs[3] = 0x0800_0000 + start;
                machine.cpu.regs[4] = 0x0201_0000;
            }
            poison(&mut managed);
            let before = owner.copied_halfword_lookups_stats().unwrap().table_bytes;
            let stop = 0x0814_7e1a;
            for _ in 0..128 {
                if raw.cpu.regs[15] == stop {
                    break;
                }
                raw.bus.frame_ready = false;
                raw.step();
            }
            for _ in 0..128 {
                let pc = managed.cpu.regs[15];
                if pc == stop {
                    break;
                }
                managed.bus.frame_ready = false;
                if native {
                    table.get(pc | 1).unwrap()(
                        &RT_API,
                        &mut managed as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    managed.step();
                }
            }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop));
            assert_eq!(
                owner.copied_halfword_lookups_stats().unwrap().table_bytes - before,
                8
            );
            restore(&mut managed);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "copy {i}, native={native}"
            );
        }
    }
    // Execute the full original functions; selectors outside 0..4 read the
    // original stack, not a new ROM or guest-domain bound.
    for (i, start) in [0x360cu32, 0x3634, 0x365c].into_iter().enumerate() {
        for selector in [0u32, 1, 2, 3, 4, 255] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, start, selector);
            }
            poison(&mut managed);
            let before = owner.copied_halfword_lookups_stats().unwrap().table_bytes;
            for machine in [&mut raw, &mut managed] {
                for _ in 0..512 {
                    if machine.cpu.regs[15] == 0x0800_0300 {
                        break;
                    }
                    machine.bus.frame_ready = false;
                    machine.step();
                }
                assert_eq!(
                    machine.cpu.regs[15], 0x0800_0300,
                    "reader {start:x}, selector {selector}"
                );
            }
            assert_eq!(
                owner.copied_halfword_lookups_stats().unwrap().table_bytes - before,
                8
            );
            if selector < 4 {
                assert_eq!(
                    raw.cpu.regs[0],
                    u32::from(data.tables[usize::from(i != 0)].values[selector as usize])
                );
            }
            restore(&mut managed);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "reader {start:x}, selector {selector}"
            );
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
        poison(&mut dma_managed);
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .copied_halfword_lookups_stats()
            .unwrap()
            .table_bytes;
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
                .table_bytes
                > before
        );
        restore(&mut dma_managed);
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
        "status":"PASS", "revision":[248,247], "tables":2, "entries_per_table":4, "source_bytes":24,
        "prior_other_resources_unchanged":117, "previous_copied_halfwords_preserved":true, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_switch_tables_preserved":true, "prior_thumb_literals_preserved":true, "prior_literal_root_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_prefix_cases":10,
        "original_full_interpreter_cases":18, "outside_indices_preserve_original_stack":true,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
