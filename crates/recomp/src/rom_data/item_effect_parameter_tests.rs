//! Original packed getters and live-current delta, without UI writes or source fallback.
use super::*;
use gba_core::{
    rom_data::{
        item_effect_parameters::{
            Catalog, ItemEffectParameters, END_OFFSET, FIRST_ITEM_ID, SOURCE_OFFSET,
        },
        items,
    },
    Machine,
};
use std::collections::BTreeSet;
pub(super) const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007e00;

#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Property {
        property: u32,
        index: u32,
    },
    Delta {
        slot: u32,
        item: u32,
        quantity: u32,
        current: u8,
    },
}
impl Case {
    fn getter(self) -> (u32, u32) {
        match self {
            Self::Property { property, index } => (property, index),
            Self::Delta { slot, item, .. } => {
                (slot, u32::from((item as u16).wrapping_sub(FIRST_ITEM_ID)))
            }
        }
    }
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
        Case::Property { property, index } => {
            m.cpu.regs[0] = property;
            m.cpu.regs[1] = index;
            0x080cfe28
        }
        Case::Delta {
            slot,
            item,
            quantity,
            current,
        } => {
            m.cpu.regs[0] = slot;
            m.cpu.regs[1] = item;
            m.cpu.regs[2] = quantity;
            m.bus.ewram[0x2e7f + 16 * usize::from(slot as u8)] = current;
            0x080cfda8
        }
    };
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &ItemEffectParameters,
    visited: &mut BTreeSet<(u32, bool)>,
    fields: &mut BTreeSet<usize>,
    external_cap: &mut BTreeSet<u32>,
) -> usize {
    setup(m, c);
    let ram = m.bus.ewram.to_vec();
    let preserved = m.cpu.regs;
    let (property, index) = c.getter();
    let expected = match c {
        Case::Property { .. } => data.property(property, index).unwrap(),
        Case::Delta {
            slot,
            item,
            quantity,
            current,
        } => {
            if let Some(value) = data.delta(slot, item, quantity, current) {
                u32::from(value)
            } else {
                // Explicit raw-cap alias probe, never a typed owner or diagnostic fallback.
                assert!(index >= 14 && (index as u8) < 14);
                let at = SOURCE_OFFSET as usize + index as usize * 7;
                let cap = (m.bus.rom[at + 5] >> 5) | ((m.bus.rom[at + 6] & 31) << 3);
                let delta = data.property(slot, index).unwrap() * u32::from(quantity as u8);
                let room = cap.saturating_sub(current);
                delta.min(u32::from(room))
            }
        }
    };
    let mut count = 0;
    let mut getter_reads = 0;
    let mut cap_reads = 0;
    let mut live_reads = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(count < 1000, "{c:?} pc{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert((pc, m.cpu.thumb()));
        let read = match pc {
            0x080cfea0 => Some((m.cpu.regs[1], 0, false)),
            0x080cfeb0 => Some((m.cpu.regs[0], 0, false)),
            0x080cfeb4 => Some((m.cpu.regs[0] + 1, 1, false)),
            0x080cfecc => Some((m.cpu.regs[1] + 1, 1, false)),
            0x080cfee0 => Some((m.cpu.regs[0] + 1, 1, false)),
            0x080cfee4 => Some((m.cpu.regs[0] + 2, 2, false)),
            0x080cfefc => Some((m.cpu.regs[0] + 2, 2, false)),
            0x080cff00 => Some((m.cpu.regs[0] + 3, 3, false)),
            0x080cff18 => Some((m.cpu.regs[1] + 3, 3, false)),
            0x080cff2c => Some((m.cpu.regs[0] + 3, 3, false)),
            0x080cff30 => Some((m.cpu.regs[0] + 4, 4, false)),
            0x080cff48 => Some((m.cpu.regs[1] + 4, 4, false)),
            0x080cff5c => Some((m.cpu.regs[1] + 5, 5, false)),
            0x080cfdea => Some((m.cpu.regs[0] + 5, 5, true)),
            0x080cfdee => Some((m.cpu.regs[0] + 6, 6, true)),
            _ => None,
        };
        if let Some((at, field, cap)) = read {
            let physical = if cap { index } else { u32::from(index as u8) };
            assert_eq!(
                at,
                0x08000000 + SOURCE_OFFSET + 7 * physical + field,
                "original address {c:?}"
            );
            if cap {
                cap_reads += 1;
            } else {
                getter_reads += 1;
            }
            if physical < 14 {
                let offset = (7 * physical + field) as usize;
                if m.bus.rom_resources().is_none() {
                    assert_eq!(
                        m.bus.rom[(at - 0x08000000) as usize],
                        data.byte(offset).unwrap()
                    );
                }
                fields.insert(offset);
            } else {
                assert!(
                    cap,
                    "only explicit raw-cap alias probes may leave declared owner"
                );
                external_cap.insert(at);
            }
        }
        if pc == 0x080cfddc {
            if let Case::Delta { slot, .. } = c {
                assert_eq!(m.cpu.regs[4], 0x02002e7f + 16 * u32::from(slot as u8));
            } else {
                panic!("getter must not access live current");
            }
            live_reads += 1;
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert_eq!(m.cpu.regs[0], expected, "return {c:?}");
    assert_eq!(
        getter_reads,
        match property as u8 {
            0..=1 | 5..=6 | 12..=13 | 16..=19 => 1,
            2..=4 | 7..=11 | 14..=15 => 2,
            _ => 0,
        }
    );
    assert_eq!(
        cap_reads,
        if matches!(c, Case::Delta { .. }) {
            2
        } else {
            0
        }
    );
    assert_eq!(live_reads, usize::from(matches!(c, Case::Delta { .. })));
    assert_eq!(&m.bus.ewram[..], &ram, "read-only complete EWRAM {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "callee r{i} {c:?}");
    }
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    count
}
fn set_source(m: &mut Machine, data: &ItemEffectParameters) {
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
}

#[test]
#[ignore = "private EUR98B original packed item getter/delta oracle; Wine only, Linux/USA pending"]
fn european_item_effect_parameters_match_original_packed_getters_delta_caps_and_index_widths() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ITEM_EFFECT_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: ItemEffectParameters = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_EFFECT_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    let owner: items::Items = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEM_EFFECT_ITEM_OWNER").unwrap()).unwrap(),
    )
    .unwrap();
    data.validate_items(&owner).unwrap();
    let mut missing_owner = owner.clone();
    missing_owner.records.pop();
    assert!(data.validate_items(&missing_owner).is_err());
    assert_eq!((catalog.sources.len(), catalog.sources[0].length), (1, 98));
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    for offset in 0..98 {
        assert_eq!(
            data.byte(offset),
            Some(rom[SOURCE_OFFSET as usize + offset])
        );
    }
    assert_eq!(data.byte(98), None);
    assert_eq!(data.byte(usize::MAX), None);
    for id in [14, 255, u32::MAX] {
        assert!(data.record(id).is_none());
    }
    assert_eq!(data.property(20, 255), Some(0));
    assert_eq!(data.property(0, 14), None);
    for item in [361, 376, 618, 65535] {
        assert_eq!(data.delta(0, item, 1, 0), None);
    }
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<ItemEffectParameters>(&bytes).unwrap(),
        *data
    );
    let shape = serde_json::to_value(data).unwrap();
    for case in 0..6 {
        let mut v = shape.clone();
        match case {
            0 => v["unknown"] = true.into(),
            1 => v["records"][0]["unknown"] = true.into(),
            2 => {
                v["records"][0].as_object_mut().unwrap().remove("cap45_52");
            }
            3 => {
                v["records"][0]["coefficients"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            4 => {
                v["records"][13]["coefficients"]
                    .as_array_mut()
                    .unwrap()
                    .push(0.into());
            }
            _ => v["records"][0]["cap45_52"] = 256u32.into(),
        }
        assert!(serde_json::from_value::<ItemEffectParameters>(v).is_err());
    }
    for case in 0..13 {
        let mut v = data.clone();
        match case {
            0 => v.source_offset += 1,
            1 => {
                v.records.pop();
            }
            2 => v.records.swap(0, 13),
            3 => v.records[0].source_offset += 1,
            4 => v.records[13].id = 0,
            5 => v.records[0].item_id = 361,
            6 => v.records[0].coefficients[0] = 32,
            7 => v.records[13].opaque53_55 = 8,
            8 => v.records[0].coefficients[0] ^= 1,
            9 => v.records[13].coefficients[8] ^= 1,
            10 => v.records[0].cap45_52 ^= 1,
            11 => v.records[13].opaque53_55 ^= 1,
            _ => v.records[0].coefficients.fill(31),
        }
        if case < 8 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    assert!(Catalog::from_rom(&[]).is_err());
    let mut real = vec![];
    for index in 0..14 {
        for property in 0..20 {
            real.push(Case::Property { property, index });
        }
        let cap = data.records[index as usize].cap45_52;
        for current in [0, cap.saturating_sub(1), cap, cap.saturating_add(1), 255] {
            for quantity in [0, 1, 255] {
                real.push(Case::Delta {
                    slot: 0,
                    item: 362 + index,
                    quantity,
                    current,
                });
            }
        }
        real.push(Case::Property {
            property: 0x112,
            index: 0x100 + index,
        });
    }
    for property in 0..20 {
        real.push(Case::Delta {
            slot: property,
            item: 375,
            quantity: 255,
            current: 0,
        });
    }
    for property in [20, 255] {
        for index in [0, 13, 14, 255] {
            real.push(Case::Property { property, index });
        }
        real.push(Case::Delta {
            slot: property,
            item: 362,
            quantity: 255,
            current: 1,
        });
    }
    for slot in [0x100, 0x113] {
        for quantity in [0x100, 0x101, 0x1ff] {
            real.push(Case::Delta {
                slot,
                item: 0x10000 + 375,
                quantity,
                current: 0,
            });
        }
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut external = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited, &mut fields, &mut external);
    }
    assert_eq!(fields.len(), 98);
    assert!(external.is_empty());
    let mut alias_count = 0;
    for item in [618, 631, 0x10000 + 618, 0x10000 + 631] {
        instructions += run(
            &mut m,
            Case::Delta {
                slot: 0,
                item,
                quantity: 255,
                current: 1,
            },
            data,
            &mut visited,
            &mut fields,
            &mut external,
        );
        alias_count += 1;
    }
    assert_eq!(
        external.len(),
        4,
        "two fullu16 cap indices, each reads2raw bytes"
    );
    let mut synthetic_count = 0;
    for mode in 0..5u8 {
        let mut edited = data.clone();
        for (i, r) in edited.records.iter_mut().enumerate() {
            r.coefficients =
                std::array::from_fn(|k| [0, 1, 15, 16, 31][(i + k + usize::from(mode)) % 5]);
            r.cap45_52 = [0, 1, 99, 100, 255][(i + usize::from(mode)) % 5];
            r.opaque53_55 = ((i + usize::from(mode)) & 7) as u8;
        }
        edited.validate_items(&owner).unwrap();
        assert!(edited.verify_source(&rom).is_err());
        set_source(&mut m, &edited);
        assert!(Catalog::from_rom(&m.bus.rom).is_err());
        for property in 0..20 {
            instructions += run(
                &mut m,
                Case::Property {
                    property,
                    index: 13,
                },
                &edited,
                &mut visited,
                &mut fields,
                &mut external,
            );
            synthetic_count += 1;
        }
        for index in [0, 13] {
            let cap = edited.records[index as usize].cap45_52;
            for (slot, current, quantity) in [
                (0, 0, 255),
                (19, cap.saturating_sub(1), 255),
                (20, cap, 255),
                (0, 255, 1),
                (18, 0, 0),
            ] {
                instructions += run(
                    &mut m,
                    Case::Delta {
                        slot,
                        item: 362 + index,
                        quantity,
                        current,
                    },
                    &edited,
                    &mut visited,
                    &mut fields,
                    &mut external,
                );
                synthetic_count += 1;
            }
        }
    }
    for opaque in 0..8 {
        let mut edited = data.clone();
        edited.records[0].opaque53_55 = opaque;
        edited.records[13].opaque53_55 = opaque;
        set_source(&mut m, &edited);
        for item in [362, 375] {
            instructions += run(
                &mut m,
                Case::Delta {
                    slot: 18,
                    item,
                    quantity: 255,
                    current: 0,
                },
                &edited,
                &mut visited,
                &mut fields,
                &mut external,
            );
            synthetic_count += 1;
        }
    }
    set_source(&mut m, data);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("item-effect-parameters.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":98,"records":14,"real_cases":real.len(),"raw_cap_alias_probe_cases":alias_count,
        "synthetic_cases":synthetic_count,"instructions_coverage":instructions,"pcs":visited.len(),
        "all98source_bytes_original_reads_and_roundtrip":true,"nine5bit_coefficients_cap8_opaque3_layout":true,
        "twenty_property_map_low8_and_invalid_property_no_source":true,
        "fullu16_item_minus362_cap_index_distinct_from_low8_getter":true,"external_raw_cap_read_addresses":external,
        "original_cfda8_readonly_live_current_quantity_cap_fit_zero_remaining":true,
        "full_ewram_read_only_sp_callee_stack_guards":true,"strict_json_bitwidth_id_item_order_offset_owner_source_guards":true,
        "synthetic":"unauthenticated/unpublished; out-of-owner cap probes explicit raw only",
        "managed":"EUR e39-d38 unchanged","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
