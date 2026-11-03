//! Full file-derived EUR text catalogs; all original script/copy/decode work stays guest-side.
use super::super::text_catalogs::{self as model, Program, ProgramUnit};
use super::*;
use super::text_program::{TextProgramMap, ProgramKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextCatalogStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub pointer_table_bytes: u64,
    pub string_bytes: u64,
    pub archive_index_bytes: u64,
    pub header_bytes: u64,
    pub encoded_control_bytes: u64,
    pub decoded_program_bytes: u64,
    pub glyph_bytes: u64,
    pub width_bytes: u64,
}

#[cfg(test)]
#[path = "text_catalog_tests.rs"]
mod tests;
#[derive(Clone, Copy)]
enum Kind {
    Pointer(usize, usize),
    String(usize),
    Index(usize, usize),
    Program(usize),
    Glyph(usize),
    Width(usize),
}
struct Binding {
    offset: usize,
    length: usize,
    kind: Kind,
}
struct ProgramMap {
    category: usize,
    language: usize,
    item: usize,
    map: TextProgramMap,
}
pub(super) struct TextCatalogResources {
    texts: model::TextCatalogs,
    strings: Vec<Vec<usize>>,
    programs: Vec<ProgramMap>,
    bindings: Vec<Binding>,
    end: usize,
    file_sha256: String,
    bus_reads: AtomicU64,
    counts: [AtomicU64; 8],
}
impl TextCatalogResources {
    pub(super) fn without_font(mut self) -> Self {
        // The complete bank takes over these overlapping sources. Earlier
        // subset files remain source authenticated before this owner is built.
        self.bindings.retain(|b| !matches!(b.kind, Kind::Glyph(_) | Kind::Width(_)));
        self.texts.glyphs.clear();
        let last = self.bindings.last().unwrap();
        self.end = last.offset + last.length;
        self
    }
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index>>12,0x495..=0x498|0x49a..=0x49e|0x4a4|0x4f8..=0x6b3|0x71e..=0x766)
    }
    pub(super) fn new(
        texts: model::TextCatalogs,
        sources: &[SourceRange],
        file_sha256: String,
    ) -> Result<Self, String> {
        texts.validate()?;
        let mut ranges = sources.iter();
        let mut bindings = Vec::new();
        let mut add = |offset: u32, bytes: Vec<u8>, role: String, kind| -> Result<(), String> {
            let source = ranges.next().ok_or("Missing full text source binding")?;
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Full text source/file binding mismatch".into());
            }
            bindings.push(Binding {
                offset: offset as usize,
                length: bytes.len(),
                kind,
            });
            Ok(())
        };
        for (category, c) in texts.pointer_catalogs.iter().enumerate() {
            for (language, l) in c.languages.iter().enumerate() {
                add(
                    l.source_offset,
                    texts.table_bytes(l)?,
                    format!("text-pointer-table/{category}/{language}"),
                    Kind::Pointer(category, language),
                )?;
            }
        }
        let mut strings = Vec::new();
        for (id, text) in texts.strings.iter().enumerate() {
            let mut at = usize::from(text.encoding == text_short::Encoding::Compact);
            strings.push(
                text.units
                    .iter()
                    .map(|u| {
                        let n = at;
                        at += if matches!(u, text_short::Unit::Glyph { .. })
                            && text.encoding == text_short::Encoding::Extended
                        {
                            2
                        } else {
                            1
                        };
                        n
                    })
                    .collect(),
            );
            add(
                text.source_offset,
                text.source_bytes()?,
                format!("text-pointer-string/{id}"),
                Kind::String(id),
            )?;
        }
        let mut programs = Vec::new();
        for (category, c) in texts.archive_catalogs.iter().enumerate() {
            for (language, a) in c.languages.iter().enumerate() {
                add(
                    a.source_offset,
                    a.offsets.iter().flat_map(|v| v.to_le_bytes()).collect(),
                    format!("text-archive-index/{category}/{language}"),
                    Kind::Index(category, language),
                )?;
                for (item, p) in a.programs.iter().enumerate() {
                    let id = programs.len();
                    programs.push(ProgramMap { category, language, item, map: TextProgramMap::new(p)? });
                    add(
                        p.source_offset,
                        p.source_bytes()?,
                        format!("text-archive-program/{category}/{language}/{item}"),
                        Kind::Program(id),
                    )?;
                }
            }
        }
        for (item, g) in texts.glyphs.iter().enumerate() {
            add(
                g.source_offset(),
                g.source_bytes(),
                format!("text-font-glyph/{}", g.index),
                Kind::Glyph(item),
            )?;
            add(
                g.width_offset(),
                vec![g.width],
                format!("text-font-width/{}", g.index),
                Kind::Width(item),
            )?;
        }
        if ranges.next().is_some() {
            return Err("Unexpected full text source binding".into());
        }
        bindings.sort_by_key(|b| b.offset);
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
                return Err("Full text source page/overlap mismatch".into());
            }
        }
        let last = bindings.last().ok_or("Missing full text bindings")?;
        let end = last.offset + last.length;
        Ok(Self {
            texts,
            strings,
            programs,
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
        Some(match b.kind {
            Kind::Pointer(c, l) => {
                let id = self.texts.pointer_catalogs[c].languages[l].entries[at / 4] as usize;
                (
                    (0x08000000 + self.texts.strings[id].source_offset).to_le_bytes()[at % 4],
                    0,
                )
            }
            Kind::String(id) => {
                let text = &self.texts.strings[id];
                let compact = text.encoding == text_short::Encoding::Compact;
                let byte = if at == b.length - 1 {
                    0
                } else if compact && at == 0 {
                    1
                } else {
                    let unit = self.strings[id]
                        .partition_point(|&offset| offset <= at)
                        .checked_sub(1)?;
                    match text.units[unit] {
                        text_short::Unit::Glyph { index } => {
                            if compact {
                                index as u8 + 1
                            } else {
                                (index | 0x8000).to_be_bytes()[at - self.strings[id][unit]]
                            }
                        }
                        text_short::Unit::ControlByte { value } => value,
                    }
                };
                (byte, 1)
            }
            Kind::Index(c, l) => (
                self.texts.archive_catalogs[c].languages[l].offsets[at / 2].to_le_bytes()[at % 2],
                2,
            ),
            Kind::Program(id) => {
                let map = &self.programs[id];
                let p = &self.texts.archive_catalogs[map.category].languages[map.language].programs
                    [map.item];
                let (byte, kind) = map.map.source_byte(p, at)?;
                (byte, match kind { ProgramKind::Header => 3, ProgramKind::Control => 4, _ => 5 })
            }
            Kind::Glyph(id) => (
                self.texts.glyphs[id].tile_words[at / 4].to_le_bytes()[at % 4],
                6,
            ),
            Kind::Width(id) => (self.texts.glyphs[id].width, 7),
        })
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
        let mut counts = [0; 8];
        let bytes = std::array::from_fn(|i| match self.byte(index + i) {
            Some((byte, kind)) => {
                counts[kind] += 1;
                byte
            }
            None => prior
                .as_ref()
                .map_or_else(|| rom[index + i], |bytes| bytes[i]),
        });
        if counts.iter().all(|&v| v == 0) {
            return prior;
        }
        self.bus_reads.fetch_add(1, Ordering::Relaxed);
        for (counter, n) in self.counts.iter().zip(counts) {
            counter.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> TextCatalogStats {
        let [pointer_table_bytes, string_bytes, archive_index_bytes, header_bytes, encoded_control_bytes, decoded_program_bytes, glyph_bytes, width_bytes] =
            self.counts.each_ref().map(|c| c.load(Ordering::Relaxed));
        TextCatalogStats {
            bus_reads: self.bus_reads.load(Ordering::Relaxed),
            source_bytes: pointer_table_bytes
                + string_bytes
                + archive_index_bytes
                + header_bytes
                + encoded_control_bytes
                + decoded_program_bytes
                + glyph_bytes
                + width_bytes,
            pointer_table_bytes,
            string_bytes,
            archive_index_bytes,
            header_bytes,
            encoded_control_bytes,
            decoded_program_bytes,
            glyph_bytes,
            width_bytes,
        }
    }
    pub(super) fn file_sha256(&self) -> &str {
        &self.file_sha256
    }
    pub(super) fn resident_bytes(&self) -> usize {
        // Diagnostic accounting only, not a budget or performance measurement.
        std::mem::size_of::<Self>()
            + self.file_sha256.capacity()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.strings.capacity() * std::mem::size_of::<Vec<usize>>()
            + self
                .strings
                .iter()
                .map(|v| v.capacity() * std::mem::size_of::<usize>())
                .sum::<usize>()
            + self.programs.capacity() * std::mem::size_of::<ProgramMap>()
            + self
                .programs
                .iter()
                .map(|p| {
p.map.resident_bytes()
                })
                .sum::<usize>()
            + self.texts.pointer_catalogs.capacity() * std::mem::size_of::<model::PointerCatalog>()
            + self
                .texts
                .pointer_catalogs
                .iter()
                .map(|c| {
                    c.languages.capacity() * std::mem::size_of::<model::PointerLanguage>()
                        + c.languages
                            .iter()
                            .map(|l| l.entries.capacity() * 2)
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.texts.strings.capacity() * std::mem::size_of::<text_short::Text>()
            + self
                .texts
                .strings
                .iter()
                .map(|t| t.units.capacity() * std::mem::size_of::<text_short::Unit>())
                .sum::<usize>()
            + self.texts.archive_catalogs.capacity() * std::mem::size_of::<model::ArchiveCatalog>()
            + self
                .texts
                .archive_catalogs
                .iter()
                .map(|c| {
                    c.languages.capacity() * std::mem::size_of::<model::Archive>()
                        + c.languages
                            .iter()
                            .map(|a| {
                                a.offsets.capacity() * 2
                                    + a.programs.capacity() * std::mem::size_of::<Program>()
                                    + a.programs
                                        .iter()
                                        .map(|p| {
                                            p.units.capacity() * std::mem::size_of::<ProgramUnit>()
                                                + p.units
                                                    .iter()
                                                    .map(|u| {
                                                        if let ProgramUnit::Command {
                                                            operands,
                                                            ..
                                                        } = u
                                                        {
                                                            operands.capacity()
                                                        } else {
                                                            0
                                                        }
                                                    })
                                                    .sum::<usize>()
                                                + p.decoded_padding.capacity()
                                                + p.encoded.as_ref().map_or(0, Vec::capacity)
                                        })
                                        .sum::<usize>()
                            })
                            .sum::<usize>()
                })
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
