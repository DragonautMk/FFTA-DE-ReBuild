//! Two indexed banks of packed two-bit fields; names remain structural.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e7484;
pub const SECONDARY_OFFSET: u32 = 0x6e78d4;
pub const END_OFFSET: u32 = 0x6e7994;
pub const RECORD_BYTES: usize = 12;
pub const PAIRS_PER_RECORD: usize = 48;
pub const REFERENCES: &[(u32, u32, u32)] = &[
    (0x13762e, 0x137640, SOURCE_OFFSET),
    (0x137652, 0x137664, SECONDARY_OFFSET),
    (0x1377a2, 0x1377b0, END_OFFSET),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankId {
    Primary,
    Secondary,
}
impl BankId {
    pub fn source_offset(self) -> u32 {
        match self {
            Self::Primary => SOURCE_OFFSET,
            Self::Secondary => SECONDARY_OFFSET,
        }
    }
    pub fn record_count(self) -> usize {
        match self {
            Self::Primary => 92,
            Self::Secondary => 16,
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Primary => 0,
            Self::Secondary => 1,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u16,
    pub source_offset: u32,
    /// Four two-bit fields in each original byte, least significant first.
    pub groups: [[u8; 4]; RECORD_BYTES],
}
impl Record {
    pub fn byte(&self, at: usize) -> Option<u8> {
        Some(
            self.groups
                .get(at)?
                .iter()
                .enumerate()
                .fold(0, |v, (i, p)| v | (p << (2 * i))),
        )
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Finite diagnostic view, not a guest bounds check. Mode retains u8 truncation.
    pub fn flag(&self, slot: u32, mode: u32) -> Option<bool> {
        let slot = usize::try_from(slot)
            .ok()
            .filter(|&v| v < PAIRS_PER_RECORD)?;
        let value = self.groups[slot / 4][slot % 4];
        Some((value >> u8::from(mode as u8 != 0)) & 1 != 0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub id: BankId,
    pub source_offset: u32,
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata12 {
    pub banks: [Bank; 2],
}
impl Metadata12 {
    pub fn validate(&self) -> Result<(), String> {
        for (bank, id) in self.banks.iter().zip([BankId::Primary, BankId::Secondary]) {
            if bank.id != id
                || bank.source_offset != id.source_offset()
                || bank.records.len() != id.record_count()
            {
                return Err("Invalid fixed EUR metadata12 bank/order/extent/count".into());
            }
            for (i, r) in bank.records.iter().enumerate() {
                if r.id as usize != i
                    || r.source_offset != bank.source_offset + (i * RECORD_BYTES) as u32
                    || r.groups.iter().flatten().any(|&v| v > 3)
                {
                    return Err("Invalid metadata12 ID/order/source/two-bit field".into());
                }
            }
        }
        Ok(())
    }
    /// None is outside the owned catalog, never an emulated return value.
    pub fn record(&self, bank: BankId, row: u32) -> Option<&Record> {
        self.banks[bank.index()]
            .records
            .get(usize::try_from(row).ok()?)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.banks
            .iter()
            .flat_map(|b| b.records.iter().flat_map(Record::source_bytes))
            .collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Metadata12 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: Metadata12,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Metadata12 requires exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        for &(pc, at, source) in REFERENCES {
            let v = half(pc as usize);
            if v & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
                || u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                    != 0x08000000 + source
            {
                return Err("Metadata12 original source/next literal mismatch".into());
            }
        }
        for (pc, v) in [
            (0x137628, 0x0058),
            (0x13762a, 0x18c0),
            (0x13762c, 0x0080),
            (0x13764c, 0x0058),
            (0x13764e, 0x18c0),
            (0x137650, 0x0080),
            (0x135a22, 0x0612),
            (0x135a24, 0x005b),
            (0x135a26, 0x2a00),
            (0x135a2a, 0x3301),
            (0x135a2c, 0x10d8),
            (0x135a34, 0x7800),
            (0x135a36, 0x4108),
            (0x135a3a, 0x4008),
            (0x137632, 0xf7fe),
            (0x137634, 0xf9f3),
            (0x137656, 0xf7fe),
            (0x137658, 0xf9e1),
            (0x1376a4, 0x2c2b),
            (0x137738, 0x2c2b),
            (0x137790, 0x2c2b),
            (0x137822, 0x2c2b),
        ] {
            if half(pc) != v {
                return Err("Metadata12 original stride/reader/call/loop mismatch".into());
            }
        }
        let banks = [BankId::Primary, BankId::Secondary].map(|id| Bank {
            id,
            source_offset: id.source_offset(),
            records: (0..id.record_count())
                .map(|i| {
                    let at = id.source_offset() + (i * RECORD_BYTES) as u32;
                    let bytes = &rom[at as usize..at as usize + RECORD_BYTES];
                    Record {
                        id: i as u16,
                        source_offset: at,
                        groups: std::array::from_fn(|j| {
                            std::array::from_fn(|k| (bytes[j] >> (k * 2)) & 3)
                        }),
                    }
                })
                .collect(),
        });
        let decoded = Metadata12 { banks };
        decoded.validate()?;
        let sources = decoded
            .banks
            .iter()
            .map(|b| SourceRange {
                offset: b.source_offset,
                length: (b.records.len() * RECORD_BYTES) as u32,
                sha256: ffta_rom::digest(
                    &rom[b.source_offset as usize
                        ..b.source_offset as usize + b.records.len() * RECORD_BYTES],
                ),
                role: match b.id {
                    BankId::Primary => "metadata12/primary-records",
                    BankId::Secondary => "metadata12/secondary-records",
                }
                .into(),
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}
