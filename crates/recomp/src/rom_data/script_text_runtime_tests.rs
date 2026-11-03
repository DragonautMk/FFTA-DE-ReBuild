//! Typed EUR script archive: managed lifecycle and original native lookup with live RAM.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_MANAGED_ROOT").unwrap())
}
const MESSAGE: u32 = 0x02011000;

use gba_core::rom_data::script_text as model;
use serde_json::Value;

#[derive(Clone, Debug)]
enum Probe {
    Byte(u32),
    Top(u8, u8),
    Variant(u8, u8),
    Index(u8, u32),
    Header(u32),
}
impl Probe {
    fn bounds(&self, _: &ScriptText) -> (u32, u32) {
        match self {
            Self::Byte(_) => (0x0801414e, 0x08014150),
            Self::Top(..) => (0x08009b2a, 0x08009b36),
            Self::Variant(..) => (0x08009b58, 0x08009b66),
            Self::Index(..) => (0x08013f96, 0x08013fa0),
            Self::Header(_) => (0x08013fa0, 0x08013fc6),
        }
    }
    fn setup(&self, m: &mut Machine, d: &ScriptText) {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0xa5);
        m.cpu.regs = [0; 16];
        for n in 1..13 {
            m.cpu.regs[n] = 0x11110000 + n as u32;
        }
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = 0x08000301;
        m.cpu.regs[15] = self.bounds(d).0;
        m.cpu.cpsr = 0xbf;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        match *self {
            Self::Byte(at) => m.cpu.regs[5] = 0x08000000 + at,
            Self::Top(li, bank) => {
                m.bus.write8(0x02001f6e, li);
                m.cpu.regs[0] = 0x08000000 + model::ROOTS[usize::from(li)];
                m.cpu.regs[1] = u32::from(bank);
            }
            Self::Variant(li, q) => {
                m.bus.write8(0x02001f6e, li);
                m.cpu.regs[0] = u32::from(q);
                m.cpu.regs[4] = 0x08000000 + d.languages[usize::from(li)].variants.source_offset;
            }
            Self::Index(li, index) => {
                m.cpu.regs[1] = 0x08000000 + d.languages[usize::from(li)].banks[0].source_offset;
                m.cpu.regs[3] = index;
            }
            Self::Header(at) => {
                m.cpu.regs[2] = 0x08000000 + at;
                m.cpu.regs[4] = MESSAGE;
                m.cpu.regs[5] = MESSAGE + 4;
            }
        }
    }
}
fn original(m: &mut Machine, p: &Probe, d: &ScriptText, pcs: &mut BTreeSet<u32>) -> usize {
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
    static COMPILED:Cell<[u64;11]>=const{Cell::new([0;11])};
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
fn usage(m: *mut std::ffi::c_void) -> [u64; 11] {
    let s = unsafe {
        (&*(m as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .script_text_stats()
            .unwrap()
    };
    [
        s.top_index_bytes,
        s.variant_index_bytes,
        s.bank_index_bytes,
        s.header_bytes,
        s.encoded_control_bytes,
        s.glyph_bytes,
        s.command_bytes,
        s.operand_bytes,
        s.terminator_bytes,
        s.decoded_tail_bytes,
        s.alignment_bytes,
    ]
}
fn count(before: [u64; 11], after: [u64; 11]) {
    COMPILED.with(|v| {
        let mut x = v.get();
        for i in 0..11 {
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
#[ignore = "private EUR e52localized text native fullstate-clock field/variant/boundary checks; Wine only, Linux/USA pending"]
fn european_script_text_native_fields_and_shared_boundaries() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let data = files.script_text.as_ref().unwrap();
    let field: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_FIELDS").unwrap()).unwrap(),
    )
    .unwrap();
    let mut cases: Vec<_> = field["representatives"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["offset"].as_u64().map(|a| Probe::Byte(a as u32)))
        .collect();
    cases.extend((0u8..5).map(|li| Probe::Top(li, 1)));
    cases.push(Probe::Top(0, 255));
    for li in [0, 4] {
        for q in [0, 25, 255] {
            cases.push(Probe::Variant(li, q));
        }
    }
    for li in [0, 4] {
        let n = data.languages[usize::from(li)].banks[0].programs.len() as u32;
        for index in [0, n - 1, n, 65535, 0x10000] {
            cases.push(Probe::Index(li, index));
        }
    }
    let mut forms = BTreeSet::new();
    for program in data
        .languages
        .iter()
        .flat_map(|l| &l.banks)
        .flat_map(|b| &b.programs)
    {
        if forms.insert(program.header & 3) {
            cases.push(Probe::Header(program.source_offset));
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
    let blocks = pcs
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
    let object = out.join("script-text.o");
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
    attach_from_directory(&mut decoded, &dataset()).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    native.bus.set_rom_resources(Some(owner.clone())).unwrap();
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        guard: prefix_guard,
        ..RT_API
    };
    COMPILED.with(|v| v.set([0; 11]));
    let mut dispatches = 0;
    for p in &cases {
        for m in [&mut legacy, &mut decoded, &mut native] {
            m.load_state(&start).unwrap();
        }
        original(&mut legacy, p, data, &mut BTreeSet::new());
        for m in [&mut decoded, &mut native] {
            range(m, data, None);
        }
        let before = owner.script_text_stats().unwrap().source_bytes;
        original(&mut decoded, p, data, &mut BTreeSet::new());
        assert!(
            owner.script_text_stats().unwrap().source_bytes > before,
            "new source read {p:?}"
        );
        p.setup(&mut native, data);
        STOP_PC.with(|v| v.set(p.bounds(data).1));
        let mut n = 0;
        while native.cpu.regs[15] != p.bounds(data).1 {
            assert!(n < 100, "native {p:?}");
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
        for m in [&mut decoded, &mut native] {
            range(m, data, Some(&rom));
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
    for (i, expected) in field["category_counts"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(
            compiled[i] > 0,
            expected.as_u64().unwrap() > 0,
            "source class{i}"
        );
    }
    decoded.load_state(&start).unwrap();
    range(&mut decoded, data, None);
    for real in [false, true] {
        decoded.bus.real_bios = real;
        for (at, bytes, _) in data.source_spans().unwrap() {
            let end = at + bytes.len() as u32;
            for offset in (at - 3..at + 4).chain(end - 3..end + 4) {
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
    for (at, oldstart, newbytes) in [
        (model::ROOTS[0] - 2, model::ROOTS[0] - 2, 2),
        (model::END - 2, model::END, 2),
    ] {
        for v in &mut decoded.bus.rom[oldstart as usize..oldstart as usize + 2] {
            *v ^= 0x5a;
        }
        let b = owner.script_text_stats().unwrap().source_bytes;
        let old = owner.script_archive_stats().unwrap().source_bytes;
        assert_eq!(
            (RT_API.read32)(
                &mut decoded as *mut Machine as *mut std::ffi::c_void,
                0x08000000 + at
            ),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
        assert_eq!(
            owner.script_text_stats().unwrap().source_bytes - b,
            newbytes
        );
        assert_eq!(owner.script_archive_stats().unwrap().source_bytes - old, 2);
        decoded.bus.rom[oldstart as usize..oldstart as usize + 2]
            .copy_from_slice(&rom[oldstart as usize..oldstart as usize + 2]);
    }
    // Raw fallback follows the adjacent old key owner on the same D68page.
    let raw = gba_core::rom_data::script_archive::KEY_OFFSET
        + gba_core::rom_data::script_archive::KEY_COUNT as u32 * 4;
    for v in &mut decoded.bus.rom[raw as usize..raw as usize + 2] {
        *v ^= 0x5a;
    }
    let before = owner.script_text_stats().unwrap().source_bytes;
    let old = owner.script_archive_stats().unwrap().source_bytes;
    let mut expected: [u8; 4] = rom[raw as usize - 2..raw as usize + 2].try_into().unwrap();
    for v in &mut expected[2..] {
        *v ^= 0x5a;
    }
    assert_eq!(
        decoded.bus.read32(0x08000000 + raw - 2),
        u32::from_le_bytes(expected)
    );
    assert_eq!(owner.script_text_stats().unwrap().source_bytes, before);
    assert_eq!(owner.script_archive_stats().unwrap().source_bytes - old, 2);
    decoded.bus.rom[raw as usize..raw as usize + 2]
        .copy_from_slice(&rom[raw as usize..raw as usize + 2]);
    range(&mut decoded, data, Some(&rom));
    for (swi, src, count) in [
        (0xdf0b, model::ROOTS[0] - 2, 3),
        (0xdf0c, model::ROOTS[0], 8),
        (0xdf0b, model::END - 2, 3),
    ] {
        for m in [&mut legacy, &mut decoded] {
            m.load_state(&start).unwrap();
            m.cpu.regs[..3].copy_from_slice(&[0x0a000000 + src, 0x02032400, count]);
            m.cpu.regs[15] = 0x02000000;
            m.cpu.cpsr = 0xbf;
            m.bus.write16(0x02000000, swi);
        }
        range(&mut decoded, data, None);
        let before = owner.script_text_stats().unwrap().source_bytes;
        legacy.step();
        decoded.step();
        assert!(owner.script_text_stats().unwrap().source_bytes > before);
        range(&mut decoded, data, Some(&rom));
        assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    }
    for m in [&mut legacy, &mut decoded] {
        m.load_state(&start).unwrap();
        m.bus.write32(0x040000d4, 0x08000000 + model::END - 2);
        m.bus.write32(0x040000d8, 0x02032000);
        m.bus.write16(0x040000dc, 2);
    }
    range(&mut decoded, data, None);
    let before = owner.script_text_stats().unwrap().source_bytes;
    for m in [&mut legacy, &mut decoded] {
        m.bus.write16(0x040000de, 0x8000);
    }
    assert_eq!(owner.script_text_stats().unwrap().source_bytes - before, 2);
    range(&mut decoded, data, Some(&rom));
    assert!(legacy.save_state().unwrap() == decoded.save_state().unwrap());
    report(
        "native-result.json",
        serde_json::json!({"revision":[1,52,51],"original_interpreter_native_paths":cases.len(),"compiled_source_categories":compiled,"decoded_tail_source_literals":0,"poisoned_source_bytes":1803744,"full_states_clocks_equal":true,"raw_top255_variant255_index_sentinel65535_highbits":true,"mixed_previous2_new2_new2_previous2_shared_BB0_D68":true,"raw_fallback_after_previous_key_owner":true,"edges_widths_mirrors_bios_cpuset_fastset_dma":true,"instructions_coverage":instructions,"pcs":pcs.len(),"native_dispatches_coverage":dispatches,"scope":"controlled source fields/selectors/target prefixes; not whole text callback ABI/ordinary reachability","linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
fn names() -> [&'static str; 47] {
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
    ]
}
fn report(name: &str, value: serde_json::Value) {
    fs::write(
        root().join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
fn range(m: &mut Machine, data: &ScriptText, rom: Option<&[u8]>) {
    let start = model::ROOTS[0] as usize;
    let end = model::END as usize;
    let dst = &mut m.bus.rom[start..end];
    if let Some(r) = rom {
        dst.copy_from_slice(&r[start..end]);
    } else {
        for b in dst {
            *b ^= 0x5a;
        }
    }
    let _ = data;
}
#[test]
#[ignore = "private EUR e52 localized text command directory publication/authentication/lifecycle; Wine only, Linux/USA pending"]
fn european_script_text_managed_authentication_and_lifecycle() {
    let r = root();
    assert!(!r.exists());
    fs::create_dir_all(&r).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Vec<_> = names()[..46]
        .iter()
        .map(|n| fs::read(prior_dir.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old[0], "prior e51").unwrap();
    assert_eq!((prior.extractor_revision, prior.decoder_revision), (51, 50));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (52, 51)
    );
    extract(&dataset(), &mut source).unwrap();
    let files: Vec<_> = names()
        .iter()
        .map(|n| fs::read(dataset().join(n)).unwrap())
        .collect();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 47);
    assert_eq!(source.manifest.resources.len(), 46);
    for i in 1..46 {
        assert_eq!(files[i], old[i]);
    }
    let data = parse_script_text(&files[46]).unwrap();
    assert_eq!(
        data.source_spans()
            .unwrap()
            .iter()
            .map(|s| s.1.len())
            .sum::<usize>(),
        1803744
    );
    assert_eq!(
        files[46],
        fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_SOURCE_FILE").unwrap()).unwrap()
    );
    assert_eq!(source.manifest.resources[45].sources.len(), 5);
    assert_eq!(
        source.manifest.resources[45].dependencies,
        vec![SCRIPT_ARCHIVE_ID.to_string(), FONT_BANK_ID.to_string()]
    );
    let interrupted = r.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    assert!(write_dataset(
        &interrupted,
        &mut source,
        &mut |p, b| if p.ends_with(SCRIPT_TEXT_FILE) {
            Err("injected script command directory publication failure".into())
        } else {
            write_synced(p, b)
        }
    )
    .is_err());
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    let managed = r.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e51-d50/generation-retained");
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
    assert!(prepared.directory.parent().unwrap().ends_with("v1/e52-d51"));
    assert_eq!(
        preparation::prepare(&managed, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    fs::write(prepared.directory.join(SCRIPT_TEXT_FILE), b"partial").unwrap();
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
    missing.script_archive = None;
    assert_eq!(missing.manifest.resources.remove(44).id, SCRIPT_ARCHIVE_ID);
    missing
        .manifest
        .coverage
        .retain(|id| id != SCRIPT_ARCHIVE_ID);
    assert_eq!(
        missing.validate_files(fileset.files()),
        Err("Extraction resource set is inconsistent".into())
    );
    let mut m = crate::make_machine(rom.clone(), None);
    let boot = m.save_state().unwrap();
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    assert_eq!(
        data.languages
            .iter()
            .flat_map(|l| &l.banks)
            .map(|b| b.programs.len())
            .sum::<usize>(),
        41010
    );
    for case in 0..12 {
        let mut forged = data.clone();
        let mut manifest = source.manifest.clone();
        match case {
            0 => forged.alignment[0].bytes[0] ^= 1,
            1 => forged.languages[0].banks[0].programs[0].header ^= 0x10,
            2 => forged.languages[0].top[0] = forged.languages[0].top[1].clone(),
            3 => manifest.resources[45].sources[0].sha256 = "0".repeat(64),
            4 => manifest.resources[45].sources[0].length -= 4,
            5 => manifest.resources[45].sources[0].offset += 4,
            6 => manifest.resources[45].sources[0].role = "graphics/unknown".into(),
            7 => manifest.resources[45].path = "../script-text.json".into(),
            8 => manifest.resources[45].dependencies = vec![TASK_DESCRIPTORS_ID.into()],
            9 => manifest.resources[45].sha256 = "0".repeat(64),
            10 => manifest.coverage.retain(|id| id != SCRIPT_TEXT_ID),
            _ => {
                manifest.resources[45].sources.clear();
            }
        }
        let bytes = encode_script_text(&forged).unwrap();
        if case != 9 {
            manifest.resources[45].sha256 = gba_core::ffta_rom::digest(&bytes);
        }
        fs::write(dataset().join(SCRIPT_TEXT_FILE), bytes).unwrap();
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
    fs::write(dataset().join(SCRIPT_TEXT_FILE), &files[46]).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &files[0]).unwrap();
    fs::rename(dataset().join(SCRIPT_TEXT_FILE), r.join("missing.json")).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    assert_eq!(m.save_state().unwrap(), boot);
    fs::rename(r.join("missing.json"), dataset().join(SCRIPT_TEXT_FILE)).unwrap();
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
    let stats = owner.script_text_stats().unwrap();
    let field: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_TEXT_FIELDS").unwrap()).unwrap(),
    )
    .unwrap();
    let actual = [
        stats.top_index_bytes,
        stats.variant_index_bytes,
        stats.bank_index_bytes,
        stats.header_bytes,
        stats.encoded_control_bytes,
        stats.glyph_bytes,
        stats.command_bytes,
        stats.operand_bytes,
        stats.terminator_bytes,
        stats.decoded_tail_bytes,
        stats.alignment_bytes,
    ];
    let expected: [u64; 11] =
        std::array::from_fn(|i| field["category_counts"][i].as_u64().unwrap());
    assert_eq!(actual, expected);
    assert_eq!(
        (
            stats.source_bytes,
            stats.languages_used,
            stats.banks_used,
            stats.programs_used,
            stats.variant_records_used,
            stats.alignment_spans_used
        ),
        (1803744, 5, 385, 41010, 130, 34314)
    );
    let mut old_forms = BTreeSet::new();
    let mut old_bytes = 0;
    for p in fileset
        .text_catalogs
        .as_ref()
        .unwrap()
        .archive_catalogs
        .iter()
        .flat_map(|c| &c.languages)
        .flat_map(|a| &a.programs)
    {
        if !old_forms.insert(p.header & 3) {
            continue;
        }
        let bytes = p.source_bytes().unwrap();
        let at = p.source_offset as usize;
        for v in &mut m.bus.rom[at..at + bytes.len()] {
            *v ^= 0x5a;
        }
        let old = owner.text_catalog_stats().unwrap().source_bytes;
        let before = owner.script_text_stats().unwrap().source_bytes;
        for i in 0..bytes.len() {
            assert_eq!(
                (RT_API.read8)(
                    &mut m as *mut Machine as *mut std::ffi::c_void,
                    0x08000000 + p.source_offset + i as u32
                ),
                u32::from(rom[at + i])
            );
        }
        assert_eq!(
            owner.text_catalog_stats().unwrap().source_bytes - old,
            bytes.len() as u64
        );
        assert_eq!(owner.script_text_stats().unwrap().source_bytes, before);
        m.bus.rom[at..at + bytes.len()].copy_from_slice(&rom[at..at + bytes.len()]);
        old_bytes += bytes.len();
    }
    assert_eq!(old_forms, BTreeSet::from([0, 1, 2]));
    report(
        "managed-result.json",
        serde_json::json!({"revision":[1,52,51],"files":47,"resources":46,"source_ranges":5,"source_bytes":1803744,"all45prior_decoded_files_and_e51_retained":true,"publication_failure_create_reuse_repair_move":true,"coherent_guards":13,"preceding_owner_missing_rejected_before_files":true,"missingfile_transactional_owner_unavailable_load_undo_backup_restart":true,"all_typed_bytes_5languages385banks41010programs130variants34314gaps":true,"category_counts":actual,"affected_prior_text_0_1_2_program_source_bytes":old_bytes,"linux":"pending","usa":"pending","performance_measurements":false}),
    );
}
