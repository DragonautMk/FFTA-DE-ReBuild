//! Serve the three original indexed halfwords while preserving mixed reads.
use super::*;
use crate::rom_data::indexed_halfwords_39a5 as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndexedHalfwords39a5Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub values_mask: u8,
}

pub(super) struct IndexedHalfwords39a5Resources {
    data: model::IndexedHalfwords39a5,
    reads: AtomicU64,
    bytes: AtomicU64,
    values: AtomicU64,
}

impl IndexedHalfwords39a5Resources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        (first..=last).any(|page| page == 0x39a)
    }

    pub(super) fn new(
        data: model::IndexedHalfwords39a5,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let bytes = data.source_bytes()?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len()
            || sources[0].role != "indexed-halfwords-39a5c8"
            || sources[0].sha256 != ffta_rom::digest(&bytes)
        {
            return Err("Indexed halfword source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
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
        let (mut bytes, mut values) = (0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => {
                bytes += 1;
                values |= 1 << entry;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
        self.values.fetch_or(values, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> IndexedHalfwords39a5Stats {
        IndexedHalfwords39a5Stats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            values_mask: self.values.load(Ordering::Relaxed) as u8,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
