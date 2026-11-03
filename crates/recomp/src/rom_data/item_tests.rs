//! Typed item fields against original property code; no managed/runtime claim here.
use super::*;
use gba_core::{
    rom_data::items::{
        Catalog, Items, Record, END_OFFSET, PROPERTY_READER, RECORD_COUNT, SOURCE_OFFSET,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const RETURN: u32 = 0x08000300;
pub(super) const STACK: u32 = 0x03007e00;
#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub id: u32,
    pub property: u32,
    pub modifiers: [u8; 14],
}
pub(super) fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for bypass in [1, 0] {
        let mut modifiers = [0; 14];
        modifiers[1..5].copy_from_slice(&[255, 129, 17, 251]);
        modifiers[13] = bypass;
        for id in 0..RECORD_COUNT as u32 {
            for property in 0..19 {
                out.push(Case {
                    id,
                    property,
                    modifiers,
                });
            }
        }
    }
    for (property, id, _) in SPECIAL {
        for bypass in [0, 1, 128, 255] {
            for bonus in [0, 1, 255] {
                let mut modifiers = [bonus; 14];
                modifiers[13] = bypass;
                out.push(Case {
                    id,
                    property,
                    modifiers,
                });
            }
        }
    }
    for property in 0..19 {
        out.push(Case {
            id: 0xffff0177,
            property: 0xffffff00 + property,
            modifiers: [1; 14],
        });
    }
    for id in [0, 375, 65535, u32::MAX] {
        for property in [19, 255, 275, u32::MAX] {
            out.push(Case {
                id,
                property,
                modifiers: [0; 14],
            });
        }
    }
    out
}
const SPECIAL: [(u32, u32, usize); 4] =
    [(10, 0x33, 1), (11, 0x12e, 3), (12, 0x94, 2), (13, 0x11e, 4)];
pub(super) fn expected(items: &Items, case: Case) -> u32 {
    let property = case.property as u8;
    if property > 18 {
        return 0;
    }
    let id = case.id as u16;
    let raw = items.records[id as usize].raw_property(property);
    if (10..=13).contains(&property) && case.modifiers[13] == 0 {
        let value = SPECIAL
            .iter()
            .find(|&&(p, i, _)| p == u32::from(property) && i == u32::from(id))
            .map_or(raw, |&(_, _, at)| {
                raw.wrapping_add(u32::from(case.modifiers[at]))
            });
        value.min(255)
    } else {
        raw
    }
}
pub(super) fn setup(m: &mut Machine, case: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[0] = case.id;
    m.cpu.regs[1] = case.property;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = PROPERTY_READER;
    m.cpu.cpsr = 0xbf;
    for (i, &v) in case.modifiers.iter().enumerate() {
        m.bus.write8(0x02002030 + i as u32, v);
    }
    for i in 0..32 {
        m.bus.write8(STACK - 16 + i, 0x5a);
    }
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    items: &Items,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case);
    let mut n = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(
            n < 200,
            "item property path {case:?} at{:08x}",
            m.cpu.regs[15]
        );
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    assert_eq!(
        m.cpu.regs[0],
        expected(items, case),
        "original item property {case:?}"
    );
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..13 {
        assert_eq!(m.cpu.regs[i], 0x11220000 + i as u32);
    }
    assert!((0..4).all(|i| m.bus.read8(STACK - 16 + i) == 0x5a));
    assert!((0..16).all(|i| m.bus.read8(STACK + i) == 0x5a));
    for (i, &v) in case.modifiers.iter().enumerate() {
        assert_eq!(m.bus.read8(0x02002030 + i as u32), v);
    }
    n
}

#[test]
#[ignore = "private EUR item inventory and original property routines; Wine only, Linux/USA pending"]
fn european_item_catalog_matches_original_properties_and_live_modifiers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ITEM_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let items = &catalog.decoded;
    assert_eq!(&items.records, &independent);
    assert_eq!(
        items.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(inv["records"], RECORD_COUNT);
    assert_eq!(inv["source_offset"], SOURCE_OFFSET);
    assert_eq!(
        inv["source_sha256_range"],
        gba_core::ffta_rom::digest(&items.source_bytes())
    );
    assert_eq!(catalog.sources.len(), 1);
    assert_eq!(catalog.sources[0].offset, SOURCE_OFFSET);
    assert_eq!(catalog.sources[0].length, 12032);
    let bytes = serde_json::to_vec_pretty(items).unwrap();
    let parsed: Items = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..5 {
        let mut bad = serde_json::to_value(items).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["signed_10_17"][0] = 128.into(),
            3 => {
                bad["records"][0]["halfwords_00_06"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            _ => bad["records"][0]["bytes_18_1f"][0] = (-1).into(),
        }
        assert!(serde_json::from_value::<Items>(bad).is_err());
    }
    let mut missing = items.clone();
    missing.records.pop();
    assert!(missing.validate().is_err());
    let mut extra = items.clone();
    extra.records.push(extra.records[0].clone());
    assert!(extra.validate().is_err());
    let mut shifted = items.clone();
    shifted.source_offset += 32;
    assert!(shifted.validate().is_err());
    let mut edited = items.clone();
    edited.records[1].halfwords_00_06[0] ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    let mut changed_rom = rom.clone();
    changed_rom[SOURCE_OFFSET as usize] ^= 1;
    assert!(Catalog::from_rom(&changed_rom).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut m = crate::make_machine(rom, None);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    let cases = cases();
    let mut n = 0;
    for &case in &cases {
        n += run(&mut m, case, items, &mut visited);
    }
    // Shipped fields happen to be nonnegative. Isolate signed edge cases in
    // machine-owned bytes; never publish/authenticate this synthetic catalog.
    let mut negative = items.clone();
    let negative_ids = [1, 0x33, 0x12e, 0x94, 0x11e];
    for id in negative_ids {
        negative.records[id].signed_10_17 = [-128, -1, -100, -30, -1, -128, -100, -1];
        let offset = SOURCE_OFFSET as usize + id * 32;
        m.bus.rom[offset..offset + 32].copy_from_slice(&negative.records[id].source_bytes());
    }
    assert!(negative.verify_source(&m.bus.rom).is_err());
    let mut synthetic_calls = 0;
    for id in negative_ids {
        for property in 10..18 {
            for bypass in [0, 1] {
                for bonus in [0, 1, 255] {
                    let mut modifiers = [bonus; 14];
                    modifiers[13] = bypass;
                    n += run(
                        &mut m,
                        Case {
                            id: id as u32,
                            property,
                            modifiers,
                        },
                        &negative,
                        &mut visited,
                    );
                    synthetic_calls += 1;
                }
            }
        }
        let offset = SOURCE_OFFSET as usize + id * 32;
        m.bus.rom[offset..offset + 32].copy_from_slice(&items.records[id].source_bytes());
    }
    assert_eq!(&m.bus.ewram[..0x2030], &ewram[..0x2030]);
    assert_eq!(&m.bus.ewram[0x203e..], &ewram[0x203e..]);
    assert_eq!(&m.bus.iwram[..0x7df0], &iwram[..0x7df0]);
    assert_eq!(&m.bus.iwram[0x7e10..], &iwram[0x7e10..]);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("items.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":RECORD_COUNT,"source_bytes":12032,"property_selectors":19,"calls":cases.len(),"isolated_synthetic_negative_calls":synthetic_calls,"instructions":n,"distinct_instructions":visited.len(),"all_records_two_live_modifier_branches":true,"special_bonuses_signed_fields_unsigned_clamp_and_truncation":true,"invalid_property_and_stack_live_ram_guards":true,"strict_json_extent_and_coherent_source_guards":true,"managed_runtime":"pending; current dataset remains e22-d21","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
