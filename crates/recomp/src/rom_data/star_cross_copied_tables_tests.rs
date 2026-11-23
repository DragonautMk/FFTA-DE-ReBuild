//! EUR Star Cross copied source groups and original copy paths.
use super::*;
use gba_core::{capi::RT_API, rom_data::copied_halfword_lookups as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn poison(machine: &mut Machine, rom: &[u8], enabled: bool) {
    let start = model::STAR_CROSS_OFFSET as usize;
    for at in start..start + model::STAR_CROSS_BYTES {
        machine.bus.rom[at] = if enabled { !rom[at] } else { rom[at] };
    }
}

#[test]
#[ignore = "exact EUR ROM and E294 predecessor; Wine only, Linux/USA pending"]
fn european_star_cross_copied_tables_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STAR_CROSS_COPIED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_STAR_CROSS_COPIED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (294, 293, 148));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (295, 294, 148));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|r| r.id == prior.id).unwrap();
        if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            let old_data = parse_copied_halfword_lookups(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_copied_halfword_lookups(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            assert!(old_data.star_cross_tables.is_none());
            assert!(new_data.star_cross_tables.take().is_some());
            assert_eq!(new_data, old_data);
            let mut metadata = current.clone();
            let added = metadata.sources.pop().unwrap();
            assert_eq!((added.offset, added.length), (model::STAR_CROSS_OFFSET, model::STAR_CROSS_BYTES as u32));
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(previous.join(&prior.path)).unwrap(), fs::read(prepared.directory.join(&current.path)).unwrap());
        }
    }
    let data = &source.copied_halfword_lookups.as_ref().unwrap().decoded;
    let new_span = data.source_spans().unwrap().into_iter().find(|(offset, _, _)| *offset == model::STAR_CROSS_OFFSET).unwrap();
    let start = model::STAR_CROSS_OFFSET as usize;
    let end = start + model::STAR_CROSS_BYTES;
    assert_eq!(new_span.1, rom[start..end]);
    assert_eq!(parse_copied_halfword_lookups(&fs::read(prepared.directory.join(COPIED_HALFWORD_LOOKUPS_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone();
    invalid.star_cross_tables.as_mut().unwrap().source_offset += 2;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.star_cross_tables.as_mut().unwrap().first_coordinates[0] ^= 1;
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_copied_halfword_lookups(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut manifest = dataset.manifest.clone();
    manifest.resources.iter_mut().find(|r| r.id == COPIED_HALFWORD_LOOKUPS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &manifest;
    files.copied_halfword_lookups = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent edited file must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    poison(&mut managed, &rom, true);
    let before = owner.copied_halfword_lookups_stats().unwrap();
    for at in start..end {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
        }
    }
    let after = owner.copied_halfword_lookups_stats().unwrap();
    assert_eq!(after.star_cross_bytes - before.star_cross_bytes, 180);
    assert_eq!(after.star_cross_regions_mask, 0xf);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|r| &r.sources)
        .any(|s| (s.offset as usize..(s.offset + s.length) as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for edge in [start, start + 16, start + 32, start + 36, end] {
        for at in edge.saturating_sub(3)..edge + 4 {
            if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
        }
    }
    for edge in [start, start + 16, start + 32, start + 36, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| if changed.contains(&(at + n)) { rom[at + n] ^ 0x5a } else { rom[at + n] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
            }
        }
    }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    poison(&mut managed, &rom, false);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses = (0x080e_eb0c..0x080e_eb18).step_by(2)
        .chain((0x080e_eb64..0x080e_eb70).step_by(2))
        .chain((0x0814_7e0c..0x0814_7e1a).step_by(2));
    let blocks = addresses.map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("star-cross-copied-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for (entry, stop, src, length, helper) in [
            (0x080e_eb0c, 0x080e_eb18, model::STAR_CROSS_OFFSET, 16usize, false),
            (0x080e_eb64, 0x080e_eb70, model::STAR_CROSS_OFFSET + 16, 16, false),
            (0x0814_7e0c, 0x0814_7e1a, model::STAR_CROSS_OFFSET + 32, 4, true),
            (0x0814_7e0c, 0x0814_7e1a, model::STAR_CROSS_OFFSET + 36, 24, true),
        ] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                machine.bus.ffta_sprites.enabled = false;
                machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                machine.cpu.regs[13] = 0x0300_7a00;
                machine.cpu.regs[14] = 0x0800_0301;
                machine.cpu.regs[15] = entry;
                machine.cpu.cpsr = 0xbf;
                if helper {
                    machine.cpu.regs[1] = u32::MAX;
                    machine.cpu.regs[2] = (length - 1) as u32;
                    machine.cpu.regs[3] = 0x0800_0000 + src;
                    machine.cpu.regs[4] = 0x0201_0000;
                }
            }
            poison(&mut managed, &rom, true);
            let before = owner.copied_halfword_lookups_stats().unwrap().star_cross_bytes;
            for _ in 0..512 {
                if raw.cpu.regs[15] == stop { break; }
                raw.bus.frame_ready = false; raw.step();
            }
            for _ in 0..512 {
                let pc = managed.cpu.regs[15];
                if pc == stop { break; }
                managed.bus.frame_ready = false;
                if native { table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
                else { managed.step(); }
            }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (stop, stop));
            assert_eq!(owner.copied_halfword_lookups_stats().unwrap().star_cross_bytes - before, length as u64);
            let actual = if helper { &managed.bus.ewram[0x10000..0x10000 + length] } else { &managed.bus.iwram[0x7a10..0x7a10 + length] };
            assert_eq!(actual, &rom[src as usize..src as usize + length]);
            poison(&mut managed, &rom, false);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "copy {entry:x} from {src:x}, native={native}");
            cases += 1;
        }
    }
    assert_eq!(cases, 8);

    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[295,294], "resources":148, "new_source_bytes":60,
        "prior_representation_preserved":true, "prior_other_resources_unchanged":147,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_copy_cases":cases,
        "full_state_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
