//! EUR Star Cross rows, linked sources and original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::{observed_tilemap as map, observed_tilemap_descriptor as descriptor, observed_tilemap_row as row}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, length) in super::validation_tests::star_cross_transfer_ranges() {
        for at in start..start + length { machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] }; }
    }
}

#[test]
#[ignore = "exact EUR ROM and E297 predecessor; Wine only, Linux/USA pending"]
fn european_star_cross_transfer_sources_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STAR_CROSS_TRANSFER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_STAR_CROSS_TRANSFER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (297, 296, 148));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (298, 297, 148));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if [OBSERVED_TILEMAP_ID, OBSERVED_TILEMAP_DESCRIPTOR_ID, OBSERVED_TILEMAP_ROW_ID].contains(&prior.id.as_str()) {
            let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data: serde_json::Value = serde_json::from_slice(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            let keys: &[&str] = match prior.id.as_str() {
                OBSERVED_TILEMAP_ID => &["star_cross_transfer_sources"],
                OBSERVED_TILEMAP_DESCRIPTOR_ID => &["star_cross_lz_descriptors", "star_cross_mode7_descriptor"],
                _ => &["star_cross_rows"],
            };
            for key in keys { assert!(new_data.as_object_mut().unwrap().remove(*key).is_some()); }
            assert_eq!(new_data, old_data, "prior fields for {}", prior.id);
            let (count, bytes) = match prior.id.as_str() {
                OBSERVED_TILEMAP_ID => (4, 26), OBSERVED_TILEMAP_DESCRIPTOR_ID => (5, 32), _ => (4, 48),
            };
            assert_eq!(current.sources.len(), prior.sources.len() + count);
            assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
            assert_eq!(current.sources[prior.sources.len()..].iter().map(|s| s.length).sum::<u32>(), bytes);
            let mut metadata = current.clone(); metadata.sources.truncate(prior.sources.len());
            metadata.format = prior.format.clone(); metadata.sha256 = prior.sha256.clone(); assert_eq!(&metadata, prior);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(previous.join(&prior.path)).unwrap(), fs::read(prepared.directory.join(&current.path)).unwrap());
        }
    }
    let map_data = &source.observed_tilemap.as_ref().unwrap().decoded;
    let desc = &source.observed_tilemap_descriptor.as_ref().unwrap().decoded;
    let rows = &source.observed_tilemap_row.as_ref().unwrap().decoded;
    desc.validate_dependencies(map_data).unwrap(); desc.validate_lz_dependencies(&source.observed_lz.as_ref().unwrap().decoded).unwrap();
    rows.validate_dependencies(desc).unwrap(); rows.validate_tilemap_dependency(map_data).unwrap();
    let mut bad = map_data.clone(); bad.star_cross_transfer_sources.as_mut().unwrap().entry.rectangle_pointer += 2; assert!(bad.validate().is_err());
    let mut bad = desc.clone(); bad.star_cross_lz_descriptors.as_mut().unwrap()[0].wrapper.flags &= !2; assert!(bad.validate().is_err());
    let mut bad = rows.clone(); bad.star_cross_rows.as_mut().unwrap()[3].descriptor_pointer += 4; assert!(bad.validate().is_err());
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut forged = map_data.clone(); forged.star_cross_transfer_sources.as_mut().unwrap().mode7.first_run.value ^= 1;
    let bytes = encode_observed_tilemap(&forged).unwrap(); let mut altered = dataset.manifest.clone();
    altered.resources.iter_mut().find(|e| e.id == OBSERVED_TILEMAP_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.observed_tilemap = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent altered map and hash must be rejected");
    let mut forged = desc.clone(); forged.star_cross_lz_descriptors.as_mut().unwrap()[0].parameter16 ^= 1;
    let bytes = encode_observed_tilemap_descriptor(&forged).unwrap(); let mut altered = dataset.manifest.clone();
    altered.resources.iter_mut().find(|e| e.id == OBSERVED_TILEMAP_DESCRIPTOR_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.observed_tilemap_descriptor = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent altered descriptor and hash must be rejected");
    let mut forged = rows.clone(); forged.star_cross_rows.as_mut().unwrap()[0].field8 ^= 1;
    let bytes = encode_observed_tilemap_row(&forged).unwrap(); let mut altered = dataset.manifest.clone();
    altered.resources.iter_mut().find(|e| e.id == OBSERVED_TILEMAP_ROW_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.observed_tilemap_row = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent altered row and hash must be rejected"); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap(); let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap(); set_backing(&mut managed, &rom, true);
    let spans = super::validation_tests::star_cross_transfer_ranges(); assert_eq!(spans.iter().map(|(_, n)| n).sum::<usize>(), 106);
    for &(start, len) in &spans { for at in start..start + len {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    } }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    let edges: Vec<_> = spans.iter().flat_map(|&(start, len)| [start, start + len]).chain([0x39dda0, 0x39dda8, 0x39ddb0, 0x39ddb8, 0x39ddc0, 0x39ddcc, 0x39ddd8, 0x39dde4]).collect();
    for &edge in &edges { for at in edge.saturating_sub(3)..edge + 3 {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } }
    assert!(changed.contains(&0xaf204a), "mode-7 bytes after the selected prefix must remain ROM-backed");
    for edge in edges { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
    } }
    for at in changed { managed.bus.rom[at] = rom[at]; } set_backing(&mut managed, &rom, false);

    let cases: &[(u32, usize, u32)] = &[
        (0x080d_b388, 1, 0x0839_ddc0), (0x080d_b38e, 1, 0x0839_ddc0),
        (0x080d_b392, 6, 0x0839_dda0), (0x080d_b39a, 6, 0x0839_dda0),
        (0x080d_b39e, 4, 0x0839_dd98), (0x080d_b3a0, 4, 0x0839_dd98),
        (0x080d_8136, 7, 0x08ae_6e14),
        (0x080d_b2ba, 1, 0x0839_dde4), (0x080d_b316, 0, 0x0839_dde4),
        (0x080d_b318, 3, 0x0839_ddb8), (0x080d_b31c, 3, 0x0839_ddb8),
        (0x080d_a560, 7, 0x08af_2040), (0x080d_a75c, 7, 0x08af_2040),
        (0x080d_a15a, 2, 0x08af_2046), (0x080d_a182, 4, 0x08af_2046),
        (0x080d_b338, 0, 0x0839_ddcc), (0x080d_b338, 0, 0x0839_ddd8),
        (0x080d_b33a, 2, 0x0839_dda8), (0x080d_b33a, 2, 0x0839_ddb0),
        (0x080d_b14e, 5, 0x08af_fc6c), (0x080d_b14e, 5, 0x08b0_0d98),
    ];
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let mut addresses: BTreeSet<_> = cases.iter().map(|case| case.0).collect();
    addresses.extend((0x080e_ed4e..0x080e_ed5a).step_by(2));
    let blocks = addresses.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("star-cross-transfer-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut completed = 0;
    for native in [false, true] { for &(pc, register, address) in cases {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
            machine.cpu.regs[register] = address; machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[15] = pc; machine.cpu.cpsr = 0xbf;
        }
        set_backing(&mut managed, &rom, true); managed.bus.frame_ready = false;
        if native { table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
        else { managed.step(); }
        let stop = if native && matches!(pc, 0x080d_b39e | 0x080d_b316 | 0x080d_b338) { pc + 4 } else { pc + 2 };
        assert_eq!(managed.cpu.regs[15], stop, "unexpected generated reader extent at {pc:08x}");
        for _ in 0..2 { if raw.cpu.regs[15] == stop { break; } raw.bus.frame_ready = false; raw.step(); }
        assert_eq!(raw.cpu.regs[15], stop);
        set_backing(&mut managed, &rom, false);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "state differs at pc={pc:08x} native={native}"); completed += 1;
    }
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
            machine.cpu.regs[0] = 0; machine.cpu.regs[2] = 0x1234; machine.cpu.regs[6] = 0x5678;
            machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[15] = 0x080e_ed4e; machine.cpu.cpsr = 0xbf;
        }
        for at in 0xeed64..0xeed68 { managed.bus.rom[at] = !rom[at]; }
        for (machine, generated) in [(&mut raw, false), (&mut managed, native)] {
            let mut steps = 0;
            while machine.cpu.regs[15] != 0x080e_ed5a {
                assert!(steps < 12); machine.bus.frame_ready = false;
                if generated { table.get(machine.cpu.regs[15] | 1).unwrap()(&RT_API, machine as *mut Machine as *mut std::ffi::c_void); }
                else { machine.step(); } steps += 1;
            }
            assert_eq!(machine.cpu.regs[1..4], [0x0839_ddc0, 4, 4]);
        }
        managed.bus.rom[0xeed64..0xeed68].copy_from_slice(&rom[0xeed64..0xeed68]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "caller native={native}"); completed += 1;
    }
    assert_eq!(completed, 44);
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[298,297], "new_source_bytes":106, "prior_other_resources_unchanged":145,
        "prior_fields_and_dependencies_preserved":true, "prefix_end_remains_rom_backed":true,
        "poisoned_mirrors_and_mixed_boundaries":true, "original_interpreter_generated_cases":completed,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
