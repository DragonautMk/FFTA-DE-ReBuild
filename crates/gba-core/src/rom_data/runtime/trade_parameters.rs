//! Typed five-record source; original selectors, price arithmetic and live RAM remain active.
use super::*;
use crate::rom_data::trade_parameters as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TradeParametersStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub opaque_bytes: u64,
    pub markup_low_bytes: u64,
    pub shared41_bytes: u64,
    pub discount_bytes: u64,
    pub record_mask: u64,
}
pub(super) struct TradeParametersResources {
    data: model::TradeParameters,
    counters: [AtomicU64; 6],
    record_mask: AtomicU64,
}
impl TradeParametersResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e8
    }
    pub(super) fn new(
        data: model::TradeParameters,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1 {
            return Err("Trade parameters require one source binding".into());
        }
        let source = &sources[0];
        let bytes = data.source_bytes();
        if source.offset != model::SOURCE_OFFSET
            || source.length as usize != bytes.len()
            || source.role != "trade-parameters/eur-fixed-five"
            || source.sha256 != ffta_rom::digest(&bytes)
        {
            return Err("Trade parameter source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            record_mask: AtomicU64::new(0),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= model::END_OFFSET as usize
            || index.saturating_add(N) <= model::SOURCE_OFFSET as usize
        {
            return prior;
        }
        let mut counts = [0u64; 6];
        let mut mask = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::SOURCE_OFFSET as usize;
                let field = offset % model::RECORD_BYTES;
                counts[1] += 1;
                counts[match field {
                    0..=63 => 2,
                    64 => 3,
                    65 => 4,
                    _ => 5,
                }] += 1;
                mask |= 1u64 << (offset / model::RECORD_BYTES);
                self.data.byte(offset).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.record_mask.fetch_or(mask, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> TradeParametersStats {
        let [bus_reads, source_bytes, opaque_bytes, markup_low_bytes, shared41_bytes, discount_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        TradeParametersStats {
            bus_reads,
            source_bytes,
            opaque_bytes,
            markup_low_bytes,
            shared41_bytes,
            discount_bytes,
            record_mask: self.record_mask.load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
