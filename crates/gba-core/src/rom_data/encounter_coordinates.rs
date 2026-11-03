//! Header-linked XY payloads. Live placement, RNG, mode and null gates stay guest code.
use super::{encounter_headers, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE: u32 = 0x70b6a8;
pub const SOURCE_END: u32 = 0x70c8f4;
pub const SOURCE_BYTES: usize = 4684;
pub const LIST_COUNT: usize = 691;
pub const PAIR_COUNT: usize = 2342;
pub const SOURCE_SHA256: &str = "f5b215d130be9fc5536627f67c0dbe39d9bdb6738325d1ea8d6c16642af4e3d4";
pub type SourceSpan = (u32, Vec<u8>, String);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}
impl Point {
    /// The search reader sign-extends the same physical bytes; other readers do not.
    pub fn signed(self) -> [i8; 2] {
        [self.x as i8, self.y as i8]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinateList {
    pub index: u16,
    pub header_index: u16,
    /// Header pointer slots0/1/2/3: placement0/random/placement2/default.
    pub slot: u8,
    pub source_offset: u32,
    pub points: Vec<Point>,
}
impl CoordinateList {
    pub fn source_bytes(&self) -> Vec<u8> {
        self.points.iter().flat_map(|p| [p.x, p.y]).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterCoordinates {
    pub source_offset: u32,
    pub lists: Vec<CoordinateList>,
}
impl EncounterCoordinates {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE || self.lists.len() != LIST_COUNT {
            return Err("Invalid declared EUR coordinate catalog shape".into());
        }
        let mut cursor = SOURCE;
        for (i, l) in self.lists.iter().enumerate() {
            if usize::from(l.index) != i || l.slot > 3 || l.source_offset != cursor {
                return Err("Invalid coordinate list order/slot/extent".into());
            }
            let bytes = l
                .points
                .len()
                .checked_mul(2)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or("Coordinate extent overflow")?;
            cursor = cursor
                .checked_add(bytes)
                .filter(|&z| z <= SOURCE_END)
                .ok_or("Coordinate extent outside declared catalog")?;
        }
        if cursor != SOURCE_END {
            return Err("Coordinate lists do not cover declared bytes".into());
        }
        // No coordinate range, observed role-count caps or guest fallback policy.
        Ok(())
    }
    pub fn validate_headers(
        &self,
        owner: &encounter_headers::EncounterHeaders,
    ) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        let mut cursor = 0;
        for h in &owner.headers {
            for slot in 0..4 {
                let pointer = h.position_pointers[slot];
                if pointer == 0 {
                    continue;
                }
                let n = [
                    h.placement_count0,
                    h.random_position_range,
                    h.placement_count2,
                    1,
                ][slot];
                let l = self
                    .lists
                    .get(cursor)
                    .ok_or("Missing coordinate list for header pointer")?;
                if l.header_index != h.index
                    || usize::from(l.slot) != slot
                    || pointer != 0x08000000 + l.source_offset
                    || l.points.len() != usize::from(n)
                {
                    return Err("Coordinate list differs from header pointer/count/role".into());
                }
                cursor += 1;
            }
        }
        if cursor != self.lists.len() {
            return Err("Unexpected coordinate list without header link".into());
        }
        Ok(())
    }
    /// Diagnostic lookup. An absent list does not implement a null guest fallback.
    pub fn list(&self, header: u16, slot: u8) -> Option<&CoordinateList> {
        self.lists
            .iter()
            .find(|l| l.header_index == header && l.slot == slot)
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if !(self.source_offset..SOURCE_END).contains(&offset) {
            return None;
        }
        let index = self
            .lists
            .partition_point(|l| l.source_offset <= offset)
            .checked_sub(1)?;
        let l = &self.lists[index];
        let at = (offset - l.source_offset) as usize;
        let p = l.points.get(at / 2)?;
        Some(if at % 2 == 0 { p.x } else { p.y })
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        Ok(self
            .lists
            .iter()
            .map(|l| {
                (
                    l.source_offset,
                    l.source_bytes(),
                    format!(
                        "encounter-coordinate/{}",
                        ["placement0", "random", "placement2", "default"][usize::from(l.slot)]
                    ),
                )
            })
            .collect())
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Coordinates differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: EncounterCoordinates,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Encounter coordinates require exact EUR ROM; USA pending".into());
        }
        if ffta_rom::digest(&rom[SOURCE as usize..SOURCE_END as usize]) != SOURCE_SHA256 {
            return Err("Coordinate source fingerprint mismatch".into());
        }
        for (start, length, sha) in [
            (
                0xbd01c,
                160,
                "7e09a4f8c576d67c09f946769a3eafdda541320532e5fba7947d4e536376201c",
            ),
            (
                0xbd31a,
                192,
                "50e4f788423574f3cb26ec6b276452d428eb803af5575639b0819230a296b199",
            ),
            (
                0xbc18a,
                144,
                "200c443e2c775ba306c2f4d6c58490d921cad75c0f0bf8c99acada33dd15988d",
            ),
            (
                0x1286e0,
                144,
                "4d2586ad69fe0f87140d2a6593522142a2f5f897aab7260753a2553de8079736",
            ),
            (
                0x128770,
                144,
                "190005cbb59f30eaff59e6a6ed2f1b9168aa466f750e8b6baebbefe8e7727bb1",
            ),
            (
                0xcbe50,
                144,
                "592bb1cfa6f0235ea98bdf403eca2fa8e0c0a63eb65317f7ef4f5359cf685ba3",
            ),
            (
                0xcbe8c,
                144,
                "a88eabe6174bed0c1971caaa444a23990c51a69a2cce51059d7574b36c6865b7",
            ),
            (
                0xbcfa2,
                144,
                "987f320dd764eccff3cb5d323585a35ca79bf09a8f9795f89e255e5db78a36b3",
            ),
            (
                0xbd296,
                144,
                "b57b27af0b2f09fdc15df93c2c226a885797e1421bfa581a027c8e00e77aaf38",
            ),
            (
                0x146314,
                96,
                "245a8ff340a464142c4c053407a542e5f2c99a9fbe69210c027f3a04a96d087a",
            ),
            (
                0x146374,
                112,
                "8446f82b1ac8bf5c9fa0a5b784a26c38f098f6c0f8ae50a0559d45fd733bcd2f",
            ),
            (
                0x2868,
                88,
                "3f2401eada054f803153269e37b4456dbd487ed9e2254a49393a8bb29501b5cf",
            ),
        ] {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Original coordinate reader fingerprint mismatch".into());
            }
        }
        let headers = encounter_headers::Catalog::from_rom(rom)?.decoded;
        let mut lists = Vec::with_capacity(LIST_COUNT);
        for h in &headers.headers {
            for slot in 0..4 {
                let pointer = h.position_pointers[slot];
                if pointer == 0 {
                    continue;
                }
                let n = [
                    h.placement_count0,
                    h.random_position_range,
                    h.placement_count2,
                    1,
                ][slot];
                let source_offset = pointer - 0x08000000;
                let start = source_offset as usize;
                let points = rom[start..start + usize::from(n) * 2]
                    .chunks_exact(2)
                    .map(|v| Point { x: v[0], y: v[1] })
                    .collect();
                lists.push(CoordinateList {
                    index: lists.len() as u16,
                    header_index: h.index,
                    slot: slot as u8,
                    source_offset,
                    points,
                });
            }
        }
        let decoded = EncounterCoordinates {
            source_offset: SOURCE,
            lists,
        };
        decoded.validate_headers(&headers)?;
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
