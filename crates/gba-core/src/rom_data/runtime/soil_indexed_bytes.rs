//! Serve witnessed indexed bytes without restricting guest indexing.
use super::*;
use crate::rom_data::soil_indexed_bytes as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoilIndexedBytesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub values_mask: u8,
}
pub(super) struct SoilIndexedBytesResources {
    data: model::SoilIndexedBytes,
    reads: AtomicU64,
    source_bytes: AtomicU64,
    values: AtomicU64,
}
impl SoilIndexedBytesResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| page == 0x39e)
    }
    pub(super) fn new(
        data: model::SoilIndexedBytes,
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
            return Err("Soil indexed byte source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
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
        let (mut count, mut mask) = (0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => {
                count += 1;
                mask |= 1 << entry;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(count, Ordering::Relaxed);
        self.values.fetch_or(mask, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> SoilIndexedBytesStats {
        SoilIndexedBytesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            values_mask: self.values.load(Ordering::Relaxed) as u8,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
