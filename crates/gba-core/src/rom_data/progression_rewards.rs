//! Bounded progression deltas and live price discount; original routines remain.
use super::{items, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e79da;
pub const END_OFFSET: u32 = 0x6e7dc2;
pub const NEXT_SOURCE: u32 = 0x6e7dce;
pub const RECORD_COUNT: usize = 100;
pub const RECORD_BYTES: usize = 10;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub deltas00_07: [u8; 8],
    pub discount08: u8,
    pub byte09: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressOutcome {
    pub level: u8,
    pub progress: u8,
    pub flags: u16,
}
impl Record {
    pub fn byte(&self, at: usize) -> Option<u8> {
        match at {
            0..=7 => Some(self.deltas00_07[at]),
            8 => Some(self.discount08),
            9 => Some(self.byte09),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Diagnostic caller1..8 contract; does not replace original RAM/flag work.
    pub fn apply_delta(
        &self,
        index: usize,
        old_level: u8,
        old_progress: u8,
        growth: u8,
        old_flags: u16,
    ) -> Option<ProgressOutcome> {
        let delta = *self.deltas00_07.get(index)?;
        if old_level > 98 {
            return Some(ProgressOutcome {
                level: old_level,
                progress: 0,
                flags: old_flags,
            });
        }
        let mut level = old_level;
        let mut progress = old_progress.wrapping_add(growth);
        let mut flags = old_flags;
        let bit = 1u16 << (index + 1);
        if progress > 99 {
            progress %= 100;
            level = level.wrapping_add(1);
            flags |= bit;
        }
        if delta != 0 {
            level = level.wrapping_add(delta);
            flags |= bit;
        }
        if level > 98 {
            level = 99;
            progress = 0;
        }
        Some(ProgressOutcome {
            level,
            progress,
            flags,
        })
    }
    pub fn price(&self, item: &items::Record, predicate: u32) -> u32 {
        price_with_discount(item, self.discount08, predicate)
    }
}
/// Diagnostic scalar result. Original mode/item truncation and live predicate run in guest code.
pub fn price_with_discount(item: &items::Record, discount: u8, predicate: u32) -> u32 {
    let base = item.halfwords_00_06[2];
    let minimum = item.halfwords_00_06[3];
    let mut price = if item.bytes_08_0f[4] & 8 != 0 {
        let discount = discount.wrapping_add(if predicate as u8 != 0 { 10 } else { 0 });
        let product = i32::from(base).wrapping_mul(100 - i32::from(discount));
        ((product / 100) as u32).max(u32::from(minimum))
    } else {
        u32::from(base)
    };
    if price == 0 {
        price = 1;
    }
    price
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionRewards {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl ProgressionRewards {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR progression reward extent/count".into());
        }
        for (i, r) in self.records.iter().enumerate() {
            if r.id as usize != i || r.source_offset != SOURCE_OFFSET + (i * RECORD_BYTES) as u32 {
                return Err("Invalid progression reward ID/order/source".into());
            }
        }
        Ok(())
    }
    /// None denotes unowned IDs. Original raw out-of-range access is unchanged.
    pub fn record(&self, id: u32) -> Option<&Record> {
        self.records.get(usize::try_from(id).ok()?)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Progression rewards differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: ProgressionRewards,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Progression rewards require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        for (pc, at, target) in [
            (0xcf68eu32, 0xcf6d8u32, SOURCE_OFFSET),
            (0x4b45c, 0x4b49c, SOURCE_OFFSET),
            (0x4b4b2, 0x4b4dc, SOURCE_OFFSET),
            (0x487f6, 0x48868, NEXT_SOURCE),
            (0xcf67e, 0xcf6d4, items::SOURCE_OFFSET),
        ] {
            let v = half(pc as usize);
            if v & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
                || u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                    != 0x08000000 + target
            {
                return Err("Progression original/next/item literal mismatch".into());
            }
        }
        for (pc, v) in [
            (0xcf698, 0x0088),
            (0xcf69a, 0x1840),
            (0xcf69c, 0x0040),
            (0xcf69e, 0x3208),
            (0xcf6a2, 0x7804),
            (0xcf684, 0x7b29),
            (0xcf686, 0x2008),
            (0xcf6b4, 0x300a),
            (0xcf6b6, 0x0600),
            (0xcf6b8, 0x0e04),
            (0xcf6bc, 0x2164),
            (0xcf6be, 0x1b09),
            (0xcf6c0, 0x4348),
            (0xcf6cc, 0xd20b),
            (0xcf6e6, 0x2800),
            (0xcf6ea, 0x2001),
            (0x4b456, 0x00a0),
            (0x4b458, 0x1900),
            (0x4b45a, 0x0040),
            (0x4b460, 0x6010),
            (0x4b3e4, 0x2c62),
            (0x4b3e6, 0xd823),
            (0x4b3fc, 0x2a63),
            (0x4b40c, 0x1c60),
            (0x4b416, 0x7034),
            (0x4873e, 0x2c62),
            (0x487a4, 0x2c62),
            (0x487a8, 0x2463),
        ] {
            if half(pc) != v {
                return Err(
                    "Progression original selector/field/price/live-level reader mismatch".into(),
                );
            }
        }
        for (pc, field) in [
            (0x4b56e, 0),
            (0x4b580, 1),
            (0x4b592, 2),
            (0x4b5a4, 3),
            (0x4b5b6, 4),
            (0x4b5c8, 5),
            (0x4b5da, 6),
            (0x4b5f0, 7),
        ] {
            if half(pc) != 0x7809 + (field << 6) {
                return Err("Progression original delta field mismatch".into());
            }
        }
        let records = (0..RECORD_COUNT)
            .map(|i| {
                let at = SOURCE_OFFSET + (i * RECORD_BYTES) as u32;
                let bytes = &rom[at as usize..at as usize + RECORD_BYTES];
                Record {
                    id: i as u8,
                    source_offset: at,
                    deltas00_07: bytes[..8].try_into().unwrap(),
                    discount08: bytes[8],
                    byte09: bytes[9],
                }
            })
            .collect();
        let decoded = ProgressionRewards {
            source_offset: SOURCE_OFFSET,
            records,
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: END_OFFSET - SOURCE_OFFSET,
                sha256: ffta_rom::digest(&rom[SOURCE_OFFSET as usize..END_OFFSET as usize]),
                role: "progression-rewards/eur-records".into(),
            }],
        })
    }
}
