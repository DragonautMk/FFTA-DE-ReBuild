//! Exact static EUR item fields. Original property rules and live bonuses stay guest code.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e0bd0;
pub const END_OFFSET: u32 = 0x6e3ad0;
pub const RECORD_BYTES: usize = 32;
pub const RECORD_COUNT: usize = 376;
pub const PROPERTY_READER: u32 = 0x080ce1ec;
pub const PROPERTY_TARGETS: [u32; 19] = [
    0x080ce260, 0x080ce264, 0x080ce268, 0x080ce26c, 0x080ce270, 0x080ce274, 0x080ce278, 0x080ce27c,
    0x080ce280, 0x080ce284, 0x080ce288, 0x080ce2a8, 0x080ce2cc, 0x080ce2ec, 0x080ce322, 0x080ce328,
    0x080ce32e, 0x080ce334, 0x080ce33a,
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub halfwords_00_06: [u16; 4],
    pub bytes_08_0f: [u8; 8],
    pub signed_10_17: [i8; 8],
    pub bytes_18_1f: [u8; 8],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0..=7 => Some(self.halfwords_00_06[offset / 2].to_le_bytes()[offset % 2]),
            8..=15 => Some(self.bytes_08_0f[offset - 8]),
            16..=23 => Some(self.signed_10_17[offset - 16] as u8),
            24..=31 => Some(self.bytes_18_1f[offset - 24]),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|offset| self.byte(offset).unwrap())
    }
    /// Raw field only: properties10..13 still require the original live rules.
    pub fn raw_property(&self, property: u8) -> u32 {
        match property {
            0..=2 => u32::from(self.halfwords_00_06[property as usize]),
            3..=8 => u32::from(self.bytes_08_0f[property as usize - 3]),
            9 => u32::from(u16::from_le_bytes(
                self.bytes_08_0f[6..8].try_into().unwrap(),
            )),
            10..=17 => i32::from(self.signed_10_17[property as usize - 10]) as u32,
            18 => u32::from(self.bytes_18_1f[5]),
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Items {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Items {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR item record extent".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Item records differ from authenticated EUR source".into());
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
    pub decoded: Items,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Static item records require exact EUR ROM; USA pending".into());
        }
        // Checked code roots, including the independently rooted following table.
        for (reader, literal, target) in [
            (0xce1f8, 0xce20c, SOURCE_OFFSET),
            (0xcf888, 0xcf8dc, SOURCE_OFFSET),
            (0x72134, 0x72150, SOURCE_OFFSET),
            (0x72140, 0x72154, END_OFFSET),
        ] {
            let h = half(rom, reader);
            if h & 0xf800 != 0x4800
                || ((reader + 4) & !3) + usize::from(h & 255) * 4 != literal
                || word(rom, literal) != 0x08000000 + target
            {
                return Err("Item original root literal mismatch".into());
            }
        }
        for (at, value) in [
            (0xce1f6, 0x0169),
            (0xcf88a, 0x0179),
            (0x72132, 0x0148),
            (0xcf872, 0x0049),
            (0xcf876, 0x200e),
            (0xcec72, 0x230e),
            (0xce1fc, 0x2a12),
            (0xccf0c, 0x7800),
        ] {
            if half(rom, at) != value {
                return Err("Item original stride/bound/property mismatch".into());
            }
        }
        for (at, value) in [(0xcf86c, 361), (0xcec88, 361), (0xcec8c, 375)] {
            if word(rom, at) != value {
                return Err("Item original enumeration bound mismatch".into());
            }
        }
        if PROPERTY_TARGETS
            .iter()
            .enumerate()
            .any(|(i, &target)| word(rom, 0xce214 + 4 * i) != target)
        {
            return Err("Item original property branch mismatch".into());
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Items {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(RECORD_BYTES)
                .map(|row| Record {
                    halfwords_00_06: std::array::from_fn(|i| {
                        u16::from_le_bytes(row[2 * i..2 * i + 2].try_into().unwrap())
                    }),
                    bytes_08_0f: row[8..16].try_into().unwrap(),
                    signed_10_17: std::array::from_fn(|i| row[16 + i] as i8),
                    bytes_18_1f: row[24..32].try_into().unwrap(),
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
                role: "item-records/eur-fixed".into(),
            }],
        })
    }
}
