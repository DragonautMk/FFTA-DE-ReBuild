//! Bounded EUR event records, effect selectors and same-state dispatch tables.
//! Original guest sequencing, live indices and default branches remain authoritative.
use super::{task_descriptors, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const EVENT_OFFSET: u32 = 0x3aca0c;
pub const SELECTOR_OFFSET: u32 = 0x3ad174;
pub const SOURCE_BYTES: usize = 1516;
pub fn event_source_offset(index: u16) -> u32 {
    EVENT_OFFSET + u32::from(index) * 16
}
pub fn selector_source_offset(index: u32) -> u32 {
    SELECTOR_OFFSET + u32::from(index as u16) * 4
}
pub fn secondary_task_id(index: u32) -> u16 {
    (index as u16).wrapping_add(363)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub index: u16,
    pub opcode: u8,
    pub byte1_raw: u8,
    pub id_raw: u16,
    pub post_delay: u8,
    pub byte5_raw: u8,
    pub half6_raw: u16,
    pub arg8_raw: u32,
    pub arg12_raw: u32,
    /// Canonical script-catalog ordinal for the base opcode0 secondary task.
    pub script_descriptor: Option<u16>,
}
impl Event {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        Some(match offset {
            0 => self.opcode,
            1 => self.byte1_raw,
            2..=3 => self.id_raw.to_le_bytes()[offset - 2],
            4 => self.post_delay,
            5 => self.byte5_raw,
            6..=7 => self.half6_raw.to_le_bytes()[offset - 6],
            8..=11 => self.arg8_raw.to_le_bytes()[offset - 8],
            12..=15 => self.arg12_raw.to_le_bytes()[offset - 12],
            _ => return None,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    pub index: u16,
    pub kind: u8,
    pub parameters: [u8; 3],
    pub script_descriptor: u16,
}
impl Selector {
    pub fn byte(&self, offset: usize) -> Option<u8> {
        if offset == 0 {
            Some(self.kind)
        } else {
            self.parameters.get(offset.checked_sub(1)?).copied()
        }
    }
}
/// MOV PC preserves Thumb state. These source words are even, unlike BX callbacks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SameStateThumb {
    pub rom_offset: u32,
}
impl SameStateThumb {
    pub fn guest_word(&self) -> Option<u32> {
        (self.rom_offset < 0x1000000 && self.rom_offset & 1 == 0)
            .then(|| 0x08000000 + self.rom_offset)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchSlot {
    pub selector: u8,
    pub target: SameStateThumb,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dispatch {
    pub source_offset: u32,
    pub slots: Vec<DispatchSlot>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectEventDirectories {
    pub event_source_offset: u32,
    pub events: Vec<Event>,
    pub selector_source_offset: u32,
    pub selectors: Vec<Selector>,
    pub event_dispatch: Dispatch,
    pub selector_dispatch: Dispatch,
}
impl EffectEventDirectories {
    pub fn validate(&self) -> Result<(), String> {
        if self.event_source_offset != EVENT_OFFSET
            || self.events.len() != 75
            || self.selector_source_offset != SELECTOR_OFFSET
            || self.selectors.len() != 51
        {
            return Err("Effect event source/count mismatch".into());
        }
        for (i, e) in self.events.iter().enumerate() {
            let reference = if e.opcode == 0 {
                e.id_raw.checked_sub(363).filter(|n| *n < 51)
            } else {
                None
            };
            if usize::from(e.index) != i
                || e.script_descriptor != reference
                || e.opcode == 0 && reference.is_none()
            {
                return Err("Effect event index/descriptor reference mismatch".into());
            }
        }
        for (i, s) in self.selectors.iter().enumerate() {
            if usize::from(s.index) != i || usize::from(s.script_descriptor) != i {
                return Err("Effect selector index/descriptor reference mismatch".into());
            }
        }
        for (d, offset, count) in [
            (&self.event_dispatch, 0x124efc, 22),
            (&self.selector_dispatch, 0x128b50, 6),
        ] {
            if d.source_offset != offset || d.slots.len() != count {
                return Err("Effect dispatch source/count mismatch".into());
            }
            for (i, s) in d.slots.iter().enumerate() {
                if usize::from(s.selector) != i || s.target.guest_word().is_none() {
                    return Err("Effect dispatch selector/same-state target mismatch".into());
                }
            }
        }
        Ok(())
    }
    /// Physical source membership only, never a cap on original guest RAM indices.
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        if let Some(n) = at.checked_sub(self.event_source_offset) {
            if let Some(e) = self.events.get(n as usize / 16) {
                return e.byte(n as usize % 16);
            }
        }
        if let Some(n) = at.checked_sub(self.selector_source_offset) {
            if let Some(s) = self.selectors.get(n as usize / 4) {
                return s.byte(n as usize % 4);
            }
        }
        for d in [&self.event_dispatch, &self.selector_dispatch] {
            if let Some(n) = at.checked_sub(d.source_offset) {
                if let Some(s) = d.slots.get(n as usize / 4) {
                    return Some(s.target.guest_word()?.to_le_bytes()[n as usize % 4]);
                }
            }
        }
        None
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        [
            (0x124efc, 88, "event-dispatch-0-21"),
            (0x128b50, 24, "selector-dispatch-0-5"),
            (EVENT_OFFSET, 1200, "events-0-74"),
            (SELECTOR_OFFSET, 204, "selectors-0-50"),
        ]
        .into_iter()
        .map(|(at, n, role)| {
            let bytes = (0..n)
                .map(|i| {
                    self.byte_at(at + i)
                        .ok_or_else(|| "Invalid effect directory byte".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((at, bytes, format!("engine/effect-event-directories/{role}")))
        })
        .collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Effect event directories differ from authenticated source".into());
        }
        Ok(())
    }
}
pub struct Catalog {
    pub decoded: EffectEventDirectories,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Effect events require exact European source ROM".into());
        }
        for (at, v) in CONTRACT_HALFWORDS {
            if u16::from_le_bytes(rom[*at as usize..*at as usize + 2].try_into().unwrap()) != *v {
                return Err("Effect event original field/index/dispatch contract mismatch".into());
            }
        }
        for (at, v) in CONTRACT_WORDS {
            if u32::from_le_bytes(rom[*at as usize..*at as usize + 4].try_into().unwrap()) != *v {
                return Err("Effect event original root/boundary contract mismatch".into());
            }
        }
        let tasks = task_descriptors::Catalog::from_rom(rom)?.decoded;
        let h = |at: usize| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let w = |at: usize| u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        let mut events = Vec::new();
        for index in 0..75 {
            let at = EVENT_OFFSET as usize + 16 * index;
            let opcode = rom[at];
            let id_raw = h(at + 2);
            let script_descriptor = if opcode == 0 {
                Some(
                    id_raw
                        .checked_sub(363)
                        .filter(|i| tasks.script.entries.get(usize::from(*i)).is_some())
                        .ok_or(
                            "Effect event secondary task is outside authenticated script catalog",
                        )?,
                )
            } else {
                None
            };
            events.push(Event {
                index: index as u16,
                opcode,
                byte1_raw: rom[at + 1],
                id_raw,
                post_delay: rom[at + 4],
                byte5_raw: rom[at + 5],
                half6_raw: h(at + 6),
                arg8_raw: w(at + 8),
                arg12_raw: w(at + 12),
                script_descriptor,
            });
        }
        let selectors = (0..51)
            .map(|i| {
                let at = SELECTOR_OFFSET as usize + 4 * i;
                Selector {
                    index: i as u16,
                    kind: rom[at],
                    parameters: rom[at + 1..at + 4].try_into().unwrap(),
                    script_descriptor: i as u16,
                }
            })
            .collect();
        let dispatch = |at: u32, n: usize| -> Result<Dispatch, String> {
            let mut slots = Vec::new();
            for i in 0..n {
                let word = w(at as usize + 4 * i);
                if word & 1 != 0 || !(0x08000000..0x09000000).contains(&word) {
                    return Err("Effect dispatch is not an even same-state ROM target".into());
                }
                slots.push(DispatchSlot {
                    selector: i as u8,
                    target: SameStateThumb {
                        rom_offset: word - 0x08000000,
                    },
                });
            }
            Ok(Dispatch {
                source_offset: at,
                slots,
            })
        };
        let decoded = EffectEventDirectories {
            event_source_offset: EVENT_OFFSET,
            events,
            selector_source_offset: SELECTOR_OFFSET,
            selectors,
            event_dispatch: dispatch(0x124efc, 22)?,
            selector_dispatch: dispatch(0x128b50, 6)?,
        };
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if rom[offset as usize..offset as usize + bytes.len()] != bytes {
                return Err("Effect event source roundtrip mismatch".into());
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

const CONTRACT_HALFWORDS: &[(u32, u16)] = &[
    (0x124eb8, 0x4a0e),
    (0x124eba, 0x8838),
    (0x124ebc, 0x100),
    (0x124ebe, 0x1880),
    (0x124ec0, 0x8840),
    (0x124ec2, 0x8138),
    (0x124ec4, 0x883b),
    (0x124ec6, 0x119),
    (0x124ec8, 0x1888),
    (0x124eca, 0x7800),
    (0x124ecc, 0x4684),
    (0x124ed0, 0x3008),
    (0x124ed2, 0x1808),
    (0x124ed4, 0x6805),
    (0x124ed8, 0x300c),
    (0x124eda, 0x1809),
    (0x124edc, 0x680e),
    (0x124ee0, 0x4660),
    (0x124ee2, 0x2815),
    (0x124ee4, 0xd900),
    (0x124ee6, 0xe257),
    (0x124ee8, 0x80),
    (0x124eea, 0x4903),
    (0x124eec, 0x1840),
    (0x124eee, 0x6800),
    (0x124ef0, 0x4687),
    (0x1253ce, 0x4908),
    (0x1253d0, 0x8838),
    (0x1253d2, 0x100),
    (0x1253d4, 0x1840),
    (0x1253d6, 0x7900),
    (0x1253d8, 0x8078),
    (0x1253da, 0x8838),
    (0x1253dc, 0x3001),
    (0x1253de, 0x8038),
    (0x1253e0, 0x400),
    (0x1253e2, 0xc00),
    (0x1253e4, 0x284a),
    (0x1253e6, 0xd805),
    (0x125036, 0x8838),
    (0x125038, 0x1980),
    (0x12503a, 0x8038),
    (0x126628, 0x4b0c),
    (0x12662a, 0x7822),
    (0x12662c, 0x51),
    (0x12662e, 0x1889),
    (0x126630, 0x49),
    (0x126632, 0x18c9),
    (0x126634, 0x780a),
    (0x126636, 0x7849),
    (0x12746e, 0x7848),
    (0x127470, 0x3801),
    (0x127472, 0x400),
    (0x127474, 0xc00),
    (0x127476, 0xf001),
    (0x127478, 0xfb3d),
    (0x128b00, 0x400),
    (0x128b02, 0xc00),
    (0x128b08, 0x81),
    (0x128b0a, 0x480d),
    (0x128b0c, 0x180d),
    (0x128b20, 0x9900),
    (0x128b22, 0x4a09),
    (0x128b24, 0x1888),
    (0x128b26, 0x400),
    (0x128b28, 0xc00),
    (0x128b2a, 0x9000),
    (0x128b2c, 0x7828),
    (0x128b2e, 0x2805),
    (0x128b30, 0xd900),
    (0x128b32, 0xe1d2),
    (0x128b34, 0x80),
    (0x128b36, 0x4905),
    (0x128b38, 0x1840),
    (0x128b3a, 0x6800),
    (0x128b3c, 0x4687),
    (0x128b6e, 0x7868),
    (0x128bc8, 0x78a8),
    (0x128c2e, 0x7869),
    (0x128d20, 0x7869),
    (0x128d6a, 0x78a9),
    (0x128dc6, 0x7869),
    (0x128e70, 0x78e9),
    (0x128ed4, 0xf7ae),
    (0x128ed6, 0xfa28),
    (0x12c7fe, 0x4907),
    (0x12c800, 0x68c0),
    (0x12c802, 0x7880),
    (0x12c804, 0x740),
    (0x12c806, 0xf40),
    (0x12c808, 0x1840),
    (0x12c80a, 0x7804),
];

const CONTRACT_WORDS: &[(u32, u32)] = &[
    (0x124ef4, 0x83aca0c),
    (0x124ef8, 0x8124efc),
    (0x1253f0, 0x83aca0c),
    (0x12665c, 0x83acebc),
    (0x128b40, 0x83ad174),
    (0x128b48, 0x16b),
    (0x128b4c, 0x8128b50),
    (0x12c81c, 0x83ad240),
];
