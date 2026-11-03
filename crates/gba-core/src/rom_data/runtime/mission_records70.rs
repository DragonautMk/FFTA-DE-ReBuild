//! Immutable mission records; selector decoding and event logic remain guest code.
use super::super::{mission_records70 as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MissionRecords70Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub word_bytes: u64,
    pub packed_bytes: u64,
    pub operand_bytes: u64,
    pub inline_bytes: u64,
    pub byte_list_bytes: u64,
    pub scaled_bytes: u64,
    pub scalar_bytes: u64,
    pub mapped_bytes: u64,
    pub flag_bytes: u64,
    pub event_bytes: u64,
    pub records_used: u64,
}
pub(super) struct MissionRecords70Resources {
    data: model::Records70,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 10],
    records: [AtomicU64; 8],
}
impl MissionRecords70Resources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x713..=0x71c).contains(&(index >> 12))
    }
    pub(super) fn new(data: model::Records70, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("MissionRecords70 binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("MissionRecords70 typed source binding mismatch".into());
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
        let mut fields = [0u64; 10];
        let mut records = [0u64; 8];
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
    pub(super) fn stats(&self) -> MissionRecords70Stats {
        let [word_bytes, packed_bytes, operand_bytes, inline_bytes, byte_list_bytes, scaled_bytes, scalar_bytes, mapped_bytes, flag_bytes, event_bytes] =
            self.fields.each_ref().map(|v| v.load(Ordering::Relaxed));
        MissionRecords70Stats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            word_bytes,
            packed_bytes,
            operand_bytes,
            inline_bytes,
            byte_list_bytes,
            scaled_bytes,
            scalar_bytes,
            mapped_bytes,
            flag_bytes,
            event_bytes,
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
        0..=1 | 5..=6 | 8..=9 | 11..=12 | 34..=41 | 67..=68 => 0,
        2..=4 | 14..=17 | 56..=60 => 1,
        7 | 10 | 13 => 2,
        18..=33 => 3,
        42..=50 => 4,
        51 | 62 => 5,
        52..=53 | 63..=64 => 6,
        54..=55 | 61 => 7,
        65..=66 => 8,
        69 => 9,
        _ => unreachable!("record byte outside its fixed storage"),
    }
}
