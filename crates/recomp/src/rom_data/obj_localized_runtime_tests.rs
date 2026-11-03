//! Five new EUR language-selected banks; retain all prior bank/static proofs.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const TASK: u32 = 0x02028050;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
const OBJECT: u32 = 0xb4deb4;
const SEQUENCE: u32 = 0x3955b0;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_LOCALIZED_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 14] {
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
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn spans(banks: &ObjAnimationBanks) -> Vec<(u32, usize)> {
    let t = &banks.selector_tables[0];
    let mut out: Vec<_> = banks
        .banks
        .iter()
        .filter(|b| t.object_roots.contains(&b.object_root))
        .flat_map(|b| b.spans())
        .map(|(a, l, _)| (a, l))
        .collect();
    out.push((t.object_table_offset, t.object_roots.len() * 4));
    out.push((t.sequence_table_offset, t.sequence_roots.len() * 4));
    out
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
#[ignore = "exact EUR/e17 dataset; new localized bank authentication/retention/lifecycle; Linux/USA pending"]
fn european_localized_obj_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e17").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (17, 16));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (18, 17)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 14);
    assert_eq!(source.manifest.resources.len(), 13);
    for i in 1..13 {
        assert_eq!(files[i], old[i]);
    }
    let banks = parse_obj_animations(&files[13]).unwrap();
    let previous = parse_obj_animations(&old[13]).unwrap();
    assert_eq!((previous.banks.len(), banks.banks.len()), (11, 16));
    for b in previous.banks {
        assert!(banks.banks.contains(&b));
    }
    assert_eq!(source.manifest.resources[12].sources.len(), 456);
    assert_eq!(
        source.manifest.resources[12]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        9054
    );
    assert_eq!(spans(&banks).len(), 166);
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(OBJ_ANIMATION_BANKS_FILE)
    {
        Err("injected localized OBJ publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e17-d16/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e18-d17"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(OBJ_ANIMATION_BANKS_FILE),
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
    for case in 0..3 {
        let mut forged = banks.clone();
        match case {
            0 => {
                forged
                    .banks
                    .iter_mut()
                    .find(|b| b.object_root == OBJECT)
                    .unwrap()
                    .lists[1]
                    .objects[0]
                    .tile ^= 1
            }
            1 => {
                forged.selector_tables[0].object_roots.swap(0, 1);
                forged.selector_tables[0].sequence_roots.swap(0, 1);
            }
            _ => forged.selector_tables.clear(),
        }
        forged.validate().unwrap();
        let bytes = encode_obj_animations(&forged).unwrap();
        let mut manifest = source.manifest.clone();
        manifest.resources[12].sha256 = gba_core::ffta_rom::digest(&bytes);
        fs::write(dataset().join(OBJ_ANIMATION_BANKS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(OBJ_ANIMATION_BANKS_FILE), &files[13]).unwrap();
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
    let spans = spans(&banks);
    ranges(&mut m, &spans, None);
    let t = &banks.selector_tables[0];
    for language in 0..5 {
        let (o, s) = t.select(language).unwrap();
        assert_eq!(
            m.bus
                .read32(0x08000000 + t.object_table_offset + u32::from(language) * 4),
            0x08000000 + o
        );
        assert_eq!(
            m.bus
                .read32(0x08000000 + t.sequence_table_offset + u32::from(language) * 4),
            0x08000000 + s
        );
        assert_eq!(
            m.bus.read32(0x08000000 + o),
            u32::from_le_bytes(rom[o as usize..o as usize + 4].try_into().unwrap())
        );
    }
    let stats = owner.obj_animation_stats().unwrap();
    assert!(stats.localized_selector_bytes > 0 && stats.localized_source_bytes > 0);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,18,17],"files":14,"resources":13,"banks":16,"new_banks":5,"new_lists":149,"new_objects":574,"new_sequences":5,"new_frames":144,"new_ranges":166,"new_source_bytes":4984,"all_prior_eleven_bank_records_and_other_decoded_files_unchanged":true,"coherent_selector_and_list_edits_missing_tables_rejected_transactionally":true,"publication_failure_create_reuse_repair_move_e17_retention":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[derive(Clone, Copy)]
struct Call {
    language: u8,
    reference: usize,
    body: bool,
    last: bool,
}
fn setup(m: &mut Machine, call: Call) -> u32 {
    m.cpu.regs[13] = STACK;
    m.cpu.cpsr = 0xbf;
    m.bus.write8(0x02001f6e, call.language);
    if call.body {
        m.cpu.regs[5] = TASK;
        m.cpu.regs[15] = 0x0813f814;
        if call.last {
            let n = [35, 22, 30, 22, 35][call.language as usize];
            m.bus.write16(TASK + 8, n - 1);
            m.bus.write8(TASK + 15, 255);
        }
        m.bus.write8(0x03000028, 0);
        let mode = m.bus.read8(TASK + 12);
        let (count, buffer) = if mode == 1 {
            (0x03002c50, 0x03002c58)
        } else {
            (0x03000020, 0x03000030)
        };
        m.bus.write32(count, 0);
        for a in 0..1024 {
            m.bus.write8(buffer + a, 0x5a);
        }
        0x0813f9a6
    } else {
        for a in 0..60 {
            m.bus.write8(TASK - 16 + a, 0x5a);
        }
        for a in 0..28 {
            m.bus.write8(TASK + a, 0);
        }
        let reference = &gba_core::rom_data::obj_animations::SELECTOR_REFERENCES[call.reference];
        m.cpu.regs[..4].copy_from_slice(&[TASK - 84, 0, 0, 100]);
        m.cpu.regs[4] = 0x02001f6e;
        m.cpu.regs[15] = 0x08000000 + reference.object_reader;
        for (i, v) in [70, 0, 10, 0, 0, 0].into_iter().enumerate() {
            m.bus.write32(STACK + i as u32 * 4, v);
        }
        0x08000000 + reference.call + 4
    }
}
fn interpreted(m: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    let stop = setup(m, call);
    for n in 1..=20_000 {
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == stop {
            assert_eq!(m.cpu.regs[13], STACK);
            return n;
        }
    }
    panic!("localized selector/constructor/body did not reach original edge");
}
thread_local! { static NATIVE_BYTES:Cell<[u64;3]>=const{Cell::new([0;3])}; }
fn used(m: *mut std::ffi::c_void) -> [u64; 3] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .obj_animation_stats()
            .unwrap()
    };
    [
        s.localized_selector_bytes,
        s.localized_source_bytes,
        s.source_bytes,
    ]
}
fn count(before: [u64; 3], after: [u64; 3]) {
    NATIVE_BYTES.with(|c| {
        let mut v = c.get();
        for i in 0..3 {
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
fn draw_setup(m: &mut Machine, offset: u32) {
    m.cpu.regs[..4].copy_from_slice(&[0x08000000 + offset, 100, 70, 0]);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08001500;
    m.cpu.cpsr = 0xbf;
    for (i, v) in [7, 2, 3, 0x02030010, 128, 0].into_iter().enumerate() {
        m.bus.write32(STACK + i as u32 * 4, v);
    }
    for a in 0..1024 {
        m.bus.write8(0x02030010 + a, 0x5a);
    }
}

#[test]
#[ignore = "new EUR localized selectors/banks/TinyCC; native provenance/clock/host/bus; Linux/USA pending"]
fn european_localized_obj_native_host_and_bus_consumption() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let banks = files.obj_animations.as_ref().unwrap();
    let spans = spans(banks);
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1).sum::<usize>()),
        (166, 4984)
    );
    let mut calls = Vec::new();
    for language in 0..5 {
        for reference in 0..3 {
            calls.push(Call {
                language,
                reference,
                body: false,
                last: false,
            });
            if reference < 2 {
                calls.push(Call {
                    language,
                    reference,
                    body: true,
                    last: false,
                });
                if reference == 0 {
                    calls.push(Call {
                        language,
                        reference,
                        body: true,
                        last: true,
                    });
                }
            }
        }
    }
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &call in &calls {
        instructions += interpreted(&mut tracer, call, &mut visited);
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
    let object = out.join("localized.o");
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
    NATIVE_BYTES.with(|v| v.set([0; 3]));
    let mut dispatches = 0;
    for &call in &calls {
        interpreted(&mut legacy, call, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, &spans, None);
        }
        interpreted(&mut decoded, call, &mut BTreeSet::new());
        let stop = setup(&mut native, call);
        while native.cpu.regs[15] != stop {
            assert!(dispatches < 100_000);
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
            ranges(m, &spans, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                legacy.save_state().unwrap(),
                "localized complete state/clock"
            );
        }
    }
    let compiled = NATIVE_BYTES.with(Cell::get);
    assert!(compiled.iter().all(|&n| n > 0));
    let stats = native
        .bus
        .rom_resources()
        .unwrap()
        .obj_animation_stats()
        .unwrap();
    assert!(
        stats.index_bytes > 0
            && stats.sequence_header_bytes > 0
            && stats.count_bytes > 0
            && stats.attribute_bytes > 0
            && stats.frame_list_bytes > 0
            && stats.duration_bytes > 0
    );
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
    ranges(&mut decoded, &spans, Some(&rom));
    decoded.bus.real_bios = false;
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + SEQUENCE);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 20);
    }
    ranges(&mut decoded, &spans, None);
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    ranges(&mut decoded, &spans, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for m in [&mut legacy, &mut decoded] {
        m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + SEQUENCE, 0x02032400, 20]);
        m.cpu.regs[15] = 0x02000000;
        m.cpu.cpsr = 0xbf;
        m.bus.write16(0x02000000, 0xdf0b);
    }
    ranges(&mut decoded, &spans, None);
    legacy.step();
    decoded.step();
    ranges(&mut decoded, &spans, Some(&rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    gba_core::mem::set_extra(120);
    let mut host_checks = 0;
    for object_root in &banks.selector_tables[0].object_roots {
        let offset = banks
            .banks
            .iter()
            .find(|b| b.object_root == *object_root)
            .unwrap()
            .lists[1]
            .source_offset;
        let mut old = crate::make_machine(rom.clone(), None);
        let mut new = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut new, &dataset()).unwrap();
        ranges(&mut new, &spans, None);
        for m in [&mut old, &mut new] {
            draw_setup(m, offset);
            m.bus.ffta_sprites.enabled = true;
            m.ffta_sprite_entry();
        }
        let mut reference = crate::make_machine(rom.clone(), None);
        draw_setup(&mut reference, offset);
        for _ in 0..20_000 {
            reference.step();
            if reference.cpu.regs[15] == RETURN {
                break;
            }
        }
        assert_eq!(reference.cpu.regs[15], RETURN);
        let oam: Vec<_> = (0..1024)
            .map(|a| reference.bus.read8(0x02030010 + a))
            .collect();
        for m in [&mut old, &mut new] {
            m.bus
                .ffta_sprites
                .publish(0x02030010, 0x07000000, 1024, true, &oam);
        }
        assert!(old.bus.ffta_sprites.active.is_some());
        assert_eq!(
            format!("{:?}", old.bus.ffta_sprites.active),
            format!("{:?}", new.bus.ffta_sprites.active)
        );
        assert!(
            new.bus
                .rom_resources()
                .unwrap()
                .obj_animation_stats()
                .unwrap()
                .localized_host_reads
                > 0
        );
        host_checks += 1;
    }
    gba_core::mem::set_extra(0);
    report(
        "native-result.json",
        serde_json::json!({"calls":calls.len(),"languages":5,"all_three_original_selector_paths":true,"instructions":instructions,"native_dispatches":dispatches,"compiled_selector_bytes":compiled[0],"compiled_bank_bytes":compiled[1],"compiled_total_source_bytes":compiled[2],"poisoned_new_ranges":166,"full_states_clocks_equal":true,"all_new_edges_widths_mirrors_real_bios_bus_dma_cpuset":true,"host_decode_publications":host_checks,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
