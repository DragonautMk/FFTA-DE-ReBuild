//! Bounded EUR raster palette sources. Live windows, copies and DMA stay guest work.
use super::{bg_descriptors, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x4872b0;
pub const SOURCE_END: u32 = 0x489bb0;
pub const PROGRAM_COUNT: usize = 32;
pub const COLOR_COUNT: usize = 160;
pub const INDEX_BYTES: usize = PROGRAM_COUNT * 2;
pub const RECORD_BYTES: usize = 6 + COLOR_COUNT * 2;
pub const SOURCE_SHA256: &str = "6570159e474457326a39cc65bab5530c9467f74470d4c4a2a0f2cb2e19e13df3";

/// Original initializer arithmetic, not a bound on live guest selectors.
pub fn initializer_index_offset(raw: u32) -> u32 {
    SOURCE_OFFSET + u32::from(raw as u16) * 2
}

/// Original pointer getter gates on the current selector and sign-extends its index.
pub fn getter_index_offset(raw: u32, current: u16) -> Option<u32> {
    if current == 0xff {
        return None;
    }
    let index = if raw as u16 == 0xffff {
        current
    } else {
        raw as u16
    };
    Some(SOURCE_OFFSET.wrapping_add((i32::from(index as i16) * 2) as u32))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RasterHeader {
    pub color_count: u16,
    /// Original live+C/+E formulas use these words; full camera semantics are unclassified.
    pub range_length: u16,
    pub range_start: u16,
}
impl RasterHeader {
    pub fn words(&self) -> [u16; 3] {
        [self.color_count, self.range_length, self.range_start]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RasterPalette {
    pub id: u16,
    pub source_offset: u32,
    pub header: RasterHeader,
    /// Ordered indexed BGR555 words, retaining bit15 and physical record identity.
    pub colors: Vec<u16>,
}
impl RasterPalette {
    pub fn byte(&self, relative: usize) -> Option<u8> {
        let word = if relative < 6 {
            self.header.words()[relative / 2]
        } else {
            *self.colors.get((relative - 6) / 2)?
        };
        Some(word.to_le_bytes()[relative % 2])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapRasterPalettes {
    pub source_offset: u32,
    pub programs: Vec<RasterPalette>,
    /// Actual field54/55 links in descriptor order, not a new selector policy.
    pub descriptor_programs: Vec<[u16; 2]>,
}
impl MapRasterPalettes {
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        if !(SOURCE_OFFSET..SOURCE_END).contains(&at) {
            return None;
        }
        let relative = at.checked_sub(self.source_offset)? as usize;
        if relative < INDEX_BYTES {
            let target = self.programs.get(relative / 2)?.source_offset;
            let offset = u16::try_from(target.checked_sub(self.source_offset)?).ok()?;
            return Some(offset.to_le_bytes()[relative % 2]);
        }
        let index = self
            .programs
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)?;
        let program = self.programs.get(index)?;
        program.byte(at.checked_sub(program.source_offset)? as usize)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET
            || self.programs.len() != PROGRAM_COUNT
            || self.descriptor_programs.len() != bg_descriptors::BASE_RECORDS
        {
            return Err("Map raster palette source/count mismatch".into());
        }
        let mut next = SOURCE_OFFSET + INDEX_BYTES as u32;
        for (id, program) in self.programs.iter().enumerate() {
            if usize::from(program.id) != id
                || program.source_offset != next
                || usize::from(program.header.color_count) != COLOR_COUNT
                || program.colors.len() != COLOR_COUNT
            {
                return Err("Map raster palette ID/header/extent mismatch".into());
            }
            next += RECORD_BYTES as u32;
        }
        if next != SOURCE_END
            || self
                .descriptor_programs
                .iter()
                .flatten()
                .any(|&id| usize::from(id) >= PROGRAM_COUNT)
        {
            return Err("Map raster palette coverage/link mismatch".into());
        }
        Ok(())
    }

    pub fn validate_dependencies(
        &self,
        descriptors: &bg_descriptors::Descriptors,
    ) -> Result<(), String> {
        self.validate()?;
        descriptors.validate(0x7662a8, 0x1000000)?;
        if descriptors.entries.len() != self.descriptor_programs.len()
            || descriptors
                .entries
                .iter()
                .zip(&self.descriptor_programs)
                .any(|(entry, ids)| *ids != entry.selectors.map(u16::from))
        {
            return Err("Map raster palette descriptor links mismatch".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        let mut spans = Vec::with_capacity(PROGRAM_COUNT + 1);
        let index = (SOURCE_OFFSET..SOURCE_OFFSET + INDEX_BYTES as u32)
            .map(|at| self.byte_at(at).ok_or("Invalid raster directory byte"))
            .collect::<Result<Vec<_>, _>>()?;
        spans.push((SOURCE_OFFSET, index, "map-raster-palette-index".into()));
        for program in &self.programs {
            let bytes = (0..RECORD_BYTES)
                .map(|n| program.byte(n).ok_or("Invalid raster program byte"))
                .collect::<Result<Vec<_>, _>>()?;
            spans.push((
                program.source_offset,
                bytes,
                format!("map-raster-palette-program/{}", program.id),
            ));
        }
        Ok(spans)
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Map raster palettes differ from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: MapRasterPalettes,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Map raster palettes require exact European source ROM".into());
        }
        for at in [0x1eb2c, 0x1ee24] {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != 0x08000000 + SOURCE_OFFSET
            {
                return Err("Map raster palette root literal mismatch".into());
            }
        }
        for (at, op) in [
            (0x1eb18, 0x8809u16),
            (0x1eb1c, 0x1daf),
            (0x1eb36, 0x22a1),
            (0x1eb46, 0x8828),
            (0x1eb4a, 0x88a8),
            (0x1eb4e, 0x8868),
            (0x1ee16, 0x13c0),
            (0x145230, 0xdf0b),
        ] {
            if u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) != op {
                return Err("Map raster palette reader/copy contract mismatch".into());
            }
        }
        let h = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let mut programs = Vec::with_capacity(PROGRAM_COUNT);
        for id in 0..PROGRAM_COUNT {
            let start = SOURCE_OFFSET as usize + usize::from(h(SOURCE_OFFSET as usize + id * 2));
            let end = start
                .checked_add(RECORD_BYTES)
                .ok_or("Raster source overflow")?;
            if start < SOURCE_OFFSET as usize + INDEX_BYTES || end > SOURCE_END as usize {
                return Err("Map raster palette outside declared source".into());
            }
            programs.push(RasterPalette {
                id: id as u16,
                source_offset: start as u32,
                header: RasterHeader {
                    color_count: h(start),
                    range_length: h(start + 2),
                    range_start: h(start + 4),
                },
                colors: (start + 6..end).step_by(2).map(h).collect(),
            });
        }
        let descriptors = bg_descriptors::Catalog::from_rom(rom)?.decoded;
        let decoded = MapRasterPalettes {
            source_offset: SOURCE_OFFSET,
            programs,
            descriptor_programs: descriptors
                .entries
                .iter()
                .map(|e| e.selectors.map(u16::from))
                .collect(),
        };
        decoded.validate_dependencies(&descriptors)?;
        let mut sources = Vec::new();
        let mut combined = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Map raster palette source roundtrip mismatch".into());
            }
            combined.extend_from_slice(&bytes);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        if ffta_rom::digest(&combined) != SOURCE_SHA256 {
            return Err("Map raster palette source hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
