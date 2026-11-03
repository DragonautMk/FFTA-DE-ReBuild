//! File-derived raster palette fields. Original windowing, copies, wrap and DMA stay live.
use super::super::{map_raster_palettes as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapRasterPaletteStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub index_bytes: u64,
    pub header_bytes: u64,
    pub color_bytes: u64,
    pub programs_used: u64,
}
pub(super) struct MapRasterPaletteResources {
    data: model::MapRasterPalettes,
    counters: [AtomicU64; 5],
    program_mask: AtomicU64,
}
impl MapRasterPaletteResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x487..=0x489)
    }
    pub(super) fn new(
        data: model::MapRasterPalettes,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Raster palette source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Raster palette typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            program_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        let value = self.data.byte_at(at)?;
        let relative = (at - self.data.source_offset) as usize;
        let (program, category) = if relative < model::INDEX_BYTES {
            (relative / 2, 2)
        } else {
            let n = relative - model::INDEX_BYTES;
            (
                n / model::RECORD_BYTES,
                if n % model::RECORD_BYTES < 6 { 3 } else { 4 },
            )
        };
        self.program_mask.fetch_or(1 << program, Ordering::Relaxed);
        Some((value, category))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [1, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((value, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                value
            } else {
                prior
                    .as_ref()
                    .map_or_else(|| rom[index + i], |bytes| bytes[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> MapRasterPaletteStats {
        let [bus_reads, source_bytes, index_bytes, header_bytes, color_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        MapRasterPaletteStats {
            bus_reads,
            source_bytes,
            index_bytes,
            header_bytes,
            color_bytes,
            programs_used: u64::from(self.program_mask.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.programs.capacity() * std::mem::size_of::<model::RasterPalette>()
            + self.data.descriptor_programs.capacity() * std::mem::size_of::<[u16; 2]>()
            + self
                .data
                .programs
                .iter()
                .map(|p| p.colors.capacity() * 2)
                .sum::<usize>()
    }
}
