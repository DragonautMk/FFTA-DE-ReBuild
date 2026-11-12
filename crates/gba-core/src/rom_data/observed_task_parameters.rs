//! Authenticated task-parameter fragments; no global guest input cap.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const PHASE_SOURCE: u32 = 0x371064;
pub const BATTLE_PHASE_SOURCE: u32 = 0x37101c;
pub const BATTLE_DRAW_SEQUENCE_SOURCE: u32 = 0x370ffc;
/// Conditional phases in the original wrap sequence, not a prelookup guest cap.
pub const PHASE_COUNT: usize = 3;
pub const VERTICAL_SOURCE: u32 = 0x394e14;
pub const PHASE_SHA256: &str = "ee8357f0303dac80be96e10b1b49913c294c163771824fca6152057a6e1da957";
pub const VERTICAL_SHA256: &str =
    "0e03eb227902a1b34389e6c7333ce03887b46f7386b2c7b120fbdd496295f43a";
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
            || !matches!(self.entries.len(), 4 | 6)
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
    /// Older datasets contain indices 4..7; newer ones include observed 8..9.
    /// Other indices retain earlier owners/raw memory; no global cap is imposed.
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
    /// Separate battle-task bytes; absent in datasets before E148.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle_phase_thresholds: Option<[u8; 3]>,
    /// Values for the conditional post-mask draw sequence; first index is uncapped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle_draw_sequence: Option<[u8; 32]>,
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
            .or_else(|| self.battle_phase_thresholds.as_ref()?.get(offset.checked_sub(BATTLE_PHASE_SOURCE)? as usize).copied())
            .or_else(|| self.battle_draw_sequence.as_ref()?.get(offset.checked_sub(BATTLE_DRAW_SEQUENCE_SOURCE)? as usize).copied())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = self.phase_thresholds.source_spans()?;
        spans.extend(self.vertical_corrections.source_spans()?);
        if let Some(thresholds) = &self.battle_phase_thresholds {
            spans.push((BATTLE_PHASE_SOURCE, thresholds.to_vec(), "battle-phase-threshold".into()));
        }
        if let Some(sequence) = &self.battle_draw_sequence {
            spans.push((BATTLE_DRAW_SEQUENCE_SOURCE, sequence.to_vec(), "battle-draw-sequence".into()));
        }
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
            (0x131b8, 0x1320c, "63818f5802278ef13d944cdc09da0539f0332a7cc5ac63577327a8941b2caeb6"),
            (0x13130, 0x131b8, "2da1654a2753c0c840798f3a1be2cb23b4757423dbd6becfa85933095f620451"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != sha {
                return Err("Original task-parameter reader fingerprint mismatch".into());
            }
        }
        for (literal, expected) in [(0x18df4, 0x08371064u32), (0x26534, 0x08394e0cu32), (0x13258, 0x0837101cu32), (0x131b0, 0x08370ffcu32)] {
            if u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) != expected {
                return Err("Original task-parameter source literal mismatch".into());
            }
        }
        let phases = &rom[PHASE_SOURCE as usize..PHASE_SOURCE as usize + PHASE_COUNT];
        let vertical = &rom[VERTICAL_SOURCE as usize..VERTICAL_SOURCE as usize + 12];
        let battle = &rom[BATTLE_PHASE_SOURCE as usize..BATTLE_PHASE_SOURCE as usize + 3];
        let draw = &rom[BATTLE_DRAW_SEQUENCE_SOURCE as usize..BATTLE_DRAW_SEQUENCE_SOURCE as usize + 32];
        if ffta_rom::digest(phases) != PHASE_SHA256 || ffta_rom::digest(vertical) != VERTICAL_SHA256 || ffta_rom::digest(battle) != PHASE_SHA256
            || ffta_rom::digest(draw) != "8352f94c00b26d3b0834df0b3f1f16de59302e3dc4edf92de8625902dbdc12cc"
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
            battle_phase_thresholds: Some(battle.try_into().unwrap()),
            battle_draw_sequence: Some(draw.try_into().unwrap()),
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
