//! Score/threshold ownership while original live arithmetic and RNG run.
use super::evaluation_parameter_tests::{run, setup, Case};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::evaluation_parameters::{
        EvaluationParameters, END_OFFSET, SCORE_END, SCORE_OFFSET, THRESHOLD_SOURCE,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_EVALUATION_PARAMETERS_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 29] {
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
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn range(m: &mut Machine, rom: Option<&[u8]>) {
    for (start, end) in [(SCORE_OFFSET, SCORE_END), (THRESHOLD_SOURCE, END_OFFSET)] {
        let span = &mut m.bus.rom[start as usize..end as usize];
        if let Some(r) = rom {
            span.copy_from_slice(&r[start as usize..end as usize]);
        } else {
            span.fill(0xcd);
        }
    }
}
#[test]
#[ignore = "private EUR e33 dataset; evaluation_parameters publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_evaluation_parameters_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..28]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e33").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (33, 32));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (34, 33)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 29);
    assert_eq!(source.manifest.resources.len(), 28);
    for i in 1..28 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_evaluation_parameters(&files[28]).unwrap();
    assert_eq!(
        items.score_bytes().len() + items.threshold_bytes().len(),
        38
    );
    assert_eq!(source.manifest.resources[27].sources.len(), 2);
    assert_eq!(source.manifest.resources[27].sources[0].length, 18);
    assert_eq!(
        files[28],
        fs::read(std::env::var_os("FFTA_STATE_TEST_EVALUATION_PARAMETERS_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert_eq!(
        (
            items.terms.len(),
            items.bonuses.len(),
            items.thresholds.len()
        ),
        (7, 2, 5)
    );
    assert_eq!(source.manifest.resources[27].sources[1].length, 20);
    assert!(source.manifest.resources[27].dependencies.is_empty());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(EVALUATION_PARAMETERS_FILE)
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
        .join("v1/e33-d32/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e34-d33"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(EVALUATION_PARAMETERS_FILE),
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
            0 => forged.terms[0].divisor ^= 1,
            1 => forged.terms[6].weight ^= 0x80,
            2 => forged.bonuses[0].byte00 ^= 1,
            3 => forged.thresholds[4].bytes01_03[2] ^= 1,
            4 => manifest.resources[27].sources[0].length -= 1,
            5 => manifest.resources[27]
                .dependencies
                .push(GLOBAL_PAIR_FLAGS_ID.into()),
            6 => manifest.resources[27].path = "../evaluation-parameters.json".into(),
            7 => manifest.resources[27].sources[0].role = "evaluation-parameters/unknown".into(),
            8 => manifest.resources[27].sources[1].role = "evaluation-parameters/unknown".into(),
            _ => manifest.resources[27].sources[1].offset += 1,
        }

        forged.validate().unwrap();
        let bytes = encode_evaluation_parameters(&forged).unwrap();
        manifest.resources[27].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(EVALUATION_PARAMETERS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(EVALUATION_PARAMETERS_FILE), &files[28]).unwrap();
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
    for (start, end) in [(SCORE_OFFSET, SCORE_END), (THRESHOLD_SOURCE, END_OFFSET)] {
        for at in start..end {
            assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
        }
    }
    let s = owner.evaluation_parameters_stats().unwrap();
    assert!(
        s.divisor_bytes > 0
            && s.weight_bytes > 0
            && s.bonus_bytes > 0
            && s.score_opaque_bytes > 0
            && s.threshold_bytes > 0
            && s.threshold_opaque_bytes > 0
    );
    assert_eq!(s.threshold_mask, 31);

    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,34,33],"files":29,"resources":28,"source_bytes":38,"source_ranges":2,"all_twenty_seven_prior_decoded_files_and_e33_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_divisor_weight_unused_bonus_opaque_threshold_source_dependency_path_both_roles_guards":true,"no_unproved_semantic_dependency":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;6]>=const{Cell::new([0;6])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 6] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .evaluation_parameters_stats()
            .unwrap()
    };
    [
        s.divisor_bytes,
        s.weight_bytes,
        s.bonus_bytes,
        s.score_opaque_bytes,
        s.threshold_bytes,
        s.threshold_opaque_bytes,
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
fn cases(data: &EvaluationParameters) -> Vec<Case> {
    // Complete prefixes share an exit; isolated arithmetic fragments were already
    // proved by the source oracle and would overlap score-prefix stop addresses.
    let mut out = vec![];
    for job in [1, 2, 3, 8, 20, 43] {
        for high in [false, true] {
            out.push(Case::Score {
                job,
                level: if high { 255 } else { 12 },
                hp: if high { 65535 } else { 120 },
                mp: if high { 32768 } else { 40 },
                stats: if high {
                    [0x8000, 0x7fff, 0xffff, 0x1234]
                } else {
                    [120, 90, 70, 60]
                },
            });
        }
    }
    for t in &data.thresholds {
        for seed in [0, 0xdeadbeef] {
            out.push(Case::Threshold {
                class: t.id as u32,
                seed,
            });
        }
    }
    out.push(Case::Threshold { class: 0, seed: 2 });
    for class in [1, 5] {
        for draw in [0x8000, 0xffff0000] {
            out.push(Case::ThresholdSuffix { class, draw });
        }
    }
    out
}
#[test]
#[ignore = "private EUR evaluation typed live score/arithmetic/RNG/native/bus; reduced Wine only, Linux/USA pending"]
fn european_evaluation_parameters_native_live_score_arithmetic_rng_threshold_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.evaluation_parameters.as_ref().unwrap();
    let cases = cases(data);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
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
    let object = out.join("evaluation-parameters.o");
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
    let mut zero_paths = 0;
    for &case in &cases {
        run(&mut legacy, case, data, &mut BTreeSet::new());
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .evaluation_parameters_stats()
            .unwrap()
            .source_bytes;
        let native_before = native
            .bus
            .rom_resources()
            .unwrap()
            .evaluation_parameters_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        run(&mut decoded, case, data, &mut BTreeSet::new());
        setup(&mut native, case);
        while native.cpu.regs[15] != case.stop() {
            assert!(dispatches < 100000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original evaluation instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let expect_zero = matches!(case, Case::Threshold { class: 0, .. })
            || matches!(case,Case::Bonus{index,predicate,..}if predicate as u8!=if index==0{1}else{0});
        if expect_zero {
            assert_eq!(
                decoded
                    .bus
                    .rom_resources()
                    .unwrap()
                    .evaluation_parameters_stats()
                    .unwrap()
                    .source_bytes,
                before
            );
            assert_eq!(
                native
                    .bus
                    .rom_resources()
                    .unwrap()
                    .evaluation_parameters_stats()
                    .unwrap()
                    .source_bytes,
                native_before
            );
            zero_paths += 1;
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "evaluation fullstate/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled[0] > 0 && compiled[1] > 0 && compiled[2] > 0 && compiled[4] > 0);
    assert_eq!((compiled[3], compiled[5]), (0, 0));
    assert_eq!(zero_paths, 1);
    let native_stats = native
        .bus
        .rom_resources()
        .unwrap()
        .evaluation_parameters_stats()
        .unwrap();
    assert_eq!(native_stats.threshold_mask, 31);
    range(&mut decoded, None);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before = owner.evaluation_parameters_stats().unwrap();
    for at in SCORE_END..THRESHOLD_SOURCE {
        decoded.bus.rom[at as usize] ^= 0x5a;
        assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize] ^ 0x5a);
    }
    let after = owner.evaluation_parameters_stats().unwrap();
    assert_eq!(
        (after.source_bytes, after.bus_reads),
        (before.source_bytes, before.bus_reads)
    );
    decoded.bus.rom[SCORE_END as usize..THRESHOLD_SOURCE as usize]
        .copy_from_slice(&rom[SCORE_END as usize..THRESHOLD_SOURCE as usize]);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (start, end) in [(SCORE_OFFSET, SCORE_END), (THRESHOLD_SOURCE, END_OFFSET)] {
            for at in start..end {
                assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
            }
            for at in (start - 3..start + 4).chain(end - 3..end + 5) {
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
    }
    let before = owner.evaluation_parameters_stats().unwrap();
    for at in [
        SCORE_OFFSET - 2,
        SCORE_END - 2,
        THRESHOLD_SOURCE - 2,
        END_OFFSET - 2,
    ] {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let after = owner.evaluation_parameters_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 8);
    assert_eq!(after.divisor_bytes - before.divisor_bytes, 1);
    assert_eq!(after.weight_bytes - before.weight_bytes, 1);
    assert_eq!(after.bonus_bytes - before.bonus_bytes, 1);
    assert_eq!(after.score_opaque_bytes - before.score_opaque_bytes, 1);
    assert_eq!(after.threshold_bytes - before.threshold_bytes, 1);
    assert_eq!(
        after.threshold_opaque_bytes - before.threshold_opaque_bytes,
        3
    );
    assert_eq!(after.threshold_mask, 31);
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for (source_at, typed) in [(SCORE_OFFSET, 18), (THRESHOLD_SOURCE, 20)] {
        for swi in [0xdf0b, 0xdf0c] {
            for m in [&mut legacy, &mut decoded] {
                m.load_state(&start).unwrap();
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + source_at,
                    0x02032400,
                    if swi == 0xdf0b { 16 } else { 8 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus.write16(0x02000000, swi);
            }
            range(&mut decoded, None);
            let before = owner.evaluation_parameters_stats().unwrap().source_bytes;
            legacy.step();
            decoded.step();
            assert_eq!(
                owner.evaluation_parameters_stats().unwrap().source_bytes - before,
                typed
            );
            range(&mut decoded, Some(&rom));
            assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
        }
    }
    for at in [SCORE_END - 2, END_OFFSET - 2] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.bus.write32(0x040000d4, 0x08000000 + at);
            m.bus.write32(0x040000d8, 0x02032000);
            m.bus.write16(0x040000dc, 2);
        }
        range(&mut decoded, None);
        let before = owner.evaluation_parameters_stats().unwrap().source_bytes;
        for m in [&mut legacy, &mut decoded] {
            m.bus.write16(0x040000de, 0x8000);
        }
        assert_eq!(
            owner.evaluation_parameters_stats().unwrap().source_bytes - before,
            2
        );
        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions_coverage":instructions,"native_dispatches":dispatches,"compiled_divisor_weight_bonus_scoreopaque_threshold_thresholdopaque_bytes":compiled,"native_threshold_mask":native_stats.threshold_mask,"poisoned_source_bytes":38,"full_states_clocks_equal":true,"class0_path_new_reads0":zero_paths,"live_getters_unsigned_signed_u8_wrap_rng_and_negative_suffix":true,"all38bytes_edges_widths_mirrors_bios":true,"all18unowned_gap_and_class0row_bytes_stay_raw":true,"four_typed_raw_word_crossings_source_bytes":8,"bothspans_cpuset_cpufastset_exact_typed":[18,20],"bothspans_dma2typed2raw":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
