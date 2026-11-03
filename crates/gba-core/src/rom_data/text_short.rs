//! First bounded EUR short-string catalog (original selector 08016648).
//! Glyph IDs are game codes, not Unicode; uninterpreted control bytes stay exact.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const LANGUAGE_COUNT: usize = 5;
pub const ENTRY_COUNT: usize = 107;
pub const TABLES: [u32; LANGUAGE_COUNT] = [0x725c40, 0x734598, 0x741b9c, 0x750230, 0x75f318];
pub const STRING_STARTS: [u32; LANGUAGE_COUNT] = [0x7257f8, 0x734134, 0x7417a8, 0x74fdec, 0x75ee94];
pub const GLYPH_ROOT: u32 = 0x4954dc;
pub const WIDTH_ROOT: u32 = 0x4f885c;
pub const GLYPH_BYTES: usize = 128;
pub const MAX_STRING_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    Compact,
    Extended,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Unit {
    Glyph {
        index: u16,
    },
    /// Preserves commands and operands without claiming complete semantics.
    ControlByte {
        value: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub source_offset: u32,
    pub encoding: Encoding,
    pub units: Vec<Unit>,
}

impl Text {
    pub fn decode(input: &[u8], source_offset: u32) -> Result<(Self, usize), String> {
        let encoding = match input.first().ok_or("Missing text source")? {
            1 => Encoding::Compact,
            _ => Encoding::Extended,
        };
        let mut at = usize::from(encoding == Encoding::Compact);
        let mut units = Vec::new();
        loop {
            if at >= MAX_STRING_BYTES {
                return Err("Text exceeds source guard".into());
            }
            let byte = *input.get(at).ok_or("Unterminated text source")?;
            at += 1;
            if byte == 0 {
                break;
            }
            let unit = if encoding == Encoding::Compact {
                Unit::Glyph {
                    index: u16::from(byte) - 1,
                }
            } else if byte & 0x80 != 0 {
                let low = *input.get(at).ok_or("Truncated extended glyph")?;
                at += 1;
                Unit::Glyph {
                    index: (u16::from(byte & 0x7f) << 8) | u16::from(low),
                }
            } else {
                Unit::ControlByte { value: byte }
            };
            units.push(unit);
        }
        let result = Self {
            source_offset,
            encoding,
            units,
        };
        result.source_bytes()?;
        Ok((result, at))
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        if self.units.len() > MAX_STRING_BYTES {
            return Err("Text unit count exceeds guard".into());
        }
        let mut bytes = Vec::new();
        if self.encoding == Encoding::Compact {
            bytes.push(1);
        }
        for unit in &self.units {
            match *unit {
                Unit::Glyph { index } if self.encoding == Encoding::Compact && index < 255 => {
                    bytes.push(index as u8 + 1);
                }
                Unit::Glyph { index } if self.encoding == Encoding::Extended && index < 0x8000 => {
                    bytes.extend_from_slice(&(index | 0x8000).to_be_bytes());
                }
                Unit::ControlByte {
                    value: value @ 1..=127,
                } if self.encoding == Encoding::Extended => bytes.push(value),
                _ => return Err("Invalid text encoding unit".into()),
            }
        }
        // A leading 01 is the compact format marker, never an extended byte.
        if self.encoding == Encoding::Extended && bytes.first() == Some(&1) {
            return Err("Ambiguous extended text marker".into());
        }
        bytes.push(0);
        if bytes.len() > MAX_STRING_BYTES {
            return Err("Text source exceeds guard".into());
        }
        Ok(bytes)
    }

    /// Output of original 08019d98, including the final zero byte.
    pub fn expanded_bytes(&self) -> Result<Vec<u8>, String> {
        self.source_bytes()?;
        let mut bytes = Vec::new();
        for unit in &self.units {
            match *unit {
                Unit::Glyph { index } => bytes.extend_from_slice(&(index | 0x8000).to_be_bytes()),
                Unit::ControlByte { value } => bytes.push(value),
            }
        }
        bytes.push(0);
        Ok(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    /// Original selector IDs; human language labels are not guessed here.
    pub language_id: u8,
    pub table_offset: u32,
    pub entries: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Glyph {
    pub index: u16,
    pub width: u8,
    /// Original packed 4bpp slot words; no rearrangement or palette flattening.
    pub tile_words: Vec<u32>,
}
impl Glyph {
    pub fn source_offset(&self) -> u32 {
        GLYPH_ROOT + u32::from(self.index) * GLYPH_BYTES as u32
    }
    pub fn width_offset(&self) -> u32 {
        WIDTH_ROOT + u32::from(self.index)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.tile_words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShortTexts {
    pub languages: Vec<Language>,
    /// IDs follow first pointer occurrence; equal contents need not alias.
    pub strings: Vec<Text>,
    /// Sorted unique game glyph IDs referenced by these strings only.
    pub glyphs: Vec<Glyph>,
}

impl ShortTexts {
    pub fn validate(&self) -> Result<(), String> {
        if self.languages.len() != LANGUAGE_COUNT
            || self.strings.is_empty()
            || self.strings.len() > LANGUAGE_COUNT * ENTRY_COUNT
        {
            return Err("Invalid short-text catalog count".into());
        }
        let mut seen = BTreeSet::new();
        let mut needed = BTreeSet::new();
        let mut spans = Vec::new();
        for (id, language) in self.languages.iter().enumerate() {
            if language.language_id as usize != id
                || language.table_offset != TABLES[id]
                || language.entries.len() != ENTRY_COUNT
            {
                return Err("Invalid short-text language/index table".into());
            }
            for &entry in &language.entries {
                let text = self.strings.get(entry as usize).ok_or("Missing text ID")?;
                let length = text.source_bytes()?.len() as u32;
                if text.source_offset < STRING_STARTS[id]
                    || text
                        .source_offset
                        .checked_add(length)
                        .is_none_or(|end| end > TABLES[id])
                {
                    return Err("Text outside its declared language payload".into());
                }
                if seen.insert(entry) {
                    if entry as usize != seen.len() - 1 {
                        return Err("Noncanonical text ID".into());
                    }
                    spans.push((text.source_offset, text.source_offset + length));
                    for unit in &text.units {
                        if let Unit::Glyph { index } = unit {
                            needed.insert(*index);
                        }
                    }
                }
            }
            spans.push((TABLES[id], TABLES[id] + ENTRY_COUNT as u32 * 4));
        }
        if seen.len() != self.strings.len() || self.glyphs.len() != needed.len() {
            return Err("Unreferenced short-text resource".into());
        }
        for (glyph, &index) in self.glyphs.iter().zip(needed.iter()) {
            if glyph.index != index
                || glyph.tile_words.len() != GLYPH_BYTES / 4
                || glyph.source_offset() + GLYPH_BYTES as u32 > WIDTH_ROOT
            {
                return Err("Invalid short-text glyph extent/order".into());
            }
            spans.push((
                glyph.source_offset(),
                glyph.source_offset() + GLYPH_BYTES as u32,
            ));
            spans.push((glyph.width_offset(), glyph.width_offset() + 1));
        }
        spans.sort_unstable();
        if spans.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err("Overlapping short-text source ranges".into());
        }
        Ok(())
    }

    pub fn table_bytes(&self, language: &Language) -> Result<Vec<u8>, String> {
        language
            .entries
            .iter()
            .map(|&id| {
                self.strings
                    .get(id as usize)
                    .ok_or_else(|| "Missing text ID".to_string())
                    .and_then(|text| {
                        text.source_offset
                            .checked_add(0x08000000)
                            .map(u32::to_le_bytes)
                            .ok_or_else(|| "Text pointer overflow".to_string())
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|words| words.into_iter().flatten().collect())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("Short texts differ from authenticated EUR ROM".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ShortTexts,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Short texts require exact EUR ROM; USA unverified".into());
        }
        let mut decoded = ShortTexts {
            languages: Vec::new(),
            strings: Vec::new(),
            glyphs: Vec::new(),
        };
        let mut ids = BTreeMap::new();
        let mut needed = BTreeSet::new();
        for (language_id, &table_offset) in TABLES.iter().enumerate() {
            let mut entries = Vec::new();
            let table = &rom[table_offset as usize..table_offset as usize + ENTRY_COUNT * 4];
            for word in table.chunks_exact(4) {
                let source = u32::from_le_bytes(word.try_into().unwrap())
                    .checked_sub(0x08000000)
                    .ok_or("Text pointer outside cartridge")?;
                if source < STRING_STARTS[language_id] || source >= table_offset {
                    return Err("Text pointer outside authenticated catalog".into());
                }
                let id = if let Some(&id) = ids.get(&source) {
                    id
                } else {
                    let (text, _) =
                        Text::decode(&rom[source as usize..table_offset as usize], source)?;
                    for unit in &text.units {
                        if let Unit::Glyph { index } = unit {
                            needed.insert(*index);
                        }
                    }
                    let id = decoded.strings.len() as u16;
                    decoded.strings.push(text);
                    ids.insert(source, id);
                    id
                };
                entries.push(id);
            }
            decoded.languages.push(Language {
                language_id: language_id as u8,
                table_offset,
                entries,
            });
        }
        for index in needed {
            let start = GLYPH_ROOT as usize + index as usize * GLYPH_BYTES;
            let end = start + GLYPH_BYTES;
            if end > WIDTH_ROOT as usize {
                return Err("Glyph outside font payload guard".into());
            }
            let words = rom.get(start..end).ok_or("Truncated font slot")?;
            decoded.glyphs.push(Glyph {
                index,
                width: rom[WIDTH_ROOT as usize + index as usize],
                tile_words: words
                    .chunks_exact(4)
                    .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                    .collect(),
            });
        }
        decoded.validate()?;
        let mut sources = Vec::new();
        let mut add = |offset: u32, bytes: Vec<u8>, role: String| {
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        };
        for language in &decoded.languages {
            add(
                language.table_offset,
                decoded.table_bytes(language)?,
                format!("short-text-table/{}", language.language_id),
            );
        }
        for (id, text) in decoded.strings.iter().enumerate() {
            add(
                text.source_offset,
                text.source_bytes()?,
                format!("short-text-string/{id}"),
            );
        }
        for glyph in &decoded.glyphs {
            add(
                glyph.source_offset(),
                glyph.source_bytes(),
                format!("short-text-glyph/{}", glyph.index),
            );
            add(
                glyph.width_offset(),
                vec![glyph.width],
                format!("short-text-width/{}", glyph.index),
            );
        }
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_and_extended_preserve_low_zero_glyphs_and_control_bytes() {
        let (compact, consumed) = Text::decode(&[1, 1, 255, 0, 99], 0).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(compact.expanded_bytes().unwrap(), [128, 0, 128, 254, 0]);
        assert_eq!(compact.source_bytes().unwrap(), [1, 1, 255, 0]);
        let (wide, consumed) = Text::decode(&[128, 0, 64, 62, 3, 128, 254, 0], 0).unwrap();
        assert_eq!(consumed, 8);
        assert_eq!(
            wide.source_bytes().unwrap(),
            [128, 0, 64, 62, 3, 128, 254, 0]
        );
        assert_eq!(wide.expanded_bytes().unwrap(), wide.source_bytes().unwrap());
        assert!(Text::decode(&[128], 0).is_err());
        assert!(Text::decode(&[1, 2], 0).is_err());
        assert!(Text::decode(&vec![2; MAX_STRING_BYTES + 1], 0).is_err());
        let mut bad = compact;
        bad.units.push(Unit::Glyph { index: 255 });
        assert!(bad.source_bytes().is_err());
        bad.encoding = Encoding::Extended;
        bad.units.insert(0, Unit::ControlByte { value: 1 });
        assert!(bad.source_bytes().is_err());
        bad.source_offset = u32::MAX;
        let catalog = ShortTexts {
            languages: Vec::new(),
            strings: vec![bad],
            glyphs: Vec::new(),
        };
        let language = Language {
            language_id: 0,
            table_offset: TABLES[0],
            entries: vec![0],
        };
        assert!(catalog.table_bytes(&language).is_err());
    }
}
