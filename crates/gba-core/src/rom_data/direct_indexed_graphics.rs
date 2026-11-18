//! EUR directly rooted indexed archive; original decoding and upload stay guest work.
use super::{indexed_graphics, script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const CALLER_LITERAL: u32 = 0x023d08;
pub const ARCHIVE_OFFSET: u32 = 0x3ae484;
pub const ARCHIVE_LENGTH: usize = 288;
pub const SOURCE_SHA256: &str = "b2f76845b657826f41381a0a9f5fc8269364d219bef7b37defd65b070daf28a3";
pub const DECODED_SHA256: &str = "32f36ef4c8f2a5eef8780010587399e0199a75fe72c771f2bc071e783a7d6e4e";
pub type SourceSpan = (u32, Vec<u8>, &'static str);

/// A direct root is not a language group and does not constrain live selectors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectIndexedGraphics {
    pub caller_literal_offset: u32,
    pub archive: indexed_graphics::Archive,
}

impl DirectIndexedGraphics {
    pub fn validate(&self) -> Result<(), String> {
        let a = &self.archive;
        if self.caller_literal_offset != CALLER_LITERAL
            || a.source_offset != ARCHIVE_OFFSET
            || a.entry_offsets != [0x18, 0x59, 0x9a, 0xdb]
            || a.dictionary_relative_offset != 0x11c
            || a.dictionary_mode != 0
            || a.dictionary_length_bits != 5
            || a.fixed_decoded_bytes != 64
            || !a.dictionary_bytes.is_empty()
            || a.programs.len() != 4
            || a.programs.iter().any(|p| {
                !matches!(p.packets.as_slice(), [indexed_graphics::Packet::Literal { bytes }] if bytes.len() == 64)
            })
        {
            return Err("Invalid direct indexed graphics archive shape".into());
        }
        a.validate()?;
        if a.source_bytes()?.len() != ARCHIVE_LENGTH {
            return Err("Invalid direct indexed graphics source extent".into());
        }
        Ok(())
    }

    pub fn decoded_bytes(&self) -> Vec<u8> {
        self.archive
            .programs
            .iter()
            .flat_map(|p| p.decoded_bytes())
            .collect()
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(vec![
            (
                CALLER_LITERAL,
                (0x0800_0000u32 + ARCHIVE_OFFSET).to_le_bytes().to_vec(),
                "archive-literal",
            ),
            (ARCHIVE_OFFSET, self.archive.source_bytes()?, "archive"),
        ])
    }
}

pub struct Catalog {
    pub decoded: DirectIndexedGraphics,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Direct indexed graphics require the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0x23c98..0x23ccc])
            != "1a705951587a6919d75931a5fcf04877aa6de4c13f024b14d6602d61e334798a"
            || u32::from_le_bytes(
                rom[CALLER_LITERAL as usize..CALLER_LITERAL as usize + 4]
                    .try_into()
                    .unwrap(),
            ) != 0x083a_e484
            || u32::from_le_bytes(rom[0x23d10..0x23d14].try_into().unwrap()) != 0x083a_e5a4
        {
            return Err("Direct indexed graphics caller mismatch".into());
        }
        let decoded = DirectIndexedGraphics {
            caller_literal_offset: CALLER_LITERAL,
            archive: indexed_graphics::Archive::from_source(
                &rom[ARCHIVE_OFFSET as usize..],
                ARCHIVE_OFFSET,
            )?,
        };
        let prior = PriorSources::new(previous)?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? {
                return Err("Direct indexed graphics source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Direct indexed graphics source reconstruction mismatch".into());
            }
            combined.extend_from_slice(&bytes);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: role.into(),
            });
        }
        if combined.len() != 292
            || ffta_rom::digest(&combined) != SOURCE_SHA256
            || ffta_rom::digest(&decoded.decoded_bytes()) != DECODED_SHA256
        {
            return Err("Direct indexed graphics source or decoded hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
