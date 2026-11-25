//! The addresses copied by the European hardware-scroll and shadow readers.
use super::{invalid, rom_word, source_digest, ByteSource, SourceSpan};
use serde::{Deserialize, Serialize};

const SOURCE_SHA256: &str = "fbd1d3d17724fc3215d2df70e05d65852130acd5fec7a8c190401a892fad6fcc";
const PAIR_ROOTS: [u32; 4] = [0x0dca54, 0x0dcdac, 0x0dd008, 0x0dd110];
const ALIAS_ROOT: u32 = 0x0f115c;
const IO_TABLE: u32 = 0x14d4cc;
const SHADOW_TABLE: u32 = 0x14d4ec;
const SHADOW_ALIAS: u32 = 0x14d6e0;
const ROM_BASE: u32 = 0x0800_0000;
const IO_REGISTER_BASE: u32 = 0x0400_0010;
const SHADOW_FIELD_BASE: u32 = 0x0300_094a;
const ENTRIES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrollRegisterTables {
    pub caller_roots: [[u32; 2]; PAIR_ROOTS.len()],
    pub alias_root: u32,
    pub io_registers: [u32; ENTRIES],
    pub shadow_fields: [u32; ENTRIES],
}

fn word_byte(words: &[u32], base: u32, offset: u32) -> Option<u8> {
    let relative = offset.checked_sub(base)? as usize;
    Some(words.get(relative / 4)?.to_le_bytes()[relative % 4])
}

impl ScrollRegisterTables {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let value = Self {
            caller_roots: PAIR_ROOTS.map(|at| [rom_word(rom, at), rom_word(rom, at + 4)]),
            alias_root: rom_word(rom, ALIAS_ROOT),
            io_registers: std::array::from_fn(|i| rom_word(rom, IO_TABLE + 4 * i as u32)),
            shadow_fields: std::array::from_fn(|i| rom_word(rom, SHADOW_TABLE + 4 * i as u32)),
        };
        let alias: [u32; ENTRIES] =
            std::array::from_fn(|i| rom_word(rom, SHADOW_ALIAS + 4 * i as u32));
        if value.shadow_fields != alias || source_digest(value.source_spans()?) != SOURCE_SHA256 {
            return Err("Scroll register table source fingerprint mismatch".into());
        }
        Ok(value)
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let io_registers: [u32; ENTRIES] = std::array::from_fn(|i| IO_REGISTER_BASE + 2 * i as u32);
        let shadow_fields: [u32; ENTRIES] =
            std::array::from_fn(|i| SHADOW_FIELD_BASE + 2 * i as u32);
        if self.alias_root != ROM_BASE + SHADOW_ALIAS
            || self.io_registers != io_registers
            || self.shadow_fields != shadow_fields
        {
            return Err(invalid());
        }
        let mut spans = Vec::with_capacity(PAIR_ROOTS.len() + 4);
        for (base, roots) in PAIR_ROOTS.into_iter().zip(&self.caller_roots) {
            if *roots != [ROM_BASE + IO_TABLE, ROM_BASE + SHADOW_TABLE] {
                return Err(invalid());
            }
            let bytes = roots.iter().flat_map(|word| word.to_le_bytes()).collect();
            spans.push((base, bytes, "scroll-register-table-roots".into()));
        }
        spans.push((
            ALIAS_ROOT,
            self.alias_root.to_le_bytes().to_vec(),
            "scroll-shadow-alias-root".into(),
        ));
        for (offset, table, role) in [
            (IO_TABLE, &self.io_registers, "scroll-io-registers"),
            (SHADOW_TABLE, &self.shadow_fields, "scroll-shadow-fields"),
            (
                SHADOW_ALIAS,
                &self.shadow_fields,
                "scroll-shadow-table-alias",
            ),
        ] {
            spans.push((
                offset,
                table.iter().flat_map(|word| word.to_le_bytes()).collect(),
                role.into(),
            ));
        }
        Ok(spans)
    }

    pub(super) fn byte(&self, offset: u32) -> Option<(u8, ByteSource)> {
        for (base, roots) in PAIR_ROOTS.into_iter().zip(&self.caller_roots) {
            if let Some(value) = word_byte(roots, base, offset) {
                return Some((value, ByteSource::Literal));
            }
        }
        if let Some(value) = word_byte(std::slice::from_ref(&self.alias_root), ALIAS_ROOT, offset) {
            return Some((value, ByteSource::Literal));
        }
        for (base, words) in [
            (IO_TABLE, &self.io_registers),
            (SHADOW_TABLE, &self.shadow_fields),
            (SHADOW_ALIAS, &self.shadow_fields),
        ] {
            if let Some(value) = word_byte(words, base, offset) {
                return Some((value, ByteSource::Payload));
            }
        }
        None
    }
}

pub(super) fn source_page(index: usize) -> bool {
    // The roots and the three tables each fit entirely within their start page.
    let page = index >> 12;
    PAIR_ROOTS
        .into_iter()
        .chain([ALIAS_ROOT, IO_TABLE, SHADOW_TABLE, SHADOW_ALIAS])
        .any(|offset| offset as usize >> 12 == page)
}
