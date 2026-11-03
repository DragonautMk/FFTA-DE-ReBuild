//! Ordered flag keys for original global/context counters; no RAM or guest stores.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e9a5c;
pub const END_OFFSET: u32 = 0x6e9cb6;
pub const ENTRY_COUNT: usize = 300;
pub const TERMINATOR_OFFSET: u32 = END_OFFSET - 2;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: u16,
    pub source_offset: u32,
    pub key: u16,
}
impl Entry {
    pub fn flag_key(&self) -> u16 {
        self.key.wrapping_add(0x2ff)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terminator {
    pub source_offset: u32,
    pub value: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateFlagKeys {
    pub source_offset: u32,
    pub entries: Vec<Entry>,
    pub terminator: Terminator,
}
impl AggregateFlagKeys {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET
            || self.entries.len() != ENTRY_COUNT
            || self.terminator.source_offset != TERMINATOR_OFFSET
            || self.terminator.value != 0xffff
        {
            return Err("Invalid fixed EUR aggregate flag keys extent/terminator".into());
        }
        for (i, e) in self.entries.iter().enumerate() {
            if e.id as usize != i || e.source_offset != SOURCE_OFFSET + (i * 2) as u32 {
                return Err("Invalid aggregate flag key ID/order/source".into());
            }
        }
        Ok(())
    }
    pub fn entry(&self, id: u32) -> Option<&Entry> {
        self.entries.get(usize::try_from(id).ok()?)
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        let word = if offset / 2 == ENTRY_COUNT {
            self.terminator.value
        } else {
            self.entries.get(offset / 2)?.key
        };
        (offset < (END_OFFSET - SOURCE_OFFSET) as usize)
            .then_some((word >> ((offset & 1) * 8)) as u8)
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        self.entries
            .iter()
            .flat_map(|e| e.key.to_le_bytes())
            .chain(self.terminator.value.to_le_bytes())
            .collect()
    }
    /// Diagnostic only. Original flag36/linked-list predicates and u16 return stay active.
    pub fn count(&self, mut flag: impl FnMut(u16) -> bool, linked_match: bool) -> u16 {
        let mut count = 0u16;
        for e in self.entries.iter().take(ENTRY_COUNT) {
            if e.key == 0xffff {
                break;
            }
            count += u16::from(flag(e.flag_key()));
        }
        count + u16::from(flag(0x36) || linked_match)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Aggregate flag keys differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: AggregateFlagKeys,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Aggregate flag keys require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (pc, at, value) in [
            (0xd2300u32, 0xd2344u32, SOURCE_OFFSET + 0x08000000),
            (0x13fb22, 0x13fb74, SOURCE_OFFSET + 0x08000000),
            (0x29af4, 0x29b0c, END_OFFSET + 0x08000000),
        ] {
            let op = half(pc as usize);
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(op & 255) * 4 != at
                || word(at as usize) != value
            {
                return Err("Aggregate flag key original/next literal mismatch".into());
            }
        }
        for (pc, value) in [
            (0xd2302, 0x8810),
            (0xd2306, 0x4288),
            (0xd2308, 0xd00f),
            (0xd2310, 0x1840),
            (0xd2312, 0x0400),
            (0xd2314, 0x0c00),
            (0xd2322, 0x3402),
            (0xd2324, 0x8820),
            (0xd2328, 0xd1f1),
            (0xd232a, 0x2036),
            (0xd2368, 0x4020),
            (0xd236e, 0x4298),
            (0xd2372, 0x6892),
            (0xd2380, 0x0428),
            (0xd2382, 0x0c00),
            (0x13fb24, 0x8810),
            (0x13fb2a, 0xd010),
            (0x13fb34, 0x0400),
            (0x13fb36, 0x0c00),
            (0x13fb38, 0x1c31),
            (0x13fb46, 0x3402),
            (0x13fb48, 0x8820),
            (0x13fb4c, 0xd1f0),
        ] {
            if half(pc) != value {
                return Err("Aggregate flag key original loop/arithmetic mismatch".into());
            }
        }
        if word(0xd2348) != 0xffff
            || word(0x13fb78) != 0xffff
            || word(0xd234c) != 0x2ff
            || word(0x13fb7c) != 0x2ff
            || word(0xccec0) != 0x02001f70
            || word(0xccf58) != 0x1f70
        {
            return Err("Aggregate flag key original terminator/flag binding mismatch".into());
        }
        let decoded = AggregateFlagKeys {
            source_offset: SOURCE_OFFSET,
            entries: (0..ENTRY_COUNT)
                .map(|i| {
                    let at = SOURCE_OFFSET + (i * 2) as u32;
                    Entry {
                        id: i as u16,
                        source_offset: at,
                        key: half(at as usize),
                    }
                })
                .collect(),
            terminator: Terminator {
                source_offset: TERMINATOR_OFFSET,
                value: half(TERMINATOR_OFFSET as usize),
            },
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: END_OFFSET - SOURCE_OFFSET,
                sha256: ffta_rom::digest(&rom[SOURCE_OFFSET as usize..END_OFFSET as usize]),
                role: "aggregate-flag-keys/eur-ordered-keys".into(),
            }],
        })
    }
}
