//! Conditional EUR layered OBJ banks. Guest task state and timing stay live.
use super::{obj_animations, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const DESCRIPTOR_ROOT: u32 = 0x3ae1a4;
pub const DESCRIPTOR_BYTES: usize = 28;
/// Separate coordinate records copied by the adjacent context initializer.
pub const CONTEXT_COORDINATE_OFFSETS: [u32; 6] = [
    0x3ae12c, 0x3ae134, 0x3ae13c, 0x3ae144, 0x3ae14c, 0x3ae164,
];
pub const CONTEXT_TARGET_COORDINATE: u32 = 0x3ae16c;
pub const CONTEXT_ADDITIONAL_GEOMETRY: [u32; 8] = [0x3ae154, 0x3ae15c, 0x3ae174, 0x3ae17c, 0x3ae184, 0x3ae18c, 0x3ae194, 0x3ae19c];
pub const CONTEXT_TIMELINE_ROOT: u32 = 0x3ae038;
pub const CONTEXT_EVENT_ROOT: u32 = 0x3adfe8;
pub const TITLE_MAIN_POSITION_ROOT: u32 = 0x3adca0;
pub const TITLE_MAIN_OBJECT_ROOT: u32 = 0xb8ed28;
pub const TITLE_MAIN_SEQUENCE_ROOT: u32 = 0xb8ecb4;
pub const TITLE_LANGUAGE_POSITION_ROOT: u32 = 0x3adcfc;
pub const TITLE_OBJECT_ROOT: u32 = 0xb8eda4;
pub const TITLE_SEQUENCE_ROOT: u32 = 0xb8ecd4;
pub const CONTEXT_LANGUAGE_POSITION_A: u32 = 0x3ae42c;
pub const CONTEXT_LANGUAGE_POSITION_B: u32 = 0x3ae436;
pub const CONTEXT_ACTIVATION_GATES: u32 = 0x3ae364;
pub const CONTEXT_ACTIVATION_TICKS: u32 = 0x3ae38c;
pub const CONTEXT_TIMELINE_SOURCE_A: u32 = 0xb8c318;
pub const CONTEXT_TIMELINE_SOURCE_B: u32 = 0xb8c338;
const CONTEXT_TIMELINE_RECORD_BYTES: usize = 20;
const CONTEXT_COORDINATE_CALLS: [(usize, usize, u32); 6] = [
    (0x140b3a, 0x140d44, 0x3ae12c),
    (0x140bd6, 0x140d5c, 0x3ae134),
    (0x140e1e, 0x140e68, 0x3ae164),
    (0x14108a, 0x141264, 0x3ae13c),
    (0x1412ec, 0x141654, 0x3ae144),
    (0x1414b8, 0x141690, 0x3ae14c),
];
pub const BANK_SHAPES: &[(u32, u32, usize, &[usize])] = &[
    (0xb8fdf0, 0xb8ef84, 80, &[29, 15, 29]),
    (0xb90568, 0xb8f0c0, 85, &[33, 17, 33]),
    (0xb93cb8, 0xb8f818, 2, &[1]),
    (0xb90bb0, 0xb8f224, 56, &[57, 1, 1]),
    (0xb91de0, 0xb8f328, 56, &[57, 1, 1]),
    (0xb9369c, 0xb8f6e0, 67, &[30, 12, 30]),
    (0xb930d4, 0xb8f54c, 9, &[9, 9]),
    (0xb8f854, 0xb8ee88, 46, &[17, 9, 17]),
    (0xb93df4, 0xb8f824, 2, &[1]),
    (0xb92ad4, 0xb8f42c, 65, &[27, 9, 27]),
    (0xb93158, 0xb8f5a4, 84, &[29, 15, 29]),
    (0xb93e1c, 0xb8f830, 2, &[1]),
    (0xb93f3c, 0xb8f848, 2, &[1]),
    (0xb930c0, 0xb8f540, 2, &[1]),
    (0xb93ec8, 0xb8f83c, 2, &[1]),
    (0xb8fdac, 0xb8ef4c, 6, &[5, 5]),
];
const PRODUCERS: &[(usize, u8)] = &[
    (0x140fd0, 0),
    (0x140fea, 1),
    (0x141006, 3),
    (0x141022, 4),
    (0x14103e, 2),
    (0x1411d0, 5),
    (0x1411ec, 6),
    (0x141452, 7),
    (0x14146e, 8),
    (0x1415c6, 9),
    (0x1415e2, 10),
    (0x1415fc, 11),
    (0x141618, 12),
    (0x1417e6, 13),
    (0x141802, 15),
    (0x14181e, 14),
];
pub type SourceSpan = (u32, Vec<u8>, String);

/// The unclassified bytes retain the exact remaining descriptor storage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub object_root: u32,
    pub sequence_root: u32,
    pub opaque: [u8; 20],
}

/// Four halfwords copied unchanged; the two extents are read as signed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextCoordinateRecord {
    pub source_offset: u32,
    pub x_base: i16,
    pub y_base: i16,
    pub x_extent: i16,
    pub y_extent: i16,
}

impl ContextCoordinateRecord {
    pub fn source_bytes(&self) -> [u8; 8] {
        let mut bytes = [0; 8];
        for (chunk, value) in bytes.chunks_exact_mut(2).zip([
            self.x_base, self.y_base, self.x_extent, self.y_extent,
        ]) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

/// A bounded timing row consumed by the adjacent original context task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextTimelineRecord {
    pub first_tick: u32,
    pub second_tick: u32,
    pub source_a: u32,
    pub source_b: u32,
    pub argument: u8,
    pub mode: u8,
    pub duration: u16,
}

impl ContextTimelineRecord {
    pub fn source_bytes(&self) -> [u8; CONTEXT_TIMELINE_RECORD_BYTES] {
        let mut bytes = [0; CONTEXT_TIMELINE_RECORD_BYTES];
        for (chunk, value) in bytes[..16].chunks_exact_mut(4).zip([
            self.first_tick, self.second_tick, self.source_a, self.source_b,
        ]) { chunk.copy_from_slice(&value.to_le_bytes()); }
        bytes[16] = self.argument;
        bytes[17] = self.mode;
        bytes[18..20].copy_from_slice(&self.duration.to_le_bytes());
        bytes
    }
}

/// Independent table and copied halfword buffers; neither is an OBJ list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextTimeline {
    pub records: Vec<ContextTimelineRecord>,
    pub source_a: [u16; 16],
    pub source_b: [u16; 16],
}

/// Four timed event records and their separately preserved terminator record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextTimedEvent {
    pub tick: u32,
    pub x: i16,
    pub y: i16,
    pub argument_0: u16,
    pub argument_1: u16,
    pub argument_2: u16,
    pub argument_3: u8,
    pub argument_4: u8,
}

/// Unsigned placement arguments indexed by the live language byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextLanguagePosition {
    pub x: u8,
    pub y: u8,
}

impl ContextLanguagePosition {
    pub fn source_bytes(&self) -> [u8; 2] { [self.x, self.y] }
}

/// Two independent five-language source plans; guest indexing stays uncapped.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextLanguagePositions {
    pub first: [ContextLanguagePosition; 5],
    pub second: [ContextLanguagePosition; 5],
}

/// Source words and conditional gate bytes; activation flags remain in RAM.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextActivation {
    pub ticks: Vec<u32>,
    pub gates: Vec<u8>,
}

impl ContextActivation {
    fn validate(&self) -> Result<(), String> {
        if self.ticks.len() != 40 || self.gates.len() != 40 {
            return Err("Context activation sources require 40 ticks and gates".into());
        }
        Ok(())
    }
}

impl ContextTimedEvent {
    pub fn source_bytes(&self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..4].copy_from_slice(&self.tick.to_le_bytes());
        for (chunk, value) in bytes[4..8].chunks_exact_mut(2).zip([self.x, self.y]) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        for (chunk, value) in bytes[8..14].chunks_exact_mut(2).zip([
            self.argument_0, self.argument_1, self.argument_2,
        ]) { chunk.copy_from_slice(&value.to_le_bytes()); }
        bytes[14] = self.argument_3;
        bytes[15] = self.argument_4;
        bytes
    }
}

impl ContextTimeline {
    fn validate(&self) -> Result<(), String> {
        if self.records.len() != 5 || self.records.iter().any(|record|
            record.source_a != 0x08000000 + CONTEXT_TIMELINE_SOURCE_A
                || record.source_b != 0x08000000 + CONTEXT_TIMELINE_SOURCE_B)
        {
            return Err("Invalid context timeline records or source links".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayeredObjAnimations {
    pub descriptors: Vec<Descriptor>,
    pub banks: Vec<obj_animations::Bank>,
    /// These records are adjacent independent context data, not OBJ banks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_coordinate_records: Vec<ContextCoordinateRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_timeline: Option<ContextTimeline>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_timed_events: Option<Vec<ContextTimedEvent>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_language_positions: Option<ContextLanguagePositions>,
    /// Conditional title-menu cursor positions for five ordinary language choices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_language_positions: Option<[[u16; 2]; 5]>,
    /// Independently rooted title animation, without a layered descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_language_bank: Option<obj_animations::Bank>,
    /// Five language-specific sets of four main title cursor positions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_main_positions: Option<[[[u16; 2]; 4]; 5]>,
    /// A separate title animation selected through its own original producer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_main_bank: Option<obj_animations::Bank>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_activation: Option<ContextActivation>,
    /// Target geometry for a separate original interpolation path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_target_coordinate: Option<ContextCoordinateRecord>,
    /// Independent source rectangles; one is used both as a target and as a start.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_additional_geometry: Vec<ContextCoordinateRecord>,
}

impl LayeredObjAnimations {
    pub fn validate(&self) -> Result<(), String> {
        if self.descriptors.len() != BANK_SHAPES.len() || self.banks.len() != BANK_SHAPES.len() {
            return Err("Invalid layered OBJ descriptor/bank count".into());
        }
        if !self.context_coordinate_records.is_empty()
            && (self.context_coordinate_records.len() != CONTEXT_COORDINATE_OFFSETS.len()
                || self.context_coordinate_records.iter().zip(CONTEXT_COORDINATE_OFFSETS)
                    .any(|(record, offset)| record.source_offset != offset
                        || record.x_extent <= 0 || record.y_extent <= 0))
        {
            return Err("Invalid context coordinate records".into());
        }
        if let Some(timeline) = &self.context_timeline { timeline.validate()?; }
        if let Some(activation) = &self.context_activation { activation.validate()?; }
        if let Some(target) = &self.context_target_coordinate {
            if target.source_offset != CONTEXT_TARGET_COORDINATE || target.x_extent <= 0 || target.y_extent <= 0 {
                return Err("Invalid context target coordinate record".into());
            }
        }
        if !self.context_additional_geometry.is_empty() {
            let offsets = match self.context_additional_geometry.len() {
                6 => &CONTEXT_ADDITIONAL_GEOMETRY[2..], // E144 datasets
                8 => &CONTEXT_ADDITIONAL_GEOMETRY[..],
                _ => return Err("Invalid additional context geometry records".into()),
            };
            if self.context_additional_geometry.iter().zip(offsets)
                .any(|(record, &offset)| record.source_offset != offset) {
                return Err("Invalid additional context geometry records".into());
            }
        }
        if let Some(events) = &self.context_timed_events {
            if events.len() != 5 || events[..4].iter().any(|event| event.tick == 0) || events[4].tick != 0 {
                return Err("Invalid context timed event records or terminator".into());
            }
        }
        for ((descriptor, bank), &(objects, sequences, lists, frames)) in
            self.descriptors.iter().zip(&self.banks).zip(BANK_SHAPES)
        {
            bank.validate()?;
            if (descriptor.object_root, descriptor.sequence_root) != (objects, sequences)
                || (bank.object_root, bank.sequence_root) != (objects, sequences)
                || bank.lists.len() != lists
                || bank.sequences.len() != frames.len()
                || bank
                    .sequences
                    .iter()
                    .zip(frames)
                    .any(|(seq, &count)| seq.frames.len() != count || seq.zero_count_hold.is_some())
            {
                return Err("Invalid declared layered OBJ bank shape".into());
            }
        }
        if let Some(bank) = &self.title_language_bank {
            bank.validate()?;
            if (bank.object_root, bank.sequence_root) != (TITLE_OBJECT_ROOT, TITLE_SEQUENCE_ROOT)
                || bank.lists.len() != 15 || bank.sequences.len() != 2
                || bank.sequences.iter().map(|s| s.frames.len()).collect::<Vec<_>>() != [9, 8]
                || bank.lists.iter().any(|list| list.objects.iter().any(|object| object.shape >= 3)) {
                return Err("Invalid title language animation bank".into());
            }
        }
        if let Some(bank) = &self.title_main_bank {
            bank.validate()?;
            if (bank.object_root, bank.sequence_root) != (TITLE_MAIN_OBJECT_ROOT, TITLE_MAIN_SEQUENCE_ROOT)
                || bank.lists.len() != 6 || bank.sequences.len() != 1
                || bank.sequences[0].frames.len() != 6
                || bank.lists.iter().any(|list| list.objects.iter().any(|object| object.shape >= 3)) {
                return Err("Invalid main title animation bank".into());
            }
        }
        let spans = self.source_spans_unchecked()?;
        if spans
            .windows(2)
            .any(|v| u64::from(v[0].0) + v[0].1.len() as u64 > u64::from(v[1].0))
        {
            return Err("Overlapping layered OBJ source spans".into());
        }
        Ok(())
    }

    pub fn bank(&self, descriptor_index: u8) -> Option<&obj_animations::Bank> {
        self.banks.get(usize::from(descriptor_index))
    }

    fn source_spans_unchecked(&self) -> Result<Vec<SourceSpan>, String> {
        let mut descriptors = Vec::with_capacity(self.descriptors.len() * DESCRIPTOR_BYTES);
        for d in &self.descriptors {
            for root in [d.object_root, d.sequence_root] {
                descriptors.extend(
                    root.checked_add(0x08000000)
                        .ok_or("Layered OBJ descriptor pointer overflow")?
                        .to_le_bytes(),
                );
            }
            descriptors.extend(d.opaque);
        }
        let mut spans = vec![(
            DESCRIPTOR_ROOT,
            descriptors,
            "layered-obj-descriptors".into(),
        )];
        for record in &self.context_coordinate_records {
            spans.push((record.source_offset, record.source_bytes().to_vec(),
                "context-coordinate-record".into()));
        }
        if let Some(timeline) = &self.context_timeline {
            spans.push((CONTEXT_TIMELINE_ROOT,
                timeline.records.iter().flat_map(ContextTimelineRecord::source_bytes).collect(),
                "context-timeline-records".into()));
            for (offset, words, role) in [
                (CONTEXT_TIMELINE_SOURCE_A, &timeline.source_a, "context-timeline-source-a"),
                (CONTEXT_TIMELINE_SOURCE_B, &timeline.source_b, "context-timeline-source-b"),
            ] {
                spans.push((offset, words.iter().flat_map(|value| value.to_le_bytes()).collect(), role.into()));
            }
        }
        if let Some(events) = &self.context_timed_events {
            spans.push((CONTEXT_EVENT_ROOT,
                events.iter().flat_map(ContextTimedEvent::source_bytes).collect(),
                "context-timed-event-records".into()));
        }
        if let Some(positions) = &self.context_language_positions {
            for (offset, rows, role) in [
                (CONTEXT_LANGUAGE_POSITION_A, &positions.first, "context-language-position-a"),
                (CONTEXT_LANGUAGE_POSITION_B, &positions.second, "context-language-position-b"),
            ] {
                spans.push((offset, rows.iter().flat_map(ContextLanguagePosition::source_bytes).collect(), role.into()));
            }
        }
        if let Some(groups) = &self.title_main_positions {
            spans.push((TITLE_MAIN_POSITION_ROOT, groups.iter().flatten().flatten().flat_map(|value| value.to_le_bytes()).collect(), "title-main-positions".into()));
        }
        if let Some(rows) = &self.title_language_positions {
            spans.push((TITLE_LANGUAGE_POSITION_ROOT, rows.iter().flatten().flat_map(|value| value.to_le_bytes()).collect(), "title-language-positions".into()));
        }
        if let Some(activation) = &self.context_activation {
            spans.push((CONTEXT_ACTIVATION_GATES, activation.gates.clone(), "context-activation-gates".into()));
            spans.push((CONTEXT_ACTIVATION_TICKS, activation.ticks.iter().flat_map(|tick| tick.to_le_bytes()).collect(), "context-activation-ticks".into()));
        }
        if let Some(target) = &self.context_target_coordinate {
            spans.push((target.source_offset, target.source_bytes().to_vec(), "context-target-coordinate".into()));
        }
        for record in &self.context_additional_geometry {
            spans.push((record.source_offset, record.source_bytes().to_vec(), "context-additional-geometry".into()));
        }
        for bank in self.banks.iter().chain(self.title_language_bank.iter()).chain(self.title_main_bank.iter()) {
            for (at, offsets, role) in [
                (
                    bank.object_root,
                    &bank.object_offsets,
                    "layered-obj-list-index",
                ),
                (
                    bank.sequence_root,
                    &bank.sequence_offsets,
                    "layered-obj-sequence-index",
                ),
            ] {
                spans.push((
                    at,
                    offsets.iter().flat_map(|v| v.to_le_bytes()).collect(),
                    role.into(),
                ));
            }
            for list in &bank.lists {
                spans.push((
                    list.source_offset,
                    list.source_bytes()?,
                    "layered-obj-list".into(),
                ));
            }
            for sequence in &bank.sequences {
                spans.push((
                    sequence.source_offset,
                    sequence.source_bytes()?,
                    "layered-obj-sequence".into(),
                ));
            }
        }
        spans.sort_by_key(|s| s.0);
        Ok(spans)
    }

    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        self.source_spans_unchecked()
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.source_spans()
            .ok()?
            .into_iter()
            .find_map(|(at, raw, _)| raw.get(offset.checked_sub(at)? as usize).copied())
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Layered OBJ banks differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: LayeredObjAnimations,
    pub sources: Vec<SourceRange>,
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Layered OBJ banks require exact EUR ROM; USA pending".into());
        }
        for (start, end, hash) in [
            (
                0x141a24,
                0x141a88,
                "4739d54958cce203c5de97fd7bff36496b73b18a9d243119957a8f0894430b45",
            ),
            (
                0x142590,
                0x142664,
                "f4834f70593b895d7f69d6a8ddadc6e4a808a2b5ec52d9e29c02c5a777f793c6",
            ),
            (
                0x142664,
                0x142780,
                "ec00d9f806de57ab0b198f9ead2f3e6c62099ab4e8c0ab5aea1e3dcf230b27c9",
            ),
            (0x140964, 0x140a30, "12a4aa9ccdba5db361a3699cef29f0a622068cc1a82f8845c03fa73c1f0fdc34"),
            (0x143028, 0x143100, "6185add8794fd561de201be7534ef4690466bed61b62eed21a965742d3ee1cfc"),
            (0x145230, 0x145234, "07967ebdc3979cd4fd4de37fd22b3e6de8965b00dae4ac5585577c9709839652"),
            (0x140b18, 0x140b48, "8a2599f59ffdf1d09e934fbe6a3e2871dc58e64c847bc4014ec50519275fa484"),
            (0x142acc, 0x142b48, "2869bdf07dfdb606188c4777ac41967ba059d3c989893b6c12816dc70e05f87e"),
            (0x14352c, 0x143550, "9a22dc0aaf96f9c0dd2b60ffd5d7518db88ca232c107e35dd5914f21945400f3"),
            (0x141d86, 0x141da2, "f11ab1dcdf97ad868445f9ffa8e705c0d46528d0c9e38238abb567afc83bc266"),
            (0x14205a, 0x1420c0, "fca258e1b0151885137954deb000e55f9eaa49b27e174a36b892e20cbcbdaa87"),
            (0x13fe44, 0x13fe6e, "e14a6b5da42a4f195106717d4c67b50a7c4a47cae1dac403e74a4b31af27e598"),
            (0x13ff3c, 0x13ff88, "1029619d18a4d9ef5f79859ca793dd35865051ec9bcbb5bec66c3cc074052868"),
            (0x13ff88, 0x13ffcc, "3a969d1910cc0880282ad950aa00e5c10ae131e0f2b4a20c6319a6a4fe9f4d40"),
            (0x140e6c, 0x140eac, "64ed0eab01842d4010628836859f023fe78f17554bbf6e65305f6a9145059d53"),
            (0x140ef0, 0x140f20, "c82daf96ce31f1eee12a2d8017806d245c626166e30f150bcb8f4da25bc3ce1e"),
            (0x143238, 0x14325a, "e789647dad2e0f7ad302fd234a3ff9bcdc7e0d44fe08c385d720f8de79e26261"),
            (0x1421ee, 0x142234, "f120f01444924d6ac0efc323c3ec9715465732639242caa6d006f4de533332e4"),
            (0x1410b2, 0x1410ca, "d5d18ea4a2b07b287399a37a4ccce0f9a514aed60450e23395049e0a85445e90"),
            (0x141314, 0x14132c, "f2e765d26c9bc99abf19905e40952b67ad071903ace68bb5a43eb0b44c70a751"),
            (0x1414e0, 0x1414f8, "834314a410472302940724c3ac227ec1705294c45f1614170b2c33fb6059f6c7"),
            (0x1416d2, 0x1416e8, "599eeac3e1a42ee179f2b63b34c0e59936debb26755b3b973ddb67fda2099792"),
            (0x14177a, 0x141790, "d5656ed71fe3b6fef27e26258bad9a607466eb48704c2c17a1b4bd287e688f72"),
            (0x1431f4, 0x143238, "cda42034767c89fa21ab8f770ea9216c9472b1b79d02db0c9bccbbc64347f675"),
            (0x14325c, 0x14329c, "e82a4547a0b988614626a5c53d7575853a73de5e66beba39249592bbb5875a4b"),
            (0x14073a, 0x140756, "c72459af942ef733c90f92af033ef241c7e24ab3e36a8e7039c504629e0578f6"),
            (0x140c34, 0x140c40, "333cedd32c64c04a1a4e4f1c3ecbeff225b2da5e42656bce917a91c87b46b1ab"),
            (0x13aa5a, 0x13aa7e, "0f5e10cbe49182a13131a4e495f27c08fcee9b407bbda18444382563029abd58"),
            (0x13b20c, 0x13b228, "d18f507dd57cae950c192a0c0f821f0db96543df450f350f131bd2062f9394f5"),
            (0x139ca0, 0x139d28, "67733b8af31a06c532df977d73eeb3c8ef9c061863611aaee36194cf46655ff1"),
            (0x1500, 0x16f2, "0e5f0c56689302c705b57240819d430a96f18111a5abc548f2c321b220840f6e"),
            (0x13ba10, 0x13ba90, "91ca9d9af0d87555f09df9069bbbdc51195ed07244a225592971cbb07415721e"),
            (0x13a710, 0x13a7e6, "667e6f067cb6b24587b27d4d04a4e2d3338100cb18eb2d5105d2b6f24fa46603"),
            (0x13a21e, 0x13a23a, "048593c8b52a33967572118641842e88ab944b94dca33a28ad213f63b8f65238"),
        ] {
            if ffta_rom::digest(&rom[start..end]) != hash {
                return Err("Original layered OBJ producer/reader fingerprint mismatch".into());
            }
        }
        for &(call, index) in PRODUCERS {
            let half = |at| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            let (hi, lo) = (half(call), half(call + 2));
            let displacement =
                (((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
            if half(call - 2) != 0x2000 | u16::from(index)
                || hi & 0xf800 != 0xf000
                || lo & 0xf800 != 0xf800
                || (call as u32 + 4).wrapping_add_signed(displacement) != 0x141a24
            {
                return Err("Original layered OBJ producer call mismatch".into());
            }
        }
        for &(call, literal, offset) in &CONTEXT_COORDINATE_CALLS {
            let half = |at| u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            let (hi, lo) = (half(call), half(call + 2));
            let displacement =
                (((u32::from(hi & 2047) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
            let pointer = u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap());
            if hi & 0xf800 != 0xf000 || lo & 0xf800 != 0xf800
                || (call as u32 + 4).wrapping_add_signed(displacement) != 0x143454
                || pointer != 0x08000000 + offset
            {
                return Err("Original context coordinate source caller mismatch".into());
            }
        }
        if u32::from_le_bytes(rom[0x1409f4..0x1409f8].try_into().unwrap())
            != 0x08000000 + CONTEXT_TIMELINE_ROOT
        {
            return Err("Original context timeline table literal mismatch".into());
        }
        if u32::from_le_bytes(rom[0x140d48..0x140d4c].try_into().unwrap())
            != 0x08000000 + CONTEXT_EVENT_ROOT
        {
            return Err("Original context timed event table literal mismatch".into());
        }
        for (literal, root) in [(0x141de0, CONTEXT_LANGUAGE_POSITION_A),
                                (0x142118, CONTEXT_LANGUAGE_POSITION_A),
                                (0x142128, CONTEXT_LANGUAGE_POSITION_B),
                                (0x141208, CONTEXT_ACTIVATION_TICKS),
                                (0x141214, CONTEXT_ACTIVATION_GATES),
                                (0x141230, CONTEXT_TARGET_COORDINATE),
                                (0x1407d0, 0x3ae154), (0x140d70, 0x3ae15c),
                                (0x14126c, 0x3ae174), (0x14165c, 0x3ae17c),
                                (0x141698, 0x3ae184), (0x141850, 0x3ae18c),
                                (0x141854, 0x3ae194), (0x141864, 0x3ae19c)] {
            if u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) != 0x0800_0000 + root {
                return Err("Original context source literal mismatch".into());
            }
        }
        let mut descriptors = Vec::with_capacity(BANK_SHAPES.len());
        let mut banks = Vec::with_capacity(BANK_SHAPES.len());
        for (index, &(object_root, sequence_root, _, _)) in BANK_SHAPES.iter().enumerate() {
            let at = DESCRIPTOR_ROOT as usize + index * DESCRIPTOR_BYTES;
            let word =
                |offset| u32::from_le_bytes(rom[at + offset..at + offset + 4].try_into().unwrap());
            if word(0) != object_root + 0x08000000 || word(4) != sequence_root + 0x08000000 {
                return Err("Original layered OBJ descriptor roots mismatch".into());
            }
            descriptors.push(Descriptor {
                object_root,
                sequence_root,
                opaque: rom[at + 8..at + 28].try_into().unwrap(),
            });
            banks.push(obj_animations::Bank::decode(
                rom,
                object_root,
                sequence_root,
            )?);
        }
        let context_coordinate_records = CONTEXT_COORDINATE_OFFSETS.into_iter().map(|offset| {
            let at = offset as usize;
            let half = |index| i16::from_le_bytes(rom[at + index..at + index + 2].try_into().unwrap());
            ContextCoordinateRecord { source_offset: offset, x_base: half(0), y_base: half(2),
                x_extent: half(4), y_extent: half(6) }
        }).collect();
        let records = (0..5).map(|index| {
            let at = CONTEXT_TIMELINE_ROOT as usize + index * CONTEXT_TIMELINE_RECORD_BYTES;
            let word = |pos| u32::from_le_bytes(rom[at + pos..at + pos + 4].try_into().unwrap());
            ContextTimelineRecord { first_tick: word(0), second_tick: word(4),
                source_a: word(8), source_b: word(12), argument: rom[at + 16], mode: rom[at + 17],
                duration: u16::from_le_bytes(rom[at + 18..at + 20].try_into().unwrap()) }
        }).collect();
        let read_buffer = |offset: u32| std::array::from_fn(|index| {
            let at = offset as usize + index * 2;
            u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
        });
        let context_timeline = Some(ContextTimeline { records,
            source_a: read_buffer(CONTEXT_TIMELINE_SOURCE_A),
            source_b: read_buffer(CONTEXT_TIMELINE_SOURCE_B) });
        let context_timed_events = Some((0..5).map(|index| {
            let at = CONTEXT_EVENT_ROOT as usize + index * 16;
            let half = |pos| u16::from_le_bytes(rom[at + pos..at + pos + 2].try_into().unwrap());
            ContextTimedEvent {
                tick: u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()),
                x: half(4) as i16, y: half(6) as i16,
                argument_0: half(8), argument_1: half(10), argument_2: half(12),
                argument_3: rom[at + 14], argument_4: rom[at + 15],
            }
        }).collect());
        let language_rows = |offset: u32| std::array::from_fn(|index| {
            let at = offset as usize + 2 * index;
            ContextLanguagePosition { x: rom[at], y: rom[at + 1] }
        });
        let context_language_positions = Some(ContextLanguagePositions {
            first: language_rows(CONTEXT_LANGUAGE_POSITION_A),
            second: language_rows(CONTEXT_LANGUAGE_POSITION_B),
        });
        for (literal, expected) in [(0x13aaa8, 0x0800_0000 + TITLE_LANGUAGE_POSITION_ROOT),
            (0x13b2fc, 0x0800_0000 + TITLE_OBJECT_ROOT), (0x13b300, 0x0800_0000 + TITLE_SEQUENCE_ROOT)] {
            if u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) != expected {
                return Err("Title language source literal mismatch".into());
            }
        }
        let raw = &rom[TITLE_LANGUAGE_POSITION_ROOT as usize..TITLE_LANGUAGE_POSITION_ROOT as usize + 20];
        if ffta_rom::digest(raw) != "ffef7ade0828b2da33d4e71f9229241128a0a84919ed2cbedaac1c9a9ea39516" {
            return Err("Title language position source mismatch".into());
        }
        let title_language_positions = Some(std::array::from_fn(|index| std::array::from_fn(|axis| {
            let at = 4 * index + 2 * axis; u16::from_le_bytes([raw[at], raw[at + 1]])
        })));
        let title_bank = obj_animations::Bank::decode(rom, TITLE_OBJECT_ROOT, TITLE_SEQUENCE_ROOT)?;
        let mut title_spans = title_bank.spans(); title_spans.sort_by_key(|span| span.0);
        let mut title_bytes = Vec::new();
        for &(at, len, _) in &title_spans { title_bytes.extend_from_slice(&rom[at as usize..at as usize + len]); }
        if title_spans.len() != 19 || title_bytes.len() != 290
            || ffta_rom::digest(&title_bytes) != "a20bf3c20bee033386435098276481c16ca2dc5a6fa6c6f9fe43c47dabc62396" {
            return Err("Title animation source projection mismatch".into());
        }
        let title_language_bank = Some(title_bank);
        for (literal, expected) in [(0x13ba54, 0x0800_0000 + TITLE_MAIN_POSITION_ROOT),
            (0x13ba5c, 0x0800_0000 + TITLE_MAIN_POSITION_ROOT + 16),
            (0x13ba64, 0x0800_0000 + TITLE_MAIN_POSITION_ROOT + 32),
            (0x13ba6c, 0x0800_0000 + TITLE_MAIN_POSITION_ROOT + 48),
            (0x13ba8c, 0x0800_0000 + TITLE_MAIN_POSITION_ROOT + 64),
            (0x13a284, 0x0800_0000 + TITLE_MAIN_OBJECT_ROOT),
            (0x13a288, 0x0800_0000 + TITLE_MAIN_SEQUENCE_ROOT)] {
            if u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap()) != expected {
                return Err("Main title source literal mismatch".into());
            }
        }
        let raw = &rom[TITLE_MAIN_POSITION_ROOT as usize..TITLE_MAIN_POSITION_ROOT as usize + 80];
        if ffta_rom::digest(raw) != "7b0eb881e070b31ed8552874db7fa6d3b16aa635f500e39cafe22949e28f3092" {
            return Err("Main title position source mismatch".into());
        }
        let title_main_positions = Some(std::array::from_fn(|language| std::array::from_fn(|row| std::array::from_fn(|axis| {
            let at = 16 * language + 4 * row + 2 * axis; u16::from_le_bytes([raw[at], raw[at + 1]])
        }))));
        let main_bank = obj_animations::Bank::decode(rom, TITLE_MAIN_OBJECT_ROOT, TITLE_MAIN_SEQUENCE_ROOT)?;
        let mut main_spans = main_bank.spans(); main_spans.sort_by_key(|span| span.0);
        let mut main_bytes = Vec::new();
        for &(at, len, _) in &main_spans { main_bytes.extend_from_slice(&rom[at as usize..at as usize + len]); }
        if main_spans.len() != 9 || main_bytes.len() != 150
            || ffta_rom::digest(&main_bytes) != "e7c8f21cd00e94be51f9bb764f655046f5100736097b6a0cd4dfb50ab8dde1d8" {
            return Err("Main title animation source projection mismatch".into());
        }
        let title_main_bank = Some(main_bank);

        let context_activation = Some(ContextActivation {
            ticks: (0..40).map(|index| {
                let at = CONTEXT_ACTIVATION_TICKS as usize + 4 * index;
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            }).collect(),
            gates: rom[CONTEXT_ACTIVATION_GATES as usize..CONTEXT_ACTIVATION_GATES as usize + 40].to_vec(),
        });
        let target_at = CONTEXT_TARGET_COORDINATE as usize;
        let target_half = |index| i16::from_le_bytes(rom[target_at + index..target_at + index + 2].try_into().unwrap());
        let context_target_coordinate = Some(ContextCoordinateRecord {
            source_offset: CONTEXT_TARGET_COORDINATE, x_base: target_half(0), y_base: target_half(2),
            x_extent: target_half(4), y_extent: target_half(6),
        });
        let context_additional_geometry = CONTEXT_ADDITIONAL_GEOMETRY.into_iter().map(|offset| {
            let at = offset as usize;
            let half = |index| i16::from_le_bytes(rom[at + index..at + index + 2].try_into().unwrap());
            ContextCoordinateRecord { source_offset: offset, x_base: half(0), y_base: half(2),
                x_extent: half(4), y_extent: half(6) }
        }).collect();
        let decoded = LayeredObjAnimations { descriptors, banks, context_coordinate_records,
            context_timeline, context_timed_events, context_language_positions, title_language_positions, title_language_bank, title_main_positions, title_main_bank, context_activation, context_target_coordinate, context_additional_geometry };
        let sources = decoded
            .source_spans()?
            .into_iter()
            .map(|(offset, raw, role)| SourceRange {
                offset,
                length: raw.len() as u32,
                sha256: ffta_rom::digest(&raw),
                role,
            })
            .collect();
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests;
