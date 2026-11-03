//! Typed EUR palette programs: managed lifecycle and original native lookup with live RAM.
use super::*;
use gba_core::{capi::RT_API, rom_data::palette_programs::*, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PALETTE_PROGRAMS_MANAGED_ROOT").unwrap())
}
const CONTEXT: u32 = 0x02010000;
const MESSAGE: u32 = 0x02011000;
const OBJECT: u32 = 0x02012000;
#[derive(Clone, Debug)]
enum Probe {
    Header(u16),
    Count(u16),
    Command(u16, u8),
    Color(u16, u8, u16),
    Indexed(u8),
    RawCommand,
}
impl Probe {
    fn bounds(&self, d: &PalettePrograms) -> (u32, u32) {
        match *self {
            Self::Header(_) => (0x080d7fac, 0x080d80d8),
            Self::Count(_) => (0x080d80b2, 0x080d80b4),
            Self::Command(p, c) => {
                let cmd = &d.programs[usize::from(p)].commands[usize::from(c)];
                let queued = cmd.mode != 0 || matches!(cmd.opcode, 10 | 11);
                (
                    if queued { 0x080d7b48 } else { 0x080d7d34 },
                    if queued {
                        match cmd.opcode {
                            0 => 0x0814b744,
                            1 => 0x080d7bd0,
                            4 => 0x080d7c52,
                            7 => 0x080d7cbc,
                            8 => 0x080d7cd0,
                            9 => 0x080d7ce4,
                            10 => 0x080d7d02,
                            11 => 0x080d7d24,
                            _ => unreachable!(),
                        }
                    } else {
                        match cmd.opcode {
                            0 | 13 => 0x080d7f46,
                            4 => 0x080d7e42,
                            12 => 0x080d7f38,
                            _ => unreachable!(),
                        }
                    },
                )
            }
            Self::Color(..) => (0x0814a574, 0x0814a584),
            Self::Indexed(0) => (0x080e57d8, 0x080e57e2),
            Self::Indexed(1 | 2) => (0x080e8404, 0x080e840e),
            Self::Indexed(_) => (0x08109038, 0x0810904c),
            Self::RawCommand => (0x080d7b48, 0x080d7b6a),
        }
    }
    fn raw(&self, d: &PalettePrograms) -> Option<(usize, usize)> {
        match *self {
            Self::RawCommand => Some((d.programs[0].source_offset as usize + 6 + 6 * 65535, 6)),
            Self::Color(p, c, 65534) => Some((
                d.programs[usize::from(p)].source_offset as usize
                    + usize::from(
                        d.programs[usize::from(p)].commands[usize::from(c)].relative_parameter,
                    )
                    + 2
                    + 65534,
                2,
            )),
            Self::Indexed(2) => Some((0x39bccc, 4)),
            _ => None,
        }
    }
    fn setup(&self, m: &mut Machine, d: &PalettePrograms) {
        m.bus.ewram.fill(0);
        m.bus.iwram.fill(0);
        let copy = m.bus.rom[0xd968a8..0xd968e6].to_vec();
        m.bus.iwram[0x5ee8..0x5f26].copy_from_slice(&copy);
        m.cpu.regs = [0; 16];
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds(d).0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        let put16 = |m: &mut Machine, a: u32, v: u16| {
            let n = (a - 0x02000000) as usize;
            m.bus.ewram[n..n + 2].copy_from_slice(&v.to_le_bytes());
        };
        let put32 = |m: &mut Machine, a: u32, v: u32| {
            let n = (a - 0x02000000) as usize;
            m.bus.ewram[n..n + 4].copy_from_slice(&v.to_le_bytes());
        };
        match *self {
            Self::Header(p) => {
                put32(
                    m,
                    MESSAGE + 4,
                    0x08000000 + d.programs[usize::from(p)].source_offset,
                );
                put16(m, MESSAGE + 20, 0x81);
                m.cpu.regs[1] = MESSAGE;
                m.cpu.regs[2] = CONTEXT;
            }
            Self::Count(p) => m.cpu.regs[1] = 0x08000000 + d.programs[usize::from(p)].source_offset,
            Self::Command(p, c) => {
                let p = &d.programs[usize::from(p)];
                put32(m, CONTEXT, 0x08000000 + p.source_offset);
                put16(m, CONTEXT + 4, p.header.first_color());
                m.bus.ewram[0x10006] = p.header.width;
                put16(m, CONTEXT + 10, u16::from(c));
                m.cpu.regs[0] = CONTEXT;
            }
            Self::Color(p, c, offset) => {
                let p = &d.programs[usize::from(p)];
                m.cpu.regs[0] = OBJECT;
                put32(
                    m,
                    OBJECT + 0x1c,
                    0x08000000
                        + p.source_offset
                        + u32::from(p.commands[usize::from(c)].relative_parameter)
                        + 2,
                );
                put16(m, OBJECT + 0x20, offset);
            }
            Self::Indexed(n) => {
                m.cpu.regs[5] = CONTEXT;
                m.cpu.regs[6] = CONTEXT;
                m.cpu.regs[9] = CONTEXT;
                match n {
                    0 => m.bus.ewram[0x10005] = 4,
                    1 => m.bus.ewram[0x10000] = 1,
                    2 => m.bus.ewram[0x10000] = 2,
                    _ => {
                        m.bus.ewram[0x10002] = 4;
                        m.cpu.regs[5] = 2;
                    }
                }
            }
            Self::RawCommand => {
                put32(m, CONTEXT, 0x08000000 + d.programs[0].source_offset);
                put16(m, CONTEXT + 10, 65535);
                m.cpu.regs[0] = CONTEXT;
            }
        }
    }
}
fn original(m: &mut Machine, p: &Probe, d: &PalettePrograms, pcs: &mut BTreeSet<u32>) -> usize {
    p.setup(m, d);
    let mut n = 0;
    while m.cpu.regs[15] != p.bounds(d).1 {
        assert!(n < 20000, "{p:?} pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
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
            .palette_programs_stats()
            .unwrap()
    };
    [
        s.header_bytes,
        s.command_bytes,
        s.parameter_bytes,
        s.producer_bytes,
    ]
}
fn count(before: [u64; 4], after: [u64; 4]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..4 {
            x[i] += after[i] - before[i];
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
#[test]
#[ignore = "private EUR e46 palette programs original/native typed fields and live raw inputs; Wine only, Linux/USA pending"]
fn european_palette_programs_native_fields_live_indices_and_raw_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.palette_programs.as_ref().unwrap();
    let mut cases = vec![
        Probe::Header(0),
        Probe::Header(185),
        Probe::Header(229),
        Probe::Count(0),
    ];
    for (op, mode) in [
        (0, 0),
        (0, 1),
        (1, 1),
        (4, 0),
        (4, 1),
        (7, 1),
        (8, 1),
        (9, 1),
        (10, 0),
        (11, 0),
        (12, 0),
        (13, 0),
    ] {
        let (p, c) = data
            .programs
            .iter()
            .enumerate()
            .find_map(|(p, x)| {
                x.commands
                    .iter()
                    .position(|c| c.opcode == op && c.mode == mode)
                    .map(|c| (p, c))
            })
            .unwrap();
        cases.push(Probe::Command(p as u16, c as u8));
    }
    let (p, c, last) = data
        .programs
        .iter()
        .enumerate()
        .find_map(|(p, x)| {
            x.commands
                .iter()
                .enumerate()
                .find_map(|(c, x)| match &x.parameter {
                    Parameter::ColorCycle { colors, .. } => {
                        Some((p as u16, c as u8, (2 * (colors.len() - 1)) as u16))
                    }
                    _ => None,
                })
        })
        .unwrap();
    cases.extend([
        Probe::Color(p, c, 0),
        Probe::Color(p, c, last),
        Probe::Color(p, c, 65534),
    ]);
    cases.extend((0..4).map(Probe::Indexed));
    cases.push(Probe::RawCommand);
    for probe in &cases {
        if let Some((at, len)) = probe.raw(data) {
            assert!(
                !files
                    .manifest
                    .resources
                    .iter()
                    .flat_map(|r| &r.sources)
                    .any(|s| (at..at + len)
                        .any(|a| s.offset as usize <= a && a < (s.offset + s.length) as usize)),
                "raw probe must be outside every typed owner"
            );
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
    let iwram = tracer.bus.iwram.clone();
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&iwram),
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
    let object = out.join("palette-programs.o");
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
            if let Some((a, n)) = p.raw(data) {
                for b in &mut m.bus.rom[a..a + n] {
                    *b ^= 0x5a;
                }
            }
        }
        original(&mut legacy, p, data, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
        }
        let before = owner.palette_programs_stats().unwrap().source_bytes;
        original(&mut decoded, p, data, &mut BTreeSet::new());
        p.setup(&mut native, data);
        STOP_PC.with(|stop| stop.set(p.bounds(data).1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds(data).1 {
            assert!(n < 20000, "native {p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap_or_else(|| {
                panic!(
                    "missing native block {p:?} pc={:08x} cpsr={:08x} traced={}",
                    native.cpu.regs[15],
                    native.cpu.cpsr,
                    pcs.contains(&native.cpu.regs[15])
                )
            })(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
            n += 1;
            dispatches += 1;
        }
        let bytes = owner.palette_programs_stats().unwrap().source_bytes - before;
        assert_eq!(
            bytes == 0,
            matches!(
                p,
                Probe::RawCommand | Probe::Color(_, _, 65534) | Probe::Indexed(2)
            ),
            "{p:?}"
        );
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            if let Some((a, n)) = p.raw(data) {
                m.bus.rom[a..a + n].copy_from_slice(&rom[a..a + n]);
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
        "all4field classes must be read through generated native APIs: {compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    let selected = [
        (0x14d680, 24),
        (0x39bcc4, 8),
        (data.programs[0].source_offset, 6),
    ];
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (first, len) in selected {
            for offset in (first - 3..first + 4).chain(first + len - 3..first + len + 4) {
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
    }
    decoded.bus.real_bios = false;
    let first = 0x14d680;
    let end = first + 24;
    for at in [first - 2, end - 2] {
        let before = owner.palette_programs_stats().unwrap().source_bytes;
        assert_eq!(
            decoded.bus.read32(0x08000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
        assert_eq!(
            owner.palette_programs_stats().unwrap().source_bytes - before,
            2
        );
    }
    range(&mut decoded, data, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, first - 2, 4),
        (0xdf0c, first, 8),
        (0xdf0b, end - 2, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let before = owner.palette_programs_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.palette_programs_stats().unwrap().source_bytes > before);
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
    let before = owner.palette_programs_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.palette_programs_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,46,45],"original_interpreter_native_paths":cases.len(),"compiled_header_command_parameter_producer_bytes":compiled,"poisoned_source_bytes":18697,"full_states_clocks_equal":true,"live_command65535_coloroffset65534_index2_raw":true,"edges_widths_mirrors_realbios_cpuset_fastset_dma_two_typed_two_raw_crossing":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"scope":"header-command-parameter-producer/copy prefixes, not fullallocator/callback/scheduler ABI","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 41] {
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
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &PalettePrograms, rom: Option<&[u8]>) {
    for (at, bytes, _) in data.source_spans().unwrap() {
        let at = at as usize;
        let dst = &mut m.bus.rom[at..at + bytes.len()];
        if let Some(r) = rom {
            dst.copy_from_slice(&r[at..at + bytes.len()]);
        } else {
            for b in dst {
                *b ^= 0x5a;
            }
        }
    }
}
#[test]
#[ignore = "private EUR e46 palette program publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_palette_programs_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..40]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e45").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (45, 44));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (46, 45)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 41);
    assert_eq!(source.manifest.resources.len(), 40);
    for i in 1..40 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_palette_programs(&files[40]).unwrap();
    assert_eq!(
        data.source_spans()
            .unwrap()
            .iter()
            .map(|s| s.1.len())
            .sum::<usize>(),
        18697
    );
    assert_eq!(
        files[40],
        fs::read(std::env::var_os("FFTA_STATE_TEST_PALETTE_PROGRAMS_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert_eq!(source.manifest.resources[39].sources.len(), 165);
    assert_eq!(
        source.manifest.resources[39].dependencies,
        Vec::<String>::new()
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(PALETTE_PROGRAMS_FILE)
    {
        Err("injected palette program publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e45-d44/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e46-d45"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(PALETTE_PROGRAMS_FILE), b"partial").unwrap();
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
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    assert_eq!(data.programs.len(), 230);
    for case in 0..12 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.programs[0].header.initial_delay ^= 1,
            1 => forged.programs[0].commands[0].opcode = 255,
            2 => {
                if let ProducerData::List { entries } = &mut forged.producers[0].data {
                    entries[0].program = 65535;
                }
            }
            3 => manifest.resources[39].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[39].sources[0].length -= 4,
            5 => manifest.resources[39].sources[0].offset += 4,
            6 => manifest.resources[39].sources[0].role = "graphics/unknown".into(),
            7 => manifest.resources[39].path = "../palette-programs.json".into(),
            8 => manifest.resources[39].dependencies = vec![PALETTE_EFFECT_DISPATCH_ID.into()],
            9 => manifest.resources[39].sha256 = "0".repeat(64),
            10 => manifest.coverage.retain(|id| id != PALETTE_PROGRAMS_ID),
            _ => {
                manifest.resources[39].sources.clear();
            }
        }
        let bytes = encode_palette_programs(&forged).unwrap();
        if case != 9 {
            manifest.resources[39].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(PALETTE_PROGRAMS_FILE), bytes).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "guard {case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), boot);
    }
    fs::write(dataset().join(PALETTE_PROGRAMS_FILE), &files[40]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(PALETTE_PROGRAMS_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(
        r.join("missing.json"),
        dataset().join(PALETTE_PROGRAMS_FILE),
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
    for (at, bytes, _) in data.source_spans().unwrap() {
        for i in 0..bytes.len() {
            assert_eq!(
                m.bus.read8(0x08000000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    let stats = owner.palette_programs_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.header_bytes,
            stats.command_bytes,
            stats.parameter_bytes,
            stats.producer_bytes,
            stats.programs_used,
            stats.producer_tables_used
        ),
        (18697, 1380, 6810, 10031, 476, 230, 25)
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,46,45],"files":41,"resources":40,"source_ranges":165,"source_bytes":18697,"all39prior_decoded_files_and_e45_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":12,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_categories_230programs25tables":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
