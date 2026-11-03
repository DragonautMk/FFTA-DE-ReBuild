//! Typed raw transfer words serve ROM reads; original copy queues and clocks remain active.
use super::super::{SourceRange, ui_raw_transfers as model};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RawTransfersStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub payload_bytes: u64,
    pub banks_used: u64,
    pub groups_used: u64,
}
struct Byte {
    value: u8,
    bank: Option<usize>,
    group: u8,
}
pub(super) struct RawTransfersResources {
    data: model::RawTransfers,
    bank_groups: [u8; 35],
    reads: AtomicU64,
    bytes: AtomicU64,
    selectors: AtomicU64,
    payload: AtomicU64,
    banks: AtomicU64,
    groups: AtomicU64,
}
impl RawTransfersResources {
    pub(super) fn source_page(index: usize) -> bool {
        SOURCE_BOUNDS.iter().any(|&(start, length)| {
            index >> 12 >= start >> 12 && index >> 12 <= (start + length - 1) >> 12
        })
    }
    pub(super) fn new(data: model::RawTransfers, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Raw transfer binding count mismatch".into());
        }
        for ((at, bytes, role), source) in spans.iter().zip(sources) {
            if *at != source.offset
                || bytes.len() != source.length as usize
                || *role != source.role
                || ffta_rom::digest(bytes) != source.sha256
            {
                return Err("Raw transfer typed source binding mismatch".into());
            }
        }
        let mut bank_groups = [0; 35];
        for (i, bank) in data.banks.iter().enumerate() {
            bank_groups[i] = data
                .tables
                .iter()
                .position(|table| {
                    table
                        .pointers
                        .iter()
                        .any(|p| p.bank_offset == bank.bank_offset)
                })
                .ok_or("Raw transfer bank lacks group")? as u8;
        }
        Ok(Self {
            data,
            bank_groups,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            selectors: AtomicU64::new(0),
            payload: AtomicU64::new(0),
            banks: AtomicU64::new(0),
            groups: AtomicU64::new(0),
        })
    }
    fn byte(&self, index: usize) -> Option<Byte> {
        let offset = u32::try_from(index).ok()?;
        for (group, table) in self.data.tables.iter().enumerate() {
            if let Some(relative) = offset.checked_sub(table.source_offset).filter(|r| *r < 20) {
                let value = table.pointers[relative as usize / 4].source_bytes().ok()?
                    [relative as usize % 4];
                return Some(Byte {
                    value,
                    bank: None,
                    group: group as u8,
                });
            }
        }
        self.data.banks.iter().enumerate().find_map(|(bank, view)| {
            view.byte_at(offset).map(|value| Byte {
                value,
                bank: Some(bank),
                group: self.bank_groups[bank],
            })
        })
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
        let values: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if values.iter().all(Option::is_none) {
            return prior;
        }
        let (mut selector, mut payload, mut banks, mut groups) = (0, 0, 0, 0);
        let result = std::array::from_fn(|i| match &values[i] {
            Some(cell) => {
                if let Some(bank) = cell.bank {
                    payload += 1;
                    banks |= 1u64 << bank;
                } else {
                    selector += 1;
                }
                groups |= 1u64 << cell.group;
                cell.value
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(selector + payload, Ordering::Relaxed);
        self.selectors.fetch_add(selector, Ordering::Relaxed);
        self.payload.fetch_add(payload, Ordering::Relaxed);
        self.banks.fetch_or(banks, Ordering::Relaxed);
        self.groups.fetch_or(groups, Ordering::Relaxed);
        Some(result)
    }
    pub(super) fn stats(&self) -> RawTransfersStats {
        RawTransfersStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            selector_bytes: self.selectors.load(Ordering::Relaxed),
            payload_bytes: self.payload.load(Ordering::Relaxed),
            banks_used: u64::from(self.banks.load(Ordering::Relaxed).count_ones()),
            groups_used: u64::from(self.groups.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        self.data
            .banks
            .iter()
            .map(|b| b.words.len() * 4 + std::mem::size_of::<model::BankView>())
            .sum::<usize>()
            + self.data.tables.len()
                * std::mem::size_of::<super::super::ui_graphics::table_selected::Table>()
    }
}
const SOURCE_BOUNDS: &[(usize, usize)] = &[
    (0x431470, 160),
    (0x4386c0, 1600),
    (0x439190, 768),
    (0x43ce28, 224),
    (0x43cf4c, 2048),
    (0x4408f4, 2432),
    (0x441488, 384),
    (0x442504, 160),
    (0x4498c0, 1600),
    (0x44a390, 768),
    (0x44e430, 224),
    (0x44e554, 2048),
    (0x451f68, 2432),
    (0x452afc, 384),
    (0x453e1c, 160),
    (0x45b4b0, 1600),
    (0x45bf80, 768),
    (0x45ff7c, 224),
    (0x4600a0, 2048),
    (0x463b04, 2432),
    (0x464698, 384),
    (0x465560, 160),
    (0x46c74c, 1600),
    (0x46d21c, 768),
    (0x470fd4, 224),
    (0x4710f8, 2048),
    (0x474acc, 2432),
    (0x475660, 384),
    (0x4769e0, 160),
    (0x47ddf4, 1600),
    (0x47e8c4, 768),
    (0x482ab8, 224),
    (0x482bdc, 2048),
    (0x48659c, 2432),
    (0x487130, 384),
    (0x71e5f4, 20),
    (0x71e608, 20),
    (0x71e66c, 20),
    (0x71e694, 20),
    (0x71e6a8, 20),
    (0x71e6e4, 20),
    (0x71e6f8, 20),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DynamicFragmentsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub payload_bytes: u64,
}

/// Independent bounded owner; missing bytes preserve earlier owners and raw ROM.
pub(super) struct DynamicFragmentsResources {
    data: model::dynamic_fragments::Fragments,
    counters: [AtomicU64; 4],
}
impl DynamicFragmentsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x399 | 0x441 | 0x452 | 0x464 | 0x475 | 0x487)
    }
    pub(super) fn new(data: model::dynamic_fragments::Fragments, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() || spans.iter().zip(sources).any(|((at, bytes, role), source)|
            *at != source.offset || bytes.len() != source.length as usize || *role != source.role || ffta_rom::digest(bytes) != source.sha256) {
            return Err("Dynamic fragments typed source binding mismatch".into());
        }
        Ok(Self { data, counters: std::array::from_fn(|_| AtomicU64::new(0)) })
    }
    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        if !Self::source_page(index) { return prior; }
        let values: [Option<u8>; N] = std::array::from_fn(|i| index.checked_add(i).and_then(|at| u32::try_from(at).ok()).and_then(|at| self.data.byte_at(at)));
        if values.iter().all(Option::is_none) { return prior; }
        let mut selector_bytes = 0u64;
        let mut payload_bytes = 0u64;
        for (i, value) in values.iter().enumerate() {
            if value.is_some() {
                let at = index + i;
                if self.data.rows.iter().any(|row| (row.source_offset as usize..row.source_offset as usize + 12).contains(&at)) {
                    selector_bytes += 1;
                } else {
                    payload_bytes += 1;
                }
            }
        }
        for (counter, count) in self.counters.iter().zip([1, selector_bytes + payload_bytes, selector_bytes, payload_bytes]) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(std::array::from_fn(|i| values[i].unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]))))
    }
    pub(super) fn stats(&self) -> DynamicFragmentsStats {
        let [bus_reads, source_bytes, selector_bytes, payload_bytes] = self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        DynamicFragmentsStats { bus_reads, source_bytes, selector_bytes, payload_bytes }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        self.data.rows.len() * std::mem::size_of::<model::dynamic_fragments::Rows>() +
        self.data.banks.iter().map(|b| std::mem::size_of::<model::BankView>() + b.words.len() * 4).sum::<usize>()
    }
}
