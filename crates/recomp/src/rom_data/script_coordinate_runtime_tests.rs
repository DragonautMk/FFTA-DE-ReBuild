//! New coordinate owner: managed lifecycle and original/native source reads, including live RAM.
use super::*;
use gba_core::rom_data::script_coordinates as model;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const MESSAGE: u32 = 0x02011000;
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COORDINATE_MANAGED_ROOT").unwrap())
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
fn range(m: &mut Machine, data: &ScriptCoordinates, rom: Option<&[u8]>) {
    for (at, bytes, _) in data.source_spans().unwrap() {
        let at = at as usize;
        let dst = &mut m.bus.rom[at..at + bytes.len()];
        if let Some(r) = rom {
            dst.copy_from_slice(&r[at..at + bytes.len()]);
        } else {
            for v in dst {
                *v ^= 0x5a;
            }
        }
    }
}
#[derive(Clone, Debug)]
enum Probe {
    Index(u32),
    Count(usize),
    Point(usize, usize, u8),
    Copy(usize, u32, u8),
    Byte(u32),
    Destination(u32),
}
impl Probe {
    fn bounds(&self) -> (u32, u32) {
        match self {
            Self::Index(_) => (0x08128914, 0x0812892c),
            Self::Count(_) => (0x0812893a, 0x08128944),
            Self::Point(..) => (0x08128958, 0x0812896e),
            Self::Copy(..) => (0x08128914, 0x08000300),
            Self::Byte(_) => (0x0801414e, 0x08014150),
            Self::Destination(_) => (0x08125768, 0x08000300),
        }
    }
    fn raw(&self) -> Option<(usize, usize)> {
        match *self {
            Self::Byte(at) if at >= model::SOURCE_END => Some((at as usize, 1)),
            _ => None,
        }
    }
    fn old(&self) -> Option<(usize, usize)> {
        match *self {
            Self::Byte(at) if at < model::SOURCE_OFFSET => Some((at as usize, 1)),
            _ => None,
        }
    }
    fn no_new_reads(&self) -> bool {
        match *self {
            Self::Destination(_) => true,
            Self::Byte(at) => !(model::SOURCE_OFFSET..model::SOURCE_END).contains(&at),
            Self::Index(raw) => !(model::SOURCE_OFFSET..model::SOURCE_END)
                .contains(&model::index_source_offset(raw)),
            _ => false,
        }
    }
    fn setup(&self, m: &mut Machine, data: &ScriptCoordinates) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 1..13 {
            m.cpu.regs[n] = 0x11110000 + n as u32;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds().0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.write32(0x020159d4, 0x02020000);
        m.bus.write32(0x02007f14, 0x02030000);
        m.bus.write8(0x02007f18, 16);
        let salt = match *self {
            Self::Point(_, _, s) | Self::Copy(_, _, s) => s,
            _ => 17,
        };
        for y in 0..16u32 {
            for x in 0..16u32 {
                m.bus.write8(
                    0x02030000 + 2 * (y * 16 + x),
                    (x * 7 + y * 13 + u32::from(salt)) as u8,
                );
            }
        }
        match *self {
            Self::Index(raw) => {
                m.cpu.regs[0] = raw;
                m.cpu.regs[1] = 0x10f;
            }
            Self::Count(id) => {
                m.cpu.regs[5] = 0x08000000 + data.paths[id].source_offset;
                m.cpu.regs[2] = MESSAGE;
            }
            Self::Point(id, n, _) => {
                m.cpu.regs[6] = 0x08000000 + data.paths[id].source_offset + 2 + n as u32 * 2;
                m.cpu.regs[4] = MESSAGE;
                m.cpu.regs[9] = 0;
            }
            Self::Copy(id, destination, _) => {
                m.cpu.regs[0] = id as u32;
                m.cpu.regs[1] = destination;
            }
            Self::Byte(at) => m.cpu.regs[5] = 0x08000000 + at,
            Self::Destination(raw) => m.cpu.regs[0] = raw,
        }
    }
}
fn original(
    m: &mut Machine,
    p: &Probe,
    data: &ScriptCoordinates,
    pcs: &mut BTreeSet<u32>,
) -> usize {
    p.setup(m, data);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds().1 {
        assert!(n < 5000, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}

#[test]
#[ignore = "private EUR e53 coordinate fullstate-clock native field/copy/mixed-owner checks; Wine only, Linux/USA pending"]
fn european_script_coordinates_native_fields_and_shared_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.script_coordinates.as_ref().unwrap();
    let owns = |at: usize, n: usize| {
        files
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| at < (s.offset + s.length) as usize && (s.offset as usize) < at + n)
    };
    let max = data
        .paths
        .iter()
        .enumerate()
        .max_by_key(|(_, p)| p.points.len())
        .unwrap()
        .0;
    let fills: Vec<_> = data
        .paths
        .iter()
        .filter(|p| !p.alignment_after.is_empty())
        .map(|p| p.source_offset + 2 + p.points.len() as u32 * 2)
        .collect();
    let mut cases: Vec<_> = [0, 209, 210, 255, 65535, 0xbeef0001]
        .into_iter()
        .map(Probe::Index)
        .collect();
    cases.extend([0, 209, max].into_iter().map(Probe::Count));
    cases.extend([
        Probe::Point(0, 0, 17),
        Probe::Point(0, data.paths[0].points.len() - 1, 193),
        Probe::Point(209, 2, 17),
        Probe::Point(max, data.paths[max].points.len() - 1, 193),
        Probe::Copy(0, 0, 17),
        Probe::Copy(209, 255, 193),
        Probe::Copy(max, 15, 31),
        Probe::Copy(0, 0x10f, 93),
    ]);
    cases.extend(
        [
            fills[0],
            *fills.last().unwrap() + 1,
            model::SOURCE_OFFSET - 1,
            gba_core::rom_data::script_text::END - 1,
            model::SOURCE_END + 1,
        ]
        .into_iter()
        .map(Probe::Byte),
    );
    cases.extend([Probe::Destination(0), Probe::Destination(0x1ff)]);
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
    let object = out.join("script-coordinates.o");
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
    COMPILED.with(|v| v.set([0; 4]));
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
            range(m, data, None);
            if let Some((at, n)) = p.old() {
                for v in &mut m.bus.rom[at..at + n] {
                    *v ^= 0x5a;
                }
            }
        }
        let before = owner.script_coordinates_stats().unwrap().source_bytes;
        original(&mut decoded, p, data, &mut BTreeSet::new());
        p.setup(&mut native, data);
        STOP_PC.with(|v| v.set(p.bounds().1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds().1 {
            assert!(n < 5000, "native {p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .unwrap_or_else(|| panic!("missing native {p:?} pc{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            n += 1;
            dispatches += 1;
        }
        assert_eq!(
            owner.script_coordinates_stats().unwrap().source_bytes == before,
            p.no_new_reads(),
            "{p:?}"
        );
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
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
        "all four typed classes through compiled API {compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for offset in (model::SOURCE_OFFSET - 3..model::SOURCE_OFFSET + 4)
            .chain(model::SOURCE_END - 3..model::SOURCE_END + 4)
        {
            for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
                assert_eq!(decoded.bus.read8(mirror + offset), rom[offset as usize]);
                assert_eq!(
                    decoded.bus.read16(mirror + offset),
                    u16::from_le_bytes(
                        rom[offset as usize..offset as usize + 2]
                            .try_into()
                            .unwrap()
                    )
                );
                assert_eq!(
                    (RT_API.read32)(
                        &mut decoded as *mut Machine as *mut std::ffi::c_void,
                        mirror + offset
                    ),
                    u32::from_le_bytes(
                        rom[offset as usize..offset as usize + 4]
                            .try_into()
                            .unwrap()
                    )
                );
            }
        }
    }
    decoded.bus.real_bios = false;
    // Three owners share D68: text through C2C, keys through FD4, then this directory.
    for (at, new_bytes, key_bytes, text_bytes) in [
        (model::SOURCE_OFFSET - 2, 2, 2, 0),
        (gba_core::rom_data::script_text::END - 2, 0, 2, 2),
    ] {
        decoded.bus.rom[at as usize..at as usize + 4]
            .copy_from_slice(&rom[at as usize..at as usize + 4]);
        for v in &mut decoded.bus.rom[at as usize..at as usize + 4] {
            *v ^= 0x5a;
        }
        let a = owner.script_coordinates_stats().unwrap().source_bytes;
        let k = owner.script_archive_stats().unwrap().key_bytes;
        let t = owner.script_text_stats().unwrap().source_bytes;
        assert_eq!(
            (RT_API.read32)(
                &mut decoded as *mut Machine as *mut std::ffi::c_void,
                0x08000000 + at
            ),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
        assert_eq!(
            owner.script_coordinates_stats().unwrap().source_bytes - a,
            new_bytes
        );
        assert_eq!(
            owner.script_archive_stats().unwrap().key_bytes - k,
            key_bytes
        );
        assert_eq!(
            owner.script_text_stats().unwrap().source_bytes - t,
            text_bytes
        );
        decoded.bus.rom[at as usize..at as usize + 4]
            .copy_from_slice(&rom[at as usize..at as usize + 4]);
    }
    let end = model::SOURCE_END;
    assert!(!owns(end as usize, 4));
    for v in &mut decoded.bus.rom[end as usize..end as usize + 2] {
        *v ^= 0x5a;
    }
    let before = owner.script_coordinates_stats().unwrap().source_bytes;
    let mut expected: [u8; 4] = rom[end as usize - 2..end as usize + 2].try_into().unwrap();
    for v in &mut expected[2..] {
        *v ^= 0x5a;
    }
    assert_eq!(
        decoded.bus.read32(0x08000000 + end - 2),
        u32::from_le_bytes(expected)
    );
    assert_eq!(
        owner.script_coordinates_stats().unwrap().source_bytes - before,
        2
    );
    decoded.bus.rom[end as usize..end as usize + 2]
        .copy_from_slice(&rom[end as usize..end as usize + 2]);
    range(&mut decoded, data, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, model::SOURCE_OFFSET, 8),
        (0xdf0c, model::SOURCE_OFFSET + 4, 8),
        (0xdf0b, model::SOURCE_OFFSET - 2, 3),
        (0xdf0b, model::SOURCE_END - 2, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let before = owner.script_coordinates_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.script_coordinates_stats().unwrap().source_bytes > before);
        range(&mut decoded, data, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + end - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let before = owner.script_coordinates_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.script_coordinates_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,53,52],"original_interpreter_native_paths":cases.len(),"compiled_index_count_coordinate_alignment_bytes":compiled,"poisoned_source_bytes":3056,"full_states_clocks_equal":true,"full_coordinate_copy_paths":4,"raw_indices_destinations_live_height":true,"triple_D68_text_key_coordinate_and_upper_raw_boundary":true,"edges_widths_mirrors_bios_cpuset_fastset_dma":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"scope":"new coordinate field/copy/owner integration, not whole allocator/movement task or ordinary reachability","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[test]
#[ignore = "private EUR e53 coordinate publication/authentication/retention/lifecycle; Wine only, Linux/USA pending"]
fn european_script_coordinates_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..47]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e52").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (52, 51));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (53, 52)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 48);
    assert_eq!(source.manifest.resources.len(), 47);
    for i in 1..47 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_script_coordinates(&files[47]).unwrap();
    assert_eq!(data.source_spans().unwrap()[0].1.len(), 3056);
    assert_eq!(
        files[47],
        fs::read(std::env::var_os("FFTA_STATE_TEST_COORDINATE_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(source.manifest.resources[46].sources.len(), 1);
    assert_eq!(
        source.manifest.resources[46].dependencies,
        vec![
            SCRIPT_ARCHIVE_ID.to_string(),
            SCRIPT_COMMANDS_ID.to_string()
        ]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(SCRIPT_COORDINATES_FILE)
    {
        Err("injected coordinate publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e52-d51/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e53-d52"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(SCRIPT_COORDINATES_FILE), b"partial").unwrap();
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
    for (index, id) in [(44, SCRIPT_ARCHIVE_ID), (43, SCRIPT_COMMANDS_ID)] {
        let mut missing = Extraction::from_rom(&rom).unwrap();
        if index == 44 {
            missing.script_archive = None;
        } else {
            missing.script_commands = None;
        }
        assert_eq!(missing.manifest.resources.remove(index).id, id);
        missing.manifest.coverage.retain(|v| v != id);
        assert_eq!(
            missing.validate_files(fileset.files()),
            Err("Extraction resource set is inconsistent".into())
        );
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    for case in 0..13 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.paths[0].points[0].x ^= 1,
            1 => {
                forged.paths[0].points.pop();
            }
            2 => forged.script_links[0].path_id = (forged.script_links[0].path_id + 1) % 210,
            3 => manifest.resources[46].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[46].sources[0].length -= 4,
            5 => manifest.resources[46].sources[0].offset += 4,
            6 => manifest.resources[46].sources[0].role = "graphics/unknown".into(),
            7 => manifest.resources[46].path = "../script-coordinate-paths.json".into(),
            8 => manifest.resources[46].dependencies = vec![FONT_BANK_ID.into()],
            9 => manifest.resources[46].sha256 = "0".repeat(64),
            10 => manifest.coverage.retain(|id| id != SCRIPT_COORDINATES_ID),
            11 => manifest.resources[46].sources.clear(),
            _ => manifest.resources[46].dependencies.reverse(),
        }
        let bytes = encode_script_coordinates(&forged).unwrap();
        if case != 9 {
            manifest.resources[46].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(SCRIPT_COORDINATES_FILE), bytes).unwrap();
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
    }
    fs::write(dataset().join(SCRIPT_COORDINATES_FILE), &files[47]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(SCRIPT_COORDINATES_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(
        r.join("missing.json"),
        dataset().join(SCRIPT_COORDINATES_FILE),
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
    range(&mut m, &data, None);
    for at in model::SOURCE_OFFSET..model::SOURCE_END {
        assert_eq!(m.bus.read8(0x08000000 + at), rom[at as usize]);
    }
    let stats = owner.script_coordinates_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.index_bytes,
            stats.count_bytes,
            stats.coordinate_bytes,
            stats.alignment_bytes,
            stats.paths_used,
            stats.alignment_spans_used
        ),
        (3056, 840, 420, 1712, 84, 210, 42)
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,53,52],"files":48,"resources":47,"source_ranges":1,"source_bytes":3056,"all46prior_decoded_files_and_e52_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_and_missing_file_guards":14,"both_actual_dependencies_required_before_files":true,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_categories_210paths856points42alignment":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

thread_local! {
    static COMPILED:Cell<[u64;4]>=const{Cell::new([0;4])};
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
fn usage(m: *mut std::ffi::c_void) -> [u64; 4] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .script_coordinates_stats()
            .unwrap()
    };
    [
        s.index_bytes,
        s.count_bytes,
        s.coordinate_bytes,
        s.alignment_bytes,
    ]
}
fn count(before: [u64; 4], after: [u64; 4]) {
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

fn names() -> [&'static str; 48] {
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
    ]
}
