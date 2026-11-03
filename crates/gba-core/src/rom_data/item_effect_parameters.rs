//! Fourteen packed EUR records indexed by original item IDs362..375.
//! Live current values, item use and original guest instructions remain active.
use super::{items, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x6e879c;
pub const END_OFFSET: u32 = 0x6e87fe;
pub const RECORD_BYTES: usize = 7;
pub const RECORD_COUNT: usize = 14;
pub const FIRST_ITEM_ID: u16 = 362;
pub const PROPERTY_COEFFICIENT: [usize; 20] =
    [0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u8,
    pub item_id: u16,
    pub source_offset: u32,
    pub coefficients: [u8; 9],
    pub cap45_52: u8,
    pub opaque53_55: u8,
}
impl Record {
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        let packed = self
            .coefficients
            .iter()
            .enumerate()
            .fold(0u64, |bits, (i, &v)| bits | (u64::from(v) << (i * 5)))
            | (u64::from(self.cap45_52) << 45)
            | (u64::from(self.opaque53_55) << 53);
        packed.to_le_bytes()[..RECORD_BYTES].try_into().unwrap()
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.source_bytes().get(offset).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemEffectParameters {
    pub source_offset: u32,
    pub records: Vec<Record>,
}
impl ItemEffectParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.records.len() != RECORD_COUNT {
            return Err("Invalid fixed EUR item effect parameter extent".into());
        }
        for (i, r) in self.records.iter().enumerate() {
            if usize::from(r.id) != i
                || r.item_id != FIRST_ITEM_ID + i as u16
                || r.source_offset != SOURCE_OFFSET + (RECORD_BYTES * i) as u32
                || r.coefficients.iter().any(|&v| v > 31)
                || r.opaque53_55 > 7
            {
                return Err("Invalid item effect ID/item/order/source/packed width".into());
            }
        }
        Ok(())
    }
    pub fn validate_items(&self, owner: &items::Items) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if self
            .records
            .iter()
            .any(|r| owner.records.get(usize::from(r.item_id)).is_none())
        {
            return Err("Missing original item effect item owner".into());
        }
        Ok(())
    }
    pub fn record(&self, id: u32) -> Option<&Record> {
        self.records.get(usize::try_from(id).ok()?)
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.records
            .get(offset / RECORD_BYTES)?
            .byte(offset % RECORD_BYTES)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(Record::source_bytes).collect()
    }
    /// Diagnostic CFE28: both arguments use low8; invalid property returns before source.
    /// None denotes a source access outside the declared owner, never a guest fallback.
    pub fn property(&self, property: u32, index: u32) -> Option<u32> {
        let Some(&coefficient) = PROPERTY_COEFFICIENT.get(usize::from(property as u8)) else {
            return Some(0);
        };
        Some(u32::from(
            self.record(u32::from(index as u8))?.coefficients[coefficient],
        ))
    }
    /// Diagnostic CFDA8. The cap uses full(item−362)u16, unlike the low8 getter index.
    /// Live current value is an input, not extracted state or a host write.
    pub fn delta(&self, slot: u32, item: u32, quantity: u32, current: u8) -> Option<u16> {
        let index = (item as u16).wrapping_sub(FIRST_ITEM_ID);
        let r = self.record(u32::from(index))?;
        let delta = self.property(slot, u32::from(index))? * u32::from(quantity as u8);
        let cap = u32::from(r.cap45_52);
        let current = u32::from(current);
        Some(if delta + current <= cap {
            delta as u16
        } else if current >= cap {
            0
        } else {
            (cap - current) as u16
        })
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Item effect parameters differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ItemEffectParameters,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Item effect parameters require exact EUR ROM; USA pending".into());
        }
        let half = |at| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (pc, at, target) in [
            (0x5aae2, 0x5ab88, SOURCE_OFFSET),
            (0xcfde2, 0xcfe14, SOURCE_OFFSET),
            (0xcfe98, 0xcfea4, SOURCE_OFFSET),
            (0xcfea8, 0xcfec0, SOURCE_OFFSET),
            (0xcfec4, 0xcfed4, SOURCE_OFFSET),
            (0xcfed8, 0xcfef0, SOURCE_OFFSET),
            (0xcfef4, 0xcff0c, SOURCE_OFFSET),
            (0xcff10, 0xcff20, SOURCE_OFFSET),
            (0xcff24, 0xcff3c, SOURCE_OFFSET),
            (0xcff40, 0xcff50, SOURCE_OFFSET),
            (0xcff54, 0xcff64, SOURCE_OFFSET),
            (0xcf9ec, 0xcfa24, END_OFFSET),
            (0xcfa30, 0xcfa60, END_OFFSET),
            (0xcfa70, 0xcfaa0, END_OFFSET),
            (0xcfaac, 0xcfaf8, END_OFFSET),
        ] {
            let op = half(pc);
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + usize::from(op & 255) * 4 != at
                || word(at) != 0x08000000 + target
            {
                return Err("Item effect original/next literal mismatch".into());
            }
        }
        let targets = [
            0xcfe98, 0xcfea8, 0xcfec4, 0xcfed8, 0xcfef4, 0xcff10, 0xcff24, 0xcff40, 0xcff54,
        ];
        if PROPERTY_COEFFICIENT
            .iter()
            .enumerate()
            .any(|(i, &k)| word(0xcfe48 + i * 4) != 0x08000000 + targets[k])
            || word(0xcfe08) != 0xfe960000
            || word(0x5ab84) != 0xfffffe96
            || word(0xcf86c) != 361
            || word(0xcec88) != 361
            || word(0xcec8c) != 375
        {
            return Err(
                "Item effect original properties/item enumeration/subtraction mismatch".into(),
            );
        }
        for (pc, op) in [
            (0xcf876, 0x200e),
            (0xcfdb0, 0x0624),
            (0xcfdb2, 0x0e24),
            (0xcfdb4, 0x042d),
            (0xcfdbe, 0x0c2d),
            (0xcfdc0, 0x0629),
            (0xcfdc2, 0x0e09),
            (0xcfdd0, 0x4377),
            (0xcfdd4, 0x0124),
            (0xcfddc, 0x7826),
            (0xcfde4, 0x00e8),
            (0xcfde6, 0x1b40),
            (0xcfdea, 0x7942),
            (0xcfdec, 0x0952),
            (0xcfdee, 0x7980),
            (0xcfdf0, 0x211f),
            (0xcfdf4, 0x00c0),
            (0xcfdf6, 0x4310),
            (0xcfdfa, 0x428b),
            (0xcfdfc, 0xdd10),
            (0xcfe00, 0xd30a),
            (0xcfe18, 0x1b80),
            (0xcfe1a, 0x0400),
            (0xcfe1c, 0x0c00),
            (0xcfe2a, 0x0600),
            (0xcfe2c, 0x0e00),
            (0xcfe2e, 0x0609),
            (0xcfe30, 0x0e0a),
            (0xcfe32, 0x2813),
            (0xcfe34, 0xd900),
            (0xcff68, 0x2000),
        ] {
            if half(pc) != op {
                return Err("Item effect original index/packed/clamp mismatch".into());
            }
        }
        let decoded = ItemEffectParameters {
            source_offset: SOURCE_OFFSET,
            records: (0..RECORD_COUNT)
                .map(|i| {
                    let at = SOURCE_OFFSET + (i * RECORD_BYTES) as u32;
                    let mut bytes = [0; 8];
                    bytes[..RECORD_BYTES]
                        .copy_from_slice(&rom[at as usize..at as usize + RECORD_BYTES]);
                    let bits = u64::from_le_bytes(bytes);
                    Record {
                        id: i as u8,
                        item_id: FIRST_ITEM_ID + i as u16,
                        source_offset: at,
                        coefficients: std::array::from_fn(|k| ((bits >> (k * 5)) & 31) as u8),
                        cap45_52: (bits >> 45) as u8,
                        opaque53_55: (bits >> 53) as u8,
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
                role: "item-effect-parameters/eur-packed14".into(),
            }],
        })
    }
}
