//! Four fixed constructor sources and their base pointer; call arguments remain live.
use super::{default48, items, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const POINTER_OFFSET: u32 = 0x70cd58;
pub const SOURCE_OFFSET: u32 = 0x6eca78;
pub const RECORD_BYTES: usize = 48;
pub const RECORD_COUNT: usize = 4;
pub const END_OFFSET: u32 = SOURCE_OFFSET + (RECORD_BYTES * RECORD_COUNT) as u32;
pub const FIRST_ID: u8 = 20;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub byte00: u8,
    pub byte01: u8,
    pub bytes02_04: [u8; 3],
    pub arguments05_07: [u8; 3],
    pub halfwords08_28: [u16; 17],
    pub flags2a: u16,
    pub halfwords2c_2e: [u16; 2],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0 => self.byte00,
            1 => self.byte01,
            2..=4 => self.bytes02_04[offset - 2],
            5..=7 => self.arguments05_07[offset - 5],
            8..=41 => (self.halfwords08_28[(offset - 8) / 2] >> (offset % 2 * 8)) as u8,
            42..=43 => (self.flags2a >> (offset % 2 * 8)) as u8,
            44..=47 => (self.halfwords2c_2e[(offset - 44) / 2] >> (offset % 2 * 8)) as u8,
            _ => return None,
        })
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|offset| self.byte(offset).unwrap())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alternates {
    pub pointer_source_offset: u32,
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Alternates {
    pub fn validate(&self) -> Result<(), String> {
        if self.pointer_source_offset != POINTER_OFFSET
            || self.source_offset != SOURCE_OFFSET
            || self.records.len() != RECORD_COUNT
        {
            return Err("Invalid fixed EUR constructor-alternate sources".into());
        }
        for (i, record) in self.records.iter().enumerate() {
            if record.id != FIRST_ID + i as u8
                || record.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32
            {
                return Err("Invalid constructor-alternate ID/order/extent".into());
            }
        }
        Ok(())
    }
    pub fn validate_items(&self, owner: &items::Items) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if self.records.iter().any(|record| {
            record.halfwords08_28[..5]
                .iter()
                .any(|&item| usize::from(item) >= owner.records.len())
        }) {
            return Err("Constructor-alternate equipment has no parsed item owner".into());
        }
        Ok(())
    }
    pub fn pointer_bytes(&self) -> [u8; 4] {
        (0x08000000 + self.source_offset).to_le_bytes()
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    /// Outside the four owned IDs is not an emulated fallback.
    pub fn record(&self, id: u32) -> Option<&Record> {
        self.records
            .get(usize::from((id as u8).checked_sub(FIRST_ID)?))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Constructor alternates differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
}
fn word(rom: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}
pub struct Catalog {
    pub decoded: Alternates,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        // Reuse authenticated constructor selection/patch fingerprints, not its oracle.
        default48::Catalog::from_rom(rom)?;
        let instruction = half(rom, 0x127e1a);
        if instruction != 0x4907
            || ((0x127e1a + 4) & !3) + usize::from(instruction & 255) * 4 != 0x127e38
            || word(rom, 0x127e38) + 4 != 0x08000000 + POINTER_OFFSET
            || word(rom, POINTER_OFFSET as usize) != 0x08000000 + SOURCE_OFFSET
            || word(rom, 0x127e40) != 0xfffffc40
        {
            return Err("Constructor-alternate pointer/index mismatch".into());
        }
        for (at, value) in [
            (0x127e10, 0x3814),
            (0x127e12, 0x0600),
            (0x127e14, 0x0e00),
            (0x127e1e, 0x0078),
            (0x127e20, 0x19c0),
            (0x127e22, 0x0100),
            (0x127e26, 0x18c0),
            (0x127e28, 0x6849),
            (0x127e30, 0x2230),
            (0xcd01a, 0x8820),
            (0xcd01c, 0x2106),
            (0xcd032, 0x8820),
            (0xcd034, 0x2103),
            (0xcd044, 0x2e04),
        ] {
            if half(rom, at) != value {
                return Err("Constructor-alternate original reader mismatch".into());
            }
        }
        let raw = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let records = raw
            .chunks_exact(RECORD_BYTES)
            .enumerate()
            .map(|(i, raw)| Record {
                id: FIRST_ID + i as u8,
                source_offset: SOURCE_OFFSET + (i * RECORD_BYTES) as u32,
                byte00: raw[0],
                byte01: raw[1],
                bytes02_04: raw[2..5].try_into().unwrap(),
                arguments05_07: raw[5..8].try_into().unwrap(),
                halfwords08_28: std::array::from_fn(|i| half(raw, 8 + i * 2)),
                flags2a: half(raw, 42),
                halfwords2c_2e: std::array::from_fn(|i| half(raw, 44 + i * 2)),
            })
            .collect();
        let decoded = Alternates {
            pointer_source_offset: POINTER_OFFSET,
            source_offset: SOURCE_OFFSET,
            records,
        };
        decoded.validate_items(&items::Catalog::from_rom(rom)?.decoded)?;
        Ok(Self {
            decoded,
            sources: vec![
                SourceRange {
                    offset: POINTER_OFFSET,
                    length: 4,
                    sha256: ffta_rom::digest(
                        &rom[POINTER_OFFSET as usize..POINTER_OFFSET as usize + 4],
                    ),
                    role: "constructor-alternates/eur-base-pointer".into(),
                },
                SourceRange {
                    offset: SOURCE_OFFSET,
                    length: raw.len() as u32,
                    sha256: ffta_rom::digest(raw),
                    role: "constructor-alternates/eur-records".into(),
                },
            ],
        })
    }
}
