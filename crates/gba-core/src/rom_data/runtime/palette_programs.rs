//! Typed source fields feed original CPU/DMA/native reads; no guest sequencing is replaced.
use super::*;
use crate::rom_data::palette_programs as model;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteProgramStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub header_bytes: u64,
    pub command_bytes: u64,
    pub parameter_bytes: u64,
    pub producer_bytes: u64,
    pub programs_used: u64,
    pub producer_tables_used: u64,
}
#[derive(Clone, Copy)]
enum Field {
    Header(u16),
    Command(u16, u8),
    Parameter(u16, u8),
    Producer(u8),
}
struct Binding {
    offset: u32,
    length: usize,
    field: Field,
}
pub(super) struct PaletteProgramResources {
    data: model::PalettePrograms,
    bindings: Vec<Binding>,
    counters: [AtomicU64; 6],
    programs: [AtomicU64; 4],
    tables: AtomicU64,
}
impl PaletteProgramResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x14d | 0x39a..=0x3a1 | 0x3a4..=0x3a6 | 0x3ac | 0xb45..=0xb4a)
    }
    pub(super) fn new(
        data: model::PalettePrograms,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Palette program source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Palette program typed source binding mismatch".into());
            }
        }
        let mut bindings = Vec::new();
        for (id, p) in data.programs.iter().enumerate() {
            bindings.push(Binding {
                offset: p.source_offset,
                length: 6,
                field: Field::Header(id as u16),
            });
            for (i, c) in p.commands.iter().enumerate() {
                bindings.push(Binding {
                    offset: p.source_offset + 6 + 6 * i as u32,
                    length: 6,
                    field: Field::Command(id as u16, i as u8),
                });
                let length = c.parameter.source_bytes().len();
                if length != 0 {
                    bindings.push(Binding {
                        offset: p.source_offset + u32::from(c.relative_parameter),
                        length,
                        field: Field::Parameter(id as u16, i as u8),
                    });
                }
            }
        }
        for (i, p) in data.producers.iter().enumerate() {
            bindings.push(Binding {
                offset: p.source_offset,
                length: data.producer_bytes(p)?.len(),
                field: Field::Producer(i as u8),
            });
        }
        bindings.sort_by_key(|b| b.offset);
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
            .partition_point(|b| b.offset as usize <= at)
            .checked_sub(1)?;
        let b = &self.bindings[i];
        let n = at - b.offset as usize;
        if n >= b.length {
            return None;
        }
        let (value, category, id) = match b.field {
            Field::Header(id) => (
                self.data.programs[usize::from(id)].header.byte(n)?,
                2,
                Some(id),
            ),
            Field::Command(id, c) => (
                self.data.programs[usize::from(id)].commands[usize::from(c)].record_bytes()[n],
                3,
                Some(id),
            ),
            Field::Parameter(id, c) => (
                self.data.programs[usize::from(id)].commands[usize::from(c)]
                    .parameter
                    .byte(n)?,
                4,
                Some(id),
            ),
            Field::Producer(table) => {
                self.tables.fetch_or(1 << table, Ordering::Relaxed);
                (self.data.producer_byte(usize::from(table), n)?, 5, None)
            }
        };
        if let Some(id) = id {
            self.programs[usize::from(id) / 64].fetch_or(1 << (id % 64), Ordering::Relaxed);
        }
        Some((value, category))
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
        let mut counts = [1, 0, 0, 0, 0, 0];
        let bytes = std::array::from_fn(|i| {
            if let Some((v, category)) = fields[i] {
                counts[1] += 1;
                counts[category] += 1;
                v
            } else {
                prior.as_ref().map_or_else(|| rom[index + i], |v| v[i])
            }
        });
        for (c, n) in self.counters.iter().zip(counts) {
            c.fetch_add(n, Ordering::Relaxed);
        }
        Some(bytes)
    }
    pub(super) fn stats(&self) -> PaletteProgramStats {
        let [bus_reads, source_bytes, header_bytes, command_bytes, parameter_bytes, producer_bytes] =
            self.counters.each_ref().map(|a| a.load(Ordering::Relaxed));
        PaletteProgramStats {
            bus_reads,
            source_bytes,
            header_bytes,
            command_bytes,
            parameter_bytes,
            producer_bytes,
            programs_used: self
                .programs
                .iter()
                .map(|a| u64::from(a.load(Ordering::Relaxed).count_ones()))
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
                .map(|p| {
                    p.commands.capacity() * std::mem::size_of::<model::Command>()
                        + p.commands
                            .iter()
                            .map(|c| match &c.parameter {
                                model::Parameter::ColorWords { colors }
                                | model::Parameter::ColorCycle { colors, .. } => {
                                    colors.capacity() * 2
                                }
                                _ => 0,
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.data.producers.capacity() * std::mem::size_of::<model::Producer>()
            + self
                .data
                .producers
                .iter()
                .map(|p| match &p.data {
                    model::ProducerData::Pointers { programs } => {
                        programs.capacity() * std::mem::size_of::<Option<u16>>()
                    }
                    model::ProducerData::List { entries } => {
                        entries.capacity() * std::mem::size_of::<model::ListEntry>()
                    }
                })
                .sum::<usize>()
    }
}
