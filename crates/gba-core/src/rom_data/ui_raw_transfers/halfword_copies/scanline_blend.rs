//! Blend coefficients transferred by the two original 160-line copy phases.

use super::{literal_span, rom_word, source_digest, Bank, SourceSpan};
use serde::{Deserialize, Serialize};

pub(super) const BANK: (u32, usize) = (0x3a62ec, 960);
const ROOT_ADDRESS: u32 = 0x0800_0000 + BANK.0;
pub(super) const LITERALS: [(u32, u32); 2] = [(0x10d050, ROOT_ADDRESS), (0x10d130, ROOT_ADDRESS)];
const SOURCE_SHA256: &str = "650becdfa472db1e500dc2301f1d3eee985a9189e385cdb35e4282b9a6394d6e";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanlineBlendCopies {
    pub caller_literals: [u32; 2],
    pub bank: Bank,
}

impl ScanlineBlendCopies {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| rom_word(rom, offset)),
            bank: Bank::from_rom(rom, BANK),
        };
        if source_digest(value.source_spans()?) != SOURCE_SHA256 {
            return Err("Scanline blend source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        Ok(vec![
            literal_span(
                self.caller_literals[0],
                LITERALS[0],
                "scanline-blend-literal",
            )?,
            literal_span(
                self.caller_literals[1],
                LITERALS[1],
                "scanline-blend-literal",
            )?,
            self.bank.source_span(BANK, "scanline-blend-coefficients")?,
        ])
    }
}
