//! Authenticated EUR Soil Evidence rectangles and their alternative literal.
use super::{
    observed_tilemap::Rectangle, script39_registry_poses::PriorSources, Resource, SourceRange,
    THUMB_PC_LITERALS_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const ALT_LITERAL: u32 = 0x0e60d8;
pub const RECTS: [(u32, u8, u8); 2] = [(0xae7810, 7, 7), (0xae7874, 1, 3)];
const HASHES: [&str; 3] = [
    "020c8a48f097da89cd0118647b1eca2a72fc3ca1014b79e2f4b3143abbf1c39d",
    "26d3c460ba26479a922785eb96766c67cf7fa4e3967ab6843b4994b5fc63cd83",
    "5b9a23b985e46443d9246653fdc83336959ba4e896cad50468ceebb9598dc0f0",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoilRemainingRectangles {
    pub alternate_literal: u32,
    pub rectangles: [Rectangle; 2],
}
impl SoilRemainingRectangles {
    pub fn validate(&self) -> Result<(), String> {
        if self.alternate_literal != 0x0800_0000 + RECTS[1].0 {
            return Err("Invalid Soil alternate rectangle pointer".into());
        }
        for (row, (offset, height, width)) in self.rectangles.iter().zip(RECTS) {
            if (row.source_offset, row.height, row.width) != (offset, height, width)
                || row.cells.len() != usize::from(height) * usize::from(width)
            {
                return Err("Invalid Soil rectangle geometry".into());
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(vec![
            (
                ALT_LITERAL,
                self.alternate_literal.to_le_bytes().to_vec(),
                "soil-remaining-alternate-literal",
            ),
            (
                RECTS[0].0,
                self.rectangles[0].source_bytes(),
                "soil-remaining-rectangle-0",
            ),
            (
                RECTS[1].0,
                self.rectangles[1].source_bytes(),
                "soil-remaining-rectangle-1",
            ),
        ])
    }
    /// Physical ownership only: the guest retains its original pointer and bounds behavior.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        if let Some(at) = offset.checked_sub(ALT_LITERAL) {
            if let Some(byte) = self.alternate_literal.to_le_bytes().get(at as usize) {
                return Some((*byte, 0));
            }
        }
        for (i, row) in self.rectangles.iter().enumerate() {
            let Some(at) = offset.checked_sub(row.source_offset).map(|n| n as usize) else {
                continue;
            };
            if at == 0 {
                return Some((row.height, i + 1));
            }
            if at == 1 {
                return Some((row.width, i + 1));
            }
            if let Some(cell) = row.cells.get((at - 2) / 2) {
                return Some((cell.to_le_bytes()[(at - 2) % 2], i + 1));
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Soil remaining rectangles differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: SoilRemainingRectangles,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Soil remaining rectangles require the exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0xe617e,
                0xe6184,
                "15aa3baf918046b9b2198e547c8966a819d30b8ab082f4b94abdc83129eb3c4f",
            ),
            (
                0xe6196,
                0xe61a4,
                "818e07a7fe0720355961c9c24bbf40057b34501321663c8ea27edb64760b3efc",
            ),
            (
                0xe5fe8,
                0xe5fee,
                "994cea5db84f372071a5d770125ee55da05e1fb91d123d18d63130a058e05b61",
            ),
            (
                0xe60c8,
                0xe60d6,
                "e285eb7a8623c73a763840401aa379bedcd4435715526b8d2dc82a724c3a0354",
            ),
            (
                0xd80e0,
                0xd8164,
                "f802daafd73dab107c4269b328430eec39b7c506f02aaa95abd0f8b0ec6fc883",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Soil rectangle reader fingerprint mismatch".into());
            }
        }
        let roots = [
            (0xe617eu32, 0xe6184u32, RECTS[0].0, true),
            (0xe5fe8, 0xe5ff0, RECTS[1].0, true),
            (0xe60c8, ALT_LITERAL, RECTS[1].0, false),
        ];
        let literals = previous.iter().find(|r| r.id == THUMB_PC_LITERALS_ID);
        if !previous.is_empty() && literals.is_none() {
            return Err("Missing Soil rectangle literal dependency".into());
        }
        for (pc, literal, target, owned) in roots {
            let op = u16::from_le_bytes(rom[pc as usize..pc as usize + 2].try_into().unwrap());
            let root = u32::from_le_bytes(
                rom[literal as usize..literal as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + u32::from(op & 255) * 4 != literal
                || root != 0x0800_0000 + target
            {
                return Err("Soil rectangle literal mismatch".into());
            }
            if owned
                && literals.is_some_and(|owner| {
                    !owner
                        .sources
                        .iter()
                        .any(|s| s.offset == literal && s.length == 4)
                })
            {
                return Err("Missing separately owned Soil rectangle literal".into());
            }
        }
        let rectangles = std::array::from_fn(|i| {
            let (offset, height, width) = RECTS[i];
            let start = offset as usize + 2;
            let cells = (0..usize::from(height) * usize::from(width))
                .map(|j| {
                    u16::from_le_bytes(rom[start + j * 2..start + j * 2 + 2].try_into().unwrap())
                })
                .collect();
            Rectangle {
                source_offset: offset,
                height,
                width,
                cells,
            }
        });
        let decoded = SoilRemainingRectangles {
            alternate_literal: u32::from_le_bytes(
                rom[ALT_LITERAL as usize..ALT_LITERAL as usize + 4]
                    .try_into()
                    .unwrap(),
            ),
            rectangles,
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Soil rectangle source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Soil rectangle source hash mismatch".into());
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
