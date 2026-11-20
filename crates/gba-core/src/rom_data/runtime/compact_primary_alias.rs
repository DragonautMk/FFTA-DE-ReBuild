//! Serve the two conditional primary-task source spans without limiting guest IDs.
use super::super::{compact_primary_alias as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompactPrimaryAliasStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub position_bytes: u64,
}

pub(super) struct CompactPrimaryAliasResources {
    data: model::CompactPrimaryAlias,
    reads: AtomicU64,
    fields: [AtomicU64; 2],
}

impl CompactPrimaryAliasResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x3ab
    }

    pub(super) fn new(
        data: model::CompactPrimaryAlias,
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
            return Err("Compact primary alias source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        if index == self.data.lookup_offset as usize {
            return Some((self.data.selector, 0));
        }
        let n = index.checked_sub(self.data.position_offset as usize)?;
        match n {
            0..=1 => Some((self.data.x.to_le_bytes()[n], 1)),
            2..=3 => Some((self.data.y.to_le_bytes()[n - 2], 1)),
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

    pub(super) fn stats(&self) -> CompactPrimaryAliasStats {
        let [selector_bytes, position_bytes] =
            self.fields.each_ref().map(|c| c.load(Ordering::Relaxed));
        CompactPrimaryAliasStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: selector_bytes + position_bytes,
            selector_bytes,
            position_bytes,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
