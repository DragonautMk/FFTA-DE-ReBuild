//! Typed direct indexed archive; the original guest decoder remains active.
use super::super::{direct_indexed_graphics as model, indexed_graphics, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectIndexedGraphicsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub literal_bytes: u64,
    pub header_bytes: u64,
    pub index_bytes: u64,
    pub token_bytes: u64,
    pub decoded_payload_bytes: u64,
    pub dictionary_header_bytes: u64,
}

pub(super) struct DirectIndexedGraphicsResources {
    data: model::DirectIndexedGraphics,
    reads: AtomicU64,
    fields: [AtomicU64; 6],
}

impl DirectIndexedGraphicsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x023 | 0x3ae)
    }

    pub(super) fn new(
        data: model::DirectIndexedGraphics,
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
            return Err("Direct indexed graphics typed source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let literal = self.data.caller_literal_offset as usize;
        let archive = &self.data.archive;
        if (literal..literal + 4).contains(&index) {
            return Some((
                (0x0800_0000u32 + archive.source_offset).to_le_bytes()[index - literal],
                0,
            ));
        }
        let relative = index.checked_sub(archive.source_offset as usize)?;
        if relative < 8 {
            let count = (archive.programs.len() as u16).to_be_bytes();
            let dict = archive.dictionary_relative_offset.to_be_bytes();
            let header = [
                archive.opaque_header[0],
                archive.opaque_header[1],
                count[0],
                count[1],
                dict[0],
                dict[1],
                dict[2],
                dict[3],
            ];
            return Some((header[relative], 1));
        }
        if relative < 24 {
            let n = relative - 8;
            return Some((archive.entry_offsets[n / 4].to_be_bytes()[n % 4], 2));
        }
        if relative < 0x11c {
            let n = relative - 24;
            let program = &archive.programs[n / 65];
            let byte = n % 65;
            if byte == 0 {
                return Some((0xbf, 3));
            }
            let payload = byte - 1;
            return Some((
                program.decoded_words[payload / 4].to_le_bytes()[payload % 4],
                4,
            ));
        }
        if relative < model::ARCHIVE_LENGTH {
            let size = archive.fixed_decoded_bytes.to_be_bytes();
            let header = [
                archive.dictionary_mode,
                archive.dictionary_length_bits,
                size[0],
                size[1],
            ];
            return Some((header[relative - 0x11c], 5));
        }
        None
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [0u64; 6];
        let values = std::array::from_fn(|i| match fields[i] {
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

    pub(super) fn stats(&self) -> DirectIndexedGraphicsStats {
        let [literal_bytes, header_bytes, index_bytes, token_bytes, decoded_payload_bytes, dictionary_header_bytes] =
            self.fields
                .each_ref()
                .map(|counter| counter.load(Ordering::Relaxed));
        DirectIndexedGraphicsStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: literal_bytes
                + header_bytes
                + index_bytes
                + token_bytes
                + decoded_payload_bytes
                + dictionary_header_bytes,
            literal_bytes,
            header_bytes,
            index_bytes,
            token_bytes,
            decoded_payload_bytes,
            dictionary_header_bytes,
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        let a = &self.data.archive;
        std::mem::size_of::<Self>()
            + a.entry_offsets.capacity() * std::mem::size_of::<u32>()
            + a.dictionary_bytes.capacity()
            + a.programs.capacity() * std::mem::size_of::<indexed_graphics::Program>()
            + a.programs
                .iter()
                .map(|p| {
                    p.decoded_words.capacity() * std::mem::size_of::<u32>()
                        + p.packets.capacity() * std::mem::size_of::<indexed_graphics::Packet>()
                        + p.packets
                            .iter()
                            .map(|packet| match packet {
                                indexed_graphics::Packet::Literal { bytes } => bytes.capacity(),
                                indexed_graphics::Packet::Dictionary { .. } => 0,
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
