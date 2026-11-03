//! File-derived BGR555 words and slice selectors; all live palette work stays guest work.
use super::super::{color_mode_palettes as model, metadata14, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

pub(in crate::rom_data) fn validate_dependencies(
    data: &model::ColorModePalettes,
    special: &metadata14::Metadata14,
) -> Result<(), String> {
    data.validate()?;
    special.validate()?;
    // These producer records selected the owned special source projection.
    // This check does not replace the guest's selector or cap live indices.
    for record in &special.records {
        for argument in [0, 1] {
            if let Some(index) = record.palette(argument).source_index {
                for mode in [0, 1, 2] {
                    let at = model::Book::SpecialUnit.source(mode, u32::from(index));
                    if (at..at + 32).any(|a| data.byte_at(a).is_none()) {
                        return Err("Color-Mode palette producer has no owned source".into());
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorModePaletteStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub unit_base_bytes: u64,
    pub companion_bytes: u64,
    pub supplemental_bytes: u64,
    pub shared_color_bytes: u64,
    pub special_unit_bytes: u64,
    pub selector_bytes: u64,
    pub fragments_used: u64,
}
pub(super) struct ColorModePaletteResources {
    data: model::ColorModePalettes,
    counters: [AtomicU64; 8],
    fragment_mask: AtomicU64,
}
impl ColorModePaletteResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x394 | 0x40f..=0x411)
    }
    pub(super) fn new(
        data: model::ColorModePalettes,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Color-Mode palette source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Color-Mode palette typed source binding mismatch".into());
            }
        }
        // Reconstructed bytes above are temporary authentication inputs. Only
        // typed colors/selectors are retained, never a raw ROM source shadow.
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            fragment_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(index).ok()?;
        let value = self.data.byte_at(at)?;
        let (fragment, category) =
            if (model::SLICE_OFFSETS_SOURCE..model::SLICE_OFFSETS_SOURCE + 4).contains(&at) {
                (29, 7)
            } else {
                let n = self.data.spans.partition_point(|s| s.source_offset <= at) - 1;
                let category = match self.data.spans[n].book {
                    model::Book::UnitBase => 2,
                    model::Book::CompanionBase => 3,
                    model::Book::Supplemental => 4,
                    model::Book::SharedColors => 5,
                    model::Book::SpecialUnit => 6,
                };
                (n, category)
            };
        self.fragment_mask
            .fetch_or(1 << fragment, Ordering::Relaxed);
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
        let mut counts = [1, 0, 0, 0, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((value, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                value
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |b| b[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ColorModePaletteStats {
        let [bus_reads, source_bytes, unit_base_bytes, companion_bytes, supplemental_bytes, shared_color_bytes, special_unit_bytes, selector_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        ColorModePaletteStats {
            bus_reads,
            source_bytes,
            unit_base_bytes,
            companion_bytes,
            supplemental_bytes,
            shared_color_bytes,
            special_unit_bytes,
            selector_bytes,
            fragments_used: u64::from(self.fragment_mask.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.spans.capacity() * std::mem::size_of::<model::ColorSpan>()
            + self
                .data
                .spans
                .iter()
                .map(|s| s.id.capacity() + s.colors.capacity() * 2)
                .sum::<usize>()
    }
}
