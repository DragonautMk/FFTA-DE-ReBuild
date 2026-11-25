//! Constructor defaults and counted lists used by the European state initializers.
use super::{
    invalid, literal_byte, literal_span, rom_word, source_digest, Bank, ByteSource, SourceSpan,
};
use crate::rom_data::soil_indirect_records::{IndirectRecord, RECORD_BYTES};
use serde::{Deserialize, Serialize};

const SOURCE_SHA256: &str = "dfac14e9e4b396182b232b1cf9847d7e0d67f4a27edee9b44b39209751b7d4ec";
pub(super) const LITERALS: [(u32, u32); 6] = [
    (0x03ebb4, 0x083954f0),
    (0x0ee27c, 0x0839db64),
    (0x0ee354, 0x0839db90),
    (0x0ee3c8, 0x0839dbb0),
    (0x0ee57c, 0x0839db24),
    (0x0ee584, 0x0839db88),
];
pub(super) const DIRECT_BANKS: [(u32, usize, &str); 3] = [
    (0x3954f0, 28, "constructor-defaults"),
    (0x39db24, 64, "state-list-records"),
    (0x39db64, 24, "state-rectangle-records"),
];
pub(super) const INDIRECT_START: u32 = 0x39db88;
const INDIRECT_COUNT: usize = 8;
pub(super) const INDIRECT_BYTES: usize = INDIRECT_COUNT * RECORD_BYTES;
const INDIRECT_GROUPS: [std::ops::Range<usize>; 3] = [0..1, 1..5, 5..8];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateInitializers {
    pub caller_literals: [u32; LITERALS.len()],
    pub direct_banks: [Bank; DIRECT_BANKS.len()],
    pub indirect_records: [IndirectRecord; INDIRECT_COUNT],
}

fn indirect_offset(index: usize) -> u32 {
    INDIRECT_START + (index * RECORD_BYTES) as u32
}

impl StateInitializers {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| rom_word(rom, offset)),
            direct_banks: DIRECT_BANKS
                .map(|(offset, bytes, _)| Bank::from_rom(rom, (offset, bytes))),
            indirect_records: std::array::from_fn(|i| {
                IndirectRecord::from_rom(rom, indirect_offset(i))
            }),
        };
        if source_digest(value.source_spans()?) != SOURCE_SHA256 {
            return Err("State initializer source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans =
            Vec::with_capacity(LITERALS.len() + DIRECT_BANKS.len() + INDIRECT_GROUPS.len());
        for (&word, shape) in self.caller_literals.iter().zip(LITERALS) {
            spans.push(literal_span(word, shape, "state-initializer-literal")?);
        }
        for (bank, (offset, bytes, role)) in self.direct_banks.iter().zip(DIRECT_BANKS) {
            spans.push(bank.source_span((offset, bytes), role)?);
        }
        for (i, record) in self.indirect_records.iter().enumerate() {
            if record.offset != indirect_offset(i) {
                return Err(invalid());
            }
        }
        for group in INDIRECT_GROUPS {
            let offset = indirect_offset(group.start);
            let bytes = self.indirect_records[group]
                .iter()
                .flat_map(IndirectRecord::source_bytes)
                .collect();
            spans.push((offset, bytes, "state-indirect-records".into()));
        }
        Ok(spans)
    }

    pub(super) fn byte(&self, offset: u32) -> Option<(u8, ByteSource)> {
        if let Some(value) = literal_byte(&self.caller_literals, &LITERALS, offset) {
            return Some((value, ByteSource::Literal));
        }
        if let Some(value) = self.direct_banks.iter().find_map(|bank| bank.byte(offset)) {
            return Some((value, ByteSource::Payload));
        }
        let relative = offset.checked_sub(INDIRECT_START)? as usize;
        let record = self.indirect_records.get(relative / RECORD_BYTES)?;
        Some((
            record.source_bytes()[relative % RECORD_BYTES],
            ByteSource::Payload,
        ))
    }
}
