//! New language pointers/payloads/shared OBJ links; preserve closed e21 proof.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_LOCALIZED_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 17] {
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
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn spans(graphics: &UiGraphics) -> Vec<(u32, usize)> {
    graphics
        .banks
        .iter()
        .filter(|b| {
            gba_core::rom_data::ui_graphics::LOCALIZED_BANK_ROOTS.contains(&b.source_offset)
        })
        .map(|b| (b.source_offset, b.encoded_source.len()))
        .chain(
            graphics
                .selectors
                .iter()
                .flat_map(|s| s.pointers.iter().map(|p| (p.source_offset, 4))),
        )
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
#[ignore = "exact EUR/e21 dataset; localized UI graphics authentication/retention/lifecycle; Linux/USA pending"]
fn european_localized_ui_graphics_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e21").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (21, 20));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (22, 21)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 17);
    assert_eq!(source.manifest.resources.len(), 16);
    for i in 1..16 {
        assert_eq!(files[i], old[i]);
    }
    let graphics = parse_ui_graphics(&files[16]).unwrap();
    assert_eq!(
        (
            graphics.banks.len(),
            graphics.selectors.len(),
            graphics.shared_obj_banks.len()
        ),
        (16, 2, 2)
    );
    let previous = parse_ui_graphics(&old[16]).unwrap();
    assert_eq!(previous.banks.len(), 9);
    assert!(previous.banks.iter().all(|b| graphics.banks.contains(b)));
    assert_eq!(source.manifest.resources[15].sources.len(), 26);
    assert_eq!(
        source.manifest.resources[15]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        71474
    );
    let oracle =
        fs::read(std::env::var_os("FFTA_STATE_TEST_UI_LOCALIZED_SOURCE_FILE").unwrap()).unwrap();
    assert_eq!(files[16], oracle);
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(UI_CUSTOM_GRAPHICS_FILE)
    {
        Err("injected custom UI publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e21-d20/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e22-d21"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(UI_CUSTOM_GRAPHICS_FILE), b"partial").unwrap();
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
        let mut forged = graphics.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => {
                let at = gba_core::rom_data::ui_graphics::LOCALIZED_BANK_ROOTS[0];
                let b = forged
                    .banks
                    .iter_mut()
                    .find(|b| b.source_offset == at)
                    .unwrap();
                let changed = (4..b.encoded_source.len().min(128))
                    .find_map(|at| {
                        let mut bytes = b.encoded_source.clone();
                        bytes[at] ^= 1;
                        gba_core::rom_data::ui_graphics::Graphic::decode(&bytes, b.source_offset)
                            .ok()
                            .filter(|v| v != b)
                    })
                    .unwrap();
                *b = changed;
            }
            1 => forged.selectors[0].pointers.swap(0, 1),
            2 => forged.selectors[0].fallback_variant = None,
            3 => {
                forged.selectors.clear();
                forged.shared_obj_banks.clear();
            }
            4 => {
                forged.selectors[1].pointers[4].bank_offset = 0xb7f874;
                forged.shared_obj_banks.retain(|&v| v != 0xb80140);
            }
            5 => manifest.resources[15].sources[0].offset += 1,
            6 => manifest.resources[15]
                .dependencies
                .retain(|v| v != OBJ_CUSTOM_GRAPHICS_ID),
            _ => manifest.resources[15].path = "../ui-graphics.json".into(),
        }
        forged.validate().unwrap();
        let bytes = encode_ui_graphics(&forged).unwrap();
        manifest.resources[15].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(UI_CUSTOM_GRAPHICS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(UI_CUSTOM_GRAPHICS_FILE), &files[16]).unwrap();
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
    let spans = spans(&graphics);
    ranges(&mut m, &spans, None);
    for selector in &graphics.selectors {
        for pointer in &selector.pointers {
            assert_eq!(
                m.bus.read32(0x08000000 + pointer.source_offset),
                0x08000000 + pointer.bank_offset
            );
        }
    }
    for bank in graphics.banks.iter().filter(|b| {
        gba_core::rom_data::ui_graphics::LOCALIZED_BANK_ROOTS.contains(&b.source_offset)
    }) {
        for (at, &byte) in bank.encoded_source.iter().enumerate() {
            assert_eq!(
                m.bus.read8(0x08000000 + bank.source_offset + at as u32),
                byte
            );
        }
    }
    let stats = owner.ui_graphics_stats().unwrap();
    assert!(
        stats.selector_bytes > 0
            && stats.localized_source_bytes > 0
            && stats.localized_literal_bytes > 0
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,22,21],"files":17,"resources":16,"banks":16,"new_banks":7,"selectors":2,"shared_obj_banks":2,"new_ranges":17,"new_source_bytes":14436,"old_nine_bank_records_and_other_decoded_files_unchanged":true,"coherent_payload_selector_reorder_fallback_shared_owner_dependency_source_path_guards_transactional":true,"publication_failure_create_reuse_repair_move_e21_retention":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
use super::ui_localized_graphics_tests::{cases, run, setup, shared, stop};
thread_local! {static COMPILED:Cell<[u64;3]>=const{Cell::new([0;3])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 3] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .ui_graphics_stats()
            .unwrap()
    };
    [
        s.selector_bytes,
        s.localized_source_bytes,
        s.localized_literal_bytes,
    ]
}
fn count(b: [u64; 3], a: [u64; 3]) {
    COMPILED.with(|v| {
        let mut n = v.get();
        for i in 0..3 {
            n[i] += a[i] - b[i];
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
#[test]
#[ignore = "new EUR localized UI pointer/new-payload/native/shared-owner proof; Wine only, Linux/USA pending"]
fn european_localized_ui_graphics_native_selectors_payloads_and_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let graphics = files.ui_graphics.as_ref().unwrap();
    let obj = shared();
    assert_eq!(files.obj_graphics.as_ref().unwrap(), &obj);
    let spans = spans(graphics);
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1).sum::<usize>()),
        (17, 14436)
    );
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let cases = cases();
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut tracer, case, graphics, &obj, &mut visited);
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
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("ui-localized.o");
    let selector_blocks = blocks
        .iter()
        .filter(|b| ![0x0808ed9c, 0x08143d44, 0x08143db4].contains(&b.addr))
        .cloned()
        .collect();
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    // Other producer cases include these decoder entry PCs. Exclude them from
    // this selector-only table so native chaining cannot cross a fixture stop.
    let selector_object = out.join("ui-selectors.o");
    recomp_core::tcc_memory::build(
        &Analysis {
            blocks: selector_blocks,
        },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &selector_object,
        &mut |_| {},
    )
    .unwrap();
    let (_selector_library, selector_table) = crate::load_native(&selector_object).unwrap();
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
    for &case in &cases {
        run(&mut legacy, case, graphics, &obj, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, &spans, None);
        }
        run(&mut decoded, case, graphics, &obj, &mut BTreeSet::new());
        setup(&mut native, case, &obj);
        let end = stop(case);
        let table = if [0x0808ed9c, 0x08143d44, 0x08143db4].contains(&end) {
            &selector_table
        } else {
            &table
        };
        while native.cpu.regs[15] != end {
            assert!(dispatches < 2_000_000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .unwrap_or_else(|| panic!("original localized UI instruction missing at {:08x} thumb={} case={case:?} end={end:08x}", native.cpu.regs[15], native.cpu.thumb()))(
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
                "localized UI full state/clock {case:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&v| v > 0));
    let used = native
        .bus
        .rom_resources()
        .unwrap()
        .ui_graphics_used_sources();
    assert!(gba_core::rom_data::ui_graphics::LOCALIZED_BANK_ROOTS
        .iter()
        .all(|at| used.iter().any(|(p, n)| p == at && *n > 0)));
    ranges(&mut decoded, &spans, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for &(a, l) in &spans {
            for at in a..a + l as u32 {
                assert_eq!(decoded.bus.read8(0x08000000 + at), rom[at as usize]);
            }
            for at in (a - 3..a + 4)
                .chain(a + l as u32 - 3..a + l as u32 + 3)
                .chain([a + l as u32 / 2])
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
    }
    ranges(&mut decoded, &spans, Some(&rom));
    decoded.bus.real_bios = false;
    for swi in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a44d22c, 0x02032400, 8]);
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
        m.bus.write32(0x040000d4, 0x0808eebc);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    ranges(&mut decoded, &spans, None);
    let before = decoded
        .bus
        .rom_resources()
        .unwrap()
        .ui_graphics_stats()
        .unwrap()
        .selector_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let after = decoded
        .bus
        .rom_resources()
        .unwrap()
        .ui_graphics_stats()
        .unwrap()
        .selector_bytes;
    assert_eq!(after - before, 4);
    ranges(&mut decoded, &spans, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"selector_paths":14,"new_producer_paths":7,"isolated_shared_tails":2,"instructions":instructions,"native_dispatches":dispatches,"compiled_selector_bytes":compiled[0],"compiled_localized_source_bytes":compiled[1],"compiled_localized_literal_bytes":compiled[2],"all_seven_new_banks_positive":true,"poisoned_new_ranges":17,"full_states_clocks_equal":true,"all_new_bytes_edges_widths_mirrors_real_bios_bus_cpuset_cpufastset":true,"dma_selector_bytes":4,"fallback_english_skip_and_shared_split_owners_preserved":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
