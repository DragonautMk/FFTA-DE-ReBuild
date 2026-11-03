//! Original item selectors, bounded pair queries and publication prefixes.
use super::*;
use gba_core::{
    rom_data::{
        item_links::{Catalog, Links, Pair, Record, END_OFFSET, RECORD_COUNT, SOURCE_OFFSET},
        items::Items,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const UNIT: u32 = 0x02024000;
const STACK: u32 = 0x03007e00;
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Selector(u32),
    Query { row: usize, selector: u8, value: u8 },
    Publish { row: usize, slot: usize },
    SentinelGate { row: usize, slot: usize },
}
pub(super) fn cases(links: &Links) -> Vec<Case> {
    let mut out: Vec<_> = (0..376).map(Case::Selector).collect();
    out.extend([Case::Selector(0x10001), Case::Selector(0xffff0177)]);
    for (row, record) in links.records.iter().enumerate() {
        for (slot, pair) in record.slots[..usize::from(record.count)].iter().enumerate() {
            out.push(Case::Query {
                row,
                selector: pair.selector,
                value: pair.value,
            });
            out.push(Case::Query {
                row,
                selector: pair.selector,
                value: 255,
            });
            out.push(Case::Publish { row, slot });
        }
        out.push(Case::Query {
            row,
            selector: 254,
            value: 255,
        });
        out.push(Case::Query {
            row,
            selector: 0,
            value: 0,
        });
    }
    out
}
pub(super) fn setup(m: &mut Machine, case: Case, links: &Links) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[8] = UNIT;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    for i in 0..0x140 {
        m.bus.write8(UNIT + i, 0x25);
    }
    for i in 0..32 {
        m.bus.write8(STACK - 16 + i, 0x5a);
    }
    match case {
        Case::Selector(id) => {
            m.cpu.regs[0] = id;
            m.cpu.regs[15] = 0x080d0cb2;
        }
        Case::Query {
            row,
            selector,
            value,
        } => {
            m.cpu.regs[1] = 0x08000000 + SOURCE_OFFSET + 20 * row as u32;
            m.cpu.regs[5] = u32::from(value);
            m.bus.write8(UNIT + 7, selector);
            m.cpu.regs[15] = 0x080d0cc2;
        }
        Case::Publish { row, slot } | Case::SentinelGate { row, slot } => {
            let p = &links.records[row].slots[slot];
            m.bus.write8(
                UNIT + 7,
                if matches!(case, Case::SentinelGate { .. }) {
                    2
                } else {
                    p.selector
                },
            );
            m.cpu.regs[1] = u32::from(p.selector);
            m.cpu.regs[4] = 0x08000000 + SOURCE_OFFSET + 20 * row as u32 + 3 + 2 * slot as u32;
            m.cpu.regs[5] = UNIT;
            m.cpu.regs[15] = if p.selector == 255 {
                0x080d09f0
            } else {
                0x080d09cc
            };
            if matches!(case, Case::SentinelGate { .. }) {
                assert_eq!(p.selector, 255);
                m.cpu.regs[1] = m.cpu.regs[4] - 1;
                m.cpu.regs[15] = 0x080d09c4;
            }
        }
    }
}
pub(super) fn stops(case: Case, links: &Links) -> (u32, u32) {
    match case {
        Case::Selector(_) => (0x080d0cc2, 0x080d0cc2),
        Case::Query { .. } => (0x080d0c84, 0x080d0cea),
        Case::Publish { row, slot } | Case::SentinelGate { row, slot } => {
            let end = if links.records[row].slots[slot].selector == 255 {
                0x080d09fc
            } else {
                0x080d09e4
            };
            (end, end)
        }
    }
}
pub(super) fn check(m: &mut Machine, case: Case, links: &Links, items: &Items) {
    let mut unit = vec![0x25; 0x140];
    match case {
        Case::Selector(id) => {
            let row = usize::from(items.records[id as u16 as usize].bytes_18_1f[5]);
            assert!(std::ptr::eq(
                links.for_item(items, id).unwrap(),
                &links.records[row]
            ));
            assert_eq!(m.cpu.regs[1], 0x08000000 + SOURCE_OFFSET + 20 * row as u32);
            assert_eq!(m.cpu.regs[15], 0x080d0cc2);
        }
        Case::Query {
            row,
            selector,
            value,
        } => {
            unit[7] = selector;
            let r = &links.records[row];
            let found = r.find(selector, value);
            assert_eq!(
                m.cpu.regs[15],
                if found.is_some() {
                    0x080d0c84
                } else {
                    0x080d0cea
                }
            );
            assert_eq!(
                m.cpu.regs[2] as usize,
                found.unwrap_or(usize::from(r.count))
            );
            if let Some(slot) = found {
                assert_eq!(
                    m.cpu.regs[1],
                    0x08000000 + SOURCE_OFFSET + 20 * row as u32 + 2 + 2 * slot as u32
                );
                assert_eq!(m.cpu.regs[0], u32::from(value));
            }
        }
        Case::Publish { row, slot } | Case::SentinelGate { row, slot } => {
            let p = &links.records[row].slots[slot];
            unit[7] = if matches!(case, Case::SentinelGate { .. }) {
                2
            } else {
                p.selector
            };
            if p.selector == 255 {
                unit[0x3d] = p.value;
            } else {
                unit[0x40 + usize::from(p.value)] |= 0x80;
            }
        }
    }
    assert_eq!(&m.bus.ewram[0x24000..0x24140], unit);
    assert_eq!(m.cpu.regs[13], STACK);
    assert!((0..4).all(|i| m.bus.read8(STACK - 16 + i) == 0x5a));
    assert!((0..16).all(|i| m.bus.read8(STACK + i) == 0x5a));
}
pub(super) fn run(
    m: &mut Machine,
    case: Case,
    links: &Links,
    items: &Items,
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, case, links);
    let end = stops(case, links);
    let mut n = 0;
    while m.cpu.regs[15] != end.0 && m.cpu.regs[15] != end.1 {
        assert!(n < 500, "item link {case:?} at{:08x}", m.cpu.regs[15]);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    check(m, case, links, items);
    n
}
#[test]
#[ignore = "private EUR item-link inventory, original selectors/query/publication prefixes; Wine only, Linux/USA pending"]
fn european_item_links_match_original_selectors_queries_and_publication_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ITEM_LINK_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_LINK_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_LINK_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let items: Items = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_SOURCE_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let links = &catalog.decoded;
    links.validate_items(&items).unwrap();
    assert_eq!(&links.records, &independent);
    assert_eq!(
        links.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(inv["records"], RECORD_COUNT);
    assert_eq!(
        inv["source_sha256"],
        gba_core::ffta_rom::digest(&links.source_bytes())
    );
    assert_eq!(
        (
            catalog.sources.len(),
            catalog.sources[0].offset,
            catalog.sources[0].length
        ),
        (1, SOURCE_OFFSET, 4500)
    );
    let bytes = serde_json::to_vec_pretty(links).unwrap();
    let parsed: Links = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..6 {
        let mut bad = serde_json::to_value(links).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][1]["slots"][0]["unknown"] = 0.into(),
            3 => bad["records"][0]["slots"][0]["selector"] = 256.into(),
            4 => bad["records"][0]["opaque"] = (-1).into(),
            _ => {
                bad["records"][0]["slots"].as_array_mut().unwrap().pop();
            }
        }
        assert!(serde_json::from_value::<Links>(bad).is_err());
    }
    for kind in 0..4 {
        let mut bad = links.clone();
        match kind {
            0 => bad.source_offset += 20,
            1 => {
                bad.records.pop();
            }
            2 => bad.records.push(bad.records[0].clone()),
            _ => bad.records[0].count = 10,
        }
        assert!(bad.validate().is_err());
    }
    let mut wrong_owner = items.clone();
    wrong_owner.records[1].bytes_18_1f[5] = 255;
    assert!(links.validate_items(&wrong_owner).is_err());
    for kind in 0..4 {
        let mut changed = links.clone();
        match kind {
            0 => changed.records[1].opaque = 1,
            1 => changed.records[1].slots[8].value = 1,
            2 => changed.records[1].slots[0].selector ^= 1,
            _ => changed.records.swap(1, 2),
        }
        changed.validate().unwrap();
        assert!(changed.verify_source(&rom).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut m = crate::make_machine(rom, None);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    let cases = cases(links);
    let mut n = 0;
    for &case in &cases {
        n += run(&mut m, case, links, &items, &mut visited);
    }
    // Isolated edge contract: zero active selector, duplicates and nonzero unused
    // slots do not get normalized or leaked past count. Never publish this data.
    let mut synthetic = links.clone();
    synthetic.records[1].count = 4;
    synthetic.records[1].opaque = 0xa5;
    synthetic.records[1].slots = std::array::from_fn(|_| Pair {
        selector: 254,
        value: 136,
    });
    for (at, (selector, value)) in [(0, 7), (2, 3), (2, 3), (255, 5)].into_iter().enumerate() {
        synthetic.records[1].slots[at] = Pair { selector, value };
    }
    let at = SOURCE_OFFSET as usize + 20;
    m.bus.rom[at..at + 20].copy_from_slice(&synthetic.records[1].source_bytes());
    assert!(synthetic.verify_source(&m.bus.rom).is_err());
    for (selector, value) in [(0, 7), (2, 3), (255, 5), (254, 136)] {
        n += run(
            &mut m,
            Case::Query {
                row: 1,
                selector,
                value,
            },
            &synthetic,
            &items,
            &mut visited,
        );
    }
    m.bus.rom[at..at + 20].copy_from_slice(&links.records[1].source_bytes());
    assert_eq!(&m.bus.ewram[..0x24000], &ewram[..0x24000]);
    assert_eq!(&m.bus.ewram[0x24140..], &ewram[0x24140..]);
    assert_eq!(&m.bus.iwram[..0x7df0], &iwram[..0x7df0]);
    assert_eq!(&m.bus.iwram[0x7e10..], &iwram[0x7e10..]);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("item-links.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":225,"source_bytes":4500,"active_pairs":472,"item_selector_paths":378,"query_paths":1394,"normal_publication_prefixes":464,"sentinel_publication_prefixes":8,"original_cases":cases.len(),"synthetic_zero_duplicate_padding_queries":4,"instructions":n,"distinct_instructions":visited.len(),"all_item_links_and_count_pair_storage_guards":true,"strict_json_extent_owner_and_coherent_source_guards":true,"live_unit_publication_and_outside_ram_stack_guards":true,"managed_runtime":"pending; current dataset e23-d22","whole_downstream_job_ability_calls":"not claimed; prefixes stop before those calls","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "private EUR new FF predecessor gate only; main item-link oracle closed; Wine only"]
fn european_item_link_sentinel_publication_follows_original_context_mismatch_gate() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ITEM_LINK_ORIGINAL_ROOT").unwrap());
    let links: Links =
        serde_json::from_slice(&fs::read(root.join("item-links.json")).unwrap()).unwrap();
    let items: Items = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_SOURCE_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    let mut m = crate::make_machine(rom, None);
    let mut visited = BTreeSet::new();
    let mut count = 0;
    let mut instructions = 0;
    for (row, record) in links.records.iter().enumerate() {
        for (slot, p) in record.slots[..usize::from(record.count)].iter().enumerate() {
            if p.selector == 255 {
                instructions += run(
                    &mut m,
                    Case::SentinelGate { row, slot },
                    &links,
                    &items,
                    &mut visited,
                );
                count += 1;
            }
        }
    }
    assert_eq!(count, 8);
    let report = root.join("sentinel-gate-result.json");
    assert!(!report.exists());
    fs::write(report,serde_json::to_vec_pretty(&serde_json::json!({"cases":count,"instructions":instructions,"actual_selector_load_context_mismatch_ff_branch_and_unit_store":true,"unit_selector":2,"main_source_oracle_repeated":false,"linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
