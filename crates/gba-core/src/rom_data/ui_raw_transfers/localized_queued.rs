//! Source-backed localized queued transfers; guest index arithmetic stays original.
use super::{BankView, SourceSpan};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const POINTERS: u32 = 0x397fbc;
pub const LENGTHS: u32 = 0x398034;
pub const POINTER_LITERAL: u32 = 0x09286c;
pub const LENGTH_LITERAL: u32 = 0x092878;
pub const BANKS: [u32; 5] = [0x438d00, 0x449f00, 0x45baf0, 0x46cd8c, 0x47e434];
pub const ENTRIES_PER_LANGUAGE: usize = 6;
pub const ENTRY_BYTES: usize = 192;
pub const BANK_BYTES: usize = ENTRIES_PER_LANGUAGE * ENTRY_BYTES;
const SOURCE_HASHES: [&str; 9] = [
    "87dc941b479b5a314366a35ffcdf5949ee0665c5b95f1c6c530038a89a204369",
    "26d4141dadbe4d0f7a744379ccfa8bd014fc71fd4206f4cb6d80046eb8afb70a",
    "8e66d77edb8dbb7167e3b068f348906f2d43e0b2651c18385404aaf2ba668424",
    "17425a59a926e006e7b9a7faa850924d2d2d41a63ea41ec3a27daff605e96988",
    "4091bd88d24beec269f76ffb21f30c29e28f4c65a54cc897b4411184c56354b8",
    "32dd42ed250645410c0b71b9bd8eaca5f37cf78ecb4e697eccbb84c7441b4a29",
    "0ae3c4007430b0e81beb94a515667b9f1c00e9a3c2b017a11963e1d93197059a",
    "ef14a7cadc3af5bf93ff2eb9c0fb7b569f09f5ff76ce6fac07a771261b057807",
    "62537933cf5b3c0a313e006c1cd76d1f4f287bca4c290e1d78e4f4ea9098ddb5",
];
const READER_HASH: &str = "9f2cc10b0da07510bcd013e3608ec8983cffd7da51c84bbb11566034a594e751";

fn invalid() -> String { "Invalid localized queued-transfer catalog".into() }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub pointers: Vec<u32>,
    pub lengths: Vec<u32>,
    pub literals: [u32; 2],
    pub banks: Vec<BankView>,
}

impl Catalog {
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        let count = BANKS.len() * ENTRIES_PER_LANGUAGE;
        if self.pointers.len() != count || self.lengths.len() != count || self.banks.len() != BANKS.len()
            || self.literals != [0x0800_0000 + POINTERS, 0x0800_0000 + LENGTHS] {
            return Err(invalid());
        }
        let words = |values: &[u32]| values.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>();
        let mut spans = vec![
            (POINTERS, words(&self.pointers), "localized-queued-pointers".into()),
            (LENGTHS, words(&self.lengths), "localized-queued-lengths".into()),
            (POINTER_LITERAL, self.literals[0].to_le_bytes().to_vec(), "localized-queued-pointer-literal".into()),
            (LENGTH_LITERAL, self.literals[1].to_le_bytes().to_vec(), "localized-queued-length-literal".into()),
        ];
        for (language, (bank, root)) in self.banks.iter().zip(BANKS).enumerate() {
            if bank.bank_offset != root || bank.relative_offset != 0 || bank.words.len() != BANK_BYTES / 4 {
                return Err(invalid());
            }
            for selector in 0..ENTRIES_PER_LANGUAGE {
                let i = language * ENTRIES_PER_LANGUAGE + selector;
                if self.pointers[i] != 0x0800_0000 + root + (selector * ENTRY_BYTES) as u32
                    || self.lengths[i] != ENTRY_BYTES as u32 {
                    return Err(invalid());
                }
            }
            spans.push((root, bank.source_bytes(), format!("localized-queued-bank/{root:08x}")));
        }
        Ok(spans)
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        for (start, values) in [(POINTERS, self.pointers.as_slice()), (LENGTHS, self.lengths.as_slice())] {
            if let Some(i) = offset.checked_sub(start).filter(|&i| (i as usize) < values.len() * 4) {
                return Some(values[i as usize / 4].to_le_bytes()[i as usize % 4]);
            }
        }
        for (start, value) in [(POINTER_LITERAL, self.literals[0]), (LENGTH_LITERAL, self.literals[1])] {
            if let Some(i) = offset.checked_sub(start).filter(|&i| i < 4) {
                return Some(value.to_le_bytes()[i as usize]);
            }
        }
        self.banks.iter().find_map(|bank| bank.byte_at(offset))
    }
}

pub fn from_rom(rom: &[u8]) -> Result<Catalog, String> {
    if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
        return Err("Localized queued transfers require exact EUR ROM; USA pending".into());
    }
    if ffta_rom::digest(&rom[0x9279c..0x927e2]) != READER_HASH {
        return Err("Localized queued-transfer reader fingerprint mismatch".into());
    }
    let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
    let count = BANKS.len() * ENTRIES_PER_LANGUAGE;
    let pointers = (0..count).map(|i| word(POINTERS as usize + i * 4)).collect();
    let lengths = (0..count).map(|i| word(LENGTHS as usize + i * 4)).collect();
    let literals = [word(POINTER_LITERAL as usize), word(LENGTH_LITERAL as usize)];
    let banks = BANKS.into_iter().map(|bank_offset| BankView {
        bank_offset,
        relative_offset: 0,
        words: (0..BANK_BYTES / 4).map(|i| word(bank_offset as usize + i * 4)).collect(),
    }).collect();
    let data = Catalog { pointers, lengths, literals, banks };
    let spans = data.source_spans()?;
    if spans.iter().zip(SOURCE_HASHES).any(|((_, bytes, _), hash)| ffta_rom::digest(bytes) != hash) {
        return Err("Localized queued-transfer source fingerprint mismatch".into());
    }
    Ok(data)
}
