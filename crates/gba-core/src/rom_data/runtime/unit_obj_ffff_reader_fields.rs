//! Immutable numeric source words; following the encoded address stays guest work.
use super::super::{unit_obj_ffff_reader_fields as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitObjFfffReaderFieldsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub property_bytes: u64,
    pub directory_bytes: u64,
    pub fields_used: u64,
}
pub(super) struct UnitObjFfffReaderFieldsResources {
    data: model::UnitObjFfffReaderFields,
    counters: [AtomicU64; 4],
    fields: AtomicU64,
}
impl UnitObjFfffReaderFieldsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x3b4 | 0x3d4)
    }
    pub(super) fn new(
        data: model::UnitObjFfffReaderFields,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("FFFF reader field binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("FFFF reader field typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            fields: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        let value = self.data.byte_at(at)?;
        let field = usize::from(at >= self.data.directory_word.source_offset);
        self.fields.fetch_or(1 << field, Ordering::Relaxed);
        Some((value, field + 2))
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
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 4];
        counts[0] = 1;
        let bytes = std::array::from_fn(|i| match fields[i] {
            Some((value, field)) => {
                counts[1] += 1;
                counts[field] += 1;
                value
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> UnitObjFfffReaderFieldsStats {
        UnitObjFfffReaderFieldsStats {
            bus_reads: self.counters[0].load(Ordering::Relaxed),
            source_bytes: self.counters[1].load(Ordering::Relaxed),
            property_bytes: self.counters[2].load(Ordering::Relaxed),
            directory_bytes: self.counters[3].load(Ordering::Relaxed),
            fields_used: u64::from(self.fields.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.producers.capacity() * std::mem::size_of::<model::Producer>()
            + self
                .data
                .producers
                .iter()
                .map(|p| p.script_sources.capacity() * 4)
                .sum::<usize>()
    }
}
