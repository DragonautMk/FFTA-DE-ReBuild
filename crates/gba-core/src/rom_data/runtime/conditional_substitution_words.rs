//! Typed delivery of conditional substitution-adjacent source words.
use super::super::{conditional_substitution_words as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConditionalSubstitutionWordsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub literal_bytes: u64,
    pub word_bytes: u64,
}
pub(super) struct ConditionalSubstitutionWordsResources {
    data: model::ConditionalSubstitutionWords,
    reads: AtomicU64,
    fields: [AtomicU64; 2],
}
impl ConditionalSubstitutionWordsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x019 | 0x371)
    }
    pub(super) fn new(
        data: model::ConditionalSubstitutionWords,
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
            return Err("Conditional substitution source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        if let Some(relative) = index
            .checked_sub(model::DISPATCH_LITERAL as usize)
            .filter(|&i| i < 4)
        {
            return Some((
                (0x0800_0000 + self.data.table_offset).to_le_bytes()[relative],
                0,
            ));
        }
        let relative = index.checked_sub(self.data.table_offset as usize)?;
        let word_index = u8::try_from(relative / 4).ok()?;
        let position = self
            .data
            .words
            .binary_search_by_key(&word_index, |w| w.index)
            .ok()?;
        Some((
            self.data.words[position].word.to_le_bytes()[relative % 4],
            1,
        ))
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
    pub(super) fn stats(&self) -> ConditionalSubstitutionWordsStats {
        let [literal_bytes, word_bytes] = self.fields.each_ref().map(|c| c.load(Ordering::Relaxed));
        ConditionalSubstitutionWordsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: literal_bytes + word_bytes,
            literal_bytes,
            word_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.words.capacity() * std::mem::size_of::<model::ConditionalWord>()
    }
}
