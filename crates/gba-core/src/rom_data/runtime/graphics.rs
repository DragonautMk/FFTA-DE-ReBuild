//! Main graphics reads use decoded words for literals and exact codec metadata.
//! Original decompression, scratch writes and staged DMA keep executing normally.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GraphicsStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub decoded_literal_bytes: u64,
}

struct Binding {
    offset: usize,
    length: usize,
    row: usize,
}

pub(super) struct GraphicsResources {
    graphics: bg_graphics::Graphics,
    /// Exact encoded-source position -> decoded word byte; MAX is metadata.
    literals: Vec<Vec<u16>>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    source_bytes: AtomicU64,
    decoded_literal_bytes: AtomicU64,
}

impl GraphicsResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x769..=0x895)
    }

    pub(super) fn new(
        graphics: bg_graphics::Graphics,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        graphics.validate()?;
        if sources.len() != graphics.graphics.len() {
            return Err("Main graphics source count mismatch".into());
        }
        let mut literals = Vec::new();
        let mut bindings = Vec::new();
        for (row, (graphic, source)) in graphics.graphics.iter().zip(sources).enumerate() {
            if source.length as usize != graphic.encoded_source.len()
                || ffta_rom::digest(&graphic.encoded_source) != source.sha256
                || source.role != format!("bg-main-graphics/{row}")
            {
                return Err("Main graphics source/file binding mismatch".into());
            }
            let body = if graphic.encoded_source[0] == 0x22 {
                8
            } else {
                4
            };
            let block = custom_lz::decode(
                &graphic.encoded_source[body..],
                bg_graphics::DESTINATION_BYTES,
                bg_graphics::MAX_ENCODED_BYTES - body,
            )?;
            let mut positions = vec![u16::MAX; graphic.encoded_source.len()];
            for span in block.literal_spans {
                for index in 0..span.length {
                    positions[body + span.encoded_offset + index] =
                        (span.output_offset + index) as u16;
                }
            }
            literals.push(positions);
            bindings.push(Binding {
                offset: source.offset as usize,
                length: source.length as usize,
                row,
            });
        }
        bindings.sort_by_key(|binding| binding.offset);
        for (index, binding) in bindings.iter().enumerate() {
            let end = binding
                .offset
                .checked_add(binding.length)
                .ok_or("Main graphics binding overflow")?;
            if binding.length == 0
                || end > 0x0200_0000
                || !Self::source_page(binding.offset)
                || !Self::source_page(end - 1)
                || bindings
                    .get(index + 1)
                    .is_some_and(|next| end > next.offset)
            {
                return Err("Invalid or overlapping main graphics bindings".into());
            }
        }
        let last = bindings.last().ok_or("Missing main graphics bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            graphics,
            literals,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            source_bytes: AtomicU64::new(0),
            decoded_literal_bytes: AtomicU64::new(0),
        })
    }

    fn byte(&self, index: usize) -> Option<(u8, bool)> {
        let binding = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - binding.offset;
        if at >= binding.length {
            return None;
        }
        let graphic = &self.graphics.graphics[binding.row];
        let output = self.literals[binding.row][at];
        if output == u16::MAX {
            Some((graphic.encoded_source[at], false))
        } else {
            let output = output as usize;
            Some((graphic.words[output / 4].to_le_bytes()[output % 4], true))
        }
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if index >= self.end || index.saturating_add(N) <= self.bindings[0].offset {
            return prior;
        }
        let (mut mapped, mut literals) = (0, 0);
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, literal)) => {
                mapped += 1;
                literals += u64::from(literal);
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if mapped == 0 {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        self.source_bytes.fetch_add(mapped, Ordering::Relaxed);
        self.decoded_literal_bytes
            .fetch_add(literals, Ordering::Relaxed);
        Some(bytes)
    }

    pub(super) fn stats(&self) -> GraphicsStats {
        GraphicsStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: self.source_bytes.load(Ordering::Relaxed),
            decoded_literal_bytes: self.decoded_literal_bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.graphics.descriptor_graphics.capacity() * 2
            + self.graphics.graphics.capacity() * std::mem::size_of::<bg_graphics::Graphic>()
            + self
                .graphics
                .graphics
                .iter()
                .map(|g| g.words.capacity() * 4 + g.encoded_source.capacity())
                .sum::<usize>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.literals.capacity() * std::mem::size_of::<Vec<u16>>()
            + self
                .literals
                .iter()
                .map(|positions| positions.capacity() * 2)
                .sum::<usize>()
    }
}

#[cfg(test)]
#[path = "graphics_tests.rs"]
mod tests;
