//! One source/model check, independent of earlier callback/runtime acceptance.
use super::*;
use gba_core::{rom_data::palette_programs::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;
const CONTEXT: u32 = 0x02010000;
const MESSAGE: u32 = 0x02011000;
const OBJECT: u32 = 0x02012000;
const RETURN: u32 = 0x08000300;

fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0);
    m.bus.iwram.fill(0);
    // Original relocated position-independent helper, already source-classified.
    let copy = m.bus.rom[0xd968a8..0xd968e6].to_vec();
    m.bus.iwram[0x5ee8..0x5f26].copy_from_slice(&copy);
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn put16(m: &mut Machine, a: u32, value: u16) {
    let at = (a - 0x02000000) as usize;
    m.bus.ewram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(m: &mut Machine, a: u32, value: u32) {
    let at = (a - 0x02000000) as usize;
    m.bus.ewram[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn context(m: &mut Machine, p: &Program, index: usize) {
    put32(m, CONTEXT, 0x08000000 + p.source_offset);
    put16(m, CONTEXT + 4, p.header.first_color());
    m.bus.ewram[0x10006] = p.header.width;
    put16(m, CONTEXT + 10, index as u16);
    m.cpu.regs[0] = CONTEXT;
}
#[derive(Default)]
struct OriginalReads {
    bytes: BTreeSet<u32>,
    pcs: BTreeSet<u32>,
    instructions: usize,
}
impl OriginalReads {
    fn observe(&mut self, m: &Machine) {
        let pc = m.cpu.regs[15];
        self.pcs.insert(pc);
        let at = if (0x08000000..0x09000000).contains(&pc) {
            let p = (pc - 0x08000000) as usize;
            u16::from_le_bytes(m.bus.rom[p..p + 2].try_into().unwrap())
        } else {
            let p = (pc - 0x03000000) as usize;
            u16::from_le_bytes(m.bus.iwram[p..p + 2].try_into().unwrap())
        };
        let r = &m.cpu.regs;
        let base = r[usize::from((at >> 3) & 7)];
        let imm = u32::from((at >> 6) & 31);
        let read = match at & 0xf800 {
            0x4800 => Some((((pc + 4) & !3) + u32::from(at & 255) * 4, 4)),
            0x6800 => Some((base.wrapping_add(imm * 4) & !3, 4)),
            0x7800 => Some((base.wrapping_add(imm), 1)),
            0x8800 => Some((base.wrapping_add(imm * 2) & !1, 2)),
            _ => None,
        };
        if let Some((a, n)) = read {
            self.add(a, n);
        }
        if at & 0xf000 == 0x5000 {
            let a = base.wrapping_add(r[usize::from((at >> 6) & 7)]);
            match (at >> 9) & 7 {
                3 | 6 => self.add(a, 1),
                4 => self.add(a & !3, 4),
                5 | 7 => self.add(a & !1, 2),
                _ => (),
            }
        }
        if at & 0xf800 == 0xc800 {
            let a = r[usize::from((at >> 8) & 7)] & !3;
            self.add(a, 4 * (at & 255).count_ones());
        }
    }
    fn add(&mut self, at: u32, n: u32) {
        if (0x08000000..0x09000000).contains(&at) {
            self.bytes.extend((at..at + n).map(|a| a - 0x08000000));
        }
    }
    fn until(&mut self, m: &mut Machine, stop: u32) {
        for _ in 0..20000 {
            if m.cpu.regs[15] == stop {
                return;
            }
            self.observe(m);
            m.bus.frame_ready = false;
            m.step();
            self.instructions += 1;
        }
        panic!(
            "Original palette prefix did not reach {stop:08x}, pc{:08x}",
            m.cpu.regs[15]
        );
    }
    fn one(&mut self, m: &mut Machine) {
        self.observe(m);
        m.bus.frame_ready = false;
        m.step();
        self.instructions += 1;
    }
}
fn number(v: &Value) -> u32 {
    v.as_u64().unwrap() as u32
}

#[test]
#[ignore = "private EUR palette program decoder and original reader prefixes; Wine only, Linux/USA pending"]
fn european_palette_program_model_matches_original_source_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PALETTE_PROGRAM_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PALETTE_PROGRAM_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let indexed: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PALETTE_PROGRAM_PRODUCERS").unwrap()).unwrap(),
    )
    .unwrap();
    let cat = Catalog::from_rom(&rom).unwrap();
    let data = &cat.decoded;
    assert_eq!(data.programs.len(), 230);
    assert_eq!(data.producers.len(), 25);
    let mut declared = BTreeSet::new();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 165);
    assert_eq!(cat.sources.len(), 165);
    let expected_ranges = evidence["combined_ranges"].as_array().unwrap();
    for (((at, bytes, _), source), expected) in spans.iter().zip(&cat.sources).zip(expected_ranges)
    {
        assert_eq!(*at, number(&expected["offset"]));
        assert_eq!(bytes.len(), number(&expected["length"]) as usize);
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            expected["sha256"].as_str().unwrap()
        );
        assert_eq!(source.offset, *at);
        assert_eq!(source.length as usize, bytes.len());
        assert_eq!(source.sha256, expected["sha256"].as_str().unwrap());
        assert_eq!(*bytes, rom[*at as usize..*at as usize + bytes.len()]);
        declared.extend(*at..*at + bytes.len() as u32);
    }
    assert_eq!(declared.len(), 18697);
    let mut program_fields = BTreeSet::new();
    let mut command_count = 0;
    for (p, e) in data
        .programs
        .iter()
        .zip(evidence["programs"].as_array().unwrap())
    {
        assert_eq!(p.source_offset, number(&e["source_offset"]));
        for (actual, name) in [
            (p.header.first, "first"),
            (p.header.flags, "flags"),
            (p.header.width, "width"),
            (p.header.command_count, "count"),
        ] {
            assert_eq!(u32::from(actual), number(&e["header"][name]));
        }
        assert_eq!(
            u32::from(p.header.initial_delay),
            number(&e["header"]["initial_delay"])
        );
        program_fields.extend(p.source_offset..p.source_offset + 6);
        assert_eq!(p.commands.len(), e["commands"].as_array().unwrap().len());
        for (i, (c, x)) in p
            .commands
            .iter()
            .zip(e["commands"].as_array().unwrap())
            .enumerate()
        {
            for (actual, name) in [
                (u32::from(c.opcode), "opcode"),
                (u32::from(c.mode), "mode"),
                (u32::from(c.duration), "duration"),
                (u32::from(c.relative_parameter), "relative_parameter"),
            ] {
                assert_eq!(actual, number(&x[name]));
            }
            let bytes = c.parameter.source_bytes();
            let pa = p.source_offset + u32::from(c.relative_parameter);
            assert_eq!(bytes.len(), number(&x["parameter_bytes"]) as usize);
            assert_eq!(
                gba_core::ffta_rom::digest(&bytes),
                x["parameter_sha256"].as_str().unwrap()
            );
            program_fields
                .extend(p.source_offset + 6 + 6 * i as u32..p.source_offset + 12 + 6 * i as u32);
            program_fields.extend(pa..pa + bytes.len() as u32);
            command_count += 1;
        }
    }
    assert_eq!(command_count, 1135);
    assert_eq!(program_fields.len(), 18221);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<PalettePrograms>(&encoded).unwrap(),
        *data
    );
    data.verify_source(&rom).unwrap();
    for case in 0..11 {
        let mut bad = data.clone();
        match case {
            0 => {
                bad.programs.pop();
            }
            1 => bad.programs[0].source_offset += 1,
            2 => bad.programs[0].header.command_count = 0,
            3 => bad.programs[0].header.command_count += 1,
            4 => bad.programs[0].commands[0].parameter = Parameter::Byte { value: 1 },
            5 => bad.programs[0].commands[0].opcode = 255,
            6 => {
                bad.producers.pop();
            }
            7 => bad.producers[0].source_offset += 4,
            8 => {
                if let ProducerData::List { entries } = &mut bad.producers[0].data {
                    entries[0].program = 65535;
                }
            }
            9 => {
                if let ProducerData::Pointers { programs } = &mut bad.producers[3].data {
                    programs[2] = Some(0);
                }
            }
            _ => {
                let (pi, ci) = bad
                    .programs
                    .iter()
                    .enumerate()
                    .find_map(|(pi, p)| {
                        p.commands
                            .iter()
                            .position(|c| !c.parameter.source_bytes().is_empty())
                            .map(|ci| (pi, ci))
                    })
                    .unwrap();
                bad.programs[pi].commands[ci].relative_parameter = 0;
            }
        }
        assert!(bad.validate().is_err(), "shape guard {case}");
    }
    let mut changed = data.clone();
    changed.programs[0].header.initial_delay ^= 1;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    let mut unknown = serde_json::to_value(data).unwrap();
    unknown["unknown"] = true.into();
    assert!(serde_json::from_value::<PalettePrograms>(unknown).is_err());
    let mut unknown = serde_json::to_value(&data.programs[0].commands[0].parameter).unwrap();
    unknown["unknown"] = true.into();
    assert!(serde_json::from_value::<Parameter>(unknown).is_err());
    assert!(Catalog::from_rom(&[]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());

    let mut m = Machine::new(rom.clone());
    let mut reads = OriginalReads::default();
    for p in &data.programs {
        reset(&mut m, 0x080d7fac);
        put32(&mut m, MESSAGE + 4, 0x08000000 + p.source_offset);
        put16(&mut m, MESSAGE + 20, 0x81);
        m.cpu.regs[1] = MESSAGE;
        m.cpu.regs[2] = CONTEXT;
        reads.until(&mut m, 0x080d80d8);
        assert_eq!(
            u16::from_le_bytes(m.bus.ewram[0x10004..0x10006].try_into().unwrap()),
            p.header.first_color()
        );
        assert_eq!(m.bus.ewram[0x10006], p.header.width);
        assert_eq!(m.bus.ewram[0x10007], 0x81);
        assert_eq!(
            u16::from_le_bytes(m.bus.ewram[0x10008..0x1000a].try_into().unwrap()),
            p.header.initial_delay
        );
        reset(&mut m, 0x080d80b2);
        m.cpu.regs[1] = 0x08000000 + p.source_offset;
        reads.one(&mut m);
        assert_eq!(m.cpu.regs[1], u32::from(p.header.command_count));
        for (i, c) in p.commands.iter().enumerate() {
            let queued = c.mode != 0 || matches!(c.opcode, 10 | 11);
            reset(&mut m, 0x080d7f54);
            context(&mut m, p, i);
            let stop = if queued {
                match c.opcode {
                    0 => 0x0814b744,
                    1 => 0x080d7bd0,
                    4 => 0x080d7c52,
                    7 => 0x080d7cbc,
                    8 => 0x080d7cd0,
                    9 => 0x080d7ce4,
                    10 => 0x080d7d02,
                    11 => 0x080d7d24,
                    _ => panic!("uncovered queued opcode"),
                }
            } else {
                RETURN
            };
            reads.until(&mut m, stop);
            if queued && c.opcode == 0 {
                if let Parameter::ColorWords { colors } = &c.parameter {
                    for (n, color) in colors.iter().enumerate() {
                        let a = 0x3e68 + 12 * (usize::from(p.header.first_color()) + n);
                        assert_eq!(
                            u16::from_le_bytes(m.bus.iwram[a..a + 2].try_into().unwrap()),
                            *color
                        );
                    }
                }
                assert_eq!(m.cpu.regs[2], u32::from(c.duration));
            }
            if !queued {
                reset(&mut m, 0x080d7d34);
                context(&mut m, p, i);
                reads.until(
                    &mut m,
                    match c.opcode {
                        0 | 13 => 0x080d7f46,
                        4 => 0x080d7e42,
                        12 => 0x080d7f38,
                        _ => panic!("uncovered direct opcode"),
                    },
                );
                if let Parameter::ColorWords { colors } = &c.parameter {
                    let a = 0x3860 + 2 * usize::from(p.header.first_color());
                    assert_eq!(
                        m.bus.iwram[a..a + 2 * colors.len()],
                        c.parameter.source_bytes()
                    );
                }
                if let Parameter::Components { values } = &c.parameter {
                    assert_eq!(m.cpu.regs[2], i32::from(values[0]) as u32);
                    assert_eq!(m.cpu.regs[3], i32::from(values[1]) as u32);
                    let sp = (m.cpu.regs[13] - 0x03000000) as usize;
                    assert_eq!(
                        u32::from_le_bytes(m.bus.iwram[sp..sp + 4].try_into().unwrap()),
                        i32::from(values[2]) as u32
                    );
                }
            }
            if let Parameter::ColorCycle { count, colors, .. } = &c.parameter {
                assert_eq!(m.cpu.regs[2], u32::from(*count));
                // Source-loaded count is normalized by the original constructor.
                reset(&mut m, 0x0814aaa0);
                m.cpu.regs[4] = OBJECT;
                m.cpu.regs[6] = u32::from(*count);
                m.cpu.regs[7] = 0;
                reads.until(&mut m, 0x0814aab0);
                assert_eq!(m.cpu.regs[6], u32::from((*count).max(1)));
                assert_eq!(m.cpu.regs[7], 1);
                for (n, color) in colors.iter().enumerate() {
                    reset(&mut m, 0x0814a574);
                    m.cpu.regs[0] = OBJECT;
                    put32(
                        &mut m,
                        OBJECT + 0x1c,
                        0x08000000 + p.source_offset + u32::from(c.relative_parameter) + 2,
                    );
                    put16(&mut m, OBJECT + 0x20, (2 * n) as u16);
                    reads.until(&mut m, 0x0814a584);
                    assert_eq!(m.cpu.regs[7], u32::from(*color));
                }
            }
        }
    }
    assert_eq!(
        program_fields.intersection(&reads.bytes).count(),
        18221,
        "every declared program field must be read by the original code"
    );
    // Six boundary inputs exercise the original queued copy, including zero
    // width's u16 wrap. They do not install a new cap on guest state.
    for (first, flags, width) in [
        (0, 0, 0),
        (255, 2, 1),
        (255, 2, 2),
        (0, 2, 0),
        (255, 0, 255),
        (250, 2, 10),
    ] {
        let h = Header {
            first,
            flags,
            width,
            command_count: 1,
            initial_delay: 0,
        };
        reset(&mut m, 0x0814b6f0);
        m.cpu.regs[0] = u32::from(h.first_color());
        m.cpu.regs[1] = u32::from(width);
        m.cpu.regs[3] = 0x02013000;
        reads.until(&mut m, 0x0814b744);
        assert_eq!(m.cpu.regs[4] as usize, h.color_words(true));
        assert_eq!(h.color_words(false), usize::from(width));
    }
    // The count and period minima come from the constructor, not the model.
    for (count, period) in [(0, 0), (1, 1), (255, 65535)] {
        reset(&mut m, 0x0814aaa0);
        m.cpu.regs[4] = OBJECT;
        m.cpu.regs[6] = count;
        m.cpu.regs[7] = period;
        reads.until(&mut m, 0x0814aab0);
        assert_eq!(m.cpu.regs[6], count.max(1));
        assert_eq!(m.cpu.regs[7], period.max(1));
    }
    // Raw direction255 remains a byte in the file; the original call argument
    // normalizes it to1. An empty raw count still selects one color word.
    reset(&mut m, 0x080d7d08);
    m.cpu.regs[6] = CONTEXT;
    m.cpu.regs[5] = 0x02013000;
    m.cpu.regs[4] = 0x02014000;
    m.cpu.regs[7] = 0;
    m.bus.ewram[0x13000..0x13004].copy_from_slice(&[0, 255, 0x34, 0x92]);
    reads.until(&mut m, 0x080d7d24);
    assert_eq!(m.cpu.regs[2], 0);
    let sp = (m.cpu.regs[13] - 0x03000000) as usize;
    assert_eq!(
        u32::from_le_bytes(m.bus.iwram[sp..sp + 4].try_into().unwrap()),
        1
    );
    let raw = Parameter::ColorCycle {
        count: 0,
        direction: 255,
        colors: vec![0x9234],
    };
    assert_eq!(raw.source_bytes(), [0, 255, 0x34, 0x92]);
    // Original LDR at every indexed consumer, without allocator/task ABI assumptions.
    let mut indexed_cells = 0;
    for row in indexed["rows"].as_array().unwrap() {
        let call = number(&row["call_pc"]);
        let base = number(&row["source_offset"]);
        let pc = (call - 20..call)
            .step_by(2)
            .rev()
            .find(|pc| {
                let a = (*pc - 0x08000000) as usize;
                let op = u16::from_le_bytes(rom[a..a + 2].try_into().unwrap());
                op & 0xf800 == 0x6800 && op & 7 == 0
            })
            .unwrap();
        let a = (pc - 0x08000000) as usize;
        let op = u16::from_le_bytes(rom[a..a + 2].try_into().unwrap());
        for (i, word) in row["words"].as_array().unwrap().iter().enumerate() {
            reset(&mut m, pc);
            m.cpu.regs[usize::from((op >> 3) & 7)] =
                0x08000000 + base + 4 * i as u32 - u32::from((op >> 6) & 31) * 4;
            reads.one(&mut m);
            assert_eq!(m.cpu.regs[0], number(word));
            indexed_cells += 1;
        }
    }
    assert_eq!(indexed_cells, 97);
    for (at, pc) in [(0x14d680, 0x080e7a90), (0x14d68c, 0x080e7a9c)] {
        reset(&mut m, pc);
        m.cpu.regs[0] = 0x08000000 + at;
        reads.one(&mut m);
    }
    let mut list_rows = 0;
    for producer in &data.producers {
        if let ProducerData::List { entries } = &producer.data {
            for (i, e) in entries.iter().enumerate() {
                reset(&mut m, 0x080d83a0);
                m.cpu.regs[4] = 0x08000000 + producer.source_offset + 8 * i as u32;
                reads.one(&mut m);
                assert_eq!(m.cpu.regs[2], data.program_word(e.program).unwrap());
                m.cpu.regs[15] = 0x080d83be;
                reads.one(&mut m);
                assert_eq!(m.cpu.regs[0], u32::from(e.delay_override));
                list_rows += 1;
            }
        }
    }
    assert_eq!(list_rows, 8);
    assert_eq!(
        declared.intersection(&reads.bytes).count(),
        18681,
        "16 reserved list bytes are preserved but never read by this producer"
    );
    fs::write(root.join("palette-programs.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "programs":230,"commands":1135,"producer_tables":25,"indexed_cells":97,"list_rows":8,"stack_tables":2,
        "source_bytes":18697,"source_ranges":165,"distinct_program_bytes_observed":18221,"distinct_all_source_bytes_observed":18681,
        "unread_reserved_list_bytes_preserved":16,"lossless_fields_independent_hash_json_shape_source_guards":true,
        "original_instructions_coverage":reads.instructions,"original_pcs_coverage":reads.pcs.len(),
        "scope":"original header/start/command/parameter/copy/color-load/producer prefixes; no whole allocator/callback/native/stateclock/ordinary claim",
        "managed":"EUR e45-d44 unchanged; USAe8-d7","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
