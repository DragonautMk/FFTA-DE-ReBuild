//! Caller-witnessed EUR compact OBJ animation projection.
//! The original guest owns selector arithmetic and all out-of-projection reads.
use super::{obj_lists, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const ROOT: u32 = 0x6b9b80;
pub const LIST_ROOT: u32 = ROOT + 4;
pub const SEQUENCE_ROOT: u32 = 0x6ba300;
pub const SELECTORS: [u16; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 9, 10];
pub const SPANS: [(u32, u32); 5] = [
    (0x6b9b80, 0x6b9bfe), (0x6b9c12, 0x6ba1fc),
    (0x6ba300, 0x6ba310), (0x6ba312, 0x6ba43a),
    (0x6ba458, 0x6ba4a4),
];
pub type SourceSpan = (u32, Vec<u8>, String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub list: u8,
    pub duration: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub selector: u16,
    pub source_offset: u32,
    pub frames: Vec<Frame>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactAnimations {
    pub list_offsets: Vec<u16>,
    pub lists: Vec<obj_lists::List>,
    pub sequence_offsets: Vec<u16>,
    pub sequences: Vec<Sequence>,
}

impl CompactAnimations {
    pub fn validate(&self) -> Result<(), String> {
        if self.list_offsets.len() != 61 || self.lists.len() != 61
            || self.sequence_offsets.len() != 10 || self.sequences.len() != 10 {
            return Err("Invalid compact OBJ projection shape".into());
        }
        let mut next = SPANS[1].0;
        let mut objects = 0usize;
        for (list, offset) in self.lists.iter().zip(&self.list_offsets) {
            list.validate()?;
            if list.source_offset != next || LIST_ROOT + u32::from(*offset) != next {
                return Err("Invalid compact OBJ list directory or extent".into());
            }
            next = next.checked_add(list.source_len() as u32).ok_or("OBJ list overflow")?;
            objects += list.objects.len();
        }
        if next != SPANS[1].1 || objects != 232 { return Err("Invalid compact OBJ lists".into()); }
        let mut next = SPANS[3].0 + 4; // two witnessed directory entries precede the records
        let mut frames = 0usize;
        for (i, ((sequence, offset), selector)) in self.sequences.iter()
            .zip(&self.sequence_offsets).zip(SELECTORS).enumerate() {
            if i == 8 { next = SPANS[4].0; }
            if sequence.selector != selector || sequence.source_offset != next
                || SEQUENCE_ROOT + u32::from(*offset) != next
                || sequence.frames.len() > u16::MAX as usize
                || sequence.frames.iter().any(|frame| frame.list >= 61) {
                return Err("Invalid compact OBJ sequence directory or frame".into());
            }
            next = next.checked_add(2 + sequence.frames.len() as u32 * 2).ok_or("OBJ sequence overflow")?;
            if i == 7 && next != SPANS[3].1 { return Err("Invalid compact OBJ sequence gap".into()); }
            frames += sequence.frames.len();
        }
        if next != SPANS[4].1 || frames != 174 { return Err("Invalid compact OBJ sequences".into()); }
        Ok(())
    }

    pub fn sequence(&self, selector: u16) -> Option<&Sequence> {
        self.sequences.iter().find(|sequence| sequence.selector == selector)
    }
    pub fn list(&self, selector: u16) -> Option<&obj_lists::List> {
        self.lists.get(usize::from(selector))
    }
    /// Returns only owned bytes; a missing value does not restrict guest reads.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.source_spans().ok()?.into_iter().find_map(|(start, bytes, _)| {
            bytes.get(offset.checked_sub(start)? as usize).copied()
        })
    }
    pub fn source_spans(&self) -> Result<Vec<SourceSpan>, String> {
        self.validate()?;
        let mut head = vec![12, 0, 0x80, 7];
        head.extend(self.list_offsets.iter().flat_map(|offset| offset.to_le_bytes()));
        let mut lists = Vec::new();
        for list in &self.lists { lists.extend(list.source_bytes()?); }
        let offsets = |range: std::ops::Range<usize>| -> Vec<u8> {
            self.sequence_offsets[range].iter().flat_map(|offset| offset.to_le_bytes()).collect()
        };
        let sequences = |range: std::ops::Range<usize>| -> Vec<u8> {
            let mut out = Vec::new();
            for sequence in &self.sequences[range] {
                out.extend((sequence.frames.len() as u16).to_le_bytes());
                for frame in &sequence.frames { out.extend([frame.list, frame.duration]); }
            }
            out
        };
        let mut middle = offsets(8..10);
        middle.extend(sequences(0..8));
        let bytes = [head, lists, offsets(0..8), middle, sequences(8..10)];
        SPANS.into_iter().zip(bytes).map(|((start, end), raw)| {
            if raw.len() != (end - start) as usize { return Err("Invalid compact OBJ span length".into()); }
            Ok((start, raw, "conditional-compact-obj-animations".into()))
        }).collect()
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Compact OBJ projection differs from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog { pub decoded: CompactAnimations, pub sources: Vec<SourceRange> }
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Compact OBJ projection requires exact EUR ROM; USA pending".into());
        }
        let half = |at: u32| u16::from_le_bytes([rom[at as usize], rom[at as usize + 1]]);
        if half(ROOT) != 12 || half(ROOT + 2) != 0x780 { return Err("Invalid compact OBJ header".into()); }
        let list_offsets: Vec<_> = (0..61).map(|i| half(LIST_ROOT + i * 2)).collect();
        let mut lists = Vec::new();
        for offset in &list_offsets {
            let at = LIST_ROOT + u32::from(*offset);
            let len = 2 + usize::from(half(at)) * 6;
            let bytes = rom.get(at as usize..at as usize + len).ok_or("Truncated compact OBJ list")?;
            lists.push(obj_lists::List::decode(at, bytes)?);
        }
        let sequence_offsets: Vec<_> = SELECTORS.iter().map(|i| half(SEQUENCE_ROOT + u32::from(*i) * 2)).collect();
        let mut sequences = Vec::new();
        for (selector, offset) in SELECTORS.into_iter().zip(&sequence_offsets) {
            let at = SEQUENCE_ROOT + u32::from(*offset);
            let count = usize::from(half(at));
            let bytes = rom.get(at as usize + 2..at as usize + 2 + count * 2).ok_or("Truncated compact OBJ sequence")?;
            sequences.push(Sequence { selector, source_offset: at,
                frames: bytes.chunks_exact(2).map(|v| Frame {list: v[0], duration: v[1]}).collect() });
        }
        let decoded = CompactAnimations { list_offsets, lists, sequence_offsets, sequences };
        let sources = decoded.source_spans()?.into_iter().map(|(offset, raw, role)| SourceRange {
            offset, length: raw.len() as u32, sha256: ffta_rom::digest(&raw), role,
        }).collect();
        Ok(Self { decoded, sources })
    }
}
