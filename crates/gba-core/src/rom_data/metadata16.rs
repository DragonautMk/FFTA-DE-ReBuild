//! Lossless EUR sixteen-byte metadata selected by original live linked records.
//! Semantic family/opaque fields remain unnamed until independently established.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e4c64;
pub const END_OFFSET: u32 = 0x6e5464;
pub const RECORD_BYTES: usize = 16;
pub const RECORD_COUNT: usize = 128;
pub const REFERENCES: &[(u32, u32)] = &[
    (0x161e6, 0x161fc),
    (0x1639a, 0x163a8),
    (0x1aa92, 0x1aaac),
    (0x39070, 0x39094),
    (0x392bc, 0x392e0),
    (0x4c7da, 0x4c830),
    (0x4cb56, 0x4cba8),
    (0x4d97c, 0x4d9b8),
    (0x51b80, 0x51b98),
    (0x52810, 0x5282c),
    (0x621a4, 0x621c0),
    (0x621f6, 0x62214),
    (0x62410, 0x62428),
    (0x69950, 0x69968),
    (0x69d28, 0x69d44),
    (0xd5e6e, 0xd5e9c),
    (0xd5e80, 0xd5e9c),
    (0xd5eee, 0xd5f2c),
    (0xd5efc, 0xd5f2c),
    (0xd6758, 0xd676c),
    (0xd6796, 0xd67a8),
    (0xd6828, 0xd6860),
    (0xd6844, 0xd6860),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub alias_u16: u16,
    /// Raw bytes retain unknown fields and original unsigned reader semantics.
    pub bytes_02_0f: [u8; 14],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0..=1 => Some(self.alias_u16.to_le_bytes()[offset]),
            2..=15 => Some(self.bytes_02_0f[offset - 2]),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Original bounded display-index calculation; no text/icon meaning assumed.
    pub fn display_index(&self) -> u32 {
        match self.bytes_02_0f[0] {
            0 => 0,
            v => 394 + u32::from(v),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata16 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Metadata16 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR metadata16 extent/count".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata16 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn literal(rom: &[u8], pc: u32, at: u32, target: u32) -> Result<(), String> {
    let h = half(rom, pc);
    let word = u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap());
    if h & 0xf800 != 0x4800
        || ((pc + 4) & !3) + u32::from(h & 255) * 4 != at
        || word != 0x08000000 + target
    {
        return Err("Metadata16 original literal mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Metadata16,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata16 requires exact EUR ROM; USA pending".into());
        }
        for &(pc, at) in REFERENCES {
            literal(rom, pc, at, SOURCE_OFFSET)?;
        }
        literal(rom, 0xcbeda, 0xcbef0, END_OFFSET)?;
        for (pc, value) in [
            (0x161e4, 0x0120),
            (0x161ea, 0x7888),
            (0x39076, 0x8805),
            (0x39078, 0x0128),
            (0xd5e72, 0x0109),
            (0xd5e76, 0x7b48),
            (0xd5ef4, 0x7b87),
            (0xd677a, 0x7a48),
            (0xd677e, 0x7a88),
            (0xd6782, 0x7ac8),
            (0xd6786, 0x7b08),
            (0xd678a, 0x0600),
            (0xd678c, 0x0e05),
            (0xd682c, 0x79c8),
            (0xd6830, 0x4108),
        ] {
            if half(rom, pc) != value {
                return Err("Metadata16 original stride/field reader mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Metadata16 {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(RECORD_BYTES)
                .map(|r| Record {
                    alias_u16: u16::from_le_bytes(r[..2].try_into().unwrap()),
                    bytes_02_0f: r[2..].try_into().unwrap(),
                })
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(bytes),
                role: "metadata16/eur-fixed".into(),
            }],
        })
    }
}
