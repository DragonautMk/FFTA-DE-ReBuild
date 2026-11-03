//! Fixed default constructor template; live patches and four alternate IDs stay distinct.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_OFFSET: u32 = 0x6e6c9c;
pub const END_OFFSET: u32 = SOURCE_OFFSET + 48;
pub const RECORD_BYTES: usize = 48;
pub const ALTERNATE_BASE: u32 = 0x6eca78;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub source_offset: u32,
    pub byte00: u8,
    pub byte01: u8,
    pub bytes02_04: [u8; 3],
    pub arguments05_07: [u8; 3],
    pub halfwords08_28: [u16; 17],
    pub flags2a: u16,
    pub halfwords2c_2e: [u16; 2],
}
impl Template {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET {
            return Err("Invalid fixed EUR default48 source".into());
        }
        Ok(())
    }
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0 => self.byte00,
            1 => self.byte01,
            2..=4 => self.bytes02_04[offset - 2],
            5..=7 => self.arguments05_07[offset - 5],
            8..=41 => (self.halfwords08_28[(offset - 8) / 2] >> (offset % 2 * 8)) as u8,
            42..=43 => (self.flags2a >> (offset % 2 * 8)) as u8,
            44..=47 => (self.halfwords2c_2e[(offset - 44) / 2] >> (offset % 2 * 8)) as u8,
            _ => return None,
        })
    }
    pub fn source_bytes(&self) -> [u8; RECORD_BYTES] {
        std::array::from_fn(|at| self.byte(at).unwrap())
    }
    /// Original selection truncates the caller ID before checking 20..23.
    pub fn default_id(id: u32) -> bool {
        !(20..=23).contains(&(id as u8))
    }
    /// Diagnostic materialization; runtime still executes the original copy/patch code.
    pub fn patched(&self, id: u32, selector: u32, arguments: [u32; 3], flag: u32) -> [u8; 48] {
        let mut out = self.source_bytes();
        if id as u8 as i8 >= 0 {
            out[0] = id as u8;
        }
        if selector as u8 > 1 {
            out[1] = selector as u8;
        }
        for (byte, argument) in out[5..8].iter_mut().zip(arguments) {
            *byte = argument as u8;
        }
        let mask = match flag as u8 {
            1 => 0x8000,
            2 => 0x2000,
            3 => 0x1000,
            _ => 0,
        };
        out[42..44].copy_from_slice(&(self.flags2a | mask).to_le_bytes());
        out
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Default48 differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
}
fn word(rom: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}
pub struct Catalog {
    pub decoded: Template,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Default48 requires exact EUR ROM; USA pending".into());
        }
        let v = half(rom, 0x127e46);
        if v != 0x4925
            || ((0x127e46 + 4) & !3) + usize::from(v & 255) * 4 != 0x127edc
            || word(rom, 0x127edc) != 0x08000000 + SOURCE_OFFSET
            || word(rom, 0x370eb4) != 0x03005ee9
            || word(rom, 0x127e38) != 0x0870cd54
            || word(rom, 0x70cd58) != 0x08000000 + ALTERNATE_BASE
        {
            return Err("Default48 literal/alternate/copy-helper mismatch".into());
        }
        for (at, expected) in [
            (0x127dc4, 0xb5f0),
            (0x127dce, 0xb08d),
            (0x127dd6, 0x0600),
            (0x127dd8, 0x0e07),
            (0x127dfc, 0x2e00),
            (0x127e08, 0x2e02),
            (0x127e16, 0x2803),
            (0x127e18, 0xd814),
            (0x127e1e, 0x0078),
            (0x127e20, 0x19c0),
            (0x127e22, 0x0100),
            (0x127e28, 0x6849),
            (0x127e4c, 0x2230),
            (0x127e54, 0x1603),
            (0x127e5a, 0xdb01),
            (0x127e5e, 0x7007),
            (0x127e62, 0x2801),
            (0x127e6a, 0x7041),
            (0x127e70, 0x7142),
            (0x127e74, 0x7181),
            (0x127e7a, 0x71c2),
            (0x127e7e, 0x8d41),
            (0x127e8a, 0x8551),
            (0x127e9c, 0x8551),
            (0x127eae, 0x8551),
            (0x145c14, 0x4718),
        ] {
            if half(rom, at) != expected {
                return Err("Default48 original constructor-prefix mismatch".into());
            }
        }
        let raw = &rom[SOURCE_OFFSET as usize..END_OFFSET as usize];
        let decoded = Template {
            source_offset: SOURCE_OFFSET,
            byte00: raw[0],
            byte01: raw[1],
            bytes02_04: raw[2..5].try_into().unwrap(),
            arguments05_07: raw[5..8].try_into().unwrap(),
            halfwords08_28: std::array::from_fn(|i| half(raw, 8 + i * 2)),
            flags2a: half(raw, 42),
            halfwords2c_2e: std::array::from_fn(|i| half(raw, 44 + i * 2)),
        };
        decoded.validate()?;
        Ok(Self {
            decoded,
            sources: vec![SourceRange {
                offset: SOURCE_OFFSET,
                length: RECORD_BYTES as u32,
                sha256: ffta_rom::digest(raw),
                role: "default48/eur-fixed".into(),
            }],
        })
    }
}
