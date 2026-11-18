//! Source view rebuilt from typed packets; decoding remains original guest work.
use super::super::{direct_indexed_dictionary as model, indexed_graphics, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectIndexedDictionaryStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub literal_pointer_bytes: u64,
    pub header_bytes: u64,
    pub index_bytes: u64,
    pub token_bytes: u64,
    pub payload_bytes: u64,
    pub dictionary_header_bytes: u64,
    pub dictionary_bytes: u64,
}

struct Span {
    offset: usize,
    cells: Vec<(u8, usize)>,
}
pub(super) struct DirectIndexedDictionaryResources {
    spans: Vec<Span>,
    reads: AtomicU64,
    fields: [AtomicU64; 7],
}
impl DirectIndexedDictionaryResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x023 | 0x3ae)
    }

    pub(super) fn new(
        data: model::DirectIndexedDictionary,
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
            return Err("Direct indexed dictionary typed source binding mismatch".into());
        }
        let mut result = Vec::new();
        for (offset, bytes, _) in spans {
            let mut cells: Vec<_> = bytes.into_iter().map(|byte| (byte, usize::MAX)).collect();
            if offset == model::CALLER_LITERAL {
                cells.iter_mut().for_each(|cell| cell.1 = 0);
            } else {
                cells[..8].iter_mut().for_each(|cell| cell.1 = 1);
                let archive = &data.archive;
                cells[8..8 + archive.programs.len() * 4]
                    .iter_mut()
                    .for_each(|cell| cell.1 = 2);
                for program in &archive.programs {
                    let mut at = program.relative_offset as usize;
                    for packet in &program.packets {
                        let token_length = match packet {
                            indexed_graphics::Packet::Literal { .. } => 1,
                            indexed_graphics::Packet::Dictionary { .. } => 2,
                        };
                        cells[at..at + token_length]
                            .iter_mut()
                            .for_each(|cell| cell.1 = 3);
                        at += token_length;
                        if let indexed_graphics::Packet::Literal { bytes } = packet {
                            cells[at..at + bytes.len()]
                                .iter_mut()
                                .for_each(|cell| cell.1 = 4);
                            at += bytes.len();
                        }
                    }
                }
                let dict = archive.dictionary_relative_offset as usize;
                cells[dict..dict + 4].iter_mut().for_each(|cell| cell.1 = 5);
                cells[dict + 4..].iter_mut().for_each(|cell| cell.1 = 6);
            }
            if cells.iter().any(|cell| cell.1 == usize::MAX) {
                return Err("Incomplete direct indexed dictionary source projection".into());
            }
            result.push(Span {
                offset: offset as usize,
                cells,
            });
        }
        Ok(Self {
            spans: result,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        self.spans
            .iter()
            .find_map(|span| span.cells.get(index.checked_sub(span.offset)?).copied())
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let cells: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if cells.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 7];
        let values = std::array::from_fn(|i| match cells[i] {
            Some((value, kind)) => {
                counts[kind] += 1;
                value
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        self.reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.fields.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(values)
    }

    pub(super) fn stats(&self) -> DirectIndexedDictionaryStats {
        let [literal_pointer_bytes, header_bytes, index_bytes, token_bytes, payload_bytes, dictionary_header_bytes, dictionary_bytes] =
            self.fields
                .each_ref()
                .map(|counter| counter.load(Ordering::Relaxed));
        DirectIndexedDictionaryStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: literal_pointer_bytes
                + header_bytes
                + index_bytes
                + token_bytes
                + payload_bytes
                + dictionary_header_bytes
                + dictionary_bytes,
            literal_pointer_bytes,
            header_bytes,
            index_bytes,
            token_bytes,
            payload_bytes,
            dictionary_header_bytes,
            dictionary_bytes,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.spans.capacity() * std::mem::size_of::<Span>()
            + self
                .spans
                .iter()
                .map(|span| span.cells.capacity() * std::mem::size_of::<(u8, usize)>())
                .sum::<usize>()
    }
}
