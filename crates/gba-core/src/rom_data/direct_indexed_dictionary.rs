//! EUR directly rooted indexed archive with dictionary packets.
use super::{indexed_graphics, script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const CALLER_LITERAL: u32 = 0x023d10;
pub const ARCHIVE_OFFSET: u32 = 0x3ae5a4;
pub const ARCHIVE_LENGTH: usize = 2008;
pub const SOURCE_SHA256: &str = "4a68eb98f073f5f5179edf941a2d5a20773c2b6fef2ae52ad8562093ab6bfaba";
pub const DECODED_SHA256: &str = "668112f0c7946c0ad46564c6c79649a615560c1054116807fb6113d8395ef1cd";
pub type SourceSpan = (u32, Vec<u8>, &'static str);

/// A separate direct root; it is not a language group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectIndexedDictionary {
    pub caller_literal_offset: u32,
    pub archive: indexed_graphics::Archive,
}

impl DirectIndexedDictionary {
    pub fn validate(&self) -> Result<(), String> {
        let a = &self.archive;
        if self.caller_literal_offset != CALLER_LITERAL
            || a.source_offset != ARCHIVE_OFFSET
            || a.programs.len() != 41
            || a.dictionary_relative_offset != 0x774
            || a.dictionary_mode != 0
            || a.dictionary_length_bits != 5
            || a.fixed_decoded_bytes != 64
            || a.dictionary_bytes.len() != 96
        {
            return Err("Invalid direct indexed dictionary archive shape".into());
        }
        a.validate()?;
        if a.source_bytes()?.len() != ARCHIVE_LENGTH {
            return Err("Invalid direct indexed dictionary archive extent".into());
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
    pub decoded: DirectIndexedDictionary,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Direct indexed dictionary requires the exact EUR ROM".into());
        }
        if ffta_rom::digest(&rom[0x23c98..0x23ccc])
            != "1a705951587a6919d75931a5fcf04877aa6de4c13f024b14d6602d61e334798a"
            || u32::from_le_bytes(
                rom[CALLER_LITERAL as usize..CALLER_LITERAL as usize + 4]
                    .try_into()
                    .unwrap(),
            ) != 0x083a_e5a4
        {
            return Err("Direct indexed dictionary caller mismatch".into());
        }
        let decoded = DirectIndexedDictionary {
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
                return Err("Direct indexed dictionary source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Direct indexed dictionary source reconstruction mismatch".into());
            }
            combined.extend_from_slice(&bytes);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: role.into(),
            });
        }
        if combined.len() != 2012
            || ffta_rom::digest(&combined) != SOURCE_SHA256
            || ffta_rom::digest(&decoded.decoded_bytes()) != DECODED_SHA256
        {
            return Err("Direct indexed dictionary source or decoded hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
