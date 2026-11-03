//! Producer-bounded EUR tones, keymap bytes and lossless signed PCM/CGB samples.
//! Original sequencing, pitch/envelope work and mutable RAM are not replaced.
pub use super::audio_producers::Summary as ProducerSummary;
use super::{audio_directory, audio_producers, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const BASE: u32 = 0x08000000;
const ROM_BYTES: u32 = 0x01000000;
pub const FIRST_WAVE: u32 = 0x150944;
pub const NEW_SOURCE_BYTES: usize = 1_941_759;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub selector: u8,
    pub tone: u16,
    pub note_keys: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    /// Lowest song ID selecting this bank, independent of its source address.
    pub id: u16,
    pub source_address: u32,
    pub programs: Vec<Program>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tone {
    /// First occurrence in bank/program order, then in parent/key order.
    pub id: u16,
    pub source_offset: u32,
    pub ctype: u8,
    pub key: u8,
    pub length: u8,
    pub pan_or_sweep: u8,
    pub word04: u32,
    pub word08: u32,
}
impl Tone {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0 => self.ctype,
            1 => self.key,
            2 => self.length,
            3 => self.pan_or_sweep,
            4..=7 => self.word04.to_le_bytes()[offset - 4],
            8..=11 => self.word08.to_le_bytes()[offset - 8],
            _ => return None,
        })
    }
    pub fn source_bytes(&self) -> [u8; 12] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapByte {
    pub id: u16,
    pub source_offset: u32,
    pub selector: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapLink {
    pub parent_tone: u16,
    pub key: u8,
    pub byte: u16,
    pub tone: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PcmWave {
    /// First referring tone ID determines wave order and alias membership.
    pub id: u16,
    pub source_offset: u32,
    pub header_type: u16,
    pub flags: u16,
    pub frequency_word: u32,
    pub loop_start: u32,
    pub declared_length: u32,
    pub samples: Vec<i8>,
    pub interpolation_endpoint: Option<i8>,
    pub tone_ids: Vec<u16>,
}
impl PcmWave {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0..=1 => self.header_type.to_le_bytes()[offset],
            2..=3 => self.flags.to_le_bytes()[offset - 2],
            4..=7 => self.frequency_word.to_le_bytes()[offset - 4],
            8..=11 => self.loop_start.to_le_bytes()[offset - 8],
            12..=15 => self.declared_length.to_le_bytes()[offset - 12],
            _ => {
                let i = offset - 16;
                if i < self.samples.len() {
                    self.samples[i] as u8
                } else if i == self.samples.len() {
                    self.interpolation_endpoint? as u8
                } else {
                    return None;
                }
            }
        })
    }
    pub fn source_bytes(&self) -> Vec<u8> {
        (0..16 + self.samples.len() + usize::from(self.interpolation_endpoint.is_some()))
            .map(|i| self.byte(i).unwrap())
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CgbWave {
    pub id: u8,
    pub source_offset: u32,
    pub amplitudes: [u8; 32],
    pub tone_ids: Vec<u16>,
}
impl CgbWave {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(
            *self.amplitudes.get(offset.checked_mul(2)?)? << 4
                | *self.amplitudes.get(offset * 2 + 1)?,
        )
    }
    pub fn source_bytes(&self) -> [u8; 16] {
        std::array::from_fn(|i| self.byte(i).unwrap())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryAlias {
    pub wave: u16,
    pub directory_header: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioTones {
    pub banks: Vec<Bank>,
    pub tones: Vec<Tone>,
    pub map_bytes: Vec<MapByte>,
    pub map_links: Vec<MapLink>,
    pub pcm_waves: Vec<PcmWave>,
    pub cgb_waves: Vec<CgbWave>,
    pub directory_aliases: Vec<DirectoryAlias>,
}
fn sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|w| w[0] < w[1])
}
fn address(offset: u32) -> Result<u32, String> {
    if offset >= ROM_BYTES {
        return Err("M4A source offset outside EUR ROM".into());
    }
    Ok(BASE + offset)
}
impl AudioTones {
    pub fn tone(&self, id: u16) -> Option<&Tone> {
        self.tones.get(usize::from(id)).filter(|t| t.id == id)
    }
    pub fn validate(&self) -> Result<(), String> {
        let bad =
            || "Invalid EUR M4A tone/keymap/wave shape, ID, link or source extent".to_string();
        if (
            self.banks.len(),
            self.tones.len(),
            self.map_bytes.len(),
            self.map_links.len(),
            self.pcm_waves.len(),
            self.cgb_waves.len(),
            self.directory_aliases.len(),
        ) != (4, 429, 347, 348, 277, 2, 1)
        {
            return Err(bad());
        }
        let mut used = BTreeSet::new();
        let mut tone_addresses = BTreeSet::new();
        for (i, t) in self.tones.iter().enumerate() {
            if t.id as usize != i
                || t.source_offset & 3 != 0
                || t.source_offset > ROM_BYTES - 12
                || !tone_addresses.insert(t.source_offset)
                || !matches!(t.ctype, 0 | 1 | 2 | 3 | 8 | 0x40)
            {
                return Err(bad());
            }
        }
        let mut bank_ids = BTreeSet::new();
        let mut bank_roots = BTreeSet::new();
        let mut count = 0;
        for b in &self.banks {
            if !bank_ids.insert(b.id)
                || !bank_roots.insert(b.source_address)
                || b.id >= 640
                || !(BASE..BASE + ROM_BYTES).contains(&b.source_address)
                || b.source_address & 3 != 0
                || b.programs.is_empty()
                || !sorted(&b.programs.iter().map(|p| p.selector).collect::<Vec<_>>())
            {
                return Err(bad());
            }
            for p in &b.programs {
                let t = self.tone(p.tone).ok_or_else(bad)?;
                if b.source_address.checked_add(12 * u32::from(p.selector))
                    != Some(address(t.source_offset)?)
                    || !sorted(&p.note_keys)
                    || p.note_keys.iter().any(|k| *k >= 128)
                {
                    return Err(bad());
                }
                used.insert(t.id);
                count += 1;
            }
        }
        if count != 390 || !sorted(&self.banks.iter().map(|b| b.id).collect::<Vec<_>>()) {
            return Err(bad());
        }
        let mut map_addresses = BTreeSet::new();
        for (i, v) in self.map_bytes.iter().enumerate() {
            if v.id as usize != i
                || v.source_offset >= ROM_BYTES
                || !map_addresses.insert(v.source_offset)
            {
                return Err(bad());
            }
        }
        let mut link_keys = BTreeSet::new();
        let mut referenced_bytes = BTreeSet::new();
        for link in &self.map_links {
            let parent = self.tone(link.parent_tone).ok_or_else(bad)?;
            let child = self.tone(link.tone).ok_or_else(bad)?;
            let byte = self.map_bytes.get(link.byte as usize).ok_or_else(bad)?;
            if parent.ctype != 0x40
                || link.key >= 128
                || child.ctype & 0xc0 != 0
                || !link_keys.insert((link.parent_tone, link.key))
                || parent.word08.checked_add(u32::from(link.key))
                    != Some(address(byte.source_offset)?)
                || parent.word04.checked_add(12 * u32::from(byte.selector))
                    != Some(address(child.source_offset)?)
                || !self
                    .banks
                    .iter()
                    .flat_map(|b| &b.programs)
                    .any(|p| p.tone == parent.id && p.note_keys.contains(&link.key))
            {
                return Err(bad());
            }
            used.insert(child.id);
            referenced_bytes.insert(byte.id);
        }
        for b in &self.banks {
            for p in &b.programs {
                if self.tones[p.tone as usize].ctype == 0x40 {
                    for key in &p.note_keys {
                        if !link_keys.contains(&(p.tone, *key)) {
                            return Err(bad());
                        }
                    }
                }
            }
        }
        if used.len() != 429
            || referenced_bytes.len() != 347
            || !sorted(
                &self
                    .map_links
                    .iter()
                    .map(|l| (l.parent_tone, l.key))
                    .collect::<Vec<_>>(),
            )
        {
            return Err(bad());
        }
        let mut wave_addresses = BTreeSet::new();
        let mut wave_tones = BTreeSet::new();
        let mut previous = None;
        for (i, w) in self.pcm_waves.iter().enumerate() {
            let length = w.samples.len();
            let end = u64::from(w.source_offset)
                + 16
                + length as u64
                + u64::from(w.interpolation_endpoint.is_some());
            if w.id as usize != i
                || w.source_offset & 3 != 0
                || !wave_addresses.insert(w.source_offset)
                || w.header_type != 0
                || !matches!(w.flags, 0 | 0x4000)
                || length == 0
                || length != w.declared_length as usize
                || w.loop_start > w.declared_length
                || end > u64::from(ROM_BYTES)
                || w.tone_ids.is_empty()
                || !sorted(&w.tone_ids)
                || previous.is_some_and(|v| v >= w.tone_ids[0])
            {
                return Err(bad());
            }
            previous = Some(w.tone_ids[0]);
            let mut interpolated = false;
            for id in &w.tone_ids {
                let t = self.tone(*id).ok_or_else(bad)?;
                if !matches!(t.ctype, 0 | 8)
                    || t.word04 != address(w.source_offset)?
                    || !wave_tones.insert(t.id)
                {
                    return Err(bad());
                }
                interpolated |= t.ctype == 0;
            }
            if w.interpolation_endpoint.is_some() != interpolated {
                return Err(bad());
            }
        }
        let mut previous = None;
        for (i, w) in self.cgb_waves.iter().enumerate() {
            if w.id as usize != i
                || w.source_offset & 3 != 0
                || w.source_offset > ROM_BYTES - 16
                || !wave_addresses.insert(w.source_offset)
                || w.amplitudes.iter().any(|v| *v > 15)
                || w.tone_ids.is_empty()
                || !sorted(&w.tone_ids)
                || previous.is_some_and(|v| v >= w.tone_ids[0])
            {
                return Err(bad());
            }
            previous = Some(w.tone_ids[0]);
            for id in &w.tone_ids {
                let t = self.tone(*id).ok_or_else(bad)?;
                if t.ctype != 3 || t.word04 != address(w.source_offset)? || !wave_tones.insert(t.id)
                {
                    return Err(bad());
                }
            }
        }
        if self
            .tones
            .iter()
            .any(|t| matches!(t.ctype, 0 | 8 | 3) && !wave_tones.contains(&t.id))
        {
            return Err(bad());
        }
        let alias = &self.directory_aliases[0];
        let w = self.pcm_waves.get(alias.wave as usize).ok_or_else(bad)?;
        if w.source_offset != FIRST_WAVE || alias.directory_header >= 640 {
            return Err(bad());
        }
        let spans = self.source_spans();
        if spans.len() != 736
            || spans.iter().map(|(_, b, _)| b.len()).sum::<usize>() != NEW_SOURCE_BYTES
            || spans
                .windows(2)
                .any(|v| u64::from(v[0].0) + v[0].1.len() as u64 > u64::from(v[1].0))
        {
            return Err(bad());
        }
        Ok(())
    }
    pub fn validate_directory(&self, d: &audio_directory::AudioDirectory) -> Result<(), String> {
        self.validate()?;
        d.validate()?;
        let expected = bank_roots(d)?;
        if self
            .banks
            .iter()
            .map(|b| (b.id, b.source_address))
            .collect::<Vec<_>>()
            != expected
        {
            return Err("M4A tone bank directory dependency mismatch".into());
        }
        let a = &self.directory_aliases[0];
        let wave = &self.pcm_waves[a.wave as usize];
        let header = d
            .header(a.directory_header)
            .ok_or("Missing M4A wave directory alias owner")?;
        if header.track_count != 0
            || header.source_offset + 4 != wave.source_offset
            || header.source_bytes()[4..8] != wave.source_bytes()[..4]
        {
            return Err("M4A shared wave/directory bytes disagree".into());
        }
        Ok(())
    }
    /// Exact new source owners. The first wave's shared four bytes remain directory-owned.
    pub fn source_spans(&self) -> Vec<(u32, Vec<u8>, String)> {
        let mut spans = Vec::new();
        for t in &self.tones {
            spans.push((
                t.source_offset,
                t.source_bytes().to_vec(),
                format!("audio/tone/{}", t.id),
            ));
        }
        let mut maps: Vec<_> = self.map_bytes.iter().collect();
        maps.sort_by_key(|m| m.source_offset);
        let mut run: Option<(u32, Vec<u8>)> = None;
        let mut range = 0;
        for m in maps {
            if run
                .as_ref()
                .is_some_and(|(at, values)| *at + values.len() as u32 == m.source_offset)
            {
                run.as_mut().unwrap().1.push(m.selector);
            } else {
                if let Some((at, values)) = run.take() {
                    spans.push((at, values, format!("audio/keymap/{range}")));
                    range += 1;
                }
                run = Some((m.source_offset, vec![m.selector]));
            }
        }
        if let Some((at, values)) = run {
            spans.push((at, values, format!("audio/keymap/{range}")));
        }
        for w in &self.pcm_waves {
            let skip = if self.directory_aliases.iter().any(|a| a.wave == w.id) {
                4
            } else {
                0
            };
            let bytes = w.source_bytes();
            spans.push((
                w.source_offset + skip,
                bytes[skip as usize..].to_vec(),
                format!("audio/pcm-wave/{}", w.id),
            ));
        }
        for w in &self.cgb_waves {
            spans.push((
                w.source_offset,
                w.source_bytes().to_vec(),
                format!("audio/cgb-wave/{}", w.id),
            ));
        }
        spans.sort_by_key(|v| v.0);
        spans
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("M4A tones differ from authenticated EUR source".into());
        }
        Ok(())
    }
}
fn bank_roots(d: &audio_directory::AudioDirectory) -> Result<Vec<(u16, u32)>, String> {
    let mut roots = BTreeMap::new();
    for s in &d.songs {
        let h = d.header(s.header_id).ok_or("Missing M4A source header")?;
        if h.track_count != 0 {
            roots.entry(h.tone_bank_address).or_insert(s.id);
        }
    }
    let mut values: Vec<_> = roots.into_iter().map(|(root, id)| (id, root)).collect();
    values.sort();
    Ok(values)
}
fn intern(
    rom: &[u8],
    tones: &mut Vec<Tone>,
    ids: &mut BTreeMap<u32, u16>,
    pointer: u32,
) -> Result<u16, String> {
    let at = pointer.checked_sub(BASE).ok_or("Non-ROM M4A tone")?;
    if let Some(id) = ids.get(&at) {
        return Ok(*id);
    }
    let raw = rom
        .get(at as usize..at as usize + 12)
        .ok_or("Truncated M4A tone")?;
    let id = u16::try_from(tones.len()).map_err(|_| "M4A tone ID overflow")?;
    tones.push(Tone {
        id,
        source_offset: at,
        ctype: raw[0],
        key: raw[1],
        length: raw[2],
        pan_or_sweep: raw[3],
        word04: u32::from_le_bytes(raw[4..8].try_into().unwrap()),
        word08: u32::from_le_bytes(raw[8..12].try_into().unwrap()),
    });
    ids.insert(at, id);
    Ok(id)
}
pub struct Catalog {
    pub decoded: AudioTones,
    pub sources: Vec<SourceRange>,
    pub producer: ProducerSummary,
    pub consumed_bytecode: Vec<(u32, u32)>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        let directory = audio_directory::Catalog::from_rom(rom)?.decoded;
        let graph = audio_producers::analyze(rom, &directory)?;
        let mut tones = Vec::new();
        let mut ids = BTreeMap::new();
        let mut banks = Vec::new();
        for (id, source_address) in bank_roots(&directory)? {
            let mut programs = Vec::new();
            for (&(root, selector), keys) in &graph.programs {
                if root == source_address {
                    let pointer = root
                        .checked_add(12 * u32::from(selector))
                        .ok_or("M4A selector overflow")?;
                    let tone = intern(rom, &mut tones, &mut ids, pointer)?;
                    programs.push(Program {
                        selector,
                        tone,
                        note_keys: keys.iter().copied().collect(),
                    });
                }
            }
            banks.push(Bank {
                id,
                source_address,
                programs,
            });
        }
        let main = tones.clone();
        let mut map_bytes = Vec::new();
        let mut map_ids = BTreeMap::new();
        let mut map_links = Vec::new();
        for parent in main.into_iter().filter(|t| t.ctype == 0x40) {
            let keys: BTreeSet<_> = banks
                .iter()
                .flat_map(|b| &b.programs)
                .filter(|p| p.tone == parent.id)
                .flat_map(|p| p.note_keys.iter().copied())
                .collect();
            for key in keys {
                let pointer = parent
                    .word08
                    .checked_add(u32::from(key))
                    .ok_or("M4A keymap overflow")?;
                let at = pointer.checked_sub(BASE).ok_or("Non-ROM M4A keymap")?;
                let selector = *rom.get(at as usize).ok_or("Truncated M4A keymap")?;
                let byte = *map_ids.entry(at).or_insert_with(|| {
                    let id = map_bytes.len() as u16;
                    map_bytes.push(MapByte {
                        id,
                        source_offset: at,
                        selector,
                    });
                    id
                });
                let pointer = parent
                    .word04
                    .checked_add(12 * u32::from(selector))
                    .ok_or("M4A mapped selector overflow")?;
                let tone = intern(rom, &mut tones, &mut ids, pointer)?;
                map_links.push(MapLink {
                    parent_tone: parent.id,
                    key,
                    byte,
                    tone,
                });
            }
        }
        let mut pcm_groups: BTreeMap<u32, Vec<u16>> = BTreeMap::new();
        let mut cgb_groups: BTreeMap<u32, Vec<u16>> = BTreeMap::new();
        for t in &tones {
            match t.ctype {
                0 | 8 => {
                    pcm_groups.entry(t.word04).or_default().push(t.id);
                }
                3 => {
                    cgb_groups.entry(t.word04).or_default().push(t.id);
                }
                _ => {}
            }
        }
        let mut pcm: Vec<_> = pcm_groups.into_iter().collect();
        pcm.sort_by_key(|(_, ids)| ids[0]);
        let mut pcm_waves = Vec::new();
        let mut directory_aliases = Vec::new();
        for (pointer, tone_ids) in pcm {
            let at = pointer.checked_sub(BASE).ok_or("Non-ROM M4A PCM wave")? as usize;
            let h = rom.get(at..at + 16).ok_or("Truncated M4A wave header")?;
            let word = |i| u32::from_le_bytes(h[i..i + 4].try_into().unwrap());
            let size = word(12) as usize;
            let samples = rom
                .get(at + 16..at + 16 + size)
                .ok_or("Truncated M4A PCM samples")?
                .iter()
                .map(|v| *v as i8)
                .collect();
            let endpoint = if tone_ids.iter().any(|id| tones[*id as usize].ctype == 0) {
                Some(
                    *rom.get(at + 16 + size)
                        .ok_or("Missing M4A interpolation endpoint")? as i8,
                )
            } else {
                None
            };
            let id = pcm_waves.len() as u16;
            if at as u32 == FIRST_WAVE {
                let header = directory
                    .headers
                    .iter()
                    .find(|h| h.source_offset + 4 == FIRST_WAVE)
                    .ok_or("Missing directory wave alias")?;
                directory_aliases.push(DirectoryAlias {
                    wave: id,
                    directory_header: header.id,
                });
            }
            pcm_waves.push(PcmWave {
                id,
                source_offset: at as u32,
                header_type: u16::from_le_bytes(h[..2].try_into().unwrap()),
                flags: u16::from_le_bytes(h[2..4].try_into().unwrap()),
                frequency_word: word(4),
                loop_start: word(8),
                declared_length: word(12),
                samples,
                interpolation_endpoint: endpoint,
                tone_ids,
            });
        }
        let mut cgb: Vec<_> = cgb_groups.into_iter().collect();
        cgb.sort_by_key(|(_, ids)| ids[0]);
        let mut cgb_waves = Vec::new();
        for (pointer, tone_ids) in cgb {
            let at = pointer.checked_sub(BASE).ok_or("Non-ROM M4A CGB wave")?;
            let bytes = rom
                .get(at as usize..at as usize + 16)
                .ok_or("Truncated CGB wave")?;
            cgb_waves.push(CgbWave {
                id: cgb_waves.len() as u8,
                source_offset: at,
                amplitudes: std::array::from_fn(|i| {
                    if i % 2 == 0 {
                        bytes[i / 2] >> 4
                    } else {
                        bytes[i / 2] & 15
                    }
                }),
                tone_ids,
            });
        }
        let decoded = AudioTones {
            banks,
            tones,
            map_bytes,
            map_links,
            pcm_waves,
            cgb_waves,
            directory_aliases,
        };
        decoded.validate_directory(&directory)?;
        let sources = decoded
            .source_spans()
            .into_iter()
            .map(|(offset, bytes, role)| SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            })
            .collect();
        Ok(Self {
            decoded,
            sources,
            producer: graph.summary,
            consumed_bytecode: graph.consumed,
        })
    }
}
