//! Bounded special-unit records indexed through an original biased address.
use super::{metadata52, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const ADDRESS_BIAS: u32 = 0x6e6f50;
pub const SOURCE_OFFSET: u32 = 0x6e6f6c;
pub const END_OFFSET: u32 = 0x6e7482;
pub const NEXT_SOURCE: u32 = 0x6e7484;
pub const FIRST_ID: u8 = 2;
pub const RECORD_COUNT: usize = 93;
pub const RECORD_BYTES: usize = 14;
pub const REFERENCES: &[(u32, u32)] = &[
    (0xcd05c, 0xcd07c),
    (0xcf0a6, 0xcf0e8),
    (0xcf168, 0xcf17c),
    (0xcf1a4, 0xcf1b8),
    (0xcf1e0, 0xcf1ec),
    (0xcf20a, 0xcf218),
    (0xcf250, 0xcf25c),
    (0xcf2b4, 0xcf2c0),
    (0xcf370, 0xcf37c),
    (0xcd438, 0xcd44c),
    (0xd252e, 0xd255c),
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub byte00: u8,
    pub selectors01_02: [u8; 2],
    pub byte03: u8,
    pub halfwords04_06: [u16; 2],
    pub packed08_0a: [u8; 3],
    pub byte0b: u8,
    pub bytes0c_0d: [u8; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteSelection {
    pub bank: u8,
    pub source_index: Option<u8>,
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0 => self.byte00,
            1..=2 => self.selectors01_02[offset - 1],
            3 => self.byte03,
            4..=7 => (self.halfwords04_06[(offset - 4) / 2] >> (offset % 2 * 8)) as u8,
            8..=10 => self.packed08_0a[offset - 8],
            11 => self.byte0b,
            12..=13 => self.bytes0c_0d[offset - 12],
            _ => return None,
        })
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Diagnostic selection only; live book lookup and original copy still execute.
    pub fn palette(&self, argument: u32) -> PaletteSelection {
        let [a, b, c] = self.packed08_0a;
        let (bank, selector) = if argument as u8 == 1 {
            (a & 15, (a >> 4) | ((b & 15) << 4))
        } else {
            (b >> 4, c)
        };
        PaletteSelection {
            bank,
            source_index: selector.checked_sub(1),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata14 {
    pub address_bias: u32,
    pub source_offset: u32,
    pub first_id: u8,
    pub records: Vec<Record>,
}
impl Metadata14 {
    pub fn validate(&self) -> Result<(), String> {
        if self.address_bias != ADDRESS_BIAS
            || self.source_offset != SOURCE_OFFSET
            || self.first_id != FIRST_ID
            || self.records.len() != RECORD_COUNT
        {
            return Err("Invalid fixed EUR metadata14 extent/bias/count".into());
        }
        for (i, r) in self.records.iter().enumerate() {
            if r.id != FIRST_ID + i as u8
                || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32
            {
                return Err("Invalid metadata14 ID/order/source offset".into());
            }
        }
        Ok(())
    }
    pub fn validate_metadata52(&self, owner: &metadata52::Metadata52) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        // Byte02 becomes the live job when the descriptor does not override it.
        // Byte01 has no proven semantic owner and is deliberately not constrained here.
        if self
            .records
            .iter()
            .any(|r| usize::from(r.selectors01_02[1]) >= owner.records.len())
        {
            return Err("Metadata14 byte02 has no parsed metadata52 job owner".into());
        }
        Ok(())
    }
    /// None denotes outside the owned source, never an emulated bounds fallback.
    pub fn record(&self, id: u32) -> Option<&Record> {
        self.records
            .get(usize::from((id as u8).checked_sub(FIRST_ID)?))
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata14 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(r: &[u8], a: u32) -> u16 {
    u16::from_le_bytes(r[a as usize..a as usize + 2].try_into().unwrap())
}
fn word(r: &[u8], a: u32) -> u32 {
    u32::from_le_bytes(r[a as usize..a as usize + 4].try_into().unwrap())
}
fn literal(r: &[u8], pc: u32, at: u32, target: u32) -> Result<(), String> {
    let v = half(r, pc);
    if v & 0xf800 != 0x4800
        || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
        || word(r, at) != 0x08000000 + target
    {
        return Err("Metadata14 original biased/next literal mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Metadata14,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata14 requires exact EUR ROM; USA pending".into());
        }
        for &(pc, at) in REFERENCES {
            literal(rom, pc, at, ADDRESS_BIAS)?;
        }
        literal(rom, 0x13762e, 0x137640, NEXT_SOURCE)?;
        for (pc, v) in [
            (0xcbadc, 0x7900),
            (0xcbade, 0x2801),
            (0xcbae0, 0xd900),
            (0xcd056, 0x00c8),
            (0xcd058, 0x1a40),
            (0xcd05a, 0x0040),
            (0xcf162, 0x00c8),
            (0xcf164, 0x1a40),
            (0xcf166, 0x0040),
            (0xcf16c, 0x7901),
            (0xcf16e, 0x7940),
            (0xcf1a8, 0x7981),
            (0xcf1aa, 0x79c0),
            (0xcf1e4, 0x7ac0),
            (0xcf212, 0x7b00),
            (0xcf21c, 0x7b40),
            (0x137628, 0x0058),
            (0x13762a, 0x18c0),
            (0x13762c, 0x0080),
            (0xcf0ae, 0x7a34),
            (0xcf0b2, 0x7a71),
            (0xcf0f4, 0x7ab1),
            (0xcf11c, 0x7a70),
        ] {
            if half(rom, pc) != v {
                return Err(
                    "Metadata14 original stride/predicate/field/palette reader mismatch".into(),
                );
            }
        }
        if rom[END_OFFSET as usize..NEXT_SOURCE as usize] != [0, 0] {
            return Err("Metadata14 trailing gap mismatch".into());
        }
        let raw = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let records = raw
            .chunks_exact(RECORD_BYTES)
            .enumerate()
            .map(|(i, r)| Record {
                id: FIRST_ID + i as u8,
                source_offset: SOURCE_OFFSET + (i * RECORD_BYTES) as u32,
                byte00: r[0],
                selectors01_02: r[1..3].try_into().unwrap(),
                byte03: r[3],
                halfwords04_06: std::array::from_fn(|i| {
                    u16::from_le_bytes(r[4 + i * 2..6 + i * 2].try_into().unwrap())
                }),
                packed08_0a: r[8..11].try_into().unwrap(),
                byte0b: r[11],
                bytes0c_0d: r[12..14].try_into().unwrap(),
            })
            .collect();
        let decoded = Metadata14 {
            address_bias: ADDRESS_BIAS,
            source_offset: SOURCE_OFFSET,
            first_id: FIRST_ID,
            records,
        };
        decoded.validate_metadata52(&metadata52::Catalog::from_rom(rom)?.decoded)?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: raw.len() as u32,
                sha256: ffta_rom::digest(raw),
                role: "metadata14/eur-records".into(),
            }],
        })
    }
}
