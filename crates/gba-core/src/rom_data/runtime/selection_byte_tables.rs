//! Immutable byte storage; live eligibility, numeric helpers and choice stay guest code.
use super::super::{selection_byte_tables as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionByteTablesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub threshold_bytes: u64,
    pub weight_bytes: u64,
    pub threshold_positions_used: u64,
    pub weight_positions_used: u64,
}
pub(super) struct SelectionByteTablesResources {
    data: model::ByteTables,
    reads: AtomicU64,
    thresholds: AtomicU64,
    weights: AtomicU64,
    positions: AtomicU64,
}
impl SelectionByteTablesResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x71c
    }
    pub(super) fn new(data: model::ByteTables, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("SelectionByteTables binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("SelectionByteTables typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            thresholds: AtomicU64::new(0),
            weights: AtomicU64::new(0),
            positions: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index) {
            return prior;
        }
        let values: [_; N] = std::array::from_fn(|i| {
            u32::try_from(index + i)
                .ok()
                .and_then(|at| self.data.byte_at(at))
        });
        if values.iter().all(Option::is_none) {
            return prior;
        }
        let (mut thresholds, mut weights, mut positions) = (0, 0, 0u64);
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let position = index + i - model::SOURCE as usize;
                positions |= 1 << position;
                if position < model::THRESHOLD_COUNT {
                    thresholds += 1;
                } else {
                    weights += 1;
                }
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.thresholds.fetch_add(thresholds, Ordering::Relaxed);
        self.weights.fetch_add(weights, Ordering::Relaxed);
        self.positions.fetch_or(positions, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> SelectionByteTablesStats {
        let threshold_bytes = self.thresholds.load(Ordering::Relaxed);
        let weight_bytes = self.weights.load(Ordering::Relaxed);
        let positions = self.positions.load(Ordering::Relaxed);
        SelectionByteTablesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: threshold_bytes + weight_bytes,
            threshold_bytes,
            weight_bytes,
            threshold_positions_used: u64::from((positions & 15).count_ones()),
            weight_positions_used: u64::from((positions >> model::THRESHOLD_COUNT).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
