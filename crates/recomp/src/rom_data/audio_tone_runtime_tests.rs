//! Typed EUR tones/samples: publication, original guest/native paths and MP2K shadow reads.
use super::*;
use gba_core::{
    capi::RT_API,
    mp2k::{MemView, Mp2kHle},
    rom_data::audio_tones::*,
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUDIO_TONES_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 37] {
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
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &AudioTones, rom: Option<&[u8]>) {
    for (at, bytes, _) in data.source_spans() {
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
    let at = FIRST_WAVE as usize;
    let dst = &mut m.bus.rom[at..at + 4];
    if let Some(r) = rom {
        dst.copy_from_slice(&r[at..at + 4]);
    } else {
        for b in dst {
            *b ^= 0x5a;
        }
    }
}
#[test]
#[ignore = "private EUR e42 M4A publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_audio_tones_managed_authentication_alias_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..36]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e41").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (41, 40));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (42, 41)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 37);
    assert_eq!(source.manifest.resources.len(), 36);
    for i in 1..36 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_audio_tones(&files[36]).unwrap();
    assert_eq!(
        (data.tones.len(), data.map_links.len(), data.pcm_waves.len()),
        (429, 348, 277)
    );
    assert_eq!(
        files[36],
        fs::read(std::env::var_os("FFTA_STATE_TEST_AUDIO_TONES_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(source.manifest.resources[35].sources.len(), 736);
    assert_eq!(
        source.manifest.resources[35].dependencies,
        vec![AUDIO_DIRECTORY_ID]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(
        &interrupted,
        &mut source,
        &mut |p, b| if p.ends_with(AUDIO_TONES_FILE) {
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
        .join("v1/e41-d40/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e42-d41"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(AUDIO_TONES_FILE), b"partial").unwrap();
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
    for case in 0..26 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        fs::write(dataset().join(AUDIO_DIRECTORY_FILE), &files[35]).unwrap();
        match case {
            0 => forged.tones[0].key ^= 1,
            1 => forged.tones[0].word08 ^= 1,
            2 => forged.map_bytes[0].selector ^= 1,
            3 => forged.pcm_waves[0].samples[0] ^= 1,
            4 => {
                let e = forged.pcm_waves[0].interpolation_endpoint.as_mut().unwrap();
                *e ^= 1;
            }
            5 => forged.pcm_waves[0].interpolation_endpoint = None,
            6 => forged.cgb_waves[0].amplitudes[0] ^= 1,
            7 => forged.pcm_waves[0].frequency_word ^= 1,
            8 => {
                let alias = forged.directory_aliases[0].wave as usize;
                forged.pcm_waves[alias].flags ^= 0x4000;
                forged.validate().unwrap();
                assert_eq!(forged.source_spans(), data.source_spans());
            }
            9 => forged.banks[0].programs[0].selector ^= 1,
            10 => {
                let mut owner = parse_audio_directory(&files[35]).unwrap();
                let id = data.directory_aliases[0].directory_header;
                owner
                    .headers
                    .iter_mut()
                    .find(|h| h.id == id)
                    .unwrap()
                    .tone_bank_address ^= 1;
                let bytes = encode_audio_directory(&owner).unwrap();
                manifest.resources[34].sha256 = gba_core::ffta_rom::digest(&bytes);
                fs::write(dataset().join(AUDIO_DIRECTORY_FILE), bytes).unwrap();
                assert!(data.validate_directory(&owner).is_err());
            }
            11 => manifest.resources[35].sources[0].length -= 1,
            12 => manifest.resources[35].sources[735].offset += 1,
            13 => manifest.resources[35].sources[0].role = "audio/unknown".into(),
            14 => manifest.resources[35].sources.swap(0, 735),
            15 => {
                manifest.resources[35].sources.pop();
            }
            16 => manifest.resources[35].path = "../audio-tones.json".into(),
            17 => manifest.resources[35].dependencies.clear(),
            18 => manifest.resources[35].dependencies = vec![ITEM_RECORDS_ID.into()],
            19 => manifest.resources[35].sources[0].sha256 = "0".repeat(64),
            20 => manifest.resources[35].sha256 = "0".repeat(64),
            21 => manifest.coverage.retain(|id| id != AUDIO_TONES_ID),
            22 => forged.pcm_waves[0].declared_length += 1,
            23 => forged.cgb_waves[0].amplitudes[0] = 16,
            24 => forged.tones[0].ctype = 0x20,
            _ => forged.banks[0].source_address += 4,
        }
        let bytes = encode_audio_tones(&forged).unwrap();
        if case != 20 {
            manifest.resources[35].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(AUDIO_TONES_FILE), bytes).unwrap();
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
    fs::write(dataset().join(AUDIO_TONES_FILE), &files[36]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(dataset().join(AUDIO_TONES_FILE), r.join("missing.json")).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(r.join("missing.json"), dataset().join(AUDIO_TONES_FILE)).unwrap();
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
    for (at, bytes, _) in data.source_spans() {
        for i in 0..bytes.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    let stats = owner.audio_tones_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.tone_bytes,
            stats.keymap_bytes,
            stats.wave_header_bytes,
            stats.pcm_bytes,
            stats.endpoint_bytes,
            stats.cgb_bytes
        ),
        (1941759, 5148, 347, 4428, 1931528, 276, 32)
    );
    assert_eq!(
        stats.tone_masks.iter().map(|v| v.count_ones()).sum::<u32>(),
        429
    );
    assert_eq!(
        stats.map_masks.iter().map(|v| v.count_ones()).sum::<u32>(),
        347
    );
    assert_eq!(
        stats.wave_masks.iter().map(|v| v.count_ones()).sum::<u32>(),
        277
    );
    assert_eq!(stats.cgb_mask, 3);
    let view = MemView {
        rom: &m.bus.rom,
        ewram: &m.bus.ewram,
        iwram: &m.bus.iwram,
        resources: Some(&owner),
    };
    assert_eq!(
        view.bytes::<16>(0x08000000 + FIRST_WAVE).unwrap(),
        rom[FIRST_WAVE as usize..FIRST_WAVE as usize + 16]
    );
    assert_eq!(
        view.u8(0x08000000 + FIRST_WAVE + 16),
        Some(rom[FIRST_WAVE as usize + 16])
    );
    assert!(owner.audio_tones_stats().unwrap().host_pcm_bytes > 0);
    assert!(owner.audio_directory_stats().unwrap().host_source_bytes > 0);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,42,41],"files":37,"resources":36,"source_ranges":736,"source_bytes":1941759,
        "all35prior_decoded_files_and_e41_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":26,
        "real_directory_alias_edited_without_new_span_change_rejected":true,"actual_forged_directory_owner_rejected":true,
        "missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_masks_and_host_after_restore":true,
        "linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

const PLAYER: u32 = 0x02010000;
const TRACK: u32 = 0x02010100;
const RETURN: u32 = 0x08000300;
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn num(v: &serde_json::Value, k: &str) -> u32 {
    v[k].as_u64().unwrap().try_into().unwrap()
}
#[derive(Clone, Debug)]
enum Probe {
    Program { bank: u32, operand: u32 },
    Map { tone: u32, key: u8, operand: u32 },
    Header(u32),
    Pcm { wave: u32, size: u32, fixed: bool },
    Cgb(u32),
}
impl Probe {
    fn bounds(&self) -> (u32, u32, bool) {
        match self {
            Self::Program { .. } => (0x081483e4, RETURN, true),
            Self::Map { .. } => (0x081487cc, 0x08148868, true),
            Self::Header(_) => (0x08148006, 0x0814807e, true),
            Self::Pcm { fixed, .. } => (
                0x081480c8,
                if *fixed { 0x08148144 } else { 0x081481e8 },
                false,
            ),
            Self::Cgb(_) => (0x081497b0, 0x081497cc, true),
        }
    }
    fn setup(&self, m: &mut Machine, rom: &[u8]) {
        m.bus.ewram.fill(0);
        m.bus.iwram.fill(0);
        put(&mut m.bus.iwram, 0x7ff0, 0x020084e0);
        m.cpu.regs = [0; 16];
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = if self.bounds().2 { 0xbf } else { 0x9f };
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match self {
            Self::Program { bank, operand } => {
                m.cpu.regs[0] = PLAYER;
                m.cpu.regs[1] = TRACK;
                put(&mut m.bus.ewram, 0x10030, *bank);
                put(&mut m.bus.ewram, 0x10140, 0x08000000 + operand);
            }
            Self::Map { tone, key, operand } => {
                m.cpu.regs[1] = PLAYER;
                m.cpu.regs[2] = TRACK;
                m.bus.ewram[0x10124..0x10130]
                    .copy_from_slice(&rom[*tone as usize..*tone as usize + 12]);
                m.bus.ewram[0x10105] = *key;
                put(&mut m.bus.ewram, 0x10140, 0x08000000 + operand);
            }
            Self::Header(wave) => {
                m.cpu.regs[3] = 0x08000000 + wave;
                m.cpu.regs[4] = PLAYER;
                m.cpu.regs[6] = 3;
            }
            Self::Pcm { wave, size, fixed } => {
                m.cpu.regs[2] = 1;
                m.cpu.regs[3] = 0x08000000 + wave + 16 + size - 1;
                m.cpu.regs[4] = PLAYER;
                m.cpu.regs[5] = 0x02020000;
                m.cpu.regs[8] = 1;
                m.bus.ewram[0x10001] = if *fixed { 8 } else { 0 };
            }
            Self::Cgb(wave) => {
                m.cpu.regs[4] = PLAYER;
                put(&mut m.bus.ewram, 0x10024, 0x08000000 + wave);
            }
        }
    }
}
fn original(m: &mut Machine, p: &Probe, rom: &[u8], pcs: &mut BTreeSet<(u32, bool)>) -> usize {
    p.setup(m, rom);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 1000, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert((m.cpu.regs[15], m.cpu.cpsr & 32 != 0));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
thread_local! {static COMPILED:Cell<[u64;6]>=const{Cell::new([0;6])};}
fn usage(m: *mut std::ffi::c_void) -> [u64; 6] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .audio_tones_stats()
            .unwrap()
    };
    [
        s.tone_bytes,
        s.keymap_bytes,
        s.wave_header_bytes,
        s.pcm_bytes,
        s.endpoint_bytes,
        s.cgb_bytes,
    ]
}
fn count(b: [u64; 6], a: [u64; 6]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..6 {
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
#[ignore = "private EUR e42 typed native/bus/MP2K shadow; Wine only, Linux/USA pending"]
fn european_audio_tones_guest_native_shadow_and_source_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.audio_tones.as_ref().unwrap();
    let source: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_TONE_SOURCE_CASES").unwrap()).unwrap(),
    )
    .unwrap();
    let mut cases = Vec::new();
    let mut bank_types = BTreeSet::new();
    for p in source["programs"].as_array().unwrap() {
        if bank_types.insert((num(p, "bank"), num(&p["tone"], "type"))) {
            cases.push(Probe::Program {
                bank: num(p, "bank"),
                operand: num(p, "operand_offset"),
            });
        }
    }
    let mut parents = BTreeSet::new();
    for p in source["mapped"].as_array().unwrap() {
        if parents.insert(num(p, "parent_tone_offset")) {
            cases.push(Probe::Map {
                tone: num(p, "parent_tone_offset"),
                key: num(p, "key") as u8,
                operand: num(p, "note_operand_offset"),
            });
        }
    }
    let mut wave_types = BTreeSet::new();
    for w in &data.pcm_waves {
        let fixed = w.interpolation_endpoint.is_none();
        if wave_types.insert((w.flags, fixed)) || w.source_offset == FIRST_WAVE {
            cases.push(Probe::Header(w.source_offset));
            cases.push(Probe::Pcm {
                wave: w.source_offset,
                size: w.declared_length,
                fixed,
            });
        }
    }
    for w in &data.cgb_waves {
        cases.push(Probe::Cgb(w.source_offset));
    }
    assert!(!data.tones.iter().any(|t| t.source_offset == 0x14d9fc));
    cases.push(Probe::Program {
        bank: 0x0814d9fc,
        operand: 0x150940,
    }); // Original zero selector outside the produced tone domain.
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, &rom, &mut pcs);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = pcs
        .iter()
        .map(|&(addr, thumb)| {
            let mut instrs = vec![if thumb {
                armv4t::decode_thumb(view.read16(addr), addr)
            } else {
                armv4t::decode_arm(view.read32(addr), addr)
            }];
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
    let object = out.join("audio-tones.o");
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
    COMPILED.with(|v| v.set([0; 6]));
    let mut dispatches = 0;
    for (i, p) in cases.iter().enumerate() {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
        }
        original(&mut legacy, p, &rom, &mut BTreeSet::new());
        let db = owner.audio_tones_stats().unwrap().source_bytes;
        let nb = native
            .bus
            .rom_resources()
            .unwrap()
            .audio_tones_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
        }
        original(&mut decoded, p, &rom, &mut BTreeSet::new());
        p.setup(&mut native, &rom);
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 1000, "native {p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            let key = native.cpu.regs[15] | u32::from(native.cpu.cpsr & 32 != 0);
            table.get(key).expect("observed instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        let d = owner.audio_tones_stats().unwrap().source_bytes - db;
        let n = native
            .bus
            .rom_resources()
            .unwrap()
            .audio_tones_stats()
            .unwrap()
            .source_bytes
            - nb;
        assert_eq!(d, n, "{p:?}");
        if i + 1 == cases.len() {
            assert_eq!(d, 0);
        } else {
            assert!(d > 0, "{p:?}");
        }
        let expected = legacy.save_state().unwrap();
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
            assert!(
                m.save_state().unwrap() == expected,
                "full state/clock {p:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled.iter().all(|v| *v > 0), "{compiled:?}");
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in data.source_spans() {
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
    let b = owner.audio_tones_stats().unwrap();
    let bd = owner.audio_directory_stats().unwrap();
    assert_eq!(
        decoded.bus.read32(0x08000000 + FIRST_WAVE + 2),
        u32::from_le_bytes(
            rom[FIRST_WAVE as usize + 2..FIRST_WAVE as usize + 6]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        owner.audio_tones_stats().unwrap().source_bytes - b.source_bytes,
        2
    );
    assert_eq!(
        owner.audio_directory_stats().unwrap().source_bytes - bd.source_bytes,
        2
    );
    let fixed = data
        .pcm_waves
        .iter()
        .find(|w| w.interpolation_endpoint.is_none())
        .unwrap();
    let raw = (fixed.source_offset + 16 + fixed.declared_length) as usize;
    let before = owner.audio_tones_stats().unwrap();
    decoded.bus.rom[raw] ^= 0x5a;
    assert_eq!(decoded.bus.read8(0x08000000 + raw as u32), rom[raw] ^ 0x5a);
    assert_eq!(owner.audio_tones_stats().unwrap(), before);
    decoded.bus.rom[raw] = rom[raw];
    range(&mut decoded, data, Some(&rom));
    decoded.bus.real_bios = false;
    for (swi, src, count) in [
        (0xdf0b, FIRST_WAVE + 2, 8),
        (0xdf0c, FIRST_WAVE + 16, 8),
        (
            0xdf0b,
            fixed.source_offset + 16 + fixed.declared_length - 2,
            4,
        ),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let b = owner.audio_tones_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.audio_tones_stats().unwrap().source_bytes > b);
        range(&mut decoded, data, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + FIRST_WAVE + 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let b = owner.audio_tones_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(owner.audio_tones_stats().unwrap().source_bytes - b, 2);
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    // Exercise both actual MemMap host callsites: frame_hook headers and tick sample rendering.
    let sigs = gba_core::mp2k::detect(&rom);
    let key = 0x030034dd;
    assert!(sigs.iter().any(|s| s.sound_main_ram == key));
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.mp2k = Some(Box::new(Mp2kHle::new(&sigs)));
        m.bus.mp2k.as_mut().unwrap().active = true;
        m.bus.tap_channels = true;
        m.bus.hle_tap.clear();
    }
    range(&mut decoded, data, None);
    let mut nonzero = false;
    for wave in &data.pcm_waves {
        let before = owner.audio_tones_stats().unwrap();
        for m in [&mut legacy, &mut decoded] {
            m.bus.ewram.fill(0);
            m.bus.iwram.fill(0);
            put(&mut m.bus.iwram, 0x7ff0, 0x020084e0);
            let s = 0x84e0;
            put(&mut m.bus.ewram, s, 0x68736d54);
            m.bus.ewram[s + 6] = 1;
            m.bus.ewram[s + 7] = 15;
            m.bus.ewram[s + 0xb] = 7;
            put(&mut m.bus.ewram, s + 0x10, 224);
            put(&mut m.bus.ewram, s + 0x14, 13379);
            let c = s + 0x50;
            m.bus.ewram[c] = 0x80;
            m.bus.ewram[c + 1] = data.tones[wave.tone_ids[0] as usize].ctype;
            m.bus.ewram[c + 2] = 255;
            m.bus.ewram[c + 3] = 128;
            m.bus.ewram[c + 4] = 255;
            m.bus.ewram[c + 5] = 200;
            m.bus.ewram[c + 6] = 180;
            m.bus.ewram[c + 7] = 150;
            put(&mut m.bus.ewram, c + 0x20, 8000);
            put(&mut m.bus.ewram, c + 0x24, 0x08000000 + wave.source_offset);
            m.bus.mp2k_frame_hook(key);
            m.bus.tick(256 * 64);
        }
        let after = owner.audio_tones_stats().unwrap();
        assert!(
            after.host_wave_header_bytes > before.host_wave_header_bytes,
            "header {}",
            wave.id
        );
        assert!(
            after.host_pcm_bytes > before.host_pcm_bytes,
            "samples {}",
            wave.id
        );
        assert_eq!(legacy.bus.hle_tap.len(), decoded.bus.hle_tap.len());
        assert!(
            legacy
                .bus
                .hle_tap
                .iter()
                .zip(&decoded.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "shadow {}",
            wave.id
        );
        nonzero |= legacy.bus.hle_tap.iter().any(|v| *v != 0.0);
        legacy.bus.hle_tap.clear();
        decoded.bus.hle_tap.clear();
    }
    assert!(nonzero);
    let host = owner.audio_tones_stats().unwrap();
    assert_eq!(
        host.host_wave_masks
            .iter()
            .map(|v| v.count_ones())
            .sum::<u32>(),
        277
    );
    assert!(owner.audio_directory_stats().unwrap().host_source_bytes >= 4);
    // Endpoint provider is a separate direct-host check: current HLE interpolation clamps at its existing boundary.
    for wave in &data.pcm_waves {
        if let Some(v) = wave.interpolation_endpoint {
            let view = MemView {
                rom: &decoded.bus.rom,
                ewram: &decoded.bus.ewram,
                iwram: &decoded.bus.iwram,
                resources: Some(&owner),
            };
            assert_eq!(
                view.u8(0x08000000 + wave.source_offset + 16 + wave.declared_length),
                Some(v as u8)
            );
        }
    }
    let before = owner.audio_tones_stats().unwrap();
    let mut ram = vec![0u8; 0x40000];
    for value in [7, 99] {
        ram[20] = value;
        let view = MemView {
            rom: &decoded.bus.rom,
            ewram: &ram,
            iwram: &decoded.bus.iwram,
            resources: Some(&owner),
        };
        assert_eq!(view.u8(0x02000014), Some(value));
    }
    decoded.bus.rom[raw] ^= 0x5a;
    let view = MemView {
        rom: &decoded.bus.rom,
        ewram: &ram,
        iwram: &decoded.bus.iwram,
        resources: Some(&owner),
    };
    assert_eq!(view.u8(0x08000000 + raw as u32), Some(rom[raw] ^ 0x5a));
    decoded.bus.rom[raw] = rom[raw];
    assert_eq!(owner.audio_tones_stats().unwrap(), before);
    range(&mut decoded, data, Some(&rom));
    assert!(
        legacy.save_state().unwrap() == decoded.save_state().unwrap(),
        "complete host/clock state"
    );
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,42,41],"representative_original_interpreter_native_paths":cases.len(),"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"compiled_tone_map_header_pcm_endpoint_cgb_bytes":compiled,"full_state_clocks_equal_at_declared_checkpoints":true,"checkpoints":"BD return; note selection/header setup/ARM sample/CGB copy partial, not full original mixer ABI","unproduced_program_raw_fallback":true,"all736spans_edges_widths_mirrors_bios":true,"directory2_tone2_alias_cpuset_fastset_dma":true,"fixed8_unowned_tail_guest_and_host_raw":true,"actual_mp2k_hook_tick_waves":277,"shadow_sample_bits_and_complete_host_state_equal":true,"shadow_nonzero":nonzero,"host_header_bytes":host.host_wave_header_bytes,"host_pcm_bytes":host.host_pcm_bytes,"direct_host_endpoint_probes":276,"all_pcm_payload_values_read_by_guest_managed_test":true,"all_pcm_payload_bytes_shadow_read":false,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
