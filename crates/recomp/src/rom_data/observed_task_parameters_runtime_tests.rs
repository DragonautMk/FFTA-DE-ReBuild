//! Two targeted Wine filters for the e73 task-parameter owner.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TASK_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn names(manifest: &Manifest) -> Vec<String> {
    std::iter::once(MANIFEST_FILE.to_string())
        .chain(manifest.resources.iter().map(|r| r.path.clone()))
        .collect()
}
fn ranges(m: &mut Machine, sources: &[gba_core::rom_data::SourceRange], rom: Option<&[u8]>) {
    for s in sources {
        let a = s.offset as usize;
        let z = a + s.length as usize;
        if let Some(rom) = rom {
            m.bus.rom[a..z].copy_from_slice(&rom[a..z]);
        } else {
            m.bus.rom[a..z].fill(0xcd);
        }
    }
}
#[test]
#[ignore = "private EUR e73 managed task parameters; Wine only, Linux/USA pending"]
fn european_observed_task_parameters_managed_publication_and_lifecycle() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior: Manifest = parse(&fs::read(old.join(MANIFEST_FILE)).unwrap(), "e72").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (72, 71));
    assert_eq!(prior.resources.len(), 65);
    let old_names = names(&prior);
    let old_files: Vec<_> = old_names
        .iter()
        .map(|n| fs::read(old.join(n)).unwrap())
        .collect();
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (73, 72)
    );
    assert_eq!(source.manifest.resources.len(), 66);
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 67);
    assert_eq!(&source.manifest.resources[..65], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..65], prior.coverage.as_slice());
    for (n, bytes) in old_names.iter().zip(&old_files).skip(1) {
        assert_eq!(fs::read(dataset().join(n)).unwrap(), *bytes);
    }
    let entry = &source.manifest.resources[65];
    assert_eq!(entry.id, OBSERVED_TASK_PARAMETERS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 2);
    assert_eq!(entry.sources.iter().map(|s| s.length).sum::<u32>(), 10);
    let bytes = fs::read(dataset().join(OBSERVED_TASK_PARAMETERS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        bytes,
        fs::read(std::env::var_os("FFTA_STATE_TEST_TASK_MODEL").unwrap()).unwrap()
    );
    assert_eq!(
        encode_observed_task_parameters(&parse_observed_task_parameters(&bytes).unwrap()).unwrap(),
        bytes
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut writes = 0;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        writes += 1;
        if writes == 13 {
            Err("injected publication failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e72-d71/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (n, b) in old_names.iter().zip(&old_files) {
        fs::write(retained.join(n), b).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e73-d72"));
    let pointer = first.directory.parent().unwrap().join("current");
    let pointer_bytes = fs::read(&pointer).unwrap();
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(fs::read(&pointer).unwrap(), pointer_bytes);
    fs::write(
        first.directory.join(OBSERVED_TASK_PARAMETERS_FILE),
        b"partial",
    )
    .unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    fs::remove_file(repaired.directory.join(OBSERVED_TASK_PARAMETERS_FILE)).unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    let mut m = crate::make_machine(rom, None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let state = m.save_state().unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    for case in 0..13 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0 | 1 | 10 | 11 => {
                let mut v = parse_observed_task_parameters(&data).unwrap();
                match case {
                    0 => v.phase_thresholds.entries[0].threshold ^= 1,
                    1 => v.vertical_corrections.entries[0].correction ^= 1,
                    10 => {
                        v.phase_thresholds.entries.pop();
                    }
                    _ => {
                        v.vertical_corrections.entries.pop();
                    }
                }
                data = encode_observed_task_parameters(&v).unwrap();
                bad.resources[65].sha256 = gba_core::ffta_rom::digest(&data);
            }
            2 | 3 => {
                bad.resources[65].sources.remove(case - 2);
            }
            4 => bad.resources[65].sources[0].offset += 1,
            5 => bad.resources[65].sources[1].length -= 2,
            6 => bad.resources[65].sources[1].sha256 = "00".repeat(32),
            7 => bad.resources[65]
                .dependencies
                .push(gba_core::rom_data::OBJ_DIMENSIONS_ID.into()),
            8 => bad.decoder_revision -= 1,
            9 => {
                data.pop();
                bad.resources[65].sha256 = gba_core::ffta_rom::digest(&data);
            }
            _ => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unknown"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
                bad.resources[65].sha256 = gba_core::ffta_rom::digest(&data);
            }
        }
        fs::write(dataset().join(OBSERVED_TASK_PARAMETERS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(OBSERVED_TASK_PARAMETERS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.observed_task_parameters.take();
    assert!(source.validate_files(files.files()).is_err());
    source.observed_task_parameters = saved;
    let mut missing = files.files();
    missing.observed_task_parameters = None;
    assert!(source.validate_files(missing).is_err());
    let saved = source.slot_job_lookup.take();
    assert!(source.validate_files(files.files()).is_err());
    source.slot_job_lookup = saved;
    let relative = repaired
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = root.join("moved resources");
    fs::rename(&managed, &moved).unwrap();
    let relocated = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(relocated.action, "reused");
    assert_eq!(relocated.directory, moved.join(relative));
    for (n, b) in old_names.iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(n)).unwrap(),
            *b
        );
        assert_eq!(fs::read(old.join(n)).unwrap(), *b);
    }
    owner_lifecycle();
    report(
        "managed-result.json",
        serde_json::json!({"files":67,"resources":66,"revision":[1,73,72],"old65_files_records_retained":true,"canonical_model_exact":true,"source_bytes":10,"spans":2,"attach_guards":13,"full_set_guards":3,"interruption_create_reuse_repair_move":true,"failed_attach_owner_fullstate_exact":true,"linux":"pending","USA":"pending","performance_measurements":false}),
    );
}
fn owner_lifecycle() {
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("dataset");
    fs::create_dir(&copy).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    for name in names(&files.manifest) {
        fs::copy(dataset().join(&name), copy.join(name)).unwrap();
    }
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &copy).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    fs::rename(&copy, temp.path().join("unavailable")).unwrap();
    assert!(attach_from_directory(&mut m, &copy).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    let save = temp.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &m.bus.rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 99;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    assert_eq!(m.cpu.regs[4], 0);
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
    assert_eq!(fs::read(&save).unwrap(), vec![9; len]);
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    ranges(&mut m, &files.manifest.resources[65].sources, None);
    for s in &files.manifest.resources[65].sources {
        for at in s.offset..s.offset + s.length {
            assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
        }
    }
    let stats = owner.observed_task_parameters_stats().unwrap();
    assert_eq!((stats.phases_used, stats.corrections_used), (2, 4));
    report(
        "lifecycle-result.json",
        serde_json::json!({"directory_unavailable":true,"activation_transactional":true,"load_undo_backup_restart_owner_retained":true,"all10_bytes_6_rows_positive":true}),
    );
}
#[derive(Clone, Copy)]
pub(super) enum Call {
    Phase(u8, u16),
    Vertical(u8),
}
pub(super) fn setup(m: &mut Machine, call: Call) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    match call {
        Call::Phase(phase, old) => {
            m.cpu.regs[4] = 0x02010000;
            m.cpu.regs[15] = 0x08018dca;
            m.bus.write16(0x0201000c, old);
            m.bus.write8(0x02010012, phase);
        }
        Call::Vertical(byte) => {
            m.cpu.regs[2] = if byte & 1 == 0 { 10 } else { 0x80000010 };
            m.cpu.regs[6] = 0x02010000;
            m.cpu.regs[15] = 0x080264ea;
            m.bus.write8(0x02010007, byte);
        }
    }
}
pub(super) fn stop(call: Call) -> u32 {
    match call {
        Call::Phase(..) => 0x08018e40,
        Call::Vertical(..) => 0x080264fa,
    }
}
pub(super) fn interpreted(m: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    setup(m, call);
    for steps in 0..=40 {
        if m.cpu.regs[15] == stop(call) {
            return steps;
        }
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
    }
    panic!("original prefix failed to reach bound")
}
thread_local! {static NATIVE_BYTES:Cell<u64>=const{Cell::new(0)};}
fn used(m: *mut std::ffi::c_void) -> u64 {
    unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .observed_task_parameters_stats()
            .unwrap()
            .source_bytes
    }
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let n = used(m);
    let v = (RT_API.read8)(m, a);
    NATIVE_BYTES.with(|x| x.set(x.get() + used(m) - n));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let n = used(m);
    let v = (RT_API.read16)(m, a);
    NATIVE_BYTES.with(|x| x.set(x.get() + used(m) - n));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let n = used(m);
    let v = (RT_API.read32)(m, a);
    NATIVE_BYTES.with(|x| x.set(x.get() + used(m) - n));
    v
}
#[test]
#[ignore = "private EUR e73 typed task parameter CPU/native/bus; Wine only, Linux/USA pending"]
fn european_observed_task_parameters_native_and_bus_consumption() {
    let output = root().join("native");
    assert!(!output.exists());
    fs::create_dir(&output).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let sources = &files.manifest.resources[65].sources;
    let mut calls = Vec::new();
    for phase in [1, 2] {
        for old in [0, 7, 8, 9, u16::MAX] {
            calls.push(Call::Phase(phase, old));
        }
    }
    for (phase, old) in [(0, 16), (3, 0), (255, u16::MAX)] {
        calls.push(Call::Phase(phase, old));
    }
    for byte in 8..16 {
        calls.push(Call::Vertical(byte));
    }
    for byte in [0, 16, 255] {
        calls.push(Call::Vertical(byte));
    }
    assert_eq!(calls.len(), 24);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &call in &calls {
        tracer.load_state(&start).unwrap();
        instructions += interpreted(&mut tracer, call, &mut visited);
    }
    let view = View {
        rom: &rom,
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
    let object = output.join("parameters.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut old = crate::make_machine(rom.clone(), None);
    let mut typed = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    NATIVE_BYTES.with(|x| x.set(0));
    let mut dispatches = 0;
    for &call in &calls {
        for m in [&mut old, &mut typed, &mut native] {
            m.load_state(&start).unwrap();
        }
        interpreted(&mut old, call, &mut BTreeSet::new());
        for m in [&mut typed, &mut native] {
            ranges(m, sources, None);
        }
        interpreted(&mut typed, call, &mut BTreeSet::new());
        setup(&mut native, call);
        while native.cpu.regs[15] != stop(call) {
            assert!(dispatches < 100000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatches += 1;
        }
        for m in [&mut typed, &mut native] {
            ranges(m, sources, Some(&rom));
            assert_eq!(
                m.save_state().unwrap(),
                old.save_state().unwrap(),
                "complete state and clock"
            );
        }
    }
    let compiled = NATIVE_BYTES.with(Cell::get);
    assert!(compiled > 0);
    let coverage = native
        .bus
        .rom_resources()
        .unwrap()
        .observed_task_parameters_stats()
        .unwrap();
    assert_eq!((coverage.phases_used, coverage.corrections_used), (2, 4));
    ranges(&mut typed, sources, None);
    for bios in [false, true] {
        typed.bus.real_bios = bios;
        for s in sources {
            for at in s.offset - 3..s.offset + s.length + 3 {
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
                }
            }
        }
    }
    typed.bus.real_bios = false;
    ranges(&mut typed, sources, Some(&rom));
    let mut transfers = 0;
    for (i, source) in sources.iter().enumerate() {
        for kind in 0..3 {
            let base = if i == 0 { 0x371064 } else { 0x394e10 };
            let length = if kind == 2 {
                32
            } else {
                if i == 0 {
                    4
                } else {
                    12
                }
            };
            for m in [&mut old, &mut typed] {
                m.load_state(&start).unwrap();
                if kind == 0 {
                    m.bus.write32(0x040000d4, 0x08000000 + base);
                    m.bus.write32(0x040000d8, 0x02032000);
                    m.bus.write16(0x040000dc, (length / 2) as u16);
                } else {
                    m.cpu.regs[..3].copy_from_slice(&[
                        0x0a000000 + base,
                        0x02032400,
                        if kind == 2 { 8 } else { length / 2 },
                    ]);
                    m.cpu.regs[15] = 0x02000000;
                    m.cpu.cpsr = 0xbf;
                    m.bus
                        .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
                }
            }
            let before = typed
                .bus
                .rom_resources()
                .unwrap()
                .observed_task_parameters_stats()
                .unwrap()
                .source_bytes;
            ranges(&mut typed, sources, None);
            for m in [&mut old, &mut typed] {
                if kind == 0 {
                    m.bus.write16(0x040000de, 0x8000);
                } else {
                    m.step();
                }
            }
            assert_eq!(
                typed
                    .bus
                    .rom_resources()
                    .unwrap()
                    .observed_task_parameters_stats()
                    .unwrap()
                    .source_bytes
                    - before,
                u64::from(source.length)
            );
            ranges(&mut typed, sources, Some(&rom));
            assert_eq!(typed.save_state().unwrap(), old.save_state().unwrap());
            transfers += 1;
        }
    }
    let mut mixed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut mixed, &dataset()).unwrap();
    ranges(&mut mixed, sources, None);
    let owner = mixed.bus.rom_resources().unwrap().clone();
    let n = owner.observed_task_parameters_stats().unwrap().source_bytes;
    let p = owner
        .unit_obj_byte_property_words_stats()
        .unwrap()
        .source_bytes;
    mixed.bus.rom[0x394e12] ^= 0x55;
    mixed.bus.rom[0x394e13] ^= 0x33;
    assert_eq!(
        (RT_API.read32)(
            &mut mixed as *mut Machine as *mut std::ffi::c_void,
            0x08394e12
        ),
        u32::from_le_bytes(rom[0x394e12..0x394e16].try_into().unwrap())
    );
    assert_eq!(
        owner.observed_task_parameters_stats().unwrap().source_bytes - n,
        2
    );
    assert_eq!(
        owner
            .unit_obj_byte_property_words_stats()
            .unwrap()
            .source_bytes
            - p,
        2
    );
    for at in [0x371064u32, 0x371067] {
        assert!(files
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .all(|s| at < s.offset || at >= s.offset + s.length));
    }
    mixed.bus.rom[0x371064] ^= 0x55;
    mixed.bus.rom[0x371067] ^= 0x33;
    let mut expected: [u8; 4] = rom[0x371064..0x371068].try_into().unwrap();
    expected[0] ^= 0x55;
    expected[3] ^= 0x33;
    let n = owner.observed_task_parameters_stats().unwrap().source_bytes;
    assert_eq!(
        (RT_API.read32)(
            &mut mixed as *mut Machine as *mut std::ffi::c_void,
            0x08371064
        ),
        u32::from_le_bytes(expected)
    );
    assert_eq!(
        owner.observed_task_parameters_stats().unwrap().source_bytes - n,
        2
    );
    report(
        "native-result.json",
        serde_json::json!({"calls":calls.len(),"original_instructions_coverage_only":instructions,"native_dispatches_coverage_only":dispatches,"compiled_source_bytes":compiled,"all_states_clocks_equal":true,"all10_raw_bytes_poisoned":true,"phase_rows":coverage.phases_used,"correction_rows":coverage.corrections_used,"outside_inputs_previous_owner_and_raw_preserved":true,"both_edges_widths_mirrors_realBIOS_bus":true,"DMA_CpuSet_FastSet_checks":transfers,"mixed2new2old_edited_old_raw":true,"mixed2new2editedraw":true,"linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false}),
    );
}
