//! Bounded EUR values observed through two White Flowers readers.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange, THUMB_PC_LITERALS_ID};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const ROW_OFFSET: u32 = 0x6e8994;
pub const ROW_LENGTH: usize = 20;
pub const BYTE_OFFSET: u32 = 0x3c9460;
pub const ROOT_LITERALS: [u32; 2] = [0x02ed2c, 0x0cf40c];
pub const ROW_ROLE: &str = "white-flowers-first-halfword-row";
pub const BYTE_ROLE: &str = "white-flowers-observed-nibble-byte";
const ROW_HASH: &str = "de88d036cd30dea0622921a3e5be0cc2306613462b617f610a7e1dcedbc9f372";
const BYTE_HASH: &str = "2795044ce0f83f718bc79c5f2add1e52521978df91ce9b7f82c9097191d33602";
const SETUP_HASH: &str = "4252e9d5a5d5686285d4bfc082025703c1541004612c5d28a6f87c8f41e0d358";
const ROW_READER_HASH: &str = "2364cc87a2b02991eae30a912f25068c2996b5bc1e63abc8d0dc84df32947850";
const BYTE_READER_HASH: &str = "d0917f0bcd07873751f8b871cb3ddea4f66063baa64064dfd5467f082c8253ca";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhiteFlowersIsolatedValues {
    pub row_offset: u32,
    pub row: Vec<u16>,
    pub byte_offset: u32,
    pub byte: u8,
}
impl WhiteFlowersIsolatedValues {
    pub fn validate(&self) -> Result<(), String> {
        if self.row_offset != ROW_OFFSET || self.row.len() != ROW_LENGTH || self.byte_offset != BYTE_OFFSET {
            return Err("Invalid White Flowers isolated value sources or length".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![(ROW_OFFSET, self.row.iter().flat_map(|value| value.to_le_bytes()).collect(), ROW_ROLE),
            (BYTE_OFFSET, vec![self.byte], BYTE_ROLE)])
    }
    /// The second result distinguishes the row from the isolated byte.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        if offset == self.byte_offset { return Some((self.byte, ROW_LENGTH)); }
        let at = offset.checked_sub(self.row_offset)? as usize;
        self.row.get(at / 2).map(|value| (value.to_le_bytes()[at % 2], at / 2))
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("White Flowers isolated values differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: WhiteFlowersIsolatedValues,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("White Flowers isolated values require exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[0x02ecce..0x02ece6]) != SETUP_HASH
            || ffta_rom::digest(&rom[0x02edc0..0x02edce]) != ROW_READER_HASH
            || ffta_rom::digest(&rom[0x0cf3fc..0x0cf40c]) != BYTE_READER_HASH
            || u32::from_le_bytes(rom[0x02ed2c..0x02ed30].try_into().unwrap()) != 0x086e8994
            || u32::from_le_bytes(rom[0x0cf40c..0x0cf410].try_into().unwrap()) != 0x083c943c
        {
            return Err("White Flowers isolated reader or root fingerprint mismatch".into());
        }
        if !previous.is_empty() && ROOT_LITERALS.iter().any(|&root| !previous.iter().any(|resource| {
            resource.id == THUMB_PC_LITERALS_ID && resource.sources.iter().any(|source| source.offset == root && source.length == 4)
        })) {
            return Err("Missing separately owned White Flowers isolated-value root".into());
        }
        let start = ROW_OFFSET as usize;
        let row_bytes = &rom[start..start + ROW_LENGTH * 2];
        let byte_bytes = &rom[BYTE_OFFSET as usize..BYTE_OFFSET as usize + 1];
        let prior = PriorSources::new(previous)?;
        if ffta_rom::digest(row_bytes) != ROW_HASH || ffta_rom::digest(byte_bytes) != BYTE_HASH
            || prior.contains(ROW_OFFSET, row_bytes.len())? || prior.contains(BYTE_OFFSET, 1)?
        {
            return Err("White Flowers isolated source fingerprint or owner mismatch".into());
        }
        let decoded = WhiteFlowersIsolatedValues { row_offset: ROW_OFFSET,
            row: row_bytes.chunks_exact(2).map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap())).collect(),
            byte_offset: BYTE_OFFSET, byte: byte_bytes[0] };
        let spans = decoded.source_spans()?;
        if spans[0].1 != row_bytes || spans[1].1 != byte_bytes {
            return Err("White Flowers isolated source reconstruction mismatch".into());
        }
        let sources = spans.into_iter().map(|(offset, bytes, role)| SourceRange {
            offset, length: bytes.len() as u32, sha256: ffta_rom::digest(&bytes), role: role.into()
        }).collect();
        Ok(Self { decoded, sources })
    }
}
