//! Bounded source traversal, not a host sequencer. Only authenticated original ROM is analyzed.
use super::audio_directory::AudioDirectory;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub tracks: usize,
    pub states: usize,
    pub default_ram_notes: usize,
    pub max_pattern_depth: usize,
    pub consumed_bytes: usize,
    pub consumed_ranges: usize,
    pub programs: usize,
    pub note_pairs: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InstructionObservation {
    pub command: u8,
    pub end: u32,
    pub explicit: bool,
}
pub(crate) struct Graph {
    pub summary: Summary,
    pub programs: BTreeMap<(u32, u8), BTreeSet<u8>>,
    pub consumed: Vec<(u32, u32)>,
    pub instructions: BTreeMap<u32, InstructionObservation>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    pc: usize,
    running: u8,
    stack: Vec<usize>,
    repeat: u8,
    program: Option<u8>,
    key: u8,
}
fn byte(rom: &[u8], at: usize) -> Result<u8, String> {
    rom.get(at)
        .copied()
        .ok_or_else(|| format!("Truncated M4A source at {at:x}"))
}
fn word(rom: &[u8], at: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        rom.get(at..at + 4)
            .ok_or("Truncated M4A pointer")?
            .try_into()
            .unwrap(),
    ))
}
fn pointer(rom: &[u8], at: usize) -> Result<usize, String> {
    let offset = word(rom, at)?
        .checked_sub(0x08000000)
        .ok_or("Non-ROM M4A branch")? as usize;
    if offset >= rom.len() {
        return Err("M4A branch outside source ROM".into());
    }
    Ok(offset)
}
fn driver(rom: &[u8]) -> Result<(), String> {
    for (pc, at, value) in [
        (0x1482f4, 0x148320, 0x08370a90),
        (0x148f78, 0x148fc4, 0x02008e60),
        (0x148e02, 0x148e98, 0x02008e60),
        (0x148e04, 0x148e9c, 0x08149d31),
        (0x148e10, 0x148ea8, 0x08149e89),
        (0x149e92, 0x149ea4, 0x08370d38),
    ] {
        let op = u16::from_le_bytes(
            rom.get(pc..pc + 2)
                .ok_or("Truncated M4A driver")?
                .try_into()
                .unwrap(),
        );
        if op & 0xf800 != 0x4800
            || ((pc + 4) & !3) + (op as usize & 255) * 4 != at
            || word(rom, at)? != value
        {
            return Err("M4A source command initialization mismatch".into());
        }
    }
    for (at, expected) in [
        (0x1482f2, 0x2124),
        (0x1485b0, 0x2980),
        (0x1485bc, 0x29bd),
        (0x1485c2, 0x29cf),
        (0x1485d8, 0x29b0),
        (0x1485de, 0x38b1),
        (0x1483ee, 0x005a),
        (0x1483f0, 0x18d2),
        (0x1483f2, 0x0092),
        (0x14882e, 0x7808),
        (0x148838, 0x0089),
    ] {
        if u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) != expected {
            return Err("M4A original command/selector contract mismatch".into());
        }
    }
    Ok(())
}
pub(crate) fn analyze(rom: &[u8], directory: &AudioDirectory) -> Result<Graph, String> {
    directory.validate()?;
    driver(rom)?;
    let mut programs: BTreeMap<(u32, u8), BTreeSet<u8>> = BTreeMap::new();
    let mut spans = BTreeSet::new();
    let mut instructions = BTreeMap::new();
    let mut summary = Summary {
        tracks: 0,
        states: 0,
        default_ram_notes: 0,
        max_pattern_depth: 0,
        consumed_bytes: 0,
        consumed_ranges: 0,
        programs: 0,
        note_pairs: 0,
    };
    for header in &directory.headers {
        for &entry in &header.track_addresses {
            let pc = entry.checked_sub(0x08000000).ok_or("Non-ROM M4A track")? as usize;
            let mut pending = vec![State {
                pc,
                running: 0,
                stack: vec![],
                repeat: 0,
                program: None,
                key: 0,
            }];
            let mut seen = BTreeSet::new();
            summary.tracks += 1;
            while let Some(mut s) = pending.pop() {
                if !seen.insert(s.clone()) {
                    continue;
                }
                if seen.len() > 1_000_000 || summary.states >= 8_000_000 {
                    return Err("M4A producer state guard exceeded".into());
                }
                summary.states += 1;
                summary.max_pattern_depth = summary.max_pattern_depth.max(s.stack.len());
                let start = s.pc;
                let raw = byte(rom, start)?;
                let command = if raw >= 0x80 {
                    s.pc += 1;
                    if raw >= 0xbd {
                        s.running = raw;
                    }
                    raw
                } else {
                    if s.running < 0xbd {
                        return Err("M4A running status before initialization".into());
                    }
                    s.running
                };
                let mut targets = Vec::new();
                match command {
                    0x80..=0xb0 => targets.push(s.clone()),
                    0xb1 | 0xb6..=0xb8 | 0xc6..=0xc7 | 0xc9..=0xcb => {}
                    0xb2 => {
                        let mut next = s.clone();
                        next.pc = pointer(rom, s.pc)?;
                        targets.push(next);
                        s.pc += 4;
                    }
                    0xb3 => {
                        if s.stack.len() < 3 {
                            let mut next = s.clone();
                            next.pc = pointer(rom, s.pc)?;
                            next.stack.push(s.pc + 4);
                            targets.push(next);
                            s.pc += 4;
                        }
                    }
                    0xb4 => {
                        let mut next = s.clone();
                        if let Some(ret) = next.stack.pop() {
                            next.pc = ret;
                        }
                        targets.push(next);
                    }
                    0xb5 => {
                        let count = byte(rom, s.pc)?;
                        let target = pointer(rom, s.pc + 1)?;
                        s.pc += 5;
                        let mut next = s.clone();
                        if count == 0 {
                            next.pc = target;
                        } else {
                            next.repeat = next.repeat.wrapping_add(1);
                            if next.repeat < count {
                                next.pc = target;
                            } else {
                                next.repeat = 0;
                            }
                        }
                        targets.push(next);
                    }
                    0xb9 => {
                        let op = byte(rom, s.pc)?;
                        if op > 17 {
                            return Err("Unsupported M4A memory operation".into());
                        }
                        if op < 6 {
                            s.pc += 3;
                            targets.push(s.clone());
                        } else {
                            let target = pointer(rom, s.pc + 3)?;
                            s.pc += 7;
                            targets.push(s.clone());
                            let mut next = s.clone();
                            next.pc = target;
                            targets.push(next);
                        }
                    }
                    0xcd => {
                        // These are the only extended operations in this exact source graph.
                        // Wave/tone replacement would require additional producer state.
                        if !matches!(byte(rom, s.pc)?, 8 | 9) {
                            return Err("Unproved M4A extended producer operation".into());
                        }
                        s.pc += 2;
                        targets.push(s.clone());
                    }
                    0xcc => {
                        s.pc += 2;
                        targets.push(s.clone());
                    }
                    0xce => {
                        if byte(rom, s.pc)? < 0x80 {
                            s.key = byte(rom, s.pc)?;
                            s.pc += 1;
                        }
                        targets.push(s.clone());
                    }
                    0xcf..=0xff => {
                        for i in 0..3 {
                            let value = byte(rom, s.pc)?;
                            if value >= 0x80 {
                                break;
                            }
                            if i == 0 {
                                s.key = value;
                            }
                            s.pc += 1;
                        }
                        if let Some(program) = s.program {
                            programs
                                .entry((header.tone_bank_address, program))
                                .or_default()
                                .insert(s.key);
                        } else {
                            summary.default_ram_notes += 1;
                        }
                        targets.push(s.clone());
                    }
                    0xba..=0xc8 => {
                        if command == 0xbd {
                            let program = byte(rom, s.pc)?;
                            s.program = Some(program);
                            programs
                                .entry((header.tone_bank_address, program))
                                .or_default();
                        }
                        s.pc += 1;
                        targets.push(s.clone());
                    }
                    _ => return Err("Unproved M4A command".into()),
                }
                if s.pc <= start || s.pc > rom.len() {
                    return Err("Invalid M4A consumed source extent".into());
                }
                let observation = InstructionObservation {
                    command,
                    end: s.pc as u32,
                    explicit: raw >= 0x80,
                };
                if instructions
                    .insert(start as u32, observation)
                    .is_some_and(|prior| prior != observation)
                {
                    return Err("Ambiguous physical M4A instruction interpretation".into());
                }
                spans.insert((start as u32, s.pc as u32));
                pending.extend(targets);
            }
        }
    }
    let mut consumed: Vec<(u32, u32)> = Vec::new();
    for (a, z) in spans {
        if let Some(last) = consumed.last_mut().filter(|last| a <= last.1) {
            last.1 = last.1.max(z);
        } else {
            consumed.push((a, z));
        }
    }
    summary.consumed_ranges = consumed.len();
    summary.consumed_bytes = consumed.iter().map(|(a, z)| (z - a) as usize).sum();
    summary.programs = programs.len();
    summary.note_pairs = programs.values().map(BTreeSet::len).sum();
    Ok(Graph {
        summary,
        programs,
        consumed,
        instructions,
    })
}
