//! Raw UI palette publication, immutable ownership and original native copies.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_PALETTE_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 16] {
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
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn spans(palettes: &UiPalettes) -> Vec<(u32, usize)> {
    palettes
        .banks
        .iter()
        .map(|b| (b.source_offset, b.colors.len() * 2))
        .collect()
}
fn ranges(m: &mut Machine, spans: &[(u32, usize)], rom: Option<&[u8]>) {
    for &(a, l) in spans {
        let a = a as usize;
        if let Some(rom) = rom {
            m.bus.rom[a..a + l].copy_from_slice(&rom[a..a + l]);
        } else {
            m.bus.rom[a..a + l].fill(0xcd);
        }
    }
}

#[test]
#[ignore = "exact EUR/e19 dataset; UI palette authentication/retention/lifecycle; Linux/USA pending"]
fn european_ui_palettes_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..15]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e19").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (19, 18));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (20, 19)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 16);
    assert_eq!(source.manifest.resources.len(), 15);
    for i in 1..15 {
        assert_eq!(files[i], old[i]);
    }
    let palettes = parse_ui_palettes(&files[15]).unwrap();
    assert_eq!(palettes.banks.len(), 21);
    assert_eq!(source.manifest.resources[14].sources.len(), 21);
    assert_eq!(
        source.manifest.resources[14]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        2656
    );
    let oracle =
        fs::read(std::env::var_os("FFTA_STATE_TEST_UI_PALETTE_SOURCE_FILE").unwrap()).unwrap();
    assert_eq!(files[15], oracle);
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(UI_RAW_PALETTES_FILE)
    {
        Err("injected UI palette publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e19-d18/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e20-d19"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(UI_RAW_PALETTES_FILE), b"partial").unwrap();
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
    for case in 0..6 {
        let mut forged = palettes.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.banks[0].colors[0] ^= 0x8000,
            1 => {
                forged.banks.pop();
            }
            2 => forged.banks.last_mut().unwrap().source_offset += 2,
            3 => manifest.resources[14].sources[0].offset += 1,
            4 => manifest.resources[14].path = "../ui-palettes.json".into(),
            _ => manifest.resources[14].sha256 = "0".repeat(64),
        }
        forged.validate().unwrap();
        let bytes = encode_ui_palettes(&forged).unwrap();
        if case != 5 {
            manifest.resources[14].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(UI_RAW_PALETTES_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(UI_RAW_PALETTES_FILE), &files[15]).unwrap();
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
    let spans = spans(&palettes);
    ranges(&mut m, &spans, None);
    for bank in &palettes.banks {
        for at in 0..bank.colors.len() * 2 {
            assert_eq!(
                m.bus.read8(0x08000000 + bank.source_offset + at as u32),
                bank.source_bytes()[at]
            );
        }
    }
    assert!(owner.ui_palette_stats().unwrap().source_bytes > 0);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,20,19],"files":16,"resources":15,"banks":21,"ranges":21,"source_bytes":2656,"all_fourteen_prior_decoded_files_and_e19_dataset_unchanged":true,"coherent_bit15_source_missing_bank_hash_path_guards_transactional":true,"publication_failure_create_reuse_repair_move_e19_retention":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
use super::ui_palette_tests::{cases, initialize, inventory, run, setup};
thread_local! {static COMPILED_BYTES:Cell<u64>=const{Cell::new(0)};}
fn used(m: *mut std::ffi::c_void) -> u64 {
    unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .ui_palette_stats()
            .unwrap()
            .source_bytes
    }
}
fn counted(before: u64, after: u64) {
    COMPILED_BYTES.with(|c| c.set(c.get() + after - before));
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let before = used(m);
    let v = (RT_API.read8)(m, a);
    counted(before, used(m));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let before = used(m);
    let v = (RT_API.read16)(m, a);
    counted(before, used(m));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let before = used(m);
    let v = (RT_API.read32)(m, a);
    counted(before, used(m));
    v
}
#[test]
#[ignore = "new EUR palette native/source/bus/RAM proof; Wine only, Linux/USA pending"]
fn european_ui_palettes_native_copy_and_bus_consumption() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let palettes = files.ui_palettes.as_ref().unwrap();
    let spans = spans(palettes);
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1).sum::<usize>()),
        (21, 2656)
    );
    let inv = inventory();
    let cases = cases();
    let mut tracer = crate::make_machine(rom.clone(), None);
    initialize(&mut tracer, &mut BTreeSet::new());
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut tracer, case, &inv, &mut visited);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&tracer.bus.iwram),
        bios: None,
    };
    let blocks = visited
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
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("ui-palettes.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
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
    COMPILED_BYTES.with(|v| v.set(0));
    let mut dispatches = 0;
    for &case in &cases {
        run(&mut legacy, case, &inv, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, &spans, None);
        }
        run(&mut decoded, case, &inv, &mut BTreeSet::new());
        setup(&mut native, case, &inv);
        let stop =
            0x08000000 + gba_core::rom_data::ui_palettes::REFERENCES[case.reference].call + 4;
        while native.cpu.regs[15] != stop {
            assert!(dispatches < 100_000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original palette instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        for m in [&mut decoded, &mut native] {
            ranges(m, &spans, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "UI palette complete state/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED_BYTES.with(Cell::get);
    assert!(compiled > 0);
    let owner = native.bus.rom_resources().unwrap();
    assert!(palettes
        .banks
        .iter()
        .all(|b| owner.ui_palette_source_bytes(b.source_offset).unwrap() > 0));
    ranges(&mut decoded, &spans, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for &(a, l) in &spans {
            for at in a - 3..a + l as u32 + 3 {
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
    // A wide read crosses the exact boundary between two new typed banks.
    // Both raw spans are already altered; each byte must use its own word owner.
    for at in 0x40fd4d..0x40fd53 {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    ranges(&mut decoded, &spans, Some(&rom));
    decoded.bus.real_bios = false;
    // Original CpuSet/FastSet bus sources consume typed colors as well.
    for swi in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a40fd90, 0x02032400, 8]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        ranges(&mut decoded, &spans, None);
        legacy.step();
        decoded.step();
        ranges(&mut decoded, &spans, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x0840fcb0);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 80);
    }
    ranges(&mut decoded, &spans, None);
    let before = decoded
        .bus
        .rom_resources()
        .unwrap()
        .ui_palette_stats()
        .unwrap()
        .source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let after = decoded
        .bus
        .rom_resources()
        .unwrap()
        .ui_palette_stats()
        .unwrap()
        .source_bytes;
    assert_eq!(after - before, 160);
    ranges(&mut decoded, &spans, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    // Palette RAM publication reads live shadow edits, not immutable ROM colors.
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        run(
            m,
            cases
                .iter()
                .copied()
                .find(|c| {
                    gba_core::rom_data::ui_palettes::REFERENCES[c.reference].call == 0x4fb80
                        && c.mode == Some(0)
                })
                .unwrap(),
            &inv,
            &mut BTreeSet::new(),
        );
        m.bus.write16(0x03003862, 0x9234);
        m.bus.write32(0x040000d4, 0x03003860);
        m.bus.write32(0x040000d8, 0x05000000);
        m.bus.write16(0x040000dc, 512);
        m.bus.write16(0x040000de, 0x8000);
        assert_eq!(m.bus.read16(0x05000002), 0x9234);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"banks":21,"calls":cases.len(),"color_mode_cases":10,"instructions":instructions,"native_dispatches":dispatches,"compiled_source_bytes":compiled,"all_banks_positive":true,"poisoned_ranges":21,"full_states_clocks_equal":true,"all_bytes_edges_widths_mirrors_real_bios_bus_cpuset_cpufastset":true,"adjacent_typed_bank_bytes_poisoned":true,"dma_typed_color_bytes":160,"live_shadow_change_reaches_palette_ram":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
