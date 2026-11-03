//! Five EUR descriptor records shared by the original price calculation.
//! Structural fields preserve the byte41 alias; RAM flags and guest code remain live.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e85bc;
pub const END_OFFSET: u32 = 0x6e879c;
pub const RECORD_BYTES: usize = 96;
pub const RECORD_COUNT: usize = 5;
pub const FIRST_ID: u8 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub source_offset: u32,
    pub bytes00_1f: [u8; 32],
    pub bytes20_3f: [u8; 32],
    pub markup40_41: u16,
    pub discounts42_5f: [u8; 30],
}
impl Record {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        match offset {
            0..=31 => Some(self.bytes00_1f[offset]),
            32..=63 => Some(self.bytes20_3f[offset - 32]),
            64..=65 => Some(self.markup40_41.to_le_bytes()[offset - 64]),
            66..=95 => Some(self.discounts42_5f[offset - 66]),
            _ => None,
        }
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Count0 reads byte41, also the high byte of markup. No duplicate owner.
    pub fn discount(&self, count: u8) -> Option<u8> {
        match count {
            0 => Some(self.markup40_41.to_le_bytes()[1]),
            1..=30 => Some(self.discounts42_5f[usize::from(count - 1)]),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TradeParameters {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl TradeParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR trade parameter extent".into());
        }
        for (i, r) in self.records.iter().enumerate() {
            if r.id != FIRST_ID + i as u8
                || r.source_offset != SOURCE_OFFSET + (RECORD_BYTES * i) as u32
            {
                return Err("Invalid trade parameter ID/order/source".into());
            }
        }
        Ok(())
    }
    pub fn record(&self, id: u32) -> Option<&Record> {
        self.records
            .get(usize::try_from(id.checked_sub(u32::from(FIRST_ID))?).ok()?)
    }
    /// Original D0744 normalizes low8, returning null outside IDs2..6.
    pub fn descriptor(&self, selector: u32) -> u32 {
        self.record(u32::from(selector as u8))
            .map_or(0, |r| 0x08000000 + r.source_offset)
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.records
            .get(offset / RECORD_BYTES)?
            .byte(offset % RECORD_BYTES)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    /// Diagnostic only. D1F30, live flags, division, stores and clocks stay guest code.
    /// None denotes unowned source access, never a guest fallback or invented price.
    pub fn price(&self, base: u32, selector: u32, count: u32, flags: u8) -> Option<u32> {
        if base == 0 {
            return Some(0); // Original returns before reading these parameters or RAM flags.
        }
        let r = self.record(u32::from(selector as u8))?;
        if flags & 3 == 3 {
            let percentage = u32::from(r.discount(count as u8)?);
            Some(base.wrapping_sub(base.wrapping_mul(percentage) / 100))
        } else {
            Some(base.wrapping_add(base.wrapping_mul(u32::from(r.markup40_41)) / 100))
        }
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Trade parameters differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: TradeParameters,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Trade parameters require exact EUR ROM; USA pending".into());
        }
        let half = |at| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for i in 0..RECORD_COUNT {
            let pc = 0xd0774 + i * 8;
            let op = half(pc);
            if word(0xd0760 + i * 4) != 0x08000000 + pc as u32
                || op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + usize::from(op & 255) * 4 != pc + 4
                || word(pc + 4) != 0x08000000 + SOURCE_OFFSET + (i * RECORD_BYTES) as u32
            {
                return Err("Trade parameter five-root selector mismatch".into());
            }
        }
        for (pc, at, target) in [
            (0xd26d0, 0xd2700, SOURCE_OFFSET - 2 * RECORD_BYTES as u32),
            (0xcfde2, 0xcfe14, END_OFFSET),
        ] {
            let op = half(pc);
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + usize::from(op & 255) * 4 != at
                || word(at) != 0x08000000 + target
            {
                return Err("Trade parameter biased/next root mismatch".into());
            }
        }
        for (pc, op) in [
            (0xd0746, 0x0600),
            (0xd0748, 0x0e00),
            (0xd074a, 0x3802),
            (0xd074c, 0x2804),
            (0xd074e, 0xd825),
            (0xd079c, 0x2000),
            (0xd26b2, 0x0609),
            (0xd26b4, 0x0e0c),
            (0xd26b6, 0x0612),
            (0xd26b8, 0x0e16),
            (0xd26ba, 0x2136),
            (0xd26c2, 0x2d00),
            (0xd26ca, 0x0060),
            (0xd26cc, 0x1900),
            (0xd26ce, 0x0142),
            (0xd26da, 0x7841),
            (0xd26ec, 0x1990),
            (0xd26ee, 0x3041),
            (0xd26f0, 0x7800),
            (0xd26f2, 0x4368),
            (0xd26f4, 0x2164),
            (0xd26fa, 0x1a28),
            (0xd270a, 0x3040),
            (0xd270c, 0x7801),
            (0xd2710, 0x7800),
            (0xd2716, 0x4368),
            (0xd2718, 0x2164),
            (0xd271e, 0x1828),
            (0xd2732, 0x221d),
            (0xd2748, 0x330c),
            (0xd274a, 0x3a01),
            (0xd274e, 0xdaf1),
        ] {
            if half(pc) != op {
                return Err("Trade parameter original stride/field/count mismatch".into());
            }
        }
        if word(0xd2704) != 0x02002cb4 {
            return Err("Trade parameter live flag root mismatch".into());
        }
        let decoded = TradeParameters {
            source_offset: SOURCE_OFFSET,
            records: (0..RECORD_COUNT)
                .map(|i| {
                    let at = SOURCE_OFFSET + (i * RECORD_BYTES) as u32;
                    let row = &rom[at as usize..at as usize + RECORD_BYTES];
                    Record {
                        id: FIRST_ID + i as u8,
                        source_offset: at,
                        bytes00_1f: row[..32].try_into().unwrap(),
                        bytes20_3f: row[32..64].try_into().unwrap(),
                        markup40_41: u16::from_le_bytes(row[64..66].try_into().unwrap()),
                        discounts42_5f: row[66..].try_into().unwrap(),
                    }
                })
                .collect(),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: END_OFFSET - SOURCE_OFFSET,
                sha256: ffta_rom::digest(&rom[SOURCE_OFFSET as usize..END_OFFSET as usize]),
                role: "trade-parameters/eur-fixed-five".into(),
            }],
        })
    }
}
