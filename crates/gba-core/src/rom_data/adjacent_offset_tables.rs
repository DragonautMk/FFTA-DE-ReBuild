//! EUR adjacent offset tables; original index arithmetic remains guest-owned.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0x3ad460;
pub const SOURCE_LENGTH: usize = 32;
pub const TABLE_COUNT: usize = 5;
pub const SOURCE_SHA256: &str = "296d15a9ce5b40a4a58c76c71b76eedc966fd1cf55fd8db3ae5b10b3a4d75dbb";
pub const DIRECTION_OFFSETS: [u32; 2] = [0x399568, 0x39956c];
pub const DIRECTION_SOURCE_ROLE: &str = "indexed-exchange-directions";
pub const FIVE_BYTE_LIST_DIRECTION_OFFSET: u32 = 0xb946d4;
pub const FIVE_BYTE_LIST_DIRECTION_ROLE: &str = "five-byte-list-directions";
const FIVE_BYTE_LIST_DIRECTION_COUNT: usize = 6;
const FIVE_BYTE_LIST_DIRECTION_SHA256: &str =
    "16060a3a8062cfaf3d5cd0be3d18c2b77d7c94d10aad6d06454144d9a254ad5a";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectionOffsets {
    pub four_directions: [i8; 4],
    pub two_directions: [i8; 2],
}

impl DirectionOffsets {
    fn banks(&self) -> [(u32, &[i8]); 2] {
        [
            (DIRECTION_OFFSETS[0], &self.four_directions),
            (DIRECTION_OFFSETS[1], &self.two_directions),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FiveByteListDirection {
    pub x: i8,
    pub y: i8,
    /// Preserved physical bytes beyond the two signed coordinate reads.
    pub trailing_bytes: [u8; 2],
}

impl FiveByteListDirection {
    fn source_bytes(self) -> [u8; 4] {
        let [third, fourth] = self.trailing_bytes;
        [self.x as u8, self.y as u8, third, fourth]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdjacentOffsetTables {
    /// Four physical four-byte rows; the guest interprets their values.
    pub rows: [[u8; 4]; 4],
    /// Four independently rooted four-byte tables, including original padding.
    pub vectors: [[u8; 4]; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction_offsets: Option<DirectionOffsets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub five_byte_list_directions: Option<[FiveByteListDirection; FIVE_BYTE_LIST_DIRECTION_COUNT]>,
}
impl AdjacentOffsetTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.five_byte_list_directions.is_some() && self.direction_offsets.is_none() {
            return Err("Five-byte list direction data requires the preceding direction tables".into());
        }
        Ok(())
    }
    /// Returns physical source membership, not a bound on guest indices.
    pub fn byte_at(&self, at: u32) -> Option<(u8, usize)> {
        if let Some(relative) = at.checked_sub(SOURCE_OFFSET).map(|offset| offset as usize) {
            if relative < 16 {
                return Some((self.rows[relative / 4][relative % 4], 0));
            }
            if relative < SOURCE_LENGTH {
                let index = relative - 16;
                return Some((self.vectors[index / 4][index % 4], 1 + index / 4));
            }
        }
        if let (Some(relative), Some(directions)) = (
            at.checked_sub(FIVE_BYTE_LIST_DIRECTION_OFFSET),
            &self.five_byte_list_directions,
        ) {
            let relative = relative as usize;
            if let Some(row) = directions.get(relative / 4) {
                return Some((row.source_bytes()[relative % 4], TABLE_COUNT + DIRECTION_OFFSETS.len()));
            }
        }
        self.direction_offsets.as_ref()?.banks().into_iter().enumerate().find_map(
            |(bank, (start, bytes))| {
                let index = at.checked_sub(start)? as usize;
                Some((*bytes.get(index)? as u8, TABLE_COUNT + bank))
            },
        )
    }
    pub fn source_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        Ok(self.rows.iter().chain(self.vectors.iter()).flatten().copied().collect())
    }

    pub(super) fn source_spans(&self) -> Result<Vec<SourceRange>, String> {
        let bytes = self.source_bytes()?;
        let mut sources = vec![SourceRange {
            offset: SOURCE_OFFSET,
            length: bytes.len() as u32,
            sha256: ffta_rom::digest(&bytes),
            role: "adjacent-offset-tables".into(),
        }];
        if let Some(directions) = &self.direction_offsets {
            sources.extend(directions.banks().into_iter().map(|(offset, bank)| {
                let bytes: Vec<_> = bank.iter().map(|&value| value as u8).collect();
                SourceRange {
                    offset,
                    length: bytes.len() as u32,
                    sha256: ffta_rom::digest(&bytes),
                    role: DIRECTION_SOURCE_ROLE.into(),
                }
            }));
        }
        if let Some(directions) = &self.five_byte_list_directions {
            let bytes: Vec<_> = directions.iter().flat_map(|row| row.source_bytes()).collect();
            sources.push(SourceRange {
                offset: FIVE_BYTE_LIST_DIRECTION_OFFSET,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: FIVE_BYTE_LIST_DIRECTION_ROLE.into(),
            });
        }
        Ok(sources)
    }
}

pub struct Catalog {
    pub decoded: AdjacentOffsetTables,
    pub sources: Vec<SourceRange>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum SourceLayout {
    Legacy,
    IndexedExchange,
    FiveByteList,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        Self::from_rom_with_layout(rom, previous, SourceLayout::Legacy)
    }

    pub(super) fn from_rom_with_layout(
        rom: &[u8],
        previous: &[Resource],
        layout: SourceLayout,
    ) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Adjacent offset tables require the exact EUR ROM".into());
        }
        for (start, end, expected) in [
            (0x130360, 0x1303e8, "16dbb0880b15c6bd8e2e663f802cf5a6eddbf970f0127c8911d5e2b5ea6e5b20"),
            (0x1304d0, 0x130524, "db05e87e043f5d3acf0c438a8d223322d6b592c89cc1554ec18b1e7396af352e"),
            (0x130f20, 0x130f68, "10f0c2bfa45a543c6beb6a0603e6e637f278be5811c30599dddf6de50d072dcc"),
            (0x1311a8, 0x1311b8, "a66e18bf7f096c1445364af23edea6e2d9397de35bba85720a239517ffe78043"),
            (0x3ad480, 0x3ad488, "ea99f710d9d0b8ba192295c969a63ed7ce8fc5743da20d2057fa2b6d2c404bfb"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != expected {
                return Err("Adjacent offset reader or successor mismatch".into());
            }
        }
        for (at, expected) in [(0x1303dc, 0x083ad470), (0x1303e0, 0x083ad460),
            (0x1303e4, 0x083ad474), (0x130520, 0x083ad478), (0x130f64, 0x083ad47c),
            (0x1311b4, 0x083ad480)] {
            if u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) != expected {
                return Err("Adjacent offset reader literal mismatch".into());
            }
        }
        let start = SOURCE_OFFSET as usize; let bytes = &rom[start..start + SOURCE_LENGTH];
        if ffta_rom::digest(bytes) != SOURCE_SHA256 { return Err("Adjacent offset source mismatch".into()); }
        let direction_offsets = if layout >= SourceLayout::IndexedExchange {
            for (start, end, digest) in [
                (0x0b8320, 0x0b83a4, "fc1dc8739c225ab4a58538c44bef0f79c061e68a8c2538cf671cf7f771c7bcb5"),
                (0x0b83a4, 0x0b8428, "e9a36d996d141c4b0e4e1edce13a318c91f0f6d85fdc06e8d81ca4cb6dcabba6"),
            ] {
                if ffta_rom::digest(&rom[start..end]) != digest {
                    return Err("Indexed exchange direction reader fingerprint mismatch".into());
                }
            }
            let [four, two] = DIRECTION_OFFSETS.map(|offset| offset as usize);
            Some(DirectionOffsets {
                four_directions: std::array::from_fn(|index| rom[four + index] as i8),
                two_directions: std::array::from_fn(|index| rom[two + index] as i8),
            })
        } else {
            None
        };
        let five_byte_list_directions = if layout >= SourceLayout::FiveByteList {
            for (start, end, digest) in [
                (0x14c3f8, 0x14c420, "ef11e8d9c2d66c58ac4bfd462e2a991ee94d7500fe02b13ecab91bf043c4bcd2"),
                (0x14c79c, 0x14c7dc, "e15325900d15181b8f0b3ee11e7c55c879c3994396012fe059f0808935d5e106"),
            ] {
                if ffta_rom::digest(&rom[start..end]) != digest {
                    return Err("Five-byte list direction reader fingerprint mismatch".into());
                }
            }
            let start = FIVE_BYTE_LIST_DIRECTION_OFFSET as usize;
            let raw = &rom[start..start + 4 * FIVE_BYTE_LIST_DIRECTION_COUNT];
            if ffta_rom::digest(raw) != FIVE_BYTE_LIST_DIRECTION_SHA256 {
                return Err("Five-byte list direction data fingerprint mismatch".into());
            }
            Some(std::array::from_fn(|index| {
                let start = index * 4;
                FiveByteListDirection {
                    x: raw[start] as i8,
                    y: raw[start + 1] as i8,
                    trailing_bytes: [raw[start + 2], raw[start + 3]],
                }
            }))
        } else {
            None
        };
        let decoded = AdjacentOffsetTables {
            rows: std::array::from_fn(|i| bytes[i * 4..i * 4 + 4].try_into().unwrap()),
            vectors: std::array::from_fn(|i| bytes[16 + i * 4..20 + i * 4].try_into().unwrap()),
            direction_offsets,
            five_byte_list_directions,
        };
        let reconstructed = decoded.source_bytes()?;
        if reconstructed != bytes { return Err("Adjacent offset reconstruction mismatch".into()); }
        let sources = decoded.source_spans()?;
        let prior = PriorSources::new(previous)?;
        for span in &sources {
            if prior.contains(span.offset, span.length as usize)? {
                return Err("Adjacent offset source already has an owner".into());
            }
        }
        Ok(Self { decoded, sources })
    }
}
