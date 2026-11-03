//! Producer-bounded EUR M4A pitch/noise/rate/volume tables; guest arithmetic stays original.
use super::{audio_sequences, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};
pub const SOURCE_BYTES: usize = 484;
pub const TABLES: [(&str, u32, usize, usize); 7] = [
    ("pcm-keys", 0x370b20, 180, 1),
    ("pcm-bases", 0x370bd4, 12, 4),
    ("sample-counts", 0x370c04, 12, 2),
    ("cgb-keys", 0x370c1c, 132, 1),
    ("cgb-bases", 0x370ca0, 12, 2),
    ("noise-codes", 0x370cb8, 60, 1),
    ("wave-volume-codes", 0x370cf4, 16, 1),
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table<T> {
    pub source_offset: u32,
    pub entries: Vec<T>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyCode {
    pub base: u8,
    pub shift: u8,
}
impl KeyCode {
    pub fn byte(&self) -> Option<u8> {
        (self.base < 16 && self.shift < 16).then(|| self.base | (self.shift << 4))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioDriver {
    pub pcm_keys: Table<KeyCode>,
    pub pcm_bases: Table<u32>,
    pub sample_counts: Table<u16>,
    pub cgb_keys: Table<KeyCode>,
    pub cgb_bases: Table<i16>,
    pub noise_codes: Table<u8>,
    pub wave_volume_codes: Table<u8>,
}
fn high(a: u32, b: u32) -> u32 {
    ((u64::from(a) * u64::from(b)) >> 32) as u32
}
impl AudioDriver {
    pub fn table(&self, id: usize) -> Option<(u32, usize, usize)> {
        Some(match id {
            0 => (self.pcm_keys.source_offset, self.pcm_keys.entries.len(), 1),
            1 => (
                self.pcm_bases.source_offset,
                self.pcm_bases.entries.len(),
                4,
            ),
            2 => (
                self.sample_counts.source_offset,
                self.sample_counts.entries.len(),
                2,
            ),
            3 => (self.cgb_keys.source_offset, self.cgb_keys.entries.len(), 1),
            4 => (
                self.cgb_bases.source_offset,
                self.cgb_bases.entries.len(),
                2,
            ),
            5 => (
                self.noise_codes.source_offset,
                self.noise_codes.entries.len(),
                1,
            ),
            6 => (
                self.wave_volume_codes.source_offset,
                self.wave_volume_codes.entries.len(),
                1,
            ),
            _ => return None,
        })
    }
    pub fn byte(&self, table: usize, offset: usize) -> Option<u8> {
        Some(match table {
            0 => self.pcm_keys.entries.get(offset)?.byte()?,
            1 => self.pcm_bases.entries.get(offset / 4)?.to_le_bytes()[offset % 4],
            2 => self.sample_counts.entries.get(offset / 2)?.to_le_bytes()[offset % 2],
            3 => self.cgb_keys.entries.get(offset)?.byte()?,
            4 => self.cgb_bases.entries.get(offset / 2)?.to_le_bytes()[offset % 2],
            5 => *self.noise_codes.entries.get(offset)?,
            6 => *self.wave_volume_codes.entries.get(offset)?,
            _ => return None,
        })
    }
    pub fn source_byte(&self, address: u32) -> Option<u8> {
        for id in 0..7 {
            let (at, count, width) = self.table(id)?;
            if let Some(offset) = address
                .checked_sub(at)
                .filter(|n| (*n as usize) < count * width)
            {
                return self.byte(id, offset as usize);
            }
        }
        None
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        let mut spans = Vec::new();
        for (id, (name, _, _, _)) in TABLES.iter().enumerate() {
            let (at, count, width) = self.table(id).unwrap();
            let bytes = (0..count * width)
                .map(|n| {
                    self.byte(id, n)
                        .ok_or("Invalid M4A driver packed field".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            spans.push((at, bytes, format!("audio/driver-lut/{name}")));
        }
        Ok(spans)
    }
    pub fn validate(&self) -> Result<(), String> {
        for (id, (_, at, count, width)) in TABLES.iter().enumerate() {
            if self.table(id) != Some((*at, *count, *width)) {
                return Err("M4A driver table source/count/width mismatch".into());
            }
        }
        if self
            .pcm_keys
            .entries
            .iter()
            .chain(&self.cgb_keys.entries)
            .any(|k| k.base >= 12 || k.shift >= 16)
        {
            return Err("M4A driver key base/shift outside physical table".into());
        }
        Ok(())
    }
    pub fn validate_sequence_owner(
        &self,
        owner: &audio_sequences::AudioSequences,
    ) -> Result<(), String> {
        self.validate()?;
        owner.validate()?;
        if owner.driver.command_source_offset + owner.driver.commands.len() as u32 * 4
            != self.pcm_keys.source_offset
            || self.wave_volume_codes.source_offset + self.wave_volume_codes.entries.len() as u32
                != owner.driver.clock_source_offset
        {
            return Err("M4A driver table boundary differs from actual sequence owner".into());
        }
        Ok(())
    }
    /// Diagnostic only: exact unsigned multiply-high arithmetic, no inferred unit conversion.
    pub fn pcm_frequency_word(&self, wave_word: u32, key: u32, fraction: u32) -> Option<u32> {
        let mut key = key as u8 as usize;
        let mut fraction = fraction as u8;
        if key > 178 {
            key = 178;
            fraction = 255;
        }
        let code = *self.pcm_keys.entries.get(key)?;
        let next = *self.pcm_keys.entries.get(key + 1)?;
        let a = self
            .pcm_bases
            .entries
            .get(code.base as usize)?
            .wrapping_shr(u32::from(code.shift));
        let b = self
            .pcm_bases
            .entries
            .get(next.base as usize)?
            .wrapping_shr(u32::from(next.shift));
        Some(high(
            wave_word,
            a.wrapping_add(high(b.wrapping_sub(a), u32::from(fraction) << 24)),
        ))
    }
    /// Diagnostic of the original CGB helper, including type4 noise and signed tonal bases.
    pub fn cgb_frequency_word(&self, kind: u32, key: u32, fraction: u32) -> Option<u32> {
        let mut key = key as u8 as usize;
        let mut fraction = fraction as u8;
        if kind as u8 == 4 {
            let index = if key <= 20 { 0 } else { (key - 21).min(59) };
            return self.noise_codes.entries.get(index).map(|v| u32::from(*v));
        }
        if key <= 35 {
            key = 0;
            fraction = 0;
        } else {
            key -= 36;
            if key > 130 {
                key = 130;
                fraction = 255;
            }
        }
        let code = *self.cgb_keys.entries.get(key)?;
        let next = *self.cgb_keys.entries.get(key + 1)?;
        let a = i32::from(*self.cgb_bases.entries.get(code.base as usize)?)
            .wrapping_shr(u32::from(code.shift));
        let b = i32::from(*self.cgb_bases.entries.get(next.base as usize)?)
            .wrapping_shr(u32::from(next.shift));
        Some(
            a.wrapping_add(i32::from(fraction).wrapping_mul(b.wrapping_sub(a)) >> 8)
                .wrapping_add(0x800) as u32,
        )
    }
    /// Direct helper lookup, not the public SoundMode zero-nibble gate. Preserves adjacent aliases.
    pub fn sample_count_lookup(&self, mode: u32) -> Option<u16> {
        let selector = ((mode >> 16) & 15) as i64;
        let at = (i64::from(self.sample_counts.source_offset) + 2 * (selector - 1)) as u32;
        Some(u16::from_le_bytes([
            self.source_byte(at)?,
            self.source_byte(at + 1)?,
        ]))
    }
    /// Membership in the physical sixteen-entry map; never used to guard a guest RAM index.
    pub fn wave_volume_code(&self, level: u8) -> Option<u8> {
        self.wave_volume_codes.entries.get(level as usize).copied()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        let expected = Catalog::from_rom(rom)?;
        if self != &expected.decoded {
            return Err("Decoded M4A driver tables differ from authenticated source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: AudioDriver,
    pub sources: Vec<SourceRange>,
}
fn word(rom: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("M4A driver tables require exact European source ROM".into());
        }
        for (pc, at, want) in [
            (0x148a70, 0x148ab8, 0x08370b20),
            (0x148a76, 0x148abc, 0x08370bd4),
            (0x148fe0, 0x149050, 0x08370c04),
            (0x149566, 0x1495b0, 0x08370c1c),
            (0x14956c, 0x1495b4, 0x08370ca0),
            (0x14953a, 0x149544, 0x08370cb8),
            (0x149a32, 0x149a64, 0x08370cf4),
        ] {
            let op = u16::from_le_bytes(rom[pc..pc + 2].try_into().unwrap());
            if op & 0xf800 != 0x4800
                || ((pc + 4) & !3) + (op as usize & 255) * 4 != at
                || word(rom, at) != want
            {
                return Err("M4A driver table literal mismatch".into());
            }
        }
        for (at, want) in [
            (0x148a66, 0x2eb2),
            (0x148a88, 0x1c70),
            (0x149524, 0x2d14),
            (0x149534, 0x2d3b),
            (0x149548, 0x2d23),
            (0x14955c, 0x2d82),
            (0x14957a, 0x5fc1),
            (0x149592, 0x5e80),
            (0x148fd4, 0x20f0),
            (0x148fe2, 0x1e50),
            (0x148fe8, 0x8805),
            (0x149a34, 0x7a61),
            (0x14964c, 0x280f),
        ] {
            if u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) != want {
                return Err("M4A driver index/width contract mismatch".into());
            }
        }
        let keys = |at: usize, count: usize| Table {
            source_offset: at as u32,
            entries: rom[at..at + count]
                .iter()
                .map(|v| KeyCode {
                    base: v & 15,
                    shift: v >> 4,
                })
                .collect(),
        };
        let decoded = AudioDriver {
            pcm_keys: keys(0x370b20, 180),
            pcm_bases: Table {
                source_offset: 0x370bd4,
                entries: (0..12).map(|n| word(rom, 0x370bd4 + 4 * n)).collect(),
            },
            sample_counts: Table {
                source_offset: 0x370c04,
                entries: rom[0x370c04..0x370c1c]
                    .chunks_exact(2)
                    .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
                    .collect(),
            },
            cgb_keys: keys(0x370c1c, 132),
            cgb_bases: Table {
                source_offset: 0x370ca0,
                entries: rom[0x370ca0..0x370cb8]
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes(b.try_into().unwrap()))
                    .collect(),
            },
            noise_codes: Table {
                source_offset: 0x370cb8,
                entries: rom[0x370cb8..0x370cf4].to_vec(),
            },
            wave_volume_codes: Table {
                source_offset: 0x370cf4,
                entries: rom[0x370cf4..0x370d04].to_vec(),
            },
        };
        decoded.validate()?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom.get(offset as usize..offset as usize + bytes.len()) != Some(bytes.as_slice()) {
                return Err("M4A driver source roundtrip mismatch".into());
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
