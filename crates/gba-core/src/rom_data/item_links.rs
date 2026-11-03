//! Counted item-linked byte pairs; original unit/job/ability logic stays guest code.
use super::{items, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e3ad0;
pub const END_OFFSET: u32 = 0x6e4c64;
pub const RECORD_BYTES: usize = 20;
pub const RECORD_COUNT: usize = 225;
pub const REFERENCES: &[(u32, u32)] = &[
    (0x4aef0, 0x4b018),
    (0x694e8, 0x69618),
    (0x72140, 0x72154),
    (0x8f5a8, 0x8f674),
    (0xcc698, 0xcc6ac),
    (0xce8cc, 0xce94c),
    (0xce9fc, 0xcea68),
    (0xceb5a, 0xcebc4),
    (0xd09b0, 0xd09ec),
    (0xd0a6a, 0xd0ae4),
    (0xd0ba0, 0xd0c0c),
    (0xd0cbe, 0xd0d14),
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub selector: u8,
    pub value: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub count: u8,
    pub opaque: u8,
    /// All storage slots remain exact, including those beyond count.
    pub slots: [Pair; 9],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0 => Some(self.count),
            1 => Some(self.opaque),
            2..=19 => {
                let p = &self.slots[(offset - 2) / 2];
                Some(if offset & 1 == 0 { p.selector } else { p.value })
            }
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Raw first-match query used for comparison with the original bounded loop.
    pub fn find(&self, selector: u8, value: u8) -> Option<usize> {
        self.slots
            .get(..usize::from(self.count))?
            .iter()
            .position(|p| p.selector != 0 && p.selector == selector && p.value == value)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Links {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Links {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET
            || self.records.len() != RECORD_COUNT
            || self.records.iter().any(|r| r.count > 9)
        {
            return Err("Invalid fixed EUR item-link extent/count".into());
        }
        Ok(())
    }
    pub fn validate_items(&self, items: &items::Items) -> Result<(), String> {
        self.validate()?;
        items.validate()?;
        if items
            .records
            .iter()
            .any(|r| usize::from(r.bytes_18_1f[5]) >= self.records.len())
        {
            return Err("Item record has no item-link owner".into());
        }
        Ok(())
    }
    pub fn for_item(&self, items: &items::Items, id: u32) -> Option<&Record> {
        self.records.get(usize::from(
            items.records.get(id as u16 as usize)?.bytes_18_1f[5],
        ))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Item links differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
}
fn literal(rom: &[u8], pc: u32, at: u32, target: u32) -> Result<(), String> {
    let h = half(rom, pc);
    if h & 0xf800 != 0x4800
        || ((pc + 4) & !3) + u32::from(h & 255) * 4 != at
        || word(rom, at) != 0x08000000 + target
    {
        return Err("Item-link original literal mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Links,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Item links require exact EUR ROM; USA pending".into());
        }
        for &(pc, at) in REFERENCES {
            literal(rom, pc, at, SOURCE_OFFSET)?;
        }
        literal(rom, 0xd5e80, 0xd5e9c, END_OFFSET)?;
        for (pc, value) in [
            (0xd0cb8, 0x0081),
            (0xd0cba, 0x1809),
            (0xd0cbc, 0x0089),
            (0xd0cc4, 0x7808),
            (0xd0cca, 0x3102),
            (0xd0cce, 0x7808),
            (0xd0cdc, 0x7848),
            (0xd09f0, 0x29ff),
            (0xd09fa, 0x7008),
            (0xd09dc, 0x4311),
            (0xd09de, 0x7001),
        ] {
            if half(rom, pc) != value {
                return Err("Item-link original count/pair/publication mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Links {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(RECORD_BYTES)
                .map(|r| Record {
                    count: r[0],
                    opaque: r[1],
                    slots: std::array::from_fn(|i| Pair {
                        selector: r[2 + 2 * i],
                        value: r[3 + 2 * i],
                    }),
                })
                .collect(),
        };
        decoded.validate_items(&items::Catalog::from_rom(rom)?.decoded)?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(bytes),
                role: "item-links/eur-fixed".into(),
            }],
        })
    }
}
