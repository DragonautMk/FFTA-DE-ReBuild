//! Typed context callback descriptor; guest dispatch remains unchanged.
use super::super::{context_callback_descriptor as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContextCallbackDescriptorStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub constructor_literal_bytes: u64,
    pub callback_bytes: u64,
    pub delay_bytes: u64,
    pub flags_bytes: u64,
    pub opaque_bytes: u64,
}
pub(super) struct ContextCallbackDescriptorResources {
    data: model::ContextCallbackDescriptor,
    reads: AtomicU64,
    fields: [AtomicU64; 5],
}
impl ContextCallbackDescriptorResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x144 | 0x3ae)
    }
    pub(super) fn new(
        data: model::ContextCallbackDescriptor,
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
            return Err("Context callback typed source binding mismatch".into());
        }
        Ok(Self {
            data,
            reads: AtomicU64::new(0),
            fields: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let literal = self.data.constructor_literal_offset as usize;
        if (literal..literal + 4).contains(&index) {
            return Some((
                (0x0800_0000 + self.data.descriptor_offset).to_le_bytes()[index - literal],
                0,
            ));
        }
        let relative = index.checked_sub(self.data.descriptor_offset as usize)?;
        match relative {
            0..=3 => Some((self.data.callback_thumb_pointer.to_le_bytes()[relative], 1)),
            4..=5 => Some((self.data.signed_delay.to_le_bytes()[relative - 4], 2)),
            6 => Some((self.data.flags, 3)),
            7 => Some((self.data.opaque_byte, 4)),
            _ => None,
        }
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
        let mut counts = [0u64; 5];
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
    pub(super) fn stats(&self) -> ContextCallbackDescriptorStats {
        let [constructor_literal_bytes, callback_bytes, delay_bytes, flags_bytes, opaque_bytes] =
            self.fields
                .each_ref()
                .map(|counter| counter.load(Ordering::Relaxed));
        ContextCallbackDescriptorStats {
            bus_reads: self.reads.load(Ordering::Relaxed),
            source_bytes: constructor_literal_bytes
                + callback_bytes
                + delay_bytes
                + flags_bytes
                + opaque_bytes,
            constructor_literal_bytes,
            callback_bytes,
            delay_bytes,
            flags_bytes,
            opaque_bytes,
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
