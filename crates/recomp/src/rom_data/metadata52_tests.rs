//! Original packed properties, redirect paths and raw live-index tail readers.
use super::*;
use gba_core::{
    rom_data::metadata52::{
        Catalog, Metadata52, Record, END_OFFSET, PROPERTY_FIELDS, RECORD_COUNT, SOURCE_OFFSET,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
const UNIT: u32 = 0x02024000;
const STACK: u32 = 0x03007e00;
pub(super) const RETURN: u32 = 0x08000300;
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Property {
        row: u32,
        alternate: u32,
        property: u32,
    },
    Tail32 {
        row: u8,
        selector: u32,
    },
    Tail33(u8),
}
pub(super) fn cases() -> Vec<Case> {
    let mut v = vec![];
    for row in 0..RECORD_COUNT as u32 {
        for property in 0..48 {
            v.push(Case::Property {
                row,
                alternate: 0,
                property,
            });
        }
        v.push(Case::Tail32 {
            row: row as u8,
            selector: 0,
        });
        v.push(Case::Tail33(row as u8));
    }
    for row in [80, 81, 82, 85] {
        for alternate in [2, 79, 115] {
            for property in (0..48).filter(|&p| p != 2) {
                v.push(Case::Property {
                    row,
                    alternate,
                    property,
                });
            }
        }
    }
    for (row, alternate, property) in [
        (0xffff0073, 0xabcd0002, 0xdead002f),
        (0x10050, 0x10002, 0x10000),
        (0x10055, 0x10073, 0x1001b),
        (80, 85, 2),
    ] {
        v.push(Case::Property {
            row,
            alternate,
            property,
        });
    }
    v.push(Case::Tail32 {
        row: 115,
        selector: 0x10000,
    });
    v
}
pub(super) fn setup(m: &mut Machine, case: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    for i in 0..64 {
        m.bus.write8(STACK - 32 + i, 0x5a);
    }
    for i in 0..32 {
        m.bus.write8(UNIT + i, 0x3c);
    }
    match case {
        Case::Property {
            row,
            alternate,
            property,
        } => {
            m.cpu.regs[..3].copy_from_slice(&[row, alternate, property]);
            m.cpu.regs[15] = 0x080cbec8;
        }
        Case::Tail32 { row, selector } => {
            m.bus.write8(UNIT + 5, row);
            m.cpu.regs[0] = UNIT;
            m.cpu.regs[1] = selector;
            m.cpu.regs[15] = 0x08137db4;
        }
        Case::Tail33(row) => {
            m.bus.write8(UNIT + 5, row);
            m.cpu.regs[0] = UNIT;
            m.cpu.regs[15] = 0x081344b4;
        }
    }
}
pub(super) fn check(m: &mut Machine, case: Case, data: &Metadata52) {
    let expected = match case {
        Case::Property {
            row,
            alternate,
            property,
        } => data
            .property(row, alternate, property)
            .expect("returning contract"),
        Case::Tail32 { row, selector } => {
            assert_eq!(selector as u16, 0);
            u32::from(data.records[usize::from(row)].byte(50).unwrap())
        }
        Case::Tail33(row) => u32::from(data.records[usize::from(row)].byte(51).unwrap()),
    };
    assert_eq!(m.cpu.regs[0], expected, "{case:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..13 {
        assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32, "callee {case:?} r{i}");
    }
    let mut unit = vec![0x3c; 32];
    match case {
        Case::Tail32 { row, .. } | Case::Tail33(row) => unit[5] = row,
        _ => {}
    }
    assert_eq!(&m.bus.ewram[0x24000..0x24020], unit);
    for i in 0..16 {
        assert_eq!(m.bus.read8(STACK - 32 + i), 0x5a);
    }
    for i in 0..32 {
        assert_eq!(m.bus.read8(STACK + i), 0x5a);
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    data: &Metadata52,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case);
    let mut n = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(n < 200, "metadata52 {case:?} at{:08x}", m.cpu.regs[15]);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    check(m, case, data);
    n
}
fn nonreturning(
    m: &mut Machine,
    case: Case,
    data: &Metadata52,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    let Case::Property {
        row,
        alternate,
        property,
    } = case
    else {
        unreachable!()
    };
    assert!(data.property(row, alternate, property).is_none());
    setup(m, case);
    let mut dispatch = 0;
    for _ in 0..192 {
        let pc = m.cpu.regs[15];
        assert!(
            (0x080cbec8..0x080cc37c).contains(&pc),
            "loop escaped {case:?} {pc:08x}"
        );
        if pc == 0x080cbee2 {
            dispatch += 1;
        }
        visited.insert((pc, m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
    }
    assert!(dispatch >= 3, "original loop {case:?}");
    assert_eq!(m.cpu.regs[13], STACK - 16);
    assert_eq!(&m.bus.ewram[0x24000..0x24020], &[0x3c; 32]);
    192
}
#[test]
#[ignore = "private EUR metadata52 packed fields/redirect/raw tail and nonreturning paths; reduced Wine only, Linux/USA pending"]
fn european_metadata52_matches_original_packed_redirect_and_raw_tail_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA52_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA52_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA52_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
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
        (1, SOURCE_OFFSET, 6032)
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Metadata52 = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..7 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["routing"] = 256.into(),
            3 => bad["records"][0]["bytes_06_1f"][0] = (-1).into(),
            4 => {
                bad["records"][0]["bytes_00_04"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            5 => {
                bad["records"][0]["bytes_06_1f"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            _ => {
                bad["records"][0]["bytes_20_33"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
        }
        assert!(serde_json::from_value::<Metadata52>(bad).is_err());
    }
    for kind in 0..3 {
        let mut bad = data.clone();
        match kind {
            0 => bad.source_offset += 52,
            1 => {
                bad.records.pop();
            }
            _ => bad.records.push(bad.records[0].clone()),
        };
        assert!(bad.validate().is_err());
    }
    for kind in 0..5 {
        let mut bad = data.clone();
        match kind {
            0 => bad.records[0].bytes_00_04[2] ^= 1,
            1 => bad.records[2].routing = 3,
            2 => bad.records[115].bytes_06_1f[12] ^= 1,
            3 => bad.records[115].bytes_20_33[19] ^= 1,
            _ => bad.records.swap(2, 3),
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
    let cases = cases();
    for &case in &cases {
        instructions += run(&mut m, case, data, &mut visited);
    }
    let real_loops = [
        Case::Property {
            row: 0,
            alternate: 0,
            property: 48,
        },
        Case::Property {
            row: 2,
            alternate: 2,
            property: 255,
        },
        Case::Property {
            row: 80,
            alternate: 80,
            property: 0,
        },
        Case::Property {
            row: 80,
            alternate: 81,
            property: 47,
        },
    ];
    for case in real_loops {
        instructions += nonreturning(&mut m, case, data, &mut visited);
    }
    // Nonzero redirects are absent in the shipped table. Exercise their original
    // branch and high packed bits only in an isolated, unauthenticated ROM copy.
    let mut synthetic = data.clone();
    let mut synthetic_calls = 0;
    for pattern in [0, 255, 0x5a] {
        let raw: [u8; 52] = std::array::from_fn(|i| {
            if pattern == 0x5a {
                (i as u8).wrapping_mul(17).wrapping_add(pattern)
            } else {
                pattern
            }
        });
        synthetic.records[4] = Record {
            bytes_00_04: raw[..5].try_into().unwrap(),
            routing: 0,
            bytes_06_1f: raw[6..32].try_into().unwrap(),
            bytes_20_33: raw[32..].try_into().unwrap(),
        };
        synthetic.records[2].routing = 3;
        synthetic.records[3].routing = 4;
        m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
            .copy_from_slice(&synthetic.source_bytes());
        assert!(synthetic.verify_source(&m.bus.rom).is_err());
        for row in [2, 80] {
            for property in 0..48 {
                instructions += run(
                    &mut m,
                    Case::Property {
                        row,
                        alternate: 2,
                        property,
                    },
                    &synthetic,
                    &mut visited,
                );
                synthetic_calls += 1;
            }
        }
        for case in [
            Case::Tail32 {
                row: 4,
                selector: 0,
            },
            Case::Tail33(4),
        ] {
            instructions += run(&mut m, case, &synthetic, &mut visited);
            synthetic_calls += 1;
        }
    }
    synthetic.records[3].routing = 2;
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&synthetic.source_bytes());
    for row in [2, 80] {
        instructions += nonreturning(
            &mut m,
            Case::Property {
                row,
                alternate: 2,
                property: 0,
            },
            &synthetic,
            &mut visited,
        );
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    for (i, (&actual, &old)) in m.bus.ewram.iter().zip(ewram.iter()).enumerate() {
        if !(0x24000..0x24020).contains(&i) {
            assert_eq!(actual, old, "EWRAM{i:x}");
        }
    }
    for (i, (&actual, &old)) in m.bus.iwram.iter().zip(iwram.iter()).enumerate() {
        if !(0x7de0..0x7e20).contains(&i) {
            assert_eq!(actual, old, "IWRAM{i:x}");
        }
    }
    assert_eq!(PROPERTY_FIELDS.len(), 48);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("metadata52.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":116,"source_bytes":6032,"original_returning_cases":cases.len(),"original_nonreturning_probes":real_loops.len(),"synthetic_returning_cases":synthetic_calls,"synthetic_nonreturning_probes":2,"instructions":instructions,"distinct_instructions":visited.len(),"all_48_properties_all_116_records_with_valid_alternates":true,"four_ff_rows_multiple_alternates_and_u8_truncation":true,"raw_live_index_tail32_tail33":true,"packed_masks_nonzero_redirect_chain_and_ff_chain":true,"invalid_property_ff_cycles_and_redirect_cycles_remain_nonreturning":true,"strict_json_extent_coherent_source_and_ram_stack_callee_guards":true,"managed_runtime":"pending; dataset remains e25-d24","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
