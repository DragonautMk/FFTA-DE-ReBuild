//! Source-equal raw UI palette banks copied to the original BG/OBJ shadow RAM.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
mod roots;
pub use roots::{Reference, BANK_ROOTS, REFERENCES};
const ROM_LIMIT: u32 = 32 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub source_offset: u32,
    /// Exact BGR555 storage words, including bit15. Live fades stay in RAM.
    pub colors: Vec<u16>,
}
impl Bank {
    pub fn source_bytes(&self) -> Vec<u8> {
        self.colors.iter().flat_map(|v| v.to_le_bytes()).collect()
    }
    fn validate(&self) -> Result<(), String> {
        if self.source_offset & 1 != 0
            || self.colors.is_empty()
            || self.colors.len() > 512
            || self.colors.len() % 16 != 0
            || self
                .source_offset
                .checked_add((self.colors.len() * 2) as u32)
                .is_none_or(|end| end > ROM_LIMIT)
        {
            return Err("Invalid UI raw palette extent/alignment".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palettes {
    pub banks: Vec<Bank>,
}
impl Palettes {
    pub fn validate(&self) -> Result<(), String> {
        if self.banks.is_empty() || self.banks.len() > 1024 {
            return Err("Invalid UI palette bank count".into());
        }
        for b in &self.banks {
            b.validate()?;
        }
        if self.banks.windows(2).any(|v| {
            v[0].source_offset as usize + v[0].colors.len() * 2 > v[1].source_offset as usize
        }) {
            return Err("Unordered/overlapping UI raw palette banks".into());
        }
        Ok(())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("UI raw palettes differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn word(rom: &[u8], at: u32) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        rom.get(at as usize..at as usize + 4)
            .ok_or("Truncated UI palette root")?
            .try_into()
            .unwrap(),
    ))
}
fn half(rom: &[u8], at: u32) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        rom.get(at as usize..at as usize + 2)
            .ok_or("Truncated UI palette reader")?
            .try_into()
            .unwrap(),
    ))
}
fn literal(rom: &[u8], reader: u32, at: u32, value: u32) -> Result<(), String> {
    let h = half(rom, reader)?;
    if h & 0xf800 != 0x4800
        || ((reader + 4) & !3) + u32::from(h & 255) * 4 != at
        || word(rom, at)? != value
    {
        return Err("UI palette literal provenance mismatch".into());
    }
    Ok(())
}
fn call(rom: &[u8], at: u32, target: u32) -> Result<(), String> {
    let (h, l) = (half(rom, at)?, half(rom, at + 2)?);
    let d = (((u32::from(h & 2047) << 12) | (u32::from(l & 2047) << 1)) as i32) << 9 >> 9;
    if h & 0xf800 != 0xf000
        || l & 0xf800 != 0xf800
        || at.wrapping_add(4).wrapping_add_signed(d) != target
    {
        return Err("UI palette copy call provenance mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Palettes,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("UI raw palettes require exact EUR ROM; USA pending".into());
        }
        // The callback is code, retained in ROM and copied by original startup.
        literal(rom, 0x27ee, 0x2810, 0x08d968e8)?;
        literal(rom, 0x27f0, 0x2814, 0x08d96838)?;
        literal(rom, 0x27f4, 0x2818, 0x03005e78)?;
        call(rom, 0x27fa, 0x145230)?;
        if word(rom, 0x370eb4)? != 0x03005ee9 || half(rom, 0x145c14)? != 0x4718 {
            return Err("UI palette dispatcher/callback mismatch".into());
        }
        let mut banks = Vec::new();
        let mut sources = Vec::new();
        for &(offset, length) in BANK_ROOTS {
            let bytes = rom
                .get(offset as usize..offset as usize + length as usize)
                .ok_or("Truncated raw UI palette")?;
            banks.push(Bank {
                source_offset: offset,
                colors: bytes
                    .chunks_exact(2)
                    .map(|v| u16::from_le_bytes(v.try_into().unwrap()))
                    .collect(),
            });
            sources.push(SourceRange {
                offset,
                length,
                sha256: ffta_rom::digest(bytes),
                role: format!("ui-raw-palette/{offset:08x}"),
            });
        }
        let decoded = Palettes { banks };
        decoded.validate()?;
        for r in REFERENCES {
            literal(
                rom,
                r.reader,
                r.literal,
                0x08000000 + r.source_offset - r.literal_delta,
            )?;
            call(rom, r.call, 0x145c14)?;
            if r.length == 0
                || r.length & 1 != 0
                || r.destination < 0x03003860
                || r.destination
                    .checked_add(r.length)
                    .is_none_or(|end| end > 0x03003c60)
                || !decoded.banks.iter().any(|b| {
                    b.source_offset <= r.source_offset
                        && r.source_offset
                            .checked_add(r.length)
                            .is_some_and(|end| end <= b.source_offset + b.colors.len() as u32 * 2)
                })
            {
                return Err("UI palette reference lacks an exact bank owner".into());
            }
        }
        Ok(Self { decoded, sources })
    }
}
