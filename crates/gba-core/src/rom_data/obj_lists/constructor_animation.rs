//! Conditional EUR phases passed through the copied OBJ constructor.
//! The old phase byte is unchecked: this model owns four entries, not a guest cap.
use super::{List, Lists};
use crate::{ffta_rom, rom_data::SourceRange};
use serde::{Deserialize, Serialize};

pub const TABLE_OFFSET: u32 = 0x394fbc;
pub const TABLE_LENGTH: usize = 32;
pub const TABLE_SHA256: &str = "890ee971de444ff27b2fb2a237ed92f3a850268d1caba6fef8b88b472131c90b";
pub const LIST_ROOTS: [(u32, &str); 4] = [
    (
        0x394f82,
        "0c9834636fcdd21b9ab97de6e849e812eef08cb993b636e75c86f6dc9cdeb25e",
    ),
    (
        0x394f90,
        "d7fc6a7f45b1acf4390ea4f3801439c3c367bfb50afbab2293e0f56b3db5d307",
    ),
    (
        0x394f9e,
        "154fcdf3f57f4e2feeeca347d71060d8123ae1075c45e81eee5826ab9903b9e5",
    ),
    (
        0x394fac,
        "3d5701b734f74857da53ad42b9fc08f420ae23c7a0e6c2ff1dae28b744b81e63",
    ),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    /// Unsigned word compared with the incremented/stored/reloaded u8 counter.
    /// The source establishes no time unit or u8 limit on this word.
    pub threshold: u32,
    pub list_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub source_offset: u32,
    pub frames: Vec<Frame>,
}

impl Animation {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != TABLE_OFFSET
            || self.frames.len() != LIST_ROOTS.len()
            || self
                .frames
                .iter()
                .zip(LIST_ROOTS)
                .any(|(frame, (offset, _))| frame.list_offset != offset)
        {
            return Err("Constructor OBJ animation source/phase/list identity mismatch".into());
        }
        Ok(())
    }

    pub fn validate_lists(&self, lists: &Lists) -> Result<(), String> {
        self.validate()?;
        lists.validate()?;
        if self.frames.iter().any(|frame| {
            lists
                .lists
                .iter()
                .find(|list| list.source_offset == frame.list_offset)
                .is_none_or(|list| list.objects.iter().any(|object| object.shape >= 3))
        }) {
            return Err("Constructor OBJ animation needs every referenced list".into());
        }
        Ok(())
    }

    pub fn frame(&self, phase: u8) -> Option<&Frame> {
        self.frames.get(usize::from(phase))
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(self.source_offset)? as usize;
        let frame = self.frames.get(relative / 8)?;
        let word = if relative % 8 < 4 {
            frame.threshold
        } else {
            0x08000000 + frame.list_offset
        };
        Some(word.to_le_bytes()[relative % 4])
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let mut bytes = Vec::with_capacity(TABLE_LENGTH);
        for frame in &self.frames {
            bytes.extend_from_slice(&frame.threshold.to_le_bytes());
            bytes.extend_from_slice(&(0x08000000 + frame.list_offset).to_le_bytes());
        }
        Ok(vec![(
            self.source_offset,
            bytes,
            "constructor-obj-animation",
        )])
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Constructor OBJ animation differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: Animation,
    pub lists: Lists,
    /// Table and four independently bounded lists; alignment gaps stay outside.
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Constructor OBJ animation requires exact EUR ROM; USA pending".into());
        }
        // Caller stride, unsigned counter comparison, terminal phase and r1 flow.
        // The already authenticated copy/constructor/draw ABI remains unchanged.
        for (start, end, expected) in [
            (
                0x9db0c,
                0x9dba4,
                "688f2354ccec493358e570c0ae982aec1ad29444f67cad15012a0dcbb71649dc",
            ),
            (
                0x9dba4,
                0x9dc48,
                "4f01e7cda25637c7d817ee85f7e8ebfb93a8411b34d0ddb1e9d7604ac03243c1",
            ),
            (
                0xd9752c,
                0xd97540,
                "28fbf7ea3c419aff077a4047df36d1dd3a9027fa06f9f7bf33250a6847201cdb",
            ),
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
            if ffta_rom::digest(
                rom.get(start..end)
                    .ok_or("Truncated OBJ animation reader")?,
            ) != expected
            {
                return Err("Constructor OBJ animation reader identity mismatch".into());
            }
        }
        let bytes = rom
            .get(TABLE_OFFSET as usize..TABLE_OFFSET as usize + TABLE_LENGTH)
            .ok_or("Truncated constructor OBJ animation")?;
        if ffta_rom::digest(bytes) != TABLE_SHA256 {
            return Err("Constructor OBJ animation table identity mismatch".into());
        }
        let decoded = Animation {
            source_offset: TABLE_OFFSET,
            frames: bytes
                .chunks_exact(8)
                .map(|row| Frame {
                    threshold: u32::from_le_bytes(row[..4].try_into().unwrap()),
                    list_offset: u32::from_le_bytes(row[4..].try_into().unwrap())
                        .wrapping_sub(0x08000000),
                })
                .collect(),
        };
        decoded.validate()?;
        let mut lists = Vec::new();
        let mut sources = vec![SourceRange {
            offset: TABLE_OFFSET,
            length: TABLE_LENGTH as u32,
            sha256: TABLE_SHA256.into(),
            role: "constructor-obj-animation".into(),
        }];
        for (offset, expected) in LIST_ROOTS {
            // Both original readers establish u16 count plus six-byte objects.
            let bytes = rom
                .get(offset as usize..offset as usize + 14)
                .ok_or("Truncated constructor OBJ list")?;
            if ffta_rom::digest(bytes) != expected {
                return Err("Constructor OBJ list source identity mismatch".into());
            }
            let list = List::decode(offset, bytes)?;
            if list.objects.len() != 2 || list.objects.iter().any(|object| object.shape >= 3) {
                return Err("Constructor OBJ list count/available shape mismatch".into());
            }
            lists.push(list);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: expected.into(),
                role: "static-obj-list".into(),
            });
        }
        let lists = Lists { lists };
        decoded.validate_lists(&lists)?;
        sources.sort_by_key(|source| source.offset);
        Ok(Self {
            decoded,
            lists,
            sources,
        })
    }
}
