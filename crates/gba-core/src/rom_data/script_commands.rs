//! Bounded EUR six-byte script command directory. Guest control flow stays live.
use super::{palette_effect_dispatch::ThumbEntry, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3acebc;
pub const COMMAND_COUNT: usize = 115;
pub const SOURCE_BYTES: usize = COMMAND_COUNT * 6;

/// Original unsigned-byte opcode arithmetic, without a new source-domain cap.
pub fn command_source_offset(opcode: u8) -> u32 {
    SOURCE_OFFSET + u32::from(opcode) * 6
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub opcode: u8,
    pub advance: u16,
    pub callback: ThumbEntry,
}
impl Command {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0..=1 => Some(self.advance.to_le_bytes()[offset]),
            2..=5 => Some(self.callback.guest_word()?.to_le_bytes()[offset - 2]),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptCommands {
    pub source_offset: u32,
    pub commands: Vec<Command>,
}
impl ScriptCommands {
    pub fn byte_at(&self, source_offset: u32) -> Option<u8> {
        let relative = source_offset.checked_sub(self.source_offset)? as usize;
        self.commands.get(relative / 6)?.byte(relative % 6)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.commands.len() != COMMAND_COUNT {
            return Err("Script command source/count mismatch".into());
        }
        for (opcode, command) in self.commands.iter().enumerate() {
            if usize::from(command.opcode) != opcode || command.callback.guest_word().is_none() {
                return Err("Script command opcode/Thumb callback mismatch".into());
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        let bytes = (0..SOURCE_BYTES)
            .map(|i| self.byte_at(SOURCE_OFFSET + i as u32).unwrap())
            .collect();
        Ok(vec![(
            SOURCE_OFFSET,
            bytes,
            "engine/script-command-directory/opcodes-0-114".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Script commands differ from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ScriptCommands,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Script commands require exact European source ROM".into());
        }
        verify_contract(rom)?;
        let mut commands = Vec::with_capacity(COMMAND_COUNT);
        for opcode in 0..COMMAND_COUNT {
            let at = SOURCE_OFFSET as usize + opcode * 6;
            let word = u32::from_le_bytes(rom[at + 2..at + 6].try_into().unwrap());
            if word & 1 == 0 || !(0x08000001..0x09000000).contains(&word) {
                return Err("Script command callback is not a ROM Thumb entry".into());
            }
            commands.push(Command {
                opcode: opcode as u8,
                advance: u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()),
                callback: ThumbEntry {
                    rom_offset: (word & !1) - 0x08000000,
                },
            });
        }
        let decoded = ScriptCommands {
            source_offset: SOURCE_OFFSET,
            commands,
        };
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Script command source roundtrip mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        Ok(Self { decoded, sources })
    }
}

fn verify_contract(rom: &[u8]) -> Result<(), String> {
    for &(offset, value) in CONTRACT_HALFWORDS {
        if u16::from_le_bytes(
            rom[offset as usize..offset as usize + 2]
                .try_into()
                .unwrap(),
        ) != value
        {
            return Err("Script command original reader contract mismatch".into());
        }
    }
    for &(offset, value) in CONTRACT_WORDS {
        if u32::from_le_bytes(
            rom[offset as usize..offset as usize + 4]
                .try_into()
                .unwrap(),
        ) != value
        {
            return Err("Script command original root/end contract mismatch".into());
        }
    }
    Ok(())
}

// Original-reader and independent-successor fingerprints from the source inventory.
const CONTRACT_HALFWORDS: &[(u32, u16)] = &[
    (0x1263b2, 0x4b04),
    (0x1263b4, 0x78f2),
    (0x1263b6, 0x3a01),
    (0x1263b8, 0x18d2),
    (0x1263ba, 0x7812),
    (0x126454, 0x4b04),
    (0x126456, 0x7992),
    (0x126458, 0x3a01),
    (0x12645a, 0x18d2),
    (0x12645c, 0x7812),
    (0x1265cc, 0xb570),
    (0x1265ce, 0x1c05),
    (0x1265d0, 0x4e13),
    (0x1265d2, 0x2100),
    (0x1265d4, 0x5e68),
    (0x1265d6, 0x68e9),
    (0x1265d8, 0x1809),
    (0x1265da, 0x7808),
    (0x1265dc, 0x0044),
    (0x1265de, 0x1824),
    (0x1265e0, 0x0064),
    (0x1265e2, 0x19a4),
    (0x1265e4, 0x78a2),
    (0x1265e6, 0x78e0),
    (0x1265e8, 0x0200),
    (0x1265ea, 0x4310),
    (0x1265ec, 0x7922),
    (0x1265ee, 0x0412),
    (0x1265f0, 0x4302),
    (0x1265f2, 0x7963),
    (0x1265f4, 0x061b),
    (0x1265f6, 0x4313),
    (0x1265f8, 0x1c28),
    (0x1265fa, 0xf01f),
    (0x1265fc, 0xfb0b),
    (0x1265fe, 0x1c02),
    (0x126600, 0x7821),
    (0x126602, 0x7860),
    (0x126604, 0x0200),
    (0x126606, 0x4308),
    (0x126608, 0x8829),
    (0x12660a, 0x1840),
    (0x12660c, 0x8028),
    (0x12660e, 0x2088),
    (0x126610, 0x0040),
    (0x126612, 0x4282),
    (0x126614, 0xd0dd),
    (0x126616, 0x1c10),
    (0x126618, 0xbc70),
    (0x12661a, 0xbc02),
    (0x12661c, 0x4708),
    (0x126624, 0xb510),
    (0x126626, 0x1c0c),
    (0x126628, 0x4b0c),
    (0x12662a, 0x7822),
    (0x12662c, 0x0051),
    (0x12662e, 0x1889),
    (0x126630, 0x0049),
    (0x126632, 0x18c9),
    (0x126634, 0x780a),
    (0x126636, 0x7849),
    (0x126638, 0x0209),
    (0x12663a, 0x4311),
    (0x12663c, 0x78a3),
    (0x12663e, 0x78e2),
    (0x126640, 0x0212),
    (0x126642, 0x4313),
    (0x126644, 0x18c9),
    (0x126646, 0x0409),
    (0x126648, 0x0c09),
    (0x12664a, 0x7862),
    (0x12664c, 0xf7ff),
    (0x12664e, 0xfc44),
    (0x126650, 0x2088),
    (0x126652, 0x0040),
    (0x126654, 0xbc10),
    (0x126656, 0xbc02),
    (0x126658, 0x4708),
    (0x126660, 0xb510),
    (0x126662, 0x1c0c),
    (0x126664, 0x4b0c),
    (0x126666, 0x7822),
    (0x126668, 0x0051),
    (0x12666a, 0x1889),
    (0x12666c, 0x0049),
    (0x12666e, 0x18c9),
    (0x126670, 0x780a),
    (0x126672, 0x7849),
    (0x126674, 0x0209),
    (0x126676, 0x4311),
    (0x126678, 0x7863),
    (0x12667a, 0x78a2),
    (0x12667c, 0x0212),
    (0x12667e, 0x4313),
    (0x126680, 0x18c9),
    (0x126682, 0x0409),
    (0x126684, 0x0c09),
    (0x126686, 0x2280),
    (0x126688, 0xf7ff),
    (0x12668a, 0xfc26),
    (0x12668c, 0x2088),
    (0x12668e, 0x0040),
    (0x126690, 0xbc10),
    (0x126692, 0xbc02),
    (0x126694, 0x4708),
    (0x145c14, 0x4718),
];
const CONTRACT_WORDS: &[(u32, u32)] = &[
    (0x126620, 0x083acebc),
    (0x12665c, 0x083acebc),
    (0x126698, 0x083acebc),
    (0x1263c4, 0x083ad16e),
    (0x126468, 0x083ad16e),
    (0x128b40, 0x083ad174),
];
