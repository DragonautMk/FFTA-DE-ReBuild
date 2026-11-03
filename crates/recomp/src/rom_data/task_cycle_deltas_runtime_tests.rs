//! One focused Wine filter for the newly published typed cycle bytes.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{observed_task_parameters::PHASE_SOURCE, task_cycle_deltas::SOURCE},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_TASK_CYCLE_RUNTIME_ROOT").unwrap())
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
            .task_cycle_deltas_stats()
            .unwrap()
            .source_bytes
    };
    let before = count();
    let value = (RT_API.read8)(m, at);
    COMPILED_BYTES.with(|n| n.set(n.get() + count() - before));
    value
}
fn setup(m: &mut Machine, phase: u8, subtract: bool) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[4] = 0x02010000;
    m.cpu.regs[5] = if subtract { 8 } else { 0 };
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08018df8;
    m.cpu.cpsr = 0xbf;
    m.bus
        .write16(0x02010008, if subtract { 0 } else { u16::MAX });
    m.bus.write8(0x02010012, phase);
}
fn interpreted(m: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    let mut steps = 0;
    while m.cpu.regs[15] != 0x08018e40 {
        assert!(steps < 160);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
}
fn native_prefixes(rom: &[u8]) -> serde_json::Value {
    let calls = [
        (11, false),
        (12, true),
        (32, false),
        (35, true),
        (47, false),
        (48, false),
        (254, true),
        (255, true),
    ];
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut expected = Vec::new();
    for &(phase, subtract) in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, phase, subtract);
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
    let object = root().join("task-cycle.o");
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
    let api = gba_core::capi::RtApi { read8, ..RT_API };
    COMPILED_BYTES.with(|n| n.set(0));
    let mut dispatches = 0;
    for (i, &(phase, subtract)) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            // ROM identity must be restored BEFORE every state load.
            m.load_state(&initial).unwrap();
            setup(m, phase, subtract);
            m.bus.rom[SOURCE as usize..SOURCE as usize + 48].fill(0xcd);
        }
        interpreted(&mut typed, &mut BTreeSet::new());
        let before = COMPILED_BYTES.with(Cell::get);
        while native.cpu.regs[15] != 0x08018e40 {
            assert!(dispatches < 1500);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original cycle instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        assert_eq!(
            COMPILED_BYTES.with(Cell::get) - before,
            u64::from(phase < 48)
        );
        for m in [&mut typed, &mut native] {
            m.bus.rom[SOURCE as usize..SOURCE as usize + 48]
                .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + 48]);
            assert_eq!(
                m.save_state().unwrap(),
                expected[i],
                "complete state and guest clock for phase{phase}"
            );
        }
    }
    assert_eq!(COMPILED_BYTES.with(Cell::get), 5);
    serde_json::json!({"full_state_clock_prefixes":8,"explicit_compiled_source_bytes":5,
        "compiled_instructions":visited.len(),"dispatches_coverage_only":dispatches,
        "raw48bytes_poisoned":true,"outside48_254_255_prior_raw":true})
}
fn bus_transfers_lifecycle(rom: &[u8]) -> serde_json::Value {
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    m.bus.rom[SOURCE as usize..SOURCE as usize + 48].fill(0xcd);
    m.bus.rom[PHASE_SOURCE as usize..PHASE_SOURCE as usize + 3].fill(0xab);
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    assert_eq!(
        (RT_API.read32)(pointer, 0x0a000000 + SOURCE + 46),
        u32::from_le_bytes(
            rom[SOURCE as usize + 46..SOURCE as usize + 50]
                .try_into()
                .unwrap()
        )
    );
    let n = owner.task_cycle_deltas_stats().unwrap();
    let p = owner.observed_task_parameters_stats().unwrap();
    assert_eq!(
        (
            n.source_bytes,
            n.phases_used,
            p.phase_threshold_bytes,
            p.phases_used
        ),
        (2, 2, 2, 2)
    );
    for i in 0..48 {
        assert_eq!(
            m.bus.read8(0x0c000000 + SOURCE + i),
            rom[SOURCE as usize + i as usize]
        );
    }
    assert_eq!(owner.task_cycle_deltas_stats().unwrap().phases_used, 48);
    // Outer raw edges remain raw; the right edge is the earlier typed owner.
    m.bus.rom[SOURCE as usize - 1] ^= 0x55;
    assert_eq!(
        (RT_API.read32)(pointer, 0x08000000 + SOURCE - 1),
        u32::from_le_bytes([
            rom[SOURCE as usize - 1] ^ 0x55,
            rom[SOURCE as usize],
            rom[SOURCE as usize + 1],
            rom[SOURCE as usize + 2]
        ])
    );
    m.bus.rom[PHASE_SOURCE as usize + 3] ^= 0x33;
    assert_eq!(
        (RT_API.read32)(pointer, 0x0c000000 + PHASE_SOURCE + 1),
        u32::from_le_bytes([
            rom[PHASE_SOURCE as usize + 1],
            rom[PHASE_SOURCE as usize + 2],
            rom[PHASE_SOURCE as usize + 3] ^ 0x33,
            rom[PHASE_SOURCE as usize + 4]
        ])
    );
    for phase in [48, 49, 50, 254, 255] {
        assert_eq!(
            m.bus.read8(0x08000000 + SOURCE + phase),
            rom[SOURCE as usize + phase as usize]
        );
    }
    m.bus.rom.copy_from_slice(rom);
    // Attached host owner survives load/slot Undo, including when files vanish.
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
    m.bus.rom[SOURCE as usize..SOURCE as usize + 48].fill(0xcd);
    assert_eq!(m.bus.read8(0x08000000 + SOURCE + 32), 255);
    m.bus.rom.copy_from_slice(rom);
    fs::rename(&unavailable, dataset()).unwrap();
    let mut transfers = 0;
    for kind in 0..3 {
        let mut a = crate::make_machine(rom.to_vec(), None);
        let mut b = crate::make_machine(rom.to_vec(), None);
        attach_from_directory(&mut b, &dataset()).unwrap();
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write32(0x040000d4, 0x08000000 + SOURCE);
                m.bus.write32(0x040000d8, 0x02032000);
                m.bus.write16(0x040000dc, 25);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02032400,
                    if kind == 2 { 16 } else { 25 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        b.bus.rom[SOURCE as usize..SOURCE as usize + 48].fill(0xcd);
        b.bus.rom[PHASE_SOURCE as usize..PHASE_SOURCE as usize + 3].fill(0xab);
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
            "new two-owner transfer kind{kind}"
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .task_cycle_deltas_stats()
                .unwrap()
                .source_bytes
                >= 48
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .observed_task_parameters_stats()
                .unwrap()
                .phase_threshold_bytes
                > 0
        );
        transfers += 1;
    }
    serde_json::json!({"all48typed_bytes":true,"mixed46_47_and_old_threshold0_1":true,
        "outside_prior_raw_and_mirrors":true,"unavailable_files_load_slot_undo_owner_retained":true,
        "two_owner_DMA_CpuSet_CpuFastSet_full_state_clock":transfers})
}

#[test]
#[ignore = "private EUR typed48byte cycle integration; Wine only, Linux/USA pending"]
fn european_task_cycle_deltas_managed_native_and_lifecycle() {
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
        (75, 74, 67)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (76, 75, 68)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 69);
    assert_eq!(&source.manifest.resources[..67], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..67], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[67];
    assert_eq!(entry.id, TASK_CYCLE_DELTAS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(
        (
            entry.sources.len(),
            entry.sources[0].offset,
            entry.sources[0].length
        ),
        (1, SOURCE, 48)
    );
    let bytes = fs::read(dataset().join(TASK_CYCLE_DELTAS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_task_cycle_deltas(&bytes).unwrap(),
        source.task_cycle_deltas.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == TASK_CYCLE_DELTAS_FILE {
            reached = true;
            Err("injected new cycle-file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!(
        "ROM_DATA_TASK_CYCLE_STAGE publication69 resources68 prior67 exact interruption PASS"
    );
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..9 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=2 => {
                let mut v = parse_task_cycle_deltas(&data).unwrap();
                match case {
                    0 => v.entries[0].delta ^= 1,
                    1 => {
                        v.entries.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_task_cycle_deltas(&v).unwrap();
            }
            3 => bad.resources[67].sources[0].offset += 1,
            4 => bad.resources[67].sources[0].length -= 1,
            5 => bad.resources[67].sources[0].sha256 = "00".repeat(32),
            6 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            7 => bad.resources[67]
                .dependencies
                .push(OBJ_STATIC_LISTS_ID.into()),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[67].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(TASK_CYCLE_DELTAS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "new cycle attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(TASK_CYCLE_DELTAS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.task_cycle_deltas.take();
    assert!(source.validate_files(files.files()).is_err());
    source.task_cycle_deltas = saved;
    let mut missing = files.files();
    missing.task_cycle_deltas = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_TASK_CYCLE_STAGE attach9 fullset2 failed-owner-fullstate exact PASS");
    let native = native_prefixes(&rom);
    eprintln!("ROM_DATA_TASK_CYCLE_STAGE native8 fullstate-clock compiled5 source48poisoned PASS");
    let bus = bus_transfers_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,76,75],"files":69,"resources":68,
        "prior67_records_files_exact":true,"source_bytes":48,"attach_guards":9,"full_set_guards":2,
        "new_file_interruption":true,"native":native,"bus_lifecycle":bus,
        "dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),
        "cycle_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actual_CLI":"pending","ordinary_positive":"pending",
        "Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_TASK_CYCLE_RUNTIME e76-d75 files69 resources68 attach9 fullset2 native8 DMA3 PASS");
}
