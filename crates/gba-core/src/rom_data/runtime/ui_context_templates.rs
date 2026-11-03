//! Typed template bytes; the original selector, copies and live UI remain guest code.
use super::*;
use crate::rom_data::ui_context_templates as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiContextTemplateStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub coordinate_bytes: u64,
    pub count_bytes: u64,
    pub flag_bytes: u64,
    pub opaque_bytes: u64,
}

pub(super) struct UiContextTemplateResources {
    data: model::UiContextTemplates,
    counters: [AtomicU64; 6],
}

impl UiContextTemplateResources {
    pub(super) fn source_page(index: usize) -> bool { index >> 12 == 0x394 }

    pub(super) fn new(data: model::UiContextTemplates, sources: &[SourceRange]) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len()
            || sources[0].role != "ui-context-templates"
            || sources[0].sha256 != ffta_rom::digest(&bytes)
        {
            return Err("UI context template source binding mismatch".into());
        }
        Ok(Self { data, counters: std::array::from_fn(|_| AtomicU64::new(0)) })
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        if index >= model::END_OFFSET as usize || index.saturating_add(N) <= model::SOURCE_OFFSET as usize {
            return prior;
        }
        let mut counts = [0u64; 6];
        counts[0] = 1;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(byte) = self.data.byte_at(at as u32) {
                counts[1] += 1;
                let field = match (at - model::SOURCE_OFFSET as usize) % model::RECORD_BYTES {
                    0..=1 => 2,
                    4..=5 => 3,
                    9 => 4,
                    _ => 5,
                };
                counts[field] += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[at], |value| value[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }

    pub(super) fn stats(&self) -> UiContextTemplateStats {
        let values = self.counters.each_ref().map(|value| value.load(Ordering::Relaxed));
        UiContextTemplateStats {
            bus_reads: values[0], source_bytes: values[1], coordinate_bytes: values[2],
            count_bytes: values[3], flag_bytes: values[4], opaque_bytes: values[5],
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
