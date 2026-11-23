//! Scoped EUR index-2 projections and their original guest readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::soil_scoped_values as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

fn poison(machine: &mut Machine, rom: &[u8], enabled: bool) {
    for &(start, len) in &model::SPANS[6..] {
        for at in start as usize..start as usize + len {
            machine.bus.rom[at] = if enabled { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E267 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_scoped_index2_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_SCOPED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_SCOPED_PREVIOUS").unwrap());
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
        (267, 266, 146)
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
        (268, 267, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        if prior.id != SOIL_SCOPED_VALUES_ID {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(previous.join(&prior.path)).unwrap(),
                fs::read(prepared.directory.join(&current.path)).unwrap()
            );
        } else {
            let old_data =
                parse_soil_scoped_values(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_soil_scoped_values(
                &fs::read(prepared.directory.join(&current.path)).unwrap(),
            )
            .unwrap();
            assert_eq!(old_data.source_spans().unwrap().len(), 6);
            assert_eq!(new_data.source_spans().unwrap().len(), 8);
            assert_eq!(
                new_data.word_a820.unwrap().to_le_bytes(),
                rom[0x39a820..0x39a824]
            );
            assert_eq!(new_data.byte_a876, Some(rom[0x39a876]));
            new_data.word_a820 = None;
            new_data.byte_a876 = None;
            assert_eq!(new_data, old_data);
            let mut metadata = current.clone();
            metadata.sources.truncate(6);
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
        }
    }
    let decoded = &source.soil_scoped_values.as_ref().unwrap().decoded;
    let mut incomplete = decoded.clone();
    incomplete.byte_a876 = None;
    assert!(incomplete.validate().is_err());
    let mut forged = decoded.clone();
    *forged.word_a820.as_mut().unwrap() ^= 1;
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_soil_scoped_values(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut manifest = dataset.manifest.clone();
    manifest
        .resources
        .iter_mut()
        .find(|r| r.id == SOIL_SCOPED_VALUES_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &manifest;
    files.soil_scoped_values = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    poison(&mut managed, &rom, true);
    for &(start, len) in &model::SPANS[6..] {
        for at in start as usize..start as usize + len {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
            }
        }
    }
    assert_eq!(owner.soil_scoped_values_stats().unwrap().values_mask, 0xc0);
    let owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| (s.offset as usize..(s.offset + s.length) as usize).contains(&at))
    };
    let mut changed = BTreeSet::new();
    for &(start, len) in &model::SPANS[6..] {
        for edge in [start as usize, start as usize + len] {
            for at in edge.saturating_sub(3)..edge + 4 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    for &(start, len) in &model::SPANS[6..] {
        for edge in [start as usize, start as usize + len] {
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
    let blocks = [0x080e_61cc, 0x080e_61d6]
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-scoped-index2-native.o");
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
        for (pc, base, width, dst) in [
            (0x080e_61cc, 0x39a818u32, 4u32, 2usize),
            (0x080e_61d6, 0x39a874, 1, 1),
        ] {
            for index in [2u32, 3, 4] {
                let at = base + index * width;
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    machine.cpu.regs[0] = 0x0800_0000 + at;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[15] = pc;
                    machine.cpu.cpsr = 0xbf;
                }
                poison(&mut managed, &rom, true);
                for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                    let mut steps = 0;
                    while machine.cpu.regs[15] != pc + 2 {
                        assert!(steps < 8);
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
                let expected = (0..width).fold(0u32, |v, i| {
                    v | (u32::from(rom[(at + i) as usize]) << (8 * i))
                });
                assert_eq!(raw.cpu.regs[dst], expected);
                poison(&mut managed, &rom, false);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "pc={pc:08x} index={index} native={native}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 12);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[268,267], "resources":146, "new_source_bytes":5,
        "prior_other_resources_unchanged":145, "prior_representation_preserved":true,
        "incomplete_and_forged_source_rejected":true, "poisoned_mixed_boundaries_and_mirrors":true,
        "original_interpreter_generated_reader_cases":cases, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
