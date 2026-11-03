//! Observed EUR world records; encoded guest views can address outside the catalog.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x71e0e0;
pub const RECORD_BYTES: usize = 38;
pub const RECORD_COUNT: usize = 31;
pub const SOURCE_BYTES: usize = RECORD_COUNT * RECORD_BYTES;
pub const SOURCE_END: u32 = SOURCE + SOURCE_BYTES as u32;
pub const SOURCE_SHA256: &str = "cd2b33e95a5b46a53703f8a32f89a6f1c0eea4d1b562dfd86ee7c4836084d483";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u8,
    pub opaque00: u8,
    pub byte01: u8,
    pub byte02: u8,
    pub byte03: u8,
    pub value04: u16,
    pub opaque06: u8,
    pub thresholds07: [u8; 31],
}
impl Record {
    fn decode(index: u8, b: &[u8]) -> Self {
        Self {
            index,
            opaque00: b[0],
            byte01: b[1],
            byte02: b[2],
            byte03: b[3],
            value04: u16::from_le_bytes([b[4], b[5]]),
            opaque06: b[6],
            thresholds07: b[7..38].try_into().unwrap(),
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut b = [0; RECORD_BYTES];
        b[..4].copy_from_slice(&[self.opaque00, self.byte01, self.byte02, self.byte03]);
        b[4..6].copy_from_slice(&self.value04.to_le_bytes());
        b[6] = self.opaque06;
        b[7..].copy_from_slice(&self.thresholds07);
        b
    }
    pub fn value02(&self) -> u16 {
        u16::from_le_bytes([self.byte02, self.byte03])
    }
    pub fn low10_value04(&self) -> u16 {
        self.value04 & 0x3ff
    }
    pub fn byte01_divisor(&self) -> u8 {
        self.byte01 >> 1
    }
    /// Only the arithmetic after the original external helper; no RNG replacement.
    pub fn byte01_after_helper(&self, result: u32) -> u8 {
        let value = self.byte01.wrapping_add(result as u8);
        if value == 0 {
            1
        } else {
            value
        }
    }
    /// The original updater clamps its live unsigned-byte count, not stored data.
    pub fn threshold_for_live_count(&self, count: u8) -> u8 {
        self.thresholds07[usize::from(count.min(30))]
    }
    pub fn threshold_branch_ge(&self, count: u8, result: i16) -> bool {
        i32::from(result) >= 100 - i32::from(self.threshold_for_live_count(count))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldRecords38 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl WorldRecords38 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.records.len() != RECORD_COUNT
            || self
                .records
                .iter()
                .enumerate()
                .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid observed EUR world-record physical shape".into());
        }
        Ok(())
    }
    /// Physical ownership does not truncate the requested index.
    pub fn record(&self, index: u32) -> Option<&Record> {
        self.records.get(index as usize)
    }
    pub fn encoded_u8_view(index: u32) -> u32 {
        Self::encoded_u32_view(u32::from(index as u8))
    }
    pub fn encoded_u32_view(index: u32) -> u32 {
        (0x08000000 + SOURCE).wrapping_add(index.wrapping_mul(RECORD_BYTES as u32))
    }
    /// Interior row1 shares the same physical owner; this is address arithmetic only.
    pub fn encoded_update_view(ordinal: u32) -> u32 {
        Self::encoded_u32_view(ordinal.wrapping_add(1))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(SOURCE)?;
        if relative >= SOURCE_BYTES as u32 {
            return None;
        }
        self.record(relative / RECORD_BYTES as u32)
            .map(|r| r.source_bytes()[relative as usize % RECORD_BYTES])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE,
            self.source_bytes(),
            "world-records38".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("World records differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: WorldRecords38,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("World records require exact EUR ROM; USA pending".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err("World-record reader fingerprint mismatch".into());
            }
        }
        let decoded = WorldRecords38 {
            source_offset: SOURCE,
            records: (0..RECORD_COUNT)
                .map(|i| {
                    let at = SOURCE as usize + i * RECORD_BYTES;
                    Record::decode(i as u8, &rom[at..at + RECORD_BYTES])
                })
                .collect(),
        };
        if ffta_rom::digest(&decoded.source_bytes()) != SOURCE_SHA256 {
            return Err("World-record source fingerprint mismatch".into());
        }
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

// Exact fingerprints are generated from the accepted static source contract.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0xd3460,
        224,
        "3a705608b605b31931bb5bfddd0e3023933c9ffef6f62e32209378e7f5fdf649",
    ),
    (
        0x4a100,
        160,
        "6eb6d0ebf6f38ea8f7d59c43aef12559d046b92a44ab153ec5dc6359f37af236",
    ),
    (
        0x468d0,
        128,
        "0aedaa915fd95fb6f3f78adb11807cde591f6428fbfcf853a8f1ac4259b22288",
    ),
    (
        0x4a19e,
        50,
        "7b66b4148f4725853865408bb4660ecba1ea39ad0cd130a8b09870cd52a7fd4f",
    ),
    (
        0xd4ac0,
        112,
        "fe58c562ac142ddedf33f99cfd24e1790c8231f7bc139d86a9decfcbf443cdbd",
    ),
    (
        0xd5ac0,
        96,
        "7be15e321112fcf8a8a28ca15d4ef7785455424604ca846dc770e8d49d46650a",
    ),
    (
        0xd3540,
        384,
        "7356f78562a3a758da2440a338deaded1e7a7e477a886e5cbcd554b9aec8518d",
    ),
    (
        0xd36c0,
        96,
        "c6369bcf9fd1bad5b224cd4f8df0d68500d59d5514bc257e758c939d69219c57",
    ),
    (
        0x19660,
        176,
        "374b9553e61568685700a2e7d17325df4cb92e2717c7a10c08e69321c5e4a9ea",
    ),
    (
        0xd4d60,
        576,
        "dd7cb777c1a69c40cf5f03723b82372d5dbd55c21ea68628f6ee2f45a7ca830c",
    ),
    (
        0xd4c00,
        128,
        "1a628a813056b8ddfd4ffba81b18d79c5a372a19b0cc96f5918ef3686a2858ba",
    ),
    (
        0xd4cc0,
        160,
        "54917806c84428fb47a9ef4a4433dba358ffa985aa21ea27da927fd5994816e9",
    ),
];

#[cfg(test)]
mod tests;
