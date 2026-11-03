//! One source-only oracle for the five EUR descriptor/price records.
use super::*;
use gba_core::{
    rom_data::trade_parameters::{Catalog, TradeParameters, END_OFFSET, SOURCE_OFFSET},
    Machine,
};
use std::collections::BTreeSet;
pub(super) const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007e00;
const VALUE_SOURCE: usize = 0x713b3c;

#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Descriptor(u32),
    Price {
        selector: u32,
        count: u32,
        flags: u8,
        value_id: u32,
        injected_base: Option<u32>,
    },
    Count(u8),
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    m.bus.ewram.fill(0x3c);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
    m.cpu.regs[15] = match c {
        Case::Descriptor(selector) => {
            m.cpu.regs[0] = selector;
            0x080d0744
        }
        Case::Price {
            selector,
            count,
            flags,
            value_id,
            ..
        } => {
            m.cpu.regs[0] = value_id;
            m.cpu.regs[1] = selector;
            m.cpu.regs[2] = count;
            m.bus.ewram[0x2cb5 + 12 * usize::from(selector as u8)] = flags;
            0x080d26ac
        }
        Case::Count(n) => {
            for i in 0..30 {
                m.bus.ewram[0x2cc1 + 12 * i] = if i < usize::from(n) {
                    0x83
                } else {
                    [0, 1, 2, 0xfc][i % 4]
                };
            }
            0x080d2728
        }
    };
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &TradeParameters,
    visited: &mut BTreeSet<(u32, bool)>,
    fields: &mut BTreeSet<usize>,
) -> usize {
    setup(m, c);
    let ram = m.bus.ewram.to_vec();
    let preserved = m.cpu.regs;
    let base = match c {
        Case::Price { value_id, .. } => {
            u32::from(m.bus.rom[VALUE_SOURCE + usize::from(value_id as u16) * 70 + 0x3e]) * 200
        }
        _ => 0,
    };
    let expected = match c {
        Case::Descriptor(selector) => data.descriptor(selector),
        Case::Price {
            selector,
            count,
            flags,
            injected_base,
            ..
        } => data
            .price(injected_base.unwrap_or(base), selector, count, flags)
            .unwrap(),
        Case::Count(n) => u32::from(n),
    };
    let mut count = 0;
    let mut source_reads = 0;
    let mut getter_reads = 0;
    let mut flag_reads = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(count < 30000, "{c:?} pc{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert((pc, m.cpu.thumb()));
        if let Case::Price {
            selector,
            count: slot,
            flags,
            value_id,
            injected_base,
        } = c
        {
            if pc == 0x080d225e {
                assert_eq!(
                    m.cpu.regs[0],
                    0x08000000 + VALUE_SOURCE as u32 + u32::from(value_id as u16) * 70 + 0x3e
                );
                getter_reads += 1;
            }
            if pc == 0x080d26c0 {
                assert_eq!(m.cpu.regs[0], base, "original D1F30 property36");
                if let Some(value) = injected_base {
                    // Separate arithmetic-tail coverage, never authenticated source behavior.
                    m.cpu.regs[0] = value;
                }
            }
            if pc == 0x080d26da {
                assert_eq!(
                    m.cpu.regs[0] + 1,
                    0x02002cb5 + 12 * u32::from(selector as u8)
                );
                flag_reads += 1;
            }
            if matches!(pc, 0x080d26f0 | 0x080d270c | 0x080d2710) {
                let row = data.record(u32::from(selector as u8)).unwrap();
                let field = if flags & 3 == 3 {
                    0x41 + u32::from(slot as u8)
                } else if pc == 0x080d270c {
                    0x40
                } else {
                    0x41
                };
                let at = 0x08000000 + row.source_offset + field;
                assert_eq!(m.cpu.regs[0], at, "exact original field {c:?}");
                if m.bus.rom_resources().is_none() {
                    assert_eq!(
                        m.bus.rom[(at - 0x08000000) as usize],
                        row.byte(field as usize).unwrap()
                    );
                }
                fields.insert((at - 0x08000000 - SOURCE_OFFSET) as usize);
                source_reads += 1;
            }
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert_eq!(m.cpu.regs[0], expected, "return {c:?}");
    if let Case::Price {
        flags,
        injected_base,
        ..
    } = c
    {
        let zero = injected_base.unwrap_or(base) == 0;
        assert_eq!(getter_reads, 1);
        assert_eq!(flag_reads, usize::from(!zero));
        assert_eq!(
            source_reads,
            if zero {
                0
            } else if flags & 3 == 3 {
                1
            } else {
                2
            }
        );
    }
    assert_eq!(&m.bus.ewram[..], &ram, "read-only complete EWRAM {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "callee r{i} {c:?}");
    }
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    count
}
fn set_source(m: &mut Machine, data: &TradeParameters) {
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
}

#[test]
#[ignore = "private EUR480B original selector/price/count oracle; Wine only, Linux/USA pending"]
fn european_trade_parameters_match_original_selectors_prices_alias_count_and_unsigned_math() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TRADE_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: TradeParameters = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_TRADE_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    assert_eq!((catalog.sources.len(), catalog.sources[0].length), (1, 480));
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    for i in 0..480 {
        assert_eq!(data.byte(i), Some(rom[SOURCE_OFFSET as usize + i]));
    }
    assert_eq!(data.byte(480), None);
    assert_eq!(data.byte(usize::MAX), None);
    for id in [0, 1, 7, 255, u32::MAX] {
        assert!(data.record(id).is_none());
    }
    assert!(data.price(1, 7, 0, 0).is_none());
    assert!(data.price(1, 2, 31, 3).is_none());
    assert_eq!(data.price(0, 255, 255, 3), Some(0));
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<TradeParameters>(&bytes).unwrap(),
        *data
    );
    let shape = serde_json::to_value(data).unwrap();
    for n in 0..7 {
        let mut v = shape.clone();
        match n {
            0 => v["unknown"] = true.into(),
            1 => v["records"][0]["unknown"] = true.into(),
            2 => {
                v["records"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("markup40_41");
            }
            3 => {
                v["records"][0]["bytes00_1f"].as_array_mut().unwrap().pop();
            }
            4 => {
                v["records"][4]["discounts42_5f"]
                    .as_array_mut()
                    .unwrap()
                    .push(0.into());
            }
            5 => v["records"][0]["markup40_41"] = 65536u32.into(),
            _ => v["records"][0]["discounts42_5f"][0] = 256u32.into(),
        }
        assert!(serde_json::from_value::<TradeParameters>(v).is_err());
    }
    for n in 0..11 {
        let mut v = data.clone();
        match n {
            0 => v.source_offset += 1,
            1 => {
                v.records.pop();
            }
            2 => v.records.swap(0, 4),
            3 => v.records[0].source_offset += 1,
            4 => v.records[4].id = 2,
            5 => v.records[0].bytes00_1f[0] ^= 1,
            6 => v.records[4].bytes20_3f[31] ^= 1,
            7 => v.records[0].markup40_41 ^= 1,
            8 => v.records[4].discounts42_5f[29] ^= 1,
            9 => v.records[0].markup40_41 = 65535,
            _ => v.records[4].discounts42_5f[0] = 255,
        }
        if n < 5 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    assert!(Catalog::from_rom(&[]).is_err());
    let positive = (0..300u32)
        .max_by_key(|&id| rom[VALUE_SOURCE + id as usize * 70 + 0x3e])
        .unwrap();
    let zero = (0..300u32)
        .find(|&id| rom[VALUE_SOURCE + id as usize * 70 + 0x3e] == 0)
        .unwrap();
    let mut real = (0..256).map(Case::Descriptor).collect::<Vec<_>>();
    for id in [0, 1, 2, 6, 7, 255] {
        real.push(Case::Descriptor(0xabcd0100 + id));
    }
    for selector in 2..=6 {
        for count in 0..=30 {
            real.push(Case::Price {
                selector,
                count,
                flags: 0x83,
                value_id: positive,
                injected_base: None,
            });
        }
        for flags in [0, 1, 2, 0xfc, 0xff] {
            real.push(Case::Price {
                selector: selector + 0x100,
                count: if flags == 0xff { 0x11e } else { 255 },
                flags,
                value_id: positive + 0x10000,
                injected_base: None,
            });
        }
        for value_id in [0, 1, 147, 299] {
            real.push(Case::Price {
                selector,
                count: 1,
                flags: 3,
                value_id,
                injected_base: None,
            });
        }
    }
    for selector in [0, 1, 2, 6, 7, 255] {
        real.push(Case::Price {
            selector,
            count: 255,
            flags: 3,
            value_id: zero,
            injected_base: None,
        });
    }
    for n in 0..=30 {
        real.push(Case::Count(n));
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited, &mut fields);
    }
    assert_eq!(
        fields.len(),
        160,
        "all32price bytes per record, opaque64 roundtrip only"
    );
    let real_fields = fields.clone();
    let mut synthetic_count = 0;
    let mut injected_count = 0;
    // Isolated source edits are unauthenticated/unpublished. D1F30 source stays a separate raw owner.
    for markup in [0, 1, 0x8001, 65535] {
        let mut edited = data.clone();
        for r in &mut edited.records {
            r.markup40_41 = markup;
            r.bytes00_1f.fill(0xa5);
            r.bytes20_3f.fill(0x5a);
            r.discounts42_5f = std::array::from_fn(|i| [0, 1, 99, 100, 101, 255][i % 6]);
        }
        edited.validate().unwrap();
        assert!(edited.verify_source(&rom).is_err());
        set_source(&mut m, &edited);
        m.bus.rom[VALUE_SOURCE + positive as usize * 70 + 0x3e] = 255;
        assert!(Catalog::from_rom(&m.bus.rom).is_err());
        for selector in 2..=6 {
            for (flags, count) in [
                (0, 255),
                (1, 0),
                (2, 30),
                (3, 0),
                (3, 1),
                (3, 6),
                (3, 30),
                (0xff, 0x100),
            ] {
                instructions += run(
                    &mut m,
                    Case::Price {
                        selector,
                        count,
                        flags,
                        value_id: positive,
                        injected_base: None,
                    },
                    &edited,
                    &mut visited,
                    &mut fields,
                );
                synthetic_count += 1;
            }
        }
        if markup == 65535 {
            for base in [0, 1, 0x80000000, 0xffff0001, u32::MAX] {
                for (flags, count) in [(0, 255), (3, 0), (3, 6)] {
                    instructions += run(
                        &mut m,
                        Case::Price {
                            selector: 2,
                            count,
                            flags,
                            value_id: positive,
                            injected_base: Some(base),
                        },
                        &edited,
                        &mut visited,
                        &mut fields,
                    );
                    injected_count += 1;
                }
            }
        }
    }
    set_source(&mut m, data);
    m.bus.rom[VALUE_SOURCE + positive as usize * 70 + 0x3e] =
        rom[VALUE_SOURCE + positive as usize * 70 + 0x3e];
    assert_eq!(fields, real_fields);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("trade-parameters.json"), bytes).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":480,"records":5,"physical_ids":[2,3,4,5,6],"real_cases":real.len(),
        "synthetic_source_cases":synthetic_count,"separate_injected_arithmetic_operand_cases":injected_count,
        "instructions_coverage":instructions,"pcs":visited.len(),"original_price_field_bytes":real_fields.len(),
        "opaque_bytes_roundtrip_only":320,"full_original_d0744_d26ac_d1f30_146474_and_d2728":true,
        "descriptor_low8_null_outside_and_count30_live_ram_rows":true,
        "original_separate70B_property36_byte3e_times200_no_item_catalog_claim":true,
        "discount_count0_aliases_markup_high_byte41_and_tail_count1_to30":true,
        "base_zero_skips_source_and_live_flags":true,"unsigned_divide_wrap32_product_add_sub":true,
        "full_ewram_read_only_sp_callee_stack_guards":true,"strict_json_order_ids_offsets_fields_source_guards":true,
        "synthetic":"unauthenticated/unpublished, injected operand cases explicitly separate",
        "managed":"EUR e38-d37 unchanged","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
