//! Two selector/value pairs per EUR row; original live requirement scans stay guest code.
use super::{metadata52, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e6bf4;
pub const END_OFFSET: u32 = 0x6e6c44;
pub const RECORD_BYTES: usize = 4;
pub const RECORD_COUNT: usize = 20;
pub const REFERENCES: &[(u32, u32)] = &[(0x64300, 0x6434c), (0xcc426, 0xcc470)];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub selector: u8,
    pub value: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub slots: [Pair; 2],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        let p = self.slots.get(offset / 2)?;
        Some(if offset & 1 == 0 { p.selector } else { p.value })
    }
    pub fn source_bytes(&self) -> [u8; 4] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Count and contiguous walking are different original contracts.
    pub fn active_count(&self) -> u8 {
        self.slots.iter().filter(|p| p.selector != 0).count() as u8
    }
    pub fn leading_count(&self) -> usize {
        self.slots.iter().take_while(|p| p.selector != 0).count()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata4 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Metadata4 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR metadata4 extent/count".into());
        }
        Ok(())
    }
    pub fn validate_metadata52(&self, owner: &metadata52::Metadata52) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if owner
            .records
            .iter()
            .any(|r| usize::from(r.bytes_20_33[16]) >= self.records.len())
            || self
                .records
                .iter()
                .flat_map(|r| &r.slots)
                .any(|p| usize::from(p.selector) >= owner.records.len())
        {
            return Err("Metadata4 row/selector has no parsed metadata52 owner".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata4 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn literal(rom: &[u8], pc: u32, at: u32, target: u32) -> Result<(), String> {
    let v = half(rom, pc);
    let word = u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap());
    if v & 0xf800 != 0x4800
        || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
        || word != 0x08000000 + target
    {
        return Err("Metadata4 original literal mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Metadata4,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata4 requires exact EUR ROM; USA pending".into());
        }
        for &(pc, at) in REFERENCES {
            literal(rom, pc, at, SOURCE_OFFSET)?;
        }
        for (pc, at) in [(0xccd4c, 0xccd54), (0xccd5e, 0xccd6c)] {
            literal(rom, pc, at, END_OFFSET)?;
        }
        for (pc, want) in [
            (0xcc424, 0x0080),
            (0xcc434, 0x2301),
            (0xcc436, 0x7808),
            (0xcc446, 0x3102),
            (0xcc44c, 0xdaf3),
            (0x642fe, 0x0080),
            (0x64312, 0x7808),
            (0x64322, 0x3102),
            (0xcc476, 0x7815),
            (0xcc478, 0x2d00),
            (0xcc4ee, 0x7840),
            (0xcc4f0, 0x4282),
            (0xcc4f2, 0xdb0d),
            (0xcc4fe, 0x2102),
            (0x643c4, 0x7849),
            (0x643c6, 0x428b),
            (0x643c8, 0xdb0f),
            (0x643d4, 0x2002),
        ] {
            if half(rom, pc) != want {
                return Err("Metadata4 original stride/count/gate/threshold mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Metadata4 {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(4)
                .map(|r| Record {
                    slots: std::array::from_fn(|i| Pair {
                        selector: r[2 * i],
                        value: r[2 * i + 1],
                    }),
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
                role: "metadata4/eur-fixed".into(),
            }],
        })
    }
}
