//! Conditional twenty-slot job references; original live selection stays guest work.
use super::{metadata52, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x399750;
pub const SLOT_COUNT: usize = 20;
pub const SOURCE_SHA256: &str = "969b2cd56b835c15f5bec9d962d9f007a8746ea192857bf0ac25527a44019a38";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub slot: u8,
    pub job_id: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotJobLookup {
    pub source_offset: u32,
    pub entries: Vec<Entry>,
}
impl SlotJobLookup {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET
            || self.entries.len() != SLOT_COUNT
            || self.entries.iter().enumerate().any(|(slot, e)| {
                usize::from(e.slot) != slot || usize::from(e.job_id) >= metadata52::RECORD_COUNT
            })
        {
            return Err("Invalid fixed EUR slot-job source projection".into());
        }
        Ok(())
    }
    pub fn validate_dependencies(&self, jobs: &metadata52::Metadata52) -> Result<(), String> {
        self.validate()?;
        jobs.validate()?;
        // These selected source references use direct rows. A live getter's
        // other inputs retain their original redirects and raw memory path.
        if self
            .entries
            .iter()
            .any(|e| jobs.records[usize::from(e.job_id)].routing != 0)
        {
            return Err("Slot-job reference differs from the direct metadata52 domain".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            self.source_offset,
            self.entries.iter().map(|e| e.job_id).collect(),
            "slot-job-ids".into(),
        )])
    }
    /// Only the declared source projection; this does not cap guest inputs.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let slot = offset.checked_sub(self.source_offset)? as usize;
        self.entries.get(slot).map(|e| e.job_id)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Slot-job lookup differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: SlotJobLookup,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Slot-job lookup requires exact EUR ROM; USA pending".into());
        }
        for (at, len, expected) in [
            (
                0xd0538,
                176,
                "feb166ab29f3a36997bf2aba7951f4ad37576002ed0e52a978271583c2bb3882",
            ),
            (
                0x12ca98,
                224,
                "ab60b4b9ce119d747142562edc300f01f0fd69e6398b2213e1e041797f8761d1",
            ),
            (
                0xcff70,
                136,
                "ddd60dc2ad22466dc64ae79fc25bdc48b045b204c07d7279082112191c110f2d",
            ),
        ] {
            if ffta_rom::digest(&rom[at..at + len]) != expected {
                return Err("Original slot-job producer/reader fingerprint mismatch".into());
            }
        }
        let raw = &rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + SLOT_COUNT];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Slot-job source hash mismatch".into());
        }
        let decoded = SlotJobLookup {
            source_offset: SOURCE_OFFSET,
            entries: raw
                .iter()
                .enumerate()
                .map(|(slot, &job_id)| Entry {
                    slot: slot as u8,
                    job_id,
                })
                .collect(),
        };
        decoded.validate_dependencies(&metadata52::Catalog::from_rom(rom)?.decoded)?;
        let sources = decoded
            .source_spans()?
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
