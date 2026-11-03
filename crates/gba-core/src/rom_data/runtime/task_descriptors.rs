//! Typed descriptor bytes; original guest ID arithmetic, callbacks and RAM stay live.
use super::*;
use crate::rom_data::task_descriptors as model;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskDescriptorStats {
    pub bus_reads: u64,
    pub source_bytes: u64,
    pub callback_bytes: u64,
    pub context_size_bytes: u64,
    pub metadata_bytes: u64,
    pub records_used: u64,
    pub catalogs_used: u64,
}

pub(super) struct TaskDescriptorResources {
    data: model::TaskDescriptors,
    counters: [AtomicU64; 5],
    records: [AtomicU64; 7],
    catalogs: AtomicU64,
}
impl TaskDescriptorResources {
    pub(super) fn source_page(index: usize) -> bool {
        matches!(index >> 12, 0x399..=0x39a)
    }
    pub(super) fn new(
        data: model::TaskDescriptors,
        sources: &[SourceRange],
    ) -> Result<Self, String> {
        let expected = data.source_spans()?;
        if expected.len() != sources.len() {
            return Err("Task descriptor source binding count mismatch".into());
        }
        for ((offset, bytes, role), source) in expected.into_iter().zip(sources) {
            if source.offset != offset
                || source.length as usize != bytes.len()
                || source.role != role
                || source.sha256 != ffta_rom::digest(&bytes)
            {
                return Err("Task descriptor typed source binding mismatch".into());
            }
        }
        Ok(Self {
            data,
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            records: std::array::from_fn(|_| AtomicU64::new(0)),
            catalogs: AtomicU64::new(0),
        })
    }
    fn byte(&self, at: usize) -> Option<(u8, usize)> {
        let mut first_record = 0;
        for (catalog, table) in self.data.tables().into_iter().enumerate() {
            if let Some(relative) = at.checked_sub(table.source_offset as usize) {
                if let Some(entry) = table.entries.get(relative / 8) {
                    let field = relative % 8;
                    let record = first_record + relative / 8;
                    let byte = entry.byte(field)?;
                    self.records[record / 64].fetch_or(1 << (record % 64), Ordering::Relaxed);
                    self.catalogs.fetch_or(1 << catalog, Ordering::Relaxed);
                    return Some((
                        byte,
                        match field {
                            0..=3 => 2,
                            4..=5 => 3,
                            _ => 4,
                        },
                    ));
                }
            }
            first_record += table.entries.len();
        }
        None
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
    pub(super) fn stats(&self) -> TaskDescriptorStats {
        let [bus_reads, source_bytes, callback_bytes, context_size_bytes, metadata_bytes] =
            self.counters.each_ref().map(|v| v.load(Ordering::Relaxed));
        TaskDescriptorStats {
            bus_reads,
            source_bytes,
            callback_bytes,
            context_size_bytes,
            metadata_bytes,
            records_used: self
                .records
                .iter()
                .map(|v| u64::from(v.load(Ordering::Relaxed).count_ones()))
                .sum(),
            catalogs_used: u64::from(self.catalogs.load(Ordering::Relaxed).count_ones()),
        }
    }
    pub(super) fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .data
                .tables()
                .iter()
                .map(|t| t.entries.capacity() * std::mem::size_of::<model::Descriptor>())
                .sum::<usize>()
    }
}
