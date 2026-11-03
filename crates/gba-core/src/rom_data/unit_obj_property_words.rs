//! Raw actor property words selected by authenticated normal/alternate assignments.
//! The original getter, allocation, task state and bus behavior remain guest work.
use super::{metadata14, metadata52, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SOURCE_BASE: u32 = 0x394c14;
pub const SOURCE_SHA256: &str = "ad328c8ebccfc43091dc878c06dab33b22521d7b617d4b11232e4a532ffc2208";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Normal,
    Alternate,
}
impl Mode {
    fn field(self) -> u32 {
        match self {
            Self::Normal => 7,
            Self::Alternate => 9,
        }
    }
    fn special_field(self) -> u32 {
        match self {
            Self::Normal => 4,
            Self::Alternate => 6,
        }
    }
    fn forced_sprite(self) -> u16 {
        match self {
            Self::Normal => 42,
            Self::Alternate => 230,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Job,
    Special,
    Override,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub mode: Mode,
    pub kind: Kind,
    pub logical_id: u8,
    pub metadata_source: Option<u32>,
    pub sprite_id: u16,
}
impl Producer {
    fn key(&self) -> (Mode, Kind, u8) {
        (self.mode, self.kind, self.logical_id)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Word {
    pub sprite_id: u16,
    pub source_offset: u32,
    pub word: u16,
}

/// A finite producer projection, not an extent or cap for live sprite inputs.
pub fn producer_pairs(
    jobs: &metadata52::Metadata52,
    special: &metadata14::Metadata14,
) -> Result<Vec<Producer>, String> {
    jobs.validate()?;
    special.validate_metadata52(jobs)?;
    let mut producers = Vec::new();
    for mode in [Mode::Normal, Mode::Alternate] {
        for (i, job) in jobs.records.iter().enumerate() {
            let sprite_id = job
                .packed_property(if mode == Mode::Normal { 4 } else { 5 })
                .unwrap() as u16;
            if job.routing == 0 && sprite_id != u16::MAX {
                producers.push(Producer {
                    mode,
                    kind: Kind::Job,
                    logical_id: i as u8,
                    metadata_source: Some(metadata52::SOURCE_OFFSET + i as u32 * 52 + mode.field()),
                    sprite_id,
                });
            }
        }
        for unit in &special.records {
            producers.push(Producer {
                mode,
                kind: Kind::Special,
                logical_id: unit.id,
                metadata_source: Some(unit.source_offset + mode.special_field()),
                sprite_id: unit.halfwords04_06[usize::from(mode == Mode::Alternate)],
            });
        }
        producers.push(Producer {
            mode,
            kind: Kind::Override,
            logical_id: 0,
            metadata_source: None,
            sprite_id: mode.forced_sprite(),
        });
    }
    Ok(producers)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitObjPropertyWords {
    pub producers: Vec<Producer>,
    pub words: Vec<Word>,
}
impl UnitObjPropertyWords {
    pub fn validate(&self) -> Result<(), String> {
        if self.producers.len() != 398
            || self.words.len() != 182
            || !self.producers.windows(2).all(|w| w[0].key() < w[1].key())
            || !self
                .words
                .windows(2)
                .all(|w| w[0].sprite_id < w[1].sprite_id)
        {
            return Err("Actor propertyword projection shape/order mismatch".into());
        }
        let mut classes = [0usize; 6];
        for p in &self.producers {
            let expected = match p.kind {
                Kind::Job if usize::from(p.logical_id) < metadata52::RECORD_COUNT => {
                    classes[usize::from(p.mode == Mode::Alternate) * 3] += 1;
                    Some(metadata52::SOURCE_OFFSET + u32::from(p.logical_id) * 52 + p.mode.field())
                }
                Kind::Special if (2..=94).contains(&p.logical_id) => {
                    classes[usize::from(p.mode == Mode::Alternate) * 3 + 1] += 1;
                    Some(
                        metadata14::ADDRESS_BIAS
                            + u32::from(p.logical_id) * 14
                            + p.mode.special_field(),
                    )
                }
                Kind::Override if p.logical_id == 0 && p.sprite_id == p.mode.forced_sprite() => {
                    classes[usize::from(p.mode == Mode::Alternate) * 3 + 2] += 1;
                    None
                }
                _ => return Err("Invalid actor propertyword producer".into()),
            };
            if p.metadata_source != expected || p.sprite_id == u16::MAX {
                return Err("Actor propertyword producer source/sentinel mismatch".into());
            }
        }
        if classes != [110, 93, 1, 100, 93, 1]
            || self
                .producers
                .iter()
                .map(|p| p.sprite_id)
                .collect::<BTreeSet<_>>()
                != self.words.iter().map(|w| w.sprite_id).collect()
            || self
                .words
                .iter()
                .any(|w| w.source_offset != SOURCE_BASE + 2 * u32::from(w.sprite_id))
        {
            return Err("Actor propertyword producer/query mismatch".into());
        }
        Ok(())
    }
    pub fn validate_producers(
        &self,
        jobs: &metadata52::Metadata52,
        special: &metadata14::Metadata14,
    ) -> Result<(), String> {
        self.validate()?;
        if self.producers != producer_pairs(jobs, special)? {
            return Err("Actor propertyword metadata dependency mismatch".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans: Vec<SourceSpan> = Vec::new();
        for word in &self.words {
            let bytes = word.word.to_le_bytes();
            if let Some((at, data, _)) = spans.last_mut() {
                if *at + data.len() as u32 == word.source_offset {
                    data.extend_from_slice(&bytes);
                    continue;
                }
            }
            spans.push((word.source_offset, bytes.to_vec(), "property-word".into()));
        }
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Actor propertyword model differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: UnitObjPropertyWords,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Actor propertywords require exact EUR ROM; USA pending".into());
        }
        for (at, size, expected) in [
            (
                0x221c4,
                16,
                "bb5cbae174700ce609bb594d543334e9c862b5f624bb409cda8fdc285ba282af",
            ),
            (
                0x9acf8,
                0x124,
                "d111a66dee93686aee00a94af3e17a73695b6db48ebea126687119433f476d33",
            ),
            (
                0x9b734,
                0x110,
                "d6efd8425abbd261bdea4b8e0a58b964e8a3abf2fc8081cace5e725bf1504749",
            ),
        ] {
            if ffta_rom::digest(&rom[at..at + size]) != expected {
                return Err("Actor propertyword original reader/assignment mismatch".into());
            }
        }
        let jobs = metadata52::Catalog::from_rom(rom)?.decoded;
        let special = metadata14::Catalog::from_rom(rom)?.decoded;
        let producers = producer_pairs(&jobs, &special)?;
        let words = producers
            .iter()
            .map(|p| p.sprite_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|sprite_id| {
                let at = SOURCE_BASE + 2 * u32::from(sprite_id);
                Word {
                    sprite_id,
                    source_offset: at,
                    word: u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap()),
                }
            })
            .collect();
        let decoded = UnitObjPropertyWords { producers, words };
        decoded.validate_producers(&jobs, &special)?;
        let spans = decoded.source_spans()?;
        let combined: Vec<_> = spans
            .iter()
            .flat_map(|(_, data, _)| data.iter().copied())
            .collect();
        if spans.len() != 4 || combined.len() != 364 || ffta_rom::digest(&combined) != SOURCE_SHA256
        {
            return Err("Actor propertyword source contract mismatch".into());
        }
        let sources = spans
            .into_iter()
            .map(|(offset, data, role)| SourceRange {
                offset,
                length: data.len() as u32,
                sha256: ffta_rom::digest(&data),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}
