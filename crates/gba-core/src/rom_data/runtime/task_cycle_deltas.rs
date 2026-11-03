//! Typed cycle bytes only; original guest lookup and arithmetic stay intact.
use super::super::{task_cycle_deltas as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskCycleDeltasStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub phases_used: u64,
}
pub(super) struct TaskCycleDeltasResources {
    data: model::TaskCycleDeltas,
    reads: AtomicU64,
    bytes: AtomicU64,
    used: AtomicU64,
}
impl TaskCycleDeltasResources {
    pub(super) fn new(
        data: model::TaskCycleDeltas,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if sources.len() != 1 {
            return Err("Task-cycle source binding count mismatch".into());
        }
        let (offset, bytes, role) = &spans[0];
        let source = &sources[0];
        if source.offset != *offset
            || source.length as usize != bytes.len()
            || source.role != *role
            || source.sha256 != ffta_rom::digest(bytes)
        {
            return Err("Task-cycle typed source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            used: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >> 12 != 0x371 {
            return prior;
        }
        let fields: [_; N] = std::array::from_fn(|i| {
            u32::try_from(index + i)
                .ok()
                .and_then(|at| self.data.byte_at(at))
        });
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut count = 0;
        let mut used = 0;
        let bytes = std::array::from_fn(|i| match fields[i] {
            Some(value) => {
                count += 1;
                used |= 1 << (index + i - model::SOURCE as usize);
                value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count, Ordering::Relaxed);
        self.used.fetch_or(used, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> TaskCycleDeltasStats {
        TaskCycleDeltasStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            phases_used: u64::from(self.used.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.entries.capacity() * std::mem::size_of::<model::CycleDelta>()
    }
}
