//! Typed packed item parameters; original getter/delta, index widths and RAM stay live.
use super::item_effect_parameter_tests::{run as original, setup, Case, RETURN};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::item_effect_parameters::{END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ITEM_EFFECT_PARAMETERS_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 35] {
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
        AGGREGATE_FLAG_KEYS_FILE,
        TRADE_PARAMETERS_FILE,
        ITEM_EFFECT_PARAMETERS_FILE,
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
#[ignore = "private EUR e40 dataset; item_effect_parameters publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_item_effect_parameters_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..34]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e39").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (39, 38));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (40, 39)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 35);
    assert_eq!(source.manifest.resources.len(), 34);
    for i in 1..34 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_item_effect_parameters(&files[34]).unwrap();
    assert_eq!((items.records.len(), items.source_bytes().len()), (14, 98));
    assert_eq!(
        (
            source.manifest.resources[33].sources.len(),
            source.manifest.resources[33].sources[0].length
        ),
        (1, 98)
    );
    assert_eq!(
        files[34],
        fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_EFFECT_PARAMETERS_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert_eq!(
        source.manifest.resources[33].dependencies,
        vec![ITEM_RECORDS_ID]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(ITEM_EFFECT_PARAMETERS_FILE)
    {
        Err("injected trade parameter publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e39-d38/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e40-d39"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(ITEM_EFFECT_PARAMETERS_FILE),
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
    for case in 0..22 {
        let mut forged = items.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.records[0].coefficients[0] ^= 1,
            1 => forged.records[13].coefficients[8] ^= 16,
            2 => forged.records[0].cap45_52 ^= 1,
            3 => forged.records[13].cap45_52 ^= 128,
            4 => forged.records[13].opaque53_55 ^= 1,
            5 => forged.records[13].item_id = 362,
            6 => forged.records[13].id = 0,
            7 => forged.records[0].source_offset += 1,
            8 => forged.records.swap(0, 13),
            9 => manifest.resources[33].sources[0].length -= 1,
            10 => manifest.resources[33].dependencies.clear(),
            11 => manifest.resources[33].path = "../item-effect-parameters.json".into(),
            12 => manifest.resources[33].sources[0].role = "item-effect-parameters/unknown".into(),
            13 => manifest.resources[33].sources[0].offset += 1,
            14 => manifest.resources[33].sources.clear(),
            15 => manifest.resources[33].sha256 = "0".repeat(64),
            16 => manifest
                .coverage
                .retain(|id| id != ITEM_EFFECT_PARAMETERS_ID),
            17 => forged.records[0].coefficients[0] = 32,
            18 => {
                forged.records.pop();
            }
            19 => forged.records[13].opaque53_55 = 8,
            20 => manifest.resources[33].dependencies = vec![PROGRESSION_GATES_ID.into()],
            _ => manifest.resources[33].sources[0].sha256 = "0".repeat(64),
        }
        if [5, 6, 7, 8, 17, 18, 19].contains(&case) {
            assert!(forged.validate().is_err());
        } else {
            forged.validate().unwrap();
        }
        let bytes = encode_item_effect_parameters(&forged).unwrap();
        if case != 15 {
            manifest.resources[33].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(ITEM_EFFECT_PARAMETERS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(ITEM_EFFECT_PARAMETERS_FILE), &files[34]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(ITEM_EFFECT_PARAMETERS_FILE),
        r.join("missing-resource.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(
        r.join("missing-resource.json"),
        dataset().join(ITEM_EFFECT_PARAMETERS_FILE),
    )
    .unwrap();
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
    let s = owner.item_effect_parameters_stats().unwrap();
    assert_eq!(
        (
            s.source_bytes,
            s.coefficient_bytes,
            s.shared05_bytes,
            s.shared06_bytes,
            s.record_mask
        ),
        (98, 70, 14, 14, (1 << 14) - 1)
    );
    report(
        "managed-result.json",
        serde_json::json!({
            "revision":[1,40,39],"files":35,"resources":34,"source_bytes":98,"source_ranges":1,
            "all33prior_decoded_files_and_e39_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,
            "coherent_coefficients_cap_opaque_bitwidth_id_item_order_source_dependency_path_role_hash_coverage_guards":true,
            "actual_item_owner_dependency":true,"missing_file_transactional_owner":true,"unavailable_copy_load_undo_backup_restart":true,
            "all98bytes_and14record_mask":true,"linux":"pending","usa":"pending","performance_measurements":false
        }),
    );
}

thread_local! {static COMPILED:Cell<[u64;3]>=const{Cell::new([0;3])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 3] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .item_effect_parameters_stats()
            .unwrap()
    };
    [s.coefficient_bytes, s.shared05_bytes, s.shared06_bytes]
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
fn run(
    m: &mut Machine,
    c: Case,
    data: &gba_core::rom_data::item_effect_parameters::ItemEffectParameters,
    visited: &mut BTreeSet<(u32, bool)>,
    fields: &mut BTreeSet<usize>,
) -> usize {
    original(m, c, data, visited, fields, &mut BTreeSet::new())
}
fn native_cases() -> Vec<Case> {
    let mut cases = vec![];
    for index in 0..14 {
        cases.push(Case::Property { property: 0, index });
        cases.push(Case::Delta {
            slot: 18,
            item: 362 + index,
            quantity: 255,
            current: 1,
        });
    }
    for property in [0, 2, 5, 7, 10, 12, 14, 16, 18] {
        cases.push(Case::Property {
            property,
            index: 13,
        });
    }
    for item in [362, 375] {
        for current in [0, 98, 99, 255] {
            cases.push(Case::Delta {
                slot: 0,
                item,
                quantity: 255,
                current,
            });
        }
    }
    for item in [618, 631] {
        cases.push(Case::Delta {
            slot: 0,
            item,
            quantity: 255,
            current: 1,
        });
    }
    cases.push(Case::Property {
        property: 20,
        index: 255,
    });
    cases.push(Case::Property {
        property: 0x112,
        index: 0x10d,
    });
    cases.push(Case::Delta {
        slot: 0x100,
        item: 0x10000 + 375,
        quantity: 0x101,
        current: 0,
    });
    cases
}
#[test]
#[ignore = "private EUR e40 packed item native/full-state/bus; Wine only, Linux/USA pending"]
fn european_item_effect_parameters_native_original_getters_delta_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.item_effect_parameters.as_ref().unwrap();
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
        .map(|&(addr, thumb)| {
            assert!(thumb);
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
    assert!(!visited.contains(&(RETURN, true)));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("item-effect-parameters.o");
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
            .item_effect_parameters_stats()
            .unwrap()
            .source_bytes;
        let nbefore = native
            .bus
            .rom_resources()
            .unwrap()
            .item_effect_parameters_stats()
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
                .expect("original packed item instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let actual = decoded
            .bus
            .rom_resources()
            .unwrap()
            .item_effect_parameters_stats()
            .unwrap()
            .source_bytes
            - before;
        let nactual = native
            .bus
            .rom_resources()
            .unwrap()
            .item_effect_parameters_stats()
            .unwrap()
            .source_bytes
            - nbefore;
        if let Case::Property { property, .. } = c {
            assert_eq!(actual > 0, (property as u8) < 20);
        } else {
            assert!(actual > 0);
        }
        assert_eq!(actual, nactual);
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "item-effect full-state/clock {c:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&n| n > 0));
    range(&mut decoded, None);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let before = owner.item_effect_parameters_stats().unwrap();
    let prior_end = gba_core::rom_data::trade_parameters::END_OFFSET;
    for at in (prior_end..SOURCE_OFFSET).chain(END_OFFSET..END_OFFSET + 4) {
        decoded.bus.rom[at as usize] ^= 0x5a;
        assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize] ^ 0x5a);
    }
    assert_eq!(owner.item_effect_parameters_stats().unwrap(), before);
    decoded.bus.rom[prior_end as usize..SOURCE_OFFSET as usize]
        .copy_from_slice(&rom[prior_end as usize..SOURCE_OFFSET as usize]);
    decoded.bus.rom[END_OFFSET as usize..END_OFFSET as usize + 4]
        .copy_from_slice(&rom[END_OFFSET as usize..END_OFFSET as usize + 4]);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in SOURCE_OFFSET..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        for at in (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 5)
            .chain((1..14).map(|i| SOURCE_OFFSET + i * 7 - 1))
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
        owner.item_effect_parameters_stats().unwrap().record_mask,
        (1 << 14) - 1
    );
    for row in 0..14 {
        for field in 0..7 {
            let at = SOURCE_OFFSET + row * 7 + field;
            assert_eq!(
                (RT_API.read32)(
                    &mut decoded as *mut Machine as *mut std::ffi::c_void,
                    0x08000000 + at
                ),
                u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
            );
        }
    }
    let before = owner.item_effect_parameters_stats().unwrap();
    assert_eq!(
        decoded.bus.read32(0x08000000 + END_OFFSET - 2),
        u32::from_le_bytes(
            rom[END_OFFSET as usize - 2..END_OFFSET as usize + 2]
                .try_into()
                .unwrap()
        )
    );
    let after = owner.item_effect_parameters_stats().unwrap();
    assert_eq!(
        (
            after.source_bytes - before.source_bytes,
            after.shared05_bytes - before.shared05_bytes,
            after.shared06_bytes - before.shared06_bytes
        ),
        (2, 1, 1)
    );
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for (swi, src, count, typed) in [
        (0xdf0b, SOURCE_OFFSET, 16, 32),
        (0xdf0c, SOURCE_OFFSET, 8, 32),
        (0xdf0b, END_OFFSET - 2, 2, 2),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, None);
        let before = owner.item_effect_parameters_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert_eq!(
            owner.item_effect_parameters_stats().unwrap().source_bytes - before,
            typed
        );

        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.cpu.regs[..3].copy_from_slice(&[
            0x08000000 + prior_end - 2,
            0x02032400,
            (SOURCE_OFFSET + 2 - (prior_end - 2)) / 2,
        ]);
        m.cpu.regs[15] = 0x02000000;
        m.cpu.cpsr = 0xbf;
        m.bus.write16(0x02000000, 0xdf0b);
    }
    range(&mut decoded, None);
    for at in prior_end - 2..prior_end {
        decoded.bus.rom[at as usize] ^= 0x5a;
    }
    let before = owner.item_effect_parameters_stats().unwrap().source_bytes;
    let old_before = owner.trade_parameters_stats().unwrap().source_bytes;
    legacy.step();
    decoded.step();
    assert_eq!(
        owner.item_effect_parameters_stats().unwrap().source_bytes - before,
        2
    );
    assert_eq!(
        owner.trade_parameters_stats().unwrap().source_bytes - old_before,
        2
    );
    range(&mut decoded, Some(&rom));
    decoded.bus.rom[prior_end as usize - 2..prior_end as usize]
        .copy_from_slice(&rom[prior_end as usize - 2..prior_end as usize]);
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + END_OFFSET - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, None);
    let before = owner.item_effect_parameters_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.item_effect_parameters_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({
            "original_paths":cases.len(),"instructions_coverage":instructions,"native_dispatches":dispatches,
            "compiled_coefficient_shared05_shared06_bytes":compiled,"poisoned_source_bytes":98,"full_states_clocks_equal":true,
            "original_getter_delta_indexwidths_cap_rawalias_and_live_current":true,
            "all98bytes_record_fieldedges_widths_mirrors_bios":true,"following4_raw_bytes":true,
            "word_and_cpuset_adjacent_trade2_item_effect2_typed_owners":true,
            "tail2typed2raw_word_cpuset_dma":true,"cpuset_cpufastset_typed":[32,32],
            "linux":"pending","usa":"pending","performance_measurements":false
        }),
    );
}
