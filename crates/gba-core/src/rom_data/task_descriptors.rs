//! Bounded European task descriptor catalogs. Guest selectors and scheduling stay live.
use super::{palette_effect_dispatch::ThumbEntry, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLES: [(u32, usize, &str); 3] = [
    (0x3997b0, 341, "primary"),
    (0x39a258, 20, "auxiliary"),
    (0x39a2f8, 51, "script"),
];
pub const SOURCE_BYTES: usize = 3296;

/// Original 080d7018 address arithmetic, including low16 truncation and ID0.
pub fn primary_source_offset(raw_id: u32) -> u32 {
    let id = u32::from(raw_id as u16);
    let (root, first) = if id <= 341 {
        (0x3997b0u32, 1)
    } else {
        (0x39a258u32, 342)
    };
    root.wrapping_add(id.wrapping_sub(first).wrapping_mul(8))
}

/// Original 080d7328 arithmetic. Lower IDs alias earlier physical catalogs.
pub fn secondary_source_offset(raw_id: u32) -> u32 {
    0x39a2f8u32.wrapping_add(u32::from(raw_id as u16).wrapping_sub(363).wrapping_mul(8))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub ordinal: u16,
    pub callback: Option<ThumbEntry>,
    pub context_size: u16,
    /// Original +6 halfword; +6 low byte is independently compared with 0x10.
    /// Keep the full bit pattern without assigning unproved flag semantics.
    pub metadata_raw: u16,
}
impl Descriptor {
    pub fn callback_word(&self) -> Option<u32> {
        self.callback
            .as_ref()
            .map_or(Some(0), ThumbEntry::guest_word)
    }
    pub fn byte(&self, relative: usize) -> Option<u8> {
        Some(match relative {
            0..=3 => self.callback_word()?.to_le_bytes()[relative],
            4..=5 => self.context_size.to_le_bytes()[relative - 4],
            6..=7 => self.metadata_raw.to_le_bytes()[relative - 6],
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptorTable {
    pub source_offset: u32,
    pub entries: Vec<Descriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDescriptors {
    pub primary: DescriptorTable,
    pub auxiliary: DescriptorTable,
    pub script: DescriptorTable,
}
impl TaskDescriptors {
    pub fn tables(&self) -> [&DescriptorTable; 3] {
        [&self.primary, &self.auxiliary, &self.script]
    }
    /// Physical projection membership only; None requests the existing raw owner.
    pub fn descriptor_at(&self, source_offset: u32) -> Option<&Descriptor> {
        for table in self.tables() {
            if let Some(relative) = source_offset.checked_sub(table.source_offset) {
                if relative % 8 == 0 {
                    if let Some(entry) = table.entries.get(relative as usize / 8) {
                        return Some(entry);
                    }
                }
            }
        }
        None
    }
    pub fn byte_at(&self, source_offset: u32) -> Option<u8> {
        let row_offset = source_offset & !7;
        self.descriptor_at(row_offset)?
            .byte((source_offset & 7) as usize)
    }
    pub fn validate(&self) -> Result<(), String> {
        for (table, (offset, count, _)) in self.tables().into_iter().zip(TABLES) {
            if table.source_offset != offset || table.entries.len() != count {
                return Err("Task descriptor catalog source/count mismatch".into());
            }
            for (n, entry) in table.entries.iter().enumerate() {
                if usize::from(entry.ordinal) != n || entry.callback_word().is_none() {
                    return Err("Task descriptor ordinal/Thumb entry mismatch".into());
                }
            }
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        self.tables()
            .into_iter()
            .zip(TABLES)
            .map(|(table, (offset, count, role))| {
                let bytes = (0..count * 8)
                    .map(|i| {
                        self.byte_at(offset + i as u32)
                            .ok_or_else(|| "Invalid task descriptor byte".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    table.source_offset,
                    bytes,
                    format!("engine/task-descriptors/{role}"),
                ))
            })
            .collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Task descriptors differ from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: TaskDescriptors,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Task descriptors require exact European source ROM".into());
        }
        verify_contract(rom)?;
        let mut tables = Vec::new();
        for (offset, count, _) in TABLES {
            let mut entries = Vec::new();
            for ordinal in 0..count {
                let at = offset as usize + ordinal * 8;
                let word = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
                let callback = if word == 0 {
                    None
                } else {
                    if word & 1 == 0 || !(0x08000001..0x09000000).contains(&word) {
                        return Err("Task descriptor callback is not a ROM Thumb entry".into());
                    }
                    Some(ThumbEntry {
                        rom_offset: (word & !1) - 0x08000000,
                    })
                };
                entries.push(Descriptor {
                    ordinal: ordinal as u16,
                    callback,
                    context_size: u16::from_le_bytes(rom[at + 4..at + 6].try_into().unwrap()),
                    metadata_raw: u16::from_le_bytes(rom[at + 6..at + 8].try_into().unwrap()),
                });
            }
            tables.push(DescriptorTable {
                source_offset: offset,
                entries,
            });
        }
        let mut tables = tables.into_iter();
        let decoded = TaskDescriptors {
            primary: tables.next().unwrap(),
            auxiliary: tables.next().unwrap(),
            script: tables.next().unwrap(),
        };
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Task descriptor source roundtrip mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        Ok(Self { decoded, sources })
    }
}

fn verify_contract(rom: &[u8]) -> Result<(), String> {
    // Fingerprints appended from the separately recorded original-reader inventory.
    for (offset, expected) in CONTRACT_HALFWORDS {
        if u16::from_le_bytes(
            rom[*offset as usize..*offset as usize + 2]
                .try_into()
                .unwrap(),
        ) != *expected
        {
            return Err("Task descriptor original selector/reader contract mismatch".into());
        }
    }
    for (offset, expected) in CONTRACT_WORDS {
        if u32::from_le_bytes(
            rom[*offset as usize..*offset as usize + 4]
                .try_into()
                .unwrap(),
        ) != *expected
        {
            return Err("Task descriptor original root/boundary contract mismatch".into());
        }
    }
    // Independent kind0 directory row2 selects secondaryID413, the last source row.
    if rom[0x3aca2c] != 0 || u16::from_le_bytes([rom[0x3aca2e], rom[0x3aca2f]]) != 413 {
        return Err("Task descriptor final source reference mismatch".into());
    }
    Ok(())
}

const CONTRACT_HALFWORDS: &[(u32, u16)] = &[
    (0xd7030, 0x480d),
    (0xd7032, 0x4284),
    (0xd7034, 0xd81c),
    (0xd705a, 0x4904),
    (0xd705c, 0x1e60),
    (0xd705e, 0xc0),
    (0xd7060, 0x1840),
    (0xd7062, 0x6803),
    (0xd7064, 0x8882),
    (0xd7070, 0x4816),
    (0xd7072, 0x4a17),
    (0xd7074, 0x18a1),
    (0xd7076, 0xc9),
    (0xd7078, 0x1809),
    (0xd707a, 0x680b),
    (0xd707c, 0x888a),
    (0xd709a, 0x490f),
    (0xd709c, 0x1e70),
    (0xd709e, 0xc0),
    (0xd70a0, 0x1840),
    (0xd70a2, 0x88c0),
    (0xd70a4, 0x82d0),
    (0xd733c, 0x4909),
    (0xd733e, 0x4a0a),
    (0xd7340, 0x1880),
    (0xd7342, 0xc0),
    (0xd7344, 0x1840),
    (0xd7346, 0x6801),
    (0xd7348, 0x8882),
    (0xd76ac, 0x409),
    (0xd76ae, 0x4809),
    (0xd76b0, 0xb49),
    (0xd76b2, 0x3908),
    (0xd76b4, 0x1809),
    (0xd76b6, 0x7988),
    (0xd76b8, 0x2810),
    (0xd9b56, 0x4812),
    (0xd9b82, 0xf737),
    (0xd9b84, 0xfc57),
    (0x124eb8, 0x4a0e),
    (0x124eba, 0x8838),
    (0x124ebc, 0x100),
    (0x124ebe, 0x1880),
    (0x124ec0, 0x8840),
    (0x124ec2, 0x8138),
    (0x124f7a, 0x8938),
    (0x124f80, 0xf7b2),
    (0x124f82, 0xf9d2),
    (0x124f54, 0x118),
    (0x124f56, 0x1c21),
    (0x124f58, 0x310c),
    (0x124f5a, 0x1840),
    (0x124f5c, 0x6800),
    (0x124f5e, 0x2801),
    (0x124f60, 0xd101),
    (0x124f66, 0x2802),
    (0x124f68, 0xd102),
    (0x124f70, 0x2803),
    (0x124f72, 0xd102),
];

const CONTRACT_WORDS: &[(u32, u32)] = &[
    (0xd7068, 0x155),
    (0xd706c, 0x83997b0),
    (0xd70cc, 0x839a258),
    (0xd70d0, 0xfffffeaa),
    (0xd70d4, 0x159),
    (0xd70d8, 0x83997b0),
    (0xd7364, 0x839a2f8),
    (0xd7368, 0xfffffe95),
    (0xd76d4, 0x83997b0),
    (0xd9ba0, 0x839a490),
    (0x124ef4, 0x83aca0c),
    (0x124efc, 0x08124f54),
];
