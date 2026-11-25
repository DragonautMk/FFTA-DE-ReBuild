//! Rectangular tilemap sources for the original direct and staged draw paths.

use super::{literal_span, rom_word, source_digest, Bank, SourceSpan};
use serde::{Deserialize, Serialize};

pub(super) const SHAPES: [(u32, u8, u8); 3] =
    [(0x3b0e5c, 10, 8), (0x3b0efc, 12, 3), (0x3ba95c, 22, 17)];
pub(super) const LITERALS: [(u32, u32); 6] = [
    (0x06a6d4, 0x083b0e5c),
    (0x06a6dc, 0x083b0efc),
    (0x071534, 0x083b0e5c),
    (0x071800, 0x083b0efc),
    (0x12ee90, 0x0871e66c),
    (0x12eeac, 0x083ba95c),
];
const SOURCE_SHA256: &str = "94f4feacf27ec17822a2aa56fb0f195e36532326221e0ce797dd75300eefcfce";

pub(super) fn bank_range((offset, width, height): (u32, u8, u8)) -> (u32, usize) {
    (offset, 2 * usize::from(width) * usize::from(height))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TilemapRectangle {
    pub width: u8,
    pub height: u8,
    pub bank: Bank,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TilemapRectangles {
    pub caller_literals: [u32; LITERALS.len()],
    pub rectangles: [TilemapRectangle; SHAPES.len()],
}

impl TilemapRectangles {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_literals: LITERALS.map(|(offset, _)| rom_word(rom, offset)),
            rectangles: SHAPES.map(|shape| TilemapRectangle {
                width: shape.1,
                height: shape.2,
                bank: Bank::from_rom(rom, bank_range(shape)),
            }),
        };
        if source_digest(value.source_spans()?) != SOURCE_SHA256 {
            return Err("Tilemap rectangle source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = Vec::with_capacity(LITERALS.len() + SHAPES.len());
        for (&word, literal) in self.caller_literals.iter().zip(LITERALS) {
            spans.push(literal_span(word, literal, "tilemap-rectangle-literal")?);
        }
        for (rectangle, shape) in self.rectangles.iter().zip(SHAPES) {
            if (rectangle.width, rectangle.height) != (shape.1, shape.2) {
                return Err("Invalid tilemap rectangle dimensions".into());
            }
            spans.push(
                rectangle
                    .bank
                    .source_span(bank_range(shape), "tilemap-rectangle-cells")?,
            );
        }
        Ok(spans)
    }
}
