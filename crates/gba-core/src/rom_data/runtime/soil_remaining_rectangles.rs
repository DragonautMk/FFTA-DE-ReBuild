//! Serve authenticated Soil rectangles and one literal with ordinary fallback.
use super::*;
use crate::rom_data::soil_remaining_rectangles as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoilRemainingRectanglesStats {
    pub bus_reads: u64,
    pub literal_bytes: u64,
    pub rectangle_bytes: [u64; 2],
}
pub(super) struct SoilRemainingRectanglesResources {
    data: model::SoilRemainingRectangles,
    reads: AtomicU64,
    literal_bytes: AtomicU64,
    rectangle_bytes: [AtomicU64; 2],
}
impl SoilRemainingRectanglesResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| page == 0xe6 || page == 0xae7)
    }
    pub(super) fn new(
        data: model::SoilRemainingRectangles,
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
            return Err("Soil remaining rectangle source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            literal_bytes: AtomicU64::new(0),
            rectangle_bytes: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut counts = [0u64; 3];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => {
                counts[entry] += 1;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.literal_bytes.fetch_add(counts[0], Ordering::Relaxed);
        for (counter, count) in self.rectangle_bytes.iter().zip(&counts[1..]) {
            counter.fetch_add(*count, Ordering::Relaxed);
        }
        Some(result)
    }
    pub(super) fn stats(&self) -> SoilRemainingRectanglesStats {
        SoilRemainingRectanglesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            literal_bytes: self.literal_bytes.load(Ordering::Relaxed),
            rectangle_bytes: std::array::from_fn(|i| {
                self.rectangle_bytes[i].load(Ordering::Relaxed)
            }),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .data
                .rectangles
                .iter()
                .map(|r| r.cells.capacity() * std::mem::size_of::<u16>())
                .sum::<usize>()
    }
}
