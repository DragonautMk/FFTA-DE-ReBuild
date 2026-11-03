//! File-derived static OBJ fields on guest bus and host descriptor paths.
use super::*;
use crate::rom_data::obj_lists::{List, Lists, Object};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjListStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub count_bytes: u64,
    pub attribute_bytes: u64,
    pub host_reads: u64,
}

pub(super) struct ObjListResources {
    lists: Lists,
    counts: [AtomicU64; 4],
    host_reads: AtomicU64,
    used: Vec<[AtomicBool; 2]>,
}
impl ObjListResources {
    pub(super) fn new(lists: Lists) -> Result<Self, String> {
        lists.validate()?;
        Ok(Self {
            used: (0..lists.lists.len())
                .map(|_| std::array::from_fn(|_| AtomicBool::new(false)))
                .collect(),
            lists,
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            host_reads: AtomicU64::new(0),
        })
    }

    pub(super) fn source_page(index: usize) -> bool {
        matches!(
            index >> 12,
            0x394 | 0x395 | 0x397 | 0x399 | 0x3ad | 0x3ae | 0x40d | 0x6df
        )
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let i = self
            .lists
            .lists
            .partition_point(|l| l.source_offset as usize <= index)
            .checked_sub(1)?;
        let list = &self.lists.lists[i];
        let at = index - list.source_offset as usize;
        let byte = list.source_byte(at)?;
        self.used[i][0].store(true, Ordering::Relaxed);
        Some((byte, if at < 2 { 2 } else { 3 }))
    }

    pub(super) fn host_list(&self, offset: u32) -> Option<&List> {
        let index = self
            .lists
            .lists
            .binary_search_by_key(&offset, |l| l.source_offset)
            .ok()?;
        self.host_reads.fetch_add(1, Ordering::Relaxed);
        self.used[index][1].store(true, Ordering::Relaxed);
        Some(&self.lists.lists[index])
    }

    pub(super) fn source_used(&self, offset: u32) -> Option<(bool, bool)> {
        let index = self
            .lists
            .lists
            .binary_search_by_key(&offset, |l| l.source_offset)
            .ok()?;
        Some((
            self.used[index][0].load(Ordering::Relaxed),
            self.used[index][1].load(Ordering::Relaxed),
        ))
    }

    pub(super) fn used_sources(&self) -> Vec<(u32, bool, bool)> {
        self.lists
            .lists
            .iter()
            .zip(&self.used)
            .filter_map(|(list, used)| {
                let guest = used[0].load(Ordering::Relaxed);
                let host = used[1].load(Ordering::Relaxed);
                (guest || host).then_some((list.source_offset, guest, host))
            })
            .collect()
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
        let mut counts = [0; 4];
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

    pub(super) fn stats(&self) -> ObjListStats {
        let [bus_reads, source_bytes, count_bytes, attribute_bytes] =
            self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        ObjListStats {
            bus_reads,
            source_bytes,
            count_bytes,
            attribute_bytes,
            host_reads: self.host_reads.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.used.capacity() * std::mem::size_of::<[AtomicBool; 2]>()
            + self.lists.lists.capacity() * std::mem::size_of::<List>()
            + self
                .lists
                .lists
                .iter()
                .map(|l| l.objects.capacity() * std::mem::size_of::<Object>())
                .sum::<usize>()
    }
}
