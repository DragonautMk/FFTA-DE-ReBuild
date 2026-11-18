//! Typed delivery of a conditional text-reader callback registry.
use super::super::{text_reader_registry as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextReaderRegistryStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub literal_bytes: u64,
    pub callback_bytes: u64,
}

pub(super) struct TextReaderRegistryResources {
    data: model::TextReaderRegistry,
    reads: AtomicU64,
    fields: [AtomicU64; 2],
}
impl TextReaderRegistryResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x019 | 0x371)
    }
    pub(super) fn new(
        data: model::TextReaderRegistry,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len()
            || spans
                .iter()
                .zip(sources)
                .any(|((offset, bytes, role), source)| {
                    source.offset != *offset
                        || source.length as usize != bytes.len()
                        || source.role != *role
                        || source.sha256 != ffta_rom::digest(bytes)
                })
        {
            return Err("Text-reader registry source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        for at in [model::PRODUCER_LITERAL, model::LOOKUP_LITERAL] {
            if let Some(relative) = index.checked_sub(at as usize).filter(|&i| i < 4) {
                return Some((
                    (0x0800_0000 + self.data.table_offset).to_le_bytes()[relative],
                    0,
                ));
            }
        }
        let relative = index.checked_sub(self.data.table_offset as usize)?;
        let word = self.data.callbacks.get(relative / 4)?;
        Some((word.to_le_bytes()[relative % 4], 1))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 2];
        let values = std::array::from_fn(|i| match cells[i] {
            Some((value, kind)) => {
                counts[kind] += 1;
                value
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.fields.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(values)
    }
    pub(super) fn stats(&self) -> TextReaderRegistryStats {
        let [literal_bytes, callback_bytes] =
            self.fields.each_ref().map(|c| c.load(Ordering::Relaxed));
        TextReaderRegistryStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: literal_bytes + callback_bytes,
            literal_bytes,
            callback_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
