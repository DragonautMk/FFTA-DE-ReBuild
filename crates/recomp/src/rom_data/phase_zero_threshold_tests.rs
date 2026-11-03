//! One Wine filter for the additional conditional phase-zero byte.
use super::observed_task_parameters_runtime_tests::{interpreted, setup, stop, Call};
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::observed_task_parameters::{
        Catalog, ObservedTaskParameters, PHASE_COUNT, PHASE_SOURCE, VERTICAL_SOURCE,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_PHASE_ZERO_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static COMPILED_BYTES:Cell<u64>=const{Cell::new(0)};}
extern "C" fn read8(m: *mut std::ffi::c_void, address: u32) -> u32 {
    let count = || unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .observed_task_parameters_stats()
            .unwrap()
            .source_bytes
    };
    let before = count();
    let value = (RT_API.read8)(m, address);
    COMPILED_BYTES.with(|n| n.set(n.get() + count() - before));
    value
}

fn native_phase_zero(rom: &[u8]) -> serde_json::Value {
    let mut tracer = crate::make_machine(rom.to_vec(), None);
    let initial = tracer.save_state().unwrap();
    let calls = [
        Call::Phase(0, 0),
        Call::Phase(0, 15),
        Call::Phase(0, 16),
        Call::Phase(0, u16::MAX),
        Call::Phase(3, 0),
        Call::Phase(255, u16::MAX),
    ];
    let mut visited = BTreeSet::new();
    for &call in &calls {
        tracer.load_state(&initial).unwrap();
        interpreted(&mut tracer, call, &mut visited);
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
    let object = root().join("phase-zero.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let mut typed = crate::make_machine(rom.to_vec(), None);
    let mut native = crate::make_machine(rom.to_vec(), None);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi { read8, ..RT_API };
    COMPILED_BYTES.with(|n| n.set(0));
    let mut dispatches = 0;
    for (index, &call) in calls.iter().enumerate() {
        for m in [&mut raw, &mut typed, &mut native] {
            m.load_state(&initial).unwrap();
        }
        interpreted(&mut raw, call, &mut BTreeSet::new());
        for m in [&mut typed, &mut native] {
            m.bus.rom[PHASE_SOURCE as usize] = 0xcd;
        }
        interpreted(&mut typed, call, &mut BTreeSet::new());
        setup(&mut native, call);
        let before = COMPILED_BYTES.with(Cell::get);
        while native.cpu.regs[15] != stop(call) {
            assert!(dispatches < 1000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("new phase0 instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        assert_eq!(
            COMPILED_BYTES.with(Cell::get) - before,
            if index < 4 { 1 } else { 0 }
        );
        for m in [&mut typed, &mut native] {
            m.bus.rom[PHASE_SOURCE as usize] = rom[PHASE_SOURCE as usize];
            assert_eq!(
                m.save_state().unwrap(),
                raw.save_state().unwrap(),
                "new phase0 full state and clock"
            );
        }
    }
    assert_eq!(COMPILED_BYTES.with(Cell::get), 4);
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    m.bus.rom[PHASE_SOURCE as usize..PHASE_SOURCE as usize + 3].fill(0xcd);
    m.bus.rom[VERTICAL_SOURCE as usize..VERTICAL_SOURCE as usize + 8].fill(0xcd);
    assert_eq!(m.bus.read8(0x08000000 + PHASE_SOURCE), 16);
    let s = owner.observed_task_parameters_stats().unwrap();
    assert_eq!((s.phases_used, s.corrections_used), (1, 0));
    assert_eq!(m.bus.read16(0x08000000 + VERTICAL_SOURCE), 40);
    let s = owner.observed_task_parameters_stats().unwrap();
    assert_eq!((s.phases_used, s.corrections_used), (1, 1));
    for phase in 1..3 {
        assert_eq!(m.bus.read8(0x08000000 + PHASE_SOURCE + phase), 8);
    }
    for row in 1..4 {
        assert_eq!(
            m.bus.read16(0x08000000 + VERTICAL_SOURCE + row * 2),
            if row == 1 { 40 } else { 41 }
        );
    }
    let s = owner.observed_task_parameters_stats().unwrap();
    assert_eq!(
        (s.phases_used, s.corrections_used, s.source_bytes),
        (3, 4, 11)
    );
    m.bus.rom[PHASE_SOURCE as usize - 1] ^= 0x55;
    m.bus.rom[PHASE_SOURCE as usize + 3] ^= 0x33;
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for (at, positions) in [
        (PHASE_SOURCE - 1, vec![(0, 0x55)]),
        (PHASE_SOURCE + 1, vec![(2, 0x33)]),
    ] {
        let mut expected: [u8; 4] = rom[at as usize..at as usize + 4].try_into().unwrap();
        for (i, mask) in positions {
            expected[i] ^= mask;
        }
        assert_eq!(
            (RT_API.read32)(pointer, 0x0a000000 + at),
            u32::from_le_bytes(expected)
        );
    }
    let mut transfers = 0;
    for kind in 0..3 {
        let mut a = crate::make_machine(rom.to_vec(), None);
        let mut b = crate::make_machine(rom.to_vec(), None);
        attach_from_directory(&mut b, &dataset()).unwrap();
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write32(0x040000d4, 0x08000000 + PHASE_SOURCE);
                m.bus.write32(0x040000d8, 0x02032000);
                m.bus.write16(0x040000dc, 2);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0c000000 + PHASE_SOURCE,
                    0x02032400,
                    if kind == 2 { 8 } else { 2 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        b.bus.rom[PHASE_SOURCE as usize] = 0xcd;
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write16(0x040000de, 0x8000);
            } else {
                m.step();
            }
        }
        assert_eq!(
            b.bus
                .rom_resources()
                .unwrap()
                .observed_task_parameters_stats()
                .unwrap()
                .phase_threshold_bytes,
            3
        );
        b.bus.rom[PHASE_SOURCE as usize] = rom[PHASE_SOURCE as usize];
        assert_eq!(a.save_state().unwrap(), b.save_state().unwrap());
        transfers += 1;
    }
    serde_json::json!({"fullstate_clock_prefixes":6,"newphase0_cases":4,"outside_prior_raw_cases":2,"compiled_phase0_bytes":4,"native_dispatches_coverage_only":dispatches,"phase_correction_coverage_disjoint":[3,4],"all11typed_bytes":true,"new_old_raw_edges":true,"DMA_SWI_checks":transfers})
}

#[test]
#[ignore = "private EUR conditional phase-zero model/managed/native extension; Wine only"]
fn european_phase_zero_threshold_model_managed_native() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    assert_eq!(PHASE_COUNT, 3);
    assert_eq!(model.phase_thresholds.threshold(0), Some(16));
    assert_eq!(
        model
            .phase_thresholds
            .entries
            .iter()
            .map(|e| e.phase)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        model
            .source_spans()
            .unwrap()
            .iter()
            .map(|(_, v, _)| v.len())
            .sum::<usize>(),
        11
    );
    assert_eq!(model.byte_at(PHASE_SOURCE), Some(16));
    for phase in 3..=255 {
        assert_eq!(model.phase_thresholds.threshold(phase), None);
    }
    for old in [0, 15, 16, u16::MAX] {
        super::observed_task_parameters_model_tests::original_phase(&rom, model, 0, old);
    }
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior: Manifest =
        toml::from_str(&fs::read_to_string(old.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            prior.extractor_revision,
            prior.decoder_revision,
            prior.resources.len()
        ),
        (74, 73, 67)
    );
    // Historical two-row JSON is read as evidence, not validated against e75.
    let previous: ObservedTaskParameters =
        serde_json::from_slice(&fs::read(old.join(OBSERVED_TASK_PARAMETERS_FILE)).unwrap())
            .unwrap();
    assert_eq!(
        previous.phase_thresholds.entries,
        model.phase_thresholds.entries[1..]
    );
    assert_eq!(previous.vertical_corrections, model.vertical_corrections);
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (75, 74, 67)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 68);
    for (i, entry) in prior.resources.iter().enumerate() {
        if entry.id != OBSERVED_TASK_PARAMETERS_ID {
            assert_eq!(&source.manifest.resources[i], entry);
            assert_eq!(
                fs::read(dataset().join(&entry.path)).unwrap(),
                fs::read(old.join(&entry.path)).unwrap()
            );
        }
    }
    let entry = &source.manifest.resources[65];
    assert_eq!(entry.id, OBSERVED_TASK_PARAMETERS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(
        (
            entry.sources.len(),
            entry.sources[0].offset,
            entry.sources[0].length,
            entry.sources[1].length
        ),
        (2, PHASE_SOURCE, 3, 8)
    );
    let bytes = fs::read(dataset().join(OBSERVED_TASK_PARAMETERS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(parse_observed_task_parameters(&bytes).unwrap(), *model);
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..7 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=2 => {
                let mut value = parse_observed_task_parameters(&data).unwrap();
                match case {
                    0 => value.phase_thresholds.entries[0].threshold ^= 1,
                    1 => {
                        value.phase_thresholds.entries.remove(0);
                    }
                    _ => value.phase_thresholds.entries[0].phase = 3,
                };
                data = encode_observed_task_parameters(&value).unwrap();
            }
            3 => bad.resources[65].sources[0].offset += 1,
            4 => bad.resources[65].sources[0].length -= 1,
            5 => bad.resources[65].sources[0].sha256 = "00".repeat(32),
            _ => {
                let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
                value["phase_thresholds"]["unrecognized"] = true.into();
                data = serde_json::to_vec(&value).unwrap();
            }
        }
        bad.resources[65].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(OBSERVED_TASK_PARAMETERS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "newphase0 attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(OBSERVED_TASK_PARAMETERS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let native = native_phase_zero(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,75,74],"files":68,"resources":67,"all66_other_records_files_retained":true,"old2phase_and4vertical_rows_exact":true,"new_source_bytes":1,"total_parameter_bytes":11,"spans":[3,8],"model_phase0_original_cases":4,"new_attach_guards":7,"native":native,"parameter_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"Linux":"pending","USA":"pending","nativeWindows":"pending","ordinary_phase0_positive":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_PHASE_ZERO e75-d74/files68/resources67/new1B/total11B guards7 original4 native6 DMA3 PASS");
}
