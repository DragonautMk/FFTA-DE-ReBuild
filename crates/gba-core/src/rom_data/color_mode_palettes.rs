//! Bounded EUR unit/UI color sources; live selection, fades and RAM copies stay guest work.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_SHA256: &str = "2e7a4ae78f6841669ed2f11fddd3635d119c9af6edd6602fb3e34f607314485c";
pub const SLICE_OFFSETS_SOURCE: u32 = 0x394e04;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Book {
    UnitBase,
    CompanionBase,
    Supplemental,
    SharedColors,
    SpecialUnit,
}
impl Book {
    pub fn role(self) -> &'static str {
        match self {
            Self::UnitBase => "color-mode/unit-base",
            Self::CompanionBase => "color-mode/companion-base",
            Self::Supplemental => "color-mode/supplemental",
            Self::SharedColors => "color-mode/shared-colors",
            Self::SpecialUnit => "color-mode/special-unit",
        }
    }

    /// Original pointer arithmetic, including unowned indices. This is not an
    /// availability check or a bound on the guest's live arguments.
    pub fn source(self, mode: u8, argument: u32) -> u32 {
        let mode = match mode & 3 {
            1 => 1,
            2 => 2,
            _ => 0,
        };
        let index = u32::from(argument as u8);
        let roots = match self {
            Self::UnitBase => [0x40f290, 0x4108f0, 0x40fe10],
            Self::CompanionBase => [0x40f490, 0x40f490, 0x410010],
            Self::Supplemental => [0x40f690, 0x410cf0, 0x410210],
            Self::SharedColors => return [0x410810, 0x4112f0, 0x40fc90][mode],
            Self::SpecialUnit => [0x40fa90, 0x4110f0, 0x410610],
        };
        let root = roots[mode];
        // CF51C has distinct literals below/above index16; both branches have
        // this exact address relation. Keep the split explicit for provenance.
        if self == Self::Supplemental && index >= 16 {
            root + 512 + 32 * (index - 16)
        } else {
            root + 32 * index
        }
    }
}

/// Owned physical fragments, not complete books or live guest index limits.
pub fn definitions() -> Vec<(Book, u8, u32, usize)> {
    let mut out = Vec::new();
    for (book, variants, length) in [
        (Book::UnitBase, &[0, 1, 2][..], 512),
        (Book::CompanionBase, &[0, 2][..], 512),
        (Book::SharedColors, &[0, 1, 2][..], 12),
        (Book::SpecialUnit, &[0, 1, 2][..], 288),
    ] {
        for &variant in variants {
            out.push((book, variant, book.source(variant, 0), length));
        }
    }
    for variant in [0, 1, 2] {
        let base = Book::Supplemental.source(variant, 0);
        for (delta, length) in [
            (0, 128),
            (132, 28),
            (164, 28),
            (256, 480),
            (760, 2),
            (768, 96),
        ] {
            out.push((Book::Supplemental, variant, base + delta, length));
        }
    }
    out.sort_by_key(|v| v.2);
    out
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorSpan {
    pub id: String,
    pub book: Book,
    /// Canonical physical variant; companion mode1 aliases variant0.
    pub variant: u8,
    pub source_offset: u32,
    /// Lossless little-endian BGR555 storage words, including bit15.
    pub colors: Vec<u16>,
}
impl ColorSpan {
    pub fn byte(&self, relative: usize) -> Option<u8> {
        Some(self.colors.get(relative / 2)?.to_le_bytes()[relative % 2])
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.colors.iter().flat_map(|v| v.to_le_bytes()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorModePalettes {
    pub spans: Vec<ColorSpan>,
    /// Four rooted low8 arguments1..4 of 23DB4. Other arguments remain raw.
    pub slice_offsets: [u8; 4],
}
impl ColorModePalettes {
    pub fn validate(&self) -> Result<(), String> {
        let definitions = definitions();
        if self.spans.len() != definitions.len() {
            return Err("Color-Mode palette fragment count mismatch".into());
        }
        for (span, (book, variant, offset, length)) in self.spans.iter().zip(definitions) {
            if span.id != format!("color-{offset:06x}")
                || span.book != book
                || span.variant != variant
                || span.source_offset != offset
                || span.colors.len() != length / 2
            {
                return Err("Color-Mode palette ID/book/variant/source/extent mismatch".into());
            }
        }
        // Full u16 color and u8 selector values are retained. Authentication,
        // rather than a guessed color/selector constraint, establishes source equality.
        Ok(())
    }
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        if let Some(relative) = at.checked_sub(SLICE_OFFSETS_SOURCE) {
            if let Some(&byte) = self.slice_offsets.get(relative as usize) {
                return Some(byte);
            }
        }
        let n = self
            .spans
            .partition_point(|span| span.source_offset <= at)
            .checked_sub(1)?;
        let span = self.spans.get(n)?;
        span.byte(at.checked_sub(span.source_offset)? as usize)
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        let mut out = vec![(
            SLICE_OFFSETS_SOURCE,
            self.slice_offsets.to_vec(),
            "color-mode/slice-offsets".into(),
        )];
        out.extend(self.spans.iter().map(|span| {
            (
                span.source_offset,
                span.bytes(),
                span.book.role().to_owned(),
            )
        }));
        Ok(out)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Color-Mode palettes differ from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ColorModePalettes,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err(
                "Color-Mode palettes require exact European source ROM; USA pending".into(),
            );
        }
        let decoded = ColorModePalettes {
            spans: definitions()
                .into_iter()
                .map(|(book, variant, offset, length)| ColorSpan {
                    id: format!("color-{offset:06x}"),
                    book,
                    variant,
                    source_offset: offset,
                    colors: rom[offset as usize..offset as usize + length]
                        .chunks_exact(2)
                        .map(|v| u16::from_le_bytes(v.try_into().unwrap()))
                        .collect(),
                })
                .collect(),
            slice_offsets: rom[SLICE_OFFSETS_SOURCE as usize..SLICE_OFFSETS_SOURCE as usize + 4]
                .try_into()
                .unwrap(),
        };
        let mut combined = Vec::new();
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Color-Mode palette source roundtrip mismatch".into());
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
            return Err("Color-Mode palette source hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
