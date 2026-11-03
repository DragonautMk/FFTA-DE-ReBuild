//! Exact original source bytes projected from typed text/font data.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub table_bytes: u64,
    pub string_bytes: u64,
    pub glyph_bytes: u64,
    pub width_bytes: u64,
}
#[derive(Clone, Copy)]
enum Kind {
    Table,
    String,
    Glyph,
    Width,
}
struct Binding {
    offset: usize,
    length: usize,
    item: usize,
    kind: Kind,
}
pub(super) struct TextResources {
    texts: text_short::ShortTexts,
    unit_offsets: Vec<Vec<usize>>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    counts: [AtomicU64; 4],
}
impl TextResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x495..=0x49d | 0x4f8 | 0x725 | 0x734 | 0x741 | 0x74f..=0x750 | 0x75e..=0x75f)
    }
    pub(super) fn new(
        texts: text_short::ShortTexts,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        texts.validate()?;
        let mut bindings = Vec::new();
        let mut ranges = sources.iter();
        let mut add =
            |offset: u32, bytes: Vec<u8>, role: String, item, kind| -> Result<(), String> {
                let source = ranges.next().ok_or("Missing text source binding")?;
                if source.offset != offset
                    || source.length as usize != bytes.len()
                    || source.sha256 != ffta_rom::digest(&bytes)
                    || source.role != role
                {
                    return Err("Text source/file binding mismatch".into());
                }
                bindings.push(Binding {
                    offset: offset as usize,
                    length: bytes.len(),
                    item,
                    kind,
                });
                Ok(())
            };
        for (id, language) in texts.languages.iter().enumerate() {
            add(
                language.table_offset,
                texts.table_bytes(language)?,
                format!("short-text-table/{id}"),
                id,
                Kind::Table,
            )?;
        }
        let mut unit_offsets = Vec::new();
        for (id, text) in texts.strings.iter().enumerate() {
            let mut at = usize::from(text.encoding == text_short::Encoding::Compact);
            let offsets = text
                .units
                .iter()
                .map(|unit| {
                    let offset = at;
                    at += if matches!(unit, text_short::Unit::Glyph { .. })
                        && text.encoding == text_short::Encoding::Extended
                    {
                        2
                    } else {
                        1
                    };
                    offset
                })
                .collect();
            unit_offsets.push(offsets);
            add(
                text.source_offset,
                text.source_bytes()?,
                format!("short-text-string/{id}"),
                id,
                Kind::String,
            )?;
        }
        for (item, glyph) in texts.glyphs.iter().enumerate() {
            add(
                glyph.source_offset(),
                glyph.source_bytes(),
                format!("short-text-glyph/{}", glyph.index),
                item,
                Kind::Glyph,
            )?;
            add(
                glyph.width_offset(),
                vec![glyph.width],
                format!("short-text-width/{}", glyph.index),
                item,
                Kind::Width,
            )?;
        }
        if ranges.next().is_some() {
            return Err("Unexpected text source binding".into());
        }
        bindings.sort_by_key(|binding| binding.offset);
        for (i, b) in bindings.iter().enumerate() {
            let end = b
                .offset
                .checked_add(b.length)
                .ok_or("Text binding overflow")?;
            if b.length == 0
                || !Self::source_page(b.offset)
                || !Self::source_page(end - 1)
                || bindings.get(i + 1).is_some_and(|next| end > next.offset)
            {
                return Err("Invalid/overlapping text bindings or unsupported source page".into());
            }
        }
        let last = bindings.last().ok_or("Missing text bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            texts,
            unit_offsets,
            bindings,
            end,
            file_sha256,
            bus_reads: AtomicU64::new(0),
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, index: usize) -> Option<(u8, usize)> {
        let b = self.bindings.get(
            self.bindings
                .partition_point(|b| b.offset <= index)
                .checked_sub(1)?,
        )?;
        let at = index - b.offset;
        if at >= b.length {
            return None;
        }
        let (byte, kind) = match b.kind {
            Kind::Table => {
                let id = self.texts.languages[b.item].entries[at / 4] as usize;
                (
                    (0x08000000 + self.texts.strings[id].source_offset).to_le_bytes()[at % 4],
                    0,
                )
            }
            Kind::String => {
                let text = &self.texts.strings[b.item];
                let byte = if at == b.length - 1 {
                    0
                } else if text.encoding == text_short::Encoding::Compact && at == 0 {
                    1
                } else {
                    let unit = self.unit_offsets[b.item]
                        .partition_point(|&offset| offset <= at)
                        .checked_sub(1)?;
                    match text.units[unit] {
                        text_short::Unit::Glyph { index } => {
                            if text.encoding == text_short::Encoding::Compact {
                                index as u8 + 1
                            } else {
                                (index | 0x8000).to_be_bytes()[at - self.unit_offsets[b.item][unit]]
                            }
                        }
                        text_short::Unit::ControlByte { value } => value,
                    }
                };
                (byte, 1)
            }
            Kind::Glyph => (
                self.texts.glyphs[b.item].tile_words[at / 4].to_le_bytes()[at % 4],
                2,
            ),
            Kind::Width => (self.texts.glyphs[b.item].width, 3),
        };
        Some((byte, kind))
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
        let mut counts = [0; 4];
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, kind)) => {
                counts[kind] += 1;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if counts.iter().all(|&count| count == 0) {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        for (counter, count) in self.counts.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> TextStats {
        let [table_bytes, string_bytes, glyph_bytes, width_bytes] = self
            .counts
            .each_ref()
            .map(|counter| counter.load(Ordering::Relaxed));
        TextStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: table_bytes + string_bytes + glyph_bytes + width_bytes,
            table_bytes,
            string_bytes,
            glyph_bytes,
            width_bytes,
        }
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.unit_offsets.capacity() * std::mem::size_of::<Vec<usize>>()
            + self
                .unit_offsets
                .iter()
                .map(|v| v.capacity() * std::mem::size_of::<usize>())
                .sum::<usize>()
            + self.texts.languages.capacity() * std::mem::size_of::<text_short::Language>()
            + self
                .texts
                .languages
                .iter()
                .map(|l| l.entries.capacity() * 2)
                .sum::<usize>()
            + self.texts.strings.capacity() * std::mem::size_of::<text_short::Text>()
            + self
                .texts
                .strings
                .iter()
                .map(|t| t.units.capacity() * std::mem::size_of::<text_short::Unit>())
                .sum::<usize>()
            + self.texts.glyphs.capacity() * std::mem::size_of::<text_short::Glyph>()
            + self
                .texts
                .glyphs
                .iter()
                .map(|g| g.tile_words.capacity() * 4)
                .sum::<usize>()
    }
}
#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
