//! EUR script lookup: adjacent owners and both original reader prefixes.
use super::*;
use gba_core::{capi::RT_API, rom_data::script_byte_lookup_3ad as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const ENTRIES: [u32; 2] = [0x0812_63b2, 0x0812_6454];
const STOPS: [u32; 2] = [0x0812_63bc, 0x0812_645e];
const FIRST_ROOT: usize = 0x1263c4;
const INPUT: u32 = 0x0203_2000;

fn setup(machine: &mut Machine, which: usize, selector: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[2] = INPUT;
    machine.cpu.regs[6] = INPUT;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = ENTRIES[which];
    machine.cpu.cpsr = 0xbf;
    machine.bus.write8(INPUT + 3, selector);
    machine.bus.write8(INPUT + 6, selector);
}

#[test]
#[ignore = "exact EUR ROM and E218 predecessor; Wine only, Linux/USA pending"]
fn european_script_byte_lookup_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SCRIPT_BYTE_LOOKUP_3AD_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SCRIPT_BYTE_LOOKUP_3AD_PREVIOUS").unwrap());
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
        (218, 217, 129)
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
        if prior.id == THUMB_PC_LITERALS_ID {
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
    let data = &source.script_byte_lookup_3ad.as_ref().unwrap().decoded;
    let spans = data.source_spans().unwrap();
    let start = model::SOURCE_OFFSET as usize;
    let end = start + model::SOURCE_BYTES;
    assert_eq!(spans.len(), 2);
    assert_eq!(
        spans.iter().map(|(_, bytes, _)| bytes.len()).sum::<usize>(),
        10
    );
    for (offset, bytes, _) in &spans {
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
    }
    assert_eq!(
        parse_script_byte_lookup_3ad(
            &fs::read(prepared.directory.join(SCRIPT_BYTE_LOOKUP_3AD_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.source_offset += 1;
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ScriptByteLookup3ad>(invalid).is_err());
    let mut forged = data.clone();
    forged.values[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_script_byte_lookup_3ad(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == SCRIPT_BYTE_LOOKUP_3AD_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.script_byte_lookup_3ad = Some((&forged, &bytes));
    assert!(
        source.validate_files(files).is_err(),
        "coherent file/hash edits must be rejected"
    );
    drop(dataset);

    let prior_owned = |at: usize| {
        old.resources
            .iter()
            .flat_map(|entry| &entry.sources)
            .any(|span| {
                (span.offset as usize..span.offset as usize + span.length as usize).contains(&at)
            })
    };
    assert!(prior_owned(start - 1) && prior_owned(end) && prior_owned(FIRST_ROOT));
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for (offset, bytes, _) in &spans {
        for at in *offset as usize..*offset as usize + bytes.len() {
            managed.bus.rom[at] = !rom[at];
        }
    }
    for at in FIRST_ROOT..FIRST_ROOT + 4 {
        managed.bus.rom[at] = !rom[at];
    }
    let before = owner.script_byte_lookup_3ad_stats().unwrap();
    for (offset, bytes, _) in &spans {
        for at in *offset..*offset + bytes.len() as u32 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.script_byte_lookup_3ad_stats().unwrap();
    assert_eq!(
        (
            after.table_bytes - before.table_bytes,
            after.root_bytes - before.root_bytes,
            after.values_mask
        ),
        (6, 4, 63)
    );
    assert_eq!(
        managed.bus.read32(0x0800_0000 + FIRST_ROOT as u32),
        u32::from_le_bytes(rom[FIRST_ROOT..FIRST_ROOT + 4].try_into().unwrap())
    );
    let mut changed = BTreeSet::new();
    for (offset, bytes, _) in &spans {
        let at = *offset as usize;
        for x in (at - 3..at).chain(at + bytes.len()..at + bytes.len() + 4) {
            if data.byte_at(x as u32).is_none() && !(FIRST_ROOT..FIRST_ROOT + 4).contains(&x) {
                changed.insert(x);
                managed.bus.rom[x] = rom[x] ^ 0x5a;
            }
        }
    }
    for (offset, bytes, _) in &spans {
        for edge in [*offset as usize, *offset as usize + bytes.len()] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    let x = at + n;
                    if changed.contains(&x) && !prior_owned(x) {
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
    }
    for (offset, bytes, _) in &spans {
        let at = *offset as usize;
        managed.bus.rom[at..at + bytes.len()].copy_from_slice(&rom[at..at + bytes.len()]);
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }
    managed.bus.rom[FIRST_ROOT..FIRST_ROOT + 4].copy_from_slice(&rom[FIRST_ROOT..FIRST_ROOT + 4]);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses = (ENTRIES[0]..STOPS[0])
        .step_by(2)
        .chain((ENTRIES[1]..STOPS[1]).step_by(2));
    let blocks = addresses
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("script-byte-lookup-3ad16e-native.o");
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
        for which in 0..2 {
            for selector in [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 255] {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(machine, which, selector);
                }
                for (offset, bytes, _) in &spans {
                    for at in *offset as usize..*offset as usize + bytes.len() {
                        managed.bus.rom[at] = !rom[at];
                    }
                }
                for at in FIRST_ROOT..FIRST_ROOT + 4 {
                    managed.bus.rom[at] = !rom[at];
                }
                managed.bus.rom[start - 1] = !rom[start - 1];
                managed.bus.rom[end] = !rom[end];
                let before = owner.script_byte_lookup_3ad_stats().unwrap();
                for _ in 0..20 {
                    if raw.cpu.regs[15] == STOPS[which] {
                        break;
                    }
                    raw.bus.frame_ready = false;
                    raw.step();
                }
                for _ in 0..20 {
                    if managed.cpu.regs[15] == STOPS[which] {
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
                    (STOPS[which], STOPS[which])
                );
                let after = owner.script_byte_lookup_3ad_stats().unwrap();
                assert_eq!(
                    after.table_bytes - before.table_bytes,
                    if (1..=6).contains(&selector) { 1 } else { 0 }
                );
                assert_eq!(
                    after.root_bytes - before.root_bytes,
                    if which == 1 { 4 } else { 0 }
                );
                let at = (start as isize + selector as isize - 1) as usize;
                assert_eq!(raw.cpu.regs[2], u32::from(rom[at]));
                for (offset, bytes, _) in &spans {
                    let at = *offset as usize;
                    managed.bus.rom[at..at + bytes.len()]
                        .copy_from_slice(&rom[at..at + bytes.len()]);
                }
                managed.bus.rom[FIRST_ROOT..FIRST_ROOT + 4]
                    .copy_from_slice(&rom[FIRST_ROOT..FIRST_ROOT + 4]);
                managed.bus.rom[start - 1] = rom[start - 1];
                managed.bus.rom[end] = rom[end];
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "selector={selector}, reader={which}, native={native}"
                );
            }
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
                    .write32(0x0400_00d4, 0x0800_0000 + model::SOURCE_OFFSET - 2);
                machine.bus.write32(0x0400_00d8, 0x0203_2400);
                machine.bus.write16(0x0400_00dc, 32);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + model::SOURCE_OFFSET - 2,
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
        for at in start - 2..end + 4 {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .script_byte_lookup_3ad_stats()
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
                .script_byte_lookup_3ad_stats()
                .unwrap()
                .table_bytes
                > before
        );
        dma_managed.bus.rom[start - 2..end + 4].copy_from_slice(&rom[start - 2..end + 4]);
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
        "status":"PASS", "revision":[257,256], "window_bytes":6, "additional_root_bytes":4,
        "prior_other_resources_unchanged":119, "previous_compact_catalog_preserved":true, "previous_obj_lists_preserved":true, "previous_indexed_words_preserved":true, "prior_observed_lz_preserved":true, "prior_copied_halfwords_preserved":true, "prior_switch_tables_preserved":true, "prior_thumb_literals_preserved":true, "prior_literal_root_preserved":true, "adjacent_owners_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":40,
        "underflow_and_outside_indices_unchanged":true, "dma_cpuset_cpufastset_state_and_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
