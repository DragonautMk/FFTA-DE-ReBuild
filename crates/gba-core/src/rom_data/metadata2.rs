//! EUR byte-pair map; direct indexing and inverse scan have different extents.
use super::{metadata52, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e6c44;
pub const END_OFFSET: u32 = 0x6e6c9c;
pub const RECORD_BYTES: usize = 2;
pub const RECORD_COUNT: usize = 44;
pub const INVERSE_SCAN_COUNT: usize = 43;
pub const REFERENCES: &[(u32, u32)] = &[(0xccd4c, 0xccd54), (0xccd5e, 0xccd6c)];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub key: u8,
    pub value: u8,
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0 => Some(self.key),
            1 => Some(self.value),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; 2] {
        [self.key, self.value]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata2 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Metadata2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR metadata2 extent/count".into());
        }
        Ok(())
    }
    pub fn validate_metadata52(&self, owner: &metadata52::Metadata52) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if self
            .records
            .iter()
            .any(|r| usize::from(r.key) >= owner.records.len())
        {
            return Err("Metadata2 stored key has no parsed metadata52 owner".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    /// None means outside this owned source, not an emulated bounds fallback.
    pub fn forward(&self, index: u32) -> Option<u8> {
        self.records.get(index as u8 as usize).map(|r| r.value)
    }
    pub fn inverse(&self, value: u32) -> u8 {
        self.records
            .iter()
            .take(INVERSE_SCAN_COUNT)
            .find(|r| r.value == value as u8)
            .map_or(0, |r| r.key)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata2 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn literal(rom: &[u8], pc: u32, at: u32, target: u32) -> Result<(), String> {
    let v = half(rom, pc);
    let w = u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap());
    if v & 0xf800 != 0x4800
        || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
        || w != 0x08000000 + target
    {
        return Err("Metadata2 original literal mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Metadata2,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata2 requires exact EUR ROM; USA pending".into());
        }
        for &(pc, at) in REFERENCES {
            literal(rom, pc, at, SOURCE_OFFSET)?;
        }
        literal(rom, 0x127e46, 0x127edc, END_OFFSET)?;
        for (pc, want) in [
            (0xccd48, 0x0600),
            (0xccd4a, 0x0dc0),
            (0xccd50, 0x7840),
            (0xccd5a, 0x0600),
            (0xccd5c, 0x0e03),
            (0xccd62, 0x7848),
            (0xccd68, 0x7808),
            (0xccd72, 0x3102),
            (0xccd74, 0x2a2a),
            (0xccd76, 0xddf4),
            (0xcbd74, 0xb500),
            (0xcbd76, 0x79c0),
            (0x127e4c, 0x2230),
        ] {
            if half(rom, pc) != want {
                return Err("Metadata2 original direct/inverse/live-reader mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Metadata2 {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(2)
                .map(|r| Record {
                    key: r[0],
                    value: r[1],
                })
                .collect(),
        };
        decoded.validate_metadata52(&metadata52::Catalog::from_rom(rom)?.decoded)?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(bytes),
                role: "metadata2/eur-fixed".into(),
            }],
        })
    }
}
