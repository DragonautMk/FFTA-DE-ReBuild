//! One reduced Wine oracle for the newly authenticated UI/property caller paths.
use super::*;
use gba_core::{rom_data::unit_obj_byte_property_words::*, Machine};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const CONTEXT: usize = 0x10000;
const TASK: usize = 0x20000;
const ACTOR: usize = 0x21000;
const WRAPPER: usize = 0x22000;
const STACK: u32 = 0x03007e00;
fn put16(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn half(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn address(offset: usize) -> u32 {
    0x02000000 + offset as u32
}
fn cpu(m: &mut Machine, pc: u32) {
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
#[derive(Default)]
struct Coverage {
    reads: BTreeSet<u32>,
    instructions: usize,
}
fn until(m: &mut Machine, endpoint: u32, coverage: &mut Coverage) {
    let mut steps = 0;
    while m.cpu.regs[15] != endpoint {
        assert!(steps < 200, "UI property prefix {:08x}", m.cpu.regs[15]);
        if m.cpu.regs[15] == 0x080221ca {
            let at = m.cpu.regs[0] - 0x08000000;
            coverage.reads.extend([at, at + 1]);
        }
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    coverage.instructions += steps;
}
fn guards(m: &Machine, expected_ewram: &[u8], iwram: &[u8], ui_stack_writes: bool) {
    assert_eq!(m.bus.ewram, expected_ewram);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iwram.to_vec();
    if ui_stack_writes {
        // Prefix entry is after the real prologue's stack reservation.
        actual[0x7e00..0x7e1c].fill(0);
        expected[0x7e00..0x7e1c].fill(0);
    }
    assert_eq!(actual, expected);
    assert_eq!(m.cpu.regs[13], STACK);
    assert!(m.bus.vram.iter().all(|&v| v == 0xa5));
    assert!(m.bus.oam.iter().all(|&v| v == 0));
    assert!(m.bus.palette.iter().all(|&v| v == 0));
}
fn ui_prefix(m: &mut Machine, input: u8, property: u16, first: bool, c: &mut Coverage) {
    m.bus.ewram[CONTEXT + 0x1084] = if first { 2 } else { 1 };
    m.bus.ewram[CONTEXT + 0x1085] = input;
    m.bus.ewram[CONTEXT + 0x1086] = 1;
    cpu(m, if first { 0x0802ad4e } else { 0x0802add0 });
    m.cpu.regs[5] = address(CONTEXT);
    m.cpu.regs[6] = address(CONTEXT + 0x1084);
    m.cpu.regs[7] = 63;
    m.cpu.regs[8] = 47;
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    until(m, 0x0802ae20, c);
    assert_eq!(
        m.cpu.regs[0],
        if property == 16 {
            0x083950a4
        } else {
            0x083950b2
        }
    );
    assert_eq!(m.cpu.regs[5], address(CONTEXT));
    assert_eq!(m.cpu.regs[7], 63);
    assert_eq!(m.cpu.regs[8], 47);
    assert_eq!(half(&m.bus.iwram, 0x7e08), 1);
    guards(m, &ew, &iw, true);
}

#[test]
#[ignore = "private EUR UI byte property model/caller prefixes; Wine only, Linux/USA pending"]
fn european_ui_byte_property_words_match_original_callers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_BYTE_PROPERTY_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_OBJ_BYTE_PROPERTY_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let properties = gba_core::rom_data::unit_obj_property_words::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let palettes = gba_core::rom_data::color_mode_palettes::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    data.validate_shared(&properties, &palettes).unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(data.words.len(), 72);
    for (word, expected) in data.words.iter().zip(plan["new_words"].as_array().unwrap()) {
        assert_eq!(
            u64::from(word.input_byte),
            expected["input_byte"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(word.source_offset),
            expected["source_offset"].as_u64().unwrap()
        );
        assert_eq!(u64::from(word.word), expected["word"].as_u64().unwrap());
    }
    let mut bytes = Vec::new();
    for (((at, raw, role), source), expected) in data
        .source_spans()
        .unwrap()
        .iter()
        .zip(&catalog.sources)
        .zip(plan["spans"].as_array().unwrap())
    {
        assert_eq!(u64::from(*at), expected["offset"].as_u64().unwrap());
        assert_eq!(raw.len() as u64, expected["length"].as_u64().unwrap());
        assert_eq!(role, expected["role"].as_str().unwrap());
        assert_eq!(source.sha256, expected["sha256"].as_str().unwrap());
        assert_eq!(
            (source.offset, source.length as usize, &source.role),
            (*at, raw.len(), role)
        );
        assert_eq!(raw, &rom[*at as usize..*at as usize + raw.len()]);
        bytes.extend_from_slice(raw);
    }
    assert_eq!(catalog.sources.len(), 4);
    assert_eq!(bytes.len(), 144);
    assert_eq!(gba_core::ffta_rom::digest(&bytes), SOURCE_SHA256);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<UnitObjBytePropertyWords>(&encoded).unwrap(),
        *data
    );
    fs::write(root.join("unit-obj-byte-property-words.json"), &encoded).unwrap();
    let mut guard_count = 0;
    for variant in 0..11 {
        let mut v = serde_json::to_value(data).unwrap();
        match variant {
            0 => v["unknown"] = true.into(),
            1 => v["words"][0]["unknown"] = true.into(),
            2 => v["words"][0]["word"] = 65536.into(),
            3 => v["words"][0]["input_byte"] = 256.into(),
            4 => v["words"][0]["source_offset"] = 1.into(),
            5 => {
                v["words"].as_array_mut().unwrap().pop();
            }
            6 => v["words"][1] = v["words"][0].clone(),
            7 => v["words"].as_array_mut().unwrap().swap(0, 1),
            8 => v["words"][0]["input_byte"] = 0.into(),
            9 => v["words"][0]["input_byte"] = 248.into(),
            _ => {
                v["words"][0].as_object_mut().unwrap().remove("word");
            }
        }
        assert!(serde_json::from_value::<UnitObjBytePropertyWords>(v)
            .map_or(true, |x| x.validate_shared(&properties, &palettes).is_err()));
        guard_count += 1;
    }
    let mut changed = data.clone();
    changed.words[0].word = 0xfedc;
    changed.validate_shared(&properties, &palettes).unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guard_count += 1;
    let mut invalid_shared = properties.clone();
    invalid_shared.words.pop();
    assert!(data.validate_shared(&invalid_shared, &palettes).is_err());
    guard_count += 1;
    let mut invalid_palette = palettes.clone();
    invalid_palette.spans.pop();
    assert!(data.validate_shared(&properties, &invalid_palette).is_err());
    guard_count += 1;
    // Shared values come from their actual owner, with no duplicate stored copy.
    let mut shared_values = properties.clone();
    shared_values.words[0].word ^= 1;
    data.validate_shared(&shared_values, &palettes).unwrap();
    assert_eq!(
        data.word_for_input(0, &shared_values, &palettes),
        Some(properties.words[0].word ^ 1)
    );
    let mut palette_values = palettes.clone();
    palette_values.slice_offsets[0] ^= 1;
    data.validate_shared(&properties, &palette_values).unwrap();
    assert_eq!(
        data.word_for_input(248, &properties, &palette_values),
        data.word_for_input(248, &properties, &palettes)
            .map(|v| v ^ 1)
    );

    let mut m = Machine::new(rom.clone());
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0);
    m.bus.vram.fill(0xa5);
    let mut c = Coverage::default();
    for query in plan["queries"].as_array().unwrap() {
        let input = query["input_byte"].as_u64().unwrap() as u8;
        let expected = query["word"].as_u64().unwrap() as u16;
        assert_eq!(
            data.word_for_input(input, &properties, &palettes),
            Some(expected)
        );
        for first in [true, false] {
            ui_prefix(&mut m, input, expected, first, &mut c);
        }
    }
    assert_eq!(c.reads, (SOURCE_BASE..SOURCE_BASE + 512).collect());
    // Skip branches stop before unrelated calls, so no fake complete-UI claim.
    for (first, tag, animation) in [
        (true, 0, 1),
        (true, 3, 1),
        (false, 1, 0),
        (false, 1, 12),
        (false, 1, 16),
        (false, 0, 1),
        (false, 2, 1),
        (false, 255, 1),
    ] {
        m.bus.ewram[CONTEXT + 0x1084] = tag;
        m.bus.ewram[CONTEXT + 0x1086] = animation;
        cpu(&mut m, if first { 0x0802ad4e } else { 0x0802add0 });
        m.cpu.regs[5] = address(CONTEXT);
        m.cpu.regs[6] = address(CONTEXT + 0x1084);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let mut skip = Coverage::default();
        until(
            &mut m,
            if first { 0x0802adac } else { 0x0802ae58 },
            &mut skip,
        );
        assert!(skip.reads.is_empty());
        guards(&m, &ew, &iw, false);
    }
    // Fresh synthetic Machine reader values exercise the full u16 comparison.
    for value in [16u16, 0x8010] {
        let mut synthetic = rom.clone();
        put16(&mut synthetic, SOURCE_BASE as usize + 2 * 112, value);
        let mut high = Machine::new(synthetic);
        high.bus.ewram.fill(0x5a);
        high.bus.iwram.fill(0);
        high.bus.vram.fill(0xa5);
        for first in [true, false] {
            ui_prefix(&mut high, 112, value, first, &mut c);
        }
    }
    // The builder writer is a declared prefix after unrelated construction calls.
    for raw in [0x12345600u32, 0xffff, 0x80000070, 0x1f9] {
        cpu(&mut m, 0x0802ab3a);
        m.cpu.regs[4] = address(CONTEXT);
        m.cpu.regs[8] = raw;
        m.cpu.regs[9] = 3;
        m.cpu.regs[10] = 1;
        put32(&mut m.bus.iwram, 0x7e20, 5);
        put32(&mut m.bus.iwram, 0x7e24, 2);
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        ew[CONTEXT + 0x1084..CONTEXT + 0x108b].copy_from_slice(&[2, raw as u8, 1, 0, 3, 0, 5]);
        until(&mut m, 0x0802ab72, &mut c);
        guards(&m, &ew, &iw, false);
    }
    for input in [0u8, 112, 255] {
        let expected = data.word_for_input(input, &properties, &palettes).unwrap();
        for first in [true, false] {
            put32(&mut m.bus.ewram, WRAPPER, address(ACTOR));
            put32(&mut m.bus.ewram, ACTOR + 0x44, address(TASK));
            put16(&mut m.bus.ewram, ACTOR + 0x34, u16::from(input));
            put16(&mut m.bus.ewram, CONTEXT + 0x344c, u16::from(input));
            put16(&mut m.bus.ewram, TASK + 0x12, 17);
            put16(&mut m.bus.ewram, TASK + 0x16, 0xa35c);
            cpu(&mut m, if first { 0x080a97a0 } else { 0x080aa4ae });
            m.cpu.regs[4] = address(CONTEXT + 0x344c);
            m.cpu.regs[7] = address(WRAPPER);
            let mut ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            put16(&mut ew, TASK + 0x16, expected);
            until(&mut m, if first { 0x080a97c4 } else { 0x080aa4d4 }, &mut c);
            assert_eq!(half(&m.bus.ewram, TASK + 0x12), 17);
            assert_eq!(half(&m.bus.ewram, TASK + 0x16), expected);
            guards(&m, &ew, &iw, false);
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&json!({
        "new_words":72,"source_ranges":4,"source_bytes":144,"shared_words":184,"byte_queries":256,
        "guards":guard_count,"UI_branch_prefixes":512,"skip_prefixes":8,"synthetic_full_u16_comparisons":4,
        "builder_low_byte_prefixes":4,"additional_existing_task_setter_prefixes":6,
        "all512_original_reader_bytes":true,"all_RAM_outside_declared_scratch_destinations":true,
        "shared_owner_values_resolved_without_duplicate_copy":true,"instructions_coverage_only":c.instructions,
        "scope":"partial caller prefixes, no full UI/allocator/native-clock/ordinary claim",
        "dataset":"e67-d66 unchanged","linux":"pending","usa":"pending","native_windows":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
