//! Project authenticated Thumb literal operands without replacing guest code.
use super::*;
use crate::rom_data::thumb_pc_literals as model;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThumbPcLiteralsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub literals_mask: [u64; 6],
    pub additional_source_bytes: u64,
    pub additional_mask: [u64; 2],
    pub observed_source_bytes: u64,
    pub observed_mask: [u64; 7],
    pub home_anchored_bytes: u64,
    pub home_anchored_mask: u64,
    pub home_observed_bytes: u64,
    pub home_observed_mask: u64,
}

pub(super) struct ThumbPcLiteralsResources {
    data: model::ThumbPcLiterals,
    reads: AtomicU64,
    bytes: AtomicU64,
    literals: [AtomicU64; 6],
    additional_bytes: AtomicU64,
    additional: [AtomicU64; 2],
    observed_bytes: AtomicU64,
    observed: [AtomicU64; 7],
    home_anchored_bytes: AtomicU64,
    home_anchored: AtomicU64,
    home_observed_bytes: AtomicU64,
    home_observed: AtomicU64,
}

impl ThumbPcLiteralsResources {
    pub(super) fn source_page(index: usize) -> bool {
        let first = index >> 12;
        let last = index.saturating_add(3) >> 12;
        model::SOURCE_PAGES.binary_search(&first).is_ok()
            || model::SOURCE_PAGES.binary_search(&last).is_ok()
    }

    pub(super) fn new(
        data: model::ThumbPcLiterals,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        data.validate()?;
        if sources.len()
            != model::LITERAL_COUNT
                + model::ADDITIONAL_COUNT
                + model::OBSERVED_COUNT
                + model::HOME_ANCHORED_COUNT
                + model::HOME_OBSERVED_COUNT
        {
            return Err("Thumb literal source count mismatch".into());
        }
        for (literal, source) in data
            .literals
            .iter()
            .chain(&data.additional)
            .chain(&data.observed)
            .chain(&data.home_anchored)
            .chain(&data.home_observed)
            .zip(sources)
        {
            if source.offset != literal.source_offset
                || source.length != 4
                || source.role != model::SOURCE_ROLE
                || source.sha256 != ffta_rom::digest(&literal.value.to_le_bytes())
            {
                return Err("Thumb literal source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            literals: std::array::from_fn(|_| AtomicU64::new(0)),
            additional_bytes: AtomicU64::new(0),
            additional: std::array::from_fn(|_| AtomicU64::new(0)),
            observed_bytes: AtomicU64::new(0),
            observed: std::array::from_fn(|_| AtomicU64::new(0)),
            home_anchored_bytes: AtomicU64::new(0),
            home_anchored: AtomicU64::new(0),
            home_observed_bytes: AtomicU64::new(0),
            home_observed: AtomicU64::new(0),
        })
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.data.byte_at((index + i) as u32));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let mut count = 0;
        let mut literals = [0u64; 6];
        let mut additional_count = 0;
        let mut additional = [0u64; 2];
        let mut observed_count = 0;
        let mut observed = [0u64; 7];
        let mut home_count = 0;
        let mut home = 0u64;
        let mut home_observed_count = 0;
        let mut home_observed = 0u64;
        let result = std::array::from_fn(|i| match cells[i] {
            Some((byte, word)) => {
                count += 1;
                if word < model::LITERAL_COUNT {
                    literals[word / 64] |= 1u64 << (word % 64);
                } else if word < model::LITERAL_COUNT + model::ADDITIONAL_COUNT {
                    let word = word - model::LITERAL_COUNT;
                    additional_count += 1;
                    additional[word / 64] |= 1u64 << (word % 64);
                } else if word
                    < model::LITERAL_COUNT + model::ADDITIONAL_COUNT + model::OBSERVED_COUNT
                {
                    let word = word - model::LITERAL_COUNT - model::ADDITIONAL_COUNT;
                    observed_count += 1;
                    observed[word / 64] |= 1u64 << (word % 64);
                } else if word
                    < model::LITERAL_COUNT
                        + model::ADDITIONAL_COUNT
                        + model::OBSERVED_COUNT
                        + model::HOME_ANCHORED_COUNT
                {
                    let word = word
                        - model::LITERAL_COUNT
                        - model::ADDITIONAL_COUNT
                        - model::OBSERVED_COUNT;
                    home_count += 1;
                    home |= 1u64 << word;
                } else {
                    let word = word
                        - model::LITERAL_COUNT
                        - model::ADDITIONAL_COUNT
                        - model::OBSERVED_COUNT
                        - model::HOME_ANCHORED_COUNT;
                    home_observed_count += 1;
                    home_observed |= 1u64 << word;
                }
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |value| value[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count, Ordering::Relaxed);
        for (counter, mask) in self.literals.iter().zip(literals) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.additional_bytes
            .fetch_add(additional_count, Ordering::Relaxed);
        for (counter, mask) in self.additional.iter().zip(additional) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.observed_bytes
            .fetch_add(observed_count, Ordering::Relaxed);
        for (counter, mask) in self.observed.iter().zip(observed) {
            counter.fetch_or(mask, Ordering::Relaxed);
        }
        self.home_anchored_bytes
            .fetch_add(home_count, Ordering::Relaxed);
        self.home_anchored.fetch_or(home, Ordering::Relaxed);
        self.home_observed_bytes
            .fetch_add(home_observed_count, Ordering::Relaxed);
        self.home_observed
            .fetch_or(home_observed, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> ThumbPcLiteralsStats {
        ThumbPcLiteralsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
            literals_mask: self.literals.each_ref().map(|v| v.load(Ordering::Relaxed)),
            additional_source_bytes: self.additional_bytes.load(Ordering::Relaxed),
            additional_mask: self
                .additional
                .each_ref()
                .map(|v| v.load(Ordering::Relaxed)),
            observed_source_bytes: self.observed_bytes.load(Ordering::Relaxed),
            observed_mask: self.observed.each_ref().map(|v| v.load(Ordering::Relaxed)),
            home_anchored_bytes: self.home_anchored_bytes.load(Ordering::Relaxed),
            home_anchored_mask: self.home_anchored.load(Ordering::Relaxed),
            home_observed_bytes: self.home_observed_bytes.load(Ordering::Relaxed),
            home_observed_mask: self.home_observed.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.literals.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.additional.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.observed.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.home_anchored.capacity() * std::mem::size_of::<model::LiteralWord>()
            + self.data.home_observed.capacity() * std::mem::size_of::<model::LiteralWord>()
    }
}
