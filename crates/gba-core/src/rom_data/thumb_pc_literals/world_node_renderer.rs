//! Remaining literal words of the original European world-node renderer.

use super::LiteralWord;
use crate::ffta_rom;

const RENDERER_SHA256: &str = "defcc9d09e93c07b0ac5b06ffe350753755c28a092561bea53f5d9250c77fd6b";
const WORDS_SHA256: &str = "4e878c9de7a225ea55fa5792342c4150ae7b2ef0cd32c73dab5a77a955d170e9";

// Source offset, original Thumb load offset and opcode in source order.
const LOADS: [(u32, u32, u16); 8] = [
    (0x0386ac, 0x038684, 0x4b09),
    (0x0386b0, 0x0386a0, 0x4903),
    (0x0387f0, 0x03879e, 0x4814),
    (0x0387f4, 0x0387be, 0x4a0d),
    (0x0387f8, 0x0387c2, 0x4a0d),
    (0x0387fc, 0x0387d0, 0x480a),
    (0x038800, 0x0387d4, 0x480a),
    (0x038804, 0x0387de, 0x4809),
];

pub(super) fn validate(words: &[LiteralWord]) -> Result<(), String> {
    if words.is_empty() {
        return Ok(());
    }
    if words.len() != LOADS.len()
        || words.iter().zip(LOADS).any(|(word, (source, reader, _))| {
            word.source_offset != source || word.reader_offset != reader
        })
    {
        return Err("Invalid world-node renderer literal sources or readers".into());
    }
    let bytes: Vec<_> = words
        .iter()
        .flat_map(|word| word.value.to_le_bytes())
        .collect();
    if ffta_rom::digest(&bytes) != WORDS_SHA256 {
        return Err("World-node renderer literal fingerprint mismatch".into());
    }
    Ok(())
}

pub(super) fn extract(rom: &[u8]) -> Result<Vec<LiteralWord>, String> {
    if ffta_rom::digest(&rom[0x038588..0x03885c]) != RENDERER_SHA256 {
        return Err("World-node renderer instruction fingerprint mismatch".into());
    }
    let words = LOADS
        .into_iter()
        .map(|(source, reader, opcode)| LiteralWord::from_thumb_load(rom, source, reader, opcode))
        .collect::<Result<Vec<_>, String>>()?;
    validate(&words)?;
    Ok(words)
}
