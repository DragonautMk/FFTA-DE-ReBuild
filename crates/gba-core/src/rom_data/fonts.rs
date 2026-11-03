//! Complete bounded EUR packed font bank. Glyph3174is referenced by name entry.
use super::{
    text_short::{Glyph, GLYPH_BYTES, GLYPH_ROOT, WIDTH_ROOT},
    SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const GLYPH_COUNT: usize = 3175;
pub const BANK_BYTES: usize = GLYPH_COUNT * GLYPH_BYTES;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontBank {
    pub glyphs: Vec<Glyph>,
}
impl FontBank {
    pub fn decode(bank: &[u8], widths: &[u8]) -> Result<Self, String> {
        if bank.len() != BANK_BYTES
            || widths.len() != GLYPH_COUNT
            || GLYPH_ROOT + BANK_BYTES as u32 != WIDTH_ROOT
        {
            return Err("Invalid complete font source extents".into());
        }
        let result = Self {
            glyphs: bank
                .chunks_exact(GLYPH_BYTES)
                .zip(widths)
                .enumerate()
                .map(|(index, (bytes, &width))| Glyph {
                    index: index as u16,
                    width,
                    tile_words: bytes
                        .chunks_exact(4)
                        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                        .collect(),
                })
                .collect(),
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.glyphs.len() != GLYPH_COUNT
            || self.glyphs.iter().enumerate().any(|(index, g)| {
                g.index as usize != index || g.tile_words.len() != GLYPH_BYTES / 4
            })
        {
            return Err("Invalid complete font slot count/order/words".into());
        }
        Ok(())
    }
    pub fn bank_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.glyphs.iter().flat_map(|g| g.source_bytes()).collect())
    }
    pub fn width_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.glyphs.iter().map(|g| g.width).collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("Font bank differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FontBank,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Font bank requires exact EUR ROM; USA unverified".into());
        }
        let bank = rom
            .get(GLYPH_ROOT as usize..WIDTH_ROOT as usize)
            .ok_or("Truncated font bank")?;
        let widths = rom
            .get(WIDTH_ROOT as usize..WIDTH_ROOT as usize + GLYPH_COUNT)
            .ok_or("Truncated font metrics")?;
        Ok(Self {
            decoded: FontBank::decode(bank, widths)?,
            sources: vec![
                SourceRange {
                    offset: GLYPH_ROOT,
                    length: BANK_BYTES as u32,
                    sha256: ffta_rom::digest(bank),
                    role: "font-bank-packed-words".into(),
                },
                SourceRange {
                    offset: WIDTH_ROOT,
                    length: GLYPH_COUNT as u32,
                    sha256: ffta_rom::digest(widths),
                    role: "font-bank-widths".into(),
                },
            ],
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_font_bank_preserves_last_slot_metric_and_full_words() {
        let bank: Vec<_> = (0..BANK_BYTES)
            .map(|i| (i as u8).wrapping_mul(73))
            .collect();
        let widths: Vec<_> = (0..GLYPH_COUNT).map(|i| i as u8).collect();
        let mut decoded = FontBank::decode(&bank, &widths).unwrap();
        assert_eq!(decoded.bank_bytes().unwrap(), bank);
        assert_eq!(decoded.width_bytes().unwrap(), widths);
        assert_eq!(decoded.glyphs[3174].source_offset() + 128, WIDTH_ROOT);
        assert_eq!(decoded.glyphs[3174].width_offset() + 1, WIDTH_ROOT + 3175);
        assert!(FontBank::decode(&bank[..bank.len() - 1], &widths).is_err());
        decoded.glyphs[3174].index = 3173;
        assert!(decoded.validate().is_err());
    }
}
