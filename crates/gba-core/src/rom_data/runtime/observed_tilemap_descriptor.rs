//! Reconstruct selected descriptors and LZ wrappers while guest task state stays live.
use super::super::{observed_tilemap_descriptor as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedTilemapDescriptorStats { pub bus_reads: u64, pub source_bytes: u64, pub descriptor_bytes: u64, pub wrapper_bytes: u64, pub transfer_descriptor_bytes: u64, pub transfer_wrapper_bytes: u64, pub additional_tilemap_descriptor_bytes: u64, pub interleaved_lz_descriptor_bytes: u64, pub interleaved_lz_wrapper_bytes: u64, pub soil_descriptor_mask: u8, pub soil_wrapper_mask: u8, pub soil_mode7_descriptor_bytes: u64, pub firaga_transfer_descriptor_bytes: u64, pub star_cross_descriptor_bytes: [u64; 3], pub star_cross_wrapper_bytes: [u64; 2] }
pub(super) struct ObservedTilemapDescriptorResources { data: model::ObservedTilemapDescriptor, reads: AtomicU64, bytes: [AtomicU64; 22] }

impl ObservedTilemapDescriptorResources {
    pub(super) fn source_page(index: usize) -> bool {
        ((index >> 12)..=(index.saturating_add(3) >> 12)).any(|page| matches!(page, 0x399 | 0x39a | 0x39d | 0x39e | 0x3a4 | 0xaff | 0xb00 | 0xb02 | 0xb03 | 0xb0b | 0xb0d | 0xb0e | 0xb0f | 0xb15))
    }
    pub(super) fn new(data: model::ObservedTilemapDescriptor, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let spans = data.source_spans();
        if sources.len() != spans.len() || spans.iter().zip(sources).any(|((offset, bytes, role), source)| {
            source.offset != *offset || source.length as usize != bytes.len()
                || source.role != *role || source.sha256 != ffta_rom::digest(bytes)
        }) { return Err("Observed tilemap descriptor source binding mismatch".into()); }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        if let Some(value) = index.checked_sub(self.data.source_offset as usize).and_then(|at| self.data.source_bytes().get(at).copied()) {
            return Some((value, 0));
        }
        if let Some(entries) = &self.data.additional_tilemap_descriptors {
            if let Some(value) = entries.iter().find_map(|entry| {
                index.checked_sub(entry.source_offset as usize).and_then(|at| entry.source_bytes().get(at).copied())
            }) { return Some((value, 4)); }
        }
        if let Some(entries) = &self.data.interleaved_lz_descriptors {
            if let Some(value) = entries.iter().find_map(|entry| {
                index.checked_sub(entry.source_offset as usize).and_then(|at| entry.source_bytes().get(at).copied()).map(|value| (value, 5))
                    .or_else(|| index.checked_sub(entry.wrapper.source_offset as usize).and_then(|at| entry.wrapper.source_bytes().get(at).copied()).map(|value| (value, 6)))
            }) { return Some(value); }
        }
        if let Some(entries) = &self.data.soil_transfer_descriptors {
            if let Some(value) = entries.iter().enumerate().find_map(|(entry_index, entry)| {
                index.checked_sub(entry.source_offset as usize).and_then(|at| entry.source_bytes().get(at).copied()).map(|value| (value, 7 + entry_index))
                    .or_else(|| (entry_index != 1).then(|| index.checked_sub(entry.wrapper.source_offset as usize)
                        .and_then(|at| entry.wrapper.source_bytes().get(at).copied()).map(|value| (value, 11 + entry_index))).flatten())
            }) { return Some(value); }
        }
        if let Some(entry) = &self.data.soil_mode7_descriptor {
            if let Some(value) = index.checked_sub(entry.source_offset as usize)
                .and_then(|at| entry.source_bytes().get(at).copied()) {
                return Some((value, 15));
            }
        }
        if let Some(entry) = &self.data.firaga_transfer_descriptor {
            if let Some(value) = index.checked_sub(entry.source_offset as usize)
                .and_then(|at| entry.source_bytes().get(at).copied()) {
                return Some((value, 16));
            }
        }
        if let Some(entries) = &self.data.star_cross_lz_descriptors {
            if let Some(value) = entries.iter().enumerate().find_map(|(i, entry)| {
                index.checked_sub(entry.source_offset as usize).and_then(|at| entry.source_bytes().get(at).copied()).map(|value| (value, 17 + i))
                    .or_else(|| index.checked_sub(entry.wrapper.source_offset as usize).and_then(|at| entry.wrapper.source_bytes().get(at).copied()).map(|value| (value, 19 + i)))
            }) { return Some(value); }
        }
        if let Some(entry) = &self.data.star_cross_mode7_descriptor {
            if let Some(value) = index.checked_sub(entry.source_offset as usize).and_then(|at| entry.source_bytes().get(at).copied()) {
                return Some((value, 21));
            }
        }
        self.data.additional_descriptors.iter().map(|entry| (entry, 0))
            .chain(self.data.transfer_descriptors.iter().map(|entry| (entry, 2))).find_map(|(entry, kind)| {
            index.checked_sub(entry.source_offset as usize).and_then(|at| entry.source_bytes().get(at).copied()).map(|value| (value, kind))
                .or_else(|| index.checked_sub(entry.wrapper.source_offset as usize).and_then(|at| entry.wrapper.source_bytes().get(at).copied()).map(|value| (value, kind + 1)))
        })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if cells.iter().all(Option::is_none) { return prior; }
        let mut counts = [0u64; 22];
        let result = std::array::from_fn(|i| match cells[i] {
            Some((value, kind)) => { counts[kind] += 1; value }
            None => prior.as_ref().map_or_else(|| rom[index + i], |v| v[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.bytes.iter().zip(counts) { counter.fetch_add(count, Ordering::Relaxed); }
        Some(result)
    }
    pub(super) fn stats(&self) -> ObservedTilemapDescriptorStats {
        let counts = self.bytes.each_ref().map(|counter| counter.load(Ordering::Relaxed));
        let [old_descriptor, old_wrapper, transfer_descriptor_bytes, transfer_wrapper_bytes, additional_tilemap_descriptor_bytes, interleaved_lz_descriptor_bytes, interleaved_lz_wrapper_bytes, ..] = counts;
        let descriptor_bytes = old_descriptor + transfer_descriptor_bytes + additional_tilemap_descriptor_bytes + interleaved_lz_descriptor_bytes + counts[7..11].iter().sum::<u64>() + counts[15] + counts[16] + counts[17] + counts[18] + counts[21];
        let wrapper_bytes = old_wrapper + transfer_wrapper_bytes + interleaved_lz_wrapper_bytes + counts[11..15].iter().sum::<u64>() + counts[19] + counts[20];
        let soil_descriptor_mask = counts[7..11].iter().enumerate().fold(0, |mask, (i, &count)| mask | if count > 0 { 1 << i } else { 0 });
        let soil_wrapper_mask = counts[11..15].iter().enumerate().fold(0, |mask, (i, &count)| mask | if count > 0 { 1 << i } else { 0 });
        ObservedTilemapDescriptorStats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: descriptor_bytes + wrapper_bytes, descriptor_bytes, wrapper_bytes, transfer_descriptor_bytes, transfer_wrapper_bytes, additional_tilemap_descriptor_bytes, interleaved_lz_descriptor_bytes, interleaved_lz_wrapper_bytes, soil_descriptor_mask, soil_wrapper_mask, soil_mode7_descriptor_bytes: counts[15], firaga_transfer_descriptor_bytes: counts[16], star_cross_descriptor_bytes: [counts[17], counts[18], counts[21]], star_cross_wrapper_bytes: [counts[19], counts[20]] }
    }
    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
