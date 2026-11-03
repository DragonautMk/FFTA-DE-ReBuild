//! One new EUR sequence decoder/source-reader check; closed tone/sample and e42 acceptance stay closed.
use super::*;
use gba_core::{
    rom_data::{audio_directory, audio_sequences::*},
    Machine,
};
use serde_json::Value;
use std::collections::BTreeSet;
const PLAYER: u32 = 0x02010000;
const TRACK: u32 = 0x02010100;
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007e00;
fn n(v: &Value, k: &str) -> u32 {
    v[k].as_u64().unwrap().try_into().unwrap()
}
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0);
    m.bus.iwram.fill(0);
    m.bus.iwram[0x7c00..0x7e20].fill(0x5a);
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(m: &mut Machine, end: u32, pcs: &mut BTreeSet<u32>) -> usize {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        assert!(n < 1000, "pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
fn file(name: &str) -> Value {
    let p = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SEQUENCE_EVIDENCE").unwrap());
    serde_json::from_slice(&fs::read(p.join(name)).unwrap()).unwrap()
}
#[test]
#[ignore = "private EUR sequence decoder and original operand/driver readers; Wine only, Linux/USA pending"]
fn european_audio_sequence_model_matches_instruction_inventory_and_original_readers() {
    let out = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SEQUENCE_ROOT").unwrap());
    assert!(!out.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent = file("sequence-instructions-independent.json");
    let producers: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SEQUENCE_PRODUCERS").unwrap()).unwrap(),
    )
    .unwrap();
    let cat = Catalog::from_rom(&rom).unwrap();
    let data = &cat.decoded;
    let directory = audio_directory::Catalog::from_rom(&rom).unwrap().decoded;
    data.validate_directory(&directory).unwrap();
    assert_eq!(
        (
            cat.producer.tracks,
            cat.producer.states,
            cat.producer.default_ram_notes,
            cat.producer.max_pattern_depth,
            cat.producer.consumed_ranges,
            cat.producer.consumed_bytes,
            cat.producer.programs,
            cat.producer.note_pairs
        ),
        (1610, 147424, 18, 1, 805, 177047, 390, 2065)
    );
    let roots = producers["tracks"].as_array().unwrap();
    for (t, e) in data.tracks.iter().zip(roots) {
        assert_eq!(
            (
                t.directory_header,
                u32::from(t.track_index),
                0x08000000 + data.instruction(t.entry).unwrap().source_offset
            ),
            (n(e, "header") as u16, n(e, "track"), n(e, "entry"))
        );
    }
    assert_eq!(roots.len(), data.tracks.len());
    let expected = independent["instructions"].as_array().unwrap();
    assert_eq!(expected.len(), INSTRUCTIONS);
    for (i, e) in data.instructions.iter().zip(expected) {
        let bytes = data.instruction_bytes(i.id).unwrap();
        assert_eq!(
            (
                i.source_offset,
                i.source_offset + bytes.len() as u32,
                i.code.command().unwrap(),
                i.explicit
            ),
            (
                n(e, "pc"),
                n(e, "end"),
                n(e, "command") as u8,
                e["explicit"].as_bool().unwrap()
            )
        );
        assert_eq!(
            gba_core::ffta_rom::digest(&bytes),
            e["sha256"].as_str().unwrap()
        );
        assert_eq!(data.byte(i.id, bytes.len()), None);
        assert_eq!(data.byte(i.id, usize::MAX), None);
        // Independent byte-field decode, including physical running-status forms.
        let p = i.source_offset as usize + usize::from(i.explicit);
        match &i.code {
            Code::Jump { target } | Code::Pattern { target } => assert_eq!(
                0x08000000 + data.instruction(*target).unwrap().source_offset,
                u32::from_le_bytes(rom[p..p + 4].try_into().unwrap())
            ),
            Code::Parameter { value, .. } => assert_eq!(*value, rom[p]),
            Code::Extended { selector, value } => assert_eq!([*selector, *value], rom[p..p + 2]),
            Code::EndTie { key } => assert_eq!(*key, rom.get(p).copied().filter(|v| *v < 128)),
            Code::Note {
                key,
                velocity,
                extension,
                ..
            } => {
                let mut values = Vec::new();
                for at in p..p + 3 {
                    if rom[at] >= 128 {
                        break;
                    }
                    values.push(rom[at]);
                }
                assert_eq!(
                    [*key, *velocity, *extension],
                    [
                        values.first().copied(),
                        values.get(1).copied(),
                        values.get(2).copied()
                    ]
                );
            }
            _ => {}
        }
    }
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 808);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), SOURCE_BYTES);
    assert_eq!(cat.sources.len(), 808);
    let hashes = independent["source_hashes"].as_array().unwrap();
    for ((at, bytes, role), e) in spans
        .iter()
        .filter(|s| s.2.starts_with("audio/sequence/"))
        .zip(hashes)
    {
        assert_eq!(
            (*at, bytes.len() as u32, gba_core::ffta_rom::digest(bytes)),
            (
                n(e, "offset"),
                n(e, "length"),
                e["sha256"].as_str().unwrap().into()
            )
        );
        assert!(role.starts_with("audio/sequence/"));
    }
    assert_eq!(hashes.len(), 805);
    assert_eq!(
        data.driver.clocks,
        rom[CLOCK_OFFSET as usize..CLOCK_OFFSET as usize + 49]
    );
    for (at, words) in [
        (COMMANDS_OFFSET, &data.driver.commands),
        (EXTENDED_OFFSET, &data.driver.extended8_9),
    ] {
        for (i, &v) in words.iter().enumerate() {
            assert_eq!(
                v,
                u32::from_le_bytes(
                    rom[at as usize + 4 * i..at as usize + 4 * i + 4]
                        .try_into()
                        .unwrap()
                )
            );
        }
    }
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<AudioSequences>(&bytes).unwrap(),
        *data
    );
    let mut unknown = serde_json::to_value(data).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unknown".into(), true.into());
    assert!(serde_json::from_value::<AudioSequences>(unknown).is_err());
    let mut unknown = serde_json::to_value(&data.instructions[0].code).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unknown".into(), true.into());
    assert!(serde_json::from_value::<Code>(unknown).is_err());
    let note = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Note { .. }))
        .unwrap();
    let implicit = data
        .instructions
        .iter()
        .position(|i| !i.explicit && matches!(i.code, Code::Note { .. }))
        .unwrap();
    let jump = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Jump { .. }))
        .unwrap();
    let wait = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Wait { .. }))
        .unwrap();
    let parameter = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Parameter { .. }))
        .unwrap();
    let ext = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::Extended { .. }))
        .unwrap();
    let tie = data
        .instructions
        .iter()
        .position(|i| matches!(i.code, Code::EndTie { .. }))
        .unwrap();
    for case in 0..25 {
        let mut bad = data.clone();
        match case {
            0 => {
                bad.instructions.pop();
            }
            1 => bad.instructions[1].id = 0,
            2 => bad.instructions[1].source_offset = bad.instructions[0].source_offset,
            3 => bad.instructions[jump].code = Code::Jump { target: u32::MAX },
            4 => {
                bad.instructions[note].code = Code::Note {
                    clock: 0,
                    key: Some(128),
                    velocity: None,
                    extension: None,
                }
            }
            5 => {
                bad.instructions[note].code = Code::Note {
                    clock: 0,
                    key: None,
                    velocity: Some(2),
                    extension: None,
                }
            }
            6 => {
                bad.instructions[note].code = Code::Note {
                    clock: 0,
                    key: Some(1),
                    velocity: None,
                    extension: Some(2),
                }
            }
            7 => {
                bad.instructions[implicit].code = Code::Note {
                    clock: 0,
                    key: None,
                    velocity: None,
                    extension: None,
                }
            }
            8 => bad.instructions[wait].code = Code::Wait { clock: 49 },
            9 => {
                bad.instructions[parameter].code = Code::Parameter {
                    command: 0xca,
                    value: 1,
                }
            }
            10 => {
                bad.instructions[parameter].explicit = false;
                bad.instructions[parameter].code = Code::Parameter {
                    command: 0xbb,
                    value: 1,
                };
            }
            11 => {
                bad.instructions[ext].code = Code::Extended {
                    selector: 7,
                    value: 0,
                }
            }
            12 => bad.instructions[tie].code = Code::EndTie { key: Some(128) },
            13 => {
                bad.driver.clocks.pop();
            }
            14 => {
                bad.driver.commands.pop();
            }
            15 => {
                bad.driver.extended8_9.pop();
            }
            16 => bad.driver.command_source_offset += 4,
            17 => bad.driver.clock_source_offset += 1,
            18 => bad.driver.extended_source_offset -= 4,
            19 => bad.driver.commands[0] = 0x02000001,
            20 => bad.driver.commands[0] &= !1,
            21 => bad.tracks[0].entry = u32::MAX,
            22 => bad.tracks[0].directory_header = 640,
            23 => bad.tracks[0].track_index = 16,
            _ => {
                bad.instructions[parameter].explicit = false;
                bad.instructions[parameter].code = Code::Parameter {
                    command: 0xbd,
                    value: 128,
                };
            }
        }
        assert!(bad.validate().is_err(), "guard {case}");
    }
    let mut edited = data.clone();
    if let Code::Parameter { value, .. } = &mut edited.instructions[parameter].code {
        *value ^= 1;
    }
    edited.validate_directory(&directory).unwrap();
    assert!(edited.verify_source(&rom).is_err());
    let mut metadata = data.clone();
    if let Code::Note { clock, .. } = &mut metadata.instructions[implicit].code {
        *clock = (*clock + 1) % 49;
    }
    metadata.validate().unwrap();
    assert_eq!(metadata.source_spans().unwrap(), spans);
    assert!(
        metadata.verify_source(&rom).is_err(),
        "inherited command metadata cannot bypass source identity"
    );
    let mut owner = directory.clone();
    owner
        .headers
        .iter_mut()
        .find(|h| h.id == data.tracks[0].directory_header)
        .unwrap()
        .track_addresses[0] += 1;
    assert!(data.validate_directory(&owner).is_err());
    assert!(Catalog::from_rom(&[]).is_err());
    // New original-source probes stop before tone selection/mixing; no closed tone oracle is repeated.
    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut instructions = 0;
    let mut prefix = BTreeSet::new();
    let mut notes = BTreeSet::new();
    let mut branches = BTreeSet::new();
    let mut params = BTreeSet::new();
    let mut counters = [0usize; 7];
    let mut ties = BTreeSet::new();
    for i in &data.instructions {
        let command = i.code.command().unwrap();
        let operand = 0x08000000 + i.source_offset + u32::from(i.explicit);
        if prefix.insert((command, i.explicit)) {
            reset(&mut m, 0x081485ac);
            m.cpu.regs[5] = TRACK;
            m.bus.ewram[0x10107] = if i.explicit { 0xe0 } else { command };
            put(&mut m.bus.ewram, 0x10140, 0x08000000 + i.source_offset);
            let mut expected = m.bus.ewram.clone();
            put(&mut expected, 0x10140, operand);
            if i.explicit && command >= 0xbd {
                expected[0x10107] = command;
            }
            instructions += until(&mut m, 0x081485c2, &mut pcs);
            assert_eq!(m.cpu.regs[1], u32::from(command));
            assert_eq!(m.bus.ewram, expected);
            counters[0] += 1;
        }
        if let Code::Note {
            clock,
            key,
            velocity,
            extension,
        } = &i.code
        {
            if notes.insert((*clock, i.code.operand_len())) {
                reset(&mut m, 0x081487cc);
                m.cpu.regs[0] = u32::from(*clock);
                m.cpu.regs[1] = PLAYER;
                m.cpu.regs[2] = TRACK;
                put(&mut m.bus.ewram, 0x10140, operand);
                m.bus.ewram[0x10105] = 44;
                m.bus.ewram[0x10106] = 55;
                let mut expected = m.bus.ewram.clone();
                expected[0x10104] =
                    data.driver.clocks[*clock as usize].wrapping_add(extension.unwrap_or(0));
                if let Some(v) = key {
                    expected[0x10105] = *v;
                }
                if let Some(v) = velocity {
                    expected[0x10106] = *v;
                }
                put(
                    &mut expected,
                    0x10140,
                    operand + i.code.operand_len() as u32,
                );
                let before_iwram = m.bus.iwram.clone();
                instructions += until(&mut m, 0x08148812, &mut pcs);
                assert_eq!(m.bus.ewram, expected, "note {}", i.id);
                assert_eq!(m.cpu.regs[13], STACK - 60);
                assert_eq!(&m.bus.iwram[..0x7dc4], &before_iwram[..0x7dc4]);
                assert_eq!(&m.bus.iwram[0x7e00..], &before_iwram[0x7e00..]);
                counters[1] += 1;
            }
        }
        if let Code::Jump { target } | Code::Pattern { target } = &i.code {
            let pattern = matches!(i.code, Code::Pattern { .. });
            if branches.insert((pattern, operand & 3)) {
                for depth in 0..if pattern { 3 } else { 1 } {
                    reset(&mut m, if pattern { 0x08148350 } else { 0x08148330 });
                    m.cpu.regs[1] = TRACK;
                    m.bus.ewram[0x10102] = depth;
                    put(&mut m.bus.ewram, 0x10140, operand);
                    let mut expected = m.bus.ewram.clone();
                    put(
                        &mut expected,
                        0x10140,
                        0x08000000 + data.instruction(*target).unwrap().source_offset,
                    );
                    if pattern {
                        expected[0x10102] = depth + 1;
                        put(&mut expected, 0x10144 + usize::from(depth) * 4, operand + 4);
                    }
                    instructions += until(&mut m, RETURN, &mut pcs);
                    assert_eq!(m.bus.ewram, expected);
                    assert_eq!(m.cpu.regs[13], STACK);
                    counters[2] += 1;
                }
            }
        }
        if let Code::Parameter { value, .. } = &i.code {
            if params.insert(command) {
                reset(&mut m, 0x08148324);
                m.cpu.regs[1] = TRACK;
                put(&mut m.bus.ewram, 0x10140, operand);
                let mut expected = m.bus.ewram.clone();
                put(&mut expected, 0x10140, operand + 1);
                instructions += until(&mut m, RETURN, &mut pcs);
                assert_eq!(m.cpu.regs[3], u32::from(*value));
                assert_eq!(m.bus.ewram, expected);
                counters[3] += 1;
            }
        }
        if let Code::EndTie { key } = &i.code {
            if ties.insert((i.explicit, key.is_some())) {
                reset(&mut m, 0x081489cc);
                m.cpu.regs[1] = TRACK;
                m.bus.ewram[0x10105] = 44;
                put(&mut m.bus.ewram, 0x10140, operand);
                let mut expected = m.bus.ewram.clone();
                if let Some(key) = key {
                    expected[0x10105] = *key;
                }
                put(
                    &mut expected,
                    0x10140,
                    operand + i.code.operand_len() as u32,
                );
                instructions += until(&mut m, RETURN, &mut pcs);
                assert_eq!(m.bus.ewram, expected);
                assert_eq!(m.cpu.regs[13], STACK);
                counters[6] += 1;
            }
        }
        if let Code::Extended { selector, value } = &i.code {
            reset(&mut m, 0x08149e88);
            m.cpu.regs[1] = TRACK;
            put(&mut m.bus.ewram, 0x10140, operand);
            let mut expected = m.bus.ewram.clone();
            put(&mut expected, 0x10140, operand + 2);
            expected[0x1011e + usize::from(selector - 8)] = *value;
            instructions += until(&mut m, RETURN, &mut pcs);
            assert_eq!(m.bus.ewram, expected);
            assert_eq!(m.cpu.regs[13], STACK);
            counters[4] += 1;
        }
    }
    for clock in 0..49 {
        reset(&mut m, 0x081485fa);
        m.cpu.regs[1] = 0x80 + clock;
        m.cpu.regs[5] = TRACK;
        let mut expected = m.bus.ewram.clone();
        expected[0x10101] = data.driver.clocks[clock as usize];
        instructions += until(&mut m, 0x08148604, &mut pcs);
        assert_eq!(m.bus.ewram, expected);
        counters[5] += 1;
    }
    reset(&mut m, 0x081482f0);
    m.cpu.regs[0] = 0x02018000;
    let mut expected = m.bus.ewram.clone();
    for (i, w) in data.driver.commands.iter().enumerate() {
        put(&mut expected, 0x18000 + i * 4, *w);
    }
    instructions += until(&mut m, RETURN, &mut pcs);
    assert_eq!(m.bus.ewram, expected);
    assert_eq!(m.cpu.regs[13], STACK);
    assert!(m.cpu.regs[4..12].iter().all(|v| *v == 0));
    assert!(
        counters[0] > 100
            && counters[1] > 49
            && counters[2] > 4
            && counters[3] == 12
            && counters[4] == 4
            && counters[5] == 49
            && counters[6] == 3,
        "{counters:?}"
    );
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("audio-sequences.json"), bytes).unwrap();
    fs::write(out.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"tracks":1610,"canonical_instructions":INSTRUCTIONS,"bytecode_ranges":805,"bytecode_bytes":BYTECODE_BYTES,"source_ranges":808,"source_bytes":SOURCE_BYTES,"driver_clocks":49,"driver_command_words":36,"extended_selectors":[8,9],"all_independent_instruction_fields_hashes_root_links_and_source_roundtrip":true,"strict_json_and_structural_guards":27,"coherent_encoded_and_samebytes_inherited_metadata_edits_rejected":true,"actual_directory_dependency":true,"original_dispatch_prefix_note_operands_jump_pattern_parameter_extended_wait_endtie_cases":counters,"original36word_command_copy":true,"instructions_coverage":instructions,"pcs":pcs.len(),"source_probes":"explicit dispatch/note/wait checkpoints and branch/operand/CD/copy returns; not full sequencer/mixer/state-clock/native/ordinary acceptance","managed":"e42-d41 unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
