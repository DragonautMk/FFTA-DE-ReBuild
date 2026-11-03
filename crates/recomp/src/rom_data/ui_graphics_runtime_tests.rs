//! New custom UI graphics publication, ownership and original decode/upload work.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UI_GRAPHICS_MANAGED_ROOT").unwrap())
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
        .map(|b| (b.source_offset, b.encoded_source.len()))
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
#[ignore = "exact EUR/e20 dataset; UI custom graphics authentication/retention/lifecycle; Linux/USA pending"]
fn european_ui_custom_graphics_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..16]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e20").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (20, 19));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (21, 20)
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
    assert_eq!(graphics.banks.len(), 9);
    assert_eq!(source.manifest.resources[15].sources.len(), 9);
    assert_eq!(
        source.manifest.resources[15]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        57038
    );
    let oracle =
        fs::read(std::env::var_os("FFTA_STATE_TEST_UI_GRAPHICS_SOURCE_FILE").unwrap()).unwrap();
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
        .join("v1/e20-d19/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e21-d20"));
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
    for case in 0..6 {
        let mut forged = graphics.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => {
                let b = &graphics.banks[0];
                let changed = (4..b.encoded_source.len().min(128))
                    .find_map(|at| {
                        let mut bytes = b.encoded_source.clone();
                        bytes[at] ^= 1;
                        gba_core::rom_data::ui_graphics::Graphic::decode(&bytes, b.source_offset)
                            .ok()
                            .filter(|v| v != b)
                    })
                    .unwrap();
                forged.banks[0] = changed;
            }
            1 => {
                forged.banks.pop();
            }
            2 => forged.banks.last_mut().unwrap().source_offset += 4,
            3 => manifest.resources[15].sources[0].offset += 1,
            4 => manifest.resources[15].path = "../ui-graphics.json".into(),
            _ => manifest.resources[15].sha256 = "0".repeat(64),
        }
        forged.validate().unwrap();
        let bytes = encode_ui_graphics(&forged).unwrap();
        if case != 5 {
            manifest.resources[15].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
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
    for bank in &graphics.banks {
        for (at, &byte) in bank.encoded_source.iter().enumerate() {
            assert_eq!(
                m.bus.read8(0x08000000 + bank.source_offset + at as u32),
                byte
            );
        }
    }
    let stats = owner.ui_graphics_stats().unwrap();
    assert!(stats.source_bytes > 0 && stats.decoded_literal_bytes > 0);
    assert_eq!(owner.ui_graphics_used_sources().len(), 9);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,21,20],"files":17,"resources":16,"banks":9,"ranges":9,"source_bytes":57038,"decoded_bytes":84160,"all_fifteen_prior_decoded_files_and_e20_retained":true,"coherent_payload_missing_bank_source_hash_path_guards_transactional":true,"publication_failure_create_reuse_repair_move_e20_retention":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
use super::ui_graphics_tests::setup;
use gba_core::rom_data::ui_graphics::REFERENCES;
fn interpreted(m: &mut Machine, row: usize, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    setup(m, row);
    let stop = 0x08000000 + REFERENCES[row].transfer_call + 4;
    for n in 1..=2_000_000 {
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == stop {
            return n;
        }
    }
    panic!("original custom UI path did not terminate");
}
thread_local! {static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 2] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .ui_graphics_stats()
            .unwrap()
    };
    [s.source_bytes, s.decoded_literal_bytes]
}
fn count(before: [u64; 2], after: [u64; 2]) {
    COMPILED.with(|c| {
        let mut v = c.get();
        for i in 0..2 {
            v[i] += after[i] - before[i];
        }
        c.set(v);
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
#[ignore = "new EUR UI custom graphics/TinyCC/source/clock/bus; Wine only, Linux/USA pending"]
fn european_ui_custom_graphics_native_and_bus_consumption() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let graphics = files.ui_graphics.as_ref().unwrap();
    let spans = spans(graphics);
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1).sum::<usize>()),
        (9, 57038)
    );
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for row in 0..REFERENCES.len() {
        instructions += interpreted(&mut tracer, row, &mut visited);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
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
    let object = out.join("ui-graphics.o");
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
    COMPILED.with(|v| v.set([0; 2]));
    let mut dispatches = 0;
    for row in 0..REFERENCES.len() {
        interpreted(&mut legacy, row, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, &spans, None);
        }
        interpreted(&mut decoded, row, &mut BTreeSet::new());
        setup(&mut native, row);
        let stop = 0x08000000 + REFERENCES[row].transfer_call + 4;
        while native.cpu.regs[15] != stop {
            assert!(dispatches < 5_000_000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original UI graphics instruction compiled")(
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
                "custom UI full state/clock row{row}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|&v| v > 0));
    let owner = native.bus.rom_resources().unwrap();
    assert_eq!(owner.ui_graphics_used_sources().len(), 9);
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
    // New banks b70ea8/b7233c are exactly adjacent and share a crossing read.
    for at in 0xb72339..0xb72340 {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    ranges(&mut decoded, &spans, Some(&rom));
    decoded.bus.real_bios = false;
    for swi in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a3bade0, 0x02032400, 8]);
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
        m.bus.write32(0x040000d4, 0x083bade0);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 16);
    }
    ranges(&mut decoded, &spans, None);
    let before = decoded
        .bus
        .rom_resources()
        .unwrap()
        .ui_graphics_stats()
        .unwrap()
        .source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let after = decoded
        .bus
        .rom_resources()
        .unwrap()
        .ui_graphics_stats()
        .unwrap()
        .source_bytes;
    assert_eq!(after - before, 32);
    ranges(&mut decoded, &spans, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"banks":9,"calls":10,"instructions":instructions,"native_dispatches":dispatches,"compiled_source_bytes":compiled[0],"compiled_decoded_literal_bytes":compiled[1],"all_banks_positive":true,"poisoned_ranges":9,"full_states_clocks_equal":true,"original_returned_length_full_and_partial_uploads":true,"all_bytes_edges_widths_mirrors_real_bios_bus_cpuset_cpufastset":true,"adjacent_new_banks_poisoned":true,"dma_typed_source_bytes":32,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
