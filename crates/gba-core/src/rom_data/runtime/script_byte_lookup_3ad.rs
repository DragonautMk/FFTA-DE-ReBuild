//! Serve a script lookup window and its second root with mixed-owner reads.
use super::*;
use crate::rom_data::script_byte_lookup_3ad as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptByteLookup3adStats {
    pub bus_reads: u64,
    pub table_bytes: u64,
    pub root_bytes: u64,
    pub values_mask: u8,
}

pub(super) struct ScriptByteLookup3adResources {
    data: model::ScriptByteLookup3ad,
    reads: AtomicU64,
    table_bytes: AtomicU64,
    root_bytes: AtomicU64,
    values: AtomicU64,
}

impl ScriptByteLookup3adResources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        (first..=last).any(|page| matches!(page, 0x126 | 0x3ad))
    }

    pub(super) fn new(
        data: model::ScriptByteLookup3ad,
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
            return Err("Script byte lookup source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            table_bytes: AtomicU64::new(0),
            root_bytes: AtomicU64::new(0),
            values: AtomicU64::new(0),
        })
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let (mut table_bytes, mut root_bytes, mut values) = (0u64, 0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry, root)) => {
                if root {
                    root_bytes += 1;
                } else {
                    table_bytes += 1;
                    values |= 1 << entry;
                }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.table_bytes.fetch_add(table_bytes, Ordering::Relaxed);
        self.root_bytes.fetch_add(root_bytes, Ordering::Relaxed);
        self.values.fetch_or(values, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> ScriptByteLookup3adStats {
        ScriptByteLookup3adStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            table_bytes: self.table_bytes.load(Ordering::Relaxed),
            root_bytes: self.root_bytes.load(Ordering::Relaxed),
            values_mask: self.values.load(Ordering::Relaxed) as u8,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
