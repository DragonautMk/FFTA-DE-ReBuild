//! Stored key/value ownership while original direct/inverse and live byte reader run.
use super::metadata2_tests::{run, setup, stop, Case};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::metadata2::{END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA2_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 23] {
    [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
        BG_MAIN_GRAPHICS_FILE,
        BG_TRANSFER_PROGRAMS_FILE,
        SHORT_TEXTS_FILE,
        TEXT_CATALOGS_FILE,
        FONT_BANK_FILE,
        NAME_KEYBOARD_FILE,
        OBJ_STATIC_LISTS_FILE,
        OBJ_ANIMATION_BANKS_FILE,
        OBJ_CUSTOM_GRAPHICS_FILE,
        UI_RAW_PALETTES_FILE,
        UI_CUSTOM_GRAPHICS_FILE,
        ITEM_RECORDS_FILE,
        ITEM_LINKS_FILE,
        METADATA16_FILE,
        METADATA52_FILE,
        METADATA4_FILE,
        METADATA2_FILE,
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn range(m: &mut Machine, rom: Option<&[u8]>) {
    let span = &mut m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
    if let Some(rom) = rom {
        span.copy_from_slice(&rom[SOURCE_OFFSET as usize..END_OFFSET as usize]);
    } else {
        span.fill(0xcd);
    }
}
#[test]
#[ignore = "private EUR e27 dataset; metadata2 publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_metadata2_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..22]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e27").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (27, 26));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (28, 27)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 23);
    assert_eq!(source.manifest.resources.len(), 22);
    for i in 1..22 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_metadata2(&files[22]).unwrap();
    assert_eq!(items.records.len(), 44);
    assert_eq!(source.manifest.resources[21].sources.len(), 1);
    assert_eq!(source.manifest.resources[21].sources[0].length, 88);
    assert_eq!(
        files[22],
        fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA2_SOURCE_FILE").unwrap()).unwrap()
    );
    let actual_owner = read_dataset(&dataset()).unwrap().metadata52.unwrap();
    items.validate_metadata52(&actual_owner).unwrap();
    let mut bad_key = items.clone();
    bad_key.records[43].key = 116;
    assert!(bad_key.validate_metadata52(&actual_owner).is_err());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(
        &interrupted,
        &mut source,
        &mut |p, b| if p.ends_with(METADATA2_FILE) {
            Err("injected item publication failure".into())
        } else {
            write_synced(p, b)
        }
    )
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e27-d26/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in names().iter().zip(&old) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let prepared = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e28-d27"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(METADATA2_FILE), b"partial").unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    let relative = repaired
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = r.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    assert_eq!(
        preparation::prepare(&moved, &mut source, &mut |_| Ok(()))
            .unwrap()
            .directory,
        moved.join(relative)
    );
    for (n, b) in names().iter().zip(&old) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(prior_dir.join(n)).unwrap(), *b);
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    for case in 0..8 {
        let mut forged = items.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.records[1].key ^= 1,
            1 => forged.records[0].value = 127,
            2 => forged.records[43].value ^= 1,
            3 => forged.records.swap(1, 2),
            4 => manifest.resources[21].sources[0].length -= 1,
            5 => manifest.resources[21].dependencies.clear(),
            6 => manifest.resources[21].path = "../linked-metadata2.json".into(),
            _ => manifest.resources[21].sources[0].role = "metadata2/unknown".into(),
        }
        forged.validate().unwrap();
        let bytes = encode_metadata2(&forged).unwrap();
        manifest.resources[21].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(METADATA2_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(METADATA2_FILE), &files[22]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("copy");
    fs::create_dir(&copy).unwrap();
    for n in names() {
        fs::copy(dataset().join(n), copy.join(n)).unwrap();
    }
    attach_from_directory(&mut m, &copy).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    fs::rename(copy, temp.path().join("unavailable")).unwrap();
    let save = temp.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 99;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    slots.undo(&mut m).unwrap();
    assert_eq!(m.save_state().unwrap(), changed);
    let len = m.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; len]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups
        .restore(backups.choices(len)[0].generation.unwrap(), &mut m, &boot)
        .unwrap();
    m.bus.load_save_data(&vec![9; len]);
    crate::save_backups::restart(&mut m, &boot, &save).unwrap();
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    range(&mut m, None);
    for at in SOURCE_OFFSET..END_OFFSET {
        assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
    }
    let s = owner.metadata2_stats().unwrap();
    assert!(s.source_bytes > 0 && s.key_bytes > 0 && s.value_bytes > 0);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,28,27],"files":23,"resources":22,"records":44,"source_bytes":88,"all_twenty_one_prior_decoded_files_and_e27_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_key_value_lastrow_reorder_source_dependency_path_role_transactional_guards":true,"parsed_metadata52_stored_key_owner_bounds":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 2] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .metadata2_stats()
            .unwrap()
    };
    [s.key_bytes, s.value_bytes]
}
fn count(before: [u64; 2], after: [u64; 2]) {
    COMPILED.with(|v| {
        let mut n = v.get();
        for i in 0..2 {
            n[i] += after[i] - before[i];
        }
        v.set(n);
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = used(m);
    let v = (RT_API.read8)(m, a);
    count(b, used(m));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = used(m);
    let v = (RT_API.read16)(m, a);
    count(b, used(m));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = used(m);
    let v = (RT_API.read32)(m, a);
    count(b, used(m));
    v
}
fn cases() -> Vec<Case> {
    vec![
        Case::Direct(0),
        Case::Direct(1),
        Case::Direct(7),
        Case::Direct(22),
        Case::Direct(42),
        Case::Direct(43),
        Case::Direct(0x10000),
        Case::Direct(0xffff002b),
        Case::Inverse(0),
        Case::Inverse(4),
        Case::Inverse(6),
        Case::Inverse(7),
        Case::Inverse(19),
        Case::Inverse(34),
        Case::Inverse(35),
        Case::Inverse(254),
        Case::Inverse(255),
        Case::Inverse(0x10004),
        Case::Live(0),
        Case::Live(7),
        Case::Live(43),
    ]
}
#[test]
#[ignore = "private EUR new metadata2 ownership/native direct/inverse/live proof; Wine only, Linux/USA pending"]
fn european_metadata2_native_direct_inverse_live_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.metadata2.as_ref().unwrap();
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let cases = cases();
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut tracer, case, data, &mut visited);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = visited
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
    assert!(cases.iter().all(|&case| {
        let end = stop(case);
        !visited.iter().any(|&(pc, _)| pc == end)
    }));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("metadata2.o");
    let loader = recomp_core::tcc_memory::library_path().unwrap();
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &loader, &object, &mut |_| {})
        .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    for m in [&mut decoded, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    for m in [&mut legacy, &mut decoded, &mut native] {
        m.load_state(&start).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|v| v.set([0; 2]));
    let mut dispatches = 0;
    for &case in &cases {
        run(&mut legacy, case, data, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        run(&mut decoded, case, data, &mut BTreeSet::new());
        setup(&mut native, case);
        let end = stop(case);
        while native.cpu.regs[15] != end {
            assert!(dispatches < 100000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original metadata2 instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "metadata2 full state/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&n| n > 0));
    range(&mut decoded, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in SOURCE_OFFSET..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        for at in (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 3)
            .chain([SOURCE_OFFSET + 3, SOURCE_OFFSET + 4, SOURCE_OFFSET + 40])
        {
            for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                assert_eq!(decoded.bus.read8(mirror + at), rom[at as usize]);
                assert_eq!(
                    decoded.bus.read16(mirror + at),
                    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
                );
                assert_eq!(
                    (RT_API.read32)(
                        &mut decoded as *mut Machine as *mut std::ffi::c_void,
                        mirror + at
                    ),
                    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                );
            }
        }
    }
    // Adjacent prior metadata4 bytes are independently typed, including a crossing
    // read where both old and new raw owners are unavailable in the ROM buffer.
    decoded.bus.rom[SOURCE_OFFSET as usize - 4..SOURCE_OFFSET as usize].fill(0xcd);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before_old = owner.metadata4_stats().unwrap().source_bytes;
    let before_new = owner.metadata2_stats().unwrap().source_bytes;
    assert_eq!(
        decoded.bus.read32(0x08000000 + SOURCE_OFFSET - 2),
        u32::from_le_bytes(
            rom[SOURCE_OFFSET as usize - 2..SOURCE_OFFSET as usize + 2]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        owner.metadata4_stats().unwrap().source_bytes - before_old,
        2
    );
    assert_eq!(
        owner.metadata2_stats().unwrap().source_bytes - before_new,
        2
    );
    decoded.bus.rom[SOURCE_OFFSET as usize - 4..SOURCE_OFFSET as usize]
        .copy_from_slice(&rom[SOURCE_OFFSET as usize - 4..SOURCE_OFFSET as usize]);
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for swi in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + SOURCE_OFFSET, 0x02032400, 8]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, None);
        legacy.step();
        decoded.step();
        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + SOURCE_OFFSET + 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, None);
    let before = decoded
        .bus
        .rom_resources()
        .unwrap()
        .metadata2_stats()
        .unwrap()
        .source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let after = decoded
        .bus
        .rom_resources()
        .unwrap()
        .metadata2_stats()
        .unwrap()
        .source_bytes;
    assert_eq!(after - before, 4);
    range(&mut decoded, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions":instructions,"native_dispatches":dispatches,"compiled_key_bytes":compiled[0],"compiled_value_bytes":compiled[1],"poisoned_source_bytes":88,"direct_inverse_live_byte_prefix_original_instructions":true,"duplicate_first_stored_key_miss_zero_u8truncation_last43_forward":true,"full_states_clocks_equal":true,"all_bytes_edges_widths_mirrors_real_bios_bus_cpuset_cpufastset":true,"adjacent_old_new_typed_owner_crossing":[2,2],"dma_typed_source_bytes":4,"live_prefix":"after getter with caller frame retained; global direct index bounds unproved","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
