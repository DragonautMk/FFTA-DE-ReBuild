//! Isolated original arithmetic/load prefixes; live helpers and routine ABI excluded.
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
    patches: Vec<(u8, [u8; 27])>,
    expected: Vec<u32>,
    owned: Option<Vec<u32>>,
    reads: Vec<u32>,
}
fn projection(data: &ParameterRecords27, c: &Case) -> Option<Vec<u32>> {
    if c.stage == 11 {
        return Some(vec![ParameterRecords27::encoded_view(c.index)]);
    }
    let r = data.record(c.index)?;
    Some(match c.stage {
        0 => vec![u32::from(r.header00)],
        1 => vec![u32::from(r.selector01_02())],
        2 => vec![u32::from(r.selector03_04())],
        3 => {
            let (a, b) = r.nibble_operands();
            vec![u32::from(a), u32::from(b)]
        }
        4 => vec![u32::from(r.value07)],
        5 => vec![u32::from(r.value0a)],
        6 => vec![r.signed0c_scaled() as u32],
        7 => vec![u32::from(r.flags0d & 0x30)],
        8 => {
            let (a, b) = r.marked_operands0e();
            vec![u32::from(a), u32::from(b)]
        }
        9 => {
            let (a, b) = r.marked_operands10();
            vec![u32::from(a), u32::from(b)]
        }
        10 => vec![
            u32::from(r.value12),
            if r.value12 == 0 {
                0
            } else {
                u32::from(r.signed15_encoded())
            },
        ],
        _ => panic!("unknown stage"),
    })
}
fn original(
    m: &mut Machine,
    data: &ParameterRecords27,
    c: &Case,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.ewram[0x1a8] = c.index as u8;
    let (ewram, iwram, vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007d00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.cpsr = 0xbf;
    let address = ParameterRecords27::encoded_view(c.index);
    let (entry, stop, result_regs): (u32, u32, &[usize]) = match c.stage {
        0 => {
            m.cpu.regs[0] = address;
            (0xd25bc, 0xd25be, &[0])
        }
        1 => {
            m.cpu.regs[0] = address;
            (0x47216, 0x47226, &[1])
        }
        2 => {
            m.cpu.regs[6] = address;
            (0x34d74, 0x34d82, &[0])
        }
        3 => {
            m.cpu.regs[4] = address;
            (0x405b2, 0x405cc, &[1, 2])
        }
        4 => {
            m.cpu.regs[5] = address;
            (0x406da, 0x406e6, &[1])
        }
        5 => {
            m.cpu.regs[6] = address;
            (0x40d22, 0x40d2a, &[2])
        }
        6 => {
            m.cpu.regs[6] = address;
            (0x40d50, 0x40d58, &[0])
        }
        7 => {
            m.cpu.regs[0] = address;
            (0x40fb0, 0x40fb8, &[1])
        }
        8 => {
            m.cpu.regs[4] = address;
            (0x40b5a, 0x40b66, &[1, 2])
        }
        9 => {
            m.cpu.regs[0] = address;
            (0x409de, 0x409ea, &[1, 2])
        }
        10 => {
            m.cpu.regs[4] = address;
            (
                0x4a1ee,
                if c.expected[0] == 0 { 0x4a206 } else { 0x4a202 },
                &[0, 1],
            )
        }
        11 => {
            m.cpu.regs[4] = 0x02000100;
            (0x34d40, 0x34d52, &[6])
        }
        _ => panic!("unknown stage"),
    };
    m.cpu.regs[15] = 0x08000000 + entry;
    let mut reads = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != 0x08000000 + stop {
        let pc = m.cpu.regs[15] - 0x08000000;
        let load = match pc {
            0xd25bc => Some((0, 0)),
            0x47216 => Some((0, 1)),
            0x4721a => Some((0, 2)),
            0x34d74 => Some((6, 3)),
            0x34d78 => Some((6, 4)),
            0x405b2 | 0x405c4 => Some((4, 5)),
            0x405b6 => Some((4, 6)),
            0x405c0 => Some((4, 4)),
            0x406da => Some((5, 7)),
            0x406dc => Some((5, 8)),
            0x40d22 => Some((6, 10)),
            0x40d24 => Some((6, 11)),
            0x40d52 => Some((6, m.cpu.regs[1])),
            0x40fb0 => Some((0, 13)),
            0x40b5a => Some((4, 15)),
            0x40b5c => Some((4, 14)),
            0x409de => Some((0, 17)),
            0x409e0 => Some((0, 16)),
            0x4a1ee => Some((4, 18)),
            0x4a1f0 => Some((4, 19)),
            0x4a1fc => Some((4, m.cpu.regs[1])),
            _ => None,
        };
        if let Some((base, relative)) = load {
            let at = m.cpu.regs[base] + relative - 0x08000000;
            if let Some(byte) = data.byte_at(at) {
                assert_eq!(byte, m.bus.rom[at as usize]);
                covered.insert(at);
            }
            reads.push(at);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 64, "{} PC{:08x}", c.label, m.cpu.regs[15]);
    }
    assert_eq!(reads, c.reads, "{} actual load order", c.label);
    assert_eq!(
        result_regs
            .iter()
            .map(|&r| m.cpu.regs[r])
            .collect::<Vec<_>>(),
        c.expected,
        "{} original prefix",
        c.label
    );
    assert_eq!(m.cpu.regs[13], 0x03007d00);
    assert_eq!(
        &m.cpu.regs[8..13],
        &(8..13).map(|i| 0x11000000 + i).collect::<Vec<_>>()
    );
    assert_eq!(m.bus.ewram, ewram);
    assert_eq!(m.bus.iwram, iwram);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR sparse parameter-record source/original-prefix proof; Wine only, Linux/USA pending"]
fn european_parameter_records27_match_source_and_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_PARAMETER_RECORDS27_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: ParameterRecords27 =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    assert_eq!(data.source_bytes().len(), SOURCE_BYTES);
    assert_eq!(ffta_rom::digest(&data.source_bytes()), SOURCE_SHA256);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 2);
    for (at, bytes, role) in &spans {
        assert_eq!(role, "parameter-records27");
        assert_eq!(
            bytes.as_slice(),
            &rom[*at as usize..*at as usize + bytes.len()]
        );
        for (i, &b) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*at + i as u32), Some(b));
        }
    }
    for i in 0..256 {
        assert_eq!(
            data.record(i).is_some(),
            i <= 50 || (115..=122).contains(&i)
        );
        for off in 0..27 {
            assert_eq!(
                data.byte_at(SOURCE + i * 27 + off).is_some(),
                data.record(i).is_some()
            );
        }
    }
    for i in [256, 65535, u32::MAX] {
        assert_eq!(data.record(i), None);
    }
    for at in [0, SOURCE - 1, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<ParameterRecords27>(&encoded).unwrap(),
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
            3 => bad.records[0].index = 51,
            _ => bad.records.push(bad.records[0].clone()),
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..16 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(0),
            1 => bad["records"][0]["extra"] = json!(0),
            2 => {
                bad["records"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("packed01");
            }
            3 => bad["records"][0]["header00"] = json!(-1),
            4 => bad["records"][0]["packed02"] = json!(256),
            5 => bad["records"][0]["value07"] = json!(-1),
            6 => bad["records"][0]["value0a"] = json!(65536),
            7 => bad["records"][0]["signed0c"] = json!(-129),
            8 => bad["records"][0]["signed0c"] = json!(128),
            9 => bad["records"][0]["signed15"] = json!(-129),
            10 => bad["records"][0]["signed15"] = json!(128),
            11 => bad["records"][0]["flags0d"] = json!(true),
            12 => bad["records"][0]["opaque16"] = json!(vec![0; 4]),
            13 => bad["records"][0]["opaque16"] = json!(vec![0; 6]),
            14 => bad["records"][0]["byte0e"] = json!("0"),
            _ => bad["records"][0]["index"] = json!(256),
        }
        assert!(serde_json::from_value::<ParameterRecords27>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0x47216] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.records[0].opaque16 = [255, 128, 1, 2, 3];
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let cases: Vec<Case> =
        serde_json::from_slice(&fs::read(root.join("independent-cases.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    let (mut base, mut synthetic) = (0, 0);
    let (mut base_bytes, mut synthetic_bytes) = (BTreeSet::new(), BTreeSet::new());
    let mut instructions = 0;
    for c in &cases {
        let mut current = data.clone();
        for (index, raw) in &c.patches {
            let at = current
                .records
                .binary_search_by_key(index, |r| r.index)
                .unwrap();
            current.records[at] = Record::decode(*index, raw);
        }
        current.validate().unwrap();
        assert_eq!(
            projection(&current, c),
            c.owned,
            "{} owned projection",
            c.label
        );
        for (at, bytes, _) in current.source_spans().unwrap() {
            m.bus.rom[at as usize..at as usize + bytes.len()].copy_from_slice(&bytes);
        }
        let cover = if c.patches.is_empty() {
            base += 1;
            &mut base_bytes
        } else {
            synthetic += 1;
            &mut synthetic_bytes
        };
        instructions += original(&mut m, &current, c, cover);
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()].copy_from_slice(bytes);
        }
    }
    assert_eq!(m.bus.rom, rom);
    let report = json!({"status":"PASS-source-model-original-prefixes","guards":guards,"observed_records":59,"source_bytes":1593,"source_spans":2,
        "base_prefix_cases":base,"synthetic_prefix_cases":synthetic,"actual_base_source_bytes_observed":base_bytes.len(),"actual_synthetic_source_bytes_observed":synthetic_bytes.len(),
        "instructions_coverage_only":instructions,"all_EWRAM_IWRAM_VRAM_OAM_palette_exact":true,"SP_R8_R12_final_ROM_checked":true,"actual_load_order_and_results_exact":true,
        "scope":"unsigned/packed/nibble/u16/signed/marked-operand/computed-u8-view prefixes only; helpers/prologues/epilogues/fullABI/liveeffects/nativeclock/ordinary/runtime excluded",
        "complete_catalog_extent_verified":false,"schema_extractor_decoder_unchanged":[1,88,87],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_PARAMETER_RECORDS27_MODEL bytes1593/spans2 guards{guards} base{base} synthetic{synthetic} PASS");
}
