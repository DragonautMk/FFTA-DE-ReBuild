//! Typed EUR sequences/tables: publication, original guest/native paths and MP2K shadow reads.
use super::*;
use gba_core::{capi::RT_API, mp2k::MemView, rom_data::audio_driver::*, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUDIO_DRIVER_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 39] {
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
        AUDIO_TONES_FILE,
        AUDIO_SEQUENCES_FILE,
        AUDIO_DRIVER_FILE,
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &AudioDriver, rom: Option<&[u8]>) {
    for (at, bytes, _) in data.source_spans().unwrap() {
        let at = at as usize;
        let dst = &mut m.bus.rom[at..at + bytes.len()];
        if let Some(r) = rom {
            dst.copy_from_slice(&r[at..at + bytes.len()]);
        } else {
            for b in dst {
                *b ^= 0x5a;
            }
        }
    }
}
#[test]
#[ignore = "private EUR e44 M4A driver publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_audio_driver_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..38]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e43").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (43, 42));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (44, 43)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 39);
    assert_eq!(source.manifest.resources.len(), 38);
    for i in 1..38 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_audio_driver(&files[38]).unwrap();
    assert_eq!(
        data.source_spans()
            .unwrap()
            .iter()
            .map(|s| s.1.len())
            .sum::<usize>(),
        484
    );
    assert_eq!(
        files[38],
        fs::read(std::env::var_os("FFTA_STATE_TEST_AUDIO_DRIVER_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(source.manifest.resources[37].sources.len(), 7);
    assert_eq!(
        source.manifest.resources[37].dependencies,
        vec![AUDIO_SEQUENCES_ID]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(
        &interrupted,
        &mut source,
        &mut |p, b| if p.ends_with(AUDIO_DRIVER_FILE) {
            Err("injected audio tone publication failure".into())
        } else {
            write_synced(p, b)
        }
    )
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e43-d42/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e44-d43"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(AUDIO_DRIVER_FILE), b"partial").unwrap();
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
    let sequence = parse_audio_sequences(&files[37]).unwrap();
    data.validate_sequence_owner(&sequence).unwrap();
    for case in 0..19 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        fs::write(dataset().join(AUDIO_SEQUENCES_FILE), &files[37]).unwrap();
        match case {
            0 => forged.pcm_keys.entries[0].shift ^= 1,
            1 => forged.pcm_bases.entries[0] ^= 1,
            2 => forged.sample_counts.entries[0] ^= 1,
            3 => forged.cgb_keys.entries[0].shift ^= 1,
            4 => forged.cgb_bases.entries[0] ^= 0x100,
            5 => forged.noise_codes.entries[0] ^= 1,
            6 => forged.wave_volume_codes.entries[0] ^= 1,
            7 => manifest.resources[37].sources[0].length -= 1,
            8 => manifest.resources[37].sources[6].offset += 1,
            9 => manifest.resources[37].sources[0].role = "audio/unknown".into(),
            10 => manifest.resources[37].sources.swap(0, 6),
            11 => {
                manifest.resources[37].sources.pop();
            }
            12 => manifest.resources[37].path = "../audio-driver-tables.json".into(),
            13 => manifest.resources[37].dependencies.clear(),
            14 => manifest.resources[37].dependencies = vec![AUDIO_TONES_ID.into()],
            15 => manifest.resources[37].sources[0].sha256 = "0".repeat(64),
            16 => manifest.resources[37].sha256 = "0".repeat(64),
            17 => manifest.coverage.retain(|id| id != AUDIO_DRIVER_ID),
            _ => {
                let mut owner = sequence.clone();
                owner.driver.clock_source_offset += 1;
                assert!(data.validate_sequence_owner(&owner).is_err());
                let bytes = encode_audio_sequences(&owner).unwrap();
                manifest.resources[36].sha256 = gba_core::ffta_rom::digest(&bytes);
                fs::write(dataset().join(AUDIO_SEQUENCES_FILE), bytes).unwrap();
            }
        }
        forged.validate().unwrap();
        let bytes = encode_audio_driver(&forged).unwrap();
        if case != 16 {
            manifest.resources[37].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(AUDIO_DRIVER_FILE), bytes).unwrap();
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
    fs::write(dataset().join(AUDIO_SEQUENCES_FILE), &files[37]).unwrap();
    fs::write(dataset().join(AUDIO_DRIVER_FILE), &files[38]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(dataset().join(AUDIO_DRIVER_FILE), r.join("missing.json")).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(r.join("missing.json"), dataset().join(AUDIO_DRIVER_FILE)).unwrap();
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
    for (at, bytes, _) in data.source_spans().unwrap() {
        for i in 0..bytes.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    let stats = owner.audio_driver_stats().unwrap();
    assert_eq!(stats.source_bytes, 484);
    assert_eq!(stats.table_bytes, [180, 48, 24, 132, 24, 60, 16]);
    assert_eq!(stats.entries_used, [180, 12, 12, 132, 12, 60, 16]);
    let view = MemView {
        rom: &m.bus.rom,
        ewram: &m.bus.ewram,
        iwram: &m.bus.iwram,
        resources: Some(&owner),
    };
    for (_, at, _, _) in TABLES {
        assert_eq!(
            view.bytes::<4>(0x08000000 + at).unwrap(),
            rom[at as usize..at as usize + 4]
        );
    }
    let after = owner.audio_driver_stats().unwrap();
    assert_eq!(after.host_reads, 7);
    assert_eq!(after.host_source_bytes, 28);
    assert_eq!(after.source_bytes, stats.source_bytes);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,44,43],"files":39,"resources":38,"source_ranges":7,"source_bytes":484,"all37prior_decoded_files_and_e43_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":19,"actual_sequence_owner_rejected":true,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_and424entry_masks_and_generic_host_after_restore":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

const CHANNEL: u32 = 0x02010000;
const RETURN: u32 = 0x08000300;
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
#[derive(Clone, Debug)]
enum Probe {
    Pcm(u32, u32, u32),
    Cgb(u32, u32, u32),
    Rate(u32),
    Volume(u8),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Pcm(..) => (0x08148a5c, RETURN),
            Self::Cgb(..) => (0x08149510, RETURN),
            Self::Rate(_) => (0x08148fcc, 0x08148fec),
            Self::Volume(_) => (0x08149a32, 0x08149a3e),
        }
    }
    fn setup(&self, m: &mut Machine) {
        m.bus.ewram.fill(0);
        m.bus.iwram.fill(0);
        m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
        put(&mut m.bus.iwram, 0x7ff0, 0x020084e0);
        m.cpu.regs = [0; 16];
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match *self {
            Self::Pcm(word, key, frac) => {
                put(&mut m.bus.ewram, 0x10004, word);
                m.cpu.regs[..3].copy_from_slice(&[CHANNEL, key, frac]);
            }
            Self::Cgb(kind, key, frac) => m.cpu.regs[..3].copy_from_slice(&[kind, key, frac]),
            Self::Rate(mode) => m.cpu.regs[0] = mode,
            Self::Volume(level) => {
                m.cpu.regs[4] = CHANNEL;
                m.bus.ewram[0x10009] = level;
                put(&mut m.bus.iwram, 0x7e0c, 0x02020000);
            }
        }
    }
}
fn original(m: &mut Machine, p: &Probe, pcs: &mut BTreeSet<(u32, bool)>) -> usize {
    p.setup(m);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 1500, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert((m.cpu.regs[15], m.cpu.cpsr & 32 != 0));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
thread_local! {static COMPILED:Cell<[u64;7]>=const{Cell::new([0;7])};}
fn usage(m: *mut std::ffi::c_void) -> [u64; 7] {
    unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .audio_driver_stats()
            .unwrap()
            .table_bytes
    }
}
fn count(b: [u64; 7], a: [u64; 7]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..7 {
            x[i] += a[i] - b[i];
        }
        v.set(x);
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read8)(m, a);
    count(b, usage(m));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read16)(m, a);
    count(b, usage(m));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = usage(m);
    let v = (RT_API.read32)(m, a);
    count(b, usage(m));
    v
}
#[test]
#[ignore = "private EUR e44 typed driver mixed Thumb/ARM native and adjacent aliases; Wine only, Linux/USA pending"]
fn european_audio_driver_native_integer_readers_and_adjacent_aliases() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.audio_driver.as_ref().unwrap();
    let mut cases = Vec::new();
    for (word, key, frac) in [
        (14069760, 0, 0),
        (14069760, 60, 127),
        (14069760, 178, 1),
        (14069760, 179, 0),
        (0xffffffff, 0x1234003c, 0xabcdef7f),
    ] {
        cases.push(Probe::Pcm(word, key, frac));
    }
    for (kind, key, frac) in [
        (3, 35, 255),
        (1, 36, 0),
        (2, 80, 127),
        (3, 166, 255),
        (255, 167, 0),
        (0, 0xabcdef24, 0x123456ff),
        (4, 20, 0),
        (4, 21, 255),
        (4, 80, 127),
        (0x104, 255, 0),
    ] {
        cases.push(Probe::Cgb(kind, key, frac));
    }
    for mode in [0, 1, 7, 12, 13, 14, 15] {
        cases.push(Probe::Rate(mode << 16));
    }
    for level in [0, 15, 16, 31, 64, 255] {
        cases.push(Probe::Volume(level));
    }
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, &mut pcs);
    }
    assert!(
        pcs.contains(&(0x08147ec8, false)),
        "original unsigned ARM multiply-high reached"
    );
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = pcs
        .iter()
        .map(|&(addr, thumb)| {
            let mut instrs = if thumb {
                vec![armv4t::decode_thumb(view.read16(addr), addr)]
            } else {
                vec![armv4t::decode_arm(view.read32(addr), addr)]
            };
            if thumb {
                let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
                if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                    instrs.push(next);
                }
            }
            Block {
                addr,
                thumb,
                instrs,
            }
        })
        .collect();
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let object = out.join("audio-driver.o");
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
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|v| v.set([0; 7]));
    let mut dispatches = 0;
    // Volume16/31/64 belong to the existing clock table, not the new driver owner.
    let outside = [0x370d04usize, 0x370d13, 0x370d34];
    let raw = 0x370df3usize;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
            if matches!(p, Probe::Volume(255)) {
                m.bus.rom[raw] ^= 0x5a;
            }
        }
        original(&mut legacy, p, &mut BTreeSet::new());
        let before = owner.audio_driver_stats().unwrap().source_bytes;
        let seqbefore = owner.audio_sequences_stats().unwrap().source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
            for at in outside {
                m.bus.rom[at] ^= 0x5a;
            }
        }
        original(&mut decoded, p, &mut BTreeSet::new());
        p.setup(&mut native);
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 1500, "native {p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.cpsr & 32 != 0))
                .expect("observed mixed instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        let bytes = owner.audio_driver_stats().unwrap().source_bytes - before;
        if matches!(p, Probe::Volume(16 | 31 | 64 | 255)) {
            assert_eq!(bytes, 0);
        } else {
            assert!(bytes > 0, "{p:?}");
        }
        if matches!(p, Probe::Volume(16 | 31 | 64)) {
            assert_eq!(
                owner.audio_sequences_stats().unwrap().source_bytes - seqbefore,
                1
            );
        }
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
            for at in outside {
                m.bus.rom[at] = rom[at];
            }
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.bus.rom[raw] = rom[raw];
        }
        let expected = legacy.save_state().unwrap();
        for m in [&mut decoded, &mut native] {
            assert!(
                m.save_state().unwrap() == expected,
                "complete state/clock {p:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(
        compiled.iter().all(|v| *v > 0),
        "all seven compiled table classes {compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    // Poison neighbouring owned bytes too: merged words must restore both typed owners.
    for at in [0x370b1eusize, 0x370b1f, 0x370d04, 0x370d05] {
        decoded.bus.rom[at] ^= 0x5a;
    }
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in data.source_spans().unwrap() {
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
    decoded.bus.real_bios = false;
    for at in [0x370b1eusize, 0x370d02] {
        let d = owner.audio_driver_stats().unwrap();
        let s = owner.audio_sequences_stats().unwrap();
        assert_eq!(
            decoded.bus.read32(0x08000000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
        assert_eq!(
            owner.audio_driver_stats().unwrap().source_bytes - d.source_bytes,
            2
        );
        assert_eq!(
            owner.audio_sequences_stats().unwrap().source_bytes - s.source_bytes,
            2
        );
        let d = owner.audio_driver_stats().unwrap();
        let s = owner.audio_sequences_stats().unwrap();
        let view = MemView {
            rom: &decoded.bus.rom,
            ewram: &decoded.bus.ewram,
            iwram: &decoded.bus.iwram,
            resources: Some(&owner),
        };
        assert_eq!(
            view.bytes::<4>(0x08000000 + at as u32).unwrap(),
            rom[at..at + 4]
        );
        assert_eq!(
            owner.audio_driver_stats().unwrap().host_source_bytes - d.host_source_bytes,
            2
        );
        assert_eq!(
            owner.audio_sequences_stats().unwrap().host_source_bytes - s.host_source_bytes,
            2
        );
    }
    let before = owner.audio_driver_stats().unwrap();
    decoded.bus.rom[raw] ^= 0x5a;
    assert_eq!(decoded.bus.read8(0x08000000 + raw as u32), rom[raw] ^ 0x5a);
    let view = MemView {
        rom: &decoded.bus.rom,
        ewram: &decoded.bus.ewram,
        iwram: &decoded.bus.iwram,
        resources: Some(&owner),
    };
    assert_eq!(view.u8(0x08000000 + raw as u32), Some(rom[raw] ^ 0x5a));
    assert_eq!(owner.audio_driver_stats().unwrap(), before);
    range(&mut decoded, data, Some(&rom));
    for at in [0x370b1eusize, 0x370b1f, 0x370d04, 0x370d05, raw] {
        decoded.bus.rom[at] = rom[at];
    }
    for (swi, src, count) in [
        (0xdf0b, 0x370b1e, 4),
        (0xdf0c, 0x370bd0, 8),
        (0xdf0b, 0x370d02, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let before = owner.audio_driver_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.audio_driver_stats().unwrap().source_bytes > before);
        range(&mut decoded, data, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08370d02);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let before = owner.audio_driver_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(owner.audio_driver_stats().unwrap().source_bytes - before, 2);
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,44,43],"original_interpreter_native_paths":cases.len(),"compiled_table_bytes":compiled,"poisoned_driver_source_bytes":484,"full_states_clocks_equal_at_declared_checkpoints":true,"mixed_thumb_arm_unsigned_multiply_high":true,"sample_selector0_13_14_15_physical_owner_aliases":true,"volume16_31_64_prior_clock_owner_and255_raw_native_bus_host_fallback":true,"all7spans_edges_widths_mirrors_realbios":true,"cpuset_fastset_dma_two_sequence_two_driver_guest_host_boundaries":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"scope":"pitch full returns; rate/volume internal checkpoints, not whole initialization or mixer ABI","host_reads":"generic typed view only, no host driver arithmetic rewrite","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
