//! Signed offsets read by the original guarded map update.
use super::SourceSpan;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_OFFSET: u32 = 0x3954a4;
pub const ENTRY_COUNT: usize = 48;
pub const LITERAL_OFFSET: u32 = 0x038704;
const TABLE_ADDRESS: u32 = 0x0800_0000 + TABLE_OFFSET;
const TABLE_SHA256: &str = "5b82c01bbd9483b85a5b36f50e8534b5b2b00c2c1a87426db74cc4ad02c8e2a9";
const READER_SHA256: &str = "b4de7c20768fd30a840a4b9e2623ea3939ecdde88388e150f4ee8da933776083";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedMapDeltas {
    pub caller_literal: u32,
    pub entries: Vec<i8>,
}

impl SignedMapDeltas {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let start = TABLE_OFFSET as usize;
        let table = &rom[start..start + ENTRY_COUNT];
        if ffta_rom::digest(table) != TABLE_SHA256
            || ffta_rom::digest(&rom[0x0386ea..0x038700]) != READER_SHA256
        {
            return Err("Signed map delta source or reader fingerprint mismatch".into());
        }
        let literal = LITERAL_OFFSET as usize;
        let result = Self {
            caller_literal: u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()),
            entries: table.iter().map(|&byte| byte as i8).collect(),
        };
        result.validate()?;
        Ok(result)
    }

    fn validate(&self) -> Result<(), String> {
        if self.caller_literal != TABLE_ADDRESS || self.entries.len() != ENTRY_COUNT {
            return Err("Invalid signed map delta table or caller literal".into());
        }
        Ok(())
    }

    pub(super) fn source_spans(&self) -> Result<[SourceSpan; 2], String> {
        self.validate()?;
        Ok([
            (
                LITERAL_OFFSET,
                self.caller_literal.to_le_bytes().to_vec(),
                "signed-map-delta-literal".into(),
            ),
            (
                TABLE_OFFSET,
                self.entries.iter().map(|&entry| entry as u8).collect(),
                "signed-map-deltas".into(),
            ),
        ])
    }

    /// The flag marks payload bytes for the existing raw-transfer counters.
    pub(crate) fn byte_at(&self, offset: u32) -> Option<(u8, bool)> {
        if let Some(index) = offset
            .checked_sub(LITERAL_OFFSET)
            .filter(|&index| index < 4)
        {
            return Some((self.caller_literal.to_le_bytes()[index as usize], false));
        }
        let index = offset.checked_sub(TABLE_OFFSET)? as usize;
        self.entries.get(index).map(|&entry| (entry as u8, true))
    }
}
