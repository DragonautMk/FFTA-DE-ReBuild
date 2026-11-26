//! EUR literal-rooted three-word initializer copy patterns.
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

/// Copy instruction, pointer literal, physical source, word count and source digest.
pub const SPECS: [(u32, u32, u32, usize, &str); 14] = [
    (0x00cdb6, 0x00ce0c, 0x14d1b0, 3, "3efc99eec10f70accad47eee8e307022fd81b545b06b207daf73a38a77d3ed23"),
    (0x00cf18, 0x00cf5c, 0x14d1d4, 3, "d1932f66651945049f739349814acf4b96c801f89b4efe0ee4d8c7efbcda67b6"),
    (0x01e7ba, 0x01e864, 0x14d2cc, 3, "fdf761171208018c2ffe103773592b07f7c74d91a107be7c2f942552a7dee364"),
    (0x01e900, 0x01e984, 0x14d2bc, 3, "c3e0f38a7aa95a60c251498ee8b9d401ceb12dc65c57f1fa12ce2efd783f4f1d"),
    (0x08f3c8, 0x08f630, 0x14d38c, 3, "9514bcc9fa373207d4e237df683554e01ae48fbc2825829a968f75dd7eb729a0"),
    (0x08f3d2, 0x08f634, 0x14d398, 3, "db2bf23ab8c556665b772ee3399c39ab8b2ca8b9e07cf6382cb3075da00b7a88"),
    (0x0dce8e, 0x0dcf20, 0x14d50c, 3, "7787744e4d2bff04e17778808c8978fb68590c3914db6383145401b3451d918e"),
    (0x0ded40, 0x0ded8c, 0x14d52c, 12, "037c286339bf19cd6e9989950ed981df46422505b4eb52aefea566cca9d3d02a"),
    (0x0fb348, 0x0fb3c8, 0x14d744, 3, "9c6781eeb1063de95c061b8b43a92ea2668d344d2c8b6cf3c8eef21994e38405"),
    (0x101794, 0x101810, 0x14d78c, 3, "7411b1af8e0a9cda6b7e33b01424c806145f3675050c56bccd9e3d5d5843699d"),
    (0x101b30, 0x101c00, 0x14d7ac, 3, "6f3cb74dafa4fcc77bf4dcc81b7c66280a8e0e7a2724347cedb933fab09f0127"),
    (0x10d770, 0x10d798, 0x14d8dc, 3, "d4a79e23ddcdb67a72afcd69bf5029183408915f3a8bf56707dc42d0f5638e53"),
    (0x10e474, 0x10e584, 0x14d8ec, 3, "74c77845bfdbdf8868982d08ae8b5ec49fae3423cf4cc9df4fe073a76585e1f2"),
    (0x11d50c, 0x11d588, 0x14d970, 3, "9c6781eeb1063de95c061b8b43a92ea2668d344d2c8b6cf3c8eef21994e38405"),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Initializer {
    pub source_offset: u32,
    pub words: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitializerBank {
    pub records: Vec<Initializer>,
}

impl InitializerBank {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let mut records = Vec::with_capacity(SPECS.len());
        for &(site, literal, source, count, expected) in &SPECS {
            let op = u16::from_le_bytes(rom[site as usize - 2..site as usize].try_into().unwrap());
            let root = u32::from_le_bytes(rom[literal as usize..literal as usize + 4].try_into().unwrap());
            if op & 0xff00 != 0x4800 || ((site + 2) & !3) + u32::from(op & 255) * 4 != literal
                || root != 0x0800_0000 + source || count % 3 != 0
                || (0..count / 3).any(|i| &rom[site as usize + i * 4..site as usize + i * 4 + 4] != [0x1c, 0xc8, 0x1c, 0xc1])
                || ffta_rom::digest(&rom[source as usize..source as usize + count * 4]) != expected {
                return Err("Initializer copy source or original reader mismatch".into());
            }
            let words = rom[source as usize..source as usize + count * 4].chunks_exact(4)
                .map(|word| u32::from_le_bytes(word.try_into().unwrap())).collect();
            records.push(Initializer { source_offset: source, words });
        }
        Ok(Self { records })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.records.len() != SPECS.len() || self.records.iter().zip(SPECS).any(|(record, (_, _, source, count, _))|
            record.source_offset != source || record.words.len() != count) {
            return Err("Invalid initializer copy records".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>, &'static str)> {
        let mut spans = Vec::with_capacity(SPECS.len() * 2);
        for (record, (_, literal, source, _, _)) in self.records.iter().zip(SPECS) {
            spans.push((source, record.words.iter().flat_map(|word| word.to_le_bytes()).collect(), "initializer-copy-words"));
            spans.push((literal, (0x0800_0000u32 + source).to_le_bytes().to_vec(), "initializer-copy-root"));
        }
        spans
    }

    pub fn byte_at(&self, offset: u32) -> Option<(u8, bool)> {
        for (record, (_, literal, source, _, _)) in self.records.iter().zip(SPECS) {
            if let Some(n) = offset.checked_sub(source) {
                if let Some(word) = record.words.get((n / 4) as usize) { return Some((word.to_le_bytes()[(n % 4) as usize], false)); }
            }
            if let Some(n) = offset.checked_sub(literal) {
                if n < 4 { return Some(((0x0800_0000u32 + source).to_le_bytes()[n as usize], true)); }
            }
        }
        None
    }
}
