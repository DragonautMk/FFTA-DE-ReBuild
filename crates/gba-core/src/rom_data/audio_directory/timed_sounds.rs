//! Source-rooted timed-sound records and selectors used by the original scheduler.
use super::TimedSoundRow;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

#[path = "timed_sound_shapes.rs"]
mod shapes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Literal,
    Directory,
    Counts,
    RamWords,
    Rows,
}
impl Kind {
    fn role(self) -> &'static str {
        match self {
            Self::Literal => "audio/timed-sound-literal",
            Self::Directory => "audio/timed-sound-directory",
            Self::Counts => "audio/timed-sound-counts",
            Self::RamWords => "audio/timed-sound-ram-words",
            Self::Rows => "audio/timed-sound-rows",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "values", rename_all = "kebab-case")]
pub enum Values {
    Literal(u32),
    Directory(Vec<u32>),
    Counts(Vec<u8>),
    RamWords(Vec<u16>),
    Rows(Vec<TimedSoundRow>),
}
impl Values {
    fn kind(&self) -> Kind {
        match self {
            Self::Literal(_) => Kind::Literal,
            Self::Directory(_) => Kind::Directory,
            Self::Counts(_) => Kind::Counts,
            Self::RamWords(_) => Kind::RamWords,
            Self::Rows(_) => Kind::Rows,
        }
    }

    fn source_bytes(&self) -> Vec<u8> {
        match self {
            Self::Literal(word) => word.to_le_bytes().to_vec(),
            Self::Directory(words) => words.iter().flat_map(|word| word.to_le_bytes()).collect(),
            Self::Counts(bytes) => bytes.clone(),
            Self::RamWords(words) => words.iter().flat_map(|word| word.to_le_bytes()).collect(),
            Self::Rows(rows) => rows.iter().flat_map(TimedSoundRow::source_bytes).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub offset: u32,
    pub values: Values,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedSounds {
    pub sources: Vec<Source>,
}
impl TimedSounds {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let mut sources = Vec::with_capacity(shapes::SHAPES.len());
        let mut fingerprint = Vec::new();
        for &(offset, length, kind) in shapes::SHAPES {
            let data = &rom[offset as usize..offset as usize + length];
            fingerprint.extend(offset.to_le_bytes());
            fingerprint.extend((length as u32).to_le_bytes());
            fingerprint.push(match kind {
                Kind::Literal => 0,
                Kind::Directory => 1,
                Kind::Counts => 2,
                Kind::RamWords => 3,
                Kind::Rows => 4,
            });
            fingerprint.extend(data);
            let values = match kind {
                Kind::Literal => Values::Literal(u32::from_le_bytes(data.try_into().unwrap())),
                Kind::Directory => Values::Directory(
                    data.chunks_exact(4)
                        .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
                        .collect(),
                ),
                Kind::Counts => Values::Counts(data.to_vec()),
                Kind::RamWords => Values::RamWords(
                    data.chunks_exact(2)
                        .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
                        .collect(),
                ),
                Kind::Rows => Values::Rows(
                    data.chunks_exact(8)
                        .map(|row| TimedSoundRow {
                            tick: u16::from_le_bytes([row[0], row[1]]),
                            channel: row[2],
                            opaque03: row[3],
                            sound_words: [
                                u16::from_le_bytes([row[4], row[5]]),
                                u16::from_le_bytes([row[6], row[7]]),
                            ],
                        })
                        .collect(),
                ),
            };
            sources.push(Source { offset, values });
        }
        if ffta_rom::digest(&fingerprint)
            != "0de3d6f452cdc163063be3a9d164fc52013a69fa4b7305a2b1452acbb88a53da"
        {
            return Err("Timed sound source fingerprint mismatch".into());
        }
        let value = Self { sources };
        value.validate()?;
        Ok(value)
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        if self.sources.len() != shapes::SHAPES.len() {
            return Err("Invalid timed sound source count".into());
        }
        for (source, &(offset, length, kind)) in self.sources.iter().zip(shapes::SHAPES) {
            if source.offset != offset
                || source.values.kind() != kind
                || source.values.source_bytes().len() != length
            {
                return Err("Invalid timed sound source shape".into());
            }
        }
        Ok(())
    }

    pub(crate) fn source_spans(&self) -> Vec<(u32, Vec<u8>, &'static str)> {
        self.sources
            .iter()
            .map(|s| (s.offset, s.values.source_bytes(), s.values.kind().role()))
            .collect()
    }

    pub(crate) fn byte_at(&self, offset: u32) -> Option<u8> {
        let index = self
            .sources
            .partition_point(|s| s.offset <= offset)
            .checked_sub(1)?;
        let source = &self.sources[index];
        let relative = usize::try_from(offset - source.offset).ok()?;
        source.values.source_bytes().get(relative).copied()
    }
}
