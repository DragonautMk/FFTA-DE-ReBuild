//! Typed fields serve original guest readers; interpolation and mutable RAM stay original.
use super::*;
use crate::rom_data::blend_programs as model;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlendProgramStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub header_bytes: u64,
    pub command_bytes: u64,
    pub producer_bytes: u64,
    pub programs_used: u64,
    pub producer_tables_used: u64,
}
#[derive(Clone, Copy)]
enum Field {
    Header(u16),
    Command(u16, u16),
    Producer(u8),
}
struct Binding {
    offset: u32,
    length: usize,
    field: Field,
}
pub(super) struct BlendProgramResources {
    data: model::BlendPrograms,
    bindings: Vec<Binding>,
    counters: [AtomicU64; 5],
    programs: [AtomicU64; 3],
    tables: AtomicU64,
}
impl BlendProgramResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x39a..=0x39e | 0x3a0 | 0x3a2 | 0x3a7..=0x3a8 | 0x3ab..=0x3ac | 0xb4a..=0xb4b)
    }
    pub(super) fn new(data: model::BlendPrograms, sources: &[SourceRange]) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Blend source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Blend typed source binding mismatch".into());
            }
        }
        let mut bindings = Vec::new();
        for (id, program) in data.programs.iter().enumerate() {
            bindings.push(Binding {
                offset: program.source_offset,
                length: 6,
                field: Field::Header(id as u16),
            });
            for i in 0..program.commands.len() {
                bindings.push(Binding {
                    offset: program.source_offset + 6 + 4 * i as u32,
                    length: 4,
                    field: Field::Command(id as u16, i as u16),
                });
            }
        }
        for (i, producer) in data.producers.iter().enumerate() {
            bindings.push(Binding {
                offset: producer.source_offset,
                length: 4 * producer.programs.len(),
                field: Field::Producer(i as u8),
            });
        }
        bindings.sort_by_key(|binding| binding.offset);
        Ok(Self {
            data,
            bindings,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            programs: std::array::from_fn(|_| AtomicU64::new(0)),
            tables: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let i = self
            .bindings
            .partition_point(|binding| binding.offset as usize <= at)
            .checked_sub(1)?;
        let binding = &self.bindings[i];
        let offset = at - binding.offset as usize;
        if offset >= binding.length {
            return None;
        }
        let (byte, category, program) = match binding.field {
            Field::Header(id) => (
                self.data.programs[usize::from(id)].header.byte(offset)?,
                2,
                Some(id),
            ),
            Field::Command(id, index) => (
                self.data.programs[usize::from(id)].commands[usize::from(index)].bytes()[offset],
                3,
                Some(id),
            ),
            Field::Producer(table) => {
                self.tables.fetch_or(1 << table, Ordering::Relaxed);
                (
                    self.data.producer_byte(usize::from(table), offset)?,
                    4,
                    None,
                )
            }
        };
        if let Some(id) = program {
            self.programs[usize::from(id) / 64].fetch_or(1 << (id % 64), Ordering::Relaxed);
        }
        Some((byte, category))
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
        let mut counts = [1, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((byte, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                byte
            } else {
                prior
                    .as_ref()
                    .map_or_else(|| rom[index + i], |bytes| bytes[i])
            }
        });
        for (counter, count) in self.counters.iter().zip(counts) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> BlendProgramStats {
        let [bus_reads, source_bytes, header_bytes, command_bytes, producer_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        BlendProgramStats {
            bus_reads,
            source_bytes,
            header_bytes,
            command_bytes,
            producer_bytes,
            programs_used: self
                .programs
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
            producer_tables_used: u64::from(self.tables.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.bindings.capacity() * std::mem::size_of::<Binding>()
            + self.data.programs.capacity() * std::mem::size_of::<model::Program>()
            + self
                .data
                .programs
                .iter()
                .map(|p| p.commands.capacity() * std::mem::size_of::<model::Command>())
                .sum::<usize>()
            + self.data.producers.capacity() * std::mem::size_of::<model::Producer>()
            + self
                .data
                .producers
                .iter()
                .map(|p| p.programs.capacity() * std::mem::size_of::<u16>())
                .sum::<usize>()
    }
}
