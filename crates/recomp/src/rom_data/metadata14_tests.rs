//! Original getters, live palette books/copy and bounded unit initialization prefix.
use super::*;
use gba_core::{
    rom_data::{
        metadata14::{Catalog, Metadata14, Record, END_OFFSET, SOURCE_OFFSET},
        metadata52::Metadata52,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const UNIT: u32 = 0x02024000;
pub(super) const DESCRIPTOR: u32 = 0x02025000;
pub(super) const STACK: u32 = 0x03007e00;
pub(super) const RETURN: u32 = 0x08000300;
const PALETTE: usize = 0x3a60;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Case {
    Get { id: u8, field: u8, argument: u32 },
    Palette { id: u8, argument: u32, book: u8 },
    Initialize { id: u8, override_job: u8, aux: u8 },
}
impl Case {
    pub(super) fn id(self) -> u8 {
        match self {
            Self::Get { id, .. } | Self::Palette { id, .. } | Self::Initialize { id, .. } => id,
        }
    }
    pub(super) fn stop(self) -> u32 {
        if matches!(self, Self::Initialize { .. }) {
            0x080cd0b6
        } else {
            RETURN
        }
    }
}
pub(super) fn cases(data: &Metadata14) -> Vec<Case> {
    let mut out = vec![];
    for r in &data.records {
        for (field, argument) in [(4, 0), (6, 0), (11, 0), (12, 1), (12, 0)] {
            out.push(Case::Get {
                id: r.id,
                field,
                argument,
            });
        }
    }
    for id in [2, 94] {
        for argument in [0x10001, 0xffff0000, 0xffff00ff] {
            out.push(Case::Get {
                id,
                field: 12,
                argument,
            });
        }
    }
    for argument in [0, 1] {
        let mut rows = vec![];
        for r in [
            data.records
                .iter()
                .find(|r| r.palette(argument).source_index.is_none()),
            data.records
                .iter()
                .find(|r| r.palette(argument).source_index.is_some()),
            data.records
                .iter()
                .filter(|r| r.palette(argument).source_index.is_some())
                .max_by_key(|r| r.palette(argument).bank),
            data.records
                .iter()
                .filter(|r| r.palette(argument).source_index.is_some())
                .max_by_key(|r| r.palette(argument).source_index),
        ]
        .into_iter()
        .flatten()
        {
            if !rows.contains(&r.id) {
                rows.push(r.id);
            }
        }
        for id in rows {
            for book in [0, 1, 2] {
                out.push(Case::Palette { id, argument, book });
            }
        }
    }
    if let Some(r) = data
        .records
        .iter()
        .find(|r| r.palette(1).source_index.is_some())
    {
        out.push(Case::Palette {
            id: r.id,
            argument: 0x10001,
            book: 3,
        });
    }
    for id in [2, 20, 23, 76, 94] {
        for override_job in [0, 2] {
            out.push(Case::Initialize {
                id,
                override_job,
                aux: 0x85,
            });
        }
    }
    out.push(Case::Initialize {
        id: 2,
        override_job: 255,
        aux: 0xff,
    });
    out
}
fn descriptor(case: Case) -> [u8; 48] {
    let mut d = [0u8; 48];
    d[0] = case.id();
    if let Case::Initialize {
        override_job, aux, ..
    } = case
    {
        d[1] = override_job;
        d[2] = aux;
    }
    d[5..8].copy_from_slice(&[0x22, 0x44, 0x66]);
    d[18..20].copy_from_slice(&0xfedcu16.to_le_bytes());
    d[42..44].copy_from_slice(&0x0555u16.to_le_bytes());
    d[45] = 0x81;
    d
}
pub(super) fn setup(m: &mut Machine, case: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    for a in STACK - 256..STACK + 32 {
        m.bus.write8(a, 0x5a);
    }
    for a in UNIT - 16..UNIT + 0x108 + 16 {
        m.bus.write8(a, 0x3c);
    }
    for a in DESCRIPTOR - 16..DESCRIPTOR + 48 + 16 {
        m.bus.write8(a, 0x3c);
    }
    for (i, v) in descriptor(case).into_iter().enumerate() {
        m.bus.write8(DESCRIPTOR + i as u32, v);
    }
    m.bus.write8(UNIT + 4, case.id());
    m.bus.write8(UNIT + 5, 2);
    m.bus.write8(UNIT + 7, 2);
    m.bus.iwram[PALETTE..PALETTE + 512].fill(0xa5);
    m.bus.write8(
        0x02002fc2,
        0xa0 | if let Case::Palette { book, .. } = case {
            book
        } else {
            0
        },
    );
    m.cpu.regs[0] = UNIT;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = match case {
        Case::Get {
            field, argument, ..
        } => {
            m.cpu.regs[1] = argument;
            match field {
                4 => 0x080cf144,
                6 => 0x080cf180,
                11 => 0x080cf1bc,
                12 => 0x080cf1f0,
                _ => unreachable!(),
            }
        }
        Case::Palette { argument, .. } => {
            m.cpu.regs[1] = argument;
            0x080cf08c
        }
        Case::Initialize { .. } => {
            m.cpu.regs[1] = DESCRIPTOR;
            m.cpu.regs[2] = 0x101;
            0x080ccf9c
        }
    };
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    data: &Metadata14,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case);
    let record = data.record(u32::from(case.id())).unwrap();
    let mut n = 0;
    let mut selector_seen = None;
    let mut palette_bytes = None;
    let mut copy_calls = 0;
    while m.cpu.regs[15] != case.stop() {
        assert!(n < 4000, "metadata14 {case:?} at{:08x}", m.cpu.regs[15]);
        if let Case::Palette { argument, .. } = case {
            let selection = record.palette(argument);
            if matches!(m.cpu.regs[15], 0x080cf0d0 | 0x080cf10c) {
                selector_seen = Some(m.cpu.regs[0] as u8);
                assert_eq!(selector_seen, selection.source_index);
            }
            if matches!(m.cpu.regs[15], 0x080cf0d4 | 0x080cf110) {
                let at = m.cpu.regs[0].checked_sub(0x08000000).unwrap() as usize;
                assert!(at + 32 <= m.bus.rom.len());
                palette_bytes = Some(m.bus.rom[at..at + 32].to_vec());
            }
            if matches!(m.cpu.regs[15], 0x080cf0dc | 0x080cf118) {
                assert_eq!(
                    m.cpu.regs[0],
                    0x03000000 + PALETTE as u32 + u32::from(selection.bank) * 32
                );
                assert_eq!(m.cpu.regs[2], 32);
                copy_calls += 1;
            }
        }
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    let mut expected_unit = [0x3c; 0x108];
    expected_unit[4] = case.id();
    expected_unit[5] = 2;
    expected_unit[7] = 2;
    let mut expected_palette = vec![0xa5; 512];
    match case {
        Case::Get {
            field, argument, ..
        } => {
            let expected = match field {
                4 => u32::from(record.halfwords04_06[0]),
                6 => u32::from(record.halfwords04_06[1]),
                11 => u32::from(record.byte0b),
                12 => u32::from(record.bytes0c_0d[usize::from(argument as u8 != 1)]),
                _ => unreachable!(),
            };
            assert_eq!(m.cpu.regs[0], expected, "{case:?}");
        }
        Case::Palette { argument, .. } => {
            let selection = record.palette(argument);
            assert_eq!(m.cpu.regs[0], u32::from(selection.bank));
            assert_eq!(selector_seen, selection.source_index);
            assert_eq!(copy_calls, usize::from(selection.source_index.is_some()));
            if let Some(bytes) = palette_bytes {
                let a = usize::from(selection.bank) * 32;
                expected_palette[a..a + 32].copy_from_slice(&bytes);
            }
        }
        Case::Initialize {
            override_job, aux, ..
        } => {
            expected_unit.fill(0);
            let flags = 0x0555u16
                | if (20..=23).contains(&case.id()) {
                    0x1000
                } else {
                    0
                };
            expected_unit[0x28..0x2a].copy_from_slice(&flags.to_le_bytes());
            expected_unit[4] = case.id();
            expected_unit[5] = record.selectors01_02[0];
            expected_unit[7] = if override_job == 0 {
                record.selectors01_02[1]
            } else {
                override_job
            };
            expected_unit[8] = if override_job == 0 {
                record.byte03
            } else {
                aux
            };
            expected_unit[0x100] = 0x81;
            expected_unit[0x105] = 0xdc;
            assert_eq!(m.cpu.regs[13], STACK - 0x38);
            assert_eq!(m.cpu.regs[7], UNIT);
            assert_eq!(m.cpu.regs[8], DESCRIPTOR);
            assert_eq!(m.cpu.regs[10], 1);
        }
    }
    assert_eq!(
        &m.bus.ewram[0x24000..0x24108],
        &expected_unit,
        "unit {case:?}"
    );
    assert_eq!(&m.bus.ewram[0x25000..0x25030], &descriptor(case));
    assert_eq!(&m.bus.iwram[PALETTE..PALETTE + 512], expected_palette);
    if !matches!(case, Case::Initialize { .. }) {
        assert_eq!(m.cpu.regs[13], STACK);
        for i in 4..12 {
            assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32, "callee r{i} {case:?}");
        }
    }
    for a in [UNIT - 16, UNIT + 0x108, DESCRIPTOR - 16, DESCRIPTOR + 48] {
        for i in 0..16 {
            assert_eq!(m.bus.read8(a + i), 0x3c);
        }
    }
    for a in STACK - 256..STACK - 128 {
        assert_eq!(m.bus.read8(a), 0x5a);
    }
    for a in STACK..STACK + 32 {
        assert_eq!(m.bus.read8(a), 0x5a);
    }
    n
}
#[test]
#[ignore = "private EUR metadata14 source/getter/palette/initializer proof; reduced Wine only, Linux/USA pending"]
fn european_metadata14_matches_original_getters_live_palette_books_and_initializer_prefix() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_METADATA14_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA14_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA14_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let owner: Metadata52 = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_METADATA52_SOURCE_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate_metadata52(&owner).unwrap();
    assert_eq!(data.records, independent);
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(
        inv["source_sha256"],
        gba_core::ffta_rom::digest(&data.source_bytes())
    );
    assert_eq!(
        (catalog.sources.len(), catalog.sources[0].length),
        (1, 1302)
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Metadata14 = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..6 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["byte00"] = 256.into(),
            3 => bad["records"][0]["halfwords04_06"][0] = 65536.into(),
            4 => {
                bad["records"][0]["packed08_0a"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            _ => bad["records"][0]["byte03"] = (-1).into(),
        };
        assert!(serde_json::from_value::<Metadata14>(bad).is_err());
    }
    for kind in 0..8 {
        let mut bad = data.clone();
        match kind {
            0 => bad.address_bias += 28,
            1 => bad.source_offset -= 28,
            2 => bad.first_id = 0,
            3 => {
                bad.records.pop();
            }
            4 => bad.records.push(bad.records[0].clone()),
            5 => bad.records[0].id = 1,
            6 => bad.records[0].source_offset += 14,
            _ => bad.records.swap(0, 1),
        };
        assert!(bad.validate().is_err());
    }
    let mut bad = data.clone();
    bad.records[0].selectors01_02[1] = 116;
    assert!(bad.validate_metadata52(&owner).is_err());
    let mut unproved = data.clone();
    unproved.records[0].selectors01_02[0] = 255;
    unproved.validate_metadata52(&owner).unwrap();
    assert!(unproved.verify_source(&rom).is_err());
    for kind in 0..4 {
        let mut bad = data.clone();
        match kind {
            0 => bad.records[0].byte00 ^= 1,
            1 => bad.records[20].halfwords04_06[1] ^= 1,
            2 => bad.records[40].packed08_0a[2] ^= 1,
            _ => bad.records[92].bytes0c_0d[1] ^= 1,
        };
        bad.validate_metadata52(&owner).unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    assert!(data.record(0).is_none() && data.record(1).is_none() && data.record(95).is_none());
    assert_eq!(data.record(0x10002).unwrap().id, 2);
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut m = crate::make_machine(rom, None);
    let mut visited = BTreeSet::new();
    let initialization = super::ui_palette_tests::initialize(&mut m, &mut visited);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let cases = cases(data);
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut m, case, data, &mut visited);
    }
    let mut synthetic = data.clone();
    let packed_a = data
        .records
        .iter()
        .find(|r| r.palette(1).source_index.is_some())
        .unwrap()
        .packed08_0a;
    let packed_b = data
        .records
        .iter()
        .find(|r| r.palette(0).source_index.is_some())
        .unwrap()
        .packed08_0a;
    let r = &mut synthetic.records[0];
    r.byte00 = 0xa5;
    r.selectors01_02 = [250, 115];
    r.byte03 = 255;
    r.halfwords04_06 = [0x8001, 0xfedc];
    r.byte0b = 255;
    r.bytes0c_0d = [128, 255];
    r.packed08_0a = [
        (packed_a[0] & 0xf0) | 15,
        (15 << 4) | (packed_a[1] & 15),
        packed_b[2],
    ];
    synthetic.validate_metadata52(&owner).unwrap();
    assert!(synthetic.verify_source(&m.bus.rom).is_err());
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&synthetic.source_bytes());
    let mut synthetic_cases = vec![];
    for (field, argument) in [(4, 0), (6, 0), (11, 0), (12, 1), (12, 0)] {
        synthetic_cases.push(Case::Get {
            id: 2,
            field,
            argument,
        });
    }
    for argument in [0, 1] {
        for book in [0, 1, 2] {
            synthetic_cases.push(Case::Palette {
                id: 2,
                argument,
                book,
            });
        }
    }
    for override_job in [0, 255] {
        synthetic_cases.push(Case::Initialize {
            id: 2,
            override_job,
            aux: 0x80,
        });
    }
    for &case in &synthetic_cases {
        instructions += run(&mut m, case, &synthetic, &mut visited);
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    for (i, (&a, &v)) in m.bus.ewram.iter().zip(&ewram).enumerate() {
        if !((0x23ff0..0x24118).contains(&i) || (0x24ff0..0x25040).contains(&i) || i == 0x2fc2) {
            assert_eq!(a, v, "EWRAM{i:x}");
        }
    }
    for (i, (&a, &v)) in m.bus.iwram.iter().zip(&iwram).enumerate() {
        if !((0x7d00..0x7e20).contains(&i) || (PALETTE..PALETTE + 512).contains(&i)) {
            assert_eq!(a, v, "IWRAM{i:x}");
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("metadata14.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":93,"logical_ids":[2,94],"source_bytes":1302,"original_cases":cases.len(),"original_getter_cases":cases.iter().filter(|c|matches!(c,Case::Get{..})).count(),"original_palette_cases":cases.iter().filter(|c|matches!(c,Case::Palette{..})).count(),"original_initializer_prefixes":cases.iter().filter(|c|matches!(c,Case::Initialize{..})).count(),"synthetic_cases":synthetic_cases.len(),"instructions":instructions,"distinct_instructions":visited.len(),"initialization_instructions":initialization,"strict_json_extent_bias_ids_order_source_byte02_owner_guards":true,"byte01_unproved_owner_not_constrained":true,"all93_original_getters_u16_u8_argument_truncation":true,"live_palette_books_0_1_2_default3_packed_selector_bank_original_copy32_guards":true,"initializer_descriptor_override_byte02_fallback_flags_outer_ram_prefix_guards":true,"synthetic_high_u16_u8_bank15_unused_fields_and_unproved_selector_preserved":true,"normal_ids0_1_and_upper95plus":"outside owned source; original job branch/unbounded guest behavior retained","trailing2bytes":"unowned","managed_runtime":"pending; e30-d29unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
