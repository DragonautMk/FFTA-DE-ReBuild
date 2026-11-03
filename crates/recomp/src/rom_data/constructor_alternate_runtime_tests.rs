//! Four alternate records/base-pointer ownership while original constructor work runs.
use super::default48_tests::{run, setup, Case};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::constructor_alternates::{END_OFFSET, POINTER_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_CONSTRUCTOR_ALTERNATES_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 25] {
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
        DEFAULT48_FILE,
        CONSTRUCTOR_ALTERNATES_FILE,
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn range(m: &mut Machine, rom: Option<&[u8]>) {
    for (a, z) in [
        (POINTER_OFFSET, POINTER_OFFSET + 4),
        (SOURCE_OFFSET, END_OFFSET),
    ] {
        let span = &mut m.bus.rom[a as usize..z as usize];
        if let Some(rom) = rom {
            span.copy_from_slice(&rom[a as usize..z as usize]);
        } else {
            span.fill(0xcd);
        }
    }
}
#[test]
#[ignore = "private EUR e29 dataset; constructor_alternates publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_constructor_alternates_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..24]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e29").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (29, 28));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (30, 29)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 25);
    assert_eq!(source.manifest.resources.len(), 24);
    for i in 1..24 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_constructor_alternates(&files[24]).unwrap();
    assert_eq!(items.source_bytes().len(), 192);
    assert_eq!(source.manifest.resources[23].sources.len(), 2);
    assert_eq!(source.manifest.resources[23].sources[0].length, 4);
    assert_eq!(
        files[24],
        fs::read(std::env::var_os("FFTA_STATE_TEST_CONSTRUCTOR_ALTERNATES_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert_eq!(items.records.len(), 4);
    assert_eq!(source.manifest.resources[23].sources[1].length, 192);
    let item_owner = read_dataset(&dataset()).unwrap().items.unwrap();
    items.validate_items(&item_owner).unwrap();
    let mut bad_item = items.clone();
    bad_item.records[3].halfwords08_28[4] = item_owner.records.len() as u16;
    assert!(bad_item.validate_items(&item_owner).is_err());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(CONSTRUCTOR_ALTERNATES_FILE)
    {
        Err("injected item publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e29-d28/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e30-d29"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(CONSTRUCTOR_ALTERNATES_FILE),
        b"partial",
    )
    .unwrap();
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
    for case in 0..10 {
        let mut forged = items.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.records[0].byte00 ^= 1,
            1 => forged.records[1].bytes02_04[1] = 127,
            2 => forged.records[2].flags2a ^= 1,
            3 => forged.records[3].halfwords2c_2e[1] ^= 0x8000,
            4 => manifest.resources[23].sources[0].length -= 1,
            5 => manifest.resources[23].dependencies.clear(),
            6 => manifest.resources[23].path = "../constructor-alternate-templates.json".into(),
            7 => manifest.resources[23].sources[1].role = "constructor-alternates/unknown".into(),
            8 => manifest.resources[23].sources[0].role = "constructor-alternates/unknown".into(),
            _ => {
                let a = forged.records[0].byte01;
                forged.records[0].byte01 = forged.records[1].byte01;
                forged.records[1].byte01 = a;
            }
        }
        forged.validate().unwrap();
        let bytes = encode_constructor_alternates(&forged).unwrap();
        manifest.resources[23].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(CONSTRUCTOR_ALTERNATES_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(CONSTRUCTOR_ALTERNATES_FILE), &files[24]).unwrap();
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
    for at in (SOURCE_OFFSET..END_OFFSET).chain(POINTER_OFFSET..POINTER_OFFSET + 4) {
        assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
    }
    let s = owner.constructor_alternates_stats().unwrap();
    assert!(s.pointer_bytes > 0 && s.header_bytes > 0 && s.halfword_bytes > 0 && s.flag_bytes > 0);
    assert_eq!(s.record_mask, 15);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,30,29],"files":25,"resources":24,"records":4,"pointer_bytes":4,"record_bytes":192,"source_bytes":196,"all_twenty_three_prior_decoded_files_and_e29_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_header_opaque_flag_tail_record_swap_pointer_source_dependency_path_role_transactional_guards":true,"parsed_item_owner_equipment_bound":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;6]>=const{Cell::new([0;6])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 6] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .constructor_alternates_stats()
            .unwrap()
    };
    [
        s.pointer_bytes,
        s.header_bytes,
        s.argument_bytes,
        s.halfword_bytes,
        s.flag_bytes,
        s.tail_bytes,
    ]
}
fn count(before: [u64; 6], after: [u64; 6]) {
    COMPILED.with(|v| {
        let mut n = v.get();
        for i in 0..6 {
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
fn stop(case: Case) -> u32 {
    case.stop()
}
fn cases() -> Vec<Case> {
    [
        (20, 0, 0, 1),
        (20, 1, 1, 2),
        (21, 2, 2, 1),
        (21, 3, 255, 2),
        (22, 0, 1, 1),
        (22, 255, 2, 2),
        (23, 1, 255, 1),
        (23, 3, 2, 2),
        (0xffff0014, 0x103, 0x102, 0x102),
        (0, 1, 2, 1),
        (19, 2, 0, 2),
        (24, 3, 255, 1),
        (20, 1, 2, 0),
        (23, 3, 1, 3),
    ]
    .into_iter()
    .map(|(id, flag, selector, mode)| Case {
        args: [id, 0x100ab, 0xffff00cd, 0x100ef],
        flag,
        selector,
        mode,
    })
    .collect()
}
// Assertions that read the raw record bytes run only after guest execution ends
// and the two ROM spans have been restored. Guest execution sees both spans poisoned.
fn execute(m: &mut Machine, case: Case) {
    setup(m, case);
    let mut n = 0;
    while m.cpu.regs[15] != case.stop() {
        assert!(n < 2000);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
}
#[test]
#[ignore = "private EUR new constructor_alternates ownership/native constructor prefix proof; Wine only, Linux/USA pending"]
fn european_constructor_alternates_native_pointer_four_records_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.default48.as_ref().unwrap();
    let mut tracer = crate::make_machine(rom.clone(), None);
    super::ui_palette_tests::initialize(&mut tracer, &mut BTreeSet::new());
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
        iwram: Some(&tracer.bus.iwram),
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
    let object = out.join("constructor_alternates.o");
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
    COMPILED.with(|v| v.set([0; 6]));
    let mut dispatches = 0;
    let mut separate_paths = 0;
    for &case in &cases {
        let before_branch = decoded
            .bus
            .rom_resources()
            .unwrap()
            .constructor_alternates_stats()
            .unwrap()
            .source_bytes;
        let before_default = decoded
            .bus
            .rom_resources()
            .unwrap()
            .default48_stats()
            .unwrap()
            .source_bytes;
        run(&mut legacy, case, data, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        execute(&mut decoded, case);
        setup(&mut native, case);
        let end = stop(case);
        while native.cpu.regs[15] != end {
            assert!(dispatches < 100000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original constructor_alternates instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        if gba_core::rom_data::default48::Template::default_id(case.args[0])
            || case.stop() == super::default48_tests::RETURN
        {
            assert_eq!(
                decoded
                    .bus
                    .rom_resources()
                    .unwrap()
                    .constructor_alternates_stats()
                    .unwrap()
                    .source_bytes,
                before_branch
            );
            separate_paths += 1;
            if case.stop() != super::default48_tests::RETURN {
                assert!(
                    decoded
                        .bus
                        .rom_resources()
                        .unwrap()
                        .default48_stats()
                        .unwrap()
                        .source_bytes
                        > before_default
                );
            }
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            super::default48_tests::check(m, case, data);
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "constructor_alternates full state/clock {case:?}"
            );
        }
    }
    let typed = native
        .bus
        .rom_resources()
        .unwrap()
        .constructor_alternates_stats()
        .unwrap();
    assert_eq!(typed.record_mask, 15);
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&n| n > 0));
    range(&mut decoded, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in (SOURCE_OFFSET..END_OFFSET).chain(POINTER_OFFSET..POINTER_OFFSET + 4) {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        let edges = (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 3)
            .chain(POINTER_OFFSET - 3..POINTER_OFFSET + 7)
            .chain([
                SOURCE_OFFSET + 47,
                SOURCE_OFFSET + 48,
                SOURCE_OFFSET + 95,
                SOURCE_OFFSET + 96,
                SOURCE_OFFSET + 143,
                SOURCE_OFFSET + 144,
            ]);
        for at in edges {
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
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before = owner.constructor_alternates_stats().unwrap();
    for at in [POINTER_OFFSET - 2, END_OFFSET - 2] {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let after = owner.constructor_alternates_stats().unwrap();
    assert_eq!(after.pointer_bytes - before.pointer_bytes, 2);
    assert_eq!(after.source_bytes - before.source_bytes, 4);
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for source_at in [POINTER_OFFSET, SOURCE_OFFSET] {
        for swi in [0xdf0b, 0xdf0c] {
            for m in [&mut legacy, &mut decoded] {
                m.load_state(&start).unwrap();
                m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + source_at, 0x02032400, 8]);
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
    }
    for source_at in [POINTER_OFFSET, SOURCE_OFFSET + 2] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.bus.write32(0x040000d4, 0x08000000 + source_at);
            m.bus.write32(0x040000d8, 0x02032000);
            m.bus.write16(0x040000dc, 2);
        }
        range(&mut decoded, None);
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .constructor_alternates_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut legacy, &mut decoded] {
            m.bus.write16(0x040000de, 0x8000);
        }
        let after = decoded
            .bus
            .rom_resources()
            .unwrap()
            .constructor_alternates_stats()
            .unwrap()
            .source_bytes;
        assert_eq!(after - before, 4);
        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions":instructions,"native_dispatches":dispatches,"compiled_pointer_header_argument_halfword_flag_tail_bytes":compiled,"record_mask":typed.record_mask,"poisoned_source_bytes":196,"original_pointer_index_iwram_copy_common_patch_prefix":true,"all_four_alternate_ids_u8truncation_selectors_live_args_flags_modes":true,"default_or_rejected_paths_zero_new_source_reads":separate_paths,"full_states_clocks_equal":true,"all196bytes_edges_record_boundaries_widths_mirrors_real_bios_bus":true,"both_source_spans_cpuset_cpufastset_dma4bytes_each":true,"raw_header_and_following_source_crossing_typed_bytes":[2,2],"allocation_tail":"unclaimed, original code remains","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
