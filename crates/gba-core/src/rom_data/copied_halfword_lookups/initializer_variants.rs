//! EUR literal-rooted initializer copy register variants.
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub struct Spec {
    pub entry: u32, pub literal: u32, pub source: u32, pub words: usize,
    pub source_reg: u8, pub destination_reg: u8, pub masks: &'static [u8],
    pub added_offset: u32, pub added_words: usize, pub owns_literal: bool,
    pub source_sha256: &'static str, pub code_sha256: &'static str,
}

pub const SPECS: [Spec; 16] = [
    Spec { entry: 0x00cdb4, literal: 0x00ce0c, source: 0x14d1b0, words: 6, source_reg: 0, destination_reg: 1, masks: &[0x1c, 0x4c], added_offset: 0x14d1bc, added_words: 3, owns_literal: false, source_sha256: "e6759a2c456bfbb7385ca9f93f1a6ec6940b4a4b576d6ce537cc3280cd7ea654", code_sha256: "668e674ed16613e4dde1cc5d7a1d0299db7798b432fbb35696e766f36cdd2e70" },
    Spec { entry: 0x01e2d4, literal: 0x01e300, source: 0x14d2bc, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x2c], added_offset: 0x14d2bc, added_words: 0, owns_literal: true, source_sha256: "c3e0f38a7aa95a60c251498ee8b9d401ceb12dc65c57f1fa12ce2efd783f4f1d", code_sha256: "7ef733fb0baa83b08fb7ba7c4bcc8043f918c103d61ec8431a35a89b91d2f9ef" },
    Spec { entry: 0x0d8874, literal: 0x0d8890, source: 0x14d4ac, words: 3, source_reg: 0, destination_reg: 2, masks: &[0x70], added_offset: 0x14d4ac, added_words: 3, owns_literal: true, source_sha256: "62c883682ab2f429849ecef35ecfdfc9ce217f71624c235b61d97c91ef8be44d", code_sha256: "5874f1219f25b5dc789c91a43ae85789abb0528747cb6caec4c6fbf9ab1c108d" },
    Spec { entry: 0x0e504c, literal: 0x0e50b4, source: 0x14d60c, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x64], added_offset: 0x14d60c, added_words: 3, owns_literal: true, source_sha256: "9efa40a909716cda48bcd6f71e20e7e867e5ab81a21070a79da88a4726eb78bf", code_sha256: "2d0371f712e3b48e65a0589042e262357f3a4020e7255ee5ab283dd5405a0de9" },
    Spec { entry: 0x0e50e0, literal: 0x0e518c, source: 0x14d61c, words: 8, source_reg: 0, destination_reg: 1, masks: &[0x2c, 0x4c, 0x60], added_offset: 0x14d61c, added_words: 8, owns_literal: true, source_sha256: "8d8c6ffd1fb42d388c224f016a249c9bafcde44a569883e4b414813e53d550e1", code_sha256: "027176fe6bf8286c104cde97e6ade327f1e046744090487432ad35e7f8086268" },
    Spec { entry: 0x0e5696, literal: 0x0e56c0, source: 0x14d63c, words: 3, source_reg: 0, destination_reg: 1, masks: &[0xc4], added_offset: 0x14d63c, added_words: 3, owns_literal: true, source_sha256: "cdf4d5b609d463aa7c55ecc77accd78c21a2598d9a06538a41f6c639ed39fe18", code_sha256: "a450fc971de942840c8a0599144bb5e41afe9b72a7cdcb684cbbc176f559b155" },
    Spec { entry: 0x0e56ca, literal: 0x0e5708, source: 0x14d64c, words: 8, source_reg: 1, destination_reg: 0, masks: &[0x4c, 0x8c, 0xc0], added_offset: 0x14d64c, added_words: 8, owns_literal: true, source_sha256: "66ed48343053a0efa01a09a651495504f0eddcb30828a2502a7102da16d03e94", code_sha256: "c0ef49e058cb08f1dfbe1b4c62ed1556c4033fdc6b747fd458ab8a653d9461a0" },
    Spec { entry: 0x0e570e, literal: 0x0e5740, source: 0x14d66c, words: 3, source_reg: 0, destination_reg: 1, masks: &[0xc4], added_offset: 0x14d66c, added_words: 3, owns_literal: true, source_sha256: "2cde4980c38056c975c9d76f99f99c025f4c1c2ec8b2326fb786ff0ce9d51155", code_sha256: "a7561f6264ccea08eff6d07610a451bcb5cb3077d37d60463c047a40ce5f2217" },
    Spec { entry: 0x0e7a8e, literal: 0x0e7af8, source: 0x14d680, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x8c], added_offset: 0x14d680, added_words: 0, owns_literal: true, source_sha256: "fb044eaaca05e18ac005dcab603ecfc734205d9e15fab6e9756135c88521fcb3", code_sha256: "1a056eaed868c1a884521afa7e1fed303ebaaf853bfa67f7e2e974ae4765c0df" },
    Spec { entry: 0x0e7a9a, literal: 0x0e7afc, source: 0x14d68c, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x8c], added_offset: 0x14d68c, added_words: 0, owns_literal: true, source_sha256: "940cf375e43ac954e93fa842fb7b3bb6b538b9e8b8833364b941789be3f4b022", code_sha256: "35ea801305ac775129fb2be8363934e1345a2bcc5651985a018d4e0acc854d99" },
    Spec { entry: 0x0fa9e2, literal: 0x0faa8c, source: 0x14d734, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x38], added_offset: 0x14d734, added_words: 3, owns_literal: true, source_sha256: "df46f9c26bbb21a4c039989219c82e5b970aaefd0eef4a30942e574817f49836", code_sha256: "b818152f8f20d33cd6e615c99229977b69db290613df0ac54be12317d2d7f135" },
    Spec { entry: 0x106150, literal: 0x1061dc, source: 0x14d7d8, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x4c], added_offset: 0x14d7d8, added_words: 3, owns_literal: true, source_sha256: "6f3cb74dafa4fcc77bf4dcc81b7c66280a8e0e7a2724347cedb933fab09f0127", code_sha256: "8dc4cbbc2d70f30c75d26a99a1204bfd6914bad3ff14533e639cf999a2b10a3a" },
    Spec { entry: 0x10ea16, literal: 0x10eaac, source: 0x14d8fc, words: 3, source_reg: 0, destination_reg: 1, masks: &[0x2c], added_offset: 0x14d8fc, added_words: 3, owns_literal: true, source_sha256: "888f003142e9accae002b5c6cbe494e955cfcf610a92846773c9f1c120bd5b72", code_sha256: "698284886a95818d1a09e959769aa19977e24686347f229ddd7d1fc1112daf65" },
    Spec { entry: 0x1220fc, literal: 0x122124, source: 0x14d9b0, words: 3, source_reg: 1, destination_reg: 2, masks: &[0x38], added_offset: 0x14d9b0, added_words: 3, owns_literal: true, source_sha256: "b186053c7533d40a7ac4bad4a9d5fa198eb7b567b4bd4c435bfdbcbac0a2c7df", code_sha256: "c1b3039caea9582f6a42730f6f8527a359a4eb760e0acd8676896a890da1a9b0" },
    Spec { entry: 0x1247aa, literal: 0x12482c, source: 0x14d9d0, words: 8, source_reg: 0, destination_reg: 1, masks: &[0xa4, 0xa4, 0x24], added_offset: 0x14d9d0, added_words: 8, owns_literal: true, source_sha256: "463d4cd0aa87f246876d2f69ff184648e2dc596b9ba2c65d0454009d6d3bd43c", code_sha256: "85dc65648bfa9afbf00d4b617d8b47edec0b33985adc66e5d2490f8758cecf7d" },
    Spec { entry: 0x13f416, literal: 0x13f474, source: 0x14d9f0, words: 2, source_reg: 0, destination_reg: 1, masks: &[0x18], added_offset: 0x14d9f0, added_words: 2, owns_literal: true, source_sha256: "66735bd791db1ed24b8f671964e2f49c36e7fbdc931b3c629216e6e2a0a6f463", code_sha256: "77fe732e948b070fcf2998ca84a6efb1b54ed5c797aa545683ed51c6d007cfed" },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant { pub source_offset: u32, pub words: Vec<u32> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantBank { pub records: Vec<Variant> }

impl VariantBank {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let mut records = Vec::with_capacity(SPECS.len());
        for spec in SPECS {
            let start = spec.entry as usize;
            let op = u16::from_le_bytes(rom[start..start+2].try_into().unwrap());
            let root = u32::from_le_bytes(rom[spec.literal as usize..spec.literal as usize+4].try_into().unwrap());
            let end = start + 2 + spec.masks.len() * 4;
            if op & 0xf800 != 0x4800 || ((op >> 8) & 7) != u16::from(spec.source_reg)
                || ((spec.entry + 4) & !3) + u32::from(op & 255) * 4 != spec.literal
                || root != 0x0800_0000 + spec.source
                || spec.masks.iter().map(|mask| mask.count_ones() as usize).sum::<usize>() != spec.words
                || ffta_rom::digest(&rom[start..end]) != spec.code_sha256
                || ffta_rom::digest(&rom[spec.source as usize..spec.source as usize+4*spec.words]) != spec.source_sha256 {
                return Err("Initializer variant original source or reader mismatch".into());
            }
            for (i, &mask) in spec.masks.iter().enumerate() {
                let at = start + 2 + i * 4;
                let load = u16::from_le_bytes(rom[at..at+2].try_into().unwrap());
                let store = u16::from_le_bytes(rom[at+2..at+4].try_into().unwrap());
                if load != (0xc800 | u16::from(spec.source_reg) << 8 | u16::from(mask))
                    || store != (0xc000 | u16::from(spec.destination_reg) << 8 | u16::from(mask)) {
                    return Err("Initializer variant copy instruction mismatch".into());
                }
            }
            let first = spec.added_offset as usize;
            let words = rom[first..first+spec.added_words*4].chunks_exact(4)
                .map(|word| u32::from_le_bytes(word.try_into().unwrap())).collect();
            records.push(Variant { source_offset: spec.added_offset, words });
        }
        Ok(Self { records })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.records.len() != SPECS.len() || self.records.iter().zip(SPECS).any(|(record, spec)|
            record.source_offset != spec.added_offset || record.words.len() != spec.added_words) {
            return Err("Invalid initializer variant records".into());
        }
        Ok(())
    }

    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>, &'static str)> {
        let mut spans = Vec::new();
        for (record, spec) in self.records.iter().zip(SPECS) {
            if !record.words.is_empty() {
                spans.push((spec.added_offset, record.words.iter().flat_map(|word| word.to_le_bytes()).collect(), "initializer-variant-words"));
            }
            if spec.owns_literal {
                spans.push((spec.literal, (0x0800_0000u32 + spec.source).to_le_bytes().to_vec(), "initializer-variant-root"));
            }
        }
        spans
    }

    pub fn byte_at(&self, offset: u32) -> Option<(u8, bool)> {
        for (record, spec) in self.records.iter().zip(SPECS) {
            if let Some(n) = offset.checked_sub(spec.added_offset) {
                if let Some(word) = record.words.get((n / 4) as usize) { return Some((word.to_le_bytes()[(n % 4) as usize], false)); }
            }
            if spec.owns_literal {
                if let Some(n) = offset.checked_sub(spec.literal) {
                    if n < 4 { return Some(((0x0800_0000u32 + spec.source).to_le_bytes()[n as usize], true)); }
                }
            }
        }
        None
    }
}
