//! Two authenticated EUR DMA register directories; selection and DMA stay guest-owned.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const WRITE_ROOT: u32 = 0x370e68;
pub const POLL_ROOT: u32 = 0x370e78;
pub const DIRECTORY_LENGTH: usize = 16;
pub const DIRECTORY_SHA256: &str = "fd01a90e55a89406f20add00c25ffc8864899434b75c38effe084aaf78fbe076";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DmaRegisterDirectories {
    pub write_registers: [u32; 4],
    pub poll_registers: [u32; 4],
}

impl DmaRegisterDirectories {
    pub fn validate(&self) -> Result<(), String> {
        for (index, (&write, &poll)) in self.write_registers.iter().zip(&self.poll_registers).enumerate() {
            let expected = 0x0400_00b0 + index as u32 * 12;
            if write != expected || poll != expected {
                return Err("Invalid DMA register directory projection".into());
            }
        }
        Ok(())
    }

    /// Physical ownership only; a miss leaves the guest's original arithmetic intact.
    pub fn byte_at(&self, at: u32) -> Option<(u8, usize)> {
        for (index, (root, values)) in [(WRITE_ROOT, &self.write_registers), (POLL_ROOT, &self.poll_registers)].into_iter().enumerate() {
            if let Some(relative) = at.checked_sub(root) {
                if let Some(word) = values.get(relative as usize / 4) {
                    return Some((word.to_le_bytes()[relative as usize % 4], index));
                }
            }
        }
        None
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![
            (WRITE_ROOT, self.write_registers.iter().flat_map(|word| word.to_le_bytes()).collect(), "dma-register-write-directory"),
            (POLL_ROOT, self.poll_registers.iter().flat_map(|word| word.to_le_bytes()).collect(), "dma-register-poll-directory"),
        ])
    }
}

pub struct Catalog {
    pub decoded: DmaRegisterDirectories,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("DMA register directories require the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (0x800, 0x918, "ac9a693288b02a8ae2d7024fdd53894d80016d550721178091e91bdfc5de6864"),
            (0x918, 0x9e4, "98a2b5ea927e67218e81e82b0abb30deeb2b8e9dfda97f8ec1d66e1257378236"),
            (0xd40, 0xd80, "1e762a07dfdf125e6ab4d871ac1b54bc916dd4ffa8449d179fb80f91b524b4b1"),
            (0x3a6c, 0x3aa4, "cec9270ad9cb60426c7d589c1b7bd7a3aa81d5cace35a55287ea0250f6f0c8b8"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("DMA register original reader mismatch".into());
            }
        }
        let read = |root: u32| -> Result<[u32; 4], String> {
            let raw = &rom[root as usize..root as usize + DIRECTORY_LENGTH];
            if ffta_rom::digest(raw) != DIRECTORY_SHA256 { return Err("DMA register directory source mismatch".into()); }
            Ok(std::array::from_fn(|index| {
                u32::from_le_bytes(raw[index * 4..index * 4 + 4].try_into().unwrap())
            }))
        };
        let decoded = DmaRegisterDirectories { write_registers: read(WRITE_ROOT)?, poll_registers: read(POLL_ROOT)? };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? { return Err("DMA register directory already has an owner".into()); }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes { return Err("DMA directory reconstruction mismatch".into()); }
            sources.push(SourceRange { offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role: role.into() });
        }
        Ok(Self { decoded, sources })
    }
}
