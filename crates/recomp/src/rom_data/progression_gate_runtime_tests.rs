//! Gate ownership while original live minima and received-mask routines run.
use super::progression_gate_tests::{run, setup, Case, RETURN};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::progression_gates::{ProgressionGates, END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_GATES_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 31] {
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
#[ignore = "private EUR e35 dataset; progression_gates publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_progression_gates_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..30]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e35").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (35, 34));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (36, 35)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 31);
    assert_eq!(source.manifest.resources.len(), 30);
    for i in 1..30 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_progression_gates(&files[30]).unwrap();
    assert_eq!((items.records.len(), items.source_bytes().len()), (64, 768));
    assert_eq!(
        (
            source.manifest.resources[29].sources.len(),
            source.manifest.resources[29].sources[0].length
        ),
        (1, 768)
    );
    assert_eq!(
        files[30],
        fs::read(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_GATES_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert!(source.manifest.resources[29].dependencies.is_empty());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(PROGRESSION_GATES_FILE)
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
        .join("v1/e35-d34/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e36-d35"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(PROGRESSION_GATES_FILE), b"partial").unwrap();
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
    for case in 0..11 {
        let mut forged = items.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.records[0].requirements04_0b[0] ^= 1,
            1 => forged.records[63].requirements04_0b[7] ^= 0x80,
            2 => forged.records[63].halfword01 ^= 1,
            3 => forged.records[0].byte00 ^= 1,
            4 => manifest.resources[29].sources[0].length -= 1,
            5 => manifest.resources[29]
                .dependencies
                .push(ITEM_RECORDS_ID.into()),
            6 => manifest.resources[29].path = "../progression-rewards.json".into(),
            7 => manifest.resources[29].sources[0].role = "progression-rewards/unknown".into(),
            8 => manifest.resources[29].sources[0].offset += 1,
            9 => forged.records.swap(0, 63),
            _ => forged.records[63].byte03 ^= 0x80,
        }

        if case != 9 {
            forged.validate().unwrap();
        } else {
            assert!(forged.validate().is_err());
        }
        let bytes = encode_progression_gates(&forged).unwrap();
        manifest.resources[29].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(PROGRESSION_GATES_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(PROGRESSION_GATES_FILE), &files[30]).unwrap();
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
    let s = owner.progression_gates_stats().unwrap();
    assert_eq!(
        (
            s.requirement_bytes,
            s.return_bytes,
            s.output_bytes,
            s.unused_bytes
        ),
        (512, 128, 64, 64)
    );
    assert_eq!(s.record_mask, u64::MAX);

    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,36,35],"files":31,"resources":30,"source_bytes":768,"source_ranges":1,"all_twenty_nine_prior_decoded_files_and_e35_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_requirement_return_output_unused_lastrow_reorder_source_dependency_path_role_guards":true,"no_invented_receipt_foreign_key_dependency":true,"unavailable_copy_load_undo_backup_restart":true,"all768bytes_and64record_mask":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;4]>=const{Cell::new([0;4])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 4] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .progression_gates_stats()
            .unwrap()
    };
    [
        s.requirement_bytes,
        s.return_bytes,
        s.output_bytes,
        s.unused_bytes,
    ]
}
fn count(before: [u64; 4], after: [u64; 4]) {
    COMPILED.with(|v| {
        let mut n = v.get();
        for i in 0..4 {
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
fn cases(data: &ProgressionGates) -> Vec<Case> {
    let mut out = vec![];
    for row in [0, 1, 31, 32, 46, 63] {
        out.push(Case {
            levels: data.records[row].requirements04_0b,
            received: !(1u64 << row),
        });
    }
    for field in 0..8 {
        let row = data
            .records
            .iter()
            .position(|r| r.requirements04_0b[field] > 0)
            .unwrap();
        let mut levels = data.records[row].requirements04_0b;
        levels[field] -= 1;
        out.push(Case {
            levels,
            received: !(1u64 << row),
        });
    }
    for levels in [[0; 8], [255; 8]] {
        out.push(Case {
            levels,
            received: u64::MAX,
        });
        out.push(Case {
            levels,
            received: 0,
        });
    }
    out.push(Case {
        levels: [255; 8],
        received: !((1u64 << 31) | (1u64 << 32)),
    });
    out
}
#[test]
#[ignore = "private EUR typed gates/minima/u64mask/native/bus; reduced Wine only, Linux/USA pending"]
fn european_progression_gates_native_live_minima_received_mask_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.progression_gates.as_ref().unwrap();
    let cases = cases(data);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut tracer, case, data, &mut visited, &mut BTreeSet::new());
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
            assert!(thumb);
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
    assert!(!visited.iter().any(|&(pc, _)| pc == RETURN));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("progression-gates.o");
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
    COMPILED.with(|v| v.set([0; 4]));
    let mut dispatches = 0;
    let mut zero_paths = 0;
    let mut zero_value_successes = 0;
    for &case in &cases {
        run(
            &mut legacy,
            case,
            data,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        );
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .progression_gates_stats()
            .unwrap()
            .source_bytes;
        let native_before = native
            .bus
            .rom_resources()
            .unwrap()
            .progression_gates_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        run(
            &mut decoded,
            case,
            data,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        );
        setup(&mut native, case);
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 100000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original gate instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let actual = decoded
            .bus
            .rom_resources()
            .unwrap()
            .progression_gates_stats()
            .unwrap()
            .source_bytes
            - before;
        let native_actual = native
            .bus
            .rom_resources()
            .unwrap()
            .progression_gates_stats()
            .unwrap()
            .source_bytes
            - native_before;
        assert_eq!(actual, native_actual);
        if case.received == u64::MAX {
            assert_eq!(actual, 0);
            zero_paths += 1;
        } else {
            assert!(actual > 0);
        }
        let outcome = data.evaluate(case.levels, case.received);
        if outcome.value == 0 && outcome.matched_id.is_some() {
            assert_ne!(outcome.received, case.received);
            zero_value_successes += 1;
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "gate fullstate/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled[0] > 0 && compiled[1] > 0 && compiled[2] > 0);
    assert_eq!(compiled[3], 0);
    assert_eq!(zero_paths, 2);
    assert_eq!(zero_value_successes, 2);
    range(&mut decoded, None);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before = owner.progression_gates_stats().unwrap();
    let old_end = gba_core::rom_data::progression_rewards::END_OFFSET;
    for at in old_end..SOURCE_OFFSET {
        decoded.bus.rom[at as usize] ^= 0x5a;
        assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize] ^ 0x5a);
    }
    let after = owner.progression_gates_stats().unwrap();
    assert_eq!(
        (before.source_bytes, before.bus_reads),
        (after.source_bytes, after.bus_reads)
    );
    decoded.bus.rom[old_end as usize..SOURCE_OFFSET as usize]
        .copy_from_slice(&rom[old_end as usize..SOURCE_OFFSET as usize]);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in SOURCE_OFFSET..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        for at in (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 5)
            .chain((1..64).map(|i| SOURCE_OFFSET + i * 12 - 1))
            .chain([0x6e7ffe])
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
        owner.progression_gates_stats().unwrap().record_mask,
        u64::MAX
    );
    let before = owner.progression_gates_stats().unwrap();
    let old = owner.progression_rewards_stats().unwrap();
    for at in [old_end - 2, SOURCE_OFFSET - 2, END_OFFSET - 2] {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let after = owner.progression_gates_stats().unwrap();
    assert_eq!(
        (
            after.source_bytes - before.source_bytes,
            after.requirement_bytes - before.requirement_bytes,
            after.return_bytes - before.return_bytes,
            after.unused_bytes - before.unused_bytes
        ),
        (4, 2, 1, 1)
    );
    let old_after = owner.progression_rewards_stats().unwrap();
    assert_eq!(
        (
            old_after.source_bytes - old.source_bytes,
            old_after.discount_bytes - old.discount_bytes,
            old_after.opaque_bytes - old.opaque_bytes
        ),
        (2, 1, 1)
    );
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for swi in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[
                0x0a6e7ff0,
                0x02032400,
                if swi == 0xdf0b { 16 } else { 8 },
            ]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, None);
        let before = owner.progression_gates_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert_eq!(
            owner.progression_gates_stats().unwrap().source_bytes - before,
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
    let before = owner.progression_gates_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.progression_gates_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions_coverage":instructions,"native_dispatches":dispatches,"compiled_requirement_return_output_unused_bytes":compiled,"poisoned_source_bytes":768,"full_states_clocks_equal":true,"fullmask_paths_new_reads0":zero_paths,"zero_value_successes_still_mark_bit":zero_value_successes,"original_live8minima_physical_order_u64shift_mask_output_return":true,"all768bytes_recordedges_widths_mirrors_bios_bothpages":true,"all12unowned_preceding_gap_bytes_stay_raw":true,"typed_raw_word_crossing_new_source_bytes":4,"oldreward2_and_newgate4_typed_coexistence":true,"cpuset_cpufastset_cross_page_exact_typed":[32,32],"dma2typed2raw":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
