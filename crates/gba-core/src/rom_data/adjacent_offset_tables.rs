//! EUR adjacent offset tables; original index arithmetic remains guest-owned.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3ad460;
pub const SOURCE_LENGTH: usize = 32;
pub const TABLE_COUNT: usize = 5;
pub const SOURCE_SHA256: &str = "296d15a9ce5b40a4a58c76c71b76eedc966fd1cf55fd8db3ae5b10b3a4d75dbb";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdjacentOffsetTables {
    /// Four physical four-byte rows; the guest interprets their values.
    pub rows: [[u8; 4]; 4],
    /// Four independently rooted four-byte tables, including original padding.
    pub vectors: [[u8; 4]; 4],
}
impl AdjacentOffsetTables {
    pub fn validate(&self) -> Result<(), String> { Ok(()) }
    /// Returns physical source membership, not a bound on guest indices.
    pub fn byte_at(&self, at: u32) -> Option<(u8, usize)> {
        let relative = at.checked_sub(SOURCE_OFFSET)? as usize;
        if relative < 16 { Some((self.rows[relative / 4][relative % 4], 0)) }
        else { let n = relative - 16; Some((*self.vectors.get(n / 4)?.get(n % 4)?, 1 + n / 4)) }
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.rows.iter().chain(self.vectors.iter()).flatten().copied().collect())
    }
}

pub struct Catalog {
    pub decoded: AdjacentOffsetTables,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Adjacent offset tables require the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (0x130360, 0x1303e8, "16dbb0880b15c6bd8e2e663f802cf5a6eddbf970f0127c8911d5e2b5ea6e5b20"),
            (0x1304d0, 0x130524, "db05e87e043f5d3acf0c438a8d223322d6b592c89cc1554ec18b1e7396af352e"),
            (0x130f20, 0x130f68, "10f0c2bfa45a543c6beb6a0603e6e637f278be5811c30599dddf6de50d072dcc"),
            (0x1311a8, 0x1311b8, "a66e18bf7f096c1445364af23edea6e2d9397de35bba85720a239517ffe78043"),
            (0x3ad480, 0x3ad488, "ea99f710d9d0b8ba192295c969a63ed7ce8fc5743da20d2057fa2b6d2c404bfb"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Adjacent offset reader or successor mismatch".into());
            }
        }
        for (at, expected) in [(0x1303dc, 0x083ad470), (0x1303e0, 0x083ad460),
            (0x1303e4, 0x083ad474), (0x130520, 0x083ad478), (0x130f64, 0x083ad47c),
            (0x1311b4, 0x083ad480)] {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != expected {
                return Err("Adjacent offset reader literal mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize; let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Adjacent offset source mismatch".into()); }
        let decoded = AdjacentOffsetTables {
            rows: std::array::from_fn(|i| bytes[i * 4..i * 4 + 4].try_into().unwrap()),
            vectors: std::array::from_fn(|i| bytes[16 + i * 4..20 + i * 4].try_into().unwrap()),
        };
        let reconstructed = decoded.source_bytes()?;
        if reconstructed != bytes { return Err("Adjacent offset reconstruction mismatch".into()); }
        if PriorSources::new(previous)?.contains(SOURCE_OFFSET, SOURCE_LENGTH)? {
            return Err("Adjacent offset source already has an owner".into());
        }
        let sources = vec![SourceRange { offset: SOURCE_OFFSET, length: SOURCE_LENGTH as u32,
            sha256: ffta_rom::digest(&reconstructed), role: "adjacent-offset-tables".into() }];
        Ok(Self { decoded, sources })
    }
}
