//! EUR physical conditional slots and their original selection predicates.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x713744;
pub const SOURCE_END: u32 = 0x713b3c;
pub const RECORD_BYTES: usize = 8;
pub const RECORD_COUNT: usize = 127;
pub const SOURCE_BYTES: usize = RECORD_BYTES * RECORD_COUNT;
pub const SOURCE_SHA256: &str = "d52e829c2ae2e3f8a519fc1fe28797a1005fcdc0e2afc81201e872024608b18b";
pub type SourceSpan = (u32, Vec<u8>, String);

/// Mutable live state is supplied by the caller, never exported as base ROM data.
pub trait LivePredicates {
    fn flag(&mut self, key: u16) -> bool;
    fn counter(&mut self, index: u16) -> u8;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u8,
    pub field00: u8,
    pub field01: u8,
    pub key02: u16,
    pub operand04: u8,
    pub key05: u16,
    pub operand07: u8,
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let a = self.key02.to_le_bytes();
        let b = self.key05.to_le_bytes();
        [
            self.field00,
            self.field01,
            a[0],
            a[1],
            self.operand04,
            b[0],
            b[1],
            self.operand07,
        ]
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.source_bytes().get(offset).copied()
    }
    pub fn source_address(&self) -> u32 {
        0x08000000 + SOURCE + u32::from(self.index) * RECORD_BYTES as u32
    }
    pub fn eligible(&self, context: u32, live: &mut impl LivePredicates) -> bool {
        if self.field00 == 0 || (self.field01 != 0 && u32::from(self.field01) != context) {
            return false;
        }
        if self.key02 > 0x05ff {
            if live.counter(self.key02 - 0x0600) < self.operand04 {
                return false;
            }
        } else {
            // Preserve the original two live queries and literal 0/1 comparisons.
            if live.flag(self.key02) && self.operand04 == 0 {
                return false;
            }
            if !live.flag(self.key02) && self.operand04 == 1 {
                return false;
            }
        }
        if self.key05 > 0x05ff {
            live.counter(self.key05 - 0x0600) < self.operand07
        } else if self.key05 == 0 {
            true
        } else {
            let set = live.flag(self.key05);
            !((set && self.operand07 == 1) || (!set && self.operand07 == 0))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records8 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Records8 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid declared EUR conditional-records8 source/count".into());
        }
        if self
            .records
            .iter()
            .enumerate()
            .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid conditional-records8 physical slot order/index".into());
        }
        Ok(())
    }
    /// Physical ownership only; an absent slot does not replace a guest read.
    pub fn record(&self, index: u8) -> Option<&Record> {
        self.records.get(usize::from(index))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if !(SOURCE..SOURCE_END).contains(&offset) {
            return None;
        }
        let at = (offset - SOURCE) as usize;
        self.records.get(at / RECORD_BYTES)?.byte(at % RECORD_BYTES)
    }
    /// Selection stage only: no output-buffer clearing or guest-memory writes.
    /// The original tests capacity after writing; zero therefore scans all slots.
    pub fn select(&self, context: u32, capacity: u32, live: &mut impl LivePredicates) -> Vec<u32> {
        let mut remaining = capacity;
        let mut result = Vec::new();
        for record in &self.records {
            if record.eligible(context, live) {
                result.push(record.source_address());
                remaining = remaining.wrapping_sub(1);
                if remaining == 0 {
                    break;
                }
            }
        }
        result
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE,
            self.source_bytes(),
            "conditional-records8-slots".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Conditional-records8 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Records8,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional-records8 requires exact EUR ROM; USA pending".into());
        }
        let raw = &rom[SOURCE as usize..SOURCE_END as usize];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Conditional-records8 source fingerprint mismatch".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err(
                    "Conditional-records8 original reader/boundary fingerprint mismatch".into(),
                );
            }
        }
        let decoded = Records8 {
            source_offset: SOURCE,
            records: raw
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, b)| Record {
                    index: i as u8,
                    field00: b[0],
                    field01: b[1],
                    key02: u16::from_le_bytes([b[2], b[3]]),
                    operand04: b[4],
                    key05: u16::from_le_bytes([b[5], b[6]]),
                    operand07: b[7],
                })
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE,
                length: SOURCE_BYTES as u32,
                sha256: ffta_rom::digest(raw),
                role: "conditional-records8-slots".into(),
            }],
        })
    }
}

// Exact source-contract windows, including both callers and separate successor.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0xd56aa,
        180,
        "af4b1b981d48365083439ae48233adadedd00d73840946d1b3e42b38b2006a59",
    ),
    (
        0xd573e,
        160,
        "2807672ca77b65fd0fc874da2457c17f0b50ac71e8aa2526abe724545a3efff8",
    ),
    (
        0xd5680,
        48,
        "b46860d38e313b5d682df183c85011880fdb5125240aab2c775d4fc8da85e369",
    ),
    (
        0xcce80,
        220,
        "49a138cb8a1ec4da3db8a07ca228b7338ed54fa67d021c2d54fd3c059aa70a70",
    ),
    (
        0xd1f00,
        140,
        "b79596f9fe4e76e3c5647d48abe168ff8b732d13f74db3e5a373e8765ce3ce78",
    ),
    (
        0x31c20,
        180,
        "6a81135e710e8e8592c6685de3a9c75637b2033ac799638c0ec3690fd68b42c0",
    ),
    (
        0x5f4f0,
        200,
        "4e5996c850a0eadf86c8d5e3b56dfa5399cde3cf66d395d06c8a85f28ed6f10f",
    ),
    (
        0x5f9e0,
        220,
        "0de078ef5f988e038648227ca341ed5f051afb99bab3d6115213c7ea71c00dee",
    ),
];

#[cfg(test)]
mod tests;
