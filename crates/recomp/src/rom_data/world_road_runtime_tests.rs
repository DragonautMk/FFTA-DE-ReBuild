//! Reduced Wine integration for the newly published world-road owner.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::world_roads::{HEADERS, POINTS_START as SOURCE, SOURCE_BYTES},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_WORLD_ROAD_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static COMPILED_BYTES:Cell<u64>=const{Cell::new(0)};}
extern "C" fn read8(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let count = || unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .world_roads_stats()
            .unwrap()
            .source_bytes
    };
    let before = count();
    let value = (RT_API.read8)(m, at);
    COMPILED_BYTES.with(|n| n.set(n.get() + count() - before));
    value
}
extern "C" fn read32(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let count = || unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .world_roads_stats()
            .unwrap()
            .source_bytes
    };
    let before = count();
    let value = (RT_API.read32)(m, at);
    COMPILED_BYTES.with(|n| n.set(n.get() + count() - before));
    value
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    args: [u32; 3],
    typed: bool,
}
fn setup(m: &mut Machine, call: Call) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    if call.entry == 0x4dd0c {
        m.cpu.regs[0] = 0x02005000;
        m.cpu.regs[1..4].copy_from_slice(&call.args);
    } else {
        m.cpu.regs[..3].copy_from_slice(&call.args);
    }
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08000000 + call.entry;
    m.cpu.cpsr = 0xbf;
}
fn interpreted(m: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    let mut n = 0;
    while m.cpu.regs[15] != 0x08000300 {
        assert!(n < 128);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
}
fn native_prefixes(rom: &[u8]) -> serde_json::Value {
    let cases = [
        (0x4dd0c, [0, 0, 0], true),
        (0x4dd0c, [1, 0, 0], true),
        (0x4dd0c, [255, 44, 0], true),
        (0x4dd0c, [0, 3, 54], true),
        (0x4dd0c, [0, 0, 50], true),
        (0x4dd0c, [1, 0, 255], true),
        (0x4dd0c, [0x100, 0x12c, 0x100], true),
        (0x4dd0c, [0x1ff, 0x100, 0xffffff00], true),
        (0x4dd0c, [0, 45, 255], false),
        (0x4dd0c, [0, 255, 255], false),
        (0x4dd70, [0x800000ff, 0x100, 0], false),
        (0x4dd84, [0x800000ff, 0x100, 0], false),
    ];
    let calls: Vec<_> = cases
        .into_iter()
        .map(|(entry, args, typed)| Call { entry, args, typed })
        .collect();
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut expected = Vec::new();
    for &call in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, call);
        interpreted(&mut raw, &mut visited);
        expected.push(raw.save_state().unwrap());
    }
    let view = View {
        rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(addr, thumb)| Block {
            addr,
            thumb,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root().join("world-road.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut typed = crate::make_machine(rom.to_vec(), None);
    let mut native = crate::make_machine(rom.to_vec(), None);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read32,
        ..RT_API
    };
    COMPILED_BYTES.with(|n| n.set(0));
    let mut dispatch = 0;
    for (i, &call) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            // Restore identity before loading; poison only after load and setup.
            m.load_state(&initial).unwrap();
            setup(m, call);
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
        }
        interpreted(&mut typed, &mut BTreeSet::new());
        let before = COMPILED_BYTES.with(Cell::get);
        while native.cpu.regs[15] != 0x08000300 {
            assert!(dispatch < 2000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original node instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatch += 1;
        }
        let consumed = COMPILED_BYTES.with(Cell::get) - before;
        assert_eq!(
            consumed > 0,
            call.typed,
            "new typed read expected for case{i}"
        );
        for m in [&mut typed, &mut native] {
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]
                .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]);
            assert_eq!(
                m.save_state().unwrap(),
                expected[i],
                "complete state/clock node case{i}"
            );
        }
    }
    serde_json::json!({"full_state_clock_prefixes":calls.len(),"compiled_source_bytes":COMPILED_BYTES.with(Cell::get),"compiled_instructions":visited.len(),"dispatch_coverage_only":dispatch,"source4736_poisoned":true,"outside45_255_raw_and_unsigned_order_helpers":true})
}
fn bus_lifecycle(rom: &[u8]) -> serde_json::Value {
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
    for i in 0..SOURCE_BYTES {
        assert_eq!(
            m.bus.read8(0x0c000000 + SOURCE + i as u32),
            rom[SOURCE as usize + i]
        );
    }
    let s = owner.world_roads_stats().unwrap();
    assert_eq!(
        (
            s.bus_reads,
            s.source_bytes,
            s.count_bytes,
            s.pointer_bytes,
            s.sample_bytes,
            s.opaque_bytes,
            s.paths_used
        ),
        (4736, 4736, 45, 180, 4376, 135, 45)
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for delta in [
        -1,
        1,
        99,
        (HEADERS - SOURCE) as i32 - 1,
        SOURCE_BYTES as i32 - 1,
    ] {
        let at = (SOURCE as i32 + delta) as usize;
        assert_eq!(
            (RT_API.read16)(pointer, 0x0a000000 + at as u32),
            u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
        );
        assert_eq!(
            (RT_API.read32)(pointer, 0x08000000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
    }
    let end = SOURCE as usize + SOURCE_BYTES;
    m.bus.rom[SOURCE as usize - 1] ^= 0x55;
    m.bus.rom[end] ^= 0x33;
    assert_eq!(
        (RT_API.read32)(pointer, 0x08000000 + SOURCE - 1),
        u32::from_le_bytes([
            rom[SOURCE as usize - 1],
            rom[SOURCE as usize],
            rom[SOURCE as usize + 1],
            rom[SOURCE as usize + 2]
        ])
    );
    assert_eq!(
        (RT_API.read32)(pointer, 0x0c000000 + end as u32 - 1),
        u32::from_le_bytes([rom[end - 1], rom[end] ^ 0x33, rom[end + 1], rom[end + 2]])
    );
    assert!(owner.world_nodes_stats().unwrap().opaque_bytes > 0);
    let before = owner.world_roads_stats().unwrap();
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.world_roads_stats().unwrap(), before);
    m.bus.rom.copy_from_slice(rom);
    let unavailable = root().join("unavailable dataset");
    fs::rename(dataset(), &unavailable).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    m.load_state(&initial).unwrap();
    let mut slots = crate::save_state::Slots::new(&root().join("private.sav"), rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 123;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    assert_eq!(m.save_state().unwrap(), initial);
    slots.undo(&mut m).unwrap();
    assert_eq!(m.save_state().unwrap(), changed);
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    m.bus.rom[SOURCE as usize..end].fill(0xcd);
    assert_eq!(m.bus.read8(0x08000000 + SOURCE), rom[SOURCE as usize]);
    m.bus.rom.copy_from_slice(rom);
    fs::rename(&unavailable, dataset()).unwrap();
    for kind in 0..3 {
        let mut a = crate::make_machine(rom.to_vec(), None);
        let mut b = crate::make_machine(rom.to_vec(), None);
        attach_from_directory(&mut b, &dataset()).unwrap();
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write32(0x040000d4, 0x08000000 + SOURCE);
                m.bus.write32(0x040000d8, 0x02032000);
                m.bus.write16(0x040000dc, 2370);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02032000,
                    if kind == 2 { 1192 } else { 2370 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        b.bus.rom[SOURCE as usize..end].fill(0xcd);
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write16(0x040000de, 0x8000);
            } else {
                m.step();
            }
        }
        b.bus.rom.copy_from_slice(rom);
        assert_eq!(
            b.save_state().unwrap(),
            a.save_state().unwrap(),
            "full state/clock node transfer{kind}"
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .world_roads_stats()
                .unwrap()
                .source_bytes
                >= 4736
        );
    }
    serde_json::json!({"all4736_typed_bytes":true,"all4field_classes45paths":true,"widths_mirrors_mixedraw_edges_prior_owner":true,"unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3})
}

#[test]
#[ignore = "private EUR world-road managed/native integration; Wine only, Linux/USA pending"]
fn european_world_roads_managed_native_and_lifecycle() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior: Manifest =
        toml::from_str(&fs::read_to_string(old.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            prior.extractor_revision,
            prior.decoder_revision,
            prior.resources.len()
        ),
        (78, 77, 70)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (79, 78, 71)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 72);
    assert_eq!(&source.manifest.resources[..70], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..70], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[70];
    assert_eq!(entry.id, WORLD_ROADS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 46);
    assert_eq!(entry.sources.iter().map(|s| s.length).sum::<u32>(), 4736);
    assert_eq!(entry.sources.first().unwrap().offset, SOURCE);
    assert_eq!(entry.sources.last().unwrap().offset, HEADERS);
    let bytes = fs::read(dataset().join(WORLD_ROADS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_world_roads(&bytes).unwrap(),
        source.world_roads.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == WORLD_ROADS_FILE {
            reached = true;
            Err("injected world-road file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_WORLD_ROAD_STAGE publication72/prior70exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..12 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_world_roads(&data).unwrap();
                match case {
                    0 => v.paths[0].points[0].x ^= 1,
                    1 => v.paths[0].encoded_pointer ^= 1,
                    2 => v.paths[0].opaque[1] = 255,
                    3 => {
                        v.paths.pop();
                    }
                    _ => v.headers_offset += 1,
                };
                data = encode_world_roads(&v).unwrap();
            }
            5 => bad.resources[70].sources[0].offset += 1,
            6 => bad.resources[70].sources[0].length -= 1,
            7 => bad.resources[70].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[70].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[70]
                .dependencies
                .push(OBJ_STATIC_LISTS_ID.into()),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[70].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(WORLD_ROADS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "world-road attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(WORLD_ROADS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.world_roads.take();
    assert!(source.validate_files(files.files()).is_err());
    source.world_roads = saved;
    let mut missing = files.files();
    missing.world_roads = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_WORLD_ROAD_STAGE attach12/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!("ROM_DATA_WORLD_ROAD_STAGE native12/fullstate-clock/source4736poisoned PASS");
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,79,78],"files":72,"resources":71,"prior70_records_files_exact":true,"source_bytes":4736,"attach_guards":12,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"world_roads_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORLD_ROAD_RUNTIME e79-d78/files72/resources71/attach12/fullset2/native12/DMA3 PASS");
}

#[test]
#[ignore = "private EUR remaining road native/bus/lifecycle proof; Wine only"]
fn european_world_roads_remaining_native_and_lifecycle() {
    let root = root();
    assert!(root.is_dir());
    assert!(!root.join("result.json").exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    let bytes = fs::read(dataset().join(WORLD_ROADS_FILE)).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let native = native_prefixes(&rom);
    eprintln!(
        "ROM_DATA_WORLD_ROAD_STAGE remaining-native12/fullstate-clock/source4736poisoned PASS"
    );
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,79,78],"files":72,"resources":71,"prior70_records_files_exact":true,"source_bytes":4736,"attach_guards":12,"fullset_guards":2,"newfile_interruption":true,"publication_attach_fullset":"cached first filter PASS, not rerun","native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"world_roads_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORLD_ROAD_RUNTIME remaining-native12/bus4736B/DMA3 PASS");
}
