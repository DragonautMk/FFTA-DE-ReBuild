//! New propertyword publication and runtime checks; closed source oracles stay closed.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_PROPERTY_MANAGED_ROOT").unwrap())
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
fn spans(files: &Dataset) -> Vec<(u32, Vec<u8>, String)> {
    files
        .unit_obj_property_words
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
}
fn sources(m: &mut Machine, files: &Dataset, rom: &[u8], poison: bool) {
    for (at, bytes, _) in spans(files) {
        let at = at as usize;
        m.bus.rom[at..at + bytes.len()].copy_from_slice(&rom[at..at + bytes.len()]);
        if poison {
            m.bus.rom[at..at + bytes.len()]
                .iter_mut()
                .for_each(|b| *b ^= 0x5a);
        }
    }
}
#[test]
#[ignore = "private EUR e67 actor propertyword managed authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_actor_property_words_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..62]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e66").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (66, 65));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (67, 66)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 63);
    assert_eq!(source.manifest.resources.len(), 62);
    for i in 1..62 {
        assert_eq!(files[i], old[i]);
    }
    assert_eq!(&source.manifest.resources[..61], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..61], prior.coverage.as_slice());
    let data = parse_unit_obj_property_words(&files[62]).unwrap();
    let accepted =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_PROPERTY_SOURCE_ROOT").unwrap());
    assert_eq!(
        files[62],
        fs::read(accepted.join(UNIT_OBJ_PROPERTY_WORDS_FILE)).unwrap()
    );
    let resource = &source.manifest.resources[61];
    assert_eq!(
        resource.dependencies,
        vec![METADATA52_ID.to_string(), METADATA14_ID.to_string()]
    );
    assert_eq!(resource.sources.len(), 4);
    assert_eq!(resource.sources.iter().map(|s| s.length).sum::<u32>(), 364);
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.ends_with(UNIT_OBJ_PROPERTY_WORDS_FILE) {
            Err("injected propertyword publication failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e66-d65/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (name, bytes) in names().iter().zip(&old) {
        fs::write(retained.join(name), bytes).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let prepared = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e67-d66"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(UNIT_OBJ_PROPERTY_WORDS_FILE),
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
    for (name, bytes) in names().iter().zip(&old) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(name)).unwrap(),
            *bytes
        );
        assert_eq!(fs::read(prior_dir.join(name)).unwrap(), *bytes);
    }
    let fileset = read_dataset(&dataset()).unwrap();
    let mut dependency_guards = 0;
    for case in 0..3 {
        let mut missing = fileset.files();
        match case {
            0 => missing.unit_obj_property_words = None,
            1 => missing.metadata52 = None,
            _ => missing.metadata14 = None,
        }
        assert!(source.validate_files(missing).is_err());
        dependency_guards += 1;
    }
    let mut inconsistent = Extraction::from_rom(&rom).unwrap();
    inconsistent.metadata14 = None;
    assert_eq!(
        inconsistent.validate_files(fileset.files()),
        Err("Extraction resource set is inconsistent".into())
    );
    dependency_guards += 1;
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let mut guards = 0;
    for case in 0..16 {
        let mut manifest = source.manifest.clone();
        let mut json: serde_json::Value = serde_json::from_slice(&files[62]).unwrap();
        match case {
            0 => json["words"][0]["word"] = (data.words[0].word ^ 0x8000).into(),
            1 => json["producers"][0]["metadata_source"] = (0x6e5464u32 + 7).into(),
            2 => manifest.resources[61].sources[0].sha256 = "0".repeat(64),
            3 => manifest.resources[61].sources[0].offset += 2,
            4 => manifest.resources[61].sources[0].length -= 2,
            5 => manifest.resources[61].sources[0].role = "unknown/property".into(),
            6 => manifest.resources[61].path = "../property.json".into(),
            7 => manifest.resources[61].dependencies.clear(),
            8 => manifest.resources[61].sha256 = "0".repeat(64),
            9 => manifest
                .coverage
                .retain(|id| id != UNIT_OBJ_PROPERTY_WORDS_ID),
            10 => {
                manifest.resources[61].sources.pop();
            }
            11 => json["words"][1] = json["words"][0].clone(),
            12 => json["unknown"] = true.into(),
            13 => manifest.resources[61]
                .dependencies
                .retain(|id| id != METADATA52_ID),
            14 => manifest.resources[61]
                .dependencies
                .retain(|id| id != METADATA14_ID),
            _ => manifest.resources[61].format = "unknown/property".into(),
        }
        let bytes = serde_json::to_vec_pretty(&json).unwrap();
        if case != 8 {
            manifest.resources[61].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(UNIT_OBJ_PROPERTY_WORDS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
        guards += 1;
    }
    fs::write(dataset().join(UNIT_OBJ_PROPERTY_WORDS_FILE), &files[62]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(UNIT_OBJ_PROPERTY_WORDS_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    guards += 1;
    fs::rename(
        r.join("missing.json"),
        dataset().join(UNIT_OBJ_PROPERTY_WORDS_FILE),
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
    sources(&mut m, &fileset, &rom, true);
    for (at, bytes, _) in spans(&fileset) {
        for (n, &byte) in bytes.iter().enumerate() {
            assert_eq!(m.bus.read8(0x08000000 + at + n as u32), byte);
        }
    }
    let stats = owner.unit_obj_property_words_stats().unwrap();
    assert_eq!((stats.source_bytes, stats.words_used), (364, 182));
    let old_before = owner
        .unit_obj_alternate_battle_stats()
        .unwrap()
        .source_bytes;
    assert_eq!(
        m.bus.read32(0x08394c10),
        u32::from_le_bytes(rom[0x394c10..0x394c14].try_into().unwrap())
    );
    assert_eq!(
        owner
            .unit_obj_alternate_battle_stats()
            .unwrap()
            .source_bytes
            - old_before,
        4
    );
    // Both previously classified FFFF reader fields remain mutable raw ROM.
    for (at, width) in [(0x3d4830, 4), (0x3b4c12, 2)] {
        m.bus.rom[at..at + width]
            .iter_mut()
            .for_each(|v| *v ^= 0x33);
        for n in 0..width {
            assert_eq!(
                m.bus.read8(0x08000000 + (at + n) as u32),
                rom[at + n] ^ 0x33
            );
        }
        m.bus.rom[at..at + width].copy_from_slice(&rom[at..at + width]);
    }
    assert_eq!(
        owner.unit_obj_property_words_stats().unwrap().source_bytes,
        stats.source_bytes
    );
    report(
        "managed-result.json",
        serde_json::json!({
            "revision":[1,67,66],"files":63,"resources":62,"source_ranges":4,"source_bytes":364,
            "all61prior_records_files_and_e66_retained":true,"guards":guards,"dependency_guards":dependency_guards,
            "actual_metadata52_metadata14_dependencies":true,"failed_attach_owner_state_transactional":true,
            "publication_failure_create_reuse_repair_move":true,"unavailable_load_undo_backup_restart_sharedowner":true,
            "words_used":182,"old_alternate_directory_bytes":4,"FFF_six_raw_bytes_live":true,
            "linux":"pending","usa":"pending","performance_measurements":false
        }),
    );
}

#[derive(Clone, Debug)]
enum Probe {
    Getter(u32),
    Actor(u16),
    Read(u32, usize),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Getter(_) => (0x080221c4, 0x08000300),
            Self::Actor(_) => (0x0809acf8, 0x0809ad18),
            Self::Read(_, 1) => (0x080221de, 0x080221e0),
            Self::Read(_, 2) => (0x080221ca, 0x080221cc),
            Self::Read(_, 4) => (0x0802217e, 0x08022180),
            _ => unreachable!(),
        }
    }
    fn setup(&self, m: &mut Machine) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0);
        m.bus.vram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 4..13 {
            m.cpu.regs[n] = 0x41414000 + n as u32 * 0x101;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.write16(0x02018000, 0xfeda);
        m.bus.write32(0x03000100, 0x88776655);
        match *self {
            Self::Getter(id) => m.cpu.regs[0] = id,
            Self::Actor(id) => {
                m.bus.write16(0x02010034, id);
                m.cpu.regs[..4].copy_from_slice(&[0x02010000, 8, 1, 0]);
            }
            Self::Read(at, 1) => m.cpu.regs[4] = at,
            Self::Read(at, 2 | 4) => m.cpu.regs[0] = at,
            _ => unreachable!(),
        }
    }
}
fn original(m: &mut Machine, probe: &Probe, pcs: &mut BTreeSet<u32>) -> usize {
    probe.setup(m);
    let mut n = 0;
    while m.cpu.regs[15] != probe.bounds().1 {
        assert!(n < 2000, "{probe:?} PC{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
#[test]
#[ignore = "private EUR e67 actor propertyword runtime/native/state-clock; Wine only, Linux/USA pending"]
fn european_actor_property_words_native_reads_and_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.unit_obj_property_words.as_ref().unwrap();
    let source_spans = spans(&files);
    assert_eq!(source_spans.len(), 4);
    let first = data.words.first().unwrap().sprite_id;
    let last = data.words.last().unwrap().sprite_id;
    let other = data.words.iter().find(|w| w.word == 24).unwrap().sprite_id;
    let ids = BTreeSet::from([first, last, other]);
    let mut cases: Vec<_> = ids
        .into_iter()
        .map(|id| Probe::Getter(u32::from(id)))
        .collect();
    cases.extend([
        Probe::Getter(0x80000000 | u32::from(last)),
        Probe::Getter(0x10000 + u32::from(first)),
        Probe::Getter(112),
        Probe::Getter(117),
        Probe::Actor(first),
        Probe::Actor(other),
        Probe::Read(0x08394c14, 1),
        Probe::Read(0x08394e02, 2),
        Probe::Read(0x08394cf4, 4),
        Probe::Read(0x0a394c14, 4),
        Probe::Read(0x08394c10, 4),
        Probe::Read(0x02018000, 2),
        Probe::Read(0x03000100, 4),
    ]);
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for probe in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, probe, &mut pcs);
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
    let object = out.join("unit-obj-property-words.o");
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
        guard: prefix_guard,
        ..RT_API
    };
    COMPILED.with(|v| v.set(0));
    let mut dispatches = 0;
    for probe in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
        }
        original(&mut legacy, probe, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, true);
        }
        let before = owner.unit_obj_property_words_stats().unwrap().source_bytes;
        original(&mut decoded, probe, &mut BTreeSet::new());
        native.bus.frame_ready = false;
        probe.setup(&mut native);
        STOP_PC.with(|v| v.set(probe.bounds().1));
        let mut n = 0;
        while native.cpu.regs[15] != probe.bounds().1 {
            assert!(n < 2000, "native{probe:?} PC{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .unwrap_or_else(|| panic!("missing native{probe:?} PC{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        if matches!(probe, Probe::Actor(_)) {
            assert_eq!(
                owner.unit_obj_property_words_stats().unwrap().source_bytes - before,
                2
            );
        }
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, false);
        }
        let expected = legacy.save_state().unwrap();
        for m in [&mut decoded, &mut native] {
            assert!(
                m.save_state().unwrap() == expected,
                "complete state/clock{probe:?}"
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(compiled > 0);
    decoded.load_state(&start).unwrap();
    sources(&mut decoded, &files, &rom, true);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in &source_spans {
            for boundary in [*at, *at + bytes.len() as u32] {
                for off in boundary - 3..boundary + 4 {
                    for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                        assert_eq!(decoded.bus.read8(mirror + off), rom[off as usize]);
                        assert_eq!(
                            decoded.bus.read16(mirror + off),
                            u16::from_le_bytes(
                                rom[off as usize..off as usize + 2].try_into().unwrap()
                            )
                        );
                        assert_eq!(
                            (RT_API.read32)(
                                &mut decoded as *mut Machine as *mut std::ffi::c_void,
                                mirror + off
                            ),
                            u32::from_le_bytes(
                                rom[off as usize..off as usize + 4].try_into().unwrap()
                            )
                        );
                    }
                }
            }
        }
    }
    decoded.bus.real_bios = false;
    let before = owner.unit_obj_property_words_stats().unwrap().source_bytes;
    let old = owner
        .unit_obj_alternate_battle_stats()
        .unwrap()
        .source_bytes;
    assert_eq!(
        (RT_API.read32)(
            &mut decoded as *mut Machine as *mut std::ffi::c_void,
            0x08394c12
        ),
        u32::from_le_bytes(rom[0x394c12..0x394c16].try_into().unwrap())
    );
    assert_eq!(
        (
            owner.unit_obj_property_words_stats().unwrap().source_bytes - before,
            owner
                .unit_obj_alternate_battle_stats()
                .unwrap()
                .source_bytes
                - old
        ),
        (2, 2)
    );
    let raw = 0x394cf4;
    decoded.bus.rom[raw..raw + 2]
        .iter_mut()
        .for_each(|v| *v ^= 0x33);
    let mut expected: [u8; 4] = rom[raw..raw + 4].try_into().unwrap();
    expected[..2].iter_mut().for_each(|v| *v ^= 0x33);
    let before = owner.unit_obj_property_words_stats().unwrap().source_bytes;
    assert_eq!(
        (RT_API.read32)(
            &mut decoded as *mut Machine as *mut std::ffi::c_void,
            0x08000000 + raw as u32
        ),
        u32::from_le_bytes(expected)
    );
    assert_eq!(
        owner.unit_obj_property_words_stats().unwrap().source_bytes - before,
        2
    );
    decoded.bus.rom[raw..raw + 2].copy_from_slice(&rom[raw..raw + 2]);
    sources(&mut decoded, &files, &rom, false);
    let mut swi_counts = Vec::new();
    for (swi, src, count, expected_delta) in [
        (0xdf0b, 0x394c12, 3, (4, 2)),
        (0xdf0c, 0x394c10, 8, (28, 4)),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        sources(&mut decoded, &files, &rom, true);
        let before = owner.unit_obj_property_words_stats().unwrap().source_bytes;
        let old = owner
            .unit_obj_alternate_battle_stats()
            .unwrap()
            .source_bytes;
        legacy.step();
        decoded.step();
        let delta = (
            owner.unit_obj_property_words_stats().unwrap().source_bytes - before,
            owner
                .unit_obj_alternate_battle_stats()
                .unwrap()
                .source_bytes
                - old,
        );
        assert_eq!(delta, expected_delta);
        swi_counts.push(delta);
        sources(&mut decoded, &files, &rom, false);
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x0a394c12);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 3);
    }
    sources(&mut decoded, &files, &rom, true);
    let before = owner.unit_obj_property_words_stats().unwrap().source_bytes;
    let old = owner
        .unit_obj_alternate_battle_stats()
        .unwrap()
        .source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let dma_counts = (
        owner.unit_obj_property_words_stats().unwrap().source_bytes - before,
        owner
            .unit_obj_alternate_battle_stats()
            .unwrap()
            .source_bytes
            - old,
    );
    assert_eq!(dma_counts, (4, 2));
    sources(&mut decoded, &files, &rom, false);
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({
            "revision":[1,67,66],"original_interpreter_compiled_paths":cases.len(),
            "compiled_propertyword_bytes":compiled,"full_states_clocks_equal":true,
            "all364raw_source_bytes_altered":true,"all4ranges_widths_mirrors_realBIOS_bus":true,
            "fullword_getter_and_raw_holes_dynamic_RAM":true,"actor_prefixes":2,
            "mixed_old_new_API_bytes":[2,2],"mixed_raw_new_API_bytes":[2,2],
            "mixed_new_old_SWI_bytes":swi_counts,"mixed_new_old_DMA_bytes":dma_counts,
            "instructions_coverage_only":instructions,"PCs":pcs.len(),"native_dispatches_coverage_only":dispatches,
            "scope":"new typed propertyword sources/partial actor prefixes; no full allocator or ordinary claim",
            "linux":"pending","usa":"pending","performance_measurements":false
        }),
    );
}

thread_local! {
    static COMPILED:Cell<u64>=const{Cell::new(0)};
    static STOP_PC:Cell<u32>=const{Cell::new(0)};
}
// The production emitter chains neighboring blocks. A prefix endpoint can
// also be an interior instruction in another probe's compiled graph. Keep the
// real event guard, and yield at the declared checkpoint before executing it.
extern "C" fn prefix_guard(m: *mut std::ffi::c_void, a: u32, len: u32, expect: u64) -> u32 {
    if len == 0 && expect == gba_core::capi::EVENT_GUARD_TAG {
        let pc = unsafe { (&*(m as *const Machine)).cpu.regs[15] };
        if STOP_PC.with(Cell::get) == pc {
            return gba_core::capi::EVENT_YIELD;
        }
        let budget = (RT_API.guard)(m, a, len, expect);
        if budget & gba_core::capi::EVENT_YIELD != 0 {
            return budget;
        }
        let machine = unsafe { &*(m as *const Machine) };
        let off = (pc & 0x01ffffff) as usize;
        let bytes = if pc >> 24 == 3 {
            &machine.bus.iwram[(pc & 0x7fff) as usize..][..2]
        } else {
            &machine.bus.rom[off..off + 2]
        };
        let instr = armv4t::decode_thumb(u16::from_le_bytes(bytes.try_into().unwrap()), pc);
        let cost = gba_core::machine_instr_cost((pc >> 24) as usize & 15, &instr) as u32;
        budget.min(cost.saturating_add(1))
    } else {
        (RT_API.guard)(m, a, len, expect)
    }
}
fn usage(m: *mut std::ffi::c_void) -> u64 {
    unsafe { (&*(m as *const Machine)).bus.rom_resources().unwrap() }
        .unit_obj_property_words_stats()
        .unwrap()
        .source_bytes
}
fn count(before: u64, after: u64) {
    COMPILED.with(|v| v.set(v.get() + after - before));
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

fn names() -> [&'static str; 63] {
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
        PALETTE_PROGRAMS_FILE,
        BLEND_PROGRAMS_FILE,
        TASK_DESCRIPTORS_FILE,
        EFFECT_EVENT_DIRECTORIES_FILE,
        SCRIPT_COMMANDS_FILE,
        SCRIPT_ARCHIVE_FILE,
        SCRIPT_TEXT_FILE,
        SCRIPT_COORDINATES_FILE,
        AUXILIARY_COMMANDS_FILE,
        AUXILIARY_ARCHIVE_FILE,
        MAP_RASTER_PALETTES_FILE,
        UI_CHOICE_LISTS_FILE,
        UNIT_OBJ_INITIAL_POSE_FILE,
        COLOR_MODE_PALETTES_FILE,
        UNIT_OBJ_EQUIPMENT_FILE,
        UNIT_OBJ_NORMAL_BATTLE_FILE,
        UNIT_OBJ_ALTERNATE_BATTLE_FILE,
        UNIT_OBJ_SCRIPT_SPECIAL_NORMAL_FILE,
        UNIT_OBJ_SCRIPT_SPECIAL_ALTERNATE_FILE,
        UNIT_OBJ_SCRIPT_OVERRIDE_FILE,
        UNIT_OBJ_SCRIPT_REGISTRY_SPECIAL_FILE,
        UNIT_OBJ_SCRIPT_REGISTRY_GENERIC_FILE,
        UNIT_OBJ_PROPERTY_WORDS_FILE,
    ]
}
