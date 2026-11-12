//! Conditional OBJ lists selected by the EUR context initializer.
//!
//! The low byte of the original selector chooses a list for values 0..=4;
//! higher values leave the caller's pointer unchanged. The separate live
//! index used by nearby context parameters is not bounded by this contract.
use super::{List, Lists};
use crate::{ffta_rom, rom_data::SourceRange};

pub const ROOTS: [(u32, usize, &str); 4] = [
    (0x3ae440, 4, "17fe7f7981eb0677d2b83f2a5b4008a7ca0904ee8071016f8f6422cbfd851a94"),
    (0x3ae45a, 1, "7c9fa136d4413fa6173637e883b6998d32e1d675f88cddff9dcbcf331820f4b8"),
    (0x3ae462, 1, "f9d10b77126ab15bc3488ce712403813f59b1dc4781f4cf22bef2feacd0a2868"),
    (0x3ae46a, 1, "58c397879d42e8f2c7d17d42792d5ce64ef21866763760d1b5b1870981770a5d"),
];

/// Offset selected by the authenticated original writer; None means it does
/// not write the pointer, rather than an invented fallback or guest cap.
pub fn select(selector: u32) -> Option<u32> {
    match selector as u8 {
        0 => Some(ROOTS[0].0),
        1 => Some(ROOTS[1].0),
        2 | 4 => Some(ROOTS[2].0),
        3 => Some(ROOTS[3].0),
        _ => None,
    }
}

pub struct Catalog {
    pub decoded: Lists,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Context OBJ selector requires the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (0x14380c, 0x143860, "0be370a2030cf83b43e4cea7a6f2a037616547aa7314e0827e99d646cd04b517"),
            (0x143580, 0x14360e, "b79fb547030f7713c5724f0137e6b77b58e69eee80fed6d15462d14b7a907465"),
            (0x1500, 0x170c, "991e33b72f908114b349f73626f506c67f3e055981c5e85131333c1ff5f1914e"),
        ] {
            if ffta_rom::digest(rom.get(start..end).ok_or("Truncated context OBJ reader")?) != expected {
                return Err("Context OBJ selector, constructor or draw identity mismatch".into());
            }
        }
        let mut lists = Vec::new();
        let mut sources = Vec::new();
        for (offset, count, expected) in ROOTS {
            let length = 2 + count * 6;
            let bytes = rom.get(offset as usize..offset as usize + length).ok_or("Truncated context OBJ list")?;
            if ffta_rom::digest(bytes) != expected { return Err("Context OBJ source identity mismatch".into()); }
            let list = List::decode(offset, bytes)?;
            if list.objects.len() != count || list.objects.iter().any(|object| object.shape >= 3) {
                return Err("Context OBJ list count or shape mismatch".into());
            }
            sources.push(SourceRange { offset, length: length as u32, sha256: expected.into(), role: "static-obj-list".into() });
            lists.push(list);
        }
        let decoded = Lists { lists, battle_directory: None, actor_lookup: None, title_opening_directory: None };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}
