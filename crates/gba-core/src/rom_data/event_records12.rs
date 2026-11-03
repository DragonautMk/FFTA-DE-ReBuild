//! EUR event storage and diagnostic reader projections; live event logic stays guest code.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x71c760;
pub const SOURCE_END: u32 = 0x71d360;
pub const ALIAS_SOURCE: u32 = 0x71d1ec;
pub const ALIAS_ROW: usize = 225;
pub const RECORD_COUNT: usize = 256;
pub const RECORD_BYTES: usize = 12;
pub const SOURCE_BYTES: usize = RECORD_COUNT * RECORD_BYTES;
pub const SOURCE_SHA256: &str = "e74541202207ed2249fcec14f6ca56073ce3b786acaa08f8b0efbdec1d858f24";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u16,
    pub header00: u8,
    pub event_key01: u8,
    pub primary02: u16,
    pub alternate04: u16,
    pub parameter06: u8,
    pub byte07: u8,
    pub flags08_raw: u16,
    pub opaque0a: [u8; 2],
}
impl Record {
    fn decode(index: u16, b: &[u8]) -> Self {
        let word = |at| u16::from_le_bytes([b[at], b[at + 1]]);
        Self {
            index,
            header00: b[0],
            event_key01: b[1],
            primary02: word(2),
            alternate04: word(4),
            parameter06: b[6],
            byte07: b[7],
            flags08_raw: word(8),
            opaque0a: [b[10], b[11]],
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut b = [0; RECORD_BYTES];
        b[0] = self.header00;
        b[1] = self.event_key01;
        b[2..4].copy_from_slice(&self.primary02.to_le_bytes());
        b[4..6].copy_from_slice(&self.alternate04.to_le_bytes());
        b[6] = self.parameter06;
        b[7] = self.byte07;
        b[8..10].copy_from_slice(&self.flags08_raw.to_le_bytes());
        b[10..12].copy_from_slice(&self.opaque0a);
        b
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventRecords12 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl EventRecords12 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.records.len() != RECORD_COUNT
            || self
                .records
                .iter()
                .enumerate()
                .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid declared EUR event-record physical shape".into());
        }
        Ok(())
    }
    /// Physical ownership only; None does not introduce a guest bounds check.
    pub fn record(&self, index: u32) -> Option<&Record> {
        self.records.get(index as usize)
    }
    /// Original computed view of a supplied unsigned16 index, even outside this owner.
    pub fn encoded_view(index: u32) -> u32 {
        0x08000000 + SOURCE + 12 * u32::from(index as u16)
    }
    pub fn encoded_alias_view(argument: u32) -> u32 {
        0x08000000 + ALIAS_SOURCE + 12 * u32::from(argument as u8)
    }
    pub fn alias_record(&self, argument: u32) -> Option<&Record> {
        self.records.get(ALIAS_ROW + usize::from(argument as u8))
    }
    /// Original pair before the live setter. Unowned alias reads remain unanswered.
    pub fn conditional_pair(&self, supplied: &Record, argument: u32) -> Option<(u16, u16)> {
        if supplied.alternate04 == 0 {
            Some((supplied.primary02, 0))
        } else {
            self.alias_record(argument)
                .map(|r| (r.primary02, supplied.alternate04))
        }
    }
    /// Full16-bit script key; first match in backward255..1 order, excluding row0.
    pub fn script_match(&self, key: u16) -> Option<(u8, u16)> {
        self.records
            .iter()
            .enumerate()
            .skip(1)
            .rev()
            .find(|(_, r)| u16::from(r.event_key01) == key)
            .map(|(i, r)| (i as u8, r.primary02))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let at = offset.checked_sub(SOURCE)? as usize;
        if at >= SOURCE_BYTES {
            return None;
        }
        self.records
            .get(at / RECORD_BYTES)
            .map(|r| r.source_bytes()[at % RECORD_BYTES])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE,
            self.source_bytes(),
            "event-records12".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Event records differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: EventRecords12,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Event records require exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[SOURCE as usize..SOURCE_END as usize]) != SOURCE_SHA256 {
            return Err("Event-record source fingerprint mismatch".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err("Event-record reader/boundary fingerprint mismatch".into());
            }
        }
        let decoded = EventRecords12 {
            source_offset: SOURCE,
            records: rom[SOURCE as usize..SOURCE_END as usize]
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, b)| Record::decode(i as u16, b))
                .collect(),
        };
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

const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x9c70,
        304,
        "51f3f88d461ca7c2237a40457e7c55bb87f1998705e841246d9bce4cbe140816",
    ),
    (
        0x127388,
        64,
        "96ff25467162775e7df3839ddbbce39a2d38abab097421a5fd83d4fd4771a802",
    ),
    (
        0x63920,
        136,
        "50d0c95bdd79f2bad90ed7190427516da40d93408b7ecb67ca49bf32307b09ea",
    ),
    (
        0x1273c4,
        64,
        "5ebf187856545a058f18ec68a17f176f9c95f73dcaf469d5335ac67d76f855a2",
    ),
    (
        0x639a4,
        20,
        "90a908e336ef9ce0f399bbd6f15b851203434b7732260f91fcac5cabf9a09de2",
    ),
    (
        0xd40e0,
        256,
        "44cc1172b649bdf0d6c542695a0dfa7003bf76d0341ece63d8b92c8a5831f5c7",
    ),
    (
        0x359e0,
        96,
        "96e667bd332ab4b167a81d7f56d328c1f698df8ad7f0f8bae3325ff3d87ce950",
    ),
    (
        0x28de0,
        112,
        "d91c8fdd1cc747800eb7dff259d4656f4e8fda4b5469e8ed142fb956a72452ab",
    ),
    (
        0xd2540,
        224,
        "f2d205de3dca63a1437dedd058b6b64eb5ace5f0daf85be7abe8da9a1e2396ec",
    ),
    (
        0x34500,
        192,
        "f78d4d8326d304bcfb1fd3fbc23e23cc35f0b1355737d0d7cdd9dac5899947a4",
    ),
    (
        0xd3348,
        224,
        "2b05d1f1792ba5355746186549e33c14c73f8c81fd09351c773a440c6722bdfd",
    ),
    (
        0x24d70,
        128,
        "86c78eb99654cfb6bfac2f6447a7d91efb3632c0c505211af3376c91d8c06be4",
    ),
    (
        0xd46a0,
        128,
        "90bdb1d7334e286b249c73cd3e33968385314b519d114aa8bd8c2befa2564e5c",
    ),
    (
        0x28e50,
        128,
        "8adaf6df496fe7672e0eb9d7fb241740b0839319eaee82bda05ff3e732c36198",
    ),
    (
        0xd4ca0,
        160,
        "7a755d9d602a87f01d0abb48a04a2ea14ee454be49c9cc6d75d55b35d7684ff4",
    ),
];

#[cfg(test)]
mod tests;
