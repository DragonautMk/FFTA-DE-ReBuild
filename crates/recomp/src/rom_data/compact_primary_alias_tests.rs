//! Conditional primary-task alias publication and original reader prefixes.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::compact_primary_alias::{
        Catalog, CompactPrimaryAlias, LOOKUP_OFFSET, POSITION_OFFSET,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::sync::Arc;

const STACK: u32 = 0x0300_7c00;
const MESSAGE: u32 = 0x0201_0000;
const CONTEXT: u32 = 0x0201_1000;

fn setup(machine: &mut Machine, case: usize) -> u32 {
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.cpsr = 0xbf;
    match case {
        0 => {
            machine.cpu.regs[0] = 362;
            machine.cpu.regs[15] = 0x080d_7020;
            0x080d_702a
        }
        1 => {
            machine.cpu.regs[4] = 362;
            machine.cpu.regs[15] = 0x080d_7070;
            0x080d_707e
        }
        2 => {
            machine.bus.write16(MESSAGE + 0x14, 362);
            machine.cpu.regs[8] = MESSAGE;
            machine.cpu.regs[5] = CONTEXT;
            machine.cpu.regs[15] = 0x0811_eae4;
            0x0811_eaec
        }
        3 => {
            machine.bus.write8(CONTEXT + 2, 255);
            machine.cpu.regs[5] = CONTEXT;
            machine.cpu.regs[15] = 0x0811_eb4a;
            0x0811_eb5a
        }
        4 => {
            machine.bus.write8(CONTEXT + 2, 255);
            machine.cpu.regs[5] = CONTEXT;
            machine.cpu.regs[15] = 0x0811_eb84;
            0x0811_eb8e
        }
        _ => unreachable!(),
    }
}

#[test]
#[ignore = "exact EUR ROM, E171 predecessor and TinyCC; Wine only"]
fn european_compact_primary_alias_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_PRIMARY_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_COMPACT_PRIMARY_PREVIOUS").unwrap());
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
        (171, 170, 114)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (172, 171, 115)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let entry = source
        .manifest
        .resources
        .iter()
        .find(|r| r.id == COMPACT_PRIMARY_ALIAS_ID)
        .unwrap();
    assert_eq!(
        entry.dependencies,
        vec![
            TASK_DESCRIPTORS_ID.to_string(),
            COMPACT_OBJ_CATALOG_ID.to_string()
        ]
    );
    let catalog = source.compact_primary_alias.as_ref().unwrap();
    assert_eq!(catalog.sources.len(), 2);
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 5);
    let model: CompactPrimaryAlias =
        serde_json::from_slice(&serde_json::to_vec(&catalog.decoded).unwrap()).unwrap();
    let mut invalid = model.clone();
    invalid.lookup_offset += 1;
    assert!(invalid.validate().is_err());
    let mut overlap = old.resources.clone();
    overlap[0].sources.push(catalog.sources[0].clone());
    assert!(Catalog::from_rom(&rom, &overlap).is_err());
    let mut decoded = read_dataset(&prepared.directory).unwrap();
    source.validate_files(decoded.files()).unwrap();
    let mut bad_tasks = decoded.task_descriptors.as_ref().unwrap().clone();
    bad_tasks.script.entries[0].callback = None;
    assert!(model
        .validate_dependencies(&bad_tasks, decoded.compact_obj_catalog.as_ref().unwrap())
        .is_err());
    let mut bad_selector = model.clone();
    bad_selector.selector = 255;
    assert!(bad_selector
        .validate_dependencies(
            decoded.task_descriptors.as_ref().unwrap(),
            decoded.compact_obj_catalog.as_ref().unwrap()
        )
        .is_err());
    decoded.compact_primary_alias.as_mut().unwrap().selector ^= 1;
    let changed =
        encode_compact_primary_alias(decoded.compact_primary_alias.as_ref().unwrap()).unwrap();
    decoded
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == COMPACT_PRIMARY_ALIAS_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&changed);
    decoded.compact_primary_alias_bytes = Some(changed);
    assert!(source.validate_files(decoded.files()).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let file = prepared.directory.join(COMPACT_PRIMARY_ALIAS_FILE);
    let saved_file = fs::read(&file).unwrap();
    fs::write(&file, b"{}").unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::write(&file, saved_file).unwrap();
    let poison = |m: &mut Machine| {
        for (a, b) in [
            (LOOKUP_OFFSET as usize, LOOKUP_OFFSET as usize + 1),
            (POSITION_OFFSET as usize, POSITION_OFFSET as usize + 4),
        ] {
            for i in a..b {
                m.bus.rom[i] = rom[i] ^ 0xff;
            }
        }
    };
    let restore = |m: &mut Machine| {
        for (a, b) in [
            (LOOKUP_OFFSET as usize, LOOKUP_OFFSET as usize + 1),
            (POSITION_OFFSET as usize, POSITION_OFFSET as usize + 4),
        ] {
            m.bus.rom[a..b].copy_from_slice(&rom[a..b]);
        }
    };
    poison(&mut managed);
    let before = owner.compact_primary_alias_stats().unwrap();
    for i in 0..1 {
        assert_eq!(
            managed.bus.read8(0x0800_0000 + LOOKUP_OFFSET + i),
            rom[(LOOKUP_OFFSET + i) as usize]
        );
    }
    for i in 0..4 {
        assert_eq!(
            managed.bus.read8(0x0800_0000 + POSITION_OFFSET + i),
            rom[(POSITION_OFFSET + i) as usize]
        );
    }
    let after = owner.compact_primary_alias_stats().unwrap();
    assert_eq!(
        (
            after.selector_bytes - before.selector_bytes,
            after.position_bytes - before.position_bytes
        ),
        (1, 4)
    );
    let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
    let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for mirror in [0x0800_0000u32, 0x0a00_0000, 0x0c00_0000] {
        for edge in [
            LOOKUP_OFFSET,
            LOOKUP_OFFSET + 1,
            POSITION_OFFSET,
            POSITION_OFFSET + 4,
        ] {
            for delta in [-3i32, -1, 0, 1] {
                let at = mirror + (edge as i32 + delta) as u32;
                assert_eq!((RT_API.read8)(raw_ptr, at), (RT_API.read8)(managed_ptr, at));
                assert_eq!(
                    (RT_API.read16)(raw_ptr, at),
                    (RT_API.read16)(managed_ptr, at)
                );
                assert_eq!(
                    (RT_API.read32)(raw_ptr, at),
                    (RT_API.read32)(managed_ptr, at)
                );
            }
        }
    }
    restore(&mut managed);
    let sequence_offset = 0x6cb650usize;
    managed.bus.rom[sequence_offset..sequence_offset + 2]
        .iter_mut()
        .for_each(|b| *b ^= 0xff);
    let before_compact = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    assert_eq!(
        raw.bus.read16(0x0800_0000 + sequence_offset as u32),
        managed.bus.read16(0x0800_0000 + sequence_offset as u32)
    );
    assert!(owner.compact_obj_catalog_stats().unwrap().source_bytes > before_compact);
    managed.bus.rom[sequence_offset..sequence_offset + 2]
        .copy_from_slice(&rom[sequence_offset..sequence_offset + 2]);
    let initial = raw.save_state().unwrap();
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&raw.bus.iwram),
        bios: None,
    };
    let blocks = [
        (0x080d_7020, 0x080d_702a),
        (0x080d_7070, 0x080d_707e),
        (0x0811_eae4, 0x0811_eaec),
        (0x0811_eb4a, 0x0811_eb5a),
        (0x0811_eb84, 0x0811_eb8e),
    ]
    .into_iter()
    .flat_map(|(a, b)| (a..b).step_by(2))
    .map(|addr| Block {
        addr,
        thumb: true,
        instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    })
    .collect();
    let object = root.join("compact-primary-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_compiled, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for case in 0..5 {
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            let stop = setup(&mut raw, case);
            assert_eq!(setup(&mut managed, case), stop);
            poison(&mut managed);
            let before = owner.compact_primary_alias_stats().unwrap();
            for _ in 0..32 {
                if raw.cpu.regs[15] == stop {
                    break;
                }
                raw.bus.frame_ready = false;
                raw.step();
            }
            assert_eq!(raw.cpu.regs[15], stop);
            for _ in 0..32 {
                if managed.cpu.regs[15] == stop {
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
            assert_eq!(managed.cpu.regs[15], stop);
            match case {
                0 => assert_eq!(managed.bus.read16(STACK + 0x18), 362),
                1 => {
                    assert_eq!(managed.cpu.regs[3], 0x0811_eaa9);
                    assert_eq!(managed.cpu.regs[2], 68);
                }
                2 => assert_eq!(managed.bus.read8(CONTEXT + 2), 255),
                3 => assert_eq!(
                    owner.compact_primary_alias_stats().unwrap().position_bytes
                        - before.position_bytes,
                    4
                ),
                4 => {
                    assert_eq!(managed.cpu.regs[1], 0);
                    assert_eq!(
                        owner.compact_primary_alias_stats().unwrap().selector_bytes
                            - before.selector_bytes,
                        1
                    );
                }
                _ => unreachable!(),
            }
            restore(&mut managed);
            assert_eq!(raw.bus.clock, managed.bus.clock);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "case {case}, compiled {native}"
            );
            cases += 1;
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[172,171], "resources":115, "prior_resources_preserved":old.resources.len(),
        "created_and_reused":true, "source_bytes":5, "mixed_boundaries_and_three_mirrors":true,
        "dependency_overlap_coherent_edit_and_failed_attach_rejected":true, "existing_compact_owner_preserved":true,
        "original_prefix_cases":cases, "full_state_clock_equal":true, "managed_sources_inverted_during_execution":true,
        "scope":"Conditional primary ID 362 and separately initialized callback prefixes; no normal-gameplay invocation or full callback ABI claim.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
