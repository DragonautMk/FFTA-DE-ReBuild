//! The unowned complement of the original UI LDRB property-input window.
//! This address view is not a sprite catalog or a cap on the general getter.
use super::{color_mode_palettes, unit_obj_property_words, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SOURCE_BASE: u32 = unit_obj_property_words::SOURCE_BASE;
pub const SOURCE_SHA256: &str = "9d1239cc29ec40185c1605aed84b4cf74cedbc21cc217c001efb8348407253f9";
pub type SourceSpan = (u32, Vec<u8>, String);

pub fn owned_inputs() -> impl Iterator<Item = u8> {
    [112, 117].into_iter().chain(124..=187).chain(250..=255)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Word {
    pub input_byte: u8,
    pub source_offset: u32,
    pub word: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjBytePropertyWords {
    pub words: Vec<Word>,
}
impl UnitObjBytePropertyWords {
    pub fn validate(&self) -> Result<(), String> {
        if self.words.len() != 72
            || !self.words.iter().zip(owned_inputs()).all(|(word, input)| {
                word.input_byte == input && word.source_offset == SOURCE_BASE + 2 * u32::from(input)
            })
        {
            return Err("UI byte propertyword identity/order/source mismatch".into());
        }
        Ok(())
    }
    /// The two dependencies own the other 184 queries. No source is duplicated.
    pub fn validate_shared(
        &self,
        properties: &unit_obj_property_words::UnitObjPropertyWords,
        palettes: &color_mode_palettes::ColorModePalettes,
    ) -> Result<(), String> {
        self.validate()?;
        properties.validate()?;
        palettes.validate()?;
        let mut inputs: BTreeSet<_> = self.words.iter().map(|v| u16::from(v.input_byte)).collect();
        for word in &properties.words {
            if word.sprite_id > 255 || !inputs.insert(word.sprite_id) {
                return Err("UI byte propertyword shared owner overlap/domain mismatch".into());
            }
        }
        for input in [248, 249] {
            let at = SOURCE_BASE + 2 * u32::from(input);
            if !inputs.insert(input)
                || palettes.byte_at(at).is_none()
                || palettes.byte_at(at + 1).is_none()
            {
                return Err("UI byte propertyword palette alias mismatch".into());
            }
        }
        if inputs != (0..=255).collect() {
            return Err("UI byte propertyword shared window incomplete".into());
        }
        Ok(())
    }
    /// Query the validated byte-input view, including its existing typed aliases.
    pub fn word_for_input(
        &self,
        input: u8,
        properties: &unit_obj_property_words::UnitObjPropertyWords,
        palettes: &color_mode_palettes::ColorModePalettes,
    ) -> Option<u16> {
        if let Some(word) = self.words.iter().find(|v| v.input_byte == input) {
            return Some(word.word);
        }
        if let Some(word) = properties
            .words
            .iter()
            .find(|v| v.sprite_id == u16::from(input))
        {
            return Some(word.word);
        }
        let at = SOURCE_BASE + 2 * u32::from(input);
        Some(u16::from_le_bytes([
            palettes.byte_at(at)?,
            palettes.byte_at(at + 1)?,
        ]))
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans: Vec<SourceSpan> = Vec::new();
        for word in &self.words {
            let bytes = word.word.to_le_bytes();
            if let Some((at, data, _)) = spans.last_mut() {
                if *at + data.len() as u32 == word.source_offset {
                    data.extend_from_slice(&bytes);
                    continue;
                }
            }
            spans.push((
                word.source_offset,
                bytes.to_vec(),
                "byte-property-word".into(),
            ));
        }
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("UI byte propertyword model differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: UnitObjBytePropertyWords,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("UI byte propertywords require exact EUR ROM; USA pending".into());
        }
        for (at, size, expected) in [
            (
                0x221c4,
                16,
                "bb5cbae174700ce609bb594d543334e9c862b5f624bb409cda8fdc285ba282af",
            ),
            (
                0x2aaa0,
                248,
                "5f7e4f15f09d8b4b88ea317913809f9c1c1b47b0d072e06c7621b768080841d7",
            ),
            (
                0x2ab98,
                736,
                "4a7a19eeaaa5ad84fb3823951183806528f63625f4349fdd0ddd1d36d87b4dd3",
            ),
        ] {
            if ffta_rom::digest(&rom[at..at + size]) != expected {
                return Err("UI byte propertyword original producer/reader mismatch".into());
            }
        }
        let properties = unit_obj_property_words::Catalog::from_rom(rom)?.decoded;
        let palettes = color_mode_palettes::Catalog::from_rom(rom)?.decoded;
        let decoded = UnitObjBytePropertyWords {
            words: owned_inputs()
                .map(|input_byte| {
                    let at = SOURCE_BASE + 2 * u32::from(input_byte);
                    Word {
                        input_byte,
                        source_offset: at,
                        word: u16::from_le_bytes(
                            rom[at as usize..at as usize + 2].try_into().unwrap(),
                        ),
                    }
                })
                .collect(),
        };
        decoded.validate_shared(&properties, &palettes)?;
        let spans = decoded.source_spans()?;
        let combined: Vec<_> = spans
            .iter()
            .flat_map(|(_, bytes, _)| bytes.iter().copied())
            .collect();
        if spans.len() != 4 || combined.len() != 144 || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("UI byte propertyword source contract mismatch".into());
        }
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
