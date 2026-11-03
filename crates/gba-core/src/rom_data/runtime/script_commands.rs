//! File-derived EUR command fields; original stream and callback control stay live.
use super::super::{script_commands as model, SourceRange};
use crate::ffta_rom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptCommandStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub advance_bytes: u64,
    pub callback_bytes: u64,
    pub commands_used: u64,
}
pub(super) struct ScriptCommandResources {
    data: model::ScriptCommands,
    counters: [AtomicU64; 4],
    commands: [AtomicU64; 2],
}
impl ScriptCommandResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x3ac | 0x3ad)
    }
    pub(super) fn new(
        data: model::ScriptCommands,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Script command binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Script command typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            commands: std::array::from_fn(|_| AtomicU64::new(0)),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let relative = at.checked_sub(self.data.source_offset as usize)?;
        let command = self.data.commands.get(relative / 6)?;
        let byte = command.byte(relative % 6)?;
        self.commands[relative / 6 / 64].fetch_or(1 << ((relative / 6) % 64), Ordering::Relaxed);
        Some((byte, if relative % 6 < 2 { 2 } else { 3 }))
    }
    pub(super) fn guest_read<const N: usize>(
        &self,
        index: usize,
        rom: &[u8],
        prior: Option<[u8; N]>,
    ) -> Option<[u8; N]> {
        let fields: [_; N] = std::array::from_fn(|i| self.byte(index + i));
        if fields.iter().all(Option::is_none) {
            return prior;
        }
        let mut counts = [1, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                byte
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> ScriptCommandStats {
        let [bus_reads, source_bytes, advance_bytes, callback_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        ScriptCommandStats {
            bus_reads,
            source_bytes,
            advance_bytes,
            callback_bytes,
            commands_used: self
                .commands
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.commands.capacity() * std::mem::size_of::<model::Command>()
    }
}
