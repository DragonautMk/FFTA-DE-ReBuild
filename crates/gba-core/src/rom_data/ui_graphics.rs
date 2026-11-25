//! Raw custom UI graphics uploaded by original BG/OBJ callers.
//! Decoder-returned lengths, partial copies and live scratch work stay guest code.
use super::{obj_graphics, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
mod literal_sources;
mod roots;
pub mod table_selected;
pub(super) use literal_sources::bank_parts;
pub use obj_graphics::{DirectReference, Graphic};
pub use roots::{SelectorRoot, BANK_ROOTS, LOCALIZED_BANK_ROOTS, REFERENCES, SELECTOR_ROOTS};
/// Raw language-indexed alias of the banks used by the inline clan selector.
pub const CLAN_TABLE_ALIAS: u32 = 0x71e5e0;
const CLAN_TABLE_ALIAS_SHA256: &str =
    "1e9b0d6368856eecea893011919583717b2699959a2cee45f93c8b23f23c60c9";
const ADDITIONAL_TABLE: u32 = 0x3adfb0;
const ADDITIONAL_TABLE_LITERAL: u32 = 0x13c014;
const ADDITIONAL_READER: std::ops::Range<usize> = 0x13bf70..0x13bf8c;
const ADDITIONAL_READER_SHA256: &str =
    "93cd48df5ac308965d8504c3c6a5db2d79727bf4081ea7c799916021c77c6aed";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graphics {
    pub banks: Vec<Graphic>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selectors: Vec<Selector>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_obj_banks: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub direct_pointers: Vec<BankPointer>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankPointer {
    pub source_offset: u32,
    pub bank_offset: u32,
}
impl BankPointer {
    pub fn source_bytes(&self) -> Result<[u8; 4], String> {
        if self.source_offset & 3 != 0
            || self
                .source_offset
                .checked_add(4)
                .is_none_or(|end| end > 0x02000000)
            || self.bank_offset >= 0x02000000
        {
            return Err("Invalid custom UI pointer extent".into());
        }
        Ok((0x08000000 + self.bank_offset).to_le_bytes())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    pub id: String,
    pub pointers: [BankPointer; 5],
    pub fallback_variant: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_literal: Option<u32>,
}
impl Selector {
    pub fn select(&self, language: u8) -> Option<&BankPointer> {
        self.pointers.get(if language < 5 {
            language as usize
        } else {
            self.fallback_variant? as usize
        })
    }

    pub(super) fn table_literal_bytes(&self) -> [u8; 4] {
        (0x08000000 + self.pointers[0].source_offset).to_le_bytes()
    }
}
impl Graphics {
    pub fn validate(&self) -> Result<(), String> {
        if self.banks.is_empty()
            || self.banks.len() > 256
            || self.selectors.len() > 16
            || self.shared_obj_banks.len() > 256
            || self.direct_pointers.len() > 256
        {
            return Err("Invalid custom UI bank count".into());
        }
        for bank in &self.banks {
            bank.validate()?;
        }
        if self.banks.windows(2).any(|v| {
            v[0].source_offset as usize + v[0].encoded_source.len() > v[1].source_offset as usize
        }) {
            return Err("Unordered/overlapping custom UI sources".into());
        }
        if self.shared_obj_banks.windows(2).any(|v| v[0] >= v[1])
            || self.shared_obj_banks.iter().any(|at| {
                !obj_graphics::BANK_ROOTS.contains(at)
                    || self.banks.iter().any(|b| b.source_offset == *at)
            })
        {
            return Err("Invalid shared OBJ bank links".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut spans: Vec<_> = self
            .banks
            .iter()
            .map(|b| (b.source_offset, b.encoded_source.len()))
            .collect();
        for selector in &self.selectors {
            if selector.id.is_empty()
                || selector.id.len() > 64
                || !selector
                    .id
                    .bytes()
                    .all(|v| v.is_ascii_alphanumeric() || v == b'-' || v == b'_')
                || !ids.insert(&selector.id)
                || selector.fallback_variant.is_some_and(|i| i >= 5)
            {
                return Err("Invalid custom UI selector identity/fallback".into());
            }
            for pointer in &selector.pointers {
                if pointer.source_offset & 3 != 0
                    || pointer
                        .source_offset
                        .checked_add(4)
                        .is_none_or(|end| end > 0x02000000)
                    || !(self
                        .banks
                        .iter()
                        .any(|b| b.source_offset == pointer.bank_offset)
                        || self.shared_obj_banks.contains(&pointer.bank_offset))
                {
                    return Err("Invalid custom UI selector pointer/owner".into());
                }
                spans.push((pointer.source_offset, 4));
            }
            if let Some(literal) = selector.table_literal {
                if literal & 3 != 0
                    || literal.checked_add(4).is_none_or(|end| end > 0x02000000)
                    || selector.fallback_variant.is_some()
                    || selector
                        .pointers
                        .windows(2)
                        .any(|pair| pair[1].source_offset != pair[0].source_offset + 4)
                {
                    return Err("Invalid custom UI table literal".into());
                }
                spans.push((literal, 4));
            }
        }
        if self
            .direct_pointers
            .windows(2)
            .any(|pair| pair[0].source_offset >= pair[1].source_offset)
        {
            return Err("Unordered custom graphics source literals".into());
        }
        for pointer in &self.direct_pointers {
            pointer.source_bytes()?;
            if !self
                .banks
                .iter()
                .any(|b| b.source_offset == pointer.bank_offset)
                && !self.shared_obj_banks.contains(&pointer.bank_offset)
            {
                return Err("Custom graphics source literal is missing its bank".into());
            }
            spans.push((pointer.source_offset, 4));
        }
        if self.shared_obj_banks.iter().any(|at| {
            !self
                .selectors
                .iter()
                .flat_map(|selector| &selector.pointers)
                .chain(&self.direct_pointers)
                .any(|p| p.bank_offset == *at)
        }) {
            return Err("Unused shared OBJ bank link".into());
        }
        spans.sort_unstable();
        if spans
            .windows(2)
            .any(|v| v[0].0 as usize + v[0].1 > v[1].0 as usize)
        {
            return Err("Overlapping custom UI selectors/banks".into());
        }
        Ok(())
    }
    pub fn validate_shared_obj(&self, obj: &obj_graphics::Graphics) -> Result<(), String> {
        if self
            .shared_obj_banks
            .iter()
            .any(|at| !obj.banks.iter().any(|b| b.source_offset == *at))
        {
            return Err("Custom UI selector is missing its shared OBJ owner".into());
        }
        Ok(())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        let has_table = self
            .selectors
            .iter()
            .any(|selector| selector.table_literal.is_some());
        let layout = if !self.direct_pointers.is_empty() {
            SourceLayout::DirectLiterals
        } else if has_table {
            SourceLayout::AdditionalTable
        } else {
            SourceLayout::Legacy
        };
        if self != &Catalog::from_rom_with_layout(rom, layout)?.decoded {
            return Err("Custom UI banks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn half(rom: &[u8], at: u32) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        rom.get(at as usize..at as usize + 2)
            .ok_or("Truncated custom UI source instruction")?
            .try_into()
            .unwrap(),
    ))
}
fn call(rom: &[u8], at: u32, target: u32) -> Result<(), String> {
    let (h, l) = (half(rom, at)?, half(rom, at + 2)?);
    let d = (((u32::from(h & 2047) << 12) | (u32::from(l & 2047) << 1)) as i32) << 9 >> 9;
    if h & 0xf800 != 0xf000
        || l & 0xf800 != 0xf800
        || at.wrapping_add(4).wrapping_add_signed(d) != target
    {
        return Err("Custom UI original consumer mismatch".into());
    }
    Ok(())
}
pub struct Catalog {
    pub decoded: Graphics,
    pub sources: Vec<SourceRange>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceLayout {
    Legacy,
    AdditionalTable,
    DirectLiterals,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        Self::from_rom_with_layout(rom, SourceLayout::Legacy)
    }

    pub(super) fn from_rom_with_layout(rom: &[u8], layout: SourceLayout) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Custom UI graphics require exact EUR ROM; USA pending".into());
        }
        for r in REFERENCES {
            let h = half(rom, r.reader)?;
            let word = u32::from_le_bytes(
                rom.get(r.literal as usize..r.literal as usize + 4)
                    .ok_or("Truncated custom UI literal")?
                    .try_into()
                    .unwrap(),
            );
            if h & 0xf800 != 0x4800
                || ((r.reader + 4) & !3) + u32::from(h & 255) * 4 != r.literal
                || word != 0x08000000 + r.source_offset
            {
                return Err("Custom UI root literal mismatch".into());
            }
            call(rom, r.call, 0x54a0)?;
            call(rom, r.transfer_call, r.transfer_target)?;
        }
        let mut banks = BANK_ROOTS
            .iter()
            .map(|&at| {
                Graphic::decode(
                    rom.get(at as usize..).ok_or("Truncated custom UI bank")?,
                    at,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut selectors = SELECTOR_ROOTS
            .iter()
            .map(|root| {
                let pointers = std::array::from_fn(|i| BankPointer {
                    source_offset: root.literals[i],
                    bank_offset: 0,
                });
                let mut selector = Selector {
                    id: root.id.into(),
                    pointers,
                    fallback_variant: root.fallback,
                    table_literal: None,
                };
                for i in 0..5 {
                    let h = half(rom, root.readers[i])?;
                    let at = root.literals[i] as usize;
                    if h & 0xf800 != 0x4800
                        || ((root.readers[i] + 4) & !3) + u32::from(h & 255) * 4 != root.literals[i]
                        || u32::from_le_bytes(
                            rom[(root.jump_table + i as u32 * 4) as usize
                                ..(root.jump_table + i as u32 * 4 + 4) as usize]
                                .try_into()
                                .unwrap(),
                        ) != 0x08000000 + root.readers[i]
                    {
                        return Err("Custom UI original language branch mismatch".into());
                    }
                    selector.pointers[i].bank_offset =
                        u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                            .checked_sub(0x08000000)
                            .ok_or("Invalid custom UI ROM pointer")?;
                }
                Ok(selector)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let tables = table_selected::Catalog::from_rom(rom)?.decoded;
        banks.extend(tables.banks);
        selectors.extend(tables.tables.into_iter().map(|table| Selector {
            id: format!("table-{:06x}", table.source_offset),
            pointers: table.pointers,
            fallback_variant: None,
            table_literal: None,
        }));
        let alias_bytes = rom
            .get(CLAN_TABLE_ALIAS as usize..CLAN_TABLE_ALIAS as usize + 20)
            .ok_or("Truncated clan table alias")?;
        if ffta_rom::digest(alias_bytes) != CLAN_TABLE_ALIAS_SHA256 {
            return Err("Clan table alias source mismatch".into());
        }
        let clan = selectors
            .iter()
            .find(|selector| selector.id == "clan")
            .ok_or("Missing clan selector")?;
        let mut alias = Selector {
            id: "clan-table-alias".into(),
            pointers: std::array::from_fn(|language| BankPointer {
                source_offset: CLAN_TABLE_ALIAS + language as u32 * 4,
                bank_offset: 0,
            }),
            fallback_variant: None,
            table_literal: None,
        };
        for (language, pointer) in alias.pointers.iter_mut().enumerate() {
            pointer.bank_offset = u32::from_le_bytes(
                alias_bytes[language * 4..language * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
            .checked_sub(0x08000000)
            .ok_or("Invalid clan table alias pointer")?;
            if pointer.bank_offset != clan.pointers[language].bank_offset {
                return Err("Clan table alias bank mismatch".into());
            }
        }
        selectors.push(alias);
        if layout != SourceLayout::Legacy {
            if ffta_rom::digest(&rom[ADDITIONAL_READER]) != ADDITIONAL_READER_SHA256 {
                return Err("Additional custom UI table reader fingerprint mismatch".into());
            }
            let selector = Selector {
                id: format!("table-{ADDITIONAL_TABLE:06x}"),
                pointers: std::array::from_fn(|language| {
                    let source_offset = ADDITIONAL_TABLE + 4 * language as u32;
                    let at = source_offset as usize;
                    let field = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
                    BankPointer {
                        source_offset,
                        bank_offset: field.wrapping_sub(0x08000000),
                    }
                }),
                fallback_variant: None,
                table_literal: Some(ADDITIONAL_TABLE_LITERAL),
            };
            let at = ADDITIONAL_TABLE_LITERAL as usize;
            if rom[at..at + 4] != selector.table_literal_bytes() {
                return Err("Additional custom UI table root mismatch".into());
            }
            for pointer in &selector.pointers {
                let at = pointer.bank_offset as usize;
                let bank = Graphic::decode(
                    rom.get(at..)
                        .ok_or("Additional custom UI bank outside the ROM")?,
                    pointer.bank_offset,
                )?;
                if bank.words.len() * 4 != 1664 {
                    return Err("Additional custom UI bank output size mismatch".into());
                }
                banks.push(bank);
            }
            selectors.push(selector);
        }
        let mut shared_obj_banks = vec![0xb7f874, 0xb80140];
        let mut direct_pointers = Vec::new();
        if layout == SourceLayout::DirectLiterals {
            direct_pointers = literal_sources::pointers(rom)?;
            direct_pointers.sort_by_key(|pointer| pointer.source_offset);
            for pointer in &direct_pointers {
                let at = pointer.bank_offset;
                if obj_graphics::BANK_ROOTS.contains(&at) {
                    shared_obj_banks.push(at);
                } else if !banks.iter().any(|bank| bank.source_offset == at) {
                    banks.push(Graphic::decode(
                        rom.get(at as usize..).ok_or("Truncated custom UI bank")?,
                        at,
                    )?);
                }
            }
            shared_obj_banks.sort_unstable();
            shared_obj_banks.dedup();
        }
        banks.sort_by_key(|bank| bank.source_offset);
        let decoded = Graphics {
            banks,
            selectors,
            shared_obj_banks,
            direct_pointers,
        };
        decoded.validate()?;
        let mut sources = Vec::new();
        for bank in &decoded.banks {
            sources.extend(bank_parts(bank)?.map(|(offset, bytes)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(bytes),
                role: format!("ui-custom-bank/{offset:08x}"),
            }));
        }
        for selector in &decoded.selectors {
            for (language, pointer) in selector.pointers.iter().enumerate() {
                sources.push(SourceRange {
                    offset: pointer.source_offset,
                    length: 4,
                    sha256: ffta_rom::digest(&pointer.source_bytes()?),
                    role: format!("ui-custom-selector/{}/{language}", selector.id),
                });
            }
        }
        sources.extend(decoded.selectors.iter().filter_map(|selector| {
            selector.table_literal.map(|offset| SourceRange {
                offset,
                length: 4,
                sha256: ffta_rom::digest(&selector.table_literal_bytes()),
                role: format!("ui-custom-selector/{}/table-literal", selector.id),
            })
        }));
        for pointer in &decoded.direct_pointers {
            sources.push(SourceRange {
                offset: pointer.source_offset,
                length: 4,
                sha256: ffta_rom::digest(&pointer.source_bytes()?),
                role: format!("ui-custom-literal/{:08x}", pointer.source_offset),
            });
        }
        Ok(Self { decoded, sources })
    }
}
