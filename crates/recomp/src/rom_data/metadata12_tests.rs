//! Original packed pair readers; finite diagnostics do not add guest bounds.
use super::*;
use gba_core::{
    rom_data::metadata12::{
        BankId, Catalog, Metadata12, END_OFFSET, SECONDARY_OFFSET, SOURCE_OFFSET,
    },
    Machine,
};
use std::collections::BTreeSet;
pub(super) const STACK: u32 = 0x03007e00;
pub(super) const RETURN: u32 = 0x08000300;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Case {
    pub bank: BankId,
    pub row: u32,
    pub slot: u32,
    pub mode: u32,
}
pub(super) fn cases(data: &Metadata12) -> Vec<Case> {
    let mut out = BTreeSet::new();
    for bank in &data.banks {
        for r in &bank.records {
            for slot in [0, 3, 4, 31, 43, 47] {
                for mode in [0, 1] {
                    out.insert(Case {
                        bank: bank.id,
                        row: r.id as u32,
                        slot,
                        mode,
                    });
                }
            }
        }
        for row in [0, bank.records.len() as u32 - 1] {
            for slot in 0..48 {
                for mode in [0, 1] {
                    out.insert(Case {
                        bank: bank.id,
                        row,
                        slot,
                        mode,
                    });
                }
            }
        }
        for mode in [0x100, 0x101, 0xff, 0xffff0000] {
            out.insert(Case {
                bank: bank.id,
                row: bank.records.len() as u32 - 1,
                slot: 43,
                mode,
            });
        }
    }
    out.into_iter().collect()
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.bus.ewram[0x23ff0..0x24110].fill(0x3c);
    m.bus.iwram[0x7d00..0x7e20].fill(0x5a);
    m.cpu.regs[..3].copy_from_slice(&[c.row, c.slot, c.mode]);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = match c.bank {
        BankId::Primary => 0x08137620,
        BankId::Secondary => 0x08137644,
    };
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
pub(super) fn address(c: Case) -> (u32, u32) {
    let bit = c
        .slot
        .wrapping_mul(2)
        .wrapping_add(u32::from(c.mode as u8 != 0));
    (
        0x08000000
            + c.bank
                .source_offset()
                .wrapping_add(c.row.wrapping_mul(12))
                .wrapping_add(((bit as i32) >> 3) as u32),
        bit & 7,
    )
}
pub(super) fn execute(
    m: &mut Machine,
    c: Case,
    expected: bool,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, c);
    let (expected_address, _) = address(c);
    let mut reads = 0;
    let mut n = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(n < 100, "original reader must return {c:?}");
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        if m.cpu.regs[15] == 0x08135a34 {
            assert_eq!(
                m.cpu.regs[0], expected_address,
                "original byte address {c:?}"
            );
            reads += 1;
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    assert_eq!(reads, 1);
    assert_eq!(
        m.cpu.regs[0],
        u32::from(expected),
        "original pair flag {c:?}"
    );
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32, "callee r{i} {c:?}");
    }
    assert!(m.bus.ewram[0x23ff0..0x24110].iter().all(|&v| v == 0x3c));
    assert!(m.bus.iwram[0x7d00..0x7df4].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    n
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &Metadata12,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    let expected = data
        .record(c.bank, c.row)
        .unwrap()
        .flag(c.slot, c.mode)
        .unwrap();
    execute(m, c, expected, visited)
}
#[test]
#[ignore = "private EUR packed pair source oracle; one reduced Wine check, Linux/USA pending"]
fn european_metadata12_matches_original_packed_pair_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA12_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA12_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate().unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(catalog.sources.len(), 2);
    assert_eq!(
        (catalog.sources[0].length, catalog.sources[1].length),
        (1104, 192)
    );
    for (i, bank) in data.banks.iter().enumerate() {
        assert_eq!(independent["banks"][i]["source_offset"], bank.source_offset);
        assert_eq!(
            independent["banks"][i]["records"].as_array().unwrap().len(),
            bank.records.len()
        );
        for (j, r) in bank.records.iter().enumerate() {
            assert_eq!(independent["banks"][i]["records"][j]["id"], r.id);
            assert_eq!(
                independent["banks"][i]["records"][j]["source_offset"],
                r.source_offset
            );
            let flat: Vec<_> = r.groups.iter().flatten().copied().collect();
            assert_eq!(
                independent["banks"][i]["records"][j]["pairs"],
                serde_json::json!(flat)
            );
        }
    }
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(&serde_json::from_slice::<Metadata12>(&bytes).unwrap(), data);
    let shape = serde_json::to_value(data).unwrap();
    for case in 0..6 {
        let mut v = shape.clone();
        match case {
            0 => {
                v["extra"] = true.into();
            }
            1 => {
                v["banks"][0]["records"][0]["extra"] = true.into();
            }
            2 => {
                v["banks"][0]["records"][0]["groups"][0] = serde_json::json!([0, 0, 0]);
            }
            3 => {
                v["banks"][0]["records"][0]["groups"] = serde_json::json!([]);
            }
            4 => {
                v["banks"][1]["id"] = "unknown".into();
            }
            _ => {
                v["banks"].as_array_mut().unwrap().pop();
            }
        }
        assert!(serde_json::from_value::<Metadata12>(v).is_err());
    }
    for case in 0..7 {
        let mut v = data.clone();
        match case {
            0 => v.banks.swap(0, 1),
            1 => v.banks[0].source_offset += 1,
            2 => {
                v.banks[1].records.pop();
            }
            3 => v.banks[0].records[1].id = 0,
            4 => v.banks[0].records[0].source_offset += 12,
            5 => v.banks[0].records[0].groups[0][0] = 4,
            _ => {
                v.banks[1].records[0].groups[11][3] ^= 1;
            }
        }
        if case < 6 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    assert!(data.record(BankId::Primary, 92).is_none());
    assert!(data.record(BankId::Secondary, 16).is_none());
    assert!(data.record(BankId::Primary, 256).is_none());
    assert!(data.banks[0].records[0].flag(48, 0).is_none());
    assert!(data.banks[0].records[0].flag(u32::MAX, 0).is_none());
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let real = cases(data);
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited);
    }
    // Show original full row/slot arithmetic independently; no diagnostic None is returned to the guest.
    let mut outside = vec![];
    for bank in [BankId::Primary, BankId::Secondary] {
        outside.extend([
            Case {
                bank,
                row: bank.record_count() as u32,
                slot: 0,
                mode: 0,
            },
            Case {
                bank,
                row: 256,
                slot: 0,
                mode: 1,
            },
            Case {
                bank,
                row: 0x40000000,
                slot: 43,
                mode: 0x101,
            },
            Case {
                bank,
                row: 0,
                slot: u32::MAX,
                mode: 0,
            },
            Case {
                bank,
                row: 0,
                slot: 48,
                mode: 1,
            },
        ]);
    }
    for &c in &outside {
        let (a, bit) = address(c);
        let expected = (rom[(a - 0x08000000) as usize] >> bit) & 1 != 0;
        instructions += execute(&mut m, c, expected, &mut visited);
    }
    let mut synthetic = 0;
    for bank in [BankId::Primary, BankId::Secondary] {
        let mut modified = data.clone();
        let record = &mut modified.banks[bank.index()].records[0];
        record.groups = std::array::from_fn(|i| std::array::from_fn(|j| ((i * 3 + j) & 3) as u8));
        let at = record.source_offset as usize;
        let source = record.source_bytes();
        modified.validate().unwrap();
        assert!(modified.verify_source(&rom).is_err());
        m.bus.rom[at..at + 12].copy_from_slice(&source);
        assert!(Catalog::from_rom(&m.bus.rom).is_err());
        for slot in 0..48 {
            for mode in [0, 0x101, 0xff, 0x100] {
                instructions += run(
                    &mut m,
                    Case {
                        bank,
                        row: 0,
                        slot,
                        mode,
                    },
                    &modified,
                    &mut visited,
                );
                synthetic += 1;
            }
        }
        m.bus.rom[at..at + 12].copy_from_slice(&rom[at..at + 12]);
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("metadata12.json"), &bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"banks":2,"records":108,"pairs":5184,"source_bytes":1296,"real_cases":real.len(),"raw_outside_or_wrapping_cases":outside.len(),"synthetic_cases":synthetic,"instructions_coverage":instructions,"pcs":visited.len(),"first_next_source":SECONDARY_OFFSET,"independent_following_source":END_OFFSET,"strict_json_fixed_shapes_pair_range_order_ids_source":true,"original_row32_slot_signed_wrap_u8mode_and_one_byte_read":true,"callee_stack_and_outer_ram_guards":true,"ordinary_loop_slots":[0,43],"stored_pairs44_47":"structural preserved; global caller bounds unproved","synthetic":"unauthenticated/unpublished","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
