//! Typed selection fields; original reset/count/ANY/flag/store instructions remain.
use super::*;
use crate::rom_data::progression_selection_gates as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProgressionSelectionGatesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub minimum01_bytes: u64,
    pub minimum02_bytes: u64,
    pub minimum04_bytes: u64,
    pub any_minima_bytes: u64,
    pub flag_bytes: u64,
    pub record_mask: u64,
}
pub(super) struct ProgressionSelectionGatesResources {
    data: model::ProgressionSelectionGates,
    counters: [AtomicU64; 8],
    record_mask: AtomicU64,
}
impl ProgressionSelectionGatesResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e8
    }
    pub(super) fn new(
        data: model::ProgressionSelectionGates,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1 {
            return Err("Progression gates require one source binding".into());
        }
        let source = &sources[0];
        let bytes = data.source_bytes();
        if source.offset != model::SOURCE_OFFSET
            || source.length as usize != bytes.len()
            || source.role != "progression-selection-gates/eur-records"
            || source.sha256 != ffta_rom::digest(&bytes)
        {
            return Err("Progression gate source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            record_mask: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= model::END_OFFSET as usize
            || index.saturating_add(N) <= model::SOURCE_OFFSET as usize
        {
            return prior;
        }
        let mut counts = [0u64; 8];
        let mut mask = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::SOURCE_OFFSET as usize;
                let row = offset / model::RECORD_BYTES;
                let field = offset % model::RECORD_BYTES;
                counts[1] += 1;
                counts[match field {
                    0 => 2,
                    1 => 3,
                    2 | 3 => 4,
                    4 | 5 => 5,
                    6..=13 => 6,
                    _ => 7,
                }] += 1;
                mask |= 1u64 << row;
                self.data.records[row].byte(field).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.record_mask.fetch_or(mask, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ProgressionSelectionGatesStats {
        let [bus_reads, source_bytes, selector_bytes, minimum01_bytes, minimum02_bytes, minimum04_bytes, any_minima_bytes, flag_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        ProgressionSelectionGatesStats {
            bus_reads,
            source_bytes,
            selector_bytes,
            minimum01_bytes,
            minimum02_bytes,
            minimum04_bytes,
            any_minima_bytes,
            flag_bytes,
            record_mask: self.record_mask.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
