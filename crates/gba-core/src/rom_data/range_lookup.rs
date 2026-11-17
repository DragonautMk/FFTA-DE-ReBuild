//! EUR bounded range table; original selection and output writes stay guest-side.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x37140c;
pub const RECORD_COUNT: usize = 11;
pub const SOURCE_SHA256: &str = "4cae38f2a735087e740409ca58e5f19bce1614805347c472990b8419e493bb7d";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub selector: u8,
    pub opaque_01: u8,
    pub lower: u16,
    pub upper: u16,
    pub opaque_tail: [u8; 2],
}
impl Record {
    pub fn source_bytes(&self) -> [u8; 8] {
        let lower = self.lower.to_le_bytes();
        let upper = self.upper.to_le_bytes();
        [self.selector, self.opaque_01, lower[0], lower[1], upper[0], upper[1], self.opaque_tail[0], self.opaque_tail[1]]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangeLookup {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl RangeLookup {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid EUR range lookup projection".into());
        }
        Ok(())
    }
    /// First inclusive match; no guest-side selector or index guard is added.
    pub fn select(&self, value: u16) -> Option<(u8, u16)> {
        self.records.iter().find_map(|r| {
            (r.lower <= value && value <= r.upper).then(|| (r.selector, value - r.lower))
        })
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(Record::source_bytes).collect())
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = usize::try_from(offset.checked_sub(self.source_offset)?).ok()?;
        Some(self.records.get(relative / 8)?.source_bytes()[relative % 8])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Range lookup differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: RangeLookup,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Range lookup requires the exact EUR ROM".into());
        }
        for (start, end, digest) in [
            (0x1abc0, 0x1abfc, "606c326ff038d2f2b25578b111df54c2152dc1f5517711579585cac3f704edd7"),
            (0x1abfc, 0x1acac, "6222c23857a045ad321e4fa8ea44e382a2772cf26d780e2228b43a5f8ff6d078"),
            (0x1c7ba, 0x1c834, "cc2b2ab8bb13164f5d73866b90bddbe5a826fcf00774c6e0838666f4f3efc6cc"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != digest {
                return Err("Range lookup reader fingerprint mismatch".into());
            }
        }
        let raw = &rom[SOURCE as usize..SOURCE as usize + RECORD_COUNT * 8];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Range lookup source fingerprint mismatch".into());
        }
        let records = raw.chunks_exact(8).map(|r| Record {
            selector: r[0], opaque_01: r[1], lower: u16::from_le_bytes([r[2], r[3]]),
            upper: u16::from_le_bytes([r[4], r[5]]), opaque_tail: [r[6], r[7]],
        }).collect();
        let decoded = RangeLookup { source_offset: SOURCE, records };
        decoded.validate()?;
        Ok(Self { decoded, sources: vec![SourceRange {
            offset: SOURCE, length: (RECORD_COUNT * 8) as u32,
            sha256: SOURCE_SHA256.into(), role: "bounded-range-lookup".into(),
        }] })
    }
}
