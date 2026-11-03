//! Separate EUR u16-relative auxiliary scripts. Predicates and dispatch stay guest work.
use super::{palette_effect_dispatch::ThumbEntry, script_commands::Command, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SOURCE_OFFSET: u32 = 0xd69bc4;
pub const SOURCE_END: u32 = 0xd6abc6;
pub const DIRECTORY_OFFSET: u32 = 0x370f94;
pub const DIRECTORY_BYTES: usize = 102;
pub const SOURCE_BYTES: usize = 4098;
const SLOTS: [usize; 8] = [63, 63, 1, 63, 63, 63, 63, 63];

/// Original byte arithmetic, not a bound on live action IDs.
pub fn bank_index_source_offset(raw_action: u32) -> u32 {
    SOURCE_OFFSET + u32::from(raw_action as u8) * 2
}
pub fn slot_index_source_offset(bank: u32, raw_slot: u32) -> u32 {
    bank.wrapping_add(u32::from(raw_slot as u8) * 2)
}
pub fn command_source_offset(opcode: u8) -> u32 {
    DIRECTORY_OFFSET + u32::from(opcode) * 6
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuxiliaryCommands {
    pub source_offset: u32,
    pub commands: Vec<Command>,
}
impl AuxiliaryCommands {
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        let relative = at.checked_sub(self.source_offset)? as usize;
        self.commands.get(relative / 6)?.byte(relative % 6)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != DIRECTORY_OFFSET || self.commands.len() != 17 {
            return Err("Auxiliary command source/count mismatch".into());
        }
        let mut callbacks = BTreeSet::new();
        for (id, command) in self.commands.iter().enumerate() {
            if usize::from(command.opcode) != id
                || command.advance == 0
                || command.callback.guest_word().is_none()
                || !callbacks.insert(command.callback.rom_offset)
            {
                return Err("Auxiliary command ID/advance/Thumb callback mismatch".into());
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        Ok(vec![(
            DIRECTORY_OFFSET,
            (0..DIRECTORY_BYTES)
                .map(|n| self.byte_at(DIRECTORY_OFFSET + n as u32).unwrap())
                .collect(),
            "engine/auxiliary-script-command-directory/advance-callback".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.commands {
            return Err("Auxiliary command provenance mismatch".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    pub source_offset: u32,
    pub command: u8,
    /// Exact storage parameters; no replacement predicates or host interpreter.
    pub parameters: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub id: u16,
    pub source_offset: u32,
    pub instructions: Vec<Instruction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub program_id: u16,
    /// A slot can enter an interior instruction, without duplicating its storage.
    pub instruction: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub id: u8,
    pub source_offset: u32,
    pub entries: Vec<Entry>,
    pub programs: Vec<Program>,
    pub alignment_after: Vec<u8>,
}
impl Bank {
    pub fn entry_offset(&self, entry: &Entry) -> Option<u32> {
        self.programs
            .iter()
            .find(|p| p.id == entry.program_id)?
            .instructions
            .get(usize::from(entry.instruction))
            .map(|i| i.source_offset)
    }
    fn instruction_at(&self, at: u32) -> Option<&Instruction> {
        let program = self.programs.get(
            self.programs
                .partition_point(|p| p.source_offset <= at)
                .checked_sub(1)?,
        )?;
        program.instructions.get(
            program
                .instructions
                .partition_point(|i| i.source_offset <= at)
                .checked_sub(1)?,
        )
    }
    fn program_end(&self) -> Option<u32> {
        let last = self.programs.last()?.instructions.last()?;
        last.source_offset
            .checked_add(1 + last.parameters.len() as u32)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuxiliaryArchive {
    pub source_offset: u32,
    pub banks: Vec<Bank>,
}
impl AuxiliaryArchive {
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        let relative = at.checked_sub(self.source_offset)? as usize;
        if relative < self.banks.len().checked_mul(2)? {
            let rel = self.banks[relative / 2]
                .source_offset
                .checked_sub(self.source_offset)?;
            return Some(u16::try_from(rel).ok()?.to_le_bytes()[relative % 2]);
        }
        let bank = self.banks.get(
            self.banks
                .partition_point(|b| b.source_offset <= at)
                .checked_sub(1)?,
        )?;
        let relative = (at - bank.source_offset) as usize;
        if let Some(entry) = bank.entries.get(relative / 2) {
            let rel = bank.entry_offset(entry)?.checked_sub(bank.source_offset)?;
            return Some(u16::try_from(rel).ok()?.to_le_bytes()[relative % 2]);
        }
        if let Some(ins) = bank.instruction_at(at) {
            let relative = at.checked_sub(ins.source_offset)? as usize;
            if relative == 0 {
                return Some(ins.command);
            }
            if let Some(&value) = ins.parameters.get(relative - 1) {
                return Some(value);
            }
        }
        bank.alignment_after
            .get(at.checked_sub(bank.program_end()?)? as usize)
            .copied()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.banks.len() != SLOTS.len() {
            return Err("Auxiliary archive source/bank count mismatch".into());
        }
        let mut next = SOURCE_OFFSET + 16;
        let mut program_id = 0;
        let (mut instructions, mut parameters, mut padding) = (0, 0, 0);
        for (id, bank) in self.banks.iter().enumerate() {
            if usize::from(bank.id) != id
                || bank.source_offset != next
                || bank.entries.len() != SLOTS[id]
                || bank.programs.is_empty()
            {
                return Err("Auxiliary bank ID/order/entries mismatch".into());
            }
            next = next
                .checked_add(bank.entries.len() as u32 * 2)
                .ok_or("Auxiliary index overflow")?;
            for program in &bank.programs {
                if program.id != program_id
                    || program.source_offset != next
                    || program.instructions.is_empty()
                {
                    return Err("Auxiliary physical program ID/order mismatch".into());
                }
                program_id += 1;
                for (n, ins) in program.instructions.iter().enumerate() {
                    if ins.source_offset != next
                        || ins.command >= 17
                        || (ins.command == 0) != (n + 1 == program.instructions.len())
                        || (ins.command == 0 && !ins.parameters.is_empty())
                    {
                        return Err("Auxiliary instruction order/opcode/terminator mismatch".into());
                    }
                    next = next
                        .checked_add(
                            1 + u32::try_from(ins.parameters.len())
                                .map_err(|_| "Auxiliary parameter overflow")?,
                        )
                        .ok_or("Auxiliary instruction overflow")?;
                    if next > SOURCE_END {
                        return Err("Auxiliary instruction outside declared projection".into());
                    }
                    instructions += 1;
                    parameters += ins.parameters.len();
                }
            }
            for entry in &bank.entries {
                if bank.entry_offset(entry).is_none() {
                    return Err("Auxiliary slot is not a same-bank instruction entry".into());
                }
            }
            let align = if id + 1 == SLOTS.len() {
                0
            } else {
                (4 - next % 4) % 4
            };
            if bank.alignment_after.len() != align as usize
                || bank.alignment_after.iter().any(|&v| v != 255)
            {
                return Err("Auxiliary internal alignment mismatch".into());
            }
            next += align;
            padding += align;
        }
        if next != SOURCE_END
            || program_id != 133
            || instructions != 913
            || parameters != 2275
            || padding != 10
        {
            return Err("Auxiliary physical source coverage mismatch".into());
        }
        Ok(())
    }
    pub fn validate_dependencies(&self, commands: &AuxiliaryCommands) -> Result<(), String> {
        self.validate()?;
        commands.validate()?;
        if commands
            .commands
            .iter()
            .map(|c| c.callback.rom_offset)
            .ne(CALLBACK_OFFSETS.iter().copied())
        {
            return Err("Auxiliary callback dependency mismatch".into());
        }
        for bank in &self.banks {
            for ins in bank.programs.iter().flat_map(|p| &p.instructions) {
                if ins.parameters.len() + 1
                    != usize::from(commands.commands[usize::from(ins.command)].advance)
                {
                    return Err("Auxiliary command advance dependency mismatch".into());
                }
                if let Some(k) = relative_parameter_offset(ins.command) {
                    let bytes = ins
                        .parameters
                        .get(k..k + 2)
                        .ok_or("Auxiliary relative operand missing")?;
                    let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
                    let target = ins
                        .source_offset
                        .wrapping_add(1 + ins.parameters.len() as u32)
                        .wrapping_add(i32::from(raw as i16) as u32);
                    if bank.instruction_at(target).map(|i| i.source_offset) != Some(target) {
                        return Err(
                            "Auxiliary relative branch is not a same-bank instruction".into()
                        );
                    }
                }
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE_OFFSET,
            (SOURCE_OFFSET..SOURCE_END)
                .map(|at| self.byte_at(at).ok_or("Auxiliary source gap"))
                .collect::<Result<Vec<_>, _>>()?,
            "engine/auxiliary-script-archive/indices-opcodes-parameters-internal-alignment".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Auxiliary archive provenance mismatch".into());
        }
        Ok(())
    }
}

/// Storage location of a relative operand, not an implementation of its predicate.
pub fn relative_parameter_offset(opcode: u8) -> Option<usize> {
    match opcode {
        1 | 14..=16 => Some(0),
        4 | 7..=9 => Some(2),
        6 | 10..=13 => Some(1),
        _ => None,
    }
}

pub struct Catalog {
    pub commands: AuxiliaryCommands,
    pub decoded: AuxiliaryArchive,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Auxiliary scripts require exact European source ROM".into());
        }
        verify_contract(rom)?;
        let commands = AuxiliaryCommands {
            source_offset: DIRECTORY_OFFSET,
            commands: (0..17)
                .map(|opcode| {
                    let at = DIRECTORY_OFFSET as usize + opcode * 6;
                    let word = u32::from_le_bytes(rom[at + 2..at + 6].try_into().unwrap());
                    Command {
                        opcode: opcode as u8,
                        advance: u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()),
                        callback: ThumbEntry {
                            rom_offset: (word & !1).wrapping_sub(0x08000000),
                        },
                    }
                })
                .collect(),
        };
        commands.validate()?;
        let half =
            |at: u32| u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
        let offsets: Vec<_> = (0..8)
            .map(|i| SOURCE_OFFSET + u32::from(half(SOURCE_OFFSET + i * 2)))
            .collect();
        let mut banks = Vec::new();
        let mut program_id = 0;
        for id in 0..8 {
            let at = offsets[id];
            let upper = offsets.get(id + 1).copied().unwrap_or(SOURCE_END);
            let mut end = upper;
            if id + 1 < 8 {
                while end > at && rom[end as usize - 1] == 255 {
                    end -= 1;
                }
            }
            let mut programs = Vec::new();
            let mut p = at + SLOTS[id] as u32 * 2;
            while p < end {
                let start = p;
                let mut instructions = Vec::new();
                loop {
                    let op = rom[p as usize];
                    let command = commands
                        .commands
                        .get(usize::from(op))
                        .ok_or("Auxiliary source opcode outside directory")?;
                    let next = p
                        .checked_add(u32::from(command.advance))
                        .ok_or("Auxiliary source instruction overflow")?;
                    if next > end {
                        return Err("Auxiliary source instruction exceeds bank".into());
                    }
                    instructions.push(Instruction {
                        source_offset: p,
                        command: op,
                        parameters: rom[p as usize + 1..next as usize].to_vec(),
                    });
                    p = next;
                    if op == 0 {
                        break;
                    }
                }
                programs.push(Program {
                    id: program_id,
                    source_offset: start,
                    instructions,
                });
                program_id += 1;
            }
            let mut entries = Vec::new();
            for slot in 0..SLOTS[id] {
                let target = at + u32::from(half(at + slot as u32 * 2));
                let (program, instruction) = programs
                    .iter()
                    .find_map(|program| {
                        program
                            .instructions
                            .iter()
                            .position(|i| i.source_offset == target)
                            .map(|instruction| (program.id, instruction))
                    })
                    .ok_or("Auxiliary source slot is not an instruction")?;
                entries.push(Entry {
                    program_id: program,
                    instruction: instruction as u16,
                });
            }
            banks.push(Bank {
                id: id as u8,
                source_offset: at,
                entries,
                programs,
                alignment_after: rom[end as usize..upper as usize].to_vec(),
            });
        }
        let decoded = AuxiliaryArchive {
            source_offset: SOURCE_OFFSET,
            banks,
        };
        decoded.validate_dependencies(&commands)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded
            .source_spans()?
            .into_iter()
            .chain(commands.source_spans()?)
        {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Auxiliary script source roundtrip mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        Ok(Self {
            commands,
            decoded,
            sources,
        })
    }
}

fn verify_contract(rom: &[u8]) -> Result<(), String> {
    for &(at, value) in CONTRACT_HALFWORDS {
        if u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap()) != value {
            return Err("Auxiliary original-reader contract mismatch".into());
        }
    }
    for &(at, value) in CONTRACT_WORDS {
        if u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap()) != value {
            return Err("Auxiliary original-root contract mismatch".into());
        }
    }
    Ok(())
}

// Original consumer/root fingerprints, never source data or a replacement program.
const CONTRACT_HALFWORDS: &[(u32, u16)] = &[
    (0xa158, 0xb510),
    (0xa15a, 0x1c04),
    (0xa15c, 0xf000),
    (0xa15e, 0xf820),
    (0xa160, 0x2002),
    (0xa162, 0x4060),
    (0xa164, 0x4241),
    (0xa166, 0x4301),
    (0xa168, 0x0fc9),
    (0xa16a, 0x1c20),
    (0xa16c, 0xf000),
    (0xa16e, 0xf81a),
    (0xa170, 0x1c01),
    (0xa172, 0x1f60),
    (0xa174, 0x2801),
    (0xa176, 0xd901),
    (0xa178, 0x2c03),
    (0xa17a, 0xd10d),
    (0xa17c, 0x4248),
    (0xa17e, 0x4308),
    (0xa180, 0x0fc0),
    (0xa182, 0xf7ff),
    (0xa184, 0xff01),
    (0xa186, 0x0600),
    (0xa188, 0x0e01),
    (0xa18a, 0x2902),
    (0xa18c, 0xd104),
    (0xa18e, 0x2007),
    (0xa190, 0x2101),
    (0xa192, 0xf000),
    (0xa194, 0xf807),
    (0xa196, 0xe000),
    (0xa198, 0x1c08),
    (0xa19a, 0xbc10),
    (0xa19c, 0xbc02),
    (0xa19e, 0x4708),
    (0xa1a0, 0x4770),
    (0xa1a2, 0x0000),
    (0xa1a4, 0xb500),
    (0xa1a6, 0xb082),
    (0xa1a8, 0x0609),
    (0xa1aa, 0x0600),
    (0xa1ac, 0x0e00),
    (0xa1ae, 0x4a08),
    (0xa1b0, 0x9b00),
    (0xa1b2, 0x4013),
    (0xa1b4, 0x4303),
    (0xa1b6, 0x9300),
    (0xa1b8, 0x2900),
    (0xa1ba, 0xd011),
    (0xa1bc, 0x4805),
    (0xa1be, 0x4906),
    (0xa1c0, 0x1840),
    (0xa1c2, 0x7801),
    (0xa1c4, 0x0209),
    (0xa1c6, 0x4805),
    (0xa1c8, 0x4018),
    (0xa1ca, 0x4308),
    (0xa1cc, 0x9000),
    (0xa1ce, 0xe00a),
    (0xa1d0, 0xff00),
    (0xa1d2, 0xffff),
    (0xa1d4, 0x0000),
    (0xa1d6, 0x0200),
    (0xa1d8, 0x2191),
    (0xa1da, 0x0000),
    (0xa1dc, 0x00ff),
    (0xa1de, 0xffff),
    (0xa1e0, 0x4804),
    (0xa1e2, 0x4003),
    (0xa1e4, 0x9300),
    (0xa1e6, 0x4668),
    (0xa1e8, 0xf000),
    (0xa1ea, 0xf806),
    (0xa1ec, 0xb002),
    (0xa1ee, 0xbc02),
    (0xa1f0, 0x4708),
    (0xa1f2, 0x0000),
    (0xa1f4, 0x00ff),
    (0xa1f6, 0xffff),
    (0xa1f8, 0xb510),
    (0xa1fa, 0x2400),
    (0xa1fc, 0x4b0d),
    (0xa1fe, 0x7801),
    (0xa200, 0x0049),
    (0xa202, 0x18c9),
    (0xa204, 0x880a),
    (0xa206, 0x18d2),
    (0xa208, 0x7841),
    (0xa20a, 0x0049),
    (0xa20c, 0x1889),
    (0xa20e, 0x8809),
    (0xa210, 0x1852),
    (0xa212, 0x6042),
    (0xa214, 0x8044),
    (0xa216, 0xf000),
    (0xa218, 0xf813),
    (0xa21a, 0x0600),
    (0xa21c, 0x0e00),
    (0xa21e, 0x2801),
    (0xa220, 0xd103),
    (0xa222, 0x4805),
    (0xa224, 0x4905),
    (0xa226, 0x1840),
    (0xa228, 0x7804),
    (0xa22a, 0x1c20),
    (0xa22c, 0xbc10),
    (0xa22e, 0xbc02),
    (0xa230, 0x4708),
    (0xa232, 0x0000),
    (0xa234, 0x9bc4),
    (0xa236, 0x08d6),
    (0xa238, 0x0000),
    (0xa23a, 0x0200),
    (0xa23c, 0x219c),
    (0xa23e, 0x0000),
    (0xa240, 0xb570),
    (0xa242, 0x1c05),
    (0xa244, 0x4e10),
    (0xa246, 0x2002),
    (0xa248, 0x5e29),
    (0xa24a, 0x6868),
    (0xa24c, 0x1843),
    (0xa24e, 0x7818),
    (0xa250, 0x2800),
    (0xa252, 0xd01b),
    (0xa254, 0x0044),
    (0xa256, 0x1824),
    (0xa258, 0x0064),
    (0xa25a, 0x19a4),
    (0xa25c, 0x78a1),
    (0xa25e, 0x78e0),
    (0xa260, 0x0200),
    (0xa262, 0x4308),
    (0xa264, 0x7921),
    (0xa266, 0x0409),
    (0xa268, 0x4301),
    (0xa26a, 0x7962),
    (0xa26c, 0x0612),
    (0xa26e, 0x430a),
    (0xa270, 0x1c28),
    (0xa272, 0x1c19),
    (0xa274, 0xf13b),
    (0xa276, 0xfccc),
    (0xa278, 0x7821),
    (0xa27a, 0x7860),
    (0xa27c, 0x0200),
    (0xa27e, 0x4308),
    (0xa280, 0x8869),
    (0xa282, 0x1840),
    (0xa284, 0x8068),
    (0xa286, 0xe7de),
    (0xa288, 0x0f94),
    (0xa28a, 0x0837),
    (0xa28c, 0x2001),
    (0xa28e, 0xbc70),
    (0xa290, 0xbc02),
    (0xa292, 0x4708),
    (0xa298, 0x784a),
    (0xa29a, 0x7889),
    (0xa29c, 0x0209),
    (0xa29e, 0x430a),
    (0xa2a0, 0x8841),
    (0xa2a2, 0x1852),
    (0xa2a4, 0x8042),
    (0xa2a6, 0x4770),
    (0xa2a8, 0xb500),
    (0xa2aa, 0x7848),
    (0xa2ac, 0x788a),
    (0xa2ae, 0x0212),
    (0xa2b0, 0x4310),
    (0xa2b2, 0x78c9),
    (0xa2b4, 0xf0c2),
    (0xa2b6, 0xfe0a),
    (0xa2b8, 0xbc01),
    (0xa2ba, 0x4700),
    (0xa2bc, 0xb500),
    (0xa2be, 0x7848),
    (0xa2c0, 0x788a),
    (0xa2c2, 0x0212),
    (0xa2c4, 0x4310),
    (0xa2c6, 0x78c9),
    (0xa2c8, 0xf0c2),
    (0xa2ca, 0xfe26),
    (0xa2cc, 0xbc01),
    (0xa2ce, 0x4700),
    (0xa2d0, 0xb530),
    (0xa2d2, 0x1c05),
    (0xa2d4, 0x1c0c),
    (0xa2d6, 0x7860),
    (0xa2d8, 0x78a1),
    (0xa2da, 0x0209),
    (0xa2dc, 0x4308),
    (0xa2de, 0xf0c2),
    (0xa2e0, 0xfddb),
    (0xa2e2, 0x0600),
    (0xa2e4, 0x2800),
    (0xa2e6, 0xd006),
    (0xa2e8, 0x78e1),
    (0xa2ea, 0x7920),
    (0xa2ec, 0x0200),
    (0xa2ee, 0x4301),
    (0xa2f0, 0x8868),
    (0xa2f2, 0x1809),
    (0xa2f4, 0x8069),
    (0xa2f6, 0xbc30),
    (0xa2f8, 0xbc01),
    (0xa2fa, 0x4700),
    (0xa2fc, 0xb510),
    (0xa2fe, 0x4b09),
    (0xa300, 0x4809),
    (0xa302, 0x181c),
    (0xa304, 0x7822),
    (0xa306, 0x1c50),
    (0xa308, 0x7020),
    (0xa30a, 0x0612),
    (0xa30c, 0x0dd2),
    (0xa30e, 0x4807),
    (0xa310, 0x181b),
    (0xa312, 0x18d2),
    (0xa314, 0x784b),
    (0xa316, 0x7888),
    (0xa318, 0x0200),
    (0xa31a, 0x4303),
    (0xa31c, 0x8013),
    (0xa31e, 0xbc10),
    (0xa320, 0xbc01),
    (0xa322, 0x4700),
    (0xa324, 0x0000),
    (0xa326, 0x0200),
    (0xa328, 0x219c),
    (0xa32a, 0x0000),
    (0xa32c, 0x2194),
    (0xa32e, 0x0000),
    (0xa330, 0xb570),
    (0xa332, 0x1c06),
    (0xa334, 0x1c0d),
    (0xa336, 0x7868),
    (0xa338, 0xf11e),
    (0xa33a, 0xfeb0),
    (0xa33c, 0x1c04),
    (0xa33e, 0x2c00),
    (0xa340, 0xd048),
    (0xa342, 0xf0c1),
    (0xa344, 0xfcad),
    (0xa346, 0x0600),
    (0xa348, 0x2800),
    (0xa34a, 0xd143),
    (0xa34c, 0x1c20),
    (0xa34e, 0xf0c1),
    (0xa350, 0xfcb5),
    (0xa352, 0x0600),
    (0xa354, 0x2800),
    (0xa356, 0xd13d),
    (0xa358, 0x7868),
    (0xa35a, 0x2880),
    (0xa35c, 0xd115),
    (0xa35e, 0x1c20),
    (0xa360, 0xf0c1),
    (0xa362, 0xfc1a),
    (0xa364, 0x0600),
    (0xa366, 0x2800),
    (0xa368, 0xd02d),
    (0xa36a, 0x1c20),
    (0xa36c, 0xf0c1),
    (0xa36e, 0xfc50),
    (0xa370, 0x0600),
    (0xa372, 0x2800),
    (0xa374, 0xd027),
    (0xa376, 0x1c20),
    (0xa378, 0xf0c1),
    (0xa37a, 0xfc2e),
    (0xa37c, 0x0600),
    (0xa37e, 0x2800),
    (0xa380, 0xd128),
    (0xa382, 0x1c20),
    (0xa384, 0xf0c6),
    (0xa386, 0xfffc),
    (0xa388, 0xe01a),
    (0xa38a, 0x1c20),
    (0xa38c, 0xf0c1),
    (0xa38e, 0xfc24),
    (0xa390, 0x0600),
    (0xa392, 0x2800),
    (0xa394, 0xd11e),
    (0xa396, 0x1c20),
    (0xa398, 0xf0c6),
    (0xa39a, 0xfff2),
    (0xa39c, 0x0600),
    (0xa39e, 0x2800),
    (0xa3a0, 0xd118),
    (0xa3a2, 0x1c20),
    (0xa3a4, 0xf0c7),
    (0xa3a6, 0xf804),
    (0xa3a8, 0x0600),
    (0xa3aa, 0x2800),
    (0xa3ac, 0xd112),
    (0xa3ae, 0x1c20),
    (0xa3b0, 0xf0c7),
    (0xa3b2, 0xf822),
    (0xa3b4, 0x0600),
    (0xa3b6, 0x2800),
    (0xa3b8, 0xd10c),
    (0xa3ba, 0x1c20),
    (0xa3bc, 0xf0c7),
    (0xa3be, 0xf828),
    (0xa3c0, 0x0600),
    (0xa3c2, 0x2800),
    (0xa3c4, 0xd106),
    (0xa3c6, 0x78a9),
    (0xa3c8, 0x78e8),
    (0xa3ca, 0x0200),
    (0xa3cc, 0x4301),
    (0xa3ce, 0x8870),
    (0xa3d0, 0x1809),
    (0xa3d2, 0x8071),
    (0xa3d4, 0xbc70),
    (0xa3d6, 0xbc01),
    (0xa3d8, 0x4700),
    (0xa3da, 0x0000),
    (0xa3dc, 0xb570),
    (0xa3de, 0x1c06),
    (0xa3e0, 0x1c0d),
    (0xa3e2, 0x7868),
    (0xa3e4, 0xf11e),
    (0xa3e6, 0xfe5a),
    (0xa3e8, 0x1c04),
    (0xa3ea, 0x2c00),
    (0xa3ec, 0xd011),
    (0xa3ee, 0xf090),
    (0xa3f0, 0xf859),
    (0xa3f2, 0x6800),
    (0xa3f4, 0x42a0),
    (0xa3f6, 0xd10c),
    (0xa3f8, 0x4907),
    (0xa3fa, 0x1840),
    (0xa3fc, 0x7800),
    (0xa3fe, 0x78a9),
    (0xa400, 0x4288),
    (0xa402, 0xd106),
    (0xa404, 0x78e9),
    (0xa406, 0x7928),
    (0xa408, 0x0200),
    (0xa40a, 0x4301),
    (0xa40c, 0x8870),
    (0xa40e, 0x1809),
    (0xa410, 0x8071),
    (0xa412, 0xbc70),
    (0xa414, 0xbc01),
    (0xa416, 0x4700),
    (0xa418, 0x0101),
    (0xa41a, 0x0000),
    (0xa41c, 0xb510),
    (0xa41e, 0x1c04),
    (0xa420, 0x1c0b),
    (0xa422, 0x480b),
    (0xa424, 0x4a0b),
    (0xa426, 0x1881),
    (0xa428, 0x8808),
    (0xa42a, 0x2800),
    (0xa42c, 0xd00d),
    (0xa42e, 0x1c02),
    (0xa430, 0x7859),
    (0xa432, 0x7898),
    (0xa434, 0x0200),
    (0xa436, 0x4301),
    (0xa438, 0x428a),
    (0xa43a, 0xd106),
    (0xa43c, 0x78d9),
    (0xa43e, 0x7918),
    (0xa440, 0x0200),
    (0xa442, 0x4301),
    (0xa444, 0x8860),
    (0xa446, 0x1809),
    (0xa448, 0x8061),
    (0xa44a, 0xbc10),
    (0xa44c, 0xbc01),
    (0xa44e, 0x4700),
    (0xa450, 0x0000),
    (0xa452, 0x0200),
    (0xa454, 0x219e),
    (0xa456, 0x0000),
    (0xa458, 0xb5f0),
    (0xa45a, 0x4647),
    (0xa45c, 0xb480),
    (0xa45e, 0x4680),
    (0xa460, 0x1c0e),
    (0xa462, 0x7870),
    (0xa464, 0xf11e),
    (0xa466, 0xfe1a),
    (0xa468, 0x1c05),
    (0xa46a, 0x2d00),
    (0xa46c, 0xd02a),
    (0xa46e, 0x7928),
    (0xa470, 0x2800),
    (0xa472, 0xd027),
    (0xa474, 0x1c28),
    (0xa476, 0xf0c1),
    (0xa478, 0xfc21),
    (0xa47a, 0x0600),
    (0xa47c, 0x2800),
    (0xa47e, 0xd121),
    (0xa480, 0x1c28),
    (0xa482, 0xf0c1),
    (0xa484, 0xfc0d),
    (0xa486, 0x0600),
    (0xa488, 0x2800),
    (0xa48a, 0xd11b),
    (0xa48c, 0x78b1),
    (0xa48e, 0x2700),
    (0xa490, 0x2909),
    (0xa492, 0xd803),
    (0xa494, 0x8b28),
    (0xa496, 0x4288),
    (0xa498, 0xd309),
    (0xa49a, 0xe00a),
    (0xa49c, 0x8b2c),
    (0xa49e, 0x8b68),
    (0xa4a0, 0x4348),
    (0xa4a2, 0x2164),
    (0xa4a4, 0xf13b),
    (0xa4a6, 0xfdea),
    (0xa4a8, 0x4284),
    (0xa4aa, 0xdb00),
    (0xa4ac, 0x2701),
    (0xa4ae, 0x2f00),
    (0xa4b0, 0xd008),
    (0xa4b2, 0x78f1),
    (0xa4b4, 0x7930),
    (0xa4b6, 0x0200),
    (0xa4b8, 0x4301),
    (0xa4ba, 0x4640),
    (0xa4bc, 0x8840),
    (0xa4be, 0x1809),
    (0xa4c0, 0x4642),
    (0xa4c2, 0x8051),
    (0xa4c4, 0xbc08),
    (0xa4c6, 0x4698),
    (0xa4c8, 0xbcf0),
    (0xa4ca, 0xbc01),
    (0xa4cc, 0x4700),
    (0xa4ce, 0x0000),
    (0xa4d0, 0xb570),
    (0xa4d2, 0x1c06),
    (0xa4d4, 0x1c0d),
    (0xa4d6, 0x7868),
    (0xa4d8, 0xf0c1),
    (0xa4da, 0xfc24),
    (0xa4dc, 0x1c04),
    (0xa4de, 0x2c00),
    (0xa4e0, 0xd011),
    (0xa4e2, 0xf0c1),
    (0xa4e4, 0xfb69),
    (0xa4e6, 0x0600),
    (0xa4e8, 0x2800),
    (0xa4ea, 0xd10c),
    (0xa4ec, 0x1c20),
    (0xa4ee, 0xf0c1),
    (0xa4f0, 0xfb53),
    (0xa4f2, 0x0600),
    (0xa4f4, 0x2800),
    (0xa4f6, 0xd106),
    (0xa4f8, 0x78a9),
    (0xa4fa, 0x78e8),
    (0xa4fc, 0x0200),
    (0xa4fe, 0x4301),
    (0xa500, 0x8870),
    (0xa502, 0x1809),
    (0xa504, 0x8071),
    (0xa506, 0xbc70),
    (0xa508, 0xbc01),
    (0xa50a, 0x4700),
    (0xa50c, 0xb5f0),
    (0xa50e, 0x464f),
    (0xa510, 0x4646),
    (0xa512, 0xb4c0),
    (0xa514, 0xb08d),
    (0xa516, 0x4681),
    (0xa518, 0x1c0f),
    (0xa51a, 0x2000),
    (0xa51c, 0x4680),
    (0xa51e, 0xf08f),
    (0xa520, 0xffbb),
    (0xa522, 0x4669),
    (0xa524, 0xf092),
    (0xa526, 0xff66),
    (0xa528, 0x4580),
    (0xa52a, 0xda11),
    (0xa52c, 0x466e),
    (0xa52e, 0x1c05),
    (0xa530, 0x6834),
    (0xa532, 0x6820),
    (0xa534, 0xf0c1),
    (0xa536, 0xfb5c),
    (0xa538, 0x0600),
    (0xa53a, 0x2800),
    (0xa53c, 0xd104),
    (0xa53e, 0x6820),
    (0xa540, 0x490d),
    (0xa542, 0x1840),
    (0xa544, 0x7800),
    (0xa546, 0x4480),
    (0xa548, 0x3604),
    (0xa54a, 0x3d01),
    (0xa54c, 0x2d00),
    (0xa54e, 0xd1ef),
    (0xa550, 0x7878),
    (0xa552, 0x4540),
    (0xa554, 0xd108),
    (0xa556, 0x78b9),
    (0xa558, 0x78f8),
    (0xa55a, 0x0200),
    (0xa55c, 0x4301),
    (0xa55e, 0x464a),
    (0xa560, 0x8852),
    (0xa562, 0x1889),
    (0xa564, 0x4648),
    (0xa566, 0x8041),
    (0xa568, 0xb00d),
    (0xa56a, 0xbc18),
    (0xa56c, 0x4698),
    (0xa56e, 0x46a1),
    (0xa570, 0xbcf0),
    (0xa572, 0xbc01),
    (0xa574, 0x4700),
    (0xa576, 0x0000),
    (0xa578, 0x0101),
    (0xa57a, 0x0000),
    (0xa57c, 0xb570),
    (0xa57e, 0x1c05),
    (0xa580, 0x1c0c),
    (0xa582, 0x2600),
    (0xa584, 0x480a),
    (0xa586, 0x490b),
    (0xa588, 0x1840),
    (0xa58a, 0x7800),
    (0xa58c, 0xf02d),
    (0xa58e, 0xf926),
    (0xa590, 0x0600),
    (0xa592, 0x0e01),
    (0xa594, 0x7860),
    (0xa596, 0x2800),
    (0xa598, 0xd10e),
    (0xa59a, 0x2001),
    (0xa59c, 0x2901),
    (0xa59e, 0xd010),
    (0xa5a0, 0x3001),
    (0xa5a2, 0x0600),
    (0xa5a4, 0x0e00),
    (0xa5a6, 0x2806),
    (0xa5a8, 0xd809),
    (0xa5aa, 0x4281),
    (0xa5ac, 0xd1f8),
    (0xa5ae, 0xe008),
    (0xa5b0, 0x0000),
    (0xa5b2, 0x0200),
    (0xa5b4, 0x1f69),
    (0xa5b6, 0x0000),
    (0xa5b8, 0x4281),
    (0xa5ba, 0xd100),
    (0xa5bc, 0x2601),
    (0xa5be, 0x2e00),
    (0xa5c0, 0xd006),
    (0xa5c2, 0x78a1),
    (0xa5c4, 0x78e0),
    (0xa5c6, 0x0200),
    (0xa5c8, 0x4301),
    (0xa5ca, 0x8868),
    (0xa5cc, 0x1809),
    (0xa5ce, 0x8069),
    (0xa5d0, 0xbc70),
    (0xa5d2, 0xbc01),
    (0xa5d4, 0x4700),
    (0xa5d6, 0x0000),
    (0xa5d8, 0xb500),
    (0xa5da, 0xf7ff),
    (0xa5dc, 0xffcf),
    (0xa5de, 0xbc01),
    (0xa5e0, 0x4700),
    (0xa5e2, 0x0000),
    (0xa5e4, 0xb570),
    (0xa5e6, 0x1c06),
    (0xa5e8, 0x1c0d),
    (0xa5ea, 0x4a17),
    (0xa5ec, 0x1c10),
    (0xa5ee, 0x30a8),
    (0xa5f0, 0x7800),
    (0xa5f2, 0x3801),
    (0xa5f4, 0x0601),
    (0xa5f6, 0x0e08),
    (0xa5f8, 0x280b),
    (0xa5fa, 0xd822),
    (0xa5fc, 0x1609),
    (0xa5fe, 0x0148),
    (0xa600, 0x1840),
    (0xa602, 0x00c0),
    (0xa604, 0x4b11),
    (0xa606, 0x18d1),
    (0xa608, 0x1844),
    (0xa60a, 0x7920),
    (0xa60c, 0x2800),
    (0xa60e, 0xd011),
    (0xa610, 0x1c20),
    (0xa612, 0xf0c1),
    (0xa614, 0xfafd),
    (0xa616, 0x0600),
    (0xa618, 0x2800),
    (0xa61a, 0xd00b),
    (0xa61c, 0x1c20),
    (0xa61e, 0xf0c1),
    (0xa620, 0xfb4d),
    (0xa622, 0x0600),
    (0xa624, 0x2800),
    (0xa626, 0xd105),
    (0xa628, 0x1c20),
    (0xa62a, 0xf0c1),
    (0xa62c, 0xfb39),
    (0xa62e, 0x0600),
    (0xa630, 0x2800),
    (0xa632, 0xd006),
    (0xa634, 0x7869),
    (0xa636, 0x78a8),
    (0xa638, 0x0200),
    (0xa63a, 0x4301),
    (0xa63c, 0x8870),
    (0xa63e, 0x1809),
    (0xa640, 0x8071),
    (0xa642, 0xbc70),
    (0xa644, 0xbc01),
    (0xa646, 0x4700),
    (0xa648, 0x20d0),
    (0xa64a, 0x0200),
    (0xa64c, 0x0ef4),
    (0xa64e, 0x0000),
    (0xa650, 0xb530),
    (0xa652, 0x1c05),
    (0xa654, 0x1c0c),
    (0xa656, 0x200d),
    (0xa658, 0xf0c1),
    (0xa65a, 0xfb64),
    (0xa65c, 0x2800),
    (0xa65e, 0xd00e),
    (0xa660, 0x4808),
    (0xa662, 0x4909),
    (0xa664, 0x1840),
    (0xa666, 0x8801),
    (0xa668, 0x20a0),
    (0xa66a, 0x0040),
    (0xa66c, 0x4281),
    (0xa66e, 0xd106),
    (0xa670, 0x7861),
    (0xa672, 0x78a0),
    (0xa674, 0x0200),
    (0xa676, 0x4301),
    (0xa678, 0x8868),
    (0xa67a, 0x1809),
    (0xa67c, 0x8069),
    (0xa67e, 0xbc30),
    (0xa680, 0xbc01),
    (0xa682, 0x4700),
    (0xa684, 0x0000),
    (0xa686, 0x0200),
    (0xa688, 0x3c9c),
    (0xa68a, 0x0000),
    (0xa68c, 0xb530),
    (0xa68e, 0x1c05),
    (0xa690, 0x1c0c),
    (0xa692, 0xf08f),
    (0xa694, 0xff01),
    (0xa696, 0xf093),
    (0xa698, 0xfbfd),
    (0xa69a, 0x0600),
    (0xa69c, 0x2800),
    (0xa69e, 0xd106),
    (0xa6a0, 0x7861),
    (0xa6a2, 0x78a0),
    (0xa6a4, 0x0200),
    (0xa6a6, 0x4301),
    (0xa6a8, 0x8868),
    (0xa6aa, 0x1809),
    (0xa6ac, 0x8069),
    (0xa6ae, 0xbc30),
    (0xa6b0, 0xbc01),
    (0xa6b2, 0x4700),
    (0xa6b4, 0xb500),
    (0xa6b6, 0xf7ff),
    (0xa6b8, 0xfd4f),
    (0xa6ba, 0xbc02),
    (0xa6bc, 0x4708),
    (0xa6be, 0x0000),
    (0xa6c0, 0xb500),
    (0xa6c2, 0x0600),
    (0xa6c4, 0x2100),
    (0xa6c6, 0x2800),
    (0xa6c8, 0xd100),
    (0xa6ca, 0x2101),
    (0xa6cc, 0x1c08),
    (0xa6ce, 0xf7ff),
    (0xa6d0, 0xfd43),
    (0xa6d2, 0xbc02),
    (0xa6d4, 0x4708),
    (0xa6d6, 0x0000),
    (0xa6d8, 0xb500),
    (0xa6da, 0x2002),
    (0xa6dc, 0xf7ff),
    (0xa6de, 0xfd3c),
    (0xa6e0, 0xbc02),
    (0xa6e2, 0x4708),
    (0x13142, 0x4b1b),
    (0x13144, 0x3238),
    (0x13146, 0x7810),
    (0x13148, 0x1c41),
    (0x1314a, 0x7011),
    (0x1314c, 0x0600),
    (0x1314e, 0x1600),
    (0x12fa56, 0x6872),
    (0x12fa58, 0x6811),
    (0x12fa5a, 0x481e),
    (0x12fa5c, 0x1809),
    (0x12fa5e, 0x9104),
    (0x12fa60, 0x4f1d),
    (0x12fa62, 0x46b8),
    (0x12fa64, 0x6850),
];
const CONTRACT_WORDS: &[(u32, u32)] = &[
    (0xa234, 0x08d69bc4),
    (0xa288, 0x08370f94),
    (0x131b0, 0x08370ffc),
    (0x12fad4, 0x08d6b4c8),
];

const CALLBACK_OFFSETS: &[u32] = &[
    0xa294, 0xa298, 0xa2a8, 0xa2bc, 0xa2d0, 0xa2fc, 0xa330, 0xa3dc, 0xa41c, 0xa458, 0xa4d0, 0xa50c,
    0xa57c, 0xa5d8, 0xa5e4, 0xa650, 0xa68c,
];
