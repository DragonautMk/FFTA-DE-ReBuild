//! One original-code oracle for the bounded EUR progression/price source.
use super::*;
use gba_core::{
    rom_data::{
        items,
        progression_rewards::{
            self as rewards, Catalog, ProgressionRewards, END_OFFSET, SOURCE_OFFSET,
        },
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const STACK: u32 = 0x03007e00;
const SLOT: u32 = 0x0201f530;
const LEVELS: u32 = 0x020021b4;
const CONTEXT: u32 = 0x02024000;
const FLAGS: u32 = 0x02025000;
const CONTEXT_POINTER: u32 = 0x02025020;
const RETURN: u32 = 0x08000300;
const ENTRIES: [u32; 8] = [
    0x0804b56c, 0x0804b57e, 0x0804b590, 0x0804b5a2, 0x0804b5b4, 0x0804b5c6, 0x0804b5d8, 0x0804b5ea,
];
const LOADS: [u32; 8] = [
    0x0804b56e, 0x0804b580, 0x0804b592, 0x0804b5a4, 0x0804b5b6, 0x0804b5c8, 0x0804b5da, 0x0804b5f0,
];
const ENDS: [u32; 8] = [
    0x0804b57c, 0x0804b58e, 0x0804b5a0, 0x0804b5b2, 0x0804b5c4, 0x0804b5d6, 0x0804b5e8, 0x0804b5fc,
];
#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Publish {
        row: u32,
        default: bool,
    },
    Delta {
        row: u8,
        index: usize,
        level: u8,
        progress: u8,
        growth: u8,
        flags: u16,
    },
    Price {
        row: u8,
        mode: u32,
        item: u32,
    },
    PriceMath {
        row: u8,
        item: u16,
        predicate: u32,
    },
}
impl Case {
    pub(super) fn stop(self) -> u32 {
        match self {
            Self::Publish { default: true, .. } => 0x0804b4b6,
            Self::Publish { .. } => 0x0804b462,
            Self::Delta { index, .. } => ENDS[index],
            Self::Price { .. } => RETURN,
            Self::PriceMath { .. } => 0x080cf6ec,
        }
    }
}
pub(super) fn cases(items: &items::Items) -> Vec<Case> {
    let flagged = items
        .records
        .iter()
        .position(|r| r.bytes_08_0f[4] & 8 != 0 && r.halfwords_00_06[2] > 0)
        .unwrap() as u32;
    let unflagged = items
        .records
        .iter()
        .position(|r| r.bytes_08_0f[4] & 8 == 0)
        .unwrap() as u32;
    let mut out = vec![Case::Publish {
        row: 0,
        default: true,
    }];
    for row in [0, 1, 98, 99, 100, 255] {
        out.push(Case::Publish {
            row,
            default: false,
        });
    }
    for row in 0..100u8 {
        out.push(Case::Price {
            row,
            mode: u32::from(row % 7),
            item: flagged,
        });
        for index in [0, 7] {
            out.push(Case::Delta {
                row,
                index,
                level: 17,
                progress: 95,
                growth: 8,
                flags: 0x8001,
            });
        }
    }
    for index in 0..8 {
        for (level, progress, growth) in [(98, 99, 1), (99, 255, 255), (255, 17, 1), (1, 250, 10)] {
            out.push(Case::Delta {
                row: 99,
                index,
                level,
                progress,
                growth,
                flags: 0x4001,
            });
        }
    }
    for row in [0, 99, 100, 255] {
        out.push(Case::Price {
            row,
            mode: 0x102,
            item: 0x10000 + flagged,
        });
        out.push(Case::Price {
            row,
            mode: 255,
            item: unflagged,
        });
    }
    // A few actual types exercise every original mode mask; no copied predicate value.
    let mut types = BTreeSet::new();
    for (item, r) in items.records.iter().enumerate() {
        if r.bytes_08_0f[4] & 8 != 0 && types.insert(r.bytes_08_0f[0]) {
            for mode in 2..=6 {
                out.push(Case::Price {
                    row: 99,
                    mode,
                    item: item as u32,
                });
            }
            if types.len() == 3 {
                break;
            }
        }
    }
    out
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.bus.ewram[0x21b0..0x21d0].fill(0x3c);
    m.bus.ewram[0x1f520..0x1f540].fill(0x3c);
    m.bus.ewram[0x23ff0..0x24050].fill(0x3c);
    m.bus.ewram[0x24ff0..0x25030].fill(0x3c);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    m.bus.reg_ie = 0;
    m.bus.ime = false;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.cpu.regs[15] = match c {
        Case::Publish { row, default } => {
            m.cpu.regs[4] = row;
            if default {
                0x0804b4b0
            } else {
                0x0804b454
            }
        }
        Case::Delta {
            row,
            index,
            level,
            progress,
            growth,
            flags,
        } => {
            m.cpu.regs[0] = CONTEXT;
            m.cpu.regs[4] = SLOT;
            m.cpu.regs[5] = CONTEXT_POINTER;
            m.cpu.regs[6] = FLAGS;
            m.bus
                .write32(SLOT, 0x08000000 + SOURCE_OFFSET + u32::from(row) * 10);
            m.bus.write32(CONTEXT_POINTER, CONTEXT);
            m.bus.write8(CONTEXT + 0x2a + index as u32 + 1, growth);
            m.bus.write8(LEVELS + 2 * (index as u32 + 1), level);
            m.bus.write8(LEVELS + 2 * (index as u32 + 1) + 1, progress);
            m.bus.write16(FLAGS, flags);
            ENTRIES[index]
        }
        Case::Price { row, mode, item } => {
            m.bus.write8(LEVELS, row);
            m.cpu.regs[0] = mode;
            m.cpu.regs[1] = item;
            0x080cf674
        }
        Case::PriceMath {
            row,
            item,
            predicate,
        } => {
            m.cpu.regs[0] = predicate;
            m.cpu.regs[4] =
                u32::from(m.bus.rom[SOURCE_OFFSET as usize + usize::from(row) * 10 + 8]);
            m.cpu.regs[5] = 0x08000000 + items::SOURCE_OFFSET + u32::from(item) * 32;
            0x080cf6ac
        }
    };
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &ProgressionRewards,
    items: &items::Items,
    original: &[u8],
    visited: &mut BTreeSet<(u32, bool)>,
) -> usize {
    setup(m, c);
    let preserved = m.cpu.regs;
    let mut ram = m.bus.ewram.to_vec();
    let mut count = 0;
    let mut source_reads = 0;
    let mut predicate = None;
    while m.cpu.regs[15] != c.stop() {
        assert!(count < 30000, "progression {c:?} at{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert((pc, m.cpu.thumb()));
        match c {
            Case::Delta { row, index, .. } if pc == LOADS[index] => {
                assert_eq!(
                    m.cpu.regs[1],
                    0x08000000 + SOURCE_OFFSET + u32::from(row) * 10
                );
                source_reads += 1;
            }
            Case::Price { row, .. } if pc == 0x080cf6a2 => {
                assert_eq!(
                    m.cpu.regs[0],
                    0x08000000 + SOURCE_OFFSET + u32::from(row) * 10 + 8
                );
                source_reads += 1;
            }
            Case::Price { .. } if pc == 0x080cf6ac => predicate = Some(m.cpu.regs[0]),
            _ => {}
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    match c {
        Case::Publish { row, default } => {
            let address =
                0x08000000 + SOURCE_OFFSET + if default { 0 } else { row.wrapping_mul(10) };
            assert_eq!(m.bus.read32(SLOT), address);
            ram[0x1f530..0x1f534].copy_from_slice(&address.to_le_bytes());
        }
        Case::Delta {
            row,
            index,
            level,
            progress,
            growth,
            flags,
        } => {
            assert_eq!(source_reads, 1);
            let result = data
                .record(u32::from(row))
                .unwrap()
                .apply_delta(index, level, progress, growth, flags)
                .unwrap();
            let at = 0x21b4 + 2 * (index + 1);
            ram[at] = result.level;
            ram[at + 1] = result.progress;
            ram[0x25000..0x25002].copy_from_slice(&result.flags.to_le_bytes());
            assert_eq!(
                m.bus.read32(STACK),
                u32::from(data.record(u32::from(row)).unwrap().deltas00_07[index])
            );
        }
        Case::Price { row, item, .. } => {
            let item = &items.records[item as u16 as usize];
            let gated = item.bytes_08_0f[4] & 8 != 0;
            assert_eq!(source_reads, usize::from(gated));
            assert_eq!(predicate.is_some(), gated);
            let expected = if let Some(r) = data.record(u32::from(row)) {
                r.price(item, predicate.unwrap_or(0))
            } else {
                rewards::price_with_discount(
                    item,
                    original[SOURCE_OFFSET as usize + usize::from(row) * 10 + 8],
                    predicate.unwrap_or(0),
                )
            };
            assert_eq!(m.cpu.regs[0], expected, "{c:?}");
        }
        Case::PriceMath {
            row,
            item,
            predicate,
        } => assert_eq!(
            m.cpu.regs[0],
            data.record(u32::from(row))
                .unwrap()
                .price(&items.records[item as usize], predicate),
            "{c:?}"
        ),
    }
    assert_eq!(&m.bus.ewram[..], &ram, "RAM {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    let first = if matches!(c, Case::PriceMath { .. }) {
        5
    } else {
        4
    };
    for i in first..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "r{i} {c:?}");
    }
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e04..0x7e20].iter().all(|&v| v == 0x5a));
    if !matches!(c, Case::Delta { .. }) {
        assert_eq!(&m.bus.iwram[0x7e00..0x7e04], &[0x5a; 4]);
    }
    count
}
#[test]
#[ignore = "private EUR1000B source proof; one reduced Wine oracle, Linux/USA pending"]
fn european_progression_rewards_match_original_live_delta_calls_and_prices() {
    let root = PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_PROGRESSION_REWARDS_ORIGINAL_ROOT").unwrap(),
    );
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: ProgressionRewards = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_REWARDS_RECORDS").unwrap())
            .unwrap(),
    )
    .unwrap();
    let items: items::Items = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_REWARDS_ITEMS").unwrap()).unwrap(),
    )
    .unwrap();
    items.verify_source(&rom).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    assert_eq!(
        (catalog.sources.len(), catalog.sources[0].length),
        (1, 1000)
    );
    assert_eq!(
        &data.source_bytes(),
        &rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        &serde_json::from_slice::<ProgressionRewards>(&bytes).unwrap(),
        data
    );
    for id in [100, 255, 256, u32::MAX] {
        assert!(data.record(id).is_none());
    }
    assert!(data.records[0].apply_delta(8, 0, 0, 0, 0).is_none());
    assert!(data.records[0].byte(10).is_none());
    let shape = serde_json::to_value(data).unwrap();
    for n in 0..4 {
        let mut v = shape.clone();
        match n {
            0 => v["extra"] = true.into(),
            1 => v["records"][0]["extra"] = true.into(),
            2 => v["records"][0]["deltas00_07"] = serde_json::to_value([0u8; 7]).unwrap(),
            _ => {
                v["records"][0].as_object_mut().unwrap().remove("byte09");
            }
        }
        assert!(serde_json::from_value::<ProgressionRewards>(v).is_err());
    }
    for n in 0..7 {
        let mut v = data.clone();
        match n {
            0 => v.source_offset += 1,
            1 => {
                v.records.pop();
            }
            2 => v.records.swap(0, 99),
            3 => v.records[99].id = 0,
            4 => v.records[0].source_offset += 1,
            5 => v.records[0].byte09 ^= 1,
            _ => v.records[0].discount08 ^= 1,
        }
        if n < 5 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let real = cases(&items);
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &items, &rom, &mut visited);
    }
    let mut synthetic = 0;
    for delta in [128, 255] {
        let mut modified = data.clone();
        modified.records[99].deltas00_07 = [delta; 8];
        modified.records[99].byte09 = 255;
        modified.validate().unwrap();
        assert!(modified.verify_source(&rom).is_err());
        m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
            .copy_from_slice(&modified.source_bytes());
        assert!(Catalog::from_rom(&m.bus.rom).is_err());
        for index in 0..8 {
            for level in [0, 98] {
                instructions += run(
                    &mut m,
                    Case::Delta {
                        row: 99,
                        index,
                        level,
                        progress: 99,
                        growth: 1,
                        flags: 0x8001,
                    },
                    &modified,
                    &items,
                    &rom,
                    &mut visited,
                );
                synthetic += 1;
            }
        }
    }
    // Isolated typed item inputs exercise signed division, unsigned minimum, zero and u8+10 wrap.
    for discount in [100, 246, 255] {
        let mut modified = data.clone();
        modified.records[99].discount08 = discount;
        modified.records[99].byte09 = 255;
        modified.validate().unwrap();
        assert!(modified.verify_source(&rom).is_err());
        m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
            .copy_from_slice(&modified.source_bytes());
        for (base, minimum) in [(65535, 1), (0, 0), (1, 65535)] {
            let mut modified_items = items.clone();
            let item = &mut modified_items.records[1];
            item.halfwords_00_06[2] = base;
            item.halfwords_00_06[3] = minimum;
            item.bytes_08_0f[4] |= 8;
            let at = items::SOURCE_OFFSET as usize + 32;
            m.bus.rom[at..at + 32].copy_from_slice(&item.source_bytes());
            for predicate in [0, 0x100, 0x101, 255] {
                instructions += run(
                    &mut m,
                    Case::PriceMath {
                        row: 99,
                        item: 1,
                        predicate,
                    },
                    &modified,
                    &modified_items,
                    &rom,
                    &mut visited,
                );
                synthetic += 1;
            }
        }
    }
    m.bus.rom.copy_from_slice(&rom);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("progression-rewards.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_bytes":1000,"records":100,"real_cases":real.len(),"original_publisher_prefixes":real.iter().filter(|c|matches!(c,Case::Publish{..})).count(),"original_delta_caller_and_full_helper":real.iter().filter(|c|matches!(c,Case::Delta{..})).count(),"original_complete_prices":real.iter().filter(|c|matches!(c,Case::Price{..})).count(),"synthetic_cases":synthetic,"instructions_coverage":instructions,"pcs":visited.len(),"original_live_growth_progress_level_u8_wrap_carry_cap99_invalid_old_flags":true,"original_item_getter_mode_masks_and_price_predicate":true,"signed_price_division_unsigned_minimum_zero_u8_discount_wrap":true,"parsed_items_source_authenticated":true,"strict_json_ids_order_offsets_all1000bytes_opaque09":true,"stack_callee_register_and_complete_ewram_guards":true,"outside_ids":"100/255 original raw source access, model None diagnostic only","unowned_tail_bytes":12,"whole_progression_caller_semantic_skill_names":"unclaimed","synthetic":"unauthenticated/unpublished","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
