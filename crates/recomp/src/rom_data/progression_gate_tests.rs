//! One original complete0487c8oracle; structural receipts, no downstream UI claim.
use super::*;
use gba_core::{
    rom_data::progression_gates::{Catalog, ProgressionGates, END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const STACK: u32 = 0x03007e00;
const RECEIVED: u32 = 0x02002c08;
const LEVELS: u32 = 0x020021b6;
const OUTPUT: u32 = 0x02025000;
pub(super) const RETURN: u32 = 0x08000300;
#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub levels: [u8; 8],
    pub received: u64,
}
pub(super) fn cases(data: &ProgressionGates) -> Vec<Case> {
    let mut out = vec![];
    for r in &data.records {
        out.push(Case {
            levels: r.requirements04_0b,
            received: !(1u64 << r.id),
        });
    }
    for field in 0..8 {
        let r = data
            .records
            .iter()
            .find(|r| r.requirements04_0b[field] > 0)
            .unwrap();
        let mut levels = r.requirements04_0b;
        levels[field] -= 1;
        out.push(Case {
            levels,
            received: !(1u64 << r.id),
        });
    }
    for (a, b) in [(0, 1), (31, 32), (32, 63), (0, 63)] {
        out.push(Case {
            levels: [255; 8],
            received: !((1u64 << a) | (1u64 << b)),
        });
    }
    for levels in [[0; 8], [255; 8]] {
        for received in [0, u64::MAX] {
            out.push(Case { levels, received });
        }
    }
    out
}
pub(super) fn setup(m: &mut Machine, c: Case) {
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.bus.ewram[0x21b0..0x21d0].fill(0x3c);
    m.bus.ewram[0x2c00..0x2c20].fill(0x3c);
    m.bus.ewram[0x24ff0..0x25020].fill(0x3c);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    m.bus.write32(RECEIVED, c.received as u32);
    m.bus.write32(RECEIVED + 4, (c.received >> 32) as u32);
    for (i, v) in c.levels.into_iter().enumerate() {
        m.bus.write8(LEVELS + (i * 2) as u32, v);
    }
    m.cpu.regs[0] = OUTPUT;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x080487c8;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &ProgressionGates,
    visited: &mut BTreeSet<(u32, bool)>,
    fields: &mut BTreeSet<(usize, usize)>,
) -> usize {
    setup(m, c);
    let preserved = m.cpu.regs;
    let mut ram = m.bus.ewram.to_vec();
    let result = data.evaluate(c.levels, c.received);
    let mut count = 0;
    let mut matched = None;
    while m.cpu.regs[15] != RETURN {
        assert!(
            count < 30000,
            "progression gate {c:?} at{:08x}",
            m.cpu.regs[15]
        );
        let pc = m.cpu.regs[15];
        visited.insert((pc, m.cpu.thumb()));
        if matches!(
            pc,
            0x08048810 | 0x0804882e | 0x08048890 | 0x08048896 | 0x08048898
        ) {
            let offset = m.cpu.regs[7]
                .checked_sub(0x08000000 + SOURCE_OFFSET)
                .unwrap() as usize;
            assert!(offset < 768 && offset % 12 == 0);
            let row = offset / 12;
            let field = match pc {
                0x08048810 => 4,
                0x0804882e => 4 + m.cpu.regs[3] as usize,
                0x08048890 => 3,
                0x08048896 => 1,
                _ => 2,
            };
            assert!((1..12).contains(&field));
            fields.insert((row, field));
            if pc == 0x08048890 {
                matched = Some(row as u8);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert_eq!(matched, result.matched_id, "{c:?}");
    assert_eq!(m.cpu.regs[0], u32::from(result.value), "{c:?}");
    ram[0x25000] = result.output;
    ram[0x2c08..0x2c10].copy_from_slice(&result.received.to_le_bytes());
    assert_eq!(&m.bus.ewram[..], &ram, "fullRAM {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "callee r{i} {c:?}");
    }
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    count
}
fn source(m: &mut Machine, data: &ProgressionGates) {
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
}
#[test]
#[ignore = "private EUR768B original progression gates; one reduced Wine oracle, Linux/USA pending"]
fn european_progression_gates_match_original_live_levels_received_mask_and_zero_return_success() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_GATES_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: ProgressionGates = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PROGRESSION_GATES_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    assert_eq!((catalog.sources.len(), catalog.sources[0].length), (1, 768));
    assert_eq!(
        &data.source_bytes(),
        &rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        &serde_json::from_slice::<ProgressionGates>(&bytes).unwrap(),
        data
    );
    for id in [64, 255, 256, u32::MAX] {
        assert!(data.record(id).is_none());
    }
    assert!(data.records[0].byte(12).is_none());
    let shape = serde_json::to_value(data).unwrap();
    for n in 0..4 {
        let mut v = shape.clone();
        match n {
            0 => v["extra"] = true.into(),
            1 => v["records"][0]["extra"] = true.into(),
            2 => v["records"][0]["requirements04_0b"] = serde_json::to_value([0u8; 7]).unwrap(),
            _ => {
                v["records"][0].as_object_mut().unwrap().remove("byte00");
            }
        }
        assert!(serde_json::from_value::<ProgressionGates>(v).is_err());
    }
    for n in 0..8 {
        let mut v = data.clone();
        match n {
            0 => v.source_offset += 1,
            1 => {
                v.records.pop();
            }
            2 => v.records.swap(0, 63),
            3 => v.records[63].id = 0,
            4 => v.records[0].source_offset += 1,
            5 => v.records[0].byte00 ^= 0xff,
            6 => v.records[63].halfword01 ^= 1,
            _ => v.records[63].requirements04_0b[7] ^= 1,
        }
        if n < 5 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    let zero = data.evaluate([0; 8], !(1u64 << 63));
    assert_eq!(
        (zero.matched_id, zero.value, zero.output, zero.received),
        (Some(63), 0, 0, u64::MAX)
    );
    let miss = data.evaluate([255; 8], u64::MAX);
    assert_eq!(
        (miss.matched_id, miss.value, miss.output, miss.received),
        (None, 0, 0, u64::MAX)
    );
    let mut m = crate::make_machine(rom.clone(), None);
    let real = cases(data);
    let mut visited = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited, &mut fields);
    }
    assert_eq!(fields.len(), 64 * 11);
    assert!(fields.iter().all(|&(_, field)| field != 0));
    let real_fields = fields.len();
    let mut modified = data.clone();
    for (i, r) in modified.records.iter_mut().enumerate() {
        r.byte00 = 255u8.wrapping_sub(i as u8);
        r.halfword01 = if i % 2 == 0 { 0xffff } else { 0x8000 };
        r.byte03 = if i % 2 == 0 { 255 } else { 128 };
        r.requirements04_0b = [128; 8];
    }
    modified.records[63].halfword01 = 0;
    modified.records[63].byte03 = 255;
    modified.validate().unwrap();
    assert!(modified.verify_source(&rom).is_err());
    source(&mut m, &modified);
    assert!(Catalog::from_rom(&m.bus.rom).is_err());
    let mut synthetic = vec![];
    for row in [0, 1, 31, 32, 63] {
        for level in [127, 128, 255] {
            synthetic.push(Case {
                levels: [level; 8],
                received: !(1u64 << row),
            });
        }
    }
    for field in 0..8 {
        let mut levels = [128; 8];
        levels[field] = 127;
        synthetic.push(Case {
            levels,
            received: !(1u64 << 32),
        });
    }
    synthetic.push(Case {
        levels: [255; 8],
        received: 0,
    });
    synthetic.push(Case {
        levels: [255; 8],
        received: u64::MAX,
    });
    for &c in &synthetic {
        instructions += run(&mut m, c, &modified, &mut visited, &mut fields);
    }
    source(&mut m, data);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("progression-gates.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_bytes":768,"records":64,"real_cases":real.len(),"synthetic_cases":synthetic.len(),"instructions_coverage":instructions,"pcs":visited.len(),"original_source_field_bytes_distinct":real_fields,"unused_byte00_original_reads":0,"original_full0487c8_and_u64_shift_helper":true,"all64physical_rows_and8live_unsigned_minima_at_equal_and_below":true,"first_unmarked_eligible_physical_index_bit31_32_63":true,"zero_value_row63_success_marks_bit_distinct_from_fullmask_miss":true,"unused00_not_normalized_high_u16_output_unsigned128_255":true,"strict_json_fixed_array_ids_order_offsets_roundtrip_and_coherent_edit_guards":true,"complete_ewram_only_mask_and_output_changed_sp_callee_stack_guards":true,"receipt_ui_and_foreign_keys":"unclaimed; structural u16 permits original downstream larger-value branch","synthetic":"unauthenticated/unpublished","managed":"e35-d34 unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
