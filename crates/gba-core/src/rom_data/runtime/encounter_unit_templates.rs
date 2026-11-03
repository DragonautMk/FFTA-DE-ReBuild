//! Immutable template bytes; shared constructor records retain their existing owner.
use super::super::{
    constructor_alternates, encounter_headers, encounter_unit_templates as model, items,
    metadata14, metadata52, SourceRange,
};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EncounterUnitTemplatesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub selector_bytes: u64,
    pub argument_bytes: u64,
    pub equipment_bytes: u64,
    pub auxiliary_bytes: u64,
    pub ability_bit_bytes: u64,
    pub optional_bytes: u64,
    pub flag_bytes: u64,
    pub control_bytes: u64,
    pub opaque_bytes: u64,
    pub records_used: u64,
    pub headers_used: u64,
}
pub(super) struct EncounterUnitTemplatesResources {
    data: model::EncounterUnitTemplates,
    reads: AtomicU64,
    bytes: AtomicU64,
    fields: [AtomicU64; 9],
    records: [AtomicU64; 45],
    headers: [AtomicU64; 7],
}
impl EncounterUnitTemplatesResources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x6ea..=0x70b).contains(&(index >> 12))
    }
    pub(super) fn new(
        data: model::EncounterUnitTemplates,
        sources: &[SourceRange],
        headers: &encounter_headers::EncounterHeaders,
        old: &constructor_alternates::Alternates,
        item_owner: &items::Items,
        descriptors: &metadata14::Metadata14,
        jobs: &metadata52::Metadata52,
    ) -> Result<Self, String> {
        data.validate_headers(headers)?;
        data.validate_constructor_alternates(old)?;
        data.validate_references(item_owner, descriptors, jobs)?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("Encounter unit-template binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Encounter unit-template typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
            records: std::array::from_fn(|_| AtomicU64::new(0)),
            headers: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut fields = [0u64; 9];
        let mut records = [0u64; 45];
        let mut headers = [0u64; 7];
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let relative = index + i - model::SOURCE as usize;
                let record = relative / model::RECORD_BYTES;
                records[record / 64] |= 1 << (record % 64);
                let header = usize::from(self.data.records[record].header_index) - 1;
                headers[header / 64] |= 1 << (header % 64);
                let class = match relative % model::RECORD_BYTES {
                    0..=4 => 0,
                    5..=7 => 1,
                    8..=17 => 2,
                    18..=19 => 3,
                    20..=39 => 4,
                    40..=41 => 5,
                    42..=43 => 6,
                    44..=45 => 7,
                    _ => 8,
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
        for (counter, mask) in self.records.iter().zip(records) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        for (counter, mask) in self.headers.iter().zip(headers) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> EncounterUnitTemplatesStats {
        let [selector_bytes, argument_bytes, equipment_bytes, auxiliary_bytes, ability_bit_bytes, optional_bytes, flag_bytes, control_bytes, opaque_bytes] =
            self.fields.each_ref().map(|v| v.load(Ordering::Relaxed));
        EncounterUnitTemplatesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            selector_bytes,
            argument_bytes,
            equipment_bytes,
            auxiliary_bytes,
            ability_bit_bytes,
            optional_bytes,
            flag_bytes,
            control_bytes,
            opaque_bytes,
            records_used: self
                .records
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
            headers_used: self
                .headers
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.records.capacity() * std::mem::size_of::<model::Record>()
    }
}
