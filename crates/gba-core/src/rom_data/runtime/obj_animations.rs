//! Immutable typed indices, OBJ fields, frame selectors and duration words.
use super::*;
use crate::rom_data::{
    obj_animations::{Bank, Banks, Frame, SelectorTables, Sequence},
    obj_lists::{List, Object},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjAnimationStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub index_bytes: u64,
    pub count_bytes: u64,
    pub attribute_bytes: u64,
    pub sequence_header_bytes: u64,
    pub frame_list_bytes: u64,
    pub duration_bytes: u64,
    pub host_reads: u64,
    pub zero_count_selector_bytes: u64,
    pub zero_count_host_reads: u64,
    pub localized_selector_bytes: u64,
    pub localized_source_bytes: u64,
    pub localized_host_reads: u64,
}
pub(super) struct ObjAnimationResources {
    banks: Banks,
    counts: [AtomicU64; 8],
    host_reads: AtomicU64,
    zero_count_selector_bytes: AtomicU64,
    zero_count_host_reads: AtomicU64,
    localized_counts: [AtomicU64; 3],
}
impl ObjAnimationResources {
    pub(super) fn new(banks: Banks) -> Result<Self, String> {
        banks.validate()?;
        Ok(Self {
            banks,
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            host_reads: AtomicU64::new(0),
            zero_count_selector_bytes: AtomicU64::new(0),
            zero_count_host_reads: AtomicU64::new(0),
            localized_counts: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x395 | 0xb4b..=0xb4f)
    }

    fn localized(&self, bank: &Bank) -> bool {
        self.banks
            .selector_tables
            .iter()
            .any(|t| t.object_roots.contains(&bank.object_root))
    }
    fn owned_byte(&self, bank: &Bank, byte: u8, kind: usize) -> Option<(u8, usize)> {
        if self.localized(bank) {
            self.localized_counts[1].fetch_add(1, Ordering::Relaxed);
        }
        Some((byte, kind))
    }

    pub(super) fn host_list(&self, offset: u32) -> Option<&List> {
        for bank in &self.banks.banks {
            if let Ok(index) = bank
                .lists
                .binary_search_by_key(&offset, |l| l.source_offset)
            {
                self.host_reads.fetch_add(1, Ordering::Relaxed);
                if self.localized(bank) {
                    self.localized_counts[2].fetch_add(1, Ordering::Relaxed);
                }
                if bank.sequences.iter().any(|s| s.zero_count_hold.is_some()) {
                    self.zero_count_host_reads.fetch_add(1, Ordering::Relaxed);
                }
                return Some(&bank.lists[index]);
            }
        }
        None
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        for tables in &self.banks.selector_tables {
            if let Some(byte) = tables.source_byte(index) {
                self.localized_counts[0].fetch_add(1, Ordering::Relaxed);
                return Some((byte, 2));
            }
        }
        for bank in &self.banks.banks {
            for (root, offsets) in [
                (bank.object_root, &bank.object_offsets),
                (bank.sequence_root, &bank.sequence_offsets),
            ] {
                if let Some(at) = index
                    .checked_sub(root as usize)
                    .filter(|&at| at < offsets.len() * 4)
                {
                    return self.owned_byte(bank, offsets[at / 4].to_le_bytes()[at % 4], 2);
                }
            }
            if let Some(i) = bank
                .lists
                .partition_point(|l| l.source_offset as usize <= index)
                .checked_sub(1)
            {
                let list = &bank.lists[i];
                let at = index - list.source_offset as usize;
                if let Some(byte) = list.source_byte(at) {
                    return self.owned_byte(bank, byte, if at < 2 { 3 } else { 4 });
                }
            }
            if let Some(i) = bank
                .sequences
                .partition_point(|s| s.source_offset as usize <= index)
                .checked_sub(1)
            {
                let sequence = &bank.sequences[i];
                let at = index - sequence.source_offset as usize;
                if at < 2 {
                    return self.owned_byte(
                        bank,
                        (sequence.frames.len() as u16).to_le_bytes()[at],
                        5,
                    );
                }
                if at < 4 {
                    if let Some(hold) = &sequence.zero_count_hold {
                        self.zero_count_selector_bytes
                            .fetch_add(1, Ordering::Relaxed);
                        return self.owned_byte(bank, hold.object_list.to_le_bytes()[at - 2], 6);
                    }
                }
                if let Some(frame) = sequence.frames.get((at - 2) / 4) {
                    let at = (at - 2) % 4;
                    return self.owned_byte(
                        bank,
                        if at < 2 {
                            frame.object_list.to_le_bytes()[at]
                        } else {
                            frame.duration.to_le_bytes()[at - 2]
                        },
                        if at < 2 { 6 } else { 7 },
                    );
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
        if !Self::source_page(index) {
            return prior;
        }
        self.guest_read_projected(index, rom, prior)
    }

    /// Shared immutable field projection. Each owner applies its own page gate.
    pub(super) fn guest_read_projected<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let mut counts = [0; 8];
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, kind)) => {
                counts[1] += 1;
                counts[kind] += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        if counts[1] == 0 {
            return prior;
        }
        counts[0] = 1;
        for (counter, n) in self.counts.iter().zip(counts) {
            counter.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ObjAnimationStats {
        let [bus_reads, source_bytes, index_bytes, count_bytes, attribute_bytes, sequence_header_bytes, frame_list_bytes, duration_bytes] =
            self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        ObjAnimationStats {
            bus_reads,
            source_bytes,
            index_bytes,
            count_bytes,
            attribute_bytes,
            sequence_header_bytes,
            frame_list_bytes,
            duration_bytes,
            host_reads: self.host_reads.load(Ordering::Relaxed),
            zero_count_selector_bytes: self.zero_count_selector_bytes.load(Ordering::Relaxed),
            zero_count_host_reads: self.zero_count_host_reads.load(Ordering::Relaxed),
            localized_selector_bytes: self.localized_counts[0].load(Ordering::Relaxed),
            localized_source_bytes: self.localized_counts[1].load(Ordering::Relaxed),
            localized_host_reads: self.localized_counts[2].load(Ordering::Relaxed),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.banks.banks.capacity() * std::mem::size_of::<Bank>()
            + self.banks.selector_tables.capacity() * std::mem::size_of::<SelectorTables>()
            + self
                .banks
                .selector_tables
                .iter()
                .map(|t| (t.object_roots.capacity() + t.sequence_roots.capacity()) * 4)
                .sum::<usize>()
            + self
                .banks
                .banks
                .iter()
                .map(|b| {
                    (b.object_offsets.capacity() + b.sequence_offsets.capacity()) * 4
                        + b.lists.capacity() * std::mem::size_of::<List>()
                        + b.lists
                            .iter()
                            .map(|l| l.objects.capacity() * std::mem::size_of::<Object>())
                            .sum::<usize>()
                        + b.sequences.capacity() * std::mem::size_of::<Sequence>()
                        + b.sequences
                            .iter()
                            .map(|s| s.frames.capacity() * std::mem::size_of::<Frame>())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
