//! Serve authenticated Firaga matrices and literal words with ordinary fallback.
use super::*;
use crate::rom_data::firaga_copied_matrices as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FiragaCopiedMatricesStats {
    pub bus_reads: u64,
    pub literal_bytes: [u64; 4],
    pub matrix_bytes: [u64; 2],
}
pub(super) struct FiragaCopiedMatricesResources {
    data: model::FiragaCopiedMatrices,
    reads: AtomicU64,
    counts: [AtomicU64; 6],
}
impl FiragaCopiedMatricesResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| page == 0x2d || page == 0x2e || page == 0x3b9 || page == 0x3ba)
    }
    pub(super) fn new(data: model::FiragaCopiedMatrices, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len()
                || source.role != *role || source.sha256 != ffta_rom::digest(bytes)
        }) { return Err("Firaga matrix source binding mismatch".into()); }
        Ok(Self { data, reads: AtomicU64::new(0), counts: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) { return prior; }
        let mut counts = [0u64; 6];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, entry)) => { counts[entry] += 1; byte },
            None => prior.as_ref().map_or_else(|| rom[index+i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.counts.iter().zip(counts) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }
    pub(super) fn stats(&self) -> FiragaCopiedMatricesStats {
        let counts = self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        FiragaCopiedMatricesStats { bus_reads: self.reads.load(Ordering::Relaxed),
            literal_bytes: counts[..4].try_into().unwrap(), matrix_bytes: counts[4..].try_into().unwrap() }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.matrices.iter().map(|m| m.cells.capacity() * std::mem::size_of::<u16>()).sum::<usize>()
    }
}
