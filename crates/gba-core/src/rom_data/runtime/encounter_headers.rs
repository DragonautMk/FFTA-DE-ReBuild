//! Immutable encounter-header bytes. Shared constructor bytes keep their prior owner.
use super::super::{constructor_alternates, encounter_headers as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EncounterHeadersStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub unit_count_bytes: u64,
    pub map_bytes: u64,
    pub template_pointer_bytes: u64,
    pub placement_parameter_bytes: u64,
    pub placement_count_bytes: u64,
    pub position_pointer_bytes: u64,
    pub control_bytes: u64,
    pub opaque_bytes: u64,
    pub headers_used: u64,
}
pub(super) struct EncounterHeadersResources {
    data: model::EncounterHeaders,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 8],
    used: [AtomicU64; 7],
}
impl EncounterHeadersResources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x70c..=0x710).contains(&(index >> 12))
    }
    pub(super) fn new(
        data: model::EncounterHeaders,
        sources: &[SourceRange],
        owner: &constructor_alternates::Alternates,
    ) -> Result<Self, String> {
        data.validate_constructor_alternates(owner)?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("Encounter-header source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Encounter-header typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
            used: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let values: [_; N] = std::array::from_fn(|i| {
            u32::try_from(index + i)
                .ok()
                .and_then(|at| self.data.byte_at(at))
        });
        if values.iter().all(Option::is_none) {
            return prior;
        }
        let mut fields = [0u64; 8];
        let mut used = [0u64; 7];
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let relative = index + i - model::SOURCE as usize;
                let header = relative / model::HEADER_BYTES;
                used[header / 64] |= 1 << (header % 64);
                let class = match relative % model::HEADER_BYTES {
                    0..=1 => 0,
                    2 => 1,
                    4..=7 => 2,
                    8 | 10 => 3,
                    12..=14 => 4,
                    16..=31 => 5,
                    32..=34 | 36 | 38 => 6,
                    _ => 7,
                };
                fields[class] += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(fields.iter().sum(), Ordering::Relaxed);
        for (counter, count) in self.fields.iter().zip(fields) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, mask) in self.used.iter().zip(used) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> EncounterHeadersStats {
        let [unit_count_bytes, map_bytes, template_pointer_bytes, placement_parameter_bytes, placement_count_bytes, position_pointer_bytes, control_bytes, opaque_bytes] =
            self.fields.each_ref().map(|a| a.load(Ordering::Relaxed));
        EncounterHeadersStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            unit_count_bytes,
            map_bytes,
            template_pointer_bytes,
            placement_parameter_bytes,
            placement_count_bytes,
            position_pointer_bytes,
            control_bytes,
            opaque_bytes,
            headers_used: self
                .used
                .iter()
                .map(|a| u64::from(a.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.headers.capacity() * std::mem::size_of::<model::Header>()
    }
}
