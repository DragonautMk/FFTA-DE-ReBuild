//! Isolated original reader stages; numeric helpers/full routine stay guest code.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    label: String,
    stage: u8,
    headers: [u8; 5],
    statuses: [u8; 30],
    rank: i32,
    variant: Option<([u8; 4], [u8; 30])>,
    expected: Option<u32>,
    expected_rank: i32,
    queries: Vec<u8>,
    reads: Vec<u32>,
}
struct Live {
    values: [u8; 30],
    queries: Vec<u8>,
}
impl LiveEligibility for Live {
    fn read(&mut self, ordinal: u8) -> u8 {
        self.queries.push(ordinal);
        self.values[usize::from(ordinal - 1)]
    }
}

fn original_stage(
    m: &mut Machine,
    data: &ByteTables,
    c: &Case,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    for (i, &header) in c.headers.iter().enumerate() {
        m.bus.ewram[0x2e34 + 6 * i] = header;
    }
    for (i, &status) in c.statuses.iter().enumerate() {
        m.bus.ewram[0x2cc3 + 12 * i] = status;
    }
    let (ewram, iwram, vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[1] = 0x02000000;
    m.cpu.regs[4] = c.rank as u32;
    m.cpu.regs[13] = 0x03007d00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.regs[15] = match c.stage {
        0 => 0x080d323c,
        1 => 0x080d327e,
        _ => 0x080d32b4,
    };
    m.cpu.cpsr = 0xbf;
    let stop = match c.stage {
        0 if c.expected.is_some() => 0x080d3266,
        1 if c.expected != Some(0) => 0x080d32a4,
        2 if c.expected.is_some() => 0x080d32f6,
        _ => 0x080d3472,
    };
    let mut queries = Vec::new();
    let mut reads = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != stop {
        let pc = m.cpu.regs[15];
        if pc == 0x080377bc {
            queries.push(m.cpu.regs[0] as u8);
        }
        let address = match pc {
            0x080d3264 => Some(m.cpu.regs[0]),
            0x080d3292 => Some(m.cpu.regs[4]),
            0x080d32e6 => Some(m.cpu.regs[6]),
            _ => None,
        };
        if let Some(address) = address {
            let offset = address - 0x08000000;
            assert_eq!(data.byte_at(offset), Some(m.bus.rom[offset as usize]));
            covered.insert(offset);
            reads.push(offset);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 2000, "{} PC{:08x}", c.label, m.cpu.regs[15]);
    }
    assert_eq!(queries, c.queries, "{} original live order", c.label);
    assert_eq!(reads, c.reads, "{} original source order", c.label);
    match c.stage {
        0 => {
            assert_eq!(
                m.cpu.regs[7],
                c.headers.iter().filter(|&&b| b & 7 != 0).count() as u32
            );
            if let Some(v) = c.expected {
                assert_eq!(m.cpu.regs[4], v);
            }
        }
        1 => {
            assert_eq!(Some(m.cpu.regs[7]), c.expected);
            assert_eq!(m.cpu.regs[5], 31);
            assert_eq!(m.cpu.regs[4], 0x08000000 + SOURCE_END);
        }
        _ => {
            assert_eq!(m.cpu.regs[4] as i32, c.expected_rank);
            if let Some(v) = c.expected {
                assert_eq!(m.cpu.regs[7], v);
                assert_eq!(m.cpu.regs[5], v);
            } else {
                assert_eq!(m.cpu.regs[5], 31);
            }
        }
    }
    assert_eq!(m.cpu.regs[13], 0x03007d00);
    assert_eq!(
        &m.cpu.regs[8..13],
        &(8..13).map(|i| 0x11000000 + i as u32).collect::<Vec<_>>()
    );
    assert_eq!(m.bus.ewram, ewram);
    assert_eq!(m.bus.iwram, iwram);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR selection-byte original reader prefixes; Wine only, Linux/USA pending"]
fn european_selection_byte_tables_match_source_and_original_stages() {
    let root = PathBuf::from(std::env::var_os("FFTA_SELECTION_BYTE_TABLES_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: ByteTables =
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
    assert_eq!(ffta_rom::digest(&data.source_bytes()), SOURCE_SHA256);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 2);
    assert_eq!(
        spans[0],
        (SOURCE, data.thresholds.to_vec(), "count-thresholds".into())
    );
    assert_eq!(
        spans[1],
        (
            WEIGHT_SOURCE,
            data.weights.to_vec(),
            "ordinal-weights".into()
        )
    );
    for (i, &byte) in data.source_bytes().iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(byte));
    }
    for at in [
        0,
        SOURCE - 1,
        SOURCE_END,
        SOURCE_END + 1,
        SOURCE_END + 2,
        u32::MAX,
    ] {
        assert_eq!(data.byte_at(at), None);
    }
    assert_eq!(data.threshold(4), None);
    assert_eq!(data.threshold(u32::MAX), None);
    assert_eq!(data.weight(30), None);
    assert_eq!(data.weight(usize::MAX), None);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<ByteTables>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..2 {
        let mut bad = data.clone();
        if case == 0 {
            bad.threshold_source_offset += 1;
        } else {
            bad.weight_source_offset -= 1;
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..12 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(0),
            1 => {
                bad.as_object_mut().unwrap().remove("thresholds");
            }
            2 => bad["thresholds"] = json!(vec![0; 3]),
            3 => bad["thresholds"] = json!(vec![0; 5]),
            4 => bad["weights"] = json!(vec![0; 29]),
            5 => bad["weights"] = json!(vec![0; 31]),
            6 => bad["thresholds"][0] = json!(-1),
            7 => bad["weights"][0] = json!(256),
            8 => bad["weights"][0] = json!(true),
            9 => bad["weights"][0] = json!(1.5),
            10 => bad["threshold_source_offset"] = json!(-1),
            _ => bad["weight_source_offset"] = json!(false),
        }
        assert!(serde_json::from_value::<ByteTables>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0x377bc] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.thresholds.fill(255);
    changed.weights.fill(128);
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
    for c in &cases {
        let mut current = data.clone();
        if let Some((thresholds, weights)) = &c.variant {
            current.thresholds = *thresholds;
            current.weights = *weights;
            synthetic += 1;
        } else {
            base += 1;
        }
        current.validate().unwrap();
        let mut live = Live {
            values: c.statuses,
            queries: Vec::new(),
        };
        let result = match c.stage {
            0 => current.population_threshold(&c.headers).map(u32::from),
            1 => Some(current.sum_weights(&mut live)),
            _ => current.select_ordinal(c.rank, &mut live).map(u32::from),
        };
        assert_eq!(result, c.expected, "{} model", c.label);
        assert_eq!(live.queries, c.queries);
        let bytes = current.source_bytes();
        assert_eq!(bytes.len(), 34);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize].copy_from_slice(&bytes);
        let covered = if c.variant.is_some() {
            &mut synthetic_bytes
        } else {
            &mut base_bytes
        };
        instructions += original_stage(&mut m, &current, c, covered);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE_END as usize]);
    }
    assert_eq!(m.bus.rom, rom);
    assert_eq!(base_bytes.len(), 34);
    assert_eq!(synthetic_bytes.len(), 32);
    let report = json!({"status":"PASS-source-model-original-stages","source_bytes":34,"source_spans":2,
        "guards":guards,"base_stage_cases":base,"synthetic_stage_cases":synthetic,
        "actual_base_source_bytes_observed":base_bytes.len(),"actual_synthetic_source_bytes_observed":synthetic_bytes.len(),
        "instructions_coverage_only":instructions,"all_EWRAM_IWRAM_VRAM_OAM_palette_exact":true,
        "SP_R8_R12_final_ROM_checked":true,"real_377BC_live_queries_and_source_read_order_exact":true,
        "scope":"D323Cpopulation/threshold,D327Esum/zero-total,D32B4choice stages and actual377BC; external numeric helpers/prologue/epilogue/fullABI/nativeclock/ordinary/runtime excluded",
        "schema_extractor_decoder_unchanged":[1,86,85],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_SELECTION_BYTE_TABLES_MODEL bytes34/spans2 guards{guards} base{base} synthetic{synthetic} PASS");
}
