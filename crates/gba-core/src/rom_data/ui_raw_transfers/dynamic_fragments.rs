//! Bounded dynamic-transfer fragments; guest selector arithmetic remains unchanged.
use super::{BankView, SourceSpan};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

const ROW_STARTS: [u32; 2] = [0x399700, 0x399718];
const BANKS: [u32; 5] = [0x441288, 0x4528fc, 0x464498, 0x475460, 0x486f30];
const SOURCE_SHA256: &str = "09c6e92d1483c47b38ef9e905b0f5df6c47f1233b0131d6a7856ab4af90e768b";

fn invalid() -> String { "Invalid dynamic raw-transfer fragments".into() }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rows {
    pub source_offset: u32,
    /// Three pairs for selectors 7, 8 and 9; halfwords are shifted by five by guest code.
    pub pairs: [[u16; 2]; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fragments {
    pub rows: Vec<Rows>,
    pub banks: Vec<BankView>,
}

impl Fragments {
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.rows.len() != ROW_STARTS.len() || self.banks.len() != BANKS.len() {
            return Err(invalid());
        }
        let mut spans = Vec::new();
        for (row, expected) in self.rows.iter().zip(ROW_STARTS) {
            if row.source_offset != expected { return Err(invalid()); }
            // Each 64-byte view must fit entirely within the declared prefix.
            if row.pairs.iter().flatten().any(|&value| !(8..=14).contains(&value)) {
                return Err(invalid());
            }
            let bytes = row.pairs.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
            spans.push((row.source_offset, bytes, format!("dynamic-rows/{:08x}", row.source_offset)));
        }
        for (bank, expected) in self.banks.iter().zip(BANKS) {
            if bank.bank_offset != expected || bank.relative_offset != 0x100 || bank.words.len() != 64 {
                return Err(invalid());
            }
            spans.push((bank.source_offset().ok_or_else(invalid)?, bank.source_bytes(), format!("dynamic-prefix/{:08x}", bank.bank_offset)));
        }
        Ok(spans)
    }

    pub fn validate(&self) -> Result<(), String> { self.source_spans().map(|_| ()) }

    /// Returns declared fragment positions; all other offsets remain unowned.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        for row in &self.rows {
            if let Some(i) = offset.checked_sub(row.source_offset).filter(|v| *v < 12) {
                return Some(row.pairs[i as usize / 4][(i as usize % 4) / 2].to_le_bytes()[i as usize % 2]);
            }
        }
        self.banks.iter().find_map(|bank| bank.byte_at(offset))
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != from_rom(rom)? { return Err("Dynamic fragments differ from authenticated EUR source".into()); }
        Ok(())
    }
}

pub fn from_rom(rom: &[u8]) -> Result<Fragments, String> {
    if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
        return Err("Dynamic fragments require exact EUR ROM; USA pending".into());
    }
    let rows = ROW_STARTS.into_iter().map(|offset| {
        let mut pairs = [[0; 2]; 3];
        for (i, value) in pairs.iter_mut().flatten().enumerate() {
            let at = offset as usize + 2*i;
            *value = u16::from_le_bytes(rom[at..at+2].try_into().unwrap());
        }
        Rows { source_offset: offset, pairs }
    }).collect();
    let banks = BANKS.into_iter().map(|bank_offset| {
        let at = bank_offset as usize + 0x100;
        BankView { bank_offset, relative_offset: 0x100, words: rom[at..at+0x100].chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect() }
    }).collect();
    let data = Fragments { rows, banks };
    let bytes: Vec<u8> = data.source_spans()?.into_iter().flat_map(|v| v.1).collect();
    if ffta_rom::digest(&bytes) != SOURCE_SHA256 { return Err("Dynamic fragment source fingerprint mismatch".into()); }
    Ok(data)
}

pub struct Catalog {
    pub decoded: Fragments,
    pub sources: Vec<super::SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let decoded = from_rom(rom)?;
        let sources = decoded.source_spans()?.into_iter().map(|(offset, bytes, role)| super::SourceRange {
            offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role,
        }).collect();
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests;
