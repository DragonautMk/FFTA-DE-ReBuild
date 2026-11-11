//! Staged typed owner for conditional layered OBJ data; guest timing stays live.
use super::{obj_animations::ObjAnimationResources, ObjAnimationStats};
use crate::ffta_rom;
use crate::rom_data::{
    layered_obj_animations::{ContextActivation, ContextCoordinateRecord, ContextLanguagePositions, ContextTimedEvent, ContextTimeline, ContextTimelineRecord, Descriptor, LayeredObjAnimations,
        CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS,
        CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B,
        CONTEXT_EVENT_ROOT, CONTEXT_TIMELINE_ROOT, CONTEXT_TIMELINE_SOURCE_A, CONTEXT_TIMELINE_SOURCE_B,
        DESCRIPTOR_BYTES, DESCRIPTOR_ROOT},
    obj_animations::Banks,
    obj_lists::List,
    SourceRange,
};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayeredObjStats {
    pub fields: ObjAnimationStats,
    pub descriptor_reads: u64,
    pub descriptor_bytes: u64,
    pub coordinate_reads: u64,
    pub coordinate_bytes: u64,
    pub timeline_reads: u64,
    pub timeline_table_bytes: u64,
    pub timeline_source_bytes: u64,
    pub event_reads: u64,
    pub event_bytes: u64,
    pub language_position_reads: u64,
    pub language_position_a_bytes: u64,
    pub language_position_b_bytes: u64,
    pub activation_reads: u64,
    pub activation_tick_bytes: u64,
    pub activation_gate_bytes: u64,
    pub target_coordinate_reads: u64,
    pub target_coordinate_bytes: u64,
    pub additional_geometry_reads: u64,
    pub additional_geometry_bytes: u64,
}

pub(super) struct LayeredObjResources {
    descriptors: Vec<Descriptor>,
    context_coordinate_records: Vec<ContextCoordinateRecord>,
    context_timeline: Option<ContextTimeline>,
    context_timed_events: Option<Vec<ContextTimedEvent>>,
    context_language_positions: Option<ContextLanguagePositions>,
    context_activation: Option<ContextActivation>,
    context_target_coordinate: Option<ContextCoordinateRecord>,
    context_additional_geometry: Vec<ContextCoordinateRecord>,
    fields: ObjAnimationResources,
    descriptor_reads: AtomicU64,
    descriptor_bytes: AtomicU64,
    coordinate_reads: AtomicU64,
    coordinate_bytes: AtomicU64,
    timeline_reads: AtomicU64,
    timeline_table_bytes: AtomicU64,
    timeline_source_bytes: AtomicU64,
    event_reads: AtomicU64,
    event_bytes: AtomicU64,
    language_position_reads: AtomicU64,
    language_position_a_bytes: AtomicU64,
    language_position_b_bytes: AtomicU64,
    activation_reads: AtomicU64,
    activation_tick_bytes: AtomicU64,
    activation_gate_bytes: AtomicU64,
    target_coordinate_reads: AtomicU64,
    target_coordinate_bytes: AtomicU64,
    additional_geometry_reads: AtomicU64,
    additional_geometry_bytes: AtomicU64,
}

impl LayeredObjResources {
    pub(super) fn new(data: LayeredObjAnimations, sources: &[SourceRange]) -> Result<Self, String> {
        let spans = data.source_spans()?;
        if spans.len() != sources.len() {
            return Err("Layered OBJ source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in spans.iter().zip(sources) {
            if source.offset != *offset
                || source.length as usize != bytes.len()
                || source.role != *role
                || source.sha256 != ffta_rom::digest(bytes)
            {
                return Err("Layered OBJ typed source binding mismatch".into());
            }
        }
        let mut banks = data.banks;
        banks.sort_by_key(|bank| bank.object_root);
        Ok(Self {
            descriptors: data.descriptors,
            context_coordinate_records: data.context_coordinate_records,
            context_timeline: data.context_timeline,
            context_timed_events: data.context_timed_events,
            context_language_positions: data.context_language_positions,
            context_activation: data.context_activation,
            context_target_coordinate: data.context_target_coordinate,
            context_additional_geometry: data.context_additional_geometry,
            fields: ObjAnimationResources::new(Banks {
                banks,
                selector_tables: vec![],
            })?,
            descriptor_reads: AtomicU64::new(0),
            descriptor_bytes: AtomicU64::new(0),
            coordinate_reads: AtomicU64::new(0),
            coordinate_bytes: AtomicU64::new(0),
            timeline_reads: AtomicU64::new(0),
            timeline_table_bytes: AtomicU64::new(0),
            timeline_source_bytes: AtomicU64::new(0),
            event_reads: AtomicU64::new(0),
            event_bytes: AtomicU64::new(0),
            language_position_reads: AtomicU64::new(0),
            language_position_a_bytes: AtomicU64::new(0),
            language_position_b_bytes: AtomicU64::new(0),
            activation_reads: AtomicU64::new(0),
            activation_tick_bytes: AtomicU64::new(0),
            activation_gate_bytes: AtomicU64::new(0),
            target_coordinate_reads: AtomicU64::new(0),
            target_coordinate_bytes: AtomicU64::new(0),
            additional_geometry_reads: AtomicU64::new(0),
            additional_geometry_bytes: AtomicU64::new(0),
        })
    }

    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x3ad..=0x3ae | 0xb8c | 0xb8e..=0xb93)
    }

    fn descriptor_byte(&self, index: usize) -> Option<u8> {
        let at = index.checked_sub(DESCRIPTOR_ROOT as usize)?;
        let descriptor = self.descriptors.get(at / DESCRIPTOR_BYTES)?;
        let offset = at % DESCRIPTOR_BYTES;
        match offset {
            0..=3 => Some((descriptor.object_root + 0x08000000).to_le_bytes()[offset]),
            4..=7 => Some((descriptor.sequence_root + 0x08000000).to_le_bytes()[offset - 4]),
            _ => descriptor.opaque.get(offset - 8).copied(),
        }
    }

    fn coordinate_byte(&self, index: usize) -> Option<u8> {
        self.context_coordinate_records.iter().find_map(|record| {
            let relative = index.checked_sub(record.source_offset as usize)?;
            record.source_bytes().get(relative).copied()
        })
    }

    fn timeline_byte(&self, index: usize) -> Option<(u8, bool)> {
        let data = self.context_timeline.as_ref()?;
        if let Some(relative) = index.checked_sub(CONTEXT_TIMELINE_ROOT as usize) {
            if relative < data.records.len() * 20 {
                return Some((data.records[relative / 20].source_bytes()[relative % 20], true));
            }
        }
        for (base, words) in [(CONTEXT_TIMELINE_SOURCE_A, &data.source_a),
                              (CONTEXT_TIMELINE_SOURCE_B, &data.source_b)] {
            if let Some(relative) = index.checked_sub(base as usize) {
                if relative < 32 { return Some((words[relative / 2].to_le_bytes()[relative % 2], false)); }
            }
        }
        None
    }

    fn event_byte(&self, index: usize) -> Option<u8> {
        let records = self.context_timed_events.as_ref()?;
        let relative = index.checked_sub(CONTEXT_EVENT_ROOT as usize)?;
        records.get(relative / 16)?.source_bytes().get(relative % 16).copied()
    }

    fn language_position_byte(&self, index: usize) -> Option<(u8, bool)> {
        let data = self.context_language_positions.as_ref()?;
        for (base, rows, first) in [
            (CONTEXT_LANGUAGE_POSITION_A, &data.first, true),
            (CONTEXT_LANGUAGE_POSITION_B, &data.second, false),
        ] {
            if let Some(relative) = index.checked_sub(base as usize) {
                if relative < rows.len() * 2 {
                    return Some((rows[relative / 2].source_bytes()[relative % 2], first));
                }
            }
        }
        None
    }

    fn activation_byte(&self, index: usize) -> Option<(u8, bool)> {
        let data = self.context_activation.as_ref()?;
        if let Some(relative) = index.checked_sub(CONTEXT_ACTIVATION_GATES as usize) {
            if let Some(byte) = data.gates.get(relative) { return Some((*byte, false)); }
        }
        if let Some(relative) = index.checked_sub(CONTEXT_ACTIVATION_TICKS as usize) {
            if relative < data.ticks.len() * 4 {
                return Some((data.ticks[relative / 4].to_le_bytes()[relative % 4], true));
            }
        }
        None
    }

    fn target_coordinate_byte(&self, index: usize) -> Option<u8> {
        let target = self.context_target_coordinate.as_ref()?;
        let relative = index.checked_sub(target.source_offset as usize)?;
        target.source_bytes().get(relative).copied()
    }

    fn additional_geometry_byte(&self, index: usize) -> Option<u8> {
        self.context_additional_geometry.iter().find_map(|record| {
            let relative = index.checked_sub(record.source_offset as usize)?;
            record.source_bytes().get(relative).copied()
        })
    }

    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        if !Self::source_page(index)
            && !Self::source_page(index.saturating_add(N.saturating_sub(1)))
        {
            return prior;
        }
        let prior = self.fields.guest_read_projected(index, rom, prior);
        let mut used = 0;
        let mut coordinate_used = 0;
        let mut timeline_table_used = 0;
        let mut timeline_source_used = 0;
        let mut event_used = 0;
        let mut language_a_used = 0;
        let mut language_b_used = 0;
        let mut activation_tick_used = 0;
        let mut activation_gate_used = 0;
        let mut target_coordinate_used = 0;
        let mut additional_geometry_used = 0;
        let bytes = std::array::from_fn(|i| {
            if let Some(byte) = self.descriptor_byte(index + i) {
                used += 1;
                byte
            } else if let Some(byte) = self.coordinate_byte(index + i) {
                coordinate_used += 1;
                byte
            } else if let Some((byte, table)) = self.timeline_byte(index + i) {
                if table { timeline_table_used += 1; } else { timeline_source_used += 1; }
                byte
            } else if let Some(byte) = self.event_byte(index + i) {
                event_used += 1;
                byte
            } else if let Some((byte, first)) = self.language_position_byte(index + i) {
                if first { language_a_used += 1; } else { language_b_used += 1; }
                byte
            } else if let Some((byte, tick)) = self.activation_byte(index + i) {
                if tick { activation_tick_used += 1; } else { activation_gate_used += 1; }
                byte
            } else if let Some(byte) = self.target_coordinate_byte(index + i) {
                target_coordinate_used += 1;
                byte
            } else if let Some(byte) = self.additional_geometry_byte(index + i) {
                additional_geometry_used += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |p| p[i])
            }
        });
        if used == 0 && coordinate_used == 0 && timeline_table_used == 0 && timeline_source_used == 0 && event_used == 0 && language_a_used == 0 && language_b_used == 0 && activation_tick_used == 0 && activation_gate_used == 0 && target_coordinate_used == 0 && additional_geometry_used == 0 {
            return prior;
        }
        if used > 0 {
            self.descriptor_reads.fetch_add(1, Ordering::Relaxed);
            self.descriptor_bytes.fetch_add(used, Ordering::Relaxed);
        }
        if coordinate_used > 0 {
            self.coordinate_reads.fetch_add(1, Ordering::Relaxed);
            self.coordinate_bytes.fetch_add(coordinate_used, Ordering::Relaxed);
        }
        if timeline_table_used > 0 || timeline_source_used > 0 {
            self.timeline_reads.fetch_add(1, Ordering::Relaxed);
            self.timeline_table_bytes.fetch_add(timeline_table_used, Ordering::Relaxed);
            self.timeline_source_bytes.fetch_add(timeline_source_used, Ordering::Relaxed);
        }
        if event_used > 0 {
            self.event_reads.fetch_add(1, Ordering::Relaxed);
            self.event_bytes.fetch_add(event_used, Ordering::Relaxed);
        }
        if language_a_used > 0 || language_b_used > 0 {
            self.language_position_reads.fetch_add(1, Ordering::Relaxed);
            self.language_position_a_bytes.fetch_add(language_a_used, Ordering::Relaxed);
            self.language_position_b_bytes.fetch_add(language_b_used, Ordering::Relaxed);
        }
        if activation_tick_used > 0 || activation_gate_used > 0 {
            self.activation_reads.fetch_add(1, Ordering::Relaxed);
            self.activation_tick_bytes.fetch_add(activation_tick_used, Ordering::Relaxed);
            self.activation_gate_bytes.fetch_add(activation_gate_used, Ordering::Relaxed);
        }
        if target_coordinate_used > 0 {
            self.target_coordinate_reads.fetch_add(1, Ordering::Relaxed);
            self.target_coordinate_bytes.fetch_add(target_coordinate_used, Ordering::Relaxed);
        }
        if additional_geometry_used > 0 {
            self.additional_geometry_reads.fetch_add(1, Ordering::Relaxed);
            self.additional_geometry_bytes.fetch_add(additional_geometry_used, Ordering::Relaxed);
        }
        Some(bytes)
    }

    pub(super) fn host_list(&self, offset: u32) -> Option<&List> {
        self.fields.host_list(offset)
    }

    pub(super) fn stats(&self) -> LayeredObjStats {
        LayeredObjStats {
            fields: self.fields.stats(),
            descriptor_reads: self.descriptor_reads.load(Ordering::Relaxed),
            descriptor_bytes: self.descriptor_bytes.load(Ordering::Relaxed),
            coordinate_reads: self.coordinate_reads.load(Ordering::Relaxed),
            coordinate_bytes: self.coordinate_bytes.load(Ordering::Relaxed),
            timeline_reads: self.timeline_reads.load(Ordering::Relaxed),
            timeline_table_bytes: self.timeline_table_bytes.load(Ordering::Relaxed),
            timeline_source_bytes: self.timeline_source_bytes.load(Ordering::Relaxed),
            event_reads: self.event_reads.load(Ordering::Relaxed),
            event_bytes: self.event_bytes.load(Ordering::Relaxed),
            language_position_reads: self.language_position_reads.load(Ordering::Relaxed),
            language_position_a_bytes: self.language_position_a_bytes.load(Ordering::Relaxed),
            language_position_b_bytes: self.language_position_b_bytes.load(Ordering::Relaxed),
            activation_reads: self.activation_reads.load(Ordering::Relaxed),
            activation_tick_bytes: self.activation_tick_bytes.load(Ordering::Relaxed),
            activation_gate_bytes: self.activation_gate_bytes.load(Ordering::Relaxed),
            target_coordinate_reads: self.target_coordinate_reads.load(Ordering::Relaxed),
            target_coordinate_bytes: self.target_coordinate_bytes.load(Ordering::Relaxed),
            additional_geometry_reads: self.additional_geometry_reads.load(Ordering::Relaxed),
            additional_geometry_bytes: self.additional_geometry_bytes.load(Ordering::Relaxed),
        }
    }

    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.descriptors.capacity() * std::mem::size_of::<Descriptor>()
            + self.context_coordinate_records.capacity() * std::mem::size_of::<ContextCoordinateRecord>()
            + self.context_additional_geometry.capacity() * std::mem::size_of::<ContextCoordinateRecord>()
            + self.context_timeline.as_ref().map_or(0, |v| v.records.capacity() * std::mem::size_of::<ContextTimelineRecord>())
            + self.context_timed_events.as_ref().map_or(0, |v| v.capacity() * std::mem::size_of::<ContextTimedEvent>())
            + self.context_activation.as_ref().map_or(0, |v| v.ticks.capacity() * std::mem::size_of::<u32>() + v.gates.capacity())
            + self.fields.resident_bytes()
            - std::mem::size_of::<ObjAnimationResources>()
    }
}

#[cfg(test)]
mod tests;
