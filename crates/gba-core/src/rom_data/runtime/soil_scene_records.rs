//! Serve authenticated Soil Evidence records without bounding guest indexing.
use super::*;
use crate::rom_data::soil_scene_records as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoilSceneRecordsStats {
    pub bus_reads: u64,
    pub range_bytes: u64,
    pub sequence_bytes: u64,
    pub range_mask: u8,
    pub sequence_mask: u16,
}
pub(super) struct SoilSceneRecordsResources {
    data: model::SoilSceneRecords,
    reads: AtomicU64,
    range_bytes: AtomicU64,
    sequence_bytes: AtomicU64,
    ranges: AtomicU64,
    sequence: AtomicU64,
}
impl SoilSceneRecordsResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| page == 0x39e)
    }
    pub(super) fn new(
        data: model::SoilSceneRecords,
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
            return Err("Soil scene record source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            range_bytes: AtomicU64::new(0),
            sequence_bytes: AtomicU64::new(0),
            ranges: AtomicU64::new(0),
            sequence: AtomicU64::new(0),
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
        let (mut range_bytes, mut sequence_bytes, mut ranges, mut sequence) =
            (0u64, 0u64, 0u64, 0u64);
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, 0, entry)) => {
                range_bytes += 1;
                ranges |= 1 << entry;
                byte
            }
            Some((byte, 1, entry)) => {
                sequence_bytes += 1;
                sequence |= 1 << entry;
                byte
            }
            Some(_) => unreachable!(),
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.range_bytes.fetch_add(range_bytes, Ordering::Relaxed);
        self.sequence_bytes
            .fetch_add(sequence_bytes, Ordering::Relaxed);
        self.ranges.fetch_or(ranges, Ordering::Relaxed);
        self.sequence.fetch_or(sequence, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> SoilSceneRecordsStats {
        SoilSceneRecordsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            range_bytes: self.range_bytes.load(Ordering::Relaxed),
            sequence_bytes: self.sequence_bytes.load(Ordering::Relaxed),
            range_mask: self.ranges.load(Ordering::Relaxed) as u8,
            sequence_mask: self.sequence.load(Ordering::Relaxed) as u16,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
