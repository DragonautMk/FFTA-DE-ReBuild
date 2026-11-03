//! Ordered EUR four-byte slots. Physical ownership is separate from guest indexing.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x7133f8;
pub const SOURCE_END: u32 = 0x713744;
pub const RECORD_BYTES: usize = 4;
pub const RECORD_COUNT: usize = 211;
pub const SOURCE_BYTES: usize = RECORD_BYTES * RECORD_COUNT;
pub const SOURCE_SHA256: &str = "85d556e39da8baaec49308c815125532aa8947a476e2c59131532a5bf0334fa9";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u8,
    pub field00: u8,
    pub field01: u8,
    pub field02: u8,
    pub field03: u8,
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        [self.field00, self.field01, self.field02, self.field03]
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.source_bytes().get(offset).copied()
    }
}

/// Original readers load the index as u8. This returns an encoded guest address,
/// including indices outside this catalog's physical ownership.
pub fn source_address(index: u8) -> u32 {
    0x08000000 + SOURCE + u32::from(index) * RECORD_BYTES as u32
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records4 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Records4 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid declared EUR records4 source/count".into());
        }
        if self
            .records
            .iter()
            .enumerate()
            .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid records4 physical slot order/index".into());
        }
        Ok(())
    }
    /// None means unowned; it does not replace an original guest read with zero.
    pub fn record(&self, index: u8) -> Option<&Record> {
        self.records.get(usize::from(index))
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
            "linked-records4-slots".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Records4 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Records4,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Records4 requires exact EUR ROM; USA pending".into());
        }
        let raw = &rom[SOURCE as usize..SOURCE_END as usize];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Records4 source fingerprint mismatch".into());
        }
        for &(start, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Records4 original reader/boundary fingerprint mismatch".into());
            }
        }
        let decoded = Records4 {
            source_offset: SOURCE,
            records: raw
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, r)| Record {
                    index: i as u8,
                    field00: r[0],
                    field01: r[1],
                    field02: r[2],
                    field03: r[3],
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
                role: "linked-records4-slots".into(),
            }],
        })
    }
}

// Authenticated source-contract windows, including independent successor witnesses.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
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
    (
        0xc5a26,
        180,
        "e7d52ed7d1ab6ee8eef12d227858046c52e2d55b0fa8938753a876196308baf1",
    ),
    (
        0x132efc,
        180,
        "d3aa2fea837fa08b2510ecd17971b23dee4f24d3ad48e1f2ff3bee80cbd2b96e",
    ),
    (
        0x132faa,
        180,
        "dff6369797cda55a78c60351b52eae0cf9f451227e2d5ab382fda6af6087074c",
    ),
    (
        0x133030,
        180,
        "b5ad1e957c1c1d7d15612113553d2913a968d4ab292a0fecf7b8de678cf9f978",
    ),
    (
        0xd56aa,
        180,
        "af4b1b981d48365083439ae48233adadedd00d73840946d1b3e42b38b2006a59",
    ),
    (
        0x132ea0,
        1520,
        "9c4ae6e42c8c9a26a4f590293616210401d537b25a23d22f9b853330609fc2f8",
    ),
    (
        0xd573e,
        160,
        "2807672ca77b65fd0fc874da2457c17f0b50ac71e8aa2526abe724545a3efff8",
    ),
    (
        0x134c20,
        1424,
        "a1bafc8880d36c42eeeca6f38da496872af9835c8fb02625d569fef2674aed6c",
    ),
    (
        0x135490,
        120,
        "79c2cdd6290375051cf139d2237e879bea40248357d5a30c7db723aa9c9dbee0",
    ),
    (
        0x135780,
        120,
        "f23d11ec127cbb806e59aec5da4fbb312a9e82f27889720bc47cd29b16e4a104",
    ),
    (
        0xc5b80,
        48,
        "87b0c79a1dfb7e774ffae6b38c555186e9113ff7d3b28eacfb1fe1fbc5875370",
    ),
];

#[cfg(test)]
mod tests;
