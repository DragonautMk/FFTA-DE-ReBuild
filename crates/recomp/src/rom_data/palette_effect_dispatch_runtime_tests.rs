//! Typed EUR palette dispatch: managed lifecycle and original native lookup with live RAM.
use super::*;
use gba_core::{capi::RT_API, rom_data::palette_effect_dispatch::*, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PALETTE_EFFECT_DISPATCH_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 40] {
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
        PALETTE_EFFECT_DISPATCH_FILE,
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &PaletteEffectDispatch, rom: Option<&[u8]>) {
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
#[ignore = "private EUR e45 palette dispatch publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_palette_effect_dispatch_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..39]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e44").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (44, 43));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (45, 44)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 40);
    assert_eq!(source.manifest.resources.len(), 39);
    for i in 1..39 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_palette_effect_dispatch(&files[39]).unwrap();
    assert_eq!(
        data.source_spans()
            .unwrap()
            .iter()
            .map(|s| s.1.len())
            .sum::<usize>(),
        68
    );
    assert_eq!(
        files[39],
        fs::read(std::env::var_os("FFTA_STATE_TEST_PALETTE_EFFECT_DISPATCH_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert_eq!(source.manifest.resources[38].sources.len(), 1);
    assert_eq!(
        source.manifest.resources[38].dependencies,
        Vec::<String>::new()
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(PALETTE_EFFECT_DISPATCH_FILE)
    {
        Err("injected audio tone publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e44-d43/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e45-d44"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(PALETTE_EFFECT_DISPATCH_FILE),
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
    assert_eq!(data.slots.len(), 17);
    for case in 0..12 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.slots[1].target.as_mut().unwrap().rom_offset += 2,
            1 => forged.slots[1].target = None,
            2 => forged.slots[1].target.as_mut().unwrap().rom_offset |= 1,
            3 => manifest.resources[38].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[38].sources[0].length -= 4,
            5 => manifest.resources[38].sources[0].offset += 4,
            6 => manifest.resources[38].sources[0].role = "graphics/unknown".into(),
            7 => manifest.resources[38].path = "../palette-effect-dispatch.json".into(),
            8 => manifest.resources[38].dependencies = vec![AUDIO_DRIVER_ID.into()],
            9 => manifest.resources[38].sha256 = "0".repeat(64),
            10 => manifest
                .coverage
                .retain(|id| id != PALETTE_EFFECT_DISPATCH_ID),
            _ => {
                manifest.resources[38].sources.clear();
            }
        }
        let bytes = encode_palette_effect_dispatch(&forged).unwrap();
        if case != 9 {
            manifest.resources[38].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(PALETTE_EFFECT_DISPATCH_FILE), bytes).unwrap();
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
    fs::write(dataset().join(PALETTE_EFFECT_DISPATCH_FILE), &files[39]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(PALETTE_EFFECT_DISPATCH_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(
        r.join("missing.json"),
        dataset().join(PALETTE_EFFECT_DISPATCH_FILE),
    )
    .unwrap();
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
    let stats = owner.palette_effect_dispatch_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.callback_bytes,
            stats.null_bytes,
            stats.slot_mask
        ),
        (68, 20, 48, (1 << 17) - 1)
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,45,44],"files":40,"resources":39,"source_ranges":1,"source_bytes":68,"all38prior_decoded_files_and_e44_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":12,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_20callback_48null_and17slot_mask":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
const OBJECT: u32 = 0x02010000;
#[derive(Debug)]
enum Probe {
    Lookup(u32),
    Inactive(u16),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Lookup(_) => (0x0814ac68, 0x0814ac72),
            Self::Inactive(_) => (0x0814ac62, 0x0814ac88),
        }
    }
    fn raw(&self) -> Option<usize> {
        match self {
            Self::Lookup(k) if (*k as u16) >= 17 => {
                Some(SOURCE_OFFSET as usize + usize::from(*k as u16) * 4)
            }
            _ => None,
        }
    }
    fn setup(&self, m: &mut Machine) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        m.cpu.regs = [0; 16];
        m.cpu.regs[4] = OBJECT;
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match *self {
            Self::Lookup(k) => m.bus.ewram[0x10004..0x10008].copy_from_slice(&k.to_le_bytes()),
            Self::Inactive(s) => m.bus.ewram[0x10000..0x10002].copy_from_slice(&s.to_le_bytes()),
        }
    }
}
fn original(m: &mut Machine, p: &Probe, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 10, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
thread_local! {static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn usage(m: *mut std::ffi::c_void) -> [u64; 2] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .palette_effect_dispatch_stats()
            .unwrap()
    };
    [s.callback_bytes, s.null_bytes]
}
fn count(b: [u64; 2], a: [u64; 2]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..2 {
            x[i] += a[i] - b[i];
        }
        v.set(x);
    });
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
#[ignore = "private EUR e45 palette dispatch original/native live-kind and raw boundary checks; Wine only, Linux/USA pending"]
fn european_palette_effect_dispatch_native_live_kind_and_raw_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.palette_effect_dispatch.as_ref().unwrap();
    let mut cases: Vec<_> = [0, 1, 2, 4, 8, 16, 17, 65535, 0x12340001]
        .into_iter()
        .map(Probe::Lookup)
        .collect();
    cases.push(Probe::Inactive(2));
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, &mut pcs);
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks: Vec<Block> = pcs
        .iter()
        .map(|&addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let out = root().join("native");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let object = out.join("palette-effect-dispatch.o");
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
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|v| v.set([0; 2]));
    let mut dispatches = 0;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
            if let Some(at) = p.raw() {
                for v in &mut m.bus.rom[at..at + 4] {
                    *v ^= 0x5a;
                }
            }
        }
        original(&mut legacy, p, &mut BTreeSet::new());
        let before = owner.palette_effect_dispatch_stats().unwrap().source_bytes;
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
        }
        original(&mut decoded, p, &mut BTreeSet::new());
        p.setup(&mut native);
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 10, "{p:?} native pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("observed instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        let bytes = owner.palette_effect_dispatch_stats().unwrap().source_bytes - before;
        assert_eq!(
            bytes,
            if matches!(p,Probe::Lookup(k)if (*k as u16)<17) {
                4
            } else {
                0
            },
            "{p:?}"
        );
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            if let Some(at) = p.raw() {
                m.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
            }
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
    assert_eq!(compiled, [24, 4]);
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    let first = SOURCE_OFFSET;
    let end = SOURCE_OFFSET + SOURCE_BYTES as u32;
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for offset in (first - 3..first + 4).chain(end - 3..end + 4) {
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
    decoded.bus.real_bios = false;
    for at in [first - 2, end - 2] {
        let before = owner.palette_effect_dispatch_stats().unwrap().source_bytes;
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
        assert_eq!(
            owner.palette_effect_dispatch_stats().unwrap().source_bytes - before,
            2
        );
    }
    let before = owner.palette_effect_dispatch_stats().unwrap();
    decoded.bus.rom[end as usize] ^= 0x5a;
    assert_eq!(
        decoded.bus.read8(0x08000000 + end),
        rom[end as usize] ^ 0x5a
    );
    assert_eq!(owner.palette_effect_dispatch_stats().unwrap(), before);
    decoded.bus.rom[end as usize] = rom[end as usize];
    range(&mut decoded, data, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, first - 2, 4),
        (0xdf0c, first, 8),
        (0xdf0b, end - 2, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let before = owner.palette_effect_dispatch_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.palette_effect_dispatch_stats().unwrap().source_bytes > before);
        range(&mut decoded, data, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + end - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let before = owner.palette_effect_dispatch_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.palette_effect_dispatch_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,45,44],"original_interpreter_native_paths":cases.len(),"compiled_callback_null_bytes":compiled,"poisoned_source_bytes":68,"full_states_clocks_equal_at_declared_checkpoints":true,"live_u16_kind_highword_truncation_null_and_inactive_state_preserved":true,"raw17_65535_native_and_bus_fallback":true,"edges_widths_mirrors_realbios_cpuset_fastset_dma_two_typed_two_raw_crossing":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"scope":"lookup and inactive-state prefixes, not complete palette callbacks or allocator ABI","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
