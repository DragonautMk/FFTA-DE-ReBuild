//! EUR indexed words, authenticated against their original readers and source.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3959a4;
pub const END_OFFSET: u32 = 0x395a1c;
pub const RECORD_BYTES: usize = 4;
pub const RECORD_COUNT: usize = 30;
pub const SOURCE_SHA256: &str = "a8e6a30a99f122641fc7d83e75f964fb7e323a6eaf8305c342aaf4bc99e18c9c";

/// Sparse words witnessed in Soil Evidence; these indices are not a guest bound.
pub const SOIL_E3838_BASE: u32 = 0x14d580;
pub const SOIL_E3838_COUNT: usize = 30;
pub const SOIL_E3838_WORDS: [(u8, &str); SOIL_E3838_COUNT] = [
    (1, "4af7e59059dd1e1f641163f29552f79a0b2f830efb017c3101b07e5e38e586a9"),
    (2, "b2e2f6690fd4164a0cb08d785ade2a40beb6a5756652ff7af1f60a6916d52aba"),
    (3, "f9b955e5624ac6bf4c26e548a4fc1d84304212e60c39efafb94d798d7dcee3c8"),
    (4, "2982aeb0167baf05bb708635fc697abdb363468b81861738429cef2aa4ad9593"),
    (5, "74ffa268f37bd5c7fe3d13b2210bb314718d353e30993e2d4178af9bfc07d226"),
    (6, "0a52d3a52a955e6a5ea32553a2be22575d1aea95692d334e3753a71312dba765"),
    (7, "36a07f73ac644efc017814ec5368b0d2b49b048725d1e5c6a1fd26a5153725bc"),
    (9, "0239e3dcd56c36df29d30d1a97cc56b54733f815d7657d5a94fafbce11475440"),
    (10, "c3c0d64b65d760280d6f6e1572adfd0a19137e653a6f0d972643771093b7ecb0"),
    (11, "0335fca603e815b18b2e43d4e5267a9051989ece0f725715d8e8d937925baf15"),
    (12, "4061c5c0c114c5cc120a64c9ba36ac9ab1512d93d10da1d484505de32ecb8cfd"),
    (13, "458d355534742d9d2bf6a50bcf4bc37756c62141f9ddb34757a26068d892dfdd"),
    (14, "f9cfccf5b8380a88c6cd8b722ab1f40e638109e8cf3e465d2810191a61bc8758"),
    (15, "9ecfb7e5c6cf13ccefe50276e39bff2862305c6fe2bfda080dd72b7c70fe668e"),
    (16, "9a8208635e00348ab64aac2b759e76391fd47089e9a749bbcec770d9eb5c6421"),
    (17, "227fe77efddf4221e2cf9c0492257009e24b93fea24305f5ec4cce06fb0f4783"),
    (18, "c5c9318125585cd2e1a2919673e60cdc9f37c9f7a3b0d078054917787afe2c9e"),
    (19, "38e4449143d1e9202e1425a0c4d317fc15a0fc20d8b5a55d070a081e0726299a"),
    (20, "06ebd70f712594b2c876556943aaa43db9d26e9c3f4fa8fcfe39e589eae9940c"),
    (21, "1fc7ee087fa2b973f9d406348e7c155155fa40e24804c9d2f855172ef62339b2"),
    (22, "2cc2bc2496d04276b2152d2fb371bf1f73c0a81605a64e43090bea213bcd3b51"),
    (23, "5e839e9a92e2e6b68398d6e1033698f66eea081efd3133d19f7f6c0ea89146f3"),
    (25, "18c0dc151ddeb4b55e581821d2d03ab729cc90ecdbf5da5b410d16dc985a44ba"),
    (26, "0191a3fc25813e12c535b35673eb917c991d9d323b936848ab4e4bb196b5efcd"),
    (27, "891b3d45e41776de801cbd7eaa37012ff111e3f274c347eb4c82546da5eb7fb8"),
    (28, "dc7cb9d4f0ba4d498d5812eb7fc73b9e728890242cb88b30d7c27fd1bf22088b"),
    (29, "e54db2060524058b22ab7a32a620a296ed313655c8d601123520a6bf336f9f60"),
    (30, "7b700846ddcf5b8e2362a95844e049bd554410f2262b501970c4642df4a31cd0"),
    (31, "ef661b0bb5cd50358be9595077573d083f490dcf7cfa300451aeb192fe3612a2"),
    (32, "d99e58435243d9fef9c88273b8d553b4fba4d0baf8009d29eae74fa99e0d9f57"),
];

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
    /// Selected sources only; all other indices retain original raw behavior.
    pub soil_e3838_records: Vec<Record>,
}

impl IndexedWords3959 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT
            || self.records.iter().enumerate().any(|(i, r)| usize::from(r.id) != i
                || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32) {
            return Err("Invalid EUR indexed word source, count or order".into());
        }
        if self.soil_e3838_records.len() != SOIL_E3838_COUNT
            || self.soil_e3838_records.iter().zip(SOIL_E3838_WORDS).any(|(r, (id, _))|
                r.id != id || r.source_offset != SOIL_E3838_BASE + u32::from(id) * 4)
        {
            return Err("Invalid EUR Soil Evidence indexed source or order".into());
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

    /// Returns bytes only from the witnessed sparse words, without indexing guards.
    pub fn soil_e3838_byte_at(&self, offset: u32) -> Option<(u8, usize, usize)> {
        let base = offset & !3;
        let record = self.soil_e3838_records.binary_search_by_key(&base, |r| r.source_offset).ok()?;
        let field = (offset & 3) as usize;
        Some((self.soil_e3838_records[record].value.to_le_bytes()[field], record, field))
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
        if ffta_rom::digest(&rom[0xe382e..0xe383c]) != "0d2d869dd0735940b2515a57b53a6c286c4dca86efc32b06b3416771a3389e69"
            || ffta_rom::digest(&rom[0xe3854..0xe3858]) != "254a6dc3f97dbcb8cacb53954e4638cee9bcb3537ab804569b35547bc94dea70"
            || word(0xe3854) != 0x0800_0000 + SOIL_E3838_BASE
        { return Err("Soil Evidence indexed reader or root fingerprint mismatch".into()); }
        let prior = PriorSources::new(previous)?;
        let mut soil_e3838_records = Vec::with_capacity(SOIL_E3838_COUNT);
        let mut sources = vec![SourceRange { offset: SOURCE_OFFSET,
            length: raw.len() as u32, sha256: SOURCE_SHA256.into(), role: "indexed-words-3959a4".into() }];
        for (id, hash) in SOIL_E3838_WORDS {
            let offset = SOIL_E3838_BASE + u32::from(id) * 4;
            let bytes = &rom[offset as usize..offset as usize + 4];
            if ffta_rom::digest(bytes) != hash || prior.contains(offset, bytes.len())? {
                return Err("Soil Evidence indexed source mismatch or already owned".into());
            }
            soil_e3838_records.push(Record { id, source_offset: offset,
                value: u32::from_le_bytes(bytes.try_into().unwrap()) });
            sources.push(SourceRange { offset, length: 4, sha256: hash.into(),
                role: "soil-e3838-observed-word".into() });
        }
        let decoded = IndexedWords3959 { source_offset: SOURCE_OFFSET, records, soil_e3838_records };
        if decoded.source_bytes()? != raw { return Err("Indexed word reconstruction mismatch".into()); }
        Ok(Self { decoded, sources })
    }
}
