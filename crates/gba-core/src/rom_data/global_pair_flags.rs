//! Minimum global packed-pair extent proved by the original live-mask loop.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e7994;
pub const END_OFFSET: u32 = 0x6e799f;
pub const NEXT_SOURCE: u32 = 0x6e79a2;
pub const PAIRS: usize = 44;
pub const SOURCE_BYTES: usize = 11;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalPairFlags {
    pub source_offset: u32,
    /// Four two-bit fields in each source byte, least significant first.
    pub groups: [[u8; 4]; SOURCE_BYTES],
}
impl GlobalPairFlags {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.groups.iter().flatten().any(|&v| v > 3) {
            return Err("Invalid fixed EUR global pair source/two-bit value".into());
        }
        Ok(())
    }
    pub fn byte(&self, at: usize) -> Option<u8> {
        Some(
            self.groups
                .get(at)?
                .iter()
                .enumerate()
                .fold(0, |v, (i, p)| v | (p << (i * 2))),
        )
    }
    pub fn source_bytes(&self) -> [u8; SOURCE_BYTES] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
    /// Finite diagnostic view only; the guest retains raw access outside0..43.
    pub fn flag(&self, slot: u32, mode: u32) -> Option<bool> {
        let slot = usize::try_from(slot).ok().filter(|&v| v < PAIRS)?;
        Some((self.groups[slot / 4][slot % 4] >> u8::from(mode as u8 != 0)) & 1 != 0)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Global pair flags differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: GlobalPairFlags,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Global pair flags require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        for (pc, at, target) in [
            (0x1377a2u32, 0x1377b0u32, SOURCE_OFFSET),
            (0xd2db0, 0xd2f20, NEXT_SOURCE),
        ] {
            let v = half(pc as usize);
            if v & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(v & 255) * 4 != at
                || u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
                    != 0x08000000 + target
            {
                return Err("Global pair flags original/next literal mismatch".into());
            }
        }
        for (pc, v) in [
            (0x13779e, 0x0612),
            (0x1377a0, 0x0e12),
            (0x1377a4, 0xf7fe),
            (0x1377a6, 0xf93a),
            (0x1377dc, 0x1c21),
            (0x1377de, 0x2200),
            (0x1377e0, 0xf7ff),
            (0x1377e2, 0xffdc),
            (0x1377f0, 0x2c2b),
            (0x1377f2, 0xdde3),
            (0xd2db4, 0x7821),
            (0xd2dbe, 0x7861),
            (0x135a22, 0x0612),
            (0x135a24, 0x005b),
            (0x135a2a, 0x3301),
            (0x135a2c, 0x10d8),
            (0x135a34, 0x7800),
        ] {
            if half(pc) != v {
                return Err("Global pair flags original reader/call/loop mismatch".into());
            }
        }
        let bytes = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = GlobalPairFlags {
            source_offset: SOURCE_OFFSET,
            groups: std::array::from_fn(|i| std::array::from_fn(|j| (bytes[i] >> (j * 2)) & 3)),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: SOURCE_BYTES as u32,
                sha256: ffta_rom::digest(bytes),
                role: "global-pair-flags/eur-fields".into(),
            }],
        })
    }
}
