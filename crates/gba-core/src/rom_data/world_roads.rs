//! Conditional EUR sampled world roads. Movement/pathfinding remain guest code.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const HEADERS: u32 = 0x397018;
/// Headers selected by the 45 path IDs in the declared world-node projection.
/// Other byte indices remain outside this owner, not invalid guest inputs.
pub const PATH_COUNT: usize = 45;
pub const HEADER_BYTES: usize = 8;
pub const POINTS_START: u32 = 0x395f00;
pub const SOURCE_END: u32 = HEADERS + (PATH_COUNT * HEADER_BYTES) as u32;
pub const SOURCE_BYTES: usize = (SOURCE_END - POINTS_START) as usize;
pub const SOURCE_SHA256: &str = "e0a6a0a205c1dba5a9b0681309a7261056072d51c8cf7a4c1ce4814369caa8ec";
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: i8,
    pub y: i8,
}
impl Point {
    /// Original 4DD0C sign-extends each component into a halfword.
    pub fn packed(self) -> u32 {
        u32::from(self.x as i16 as u16) | (u32::from(self.y as i16 as u16) << 16)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Path {
    pub index: u8,
    /// Also consumed as a path-cost comparison term by original 4DC8C.
    pub point_count: u8,
    pub opaque: [u8; 3],
    pub encoded_pointer: u32,
    /// Sampled offsets from the lower-index node; never incremental deltas.
    pub points: Vec<Point>,
}
impl Path {
    pub fn points_offset(&self) -> Option<u32> {
        self.encoded_pointer.checked_sub(0x08000000)
    }
    fn header_bytes(&self) -> [u8; HEADER_BYTES] {
        let mut raw = [0; HEADER_BYTES];
        raw[0] = self.point_count;
        raw[1..4].copy_from_slice(&self.opaque);
        raw[4..8].copy_from_slice(&self.encoded_pointer.to_le_bytes());
        raw
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldRoads {
    pub headers_offset: u32,
    pub paths: Vec<Path>,
}
impl WorldRoads {
    pub fn validate(&self) -> Result<(), String> {
        if self.headers_offset != HEADERS
            || self.paths.len() != PATH_COUNT
            || self.paths.iter().enumerate().any(|(i, path)| {
                usize::from(path.index) != i || path.points.len() != usize::from(path.point_count)
            })
        {
            return Err("Invalid conditional EUR world-road projection".into());
        }
        // Retain this projection's payload partition. Counts are full u8 values;
        // no inferred 32..85 cap, opaque-zero or endpoint-normalization policy.
        let mut ranges = Vec::new();
        for path in &self.paths {
            let at = path.points_offset().ok_or("Invalid road ROM pointer")?;
            let end = at
                .checked_add(path.points.len() as u32 * 2)
                .ok_or("Road sample range overflow")?;
            if at < POINTS_START || end > HEADERS {
                return Err("Road samples outside declared source partition".into());
            }
            ranges.push((at, end));
        }
        ranges.sort_unstable();
        let mut next = POINTS_START;
        for (at, end) in ranges {
            if at != next {
                return Err("Overlapping or incomplete road sample partition".into());
            }
            next = end;
        }
        if next != HEADERS {
            return Err("Incomplete road sample partition".into());
        }
        Ok(())
    }
    /// Original inputs alias by their low bytes. None requires raw fallback;
    /// Some(0) is the original output when step >= the declared sample count.
    pub fn packed_point(&self, direction: u32, index: u32, step: u32) -> Option<u32> {
        let path = self.paths.get(usize::from(index as u8))?;
        let step = step as u8;
        if step >= path.point_count {
            return Some(0);
        }
        let sample = if direction as u8 == 0 {
            step
        } else {
            path.point_count - step - 1
        };
        Some(path.points.get(usize::from(sample))?.packed())
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut spans = vec![(
            self.headers_offset,
            self.paths.iter().flat_map(Path::header_bytes).collect(),
            "world-road-headers".into(),
        )];
        for path in &self.paths {
            spans.push((
                path.points_offset().ok_or("Invalid road ROM pointer")?,
                path.points
                    .iter()
                    .flat_map(|p| [p.x as u8, p.y as u8])
                    .collect(),
                "world-road-sampled-points".into(),
            ));
        }
        spans.sort_by_key(|s| s.0);
        Ok(spans)
    }
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        if let Some(byte) = offset.checked_sub(self.headers_offset) {
            if let Some(path) = self.paths.get(byte as usize / HEADER_BYTES) {
                return Some(path.header_bytes()[byte as usize % HEADER_BYTES]);
            }
        }
        for path in &self.paths {
            let byte = match offset.checked_sub(path.points_offset()?) {
                Some(b) => b as usize,
                None => continue,
            };
            if let Some(point) = path.points.get(byte / 2) {
                return Some(if byte % 2 == 0 {
                    point.x as u8
                } else {
                    point.y as u8
                });
            }
        }
        None
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("World roads differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

/// Original 4DD70 and 4DD84 compare unsigned low-byte node IDs.
pub fn reverse_direction(current: u32, target: u32) -> u8 {
    u8::from(current as u8 > target as u8)
}
pub fn base_node(current: u32, target: u32) -> u8 {
    (current as u8).min(target as u8)
}

pub struct Catalog {
    pub decoded: WorldRoads,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("World roads require exact EUR ROM; USA pending".into());
        }
        for (start, length, sha) in [
            (
                0x4dd0c,
                144,
                "1a4fb16a5244fea6b38815b1d512477add801d09d5560a098289c1c377986bd1",
            ),
            (
                0x46b64,
                180,
                "c58e13a10d8842382fee3650101467b97cf2f3cb67dac9a5d2a762a1e729a40a",
            ),
            (
                0x31488,
                220,
                "5109bfd44f224aeea656cff23be099c3a8e79b51c3b322e4d9c335088c1b4aaa",
            ),
            (
                0x4dc30,
                280,
                "41e0610c15653524002ef1dd012758ae1bf5aabb9bc2135fb3f56e315792ce0c",
            ),
            (
                0x47034,
                200,
                "034162fe5a7ae10f6baffca0241ce74fd1df311dd716e2a755dd829a3601a11b",
            ),
            (
                0x47100,
                224,
                "a63c869078503d0cb4b4168302c03b5cc8c1c3864698a769c20bdcbb60c68f65",
            ),
        ] {
            if ffta_rom::digest(&rom[start..start + length]) != sha {
                return Err("Original world-road producer/reader fingerprint mismatch".into());
            }
        }
        if ffta_rom::digest(&rom[POINTS_START as usize..SOURCE_END as usize]) != SOURCE_SHA256 {
            return Err("World-road source fingerprint mismatch".into());
        }
        let paths = rom[HEADERS as usize..SOURCE_END as usize]
            .chunks_exact(HEADER_BYTES)
            .enumerate()
            .map(|(index, raw)| {
                let encoded_pointer = u32::from_le_bytes(raw[4..8].try_into().unwrap());
                let at = (encoded_pointer - 0x08000000) as usize;
                Path {
                    index: index as u8,
                    point_count: raw[0],
                    opaque: raw[1..4].try_into().unwrap(),
                    encoded_pointer,
                    points: rom[at..at + usize::from(raw[0]) * 2]
                        .chunks_exact(2)
                        .map(|p| Point {
                            x: p[0] as i8,
                            y: p[1] as i8,
                        })
                        .collect(),
                }
            })
            .collect();
        let decoded = WorldRoads {
            headers_offset: HEADERS,
            paths,
        };
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
