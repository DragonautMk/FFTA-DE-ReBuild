//! EUR custom graphics publication, ownership and unchanged original execution.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const SCRATCH: u32 = 0x02020010;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_GRAPHICS_MANAGED_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 15] {
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
    ]
}
fn report(name: &str, v: serde_json::Value) {
    fs::write(root().join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn spans(graphics: &ObjGraphics) -> Vec<(u32, usize)> {
    graphics
        .banks
        .iter()
        .map(|b| (b.source_offset, b.encoded_source.len()))
        .chain(
            graphics
                .pointer_tables
                .iter()
                .map(|t| (t.source_offset, t.bank_offsets.len() * 4)),
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
#[ignore = "exact EUR/e18 dataset; custom graphics authentication/retention/lifecycle; Linux/USA pending"]
fn european_obj_graphics_managed_guards_retention_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..14]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e18").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (18, 17));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (19, 18)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 15);
    assert_eq!(source.manifest.resources.len(), 14);
    for i in 1..14 {
        assert_eq!(files[i], old[i]);
    }
    let graphics = parse_obj_graphics(&files[14]).unwrap();
    assert_eq!(
        (graphics.banks.len(), graphics.pointer_tables.len()),
        (39, 6)
    );
    assert_eq!(source.manifest.resources[13].sources.len(), 45);
    assert_eq!(
        source.manifest.resources[13]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        62674
    );
    let oracle =
        fs::read(std::env::var_os("FFTA_STATE_TEST_OBJ_GRAPHICS_SOURCE_FILE").unwrap()).unwrap();
    assert_eq!(files[14], oracle);
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(OBJ_CUSTOM_GRAPHICS_FILE)
    {
        Err("injected custom OBJ publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e18-d17/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e19-d18"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(OBJ_CUSTOM_GRAPHICS_FILE),
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
                        gba_core::rom_data::obj_graphics::Graphic::decode(&bytes, b.source_offset)
                            .ok()
                            .filter(|v| v != b)
                    })
                    .unwrap();
                forged.banks[0] = changed;
            }
            1 => forged.pointer_tables[0].bank_offsets.swap(0, 1),
            2 => forged.pointer_tables.clear(),
            3 => manifest.resources[13].sources[0].offset += 1,
            4 => manifest.resources[13].path = "../obj-graphics.json".into(),
            _ => manifest.resources[13].sha256 = "0".repeat(64),
        }
        forged.validate().unwrap();
        let bytes = encode_obj_graphics(&forged).unwrap();
        if case != 5 {
            manifest.resources[13].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(OBJ_CUSTOM_GRAPHICS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(OBJ_CUSTOM_GRAPHICS_FILE), &files[14]).unwrap();
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
    for t in &graphics.pointer_tables {
        for language in 0..5 {
            assert_eq!(
                m.bus
                    .read32(0x08000000 + t.source_offset + u32::from(language) * 4),
                0x08000000 + t.select(language).unwrap()
            );
        }
    }
    for b in &graphics.banks {
        for at in 0..b.encoded_source.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + b.source_offset + at as u32),
                b.encoded_source[at]
            );
        }
    }
    let stats = owner.obj_graphics_stats().unwrap();
    assert!(stats.source_bytes > 0 && stats.decoded_literal_bytes > 0 && stats.selector_bytes > 0);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,19,18],"files":15,"resources":14,"banks":39,"tables":6,"ranges":45,"source_bytes":62674,"decoded_bytes":131872,"all_fourteen_prior_files_unchanged":true,"coherent_edits_source_hash_path_guards_transactional":true,"publication_failure_create_reuse_repair_move_e18_retention":true,"unavailable_copy_load_undo_backup_restart":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
#[derive(Clone, Copy, Debug)]
enum Call {
    Decode(usize),
    Select(usize, u8),
    Copy(u32, u32, u32),
}
fn setup(m: &mut Machine, call: Call, graphics: &ObjGraphics) -> u32 {
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    match call {
        Call::Decode(row) => {
            for i in 0..32800 {
                m.bus.write8(SCRATCH - 16 + i, 0x5a);
            }
            m.cpu.regs[..4].copy_from_slice(&[
                SCRATCH,
                0x08000000 + graphics.banks[row].source_offset,
                0,
                0,
            ]);
            m.cpu.regs[15] = 0x080054a0;
            RETURN
        }
        Call::Select(row, language) => {
            let r = &gba_core::rom_data::obj_graphics::TABLE_REFERENCES[row];
            m.bus.write8(0x02001f6e, language);
            m.cpu.regs[4] = SCRATCH;
            m.cpu.regs[5] = SCRATCH;
            m.cpu.regs[7] = SCRATCH;
            m.cpu.regs[8] = 0x02001f6e;
            match r.table_offset {
                0x395588 => m.cpu.regs[4] = 0x02000000,
                0x39559c => m.cpu.regs[4] = 0x02001f6e,
                0x397d30 | 0x397d80 => m.cpu.regs[5] = 0x02001f6e,
                _ => {}
            }
            m.cpu.regs[15] = 0x08000000 + r.reader;
            0x08000000 + r.call
        }
        Call::Copy(offset, control, entry) => {
            m.cpu.regs[..4].copy_from_slice(&[SCRATCH + offset, 0x06012480, control, 0]);
            m.cpu.regs[15] = 0x08000000 + entry;
            RETURN
        }
    }
}
fn interpreted(
    m: &mut Machine,
    call: Call,
    graphics: &ObjGraphics,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    let stop = setup(m, call, graphics);
    for n in 1..=2_000_000 {
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == stop {
            return n;
        }
    }
    panic!("custom OBJ original call did not terminate: {call:?}");
}
thread_local! { static NATIVE_BYTES:Cell<[u64;3]>=const{Cell::new([0;3])}; }
fn used(m: *mut std::ffi::c_void) -> [u64; 3] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .obj_graphics_stats()
            .unwrap()
    };
    [s.source_bytes, s.decoded_literal_bytes, s.selector_bytes]
}
fn count(b: [u64; 3], a: [u64; 3]) {
    NATIVE_BYTES.with(|c| {
        let mut v = c.get();
        for i in 0..3 {
            v[i] += a[i] - b[i];
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
#[ignore = "new EUR custom graphics/TinyCC/poisoned source/clock/bus; Linux/USA pending"]
fn european_obj_graphics_native_and_bus_consumption() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let graphics = files.obj_graphics.as_ref().unwrap();
    let spans = spans(graphics);
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1).sum::<usize>()),
        (45, 62674)
    );
    let mut calls = Vec::new();
    for row in 0..graphics.banks.len() {
        calls.push(Call::Decode(row));
        // Keep all original work, including an offset, a partial prefix and the
        // 1792-byte live tail beyond source3b0ab8's decoded length.
        match graphics.banks[row].source_offset {
            0x3b0ab8 | 0x431e18 => calls.push(Call::Copy(0, 0x600, 0x145230)),
            0xb7f874 => calls.push(Call::Copy(0, 0x4f0, 0x14522c)),
            0x43c7bc => calls.push(Call::Copy(0x300, 0xa0, 0x145230)),
            _ => {}
        }
    }
    for row in 0..6 {
        for language in 0..5 {
            calls.push(Call::Select(row, language));
        }
    }
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &call in &calls {
        instructions += interpreted(&mut tracer, call, graphics, &mut visited);
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
    let object = out.join("obj-graphics.o");
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
        interpreted(&mut legacy, call, graphics, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            ranges(m, &spans, None);
        }
        interpreted(&mut decoded, call, graphics, &mut BTreeSet::new());
        let stop = setup(&mut native, call, graphics);
        while native.cpu.regs[15] != stop {
            assert!(dispatches < 3_000_000);
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
                "custom OBJ complete state/clock {call:?}"
            );
        }
    }
    let compiled = NATIVE_BYTES.with(Cell::get);
    assert!(compiled.iter().all(|&n| n > 0));
    ranges(&mut decoded, &spans, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for &(a, l) in &spans {
            // All bytes once; wider bus edges plus source middles on each mirror.
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
    // New typed table ends immediately before an old animation selector table.
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
    }
    ranges(&mut decoded, &spans, None);
    for at in 0x3955aa..0x3955b3 {
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            legacy.bus.read32(0x08000000 + at)
        );
    }
    ranges(&mut decoded, &spans, Some(&rom));
    for control in [0xdf0b, 0xdf0c] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a397d1c, 0x02032400, 8]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, control);
        }
        ranges(&mut decoded, &spans, None);
        legacy.step();
        decoded.step();
        ranges(&mut decoded, &spans, Some(&rom));
        assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    }
    dma_table(&mut legacy, &mut decoded, &spans, &rom);
    report(
        "native-result.json",
        serde_json::json!({"banks":39,"original_selector_chains":30,"decode_then_live_ram_uploads":4,"calls":calls.len(),"instructions":instructions,"native_dispatches":dispatches,"compiled_source_bytes":compiled[0],"compiled_decoded_literal_bytes":compiled[1],"compiled_selector_bytes":compiled[2],"poisoned_ranges":45,"full_states_clocks_equal":true,"all_bytes_edges_widths_mirrors_real_bios_bus_dma_cpuset_cpufastset":true,"ppu_uses_live_vram_after_original_upload":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

// Kept separate so a fixture-address correction can be checked without
// rerunning the closed 39-bank native/source campaign.
fn dma_table(legacy: &mut Machine, decoded: &mut Machine, spans: &[(u32, usize)], rom: &[u8]) {
    let before = decoded
        .bus
        .rom_resources()
        .unwrap()
        .obj_graphics_stats()
        .unwrap()
        .selector_bytes;
    for m in [&mut *legacy, &mut *decoded] {
        m.bus.write32(0x040000d4, 0x08395574);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 10);
    }
    ranges(decoded, spans, None);
    for m in [&mut *legacy, &mut *decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    ranges(decoded, spans, Some(rom));
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    let after = decoded
        .bus
        .rom_resources()
        .unwrap()
        .obj_graphics_stats()
        .unwrap()
        .selector_bytes;
    assert_eq!(
        after - before,
        20,
        "DMA must read all five new typed pointers"
    );
}
#[test]
#[ignore = "focused new EUR typed-table DMA provenance; Wine only, Linux/USA pending"]
fn european_obj_graphics_dma_reads_new_typed_table() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let spans = spans(files.obj_graphics.as_ref().unwrap());
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &dataset()).unwrap();
    dma_table(&mut legacy, &mut decoded, &spans, &rom);
    report(
        "dma-result.json",
        serde_json::json!({"source":"395574","typed_selector_bytes":20,"poisoned_ranges":45,"complete_state_clock_equal":true,"corrected_fixture_address":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
