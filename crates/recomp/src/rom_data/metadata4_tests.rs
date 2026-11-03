//! Bounded original count/gate/threshold prefixes; deeper live scans stay original.
use super::*;
use gba_core::{
    rom_data::{
        metadata4::{Catalog, Metadata4, Pair, Record, END_OFFSET, RECORD_COUNT, SOURCE_OFFSET},
        metadata52::Metadata52,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
const STACK: u32 = 0x03007e00;
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Count {
        path: u8,
        row: usize,
    },
    Select {
        path: u8,
        owner: u8,
    },
    Gate {
        path: u8,
        row: usize,
    },
    Threshold {
        path: u8,
        row: usize,
        slot: usize,
        qualified: u32,
        tally: u8,
    },
}
fn row(case: Case, owner: &Metadata52) -> usize {
    match case {
        Case::Count { row, .. } | Case::Gate { row, .. } | Case::Threshold { row, .. } => row,
        Case::Select { owner: id, .. } => {
            owner.property(u32::from(id), u32::from(id), 39).unwrap() as usize
        }
    }
}
fn path(case: Case) -> u8 {
    match case {
        Case::Count { path, .. }
        | Case::Select { path, .. }
        | Case::Gate { path, .. }
        | Case::Threshold { path, .. } => path,
    }
}
pub(super) fn cases(data: &Metadata4) -> Vec<Case> {
    let mut v = vec![];
    for path in 0..2 {
        for row in 0..RECORD_COUNT {
            v.push(Case::Count { path, row });
            v.push(Case::Gate { path, row });
            for (slot, p) in data.records[row]
                .slots
                .iter()
                .enumerate()
                .take_while(|(_, p)| p.selector != 0)
            {
                for qualified in [
                    u32::from(p.value) - 1,
                    u32::from(p.value),
                    u32::from(p.value) + 1,
                ] {
                    v.push(Case::Threshold {
                        path,
                        row,
                        slot,
                        qualified,
                        tally: 0,
                    });
                }
            }
        }
        for owner in 2..=43 {
            v.push(Case::Select { path, owner });
        }
    }
    v
}
pub(super) fn stops(case: Case) -> (u32, u32) {
    match case {
        Case::Count { path: 0, .. } | Case::Select { path: 0, .. } => (0x080cc44e, 0x080cc44e),
        Case::Count { .. } | Case::Select { .. } => (0x08064330, 0x08064330),
        Case::Gate { path: 0, .. } | Case::Threshold { path: 0, .. } => (0x080cc47e, 0x080cc510),
        _ => (0x0806435c, 0x080643ea),
    }
}
pub(super) fn setup(m: &mut Machine, case: Case, data: &Metadata4, owner: &Metadata52) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    for i in 0..64 {
        m.bus.write8(STACK - 32 + i, 0x5a);
    }
    let r = row(case, owner);
    let p = 0x08000000 + SOURCE_OFFSET + 4 * r as u32;
    m.cpu.regs[2] = p;
    match case {
        Case::Count { path, row } => {
            m.cpu.regs[0] = row as u32;
            m.cpu.regs[15] = if path == 0 { 0x080cc424 } else { 0x080642fe };
        }
        Case::Select { path, owner: id } => {
            assert!((2..=43).contains(&id));
            m.cpu.regs[5] = u32::from(id);
            m.cpu.regs[15] = if path == 0 { 0x080cc41a } else { 0x080642f4 };
        }
        Case::Gate { path, .. } => {
            if path == 0 {
                m.bus
                    .write32(STACK, u32::from(data.records[r].active_count()));
                m.bus.write32(STACK + 4, 0);
            } else {
                m.bus.write32(STACK, 0);
                m.cpu.regs[10] = u32::from(data.records[r].active_count());
            }
            m.cpu.regs[15] = if path == 0 { 0x080cc474 } else { 0x08064350 };
        }
        Case::Threshold {
            path,
            slot,
            qualified,
            tally,
            ..
        } => {
            assert!(slot < 2);
            m.cpu.regs[8] = p + 2 * slot as u32;
            if path == 0 {
                m.cpu.regs[2] = qualified;
                m.cpu.regs[3] = slot as u32;
                m.bus
                    .write32(STACK, u32::from(data.records[r].active_count()));
                m.bus.write32(STACK + 4, u32::from(tally));
            } else {
                m.cpu.regs[3] = qualified;
                m.cpu.regs[9] = slot as u32;
                m.cpu.regs[10] = u32::from(data.records[r].active_count());
                m.bus.write32(STACK, u32::from(tally));
            }
            m.cpu.regs[15] = if path == 0 { 0x080cc4ec } else { 0x080643c2 };
        }
    }
}
pub(super) fn check(m: &mut Machine, case: Case, data: &Metadata4, owner: &Metadata52) {
    let r = row(case, owner);
    let record = &data.records[r];
    let p = 0x08000000 + SOURCE_OFFSET + 4 * r as u32;
    let path = path(case);
    let ends = stops(case);
    let count = u32::from(record.active_count());
    let (words, pc) = match case {
        Case::Count { .. } | Case::Select { .. } => {
            assert_eq!(m.cpu.regs[1], p + 4);
            assert_eq!(m.cpu.regs[2], p);
            if path == 0 {
                assert_eq!(m.cpu.regs[3], u32::MAX);
            } else {
                assert_eq!(m.cpu.regs[9], u32::MAX);
                assert_eq!(m.cpu.regs[10], count);
            }
            (
                [
                    if path == 0 { count } else { 0 },
                    if path == 0 { 0 } else { 0x5a5a5a5a },
                ],
                ends.0,
            )
        }
        Case::Gate { .. } => {
            let active = record.leading_count() != 0;
            assert_eq!(m.cpu.regs[5], u32::from(record.slots[0].selector));
            assert_eq!(m.cpu.regs[8], if active { p } else { 0x11220008 });
            if path == 0 {
                assert_eq!(m.cpu.regs[3], 0);
            } else {
                assert_eq!(m.cpu.regs[9], 0);
            }
            (
                [
                    if path == 0 { count } else { 0 },
                    if path == 0 { 0 } else { 0x5a5a5a5a },
                ],
                if active { ends.0 } else { ends.1 },
            )
        }
        Case::Threshold {
            slot,
            qualified,
            tally,
            ..
        } => {
            let passed = (qualified as i32) >= i32::from(record.slots[slot].value);
            let next = slot + usize::from(passed);
            let more = passed && next < 2 && record.slots[next].selector != 0;
            assert_eq!(m.cpu.regs[8], p + 2 * next as u32);
            if path == 0 {
                assert_eq!(m.cpu.regs[3], next as u32);
            } else {
                assert_eq!(m.cpu.regs[9], next as u32);
            }
            let tally = if passed { tally.wrapping_add(1) } else { tally };
            (
                [
                    if path == 0 { count } else { u32::from(tally) },
                    if path == 0 {
                        u32::from(tally)
                    } else {
                        0x5a5a5a5a
                    },
                ],
                if more { ends.0 } else { ends.1 },
            )
        }
    };
    assert_eq!(m.cpu.regs[15], pc, "{case:?}");
    assert_eq!(m.bus.read32(STACK), words[0], "{case:?}");
    assert_eq!(m.bus.read32(STACK + 4), words[1], "{case:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in [4, 6, 7] {
        assert_eq!(
            m.cpu.regs[i],
            0x11220000 + i as u32,
            "prefix preserved r{i} {case:?}"
        );
    }
    if let Case::Select { owner: id, .. } = case {
        assert_eq!(m.cpu.regs[5], u32::from(id));
    }
    for i in 0..16 {
        assert_eq!(m.bus.read8(STACK - 32 + i), 0x5a);
    }
    for i in 8..32 {
        assert_eq!(m.bus.read8(STACK + i), 0x5a);
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    data: &Metadata4,
    owner: &Metadata52,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case, data, owner);
    let ends = stops(case);
    let mut n = 0;
    while m.cpu.regs[15] != ends.0 && m.cpu.regs[15] != ends.1 {
        assert!(n < 200, "metadata4 {case:?} at{:08x}", m.cpu.regs[15]);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    check(m, case, data, owner);
    n
}
#[test]
#[ignore = "private EUR metadata4 original selection/count/gate/threshold prefixes; reduced Wine only, Linux/USA pending"]
fn european_metadata4_matches_original_two_pair_count_gate_and_threshold_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA4_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA4_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA4_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let owner: Metadata52 = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA52_SOURCE_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate_metadata52(&owner).unwrap();
    assert_eq!(&data.records, &independent);
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(inv["records"], RECORD_COUNT);
    assert_eq!(
        inv["source_sha256"],
        gba_core::ffta_rom::digest(&data.source_bytes())
    );
    assert_eq!(
        (
            catalog.sources.len(),
            catalog.sources[0].offset,
            catalog.sources[0].length
        ),
        (1, SOURCE_OFFSET, 80)
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Metadata4 = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..6 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["slots"][0]["unknown"] = 0.into(),
            3 => bad["records"][0]["slots"][0]["selector"] = 256.into(),
            4 => bad["records"][0]["slots"][0]["value"] = (-1).into(),
            _ => {
                bad["records"][0]["slots"].as_array_mut().unwrap().pop();
            }
        };
        assert!(serde_json::from_value::<Metadata4>(bad).is_err());
    }
    for kind in 0..3 {
        let mut bad = data.clone();
        match kind {
            0 => bad.source_offset += 4,
            1 => {
                bad.records.pop();
            }
            _ => bad.records.push(bad.records[0].clone()),
        };
        assert!(bad.validate().is_err());
    }
    let mut bad_owner = owner.clone();
    bad_owner.records[115].bytes_20_33[16] = 20;
    assert!(data.validate_metadata52(&bad_owner).is_err());
    let mut bad_pair = data.clone();
    bad_pair.records[0].slots[1].selector = 116;
    assert!(bad_pair.validate_metadata52(&owner).is_err());
    for kind in 0..4 {
        let mut bad = data.clone();
        match kind {
            0 => bad.records[1].slots[0].selector ^= 1,
            1 => bad.records[0].slots[1].value = 127,
            2 => bad.records[19].slots[1].value ^= 1,
            _ => bad.records.swap(1, 2),
        };
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut m = crate::make_machine(rom, None);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    let cases = cases(data);
    for &case in &cases {
        instructions += run(&mut m, case, data, &owner, &mut visited);
    }
    let mut synthetic = data.clone();
    synthetic.records[0].slots = [
        Pair {
            selector: 0,
            value: 127,
        },
        Pair {
            selector: 2,
            value: 255,
        },
    ];
    synthetic.records[1].slots = [
        Pair {
            selector: 3,
            value: 0,
        },
        Pair {
            selector: 4,
            value: 255,
        },
    ];
    synthetic.records[2].slots = [
        Pair {
            selector: 3,
            value: 1,
        },
        Pair {
            selector: 0,
            value: 127,
        },
    ];
    assert_eq!(
        (
            synthetic.records[0].active_count(),
            synthetic.records[0].leading_count()
        ),
        (1, 0)
    );
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&synthetic.source_bytes());
    assert!(synthetic.verify_source(&m.bus.rom).is_err());
    let mut synthetic_cases = vec![];
    for path in 0..2 {
        for row in 0..3 {
            synthetic_cases.push(Case::Count { path, row });
            synthetic_cases.push(Case::Gate { path, row });
        }
        for (row, slot, qualified, tally) in [
            (1, 0, 0, 255),
            (1, 1, 254, 0),
            (1, 1, 255, 255),
            (1, 1, 0x80000000, 0),
            (2, 0, 1, 0),
            (2, 0, 0, 0),
        ] {
            synthetic_cases.push(Case::Threshold {
                path,
                row,
                slot,
                qualified,
                tally,
            });
        }
    }
    for &case in &synthetic_cases {
        instructions += run(&mut m, case, &synthetic, &owner, &mut visited);
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    assert_eq!(m.bus.ewram, ewram);
    for (i, (&actual, &old)) in m.bus.iwram.iter().zip(iwram.iter()).enumerate() {
        if !(0x7de0..0x7e20).contains(&i) {
            assert_eq!(actual, old, "IWRAM{i:x}");
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("metadata4.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":20,"source_bytes":80,"active_pairs":26,"original_cases":cases.len(),"synthetic_cases":synthetic_cases.len(),"instructions":instructions,"distinct_instructions":visited.len(),"both_original_count_and_contiguous_gate_prefixes_all20rows":true,"both_selector_prefixes_job_ids2to43":true,"both_live_qualified_count_threshold_prefixes_below_equal_above":true,"synthetic_holes_unused_payload_unsigned255_signed_comparison_tally_u8wrap":true,"strict_json_extent_source_and_parsed_metadata52_owner_guards":true,"prefix_register_stack_and_outer_ram_guards":true,"whole_live_unit_ability_scans":"not claimed; deeper original code stays live","managed_runtime":"pending; current dataset e26-d25","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
