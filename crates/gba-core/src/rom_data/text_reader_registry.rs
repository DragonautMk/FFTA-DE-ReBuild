//! EUR text-reader callback registry; original indirect dispatch remains guest work.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const PRODUCER_LITERAL: u32 = 0x0191a0;
pub const LOOKUP_LITERAL: u32 = 0x019448;
pub const TABLE_OFFSET: u32 = 0x371068;
pub const ENTRY_COUNT: usize = 27;
pub const SOURCE_SHA256: &str = "996effad5ea92ed71718a590a7cfa041e7f96c829e260525e016ea1f1fabdfa9";
pub type SourceSpan = (u32, Vec<u8>, &'static str);

/// Only the independently witnessed projection is owned; guest indices remain uncapped.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextReaderRegistry {
    pub table_offset: u32,
    pub callbacks: [u32; ENTRY_COUNT],
}

impl TextReaderRegistry {
    pub fn validate(&self) -> Result<(), String> {
        if self.table_offset != TABLE_OFFSET
            || self
                .callbacks
                .iter()
                .any(|&p| p != 0 && (p & 1 == 0 || !(0x0800_0000..0x0900_0000).contains(&p)))
        {
            return Err("Invalid text-reader registry".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let literal = (0x0800_0000u32 + self.table_offset).to_le_bytes().to_vec();
        let words = self
            .callbacks
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        Ok(vec![
            (PRODUCER_LITERAL, literal.clone(), "producer-literal"),
            (LOOKUP_LITERAL, literal, "lookup-literal"),
            (TABLE_OFFSET, words, "reader-callback-words"),
        ])
    }
}

pub struct Catalog {
    pub decoded: TextReaderRegistry,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Text-reader registry requires the exact EUR ROM".into());
        }
        for (start, end, hash) in [
            (
                0x1916c,
                0x191a4,
                "c11227523ebe6fd1c2a2c6f48f0b8dab605c6c8ad1ec1194a725d6407d7c64c7",
            ),
            (
                0x1941c,
                0x1944c,
                "6f3d0d4608dcdafa05513e2c9d33bc7967ace33861160184ef0a464f12631bbb",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Text-reader registry instruction mismatch".into());
            }
        }
        for offset in [PRODUCER_LITERAL, LOOKUP_LITERAL] {
            if u32::from_le_bytes(
                rom[offset as usize..offset as usize + 4]
                    .try_into()
                    .unwrap(),
            ) != 0x0837_1068
            {
                return Err("Text-reader registry literal mismatch".into());
            }
        }
        let callbacks = std::array::from_fn(|i| {
            let at = TABLE_OFFSET as usize + 4 * i;
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        });
        let decoded = TextReaderRegistry {
            table_offset: TABLE_OFFSET,
            callbacks,
        };
        let prior = PriorSources::new(previous)?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? {
                return Err("Text-reader registry source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Text-reader registry source reconstruction mismatch".into());
            }
            combined.extend_from_slice(&bytes);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: role.into(),
            });
        }
        if ffta_rom::digest(&combined) != SOURCE_SHA256 {
            return Err("Text-reader registry source hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
