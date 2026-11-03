//! One full original D2390 oracle; source scope, no managed/ordinary claim.
use super::*;
use gba_core::{
    rom_data::progression_selection_gates::{
        Catalog, Live, ProgressionSelectionGates, END_OFFSET, RESET_FLAGS, SOURCE_OFFSET,
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
const STACK: u32 = 0x03007e00;
pub(super) const RETURN: u32 = 0x08000300;
const OUTPUT: usize = 0x21c6;
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Case {
    pub(super) byte01: u8,
    pub(super) halfword02: u16,
    pub(super) levels: [u8; 8],
    pub(super) aggregate_flags: usize,
    pub(super) special36: bool,
    pub(super) linked_item: bool,
    pub(super) selector_flags: u8,
    pub(super) reset: u32,
}
fn flag(ram: &[u8], id: u16) -> bool {
    ram[0x1f70 + usize::from(id >> 3)] & (1 << (id & 7)) != 0
}
fn put_flag(ram: &mut [u8], id: u16, value: bool) {
    let byte = &mut ram[0x1f70 + usize::from(id >> 3)];
    let bit = 1 << (id & 7);
    *byte = (*byte & !bit) | if value { bit } else { 0 };
}
fn aggregate_ids(rom: &[u8]) -> Vec<u16> {
    let mut ids = vec![];
    for at in (0x6e9a5c..0x6e9cb6).step_by(2) {
        let id = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        if id == 0xffff {
            break;
        }
        ids.push(id.wrapping_add(0x2ff));
    }
    assert_eq!(ids.len(), 300);
    ids
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    let ids = aggregate_ids(&m.bus.rom);
    assert!(c.aggregate_flags <= ids.len());
    m.bus.ewram.fill(0x3c);
    m.bus.ewram[0x1f70..0x3f70].fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    for &id in &ids[..c.aggregate_flags] {
        put_flag(&mut m.bus.ewram, id, true);
    }
    put_flag(&mut m.bus.ewram, 0x36, c.special36);
    for (i, id) in RESET_FLAGS.into_iter().enumerate() {
        put_flag(&mut m.bus.ewram, id, c.selector_flags & (1 << i) != 0);
    }
    m.bus.write8(0x020021b4, c.byte01);
    m.bus.write16(0x02001f6c, c.halfword02);
    for (i, v) in c.levels.into_iter().enumerate() {
        m.bus.write8(0x020021b6 + (2 * i) as u32, v);
    }
    m.bus
        .write32(0x020028c8, if c.linked_item { 0x02025000 } else { 0 });
    if c.linked_item {
        m.bus.write32(0x02025000, 0x02025100);
        m.bus.write32(0x02025008, 0);
        // Original linked predicate masks the high byte to two bits.
        m.bus.write16(0x02025100, 0xfd7f);
    }
    m.bus.ewram[OUTPUT] = 0xa5;
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[0] = c.reset;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x080d2390;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &ProgressionSelectionGates,
    visited: &mut BTreeSet<u32>,
    fields: &mut BTreeSet<(u8, u8)>,
) -> usize {
    setup(m, c);
    let ids = aggregate_ids(&m.bus.rom);
    let preserved = m.cpu.regs;
    let mut ram = m.bus.ewram.to_vec();
    if c.reset as u8 != 0 {
        for id in RESET_FLAGS {
            put_flag(&mut ram, id, false);
        }
    }
    let aggregate = ids.iter().filter(|&&id| flag(&ram, id)).count()
        + usize::from(flag(&ram, 0x36) || c.linked_item);
    let result = data.evaluate(
        Live {
            byte01: c.byte01,
            halfword02: c.halfword02,
            aggregate04: aggregate as u16,
            levels: c.levels,
        },
        |id| flag(&ram, id),
    );
    if let Some(value) = result.output {
        ram[OUTPUT] = value;
    }
    let mut stored_row = None;
    let mut count = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(count < 50000, "selection {c:?} at{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert(pc);
        let field = match pc {
            0x080d23ea => Some(1),
            0x080d23fa => Some(2),
            0x080d23fc => Some(3),
            0x080d2414 => Some(4),
            0x080d2416 => Some(5),
            0x080d2432 => Some(6 + m.cpu.regs[1] as u8),
            0x080d246e => Some(14),
            0x080d2470 => Some(15),
            0x080d24b0 => Some(0),
            _ => None,
        };
        if let Some(field) = field {
            let offset = m.cpu.regs[4]
                .checked_sub(0x08000000 + SOURCE_OFFSET)
                .unwrap();
            assert!(offset < 1024 && offset % 16 == 0);
            fields.insert(((offset / 16 + 1) as u8, field));
        }
        if matches!(pc, 0x080d2490 | 0x080d24b4) {
            stored_row = Some(m.cpu.regs[9] as u8);
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert_eq!(stored_row, result.matched_index, "terminal row {c:?}");
    assert_eq!(&m.bus.ewram[..], &ram, "full RAM {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "callee r{i} {c:?}");
    }
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    count
}
fn set_source(m: &mut Machine, data: &ProgressionSelectionGates) {
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
}
fn blocked(data: &ProgressionSelectionGates) -> ProgressionSelectionGates {
    let mut result = data.clone();
    for r in &mut result.records {
        r.selector00 = 255;
        r.minimum01 = 255;
        r.minimum02 = 0;
        r.minimum04 = 0;
        r.any_minima06_0d = [0; 8];
        r.flag0e = 0;
    }
    result
}
#[test]
#[ignore = "private EUR1024B original progression selection; one Wine oracle, Linux/USA pending"]
fn european_progression_selection_matches_original_any_minima_reset_zero_terminal_and_exhaustion() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SELECTION_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: ProgressionSelectionGates = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SELECTION_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    assert_eq!(
        (catalog.sources.len(), catalog.sources[0].length),
        (1, 1024)
    );
    assert_eq!(
        &data.source_bytes(),
        &rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<ProgressionSelectionGates>(&bytes).unwrap(),
        *data
    );
    for id in [0, 65, 255, 256, u32::MAX] {
        assert!(data.record(id).is_none());
    }
    assert_eq!(data.record(1).unwrap(), &data.records[0]);
    assert_eq!(data.record(64).unwrap(), &data.records[63]);
    assert!(data.records[0].byte(16).is_none());
    let shape = serde_json::to_value(data).unwrap();
    for n in 0..4 {
        let mut v = shape.clone();
        match n {
            0 => v["extra"] = true.into(),
            1 => v["records"][0]["extra"] = true.into(),
            2 => v["records"][0]["any_minima06_0d"] = serde_json::to_value([0u8; 7]).unwrap(),
            _ => {
                v["records"][0].as_object_mut().unwrap().remove("flag0e");
            }
        }
        assert!(serde_json::from_value::<ProgressionSelectionGates>(v).is_err());
    }
    for n in 0..9 {
        let mut v = data.clone();
        match n {
            0 => v.source_offset -= 16,
            1 => {
                v.records.pop();
            }
            2 => v.records.swap(0, 63),
            3 => v.records[63].id = 0,
            4 => v.records[0].source_offset += 1,
            5 => v.records[0].selector00 ^= 255,
            6 => v.records[63].minimum02 = 65535,
            7 => v.records[63].flag0e = 65535,
            _ => v.records[63].any_minima06_0d[7] = 128,
        }
        if n < 5 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    let mut real = vec![Case::default()];
    for selector_flags in [1, 2, 4, 8, 15] {
        for reset in [0, 1, 0x100, 0x101] {
            real.push(Case {
                selector_flags,
                reset,
                ..Case::default()
            });
        }
    }
    for byte01 in [19, 20, 34, 35, 59, 60, 84, 85, 98, 99, 128, 255] {
        real.push(Case {
            byte01,
            ..Case::default()
        });
    }
    for halfword02 in [9, 10, 49, 50, 99, 100, 199, 200, 499, 500, 32768, 65535] {
        real.push(Case {
            halfword02,
            ..Case::default()
        });
    }
    for aggregate_flags in [19, 20, 69, 70, 119, 120, 199, 200, 299, 300] {
        real.push(Case {
            aggregate_flags,
            ..Case::default()
        });
    }
    for (special36, linked_item) in [(true, false), (false, true), (true, true)] {
        real.push(Case {
            aggregate_flags: 299,
            special36,
            linked_item,
            ..Case::default()
        });
    }
    for field in 0..8 {
        for value in [9, 10, 19, 20, 39, 40, 69, 70, 128, 255] {
            let mut levels = [0; 8];
            levels[field] = value;
            real.push(Case {
                levels,
                ..Case::default()
            });
        }
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited, &mut fields);
    }
    let real_fields = fields.len();
    assert!(fields.iter().all(|&(row, _)| row <= 52));
    assert!(visited.contains(&0x080d2490) && visited.contains(&0x080d24b4));
    let mut synthetic_count = 0;
    // Every physical row, including the original unreachable tail, can be selected.
    for index in 0..64 {
        let mut v = blocked(data);
        v.records[index].minimum01 = 128;
        v.records[index].selector00 = if index % 2 == 0 { 0 } else { 255 };
        v.validate().unwrap();
        assert!(v.verify_source(&rom).is_err());
        set_source(&mut m, &v);
        instructions += run(
            &mut m,
            Case {
                byte01: 128,
                ..Case::default()
            },
            &v,
            &mut visited,
            &mut fields,
        );
        synthetic_count += 1;
    }
    let mut v = blocked(data);
    let row = &mut v.records[63];
    row.minimum01 = 128;
    row.minimum02 = 0x8000;
    row.minimum04 = 300;
    row.any_minima06_0d = [128; 8];
    row.flag0e = 0x5a9;
    set_source(&mut m, &v);
    let pass = Case {
        byte01: 128,
        halfword02: 0x8000,
        aggregate_flags: 300,
        selector_flags: 8,
        ..Case::default()
    };
    // Any one of eight active minima passes; all eight fail together.
    for field in 0..8 {
        let mut levels = [127; 8];
        levels[field] = 128;
        instructions += run(
            &mut m,
            Case { levels, ..pass },
            &v,
            &mut visited,
            &mut fields,
        );
        synthetic_count += 1;
    }
    for c in [
        Case {
            levels: [127; 8],
            ..pass
        },
        Case {
            levels: [255; 8],
            byte01: 127,
            ..pass
        },
        Case {
            levels: [255; 8],
            halfword02: 0x7fff,
            ..pass
        },
        Case {
            levels: [255; 8],
            aggregate_flags: 299,
            ..pass
        },
        Case {
            levels: [255; 8],
            selector_flags: 0,
            ..pass
        },
        Case {
            levels: [255; 8],
            reset: 1,
            ..pass
        },
    ] {
        instructions += run(&mut m, c, &v, &mut visited, &mut fields);
        synthetic_count += 1;
    }
    // Explicit no-condition row overwrites the old output; full miss preserves it.
    let mut zero = blocked(data);
    zero.records[63].minimum01 = 0;
    set_source(&mut m, &zero);
    instructions += run(&mut m, Case::default(), &zero, &mut visited, &mut fields);
    synthetic_count += 1;
    set_source(&mut m, &blocked(data));
    instructions += run(
        &mut m,
        Case::default(),
        &blocked(data),
        &mut visited,
        &mut fields,
    );
    synthetic_count += 1;
    set_source(&mut m, data);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("progression-selection-gates.json"), bytes).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":1024,"records":64,"physical_indices":"1..64","real_cases":real.len(),
        "synthetic_cases":synthetic_count,"instructions_coverage":instructions,"pcs":visited.len(),
        "original_source_field_bytes_distinct":real_fields,"original_reachable_through_zero_sentinel":52,
        "full_original_d2390_reset_count_flag_helpers":true,"all64synthetic_physical_rows":true,
        "eight_active_minima_any_one_passes":true,"zero_condition_store0_distinct_exhaustion_no_store":true,
        "unsigned_u8_u16_high_values_selector_not_id_low8_reset":true,
        "full_ewram_only_original_reset_flags_and_output_changed_sp_callee_stack_guards":true,
        "strict_json_fixed_ids_order_offsets_exact_roundtrip_coherent_edits":true,
        "synthetic":"unauthenticated/unpublished","managed":"EUR e36-d35 unchanged",
        "linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
