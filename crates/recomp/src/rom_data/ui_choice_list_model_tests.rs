//! One focused EUR/Wine model, original copy, getter and live-template oracle.
use super::*;
use gba_core::{rom_data::ui_choice_lists::*, Machine};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const WIDGET: usize = 0x10000;
const ENTRY_SP: u32 = 0x03007e00;
#[derive(Default)]
struct Coverage {
    reads: BTreeSet<u32>,
    pcs: BTreeSet<u32>,
    instructions: usize,
}
fn put32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn fresh(m: &mut Machine) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn cpu(m: &mut Machine, pc: u32, args: &[u32]) {
    m.cpu.regs = [0; 16];
    for n in 4..12 {
        m.cpu.regs[n] = 0x14104000 + n as u32 * 0x101;
    }
    m.cpu.regs[..args.len()].copy_from_slice(args);
    m.cpu.regs[13] = ENTRY_SP;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
fn until(m: &mut Machine, end: u32, c: &mut Coverage) {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(n < 20000, "original PC {pc:08x}, expected {end:08x}");
        c.pcs.insert(pc);
        let source = match pc {
            0x0808bc1e => Some((m.cpu.regs[4], 8)),
            0x0808250c | 0x080826ae => Some((m.cpu.regs[0] + 4, 4)),
            0x0808250e | 0x080826b0 => Some((m.cpu.regs[0], 4)),
            0x08082538 | 0x080826d8 => Some((m.cpu.regs[2], 4)),
            0x0808253a | 0x080826da => Some((m.cpu.regs[2] + 4, 4)),
            _ => None,
        };
        if let Some((at, len)) = source {
            assert!((0x08000000..0x09000000).contains(&at));
            c.reads.extend((at..at + len).map(|a| a - 0x08000000));
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    c.instructions += n;
}
fn call(m: &mut Machine, pc: u32, args: &[u32], c: &mut Coverage) {
    cpu(m, pc, args);
    let before = m.cpu.regs;
    until(m, 0x08000300, c);
    assert_eq!(m.cpu.regs[4..12], before[4..12]);
    assert_eq!(m.cpu.regs[13], before[13]);
}
fn ram(m: &Machine, ew: &[u8], iw: &[u8], stack: bool) {
    assert_eq!(m.bus.ewram, ew);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iw.to_vec();
    if stack {
        actual[0x7d00..0x7e00].fill(0);
        expected[0x7d00..0x7e00].fill(0);
    }
    assert_eq!(actual, expected);
}
fn copy_prefix(m: &mut Machine, source: u32, count: u32, c: &mut Coverage) {
    put32(&mut m.bus.iwram, 0x7e18, 0x1234);
    let mut ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    ew[WIDGET + 0x104] = count as u8;
    ew[WIDGET + 0x105] = 0x34;
    ew[WIDGET + 0x106] = 0xff;
    if count as i32 > 0 {
        let start = source as usize;
        let len = count as usize * 8;
        ew[WIDGET + 4..WIDGET + 4 + len].copy_from_slice(&m.bus.rom[start..start + len]);
    }
    cpu(
        m,
        0x0808bbf0,
        &[0x02000000 + WIDGET as u32, 0x08000000 + source, count, 0x1c],
    );
    until(m, 0x0808bc2c, c);
    ram(m, &ew, &iw, true);
    assert_eq!(m.cpu.regs[13], ENTRY_SP - 44);
}
fn idle_and_get(m: &mut Machine, entry: &ChoiceEntry, index: usize, c: &mut Coverage) {
    m.bus.ewram[WIDGET + 0x105] = index as u8;
    m.bus.ewram[WIDGET + 0x10d] = 0xff;
    let mut ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    call(m, 0x0808ca64, &[0x02000000 + WIDGET as u32], c);
    assert_eq!(m.cpu.regs[0], u32::from(entry.identifier));
    ram(m, &ew, &iw, false);
    ew[WIDGET + 1] &= !0x20;
    ew[WIDGET + 0x110] = entry.cursor_x;
    ew[WIDGET + 0x111] = entry.cursor_y;
    call(m, 0x0808bdec, &[0x02000000 + WIDGET as u32, 0], c);
    assert_eq!(m.cpu.regs[0], 0);
    ram(m, &ew, &iw, true);
}

#[test]
#[ignore = "private EUR UI choice source/copy/getter oracle; Wine only, Linux/USA pending"]
fn european_ui_choice_lists_match_original_copies_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_CHOICE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_CHOICE_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 13);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 336);
    for (((list, source), (offset, bytes, role)), independent) in data
        .lists
        .iter()
        .zip(&catalog.sources)
        .zip(&spans)
        .zip(plan["sources"].as_array().unwrap())
    {
        assert_eq!(
            u64::from(list.source_offset),
            independent["source_offset"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(source.length),
            independent["length"].as_u64().unwrap()
        );
        assert_eq!(source.sha256, independent["sha256"].as_str().unwrap());
        assert_eq!(
            (source.offset, source.length as usize, &source.role),
            (*offset, bytes.len(), role)
        );
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
        assert_eq!(gba_core::ffta_rom::digest(bytes), source.sha256);
        for (i, entry) in list.entries.iter().enumerate() {
            let v = &independent["entries"][i];
            let expected = [
                "identifier",
                "cursor_x",
                "cursor_y",
                "opaque",
                "left_raw",
                "right_raw",
                "up_raw",
                "down_raw",
            ]
            .map(|k| v[k].as_u64().unwrap() as u8);
            assert_eq!(entry.bytes(), expected);
        }
        for (i, &byte) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*offset + i as u32), Some(byte));
        }
        assert_eq!(list.byte(bytes.len()), None);
        assert_eq!(list.byte(usize::MAX), None);
    }
    assert_eq!(data.byte_at(0), None);
    assert_eq!(data.byte_at(u32::MAX), None);
    assert_eq!(data.byte_at(0x397a74), None);
    assert_eq!(data.byte_at(0x397acc), None);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<UiChoiceLists>(&encoded).unwrap(),
        *data
    );
    let mut guards = Vec::new();
    macro_rules! invalid {
        ($name:expr,$edit:expr) => {{
            let mut v = data.clone();
            ($edit)(&mut v);
            assert!(v.validate().is_err(), "{}", $name);
            guards.push($name);
        }};
    }
    invalid!("missing list", |v: &mut UiChoiceLists| {
        v.lists.pop();
    });
    invalid!("extra list", |v: &mut UiChoiceLists| v
        .lists
        .push(v.lists[0].clone()));
    invalid!("physical ID", |v: &mut UiChoiceLists| v.lists[0].id =
        "other".into());
    invalid!("root", |v: &mut UiChoiceLists| v.lists[0].source_offset +=
        4);
    invalid!("list order", |v: &mut UiChoiceLists| v.lists.swap(0, 1));
    invalid!("kind", |v: &mut UiChoiceLists| v.lists[0].kind =
        ChoiceKind::FilteredTemplate);
    invalid!("missing entry", |v: &mut UiChoiceLists| {
        v.lists[0].entries.pop();
    });
    invalid!("extra entry", |v: &mut UiChoiceLists| {
        let e = v.lists[0].entries[0].clone();
        v.lists[0].entries.push(e);
    });
    for level in 0..4 {
        let mut v = serde_json::to_value(data).unwrap();
        match level {
            0 => v["extra"] = 1.into(),
            1 => v["lists"][0]["extra"] = 1.into(),
            2 => v["lists"][0]["entries"][0]["extra"] = 1.into(),
            _ => v["lists"][0]["entries"][0]["navigation"]["extra"] = 1.into(),
        }
        assert!(serde_json::from_value::<UiChoiceLists>(v).is_err());
        guards.push("unknown JSON field");
    }
    for bad in [-1, 256] {
        let mut v = serde_json::to_value(data).unwrap();
        v["lists"][0]["entries"][0]["opaque"] = bad.into();
        assert!(serde_json::from_value::<UiChoiceLists>(v).is_err());
        guards.push("byte width");
    }
    assert!(Catalog::from_rom(&[]).is_err());
    guards.push("source length");
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards.push("source identity");
    let mut edited = data.clone();
    edited.lists[0].entries[0] = ChoiceEntry {
        identifier: 255,
        cursor_x: 254,
        cursor_y: 253,
        opaque: 252,
        navigation: Navigation {
            left: 255,
            right: 254,
            up: 253,
            down: 252,
        },
    };
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    guards.push("authenticated source content");
    let mut variant_rom = rom.clone();
    for (at, bytes, _) in edited.source_spans().unwrap() {
        variant_rom[at as usize..at as usize + bytes.len()].copy_from_slice(&bytes);
    }
    assert!(Catalog::from_rom(&variant_rom).is_err());
    let mut variant = Machine::new(variant_rom);
    fresh(&mut variant);
    let mut c = Coverage::default();
    copy_prefix(
        &mut variant,
        edited.lists[0].source_offset,
        edited.lists[0].entries.len() as u32,
        &mut c,
    );
    idle_and_get(&mut variant, &edited.lists[0].entries[0], 0, &mut c);
    let mut m = Machine::new(rom.clone());
    for list in &data.lists {
        fresh(&mut m);
        copy_prefix(
            &mut m,
            list.source_offset,
            list.entries.len() as u32,
            &mut c,
        );
        for (i, entry) in list.entries.iter().enumerate() {
            idle_and_get(&mut m, entry, i, &mut c);
        }
    }
    // Preserve raw signed count and the copy's overwrite ordering, even outside the projection.
    for count in [0, 0xffffffff, 0xffff0000, 257] {
        fresh(&mut m);
        copy_prefix(&mut m, data.lists[0].source_offset, count, &mut c);
    }
    fresh(&mut m);
    m.bus.ewram[WIDGET + 0x105] = 255;
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    call(&mut m, 0x0808ca64, &[0x02000000 + WIDGET as u32], &mut c);
    assert_eq!(m.cpu.regs[0], 0x5a);
    ram(&m, &ew, &iw, false);
    let template = data
        .lists
        .iter()
        .find(|l| l.kind == ChoiceKind::FilteredTemplate)
        .unwrap();
    let template_bytes: Vec<u8> = template
        .entries
        .iter()
        .flat_map(ChoiceEntry::bytes)
        .collect();
    for mask in 0u32..32 {
        for (start, end) in [(0x080824e0, 0x08082544), (0x08082680, 0x080826e4)] {
            fresh(&mut m);
            cpu(&mut m, start, &[]);
            put32(&mut m.bus.iwram, 0x2818, 0x02018000);
            put32(&mut m.bus.ewram, 0x18000 + 0x1d10, 0x02019000);
            for i in 0..5 {
                let value = if mask & (1 << i) != 0 { 1u16 } else { 0 };
                let at = if start == 0x080824e0 {
                    0x1902a + 2 * i
                } else {
                    0x18000 + 0x1238 + 2 * i
                };
                m.bus.ewram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            let mut ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            let mut selected = 0usize;
            for i in 0..5 {
                if mask & (1 << i) != 0 {
                    selected += 1;
                    let at = 0xf250 + selected * 8;
                    ew[at..at + 8].copy_from_slice(&template_bytes[selected * 8..selected * 8 + 8]);
                    ew[at] = i as u8;
                    ew[at + 2] = 0x41 + 16 * i as u8;
                    ew[at + 6] = (selected - 1) as u8;
                    ew[at + 7] = (selected + 1) as u8;
                }
            }
            ew[0xf250 + selected * 8 + 7] = 0;
            ew[0xf250..0xf258].copy_from_slice(&template_bytes[..8]);
            ew[0xf256] = selected as u8;
            until(&mut m, end, &mut c);
            ram(&m, &ew, &iw, false);
            assert_eq!(m.cpu.regs[9], 0x0200f250);
            assert_eq!(
                m.bus.ewram[0xf250 + selected * 8 + 7],
                if selected == 0 { 1 } else { 0 }
            );
        }
    }
    // Observe the unchanged post-sound move prefix with raw high-bit addressing.
    for raw in [0x80u8, 0x86] {
        fresh(&mut m);
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let at = WIDGET + usize::from(raw) * 8;
        m.bus.ewram[at + 5] = 0x91;
        m.bus.ewram[at + 6] = 0x92;
        m.bus.ewram[at + 7] = 0x93;
        ew[at + 5] = 0x91;
        ew[at + 6] = 0x92;
        ew[at + 7] = 0x93;
        ew[WIDGET + 0x106] = 0x5a;
        ew[WIDGET + 0x105] = raw & 127;
        ew[WIDGET + 0x110] = 0x91;
        ew[WIDGET + 0x111] = 0x92;
        ew[WIDGET + 0x107] = 0x93;
        cpu(&mut m, 0x0808be66, &[]);
        m.cpu.regs[4] = 0x02000000 + WIDGET as u32;
        m.cpu.regs[5] = u32::from(raw);
        m.cpu.regs[6] = 0x02000000 + WIDGET as u32 + 0x105;
        until(&mut m, 0x0808bfae, &mut c);
        assert_eq!(m.cpu.regs[0], 1);
        ram(&m, &ew, &iw, false);
    }
    let owned: BTreeSet<u32> = spans
        .iter()
        .flat_map(|(at, bytes, _)| *at..*at + bytes.len() as u32)
        .collect();
    assert!(owned.is_subset(&c.reads));
    fs::write(root.join("ui-choice-lists.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&json!({
        "source_ranges":13,"source_bytes":336,"lists":12,"templates":1,"entries":42,
        "shape_json_source_guards":guards,"original_copy_prefixes":18,"original_getters":44,
        "original_idle_cursor_calls":43,"live_template_producer_prefixes":64,
        "raw_high_flag_coordinate_prefixes":2,"synthetic_all_fields_variant":1,
        "owned_distinct_bytes_at_original_loadsites":owned.intersection(&c.reads).count(),
        "instructions_coverage":c.instructions,"PC_coverage":c.pcs.len(),
        "scope":"component copy/getter/idle/live-template and raw move prefixes; not whole widget ABI/native/fullstate-clock/ordinary",
        "revision":"EUR e55-d54/USA e8-d7 unchanged","Linux_USA_nativeWindows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
