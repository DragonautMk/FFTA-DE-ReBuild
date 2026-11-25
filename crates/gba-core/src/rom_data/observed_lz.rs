//! Authenticated EUR LZ77 blocks for selected graphics sources.
use super::{lz77, script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const OFFSET: u32 = 0xb0e2b8;
pub const ENCODED_LENGTH: usize = 2578;
pub const DECODED_LENGTH: usize = 4096;
const SOURCE_SHA256: &str = "5b48c70199d66ce4b1b4f2997b2f5d7f8bf462b004430f0f1b3af2df870763a2";
const DECODED_SHA256: &str = "2a67692e4bc92a9969c2f8bf8deabcfac2a3902e503d6550b721385dce95fe90";

#[derive(Clone, Copy)]
pub struct StreamSpec {
    pub offset: u32,
    pub encoded_length: usize,
    pub decoded_length: usize,
    source_sha256: &'static str,
    decoded_sha256: &'static str,
}

/// Source-authenticated projections, not bounds on guest addresses or calls.
pub const ADDITIONAL_SPECS: &[StreamSpec] = &[
    StreamSpec { offset: 0xb02ea4, encoded_length: 535, decoded_length: 800,
        source_sha256: "36d797fc5a0d71096562a13e269dc54ce670d99a9e016d7af2926ec0b45c5bc3",
        decoded_sha256: "671776ec93b9b0231e815a3b3250fcc86e2ab7e82fb449f71439704e50c9ceed" },
    StreamSpec { offset: 0xb030c0, encoded_length: 73, decoded_length: 96,
        source_sha256: "2a334af181037d632a98fa9f73e9194e6b4200699f1bf1ac51bee4a677bcc00a",
        decoded_sha256: "082b9621c44dc151526aa5615f746f7eebdb15fae32352c54437f36d45bb3b6b" },
    StreamSpec { offset: 0xb0dd5c, encoded_length: 55, decoded_length: 160,
        source_sha256: "8c923ff312d49644f32c94adb5c6a1157e614276d336657360b03ab27ae4bf6a",
        decoded_sha256: "502970103b469162c0a681f95d4f1d8d74bc9efeccc345a3bc64a4e8957d25c8" },
    StreamSpec { offset: 0xb0df2c, encoded_length: 34, decoded_length: 48,
        source_sha256: "65ff8554fba66642a2b003158eadb9174d64cf28f40590752adb985d29177e01",
        decoded_sha256: "0eebbd207bc69f8b3f36d09d20fa8cb1b7bc7aadbcd2cef237ddb81a65385d80" },
    StreamSpec { offset: 0xb0ecd0, encoded_length: 774, decoded_length: 928,
        source_sha256: "ad47c32e828db755381ae38ac1e154e7d3a6063db2acefd80548268047282169",
        decoded_sha256: "2b97158d9a7e01f9e1fddf8c84cea6b51334b801ca2adf9f4011257f57de06d5" },
    StreamSpec { offset: 0xb151d8, encoded_length: 820, decoded_length: 1504,
        source_sha256: "5a7b9cc3b760679af76d54bb6bbd295362353df7eea5f4e1af6567c38ca0af56",
        decoded_sha256: "11b33398415f12b91f47eca9ee9f3e56ec68cd62d37a194a3478efae6ddf65b2" },
    StreamSpec { offset: 0xb0efdc, encoded_length: 473, decoded_length: 640,
        source_sha256: "b2ae228cc6239f508ae1b5bd70f056587092b1a98c8ea1da6978cd379fb0900c",
        decoded_sha256: "b0fbae427f5155e1d1269b28251c67e0d1da74c0371dbbd6c971d3c50a4c98f9" },
    StreamSpec { offset: 0xb0f1bc, encoded_length: 791, decoded_length: 992,
        source_sha256: "24dcb352182627064d91c90a8ad24789c6e769fee6ebdb7d47fab27a384d2fa9",
        decoded_sha256: "0a960e61c4ec90241f67e5ca446be8420db413f960b6dce3b741a6732b8a6ec6" },
    StreamSpec { offset: 0xb0f4d8, encoded_length: 643, decoded_length: 992,
        source_sha256: "1ed39f20f92cc0f194e460883c6cff6a6194b4cab6c0857181f169bb0b58ae6a",
        decoded_sha256: "28a9e15f327a86b907c1fb6f551a44e792da8c08e1293773bf063f61d7f43e61" },
    StreamSpec { offset: 0xb0f760, encoded_length: 688, decoded_length: 896,
        source_sha256: "69fe17486f053796ed913f192ee65967387344eb7b83509d48fbd9955b854746",
        decoded_sha256: "8c88468f80eccd99cff5432269624f27980f5db1c3afc318a66ef47b8b746669" },
    StreamSpec { offset: 0xb0fa14, encoded_length: 449, decoded_length: 704,
        source_sha256: "43c9104e6495f1fff0277ef0aac7d1861df94140668071bef1c80a27991676c5",
        decoded_sha256: "a44d8684014b90a9a41dcfdc98c4fc147afe3ff13c9b1ccc01d1a5fed3754bc7" },
    StreamSpec { offset: 0xb0fbdc, encoded_length: 526, decoded_length: 736,
        source_sha256: "8e7419e855980bf0641ffe2790afcdb60043ee90f6d6240f01086f8a33b052c7",
        decoded_sha256: "49cb1e2eebb76b6b7d7aaa20ee66102043e34c6e1a9db264f225b1274f7a1e4f" },
    StreamSpec { offset: 0xb0fdf0, encoded_length: 553, decoded_length: 768,
        source_sha256: "dfd73dd24d80f051e6ccb6e8c24a6cf080b65cfa5bd4af1c794dddae1b3205e8",
        decoded_sha256: "9574c9c8e5db19d8b5a614241c45cf76dc3f5719177845575f48e7a1b2b033d3" },
    StreamSpec { offset: 0xaff6e0, encoded_length: 1033, decoded_length: 1224,
        source_sha256: "9e12133e01e35394e54029a0e98f6c0708c679c9124763df2c043a8699b028c2",
        decoded_sha256: "117aacce4f44dbf46040c17d6387cc1aa135ee7c57dbd02c6a67f9549be2b1b8" },
    StreamSpec { offset: 0xb0b4e4, encoded_length: 231, decoded_length: 320,
        source_sha256: "58b66de0c2864fcd3f5d6d8bd9be38ff7fa722900d81ef8a9ac55ebacc2d10fe",
        decoded_sha256: "e64b8f75c635e66dbeff9a9bc29455bc1751e35c4edc1df7b0f210ec4e5bec39" },
    StreamSpec { offset: 0xb0dd10, encoded_length: 71, decoded_length: 96,
        source_sha256: "ea50a89b8ad6c573045b85625fde713dd17ef3e2b462f922609b6d12f8774301",
        decoded_sha256: "5ffa4d8b812c1ca23e7c2be5fe5f5cc84037b4daf326c3168181d3a9c46539b3" },
    StreamSpec { offset: 0xaf4998, encoded_length: 2744, decoded_length: 5136,
        source_sha256: "2cadde0263ee4f58e84479fe1383d607a27e44af2f9b9837538771388f7f8ad6",
        decoded_sha256: "6f9dbe1cb1f8373f94281c74900de06521cd28a3941c0ca6567c82797a2c3336" },
    StreamSpec { offset: 0xaf5454, encoded_length: 517, decoded_length: 784,
        source_sha256: "e8cf9c78d56245c0a4f1178e74c60afeef3663649fb73a6ce0906aa646319e2e",
        decoded_sha256: "b523340615a6eb0caba265854a1c469e6c856a23c634fe50ecd4cf3df389b733" },
    StreamSpec { offset: 0xaf5660, encoded_length: 29, decoded_length: 48,
        source_sha256: "d0c14fd944f302b7970694a8b1d2df2dd15a2e7dce7e0f5a491bb749818a12e0",
        decoded_sha256: "e29a2b0c5e5484a012841a484b76d1da4d69c1bbd848a9ec740c418de8afe821" },
    StreamSpec { offset: 0xaffc70, encoded_length: 4390, decoded_length: 8192,
        source_sha256: "2ef16acfc5fd820fa6bd70f45f80bcc42b521ce92e400777d786a613dde6a6f0",
        decoded_sha256: "a364981b58994d1b191d2ff045014e2787c78aa713083569da76b018bdd46aae" },
    StreamSpec { offset: 0xb00d9c, encoded_length: 570, decoded_length: 1104,
        source_sha256: "69f58f5e9d44a320d138d10408c9f07424b363f5b758fb30a0ea578708ae3fc4",
        decoded_sha256: "438fdf18899a99207371c3a129d4e5d538a5745a08701c5392a01760bfad99b3" },
];

/// All three LZ77 sources used by the original EUR VRAM initialization routine.
pub const VRAM_SPECS: [StreamSpec; 3] = [
    StreamSpec {
        offset: 0xb4fa4c,
        encoded_length: 11544,
        decoded_length: 20480,
        source_sha256: "bd713eae52ee7c606a1beeced0adbede64c342804fdc0fb040789c5ba19580d5",
        decoded_sha256: "43a6c65f0343a5add980320718d9c0e6641bb8468c470c322333868f553660b9",
    },
    StreamSpec {
        offset: 0xb52764,
        encoded_length: 2218,
        decoded_length: 4096,
        source_sha256: "d1c1d6d410ab14048b9f17364db5f8e745168b844841045fd40dc09e8430a6dd",
        decoded_sha256: "8433e16e9432cfcd12cf4a8cfec53d120d5788765d1a79fbdc7f922a42a1059c",
    },
    StreamSpec {
        offset: 0xb53010,
        encoded_length: 1702,
        decoded_length: 4096,
        source_sha256: "c91c1724c87e659a642cdd4921a634ce44c52f29ded5067c47c02b647d8b543e",
        decoded_sha256: "e33d591244607ec012270c17bb84ea3a8607de6f880ab77cafb992aef1101fa4",
    },
];
pub const VRAM_LITERALS: [u32; 3] = [0x039dcc, 0x039dd8, 0x039ddc];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedLzStream {
    pub source_offset: u32,
    pub encoded_source: Vec<u8>,
    pub decoded: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VramLzBank {
    /// Little-endian source addresses in original call order.
    pub source_literals: [[u8; 4]; 3],
    pub blocks: [ObservedLzStream; 3],
}

/// Decoded output is consumed by the LZ77 HLE; the exact source encoding
/// also preserves guest bus, DMA and real-BIOS access to the same block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedLzBlock {
    pub source_offset: u32,
    pub encoded_source: Vec<u8>,
    pub decoded: Vec<u8>,
    /// Kept separate to preserve the original E174 block's representation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_blocks: Vec<ObservedLzStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vram_bank: Option<VramLzBank>,
}

impl ObservedLzBlock {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != OFFSET || self.encoded_source.len() != ENCODED_LENGTH
            || self.decoded.len() != DECODED_LENGTH || self.additional_blocks.len() != ADDITIONAL_SPECS.len()
        { return Err("Invalid observed LZ block geometry".into()); }
        validate_stream(&self.encoded_source, &self.decoded)?;
        for (block, spec) in self.additional_blocks.iter().zip(ADDITIONAL_SPECS) {
            validate_block(block, spec)?;
        }
        if let Some(bank) = &self.vram_bank {
            for ((block, literal), spec) in bank.blocks.iter().zip(bank.source_literals).zip(&VRAM_SPECS) {
                if u32::from_le_bytes(literal) != 0x0800_0000 + spec.offset {
                    return Err("Invalid VRAM LZ77 source literal".into());
                }
                validate_block(block, spec)?;
            }
        }
        Ok(())
    }

    pub fn streams(&self) -> impl Iterator<Item = (u32, &[u8], &[u8])> {
        std::iter::once((self.source_offset, self.encoded_source.as_slice(), self.decoded.as_slice()))
            .chain(
                self.additional_blocks
                    .iter()
                    .chain(self.vram_bank.iter().flat_map(|bank| &bank.blocks))
                    .map(|s| (s.source_offset, s.encoded_source.as_slice(), s.decoded.as_slice())),
            )
    }

    pub(crate) fn source_spans(&self) -> impl Iterator<Item = (u32, &[u8], &'static str)> {
        self.streams()
            .enumerate()
            .map(|(index, (offset, bytes, _))| {
                let role = if index <= ADDITIONAL_SPECS.len() {
                    "observed-red-spring-lz77"
                } else {
                    "bios-lz77-vram-bank-stream"
                };
                (offset, bytes, role)
            })
            .chain(self.vram_bank.iter().flat_map(|bank| {
                VRAM_LITERALS
                    .into_iter()
                    .zip(&bank.source_literals)
                    .map(|(offset, word)| (offset, word.as_slice(), "bios-lz77-vram-bank-literal"))
            }))
    }
}

fn validate_block(block: &ObservedLzStream, spec: &StreamSpec) -> Result<(), String> {
    if block.source_offset != spec.offset
        || block.encoded_source.len() != spec.encoded_length
        || block.decoded.len() != spec.decoded_length
    {
        return Err("Invalid observed LZ stream geometry".into());
    }
    validate_stream(&block.encoded_source, &block.decoded)
}

fn validate_stream(encoded: &[u8], decoded: &[u8]) -> Result<(), String> {
    let block = lz77::decode(encoded, decoded.len())?;
    if block.consumed != encoded.len() || block.bytes != decoded {
        return Err("Observed LZ source and decoded output differ".into());
    }
    Ok(())
}

pub struct Catalog {
    pub decoded: ObservedLzBlock,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        Self::from_rom_with_vram_bank(rom, previous, true)
    }

    pub(crate) fn from_rom_with_vram_bank(
        rom: &[u8],
        previous: &[Resource],
        include_vram_bank: bool,
    ) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Observed LZ block requires the exact EUR ROM".into());
        }
        let source = rom.get(OFFSET as usize..OFFSET as usize + ENCODED_LENGTH).ok_or("Missing observed LZ source")?;
        if ffta_rom::digest(source) != SOURCE_SHA256 {
            return Err("Observed LZ source hash mismatch".into());
        }
        let block = lz77::decode(source, DECODED_LENGTH)?;
        if block.consumed != ENCODED_LENGTH || ffta_rom::digest(&block.bytes) != DECODED_SHA256 {
            return Err("Observed LZ decoded source mismatch".into());
        }
        let additional_blocks = ADDITIONAL_SPECS
            .iter()
            .map(|spec| extract_stream(rom, spec))
            .collect::<Result<_, _>>()?;
        let vram_bank = include_vram_bank.then(|| extract_vram_bank(rom)).transpose()?;
        let decoded = ObservedLzBlock {
            source_offset: OFFSET,
            encoded_source: source.to_vec(),
            decoded: block.bytes,
            additional_blocks,
            vram_bank,
        };
        decoded.validate()?;
        let prior = PriorSources::new(previous)?;
        let sources = decoded.source_spans().map(|(offset, bytes, role)| {
            if prior.contains(offset, bytes.len())? {
                return Err("Observed LZ source already has an owner".into());
            }
            Ok(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(bytes),
                role: role.into(),
            })
        }).collect::<Result<_, String>>()?;
        Ok(Self { decoded, sources })
    }
}

fn extract_stream(rom: &[u8], spec: &StreamSpec) -> Result<ObservedLzStream, String> {
    let start = spec.offset as usize;
    let encoded = rom.get(start..start + spec.encoded_length).ok_or("Missing observed LZ stream")?;
    if ffta_rom::digest(encoded) != spec.source_sha256 {
        return Err("Observed LZ stream source hash mismatch".into());
    }
    let block = lz77::decode(encoded, spec.decoded_length)?;
    if block.consumed != encoded.len() || ffta_rom::digest(&block.bytes) != spec.decoded_sha256 {
        return Err("Observed LZ stream decoded source mismatch".into());
    }
    Ok(ObservedLzStream {
        source_offset: spec.offset,
        encoded_source: encoded.to_vec(),
        decoded: block.bytes,
    })
}

fn extract_vram_bank(rom: &[u8]) -> Result<VramLzBank, String> {
    if ffta_rom::digest(&rom[0x039d4e..0x039d9a]) != "9336a2ee0f8a89167f414ae848022a15d4749f9acbf0df4c2aee757b03b0767a"
        || ffta_rom::digest(&rom[0x145250..0x145254]) != "4f32a5a0727704ed8cc1e46af9875d484bb379b73cdd4ae15086deeb7426b8a0"
    {
        return Err("VRAM LZ77 caller fingerprint mismatch".into());
    }
    let source_literals = VRAM_LITERALS.map(|offset| {
        let start = offset as usize;
        rom[start..start + 4].try_into().unwrap()
    });
    let [first, second, third] = VRAM_SPECS;
    let blocks = [
        extract_stream(rom, &first)?,
        extract_stream(rom, &second)?,
        extract_stream(rom, &third)?,
    ];
    Ok(VramLzBank { source_literals, blocks })
}
