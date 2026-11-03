//! Special-unit ownership while original getters, live palettes and initialization run.
use super::metadata14_tests::{run, setup, Case, UNIT};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::metadata14::{Metadata14, ADDRESS_BIAS, END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA14_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 26] {
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
#[ignore = "private EUR e30 dataset; metadata14 publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_metadata14_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..25]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e30").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (30, 29));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (31, 30)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 26);
    assert_eq!(source.manifest.resources.len(), 25);
    for i in 1..25 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_metadata14(&files[25]).unwrap();
    assert_eq!(items.source_bytes().len(), 1302);
    assert_eq!(source.manifest.resources[24].sources.len(), 1);
    assert_eq!(source.manifest.resources[24].sources[0].length, 1302);
    assert_eq!(
        files[25],
        fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA14_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(items.records.len(), 93);
    let owner52 = read_dataset(&dataset()).unwrap().metadata52.unwrap();
    items.validate_metadata52(&owner52).unwrap();
    let mut bad = items.clone();
    bad.records[92].selectors01_02[1] = 116;
    assert!(bad.validate_metadata52(&owner52).is_err());
    let mut unproved = items.clone();
    unproved.records[0].selectors01_02[0] = 255;
    unproved.validate_metadata52(&owner52).unwrap();
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(
        &interrupted,
        &mut source,
        &mut |p, b| if p.ends_with(METADATA14_FILE) {
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
        .join("v1/e30-d29/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e31-d30"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(METADATA14_FILE), b"partial").unwrap();
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
            1 => forged.records[1].byte03 = 127,
            2 => forged.records[2].halfwords04_06[1] ^= 1,
            3 => forged.records[92].bytes0c_0d[1] ^= 0x80,
            4 => manifest.resources[24].sources[0].length -= 1,
            5 => manifest.resources[24].dependencies.clear(),
            6 => manifest.resources[24].path = "../linked-metadata14.json".into(),
            7 => manifest.resources[24].sources[0].role = "metadata14/unknown".into(),
            8 => forged.records[4].selectors01_02[0] = 255,
            _ => {
                let a = forged.records[0].bytes0c_0d;
                forged.records[0].bytes0c_0d = forged.records[92].bytes0c_0d;
                forged.records[92].bytes0c_0d = a;
            }
        }
        forged.validate().unwrap();
        let bytes = encode_metadata14(&forged).unwrap();
        manifest.resources[24].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(METADATA14_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(METADATA14_FILE), &files[25]).unwrap();
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
    let s = owner.metadata14_stats().unwrap();
    assert!(s.byte_fields > 0 && s.halfword_bytes > 0 && s.packed_bytes > 0 && s.tail_bytes > 0);
    assert_eq!(s.records_used(), 93);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,31,30],"files":26,"resources":25,"records":93,"source_bytes":1302,"all_twenty_four_prior_decoded_files_and_e30_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_byte_halfword_packed_tail_unproved_byte01_payload_swap_source_dependency_path_role_transactional_guards":true,"parsed_byte02_metadata52_owner_bound_only":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;4]>=const{Cell::new([0;4])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 4] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .metadata14_stats()
            .unwrap()
    };
    [
        s.byte_fields,
        s.halfword_bytes,
        s.packed_bytes,
        s.tail_bytes,
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
fn cases(data: &Metadata14) -> Vec<Case> {
    let mut out = vec![];
    for id in [2, 20, 94] {
        for (field, argument) in [(4, 0), (6, 0), (11, 0), (12, 1), (12, 0)] {
            out.push(Case::Get {
                id,
                field,
                argument,
            });
        }
        for override_job in [0, 2] {
            out.push(Case::Initialize {
                id,
                override_job,
                aux: 0x85,
            });
        }
    }
    for argument in [0, 1] {
        for some in [false, true] {
            if let Some(r) = data
                .records
                .iter()
                .find(|r| r.palette(argument).source_index.is_some() == some)
            {
                for book in [0, 1, 2] {
                    out.push(Case::Palette {
                        id: r.id,
                        argument,
                        book,
                    });
                }
            }
        }
    }
    if let Some(r) = data
        .records
        .iter()
        .find(|r| r.palette(1).source_index.is_some())
    {
        out.push(Case::Palette {
            id: r.id,
            argument: 0x10001,
            book: 3,
        });
    }
    out.push(Case::Initialize {
        id: 2,
        override_job: 255,
        aux: 0xff,
    });
    for id in [0, 1] {
        out.push(Case::Get {
            id,
            field: 4,
            argument: 0,
        });
    }
    out
}
fn prepare(m: &mut Machine, case: Case) {
    setup(m, case);
    if case.id() < 2 {
        m.bus.ewram[0x24000..0x24108].fill(0);
        m.bus.write16(UNIT, 1);
        m.bus.write8(UNIT + 4, case.id());
        m.bus.write8(UNIT + 5, 2);
        m.bus.write8(UNIT + 7, 2);
    }
}
fn interpreted(
    m: &mut Machine,
    case: Case,
    data: &Metadata14,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    if case.id() >= 2 {
        return run(m, case, data, visited);
    }
    prepare(m, case);
    let mut n = 0;
    while m.cpu.regs[15] != case.stop() {
        assert!(n < 4000);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    assert_eq!(m.cpu.regs[13], super::metadata14_tests::STACK);
    n
}
#[test]
#[ignore = "private EUR metadata14 typed getter/palette/initializer/native ownership; reduced Wine only, Linux/USA pending"]
fn european_metadata14_native_getters_live_palette_initializer_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.metadata14.as_ref().unwrap();
    let cases = cases(data);
    let mut tracer = crate::make_machine(rom.clone(), None);
    super::ui_palette_tests::initialize(&mut tracer, &mut BTreeSet::new());
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &case in &cases {
        instructions += interpreted(&mut tracer, case, data, &mut visited);
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
    assert!(cases
        .iter()
        .all(|c| !visited.iter().any(|&(pc, _)| pc == c.stop())));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("metadata14.o");
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
    let mut normal = 0;
    for &case in &cases {
        interpreted(&mut legacy, case, data, &mut BTreeSet::new());
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .metadata14_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        interpreted(&mut decoded, case, data, &mut BTreeSet::new());
        prepare(&mut native, case);
        while native.cpu.regs[15] != case.stop() {
            assert!(dispatches < 200000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original metadata14 ROM/IWRAM instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        if case.id() < 2 {
            assert_eq!(
                decoded
                    .bus
                    .rom_resources()
                    .unwrap()
                    .metadata14_stats()
                    .unwrap()
                    .source_bytes,
                before
            );
            normal += 1;
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "metadata14fullstate/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&v| v > 0));
    let native_stats = native
        .bus
        .rom_resources()
        .unwrap()
        .metadata14_stats()
        .unwrap();
    let expected_ids: BTreeSet<_> = cases
        .iter()
        .filter(|c| c.id() >= 2)
        .map(|c| c.id())
        .collect();
    assert_eq!(native_stats.records_used() as usize, expected_ids.len());
    range(&mut decoded, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in ADDRESS_BIAS..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        let edges = (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 5)
            .chain([
                SOURCE_OFFSET + 13,
                SOURCE_OFFSET + 14,
                SOURCE_OFFSET + 27,
                SOURCE_OFFSET + 28,
                SOURCE_OFFSET + 64 * 14 - 1,
                SOURCE_OFFSET + 64 * 14,
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
    let before = owner.metadata14_stats().unwrap();
    for at in [SOURCE_OFFSET - 2, END_OFFSET - 2] {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let after = owner.metadata14_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 4);
    assert_eq!(after.byte_fields - before.byte_fields, 2);
    assert_eq!(after.tail_bytes - before.tail_bytes, 2);
    assert_eq!(after.records_used(), 93);
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
    for (source_at, typed) in [(SOURCE_OFFSET + 12, 4), (END_OFFSET - 2, 2)] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.bus.write32(0x040000d4, 0x08000000 + source_at);
            m.bus.write32(0x040000d8, 0x02032000);
            m.bus.write16(0x040000dc, 2);
        }
        range(&mut decoded, None);
        let before = owner.metadata14_stats().unwrap().source_bytes;
        for m in [&mut legacy, &mut decoded] {
            m.bus.write16(0x040000de, 0x8000);
        }
        assert_eq!(
            owner.metadata14_stats().unwrap().source_bytes - before,
            typed
        );
        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions":instructions,"native_dispatches":dispatches,"compiled_byte_halfword_packed_tail_bytes":compiled,"native_record_mask":native_stats.record_mask,"native_used_ids":expected_ids,"normal_live_ids0_1_new_source_reads0":normal,"poisoned_source_bytes":1302,"original_getters_live_palette_books_copy32_and_initializer_overrides":true,"full_states_clocks_equal":true,"all1302bytes_bias_unowned0_1_record_mask_boundary_edges_widths_mirrors_real_bios":true,"mixed_raw_source_and_source_padding_crossings":[2,2],"cpuset_cpufastset":true,"dma_cross_record_typed4_and_padding_typed2":true,"allocation_tail_and_global_bounds":"unclaimed, original code remains","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
