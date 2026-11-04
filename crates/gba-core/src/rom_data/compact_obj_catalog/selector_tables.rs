//! Source-bounded selector/position fields; unread row padding stays unowned.
use serde::{Deserialize, Serialize};

pub const MOTION_OFFSET: u32 = 0x3a80c8;
pub const POSITION_OFFSET: u32 = 0x39f7a4;
pub const TIMED_POSITION_OFFSET: u32 = 0x3a86d0;
pub const MOTION_POSITION_OFFSET: u32 = 0x3a8610;
pub const HORIZONTAL_OFFSET: u32 = 0x3a6b10;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table<T> { pub source_offset: u32, pub rows: Vec<T> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionRow {
    pub selector: u8,
    pub x: u8,
    pub y: u8,
    pub target_x: i16,
    pub target_y: i16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionRow { pub match_value: u8, pub selector: u8, pub x: i16, pub y: i16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionPositionRow { pub threshold: u8, pub selector: u8, pub x: i16, pub y: i16, pub dx: i8, pub dy: i8, pub x_rate: u8, pub y_rate: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HorizontalRow { pub match_value: u8, pub selector: u8, pub x: i16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectorTables {
    pub motion: Table<MotionRow>,
    pub positions: Table<PositionRow>,
    pub horizontal: Table<HorizontalRow>,
    // Historical v2 catalogs have no timed-position table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timed_positions: Option<Table<PositionRow>>,
    // Historical v3 catalogs have no motion-position table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_positions: Option<Table<MotionPositionRow>>,
}

impl SelectorTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.motion.source_offset != MOTION_OFFSET || self.motion.rows.len() != 14
            || self.positions.source_offset != POSITION_OFFSET || self.positions.rows.len() != 10
            || self.horizontal.source_offset != HORIZONTAL_OFFSET || self.horizontal.rows.len() != 15
            || self.timed_positions.as_ref().is_none_or(|table| table.source_offset != TIMED_POSITION_OFFSET || table.rows.len() != 20)
            || self.motion_positions.as_ref().is_none_or(|table| table.source_offset != MOTION_POSITION_OFFSET || table.rows.len() != 16)
        { return Err("Invalid compact selector table bindings".into()); }
        Ok(())
    }

    pub(super) fn from_rom(rom: &[u8]) -> Self {
        let signed = |at: usize| i16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        Self {
            motion: Table { source_offset: MOTION_OFFSET, rows: (0..14).map(|index| {
                let at = MOTION_OFFSET as usize + index * 8;
                MotionRow { selector: rom[at], x: rom[at + 1], y: rom[at + 2], target_x: signed(at + 4), target_y: signed(at + 6) }
            }).collect() },
            positions: Table { source_offset: POSITION_OFFSET, rows: (0..10).map(|index| {
                let at = POSITION_OFFSET as usize + index * 8;
                PositionRow { match_value: rom[at], selector: rom[at + 1], x: signed(at + 2), y: signed(at + 4) }
            }).collect() },
            motion_positions: Some(Table { source_offset: MOTION_POSITION_OFFSET, rows: (0..16).map(|index| {
                let at = MOTION_POSITION_OFFSET as usize + index * 12;
                MotionPositionRow { threshold: rom[at], selector: rom[at + 1], x: signed(at + 2), y: signed(at + 4), dx: rom[at + 6] as i8, dy: rom[at + 7] as i8, x_rate: rom[at + 8], y_rate: rom[at + 9] }
            }).collect() }),
            timed_positions: Some(Table { source_offset: TIMED_POSITION_OFFSET, rows: (0..20).map(|index| {
                let at = TIMED_POSITION_OFFSET as usize + index * 8;
                PositionRow { match_value: rom[at], selector: rom[at + 1], x: signed(at + 2), y: signed(at + 4) }
            }).collect() }),
            horizontal: Table { source_offset: HORIZONTAL_OFFSET, rows: (0..15).map(|index| {
                let at = HORIZONTAL_OFFSET as usize + index * 4;
                HorizontalRow { match_value: rom[at], selector: rom[at + 1], x: signed(at + 2) }
            }).collect() },
        }
    }

    pub(super) fn source_pieces(&self) -> Vec<(u32, Vec<u8>)> {
        let mut pieces = Vec::new();
        for (index, row) in self.motion.rows.iter().enumerate() {
            let at = MOTION_OFFSET + index as u32 * 8;
            pieces.push((at, vec![row.selector, row.x, row.y]));
            let mut targets = row.target_x.to_le_bytes().to_vec(); targets.extend(row.target_y.to_le_bytes());
            pieces.push((at + 4, targets));
        }
        for (index, row) in self.positions.rows.iter().enumerate() {
            let mut bytes = vec![row.match_value, row.selector]; bytes.extend(row.x.to_le_bytes()); bytes.extend(row.y.to_le_bytes());
            pieces.push((POSITION_OFFSET + index as u32 * 8, bytes));
        }
        if let Some(table) = &self.motion_positions {
            for (index, row) in table.rows.iter().enumerate() {
                let mut bytes = vec![row.threshold, row.selector]; bytes.extend(row.x.to_le_bytes()); bytes.extend(row.y.to_le_bytes());
                bytes.extend([row.dx as u8, row.dy as u8, row.x_rate, row.y_rate]);
                pieces.push((MOTION_POSITION_OFFSET + index as u32 * 12, bytes));
            }
        }
        if let Some(table) = &self.timed_positions {
            for (index, row) in table.rows.iter().enumerate() {
                let mut bytes = vec![row.match_value, row.selector]; bytes.extend(row.x.to_le_bytes()); bytes.extend(row.y.to_le_bytes());
                pieces.push((TIMED_POSITION_OFFSET + index as u32 * 8, bytes));
            }
        }
        for (index, row) in self.horizontal.rows.iter().enumerate() {
            let mut bytes = vec![row.match_value, row.selector]; bytes.extend(row.x.to_le_bytes());
            pieces.push((HORIZONTAL_OFFSET + index as u32 * 4, bytes));
        }
        pieces
    }

    pub(super) fn byte_at(&self, offset: u32) -> Option<u8> {
        if let Some(relative) = offset.checked_sub(MOTION_OFFSET) {
            if let Some(row) = self.motion.rows.get(relative as usize / 8) {
                return match relative % 8 {
                    0 => Some(row.selector), 1 => Some(row.x), 2 => Some(row.y),
                    4 | 5 => Some(row.target_x.to_le_bytes()[relative as usize % 8 - 4]),
                    6 | 7 => Some(row.target_y.to_le_bytes()[relative as usize % 8 - 6]),
                    _ => None,
                };
            }
        }
        if let Some(relative) = offset.checked_sub(POSITION_OFFSET) {
            if let Some(row) = self.positions.rows.get(relative as usize / 8) {
                return match relative % 8 {
                    0 => Some(row.match_value), 1 => Some(row.selector),
                    2 | 3 => Some(row.x.to_le_bytes()[relative as usize % 8 - 2]),
                    4 | 5 => Some(row.y.to_le_bytes()[relative as usize % 8 - 4]),
                    _ => None,
                };
            }
        }
        if let Some(relative) = offset.checked_sub(MOTION_POSITION_OFFSET) {
            if let Some(row) = self.motion_positions.as_ref().and_then(|table| table.rows.get(relative as usize / 12)) {
                return match relative % 12 {
                    0 => Some(row.threshold), 1 => Some(row.selector),
                    2 | 3 => Some(row.x.to_le_bytes()[relative as usize % 12 - 2]),
                    4 | 5 => Some(row.y.to_le_bytes()[relative as usize % 12 - 4]),
                    6 => Some(row.dx as u8), 7 => Some(row.dy as u8), 8 => Some(row.x_rate), 9 => Some(row.y_rate),
                    _ => None,
                };
            }
        }
        if let Some(relative) = offset.checked_sub(TIMED_POSITION_OFFSET) {
            if let Some(row) = self.timed_positions.as_ref().and_then(|table| table.rows.get(relative as usize / 8)) {
                return match relative % 8 {
                    0 => Some(row.match_value), 1 => Some(row.selector),
                    2 | 3 => Some(row.x.to_le_bytes()[relative as usize % 8 - 2]),
                    4 | 5 => Some(row.y.to_le_bytes()[relative as usize % 8 - 4]),
                    _ => None,
                };
            }
        }
        if let Some(relative) = offset.checked_sub(HORIZONTAL_OFFSET) {
            if let Some(row) = self.horizontal.rows.get(relative as usize / 4) {
                return match relative % 4 {
                    0 => Some(row.match_value), 1 => Some(row.selector),
                    2 | 3 => Some(row.x.to_le_bytes()[relative as usize % 4 - 2]),
                    _ => None,
                };
            }
        }
        None
    }

    pub(crate) fn resident_bytes(&self) -> usize {
        self.motion.rows.capacity() * std::mem::size_of::<MotionRow>()
            + self.positions.rows.capacity() * std::mem::size_of::<PositionRow>()
            + self.horizontal.rows.capacity() * std::mem::size_of::<HorizontalRow>()
            + self.timed_positions.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<PositionRow>())
            + self.motion_positions.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<MotionPositionRow>())
    }
}
