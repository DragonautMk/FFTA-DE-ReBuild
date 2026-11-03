//! EUR original name-entry archive, row widths and derived selectable cells.
use super::{
    text_catalogs::{Program, ProgramUnit},
    SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const ROOT: u32 = 0x4f95c4;
pub const END: u32 = 0x4f9744;
pub const ROW_WIDTH_ROOT: u32 = 0x3ad2fe;
pub const PAGES: usize = 3;
pub const ROWS: usize = 6;
pub const COLUMNS: usize = 10;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keyboard {
    pub offsets: Vec<u16>,
    pub row_widths: Vec<Vec<u8>>,
    pub programs: Vec<Program>,
    /// Original120-byte BEcode page; zero cells are outside the row widths.
    pub cells: Vec<Vec<u16>>,
}
impl Keyboard {
    fn page(program: &Program) -> Result<Vec<u16>, String> {
        if program.header != 0x60
            || program.encoded.is_some()
            || !program.decoded_padding.is_empty()
        {
            return Err("Unsupported name keyboard program header/encoding".into());
        }
        let mut cells = vec![0; ROWS * COLUMNS];
        let (mut row, mut column) = (0, 0);
        let mut ended = false;
        for unit in &program.units {
            if ended {
                return Err("Name keyboard data follows end command".into());
            }
            let code = match unit {
                ProgramUnit::Glyph { index } => Some(*index | 0x8000),
                ProgramUnit::Command {
                    opcode: 0x73,
                    operands,
                } if operands.is_empty() => Some(0x4073),
                ProgramUnit::Command {
                    opcode: 0x6e,
                    operands,
                } if operands.is_empty() => {
                    row += 1;
                    column = 0;
                    None
                }
                ProgramUnit::Command {
                    opcode: 0x63,
                    operands,
                } if operands.is_empty() => {
                    ended = true;
                    None
                }
                _ => return Err("Unsupported name keyboard cell command".into()),
            };
            if let Some(code) = code {
                if row >= ROWS || column >= COLUMNS {
                    return Err("Name keyboard cell overflow".into());
                }
                cells[row * COLUMNS + column] = code;
                column += 1;
            }
        }
        if !ended || row != ROWS - 1 {
            return Err("Missing name keyboard rows/end command".into());
        }
        Ok(cells)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.offsets.len() != PAGES + 1
            || self.offsets[0] != 8
            || self.offsets[PAGES] != 65535
            || self.offsets.windows(2).any(|v| v[0] >= v[1])
            || self.row_widths.len() != PAGES
            || self.programs.len() != PAGES
            || self.cells.len() != PAGES
        {
            return Err("Invalid name keyboard catalog".into());
        }
        for page in 0..PAGES {
            let p = &self.programs[page];
            let start = ROOT + u32::from(self.offsets[page]);
            let end = if page + 1 < PAGES {
                ROOT + u32::from(self.offsets[page + 1])
            } else {
                END
            };
            if p.source_offset != start
                || start
                    .checked_add(p.source_bytes()?.len() as u32)
                    .is_none_or(|v| v > end)
            {
                return Err("Invalid name keyboard program extent".into());
            }
            let cells = Self::page(p)?;
            if self.cells[page] != cells || self.row_widths[page].len() != ROWS {
                return Err("Name keyboard derived cell mismatch".into());
            }
            for row in 0..ROWS {
                let width = self.row_widths[page][row] as usize;
                if width == 0
                    || width > COLUMNS
                    || cells[row * COLUMNS..row * COLUMNS + width].contains(&0)
                    || cells[row * COLUMNS + width..(row + 1) * COLUMNS]
                        .iter()
                        .any(|&v| v != 0)
                {
                    return Err("Name keyboard row width/cells mismatch".into());
                }
            }
        }
        Ok(())
    }
    pub fn decode(input: &[u8], widths: &[u8]) -> Result<Self, String> {
        if input.len() != (END - ROOT) as usize || widths.len() != PAGES * ROWS {
            return Err("Invalid name keyboard input extents".into());
        }
        let offsets: Vec<_> = input[..8]
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes(v.try_into().unwrap()))
            .collect();
        let mut programs = Vec::new();
        let mut cells = Vec::new();
        for page in 0..PAGES {
            let at = offsets[page] as usize;
            let end = if page + 1 < PAGES {
                offsets[page + 1] as usize
            } else {
                input.len()
            };
            let (program, _) = Program::decode(
                input.get(at..end).ok_or("Keyboard index outside archive")?,
                ROOT + at as u32,
            )?;
            cells.push(Self::page(&program)?);
            programs.push(program);
        }
        let result = Self {
            offsets,
            row_widths: widths.chunks_exact(ROWS).map(<[u8]>::to_vec).collect(),
            programs,
            cells,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("Name keyboard differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: Keyboard,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Name keyboard requires exact EUR ROM; USA unverified".into());
        }
        let data = rom
            .get(ROOT as usize..END as usize)
            .ok_or("Truncated name keyboard archive")?;
        let widths = rom
            .get(ROW_WIDTH_ROOT as usize..ROW_WIDTH_ROOT as usize + PAGES * ROWS)
            .ok_or("Truncated keyboard row widths")?;
        let decoded = Keyboard::decode(data, widths)?;
        let mut sources = vec![
            SourceRange {
                offset: ROW_WIDTH_ROOT,
                length: 18,
                sha256: ffta_rom::digest(widths),
                role: "name-keyboard-row-widths".into(),
            },
            SourceRange {
                offset: ROOT,
                length: 8,
                sha256: ffta_rom::digest(&data[..8]),
                role: "name-keyboard-index".into(),
            },
        ];
        for (page, p) in decoded.programs.iter().enumerate() {
            let bytes = p.source_bytes()?;
            sources.push(SourceRange {
                offset: p.source_offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: format!("name-keyboard-program/{page}"),
            });
        }
        Ok(Self { decoded, sources })
    }
}
