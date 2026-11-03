//! Exact typed parameter fragments; original task logic remains guest code.
use super::super::{observed_task_parameters as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTaskParametersStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub phase_threshold_bytes: u64,
    pub vertical_correction_bytes: u64,
    pub phases_used: u64,
    pub corrections_used: u64,
}
pub(super) struct ObservedTaskParametersResources {
    data: model::ObservedTaskParameters,
    counts: [AtomicU64; 4],
    used: AtomicU64,
}
impl ObservedTaskParametersResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x371 | 0x394)
    }
    pub(super) fn new(
        data: model::ObservedTaskParameters,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Task parameter source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Task parameter typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            used: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let at = u32::try_from(at).ok()?;
        if let Some(value) = self.data.phase_thresholds.byte_at(at) {
            self.used
                .fetch_or(1 << (at - model::PHASE_SOURCE), Ordering::Relaxed);
            Some((value, 2))
        } else {
            let value = self.data.vertical_corrections.byte_at(at)?;
            self.used.fetch_or(
                1 << (model::PHASE_COUNT as u32 + (at - model::VERTICAL_SOURCE) / 2),
                Ordering::Relaxed,
            );
            Some((value, 3))
        }
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
        let mut counts = [0u64; 4];
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        counts[0] = 1;
        let bytes = std::array::from_fn(|i| match fields[i] {
            Some((value, kind)) => {
                counts[1] += 1;
                counts[kind] += 1;
                value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        for (counter, n) in self.counts.iter().zip(counts) {
            counter.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ObservedTaskParametersStats {
        let [bus_reads, source_bytes, phase_threshold_bytes, vertical_correction_bytes] =
            self.counts.each_ref().map(|i| i.load(Ordering::Relaxed));
        let used = self.used.load(Ordering::Relaxed);
        ObservedTaskParametersStats {
            bus_reads,
            source_bytes,
            phase_threshold_bytes,
            vertical_correction_bytes,
            phases_used: u64::from((used & ((1 << model::PHASE_COUNT) - 1)).count_ones()),
            corrections_used: u64::from((used >> model::PHASE_COUNT).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.phase_thresholds.entries.capacity()
                * std::mem::size_of::<model::PhaseThreshold>()
            + self.data.vertical_corrections.entries.capacity()
                * std::mem::size_of::<model::VerticalCorrection>()
    }
}
