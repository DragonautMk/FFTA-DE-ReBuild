//! Immutable file-derived words; allocation, task reuse and live bus state stay guest work.
use super::super::{unit_obj_byte_property_words as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitObjBytePropertyWordsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub words_used: u64,
}
pub(super) struct UnitObjBytePropertyWordsResources {
    data: model::UnitObjBytePropertyWords,
    counters: [AtomicU64; 2],
    words: [AtomicU64; 2],
}
impl UnitObjBytePropertyWordsResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x394
    }
    pub(super) fn new(
        data: model::UnitObjBytePropertyWords,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("UI byte propertyword source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("UI byte propertyword typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            words: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, at: usize) -> Option<u8> {
        let at = u32::try_from(at).ok()?;
        let index = self
            .data
            .words
            .binary_search_by_key(&(at & !1), |w| w.source_offset)
            .ok()?;
        self.words[index / 64].fetch_or(1 << (index % 64), Ordering::Relaxed);
        Some(self.data.words[index].word.to_le_bytes()[(at & 1) as usize])
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
        let mut count = 0;
        let bytes = std::array::from_fn(|i| match fields[i] {
            Some(value) => {
                count += 1;
                value
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.counters[0].fetch_add(1, Ordering::Relaxed);
        self.counters[1].fetch_add(count, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> UnitObjBytePropertyWordsStats {
        UnitObjBytePropertyWordsStats {
            bus_reads: self.counters[0].load(Ordering::Relaxed),
            source_bytes: self.counters[1].load(Ordering::Relaxed),
            words_used: self
                .words
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.words.capacity() * std::mem::size_of::<model::Word>()
    }
}
