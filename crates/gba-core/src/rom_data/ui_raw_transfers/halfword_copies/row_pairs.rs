//! Row-copy payloads and the six physical entries in the Color Mode table.
use super::{
    invalid, literal_byte, literal_span, rom_word, source_digest, Bank, ByteSource, SourceSpan,
    SELECTORS,
};
use serde::{Deserialize, Serialize};

const SOURCE_SHA256: &str = "25a64e36fb9ce75ec327efbc4d0843284f2da39f699d7442eadd009529f438de";
pub(super) const LITERALS: [(u32, u32); 11] = [
    (0x139b04, 0x08b558d4),
    (0x139b0c, 0x08b55dd4),
    (0x139b14, 0x08b56284),
    (0x139b24, 0x083adc88),
    (0x139bd4, 0x083adc88),
    (0x139c34, 0x083adc88),
    (0x13b30c, 0x083adc88),
    (0x13b930, 0x083adc88),
    (0x1404cc, 0x08b54f54),
    (0x1404d0, 0x000004d6),
    (0x1404dc, 0x08b54a58),
];
pub(super) const SELECTOR_OFFSET: u32 = 0x3adc88;
const POINTERS: [u32; 6] = [
    0x08b56734, 0x08b56934, 0x08b56f34, 0x08b57134, 0x08b56b34, 0x08b56d34,
];
pub(super) const SELECTOR_BYTES: usize = POINTERS.len() * 4;
// The second bank includes the 44-byte overlap shared by two original callers.
pub(super) const BANKS: [(u32, usize); 2] = [(0xb54a58, 1200), (0xb54f54, 9184)];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowPairCopies {
    pub caller_literals: [u32; 11],
    pub pointers: [u32; 6],
    pub banks: [Bank; 2],
}

impl RowPairCopies {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| rom_word(rom, offset)),
            pointers: std::array::from_fn(|i| rom_word(rom, SELECTOR_OFFSET + 4 * i as u32)),
            banks: BANKS.map(|shape| Bank::from_rom(rom, shape)),
        };
        if source_digest(value.source_spans()?) != SOURCE_SHA256 {
            return Err("Row-copy and Color Mode source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.pointers != POINTERS {
            return Err(invalid());
        }
        let mut spans = Vec::with_capacity(LITERALS.len() + BANKS.len() + 1);
        for (&word, shape) in self.caller_literals.iter().zip(LITERALS) {
            spans.push(literal_span(word, shape, "surrounding-copy-literal")?);
        }
        let bytes = self.pointers.iter().flat_map(|v| v.to_le_bytes()).collect();
        spans.push((SELECTOR_OFFSET, bytes, "paired-copy-selectors".into()));
        for (bank, shape) in self.banks.iter().zip(BANKS) {
            spans.push(bank.source_span(shape, "surrounding-copy-payload")?);
        }
        Ok(spans)
    }

    pub(super) fn metadata_byte(&self, offset: u32) -> Option<(u8, ByteSource)> {
        if let Some(value) = literal_byte(&self.caller_literals, &LITERALS, offset) {
            return Some((value, ByteSource::Literal));
        }
        let relative = offset.checked_sub(SELECTOR_OFFSET)? as usize;
        let word = self.pointers.get(relative / 4)?;
        Some((
            word.to_le_bytes()[relative % 4],
            ByteSource::Selector(SELECTORS.len()),
        ))
    }
}
