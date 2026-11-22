//! Serve original copied halfword lookup bytes and their second-table roots.
use super::*;
use crate::rom_data::copied_halfword_lookups as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopiedHalfwordLookupsStats {
    pub bus_reads: u64,
    pub table_bytes: u64,
    pub root_bytes: u64,
    pub table_mask: u8,
    pub root_mask: u8,
    pub soil_source_mask: u8,
}

pub(super) struct CopiedHalfwordLookupsResources {
    data: model::CopiedHalfwordLookups,
    reads: AtomicU64,
    table_bytes: AtomicU64,
    root_bytes: AtomicU64,
    tables: AtomicU64,
    roots: AtomicU64,
    soil_source: AtomicU64,
}
impl CopiedHalfwordLookupsResources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        (first..=last).any(|page| matches!(page, 0x3 | 0x14c | 0x14d))
    }

    pub(super) fn new(
        data: model::CopiedHalfwordLookups,
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
            return Err("Copied halfword lookup source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            table_bytes: AtomicU64::new(0),
            root_bytes: AtomicU64::new(0),
            tables: AtomicU64::new(0),
            roots: AtomicU64::new(0),
            soil_source: AtomicU64::new(0),
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
        let (mut table_bytes, mut root_bytes, mut tables, mut roots, mut soil_source) = (0u64, 0u64, 0u64, 0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry, root)) => {
                if root {
                    root_bytes += 1;
                    roots |= 1 << entry;
                } else {
                    table_bytes += 1;
                    tables |= 1 << entry;
                    if entry == 3 { soil_source |= 1 << (index + i - model::SOIL_EVIDENCE_OFFSET as usize); }
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
        self.tables.fetch_or(tables, Ordering::Relaxed);
        self.roots.fetch_or(roots, Ordering::Relaxed);
        self.soil_source.fetch_or(soil_source, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> CopiedHalfwordLookupsStats {
        CopiedHalfwordLookupsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            table_bytes: self.table_bytes.load(Ordering::Relaxed),
            root_bytes: self.root_bytes.load(Ordering::Relaxed),
            table_mask: self.tables.load(Ordering::Relaxed) as u8,
            root_mask: self.roots.load(Ordering::Relaxed) as u8,
            soil_source_mask: self.soil_source.load(Ordering::Relaxed) as u8,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
