//! EUR White Flowers isolated projections and their original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::white_flowers_isolated_values as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, pc: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + pc;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E290 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_isolated_values_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_ISOLATED_VALUES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_ISOLATED_VALUES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (290, 289, 147));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (291, 290, 148));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let new = source.manifest.resources.iter().find(|entry| entry.id == WHITE_FLOWERS_ISOLATED_VALUES_ID).unwrap();
    assert_eq!(new.dependencies, vec![THUMB_PC_LITERALS_ID.to_string()]);
    assert_eq!(new.sources.len(), 2);
    assert_eq!((new.sources[0].offset, new.sources[0].length), (model::ROW_OFFSET, 40));
    assert_eq!((new.sources[1].offset, new.sources[1].length), (model::BYTE_OFFSET, 1));
    let data = &source.white_flowers_isolated_values.as_ref().unwrap().decoded;
    let start = model::ROW_OFFSET as usize;
    let end = start + model::ROW_LENGTH * 2;
    let byte = model::BYTE_OFFSET as usize;
    let spans = data.source_spans().unwrap();
    assert_eq!(spans[0].1, rom[start..end]);
    assert_eq!(spans[1].1, rom[byte..byte + 1]);
    assert_eq!(parse_white_flowers_isolated_values(&fs::read(prepared.directory.join(WHITE_FLOWERS_ISOLATED_VALUES_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    invalid.row.pop();
    assert!(invalid.validate().is_err());
    for which in 0..2 {
        let mut forged = data.clone();
        if which == 0 { forged.row[11] ^= 1; } else { forged.byte ^= 1; }
        forged.validate().unwrap();
        assert!(forged.verify_source(&rom).is_err());
        let bytes = encode_white_flowers_isolated_values(&forged).unwrap();
        let dataset = read_dataset(&prepared.directory).unwrap();
        let mut altered_manifest = dataset.manifest.clone();
        altered_manifest.resources.iter_mut().find(|entry| entry.id == WHITE_FLOWERS_ISOLATED_VALUES_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
        let mut files = dataset.files();
        files.manifest = &altered_manifest;
        files.white_flowers_isolated_values = Some((&forged, &bytes));
        assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    }

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    managed.bus.rom[byte] = !rom[byte];
    let before = owner.white_flowers_isolated_values_stats().unwrap();
    for at in (start..end).chain(std::iter::once(byte)) {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    }
    let after = owner.white_flowers_isolated_values_stats().unwrap();
    assert_eq!(after.row_source_bytes - before.row_source_bytes, 120);
    assert_eq!(after.byte_source_bytes - before.byte_source_bytes, 3);
    assert_eq!(after.row_values_mask, (1u64 << 20) - 1);
    let current_owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for edge in [start, end, byte, byte + 1] {
        for at in edge - 3..edge + 4 {
            if !current_owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
        }
    }
    for edge in [start, end, byte, byte + 1] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| if changed.contains(&(at+n)) { rom[at+n] ^ 0x5a } else { rom[at+n] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])), "read16 {at:x}");
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]), "read32 {at:x}");
            }
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    managed.bus.rom[byte] = rom[byte];
    for at in changed { managed.bus.rom[at] = rom[at]; }

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses = (0x0802_ecce..0x0802_ece6).step_by(2)
        .chain((0x0802_edc0..0x0802_edce).step_by(2))
        .chain((0x080c_f3fc..0x080c_f40c).step_by(2));
    let blocks = addresses.map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("white-flowers-isolated-values-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool, stop: u32| {
        let mut native_calls = 0;
        for _ in 0..128 {
            if machine.cpu.regs[15] == stop { break; }
            machine.bus.frame_ready = false;
            let pc = machine.cpu.regs[15];
            if native {
                if let Some(block) = table.get(pc | 1) {
                    block(&RT_API, machine as *mut Machine as *mut std::ffi::c_void);
                    native_calls += 1;
                    continue;
                }
            }
            machine.step();
        }
        assert_eq!(machine.cpu.regs[15], stop, "original reader did not reach its stop");
        if native { assert!(native_calls > 0, "generated reader must run"); }
    };
    let mut setup_cases = 0;
    let mut row_cases = 0;
    let mut byte_cases = 0;
    for native in [false, true] {
        for selector in [0u8, 1] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); setup(machine, 0x02ecce);
                machine.cpu.regs[0] = 11;
                machine.bus.write8(0x0200_1f6a, selector);
            }
            execute(&mut raw, false, 0x0802_ece6);
            execute(&mut managed, native, 0x0802_ece6);
            assert_eq!(managed.cpu.regs[8], 0x086e_8994 + 40 * u32::from(selector));
            assert_eq!(managed.cpu.regs[5], 11);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            setup_cases += 1;
        }
        for (selector, index) in [(0u32, 0u8), (0, 11), (0, 19), (0, 20), (0, 255), (1, 0)] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); setup(machine, 0x02edc0);
                machine.cpu.regs[8] = 0x086e_8994 + 40 * selector;
                machine.cpu.regs[5] = u32::from(index);
            }
            for at in start..end { managed.bus.rom[at] = !rom[at]; }
            let before = owner.white_flowers_isolated_values_stats().unwrap().row_source_bytes;
            execute(&mut raw, false, 0x0802_edce);
            execute(&mut managed, native, 0x0802_edce);
            let at = start + selector as usize * 40 + usize::from(index) * 2;
            let expected = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            assert_eq!(managed.cpu.regs[1], u32::from(expected));
            assert_eq!(managed.cpu.regs[5], u32::from(index.wrapping_add(1)));
            assert_eq!(owner.white_flowers_isolated_values_stats().unwrap().row_source_bytes - before, if selector == 0 && index < 20 { 2 } else { 0 });
            managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "selector={selector} index={index} native={native}");
            row_cases += 1;
        }
        for index in [0u32, 15, 16, 17, 65535, 0x10010] {
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, 0x0cf3fc); machine.cpu.regs[0] = index; }
            managed.bus.rom[byte] = !rom[byte];
            let before = owner.white_flowers_isolated_values_stats().unwrap().byte_source_bytes;
            execute(&mut raw, false, 0x0800_0300);
            execute(&mut managed, native, 0x0800_0300);
            let at = 0x3c9440usize + ((index & 0xffff) as usize) * 2;
            assert_eq!(managed.cpu.regs[0], u32::from(rom[at] >> 4));
            assert_eq!(owner.white_flowers_isolated_values_stats().unwrap().byte_source_bytes - before, u64::from((index & 0xffff) == 16));
            managed.bus.rom[byte] = rom[byte];
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "byte index={index} native={native}");
            byte_cases += 1;
        }
    }
    assert_eq!((setup_cases, row_cases, byte_cases), (4, 12, 12));
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[291,290], "resources":148, "new_source_bytes":41,
        "prior_resources_unchanged":147, "original_selector_setup_cases":setup_cases,
        "original_row_reader_cases":row_cases, "original_byte_reader_cases":byte_cases,
        "outside_projection_fallback":true, "poisoned_mixed_boundaries_and_mirrors":true,
        "full_state_clock_equal":true, "coherent_edits_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
