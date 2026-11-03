//! Progression ownership while original live state and price routines run.
use super::progression_reward_tests::{run, setup, Case};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::progression_rewards::{END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_REWARDS_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 30] {
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
        METADATA14_FILE,
        METADATA12_FILE,
        GLOBAL_PAIR_FLAGS_FILE,
        EVALUATION_PARAMETERS_FILE,
        PROGRESSION_REWARDS_FILE,
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn range(m: &mut Machine, rom: Option<&[u8]>) {
    let span = &mut m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
    if let Some(r) = rom {
        span.copy_from_slice(&r[SOURCE_OFFSET as usize..END_OFFSET as usize]);
    } else {
        for b in span {
            *b ^= 0x5a;
        }
    }
}
#[test]
#[ignore = "private EUR e34 dataset; progression_rewards publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_progression_rewards_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..29]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e34").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (34, 33));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (35, 34)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 30);
    assert_eq!(source.manifest.resources.len(), 29);
    for i in 1..29 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_progression_rewards(&files[29]).unwrap();
    assert_eq!(
        (items.records.len(), items.source_bytes().len()),
        (100, 1000)
    );
    assert_eq!(
        (
            source.manifest.resources[28].sources.len(),
            source.manifest.resources[28].sources[0].length
        ),
        (1, 1000)
    );
    assert_eq!(
        files[29],
        fs::read(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_REWARDS_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert!(source.manifest.resources[28].dependencies.is_empty());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(PROGRESSION_REWARDS_FILE)
    {
        Err("injected packed-pair publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e34-d33/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e35-d34"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(PROGRESSION_REWARDS_FILE),
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
            0 => forged.records[0].deltas00_07[0] ^= 1,
            1 => forged.records[99].deltas00_07[7] ^= 0x80,
            2 => forged.records[99].discount08 ^= 1,
            3 => forged.records[0].byte09 ^= 1,
            4 => manifest.resources[28].sources[0].length -= 1,
            5 => manifest.resources[28]
                .dependencies
                .push(ITEM_RECORDS_ID.into()),
            6 => manifest.resources[28].path = "../progression-rewards.json".into(),
            7 => manifest.resources[28].sources[0].role = "progression-rewards/unknown".into(),
            8 => manifest.resources[28].sources[0].offset += 1,
            _ => forged.records.swap(0, 99),
        }

        if case < 9 {
            forged.validate().unwrap();
        } else {
            assert!(forged.validate().is_err());
        }
        let bytes = encode_progression_rewards(&forged).unwrap();
        manifest.resources[28].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(PROGRESSION_REWARDS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(PROGRESSION_REWARDS_FILE), &files[29]).unwrap();
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
    for (start, end) in [(SOURCE_OFFSET, END_OFFSET)] {
        for at in start..end {
            assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
        }
    }
    let s = owner.progression_rewards_stats().unwrap();
    assert_eq!(
        (s.delta_bytes, s.discount_bytes, s.opaque_bytes),
        (800, 100, 100)
    );
    assert_eq!(s.record_mask, [u64::MAX, (1u64 << 36) - 1]);

    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,35,34],"files":30,"resources":29,"source_bytes":1000,"source_ranges":1,"all_twenty_eight_prior_decoded_files_and_e34_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_delta_discount_opaque_lastrow_reorder_source_dependency_path_role_guards":true,"no_invented_item_foreign_key_dependency":true,"unavailable_copy_load_undo_backup_restart":true,"all1000bytes_and100record_mask":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;3]>=const{Cell::new([0;3])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 3] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .progression_rewards_stats()
            .unwrap()
    };
    [s.delta_bytes, s.discount_bytes, s.opaque_bytes]
}
fn count(before: [u64; 3], after: [u64; 3]) {
    COMPILED.with(|v| {
        let mut n = v.get();
        for i in 0..3 {
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
fn cases(items: &Items) -> Vec<Case> {
    let flagged = items
        .records
        .iter()
        .position(|r| r.bytes_08_0f[4] & 8 != 0 && r.halfwords_00_06[2] > 0)
        .unwrap() as u32;
    let unflagged = items
        .records
        .iter()
        .position(|r| r.bytes_08_0f[4] & 8 == 0)
        .unwrap() as u32;
    let mut out = vec![
        Case::Publish {
            row: 0,
            default: true,
        },
        Case::Publish {
            row: 99,
            default: false,
        },
        Case::Publish {
            row: 255,
            default: false,
        },
    ];
    for index in 0..8 {
        for level in [17, 98] {
            out.push(Case::Delta {
                row: 99,
                index,
                level,
                progress: 95,
                growth: 8,
                flags: 0x8001,
            });
        }
    }
    out.push(Case::Delta {
        row: 0,
        index: 1,
        level: 255,
        progress: 250,
        growth: 255,
        flags: 0x4001,
    });
    for (row, mode, item) in [
        (0, 0, flagged),
        (99, 2, flagged),
        (99, 6, flagged),
        (99, 0x102, 0x10000 + flagged),
        (0, 255, unflagged),
        (100, 2, flagged),
        (255, 2, flagged),
    ] {
        out.push(Case::Price { row, mode, item });
    }
    out
}
#[test]
#[ignore = "private EUR typed progression/native/bus; two reduced Wine checks, Linux/USA pending"]
fn european_progression_rewards_native_live_delta_prices_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.progression_rewards.as_ref().unwrap();
    let items = files.items.as_ref().unwrap();
    let cases = cases(items);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut tracer, case, data, items, &rom, &mut visited);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = visited
        .iter()
        .map(|&(addr, thumb)| {
            let mut instrs = vec![armv4t::decode_thumb(view.read16(addr), addr)];
            let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                instrs.push(next);
            }
            Block {
                addr,
                thumb,
                instrs,
            }
        })
        .collect();
    assert!(cases
        .iter()
        .all(|c| !visited.iter().any(|&(pc, _)| pc == c.stop())));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("progression-rewards.o");
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
    COMPILED.with(|v| v.set([0; 3]));
    let mut dispatches = 0;
    let mut zero_paths = 0;
    for &case in &cases {
        run(&mut legacy, case, data, items, &rom, &mut BTreeSet::new());
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .progression_rewards_stats()
            .unwrap()
            .source_bytes;
        let native_before = native
            .bus
            .rom_resources()
            .unwrap()
            .progression_rewards_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        run(&mut decoded, case, data, items, &rom, &mut BTreeSet::new());
        setup(&mut native, case);
        while native.cpu.regs[15] != case.stop() {
            assert!(dispatches < 100000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original progression instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let zero = matches!(case, Case::Publish { .. })
            || matches!(case,Case::Price{row,item,..}if row>=100||items.records[item as u16 as usize].bytes_08_0f[4]&8==0);
        if zero {
            assert_eq!(
                decoded
                    .bus
                    .rom_resources()
                    .unwrap()
                    .progression_rewards_stats()
                    .unwrap()
                    .source_bytes,
                before
            );
            assert_eq!(
                native
                    .bus
                    .rom_resources()
                    .unwrap()
                    .progression_rewards_stats()
                    .unwrap()
                    .source_bytes,
                native_before
            );
            zero_paths += 1;
        } else {
            assert!(
                decoded
                    .bus
                    .rom_resources()
                    .unwrap()
                    .progression_rewards_stats()
                    .unwrap()
                    .source_bytes
                    > before
            );
            assert!(
                native
                    .bus
                    .rom_resources()
                    .unwrap()
                    .progression_rewards_stats()
                    .unwrap()
                    .source_bytes
                    > native_before
            );
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "progression fullstate/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled[0] > 0 && compiled[1] > 0);
    assert_eq!(compiled[2], 0);
    assert_eq!(zero_paths, 6);
    range(&mut decoded, None);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before = owner.progression_rewards_stats().unwrap();
    for at in END_OFFSET..gba_core::rom_data::progression_rewards::NEXT_SOURCE {
        decoded.bus.rom[at as usize] ^= 0x5a;
        assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize] ^ 0x5a);
    }
    let after = owner.progression_rewards_stats().unwrap();
    assert_eq!(
        (before.source_bytes, before.bus_reads),
        (after.source_bytes, after.bus_reads)
    );
    decoded.bus.rom
        [END_OFFSET as usize..gba_core::rom_data::progression_rewards::NEXT_SOURCE as usize]
        .copy_from_slice(
            &rom[END_OFFSET as usize
                ..gba_core::rom_data::progression_rewards::NEXT_SOURCE as usize],
        );
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in SOURCE_OFFSET..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        for at in (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 5)
            .chain((1..100).map(|i| SOURCE_OFFSET + i * 10 - 1))
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
    assert_eq!(
        owner.progression_rewards_stats().unwrap().record_mask,
        [u64::MAX, (1u64 << 36) - 1]
    );
    let before = owner.progression_rewards_stats().unwrap();
    let old = owner.evaluation_parameters_stats().unwrap();
    for at in [SOURCE_OFFSET - 2, END_OFFSET - 2] {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let after = owner.progression_rewards_stats().unwrap();
    assert_eq!(
        (
            after.source_bytes - before.source_bytes,
            after.delta_bytes - before.delta_bytes,
            after.discount_bytes - before.discount_bytes,
            after.opaque_bytes - before.opaque_bytes
        ),
        (4, 2, 1, 1)
    );
    assert_eq!(
        owner.evaluation_parameters_stats().unwrap().source_bytes - old.source_bytes,
        2
    );
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for swi in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[
                0x0a000000 + SOURCE_OFFSET + 2,
                0x02032400,
                if swi == 0xdf0b { 16 } else { 8 },
            ]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, None);
        let before = owner.progression_rewards_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert_eq!(
            owner.progression_rewards_stats().unwrap().source_bytes - before,
            32
        );
        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + END_OFFSET - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, None);
    let before = owner.progression_rewards_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.progression_rewards_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions_coverage":instructions,"native_dispatches":dispatches,"compiled_delta_discount_opaque_bytes":compiled,"poisoned_source_bytes":1000,"full_states_clocks_equal":true,"publisher_ungated_outside_paths_new_reads0":zero_paths,"original_live_growth_progress_level_flags_prices_predicate":true,"all1000bytes_record_edges_widths_mirrors_bios":true,"all12unowned_tail_bytes_stay_raw":true,"typed_raw_word_crossing_source_bytes":4,"old_evaluation_new_progression_typed_crossing":[2,2],"cpuset_cpufastset_exact_typed":[32,32],"dma2typed2raw":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
