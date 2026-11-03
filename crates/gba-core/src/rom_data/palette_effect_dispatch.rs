//! Producer-bounded palette callback projection; original guest RAM and dispatch stay live.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0xb94690;
pub const PROJECTION_ENTRIES: usize = 17;
pub const SOURCE_BYTES: usize = PROJECTION_ENTRIES * 4;
pub const PRODUCERS: [(u16, u32, u16, u32, u16); 5] = [
    (1, 0x14a954, 0x2001, 0x14a956, 0x80b8),
    (2, 0x14aa12, 0x2002, 0x14aa14, 0x80a0),
    (4, 0x14aacc, 0x2004, 0x14aace, 0x80a0),
    (8, 0x14ab88, 0x2008, 0x14ab8a, 0x80a0),
    (16, 0x14ac30, 0x2010, 0x14ac32, 0x80a0),
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThumbEntry {
    pub rom_offset: u32,
}
impl ThumbEntry {
    pub fn guest_word(&self) -> Option<u32> {
        (self.rom_offset < 0x1000000 && self.rom_offset & 1 == 0)
            .then(|| 0x08000001 + self.rom_offset)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub kind: u16,
    pub target: Option<ThumbEntry>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteEffectDispatch {
    pub source_offset: u32,
    pub slots: Vec<Slot>,
}
impl PaletteEffectDispatch {
    /// Physical projection membership, never a guard on live object.u16(4).
    pub fn callback_word(&self, kind: u16) -> Option<u32> {
        match &self.slots.get(usize::from(kind))?.target {
            Some(t) => t.guest_word(),
            None => Some(0),
        }
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        if offset >= SOURCE_BYTES {
            return None;
        }
        Some(self.callback_word((offset / 4) as u16)?.to_le_bytes()[offset % 4])
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.slots.len() != PROJECTION_ENTRIES {
            return Err("Palette effect dispatch projection source/count mismatch".into());
        }
        for (i, s) in self.slots.iter().enumerate() {
            if usize::from(s.kind) != i
                || s.target.is_some() != matches!(i, 1 | 2 | 4 | 8 | 16)
                || s.target.as_ref().is_some_and(|t| t.guest_word().is_none())
            {
                return Err("Palette effect dispatch kind/null/Thumb target mismatch".into());
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        let bytes = (0..SOURCE_BYTES)
            .map(|n| {
                self.byte(n)
                    .ok_or_else(|| "Invalid palette dispatch byte".into())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(vec![(
            self.source_offset,
            bytes,
            "graphics/palette-effect-dispatch/projection-0-16".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Decoded palette dispatch differs from authenticated source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: PaletteEffectDispatch,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Palette dispatch requires exact European source ROM".into());
        }
        let h = |at: u32| u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
        for (pc, op) in [
            (0x14ac62, 0x8820),
            (0x14ac64, 0x2801),
            (0x14ac66, 0xd10f),
            (0x14ac68, 0x4806),
            (0x14ac6a, 0x88a1),
            (0x14ac6c, 0x0089),
            (0x14ac6e, 0x1809),
            (0x14ac70, 0x6809),
        ] {
            if h(pc) != op {
                return Err("Palette dispatch state/index/width contract mismatch".into());
            }
        }
        for (_, pc, mov, store, strh) in PRODUCERS {
            if h(pc) != mov || h(store) != strh {
                return Err("Palette dispatch fixed producer mismatch".into());
            }
        }
        if u32::from_le_bytes(rom[0x14ac84..0x14ac88].try_into().unwrap())
            != 0x08000000 + SOURCE_OFFSET
        {
            return Err("Palette dispatch literal mismatch".into());
        }
        let mut slots = Vec::new();
        for kind in 0..PROJECTION_ENTRIES {
            let at = SOURCE_OFFSET as usize + kind * 4;
            let word = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
            let target = if word == 0 {
                None
            } else {
                if word & 1 == 0 || !(0x08000000..0x09000000).contains(&word) {
                    return Err("Palette callback is not a ROM Thumb entry".into());
                }
                Some(ThumbEntry {
                    rom_offset: (word & !1) - 0x08000000,
                })
            };
            slots.push(Slot {
                kind: kind as u16,
                target,
            });
        }
        let decoded = PaletteEffectDispatch {
            source_offset: SOURCE_OFFSET,
            slots,
        };
        decoded.validate()?;
        let (offset, bytes, role) = decoded.source_spans()?.pop().unwrap();
        if rom[offset as usize..offset as usize + bytes.len()] != bytes {
            return Err("Palette dispatch source roundtrip mismatch".into());
        }
        let sources = vec![SourceRange {
            offset,
            length: bytes.len() as u32,
            sha256: ffta_rom::digest(&bytes),
            role,
        }];
        Ok(Self { decoded, sources })
    }
}
