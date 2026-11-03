//! Two reduced Wine filters: Color-Mode palette lifecycle and original/native reads.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COLOR_MODE_MANAGED_ROOT").unwrap())
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
        .color_mode_palettes
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
        .into_iter()
        .map(|(at, b, _)| (at as usize, b.len(), 0x5a))
        .collect();
    let old = files
        .manifest
        .resources
        .iter()
        .find(|r| r.id == UI_RAW_PALETTES_ID)
        .unwrap()
        .sources
        .iter()
        .find(|s| s.offset == 0x40fcb0)
        .unwrap();
    result.push((old.offset as usize, 4, 0xa5));
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
#[ignore = "private EUR e58 Color-Mode palettes publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_color_mode_palettes_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..53]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e57").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (57, 56));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (58, 57)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 54);
    assert_eq!(source.manifest.resources.len(), 53);
    assert_eq!(source.manifest.resources[..52], prior.resources);
    assert_eq!(source.manifest.coverage[..52], prior.coverage);
    for i in 1..53 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_color_mode_palettes(&files[53]).unwrap();
    let accepted =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COLOR_MODE_SOURCE_ROOT").unwrap());
    assert_eq!(
        files[53],
        fs::read(accepted.join(COLOR_MODE_PALETTES_FILE)).unwrap()
    );
    assert_eq!(
        source.manifest.resources[52].dependencies,
        vec![METADATA14_ID.to_string()]
    );
    assert_eq!(source.manifest.resources[52].sources.len(), 30);
    assert_eq!(
        source.manifest.resources[52]
            .sources
            .iter()
            .map(|s| s.length)
            .sum::<u32>(),
        5750
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(COLOR_MODE_PALETTES_FILE)
    {
        Err("injected Color-Mode palette publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e57-d56/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e58-d57"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(COLOR_MODE_PALETTES_FILE),
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
    missing.color_mode_palettes = None;
    assert!(source.validate_files(missing).is_err());
    let mut missing = fileset.files();
    missing.metadata14 = None;
    assert!(source.validate_files(missing).is_err());
    let mut inconsistent = Extraction::from_rom(&rom).unwrap();
    inconsistent.metadata14 = None;
    inconsistent.manifest.resources.remove(24);
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
        let mut json: serde_json::Value = serde_json::from_slice(&files[53]).unwrap();
        match case {
            0 => json["spans"][0]["colors"][0] = (data.spans[0].colors[0] ^ 0x8001).into(),
            1 => json["slice_offsets"][0] = 255.into(),
            2 => json["spans"][0]["book"] = "shared-colors".into(),
            3 => manifest.resources[52].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[52].sources[0].offset += 2,
            5 => manifest.resources[52].sources[0].length -= 2,
            6 => manifest.resources[52].sources[0].role = "unknown/palette".into(),
            7 => manifest.resources[52].path = "../palette.json".into(),
            8 => manifest.resources[52].dependencies = vec![AUXILIARY_ARCHIVE_ID.into()],
            9 => manifest.resources[52].sha256 = "0".repeat(64),
            10 => manifest.coverage.retain(|id| id != COLOR_MODE_PALETTES_ID),
            11 => {
                manifest.resources[52].sources.pop();
            }
            12 => json["unknown"] = 1.into(),
            13 => {
                json["spans"].as_array_mut().unwrap().pop();
            }
            14 => json["spans"][0]["variant"] = 3.into(),
            _ => {
                json["spans"][0]["colors"].as_array_mut().unwrap().pop();
            }
        }
        let bytes = serde_json::to_vec_pretty(&json).unwrap();
        if case != 9 {
            manifest.resources[52].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(COLOR_MODE_PALETTES_FILE), bytes).unwrap();
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
    fs::write(dataset().join(COLOR_MODE_PALETTES_FILE), &files[53]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(COLOR_MODE_PALETTES_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    guards += 1;
    fs::rename(
        r.join("missing.json"),
        dataset().join(COLOR_MODE_PALETTES_FILE),
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
    for (at, bytes, _) in fileset
        .color_mode_palettes
        .as_ref()
        .unwrap()
        .source_spans()
        .unwrap()
    {
        for (n, &byte) in bytes.iter().enumerate() {
            assert_eq!(m.bus.read8(0x08000000 + at + n as u32), byte);
        }
    }
    let i = owner.color_mode_palettes_stats().unwrap();
    assert_eq!(
        [
            i.source_bytes,
            i.unit_base_bytes,
            i.companion_bytes,
            i.supplemental_bytes,
            i.shared_color_bytes,
            i.special_unit_bytes,
            i.selector_bytes,
            i.fragments_used
        ],
        [5750, 1536, 1024, 2286, 36, 864, 4, 30]
    );
    let old = owner.ui_palette_stats().unwrap().source_bytes;
    assert_eq!(
        m.bus.read32(0x0840fcb0),
        u32::from_le_bytes(rom[0x40fcb0..0x40fcb4].try_into().unwrap())
    );
    assert_eq!(owner.ui_palette_stats().unwrap().source_bytes - old, 4);
    assert_eq!(
        owner.color_mode_palettes_stats().unwrap().source_bytes,
        i.source_bytes
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,58,57],"files":54,"resources":53,"source_ranges":30,"source_bytes":5750,"all52prior_decoded_and_e57_retained":true,"guards":guards,"actual_metadata14_dependency":true,"failed_attach_owner_state_transactional":true,"publication_failure_create_reuse_repair_move":true,"unavailable_load_undo_backup_restart_sharedowner":true,"typed_unit_companion_supplemental_shared_special_selector_bytes":[1536,1024,2286,36,864,4],"fragments_used":30,"prior_UI_samepage_typed_bytes":4,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}

#[derive(Clone, Debug)]
struct Probe {
    label: &'static str,
    start: u32,
    end: u32,
    mode: u8,
    registers: Vec<(usize, u32)>,
    special_id: Option<u8>,
    ram_palette: bool,
    positive: bool,
}
impl Probe {
    fn setup(&self, m: &mut Machine) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        let code = m.bus.rom[0xd968a8..0xd968e6].to_vec();
        m.bus.iwram[0x5ee8..0x5f26].copy_from_slice(&code);
        m.bus.ewram[0x2fc2] = self.mode;
        m.cpu.regs = [0; 16];
        for n in 4..12 {
            m.cpu.regs[n] = 0x14104000 + n as u32 * 0x101;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.start;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        for &(n, value) in &self.registers {
            m.cpu.regs[n] = value;
        }
        if let Some(id) = self.special_id {
            m.bus.ewram[0x10004] = id;
            m.bus.write32(0x03002818, 0x02012000);
        }
        if self.ram_palette {
            let bytes: Vec<_> = (0..32).map(|n| m.bus.read8(0x0840fa90 + n)).collect();
            m.bus.ewram[0x18000..0x18020].copy_from_slice(&bytes);
            m.bus.ewram[0x18000] ^= 0x21;
            m.bus.ewram[0x18001] ^= 0x80;
        }
    }
}
fn probe(
    label: &'static str,
    start: u32,
    end: u32,
    mode: u8,
    registers: &[(usize, u32)],
    positive: bool,
) -> Probe {
    Probe {
        label,
        start: 0x08000000 + start,
        end: 0x08000000 + end,
        mode,
        registers: registers.to_vec(),
        special_id: None,
        ram_palette: false,
        positive,
    }
}
fn original(m: &mut Machine, p: &Probe, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m);
    let mut n = 0;
    while m.cpu.regs[15] != p.end {
        assert!(n < 20000, "{} PC{:08x}", p.label, m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
#[test]
#[ignore = "private EUR e58 Color-Mode runtime/native/state-clock; Wine only, Linux/USA pending"]
fn european_color_mode_palettes_native_copies_and_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.color_mode_palettes.as_ref().unwrap();
    let mut cases = Vec::new();
    for mode in [0, 1, 2] {
        cases.push(probe(
            "base shadows",
            0x23b44,
            0x23b70,
            mode,
            &[(4, 0x03003a60), (6, 0x08370eb4)],
            true,
        ));
        cases.push(probe(
            "shared colors",
            0xa95e2,
            0xa95f4,
            mode,
            &[(4, 0x08370eb4)],
            true,
        ));
        let slice = match mode {
            0 => 0x9dd40,
            1 => 0xba1da,
            _ => 0xba1e8,
        };
        cases.push(probe("slice selector", slice, 0x23e04, mode, &[], true));
        let (id, arg) = match mode {
            0 => (28, 1),
            1 => (77, 0),
            _ => (81, 1),
        };
        for alternate in [false, true] {
            let (start, end) = if alternate {
                (0x73c32, if arg == 1 { 0x73c7c } else { 0x73cc0 })
            } else {
                (0xcf09e, if arg == 1 { 0xcf0e0 } else { 0xcf11c })
            };
            let mut p = probe(
                "special override",
                start,
                end,
                mode,
                &[(4, 0x02010000), (5, arg)],
                true,
            );
            p.special_id = Some(id);
            cases.push(p);
        }
    }
    cases.extend([
        probe(
            "supplemental six bytes",
            0x285d0,
            0x285e4,
            0,
            &[(4, 0x03003a60), (5, 0x08370eb4)],
            true,
        ),
        probe(
            "supplemental one color",
            0x2ded4,
            0x2dee8,
            1,
            &[(4, 0x03003a60), (5, 0x08370eb4)],
            true,
        ),
        probe("supplemental CpuSet", 0x12ee5a, 0x12ee6a, 2, &[], true),
        probe("staged palette", 0x9325e, 0x14b744, 0, &[(4, 0x160)], true),
        probe(
            "staged alternate",
            0x94818,
            0x14b744,
            2,
            &[(4, 0x160)],
            true,
        ),
        probe("raw low8 pointer", 0xcf51c, 0x300, 3, &[(0, 65535)], false),
        probe(
            "selector byte",
            0xcf064,
            0xcf066,
            0,
            &[(0, 0x08394e04)],
            true,
        ),
        probe(
            "color halfword",
            0xcf06c,
            0xcf06e,
            0,
            &[(0, 0x0840f690)],
            true,
        ),
        probe(
            "shared color word",
            0x2217e,
            0x22180,
            0,
            &[(0, 0x084112f0)],
            true,
        ),
        probe(
            "mixed color raw word",
            0x2217e,
            0x22180,
            0,
            &[(0, 0x0840fc9a)],
            true,
        ),
        probe(
            "raw color gap",
            0x2217e,
            0x22180,
            0,
            &[(0, 0x0840fc9c)],
            false,
        ),
        probe(
            "prior UI same page",
            0x2217e,
            0x22180,
            0,
            &[(0, 0x0840fcb0)],
            false,
        ),
    ]);
    let mut live = probe(
        "live modified RAM palette",
        0x14b6f0,
        0x14b744,
        0,
        &[(0, 0x160), (1, 16), (2, 64), (3, 0x02018000)],
        false,
    );
    live.ram_palette = true;
    cases.push(live);
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
        iwram: Some(&tracer.bus.iwram),
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
    let object = out.join("color-mode-palettes.o");
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
        }
        original(&mut legacy, p, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            sources(m, &files, &rom, true);
        }
        let before = owner.color_mode_palettes_stats().unwrap().source_bytes;
        original(&mut decoded, p, &mut BTreeSet::new());
        if p.positive {
            assert!(
                owner.color_mode_palettes_stats().unwrap().source_bytes > before,
                "positive{}",
                p.label
            );
        }
        p.setup(&mut native);
        STOP_PC.with(|v| v.set(p.end));
        let mut n = 0;
        while native.cpu.regs[15] != p.end {
            assert!(n < 20000, "native{} PC{:08x}", p.label, native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap_or_else(|| {
                panic!("missing native{} PC{:08x}", p.label, native.cpu.regs[15])
            })(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
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
                "complete state/clock{}",
                p.label
            );
        }
        if p.ram_palette {
            assert_eq!(
                &decoded.bus.iwram[0x4ee8..0x4eea],
                &decoded.bus.ewram[0x18000..0x18002]
            );
        }
    }
    let compiled = COMPILED.with(Cell::get);
    assert!(
        compiled.iter().all(|n| *n > 0),
        "all color/selector API categories{compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    sources(&mut decoded, &files, &rom, true);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in data.source_spans().unwrap() {
            for boundary in [at, at + bytes.len() as u32] {
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
    // A word crosses two owned color bytes and two independently edited raw bytes.
    let at = 0x40fc9ausize;
    decoded.bus.rom[at + 2..at + 4]
        .iter_mut()
        .for_each(|b| *b ^= 0x33);
    let before = owner.color_mode_palettes_stats().unwrap().source_bytes;
    let mut expected: [u8; 4] = rom[at..at + 4].try_into().unwrap();
    expected[2..].iter_mut().for_each(|b| *b ^= 0x33);
    assert_eq!(
        (RT_API.read32)(
            &mut decoded as *mut Machine as *mut std::ffi::c_void,
            0x08000000 + at as u32
        ),
        u32::from_le_bytes(expected)
    );
    assert_eq!(
        owner.color_mode_palettes_stats().unwrap().source_bytes - before,
        2
    );
    decoded.bus.rom[at + 2..at + 4].copy_from_slice(&rom[at + 2..at + 4]);
    sources(&mut decoded, &files, &rom, false);
    let mut mixed_swi = None;
    for (swi, src, count) in [(0xdf0b, 0x40fc9a, 13), (0xdf0c, 0x40f290, 8)] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        sources(&mut decoded, &files, &rom, true);
        let before = owner.color_mode_palettes_stats().unwrap().source_bytes;
        let old = owner.ui_palette_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        let delta = (
            owner.color_mode_palettes_stats().unwrap().source_bytes - before,
            owner.ui_palette_stats().unwrap().source_bytes - old,
        );
        assert!(delta.0 > 0);
        if src == 0x40fc9a {
            assert_eq!(delta, (2, 4));
            mixed_swi = Some(delta);
        }
        sources(&mut decoded, &files, &rom, false);
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x0a40fc9a);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 13);
    }
    sources(&mut decoded, &files, &rom, true);
    let before = owner.color_mode_palettes_stats().unwrap().source_bytes;
    let old = owner.ui_palette_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    let mixed_dma = (
        owner.color_mode_palettes_stats().unwrap().source_bytes - before,
        owner.ui_palette_stats().unwrap().source_bytes - old,
    );
    assert_eq!(mixed_dma, (2, 4));
    sources(&mut decoded, &files, &rom, false);
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,58,57],"original_interpreter_native_paths":cases.len(),"compiled_unit_companion_supplemental_shared_special_selector_bytes":compiled,"full_states_clocks_equal":true,"raw_low8_index_mode3_and_live_RAMpalette":true,"widths_mirrors_realBIOS_bus_CpuSet_FastSet_DMA":true,"mixed_raw_API_new_bytes":2,"mixed_new_raw_UI_SWI_bytes":mixed_swi,"mixed_new_raw_UI_DMA_bytes":mixed_dma,"mixed_copy_raw_bytes":20,"instructions_coverage_only":instructions,"pcs":pcs.len(),"native_dispatches_coverage_only":dispatches,"scope":"typed Color-Mode palette original source-copy/staged-reader prefixes; no wholeallocator/scheduler or ordinary ABI claim","linux":"pending","usa":"pending","performance_measurements":false}),
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
    let i = resources.color_mode_palettes_stats().unwrap();
    [
        i.unit_base_bytes,
        i.companion_bytes,
        i.supplemental_bytes,
        i.shared_color_bytes,
        i.special_unit_bytes,
        i.selector_bytes,
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

fn names() -> [&'static str; 54] {
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
    ]
}
