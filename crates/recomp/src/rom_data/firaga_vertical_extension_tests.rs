//! EUR Firaga vertical lookup: a bounded physical projection of uncapped guest indexing.
use super::*;
use gba_core::{capi::RT_API, rom_data::observed_task_parameters as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::sync::Arc;

const START: usize = 0x394e20;
const END: usize = 0x394e36;
const ENTRY: u32 = 0x0802_64ea;
const STOP: u32 = 0x0802_64fa;

fn setup(machine: &mut Machine, byte: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[2] = 0x8000_0010;
    machine.cpu.regs[6] = 0x0202_0000;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = ENTRY;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write8(0x0202_0007, byte);
}

#[test]
#[ignore = "exact EUR ROM and E266 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_vertical_projection_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_VERTICAL_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_VERTICAL_PREVIOUS").unwrap());
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
        (266, 265, 146)
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
        (267, 266, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id != OBSERVED_TASK_PARAMETERS_ID {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(prepared.directory.join(&current.path)).unwrap(),
                fs::read(previous.join(&prior.path)).unwrap()
            );
            continue;
        }
        let old_data =
            parse_observed_task_parameters(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
        let mut new_data = parse_observed_task_parameters(
            &fs::read(prepared.directory.join(&current.path)).unwrap(),
        )
        .unwrap();
        assert_eq!(old_data.vertical_corrections.entries.len(), 6);
        assert_eq!(new_data.vertical_corrections.entries.len(), model::VERTICAL_COUNT);
        for (i, row) in new_data.vertical_corrections.entries[6..]
            .iter()
            .enumerate()
        {
            assert_eq!(usize::from(row.index), i + 10);
            assert_eq!(
                row.correction,
                i16::from_le_bytes(rom[START + i * 2..START + i * 2 + 2].try_into().unwrap())
            );
        }
        new_data.vertical_corrections.entries.truncate(6);
        assert_eq!(new_data, old_data);
        new_data.vertical_corrections.entries.truncate(4);
        assert!(new_data.validate().is_ok());
        let mut metadata = current.clone();
        *metadata
            .sources
            .iter_mut()
            .find(|s| s.role == "vertical-correction")
            .unwrap() = prior
            .sources
            .iter()
            .find(|s| s.role == "vertical-correction")
            .unwrap()
            .clone();
        metadata.sha256 = prior.sha256.clone();
        assert_eq!(&metadata, prior);
    }
    let parameters = &source.observed_task_parameters.as_ref().unwrap().decoded;
    let mut invalid = parameters.clone();
    invalid.vertical_corrections.entries.pop();
    assert!(invalid.validate().is_err());
    let mut forged = parameters.clone();
    forged.vertical_corrections.entries[6].correction ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_observed_task_parameters(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut manifest = dataset.manifest.clone();
    manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == OBSERVED_TASK_PARAMETERS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &manifest;
    files.observed_task_parameters = Some((&forged, &bytes));
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
    let before = owner.observed_task_parameters_stats().unwrap();
    for at in [0x371064usize, 0x37101c, 0x371020] {
        assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]);
    }
    let unrelated = owner.observed_task_parameters_stats().unwrap();
    assert_eq!(
        unrelated.corrections_used_mask,
        before.corrections_used_mask
    );
    for at in START..END {
        managed.bus.rom[at] = !rom[at];
    }
    for at in START..END {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
        }
    }
    let selected = ((1u64 << 17) - 1) & !((1u64 << 6) - 1);
    let after = owner.observed_task_parameters_stats().unwrap();
    assert_eq!(after.corrections_used_mask, selected);
    assert_eq!(after.corrections_used, 11);
    assert_eq!(
        (after.phases_used, after.signed_lookup_indices_mask),
        (unrelated.phases_used, unrelated.signed_lookup_indices_mask)
    );
    assert_eq!(
        after.vertical_correction_bytes - unrelated.vertical_correction_bytes,
        66
    );
    // Neighboring unowned bytes must continue to use the current ROM backing.
    for at in END..END + 4 {
        managed.bus.rom[at] = rom[at] ^ 0x5a;
    }
    for edge in [START, END] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| {
                if (END..END + 4).contains(&(at + n)) {
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
    managed.bus.rom[START..END + 4].copy_from_slice(&rom[START..END + 4]);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = (ENTRY..STOP)
        .step_by(2)
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-vertical-native.o");
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
        for byte in (20u8..42).chain([18, 19, 42, 43, 254, 255]) {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                setup(machine, byte);
            }
            for at in START..END {
                managed.bus.rom[at] = !rom[at];
            }
            let before = owner
                .observed_task_parameters_stats()
                .unwrap()
                .vertical_correction_bytes;
            for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                for _ in 0..16 {
                    if machine.cpu.regs[15] == STOP {
                        break;
                    }
                    machine.bus.frame_ready = false;
                    if use_native {
                        table.get(machine.cpu.regs[15] | 1).unwrap()(
                            &RT_API,
                            machine as *mut Machine as *mut std::ffi::c_void,
                        );
                    } else {
                        machine.step();
                    }
                }
                assert_eq!(machine.cpu.regs[15], STOP);
            }
            let offset = 0x394e0c + usize::from(byte >> 1) * 2;
            let correction = i16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap());
            assert_eq!(
                raw.cpu.regs[2],
                0x8000_0010u32.wrapping_sub(i32::from(correction) as u32)
            );
            let owned = (18..42).contains(&byte);
            assert_eq!(
                owner
                    .observed_task_parameters_stats()
                    .unwrap()
                    .vertical_correction_bytes
                    - before,
                if owned { 2 } else { 0 }
            );
            managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "byte={byte}, native={native}"
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 56);

    let mut dma_raw = crate::make_machine(rom.clone(), None);
    let mut dma_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut dma_managed, &prepared.directory).unwrap();
    let dma_initial = dma_raw.save_state().unwrap();
    for kind in 0..3 {
        for machine in [&mut dma_raw, &mut dma_managed] {
            machine.load_state(&dma_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + START as u32);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 2);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[
                    0x0a00_0000 + START as u32,
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
        for at in START..END {
            dma_managed.bus.rom[at] = !rom[at];
        }
        let transfer_owner = dma_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner
            .observed_task_parameters_stats()
            .unwrap()
            .vertical_correction_bytes;
        for machine in [&mut dma_raw, &mut dma_managed] {
            if kind == 0 {
                machine.bus.write16(0x0400_00de, 0x8000);
            } else {
                machine.step();
            }
        }
        assert!(
            transfer_owner
                .observed_task_parameters_stats()
                .unwrap()
                .vertical_correction_bytes
                > before
        );
        dma_managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
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
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status":"PASS", "revision":[267,266], "resources":146, "new_source_bytes":22,
            "prior_other_resources_unchanged":145, "prior_representations_preserved":true,
            "forged_source_rejected":true, "poisoned_mixed_boundaries_and_mirrors":true,
            "usage_accounting_isolated":true, "original_interpreter_generated_reader_cases":cases,
            "dma_cpuset_cpufastset_cases":3, "full_state_clock_equal":true,
            "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
            "native_windows":"pending", "performance_measurements":false
        }))
        .unwrap(),
    )
    .unwrap();
}
