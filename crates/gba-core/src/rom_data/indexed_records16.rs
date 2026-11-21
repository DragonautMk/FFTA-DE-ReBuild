//! Source-authenticated EUR indexed records; the guest retains index semantics.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e8aac;
pub const END_OFFSET: u32 = 0x6e900c;
pub const RECORD_BYTES: usize = 16;
pub const RECORD_COUNT: usize = 86;
pub const SOURCE_SHA256: &str = "e3dd487124f34c1dd57222ce76d62017abb2e29b7023c8dc865c45429b4ce2f7";

/// Physical fields only: their broader meanings remain guest-owned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub field_00: u8,
    pub field_01: u8,
    pub flags_02_03: u16,
    pub field_04: u8,
    pub slots_05_0b: [u8; 7],
    pub field_0c: u8,
    pub field_0d: u8,
    pub field_0e: u8,
    pub field_0f: u8,
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut bytes = [0; RECORD_BYTES];
        bytes[0] = self.field_00;
        bytes[1] = self.field_01;
        bytes[2..4].copy_from_slice(&self.flags_02_03.to_le_bytes());
        bytes[4] = self.field_04;
        bytes[5..12].copy_from_slice(&self.slots_05_0b);
        bytes[12..16].copy_from_slice(&[self.field_0c, self.field_0d, self.field_0e, self.field_0f]);
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedRecords16 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl IndexedRecords16 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT
            || self.records.iter().enumerate().any(|(i, r)| usize::from(r.id) != i
                || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32) {
            return Err("Invalid EUR indexed-record source, count or order".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(Record::source_bytes).collect())
    }
    /// Returns a physical field only; it does not constrain guest indexing.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, usize)> {
        let n = offset.checked_sub(self.source_offset)? as usize;
        let (record, field) = (n / RECORD_BYTES, n % RECORD_BYTES);
        Some((*self.records.get(record)?.source_bytes().get(field)?, record, field))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Indexed records differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: IndexedRecords16,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Indexed records require exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (0xcfac0, 0xcfad4, "8e541c635b8a6c03d1e17eb0eb1564e801f018d36510217d9bcdcb35adaa00f5"),
            (0x2efd0, 0x2efda, "5beb35b03feb27f439744178fb7d544b65629132b596ab5d8346d0712d073627"),
            (0xd5d38, 0xd5d68, "3d0794eead108e3d6ddd59ad5ca8d2caacbecc292bbde3dde48dd226d1b4e3af"),
            (0x1385b8, 0x1385c0, "6e0383c76864452c8230ff4fd4f98a40a3245e1e4a4fc7bbcd00f04319a45599"),
            (0xd3a80, 0xd3a9c, "c8b7425eada1e43e1ce1a0349ca6dddf72c9047e0c1e022a7ce621fe3938b471"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Indexed record reader or boundary fingerprint mismatch".into());
            }
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (reader, literal, target) in [
            (0xcfac8, 0xcfb00, SOURCE_OFFSET),
            (0x2efd4, 0x2eff4, SOURCE_OFFSET),
            (0x2f52e, 0x2f564, SOURCE_OFFSET),
            (0x1385b8, 0x1385d0, SOURCE_OFFSET + 82 * 16),
            (0xd3a8e, 0xd3aa4, END_OFFSET),
        ] {
            let opcode = half(reader);
            if opcode & 0xf800 != 0x4800
                || ((reader + 4) & !3) + usize::from(opcode & 255) * 4 != literal
                || word(literal) != 0x0800_0000 + target {
                return Err("Indexed record root or alias literal mismatch".into());
            }
        }
        if half(0xcfac6) != 0x0111 || half(0xcfacc) != 0x2a55 || half(0xcfad0) != 0x8841
            || half(0x2f52c) != 0x0120 || half(0x2f532) != 0x7844
            || half(0xd5d4e) != 0x0110 || half(0xd5d50) != 0x4460 || half(0xd5d52) != 0x8841 {
            return Err("Indexed record stride, conditional count or flags read mismatch".into());
        }
        let raw = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        if ffta_rom::digest(raw) != SOURCE_SHA256 || raw.len() != RECORD_COUNT * RECORD_BYTES {
            return Err("Indexed record source fingerprint mismatch".into());
        }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, raw.len())? {
            return Err("Indexed records already have an owner".into());
        }
        let records = raw.chunks_exact(RECORD_BYTES).enumerate().map(|(id, b)| Record {
            id: id as u8, source_offset: SOURCE_OFFSET + (id * RECORD_BYTES) as u32,
            field_00: b[0], field_01: b[1], flags_02_03: u16::from_le_bytes([b[2], b[3]]),
            field_04: b[4], slots_05_0b: b[5..12].try_into().unwrap(),
            field_0c: b[12], field_0d: b[13], field_0e: b[14], field_0f: b[15],
        }).collect();
        let decoded = IndexedRecords16 { source_offset: SOURCE_OFFSET, records };
        if decoded.source_bytes()? != raw { return Err("Indexed record reconstruction mismatch".into()); }
        Ok(Self { decoded, sources: vec![SourceRange { offset: SOURCE_OFFSET,
            length: raw.len() as u32, sha256: SOURCE_SHA256.into(), role: "indexed-records16".into() }] })
    }
}
