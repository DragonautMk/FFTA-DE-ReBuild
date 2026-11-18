//! EUR context callback descriptor; original dispatcher and callback remain guest work.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const CONSTRUCTOR_LITERAL: u32 = 0x144e38;
pub const DESCRIPTOR_OFFSET: u32 = 0x3ae47c;
pub const SOURCE_SHA256: &str = "dd06cb9a8b08fc51645d8fdea3daafc014d4698fe9d4e11631e79925975d85cb";
pub type SourceSpan = (u32, Vec<u8>, &'static str);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextCallbackDescriptor {
    pub constructor_literal_offset: u32,
    pub descriptor_offset: u32,
    pub callback_thumb_pointer: u32,
    pub signed_delay: i16,
    pub flags: u8,
    /// No original reader has established this byte's semantics.
    pub opaque_byte: u8,
}
impl ContextCallbackDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if self.constructor_literal_offset != CONSTRUCTOR_LITERAL
            || self.descriptor_offset != DESCRIPTOR_OFFSET
            || self.callback_thumb_pointer & 1 == 0
            || !(0x0800_0000..0x0900_0000).contains(&self.callback_thumb_pointer)
        {
            return Err("Invalid context callback descriptor".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut row = Vec::with_capacity(8);
        row.extend_from_slice(&self.callback_thumb_pointer.to_le_bytes());
        row.extend_from_slice(&self.signed_delay.to_le_bytes());
        row.extend([self.flags, self.opaque_byte]);
        Ok(vec![
            (
                CONSTRUCTOR_LITERAL,
                (0x0800_0000u32 + self.descriptor_offset)
                    .to_le_bytes()
                    .to_vec(),
                "constructor-literal",
            ),
            (DESCRIPTOR_OFFSET, row, "callback-descriptor"),
        ])
    }
}

pub struct Catalog {
    pub decoded: ContextCallbackDescriptor,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Context callback descriptor requires the exact EUR ROM".into());
        }
        for (start, end, hash) in [
            (
                0x144d24,
                0x144dfc,
                "145842e353dfb06100305992ea113beb52585eb6eee0fa444e0d7ef04f7e0cd2",
            ),
            (
                0x144dfc,
                0x144e3c,
                "c14d721c31d00b534a30dce9d28b9e47d41984639ba0ec467c1087fd9a4d0781",
            ),
            (
                0x1449fc,
                0x144a3c,
                "daf441635cf1a4efc4777663408116faa44473a3239a9d85c9d8375fe86b5a74",
            ),
            (
                0x144d10,
                0x144d22,
                "6f79c8a13d3d4feb121d6afd41e2cf523a8a630c7912f3ccb410ee86a8d1f455",
            ),
            (
                0x145c10,
                0x145c14,
                "b6e741ccff336746fdcf62bc30c37deaa54afc60e5be6f8d1c33cfd2b8184ebc",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Context callback instruction mismatch".into());
            }
        }
        if u32::from_le_bytes(
            rom[CONSTRUCTOR_LITERAL as usize..CONSTRUCTOR_LITERAL as usize + 4]
                .try_into()
                .unwrap(),
        ) != 0x083a_e47c
        {
            return Err("Context callback constructor literal mismatch".into());
        }
        let at = DESCRIPTOR_OFFSET as usize;
        let decoded = ContextCallbackDescriptor {
            constructor_literal_offset: CONSTRUCTOR_LITERAL,
            descriptor_offset: DESCRIPTOR_OFFSET,
            callback_thumb_pointer: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()),
            signed_delay: i16::from_le_bytes(rom[at + 4..at + 6].try_into().unwrap()),
            flags: rom[at + 6],
            opaque_byte: rom[at + 7],
        };
        let prior = PriorSources::new(previous)?;
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())? {
                return Err("Context callback source already has an owner".into());
            }
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Context callback source reconstruction mismatch".into());
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
            return Err("Context callback source hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
