//! Selected EUR Firaga original literals and stack initialization fields.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSETS: [u32; 6] = [0xd8660, 0x14d480, 0x14d488, 0x14d490, 0x14d498, 0x14d49c];
pub const SOURCE_LENGTHS: [usize; 6] = [20, 8, 8, 8, 4, 16];
const HASHES: [&str; 6] = [
    "e48fd7e007351ec18497b87c3f1334d02b3538109d3cf817f4a519620ee0a76b",
    "be8be6e783d2cc52d66243fdcf89b8f32071b1d8afbbb226b39a13c54f3e6bbf",
    "8276e8d6138138dd8b34ec3a35ad6454390e8f8887d14ad838dbbb08be2fb449",
    "2674cff3ba9fb52816f87b54ff2769646fd5864e912d766b97c47635ac5f113a",
    "3f4f8d0598e76a99ccb6af56a6f28d2eb3dd1cdcaa479bd907909d2bc9d52cf4",
    "baed642339816affb3fe8719792d0e4ce82f12db72b7373d244eaa65445800fe",
];
const ROLES: [&str; 6] = [
    "firaga-stack-root-literals",
    "firaga-stack-words-0",
    "firaga-stack-words-1",
    "firaga-stack-words-2",
    "firaga-stack-copy-word",
    "firaga-stack-words-3",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiragaStackInitialization {
    pub roots: [u32; 5],
    pub word_pairs: [[u32; 2]; 3],
    pub copy_word: u32,
    pub final_words: [u32; 4],
}
impl FiragaStackInitialization {
    pub fn validate(&self) -> Result<(), String> {
        if self.roots != [0x0814d480, 0x0814d488, 0x0814d490, 0x0814d498, 0x0814d49c] {
            return Err("Invalid Firaga stack initialization roots".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        let words = |xs: &[u32]| xs.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let payloads = [
            words(&self.roots),
            words(&self.word_pairs[0]),
            words(&self.word_pairs[1]),
            words(&self.word_pairs[2]),
            self.copy_word.to_le_bytes().to_vec(),
            words(&self.final_words),
        ];
        Ok(SOURCE_OFFSETS
            .into_iter()
            .zip(payloads)
            .zip(ROLES)
            .map(|((offset, bytes), role)| (offset, bytes, role))
            .collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom, &[])?.decoded {
            return Err("Firaga stack fields differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: FiragaStackInitialization,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Firaga stack fields require exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let ldrs = [
            (0xd8570, 0xd8660, 0x483b),
            (0xd857e, 0xd8664, 0x4839),
            (0xd8588, 0xd8668, 0x4837),
            (0xd8592, 0xd866c, 0x4936),
            (0xd85a4, 0xd8670, 0x4832),
        ];
        for (pc, root, opcode) in ldrs {
            if half(pc) != opcode || ((pc + 4) & !3) + usize::from(opcode & 255) * 4 != root {
                return Err("Firaga stack root reader mismatch".into());
            }
        }
        let checks = [
            (0xd8572, 0x6841),
            (0xd8574, 0x6800),
            (0xd8580, 0x6841),
            (0xd8582, 0x6800),
            (0xd858a, 0x6841),
            (0xd858c, 0x6800),
            (0xd8596, 0x2204),
            (0xd8598, 0xf06f),
            (0xd859a, 0xfc12),
            (0xd85a6, 0xc82c),
            (0xd85a8, 0xc12c),
            (0xd85aa, 0x6800),
            (0xd85ac, 0x6008),
        ];
        if checks.iter().any(|&(at, value)| half(at) != value)
            || ffta_rom::digest(&rom[0x147dc0..0x147e22])
                != "aa822d873bad381a62e3c3e07b06003a41beea59bc3edc791f7e316eb08b7ebb"
        {
            return Err("Firaga stack copy reader mismatch".into());
        }
        let decoded = FiragaStackInitialization {
            roots: std::array::from_fn(|i| word(0xd8660 + 4 * i)),
            word_pairs: std::array::from_fn(|i| {
                std::array::from_fn(|j| word(0x14d480 + 8 * i + 4 * j))
            }),
            copy_word: word(0x14d498),
            final_words: std::array::from_fn(|i| word(0x14d49c + 4 * i)),
        };
        let prior = PriorSources::new(previous)?;
        let mut sources = Vec::new();
        for (i, (offset, bytes, role)) in decoded.source_spans()?.into_iter().enumerate() {
            if prior.contains(offset, bytes.len())?
                || rom[offset as usize..offset as usize + bytes.len()] != bytes
            {
                return Err("Firaga stack source overlap or reconstruction mismatch".into());
            }
            let sha256 = ffta_rom::digest(&bytes);
            if sha256 != HASHES[i] {
                return Err("Firaga stack source hash mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256,
                role: role.into(),
            });
        }
        Ok(Self { decoded, sources })
    }
}
