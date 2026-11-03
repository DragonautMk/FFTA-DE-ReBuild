//! EUR equipment OBJ poses selected by the original item/action producer.
//! Live facing, animation commands, task allocation, DMA and VRAM remain guest work.
use super::{items, obj_lists, unit_obj_initial_pose as initial, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE_SHA256: &str = "feb6a277c8696b3c5e189c928cbb2cb6936cb2dbdb1a82b3300b1407557c54ae";
pub type SourceSpan = (u32, Vec<u8>, String);

/// Original 08022174 plus the caller's full-word part offset. No live index cap.
pub fn index_offset(root: u32, animation: u32, part: u32) -> u32 {
    root.wrapping_add((animation & !3).wrapping_mul(6))
        .wrapping_add(if matches!(animation & 3, 1 | 2) {
            12
        } else {
            0
        })
        .wrapping_add(part.wrapping_mul(4))
}

/// Static query projection, not a restriction on original live inputs.
/// Directions 0/3 and 1/2 name the same physical cells; retain one key per cell.
pub fn producer_keys(data: &items::Items) -> Result<Vec<(u16, u16, u8)>, String> {
    data.validate()?;
    let mut keys = BTreeSet::new();
    for record in data.records.iter().skip(1) {
        if (1..=19).contains(&record.raw_property(3)) {
            let sprite = record.raw_property(9) as u16;
            for base in [0, 4, 8] {
                for direction in [0, 1] {
                    for part in [0, 1] {
                        keys.insert((sprite, base + direction, part));
                    }
                }
            }
        }
    }
    // Action12/13 replace the item sprite with185; action16/17 use126,
    // except original item45 selects124. These do not depend on item style.
    for (sprite, base) in [(185, 0), (185, 4), (124, 16), (126, 16)] {
        for direction in [0, 1] {
            for part in [0, 1] {
                keys.insert((sprite, base + direction, part));
            }
        }
    }
    Ok(keys.into_iter().collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    pub sprite_id: u16,
    pub source_offset: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub sprite_id: u16,
    pub animation: u16,
    pub part: u8,
    pub index_source: u32,
    /// A real null cell makes 08022788 return -1 before reading a sequence.
    pub sequence_source: Option<u32>,
}
impl Binding {
    pub fn key(&self) -> (u16, u16, u8) {
        (self.sprite_id, self.animation, self.part)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    /// These are pixel/OBJ relative offsets only for operation1.
    pub word00: u32,
    pub word04: u32,
    pub duration: u8,
    pub operation: u8,
    pub bytes_0a_13: [u8; 10],
}
impl Frame {
    pub fn bytes(&self) -> [u8; 20] {
        let mut out = [0; 20];
        out[..4].copy_from_slice(&self.word00.to_le_bytes());
        out[4..8].copy_from_slice(&self.word04.to_le_bytes());
        out[8] = self.duration;
        out[9] = self.operation;
        out[10..].copy_from_slice(&self.bytes_0a_13);
        out
    }
    pub fn from_bytes(bytes: &[u8; 20]) -> Self {
        Self {
            word00: u32::from_le_bytes(bytes[..4].try_into().unwrap()),
            word04: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjEquipment {
    pub roots: Vec<Root>,
    pub bindings: Vec<Binding>,
    pub sequences: Vec<Sequence>,
    pub objects: Vec<obj_lists::List>,
    pub pixels: Vec<initial::PixelBlock>,
}
fn extent(at: u32, len: usize, alignment: u32) -> Result<(), String> {
    if at % alignment != 0
        || len == 0
        || len > 0x1000000
        || at.checked_add(len as u32).is_none_or(|end| end > 0x1000000)
    {
        return Err("Invalid equipment OBJ source extent/alignment".into());
    }
    Ok(())
}
impl UnitObjEquipment {
    pub fn validate(&self) -> Result<(), String> {
        if self.roots.len() != 62
            || self.bindings.len() != 748
            || self.sequences.len() != 258
            || self.objects.len() != 12
            || self.pixels.len() != 280
        {
            return Err("Equipment OBJ projection shape mismatch".into());
        }
        if !self
            .roots
            .windows(2)
            .all(|w| w[0].sprite_id < w[1].sprite_id)
            || !self.bindings.windows(2).all(|w| w[0].key() < w[1].key())
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
            return Err("Unordered/duplicate equipment OBJ identity".into());
        }
        let roots: BTreeMap<_, _> = self
            .roots
            .iter()
            .map(|r| (r.sprite_id, r.source_offset))
            .collect();
        let mut referenced = BTreeSet::new();
        for root in &self.roots {
            extent(initial::DIRECTORY + u32::from(root.sprite_id) * 4, 4, 4)?;
            extent(root.source_offset, 4, 4)?;
            if !self.bindings.iter().any(|b| b.sprite_id == root.sprite_id) {
                return Err("Unreferenced equipment OBJ root".into());
            }
        }
        for b in &self.bindings {
            let root = *roots
                .get(&b.sprite_id)
                .ok_or("Missing equipment OBJ root")?;
            if b.part > 1
                || b.animation & 3 > 1
                || b.index_source != index_offset(root, u32::from(b.animation), u32::from(b.part))
            {
                return Err("Invalid equipment OBJ query/cell link".into());
            }
            extent(b.index_source, 4, 4)?;
            if let Some(source) = b.sequence_source {
                if self
                    .sequences
                    .binary_search_by_key(&source, |s| s.source_offset)
                    .is_err()
                {
                    return Err("Missing equipment OBJ sequence".into());
                }
                referenced.insert(source);
            }
        }
        if referenced.len() != self.sequences.len() {
            return Err("Unreferenced equipment OBJ sequence".into());
        }
        for list in &self.objects {
            list.validate()?;
            extent(list.source_offset, list.source_len(), 2)?;
        }
        for block in &self.pixels {
            extent(block.source_offset, block.tiles.len() * 32, 4)?;
            block.bytes()?;
        }
        for seq in &self.sequences {
            if seq.frames.is_empty() || seq.frames.len() >= 32768 {
                return Err("Equipment OBJ sequence outside positive-count projection".into());
            }
            extent(seq.source_offset, 4 + seq.frames.len() * 20, 4)?;
            for frame in &seq.frames {
                if frame.operation == 1 {
                    let at = initial::OBJECT_BASE
                        .checked_add(frame.word04)
                        .ok_or("Equipment OBJ object overflow")?;
                    let list = self
                        .objects
                        .iter()
                        .find(|o| o.source_offset == at)
                        .ok_or("Missing equipment OBJ object link")?;
                    let source = initial::PIXEL_BASE
                        .checked_add(frame.word00)
                        .ok_or("Equipment OBJ pixel overflow")?;
                    let len = initial::transfer_bytes(initial::transfer_tiles(list)) as usize;
                    if !self
                        .pixels
                        .iter()
                        .any(|p| p.source_offset == source && p.tiles.len() * 32 == len)
                    {
                        return Err("Missing equipment OBJ pixel transfer".into());
                    }
                }
                // Other commands retain all words/tail bytes. They do not
                // create fabricated pixel or object source references.
            }
        }
        Ok(())
    }
    pub fn validate_producers(&self, data: &items::Items) -> Result<(), String> {
        self.validate()?;
        if self.bindings.iter().map(Binding::key).collect::<Vec<_>>() != producer_keys(data)? {
            return Err("Equipment OBJ item/action producer mismatch".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans = Vec::new();
        for root in &self.roots {
            spans.push((
                initial::DIRECTORY + u32::from(root.sprite_id) * 4,
                (0x08000000 + root.source_offset).to_le_bytes().to_vec(),
                "directory".into(),
            ));
        }
        for b in &self.bindings {
            spans.push((
                b.index_source,
                b.sequence_source
                    .map_or(0, |at| 0x08000000 + at)
                    .to_le_bytes()
                    .to_vec(),
                "index".into(),
            ));
        }
        for seq in &self.sequences {
            spans.push((seq.source_offset, seq.bytes(), "sequence".into()));
        }
        for list in &self.objects {
            spans.push((list.source_offset, list.source_bytes()?, "objects".into()));
        }
        for block in &self.pixels {
            spans.push((block.source_offset, block.bytes()?, "pixels".into()));
        }
        spans.sort_by_key(|s| s.0);
        let mut out: Vec<SourceSpan> = Vec::new();
        for (at, bytes, role) in spans {
            if let Some((last_at, last_bytes, last_role)) = out.last_mut() {
                let end = *last_at + last_bytes.len() as u32;
                if end > at {
                    return Err("Overlapping equipment OBJ physical sources".into());
                }
                if end == at && *last_role == role {
                    last_bytes.extend(bytes);
                    continue;
                }
            }
            out.push((at, bytes, role));
        }
        Ok(out)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Equipment OBJ differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn bytes(rom: &[u8], at: u32, n: usize) -> Result<&[u8], String> {
    extent(at, n, 1)?;
    rom.get(at as usize..at as usize + n)
        .ok_or_else(|| "Truncated equipment OBJ source".into())
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
        .ok_or_else(|| "Equipment OBJ pointer outside ROM".into())
}
pub struct Catalog {
    pub decoded: UnitObjEquipment,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Equipment OBJ requires exact EUR ROM; USA pending".into());
        }
        for (at, expected) in [
            (0x2219c, initial::DIRECTORY),
            (0x221b8, initial::PIXEL_BASE),
            (0x221c0, initial::OBJECT_BASE),
            (0xce20c, items::SOURCE_OFFSET),
        ] {
            if pointer(rom, at)? != expected {
                return Err("Equipment OBJ original root mismatch".into());
            }
        }
        for (at, expected) in [
            (0x9bdd6, 0x2103),
            (0x9bde0, 0x2109),
            (0xce284, 0x89e0),
            (0x9bf7e, 0x20b9),
            (0x9bfb0, 0x207e),
            (0x9bfb4, 0x2f2d),
            (0x9bfb8, 0x217c),
            (0x9c076, 0x9502),
            (0x9c0aa, 0x3501),
        ] {
            if half(rom, at)? != expected {
                return Err("Equipment OBJ original producer mismatch".into());
            }
        }
        let items = items::Catalog::from_rom(rom)?.decoded;
        let mut roots = BTreeMap::new();
        let mut bindings = Vec::new();
        let mut sequences = BTreeMap::new();
        let mut objects = BTreeMap::new();
        let mut pixels = BTreeMap::new();
        for (sprite_id, animation, part) in producer_keys(&items)? {
            let root = pointer(rom, initial::DIRECTORY + u32::from(sprite_id) * 4)?;
            roots.insert(
                sprite_id,
                Root {
                    sprite_id,
                    source_offset: root,
                },
            );
            let index_source = index_offset(root, u32::from(animation), u32::from(part));
            let sequence_source = if word(rom, index_source)? == 0 {
                None
            } else {
                Some(pointer(rom, index_source)?)
            };
            bindings.push(Binding {
                sprite_id,
                animation,
                part,
                index_source,
                sequence_source,
            });
            let Some(at) = sequence_source else {
                continue;
            };
            if sequences.contains_key(&at) {
                continue;
            }
            let count = usize::from(half(rom, at)?);
            let frames: Vec<_> = bytes(rom, at + 4, count * 20)?
                .chunks_exact(20)
                .map(|b| Frame::from_bytes(b.try_into().unwrap()))
                .collect();
            for frame in &frames {
                if frame.operation != 1 {
                    continue;
                }
                let object_at = initial::OBJECT_BASE
                    .checked_add(frame.word04)
                    .ok_or("Equipment OBJ object overflow")?;
                let len = 2 + usize::from(half(rom, object_at)?) * 6;
                let list = obj_lists::List::decode(object_at, bytes(rom, object_at, len)?)?;
                let pixel_at = initial::PIXEL_BASE
                    .checked_add(frame.word00)
                    .ok_or("Equipment OBJ pixel overflow")?;
                let len = initial::transfer_bytes(initial::transfer_tiles(&list)) as usize;
                let block = initial::PixelBlock {
                    source_offset: pixel_at,
                    tiles: bytes(rom, pixel_at, len)?
                        .chunks_exact(32)
                        .map(|b| initial::Tile::decode(b.try_into().unwrap()))
                        .collect(),
                };
                if let Some(old) = pixels.insert(pixel_at, block.clone()) {
                    if old != block {
                        return Err("Conflicting equipment OBJ pixel extents".into());
                    }
                }
                objects.insert(object_at, list);
            }
            sequences.insert(
                at,
                Sequence {
                    source_offset: at,
                    opaque_header: half(rom, at + 2)?,
                    frames,
                },
            );
        }
        let decoded = UnitObjEquipment {
            roots: roots.into_values().collect(),
            bindings,
            sequences: sequences.into_values().collect(),
            objects: objects.into_values().collect(),
            pixels: pixels.into_values().collect(),
        };
        decoded.validate_producers(&items)?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, data, role) in decoded.source_spans()? {
            if bytes(rom, offset, data.len())? != data {
                return Err("Equipment OBJ reconstruction mismatch".into());
            }
            combined.extend_from_slice(&data);
            sources.push(SourceRange {
                offset,
                length: data.len() as u32,
                sha256: ffta_rom::digest(&data),
                role,
            });
        }
        if sources.len() != 593
            || combined.len() != 110666
            || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("Equipment OBJ source plan/hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
