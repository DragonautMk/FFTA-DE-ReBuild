//! Additional EUR White Flowers Thumb literal pools and original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::thumb_pc_literals as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, reader: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + reader;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E282 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_many_literals_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_MANY_LITERALS_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_WHITE_MANY_LITERALS_PREVIOUS").unwrap());
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
        (282, 281, 146)
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
        (283, 282, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id == THUMB_PC_LITERALS_ID {
            let old_data = parse_thumb_pc_literals(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_thumb_pc_literals(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            assert!(old_data.white_flowers_many.is_none());
            assert!(new_data.white_flowers_many.take().is_some());
            assert_eq!(new_data, old_data);
            assert_eq!(&current.sources[..prior.sources.len()], prior.sources.as_slice());
            assert_eq!(current.sources.len() - prior.sources.len(), model::WHITE_FLOWERS_MANY_COUNT);
            assert_eq!((&current.id, &current.path, &current.format, &current.dependencies), (&prior.id, &prior.path, &prior.format, &prior.dependencies));
        } else {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(prepared.directory.join(&current.path)).unwrap(),
                fs::read(previous.join(&prior.path)).unwrap()
            );
        }
    }
    let data = &source.thumb_pc_literals.as_ref().unwrap().decoded;
    let words = data.white_flowers_many.as_ref().unwrap();
    assert_eq!(words.len(), model::WHITE_FLOWERS_MANY_COUNT);
    for literal in words {
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
    invalid.white_flowers_many.as_mut().unwrap().swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap();
    invalid["white_flowers_many"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::ThumbPcLiterals>(invalid).is_err());
    let mut forged = data.clone();
    forged.white_flowers_many.as_mut().unwrap()[0].value ^= 1;
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
    for literal in words {
        let start = literal.source_offset as usize;
        for at in start..start + 4 {
            managed.bus.rom[at] = !rom[at];
        }
    }
    let before = owner.thumb_pc_literals_stats().unwrap();
    for literal in words {
        for at in literal.source_offset..literal.source_offset + 4 {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at), rom[at as usize]);
            }
        }
    }
    let after = owner.thumb_pc_literals_stats().unwrap();
    assert_eq!(
        after.white_flowers_many_bytes - before.white_flowers_many_bytes,
        1104
    );
    assert_eq!(after.white_flowers_many_mask, [u64::MAX, (1u64 << 28) - 1]);

    // Alter only adjacent bytes without any prior or new owner.
    let prior_owned = |at: usize| {
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
    for &(offset, _, _) in &model::WHITE_FLOWERS_MANY {
        let offset = offset as usize;
        for at in (offset - 3..offset).chain(offset + 4..offset + 8) {
            if data.byte_at(at as u32).is_none() && !prior_owned(at) {
                changed.insert(at);
                managed.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
    }
    for &(offset, _, _) in &model::WHITE_FLOWERS_MANY {
        for edge in [offset as usize, offset as usize + 4] {
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
    for literal in words {
        let start = literal.source_offset as usize;
        managed.bus.rom[start..start + 4].copy_from_slice(&rom[start..start + 4]);
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }

    // Split neighboring halfword readers into two non-overlapping generated block sets.
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut reader_cases = 0;
    for group in 0..2 {
        let indices: Vec<_> = (0..model::WHITE_FLOWERS_MANY_COUNT)
            .filter(|&i| (model::WHITE_FLOWERS_MANY[i].1 / 2) % 2 == group)
            .collect();
        let addresses: Vec<_> = indices
            .iter()
            .map(|&i| 0x0800_0000 + model::WHITE_FLOWERS_MANY[i].1)
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
        let object = root.join(format!(
            "white-flowers-many-literals-native-{group}.o"
        ));
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
            for &i in &indices {
                let (offset, reader, opcode) = model::WHITE_FLOWERS_MANY[i];
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    setup(machine, reader);
                }
                for at in offset as usize..offset as usize + 4 {
                    managed.bus.rom[at] = !rom[at];
                }
                let before = owner
                    .thumb_pc_literals_stats()
                    .unwrap()
                    .white_flowers_many_bytes;
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
                    owner
                        .thumb_pc_literals_stats()
                        .unwrap()
                        .white_flowers_many_bytes
                        - before,
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
                reader_cases += 1;
            }
        }
    }
    assert_eq!(reader_cases, 184);

    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[283,282], "resources":146,
        "white_flowers_many_literals":92, "new_source_bytes":368,
        "prior_representation_preserved":true, "prior_other_resources_unchanged":145,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_reader_cases":reader_cases,
        "full_state_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
