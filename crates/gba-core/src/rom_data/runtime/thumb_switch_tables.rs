//! Serve authenticated control-flow data while preserving original guest branches.
use super::*;
use crate::rom_data::thumb_switch_tables as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThumbSwitchTablesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub roots_mask: u64,
    pub destinations_mask: u64,
    pub additional_source_bytes: u64,
    pub additional_destinations_mask: u64,
}

pub(super) struct ThumbSwitchTablesResources {
    data: model::ThumbSwitchTables,
    reads: AtomicU64,
    bytes: AtomicU64,
    roots: AtomicU64,
    destinations: AtomicU64,
    additional_bytes: AtomicU64,
    additional_destinations: AtomicU64,
}

impl ThumbSwitchTablesResources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        model::TABLES.iter().any(|spec| first <= (spec.end() as usize - 1) >> 12
            && last >= spec.root as usize >> 12)
            || model::additional_table_specs().any(|spec| first <= (spec.end() as usize - 1) >> 12
                && last >= spec.start() as usize >> 12)
    }

    pub(super) fn new(data: model::ThumbSwitchTables, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != model::TABLE_COUNT + data.additional_tables.len() { return Err("Thumb switch source count mismatch".into()); }
        for ((table, spec), source) in data.tables.iter().zip(model::TABLES).zip(&sources[..model::TABLE_COUNT]) {
            let bytes = table.source_bytes();
            if source.offset != spec.root || source.length as usize != bytes.len()
                || source.role != model::SOURCE_ROLE || source.sha256 != ffta_rom::digest(&bytes) {
                return Err("Thumb switch source binding mismatch".into());
            }
        }
        for ((table, spec), source) in data.additional_tables.iter().zip(model::additional_table_specs())
            .zip(&sources[model::TABLE_COUNT..]) {
            let bytes = table.destination_bytes();
            if source.offset != spec.start() || source.length as usize != bytes.len()
                || source.role != model::ADDITIONAL_SOURCE_ROLE || source.sha256 != ffta_rom::digest(&bytes) {
                return Err("Additional Thumb switch source binding mismatch".into());
            }
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0),
            roots: AtomicU64::new(0), destinations: AtomicU64::new(0),
            additional_bytes: AtomicU64::new(0), additional_destinations: AtomicU64::new(0) })
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let (mut count, mut roots, mut destinations) = (0, 0u64, 0u64);
        let (mut additional_count, mut additional) = (0, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, table, root)) => {
                count += 1;
                if table >= model::TABLE_COUNT {
                    additional_count += 1;
                    additional |= 1u64 << (table - model::TABLE_COUNT);
                } else if root { roots |= 1u64 << table; } else { destinations |= 1u64 << table; }
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count, Ordering::Relaxed);
        self.roots.fetch_or(roots, Ordering::Relaxed);
        self.destinations.fetch_or(destinations, Ordering::Relaxed);
        self.additional_bytes.fetch_add(additional_count, Ordering::Relaxed);
        self.additional_destinations.fetch_or(additional, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> ThumbSwitchTablesStats {
        ThumbSwitchTablesStats { bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed), roots_mask: self.roots.load(Ordering::Relaxed),
            destinations_mask: self.destinations.load(Ordering::Relaxed),
            additional_source_bytes: self.additional_bytes.load(Ordering::Relaxed),
            additional_destinations_mask: self.additional_destinations.load(Ordering::Relaxed) }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.tables.capacity() * std::mem::size_of::<model::SwitchTable>()
            + self.data.tables.iter().map(|table| table.destinations.capacity() * std::mem::size_of::<model::ThumbDestination>()).sum::<usize>()
            + self.data.additional_tables.capacity() * std::mem::size_of::<model::SwitchTable>()
            + self.data.additional_tables.iter().map(|table| table.destinations.capacity() * std::mem::size_of::<model::ThumbDestination>()).sum::<usize>()
    }
}
