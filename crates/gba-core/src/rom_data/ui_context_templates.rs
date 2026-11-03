//! EUR UI context templates selected by the original category dispatcher.
//! The projection covers that selector's image, not a global UI catalog.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x394e44;
pub const END_OFFSET: u32 = 0x394f20;
pub const RECORD_BYTES: usize = 20;
pub const RECORD_COUNT: usize = 11;
pub const SOURCE_BYTES: usize = RECORD_BYTES * RECORD_COUNT;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub x_units8: u8,
    pub y_units8: u8,
    pub opaque_02_03: [u8; 2],
    pub entry_count: u16,
    pub opaque_06_08: [u8; 3],
    pub flags: u8,
    pub opaque_0a_13: [u8; 10],
}

impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0 => Some(self.x_units8),
            1 => Some(self.y_units8),
            2..=3 => Some(self.opaque_02_03[offset - 2]),
            4..=5 => Some(self.entry_count.to_le_bytes()[offset - 4]),
            6..=8 => Some(self.opaque_06_08[offset - 6]),
            9 => Some(self.flags),
            10..=19 => Some(self.opaque_0a_13[offset - 10]),
            _ => None,
        }
    }

    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiContextTemplates {
    pub records: Vec<Record>,
}

impl UiContextTemplates {
    pub fn validate(&self) -> Result<(), String> {
        if self.records.len() != RECORD_COUNT {
            return Err("Invalid UI context template projection count".into());
        }
        // Numeric values retain their full original bit widths. In particular,
        // opaque bytes are not assumed to be padding or unused fields.
        Ok(())
    }

    /// A projection miss does not introduce a guest index bound.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(SOURCE_OFFSET)? as usize;
        self.records.get(relative / RECORD_BYTES)?.byte(relative % RECORD_BYTES)
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(Record::source_bytes).collect())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("UI context templates differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: UiContextTemplates,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("UI context templates require the exact EUR ROM; USA pending".into());
        }
        let source = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let records = source.chunks_exact(RECORD_BYTES).map(|bytes| Record {
            x_units8: bytes[0],
            y_units8: bytes[1],
            opaque_02_03: bytes[2..4].try_into().unwrap(),
            entry_count: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            opaque_06_08: bytes[6..9].try_into().unwrap(),
            flags: bytes[9],
            opaque_0a_13: bytes[10..20].try_into().unwrap(),
        }).collect();
        let decoded = UiContextTemplates { records };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: SOURCE_BYTES as u32,
                sha256: ffta_rom::digest(source),
                role: "ui-context-templates".into(),
            }],
        })
    }
}
