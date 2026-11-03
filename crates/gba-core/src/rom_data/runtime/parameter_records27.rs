//! Immutable sparse parameter storage; unclassified indices retain prior/raw bytes.
use super::super::{parameter_records27 as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParameterRecords27Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub header_bytes: u64,
    pub packed_bytes: u64,
    pub value07_bytes: u64,
    pub byte09_bytes: u64,
    pub value0a_bytes: u64,
    pub signed0c_bytes: u64,
    pub flags0d_bytes: u64,
    pub operands0e_bytes: u64,
    pub operands10_bytes: u64,
    pub value12_bytes: u64,
    pub signed15_bytes: u64,
    pub opaque_bytes: u64,
    pub records_used: u64,
}
pub(super) struct ParameterRecords27Resources {
    data: model::ParameterRecords27,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 12],
    records: [AtomicU64; 4],
}
impl ParameterRecords27Resources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x71d..=0x71e).contains(&(index >> 12))
    }
    pub(super) fn new(
        data: model::ParameterRecords27,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("ParameterRecords27 binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("ParameterRecords27 typed source binding mismatch".into());
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
        let mut fields = [0u64; 12];
        let mut records = [0u64; 4];
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let at = index + i - model::SOURCE as usize;
                let record = at / model::RECORD_BYTES;
                records[record / 64] |= 1 << (record % 64);
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
        for (counter, mask) in self.records.iter().zip(records) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ParameterRecords27Stats {
        let [header_bytes, packed_bytes, value07_bytes, byte09_bytes, value0a_bytes, signed0c_bytes, flags0d_bytes, operands0e_bytes, operands10_bytes, value12_bytes, signed15_bytes, opaque_bytes] =
            self.fields.each_ref().map(|v| v.load(Ordering::Relaxed));
        ParameterRecords27Stats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            header_bytes,
            packed_bytes,
            value07_bytes,
            byte09_bytes,
            value0a_bytes,
            signed0c_bytes,
            flags0d_bytes,
            operands0e_bytes,
            operands10_bytes,
            value12_bytes,
            signed15_bytes,
            opaque_bytes,
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

// Disjoint physical storage classes; names do not infer game semantics.
fn field_class(offset: usize) -> usize {
    match offset {
        0 => 0,
        1..=6 => 1,
        7..=8 => 2,
        9 => 3,
        10..=11 => 4,
        12 => 5,
        13 => 6,
        14..=15 => 7,
        16..=17 => 8,
        18..=19 => 9,
        21 => 10,
        20 | 22..=26 => 11,
        _ => unreachable!("record byte outside its fixed storage"),
    }
}
