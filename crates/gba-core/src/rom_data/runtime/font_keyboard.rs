//! EUR complete packed font and name-keyboard typed source views.
use super::*;
use crate::rom_data::{fonts::FontBank, name_keyboard as keyboard, text_catalogs::ProgramUnit};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FontKeyboardStats {
    pub bus_reads: u64,
    pub host_row_width_reads: u64,
    pub glyph_bytes: u64,
    pub width_bytes: u64,
    pub keyboard_index_bytes: u64,
    pub keyboard_program_bytes: u64,
    pub keyboard_row_width_bytes: u64,
}

pub(super) struct FontKeyboardResources {
    font: FontBank,
    keyboard: keyboard::Keyboard,
    counts: [AtomicU64; 6],
    host_row_width_reads: AtomicU64,
}

impl FontKeyboardResources {
    pub(super) fn new(font: FontBank, keyboard: keyboard::Keyboard) -> Result<Self, String> {
        font.validate()?;
        keyboard.validate()?;
        Ok(Self {
            font,
            keyboard,
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            host_row_width_reads: AtomicU64::new(0),
        })
    }

    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x3ad | 0x495..=0x4f9)
    }

    pub(super) fn row_widths(&self, page: usize) -> Option<&[u8]> {
        let widths = self.keyboard.row_widths.get(page)?;
        self.host_row_width_reads.fetch_add(1, Ordering::Relaxed);
        Some(widths)
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let glyph = index.checked_sub(text_short::GLYPH_ROOT as usize);
        if let Some(at) = glyph.filter(|&at| at < super::super::fonts::BANK_BYTES) {
            return Some((
                self.font.glyphs[at / 128].tile_words[at % 128 / 4].to_le_bytes()[at % 4],
                1,
            ));
        }
        if let Some(at) = index
            .checked_sub(text_short::WIDTH_ROOT as usize)
            .filter(|&at| at < self.font.glyphs.len())
        {
            return Some((self.font.glyphs[at].width, 2));
        }
        if let Some(at) = index
            .checked_sub(keyboard::ROW_WIDTH_ROOT as usize)
            .filter(|&at| at < 18)
        {
            return Some((self.keyboard.row_widths[at / 6][at % 6], 5));
        }
        if let Some(at) = index
            .checked_sub(keyboard::ROOT as usize)
            .filter(|&at| at < 8)
        {
            return Some((self.keyboard.offsets[at / 2].to_le_bytes()[at % 2], 3));
        }
        for p in &self.keyboard.programs {
            let Some(at) = index.checked_sub(p.source_offset as usize) else {
                continue;
            };
            if at < 2 {
                return Some((p.header.to_le_bytes()[at], 4));
            }
            let at = at - 2;
            // This keyboard contract admits only two-byte glyphs/commands and
            // the one-byte terminator, all verified against its raw programs.
            if at == p.units.len() * 2 {
                return Some((0, 4));
            }
            if let Some(unit) = p.units.get(at / 2) {
                let code = match unit {
                    ProgramUnit::Glyph { index } => *index | 0x8000,
                    ProgramUnit::Command { opcode, .. } => 0x4000 | u16::from(*opcode),
                };
                return Some((code.to_be_bytes()[at % 2], 4));
            }
        }
        None
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index) {
            return prior;
        }
        let mut counts = [0; 6];
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, kind)) => {
                counts[kind] += 1;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if counts[1..].iter().all(|&v| v == 0) {
            return prior;
        }
        counts[0] = 1;
        for (counter, n) in self.counts.iter().zip(counts) {
            counter.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }

    pub(super) fn stats(&self) -> FontKeyboardStats {
        let [bus_reads, glyph_bytes, width_bytes, keyboard_index_bytes, keyboard_program_bytes, keyboard_row_width_bytes] =
            self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        FontKeyboardStats {
            bus_reads,
            host_row_width_reads: self.host_row_width_reads.load(Ordering::Relaxed),
            glyph_bytes,
            width_bytes,
            keyboard_index_bytes,
            keyboard_program_bytes,
            keyboard_row_width_bytes,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.font.glyphs.capacity() * std::mem::size_of::<text_short::Glyph>()
            + self
                .font
                .glyphs
                .iter()
                .map(|g| g.tile_words.capacity() * 4)
                .sum::<usize>()
            + self.keyboard.offsets.capacity() * 2
            + self.keyboard.row_widths.capacity() * std::mem::size_of::<Vec<u8>>()
            + self
                .keyboard
                .row_widths
                .iter()
                .map(Vec::capacity)
                .sum::<usize>()
            + self.keyboard.cells.capacity() * std::mem::size_of::<Vec<u16>>()
            + self
                .keyboard
                .cells
                .iter()
                .map(|v| v.capacity() * 2)
                .sum::<usize>()
            + self.keyboard.programs.capacity()
                * std::mem::size_of::<super::super::text_catalogs::Program>()
            + self
                .keyboard
                .programs
                .iter()
                .map(|p| p.units.capacity() * std::mem::size_of::<ProgramUnit>())
                .sum::<usize>()
    }
}
