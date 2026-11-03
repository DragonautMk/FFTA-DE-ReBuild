//! One combined EUR archive source oracle. No replacement script interpreter.
use super::*;
use gba_core::{rom_data::script_archive::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;

fn reset(m: &mut Machine, pc: u32) {
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(m: &mut Machine, end: u32, pcs: &mut BTreeSet<u32>, reads: &mut BTreeSet<u32>) -> usize {
    let mut steps = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(steps < 32, "pc{pc:08x} end{end:08x}");
        pcs.insert(pc);
        let access = match pc {
            0x08125e9a => Some((m.cpu.regs[0], 1)),
            0x08125ea6 => Some((m.cpu.regs[0] + 1, 1)),
            0x08125ea2 => Some((m.cpu.regs[1] & !3, 4)),
            0x08125eac => Some((m.cpu.regs[0] & !3, 4)),
            0x08009b28 => Some((m.cpu.regs[5] + 2, 1)),
            0x081265da => Some((m.cpu.regs[1], 1)),
            0x0812746e => Some((m.cpu.regs[1] + 1, 1)),
            0x0812662a | 0x08126666 => Some((m.cpu.regs[4], 1)),
            0x0812663c => Some((m.cpu.regs[4] + 2, 1)),
            0x0812663e => Some((m.cpu.regs[4] + 3, 1)),
            0x0812664a | 0x08126678 => Some((m.cpu.regs[4] + 1, 1)),
            0x0812667a => Some((m.cpu.regs[4] + 2, 1)),
            _ => None,
        };
        if let Some((at, n)) = access {
            if (0x08000000..0x09000000).contains(&at) {
                for i in 0..n {
                    reads.insert(at - 0x08000000 + i);
                }
            }
        }
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    steps
}
fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
}

#[test]
#[ignore = "private EUR whole relative script archive/key/index/opcode/parameter source oracle; Wine only, Linux/USA pending"]
fn european_script_archive_matches_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_ARCHIVE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_ARCHIVE_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_ARCHIVE_PRIOR_DATASET").unwrap());
    let command_bytes = fs::read(prior.join("script-command-directory.json")).unwrap();
    let effect_bytes = fs::read(prior.join("effect-event-directories.json")).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&command_bytes),
        evidence["dependency_file_sha256"]["script-command-directory.json"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        gba_core::ffta_rom::digest(&effect_bytes),
        evidence["dependency_file_sha256"]["effect-event-directories.json"]
            .as_str()
            .unwrap()
    );
    let commands: gba_core::rom_data::script_commands::ScriptCommands =
        serde_json::from_slice(&command_bytes).unwrap();
    let effects: gba_core::rom_data::effect_event_directories::EffectEventDirectories =
        serde_json::from_slice(&effect_bytes).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate_dependencies(&commands, &effects).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 2);
    for ((a, bytes, role), expected) in spans
        .iter()
        .zip(evidence["source_ranges"].as_array().unwrap())
    {
        assert_eq!(*a, expected["offset"].as_u64().unwrap() as u32);
        assert_eq!(bytes.len(), expected["length"].as_u64().unwrap() as usize);
        assert_eq!(role, expected["role"].as_str().unwrap());
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            expected["sha256"].as_str().unwrap()
        );
        assert!(
            bytes == &rom[*a as usize..*a as usize + bytes.len()],
            "source roundtrip"
        );
        for (i, v) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*a + i as u32), Some(*v));
        }
        assert_eq!(data.byte_at(*a - 1), None);
        assert_eq!(data.byte_at(*a + bytes.len() as u32), None);
    }
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), SOURCE_BYTES);
    assert_eq!(data.byte_at(u32::MAX), None);
    for (key, expected) in data.keys.iter().zip(evidence["keys"].as_array().unwrap()) {
        assert_eq!(
            [
                u64::from(key.id),
                u64::from(key.script_bank),
                u64::from(key.script_index),
                u64::from(key.text_bank),
                u64::from(key.byte3_raw)
            ],
            [
                "id",
                "script_bank",
                "script_index",
                "text_bank",
                "byte3_raw"
            ]
            .map(|k| expected[k].as_u64().unwrap())
        );
        assert_eq!(
            data.key_target(key.id),
            expected["target"]["target_offset"]
                .as_u64()
                .map(|v| v as u32)
        );
    }
    let mut header_targets = 0;
    let mut fill_targets = 0;
    let mut targets = BTreeSet::new();
    for (bank, expected) in data.banks.iter().zip(evidence["banks"].as_array().unwrap()) {
        assert_eq!(
            bank.source_offset,
            expected["source_offset"].as_u64().unwrap() as u32
        );
        assert_eq!(
            u64::from(bank.ordinal),
            expected["ordinal"].as_u64().unwrap()
        );
        for (slot, expected) in bank.slots.iter().zip(expected["slots"].as_array().unwrap()) {
            assert_eq!(u64::from(slot.index), expected["index"].as_u64().unwrap());
            let at = data.target_offset(&slot.target).unwrap();
            assert_eq!(at, expected["target_offset"].as_u64().unwrap() as u32);
            targets.insert(at);
            let (kind, n) = match slot.target {
                Target::Program { ordinal } => ("program", u64::from(ordinal)),
                Target::Header { bank } => {
                    header_targets += 1;
                    ("header", u64::from(bank))
                }
                Target::Fill { ordinal } => {
                    fill_targets += 1;
                    ("fill", u64::from(ordinal))
                }
            };
            assert_eq!(kind, expected["target_kind"].as_str().unwrap());
            assert_eq!(n, expected["target_ordinal"].as_u64().unwrap());
        }
    }
    assert_eq!((header_targets, fill_targets, targets.len()), (5, 2, 213));
    let mut opcodes = BTreeSet::new();
    let mut selectors = BTreeSet::new();
    let mut selector_links = 0;
    for (program, expected) in data
        .programs
        .iter()
        .zip(evidence["programs"].as_array().unwrap())
    {
        assert_eq!(
            program.source_offset,
            expected["source_offset"].as_u64().unwrap() as u32
        );
        assert_eq!(
            u64::from(program.ordinal),
            expected["ordinal"].as_u64().unwrap()
        );
        for (i, e) in program
            .instructions
            .iter()
            .zip(expected["instructions"].as_array().unwrap())
        {
            assert_eq!(i.source_offset, e["source_offset"].as_u64().unwrap() as u32);
            assert_eq!(u64::from(i.command), e["command"].as_u64().unwrap());
            assert_eq!(
                i.parameters,
                e["parameters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u8)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                i.effect_selector,
                e["effect_selector"].as_u64().map(|v| v as u8)
            );
            opcodes.insert(i.command);
            if let Some(s) = i.effect_selector {
                selectors.insert(s);
                selector_links += 1;
            }
        }
    }
    assert_eq!(
        (opcodes.len(), selector_links, selectors.len()),
        (110, 493, 49)
    );
    for (fill, e) in data
        .padding
        .iter()
        .zip(evidence["padding"].as_array().unwrap())
    {
        assert_eq!(
            fill.source_offset,
            e["source_offset"].as_u64().unwrap() as u32
        );
        assert_eq!(u64::from(fill.length), e["length"].as_u64().unwrap());
        assert_eq!(fill.value, 255);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert!(serde_json::from_slice::<ScriptArchive>(&encoded).unwrap() == *data);
    for case in 0..16 {
        let mut bad = data.clone();
        match case {
            0 => bad.archive_offset += 4,
            1 => bad.key_offset += 4,
            2 => {
                bad.keys.pop();
            }
            3 => bad.banks[0].ordinal = 0,
            4 => bad.banks[0].slots.swap(0, 1),
            5 => bad.banks[0].slots[0].target = Target::Program { ordinal: 65535 },
            6 => bad.banks[0].slots[0].target = Target::Header { bank: 0 },
            7 => bad.banks[0].slots[0].target = Target::Fill { ordinal: 255 },
            8 => bad.programs[0].ordinal = 1,
            9 => bad.programs[0].instructions[0].source_offset += 1,
            10 => bad.keys[1].script_bank = 0,
            11 => bad.keys[1].script_index = 255,
            12 => {
                bad.programs[0].instructions.pop();
            }
            13 => bad.programs[0].instructions[0].parameters.push(0),
            14 => bad.padding[0].length = 0,
            _ => bad.padding[0].source_offset += 1,
        };
        assert!(bad.validate().is_err(), "shape{case}");
    }
    for case in 0..6 {
        let mut bad = data.clone();
        match case {
            0 => bad.keys[0].text_bank ^= 1,
            1 => bad.programs[0].instructions[0].parameters[0] ^= 1,
            2 => bad.banks[0].slots[0].target = Target::Program { ordinal: 1 },
            3 => bad.padding[0].value ^= 1,
            _ => {
                let i = bad
                    .programs
                    .iter_mut()
                    .flat_map(|p| &mut p.instructions)
                    .find(|i| i.command == 90)
                    .unwrap();
                i.parameters[0] = if case == 4 { 0 } else { 255 };
                i.effect_selector = None;
            }
        };
        bad.validate_dependencies(&commands, &effects).unwrap();
        assert!(bad.verify_source(&rom).is_err(), "source{case}");
    }
    let mut wrong = commands.clone();
    wrong.commands[0].advance += 1;
    assert!(data.validate_dependencies(&wrong, &effects).is_err());
    let mut wrong = effects.clone();
    wrong.selectors.pop();
    assert!(data.validate_dependencies(&commands, &wrong).is_err());
    let mut wrong = data.clone();
    wrong
        .programs
        .iter_mut()
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 90)
        .unwrap()
        .effect_selector = None;
    wrong.validate().unwrap();
    assert!(wrong.validate_dependencies(&commands, &effects).is_err());
    for path in [
        "top",
        "key",
        "bank",
        "slot",
        "target",
        "program",
        "instruction",
        "fill",
    ] {
        let mut bad = serde_json::to_value(data).unwrap();
        let node = match path {
            "top" => &mut bad,
            "key" => &mut bad["keys"][0],
            "bank" => &mut bad["banks"][0],
            "slot" => &mut bad["banks"][0]["slots"][0],
            "target" => &mut bad["banks"][0]["slots"][0]["target"],
            "program" => &mut bad["programs"][0],
            "instruction" => &mut bad["programs"][0]["instructions"][0],
            _ => &mut bad["padding"][0],
        };
        node["unknown"] = true.into();
        assert!(
            serde_json::from_value::<ScriptArchive>(bad).is_err(),
            "JSON{path}"
        );
    }
    assert!(serde_json::from_slice::<ScriptArchive>(&encoded[..encoded.len() - 1]).is_err());
    let mut bad = serde_json::to_value(data).unwrap();
    bad["banks"][0]["slots"][0]["target"]["kind"] = "unknown".into();
    assert!(serde_json::from_value::<ScriptArchive>(bad).is_err());
    for case in 0..3 {
        let mut bad = serde_json::to_value(data).unwrap();
        match case {
            0 => bad["keys"][0]["id"] = 65536.into(),
            1 => bad["programs"][0]["instructions"][0]["command"] = 256.into(),
            _ => bad["programs"][0]["instructions"][0]["parameters"][0] = 256.into(),
        };
        assert!(serde_json::from_value::<ScriptArchive>(bad).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    let mut m = Machine::new(rom.clone());
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut steps = 0;
    for id in (0u32..234).chain([234, 239, 65535, 0xbeef0001]) {
        reset(&mut m, 0x08009aa0);
        m.cpu.regs[0] = id;
        steps += until(&mut m, 0x08009aa8, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], 0x08000000 + event_key_source_offset(id));
    }
    for id in 0u16..234 {
        let key = &data.keys[usize::from(id)];
        reset(&mut m, 0x08125e98);
        m.cpu.regs[0] = 0x08000000 + event_key_source_offset(u32::from(id));
        steps += until(&mut m, 0x08125eb0, &mut pcs, &mut reads);
        let target = if id == 0 {
            let base = ARCHIVE_OFFSET + word(&rom, ARCHIVE_OFFSET - 4);
            base.wrapping_add(word(&rom, base & !3).rotate_right((base & 3) * 8))
        } else {
            data.key_target(id).unwrap()
        };
        assert_eq!(m.cpu.regs[2], 0x08000000u32.wrapping_add(target));
        assert_eq!(key.id, id);
        reset(&mut m, 0x08009b28);
        m.cpu.regs[5] = 0x08000000 + event_key_source_offset(u32::from(id));
        steps += until(&mut m, 0x08009b2a, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[1], u32::from(key.text_bank));
    }
    let mut opcode_cases = 0;
    let mut relative_cases = 0;
    let mut selector_cases = 0;
    for i in data.programs.iter().flat_map(|p| &p.instructions) {
        reset(&mut m, 0x081265da);
        m.cpu.regs[1] = 0x08000000 + i.source_offset;
        m.cpu.regs[6] = 0x08000000 + gba_core::rom_data::script_commands::SOURCE_OFFSET;
        steps += until(&mut m, 0x081265e4, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(i.command));
        assert_eq!(
            m.cpu.regs[4],
            0x08000000 + gba_core::rom_data::script_commands::command_source_offset(i.command)
        );
        opcode_cases += 1;
        if i.command == 90 {
            reset(&mut m, 0x0812746e);
            m.cpu.regs[1] = 0x08000000 + i.source_offset;
            steps += until(&mut m, 0x08127476, &mut pcs, &mut reads);
            assert_eq!(
                m.cpu.regs[0],
                u32::from(u16::from(i.parameters[0]).wrapping_sub(1))
            );
            selector_cases += 1;
        }
        if i.command <= 1 {
            let (start, end, at) = if i.command == 0 {
                (0x08126628, 0x0812664c, 1)
            } else {
                (0x08126664, 0x08126688, 0)
            };
            reset(&mut m, start);
            m.cpu.regs[4] = 0x08000000 + i.source_offset;
            steps += until(&mut m, end, &mut pcs, &mut reads);
            let delta = u16::from_le_bytes(i.parameters[at..at + 2].try_into().unwrap());
            assert_eq!(
                m.cpu.regs[1],
                u32::from(
                    commands.commands[usize::from(i.command)]
                        .advance
                        .wrapping_add(delta)
                )
            );
            relative_cases += 1;
        }
    }
    let nonprogram_heads: BTreeSet<_> = data
        .banks
        .iter()
        .flat_map(|b| &b.slots)
        .filter(|s| !matches!(s.target, Target::Program { .. }))
        .map(|s| data.target_offset(&s.target).unwrap())
        .collect();
    for at in &nonprogram_heads {
        reset(&mut m, 0x081265da);
        m.cpu.regs[1] = 0x08000000 + at;
        m.cpu.regs[6] = 0x08000000 + gba_core::rom_data::script_commands::SOURCE_OFFSET;
        steps += until(&mut m, 0x081265e4, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(rom[*at as usize]));
    }
    assert_eq!(
        (
            opcode_cases,
            relative_cases,
            selector_cases,
            nonprogram_heads.len()
        ),
        (35048, 2397, 493, 6)
    );
    assert!(m.bus.ewram == ew);
    assert!(m.bus.iwram == iw);
    let owned: BTreeSet<_> = reads
        .iter()
        .copied()
        .filter(|a| data.byte_at(*a).is_some())
        .collect();
    let mut expected = BTreeSet::new();
    expected.extend(ARCHIVE_OFFSET..ARCHIVE_OFFSET + 62 * 4);
    for b in &data.banks {
        expected.extend(b.source_offset..b.source_offset + b.slots.len() as u32 * 4);
    }
    for id in 0..234 {
        expected.extend(KEY_OFFSET + id * 4..KEY_OFFSET + id * 4 + 3);
    }
    for i in data.programs.iter().flat_map(|p| &p.instructions) {
        expected.insert(i.source_offset);
        if i.command == 90 {
            expected.insert(i.source_offset + 1);
        }
        if i.command <= 1 {
            expected.extend(i.source_offset + 1..i.source_offset + 1 + i.parameters.len() as u32);
        }
    }
    expected.extend(nonprogram_heads);
    assert!(owned == expected, "original-owned-byte coverage");
    assert_eq!(owned.len(), 42333);
    fs::write(root.join("script-archive.json"), encoded).unwrap();
    let result = serde_json::json!({"status":"PASS","source_bytes":115464,"source_ranges":2,"event_keys":234,"banks":62,"nested_slots":233,"program_segments":207,"instruction_layout_records":35048,"header_target_links":5,"fill_target_links":2,"selector_links":493,"original_key_address_prefixes":238,"original_two_level_target_prefixes":234,"original_text_bank_prefixes":234,"original_opcode_prefixes":35054,"original_relative_target_prefixes":2397,"original_selector_parameter_prefixes":493,"original_owned_source_bytes_observed":owned.len(),"preserved_unread_parameter_keyraw_fill_bytes":73131,"shape_guards":16,"source_edit_guards":6,"dependency_guards":3,"json_guards":13,"rom_identity_guards":2,"instructions_coverage":steps,"pcs_coverage":pcs.len(),"scope":"storage/key/index/opcode/selectedparameter original prefixes, not whole interpreter/callback ABI/native/state-clock/ordinary reachability","runtime_publication":"pending","linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_SCRIPT_ARCHIVE_MODEL {result}");
}
