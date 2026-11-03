//! Conditional signed task increments; original indexing/arithmetic stay guest code.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x371034;
/// Domain after the original stored-u8 increment and remainder, not a guest cap.
pub const CYCLE_LENGTH: usize = 48;
pub const SOURCE_SHA256: &str = "a586d9892013a2a3c8dce2bd1629b4f6c611fed7b53e6dbb4a6ba542f13db020";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleDelta {
    pub phase: u8,
    /// Added to task halfword+8, or subtracted when original mode bit8 is set.
    pub delta: i8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskCycleDeltas {
    pub source_offset: u32,
    pub entries: Vec<CycleDelta>,
}
impl TaskCycleDeltas {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.entries.len() != CYCLE_LENGTH
            || self
                .entries
                .iter()
                .enumerate()
                .any(|(i, e)| usize::from(e.phase) != i)
        {
            return Err("Invalid conditional EUR task-cycle deltas".into());
        }
        Ok(())
    }
    /// OLD indices48..255 remain outside this projection; lookup has no guest cap.
    pub fn delta(&self, phase: u8) -> Option<i8> {
        self.entries.get(usize::from(phase)).map(|e| e.delta)
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.entries
            .get(offset.checked_sub(self.source_offset)? as usize)
            .map(|e| e.delta as u8)
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        Ok(vec![(
            self.source_offset,
            self.entries.iter().map(|e| e.delta as u8).collect(),
            "task-cycle-deltas".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Task-cycle deltas differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: TaskCycleDeltas,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Task-cycle deltas require exact EUR ROM; USA pending".into());
        }
        // Complete mode gate/reader/literals, then the nonzero-divisor helper.
        // The fixed argument48 excludes its divisor-zero error path. Its r0
        // return is remainder; byte store/reload occurs BEFORE that call.
        for (start, end, sha) in [
            (
                0x18dac,
                0x18e5c,
                "8f051748209c073075bc91cfe498c7d33f898cd1eaab69d0f07354d4841a0bd1",
            ),
            (
                0x1464ec,
                0x1465a2,
                "83f774210a8e58ef8f1448f2fccae7675b8bce1c6b13cd1cac8e1333c081b42e",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != sha {
                return Err("Original task-cycle reader/helper fingerprint mismatch".into());
            }
        }
        for literal in [0x18e18, 0x18e58] {
            if u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap())
                != 0x08000000 + SOURCE
            {
                return Err("Original task-cycle source literal mismatch".into());
            }
        }
        let raw = &rom[SOURCE as usize..SOURCE as usize + CYCLE_LENGTH];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Task-cycle delta source fingerprint mismatch".into());
        }
        let decoded = TaskCycleDeltas {
            source_offset: SOURCE,
            entries: raw
                .iter()
                .enumerate()
                .map(|(i, &byte)| CycleDelta {
                    phase: i as u8,
                    delta: byte as i8,
                })
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE,
                length: CYCLE_LENGTH as u32,
                sha256: SOURCE_SHA256.into(),
                role: "task-cycle-deltas".into(),
            }],
        })
    }
}
