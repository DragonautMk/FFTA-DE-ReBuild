//! Immutable job references; original live slot selection and getters remain guest work.
use super::super::{slot_job_lookup as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotJobLookupStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub slots_used: u64,
}
pub(super) struct SlotJobLookupResources {
    data: model::SlotJobLookup,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    slots: AtomicU64,
}
impl SlotJobLookupResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x399
    }
    pub(super) fn new(data: model::SlotJobLookup, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Slot-job source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Slot-job typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            slots: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<u8> {
        let at = u32::try_from(at).ok()?;
        let value = self.data.byte_at(at)?;
        let slot = at - self.data.source_offset;
        self.slots.fetch_or(1 << slot, Ordering::Relaxed);
        Some(value)
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
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(
            fields.iter().filter(|v| v.is_some()).count() as u64,
            Ordering::Relaxed,
        );
        Some(std::array::from_fn(|i| {
            fields[i].unwrap_or_else(|| {
                prior
                    .as_ref()
                    .map_or_else(|| rom[index + i], |value| value[i])
            })
        }))
    }
    pub(super) fn stats(&self) -> SlotJobLookupStats {
        SlotJobLookupStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            slots_used: u64::from(self.slots.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.entries.capacity() * std::mem::size_of::<model::Entry>()
    }
}
