//! Bounded progression minima and structural return fields; original code remains.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e7dce;
pub const END_OFFSET: u32 = 0x6e80ce;
pub const RECORD_COUNT: usize = 64;
pub const RECORD_BYTES: usize = 12;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub byte00: u8,
    pub halfword01: u16,
    pub byte03: u8,
    pub requirements04_0b: [u8; 8],
}
impl Record {
    pub fn byte(&self, at: usize) -> Option<u8> {
        match at {
            0 => Some(self.byte00),
            1 => Some(self.halfword01 as u8),
            2 => Some((self.halfword01 >> 8) as u8),
            3 => Some(self.byte03),
            4..=11 => Some(self.requirements04_0b[at - 4]),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    pub fn eligible(&self, levels: [u8; 8]) -> bool {
        levels
            .into_iter()
            .zip(self.requirements04_0b)
            .all(|(live, minimum)| live >= minimum)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub matched_id: Option<u8>,
    pub received: u64,
    pub output: u8,
    pub value: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionGates {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl ProgressionGates {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR progression gate extent/count".into());
        }
        for (i, r) in self.records.iter().enumerate() {
            if r.id as usize != i || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32 {
                return Err("Invalid progression gate ID/order/source".into());
            }
        }
        Ok(())
    }
    /// Finite diagnostic view; None does not introduce a guest fallback.
    pub fn record(&self, id: u32) -> Option<&Record> {
        self.records.get(usize::try_from(id).ok()?)
    }
    /// Diagnostic result, including a zero-valued success. Original code owns RAM writes.
    pub fn evaluate(&self, levels: [u8; 8], received: u64) -> Outcome {
        for (i, r) in self.records.iter().take(RECORD_COUNT).enumerate() {
            let bit = 1u64 << i;
            if received & bit == 0 && r.eligible(levels) {
                return Outcome {
                    matched_id: Some(i as u8),
                    received: received | bit,
                    output: r.byte03,
                    value: r.halfword01,
                };
            }
        }
        Outcome {
            matched_id: None,
            received,
            output: 0,
            value: 0,
        }
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Progression gates differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: ProgressionGates,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Progression gates require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (pc, at, target) in [
            (0x487f6u32, 0x48868u32, SOURCE_OFFSET),
            (0xd23e6, 0xd24a8, END_OFFSET),
        ] {
            let v = half(pc as usize);
            if v & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
                || word(at as usize) != 0x08000000 + target
            {
                return Err("Progression gate original/next literal mismatch".into());
            }
        }
        if word(0x48864) != 0x02002c08
            || word(0x4886c) != 0x020021b6
            || word(0x48870) + word(0x48874) != 0x020021b6
        {
            return Err("Progression gate live mask/levels mismatch".into());
        }
        for (pc, v) in [
            (0x487da, 0x2501),
            (0x487dc, 0x2600),
            (0x487e6, 0x6808),
            (0x487e8, 0x6849),
            (0x487f2, 0x2000),
            (0x48806, 0xd117),
            (0x48810, 0x7939),
            (0x48814, 0xd310),
            (0x4881c, 0x1d3c),
            (0x48822, 0x3202),
            (0x48824, 0x3301),
            (0x48826, 0x2b07),
            (0x48828, 0xdc06),
            (0x4882e, 0x7809),
            (0x48832, 0xd2f4),
            (0x4883c, 0xd11c),
            (0x4884c, 0x210c),
            (0x48856, 0x293f),
            (0x48858, 0xddcd),
            (0x4885e, 0x7001),
            (0x48860, 0x2000),
            (0x4888c, 0x6002),
            (0x4888e, 0x6043),
            (0x48890, 0x78f8),
            (0x48894, 0x7008),
            (0x48896, 0x7879),
            (0x48898, 0x78b8),
            (0x4889c, 0x4308),
            (0xd23e4, 0x0111),
        ] {
            if half(pc) != v {
                return Err(
                    "Progression gate original field/index/mask/minimum/return reader mismatch"
                        .into(),
                );
            }
        }
        let records = (0..RECORD_COUNT)
            .map(|i| {
                let at = SOURCE_OFFSET + (i * RECORD_BYTES) as u32;
                let b = &rom[at as usize..at as usize + RECORD_BYTES];
                Record {
                    id: i as u8,
                    source_offset: at,
                    byte00: b[0],
                    halfword01: u16::from_le_bytes([b[1], b[2]]),
                    byte03: b[3],
                    requirements04_0b: b[4..].try_into().unwrap(),
                }
            })
            .collect();
        let decoded = ProgressionGates {
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
                role: "progression-gates/eur-records".into(),
            }],
        })
    }
}
