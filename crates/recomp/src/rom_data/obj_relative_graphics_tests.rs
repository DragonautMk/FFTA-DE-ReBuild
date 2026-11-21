//! The original kind-4 relative directory and its decoded custom streams.
use super::*;
use gba_core::{capi::RT_API, rom_data::obj_graphics as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x0800_0300;
const DESTINATION: u32 = 0x0202_0000;

fn spans(archive: &model::RelativeArchive) -> Vec<(usize, usize)> {
    std::iter::once((
        archive.source_offset as usize,
        archive.source_bytes().unwrap().len(),
    ))
    .chain(
        archive
            .banks
            .iter()
            .map(|bank| (bank.source_offset as usize, bank.encoded_source.len())),
    )
    .collect()
}

fn backing(machine: &mut Machine, rom: &[u8], spans: &[(usize, usize)], altered: bool) {
    for &(start, length) in spans {
        if altered {
            for at in start..start + length {
                machine.bus.rom[at] = !rom[at];
            }
        } else {
            machine.bus.rom[start..start + length].copy_from_slice(&rom[start..start + length]);
        }
    }
}

fn setup(machine: &mut Machine, entry: u32, first: u32, second: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = first;
    machine.cpu.regs[1] = second;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
}

fn interpret(machine: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..2_000_000 {
        if machine.cpu.regs[15] == RETURN {
            return;
        }
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original reader did not return");
}

#[test]
#[ignore = "exact EUR ROM and E202 predecessor; Wine only, Linux/USA pending"]
fn european_relative_graphics_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_RELATIVE_GRAPHICS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_RELATIVE_GRAPHICS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let old_manifest: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old_manifest.extractor_revision,
            old_manifest.decoder_revision,
            old_manifest.resources.len()
        ),
        (202, 201, 123)
    );
    let old =
        parse_obj_graphics(&fs::read(previous.join(OBJ_CUSTOM_GRAPHICS_FILE)).unwrap()).unwrap();
    assert!(old.relative_archive.is_none());

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
        (203, 202, 123)
    );
    for prior in &old_manifest.resources {
        if prior.id == OBJ_CUSTOM_GRAPHICS_ID {
            continue;
        }
        let current = source
            .manifest
            .resources
            .iter()
            .find(|item| item.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&prior.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let graphics = &source.obj_graphics.as_ref().unwrap().decoded;
    let mut without = graphics.clone();
    let archive = without.relative_archive.take().unwrap();
    assert_eq!(without, old);
    assert_eq!(archive.banks.len(), model::RELATIVE_ARCHIVE_COUNT);
    assert_eq!(archive.source_bytes().unwrap(), rom[0x3d63d4..0x3d6574]);
    assert_eq!(
        archive
            .banks
            .iter()
            .map(|b| b.encoded_source.len())
            .sum::<usize>(),
        224_516
    );
    assert_eq!(
        archive
            .banks
            .iter()
            .map(|b| b.words.len() * 4)
            .sum::<usize>(),
        353_536
    );
    assert_eq!(
        archive.banks.last().unwrap().source_offset as usize
            + archive.banks.last().unwrap().encoded_source.len(),
        0x40d300
    );
    for bank in &archive.banks {
        let at = bank.source_offset as usize;
        assert_eq!(bank.encoded_source, rom[at..at + bank.encoded_source.len()]);
    }
    let current = source
        .manifest
        .resources
        .iter()
        .find(|item| item.id == OBJ_CUSTOM_GRAPHICS_ID)
        .unwrap();
    let prior = old_manifest
        .resources
        .iter()
        .find(|item| item.id == OBJ_CUSTOM_GRAPHICS_ID)
        .unwrap();
    assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 104);
    assert_eq!(
        parse_obj_graphics(&fs::read(prepared.directory.join(OBJ_CUSTOM_GRAPHICS_FILE)).unwrap())
            .unwrap(),
        *graphics
    );
    let mut malformed = graphics.clone();
    malformed.relative_archive.as_mut().unwrap().banks.pop();
    assert!(malformed.validate().is_err());
    let mut malformed = graphics.clone();
    malformed.relative_archive.as_mut().unwrap().kind = 2;
    assert!(malformed.validate().is_err());
    let mut forged = graphics.clone();
    forged.relative_archive.as_mut().unwrap().reserved ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let ranges = spans(&archive);
    backing(&mut managed, &rom, &ranges, true);
    for &(start, len) in &ranges {
        for at in start..start + len {
            assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at]);
        }
        for edge in [start, start + len] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                    assert_eq!(
                        (RT_API.read16)(ptr, mirror + at as u32),
                        u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                    );
                }
            }
        }
    }
    // The final new stream touches an earlier managed battle-list directory.
    for at in 0x40d300..0x40d304 {
        managed.bus.rom[at] = !rom[at];
    }
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!(
        (RT_API.read32)(ptr, 0x0840d2fe),
        u32::from_le_bytes(rom[0x40d2fe..0x40d302].try_into().unwrap())
    );
    managed.bus.rom[0x40d300..0x40d304].copy_from_slice(&rom[0x40d300..0x40d304]);
    backing(&mut managed, &rom, &ranges, false);

    let initial = raw.save_state().unwrap();
    let selectors = [0u32, 69, 71, 73, 102, 103, 65535, 0x1_0045];
    let mut states = Vec::new();
    let mut visited = BTreeSet::new();
    for &selector in &selectors {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        for machine in [&mut raw, &mut managed] {
            setup(
                machine,
                0x0800_720c,
                0x0800_0000 + archive.source_offset,
                selector,
            );
        }
        backing(&mut managed, &rom, &ranges, true);
        interpret(&mut raw, &mut visited);
        interpret(&mut managed, &mut BTreeSet::new());
        let index = selector as u16 as usize;
        let expected = if index < archive.banks.len() {
            0x0800_0000 + archive.banks[index].source_offset
        } else {
            0
        };
        assert_eq!(raw.cpu.regs[0], expected);
        backing(&mut managed, &rom, &ranges, false);
        let state = raw.save_state().unwrap();
        assert_eq!(
            managed.save_state().unwrap(),
            state,
            "interpreter directory selector {selector:x}"
        );
        states.push(state);
    }

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let mut instrs = vec![if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            }];
            if thumb {
                let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
                if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                    instrs.push(next);
                }
            }
            Block {
                addr: address,
                thumb,
                instrs,
            }
        })
        .collect();
    let object = root.join("relative-graphics-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for (&selector, expected) in selectors.iter().zip(&states) {
        managed.load_state(&initial).unwrap();
        setup(
            &mut managed,
            0x0800_720c,
            0x0800_0000 + archive.source_offset,
            selector,
        );
        backing(&mut managed, &rom, &ranges, true);
        for _ in 0..200 {
            if managed.cpu.regs[15] == RETURN {
                break;
            }
            managed.bus.frame_ready = false;
            table
                .get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb()))
                .unwrap()(
                &RT_API,
                &mut managed as *mut Machine as *mut std::ffi::c_void,
            );
        }
        assert_eq!(managed.cpu.regs[15], RETURN);
        backing(&mut managed, &rom, &ranges, false);
        assert_eq!(
            &managed.save_state().unwrap(),
            expected,
            "native directory selector {selector:x}"
        );
    }

    for &index in &[69usize, 71, 73] {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        for machine in [&mut raw, &mut managed] {
            setup(
                machine,
                0x0800_54a0,
                DESTINATION,
                0x0800_0000 + archive.banks[index].source_offset,
            );
        }
        backing(&mut managed, &rom, &ranges, true);
        interpret(&mut raw, &mut BTreeSet::new());
        interpret(&mut managed, &mut BTreeSet::new());
        let decoded = archive.banks[index].decoded_bytes();
        let at = (DESTINATION - 0x0200_0000) as usize;
        assert_eq!(&raw.bus.ewram[at..at + decoded.len()], decoded);
        backing(&mut managed, &rom, &ranges, false);
        assert_eq!(
            managed.save_state().unwrap(),
            raw.save_state().unwrap(),
            "original decoder {index}"
        );
    }

    let snapshot = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&snapshot).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    let stats = owner.obj_graphics_stats().unwrap();
    assert!(
        stats.relative_directory_bytes > 0
            && stats.relative_bank_bytes > 0
            && stats.relative_literal_bytes > 0
    );
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[203,202], "entries":103, "directory_bytes":416,
        "compressed_bytes":224516, "decoded_bytes":353536, "prior_resources_preserved":122,
        "all_source_bytes_and_boundaries":true, "original_interpreter_native_selector_cases":selectors.len(),
        "original_decoder_cases":3, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
