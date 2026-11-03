//! One reduced EUR source oracle; no replacement of script execution.
use super::*;
use gba_core::{rom_data::script_commands::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;

const CONTEXT: u32 = 0x02010000;
const STREAM: u32 = 0x02018000;
fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    m.cpu.regs = [0; 16];
    for n in 1..13 {
        m.cpu.regs[n] = 0x11110000 + n as u32;
    }
    m.cpu.regs[5] = CONTEXT;
    m.cpu.regs[6] = 0x08000000 + SOURCE_OFFSET;
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
        let address = match pc {
            0x081265e4 => Some(m.cpu.regs[4] + 2),
            0x081265e6 => Some(m.cpu.regs[4] + 3),
            0x081265ec => Some(m.cpu.regs[4] + 4),
            0x081265f2 => Some(m.cpu.regs[4] + 5),
            0x08126600 => Some(m.cpu.regs[4]),
            0x08126602 => Some(m.cpu.regs[4] + 1),
            0x08126634 | 0x08126670 => Some(m.cpu.regs[1]),
            0x08126636 | 0x08126672 => Some(m.cpu.regs[1] + 1),
            _ => None,
        };
        if let Some(address) = address {
            reads.insert(address - 0x08000000);
        }
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    steps
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn callback(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize + 2..at as usize + 6].try_into().unwrap())
}

#[test]
#[ignore = "private EUR script command source/field/address/BX oracle; Wine only, Linux/USA pending"]
fn european_script_commands_match_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_COMMAND_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_COMMAND_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    let (offset, bytes, role) = &spans[0];
    let expected = &evidence["source_ranges"][0];
    assert_eq!(*offset, expected["offset"].as_u64().unwrap() as u32);
    assert_eq!(bytes.len(), SOURCE_BYTES);
    assert_eq!(bytes.len(), expected["length"].as_u64().unwrap() as usize);
    assert_eq!(role, expected["role"].as_str().unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(bytes),
        expected["sha256"].as_str().unwrap()
    );
    assert_eq!(
        bytes,
        &rom[*offset as usize..*offset as usize + SOURCE_BYTES]
    );
    assert_eq!(
        catalog.sources[0].sha256,
        expected["sha256"].as_str().unwrap()
    );
    for (command, row) in data
        .commands
        .iter()
        .zip(evidence["entries"].as_array().unwrap())
    {
        assert_eq!(command.opcode, row["opcode"].as_u64().unwrap() as u8);
        assert_eq!(command.advance, row["advance"].as_u64().unwrap() as u16);
        assert_eq!(
            command.callback.rom_offset,
            row["callback_rom_offset"].as_u64().unwrap() as u32
        );
        assert_eq!(command.byte(6), None);
    }
    for address in [
        SOURCE_OFFSET - 1,
        SOURCE_OFFSET + SOURCE_BYTES as u32,
        u32::MAX,
    ] {
        assert_eq!(data.byte_at(address), None);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<ScriptCommands>(&encoded).unwrap(),
        *data
    );
    for case in 0..7 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 6,
            1 => {
                bad.commands.pop();
            }
            2 => bad.commands.push(bad.commands[0].clone()),
            3 => bad.commands.swap(0, 1),
            4 => bad.commands[0].opcode = 1,
            5 => bad.commands[0].callback.rom_offset |= 1,
            _ => bad.commands[0].callback.rom_offset = 0x1000000,
        }
        assert!(bad.validate().is_err(), "shape{case}");
    }
    for case in 0..4 {
        let mut bad = data.clone();
        match case {
            0 => bad.commands[0].advance ^= 1,
            1 => bad.commands[0].callback.rom_offset += 2,
            2 => bad.commands[0].advance = 65535,
            _ => bad.commands[0].advance = 0,
        }
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err(), "source{case}");
    }
    for path in ["top", "command", "callback"] {
        let mut bad = serde_json::to_value(data).unwrap();
        let node = match path {
            "top" => &mut bad,
            "command" => &mut bad["commands"][0],
            _ => &mut bad["commands"][0]["callback"],
        };
        node["unknown"] = true.into();
        assert!(serde_json::from_value::<ScriptCommands>(bad).is_err());
    }
    assert!(serde_json::from_slice::<ScriptCommands>(&encoded[..encoded.len() - 1]).is_err());
    for (key, value) in [("advance", 65536), ("opcode", 256)] {
        let mut bad = serde_json::to_value(data).unwrap();
        bad["commands"][0][key] = value.into();
        assert!(serde_json::from_value::<ScriptCommands>(bad).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    for opcode in (0u8..115).chain([115, 255]) {
        let at = command_source_offset(opcode);
        reset(&mut m, 0x081265da);
        m.cpu.regs[1] = STREAM;
        m.bus.ewram[0x18000] = opcode;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        instructions += until(&mut m, 0x081265f8, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[4], 0x08000000 + at);
        assert_eq!(m.cpu.regs[3], callback(&rom, at));
        assert_eq!(m.cpu.regs[1], STREAM);
        assert_eq!(m.cpu.regs[5..15], regs[5..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        reset(&mut m, 0x08126600);
        m.cpu.regs[4] = 0x08000000 + at;
        let cursor = [0u16, 32767, 65535][usize::from(opcode) % 3];
        m.bus.ewram[0x10000..0x10002].copy_from_slice(&cursor.to_le_bytes());
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        ew[0x10000..0x10002].copy_from_slice(&cursor.wrapping_add(half(&rom, at)).to_le_bytes());
        instructions += until(&mut m, 0x0812660e, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(cursor) + u32::from(half(&rom, at)));
        assert_eq!(m.cpu.regs[1], u32::from(cursor));
        assert_eq!(m.cpu.regs[2..15], regs[2..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        if opcode < 115 {
            reset(&mut m, 0x08145c14);
            let target = data.commands[usize::from(opcode)]
                .callback
                .guest_word()
                .unwrap();
            m.cpu.regs[3] = target;
            let ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            instructions += until(&mut m, target & !1, &mut pcs, &mut reads);
            assert_eq!(m.cpu.cpsr & 0x20, 0x20);
            assert_eq!(m.bus.ewram, ew);
            assert_eq!(m.bus.iwram, iw);
        } else {
            assert_eq!(data.byte_at(at), None);
        }
    }
    for cursor in [0u16, 32767, 32768, 65535] {
        reset(&mut m, 0x081265d2);
        m.bus.ewram[0x10000..0x10002].copy_from_slice(&cursor.to_le_bytes());
        m.bus.ewram[0x1000c..0x10010].copy_from_slice(&STREAM.to_le_bytes());
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, 0x081265da, &mut pcs, &mut reads);
        assert_eq!(
            m.cpu.regs[1],
            STREAM.wrapping_add((cursor as i16 as i32) as u32)
        );
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for (start, end, second) in [
        (0x08126628, 0x0812664c, false),
        (0x08126664, 0x08126688, true),
    ] {
        for (opcode, delta) in [
            (0u8, 0u16),
            (1, 65535),
            (114, 32767),
            (115, 1),
            (255, 65535),
        ] {
            reset(&mut m, start);
            m.cpu.regs[4] = STREAM;
            m.bus.ewram[0x18000] = opcode;
            let at = if second { 0x18001 } else { 0x18002 };
            m.bus.ewram[at..at + 2].copy_from_slice(&delta.to_le_bytes());
            let ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            instructions += until(&mut m, end, &mut pcs, &mut reads);
            assert_eq!(
                m.cpu.regs[1],
                u32::from(half(&rom, command_source_offset(opcode)).wrapping_add(delta))
            );
            if second {
                assert_eq!(m.cpu.regs[2], 128);
            } else {
                assert_eq!(m.cpu.regs[2], u32::from(m.bus.ewram[0x18001]));
            }
            assert_eq!(m.bus.ewram, ew);
            assert_eq!(m.bus.iwram, iw);
        }
    }
    for result in [0u32, 0x110, 0x114, 0x115, 65535] {
        reset(&mut m, 0x0812660e);
        m.cpu.regs[2] = result;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let end = if result == 0x110 {
            0x081265d2
        } else {
            0x08126616
        };
        instructions += until(&mut m, end, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[2], result);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    let owned: BTreeSet<_> = reads
        .iter()
        .copied()
        .filter(|a| data.byte_at(*a).is_some())
        .collect();
    assert_eq!(
        owned,
        (SOURCE_OFFSET..SOURCE_OFFSET + SOURCE_BYTES as u32).collect()
    );
    fs::write(root.join("script-commands.json"), &encoded).unwrap();
    let result = serde_json::json!({"status":"PASS","commands":115,"source_bytes":690,"original_owned_source_bytes_observed":owned.len(),"original_callback_prefixes":117,"original_advance_prefixes":117,"original_BX_prefixes":115,"original_signed_cursor_prefixes":4,"original_relative_target_prefixes":10,"original_return_loop_prefixes":5,"shape_guards":7,"source_edit_guards":4,"json_guards":6,"rom_identity_guards":2,"instructions_coverage":instructions,"pcs_coverage":pcs.len(),"runtime_publication":"pending","native_fullstate_clock":"pending","ordinary_source_use":"pending","linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_SCRIPT_COMMAND_MODEL {result}");
}
