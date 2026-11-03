//! Bounded EUR pointer strings and relative-offset text programs.
//! Script commands retain operands; dynamic names/flags stay original guest work.
use super::{
    custom_lz,
    text_short::{self, Glyph, Text, Unit},
    SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[path = "text_roots.rs"]
mod roots;
pub use roots::*;
pub const MAX_PROGRAM_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointerLanguage {
    pub source_offset: u32,
    pub entries: Vec<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointerCatalog {
    pub reader: u32,
    /// Original language IDs0..4, including reader's default mapping to0.
    pub languages: Vec<PointerLanguage>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProgramUnit {
    Glyph {
        index: u16,
    },
    /// Opcode uses its exact source representation; semantics remain guest-side.
    Command {
        opcode: u8,
        operands: Vec<u8>,
    },
}
fn operand_count(opcode: u8, compact: bool, input: &[u8]) -> Result<usize, String> {
    if compact {
        return match opcode {
            0x80..=0x96 if !matches!(opcode, 0x85 | 0x92) => Ok(usize::from(matches!(
                opcode,
                0x80..=0x84 | 0x86 | 0x87 | 0x93 | 0x94 | 0x96
            ))),
            _ => Err("Unsupported compact archive opcode".into()),
        };
    }
    match opcode {
        0x21 | 0x25 | 0x3c | 0x3e | 0x67 | 0x6c | 0x72 | 0x74 => Ok(1),
        0x66 => Ok(2),
        // Original copies two zero-terminated inline byte strings verbatim.
        0x53 => {
            let first = input
                .iter()
                .position(|&v| v == 0)
                .ok_or("Unterminated first inline text operand")?
                + 1;
            let second = input[first..]
                .iter()
                .position(|&v| v == 0)
                .ok_or("Unterminated second inline text operand")?
                + 1;
            Ok(first + second)
        }
        0x21..=0x79 => Ok(0),
        _ => Err("Unsupported extended archive opcode".into()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub source_offset: u32,
    /// All original16bits retained; bit0compact, bit1customLZ, upper bits live metadata.
    pub header: u16,
    pub units: Vec<ProgramUnit>,
    /// Exact decompressed bytes after logical terminator, independently retained.
    pub decoded_padding: Vec<u8>,
    /// Original custom stream, including controls; None for uncompressed programs.
    pub encoded: Option<Vec<u8>>,
}
impl Program {
    pub fn decode(input: &[u8], source_offset: u32) -> Result<(Self, usize), String> {
        let header = u16::from_le_bytes(
            input
                .get(..2)
                .ok_or("Missing archive header")?
                .try_into()
                .unwrap(),
        );
        let block = if header & 2 != 0 {
            Some(custom_lz::decode(
                &input[2..],
                MAX_PROGRAM_BYTES,
                MAX_PROGRAM_BYTES,
            )?)
        } else {
            None
        };
        let data = block.as_ref().map_or(&input[2..], |b| b.bytes.as_slice());
        let compact = header & 1 != 0;
        let mut at = 0;
        let mut units = Vec::new();
        loop {
            if at >= MAX_PROGRAM_BYTES {
                return Err("Archive program exceeds source guard".into());
            }
            let byte = *data.get(at).ok_or("Unterminated archive program")?;
            at += 1;
            if byte == if compact { 255 } else { 0 } {
                break;
            }
            let unit = if compact && byte < 128 {
                ProgramUnit::Glyph {
                    index: u16::from(byte),
                }
            } else if !compact && byte != 64 {
                let low = *data.get(at).ok_or("Truncated archive glyph")?;
                at += 1;
                if byte & 128 == 0 {
                    return Err("Invalid extended archive glyph prefix".into());
                }
                ProgramUnit::Glyph {
                    index: (u16::from(byte & 127) << 8) | u16::from(low),
                }
            } else {
                let opcode = if compact {
                    byte
                } else {
                    let v = *data.get(at).ok_or("Missing archive opcode")?;
                    at += 1;
                    v
                };
                let count =
                    operand_count(opcode, compact, data.get(at..).ok_or("Missing operands")?)?;
                let operands = data
                    .get(at..at + count)
                    .ok_or("Truncated archive operands")?
                    .to_vec();
                at += count;
                ProgramUnit::Command { opcode, operands }
            };
            units.push(unit);
        }
        let used = block.as_ref().map_or(at, |b| b.consumed);
        let result = Self {
            source_offset,
            header,
            units,
            decoded_padding: if block.is_some() {
                data[at..].to_vec()
            } else {
                Vec::new()
            },
            encoded: block.as_ref().map(|_| input[2..2 + used].to_vec()),
        };
        result.validate()?;
        Ok((result, 2 + used))
    }
    /// Decoded script bytes, including terminator; dynamic substitution deferred.
    pub fn program_bytes(&self) -> Result<Vec<u8>, String> {
        if self.units.len() > MAX_PROGRAM_BYTES || self.decoded_padding.len() > MAX_PROGRAM_BYTES {
            return Err("Archive unit/padding guard".into());
        }
        let compact = self.header & 1 != 0;
        let mut bytes = Vec::new();
        for unit in &self.units {
            match unit {
                ProgramUnit::Glyph { index } if compact && *index < 128 => bytes.push(*index as u8),
                ProgramUnit::Glyph { index } if !compact && *index < 0x8000 => {
                    bytes.extend_from_slice(&(*index | 0x8000).to_be_bytes())
                }
                ProgramUnit::Command { opcode, operands } => {
                    let count = operand_count(*opcode, compact, operands)?;
                    if count != operands.len() {
                        return Err("Archive operand count mismatch".into());
                    }
                    if !compact {
                        bytes.push(64);
                    }
                    bytes.push(*opcode);
                    bytes.extend_from_slice(operands);
                }
                _ => return Err("Invalid archive glyph encoding".into()),
            }
        }
        bytes.push(if compact { 255 } else { 0 });
        if bytes.len() + self.decoded_padding.len() > MAX_PROGRAM_BYTES {
            return Err("Archive program length guard".into());
        }
        Ok(bytes)
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut bytes = self.program_bytes()?;
        bytes.extend_from_slice(&self.decoded_padding);
        match (&self.encoded, self.header & 2 != 0) {
            (Some(encoded), true) => {
                let block = custom_lz::decode(encoded, MAX_PROGRAM_BYTES, MAX_PROGRAM_BYTES)?;
                if block.consumed != encoded.len() || block.bytes != bytes {
                    return Err("Archive encoded/typed script mismatch".into());
                }
            }
            (None, false) if self.decoded_padding.is_empty() => {}
            _ => return Err("Archive compression/header/padding mismatch".into()),
        }
        Ok(())
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = self.header.to_le_bytes().to_vec();
        if let Some(encoded) = &self.encoded {
            bytes.extend_from_slice(encoded);
        } else {
            bytes.extend(self.program_bytes()?);
        }
        Ok(bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub source_offset: u32,
    /// Original finalFFFFsentinel is explicit; there is no program for it.
    pub offsets: Vec<u16>,
    pub programs: Vec<Program>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveCatalog {
    pub reader: u32,
    pub languages: Vec<Archive>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextCatalogs {
    pub pointer_catalogs: Vec<PointerCatalog>,
    pub strings: Vec<Text>,
    pub archive_catalogs: Vec<ArchiveCatalog>,
    pub glyphs: Vec<Glyph>,
}
impl TextCatalogs {
    pub fn validate(&self) -> Result<(), String> {
        if self.pointer_catalogs.len() != 9
            || self.archive_catalogs.len() != 29
            || self.strings.len() > 16100
        {
            return Err("Invalid EUR text catalog counts".into());
        }
        let mut seen = BTreeSet::new();
        let mut glyphs = BTreeSet::new();
        let mut spans = Vec::new();
        for (id, catalog) in self.pointer_catalogs.iter().enumerate() {
            if catalog.reader != POINTER_READERS[id] || catalog.languages.len() != 5 {
                return Err("Invalid pointer catalog selector".into());
            }
            for (language, table) in catalog.languages.iter().enumerate() {
                let root = POINTER_ROOTS[id][language];
                if table.source_offset != root || table.entries.len() != POINTER_COUNTS[id] {
                    return Err("Invalid pointer catalog table".into());
                }
                spans.push((root, root + table.entries.len() as u32 * 4));
                for &entry in &table.entries {
                    let text = self
                        .strings
                        .get(entry as usize)
                        .ok_or("Missing pointer string ID")?;
                    let bytes = text.source_bytes()?;
                    let end = text
                        .source_offset
                        .checked_add(bytes.len() as u32)
                        .ok_or("Text extent overflow")?;
                    if text.source_offset < POINTER_STARTS[id][language] || end > root {
                        return Err("String outside declared catalog".into());
                    }
                    if seen.insert(entry) {
                        if entry as usize != seen.len() - 1 {
                            return Err("Noncanonical pointer string ID".into());
                        }
                        spans.push((text.source_offset, end));
                        for unit in &text.units {
                            if let Unit::Glyph { index } = unit {
                                glyphs.insert(*index);
                            }
                        }
                    }
                }
            }
        }
        if seen.len() != self.strings.len() {
            return Err("Unreferenced pointer text".into());
        }
        for (id, catalog) in self.archive_catalogs.iter().enumerate() {
            if catalog.reader != ARCHIVE_READERS[id] || catalog.languages.len() != 5 {
                return Err("Invalid archive selector".into());
            }
            for (language, archive) in catalog.languages.iter().enumerate() {
                let root = ARCHIVE_ROOTS[id][language];
                let count = archive.offsets.len();
                if archive.source_offset != root
                    || count < 2
                    || count > 2048
                    || archive.programs.len() != count - 1
                    || archive.offsets[0] as usize != count * 2
                    || archive.offsets[count - 1] != 65535
                    || archive.offsets.windows(2).any(|v| v[0] >= v[1])
                {
                    return Err("Invalid relative archive index/sentinel".into());
                }
                spans.push((root, root + count as u32 * 2));
                let next = ARCHIVE_ROOTS
                    .iter()
                    .map(|r| r[language])
                    .filter(|&r| r > root)
                    .min()
                    .unwrap_or(0x71e70c);
                for (index, program) in archive.programs.iter().enumerate() {
                    let at = root + u32::from(archive.offsets[index]);
                    let bytes = program.source_bytes()?;
                    let bound = if index + 1 < count - 1 {
                        root + u32::from(archive.offsets[index + 1])
                    } else {
                        next
                    };
                    if program.source_offset != at
                        || at
                            .checked_add(bytes.len() as u32)
                            .is_none_or(|end| end > bound)
                    {
                        return Err("Archive program extent mismatch".into());
                    }
                    spans.push((at, at + bytes.len() as u32));
                    for unit in &program.units {
                        if let ProgramUnit::Glyph { index } = unit {
                            glyphs.insert(*index);
                        }
                    }
                }
            }
        }
        if self.glyphs.len() != glyphs.len() {
            return Err("Text glyph coverage mismatch".into());
        }
        for (glyph, &index) in self.glyphs.iter().zip(&glyphs) {
            if glyph.index != index
                || glyph.tile_words.len() != 32
                || glyph.source_offset() + 128 > text_short::WIDTH_ROOT
            {
                return Err("Invalid referenced text font slot".into());
            }
            spans.push((glyph.source_offset(), glyph.source_offset() + 128));
            spans.push((glyph.width_offset(), glyph.width_offset() + 1));
        }
        spans.sort_unstable();
        if spans.windows(2).any(|v| v[0].1 > v[1].0) {
            return Err("Overlapping text sources".into());
        }
        Ok(())
    }
    pub fn table_bytes(&self, table: &PointerLanguage) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        for &id in &table.entries {
            let offset = self
                .strings
                .get(id as usize)
                .ok_or("Missing pointer text ID")?
                .source_offset;
            bytes.extend_from_slice(
                &offset
                    .checked_add(0x08000000)
                    .ok_or("Text pointer overflow")?
                    .to_le_bytes(),
            );
        }
        Ok(bytes)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("Text catalogs differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: TextCatalogs,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Text catalogs require exact EUR ROM; USA unverified".into());
        }
        let mut decoded = TextCatalogs {
            pointer_catalogs: Vec::new(),
            strings: Vec::new(),
            archive_catalogs: Vec::new(),
            glyphs: Vec::new(),
        };
        let mut ids = BTreeMap::new();
        let mut glyphs = BTreeSet::new();
        for id in 0..9 {
            let mut languages = Vec::new();
            for language in 0..5 {
                let root = POINTER_ROOTS[id][language] as usize;
                let count = POINTER_COUNTS[id];
                let mut entries = Vec::new();
                for bytes in rom[root..root + count * 4].chunks_exact(4) {
                    let offset = u32::from_le_bytes(bytes.try_into().unwrap())
                        .checked_sub(0x08000000)
                        .ok_or("Text pointer outside cartridge")?;
                    if offset < POINTER_STARTS[id][language] || offset as usize >= root {
                        return Err("Text pointer outside declared payload".into());
                    }
                    let entry = if let Some(&id) = ids.get(&offset) {
                        id
                    } else {
                        let (text, _) = Text::decode(&rom[offset as usize..root], offset)?;
                        for unit in &text.units {
                            if let Unit::Glyph { index } = unit {
                                glyphs.insert(*index);
                            }
                        }
                        let id = decoded.strings.len() as u16;
                        decoded.strings.push(text);
                        ids.insert(offset, id);
                        id
                    };
                    entries.push(entry);
                }
                languages.push(PointerLanguage {
                    source_offset: root as u32,
                    entries,
                });
            }
            decoded.pointer_catalogs.push(PointerCatalog {
                reader: POINTER_READERS[id],
                languages,
            });
        }
        for id in 0..29 {
            let mut languages = Vec::new();
            for language in 0..5 {
                let root = ARCHIVE_ROOTS[id][language];
                let at = root as usize;
                let length = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) as usize;
                if length < 4 || length % 2 != 0 || length > 4096 {
                    return Err("Invalid archive index length".into());
                }
                let offsets: Vec<_> = rom[at..at + length]
                    .chunks_exact(2)
                    .map(|v| u16::from_le_bytes(v.try_into().unwrap()))
                    .collect();
                if offsets.last() != Some(&65535) || offsets.windows(2).any(|v| v[0] >= v[1]) {
                    return Err("Invalid archive index order/sentinel".into());
                }
                let next = ARCHIVE_ROOTS
                    .iter()
                    .map(|r| r[language])
                    .filter(|&r| r > root)
                    .min()
                    .unwrap_or(0x71e70c);
                let mut programs = Vec::new();
                for index in 0..offsets.len() - 1 {
                    let start = root + u32::from(offsets[index]);
                    let end = if index + 1 < offsets.len() - 1 {
                        root + u32::from(offsets[index + 1])
                    } else {
                        next
                    };
                    let (program, _) = Program::decode(
                        rom.get(start as usize..end as usize)
                            .ok_or("Archive extent outside ROM")?,
                        start,
                    )?;
                    for unit in &program.units {
                        if let ProgramUnit::Glyph { index } = unit {
                            glyphs.insert(*index);
                        }
                    }
                    programs.push(program);
                }
                languages.push(Archive {
                    source_offset: root,
                    offsets,
                    programs,
                });
            }
            decoded.archive_catalogs.push(ArchiveCatalog {
                reader: ARCHIVE_READERS[id],
                languages,
            });
        }
        for index in glyphs {
            let at = text_short::GLYPH_ROOT as usize + index as usize * 128;
            if at + 128 > text_short::WIDTH_ROOT as usize {
                return Err("Font slot outside payload guard".into());
            }
            decoded.glyphs.push(Glyph {
                index,
                width: rom[text_short::WIDTH_ROOT as usize + index as usize],
                tile_words: rom[at..at + 128]
                    .chunks_exact(4)
                    .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                    .collect(),
            });
        }
        decoded.validate()?;
        let mut sources = Vec::new();
        let mut add = |offset, bytes: Vec<u8>, role| {
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            })
        };
        for (id, c) in decoded.pointer_catalogs.iter().enumerate() {
            for (language, t) in c.languages.iter().enumerate() {
                add(
                    t.source_offset,
                    decoded.table_bytes(t)?,
                    format!("text-pointer-table/{id}/{language}"),
                );
            }
        }
        for (id, t) in decoded.strings.iter().enumerate() {
            add(
                t.source_offset,
                t.source_bytes()?,
                format!("text-pointer-string/{id}"),
            );
        }
        for (id, c) in decoded.archive_catalogs.iter().enumerate() {
            for (language, a) in c.languages.iter().enumerate() {
                add(
                    a.source_offset,
                    a.offsets.iter().flat_map(|v| v.to_le_bytes()).collect(),
                    format!("text-archive-index/{id}/{language}"),
                );
                for (index, p) in a.programs.iter().enumerate() {
                    add(
                        p.source_offset,
                        p.source_bytes()?,
                        format!("text-archive-program/{id}/{language}/{index}"),
                    );
                }
            }
        }
        for g in &decoded.glyphs {
            add(
                g.source_offset(),
                g.source_bytes(),
                format!("text-font-glyph/{}", g.index),
            );
            add(
                g.width_offset(),
                vec![g.width],
                format!("text-font-width/{}", g.index),
            );
        }
        Ok(Self { decoded, sources })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_commands_preserve_zero_operands_inline_strings_and_encoded_padding() {
        let raw = [0x30, 0x18, 0x80, 0, 64, 0x3e, 0, 64, 0x53, 1, 0, 2, 0, 0];
        let (p, n) = Program::decode(&raw, 0).unwrap();
        assert_eq!(n, raw.len());
        assert_eq!(p.source_bytes().unwrap(), raw);
        let compact = [0x11, 0x14, 0, 0x81, 0, 0x8a, 0xff];
        let (p, n) = Program::decode(&compact, 0).unwrap();
        assert_eq!(n, compact.len());
        assert_eq!(p.source_bytes().unwrap(), compact);
        let mut compressed = vec![0x12, 0, 0, 0, 0, 4, 0x43, 128, 0, 0, 0];
        let (p, n) = Program::decode(&compressed, 0).unwrap();
        assert_eq!(n, compressed.len());
        assert_eq!(p.decoded_padding, [0]);
        assert_eq!(p.source_bytes().unwrap(), compressed);
        compressed[6] = 0x44;
        assert!(Program::decode(&compressed, 0).is_err());
        for data in [
            vec![],
            vec![0],
            vec![0, 0, 128],
            vec![1, 0, 0x85],
            vec![0, 0, 64, 0x53, 0],
        ] {
            assert!(Program::decode(&data, 0).is_err());
        }
        let mut bad = p;
        bad.header &= !2;
        assert!(bad.validate().is_err());
    }
}
