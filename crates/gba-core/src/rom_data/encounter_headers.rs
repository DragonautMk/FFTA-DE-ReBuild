//! Declared EUR encounter headers; unit/position payloads and live algorithms stay separate.
use super::{constructor_alternates, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const INDEX_ORIGIN: u32 = 0x70c8cc;
pub const SOURCE: u32 = 0x70c8f4;
pub const SOURCE_END: u32 = 0x710e04;
pub const HEADER_COUNT: usize = 442;
pub const HEADER_BYTES: usize = 40;
pub const SOURCE_BYTES: usize = HEADER_COUNT * HEADER_BYTES;
pub const OWNED_BYTES: usize = SOURCE_BYTES - 4;
pub const SOURCE_SHA256: &str = "580d0702d55d148643c3d920282eb52f73afc2514e0f1f95564e0e0e0b569ee6";
pub const OPAQUE_OFFSETS: [usize; 7] = [3, 9, 11, 15, 35, 37, 39];
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub index: u16,
    pub unit_count: u16,
    pub map_argument: u8,
    pub unit_templates_pointer: u32,
    pub placement_parameter0: u8,
    pub placement_parameter2: u8,
    pub placement_count0: u8,
    pub random_position_range: u8,
    pub placement_count2: u8,
    /// Exact encoded words. Pointer3's first XY is read; no extent is inferred.
    pub position_pointers: [u32; 4],
    pub unit_count_comparison: u8,
    /// A full byte tested for nonzero, not a canonical boolean.
    pub nonzero_gate: u8,
    pub switch_selector: u8,
    pub zero_gate: u8,
    pub script_default: u8,
    /// Source offsets 3/9/11/15/35/37/39, in that order.
    pub opaque: [u8; 7],
}
impl Header {
    pub fn source_bytes(&self) -> [u8; HEADER_BYTES] {
        let mut raw = [0; HEADER_BYTES];
        raw[..2].copy_from_slice(&self.unit_count.to_le_bytes());
        raw[2] = self.map_argument;
        raw[4..8].copy_from_slice(&self.unit_templates_pointer.to_le_bytes());
        raw[8] = self.placement_parameter0;
        raw[10] = self.placement_parameter2;
        raw[12] = self.placement_count0;
        raw[13] = self.random_position_range;
        raw[14] = self.placement_count2;
        for (i, pointer) in self.position_pointers.iter().enumerate() {
            raw[16 + i * 4..20 + i * 4].copy_from_slice(&pointer.to_le_bytes());
        }
        raw[32] = self.unit_count_comparison;
        raw[33] = self.nonzero_gate;
        raw[34] = self.switch_selector;
        raw[36] = self.zero_gate;
        raw[38] = self.script_default;
        for (&offset, &byte) in OPAQUE_OFFSETS.iter().zip(&self.opaque) {
            raw[offset] = byte;
        }
        raw
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterHeaders {
    pub source_offset: u32,
    pub headers: Vec<Header>,
}
impl EncounterHeaders {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE
            || self.headers.len() != HEADER_COUNT
            || self
                .headers
                .iter()
                .enumerate()
                .any(|(i, h)| usize::from(h.index) != i + 1)
        {
            return Err("Invalid declared EUR encounter-header shape".into());
        }
        // No inferred count caps, pointer validity, enum sets or opaque-zero policy.
        Ok(())
    }
    pub fn validate_constructor_alternates(
        &self,
        owner: &constructor_alternates::Alternates,
    ) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if self.headers[28].unit_templates_pointer != 0x08000000 + owner.source_offset {
            return Err("Encounter header29 differs from shared constructor pointer".into());
        }
        Ok(())
    }
    /// Original inputs alias by low16. Index0 is coordinate data, not a header;
    /// indices outside 1..442 require prior-owner/raw fallback, not a guest cap.
    pub fn header(&self, index: u32) -> Option<&Header> {
        self.headers
            .get(usize::from((index as u16).checked_sub(1)?))
    }
    /// Original C0034 returns zero/one for byte33. No replacement of live routines.
    pub fn nonzero_gate(&self, index: u32) -> Option<bool> {
        Some(self.header(index)?.nonzero_gate != 0)
    }
    /// Only newly owned bytes. The existing constructor owner retains four bytes.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if (constructor_alternates::POINTER_OFFSET..constructor_alternates::POINTER_OFFSET + 4)
            .contains(&offset)
        {
            return None;
        }
        let at = offset.checked_sub(self.source_offset)? as usize;
        Some(self.headers.get(at / HEADER_BYTES)?.source_bytes()[at % HEADER_BYTES])
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let full: Vec<u8> = self.headers.iter().flat_map(Header::source_bytes).collect();
        let shared = (constructor_alternates::POINTER_OFFSET - SOURCE) as usize;
        Ok(vec![
            (
                SOURCE,
                full[..shared].to_vec(),
                "encounter-record-header-fragment".into(),
            ),
            (
                constructor_alternates::POINTER_OFFSET + 4,
                full[shared + 4..].to_vec(),
                "encounter-record-header-fragment".into(),
            ),
        ])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        // Compare the full model, including the shared field absent from new spans.
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Encounter headers differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: EncounterHeaders,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Encounter headers require exact EUR ROM; USA pending".into());
        }
        // Independent first/next roots, unit count/pointer importer, pure gate,
        // and original live zero/switch readers. All windows have source evidence.
        for (start, length, sha) in [
            (
                0x981a,
                160,
                "a5b4ac6a2b700bb610c7fe67b9b0ac92a031baea02f6d9f1e9c235844cb8de89",
            ),
            (
                0x24506,
                160,
                "0a2ee6a785d8bc14c25d4d77355c94485681ddba6bd157b7bda9b0fd680c0886",
            ),
            (
                0xce18c,
                200,
                "5aaca0ca91fd70551612c015e800dbd7ebaecd3dcaf07fcfc8bf0fcc94277b01",
            ),
            (
                0xc0028,
                144,
                "425b44c5674339c7a452c8064edf0f8123b0365bcf7ccf3237c692d39593a368",
            ),
            (
                0x9f60,
                432,
                "9cb81b669e722942e2e94e05f2a9a653137d6b8e639978bf87aef8eb1c1c3ce5",
            ),
            (
                0x13edc,
                144,
                "a11ee9af25843eb43dd8b24fd42b19dd00ab234aa5acdf9a3bf0c7622cd28cbc",
            ),
        ] {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Original encounter-header root/reader fingerprint mismatch".into());
            }
        }
        let raw = &rom[SOURCE as usize..SOURCE_END as usize];
        if ffta_rom::digest(raw) != SOURCE_SHA256 {
            return Err("Encounter-header source fingerprint mismatch".into());
        }
        let headers = raw
            .chunks_exact(HEADER_BYTES)
            .enumerate()
            .map(|(i, b)| Header {
                index: i as u16 + 1,
                unit_count: u16::from_le_bytes(b[..2].try_into().unwrap()),
                map_argument: b[2],
                unit_templates_pointer: u32::from_le_bytes(b[4..8].try_into().unwrap()),
                placement_parameter0: b[8],
                placement_parameter2: b[10],
                placement_count0: b[12],
                random_position_range: b[13],
                placement_count2: b[14],
                position_pointers: std::array::from_fn(|j| {
                    u32::from_le_bytes(b[16 + j * 4..20 + j * 4].try_into().unwrap())
                }),
                unit_count_comparison: b[32],
                nonzero_gate: b[33],
                switch_selector: b[34],
                zero_gate: b[36],
                script_default: b[38],
                opaque: OPAQUE_OFFSETS.map(|offset| b[offset]),
            })
            .collect();
        let decoded = EncounterHeaders {
            source_offset: SOURCE,
            headers,
        };
        if decoded.headers[28].unit_templates_pointer
            != 0x08000000 + constructor_alternates::SOURCE_OFFSET
        {
            return Err("Shared encounter/constructor source pointer mismatch".into());
        }
        let sources = decoded
            .source_spans()?
            .into_iter()
            .map(|(offset, bytes, role)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests;
