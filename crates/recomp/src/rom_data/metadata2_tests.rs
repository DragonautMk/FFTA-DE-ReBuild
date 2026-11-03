//! Original direct/inverse readers plus live-byte caller prefix.
use super::*;
use gba_core::{
    rom_data::{
        metadata2::{
            Catalog, Metadata2, Record, END_OFFSET, INVERSE_SCAN_COUNT, RECORD_COUNT, SOURCE_OFFSET,
        },
        metadata52::Metadata52,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
const UNIT: u32 = 0x02024000;
const STACK: u32 = 0x03007e00;
pub(super) const RETURN: u32 = 0x08000300;
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Direct(u32),
    Inverse(u32),
    Live(u8),
}
pub(super) fn cases() -> Vec<Case> {
    let mut v: Vec<_> = (0..RECORD_COUNT as u32).map(Case::Direct).collect();
    v.extend([Case::Direct(0x10000), Case::Direct(0xffff002b)]);
    v.extend((0..256).map(Case::Inverse));
    v.extend([Case::Inverse(0x10000), Case::Inverse(0xffff00ff)]);
    v.extend((0..RECORD_COUNT as u8).map(Case::Live));
    v
}
pub(super) fn stop(case: Case) -> u32 {
    if matches!(case, Case::Live(_)) {
        0x080cbd7c
    } else {
        RETURN
    }
}
pub(super) fn setup(m: &mut Machine, case: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    for i in 0..32 {
        m.bus.write8(STACK - 16 + i, 0x5a);
        m.bus.write8(UNIT + i, 0x3c);
    }
    match case {
        Case::Direct(index) => {
            m.cpu.regs[0] = index;
            m.cpu.regs[15] = 0x080ccd48;
        }
        Case::Inverse(value) => {
            m.cpu.regs[0] = value;
            m.cpu.regs[15] = 0x080ccd58;
        }
        Case::Live(index) => {
            m.bus.write8(UNIT + 7, index);
            m.cpu.regs[0] = UNIT;
            m.cpu.regs[15] = 0x080cbd74;
        }
    }
}
pub(super) fn check(m: &mut Machine, case: Case, data: &Metadata2) {
    let expected = match case {
        Case::Direct(index) => u32::from(data.forward(index).unwrap()),
        Case::Inverse(value) => u32::from(data.inverse(value)),
        Case::Live(index) => u32::from(data.forward(u32::from(index)).unwrap()),
    };
    assert_eq!(m.cpu.regs[0], expected, "{case:?}");
    assert_eq!(m.cpu.regs[15], stop(case));
    if let Case::Inverse(value) = case {
        let first = data.records[..INVERSE_SCAN_COUNT]
            .iter()
            .position(|r| r.value == value as u8);
        assert_eq!(
            m.cpu.regs[2] as usize,
            first.unwrap_or(INVERSE_SCAN_COUNT),
            "first-match {case:?}"
        );
    }
    assert_eq!(
        m.cpu.regs[13],
        if matches!(case, Case::Live(_)) {
            STACK - 4
        } else {
            STACK
        }
    );
    if matches!(case, Case::Live(_) | Case::Inverse(_)) {
        assert_eq!(m.bus.read32(STACK - 4), RETURN | 1);
    }
    for i in 4..13 {
        assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32, "callee r{i} {case:?}");
    }
    let mut unit = [0x3c; 32];
    if let Case::Live(index) = case {
        unit[7] = index;
    }
    assert_eq!(&m.bus.ewram[0x24000..0x24020], &unit);
    for i in 0..12 {
        assert_eq!(m.bus.read8(STACK - 16 + i), 0x5a);
    }
    for i in 0..16 {
        assert_eq!(m.bus.read8(STACK + i), 0x5a);
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    data: &Metadata2,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case);
    let mut n = 0;
    while m.cpu.regs[15] != stop(case) {
        assert!(n < 512, "metadata2 {case:?} at{:08x}", m.cpu.regs[15]);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    check(m, case, data);
    n
}
#[test]
#[ignore = "private EUR metadata2 direct/inverse/live-byte source check; reduced Wine only, Linux/USA pending"]
fn european_metadata2_matches_original_direct_inverse_and_live_byte_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA2_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA2_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA2_RECORDS").unwrap()).unwrap(),
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
        (1, SOURCE_OFFSET, 88)
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Metadata2 = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..4 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["key"] = 256.into(),
            _ => bad["records"][0]["value"] = (-1).into(),
        };
        assert!(serde_json::from_value::<Metadata2>(bad).is_err());
    }
    for kind in 0..3 {
        let mut bad = data.clone();
        match kind {
            0 => bad.source_offset += 2,
            1 => {
                bad.records.pop();
            }
            _ => bad.records.push(bad.records[0].clone()),
        };
        assert!(bad.validate().is_err());
    }
    let mut bad_key = data.clone();
    bad_key.records[0].key = 116;
    assert!(bad_key.validate_metadata52(&owner).is_err());
    for kind in 0..4 {
        let mut bad = data.clone();
        match kind {
            0 => bad.records[0].key ^= 1,
            1 => bad.records[1].value = 255,
            2 => bad.records[43].value = 254,
            _ => bad.records.swap(2, 3),
        };
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    assert!(data.forward(44).is_none());
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
    let mut synthetic = data.clone();
    synthetic.records[0] = Record {
        key: 255,
        value: 255,
    };
    synthetic.records[2] = Record {
        key: 250,
        value: 200,
    };
    synthetic.records[3] = Record {
        key: 251,
        value: 200,
    };
    synthetic.records[43] = Record {
        key: 43,
        value: 254,
    };
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&synthetic.source_bytes());
    assert!(synthetic.verify_source(&m.bus.rom).is_err());
    assert!(synthetic.validate_metadata52(&owner).is_err());
    let synthetic_cases = [
        Case::Direct(0),
        Case::Direct(2),
        Case::Direct(3),
        Case::Direct(43),
        Case::Inverse(0),
        Case::Inverse(200),
        Case::Inverse(255),
        Case::Inverse(254),
        Case::Inverse(203),
        Case::Inverse(0x100c8),
        Case::Live(0),
        Case::Live(2),
        Case::Live(43),
    ];
    assert_eq!(
        (synthetic.forward(43), synthetic.inverse(254)),
        (Some(254), 0)
    );
    assert_eq!(synthetic.inverse(200), 250);
    for &case in &synthetic_cases {
        instructions += run(&mut m, case, &synthetic, &mut visited);
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    assert_eq!(&m.bus.ewram[..0x24000], &ewram[..0x24000]);
    assert_eq!(&m.bus.ewram[0x24020..], &ewram[0x24020..]);
    for (i, (&actual, &old)) in m.bus.iwram.iter().zip(iwram.iter()).enumerate() {
        if !(0x7df0..0x7e10).contains(&i) {
            assert_eq!(actual, old, "IWRAM{i:x}");
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("metadata2.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":44,"inverse_scan_records":43,"source_bytes":88,"original_cases":cases.len(),"synthetic_cases":synthetic_cases.len(),"instructions":instructions,"distinct_instructions":visited.len(),"all44direct_and_live_indices_all256inverse_values_u8truncation":true,"inverse_first_stored_key_and_duplicate_aliases_miss0":true,"last43record_preserved_forward_readable_but_excluded_inverse":true,"synthetic_high_byte_duplicate_stored_keys_and_last_only_match":true,"strict_json_extent_source_parsed_key_owner_guards":true,"callee_stack_live_record_and_outer_ram_guards":true,"live_wrapper_scope":"caller prefix stopped after map call, frame retained","direct_indices_beyond43":"outside owned-source proof; no guest bounds/fallback invented","managed_runtime":"pending; current dataset e27-d26","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
