//! Observed EUR parameter records. Unclassified indices retain raw guest reads.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x71d360;
pub const RECORD_BYTES: usize = 27;
pub const OBSERVED_COUNT: usize = 59;
pub const SOURCE_BYTES: usize = OBSERVED_COUNT * RECORD_BYTES;
pub const SOURCE_SHA256: &str = "40d4e406d08f42caaa9c87da76a143166a185656d7e8bf987828c562bd3edf75";
/// Inclusive observed index runs, not the full catalog extent or guest bounds.
pub const OBSERVED_RUNS: [(u8, u8); 2] = [(0, 50), (115, 122)];
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u8,
    pub header00: u8,
    pub packed01: u8,
    pub packed02: u8,
    pub packed03: u8,
    pub packed04: u8,
    pub packed05: u8,
    pub packed06: u8,
    pub value07: u16,
    pub byte09: u8,
    pub value0a: u16,
    pub signed0c: i8,
    pub flags0d: u8,
    pub byte0e: u8,
    pub byte0f: u8,
    pub byte10: u8,
    pub byte11: u8,
    pub value12: u16,
    pub opaque14: u8,
    pub signed15: i8,
    pub opaque16: [u8; 5],
}
impl Record {
    fn decode(index: u8, b: &[u8]) -> Self {
        let word = |at| u16::from_le_bytes([b[at], b[at + 1]]);
        Self {
            index,
            header00: b[0],
            packed01: b[1],
            packed02: b[2],
            packed03: b[3],
            packed04: b[4],
            packed05: b[5],
            packed06: b[6],
            value07: word(7),
            byte09: b[9],
            value0a: word(10),
            signed0c: b[12] as i8,
            flags0d: b[13],
            byte0e: b[14],
            byte0f: b[15],
            byte10: b[16],
            byte11: b[17],
            value12: word(18),
            opaque14: b[20],
            signed15: b[21] as i8,
            opaque16: b[22..27].try_into().unwrap(),
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut b = [0; RECORD_BYTES];
        b[..7].copy_from_slice(&[
            self.header00,
            self.packed01,
            self.packed02,
            self.packed03,
            self.packed04,
            self.packed05,
            self.packed06,
        ]);
        b[7..9].copy_from_slice(&self.value07.to_le_bytes());
        b[9] = self.byte09;
        b[10..12].copy_from_slice(&self.value0a.to_le_bytes());
        b[12] = self.signed0c as u8;
        b[13..18].copy_from_slice(&[
            self.flags0d,
            self.byte0e,
            self.byte0f,
            self.byte10,
            self.byte11,
        ]);
        b[18..20].copy_from_slice(&self.value12.to_le_bytes());
        b[20] = self.opaque14;
        b[21] = self.signed15 as u8;
        b[22..27].copy_from_slice(&self.opaque16);
        b
    }
    /// Packed value before the original zero/bit7/external-helper branches.
    pub fn selector01_02(&self) -> u8 {
        (self.packed01 >> 5) | ((self.packed02 & 31) << 3)
    }
    pub fn selector03_04(&self) -> u8 {
        (self.packed03 >> 5) | ((self.packed04 & 15) << 3)
    }
    /// Shared physical nibbles, in the original R1/R2 argument order.
    pub fn nibble_operands(&self) -> (u8, u8) {
        (
            (self.packed05 >> 4) | ((self.packed06 & 15) << 4),
            (self.packed04 >> 4) | ((self.packed05 & 15) << 4),
        )
    }
    pub fn signed0c_scaled(&self) -> i32 {
        i32::from(self.signed0c) * 200
    }
    pub fn signed15_encoded(&self) -> u16 {
        i16::from(self.signed15) as u16
    }
    /// Separate arguments; byte0F is not shifted into a combined halfword.
    pub fn marked_operands0e(&self) -> (u8, u16) {
        (self.byte0f, 0x8000 | u16::from(self.byte0e))
    }
    pub fn marked_operands10(&self) -> (u8, u16) {
        (self.byte11, 0x8000 | u16::from(self.byte10))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterRecords27 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
fn observed_indices() -> impl Iterator<Item = u8> {
    OBSERVED_RUNS
        .into_iter()
        .flat_map(|(first, last)| first..=last)
}
impl ParameterRecords27 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.records.len() != OBSERVED_COUNT
            || self.records.iter().map(|r| r.index).ne(observed_indices())
        {
            return Err("Invalid observed EUR parameter-record physical shape".into());
        }
        Ok(())
    }
    /// Physical observed ownership; no truncation or guest index cap.
    pub fn record(&self, index: u32) -> Option<&Record> {
        let index = u8::try_from(index).ok()?;
        let at = self
            .records
            .binary_search_by_key(&index, |r| r.index)
            .ok()?;
        self.records.get(at)
    }
    /// Original unsigned-byte address computation, including unowned indices.
    pub fn encoded_view(index: u32) -> u32 {
        0x08000000 + SOURCE + RECORD_BYTES as u32 * u32::from(index as u8)
    }
    /// Concatenates the observed runs only; does not fill their gap.
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(SOURCE)?;
        self.record(relative / RECORD_BYTES as u32)
            .map(|r| r.source_bytes()[relative as usize % RECORD_BYTES])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(OBSERVED_RUNS
            .into_iter()
            .map(|(first, last)| {
                let bytes = (first..=last)
                    .flat_map(|i| self.record(u32::from(i)).unwrap().source_bytes())
                    .collect();
                (
                    SOURCE + u32::from(first) * RECORD_BYTES as u32,
                    bytes,
                    "parameter-records27".into(),
                )
            })
            .collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Parameter records differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ParameterRecords27,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Parameter records require exact EUR ROM; USA pending".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err("Parameter-record reader/producer fingerprint mismatch".into());
            }
        }
        let decoded = ParameterRecords27 {
            source_offset: SOURCE,
            records: observed_indices()
                .map(|index| {
                    let at = SOURCE as usize + usize::from(index) * RECORD_BYTES;
                    Record::decode(index, &rom[at..at + RECORD_BYTES])
                })
                .collect(),
        };
        if ffta_rom::digest(&decoded.source_bytes()) != SOURCE_SHA256 {
            return Err("Observed parameter-record source fingerprint mismatch".into());
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

// SOURCE_WINDOWS is generated below from the accepted source contract.

const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
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
        0xd46a0,
        128,
        "90bdb1d7334e286b249c73cd3e33968385314b519d114aa8bd8c2befa2564e5c",
    ),
    (
        0x13fb80,
        176,
        "8168f26dedc17303deade3c0857be173c271024dbf033dfc9de00842386c2f15",
    ),
    (
        0x40a60,
        352,
        "c812fad9a0ebea4b806c8bcc4682129fddd99a5664d599875438ffa082af52ff",
    ),
    (
        0x40ce0,
        160,
        "ba65ecfcbe5742e411c4688eec4f7586f016ee8198b20aaec455cf9674d39f32",
    ),
    (
        0x40f80,
        224,
        "dc41c3eae38ce78e9e293146f0e1edf7cd88333ee2b5a0b6a35a9940b71c3aeb",
    ),
    (
        0x410a0,
        160,
        "c8a75713e669d75c251495e659793a4c14628f71f7a272beb6bbf0e906ce1eef",
    ),
    (
        0x41400,
        512,
        "b6d3b363b4c5acae70f14d3dd3f51c48b56fe5c9735e30559fdca90a9df91d26",
    ),
    (
        0x41600,
        192,
        "2becc1fd8cdd5052bb86e1027b04951c4ffa8e0f54c85e34458b22b66f4c2eb5",
    ),
    (
        0x416c0,
        128,
        "cc2714e446890624127462afb724740c7708918b6d94a10080d06a1a50e2e3cd",
    ),
    (
        0x4a1d0,
        224,
        "afca1a62f5f06a47fe42f76171805518fb9a1b6b02a1c7cc85ccb05854b9163b",
    ),
    (
        0x3dfc0,
        128,
        "ad857375ecee6a47dfe64049d1b72682e6bcab46a5989a52b300c7be2859874f",
    ),
    (
        0x3ed00,
        128,
        "e9c0c57e4aec9e0ddd4c84600f9c5965c2670004d002afaa43f123eda25cd2dc",
    ),
    (
        0xd3460,
        224,
        "3a705608b605b31931bb5bfddd0e3023933c9ffef6f62e32209378e7f5fdf649",
    ),
    (
        0x34d40,
        160,
        "ef75bbe3424e4b12b319f73d4cf61802053834a010f629f4d53efdd0ee88c2fb",
    ),
    (
        0x351d0,
        160,
        "05b80e5b2e7128265d51b7cc98940bf7bd12c393d4a2f783178f9268da5373cc",
    ),
    (
        0x471d0,
        160,
        "0c55b4ce081f4eaf86acb7c4ecd0e86593179e17a67549d73e971263e58dd13c",
    ),
    (
        0x493b0,
        160,
        "40191cc533acbb7129508c57a65107b687517ae43483e7423133de1efdd0fa03",
    ),
    (
        0x49da0,
        160,
        "b1663ec40ddbaa4d810137223003c7764b22ada5f6b14f83404d4cf5c8aac433",
    ),
    (
        0x4a100,
        160,
        "6eb6d0ebf6f38ea8f7d59c43aef12559d046b92a44ab153ec5dc6359f37af236",
    ),
    (
        0x40550,
        128,
        "0bab1720a21c9f35e1e08160e8913a4a5891560e89c02cd834a7ee37e1124b95",
    ),
    (
        0x406c0,
        128,
        "68030c67a65334eb7f3d51902e592dd6f14c3bf19a474d105b676023bd5849c4",
    ),
    (
        0x409c0,
        192,
        "71c178a879382a2508b113ee9919ea130efc21fdabab502f4780e6fb133681e6",
    ),
    (
        0x3df70,
        80,
        "b30b19b2a36f8e6d9e877845e23f22f14ba4588b87977316bb31686f410eb000",
    ),
    (
        0x3ec40,
        192,
        "168bb469d87e019452c11fcbd2fcbf1b006ab431027644a11550dae08247e14e",
    ),
];

#[cfg(test)]
mod tests;
