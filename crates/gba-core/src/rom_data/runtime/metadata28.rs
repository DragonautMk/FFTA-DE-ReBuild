//! Immutable record bytes. Original selectors, flag queries and live callers stay guest code.
use super::super::{metadata28 as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata28Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub halfword_bytes: u64,
    pub byte_bytes: u64,
    pub inline_bytes: u64,
    pub flag_bytes: u64,
    pub opaque_bytes: u64,
    pub records_used: u64,
}
pub(super) struct Metadata28Resources {
    data: model::Metadata28,
    reads: AtomicU64,
    bytes: AtomicU64,
    classes: [AtomicU64; 5],
    records: [AtomicU64; 6],
}
impl Metadata28Resources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x710..=0x713).contains(&(index >> 12))
    }
    pub(super) fn new(data: model::Metadata28, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("Metadata28 binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Metadata28 typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            classes: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut classes = [0u64; 5];
        let mut records = [0u64; 6];
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let at = index + i - model::SOURCE as usize;
                let record = at / model::RECORD_BYTES;
                records[record / 64] |= 1 << (record % 64);
                let class = match at % model::RECORD_BYTES {
                    0..=1 | 20..=23 => 0,
                    2 | 4..=11 | 24..=26 => 1,
                    12..=14 => 2,
                    16..=19 => 3,
                    _ => 4,
                };
                classes[class] += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes
            .fetch_add(classes.iter().sum(), Ordering::Relaxed);
        for (counter, count) in self.classes.iter().zip(classes) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, mask) in self.records.iter().zip(records) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> Metadata28Stats {
        let [halfword_bytes, byte_bytes, inline_bytes, flag_bytes, opaque_bytes] =
            self.classes.each_ref().map(|v| v.load(Ordering::Relaxed));
        Metadata28Stats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            halfword_bytes,
            byte_bytes,
            inline_bytes,
            flag_bytes,
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
