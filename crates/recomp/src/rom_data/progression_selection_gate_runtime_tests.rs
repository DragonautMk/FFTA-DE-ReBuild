//! Selection ownership while original reset/count/ANY/flag/store routines run.
use super::progression_selection_gate_tests::{run, setup, Case, RETURN};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::progression_selection_gates::{END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_PROGRESSION_SELECTION_GATES_MANAGED_ROOT").unwrap(),
    )
}
thread_local! {static COMPILED:Cell<[u64;6]>=const{Cell::new([0;6])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 6] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .progression_selection_gates_stats()
            .unwrap()
    };
    [
        s.selector_bytes,
        s.minimum01_bytes,
        s.minimum02_bytes,
        s.minimum04_bytes,
        s.any_minima_bytes,
        s.flag_bytes,
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
fn native_cases() -> Vec<Case> {
    let mut out = vec![Case::default()];
    for reset in [0, 1, 0x100, 0x101] {
        out.push(Case {
            selector_flags: 15,
            reset,
            ..Case::default()
        });
    }
    out.extend([
        Case {
            byte01: 128,
            ..Case::default()
        },
        Case {
            halfword02: 65535,
            ..Case::default()
        },
        Case {
            aggregate_flags: 300,
            ..Case::default()
        },
        Case {
            aggregate_flags: 299,
            special36: true,
            ..Case::default()
        },
        Case {
            aggregate_flags: 299,
            linked_item: true,
            ..Case::default()
        },
    ]);
    for field in 0..8 {
        let mut levels = [0; 8];
        levels[field] = 70;
        out.push(Case {
            levels,
            ..Case::default()
        });
    }
    out
}
#[test]
#[ignore = "private EUR e37 selection native/full-state/bus; Wine only, Linux/USA pending"]
fn european_progression_selection_native_original_reset_any_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.progression_selection_gates.as_ref().unwrap();
    let cases = native_cases();
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &c in &cases {
        instructions += run(&mut tracer, c, data, &mut visited, &mut BTreeSet::new());
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = visited
        .iter()
        .map(|&addr| {
            let mut instrs = vec![armv4t::decode_thumb(view.read16(addr), addr)];
            let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                instrs.push(next);
            }
            Block {
                addr,
                thumb: true,
                instrs,
            }
        })
        .collect();
    assert!(!visited.contains(&RETURN));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("progression-selection-gates.o");
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
    for &c in &cases {
        run(
            &mut legacy,
            c,
            data,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        );
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .progression_selection_gates_stats()
            .unwrap()
            .source_bytes;
        let nbefore = native
            .bus
            .rom_resources()
            .unwrap()
            .progression_selection_gates_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        run(
            &mut decoded,
            c,
            data,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        );
        setup(&mut native, c);
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 500000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original selection instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let actual = decoded
            .bus
            .rom_resources()
            .unwrap()
            .progression_selection_gates_stats()
            .unwrap()
            .source_bytes
            - before;
        let nactual = native
            .bus
            .rom_resources()
            .unwrap()
            .progression_selection_gates_stats()
            .unwrap()
            .source_bytes
            - nbefore;
        assert!(actual > 0);
        assert_eq!(actual, nactual);
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "selection full-state/clock {c:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&n| n > 0));
    range(&mut decoded, None);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before = owner.progression_selection_gates_stats().unwrap();
    let old_end = gba_core::rom_data::progression_gates::END_OFFSET;
    for at in (old_end..SOURCE_OFFSET).chain(END_OFFSET..END_OFFSET + 2) {
        decoded.bus.rom[at as usize] ^= 0x5a;
        assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize] ^ 0x5a);
    }
    assert_eq!(owner.progression_selection_gates_stats().unwrap(), before);
    decoded.bus.rom[old_end as usize..SOURCE_OFFSET as usize]
        .copy_from_slice(&rom[old_end as usize..SOURCE_OFFSET as usize]);
    decoded.bus.rom[END_OFFSET as usize..END_OFFSET as usize + 2]
        .copy_from_slice(&rom[END_OFFSET as usize..END_OFFSET as usize + 2]);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in SOURCE_OFFSET..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        for at in (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 5)
            .chain((1..64).map(|i| SOURCE_OFFSET + i * 16 - 1))
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
        owner
            .progression_selection_gates_stats()
            .unwrap()
            .record_mask,
        u64::MAX
    );
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for (swi, src, count, typed, old_typed) in [
        (0xdf0b, SOURCE_OFFSET + 2, 16, 32, 0),
        (0xdf0c, SOURCE_OFFSET + 2, 8, 32, 0),
        (0xdf0b, old_end - 2, 10, 2, 2),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, None);
        let before = owner
            .progression_selection_gates_stats()
            .unwrap()
            .source_bytes;
        let old_before = owner.progression_gates_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert_eq!(
            owner
                .progression_selection_gates_stats()
                .unwrap()
                .source_bytes
                - before,
            typed
        );
        assert_eq!(
            owner.progression_gates_stats().unwrap().source_bytes - old_before,
            old_typed
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
    let before = owner
        .progression_selection_gates_stats()
        .unwrap()
        .source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner
            .progression_selection_gates_stats()
            .unwrap()
            .source_bytes
            - before,
        2
    );
    range(&mut decoded, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions_coverage":instructions,
        "native_dispatches":dispatches,"compiled_selector_minimum01_minimum02_minimum04_any_flag_bytes":compiled,
        "poisoned_source_bytes":1024,"full_states_clocks_equal":true,"original_reset_count_any_flag_terminal_store":true,
        "all1024bytes_recordedges_widths_mirrors_bios":true,"all16row0_and2trailing_gap_bytes_raw":true,
        "oldgate2_raw16_newselection2_coexistence":true,"cpuset_cpufastset_typed":[32,32],"dma2typed2raw":true,
        "linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 32] {
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
        PROGRESSION_GATES_FILE,
        PROGRESSION_SELECTION_GATES_FILE,
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
#[ignore = "private EUR e36 dataset; progression_selection_gates publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_progression_selection_gates_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..31]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e36").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (36, 35));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (37, 36)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 32);
    assert_eq!(source.manifest.resources.len(), 31);
    for i in 1..31 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_progression_selection_gates(&files[31]).unwrap();
    assert_eq!(
        (items.records.len(), items.source_bytes().len()),
        (64, 1024)
    );
    assert_eq!(
        (
            source.manifest.resources[30].sources.len(),
            source.manifest.resources[30].sources[0].length
        ),
        (1, 1024)
    );
    assert_eq!(
        files[31],
        fs::read(
            std::env::var_os("FFTA_STATE_TEST_PROGRESSION_SELECTION_GATES_SOURCE_FILE").unwrap()
        )
        .unwrap()
    );
    assert!(source.manifest.resources[30].dependencies.is_empty());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(PROGRESSION_SELECTION_GATES_FILE)
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
        .join("v1/e36-d35/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e37-d36"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(PROGRESSION_SELECTION_GATES_FILE),
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
    for case in 0..14 {
        let mut forged = items.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.records[0].minimum01 ^= 1,
            1 => forged.records[63].any_minima06_0d[7] ^= 0x80,
            2 => forged.records[63].minimum02 ^= 1,
            3 => forged.records[0].selector00 ^= 1,
            4 => manifest.resources[30].sources[0].length -= 1,
            5 => manifest.resources[30]
                .dependencies
                .push(ITEM_RECORDS_ID.into()),
            6 => manifest.resources[30].path = "../progression-rewards.json".into(),
            7 => manifest.resources[30].sources[0].role = "progression-rewards/unknown".into(),
            8 => manifest.resources[30].sources[0].offset += 1,
            9 => forged.records.swap(0, 63),
            10 => forged.records[63].flag0e ^= 0x80,
            11 => {
                manifest.resources[30].sources.clear();
            }
            12 => forged.records[0].minimum04 ^= 1,
            _ => {
                manifest.resources[30].sha256 = "0".repeat(64);
            }
        }

        if case != 9 {
            forged.validate().unwrap();
        } else {
            assert!(forged.validate().is_err());
        }
        let bytes = encode_progression_selection_gates(&forged).unwrap();
        if case != 13 {
            manifest.resources[30].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(PROGRESSION_SELECTION_GATES_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(PROGRESSION_SELECTION_GATES_FILE), &files[31]).unwrap();
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
    let s = owner.progression_selection_gates_stats().unwrap();
    assert_eq!(
        (
            s.selector_bytes,
            s.minimum01_bytes,
            s.minimum02_bytes,
            s.minimum04_bytes,
            s.any_minima_bytes,
            s.flag_bytes
        ),
        (64, 64, 128, 128, 512, 128)
    );
    assert_eq!(s.record_mask, u64::MAX);

    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,37,36],"files":32,"resources":31,"source_bytes":1024,"source_ranges":1,"all_thirty_prior_decoded_files_and_e36_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_minimum_any_selector_flag_lastrow_reorder_source_dependency_path_role_guards":true,"no_invented_semantic_foreign_key_dependency":true,"unavailable_copy_load_undo_backup_restart":true,"all1024bytes_and64record_mask":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
