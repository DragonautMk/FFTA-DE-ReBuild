//! One reduced Wine filter for the new e74 table and four-list extension.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::obj_lists::constructor_animation::{LIST_ROOTS, TABLE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

const SLOT: u32 = 0x02010000;
const INSTANCE: u32 = 0x02012000;
const STACK: u32 = 0x03007c00;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_CONSTRUCTOR_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn ranges(m: &mut Machine, rom: &[u8], poison: bool) {
    for (offset, length) in
        std::iter::once((TABLE_OFFSET, 32)).chain(LIST_ROOTS.map(|(o, _)| (o, 14)))
    {
        let at = offset as usize;
        let end = at + length;
        if poison {
            m.bus.rom[at..end].fill(0xcd);
        } else {
            m.bus.rom[at..end].copy_from_slice(&rom[at..end]);
        }
    }
}
fn fresh(rom: &[u8]) -> Machine {
    let mut m = crate::make_machine(rom.to_vec(), None);
    m.bus.ffta_sprites.enabled = false;
    m.bus.iwram[0x5f28..0x6d68].copy_from_slice(&rom[0xd968e8..0xd97728]);
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    m
}
#[derive(Clone, Copy)]
enum Call {
    Counter(u8, u8),
    Pointer(u8),
}
impl Call {
    fn phase(self) -> u8 {
        match self {
            Self::Counter(p, _) | Self::Pointer(p) => p,
        }
    }
    fn stop(self) -> u32 {
        match self {
            Self::Counter(..) => 0x0809db82,
            Self::Pointer(..) => 0x0809dbe0,
        }
    }
}
fn setup(m: &mut Machine, call: Call) {
    m.bus.write8(SLOT + 12, call.phase());
    match call {
        Call::Counter(_, old) => {
            m.cpu.regs[2] = SLOT;
            m.cpu.regs[3] = 0xa5;
            m.cpu.regs[7] = 0x08000000 + TABLE_OFFSET;
            m.cpu.regs[15] = 0x0809db56;
            m.bus.write8(SLOT + 10, 0xa5);
            m.bus.write8(SLOT + 11, old);
        }
        Call::Pointer(_) => {
            m.cpu.regs[4] = SLOT;
            m.cpu.regs[8] = 0x08397f9c;
            m.cpu.regs[9] = 0x08000000 + TABLE_OFFSET;
            m.cpu.regs[15] = 0x0809dbca;
            m.bus.write32(SLOT, INSTANCE);
            m.bus.write32(INSTANCE + 0x154, 0x12345678);
        }
    }
}
fn interpreted(m: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) {
    setup(m, call);
    for _ in 0..60 {
        if m.cpu.regs[15] == call.stop() {
            return;
        }
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
    }
    panic!("new runtime prefix did not return");
}
thread_local! {static NATIVE_BYTES:Cell<u64>=const{Cell::new(0)};}
extern "C" fn read32(m: *mut std::ffi::c_void, address: u32) -> u32 {
    let count = || unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .constructor_obj_animation_stats()
            .unwrap()
            .source_bytes
    };
    let before = count();
    let value = (RT_API.read32)(m, address);
    NATIVE_BYTES.with(|c| c.set(c.get() + count() - before));
    value
}

fn native_and_bus(rom: &[u8], animation: &ConstructorObjAnimation) -> serde_json::Value {
    let output = root().join("native");
    fs::create_dir(&output).unwrap();
    let mut calls = Vec::new();
    for phase in 0..4 {
        calls.push(Call::Counter(
            phase,
            animation.frame(phase).unwrap().threshold as u8,
        ));
        calls.push(Call::Pointer(phase));
    }
    calls.push(Call::Counter(3, 255));
    for phase in [4, 255] {
        calls.push(Call::Counter(phase, 0));
        calls.push(Call::Pointer(phase));
    }
    let mut tracer = fresh(rom);
    let initial = tracer.save_state().unwrap();
    let iwram = tracer.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    for &call in &calls {
        tracer.load_state(&initial).unwrap();
        interpreted(&mut tracer, call, &mut visited);
    }
    let view = View {
        rom,
        ewram: None,
        iwram: Some(&iwram),
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
    let object = output.join("constructor-animation.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut old = fresh(rom);
    let mut typed = fresh(rom);
    let mut native = fresh(rom);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi { read32, ..RT_API };
    NATIVE_BYTES.with(|c| c.set(0));
    let mut dispatches = 0;
    for &call in &calls {
        for m in [&mut old, &mut typed, &mut native] {
            m.load_state(&initial).unwrap();
        }
        interpreted(&mut old, call, &mut BTreeSet::new());
        for m in [&mut typed, &mut native] {
            ranges(m, rom, true);
        }
        interpreted(&mut typed, call, &mut BTreeSet::new());
        setup(&mut native, call);
        let before = native
            .bus
            .rom_resources()
            .unwrap()
            .constructor_obj_animation_stats()
            .unwrap()
            .source_bytes;
        while native.cpu.regs[15] != call.stop() {
            assert!(dispatches < 10000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        let after = native
            .bus
            .rom_resources()
            .unwrap()
            .constructor_obj_animation_stats()
            .unwrap()
            .source_bytes;
        assert_eq!(
            after - before,
            if call.phase() < 4 { 4 } else { 0 },
            "outside phases retain raw queries"
        );
        for m in [&mut typed, &mut native] {
            ranges(m, rom, false);
            assert_eq!(
                m.save_state().unwrap(),
                old.save_state().unwrap(),
                "complete state and clock"
            );
        }
    }
    assert_eq!(NATIVE_BYTES.with(Cell::get), 36);
    let stats = native
        .bus
        .rom_resources()
        .unwrap()
        .constructor_obj_animation_stats()
        .unwrap();
    assert_eq!(
        (
            stats.threshold_bytes,
            stats.pointer_bytes,
            stats.phases_used
        ),
        (20, 16, 4)
    );
    ranges(&mut typed, rom, true);
    let mut bus_checks = 0;
    for bios in [false, true] {
        typed.bus.real_bios = bios;
        for (offset, length) in
            std::iter::once((TABLE_OFFSET, 32)).chain(LIST_ROOTS.map(|(o, _)| (o, 14)))
        {
            for at in offset - 3..offset + length + 3 {
                for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                    assert_eq!(typed.bus.read8(mirror + at), rom[at as usize]);
                    assert_eq!(
                        typed.bus.read16(mirror + at),
                        u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
                    );
                    assert_eq!(
                        (RT_API.read32)(
                            &mut typed as *mut Machine as *mut std::ffi::c_void,
                            mirror + at
                        ),
                        u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                    );
                    bus_checks += 1;
                }
            }
        }
    }
    typed.bus.real_bios = false;
    let owner = typed.bus.rom_resources().unwrap().clone();
    let p = &mut typed as *mut Machine as *mut std::ffi::c_void;
    typed.bus.rom[0x394fba] ^= 0x55;
    typed.bus.rom[0x394fbb] ^= 0x33;
    let mut expected: [u8; 4] = rom[0x394fba..0x394fbe].try_into().unwrap();
    expected[0] ^= 0x55;
    expected[1] ^= 0x33;
    let before = owner
        .constructor_obj_animation_stats()
        .unwrap()
        .source_bytes;
    assert_eq!((RT_API.read32)(p, 0x08394fba), u32::from_le_bytes(expected));
    assert_eq!(
        owner
            .constructor_obj_animation_stats()
            .unwrap()
            .source_bytes
            - before,
        2
    );
    let mut expected: [u8; 4] = rom[0x394fb8..0x394fbc].try_into().unwrap();
    expected[2] ^= 0x55;
    expected[3] ^= 0x33;
    let before = owner.obj_list_stats().unwrap().source_bytes;
    assert_eq!((RT_API.read32)(p, 0x08394fb8), u32::from_le_bytes(expected));
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before, 2);
    typed.bus.rom[0x394f4c..0x394f50].fill(0xcd);
    assert_eq!(
        (RT_API.read32)(p, 0x08394f4c),
        u32::from_le_bytes(rom[0x394f4c..0x394f50].try_into().unwrap()),
        "prior e72 owner retained"
    );
    let remaining = transfers_and_host(rom);
    serde_json::json!({"fullstate_clock_prefixes":calls.len(),"native_typed_bytes":36,"native_dispatches_coverage_only":dispatches,"threshold_bytes":20,"pointer_bytes":16,"phases_used":4,"outside_phase_queries":4,"bus_boundary_mirror_bios_cases":bus_checks,"new_prior_raw_composition":true,"remaining":remaining})
}

fn transfers_and_host(rom: &[u8]) -> serde_json::Value {
    let mut old = fresh(rom);
    let initial = old.save_state().unwrap();
    let mut typed = fresh(rom);
    attach_from_directory(&mut typed, &dataset()).unwrap();
    let owner = typed.bus.rom_resources().unwrap().clone();
    let mut transfers = 0;
    for (base, length) in [(0x394fbcu32, 32u32), (0x394f80, 64)] {
        for kind in 0..3 {
            for m in [&mut old, &mut typed] {
                // ROM identity must be restored before loading the initial state.
                m.bus.rom[0x394f4c..0x394fdc].copy_from_slice(&rom[0x394f4c..0x394fdc]);
                m.load_state(&initial).unwrap();
                if kind == 0 {
                    m.bus.write32(0x040000d4, 0x08000000 + base);
                    m.bus.write32(0x040000d8, 0x02032000);
                    m.bus.write16(0x040000dc, (length / 2) as u16);
                } else {
                    m.cpu.regs[..3].copy_from_slice(&[
                        0x0a000000 + base,
                        0x02032400,
                        if kind == 2 { length / 4 } else { length / 2 },
                    ]);
                    m.cpu.regs[15] = 0x02000000;
                    m.cpu.cpsr = 0xbf;
                    m.bus
                        .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
                }
            }
            ranges(&mut typed, rom, true);
            let before = owner
                .constructor_obj_animation_stats()
                .unwrap()
                .source_bytes;
            for m in [&mut old, &mut typed] {
                if kind == 0 {
                    m.bus.write16(0x040000de, 0x8000);
                } else {
                    m.step();
                }
            }
            assert!(
                owner
                    .constructor_obj_animation_stats()
                    .unwrap()
                    .source_bytes
                    > before
            );
            ranges(&mut typed, rom, false);
            assert_eq!(typed.save_state().unwrap(), old.save_state().unwrap());
            transfers += 1;
        }
    }
    let mut host_checks = 0;
    gba_core::mem::set_extra(120);
    for (offset, _) in LIST_ROOTS {
        let mut a = fresh(rom);
        let mut b = fresh(rom);
        attach_from_directory(&mut b, &dataset()).unwrap();
        ranges(&mut b, rom, true);
        let list = gba_core::rom_data::obj_lists::List::decode(
            offset,
            &rom[offset as usize..offset as usize + 14],
        )
        .unwrap();
        let mut oam = vec![0u8; 1024];
        for (i, object) in list.objects.iter().enumerate() {
            let [a0, a1, a2] = object.attributes();
            let attrs = [
                (a0 & 0xff00) | ((110 + i32::from(object.y)) as u16 & 255),
                (a1 & !511) | ((100 + i32::from(object.x)) as u16 & 511),
                a2,
            ];
            for (j, attr) in attrs.into_iter().enumerate() {
                oam[i * 8 + j * 2..i * 8 + j * 2 + 2].copy_from_slice(&attr.to_le_bytes());
            }
        }
        for m in [&mut a, &mut b] {
            m.cpu.regs[..4].copy_from_slice(&[0x08000000 + offset, 100, 110, 0]);
            m.cpu.regs[15] = 0x030065a8;
            for (i, value) in [0, 0, 0, 0, 0, 0x02020000, 0, 0].into_iter().enumerate() {
                m.bus.write32(STACK + i as u32 * 4, value);
            }
            m.bus.ffta_sprites.enabled = true;
            m.ffta_sprite_entry();
            m.bus
                .ffta_sprites
                .publish(0x03000030, 0x07000000, 1024, true, &oam);
        }
        assert_eq!(a.bus.ffta_sprites.active.as_ref().unwrap().len(), 2);
        assert_eq!(
            format!("{:?}", a.bus.ffta_sprites.active),
            format!("{:?}", b.bus.ffta_sprites.active)
        );
        assert_eq!(
            b.bus.rom_resources().unwrap().obj_list_source_used(offset),
            Some((false, true))
        );
        host_checks += 1;
    }
    gba_core::mem::set_extra(0);
    serde_json::json!({"status":"PASS","SWI_DMA_checks":transfers,"host_publication_checks":host_checks})
}

#[test]
#[ignore = "private EUR e74 constructor animation publication/native/bus; Wine only"]
fn european_constructor_animation_managed_native_and_bus() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = read_dataset(&old).unwrap();
    assert_eq!(
        (
            prior.manifest.extractor_revision,
            prior.manifest.decoder_revision,
            prior.manifest.resources.len()
        ),
        (73, 72, 66)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (74, 73, 67)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 68);
    let lists = parse_obj_lists(&fs::read(dataset().join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            lists.lists.len(),
            lists.lists.iter().map(|l| l.objects.len()).sum::<usize>(),
            lists.lists.iter().map(|l| l.source_len()).sum::<usize>()
        ),
        (80, 193, 1318)
    );
    for list in &prior.obj_lists.as_ref().unwrap().lists {
        assert!(lists.lists.contains(list));
    }
    assert_eq!(gba_core::rom_data::obj_lists::REFERENCES.len(), 126);
    for (i, entry) in prior.manifest.resources.iter().enumerate() {
        assert_eq!(source.manifest.coverage[i], prior.manifest.coverage[i]);
        if entry.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(&source.manifest.resources[i], entry);
            assert_eq!(
                fs::read(dataset().join(&entry.path)).unwrap(),
                fs::read(old.join(&entry.path)).unwrap()
            );
        }
    }
    let entry = &source.manifest.resources[66];
    assert_eq!(entry.id, CONSTRUCTOR_OBJ_ANIMATION_ID);
    assert_eq!(entry.dependencies, vec![OBJ_STATIC_LISTS_ID]);
    assert_eq!((entry.sources.len(), entry.sources[0].length), (1, 32));
    let bytes = fs::read(dataset().join(CONSTRUCTOR_OBJ_ANIMATION_FILE)).unwrap();
    let animation = parse_constructor_obj_animation(&bytes).unwrap();
    assert_eq!(encode_constructor_obj_animation(&animation).unwrap(), bytes);
    animation.validate_lists(&lists).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    let list_bytes = fs::read(dataset().join(OBJ_STATIC_LISTS_FILE)).unwrap();
    let mut m = fresh(&rom);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..10 {
        let mut bad = source.manifest.clone();
        let mut changed = bytes.clone();
        let mut changed_lists = list_bytes.clone();
        match case {
            0 | 1 | 2 => {
                let mut value: serde_json::Value = serde_json::from_slice(&changed).unwrap();
                match case {
                    0 => value["frames"][0]["threshold"] = 99.into(),
                    1 => value["frames"][0]["list_offset"] = 0.into(),
                    _ => value["unexpected"] = true.into(),
                }
                changed = serde_json::to_vec(&value).unwrap();
            }
            3 => bad.resources[66].sources[0].length -= 1,
            4 => bad.resources[66].sources[0].sha256 = "00".repeat(32),
            5 => bad.resources[66].dependencies.clear(),
            6 => bad.decoder_revision -= 1,
            7 => {
                changed.pop();
            }
            _ => {
                let mut value = lists.clone();
                let at = value
                    .lists
                    .iter()
                    .position(|l| l.source_offset == LIST_ROOTS[0].0)
                    .unwrap();
                if case == 8 {
                    value.lists[at].objects[0].tile ^= 1;
                } else {
                    value.lists.remove(at);
                }
                changed_lists = encode_obj_lists(&value).unwrap();
                let index = bad
                    .resources
                    .iter()
                    .position(|r| r.id == OBJ_STATIC_LISTS_ID)
                    .unwrap();
                bad.resources[index].sha256 = gba_core::ffta_rom::digest(&changed_lists);
            }
        }
        bad.resources[66].sha256 = gba_core::ffta_rom::digest(&changed);
        fs::write(dataset().join(CONSTRUCTOR_OBJ_ANIMATION_FILE), changed).unwrap();
        fs::write(dataset().join(OBJ_STATIC_LISTS_FILE), changed_lists).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(CONSTRUCTOR_OBJ_ANIMATION_FILE), &bytes).unwrap();
    fs::write(dataset().join(OBJ_STATIC_LISTS_FILE), &list_bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let held = source.constructor_obj_animation.take();
    assert!(source.validate_files(files.files()).is_err());
    source.constructor_obj_animation = held;
    let mut missing = files.files();
    missing.constructor_obj_animation = None;
    assert!(source.validate_files(missing).is_err());
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut writes = 0;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        writes += 1;
        if writes == 4 {
            Err("injected publication failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = root.join("resources");
    let prepared = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e74-d73"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(CONSTRUCTOR_OBJ_ANIMATION_FILE),
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
    let moved = root.join("moved resources");
    fs::rename(&managed, &moved).unwrap();
    let relocated = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(relocated.action, "reused");
    assert_eq!(relocated.directory, moved.join(relative));
    let unavailable = root.join("unavailable dataset");
    fs::rename(dataset(), &unavailable).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    m.cpu.regs[4] = 99;
    m.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    ranges(&mut m, &rom, true);
    for (at, n) in std::iter::once((TABLE_OFFSET, 32)).chain(LIST_ROOTS.map(|(o, _)| (o, 14))) {
        for offset in at..at + n {
            assert_eq!(m.bus.read8(0x08000000 + offset), rom[offset as usize]);
        }
    }
    fs::rename(&unavailable, dataset()).unwrap();
    let native = native_and_bus(&rom, &animation);
    let result = serde_json::json!({"status":"PASS","revision":[1,74,73],"files":68,"resources":67,"old65records_files_and76lists_retained":true,"lists":80,"objects":193,"list_source_bytes":1318,"new_table_bytes":32,"new_list_bytes":56,"attach_guards":10,"full_set_guards":2,"create_reuse_repair_move_interruption":true,"failed_attach_state_owner_exact":true,"unavailable_directory_state_load_owner_retained":true,"native":native,"Linux":"pending","USA":"pending","nativeWindows":"pending","ordinary_positive":"not yet observed","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONSTRUCTOR_RUNTIME e74-d73/files68/resources67/lists80/objects193 table32/newlists56 guards10/fullset2 native13/DMA6/host4 PASS");
}

#[test]
#[ignore = "resume only missing e74 DMA/host checks using retained authenticated offline dataset; Wine only"]
fn european_constructor_animation_remaining_dma_and_host() {
    assert!(!root().join("remaining-result.json").exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let result = transfers_and_host(&rom);
    assert_eq!(result["SWI_DMA_checks"], 6);
    assert_eq!(result["host_publication_checks"], 4);
    fs::write(
        root().join("remaining-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONSTRUCTOR_REMAINING DMA6/host4 PASS; no managed/native-prefix replay");
}
