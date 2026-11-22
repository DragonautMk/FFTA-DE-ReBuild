//! Authenticated EUR Soil Evidence range and sequence records.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const RANGE_OFFSET: u32 = 0x39e124;
pub const SEQUENCE_OFFSET: u32 = 0x39e138;
pub const RANGE_COUNT: usize = 5;
pub const SEQUENCE_COUNT: usize = 11;
pub const RECORD_BYTES: usize = 4;
const RANGE_SHA256: &str = "cd3ef394f6fbdf1fa2c138238e0714a1760476f6ced911c0f54db3c1761f0ed5";
const SEQUENCE_SHA256: &str = "9456532a831994532a04c482f8855c6ee9108ddd4023d7a20b91112271bf9d13";

/// The second pair of bytes is retained without assigning it unproved meaning.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangeRecord {
    pub field0: u8,
    pub field1: u8,
    pub opaque: [u8; 2],
}
impl RangeRecord {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        [self.field0, self.field1, self.opaque[0], self.opaque[1]]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceRecord {
    pub range_index: u8,
    pub field1: u8,
    pub field2: u8,
    pub field3: u8,
}
impl SequenceRecord {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        [self.range_index, self.field1, self.field2, self.field3]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilSceneRecords {
    pub range_offset: u32,
    pub ranges: [RangeRecord; RANGE_COUNT],
    pub sequence_offset: u32,
    pub sequence: [SequenceRecord; SEQUENCE_COUNT],
}
impl SoilSceneRecords {
    pub fn validate(&self) -> Result<(), String> {
        if self.range_offset != RANGE_OFFSET
            || self.sequence_offset != SEQUENCE_OFFSET
            || self
                .sequence
                .iter()
                .any(|row| usize::from(row.range_index) >= RANGE_COUNT)
        {
            return Err("Invalid Soil scene record source or range reference".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![
            (
                self.range_offset,
                self.ranges
                    .iter()
                    .flat_map(RangeRecord::source_bytes)
                    .collect(),
                "soil-range-records",
            ),
            (
                self.sequence_offset,
                self.sequence
                    .iter()
                    .flat_map(SequenceRecord::source_bytes)
                    .collect(),
                "soil-sequence-records",
            ),
        ])
    }

    /// This describes owned physical bytes, not a restriction on guest indices.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, usize)> {
        if let Some(at) = offset.checked_sub(self.range_offset) {
            let at = at as usize;
            if let Some(row) = self.ranges.get(at / RECORD_BYTES) {
                return Some((row.source_bytes()[at % RECORD_BYTES], 0, at / RECORD_BYTES));
            }
        }
        if let Some(at) = offset.checked_sub(self.sequence_offset) {
            let at = at as usize;
            if let Some(row) = self.sequence.get(at / RECORD_BYTES) {
                return Some((row.source_bytes()[at % RECORD_BYTES], 1, at / RECORD_BYTES));
            }
        }
        None
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Soil scene records differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: SoilSceneRecords,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Soil scene records require the exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0xf0568,
                0xf05f0,
                "9e450c749aabceb0d4db09a68346474ce6c92db8c25d755ca358630260c5e032",
            ),
            (
                0xf063a,
                0xf06ac,
                "a04ed6fc911b3d2b493c5d1f73f3fb89d1c9162af72c51b074400875ab14cb0a",
            ),
            (
                0xf06b8,
                0xf06d2,
                "99fc20b0f50c4f1dade52a6ba6892ddcd29d300b442902036bb2c1458eb8001b",
            ),
            (
                0xf086e,
                0xf0886,
                "b43f25766f6ed2bce3e8a5e0975a856757e2c40dd401615c37ac49f842024828",
            ),
            (
                0xf0a6a,
                0xf0a8c,
                "e2e2a22a1b69fcbe9d237a4f4a66c7ba4176cdb4795b650aea846cefb3a67280",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Soil scene reader fingerprint mismatch".into());
            }
        }
        let roots = [
            (0xf0578u32, 0xf05f0u32, SEQUENCE_OFFSET),
            (0xf0584, 0xf05f4, RANGE_OFFSET),
            (0xf063a, 0xf06ac, RANGE_OFFSET),
            (0xf064a, 0xf06b0, SEQUENCE_OFFSET),
            (0xf06b8, 0xf07c8, RANGE_OFFSET),
            (0xf06c8, 0xf07cc, SEQUENCE_OFFSET),
            (0xf0a6e, 0xf0aa0, SEQUENCE_OFFSET),
            (
                0xf086e,
                0xf0898,
                SEQUENCE_OFFSET + (SEQUENCE_COUNT * RECORD_BYTES) as u32,
            ),
        ];
        let literals = previous
            .iter()
            .find(|resource| resource.id == THUMB_PC_LITERALS_ID);
        if !previous.is_empty() && literals.is_none() {
            return Err("Missing Soil scene literal dependency".into());
        }
        for (reader, literal, target) in roots {
            let at = reader as usize;
            let op = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            let root = u32::from_le_bytes(
                rom[literal as usize..literal as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            if op & 0xf800 != 0x4800
                || ((reader + 4) & !3) + u32::from(op & 255) * 4 != literal
                || root != 0x0800_0000 + target
            {
                return Err("Soil scene reader literal mismatch".into());
            }
            if literals.is_some_and(|owner| {
                !owner
                    .sources
                    .iter()
                    .any(|source| source.offset == literal && source.length == 4)
            }) {
                return Err("Missing separately owned Soil scene literal".into());
            }
        }
        let ranges = std::array::from_fn(|i| {
            let at = RANGE_OFFSET as usize + i * RECORD_BYTES;
            RangeRecord {
                field0: rom[at],
                field1: rom[at + 1],
                opaque: [rom[at + 2], rom[at + 3]],
            }
        });
        let sequence = std::array::from_fn(|i| {
            let at = SEQUENCE_OFFSET as usize + i * RECORD_BYTES;
            SequenceRecord {
                range_index: rom[at],
                field1: rom[at + 1],
                field2: rom[at + 2],
                field3: rom[at + 3],
            }
        });
        let decoded = SoilSceneRecords {
            range_offset: RANGE_OFFSET,
            ranges,
            sequence_offset: SEQUENCE_OFFSET,
            sequence,
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Soil scene record source overlap or reconstruction mismatch".into());
            }
            let expected = if offset == RANGE_OFFSET {
                RANGE_SHA256
            } else {
                SEQUENCE_SHA256
            };
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != expected {
                return Err("Soil scene record source hash mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
