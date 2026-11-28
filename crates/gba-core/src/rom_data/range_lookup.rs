//! EUR bounded range table; original selection and output writes stay guest-side.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE: u32 = 0x37140c;
pub const RECORD_COUNT: usize = 11;
pub const SOURCE_SHA256: &str = "4cae38f2a735087e740409ca58e5f19bce1614805347c472990b8419e493bb7d";
const TEXT_ID_SOURCE: u32 = 0x6e9d8a;
const TEXT_ID_COUNT: usize = 351;
const TEXT_ID_LITERALS: [u32; 7] = [
    0x029b84, 0x04d424, 0x04da64, 0x051064, 0x054110, 0x054c68, 0x068af8,
];
const TEXT_ID_ADDRESS: u32 = 0x0800_0000 + TEXT_ID_SOURCE;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub selector: u8,
    pub opaque_01: u8,
    pub lower: u16,
    pub upper: u16,
    pub opaque_tail: [u8; 2],
}
impl Record {
    pub fn source_bytes(&self) -> [u8; 8] {
        let lower = self.lower.to_le_bytes();
        let upper = self.upper.to_le_bytes();
        [
            self.selector,
            self.opaque_01,
            lower[0],
            lower[1],
            upper[0],
            upper[1],
            self.opaque_tail[0],
            self.opaque_tail[1],
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangeLookup {
    pub source_offset: u32,
    pub records: Vec<Record>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub localized_text_ids: Option<LocalizedTextIds>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedTextIds {
    pub reader_literals: [u32; TEXT_ID_LITERALS.len()],
    pub ids: Vec<u16>,
}

impl LocalizedTextIds {
    fn from_rom(rom: &[u8]) -> Self {
        let start = TEXT_ID_SOURCE as usize;
        Self {
            reader_literals: TEXT_ID_LITERALS.map(|at| {
                let start = at as usize;
                u32::from_le_bytes(rom[start..start + 4].try_into().unwrap())
            }),
            ids: rom[start..start + 2 * TEXT_ID_COUNT]
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.ids.len() != TEXT_ID_COUNT
            || self.reader_literals != [TEXT_ID_ADDRESS; TEXT_ID_LITERALS.len()]
        {
            return Err("Invalid European localized text ID table".into());
        }
        Ok(())
    }

    pub(crate) fn pack(&self) -> (Vec<u8>, Vec<SourceRange>) {
        let mut bytes = Vec::with_capacity(4 * TEXT_ID_LITERALS.len() + 2 * TEXT_ID_COUNT);
        let mut spans = Vec::with_capacity(TEXT_ID_LITERALS.len() + 1);
        let mut add = |offset, source: &[u8], role: &str| {
            bytes.extend_from_slice(source);
            spans.push(SourceRange {
                offset,
                length: source.len() as u32,
                sha256: ffta_rom::digest(source),
                role: role.into(),
            });
        };
        for (offset, value) in TEXT_ID_LITERALS.into_iter().zip(self.reader_literals) {
            add(offset, &value.to_le_bytes(), "text-id-table-literal");
        }
        let table: Vec<_> = self
            .ids
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        add(TEXT_ID_SOURCE, &table, "localized-text-ids");
        (bytes, spans)
    }

    pub(crate) fn source_page(index: usize) -> bool {
        let page = index >> 12;
        let start = TEXT_ID_SOURCE as usize;
        (start >> 12..=(start + 2 * TEXT_ID_COUNT - 1) >> 12).contains(&page)
            || TEXT_ID_LITERALS.iter().any(|&at| at as usize >> 12 == page)
    }
}

impl RangeLookup {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.records.len() != RECORD_COUNT {
            return Err("Invalid EUR range lookup projection".into());
        }
        if let Some(table) = &self.localized_text_ids {
            table.validate()?;
        }
        Ok(())
    }
    /// First inclusive match; no guest-side selector or index guard is added.
    pub fn select(&self, value: u16) -> Option<(u8, u16)> {
        self.records.iter().find_map(|r| {
            (r.lower <= value && value <= r.upper).then(|| (r.selector, value - r.lower))
        })
    }
    /// Source bytes for the original range descriptors.
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.records.iter().flat_map(Record::source_bytes).collect())
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let relative = usize::try_from(offset.checked_sub(self.source_offset)?).ok()?;
        Some(self.records.get(relative / 8)?.source_bytes()[relative % 8])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom_with_text_ids(rom, self.localized_text_ids.is_some())?.decoded
        {
            return Err("Range lookup differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: RangeLookup,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        Self::from_rom_with_text_ids(rom, false)
    }

    pub(crate) fn from_rom_with_text_ids(
        rom: &[u8],
        include_text_ids: bool,
    ) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Range lookup requires the exact EUR ROM".into());
        }
        for (start, end, digest) in [
            (
                0x1abc0,
                0x1abfc,
                "606c326ff038d2f2b25578b111df54c2152dc1f5517711579585cac3f704edd7",
            ),
            (
                0x1abfc,
                0x1acac,
                "6222c23857a045ad321e4fa8ea44e382a2772cf26d780e2228b43a5f8ff6d078",
            ),
            (
                0x1c7ba,
                0x1c834,
                "cc2b2ab8bb13164f5d73866b90bddbe5a826fcf00774c6e0838666f4f3efc6cc",
            ),
        ] {
            if ffta_rom::digest(&rom[start..end]) != digest {
                return Err("Range lookup reader fingerprint mismatch".into());
            }
        }
        let raw = &rom[SOURCE as usize..SOURCE as usize + RECORD_COUNT * 8];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Range lookup source fingerprint mismatch".into());
        }
        let records = raw
            .chunks_exact(8)
            .map(|r| Record {
                selector: r[0],
                opaque_01: r[1],
                lower: u16::from_le_bytes([r[2], r[3]]),
                upper: u16::from_le_bytes([r[4], r[5]]),
                opaque_tail: [r[6], r[7]],
            })
            .collect();
        let decoded = RangeLookup {
            source_offset: SOURCE,
            records,
            localized_text_ids: include_text_ids.then(|| LocalizedTextIds::from_rom(rom)),
        };
        decoded.validate()?;
        let mut sources = vec![SourceRange {
            offset: SOURCE,
            length: (RECORD_COUNT * 8) as u32,
            sha256: SOURCE_SHA256.into(),
            role: "bounded-range-lookup".into(),
        }];
        if let Some(table) = &decoded.localized_text_ids {
            sources.extend(table.pack().1);
        }
        Ok(Self { decoded, sources })
    }
}
