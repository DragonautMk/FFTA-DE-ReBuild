//! Packed-pair ownership while original row, slot and mode instructions run.
use super::metadata12_tests::{address, execute, run, setup, Case, RETURN};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::metadata12::{BankId, Metadata12, END_OFFSET, SECONDARY_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA12_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 27] {
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
#[ignore = "private EUR e31 dataset; metadata12 publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_metadata12_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..26]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e31").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (31, 30));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (32, 31)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 27);
    assert_eq!(source.manifest.resources.len(), 26);
    for i in 1..26 {
        assert_eq!(files[i], old[i]);
    }
    let items = parse_metadata12(&files[26]).unwrap();
    assert_eq!(items.source_bytes().len(), 1296);
    assert_eq!(source.manifest.resources[25].sources.len(), 2);
    assert_eq!(source.manifest.resources[25].sources[0].length, 1104);
    assert_eq!(
        files[26],
        fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA12_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(
        (items.banks[0].records.len(), items.banks[1].records.len()),
        (92, 16)
    );
    assert_eq!(source.manifest.resources[25].sources[1].length, 192);
    assert!(source.manifest.resources[25].dependencies.is_empty());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(
        &interrupted,
        &mut source,
        &mut |p, b| if p.ends_with(METADATA12_FILE) {
            Err("injected packed-pair publication failure".into())
        } else {
            write_synced(p, b)
        }
    )
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e31-d30/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e32-d31"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(METADATA12_FILE), b"partial").unwrap();
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
            0 => forged.banks[0].records[0].groups[0][0] ^= 1,
            1 => forged.banks[1].records[15].groups[11][3] ^= 2,
            2 => forged.banks[0].records[91].groups[10][2] ^= 3,
            3 => forged.banks[1].records[0].groups[11][0] ^= 1,
            4 => manifest.resources[25].sources[0].length -= 1,
            5 => manifest.resources[25]
                .dependencies
                .push(METADATA14_ID.into()),
            6 => manifest.resources[25].path = "../linked-metadata12.json".into(),
            7 => manifest.resources[25].sources[0].role = "metadata12/unknown".into(),
            8 => manifest.resources[25].sources[1].role = "metadata12/unknown".into(),
            _ => manifest.resources[25].sources[1].offset += 1,
        }

        forged.validate().unwrap();
        let bytes = encode_metadata12(&forged).unwrap();
        manifest.resources[25].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(METADATA12_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(METADATA12_FILE), &files[26]).unwrap();
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
    let s = owner.metadata12_stats().unwrap();
    assert!(s.primary_bytes > 0 && s.secondary_bytes > 0);
    assert_eq!(s.records_used(), 108);

    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,32,31],"files":27,"resources":26,"records":108,"source_bytes":1296,"all_twenty_five_prior_decoded_files_and_e31_generation_unchanged":true,"publication_failure_create_reuse_repair_move":true,"coherent_pair_unused_tail_source_unexpected_dependency_path_both_roles_guards":true,"no_unproved_semantic_dependency":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
thread_local! {static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 2] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .metadata12_stats()
            .unwrap()
    };
    [s.primary_bytes, s.secondary_bytes]
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
    let mut out = vec![];
    for bank in [BankId::Primary, BankId::Secondary] {
        for row in [0, bank.record_count() as u32 - 1] {
            for slot in [0, 3, 4, 43, 47] {
                for mode in [0, 0x101] {
                    out.push(Case {
                        bank,
                        row,
                        slot,
                        mode,
                    });
                }
            }
        }
        for mode in [0xff, 0x100] {
            out.push(Case {
                bank,
                row: bank.record_count() as u32 - 1,
                slot: 43,
                mode,
            });
        }
    }
    out.extend([
        Case {
            bank: BankId::Primary,
            row: 92,
            slot: 0,
            mode: 0,
        },
        Case {
            bank: BankId::Primary,
            row: 256,
            slot: 0,
            mode: 1,
        },
        Case {
            bank: BankId::Secondary,
            row: 16,
            slot: 0,
            mode: 0,
        },
        Case {
            bank: BankId::Primary,
            row: 0x40000000,
            slot: 43,
            mode: 0x101,
        },
        Case {
            bank: BankId::Primary,
            row: 0,
            slot: u32::MAX,
            mode: 0,
        },
    ]);
    out
}
fn interpreted(
    m: &mut Machine,
    c: Case,
    data: &Metadata12,
    rom: &[u8],
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    if data
        .record(c.bank, c.row)
        .and_then(|r| r.flag(c.slot, c.mode))
        .is_some()
    {
        run(m, c, data, visited)
    } else {
        let (a, bit) = address(c);
        execute(
            m,
            c,
            ((rom[(a - 0x08000000) as usize] >> bit) & 1) != 0,
            visited,
        )
    }
}
#[test]
#[ignore = "private EUR packed pair native ownership and bus; reduced Wine only, Linux/USA pending"]
fn european_metadata12_native_packed_pair_readers_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.metadata12.as_ref().unwrap();
    let cases = cases();
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &case in &cases {
        instructions += interpreted(&mut tracer, case, data, &rom, &mut visited);
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
    assert!(!visited.iter().any(|&(pc, _)| pc == RETURN));
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("metadata12.o");
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
    let mut unowned = 0;
    for &case in &cases {
        interpreted(&mut legacy, case, data, &rom, &mut BTreeSet::new());
        let before = decoded
            .bus
            .rom_resources()
            .unwrap()
            .metadata12_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, None);
        }
        interpreted(&mut decoded, case, data, &rom, &mut BTreeSet::new());
        setup(&mut native, case);
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 20000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original packed pair instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let (a, _) = address(case);
        if !(0x08000000 + SOURCE_OFFSET..0x08000000 + END_OFFSET).contains(&a) {
            assert_eq!(
                decoded
                    .bus
                    .rom_resources()
                    .unwrap()
                    .metadata12_stats()
                    .unwrap()
                    .source_bytes,
                before
            );
            unowned += 1;
        }
        for m in [&mut decoded, &mut native] {
            range(m, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "packed pair fullstate/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&v| v > 0));
    assert_eq!(unowned, 3);
    let native_stats = native
        .bus
        .rom_resources()
        .unwrap()
        .metadata12_stats()
        .unwrap();
    assert_eq!(native_stats.records_used(), 4);
    range(&mut decoded, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for at in SOURCE_OFFSET..END_OFFSET {
            assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
        }
        let edges = (SOURCE_OFFSET - 3..SOURCE_OFFSET + 4)
            .chain(SECONDARY_OFFSET - 3..SECONDARY_OFFSET + 4)
            .chain(END_OFFSET - 3..END_OFFSET + 5)
            .chain([
                SOURCE_OFFSET + 11,
                SOURCE_OFFSET + 12,
                SOURCE_OFFSET + 64 * 12 - 1,
                SOURCE_OFFSET + 64 * 12,
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
    let before = owner.metadata12_stats().unwrap();
    for at in [SOURCE_OFFSET - 2, SECONDARY_OFFSET - 2, END_OFFSET - 2] {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let after = owner.metadata12_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 8);
    assert_eq!(after.primary_bytes - before.primary_bytes, 4);
    assert_eq!(after.secondary_bytes - before.secondary_bytes, 4);
    assert_eq!(after.records_used(), 108);
    range(&mut decoded, Some(&rom));
    decoded.bus.real_bios = false;
    for source_at in [SOURCE_OFFSET, SECONDARY_OFFSET] {
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
    for (at, typed) in [(SECONDARY_OFFSET - 2, 4), (END_OFFSET - 2, 2)] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.bus.write32(0x040000d4, 0x08000000 + at);
            m.bus.write32(0x040000d8, 0x02032000);
            m.bus.write16(0x040000dc, 2);
        }
        range(&mut decoded, None);
        let before = owner.metadata12_stats().unwrap().source_bytes;
        for m in [&mut legacy, &mut decoded] {
            m.bus.write16(0x040000de, 0x8000);
        }
        assert_eq!(
            owner.metadata12_stats().unwrap().source_bytes - before,
            typed
        );
        range(&mut decoded, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions_coverage":instructions,"native_dispatches":dispatches,"compiled_primary_secondary_bytes":compiled,"native_record_mask":native_stats.record_mask,"native_used_records":native_stats.records_used(),"unowned_original_paths_new_reads0":unowned,"poisoned_source_bytes":1296,"full_states_clocks_equal":true,"original_row32_slot_wrap_mode_u8":true,"all1296bytes_edges_bankboundary_maskboundary_widths_mirrors_bios":true,"typed_raw_and_crossbank_words":[2,4,2],"both_banks_cpuset_cpufastset":true,"dma_crossbank4typed_and_globalboundary2typed":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
