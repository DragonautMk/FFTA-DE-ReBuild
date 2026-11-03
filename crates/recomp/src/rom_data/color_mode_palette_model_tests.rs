//! One bounded new EUR/Wine source-model and original-reader oracle.
use super::*;
use gba_core::{rom_data::color_mode_palettes::*, Machine};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;
const DEST: u32 = 0x03003a60;
#[derive(Default)]
struct Coverage {
    reads: BTreeSet<u32>,
    pcs: BTreeSet<u32>,
    instructions: usize,
}
fn number(v: &Value) -> u32 {
    v.as_u64().unwrap() as u32
}
fn put32(b: &mut [u8], at: usize, n: u32) {
    b[at..at + 4].copy_from_slice(&n.to_le_bytes());
}
fn reset(m: &mut Machine, mode: u8, pc: u32, args: &[u32]) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    // The real startup relocates this position-independent executable helper.
    // Its original instructions are executed here, never replaced by a host copy.
    let copy = m.bus.rom[0xd968a8..0xd968e6].to_vec();
    m.bus.iwram[0x5ee8..0x5f26].copy_from_slice(&copy);
    m.bus.ewram[0x2fc2] = mode;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
    m.cpu.regs = [0; 16];
    for i in 4..12 {
        m.cpu.regs[i] = 0x14104000 + i as u32 * 0x101;
    }
    m.cpu.regs[..args.len()].copy_from_slice(args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
impl Coverage {
    fn add(&mut self, at: u32, length: u32) {
        if (0x08000000..0x09000000).contains(&at) {
            self.reads.extend((at..at + length).map(|a| a - 0x08000000));
        }
    }
    fn observe(&mut self, m: &Machine) {
        let pc = m.cpu.regs[15];
        self.pcs.insert(pc);
        let op = if (0x08000000..0x09000000).contains(&pc) {
            let at = (pc - 0x08000000) as usize;
            u16::from_le_bytes(m.bus.rom[at..at + 2].try_into().unwrap())
        } else {
            let at = (pc - 0x03000000) as usize;
            u16::from_le_bytes(m.bus.iwram[at..at + 2].try_into().unwrap())
        };
        let r = &m.cpu.regs;
        let base = r[usize::from((op >> 3) & 7)];
        let imm = u32::from((op >> 6) & 31);
        match op & 0xf800 {
            0x6800 => self.add(base.wrapping_add(imm * 4) & !3, 4),
            0x7800 => self.add(base.wrapping_add(imm), 1),
            0x8800 => self.add(base.wrapping_add(imm * 2) & !1, 2),
            _ => (),
        }
        if op & 0xf000 == 0x5000 {
            let at = base.wrapping_add(r[usize::from((op >> 6) & 7)]);
            match (op >> 9) & 7 {
                3 | 6 => self.add(at, 1),
                4 => self.add(at & !3, 4),
                5 | 7 => self.add(at & !1, 2),
                _ => (),
            }
        }
        if op & 0xf800 == 0xc800 {
            self.add(
                r[usize::from((op >> 8) & 7)] & !3,
                (op & 255).count_ones() * 4,
            );
        }
        if op == 0xdf0b {
            assert_eq!(r[2] & !0x1fffff, 0);
            self.add(r[0] & !1, (r[2] & 0x1fffff) * 2);
        }
    }
    fn until(&mut self, m: &mut Machine, end: u32) {
        for _ in 0..20000 {
            if m.cpu.regs[15] == end {
                return;
            }
            self.observe(m);
            m.bus.frame_ready = false;
            m.step();
            self.instructions += 1;
        }
        panic!(
            "Color-Mode original PC {:08x}, expected {end:08x}",
            m.cpu.regs[15]
        );
    }
}
fn ram(m: &Machine, ew: &[u8], iw: &[u8]) {
    assert_eq!(m.bus.ewram, ew);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iw.to_vec();
    actual[0x7d00..0x7e00].fill(0);
    expected[0x7d00..0x7e00].fill(0);
    assert_eq!(actual, expected);
}
fn expected_copy(m: &Machine, ew: &mut [u8], iw: &mut [u8], dest: u32, source: u32, length: usize) {
    let bytes = &m.bus.rom[source as usize..source as usize + length];
    if (0x03000000..0x03008000).contains(&dest) {
        let at = (dest - 0x03000000) as usize;
        iw[at..at + length].copy_from_slice(bytes);
    } else {
        assert!((0x02000000..0x02040000).contains(&dest));
        let at = (dest - 0x02000000) as usize;
        ew[at..at + length].copy_from_slice(bytes);
    }
}
fn copy_prefix(
    m: &mut Machine,
    c: &mut Coverage,
    call: u32,
    source: u32,
    length: usize,
    swi: bool,
) {
    c.until(m, 0x08000000 + call);
    let (dest, actual_source) = if swi {
        assert_eq!(m.cpu.regs[2] as usize * 2, length);
        (m.cpu.regs[1], m.cpu.regs[0])
    } else {
        assert_eq!(m.cpu.regs[2] as usize, length);
        assert_eq!(m.cpu.regs[3], 0x03005ee9);
        (m.cpu.regs[0], m.cpu.regs[1])
    };
    assert_eq!(actual_source, 0x08000000 + source);
    assert!(
        (0x02000000..0x02040000).contains(&dest) || (0x03000000..0x03008000).contains(&dest),
        "copy call{call:08x} has invalid seeded destination{dest:08x}"
    );
    let mut ew = m.bus.ewram.clone();
    let mut iw = m.bus.iwram.clone();
    expected_copy(m, &mut ew, &mut iw, dest, source, length);
    let before = m.cpu.regs;
    c.until(m, 0x08000000 + call + 4);
    ram(m, &ew, &iw);
    assert_eq!(m.cpu.regs[4..12], before[4..12]);
    assert_eq!(m.cpu.regs[13], before[13]);
}

#[test]
#[ignore = "private new EUR Color-Mode source-model/original-copy oracle; Wine only, Linux/USA pending"]
fn european_color_mode_palettes_match_original_selectors_and_copies() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COLOR_MODE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_COLOR_MODE_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 30);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 5750);
    for (((at, bytes, role), source), independent) in spans
        .iter()
        .zip(&catalog.sources)
        .zip(plan["source_ranges"].as_array().unwrap())
    {
        assert_eq!(*at, number(&independent["offset"]));
        assert_eq!(bytes.len(), number(&independent["length"]) as usize);
        assert_eq!(role, independent["role"].as_str().unwrap());
        assert_eq!(source.sha256, independent["sha256"].as_str().unwrap());
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        for (n, &byte) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*at + n as u32), Some(byte));
        }
        if !spans.iter().any(|s| s.0 + s.1.len() as u32 == *at) {
            assert_eq!(data.byte_at(*at - 1), None);
        }
        // Adjacent books can have their own immediately following owned span.
        let end = *at + bytes.len() as u32;
        if !spans.iter().any(|s| s.0 == end) {
            assert_eq!(data.byte_at(end), None);
        }
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        &serde_json::from_slice::<ColorModePalettes>(&encoded).unwrap(),
        data
    );
    data.verify_source(&rom).unwrap();
    let mut malformed = serde_json::to_value(data).unwrap();
    malformed["unknown"] = json!(true);
    assert!(serde_json::from_value::<ColorModePalettes>(malformed).is_err());
    let mut guards = 1;
    for field in ["id", "book", "variant", "source_offset", "colors"] {
        let mut value = serde_json::to_value(data).unwrap();
        value["spans"][0][field] = match field {
            "id" => json!("wrong"),
            "book" => json!("shared-colors"),
            "variant" => json!(3),
            "source_offset" => json!(0),
            _ => json!([]),
        };
        assert!(serde_json::from_value::<ColorModePalettes>(value)
            .unwrap()
            .validate()
            .is_err());
        guards += 1;
    }
    let mut changed = data.clone();
    changed.spans[0].colors[0] ^= 0x8001;
    changed.slice_offsets[0] = 255;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&rom[..128]).is_err());
    guards += 1;

    let mut m = Machine::new(rom.clone());
    let mut c = Coverage::default();
    let mut selectors = 0;
    for mode in [0, 1, 2, 3, 0xfe] {
        for (book, pc) in [
            (Book::UnitBase, 0xcf49c),
            (Book::CompanionBase, 0xcf4e4),
            (Book::Supplemental, 0xcf51c),
            (Book::SharedColors, 0xcf5a0),
            (Book::SpecialUnit, 0xcf5dc),
        ] {
            let args: &[u32] = if book == Book::SharedColors {
                &[0]
            } else {
                &[0, 15, 16, 26, 255, 0x100, 0x101, 0xffff]
            };
            for &arg in args {
                reset(&mut m, mode, 0x08000000 + pc, &[arg]);
                let before = m.cpu.regs;
                let ew = m.bus.ewram.clone();
                let iw = m.bus.iwram.clone();
                c.until(&mut m, RETURN);
                assert_eq!(m.cpu.regs[0], 0x08000000 + book.source(mode, arg));
                assert_eq!(m.cpu.regs[4..12], before[4..12]);
                assert_eq!(m.cpu.regs[13], STACK);
                ram(&m, &ew, &iw);
                selectors += 1;
            }
        }
    }
    let mut copies = 0;
    let mut staged = 0;
    let mut special_cases = Vec::new();
    for row in plan["special_projections"].as_array().unwrap() {
        let index = number(&row[3]);
        if !special_cases
            .iter()
            .any(|r: &&Value| number(&r[3]) == index)
        {
            special_cases.push(row);
        }
    }
    assert_eq!(special_cases.len(), 9);
    let mut special = 0;
    let mut slice_cases = 0;
    for mode in [0, 1, 2] {
        // Original initializer's two complete512B copies, including mode1 BG alias.
        reset(&mut m, mode, 0x08023b44, &[]);
        m.cpu.regs[4] = DEST;
        m.cpu.regs[6] = 0x08370eb4;
        copy_prefix(
            &mut m,
            &mut c,
            0x23b56,
            Book::UnitBase.source(mode, 0),
            512,
            false,
        );
        copy_prefix(
            &mut m,
            &mut c,
            0x23b6c,
            Book::CompanionBase.source(mode, 0),
            512,
            false,
        );
        copies += 2;
        // Every independently rooted CF51C constant consumer; no adjacent rows.
        for r in plan["supplemental_requests"].as_array().unwrap() {
            let call = number(&r["helper_call"]);
            if call == 0x23dc4 {
                continue;
            }
            let consumer = number(&r["consumer"]);
            let consume_call = number(&r["consumer_call"]);
            let source = Book::Supplemental.source(mode, number(&r["index"])) + number(&r["delta"]);
            reset(&mut m, mode, 0x08000000 + call - 2, &[]);
            if consumer == 0x145c14 {
                // Each isolated prefix needs its actual live destination and
                // callback registers. Decode the nearby original MOV/LDR rather
                // than guessing that every caller uses the same register pair.
                let at = consume_call as usize;
                let callback = u16::from_le_bytes(rom[at - 6..at - 4].try_into().unwrap());
                assert_eq!(callback & 0xffc7, 0x6803);
                m.cpu.regs[usize::from((callback >> 3) & 7)] = 0x08370eb4;
                let dest = u16::from_le_bytes(rom[at - 4..at - 2].try_into().unwrap());
                if dest & 0xffc7 == 0x1c00 {
                    m.cpu.regs[usize::from((dest >> 3) & 7)] = DEST;
                } else {
                    // 29364 loads its fixed IWRAM destination from a literal.
                    assert_eq!(dest & 0xff00, 0x4800);
                }
            }
            if consumer == 0x14b6f0 {
                m.cpu.regs[4] = 0x160;
                c.until(&mut m, 0x08000000 + consume_call);
                assert_eq!(m.cpu.regs[3], 0x08000000 + source);
                assert_eq!(m.cpu.regs[1], 16);
                let start = m.cpu.regs[0] as usize;
                assert!(start + 16 <= 512);
                let ew = m.bus.ewram.clone();
                let mut iw = m.bus.iwram.clone();
                for i in 0..16 {
                    let at = 0x3e68 + 12 * (start + i);
                    iw[at..at + 2].copy_from_slice(
                        &rom[source as usize + 2 * i..source as usize + 2 * i + 2],
                    );
                }
                // Stop before allocation/scheduling: precisely the source reader contract.
                c.until(&mut m, 0x0814b744);
                ram(&m, &ew, &iw);
                assert_eq!(m.cpu.regs[13], STACK - 20);
                staged += 1;
            } else {
                copy_prefix(
                    &mut m,
                    &mut c,
                    consume_call,
                    source,
                    number(&r["length"]) as usize,
                    consumer == 0x145230,
                );
                copies += 1;
            }
        }
        for (start, call) in [(0xa95e2, 0xa95f0), (0xa9cb8, 0xa9cc6)] {
            reset(&mut m, mode, 0x08000000 + start, &[]);
            m.cpu.regs[4] = 0x08370eb4;
            copy_prefix(
                &mut m,
                &mut c,
                call,
                Book::SharedColors.source(mode, 0),
                12,
                false,
            );
            copies += 1;
        }
        // One actual metadata14 producer per physical special selector. Both
        // argument branches occur in these representatives; no invented live cap.
        for row in &special_cases {
            let id = number(&row[0]);
            let arg = number(&row[1]);
            let bank = number(&row[2]);
            let index = number(&row[3]);
            for alternate in [false, true] {
                let (start, call) = if alternate {
                    (0x73c32, if arg == 1 { 0x73c78 } else { 0x73cbc })
                } else {
                    (0xcf09e, if arg == 1 { 0xcf0dc } else { 0xcf118 })
                };
                reset(&mut m, mode, 0x08000000 + start, &[]);
                m.bus.ewram[0x10004] = id as u8;
                put32(&mut m.bus.iwram, 0x2818, 0x02012000);
                m.cpu.regs[4] = 0x02010000;
                m.cpu.regs[5] = arg;
                c.until(&mut m, 0x08000000 + call);
                let expected_dest = if alternate { 0x02012228 } else { DEST } + 32 * bank;
                assert_eq!(m.cpu.regs[0], expected_dest);
                copy_prefix(
                    &mut m,
                    &mut c,
                    call,
                    Book::SpecialUnit.source(mode, index),
                    32,
                    false,
                );
                special += 1;
            }
        }
        for r in plan["slice_callers"].as_array().unwrap() {
            let call = number(&r["call"]);
            let a = number(&r["args"][0]) as usize;
            let b = number(&r["args"][1]) as usize;
            reset(&mut m, mode, 0x08000000 + call - 4, &[]);
            let ew = m.bus.ewram.clone();
            let mut iw = m.bus.iwram.clone();
            let base = Book::Supplemental.source(mode, 4) as usize;
            for (arg, dest) in [(a, 0x39a4), (b, 0x39b2)] {
                let source = base + usize::from(data.slice_offsets[arg - 1]) * 2;
                iw[dest..dest + 14].copy_from_slice(&rom[source..source + 14]);
            }
            c.until(&mut m, 0x08023e04);
            ram(&m, &ew, &iw);
            assert_eq!(m.cpu.regs[13], STACK - 28);
            slice_cases += 1;
        }
    }
    let owned: BTreeSet<u32> = spans
        .iter()
        .flat_map(|(at, bytes, _)| *at..*at + bytes.len() as u32)
        .collect();
    let missing: Vec<_> = owned.difference(&c.reads).copied().collect();
    assert!(
        missing.is_empty(),
        "unread owned colors/selectors: {missing:x?}"
    );
    fs::write(root.join("color-mode-palettes.json"), encoded).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&json!({
        "source_spans":30,"source_bytes":5750,"color_words":2873,"selector_bytes":4,
        "original_pointer_cases":selectors,"original_copy_prefixes":copies,
        "original_staged_reader_prefixes":staged,"original_special_copy_prefixes":special,
        "special_selector_representatives":9,"static_special_projections":51,
        "original_slice_prefixes":slice_cases,"owned_distinct_bytes_at_actual_readers":owned.intersection(&c.reads).count(),
        "model_guards":guards,"instructions_coverage":c.instructions,"pcs_coverage":c.pcs.len(),
        "ram_outside_copies_and_stack_unchanged":true,"fullword_to_low8_arguments_and_low2_modes_preserved":true,
        "scope":"bounded source model and original read/copy prefixes; no full allocator/native/state-clock/ordinary claim",
        "managed":"EUR e57-d56 unchanged; USAe8-d7 unchanged","linux":"pending","usa":"pending",
        "native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
