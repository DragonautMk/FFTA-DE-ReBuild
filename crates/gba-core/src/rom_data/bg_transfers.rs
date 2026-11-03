//! Descriptor-linked EUR tile animation controls and shared raw word banks.
//! Offline model only; mutable frame/tick/DMA progress stays in guest RAM.
use super::{bg_descriptors, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SLOTS: usize = 5;
pub const MAX_FRAMES: usize = 255;
pub const MAX_TILE_BYTES: usize = 4 * 1024 * 1024;
const VRAM_START: u32 = 0x0600_0000;
const VRAM_END: u32 = 0x0601_8000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    /// Halfword countdown; zero retains the original 65536-tick wrap behavior.
    pub duration_ticks: u16,
    pub source_tile: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub relative_source: u32,
    pub transfer_bytes: u16,
    /// Original update reads only the low byte of the frame-count halfword.
    pub header_byte3: u8,
    pub frames: Vec<Frame>,
}

impl Control {
    pub fn decode(input: &[u8], relative_source: u32) -> Result<(Self, usize), String> {
        let header = input.get(..4).ok_or("Truncated BG transfer control")?;
        let count = header[2] as usize;
        if count == 0 {
            return Err("Empty BG transfer animation".into());
        }
        let consumed = 4 + count * 4;
        let body = input
            .get(4..consumed)
            .ok_or("Truncated BG transfer frames")?;
        let result = Self {
            relative_source,
            transfer_bytes: u16::from_le_bytes(header[..2].try_into().unwrap()),
            header_byte3: header[3],
            frames: body
                .chunks_exact(4)
                .map(|bytes| Frame {
                    duration_ticks: u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                    source_tile: u16::from_le_bytes(bytes[2..].try_into().unwrap()),
                })
                .collect(),
        };
        result.validate()?;
        Ok((result, consumed))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.transfer_bytes == 0
            || self.transfer_bytes % 32 != 0
            || self.frames.is_empty()
            || self.frames.len() > MAX_FRAMES
        {
            return Err("Invalid BG transfer size/frame count".into());
        }
        self.referenced_bytes()?;
        Ok(())
    }

    pub fn referenced_bytes(&self) -> Result<usize, String> {
        let extent = self
            .frames
            .iter()
            .map(|frame| frame.source_tile as usize * 32 + self.transfer_bytes as usize)
            .max()
            .ok_or("Missing BG transfer frame")?;
        if extent > MAX_TILE_BYTES {
            return Err("BG transfer tile extent exceeds input guard".into());
        }
        Ok(extent)
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = self.transfer_bytes.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[self.frames.len() as u8, self.header_byte3]);
        for frame in &self.frames {
            bytes.extend_from_slice(&frame.duration_ticks.to_le_bytes());
            bytes.extend_from_slice(&frame.source_tile.to_le_bytes());
        }
        Ok(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tiles {
    pub relative_source: u32,
    pub words: Vec<u32>,
}
impl Tiles {
    pub fn source_bytes(&self) -> Vec<u8> {
        self.words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub control: u16,
    pub tiles: u16,
    pub destination: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transfers {
    /// Slot0 is the base triple; slots1..4 are the four extra triples.
    /// IDs for controls/tiles independently follow first descriptor occurrence.
    pub descriptor_links: Vec<[Option<Link>; SLOTS]>,
    /// All absent in the declared EUR catalog. A future nonzero program needs
    /// its own contract; it must never be silently discarded as a boolean.
    pub variant_sources: Vec<Option<u32>>,
    pub controls: Vec<Control>,
    pub tiles: Vec<Tiles>,
}

impl Transfers {
    pub fn validate(&self) -> Result<(), String> {
        let count = self.descriptor_links.len();
        if count == 0
            || count > bg_descriptors::MAX_RECORDS
            || self.variant_sources.len() != count
            || self.variant_sources.iter().any(Option::is_some)
            || self.controls.is_empty()
            || self.controls.len() > count * SLOTS
            || self.tiles.is_empty()
            || self.tiles.len() > count * SLOTS
        {
            return Err("Invalid BG transfer catalog count or unsupported variant program".into());
        }
        let mut seen_controls = BTreeSet::new();
        let mut seen_tiles = BTreeSet::new();
        let mut tile_lengths = vec![0; self.tiles.len()];
        for link in self.descriptor_links.iter().flatten().flatten() {
            let control = self
                .controls
                .get(link.control as usize)
                .ok_or("Missing transfer control ID")?;
            if link.tiles as usize >= self.tiles.len()
                || (seen_controls.insert(link.control)
                    && link.control as usize != seen_controls.len() - 1)
                || (seen_tiles.insert(link.tiles) && link.tiles as usize != seen_tiles.len() - 1)
            {
                return Err("Invalid BG transfer canonical resource IDs".into());
            }
            control.validate()?;
            if link.destination < VRAM_START
                || link.destination % 32 != 0
                || link
                    .destination
                    .checked_add(control.transfer_bytes as u32)
                    .is_none_or(|end| end > VRAM_END)
            {
                return Err("BG transfer destination outside VRAM".into());
            }
            tile_lengths[link.tiles as usize] =
                tile_lengths[link.tiles as usize].max(control.referenced_bytes()?);
        }
        if seen_controls.len() != self.controls.len() || seen_tiles.len() != self.tiles.len() {
            return Err("Unreferenced BG transfer resource".into());
        }
        let mut ranges = Vec::new();
        let table_bytes = count * bg_descriptors::RECORD_BYTES;
        let mut range = |start: u32, length: usize| -> Result<(), String> {
            let end = start
                .checked_add(u32::try_from(length).map_err(|_| "Transfer source length overflow")?)
                .ok_or("BG transfer source overflow")?;
            if start < table_bytes as u32 || end > 0x0200_0000 {
                return Err("BG transfer source outside payload bounds".into());
            }
            ranges.push((start, end));
            Ok(())
        };
        for control in &self.controls {
            range(control.relative_source, control.source_bytes()?.len())?;
        }
        for (tiles, length) in self.tiles.iter().zip(tile_lengths) {
            if tiles.words.len() > MAX_TILE_BYTES / 4 || tiles.words.len() * 4 != length {
                return Err("BG transfer shared tile extent mismatch".into());
            }
            range(tiles.relative_source, length)?;
        }
        ranges.sort_unstable();
        if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err("Overlapping or duplicate BG transfer resources".into());
        }
        Ok(())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("BG transfer programs differ from the authenticated ROM".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Transfers,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("BG transfers currently require the exact EUR ROM; USA unverified".into());
        }
        let descriptors = bg_descriptors::Catalog::from_rom(rom)?;
        let root = descriptors.source.offset;
        let mut decoded = Transfers {
            descriptor_links: Vec::new(),
            variant_sources: Vec::new(),
            controls: Vec::new(),
            tiles: Vec::new(),
        };
        let mut control_ids = BTreeMap::new();
        let mut tile_ids = BTreeMap::new();
        let mut lengths: Vec<usize> = Vec::new();
        for descriptor in &descriptors.decoded.entries {
            if descriptor.variant_source != 0 {
                return Err("Unsupported nonzero EUR variant program".into());
            }
            decoded.variant_sources.push(None);
            let mut links: [Option<Link>; SLOTS] = std::array::from_fn(|_| None);
            for (slot, transfer) in std::iter::once(&descriptor.transfer)
                .chain(&descriptor.extra_transfers)
                .enumerate()
            {
                if transfer.control_source == 0
                    && transfer.data_source == 0
                    && transfer.destination == 0
                {
                    continue;
                }
                let control = if let Some(&id) = control_ids.get(&transfer.control_source) {
                    id
                } else {
                    let offset = root
                        .checked_add(transfer.control_source)
                        .ok_or("Transfer source overflow")?;
                    let input = rom
                        .get(offset as usize..)
                        .ok_or("Transfer control outside ROM")?;
                    let (control, _) = Control::decode(input, transfer.control_source)?;
                    let id = decoded.controls.len() as u16;
                    decoded.controls.push(control);
                    control_ids.insert(transfer.control_source, id);
                    id
                };
                let tiles = if let Some(&id) = tile_ids.get(&transfer.data_source) {
                    id
                } else {
                    let id = decoded.tiles.len() as u16;
                    decoded.tiles.push(Tiles {
                        relative_source: transfer.data_source,
                        words: Vec::new(),
                    });
                    lengths.push(0);
                    tile_ids.insert(transfer.data_source, id);
                    id
                };
                lengths[tiles as usize] = lengths[tiles as usize]
                    .max(decoded.controls[control as usize].referenced_bytes()?);
                links[slot] = Some(Link {
                    control,
                    tiles,
                    destination: transfer.destination,
                });
            }
            decoded.descriptor_links.push(links);
        }
        for (tiles, length) in decoded.tiles.iter_mut().zip(lengths) {
            let offset = root
                .checked_add(tiles.relative_source)
                .ok_or("Transfer tiles source overflow")? as usize;
            let input = rom
                .get(
                    offset
                        ..offset
                            .checked_add(length)
                            .ok_or("Transfer tiles extent overflow")?,
                )
                .ok_or("Truncated BG transfer tile bank")?;
            tiles.words = input
                .chunks_exact(4)
                .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
                .collect();
        }
        decoded.validate()?;
        let mut sources = Vec::new();
        for (kind, rows) in [
            (
                "control",
                decoded
                    .controls
                    .iter()
                    .map(|c| Ok((c.relative_source, c.source_bytes()?)))
                    .collect::<Result<Vec<_>, String>>()?,
            ),
            (
                "tiles",
                decoded
                    .tiles
                    .iter()
                    .map(|t| (t.relative_source, t.source_bytes()))
                    .collect(),
            ),
        ] {
            for (id, (relative, bytes)) in rows.into_iter().enumerate() {
                sources.push(SourceRange {
                    offset: root
                        .checked_add(relative)
                        .ok_or("Transfer source overflow")?,
                    length: bytes.len() as u32,
                    sha256: ffta_rom::digest(&bytes),
                    role: format!("bg-transfer-{kind}/{id}"),
                });
            }
        }
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn control() -> Control {
        Control {
            relative_source: 0x1000,
            transfer_bytes: 32,
            header_byte3: 0x81,
            frames: vec![
                Frame {
                    duration_ticks: 0,
                    source_tile: 1,
                },
                Frame {
                    duration_ticks: 16,
                    source_tile: 0,
                },
            ],
        }
    }
    fn fixture() -> Transfers {
        Transfers {
            descriptor_links: vec![
                [
                    Some(Link {
                        control: 0,
                        tiles: 0,
                        destination: VRAM_START + 32
                    }),
                    None,
                    None,
                    None,
                    None
                ];
                2
            ],
            variant_sources: vec![None; 2],
            controls: vec![control()],
            tiles: vec![Tiles {
                relative_source: 0x2000,
                words: vec![0x81234567; 16],
            }],
        }
    }
    #[test]
    fn control_roundtrip_preserves_zero_ticks_high_header_and_exact_extent() {
        let value = control();
        let encoded = value.source_bytes().unwrap();
        let mut padded = encoded.clone();
        padded.extend_from_slice(&[0xcd; 4]);
        let (decoded, consumed) = Control::decode(&padded, value.relative_source).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(consumed, encoded.len());
        assert_eq!(value.referenced_bytes().unwrap(), 64);
        for end in 0..consumed {
            assert!(Control::decode(&padded[..end], 0x1000).is_err());
        }
        for bytes in [
            vec![32, 0, 0, 0],
            vec![31, 0, 1, 0, 1, 0, 0, 0],
            vec![0, 0, 1, 0, 1, 0, 0, 0],
        ] {
            assert!(Control::decode(&bytes, 0x1000).is_err());
        }
    }
    #[test]
    fn shared_banks_require_exact_extents_ids_destinations_and_no_overlap() {
        let valid = fixture();
        valid.validate().unwrap();
        for case in 0..8 {
            let mut bad = valid.clone();
            match case {
                0 => bad.descriptor_links[0][0].as_mut().unwrap().control = 1,
                1 => bad.descriptor_links[0][0].as_mut().unwrap().tiles = 1,
                2 => bad.descriptor_links[0][0].as_mut().unwrap().destination = VRAM_END,
                3 => bad.tiles[0].words.pop().map(|_| ()).unwrap(),
                4 => bad.tiles[0].relative_source = 0x1004,
                5 => bad.variant_sources[0] = Some(0x3000),
                6 => bad.controls.push(control()),
                _ => bad.descriptor_links[0][0].as_mut().unwrap().destination += 1,
            }
            assert!(bad.validate().is_err(), "case{case}");
        }
    }
    #[test]
    fn separate_control_aliases_can_share_one_raw_word_bank() {
        let mut value = fixture();
        let mut second = control();
        second.relative_source = 0x1100;
        second.transfer_bytes = 64;
        second.frames[0].source_tile = 0;
        value.controls.push(second);
        value.descriptor_links[0][1] = Some(Link {
            control: 1,
            tiles: 0,
            destination: VRAM_START + 64,
        });
        value.validate().unwrap();
        assert_eq!(
            value.tiles[0].source_bytes(),
            0x81234567u32.to_le_bytes().repeat(16)
        );
    }
}
