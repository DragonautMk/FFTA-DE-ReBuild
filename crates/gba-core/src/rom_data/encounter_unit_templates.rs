//! Header-linked EUR unit templates. Live initialization and bit loops stay guest code.
use super::{
    constructor_alternates, encounter_headers, items, metadata14, metadata52, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x6ea048;
pub const SOURCE_END: u32 = 0x70b6a8;
pub const RECORD_COUNT: usize = 2850;
pub const RECORD_BYTES: usize = 48;
pub const SOURCE_BYTES: usize = RECORD_COUNT * RECORD_BYTES;
pub const OWNED_BYTES: usize = SOURCE_BYTES - 192;
pub const SOURCE_SHA256: &str = "15cc1b488912bf0609d6e2bc45cd3f8b28fcb5eb414b1aa3b32fac60199d636e";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Zero-based physical order, not a newly imposed guest index domain.
    pub index: u16,
    pub header_index: u16,
    pub unit_index: u16,
    pub source_offset: u32,
    pub byte00: u8,
    pub byte01: u8,
    pub byte02: u8,
    pub byte03: u8,
    pub byte04: u8,
    pub arguments05_07: [u8; 3],
    pub equipment08_10: [u16; 5],
    /// The original reads all sixteen bits, then stores the low byte.
    pub halfword12: u16,
    /// Physical field only. The original dynamic bit reader has no 160-bit cap.
    pub ability_bits14_27: [u8; 20],
    pub byte28: u8,
    pub byte29: u8,
    pub flags2a: u16,
    pub byte2c: u8,
    pub byte2d: u8,
    pub opaque2e_2f: [u8; 2],
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut raw = [0; RECORD_BYTES];
        raw[..5].copy_from_slice(&[
            self.byte00,
            self.byte01,
            self.byte02,
            self.byte03,
            self.byte04,
        ]);
        raw[5..8].copy_from_slice(&self.arguments05_07);
        for (i, value) in self.equipment08_10.iter().enumerate() {
            raw[8 + i * 2..10 + i * 2].copy_from_slice(&value.to_le_bytes());
        }
        raw[18..20].copy_from_slice(&self.halfword12.to_le_bytes());
        raw[20..40].copy_from_slice(&self.ability_bits14_27);
        raw[40] = self.byte28;
        raw[41] = self.byte29;
        raw[42..44].copy_from_slice(&self.flags2a.to_le_bytes());
        raw[44] = self.byte2c;
        raw[45] = self.byte2d;
        raw[46..48].copy_from_slice(&self.opaque2e_2f);
        raw
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterUnitTemplates {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl EncounterUnitTemplates {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.records.len() != RECORD_COUNT
            || self.records.iter().enumerate().any(|(i, r)| {
                usize::from(r.index) != i || r.source_offset != SOURCE + (i * RECORD_BYTES) as u32
            })
        {
            return Err("Invalid declared EUR encounter unit-template shape".into());
        }
        // No observed-value caps, enum/boolean canonicalization or opaque-zero policy.
        Ok(())
    }
    pub fn validate_headers(
        &self,
        owner: &encounter_headers::EncounterHeaders,
    ) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        let mut cursor = 0usize;
        for header in &owner.headers {
            let count = usize::from(header.unit_count);
            let end = cursor
                .checked_add(count)
                .filter(|&end| end <= self.records.len())
                .ok_or("Encounter unit-template group exceeds declared records")?;
            if header.unit_templates_pointer != 0x08000000 + SOURCE + (cursor * RECORD_BYTES) as u32
                || self.records[cursor..end]
                    .iter()
                    .enumerate()
                    .any(|(i, r)| r.header_index != header.index || usize::from(r.unit_index) != i)
            {
                return Err("Encounter unit templates differ from header partition".into());
            }
            cursor = end;
        }
        if cursor != self.records.len() {
            return Err("Encounter headers do not cover declared unit templates".into());
        }
        Ok(())
    }
    pub fn validate_constructor_alternates(
        &self,
        owner: &constructor_alternates::Alternates,
    ) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        for old in &owner.records {
            let index = ((old.source_offset - SOURCE) as usize) / RECORD_BYTES;
            if self.records[index].source_offset != old.source_offset
                || self.records[index].source_bytes() != old.source_bytes()
            {
                return Err(
                    "Encounter unit template differs from shared constructor record".into(),
                );
            }
        }
        Ok(())
    }
    pub fn validate_references(
        &self,
        item_owner: &items::Items,
        descriptor_owner: &metadata14::Metadata14,
        job_owner: &metadata52::Metadata52,
    ) -> Result<(), String> {
        self.validate()?;
        item_owner.validate()?;
        descriptor_owner.validate_metadata52(job_owner)?;
        for r in &self.records {
            if r.equipment08_10
                .iter()
                .any(|&id| usize::from(id) >= item_owner.records.len())
            {
                return Err("Encounter equipment has no parsed item owner".into());
            }
            let descriptor = descriptor_owner.record(u32::from(r.byte00));
            if r.byte00 != 1 && descriptor.is_none() {
                return Err("Encounter selector has no parsed descriptor owner".into());
            }
            let job = if r.byte00 != 1 && r.byte01 == 0 {
                descriptor.unwrap().selectors01_02[1]
            } else {
                r.byte01
            };
            if usize::from(job) >= job_owner.records.len() {
                return Err("Encounter job selector has no parsed job owner".into());
            }
        }
        // Descriptor1's later raw metadata-prefix read is not a new typed owner.
        Ok(())
    }
    /// Diagnostic physical lookup only; original guest indexing remains unchanged.
    pub fn record(&self, index: u32) -> Option<&Record> {
        self.records.get(index as usize)
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if (constructor_alternates::SOURCE_OFFSET..constructor_alternates::END_OFFSET)
            .contains(&offset)
        {
            return None;
        }
        let at = offset.checked_sub(self.source_offset)? as usize;
        Some(self.records.get(at / RECORD_BYTES)?.source_bytes()[at % RECORD_BYTES])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let raw: Vec<_> = self.records.iter().flat_map(Record::source_bytes).collect();
        let left = (constructor_alternates::SOURCE_OFFSET - SOURCE) as usize;
        let right = (constructor_alternates::END_OFFSET - SOURCE) as usize;
        Ok(vec![
            (
                SOURCE,
                raw[..left].to_vec(),
                "encounter-unit-template-fragment".into(),
            ),
            (
                constructor_alternates::END_OFFSET,
                raw[right..].to_vec(),
                "encounter-unit-template-fragment".into(),
            ),
        ])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Encounter unit templates differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: EncounterUnitTemplates,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Encounter unit templates require exact EUR ROM; USA pending".into());
        }
        for (start, length, sha) in [
            (
                0xcd0e6,
                400,
                "b0237eec84fea162c3ff609893643b87b4f787ae8c045b27c0f62d230493e5a0",
            ),
            (
                0xcd276,
                512,
                "99bc30c7d18e32e2117dafbc17989268baa651c66552a6773d6ec9a333632fa1",
            ),
            (
                0xcd476,
                128,
                "2655cff0c78227aadddc41c0d970e5fb37361f261ce88dbb9c3603c1079c9d70",
            ),
            (
                0xcd9e8,
                240,
                "782f782e28f74e6008e6dba0d0b720f1a1cf19259b016cce19fd592c95be6db4",
            ),
            (
                0xcdcb0,
                240,
                "f5480f5df35e273af887365feb5f96569987f79240530fa70db73f3b8f258622",
            ),
            (
                0xccf9c,
                220,
                "1ed6e29c6a7b2de9271c50392bfed2859b7aef88b028890b939adf4513ed9884",
            ),
            (
                0xcd048,
                160,
                "e7412d69df83d63d4652971370a40b86ddb1b662ef532003b703284ee691ddc0",
            ),
        ] {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Original encounter unit-template reader fingerprint mismatch".into());
            }
        }
        if ffta_rom::digest(&rom[SOURCE as usize..SOURCE_END as usize]) != SOURCE_SHA256 {
            return Err("Encounter unit-template source fingerprint mismatch".into());
        }
        let headers = encounter_headers::Catalog::from_rom(rom)?.decoded;
        let mut records = Vec::with_capacity(RECORD_COUNT);
        for header in &headers.headers {
            let start = (header.unit_templates_pointer - 0x08000000) as usize;
            for unit_index in 0..header.unit_count {
                let at = start + usize::from(unit_index) * RECORD_BYTES;
                let b = &rom[at..at + RECORD_BYTES];
                records.push(Record {
                    index: records.len() as u16,
                    header_index: header.index,
                    unit_index,
                    source_offset: at as u32,
                    byte00: b[0],
                    byte01: b[1],
                    byte02: b[2],
                    byte03: b[3],
                    byte04: b[4],
                    arguments05_07: b[5..8].try_into().unwrap(),
                    equipment08_10: std::array::from_fn(|i| {
                        u16::from_le_bytes(b[8 + i * 2..10 + i * 2].try_into().unwrap())
                    }),
                    halfword12: u16::from_le_bytes(b[18..20].try_into().unwrap()),
                    ability_bits14_27: b[20..40].try_into().unwrap(),
                    byte28: b[40],
                    byte29: b[41],
                    flags2a: u16::from_le_bytes(b[42..44].try_into().unwrap()),
                    byte2c: b[44],
                    byte2d: b[45],
                    opaque2e_2f: b[46..48].try_into().unwrap(),
                });
            }
        }
        let decoded = EncounterUnitTemplates {
            source_offset: SOURCE,
            records,
        };
        decoded.validate_headers(&headers)?;
        decoded.validate_constructor_alternates(
            &constructor_alternates::Catalog::from_rom(rom)?.decoded,
        )?;
        decoded.validate_references(
            &items::Catalog::from_rom(rom)?.decoded,
            &metadata14::Catalog::from_rom(rom)?.decoded,
            &metadata52::Catalog::from_rom(rom)?.decoded,
        )?;
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

#[cfg(test)]
mod tests;
