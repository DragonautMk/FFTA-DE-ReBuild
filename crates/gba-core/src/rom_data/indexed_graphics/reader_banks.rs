//! European indexed archives rooted in the original reader tables or literals.
use super::{Archive, SourceSpan, additional::ArchiveSpec};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

const ROM_BASE: u32 = 0x0800_0000;
pub const PROGRAM_COUNT: usize = 852;

pub const TABLES: &[(u32, &[u32])] = &[
    (
        0x397d6c,
        &[0x43c294, 0x44d88c, 0x45f3b4, 0x47041c, 0x481f40],
    ),
    (
        0x3980ac,
        &[0x437300, 0x4483e4, 0x459e40, 0x46b2a4, 0x47c948],
    ),
    (
        0x398e54,
        &[
            0x42dd48, 0x42dfa0, 0x42e1f0, 0x42e470, 0x42e6f8, 0x42e950, 0x42eb7c, 0x42edbc,
            0x42efc0, 0x42f1e0, 0x42f42c, 0x42f728, 0x42f99c, 0x42fbcc, 0x42fde8, 0x43002c,
        ],
    ),
];

pub const LITERALS: &[(u32, u32)] = &[
    (0x032920, 0x083aed7c),
    (0x0690b8, 0x08397d6c),
    (0x070ed0, 0x08397d6c),
    (0x073870, 0x083b8c50),
    (0x073880, 0x083b5acc),
    (0x07e680, 0x08397d6c),
    (0x07e8a4, 0x08397d6c),
    (0x08a9c8, 0x083b5acc),
    (0x08aa48, 0x083b5acc),
    (0x08ab80, 0x083b5acc),
    (0x08f3a4, 0x08397d6c),
    (0x092864, 0x083980ac),
    (0x0a47e4, 0x08398e54),
    (0x0abbd8, 0x08398e54),
    (0x0cf3c8, 0x083bdb24),
    (0x12f068, 0x083bafb4),
];

const READER_WINDOWS: &[(usize, usize, &str)] = &[
    (
        0x005224,
        0x20a,
        "00a1d3204d66e42d9f3dbfb33b1627ec6ba036dda3f9cae093b8cc626b036a7b",
    ),
    (
        0x032810,
        0xa8,
        "7e707c3463a2e1e4377a82f9e7c50405382b60c9d7c1341fe13f19ba8cb5d236",
    ),
    (
        0x068fc0,
        0x54,
        "43fb4908be62d8c89f8bf2cf000f06cf27d0f30308bdad78f731f5a2e4befb6f",
    ),
    (
        0x070e2a,
        0x66,
        "c4beff2d93010c3fc3419795451066d89f4460cd8e352217b8dd7c75a8591162",
    ),
    (
        0x0737ba,
        0x66,
        "2493552cae05f82e4290f0b59c1fa1e7b43befb674bc2231efa2e6cf6b1b6973",
    ),
    (
        0x07e61e,
        0x52,
        "dccfc747dbeeba60cab06bb20ac3cc48621deebb0f3caa49082db02181600db5",
    ),
    (
        0x07e800,
        0x5e,
        "20b0891ed65d127820707e152d8d3f68caca41dd96431c41357f2b8dac5dff7a",
    ),
    (
        0x08a950,
        0x50,
        "a8d4aa4992a9f7433ab41b68fe782ec2c79adb4ff508b72e0610d023db959786",
    ),
    (
        0x08a9f0,
        0x46,
        "c1335ca4dd3b531ed2273964ffe6fd975c1bf720532774157ac1b81cd2d839b7",
    ),
    (
        0x08ab1a,
        0x5a,
        "da0dbe4974e8acbe4939e19d077b4afe2c6e3c6a64d47f0ae825c65b440a1da6",
    ),
    (
        0x08f346,
        0x58,
        "331fe1efeba9251b6a09527cda3b0b5030e65e556720aa99c32d6a4061290f85",
    ),
    (
        0x09272a,
        0x68,
        "f2ccc3827e73d7c269067b6d684531516e75da9dab7a4fa1e38034b20ae48347",
    ),
    (
        0x0a4770,
        0x6c,
        "de145c190f36b40e828fdfc08ee62f9e88bfccdcecf86b516d1cdebc9ce660a6",
    ),
    (
        0x0abb9c,
        0x38,
        "fbb1d95f1922a446c92d7108f7f71a662739843dc18084741e54bf1467bc80a3",
    ),
    (
        0x0cf326,
        0x13e,
        "2abe18e08c6fdfb426cab86779b3f1bb2c5d5dbc1d3e82d651481cb786d395c2",
    ),
    (
        0x12f020,
        0x42,
        "fe7c847cfcd278b0c41f4f3323d25f6ad5636eda142b4aed9ac8f83e7d2fa3e1",
    ),
];

pub const ARCHIVES: [ArchiveSpec; 31] = [
    ArchiveSpec {
        offset: 0x3aed7c,
        length: 4774,
        programs: 90,
        decoded_bytes: 64,
        sha256: "4b851fe464c8fdb42c16814d8ef85e22cdba48ca90451e4a354a011d9b79db98",
    },
    ArchiveSpec {
        offset: 0x3b5acc,
        length: 12619,
        programs: 54,
        decoded_bytes: 640,
        sha256: "7f0b63bce41068300aa33e4cc57477bc481932fa530230bd3847348fc2da265d",
    },
    ArchiveSpec {
        offset: 0x3b8c50,
        length: 873,
        programs: 5,
        decoded_bytes: 640,
        sha256: "9743b8f79201a6cb5a2ef77ec0415f07f6b4edd55704b5b5857f326b5438a5d6",
    },
    ArchiveSpec {
        offset: 0x3bafb4,
        length: 992,
        programs: 3,
        decoded_bytes: 768,
        sha256: "27edf50fe416b8bf031f501f7cf698afd55aab319f62d826322f9d42871f8ac1",
    },
    ArchiveSpec {
        offset: 0x3bdb24,
        length: 47383,
        programs: 522,
        decoded_bytes: 128,
        sha256: "f81346493bdf7eba1a2c357118e94cb2dbfccb10766686a86e902f6bba185d7b",
    },
    ArchiveSpec {
        offset: 0x42dd48,
        length: 600,
        programs: 8,
        decoded_bytes: 128,
        sha256: "073eec17fabdf414759b6e033c290e5b128be2a1699c28b60ddc310007c50459",
    },
    ArchiveSpec {
        offset: 0x42dfa0,
        length: 590,
        programs: 8,
        decoded_bytes: 128,
        sha256: "67e8045a62ca4f3c28e5f9d2b2f3460688234b4cb6d6c1ff740b3bc563530f5b",
    },
    ArchiveSpec {
        offset: 0x42e1f0,
        length: 639,
        programs: 8,
        decoded_bytes: 128,
        sha256: "4cba9f7ba8328a362c60735c8ac8455eb086fb20da4a3a2f95a7d9c9c29ef580",
    },
    ArchiveSpec {
        offset: 0x42e470,
        length: 648,
        programs: 8,
        decoded_bytes: 128,
        sha256: "771da7064c3a61f3036338a3c95b434beb84ffd31830e1a7d8b468574697f372",
    },
    ArchiveSpec {
        offset: 0x42e6f8,
        length: 600,
        programs: 8,
        decoded_bytes: 128,
        sha256: "a275ca63e081f05ca59ca7a82ef92e0f0ff6f6e65e374e08f22e8c96be5860ea",
    },
    ArchiveSpec {
        offset: 0x42e950,
        length: 554,
        programs: 8,
        decoded_bytes: 128,
        sha256: "2f12ad84db5fb4b8c22a1e929ec9cb9a7c338f731241083d07f73c5e3de5c65b",
    },
    ArchiveSpec {
        offset: 0x42eb7c,
        length: 575,
        programs: 8,
        decoded_bytes: 128,
        sha256: "18116c9db5dbf2e403aec9997b454bc165157a5b859314f6a3daf0530155f130",
    },
    ArchiveSpec {
        offset: 0x42edbc,
        length: 515,
        programs: 8,
        decoded_bytes: 128,
        sha256: "1a05483c1c24da94a9f1849c6f7e1a6b11352da6c8123b615c9a37b887327439",
    },
    ArchiveSpec {
        offset: 0x42efc0,
        length: 544,
        programs: 8,
        decoded_bytes: 128,
        sha256: "080a9c859e47921ad9e729f62fd4e3997090a4e66ef071b81930faeff7968aa5",
    },
    ArchiveSpec {
        offset: 0x42f1e0,
        length: 588,
        programs: 8,
        decoded_bytes: 128,
        sha256: "0bc50eeee09f81727f7daa77debf439b3a422907f79df956ca06853e4ef2a32f",
    },
    ArchiveSpec {
        offset: 0x42f42c,
        length: 763,
        programs: 8,
        decoded_bytes: 128,
        sha256: "da29ef7ec6bbf6c8e828450fee1ba0a95870e9ecbff36578cff869a6fb13941c",
    },
    ArchiveSpec {
        offset: 0x42f728,
        length: 627,
        programs: 8,
        decoded_bytes: 128,
        sha256: "2d704ba5255c752ad3fb4c23bbeb6beda3febbdcc12cd3e7aefae0b161d6a883",
    },
    ArchiveSpec {
        offset: 0x42f99c,
        length: 560,
        programs: 8,
        decoded_bytes: 128,
        sha256: "c2562af4a5fcbd664c4db9858285657ba91d2655f3a0aaabad5591f4159e3b64",
    },
    ArchiveSpec {
        offset: 0x42fbcc,
        length: 540,
        programs: 8,
        decoded_bytes: 128,
        sha256: "371af04cc1762914151c1fec9e836323d30481807036eb7c9cc78fa1b911e590",
    },
    ArchiveSpec {
        offset: 0x42fde8,
        length: 578,
        programs: 8,
        decoded_bytes: 128,
        sha256: "b958d27d1086ac6cb4a1fe1ce9876f262bdfc75b51b18a3f68de8b4068decc1a",
    },
    ArchiveSpec {
        offset: 0x43002c,
        length: 539,
        programs: 8,
        decoded_bytes: 128,
        sha256: "854abc4c37d949468294463e5058d0240d5219b7e41ec96303210c515b8021fe",
    },
    ArchiveSpec {
        offset: 0x437300,
        length: 5024,
        programs: 6,
        decoded_bytes: 1792,
        sha256: "de0859b93c1fff589b332e109352d08efc8d79980865de0a4b3fc246d5bf934f",
    },
    ArchiveSpec {
        offset: 0x43c294,
        length: 1320,
        programs: 4,
        decoded_bytes: 320,
        sha256: "4d07c8f3840a3eb0f98b3bef2e7071bb24ace8725d529ef500c565b7d3148d79",
    },
    ArchiveSpec {
        offset: 0x4483e4,
        length: 5306,
        programs: 6,
        decoded_bytes: 1792,
        sha256: "a40b674f5228ac233662b6cbb152a84a3bd521a1cab951a8b3bc057c91b2ffb7",
    },
    ArchiveSpec {
        offset: 0x44d88c,
        length: 1320,
        programs: 4,
        decoded_bytes: 320,
        sha256: "e705dceb31b31258318c497f2d6438625d1f90b847b471069dfe542dc963989d",
    },
    ArchiveSpec {
        offset: 0x459e40,
        length: 5709,
        programs: 6,
        decoded_bytes: 1792,
        sha256: "0303636f76d96ed6662d6d13dc88fd3ac6d8012430bf7eeefb2ab9bd79f784ca",
    },
    ArchiveSpec {
        offset: 0x45f3b4,
        length: 1320,
        programs: 4,
        decoded_bytes: 320,
        sha256: "17c70460b9e33df388a2ea6bddd347c5ed36feae1904db655bbaeff8f457627b",
    },
    ArchiveSpec {
        offset: 0x46b2a4,
        length: 5255,
        programs: 6,
        decoded_bytes: 1792,
        sha256: "6ae007f2bce5ce4c441f68f4056ad780a2238105974bc2bb98b33f793125b3bc",
    },
    ArchiveSpec {
        offset: 0x47041c,
        length: 1320,
        programs: 4,
        decoded_bytes: 320,
        sha256: "d941eb1f2a43b2f8ca785599a1fe2aab8939428fdd30a6037f342c3d843ae540",
    },
    ArchiveSpec {
        offset: 0x47c948,
        length: 5258,
        programs: 6,
        decoded_bytes: 1792,
        sha256: "b36cafd734b96fb47287a720fb8799c46906335a82131a0e972d659e8cdbc9c5",
    },
    ArchiveSpec {
        offset: 0x481f40,
        length: 1320,
        programs: 4,
        decoded_bytes: 320,
        sha256: "8930c1743c0b0f410f752ffb1128d3df3194bec64324683ef409cddfe89e6a14",
    },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointerTable {
    pub source_offset: u32,
    /// ROM offsets. The guest reads absolute addresses from the projected table.
    pub pointers: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReaderBanks {
    pub tables: Vec<PointerTable>,
    pub caller_literals: Vec<u32>,
    pub archives: Vec<Archive>,
}

fn invalid() -> String {
    "Invalid European indexed reader banks".into()
}

impl ReaderBanks {
    pub(super) fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        if self.tables.len() != TABLES.len()
            || self.caller_literals.len() != LITERALS.len()
            || self.archives.len() != ARCHIVES.len()
        {
            return Err(invalid());
        }
        let mut spans = Vec::new();
        for (table, &(offset, pointers)) in self.tables.iter().zip(TABLES) {
            if table.source_offset != offset || table.pointers != pointers {
                return Err(invalid());
            }
            let bytes = pointers
                .iter()
                .flat_map(|value| (ROM_BASE + value).to_le_bytes())
                .collect();
            spans.push((offset, bytes, format!("reader-indexed-table/{offset:06x}")));
        }
        for (&value, &(offset, expected)) in self.caller_literals.iter().zip(LITERALS) {
            if value != expected {
                return Err(invalid());
            }
            spans.push((
                offset,
                value.to_le_bytes().to_vec(),
                format!("reader-indexed-literal/{offset:06x}"),
            ));
        }
        for (archive, spec) in self.archives.iter().zip(ARCHIVES) {
            if archive.source_offset != spec.offset
                || archive.programs.len() != spec.programs
                || archive.fixed_decoded_bytes != spec.decoded_bytes
            {
                return Err(invalid());
            }
            let bytes = archive.source_bytes()?;
            if bytes.len() != spec.length {
                return Err(invalid());
            }
            spans.push((
                spec.offset,
                bytes,
                format!("reader-indexed-archive/{:06x}", spec.offset),
            ));
        }
        if self
            .archives
            .iter()
            .map(|entry| entry.programs.len())
            .sum::<usize>()
            != PROGRAM_COUNT
        {
            return Err(invalid());
        }
        Ok(spans)
    }
}

/// The catalog has already authenticated the full EUR ROM and its length.
pub(super) fn from_rom(rom: &[u8]) -> Result<ReaderBanks, String> {
    for &(offset, length, hash) in READER_WINDOWS {
        if ffta_rom::digest(&rom[offset..offset + length]) != hash {
            return Err("Indexed bank reader fingerprint mismatch".into());
        }
    }
    let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
    let tables = TABLES
        .iter()
        .map(|&(source_offset, pointers)| {
            let pointers = (0..pointers.len())
                .map(|i| {
                    word(source_offset as usize + i * 4)
                        .checked_sub(ROM_BASE)
                        .ok_or_else(invalid)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(PointerTable {
                source_offset,
                pointers,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let caller_literals = LITERALS
        .iter()
        .map(|&(offset, _)| word(offset as usize))
        .collect();
    let archives = ARCHIVES
        .iter()
        .map(|spec| {
            let start = spec.offset as usize;
            let source = &rom[start..start + spec.length];
            if ffta_rom::digest(source) != spec.sha256 {
                return Err(format!(
                    "Indexed bank archive {:06x} fingerprint mismatch",
                    spec.offset
                ));
            }
            let archive = Archive::from_source(source, spec.offset)?;
            if archive.source_bytes()? != source {
                return Err(invalid());
            }
            Ok(archive)
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(ReaderBanks {
        tables,
        caller_literals,
        archives,
    })
}

pub(crate) fn source_page(index: usize) -> bool {
    let contains = |start: u32, length: usize| {
        let page = index >> 12;
        let start = start as usize;
        (start >> 12..=(start + length - 1) >> 12).contains(&page)
    };
    LITERALS.iter().any(|&(offset, _)| contains(offset, 4))
        || TABLES
            .iter()
            .any(|&(offset, pointers)| contains(offset, pointers.len() * 4))
        || ARCHIVES
            .iter()
            .any(|spec| contains(spec.offset, spec.length))
}
