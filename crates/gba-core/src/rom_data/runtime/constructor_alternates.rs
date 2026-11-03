//! Exact base-pointer and record bytes; original constructor and live patches still run.
use super::*;
use crate::rom_data::constructor_alternates as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConstructorAlternatesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub pointer_bytes: u64,
    pub header_bytes: u64,
    pub argument_bytes: u64,
    pub halfword_bytes: u64,
    pub flag_bytes: u64,
    pub tail_bytes: u64,
    pub record_mask: u64,
}
pub(super) struct ConstructorAlternatesResources {
    data: model::Alternates,
    counters: [AtomicU64; 9],
}
impl ConstructorAlternatesResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x6ec | 0x70c)
    }
    pub(super) fn new(
        data: model::Alternates,
        sources: &[SourceRange],
        owner: &crate::rom_data::items::Items,
    ) -> Result<Self, String> {
        data.validate_items(owner)?;
        if sources.len() != 2
            || sources[0].offset != model::POINTER_OFFSET
            || sources[0].length != 4
            || sources[0].role != "constructor-alternates/eur-base-pointer"
            || sources[0].sha256 != ffta_rom::digest(&data.pointer_bytes())
            || sources[1].offset != model::SOURCE_OFFSET
            || sources[1].length != (model::END_OFFSET - model::SOURCE_OFFSET)
            || sources[1].role != "constructor-alternates/eur-records"
            || sources[1].sha256 != ffta_rom::digest(&data.source_bytes())
        {
            return Err("Constructor-alternate source binding mismatch".into());
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let intersects =
            |start: u32, end: u32| index < end as usize && index.saturating_add(N) > start as usize;
        if !intersects(model::POINTER_OFFSET, model::POINTER_OFFSET + 4)
            && !intersects(model::SOURCE_OFFSET, model::END_OFFSET)
        {
            return prior;
        }
        let mut counts = [0u64; 8];
        let mut record_mask = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::POINTER_OFFSET as usize..model::POINTER_OFFSET as usize + 4).contains(&at) {
                counts[1] += 1;
                counts[2] += 1;
                self.data.pointer_bytes()[at - model::POINTER_OFFSET as usize]
            } else if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::SOURCE_OFFSET as usize;
                let record = offset / model::RECORD_BYTES;
                let field = offset % model::RECORD_BYTES;
                record_mask |= 1 << record;
                counts[1] += 1;
                counts[match field {
                    0..=4 => 3,
                    5..=7 => 4,
                    8..=41 => 5,
                    42..=43 => 6,
                    _ => 7,
                }] += 1;
                self.data.records[record].byte(field).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |b| b[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters[..8].iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.counters[8].fetch_or(record_mask, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ConstructorAlternatesStats {
        let [bus_reads, source_bytes, pointer_bytes, header_bytes, argument_bytes, halfword_bytes, flag_bytes, tail_bytes, record_mask] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        ConstructorAlternatesStats {
            bus_reads,
            source_bytes,
            pointer_bytes,
            header_bytes,
            argument_bytes,
            halfword_bytes,
            flag_bytes,
            tail_bytes,
            record_mask,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
