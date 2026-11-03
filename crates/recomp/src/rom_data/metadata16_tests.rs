//! New EUR metadata16 original-reader oracle; no managed dataset revision yet.
use super::*;
use gba_core::{
    rom_data::metadata16::{Catalog, Metadata16, Record, END_OFFSET, RECORD_COUNT, SOURCE_OFFSET},
    Bus, Machine,
};
use std::collections::BTreeSet;
const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;
const HEAD: u32 = 0x020028c8;
const ACTIVE: u32 = 0x02003c28;
const NODES: u32 = 0x02025000;
const RECORDS: u32 = NODES + 32;
const RNG: u32 = 0x030034b0;
const ID: u32 = 0x2a5;

#[derive(Clone, Copy, Debug)]
pub(super) enum Reader {
    Sum13,
    Choice14,
    SumFields,
    Flags,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Lookup {
    First,
    Second,
    Missing,
    Empty,
    Inactive,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Display(usize),
    Alias(usize),
    Pair {
        reader: Reader,
        first: u8,
        second: u8,
        selector: u32,
        lookup: Lookup,
        seed: u32,
    },
}
fn pair(reader: Reader, first: u8, second: u8, selector: u32) -> Case {
    Case::Pair {
        reader,
        first,
        second,
        selector,
        lookup: Lookup::Second,
        seed: 0xdeadbeef,
    }
}
pub(super) fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for row in 0..RECORD_COUNT {
        cases.push(Case::Display(row));
        if row != 0 {
            cases.push(Case::Alias(row));
        }
        let other = ((row * 37 + 5) % RECORD_COUNT) as u8;
        for reader in [Reader::Sum13, Reader::Choice14] {
            for (first, second) in [(row as u8, 0), (0, row as u8), (row as u8, other)] {
                cases.push(pair(reader, first, second, 0));
            }
        }
        for selector in 10..=13 {
            cases.push(pair(Reader::SumFields, row as u8, other, selector));
        }
        for selector in 1..=8 {
            cases.push(pair(Reader::Flags, row as u8, other, selector));
        }
    }
    for selector in [0, 9, 14, 255, 0x1000a] {
        cases.push(pair(Reader::SumFields, 5, 65, selector));
    }
    for selector in [0, 9, 32, 33, 255, 257] {
        cases.push(pair(Reader::Flags, 21, 65, selector));
    }
    for reader in [Reader::Sum13, Reader::Choice14] {
        for lookup in [Lookup::First, Lookup::Missing, Lookup::Empty] {
            cases.push(Case::Pair {
                reader,
                first: 5,
                second: 1,
                selector: 0,
                lookup,
                seed: 2,
            });
        }
    }
    for reader in [Reader::SumFields, Reader::Flags] {
        cases.push(Case::Pair {
            reader,
            first: 5,
            second: 65,
            selector: 10,
            lookup: Lookup::Inactive,
            seed: 0,
        });
    }
    for seed in [0, 2, 0xdeadbeef] {
        cases.push(Case::Pair {
            reader: Reader::Choice14,
            first: 5,
            second: 1,
            selector: 0,
            lookup: Lookup::Second,
            seed,
        });
    }
    cases
}
pub(super) fn stop(case: Case) -> u32 {
    match case {
        Case::Display(_) => 0x08016204,
        Case::Alias(_) => 0x080390a0,
        Case::Pair { .. } => RETURN,
    }
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
    for i in 0..96 {
        m.bus.write8(NODES + i, 0x5a);
    }
    m.bus.write32(HEAD, 0);
    m.bus.write16(ACTIVE, 0);
    m.bus.write32(RNG, 0);
    match case {
        Case::Display(row) => {
            m.cpu.regs[4] = row as u32;
            m.cpu.regs[15] = 0x080161e4;
        }
        Case::Alias(row) => {
            m.cpu.regs[5] = row as u32;
            m.cpu.regs[15] = 0x08039070;
        }
        Case::Pair {
            reader,
            first,
            second,
            selector,
            lookup,
            seed,
        } => {
            let chosen = if matches!(lookup, Lookup::First) {
                RECORDS
            } else {
                RECORDS + 16
            };
            for (node, record, next) in
                [(NODES, RECORDS, NODES + 16), (NODES + 16, RECORDS + 16, 0)]
            {
                m.bus.write32(node, record);
                m.bus.write32(node + 8, next);
                m.bus.write8(record, 1);
                m.bus.write8(record + 1, 0xfc);
            }
            let identity = if matches!(lookup, Lookup::Missing) {
                ID + 1
            } else {
                ID
            };
            m.bus.write8(chosen, identity as u8);
            m.bus.write8(chosen + 1, ((identity >> 8) as u8) | 0xfc);
            m.bus.write8(chosen + 4, first);
            m.bus.write8(chosen + 5, second);
            m.bus.write32(
                HEAD,
                if matches!(lookup, Lookup::Empty) {
                    0
                } else {
                    NODES
                },
            );
            m.bus.write16(
                ACTIVE,
                if matches!(lookup, Lookup::Inactive) {
                    0
                } else {
                    ID as u16
                },
            );
            m.bus.write32(RNG, seed);
            m.cpu.regs[0] = match reader {
                Reader::Sum13 | Reader::Choice14 => 0xabcd0000 | ID,
                _ => selector,
            };
            m.cpu.regs[15] = match reader {
                Reader::Sum13 => 0x080d5e20,
                Reader::Choice14 => 0x080d5ea0,
                Reader::SumFields => 0x080d6704,
                Reader::Flags => 0x080d67d4,
            };
        }
    }
}
fn expected(data: &Metadata16, case: Case) -> (u32, Option<u32>) {
    let Case::Pair {
        reader,
        first,
        second,
        selector,
        lookup,
        seed,
    } = case
    else {
        unreachable!()
    };
    if matches!(lookup, Lookup::Missing | Lookup::Empty | Lookup::Inactive) {
        return (0, Some(seed));
    }
    let byte = |row: u8, field| {
        if row == 0 {
            0
        } else {
            u32::from(data.records[usize::from(row)].byte(field).unwrap())
        }
    };
    let result = match reader {
        Reader::Sum13 => byte(first, 13) + byte(second, 13),
        Reader::SumFields => match selector as u8 {
            10..=13 => {
                (byte(first, selector as u8 as usize - 1)
                    + byte(second, selector as u8 as usize - 1))
                    & 255
            }
            _ => 0,
        },
        Reader::Flags => {
            let shift = (selector as u8).wrapping_sub(1);
            if shift >= 32 {
                0
            } else {
                ((byte(first, 7) | byte(second, 7)) >> shift) & 1
            }
        }
        Reader::Choice14 => {
            let a = byte(first, 14);
            let b = byte(second, 14);
            if a == 0 {
                b
            } else if b == 0 {
                a
            } else if a == 1 {
                b
            } else if b == 1 {
                a
            } else {
                let next = seed.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
                return (if (next >> 16) & 1 == 1 { a } else { b }, Some(next));
            }
        }
    };
    (result, Some(seed))
}
pub(super) fn check(m: &mut Machine, data: &Metadata16, case: Case, live: &[u8]) {
    match case {
        Case::Display(row) => {
            assert_eq!(m.cpu.regs[4], data.records[row].display_index(), "{case:?}")
        }
        Case::Alias(row) => {
            let alias = data.records[row].alias_u16;
            // Row zero is deliberately excluded: the original bias points before this table.
            assert_eq!(m.cpu.regs[5], u32::from(alias));
            assert_eq!(
                m.cpu.regs[4],
                data.records[usize::from(alias) - 375].display_index(),
                "{case:?}"
            );
        }
        Case::Pair { .. } => {
            let (value, seed) = expected(data, case);
            assert_eq!(m.cpu.regs[0], value, "{case:?}");
            assert_eq!(m.bus.read32(RNG), seed.unwrap(), "RNG {case:?}");
            for i in 4..13 {
                assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32, "callee {case:?} r{i}");
            }
        }
    }
    assert_eq!(&m.bus.ewram[0x25000..0x25060], live);
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 0..8 {
        assert_eq!(m.bus.read8(STACK - 32 + i), 0x5a);
    }
    for i in 0..32 {
        assert_eq!(m.bus.read8(STACK + i), 0x5a);
    }
}
pub(super) fn run(
    m: &mut Machine,
    data: &Metadata16,
    case: Case,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case);
    let live = m.bus.ewram[0x25000..0x25060].to_vec();
    let mut n = 0;
    while m.cpu.regs[15] != stop(case) {
        assert!(n < 250, "metadata16 {case:?} at{:08x}", m.cpu.regs[15]);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    check(m, data, case, &live);
    n
}
#[test]
#[ignore = "private EUR metadata16 source/display/alias/live-pair oracle; reduced Wine only, Linux/USA pending"]
fn european_metadata16_matches_original_display_alias_and_live_pair_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA16_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA16_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA16_RECORDS").unwrap()).unwrap(),
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
        (1, SOURCE_OFFSET, 2048)
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Metadata16 = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..6 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["alias_u16"] = 65536.into(),
            3 => bad["records"][0]["bytes_02_0f"][0] = (-1).into(),
            4 => bad["records"][0]["bytes_02_0f"][0] = 256.into(),
            _ => {
                bad["records"][0]["bytes_02_0f"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
        }
        assert!(serde_json::from_value::<Metadata16>(bad).is_err());
    }
    for kind in 0..3 {
        let mut bad = data.clone();
        match kind {
            0 => bad.source_offset += 16,
            1 => {
                bad.records.pop();
            }
            _ => bad.records.push(bad.records[0].clone()),
        }
        assert!(bad.validate().is_err());
    }
    for kind in 0..5 {
        let mut bad = data.clone();
        match kind {
            0 => bad.records[1].alias_u16 ^= 1,
            1 => bad.records[1].bytes_02_0f[2] = 1,
            2 => bad.records[1].bytes_02_0f[5] ^= 1,
            3 => bad.records[1].bytes_02_0f[13] ^= 1,
            _ => bad.records.swap(1, 2),
        }
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut m = crate::make_machine(rom, None);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    let mut n = 0;
    let cases = cases();
    for &case in &cases {
        n += run(&mut m, data, case, &mut visited);
    }
    // Isolated machine-owned fixtures exercise absent field14==1 and u8 wrapping.
    // They are never published or accepted by source authentication.
    let mut synthetic = data.clone();
    synthetic.records[1].bytes_02_0f[0] = 255;
    synthetic.records[1].bytes_02_0f[5] = 0x81;
    synthetic.records[2].bytes_02_0f[5] = 0x42;
    for field in 9..=13 {
        synthetic.records[1].bytes_02_0f[field - 2] = 240;
        synthetic.records[2].bytes_02_0f[field - 2] = 40;
    }
    let mut synthetic_count = 0;
    for (a, b) in [(1, 2), (2, 1), (1, 1)] {
        synthetic.records[1].bytes_02_0f[12] = a;
        synthetic.records[2].bytes_02_0f[12] = b;
        let at = SOURCE_OFFSET as usize;
        m.bus.rom[at..END_OFFSET as usize].copy_from_slice(&synthetic.source_bytes());
        assert!(synthetic.verify_source(&m.bus.rom).is_err());
        n += run(
            &mut m,
            &synthetic,
            pair(Reader::Choice14, 1, 2, 0),
            &mut visited,
        );
        synthetic_count += 1;
    }
    let mut extra = vec![
        Case::Display(1),
        Case::Alias(1),
        pair(Reader::Sum13, 1, 2, 0),
    ];
    extra.extend((10..=13).map(|p| pair(Reader::SumFields, 1, 2, p)));
    extra.extend(
        [0, 1, 2, 7, 8, 9, 32, 33, 255]
            .into_iter()
            .map(|p| pair(Reader::Flags, 1, 2, p)),
    );
    for case in extra {
        n += run(&mut m, &synthetic, case, &mut visited);
        synthetic_count += 1;
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    for (i, (&actual, &old)) in m.bus.ewram.iter().zip(ewram.iter()).enumerate() {
        if !(0x28c8..0x28cc).contains(&i)
            && !(0x3c28..0x3c2a).contains(&i)
            && !(0x25000..0x25060).contains(&i)
        {
            assert_eq!(actual, old, "EWRAM {i:x}");
        }
    }
    for (i, (&actual, &old)) in m.bus.iwram.iter().zip(iwram.iter()).enumerate() {
        if !(0x34b0..0x34b4).contains(&i) && !(0x7de0..0x7e20).contains(&i) {
            assert_eq!(actual, old, "IWRAM {i:x}");
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("metadata16.json"), bytes).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "records":128,"source_bytes":2048,"original_cases":cases.len(),"synthetic_cases":synthetic_count,"instructions":n,"distinct_instructions":visited.len(),
        "display_and_nonzero_alias_paths":true,"all_source_rows_live_pair_sums_flags_and_choice":true,
        "list_first_second_missing_empty_and_inactive_preconditions":true,"original_rng_state_and_choice":true,
        "synthetic_absent_one_priority_u8wrap_and_shift_edges":true,"strict_json_extent_and_coherent_source_guards":true,
        "live_list_outer_ram_stack_and_callee_guards":true,"managed_runtime":"pending; current dataset e24-d23",
        "semantic_family":"unproved; structural names retained","alias_zero_precondition":"not claimed; original bias points before table",
        "linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
