//! Adjoining EUR label IDs and calendar offsets read by the original map code.

use super::SourceSpan;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const LABEL_TABLE: u32 = 0x3955ec;
pub const CALENDAR_TABLE: u32 = 0x39562a;
const LABEL_COUNT: usize = 31;
const CALENDAR_COUNT: usize = 26;
const SOURCE_SHA256: &str = "2dbe771cf5bf2a2fd1f005ffdd6832f386e8c81b8cf8427029f66171383ba0a9";
const ROM_BASE: u32 = 0x0800_0000;
const LITERALS: [(u32, u32); 3] = [
    (0x033cdc, ROM_BASE + LABEL_TABLE),
    (0x037264, ROM_BASE + CALENDAR_TABLE),
    (0x03ef10, ROM_BASE + LABEL_TABLE),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldMapLookups {
    pub caller_literals: [u32; LITERALS.len()],
    pub label_ids: [u16; LABEL_COUNT],
    pub calendar_offsets: [u8; CALENDAR_COUNT],
}

impl WorldMapLookups {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| {
                let at = offset as usize;
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            }),
            label_ids: std::array::from_fn(|slot| {
                let at = LABEL_TABLE as usize + 2 * slot;
                u16::from_le_bytes([rom[at], rom[at + 1]])
            }),
            calendar_offsets: rom
                [CALENDAR_TABLE as usize..CALENDAR_TABLE as usize + CALENDAR_COUNT]
                .try_into()
                .unwrap(),
        };
        let bytes: Vec<_> = value
            .source_spans()?
            .into_iter()
            .flat_map(|(_, bytes, _)| bytes)
            .collect();
        if ffta_rom::digest(&bytes) != SOURCE_SHA256 {
            return Err("World map lookup source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = Vec::with_capacity(LITERALS.len() + 2);
        for (&word, (offset, expected)) in self.caller_literals.iter().zip(LITERALS) {
            if word != expected {
                return Err("World map lookup literal mismatch".into());
            }
            spans.push((
                offset,
                word.to_le_bytes().to_vec(),
                "world-map-lookup-literal".into(),
            ));
        }
        // Keep the original boundary. The controlled label and calendar paths
        // can each read bytes belonging to the neighboring table.
        spans.push((
            LABEL_TABLE,
            self.label_ids
                .iter()
                .flat_map(|id| id.to_le_bytes())
                .collect(),
            "world-map-label-ids".into(),
        ));
        spans.push((
            CALENDAR_TABLE,
            self.calendar_offsets.to_vec(),
            "world-map-calendar-offsets".into(),
        ));
        Ok(spans)
    }
}

pub(crate) fn is_source_start(offset: u32) -> bool {
    offset == LABEL_TABLE
        || offset == CALENDAR_TABLE
        || LITERALS.iter().any(|&(at, _)| at == offset)
}

pub(crate) fn source_page(index: usize) -> bool {
    let page = index >> 12;
    page == LABEL_TABLE as usize >> 12 || LITERALS.iter().any(|&(at, _)| page == at as usize >> 12)
}
