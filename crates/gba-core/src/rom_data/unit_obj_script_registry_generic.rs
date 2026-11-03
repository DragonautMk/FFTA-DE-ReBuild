//! Conditional EUR registry script poses for direct generic-job current assignments.
//! Live facing, animation commands, task allocation, DMA and VRAM remain guest work.
use super::{
    metadata52, obj_lists, script_archive, unit_obj_alternate_battle as alternate,
    unit_obj_equipment as equipment, unit_obj_initial_pose as initial,
    unit_obj_normal_battle as normal, unit_obj_script_registry_special as known_special,
    unit_obj_script_special_alternate as low_alternate, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE_SHA256: &str = "44033ff6d5c52eb5ce768cf725db2b1c5bd906fe98a36be7082874304921cdd9";
pub type SourceSpan = (u32, Vec<u8>, String);

pub use alternate::{Binding, Target};
pub use equipment::{index_offset, Frame, Root, Sequence};

/// Original assignment mode within the conditional known-unit source domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Normal,
    Alternate,
}
impl Mode {
    fn field(self) -> usize {
        match self {
            Self::Normal => 7,
            Self::Alternate => 9,
        }
    }
}
/// Exact archive pairs, not live selector/base caps or fixed registry identities.
pub const REGISTRY_PAIRS: [(u8, u16); 7] = [
    (128, 8),
    (128, 48),
    (128, 136),
    (129, 0),
    (129, 140),
    (132, 136),
    (133, 136),
];
/// Conditional generic unit byte4=0/1 and direct route0 current-job assignment.
/// The original alternate-job byte is unrestricted for direct rows. Sentinel,
/// redirect, out-of-catalog and changed-current-actor cases remain separate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub mode: Mode,
    pub unit_kind: u8,
    pub selector: u8,
    pub job_id: u8,
    pub base: u16,
    pub sprite_id: u16,
    pub metadata_source: u32,
    pub script_sources: Vec<u32>,
}
impl Producer {
    pub fn key(&self) -> (Mode, u8, u8, u16, u8) {
        (
            self.mode,
            self.unit_kind,
            self.selector,
            self.base,
            self.job_id,
        )
    }
}
pub fn producer_pairs(
    scripts: &script_archive::ScriptArchive,
    jobs: &metadata52::Metadata52,
) -> Result<Vec<Producer>, String> {
    scripts.validate()?;
    jobs.validate()?;
    let mut pairs: BTreeMap<(u8, u16), Vec<u32>> = BTreeMap::new();
    let (mut low, mut high) = (0, 0);
    for instruction in scripts.programs.iter().flat_map(|p| &p.instructions) {
        if instruction.command != 49 {
            continue;
        }
        let p = &instruction.parameters;
        if p.len() != 4 {
            return Err("Registry-generic script49 parameter shape mismatch".into());
        }
        if p[0] < 128 {
            if p[0] <= 1 {
                return Err("Unexpected generic low-selector script reference".into());
            }
            low += 1;
            continue;
        }
        high += 1;
        pairs
            .entry((p[0], u16::from_le_bytes([p[1], p[2]]).wrapping_mul(4)))
            .or_default()
            .push(instruction.source_offset);
    }
    if low != 151 || high != 16 || pairs.keys().copied().collect::<Vec<_>>() != REGISTRY_PAIRS {
        return Err("Registry-generic script producer projection shape mismatch".into());
    }
    // Preserve/classify the entire known table before selecting concrete
    // mode-specific fields. This does not normalize a live FFFF/redirect input.
    let mut classes = [0usize; 4];
    for row in &jobs.records {
        let n = row.packed_property(4).unwrap();
        let a = row.packed_property(5).unwrap();
        match (row.routing, n == 65535, a == 65535) {
            (0, false, false) => classes[0] += 1,
            (0, false, true) => classes[1] += 1,
            (0, true, true) => classes[2] += 1,
            (255, _, _) => classes[3] += 1,
            _ => return Err("Registry-generic job source classification mismatch".into()),
        }
    }
    if classes != [100, 10, 2, 4] {
        return Err("Registry-generic job mode-specific source shape mismatch".into());
    }
    let mut out = Vec::new();
    for mode in [Mode::Normal, Mode::Alternate] {
        let property = if mode == Mode::Normal { 4 } else { 5 };
        for unit_kind in [0, 1] {
            for (&(selector, base), sources) in &pairs {
                for (job_id, row) in jobs.records.iter().enumerate() {
                    let sprite_id = row.packed_property(property).unwrap() as u16;
                    if row.routing != 0 || sprite_id == u16::MAX {
                        continue;
                    }
                    out.push(Producer {
                        mode,
                        unit_kind,
                        selector,
                        job_id: job_id as u8,
                        base,
                        sprite_id,
                        metadata_source: metadata52::SOURCE_OFFSET
                            + job_id as u32 * 52
                            + mode.field() as u32,
                        script_sources: sources.clone(),
                    });
                }
            }
        }
    }
    if out.len() != 2940 {
        return Err("Registry-generic direct job witness count mismatch".into());
    }
    Ok(out)
}
pub fn producer_keys(producers: &[Producer]) -> Vec<(u16, u16, u8)> {
    producers
        .iter()
        .flat_map(|p| {
            [0u16, 1].into_iter().flat_map(move |direction| {
                [0u8, 1, 2]
                    .into_iter()
                    .map(move |part| (p.sprite_id, p.base.wrapping_add(direction), part))
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjScriptRegistryGeneric {
    pub producers: Vec<Producer>,
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
        return Err("Invalid registry-generic script OBJ source extent/alignment".into());
    }
    Ok(())
}
impl UnitObjScriptRegistryGeneric {
    pub fn validate(&self) -> Result<(), String> {
        if self.producers.len() != 2940
            || self.roots.len() != 107
            || self.bindings.len() != 3210
            || self.sequences.len() != 777
            || self.objects.len() != 45
            || self.pixels.len() != 1895
        {
            return Err("Registry-generic script OBJ projection shape mismatch".into());
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
            return Err("Unordered/duplicate registry-generic script OBJ identity".into());
        }
        if !self.producers.windows(2).all(|p| p[0].key() < p[1].key()) {
            return Err("Unordered/duplicate registry-generic script producer witness".into());
        }
        let mut references = BTreeSet::new();
        let mut canonical = BTreeMap::new();
        for p in &self.producers {
            if !REGISTRY_PAIRS.contains(&(p.selector, p.base))
                || p.unit_kind > 1
                || !(2..metadata52::RECORD_COUNT as u8).contains(&p.job_id)
                || p.sprite_id == u16::MAX
                || p.metadata_source
                    != metadata52::SOURCE_OFFSET + u32::from(p.job_id) * 52 + p.mode.field() as u32
                || p.script_sources.is_empty()
                || !p.script_sources.windows(2).all(|w| w[0] < w[1])
            {
                return Err("Invalid registry-generic script producer witness".into());
            }
            let expected = canonical
                .entry((p.selector, p.base))
                .or_insert(&p.script_sources);
            if **expected != p.script_sources {
                return Err("Registry-generic conditional reference views differ".into());
            }
            for &source in &p.script_sources {
                if source < script_archive::ARCHIVE_OFFSET
                    || source > script_archive::ARCHIVE_END - 5
                    || !references.insert((p.mode, p.unit_kind, p.job_id, source))
                {
                    return Err("Invalid/duplicate registry-generic instruction reference".into());
                }
            }
        }
        if references.len() != 6720
            || self.bindings.iter().map(Binding::key).collect::<Vec<_>>()
                != producer_keys(&self.producers)
        {
            return Err("Registry-generic script producer/key coverage mismatch".into());
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
                return Err("Unreferenced registry-generic script OBJ root".into());
            }
        }
        for b in &self.bindings {
            let root = *roots
                .get(&b.sprite_id)
                .ok_or("Missing registry-generic script OBJ root")?;
            if b.part > 2
                || b.animation & 3 > 1
                || b.index_source != index_offset(root, u32::from(b.animation), u32::from(b.part))
            {
                return Err("Invalid registry-generic script OBJ query/cell link".into());
            }
            extent(b.index_source, 4, 4)?;
            match b.target {
                Target::Sequence { source_offset } => {
                    if self
                        .sequences
                        .binary_search_by_key(&source_offset, |s| s.source_offset)
                        .is_err()
                    {
                        return Err("Missing registry-generic script OBJ sequence".into());
                    }
                    referenced.insert(source_offset);
                }
                Target::SharedIndexRoot { source_offset, .. } => {
                    extent(source_offset, 4, 4)?;
                    if self
                        .sequences
                        .iter()
                        .any(|s| s.source_offset == source_offset)
                    {
                        return Err("Registry-generic index root mislabeled as sequence".into());
                    }
                }
                Target::Null {} => {}
            }
        }
        if self
            .bindings
            .iter()
            .filter(|b| matches!(b.target, Target::SharedIndexRoot { .. }))
            .count()
            != 12
        {
            return Err("Registry-generic shared index-root projection mismatch".into());
        }
        if referenced.len() != self.sequences.len() {
            return Err("Unreferenced registry-generic script OBJ sequence".into());
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
                    "Registry-generic script OBJ sequence outside positive-count projection".into(),
                );
            }
            extent(seq.source_offset, 4 + seq.frames.len() * 20, 4)?;
            for frame in &seq.frames {
                if frame.operation == 1 {
                    let at = initial::OBJECT_BASE
                        .checked_add(frame.word04)
                        .ok_or("Registry-generic script OBJ object overflow")?;
                    let list = self
                        .objects
                        .iter()
                        .find(|o| o.source_offset == at)
                        .ok_or("Missing registry-generic script OBJ object link")?;
                    let source = initial::PIXEL_BASE
                        .checked_add(frame.word00)
                        .ok_or("Registry-generic script OBJ pixel overflow")?;
                    let len = initial::transfer_bytes(initial::transfer_tiles(list)) as usize;
                    if !self
                        .pixels
                        .iter()
                        .any(|p| p.source_offset == source && p.tiles.len() * 32 == len)
                    {
                        return Err("Missing registry-generic script OBJ pixel transfer".into());
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
        scripts: &script_archive::ScriptArchive,
        jobs: &metadata52::Metadata52,
    ) -> Result<(), String> {
        self.validate()?;
        if self.producers != producer_pairs(scripts, jobs)? {
            return Err("Registry-generic script dependency witnesses differ".into());
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
                    // Logical keys may alias one physical index cell. Accept
                    // only an exact same-role suffix; no inferred bank extent.
                    let start = (at - *last_at) as usize;
                    if end == at + bytes.len() as u32
                        && *last_role == role
                        && last_bytes[start..] == bytes
                    {
                        continue;
                    }
                    return Err("Conflicting registry-generic script OBJ physical views".into());
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
    /// A shared target names a separately authenticated initial index root,
    /// not a header selected by a count/neighbor heuristic.
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
                        .ok_or("Missing registry-generic initial-root witness")?;
                    if source_offset != witness.index_source
                        || b.index_source != initial::DIRECTORY + 4 * u32::from(sprite_id)
                    {
                        return Err(
                            "Registry-generic shared root differs from initial witness".into()
                        );
                    }
                }
                Target::Sequence { source_offset } => {
                    if initial_pose
                        .bindings
                        .iter()
                        .any(|w| w.index_source == source_offset)
                    {
                        return Err("Registry-generic initial root mislabeled as sequence".into());
                    }
                }
                Target::Null {} => {}
            }
        }
        Ok(())
    }
    /// Validate every full shared view, even where an earlier dependency
    /// covers the same physical bytes, then subtract their union.
    pub fn owned_spans(
        &self,
        initial_pose: &initial::UnitObjInitialPose,
        equipment_pose: &equipment::UnitObjEquipment,
        normal_pose: &normal::UnitObjNormalBattle,
        alternate_pose: &alternate::UnitObjAlternateBattle,
        low_alternate_pose: &low_alternate::UnitObjScriptSpecialAlternate,
        known_special_pose: &known_special::UnitObjScriptRegistrySpecial,
    ) -> Result<Vec<SourceSpan>, String> {
        self.validate_initial_views(initial_pose)?;
        alternate_pose.validate_initial_views(initial_pose)?;
        known_special_pose.validate()?;
        let mut previous = initial_pose.source_spans()?;
        previous.extend(equipment_pose.source_spans()?);
        previous.extend(normal_pose.source_spans()?);
        previous.extend(alternate_pose.source_spans()?);
        previous.extend(low_alternate_pose.source_spans()?);
        previous.extend(known_special_pose.source_spans()?);
        previous.sort_by_key(|s| s.0);
        let mut owned = Vec::new();
        for (at, bytes, role) in self.source_spans()? {
            let end = at + bytes.len() as u32;
            let mut cursor = at;
            for (old_at, old_bytes, _) in previous.iter().filter(|s| s.0 + s.1.len() as u32 > at) {
                if *old_at >= end {
                    break;
                }
                let lo = at.max(*old_at);
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
                    return Err(
                        "Registry-generic script OBJ shared view differs from dependency".into(),
                    );
                }
                cursor = cursor.max(hi);
            }
            if cursor < end {
                owned.push((cursor, bytes[(cursor - at) as usize..].to_vec(), role));
            }
        }
        Ok(owned)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Registry-generic script OBJ differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn bytes(rom: &[u8], at: u32, n: usize) -> Result<&[u8], String> {
    extent(at, n, 1)?;
    rom.get(at as usize..at as usize + n)
        .ok_or_else(|| "Truncated registry-generic script OBJ source".into())
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
        .ok_or_else(|| "Registry-generic script OBJ pointer outside ROM".into())
}
pub struct Catalog {
    pub decoded: UnitObjScriptRegistryGeneric,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Registry-generic script OBJ requires exact EUR ROM; USA pending".into());
        }
        for (at, expected) in [
            (0x2219c, initial::DIRECTORY),
            (0x221b8, initial::PIXEL_BASE),
            (0x221c0, initial::OBJECT_BASE),
        ] {
            if pointer(rom, at)? != expected {
                return Err("Registry-generic script OBJ original root mismatch".into());
            }
        }
        // Conditional registry/unit/current-assignment and original body/part
        // control flow. These checks never replace or cap live guest inputs.
        for (at, len, expected) in [
            (
                0xcbad4,
                0x18,
                "73117091fb363fb7c92d82cc4fe012fb8e8290c633d864a095a972ccae69a018",
            ),
            (
                0x1257c4,
                0x56,
                "aae9d889ab661703c84ca86afb5e10ebd2c29d44f04732e7422f817d286afc21",
            ),
            (
                0x9cc78,
                0x20,
                "272f3483ae5f4e26102f86e0666a49325a27bdb382254fe34ea4f0543b897433",
            ),
            (
                0x9e274,
                0x20,
                "edaf65be1358d382721fb49025c4593324df4631f9b41726455162e4bfa06335",
            ),
            (
                0xcf144,
                0x3c,
                "791bca0c9520ddaf9a531a4dc84bb54feec005590f84c66c1a30c29dc4c75a6c",
            ),
            (
                0xcf180,
                0x3c,
                "5aa1e92cf6c0be7d1e124e2ad58f4d0dcbf6b8518baccd8c2499451a2bdbccaf",
            ),
            (
                0x9b734,
                0x110,
                "d6efd8425abbd261bdea4b8e0a58b964e8a3abf2fc8081cace5e725bf1504749",
            ),
            (
                0x126cc0,
                0x4c,
                "333fb21727c0b0553e880a8c8d57da56d550d7fdd11dacce4dd2c638e737baf9",
            ),
            (
                0x1290ec,
                0x5c,
                "da8f38ae35f2e41b695c45afc9f4d1dfbdb6051305c068c8d723731312a6db92",
            ),
            (
                0x9acf8,
                0x124,
                "d111a66dee93686aee00a94af3e17a73695b6db48ebea126687119433f476d33",
            ),
            (
                0x9ae1c,
                0x98,
                "742793989ab85cca2c28ef48b3a4fba550c5da9bc350cbe54a35c544df2e1795",
            ),
            (
                0x9d9c4,
                0x2c,
                "f36ff8212e95dcdcf20d7f9fc33e607758935c0dc40c0d93d6973e3c69cb5b9d",
            ),
            (
                0xccc48,
                0x24,
                "75875c8fb568e97c7d96eee0e6bf7a48f39a75601d60204372f4ad19cd0a4c25",
            ),
            (
                0xcbec8,
                0x20,
                "bb009cad2585fbd95e2df62d9dbed3b337a6cf822e14fcf3d1bde003a47f1ee9",
            ),
            (
                0xcbff8,
                0x30,
                "61bf556c319d104e6c72aef45660580beb09695354938cafcb7aba923a464e81",
            ),
            (
                0xcc35a,
                0x16,
                "1fde9952175ca11d06daa90be25883de84d5e53f285ea189d3b7382c6b41b4e9",
            ),
        ] {
            if ffta_rom::digest(bytes(rom, at, len)?) != expected {
                return Err("Registry-generic script original producer mismatch".into());
            }
        }
        let command = super::script_commands::command_source_offset(49);
        if half(rom, command)? != 5 || word(rom, command + 2)? != 0x08126cc1 {
            return Err("Registry-generic script command directory mismatch".into());
        }
        let scripts = script_archive::Catalog::from_rom(rom)?.decoded;
        let jobs = metadata52::Catalog::from_rom(rom)?.decoded;
        let producers = producer_pairs(&scripts, &jobs)?;
        let initial_pose = initial::Catalog::from_rom(rom)?.decoded;
        let equipment_pose = equipment::Catalog::from_rom(rom)?.decoded;
        let normal_pose = normal::Catalog::from_rom(rom)?.decoded;
        let alternate_pose = alternate::Catalog::from_rom(rom)?.decoded;
        let low_alternate_pose = low_alternate::Catalog::from_rom(rom)?.decoded;
        let known_special_pose = known_special::Catalog::from_rom(rom)?.decoded;
        let initial_roots: BTreeMap<_, _> = initial_pose
            .bindings
            .iter()
            .map(|b| (b.index_source, b.sprite_id))
            .collect();
        let mut known_roots: BTreeSet<_> = initial_roots.keys().copied().collect();
        known_roots.extend(
            equipment_pose
                .roots
                .iter()
                .chain(&normal_pose.roots)
                .chain(&alternate_pose.roots)
                .chain(&low_alternate_pose.roots)
                .chain(&known_special_pose.roots)
                .map(|r| r.source_offset),
        );
        let mut roots = BTreeMap::new();
        let mut bindings = Vec::new();
        let mut sequences = BTreeMap::new();
        let mut objects = BTreeMap::new();
        let mut pixels = BTreeMap::new();
        for (sprite_id, animation, part) in producer_keys(&producers) {
            let root = pointer(rom, initial::DIRECTORY + u32::from(sprite_id) * 4)?;
            roots.insert(
                sprite_id,
                Root {
                    sprite_id,
                    source_offset: root,
                },
            );
            let index_source = index_offset(root, u32::from(animation), u32::from(part));
            let value = word(rom, index_source)?;
            let target = if value == 0 {
                Target::Null {}
            } else {
                let source = pointer(rom, index_source)?;
                if let Some(&witness) = initial_roots.get(&source) {
                    Target::SharedIndexRoot {
                        source_offset: source,
                        sprite_id: witness,
                    }
                } else if known_roots.contains(&source) {
                    return Err("Unexpected registry-generic shared-root provenance".into());
                } else {
                    Target::Sequence {
                        source_offset: source,
                    }
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
                    .ok_or("Registry-generic script OBJ object overflow")?;
                let len = 2 + usize::from(half(rom, object_at)?) * 6;
                let list = obj_lists::List::decode(object_at, bytes(rom, object_at, len)?)?;
                let pixel_at = initial::PIXEL_BASE
                    .checked_add(frame.word00)
                    .ok_or("Registry-generic script OBJ pixel overflow")?;
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
                        return Err("Conflicting registry-generic script OBJ pixel extents".into());
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
        let decoded = UnitObjScriptRegistryGeneric {
            producers,
            roots: roots.into_values().collect(),
            bindings,
            sequences: sequences.into_values().collect(),
            objects: objects.into_values().collect(),
            pixels: pixels.into_values().collect(),
        };
        decoded.validate_producers(&scripts, &jobs)?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, data, role) in decoded.owned_spans(
            &initial_pose,
            &equipment_pose,
            &normal_pose,
            &alternate_pose,
            &low_alternate_pose,
            &known_special_pose,
        )? {
            if bytes(rom, offset, data.len())? != data {
                return Err("Registry-generic script OBJ reconstruction mismatch".into());
            }
            combined.extend_from_slice(&data);
            sources.push(SourceRange {
                offset,
                length: data.len() as u32,
                sha256: ffta_rom::digest(&data),
                role,
            });
        }
        if sources.len() != 659
            || combined.len() != 99824
            || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("Registry-generic script OBJ source plan/hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
