//! EUR indexed words, authenticated against their original readers and source.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3959a4;
pub const END_OFFSET: u32 = 0x395a1c;
pub const RECORD_BYTES: usize = 4;
pub const RECORD_COUNT: usize = 30;
pub const SOURCE_SHA256: &str = "a8e6a30a99f122641fc7d83e75f964fb7e323a6eaf8305c342aaf4bc99e18c9c";

/// Physical word and position; the guest supplies the index and interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub value: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedWords3959 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}

impl IndexedWords3959 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT
            || self.records.iter().enumerate().any(|(i, r)| usize::from(r.id) != i
                || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32) {
            return Err("Invalid EUR indexed word source, count or order".into());
        }
        Ok(())
    }

    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(|r| r.value.to_le_bytes()).collect())
    }

    /// Returns one physical byte without restricting original guest indexing.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, usize)> {
        let n = offset.checked_sub(self.source_offset)? as usize;
        let (record, field) = (n / RECORD_BYTES, n % RECORD_BYTES);
        Some((*self.records.get(record)?.value.to_le_bytes().get(field)?, record, field))
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Indexed words differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: IndexedWords3959,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Indexed words require exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (0x36e04, 0x36e0c, "182785ab2ae7fc0d29d5c0243d64ccab9cd6a245d67c03ca16fb7354aea90872"),
            (0x385ec, 0x385f4, "8afb73167791ad43ccc840ebdaf9bd087e70174150ea316377982eddde007801"),
            (0x3887c, 0x38886, "7798130e47df07f5c7cc4a6779071a52c6c33246977b708f59c2a836c59a242a"),
            (0x3a5d4, 0x3a5e2, "f93fa695771cdbd576519e69f20089264b846aff75b54ce36b01740366a1a811"),
            (0x3c4a4, 0x3c4b0, "1afedec1fe3b17b0d7ba01adf1af743501f163fc1ba27b8d464ba6f020d03b33"),
            (0x3e8dc, 0x3e8ea, "c699901bc14f17aa688aed262fa6d5c4fa910de41d58258ae7da028f1cb20fd4"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Indexed word reader or successor fingerprint mismatch".into());
            }
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (reader, literal, target) in [
            (0x36e04, 0x36e44, SOURCE_OFFSET),
            (0x385ec, 0x38620, SOURCE_OFFSET),
            (0x3887c, 0x388b8, SOURCE_OFFSET),
            (0x3a5d6, 0x3a5e4, SOURCE_OFFSET),
            (0x3c4a8, 0x3c5b8, SOURCE_OFFSET),
            (0x3e8de, 0x3ea10, END_OFFSET),
        ] {
            let opcode = half(reader);
            if opcode & 0xf800 != 0x4800
                || ((reader + 4) & !3) + usize::from(opcode & 255) * 4 != literal
                || word(literal) != 0x0800_0000 + target {
                return Err("Indexed word root or successor literal mismatch".into());
            }
        }
        let raw = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        if raw.len() != RECORD_COUNT * RECORD_BYTES || ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Indexed word source fingerprint mismatch".into());
        }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, raw.len())? {
            return Err("Indexed words already have an owner".into());
        }
        let records = raw.chunks_exact(RECORD_BYTES).enumerate().map(|(id, b)| Record {
            id: id as u8, source_offset: SOURCE_OFFSET + (id * RECORD_BYTES) as u32,
            value: u32::from_le_bytes(b.try_into().unwrap()),
        }).collect();
        let decoded = IndexedWords3959 { source_offset: SOURCE_OFFSET, records };
        if decoded.source_bytes()? != raw { return Err("Indexed word reconstruction mismatch".into()); }
        Ok(Self { decoded, sources: vec![SourceRange { offset: SOURCE_OFFSET,
            length: raw.len() as u32, sha256: SOURCE_SHA256.into(), role: "indexed-words-3959a4".into() }] })
    }
}
