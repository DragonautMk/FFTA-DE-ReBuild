//! One reduced EUR propertyword oracle; earlier animation/metadata proofs stay closed.
use super::*;
use gba_core::{rom_data::unit_obj_property_words::*, Machine};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const ACTOR: usize = 0x10000;
const UNIT: usize = 0x11000;
const CONTEXT: usize = 0x12000;
const TASK: usize = 0x13000;
const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;
fn half(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn put16(bytes: &mut [u8], at: usize, v: u16) {
    bytes[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
fn put32(bytes: &mut [u8], at: usize, v: u32) {
    bytes[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn cpu(m: &mut Machine, pc: u32, args: &[u32]) {
    m.cpu.regs = [0; 16];
    for i in 4..13 {
        m.cpu.regs[i] = 0x31310000 + i as u32;
    }
    m.cpu.regs[..args.len()].copy_from_slice(args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
#[derive(Default)]
struct Coverage {
    reads: BTreeSet<u32>,
    pcs: BTreeSet<u32>,
    instructions: usize,
}
fn until(m: &mut Machine, pc: u32, c: &mut Coverage) {
    let mut steps = 0;
    while m.cpu.regs[15] != pc {
        assert!(steps < 2000, "propertyword prefix {:08x}", m.cpu.regs[15]);
        c.pcs.insert(m.cpu.regs[15]);
        if m.cpu.regs[15] == 0x080221ca {
            let at = m.cpu.regs[0];
            c.reads.extend([at - 0x08000000, at + 1 - 0x08000000]);
        }
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    c.instructions += steps;
}
fn call_getter(m: &mut Machine, id: u32, expected: u16, c: &mut Coverage) {
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    cpu(m, 0x080221c4, &[id]);
    let before = m.cpu.regs;
    until(m, RETURN, c);
    assert_eq!(m.cpu.regs[0], u32::from(expected));
    assert_eq!(m.cpu.regs[4..14], before[4..14]);
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(m.bus.iwram, iw);
}
fn prefix_guards(m: &Machine, ew: &[u8], iw: &[u8]) {
    assert_eq!(m.bus.ewram, ew);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iw.to_vec();
    actual[0x7c00..0x7e00].fill(0);
    expected[0x7c00..0x7e00].fill(0);
    assert_eq!(actual, expected);
    assert!(m.bus.vram.iter().all(|&v| v == 0xa5));
    assert!(m.bus.oam.iter().all(|&v| v == 0));
    assert!(m.bus.palette.iter().all(|&v| v == 0));
}

#[test]
#[ignore = "private EUR actor propertyword source/getter/constructor-prefix oracle; Wine only, Linux/USA pending"]
fn european_actor_property_words_match_original_getter_and_task_arguments() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_PROPERTY_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_PROPERTY_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let producer: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_PROPERTY_PRODUCERS").unwrap())
            .unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let jobs = gba_core::rom_data::metadata52::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let special = gba_core::rom_data::metadata14::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    data.validate_producers(&jobs, &special).unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(
        serde_json::to_value(&data.producers).unwrap(),
        producer["producers"]
    );
    assert_eq!(data.words.len(), 182);
    for (value, expected) in data.words.iter().zip(plan["words"].as_array().unwrap()) {
        assert_eq!(
            u64::from(value.sprite_id),
            expected["sprite_id"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(value.source_offset),
            expected["source_offset"].as_u64().unwrap()
        );
        assert_eq!(u64::from(value.word), expected["word"].as_u64().unwrap());
    }
    let spans = data.source_spans().unwrap();
    let mut combined = Vec::new();
    for (((at, bytes, role), source), expected) in spans
        .iter()
        .zip(&catalog.sources)
        .zip(plan["spans"].as_array().unwrap())
    {
        assert_eq!(u64::from(*at), expected["offset"].as_u64().unwrap());
        assert_eq!(bytes.len() as u64, expected["length"].as_u64().unwrap());
        assert_eq!(role, expected["role"].as_str().unwrap());
        assert_eq!(source.sha256, expected["sha256"].as_str().unwrap());
        assert_eq!(
            (source.offset, source.length as usize, &source.role),
            (*at, bytes.len(), role)
        );
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        combined.extend_from_slice(bytes);
    }
    assert_eq!(spans.len(), 4);
    assert_eq!(combined.len(), 364);
    assert_eq!(gba_core::ffta_rom::digest(&combined), SOURCE_SHA256);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<UnitObjPropertyWords>(&encoded).unwrap(),
        *data
    );
    fs::write(root.join("unit-obj-property-words.json"), &encoded).unwrap();
    let mut guards = 0;
    for variant in 0..17 {
        let mut v = serde_json::to_value(data).unwrap();
        match variant {
            0 => v["unknown"] = true.into(),
            1 => v["producers"][0]["unknown"] = true.into(),
            2 => v["words"][0]["unknown"] = true.into(),
            3 => v["producers"][0]["mode"] = "invalid".into(),
            4 => v["producers"][0]["kind"] = "invalid".into(),
            5 => v["words"][0]["word"] = 65536.into(),
            6 => v["words"][0]["source_offset"] = 1.into(),
            7 => {
                v["producers"].as_array_mut().unwrap().pop();
            }
            8 => v["producers"][1] = v["producers"][0].clone(),
            9 => v["producers"][0]["metadata_source"] = Value::Null,
            10 => v["producers"][0]["logical_id"] = 255.into(),
            11 => {
                v["words"].as_array_mut().unwrap().pop();
            }
            12 => v["words"][1] = v["words"][0].clone(),
            13 => v["producers"][0]["sprite_id"] = 65535.into(),
            14 => v["words"].as_array_mut().unwrap().swap(0, 1),
            15 => v["producers"].as_array_mut().unwrap().swap(0, 1),
            _ => {
                v["producers"][0]["logical_id"] = 0.into();
                v["producers"][0]["metadata_source"] = (0x6e5464u32 + 7).into();
            }
        }
        assert!(serde_json::from_value::<UnitObjPropertyWords>(v)
            .map_or(true, |x| x.validate_producers(&jobs, &special).is_err()));
        guards += 1;
    }
    let mut altered = data.clone();
    altered.words[0].word = 0xfedc;
    altered.validate().unwrap();
    assert!(altered.verify_source(&rom).is_err());
    guards += 1;
    let mut changed_jobs = jobs.clone();
    changed_jobs.records[2].routing = 255;
    assert!(data.validate_producers(&changed_jobs, &special).is_err());
    guards += 1;
    let mut changed_special = special.clone();
    changed_special.records[0].halfwords04_06[0] ^= 1;
    assert!(data.validate_producers(&jobs, &changed_special).is_err());
    guards += 1;

    let mut m = Machine::new(rom.clone());
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0);
    let mut c = Coverage::default();
    for value in &data.words {
        call_getter(&mut m, u32::from(value.sprite_id), value.word, &mut c);
    }
    assert!(spans
        .iter()
        .all(|(at, bytes, _)| (*at..*at + bytes.len() as u32).all(|a| c.reads.contains(&a))));
    // Bit31 is discarded by LSL1; bits16..30 are not truncated to a u16 ID.
    for value in [data.words.first().unwrap(), data.words.last().unwrap()] {
        call_getter(
            &mut m,
            0x80000000 | u32::from(value.sprite_id),
            value.word,
            &mut c,
        );
    }
    let raw = data
        .words
        .iter()
        .find_map(|v| {
            let id = 0x10000 + u32::from(v.sprite_id);
            let at = SOURCE_BASE + 2 * id;
            let expected = half(&rom, at as usize);
            (expected != v.word).then_some((id, expected))
        })
        .unwrap();
    call_getter(&mut m, raw.0, raw.1, &mut c);

    let mut new_prefixes = 0;
    let mut reuse_prefixes = 0;
    let mut setters = 0;
    for original_word in [16, 24] {
        let selected = data
            .words
            .iter()
            .find(|v| {
                let base = word(&rom, 0x394834 + usize::from(v.sprite_id) * 4) - 0x08000000;
                v.word == original_word && word(&rom, base as usize + 6 * 8 + 12) != 0
            })
            .unwrap();
        for (effective_word, reuse) in [
            (original_word, false),
            (original_word | 0x8000, false),
            (original_word, true),
        ] {
            let mut machine = Machine::new(rom.clone());
            machine.bus.rom[selected.source_offset as usize..selected.source_offset as usize + 2]
                .copy_from_slice(&effective_word.to_le_bytes());
            machine.bus.ewram.fill(0x5a);
            machine.bus.iwram.fill(0);
            machine.bus.vram.fill(0xa5);
            put32(&mut machine.bus.ewram, ACTOR, 0x02000000 + UNIT as u32);
            put32(
                &mut machine.bus.ewram,
                ACTOR + 0x40,
                0x02000000 + CONTEXT as u32,
            );
            put32(
                &mut machine.bus.ewram,
                ACTOR + 0x44,
                if reuse { 0x02000000 + TASK as u32 } else { 0 },
            );
            put16(&mut machine.bus.ewram, ACTOR + 0x34, selected.sprite_id);
            machine.bus.ewram[ACTOR + 0x20] = 1;
            machine.bus.ewram[UNIT + 4] = 0;
            machine.bus.ewram[UNIT + 5] = 2;
            machine.bus.ewram[UNIT + 7] = 2;
            put32(&mut machine.bus.ewram, CONTEXT + 8, 0x02014000);
            put16(&mut machine.bus.ewram, TASK + 0x12, 7);
            put16(&mut machine.bus.ewram, TASK + 0x16, 0xa35c);
            let mut ew = machine.bus.ewram.clone();
            let iw = machine.bus.iwram.clone();
            put16(&mut ew, ACTOR + 0x36, 8);
            ew[ACTOR + 0x1f] = 1;
            cpu(
                &mut machine,
                0x0809acf8,
                &[0x02000000 + ACTOR as u32, 8, 1, 0],
            );
            until(&mut machine, 0x0809ad18, &mut c);
            assert_eq!(machine.cpu.regs[8], u32::from(effective_word));
            until(
                &mut machine,
                if reuse { 0x080227c4 } else { 0x080227cc },
                &mut c,
            );
            if reuse {
                assert_eq!(machine.cpu.regs[7], 0xa35c);
                assert_eq!(half(&machine.bus.ewram, TASK + 0x16), 0xa35c);
                reuse_prefixes += 1;
            } else {
                assert_eq!(machine.cpu.regs[7], u32::from(effective_word));
                assert_eq!(machine.cpu.regs[2], u32::from(effective_word));
                assert_eq!(machine.cpu.regs[0], 0x02014000);
                new_prefixes += 1;
            }
            prefix_guards(&machine, &ew, &iw);
            if !reuse {
                // Isolated original setter, not a full allocator/constructor ABI claim.
                cpu(
                    &mut machine,
                    0x08022df4,
                    &[0x02000000 + TASK as u32, 17, u32::from(effective_word)],
                );
                let regs = machine.cpu.regs;
                until(&mut machine, RETURN, &mut c);
                put16(&mut ew, TASK + 0x12, 17);
                put16(&mut ew, TASK + 0x16, effective_word);
                assert_eq!(machine.cpu.regs[4..14], regs[4..14]);
                prefix_guards(&machine, &ew, &iw);
                setters += 1;
            }
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&json!({
        "producer_witnesses":398,"words":182,"source_ranges":4,"source_bytes":364,
        "all_independent_fields_sources_and_hashes":true,"guards":guards,
        "original_getters":182,"raw_fullword_getters":3,
        "original_actor_new_task_prefixes":new_prefixes,"original_existing_task_prefixes":reuse_prefixes,
        "isolated_original_raw_u16_setters":setters,"all_364_original_getter_bytes_read":true,
        "raw_high_bits_preserved":true,"existing_task_retains_own_word":true,
        "entire_RAM_outside_declared_scratch_and_destination":true,
        "instructions_coverage_only":c.instructions,"distinct_PCs":c.pcs.len(),
        "full_allocator_constructor_native_clock_ordinary":"not claimed; partial prefixes",
        "dataset":"e66-d65 unchanged","linux":"pending","usa":"pending",
        "native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
