//! Isolated original-function probes; no ordinary checkpoint is mutated.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

const STACK: u32 = 0x03007d00;
const STOP: u32 = 0x03007f00;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    label: String,
    index: u32,
    selector: u32,
    replacement: Option<Vec<u8>>,
    expected: u32,
    expected_query: Option<u32>,
    expected_reads: Vec<u32>,
}

fn original_getter(
    m: &mut Machine,
    data: &Records70,
    case: &Case,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    let (ewram, mut iwram, vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    let scratch = (STACK - 0x03000000 - 4) as usize;
    iwram[scratch..scratch + 4].copy_from_slice(&(STOP | 1).to_le_bytes());
    if matches!(case.selector as u8, 42 | 44 | 53) {
        iwram[scratch - 4..scratch].copy_from_slice(&0x080d2255u32.to_le_bytes());
    }
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[0] = case.index;
    m.cpu.regs[1] = case.selector;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = STOP | 1;
    m.cpu.regs[15] = 0x080d1f30;
    m.cpu.cpsr = 0xbf;
    let mut reads = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != STOP {
        let pc = (m.cpu.regs[15] - 0x08000000) as usize;
        let instruction = u16::from_le_bytes([m.bus.rom[pc], m.bus.rom[pc + 1]]);
        // All original record loads are Thumb immediate LDRB. Observe the actual
        // effective address before executing, including the one-byte flag path.
        if instruction & 0xf800 == 0x7800 {
            let base = usize::from((instruction >> 3) & 7);
            let address = m.cpu.regs[base] + u32::from((instruction >> 6) & 31);
            assert!(
                (0x08000000..0x09000000).contains(&address),
                "{}",
                case.label
            );
            let offset = address - 0x08000000;
            reads.push(offset);
            if (SOURCE..SOURCE_END).contains(&offset) {
                assert_eq!(data.byte_at(offset), Some(m.bus.rom[offset as usize]));
                covered.insert(offset);
            } else {
                assert_eq!(data.byte_at(offset), None);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(
            instructions < 200,
            "{} PC {:08x}",
            case.label,
            m.cpu.regs[15]
        );
    }
    assert_eq!(
        reads, case.expected_reads,
        "{} actual read order",
        case.label
    );
    assert_eq!(
        m.cpu.regs[0], case.expected,
        "{} original result",
        case.label
    );
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(
        &m.cpu.regs[4..13],
        &(4..13).map(|i| 0x11000000 + i).collect::<Vec<_>>()
    );
    assert_ne!(m.cpu.cpsr & 0x20, 0);
    assert_eq!(m.bus.ewram, ewram, "{} all EWRAM", case.label);
    assert_eq!(
        m.bus.iwram, iwram,
        "{} exact return-link scratch",
        case.label
    );
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR mission-records70 getter; Wine only, Linux/USA pending"]
fn european_mission_records70_matches_source_and_original_getter() {
    let root = PathBuf::from(std::env::var_os("FFTA_MISSION_RECORDS70_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: Records70 =
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
    assert_eq!(
        spans,
        vec![(SOURCE, data.source_bytes(), "mission-records70".into())]
    );
    assert_eq!(ffta_rom::digest(&spans[0].1), SOURCE_SHA256);
    for (i, &byte) in spans[0].1.iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(byte));
    }
    for at in [0, SOURCE - 1, SOURCE_END, SOURCE_END + 1, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for i in [0, 1, 300, 511, 512, u16::MAX] {
        assert_eq!(data.record(i).is_some(), usize::from(i) < RECORD_COUNT);
    }
    for r in &data.records {
        assert_eq!(
            r.source_address(),
            0x08000000 + SOURCE + u32::from(r.index) * 70
        );
        assert_eq!(r.value(19), r.value(20));
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<Records70>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 70,
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
    for case in 0..12 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(0),
            1 => bad["records"][0]["extra"] = json!(0),
            2 => bad["records"][0]["index"] = json!(-1),
            3 => bad["records"][0]["index"] = json!(65536),
            4 => bad["records"][0]["word00"] = json!(65536),
            5 => bad["records"][0]["operand07"] = json!(256),
            6 => bad["records"][0]["packed02"] = json!([0, 1]),
            7 => bad["records"][0]["inline12"] = json!([0]),
            8 => bad["records"][0]["flags41_raw"] = json!(true),
            9 => bad["records"][0]["word43"] = json!(1.5),
            10 => {
                bad["records"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("mapped36_raw");
            }
            _ => bad["records"] = Value::Null,
        }
        assert!(serde_json::from_value::<Records70>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0xd1f30] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.records[0] = Record::decode(0, &[255; RECORD_BYTES]);
    changed.validate().unwrap();
    assert_eq!(changed.records[0].source_bytes(), [255; RECORD_BYTES]);
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
        let mut current = data.clone();
        if let Some(raw) = &case.replacement {
            assert_eq!(raw.len(), RECORD_BYTES);
            let index = case.index as u16;
            assert_eq!(index, 511);
            current.records[usize::from(index)] = Record::decode(index, raw);
            current.validate().unwrap();
            let at = SOURCE as usize + usize::from(index) * RECORD_BYTES;
            m.bus.rom[at..at + RECORD_BYTES].copy_from_slice(raw);
            synthetic += 1;
        } else {
            base += 1;
        }
        assert_eq!(
            current.query(case.index, case.selector),
            case.expected_query,
            "{} query",
            case.label
        );
        if let Some(row) = current.record(case.index as u16) {
            assert_eq!(row.value(case.selector), case.expected);
        }
        let bytes = if case.replacement.is_some() {
            &mut synthetic_bytes
        } else {
            &mut base_bytes
        };
        instructions += original_getter(&mut m, &current, case, bytes);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE_END as usize]);
    }
    assert_eq!(m.bus.rom, rom);
    assert!(base_bytes.len() > 70);
    assert!(synthetic_bytes.contains(&(SOURCE_END - 1)));
    assert!(!synthetic_bytes.contains(&(SOURCE_END - 70 + 18)));
    let report = json!({
        "status":"PASS-source-model-original-getter", "physical_records":RECORD_COUNT,
        "source_bytes":SOURCE_BYTES,"source_spans":1,"guards":guards,
        "base_getter_cases":base,"synthetic_getter_cases":synthetic,
        "actual_base_source_bytes_observed":base_bytes.len(),
        "actual_synthetic_source_bytes_observed":synthetic_bytes.len(),
        "instructions_coverage_only":instructions,
        "all_EWRAM_VRAM_OAM_palette_exact":true,"IWRAM_exact_including_expected_return_links":true,
        "SP_R4_R12_final_ROM_checked":true,
        "scope":"original D1F30 entry through return plus actual CEEEC helper; full caller ABI/nativeclock/ordinary/runtime excluded",
        "schema_extractor_decoder_unchanged":[1,85,84],"Linux":"pending","USA":"pending",
        "nativeWindows":"pending","performance_measurements":false
    });
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_MISSION_RECORDS70_MODEL records512/source35840B guards{guards} base{base} synthetic{synthetic} PASS");
}
