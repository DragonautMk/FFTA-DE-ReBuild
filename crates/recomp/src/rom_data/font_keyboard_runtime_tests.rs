//! EUR font/keyboard managed publication, typed CPU/native reads and lifecycle.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
thread_local! {static NATIVE_BYTES:Cell<[u64;5]>=const{Cell::new([0;5])};}
fn stats(m: *mut std::ffi::c_void) -> [u64; 5] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .font_keyboard_stats()
            .unwrap()
    };
    [
        s.glyph_bytes,
        s.width_bytes,
        s.keyboard_index_bytes,
        s.keyboard_program_bytes,
        s.keyboard_row_width_bytes,
    ]
}
fn count(before: [u64; 5], after: [u64; 5]) {
    NATIVE_BYTES.with(|v| {
        let mut n = v.get();
        for i in 0..5 {
            n[i] += after[i] - before[i];
        }
        v.set(n);
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = stats(m);
    let r = (RT_API.read8)(m, a);
    count(b, stats(m));
    r
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = stats(m);
    let r = (RT_API.read16)(m, a);
    count(b, stats(m));
    r
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = stats(m);
    let r = (RT_API.read32)(m, a);
    count(b, stats(m));
    r
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    args: [u32; 4],
}
fn setup(machine: &mut Machine, call: Call) {
    machine.cpu.regs[..4].copy_from_slice(&call.args);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = call.entry;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write32(STACK, 0);
    machine.bus.write32(STACK + 4, 0);
}
fn interpreted(machine: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    setup(machine, call);
    for steps in 1..=500_000 {
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!(
        "original text/font call did not return at{:08x}",
        machine.cpu.regs[15]
    );
}
fn ranges(machine: &mut Machine, sources: &[gba_core::rom_data::SourceRange], rom: Option<&[u8]>) {
    for source in sources {
        let at = source.offset as usize;
        let end = at + source.length as usize;
        if let Some(rom) = rom {
            machine.bus.rom[at..end].copy_from_slice(&rom[at..end]);
        } else {
            machine.bus.rom[at..end].fill(0xcd);
        }
    }
}

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_FONT_KEYBOARD_MANAGED_ROOT").unwrap())
}
fn names() -> [&'static str; 12] {
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
    ]
}
fn selected_dataset() -> PathBuf {
    root().join("offline dataset")
}

#[test]
#[ignore = "exact EUR ROM/new dataset; controlled host mouse scene, ordinary name-entry scene pending"]
fn european_name_keyboard_mouse_uses_typed_rows_with_changed_raw_source() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &selected_dataset()).unwrap();
    let mut mouse = gba_core::ffta_mouse::LanguageMouse::new(&rom, true);
    decoded.bus.rom[0x3ad2fe..0x3ad310].fill(0);
    for m in [&mut legacy, &mut decoded] {
        m.bus.iwram[0x27d1] = 2;
        m.bus.write16(0x04000000, 0x1f40);
        m.bus.write16(0x04000008, 0x4c03);
        m.bus.write16(0x0400000a, 0x4e02);
        m.bus.write16(0x07000004, 0x120);
        m.bus.ewram[0x166db] = 3;
    }
    for page in 0..3 {
        legacy.bus.ewram[0x166da] = page;
        decoded.bus.ewram[0x166da] = page;
        let expected = mouse.scene(&legacy.bus).unwrap();
        let actual = mouse.scene(&decoded.bus).unwrap();
        let targets = |s: gba_core::ffta_mouse::LanguageScene| {
            s.targets
                .into_iter()
                .map(|t| (t.index, t.bounds))
                .collect::<Vec<_>>()
        };
        assert_eq!(targets(expected), targets(actual));
        for cell in 0..60 {
            let mut reference = gba_core::ffta_mouse::LanguageMouse::new(&rom, true);
            assert_eq!(
                reference.select(&mut legacy.bus, 80 + cell),
                mouse.select(&mut decoded.bus, 80 + cell)
            );
            assert_eq!(
                &legacy.bus.ewram[0x166f4..0x166f6],
                &decoded.bus.ewram[0x166f4..0x166f6]
            );
        }
    }
    assert!(
        decoded
            .bus
            .rom_resources()
            .unwrap()
            .font_keyboard_stats()
            .unwrap()
            .host_row_width_reads
            > 0
    );
    decoded.bus.set_rom_resources(None).unwrap();
    assert!(
        mouse.scene(&decoded.bus).is_none(),
        "negative probe proves typed host rows were required"
    );
    fs::write(root().join("host-rows-result.json"), serde_json::to_vec_pretty(&serde_json::json!({"controlled_scene":true,"pages":3,"all_180_cells_and_targets_match":true,"raw_rows_zeroed":true,"owner_removed_negative_probe":true,"typed_host_reads_positive":true,"ordinary_name_entry":"pending","linux":"pending","usa":"pending"})).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM/e12 dataset/offline font+keyboard JSON; new Linux/USA pending"]
fn european_font_keyboard_managed_publication_authentication_repair_and_move() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old_files: Vec<_> = names()[..10]
        .iter()
        .map(|n| fs::read(old.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "prior e12").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (12, 11));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (13, 12)
    );
    assert_eq!(source.manifest.resources.len(), 11);
    let offline = selected_dataset();
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 12);
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    for i in 1..10 {
        assert_eq!(files[i], old_files[i]);
    }
    let typed =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_FONT_KEYBOARD_TYPED_ROOT").unwrap());
    for i in 10..12 {
        assert_eq!(files[i], fs::read(typed.join(names()[i])).unwrap());
    }
    // Interrupt publication before the last file; no partial generation may be selected.
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut writes = 0;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        writes += 1;
        if writes == 11 {
            Err("injected interruption".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e12-d11/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in names()[..10].iter().zip(&old_files) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e13-d12"));
    let current = first.directory.parent().unwrap().join("current");
    let current_bytes = fs::read(&current).unwrap();
    let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(fs::read(&current).unwrap(), current_bytes);
    fs::write(first.directory.join(FONT_BANK_FILE), b"partial").unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    fs::remove_file(repaired.directory.join(NAME_KEYBOARD_FILE)).unwrap();
    let repaired2 = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired2.action, "repaired");
    let mut machine = crate::make_machine(rom, None);
    let state = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    for id in [9, 10] {
        let owner = machine.bus.rom_resources().unwrap().clone();
        for case in 0..7 {
            let mut manifest = source.manifest.clone();
            let mut bytes = files[id + 1].clone();
            match case {
                0 => {
                    if id == 9 {
                        let mut value = parse_font_bank(&bytes).unwrap();
                        value.glyphs[3174].tile_words[0] ^= 1;
                        bytes = encode_font_bank(&value).unwrap();
                    } else {
                        let mut value = parse_name_keyboard(&bytes).unwrap();
                        value.cells[0][0] ^= 1;
                        bytes = serde_json::to_vec_pretty(&value).unwrap();
                    }
                    manifest.resources[id].sha256 = gba_core::ffta_rom::digest(&bytes);
                }
                1 => manifest.resources[id].path = "../outside.json".into(),
                2 => {
                    manifest.resources.remove(id);
                    manifest.coverage.remove(id);
                }
                3 => manifest.resources[id].sources[0].offset += 1,
                4 => manifest.resources[id].dependencies.clear(),
                5 => {
                    bytes.truncate(bytes.len() - 1);
                    manifest.resources[id].sha256 = gba_core::ffta_rom::digest(&bytes);
                }
                _ => manifest.decoder_revision -= 1,
            }
            fs::write(offline.join(names()[id + 1]), bytes).unwrap();
            fs::write(
                offline.join(MANIFEST_FILE),
                toml::to_string_pretty(&manifest).unwrap(),
            )
            .unwrap();
            assert!(attach_from_directory(&mut machine, &offline).is_err());
            assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
            assert_eq!(machine.save_state().unwrap(), state);
        }
        fs::write(offline.join(names()[id + 1]), &files[id + 1]).unwrap();
        fs::write(offline.join(MANIFEST_FILE), &files[0]).unwrap();
        let mut whitespace = files[id + 1].clone();
        whitespace.extend_from_slice(b"\n \n");
        let mut manifest = source.manifest.clone();
        manifest.resources[id].sha256 = gba_core::ffta_rom::digest(&whitespace);
        fs::write(offline.join(names()[id + 1]), whitespace).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        attach_from_directory(&mut machine, &offline).unwrap();
        assert_ne!(
            machine.bus.rom_resources().unwrap().dataset_sha256(),
            owner.dataset_sha256()
        );
        fs::File::options()
            .write(true)
            .open(offline.join(names()[id + 1]))
            .unwrap()
            .set_len(if id == 9 {
                MAX_FONT_BANK_FILE_BYTES + 1
            } else {
                MAX_NAME_KEYBOARD_FILE_BYTES + 1
            })
            .unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        fs::write(offline.join(names()[id + 1]), &files[id + 1]).unwrap();
        fs::write(offline.join(MANIFEST_FILE), &files[0]).unwrap();
        attach_from_directory(&mut machine, &offline).unwrap();
    }
    let relative = repaired2
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = root.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    let moved_result = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(moved_result.action, "reused");
    assert_eq!(moved_result.directory, moved.join(relative));
    for (n, b) in names()[..10].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(old.join(n)).unwrap(), *b);
    }
    fs::write(root.join("managed-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"files":12,"resources":11,"revision":[1,13,12],"source_ranges":[2,5],"offline_json_exact":true,"prior_e12_and_resource_bytes_unchanged":true,"atomic_failure_create_reuse_repair_move":true,"forged_edits_paths_metadata_rejected":true,"identity_all_files":true,"activation_transactional":true,"linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM/new dataset/retained initialized slot/TinyCC; Wine EUR only"]
fn european_font_keyboard_original_native_provenance_and_bus_edges() {
    let root = root().join("native");
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = selected_dataset();
    let files = read_dataset(&dataset).unwrap();
    let sources: Vec<_> = files.manifest.resources[9..]
        .iter()
        .flat_map(|r| r.sources.iter().cloned())
        .collect();
    let mut tracer = crate::make_machine(rom.clone(), None);
    crate::save_state::load(
        &mut tracer,
        Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    let start = tracer.save_state().unwrap();
    let iwram = tracer.bus.iwram.clone();
    let calls: Vec<_> = (0..3)
        .map(|page| Call {
            entry: 0x0812e900,
            args: [0x02028000, page, 0, 0],
        })
        .chain([
            Call {
                entry: 0x080135a4,
                args: [0x02030010, 3174, 0, 0],
            },
            Call {
                entry: 0x08013bb4,
                args: [3174, 0, 0, 0],
            },
        ])
        .collect();
    let initialize = |m: &mut Machine| {
        m.bus.write32(0x02028304, 0x02029010);
        m.bus.write32(0x02028308, 0x02029410);
        m.bus.write32(0x0202830c, 0x02029810);
        m.bus.write8(0x020020e3, 0);
    };
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    initialize(&mut tracer);
    for &call in &calls {
        instructions += interpreted(&mut tracer, call, &mut visited);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&iwram),
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
    fs::create_dir_all(&root).unwrap();
    let object = root.join("font-keyboard.o");
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
        attach_from_directory(m, &dataset).unwrap();
    }
    for m in [&mut legacy, &mut decoded, &mut native] {
        m.load_state(&start).unwrap();
        initialize(m);
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    NATIVE_BYTES.with(|v| v.set([0; 5]));
    let mut dispatches = 0;
    for &call in &calls {
        interpreted(&mut legacy, call, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, &sources, None);
        }
        interpreted(&mut decoded, call, &mut BTreeSet::new());
        setup(&mut native, call);
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 500000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        for m in [&mut decoded, &mut native] {
            ranges(m, &sources, Some(&rom));
            assert_eq!(
                legacy.save_state().unwrap(),
                m.save_state().unwrap(),
                "full state/clock call{:08x}",
                call.entry
            );
        }
    }
    let used = NATIVE_BYTES.with(Cell::get);
    assert!(used[..4].iter().all(|&v| v > 0));
    // Exercise every source edge and width, all ROM mirrors and native bus APIs
    // with the old source altered; unchanged adjacent bytes must be preserved.
    ranges(&mut decoded, &sources, None);
    for s in &sources {
        for at in [
            s.offset - 3,
            s.offset,
            s.offset + s.length - 3,
            s.offset + s.length,
        ] {
            for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                let a = mirror + at;
                assert_eq!(decoded.bus.read8(a), rom[at as usize]);
                let aligned = at & !1;
                assert_eq!(
                    decoded.bus.read16(mirror + aligned),
                    u16::from_le_bytes(
                        rom[aligned as usize..aligned as usize + 2]
                            .try_into()
                            .unwrap()
                    )
                );
                let aligned = at & !3;
                assert_eq!(
                    (RT_API.read32)(
                        &mut decoded as *mut Machine as *mut std::ffi::c_void,
                        mirror + aligned
                    ),
                    u32::from_le_bytes(
                        rom[aligned as usize..aligned as usize + 4]
                            .try_into()
                            .unwrap()
                    )
                );
            }
        }
    }
    assert!(
        decoded
            .bus
            .rom_resources()
            .unwrap()
            .font_keyboard_stats()
            .unwrap()
            .keyboard_row_width_bytes
            > 0
    );
    ranges(&mut decoded, &sources, Some(&rom));
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"calls":calls.len(),"original_instructions":instructions,"native_dispatches":dispatches,"compiled_source_bytes":used,"kinds":["glyph","metric","index","program","row_width"],"poisoned_ranges":7,"states_and_clocks_equal":true,"all_edges_widths_mirrors_native_api":true,"linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM/new dataset; temporary copied files only"]
fn european_font_keyboard_owner_survives_slots_undo_backup_restart_and_missing_files() {
    let temp = tempfile::tempdir().unwrap();
    let dataset = temp.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for n in names() {
        fs::copy(selected_dataset().join(n), dataset.join(n)).unwrap();
    }
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    fs::rename(&dataset, temp.path().join("unavailable")).unwrap();
    assert!(attach_from_directory(&mut m, &dataset).is_err());
    assert_eq!(m.save_state().unwrap(), boot);
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    let save = temp.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &m.bus.rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 99;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    assert_eq!(m.cpu.regs[4], 0);
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
    assert_eq!(fs::read(&save).unwrap(), vec![9; len]);
    let source = Extraction::from_rom(&rom).unwrap();
    let sources: Vec<_> = source.manifest.resources[9..]
        .iter()
        .flat_map(|r| r.sources.iter().cloned())
        .collect();
    ranges(&mut m, &sources, None);
    for s in sources {
        assert_eq!(m.bus.read8(0x08000000 + s.offset), rom[s.offset as usize]);
    }
    let stats = owner.font_keyboard_stats().unwrap();
    assert!(
        stats.glyph_bytes > 0
            && stats.width_bytes > 0
            && stats.keyboard_index_bytes > 0
            && stats.keyboard_program_bytes > 0
            && stats.keyboard_row_width_bytes > 0
    );
    fs::write(root().join("lifecycle-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"temporary_files_unavailable":true,"failed_attachment_transactional":true,"load_undo_backup_restart_owner_retained":true,"all_kinds_positive":true,"linux":"pending","usa":"pending"})).unwrap()).unwrap();
}
