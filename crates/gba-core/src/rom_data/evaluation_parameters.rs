//! Bounded score coefficients and class thresholds; original live evaluation remains.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SCORE_OFFSET: u32 = 0x6e79a2;
pub const SCORE_END: u32 = 0x6e79b4;
pub const THRESHOLD_BIAS: u32 = 0x6e79c2;
pub const THRESHOLD_SOURCE: u32 = 0x6e79c6;
pub const END_OFFSET: u32 = 0x6e79da;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term {
    pub id: u8,
    pub source_offset: u32,
    pub divisor: u8,
    pub weight: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bonus {
    pub id: u8,
    pub source_offset: u32,
    pub byte00: u8,
    pub weight: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Threshold {
    pub id: u8,
    pub source_offset: u32,
    pub threshold: u8,
    pub bytes01_03: [u8; 3],
}
impl Threshold {
    /// Original signed low16 normalization. Actual LCG returns only15bits.
    pub fn accepts_random(&self, value: u32) -> bool {
        ((value as u16 as i16) % 256) < i16::from(self.threshold)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationParameters {
    pub score_offset: u32,
    pub terms: [Term; 7],
    pub bonuses: [Bonus; 2],
    pub threshold_address_bias: u32,
    pub thresholds: [Threshold; 5],
}
impl EvaluationParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.score_offset != SCORE_OFFSET || self.threshold_address_bias != THRESHOLD_BIAS {
            return Err("Invalid fixed EUR evaluation roots".into());
        }
        for (i, r) in self.terms.iter().enumerate() {
            if r.id as usize != i || r.source_offset != SCORE_OFFSET + (i * 2) as u32 {
                return Err("Invalid evaluation term ID/order/source".into());
            }
        }
        for (i, r) in self.bonuses.iter().enumerate() {
            if r.id as usize != i + 7 || r.source_offset != SCORE_OFFSET + ((i + 7) * 2) as u32 {
                return Err("Invalid evaluation bonus ID/order/source".into());
            }
        }
        for (i, r) in self.thresholds.iter().enumerate() {
            if r.id as usize != i + 1 || r.source_offset != THRESHOLD_BIAS + ((i + 1) * 4) as u32 {
                return Err("Invalid evaluation threshold ID/order/source".into());
            }
        }
        Ok(())
    }
    pub fn score_bytes(&self) -> [u8; 18] {
        std::array::from_fn(|i| {
            let n = i / 2;
            if n < 7 {
                if i % 2 == 0 {
                    self.terms[n].divisor
                } else {
                    self.terms[n].weight
                }
            } else if i % 2 == 0 {
                self.bonuses[n - 7].byte00
            } else {
                self.bonuses[n - 7].weight
            }
        })
    }
    pub fn threshold_bytes(&self) -> [u8; 20] {
        std::array::from_fn(|i| {
            let r = &self.thresholds[i / 4];
            if i % 4 == 0 {
                r.threshold
            } else {
                r.bytes01_03[i % 4 - 1]
            }
        })
    }
    /// Diagnostic domain only. Zero-divisor guest code remains untouched.
    pub fn term_product(&self, index: usize, value: u32) -> Option<u32> {
        let t = self.terms.get(index)?;
        if t.divisor == 0 {
            return None;
        }
        let quotient = if index == 0 {
            ((value / u32::from(t.divisor)) as u8) as u32
        } else {
            ((value as i32) / i32::from(t.divisor)) as u32
        };
        Some(quotient.wrapping_mul(u32::from(t.weight)))
    }
    pub fn score(&self, inputs: [u32; 7], predicates: [u32; 2]) -> Option<u32> {
        let mut score = 0u32;
        for (i, value) in inputs.into_iter().enumerate() {
            score = score.wrapping_add(self.term_product(i, value)?);
        }
        for (i, predicate) in predicates.into_iter().enumerate() {
            if predicate as u8 == if i == 0 { 1 } else { 0 } {
                score = score.wrapping_add(u32::from(self.bonuses[i].weight));
            }
        }
        Some(score)
    }
    /// None denotes an unowned class, never an emulated bounds return.
    pub fn threshold(&self, class: u32) -> Option<&Threshold> {
        self.thresholds
            .get(usize::try_from(class.checked_sub(1)?).ok()?)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Evaluation parameters differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: EvaluationParameters,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Evaluation parameters require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        for (pc, at, target) in [
            (0xd2db0u32, 0xd2f20u32, SCORE_OFFSET),
            (0xd4846, 0xd4888, THRESHOLD_BIAS),
            (0x4b45c, 0x4b49c, END_OFFSET),
        ] {
            let v = half(pc as usize);
            if v & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
                || u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                    != 0x08000000 + target
            {
                return Err("Evaluation original/next literal mismatch".into());
            }
        }
        for (pc, v) in [
            (0xd2db4, 0x7821),
            (0xd2dbe, 0x7861),
            (0xd2dba, 0x0600),
            (0xd2dbc, 0x0e00),
            (0xd2dc8, 0x7821),
            (0xd2dce, 0x7861),
            (0xd2dd8, 0x7821),
            (0xd2df0, 0x7821),
            (0xd2e08, 0x7821),
            (0xd2e20, 0x7821),
            (0xd2e38, 0x7821),
            (0xd2e50, 0x2801),
            (0xd2e62, 0x2800),
            (0xd2e54, 0x7860),
            (0xd2e66, 0x7860),
            (0xd483a, 0xf7fe),
            (0xd483c, 0xfa83),
            (0xd4840, 0x2c00),
            (0xd4842, 0xd013),
            (0xd4844, 0x00a1),
            (0xd484a, 0xf72e),
            (0xd484c, 0xf80d),
            (0xd484e, 0x0400),
            (0xd4850, 0x1401),
            (0xd4864, 0x782d),
            (0xd4866, 0x42a8),
            (0xd4868, 0xdb00),
            (0xd486a, 0x2400),
            (0xd2f30, 0x2809),
            (0xd2f36, 0x281d),
            (0xd2f3c, 0x2831),
            (0x4b456, 0x00a0),
            (0x4b458, 0x1900),
            (0x4b45a, 0x0040),
        ] {
            if half(pc) != v {
                return Err(
                    "Evaluation original score/class/threshold/next reader mismatch".into(),
                );
            }
        }
        let terms = std::array::from_fn(|i| {
            let at = SCORE_OFFSET + (i * 2) as u32;
            Term {
                id: i as u8,
                source_offset: at,
                divisor: rom[at as usize],
                weight: rom[at as usize + 1],
            }
        });
        let bonuses = std::array::from_fn(|i| {
            let at = SCORE_OFFSET + ((i + 7) * 2) as u32;
            Bonus {
                id: (i + 7) as u8,
                source_offset: at,
                byte00: rom[at as usize],
                weight: rom[at as usize + 1],
            }
        });
        let thresholds = std::array::from_fn(|i| {
            let at = THRESHOLD_BIAS + ((i + 1) * 4) as u32;
            Threshold {
                id: (i + 1) as u8,
                source_offset: at,
                threshold: rom[at as usize],
                bytes01_03: rom[at as usize + 1..at as usize + 4].try_into().unwrap(),
            }
        });
        let decoded = EvaluationParameters {
            score_offset: SCORE_OFFSET,
            terms,
            bonuses,
            threshold_address_bias: THRESHOLD_BIAS,
            thresholds,
        };
        decoded.validate()?;
        let sources = [
            (SCORE_OFFSET, SCORE_END, "evaluation-parameters/score"),
            (
                THRESHOLD_SOURCE,
                END_OFFSET,
                "evaluation-parameters/thresholds",
            ),
        ]
        .into_iter()
        .map(|(start, end, role)| SourceRange {
            offset: start,
            length: end - start,
            sha256: ffta_rom::digest(&rom[start as usize..end as usize]),
            role: role.into(),
        })
        .collect();
        Ok(Self { decoded, sources })
    }
}
