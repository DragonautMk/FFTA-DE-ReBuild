//! Index tables used by the original indexed decode and VRAM copy routines.

use super::SourceSpan;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const PROGRAM_TABLE: u32 = 0x397e70;
pub const TILE_ORDER: u32 = 0x397f24;
const PROGRAM_SLOTS: usize = 90;
const TILE_SLOTS: usize = 42;
const SOURCE_SHA256: &str = "bbd939186e50fc52ac7b9f29c87c10747bc9b4ad3171dee9d6d6fe68dcdffd5d";
const LITERALS: [(u32, u32); 5] = [
    (0x08a9c4, 0x0800_0000 + PROGRAM_TABLE),
    (0x08aa44, 0x0800_0000 + PROGRAM_TABLE),
    (0x08ab7c, 0x0800_0000 + PROGRAM_TABLE),
    (0x08e8f4, 0x0800_0000 + TILE_ORDER),
    (0x091c4c, 0x0800_0000 + TILE_ORDER),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VramSelectors {
    pub caller_literals: [u32; 5],
    pub program_indices: Vec<u16>,
    pub tile_order: Vec<u8>,
}

impl VramSelectors {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let start = PROGRAM_TABLE as usize;
        let order = TILE_ORDER as usize;
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| {
                let at = offset as usize;
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            }),
            program_indices: rom[start..start + 2 * PROGRAM_SLOTS]
                .chunks_exact(2)
                .map(|word| u16::from_le_bytes([word[0], word[1]]))
                .collect(),
            tile_order: rom[order..order + TILE_SLOTS].to_vec(),
        };
        let bytes: Vec<_> = value
            .source_spans()?
            .into_iter()
            .flat_map(|(_, bytes, _)| bytes)
            .collect();
        if ffta_rom::digest(&bytes) != SOURCE_SHA256 {
            return Err("Indexed VRAM selector source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut order = self.tile_order.clone();
        order.sort_unstable();
        if self.program_indices.len() != PROGRAM_SLOTS
            || self.program_indices.iter().any(|&index| index >= 54)
            || !order.into_iter().eq(2..44)
        {
            return Err("Invalid indexed VRAM selector table".into());
        }
        let mut spans = Vec::with_capacity(LITERALS.len() + 2);
        for (&word, (offset, expected)) in self.caller_literals.iter().zip(LITERALS) {
            if word != expected {
                return Err("Indexed VRAM selector literal mismatch".into());
            }
            spans.push((
                offset,
                word.to_le_bytes().to_vec(),
                "indexed-vram-source-literal".into(),
            ));
        }
        spans.push((
            PROGRAM_TABLE,
            self.program_indices
                .iter()
                .flat_map(|index| index.to_le_bytes())
                .collect(),
            "indexed-vram-program-table".into(),
        ));
        spans.push((
            TILE_ORDER,
            self.tile_order.clone(),
            "indexed-vram-tile-order".into(),
        ));
        Ok(spans)
    }
}

pub(crate) fn is_source_start(offset: u32) -> bool {
    offset == PROGRAM_TABLE || offset == TILE_ORDER || LITERALS.iter().any(|&(at, _)| at == offset)
}

pub(crate) fn source_page(index: usize) -> bool {
    let page = index >> 12;
    page == PROGRAM_TABLE as usize >> 12
        || LITERALS.iter().any(|&(at, _)| page == at as usize >> 12)
}
