//! Authenticated EUR Soil Evidence compact record and observed pointer.
use super::{
    compact_obj_catalog, script39_registry_poses::PriorSources, Resource, SourceRange,
    COMPACT_OBJ_CATALOG_ID, THUMB_PC_LITERALS_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const RECORD_OFFSET: u32 = 0x39a91c;
pub const RECORD_BYTES: usize = 24;
pub const POINTER_OFFSET: u32 = 0x39aa34;
pub const POINTER_BYTES: usize = 4;
const RECORD_SHA256: &str = "0fe3a365ab86a411912e1989e8814ea5a3d37a18d6b1aec439a92a54c0198c0b";
const POINTER_SHA256: &str = "8c41bf79ad2efc5ebe2e225a5b58db623ce2fd37ffd88778052b1431ef5fcb13";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilCompactRecord {
    pub record_offset: u32,
    pub bank_pointer: u32,
    pub halfwords: [u16; 10],
    pub pointer_offset: u32,
    pub record_pointer: u32,
}
impl SoilCompactRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.record_offset != RECORD_OFFSET
            || self.pointer_offset != POINTER_OFFSET
            || self.bank_pointer != 0x0800_0000 + compact_obj_catalog::SOIL_EVIDENCE_ROOT
            || self.record_pointer != 0x0800_0000 + RECORD_OFFSET
            || self.halfwords[0] != 0
        {
            return Err("Invalid Soil compact record or source links".into());
        }
        Ok(())
    }
    pub fn record_bytes(&self) -> [u8; RECORD_BYTES] {
        let mut bytes = [0; RECORD_BYTES];
        bytes[..4].copy_from_slice(&self.bank_pointer.to_le_bytes());
        for (index, value) in self.halfwords.iter().enumerate() {
            bytes[4 + index * 2..6 + index * 2].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![
            (
                self.record_offset,
                self.record_bytes().to_vec(),
                "soil-compact-record",
            ),
            (
                self.pointer_offset,
                self.record_pointer.to_le_bytes().to_vec(),
                "soil-compact-record-pointer",
            ),
        ])
    }
    /// These two witnessed physical spans do not bound guest pointer arithmetic.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        if let Some(at) = offset.checked_sub(self.record_offset) {
            if let Some(byte) = self.record_bytes().get(at as usize) {
                return Some((*byte, 0));
            }
        }
        let at = offset.checked_sub(self.pointer_offset)? as usize;
        self.record_pointer
            .to_le_bytes()
            .get(at)
            .copied()
            .map(|byte| (byte, 1))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Soil compact record differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: SoilCompactRecord,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Soil compact record requires the exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0xe4af4,
                0xe4b00,
                "170ccfae7d30100baa2843e90ea2f86eb4937d7789dff42eb1aaadb2887037f0",
            ),
            (
                0xd8b60,
                0xd8bf0,
                "0f245d9f3002a8d007b67ea9fd43eb0577146c522cc054fb98c1b37a24f723dc",
            ),
            (
                0xd8974,
                0xd8976,
                "13f0723b49e1230d6bef62230fa215683660e447244629f25501d6eae55530ca",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Soil compact record reader fingerprint mismatch".into());
            }
        }
        let pc = 0xe4af4u32;
        let literal = 0xe4b80u32;
        let op = u16::from_le_bytes(rom[pc as usize..pc as usize + 2].try_into().unwrap());
        let root = u32::from_le_bytes(
            rom[literal as usize..literal as usize + 4]
                .try_into()
                .unwrap(),
        );
        if op & 0xf800 != 0x4800
            || ((pc + 4) & !3) + u32::from(op & 255) * 4 != literal
            || root != 0x0839_aa24
            || 0x39aa24 + 4 * 4 != POINTER_OFFSET
        {
            return Err("Soil compact record pointer root mismatch".into());
        }
        let literals = previous.iter().find(|r| r.id == THUMB_PC_LITERALS_ID);
        let bank = previous.iter().find(|r| r.id == COMPACT_OBJ_CATALOG_ID);
        if !previous.is_empty() && (literals.is_none() || bank.is_none()) {
            return Err("Missing Soil compact record dependencies".into());
        }
        if literals.is_some_and(|owner| {
            !owner
                .sources
                .iter()
                .any(|s| s.offset == literal && s.length == 4)
        }) || bank.is_some_and(|owner| {
            !owner
                .sources
                .iter()
                .any(|s| s.offset == compact_obj_catalog::SOIL_EVIDENCE_ROOT && s.length >= 4)
        }) {
            return Err("Missing separately owned Soil compact record roots".into());
        }
        let at = RECORD_OFFSET as usize;
        let decoded = SoilCompactRecord {
            record_offset: RECORD_OFFSET,
            bank_pointer: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()),
            halfwords: std::array::from_fn(|i| {
                u16::from_le_bytes(rom[at + 4 + i * 2..at + 6 + i * 2].try_into().unwrap())
            }),
            pointer_offset: POINTER_OFFSET,
            record_pointer: u32::from_le_bytes(
                rom[POINTER_OFFSET as usize..POINTER_OFFSET as usize + 4]
                    .try_into()
                    .unwrap(),
            ),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Soil compact source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            let expected = if offset == RECORD_OFFSET {
                RECORD_SHA256
            } else {
                POINTER_SHA256
            };
            if sha256 != expected {
                return Err("Soil compact record source hash mismatch".into());
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
