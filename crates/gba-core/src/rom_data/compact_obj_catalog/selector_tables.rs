//! Source-bounded selector/position fields; unread row padding stays unowned.
use serde::{Deserialize, Serialize};

pub const MOTION_OFFSET: u32 = 0x3a80c8;
pub const POSITION_OFFSET: u32 = 0x39f7a4;
pub const TIMED_POSITION_OFFSET: u32 = 0x3a86d0;
pub const MOTION_POSITION_OFFSET: u32 = 0x3a8610;
pub const HORIZONTAL_OFFSET: u32 = 0x3a6b10;
pub const TIMER_OFFSET: u32 = 0x39e164;
pub const SCHEDULE_OFFSET: u32 = 0x39e0c4;
pub const FLIGHT_OFFSET: u32 = 0x39dc50;
pub const BURST_OFFSET: u32 = 0x39f6cc;
pub const EFFECT_OFFSET: u32 = 0x39defc;
pub const GRID_FIRST_OFFSET: u32 = 0x3a9abc;
pub const GRID_SECOND_OFFSET: u32 = 0x3aae44;
pub const SCRIPT_LOOKUP_OFFSET: u32 = 0x3ab98c;
pub const SCENE_OFFSET: u32 = 0x3ac944;

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
pub struct TimerRow { pub match_value: u8, pub selector: u8, pub x: u8, pub y: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRow { pub match_value: u16, pub selector: u8, pub option: u8, pub x: u8, pub y: u8, pub velocity: i16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlightRow { pub selector: u8, pub vector_index: u8, pub trigger: u16, pub x: i16, pub y: i16 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BurstRow { pub trigger: u16, pub selector: u8, pub duration: u8, pub x: i16, pub y: i16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneRow { pub first_selector: u8, pub second_selector: u8, pub x: i16, pub y: i16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridRow {
    pub base_selector: u8, pub x_mode: u8, pub y_mode: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub flag: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub signed_flag: Option<i8>,
    pub x: i16, pub y: i16, pub value: u32,
}

impl GridRow {
    fn normalize(mode: u8) -> (u8, bool) {
        if (3..=5).contains(&mode) || mode > 8 { (mode.wrapping_sub(3), true) } else { (mode, false) }
    }
    pub(super) fn selector(&self) -> u16 {
        let adjusted = Self::normalize(self.x_mode).1 || Self::normalize(self.y_mode).1;
        u16::from(self.base_selector.wrapping_add(if adjusted { 4 } else { 0 }))
    }
    fn reads_flag(&self) -> bool {
        let x = Self::normalize(self.x_mode).0; let y = Self::normalize(self.y_mode).0;
        (x == 0 || x == 6) && (y == 0 || y == 6)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRow {
    pub kind: u8, pub trigger: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub selector: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub x: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub y: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub target_x: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub target_y: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub effect: Option<u16>,
}

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
    // Historical v4 catalogs have no timer-position table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timer_positions: Option<Table<TimerRow>>,
    // Historical v5 catalogs have no schedule table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<Table<ScheduleRow>>,
    // Historical v6 catalogs have no flight or burst tables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flight: Option<Table<FlightRow>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst: Option<Table<BurstRow>>,
    // Historical v7 catalogs have no effect table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Table<EffectRow>>,
    // Historical v8 catalogs have no bounded grid tables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_first: Option<Table<GridRow>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_second: Option<Table<GridRow>>,
    // Historical v9 catalogs have no secondary-script lookup projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_lookup: Option<Table<u8>>,
    // Historical v10 catalogs have no scene row projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<Table<SceneRow>>,
}

impl SelectorTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.motion.source_offset != MOTION_OFFSET || self.motion.rows.len() != 14
            || self.positions.source_offset != POSITION_OFFSET || self.positions.rows.len() != 10
            || self.horizontal.source_offset != HORIZONTAL_OFFSET || self.horizontal.rows.len() != 15
            || self.timed_positions.as_ref().is_none_or(|table| table.source_offset != TIMED_POSITION_OFFSET || table.rows.len() != 20)
            || self.motion_positions.as_ref().is_none_or(|table| table.source_offset != MOTION_POSITION_OFFSET || table.rows.len() != 16)
            || self.timer_positions.as_ref().is_none_or(|table| table.source_offset != TIMER_OFFSET || table.rows.len() != 44)
            || self.schedule.as_ref().is_none_or(|table| table.source_offset != SCHEDULE_OFFSET || table.rows.len() != 12)
            || self.flight.as_ref().is_none_or(|table| table.source_offset != FLIGHT_OFFSET || table.rows.len() != 30)
            || self.burst.as_ref().is_none_or(|table| table.source_offset != BURST_OFFSET || table.rows.len() != 11)
            || self.effects.as_ref().is_none_or(|table| table.source_offset != EFFECT_OFFSET || table.rows.len() != 10)
            || self.grid_first.as_ref().is_none_or(|table| table.source_offset != GRID_FIRST_OFFSET || table.rows.len() != 24)
            || self.grid_second.as_ref().is_none_or(|table| table.source_offset != GRID_SECOND_OFFSET || table.rows.len() != 24)
            || self.script_lookup.as_ref().is_none_or(|table| table.source_offset != SCRIPT_LOOKUP_OFFSET || table.rows.len() != 4)
            || self.scene.as_ref().is_none_or(|table| table.source_offset != SCENE_OFFSET || table.rows.len() != 20)
        { return Err("Invalid compact selector table bindings".into()); }
        for row in &self.effects.as_ref().unwrap().rows {
            let active = row.kind != 1; let animated = row.kind == 0;
            if row.selector.is_some() != active || row.x.is_some() != active || row.y.is_some() != active
                || row.duration.is_some() != active || row.target_x.is_some() != animated
                || row.target_y.is_some() != animated || row.effect.is_some() != animated {
                return Err("Invalid conditional effect row projection".into());
            }
        }
        for row in self.grid_first.as_ref().unwrap().rows.iter().chain(&self.grid_second.as_ref().unwrap().rows) {
            if row.flag.is_some() != row.reads_flag() || row.signed_flag.is_some() != (row.reads_flag() && row.flag == Some(0)) {
                return Err("Invalid conditional grid row projection".into());
            }
        }
        Ok(())
    }

    pub(super) fn from_rom(rom: &[u8]) -> Self {
        let signed = |at: usize| i16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let grid = |offset: u32| Table { source_offset: offset, rows: (0..24).map(|index| {
            let at = offset as usize + index * 16;
            let mut row = GridRow { base_selector: rom[at], x_mode: rom[at + 1], y_mode: rom[at + 2], flag: None, signed_flag: None, x: signed(at + 6), y: signed(at + 8), value: u32::from_le_bytes(rom[at + 12..at + 16].try_into().unwrap()) };
            if row.reads_flag() { row.flag = Some(rom[at + 3]); if row.flag == Some(0) { row.signed_flag = Some(rom[at + 4] as i8); } }
            row
        }).collect() };
        Self {
            scene: Some(Table { source_offset: SCENE_OFFSET, rows: (0..20).map(|index| {
                let at = SCENE_OFFSET as usize + index * 8;
                SceneRow { first_selector: rom[at], second_selector: rom[at + 1], x: signed(at + 2), y: signed(at + 4) }
            }).collect() }),
            script_lookup: Some(Table { source_offset: SCRIPT_LOOKUP_OFFSET, rows: rom[SCRIPT_LOOKUP_OFFSET as usize..SCRIPT_LOOKUP_OFFSET as usize + 4].to_vec() }),
            grid_first: Some(grid(GRID_FIRST_OFFSET)), grid_second: Some(grid(GRID_SECOND_OFFSET)),
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
            effects: Some(Table { source_offset: EFFECT_OFFSET, rows: (0..10).map(|index| {
                let at = EFFECT_OFFSET as usize + index * 16; let kind = rom[at]; let active = kind != 1; let animated = kind == 0;
                EffectRow { kind, trigger: u16::from_le_bytes([rom[at + 2], rom[at + 3]]), selector: active.then(|| rom[at + 1]), x: active.then(|| signed(at + 4)), y: active.then(|| signed(at + 6)), target_x: animated.then(|| signed(at + 8)), target_y: animated.then(|| signed(at + 10)), duration: active.then(|| u16::from_le_bytes([rom[at + 12], rom[at + 13]])), effect: animated.then(|| u16::from_le_bytes([rom[at + 14], rom[at + 15]])) }
            }).collect() }),
            flight: Some(Table { source_offset: FLIGHT_OFFSET, rows: (0..30).map(|index| {
                let at = FLIGHT_OFFSET as usize + index * 8;
                FlightRow { selector: rom[at], vector_index: rom[at + 1], trigger: u16::from_le_bytes([rom[at + 2], rom[at + 3]]), x: signed(at + 4), y: signed(at + 6) }
            }).collect() }),
            burst: Some(Table { source_offset: BURST_OFFSET, rows: (0..11).map(|index| {
                let at = BURST_OFFSET as usize + index * 8;
                BurstRow { trigger: u16::from_le_bytes([rom[at], rom[at + 1]]), selector: rom[at + 2], duration: rom[at + 3], x: signed(at + 4), y: signed(at + 6) }
            }).collect() }),
            schedule: Some(Table { source_offset: SCHEDULE_OFFSET, rows: (0..12).map(|index| {
                let at = SCHEDULE_OFFSET as usize + index * 8;
                ScheduleRow { match_value: u16::from_le_bytes([rom[at], rom[at + 1]]), selector: rom[at + 2], option: rom[at + 3], x: rom[at + 4], y: rom[at + 5], velocity: signed(at + 6) }
            }).collect() }),
            timer_positions: Some(Table { source_offset: TIMER_OFFSET, rows: (0..44).map(|index| {
                let at = TIMER_OFFSET as usize + index * 4;
                TimerRow { match_value: rom[at], selector: rom[at + 1], x: rom[at + 2], y: rom[at + 3] }
            }).collect() }),
            horizontal: Table { source_offset: HORIZONTAL_OFFSET, rows: (0..15).map(|index| {
                let at = HORIZONTAL_OFFSET as usize + index * 4;
                HorizontalRow { match_value: rom[at], selector: rom[at + 1], x: signed(at + 2) }
            }).collect() },
        }
    }

    pub(super) fn source_pieces(&self) -> Vec<(u32, Vec<u8>)> {
        let mut pieces = Vec::new();
        if let Some(table) = &self.scene {
            for (index, row) in table.rows.iter().enumerate() {
                let mut bytes = vec![row.first_selector, row.second_selector]; bytes.extend(row.x.to_le_bytes()); bytes.extend(row.y.to_le_bytes());
                pieces.push((table.source_offset + index as u32 * 8, bytes));
            }
        }
        if let Some(table) = &self.script_lookup { pieces.push((table.source_offset, table.rows.clone())); }
        for table in [&self.grid_first, &self.grid_second].into_iter().flatten() {
            for (index, row) in table.rows.iter().enumerate() {
                let at = table.source_offset + index as u32 * 16;
                let mut head = vec![row.base_selector, row.x_mode, row.y_mode];
                if let Some(flag) = row.flag { head.push(flag); }
                pieces.push((at, head));
                if let Some(flag) = row.signed_flag { pieces.push((at + 4, vec![flag as u8])); }
                let mut position = row.x.to_le_bytes().to_vec(); position.extend(row.y.to_le_bytes());
                pieces.push((at + 6, position)); pieces.push((at + 12, row.value.to_le_bytes().to_vec()));
            }
        }
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
        if let Some(table) = &self.effects {
            for (index, row) in table.rows.iter().enumerate() {
                let at = EFFECT_OFFSET + index as u32 * 16;
                let mut head = vec![row.kind]; if let Some(selector) = row.selector { head.push(selector); }
                pieces.push((at, head)); pieces.push((at + 2, row.trigger.to_le_bytes().to_vec()));
                for (offset, value) in [(4, row.x), (6, row.y), (8, row.target_x), (10, row.target_y)] {
                    if let Some(value) = value { pieces.push((at + offset, value.to_le_bytes().to_vec())); }
                }
                for (offset, value) in [(12, row.duration), (14, row.effect)] {
                    if let Some(value) = value { pieces.push((at + offset, value.to_le_bytes().to_vec())); }
                }
            }
        }
        if let Some(table) = &self.flight {
            for (index, row) in table.rows.iter().enumerate() {
                let mut bytes = vec![row.selector, row.vector_index]; bytes.extend(row.trigger.to_le_bytes()); bytes.extend(row.x.to_le_bytes()); bytes.extend(row.y.to_le_bytes());
                pieces.push((FLIGHT_OFFSET + index as u32 * 8, bytes));
            }
        }
        if let Some(table) = &self.burst {
            for (index, row) in table.rows.iter().enumerate() {
                let mut bytes = row.trigger.to_le_bytes().to_vec(); bytes.extend([row.selector, row.duration]); bytes.extend(row.x.to_le_bytes()); bytes.extend(row.y.to_le_bytes());
                pieces.push((BURST_OFFSET + index as u32 * 8, bytes));
            }
        }
        if let Some(table) = &self.schedule {
            for (index, row) in table.rows.iter().enumerate() {
                let mut bytes = row.match_value.to_le_bytes().to_vec();
                bytes.extend([row.selector, row.option, row.x, row.y]); bytes.extend(row.velocity.to_le_bytes());
                pieces.push((SCHEDULE_OFFSET + index as u32 * 8, bytes));
            }
        }
        if let Some(table) = &self.timer_positions {
            for (index, row) in table.rows.iter().enumerate() {
                pieces.push((TIMER_OFFSET + index as u32 * 4, vec![row.match_value, row.selector, row.x, row.y]));
            }
        }
        for (index, row) in self.horizontal.rows.iter().enumerate() {
            let mut bytes = vec![row.match_value, row.selector]; bytes.extend(row.x.to_le_bytes());
            pieces.push((HORIZONTAL_OFFSET + index as u32 * 4, bytes));
        }
        pieces
    }

    pub(super) fn byte_at(&self, offset: u32) -> Option<u8> {
        if let Some(table) = &self.scene {
            if let Some(relative) = offset.checked_sub(table.source_offset) {
                if let Some(row) = table.rows.get(relative as usize / 8) {
                    let index = relative as usize % 8;
                    return match index { 0 => Some(row.first_selector), 1 => Some(row.second_selector), 2 | 3 => Some(row.x.to_le_bytes()[index - 2]), 4 | 5 => Some(row.y.to_le_bytes()[index - 4]), _ => None };
                }
            }
        }
        if let Some(table) = &self.script_lookup {
            if let Some(relative) = offset.checked_sub(table.source_offset) {
                if let Some(byte) = table.rows.get(relative as usize) { return Some(*byte); }
            }
        }
        for table in [&self.grid_first, &self.grid_second].into_iter().flatten() {
            if let Some(relative) = offset.checked_sub(table.source_offset) {
                if let Some(row) = table.rows.get(relative as usize / 16) {
                    let index = relative as usize % 16;
                    return match index { 0 => Some(row.base_selector), 1 => Some(row.x_mode), 2 => Some(row.y_mode), 3 => row.flag, 4 => row.signed_flag.map(|v| v as u8), 6 | 7 => Some(row.x.to_le_bytes()[index - 6]), 8 | 9 => Some(row.y.to_le_bytes()[index - 8]), 12..=15 => Some(row.value.to_le_bytes()[index - 12]), _ => None };
                }
            }
        }
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
        if let Some(relative) = offset.checked_sub(EFFECT_OFFSET) {
            if let Some(row) = self.effects.as_ref().and_then(|table| table.rows.get(relative as usize / 16)) {
                let index = relative as usize % 16;
                return match index {
                    0 => Some(row.kind), 1 => row.selector, 2 | 3 => Some(row.trigger.to_le_bytes()[index - 2]),
                    4 | 5 => row.x.map(|value| value.to_le_bytes()[index - 4]),
                    6 | 7 => row.y.map(|value| value.to_le_bytes()[index - 6]),
                    8 | 9 => row.target_x.map(|value| value.to_le_bytes()[index - 8]),
                    10 | 11 => row.target_y.map(|value| value.to_le_bytes()[index - 10]),
                    12 | 13 => row.duration.map(|value| value.to_le_bytes()[index - 12]),
                    14 | 15 => row.effect.map(|value| value.to_le_bytes()[index - 14]), _ => None,
                };
            }
        }
        if let Some(relative) = offset.checked_sub(FLIGHT_OFFSET) {
            if let Some(row) = self.flight.as_ref().and_then(|table| table.rows.get(relative as usize / 8)) {
                return match relative % 8 { 0 => Some(row.selector), 1 => Some(row.vector_index), 2 | 3 => Some(row.trigger.to_le_bytes()[relative as usize % 8 - 2]), 4 | 5 => Some(row.x.to_le_bytes()[relative as usize % 8 - 4]), 6 | 7 => Some(row.y.to_le_bytes()[relative as usize % 8 - 6]), _ => None };
            }
        }
        if let Some(relative) = offset.checked_sub(BURST_OFFSET) {
            if let Some(row) = self.burst.as_ref().and_then(|table| table.rows.get(relative as usize / 8)) {
                return match relative % 8 { 0 | 1 => Some(row.trigger.to_le_bytes()[relative as usize % 8]), 2 => Some(row.selector), 3 => Some(row.duration), 4 | 5 => Some(row.x.to_le_bytes()[relative as usize % 8 - 4]), 6 | 7 => Some(row.y.to_le_bytes()[relative as usize % 8 - 6]), _ => None };
            }
        }
        if let Some(relative) = offset.checked_sub(SCHEDULE_OFFSET) {
            if let Some(row) = self.schedule.as_ref().and_then(|table| table.rows.get(relative as usize / 8)) {
                return match relative % 8 {
                    0 | 1 => Some(row.match_value.to_le_bytes()[relative as usize % 8]),
                    2 => Some(row.selector), 3 => Some(row.option), 4 => Some(row.x), 5 => Some(row.y),
                    6 | 7 => Some(row.velocity.to_le_bytes()[relative as usize % 8 - 6]), _ => None,
                };
            }
        }
        if let Some(relative) = offset.checked_sub(TIMER_OFFSET) {
            if let Some(row) = self.timer_positions.as_ref().and_then(|table| table.rows.get(relative as usize / 4)) {
                return Some([row.match_value, row.selector, row.x, row.y][relative as usize % 4]);
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
            + self.timer_positions.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<TimerRow>())
            + self.schedule.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<ScheduleRow>())
            + self.flight.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<FlightRow>())
            + self.burst.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<BurstRow>())
            + self.effects.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<EffectRow>())
            + self.grid_first.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<GridRow>())
            + self.grid_second.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<GridRow>())
            + self.script_lookup.as_ref().map_or(0, |table| table.rows.capacity())
            + self.scene.as_ref().map_or(0, |table| table.rows.capacity() * std::mem::size_of::<SceneRow>())
    }
}
