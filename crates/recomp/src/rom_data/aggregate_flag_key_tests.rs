//! Both original counters, including context isolation and live extra predicates.
use super::*;
use gba_core::{
    rom_data::aggregate_flag_keys::{AggregateFlagKeys, Catalog, END_OFFSET, SOURCE_OFFSET},
    Bus, Machine,
};
use std::collections::BTreeSet;
pub(super) const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007e00;
const CONTEXT: u32 = 0x02008000;
#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub context: bool,
    pub pattern: u8,
    pub special: bool,
    pub linked: u8,
}
fn get_flag(ram: &[u8], base: usize, id: u16) -> bool {
    ram[base + 0x1f70 + usize::from(id >> 3)] & (1 << (id & 7)) != 0
}
fn put_flag(ram: &mut [u8], base: usize, id: u16, value: bool) {
    let p = &mut ram[base + 0x1f70 + usize::from(id >> 3)];
    let bit = 1 << (id & 7);
    *p = (*p & !bit) | if value { bit } else { 0 };
}
fn pattern(i: usize, p: u8) -> bool {
    match p {
        0 => false,
        1 => true,
        2 => i == 0,
        3 => i == 299,
        4 => i % 2 == 0,
        5 => i < 17,
        _ => false,
    }
}
pub(super) fn setup(m: &mut Machine, c: Case, data: &AggregateFlagKeys) {
    m.bus.ewram.fill(0x3c);
    m.bus.ewram[0x1f70..0x3f70].fill(0);
    m.bus.ewram[0x9f70..0xbf70].fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    let selected = if c.context { 0x8000 } else { 0 };
    for base in [0, 0x8000] {
        for e in &data.entries {
            let value = if base == selected {
                pattern(e.id as usize, c.pattern)
            } else {
                c.pattern == 0
            };
            // Repeated key slots describe one bit, but original count includes every occurrence.
            let old = get_flag(&m.bus.ewram, base, e.flag_key());
            put_flag(&mut m.bus.ewram, base, e.flag_key(), old || value);
        }
        put_flag(
            &mut m.bus.ewram,
            base,
            0x36,
            if base == selected {
                c.special
            } else {
                !c.special
            },
        );
        // A flag not present in the actual source must not affect the total.
        put_flag(&mut m.bus.ewram, base, 0x2fe, true);
        let linked = if base == selected { c.linked } else { 0 };
        let node = if base == 0 { 0x02030000 } else { 0x02031000 };
        m.bus.write32(
            0x02000000 + base as u32 + 0x28c8,
            if linked == 0 { 0 } else { node },
        );
        if linked != 0 {
            m.bus.write32(node, node + 0x100);
            m.bus.write16(
                node + 0x100,
                if linked == 2 || linked == 4 {
                    0xfd7f
                } else {
                    0xcaaa
                },
            );
            m.bus.write32(
                node + 8,
                if linked == 3 {
                    node + 0x20
                } else if linked == 4 {
                    0xdeadbeef
                } else {
                    0
                },
            );
            if linked == 3 {
                m.bus.write32(node + 0x20, node + 0x120);
                m.bus.write16(node + 0x120, 0x017f);
                m.bus.write32(node + 0x28, 0);
            }
        }
    }
    for i in 0..13 {
        m.cpu.regs[i] = 0x11220000 + i as u32;
    }
    m.cpu.regs[0] = if c.context { CONTEXT } else { 0xdeadbeef };
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = if c.context { 0x0813fb1c } else { 0x080d22fc };
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
pub(super) fn run(
    m: &mut Machine,
    c: Case,
    data: &AggregateFlagKeys,
    visited: &mut BTreeSet<(u32, bool)>,
    fields: &mut BTreeSet<usize>,
) -> usize {
    setup(m, c, data);
    let ram = m.bus.ewram.to_vec();
    let preserved = m.cpu.regs;
    let base = if c.context { 0x8000 } else { 0 };
    let expected = data.count(|id| get_flag(&ram, base, id), c.linked >= 2);
    let mut count = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(count < 30000, "counter {c:?} at{:08x}", m.cpu.regs[15]);
        let pc = m.cpu.regs[15];
        visited.insert((pc, m.cpu.thumb()));
        let source = match pc {
            0x080d2302 | 0x0813fb24 => Some(m.cpu.regs[2]),
            0x080d2324 | 0x0813fb48 => Some(m.cpu.regs[4]),
            _ => None,
        };
        if let Some(at) = source {
            let offset = at.checked_sub(0x08000000 + SOURCE_OFFSET).unwrap() as usize;
            assert!(offset < 602 && offset % 2 == 0);
            fields.insert(offset);
            fields.insert(offset + 1);
        }
        if matches!(pc, 0x080d2316 | 0x0813fb3a) {
            let offset = m.cpu.regs[4] - (0x08000000 + SOURCE_OFFSET);
            assert_eq!(
                m.cpu.regs[0],
                u32::from(data.entries[(offset / 2) as usize].flag_key())
            );
            if c.context {
                assert_eq!(m.cpu.regs[1], CONTEXT);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        count += 1;
    }
    assert_eq!(m.cpu.regs[0], u32::from(expected), "return {c:?}");
    assert_eq!(&m.bus.ewram[..], &ram, "read-only fullRAM {c:?}");
    assert_eq!(m.cpu.regs[13], STACK);
    for i in 4..12 {
        assert_eq!(m.cpu.regs[i], preserved[i], "callee r{i} {c:?}");
    }
    assert!(m.bus.iwram[0x7c00..0x7d80].iter().all(|&v| v == 0x5a));
    assert!(m.bus.iwram[0x7e00..0x7e20].iter().all(|&v| v == 0x5a));
    count
}
fn set_source(m: &mut Machine, data: &AggregateFlagKeys) {
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
}
#[test]
#[ignore = "private EUR602B full global/context counters; Wine only, Linux/USA pending"]
fn european_aggregate_flag_keys_match_original_global_context_wrap_duplicates_and_terminators() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AGGREGATE_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: AggregateFlagKeys = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_AGGREGATE_RECORDS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(data, &independent);
    data.verify_source(&rom).unwrap();
    assert_eq!((catalog.sources.len(), catalog.sources[0].length), (1, 602));
    assert_eq!(
        &data.source_bytes(),
        &rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    for at in 0..602 {
        assert_eq!(data.byte(at), Some(rom[SOURCE_OFFSET as usize + at]));
    }
    assert!(data.byte(602).is_none());
    assert!(data.byte(usize::MAX).is_none());
    for id in [300, 65535, u32::MAX] {
        assert!(data.entry(id).is_none());
    }
    assert_eq!(data.entry(299).unwrap(), &data.entries[299]);
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<AggregateFlagKeys>(&bytes).unwrap(),
        *data
    );
    let shape = serde_json::to_value(data).unwrap();
    for n in 0..4 {
        let mut v = shape.clone();
        match n {
            0 => v["extra"] = true.into(),
            1 => v["entries"][0]["extra"] = true.into(),
            2 => v["terminator"]["extra"] = true.into(),
            _ => {
                v["entries"][0].as_object_mut().unwrap().remove("key");
            }
        }
        assert!(serde_json::from_value::<AggregateFlagKeys>(v).is_err());
    }
    for n in 0..9 {
        let mut v = data.clone();
        match n {
            0 => v.source_offset += 2,
            1 => {
                v.entries.pop();
            }
            2 => v.entries.swap(0, 299),
            3 => v.entries[0].source_offset += 1,
            4 => v.entries[299].id = 0,
            5 => v.terminator.source_offset += 2,
            6 => v.terminator.value = 0,
            7 => v.entries[299].key = 0xffff,
            _ => v.entries[0].key = 0xfffe,
        }
        if n < 7 {
            assert!(v.validate().is_err());
        } else {
            v.validate().unwrap();
            assert!(v.verify_source(&rom).is_err());
        }
    }
    let mut real = vec![];
    for context in [false, true] {
        for pattern in 0..6 {
            real.push(Case {
                context,
                pattern,
                special: false,
                linked: 0,
            });
        }
        for special in [false, true] {
            for linked in 0..5 {
                real.push(Case {
                    context,
                    pattern: 5,
                    special,
                    linked,
                });
            }
        }
    }
    let mut m = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut instructions = 0;
    for &c in &real {
        instructions += run(&mut m, c, data, &mut visited, &mut fields);
    }
    assert_eq!(fields.len(), 602);
    let mut synthetic_count = 0;
    // Duplicate occurrences, high keys and wrapping are distinct from a set of flag IDs.
    let mut modified = data.clone();
    for (i, key) in [
        (0, 0xfffe),
        (1, 0xfd01),
        (2, 0x7fff),
        (3, 0xff00),
        (299, 0xfffe),
    ] {
        modified.entries[i].key = key;
    }
    modified.validate().unwrap();
    assert!(modified.verify_source(&rom).is_err());
    set_source(&mut m, &modified);
    assert!(Catalog::from_rom(&m.bus.rom).is_err());
    for context in [false, true] {
        for pattern in [0, 1, 2, 3, 4] {
            instructions += run(
                &mut m,
                Case {
                    context,
                    pattern,
                    special: false,
                    linked: 0,
                },
                &modified,
                &mut visited,
                &mut fields,
            );
            synthetic_count += 1;
        }
    }
    for index in [0, 1, 149, 299] {
        let mut early = data.clone();
        early.entries[index].key = 0xffff;
        early.validate().unwrap();
        set_source(&mut m, &early);
        for context in [false, true] {
            for special in [false, true] {
                instructions += run(
                    &mut m,
                    Case {
                        context,
                        pattern: 1,
                        special,
                        linked: 0,
                    },
                    &early,
                    &mut visited,
                    &mut fields,
                );
                synthetic_count += 1;
            }
        }
    }
    set_source(&mut m, data);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("aggregate-flag-keys.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":602,"entries":300,"terminator_bytes":2,"real_cases":real.len(),
        "synthetic_cases":synthetic_count,"instructions_coverage":instructions,"pcs":visited.len(),
        "all602source_bytes_original_reads":true,"both_full_global_d22fc_and_context_13fb1c":true,
        "original_flag_predicates_context_isolation_and_key_plus2ff_u16":true,
        "live36_or_masked017f_linked_first_second_short_circuit_extra_one":true,
        "duplicates_count_occurrences_and_early_slot_ffff_stops":true,
        "strict_json_ids_order_offsets_terminator_roundtrip_source_guards":true,
        "full_ewram_read_only_sp_callee_stack_guards":true,"synthetic":"unauthenticated/unpublished",
        "managed":"EUR e37-d36 unchanged","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
