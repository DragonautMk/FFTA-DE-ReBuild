//! Lossless producer-bounded EUR sequence instructions and their original driver tables.
//! This is a source model: the mutable guest sequencer, running status and clock remain original.
pub use super::audio_producers::Summary as ProducerSummary;
use super::{audio_directory, audio_producers, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const BASE: u32 = 0x08000000;
const ROM_BYTES: u32 = 0x01000000;
pub const CLOCK_OFFSET: u32 = 0x370d04;
pub const COMMANDS_OFFSET: u32 = 0x370a90;
/// Only selectors8/9 are produced. The other extended dispatch slots remain unowned.
pub const EXTENDED_OFFSET: u32 = 0x370d58;
pub const BYTECODE_BYTES: usize = 177047;
pub const SOURCE_BYTES: usize = 177248;
pub const INSTRUCTIONS: usize = 105253;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub id: u16,
    pub directory_header: u16,
    pub track_index: u8,
    pub entry: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Code {
    Wait {
        clock: u8,
    },
    End,
    Jump {
        target: u32,
    },
    Pattern {
        target: u32,
    },
    Return,
    Parameter {
        command: u8,
        value: u8,
    },
    Extended {
        selector: u8,
        value: u8,
    },
    EndTie {
        key: Option<u8>,
    },
    Note {
        clock: u8,
        key: Option<u8>,
        velocity: Option<u8>,
        extension: Option<u8>,
    },
}
impl Code {
    pub fn command(&self) -> Option<u8> {
        Some(match self {
            Self::Wait { clock } => 0x80u8.checked_add(*clock)?,
            Self::End => 0xb1,
            Self::Jump { .. } => 0xb2,
            Self::Pattern { .. } => 0xb3,
            Self::Return => 0xb4,
            Self::Parameter { command, .. } => *command,
            Self::Extended { .. } => 0xcd,
            Self::EndTie { .. } => 0xce,
            Self::Note { clock, .. } => 0xcfu8.checked_add(*clock)?,
        })
    }
    pub fn operand_len(&self) -> usize {
        match self {
            Self::Wait { .. } | Self::End | Self::Return => 0,
            Self::Jump { .. } | Self::Pattern { .. } => 4,
            Self::Parameter { .. } => 1,
            Self::Extended { .. } => 2,
            Self::EndTie { key } => usize::from(key.is_some()),
            Self::Note {
                key,
                velocity,
                extension,
                ..
            } => {
                usize::from(key.is_some())
                    + usize::from(velocity.is_some())
                    + usize::from(extension.is_some())
            }
        }
    }
    fn valid(&self) -> bool {
        match self {
            Self::Wait { clock }
            | Self::Note {
                clock,
                key: None,
                velocity: None,
                extension: None,
            } => *clock <= 48,
            Self::Note {
                clock,
                key,
                velocity,
                extension,
            } => {
                *clock <= 48
                    && (!velocity.is_some() || key.is_some())
                    && (!extension.is_some() || velocity.is_some())
                    && [key, velocity, extension]
                        .into_iter()
                        .all(|v| v.is_none_or(|n| n < 128))
            }
            Self::Parameter { command, .. } => matches!(*command, 0xbb..=0xc5 | 0xc8),
            Self::Extended { selector, .. } => matches!(*selector, 8 | 9),
            Self::EndTie { key } => key.is_none_or(|n| n < 128),
            _ => true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    pub id: u32,
    pub source_offset: u32,
    pub explicit: bool,
    pub code: Code,
}
impl Instruction {
    pub fn source_len(&self) -> usize {
        usize::from(self.explicit) + self.code.operand_len()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverTables {
    pub clock_source_offset: u32,
    pub clocks: Vec<u8>,
    pub command_source_offset: u32,
    pub commands: Vec<u32>,
    pub extended_source_offset: u32,
    pub extended8_9: Vec<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioSequences {
    pub tracks: Vec<Track>,
    pub instructions: Vec<Instruction>,
    pub driver: DriverTables,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProducerState {
    instruction: u32,
    running: u8,
    stack: Vec<u32>,
    program: Option<u8>,
    key: u8,
}
impl AudioSequences {
    pub fn instruction(&self, id: u32) -> Option<&Instruction> {
        self.instructions.get(id as usize).filter(|i| i.id == id)
    }
    pub fn byte(&self, id: u32, offset: usize) -> Option<u8> {
        let i = self.instruction(id)?;
        if offset >= i.source_len() {
            return None;
        }
        if i.explicit && offset == 0 {
            return i.code.command();
        }
        let p = offset.checked_sub(usize::from(i.explicit))?;
        Some(match &i.code {
            Code::Jump { target } | Code::Pattern { target } => {
                let at = self.instruction(*target)?.source_offset;
                BASE.checked_add(at)?.to_le_bytes()[p]
            }
            Code::Parameter { value, .. } => *value,
            Code::Extended { selector, value } => {
                if p == 0 {
                    *selector
                } else {
                    *value
                }
            }
            Code::EndTie { key } => key.as_ref().copied()?,
            Code::Note {
                key,
                velocity,
                extension,
                ..
            } => [key, velocity, extension][p].as_ref().copied()?,
            _ => return None,
        })
    }
    pub fn instruction_bytes(&self, id: u32) -> Result<Vec<u8>, String> {
        let i = self.instruction(id).ok_or("Missing M4A instruction ID")?;
        (0..i.source_len())
            .map(|o| {
                self.byte(id, o)
                    .ok_or_else(|| "Invalid M4A instruction field".into())
            })
            .collect()
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        let mut spans: Vec<(u32, Vec<u8>, String)> = Vec::new();
        for i in &self.instructions {
            let bytes = self.instruction_bytes(i.id)?;
            if let Some((_, prior, _)) = spans.last_mut().filter(|(at, prior, _)| {
                u64::from(*at) + prior.len() as u64 == u64::from(i.source_offset)
            }) {
                prior.extend(bytes);
            } else {
                let id = spans.len();
                spans.push((i.source_offset, bytes, format!("audio/sequence/{id}")));
            }
        }
        spans.push((
            self.driver.command_source_offset,
            self.driver
                .commands
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            "audio/driver/commands".into(),
        ));
        spans.push((
            self.driver.clock_source_offset,
            self.driver.clocks.clone(),
            "audio/driver/clocks".into(),
        ));
        spans.push((
            self.driver.extended_source_offset,
            self.driver
                .extended8_9
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            "audio/driver/extended8-9".into(),
        ));
        spans.sort_by_key(|s| s.0);
        Ok(spans)
    }
    pub fn validate(&self) -> Result<(), String> {
        let bad = || {
            "Invalid EUR M4A sequence instruction/track/table shape, ID, link or source extent"
                .to_string()
        };
        let d = &self.driver;
        if self.instructions.len() != INSTRUCTIONS
            || self.tracks.len() != 1610
            || d.clock_source_offset != CLOCK_OFFSET
            || d.command_source_offset != COMMANDS_OFFSET
            || d.extended_source_offset != EXTENDED_OFFSET
            || d.clocks.len() != 49
            || d.commands.len() != 36
            || d.extended8_9.len() != 2
        {
            return Err(bad());
        }
        if d.commands
            .iter()
            .chain(&d.extended8_9)
            .any(|p| p & 1 == 0 || !(BASE..BASE + ROM_BYTES).contains(p))
        {
            return Err(bad());
        }
        let mut previous = 0u64;
        for (id, i) in self.instructions.iter().enumerate() {
            let len = i.source_len();
            let end = u64::from(i.source_offset) + len as u64;
            if i.id as usize != id
                || len == 0
                || u64::from(i.source_offset) < previous
                || end > u64::from(ROM_BYTES)
                || !i.code.valid()
                || (!i.explicit && i.code.command().is_none_or(|n| n < 0xbd))
            {
                return Err(bad());
            }
            previous = end;
            if !i.explicit && matches!(i.code, Code::Parameter { value, .. } if value >= 128) {
                return Err(bad());
            }
            if let Code::Jump { target } | Code::Pattern { target } = i.code {
                if self.instruction(target).is_none() {
                    return Err(bad());
                }
            }
            if !matches!(i.code, Code::End | Code::Jump { .. } | Code::Return)
                && self
                    .instructions
                    .binary_search_by_key(&(end as u32), |i| i.source_offset)
                    .is_err()
            {
                return Err(bad());
            }
        }
        for (id, t) in self.tracks.iter().enumerate() {
            if t.id as usize != id
                || t.directory_header >= 640
                || t.track_index >= 16
                || self.instruction(t.entry).is_none()
            {
                return Err(bad());
            }
        }
        let spans = self.source_spans()?;
        if spans.len() != 808
            || spans.iter().map(|s| s.1.len()).sum::<usize>() != SOURCE_BYTES
            || spans
                .windows(2)
                .any(|s| u64::from(s[0].0) + s[0].1.len() as u64 > u64::from(s[1].0))
        {
            return Err(bad());
        }
        Ok(())
    }
    pub fn validate_directory(&self, d: &audio_directory::AudioDirectory) -> Result<(), String> {
        self.validate()?;
        d.validate()?;
        let roots: Vec<_> = d
            .headers
            .iter()
            .flat_map(|h| {
                h.track_addresses
                    .iter()
                    .enumerate()
                    .map(move |(n, &a)| (h.id, n as u8, a))
            })
            .collect();
        let actual: Vec<_> = self
            .tracks
            .iter()
            .map(|t| {
                (
                    t.directory_header,
                    t.track_index,
                    BASE + self.instruction(t.entry).unwrap().source_offset,
                )
            })
            .collect();
        if actual != roots {
            return Err("M4A sequence real directory root dependency mismatch".into());
        }
        Ok(())
    }
    /// Resolve program/key domains from the typed instructions against both real owners.
    /// This validates source dependencies; guest execution still uses live RAM and its own clock.
    pub fn validate_owners(
        &self,
        directory: &audio_directory::AudioDirectory,
        tones: &super::audio_tones::AudioTones,
    ) -> Result<(), String> {
        self.validate_directory(directory)?;
        tones.validate_directory(directory)?;
        let mut domains: BTreeMap<(u32, u8), BTreeSet<u8>> = BTreeMap::new();
        let mut states = 0usize;
        for track in &self.tracks {
            let bank = directory
                .header(track.directory_header)
                .unwrap()
                .tone_bank_address;
            let mut pending = vec![ProducerState {
                instruction: track.entry,
                running: 0,
                stack: vec![],
                program: None,
                key: 0,
            }];
            let mut seen = BTreeSet::new();
            while let Some(mut state) = pending.pop() {
                if !seen.insert(state.clone()) {
                    continue;
                }
                states += 1;
                if seen.len() > 1_000_000 || states > 8_000_000 {
                    return Err("M4A typed sequence dependency state guard exceeded".into());
                }
                let i = self.instruction(state.instruction).unwrap();
                let command = i.code.command().unwrap();
                if i.explicit {
                    if command >= 0xbd {
                        state.running = command;
                    }
                } else if state.running != command {
                    return Err("M4A inherited command differs from typed producer state".into());
                }
                let next = || -> Result<u32, String> {
                    let at = i.source_offset + i.source_len() as u32;
                    self.instructions
                        .binary_search_by_key(&at, |i| i.source_offset)
                        .map(|n| n as u32)
                        .map_err(|_| "Missing M4A typed producer successor".into())
                };
                state.instruction = match i.code {
                    Code::End => continue,
                    Code::Jump { target } => target,
                    Code::Pattern { target } => {
                        if state.stack.len() >= 3 {
                            continue;
                        }
                        state.stack.push(next()?);
                        target
                    }
                    Code::Return => match state.stack.pop() {
                        Some(n) => n,
                        None => next()?,
                    },
                    Code::Parameter {
                        command: 0xbd,
                        value,
                    } => {
                        state.program = Some(value);
                        domains.entry((bank, value)).or_default();
                        next()?
                    }
                    Code::Note { key, .. } => {
                        if let Some(key) = key {
                            state.key = key;
                        }
                        if let Some(program) = state.program {
                            domains
                                .entry((bank, program))
                                .or_default()
                                .insert(state.key);
                        }
                        next()?
                    }
                    Code::EndTie { key } => {
                        if let Some(key) = key {
                            state.key = key;
                        }
                        next()?
                    }
                    _ => next()?,
                };
                pending.push(state);
            }
        }
        let expected: BTreeMap<_, BTreeSet<_>> = tones
            .banks
            .iter()
            .flat_map(|b| {
                b.programs.iter().map(move |p| {
                    (
                        (b.source_address, p.selector),
                        p.note_keys.iter().copied().collect(),
                    )
                })
            })
            .collect();
        if states != 147424 || domains != expected {
            return Err("M4A sequence differs from actual tone bank program/key domains".into());
        }
        Ok(())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        let expected = Catalog::from_rom(rom)?;
        if self != &expected.decoded {
            return Err("Decoded M4A sequence differs from authenticated source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: AudioSequences,
    pub sources: Vec<SourceRange>,
    pub producer: ProducerSummary,
    pub consumed_bytecode: Vec<(u32, u32)>,
}
fn word(rom: &[u8], at: u32) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        rom.get(at as usize..at as usize + 4)
            .ok_or("Truncated M4A sequence table/pointer")?
            .try_into()
            .unwrap(),
    ))
}
fn driver(rom: &[u8]) -> Result<DriverTables, String> {
    for (pc, at, want) in [
        (0x1485fa, 0x14874c, BASE + CLOCK_OFFSET),
        (0x1487e4, 0x1489c8, BASE + CLOCK_OFFSET),
        (0x1482f4, 0x148320, BASE + COMMANDS_OFFSET),
        (0x149e92, 0x149ea4, 0x08370d38),
    ] {
        let op = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
        if op & 0xf800 != 0x4800
            || ((pc + 4) & !3) + (op as usize & 255) * 4 != at
            || word(rom, at as u32)? != want
        {
            return Err("M4A sequence driver literal mismatch".into());
        }
    }
    for (at, want) in [
        (0x1482f2, 0x2124),
        (0x1485ca, 0x1c08),
        (0x1485cc, 0x38cf),
        (0x1485fc, 0x3980),
        (0x1487e8, 0x7800),
        (0x149e94, 0x009b),
    ] {
        if u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) != want {
            return Err("M4A sequence original reader contract mismatch".into());
        }
    }
    Ok(DriverTables {
        clock_source_offset: CLOCK_OFFSET,
        clocks: rom[CLOCK_OFFSET as usize..CLOCK_OFFSET as usize + 49].to_vec(),
        command_source_offset: COMMANDS_OFFSET,
        commands: (0..36)
            .map(|i| word(rom, COMMANDS_OFFSET + 4 * i))
            .collect::<Result<_, _>>()?,
        extended_source_offset: EXTENDED_OFFSET,
        extended8_9: (0..2)
            .map(|i| word(rom, EXTENDED_OFFSET + 4 * i))
            .collect::<Result<_, _>>()?,
    })
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let directory = audio_directory::Catalog::from_rom(rom)?.decoded;
        let graph = audio_producers::analyze(rom, &directory)?;
        let ids: BTreeMap<_, _> = graph
            .instructions
            .keys()
            .enumerate()
            .map(|(id, &pc)| (pc, id as u32))
            .collect();
        let mut instructions = Vec::new();
        for (&pc, observed) in &graph.instructions {
            let p = pc as usize + usize::from(observed.explicit);
            let end = observed.end as usize;
            let operands = &rom[p..end];
            let byte = |n: usize| operands.get(n).copied();
            let target = || -> Result<u32, String> {
                let at = word(rom, p as u32)?
                    .checked_sub(BASE)
                    .ok_or("Non-ROM M4A target")?;
                ids.get(&at)
                    .copied()
                    .ok_or_else(|| "M4A target has no canonical instruction owner".into())
            };
            let code = match observed.command {
                0x80..=0xb0 => Code::Wait {
                    clock: observed.command - 0x80,
                },
                0xb1 => Code::End,
                0xb2 => Code::Jump { target: target()? },
                0xb3 => Code::Pattern { target: target()? },
                0xb4 => Code::Return,
                0xbb..=0xc5 | 0xc8 => Code::Parameter {
                    command: observed.command,
                    value: byte(0).ok_or("Missing M4A parameter")?,
                },
                0xcd => Code::Extended {
                    selector: byte(0).ok_or("Missing M4A extension")?,
                    value: byte(1).ok_or("Missing M4A extension operand")?,
                },
                0xce => Code::EndTie { key: byte(0) },
                0xcf..=0xff => Code::Note {
                    clock: observed.command - 0xcf,
                    key: byte(0),
                    velocity: byte(1),
                    extension: byte(2),
                },
                _ => return Err("Unproved M4A sequence instruction kind".into()),
            };
            let i = Instruction {
                id: ids[&pc],
                source_offset: pc,
                explicit: observed.explicit,
                code,
            };
            if i.source_len() != end - pc as usize {
                return Err("M4A typed instruction consumed extent mismatch".into());
            }
            instructions.push(i);
        }
        let mut tracks = Vec::new();
        for h in &directory.headers {
            for (n, &a) in h.track_addresses.iter().enumerate() {
                tracks.push(Track {
                    id: tracks.len() as u16,
                    directory_header: h.id,
                    track_index: n as u8,
                    entry: *ids
                        .get(&(a - BASE))
                        .ok_or("M4A track has no canonical sequence owner")?,
                });
            }
        }
        let decoded = AudioSequences {
            tracks,
            instructions,
            driver: driver(rom)?,
        };
        decoded.validate_directory(&directory)?;
        let spans = decoded.source_spans()?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in spans {
            if rom.get(offset as usize..offset as usize + bytes.len()) != Some(bytes.as_slice()) {
                return Err("M4A sequence source roundtrip mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        Ok(Self {
            decoded,
            sources,
            producer: graph.summary,
            consumed_bytecode: graph.consumed,
        })
    }
}
