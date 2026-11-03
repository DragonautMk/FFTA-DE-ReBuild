//! Shared typed text byte mapping. Encoded controls stay metadata; literals read decoded units.
use super::super::{
    custom_lz,
    text_catalogs::{Program, ProgramUnit, MAX_PROGRAM_BYTES},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum ProgramKind {
    Header,
    Control,
    Glyph,
    Command,
    Operand,
    Terminator,
    Tail,
}
pub(super) struct TextProgramMap {
    offsets: Vec<usize>,
    logical_len: usize,
    pub(super) literals: Vec<custom_lz::LiteralSpan>,
}
impl TextProgramMap {
    pub(super) fn new(p: &Program) -> Result<Self, String> {
        p.validate()?;
        let compact = p.header & 1 != 0;
        let mut at = 0;
        let offsets = p
            .units
            .iter()
            .map(|unit| {
                let start = at;
                at += match unit {
                    ProgramUnit::Glyph { .. } => {
                        if compact {
                            1
                        } else {
                            2
                        }
                    }
                    ProgramUnit::Command { operands, .. } => {
                        if compact {
                            1 + operands.len()
                        } else {
                            2 + operands.len()
                        }
                    }
                };
                start
            })
            .collect();
        let literals = p
            .encoded
            .as_ref()
            .map(|bytes| {
                custom_lz::decode(bytes, MAX_PROGRAM_BYTES, MAX_PROGRAM_BYTES)
                    .map(|b| b.literal_spans)
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            offsets,
            logical_len: at + 1,
            literals,
        })
    }
    pub(super) fn source_len(&self, p: &Program) -> usize {
        2 + p.encoded.as_ref().map_or(self.logical_len, Vec::len)
    }
    fn logical_byte(&self, p: &Program, at: usize) -> Option<(u8, ProgramKind)> {
        let compact = p.header & 1 != 0;
        if at >= self.logical_len {
            return p
                .decoded_padding
                .get(at - self.logical_len)
                .map(|&v| (v, ProgramKind::Tail));
        }
        if at == self.logical_len - 1 {
            return Some((if compact { 255 } else { 0 }, ProgramKind::Terminator));
        }
        let id = self.offsets.partition_point(|&n| n <= at).checked_sub(1)?;
        let at = at - self.offsets[id];
        match &p.units[id] {
            ProgramUnit::Glyph { index } => Some((
                if compact {
                    *index as u8
                } else {
                    (*index | 0x8000).to_be_bytes()[at]
                },
                ProgramKind::Glyph,
            )),
            ProgramUnit::Command { opcode, operands } => {
                if compact {
                    if at == 0 {
                        Some((*opcode, ProgramKind::Command))
                    } else {
                        operands.get(at - 1).map(|&v| (v, ProgramKind::Operand))
                    }
                } else {
                    match at {
                        0 => Some((64, ProgramKind::Command)),
                        1 => Some((*opcode, ProgramKind::Command)),
                        _ => operands.get(at - 2).map(|&v| (v, ProgramKind::Operand)),
                    }
                }
            }
        }
    }
    pub(super) fn source_byte(&self, p: &Program, at: usize) -> Option<(u8, ProgramKind)> {
        if at >= self.source_len(p) {
            return None;
        }
        if at < 2 {
            return Some((p.header.to_le_bytes()[at], ProgramKind::Header));
        }
        let at = at - 2;
        if let Some(encoded) = &p.encoded {
            let literal = self
                .literals
                .partition_point(|s| s.encoded_offset <= at)
                .checked_sub(1)
                .and_then(|i| self.literals.get(i));
            if let Some(span) = literal.filter(|s| at < s.encoded_offset + s.length) {
                self.logical_byte(p, span.output_offset + at - span.encoded_offset)
            } else {
                encoded.get(at).map(|&v| (v, ProgramKind::Control))
            }
        } else {
            self.logical_byte(p, at)
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        self.offsets.capacity() * std::mem::size_of::<usize>()
            + self.literals.capacity() * std::mem::size_of::<custom_lz::LiteralSpan>()
    }
}
