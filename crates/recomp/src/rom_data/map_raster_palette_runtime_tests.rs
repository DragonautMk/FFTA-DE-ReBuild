//! Two reduced Wine filters: new file lifecycle and original/native raster source reads.
use super::*;
use gba_core::{capi::RT_API, rom_data::map_raster_palettes as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_RASTER_MANAGED_ROOT").unwrap())
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
fn ranges(files: &Dataset) -> Vec<(usize, usize, u8)> {
    let mut result: Vec<_> = files
        .map_raster_palettes
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
        .into_iter()
        .map(|(at, b, _)| (at as usize, b.len(), 0x5a))
        .collect();
    let alt = files
        .manifest
        .resources
        .iter()
        .find(|r| r.id == BG_ALTERNATE_PALETTES_ID)
        .unwrap()
        .sources
        .iter()
        .find(|s| s.offset == model::SOURCE_END)
        .unwrap();
    result.push((alt.offset as usize, alt.length as usize, 0xa5));
    result
}
fn sources(m: &mut Machine, files: &Dataset, rom: &[u8], poison: bool) {
    for (at, len, salt) in ranges(files) {
        m.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        if poison {
            for byte in &mut m.bus.rom[at..at + len] {
                *byte ^= salt;
            }
        }
    }
}

#[test]
#[ignore = "private EUR e55 raster publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_map_raster_palettes_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..50]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e54").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (54, 53));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (55, 54)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 51);
    assert_eq!(source.manifest.resources.len(), 50);
    for i in 1..50 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_map_raster_palettes(&files[50]).unwrap();
    let accepted = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_RASTER_SOURCE_ROOT").unwrap());
    assert_eq!(
        files[50],
        fs::read(accepted.join(MAP_RASTER_PALETTES_FILE)).unwrap()
    );
    assert_eq!(
        source.manifest.resources[49].dependencies,
        vec![BG_DESCRIPTORS_ID.to_string()]
    );
    assert_eq!(source.manifest.resources[49].sources.len(), 33);
    assert_eq!(
        source.manifest.resources[49]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        10496
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(MAP_RASTER_PALETTES_FILE)
    {
        Err("injected raster publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e54-d53/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e55-d54"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(MAP_RASTER_PALETTES_FILE),
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
    let fileset = read_dataset(&dataset()).unwrap();
    let mut missing = fileset.files();
    missing.map_raster_palettes = None;
    assert!(source.validate_files(missing).is_err());
    let mut missing = fileset.files();
    missing.background = None;
    assert!(source.validate_files(missing).is_err());
    let mut inconsistent = Extraction::from_rom(&rom).unwrap();
    inconsistent.auxiliary_scripts = None;
    inconsistent.manifest.resources.remove(48);
    inconsistent.manifest.resources.remove(47);
    assert_eq!(
        inconsistent.validate_files(fileset.files()),
        Err("Extraction resource set is inconsistent".into())
    );
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let mut guards = 0;
    for case in 0..16 {
        let mut manifest = source.manifest.clone();
        let mut json: serde_json::Value = serde_json::from_slice(&files[50]).unwrap();
        match case {
            0 => json["programs"][0]["colors"][0] = (data.programs[0].colors[0] ^ 0x8000).into(),
            1 => json["programs"][0]["header"]["range_start"] = 0x8000.into(),
            2 => json["descriptor_programs"][0][0] = 1.into(),
            3 => manifest.resources[49].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[49].sources[0].offset += 2,
            5 => manifest.resources[49].sources[0].length -= 2,
            6 => manifest.resources[49].sources[0].role = "unknown/raster".into(),
            7 => manifest.resources[49].path = "../raster.json".into(),
            8 => manifest.resources[49].dependencies = vec![AUXILIARY_ARCHIVE_ID.into()],
            9 => manifest.resources[49].sha256 = "0".repeat(64),
            10 => manifest.coverage.retain(|id| id != MAP_RASTER_PALETTES_ID),
            11 => {
                manifest.resources[49].sources.pop();
            }
            12 => json["unknown"] = 1.into(),
            13 => json["programs"][31]["colors"]
                .as_array_mut()
                .unwrap()
                .push(0x13c.into()),
            14 => {
                json["descriptor_programs"].as_array_mut().unwrap().pop();
            }
            _ => json["programs"][1]["source_offset"] = data.programs[0].source_offset.into(),
        }
        let bytes = serde_json::to_vec_pretty(&json).unwrap();
        if case != 9 {
            manifest.resources[49].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(MAP_RASTER_PALETTES_FILE), bytes).unwrap();
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
    fs::write(dataset().join(MAP_RASTER_PALETTES_FILE), &files[50]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(MAP_RASTER_PALETTES_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    guards += 1;
    fs::rename(
        r.join("missing.json"),
        dataset().join(MAP_RASTER_PALETTES_FILE),
    )
    .unwrap();
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
    sources(&mut m, &fileset, &rom, true);
    for at in model::SOURCE_OFFSET..model::SOURCE_END {
        assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
    }
    let i = owner.map_raster_palettes_stats().unwrap();
    assert_eq!(
        (
            i.source_bytes,
            i.index_bytes,
            i.header_bytes,
            i.color_bytes,
            i.programs_used
        ),
        (10496, 64, 192, 10240, 32)
    );
    let before = owner.alternate_palette_stats().unwrap().table_bytes;
    assert_eq!(
        m.bus.read32(0x08000000 + model::SOURCE_END - 2),
        u32::from_le_bytes(
            rom[model::SOURCE_END as usize - 2..model::SOURCE_END as usize + 2]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        owner.map_raster_palettes_stats().unwrap().source_bytes - i.source_bytes,
        2
    );
    assert_eq!(
        owner.alternate_palette_stats().unwrap().table_bytes - before,
        2
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,55,54],"files":51,"resources":50,"source_ranges":33,"source_bytes":10496,"all49prior_decoded_and_e54_retained":true,"guards":guards,"actual_BGdependency_complete_set":true,"failed_attach_owner_state_transactional":true,"publication_failure_create_reuse_repair_move":true,"unavailable_load_undo_backup_restart_sharedowner":true,"typed_bytes":[64,192,10240],"programs_used":32,"mixed_owner_halfwords":[2,2],"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[derive(Clone, Debug)]
enum Probe {
    Init(u32, u32, u16),
    Getter(u32, u16),
    Prefix(u32),
    Update(u32),
    Explicit(u32, u8),
    Queue,
    Read(u32, usize),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Init(..) => (0x0801eaf0, 0x08000300),
            Self::Getter(..) => (0x0801edec, 0x08000300),
            Self::Prefix(..) => (0x0801eaf0, 0x0801eb1c),
            Self::Update(..) => (0x0801eed8, 0x08000300),
            Self::Explicit(..) => (0x0801ee28, 0x08000300),
            Self::Queue => (0x0801ecac, 0x08000300),
            Self::Read(_, 1) => (0x0801414e, 0x08014150),
            Self::Read(_, 2) => (0x0801eb46, 0x0801eb48),
            Self::Read(_, 4) => (0x0801eab2, 0x0801eab4),
            _ => unreachable!(),
        }
    }
    fn setup(&self, m: &mut Machine) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 4..12 {
            m.cpu.regs[n] = 0x14104000 + n as u32 * 0x101;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.write16(0x02007f22, 0);
        match *self {
            Self::Init(a, b, current) => {
                m.cpu.regs[0] = a;
                m.cpu.regs[1] = b;
                m.bus.write16(0x02007f22, current);
            }
            Self::Getter(a, current) => {
                m.cpu.regs[0] = a;
                m.bus.write16(0x02007f22, current);
            }
            Self::Prefix(a) => m.cpu.regs[0] = a,
            Self::Update(at) => {
                m.bus.write8(0x02007f20, 1);
                m.bus.write8(0x02007f26, 1);
                m.bus.write32(0x02007f28, 0x08000000 + at);
            }
            Self::Explicit(at, active) => {
                m.cpu.regs[0] = 0x08000000 + at;
                m.bus.write8(0x02007f20, active);
            }
            Self::Queue => {
                m.bus.write16(0x03002bc4, 0);
                m.bus.write16(0x02008390, 0x8123);
            }
            Self::Read(at, 1 | 2) => m.cpu.regs[5] = 0x08000000 + at,
            Self::Read(at, 4) => m.cpu.regs[4] = 0x08000000 + at - 16,
            _ => unreachable!(),
        }
    }
    fn reads_new(&self) -> bool {
        let overlaps = |at: u32, n: u32| at < model::SOURCE_END && at + n > model::SOURCE_OFFSET;
        match *self {
            Self::Init(_, _, 0xff) | Self::Queue => false,
            Self::Init(..) => true,
            Self::Getter(raw, current) => {
                model::getter_index_offset(raw, current).is_some_and(|at| overlaps(at, 2))
            }
            Self::Prefix(raw) => overlaps(model::initializer_index_offset(raw), 2),
            Self::Update(..) | Self::Explicit(..) => true,
            Self::Read(at, n) => overlaps(at & !(n as u32 - 1), n as u32),
        }
    }
}
fn original(m: &mut Machine, p: &Probe, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 1000, "{p:?} PC{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}

#[test]
#[ignore = "private EUR e55 raster original/native fullstate-clock and mixed owners; Wine only, Linux/USA pending"]
fn european_map_raster_palettes_native_reads_copies_and_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.map_raster_palettes.as_ref().unwrap();
    let first = data.programs[0].source_offset + 6;
    let last = data.programs[31].source_offset + 6;
    let owns = |at: u32, n: u32| {
        files
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| at < s.offset + s.length && s.offset < at + n)
    };
    assert!(!owns(model::SOURCE_OFFSET - 4, 4));
    let cases = vec![
        Probe::Init(0, 0, 0),
        Probe::Init(31, 0, 0),
        Probe::Init(0, 1, 0),
        Probe::Init(31, 1, 0),
        Probe::Init(0x12340001, 0x100, 0),
        Probe::Init(31, 0, 0xff),
        Probe::Getter(31, 0),
        Probe::Getter(0xffff, 31),
        Probe::Getter(32, 0),
        Probe::Getter(0xfffe, 0),
        Probe::Getter(0xcafeffff, 0xffff),
        Probe::Getter(0, 0xff),
        Probe::Prefix(32),
        Probe::Prefix(0x8000),
        Probe::Prefix(0xffff),
        Probe::Update(last),
        Probe::Update(last + 2),
        Probe::Explicit(last, 0),
        Probe::Explicit(last, 1),
        Probe::Queue,
        Probe::Read(model::SOURCE_OFFSET, 1),
        Probe::Read(model::SOURCE_OFFSET + model::INDEX_BYTES as u32, 2),
        Probe::Read(first, 2),
        Probe::Read(first + 2, 4),
        Probe::Read(model::SOURCE_END - 2, 4),
        Probe::Read(model::SOURCE_END - 1, 2),
        Probe::Read(model::SOURCE_END, 1),
        Probe::Read(model::SOURCE_OFFSET - 4, 1),
    ];
    let mut tracer = crate::make_machine(rom.clone(), None);
    let start = tracer.save_state().unwrap();
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    for p in &cases {
        tracer.load_state(&start).unwrap();
        instructions += original(&mut tracer, p, &mut pcs);
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
    let object = out.join("map-raster-palettes.o");
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
    COMPILED.with(|v| v.set([0; 3]));
    let mut dispatches = 0;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
            if matches!(p,Probe::Read(at,_) if *at==model::SOURCE_OFFSET-4) {
                m.bus.rom[model::SOURCE_OFFSET as usize - 4] ^= 0x33;
            }
        }
        original(&mut legacy, p, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, true);
        }
        let before = owner.map_raster_palettes_stats().unwrap().source_bytes;
        original(&mut decoded, p, &mut BTreeSet::new());
        p.setup(&mut native);
        STOP_PC.with(|v| v.set(p.bounds().1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 1000, "native{p:?} PC{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .unwrap_or_else(|| panic!("missing native {p:?} PC{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        assert_eq!(
            owner.map_raster_palettes_stats().unwrap().source_bytes > before,
            p.reads_new(),
            "{p:?}"
        );
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, false);
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.bus.rom[model::SOURCE_OFFSET as usize - 4] = rom[model::SOURCE_OFFSET as usize - 4];
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
        "all typed index/header/color API bytes {compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    sources(&mut decoded, &files, &rom, true);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for boundary in [
            model::SOURCE_OFFSET,
            model::SOURCE_OFFSET + model::INDEX_BYTES as u32,
            first,
            model::SOURCE_END,
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
    let before = owner.map_raster_palettes_stats().unwrap().source_bytes;
    let old = owner.alternate_palette_stats().unwrap().table_bytes;
    assert_eq!(
        decoded.bus.read32(0x08000000 + model::SOURCE_END - 2),
        u32::from_le_bytes(
            rom[model::SOURCE_END as usize - 2..model::SOURCE_END as usize + 2]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        owner.map_raster_palettes_stats().unwrap().source_bytes - before,
        2
    );
    assert_eq!(
        owner.alternate_palette_stats().unwrap().table_bytes - old,
        2
    );
    let at = model::SOURCE_OFFSET - 2;
    for v in &mut decoded.bus.rom[at as usize..at as usize + 2] {
        *v ^= 0x33;
    }
    let mut expected: [u8; 4] = rom[at as usize..at as usize + 4].try_into().unwrap();
    for v in &mut expected[..2] {
        *v ^= 0x33;
    }
    assert_eq!(
        decoded.bus.read32(0x08000000 + at),
        u32::from_le_bytes(expected)
    );
    decoded.bus.rom[at as usize..at as usize + 2]
        .copy_from_slice(&rom[at as usize..at as usize + 2]);
    sources(&mut decoded, &files, &rom, false);
    for (swi, src, count) in [
        (0xdf0b, last, 161),
        (0xdf0c, model::SOURCE_OFFSET + model::INDEX_BYTES as u32, 8),
        (0xdf0b, model::SOURCE_END - 2, 3),
        (0xdf0b, model::SOURCE_OFFSET - 2, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        sources(&mut decoded, &files, &rom, true);
        let before = owner.map_raster_palettes_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.map_raster_palettes_stats().unwrap().source_bytes > before);
        sources(&mut decoded, &files, &rom, false);
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus
            .write32(0x040000d4, 0x08000000 + model::SOURCE_END - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    sources(&mut decoded, &files, &rom, true);
    let before = owner.map_raster_palettes_stats().unwrap().source_bytes;
    let old = owner.alternate_palette_stats().unwrap().table_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.map_raster_palettes_stats().unwrap().source_bytes - before,
        2
    );
    assert_eq!(
        owner.alternate_palette_stats().unwrap().table_bytes - old,
        2
    );
    sources(&mut decoded, &files, &rom, false);
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,55,54],"original_interpreter_native_paths":cases.len(),"compiled_index_header_color_bytes":compiled,"full_states_clocks_equal":true,"raw_signed_unsigned_sentinels_livewindows":true,"original_copy_wrap_and_queuedDMA":true,"mixed_alternate_owner_and_raw_neighbor":true,"widths_mirrors_realBIOS_bus_CpuSet_FastSet_DMA":true,"instructions_coverage_only":instructions,"pcs":pcs.len(),"native_dispatches_coverage_only":dispatches,"scope":"new typed raster source/guest copies; no whole camera/rasterDMA/ordinary ABI claim","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

thread_local! {
    static COMPILED:Cell<[u64;3]>=const{Cell::new([0;3])};
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
fn usage(m: *mut std::ffi::c_void) -> [u64; 3] {
    let resources = unsafe { (&*(m as *const Machine)).bus.rom_resources().unwrap() };
    let i = resources.map_raster_palettes_stats().unwrap();
    [i.index_bytes, i.header_bytes, i.color_bytes]
}
fn count(before: [u64; 3], after: [u64; 3]) {
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

fn names() -> [&'static str; 51] {
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
    ]
}
