//! Authenticated EUR Soil Evidence bytes read by a guest-supplied index.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const OFFSET: u32 = 0x39e224;
pub const COUNT: usize = 6;
const SOURCE_SHA256: &str = "b0f59dcf85159d0959435892a72eb41b37c89a81d7a248235ac565d3b770dffc";
const READER_SHA256: &str = "3e5e39ed857de968a49ae55643117b8a2a110b6a90511d8ddfd0f27e92ab211a";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilIndexedBytes {
    pub offset: u32,
    pub values: [u8; COUNT],
}
impl SoilIndexedBytes {
    pub fn validate(&self) -> Result<(), String> {
        if self.offset != OFFSET {
            return Err("Invalid Soil indexed byte source offset".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![(
            self.offset,
            self.values.to_vec(),
            "soil-indexed-bytes",
        )])
    }
    /// This covers witnessed physical bytes, not the guest index domain.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        let index = offset.checked_sub(self.offset)? as usize;
        self.values.get(index).copied().map(|byte| (byte, index))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Soil indexed bytes differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: SoilIndexedBytes,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Soil indexed bytes require the exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[0xf05c6..0xf05d2]) != READER_SHA256 {
            return Err("Soil indexed byte reader fingerprint mismatch".into());
        }
        let pc = 0xf05c6u32;
        let literal = 0xf05fcu32;
        let op = u16::from_le_bytes(rom[pc as usize..pc as usize + 2].try_into().unwrap());
        let target = u32::from_le_bytes(
            rom[literal as usize..literal as usize + 4]
                .try_into()
                .unwrap(),
        );
        if op & 0xf800 != 0x4800
            || ((pc + 4) & !3) + u32::from(op & 255) * 4 != literal
            || target != 0x0800_0000 + OFFSET
        {
            return Err("Soil indexed byte reader literal mismatch".into());
        }
        let literals = previous
            .iter()
            .find(|resource| resource.id == THUMB_PC_LITERALS_ID);
        if !previous.is_empty() && literals.is_none() {
            return Err("Missing Soil indexed byte literal dependency".into());
        }
        if literals.is_some_and(|owner| {
            !owner
                .sources
                .iter()
                .any(|source| source.offset == literal && source.length == 4)
        }) {
            return Err("Missing separately owned Soil indexed byte literal".into());
        }
        let at = OFFSET as usize;
        let decoded = SoilIndexedBytes {
            offset: OFFSET,
            values: rom[at..at + COUNT].try_into().unwrap(),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Soil indexed byte source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != SOURCE_SHA256 {
                return Err("Soil indexed byte source hash mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
