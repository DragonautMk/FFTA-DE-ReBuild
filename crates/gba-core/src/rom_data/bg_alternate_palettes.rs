//! The verified two alternate BG palette catalogs and their custom LZ/RLE codec.
//! Offline decoding only: original guest instructions, writes and timing remain.
use super::{bg_descriptors, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const BASE_KEYS: usize = 157;
pub const BANKS: usize = 8;
pub const WORDS_PER_BANK: usize = 16;
pub const OUTPUT_BYTES: usize = BANKS * WORDS_PER_BANK * 2;
/// Malformed streams may contain nonproducing control bytes. Bound those reads
/// independently of the output size; this is not a compressed-size target.
pub const MAX_ENCODED_BYTES: usize = 4096;
const TABLE_BYTES: usize = BASE_KEYS * 2;
const TABLE_HASHES: [&str; 2] = [
    "a5210098f1c8709783894f51ef5b3e42de2b774b1c9995a6d0801e6ddaa615f8",
    "53924aa8dd833e8a1ec429dbdaff5f407ad24eb874c9586642150e5225ff6eb5",
];

pub use super::custom_lz::{Block, LiteralSpan};

/// Preserve the alternate palette's independent 256-byte/4096-byte guards.
pub fn decode(input: &[u8], output_budget: usize) -> Result<Block, String> {
    super::custom_lz::decode(input, output_budget.min(OUTPUT_BYTES), MAX_ENCODED_BYTES)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    /// Indexed full-u16 banks; bit15 is preserved without assigning alpha.
    pub banks: [[u16; WORDS_PER_BANK]; BANKS],
    /// Exact compatibility view, not a newly recompressed representation.
    pub encoded_source: Vec<u8>,
}

impl Palette {
    pub fn decode(input: &[u8]) -> Result<(Self, usize), String> {
        let block = decode(input, OUTPUT_BYTES)?;
        if block.bytes.len() != OUTPUT_BYTES {
            return Err("Alternate palette must contain eight complete banks".into());
        }
        Ok((
            Self {
                banks: std::array::from_fn(|bank| {
                    std::array::from_fn(|word| {
                        let index = (bank * WORDS_PER_BANK + word) * 2;
                        u16::from_le_bytes([block.bytes[index], block.bytes[index + 1]])
                    })
                }),
                encoded_source: input[..block.consumed].to_vec(),
            },
            block.consumed,
        ))
    }

    pub fn source_bytes(&self) -> [u8; OUTPUT_BYTES] {
        std::array::from_fn(|index| {
            self.banks[index / 32][index / 2 % WORDS_PER_BANK].to_le_bytes()[index % 2]
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.encoded_source.len() > MAX_ENCODED_BYTES {
            return Err("Alternate palette encoded source exceeds bound".into());
        }
        let block = decode(&self.encoded_source, OUTPUT_BYTES)?;
        if block.consumed != self.encoded_source.len() || block.bytes != self.source_bytes() {
            return Err("Alternate palette banks and source representation disagree".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palettes {
    /// Original descriptor byte56, including unavailable ff; never dereference it.
    pub descriptor_keys: Vec<u8>,
    /// Source compatibility metadata, relative to each catalog's own root.
    pub relative_sources: [Vec<u16>; 2],
    /// Catalog0 is environment low bits2; catalog1 is low bits1.
    pub catalogs: [Vec<Palette>; 2],
}

impl Palettes {
    pub fn get(&self, descriptor: usize, environment: u8) -> Option<&Palette> {
        let group = match environment & 3 {
            2 => 0,
            1 => 1,
            _ => return None,
        };
        let key = *self.descriptor_keys.get(descriptor)?;
        if key == 255 {
            return None;
        }
        self.catalogs[group].get(key as usize)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.descriptor_keys.len() != bg_descriptors::BASE_RECORDS
            || self
                .catalogs
                .iter()
                .any(|entries| entries.len() != BASE_KEYS)
            || self
                .relative_sources
                .iter()
                .any(|links| links.len() != BASE_KEYS)
        {
            return Err("Invalid alternate palette descriptor/catalog count".into());
        }
        let mut keys = [false; BASE_KEYS];
        for &key in &self.descriptor_keys {
            if key == 255 {
                continue;
            }
            let seen = keys
                .get_mut(key as usize)
                .ok_or("Invalid alternate palette key")?;
            if *seen {
                return Err("Duplicate alternate palette key".into());
            }
            *seen = true;
        }
        if keys.contains(&false) {
            return Err("Incomplete alternate palette keys".into());
        }
        for (palettes, links) in self.catalogs.iter().zip(&self.relative_sources) {
            let mut ranges = Vec::new();
            for (palette, &relative) in palettes.iter().zip(links) {
                palette.validate()?;
                let start = relative as usize;
                if start < TABLE_BYTES {
                    return Err("Alternate palette source overlaps link table".into());
                }
                ranges.push((start, start + palette.encoded_source.len()));
            }
            ranges.sort_unstable();
            if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
                return Err("Overlapping alternate palette source extents".into());
            }
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Palettes,
    /// The two exact relative-link tables. Padding is not part of either table.
    pub tables: [SourceRange; 2],
    /// Group-major/key-minor source ranges, including every actual stream.
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let descriptors = bg_descriptors::Catalog::from_rom(rom)?;
        let (roots, literals) = match ffta_rom::digest(rom).as_str() {
            ffta_rom::SHA256 => ([0x427a04usize, 0x42cc80], [0x1a4f8, 0x1a514]),
            ffta_rom::EUROPE_SHA256 => ([0x489bb0usize, 0x48ee2c], [0x1b668, 0x1b684]),
            _ => return Err("Alternate palettes require an exact supported ROM".into()),
        };
        let mut catalogs = [Vec::new(), Vec::new()];
        let mut relative_sources = [Vec::new(), Vec::new()];
        let mut tables = Vec::new();
        let mut sources = Vec::new();
        for group in 0..2 {
            let root = roots[group];
            if rom.get(literals[group]..literals[group] + 4)
                != Some(&(0x0800_0000u32 + root as u32).to_le_bytes()[..])
            {
                return Err("Alternate palette root literal mismatch".into());
            }
            let links = rom
                .get(root..root + TABLE_BYTES)
                .ok_or("Truncated alternate palette table")?;
            if ffta_rom::digest(links) != TABLE_HASHES[group] {
                return Err("Alternate palette link-table source mismatch".into());
            }
            tables.push(SourceRange {
                offset: root as u32,
                length: TABLE_BYTES as u32,
                sha256: TABLE_HASHES[group].into(),
                role: format!("alternate-palette-links/{group}"),
            });
            let mut ranges = Vec::new();
            for key in 0..BASE_KEYS {
                let relative = u16::from_le_bytes([links[key * 2], links[key * 2 + 1]]) as usize;
                if relative < TABLE_BYTES {
                    return Err("Alternate palette link overlaps table".into());
                }
                let offset = root
                    .checked_add(relative)
                    .ok_or("Alternate palette link overflow")?;
                let input = rom
                    .get(offset..)
                    .ok_or("Alternate palette link outside ROM")?;
                let (palette, consumed) = Palette::decode(input)?;
                sources.push(SourceRange {
                    offset: offset as u32,
                    length: consumed as u32,
                    sha256: ffta_rom::digest(&input[..consumed]),
                    role: format!("alternate-palette/{group}/{key}"),
                });
                catalogs[group].push(palette);
                relative_sources[group].push(relative as u16);
                ranges.push((offset, offset + consumed));
            }
            ranges.sort_unstable();
            if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
                return Err("Overlapping alternate palette streams".into());
            }
        }
        let decoded = Palettes {
            descriptor_keys: descriptors
                .decoded
                .entries
                .iter()
                .map(|entry| entry.alternate_palette_key)
                .collect(),
            relative_sources,
            catalogs,
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            tables: tables.try_into().unwrap(),
            sources,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(size: u32, tokens: &[u8]) -> Vec<u8> {
        [size.to_be_bytes().as_slice(), tokens].concat()
    }

    #[test]
    fn literals_all_copy_and_fill_classes_preserve_overlap_and_extent() {
        for (tokens, expected) in [
            (vec![0x43, 1, 2, 3, 4, 0x80, 3], vec![1, 2, 3, 4, 1, 2, 3]),
            (vec![0x40, 0xa5, 0xf8, 0], vec![0xa5; 19]),
            (vec![0x40, 0x81, 0x1f, 0xc0, 0], vec![0x81; 68]),
            (vec![0x40, 0xff, 0, 250, 0, 0], vec![0xff; 256]),
            (vec![0x20, 0x3f], vec![0; 35]),
            (vec![1, 253], vec![0xff; 256]),
            (vec![2, 253], vec![0; 256]),
            (vec![3, 15, 0x40, 0x80], vec![0x80]),
        ] {
            let mut source = stream(expected.len() as u32, &tokens);
            let extent = source.len();
            source.extend_from_slice(&[0xcc; 16]);
            let block = decode(&source, OUTPUT_BYTES).unwrap();
            assert_eq!(block.bytes, expected);
            assert_eq!(block.consumed, extent);
            for truncated in 0..extent {
                assert!(decode(&source[..truncated], OUTPUT_BYTES).is_err());
            }
        }
    }

    #[test]
    fn malformed_headers_tokens_history_and_budgets_are_rejected() {
        for source in [
            vec![],
            vec![0; 3],
            stream(0, &[]),
            stream(257, &[]),
            stream(u32::MAX, &[]),
            stream(1, &[0x41, 1, 2]),
            stream(3, &[0x80, 0]),
            stream(4, &[0x40, 1, 0x80, 1]),
            stream(5, &[0x10, 0xff, 0xff]),
            stream(5, &[0, 0, 0xff, 0xff]),
            stream(2, &[1, 0]),
            stream(1, &vec![3; MAX_ENCODED_BYTES]),
        ] {
            assert!(decode(&source, OUTPUT_BYTES).is_err(), "source={source:?}");
        }
        let valid = stream(OUTPUT_BYTES as u32, &[1, 253]);
        for length in 0..valid.len() {
            assert!(decode(&valid[..length], OUTPUT_BYTES).is_err());
        }
        assert!(decode(&valid, OUTPUT_BYTES - 1).is_err());
        assert!(decode(&valid, 0).is_err());
        for token in [0x80, 0x40, 0x10, 0, 1, 2] {
            assert!(decode(&stream(16, &[token]), OUTPUT_BYTES).is_err());
        }
    }

    #[test]
    fn typed_banks_preserve_full_words_and_reject_edits_or_trailing_source() {
        let source = stream(OUTPUT_BYTES as u32, &[1, 253]);
        let (mut palette, consumed) = Palette::decode(&source).unwrap();
        assert_eq!(consumed, 6);
        assert_eq!(palette.banks, [[0xffff; 16]; 8]);
        assert_eq!(palette.source_bytes(), [0xff; OUTPUT_BYTES]);
        palette.validate().unwrap();
        palette.banks[7][15] = 0x7fff;
        assert!(palette.validate().is_err());
        palette.banks[7][15] = 0xffff;
        palette.encoded_source.push(0);
        assert!(palette.validate().is_err());
        assert!(Palette::decode(&stream(255, &[1, 252])).is_err());
    }

    #[test]
    fn typed_catalog_selection_preserves_unavailable_keys_and_checks_all_links() {
        let palette = Palette::decode(&stream(256, &[1, 253])).unwrap().0;
        let zero = Palette::decode(&stream(256, &[2, 253])).unwrap().0;
        let mut palettes = Palettes {
            descriptor_keys: (0..BASE_KEYS as u8).chain([255; 5]).collect(),
            relative_sources: std::array::from_fn(|_| {
                (0..BASE_KEYS).map(|i| (316 + i * 8) as u16).collect()
            }),
            catalogs: [vec![palette; BASE_KEYS], vec![zero; BASE_KEYS]],
        };
        palettes.validate().unwrap();
        for environment in 0..=255 {
            let selected = palettes.get(156, environment);
            match environment & 3 {
                1 => assert_eq!(selected.unwrap().banks[0][0], 0),
                2 => assert_eq!(selected.unwrap().banks[0][0], 0xffff),
                _ => assert!(selected.is_none()),
            }
            assert!(palettes.get(157, environment).is_none());
            assert!(palettes.get(usize::MAX, environment).is_none());
        }
        palettes.descriptor_keys[0] = 157;
        assert!(palettes.validate().is_err());
        palettes.descriptor_keys[0] = 1;
        assert!(palettes.validate().is_err());
        palettes.descriptor_keys[0] = 255;
        assert!(palettes.validate().is_err());
        palettes.descriptor_keys[0] = 0;
        palettes.relative_sources[0][0] = 314;
        palettes.relative_sources[0][1] = 315;
        assert!(palettes.validate().is_err());
        palettes.relative_sources[0][0] = 0;
        assert!(palettes.validate().is_err());
        palettes.catalogs[0].pop();
        assert!(palettes.validate().is_err());
    }
}
