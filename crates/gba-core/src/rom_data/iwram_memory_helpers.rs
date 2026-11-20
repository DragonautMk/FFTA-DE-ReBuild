//! Observed EUR IWRAM helper pointers. Live code and guest calls remain guest-owned.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x370eb0;
pub const HELPER_COUNT: usize = 3;
pub const SOURCE_LENGTH: usize = HELPER_COUNT * 4;
pub const SOURCE_SHA256: &str = "fec5c1cc1c2f75d5d856d4d487afebca78400372eff2b7359c07e7957e2b7ddb";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IwramMemoryHelpers {
    /// Encoded Thumb addresses; mutable IWRAM instructions are not exported.
    pub callbacks: [u32; HELPER_COUNT],
}
impl IwramMemoryHelpers {
    pub fn validate(&self) -> Result<(), String> {
        if self.callbacks.iter().any(|&word| word & 1 == 0 || !(0x0300_0000..0x0300_8000).contains(&(word & !1))) {
            return Err("Invalid IWRAM memory-helper pointer".into());
        }
        Ok(())
    }
    /// Physical source membership only; no guest selector bound is imposed.
    pub fn byte_at(&self, at: u32) -> Option<(u8, usize)> {
        let relative = at.checked_sub(SOURCE_OFFSET)? as usize;
        let index = relative / 4;
        Some((self.callbacks.get(index)?.to_le_bytes()[relative % 4], index))
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.callbacks.iter().flat_map(|word| word.to_le_bytes()).collect())
    }
}

pub struct Catalog {
    pub decoded: IwramMemoryHelpers,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("IWRAM memory helpers require the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (0x28fdc, 0x29000, "2ac866064dd7993e4342f9895f451aaf73e8429d431157660f42a126c93b39a5"),
            (0x29030, 0x29042, "a266411168c978ef8eb9d5e6fefdf20b100afcedf4b0e4121b3967ae350ded78"),
            (0x11cc, 0x1200, "065d6c7dedeb954aeef80e0e0fae592909c7d9eb04db36ca54b0c3d83b88917e"),
            (0x145c0c, 0x145c18, "c99a70b603223eece62ea525a01b8de0867bf9ffcb20b8126a702095f354ca16"),
            (0xd96838, 0xd96858, "c6b3141922fd7ce1eddd91f84ef1d7f6499f5fbee2517ad8b15021825e075937"),
            (0xd968a8, 0xd968c8, "31e1c5c89ad1503b60109458eee63ff9f4b25625ff5fd78c6e365461fb38e2f8"),
            (0xd9686c, 0xd9688c, "dfa2f7fe38fa092075b96d45511474e56f92ad9d1331e10b4dd39891bd32edc1"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("IWRAM helper reader or ROM prefix mismatch".into());
            }
        }
        for (at, expected) in [(0x1254, 0x0837_0eb0), (0x2905c, 0x0837_0eb4), (0x2906c, 0x0837_0eb8)] {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != expected {
                return Err("IWRAM helper reader literal mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize;
        let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("IWRAM helper source mismatch".into()); }
        let decoded = IwramMemoryHelpers { callbacks: std::array::from_fn(|i|
            u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())) };
        let reconstructed = decoded.source_bytes()?;
        if reconstructed != bytes { return Err("IWRAM helper reconstruction mismatch".into()); }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_LENGTH)? {
            return Err("IWRAM memory-helper source already has an owner".into());
        }
        let sources = vec![SourceRange { offset: SOURCE_OFFSET, length: SOURCE_LENGTH as u32,
            sha256: ffta_rom::digest(&reconstructed), role: "iwram-memory-helper-vector".into() }];
        Ok(Self { decoded, sources })
    }
}
