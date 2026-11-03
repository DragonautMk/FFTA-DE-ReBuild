//! One reduced EUR/Wine source oracle; no replacement predicates or whole interpreter ABI.
use super::*;
use gba_core::{rom_data::auxiliary_scripts::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;

fn reset(m: &mut Machine, pc: u32) {
    m.cpu.regs = [0; 16];
    for n in 4..12 {
        m.cpu.regs[n] = 0x14004000 + n as u32 * 0x101;
    }
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(
    m: &mut Machine,
    end: u32,
    limit: usize,
    pcs: &mut BTreeSet<u32>,
    reads: &mut BTreeSet<u32>,
) -> usize {
    let mut steps = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(steps < limit, "pc{pc:08x} end{end:08x}");
        pcs.insert(pc);
        let reg = |n: usize| m.cpu.regs[n];
        let access = match pc {
            0x0800a204 | 0x0800a20e => Some((reg(1) & !1, 2)),
            0x0800a24e => Some((reg(3), 1)),
            0x0800a25c => Some((reg(4) + 2, 1)),
            0x0800a25e => Some((reg(4) + 3, 1)),
            0x0800a264 => Some((reg(4) + 4, 1)),
            0x0800a26a => Some((reg(4) + 5, 1)),
            0x0800a278 => Some((reg(4), 1)),
            0x0800a27a => Some((reg(4) + 1, 1)),
            0x0800a298 => Some((reg(1) + 1, 1)),
            0x0800a29a => Some((reg(1) + 2, 1)),
            0x0800a2e8 => Some((reg(4) + 3, 1)),
            0x0800a2ea => Some((reg(4) + 4, 1)),
            0x0800a3c6 | 0x0800a4f8 => Some((reg(5) + 2, 1)),
            0x0800a3c8 | 0x0800a4fa | 0x0800a404 => Some((reg(5) + 3, 1)),
            0x0800a406 => Some((reg(5) + 4, 1)),
            0x0800a43c => Some((reg(3) + 3, 1)),
            0x0800a43e => Some((reg(3) + 4, 1)),
            0x0800a4b2 => Some((reg(6) + 3, 1)),
            0x0800a4b4 => Some((reg(6) + 4, 1)),
            0x0800a556 => Some((reg(7) + 2, 1)),
            0x0800a558 => Some((reg(7) + 3, 1)),
            0x0800a5c2 | 0x0800a672 | 0x0800a6a2 => Some((reg(4) + 2, 1)),
            0x0800a5c4 => Some((reg(4) + 3, 1)),
            0x0800a634 => Some((reg(5) + 1, 1)),
            0x0800a636 => Some((reg(5) + 2, 1)),
            0x0800a670 | 0x0800a6a0 => Some((reg(4) + 1, 1)),
            _ => None,
        };
        if let Some((at, n)) = access {
            for a in at..at + n {
                let offset = a.wrapping_sub(0x08000000);
                if (SOURCE_OFFSET..SOURCE_END).contains(&offset)
                    || (DIRECTORY_OFFSET..DIRECTORY_OFFSET + DIRECTORY_BYTES as u32)
                        .contains(&offset)
                {
                    reads.insert(offset);
                }
            }
        }
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    steps
}
fn put16(m: &mut Machine, at: usize, v: u16) {
    m.bus.ewram[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
fn put32(m: &mut Machine, at: usize, v: u32) {
    m.bus.ewram[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn half(m: &Machine, at: usize) -> u16 {
    u16::from_le_bytes(m.bus.ewram[at..at + 2].try_into().unwrap())
}

#[test]
#[ignore = "private EUR auxiliary script model/original-reader oracle; Wine only, Linux/USA pending"]
fn european_auxiliary_scripts_match_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_AUXILIARY_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_AUXILIARY_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let commands = &catalog.commands;
    data.validate_dependencies(commands).unwrap();
    data.verify_source(&rom).unwrap();
    commands.verify_source(&rom).unwrap();
    assert_eq!(catalog.sources.len(), 2);
    for (source, independent) in catalog
        .sources
        .iter()
        .zip(evidence["source_ranges"].as_array().unwrap())
    {
        assert_eq!(
            u64::from(source.offset),
            independent["offset"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(source.length),
            independent["length"].as_u64().unwrap()
        );
        assert_eq!(source.sha256, independent["sha256"].as_str().unwrap());
    }
    for (offset, bytes, _) in data
        .source_spans()
        .unwrap()
        .into_iter()
        .chain(commands.source_spans().unwrap())
    {
        assert_eq!(bytes, rom[offset as usize..offset as usize + bytes.len()]);
        for (n, &byte) in bytes.iter().enumerate() {
            assert_eq!(
                if offset == SOURCE_OFFSET {
                    data.byte_at(offset + n as u32)
                } else {
                    commands.byte_at(offset + n as u32)
                },
                Some(byte)
            );
        }
    }
    assert_eq!(data.byte_at(SOURCE_OFFSET - 1), None);
    assert_eq!(data.byte_at(SOURCE_END), None);
    assert_eq!(commands.byte_at(DIRECTORY_OFFSET - 1), None);
    assert_eq!(commands.byte_at(DIRECTORY_OFFSET + 102), None);
    for (command, independent) in commands
        .commands
        .iter()
        .zip(evidence["commands"].as_array().unwrap())
    {
        assert_eq!(
            u64::from(command.opcode),
            independent["opcode"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(command.advance),
            independent["advance"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(command.callback.rom_offset),
            independent["callback_offset"].as_u64().unwrap()
        );
    }
    for program in data.banks.iter().flat_map(|b| &b.programs) {
        let independent = &evidence["programs"][usize::from(program.id)];
        assert_eq!(
            u64::from(program.source_offset),
            independent["source_offset"].as_u64().unwrap()
        );
        assert_eq!(
            serde_json::to_value(&program.instructions).unwrap(),
            independent["instructions"]
        );
    }
    let mut interior = 0;
    for link in evidence["slot_links"].as_array().unwrap() {
        let bank = &data.banks[link["bank_id"].as_u64().unwrap() as usize];
        let entry = &bank.entries[link["slot_id"].as_u64().unwrap() as usize];
        assert_eq!(
            u64::from(entry.program_id),
            link["program_id"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(bank.entry_offset(entry).unwrap()),
            link["target_offset"].as_u64().unwrap()
        );
        interior += usize::from(entry.instruction != 0);
    }
    assert_eq!(interior, 30);
    for field in ["halfwords", "words"] {
        for e in evidence[field].as_array().unwrap() {
            let at = e["offset"].as_u64().unwrap() as usize;
            let value = if field == "halfwords" {
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            } else {
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            };
            assert_eq!(u64::from(value), e["value"].as_u64().unwrap());
        }
    }
    assert_eq!(
        serde_json::from_slice::<AuxiliaryArchive>(&serde_json::to_vec(data).unwrap()).unwrap(),
        *data
    );
    assert_eq!(
        serde_json::from_slice::<AuxiliaryCommands>(&serde_json::to_vec(commands).unwrap())
            .unwrap(),
        *commands
    );
    let mut guards = Vec::new();
    macro_rules! invalid {
        ($name:expr, $edit:expr) => {{
            let mut v = data.clone();
            ($edit)(&mut v);
            assert!(v.validate().is_err(), "{}", $name);
            guards.push($name);
        }};
    }
    invalid!("archive root", |v: &mut AuxiliaryArchive| v
        .source_offset +=
        2);
    invalid!("bank count", |v: &mut AuxiliaryArchive| {
        v.banks.pop();
    });
    invalid!("bank ID", |v: &mut AuxiliaryArchive| v.banks[0].id = 1);
    invalid!("bank offset", |v: &mut AuxiliaryArchive| v.banks[0]
        .source_offset +=
        2);
    invalid!("entry count", |v: &mut AuxiliaryArchive| {
        v.banks[0].entries.pop();
    });
    invalid!("entry program", |v: &mut AuxiliaryArchive| v.banks[0]
        .entries[0]
        .program_id =
        65535);
    invalid!("entry instruction", |v: &mut AuxiliaryArchive| v.banks[0]
        .entries[0]
        .instruction =
        65535);
    invalid!("cross-bank entry", |v: &mut AuxiliaryArchive| v.banks[0]
        .entries[0]
        .program_id =
        v.banks[1].programs[0].id);
    invalid!("program ID", |v: &mut AuxiliaryArchive| v.banks[0]
        .programs[0]
        .id = 1);
    invalid!("program offset", |v: &mut AuxiliaryArchive| v.banks[0]
        .programs[0]
        .source_offset +=
        1);
    invalid!("empty program", |v: &mut AuxiliaryArchive| v.banks[0]
        .programs[0]
        .instructions
        .clear());
    invalid!("instruction offset", |v: &mut AuxiliaryArchive| v.banks
        [0]
    .programs[0]
        .instructions[0]
        .source_offset +=
        1);
    invalid!("instruction opcode", |v: &mut AuxiliaryArchive| v.banks
        [0]
    .programs[0]
        .instructions[0]
        .command =
        17);
    invalid!("early zero", |v: &mut AuxiliaryArchive| v.banks[0]
        .programs[0]
        .instructions[0]
        .command = 0);
    invalid!("missing terminator", |v: &mut AuxiliaryArchive| v.banks
        [0]
    .programs[0]
        .instructions
        .last_mut()
        .unwrap()
        .command =
        1);
    invalid!("parameter extent", |v: &mut AuxiliaryArchive| v.banks[0]
        .programs[0]
        .instructions[0]
        .parameters
        .push(1));
    invalid!("internal gap value", |v: &mut AuxiliaryArchive| v.banks
        [1]
    .alignment_after[0] =
        0);
    invalid!("internal gap extent", |v: &mut AuxiliaryArchive| v.banks[0]
        .alignment_after
        .push(255));
    invalid!("excluded final gap", |v: &mut AuxiliaryArchive| v.banks[7]
        .alignment_after
        .extend([255, 255]));
    for n in 0..6 {
        let mut v = commands.clone();
        match n {
            0 => v.source_offset += 2,
            1 => {
                v.commands.pop();
            }
            2 => v.commands[0].opcode = 1,
            3 => v.commands[0].advance = 0,
            4 => v.commands[0].callback.rom_offset = 1,
            _ => v.commands[0].callback = v.commands[1].callback.clone(),
        }
        assert!(v.validate().is_err());
        guards.push("directory shape");
    }
    let mut edited = data.clone();
    edited
        .banks
        .iter_mut()
        .flat_map(|b| &mut b.programs)
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 2)
        .unwrap()
        .parameters[0] ^= 128;
    edited.validate_dependencies(commands).unwrap();
    assert!(edited.verify_source(&rom).is_err());
    guards.push("coherent parameter provenance");
    let mut edited = data.clone();
    edited.banks[0].entries[0] = edited.banks[0].entries[4].clone();
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    guards.push("coherent entry provenance");
    let mut edited = commands.clone();
    edited.commands[0].callback.rom_offset += 2;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    assert!(data.validate_dependencies(&edited).is_err());
    guards.push("callback provenance and dependency");
    let mut edited = commands.clone();
    edited.commands[1].advance += 1;
    edited.validate().unwrap();
    assert!(data.validate_dependencies(&edited).is_err());
    guards.push("advance dependency");
    // A coherent short relative operand must return Err, never index past a vector.
    let mut edited = data.clone();
    let ins = edited
        .banks
        .iter_mut()
        .flat_map(|b| &mut b.programs)
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 2)
        .unwrap();
    ins.command = 4;
    let mut directory = commands.clone();
    directory.commands[4].advance = 4;
    edited.validate().unwrap();
    assert!(edited.validate_dependencies(&directory).is_err());
    guards.push("short relative operand");
    for n in 0..5 {
        let mut json = serde_json::to_value(data).unwrap();
        let object = match n {
            0 => &mut json,
            1 => &mut json["banks"][0],
            2 => &mut json["banks"][0]["entries"][0],
            3 => &mut json["banks"][0]["programs"][0],
            _ => &mut json["banks"][0]["programs"][0]["instructions"][0],
        };
        object["unknown"] = 1.into();
        assert!(serde_json::from_value::<AuxiliaryArchive>(json).is_err());
        guards.push("archive strict JSON");
    }
    for n in 0..3 {
        let mut json = serde_json::to_value(commands).unwrap();
        let object = match n {
            0 => &mut json,
            1 => &mut json["commands"][0],
            _ => &mut json["commands"][0]["callback"],
        };
        object["unknown"] = 1.into();
        assert!(serde_json::from_value::<AuxiliaryCommands>(json).is_err());
        guards.push("directory strict JSON");
    }
    assert!(Catalog::from_rom(&[]).is_err());
    guards.push("ROM length");
    let mut wrong = rom.clone();
    wrong[SOURCE_OFFSET as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards.push("ROM identity");

    // Fresh controlled scratch context, never a gameplay checkpoint/save/input edit.
    const CONTEXT: u32 = 0x02011000;
    let mut m = Machine::new(rom.clone());
    let (mut pcs, mut reads) = (BTreeSet::new(), BTreeSet::new());
    let (mut steps, mut lookups, mut dispatches, mut advances, mut relatives) = (0, 0, 0, 0, 0);
    for bank in &data.banks {
        for (slot, entry) in bank.entries.iter().enumerate() {
            m.bus.ewram[0x11000..0x11008]
                .copy_from_slice(&[bank.id, slot as u8, 0xa5, 0x5a, 0x12, 0x34, 0x56, 0x78]);
            let mut expected = m.bus.ewram.clone();
            expected[0x11002..0x11004].fill(0);
            expected[0x11004..0x11008]
                .copy_from_slice(&(0x08000000 + bank.entry_offset(entry).unwrap()).to_le_bytes());
            reset(&mut m, 0x0800a1f8);
            m.cpu.regs[0] = CONTEXT;
            steps += until(&mut m, 0x0800a216, 32, &mut pcs, &mut reads);
            assert_eq!(m.bus.ewram, expected);
            lookups += 1;
        }
    }
    for program in data.banks.iter().flat_map(|b| &b.programs) {
        for ins in &program.instructions {
            reset(&mut m, 0x0800a240);
            m.cpu.regs[0] = CONTEXT;
            put32(&mut m, 0x11004, 0x08000000 + program.source_offset);
            put16(
                &mut m,
                0x11002,
                (ins.source_offset - program.source_offset) as u16,
            );
            let expected = m.bus.ewram.clone();
            if ins.command == 0 {
                steps += until(&mut m, 0x08000300, 40, &mut pcs, &mut reads);
                assert_eq!(m.cpu.regs[0], 1);
                assert_eq!(m.cpu.regs[13], 0x03007e00);
            } else {
                steps += until(&mut m, 0x0800a270, 40, &mut pcs, &mut reads);
                assert_eq!(m.cpu.regs[3], 0x08000000 + ins.source_offset);
                assert_eq!(
                    m.cpu.regs[4],
                    0x08000000 + command_source_offset(ins.command)
                );
                assert_eq!(
                    m.cpu.regs[2],
                    commands.commands[usize::from(ins.command)]
                        .callback
                        .guest_word()
                        .unwrap()
                );
            }
            assert_eq!(m.bus.ewram, expected);
            dispatches += 1;
        }
    }
    // Canonical storage has no opcode3; observe its actual directory row with a RAM instruction.
    reset(&mut m, 0x0800a240);
    m.cpu.regs[0] = CONTEXT;
    put32(&mut m, 0x11004, 0x02012000);
    put16(&mut m, 0x11002, 0);
    m.bus.ewram[0x12000] = 3;
    steps += until(&mut m, 0x0800a270, 40, &mut pcs, &mut reads);
    assert_eq!(
        m.cpu.regs[2],
        commands.commands[3].callback.guest_word().unwrap()
    );
    for command in &commands.commands[1..] {
        for cursor in [0u16, 1, 32767, 32768, 65535] {
            reset(&mut m, 0x0800a278);
            m.cpu.regs[4] = 0x08000000 + command_source_offset(command.opcode);
            m.cpu.regs[5] = CONTEXT;
            put16(&mut m, 0x11002, cursor);
            let mut expected = m.bus.ewram.clone();
            expected[0x11002..0x11004]
                .copy_from_slice(&cursor.wrapping_add(command.advance).to_le_bytes());
            steps += until(&mut m, 0x0800a286, 20, &mut pcs, &mut reads);
            assert_eq!(m.bus.ewram, expected);
            advances += 1;
        }
    }
    for edge in evidence["relative_links"].as_array().unwrap() {
        let at = edge["instruction_offset"].as_u64().unwrap() as u32;
        let op = edge["command"].as_u64().unwrap() as u8;
        let raw = edge["displacement_u16"].as_u64().unwrap() as u16;
        let (start, end, context_reg, source_reg) = match op {
            1 => (0xa298, 0xa2a6, 0, 1),
            4 => (0xa2e8, 0xa2f6, 5, 4),
            6 => (0xa3c6, 0xa3d4, 6, 5),
            7 => (0xa404, 0xa412, 6, 5),
            8 => (0xa43c, 0xa44a, 4, 3),
            9 => (0xa4b2, 0xa4c4, 8, 6),
            10 => (0xa4f8, 0xa506, 6, 5),
            11 => (0xa556, 0xa568, 9, 7),
            12 | 13 => (0xa5c2, 0xa5d0, 5, 4),
            14 => (0xa634, 0xa642, 6, 5),
            15 => (0xa670, 0xa67e, 5, 4),
            16 => (0xa6a0, 0xa6ae, 5, 4),
            _ => unreachable!(),
        };
        reset(&mut m, 0x08000000 + start);
        m.cpu.regs[context_reg] = CONTEXT;
        m.cpu.regs[source_reg] = 0x08000000 + at;
        let cursor = (at as u16).wrapping_mul(17);
        put16(&mut m, 0x11002, cursor);
        let mut expected = m.bus.ewram.clone();
        expected[0x11002..0x11004].copy_from_slice(&cursor.wrapping_add(raw).to_le_bytes());
        steps += until(&mut m, 0x08000000 + end, 24, &mut pcs, &mut reads);
        assert_eq!(m.bus.ewram, expected, "relative op{op} at{at:x}");
        relatives += 1;
    }
    let canonical_reads = reads.clone();
    assert_eq!(canonical_reads.len(), 2887); //900index+913opcode+978relative-param+96directory; row0 remains unread.
    let mut raw_probes = 0;
    for (action, slot) in [
        (8, 0),
        (8, 1),
        (255, 255),
        (2, 63),
        (0x102, 0x100),
        (u32::MAX, 511),
    ] {
        reset(&mut m, 0x0800a1f8);
        m.cpu.regs[0] = CONTEXT;
        m.bus.ewram[0x11000] = action as u8;
        m.bus.ewram[0x11001] = slot as u8;
        let bank_index = bank_index_source_offset(action) as usize;
        let bank = SOURCE_OFFSET
            + u32::from(u16::from_le_bytes(
                rom[bank_index..bank_index + 2].try_into().unwrap(),
            ));
        let index = slot_index_source_offset(bank, slot) as usize;
        let aligned = index & !1;
        let relative = u16::from_le_bytes(rom[aligned..aligned + 2].try_into().unwrap());
        let relative = u32::from(relative).rotate_right((index as u32 & 1) * 8);
        let target = (0x08000000u32 + bank).wrapping_add(relative);
        steps += until(&mut m, 0x0800a212, 30, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[2], target);
        raw_probes += 1;
    }
    for raw in [0u32, 1, 32767, 32768, 65535, 65536, u32::MAX] {
        reset(&mut m, 0x0800a240);
        m.cpu.regs[0] = CONTEXT;
        put32(&mut m, 0x11004, 0x02020000);
        put16(&mut m, 0x11002, raw as u16);
        let address = 0x02020000u32.wrapping_add(i32::from(raw as i16) as u32);
        m.bus.ewram[(address - 0x02000000) as usize] = 1;
        steps += until(&mut m, 0x0800a270, 40, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[3], address);
        raw_probes += 1;
    }
    for op in [17u8, 255] {
        reset(&mut m, 0x0800a240);
        m.cpu.regs[0] = CONTEXT;
        put32(&mut m, 0x11004, 0x02012000);
        put16(&mut m, 0x11002, 0);
        m.bus.ewram[0x12000] = op;
        steps += until(&mut m, 0x0800a270, 40, &mut pcs, &mut reads);
        let at = command_source_offset(op) as usize;
        assert_eq!(
            m.cpu.regs[2],
            u32::from_le_bytes(rom[at + 2..at + 6].try_into().unwrap())
        );
        raw_probes += 1;
    }
    let mut wrapper_probes = 0;
    for (action, flag, live) in [
        (0u32, 0u32, 62u8),
        (7, 1, 62),
        (0x107, 0x100, 255),
        (u32::MAX, 0x101, 255),
        (2, 255, 1),
    ] {
        reset(&mut m, 0x0800a1a4);
        m.cpu.regs[0] = action;
        m.cpu.regs[1] = flag;
        m.bus.ewram[0x2191] = live;
        steps += until(&mut m, 0x0800a1e6, 32, &mut pcs, &mut reads);
        let at = (m.cpu.regs[13] - 0x03000000) as usize;
        assert_eq!(
            &m.bus.iwram[at..at + 2],
            &[action as u8, if flag as u8 != 0 { live } else { 0 }]
        );
        wrapper_probes += 1;
    }
    for action in [2u32, 0x102, 0xffffff02, 0, 7, 0x100] {
        reset(&mut m, 0x0800a158);
        m.cpu.regs[0] = action;
        steps += until(&mut m, 0x0800a1a4, 32, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], action);
        assert_eq!(m.cpu.regs[1], u32::from(action != 2));
        wrapper_probes += 1;
    }
    let mut wrap_probes = 0;
    for cursor in [0x7fffu16, 0xffff] {
        for displacement in [0u16, 1, 32767, 32768, 65535] {
            reset(&mut m, 0x0800a298);
            m.cpu.regs[0] = CONTEXT;
            m.cpu.regs[1] = 0x02012000;
            put16(&mut m, 0x11002, cursor);
            put16(&mut m, 0x12001, displacement);
            steps += until(&mut m, 0x08000300, 20, &mut pcs, &mut reads);
            assert_eq!(half(&m, 0x11002), cursor.wrapping_add(displacement));
            wrap_probes += 1;
        }
    }
    assert_eq!(
        (lookups, dispatches, advances, relatives),
        (442, 913, 80, 489)
    );
    fs::write(
        root.join("auxiliary-script-archive.json"),
        serde_json::to_vec_pretty(data).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("auxiliary-script-command-directory.json"),
        serde_json::to_vec_pretty(commands).unwrap(),
    )
    .unwrap();
    let result = serde_json::json!({"source_bytes":4200,"archive_bytes":4098,"directory_bytes":102,"guards":guards,"lookups":lookups,"dispatch_prefixes":dispatches,"extra_opcode3_RAM_prefix":1,"advance_prefixes":advances,"relative_suffixes":relatives,"relative_wrap_probes":wrap_probes,"raw_probes":raw_probes,"wrapper_probes":wrapper_probes,"original_canonical_source_bytes_observed":canonical_reads.len(),"preserved_unread_source_bytes":4200-canonical_reads.len(),"instructions_coverage_only":steps,"instruction_pcs":pcs.len(),"scope":"original source prefixes only; no predicates/whole-interpreter ABI/native/fullstate-clock/ordinary proof","linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("AUXILIARY_SOURCE_MODEL {}", result);
}
