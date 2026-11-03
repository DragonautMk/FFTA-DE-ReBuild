//! Immutable world-record storage, composed with prior owners and raw borders.
use super::super::{world_records38 as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldRecords38Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub byte01_bytes: u64,
    pub value02_bytes: u64,
    pub value04_bytes: u64,
    pub threshold_bytes: u64,
    pub opaque_bytes: u64,
    pub records_used: u64,
}
pub(super) struct WorldRecords38Resources {
    data: model::WorldRecords38,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 5],
    records: AtomicU64,
}
impl WorldRecords38Resources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x71e
    }
    pub(super) fn new(
        data: model::WorldRecords38,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("WorldRecords38 binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("WorldRecords38 typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
            records: AtomicU64::new(0),
        })
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
        let values: [_; N] = std::array::from_fn(|i| {
            u32::try_from(index + i)
                .ok()
                .and_then(|at| self.data.byte_at(at))
        });
        if values.iter().all(Option::is_none) {
            return prior;
        }
        let mut fields = [0u64; 5];
        let mut records = 0;
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let at = index + i - model::SOURCE as usize;
                records |= 1 << (at / model::RECORD_BYTES);
                fields[field_class(at % model::RECORD_BYTES)] += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(fields.iter().sum(), Ordering::Relaxed);
        for (counter, count) in self.fields.iter().zip(fields) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.records.fetch_or(records, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> WorldRecords38Stats {
        let [byte01_bytes, value02_bytes, value04_bytes, threshold_bytes, opaque_bytes] =
            self.fields.each_ref().map(|v| v.load(Ordering::Relaxed));
        WorldRecords38Stats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            byte01_bytes,
            value02_bytes,
            value04_bytes,
            threshold_bytes,
            opaque_bytes,
            records_used: u64::from(self.records.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
fn field_class(offset: usize) -> usize {
    match offset {
        1 => 0,
        2..=3 => 1,
        4..=5 => 2,
        7..=37 => 3,
        0 | 6 => 4,
        _ => unreachable!("outside fixed record storage"),
    }
}
