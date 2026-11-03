//! Isolated original load/arithmetic prefixes, without live RNG/helper execution.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    label: String,
    stage: u8,
    index: u32,
    count: u8,
    sample: i16,
    helper: u32,
    patch: Option<Vec<u8>>,
    expected: Vec<u32>,
    owned: Option<Vec<u32>>,
    reads: Vec<u32>,
}
fn projection(data: &WorldRecords38, c: &Case) -> Option<Vec<u32>> {
    match c.stage {
        7 => return Some(vec![WorldRecords38::encoded_u8_view(c.index)]),
        8 => return Some(vec![WorldRecords38::encoded_u32_view(c.index)]),
        9 => return Some(vec![WorldRecords38::encoded_update_view(c.index)]),
        10 => return Some(vec![u32::from(c.count.min(30))]),
        _ => (),
    }
    let r = data.record(c.index)?;
    Some(match c.stage {
        0 => vec![u32::from(r.byte01)],
        1 => vec![u32::from(r.byte01_divisor())],
        2 => vec![u32::from(r.byte01_after_helper(c.helper))],
        3 => vec![u32::from(r.value02())],
        4 => vec![u32::from(r.byte02)],
        5 => vec![u32::from(r.low10_value04())],
        6 => vec![
            u32::from(r.threshold_for_live_count(c.count)),
            (100 - i32::from(r.threshold_for_live_count(c.count))) as u32,
            if r.threshold_branch_ge(c.count, c.sample) {
                0x080d36a0
            } else {
                0x080d3678
            },
        ],
        _ => panic!("unknown stage"),
    })
}
fn original(
    m: &mut Machine,
    data: &WorldRecords38,
    c: &Case,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007d00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.cpsr = 0xbf;
    let address = WorldRecords38::encoded_u32_view(c.index);
    let record = data.record(c.index);
    let (entry, stops): (u32, &[u32]) = match c.stage {
        0 => {
            m.cpu.regs[3] = address;
            (0xd34aa, &[0xd34ac])
        }
        1 => {
            m.cpu.regs[4] = u32::from(record.unwrap().byte01) << 24;
            (0xd34b8, &[0xd34bc])
        }
        2 => {
            m.cpu.regs[5] = u32::from(record.unwrap().byte01);
            m.cpu.regs[0] = c.helper;
            (0xd34c0, &[0xd34cc])
        }
        3 => {
            m.cpu.regs[7] = address;
            (0xd35d8, &[0xd35e0])
        }
        4 => {
            m.cpu.regs[7] = address;
            (0xd3684, &[0xd3686])
        }
        5 => {
            m.cpu.regs[4] = address;
            m.cpu.regs[7] = 0x02000100;
            (0xd4dfe, &[0xd4e1e])
        }
        6 => {
            m.cpu.regs[7] = address;
            m.cpu.regs[4] = 100;
            m.cpu.regs[0] = i32::from(c.sample) as u32;
            m.bus.iwram[0x7d00..0x7d04].copy_from_slice(&u32::from(c.count.min(30)).to_le_bytes());
            (0xd366c, &[0xd3678, 0xd36a0])
        }
        7 => {
            m.cpu.regs[0] = c.index;
            (0xd347e, &[0xd3496])
        }
        8 => {
            m.cpu.regs[5] = c.index;
            (0xd4dce, &[0xd4de4])
        }
        9 => {
            m.cpu.regs[8] = c.index;
            (0xd3582, &[0xd358c])
        }
        10 => {
            m.bus.iwram[0x7d00..0x7d04].copy_from_slice(&u32::from(c.count).to_le_bytes());
            (0xd3566, &[0xd3570])
        }
        _ => panic!("unknown stage"),
    };
    let (mut ewram, mut iwram, vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    let mut preserved = m.cpu.regs[8..13].to_vec();
    if c.stage == 8 {
        preserved[1] = 0x02002cb4;
    }
    if c.stage == 5 {
        ewram[0x100..0x102].copy_from_slice(&(c.expected[0] as u16).to_le_bytes());
    }
    if c.stage == 10 {
        iwram[0x7d00..0x7d04].copy_from_slice(&c.expected[0].to_le_bytes());
    }
    m.cpu.regs[15] = 0x08000000 + entry;
    let mut reads = Vec::new();
    let mut instructions = 0;
    while !stops.iter().any(|s| m.cpu.regs[15] == 0x08000000 + s) {
        let pc = m.cpu.regs[15] - 0x08000000;
        let load = match pc {
            0xd34aa => Some((3, 1)),
            0xd35d8 => Some((7, 2)),
            0xd35da => Some((7, 3)),
            0xd3684 => Some((7, 2)),
            0xd4dfe => Some((4, 4)),
            0xd4e00 => Some((4, 5)),
            0xd3670 => Some((1, 7)),
            _ => None,
        };
        if let Some((reg, off)) = load {
            let at = m.cpu.regs[reg] + off - 0x08000000;
            assert_eq!(data.byte_at(at), Some(m.bus.rom[at as usize]));
            reads.push(at);
            covered.insert(at);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 64, "{} PC{:08x}", c.label, m.cpu.regs[15]);
    }
    let got = match c.stage {
        0 | 1 => vec![m.cpu.regs[4]],
        2 => vec![m.cpu.regs[5]],
        3 | 4 => vec![m.cpu.regs[1]],
        5 => vec![u32::from(u16::from_le_bytes(
            m.bus.ewram[0x100..0x102].try_into().unwrap(),
        ))],
        6 => vec![m.cpu.regs[1], m.cpu.regs[4], m.cpu.regs[15]],
        7 => vec![m.cpu.regs[3]],
        8 => vec![m.cpu.regs[4]],
        9 => vec![m.cpu.regs[7]],
        10 => vec![u32::from_le_bytes(
            m.bus.iwram[0x7d00..0x7d04].try_into().unwrap(),
        )],
        _ => unreachable!(),
    };
    assert_eq!(got, c.expected, "{} original result", c.label);
    assert_eq!(reads, c.reads, "{} load order", c.label);
    assert_eq!(m.cpu.regs[13], 0x03007d00);
    assert_eq!(&m.cpu.regs[8..13], preserved);
    assert_eq!(m.bus.ewram, ewram);
    assert_eq!(m.bus.iwram, iwram);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR world-record source/original-prefix proof; Wine only, Linux/USA pending"]
fn european_world_records38_match_source_and_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_WORLD_RECORDS38_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: WorldRecords38 =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE as usize..SOURCE_END as usize]
    );
    for (i, &b) in data.source_bytes().iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(b));
    }
    for i in [31, 63, 255, 256, u32::MAX] {
        assert_eq!(data.record(i), None);
    }
    for at in [0, SOURCE - 1, SOURCE_END, SOURCE_END + 1, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<WorldRecords38>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 1,
            1 => {
                bad.records.pop();
            }
            2 => bad.records.swap(0, 1),
            3 => bad.records[0].index = 31,
            _ => bad.records.push(bad.records[0].clone()),
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..10 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(0),
            1 => bad["records"][0]["extra"] = json!(0),
            2 => {
                bad["records"][0].as_object_mut().unwrap().remove("byte01");
            }
            3 => bad["records"][0]["byte01"] = json!(-1),
            4 => bad["records"][0]["byte02"] = json!(256),
            5 => bad["records"][0]["value04"] = json!(65536),
            6 => bad["records"][0]["thresholds07"] = json!(vec![0; 30]),
            7 => bad["records"][0]["thresholds07"] = json!(vec![0; 32]),
            8 => bad["records"][0]["opaque06"] = json!(true),
            _ => bad["records"][0]["index"] = json!(256),
        }
        assert!(serde_json::from_value::<WorldRecords38>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0xd347e] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.records[0].opaque00 = 255;
    changed.records[0].thresholds07[30] = 255;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let cases: Vec<Case> =
        serde_json::from_slice(&fs::read(root.join("independent-cases.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    let (mut base, mut synthetic, mut instructions) = (0, 0, 0);
    let (mut base_bytes, mut synthetic_bytes) = (BTreeSet::new(), BTreeSet::new());
    for c in &cases {
        let mut current = data.clone();
        if let Some(raw) = &c.patch {
            assert_eq!(raw.len(), 38);
            current.records[c.index as usize] = Record::decode(c.index as u8, raw);
        }
        current.validate().unwrap();
        assert_eq!(projection(&current, c), c.owned, "{} model result", c.label);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize].copy_from_slice(&current.source_bytes());
        let covered = if c.patch.is_some() {
            synthetic += 1;
            &mut synthetic_bytes
        } else {
            base += 1;
            &mut base_bytes
        };
        instructions += original(&mut m, &current, c, covered);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE_END as usize]);
    }
    assert_eq!(m.bus.rom, rom);
    let report = json!({"status":"PASS-source-model-original-prefixes","guards":guards,"records":31,"source_bytes":1178,"source_spans":1,"base_prefix_cases":base,"synthetic_prefix_cases":synthetic,
        "actual_base_source_bytes_observed":base_bytes.len(),"actual_synthetic_source_bytes_observed":synthetic_bytes.len(),"instructions_coverage_only":instructions,
        "actual_load_order_and_results_exact":true,"all_EWRAM_IWRAM_VRAM_OAM_palette_exact":true,"SP_R8_R12_final_ROM_checked":true,
        "scope":"byte/u16/low10-copy/threshold-clamp-and-branch/posthelper-wrap/rawlow8-fullu32-interior-address prefixes only; controlled inputs and expected destination writes. No helper-RNG/prologue-epilogue/fullABI/nativeclock/ordinary/runtime proof",
        "schema_extractor_decoder_unchanged":[1,89,88],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORLD_RECORDS38_MODEL bytes1178 guards{guards} base{base} synthetic{synthetic} PASS");
}
