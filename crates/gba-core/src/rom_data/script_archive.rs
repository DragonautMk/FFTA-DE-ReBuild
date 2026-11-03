//! EUR relative script storage. Original cursors, branches and callback execution stay live.
use super::{effect_event_directories, script_commands, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const ARCHIVE_OFFSET: u32 = 0xb946ec;
pub const ARCHIVE_END: u32 = 0xbb064c;
pub const KEY_OFFSET: u32 = 0xd68c2c;
pub const KEY_COUNT: usize = 234;
pub const SOURCE_BYTES: usize = 115464;

/// Original 08009aa0 low16 arithmetic; this is not a bound on guest IDs.
pub fn event_key_source_offset(raw_id: u32) -> u32 {
    KEY_OFFSET + u32::from(raw_id as u16) * 4
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventKey {
    pub id: u16,
    pub script_bank: u8,
    pub script_index: u8,
    pub text_bank: u8,
    pub byte3_raw: u8,
}
impl EventKey {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        [
            self.script_bank,
            self.script_index,
            self.text_bank,
            self.byte3_raw,
        ]
        .get(offset)
        .copied()
    }
}

/// Some source slots name a bank header or alignment fill, not a command segment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Program { ordinal: u16 },
    Header { bank: u8 },
    Fill { ordinal: u8 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub index: u8,
    pub target: Target,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub ordinal: u8,
    pub source_offset: u32,
    pub slots: Vec<Slot>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    pub source_offset: u32,
    pub command: u8,
    /// Storage parameters; no unproved argument semantics or branch normalization.
    pub parameters: Vec<u8>,
    pub effect_selector: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub ordinal: u16,
    pub source_offset: u32,
    pub instructions: Vec<Instruction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlignmentFill {
    pub source_offset: u32,
    pub length: u8,
    pub value: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptArchive {
    pub archive_offset: u32,
    pub key_offset: u32,
    pub keys: Vec<EventKey>,
    pub banks: Vec<Bank>,
    pub programs: Vec<Program>,
    pub padding: Vec<AlignmentFill>,
}
impl ScriptArchive {
    pub fn target_offset(&self, target: &Target) -> Option<u32> {
        match *target {
            Target::Program { ordinal } => {
                Some(self.programs.get(usize::from(ordinal))?.source_offset)
            }
            Target::Header { bank } => Some(
                self.banks
                    .get(usize::from(bank.checked_sub(1)?))?
                    .source_offset,
            ),
            Target::Fill { ordinal } => Some(self.padding.get(usize::from(ordinal))?.source_offset),
        }
    }
    /// Logical projection reference only. None does not change original guest reads.
    pub fn key_target(&self, id: u16) -> Option<u32> {
        let key = self.keys.get(usize::from(id))?;
        let bank = self
            .banks
            .get(usize::from(key.script_bank.checked_sub(1)?))?;
        self.target_offset(&bank.slots.get(usize::from(key.script_index))?.target)
    }
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        if let Some(n) = at.checked_sub(self.key_offset) {
            if let Some(key) = self.keys.get(n as usize / 4) {
                return key.byte(n as usize % 4);
            }
        }
        if let Some(n) = at.checked_sub(self.archive_offset) {
            if n < self.banks.len() as u32 * 4 {
                return Some(
                    self.banks[n as usize / 4]
                        .source_offset
                        .wrapping_sub(self.archive_offset)
                        .to_le_bytes()[n as usize % 4],
                );
            }
        }
        if let Some(n) = self
            .banks
            .partition_point(|b| b.source_offset <= at)
            .checked_sub(1)
        {
            let bank = &self.banks[n];
            let relative = (at - bank.source_offset) as usize;
            if let Some(slot) = bank.slots.get(relative / 4) {
                return Some(
                    self.target_offset(&slot.target)?
                        .wrapping_sub(bank.source_offset)
                        .to_le_bytes()[relative % 4],
                );
            }
        }
        if let Some(n) = self
            .programs
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)
        {
            let p = &self.programs[n];
            if let Some(n) = p
                .instructions
                .partition_point(|i| i.source_offset <= at)
                .checked_sub(1)
            {
                let i = &p.instructions[n];
                let relative = (at - i.source_offset) as usize;
                if relative == 0 {
                    return Some(i.command);
                }
                if let Some(byte) = i.parameters.get(relative - 1) {
                    return Some(*byte);
                }
            }
        }
        if let Some(n) = self
            .padding
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)
        {
            let p = &self.padding[n];
            if at - p.source_offset < u32::from(p.length) {
                return Some(p.value);
            }
        }
        None
    }
    pub fn validate(&self) -> Result<(), String> {
        let bad = || "Script archive shape/source/reference mismatch".to_string();
        if self.archive_offset != ARCHIVE_OFFSET
            || self.key_offset != KEY_OFFSET
            || self.keys.len() != KEY_COUNT
            || self.banks.len() != 62
            || self.programs.len() != 207
            || self.padding.len() != 45
        {
            return Err(bad());
        }
        if self
            .banks
            .windows(2)
            .any(|b| b[0].source_offset >= b[1].source_offset)
            || self
                .programs
                .windows(2)
                .any(|b| b[0].source_offset >= b[1].source_offset)
            || self
                .padding
                .windows(2)
                .any(|b| b[0].source_offset >= b[1].source_offset)
        {
            return Err(bad());
        }
        let mut intervals = vec![(ARCHIVE_OFFSET, ARCHIVE_OFFSET + 62 * 4)];
        let mut slots = 0;
        for (n, bank) in self.banks.iter().enumerate() {
            if usize::from(bank.ordinal) != n + 1
                || bank.slots.is_empty()
                || bank.slots.len() > 256
                || bank.source_offset % 4 != 0
            {
                return Err(bad());
            }
            let end = bank
                .source_offset
                .checked_add(bank.slots.len() as u32 * 4)
                .ok_or_else(bad)?;
            intervals.push((bank.source_offset, end));
            slots += bank.slots.len();
            for (i, slot) in bank.slots.iter().enumerate() {
                if usize::from(slot.index) != i || self.target_offset(&slot.target).is_none() {
                    return Err(bad());
                }
            }
        }
        if slots != 233 {
            return Err(bad());
        }
        for (n, key) in self.keys.iter().enumerate() {
            if usize::from(key.id) != n {
                return Err(bad());
            }
            if n == 0 {
                if key.script_bank != 0 || key.script_index != 0 {
                    return Err(bad());
                }
            } else if self.key_target(key.id).is_none() {
                return Err(bad());
            }
        }
        let mut commands = 0usize;
        let mut args = 0usize;
        for (n, p) in self.programs.iter().enumerate() {
            if usize::from(p.ordinal) != n || p.instructions.is_empty() {
                return Err(bad());
            }
            let mut at = p.source_offset;
            for i in &p.instructions {
                if i.source_offset != at || usize::from(i.command) >= script_commands::COMMAND_COUNT
                {
                    return Err(bad());
                }
                if let Some(selector) = i.effect_selector {
                    if i.command != 90
                        || i.parameters.len() != 1
                        || i.parameters[0] == 0
                        || i.parameters[0] - 1 != selector
                    {
                        return Err(bad());
                    }
                }
                let length = u32::try_from(i.parameters.len())
                    .map_err(|_| bad())?
                    .checked_add(1)
                    .ok_or_else(bad)?;
                at = at.checked_add(length).ok_or_else(bad)?;
                commands += 1;
                args += i.parameters.len();
            }
            intervals.push((p.source_offset, at));
        }
        if commands != 35048 || args != 78212 {
            return Err(bad());
        }
        let mut fills = 0;
        for p in &self.padding {
            if !(1..=3).contains(&p.length) {
                return Err(bad());
            }
            let end = p
                .source_offset
                .checked_add(u32::from(p.length))
                .ok_or_else(bad)?;
            if end % 4 != 0
                || !(end == ARCHIVE_END || self.banks.iter().any(|b| b.source_offset == end))
            {
                return Err(bad());
            }
            intervals.push((p.source_offset, end));
            fills += usize::from(p.length);
        }
        if fills != 88 {
            return Err(bad());
        }
        intervals.sort_unstable();
        let mut at = ARCHIVE_OFFSET;
        for (a, z) in intervals {
            if a != at || z <= a || z > ARCHIVE_END {
                return Err(bad());
            }
            at = z;
        }
        if at != ARCHIVE_END {
            return Err(bad());
        }
        Ok(())
    }
    pub fn validate_dependencies(
        &self,
        commands: &script_commands::ScriptCommands,
        effects: &effect_event_directories::EffectEventDirectories,
    ) -> Result<(), String> {
        self.validate()?;
        commands.validate()?;
        effects.validate()?;
        for i in self.programs.iter().flat_map(|p| &p.instructions) {
            if usize::from(commands.commands[usize::from(i.command)].advance)
                != 1 + i.parameters.len()
            {
                return Err("Script archive command length dependency mismatch".into());
            }
            let expected = if i.command == 90 {
                i.parameters
                    .first()
                    .and_then(|q| q.checked_sub(1))
                    .filter(|q| usize::from(*q) < effects.selectors.len())
            } else {
                None
            };
            if i.effect_selector != expected {
                return Err("Script archive effect selector dependency mismatch".into());
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        [
            (
                ARCHIVE_OFFSET,
                ARCHIVE_END,
                "engine/script-archive/indexed-bytecode",
            ),
            (
                KEY_OFFSET,
                KEY_OFFSET + KEY_COUNT as u32 * 4,
                "engine/script-archive/event-keys-0-233",
            ),
        ]
        .into_iter()
        .map(|(a, z, role)| {
            Ok((
                a,
                (a..z)
                    .map(|p| {
                        self.byte_at(p)
                            .ok_or_else(|| "Missing script archive physical byte".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                role.into(),
            ))
        })
        .collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Script archive differs from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ScriptArchive,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Script archive requires exact European source ROM".into());
        }
        verify_contract(rom)?;
        let word = |a: u32| u32::from_le_bytes(rom[a as usize..a as usize + 4].try_into().unwrap());
        let commands = script_commands::Catalog::from_rom(rom)?.decoded;
        let effects = effect_event_directories::Catalog::from_rom(rom)?.decoded;
        let keys: Vec<_> = (0..KEY_COUNT)
            .map(|n| {
                let a = KEY_OFFSET as usize + n * 4;
                EventKey {
                    id: n as u16,
                    script_bank: rom[a],
                    script_index: rom[a + 1],
                    text_bank: rom[a + 2],
                    byte3_raw: rom[a + 3],
                }
            })
            .collect();
        let count = usize::from(keys.iter().map(|k| k.script_bank).max().unwrap());
        if count != 62 || word(ARCHIVE_OFFSET) != count as u32 * 4 {
            return Err("Script archive top-level domain mismatch".into());
        }
        let mut headers = vec![(ARCHIVE_OFFSET, ARCHIVE_OFFSET + count as u32 * 4)];
        let mut raw_banks = Vec::new();
        let mut heads = BTreeSet::new();
        for n in 0..count {
            let a = ARCHIVE_OFFSET
                .checked_add(word(ARCHIVE_OFFSET + n as u32 * 4))
                .ok_or("Script archive bank overflow")?;
            if a < ARCHIVE_OFFSET + count as u32 * 4 || a + 4 > ARCHIVE_END {
                return Err("Script archive bank source mismatch".into());
            }
            let length = word(a);
            if length == 0
                || length % 4 != 0
                || a.checked_add(length).is_none_or(|z| z >= ARCHIVE_END)
            {
                return Err("Script archive nested index mismatch".into());
            }
            let domain: BTreeSet<_> = keys
                .iter()
                .filter(|k| usize::from(k.script_bank) == n + 1)
                .map(|k| u32::from(k.script_index))
                .collect();
            if domain.iter().copied().ne(0..length / 4) {
                return Err("Script archive event-key slot domain mismatch".into());
            }
            let targets: Vec<_> = (0..length / 4)
                .map(|i| a.wrapping_add(word(a + i * 4)))
                .collect();
            if targets
                .iter()
                .any(|p| !(ARCHIVE_OFFSET..ARCHIVE_END).contains(p))
            {
                return Err("Script archive target outside source projection".into());
            }
            heads.extend(targets.iter().copied());
            headers.push((a, a + length));
            raw_banks.push((a, targets));
        }
        if headers.windows(2).any(|h| h[0].1 > h[1].0) {
            return Err("Script archive overlapping index owners".into());
        }
        let mut anchors = heads;
        anchors.extend([ARCHIVE_OFFSET, ARCHIVE_END]);
        for &(a, z) in &headers {
            anchors.extend([a, z]);
        }
        let anchors: Vec<_> = anchors.into_iter().collect();
        let mut programs = Vec::new();
        let mut padding = Vec::new();
        for interval in anchors.windows(2) {
            let (a, z) = (interval[0], interval[1]);
            if headers.iter().any(|h| h.0 <= a && z <= h.1) {
                continue;
            }
            let mut at = a;
            let mut instructions = Vec::new();
            while at < z {
                let opcode = rom[at as usize];
                let command = commands.commands.get(usize::from(opcode));
                if command.is_none_or(|c| c.advance == 0 || at + u32::from(c.advance) > z) {
                    if z - at >= 4
                        || z % 4 != 0
                        || !(z == ARCHIVE_END || headers.iter().any(|h| h.0 == z))
                        || rom[at as usize..z as usize].iter().any(|b| *b != 255)
                    {
                        return Err("Script archive noncanonical command/fill boundary".into());
                    }
                    padding.push(AlignmentFill {
                        source_offset: at,
                        length: (z - at) as u8,
                        value: 255,
                    });
                    break;
                }
                let size = u32::from(command.unwrap().advance);
                let parameters = rom[at as usize + 1..(at + size) as usize].to_vec();
                let effect_selector = if opcode == 90 {
                    parameters
                        .first()
                        .and_then(|q| q.checked_sub(1))
                        .filter(|q| usize::from(*q) < effects.selectors.len())
                } else {
                    None
                };
                instructions.push(Instruction {
                    source_offset: at,
                    command: opcode,
                    parameters,
                    effect_selector,
                });
                at += size;
            }
            if !instructions.is_empty() {
                programs.push(Program {
                    ordinal: programs.len() as u16,
                    source_offset: a,
                    instructions,
                });
            }
        }
        let mut banks = Vec::new();
        for (n, (a, targets)) in raw_banks.iter().enumerate() {
            let mut slots = Vec::new();
            for (i, target) in targets.iter().enumerate() {
                let target = if let Some(p) =
                    programs.iter().position(|p| p.source_offset == *target)
                {
                    Target::Program { ordinal: p as u16 }
                } else if let Some(p) = raw_banks.iter().position(|b| b.0 == *target) {
                    Target::Header {
                        bank: (p + 1) as u8,
                    }
                } else if let Some(p) = padding.iter().position(|p| p.source_offset == *target) {
                    Target::Fill { ordinal: p as u8 }
                } else {
                    return Err("Script archive target view mismatch".into());
                };
                slots.push(Slot {
                    index: i as u8,
                    target,
                });
            }
            banks.push(Bank {
                ordinal: (n + 1) as u8,
                source_offset: *a,
                slots,
            });
        }
        let decoded = ScriptArchive {
            archive_offset: ARCHIVE_OFFSET,
            key_offset: KEY_OFFSET,
            keys,
            banks,
            programs,
            padding,
        };
        decoded.validate_dependencies(&commands, &effects)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Script archive source roundtrip mismatch".into());
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
            return Err("Script archive original reader contract mismatch".into());
        }
    }
    for &(offset, value) in CONTRACT_WORDS {
        if u32::from_le_bytes(
            rom[offset as usize..offset as usize + 4]
                .try_into()
                .unwrap(),
        ) != value
        {
            return Err("Script archive original root/boundary contract mismatch".into());
        }
    }
    Ok(())
}

// Separately retained original-reader and independent-successor fingerprints.
const CONTRACT_HALFWORDS: &[(u32, u16)] = &[
    (0x9aa0, 0x0400),
    (0x9aa2, 0x4902),
    (0x9aa4, 0x0b80),
    (0x9aa6, 0x1840),
    (0x9aa8, 0x4770),
    (0x9afc, 0x4801),
    (0x9afe, 0xbc02),
    (0x9b00, 0x4708),
    (0x9b08, 0xb570),
    (0x9b0a, 0x0400),
    (0x9b0c, 0x0c06),
    (0x9b0e, 0x2eef),
    (0x9b10, 0xd901),
    (0x9b12, 0x20aa),
    (0x9b14, 0xe003),
    (0x9b16, 0x4816),
    (0x9b18, 0x4916),
    (0x9b1a, 0x1840),
    (0x9b1c, 0x8800),
    (0x9b1e, 0xf7ff),
    (0x9b20, 0xffbf),
    (0x9b22, 0x1c05),
    (0x9b24, 0xf7ff),
    (0x9b26, 0xffc4),
    (0x9b28, 0x78a9),
    (0x9b2a, 0x0089),
    (0x9b2c, 0x1844),
    (0x9b2e, 0xf7ff),
    (0x9b30, 0xffbf),
    (0x9b32, 0x6821),
    (0x9b34, 0x1844),
    (0x9b36, 0x2eef),
    (0x9b38, 0xd815),
    (0x9b3a, 0x78a8),
    (0x9b3c, 0x283d),
    (0x125b68, 0x1c10),
    (0x125b6a, 0xf000),
    (0x125b6c, 0xfd2f),
    (0x125b6e, 0x1c01),
    (0x125b70, 0x208f),
    (0x125b72, 0x0040),
    (0x125b74, 0x4281),
    (0x125b76, 0xd804),
    (0x125b78, 0x3801),
    (0x125b7a, 0x4281),
    (0x125b7c, 0xd301),
    (0x125b7e, 0x2003),
    (0x125b80, 0xe071),
    (0x125b82, 0x8031),
    (0x125e8c, 0xb500),
    (0x125e8e, 0xb087),
    (0x125e90, 0x0400),
    (0x125e92, 0x0c00),
    (0x125e94, 0xf6e3),
    (0x125e96, 0xfe04),
    (0x125e98, 0x4a0d),
    (0x125e9a, 0x7801),
    (0x125e9c, 0x0089),
    (0x125e9e, 0x1889),
    (0x125ea0, 0x3904),
    (0x125ea2, 0x6809),
    (0x125ea4, 0x188a),
    (0x125ea6, 0x7840),
    (0x125ea8, 0x0080),
    (0x125eaa, 0x1880),
    (0x125eac, 0x6800),
    (0x125eae, 0x1812),
    (0x125eb0, 0x9201),
    (0x125eb2, 0x4669),
    (0x125eb4, 0x2000),
    (0x125eb6, 0x8288),
    (0x125eb8, 0x20ff),
    (0x125eba, 0x82c8),
    (0x125ebc, 0x4805),
    (0x125ebe, 0x2110),
    (0x125ec0, 0x466a),
    (0x125ec2, 0x2300),
    (0x125ec4, 0xf7ff),
    (0x125ec6, 0xfbea),
    (0x125ec8, 0xb007),
    (0x125eca, 0xbc02),
    (0x125ecc, 0x4708),
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
    (0x12891c, 0x0400),
    (0x12891e, 0x0609),
    (0x128920, 0x0e09),
    (0x128922, 0x4a1c),
    (0x128924, 0x0b80),
    (0x128926, 0x1880),
    (0x128928, 0x6800),
    (0x12892a, 0x1885),
];
const CONTRACT_WORDS: &[(u32, u32)] = &[
    (0x9aac, 0x08d68c2c),
    (0x125ed0, 0x08b946ec),
    (0x125ed4, 0x08125ac9),
    (0x9b04, 0x08bb064c),
    (0x128994, 0x08d68fd4),
];
