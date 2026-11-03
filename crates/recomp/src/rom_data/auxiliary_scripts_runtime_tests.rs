//! New auxiliary fields: managed authentication/lifecycle and original/native prefixes.
use super::*;
use gba_core::rom_data::auxiliary_scripts as model;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const CONTEXT: u32 = 0x02011000;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUXILIARY_MANAGED_ROOT").unwrap())
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
fn range(m: &mut Machine, files: &Dataset, rom: Option<&[u8]>) {
    for (at, bytes, _) in files
        .auxiliary_archive
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
        .into_iter()
        .chain(
            files
                .auxiliary_commands
                .as_ref()
                .unwrap()
                .source_spans()
                .unwrap(),
        )
    {
        let at = at as usize;
        if let Some(rom) = rom {
            m.bus.rom[at..at + bytes.len()].copy_from_slice(&rom[at..at + bytes.len()]);
        } else {
            for byte in &mut m.bus.rom[at..at + bytes.len()] {
                *byte ^= 0x5a;
            }
        }
    }
}
#[test]
#[ignore = "private EUR e54 auxiliary publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_auxiliary_scripts_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..48]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e53").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (53, 52));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (54, 53)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 50);
    assert_eq!(source.manifest.resources.len(), 49);
    for i in 1..48 {
        assert_eq!(files[i], old[i]);
    }
    let commands = parse_auxiliary_commands(&files[48]).unwrap();
    let archive = parse_auxiliary_archive(&files[49]).unwrap();
    archive.validate_dependencies(&commands).unwrap();
    let accepted =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUXILIARY_SOURCE_ROOT").unwrap());
    for name in [AUXILIARY_COMMANDS_FILE, AUXILIARY_ARCHIVE_FILE] {
        assert_eq!(
            fs::read(dataset().join(name)).unwrap(),
            fs::read(accepted.join(name)).unwrap()
        );
    }
    assert_eq!(
        source.manifest.resources[47].dependencies,
        Vec::<String>::new()
    );
    assert_eq!(
        source.manifest.resources[48].dependencies,
        vec![AUXILIARY_COMMANDS_ID.to_string()]
    );
    assert_eq!(
        (
            source.manifest.resources[47].sources[0].length,
            source.manifest.resources[48].sources[0].length
        ),
        (102, 4098)
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(AUXILIARY_ARCHIVE_FILE)
    {
        Err("injected auxiliary publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e53-d52/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e54-d53"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(AUXILIARY_COMMANDS_FILE), b"partial").unwrap();
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
    let fileset = read_dataset(&dataset()).unwrap();
    for missing_commands in [true, false] {
        let mut missing = fileset.files();
        if missing_commands {
            missing.auxiliary_commands = None;
        } else {
            missing.auxiliary_archive = None;
        }
        assert!(source.validate_files(missing).is_err());
    }
    let mut inconsistent = Extraction::from_rom(&rom).unwrap();
    inconsistent.script_coordinates = None;
    inconsistent.manifest.resources.remove(46);
    inconsistent
        .manifest
        .coverage
        .retain(|id| id != SCRIPT_COORDINATES_ID);
    assert_eq!(
        inconsistent.validate_files(fileset.files()),
        Err("Extraction resource set is inconsistent".into())
    );
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let mut guards = 0;
    for file_index in [48usize, 49] {
        let resource = file_index - 1;
        for case in 0..12 {
            let mut manifest = source.manifest.clone();
            let mut json: serde_json::Value = serde_json::from_slice(&files[file_index]).unwrap();
            match case {
                0 if file_index == 48 => {
                    json["commands"][0]["callback"]["rom_offset"] =
                        (commands.commands[0].callback.rom_offset + 2).into()
                }
                0 => {
                    let bank = archive
                        .banks
                        .iter()
                        .position(|b| {
                            b.programs
                                .iter()
                                .any(|p| p.instructions.iter().any(|i| i.command == 2))
                        })
                        .unwrap();
                    let program = archive.banks[bank]
                        .programs
                        .iter()
                        .position(|p| p.instructions.iter().any(|i| i.command == 2))
                        .unwrap();
                    let ins = archive.banks[bank].programs[program]
                        .instructions
                        .iter()
                        .position(|i| i.command == 2)
                        .unwrap();
                    let value =
                        archive.banks[bank].programs[program].instructions[ins].parameters[0] ^ 128;
                    json["banks"][bank]["programs"][program]["instructions"][ins]["parameters"]
                        [0] = value.into();
                }
                1 if file_index == 48 => {
                    json["commands"][1]["advance"] = (commands.commands[1].advance + 1).into()
                }
                1 => json["banks"][0]["entries"][0] = json["banks"][0]["entries"][4].clone(),
                2 => manifest.resources[resource].sources[0].sha256 = "0".repeat(64),
                3 => manifest.resources[resource].sources[0].offset += 2,
                4 => manifest.resources[resource].sources[0].length -= 2,
                5 => manifest.resources[resource].sources[0].role = "unknown/auxiliary".into(),
                6 => manifest.resources[resource].path = "../auxiliary.json".into(),
                7 => {
                    manifest.resources[resource].dependencies = if file_index == 48 {
                        vec![FONT_BANK_ID.into()]
                    } else {
                        vec![]
                    }
                }
                8 => manifest.resources[resource].sha256 = "0".repeat(64),
                9 => {
                    let id = manifest.resources[resource].id.clone();
                    manifest.coverage.retain(|v| v != &id);
                }
                10 => manifest.resources[resource].sources.clear(),
                _ => json["unknown"] = 1.into(),
            }
            let bytes = serde_json::to_vec_pretty(&json).unwrap();
            if case != 8 {
                manifest.resources[resource].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            fs::write(dataset().join(names()[file_index]), bytes).unwrap();
            fs::write(
                dataset().join(MANIFEST_FILE),
                toml::to_string_pretty(&manifest).unwrap(),
            )
            .unwrap();
            assert!(
                attach_from_directory(&mut m, &dataset()).is_err(),
                "resource{resource} guard{case}"
            );
            assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
            assert_eq!(m.save_state().unwrap(), boot);
            guards += 1;
        }
        fs::write(dataset().join(names()[file_index]), &files[file_index]).unwrap();
        fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    }
    for name in [AUXILIARY_COMMANDS_FILE, AUXILIARY_ARCHIVE_FILE] {
        fs::rename(dataset().join(name), r.join("missing.json")).unwrap();
        assert!(attach_from_directory(&mut m, &dataset()).is_err());
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
        guards += 1;
        fs::rename(r.join("missing.json"), dataset().join(name)).unwrap();
    }
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("copy");
    fs::create_dir(&copy).unwrap();
    for name in names() {
        fs::copy(dataset().join(name), copy.join(name)).unwrap();
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
    range(&mut m, &fileset, None);
    for (at, bytes, _) in archive
        .source_spans()
        .unwrap()
        .into_iter()
        .chain(commands.source_spans().unwrap())
    {
        for n in 0..bytes.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + at + n as u32),
                rom[at as usize + n]
            );
        }
    }
    let c = owner.auxiliary_commands_stats().unwrap();
    let a = owner.auxiliary_archive_stats().unwrap();
    assert_eq!(
        (
            c.source_bytes,
            c.advance_bytes,
            c.callback_bytes,
            c.commands_used
        ),
        (102, 34, 68, 17)
    );
    assert_eq!(
        (
            a.source_bytes,
            a.index_bytes,
            a.opcode_bytes,
            a.parameter_bytes,
            a.alignment_bytes,
            a.banks_used,
            a.programs_used,
            a.alignment_spans_used
        ),
        (4098, 900, 913, 2275, 10, 8, 133, 5)
    );
    assert_eq!(guards, 26);
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,54,53],"files":50,"resources":49,"source_ranges":2,"source_bytes":4200,"all47prior_decoded_and_e53_retained":true,"coherent_and_missing_file_guards":guards,"actual_directory_dependency_and_complete_set_required":true,"failed_attach_owner_state_transactional":true,"publication_failure_create_reuse_repair_move":true,"unavailable_load_undo_backup_restart_owner_retained":true,"typed_directory_bytes":[34,68],"typed_archive_bytes":[900,913,2275,10],"banks":8,"programs":133,"alignment_spans":5,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[derive(Clone, Debug)]
enum Probe {
    Lookup(u32, u32),
    Dispatch(u16, usize),
    RamOpcode(u8),
    Advance(u8, u16),
    Relative(u16, usize, u16),
    Byte(u32),
    Outer(u32),
}
fn program(data: &AuxiliaryArchive, id: u16) -> &model::Program {
    data.banks
        .iter()
        .flat_map(|b| &b.programs)
        .find(|p| p.id == id)
        .unwrap()
}
fn relative_bounds(op: u8) -> (u32, u32, usize, usize) {
    match op {
        1 => (0xa298, 0xa2a6, 0, 1),
        8 => (0xa43c, 0xa44a, 4, 3),
        13 => (0xa5c2, 0xa5d0, 5, 4),
        16 => (0xa6a0, 0xa6ae, 5, 4),
        _ => panic!("relative {op}"),
    }
}
impl Probe {
    fn bounds(&self, data: &AuxiliaryArchive) -> (u32, u32) {
        match *self {
            Self::Lookup(..) => (0x0800a1f8, 0x0800a216),
            Self::Dispatch(id, n) => (
                0x0800a240,
                if program(data, id).instructions[n].command == 0 {
                    0x08000300
                } else {
                    0x0800a270
                },
            ),
            Self::RamOpcode(..) => (0x0800a240, 0x0800a270),
            Self::Advance(..) => (0x0800a278, 0x0800a286),
            Self::Relative(id, n, _) => {
                let (s, e, _, _) = relative_bounds(program(data, id).instructions[n].command);
                (0x08000000 + s, 0x08000000 + e)
            }
            Self::Byte(_) => (0x0801414e, 0x08014150),
            Self::Outer(_) => (0x0800a158, 0x0800a1a4),
        }
    }
    fn raw(&self) -> Option<(usize, usize)> {
        match *self {
            Self::Byte(at)
                if at == model::SOURCE_END
                    || at == model::DIRECTORY_OFFSET - 1
                    || at == model::DIRECTORY_OFFSET + 102 =>
            {
                Some((at as usize, 1))
            }
            _ => None,
        }
    }
    fn old(&self) -> Option<(usize, usize)> {
        match *self {
            Self::Byte(at) if at == model::SOURCE_OFFSET - 1 || at == 0x370b20 => {
                Some((at as usize, 1))
            }
            _ => None,
        }
    }
    fn no_new_reads(&self) -> bool {
        self.raw().is_some()
            || self.old().is_some()
            || matches!(
                self,
                Self::Outer(_) | Self::Advance(17..=255, _) | Self::RamOpcode(17..=255)
            )
    }
    fn setup(&self, m: &mut Machine, data: &AuxiliaryArchive) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 1..13 {
            m.cpu.regs[n] = 0x11110000 + n as u32;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds(data).0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match *self {
            Self::Lookup(action, slot) => {
                m.cpu.regs[0] = CONTEXT;
                m.bus.write8(CONTEXT, action as u8);
                m.bus.write8(CONTEXT + 1, slot as u8);
            }
            Self::Dispatch(id, n) => {
                let p = program(data, id);
                m.cpu.regs[0] = CONTEXT;
                m.bus.write32(CONTEXT + 4, 0x08000000 + p.source_offset);
                m.bus.write16(
                    CONTEXT + 2,
                    (p.instructions[n].source_offset - p.source_offset) as u16,
                );
            }
            Self::RamOpcode(op) => {
                m.cpu.regs[0] = CONTEXT;
                m.bus.write32(CONTEXT + 4, 0x02020000);
                m.bus.write16(CONTEXT + 2, 0xffff);
                m.bus.write8(0x0201ffff, op);
            }
            Self::Advance(op, cursor) => {
                m.cpu.regs[4] = 0x08000000 + model::command_source_offset(op);
                m.cpu.regs[5] = CONTEXT;
                m.bus.write16(CONTEXT + 2, cursor);
            }
            Self::Relative(id, n, cursor) => {
                let ins = &program(data, id).instructions[n];
                let (_, _, ctx, src) = relative_bounds(ins.command);
                m.cpu.regs[ctx] = CONTEXT;
                m.cpu.regs[src] = 0x08000000 + ins.source_offset;
                m.bus.write16(CONTEXT + 2, cursor);
            }
            Self::Byte(at) => m.cpu.regs[5] = 0x08000000 + at,
            Self::Outer(action) => m.cpu.regs[0] = action,
        }
    }
}
fn original(m: &mut Machine, p: &Probe, data: &AuxiliaryArchive, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m, data);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds(data).1 {
        assert!(n < 500, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
#[test]
#[ignore = "private EUR e54 auxiliary fullstate-clock native fields and shared pages; Wine only, Linux/USA pending"]
fn european_auxiliary_scripts_native_fields_and_shared_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.auxiliary_archive.as_ref().unwrap();
    let owns = |at: usize, n: usize| {
        files
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| at < (s.offset + s.length) as usize && (s.offset as usize) < at + n)
    };
    let mut cases = vec![
        Probe::Lookup(0, 16),
        Probe::Lookup(2, 0),
        Probe::Lookup(7, 62),
        Probe::Lookup(8, 0),
        Probe::Lookup(8, 1),
        Probe::Lookup(255, 255),
        Probe::Lookup(2, 63),
    ];
    for op in 0..17 {
        if let Some((p, n)) = data.banks.iter().flat_map(|b| &b.programs).find_map(|p| {
            p.instructions
                .iter()
                .position(|i| i.command == op)
                .map(|n| (p.id, n))
        }) {
            cases.push(Probe::Dispatch(p, n));
        }
    }
    cases.extend([
        Probe::RamOpcode(3),
        Probe::RamOpcode(255),
        Probe::Advance(0, 65535),
        Probe::Advance(1, 32767),
        Probe::Advance(4, 65535),
        Probe::Advance(17, 65535),
        Probe::Advance(255, 65535),
    ]);
    for op in [1, 8, 13, 16] {
        let (p, n) = data
            .banks
            .iter()
            .flat_map(|b| &b.programs)
            .find_map(|p| {
                p.instructions
                    .iter()
                    .position(|i| i.command == op)
                    .map(|n| (p.id, n))
            })
            .unwrap();
        cases.push(Probe::Relative(p, n, 65535));
    }
    let gaps: Vec<_> = data
        .banks
        .iter()
        .filter(|b| !b.alignment_after.is_empty())
        .map(|b| {
            b.programs
                .last()
                .unwrap()
                .instructions
                .last()
                .unwrap()
                .source_offset
                + 1
        })
        .collect();
    cases.extend(
        [
            gaps[0],
            *gaps.last().unwrap() + 1,
            model::DIRECTORY_OFFSET + 3,
            model::SOURCE_OFFSET - 1,
            0x370b20,
            model::DIRECTORY_OFFSET - 1,
            model::DIRECTORY_OFFSET + 102,
            model::SOURCE_END,
        ]
        .into_iter()
        .map(Probe::Byte),
    );
    cases.extend([Probe::Outer(2), Probe::Outer(0x102)]);
    for p in &cases {
        if let Some((at, n)) = p.raw() {
            assert!(!owns(at, n));
        }
        if let Some((at, n)) = p.old() {
            assert!(owns(at, n));
        }
    }
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, data, &mut pcs);
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
    let object = out.join("auxiliary-scripts.o");
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
    COMPILED.with(|v| v.set([0; 6]));
    let mut dispatches = 0;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
            if let Some((at, n)) = p.raw() {
                for v in &mut m.bus.rom[at..at + n] {
                    *v ^= 0x5a;
                }
            }
        }
        original(&mut legacy, p, data, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            range(m, &files, None);
            if let Some((at, n)) = p.old() {
                for v in &mut m.bus.rom[at..at + n] {
                    *v ^= 0x5a;
                }
            }
        }
        let before = usage(&mut decoded as *mut Machine as *mut std::ffi::c_void)
            .iter()
            .sum::<u64>();
        original(&mut decoded, p, data, &mut BTreeSet::new());
        p.setup(&mut native, data);
        STOP_PC.with(|v| v.set(p.bounds(data).1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds(data).1 {
            assert!(n < 500, "native{p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .unwrap_or_else(|| panic!("missing native{p:?} pc{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        let after = usage(&mut decoded as *mut Machine as *mut std::ffi::c_void)
            .iter()
            .sum::<u64>();
        assert_eq!(after == before, p.no_new_reads(), "{p:?}");
        for m in [&mut decoded, &mut native] {
            range(m, &files, Some(&rom));
            if let Some((at, n)) = p.old() {
                m.bus.rom[at..at + n].copy_from_slice(&rom[at..at + n]);
            }
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            if let Some((at, n)) = p.raw() {
                m.bus.rom[at..at + n].copy_from_slice(&rom[at..at + n]);
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
    assert!(
        compiled.iter().all(|n| *n > 0),
        "all six typed source classes {compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    range(&mut decoded, &files, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for boundary in [
            model::SOURCE_OFFSET,
            model::SOURCE_END,
            model::DIRECTORY_OFFSET,
            model::DIRECTORY_OFFSET + 102,
        ] {
            for at in boundary - 3..boundary + 4 {
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
    decoded.bus.real_bios = false;
    let at = model::SOURCE_OFFSET - 2;
    for v in &mut decoded.bus.rom[at as usize..at as usize + 2] {
        *v ^= 0x5a;
    }
    let a = owner.auxiliary_archive_stats().unwrap().source_bytes;
    let c = owner.script_coordinates_stats().unwrap().source_bytes;
    assert_eq!(
        (RT_API.read32)(
            &mut decoded as *mut Machine as *mut std::ffi::c_void,
            0x08000000 + at
        ),
        u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
    );
    assert_eq!(owner.auxiliary_archive_stats().unwrap().source_bytes - a, 2);
    assert_eq!(
        owner.script_coordinates_stats().unwrap().source_bytes - c,
        2
    );
    decoded.bus.rom[at as usize..at as usize + 2]
        .copy_from_slice(&rom[at as usize..at as usize + 2]);
    for end in [model::SOURCE_END, model::DIRECTORY_OFFSET + 102] {
        assert!(!owns(end as usize, 2));
        for v in &mut decoded.bus.rom[end as usize..end as usize + 2] {
            *v ^= 0x5a;
        }
        let before = owner.auxiliary_archive_stats().unwrap().source_bytes
            + owner.auxiliary_commands_stats().unwrap().source_bytes;
        let mut expected: [u8; 4] = rom[end as usize - 2..end as usize + 2].try_into().unwrap();
        for v in &mut expected[2..] {
            *v ^= 0x5a;
        }
        assert_eq!(
            decoded.bus.read32(0x08000000 + end - 2),
            u32::from_le_bytes(expected)
        );
        assert_eq!(
            owner.auxiliary_archive_stats().unwrap().source_bytes
                + owner.auxiliary_commands_stats().unwrap().source_bytes
                - before,
            2
        );
        decoded.bus.rom[end as usize..end as usize + 2]
            .copy_from_slice(&rom[end as usize..end as usize + 2]);
    }
    range(&mut decoded, &files, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, model::SOURCE_OFFSET, 8),
        (0xdf0c, model::SOURCE_OFFSET + 4, 8),
        (0xdf0b, model::DIRECTORY_OFFSET, 8),
        (0xdf0b, model::SOURCE_OFFSET - 2, 3),
        (0xdf0b, model::SOURCE_END - 2, 3),
        (0xdf0b, model::DIRECTORY_OFFSET + 100, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, &files, None);
        let before = owner.auxiliary_archive_stats().unwrap().source_bytes
            + owner.auxiliary_commands_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(
            owner.auxiliary_archive_stats().unwrap().source_bytes
                + owner.auxiliary_commands_stats().unwrap().source_bytes
                > before
        );
        range(&mut decoded, &files, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus
            .write32(0x040000d4, 0x08000000 + model::SOURCE_END - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, &files, None);
    let before = owner.auxiliary_archive_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.auxiliary_archive_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, &files, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,54,53],"original_interpreter_native_paths":cases.len(),"compiled_advance_callback_index_opcode_parameter_alignment_bytes":compiled,"source_bytes":4200,"full_states_clocks_equal":true,"raw8_1_u32_LDRH_highbits":true,"interior_entries_signed_cursor_raw_opcodes":true,"shared_370_audio_and_D69coordinate_raw_tails":true,"edges_widths_mirrors_realBIOS_CpuSet_FastSet_DMA":true,"instructions_coverage_only":instructions,"pcs":pcs.len(),"native_dispatches_coverage_only":dispatches,"scope":"auxiliary fields and original prefixes; no whole predicates/interpreter/callback/ordinary ABI claim","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

thread_local! {
    static COMPILED:Cell<[u64;6]>=const{Cell::new([0;6])};
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
fn usage(m: *mut std::ffi::c_void) -> [u64; 6] {
    let resources = unsafe { (&*(m as *const Machine)).bus.rom_resources().unwrap() };
    let c = resources.auxiliary_commands_stats().unwrap();
    let a = resources.auxiliary_archive_stats().unwrap();
    [
        c.advance_bytes,
        c.callback_bytes,
        a.index_bytes,
        a.opcode_bytes,
        a.parameter_bytes,
        a.alignment_bytes,
    ]
}
fn count(before: [u64; 6], after: [u64; 6]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for ((value, before), after) in x.iter_mut().zip(before).zip(after) {
            *value += after - before;
        }
        v.set(x);
    });
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

fn names() -> [&'static str; 50] {
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
    ]
}
