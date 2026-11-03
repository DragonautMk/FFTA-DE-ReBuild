//! Script-selected EUR text. Typed indices share physical banks and the existing text codec.
//! Source projection only: original live selection, text expansion and guest control remain authoritative.
use super::{
    fonts, script_archive, text_catalogs::Program, text_catalogs::ProgramUnit, SourceRange,
};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const ROOTS: [u32; 5] = [0xbb064c, 0xc076f0, 0xc5fbf0, 0xcb7c48, 0xd10644];
pub const END: u32 = 0xd68c2c;
pub const SOURCE_BYTES: usize = 1_803_744;
pub const BANKS_PER_LANGUAGE: usize = 77;
pub const PROGRAMS_PER_LANGUAGE: usize = 8202;
pub const ALIGNMENT_COUNT: usize = 34314;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TopTarget {
    Bank { index: u8 },
    Variants {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantIndex {
    pub source_offset: u32,
    pub bank_indices: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub source_offset: u32,
    /// Physical programs own their data once. Index offsets are reconstructed from these addresses.
    pub programs: Vec<Program>,
    pub sentinel: u16,
}
impl Bank {
    pub fn index_bytes(&self) -> Result<Vec<u8>, String> {
        if self.programs.is_empty() || self.programs.len() >= 2048 || self.sentinel != 0xffff {
            return Err("Invalid script text bank count/sentinel".into());
        }
        let mut bytes = Vec::with_capacity((self.programs.len() + 1) * 2);
        let mut last = 0;
        for (i, program) in self.programs.iter().enumerate() {
            let relative = program
                .source_offset
                .checked_sub(self.source_offset)
                .and_then(|v| u16::try_from(v).ok())
                .ok_or("Script text program offset overflow")?;
            if relative == 0xffff
                || i == 0 && usize::from(relative) != (self.programs.len() + 1) * 2
                || i != 0 && relative <= last
            {
                return Err("Invalid script text program index order/length".into());
            }
            bytes.extend_from_slice(&relative.to_le_bytes());
            last = relative;
        }
        bytes.extend_from_slice(&self.sentinel.to_le_bytes());
        Ok(bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    pub id: u8,
    pub source_offset: u32,
    pub end: u32,
    pub top: Vec<TopTarget>,
    pub variants: VariantIndex,
    pub banks: Vec<Bank>,
}
impl Language {
    pub fn target_offset(&self, target: &TopTarget) -> Option<u32> {
        match target {
            TopTarget::Bank { index } => {
                self.banks.get(usize::from(*index)).map(|b| b.source_offset)
            }
            TopTarget::Variants {} => Some(self.variants.source_offset),
        }
    }
    /// Bounded source lookup, not a replacement for original message/key or raw-ID handling.
    pub fn source_bank(&self, text_bank: u8, live: u8) -> Option<&Bank> {
        let index = match self.top.get(usize::from(text_bank))? {
            TopTarget::Bank { index } => *index,
            TopTarget::Variants {} => *self
                .variants
                .bank_indices
                .get(usize::from(live.saturating_sub(1) / 10))?,
        };
        self.banks.get(usize::from(index))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alignment {
    pub source_offset: u32,
    /// Exact bounded slack; mixed values have no invented text semantics.
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptText {
    pub languages: Vec<Language>,
    pub alignment: Vec<Alignment>,
}
impl ScriptText {
    pub fn source_language(&self, language: u8) -> Option<&Language> {
        self.languages.get(if language < 5 {
            usize::from(language)
        } else {
            0
        })
    }
    fn parts(&self) -> Result<Vec<(u32, Vec<u8>)>, String> {
        if self.languages.len() != 5 || self.alignment.len() != ALIGNMENT_COUNT {
            return Err("Invalid script text language/alignment count".into());
        }
        let mut parts = Vec::new();
        for (id, language) in self.languages.iter().enumerate() {
            let end = ROOTS.get(id + 1).copied().unwrap_or(END);
            if usize::from(language.id) != id
                || language.source_offset != ROOTS[id]
                || language.end != end
                || language.top.len() != 63
                || language.banks.len() != BANKS_PER_LANGUAGE
                || language.variants.bank_indices.len() != 26
                || language
                    .banks
                    .windows(2)
                    .any(|b| b[0].source_offset >= b[1].source_offset)
                || language
                    .banks
                    .iter()
                    .map(|b| b.programs.len())
                    .sum::<usize>()
                    != PROGRAMS_PER_LANGUAGE
            {
                return Err("Invalid script text language identity/shape/order".into());
            }
            let mut used = BTreeSet::new();
            let mut top = Vec::with_capacity(252);
            for (n, target) in language.top.iter().enumerate() {
                if matches!(target, TopTarget::Variants {}) != (n == 61) {
                    return Err("Invalid script text variant target position".into());
                }
                if let TopTarget::Bank { index } = target {
                    used.insert(*index);
                }
                let offset = language
                    .target_offset(target)
                    .and_then(|a| a.checked_sub(language.source_offset))
                    .ok_or("Invalid script text bank target")?;
                top.extend_from_slice(&offset.to_le_bytes());
            }
            parts.push((language.source_offset, top));
            let mut variants = Vec::with_capacity(104);
            for &index in &language.variants.bank_indices {
                used.insert(index);
                let offset = language
                    .banks
                    .get(usize::from(index))
                    .and_then(|b| b.source_offset.checked_sub(language.source_offset))
                    .ok_or("Invalid script text variant bank reference")?;
                variants.extend_from_slice(&offset.to_le_bytes());
            }
            if used.len() != BANKS_PER_LANGUAGE
                || used.iter().any(|&i| usize::from(i) >= BANKS_PER_LANGUAGE)
            {
                return Err("Script text unreferenced/invalid physical bank".into());
            }
            parts.push((language.variants.source_offset, variants));
            for bank in &language.banks {
                parts.push((bank.source_offset, bank.index_bytes()?));
                for program in &bank.programs {
                    parts.push((program.source_offset, program.source_bytes()?));
                }
            }
            if parts
                .iter()
                .rev()
                .take(
                    2 + language
                        .banks
                        .iter()
                        .map(|b| b.programs.len() + 1)
                        .sum::<usize>(),
                )
                .any(|(at, bytes)| {
                    *at < language.source_offset
                        || at.checked_add(bytes.len() as u32).is_none_or(|z| z > end)
                })
            {
                return Err("Script text field outside language range".into());
            }
        }
        if self
            .alignment
            .windows(2)
            .any(|v| v[0].source_offset >= v[1].source_offset)
        {
            return Err("Invalid script text alignment order".into());
        }
        for gap in &self.alignment {
            if !(1..=3).contains(&gap.bytes.len())
                || !self.languages.iter().any(|l| {
                    gap.source_offset >= l.source_offset
                        && gap
                            .source_offset
                            .checked_add(gap.bytes.len() as u32)
                            .is_some_and(|z| z <= l.end)
                })
            {
                return Err("Invalid script text alignment extent".into());
            }
            parts.push((gap.source_offset, gap.bytes.clone()));
        }
        parts.sort_by_key(|p| p.0);
        let mut cursor = ROOTS[0];
        for (offset, bytes) in &parts {
            if *offset != cursor || bytes.is_empty() {
                return Err("Script text physical ownership gap/overlap".into());
            }
            cursor = cursor
                .checked_add(bytes.len() as u32)
                .ok_or("Script text extent overflow")?;
        }
        if cursor != END {
            return Err("Script text incomplete physical source coverage".into());
        }
        Ok(parts)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.parts().map(|_| ())
    }
    pub fn validate_dependencies(
        &self,
        keys: &script_archive::ScriptArchive,
        font: &fonts::FontBank,
    ) -> Result<(), String> {
        self.validate()?;
        keys.validate()?;
        font.validate()?;
        for key in &keys.keys {
            for language in &self.languages {
                if language.top.get(usize::from(key.text_bank)).is_none() {
                    return Err("Script text key bank dependency outside projection".into());
                }
            }
        }
        for program in self
            .languages
            .iter()
            .flat_map(|l| &l.banks)
            .flat_map(|b| &b.programs)
        {
            for unit in &program.units {
                if let ProgramUnit::Glyph { index } = unit {
                    if font
                        .glyphs
                        .get(usize::from(*index))
                        .is_none_or(|g| g.index != *index)
                    {
                        return Err("Script text glyph dependency missing".into());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        let parts = self.parts()?;
        let mut spans = Vec::new();
        for language in &self.languages {
            let mut bytes = Vec::with_capacity((language.end - language.source_offset) as usize);
            for (_, part) in parts
                .iter()
                .filter(|(at, _)| *at >= language.source_offset && *at < language.end)
            {
                bytes.extend_from_slice(part);
            }
            spans.push((
                language.source_offset,
                bytes,
                format!("script-text-language:{}", language.id),
            ));
        }
        Ok(spans)
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if *self != Catalog::from_rom(rom)?.decoded {
            return Err("Script text differs from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: ScriptText,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Script text requires exact EUR source; USA unverified".into());
        }
        for &(at, length, expected) in READER_WINDOWS {
            if ffta_rom::digest(&rom[at..at + length]) != expected {
                return Err("Script text reader fingerprint mismatch".into());
            }
        }
        for &(at, expected) in ROOT_WORDS {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != expected {
                return Err("Script text literal root mismatch".into());
            }
        }
        let word = |a: u32| u32::from_le_bytes(rom[a as usize..a as usize + 4].try_into().unwrap());
        let half = |a: u32| u16::from_le_bytes(rom[a as usize..a as usize + 2].try_into().unwrap());
        let mut languages = Vec::new();
        let mut occupied = Vec::new();
        for (id, &root) in ROOTS.iter().enumerate() {
            let end = ROOTS.get(id + 1).copied().unwrap_or(END);
            let top: Vec<_> = (0..63).map(|n| word(root + n * 4)).collect();
            if top.iter().min() != Some(&252) {
                return Err("Script text top source domain mismatch".into());
            }
            let variant_at = root
                .checked_add(top[61])
                .filter(|&a| a >= root + 252 && a + 104 < end)
                .ok_or("Script text variant source overflow")?;
            let variants: Vec<_> = (0..26).map(|n| word(variant_at + n * 4)).collect();
            if variants.iter().min().and_then(|v| v.checked_sub(top[61])) != Some(104) {
                return Err("Script text variant index source length mismatch".into());
            }
            let mut heads = BTreeSet::new();
            for (i, relative) in top.iter().enumerate() {
                if i != 61 {
                    heads.insert(
                        root.checked_add(*relative)
                            .ok_or("Script text root overflow")?,
                    );
                }
            }
            for relative in &variants {
                heads.insert(
                    root.checked_add(*relative)
                        .ok_or("Script text variant overflow")?,
                );
            }
            if heads.len() != BANKS_PER_LANGUAGE
                || heads.iter().any(|&a| a < root + 252 || a + 2 >= end)
            {
                return Err("Script text physical bank source mismatch".into());
            }
            let heads: Vec<_> = heads.into_iter().collect();
            let mut structures = heads.clone();
            structures.extend([variant_at, end]);
            structures.sort_unstable();
            occupied.extend([(root, root + 252), (variant_at, variant_at + 104)]);
            let mut banks = Vec::new();
            for &at in &heads {
                let length = u32::from(half(at));
                let limit = *structures
                    .iter()
                    .find(|&&a| a > at)
                    .ok_or("Missing script text bank bound")?;
                if length < 4 || length > 4096 || length % 2 != 0 || at + length >= limit {
                    return Err("Invalid script text bank index source length".into());
                }
                let offsets: Vec<_> = (0..length / 2).map(|n| half(at + n * 2)).collect();
                if offsets.last() != Some(&0xffff) || offsets.windows(2).any(|v| v[0] >= v[1]) {
                    return Err("Invalid script text source index/sentinel".into());
                }
                occupied.push((at, at + length));
                let mut programs = Vec::new();
                for (i, &offset) in offsets[..offsets.len() - 1].iter().enumerate() {
                    let start = at + u32::from(offset);
                    let stop = if i + 1 < offsets.len() - 1 {
                        at + u32::from(offsets[i + 1])
                    } else {
                        limit
                    };
                    if start < at + length || start >= stop || stop > limit {
                        return Err("Script text program outside independent source bound".into());
                    }
                    let (program, used) =
                        Program::decode(&rom[start as usize..stop as usize], start)?;
                    occupied.push((start, start + used as u32));
                    programs.push(program);
                }
                banks.push(Bank {
                    source_offset: at,
                    programs,
                    sentinel: 0xffff,
                });
            }
            let index = |at| {
                heads
                    .binary_search(&at)
                    .map(|i| i as u8)
                    .map_err(|_| "Missing script text physical bank".to_owned())
            };
            let top = top
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    if i == 61 {
                        Ok(TopTarget::Variants {})
                    } else {
                        index(root + v).map(|index| TopTarget::Bank { index })
                    }
                })
                .collect::<Result<_, _>>()?;
            let bank_indices = variants
                .iter()
                .map(|&v| index(root + v))
                .collect::<Result<_, _>>()?;
            languages.push(Language {
                id: id as u8,
                source_offset: root,
                end,
                top,
                variants: VariantIndex {
                    source_offset: variant_at,
                    bank_indices,
                },
                banks,
            });
        }
        occupied.sort_unstable();
        let mut cursor = ROOTS[0];
        let mut alignment = Vec::new();
        for (at, end) in occupied {
            if at < cursor {
                return Err("Script text overlapping source fields".into());
            }
            if at > cursor {
                if at - cursor > 3 {
                    return Err("Unclassified script text source gap".into());
                }
                alignment.push(Alignment {
                    source_offset: cursor,
                    bytes: rom[cursor as usize..at as usize].to_vec(),
                });
            }
            cursor = end;
        }
        if cursor < END {
            if END - cursor > 3 {
                return Err("Unclassified final script text source gap".into());
            }
            alignment.push(Alignment {
                source_offset: cursor,
                bytes: rom[cursor as usize..END as usize].to_vec(),
            });
        }
        let decoded = ScriptText {
            languages,
            alignment,
        };
        let keys = script_archive::Catalog::from_rom(rom)?.decoded;
        let font = fonts::Catalog::from_rom(rom)?.decoded;
        decoded.validate_dependencies(&keys, &font)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if bytes != rom[offset as usize..offset as usize + bytes.len()] {
                return Err("Script text source roundtrip mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        Ok(Self { decoded, sources })
    }
}

const READER_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x9ab0,
        24,
        "2a30484d200d8909291d2d5820994dd1bdb743dcfa6fe627a6d16ed0a728898d",
    ),
    (
        0x9ad0,
        14,
        "804149dfe16e88cc48a98850340571e2a335cc8235baf3d57492d0c5459083ff",
    ),
    (
        0x9ae4,
        4,
        "705ba7738eca5891fd4176d4670b686772ca718863c9ae15346b2046bc23975e",
    ),
    (
        0x9aec,
        4,
        "b289ef1afd93328fef061442f39783d96f5d396352aa4ff2070ef57cab4c89f6",
    ),
    (
        0x9af4,
        4,
        "af9d17361705aca66fcef304a6ee7f30608d741f867922ea2e88f2f393c776d6",
    ),
    (
        0x9afc,
        6,
        "0434ea54a916567f2b98196d9c690d279533f8fb6e46e688f36d3c5112410114",
    ),
    (
        0x9b08,
        102,
        "2bcec439f226f4a19cedd456a6c0fadb86c7f8a39463f57a89dba375f1a4da51",
    ),
    (
        0x9b7c,
        86,
        "e66cb36ded7ecfe0367862f6c92feedc675d31e9e7ec4d28d4a2427dbdac7074",
    ),
    (
        0x9be0,
        8,
        "cf5c18a7c711459ce1364d28600b2999d06a9a8db7ae994e6ac7ea98ae9478e5",
    ),
    (
        0x118fe,
        38,
        "14bc977106cf55fb0098837dc8302dd0cf232ddb4728a56f4539e4d11e48400b",
    ),
    (
        0x11e28,
        18,
        "c370daa514087a53e99312bbbf3035b9fc2f7583e4bd2d6b09d732122959a0e4",
    ),
    (
        0x11e64,
        32,
        "56e7ffadb1594734e4c84a19293dfc3deccb07c22bf4da294a2d6328c2840b69",
    ),
    (
        0x13f8c,
        84,
        "843e99c1c7fce3defa0a5c17d56df9125ad80ad9fac88c9a3228eaa6727ec9ae",
    ),
    (
        0x1916c,
        50,
        "2d679edd4e7ba30f94c412ddcde86ed29127f2e15ecab255ee993534b74efb5c",
    ),
    (
        0x19244,
        64,
        "6ddc6cf60dacbff041d90e204e1a0c105c00dbd431bd08e3d65e7e69d3876bba",
    ),
    (
        0x193f0,
        38,
        "931b2e39ede1dda3f2ac4dc2fadbbfaa4fc9f553a83f60d4d942add5247d3a5f",
    ),
    (
        0x146474,
        110,
        "d8771e326b4bb4305660854e97a3dff06a0071aaf6ae9e08538f8d1930bc781a",
    ),
    (
        0x1464ec,
        12,
        "051cdac2b8b8cd6d45e983b76f083d2bc70942efbaf72f0e044b24f7bcc00183",
    ),
];
const ROOT_WORDS: &[(usize, u32)] = &[
    (0x9ae0, 0x08c5fbf0),
    (0x9ae8, 0x08cb7c48),
    (0x9af0, 0x08d10644),
    (0x9af8, 0x08c076f0),
    (0x9b04, 0x08bb064c),
    (0x9ac8, 0x02000000),
    (0x9acc, 0x00001f6e),
    (0x9b70, 0x02000000),
    (0x9b74, 0x00002192),
    (0x9b78, 0x00003c2a),
    (0x9bd4, 0x02000000),
    (0x9bd8, 0x00002192),
    (0x9bdc, 0x00003c2a),
];
