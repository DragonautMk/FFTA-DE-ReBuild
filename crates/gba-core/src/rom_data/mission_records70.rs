//! EUR mission records, preserving raw storage and original getter projections.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x713b3c;
pub const SOURCE_END: u32 = 0x71c73c;
pub const RECORD_BYTES: usize = 70;
pub const RECORD_COUNT: usize = 512;
pub const SOURCE_BYTES: usize = RECORD_COUNT * RECORD_BYTES;
pub const SOURCE_SHA256: &str = "bbdbc955b9b3f7ed678123aa6bea8b6098f25f6a13a4ab94de669d1e103253dc";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub index: u16,
    pub word00: u16,
    pub packed02: [u8; 3],
    pub key05: u16,
    pub operand07: u8,
    pub key08: u16,
    pub operand0a: u8,
    pub key0b: u16,
    pub operand0d: u8,
    pub packed0e: [u8; 4],
    pub inline12: [u8; 16],
    pub inline22: [u8; 8],
    pub inline2a: [u8; 9],
    pub scaled33_raw: u8,
    pub byte34: u8,
    pub byte35: u8,
    pub mapped36_raw: u8,
    pub mapped37_raw: u8,
    pub packed38: [u8; 5],
    pub mapped3d_raw: u8,
    pub scaled3e_raw: u8,
    pub byte3f: u8,
    pub byte40: u8,
    pub flags41_raw: u16,
    pub word43: u16,
    pub event_row45: u8,
}

impl Record {
    fn decode(index: u16, b: &[u8]) -> Self {
        let word = |off| u16::from_le_bytes([b[off], b[off + 1]]);
        Self {
            index,
            word00: word(0),
            packed02: b[2..5].try_into().unwrap(),
            key05: word(5),
            operand07: b[7],
            key08: word(8),
            operand0a: b[10],
            key0b: word(11),
            operand0d: b[13],
            packed0e: b[14..18].try_into().unwrap(),
            inline12: b[18..34].try_into().unwrap(),
            inline22: b[34..42].try_into().unwrap(),
            inline2a: b[42..51].try_into().unwrap(),
            scaled33_raw: b[51],
            byte34: b[52],
            byte35: b[53],
            mapped36_raw: b[54],
            mapped37_raw: b[55],
            packed38: b[56..61].try_into().unwrap(),
            mapped3d_raw: b[61],
            scaled3e_raw: b[62],
            byte3f: b[63],
            byte40: b[64],
            flags41_raw: word(65),
            word43: word(67),
            event_row45: b[69],
        }
    }

    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut b = [0; RECORD_BYTES];
        b[0..2].copy_from_slice(&self.word00.to_le_bytes());
        b[2..5].copy_from_slice(&self.packed02);
        b[5..7].copy_from_slice(&self.key05.to_le_bytes());
        b[7] = self.operand07;
        b[8..10].copy_from_slice(&self.key08.to_le_bytes());
        b[10] = self.operand0a;
        b[11..13].copy_from_slice(&self.key0b.to_le_bytes());
        b[13] = self.operand0d;
        b[14..18].copy_from_slice(&self.packed0e);
        b[18..34].copy_from_slice(&self.inline12);
        b[34..42].copy_from_slice(&self.inline22);
        b[42..51].copy_from_slice(&self.inline2a);
        b[51] = self.scaled33_raw;
        b[52] = self.byte34;
        b[53] = self.byte35;
        b[54] = self.mapped36_raw;
        b[55] = self.mapped37_raw;
        b[56..61].copy_from_slice(&self.packed38);
        b[61] = self.mapped3d_raw;
        b[62] = self.scaled3e_raw;
        b[63] = self.byte3f;
        b[64] = self.byte40;
        b[65..67].copy_from_slice(&self.flags41_raw.to_le_bytes());
        b[67..69].copy_from_slice(&self.word43.to_le_bytes());
        b[69] = self.event_row45;
        b
    }

    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.source_bytes().get(offset).copied()
    }

    pub fn source_address(&self) -> u32 {
        encoded_record_address(u32::from(self.index))
    }

    /// Original unsigned selector projection; inline views keep ROM addresses.
    pub fn value(&self, selector: u32) -> u32 {
        let selector = selector as u8;
        if let Some(offset) = inline_offset(selector) {
            return self.source_address() + offset;
        }
        let b = self.source_bytes();
        let word = |off| u32::from(u16::from_le_bytes([b[off], b[off + 1]]));
        let bits = |off: usize, shift: u32, width: u32| {
            let high = if shift + width > 8 {
                u32::from(b[off + 1]) << 8
            } else {
                0
            };
            ((u32::from(b[off]) | high) >> shift) & ((1 << width) - 1)
        };
        let mapped = |off: usize| {
            if b[off] == 0 {
                0
            } else {
                u32::from(b[off]) + 0x177
            }
        };
        match selector {
            0 => u32::from(self.word00),
            1 => bits(2, 0, 3),
            2 => bits(2, 3, 3),
            3 => bits(2, 6, 6),
            4 => bits(3, 4, 3),
            5 => u32::from(self.flags41_raw & 1),
            6 => u32::from((self.flags41_raw >> 1) & 1),
            7 => bits(3, 7, 3),
            8 => u32::from(self.key05),
            9 => u32::from(self.operand07),
            10 => u32::from(self.key08),
            11 => u32::from(self.operand0a),
            12 => u32::from(self.key0b),
            13 => u32::from(self.operand0d),
            14 => bits(14, 0, 3),
            15 => bits(14, 3, 8),
            16 => bits(15, 3, 8),
            17 => bits(16, 3, 3),
            18 => bits(16, 6, 8),
            25..=28 => word(34 + usize::from(selector - 25) * 2),
            29 => u32::from(self.byte35),
            30 => u32::from(self.scaled33_raw) * 200,
            31 => u32::from(self.byte34),
            33..=41 => u32::from(self.inline2a[usize::from(selector - 33)]),
            42 => mapped(54),
            43 => u32::from((self.flags41_raw >> 4) & 1),
            44 => mapped(55),
            45 => u32::from((self.flags41_raw >> 5) & 1),
            46 => bits(56, 0, 4),
            47 => bits(56, 4, 7),
            48 => bits(57, 3, 6),
            49 => bits(58, 1, 4),
            50 => bits(58, 5, 7),
            51 => bits(59, 4, 6),
            52 => bits(60, 2, 6),
            53 => mapped(61),
            54 => u32::from(self.scaled3e_raw) * 200,
            55 => u32::from((self.flags41_raw >> 2) & 1),
            56 => u32::from((self.flags41_raw >> 3) & 1),
            57 => u32::from(self.byte3f),
            58 => u32::from(self.byte40),
            59 => u32::from(self.word43),
            60 => u32::from(self.event_row45),
            61 => u32::from((self.flags41_raw >> 6) & 1),
            62 => u32::from((self.flags41_raw >> 7) & 1),
            63 => u32::from((self.flags41_raw >> 8) & 1),
            64 => u32::from((self.flags41_raw >> 9) & 1),
            _ => 0,
        }
    }
}

pub fn encoded_record_address(index: u32) -> u32 {
    0x08000000 + SOURCE + (index & 0xffff) * RECORD_BYTES as u32
}

fn inline_offset(selector: u8) -> Option<u32> {
    Some(match selector {
        19 | 20 => 18,
        21 => 22,
        22 => 26,
        23 => 30,
        24 => 34,
        32 => 42,
        _ => return None,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records70 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Records70 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid declared EUR mission-records70 source/count".into());
        }
        if self
            .records
            .iter()
            .enumerate()
            .any(|(i, r)| usize::from(r.index) != i)
        {
            return Err("Invalid mission-records70 physical order/index".into());
        }
        Ok(())
    }

    pub fn record(&self, index: u16) -> Option<&Record> {
        self.records.get(usize::from(index))
    }

    /// A missing scalar means this owner cannot answer the original read.
    /// Computed views and invalid-selector zero do not require an owned record.
    pub fn query(&self, index: u32, selector: u32) -> Option<u32> {
        let selector = selector as u8;
        if selector > 64 {
            return Some(0);
        }
        if let Some(offset) = inline_offset(selector) {
            return Some(encoded_record_address(index) + offset);
        }
        Some(self.record(index as u16)?.value(u32::from(selector)))
    }

    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if !(SOURCE..SOURCE_END).contains(&offset) {
            return None;
        }
        let at = (offset - SOURCE) as usize;
        self.records.get(at / RECORD_BYTES)?.byte(at % RECORD_BYTES)
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![(
            SOURCE,
            self.source_bytes(),
            "mission-records70".into(),
        )])
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Mission-records70 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Records70,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Mission-records70 requires exact EUR ROM; USA pending".into());
        }
        let raw = &rom[SOURCE as usize..SOURCE_END as usize];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Mission-records70 source fingerprint mismatch".into());
        }
        for &(at, length, sha) in SOURCE_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != sha {
                return Err("Mission-records70 reader/boundary fingerprint mismatch".into());
            }
        }
        let decoded = Records70 {
            source_offset: SOURCE,
            records: raw
                .chunks_exact(RECORD_BYTES)
                .enumerate()
                .map(|(i, b)| Record::decode(i as u16, b))
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE,
                length: SOURCE_BYTES as u32,
                sha256: ffta_rom::digest(raw),
                role: "mission-records70".into(),
            }],
        })
    }
}

// The closed source contract authenticates these independently witnessed readers.
const SOURCE_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0xd1f00,
        140,
        "b79596f9fe4e76e3c5647d48abe168ff8b732d13f74db3e5a373e8765ce3ce78",
    ),
    (
        0x31c20,
        180,
        "6a81135e710e8e8592c6685de3a9c75637b2033ac799638c0ec3690fd68b42c0",
    ),
    (
        0xd26e0,
        240,
        "c0b7cfdba6c3524fdb46976479a75b91c2b700d216a1a57605768db78bb91486",
    ),
    (
        0x63920,
        136,
        "50d0c95bdd79f2bad90ed7190427516da40d93408b7ecb67ca49bf32307b09ea",
    ),
    (
        0xcbe4c,
        128,
        "d15891f19f1b338423457d55b50c2dbfca2f3aebbe95f15fcde6c77e69b6c0d5",
    ),
    (
        0xbc058,
        192,
        "b7b5ec5c4308810347b665237d50e8b83b50d6a63fd3ed1ea7e609d96b5bbfe8",
    ),
    (
        0x639a4,
        20,
        "90a908e336ef9ce0f399bbd6f15b851203434b7732260f91fcac5cabf9a09de2",
    ),
    (
        0xd2f40,
        80,
        "efdcc7ac85a34da1c5faa8d8ad43b404e94aa5b4dc1add19de6a8cd9debc1772",
    ),
    (
        0x515e0,
        224,
        "23e560ed07288d15d5a380185bd33e0742a0fd4f194940e1a10af2476c8d395a",
    ),
    (
        0x516c0,
        160,
        "49245b35ae543988cab8aef9ee60cdc775b856bbd87ea5899d0ea9c840d7a282",
    ),
    (
        0xd1f30,
        1040,
        "e30e95ccd1360863f209f9b59ccf0bb8d904d8939eaccfb2da169ea7fdce6fa3",
    ),
    (
        0xd3280,
        200,
        "3a11e3c23ad52ade3a8b66fb9c7e6fd3948b43c4ddc417a48243a274903afa09",
    ),
    (
        0xd37c0,
        160,
        "2324fe25427824dad62f1968b27a1d0f2a6a9225b3e02a4970be2127d9346eff",
    ),
    (
        0xd6580,
        160,
        "cb732b4d540807bab4ec0c59e318aca2abd0c6849a094a4a194f33e344c034a9",
    ),
    (
        0xceed0,
        150,
        "81b255384a4ef093c9fe7d8ed2515d68f0939357f69b01b1473a7d134fd78fdd",
    ),
    (
        0xd3230,
        80,
        "a6a8d5fe0038ed5dede6a552a42a3313fcb00467b724c05d2f19a9ccf06714b9",
    ),
    (
        0xd3710,
        176,
        "c9f861dd019d196e01442b2970efd49369ac3b40b115e7b18b072965a504694d",
    ),
    (
        0xd3f70,
        128,
        "503762d3915c211468994dc763cc3d1fcdd6df35d233f2a3a5c80036a0981196",
    ),
    (
        0xd64c0,
        80,
        "b406b8ee427a61c9368a49db8b344f82161c71a660991c09c935c0d3e523f239",
    ),
];

#[cfg(test)]
mod tests;
