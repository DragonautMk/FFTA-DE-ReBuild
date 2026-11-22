//! Serve the selected compact OBJ bank and indirect roots with normal fallback.
use super::*;
use crate::rom_data::firaga_compact_obj_bank as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FiragaCompactObjBankStats {
    pub bus_reads: u64,
    pub source_bytes: [u64; 5],
}
pub(super) struct FiragaCompactObjBankResources {
    data: model::FiragaCompactObjBank,
    reads: AtomicU64,
    counts: [AtomicU64; 5],
}
impl FiragaCompactObjBankResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12))
            .any(|page| matches!(page, 0xeb | 0x39c | 0x6b3))
    }
    pub(super) fn new(
        data: model::FiragaCompactObjBank,
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
            return Err("Firaga compact OBJ source binding mismatch".into());
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
        let mut counts = [0u64; 5];
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
    pub(super) fn stats(&self) -> FiragaCompactObjBankStats {
        FiragaCompactObjBankStats {
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
