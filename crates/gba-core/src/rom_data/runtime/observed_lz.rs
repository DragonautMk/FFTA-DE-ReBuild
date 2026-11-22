//! Authenticated source views and decoded LZ77 HLE outputs.
use super::*;
use crate::rom_data::observed_lz::{self, ObservedLzBlock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedLzStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub hle_calls: u64,
    /// First the E174 block, then the six E175 blocks in source-spec order.
    pub hle_calls_by_block: [u64; 7],
    /// Seven interleaved row payloads added after the original seven blocks.
    pub interleaved_hle_calls_by_block: [u64; 7],
    /// Three streams observed in the Soil Evidence scene.
    pub soil_source_bytes_by_block: [u64; 3],
    pub soil_hle_calls_by_block: [u64; 3],
}

pub struct ObservedLzRead { owner: Arc<RuntimeResources>, index: usize }
impl ObservedLzRead {
    pub fn bytes(&self) -> &[u8] {
        let data = &self.owner.observed_lz.as_ref().unwrap().data;
        if self.index == 0 { &data.decoded } else { &data.additional_blocks[self.index - 1].decoded }
    }
}

pub(super) struct ObservedLzResources {
    data: ObservedLzBlock,
    counters: [AtomicU64; 2],
    hle_calls: [AtomicU64; 7],
    interleaved_hle_calls: [AtomicU64; 7],
    soil_source_bytes: [AtomicU64; 3],
    soil_hle_calls: [AtomicU64; 3],
}
impl ObservedLzResources {
    pub(super) fn new(data: ObservedLzBlock, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        if sources.len() != 1 + observed_lz::ADDITIONAL_SPECS.len()
            || data.streams().zip(sources).any(|((offset, encoded, _), source)|
                source.offset != offset || source.length as usize != encoded.len()
                    || source.role != "observed-red-spring-lz77" || source.sha256 != ffta_rom::digest(encoded))
        { return Err("Observed LZ source binding mismatch".into()); }
        Ok(Self { data, counters: std::array::from_fn(|_| AtomicU64::new(0)),
            hle_calls: std::array::from_fn(|_| AtomicU64::new(0)),
            interleaved_hle_calls: std::array::from_fn(|_| AtomicU64::new(0)),
            soil_source_bytes: std::array::from_fn(|_| AtomicU64::new(0)),
            soil_hle_calls: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn source_page(index: usize) -> bool {
        let in_page = |start: usize, len: usize|
            (start >> 12..=(start + len - 1) >> 12).contains(&(index >> 12));
        in_page(observed_lz::OFFSET as usize, observed_lz::ENCODED_LENGTH)
            || observed_lz::ADDITIONAL_SPECS.iter().any(|s| in_page(s.offset as usize, s.encoded_length))
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        if !self.data.streams().any(|(offset, encoded, _)| {
            let start = offset as usize;
            index < start + encoded.len() && index.saturating_add(N) > start
        }) { return prior; }
        let mut served = 0;
        let mut soil_bytes = [0u64; 3];
        let result = std::array::from_fn(|i| {
            let at = index + i;
            if let Some(byte) = self.data.streams().find_map(|(offset, encoded, _)|
                at.checked_sub(offset as usize).and_then(|n| encoded.get(n)).copied()) {
                served += 1;
                for (group, spec) in observed_lz::ADDITIONAL_SPECS[13..16].iter().enumerate() {
                    let start = spec.offset as usize;
                    if (start..start + spec.encoded_length).contains(&at) { soil_bytes[group] += 1; }
                }
                byte
            } else { prior.as_ref().map_or_else(|| rom[at], |bytes| bytes[i]) }
        });
        self.counters[0].fetch_add(1, Ordering::Relaxed);
        self.counters[1].fetch_add(served, Ordering::Relaxed);
        for (counter, count) in self.soil_source_bytes.iter().zip(soil_bytes) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(result)
    }
    pub(super) fn stats(&self) -> ObservedLzStats {
        let hle_calls_by_block = self.hle_calls.each_ref().map(|c| c.load(Ordering::Relaxed));
        let interleaved_hle_calls_by_block = self.interleaved_hle_calls.each_ref().map(|c| c.load(Ordering::Relaxed));
        let soil_source_bytes_by_block = self.soil_source_bytes.each_ref().map(|c| c.load(Ordering::Relaxed));
        let soil_hle_calls_by_block = self.soil_hle_calls.each_ref().map(|c| c.load(Ordering::Relaxed));
        ObservedLzStats { bus_reads: self.counters[0].load(Ordering::Relaxed),
            source_bytes: self.counters[1].load(Ordering::Relaxed),
            hle_calls: hle_calls_by_block.iter().chain(&interleaved_hle_calls_by_block).chain(&soil_hle_calls_by_block).sum(),
            hle_calls_by_block, interleaved_hle_calls_by_block, soil_source_bytes_by_block, soil_hle_calls_by_block }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.encoded_source.capacity() + self.data.decoded.capacity()
            + self.data.additional_blocks.capacity() * std::mem::size_of::<observed_lz::ObservedLzStream>()
            + self.data.additional_blocks.iter().map(|s| s.encoded_source.capacity() + s.decoded.capacity()).sum::<usize>()
    }
}
impl RuntimeResources {
    pub(crate) fn decoded_observed_lz(self: &Arc<Self>, source: u32) -> Option<ObservedLzRead> {
        if !matches!(source >> 24, 0x08..=0x0d) { return None; }
        let block = self.observed_lz.as_ref()?;
        let index = block.data.streams().position(|(offset, _, _)| offset == (source & 0x01ff_ffff))?;
        if index < block.hle_calls.len() { block.hle_calls[index].fetch_add(1, Ordering::Relaxed); }
        else if index < block.hle_calls.len() + block.interleaved_hle_calls.len() {
            block.interleaved_hle_calls[index - block.hle_calls.len()].fetch_add(1, Ordering::Relaxed);
        } else {
            block.soil_hle_calls[index - block.hle_calls.len() - block.interleaved_hle_calls.len()].fetch_add(1, Ordering::Relaxed);
        }
        Some(ObservedLzRead { owner: self.clone(), index })
    }
}
