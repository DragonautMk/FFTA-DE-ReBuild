//! EUR indexed records. Physical ownership does not impose a guest index limit.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x710e04;
pub const SOURCE_END: u32 = 0x7133f8;
pub const RECORD_BYTES: usize = 28;
pub const RECORD_COUNT: usize = 347;
pub const SOURCE_BYTES: usize = RECORD_BYTES * RECORD_COUNT;
pub const SOURCE_SHA256: &str = "68cea1f29a7c4a6a50e2184538d2df954c00d792f84eb59405643221410fa192";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u16,
    pub field00: u16,
    pub field02: u8,
    pub opaque03: u8,
    pub fields04_0b: [u8; 8],
    /// Original selector9 returns a view here; inspected consumers read its first3 bytes.
    pub inline0c_0f: [u8; 4],
    pub flags10: u32,
    pub field14: u16,
    pub field16: u16,
    pub field18: u8,
    pub field19: u8,
    pub field1a: u8,
    pub opaque1b: u8,
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut raw = [0; RECORD_BYTES];
        raw[..2].copy_from_slice(&self.field00.to_le_bytes());
        raw[2] = self.field02;
        raw[3] = self.opaque03;
        raw[4..12].copy_from_slice(&self.fields04_0b);
        raw[12..16].copy_from_slice(&self.inline0c_0f);
        raw[16..20].copy_from_slice(&self.flags10.to_le_bytes());
        raw[20..22].copy_from_slice(&self.field14.to_le_bytes());
        raw[22..24].copy_from_slice(&self.field16.to_le_bytes());
        raw[24..].copy_from_slice(&[self.field18, self.field19, self.field1a, self.opaque1b]);
        raw
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.source_bytes().get(offset).copied()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lookup {
    Scalar(u32),
    /// Encoded guest address; this is not a host pointer or an ownership claim.
    SourceAddress(u32),
}
impl Lookup {
    pub fn value(self) -> u32 {
        match self {
            Self::Scalar(v) | Self::SourceAddress(v) => v,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata28 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Metadata28 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid declared EUR metadata28 source/count".into());
        }
        if self
            .records
            .iter()
            .enumerate()
            .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid metadata28 physical record order/index".into());
        }
        Ok(())
    }
    /// None identifies an unowned record, never a guest null/zero fallback.
    pub fn record(&self, index: u32) -> Option<&Record> {
        self.records.get(index as u16 as usize)
    }
    /// Project the original low16-index/low8-selector getter without reading raw ROM.
    /// Address/default selectors require no record-data read, even outside ownership.
    pub fn lookup(&self, index: u32, selector: u32) -> Option<Lookup> {
        let selector = selector as u8;
        if selector >= 34 {
            return Some(Lookup::Scalar(0));
        }
        if selector == 9 {
            return Some(Lookup::SourceAddress(
                0x08000000 + SOURCE + u32::from(index as u16) * RECORD_BYTES as u32 + 12,
            ));
        }
        let r = self.record(index)?;
        Some(Lookup::Scalar(match selector {
            0 => u32::from(r.field00),
            1 => u32::from(r.field02),
            2..=8 => u32::from(r.fields04_0b[usize::from(selector - 2)]),
            10 => r.flags10,
            11..=31 => (r.flags10 >> (selector - 11)) & 1,
            32 => u32::from(r.field14),
            33 => u32::from(r.fields04_0b[7]),
            _ => unreachable!(),
        }))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if !(SOURCE..SOURCE_END).contains(&offset) {
            return None;
        }
        let relative = (offset - SOURCE) as usize;
        self.records
            .get(relative / RECORD_BYTES)?
            .byte(relative % RECORD_BYTES)
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE,
            self.source_bytes(),
            "linked-metadata28-records".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata28 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: Metadata28,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata28 requires exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[SOURCE as usize..SOURCE_END as usize]) != SOURCE_SHA256 {
            return Err("Metadata28 source fingerprint mismatch".into());
        }
        for &(start, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Metadata28 original reader/boundary fingerprint mismatch".into());
            }
        }
        let raw = &rom[SOURCE as usize..SOURCE_END as usize];
        let decoded = Metadata28 {
            source_offset: SOURCE,
            records: raw
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, r)| Record {
                    index: i as u16,
                    field00: u16::from_le_bytes(r[..2].try_into().unwrap()),
                    field02: r[2],
                    opaque03: r[3],
                    fields04_0b: r[4..12].try_into().unwrap(),
                    inline0c_0f: r[12..16].try_into().unwrap(),
                    flags10: u32::from_le_bytes(r[16..20].try_into().unwrap()),
                    field14: u16::from_le_bytes(r[20..22].try_into().unwrap()),
                    field16: u16::from_le_bytes(r[22..24].try_into().unwrap()),
                    field18: r[24],
                    field19: r[25],
                    field1a: r[26],
                    opaque1b: r[27],
                })
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE,
                length: SOURCE_BYTES as u32,
                sha256: ffta_rom::digest(raw),
                role: "linked-metadata28-records".into(),
            }],
        })
    }
}

// Authenticated source-contract windows, including independent successor type witnesses.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x24506,
        160,
        "0a2ee6a785d8bc14c25d4d77355c94485681ddba6bd157b7bda9b0fd680c0886",
    ),
    (
        0x24896,
        160,
        "c31d8182d6cc23a5aacef7530b49ed2baae398900c4919c5c5b5654f29a64eb1",
    ),
    (
        0x26b18,
        160,
        "be6573c72222803f841cb133149db7d26ca3b1364ca6ad9a2a8c5dfc2b7f9b6c",
    ),
    (
        0x27e0e,
        160,
        "f8a415b7a0392e4d7461a825c573578e968cabf31d3bfeb8fb7dbf615ab5e224",
    ),
    (
        0x27eb8,
        160,
        "db9e057635a2483a2ccf57c628f859e79aece6c1c7c0c89cb16412f2ca2b2749",
    ),
    (
        0x27f6a,
        160,
        "36f9ce6f224c22adbfe31ef724e5e6124a06c6410f3d439e09f4421527d7e667",
    ),
    (
        0x281b8,
        160,
        "5015e4a7cdefb6598c554c59727b2b6cd569360d0ad3dc634df3c7c3185ef51d",
    ),
    (
        0x282ce,
        160,
        "0d1e970b367954ecf666a3d03cf4f2630ddced1da2e3a5580a2eea76dd389b4f",
    ),
    (
        0x28b40,
        160,
        "7d045b3eed06e69be518eb83c572da78992c04e0e0a2c0b8aaf34261c3edebdd",
    ),
    (
        0x28c18,
        160,
        "af36b3882ef41a9395cee351f6c1725934eb5bb79d4c88754d8d87b9e376f832",
    ),
    (
        0xaaf3e,
        160,
        "ca748990d46039409cbe3850498b73104860b8b61f462d2948f778c1e0873d01",
    ),
    (
        0xb955e,
        160,
        "456213a80f16a4505cc653a856b988761b2439966be0e7fecdb6008a92f4c597",
    ),
    (
        0xc67ac,
        160,
        "556dacaa20b8b7a575950c9ffcda6f3ec15f79348658a09d976d8343a82fac01",
    ),
    (
        0xc6bd2,
        160,
        "039e0d57a12595cca94369141167fdac6ac66c2af2a7174dfd3e711e19b534e8",
    ),
    (
        0xd0796,
        160,
        "fd50352cda7826351311a3b34096164e1a68b3028a233a4a54b8550c981b9d54",
    ),
    (
        0x137a98,
        160,
        "25c007ffe114369b00d3ad42f5f79a9831ebcbf93f9673d352114704eb722034",
    ),
    (
        0x137dba,
        160,
        "706963e83eaeec3fde617fce923a6165604f9fcc2dd1da0467aa4af306bffff4",
    ),
    (
        0xd07a4,
        308,
        "aeb379541ff9887f106e89b5e189a39122dd19827a432c53885794b8b0063e94",
    ),
    (
        0xb8526,
        180,
        "a7c074fe46a684889c3f4086ee6f29e4fb8e7db6209cb6d3a651e3a2a0409206",
    ),
    (
        0xc5508,
        180,
        "e3fe001765794483c7c39477f6bd34ee80de460f5ca074a7ce0fcd33827dbfb2",
    ),
];

#[cfg(test)]
mod tests;
