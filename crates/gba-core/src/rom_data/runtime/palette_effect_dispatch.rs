//! Original palette dispatch reads typed guest entries; object state/kind and callback code stay live.
use super::*;
use crate::rom_data::palette_effect_dispatch as model;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteEffectDispatchStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub callback_bytes: u64,
    pub null_bytes: u64,
    pub slot_mask: u64,
}
pub(super) struct PaletteEffectDispatchResources {
    data: model::PaletteEffectDispatch,
    counters: [AtomicU64; 4],
    slot_mask: AtomicU64,
}
impl PaletteEffectDispatchResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0xb94
    }
    pub(super) fn new(
        data: model::PaletteEffectDispatch,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Palette dispatch source binding count mismatch".into());
        }
        for ((offset, bytes, role), s) in expected.into_iter().zip(sources) {
            if s.offset != offset
                || s.length as usize != bytes.len()
                || s.role != role
                || s.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Palette dispatch typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            slot_mask: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize, bool)> {
        let n = at.checked_sub(self.data.source_offset as usize)?;
        let value = self.data.byte(n)?;
        Some((value, n / 4, self.data.slots[n / 4].target.is_some()))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [1, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((v, slot, active)) = fields[i] {
                counts[1] += 1;
                counts[if active { 2 } else { 3 }] += 1;
                self.slot_mask.fetch_or(1 << slot, Ordering::Relaxed);
                v
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        for (c, n) in self.counters.iter().zip(counts) {
            c.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> PaletteEffectDispatchStats {
        let [bus_reads, source_bytes, callback_bytes, null_bytes] =
            self.counters.each_ref().map(|n| n.load(Ordering::Relaxed));
        PaletteEffectDispatchStats {
            bus_reads,
            source_bytes,
            callback_bytes,
            null_bytes,
            slot_mask: self.slot_mask.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.slots.capacity() * std::mem::size_of::<model::Slot>()
    }
}
