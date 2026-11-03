//! Isolated original-loop probes; no ordinary save/checkpoint fixture is edited.
use super::*;
use crate::{Bus, Machine};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

const OUTPUT: u32 = 0x02020000;
const STACK: u32 = 0x03007d00;
const LOOP: u32 = 0x080d56ce;
const STOP: u32 = 0x080d57a4;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    label: String,
    context: u32,
    capacity: u32,
    flag_fill: u8,
    counter_fill: u8,
    single: Option<Record>,
    expected: Vec<u32>,
    queries: Vec<(u8, u16)>,
}
struct Live {
    flag_fill: u8,
    counter_fill: u8,
    queries: Vec<(u8, u16)>,
}
impl LivePredicates for Live {
    fn flag(&mut self, key: u16) -> bool {
        self.queries.push((0, key));
        (self.flag_fill & (1 << (key & 7))) != 0
    }
    fn counter(&mut self, index: u16) -> u8 {
        self.queries.push((1, index));
        self.counter_fill
    }
}

fn original_loop(
    m: &mut Machine,
    data: &Records8,
    case: &Case,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.ewram[0x1f70..0x2030].fill(case.flag_fill);
    m.bus.ewram[0x2030..0x2030 + 0xfa00].fill(case.counter_fill);
    m.bus.iwram.fill(0x3c);
    m.bus.write32(STACK, case.context);
    let mut expected_ram = m.bus.ewram.clone();
    for (i, pointer) in case.expected.iter().enumerate() {
        let at = (OUTPUT - 0x02000000) as usize + i * 4;
        expected_ram[at..at + 4].copy_from_slice(&pointer.to_le_bytes());
    }
    let iwram = m.bus.iwram.clone();
    let (vram, oam, palette) = (m.bus.vram.clone(), m.bus.oam.clone(), m.bus.palette.clone());
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[4] = 0x12000000;
    m.cpu.regs[5] = 0x08000000 + SOURCE;
    m.cpu.regs[6] = case.capacity;
    m.cpu.regs[7] = OUTPUT;
    m.cpu.regs[8] = 1;
    m.cpu.regs[9] = 0;
    m.cpu.regs[10] = 0x05ff;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = LOOP;
    m.cpu.cpsr = 0xbf;
    let mut queries = Vec::new();
    let mut instructions = 0;
    let mut visited = BTreeSet::new();
    while m.cpu.regs[15] != STOP {
        let pc = m.cpu.regs[15];
        if pc == 0x080cce98 {
            queries.push((0, m.cpu.regs[0] as u16));
        }
        if pc == 0x080ccf00 {
            queries.push((1, m.cpu.regs[0] as u16));
        }
        let byte = match pc {
            0x080d56ce => {
                visited.insert(m.cpu.regs[5]);
                Some(0)
            }
            0x080d56d4 => Some(1),
            0x080d56e0 => Some(2),
            0x080d56e2 => Some(3),
            0x080d56fe | 0x080d5726 | 0x080d5738 => Some(4),
            0x080d573e => Some(5),
            0x080d5740 => Some(6),
            0x080d575c | 0x080d577a | 0x080d5784 => Some(7),
            _ => None,
        };
        if let Some(byte) = byte {
            let offset = m.cpu.regs[5] - 0x08000000 + byte;
            assert_eq!(
                data.byte_at(offset),
                Some(m.bus.rom[offset as usize]),
                "{}",
                case.label
            );
            covered.insert(offset);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(
            instructions < 20000,
            "{} unexpected PC {:08x}",
            case.label,
            m.cpu.regs[15]
        );
    }
    assert_eq!(queries, case.queries, "{} actual live calls", case.label);
    assert_eq!(
        m.cpu.regs[0],
        case.expected.len() as u32,
        "{} count",
        case.label
    );
    assert_eq!(m.cpu.regs[9], case.expected.len() as u32);
    assert_eq!(m.cpu.regs[7], OUTPUT + case.expected.len() as u32 * 4);
    assert_eq!(
        m.cpu.regs[6],
        case.capacity.wrapping_sub(case.expected.len() as u32)
    );
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(&m.cpu.regs[11..13], &[0x1100000b, 0x1100000c]);
    let visited_expected = if case.capacity != 0 && case.expected.len() as u32 == case.capacity {
        ((case.expected.last().unwrap() - 0x08000000 - SOURCE) / 8 + 1) as usize
    } else {
        RECORD_COUNT
    };
    assert_eq!(
        visited,
        (0..visited_expected)
            .map(|i| 0x08000000 + SOURCE + i as u32 * 8)
            .collect()
    );
    assert_eq!(
        m.bus.ewram, expected_ram,
        "{} output and all other EWRAM",
        case.label
    );
    // The real flag reader pushes its return link into one four-byte scratch word.
    let scratch = (STACK - 0x03000000 - 4) as usize;
    assert_eq!(&m.bus.iwram[..scratch], &iwram[..scratch]);
    assert_eq!(&m.bus.iwram[scratch + 4..], &iwram[scratch + 4..]);
    assert_eq!(
        (m.bus.vram.clone(), m.bus.oam.clone(), m.bus.palette.clone()),
        (vram, oam, palette)
    );
    instructions
}

#[test]
#[ignore = "private EUR conditional-records8 original loop; Wine only, Linux/USA pending"]
fn european_conditional_records8_matches_source_and_original_loop() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONDITIONAL_RECORDS8_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: Records8 =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE as usize..SOURCE_END as usize]
    );
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].0, SOURCE);
    assert_eq!(spans[0].1, data.source_bytes());
    assert_eq!(spans[0].2, "conditional-records8-slots");
    assert_eq!(ffta_rom::digest(&spans[0].1), SOURCE_SHA256);
    for (i, &byte) in spans[0].1.iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(byte));
    }
    for at in [0, SOURCE - 1, SOURCE_END, SOURCE_END + 1, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for i in 0..=u8::MAX {
        assert_eq!(data.record(i).is_some(), usize::from(i) < RECORD_COUNT);
    }
    for r in &data.records {
        assert_eq!(
            r.source_address(),
            0x08000000 + SOURCE + u32::from(r.index) * 8
        );
    }
    assert_eq!(data.records.iter().filter(|r| r.field00 == 0).count(), 51);
    for r in data.records.iter().filter(|r| r.field00 == 0) {
        assert_eq!(r.source_bytes(), [0; 8]);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(serde_json::from_slice::<Records8>(&encoded).unwrap(), *data);
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 8,
            1 => {
                bad.records.pop();
            }
            2 => bad.records.push(bad.records[0].clone()),
            3 => bad.records.swap(0, 1),
            _ => bad.records[1].index = 0,
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..15 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(0),
            1 => bad["records"][0]["extra"] = json!(0),
            2 => bad["records"][0]["index"] = json!(-1),
            3 => bad["records"][0]["index"] = json!(256),
            4 => bad["records"][0]["field00"] = json!(256),
            5 => bad["records"][0]["field01"] = json!(-1),
            6 => bad["records"][0]["key02"] = json!(65536),
            7 => bad["records"][0]["key05"] = json!(-1),
            8 => bad["records"][0]["operand04"] = json!(256),
            9 => bad["records"][0]["operand07"] = json!(-1),
            10 => bad["records"][0]["key05"] = json!(1.5),
            11 => bad["records"][0]["key02"] = json!(true),
            12 => bad["records"][0]["operand07"] = json!("2"),
            13 => {
                bad["records"][0].as_object_mut().unwrap().remove("key02");
            }
            _ => {
                bad.as_object_mut().unwrap().remove("records");
            }
        }
        assert!(serde_json::from_value::<Records8>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0xd56ce] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.records[0].field01 = 255;
    changed.records[0].operand04 = 128;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let cases: Vec<Case> =
        serde_json::from_slice(&fs::read(root.join("independent-cases.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    let (mut base, mut synthetic) = (0, 0);
    let (mut base_bytes, mut synthetic_bytes) = (BTreeSet::new(), BTreeSet::new());
    let mut instructions = 0;
    for case in &cases {
        let current = if let Some(record) = &case.single {
            let mut current = data.clone();
            for r in &mut current.records {
                *r = Record {
                    index: r.index,
                    field00: 0,
                    field01: 0,
                    key02: 0,
                    operand04: 0,
                    key05: 0,
                    operand07: 0,
                };
            }
            current.records[usize::from(record.index)] = record.clone();
            current.validate().unwrap();
            synthetic += 1;
            current
        } else {
            base += 1;
            data.clone()
        };
        m.bus.rom[SOURCE as usize..SOURCE_END as usize].copy_from_slice(&current.source_bytes());
        let mut live = Live {
            flag_fill: case.flag_fill,
            counter_fill: case.counter_fill,
            queries: Vec::new(),
        };
        assert_eq!(
            current.select(case.context, case.capacity, &mut live),
            case.expected,
            "{} model selection",
            case.label
        );
        assert_eq!(
            live.queries, case.queries,
            "{} model live calls",
            case.label
        );
        let bytes = if case.single.is_some() {
            &mut synthetic_bytes
        } else {
            &mut base_bytes
        };
        instructions += original_loop(&mut m, &current, case, bytes);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE_END as usize]);
    }
    assert_eq!(m.bus.rom, rom);
    assert!(base_bytes.len() > RECORD_COUNT);
    assert!((SOURCE_END - 8..SOURCE_END).all(|at| synthetic_bytes.contains(&at)));
    let report = json!({"status":"PASS-source-model-original-loop","physical_slots":RECORD_COUNT,"source_bytes":SOURCE_BYTES,
        "source_spans":1,"guards":guards,"base_loop_cases":base,"synthetic_loop_cases":synthetic,
        "actual_base_source_bytes_observed":base_bytes.len(),"actual_synthetic_source_bytes_observed":synthetic_bytes.len(),
        "instructions_coverage_only":instructions,"all_ewram_exact_except_expected_output":true,
        "iwram_exact_except_original_flag_reader_scratch4":true,"stack_VRAM_OAM_palette_final_ROM_checked":true,
        "scope":"original D56CE..D57A4 loop and actual flag/counter callees; entry clear helper/epilogue/fullABI/nativeclock/ordinary/runtime excluded",
        "schema_extractor_decoder_unchanged":[1,84,83],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_MODEL slots127/source1016B guards{guards} base{base} synthetic{synthetic} PASS");
}
