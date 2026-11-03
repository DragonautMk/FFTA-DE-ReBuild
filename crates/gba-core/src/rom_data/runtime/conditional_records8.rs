//! Immutable eight-byte slots; live predicate evaluation remain guest code.
use super::super::{conditional_records8 as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConditionalRecords8Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub gate_bytes: u64,
    pub filter_bytes: u64,
    pub first_key_bytes: u64,
    pub first_operand_bytes: u64,
    pub second_key_bytes: u64,
    pub second_operand_bytes: u64,
    pub records_used: u64,
}
pub(super) struct ConditionalRecords8Resources {
    data: model::Records8,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 6],
    records: [AtomicU64; 2],
}
impl ConditionalRecords8Resources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x713
    }
    pub(super) fn new(data: model::Records8, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("ConditionalRecords8 binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("ConditionalRecords8 typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
            records: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut fields = [0u64; 6];
        let mut records = [0u64; 2];
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let at = index + i - model::SOURCE as usize;
                let record = at / model::RECORD_BYTES;
                records[record / 64] |= 1 << (record % 64);
                fields[[0, 1, 2, 2, 3, 4, 4, 5][at % model::RECORD_BYTES]] += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(fields.iter().sum(), Ordering::Relaxed);
        for (counter, count) in self.fields.iter().zip(fields) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, mask) in self.records.iter().zip(records) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ConditionalRecords8Stats {
        let [gate_bytes, filter_bytes, first_key_bytes, first_operand_bytes, second_key_bytes, second_operand_bytes] =
            self.fields.each_ref().map(|v| v.load(Ordering::Relaxed));
        ConditionalRecords8Stats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            gate_bytes,
            filter_bytes,
            first_key_bytes,
            first_operand_bytes,
            second_key_bytes,
            second_operand_bytes,
            records_used: self
                .records
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
