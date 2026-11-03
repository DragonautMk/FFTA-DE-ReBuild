//! Immutable file-derived choice fields; original live filters, copies and input remain guest work.
use super::super::{ui_choice_lists as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiChoiceListStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub identifier_bytes: u64,
    pub coordinate_bytes: u64,
    pub opaque_bytes: u64,
    pub navigation_bytes: u64,
    pub lists_used: u64,
}
pub(super) struct UiChoiceListResources {
    data: model::UiChoiceLists,
    counters: [AtomicU64; 6],
    list_mask: AtomicU64,
}
impl UiChoiceListResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x397 | 0x3ad)
    }
    pub(super) fn new(data: model::UiChoiceLists, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("UI choice source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("UI choice typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            list_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        let value = self.data.byte_at(at)?;
        let index = self.data.lists.partition_point(|p| p.source_offset <= at) - 1;
        let relative = (at - self.data.lists[index].source_offset) as usize;
        let category = match relative % model::ENTRY_BYTES {
            0 => 2,
            1 | 2 => 3,
            3 => 4,
            _ => 5,
        };
        self.list_mask.fetch_or(1 << index, Ordering::Relaxed);
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
        let mut counts = [1, 0, 0, 0, 0, 0];
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
    pub(super) fn stats(&self) -> UiChoiceListStats {
        let [bus_reads, source_bytes, identifier_bytes, coordinate_bytes, opaque_bytes, navigation_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        UiChoiceListStats {
            bus_reads,
            source_bytes,
            identifier_bytes,
            coordinate_bytes,
            opaque_bytes,
            navigation_bytes,
            lists_used: u64::from(self.list_mask.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.lists.capacity() * std::mem::size_of::<model::ChoiceList>()
            + self
                .data
                .lists
                .iter()
                .map(|p| {
                    p.id.capacity()
                        + p.entries.capacity() * std::mem::size_of::<model::ChoiceEntry>()
                })
                .sum::<usize>()
    }
}
