//! Two selected EUR Firaga palette row prefixes; later rows retain ROM fallback.
use super::{
    script39_registry_poses::PriorSources, Resource, SourceRange, FIRAGA_TRANSFER_PROGRAM_ID,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const ROW_OFFSETS: [u32; 2] = [0xae6db0, 0xae6db8];
const HASHES: [&str; 2] = [
    "341a649269d7b4e78ecd65fa15a04a02561357604759ee66f851370c17d88abb",
    "e41e062af107a66b6f98d71eeb3a1afb0cd64401e15978fc1589486d1a00af3c",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteRow {
    pub declared_rows: u8,
    pub width: u8,
    pub colors: [u16; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaPaletteRows {
    pub rows: [PaletteRow; 2],
}
impl FiragaPaletteRows {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .rows
            .iter()
            .map(|r| (r.declared_rows, r.width))
            .collect::<Vec<_>>()
            != [(1, 3), (3, 3)]
        {
            return Err("Invalid Firaga selected palette row headers".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        Ok(self
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let mut bytes = vec![row.declared_rows, row.width];
                for color in row.colors {
                    bytes.extend(color.to_le_bytes());
                }
                (
                    ROW_OFFSETS[i],
                    bytes,
                    if i == 0 {
                        "firaga-palette-row-0"
                    } else {
                        "firaga-palette-row-1"
                    },
                )
            })
            .collect())
    }
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize)> {
        for (i, row) in self.rows.iter().enumerate() {
            if let Some(n) = offset.checked_sub(ROW_OFFSETS[i]) {
                match n {
                    0 => return Some((row.declared_rows, i)),
                    1 => return Some((row.width, i)),
                    2..=7 => {
                        let n = (n - 2) as usize;
                        return Some((row.colors[n / 2].to_le_bytes()[n % 2], i));
                    }
                    _ => {}
                }
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga palette rows differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FiragaPaletteRows,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga palette rows require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let checks = [
            (0xdb39e, 0x6820),
            (0xdb3a0, 0x7921),
            (0xdb3a2, 0x7962),
            (0xdb3a4, 0x79a3),
            (0xdb3a6, 0x79e4),
            (0xdb3a8, 0x9400),
            (0xdb3aa, 0xf7fc),
            (0xdb3ac, 0xfe99),
            (0xd80e8, 0x1c07),
            (0xd80ea, 0x9807),
            (0xd8114, 0x1cbd),
            (0xd811c, 0x7878),
            (0xd811e, 0x0041),
            (0xd8136, 0x787a),
            (0xd8138, 0x0052),
            (0xd8142, 0xf06d),
            (0xd8144, 0xfd67),
            (0xd8194, 0x7828),
        ];
        if checks.iter().any(|&(at, val)| half(at) != val)
            || word(0xd8168) != 0x08370eb4
            || word(0x370eb4) != 0x03005ee9
            || word(0x39caec) != 0x08ae6db8
            || rom[0x39caf0..0x39caf4] != [7, 1, 0, 1]
            || word(0x39caf4) != 0x08ae6db0
            || rom[0x39caf8..0x39cafc] != [6, 1, 0, 1]
        {
            return Err("Firaga palette row original consumer contract mismatch".into());
        }
        let decoded = FiragaPaletteRows {
            rows: std::array::from_fn(|i| {
                let a = ROW_OFFSETS[i] as usize;
                PaletteRow {
                    declared_rows: rom[a],
                    width: rom[a + 1],
                    colors: std::array::from_fn(|j| half(a + 2 + 2 * j)),
                }
            }),
        };
        let prior = PriorSources::new(previous)?;
        if !previous.is_empty() {
            let program = previous
                .iter()
                .find(|r| r.id == FIRAGA_TRANSFER_PROGRAM_ID)
                .ok_or("Missing Firaga transfer program owner")?;
            if !program
                .sources
                .iter()
                .any(|s| s.offset == 0x39caec && s.length == 16)
            {
                return Err("Missing Firaga palette producer entries".into());
            }
        }
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga palette row source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga palette row source hash mismatch".into());
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
