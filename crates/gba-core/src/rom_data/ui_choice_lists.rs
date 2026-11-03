//! Immutable EUR choice entries; copied lists, navigation and cursor tasks stay guest work.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_SHA256: &str = "74323d5436d4fa7dadbb51f571222031410c89fd708276f39b6adf36c7400f7f";
pub const ENTRY_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChoiceKind {
    DirectList,
    FilteredTemplate,
}
impl ChoiceKind {
    fn role(self) -> &'static str {
        match self {
            Self::DirectList => "ui-choice-list",
            Self::FilteredTemplate => "ui-choice-template",
        }
    }
}

/// Source projection sizes, never bounds on live guest count or navigation values.
pub const DEFINITIONS: &[(u32, usize, ChoiceKind)] = &[
    (0x3979bc, 4, ChoiceKind::DirectList),
    (0x3979dc, 3, ChoiceKind::DirectList),
    (0x3979f4, 4, ChoiceKind::DirectList),
    (0x397a14, 3, ChoiceKind::DirectList),
    (0x397a2c, 5, ChoiceKind::DirectList),
    (0x397a54, 4, ChoiceKind::DirectList),
    (0x397a9c, 6, ChoiceKind::FilteredTemplate),
    (0x397f50, 6, ChoiceKind::DirectList),
    (0x3adf68, 2, ChoiceKind::DirectList),
    (0x3adf78, 2, ChoiceKind::DirectList),
    (0x3adf88, 1, ChoiceKind::DirectList),
    (0x3adf90, 1, ChoiceKind::DirectList),
    (0x3adf98, 1, ChoiceKind::DirectList),
];
const ROOT_LITERALS: &[(usize, u32)] = &[
    (0x0796c8, 0x3979bc),
    (0x081384, 0x3979dc),
    (0x079604, 0x3979f4),
    (0x0812d8, 0x397a14),
    (0x07a338, 0x397a2c),
    (0x07a40c, 0x397a2c),
    (0x07b26c, 0x397a54),
    (0x07b33c, 0x397a54),
    (0x082588, 0x397a9c),
    (0x082728, 0x397a9c),
    (0x090f1c, 0x397f50),
    (0x13e400, 0x3adf68),
    (0x13e470, 0x3adf68),
    (0x13d590, 0x3adf78),
    (0x13e728, 0x3adf78),
    (0x13d6a0, 0x3adf88),
    (0x13d7ac, 0x3adf90),
    (0x13e7f8, 0x3adf98),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigation {
    pub left: u8,
    pub right: u8,
    pub up: u8,
    pub down: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceEntry {
    pub identifier: u8,
    pub cursor_x: u8,
    pub cursor_y: u8,
    /// Copied byte3; no padding-removal or zero constraint is inferred.
    pub opaque: u8,
    /// Entire original bytes, including flags and raw coordinate-address bits.
    pub navigation: Navigation,
}
impl ChoiceEntry {
    pub fn bytes(&self) -> [u8; ENTRY_BYTES] {
        [
            self.identifier,
            self.cursor_x,
            self.cursor_y,
            self.opaque,
            self.navigation.left,
            self.navigation.right,
            self.navigation.up,
            self.navigation.down,
        ]
    }
    fn from_bytes(b: &[u8]) -> Self {
        Self {
            identifier: b[0],
            cursor_x: b[1],
            cursor_y: b[2],
            opaque: b[3],
            navigation: Navigation {
                left: b[4],
                right: b[5],
                up: b[6],
                down: b[7],
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceList {
    /// Physical-root ID preserves equal contents at distinct source locations.
    pub id: String,
    pub source_offset: u32,
    pub kind: ChoiceKind,
    pub entries: Vec<ChoiceEntry>,
}
impl ChoiceList {
    pub fn byte(&self, relative: usize) -> Option<u8> {
        Some(self.entries.get(relative / ENTRY_BYTES)?.bytes()[relative % ENTRY_BYTES])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiChoiceLists {
    pub lists: Vec<ChoiceList>,
}
impl UiChoiceLists {
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        let n = self
            .lists
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)?;
        let list = self.lists.get(n)?;
        list.byte(at.checked_sub(list.source_offset)? as usize)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.lists.len() != DEFINITIONS.len() {
            return Err("UI choice list count mismatch".into());
        }
        for (list, &(offset, count, kind)) in self.lists.iter().zip(DEFINITIONS) {
            if list.id != format!("choice-{offset:06x}")
                || list.source_offset != offset
                || list.kind != kind
                || list.entries.len() != count
            {
                return Err("UI choice list ID/source/kind/extent mismatch".into());
            }
        }
        // Entry bytes are lossless values. No navigation-target, ID, coordinate,
        // opaque-byte or live count bounds follow from these physical extents.
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        Ok(self
            .lists
            .iter()
            .map(|list| {
                (
                    list.source_offset,
                    list.entries.iter().flat_map(ChoiceEntry::bytes).collect(),
                    list.kind.role().into(),
                )
            })
            .collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("UI choice lists differ from authenticated source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: UiChoiceLists,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("UI choice lists require exact European source ROM".into());
        }
        for &(at, offset) in ROOT_LITERALS {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != 0x08000000 + offset {
                return Err("UI choice list root literal mismatch".into());
            }
        }
        let decoded = UiChoiceLists {
            lists: DEFINITIONS
                .iter()
                .map(|&(offset, count, kind)| ChoiceList {
                    id: format!("choice-{offset:06x}"),
                    source_offset: offset,
                    kind,
                    entries: rom[offset as usize..offset as usize + count * ENTRY_BYTES]
                        .chunks_exact(ENTRY_BYTES)
                        .map(ChoiceEntry::from_bytes)
                        .collect(),
                })
                .collect(),
        };
        let mut sources = Vec::new();
        let mut combined = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("UI choice list source roundtrip mismatch".into());
            }
            combined.extend_from_slice(&bytes);
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        if ffta_rom::digest(&combined) != SOURCE_SHA256 {
            return Err("UI choice list source hash mismatch".into());
        }
        Ok(Self { decoded, sources })
    }
}
