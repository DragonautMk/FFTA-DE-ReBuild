//! EUR unit initial-pose projection, rooted in existing job/special-unit fields.
//! Original animation scheduling, RAM tasks, DMA and VRAM remain guest state.
use super::{metadata14, metadata52, obj_lists, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DIRECTORY: u32 = 0x394834;
pub const PIXEL_BASE: u32 = 0x898a40;
pub const OBJECT_BASE: u32 = 0xa50020;
pub const SOURCE_SHA256: &str = "397b721c18a26cd080f0835e056f9af411b2c3ab34cfef8bc47291ca17dd490b";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub sprite_id: u16,
    pub index_source: u32,
    pub sequence_source: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub pixel_offset: u32,
    pub object_offset: u32,
    pub duration: u8,
    pub operation: u8,
    pub bytes_0a_13: [u8; 10],
}
impl Frame {
    pub fn bytes(&self) -> [u8; 20] {
        let mut out = [0; 20];
        out[..4].copy_from_slice(&self.pixel_offset.to_le_bytes());
        out[4..8].copy_from_slice(&self.object_offset.to_le_bytes());
        out[8] = self.duration;
        out[9] = self.operation;
        out[10..].copy_from_slice(&self.bytes_0a_13);
        out
    }
    pub fn from_bytes(bytes: &[u8; 20]) -> Self {
        Self {
            pixel_offset: u32::from_le_bytes(bytes[..4].try_into().unwrap()),
            object_offset: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            duration: bytes[8],
            operation: bytes[9],
            bytes_0a_13: bytes[10..].try_into().unwrap(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub source_offset: u32,
    pub opaque_header: u16,
    pub frames: Vec<Frame>,
}
impl Sequence {
    pub fn bytes(&self) -> Vec<u8> {
        (self.frames.len() as u16)
            .to_le_bytes()
            .into_iter()
            .chain(self.opaque_header.to_le_bytes())
            .chain(self.frames.iter().flat_map(Frame::bytes))
            .collect()
    }
}

/// An 8x8 tile of palette indices, with the low nibble at the left pixel.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tile {
    pub indices: [[u8; 8]; 8],
}
impl Tile {
    pub fn decode(bytes: &[u8; 32]) -> Self {
        Self {
            indices: std::array::from_fn(|y| {
                std::array::from_fn(|x| (bytes[y * 4 + x / 2] >> (4 * (x % 2))) & 15)
            }),
        }
    }
    pub fn bytes(&self) -> Result<[u8; 32], String> {
        if self.indices.iter().flatten().any(|&i| i > 15) {
            return Err("Unit OBJ tile index exceeds four bits".into());
        }
        Ok(std::array::from_fn(|at| {
            let y = at / 4;
            let x = at % 4 * 2;
            self.indices[y][x] | (self.indices[y][x + 1] << 4)
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PixelBlock {
    pub source_offset: u32,
    pub tiles: Vec<Tile>,
}
impl PixelBlock {
    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        self.tiles.iter().try_fold(Vec::new(), |mut bytes, tile| {
            bytes.extend_from_slice(&tile.bytes()?);
            Ok(bytes)
        })
    }
}

/// Exactly 080221D4's shape/size branches, including reserved shape3.
/// This is a tile count for the original transfer, not a rendering fallback.
pub fn transfer_tiles(list: &obj_lists::List) -> u32 {
    list.objects
        .iter()
        .map(|object| {
            let counts = if object.shape == 0 {
                [1, 4, 16, 64]
            } else {
                [2, 4, 8, 32]
            };
            counts[usize::from(object.size)]
        })
        .sum()
}
pub fn transfer_bytes(tiles: u32) -> u32 {
    // STRH caches the count; LSLS19/LSRS16 supplies DMA's word count.
    ((tiles as u16 as u32).wrapping_shl(19) >> 16) * 4
}

/// The known producer projection deliberately omits unresolved job0/1's FFFF.
/// It does not reject/normalize any value used by the original live code.
pub fn producer_ids(
    jobs: &metadata52::Metadata52,
    special: &metadata14::Metadata14,
) -> Result<Vec<u16>, String> {
    jobs.validate()?;
    special.validate_metadata52(jobs)?;
    let mut ids = BTreeSet::new();
    for job in &jobs.records {
        if job.routing == 0 {
            let id = job.packed_property(4).unwrap() as u16;
            if id != u16::MAX {
                ids.insert(id);
            }
        }
    }
    ids.extend(special.records.iter().map(|r| r.halfwords04_06[0]));
    Ok(ids.into_iter().collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjInitialPose {
    pub bindings: Vec<Binding>,
    pub sequences: Vec<Sequence>,
    pub objects: Vec<obj_lists::List>,
    pub pixels: Vec<PixelBlock>,
}
fn extent(offset: u32, length: usize, alignment: u32) -> Result<(), String> {
    if offset % alignment != 0
        || length == 0
        || length > 0x1000000
        || offset
            .checked_add(length as u32)
            .is_none_or(|end| end > 0x1000000)
    {
        return Err("Invalid unit OBJ source alignment/extent".into());
    }
    Ok(())
}
impl UnitObjInitialPose {
    pub fn validate(&self) -> Result<(), String> {
        // Fixed authenticated projection, not a guest sprite/action cap.
        if self.bindings.len() != 121
            || self.sequences.len() != 121
            || self.objects.len() != 6
            || self.pixels.len() != 333
        {
            return Err("Unit OBJ initial-pose projection shape mismatch".into());
        }
        if !self
            .bindings
            .windows(2)
            .all(|w| w[0].sprite_id < w[1].sprite_id)
            || !self
                .sequences
                .windows(2)
                .all(|w| w[0].source_offset < w[1].source_offset)
            || !self
                .objects
                .windows(2)
                .all(|w| w[0].source_offset < w[1].source_offset)
            || !self
                .pixels
                .windows(2)
                .all(|w| w[0].source_offset < w[1].source_offset)
        {
            return Err("Unordered/duplicate unit OBJ identity".into());
        }
        for binding in &self.bindings {
            extent(DIRECTORY + u32::from(binding.sprite_id) * 4, 4, 4)?;
            extent(binding.index_source, 4, 4)?;
            if !self
                .sequences
                .iter()
                .any(|s| s.source_offset == binding.sequence_source)
            {
                return Err("Missing unit OBJ sequence link".into());
            }
        }
        for list in &self.objects {
            list.validate()?;
            extent(list.source_offset, list.source_len(), 2)?;
        }
        for block in &self.pixels {
            extent(block.source_offset, block.tiles.len() * 32, 4)?;
            block.bytes()?;
        }
        for sequence in &self.sequences {
            if sequence.frames.is_empty() || sequence.frames.len() >= 32768 {
                return Err("Unit OBJ sequence outside the positive-count contract".into());
            }
            extent(sequence.source_offset, 4 + 20 * sequence.frames.len(), 4)?;
            for frame in &sequence.frames {
                let at = OBJECT_BASE
                    .checked_add(frame.object_offset)
                    .ok_or("Unit OBJ object offset overflow")?;
                let list = self
                    .objects
                    .iter()
                    .find(|o| o.source_offset == at)
                    .ok_or("Missing unit OBJ object link")?;
                let source = PIXEL_BASE
                    .checked_add(frame.pixel_offset)
                    .ok_or("Unit OBJ pixel offset overflow")?;
                let length = transfer_bytes(transfer_tiles(list));
                if frame.operation != 1
                    || !self
                        .pixels
                        .iter()
                        .any(|p| p.source_offset == source && p.tiles.len() * 32 == length as usize)
                {
                    return Err("Missing unit OBJ transfer link/operation".into());
                }
            }
        }
        Ok(())
    }
    pub fn validate_producers(
        &self,
        jobs: &metadata52::Metadata52,
        special: &metadata14::Metadata14,
    ) -> Result<(), String> {
        self.validate()?;
        if self
            .bindings
            .iter()
            .map(|b| b.sprite_id)
            .collect::<Vec<_>>()
            != producer_ids(jobs, special)?
        {
            return Err("Unit OBJ producer projection mismatch".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans = Vec::new();
        for binding in &self.bindings {
            spans.push((
                DIRECTORY + u32::from(binding.sprite_id) * 4,
                (0x08000000 + binding.index_source).to_le_bytes().to_vec(),
                "directory".to_string(),
            ));
            spans.push((
                binding.index_source,
                (0x08000000 + binding.sequence_source)
                    .to_le_bytes()
                    .to_vec(),
                "initial-pose-index".to_string(),
            ));
        }
        for sequence in &self.sequences {
            spans.push((sequence.source_offset, sequence.bytes(), "sequence".into()));
        }
        for list in &self.objects {
            spans.push((
                list.source_offset,
                list.source_bytes()?,
                "object-list".into(),
            ));
        }
        for block in &self.pixels {
            spans.push((block.source_offset, block.bytes()?, "pixels-4bpp".into()));
        }
        spans.sort_by_key(|s| s.0);
        let mut out: Vec<SourceSpan> = Vec::new();
        for (at, bytes, role) in spans {
            if let Some((last_at, last_bytes, last_role)) = out.last_mut() {
                let end = *last_at + last_bytes.len() as u32;
                if end > at {
                    return Err("Overlapping unit OBJ physical owners".into());
                }
                if end == at && *last_role == role {
                    last_bytes.extend_from_slice(&bytes);
                    continue;
                }
            }
            out.push((at, bytes, role));
        }
        Ok(out)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Unit OBJ initial pose differs from authenticated source".into());
        }
        Ok(())
    }
}

fn bytes(rom: &[u8], at: u32, length: usize) -> Result<&[u8], String> {
    extent(at, length, 1)?;
    rom.get(at as usize..at as usize + length)
        .ok_or_else(|| "Truncated unit OBJ source".into())
}
fn half(rom: &[u8], at: u32) -> Result<u16, String> {
    Ok(u16::from_le_bytes(bytes(rom, at, 2)?.try_into().unwrap()))
}
fn word(rom: &[u8], at: u32) -> Result<u32, String> {
    Ok(u32::from_le_bytes(bytes(rom, at, 4)?.try_into().unwrap()))
}
fn pointer(rom: &[u8], at: u32) -> Result<u32, String> {
    word(rom, at)?
        .checked_sub(0x08000000)
        .filter(|&p| p < 0x1000000)
        .ok_or_else(|| "Unit OBJ pointer outside source ROM".into())
}
pub struct Catalog {
    pub decoded: UnitObjInitialPose,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Unit OBJ initial pose requires exact EUR ROM; USA pending".into());
        }
        for (at, value) in [
            (0x2219c, DIRECTORY),
            (0x221b8, PIXEL_BASE),
            (0x221c0, OBJECT_BASE),
        ] {
            if pointer(rom, at)? != value {
                return Err("Unit OBJ original root mismatch".into());
            }
        }
        let jobs = metadata52::Catalog::from_rom(rom)?.decoded;
        let special = metadata14::Catalog::from_rom(rom)?.decoded;
        let mut bindings = Vec::new();
        let mut sequences = BTreeMap::new();
        let mut objects = BTreeMap::new();
        let mut pixels = BTreeMap::new();
        for sprite_id in producer_ids(&jobs, &special)? {
            let index_source = pointer(rom, DIRECTORY + u32::from(sprite_id) * 4)?;
            let sequence_source = pointer(rom, index_source)?;
            bindings.push(Binding {
                sprite_id,
                index_source,
                sequence_source,
            });
            let count = usize::from(half(rom, sequence_source)?);
            let opaque_header = half(rom, sequence_source + 2)?;
            let frame_bytes = bytes(rom, sequence_source + 4, count * 20)?;
            let frames: Vec<_> = frame_bytes
                .chunks_exact(20)
                .map(|b| Frame::from_bytes(b.try_into().unwrap()))
                .collect();
            for frame in &frames {
                let at = OBJECT_BASE
                    .checked_add(frame.object_offset)
                    .ok_or("Unit OBJ object source overflow")?;
                let size = 2 + usize::from(half(rom, at)?) * 6;
                let list = obj_lists::List::decode(at, bytes(rom, at, size)?)?;
                let length = transfer_bytes(transfer_tiles(&list)) as usize;
                let pixel_at = PIXEL_BASE
                    .checked_add(frame.pixel_offset)
                    .ok_or("Unit OBJ pixel source overflow")?;
                let block = PixelBlock {
                    source_offset: pixel_at,
                    tiles: bytes(rom, pixel_at, length)?
                        .chunks_exact(32)
                        .map(|b| Tile::decode(b.try_into().unwrap()))
                        .collect(),
                };
                if let Some(prior) = pixels.insert(pixel_at, block.clone()) {
                    if prior != block {
                        return Err("Conflicting unit OBJ transfer extents".into());
                    }
                }
                objects.insert(at, list);
            }
            sequences.insert(
                sequence_source,
                Sequence {
                    source_offset: sequence_source,
                    opaque_header,
                    frames,
                },
            );
        }
        let decoded = UnitObjInitialPose {
            bindings,
            sequences: sequences.into_values().collect(),
            objects: objects.into_values().collect(),
            pixels: pixels.into_values().collect(),
        };
        decoded.validate_producers(&jobs, &special)?;
        let spans = decoded.source_spans()?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, data, role) in spans {
            if bytes(rom, offset, data.len())? != data {
                return Err("Unit OBJ source reconstruction mismatch".into());
            }
            combined.extend_from_slice(&data);
            sources.push(SourceRange {
                offset,
                length: data.len() as u32,
                sha256: ffta_rom::digest(&data),
                role,
            });
        }
        if sources.len() != 415
            || combined.len() != 146358
            || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("Unit OBJ initial-pose source contract mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
