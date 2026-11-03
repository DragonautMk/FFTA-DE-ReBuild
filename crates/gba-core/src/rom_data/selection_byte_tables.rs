//! EUR unsigned-byte selection data, separate from mutable eligibility and RNG.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x71c73c;
pub const WEIGHT_SOURCE: u32 = 0x71c740;
pub const SOURCE_END: u32 = 0x71c75e;
pub const THRESHOLD_COUNT: usize = 4;
pub const WEIGHT_COUNT: usize = 30;
pub const SOURCE_BYTES: usize = THRESHOLD_COUNT + WEIGHT_COUNT;
pub const SOURCE_SHA256: &str = "d758af5f1a07613f1c2047257d42ef9cbfac83b63783ee6fd0e7b3879370b8dd";
pub type SourceSpan = (u32, Vec<u8>, String);

/// The original helper supplies a raw live byte for each local ordinal.
pub trait LiveEligibility {
    fn read(&mut self, ordinal: u8) -> u8;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByteTables {
    pub threshold_source_offset: u32,
    pub weight_source_offset: u32,
    pub thresholds: [u8; THRESHOLD_COUNT],
    pub weights: [u8; WEIGHT_COUNT],
}
impl ByteTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.threshold_source_offset != SOURCE || self.weight_source_offset != WEIGHT_SOURCE {
            return Err("Invalid declared EUR selection-byte-table source offsets".into());
        }
        Ok(())
    }

    /// Physical view lookup; absence does not introduce a guest-wide count cap.
    pub fn threshold(&self, population: u32) -> Option<u8> {
        self.thresholds.get(population as usize).copied()
    }

    pub fn population_threshold(&self, headers: &[u8; 5]) -> Option<u8> {
        self.threshold(headers.iter().filter(|&&v| v & 7 != 0).count() as u32)
    }

    pub fn weight(&self, physical_position: usize) -> Option<u8> {
        self.weights.get(physical_position).copied()
    }

    /// First scan only. The guest exits before its numeric stage when this is0.
    pub fn sum_weights(&self, live: &mut impl LiveEligibility) -> u32 {
        self.weights
            .iter()
            .enumerate()
            .fold(0, |sum, (i, &weight)| {
                if live.read(i as u8 + 1) != 0 {
                    sum + u32::from(weight)
                } else {
                    sum
                }
            })
    }

    /// Second-scan projection with a supplied full32-bit rank, not a RNG model.
    /// The original subtraction wraps, and only its signed result selects.
    pub fn select_ordinal(&self, mut rank: i32, live: &mut impl LiveEligibility) -> Option<u8> {
        for (i, &weight) in self.weights.iter().enumerate() {
            let ordinal = i as u8 + 1;
            if live.read(ordinal) != 0 {
                rank = rank.wrapping_sub(i32::from(weight));
                if rank < 0 {
                    return Some(ordinal);
                }
            }
        }
        None
    }

    pub fn source_bytes(&self) -> Vec<u8> {
        self.thresholds
            .iter()
            .chain(&self.weights)
            .copied()
            .collect()
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if (SOURCE..WEIGHT_SOURCE).contains(&offset) {
            self.thresholds.get((offset - SOURCE) as usize).copied()
        } else if (WEIGHT_SOURCE..SOURCE_END).contains(&offset) {
            self.weights.get((offset - WEIGHT_SOURCE) as usize).copied()
        } else {
            None
        }
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![
            (SOURCE, self.thresholds.to_vec(), "count-thresholds".into()),
            (
                WEIGHT_SOURCE,
                self.weights.to_vec(),
                "ordinal-weights".into(),
            ),
        ])
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Selection-byte tables differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ByteTables,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Selection-byte tables require exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[SOURCE as usize..SOURCE_END as usize]) != SOURCE_SHA256 {
            return Err("Selection-byte-table source fingerprint mismatch".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err("Selection-byte-table reader/boundary fingerprint mismatch".into());
            }
        }
        let decoded = ByteTables {
            threshold_source_offset: SOURCE,
            weight_source_offset: WEIGHT_SOURCE,
            thresholds: rom[SOURCE as usize..WEIGHT_SOURCE as usize]
                .try_into()
                .unwrap(),
            weights: rom[WEIGHT_SOURCE as usize..SOURCE_END as usize]
                .try_into()
                .unwrap(),
        };
        let sources = decoded
            .source_spans()?
            .into_iter()
            .map(|(offset, bytes, role)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}

const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x63920,
        136,
        "50d0c95bdd79f2bad90ed7190427516da40d93408b7ecb67ca49bf32307b09ea",
    ),
    (
        0x639a4,
        20,
        "90a908e336ef9ce0f399bbd6f15b851203434b7732260f91fcac5cabf9a09de2",
    ),
    (
        0xd3280,
        200,
        "3a11e3c23ad52ade3a8b66fb9c7e6fd3948b43c4ddc417a48243a274903afa09",
    ),
    (
        0xd3230,
        80,
        "a6a8d5fe0038ed5dede6a552a42a3313fcb00467b724c05d2f19a9ccf06714b9",
    ),
    (
        0x377bc,
        96,
        "4a932d3a233a6fdfa1eafb43a44dc932b81f722c37f536a247836b5a8b5d067b",
    ),
    (
        0xd3200,
        48,
        "6fd0f1f48d8d73bbe2a9174e59ebbdf605a172a2e756aaca986fe19f6b5c0df9",
    ),
];

#[cfg(test)]
mod tests;
