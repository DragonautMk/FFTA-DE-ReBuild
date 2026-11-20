//! Serve typed display-control constants while guest selection remains unchanged.
use super::super::{display_control_tables as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplayControlTablesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub halfword_bytes: u64,
    pub pointer_bytes: u64,
}

pub(super) struct DisplayControlTablesResources {
    data: model::DisplayControlTables,
    reads: AtomicU64,
    fields: [AtomicU64; 2],
}

impl DisplayControlTablesResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x14d
    }

    pub(super) fn new(
        data: model::DisplayControlTables,
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
            return Err("Display-control table source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let n = index.checked_sub(self.data.source_offset as usize)?;
        match n {
            0..=7 => Some((self.data.first_halfwords[n / 2].to_le_bytes()[n % 2], 0)),
            8..=15 => Some((
                self.data.second_halfwords[(n - 8) / 2].to_le_bytes()[(n - 8) % 2],
                0,
            )),
            16..=31 => Some((
                self.data.iwram_pointers[(n - 16) / 4].to_le_bytes()[(n - 16) % 4],
                1,
            )),
            _ => None,
        }
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

    pub(super) fn stats(&self) -> DisplayControlTablesStats {
        let [halfword_bytes, pointer_bytes] =
            self.fields.each_ref().map(|c| c.load(Ordering::Relaxed));
        DisplayControlTablesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: halfword_bytes + pointer_bytes,
            halfword_bytes,
            pointer_bytes,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
