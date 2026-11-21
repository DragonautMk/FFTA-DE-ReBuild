//! EUR Thumb literals with readers statically anchored to translated home-scene blocks.
use super::*;
use gba_core::{capi::RT_API, rom_data::thumb_pc_literals as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

pub(super) fn assert_previous_thumb_literals_preserved(
    previous: &std::path::Path,
    current_directory: &std::path::Path,
    prior: &gba_core::rom_data::Resource,
    current: &gba_core::rom_data::Resource,
) {
    assert_eq!(
        (
            &current.id,
            &current.path,
            &current.format,
            &current.dependencies
        ),
        (&prior.id, &prior.path, &prior.format, &prior.dependencies)
    );
    assert_eq!(
        &current.sources[..prior.sources.len()],
        prior.sources.as_slice()
    );
    assert_eq!(
        current.sources.len(),
        model::LITERAL_COUNT
            + model::ADDITIONAL_COUNT
            + model::OBSERVED_COUNT
            + model::HOME_ANCHORED_COUNT
            + model::HOME_OBSERVED_COUNT
    );
    let old: serde_json::Value =
        serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
    let new: serde_json::Value =
        serde_json::from_slice(&fs::read(current_directory.join(&current.path)).unwrap()).unwrap();
    for name in ["literals", "additional", "observed", "home_anchored"] {
        if let Some(value) = old.get(name) {
            assert_eq!(value, &new[name]);
        }
    }
}

fn setup(machine: &mut Machine, reader: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + reader;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E219 predecessor; Wine only, Linux/USA pending"]
fn european_home_anchored_thumb_literals_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_HOME_ANCHORED_THUMB_LITERALS_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_HOME_ANCHORED_THUMB_LITERALS_PREVIOUS").unwrap());
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
        (219, 218, 130)
    );
    let old_json: serde_json::Value =
        serde_json::from_slice(&fs::read(previous.join(THUMB_PC_LITERALS_FILE)).unwrap()).unwrap();
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
            assert_eq!(
                (
                    &current.id,
                    &current.path,
                    &current.format,
                    &current.dependencies
                ),
                (&prior.id, &prior.path, &prior.format, &prior.dependencies)
            );
            assert_eq!(
                &current.sources
                    [..model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT],
                prior.sources.as_slice()
            );
            assert_eq!(
                current.sources.len(),
                model::LITERAL_COUNT
                    + model::ADDITIONAL_COUNT
                    + model::OBSERVED_COUNT
                    + model::HOME_ANCHORED_COUNT
                    + model::HOME_OBSERVED_COUNT
            );
        } else if prior.id == THUMB_SWITCH_TABLES_ID {
            super::thumb_switch_tables_additional_tests::assert_previous_switch_tables_preserved(
                &previous,
                &prepared.directory,
                prior,
                current,
            );
        } else {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(prepared.directory.join(&current.path)).unwrap(),
                fs::read(previous.join(&prior.path)).unwrap()
            );
        }
    }
    let data = &source.thumb_pc_literals.as_ref().unwrap().decoded;
    assert_eq!(
        old_json["literals"],
        serde_json::to_value(&data.literals).unwrap()
    );
    assert_eq!(
        old_json["additional"],
        serde_json::to_value(&data.additional).unwrap()
    );
    assert_eq!(
        old_json["observed"],
        serde_json::to_value(&data.observed).unwrap()
    );
    assert_eq!(data.home_anchored.len(), 25);
    for literal in &data.home_anchored {
        let at = literal.source_offset as usize;
        assert_eq!(literal.value.to_le_bytes(), rom[at..at + 4]);
    }
    assert_eq!(
        parse_thumb_pc_literals(
            &fs::read(prepared.directory.join(THUMB_PC_LITERALS_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.home_anchored.swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["home_anchored"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ThumbPcLiterals>(invalid).is_err());
    let mut forged = data.clone();
    forged.home_anchored[0].value ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_thumb_pc_literals(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered_manifest;
    files.thumb_pc_literals = Some((&forged, &bytes));
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
    for literal in data
        .literals
        .iter()
        .chain(&data.additional)
        .chain(&data.observed)
        .chain(&data.home_anchored)
    {
        let start = literal.source_offset as usize;
        for at in start..start + 4 {
            managed.bus.rom[at] = !rom[at];
        }
    }
    let before = owner.thumb_pc_literals_stats().unwrap();
    for literal in data
        .literals
        .iter()
        .chain(&data.additional)
        .chain(&data.observed)
        .chain(&data.home_anchored)
    {
        for at in literal.source_offset..literal.source_offset + 4 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    let after = owner.thumb_pc_literals_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 3644);
    assert_eq!(
        after.literals_mask,
        [u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, 0x3f]
    );
    assert_eq!(
        after.additional_source_bytes - before.additional_source_bytes,
        468
    );
    assert_eq!(after.additional_mask, [u64::MAX, (1u64 << 53) - 1]);
    assert_eq!(
        after.observed_source_bytes - before.observed_source_bytes,
        1772
    );
    assert_eq!(
        after.observed_mask,
        [
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            (1u64 << 59) - 1
        ]
    );
    assert_eq!(after.home_anchored_bytes - before.home_anchored_bytes, 100);
    assert_eq!(after.home_anchored_mask, (1u64 << 25) - 1);
    // Selected boundaries include previous owners, adjacent words and raw bytes.
    let prior_owned = |at: usize| {
        old.resources
            .iter()
            .flat_map(|entry| &entry.sources)
            .any(|span| {
                (span.offset as usize..span.offset as usize + span.length as usize).contains(&at)
            })
    };
    let boundary_indices = [0usize, 1, 5, 8, 12, 17, 18, 19, 20, 24];
    let mut changed = BTreeSet::new();
    for i in boundary_indices {
        let offset = model::HOME_ANCHORED[i].0 as usize;
        for at in (offset - 3..offset).chain(offset + 4..offset + 8) {
            if data.byte_at(at as u32).is_none() {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for i in boundary_indices {
        let offset = model::HOME_ANCHORED[i].0 as usize;
        for edge in [offset, offset + 4] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |n: usize| {
                    let x = at + n;
                    if data.byte_at(x as u32).is_some() || prior_owned(x) || !changed.contains(&x) {
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
    for literal in data
        .literals
        .iter()
        .chain(&data.additional)
        .chain(&data.observed)
        .chain(&data.home_anchored)
    {
        let start = literal.source_offset as usize;
        managed.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }

    let indices = [
        0usize, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    ];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses: Vec<u32> = indices
        .iter()
        .map(|&i| 0x0800_0000 + model::HOME_ANCHORED[i].1)
        .collect();
    assert!(addresses
        .iter()
        .all(|a| !addresses.iter().any(|b| a != b && a.abs_diff(*b) == 2)));
    let blocks = addresses
        .iter()
        .map(|&addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("home-anchored-thumb-literals-native.o");
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
        for i in indices {
            let (offset, reader, opcode, _, _) = model::HOME_ANCHORED[i];
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, reader);
            }
            for at in offset as usize..offset as usize + 4 {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner.thumb_pc_literals_stats().unwrap().home_anchored_bytes;
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
                owner.thumb_pc_literals_stats().unwrap().home_anchored_bytes - before,
                4
            );
            let expected = u32::from_le_bytes(
                rom[offset as usize..offset as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(raw.cpu.regs[usize::from((opcode >> 8) & 7)], expected);
            assert_eq!(
                (raw.cpu.regs[15], managed.cpu.regs[15]),
                (0x0800_0000 + reader + 2, 0x0800_0000 + reader + 2)
            );
            managed.bus.rom[offset as usize..offset as usize + 4]
                .copy_from_slice(&rom[offset as usize..offset as usize + 4]);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "literal {offset:x}, native={native}"
            );
        }
    }

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    let offset = model::HOME_ANCHORED[0].0;
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + offset,
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
        for at in offset as usize..offset as usize + 4 {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .thumb_pc_literals_stats()
            .unwrap()
            .home_anchored_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .thumb_pc_literals_stats()
                .unwrap()
                .home_anchored_bytes
                > before
        );
        dma_managed.bus.rom[offset as usize..offset as usize + 4]
            .copy_from_slice(&rom[offset as usize..offset as usize + 4]);
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
        "status":"PASS", "revision":[221,220], "home_anchored_literals":25, "home_anchored_source_bytes":100,
        "prior_literals_preserved":886, "prior_other_resources_unchanged":128, "prior_switch_tables_preserved":true, "legacy_masks_preserved":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":48,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
