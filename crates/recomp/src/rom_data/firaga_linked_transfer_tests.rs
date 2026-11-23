//! EUR Firaga type-3 transfer row and its shared-wrapper descriptor.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{observed_tilemap_descriptor as descriptor, observed_tilemap_row as row},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, len) in [
        (row::FIRAGA_TRANSFER_ROW_OFFSET, row::SOURCE_LENGTH),
        (
            descriptor::FIRAGA_TRANSFER_DESCRIPTOR_OFFSET,
            descriptor::SOURCE_LENGTH,
        ),
    ] {
        for at in start as usize..start as usize + len {
            machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E272 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_linked_transfer_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_TRANSFER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_TRANSFER_PREVIOUS").unwrap());
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
        (272, 271, 146)
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
        (273, 272, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID {
            let mut metadata = current.clone();
            let added = metadata.sources.pop().unwrap();
            assert_eq!(
                (added.offset, added.length),
                if prior.id == OBSERVED_TILEMAP_ROW_ID {
                    (row::FIRAGA_TRANSFER_ROW_OFFSET, 12)
                } else {
                    (descriptor::FIRAGA_TRANSFER_DESCRIPTOR_OFFSET, 8)
                }
            );
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
            if prior.id == OBSERVED_TILEMAP_ROW_ID {
                let old_data =
                    parse_observed_tilemap_row(&fs::read(previous.join(&prior.path)).unwrap())
                        .unwrap();
                let mut new_data = parse_observed_tilemap_row(
                    &fs::read(prepared.directory.join(&current.path)).unwrap(),
                )
                .unwrap();
                assert!(old_data.firaga_transfer_row.is_none());
                assert!(new_data.firaga_transfer_row.take().is_some());
                assert_eq!(new_data, old_data);
            } else {
                let old_data = parse_observed_tilemap_descriptor(
                    &fs::read(previous.join(&prior.path)).unwrap(),
                )
                .unwrap();
                let mut new_data = parse_observed_tilemap_descriptor(
                    &fs::read(prepared.directory.join(&current.path)).unwrap(),
                )
                .unwrap();
                assert!(old_data.firaga_transfer_descriptor.is_none());
                assert!(new_data.firaga_transfer_descriptor.take().is_some());
                assert_eq!(new_data, old_data);
            }
        } else {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(previous.join(&prior.path)).unwrap(),
                fs::read(prepared.directory.join(&current.path)).unwrap()
            );
        }
    }
    let rows = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    let desc = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    rows.validate_dependencies(desc).unwrap();
    desc.validate_lz_dependencies(&source.observed_lz.as_ref().unwrap().decoded)
        .unwrap();
    let transfer = rows.firaga_transfer_row.as_ref().unwrap();
    let entry = desc.firaga_transfer_descriptor.as_ref().unwrap();
    assert_eq!(
        transfer.descriptor_pointer,
        0x0800_0000 + entry.source_offset
    );
    assert_eq!(entry.wrapper, desc.additional_descriptors[1].wrapper);
    assert_eq!(
        desc.source_spans()
            .iter()
            .filter(|(start, _, _)| *start == descriptor::FIRAGA_TRANSFER_WRAPPER_OFFSET)
            .count(),
        1
    );
    let mut missing = desc.clone();
    missing.firaga_transfer_descriptor = None;
    assert!(rows.validate_dependencies(&missing).is_err());
    let mut bad = desc.clone();
    bad.firaga_transfer_descriptor
        .as_mut()
        .unwrap()
        .wrapper
        .opaque_halfword ^= 1;
    assert!(bad.validate().is_err());
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut forged_row = rows.clone();
    forged_row.firaga_transfer_row.as_mut().unwrap().field8 ^= 1;
    let forged_row_bytes = encode_observed_tilemap_row(&forged_row).unwrap();
    let mut altered = dataset.manifest.clone();
    altered
        .resources
        .iter_mut()
        .find(|e| e.id == OBSERVED_TILEMAP_ROW_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&forged_row_bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.observed_tilemap_row = Some((&forged_row, &forged_row_bytes));
    assert!(source.validate_files(files).is_err());
    let mut forged_desc = desc.clone();
    forged_desc
        .firaga_transfer_descriptor
        .as_mut()
        .unwrap()
        .parameter16 ^= 1;
    let forged_desc_bytes = encode_observed_tilemap_descriptor(&forged_desc).unwrap();
    let mut altered = dataset.manifest.clone();
    altered
        .resources
        .iter_mut()
        .find(|e| e.id == OBSERVED_TILEMAP_DESCRIPTOR_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&forged_desc_bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.observed_tilemap_descriptor = Some((&forged_desc, &forged_desc_bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    set_backing(&mut managed, &rom, true);
    let ranges = [
        (row::FIRAGA_TRANSFER_ROW_OFFSET as usize, row::SOURCE_LENGTH),
        (
            descriptor::FIRAGA_TRANSFER_DESCRIPTOR_OFFSET as usize,
            descriptor::SOURCE_LENGTH,
        ),
    ];
    for (start, len) in ranges {
        for at in start..start + len {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
            }
        }
    }
    assert_eq!(
        owner
            .observed_tilemap_row_stats()
            .unwrap()
            .firaga_transfer_row_bytes,
        36
    );
    assert_eq!(
        owner
            .observed_tilemap_descriptor_stats()
            .unwrap()
            .firaga_transfer_descriptor_bytes,
        24
    );
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
    for (start, len) in ranges {
        for edge in [start, start + len] {
            for at in edge.saturating_sub(3)..edge + 3 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    for (start, len) in ranges {
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

    let ranges = [
        (0x080d_b0a0, 0x080d_b0de),
        (0x080d_b30a, 0x080d_b320),
        (0x080d_b34e, 0x080d_b36a),
    ];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses: BTreeSet<_> = ranges
        .iter()
        .flat_map(|&(start, end)| (start..end).step_by(2))
        .collect();
    let blocks = addresses
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-linked-transfer-native.o");
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
        for (start, stop) in ranges {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                machine.bus.ffta_sprites.enabled = false;
                machine
                    .bus
                    .write32(0x0201_8000, 0x0800_0000 + transfer.source_offset);
                machine.bus.write8(0x0201_8007, 0);
                machine.bus.write32(0x0201_8024, 0x0202_42bc);
                machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
                machine.cpu.regs[4] = 0x0201_8000;
                machine.cpu.regs[13] = 0x0300_7c00;
                machine.cpu.regs[15] = start;
                machine.cpu.cpsr = 0xbf;
            }
            set_backing(&mut managed, &rom, true);
            for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                let mut steps = 0;
                while machine.cpu.regs[15] != stop {
                    assert!(steps < 48, "reader escaped at {:08x}", machine.cpu.regs[15]);
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
            set_backing(&mut managed, &rom, false);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "reader={start:08x} native={native}"
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 6);
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[273,272], "resources":146, "new_source_bytes":20,
        "prior_other_resources_unchanged":144, "prior_representation_preserved":true,
        "shared_wrapper_and_stream_owned_once":true, "unselected_sources_fall_back":true,
        "poisoned_mirrors_and_mixed_boundaries":true, "original_interpreter_generated_cases":cases,
        "full_state_clock_equal":true, "coherent_edits_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
