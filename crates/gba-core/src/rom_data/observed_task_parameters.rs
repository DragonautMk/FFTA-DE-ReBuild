//! Authenticated task-parameter fragments; no global guest input cap.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const PHASE_SOURCE: u32 = 0x371064;
/// Conditional phases in the original wrap sequence, not a prelookup guest cap.
pub const PHASE_COUNT: usize = 3;
pub const VERTICAL_SOURCE: u32 = 0x394e14;
pub const PHASE_SHA256: &str = "ee8357f0303dac80be96e10b1b49913c294c163771824fca6152057a6e1da957";
pub const VERTICAL_SHA256: &str =
    "13f75c7ff789108645b9fc6e662298dd306b4175d247e4b7da985b5bea8f0f9d";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseThreshold {
    pub phase: u8,
    /// Compared with the OLD u16 task counter, after incrementing its RAM word.
    pub threshold: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseThresholds {
    pub source_offset: u32,
    pub entries: Vec<PhaseThreshold>,
}
impl PhaseThresholds {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != PHASE_SOURCE
            || self.entries.len() != PHASE_COUNT
            || self
                .entries
                .iter()
                .enumerate()
                .any(|(i, e)| usize::from(e.phase) != i)
        {
            return Err("Invalid observed EUR phase-threshold fragment".into());
        }
        Ok(())
    }
    /// Conditional phases0..2 only. Other guest bytes retain prior/raw behavior.
    pub fn threshold(&self, phase: u8) -> Option<u8> {
        self.entries.get(usize::from(phase)).map(|e| e.threshold)
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.entries
            .get(offset.checked_sub(self.source_offset)? as usize)
            .map(|e| e.threshold)
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            self.source_offset,
            self.entries.iter().map(|e| e.threshold).collect(),
            "phase-threshold".into(),
        )])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerticalCorrection {
    /// Original index is the live task byte shifted right once, before lookup.
    pub index: u8,
    pub correction: i16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerticalCorrections {
    pub source_offset: u32,
    pub entries: Vec<VerticalCorrection>,
}
impl VerticalCorrections {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != VERTICAL_SOURCE
            || self.entries.len() != 4
            || self
                .entries
                .iter()
                .enumerate()
                .any(|(i, e)| usize::from(e.index) != i + 4)
        {
            return Err("Invalid observed EUR vertical-correction fragment".into());
        }
        Ok(())
    }
    /// Other indices retain earlier owners/raw memory; this is not a cap of8.
    pub fn correction(&self, task_byte: u8) -> Option<i16> {
        self.entries
            .get(usize::from((task_byte >> 1).checked_sub(4)?))
            .map(|e| e.correction)
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let byte = offset.checked_sub(self.source_offset)? as usize;
        Some(self.entries.get(byte / 2)?.correction.to_le_bytes()[byte % 2])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            self.source_offset,
            self.entries
                .iter()
                .flat_map(|e| e.correction.to_le_bytes())
                .collect(),
            "vertical-correction".into(),
        )])
    }
}

/// Independent parameter kinds retain separate source identities and fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTaskParameters {
    pub phase_thresholds: PhaseThresholds,
    pub vertical_corrections: VerticalCorrections,
}
impl ObservedTaskParameters {
    pub fn validate(&self) -> Result<(), String> {
        self.phase_thresholds.validate()?;
        self.vertical_corrections.validate()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.phase_thresholds
            .byte_at(offset)
            .or_else(|| self.vertical_corrections.byte_at(offset))
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = self.phase_thresholds.source_spans()?;
        spans.extend(self.vertical_corrections.source_spans()?);
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Task parameters differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ObservedTaskParameters,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed task parameters require exact EUR ROM; USA pending".into());
        }
        // Reuse the authenticated source-contract windows. The first contains
        // counter increment/old-value compare/phase wrap and its literal; the
        // second contains the live-byte shift, signed load/subtract and literal.
        for (start, end, sha) in [
            (
                0x18dca,
                0x18df8,
                "de58c75ebaf21379c219fe99e73b30db94b6abafd09b3c31c535e1050f6fffd4",
            ),
            (
                0x2644c,
                0x26538,
                "84f6899435d7407e888845b8cb41eca0c66b255c8cb7188a3cd28f48c1f43a1b",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != sha {
                return Err("Original task-parameter reader fingerprint mismatch".into());
            }
        }
        for (literal, expected) in [(0x18df4, 0x08371064u32), (0x26534, 0x08394e0cu32)] {
            if u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) != expected {
                return Err("Original task-parameter source literal mismatch".into());
            }
        }
        let phases = &rom[PHASE_SOURCE as usize..PHASE_SOURCE as usize + PHASE_COUNT];
        let vertical = &rom[VERTICAL_SOURCE as usize..VERTICAL_SOURCE as usize + 8];
        if ffta_rom::digest(phases) != PHASE_SHA256 || ffta_rom::digest(vertical) != VERTICAL_SHA256
        {
            return Err("Observed task-parameter source fingerprint mismatch".into());
        }
        let decoded = ObservedTaskParameters {
            phase_thresholds: PhaseThresholds {
                source_offset: PHASE_SOURCE,
                entries: phases
                    .iter()
                    .enumerate()
                    .map(|(i, &threshold)| PhaseThreshold {
                        phase: i as u8,
                        threshold,
                    })
                    .collect(),
            },
            vertical_corrections: VerticalCorrections {
                source_offset: VERTICAL_SOURCE,
                entries: vertical
                    .chunks_exact(2)
                    .enumerate()
                    .map(|(i, raw)| VerticalCorrection {
                        index: i as u8 + 4,
                        correction: i16::from_le_bytes(raw.try_into().unwrap()),
                    })
                    .collect(),
            },
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
