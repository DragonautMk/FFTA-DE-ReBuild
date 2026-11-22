//! Serve an authenticated compact record and observed pointer with fallback.
use super::*;
use crate::rom_data::soil_compact_record as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoilCompactRecordStats {
    pub bus_reads: u64,
    pub record_bytes: u64,
    pub pointer_bytes: u64,
}
pub(super) struct SoilCompactRecordResources {
    data: model::SoilCompactRecord,
    reads: AtomicU64,
    record_bytes: AtomicU64,
    pointer_bytes: AtomicU64,
}
impl SoilCompactRecordResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| page == 0x39a)
    }
    pub(super) fn new(
        data: model::SoilCompactRecord,
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
            return Err("Soil compact record source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            record_bytes: AtomicU64::new(0),
            pointer_bytes: AtomicU64::new(0),
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
        let (mut record_bytes, mut pointer_bytes) = (0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, 0)) => {
                record_bytes += 1;
                byte
            }
            Some((byte, 1)) => {
                pointer_bytes += 1;
                byte
            }
            Some(_) => unreachable!(),
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.record_bytes.fetch_add(record_bytes, Ordering::Relaxed);
        self.pointer_bytes
            .fetch_add(pointer_bytes, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> SoilCompactRecordStats {
        SoilCompactRecordStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            record_bytes: self.record_bytes.load(Ordering::Relaxed),
            pointer_bytes: self.pointer_bytes.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
