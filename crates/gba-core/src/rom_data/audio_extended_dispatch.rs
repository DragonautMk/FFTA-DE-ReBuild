//! Conditional extra M4A dispatch words. The original guest dispatcher remains live.
//!
//! This projection owns ten physical words only. It neither bounds the guest's
//! raw u8 selector nor describes the selected callbacks' operand formats.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_OFFSET: u32 = 0x370d38;
pub const SELECTORS: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 10, 11];
pub const SOURCE_BYTES: usize = 40;
const BASE: u32 = 0x0800_0000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub selector: u8,
    /// The original encoded guest callback word, including its Thumb bit.
    pub callback: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioExtendedDispatch {
    pub source_offset: u32,
    pub entries: Vec<Entry>,
}

impl AudioExtendedDispatch {
    /// A physical fragment lookup, not a validity check on the guest selector.
    pub fn callback(&self, selector: u8) -> Option<u32> {
        self.entries
            .iter()
            .find(|e| e.selector == selector)
            .map(|e| e.callback)
    }

    /// Returns only bytes in this projection, leaving slots 8/9 and other data alone.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(self.source_offset)?;
        let selector = u8::try_from(relative / 4).ok()?;
        Some(self.callback(selector)?.to_le_bytes()[(relative % 4) as usize])
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != TABLE_OFFSET
            || self.entries.len() != SELECTORS.len()
            || self.entries.iter().zip(SELECTORS).any(|(entry, selector)| {
                entry.selector != selector
                    || entry.callback & 1 == 0
                    || !(BASE..BASE + 0x0100_0000).contains(&entry.callback)
            })
        {
            return Err("Invalid M4A extended dispatch fragment".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        Ok(vec![
            (
                TABLE_OFFSET,
                self.entries[..8]
                    .iter()
                    .flat_map(|e| e.callback.to_le_bytes())
                    .collect(),
                "audio/extended-dispatch/slots-0-7".into(),
            ),
            (
                TABLE_OFFSET + 40,
                self.entries[8..]
                    .iter()
                    .flat_map(|e| e.callback.to_le_bytes())
                    .collect(),
                "audio/extended-dispatch/slots-10-11".into(),
            ),
        ])
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("M4A extended dispatch fragment differs from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: AudioExtendedDispatch,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("M4A extended dispatch requires exact European source ROM".into());
        }
        // The prefix reads one selector byte, advances the cursor, then loads
        // the word at TABLE_OFFSET + 4*selector. It does not check a bound.
        for (pc, expected) in [
            (0x149e8a, 0x6c0a),
            (0x149e8c, 0x7813),
            (0x149e8e, 0x3201),
            (0x149e90, 0x640a),
            (0x149e92, 0x4a04),
            (0x149e94, 0x009b),
            (0x149e96, 0x189b),
            (0x149e98, 0x681a),
        ] {
            if u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap()) != expected {
                return Err("M4A extended dispatch reader mismatch".into());
            }
        }
        if u32::from_le_bytes(rom[0x149ea4..0x149ea8].try_into().unwrap()) != BASE + TABLE_OFFSET {
            return Err("M4A extended dispatch literal mismatch".into());
        }
        let entries = SELECTORS
            .into_iter()
            .map(|selector| {
                let at = (TABLE_OFFSET + u32::from(selector) * 4) as usize;
                Entry {
                    selector,
                    callback: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()),
                }
            })
            .collect();
        let decoded = AudioExtendedDispatch {
            source_offset: TABLE_OFFSET,
            entries,
        };
        let spans = decoded.source_spans()?;
        let sources = spans
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

#[cfg(test)]
mod tests;
