//! FFFF numeric-field publication and runtime checks; closed source oracles stay closed.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_FFFF_FIELDS_MANAGED_ROOT").unwrap())
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
        .unit_obj_ffff_reader_fields
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
#[ignore = "private EUR e70 FFFF numeric-field managed authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_ffff_reader_fields_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..64]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e69").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (69, 68));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (70, 69)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 65);
    assert_eq!(source.manifest.resources.len(), 64);
    for i in 1..64 {
        assert_eq!(files[i], old[i]);
    }
    assert_eq!(&source.manifest.resources[..63], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..63], prior.coverage.as_slice());
    let data = parse_unit_obj_ffff_reader_fields(&files[64]).unwrap();
    let accepted =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_FFFF_FIELDS_SOURCE_ROOT").unwrap());
    assert_eq!(files[64], fs::read(accepted.join("decoded.json")).unwrap());
    let resource = &source.manifest.resources[63];
    assert_eq!(
        resource.dependencies,
        vec![METADATA52_ID.to_string(), SCRIPT_ARCHIVE_ID.to_string()]
    );
    assert_eq!(resource.sources.len(), 2);
    assert_eq!(resource.sources.iter().map(|s| s.length).sum::<u32>(), 6);
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.ends_with(UNIT_OBJ_FFFF_READER_FIELDS_FILE) {
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
        .join("v1/e69-d68/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e70-d69"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(UNIT_OBJ_FFFF_READER_FIELDS_FILE),
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
            0 => missing.unit_obj_ffff_reader_fields = None,
            1 => missing.metadata52 = None,
            _ => missing.script_archive = None,
        }
        assert!(source.validate_files(missing).is_err());
        dependency_guards += 1;
    }
    let mut inconsistent = Extraction::from_rom(&rom).unwrap();
    inconsistent.script_archive = None;
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
    for case in 0..19 {
        let mut manifest = source.manifest.clone();
        let mut json: serde_json::Value = serde_json::from_slice(&files[64]).unwrap();
        match case {
            0 => json["property_word"]["value"] = (data.property_word.value ^ 0x8000).into(),
            1 => json["directory_word"]["value"] = (data.directory_word.value ^ 0x80000000).into(),
            2 => manifest.resources[63].sources[0].sha256 = "0".repeat(64),
            3 => manifest.resources[63].sources[1].offset += 4,
            4 => manifest.resources[63].sources[0].length -= 1,
            5 => manifest.resources[63].sources[1].role = "unknown/field".into(),
            6 => manifest.resources[63].path = "../fields.json".into(),
            7 => manifest.resources[63].dependencies.clear(),
            8 => manifest.resources[63].sha256 = "0".repeat(64),
            9 => manifest
                .coverage
                .retain(|id| id != UNIT_OBJ_FFFF_READER_FIELDS_ID),
            10 => {
                manifest.resources[63].sources.pop();
            }
            11 => json["producers"][0]["script_sources"][0] = 0.into(),
            12 => json["unknown"] = true.into(),
            13 => manifest.resources[63]
                .dependencies
                .retain(|id| id != METADATA52_ID),
            14 => manifest.resources[63]
                .dependencies
                .retain(|id| id != SCRIPT_ARCHIVE_ID),
            15 => manifest.resources[63].format = "unknown/fields".into(),
            16 => manifest.decoder_revision -= 1,
            17 => {
                json["directory_word"]["source_offset"] =
                    (data.directory_word.source_offset + 4).into()
            }
            _ => {}
        }
        // Even a coherent changed JSON + source hash must fail canonical authentication.
        if case <= 1 {
            let bytes = if case == 0 {
                (data.property_word.value ^ 0x8000).to_le_bytes().to_vec()
            } else {
                (data.directory_word.value ^ 0x80000000)
                    .to_le_bytes()
                    .to_vec()
            };
            manifest.resources[63].sources[case].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        let mut bytes = serde_json::to_vec_pretty(&json).unwrap();
        if case == 18 {
            bytes.truncate(bytes.len() / 2);
        }
        if case != 8 {
            manifest.resources[63].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(UNIT_OBJ_FFFF_READER_FIELDS_FILE), bytes).unwrap();
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
    fs::write(dataset().join(UNIT_OBJ_FFFF_READER_FIELDS_FILE), &files[64]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(UNIT_OBJ_FFFF_READER_FIELDS_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    guards += 1;
    fs::rename(
        r.join("missing.json"),
        dataset().join(UNIT_OBJ_FFFF_READER_FIELDS_FILE),
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
    let stats = owner.unit_obj_ffff_reader_fields_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.property_bytes,
            stats.directory_bytes,
            stats.fields_used
        ),
        (6, 2, 4, 2)
    );
    let previous = owner.unit_obj_property_words_stats().unwrap().source_bytes;
    assert_eq!(
        m.bus.read16(0x08394c14),
        u16::from_le_bytes(rom[0x394c14..0x394c16].try_into().unwrap())
    );
    assert_eq!(
        owner.unit_obj_property_words_stats().unwrap().source_bytes - previous,
        2
    );
    assert_eq!(owner.unit_obj_ffff_reader_fields_stats().unwrap(), stats);
    report(
        "managed-result.json",
        serde_json::json!({
            "revision":[1,70,69],"files":65,"resources":64,"source_ranges":2,"source_bytes":6,
            "all63prior_records_files_and_e69_retained":true,"guards":guards,"dependency_guards":dependency_guards,
            "metadata52_script_archive_dependencies":true,"failed_attach_owner_state_transactional":true,
            "publication_failure_create_reuse_repair_move":true,"unavailable_load_undo_backup_restart_sharedowner":true,
            "fields_used":2,"prior_property_bytes":2,"shared_owners_not_duplicated":true,
            "linux":"pending","usa":"pending","performance_measurements":false
        }),
    );
}

#[derive(Clone, Debug)]
enum Probe {
    Getter,
    Directory(u32),
    Selected(u32, u32, u32, bool),
    Read(u32, usize),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Getter => (0x080221c4, 0x08000300),
            Self::Directory(_) => (0x08022174, 0x08000300),
            Self::Selected(_, _, _, true) => (0x08022788, 0x08000300),
            Self::Selected(_, _, _, false) => (0x08022788, 0x080227aa),
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
        m.bus.bios_open = 0;
        m.bus.write16(0x02018000, 0xfeda);
        m.bus.write32(0x03000100, 0x88776655);
        match *self {
            Self::Getter => m.cpu.regs[0] = 65535,
            Self::Directory(animation) => {
                m.cpu.regs[0] = 65535;
                m.cpu.regs[1] = animation;
            }
            Self::Selected(animation, part, bios, _) => {
                m.cpu.regs[0] = 0x02010000;
                m.bus.write32(0x03007e00, 65535);
                m.bus.write32(0x03007e04, animation);
                m.bus.write32(0x03007e08, part);
                m.bus.bios_open = bios;
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
    match *probe {
        Probe::Getter => assert_eq!(m.cpu.regs[0], 0xdc86),
        Probe::Selected(_, _, 0, true) => assert_eq!(m.cpu.regs[0], u32::MAX),
        Probe::Selected(_, _, bios, false) => assert_eq!(m.cpu.regs[0], bios.rotate_right(16)),
        _ => {}
    }
    n
}
#[test]
#[ignore = "private EUR e70 FFFF fields native/state-clock/bus; Wine only, Linux/USA pending"]
fn european_ffff_reader_fields_native_reads_and_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let source_spans = spans(&files);
    assert_eq!(source_spans.len(), 2);
    let cases = vec![
        Probe::Getter,
        Probe::Directory(8),
        Probe::Directory(9),
        Probe::Selected(8, 0, 0, true),
        Probe::Selected(8, 0, 0x12345678, false),
        Probe::Selected(9, 2, 0x80000001, false),
        Probe::Selected(142, 1, 0x12345678, false),
        Probe::Read(0x083b4c12, 1),
        Probe::Read(0x0a3b4c12, 2),
        Probe::Read(0x0c3d4830, 4),
        Probe::Read(0x083b4c12, 4),
        Probe::Read(0x083d482e, 4),
        Probe::Read(0x02018000, 2),
        Probe::Read(0x03000100, 4),
    ];
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
    let object = out.join("unit-obj-ffff-reader-fields.o");
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
                .unwrap_or_else(|| panic!("missing{probe:?} PC{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, false);
        }
        let expected = legacy.save_state().unwrap();
        for m in [&mut decoded, &mut native] {
            assert!(
                m.save_state().unwrap() == expected,
                "full state/clock{probe:?}"
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
    let mut mixed = Vec::new();
    for (at, raw, owned) in [
        (0x3b4c12, 0x3b4c14, 2),
        (0x3d482e, 0x3d482e, 2),
        (0x3d4832, 0x3d4834, 2),
    ] {
        decoded.bus.rom[raw..raw + 2]
            .iter_mut()
            .for_each(|v| *v ^= 0x33);
        let mut expect: [u8; 4] = rom[at..at + 4].try_into().unwrap();
        for i in 0..2 {
            expect[raw - at + i] ^= 0x33;
        }
        let before = owner
            .unit_obj_ffff_reader_fields_stats()
            .unwrap()
            .source_bytes;
        assert_eq!(
            (RT_API.read32)(
                &mut decoded as *mut Machine as *mut std::ffi::c_void,
                0x08000000 + at as u32
            ),
            u32::from_le_bytes(expect)
        );
        let delta = owner
            .unit_obj_ffff_reader_fields_stats()
            .unwrap()
            .source_bytes
            - before;
        assert_eq!(delta, owned);
        mixed.push(delta);
        decoded.bus.rom[raw..raw + 2].copy_from_slice(&rom[raw..raw + 2]);
    }
    sources(&mut decoded, &files, &rom, false);
    let mut swi_counts = Vec::new();
    for (swi, src, count, expect) in [(0xdf0b, 0x3b4c10, 4, 2), (0xdf0c, 0x3d482c, 8, 4)] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        sources(&mut decoded, &files, &rom, true);
        let before = owner
            .unit_obj_ffff_reader_fields_stats()
            .unwrap()
            .source_bytes;
        legacy.step();
        decoded.step();
        let delta = owner
            .unit_obj_ffff_reader_fields_stats()
            .unwrap()
            .source_bytes
            - before;
        assert_eq!(delta, expect);
        swi_counts.push(delta);
        sources(&mut decoded, &files, &rom, false);
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x0c3d482e);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 4);
    }
    sources(&mut decoded, &files, &rom, true);
    let before = owner
        .unit_obj_ffff_reader_fields_stats()
        .unwrap()
        .source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let dma = owner
        .unit_obj_ffff_reader_fields_stats()
        .unwrap()
        .source_bytes
        - before;
    assert_eq!(dma, 4);
    sources(&mut decoded, &files, &rom, false);
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({
            "revision":[1,70,69],"original_interpreter_compiled_paths":cases.len(),
            "compiled_field_bytes":compiled,"full_states_clocks_equal":true,"all6raw_source_bytes_altered":true,
            "both_ranges_widths_mirrors_realBIOS_bus":true,"live_unmapped_BIOS_rotate16_prefixes":3,"zero_selected_word_full_early_return":true,
            "mixed_raw_new_API_source_bytes":mixed,"mixed_raw_new_SWI_source_bytes":swi_counts,"mixed_raw_new_DMA_source_bytes":dma,
            "instructions_coverage_only":instructions,"PCs":pcs.len(),"native_dispatches_coverage_only":dispatches,
            "scope":"two typed numeric fields and original query prefixes, no valid sprite sequence or ordinary claim",
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
        .unit_obj_ffff_reader_fields_stats()
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

fn names() -> [&'static str; 65] {
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
        UNIT_OBJ_BYTE_PROPERTY_WORDS_FILE,
        UNIT_OBJ_FFFF_READER_FIELDS_FILE,
    ]
}
