//! EUR item-derived four-byte records; other live indices remain guest-owned.
use super::{items, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e84e0;
pub const RECORD_COUNT: usize = 31;
pub const SOURCE_BYTES: usize = RECORD_COUNT * 4;
pub const SOURCE_SHA256: &str = "dd66a69439e7ea175c547855069043dda9f36f09b6825756afc78323da64a843";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub byte00: u8,
    pub byte01: u8,
    pub value02: u16,
}

impl Record {
    pub fn source_bytes(&self) -> [u8; 4] {
        let [low, high] = self.value02.to_le_bytes();
        [self.byte00, self.byte01, low, high]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalItemParameters {
    pub source_offset: u32,
    pub records: Vec<Record>,
}

impl ConditionalItemParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid EUR item-derived parameter projection".into());
        }
        Ok(())
    }

    /// Check only the declared item-derived projection, never guest RAM indices.
    pub fn validate_items(&self, owner: &items::Items) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if owner
            .records
            .iter()
            .any(|item| usize::from(item.bytes_18_1f[0]) >= self.records.len())
        {
            return Err("Item selector has no conditional parameter record".into());
        }
        Ok(())
    }

    pub fn record(&self, index: u32) -> Option<&Record> {
        self.records.get(usize::try_from(index).ok()?)
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let index = usize::try_from(offset.checked_sub(self.source_offset)?).ok()?;
        Some(self.records.get(index / 4)?.source_bytes()[index % 4])
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(Record::source_bytes).collect())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Conditional item parameters differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ConditionalItemParameters,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional item parameters require exact EUR ROM".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        for (pc, literal, target) in [
            (0x16192, 0x161b0, 0x086e84e0),
            (0x19efe, 0x19f20, 0x086e84e0),
            (0x6fda8, 0x6fe08, 0x086e84e0),
            (0x700c4, 0x70110, 0x086e84e0),
            (0x705d0, 0x70604, 0x086e84e0),
            (0x6fd76, 0x6fd94, 0x086e0bd0),
            (0x700a0, 0x700fc, 0x086e0bd0),
            (0x705b2, 0x705f0, 0x086e0bd0),
        ] {
            let instruction = half(pc);
            if instruction & 0xf800 != 0x4800
                || ((pc + 4) & !3) + usize::from(instruction & 255) * 4 != literal
                || u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) != target
            {
                return Err("Conditional item parameter reader literal mismatch".into());
            }
        }
        for (pc, expected) in [
            (0x1618e, 0x89c4),
            (0x16190, 0x00a4),
            (0x16196, 0x7821),
            (0x19efa, 0x89c0),
            (0x19efc, 0x0080),
            (0x19f02, 0x7840),
            (0x6fda2, 0x0141),
            (0x6fda6, 0x7e0f),
            (0x6fdaa, 0x00bf),
            (0x6fdba, 0x7852),
            (0x700b8, 0x0149),
            (0x700bc, 0x7e0f),
            (0x700c6, 0x00b9),
            (0x700ca, 0x884a),
            (0x705ca, 0x0140),
            (0x705ce, 0x7e00),
            (0x705d2, 0x0080),
            (0x705d6, 0x8841),
        ] {
            if half(pc) != expected {
                return Err("Conditional item parameter reader instruction mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + SOURCE_BYTES];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 {
            return Err("Conditional item parameter source mismatch".into());
        }
        let decoded = ConditionalItemParameters {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(4)
                .map(|b| Record {
                    byte00: b[0],
                    byte01: b[1],
                    value02: u16::from_le_bytes([b[2], b[3]]),
                })
                .collect(),
        };
        decoded.validate_items(&items::Catalog::from_rom(rom)?.decoded)?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: SOURCE_BYTES as u32,
                sha256: SOURCE_SHA256.into(),
                role: "conditional-item-parameters".into(),
            }],
        })
    }
}

#[cfg(test)]
mod tests;
