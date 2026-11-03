//! Two ordinary EUR ROM list arguments outside the direct-literal catalog.
//! This isolated model does not change the published catalog or guest bounds.
use super::{List, Lists};
use crate::{ffta_rom, rom_data::SourceRange};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// Original literal -> copied constructor -> instance list -> copied draw.
    ConstructorBound,
    /// Actual ordinary task+24 -> 02E324/01D28 -> original ROM draw argument.
    /// The immutable command initializer is not established by this evidence.
    OrdinaryTaskArgument,
}

#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub source_offset: u32,
    pub object_count: usize,
    pub source_sha256: &'static str,
    pub evidence: Evidence,
}

pub const ROOTS: [Root; 2] = [
    Root {
        source_offset: 0x394f4c,
        object_count: 4,
        source_sha256: "03df6410f4e051bdff4603ac5daea36c51c6acb95442d80c5b08c45a64a18477",
        evidence: Evidence::ConstructorBound,
    },
    Root {
        source_offset: 0x40d89e,
        object_count: 6,
        source_sha256: "bc84a0ac1f2037a71f7e347f442a0ee909b5fdfffcf8bb209127a38e71daeb61",
        evidence: Evidence::OrdinaryTaskArgument,
    },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedLists {
    pub lists: Vec<List>,
}

impl ObservedLists {
    pub fn validate(&self) -> Result<(), String> {
        if self.lists.len() != ROOTS.len() {
            return Err("Observed OBJ catalog needs its two declared roots".into());
        }
        for (list, root) in self.lists.iter().zip(ROOTS) {
            list.validate()?;
            if list.source_offset != root.source_offset
                || list.objects.len() != root.object_count
                || list.objects.iter().any(|object| object.shape >= 3)
            {
                return Err("Observed OBJ root/count/available shape mismatch".into());
            }
        }
        Lists {
            lists: self.lists.clone(),
        }
        .validate()
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        self.lists
            .iter()
            .map(|list| Ok((list.source_offset, list.source_bytes()?, "static-obj-list")))
            .collect()
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.lists
            .iter()
            .find_map(|list| list.source_byte(offset.checked_sub(list.source_offset)? as usize))
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Observed OBJ lists differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ObservedLists,
    pub sources: Vec<SourceRange>,
}

fn half(rom: &[u8], offset: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        rom.get(offset..offset + 2)
            .ok_or("Truncated observed OBJ code")?
            .try_into()
            .unwrap(),
    ))
}
fn word(rom: &[u8], offset: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        rom.get(offset..offset + 4)
            .ok_or("Truncated observed OBJ literal")?
            .try_into()
            .unwrap(),
    ))
}
fn call(rom: &[u8], pc: usize, expected: usize) -> Result<(), String> {
    let hi = half(rom, pc)?;
    let lo = half(rom, pc + 2)?;
    let displacement = ((((hi as u32 & 2047) << 12) | ((lo as u32 & 2047) << 1)) as i32) << 9 >> 9;
    if hi & 0xf800 != 0xf000
        || lo & 0xf800 != 0xf800
        || (pc as u32)
            .wrapping_add(4)
            .wrapping_add_signed(displacement)
            != expected as u32
    {
        return Err("Observed OBJ original branch mismatch".into());
    }
    Ok(())
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed OBJ lists require exact EUR ROM; USA pending".into());
        }
        // Reuse the established original reader ABI; no full-archive inference.
        for (start, end, expected) in [
            (
                0x1500,
                0x170c,
                "991e33b72f908114b349f73626f506c67f3e055981c5e85131333c1ff5f1914e",
            ),
            (
                0xd96f68,
                0xd97264,
                "ce5416f21676ea64415b11aa832634fddd97e304b2d6ae29bb0a5e42828a1afc",
            ),
        ] {
            if ffta_rom::digest(rom.get(start..end).ok_or("Truncated original OBJ reader")?)
                != expected
            {
                return Err("Observed OBJ reader identity mismatch".into());
            }
        }
        for (pc, expected) in [
            (0x26464, 0x492f),
            (0x26466, 0x6812),
            (0xd9752c, 0x1c02),
            (0xd9752e, 0x23ac),
            (0xd97530, 0x005b),
            (0xd97532, 0x18d0),
            (0xd97534, 0x6001),
            (0x2e344, 0x6a55),
            (0x2e368, 0x1c28),
            (0x145c10, 0x4710),
            (0x145c18, 0x4720),
            (0x924ea, 0x4a05),
            (0x924ec, 0x4805),
            (0x924ee, 0x1a12),
            (0x924f0, 0x4905),
            (0x924f2, 0x0292),
            (0x924f4, 0x0ad2),
        ] {
            if half(rom, pc)? != expected {
                return Err("Observed OBJ source bridge mismatch".into());
            }
        }
        for (offset, expected) in [
            (0x26520, 0x08397f9c),
            (0x26524, 0x08394f4c),
            (0x397f9c, 0x03006b6d),
            (0x92500, 0x08d97728),
            (0x92504, 0x08d968e8),
            (0x92508, 0x03005f28),
            (0x397f90, 0x030065a9),
        ] {
            if word(rom, offset)? != expected {
                return Err("Observed OBJ pointer/copy mismatch".into());
            }
        }
        for (pc, target) in [
            (0x26468, 0x145c10),
            (0x924f6, 0x145230),
            (0x2e36c, 0x1d28),
            (0x92464, 0x145c18),
        ] {
            call(rom, pc, target)?;
        }
        let mut lists = Vec::new();
        let mut sources = Vec::new();
        for root in ROOTS {
            let length = 2 + root.object_count * 6;
            let bytes = rom
                .get(root.source_offset as usize..root.source_offset as usize + length)
                .ok_or("Truncated observed OBJ source")?;
            if ffta_rom::digest(bytes) != root.source_sha256 {
                return Err("Observed OBJ source identity mismatch".into());
            }
            lists.push(List::decode(root.source_offset, bytes)?);
            sources.push(SourceRange {
                offset: root.source_offset,
                length: length as u32,
                sha256: root.source_sha256.into(),
                role: "static-obj-list".into(),
            });
        }
        let decoded = ObservedLists { lists };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}
