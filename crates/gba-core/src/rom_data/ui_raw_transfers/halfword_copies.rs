//! Source banks read by the European copy routines.
use super::{CopyLayout, SourceSpan};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

mod color_mode_pairs;
pub use color_mode_pairs::ColorModePairs;
mod row_pairs;
pub use row_pairs::RowPairCopies;
mod scanline_blend;
pub use scanline_blend::ScanlineBlendCopies;
mod scroll_register_tables;
pub use scroll_register_tables::ScrollRegisterTables;
mod state_initializers;
pub use state_initializers::StateInitializers;
mod tilemap_rectangles;
pub use tilemap_rectangles::{TilemapRectangle, TilemapRectangles};

pub(crate) const MAX_TRACKED_BANKS: usize = BANKS.len()
    + PRELOAD_BANKS.len()
    + LOADER_BANKS.len()
    + row_pairs::BANKS.len()
    + color_mode_pairs::BANKS.len()
    + 1 // Scanline blend bank.
    + tilemap_rectangles::SHAPES.len();

const SOURCE_SHA256: &str = "d4c4023518de3b92cf0726d9fbe29022b75283967b91a41ec62872b506e303ae";
const PRELOAD_SHA256: &str = "ccc48cd0dd6bf470cabc8047bab95268a125f8b8ddff977b1455674182916643";
const LOADER_SHA256: &str = "c22a547906433a51de41457ff0cff2fb712fb931fb19c4bdcadfc4f47f3cda6c";
const LITERALS: [(u32, u32); 11] = [
    (0x039d28, 0x08b5376c),
    (0x039d38, 0x08b53714),
    (0x039dc8, 0x08b53740),
    (0x075c98, 0x08397d44),
    (0x075ce0, 0x08397d58),
    (0x077f90, 0x08b4c980),
    (0x078a90, 0x08b4c980),
    (0x0884ac, 0x08b4d0f8),
    (0x089b54, 0x08b4d0f8),
    (0x08e7cc, 0x08b4cb00),
    (0x08eec4, 0x08b5376c),
];
const SELECTORS: [(u32, [u32; 5]); 2] = [
    (
        0x397d44,
        [0x08b4d12c, 0x08b4d1a4, 0x08b4d21c, 0x08b4d294, 0x08b4d30c],
    ),
    (
        0x397d58,
        [0x08b4d384, 0x08b4d3fc, 0x08b4d474, 0x08b4d4ec, 0x08b4d564],
    ),
];
const BANKS: [(u32, usize); 16] = [
    (0xb4c980, 384),
    (0xb4cb00, 448),
    (0xb4d0f8, 52),
    (0xb4d12c, 120),
    (0xb4d1a4, 120),
    (0xb4d21c, 120),
    (0xb4d294, 120),
    (0xb4d30c, 120),
    (0xb4d384, 120),
    (0xb4d3fc, 120),
    (0xb4d474, 120),
    (0xb4d4ec, 120),
    (0xb4d564, 120),
    (0xb53714, 44),
    (0xb53740, 44),
    (0xb5376c, 60),
];
const PRELOAD_LITERALS: [(u32, u32); 3] = [
    (0x075c34, 0x08b4ccc0),
    (0x075c38, 0x08b4cdbc),
    (0x077c28, 0x08b4ceb8),
];
const PRELOAD_BANKS: [(u32, usize); 3] = [(0xb4ccc0, 252), (0xb4cdbc, 252), (0xb4ceb8, 576)];
const LOADER_LITERALS: [(u32, u32); 11] = [
    (0x139adc, 0x08b57334),
    (0x139ae0, 0x08b5f41c),
    (0x139ae4, 0x06008000),
    (0x139ae8, 0x00001cd4),
    (0x139aec, 0x08b65a10),
    (0x139af0, 0x06010000),
    (0x139bb8, 0x08b65a10),
    (0x139bbc, 0x06010000),
    (0x139bc0, 0x08b62d90),
    (0x139bc4, 0x06012c00),
    (0x139bc8, 0x0000165a),
];
// The final 128 bytes of the repeated source have a custom UI bank owner.
const LOADER_BANKS: [(u32, usize); 2] = [(0xb57334, 0x8000), (0xb5f41c, 0x84b4)];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub source_offset: u32,
    pub halfwords: Vec<u16>,
}

impl Bank {
    fn from_rom(rom: &[u8], (source_offset, length): (u32, usize)) -> Self {
        let at = source_offset as usize;
        let halfwords = rom[at..at + length]
            .chunks_exact(2)
            .map(|data| u16::from_le_bytes(data.try_into().unwrap()))
            .collect();
        Self {
            source_offset,
            halfwords,
        }
    }

    fn source_span(
        &self,
        (offset, length): (u32, usize),
        role: &str,
    ) -> Result<SourceSpan, String> {
        if self.source_offset != offset || self.halfwords.len() != length / 2 {
            return Err(invalid());
        }
        let bytes = self
            .halfwords
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect();
        Ok((offset, bytes, role.into()))
    }

    fn byte(&self, offset: u32) -> Option<u8> {
        let relative = offset.checked_sub(self.source_offset)? as usize;
        let word = self.halfwords.get(relative / 2)?;
        Some(word.to_le_bytes()[relative % 2])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreloadCopies {
    pub caller_literals: [u32; 3],
    pub banks: [Bank; 3],
}

impl PreloadCopies {
    fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = Vec::with_capacity(PRELOAD_LITERALS.len() + PRELOAD_BANKS.len());
        for (&word, shape) in self.caller_literals.iter().zip(PRELOAD_LITERALS) {
            spans.push(literal_span(word, shape, "preload-copy-literal")?);
        }
        for (bank, shape) in self.banks.iter().zip(PRELOAD_BANKS) {
            spans.push(bank.source_span(shape, "preload-copy-source")?);
        }
        Ok(spans)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VramLoaderCopies {
    pub caller_literals: [u32; 11],
    pub banks: [Bank; 2],
}

impl VramLoaderCopies {
    fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = Vec::with_capacity(LOADER_LITERALS.len() + LOADER_BANKS.len());
        for (&word, shape) in self.caller_literals.iter().zip(LOADER_LITERALS) {
            spans.push(literal_span(word, shape, "vram-loader-literal")?);
        }
        for (bank, shape) in self.banks.iter().zip(LOADER_BANKS) {
            spans.push(bank.source_span(shape, "vram-loader-source")?);
        }
        Ok(spans)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HalfwordCopies {
    pub caller_literals: [u32; 11],
    pub selectors: [[u32; 5]; 2],
    pub banks: Vec<Bank>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preloads: Option<PreloadCopies>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vram_loaders: Option<VramLoaderCopies>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_pairs: Option<RowPairCopies>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_initializers: Option<StateInitializers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_register_tables: Option<ScrollRegisterTables>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_mode_pairs: Option<ColorModePairs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scanline_blend: Option<ScanlineBlendCopies>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilemap_rectangles: Option<TilemapRectangles>,
}

pub(crate) enum ByteSource {
    Literal,
    Selector(usize),
    Bank(usize),
    Payload,
}

fn invalid() -> String {
    "Invalid European halfword copy source banks".into()
}

fn literal_span(
    word: u32,
    (offset, expected): (u32, u32),
    role: &str,
) -> Result<SourceSpan, String> {
    if word != expected {
        return Err(invalid());
    }
    Ok((offset, word.to_le_bytes().to_vec(), role.into()))
}

fn literal_byte(words: &[u32], literals: &[(u32, u32)], offset: u32) -> Option<u8> {
    words.iter().zip(literals).find_map(|(word, &(start, _))| {
        let relative = offset.checked_sub(start).filter(|&n| n < 4)? as usize;
        Some(word.to_le_bytes()[relative])
    })
}

fn source_digest(spans: Vec<SourceSpan>) -> String {
    let source: Vec<_> = spans.into_iter().flat_map(|(_, bytes, _)| bytes).collect();
    ffta_rom::digest(&source)
}

fn rom_word(rom: &[u8], offset: u32) -> u32 {
    let at = offset as usize;
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}

impl HalfwordCopies {
    fn biased_source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.banks.len() != BANKS.len() {
            return Err(invalid());
        }
        let mut spans = Vec::with_capacity(BANKS.len() + SELECTORS.len() + LITERALS.len());
        for (&word, shape) in self.caller_literals.iter().zip(LITERALS) {
            spans.push(literal_span(word, shape, "halfword-copy-literal")?);
        }
        for (words, (offset, expected)) in self.selectors.iter().zip(SELECTORS) {
            if *words != expected {
                return Err(invalid());
            }
            let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            spans.push((offset, bytes, "halfword-copy-selector".into()));
        }
        for (bank, shape) in self.banks.iter().zip(BANKS) {
            spans.push(bank.source_span(shape, "halfword-copy-source")?);
        }
        Ok(spans)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let mut spans = self.biased_source_spans()?;
        if let Some(preloads) = &self.preloads {
            spans.extend(preloads.source_spans()?);
        }
        if let Some(loaders) = &self.vram_loaders {
            if self.preloads.is_none() {
                return Err(invalid());
            }
            spans.extend(loaders.source_spans()?);
        }
        if let Some(row_pairs) = &self.row_pairs {
            if self.vram_loaders.is_none() {
                return Err(invalid());
            }
            spans.extend(row_pairs.source_spans()?);
        }
        if let Some(initializers) = &self.state_initializers {
            if self.row_pairs.is_none() {
                return Err(invalid());
            }
            spans.extend(initializers.source_spans()?);
        }
        if let Some(tables) = &self.scroll_register_tables {
            if self.state_initializers.is_none() {
                return Err(invalid());
            }
            spans.extend(tables.source_spans()?);
        }
        if let Some(pairs) = &self.color_mode_pairs {
            if self.scroll_register_tables.is_none() {
                return Err(invalid());
            }
            spans.extend(pairs.source_spans()?);
        }
        if let Some(blend) = &self.scanline_blend {
            if self.color_mode_pairs.is_none() {
                return Err(invalid());
            }
            spans.extend(blend.source_spans()?);
        }
        if let Some(rectangles) = &self.tilemap_rectangles {
            if self.scanline_blend.is_none() {
                return Err(invalid());
            }
            spans.extend(rectangles.source_spans()?);
        }
        Ok(spans)
    }

    fn all_banks(&self) -> impl Iterator<Item = &Bank> {
        self.banks
            .iter()
            .chain(self.preloads.iter().flat_map(|copies| &copies.banks))
            .chain(self.vram_loaders.iter().flat_map(|copies| &copies.banks))
            .chain(self.row_pairs.iter().flat_map(|copies| &copies.banks))
            .chain(self.color_mode_pairs.iter().flat_map(|copies| &copies.banks))
            .chain(self.scanline_blend.iter().map(|copies| &copies.bank))
            .chain(
                self.tilemap_rectangles
                    .iter()
                    .flat_map(|maps| maps.rectangles.iter().map(|rectangle| &rectangle.bank)),
            )
    }

    pub(crate) fn byte(&self, offset: u32) -> Option<(u8, ByteSource)> {
        if let Some(value) = literal_byte(&self.caller_literals, &LITERALS, offset) {
            return Some((value, ByteSource::Literal));
        }
        if let Some(preloads) = &self.preloads {
            if let Some(value) = literal_byte(&preloads.caller_literals, &PRELOAD_LITERALS, offset)
            {
                return Some((value, ByteSource::Literal));
            }
        }
        if let Some(value) = self
            .vram_loaders
            .as_ref()
            .and_then(|loaders| literal_byte(&loaders.caller_literals, &LOADER_LITERALS, offset))
        {
            return Some((value, ByteSource::Literal));
        }
        for (index, (words, (start, _))) in self.selectors.iter().zip(SELECTORS).enumerate() {
            if let Some(relative) = offset.checked_sub(start).filter(|&n| n < 20) {
                let relative = relative as usize;
                let value = words[relative / 4].to_le_bytes()[relative % 4];
                return Some((value, ByteSource::Selector(index)));
            }
        }
        if let Some(byte) = self
            .row_pairs
            .as_ref()
            .and_then(|copies| copies.metadata_byte(offset))
        {
            return Some(byte);
        }
        if let Some(byte) = self
            .state_initializers
            .as_ref()
            .and_then(|initializers| initializers.byte(offset))
        {
            return Some(byte);
        }
        if let Some(byte) = self
            .scroll_register_tables
            .as_ref()
            .and_then(|tables| tables.byte(offset))
        {
            return Some(byte);
        }
        if let Some(byte) = self
            .color_mode_pairs
            .as_ref()
            .and_then(|copies| copies.metadata_byte(offset))
        {
            return Some(byte);
        }
        if let Some(word) = self
            .scanline_blend
            .as_ref()
            .and_then(|blend| literal_byte(&blend.caller_literals, &scanline_blend::LITERALS, offset))
        {
            return Some((word, ByteSource::Literal));
        }
        if let Some(word) = self.tilemap_rectangles.as_ref().and_then(|maps| {
            literal_byte(&maps.caller_literals, &tilemap_rectangles::LITERALS, offset)
        }) {
            return Some((word, ByteSource::Literal));
        }
        self.all_banks().enumerate().find_map(|(index, bank)| {
            bank.byte(offset)
                .map(|value| (value, ByteSource::Bank(index)))
        })
    }

    pub(crate) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.banks.capacity() * std::mem::size_of::<Bank>()
            + self
                .all_banks()
                .map(|b| b.halfwords.capacity() * 2)
                .sum::<usize>()
            + self
                .state_initializers
                .iter()
                .flat_map(|initializers| &initializers.direct_banks)
                .map(|bank| bank.halfwords.capacity() * 2)
                .sum::<usize>()
    }
}

/// The parent catalog has already authenticated the complete EUR ROM.
pub(super) fn from_rom(rom: &[u8], layout: CopyLayout) -> Result<HalfwordCopies, String> {
    let word = |offset| rom_word(rom, offset);
    let caller_literals = LITERALS.map(|(offset, _)| word(offset));
    let selectors =
        SELECTORS.map(|(offset, _)| std::array::from_fn(|i| word(offset + 4 * i as u32)));
    let mut value = HalfwordCopies {
        caller_literals,
        selectors,
        banks: BANKS
            .into_iter()
            .map(|shape| Bank::from_rom(rom, shape))
            .collect(),
        preloads: None,
        vram_loaders: None,
        row_pairs: None,
        state_initializers: None,
        scroll_register_tables: None,
        color_mode_pairs: None,
        scanline_blend: None,
        tilemap_rectangles: None,
    };
    if source_digest(value.biased_source_spans()?) != SOURCE_SHA256 {
        return Err("Halfword copy bank source fingerprint mismatch".into());
    }
    if layout >= CopyLayout::Preloads {
        let preloads = PreloadCopies {
            caller_literals: PRELOAD_LITERALS.map(|(offset, _)| word(offset)),
            banks: PRELOAD_BANKS.map(|shape| Bank::from_rom(rom, shape)),
        };
        if source_digest(preloads.source_spans()?) != PRELOAD_SHA256 {
            return Err("Halfword preload source fingerprint mismatch".into());
        }
        value.preloads = Some(preloads);
    }
    if layout >= CopyLayout::VramLoaders {
        let loaders = VramLoaderCopies {
            caller_literals: LOADER_LITERALS.map(|(offset, _)| word(offset)),
            banks: LOADER_BANKS.map(|shape| Bank::from_rom(rom, shape)),
        };
        if source_digest(loaders.source_spans()?) != LOADER_SHA256 {
            return Err("VRAM loader source fingerprint mismatch".into());
        }
        value.vram_loaders = Some(loaders);
    }
    if layout >= CopyLayout::RowPairs {
        value.row_pairs = Some(RowPairCopies::from_rom(rom)?);
    }
    if layout >= CopyLayout::StateInitializers {
        value.state_initializers = Some(StateInitializers::from_rom(rom)?);
    }
    if layout >= CopyLayout::ScrollRegisters {
        value.scroll_register_tables = Some(ScrollRegisterTables::from_rom(rom)?);
    }
    if layout >= CopyLayout::ColorModePairs {
        value.color_mode_pairs = Some(ColorModePairs::from_rom(rom)?);
    }
    if layout >= CopyLayout::ScanlineBlend {
        value.scanline_blend = Some(ScanlineBlendCopies::from_rom(rom)?);
    }
    if layout >= CopyLayout::TilemapRectangles {
        value.tilemap_rectangles = Some(TilemapRectangles::from_rom(rom)?);
    }
    Ok(value)
}

pub(crate) fn source_page(index: usize) -> bool {
    let page = index >> 12;
    let contains = |offset: u32, len: usize| {
        let at = offset as usize;
        (at >> 12..=(at + len - 1) >> 12).contains(&page)
    };
    LITERALS
        .iter()
        .chain(&PRELOAD_LITERALS)
        .chain(&LOADER_LITERALS)
        .chain(&row_pairs::LITERALS)
        .chain(&state_initializers::LITERALS)
        .chain(&color_mode_pairs::LITERALS)
        .chain(&scanline_blend::LITERALS)
        .chain(&tilemap_rectangles::LITERALS)
        .any(|&(offset, _)| contains(offset, 4))
        || SELECTORS.iter().any(|&(offset, _)| contains(offset, 20))
        || contains(row_pairs::SELECTOR_OFFSET, row_pairs::SELECTOR_BYTES)
        || color_mode_pairs::TABLES
            .iter()
            .any(|&offset| contains(offset, color_mode_pairs::TABLE_BYTES))
        || BANKS
            .iter()
            .chain(&PRELOAD_BANKS)
            .chain(&LOADER_BANKS)
            .chain(&row_pairs::BANKS)
            .chain(&color_mode_pairs::BANKS)
            .any(|&(offset, length)| contains(offset, length))
        || contains(scanline_blend::BANK.0, scanline_blend::BANK.1)
        || tilemap_rectangles::SHAPES.iter().any(|&shape| {
            let (offset, length) = tilemap_rectangles::bank_range(shape);
            contains(offset, length)
        })
        || state_initializers::DIRECT_BANKS
            .iter()
            .any(|&(offset, length, _)| contains(offset, length))
        || contains(
            state_initializers::INDIRECT_START,
            state_initializers::INDIRECT_BYTES,
        )
        || scroll_register_tables::source_page(index)
}
