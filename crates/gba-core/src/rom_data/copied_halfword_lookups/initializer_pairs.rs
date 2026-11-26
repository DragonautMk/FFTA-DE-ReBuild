//! EUR two-word initializer copies with shared physical sources.
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCES: [(u32, &str); 10] = [
    (0x14d1e8, "1bf8fab7249d63008c7b4f72cbab8e5c63ab7aeec769733408609a30ed01ff0c"),
    (0x14d51c, "1bf8fab7249d63008c7b4f72cbab8e5c63ab7aeec769733408609a30ed01ff0c"),
    (0x14d524, "dcaf7c200e2cbb3ab1556f890d60b802de130089fb5dcba7e7861a16c8449fa1"),
    (0x14d79c, "f9593cea0c4b0c30c580d4db38fb7580bdbf09283dcd9e02113f11b7a518a0ed"),
    (0x14d7a4, "616759c0821edb6501e13f37764e403ec550d139799a3cd0cc1f97b8cccd1d5c"),
    (0x14d7c8, "02c03d236f1145db759e2393d71017d9a9930c9f7ff283e1efdf8f839f8b6e68"),
    (0x14d83c, "f9593cea0c4b0c30c580d4db38fb7580bdbf09283dcd9e02113f11b7a518a0ed"),
    (0x14d844, "616759c0821edb6501e13f37764e403ec550d139799a3cd0cc1f97b8cccd1d5c"),
    (0x14d98c, "09dc8a419cc2f56048a0a5493f511c99db5dba0dee01e860baa0950d2b67cb39"),
    (0x14d9c8, "cdbc4ced9ba9652e198383349898a6990c13ea09a2c2dc57037a5e04a8351484"),
];

#[derive(Clone, Copy)]
pub struct Reader { pub entry: u32, pub literal: u32, pub source: u32, pub base: u8, pub second: u8, pub stack_offset: u16, pub code_sha256: &'static str }

pub const READERS: [Reader; 13] = [
    Reader { entry: 0x00d01c, literal: 0x00d0a8, source: 0x14d1e8, base: 1, second: 2, stack_offset: 0, code_sha256: "3794bee83bdcc01dd1096e80a5a9eed74f8d7d9d817c4535166a094118a1d7ee" },
    Reader { entry: 0x00d0ea, literal: 0x00d174, source: 0x14d1e8, base: 1, second: 2, stack_offset: 0, code_sha256: "3794bee83bdcc01dd1096e80a5a9eed74f8d7d9d817c4535166a094118a1d7ee" },
    Reader { entry: 0x00d1be, literal: 0x00d24c, source: 0x14d1e8, base: 1, second: 2, stack_offset: 0, code_sha256: "2e2b5c9caa57dc2d49628965863459ae03d591f0f1eadf149b0003539d6f7d63" },
    Reader { entry: 0x00d296, literal: 0x00d320, source: 0x14d1e8, base: 1, second: 2, stack_offset: 0, code_sha256: "3794bee83bdcc01dd1096e80a5a9eed74f8d7d9d817c4535166a094118a1d7ee" },
    Reader { entry: 0x0dd174, literal: 0x0dd1d0, source: 0x14d51c, base: 0, second: 1, stack_offset: 20, code_sha256: "83cb71e794a2b84932cdcf8a379ca5a6c4c92a27682b9222f9b5a8ceccd6d408" },
    Reader { entry: 0x0dd790, literal: 0x0dd7d4, source: 0x14d524, base: 0, second: 1, stack_offset: 0, code_sha256: "717b7e91964d45d8b1a3104e1c89dfdc5a0cfb0df82107b24c7963f3697123da" },
    Reader { entry: 0x101a00, literal: 0x101a28, source: 0x14d79c, base: 0, second: 1, stack_offset: 4, code_sha256: "3799515648a90a91bf51b9c2e51e8cde87875ddd66310201c0814e907ddef2eb" },
    Reader { entry: 0x101a0a, literal: 0x101a2c, source: 0x14d7a4, base: 0, second: 1, stack_offset: 12, code_sha256: "a3b23f2f869335f0b3dfd9cd549c753e9082a66acb5cca3700a8f7d50397a16c" },
    Reader { entry: 0x103384, literal: 0x10343c, source: 0x14d7c8, base: 0, second: 1, stack_offset: 20, code_sha256: "b5588ac8d703a28b4b74f013963563883aae6c5dd04f232fe25bb4db4cfb4b24" },
    Reader { entry: 0x10931e, literal: 0x109350, source: 0x14d83c, base: 1, second: 2, stack_offset: 16, code_sha256: "54c0c0d436e3903b2da9420a97397b82b95fb569b437f948bee70ab11b0be752" },
    Reader { entry: 0x109328, literal: 0x109354, source: 0x14d844, base: 1, second: 2, stack_offset: 24, code_sha256: "c58b64a1001a2035c5f5df1bf6859313fac48028070733b00ca942d7a7c60bc7" },
    Reader { entry: 0x11f68e, literal: 0x11f6fc, source: 0x14d98c, base: 0, second: 1, stack_offset: 20, code_sha256: "c562e411e59f7d4e48077ab09e76f4be9468df828cbee93c5f29a5747dddb2b4" },
    Reader { entry: 0x12479e, literal: 0x124828, source: 0x14d9c8, base: 0, second: 1, stack_offset: 44, code_sha256: "931088935389d3fabc8cf426b98d61ba05a91ffa9090d4a0cc45240a92e86042" },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair { pub source_offset: u32, pub words: [u32; 2] }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairBank { pub pairs: Vec<Pair> }

impl PairBank {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        for reader in READERS {
            let at = reader.entry as usize;
            let op = u16::from_le_bytes(rom[at..at+2].try_into().unwrap());
            let pointer = u32::from_le_bytes(rom[reader.literal as usize..reader.literal as usize+4].try_into().unwrap());
            let first = 0x6800 | (1 << 6) | u16::from(reader.base) << 3 | u16::from(reader.second);
            let second = 0x6800 | u16::from(reader.base) << 3 | u16::from(reader.base);
            let low = 0x9000 | u16::from(reader.base) << 8 | reader.stack_offset / 4;
            let high = 0x9000 | u16::from(reader.second) << 8 | (reader.stack_offset / 4 + 1);
            if op & 0xf800 != 0x4800 || (op >> 8) & 7 != u16::from(reader.base)
                || ((reader.entry + 4) & !3) + u32::from(op & 255) * 4 != reader.literal
                || pointer != 0x0800_0000 + reader.source || reader.stack_offset % 4 != 0
                || reader.base == reader.second || ffta_rom::digest(&rom[at..at+10]) != reader.code_sha256
                || [first, second, low, high].iter().enumerate().any(|(i, &expected)| {
                    u16::from_le_bytes(rom[at+2+i*2..at+4+i*2].try_into().unwrap()) != expected
                }) { return Err("Two-word initializer original reader mismatch".into()); }
        }
        let mut pairs = Vec::with_capacity(SOURCES.len());
        for (source, expected) in SOURCES {
            let at = source as usize;
            if ffta_rom::digest(&rom[at..at+8]) != expected { return Err("Two-word initializer source mismatch".into()); }
            pairs.push(Pair { source_offset: source, words: std::array::from_fn(|i| u32::from_le_bytes(rom[at+4*i..at+4*i+4].try_into().unwrap())) });
        }
        Ok(Self { pairs })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.pairs.len() != SOURCES.len() || self.pairs.iter().zip(SOURCES).any(|(pair, (source, _))| pair.source_offset != source) {
            return Err("Invalid two-word initializer sources".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>, &'static str)> {
        let mut spans = Vec::with_capacity(SOURCES.len() + READERS.len());
        for pair in &self.pairs { spans.push((pair.source_offset, pair.words.iter().flat_map(|word| word.to_le_bytes()).collect(), "initializer-pair-words")); }
        for reader in READERS { spans.push((reader.literal, (0x0800_0000u32 + reader.source).to_le_bytes().to_vec(), "initializer-pair-root")); }
        spans
    }

    pub fn byte_at(&self, offset: u32) -> Option<(u8, bool)> {
        for pair in &self.pairs {
            if let Some(n) = offset.checked_sub(pair.source_offset) {
                if n < 8 { return Some((pair.words[(n/4) as usize].to_le_bytes()[(n%4) as usize], false)); }
            }
        }
        for reader in READERS {
            if let Some(n) = offset.checked_sub(reader.literal) {
                if n < 4 { return Some(((0x0800_0000u32 + reader.source).to_le_bytes()[n as usize], true)); }
            }
        }
        None
    }
}
