//! EUR fixed ten-byte stack copies and their residual literal roots.

use crate::ffta_rom;
use serde::{Deserialize, Serialize};

const ROM_BASE: u32 = 0x0800_0000;
pub const COPY_BYTES: usize = 10;
pub(super) const COPY_HELPER: u32 = 0x147dc0;
const COPY_HELPER_HASH: &str = "aa822d873bad381a62e3c3e07b06003a41beea59bc3edc791f7e316eb08b7ebb";
pub const PAYLOAD_ROLE: &str = "stack-byte-copy-data";
pub const LITERAL_ROLE: &str = "stack-byte-copy-root";

pub(super) fn verify_copy_helper(rom: &[u8]) -> Result<(), String> {
    if ffta_rom::digest(&rom[COPY_HELPER as usize..0x147e22]) != COPY_HELPER_HASH {
        return Err("Stack byte copy helper differs from the original".into());
    }
    Ok(())
}

pub struct Reader {
    pub entry: u32,
    pub call: u32,
    pub literal: u32,
    pub source: u32,
    pub stack_offset: u16,
    pub owns_literal: bool,
    pub reader_sha256: &'static str,
    pub source_sha256: &'static str,
}

// E565 bounds the seven immediate-count readers. The first two roots
// already belong to engine.thumb-pc-literals.
pub const READERS: [Reader; 7] = [
    Reader {
        entry: 0x0357fc,
        call: 0x035804,
        literal: 0x035824,
        source: 0x14d360,
        stack_offset: 0x1c,
        owns_literal: false,
        reader_sha256: "ea0945217ed5618e81384a259aa5bc152c4e4e783e0dcaf062fc6982abc58878",
        source_sha256: "3ddd0ba1b3d1e6e3e260ba66fc0ee64a03ab78d8780e89c6c7402d59eebbd991",
    },
    Reader {
        entry: 0x035a30,
        call: 0x035a3a,
        literal: 0x035aa0,
        source: 0x14d36a,
        stack_offset: 0x1c,
        owns_literal: false,
        reader_sha256: "c0cb38af93c002b4d6745467d52c83cd67c1c206c1884d5009fb6336a1aa15a3",
        source_sha256: "ac66da3052d2e45dea2397429ec698097f5f66628fb3ecec97a33ec4522a99df",
    },
    Reader {
        entry: 0x045be2,
        call: 0x045be8,
        literal: 0x045c54,
        source: 0x14d374,
        stack_offset: 0,
        owns_literal: true,
        reader_sha256: "9f51b48e8f580598413637e537bd8bf7b03a726652122eca45c5c941326ed3ca",
        source_sha256: "1ea1ffd655f132b6b66331da94b8f7e9f7197f8c5726846f4ef5bb26bb8c15d1",
    },
    Reader {
        entry: 0x113264,
        call: 0x11326c,
        literal: 0x11338c,
        source: 0x14d92c,
        stack_offset: 0x38,
        owns_literal: true,
        reader_sha256: "ebe972ad2aa69611dbd82a0ff57cbd2753347e2183a5248b72b691fd6d3ea0c2",
        source_sha256: "cffd45578cbabe66a56b964ca9054b2db546d20a9dec759577985542929f8e9b",
    },
    Reader {
        entry: 0x11588e,
        call: 0x115894,
        literal: 0x1158f4,
        source: 0x14d938,
        stack_offset: 0,
        owns_literal: true,
        reader_sha256: "be4a3b78b108f692b8cc186290d0dbc90b185ef7cd7a0d9280ba9d58ade60abb",
        source_sha256: "cffd45578cbabe66a56b964ca9054b2db546d20a9dec759577985542929f8e9b",
    },
    Reader {
        entry: 0x11826c,
        call: 0x118272,
        literal: 0x118374,
        source: 0x14d944,
        stack_offset: 0x10,
        owns_literal: true,
        reader_sha256: "e46d62e1735f54ccc4549aceaa515ae48877204ceafad66d6ed30d46157dc301",
        source_sha256: "52b45ba590b94aa2811d53c34262950dd32851b7791fe7992940130ac3a67b7d",
    },
    Reader {
        entry: 0x118276,
        call: 0x118280,
        literal: 0x118378,
        source: 0x14d94e,
        stack_offset: 0x1c,
        owns_literal: true,
        reader_sha256: "c356f77c4a5f2322c782339cc9f67eab22792178d6a1d2f4a26d14d0598336f1",
        source_sha256: "c94465a1850d392a36be81e127ca5f1fde37155a27784bddf8a133b5c68c59d7",
    },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackByteCopy {
    pub source_offset: u32,
    pub bytes: [u8; COPY_BYTES],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackByteCopies {
    copies: [StackByteCopy; READERS.len()],
}

impl StackByteCopies {
    pub(super) fn from_rom(rom: &[u8]) -> Result<Self, String> {
        verify_copy_helper(rom)?;
        for reader in &READERS {
            let entry = reader.entry as usize;
            let stop = reader.call as usize + 4;
            let root = reader.literal as usize;
            let source = reader.source as usize;
            if ffta_rom::digest(&rom[entry..stop]) != reader.reader_sha256
                || u32::from_le_bytes(rom[root..root + 4].try_into().unwrap())
                    != ROM_BASE + reader.source
                || ffta_rom::digest(&rom[source..source + COPY_BYTES]) != reader.source_sha256
            {
                return Err(format!(
                    "Stack byte copy source or reader mismatch at {entry:06x}"
                ));
            }
        }
        Ok(Self {
            copies: std::array::from_fn(|index| {
                let source_offset = READERS[index].source;
                let start = source_offset as usize;
                StackByteCopy {
                    source_offset,
                    bytes: rom[start..start + COPY_BYTES].try_into().unwrap(),
                }
            }),
        })
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        for (copy, reader) in self.copies.iter().zip(&READERS) {
            if copy.source_offset != reader.source {
                return Err("Invalid stack byte copy source order".into());
            }
        }
        Ok(())
    }

    pub fn source_spans(&self) -> impl Iterator<Item = (u32, Vec<u8>, &'static str)> + '_ {
        let payloads = self
            .copies
            .iter()
            .map(|copy| (copy.source_offset, copy.bytes.to_vec(), PAYLOAD_ROLE));
        let roots = READERS
            .iter()
            .filter(|reader| reader.owns_literal)
            .map(|reader| {
                (
                    reader.literal,
                    (ROM_BASE + reader.source).to_le_bytes().to_vec(),
                    LITERAL_ROLE,
                )
            });
        payloads.chain(roots)
    }

    pub(super) fn byte_at(&self, offset: u32) -> Option<(u8, bool)> {
        for (copy, reader) in self.copies.iter().zip(&READERS) {
            if let Some(byte) = offset
                .checked_sub(copy.source_offset)
                .and_then(|index| copy.bytes.get(index as usize))
            {
                return Some((*byte, false));
            }
            if reader.owns_literal {
                if let Some(byte) = offset.checked_sub(reader.literal).and_then(|index| {
                    (ROM_BASE + reader.source)
                        .to_le_bytes()
                        .get(index as usize)
                        .copied()
                }) {
                    return Some((byte, true));
                }
            }
        }
        None
    }
}
