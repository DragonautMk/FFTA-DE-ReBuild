//! Serve the selected queued halfword transfer while preserving earlier owners and ROM fallback.
use super::*;
use crate::rom_data::firaga_queued_transfer as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FiragaQueuedTransferStats {
    pub bus_reads: u64,
    pub source_bytes: [u64; 3],
}
pub(super) struct FiragaQueuedTransferResources {
    spans: Vec<(u32, Vec<u8>)>,
    reads: AtomicU64,
    counts: [AtomicU64; 3],
}
impl FiragaQueuedTransferResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|p| matches!(p, 0xac | 0x40e))
    }
    pub(super) fn new(
        data: model::FiragaQueuedTransfer,
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
            return Err("Firaga queued source binding mismatch".into());
        }
        Ok(Self {
            spans: spans
                .into_iter()
                .map(|(offset, bytes, _)| (offset, bytes))
                .collect(),
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
        let cells: [_; N] = std::array::from_fn(|i| {
            self.spans
                .iter()
                .enumerate()
                .find_map(|(owner, (offset, bytes))| {
                    (index + i)
                        .checked_sub(*offset as usize)
                        .and_then(|n| bytes.get(n).copied())
                        .map(|byte| (byte, owner))
                })
        });
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 3];
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
    pub(super) fn stats(&self) -> FiragaQueuedTransferStats {
        FiragaQueuedTransferStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.counts.each_ref().map(|c| c.load(Ordering::Relaxed)),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.spans.capacity() * std::mem::size_of::<(u32, Vec<u8>)>()
            + self.spans.iter().map(|(_, b)| b.capacity()).sum::<usize>()
    }
}
