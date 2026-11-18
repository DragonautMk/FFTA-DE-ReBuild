//! Conditional EUR source words beside the witnessed substitution callbacks.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const DISPATCH_LITERAL: u32 = 0x019eb4;
pub const TABLE_OFFSET: u32 = 0x371158;
pub const TABLE_END: u32 = 0x37140c;
pub const WORD_COUNT: usize = 98;
pub const SOURCE_SHA256: &str = "aca1f76561e29756c7b75c438413df33676e16cc0ed4e8360efad55bcdb0c272";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalWord {
    pub index: u8,
    /// Preserve the original bits without inferring callback behavior or padding.
    pub word: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalSubstitutionWords {
    pub table_offset: u32,
    pub words: Vec<ConditionalWord>,
}
impl ConditionalSubstitutionWords {
    pub fn validate(&self) -> Result<(), String> {
        if self.table_offset != TABLE_OFFSET
            || self.words.len() != WORD_COUNT
            || self
                .words
                .iter()
                .any(|w| u32::from(w.index) * 4 + TABLE_OFFSET >= TABLE_END)
            || self.words.windows(2).any(|v| v[0].index >= v[1].index)
        {
            return Err("Invalid conditional substitution words".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans = Vec::with_capacity(1 + self.words.len());
        spans.push((
            DISPATCH_LITERAL,
            (0x0800_0000u32 + self.table_offset).to_le_bytes().to_vec(),
            "dispatcher-literal".into(),
        ));
        for word in &self.words {
            spans.push((
                self.table_offset + 4 * u32::from(word.index),
                word.word.to_le_bytes().to_vec(),
                format!("conditional-substitution-word/{}", word.index),
            ));
        }
        Ok(spans)
    }
}

pub struct Catalog {
    pub decoded: ConditionalSubstitutionWords,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional substitution words require the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0x19e94..0x19eb8])
            != "9b33d17cd67fa6242efc06b239135e0a6ddf9a0db64f0b1a4246b6b2de78efd9"
            || u32::from_le_bytes(
                rom[DISPATCH_LITERAL as usize..DISPATCH_LITERAL as usize + 4]
                    .try_into()
                    .unwrap(),
            ) != 0x0837_1158
        {
            return Err("Conditional substitution dispatcher mismatch".into());
        }
        let prior = PriorSources::new(previous)?;
        if prior.contains(DISPATCH_LITERAL, 4)? {
            return Err("Conditional substitution literal already has an owner".into());
        }
        let mut words = Vec::new();
        for index in 0..((TABLE_END - TABLE_OFFSET) / 4) {
            let offset = TABLE_OFFSET + index * 4;
            if !prior.contains(offset, 4)? {
                let at = offset as usize;
                words.push(ConditionalWord {
                    index: index as u8,
                    word: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()),
                });
            }
        }
        let decoded = ConditionalSubstitutionWords {
            table_offset: TABLE_OFFSET,
            words,
        };
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? {
                return Err("Conditional substitution source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Conditional substitution source reconstruction mismatch".into());
            }
            combined.extend_from_slice(&bytes);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        if ffta_rom::digest(&combined) != SOURCE_SHA256 {
            return Err("Conditional substitution source hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
