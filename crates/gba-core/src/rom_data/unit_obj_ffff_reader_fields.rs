//! Two numeric ROM reads selected by the known full-width FFFF sprite query.
//! The encoded directory word is preserved; following it remains guest bus work.
use super::{
    metadata52, script_archive, unit_obj_initial_pose, unit_obj_property_words,
    unit_obj_script_registry_generic as registry, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub use registry::Mode;
pub const SPRITE_ID: u16 = u16::MAX;
pub const PROPERTY_OFFSET: u32 = unit_obj_property_words::SOURCE_BASE + 2 * SPRITE_ID as u32;
pub const DIRECTORY_OFFSET: u32 = unit_obj_initial_pose::DIRECTORY + 4 * SPRITE_ID as u32;
pub const PROPERTY_SHA256: &str =
    "f3baef64724bedf2f6780d43c08dcecf8383be899a5164ee4b3b0331918cf8c2";
pub const DIRECTORY_SHA256: &str =
    "5ee0214ff8713ddff0da3b619b1ece368d349c7546ac0dd3f1ebe1b232b74ff2";
pub type SourceSpan = (u32, Vec<u8>, String);

fn field(mode: Mode) -> u32 {
    match mode {
        Mode::Normal => 7,
        Mode::Alternate => 9,
    }
}
fn effective_jobs(mode: Mode) -> &'static [u8] {
    match mode {
        Mode::Normal => &[0, 1],
        Mode::Alternate => &[0, 1, 46, 47, 48, 49, 50, 64, 65, 69, 70, 71],
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub mode: Mode,
    pub unit_kind: u8,
    pub selector: u8,
    pub base: u16,
    pub current_job: u8,
    /// None leaves the alternate byte unrestricted on a direct route0 row.
    pub alternate_job: Option<u8>,
    pub effective_job: u8,
    pub primary_routing_source: u32,
    pub effective_field_source: u32,
    pub script_sources: Vec<u32>,
}
impl Producer {
    fn key(&self) -> (Mode, u8, u8, u16, u8, Option<u8>) {
        (
            self.mode,
            self.unit_kind,
            self.selector,
            self.base,
            self.current_job,
            self.alternate_job,
        )
    }
}

/// Reuse the authenticated archive pair parser and metadata52 classification.
/// These conditional witnesses do not freeze the live registry or actor identity.
pub fn producer_witnesses(
    scripts: &script_archive::ScriptArchive,
    jobs: &metadata52::Metadata52,
) -> Result<Vec<Producer>, String> {
    let mut references = BTreeMap::new();
    for p in registry::producer_pairs(scripts, jobs)? {
        references
            .entry((p.selector, p.base))
            .or_insert(p.script_sources);
    }
    let mut out = Vec::new();
    for mode in [Mode::Normal, Mode::Alternate] {
        for unit_kind in [0, 1] {
            for (&(selector, base), sources) in &references {
                for (current, primary) in jobs.records.iter().enumerate() {
                    for &effective in effective_jobs(mode) {
                        let row = &jobs.records[usize::from(effective)];
                        if row.routing != 0
                            || row.packed_property(if mode == Mode::Normal { 4 } else { 5 })
                                != Some(u32::from(SPRITE_ID))
                        {
                            return Err("FFFF reader effective job source mismatch".into());
                        }
                        let alternate_job = match primary.routing {
                            0 if current == usize::from(effective) => None,
                            255 => Some(effective),
                            _ => continue,
                        };
                        out.push(Producer {
                            mode,
                            unit_kind,
                            selector,
                            base,
                            current_job: current as u8,
                            alternate_job,
                            effective_job: effective,
                            primary_routing_source: metadata52::SOURCE_OFFSET
                                + current as u32 * 52
                                + 5,
                            effective_field_source: metadata52::SOURCE_OFFSET
                                + u32::from(effective) * 52
                                + field(mode),
                            script_sources: sources.clone(),
                        });
                    }
                }
            }
        }
    }
    out.sort_by_key(Producer::key);
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyWord {
    pub source_offset: u32,
    pub value: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryWord {
    pub source_offset: u32,
    /// Full encoded word, including non-ROM regions and low address bits.
    pub value: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjFfffReaderFields {
    pub sprite_id: u16,
    pub property_word: PropertyWord,
    pub directory_word: DirectoryWord,
    pub producers: Vec<Producer>,
}
impl UnitObjFfffReaderFields {
    pub fn validate(&self) -> Result<(), String> {
        if self.sprite_id != SPRITE_ID
            || self.property_word.source_offset != PROPERTY_OFFSET
            || self.directory_word.source_offset != DIRECTORY_OFFSET
            || self.producers.len() != 980
            || !self.producers.windows(2).all(|p| p[0].key() < p[1].key())
        {
            return Err("FFFF reader field identity/source/projection mismatch".into());
        }
        let mut references = BTreeMap::new();
        let mut unique_sources = BTreeSet::new();
        let mut classes = [0usize; 2];
        let mut reference_count = 0;
        for p in &self.producers {
            let redirected = match p.alternate_job {
                None if p.current_job == p.effective_job => false,
                Some(job)
                    if job == p.effective_job && [80, 81, 82, 85].contains(&p.current_job) =>
                {
                    true
                }
                _ => return Err("FFFF reader conditional job route mismatch".into()),
            };
            if p.unit_kind > 1
                || !registry::REGISTRY_PAIRS.contains(&(p.selector, p.base))
                || !effective_jobs(p.mode).contains(&p.effective_job)
                || p.primary_routing_source
                    != metadata52::SOURCE_OFFSET + u32::from(p.current_job) * 52 + 5
                || p.effective_field_source
                    != metadata52::SOURCE_OFFSET + u32::from(p.effective_job) * 52 + field(p.mode)
                || p.script_sources.is_empty()
                || !p.script_sources.windows(2).all(|s| s[0] < s[1])
                || p.script_sources.iter().any(|&at| {
                    at < script_archive::ARCHIVE_OFFSET || at > script_archive::ARCHIVE_END - 5
                })
            {
                return Err("FFFF reader producer/reference mismatch".into());
            }
            let expected = references
                .entry((p.selector, p.base))
                .or_insert(&p.script_sources);
            if **expected != p.script_sources {
                return Err("FFFF reader conditional reference aliases differ".into());
            }
            classes[usize::from(redirected)] += 1;
            reference_count += p.script_sources.len();
            unique_sources.extend(p.script_sources.iter().copied());
        }
        if classes != [196, 784]
            || reference_count != 2240
            || unique_sources.len() != 16
            || references.keys().copied().collect::<Vec<_>>() != registry::REGISTRY_PAIRS
        {
            return Err("FFFF reader conditional producer coverage mismatch".into());
        }
        Ok(())
    }
    pub fn validate_dependencies(
        &self,
        scripts: &script_archive::ScriptArchive,
        jobs: &metadata52::Metadata52,
    ) -> Result<(), String> {
        self.validate()?;
        if self.producers != producer_witnesses(scripts, jobs)? {
            return Err("FFFF reader metadata/archive dependency mismatch".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![
            (
                self.property_word.source_offset,
                self.property_word.value.to_le_bytes().to_vec(),
                "property-word".into(),
            ),
            (
                self.directory_word.source_offset,
                self.directory_word.value.to_le_bytes().to_vec(),
                "directory-word".into(),
            ),
        ])
    }
    pub fn byte_at(&self, source: u32) -> Option<u8> {
        if let Some(at) = source.checked_sub(self.property_word.source_offset) {
            if let Some(value) = self.property_word.value.to_le_bytes().get(at as usize) {
                return Some(*value);
            }
        }
        let at = source.checked_sub(self.directory_word.source_offset)?;
        self.directory_word
            .value
            .to_le_bytes()
            .get(at as usize)
            .copied()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("FFFF reader fields differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: UnitObjFfffReaderFields,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("FFFF reader fields require exact EUR ROM; USA pending".into());
        }
        for (at, len, expected) in [
            (
                0x22174,
                62,
                "cc76331f11dee50a4b083d6e943a25cab59d10d62c92c1000b94bb116b22f7d1",
            ),
            (
                0x221c4,
                16,
                "bb5cbae174700ce609bb594d543334e9c862b5f624bb409cda8fdc285ba282af",
            ),
        ] {
            if ffta_rom::digest(&rom[at..at + len]) != expected {
                return Err("FFFF original reader fingerprint mismatch".into());
            }
        }
        let jobs = metadata52::Catalog::from_rom(rom)?.decoded;
        let scripts = script_archive::Catalog::from_rom(rom)?.decoded;
        let decoded = UnitObjFfffReaderFields {
            sprite_id: SPRITE_ID,
            property_word: PropertyWord {
                source_offset: PROPERTY_OFFSET,
                value: u16::from_le_bytes(
                    rom[PROPERTY_OFFSET as usize..PROPERTY_OFFSET as usize + 2]
                        .try_into()
                        .unwrap(),
                ),
            },
            directory_word: DirectoryWord {
                source_offset: DIRECTORY_OFFSET,
                value: u32::from_le_bytes(
                    rom[DIRECTORY_OFFSET as usize..DIRECTORY_OFFSET as usize + 4]
                        .try_into()
                        .unwrap(),
                ),
            },
            producers: producer_witnesses(&scripts, &jobs)?,
        };
        decoded.validate_dependencies(&scripts, &jobs)?;
        let spans = decoded.source_spans()?;
        for ((_, bytes, _), expected) in spans.iter().zip([PROPERTY_SHA256, DIRECTORY_SHA256]) {
            if ffta_rom::digest(bytes) != expected {
                return Err("FFFF reader field source hash mismatch".into());
            }
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
