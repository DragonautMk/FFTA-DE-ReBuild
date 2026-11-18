//! Typed delivery of an opaque conditional language-reader source pair.
use super::super::{conditional_language_pair as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConditionalLanguagePairStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
}

pub(super) struct ConditionalLanguagePairResources {
    data: model::ConditionalLanguagePair,
    reads: AtomicU64,
    bytes: AtomicU64,
}

impl ConditionalLanguagePairResources {
    pub(super) fn source_page(index: usize) -> bool {
        index >> 12 == 0x3ae
    }

    pub(super) fn new(
        data: model::ConditionalLanguagePair,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len()
            || spans
                .iter()
                .zip(sources)
                .any(|((offset, bytes, role), source)| {
                    source.offset != *offset
                        || source.length as usize != bytes.len()
                        || source.role != *role
                        || source.sha256 != ffta_rom::digest(bytes)
                })
        {
            return Err("Conditional language pair source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        })
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let start = self.data.source_offset as usize;
        let mut used = 0u64;
        let values = std::array::from_fn(|i| {
            if let Some(byte) = (index + i)
                .checked_sub(start)
                .and_then(|relative| self.data.opaque_bytes.get(relative))
            {
                used += 1;
                *byte
            } else {
                prior
                    .as_ref()
                    .map_or_else(|| rom[index + i], |bytes| bytes[i])
            }
        });
        if used == 0 {
            return prior;
        }
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(used, Ordering::Relaxed);
        Some(values)
    }

    pub(super) fn stats(&self) -> ConditionalLanguagePairStats {
        ConditionalLanguagePairStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: self.bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
