//! Source-base loads for the declared European text catalogs and name keyboard.

use super::LiteralWord;
use crate::rom_data::{name_keyboard, script_text, text_catalogs};

const ARCHIVE_LOADS: [(u32, u16); 5] = [
    (0x4c, 0x4801),
    (0x44, 0x4800),
    (0x2a, 0x4801),
    (0x34, 0x4800),
    (0x3c, 0x4800),
];
const POINTER_LOADS: [(u32, u16); 5] = [
    (0x50, 0x4802),
    (0x48, 0x4800),
    (0x2e, 0x4801),
    (0x38, 0x4800),
    (0x40, 0x4800),
];
const PREVIOUSLY_OWNED: [u32; 5] = [0x09b04, 0x165e4, 0x16704, 0x167c4, 0x1693c];

struct RootLoad {
    word: LiteralWord,
    opcode: u16,
}

impl RootLoad {
    fn new(reader_offset: u32, opcode: u16, root: u32) -> Self {
        Self {
            word: LiteralWord {
                source_offset: ((reader_offset + 4) & !3) + 4 * u32::from(opcode & 255),
                reader_offset,
                value: 0x08000000 + root,
            },
            opcode,
        }
    }
}

fn append_selector(specs: &mut Vec<RootLoad>, entry: u32, loads: [(u32, u16); 5], roots: [u32; 5]) {
    for ((offset, opcode), root) in loads.into_iter().zip(roots) {
        specs.push(RootLoad::new(entry + offset, opcode, root));
    }
}

fn root_loads() -> Vec<RootLoad> {
    let mut specs = Vec::with_capacity(197);
    for (entry, roots) in text_catalogs::ARCHIVE_READERS
        .into_iter()
        .zip(text_catalogs::ARCHIVE_ROOTS)
    {
        append_selector(&mut specs, entry - 0x08000000, ARCHIVE_LOADS, roots);
    }
    for (entry, roots) in text_catalogs::POINTER_READERS
        .into_iter()
        .zip(text_catalogs::POINTER_ROOTS)
    {
        append_selector(&mut specs, entry - 0x08000000, POINTER_LOADS, roots);
    }
    append_selector(&mut specs, 0x09ab0, ARCHIVE_LOADS, script_text::ROOTS);
    specs.push(RootLoad::new(0x12e940, 0x4910, name_keyboard::ROOT));
    specs.push(RootLoad::new(0x12f0c2, 0x490f, name_keyboard::ROOT));
    specs.retain(|spec| !PREVIOUSLY_OWNED.contains(&spec.word.source_offset));
    specs.sort_by_key(|spec| spec.word.source_offset);
    specs
}

pub(super) fn validate(words: &[LiteralWord]) -> Result<(), String> {
    if words.is_empty() {
        return Ok(());
    }
    let expected = root_loads();
    if words.len() != expected.len()
        || words.iter().zip(expected).any(|(word, spec)| {
            word.source_offset != spec.word.source_offset
                || word.reader_offset != spec.word.reader_offset
        })
    {
        return Err("Invalid localized text root words or readers".into());
    }
    Ok(())
}

pub(super) fn extract(rom: &[u8]) -> Result<Vec<LiteralWord>, String> {
    root_loads()
        .into_iter()
        .map(|spec| {
            let reader = spec.word.reader_offset as usize;
            let source = spec.word.source_offset as usize;
            if rom[reader..reader + 2] != spec.opcode.to_le_bytes()
                || rom[source..source + 4] != spec.word.value.to_le_bytes()
            {
                return Err("Localized text root or original load mismatch".into());
            }
            Ok(spec.word)
        })
        .collect()
}
