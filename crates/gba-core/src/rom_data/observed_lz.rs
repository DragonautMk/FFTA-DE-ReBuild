//! EUR LZ77 blocks observed during the Red Spring action.
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
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedLzStream {
    pub source_offset: u32,
    pub encoded_source: Vec<u8>,
    pub decoded: Vec<u8>,
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
}

impl ObservedLzBlock {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != OFFSET || self.encoded_source.len() != ENCODED_LENGTH
            || self.decoded.len() != DECODED_LENGTH || self.additional_blocks.len() != ADDITIONAL_SPECS.len()
        { return Err("Invalid observed LZ block geometry".into()); }
        validate_stream(&self.encoded_source, &self.decoded)?;
        for (block, spec) in self.additional_blocks.iter().zip(ADDITIONAL_SPECS) {
            if block.source_offset != spec.offset || block.encoded_source.len() != spec.encoded_length
                || block.decoded.len() != spec.decoded_length
            { return Err("Invalid observed LZ stream geometry".into()); }
            validate_stream(&block.encoded_source, &block.decoded)?;
        }
        Ok(())
    }

    pub fn streams(&self) -> impl Iterator<Item = (u32, &[u8], &[u8])> {
        std::iter::once((self.source_offset, self.encoded_source.as_slice(), self.decoded.as_slice()))
            .chain(self.additional_blocks.iter().map(|s| (s.source_offset, s.encoded_source.as_slice(), s.decoded.as_slice())))
    }
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
        let prior = PriorSources::new(previous)?;
        if prior.contains(OFFSET, ENCODED_LENGTH)? {
            return Err("Observed LZ source already has an owner".into());
        }
        let mut additional_blocks = Vec::new();
        let mut sources = vec![SourceRange { offset: OFFSET, length: ENCODED_LENGTH as u32,
            sha256: SOURCE_SHA256.into(), role: "observed-red-spring-lz77".into() }];
        for spec in ADDITIONAL_SPECS {
            if prior.contains(spec.offset, spec.encoded_length)? {
                return Err("Observed LZ stream already has an owner".into());
            }
            let encoded = rom.get(spec.offset as usize..spec.offset as usize + spec.encoded_length)
                .ok_or("Missing observed LZ stream")?;
            if ffta_rom::digest(encoded) != spec.source_sha256 {
                return Err("Observed LZ stream source hash mismatch".into());
            }
            let block = lz77::decode(encoded, spec.decoded_length)?;
            if block.consumed != spec.encoded_length || ffta_rom::digest(&block.bytes) != spec.decoded_sha256 {
                return Err("Observed LZ stream decoded source mismatch".into());
            }
            additional_blocks.push(ObservedLzStream { source_offset: spec.offset,
                encoded_source: encoded.to_vec(), decoded: block.bytes });
            sources.push(SourceRange { offset: spec.offset, length: spec.encoded_length as u32,
                sha256: spec.source_sha256.into(), role: "observed-red-spring-lz77".into() });
        }
        let decoded = ObservedLzBlock { source_offset: OFFSET, encoded_source: source.to_vec(), decoded: block.bytes, additional_blocks };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}
