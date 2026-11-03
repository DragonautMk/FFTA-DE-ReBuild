//! Original EUR blend programs. Guest interpolation, messages and mutable RAM stay live.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Canonical source locations/record counts from bounded producer proofs; no payload.
pub const PROGRAM_LAYOUTS: [(u32, u16); 160] = [
    (0x39a5f0, 1),
    (0x39a634, 1),
    (0x39aa97, 1),
    (0x39aaa1, 1),
    (0x39b0d0, 1),
    (0x39b0da, 1),
    (0x39db7c, 1),
    (0x39e04c, 1),
    (0x3a0250, 1),
    (0x3a7dcc, 1),
    (0x3a7eaa, 1),
    (0x3a7eb4, 1),
    (0x3a83bc, 1),
    (0x3a83c6, 1),
    (0x3a84b0, 1),
    (0x3a8960, 1),
    (0x3a896a, 1),
    (0x3a8974, 1),
    (0x3a89a6, 1),
    (0x3ab960, 1),
    (0x3ab96a, 1),
    (0x3abba0, 1),
    (0x3ac2ec, 1),
    (0x3ac2f6, 1),
    (0xb4a768, 1),
    (0xb4a774, 1),
    (0xb4a780, 1),
    (0xb4a78c, 2),
    (0xb4a79c, 2),
    (0xb4a7ac, 5),
    (0xb4a7c8, 2),
    (0xb4a7d8, 2),
    (0xb4a7e8, 2),
    (0xb4a7f8, 3),
    (0xb4a80c, 3),
    (0xb4a820, 3),
    (0xb4a834, 1),
    (0xb4a840, 1),
    (0xb4a84c, 1),
    (0xb4a858, 2),
    (0xb4a868, 2),
    (0xb4a878, 3),
    (0xb4a88c, 1),
    (0xb4a8a4, 2),
    (0xb4a8b4, 2),
    (0xb4a8c4, 2),
    (0xb4a8d4, 2),
    (0xb4a8e4, 3),
    (0xb4a8f8, 3),
    (0xb4a90c, 2),
    (0xb4a91c, 1),
    (0xb4a928, 4),
    (0xb4a940, 1),
    (0xb4a94c, 3),
    (0xb4a960, 2),
    (0xb4a970, 4),
    (0xb4a988, 1),
    (0xb4a994, 3),
    (0xb4a9a8, 1),
    (0xb4a9b4, 3),
    (0xb4a9c8, 2),
    (0xb4a9d8, 2),
    (0xb4a9e8, 2),
    (0xb4a9f8, 3),
    (0xb4aa0c, 2),
    (0xb4aa1c, 2),
    (0xb4aa2c, 2),
    (0xb4aa3c, 2),
    (0xb4aa4c, 2),
    (0xb4aa5c, 2),
    (0xb4aa6c, 3),
    (0xb4aa80, 2),
    (0xb4aa90, 3),
    (0xb4aaa4, 1),
    (0xb4aab0, 2),
    (0xb4aac0, 2),
    (0xb4aad0, 2),
    (0xb4aae0, 2),
    (0xb4aaf0, 2),
    (0xb4ab00, 3),
    (0xb4ab14, 1),
    (0xb4ab20, 1),
    (0xb4ab38, 13),
    (0xb4ab74, 3),
    (0xb4ab88, 2),
    (0xb4ab98, 3),
    (0xb4abac, 3),
    (0xb4abc0, 3),
    (0xb4abd4, 3),
    (0xb4abe8, 1),
    (0xb4abf4, 2),
    (0xb4ac04, 3),
    (0xb4ac18, 1),
    (0xb4ac24, 2),
    (0xb4ac34, 2),
    (0xb4ac44, 2),
    (0xb4ac54, 2),
    (0xb4ac64, 3),
    (0xb4ac78, 1),
    (0xb4ac84, 4),
    (0xb4ac9c, 10),
    (0xb4accc, 2),
    (0xb4acdc, 2),
    (0xb4acec, 2),
    (0xb4acfc, 3),
    (0xb4ad10, 2),
    (0xb4ad20, 4),
    (0xb4ad38, 2),
    (0xb4ad48, 1),
    (0xb4ad54, 2),
    (0xb4ad64, 2),
    (0xb4ad80, 1),
    (0xb4ad8c, 1),
    (0xb4ad98, 1),
    (0xb4ada4, 1),
    (0xb4adb0, 1),
    (0xb4add0, 1),
    (0xb4addc, 1),
    (0xb4ade8, 4),
    (0xb4ae00, 3),
    (0xb4ae14, 6),
    (0xb4ae34, 3),
    (0xb4ae48, 3),
    (0xb4ae5c, 3),
    (0xb4ae70, 1),
    (0xb4ae7c, 1),
    (0xb4ae88, 3),
    (0xb4ae9c, 3),
    (0xb4aeb0, 1),
    (0xb4aebc, 1),
    (0xb4aec8, 1),
    (0xb4aed4, 7),
    (0xb4af08, 6),
    (0xb4af28, 3),
    (0xb4af3c, 3),
    (0xb4af50, 2),
    (0xb4af60, 3),
    (0xb4af74, 1),
    (0xb4af80, 3),
    (0xb4af94, 3),
    (0xb4afa8, 2),
    (0xb4afb8, 2),
    (0xb4afc8, 2),
    (0xb4afd8, 2),
    (0xb4afe8, 5),
    (0xb4b004, 1),
    (0xb4b010, 3),
    (0xb4b024, 3),
    (0xb4b038, 3),
    (0xb4b04c, 3),
    (0xb4b060, 3),
    (0xb4b074, 1),
    (0xb4b080, 1),
    (0xb4b08c, 1),
    (0xb4b098, 2),
    (0xb4b0a8, 2),
    (0xb4b0b8, 2),
    (0xb4b0c8, 2),
    (0xb4b0d8, 2),
    (0xb4b0e8, 1),
];
pub const PRODUCER_LAYOUTS: [(u32, usize); 6] = [
    (0x39b7bc, 3),
    (0x39ccac, 3),
    (0x39cf38, 3),
    (0x39d1b0, 3),
    (0x3a2c04, 5),
    (0x3ac734, 5),
];
const SOURCE_CONTRACT: &[(u32, u16)] = &[
    (0xd76e8, 0x89b0),
    (0xd76ea, 0x0080),
    (0xd76ec, 0x3006),
    (0xd76f0, 0x7829),
    (0xd76f2, 0x7868),
    (0xd76f8, 0x42b8),
    (0xd76fc, 0x78a8),
    (0xd7702, 0x78e8),
    (0xd770c, 0x791a),
    (0xd770e, 0x7959),
    (0xd7736, 0x78a8),
    (0xd774e, 0x041b),
    (0xd7750, 0x141b),
    (0xd7768, 0x78e8),
    (0xd7780, 0x041b),
    (0xd7782, 0x141b),
    (0xd77d2, 0x8831),
    (0xd77f4, 0x6872),
    (0xd77f8, 0x8ab0),
    (0xd77fc, 0x8af0),
    (0xd77fe, 0x3001),
    (0xd7800, 0x8168),
    (0xd7802, 0x7811),
    (0xd7804, 0x7850),
    (0xd780a, 0x21c0),
    (0xd780e, 0x0980),
    (0xd7812, 0x7810),
    (0xd7816, 0x7811),
    (0xd7818, 0x7850),
    (0xd781e, 0x23fc),
    (0xd7820, 0x019b),
    (0xd7826, 0x0a00),
    (0xd782a, 0x7890),
    (0xd782c, 0x211f),
    (0xd7832, 0x7891),
    (0xd7834, 0x78d0),
    (0xd783a, 0x22f8),
    (0xd783c, 0x0152),
    (0xd7842, 0x0a00),
    (0xd7886, 0x2080),
    (0xd7888, 0x4008),
    (0xd7946, 0x89a8),
    (0xd7948, 0x3001),
    (0xd794a, 0x81a8),
    (0xd794e, 0x790a),
    (0xd7950, 0x7949),
    (0xd795e, 0x7929),
    (0xd7960, 0x2001),
    (0xd7986, 0x7928),
    (0xd7988, 0x4001),
    (0xe775c, 0x7d30),
    (0xe775e, 0x3802),
    (0xe7760, 0x7068),
    (0xe781a, 0x7868),
    (0xe781c, 0x0080),
    (0xe781e, 0x1840),
    (0xe7820, 0x6800),
    (0xeb86c, 0x8ab8),
    (0xeb86e, 0x7028),
    (0xeb90e, 0x7828),
    (0xeb910, 0x0080),
    (0xeb912, 0x1840),
    (0xeb914, 0x6800),
    (0xebac4, 0x3818),
    (0xebac6, 0x7028),
    (0xebace, 0x7029),
    (0xebbe6, 0x7828),
    (0xebbe8, 0x8288),
    (0xebca4, 0x8ab8),
    (0xebca6, 0x7028),
    (0xebd50, 0x7828),
    (0xebd52, 0x0080),
    (0xebd54, 0x1840),
    (0xebd56, 0x6800),
    (0xebefc, 0x381b),
    (0xebefe, 0x7028),
    (0xebf06, 0x7029),
    (0xec01e, 0x7828),
    (0xec020, 0x8288),
    (0xec0dc, 0x8ab8),
    (0xec0de, 0x7028),
    (0xec17e, 0x7828),
    (0xec180, 0x0080),
    (0xec182, 0x1840),
    (0xec184, 0x6800),
    (0xec33c, 0x381e),
    (0xec33e, 0x7028),
    (0xec346, 0x7029),
    (0xec4a2, 0x7828),
    (0xec4a4, 0x8288),
    (0x1020d8, 0x7d30),
    (0x1020da, 0x306e),
    (0x1020dc, 0x71a8),
    (0x1021f6, 0x79a8),
    (0x1021f8, 0x0080),
    (0x1021fa, 0x1840),
    (0x1021fc, 0x6800),
    (0x1234e6, 0x8a88),
    (0x1234e8, 0x7228),
    (0x123766, 0x7a28),
    (0x123768, 0x0080),
    (0x12376a, 0x1840),
    (0x12376c, 0x6800),
    (0x124082, 0x7a28),
    (0x124084, 0x8288),
    (0x124170, 0x7a28),
    (0x124172, 0x3001),
    (0x124174, 0x7228),
    (0x12417a, 0x2804),
    (0x12417c, 0xd802),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub first_targets: u8,
    pub mode: u8,
    pub second_targets: u8,
    pub reserved: u8,
}
impl Control {
    pub fn from_word(word: u16) -> Self {
        Self {
            first_targets: (word & 63) as u8,
            mode: ((word >> 6) & 3) as u8,
            second_targets: ((word >> 8) & 63) as u8,
            reserved: (word >> 14) as u8,
        }
    }
    pub fn word(&self) -> u16 {
        u16::from(self.first_targets)
            | (u16::from(self.mode) << 6)
            | (u16::from(self.second_targets) << 8)
            | (u16::from(self.reserved) << 14)
    }
    fn valid(&self) -> bool {
        self.first_targets < 64 && self.mode < 4 && self.second_targets < 64 && self.reserved < 4
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coefficients {
    pub first: u8,
    pub first_reserved: u8,
    pub second: u8,
    pub second_reserved: u8,
}
impl Coefficients {
    pub fn from_word(word: u16) -> Self {
        Self {
            first: (word & 31) as u8,
            first_reserved: ((word >> 5) & 7) as u8,
            second: ((word >> 8) & 31) as u8,
            second_reserved: (word >> 13) as u8,
        }
    }
    pub fn word(&self) -> u16 {
        u16::from(self.first)
            | (u16::from(self.first_reserved) << 5)
            | (u16::from(self.second) << 8)
            | (u16::from(self.second_reserved) << 13)
    }
    fn valid(&self) -> bool {
        self.first < 32 && self.second < 32 && self.first_reserved < 8 && self.second_reserved < 8
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub control: Control,
    pub initial: Coefficients,
    pub command_count: u16,
}
impl Header {
    pub fn bytes(&self) -> [u8; 6] {
        let control = self.control.word().to_le_bytes();
        let initial = self.initial.word().to_le_bytes();
        let count = self.command_count.to_le_bytes();
        [
            control[0], control[1], initial[0], initial[1], count[0], count[1],
        ]
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.bytes().get(offset).copied()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    /// Raw ffff is the original immediate command. No duration or coefficient clamp.
    pub duration: u16,
    pub first: u8,
    pub second: u8,
}
impl Command {
    pub fn bytes(&self) -> [u8; 4] {
        let duration = self.duration.to_le_bytes();
        [duration[0], duration[1], self.first, self.second]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub source_offset: u32,
    pub header: Header,
    pub commands: Vec<Command>,
}
impl Program {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        if offset < 6 {
            self.header.byte(offset)
        } else {
            let at = offset - 6;
            self.commands.get(at / 4).map(|c| c.bytes()[at % 4])
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub source_offset: u32,
    /// Stable IDs into this resource's canonical program catalog.
    pub programs: Vec<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlendPrograms {
    pub programs: Vec<Program>,
    pub producers: Vec<Producer>,
}
impl BlendPrograms {
    pub fn program_word(&self, id: u16) -> Option<u32> {
        let at = self.programs.get(usize::from(id))?.source_offset;
        (at < 0x1000000).then(|| 0x08000000 + at)
    }
    pub fn producer_byte(&self, table: usize, offset: usize) -> Option<u8> {
        let id = *self.producers.get(table)?.programs.get(offset / 4)?;
        Some(self.program_word(id)?.to_le_bytes()[offset % 4])
    }
    fn validate_shape(&self) -> Result<(), String> {
        if self.programs.len() != PROGRAM_LAYOUTS.len()
            || self.producers.len() != PRODUCER_LAYOUTS.len()
        {
            return Err("Blend program/producer catalog count mismatch".into());
        }
        for (program, (offset, count)) in self.programs.iter().zip(PROGRAM_LAYOUTS) {
            if program.source_offset != offset
                || program.header.command_count != count
                || program.commands.len() != usize::from(count)
                || !program.header.control.valid()
                || !program.header.initial.valid()
            {
                return Err("Blend program source/header/command shape mismatch".into());
            }
        }
        for (producer, (offset, count)) in self.producers.iter().zip(PRODUCER_LAYOUTS) {
            if producer.source_offset != offset
                || producer.programs.len() != count
                || producer
                    .programs
                    .iter()
                    .any(|id| self.program_word(*id).is_none())
            {
                return Err("Blend producer source/shape/program reference mismatch".into());
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.source_spans().map(|_| ())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate_shape()?;
        let mut fields = BTreeMap::new();
        let mut add = |offset: u32, bytes: &[u8]| -> Result<(), String> {
            for (i, byte) in bytes.iter().enumerate() {
                let at = offset
                    .checked_add(i as u32)
                    .filter(|at| *at < 0x1000000)
                    .ok_or_else(|| "Blend source outside ROM".to_string())?;
                if fields.insert(at, *byte).is_some() {
                    return Err("Overlapping blend fields".into());
                }
            }
            Ok(())
        };
        for program in &self.programs {
            add(program.source_offset, &program.header.bytes())?;
            for (i, command) in program.commands.iter().enumerate() {
                add(program.source_offset + 6 + 4 * i as u32, &command.bytes())?;
            }
        }
        for producer in &self.producers {
            for (i, id) in producer.programs.iter().enumerate() {
                let word = self
                    .program_word(*id)
                    .ok_or("Unknown blend producer program")?;
                add(producer.source_offset + 4 * i as u32, &word.to_le_bytes())?;
            }
        }
        let mut spans: Vec<(u32, Vec<u8>, String)> = Vec::new();
        for (at, byte) in fields {
            if let Some((offset, bytes, _)) = spans.last_mut() {
                if *offset + bytes.len() as u32 == at {
                    bytes.push(byte);
                    continue;
                }
            }
            let role = format!("graphics/blend-programs/range/{}", spans.len());
            spans.push((at, vec![byte], role));
        }
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Decoded blend programs differ from authenticated source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: BlendPrograms,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Blend programs require exact European source ROM".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for &(at, expected) in SOURCE_CONTRACT {
            if half(at as usize) != expected {
                return Err("Blend reader/producer contract mismatch".into());
            }
        }
        let mut programs = Vec::new();
        for (source_offset, count) in PROGRAM_LAYOUTS {
            let at = source_offset as usize;
            if half(at + 4) != count || rom.get(at..at + 6 + 4 * usize::from(count)).is_none() {
                return Err("Blend command source extent mismatch".into());
            }
            programs.push(Program {
                source_offset,
                header: Header {
                    control: Control::from_word(half(at)),
                    initial: Coefficients::from_word(half(at + 2)),
                    command_count: count,
                },
                commands: (0..usize::from(count))
                    .map(|i| {
                        let a = at + 6 + 4 * i;
                        Command {
                            duration: half(a),
                            first: rom[a + 2],
                            second: rom[a + 3],
                        }
                    })
                    .collect(),
            });
        }
        let mut producers = Vec::new();
        for (source_offset, count) in PRODUCER_LAYOUTS {
            let mut ids = Vec::new();
            for i in 0..count {
                let ptr = word(source_offset as usize + 4 * i);
                let id = PROGRAM_LAYOUTS
                    .iter()
                    .position(|(offset, _)| 0x08000000 + *offset == ptr)
                    .ok_or("Unknown blend producer program pointer")?;
                ids.push(id as u16);
            }
            producers.push(Producer {
                source_offset,
                programs: ids,
            });
        }
        let decoded = BlendPrograms {
            programs,
            producers,
        };
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Blend source roundtrip mismatch".into());
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
