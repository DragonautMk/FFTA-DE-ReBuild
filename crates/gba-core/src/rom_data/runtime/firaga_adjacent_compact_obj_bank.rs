//! Serve the selected adjacent compact OBJ bank and descriptor fields with normal fallback.
use super::*;
use crate::rom_data::firaga_adjacent_compact_obj_bank as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FiragaAdjacentCompactObjBankStats {
    pub bus_reads: u64,
    pub source_bytes: [u64; 7],
}
pub(super) struct FiragaAdjacentCompactObjBankResources {
    data: model::FiragaAdjacentCompactObjBank,
    reads: AtomicU64,
    counts: [AtomicU64; 7],
}
impl FiragaAdjacentCompactObjBankResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12))
            .any(|page| matches!(page, 0xeb | 0x39c | 0x6b3 | 0x6b4))
    }
    pub(super) fn new(
        data: model::FiragaAdjacentCompactObjBank,
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
            return Err("Adjacent Firaga compact OBJ source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut counts = [0u64; 7];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, owner)) => {
                counts[owner] += 1;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.counts.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(result)
    }
    pub(super) fn stats(&self) -> FiragaAdjacentCompactObjBankStats {
        FiragaAdjacentCompactObjBankStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self
                .counts
                .each_ref()
                .map(|counter| counter.load(Ordering::Relaxed)),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.frames.capacity()
                * std::mem::size_of::<crate::rom_data::compact_obj_animations::Frame>()
    }
}
