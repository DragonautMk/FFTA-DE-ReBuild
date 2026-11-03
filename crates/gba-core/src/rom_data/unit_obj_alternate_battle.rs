//! Finite EUR alternate-mode actor battle poses and their original body/part selectors.
//! Live facing, animation commands, task allocation, DMA and VRAM remain guest work.
use super::{
    metadata14, metadata52, obj_lists, unit_obj_equipment as equipment,
    unit_obj_initial_pose as initial, unit_obj_normal_battle as normal, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE_SHA256: &str = "bdf15f7d79e3af013fae656e16341d5a01d216be364ed881b877850f116bd002";
pub type SourceSpan = (u32, Vec<u8>, String);

pub use equipment::{index_offset, Frame, Root, Sequence};
pub use normal::{BODY_BASES, EXTRA_BASES};

/// A known index-root target is a dependency view, never a sequence header.
/// Original guest code still reads the same pointer and has no new input cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Target {
    Null {},
    Sequence { source_offset: u32 },
    SharedIndexRoot { source_offset: u32, sprite_id: u16 },
}
impl Target {
    pub fn source(self) -> Option<u32> {
        match self {
            Self::Null {} => None,
            Self::Sequence { source_offset } | Self::SharedIndexRoot { source_offset, .. } => {
                Some(source_offset)
            }
        }
    }
    pub fn bytes(self) -> [u8; 4] {
        self.source().map_or(0, |at| 0x08000000 + at).to_le_bytes()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub sprite_id: u16,
    pub animation: u16,
    pub part: u8,
    pub index_source: u32,
    pub target: Target,
}
impl Binding {
    pub fn key(&self) -> (u16, u16, u8) {
        (self.sprite_id, self.animation, self.part)
    }
}

/// Alternate metadata52 property5/metadata14 halfword06 plus forced sprite230.
/// This is a finite offline query projection, not a reachability assertion.
pub fn producer_keys(
    jobs: &metadata52::Metadata52,
    special: &metadata14::Metadata14,
) -> Result<Vec<(u16, u16, u8)>, String> {
    jobs.validate()?;
    special.validate_metadata52(jobs)?;
    let mut sprites = BTreeSet::from([230]);
    for job in &jobs.records {
        if job.routing == 0 {
            let id = job.packed_property(5).unwrap() as u16;
            if id != u16::MAX {
                sprites.insert(id);
            }
        }
    }
    sprites.extend(special.records.iter().map(|r| r.halfwords04_06[1]));
    let mut keys = BTreeSet::new();
    for sprite in sprites {
        for base in BODY_BASES {
            for direction in [0, 1] {
                keys.insert((sprite, base + direction, 0));
            }
        }
        for base in EXTRA_BASES {
            for direction in [0, 1] {
                for part in [1, 2] {
                    keys.insert((sprite, base + direction, part));
                }
            }
        }
    }
    Ok(keys.into_iter().collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjAlternateBattle {
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
        return Err("Invalid alternate battle OBJ source extent/alignment".into());
    }
    Ok(())
}
impl UnitObjAlternateBattle {
    pub fn validate(&self) -> Result<(), String> {
        if self.roots.len() != 65
            || self.bindings.len() != 5850
            || self.sequences.len() != 706
            || self.objects.len() != 30
            || self.pixels.len() != 1203
        {
            return Err("Alternate battle OBJ projection shape mismatch".into());
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
            return Err("Unordered/duplicate alternate battle OBJ identity".into());
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
                return Err("Unreferenced alternate battle OBJ root".into());
            }
        }
        for b in &self.bindings {
            let root = *roots
                .get(&b.sprite_id)
                .ok_or("Missing alternate battle OBJ root")?;
            if b.part > 2
                || b.animation & 3 > 1
                || b.index_source != index_offset(root, u32::from(b.animation), u32::from(b.part))
            {
                return Err("Invalid alternate battle OBJ query/cell link".into());
            }
            extent(b.index_source, 4, 4)?;
            match b.target {
                Target::Null {} => {}
                Target::Sequence { source_offset } => {
                    if self
                        .sequences
                        .binary_search_by_key(&source_offset, |s| s.source_offset)
                        .is_err()
                    {
                        return Err("Missing alternate battle OBJ sequence".into());
                    }
                    referenced.insert(source_offset);
                }
                Target::SharedIndexRoot {
                    source_offset,
                    sprite_id,
                } => {
                    extent(source_offset, 4, 4)?;
                    if b.index_source != initial::DIRECTORY + 4 * u32::from(sprite_id)
                        || self
                            .sequences
                            .binary_search_by_key(&source_offset, |s| s.source_offset)
                            .is_ok()
                    {
                        return Err("Invalid alternate battle shared index-root view".into());
                    }
                }
            }
        }
        if self
            .bindings
            .iter()
            .filter(|b| matches!(b.target, Target::SharedIndexRoot { .. }))
            .count()
            != 20
        {
            return Err("Alternate battle shared target projection mismatch".into());
        }
        if referenced.len() != self.sequences.len() {
            return Err("Unreferenced alternate battle OBJ sequence".into());
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
                return Err(
                    "Alternate battle OBJ sequence outside positive-count projection".into(),
                );
            }
            extent(seq.source_offset, 4 + seq.frames.len() * 20, 4)?;
            for frame in &seq.frames {
                if frame.operation == 1 {
                    let at = initial::OBJECT_BASE
                        .checked_add(frame.word04)
                        .ok_or("Alternate battle OBJ object overflow")?;
                    let list = self
                        .objects
                        .iter()
                        .find(|o| o.source_offset == at)
                        .ok_or("Missing alternate battle OBJ object link")?;
                    let source = initial::PIXEL_BASE
                        .checked_add(frame.word00)
                        .ok_or("Alternate battle OBJ pixel overflow")?;
                    let len = initial::transfer_bytes(initial::transfer_tiles(list)) as usize;
                    if !self
                        .pixels
                        .iter()
                        .any(|p| p.source_offset == source && p.tiles.len() * 32 == len)
                    {
                        return Err("Missing alternate battle OBJ pixel transfer".into());
                    }
                }
                // Other commands retain all words/tail bytes. They do not
                // create fabricated pixel or object source references.
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
        if self.bindings.iter().map(Binding::key).collect::<Vec<_>>()
            != producer_keys(jobs, special)?
        {
            return Err("Alternate battle OBJ producer projection mismatch".into());
        }
        Ok(())
    }
    /// The alias is established by independently authenticated directory/index
    /// fields, never by a sequence-count threshold or neighboring bank address.
    pub fn validate_initial_views(
        &self,
        initial_pose: &initial::UnitObjInitialPose,
    ) -> Result<(), String> {
        self.validate()?;
        initial_pose.validate()?;
        for b in &self.bindings {
            match b.target {
                Target::SharedIndexRoot {
                    source_offset,
                    sprite_id,
                } => {
                    let witness = initial_pose
                        .bindings
                        .iter()
                        .find(|w| w.sprite_id == sprite_id)
                        .ok_or("Missing alternate shared index-root witness")?;
                    if source_offset != witness.index_source
                        || b.index_source != initial::DIRECTORY + 4 * u32::from(witness.sprite_id)
                    {
                        return Err(
                            "Alternate shared target differs from initial-pose witness".into()
                        );
                    }
                }
                Target::Sequence { source_offset } => {
                    if initial_pose
                        .bindings
                        .iter()
                        .any(|w| w.index_source == source_offset)
                    {
                        return Err("Initial index root mislabeled as alternate sequence".into());
                    }
                }
                Target::Null {} => {}
            }
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
            spans.push((b.index_source, b.target.bytes().to_vec(), "index".into()));
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
                    // Logical index keys alias, and the witnessed sprite0
                    // directory entry is also sprite247's animation104 cell.
                    let start = (at - *last_at) as usize;
                    let witnessed_directory = last_role == "directory"
                        && role == "index"
                        && self.bindings.iter().any(|b| {
                            b.index_source == at
                                && matches!(b.target, Target::SharedIndexRoot { .. })
                        });
                    if end == at + bytes.len() as u32
                        && (*last_role == role || witnessed_directory)
                        && last_bytes[start..] == bytes
                    {
                        continue;
                    }
                    return Err("Conflicting alternate battle OBJ physical views".into());
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
    /// New ownership excludes byte-identical views of the three prior assets.
    /// Shared semantic data remains inspectable; the older runtime owners retain it.
    pub fn owned_spans(
        &self,
        initial_pose: &initial::UnitObjInitialPose,
        equipment_pose: &equipment::UnitObjEquipment,
        normal_pose: &normal::UnitObjNormalBattle,
    ) -> Result<Vec<SourceSpan>, String> {
        self.validate_initial_views(initial_pose)?;
        let mut previous = initial_pose.source_spans()?;
        previous.extend(equipment_pose.source_spans()?);
        previous.extend(normal_pose.owned_spans(initial_pose, equipment_pose)?);
        previous.sort_by_key(|s| s.0);
        if previous
            .windows(2)
            .any(|p| p[0].0 + p[0].1.len() as u32 > p[1].0)
        {
            return Err("Overlapping alternate battle OBJ dependencies".into());
        }
        let mut owned = Vec::new();
        for (at, bytes, role) in self.source_spans()? {
            let end = at + bytes.len() as u32;
            let mut cursor = at;
            for (old_at, old_bytes, _) in
                previous.iter().skip_while(|s| s.0 + s.1.len() as u32 <= at)
            {
                if *old_at >= end {
                    break;
                }
                let lo = cursor.max(*old_at);
                let hi = end.min(*old_at + old_bytes.len() as u32);
                if cursor < lo {
                    owned.push((
                        cursor,
                        bytes[(cursor - at) as usize..(lo - at) as usize].to_vec(),
                        role.clone(),
                    ));
                }
                if bytes[(lo - at) as usize..(hi - at) as usize]
                    != old_bytes[(lo - old_at) as usize..(hi - old_at) as usize]
                {
                    return Err("Alternate battle OBJ shared view differs from dependency".into());
                }
                cursor = hi;
            }
            if cursor < end {
                owned.push((cursor, bytes[(cursor - at) as usize..].to_vec(), role));
            }
        }
        Ok(owned)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Alternate battle OBJ differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn bytes(rom: &[u8], at: u32, n: usize) -> Result<&[u8], String> {
    extent(at, n, 1)?;
    rom.get(at as usize..at as usize + n)
        .ok_or_else(|| "Truncated alternate battle OBJ source".into())
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
        .ok_or_else(|| "Alternate battle OBJ pointer outside ROM".into())
}
pub struct Catalog {
    pub decoded: UnitObjAlternateBattle,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Alternate battle OBJ requires exact EUR ROM; USA pending".into());
        }
        for (at, expected) in [
            (0x2219c, initial::DIRECTORY),
            (0x221b8, initial::PIXEL_BASE),
            (0x221c0, initial::OBJECT_BASE),
        ] {
            if pointer(rom, at)? != expected {
                return Err("Alternate battle OBJ original root mismatch".into());
            }
        }
        // Whole selector/wrapper fingerprints bind the finite projection to
        // original control flow, including status substitutions and part loops.
        for (at, len, expected) in [
            (
                0x9acf8,
                0x124,
                "d111a66dee93686aee00a94af3e17a73695b6db48ebea126687119433f476d33",
            ),
            (
                0x9ba84,
                0x640,
                "e570f013d1d39fbaed53b134a1b80bd0b9d287003f5e057f48042b1d57879d5d",
            ),
            (
                0x9d9b0,
                0x6c,
                "0579525533340e6a212e40f89532d11cc5b860f688bc70f7563643f573e39f84",
            ),
        ] {
            if ffta_rom::digest(bytes(rom, at, len)?) != expected {
                return Err("Alternate battle OBJ original selector mismatch".into());
            }
        }
        let jobs = metadata52::Catalog::from_rom(rom)?.decoded;
        let special = metadata14::Catalog::from_rom(rom)?.decoded;
        let initial_pose = initial::Catalog::from_rom(rom)?.decoded;
        let known_index_roots: BTreeMap<_, _> = initial_pose
            .bindings
            .iter()
            .map(|b| (b.index_source, b.sprite_id))
            .collect();
        let mut roots = BTreeMap::new();
        let mut bindings = Vec::new();
        let mut sequences = BTreeMap::new();
        let mut objects = BTreeMap::new();
        let mut pixels = BTreeMap::new();
        for (sprite_id, animation, part) in producer_keys(&jobs, &special)? {
            let root = pointer(rom, initial::DIRECTORY + u32::from(sprite_id) * 4)?;
            roots.insert(
                sprite_id,
                Root {
                    sprite_id,
                    source_offset: root,
                },
            );
            let index_source = index_offset(root, u32::from(animation), u32::from(part));
            let target = if word(rom, index_source)? == 0 {
                Target::Null {}
            } else {
                let source_offset = pointer(rom, index_source)?;
                if let Some(&sprite_id) = known_index_roots.get(&source_offset) {
                    Target::SharedIndexRoot {
                        source_offset,
                        sprite_id,
                    }
                } else {
                    Target::Sequence { source_offset }
                }
            };
            bindings.push(Binding {
                sprite_id,
                animation,
                part,
                index_source,
                target,
            });
            let Target::Sequence { source_offset: at } = target else {
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
                    .ok_or("Alternate battle OBJ object overflow")?;
                let len = 2 + usize::from(half(rom, object_at)?) * 6;
                let list = obj_lists::List::decode(object_at, bytes(rom, object_at, len)?)?;
                let pixel_at = initial::PIXEL_BASE
                    .checked_add(frame.word00)
                    .ok_or("Alternate battle OBJ pixel overflow")?;
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
                        return Err("Conflicting alternate battle OBJ pixel extents".into());
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
        let decoded = UnitObjAlternateBattle {
            roots: roots.into_values().collect(),
            bindings,
            sequences: sequences.into_values().collect(),
            objects: objects.into_values().collect(),
            pixels: pixels.into_values().collect(),
        };
        decoded.validate_producers(&jobs, &special)?;
        let equipment_pose = equipment::Catalog::from_rom(rom)?.decoded;
        let normal_pose = normal::Catalog::from_rom(rom)?.decoded;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, data, role) in
            decoded.owned_spans(&initial_pose, &equipment_pose, &normal_pose)?
        {
            if bytes(rom, offset, data.len())? != data {
                return Err("Alternate battle OBJ reconstruction mismatch".into());
            }
            combined.extend_from_slice(&data);
            sources.push(SourceRange {
                offset,
                length: data.len() as u32,
                sha256: ffta_rom::digest(&data),
                role,
            });
        }
        if sources.len() != 3321
            || combined.len() != 410086
            || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("Alternate battle OBJ source plan/hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
