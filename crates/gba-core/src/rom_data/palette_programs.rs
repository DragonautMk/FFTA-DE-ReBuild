//! Original palette programs and bounded producer projections. Guest task state stays live.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Fixed addresses are metadata from the retained producer proofs, never payload bytes.
pub const PROGRAM_OFFSETS: [u32; 230] = [
    0xb45630, 0xb45680, 0xb457d8, 0xb45828, 0xb45878, 0xb459d8, 0xb45b38, 0xb45d38, 0xb45ddc,
    0xb45e20, 0xb45e68, 0xb45ea4, 0xb45edc, 0xb45eec, 0xb45f4c, 0xb45fe8, 0xb46198, 0xb461d8,
    0xb461f0, 0xb46218, 0xb46298, 0xb462b4, 0xb462d8, 0xb462f0, 0xb46358, 0xb463c8, 0xb463e0,
    0xb4640c, 0xb46460, 0xb46470, 0xb46480, 0xb464d0, 0xb46518, 0xb46560, 0xb465dc, 0xb46608,
    0xb46618, 0xb4663c, 0xb466cc, 0xb466dc, 0xb466f8, 0xb46708, 0xb46718, 0xb46740, 0xb467a0,
    0xb468b8, 0xb468cc, 0xb468e0, 0xb468f4, 0xb46908, 0xb46928, 0xb4693c, 0xb46950, 0xb46964,
    0xb469ac, 0xb469c4, 0xb46c44, 0xb46c58, 0xb46c6c, 0xb46c80, 0xb46ca0, 0xb46e5c, 0xb46eb4,
    0xb46ed4, 0xb46f54, 0xb46f7c, 0xb470bc, 0xb4710c, 0xb47164, 0xb471cc, 0xb471f4, 0xb47220,
    0xb47254, 0xb4728c, 0xb472a0, 0xb472d4, 0xb47410, 0xb47514, 0xb47528, 0xb47548, 0xb4756c,
    0xb47588, 0xb475b4, 0xb475fc, 0xb47634, 0xb47794, 0xb477b0, 0xb4788c, 0xb47910, 0xb47920,
    0xb47970, 0xb47a48, 0xb47a74, 0xb47a88, 0xb47b6c, 0xb47bbc, 0xb47c54, 0xb47ca4, 0xb47cf4,
    0xb47d44, 0xb47d94, 0xb47de4, 0xb47e34, 0xb47e84, 0xb47ed4, 0xb47f24, 0xb47f38, 0xb47f50,
    0xb4821c, 0xb482cc, 0xb48394, 0xb4840c, 0xb48468, 0xb484a4, 0xb48554, 0xb4858c, 0xb486a8,
    0xb486f8, 0xb48724, 0xb48744, 0xb48754, 0xb48764, 0xb487fc, 0xb48894, 0xb488e8, 0xb48944,
    0xb489a0, 0xb489f4, 0xb48a8c, 0xb48b08, 0xb48b80, 0xb48ee4, 0xb48ef4, 0xb48f0c, 0xb48f44,
    0xb48f58, 0xb48f6c, 0xb48f7c, 0xb48f9c, 0xb49028, 0xb49050, 0xb4907c, 0xb490a4, 0xb490e4,
    0xb49118, 0xb491bc, 0xb491cc, 0xb4922c, 0xb49260, 0xb49494, 0xb494fc, 0xb495c8, 0xb49618,
    0xb49674, 0xb49688, 0xb496b8, 0xb496cc, 0xb49794, 0xb4984c, 0xb49860, 0xb498e8, 0xb498fc,
    0xb49914, 0xb49948, 0xb49988, 0xb499f4, 0xb49a3c, 0xb49ae0, 0xb49b54, 0xb49cb8, 0xb49cf0,
    0xb49d4c, 0xb49dc0, 0xb49dd4, 0xb49e50, 0xb49e64, 0xb49fb8, 0xb4a064, 0xb4a0d8, 0xb4a134,
    0xb4a160, 0xb4a1bc, 0xb4a320, 0xb4a330, 0xb4a554, 0xb455f4, 0xb45608, 0xb4561c, 0xb456d0,
    0xb4576c, 0xb45790, 0xb457b4, 0xb469ec, 0xb46a50, 0xb46ab4, 0xb46b18, 0xb46b7c, 0xb46be0,
    0xb46fcc, 0xb46fe0, 0xb46ff4, 0xb47008, 0xb4701c, 0xb47030, 0xb47044, 0xb47058, 0xb4706c,
    0xb47080, 0xb47094, 0xb470a8, 0xb47424, 0xb47474, 0xb474c4, 0xb4787c, 0xb47900, 0xb497f0,
    0xb4a010, 0xb4a038, 0xb4a084, 0xb4a0ac, 0xb4a5b0, 0xb4a5dc, 0xb4a608, 0xb4a634, 0xb4a660,
    0xb4a68c, 0xb4a6b8, 0xb4a6e4, 0xb4a710, 0xb4a73c,
];
pub const PRODUCER_LAYOUTS: [(u32, usize, bool); 25] = [
    (0x39dee4, 3, true),
    (0x3a0990, 3, true),
    (0x3a5208, 2, true),
    (0x14d680, 3, false),
    (0x14d68c, 3, false),
    (0x39ab94, 5, false),
    (0x39aba8, 5, false),
    (0x39b058, 2, false),
    (0x39bcc4, 2, false),
    (0x39cc7c, 3, false),
    (0x39cc88, 3, false),
    (0x39cf28, 3, false),
    (0x39d184, 3, false),
    (0x39d190, 3, false),
    (0x39d7fc, 3, false),
    (0x39e974, 7, false),
    (0x39fa48, 8, false),
    (0x3a1338, 3, false),
    (0x3a19b0, 2, false),
    (0x3a4a50, 15, false),
    (0x3a4d64, 8, false),
    (0x3a6054, 6, false),
    (0x3a606c, 6, false),
    (0x3ac748, 5, false),
    (0x3ac75c, 5, false),
];
const SOURCE_CONTRACT: &[(u32, u16)] = &[
    (0xd7b4e, 0x8971),
    (0xd7b5c, 0x7921),
    (0xd7b5e, 0x7960),
    (0xd7b68, 0x7820),
    (0xd7d50, 0x790a),
    (0xd7d52, 0x7948),
    (0xd7d5e, 0x7808),
    (0xd7fe4, 0x7810),
    (0xd7fe8, 0x7851),
    (0xd7ffa, 0x7881),
    (0xd7ffe, 0x7901),
    (0xd8000, 0x7940),
    (0xd80b2, 0x78c9),
    (0xe55d2, 0x2100),
    (0xe55d8, 0x7169),
    (0xe562c, 0x2001),
    (0xe562e, 0x7168),
    (0xe56bc, 0x2002),
    (0xe5736, 0x2004),
    (0xe5738, 0x7168),
    (0xe574c, 0x2003),
    (0xe574e, 0x7168),
    (0xe6cc4, 0x7d38),
    (0xe6cc6, 0x3862),
    (0xe6cc8, 0x70b0),
    (0xe8388, 0x8ab0),
    (0xe838a, 0x7028),
    (0xe855c, 0x7d30),
    (0xe855e, 0x3809),
    (0xe8560, 0x7028),
    (0xe85ea, 0x7828),
    (0xe85ec, 0x8288),
    (0xeb86c, 0x8ab8),
    (0xeb86e, 0x7028),
    (0xebac4, 0x3818),
    (0xebac6, 0x7028),
    (0xebace, 0x7029),
    (0xebbe6, 0x7828),
    (0xebbe8, 0x8288),
    (0xebca4, 0x8ab8),
    (0xebca6, 0x7028),
    (0xebefc, 0x381b),
    (0xebefe, 0x7028),
    (0xebf06, 0x7029),
    (0xec01e, 0x7828),
    (0xec020, 0x8288),
    (0xec0dc, 0x8ab8),
    (0xec0de, 0x7028),
    (0xec33c, 0x381e),
    (0xec33e, 0x7028),
    (0xec346, 0x7029),
    (0xec4a2, 0x7828),
    (0xec4a4, 0x8288),
    (0xed99e, 0x2854),
    (0xed9a2, 0x2002),
    (0xed9a8, 0x3827),
    (0xed9aa, 0x7020),
    (0xf28d0, 0x8b28),
    (0xf28d2, 0x7020),
    (0xf29d8, 0x4641),
    (0xf29da, 0x8301),
    (0xf6ba0, 0x2000),
    (0xf6ba4, 0x2001),
    (0xf6ba8, 0x2002),
    (0xf6bac, 0x2003),
    (0xf6bb0, 0x2004),
    (0xf6bb4, 0x2005),
    (0xf6bb8, 0x2006),
    (0xf6bbc, 0x2007),
    (0xf6bbe, 0x7278),
    (0xf7a4a, 0x2054),
    (0xf7a4c, 0x8288),
    (0xfc98e, 0x7d08),
    (0xfc990, 0x3871),
    (0xfc992, 0x71b8),
    (0xfe306, 0x7033),
    (0xfe30e, 0x28b8),
    (0xfe312, 0x2001),
    (0xfe314, 0x7030),
    (0x108eb2, 0x7d38),
    (0x108eb4, 0x3016),
    (0x108eb6, 0x70b0),
    (0x109030, 0x2500),
    (0x10903e, 0x78b1),
    (0x109040, 0x0048),
    (0x109042, 0x1840),
    (0x109044, 0x0080),
    (0x10905c, 0x2d02),
    (0x10905e, 0xd9ed),
    (0x1091be, 0x8ab0),
    (0x1091c0, 0x7020),
    (0x1094c8, 0x7d38),
    (0x1094ca, 0x3011),
    (0x1094cc, 0x7230),
    (0x10975e, 0x7a30),
    (0x109760, 0x8290),
    (0x10c172, 0x8b30),
    (0x10c174, 0x7028),
    (0x10c244, 0x387c),
    (0x10c246, 0x71e0),
    (0x10c25c, 0x79e1),
    (0x10c25e, 0x0049),
    (0x10c260, 0x71e1),
    (0x10c268, 0x2801),
    (0x10c26a, 0xd801),
    (0x10c26c, 0x1c48),
    (0x10c26e, 0x71e0),
    (0x10c3c4, 0x79e0),
    (0x10c3c6, 0x8330),
    (0x1234e6, 0x8a88),
    (0x1234e8, 0x7228),
    (0x124082, 0x7a28),
    (0x124084, 0x8288),
    (0x124170, 0x7a28),
    (0x124172, 0x3001),
    (0x124174, 0x7228),
    (0x12417a, 0x2804),
    (0x12417c, 0xd802),
    (0x14a582, 0x8807),
    (0x14b728, 0x8800),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub first: u8,
    pub flags: u8,
    pub width: u8,
    pub command_count: u8,
    pub initial_delay: u16,
}
impl Header {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0 => Some(self.first),
            1 => Some(self.flags),
            2 => Some(self.width),
            3 => Some(self.command_count),
            4 | 5 => Some(self.initial_delay.to_le_bytes()[offset - 4]),
            _ => None,
        }
    }
    pub fn first_color(&self) -> u16 {
        u16::from(self.first) + if self.flags & 2 != 0 { 256 } else { 0 }
    }
    /// The public queued-copy wrapper wraps u16 before clamping its end to511.
    /// Direct copies instead use the raw header width, including zero.
    pub fn color_words(&self, queued: bool) -> usize {
        if !queued {
            return usize::from(self.width);
        }
        let start = self.first_color();
        let end = start
            .wrapping_add(u16::from(self.width))
            .wrapping_sub(1)
            .min(511);
        if end < start {
            0
        } else {
            usize::from(end - start) + 1
        }
    }
    fn bytes(&self) -> Vec<u8> {
        let mut out = vec![self.first, self.flags, self.width, self.command_count];
        out.extend(self.initial_delay.to_le_bytes());
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Parameter {
    Unused,
    ColorWords {
        colors: Vec<u16>,
    },
    Components {
        values: [i16; 3],
    },
    Halfword {
        value: u16,
    },
    HalfwordTwoBytes {
        value: u16,
        first: u8,
        second: u8,
    },
    Byte {
        value: u8,
    },
    TwoBytes {
        first: u8,
        second: u8,
    },
    ColorCycle {
        count: u8,
        direction: u8,
        colors: Vec<u16>,
    },
}
impl Parameter {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match self {
            Self::Unused => None,
            Self::ColorWords { colors } => {
                colors.get(offset / 2).map(|v| v.to_le_bytes()[offset % 2])
            }
            Self::Components { values } => {
                values.get(offset / 2).map(|v| v.to_le_bytes()[offset % 2])
            }
            Self::Halfword { value } => value.to_le_bytes().get(offset).copied(),
            Self::HalfwordTwoBytes {
                value,
                first,
                second,
            } => match offset {
                0 | 1 => Some(value.to_le_bytes()[offset]),
                2 => Some(*first),
                3 => Some(*second),
                _ => None,
            },
            Self::Byte { value } => (offset == 0).then_some(*value),
            Self::TwoBytes { first, second } => [*first, *second].get(offset).copied(),
            Self::ColorCycle {
                count,
                direction,
                colors,
            } => match offset {
                0 => Some(*count),
                1 => Some(*direction),
                _ => colors
                    .get((offset - 2) / 2)
                    .map(|v| v.to_le_bytes()[(offset - 2) % 2]),
            },
        }
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        match self {
            Self::Unused => vec![],
            Self::ColorWords { colors } => colors.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Self::Components { values } => values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            Self::Halfword { value } => value.to_le_bytes().to_vec(),
            Self::HalfwordTwoBytes {
                value,
                first,
                second,
            } => {
                let mut out = value.to_le_bytes().to_vec();
                out.extend([*first, *second]);
                out
            }
            Self::Byte { value } => vec![*value],
            Self::TwoBytes { first, second } => vec![*first, *second],
            Self::ColorCycle {
                count,
                direction,
                colors,
            } => {
                let mut out = vec![*count, *direction];
                out.extend(colors.iter().flat_map(|v| v.to_le_bytes()));
                out
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub opcode: u8,
    pub mode: u8,
    pub duration: u16,
    pub relative_parameter: u16,
    pub parameter: Parameter,
}
impl Command {
    pub fn record_bytes(&self) -> [u8; 6] {
        let d = self.duration.to_le_bytes();
        let p = self.relative_parameter.to_le_bytes();
        [self.opcode, self.mode, d[0], d[1], p[0], p[1]]
    }
    fn validate(&self, header: &Header) -> bool {
        match (&self.parameter, self.opcode) {
            (Parameter::Unused, 1 | 2 | 13) => true,
            (Parameter::Unused, 12) => self.mode != 0,
            (Parameter::ColorWords { colors }, 0) => {
                colors.len() == header.color_words(self.mode != 0)
            }
            (Parameter::Components { .. }, 3 | 6) => true,
            (Parameter::Components { .. }, 12) => self.mode == 0,
            (Parameter::Halfword { .. }, 4 | 5) => true,
            (Parameter::HalfwordTwoBytes { .. }, 7) => true,
            (Parameter::Byte { .. }, 8 | 9) => true,
            (Parameter::TwoBytes { .. }, 10) => true,
            (Parameter::ColorCycle { count, colors, .. }, 11) => {
                colors.len() == usize::from((*count).max(1))
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub source_offset: u32,
    pub header: Header,
    pub commands: Vec<Command>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListEntry {
    pub program: u16,
    pub delay_override: u16,
    pub reserved: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ProducerData {
    Pointers { programs: Vec<Option<u16>> },
    List { entries: Vec<ListEntry> },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub source_offset: u32,
    pub data: ProducerData,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PalettePrograms {
    pub programs: Vec<Program>,
    pub producers: Vec<Producer>,
}
impl PalettePrograms {
    pub fn producer_byte(&self, table: usize, offset: usize) -> Option<u8> {
        match &self.producers.get(table)?.data {
            ProducerData::Pointers { programs } => {
                let word = match programs.get(offset / 4)? {
                    Some(id) => self.program_word(*id)?,
                    None => 0,
                };
                Some(word.to_le_bytes()[offset % 4])
            }
            ProducerData::List { entries } => {
                let e = entries.get(offset / 8)?;
                Some(match offset % 8 {
                    n @ 0..=3 => self.program_word(e.program)?.to_le_bytes()[n],
                    n @ 4..=5 => e.delay_override.to_le_bytes()[n - 4],
                    n => e.reserved.to_le_bytes()[n - 6],
                })
            }
        }
    }
    pub fn program_word(&self, id: u16) -> Option<u32> {
        let at = self.programs.get(usize::from(id))?.source_offset;
        (at < 0x1000000).then(|| 0x08000000 + at)
    }
    pub fn producer_bytes(&self, producer: &Producer) -> Result<Vec<u8>, String> {
        let word = |id| {
            self.program_word(id)
                .ok_or_else(|| "Palette producer references an unknown program".to_string())
        };
        let mut out = Vec::new();
        match &producer.data {
            ProducerData::Pointers { programs } => {
                for id in programs {
                    out.extend(
                        match id {
                            Some(id) => word(*id)?,
                            None => 0,
                        }
                        .to_le_bytes(),
                    );
                }
            }
            ProducerData::List { entries } => {
                for e in entries {
                    out.extend(word(e.program)?.to_le_bytes());
                    out.extend(e.delay_override.to_le_bytes());
                    out.extend(e.reserved.to_le_bytes());
                }
            }
        }
        Ok(out)
    }
    fn validate_shape(&self) -> Result<(), String> {
        if self.programs.len() != PROGRAM_OFFSETS.len()
            || self.producers.len() != PRODUCER_LAYOUTS.len()
        {
            return Err("Palette program/producer catalog count mismatch".into());
        }
        for (p, expected) in self.programs.iter().zip(PROGRAM_OFFSETS) {
            if p.source_offset != expected
                || p.header.command_count == 0
                || p.commands.len() != usize::from(p.header.command_count)
                || p.commands.iter().any(|c| !c.validate(&p.header))
            {
                return Err(
                    "Palette program source/header/command/parameter shape mismatch".into(),
                );
            }
        }
        for (p, (offset, count, list)) in self.producers.iter().zip(PRODUCER_LAYOUTS) {
            if p.source_offset != offset {
                return Err("Palette producer source/order mismatch".into());
            }
            let valid = match &p.data {
                ProducerData::List { entries } => list && entries.len() == count,
                ProducerData::Pointers { programs } => {
                    !list
                        && programs.len() == count
                        && programs
                            .iter()
                            .enumerate()
                            .all(|(i, id)| id.is_none() == (offset == 0x14d680 && i == 2))
                }
            };
            if !valid {
                return Err("Palette producer shape/null membership mismatch".into());
            }
            self.producer_bytes(p)?;
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.source_spans().map(|_| ())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate_shape()?;
        let mut fields = BTreeMap::new();
        let mut add = |start: u32, bytes: Vec<u8>| -> Result<(), String> {
            for (i, byte) in bytes.into_iter().enumerate() {
                let at = start
                    .checked_add(i as u32)
                    .filter(|a| *a < 0x1000000)
                    .ok_or_else(|| "Palette source outside ROM".to_string())?;
                if fields.insert(at, byte).is_some() {
                    return Err("Overlapping palette program/producer fields".into());
                }
            }
            Ok(())
        };
        for p in &self.programs {
            add(p.source_offset, p.header.bytes())?;
            for (i, c) in p.commands.iter().enumerate() {
                add(
                    p.source_offset + 6 + 6 * i as u32,
                    c.record_bytes().to_vec(),
                )?;
                add(
                    p.source_offset + u32::from(c.relative_parameter),
                    c.parameter.source_bytes(),
                )?;
            }
        }
        for p in &self.producers {
            add(p.source_offset, self.producer_bytes(p)?)?;
        }
        let mut spans: Vec<(u32, Vec<u8>, String)> = Vec::new();
        for (at, byte) in fields {
            if let Some((start, bytes, _)) = spans.last_mut() {
                if *start + bytes.len() as u32 == at {
                    bytes.push(byte);
                    continue;
                }
            }
            let role = format!("graphics/palette-programs/range/{}", spans.len());
            spans.push((at, vec![byte], role));
        }
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Decoded palette programs differ from authenticated source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: PalettePrograms,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Palette programs require exact European source ROM".into());
        }
        let u16_at = |a: usize| u16::from_le_bytes(rom[a..a + 2].try_into().unwrap());
        let u32_at = |a: usize| u32::from_le_bytes(rom[a..a + 4].try_into().unwrap());
        let words = |a: usize, n: usize| (0..n).map(|i| u16_at(a + 2 * i)).collect();
        for &(at, expected) in SOURCE_CONTRACT {
            if u16_at(at as usize) != expected {
                return Err("Palette original reader/producer contract mismatch".into());
            }
        }
        let mut programs = Vec::new();
        for source_offset in PROGRAM_OFFSETS {
            let at = source_offset as usize;
            let header = Header {
                first: rom[at],
                flags: rom[at + 1],
                width: rom[at + 2],
                command_count: rom[at + 3],
                initial_delay: u16_at(at + 4),
            };
            let mut commands = Vec::new();
            for i in 0..usize::from(header.command_count) {
                let ca = at + 6 + 6 * i;
                let opcode = rom[ca];
                let mode = rom[ca + 1];
                let relative_parameter = u16_at(ca + 4);
                let pa = at + usize::from(relative_parameter);
                let parameter_len = match opcode {
                    0 => 2 * header.color_words(mode != 0),
                    1 | 2 | 13 => 0,
                    12 if mode != 0 => 0,
                    3 | 6 | 12 => 6,
                    4 | 5 | 10 => 2,
                    7 => 4,
                    8 | 9 => 1,
                    11 => {
                        2 + 2 * usize::from(*rom.get(pa).ok_or("Palette count outside ROM")?).max(1)
                    }
                    _ => return Err("Unknown palette program opcode".into()),
                };
                if parameter_len != 0 && rom.get(pa..pa + parameter_len).is_none() {
                    return Err("Palette parameter source outside ROM".into());
                }
                let parameter = match opcode {
                    0 => Parameter::ColorWords {
                        colors: words(pa, header.color_words(mode != 0)),
                    },
                    1 | 2 | 13 => Parameter::Unused,
                    12 if mode != 0 => Parameter::Unused,
                    3 | 6 | 12 => Parameter::Components {
                        values: [
                            u16_at(pa) as i16,
                            u16_at(pa + 2) as i16,
                            u16_at(pa + 4) as i16,
                        ],
                    },
                    4 | 5 => Parameter::Halfword { value: u16_at(pa) },
                    7 => Parameter::HalfwordTwoBytes {
                        value: u16_at(pa),
                        first: rom[pa + 2],
                        second: rom[pa + 3],
                    },
                    8 | 9 => Parameter::Byte { value: rom[pa] },
                    10 => Parameter::TwoBytes {
                        first: rom[pa],
                        second: rom[pa + 1],
                    },
                    11 => Parameter::ColorCycle {
                        count: rom[pa],
                        direction: rom[pa + 1],
                        colors: words(pa + 2, usize::from(rom[pa].max(1))),
                    },
                    _ => return Err("Unknown palette program opcode".into()),
                };
                commands.push(Command {
                    opcode,
                    mode,
                    duration: u16_at(ca + 2),
                    relative_parameter,
                    parameter,
                });
            }
            programs.push(Program {
                source_offset,
                header,
                commands,
            });
        }
        let id_for = |word: u32| -> Result<u16, String> {
            PROGRAM_OFFSETS
                .iter()
                .position(|a| *a + 0x08000000 == word)
                .map(|i| i as u16)
                .ok_or_else(|| "Unknown palette producer program pointer".into())
        };
        let mut producers = Vec::new();
        for (source_offset, count, list) in PRODUCER_LAYOUTS {
            let at = source_offset as usize;
            let data = if list {
                let entries = (0..count)
                    .map(|i| {
                        let a = at + 8 * i;
                        Ok(ListEntry {
                            program: id_for(u32_at(a))?,
                            delay_override: u16_at(a + 4),
                            reserved: u16_at(a + 6),
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                ProducerData::List { entries }
            } else {
                let programs = (0..count)
                    .map(|i| match u32_at(at + 4 * i) {
                        0 => Ok(None),
                        word => id_for(word).map(Some),
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                ProducerData::Pointers { programs }
            };
            producers.push(Producer {
                source_offset,
                data,
            });
        }
        let decoded = PalettePrograms {
            programs,
            producers,
        };
        let spans = decoded.source_spans()?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in spans {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Palette program source roundtrip mismatch".into());
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
