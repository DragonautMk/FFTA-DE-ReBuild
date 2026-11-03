//! Typed EUR effect event directories: managed lifecycle and original native lookup with live RAM.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_EFFECT_EVENT_DIRECTORIES_MANAGED_ROOT").unwrap(),
    )
}
const MESSAGE: u32 = 0x02011000;

#[derive(Clone, Debug)]
enum Probe {
    EventHeader(u16),
    Delay(u16),
    Advance(u16),
    SelectorAddress(u16),
    SelectorKind(u16),
    Parameter(u16, u8),
    EventDispatch(u8),
    SelectorDispatch(u8),
    Frontdoor(u8),
    Low16(u32),
    TaskID(u16),
}
impl Probe {
    fn bounds(&self, d: &EffectEventDirectories) -> (u32, u32) {
        match *self {
            Self::EventHeader(_) => (0x08124eb8, 0x08124ee0),
            Self::Delay(_) => (0x081253ce, 0x081253da),
            Self::Advance(i) => (
                0x081253da,
                if i.wrapping_add(1) <= 74 {
                    0x081253e8
                } else {
                    0x081253f4
                },
            ),
            Self::SelectorAddress(_) => (0x08128b08, 0x08128b0e),
            Self::SelectorKind(_) => (0x08128b2c, 0x08128b2e),
            Self::Parameter(_, n) => {
                let pc = match n {
                    0 => 0x08128b6e,
                    1 => 0x08128bc8,
                    _ => 0x08128e70,
                };
                (pc, pc + 2)
            }
            Self::EventDispatch(i) => (
                0x08124ee0,
                d.event_dispatch
                    .slots
                    .get(usize::from(i))
                    .map_or(0x08125398, |s| s.target.guest_word().unwrap()),
            ),
            Self::SelectorDispatch(i) => (
                0x08128b2e,
                d.selector_dispatch
                    .slots
                    .get(usize::from(i))
                    .map_or(0x08128eda, |s| s.target.guest_word().unwrap()),
            ),
            Self::Frontdoor(_) => (0x0812746e, 0x08127476),
            Self::Low16(_) => (0x08128b00, 0x08128b06),
            Self::TaskID(_) => (0x08128b20, 0x08128b2c),
        }
    }
    fn raw(&self) -> Option<(usize, usize)> {
        match *self {
            Self::EventHeader(75) => Some((0x3acebc, 16)),
            Self::SelectorKind(51) => Some((0x3ad240, 1)),
            Self::SelectorKind(65535) => Some((0x3ad174 + 65535 * 4, 1)),
            _ => None,
        }
    }
    fn font(&self) -> bool {
        matches!(self, Self::EventHeader(65535))
    }
    fn no_new_reads(&self) -> bool {
        matches!(
            self,
            Self::EventHeader(75 | 65535)
                | Self::Advance(_)
                | Self::SelectorAddress(_)
                | Self::SelectorKind(51 | 65535)
                | Self::EventDispatch(22)
                | Self::SelectorDispatch(6)
                | Self::Frontdoor(_)
                | Self::Low16(_)
                | Self::TaskID(_)
        )
    }
    fn setup(&self, m: &mut Machine, d: &EffectEventDirectories) {
        use gba_core::rom_data::effect_event_directories::selector_source_offset;
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 1..13 {
            m.cpu.regs[n] = 0x11110000 + n as u32;
        }
        m.cpu.regs[7] = MESSAGE;
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds(d).0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match *self {
            Self::EventHeader(i) | Self::Delay(i) | Self::Advance(i) => {
                m.bus.ewram[0x11000..0x11002].copy_from_slice(&i.to_le_bytes())
            }
            Self::SelectorAddress(i) => m.cpu.regs[0] = u32::from(i),
            Self::SelectorKind(i) | Self::Parameter(i, _) => {
                m.cpu.regs[5] = 0x08000000 + selector_source_offset(u32::from(i))
            }
            Self::EventDispatch(i) => m.cpu.regs[12] = u32::from(i),
            Self::SelectorDispatch(i) => m.cpu.regs[0] = u32::from(i),
            Self::Frontdoor(q) => {
                m.cpu.regs[1] = MESSAGE;
                m.bus.ewram[0x11001] = q;
            }
            Self::Low16(raw) => m.cpu.regs[0] = raw,
            Self::TaskID(i) => {
                m.bus.iwram[0x7e00..0x7e04].copy_from_slice(&u32::from(i).to_le_bytes())
            }
        }
    }
}
fn original(
    m: &mut Machine,
    p: &Probe,
    d: &EffectEventDirectories,
    pcs: &mut BTreeSet<u32>,
) -> usize {
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
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .effect_event_directories_stats()
            .unwrap()
    };
    [s.event_bytes, s.selector_bytes, s.dispatch_bytes]
}
fn count(before: [u64; 3], after: [u64; 3]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..3 {
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
#[ignore = "private EUR e49 effect event fields/even native dispatch/fullstate-clock; Wine only, Linux/USA pending"]
fn european_effect_event_directories_native_fields_and_raw_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.effect_event_directories.as_ref().unwrap();
    let owns = |at: usize, n: usize| {
        files
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| at < (s.offset + s.length) as usize && (s.offset as usize) < at + n)
    };
    let multi = data.selectors.iter().position(|s| s.kind == 4).unwrap() as u16;
    let mut cases: Vec<_> = [0, 74, 75, 65535]
        .into_iter()
        .map(Probe::EventHeader)
        .collect();
    cases.extend([
        Probe::Delay(0),
        Probe::Delay(74),
        Probe::Advance(74),
        Probe::Advance(65535),
        Probe::SelectorAddress(65535),
    ]);
    cases.extend(
        [0, multi, 50, 51, 65535]
            .into_iter()
            .map(Probe::SelectorKind),
    );
    cases.extend((0..3).map(|n| Probe::Parameter(multi, n)));
    cases.extend([0, 12, 21, 22].into_iter().map(Probe::EventDispatch));
    cases.extend([0, 4, 5, 6].into_iter().map(Probe::SelectorDispatch));
    cases.extend([
        Probe::Frontdoor(0),
        Probe::Frontdoor(255),
        Probe::Low16(0xbeef0001),
        Probe::TaskID(65535),
    ]);
    for p in &cases {
        if let Some((a, n)) = p.raw() {
            assert!(!owns(a, n), "raw outside every owner {p:?}");
        }
    }
    let font_at = 0x3aca0c + 65535 * 16;
    assert!(files
        .manifest
        .resources
        .iter()
        .find(|r| r.id == FONT_BANK_ID)
        .unwrap()
        .sources
        .iter()
        .any(|s| s.offset as usize <= font_at && font_at + 16 <= (s.offset + s.length) as usize));
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
    let object = out.join("effect-event-directories.o");
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
            if let Some((a, n)) = p.raw() {
                for v in &mut m.bus.rom[a..a + n] {
                    *v ^= 0x5a;
                }
            }
        }
        original(&mut legacy, p, data, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
            if p.font() {
                for v in &mut m.bus.rom[font_at..font_at + 16] {
                    *v ^= 0x5a;
                }
            }
        }
        let before = owner.effect_event_directories_stats().unwrap().source_bytes;
        let old_before = owner.font_keyboard_stats().unwrap().glyph_bytes;
        original(&mut decoded, p, data, &mut BTreeSet::new());
        p.setup(&mut native, data);
        STOP_PC.with(|v| v.set(p.bounds(data).1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds(data).1 {
            assert!(n < 100, "native {p:?} pc{:08x}", native.cpu.regs[15]);
            native.bus.frame_ready = false;
            table.get(native.cpu.regs[15] | 1).unwrap_or_else(|| {
                panic!("missing native block {p:?} pc{:08x}", native.cpu.regs[15])
            })(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
            n += 1;
            dispatches += 1;
        }
        assert_eq!(
            owner.effect_event_directories_stats().unwrap().source_bytes == before,
            p.no_new_reads(),
            "{p:?}"
        );
        if p.font() {
            assert_eq!(
                owner.font_keyboard_stats().unwrap().glyph_bytes - old_before,
                11
            );
        }
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
            if p.font() {
                m.bus.rom[font_at..font_at + 16].copy_from_slice(&rom[font_at..font_at + 16]);
            }
        }
        for m in [&mut legacy, &mut decoded, &mut native] {
            if let Some((a, n)) = p.raw() {
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
        "all three classes native API {compiled:?}"
    );
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    for bios in [false, true] {
        decoded.bus.real_bios = bios;
        for (at, bytes, _) in data.source_spans().unwrap() {
            for offset in
                (at - 3..at + 4).chain(at + bytes.len() as u32 - 3..at + bytes.len() as u32 + 4)
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
    }
    decoded.bus.real_bios = false;
    // The added3acpage must retain a different typed owner on the same page.
    let palette_at = 0x3ac748;
    assert!(files
        .manifest
        .resources
        .iter()
        .find(|r| r.id == PALETTE_PROGRAMS_ID)
        .unwrap()
        .sources
        .iter()
        .any(|s| s.offset <= palette_at && palette_at + 4 <= s.offset + s.length));
    for v in &mut decoded.bus.rom[palette_at as usize..palette_at as usize + 4] {
        *v ^= 0x5a;
    }
    let before = owner.effect_event_directories_stats().unwrap().source_bytes;
    let old_before = owner.palette_programs_stats().unwrap().source_bytes;
    assert_eq!(
        (RT_API.read32)(
            &mut decoded as *mut Machine as *mut std::ffi::c_void,
            0x08000000 + palette_at
        ),
        u32::from_le_bytes(
            rom[palette_at as usize..palette_at as usize + 4]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        owner.effect_event_directories_stats().unwrap().source_bytes,
        before
    );
    assert_eq!(
        owner.palette_programs_stats().unwrap().source_bytes - old_before,
        4
    );
    decoded.bus.rom[palette_at as usize..palette_at as usize + 4]
        .copy_from_slice(&rom[palette_at as usize..palette_at as usize + 4]);
    let end = 0x3ad240;
    assert!(!owns(end as usize, 2));
    let before = owner.effect_event_directories_stats().unwrap().source_bytes;
    assert_eq!(
        decoded.bus.read32(0x08000000 + end - 2),
        u32::from_le_bytes(rom[end as usize - 2..end as usize + 2].try_into().unwrap())
    );
    assert_eq!(
        owner.effect_event_directories_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, 0x3aca0c, 8),
        (0xdf0c, 0x128b50, 8),
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
        let before = owner.effect_event_directories_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.effect_event_directories_stats().unwrap().source_bytes > before);
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
    let before = owner.effect_event_directories_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(
        owner.effect_event_directories_stats().unwrap().source_bytes - before,
        2
    );
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,49,48],"original_interpreter_native_paths":cases.len(),"compiled_event_selector_dispatch_bytes":compiled,"poisoned_source_bytes":1516,"full_states_clocks_equal":true,"raw_event75_selector51_65535_q0_wrap_defaults_even_dispatch":true,"event65535_aliases_previous_font_owner_bytes":11,"samepage_previous_palette_owner_bytes":4,"four_spans_edges_widths_mirrors_bios_cpuset_fastset_dma_new2_raw2":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"scope":"controlled field/dispatch/raw/low16 prefixes, not whole event interpreter/allocator/scheduler ABI or ordinary reachability","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 44] {
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
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &EffectEventDirectories, rom: Option<&[u8]>) {
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
#[ignore = "private EUR e49 effect event directory publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_effect_event_directories_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..43]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e48").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (48, 47));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (49, 48)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 44);
    assert_eq!(source.manifest.resources.len(), 43);
    for i in 1..43 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_effect_event_directories(&files[43]).unwrap();
    assert_eq!(
        data.source_spans()
            .unwrap()
            .iter()
            .map(|s| s.1.len())
            .sum::<usize>(),
        1516
    );
    assert_eq!(
        files[43],
        fs::read(std::env::var_os("FFTA_STATE_TEST_EFFECT_EVENT_DIRECTORIES_SOURCE_FILE").unwrap())
            .unwrap()
    );
    assert_eq!(source.manifest.resources[42].sources.len(), 4);
    assert_eq!(
        source.manifest.resources[42].dependencies,
        vec![TASK_DESCRIPTORS_ID.to_string()]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| if p
        .ends_with(EFFECT_EVENT_DIRECTORIES_FILE)
    {
        Err("injected effect event directory publication failure".into())
    } else {
        write_synced(p, b)
    })
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e48-d47/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e49-d48"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(
        prepared.directory.join(EFFECT_EVENT_DIRECTORIES_FILE),
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
    let mut missing = Extraction::from_rom(&rom).unwrap();
    missing.task_descriptors = None;
    assert_eq!(
        missing.manifest.resources.remove(41).id,
        TASK_DESCRIPTORS_ID
    );
    missing
        .manifest
        .coverage
        .retain(|id| id != TASK_DESCRIPTORS_ID);
    assert_eq!(
        missing.validate_files(fileset.files()),
        Err("Extraction resource set is inconsistent".into())
    );
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    assert_eq!(data.events.len(), 75);
    for case in 0..12 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.events[0].post_delay ^= 1,
            1 => forged.selectors[0].parameters[0] ^= 1,
            2 => forged.event_dispatch.slots[0].target.rom_offset += 2,
            3 => manifest.resources[42].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[42].sources[0].length -= 4,
            5 => manifest.resources[42].sources[0].offset += 4,
            6 => manifest.resources[42].sources[0].role = "graphics/unknown".into(),
            7 => manifest.resources[42].path = "../effect-event-directories.json".into(),
            8 => manifest.resources[42].dependencies.clear(),
            9 => manifest.resources[42].sha256 = "0".repeat(64),
            10 => manifest
                .coverage
                .retain(|id| id != EFFECT_EVENT_DIRECTORIES_ID),
            _ => {
                manifest.resources[42].sources.clear();
            }
        }
        let bytes = encode_effect_event_directories(&forged).unwrap();
        if case != 9 {
            manifest.resources[42].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(EFFECT_EVENT_DIRECTORIES_FILE), bytes).unwrap();
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
    fs::write(dataset().join(EFFECT_EVENT_DIRECTORIES_FILE), &files[43]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(
        dataset().join(EFFECT_EVENT_DIRECTORIES_FILE),
        r.join("missing.json"),
    )
    .unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(
        r.join("missing.json"),
        dataset().join(EFFECT_EVENT_DIRECTORIES_FILE),
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
    let stats = owner.effect_event_directories_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.event_bytes,
            stats.selector_bytes,
            stats.dispatch_bytes,
            stats.event_records_used,
            stats.selectors_used,
            stats.dispatch_tables_used
        ),
        (1516, 1200, 204, 112, 75, 51, 2)
    );
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,49,48],"files":44,"resources":43,"source_ranges":4,"source_bytes":1516,"all42prior_decoded_files_and_e48_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":13,"preceding_owner_missing_rejected_before_files":true,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_categories_75events51selectors2dispatch":true,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
