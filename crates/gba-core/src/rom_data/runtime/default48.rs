//! Typed default bytes; original IWRAM copy, live patches and allocation still execute.
use super::*;
use crate::rom_data::default48 as model;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Default48Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub header_bytes: u64,
    pub argument_bytes: u64,
    pub halfword_bytes: u64,
    pub flag_bytes: u64,
    pub tail_bytes: u64,
}
pub(super) struct Default48Resources {
    data: model::Template,
    counters: [AtomicU64; 7],
}
impl Default48Resources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x6e6
    }
    pub(super) fn new(data: model::Template, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1
            || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length != model::RECORD_BYTES as u32
            || sources[0].role != "default48/eur-fixed"
            || sources[0].sha256 != ffta_rom::digest(&data.source_bytes())
        {
            return Err("Default48 source binding mismatch".into());
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
        if index >= model::END_OFFSET as usize
            || index.saturating_add(N) <= model::SOURCE_OFFSET as usize
        {
            return prior;
        }
        let mut counts = [0u64; 7];
        let bytes = std::array::from_fn(|i| {
            let at = (index + i).checked_sub(model::SOURCE_OFFSET as usize);
            if let Some(at) = at.filter(|&at| at < model::RECORD_BYTES) {
                counts[1] += 1;
                counts[match at {
                    0..=4 => 2,
                    5..=7 => 3,
                    8..=41 => 4,
                    42..=43 => 5,
                    _ => 6,
                }] += 1;
                self.data.byte(at).unwrap()
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |b| b[i])
            }
        });
        counts[0] = 1;
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> Default48Stats {
        let [bus_reads, source_bytes, header_bytes, argument_bytes, halfword_bytes, flag_bytes, tail_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        Default48Stats {
            bus_reads,
            source_bytes,
            header_bytes,
            argument_bytes,
            halfword_bytes,
            flag_bytes,
            tail_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
