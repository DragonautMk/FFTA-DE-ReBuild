//! Typed event/selector/even-target bytes; guest sequencing and live indices stay original.
use super::*;
use crate::rom_data::effect_event_directories as model;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EffectEventDirectoryStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub event_bytes: u64,
    pub selector_bytes: u64,
    pub dispatch_bytes: u64,
    pub event_records_used: u64,
    pub selectors_used: u64,
    pub dispatch_tables_used: u64,
}
pub(super) struct EffectEventDirectoryResources {
    data: model::EffectEventDirectories,
    counters: [AtomicU64; 5],
    events: [AtomicU64; 2],
    selectors: AtomicU64,
    dispatch: AtomicU64,
}
impl EffectEventDirectoryResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x124 | 0x128 | 0x3ac | 0x3ad)
    }
    pub(super) fn new(
        data: model::EffectEventDirectories,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Effect event binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Effect event typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            events: std::array::from_fn(|_| AtomicU64::new(0)),
            selectors: AtomicU64::new(0),
            dispatch: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        if let Some(n) = at.checked_sub(self.data.event_source_offset as usize) {
            if let Some(e) = self.data.events.get(n / 16) {
                let byte = e.byte(n % 16)?;
                self.events[n / 16 / 64].fetch_or(1 << ((n / 16) % 64), Ordering::Relaxed);
                return Some((byte, 2));
            }
        }
        if let Some(n) = at.checked_sub(self.data.selector_source_offset as usize) {
            if let Some(s) = self.data.selectors.get(n / 4) {
                let byte = s.byte(n % 4)?;
                self.selectors.fetch_or(1 << (n / 4), Ordering::Relaxed);
                return Some((byte, 3));
            }
        }
        for (i, d) in [&self.data.event_dispatch, &self.data.selector_dispatch]
            .into_iter()
            .enumerate()
        {
            if let Some(n) = at.checked_sub(d.source_offset as usize) {
                if let Some(slot) = d.slots.get(n / 4) {
                    let byte = slot.target.guest_word()?.to_le_bytes()[n % 4];
                    self.dispatch.fetch_or(1 << i, Ordering::Relaxed);
                    return Some((byte, 4));
                }
            }
        }
        None
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
        let mut counts = [1, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> EffectEventDirectoryStats {
        let [bus_reads, source_bytes, event_bytes, selector_bytes, dispatch_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        EffectEventDirectoryStats {
            bus_reads,
            source_bytes,
            event_bytes,
            selector_bytes,
            dispatch_bytes,
            event_records_used: self
                .events
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
            selectors_used: u64::from(self.selectors.load(Ordering::Relaxed).count_ones()),
            dispatch_tables_used: u64::from(self.dispatch.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.events.capacity() * std::mem::size_of::<model::Event>()
            + self.data.selectors.capacity() * std::mem::size_of::<model::Selector>()
            + (self.data.event_dispatch.slots.capacity()
                + self.data.selector_dispatch.slots.capacity())
                * std::mem::size_of::<model::DispatchSlot>()
    }
}
