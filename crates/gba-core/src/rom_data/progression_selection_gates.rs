//! Structural progression selection conditions; original code owns live state.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const INDEX_BASE: u32 = 0x6e80ce;
pub const SOURCE_OFFSET: u32 = INDEX_BASE + 16;
pub const END_OFFSET: u32 = INDEX_BASE + 65 * 16;
pub const RECORD_COUNT: usize = 64;
pub const RECORD_BYTES: usize = 16;
pub const RESET_FLAGS: [u16; 4] = [0x5a1, 0x5a2, 0x5a3, 0x5a9];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub selector00: u8,
    pub minimum01: u8,
    pub minimum02: u16,
    pub minimum04: u16,
    pub any_minima06_0d: [u8; 8],
    pub flag0e: u16,
}
impl Record {
    pub fn byte(&self, at: usize) -> Option<u8> {
        match at {
            0 => Some(self.selector00),
            1 => Some(self.minimum01),
            2 => Some(self.minimum02 as u8),
            3 => Some((self.minimum02 >> 8) as u8),
            4 => Some(self.minimum04 as u8),
            5 => Some((self.minimum04 >> 8) as u8),
            6..=13 => Some(self.any_minima06_0d[at - 6]),
            14 => Some(self.flag0e as u8),
            15 => Some((self.flag0e >> 8) as u8),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Live {
    pub byte01: u8,
    pub halfword02: u16,
    pub aggregate04: u16,
    pub levels: [u8; 8],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub matched_index: Option<u8>,
    /// None means no store. A no-condition terminal row explicitly stores zero.
    pub output: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionSelectionGates {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl ProgressionSelectionGates {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR progression selection gate extent/count".into());
        }
        for (i, r) in self.records.iter().enumerate() {
            if r.id as usize != i + 1
                || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32
            {
                return Err("Invalid progression selection gate ID/order/source".into());
            }
        }
        Ok(())
    }
    pub fn record(&self, index: u32) -> Option<&Record> {
        self.records
            .get(usize::try_from(index.checked_sub(1)?).ok()?)
    }
    /// Diagnostic view after original reset/count helpers. No guest fallback or stores.
    pub fn evaluate(&self, live: Live, mut flag: impl FnMut(u16) -> bool) -> Outcome {
        for r in self.records.iter().take(RECORD_COUNT) {
            if live.byte01 < r.minimum01
                || live.halfword02 < r.minimum02
                || live.aggregate04 < r.minimum04
            {
                continue;
            }
            let any_present = r.any_minima06_0d.iter().any(|&v| v != 0);
            if any_present
                && !live
                    .levels
                    .iter()
                    .zip(r.any_minima06_0d)
                    .any(|(&value, minimum)| minimum != 0 && value >= minimum)
            {
                continue;
            }
            if r.flag0e != 0 && !flag(r.flag0e) {
                continue;
            }
            let has_condition = r.minimum01 != 0
                || r.minimum02 != 0
                || r.minimum04 != 0
                || any_present
                || r.flag0e != 0;
            return Outcome {
                matched_index: Some(r.id),
                output: Some(if has_condition { r.selector00 } else { 0 }),
            };
        }
        Outcome {
            matched_index: None,
            output: None,
        }
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Progression selection gates differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: ProgressionSelectionGates,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Progression selection gates require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (pc, at, value) in [
            (0xd23e6u32, 0xd24a8u32, 0x08000000 + INDEX_BASE),
            (0xd239e, 0xd2494, 0x020021a0),
            (0xd2406, 0xd24ac, 0x02001f6c),
            (0x16192, 0x161b0, 0x086e84e0),
        ] {
            let v = half(pc as usize);
            if v & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
                || word(at as usize) != value
            {
                return Err("Progression selection original literal mismatch".into());
            }
        }
        for (pc, value) in [
            (0xd23d0, 0x2201),
            (0xd23e4, 0x0111),
            (0xd23ea, 0x7861),
            (0xd23f6, 0xd35f),
            (0xd240c, 0xd354),
            (0xd2424, 0xd348),
            (0xd242e, 0x1da6),
            (0xd244c, 0xd307),
            (0xd2460, 0x2907),
            (0xd2466, 0xd002),
            (0xd246c, 0xd024),
            (0xd246e, 0x7ba1),
            (0xd2470, 0x7be0),
            (0xd2482, 0xd019),
            (0xd248a, 0x2d00),
            (0xd248c, 0xd110),
            (0xd2490, 0x700d),
            (0xd24b0, 0x7820),
            (0xd24b4, 0x7010),
            (0xd24be, 0x2940),
            (0xd24c0, 0xdd8b),
        ] {
            if half(pc) != value {
                return Err("Progression selection original condition/terminal mismatch".into());
            }
        }
        let records = (0..RECORD_COUNT)
            .map(|i| {
                let at = SOURCE_OFFSET + (i * RECORD_BYTES) as u32;
                let b = &rom[at as usize..at as usize + RECORD_BYTES];
                Record {
                    id: (i + 1) as u8,
                    source_offset: at,
                    selector00: b[0],
                    minimum01: b[1],
                    minimum02: u16::from_le_bytes([b[2], b[3]]),
                    minimum04: u16::from_le_bytes([b[4], b[5]]),
                    any_minima06_0d: b[6..14].try_into().unwrap(),
                    flag0e: u16::from_le_bytes([b[14], b[15]]),
                }
            })
            .collect();
        let decoded = ProgressionSelectionGates {
            source_offset: SOURCE_OFFSET,
            records,
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: END_OFFSET - SOURCE_OFFSET,
                sha256: ffta_rom::digest(&rom[SOURCE_OFFSET as usize..END_OFFSET as usize]),
                role: "progression-selection-gates/eur-records".into(),
            }],
        })
    }
}
