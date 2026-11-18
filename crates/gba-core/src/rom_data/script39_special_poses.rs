//! Conditional EUR opcode-39 low-selector special-unit pose sources.
//! Actor identity, current assignment, facing, task execution and DMA stay guest work.
use super::{
    metadata14, obj_lists,
    script39_registry_poses::{
        bytes, extent, half, pointer, word, IndexCell, IndexTarget, PriorSources, ProducerPair,
        SourceSpan,
    },
    script_archive, script_commands,
    unit_obj_equipment::{index_offset, Frame, Sequence},
    unit_obj_initial_pose::{self as initial, PixelBlock, Tile},
    Resource, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE_SHA256: &str = "b7e72f2b5611b16f5396212c7c06f76e6e3aed5de268681ef2ad3de82db73dc5";
const EXCLUDED_SPECIAL_BRANCH: [u32; 2] = [0xb96a1e, 0xba0841];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Script39SpecialPoses {
    pub producer_pairs: Vec<ProducerPair>,
    pub cells: Vec<IndexCell>,
    pub sequences: Vec<Sequence>,
    pub objects: Vec<obj_lists::List>,
    pub pixels: Vec<PixelBlock>,
}

/// Low selectors 2..94 use the special-unit lookup. The two original
/// group-9/selector-20..23 instructions take a different task branch.
pub fn producer_pairs(
    scripts: &script_archive::ScriptArchive,
) -> Result<Vec<ProducerPair>, String> {
    scripts.validate()?;
    let mut pairs: BTreeMap<(u8, u16), Vec<u32>> = BTreeMap::new();
    let mut excluded = Vec::new();
    let mut references = 0;
    for instruction in scripts.programs.iter().flat_map(|p| &p.instructions) {
        if instruction.command != 39 {
            continue;
        }
        let p = &instruction.parameters;
        if p.len() != 4 {
            return Err("Invalid opcode-39 parameter shape".into());
        }
        if p[0] >= 128 {
            continue;
        }
        if !(2..=94).contains(&p[0]) {
            return Err("Unsupported opcode-39 low selector".into());
        }
        let group = u16::from_le_bytes([p[1], p[2]]);
        if group == 9 && (20..=23).contains(&p[0]) {
            excluded.push(instruction.source_offset);
            continue;
        }
        references += 1;
        pairs
            .entry((p[0], group))
            .or_default()
            .push(instruction.source_offset);
    }
    excluded.sort_unstable();
    if references != 1857 || pairs.len() != 285 || excluded != EXCLUDED_SPECIAL_BRANCH {
        return Err("Opcode-39 special producer projection shape mismatch".into());
    }
    Ok(pairs
        .into_iter()
        .map(|((selector, group), mut script_sources)| {
            script_sources.sort_unstable();
            ProducerPair {
                selector,
                group,
                script_sources,
            }
        })
        .collect())
}
fn producer_keys(
    pairs: &[ProducerPair],
    special: &metadata14::Metadata14,
) -> Result<BTreeSet<(u16, u16, u8)>, String> {
    special.validate()?;
    let mut keys = BTreeSet::new();
    for pair in pairs {
        let row = special
            .records
            .iter()
            .find(|r| r.id == pair.selector)
            .ok_or("Missing opcode-39 special-unit row")?;
        for sprite in row.halfwords04_06 {
            if sprite == u16::MAX {
                return Err("Unresolved opcode-39 special sprite".into());
            }
            let base = pair.group.wrapping_mul(4);
            for direction in [0u16, 1] {
                for part in [0u8, 1, 2] {
                    keys.insert((sprite, base.wrapping_add(direction), part));
                }
            }
        }
    }
    Ok(keys)
}
impl Script39SpecialPoses {
    pub fn validate(&self) -> Result<(), String> {
        if self.producer_pairs.len() != 285
            || self.cells.len() != 964
            || self.sequences.len() != 109
            || self.objects.len() != 7
            || self.pixels.len() != 87
        {
            return Err("Invalid opcode-39 special pose projection shape".into());
        }
        if !self
            .producer_pairs
            .windows(2)
            .all(|w| (w[0].selector, w[0].group) < (w[1].selector, w[1].group))
            || !self
                .cells
                .windows(2)
                .all(|w| w[0].source_offset < w[1].source_offset)
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
            return Err("Unordered or duplicate opcode-39 special pose identity".into());
        }
        let mut references = BTreeSet::new();
        for pair in &self.producer_pairs {
            if !(2..=94).contains(&pair.selector)
                || (pair.group == 9 && (20..=23).contains(&pair.selector))
                || pair.script_sources.is_empty()
                || !pair.script_sources.windows(2).all(|w| w[0] < w[1])
            {
                return Err("Invalid opcode-39 special producer pair".into());
            }
            for &at in &pair.script_sources {
                if at < script_archive::ARCHIVE_OFFSET
                    || at > script_archive::ARCHIVE_END - 5
                    || !references.insert(at)
                {
                    return Err("Invalid opcode-39 special script witness".into());
                }
            }
        }
        if references.len() != 1857 {
            return Err("Opcode-39 special script witness count mismatch".into());
        }
        let mut targets = BTreeSet::new();
        for cell in &self.cells {
            extent(cell.source_offset, 4, 4)?;
            if let IndexTarget::Sequence { source_offset } = cell.target {
                extent(source_offset, 4, 4)?;
                targets.insert(source_offset);
            }
        }
        let mut pixel_targets = BTreeSet::new();
        let mut object_targets = BTreeSet::new();
        for seq in &self.sequences {
            if seq.frames.is_empty() || seq.frames.len() >= 32768 {
                return Err("Invalid opcode-39 special frame count".into());
            }
            extent(seq.source_offset, 4 + 20 * seq.frames.len(), 4)?;
            if !targets.contains(&seq.source_offset) {
                return Err("Unreferenced opcode-39 special sequence".into());
            }
            for frame in &seq.frames {
                if frame.operation == 1 {
                    pixel_targets.insert(
                        initial::PIXEL_BASE
                            .checked_add(frame.word00)
                            .ok_or("Opcode-39 special pixel target overflow")?,
                    );
                    object_targets.insert(
                        initial::OBJECT_BASE
                            .checked_add(frame.word04)
                            .ok_or("Opcode-39 special object target overflow")?,
                    );
                }
            }
        }
        for object in &self.objects {
            object.validate()?;
            extent(object.source_offset, object.source_len(), 2)?;
            if !object_targets.contains(&object.source_offset) {
                return Err("Unreferenced opcode-39 special OBJ list".into());
            }
        }
        for pixel in &self.pixels {
            extent(pixel.source_offset, pixel.tiles.len() * 32, 4)?;
            pixel.bytes()?;
            if !pixel_targets.contains(&pixel.source_offset) {
                return Err("Unreferenced opcode-39 special pixel block".into());
            }
        }
        Ok(())
    }
    pub fn validate_producers(
        &self,
        scripts: &script_archive::ScriptArchive,
    ) -> Result<(), String> {
        self.validate()?;
        if self.producer_pairs != producer_pairs(scripts)? {
            return Err("Opcode-39 special producer witnesses differ".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans = Vec::new();
        spans.extend(
            self.cells
                .iter()
                .map(|c| (c.source_offset, c.target.bytes().to_vec(), "index")),
        );
        spans.extend(
            self.sequences
                .iter()
                .map(|s| (s.source_offset, s.bytes(), "sequence")),
        );
        for object in &self.objects {
            spans.push((object.source_offset, object.source_bytes()?, "objects"));
        }
        for pixel in &self.pixels {
            spans.push((pixel.source_offset, pixel.bytes()?, "pixel"));
        }
        spans.sort_by_key(|s| s.0);
        let mut out: Vec<SourceSpan> = Vec::new();
        for (at, data, role) in spans {
            if let Some(last) = out.last_mut() {
                let end = last.0 + last.1.len() as u32;
                if end > at {
                    return Err("Overlapping new opcode-39 special pose sources".into());
                }
                if end == at && last.2 == role {
                    last.1.extend(data);
                    continue;
                }
            }
            out.push((at, data, role));
        }
        Ok(out)
    }
}

pub struct Catalog {
    pub decoded: Script39SpecialPoses,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(
        rom: &[u8],
        previous: &[Resource],
        scripts: &script_archive::ScriptArchive,
        special: &metadata14::Metadata14,
    ) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Opcode-39 special poses require the exact EUR ROM".into());
        }
        scripts.verify_source(rom)?;
        special.verify_source(rom)?;
        for (start, end, hash) in [
            (
                0x126b54,
                0x126b6a,
                "389869db5dc637b0aa5f8ead8e0b8f33f82142d92746fbd372443073c9f0273b",
            ),
            (
                0x12aa34,
                0x12aa68,
                "0ee4dbc2f40802ebf36673a21bcb55599470d6c648dbce09db72fb94898ce05c",
            ),
            (
                0x12a8c4,
                0x12a940,
                "27773cc9b3db2fde9e51149976dce1cd146f74cf905f7dbec2fa679a7e31efcd",
            ),
            (
                0x12909c,
                0x1290ec,
                "bf64af22a3c06ef28dbc5b61c86969d00825fd72023ba2623db34bb4cc3c9006",
            ),
            (
                0xcbd24,
                0xcbd74,
                "544a2115e0e92a7618db8bb62fb7a0fd1659b662220ad9c802c689f036466de9",
            ),
            (
                0xcbad4,
                0xcbaec,
                "73117091fb363fb7c92d82cc4fe012fb8e8290c633d864a095a972ccae69a018",
            ),
            (
                0xcf144,
                0xcf180,
                "791bca0c9520ddaf9a531a4dc84bb54feec005590f84c66c1a30c29dc4c75a6c",
            ),
            (
                0xcf180,
                0xcf1bc,
                "5aa1e92cf6c0be7d1e124e2ad58f4d0dcbf6b8518baccd8c2499451a2bdbccaf",
            ),
            (
                0x9b734,
                0x9b844,
                "d6efd8425abbd261bdea4b8e0a58b964e8a3abf2fc8081cace5e725bf1504749",
            ),
            (
                0x9a498,
                0x9a4a4,
                "87a96ac9b624426930b1c117b9e2b0809fc860ad0b25bab627b1d9211207c8a9",
            ),
            (
                0x9cc78,
                0x9cc98,
                "272f3483ae5f4e26102f86e0666a49325a27bdb382254fe34ea4f0543b897433",
            ),
            (
                0x9e274,
                0x9e294,
                "edaf65be1358d382721fb49025c4593324df4631f9b41726455162e4bfa06335",
            ),
            (
                0x9acf8,
                0x9ae1c,
                "d111a66dee93686aee00a94af3e17a73695b6db48ebea126687119433f476d33",
            ),
            (
                0x9d9c4,
                0x9d9f0,
                "f36ff8212e95dcdcf20d7f9fc33e607758935c0dc40c0d93d6973e3c69cb5b9d",
            ),
            (
                0x9ae1c,
                0x9aeb4,
                "742793989ab85cca2c28ef48b3a4fba550c5da9bc350cbe54a35c544df2e1795",
            ),
        ] {
            if ffta_rom::digest(bytes(rom, start, (end - start) as usize)?) != hash {
                return Err("Opcode-39 special producer fingerprint mismatch".into());
            }
        }
        let command = script_commands::command_source_offset(39);
        if half(rom, command)? != 5 || word(rom, command + 2)? != 0x0812_6b55 {
            return Err("Opcode-39 special command directory mismatch".into());
        }
        let pairs = producer_pairs(scripts)?;
        let prior = PriorSources::new(previous)?;
        let mut physical = BTreeSet::new();
        for (sprite, animation, part) in producer_keys(&pairs, special)? {
            let root = pointer(word(rom, initial::DIRECTORY + 4 * u32::from(sprite))?)?;
            physical.insert(index_offset(root, u32::from(animation), u32::from(part)));
        }
        if physical.len() != 2334 {
            return Err("Opcode-39 special physical index projection mismatch".into());
        }
        let mut cells = Vec::new();
        let mut seqs = BTreeMap::new();
        let mut objects = BTreeMap::new();
        let mut pixels: BTreeMap<u32, PixelBlock> = BTreeMap::new();
        for at in physical {
            if prior.contains(at, 4)? {
                continue;
            }
            let raw = word(rom, at)?;
            let target = if raw == 0 {
                IndexTarget::Null {}
            } else {
                let source_offset = pointer(raw)?;
                let count = usize::from(half(rom, source_offset)?);
                if count == 0 || count >= 32768 {
                    return Err("Invalid opcode-39 special sequence count".into());
                }
                let frames: Vec<_> = bytes(rom, source_offset + 4, count * 20)?
                    .chunks_exact(20)
                    .map(|b| Frame::from_bytes(b.try_into().unwrap()))
                    .collect();
                for frame in &frames {
                    if frame.operation != 1 {
                        continue;
                    }
                    let object_at = initial::OBJECT_BASE
                        .checked_add(frame.word04)
                        .ok_or("Opcode-39 special object overflow")?;
                    let object_len = 2 + usize::from(half(rom, object_at)?) * 6;
                    let list =
                        obj_lists::List::decode(object_at, bytes(rom, object_at, object_len)?)?;
                    let pixel_at = initial::PIXEL_BASE
                        .checked_add(frame.word00)
                        .ok_or("Opcode-39 special pixel overflow")?;
                    let pixel_len =
                        initial::transfer_bytes(initial::transfer_tiles(&list)) as usize;
                    if pixel_len == 0 || pixel_len % 32 != 0 {
                        return Err("Invalid opcode-39 special pixel extent".into());
                    }
                    if !prior.contains(object_at, object_len)? {
                        if let Some(old) = objects.insert(object_at, list.clone()) {
                            if old != list {
                                return Err("Conflicting opcode-39 special object views".into());
                            }
                        }
                    }
                    if !prior.contains(pixel_at, pixel_len)? {
                        let block = PixelBlock {
                            source_offset: pixel_at,
                            tiles: bytes(rom, pixel_at, pixel_len)?
                                .chunks_exact(32)
                                .map(|b| Tile::decode(b.try_into().unwrap()))
                                .collect(),
                        };
                        if let Some(old) = pixels.insert(pixel_at, block.clone()) {
                            if old != block {
                                return Err("Conflicting opcode-39 special pixel extents".into());
                            }
                        }
                    }
                }
                if !prior.contains(source_offset, 4 + count * 20)? {
                    let sequence = Sequence {
                        source_offset,
                        opaque_header: half(rom, source_offset + 2)?,
                        frames,
                    };
                    if let Some(old) = seqs.insert(source_offset, sequence.clone()) {
                        if old != sequence {
                            return Err("Conflicting opcode-39 special sequence views".into());
                        }
                    }
                }
                IndexTarget::Sequence { source_offset }
            };
            cells.push(IndexCell {
                source_offset: at,
                target,
            });
        }
        let decoded = Script39SpecialPoses {
            producer_pairs: pairs,
            cells,
            sequences: seqs.into_values().collect(),
            objects: objects.into_values().collect(),
            pixels: pixels.into_values().collect(),
        };
        decoded.validate_producers(scripts)?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, data, role) in decoded.source_spans()? {
            if bytes(rom, offset, data.len())? != data {
                return Err("Opcode-39 special source reconstruction mismatch".into());
            }
            combined.extend_from_slice(&data);
            sources.push(SourceRange {
                offset,
                length: data.len() as u32,
                sha256: ffta_rom::digest(&data),
                role: role.into(),
            });
        }
        if sources.len() != 418
            || combined.len() != 34026
            || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("Opcode-39 special source plan/hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
