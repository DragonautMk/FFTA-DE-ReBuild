//! Focused Wine proof of the newly published u16-task bank fields.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
const TASK: u32 = 0x02010000;
const SP: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_WORD_OBJ_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn fresh(rom: &[u8]) -> Machine {
    let mut m = crate::make_machine(rom.to_vec(), None);
    m.bus.ffta_sprites.enabled = false;
    m.bus.iwram[0x5f28..0x6d68].copy_from_slice(&rom[0xd968e8..0xd97728]);
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = SP;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m
}
fn ranges(m: &mut Machine, rom: &[u8], data: &WordTaskObjAnimations, poison: bool) {
    for (at, bytes, _) in data.source_spans().unwrap() {
        let region = &mut m.bus.rom[at as usize..at as usize + bytes.len()];
        if poison {
            region.fill(0xcd);
        } else {
            region.copy_from_slice(&rom[at as usize..at as usize + bytes.len()]);
        }
    }
}
thread_local! {static COMPILED_BYTES:Cell<u64>=const{Cell::new(0)};}
fn source_bytes(m: *mut std::ffi::c_void) -> u64 {
    unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .word_task_obj_stats()
            .unwrap()
            .source_bytes
    }
}
extern "C" fn read16(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = source_bytes(m);
    let value = (RT_API.read16)(m, at);
    COMPILED_BYTES.with(|n| n.set(n.get() + source_bytes(m) - before));
    value
}
extern "C" fn read32(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = source_bytes(m);
    let value = (RT_API.read32)(m, at);
    COMPILED_BYTES.with(|n| n.set(n.get() + source_bytes(m) - before));
    value
}
#[derive(Clone, Copy)]
enum Call {
    Constructor(usize, u16),
    Update(usize, u16, u16, u8),
}
fn setup(m: &mut Machine, data: &WordTaskObjAnimations, call: Call) {
    match call {
        Call::Constructor(i, selector) => {
            let b = &data.banks[i];
            m.cpu.regs[..4].copy_from_slice(&[
                TASK,
                0x08000000 + b.object_root,
                0x08000000 + b.sequence_root,
                u32::from(selector),
            ]);
            m.cpu.regs[15] = 0x0802c5e0;
            m.bus.write32(SP, 0x102);
        }
        Call::Update(i, phase, tick, repeat) => {
            let b = &data.banks[i];
            let s = &b.sequences[0];
            for (offset, value) in [
                (0, 0x08000000 + b.object_root),
                (4, 0x08000000 + b.sequence_root),
                (12, 0x08000000 + s.source_offset + 2),
            ] {
                m.bus.write32(TASK + offset, value);
            }
            for (offset, value) in [
                (8, s.frames.len() as u16),
                (16, 0),
                (18, tick),
                (20, phase),
                (24, 0x5544),
            ] {
                m.bus.write16(TASK + offset, value);
            }
            m.bus.write8(TASK + 22, repeat);
            m.cpu.regs[0] = TASK;
            m.cpu.regs[15] = 0x0802c60c;
        }
    }
}
fn interpreted(m: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..120 {
        if m.cpu.regs[15] == RETURN {
            return;
        }
        assert!(m.cpu.thumb());
        visited.insert((m.cpu.regs[15], true));
        m.bus.frame_ready = false;
        m.step();
    }
    panic!("original u16-task return bound");
}
fn native_prefixes(rom: &[u8], data: &WordTaskObjAnimations) -> serde_json::Value {
    let mut calls = Vec::new();
    for (i, b) in data.banks.iter().enumerate() {
        for selector in 0..b.sequences.len() {
            calls.push(Call::Constructor(i, selector as u16));
        }
        let s = &b.sequences[0];
        let last = (s.frames.len() - 1) as u16;
        for (phase, tick, repeat) in [
            (0, 0, 0),
            (0, 255, 0),
            (0, 65535, 0),
            (last, s.frames[last as usize].duration, 0),
            (last, s.frames[last as usize].duration, 0x80),
        ] {
            calls.push(Call::Update(i, phase, tick, repeat));
        }
    }
    assert_eq!(calls.len(), 27);
    let mut raw = fresh(rom);
    let initial = raw.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut expected = Vec::new();
    for &call in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, data, call);
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
    let object = root().join("word-task-obj.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut typed = fresh(rom);
    let mut native = fresh(rom);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read16,
        read32,
        ..RT_API
    };
    COMPILED_BYTES.with(|n| n.set(0));
    let mut dispatches = 0;
    for (i, &call) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            ranges(m, rom, data, false);
            m.load_state(&initial).unwrap();
            setup(m, data, call);
            ranges(m, rom, data, true);
        }
        interpreted(&mut typed, &mut BTreeSet::new());
        let before = COMPILED_BYTES.with(Cell::get);
        let mut steps = 0;
        while native.cpu.regs[15] != RETURN {
            assert!(steps < 120);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
            steps += 1;
        }
        assert_eq!(
            COMPILED_BYTES.with(Cell::get) - before,
            match call {
                Call::Constructor(..) => 6,
                Call::Update(..) => 8,
            }
        );
        for m in [&mut typed, &mut native] {
            ranges(m, rom, data, false);
            assert_eq!(
                m.save_state().unwrap(),
                expected[i],
                "complete state-clock prefix{i}"
            );
        }
    }
    assert_eq!(COMPILED_BYTES.with(Cell::get), 202);
    serde_json::json!({"status":"PASS","full_state_clock_prefixes":27,"constructors":7,"updates":20,"explicit_compiled_source_bytes":202,"compiled_instructions":visited.len(),"dispatches_coverage_only":dispatches,"raw65spans_poisoned":true})
}
fn bus_lifecycle_transfers(rom: &[u8], data: &WordTaskObjAnimations) -> serde_json::Value {
    let mut m = fresh(rom);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    let spans = data.source_spans().unwrap();
    let typed: BTreeMap<u32, u8> = spans
        .iter()
        .flat_map(|(at, b, _)| b.iter().enumerate().map(move |(i, &v)| (*at + i as u32, v)))
        .collect();
    ranges(&mut m, rom, data, true);
    for (&at, &byte) in &typed {
        assert_eq!(m.bus.read8(0x0c000000 + at), byte);
    }
    let stats = owner.word_task_obj_stats().unwrap();
    assert_eq!(stats.source_bytes, 1378);
    assert_eq!(
        stats.source_bytes,
        stats.index_bytes
            + stats.count_bytes
            + stats.attribute_bytes
            + stats.sequence_header_bytes
            + stats.frame_list_bytes
            + stats.duration_bytes
    );
    assert!([
        stats.index_bytes,
        stats.count_bytes,
        stats.attribute_bytes,
        stats.sequence_header_bytes,
        stats.frame_list_bytes,
        stats.duration_bytes
    ]
    .into_iter()
    .all(|n| n > 0));
    let original = Extraction::from_rom(rom).unwrap();
    let old_spans: Vec<_> = original.manifest.resources[..68]
        .iter()
        .flat_map(|r| r.sources.iter().map(|s| (s.offset, s.offset + s.length)))
        .collect();
    let mut edges = BTreeSet::new();
    for (at, b, _) in &spans {
        for x in [at - 1, *at, *at + b.len() as u32 - 2, *at + b.len() as u32] {
            edges.insert(x);
        }
    }
    let mut raw_gaps = BTreeSet::new();
    for &at in &edges {
        for x in at..at + 4 {
            if !typed.contains_key(&x) && !old_spans.iter().any(|&(a, b)| a <= x && x < b) {
                raw_gaps.insert(x);
            }
        }
    }
    assert!(!raw_gaps.is_empty());
    for &at in &raw_gaps {
        m.bus.rom[at as usize] ^= 0x55;
    }
    for (i, &at) in edges.iter().enumerate() {
        let expected = std::array::from_fn(|j| {
            let x = at + j as u32;
            typed
                .get(&x)
                .copied()
                .unwrap_or_else(|| rom[x as usize] ^ if raw_gaps.contains(&x) { 0x55 } else { 0 })
        });
        let ptr = &mut m as *mut Machine as *mut std::ffi::c_void;
        assert_eq!(
            (RT_API.read32)(ptr, [0x08000000, 0x0a000000, 0x0c000000][i % 3] + at),
            u32::from_le_bytes(expected),
            "mixed span/padding read {at:x}"
        );
    }
    // One old animation read proves its preserved page wrapper still composes.
    let old = original.obj_animations.as_ref().unwrap().decoded.banks[0].object_root;
    m.bus.rom[old as usize..old as usize + 4].fill(0xa9);
    assert_eq!(
        (RT_API.read32)(
            &mut m as *mut Machine as *mut std::ffi::c_void,
            0x08000000 + old
        ),
        u32::from_le_bytes(rom[old as usize..old as usize + 4].try_into().unwrap())
    );
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
    ranges(&mut m, rom, data, true);
    assert_eq!(
        m.bus.read8(0x08000000 + data.banks[0].object_root),
        rom[data.banks[0].object_root as usize]
    );
    m.bus.rom.copy_from_slice(rom);
    fs::rename(&unavailable, dataset()).unwrap();
    let mut raw = fresh(rom);
    let mut owned = fresh(rom);
    attach_from_directory(&mut owned, &dataset()).unwrap();
    let initial = raw.save_state().unwrap();
    let base = data.banks[2].object_root;
    let mut transfers = 0;
    for kind in 0..3 {
        for m in [&mut raw, &mut owned] {
            m.bus.rom.copy_from_slice(rom);
            m.load_state(&initial).unwrap();
            if kind == 0 {
                m.bus.write32(0x040000d4, 0x08000000 + base);
                m.bus.write32(0x040000d8, 0x02032000);
                m.bus.write16(0x040000dc, 32);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + base,
                    0x02032400,
                    if kind == 2 { 16 } else { 32 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        ranges(&mut owned, rom, data, true);
        let before = owned
            .bus
            .rom_resources()
            .unwrap()
            .word_task_obj_stats()
            .unwrap()
            .source_bytes;
        for m in [&mut raw, &mut owned] {
            if kind == 0 {
                m.bus.write16(0x040000de, 0x8000);
            } else {
                m.step();
            }
        }
        assert!(
            owned
                .bus
                .rom_resources()
                .unwrap()
                .word_task_obj_stats()
                .unwrap()
                .source_bytes
                > before
        );
        ranges(&mut owned, rom, data, false);
        assert_eq!(
            owned.save_state().unwrap(),
            raw.save_state().unwrap(),
            "transfer{kind} state-clock"
        );
        transfers += 1;
    }
    serde_json::json!({"status":"PASS","typed_bytes":1378,"field_classes":6,"mixed_reads":edges.len(),"raw_gap_bytes":raw_gaps.len(),"old_animation_wrapper":true,"missing_files_load_slot_undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":transfers})
}
fn host_lists(rom: &[u8], data: &WordTaskObjAnimations) -> serde_json::Value {
    let mut raw = fresh(rom);
    let mut typed = fresh(rom);
    attach_from_directory(&mut typed, &dataset()).unwrap();
    ranges(&mut typed, rom, data, true);
    let owner = typed.bus.rom_resources().unwrap().clone();
    let mut checks = 0;
    let mut descriptors = 0;
    gba_core::mem::set_extra(120);
    for list in data.banks.iter().flat_map(|b| &b.lists) {
        let mut oam = vec![0u8; 1024];
        for (i, object) in list.objects.iter().enumerate() {
            let [a0, a1, a2] = object.attributes();
            let attrs = [
                (a0 & 0xff00) | ((64 + i32::from(object.y)) as u16 & 255),
                (a1 & !511) | ((100 + i32::from(object.x)) as u16 & 511),
                a2,
            ];
            for (j, attr) in attrs.into_iter().enumerate() {
                oam[i * 8 + j * 2..i * 8 + j * 2 + 2].copy_from_slice(&attr.to_le_bytes());
            }
        }
        let before = owner.word_task_obj_stats().unwrap().host_reads;
        for m in [&mut raw, &mut typed] {
            m.bus.frames = checks as u64 + 1;
            m.cpu.regs[..4].copy_from_slice(&[0x08000000 + list.source_offset, 100, 64, 0]);
            m.cpu.regs[15] = 0x030065a8;
            for (i, value) in [0, 0, 0, 0, 0, 0x02020000, 0, 0].into_iter().enumerate() {
                m.bus.write32(SP + i as u32 * 4, value);
            }
            m.bus.ffta_sprites.enabled = true;
            m.ffta_sprite_entry();
            m.bus
                .ffta_sprites
                .publish(0x03000030, 0x07000000, 1024, true, &oam);
        }
        assert!(
            owner.word_task_obj_stats().unwrap().host_reads > before,
            "typed host list {:x}",
            list.source_offset
        );
        assert_eq!(
            format!("{:?}", typed.bus.ffta_sprites.active),
            format!("{:?}", raw.bus.ffta_sprites.active),
            "host descriptor equality {:x}",
            list.source_offset
        );
        for m in [&raw, &typed] {
            assert_eq!(
                m.bus
                    .ffta_sprites
                    .active
                    .as_ref()
                    .expect("OAM publication accepted")
                    .len(),
                list.objects.len(),
                "one publication per list {:x}",
                list.source_offset
            );
        }
        descriptors += raw.bus.ffta_sprites.active.as_ref().map_or(0, |v| v.len());
        checks += 1;
    }
    gba_core::mem::set_extra(0);
    assert_eq!(checks, 50);
    assert_eq!(descriptors, 140);
    serde_json::json!({"status":"PASS","lists":checks,"descriptors":descriptors,"typed_host_reads":owner.word_task_obj_stats().unwrap().host_reads,"all65rawspans_poisoned":true})
}
#[test]
#[ignore = "private EUR e77 u16-task OBJ managed/runtime; Wine only, Linux/USA pending"]
fn european_word_task_obj_managed_native_bus_host_lifecycle() {
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
        (76, 75, 68)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (77, 76, 69)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 70);
    assert_eq!(&source.manifest.resources[..68], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..68], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[68];
    assert_eq!(entry.id, WORD_TASK_OBJ_ANIMATIONS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(
        (
            entry.sources.len(),
            entry.sources.iter().map(|s| s.length).sum::<u32>()
        ),
        (65, 1378)
    );
    let bytes = fs::read(dataset().join(WORD_TASK_OBJ_ANIMATIONS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_word_task_obj_animations(&bytes).unwrap(),
        source.word_task_obj_animations.as_ref().unwrap().decoded
    );
    assert_eq!(
        bytes,
        fs::read(std::env::var_os("FFTA_WORD_OBJ_ACCEPTED_MODEL").unwrap()).unwrap()
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == WORD_TASK_OBJ_ANIMATIONS_FILE {
            reached = true;
            Err("injected word-task OBJ-file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_WORD_OBJ_STAGE publication70 resources69 prior68 exact interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..9 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=2 => {
                let mut v = parse_word_task_obj_animations(&data).unwrap();
                match case {
                    0 => v.banks[0].sequences[0].frames[0].duration ^= 1,
                    1 => {
                        v.banks.pop();
                    }
                    _ => v.banks[0].object_root += 4,
                };
                data = encode_word_task_obj_animations(&v).unwrap();
            }
            3 => bad.resources[68].sources[0].offset += 1,
            4 => bad.resources[68].sources[0].length -= 1,
            5 => bad.resources[68].sources[0].sha256 = "00".repeat(32),
            6 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            7 => bad.resources[68]
                .dependencies
                .push(OBJ_STATIC_LISTS_ID.into()),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[68].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(WORD_TASK_OBJ_ANIMATIONS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "word-task OBJ attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(WORD_TASK_OBJ_ANIMATIONS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.word_task_obj_animations.take();
    assert!(source.validate_files(files.files()).is_err());
    source.word_task_obj_animations = saved;
    let mut missing = files.files();
    missing.word_task_obj_animations = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_WORD_OBJ_STAGE attach9 fullset2 failed-owner-fullstate exact PASS");

    let data = &source.word_task_obj_animations.as_ref().unwrap().decoded;
    let managed = serde_json::json!({"status":"PASS","revision":[1,77,76],"files":70,"resources":69,"prior68_records_files_exact":true,"source_bytes":1378,"attach_guards":9,"full_set_guards":2,"new_file_interruption":true,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"word_task_JSON_sha256":gba_core::ffta_rom::digest(&bytes)});
    fs::write(
        root.join("managed-result.json"),
        serde_json::to_vec_pretty(&managed).unwrap(),
    )
    .unwrap();
    let native = native_prefixes(&rom, data);
    fs::write(
        root.join("native-result.json"),
        serde_json::to_vec_pretty(&native).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORD_OBJ_STAGE native27 compiled202 full-state-clock PASS");
    let bus = bus_lifecycle_transfers(&rom, data);
    fs::write(
        root.join("bus-result.json"),
        serde_json::to_vec_pretty(&bus).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORD_OBJ_STAGE bus1378 mixed/padding old-owner lifecycle DMA3 PASS");
    let host = host_lists(&rom, data);
    fs::write(
        root.join("host-result.json"),
        serde_json::to_vec_pretty(&host).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORD_OBJ_STAGE host50 PASS");
    let result = serde_json::json!({"status":"PASS","managed":managed,"native":native,"bus_lifecycle":bus,"host":host,"actual_CLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORD_OBJ_RUNTIME e77-d76 files70 resources69 attach9 fullset2 native27 DMA3 host50 PASS");
}

#[test]
#[ignore = "only missing e77 host-list stage after terminal interrupted filter; Wine only"]
fn european_word_task_obj_remaining_host() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    for name in [
        "managed-result.json",
        "native-result.json",
        "bus-result.json",
    ] {
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(root().join(name)).unwrap()).unwrap();
        assert_eq!(value["status"], "PASS");
    }
    assert!(!root().join("host-result.json").exists());
    let bytes = fs::read(dataset().join(WORD_TASK_OBJ_ANIMATIONS_FILE)).unwrap();
    let data = parse_word_task_obj_animations(&bytes).unwrap();
    let host = host_lists(&rom, &data);
    fs::write(
        root().join("host-result.json"),
        serde_json::to_vec_pretty(&host).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_WORD_OBJ_REMAINING host50 PASS; prior managed/native/bus stages not replayed"
    );
}
