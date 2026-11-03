//! Typed minimum global source; original bit selection and live-mask loops remain.
use super::*;
use crate::rom_data::global_pair_flags as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlobalPairFlagsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub byte_mask: u16,
}
impl GlobalPairFlagsStats {
    pub fn bytes_used(&self) -> u32 {
        self.byte_mask.count_ones()
    }
}
pub(super) struct GlobalPairFlagsResources {
    data: model::GlobalPairFlags,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    byte_mask: AtomicU64,
}
impl GlobalPairFlagsResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e7
    }
    pub(super) fn new(
        data: model::GlobalPairFlags,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::SOURCE_BYTES as u32
            || sources[0].role != "global-pair-flags/eur-fields"
            || sources[0].sha256 != ffta_rom::digest(&data.source_bytes())
        {
            return Err("Global pair flags source binding mismatch".into());
        }
        Ok(Self {
            data,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            byte_mask: AtomicU64::new(0),
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
        let mut count = 0;
        let mut mask = 0;
        let bytes = std::array::from_fn(|i| {
            let at = index + i;
            if (model::SOURCE_OFFSET as usize..model::END_OFFSET as usize).contains(&at) {
                let offset = at - model::SOURCE_OFFSET as usize;
                count += 1;
                mask |= 1 << offset;
                self.data.byte(offset).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[at], |v| v[i])
            }
        });
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(count, Ordering::Relaxed);
        self.byte_mask.fetch_or(mask, Ordering::Relaxed);
        Some(bytes)
    }
    pub(super) fn stats(&self) -> GlobalPairFlagsStats {
        GlobalPairFlagsStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            byte_mask: self.byte_mask.load(Ordering::Relaxed) as u16,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
