//! European descriptor field00 graphics: bounded offline decoding only.
//! Keep the destination offset and exact compressed source beside decoded words.
use super::{bg_descriptors, custom_lz, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Original full scratch transfer is 0x13f8 words. These are malformed-input
/// guards, independent of extracted JSON size or any performance budget.
pub const DESTINATION_BYTES: usize = 0x4fe0;
pub const MAX_ENCODED_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graphic {
    pub relative_source: u32,
    /// Tag22 retains a live prefix at this byte offset; tag20 starts at zero.
    pub destination_offset: u32,
    /// Guest little-endian words, without palette conversion or discarded bits.
    pub words: Vec<u32>,
    pub encoded_source: Vec<u8>,
}

impl Graphic {
    pub fn decode(input: &[u8], relative_source: u32) -> Result<(Self, usize), String> {
        let input = &input[..input.len().min(MAX_ENCODED_BYTES)];
        let header = |at: usize| -> Result<(u8, usize), String> {
            let bytes = input
                .get(at..at + 4)
                .ok_or("Truncated BG graphics wrapper")?;
            Ok((
                bytes[0],
                (u32::from_le_bytes(bytes.try_into().unwrap()) >> 8) as usize,
            ))
        };
        let (tag, value) = header(0)?;
        let (destination_offset, size, body) = match tag {
            0x20 => (0, value, 4),
            0x22 => {
                let (nested, size) = header(4)?;
                if nested != 0x20 {
                    return Err("BG graphics tag22 requires a nested tag20".into());
                }
                (value, size, 8)
            }
            _ => return Err("Unsupported BG graphics wrapper".into()),
        };
        if size == 0
            || size % 32 != 0
            || destination_offset % 32 != 0
            || destination_offset
                .checked_add(size)
                .is_none_or(|end| end > DESTINATION_BYTES)
        {
            return Err("BG graphics size/offset exceeds the complete tile destination".into());
        }
        let decoded = custom_lz::decode(&input[body..], size, MAX_ENCODED_BYTES - body)?;
        if decoded.bytes.len() != size {
            return Err("BG graphics outer and custom LZ sizes disagree".into());
        }
        let consumed = body + decoded.consumed;
        Ok((
            Self {
                relative_source,
                destination_offset: destination_offset as u32,
                words: decoded
                    .bytes
                    .chunks_exact(4)
                    .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
                    .collect(),
                encoded_source: input[..consumed].to_vec(),
            },
            consumed,
        ))
    }

    pub fn source_bytes(&self) -> Vec<u8> {
        self.words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.words.len() > DESTINATION_BYTES / 4 || self.encoded_source.len() > MAX_ENCODED_BYTES
        {
            return Err("BG graphics words/source exceeds bounds".into());
        }
        let (expected, consumed) = Self::decode(&self.encoded_source, self.relative_source)?;
        if consumed != self.encoded_source.len() || expected != *self {
            return Err("BG graphics words, offset and encoded source disagree".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graphics {
    /// Stable IDs follow first descriptor occurrence; aliases retain every link.
    pub descriptor_graphics: Vec<u16>,
    pub graphics: Vec<Graphic>,
}

impl Graphics {
    pub fn graphic(&self, descriptor: usize) -> Option<&Graphic> {
        self.graphics
            .get(*self.descriptor_graphics.get(descriptor)? as usize)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.descriptor_graphics.is_empty()
            || self.descriptor_graphics.len() > bg_descriptors::MAX_RECORDS
            || self.graphics.is_empty()
            || self.graphics.len() > self.descriptor_graphics.len()
        {
            return Err("Invalid BG graphics descriptor/resource count".into());
        }
        let mut seen = BTreeSet::new();
        for &id in &self.descriptor_graphics {
            if id as usize >= self.graphics.len()
                || (seen.insert(id) && id as usize != seen.len() - 1)
            {
                return Err("Invalid BG graphics ID/first occurrence".into());
            }
        }
        if seen.len() != self.graphics.len() {
            return Err("Unreferenced BG graphic".into());
        }
        let table_bytes = self.descriptor_graphics.len() * bg_descriptors::RECORD_BYTES;
        let mut extents = Vec::new();
        for graphic in &self.graphics {
            graphic.validate()?;
            let start = graphic.relative_source;
            let end = start
                .checked_add(graphic.encoded_source.len() as u32)
                .ok_or("BG graphics source overflow")?;
            if start < table_bytes as u32 {
                return Err("BG graphics source outside payload bounds".into());
            }
            extents.push((start, end));
        }
        extents.sort_unstable();
        if extents.windows(2).any(|p| p[0].1 > p[1].0) {
            return Err("Overlapping BG graphics sources".into());
        }
        Ok(())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("BG graphics differ from the authenticated source ROM".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Graphics,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err(
                "BG graphics currently require the exact European ROM; USA unverified".into(),
            );
        }
        let descriptors = bg_descriptors::Catalog::from_rom(rom)?;
        let mut decoded = Graphics {
            descriptor_graphics: Vec::new(),
            graphics: Vec::new(),
        };
        let mut sources = Vec::new();
        let mut ids = BTreeMap::new();
        for descriptor in &descriptors.decoded.entries {
            let relative = descriptor.graphics_source;
            let id = if let Some(&id) = ids.get(&relative) {
                id
            } else {
                let offset = descriptors
                    .source
                    .offset
                    .checked_add(relative)
                    .ok_or("BG graphics source overflow")?;
                let input = rom
                    .get(offset as usize..)
                    .ok_or("BG graphics source outside ROM")?;
                let (graphic, consumed) = Graphic::decode(input, relative)?;
                let id = decoded.graphics.len() as u16;
                sources.push(SourceRange {
                    offset,
                    length: consumed as u32,
                    sha256: ffta_rom::digest(&input[..consumed]),
                    role: format!("bg-main-graphics/{id}"),
                });
                decoded.graphics.push(graphic);
                ids.insert(relative, id);
                id
            };
            decoded.descriptor_graphics.push(id);
        }
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(offset: Option<u32>) -> Vec<u8> {
        let mut input = Vec::new();
        if let Some(offset) = offset {
            input.extend_from_slice(&((offset << 8) | 0x22).to_le_bytes());
        }
        input.extend_from_slice(&(32u32 << 8 | 0x20).to_le_bytes());
        input.extend_from_slice(&32u32.to_be_bytes());
        input.extend_from_slice(&[0x5f]);
        input.extend(0..32);
        input
    }

    #[test]
    fn wrappers_preserve_words_destination_and_exact_extent() {
        for offset in [None, Some(32), Some(DESTINATION_BYTES as u32 - 32)] {
            let encoded = source(offset);
            let mut padded = encoded.clone();
            padded.extend_from_slice(&[0xcd; 8]);
            let (graphic, consumed) = Graphic::decode(&padded, 0x1000).unwrap();
            assert_eq!(consumed, encoded.len());
            assert_eq!(graphic.encoded_source, encoded);
            assert_eq!(graphic.destination_offset, offset.unwrap_or(0));
            assert_eq!(graphic.source_bytes(), (0..32u8).collect::<Vec<_>>());
            graphic.validate().unwrap();
            for end in 0..consumed {
                assert!(Graphic::decode(&padded[..end], 0x1000).is_err());
            }
        }
    }

    #[test]
    fn malformed_sizes_streams_and_incoherent_words_are_rejected() {
        for offset in [1, 31, DESTINATION_BYTES as u32] {
            assert!(Graphic::decode(&source(Some(offset)), 0x1000).is_err());
        }
        for index in [0, 1, 4, 7, 8] {
            let mut input = source(None);
            input[index] ^= 1;
            assert!(Graphic::decode(&input, 0x1000).is_err());
        }
        let mut input = source(None);
        input[8] = 0x80;
        assert!(Graphic::decode(&input, 0x1000).is_err());
        let mut graphic = Graphic::decode(&source(None), 0x1000).unwrap().0;
        graphic.words[0] ^= 0x8000_0000;
        assert!(graphic.validate().is_err());
        let mut control_bomb = source(None);
        control_bomb.truncate(8);
        control_bomb.resize(MAX_ENCODED_BYTES + 1, 3);
        assert!(Graphic::decode(&control_bomb, 0x1000).is_err());
    }

    #[test]
    fn aliases_require_canonical_ids_and_disjoint_sources() {
        let first = Graphic::decode(&source(None), 0x1000).unwrap().0;
        let second = Graphic::decode(&source(Some(32)), 0x2000).unwrap().0;
        let valid = Graphics {
            descriptor_graphics: vec![0, 0, 1],
            graphics: vec![first, second],
        };
        valid.validate().unwrap();
        assert_eq!(valid.graphic(0), valid.graphic(1));
        for links in [vec![1, 0, 1], vec![0, 0, 0], vec![0, 2, 1]] {
            let mut bad = valid.clone();
            bad.descriptor_graphics = links;
            assert!(bad.validate().is_err());
        }
        let mut bad = valid.clone();
        bad.graphics[1].relative_source = 0x1004;
        assert!(bad.validate().is_err());
        let mut bad = valid;
        bad.graphics[0].relative_source = 0;
        assert!(bad.validate().is_err());
    }
}
