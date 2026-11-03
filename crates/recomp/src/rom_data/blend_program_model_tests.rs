//! One EUR source/model oracle; managed/native/ordinary acceptance is separate.
use super::*;
use gba_core::{rom_data::blend_programs::*, Machine};
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
            "Original blend prefix did not reach {stop:08x}, pc{:08x}",
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

fn half(m: &Machine, a: u32) -> u16 {
    let i = (a - 0x02000000) as usize;
    u16::from_le_bytes(m.bus.ewram[i..i + 2].try_into().unwrap())
}
fn byte(m: &mut Machine, a: u32, value: u8) {
    m.bus.ewram[(a - 0x02000000) as usize] = value;
}
fn context(m: &mut Machine, ptr: u32, index: u16, mode: u8, first: u8, second: u8) {
    put32(m, CONTEXT, ptr);
    put16(m, CONTEXT + 12, index);
    byte(m, CONTEXT + 5, mode);
    byte(m, CONTEXT + 8, first);
    byte(m, CONTEXT + 9, second);
    m.cpu.regs[0] = CONTEXT;
}
fn header(m: &mut Machine, reads: &mut OriginalReads, ptr: u32, delay: u16) {
    put32(m, MESSAGE + 4, ptr);
    put16(m, MESSAGE + 20, 0x1a5);
    put16(m, MESSAGE + 22, delay);
    m.cpu.regs[1] = MESSAGE;
    m.cpu.regs[2] = CONTEXT;
    reads.until(m, 0x080d7846);
}
fn interpolation(m: &Machine, at: u32, first: u8, target: u8, duration: u16) {
    let delta = i16::from(target) - i16::from(first);
    assert_eq!(half(m, at), u16::from(first));
    assert_eq!(half(m, at + 2), duration);
    assert_eq!(half(m, at + 4), ((duration as i16) >> 1) as u16);
    assert_eq!(half(m, at + 6), if delta < 0 { u16::MAX } else { 1 });
    assert_eq!(half(m, at + 8), duration);
    assert_eq!(half(m, at + 10), delta.unsigned_abs());
}
fn n(v: &Value) -> u32 {
    v.as_u64().unwrap().try_into().unwrap()
}

#[test]
#[ignore = "private EUR blend decoder/original-reader prefixes; Wine only, Linux/USA pending"]
fn european_blend_model_matches_original_source_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_BLEND_PROGRAM_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_BLEND_PROGRAM_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let cat = Catalog::from_rom(&rom).unwrap();
    let data = &cat.decoded;
    assert_eq!(data.programs.len(), 160);
    assert_eq!(data.producers.len(), 6);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 158);
    let mut declared = BTreeSet::new();
    for (((at, bytes, _), source), e) in spans
        .iter()
        .zip(&cat.sources)
        .zip(evidence["source_ranges"].as_array().unwrap())
    {
        assert_eq!(*at, n(&e["offset"]));
        assert_eq!(bytes.len(), n(&e["length"]) as usize);
        assert_eq!(*bytes, rom[*at as usize..*at as usize + bytes.len()]);
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            e["sha256"].as_str().unwrap()
        );
        assert_eq!(source.offset, *at);
        assert_eq!(source.length as usize, bytes.len());
        assert_eq!(source.sha256, e["sha256"].as_str().unwrap());
        declared.extend(*at..*at + bytes.len() as u32);
    }
    assert_eq!(declared.len(), 2428);
    let mut commands = 0;
    let mut unread = BTreeSet::new();
    for (p, e) in data
        .programs
        .iter()
        .zip(evidence["programs"].as_array().unwrap())
    {
        assert_eq!(p.source_offset, n(&e["offset"]));
        assert_eq!(u32::from(p.header.control.word()), n(&e["control_raw"]));
        assert_eq!(
            u32::from(p.header.initial.word()),
            n(&e["coefficients_raw"])
        );
        assert_eq!(u32::from(p.header.command_count), n(&e["command_count"]));
        for (i, (c, e)) in p
            .commands
            .iter()
            .zip(e["commands"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(u32::from(c.duration), n(&e["duration_raw"]));
            assert_eq!(u32::from(c.first), n(&e["first"]));
            assert_eq!(u32::from(c.second), n(&e["second"]));
            if p.header.control.mode != 1 && c.duration != u16::MAX {
                unread.insert(p.source_offset + 9 + 4 * i as u32);
            }
            commands += 1;
        }
        assert!(p.byte(6 + 4 * p.commands.len()).is_none());
    }
    assert_eq!(commands, 345);
    assert_eq!(unread.len(), 52);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<BlendPrograms>(&encoded).unwrap(),
        *data
    );
    data.verify_source(&rom).unwrap();
    for case in 0..17 {
        let mut bad = data.clone();
        match case {
            0 => {
                bad.programs.pop();
            }
            1 => bad.programs[0].source_offset += 1,
            2 => bad.programs[0].header.command_count = 0,
            3 => bad.programs[0].commands.clear(),
            4 => bad.programs[0].header.control.first_targets = 64,
            5 => bad.programs[0].header.control.mode = 4,
            6 => bad.programs[0].header.control.second_targets = 64,
            7 => bad.programs[0].header.control.reserved = 4,
            8 => bad.programs[0].header.initial.first = 32,
            9 => bad.programs[0].header.initial.second = 32,
            10 => bad.programs[0].header.initial.first_reserved = 8,
            11 => bad.programs[0].header.initial.second_reserved = 8,
            12 => {
                bad.producers.pop();
            }
            13 => bad.producers[0].source_offset += 4,
            14 => {
                bad.producers[0].programs.pop();
            }
            15 => bad.producers[0].programs[0] = u16::MAX,
            16 => bad.programs[0].header.command_count += 1,
            _ => unreachable!(),
        }
        assert!(bad.validate().is_err(), "shape guard{case}");
    }
    for case in 0..3 {
        let mut bad = data.clone();
        match case {
            0 => bad.programs[0].header.control.reserved ^= 1,
            1 => bad.programs[0].commands[0].duration ^= 1,
            2 => bad.producers[0].programs[0] ^= 1,
            _ => unreachable!(),
        }
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    let mut unknown = serde_json::to_value(data).unwrap();
    unknown["unknown"] = true.into();
    assert!(serde_json::from_value::<BlendPrograms>(unknown).is_err());
    let mut unknown = serde_json::to_value(&data.programs[0].header.control).unwrap();
    unknown["unknown"] = true.into();
    assert!(serde_json::from_value::<Control>(unknown).is_err());
    assert!(Catalog::from_rom(&[]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    let mut m = Machine::new(rom.clone());
    let mut reads = OriginalReads::default();
    let mut instant = 0;
    for p in &data.programs {
        reset(&mut m, 0x080d77c4);
        header(&mut m, &mut reads, 0x08000000 + p.source_offset, 0xffff);
        assert_eq!(
            &m.bus.ewram[0x10000..0x10004],
            &(0x08000000 + p.source_offset).to_le_bytes()
        );
        assert_eq!(m.bus.ewram[0x10004], 0xa5);
        assert_eq!(m.bus.ewram[0x10005], p.header.control.mode);
        assert_eq!(m.bus.ewram[0x10006], p.header.control.word() as u8);
        assert_eq!(m.bus.ewram[0x10007], p.header.control.second_targets);
        assert_eq!(m.bus.ewram[0x10008], p.header.initial.first);
        assert_eq!(m.bus.ewram[0x10009], p.header.initial.second);
        assert_eq!(half(&m, CONTEXT + 10), 0);
        reset(&mut m, 0x080d794c);
        put32(&mut m, CONTEXT, 0x08000000 + p.source_offset);
        m.cpu.regs[5] = CONTEXT;
        m.cpu.regs[0] = 65535;
        reads.until(&mut m, 0x080d795c);
        assert_eq!(m.cpu.regs[1], u32::from(p.header.command_count));
        assert_eq!(m.cpu.regs[0], 65535);
        for (i, c) in p.commands.iter().enumerate() {
            let first = c.first ^ 1;
            let second = c.second ^ 1;
            reset(&mut m, 0x080d76e0);
            context(
                &mut m,
                0x08000000 + p.source_offset,
                i as u16,
                p.header.control.mode,
                first,
                second,
            );
            if c.duration == u16::MAX {
                reads.until(&mut m, 0x080d7706);
                assert_eq!(m.bus.ewram[0x10008], c.first);
                assert_eq!(m.bus.ewram[0x10009], c.second);
                instant += 1;
                // Resume the actual instant chain into its following timed command.
                reads.until(&mut m, RETURN);
                let next = p.commands[i..]
                    .iter()
                    .position(|c| c.duration != u16::MAX)
                    .unwrap()
                    + i;
                assert_eq!(half(&m, CONTEXT + 12), next as u16);
            } else {
                reads.until(&mut m, RETURN);
                interpolation(&m, CONTEXT + 20, first, c.first, c.duration);
                if p.header.control.mode == 1 {
                    interpolation(&m, CONTEXT + 32, second, c.second, c.duration);
                } else {
                    assert_eq!(half(&m, CONTEXT + 16), c.duration);
                }
            }
        }
    }
    assert_eq!(instant, 5);
    let mut cells = 0;
    for (table_index, (table, row)) in data
        .producers
        .iter()
        .zip(evidence["producer_tables"].as_array().unwrap())
        .enumerate()
    {
        assert_eq!(table.source_offset, n(&row["offset"]));
        let pc = n(&row["proof"].as_array().unwrap().last().unwrap()["pc"]);
        let at = (pc - 0x08000000) as usize;
        let op = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        assert_eq!(op & 0xf800, 0x6800);
        for (i, id) in table.programs.iter().enumerate() {
            let expected = data.program_word(*id).unwrap();
            assert_eq!(expected, n(&row["words"][i]));
            reset(&mut m, pc);
            m.cpu.regs[usize::from((op >> 3) & 7)] =
                0x08000000 + table.source_offset + 4 * i as u32;
            reads.one(&mut m);
            assert_eq!(m.cpu.regs[usize::from(op & 7)], expected);
            cells += 1;
            for byte in 0..4 {
                assert_eq!(
                    data.producer_byte(table_index, 4 * i + byte),
                    Some(expected.to_le_bytes()[byte])
                );
            }
        }
    }
    assert_eq!(cells, 22);
    // Original copied-IWRAM memcpy followed by the real RAM duration override.
    reset(&mut m, 0x080e2ad8);
    m.cpu.regs[4] = OBJECT;
    m.cpu.regs[5] = CONTEXT;
    put16(&mut m, OBJECT + 2, 0x8123);
    reads.until(&mut m, 0x080e2aec);
    let mut template = rom[0x39a5f0..0x39a5fa].to_vec();
    template[6..8].copy_from_slice(&0x8123u16.to_le_bytes());
    assert_eq!(m.bus.ewram[0x10010..0x1001a], template);
    m.cpu.regs[15] = 0x080e2af8;
    m.cpu.regs[3] = OBJECT;
    reads.until(&mut m, 0x080e2afe);
    assert_eq!(m.cpu.regs[0], CONTEXT + 16);
    assert_eq!(m.cpu.regs[1], OBJECT);
    // Packed masks retain every reserved bit in the file; original initial fields
    // are masked to31, not16. Mode0 also remains valid as raw file structure.
    for control in [0xffff, 0x3f3f] {
        let h = Header {
            control: Control::from_word(control),
            initial: Coefficients::from_word(0xffff),
            command_count: 1,
        };
        assert_eq!(
            &h.bytes()[..4],
            &[control as u8, (control >> 8) as u8, 255, 255]
        );
        let mut maximal = data.clone();
        maximal.programs[0].header = h.clone();
        maximal.validate().unwrap();
        assert_eq!(
            serde_json::from_slice::<BlendPrograms>(&serde_json::to_vec(&maximal).unwrap())
                .unwrap(),
            maximal
        );
        reset(&mut m, 0x080d77c4);
        m.bus.ewram[0x12000..0x12006].copy_from_slice(&h.bytes());
        header(&mut m, &mut reads, OBJECT, 0);
        assert_eq!(m.bus.ewram[0x10005], h.control.mode);
        assert_eq!(m.bus.ewram[0x10008], 31);
        assert_eq!(m.bus.ewram[0x10009], 31);
    }
    for duration in [0u16, 1, 0x7fff, 0x8000, 0xfffe] {
        reset(&mut m, 0x080d76e0);
        context(&mut m, OBJECT, 0, 1, 10, 14);
        let c = Command {
            duration,
            first: 11,
            second: 13,
        };
        m.bus.ewram[0x12006..0x1200a].copy_from_slice(&c.bytes());
        reads.until(&mut m, RETURN);
        interpolation(&m, CONTEXT + 20, 10, 11, duration);
        interpolation(&m, CONTEXT + 32, 14, 13, duration);
    }
    reset(&mut m, 0x080e4118);
    m.cpu.regs[0] = CONTEXT + 20;
    m.cpu.regs[1] = 10;
    m.cpu.regs[2] = 11;
    m.cpu.regs[3] = 65535;
    reads.until(&mut m, RETURN);
    interpolation(&m, CONTEXT + 20, 10, 11, 65535);
    for delay in [0u16, 1, 65535] {
        reset(&mut m, 0x080d77c4);
        header(
            &mut m,
            &mut reads,
            0x08000000 + data.programs[0].source_offset,
            delay,
        );
        assert_eq!(half(&m, CONTEXT + 10), delay.wrapping_add(1));
        m.cpu.regs = [0; 16];
        m.cpu.regs[13] = 0x03007e00;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = 0x080d77c4;
        m.cpu.regs[1] = MESSAGE;
        m.cpu.regs[2] = CONTEXT;
        put16(&mut m, MESSAGE, 0x100);
        reads.until(&mut m, if delay == 0 { 0x080d7856 } else { RETURN });
        assert_eq!(half(&m, CONTEXT + 10), delay);
    }
    reset(&mut m, 0x080d76e0);
    context(
        &mut m,
        0x08000000 + data.programs[0].source_offset,
        65535,
        1,
        0,
        0,
    );
    reads.until(&mut m, 0x080d76f8);
    let raw = data.programs[0].source_offset + 6 + 4 * 65535;
    assert!(!declared.contains(&raw));
    assert_eq!(
        m.cpu.regs[0],
        u32::from(u16::from_le_bytes(
            rom[raw as usize..raw as usize + 2].try_into().unwrap()
        ))
    );
    let actual_unread: BTreeSet<_> = declared.difference(&reads.bytes).copied().collect();
    assert_eq!(
        actual_unread, unread,
        "only nonalpha timed second-target fields are preserved but unread"
    );
    assert_eq!(declared.intersection(&reads.bytes).count(), 2376);
    fs::write(root.join("blend-programs.json"), &encoded).unwrap();
    let result = serde_json::json!({"status":"pass source/model/original-reader prefixes only", "programs":160,"commands":345,"producer_cells":cells,"source_bytes":2428,"observed_source_bytes":2376,"preserved_unread_nonalpha_second_bytes":52,"ranges":158,"shape_guards":17,"source_edit_guards":3,"strict_json_guards":2,"packed_header_cases":2,"duration_cases":6,"delay_cases":3,"instant_chains":instant,"ram_template_override":true,"raw_index65535":true,"original_instructions":reads.instructions,"original_pcs":reads.pcs.len(),"guest_clock_native_wholeabi_ordinary":"pending","linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("BLEND_PROGRAM_MODEL_RESULT {result}");
}
