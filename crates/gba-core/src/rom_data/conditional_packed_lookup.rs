//! Source-authenticated type-5 EUR lookup projection; original guest selectors remain live.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_ROOT: u32 = 0x6e6ee4;
pub const FIRST_INDEX: u32 = 1;
pub const SOURCE_OFFSET: u32 = TABLE_ROOT + FIRST_INDEX * 4;
pub const RECORD_COUNT: usize = 33;
pub const SOURCE_SHA256: &str = "5f8c27c41576800b3eded5d5c7e57c7ceb10f5a1ca9e31d8c07fb72c19cfcae9";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub byte0: u8,
    pub low5: u8,
    pub high3: u8,
    pub low2: u8,
    pub high6: u8,
    pub byte3: u8,
}
impl Record {
    pub fn bytes(&self) -> [u8; 4] {
        [self.byte0, self.low5 | (self.high3 << 5), self.low2 | (self.high6 << 2), self.byte3]
    }
    pub fn combined(&self) -> u8 { self.high3 | (self.low2 << 3) }
    fn valid(&self) -> bool { self.low5 < 32 && self.high3 < 8 && self.low2 < 4 && self.high6 < 64 }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lookup { pub source_offset: u32, pub first_index: u32, pub records: Vec<Record> }
impl Lookup {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.first_index != FIRST_INDEX || self.records.len() != RECORD_COUNT || !self.records.iter().all(Record::valid) {
            return Err("Invalid conditional EUR packed lookup projection".into());
        }
        Ok(())
    }
    pub fn select(&self, index: u32) -> Option<&Record> { self.records.get(index.checked_sub(self.first_index)? as usize) }
    pub fn encoded_view(index: u32) -> u32 { (0x08000000u32 + TABLE_ROOT).wrapping_add(index.wrapping_mul(4)) }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> { self.validate()?; Ok(self.records.iter().flat_map(Record::bytes).collect()) }
    pub fn byte_at(&self, offset: u32) -> Option<u8> { let n = offset.checked_sub(self.source_offset)? as usize; Some(self.records.get(n / 4)?.bytes()[n % 4]) }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded { return Err("Conditional packed lookup differs from authenticated EUR source".into()); }
        Ok(())
    }
}
pub struct Catalog { pub decoded: Lookup, pub sources: Vec<SourceRange> }
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 { return Err("Conditional packed lookup requires exact EUR ROM; USA pending".into()); }
        let source = &rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + RECORD_COUNT * 4];
        if ffta_rom::digest(source) != SOURCE_SHA256 { return Err("Conditional packed lookup source fingerprint mismatch".into()); }
        let decoded = Lookup { source_offset: SOURCE_OFFSET, first_index: FIRST_INDEX, records: source.chunks_exact(4).map(|v| Record { byte0:v[0], low5:v[1]&31, high3:v[1]>>5, low2:v[2]&3, high6:v[2]>>2, byte3:v[3] }).collect() };
        decoded.validate()?;
        Ok(Self { decoded, sources:vec![SourceRange { offset:SOURCE_OFFSET, length:source.len() as u32, sha256:SOURCE_SHA256.into(), role:"conditional-packed-lookup".into() }] })
    }
}
