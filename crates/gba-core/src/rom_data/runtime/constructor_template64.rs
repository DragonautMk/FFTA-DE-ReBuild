//! Overlay only the immutable initial template; the guest owns live records.
use super::super::{constructor_template64 as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConstructorTemplate64Stats {
    pub bus_reads: u64,
    pub source_bytes: u64,
}

pub(super) struct ConstructorTemplate64Resources {
    data: model::ConstructorTemplate64,
    reads: AtomicU64,
    bytes: AtomicU64,
}

impl ConstructorTemplate64Resources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x14d
    }

    pub(super) fn new(data: model::ConstructorTemplate64, sources: &[SourceRange]) -> Result<Self, String> {
        data.validate()?;
        let bytes = data.source_bytes();
        if sources.len() != 1 || sources[0].offset != model::SOURCE_OFFSET
            || sources[0].length as usize != bytes.len()
            || sources[0].role != "constructor-template"
            || sources[0].sha256 != ffta_rom::digest(&bytes)
        {
            return Err("Constructor template source binding mismatch".into());
        }
        Ok(Self { data, reads: AtomicU64::new(0), bytes: AtomicU64::new(0) })
    }

    fn byte(&self, index: usize) -> Option<u8> {
        let n = index.checked_sub(self.data.source_offset as usize)?;
        (n < model::SOURCE_LENGTH).then(|| self.data.words[n / 4].to_le_bytes()[n % 4])
    }

    pub(super) fn guest_read<const N: usize>(&self, index: usize, rom: &[u8], prior: Option<[u8; N]>) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        let count = cells.iter().filter(|cell| cell.is_some()).count();
        if count == 0 { return prior; }
        let result = std::array::from_fn(|i| cells[i].unwrap_or_else(|| prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])));
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
        Some(result)
    }

    pub(super) fn stats(&self) -> ConstructorTemplate64Stats {
        ConstructorTemplate64Stats { bus_reads: self.reads.load(Ordering::Relaxed), source_bytes: self.bytes.load(Ordering::Relaxed) }
    }

    pub(super) fn resident_bytes(&self) -> usize { std::mem::size_of::<Self>() }
}
