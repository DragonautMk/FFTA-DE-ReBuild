//! Immutable XY payloads; mode, signedness, RNG and null gates stay in original code.
use super::super::{encounter_coordinates as model, encounter_headers, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EncounterCoordinatesStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub placement0_bytes: u64,
    pub random_bytes: u64,
    pub placement2_bytes: u64,
    pub default_bytes: u64,
    pub x_bytes: u64,
    pub y_bytes: u64,
    pub lists_used: u64,
    pub headers_used: u64,
}
pub(super) struct EncounterCoordinatesResources {
    data: model::EncounterCoordinates,
    reads: AtomicU64,
    bytes: AtomicU64,
    roles: [AtomicU64; 4],
    xy: [AtomicU64; 2],
    lists: [AtomicU64; 11],
    headers: [AtomicU64; 7],
}
impl EncounterCoordinatesResources {
    pub(super) fn source_page(index: usize) -> bool {
        (0x70b..=0x70c).contains(&(index >> 12))
    }
    pub(super) fn new(
        data: model::EncounterCoordinates,
        sources: &[SourceRange],
        headers: &encounter_headers::EncounterHeaders,
    ) -> Result<Self, String> {
        data.validate_headers(headers)?;
        let spans = data.source_spans()?;
        if sources.len() != spans.len() {
            return Err("Coordinate binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Coordinate typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            roles: std::array::from_fn(|_| AtomicU64::new(0)),
            xy: std::array::from_fn(|_| AtomicU64::new(0)),
            lists: std::array::from_fn(|_| AtomicU64::new(0)),
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
        let mut roles = [0u64; 4];
        let mut xy = [0u64; 2];
        let mut lists = [0u64; 11];
        let mut headers = [0u64; 7];
        let bytes = std::array::from_fn(|i| match values[i] {
            Some(byte) => {
                let at = (index + i) as u32;
                let list = self.data.lists.partition_point(|l| l.source_offset <= at) - 1;
                let l = &self.data.lists[list];
                lists[list / 64] |= 1 << (list % 64);
                let header = usize::from(l.header_index) - 1;
                headers[header / 64] |= 1 << (header % 64);
                roles[usize::from(l.slot)] += 1;
                xy[((at - l.source_offset) % 2) as usize] += 1;
                byte
            }
            None => prior.as_ref().map_or_else(|| rom[index + i], |p| p[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(roles.iter().sum(), Ordering::Relaxed);
        for (counter, count) in self.roles.iter().zip(roles) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, count) in self.xy.iter().zip(xy) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        for (counter, mask) in self.lists.iter().zip(lists) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        for (counter, mask) in self.headers.iter().zip(headers) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> EncounterCoordinatesStats {
        let [placement0_bytes, random_bytes, placement2_bytes, default_bytes] =
            self.roles.each_ref().map(|v| v.load(Ordering::Relaxed));
        let [x_bytes, y_bytes] = self.xy.each_ref().map(|v| v.load(Ordering::Relaxed));
        EncounterCoordinatesStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            placement0_bytes,
            random_bytes,
            placement2_bytes,
            default_bytes,
            x_bytes,
            y_bytes,
            lists_used: self
                .lists
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
            + self.data.lists.capacity() * std::mem::size_of::<model::CoordinateList>()
            + self
                .data
                .lists
                .iter()
                .map(|l| l.points.capacity() * std::mem::size_of::<model::Point>())
                .sum::<usize>()
    }
}
