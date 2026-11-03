//! EUR M4A configuration, directory and headers. Sequencing and mutable player state stay guest-owned.
use super::SourceRange;
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PLAYER_OFFSET: u32 = 0x14f51c;
pub const DIRECTORY_OFFSET: u32 = 0x14f540;
pub const DIRECTORY_END: u32 = 0x150940;
pub const PLAYER_COUNT: usize = 3;
pub const SONG_COUNT: usize = 640;
pub const HEADER_COUNT: usize = 455;
const ROM_BASE: u32 = 0x08000000;
const ROM_END: u32 = 0x09000000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub id: u8,
    pub source_offset: u32,
    pub player_address: u32,
    pub tracks_address: u32,
    pub track_capacity: u8,
    pub opaque09: u8,
    pub priority_gate: u16,
}
impl Player {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0..=3 => self.player_address.to_le_bytes()[offset],
            4..=7 => self.tracks_address.to_le_bytes()[offset - 4],
            8 => self.track_capacity,
            9 => self.opaque09,
            10..=11 => self.priority_gate.to_le_bytes()[offset - 10],
            _ => return None,
        })
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        let mut b = self.player_address.to_le_bytes().to_vec();
        b.extend(self.tracks_address.to_le_bytes());
        b.extend([self.track_capacity, self.opaque09]);
        b.extend(self.priority_gate.to_le_bytes());
        b
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Song {
    pub id: u16,
    pub source_offset: u32,
    /// Stable header ID: the lowest directory ID referring to this header.
    pub header_id: u16,
    pub header_address: u32,
    pub player_index: u16,
    pub opaque06: u16,
}
impl Song {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0..=3 => self.header_address.to_le_bytes()[offset],
            4..=5 => self.player_index.to_le_bytes()[offset - 4],
            6..=7 => self.opaque06.to_le_bytes()[offset - 6],
            _ => return None,
        })
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        let mut b = self.header_address.to_le_bytes().to_vec();
        b.extend(self.player_index.to_le_bytes());
        b.extend(self.opaque06.to_le_bytes());
        b
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub id: u16,
    pub source_offset: u32,
    pub track_count: u8,
    pub opaque01: u8,
    pub priority: u8,
    pub reverb: u8,
    /// Retained verbatim even for the zero-track header's overlapping 0x40000000 word.
    pub tone_bank_address: u32,
    pub track_addresses: Vec<u32>,
}
impl Header {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0 => self.track_count,
            1 => self.opaque01,
            2 => self.priority,
            3 => self.reverb,
            4..=7 => self.tone_bank_address.to_le_bytes()[offset - 4],
            _ => self.track_addresses.get((offset - 8) / 4)?.to_le_bytes()[(offset - 8) % 4],
        })
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        let mut b = vec![self.track_count, self.opaque01, self.priority, self.reverb];
        b.extend(self.tone_bank_address.to_le_bytes());
        for p in &self.track_addresses {
            b.extend(p.to_le_bytes());
        }
        b
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioDirectory {
    pub player_source_offset: u32,
    pub directory_source_offset: u32,
    pub players: Vec<Player>,
    pub songs: Vec<Song>,
    /// Ordered by original source offset. Shared headers have one owner.
    pub headers: Vec<Header>,
}
impl AudioDirectory {
    pub fn validate(&self) -> Result<(), String> {
        let bad = || "Invalid fixed EUR M4A directory shape, link or extent".to_string();
        if self.player_source_offset != PLAYER_OFFSET
            || self.directory_source_offset != DIRECTORY_OFFSET
            || self.players.len() != PLAYER_COUNT
            || self.songs.len() != SONG_COUNT
            || self.headers.len() != HEADER_COUNT
        {
            return Err(bad());
        }
        for (i, p) in self.players.iter().enumerate() {
            if p.id as usize != i
                || p.source_offset != PLAYER_OFFSET + i as u32 * 12
                || p.player_address & 3 != 0
                || !(0x02000000..=0x0203ffc0).contains(&p.player_address)
                || p.tracks_address & 3 != 0
                || !(1..=16).contains(&p.track_capacity)
                || !(0x03000000..0x03008000).contains(&p.tracks_address)
                || p.tracks_address + 80 * u32::from(p.track_capacity) > 0x03008000
            {
                return Err(bad());
            }
        }
        let mut links = BTreeMap::new();
        for (i, s) in self.songs.iter().enumerate() {
            if s.id as usize != i
                || s.source_offset != DIRECTORY_OFFSET + i as u32 * 8
                || s.player_index as usize >= PLAYER_COUNT
                || !(ROM_BASE..ROM_END).contains(&s.header_address)
            {
                return Err(bad());
            }
            links.entry(s.header_address).or_insert(s.id);
        }
        let mut ids = BTreeSet::new();
        let mut end = DIRECTORY_END;
        for h in &self.headers {
            if h.source_offset < end
                || h.source_offset & 3 != 0
                || h.source_offset >= ROM_END - ROM_BASE
                || h.track_count as usize != h.track_addresses.len()
                || h.track_count > 16
                || !ids.insert(h.id)
                || links.get(&(ROM_BASE + h.source_offset)) != Some(&h.id)
                || h.track_addresses
                    .iter()
                    .any(|p| !(ROM_BASE..ROM_END).contains(p))
                || (h.track_count != 0 && !(ROM_BASE..ROM_END).contains(&h.tone_bank_address))
            {
                return Err(bad());
            }
            end = h.source_offset + 8 + 4 * u32::from(h.track_count);
            if end > ROM_END - ROM_BASE {
                return Err(bad());
            }
        }
        if links.len() != self.headers.len() {
            return Err(bad());
        }
        for s in &self.songs {
            let h = self.header(s.header_id).ok_or_else(bad)?;
            if s.header_address != ROM_BASE + h.source_offset
                || h.track_count > self.players[s.player_index as usize].track_capacity
            {
                return Err(bad());
            }
        }
        Ok(())
    }
    pub fn header(&self, id: u16) -> Option<&Header> {
        self.headers.iter().find(|h| h.id == id)
    }
    /// Original directory lookup uses low16. None means outside this catalog,
    /// not an invalid guest input or a replacement for raw guest behavior.
    pub fn selected(&self, argument: u32) -> Option<(&Song, &Player, &Header)> {
        let s = self.songs.get(usize::from(argument as u16))?;
        Some((
            s,
            self.players.get(usize::from(s.player_index))?,
            self.header(s.header_id)?,
        ))
    }
    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>)> {
        let mut spans = vec![
            (
                PLAYER_OFFSET,
                self.players.iter().flat_map(Player::source_bytes).collect(),
            ),
            (
                DIRECTORY_OFFSET,
                self.songs.iter().flat_map(Song::source_bytes).collect(),
            ),
        ];
        spans.extend(
            self.headers
                .iter()
                .map(|h| (h.source_offset, h.source_bytes())),
        );
        spans
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("M4A directory differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: AudioDirectory,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("M4A directory requires exact EUR ROM; USA pending".into());
        }
        let half = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let word = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for (pc, at, target) in [
            (0x148b24, 0x148b6c, 3),
            (0x148b2e, 0x148b70, ROM_BASE + PLAYER_OFFSET),
            (0x148b88, 0x148ba8, ROM_BASE + PLAYER_OFFSET),
            (0x148b8a, 0x148bac, ROM_BASE + DIRECTORY_OFFSET),
            (0x148bb4, 0x148bdc, ROM_BASE + PLAYER_OFFSET),
            (0x148bb6, 0x148be0, ROM_BASE + DIRECTORY_OFFSET),
            (0x148c00, 0x148c28, ROM_BASE + PLAYER_OFFSET),
            (0x148c02, 0x148c2c, ROM_BASE + DIRECTORY_OFFSET),
            (0x148c54, 0x148c7c, ROM_BASE + PLAYER_OFFSET),
            (0x148c56, 0x148c80, ROM_BASE + DIRECTORY_OFFSET),
            (0x148c88, 0x148cb0, ROM_BASE + PLAYER_OFFSET),
            (0x148c8a, 0x148cb4, ROM_BASE + DIRECTORY_OFFSET),
            (0x144f18, 0x144f30, ROM_BASE + DIRECTORY_OFFSET),
            (0x144f62, 0x144f80, ROM_BASE + DIRECTORY_OFFSET),
        ] {
            let op = half(pc);
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + usize::from(op & 255) * 4 != at
                || word(at) != target
            {
                return Err("M4A original literal mismatch".into());
            }
        }
        for (pc, op) in [
            (0x148b32, 0x682c),
            (0x148b34, 0x6869),
            (0x148b36, 0x7a2a),
            (0x148b3e, 0x8968),
            (0x148b46, 0x350c),
            (0x148b86, 0x0400),
            (0x148b8c, 0x0b40),
            (0x148b90, 0x8883),
            (0x148b92, 0x0059),
            (0x148b94, 0x18c9),
            (0x148b96, 0x0089),
            (0x148b9a, 0x680a),
            (0x148b9c, 0x6801),
            (0x1492be, 0x6878),
            (0x1492d8, 0x7839),
            (0x1492de, 0x7a28),
            (0x1492fa, 0x3008),
            (0x1492fe, 0x6800),
        ] {
            if half(pc) != op {
                return Err("M4A original stride/index/header mismatch".into());
            }
        }
        // Physical end is independently rooted by the empty header and first tone's wave link.
        if word(0x14da18) != ROM_BASE + DIRECTORY_END + 4
            || word(DIRECTORY_END as usize) != 0
            || word(DIRECTORY_END as usize + 4) != 0x40000000
        {
            return Err("M4A directory end/header/wave alias mismatch".into());
        }
        let players = (0..PLAYER_COUNT)
            .map(|i| {
                let at = PLAYER_OFFSET as usize + i * 12;
                Player {
                    id: i as u8,
                    source_offset: at as u32,
                    player_address: word(at),
                    tracks_address: word(at + 4),
                    track_capacity: rom[at + 8],
                    opaque09: rom[at + 9],
                    priority_gate: half(at + 10),
                }
            })
            .collect();
        let mut links = BTreeMap::new();
        for i in 0..SONG_COUNT {
            links
                .entry(word(DIRECTORY_OFFSET as usize + i * 8))
                .or_insert(i as u16);
        }
        let songs = (0..SONG_COUNT)
            .map(|i| {
                let at = DIRECTORY_OFFSET as usize + i * 8;
                Song {
                    id: i as u16,
                    source_offset: at as u32,
                    header_id: links[&word(at)],
                    header_address: word(at),
                    player_index: half(at + 4),
                    opaque06: half(at + 6),
                }
            })
            .collect();
        let mut headers = Vec::new();
        for (address, id) in links {
            let at = address
                .checked_sub(ROM_BASE)
                .ok_or("Invalid M4A header address")? as usize;
            let prefix = rom.get(at..at + 8).ok_or("Truncated M4A header")?;
            let count = prefix[0];
            let raw = rom
                .get(at + 8..at + 8 + 4 * usize::from(count))
                .ok_or("Truncated M4A tracks")?;
            headers.push(Header {
                id,
                source_offset: at as u32,
                track_count: count,
                opaque01: prefix[1],
                priority: prefix[2],
                reverb: prefix[3],
                tone_bank_address: word(at + 4),
                track_addresses: raw
                    .chunks_exact(4)
                    .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                    .collect(),
            });
        }
        let decoded = AudioDirectory {
            player_source_offset: PLAYER_OFFSET,
            directory_source_offset: DIRECTORY_OFFSET,
            players,
            songs,
            headers,
        };
        decoded.validate()?;
        let sources = decoded
            .source_spans()
            .into_iter()
            .enumerate()
            .map(|(i, (offset, bytes))| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: if i == 0 {
                    "audio/m4a-players".into()
                } else if i == 1 {
                    "audio/m4a-directory".into()
                } else {
                    format!("audio/m4a-header/{}", decoded.headers[i - 2].id)
                },
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}
