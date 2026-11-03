//! File-derived threshold/pointer pairs; guest animation state stays in RAM.
use super::super::{obj_lists::constructor_animation as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConstructorObjAnimationStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub threshold_bytes: u64,
    pub pointer_bytes: u64,
    pub phases_used: u64,
}

pub(super) struct ConstructorObjAnimationResources {
    data: model::Animation,
    counts: [AtomicU64; 4],
    used: AtomicU64,
}
impl ConstructorObjAnimationResources {
    pub(super) fn new(data: model::Animation, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Constructor OBJ animation source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Constructor OBJ animation typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            used: AtomicU64::new(0),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let index = u32::try_from(index).ok()?;
        let value = self.data.byte_at(index)?;
        let relative = index - model::TABLE_OFFSET;
        self.used.fetch_or(1 << (relative / 8), Ordering::Relaxed);
        Some((value, if relative % 8 < 4 { 2 } else { 3 }))
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >> 12 != 0x394 {
            return prior;
        }
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [1u64, 0, 0, 0];
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

    pub(super) fn stats(&self) -> ConstructorObjAnimationStats {
        let [bus_reads, source_bytes, threshold_bytes, pointer_bytes] =
            self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        ConstructorObjAnimationStats {
            bus_reads,
            source_bytes,
            threshold_bytes,
            pointer_bytes,
            phases_used: u64::from(self.used.load(Ordering::Relaxed).count_ones()),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.frames.capacity() * std::mem::size_of::<model::Frame>()
    }
}
