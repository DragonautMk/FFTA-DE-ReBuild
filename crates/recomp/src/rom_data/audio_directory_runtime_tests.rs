//! Managed EUR M4A directory/header consumption; original sequencer and mixer remain guest work.
use super::audio_directory_tests::{run as original, setup, Live, RETURN};
use super::*;
use gba_core::{capi::RT_API, rom_data::audio_directory::*, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUDIO_DIRECTORY_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 36] {
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
        AUDIO_DIRECTORY_FILE,
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, d: &AudioDirectory, rom: Option<&[u8]>) {
    for (at, bytes) in d.source_spans() {
        let at = at as usize;
        if let Some(r) = rom {
            m.bus.rom[at..at + bytes.len()].copy_from_slice(&r[at..at + bytes.len()]);
        } else {
            for v in &mut m.bus.rom[at..at + bytes.len()] {
                *v ^= 0x5a;
            }
        }
    }
}
#[test]
#[ignore = "private EUR e41 M4A publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_audio_directory_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..35]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e40").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (40, 39));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (41, 40)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 36);
    assert_eq!(source.manifest.resources.len(), 35);
    for i in 1..35 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_audio_directory(&files[35]).unwrap();
    assert_eq!(
        (data.players.len(), data.songs.len(), data.headers.len()),
        (3, 640, 455)
    );
    assert_eq!(
        files[35],
        fs::read(std::env::var_os("FFTA_STATE_TEST_AUDIO_DIRECTORY_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(source.manifest.resources[34].sources.len(), 457);
    assert!(source.manifest.resources[34].dependencies.is_empty());
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(AUDIO_DIRECTORY_FILE)
    {
        Err("injected audio directory publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e40-d39/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e41-d40"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(AUDIO_DIRECTORY_FILE), b"partial").unwrap();
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
    for case in 0..25 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.players[0].opaque09 ^= 1,
            1 => forged.players[0].priority_gate ^= 0x8000,
            2 => forged.songs[0].opaque06 ^= 1,
            3 => forged.headers[0].tone_bank_address ^= 4,
            4 => forged.headers[1].priority ^= 1,
            5 => forged.headers[1].reverb ^= 1,
            6 => forged.headers[1].opaque01 ^= 1,
            7 => forged.headers[1].track_addresses[0] ^= 1,
            8 => forged.songs[0].header_id = 1,
            9 => forged.songs.swap(0, 1),
            10 => forged.headers[0].source_offset += 4,
            11 => forged.players[0].tracks_address = 0x03007ffc,
            12 => {
                forged.headers.pop();
            }
            13 => manifest.resources[34].sources[0].length -= 1,
            14 => manifest.resources[34].sources[456].role = "audio/unknown".into(),
            15 => manifest.resources[34].sources[1].offset += 1,
            16 => manifest.resources[34].sources.swap(0, 1),
            17 => {
                manifest.resources[34].sources.pop();
            }
            18 => manifest.resources[34].sources[456].sha256 = "0".repeat(64),
            19 => manifest.resources[34].dependencies = vec![ITEM_RECORDS_ID.into()],
            20 => manifest.resources[34].path = "../audio-directory.json".into(),
            21 => manifest.resources[34].sha256 = "0".repeat(64),
            22 => manifest.coverage.retain(|id| id != AUDIO_DIRECTORY_ID),
            23 => manifest.resources[34].format = "audio-unknown".into(),
            _ => forged.players[0].player_address += 4,
        }
        if (8..=12).contains(&case) {
            assert!(forged.validate().is_err(), "guard {case}");
        } else {
            forged.validate().unwrap();
        }
        let bytes = encode_audio_directory(&forged).unwrap();
        if case != 21 {
            manifest.resources[34].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(AUDIO_DIRECTORY_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "guard {case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(AUDIO_DIRECTORY_FILE), &files[35]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(dataset().join(AUDIO_DIRECTORY_FILE), r.join("missing.json")).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(r.join("missing.json"), dataset().join(AUDIO_DIRECTORY_FILE)).unwrap();
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
    range(&mut m, &data, None);
    for (at, bytes) in data.source_spans() {
        for i in 0..bytes.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    let stats = owner.audio_directory_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.player_bytes,
            stats.directory_bytes,
            stats.header_bytes,
            stats.track_pointer_bytes
        ),
        (15236, 36, 5120, 3640, 6440)
    );
    assert_eq!(stats.player_mask, 7);
    assert_eq!(
        stats.song_masks.iter().map(|v| v.count_ones()).sum::<u32>(),
        640
    );
    assert_eq!(
        stats
            .header_masks
            .iter()
            .map(|v| v.count_ones())
            .sum::<u32>(),
        455
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,41,40],"files":36,"resources":35,"source_ranges":457,"source_bytes":15236,
        "all34prior_decoded_files_and_e40_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":25,
        "missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_masks_3players_640songs_455headers":true,
        "linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

thread_local! {static COMPILED:Cell<[u64;4]>=const{Cell::new([0;4])};}
fn used(m: *mut std::ffi::c_void) -> [u64; 4] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .audio_directory_stats()
            .unwrap()
    };
    [
        s.player_bytes,
        s.directory_bytes,
        s.header_bytes,
        s.track_pointer_bytes,
    ]
}
fn count(b: [u64; 4], a: [u64; 4]) {
    COMPILED.with(|v| {
        let mut n = v.get();
        for i in 0..4 {
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
#[ignore = "private EUR e41 M4A native/full-state/bus; Wine only, Linux/USA pending"]
fn european_audio_directory_native_original_start_and_fragmented_bus() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.audio_directory.as_ref().unwrap();
    let mut representatives = BTreeMap::new();
    for s in &data.songs {
        let h = data.header(s.header_id).unwrap();
        representatives
            .entry((
                s.player_index,
                h.track_count,
                h.tone_bank_address,
                h.reverb & 0x80,
            ))
            .or_insert(u32::from(s.id));
    }
    let mut cases: Vec<_> = representatives
        .values()
        .map(|&id| (id, Live::default()))
        .collect();
    for id in [0, 1, 639, 0xffff0000, 0x1234027f] {
        cases.push((id, Live::default()));
    }
    for capacity in [0, 1, 6, 16] {
        cases.push((
            0,
            Live {
                capacity: Some(capacity),
                ..Live::default()
            },
        ));
    }
    cases.push((
        0,
        Live {
            reject_priority: true,
            ..Live::default()
        },
    ));
    cases.push((
        0,
        Live {
            locked: true,
            ..Live::default()
        },
    ));
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for &(id, live) in &cases {
        instructions += original(&mut tracer, data, id, live, &mut pcs, &mut BTreeSet::new());
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = pcs
        .iter()
        .map(|&addr| {
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
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let object = out.join("audio-directory.o");
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
    for &(id, live) in &cases {
        original(
            &mut legacy,
            data,
            id,
            live,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        );
        let db = decoded
            .bus
            .rom_resources()
            .unwrap()
            .audio_directory_stats()
            .unwrap()
            .source_bytes;
        let nb = native
            .bus
            .rom_resources()
            .unwrap()
            .audio_directory_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
        }
        original(
            &mut decoded,
            data,
            id,
            live,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        );
        setup(&mut native, data, id, live);
        while native.cpu.regs[15] != RETURN {
            assert!(dispatches < 500000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original audio instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let d = decoded
            .bus
            .rom_resources()
            .unwrap()
            .audio_directory_stats()
            .unwrap()
            .source_bytes
            - db;
        let n = native
            .bus
            .rom_resources()
            .unwrap()
            .audio_directory_stats()
            .unwrap()
            .source_bytes
            - nb;
        assert!(d > 0);
        assert_eq!(d, n);
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
            assert!(
                m.save_state().unwrap() == legacy.save_state().unwrap(),
                "full state/clock {id} {live:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|v| *v > 0));
    range(&mut decoded, data, None);
    let owner = decoded.bus.rom_resources().unwrap().clone();
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes) in data.source_spans() {
            for i in 0..bytes.len() {
                assert_eq!(
                    decoded.bus.read8(0x08000000 + at + i as u32),
                    rom[at as usize + i]
                );
            }
            let end = at + bytes.len() as u32;
            for offset in (at - 3..at + 4).chain(end - 3..end + 4) {
                for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                    assert_eq!(decoded.bus.read8(mirror + offset), rom[offset as usize]);
                    assert_eq!(
                        decoded.bus.read16(mirror + offset),
                        u16::from_le_bytes(
                            rom[offset as usize..offset as usize + 2]
                                .try_into()
                                .unwrap()
                        )
                    );
                    assert_eq!(
                        (RT_API.read32)(
                            &mut decoded as *mut Machine as *mut std::ffi::c_void,
                            mirror + offset
                        ),
                        u32::from_le_bytes(
                            rom[offset as usize..offset as usize + 4]
                                .try_into()
                                .unwrap()
                        )
                    );
                }
            }
        }
    }
    let raw = 0x150948usize;
    let before = owner.audio_directory_stats().unwrap();
    for at in raw..raw + 4 {
        decoded.bus.rom[at] ^= 0x5a;
        assert_eq!(decoded.bus.read8(0x08000000 + at as u32), rom[at] ^ 0x5a);
    }
    assert_eq!(owner.audio_directory_stats().unwrap(), before);
    decoded.bus.rom[raw..raw + 4].copy_from_slice(&rom[raw..raw + 4]);
    let before = owner.audio_directory_stats().unwrap();
    assert_eq!(
        decoded.bus.read32(0x08150946),
        u32::from_le_bytes(rom[0x150946..0x15094a].try_into().unwrap())
    );
    let after = owner.audio_directory_stats().unwrap();
    assert_eq!(
        (
            after.source_bytes - before.source_bytes,
            after.header_bytes - before.header_bytes
        ),
        (2, 2)
    );
    range(&mut decoded, data, Some(&rom));
    decoded.bus.real_bios = false;
    for (swi, src, count, typed) in [
        (0xdf0b, PLAYER_OFFSET, 16, 32),
        (0xdf0c, DIRECTORY_OFFSET, 8, 32),
        (0xdf0b, 0x150946, 2, 2),
        (0xdf0b, DIRECTORY_END - 2, 6, 10),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let before = owner.audio_directory_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert_eq!(
            owner.audio_directory_stats().unwrap().source_bytes - before,
            typed
        );
        range(&mut decoded, data, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08150946);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let before = owner.audio_directory_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.audio_directory_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"original_paths":cases.len(),"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches":dispatches,
        "compiled_player_directory_header_track_bytes":compiled,"poisoned_source_bytes":15236,"full_states_clocks_equal":true,"original_start_alias_low16_live_capacity_priority_lock_reverb":true,
        "all457spans_allbytes_edges_widths_mirrors_bios":true,"dummy_wave_tail2typed2raw_word_cpuset_dma":true,"cpuset_directory2_dummyheader8_raw2":true,
        "linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
