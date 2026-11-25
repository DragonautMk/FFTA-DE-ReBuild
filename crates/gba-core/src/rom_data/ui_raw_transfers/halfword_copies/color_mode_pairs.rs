//! Paired Color Mode palettes selected by the original CPUSet callers.

use super::{
    invalid, literal_byte, literal_span, rom_word, source_digest, Bank, ByteSource, SourceSpan,
    SELECTORS,
};
use serde::{Deserialize, Serialize};

const SOURCE_SHA256: &str = "e79764dc5b172312a9ee08b0955f33b81665cc18d64b45cecce7b1865b3ea9fc";
const ROM_BASE: u32 = 0x0800_0000;
pub(super) const TABLES: [u32; 3] = [0x395644, 0x397dd0, 0x3adf48];
const BANK_BASES: [u32; 3] = [0xb537a8, 0xb4bd80, 0xb4bd80];
const PAIR_ORDER: [u32; 8] = [0, 1, 4, 5, 2, 3, 0, 1];
pub(super) const TABLE_BYTES: usize = PAIR_ORDER.len() * 4;
pub(super) const LITERALS: [(u32, u32); 5] = [
    (0x0329b4, ROM_BASE + TABLES[0]),
    (0x059d8c, ROM_BASE + TABLES[0]),
    (0x070fe4, ROM_BASE + TABLES[1]),
    (0x08c4b0, ROM_BASE + TABLES[1]),
    (0x13bc9c, ROM_BASE + TABLES[2]),
];
// The b4bea0..b4bec0 fragment has an existing palette owner.
pub(super) const BANKS: [(u32, usize); 3] = [(0xb4bd80, 288), (0xb4bec0, 2752), (0xb537a8, 3072)];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorModePairs {
    pub caller_literals: [u32; 5],
    pub selectors: [[u32; 8]; 3],
    pub banks: [Bank; 3],
}

impl ColorModePairs {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| rom_word(rom, offset)),
            selectors: TABLES
                .map(|offset| std::array::from_fn(|i| rom_word(rom, offset + 4 * i as u32))),
            banks: BANKS.map(|shape| Bank::from_rom(rom, shape)),
        };
        if source_digest(value.source_spans()?) != SOURCE_SHA256 {
            return Err("Paired Color Mode bank source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let expected = BANK_BASES.map(|base| PAIR_ORDER.map(|i| ROM_BASE + base + i * 512));
        if self.selectors != expected {
            return Err(invalid());
        }
        let mut spans = Vec::with_capacity(LITERALS.len() + TABLES.len() + BANKS.len());
        for (&word, shape) in self.caller_literals.iter().zip(LITERALS) {
            spans.push(literal_span(word, shape, "color-mode-pair-literal")?);
        }
        for (&words, offset) in self.selectors.iter().zip(TABLES) {
            let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            spans.push((offset, bytes, "color-mode-pair-selectors".into()));
        }
        for (bank, shape) in self.banks.iter().zip(BANKS) {
            spans.push(bank.source_span(shape, "color-mode-pair-payload")?);
        }
        Ok(spans)
    }

    pub(super) fn metadata_byte(&self, offset: u32) -> Option<(u8, ByteSource)> {
        if let Some(byte) = literal_byte(&self.caller_literals, &LITERALS, offset) {
            return Some((byte, ByteSource::Literal));
        }
        self.selectors
            .iter()
            .zip(TABLES)
            .enumerate()
            .find_map(|(index, (words, root))| {
                let relative = offset.checked_sub(root)? as usize;
                let word = words.get(relative / 4)?;
                Some((
                    word.to_le_bytes()[relative % 4],
                    // The earlier row-pair table occupies the next group after SELECTORS.
                    ByteSource::Selector(SELECTORS.len() + 1 + index),
                ))
            })
    }
}
