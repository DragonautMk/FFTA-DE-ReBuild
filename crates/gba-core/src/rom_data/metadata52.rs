//! Lossless EUR 52-byte records; original packed-field and redirect logic stays guest code.
use super::{item_links, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e5464;
pub const END_OFFSET: u32 = 0x6e6bf4;
pub const RECORD_BYTES: usize = 52;
pub const RECORD_COUNT: usize = 116;
pub const REFERENCES: &[(u32, u32)] = &[
    (0xcbeda, 0xcbef0),
    (0xcc6bc, 0xcc6e0),
    (0xcc704, 0xcc73c),
    (0xcce48, 0xcce94),
    (0x133b72, 0x133b7c),
    (0x133ba6, 0x133bb0),
    (0x1344ba, 0x1344c4),
    (0x137dc4, 0x137dd0),
];
pub const JUMP_TARGETS: [u32; 48] = [
    0x80cbfb8, 0x80cbfd0, 0x80cbfe2, 0x80cbfe6, 0x80cbff8, 0x80cc010, 0x80cc028, 0x80cc03e,
    0x80cc052, 0x80cc064, 0x80cc076, 0x80cc088, 0x80cc09a, 0x80cc0ac, 0x80cc0be, 0x80cc0d4,
    0x80cc0f2, 0x80cc108, 0x80cc11e, 0x80cc13c, 0x80cc152, 0x80cc166, 0x80cc17c, 0x80cc18e,
    0x80cc1a0, 0x80cc1b2, 0x80cc1c4, 0x80cc1e0, 0x80cc1fa, 0x80cc216, 0x80cc230, 0x80cc244,
    0x80cc258, 0x80cc294, 0x80cc26a, 0x80cc280, 0x80cc2a4, 0x80cc2b4, 0x80cc2c4, 0x80cc2d4,
    0x80cc2e4, 0x80cc2f4, 0x80cc304, 0x80cc314, 0x80cc324, 0x80cc334, 0x80cc344, 0x80cc354,
];
/// (First byte, bit offset, bit width) in the original property's dispatch order.
pub const PROPERTY_FIELDS: [(usize, u32, u32); 48] = [
    (0, 0, 16),
    (4, 0, 8),
    (5, 0, 8),
    (6, 0, 8),
    (7, 0, 16),
    (9, 0, 16),
    (11, 0, 4),
    (11, 4, 4),
    (12, 0, 8),
    (13, 0, 8),
    (14, 0, 8),
    (15, 0, 8),
    (16, 0, 8),
    (17, 0, 8),
    (18, 3, 3),
    (18, 6, 3),
    (19, 1, 3),
    (19, 4, 3),
    (19, 7, 3),
    (20, 2, 3),
    (20, 5, 3),
    (21, 0, 3),
    (22, 0, 8),
    (23, 0, 8),
    (24, 0, 8),
    (25, 0, 8),
    (26, 0, 12),
    (27, 4, 12),
    (29, 0, 12),
    (30, 4, 12),
    (40, 0, 8),
    (41, 0, 8),
    (42, 0, 8),
    (44, 0, 8),
    (43, 0, 4),
    (43, 4, 4),
    (45, 0, 8),
    (46, 0, 8),
    (47, 0, 8),
    (48, 0, 8),
    (49, 0, 8),
    (32, 0, 8),
    (33, 0, 8),
    (34, 0, 8),
    (35, 0, 8),
    (36, 0, 8),
    (37, 0, 8),
    (38, 0, 8),
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub bytes_00_04: [u8; 5],
    pub routing: u8,
    pub bytes_06_1f: [u8; 26],
    pub bytes_20_33: [u8; 20],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0..=4 => Some(self.bytes_00_04[offset]),
            5 => Some(self.routing),
            6..=31 => Some(self.bytes_06_1f[offset - 6]),
            32..=51 => Some(self.bytes_20_33[offset - 32]),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    pub fn packed_property(&self, property: u8) -> Option<u32> {
        let &(offset, shift, width) = PROPERTY_FIELDS.get(usize::from(property))?;
        let window = u32::from(self.byte(offset)?) | (u32::from(self.byte(offset + 1)?) << 8);
        Some((window >> shift) & ((1 << width) - 1))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata52 {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl Metadata52 {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR metadata52 extent/count".into());
        }
        Ok(())
    }
    /// Normal active selectors name a row; FF keeps its separate guest contract.
    pub fn validate_links(&self, links: &item_links::Links) -> Result<(), String> {
        self.validate()?;
        links.validate()?;
        if links.records.iter().any(|r| {
            r.slots[..usize::from(r.count)]
                .iter()
                .any(|p| p.selector != 255 && usize::from(p.selector) >= self.records.len())
        }) {
            return Err("Metadata52 link has no row owner".into());
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    /// None denotes a nonreturning/out-of-contract path, never an emulated fallback.
    /// The runtime continues executing the original getter and does not call this.
    pub fn property(&self, row: u32, alternate: u32, property: u32) -> Option<u32> {
        let property = property as u8;
        PROPERTY_FIELDS.get(usize::from(property))?;
        let mut row = row as u8;
        let alternate = alternate as u8;
        let mut seen = [false; RECORD_COUNT];
        loop {
            let record = self.records.get(usize::from(row))?;
            if property == 2 || record.routing == 0 {
                return record.packed_property(property);
            }
            let visited = seen.get_mut(usize::from(row))?;
            if *visited {
                return None;
            }
            *visited = true;
            row = if record.routing == 255 {
                alternate
            } else {
                record.routing
            };
        }
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata52 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}
fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
}
fn literal(rom: &[u8], pc: u32, at: u32, target: u32) -> Result<(), String> {
    let h = half(rom, pc);
    if h & 0xf800 != 0x4800
        || ((pc + 4) & !3) + u32::from(h & 255) * 4 != at
        || word(rom, at) != 0x08000000 + target
    {
        return Err("Metadata52 original literal mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Metadata52,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata52 requires exact EUR ROM; USA pending".into());
        }
        for &(pc, at) in REFERENCES {
            literal(rom, pc, at, SOURCE_OFFSET)?;
        }
        for (pc, at) in [(0x64300, 0x6434c), (0xcc426, 0xcc470)] {
            literal(rom, pc, at, END_OFFSET)?;
        }
        for (pc, value) in [
            (0xcce82, 0x2e73),
            (0xcbed6, 0x2134),
            (0xcbed8, 0x4341),
            (0xcbee2, 0x2d2f),
            (0xcbee4, 0xd8fd),
            (0xcbfe2, 0x7958),
            (0xcc35a, 0x2034),
            (0xcc35c, 0x4360),
            (0xcc366, 0x7958),
            (0xcc36a, 0x4348),
            (0x1344be, 0x3033),
            (0x137dc8, 0x3032),
        ] {
            if half(rom, pc) != value {
                return Err("Metadata52 original enumeration/stride/routing mismatch".into());
            }
        }
        for (i, &target) in JUMP_TARGETS.iter().enumerate() {
            if word(rom, 0xcbef8 + 4 * i as u32) != target {
                return Err("Metadata52 property code target mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Metadata52 {
            source_offset: SOURCE_OFFSET,
            records: bytes
                .chunks_exact(RECORD_BYTES)
                .map(|r| Record {
                    bytes_00_04: r[..5].try_into().unwrap(),
                    routing: r[5],
                    bytes_06_1f: r[6..32].try_into().unwrap(),
                    bytes_20_33: r[32..].try_into().unwrap(),
                })
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(bytes),
                role: "metadata52/eur-fixed".into(),
            }],
        })
    }
}
