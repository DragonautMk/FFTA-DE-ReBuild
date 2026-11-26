//! European byte-copy payloads, argument words and producer literals.

use super::stack_byte_copies::{verify_copy_helper, COPY_HELPER};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const ROM_BYTES: usize = 0x0100_0000;
pub const PAYLOAD_ROLE: &str = "fixed-byte-copy-data";
pub const LITERAL_ROLE: &str = "fixed-byte-copy-root";
pub const DIRECT_PAYLOAD_ROLE: &str = "direct-byte-copy-data";
pub const DIRECT_LITERAL_ROLE: &str = "direct-byte-copy-argument";
pub const PRODUCER_LITERAL_ROLE: &str = "byte-copy-producer-literal";
pub const SHARED_SLOT_LITERAL_ROLE: &str = "byte-copy-shared-slot-literal";

// E568 projected against e341. Twenty-one other verified calls already have
// owners for both their payloads and literals.
const CALLERS: [u32; 99] = [
    0x008d1e, 0x00970a, 0x00982e, 0x00cdc6, 0x00cdd2, 0x00cf24, 0x00cf30, 0x01550a, 0x023fea,
    0x045cb8, 0x080684, 0x0db028, 0x0db036, 0x0df0d4, 0x0df0e0, 0x0e1e64, 0x0e2532, 0x0e2d3e,
    0x0e2d4c, 0x0e2f50, 0x0e4fc6, 0x0e4fd4, 0x0e50ca, 0x0e50d8, 0x0e6cdc, 0x0e6ce8, 0x0ea790,
    0x0f20de, 0x0f34b0, 0x0f7dcc, 0x0f87f8, 0x0f880c, 0x0f8818, 0x0f927a, 0x0f9286, 0x0fa7de,
    0x0fa7f2, 0x0fa7fe, 0x0fbba4, 0x0fc9d4, 0x0fc9e4, 0x0fd9b2, 0x0fe9c4, 0x0fecc6, 0x0fecd6,
    0x100116, 0x100122, 0x10052a, 0x100ca2, 0x100cae, 0x101cb4, 0x102814, 0x10563c, 0x10564a,
    0x106dda, 0x106de6, 0x106df4, 0x106e00, 0x107312, 0x10731e, 0x107508, 0x10759e, 0x1075aa,
    0x1075b8, 0x1075c8, 0x107a2c, 0x107a7c, 0x108eae, 0x109096, 0x10b0bc, 0x10b1c8, 0x10b98c,
    0x10ba98, 0x10c318, 0x10f78e, 0x10f79a, 0x10f7a8, 0x10f7b4, 0x10fab0, 0x111346, 0x111352,
    0x111360, 0x11136c, 0x11166c, 0x119caa, 0x119cb8, 0x11ad36, 0x11ad42, 0x11cf86, 0x11f16c,
    0x11f178, 0x11f69e, 0x11f6aa, 0x1205c0, 0x120b86, 0x121dd0, 0x1221c2, 0x1221ce, 0x1221da,
];

// E571 source/count readers whose literal words have no e342 owner.
const DIRECT_ARGUMENT_READERS: [u32; 52] = [
    0x0088a8, 0x008d6e, 0x0326de, 0x0326fc, 0x032700, 0x057be4, 0x057cd6, 0x059080, 0x059172,
    0x05cb24, 0x05cc0e, 0x062efe, 0x06772e, 0x06782a, 0x072966, 0x072ae4, 0x07d98e, 0x07da9a,
    0x07dae4, 0x07f128, 0x07f130, 0x07f250, 0x07f26a, 0x07f3e0, 0x07f3f6, 0x07fb64, 0x07fb6c,
    0x07fc48, 0x07fc64, 0x07fd9c, 0x07fdb4, 0x08025e, 0x08027a, 0x0830b2, 0x08325c, 0x083260,
    0x088462, 0x08aa62, 0x090892, 0x0908da, 0x090960, 0x090ffa, 0x09105c, 0x091a0a, 0x091a52,
    0x091aba, 0x0f87e4, 0x0fa7c8, 0x1299d2, 0x129b94, 0x12fa2c, 0x12ff6c,
];
const DIRECT_PAYLOAD_READERS: [u32; 2] = [0x0f87e4, 0x0fa7c8];
const DIRECT_PAYLOAD_BYTES: usize = 4;

// E574 projected against e343. Two of these loads share a literal word.
const PRODUCER_LITERAL_READERS: [u32; 85] = [
    0x0068f4, 0x00694a, 0x00888e, 0x0089bc, 0x008d16, 0x024be6, 0x03203c, 0x03203e, 0x032490,
    0x03249e, 0x057bec, 0x057bf0, 0x057bf4, 0x057c04, 0x057c94, 0x059088, 0x05908c, 0x059090,
    0x0590a0, 0x059130, 0x05cb2e, 0x05cb3c, 0x05cbcc, 0x062f06, 0x062fa8, 0x06754e, 0x072930,
    0x072bf8, 0x072bfc, 0x072c00, 0x072fba, 0x072fbe, 0x073234, 0x073238, 0x07337c, 0x073380,
    0x0735ba, 0x0735cc, 0x0754a8, 0x0754ac, 0x0754ba, 0x075dee, 0x075df8, 0x078f2e, 0x078f32,
    0x078f3e, 0x079008, 0x07900c, 0x079010, 0x07901c, 0x07a348, 0x07a34c, 0x07a350, 0x07a35c,
    0x07a5f2, 0x07a5f6, 0x07a602, 0x07b27c, 0x07b280, 0x07b284, 0x07b290, 0x07b4bc, 0x07b4c0,
    0x07b4cc, 0x07d8ac, 0x07de32, 0x07de36, 0x07de42, 0x07de4a, 0x080d34, 0x080d38, 0x081ad2,
    0x081ad6, 0x081ada, 0x082c92, 0x082c96, 0x08bfe4, 0x08cd84, 0x08cd88, 0x090778, 0x090fc4,
    0x091930, 0x0d84bc, 0x125e3c, 0x14c6ea,
];

// The E577 production and backing-slot reads of words unowned in e344.
const SHARED_SLOT_LITERAL_READERS: [u32; 27] = [
    0x027096, 0x0298da, 0x02a738, 0x052f74, 0x052f76, 0x0564d8, 0x0564da, 0x05a908, 0x05a90a,
    0x05cb1c, 0x05cb20, 0x05cb32, 0x05cb4e, 0x061636, 0x06163a, 0x06163c, 0x06164e, 0x06167a,
    0x062ef4, 0x062f28, 0x06af44, 0x06af46, 0x125dd8, 0x125dde, 0x125de0, 0x125de6, 0x125e1a,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
    FixedSize,
    WithDirectArguments,
    WithProducerLiterals,
    WithSharedSlotLiterals,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CopyValue {
    Payload(Vec<u8>),
    Literal(u32),
}

impl CopyValue {
    fn len(&self) -> usize {
        match self {
            Self::Payload(bytes) => bytes.len(),
            Self::Literal(_) => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CopySpan {
    offset: u32,
    value: CopyValue,
}

impl CopySpan {
    fn source_span(
        &self,
        payload_role: &'static str,
        literal_role: &'static str,
    ) -> (u32, Vec<u8>, &'static str) {
        match &self.value {
            CopyValue::Payload(bytes) => (self.offset, bytes.clone(), payload_role),
            CopyValue::Literal(value) => (self.offset, value.to_le_bytes().to_vec(), literal_role),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedByteCopies {
    spans: Vec<CopySpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    direct_arguments: Vec<CopySpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    producer_literals: Vec<CopySpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    shared_slot_literals: Vec<CopySpan>,
}

fn literal_at(rom: &[u8], reader: u32, instruction: u16) -> Option<(u32, u32)> {
    if instruction & 0xf800 != 0x4800 {
        return None;
    }
    let literal = ((reader + 4) & !3) + 4 * u32::from(instruction & 0xff);
    let at = literal as usize;
    let value = u32::from_le_bytes(rom.get(at..at + 4)?.try_into().ok()?);
    Some((literal, value))
}

fn payload_at(rom: &[u8], address: u32, length: usize) -> Option<(u32, CopyValue)> {
    if !(0x08..=0x0d).contains(&(address >> 24)) {
        return None;
    }
    let source = address & 0x01ff_ffff;
    let start = source as usize;
    let payload = rom.get(start..start + length)?.to_vec();
    Some((source, CopyValue::Payload(payload)))
}

fn copy_at(rom: &[u8], call: u32) -> Option<[(u32, CopyValue); 2]> {
    let half = |at: u32| {
        let at = at as usize;
        u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
    };
    let high = half(call);
    let low = half(call + 2);
    if high & 0xf800 != 0xf000 || low & 0xf800 != 0xf800 {
        return None;
    }
    let displacement = ((u32::from(high & 0x7ff) << 12) | (u32::from(low & 0x7ff) << 1)) as i32;
    let displacement = (displacement << 9) >> 9;
    if (call + 4).wrapping_add_signed(displacement) != COPY_HELPER {
        return None;
    }
    let count = half(call - 2);
    let length = usize::from(count & 0xff);
    if count >> 8 != 0x22 || length == 0 {
        return None;
    }

    let mut reader = call - 4;
    let opcode = half(reader);
    // The optional r0 setup preserves the source and size registers.
    if opcode & 0xffc7 == 0x1c00 || opcode & 0xff87 == 0x4600 || opcode & 0xff00 == 0xa800 {
        reader -= 2;
    }
    let load = half(reader);
    if load >> 8 != 0x49 {
        return None;
    }
    let (literal, address) = literal_at(rom, reader, load)?;
    Some([
        payload_at(rom, address, length)?,
        (literal, CopyValue::Literal(address)),
    ])
}

fn direct_arguments_from_rom(rom: &[u8]) -> Result<Vec<CopySpan>, String> {
    let mut spans =
        Vec::with_capacity(DIRECT_ARGUMENT_READERS.len() + DIRECT_PAYLOAD_READERS.len());
    for reader in DIRECT_ARGUMENT_READERS {
        let at = reader as usize;
        let instruction = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let (offset, address) = literal_at(rom, reader, instruction)
            .ok_or_else(|| format!("Direct byte copy argument reader differs at {reader:06x}"))?;
        spans.push(CopySpan {
            offset,
            value: CopyValue::Literal(address),
        });
        if DIRECT_PAYLOAD_READERS.contains(&reader) {
            let (offset, value) = payload_at(rom, address, DIRECT_PAYLOAD_BYTES)
                .ok_or_else(|| format!("Invalid direct byte copy payload at {reader:06x}"))?;
            spans.push(CopySpan { offset, value });
        }
    }
    spans.sort_unstable_by_key(|span| span.offset);
    Ok(spans)
}

fn producer_literals_from_rom(rom: &[u8], readers: &[u32]) -> Result<Vec<CopySpan>, String> {
    let mut words = BTreeMap::new();
    for &reader in readers {
        let at = reader as usize;
        let instruction = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let (offset, value) = literal_at(rom, reader, instruction)
            .ok_or_else(|| format!("Byte copy producer reader differs at {reader:06x}"))?;
        words.insert(offset, value);
    }
    Ok(words
        .into_iter()
        .map(|(offset, value)| CopySpan {
            offset,
            value: CopyValue::Literal(value),
        })
        .collect())
}

fn span_before(spans: &[CopySpan], end: usize) -> Option<&CopySpan> {
    let count = spans.partition_point(|span| (span.offset as usize) < end);
    count.checked_sub(1).map(|index| &spans[index])
}

fn bank_overlaps(spans: &[CopySpan], start: usize, length: usize) -> bool {
    length > 0
        && span_before(spans, start.saturating_add(length))
            .is_some_and(|span| start < span.offset as usize + span.value.len())
}

fn validate_bank(spans: &[CopySpan], expected: (usize, usize, usize)) -> Result<(), String> {
    let mut previous_end = 0;
    let mut payloads = 0;
    let mut payload_bytes = 0;
    let mut literals = 0;
    for span in spans {
        let start = span.offset as usize;
        let length = span.value.len();
        if start < previous_end || length == 0 || length > ROM_BYTES.saturating_sub(start) {
            return Err("Invalid fixed byte copy span or order".into());
        }
        previous_end = start + length;
        match span.value {
            CopyValue::Payload(_) => {
                payloads += 1;
                payload_bytes += length;
            }
            CopyValue::Literal(_) if start % 4 == 0 => literals += 1,
            CopyValue::Literal(_) => return Err("Unaligned fixed byte copy literal".into()),
        }
    }
    if (payloads, payload_bytes, literals) != expected {
        return Err("Incomplete fixed byte copy bank".into());
    }
    Ok(())
}

impl FixedByteCopies {
    pub(super) fn from_rom(rom: &[u8], layout: Layout) -> Result<Self, String> {
        verify_copy_helper(rom)?;
        let mut spans = BTreeMap::new();
        for call in CALLERS {
            let pair = copy_at(rom, call)
                .ok_or_else(|| format!("Fixed byte copy reader differs at {call:06x}"))?;
            for (offset, value) in pair {
                if let Some(previous) = spans.get(&offset) {
                    if previous != &value {
                        return Err(format!("Conflicting fixed byte copy views at {offset:06x}"));
                    }
                } else {
                    spans.insert(offset, value);
                }
            }
        }
        let data = Self {
            spans: spans
                .into_iter()
                .map(|(offset, value)| CopySpan { offset, value })
                .collect(),
            direct_arguments: if layout == Layout::FixedSize {
                Vec::new()
            } else {
                direct_arguments_from_rom(rom)?
            },
            producer_literals: if matches!(
                layout,
                Layout::WithProducerLiterals | Layout::WithSharedSlotLiterals
            ) {
                producer_literals_from_rom(rom, &PRODUCER_LITERAL_READERS)?
            } else {
                Vec::new()
            },
            shared_slot_literals: if layout == Layout::WithSharedSlotLiterals {
                producer_literals_from_rom(rom, &SHARED_SLOT_LITERAL_READERS)?
            } else {
                Vec::new()
            },
        };
        data.validate()?;
        Ok(data)
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        validate_bank(&self.spans, (91, 526, 97))?;
        if !self.direct_arguments.is_empty() {
            validate_bank(&self.direct_arguments, (2, 8, 52))?;
            if self.direct_arguments.iter().any(|span| {
                span.value.len() != DIRECT_PAYLOAD_BYTES
                    || bank_overlaps(&self.spans, span.offset as usize, span.value.len())
            }) {
                return Err("Invalid or overlapping direct byte copy spans".into());
            }
        }
        if !self.producer_literals.is_empty() {
            validate_bank(&self.producer_literals, (0, 0, 84))?;
            if self.direct_arguments.is_empty()
                || self.producer_literals.iter().any(|span| {
                    bank_overlaps(&self.spans, span.offset as usize, 4)
                        || bank_overlaps(&self.direct_arguments, span.offset as usize, 4)
                })
            {
                return Err("Missing arguments or overlapping byte copy producer literals".into());
            }
        }
        if !self.shared_slot_literals.is_empty() {
            validate_bank(
                &self.shared_slot_literals,
                (0, 0, SHARED_SLOT_LITERAL_READERS.len()),
            )?;
            if self.producer_literals.is_empty()
                || self.shared_slot_literals.iter().any(|span| {
                    let at = span.offset as usize;
                    bank_overlaps(&self.spans, at, 4)
                        || bank_overlaps(&self.direct_arguments, at, 4)
                        || bank_overlaps(&self.producer_literals, at, 4)
                })
            {
                return Err(
                    "Missing producer words or overlapping byte copy shared slot literals".into(),
                );
            }
        }
        Ok(())
    }

    pub(super) fn layout(&self) -> Layout {
        if !self.shared_slot_literals.is_empty() {
            Layout::WithSharedSlotLiterals
        } else if !self.producer_literals.is_empty() {
            Layout::WithProducerLiterals
        } else if self.direct_arguments.is_empty() {
            Layout::FixedSize
        } else {
            Layout::WithDirectArguments
        }
    }

    pub fn source_spans(&self) -> impl Iterator<Item = (u32, Vec<u8>, &'static str)> + '_ {
        self.spans
            .iter()
            .map(|span| span.source_span(PAYLOAD_ROLE, LITERAL_ROLE))
            .chain(
                self.direct_arguments
                    .iter()
                    .map(|span| span.source_span(DIRECT_PAYLOAD_ROLE, DIRECT_LITERAL_ROLE)),
            )
            .chain(
                self.producer_literals
                    .iter()
                    .map(|span| span.source_span(PRODUCER_LITERAL_ROLE, PRODUCER_LITERAL_ROLE)),
            )
            .chain(
                self.shared_slot_literals.iter().map(|span| {
                    span.source_span(SHARED_SLOT_LITERAL_ROLE, SHARED_SLOT_LITERAL_ROLE)
                }),
            )
    }

    pub(crate) fn overlaps(&self, start: usize, length: usize) -> bool {
        bank_overlaps(&self.spans, start, length)
            || bank_overlaps(&self.direct_arguments, start, length)
            || bank_overlaps(&self.producer_literals, start, length)
            || bank_overlaps(&self.shared_slot_literals, start, length)
    }

    pub(super) fn byte_at(&self, offset: u32) -> Option<(u8, bool)> {
        [
            self.spans.as_slice(),
            self.direct_arguments.as_slice(),
            self.producer_literals.as_slice(),
            self.shared_slot_literals.as_slice(),
        ]
        .into_iter()
        .find_map(|spans| {
            let span = span_before(spans, (offset as usize).saturating_add(1))?;
            let index = (offset - span.offset) as usize;
            match &span.value {
                CopyValue::Payload(bytes) => bytes.get(index).map(|&byte| (byte, false)),
                CopyValue::Literal(value) => {
                    value.to_le_bytes().get(index).map(|&byte| (byte, true))
                }
            }
        })
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        (self.spans.capacity()
            + self.direct_arguments.capacity()
            + self.producer_literals.capacity()
            + self.shared_slot_literals.capacity())
            * std::mem::size_of::<CopySpan>()
            + self
                .spans
                .iter()
                .chain(&self.direct_arguments)
                .map(|span| match &span.value {
                    CopyValue::Payload(bytes) => bytes.capacity(),
                    CopyValue::Literal(_) => 0,
                })
                .sum::<usize>()
    }
}
