//! Serve authenticated indirect records without bounding guest pointers.
use super::*;
use crate::rom_data::soil_indirect_records as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoilIndirectRecordsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub record_mask: u8,
    pub star_cross_bytes: u64,
    pub star_cross_mask: u8,
}
pub(super) struct SoilIndirectRecordsResources {
    data: model::SoilIndirectRecords,
    reads: AtomicU64,
    source_bytes: AtomicU64,
    records: AtomicU64,
    star_cross_bytes: AtomicU64,
}
impl SoilIndirectRecordsResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| matches!(page, 0x39a | 0x39c | 0x39e))
    }
    pub(super) fn new(
        data: model::SoilIndirectRecords,
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
            return Err("Soil indirect record source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            records: AtomicU64::new(0),
            star_cross_bytes: AtomicU64::new(0),
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
        let (mut count, mut mask, mut star_cross_bytes) = (0u64, 0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => {
                count += 1;
                mask |= 1 << entry;
                if entry >= 7 { star_cross_bytes += 1; }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(count, Ordering::Relaxed);
        self.records.fetch_or(mask, Ordering::Relaxed);
        self.star_cross_bytes.fetch_add(star_cross_bytes, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> SoilIndirectRecordsStats {
        SoilIndirectRecordsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            record_mask: (self.records.load(Ordering::Relaxed) & 0x7f) as u8,
            star_cross_bytes: self.star_cross_bytes.load(Ordering::Relaxed),
            star_cross_mask: ((self.records.load(Ordering::Relaxed) >> 7) & 0x7) as u8,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
